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
