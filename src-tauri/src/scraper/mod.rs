pub mod daily_dictation;

/// dailydictation.com doesn't appear to bot-block, but VOA (the previous source) did without
/// a browser-like User-Agent, so this is kept as a defensive default for all outbound fetches.
pub const CHROME_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";
