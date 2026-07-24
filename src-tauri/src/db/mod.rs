pub mod models;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::str::FromStr;

/// Connects to the given SQLite database URL and runs all pending migrations.
pub async fn connect(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);
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
    async fn migrations_run_and_lesson_attempt_roundtrip() {
        let pool = connect("sqlite::memory:")
            .await
            .expect("migrations should run cleanly on a fresh in-memory db");

        sqlx::query(
            "INSERT INTO lessons (id, title, level, audio_url, transcript, published_at, guid, source_show, word_count)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("lesson-1")
        .bind("Sew and Knit")
        .bind("A2")
        .bind("https://voa-audio.voanews.eu/sew-and-knit.mp3")
        .bind("This is a transcript.")
        .bind("2026-07-01T00:00:00Z")
        .bind("guid-1")
        .bind("As It Is")
        .bind(4_i64)
        .execute(&pool)
        .await
        .expect("lesson insert should succeed");

        let lesson: models::Lesson = sqlx::query_as("SELECT * FROM lessons WHERE id = ?")
            .bind("lesson-1")
            .fetch_one(&pool)
            .await
            .expect("lesson should be queryable back");
        assert_eq!(lesson.guid, "guid-1");
        assert_eq!(lesson.word_count, 4);

        sqlx::query(
            "INSERT INTO attempts (lesson_id, accuracy, attempted_at, user_transcript, correct_count, missing_count, extra_count)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("lesson-1")
        .bind(0.75_f64)
        .bind("2026-07-02T00:00:00Z")
        .bind("this is a transcript")
        .bind(3_i64)
        .bind(1_i64)
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("attempt insert should succeed");

        let attempt_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM attempts WHERE lesson_id = ?")
                .bind("lesson-1")
                .fetch_one(&pool)
                .await
                .expect("attempt count query should succeed");
        assert_eq!(attempt_count, 1);

        // Duplicate guid must be rejected by the unique index added in 0002_refine.sql.
        let dup = sqlx::query(
            "INSERT INTO lessons (id, title, level, audio_url, transcript, published_at, guid, source_show, word_count)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("lesson-2")
        .bind("Another Episode")
        .bind("B1")
        .bind("https://voa-audio.voanews.eu/another.mp3")
        .bind("Another transcript.")
        .bind("2026-07-03T00:00:00Z")
        .bind("guid-1")
        .bind("Words and Their Stories")
        .bind(2_i64)
        .execute(&pool)
        .await;
        assert!(dup.is_err(), "duplicate guid should violate unique index");
    }
}
