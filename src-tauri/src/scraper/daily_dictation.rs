//! dailydictation.com ingestion. Verified by hand via curl on 2026-07-24 (robots.txt allows
//! `/exercises`; disallows `/api/`, `/fb`, `/google`, `/discussion`, none of which this touches).
//!
//! Each exercise page embeds a `window.appGlobals = {...};` JSON blob (used to hydrate the
//! site's own React "appDictation" bundle) containing the lesson's audio URL and, per sentence,
//! exact `timeStart`/`timeEnd` cut points into that one audio file — real authored timestamps,
//! not something we estimate. Some categories (stories-for-kids, news, ted-ed, youtube,
//! english-pronunciation) embed a YouTube video instead of an `<audio>`-playable mp3
//! (`audioSrc` is empty there); those are skipped since this app only plays local files.
//!
//! Category → CEFR level has no source of truth on the site (it doesn't publish CEFR labels),
//! so it's a hand-picked mapping based on eyeballing real sentences from each category.

use crate::error::AppError;
use crate::scraper::{get_text_with_retry, Category, ScrapedLesson, ScrapedSegment};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;

pub const CATEGORIES: &[Category] = &[
    // "0 to 10", "counting by 5s" — simplest content on the site.
    Category { slug: "numbers", level: "A1" },
    // Short Q&A exchanges, e.g. "Where is Jane?" / "She is in the living room."
    Category { slug: "english-conversations", level: "A1" },
    // Simple graded-reader narration, e.g. "Today is November 26th. It snowed all day today."
    Category { slug: "short-stories", level: "A2" },
    Category { slug: "toefl-listening", level: "B1" },
    Category { slug: "ielts-listening", level: "B1" },
    Category { slug: "toeic", level: "B1" },
];

#[derive(Debug, Deserialize)]
struct Challenge {
    content: String,
    #[serde(rename = "timeStart")]
    time_start: f64,
    #[serde(rename = "timeEnd")]
    time_end: f64,
}

#[derive(Debug, Deserialize)]
struct AppGlobals {
    #[serde(rename = "lessonId")]
    lesson_id: i64,
    #[serde(rename = "lessonName")]
    lesson_name: String,
    #[serde(rename = "audioSrc")]
    audio_src: String,
    challenges: Vec<Challenge>,
}

static LOC_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<loc>([^<]+)</loc>").unwrap());
// `(?s)` so `.` also matches newlines, in case the blob is ever pretty-printed across lines —
// on the real page it's a single line, but this is cheap insurance against that changing.
static APP_GLOBALS_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)window\.appGlobals\s*=\s*(\{.*?\});\s*</script>").unwrap());
// The exercise URL embeds the lesson id right before "/listen-and-type", e.g.
// ".../1-at-home-1.399/listen-and-type" -> "399". Lets fetch_new_lessons skip a page fetch
// entirely for lessons it already has, instead of re-downloading every known page every refresh.
static LESSON_ID_IN_URL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.(\d+)/listen-and-type/?$").unwrap());

pub fn lesson_id_from_url(url: &str) -> Option<String> {
    LESSON_ID_IN_URL_RE.captures(url).map(|c| c[1].to_string())
}

/// Delay `fetch_new_lessons` should sleep between exercise page fetches. Confirmed by hand:
/// hammering real exercise URLs back-to-back with no delay gets rate-limited (HTTP 429) after
/// ~50-55 requests, while 200-350ms of spacing ran 40-50 requests straight through with none.
pub const REQUEST_PACING: std::time::Duration = std::time::Duration::from_millis(300);

/// Lists every exercise URL in a category via its sitemap (not the paginated HTML browse page —
/// the sitemap gives the full catalog in one request).
pub async fn fetch_category_urls(client: &reqwest::Client, slug: &str) -> Result<Vec<String>, AppError> {
    let url = format!("https://dailydictation.com/sitemap.exercises-{slug}.xml");
    let body = get_text_with_retry(client, &url).await?;
    Ok(parse_sitemap_urls(&body))
}

