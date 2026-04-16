#![deny(dead_code, unused_imports)]

pub mod client;
pub mod rate_limiter;

use serde::Deserialize;

// Named constants — no magic numbers.
pub const DEFAULT_SEARCH_TIMEOUT_SECS: u64 = 10;
pub const HEALTH_CHECK_TIMEOUT_SECS: u64 = 2;
pub const DEFAULT_MAX_RESULTS: usize = 20;
pub const DEFAULT_RATE_LIMIT_INTERVAL_MS: u64 = 1000;
pub const MAX_RETRY_COUNT: u32 = 1;
pub const RETRY_BASE_DELAY_MS: u64 = 500;

/// A single search result from SearXNG, enriched with parsed domain.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    /// HTML-stripped snippet text.
    pub snippet: String,
    pub engine: String,
    pub score: f64,
    pub parsed_domain: String,
}

/// Raw SearXNG JSON result before post-processing.
#[derive(Debug, Deserialize)]
struct RawSearchResult {
    #[serde(default)]
    title: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    content: String,
    #[serde(default)]
    engine: String,
    #[serde(default)]
    score: Option<f64>,
}

/// SearXNG search response envelope.
#[derive(Debug, Deserialize)]
struct SearxResponse {
    #[serde(default)]
    results: Vec<RawSearchResult>,
}

impl RawSearchResult {
    fn into_search_result(self) -> SearchResult {
        let score = self.score.unwrap_or(0.0);
        // Non-finite safety: NaN or Infinity from upstream -> 0.0.
        // serde_json rejects non-finite floats during serialization.
        let score = if score.is_finite() { score } else { 0.0 };
        let snippet = strip_html(&self.content);
        let parsed_domain = parse_domain(&self.url);
        SearchResult {
            title: self.title,
            url: self.url,
            snippet,
            engine: self.engine,
            score,
            parsed_domain,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Rate limit exceeded")]
    RateLimited,
    #[error("Service unavailable: {status}")]
    ServiceUnavailable { status: u16 },
}

/// Strip HTML tags from a string. Simple scanner, no external crate.
/// On unclosed `<` (no matching `>`), the partial tag content is preserved
/// to avoid silent data loss from malformed HTML snippets.
pub fn strip_html(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut inside_tag = false;
    let mut tag_start = 0;
    for (i, ch) in input.chars().enumerate() {
        match ch {
            '<' => {
                inside_tag = true;
                tag_start = i;
            }
            '>' if inside_tag => inside_tag = false,
            _ if !inside_tag => result.push(ch),
            _ => {}
        }
    }
    // If we ended inside a tag, emit the unclosed content to avoid data loss.
    if inside_tag {
        let remainder: String = input.chars().skip(tag_start).collect();
        result.push_str(&remainder);
    }
    result
}

/// Extract domain from a URL. Returns empty string on malformed input.
pub fn parse_domain(url: &str) -> String {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    without_scheme
        .split('/')
        .next()
        .unwrap_or("")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- TDD: strip_html ---

    #[test]
    fn test_html_strip_snippet() {
        assert_eq!(strip_html("<b>tags</b>"), "tags");
        assert_eq!(strip_html("<b>foo</b> bar"), "foo bar");
        assert_eq!(strip_html("no tags here"), "no tags here");
        assert_eq!(strip_html(""), "");
        assert_eq!(strip_html("<a href=\"x\">link</a>"), "link");
    }

    // --- TDD: parse_domain ---

    #[test]
    fn test_parsed_domain_extraction() {
        assert_eq!(
            parse_domain("https://docs.rust-lang.org/std/"),
            "docs.rust-lang.org"
        );
        assert_eq!(
            parse_domain("http://example.com/path?q=1"),
            "example.com"
        );
        assert_eq!(parse_domain(""), "");
        assert_eq!(parse_domain("not-a-url"), "not-a-url");
    }

    // --- TDD: score NaN safety ---

    #[test]
    fn test_score_nan_safe() {
        // score = null in JSON -> Option<f64> = None -> 0.0
        let json = r#"{"title":"t","url":"u","content":"c","engine":"e","score":null}"#;
        let raw: RawSearchResult = serde_json::from_str(json).expect("parse");
        let result = raw.into_search_result();
        assert_eq!(result.score, 0.0);
        assert!(!result.score.is_nan());
    }

    #[test]
    fn test_score_missing_field() {
        let json = r#"{"title":"t","url":"u","content":"c","engine":"e"}"#;
        let raw: RawSearchResult = serde_json::from_str(json).expect("parse");
        let result = raw.into_search_result();
        assert_eq!(result.score, 0.0);
    }

    // --- TDD: RawSearchResult -> SearchResult conversion ---

    #[test]
    fn test_raw_to_search_result_strips_html() {
        let raw = RawSearchResult {
            title: "Title".to_owned(),
            url: "https://example.com/page".to_owned(),
            content: "<b>bold</b> text".to_owned(),
            engine: "duckduckgo".to_owned(),
            score: Some(1.5),
        };
        let result = raw.into_search_result();
        assert_eq!(result.snippet, "bold text");
        assert_eq!(result.parsed_domain, "example.com");
        assert_eq!(result.score, 1.5);
    }

    // --- TDD: rate limiter (uses mock clock) ---

    #[tokio::test]
    async fn test_rate_limit_two_calls_take_one_second() {
        use elai_core::clock::{ClockProvider, SeededClock};
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let clock = Arc::new(SeededClock::new());
        let sleep_called = Arc::new(AtomicBool::new(false));
        let sleep_flag = sleep_called.clone();

        // Injectable noop sleep that records whether it was invoked.
        let sleep_fn: rate_limiter::SleepFn = Arc::new(move |_dur| {
            sleep_flag.store(true, Ordering::SeqCst);
            Box::pin(async {})
        });

        let mut limiter = rate_limiter::RateLimiter::with_sleep(
            clock.clone(),
            std::time::Duration::from_millis(DEFAULT_RATE_LIMIT_INTERVAL_MS),
            sleep_fn,
        );

        // First call: no sleep needed.
        limiter.wait_if_needed().await;
        assert!(!sleep_called.load(Ordering::SeqCst));

        let t1 = clock.now();

        // Second call: SeededClock elapsed = 1ms < 1000ms interval, sleep_fn invoked.
        limiter.wait_if_needed().await;
        assert!(sleep_called.load(Ordering::SeqCst));

        let t2 = clock.now();
        assert!(t2.tick() > t1.tick());
    }

    // --- TDD: SearxResponse deserialization ---

    #[test]
    fn test_searx_response_missing_results_key() {
        let json = r#"{}"#;
        let resp: SearxResponse = serde_json::from_str(json).expect("parse");
        assert!(resp.results.is_empty());
    }

    #[test]
    fn test_searx_response_empty_results() {
        let json = r#"{"results":[]}"#;
        let resp: SearxResponse = serde_json::from_str(json).expect("parse");
        assert!(resp.results.is_empty());
    }
}
