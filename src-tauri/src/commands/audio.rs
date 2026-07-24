use crate::db;
use crate::error::AppError;
use crate::scraper::CHROME_USER_AGENT;
use futures_util::StreamExt;
use sqlx::SqlitePool;
use tauri::State;
use tokio::io::AsyncWriteExt;

/// `lesson_id` is dailydictation.com's numeric lesson id (a plain string, e.g. "399") — this is
/// just a defensive flattening in case that ever stops holding, so a stray `/` or `:` can't be
/// read by `Path::join` as a nested (non-existent) directory.
fn safe_filename(lesson_id: &str) -> String {
    lesson_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

/// Streams the lesson's mp3 to `{exe_dir}/audio/{safe_filename(lesson_id)}.mp3` and records
/// the local path — the frontend never opens the source URL directly in the webview (see
/// `get_lesson_audio_path`).
#[tauri::command]
pub async fn download_audio(
    pool: State<'_, SqlitePool>,
    lesson_id: String,
) -> Result<(), AppError> {
    let audio_url: Option<(String,)> = sqlx::query_as("SELECT audio_url FROM lessons WHERE id = ?")
        .bind(&lesson_id)
        .fetch_optional(pool.inner())
        .await?;
    let (audio_url,) =
        audio_url.ok_or_else(|| AppError::NotFound(format!("lesson '{lesson_id}' not found")))?;

    let data_dir = db::portable_data_dir()?;
    let audio_dir = data_dir.join("audio");
    tokio::fs::create_dir_all(&audio_dir).await?;
    let file_path = audio_dir.join(format!("{}.mp3", safe_filename(&lesson_id)));

    let response = reqwest::Client::new()
        .get(&audio_url)
        .header("User-Agent", CHROME_USER_AGENT)
        .send()
        .await?
        .error_for_status()?;

    let mut file = tokio::fs::File::create(&file_path).await?;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        file.write_all(&chunk?).await?;
    }

    let path_str = file_path.to_string_lossy().to_string();
    sqlx::query("UPDATE lessons SET local_audio_path = ? WHERE id = ?")
        .bind(&path_str)
        .bind(&lesson_id)
        .execute(pool.inner())
        .await?;

    Ok(())
}

/// Returns the local mp3 path (if downloaded) for the frontend to resolve via `convertFileSrc()` —
/// the webview never opens the source URL directly.
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
