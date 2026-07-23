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
