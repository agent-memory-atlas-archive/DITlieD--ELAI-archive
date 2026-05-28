//! E-48 Task 3 — extractor pool with crawl4ai primary + Jina Reader fallback.
//!
//! Live HTTP wiring is RUNPOD-PENDING -- crawl4ai requires a
//! self-hosted Playwright endpoint (configurable in
//! `[web_search.extractor]` in `elai_cc.toml`); Jina Reader
//! (r.jina.ai/{URL}) is a managed free-tier HTTP GET. This module
//! ships the substrate (config struct, extractor outcome type, fallback
//! decision logic) that the concrete clients plug into.
//!
//! Cascade:
//!   1. Try crawl4ai (10s timeout).
//!   2. On timeout / JS-render failure, fall back to Jina (5s timeout).
//!   3. On both fail, emit `extraction_failed_all_extractors` row and
//!      drop the source.
//!
//! Grounding:
//!   * spider.cloud/blog/firecrawl-vs-crawl4ai-vs-spider-honest-benchmark
//!     -- Crawl4AI free self-host (grounds primary choice).
//!   * r.jina.ai/{URL} -- Jina Reader free-tier HTTP GET (grounds
//!     fallback choice).

use serde::{Deserialize, Serialize};

/// Default crawl4ai timeout in milliseconds. Per the plan task spec.
pub const DEFAULT_CRAWL4AI_TIMEOUT_MS: u32 = 10_000;

/// Default Jina Reader timeout in milliseconds. Per the plan task spec.
pub const DEFAULT_JINA_TIMEOUT_MS: u32 = 5_000;

/// Which extractor handled this row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractorBackend {
    /// crawl4ai (primary, JS-render-capable).
    Crawl4ai,
    /// Jina Reader (fallback, free-tier HTTP GET).
    Jina,
}

impl ExtractorBackend {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Crawl4ai => "crawl4ai",
            Self::Jina => "jina",
        }
    }
}

/// Failure reason for a single extractor attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractorFailure {
    /// Hit the configured timeout.
    Timeout,
    /// Upstream returned a 4xx (URL-shape problem, paywall, etc).
    HttpClientError,
    /// Upstream returned a 5xx (server-side failure).
    HttpServerError,
    /// JS render failed (crawl4ai only) -- caller should fall through
    /// to Jina.
    JsRenderFailed,
    /// Connection-level error (DNS, TLS, refused, etc).
    Connection,
}

impl ExtractorFailure {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::HttpClientError => "http_client_error",
            Self::HttpServerError => "http_server_error",
            Self::JsRenderFailed => "js_render_failed",
            Self::Connection => "connection",
        }
    }
}

/// Outcome of the per-source extractor cascade.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtractionOutcome {
    /// At least one extractor returned content. `backend` records
    /// which one.
    Success {
        backend: ExtractorBackend,
        content: String,
    },
    /// Both extractors failed. Caller should drop the source AND emit
    /// the `extraction_failed_all_extractors` SSB row.
    AllFailed {
        primary_failure: ExtractorFailure,
        fallback_failure: ExtractorFailure,
    },
}

/// Configuration for the extractor pool.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtractorPoolConfig {
    /// crawl4ai self-hosted endpoint URL (e.g. `http://127.0.0.1:11235`).
    /// `None` disables the primary -- the cascade falls through to Jina
    /// immediately.
    pub crawl4ai_endpoint: Option<String>,
    /// Jina Reader base URL (default `https://r.jina.ai`). Always
    /// configured; Jina has no API key for the free tier.
    pub jina_base_url: String,
    pub crawl4ai_timeout_ms: u32,
    pub jina_timeout_ms: u32,
}

impl Default for ExtractorPoolConfig {
    fn default() -> Self {
        Self {
            crawl4ai_endpoint: None,
            jina_base_url: "https://r.jina.ai".to_owned(),
            crawl4ai_timeout_ms: DEFAULT_CRAWL4AI_TIMEOUT_MS,
            jina_timeout_ms: DEFAULT_JINA_TIMEOUT_MS,
        }
    }
}

/// Decide whether a crawl4ai attempt should fall through to Jina.
/// Returns `true` for the failure modes that justify a fallback;
/// returns `false` for failure modes that indicate a URL-shape
/// problem (the same URL would fail Jina too -- skip the second call).
#[must_use]
pub fn should_fall_back(primary: ExtractorFailure) -> bool {
    matches!(
        primary,
        ExtractorFailure::Timeout
            | ExtractorFailure::JsRenderFailed
            | ExtractorFailure::HttpServerError
            | ExtractorFailure::Connection
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_labels_canonical() {
        assert_eq!(ExtractorBackend::Crawl4ai.as_str(), "crawl4ai");
        assert_eq!(ExtractorBackend::Jina.as_str(), "jina");
    }

    #[test]
    fn failure_labels_canonical() {
        assert_eq!(ExtractorFailure::Timeout.as_str(), "timeout");
        assert_eq!(ExtractorFailure::JsRenderFailed.as_str(), "js_render_failed");
    }

    #[test]
    fn timeout_triggers_fallback() {
        assert!(should_fall_back(ExtractorFailure::Timeout));
    }

    #[test]
    fn js_render_failed_triggers_fallback() {
        assert!(should_fall_back(ExtractorFailure::JsRenderFailed));
    }

    #[test]
    fn http_client_error_skips_fallback() {
        // 4xx == URL-shape problem; Jina would fail too.
        assert!(!should_fall_back(ExtractorFailure::HttpClientError));
    }

    #[test]
    fn default_config_has_jina_base_url() {
        let cfg = ExtractorPoolConfig::default();
        assert_eq!(cfg.jina_base_url, "https://r.jina.ai");
        assert!(cfg.crawl4ai_endpoint.is_none());
    }
}
