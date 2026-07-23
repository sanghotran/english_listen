pub mod feeds;
pub mod transcript;
pub mod voa_rss;

/// VOA has bot-protection: a request without a browser-like User-Agent gets a 403.
pub const CHROME_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";
