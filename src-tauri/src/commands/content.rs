use crate::db::models::{Attempt, Lesson, LessonProgress, LevelProgress, Segment};
use crate::diff::{self, WordStatus};
use crate::error::AppError;
use crate::scraper::{daily_dictation, ted, Category, ScrapedLesson};
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, State};

async fn fetch_lesson(pool: &SqlitePool, id: &str) -> Result<Lesson, AppError> {
    sqlx::query_as::<_, Lesson>("SELECT * FROM lessons WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("lesson '{id}' not found")))
}

/// Lets the frontend forward uncaught render errors (see `ErrorBoundary`) into the same
/// file log Rust panics go to — otherwise a blank-page JS crash leaves zero trace anywhere.
#[tauri::command]
pub fn log_frontend_error(message: String) {
    crate::logging::append(&format!("FRONTEND ERROR: {message}"));
}

#[tauri::command]
pub async fn list_lessons(pool: State<'_, SqlitePool>) -> Result<Vec<Lesson>, AppError> {
    let lessons = sqlx::query_as::<_, Lesson>("SELECT * FROM lessons ORDER BY published_at ASC")
        .fetch_all(pool.inner())
        .await?;
    Ok(lessons)
}

#[tauri::command]
pub async fn get_lesson(pool: State<'_, SqlitePool>, id: String) -> Result<Lesson, AppError> {
    fetch_lesson(pool.inner(), &id).await
}

#[tauri::command]
pub async fn list_segments(pool: State<'_, SqlitePool>, lesson_id: String) -> Result<Vec<Segment>, AppError> {
    let segments = sqlx::query_as::<_, Segment>(
        "SELECT * FROM segments WHERE lesson_id = ? ORDER BY position",
    )
    .bind(&lesson_id)
    .fetch_all(pool.inner())
    .await?;
    Ok(segments)
}

#[derive(Debug, Serialize)]
pub struct FetchResult {
    pub new: u32,
}

/// Emitted to the frontend (`lessons-refresh-progress`) after every exercise URL processed by
/// `fetch_new_lessons`, since a full first run walks ~1000+ pages at a paced rate and can take
/// several minutes — without this the "Refresh" button just looks stuck the whole time.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RefreshProgress {
    processed: u32,
    total: u32,
    new_count: u32,
    category: &'static str,
}

