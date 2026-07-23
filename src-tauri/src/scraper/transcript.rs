use crate::error::AppError;
use crate::scraper::CHROME_USER_AGENT;
use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct EpisodeContent {
    pub audio_url: String,
    pub transcript: String,
}

pub async fn fetch_episode(
    client: &reqwest::Client,
    episode_url: &str,
) -> Result<Option<EpisodeContent>, AppError> {
    let html = client
        .get(episode_url)
        .header("User-Agent", CHROME_USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    Ok(extract_episode(&html))
}

/// Returns `None` when the episode page is missing audio and/or transcript — not every VOA item
/// has both (some are video-only, some are audio without transcript). That is a normal skip
/// during ingestion, not an error.
pub fn extract_episode(html: &str) -> Option<EpisodeContent> {
    let document = Html::parse_document(html);

    let audio_selector = Selector::parse("audio[src]").ok()?;
    let audio_url = document.select(&audio_selector).next()?.attr("src")?.to_string();

    let wsw_selector = Selector::parse("div.wsw").ok()?;
    let wsw = document.select(&wsw_selector).next()?;

    // Transcript is the <p> tags in div.wsw, stopping before the "Words in This Story" glossary.
    let mut paragraphs = Vec::new();
    for child in wsw.child_elements() {
        if child.value().name() == "h2" {
            break;
        }
        if child.value().name() == "p" {
            let text = child.text().collect::<Vec<_>>().join(" ");
            let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if !normalized.is_empty() {
                paragraphs.push(normalized);
            }
        }
    }

    if paragraphs.is_empty() {
        return None;
    }

    Some(EpisodeContent {
        audio_url,
        transcript: paragraphs.join("\n\n"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real VOA episode page ("Researchers: South Korea's Birth Rate..."), fetched by hand on
    // 2026-07-23 — see docs/PLAN.md Phase 7. The <audio> tag sits deeply nested inside
    // div.wsw > div.wsw__embed, well before the transcript <p> tags that follow it.
    const EPISODE_HTML: &str = include_str!("../../tests/fixtures/episode_south_korea_birth_rate.html");

    #[test]
    fn extracts_audio_and_transcript_from_real_episode_page() {
        let episode =
            extract_episode(EPISODE_HTML).expect("real episode page should have audio+transcript");
        assert!(episode.audio_url.contains("e4a0af84-7057-4f35-9015-08dd5b02d8d7.mp3"));
        assert!(episode
            .transcript
            .starts_with("In 2024, the number of babies born in South Korea"));
        assert!(episode.transcript.contains("238,300 babies were born last year"));
        // The glossary section after the h2 boundary must be excluded.
        assert!(!episode.transcript.contains("Words in This Story"));
    }

    #[test]
    fn returns_none_when_audio_or_transcript_missing() {
        assert!(extract_episode("<html><body><p>no audio or wsw here</p></body></html>").is_none());
    }

    #[test]
    fn returns_none_when_wsw_has_no_paragraphs() {
        let html = r#"<html><body><audio src="https://example.com/a.mp3"></audio><div class="wsw"><h2>Words in This Story</h2></div></body></html>"#;
        assert!(extract_episode(html).is_none());
    }
}
