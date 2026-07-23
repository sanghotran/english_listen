//! Static feed config, verified by hand against the real VOA site on 2026-07-23 (see docs/PLAN.md).
//! There is no single combined RSS feed — level is assigned per-show, not guessed per-item.

pub const VOA_BASE_URL: &str = "https://learningenglish.voanews.com";

#[derive(Debug, Clone, Copy)]
pub struct FeedConfig {
    pub show: &'static str,
    pub level: &'static str,
    pub path: &'static str,
}

impl FeedConfig {
    pub fn url(&self) -> String {
        format!("{VOA_BASE_URL}{}", self.path)
    }
}

/// A1 content has no confirmed RSS structure yet (follow-up, see docs/PLAN.md Phase 8) —
/// starting with the 4 confirmed A2/B1 feeds only.
pub const FEEDS: &[FeedConfig] = &[
    FeedConfig {
        show: "As It Is",
        level: "B1",
        path: "/api/zkm-ql-vomx-tpej-rqi",
    },
    FeedConfig {
        show: "Ask a Teacher",
        level: "A2",
        path: "/api/zti_qvl-vomx-tpekgvqr",
    },
    FeedConfig {
        show: "Everyday Grammar",
        level: "A2",
        path: "/api/zoroqql-vomx-tpeptpqq",
    },
    FeedConfig {
        show: "Words and Their Stories",
        level: "B1",
        path: "/api/zmypyl-vomx-tpeyry_",
    },
];
