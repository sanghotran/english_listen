mod commands;
mod db;
mod diff;
mod error;
mod scraper;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let pool = tauri::async_runtime::block_on(db::init_pool(&handle))?;
            app.manage(pool);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::content::list_lessons,
            commands::content::get_lesson,
            commands::content::fetch_new_lessons,
            commands::content::record_attempt,
            commands::content::list_attempts,
            commands::content::get_level_progress,
            commands::audio::download_audio,
            commands::audio::get_lesson_audio_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
