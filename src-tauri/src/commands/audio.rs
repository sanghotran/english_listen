use crate::error::AppError;
use sqlx::SqlitePool;
use tauri::State;

/// Real HTTP streaming to `{app_data_dir}/audio/{lesson_id}.mp3` lands in Phase 4. For now
/// this validates the lesson exists so the command's NotFound path is already exercised.
#[tauri::command]
pub async fn download_audio(
    pool: State<'_, SqlitePool>,
    lesson_id: String,
) -> Result<(), AppError> {
    let exists: Option<(String,)> = sqlx::query_as("SELECT id FROM lessons WHERE id = ?")
        .bind(&lesson_id)
        .fetch_optional(pool.inner())
        .await?;
    exists.ok_or_else(|| AppError::NotFound(format!("lesson '{lesson_id}' not found")))?;
    Ok(())
}

/// Returns the local mp3 path (if downloaded) for the frontend to resolve via `convertFileSrc()` —
/// the webview never opens the VOA URL directly.
#[tauri::command]
pub async fn get_lesson_audio_path(
    pool: State<'_, SqlitePool>,
    lesson_id: String,
) -> Result<Option<String>, AppError> {
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT local_audio_path FROM lessons WHERE id = ?")
            .bind(&lesson_id)
            .fetch_optional(pool.inner())
            .await?;
    match row {
        Some((path,)) => Ok(path),
        None => Err(AppError::NotFound(format!("lesson '{lesson_id}' not found"))),
    }
}
