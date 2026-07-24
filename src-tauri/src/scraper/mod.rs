pub mod daily_dictation;
pub mod ted;

use crate::error::AppError;

/// dailydictation.com doesn't appear to bot-block, but VOA (the previous source) did without
/// a browser-like User-Agent, so this is kept as a defensive default for all outbound fetches.
pub const CHROME_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

/// One content source's category → CEFR level mapping. Each source (`daily_dictation`, `ted`)
/// defines its own list of these; neither site publishes real CEFR labels, so both are hand-picked.
#[derive(Debug, Clone, Copy)]
pub struct Category {
    pub slug: &'static str,
    pub level: &'static str,
}

/// One dictated/captioned line within a lesson, with its cut points into the lesson's single
/// audio file. For dailydictation these are authored (exact); for ted they're inferred from
/// caption start times (see scraper::ted) — either way this is the shape `content.rs` persists.
#[derive(Debug, Clone)]
pub struct ScrapedSegment {
    /// 0-based, matching this app's existing `attempts.segment_index` convention.
    pub position: i64,
    pub content: String,
    pub time_start: f64,
    pub time_end: f64,
}

#[derive(Debug, Clone)]
pub struct ScrapedLesson {
    /// Prefixed per source (e.g. "ted-...") so ids from different sources can't collide in the
    /// `lessons` table — dailydictation's own ids are left unprefixed for backward compatibility
    /// with rows already ingested before this app supported more than one source.
    pub id: String,
    pub title: String,
    pub audio_url: String,
    pub page_url: String,
    pub segments: Vec<ScrapedSegment>,
}

const MAX_RATE_LIMIT_RETRIES: u32 = 5;
const DEFAULT_RETRY_AFTER: std::time::Duration = std::time::Duration::from_secs(3);

/// GETs `url` as text, retrying on HTTP 429 — honors `Retry-After` when the response sends one,
/// otherwise backs off a flat few seconds. Shared by every scraper source; each source is still
/// responsible for pacing its own requests between calls (see each module's `REQUEST_PACING`).
pub(crate) async fn get_text_with_retry(client: &reqwest::Client, url: &str) -> Result<String, AppError> {
    for attempt in 0..=MAX_RATE_LIMIT_RETRIES {
        let response = client
            .get(url)
            .header("User-Agent", CHROME_USER_AGENT)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS && attempt < MAX_RATE_LIMIT_RETRIES {
            let wait = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .map(std::time::Duration::from_secs)
                .unwrap_or(DEFAULT_RETRY_AFTER);
            tokio::time::sleep(wait).await;
            continue;
        }

        return Ok(response.error_for_status()?.text().await?);
    }
    unreachable!("loop always returns on its last iteration (attempt == MAX_RATE_LIMIT_RETRIES)")
}
