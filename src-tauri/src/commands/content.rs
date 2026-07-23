use crate::db::models::{Attempt, Lesson, LevelProgress};
use crate::diff::{self, WordStatus};
use crate::error::AppError;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::State;

async fn fetch_lesson(pool: &SqlitePool, id: &str) -> Result<Lesson, AppError> {
    sqlx::query_as::<_, Lesson>("SELECT * FROM lessons WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("lesson '{id}' not found")))
}

#[tauri::command]
pub async fn list_lessons(pool: State<'_, SqlitePool>) -> Result<Vec<Lesson>, AppError> {
    let lessons = sqlx::query_as::<_, Lesson>("SELECT * FROM lessons ORDER BY published_at DESC")
        .fetch_all(pool.inner())
        .await?;
    Ok(lessons)
}

#[tauri::command]
pub async fn get_lesson(pool: State<'_, SqlitePool>, id: String) -> Result<Lesson, AppError> {
    fetch_lesson(pool.inner(), &id).await
}

#[derive(Debug, Serialize)]
pub struct FetchResult {
    pub new: u32,
}

/// Pulls the 4 confirmed VOA show feeds, opens each episode page for audio+transcript, and
/// upserts by `guid` — re-running this is naturally idempotent (a second run reports `new: 0`
/// once nothing has changed upstream), so no separate "last seen" bookkeeping is needed.
#[tauri::command]
pub async fn fetch_new_lessons(pool: State<'_, SqlitePool>) -> Result<FetchResult, AppError> {
    let client = reqwest::Client::new();

    let existing_guids: std::collections::HashSet<String> =
        sqlx::query_scalar("SELECT guid FROM lessons")
            .fetch_all(pool.inner())
            .await?
            .into_iter()
            .collect();

    let mut new_count: u32 = 0;

    for feed in crate::scraper::feeds::FEEDS {
        let items = match crate::scraper::voa_rss::fetch_feed(&client, &feed.url()).await {
            Ok(items) => items,
            // One unreachable feed shouldn't abort the whole refresh.
            Err(_) => continue,
        };

        for item in items {
            let episode = match crate::scraper::transcript::fetch_episode(&client, &item.link).await {
                Ok(Some(episode)) => episode,
                // Missing audio/transcript (video-only, audio-only items) or a fetch failure:
                // skip, not an error — see docs/PLAN.md Phase 4.
                Ok(None) | Err(_) => continue,
            };

            let word_count = episode.transcript.split_whitespace().count() as i64;
            let is_new = !existing_guids.contains(&item.guid);

            sqlx::query(
                "INSERT INTO lessons (id, title, level, audio_url, transcript, published_at, guid, source_show, word_count)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(guid) DO UPDATE SET
                    title = excluded.title,
                    level = excluded.level,
                    audio_url = excluded.audio_url,
                    transcript = excluded.transcript,
                    published_at = excluded.published_at,
                    source_show = excluded.source_show,
                    word_count = excluded.word_count",
            )
            .bind(&item.guid)
            .bind(&item.title)
            .bind(feed.level)
            .bind(&episode.audio_url)
            .bind(&episode.transcript)
            .bind(&item.pub_date)
            .bind(&item.guid)
            .bind(feed.show)
            .bind(word_count)
            .execute(pool.inner())
            .await?;

            if is_new {
                new_count += 1;
            }
        }
    }

    Ok(FetchResult { new: new_count })
}

#[tauri::command]
pub async fn record_attempt(
    pool: State<'_, SqlitePool>,
    lesson_id: String,
    user_transcript: String,
) -> Result<Attempt, AppError> {
    let lesson = fetch_lesson(pool.inner(), &lesson_id).await?;

    let tokens = diff::diff_words(&user_transcript, &lesson.transcript);
    let accuracy = diff::compute_accuracy(&tokens);
    let correct_count = tokens.iter().filter(|t| t.status == WordStatus::Correct).count() as i64;
    let missing_count = tokens.iter().filter(|t| t.status == WordStatus::Missing).count() as i64;
    let extra_count = tokens.iter().filter(|t| t.status == WordStatus::Extra).count() as i64;
    let attempted_at = chrono::Utc::now().to_rfc3339();

    let attempt = sqlx::query_as::<_, Attempt>(
        "INSERT INTO attempts
            (lesson_id, accuracy, attempted_at, user_transcript, correct_count, missing_count, extra_count)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         RETURNING *",
    )
    .bind(&lesson_id)
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
/// aggregate update is ever missed — see docs/PLAN.md Phase 2.
#[tauri::command]
pub async fn get_level_progress(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<LevelProgress>, AppError> {
    let progress = sqlx::query_as::<_, LevelProgress>(
        "SELECT l.level AS level,
                COUNT(DISTINCT a.lesson_id) AS lessons_attempted,
                COALESCE(AVG(a.accuracy), 0.0) AS avg_accuracy
         FROM lessons l
         LEFT JOIN attempts a ON a.lesson_id = l.id
         GROUP BY l.level
         ORDER BY l.level",
    )
    .fetch_all(pool.inner())
    .await?;
    Ok(progress)
}
