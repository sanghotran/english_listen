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
            let (pool, data_dir) = tauri::async_runtime::block_on(db::init_pool())?;
            app.manage(pool);
            // Audio files are cached at {exe_dir}/audio — allow that exact runtime-resolved
            // path since it can't be expressed as a static glob in tauri.conf.json (there is
            // no "next to the exe" path variable; see db::portable_data_dir).
            app.asset_protocol_scope()
                .allow_directory(data_dir.join("audio"), false)?;
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
