use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Lesson {
    pub id: String,
    pub title: String,
    pub level: String,
    pub audio_url: String,
    pub local_audio_path: Option<String>,
    pub transcript: String,
    pub published_at: String,
    pub guid: String,
    pub source_show: String,
    pub word_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Attempt {
    pub id: i64,
    pub lesson_id: String,
    pub accuracy: f64,
    pub attempted_at: String,
    pub user_transcript: String,
    pub correct_count: i64,
    pub missing_count: i64,
    pub extra_count: i64,
}

/// Not a table — computed on the fly via GROUP BY over `attempts` joined with `lessons`.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LevelProgress {
    pub level: String,
    pub lessons_attempted: i64,
    pub avg_accuracy: f64,
}
