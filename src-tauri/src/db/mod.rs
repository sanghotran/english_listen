pub mod models;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::str::FromStr;

/// Connects to the given SQLite database URL and runs all pending migrations.
///
/// `foreign_keys(false)`: this app never deletes a `lessons` row (no such command exists), so FK
/// enforcement was never protecting anything here — and it actively breaks the table-rebuild
/// dance migrations like `0006_b2_level.sql` use to widen a CHECK constraint (that pattern needs
/// `PRAGMA legacy_alter_table`, which itself only has an effect while `foreign_keys` is off; since
/// `foreign_keys` is a no-op to toggle *inside* a transaction and sqlx runs each migration in one,
/// it has to already be off before migrations start, not toggled from within a migration file).
pub async fn connect(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(false);
    let pool = SqlitePoolOptions::new().connect_with(options).await?;
    sqlx::migrate!("./src/db/migrations").run(&pool).await?;
    Ok(pool)
}

/// Directory the app stores its data in: the folder containing the running executable
/// (portable install — data travels with the `.exe`, not the OS's per-user app data dir).
///
/// Note: `tauri::path::PathResolver::executable_dir` is NOT usable for this — on Windows it
/// resolves via the `dirs` crate, which returns `None` there (it's meant for a user's installed
/// binaries dir, e.g. `~/.local/bin` on Linux). We use `std::env::current_exe()` directly instead.
pub fn portable_data_dir() -> std::io::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    Ok(exe
        .parent()
        .expect("executable path must have a parent directory")
        .to_path_buf())
}

/// Ensures the `audio/` cache dir exists next to the executable, and opens (creating if needed)
/// `english_listen.db` there. Returns the resolved data dir so callers can register it with the
/// asset protocol scope.
pub async fn init_pool() -> Result<(SqlitePool, PathBuf), sqlx::Error> {
    let data_dir = portable_data_dir().expect("failed to resolve executable directory");
    std::fs::create_dir_all(&data_dir).expect("failed to create data dir");
    std::fs::create_dir_all(data_dir.join("audio")).expect("failed to create audio cache dir");

    let db_path: PathBuf = data_dir.join("english_listen.db");
    let pool = connect(&format!("sqlite://{}", db_path.display())).await?;
    Ok((pool, data_dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn migrations_run_and_lesson_segment_attempt_roundtrip() {
        let pool = connect("sqlite::memory:")
            .await
            .expect("migrations should run cleanly on a fresh in-memory db");

        sqlx::query(
            "INSERT INTO lessons (id, title, level, category, audio_url, page_url, published_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("399")
        .bind("1. At home (1)")
        .bind("A1")
        .bind("english-conversations")
        .bind("https://dailydictation.com/upload/.../1-at-home.mp3")
        .bind("https://dailydictation.com/exercises/english-conversations/1-at-home-1.399/listen-and-type")
        .bind("2026-07-24T00:00:00Z")
        .execute(&pool)
        .await
        .expect("lesson insert should succeed");

        sqlx::query(
            "INSERT INTO segments (lesson_id, position, content, time_start, time_end)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind("399")
        .bind(0_i64)
        .bind("Where is Jane?")
        .bind(3.39_f64)
        .bind(4.88_f64)
        .execute(&pool)
        .await
        .expect("segment insert should succeed");

        let lesson: models::Lesson = sqlx::query_as("SELECT * FROM lessons WHERE id = ?")
            .bind("399")
            .fetch_one(&pool)
            .await
            .expect("lesson should be queryable back");
        assert_eq!(lesson.category, "english-conversations");

        let segment: models::Segment = sqlx::query_as("SELECT * FROM segments WHERE lesson_id = ? AND position = ?")
            .bind("399")
            .bind(0_i64)
            .fetch_one(&pool)
            .await
            .expect("segment should be queryable back");
        assert_eq!(segment.content, "Where is Jane?");

        sqlx::query(
            "INSERT INTO attempts (lesson_id, segment_index, accuracy, attempted_at, user_transcript, correct_count, missing_count, extra_count)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("399")
        .bind(0_i64)
        .bind(0.75_f64)
        .bind("2026-07-24T00:01:00Z")
        .bind("where is jane")
        .bind(3_i64)
        .bind(0_i64)
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("attempt insert should succeed");

        let attempt_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM attempts WHERE lesson_id = ?")
                .bind("399")
                .fetch_one(&pool)
                .await
                .expect("attempt count query should succeed");
        assert_eq!(attempt_count, 1);

        // Duplicate (lesson_id, position) must be rejected by the unique index on segments.
        let dup = sqlx::query(
            "INSERT INTO segments (lesson_id, position, content, time_start, time_end)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind("399")
        .bind(0_i64)
        .bind("Duplicate position.")
        .bind(0.0_f64)
        .bind(1.0_f64)
        .execute(&pool)
        .await;
        assert!(dup.is_err(), "duplicate (lesson_id, position) should violate unique index");
    }
}
