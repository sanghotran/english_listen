use crate::db;
use crate::error::AppError;
use crate::scraper::CHROME_USER_AGENT;
use futures_util::StreamExt;
use sqlx::SqlitePool;
use tauri::State;
use tokio::io::AsyncWriteExt;

/// `lesson_id` is a source's lesson id (e.g. dailydictation's numeric "399", or "ted-<slug>") —
/// this is just a defensive flattening so a stray `/` or `:` can't be read by `Path::join` as a
/// nested (non-existent) directory.
fn safe_filename(lesson_id: &str) -> String {
    lesson_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

/// The audio file's extension, taken from its URL — dailydictation serves mp3, TED serves
/// progressive mp4 (see scraper::ted). Falls back to "mp3" if the URL's last path segment has no
/// dot, rather than guessing wrong; every source seen so far has a real extension in its URL.
fn audio_extension(audio_url: &str) -> &str {
    let path = audio_url.split(['?', '#']).next().unwrap_or(audio_url);
    path.rsplit('/')
        .next()
        .and_then(|last_segment| last_segment.rsplit_once('.'))
        .map(|(_, ext)| ext)
        .unwrap_or("mp3")
}

/// Streams the lesson's audio to `{exe_dir}/audio/{safe_filename(lesson_id)}.{ext}` and records
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
    let ext = audio_extension(&audio_url);
    let file_path = audio_dir.join(format!("{}.{}", safe_filename(&lesson_id), ext));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_extension_reads_dailydictation_mp3() {
        assert_eq!(
            audio_extension("https://dailydictation.com/upload/audio/1-at-home.mp3"),
            "mp3"
        );
    }

    #[test]
    fn audio_extension_reads_ted_mp4() {
        assert_eq!(
            audio_extension(
                "https://py.tedcdn.com/consus/projects/00/08/98/007/products/2008-helen-fisher-007-fallback-12d69c8803a25b830a2c29904f64693b-1200k.mp4"
            ),
            "mp4"
        );
    }

    #[test]
    fn audio_extension_ignores_query_string() {
        assert_eq!(audio_extension("https://example.com/audio.mp4?token=abc"), "mp4");
    }

    #[test]
    fn audio_extension_falls_back_to_mp3_without_a_dot() {
        assert_eq!(audio_extension("https://example.com/audio"), "mp3");
    }
}
