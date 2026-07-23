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

/// Real VOA ingestion lands in Phase 4 (RSS + transcript scraping). Until then this keeps
/// the IPC contract stable so the frontend can wire the "refresh" action end-to-end now.
#[tauri::command]
pub async fn fetch_new_lessons(_pool: State<'_, SqlitePool>) -> Result<FetchResult, AppError> {
    Ok(FetchResult { new: 0 })
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
