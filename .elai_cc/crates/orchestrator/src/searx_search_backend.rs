// Session 4D.11: production SearchBackend impl that proxies the research
// pipeline's query path to the sidecar's SearXNG-compatible `/search` endpoint.
//
// The sidecar (python/elai_inference/server.py @app.get("/search")) brokers
// three strategies (SearXNG auto-start -> Brave fallback -> empty), so Rust
// only has to speak the SearXNG contract: GET /search?q=...&format=json.
//
// Bridging: SearchBackend::search is sync; SearxClient::search is async. We
// own a dedicated current-thread tokio runtime plus a std::sync::Mutex around
// the client (SearxClient::search takes &mut self for rate-limiter state).

use std::sync::{Arc, Mutex};

use elai_core::clock::{ClockProvider, OsClock};
use elai_core::research_types::ResearchPipelineError;
use elai_web_search::client::SearxClient;
use tokio::runtime::Runtime;
use tracing::{debug, warn};

use crate::research_pipeline::{SearchBackend, SearchResult};

/// Default sidecar base URL — matches providers::DEFAULT_SIDECAR_URL and the
/// port that server.py binds by default. The sidecar exposes `/search` as a
/// SearXNG-compatible drop-in, so SearxClient can target it unchanged.
pub const DEFAULT_SIDECAR_SEARCH_URL: &str = "http://localhost:8100";

/// Production `SearchBackend` that delegates to the sidecar `/search` proxy.
///
/// Owns a dedicated current-thread tokio runtime so it can bridge the sync
/// `SearchBackend::search` call to `SearxClient`'s async body from the CLI's
/// non-runtime slash-command context (see `fsm_transition_sync` for the
/// existing pattern in cli/src/main.rs).
///
/// Note: `SearxClient` internally caps results at `DEFAULT_MAX_RESULTS = 20`
/// (elai-web-search crate constant). The `max_results` arg passed through
/// `SearchBackend::search` applies *on top of* that ceiling via `.take()`, so
/// any value above 20 is silently clamped to 20 by the inner client.
pub struct SearxSearchBackend {
    client: Mutex<SearxClient>,
    runtime: Runtime,
}

impl SearxSearchBackend {
    /// Build a backend targeting `base_url` (the sidecar host, not `/search`).
    /// `SearxClient::search` appends `/search` to this base.
    pub fn new(base_url: impl Into<String>) -> Result<Self, ResearchPipelineError> {
        Self::with_clock(base_url, Arc::new(OsClock))
    }

    /// Build with a custom clock (test-friendly).
    pub fn with_clock(
        base_url: impl Into<String>,
        clock: Arc<dyn ClockProvider>,
    ) -> Result<Self, ResearchPipelineError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                ResearchPipelineError::SearchBackend(format!("runtime build: {e}"))
            })?;

        let client = SearxClient::new(base_url, clock).map_err(|e| {
            ResearchPipelineError::SearchBackend(format!("searx client: {e}"))
        })?;

        Ok(Self {
            client: Mutex::new(client),
            runtime,
        })
    }
}

impl SearchBackend for SearxSearchBackend {
    fn search(
        &self,
        query: &str,
        max_results: u32,
    ) -> Result<Vec<SearchResult>, ResearchPipelineError> {
        let mut guard = self.client.lock().map_err(|e| {
            ResearchPipelineError::SearchBackend(format!("mutex poisoned: {e}"))
        })?;

        debug!(query = query, "searx_backend: issuing search");

        let web_results = self
            .runtime
            .block_on(guard.search(query))
            .map_err(|e| {
                warn!(error = %e, "searx_backend: search failed");
                ResearchPipelineError::SearchBackend(format!("searx: {e}"))
            })?;

        let max = max_results as usize;
        Ok(web_results
            .into_iter()
            .take(max)
            .map(|r| SearchResult {
                url: r.url,
                title: r.title,
                snippet: r.snippet,
            })
            .collect())
    }
}

// ===========================================================================
// Unit tests — cover construction + sync/async bridge without real network.
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_with_default_url() {
        let backend = SearxSearchBackend::new(DEFAULT_SIDECAR_SEARCH_URL);
        assert!(backend.is_ok());
    }

    #[test]
    fn rejects_ill_formed_url_at_request_time() {
        // SearxClient::new does not validate the URL; the error surfaces on search.
        let backend = SearxSearchBackend::new("not-a-valid-url")
            .expect("constructs — validation deferred to request time");
        let err = backend.search("rust", 5).expect_err("invalid URL should error");
        match err {
            ResearchPipelineError::SearchBackend(msg) => {
                assert!(msg.contains("searx"), "error should originate from searx: {msg}");
            }
            other => panic!("expected SearchBackend error, got {other:?}"),
        }
    }

    #[test]
    fn returns_search_backend_error_on_unreachable() {
        // Port 1 (tcpmux) is reserved and reliably refuses connections on
        // localhost. Confirms reqwest connect failures map to the typed
        // ResearchPipelineError::SearchBackend variant instead of panicking.
        // Truncation by `max_results` is covered by
        // contracts/tests/searx_search_backend_contract.rs against a real
        // local HTTP stub.
        let backend = SearxSearchBackend::new("http://127.0.0.1:1")
            .expect("constructs");
        let err = backend.search("anything", 3).expect_err("should fail fast");
        assert!(matches!(err, ResearchPipelineError::SearchBackend(_)));
    }
}
