mod commands;
mod db;
mod scraper;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::content::fetch_new_lessons,
            commands::audio::download_audio,
            commands::audio::get_lesson_audio_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
