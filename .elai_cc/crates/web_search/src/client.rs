use std::sync::Arc;
use std::time::Duration;

use elai_core::clock::ClockProvider;
use tracing::{debug, warn};

use crate::rate_limiter::{self, RateLimiter, SleepFn};
use crate::{
    SearchError, SearchResult, SearxResponse,
    DEFAULT_MAX_RESULTS, DEFAULT_RATE_LIMIT_INTERVAL_MS,
    DEFAULT_SEARCH_TIMEOUT_SECS, HEALTH_CHECK_TIMEOUT_SECS,
    MAX_RETRY_COUNT, RETRY_BASE_DELAY_MS,
};

/// SearXNG HTTP client with rate limiting and retry.
/// IPC proxy wiring deferred to 4D.17: agent sends SearchRequest IPC variant,
/// orchestrator calls this client directly until then.
pub struct SearxClient {
    base_url: String,
    timeout: Duration,
    max_results: usize,
    rate_limiter: RateLimiter,
    client: reqwest::Client,
    health_client: reqwest::Client,
    sleep_fn: SleepFn,
}

impl SearxClient {
    /// Create a new SearxClient with default settings.
    pub fn new(base_url: impl Into<String>, clock: Arc<dyn ClockProvider>) -> Result<Self, SearchError> {
        Self::with_config(
            base_url,
            clock,
            Duration::from_secs(DEFAULT_SEARCH_TIMEOUT_SECS),
            DEFAULT_MAX_RESULTS,
            Duration::from_millis(DEFAULT_RATE_LIMIT_INTERVAL_MS),
            rate_limiter::tokio_sleep(),
        )
    }

    /// Create a SearxClient with custom configuration and injectable sleep.
    pub fn with_config(
        base_url: impl Into<String>,
        clock: Arc<dyn ClockProvider>,
        timeout: Duration,
        max_results: usize,
        rate_limit_interval: Duration,
        sleep_fn: SleepFn,
    ) -> Result<Self, SearchError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()?;

        let health_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(HEALTH_CHECK_TIMEOUT_SECS))
            .build()?;

        Ok(Self {
            base_url: base_url.into(),
            timeout,
            max_results,
            rate_limiter: RateLimiter::with_sleep(clock, rate_limit_interval, sleep_fn.clone()),
            client,
            health_client,
            sleep_fn,
        })
    }

    /// Search SearXNG. URL-encodes query via reqwest's .query() (no manual format!).
    /// Retries once on 5xx with exponential backoff. Does not retry 4xx.
    pub async fn search(&mut self, query: &str) -> Result<Vec<SearchResult>, SearchError> {
        self.rate_limiter.wait_if_needed().await;

        debug!(query = query, "search: starting request");

        let mut last_err = None;

        for attempt in 0..=MAX_RETRY_COUNT {
            if attempt > 0 {
                let delay_ms = RETRY_BASE_DELAY_MS * 2u64.pow(attempt - 1);
                warn!(attempt = attempt, delay_ms = delay_ms, "search: retrying after 5xx");
                (self.sleep_fn)(Duration::from_millis(delay_ms)).await;
            }

            let result = self
                .client
                .get(format!("{}/search", self.base_url))
                .query(&[
                    ("q", query),
                    ("format", "json"),
                    ("pageno", "1"),
                ])
                .send()
                .await;

            let response = match result {
                Ok(r) => r,
                Err(e) => {
                    last_err = Some(SearchError::Http(e));
                    continue;
                }
            };

            let status = response.status().as_u16();

            if status >= 500 {
                warn!(status = status, "search: received 5xx response");
                last_err = Some(SearchError::ServiceUnavailable { status });
                continue;
            }

            if status >= 400 {
                // 4xx: do not retry.
                return Err(SearchError::ServiceUnavailable { status });
            }

            let body = response.text().await.map_err(SearchError::Http)?;
            let searx_resp: SearxResponse = serde_json::from_str(&body)?;

            let results: Vec<SearchResult> = searx_resp
                .results
                .into_iter()
                .take(self.max_results)
                .map(|raw| raw.into_search_result())
                .collect();

            debug!(count = results.len(), "search: got results");
            return Ok(results);
        }

        // All retries exhausted.
        Err(last_err.unwrap_or(SearchError::ServiceUnavailable { status: 503 }))
    }

    /// Health check: GET base_url with a short timeout.
    /// Returns Ok(true) if reachable, Err on failure.
    pub async fn health_check(&self) -> Result<bool, SearchError> {
        let response = self.health_client
            .get(&self.base_url)
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Get the configured base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the configured timeout.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Get the configured max results.
    pub fn max_results(&self) -> usize {
        self.max_results
    }
}
