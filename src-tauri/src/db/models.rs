use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Lesson {
    pub id: String,
    pub title: String,
    pub level: String,
    pub category: String,
    pub audio_url: String,
    pub local_audio_path: Option<String>,
    pub page_url: String,
    pub published_at: String,
}

/// One dictated sentence within a lesson, with its exact cut points into the lesson's single
/// audio file (authored by the source, not estimated — see scraper::daily_dictation).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub id: i64,
    pub lesson_id: String,
    pub position: i64,
    pub content: String,
    pub time_start: f64,
    pub time_end: f64,
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

/// Not a table — computed on the fly via GROUP BY over `segments` left-joined with `attempts`.
/// `completion` is the fraction (0.0-1.0) of a lesson's segments that have at least one recorded
/// attempt; used by the frontend to badge lesson cards and sort finished ones to the bottom.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LessonProgress {
    pub lesson_id: String,
    pub completion: f64,
}
