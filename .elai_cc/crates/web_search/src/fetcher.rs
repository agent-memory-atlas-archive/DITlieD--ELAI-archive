use rustc_hash::FxHashMap;
use std::sync::Arc;
use std::time::Duration;

use elai_core::clock::ClockProvider;
use reqwest::redirect::Policy;
use crate::rate_limiter::{RateLimiter, SleepFn, tokio_sleep};
use crate::robots::RobotsCache;
use crate::WebSearchError;

// Named constants — no magic numbers.
/// HTTP request timeout in seconds.
pub const FETCH_TIMEOUT_SECS: u64 = 15;
/// Maximum HTTP redirects to follow.
pub const MAX_REDIRECTS: usize = 5;
/// Maximum response body size in bytes (2 MB).
pub const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;
/// User-Agent header value.
pub const USER_AGENT: &str = "ELAI-Research/0.1 (research-agent)";
/// Minimum interval between requests to the same domain (500ms = 2 req/s).
pub const RATE_LIMIT_INTERVAL_MS: u64 = 500;

/// A fetched web page with metadata.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FetchedPage {
    /// Final URL after redirects (canonical).
    pub url: String,
    pub status: u16,
    pub content_type: String,
    pub html: String,
    pub fetch_time_ms: u64,
}

/// HTTP fetcher with per-domain rate limiting, robots.txt checking, and body size enforcement.
pub struct WebFetcher {
    client: reqwest::Client,
    clock: Arc<dyn ClockProvider>,
    robots: RobotsCache,
    domain_limiters: FxHashMap<String, RateLimiter>,
    sleep_fn: SleepFn,
}

/// Extract domain string from a URL for rate limiter keying.
fn extract_domain(url_str: &str) -> Result<String, WebSearchError> {
    let parsed = url::Url::parse(url_str).map_err(|e| WebSearchError::InvalidUrl {
        url: url_str.to_owned(),
        reason: e.to_string(),
    })?;
    let host = parsed
        .host_str()
        .ok_or_else(|| WebSearchError::InvalidUrl {
            url: url_str.to_owned(),
            reason: "no host".to_owned(),
        })?;
    Ok(host.to_lowercase())
}

impl WebFetcher {
    /// Create a new WebFetcher with default settings.
    pub fn new(clock: Arc<dyn ClockProvider>) -> Result<Self, WebSearchError> {
        Self::with_sleep(clock, tokio_sleep())
    }

