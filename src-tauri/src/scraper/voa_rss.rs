use crate::error::AppError;
use crate::scraper::CHROME_USER_AGENT;

#[derive(Debug, Clone)]
pub struct FeedItem {
    pub title: String,
    pub link: String,
    pub guid: String,
    pub pub_date: String,
}

/// Fetches and parses a VOA show RSS feed. RSS items only carry `title`/`link`/`guid`/`pubDate` —
/// audio and transcript live on the episode page at `link` (see `transcript::fetch_episode`).
pub async fn fetch_feed(client: &reqwest::Client, feed_url: &str) -> Result<Vec<FeedItem>, AppError> {
    let bytes = client
        .get(feed_url)
        .header("User-Agent", CHROME_USER_AGENT)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    parse_channel(&bytes)
}

pub fn parse_channel(xml: &[u8]) -> Result<Vec<FeedItem>, AppError> {
    let channel = rss::Channel::read_from(xml).map_err(|e| AppError::Parse(e.to_string()))?;

    let items = channel
        .items()
        .iter()
        .filter_map(|item| {
            let title = item.title()?.to_string();
            let link = item.link()?.to_string();
            let guid = item.guid()?.value().to_string();
            let pub_date = item.pub_date().unwrap_or_default().to_string();
            Some(FeedItem {
                title,
                link,
                guid,
                pub_date,
            })
        })
        .collect();

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real "As It Is" feed XML, fetched by hand on 2026-07-23 — see docs/PLAN.md Phase 7.
    const FEED_XML: &str = include_str!("../../tests/fixtures/as_it_is_feed.xml");

    #[test]
    fn parses_real_voa_feed() {
        let items = parse_channel(FEED_XML.as_bytes()).expect("real VOA feed should parse");
        assert_eq!(items.len(), 20);

        let south_korea = items
            .iter()
            .find(|i| i.link.contains("researchers-south-korea"))
            .expect("South Korea episode should be present in the feed");
        assert_eq!(
            south_korea.guid,
            "https://learningenglish.voanews.com/a/researchers-south-korea-s-birth-rate-increase-last-year-unclear-/7997203.html"
        );
        assert_eq!(south_korea.pub_date, "Wed, 12 Mar 2025 22:00:00 +0000");
        assert!(south_korea.title.starts_with("Researchers: South Korea"));
    }

    #[test]
    fn rejects_non_rss_input() {
        assert!(parse_channel(b"<html></html>").is_err());
    }
}
