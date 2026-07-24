//! ted.com ingestion. Verified by hand via curl on 2026-07-24: `robots.txt` (`User-agent: *`)
//! allows `/talks/` and `/sitemaps/`; it separately disallows the `ClaudeBot`/`anthropic-ai`
//! user agents sitewide, but that block is scoped to crawlers identifying themselves as such —
//! this module sends the same generic browser User-Agent as every other scraper in this app
//! (see `CHROME_USER_AGENT`), for a personal listening app, not a training-data crawler.
//!
//! Each talk's `/transcript` page embeds a Next.js `__NEXT_DATA__` JSON blob containing:
//! - `transcriptData.translation.paragraphs[].cues[]`: caption lines with a *start* time each
//!   (ms) — unlike dailydictation's `challenges`, TED gives no explicit end time, so it's
//!   inferred here as the next cue's start (or the talk's total duration for the last cue).
//!   Some cues bundle two wrapped caption lines (joined by `\n`) that are really one spoken
//!   phrase, not two separate sentences — those are flattened to a single segment.
//! - `videoData.playerData` (itself a JSON *string*, needs a second parse pass) →
//!   `resources.h264`: plain progressive mp4 URLs (not just the `hlsUrl` alongside it), so this
//!   downloads/streams exactly like dailydictation's mp3 — just a `.mp4` container instead.
//!
//! Talks are discovered from TED's yearly sitemaps rather than a category browse page. Only
//! English-original talks with an English transcript are kept (see the `language` checks) —
//! this app is for English listening practice, and dubbed/translated audio wouldn't match a
//! transcript in a useful way for dictation.

use crate::error::AppError;
use crate::scraper::{get_text_with_retry, Category, ScrapedLesson, ScrapedSegment};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;

/// TED has no CEFR-labeled categories at all (unlike dailydictation, which at least labels
/// content type per category) — talks are native-paced, unscripted-feeling speech, generally
/// well above dailydictation's A1-B1 range, so everything here is flatly tagged B2.
pub const CATEGORY: Category = Category { slug: "ted", level: "B2" };

/// Yearly sitemaps to crawl. TED publishes one gzipped sitemap per year back to 2006; crawling
/// all of them would mean walking on the order of ten thousand talk pages on a source whose
/// real-world rate limit hasn't been probed (unlike dailydictation's hand-tested threshold — see
/// `REQUEST_PACING` below). Starting with the most recent couple of years keeps a first refresh
/// reasonably bounded; add more years here once real-world pacing against ted.com is confirmed.
pub const TALK_SITEMAPS: &[&str] = &[
    "https://www.ted.com/sitemaps/talks-2025.xml.gz",
    "https://www.ted.com/sitemaps/talks-2024.xml.gz",
];

/// Delay between talk page fetches. NOT empirically verified against ted.com the way
/// dailydictation's `REQUEST_PACING` was — this is a conservative default pending real-world
/// testing, since TED pages are considerably heavier (~150-250KB) than dailydictation's.
pub const REQUEST_PACING: std::time::Duration = std::time::Duration::from_millis(500);

static LOC_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<loc>([^<]+)</loc>").unwrap());
static NEXT_DATA_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?s)<script id="__NEXT_DATA__" type="application/json">(.*?)</script>"#).unwrap()
});
// Sitemap URLs are the talk's own page, e.g. "https://www.ted.com/talks/some_speaker_a_title" —
// the slug is stable and unique, so it can be used as this app's lesson id (prefixed to avoid
// colliding with dailydictation's numeric ids) without fetching the page first.
static TALK_SLUG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"/talks/([^/?#]+)/?$").unwrap());

pub fn lesson_id_from_url(url: &str) -> Option<String> {
    TALK_SLUG_RE.captures(url).map(|c| format!("ted-{}", &c[1]))
}

#[derive(Debug, Deserialize)]
struct NextData {
    props: Props,
}

#[derive(Debug, Deserialize)]
struct Props {
    #[serde(rename = "pageProps")]
    page_props: PageProps,
}

#[derive(Debug, Deserialize)]
struct PageProps {
    #[serde(rename = "videoData")]
    video_data: VideoData,
    #[serde(rename = "transcriptData")]
    transcript_data: TranscriptData,
}

#[derive(Debug, Deserialize)]
struct VideoData {
    title: String,
    slug: String,
    /// Original spoken language of the talk (e.g. "en").
    language: String,
    /// Seconds — same unit as `Cue::time` once divided by 1000.
    duration: f64,
    #[serde(rename = "canonicalUrl")]
    canonical_url: String,
    /// A JSON *string* (not a nested object) that needs its own `serde_json::from_str` pass.
    #[serde(rename = "playerData")]
    player_data: String,
}

#[derive(Debug, Deserialize)]
struct TranscriptData {
    translation: Translation,
}

#[derive(Debug, Deserialize)]
struct Translation {
    paragraphs: Vec<Paragraph>,
    language: TranscriptLanguage,
}

#[derive(Debug, Deserialize)]
struct TranscriptLanguage {
    #[serde(rename = "internalLanguageCode")]
    internal_language_code: String,
}

#[derive(Debug, Deserialize)]
struct Paragraph {
    cues: Vec<Cue>,
}

#[derive(Debug, Deserialize)]
struct Cue {
    text: String,
    /// Start time in milliseconds — TED gives no per-cue end time (see module docs).
    time: f64,
}

#[derive(Debug, Deserialize)]
struct PlayerData {
    resources: Resources,
}