pub fn parse_sitemap_urls(xml: &str) -> Vec<String> {
    LOC_RE.captures_iter(xml).map(|c| c[1].to_string()).collect()
}

/// Fetches one exercise page and extracts its lesson + segments. Returns `None` (not an error)
/// for lessons with no downloadable audio (YouTube-embedded categories) or malformed pages —
/// same "skip, don't fail the whole refresh" policy the old VOA ingestion used.
pub async fn fetch_exercise(client: &reqwest::Client, url: &str) -> Result<Option<ScrapedLesson>, AppError> {
    let html = get_text_with_retry(client, url).await?;
    Ok(extract_lesson(&html, url))
}

pub fn extract_lesson(html: &str, page_url: &str) -> Option<ScrapedLesson> {
    let captures = APP_GLOBALS_RE.captures(html)?;
    let data: AppGlobals = serde_json::from_str(&captures[1]).ok()?;

    if data.audio_src.is_empty() || data.challenges.is_empty() {
        return None;
    }

    let segments = data
        .challenges
        .into_iter()
        .enumerate()
        .map(|(i, c)| ScrapedSegment {
            position: i as i64,
            content: c.content,
            time_start: c.time_start,
            time_end: c.time_end,
        })
        .collect();

    Some(ScrapedLesson {
        id: data.lesson_id.to_string(),
        title: data.lesson_name,
        audio_url: data.audio_src,
        page_url: page_url.to_string(),
        segments,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real exercise page ("1. At home (1)", english-conversations), fetched by hand on
    // 2026-07-24 — see the dailydictation source-switch discussion in the repo history.
    const EXERCISE_HTML: &str = include_str!("../../tests/fixtures/dailydictation_at_home.html");
    const SITEMAP_XML: &str = include_str!("../../tests/fixtures/dailydictation_sitemap_sample.xml");

    #[test]
    fn extracts_lesson_and_segments_from_real_exercise_page() {
        let lesson = extract_lesson(
            EXERCISE_HTML,
            "https://dailydictation.com/exercises/english-conversations/1-at-home-1.399/listen-and-type",
        )
        .expect("real exercise page should parse");

        assert_eq!(lesson.id, "399");
        assert_eq!(lesson.title, "1. At home (1)");
        assert!(lesson.audio_url.ends_with("1-at-home.mp3"));
        assert_eq!(lesson.segments.len(), 10);

        let first = &lesson.segments[0];
        assert_eq!(first.position, 0);
        assert_eq!(first.content, "Where is Jane?");
        assert_eq!(first.time_start, 3.39);
        assert_eq!(first.time_end, 4.88);

        let last = &lesson.segments[9];
        assert_eq!(last.position, 9);
        assert_eq!(last.content, "The dog is eating.");
    }

    #[test]
    fn returns_none_for_page_without_app_globals() {
        assert!(extract_lesson("<html><body>no data here</body></html>", "https://example.com").is_none());
    }

    #[test]
    fn returns_none_for_youtube_backed_lesson_with_no_audio_src() {
        let html = r#"<script>window.appGlobals = {"lessonId":1,"lessonName":"x","audioSrc":"","challenges":[{"content":"hi","timeStart":0.0,"timeEnd":1.0}]};</script>"#;
        assert!(extract_lesson(html, "https://example.com").is_none());
    }

    #[test]
    fn parses_sitemap_urls_from_real_sitemap() {
        let urls = parse_sitemap_urls(SITEMAP_XML);
        assert_eq!(urls.len(), 100);
        assert_eq!(
            urls[0],
            "https://dailydictation.com/exercises/english-conversations/1-at-home-1.399/listen-and-type"
        );
    }

    #[test]
    fn extracts_lesson_id_from_exercise_url() {
        assert_eq!(
            lesson_id_from_url("https://dailydictation.com/exercises/english-conversations/1-at-home-1.399/listen-and-type"),
            Some("399".to_string())
        );
        assert_eq!(lesson_id_from_url("https://example.com/not-a-match"), None);
    }
}