/// Inserts one already-fetched lesson + its segments. Shared by every source's ingestion
/// function below — the sources differ in how a `ScrapedLesson` is obtained, not in how it's
/// persisted. A `?`-propagated `Err` here means a real DB failure, which should abort the
/// refresh rather than be silently skipped.
async fn insert_scraped_lesson(
    pool: &SqlitePool,
    lesson: &ScrapedLesson,
    category: &Category,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO lessons (id, title, level, category, audio_url, page_url, published_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&lesson.id)
    .bind(&lesson.title)
    .bind(category.level)
    .bind(category.slug)
    .bind(&lesson.audio_url)
    .bind(&lesson.page_url)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    for segment in &lesson.segments {
        sqlx::query(
            "INSERT INTO segments (lesson_id, position, content, time_start, time_end)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&lesson.id)
        .bind(segment.position)
        .bind(&segment.content)
        .bind(segment.time_start)
        .bind(segment.time_end)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Inserts one dailydictation.com exercise URL's lesson + segments if it's genuinely new.
/// Returns `Ok(false)` (not an error) for lessons already known, ones with no downloadable audio
/// (YouTube-embedded categories), or a fetch/parse failure — same "skip, don't fail the whole
/// refresh" policy the old VOA ingestion used.
async fn ingest_daily_dictation_url(
    pool: &SqlitePool,
    client: &reqwest::Client,
    category: &Category,
    url: &str,
    existing_ids: &std::collections::HashSet<String>,
) -> Result<bool, AppError> {
    if let Some(id) = daily_dictation::lesson_id_from_url(url) {
        if existing_ids.contains(&id) {
            return Ok(false);
        }
    }

    // Paced to stay under dailydictation.com's rate limit — see REQUEST_PACING.
    tokio::time::sleep(daily_dictation::REQUEST_PACING).await;

    let lesson = match daily_dictation::fetch_exercise(client, url).await {
        Ok(Some(lesson)) => lesson,
        Ok(None) | Err(_) => return Ok(false),
    };

    if existing_ids.contains(&lesson.id) {
        return Ok(false);
    }

    insert_scraped_lesson(pool, &lesson, category).await?;
    Ok(true)
}

/// Inserts one ted.com talk URL's lesson + segments if it's genuinely new. Same skip policy as
/// `ingest_daily_dictation_url` — additionally skips non-English talks and talks whose audio file
/// turns out to be unreachable (see `ted::fetch_talk`).
async fn ingest_ted_url(
    pool: &SqlitePool,
    client: &reqwest::Client,
    category: &Category,
    url: &str,
    existing_ids: &std::collections::HashSet<String>,
) -> Result<bool, AppError> {
    if let Some(id) = ted::lesson_id_from_url(url) {
        if existing_ids.contains(&id) {
            return Ok(false);
        }
    }

    // Paced to stay under ted.com's (untested) rate limit — see ted::REQUEST_PACING.
    tokio::time::sleep(ted::REQUEST_PACING).await;

    let lesson = match ted::fetch_talk(client, url).await {
        Ok(Some(lesson)) => lesson,
        Ok(None) | Err(_) => return Ok(false),
    };

    if existing_ids.contains(&lesson.id) {
        return Ok(false);
    }

    insert_scraped_lesson(pool, &lesson, category).await?;
    Ok(true)
}

/// Walks each configured dailydictation.com category's sitemap plus TED's yearly sitemaps, and
/// ingests exercises/talks this DB doesn't have yet. Unlike the old VOA RSS ingestion (which
/// always re-fetched every episode page to check for updates), a lesson id already in the DB is
/// skipped without even fetching its page — neither source changes content post-publish, and
/// re-fetching everything on every refresh click would make "Refresh" unusably slow after the
/// first run.
#[tauri::command]
pub async fn fetch_new_lessons(pool: State<'_, SqlitePool>, app: AppHandle) -> Result<FetchResult, AppError> {
    let client = reqwest::Client::new();

    let existing_ids: std::collections::HashSet<String> = sqlx::query_scalar("SELECT id FROM lessons")
        .fetch_all(pool.inner())
        .await?
        .into_iter()
        .collect();

    // Discover every URL up front so progress can report a real total instead of one that
    // grows as categories/sitemaps are processed.
    let mut dd_category_urls: Vec<(&Category, Vec<String>)> = Vec::new();
    for category in daily_dictation::CATEGORIES {
        if let Ok(urls) = daily_dictation::fetch_category_urls(&client, category.slug).await {
            dd_category_urls.push((category, urls));
        }
        // One unreachable category sitemap shouldn't abort the whole refresh — it's just
        // absent from dd_category_urls, contributing 0 to the total.
    }

    let mut ted_urls: Vec<String> = Vec::new();
    for sitemap_url in ted::TALK_SITEMAPS {
        if let Ok(urls) = ted::fetch_sitemap_urls(&client, sitemap_url).await {
            ted_urls.extend(urls);
        }
        // Same policy: one unreachable yearly sitemap shouldn't abort the whole refresh.
    }

    let total: u32 = dd_category_urls.iter().map(|(_, urls)| urls.len() as u32).sum::<u32>()
        + ted_urls.len() as u32;

    let mut processed: u32 = 0;
    let mut new_count: u32 = 0;

    for (category, urls) in dd_category_urls {
        for url in &urls {
            let is_new = ingest_daily_dictation_url(pool.inner(), &client, category, url, &existing_ids).await?;
            processed += 1;
            if is_new {
                new_count += 1;
            }
            // Best-effort: a dropped progress event shouldn't abort the refresh.
            let _ = app.emit(
                "lessons-refresh-progress",
                RefreshProgress { processed, total, new_count, category: category.slug },
            );
        }
    }

    for url in &ted_urls {
        let is_new = ingest_ted_url(pool.inner(), &client, &ted::CATEGORY, url, &existing_ids).await?;
        processed += 1;
        if is_new {
            new_count += 1;
        }
        let _ = app.emit(
            "lessons-refresh-progress",
            RefreshProgress { processed, total, new_count, category: ted::CATEGORY.slug },
        );
    }

    Ok(FetchResult { new: new_count })
}

#[tauri::command]
pub async fn record_attempt(
    pool: State<'_, SqlitePool>,
    lesson_id: String,
    segment_index: i64,
    user_transcript: String,
) -> Result<Attempt, AppError> {
    // Never trust a reference text the client could send directly — look the segment's
    // authored content up server-side by (lesson_id, position).
    let segment: (String,) = sqlx::query_as("SELECT content FROM segments WHERE lesson_id = ? AND position = ?")
        .bind(&lesson_id)
        .bind(segment_index)
        .fetch_optional(pool.inner())
        .await?
        .ok_or_else(|| AppError::NotFound(format!("segment {segment_index} of lesson '{lesson_id}' not found")))?;

    let tokens = diff::diff_words(&user_transcript, &segment.0);
    let accuracy = diff::compute_accuracy(&tokens);
    let correct_count = tokens.iter().filter(|t| t.status == WordStatus::Correct).count() as i64;
    let missing_count = tokens.iter().filter(|t| t.status == WordStatus::Missing).count() as i64;
    let extra_count = tokens.iter().filter(|t| t.status == WordStatus::Extra).count() as i64;
    let attempted_at = chrono::Utc::now().to_rfc3339();

    let attempt = sqlx::query_as::<_, Attempt>(
        "INSERT INTO attempts
            (lesson_id, segment_index, accuracy, attempted_at, user_transcript, correct_count, missing_count, extra_count)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         RETURNING *",
    )
    .bind(&lesson_id)
    .bind(segment_index)
    .bind(accuracy)
    .bind(&attempted_at)
    .bind(&user_transcript)
    .bind(correct_count)
    .bind(missing_count)
    .bind(extra_count)
    .fetch_one(pool.inner())
    .await?;

    Ok(attempt)
}

#[tauri::command]
pub async fn list_attempts(
    pool: State<'_, SqlitePool>,
    lesson_id: String,
) -> Result<Vec<Attempt>, AppError> {
    let attempts = sqlx::query_as::<_, Attempt>(
        "SELECT * FROM attempts WHERE lesson_id = ? ORDER BY attempted_at DESC",
    )
    .bind(&lesson_id)
    .fetch_all(pool.inner())
    .await?;
    Ok(attempts)
}

/// Aggregated on the fly (GROUP BY) rather than a materialized table, to avoid drift if an
/// aggregate update is ever missed.
#[tauri::command]
pub async fn get_level_progress(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<LevelProgress>, AppError> {
    let progress = sqlx::query_as::<_, LevelProgress>(
        "SELECT l.level AS level,
                COUNT(DISTINCT a.lesson_id) AS lessons_completed,
                COALESCE(AVG(a.accuracy), 0.0) AS average_accuracy
         FROM lessons l
         LEFT JOIN attempts a ON a.lesson_id = l.id
         GROUP BY l.level
         ORDER BY l.level",
    )
    .fetch_all(pool.inner())
    .await?;
    Ok(progress)
}

/// Aggregated on the fly (GROUP BY), same rationale as `get_level_progress`. A segment counts
/// as done once it has at least one attempt, regardless of accuracy.
#[tauri::command]
pub async fn get_lesson_progress(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<LessonProgress>, AppError> {
    let progress = sqlx::query_as::<_, LessonProgress>(
        "SELECT s.lesson_id AS lesson_id,
                CAST(COUNT(DISTINCT a.segment_index) AS REAL) / COUNT(DISTINCT s.position) AS completion
         FROM segments s
         LEFT JOIN attempts a ON a.lesson_id = s.lesson_id AND a.segment_index = s.position
         GROUP BY s.lesson_id",
    )
    .fetch_all(pool.inner())
    .await?;
    Ok(progress)
}