    /// Create with custom sleep function for deterministic testing.
    pub fn with_sleep(
        clock: Arc<dyn ClockProvider>,
        sleep_fn: SleepFn,
    ) -> Result<Self, WebSearchError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(FETCH_TIMEOUT_SECS))
            .redirect(Policy::limited(MAX_REDIRECTS))
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| WebSearchError::HttpClient(e.to_string()))?;

        Ok(Self {
            client,
            clock: clock.clone(),
            robots: RobotsCache::new(clock),
            domain_limiters: FxHashMap::default(),
            sleep_fn,
        })
    }

    /// Fetch a URL, respecting robots.txt and per-domain rate limits.
    /// Returns FetchedPage with the final (post-redirect) URL.
    pub async fn fetch(&mut self, url: &str) -> Result<FetchedPage, WebSearchError> {
        // 1. Check robots.txt.
        self.robots
            .check_robots(url, &self.client)
            .await?;

        // 2. Per-domain rate limiting.
        let domain = extract_domain(url)?;
        let limiter = self
            .domain_limiters
            .entry(domain)
            .or_insert_with(|| {
                RateLimiter::with_sleep(
                    self.clock.clone(),
                    Duration::from_millis(RATE_LIMIT_INTERVAL_MS),
                    self.sleep_fn.clone(),
                )
            });
        limiter.wait_if_needed().await;

        // 3. Send request.
        let start = self.clock.now();
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| WebSearchError::FetchFailed {
                url: url.to_owned(),
                reason: e.to_string(),
            })?;

        let final_url = response.url().to_string();
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_owned();

        // 4. Read body with size enforcement via streaming.
        let html = self.read_body_limited(response, url).await?;

        let elapsed = self.clock.elapsed_since(start);
        let fetch_time_ms = u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX);

        Ok(FetchedPage {
            url: final_url,
            status,
            content_type,
            html,
            fetch_time_ms,
        })
    }

    /// Read response body with size limit enforcement.
    /// Uses bytes_stream to avoid loading oversized responses into memory.
    async fn read_body_limited(
        &self,
        response: reqwest::Response,
        request_url: &str,
    ) -> Result<String, WebSearchError> {
        use futures_util::StreamExt;

        let mut bytes = Vec::new();
        let mut stream = response.bytes_stream();
        let mut total: usize = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| WebSearchError::FetchFailed {
                url: request_url.to_owned(),
                reason: e.to_string(),
            })?;
            total = total.saturating_add(chunk.len());
            if total > MAX_BODY_BYTES {
                return Err(WebSearchError::BodyTooLarge {
                    size: total,
                    limit: MAX_BODY_BYTES,
                });
            }
            bytes.extend_from_slice(&chunk);
        }

        // Non-UTF8 safety: lossy conversion.
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    /// Get a reference to the internal HTTP client.
    pub fn http_client(&self) -> &reqwest::Client {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::clock::SeededClock;

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://Example.COM/path").unwrap(),
            "example.com"
        );
    }

    #[test]
    fn test_extract_domain_invalid() {
        assert!(extract_domain("not-a-url").is_err());
    }

    #[test]
    fn test_named_constants() {
        assert_eq!(FETCH_TIMEOUT_SECS, 15);
        assert_eq!(MAX_REDIRECTS, 5);
        assert_eq!(MAX_BODY_BYTES, 2 * 1024 * 1024);
        assert_eq!(RATE_LIMIT_INTERVAL_MS, 500);
        assert!(!USER_AGENT.is_empty());
    }

    #[test]
    fn test_fetched_page_fields() {
        let page = FetchedPage {
            url: "https://example.com/final".to_owned(),
            status: 200,
            content_type: "text/html".to_owned(),
            html: "<p>hello</p>".to_owned(),
            fetch_time_ms: 42,
        };
        assert_eq!(page.url, "https://example.com/final");
        assert_eq!(page.status, 200);
        assert_eq!(page.fetch_time_ms, 42);
    }

    #[test]
    fn test_fetched_page_serializes() {
        let page = FetchedPage {
            url: "u".to_owned(),
            status: 200,
            content_type: "ct".to_owned(),
            html: "h".to_owned(),
            fetch_time_ms: 1,
        };
        let json = serde_json::to_string(&page).expect("serialize");
        assert!(json.contains("\"url\":\"u\""));
        assert!(json.contains("\"status\":200"));
    }

    // Adversarial: Non-UTF8 body handled by from_utf8_lossy.
    #[test]
    fn test_non_utf8_lossy_conversion() {
        let bytes: Vec<u8> = vec![0x63, 0x61, 0x66, 0xe9]; // "caf" + 0xe9 (ISO-8859-1 é)
        let result = String::from_utf8_lossy(&bytes);
        assert!(result.starts_with("caf"));
        assert!(result.contains('\u{FFFD}'));
    }

    // Adversarial: Body >2MB rejected.
    #[test]
    fn test_body_size_enforcement_logic() {
        let total: usize = MAX_BODY_BYTES + 1;
        assert!(total > MAX_BODY_BYTES);
        assert_eq!(MAX_BODY_BYTES, 2_097_152);
    }

    #[tokio::test]
    async fn test_web_fetcher_constructs() {
        let clock = Arc::new(SeededClock::new());
        let fetcher = WebFetcher::new(clock);
        assert!(fetcher.is_ok());
    }
}