#[derive(Debug, Deserialize)]
struct Resources {
    h264: Vec<H264Resource>,
}

#[derive(Debug, Deserialize)]
struct H264Resource {
    bitrate: u32,
    file: String,
}

/// Lists every talk URL in one yearly sitemap. The sitemap is served gzip-compressed
/// (`Content-Encoding: gzip`); `reqwest`'s `gzip` feature decompresses it transparently, so this
/// gets plain XML text like `daily_dictation::fetch_category_urls` does.
pub async fn fetch_sitemap_urls(client: &reqwest::Client, sitemap_url: &str) -> Result<Vec<String>, AppError> {
    let body = get_text_with_retry(client, sitemap_url).await?;
    Ok(parse_sitemap_urls(&body))
}

pub fn parse_sitemap_urls(xml: &str) -> Vec<String> {
    LOC_RE.captures_iter(xml).map(|c| c[1].to_string()).collect()
}

/// Fetches one talk's transcript page and extracts its lesson + segments. Returns `None` (not an
/// error) for non-English talks or malformed pages — same "skip, don't fail the whole refresh"
/// policy `daily_dictation::fetch_exercise` uses.
pub async fn fetch_talk(client: &reqwest::Client, talk_url: &str) -> Result<Option<ScrapedLesson>, AppError> {
    let transcript_url = format!("{}/transcript", talk_url.trim_end_matches('/'));
    let html = get_text_with_retry(client, &transcript_url).await?;
    Ok(extract_lesson(&html))
}

pub fn extract_lesson(html: &str) -> Option<ScrapedLesson> {
    let captures = NEXT_DATA_RE.captures(html)?;
    let data: NextData = serde_json::from_str(&captures[1]).ok()?;
    let video = data.props.page_props.video_data;
    let translation = data.props.page_props.transcript_data.translation;

    if video.language != "en" || translation.language.internal_language_code != "en" {
        return None;
    }

    let player_data: PlayerData = serde_json::from_str(&video.player_data).ok()?;
    let audio_url = player_data
        .resources
        .h264
        .into_iter()
        .max_by_key(|r| r.bitrate)
        .map(|r| r.file)?;

    let cues: Vec<&Cue> = translation.paragraphs.iter().flat_map(|p| p.cues.iter()).collect();
    if cues.is_empty() {
        return None;
    }

    let segments = cues
        .iter()
        .enumerate()
        .map(|(i, cue)| {
            let time_start = cue.time / 1000.0;
            let time_end = cues.get(i + 1).map(|c| c.time / 1000.0).unwrap_or(video.duration);
            ScrapedSegment {
                position: i as i64,
                content: cue.text.replace('\n', " ").trim().to_string(),
                time_start,
                time_end,
            }
        })
        .collect();

    Some(ScrapedLesson {
        id: format!("ted-{}", video.slug),
        title: video.title,
        audio_url,
        page_url: video.canonical_url,
        segments,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real transcript page ("The brain in love", Helen Fisher, TED2008) — the __NEXT_DATA__
    // blob captured by hand on 2026-07-24, trimmed into a minimal HTML wrapper around just that
    // script tag (the real page also carries ~150KB of unrelated related-talks/nav data).
    const TRANSCRIPT_HTML: &str = include_str!("../../tests/fixtures/ted_helen_fisher_transcript.html");
    const SITEMAP_XML: &str = include_str!("../../tests/fixtures/ted_sitemap_sample.xml");

    #[test]
    fn extracts_lesson_and_segments_from_real_transcript_page() {
        let lesson = extract_lesson(TRANSCRIPT_HTML).expect("real transcript page should parse");

        assert_eq!(lesson.id, "ted-helen_fisher_the_brain_in_love");
        assert_eq!(lesson.title, "The brain in love");
        assert_eq!(lesson.page_url, "https://www.ted.com/talks/helen_fisher_the_brain_in_love");
        assert!(lesson.audio_url.ends_with(".mp4"));
        assert_eq!(lesson.segments.len(), 291);

        let first = &lesson.segments[0];
        assert_eq!(first.position, 0);
        assert_eq!(first.content, "I and my colleagues Art Aron and Lucy Brown and others,");
        assert_eq!(first.time_start, 0.871);
        // Inferred end = next cue's start, not an authored value.
        assert_eq!(first.time_end, 4.538);

        // Last cue has no "next" cue to infer an end from — falls back to the talk's total duration.
        let last = lesson.segments.last().unwrap();
        assert_eq!(last.position, 290);
        assert_eq!(last.content, "(Applause)");
        assert_eq!(last.time_start, 928.579);
        assert_eq!(last.time_end, 936.0);
    }

    #[test]
    fn returns_none_for_page_without_next_data() {
        assert!(extract_lesson("<html><body>no data here</body></html>").is_none());
    }

    #[test]
    fn lesson_id_from_url_extracts_slug() {
        assert_eq!(
            lesson_id_from_url("https://www.ted.com/talks/helen_fisher_the_brain_in_love"),
            Some("ted-helen_fisher_the_brain_in_love".to_string())
        );
        assert_eq!(lesson_id_from_url("https://example.com/not-a-match"), None);
    }

    #[test]
    fn parses_sitemap_urls_from_real_sitemap() {
        let urls = parse_sitemap_urls(SITEMAP_XML);
        assert!(!urls.is_empty());
        assert!(urls[0].starts_with("https://www.ted.com/talks/"));
    }
}
