// Tauri command: fetch new VOA episodes (RSS + transcript) and upsert into SQLite
#[tauri::command]
pub async fn fetch_new_lessons() -> Result<(), String> {
    Ok(())
}
