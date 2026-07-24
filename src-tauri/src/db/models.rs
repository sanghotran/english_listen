use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
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
    pub page_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Attempt {
    pub id: i64,
    pub lesson_id: String,
    pub segment_index: i64,
    pub accuracy: f64,
    pub attempted_at: String,
    pub user_transcript: String,
    pub correct_count: i64,
    pub missing_count: i64,
    pub extra_count: i64,
}

/// Not a table — computed on the fly via GROUP BY over `attempts` joined with `lessons`.
/// Field names/semantics match the frontend's `LevelProgress` type (src/types/progress.ts):
/// a lesson counts as "completed" once it has at least one recorded attempt.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LevelProgress {
    pub level: String,
    pub lessons_completed: i64,
    pub average_accuracy: f64,
}
