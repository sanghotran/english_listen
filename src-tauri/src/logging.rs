//! Minimal file logger for release builds, where the console window is hidden
//! (see `main.rs`'s `windows_subsystem` attribute) and stderr has nowhere to go.
//! Writes next to the executable, alongside the db and audio cache.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

fn log_path() -> PathBuf {
    crate::db::portable_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("english-listen.log")
}

fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Truncates the log file for this run and installs a panic hook that appends
/// every panic (message + file:line) to it, in addition to the default hook
/// (which still prints to stderr when a console is attached, e.g. in `cargo tauri dev`).
/// Call once, before `tauri::Builder::default()`.
pub fn init() {
    if let Ok(mut f) = OpenOptions::new().create(true).write(true).truncate(true).open(log_path()) {
        let _ = writeln!(f, "=== English Listen started {} ===", now());
    }

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        append(&format!("PANIC: {info}"));
        default_hook(info);
    }));
}

/// Appends a single timestamped line to the log file. Best-effort: a logging
/// failure must never crash the app it's trying to diagnose.
pub fn append(line: &str) {
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(log_path()) {
        let _ = writeln!(f, "[{}] {}", now(), line);
    }
}
