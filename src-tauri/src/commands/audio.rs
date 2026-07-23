// Tauri commands: download mp3 for offline use, resolve local path for a lesson
#[tauri::command]
pub async fn download_audio(lesson_id: String) -> Result<String, String> {
    Ok(lesson_id)
}

#[tauri::command]
pub async fn get_lesson_audio_path(lesson_id: String) -> Result<Option<String>, String> {
    let _ = lesson_id;
    Ok(None)
}
