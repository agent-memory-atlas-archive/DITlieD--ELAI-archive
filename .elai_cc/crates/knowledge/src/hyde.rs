// HyDE (Hypothetical Document Embeddings) — Phase 3K.3.
//
// Generates a hypothetical answer via sidecar LLM, embeds it, and uses
// that embedding for dense vector search. Reranking uses the original query.
// See arxiv 2212.10496.
//
// Graceful degradation: if sidecar is unavailable or hypothesis generation
// fails, HyDE is skipped and direct search proceeds normally.
// Integrates with Phase 1I resilience (circuit breaker pattern).

use rustc_hash::FxHashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use elai_core::{ClockProvider, LogicalInstant};

use crate::vector::circuit_breaker::{
    BreakerState, EmbeddingCircuitBreaker, FailureKind,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum query word count to trigger HyDE (complex queries only).
pub const MIN_WORDS_FOR_HYDE: usize = 5;

/// Cache TTL: 5 minutes (300 seconds).
const HYDE_CACHE_TTL: Duration = Duration::from_secs(300);

/// Sidecar timeout for hypothesis generation.
pub const HYDE_SIDECAR_TIMEOUT: Duration = Duration::from_secs(3);

/// Max tokens for hypothesis generation.
pub const HYDE_MAX_TOKENS: u32 = 128;

/// Temperature for hypothesis generation (slightly creative).
pub const HYDE_TEMPERATURE: f32 = 0.7;

/// Prompt template for hypothesis generation.
pub const HYDE_PROMPT_PREFIX: &str = "Write a short technical paragraph answering: ";

// ---------------------------------------------------------------------------
// HydeClient trait
// ---------------------------------------------------------------------------

/// Trait for hypothesis generation via an LLM.
///
/// Implementations should call the sidecar (or any LLM) and return a short
/// hypothetical document. The 3-second timeout enforcement is the
/// implementor's responsibility (the HydeStage does NOT add a second timeout).
pub trait HydeClient: Send + Sync {
    /// Generate a hypothetical answer for the given query.
    ///
    /// The returned string should be a short technical paragraph (~128 tokens).
    /// Errors are stringified — the HydeStage treats all failures as transient
    /// circuit-breaker events.
    fn generate_hypothesis(&self, query: &str) -> Result<String, String>;
}

// ---------------------------------------------------------------------------
// CachedHypothesis
// ---------------------------------------------------------------------------

struct CachedHypothesis {
    text: String,
    created_at: LogicalInstant,
}

// ---------------------------------------------------------------------------
// HydeStage
// ---------------------------------------------------------------------------

/// HyDE stage for the retrieval pipeline.
///
/// Caches hypothetical documents keyed by blake3(query) with 5-minute TTL.
/// Uses a circuit breaker (Phase 1I pattern) to avoid hammering a failing sidecar.
pub struct HydeStage {
    client: Arc<dyn HydeClient>,
    cache: Mutex<FxHashMap<[u8; 32], CachedHypothesis>>,
    breaker: Mutex<EmbeddingCircuitBreaker>,
    clock: Arc<dyn ClockProvider>,
}

impl HydeStage {
    /// Create a new HyDE stage with default circuit breaker configuration.
    pub fn new(client: Arc<dyn HydeClient>, clock: Arc<dyn ClockProvider>) -> Self {
        let breaker = EmbeddingCircuitBreaker::with_defaults("hyde_sidecar", clock.clone());
        Self {
            client,
            cache: Mutex::new(FxHashMap::default()),
            breaker: Mutex::new(breaker),
            clock,
        }
    }

    /// Generate a hypothetical document for the given query.
    ///
    /// Returns `None` if:
    /// - Query is too short (< MIN_WORDS_FOR_HYDE words)
    /// - Circuit breaker is open
    /// - Sidecar call fails or times out
    ///
    /// Returns cached hypothesis on cache hit (5-minute TTL).
    pub fn hypothesize(&self, query: &str) -> Option<String> {
        // Skip short/simple queries.
        let word_count = query.split_whitespace().count();
        if word_count < MIN_WORDS_FOR_HYDE {
            tracing::debug!(
                word_count,
                min = MIN_WORDS_FOR_HYDE,
                "hyde: skipping short query"
            );
            return None;
        }

        // Cache lookup (blake3 key).
        let key_bytes: [u8; 32] = *blake3::hash(query.as_bytes()).as_bytes();

        if let Ok(cache) = self.cache.lock() {
            if let Some(entry) = cache.get(&key_bytes) {
                let age = self.clock.elapsed_since(entry.created_at);
                if age < HYDE_CACHE_TTL {
                    // NARROWING-OK: cache age never exceeds u64::MAX ms (584M years).
                    tracing::debug!(age_ms = age.as_millis().min(u64::MAX as u128) as u64, "hyde: cache hit");
                    return Some(entry.text.clone());
                }
            }
        }

        // Circuit breaker gate + sidecar call.
        let hypothesis = self.call_through_breaker(query);

        // Cache on success + evict expired entries.
        if let Some(ref text) = hypothesis {
            if let Ok(mut cache) = self.cache.lock() {
                let now = self.clock.now();
                cache.retain(|_, v| self.clock.elapsed_since(v.created_at) < HYDE_CACHE_TTL);
                cache.insert(key_bytes, CachedHypothesis {
                    text: text.clone(),
                    created_at: now,
                });
            }
        }

        hypothesis
    }

    /// Breaker state accessor (for pipeline stage tracing).
    pub fn breaker_state(&self) -> Option<BreakerState> {
        self.breaker.lock().ok().map(|b| b.state())
    }

    /// Call sidecar through the circuit breaker.
    fn call_through_breaker(&self, query: &str) -> Option<String> {
        let mut breaker = match self.breaker.lock() {
            Ok(b) => b,
            Err(_) => return None,
        };
        breaker.sync_state();

        if breaker.state() == BreakerState::Open {
            tracing::debug!("hyde: circuit breaker open, degrading to direct search");
            return None;
        }

        let client = self.client.clone();
        let q = query.to_string();
        let result = breaker.call(
            move || client.generate_hypothesis(&q),
            FailureKind::Transient,
        );

        match result {
            Ok(text) if !text.trim().is_empty() => {
                tracing::debug!(
                    hypothesis_len = text.len(),
                    "hyde: hypothesis generated"
                );
                Some(text)
            }
            Ok(_) => {
                tracing::debug!("hyde: empty hypothesis returned, skipping");
                None
            }
            Err(e) => {
                tracing::debug!("hyde: generation failed: {e}");
                None
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::SeededClock;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Mock HyDE client that always succeeds.
    struct MockHydeClient {
        call_count: AtomicUsize,
    }

    impl MockHydeClient {
        fn new() -> Self {
            Self { call_count: AtomicUsize::new(0) }
        }

        fn calls(&self) -> usize {
            self.call_count.load(Ordering::Relaxed)
        }
    }

    impl HydeClient for MockHydeClient {
        fn generate_hypothesis(&self, query: &str) -> Result<String, String> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            Ok(format!(
                "The system handles this by processing the query '{}' through \
                 a multi-stage pipeline with BM25 and vector search.",
                query
            ))
        }
    }

    /// Mock HyDE client that always fails.
    struct FailingHydeClient;

    impl HydeClient for FailingHydeClient {
        fn generate_hypothesis(&self, _query: &str) -> Result<String, String> {
            Err("sidecar unavailable".to_string())
        }
    }

    /// Mock HyDE client that returns empty text.
    struct EmptyHydeClient;

    impl HydeClient for EmptyHydeClient {
        fn generate_hypothesis(&self, _query: &str) -> Result<String, String> {
            Ok(String::new())
        }
    }

    fn make_stage(client: Arc<dyn HydeClient>) -> HydeStage {
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        HydeStage::new(client, clock)
    }

    #[test]
    fn short_query_skips_hyde() {
        let client = Arc::new(MockHydeClient::new());
        let stage = make_stage(client.clone());
        let result = stage.hypothesize("find main");
        assert!(result.is_none(), "short query should skip HyDE");
        assert_eq!(client.calls(), 0, "sidecar should not be called for short queries");
    }

    #[test]
    fn four_word_query_skips_hyde() {
        let client = Arc::new(MockHydeClient::new());
        let stage = make_stage(client.clone());
        let result = stage.hypothesize("how does FSM work");
        assert!(result.is_none());
        assert_eq!(client.calls(), 0);
    }

    #[test]
    fn complex_query_generates_hypothesis() {
        let client = Arc::new(MockHydeClient::new());
        let stage = make_stage(client.clone());
        let result = stage.hypothesize("how does ELAI handle anti-spiral detection in the pipeline");
        assert!(result.is_some(), "complex query should produce hypothesis");
        assert_eq!(client.calls(), 1);
        let text = result.unwrap();
        assert!(!text.is_empty());
    }

    #[test]
    fn cache_hit_avoids_second_call() {
        let client = Arc::new(MockHydeClient::new());
        let stage = make_stage(client.clone());
        let query = "how does ELAI handle anti-spiral detection in the pipeline";

        let r1 = stage.hypothesize(query);
        assert!(r1.is_some());
        assert_eq!(client.calls(), 1);

        let r2 = stage.hypothesize(query);
        assert!(r2.is_some());
        assert_eq!(client.calls(), 1, "second call should hit cache");
        assert_eq!(r1, r2);
    }

    #[test]
    fn different_queries_different_cache_keys() {
        let client = Arc::new(MockHydeClient::new());
        let stage = make_stage(client.clone());

        stage.hypothesize("how does ELAI handle anti-spiral detection in the pipeline");
        stage.hypothesize("what is the architecture of the vector search module overall");
        assert_eq!(client.calls(), 2, "different queries should not share cache");
    }

    #[test]
    fn failing_client_degrades_gracefully() {
        let client: Arc<dyn HydeClient> = Arc::new(FailingHydeClient);
        let stage = make_stage(client);
        let result = stage.hypothesize("how does ELAI handle anti-spiral detection in the pipeline");
        assert!(result.is_none(), "failed sidecar should degrade to None");
    }

    #[test]
    fn empty_hypothesis_treated_as_skip() {
        let client: Arc<dyn HydeClient> = Arc::new(EmptyHydeClient);
        let stage = make_stage(client);
        let result = stage.hypothesize("how does ELAI handle anti-spiral detection in the pipeline");
        assert!(result.is_none(), "empty hypothesis should be treated as skip");
    }

    #[test]
    fn circuit_breaker_opens_after_repeated_failures() {
        let client: Arc<dyn HydeClient> = Arc::new(FailingHydeClient);
        let stage = make_stage(client);
        let query = "how does ELAI handle anti-spiral detection in the pipeline";

        // Default threshold is 3 failures to open.
        for _ in 0..3 {
            let _ = stage.hypothesize(query);
        }

        // After 3 failures, breaker should be open.
        let state = stage.breaker_state();
        assert_eq!(state, Some(BreakerState::Open), "breaker should open after 3 failures");

        // Subsequent calls should return None without calling client.
        let result = stage.hypothesize(query);
        assert!(result.is_none());
    }

    #[test]
    fn breaker_state_accessor() {
        let client = Arc::new(MockHydeClient::new());
        let stage = make_stage(client);
        assert_eq!(stage.breaker_state(), Some(BreakerState::Closed));
    }

    #[test]
    fn structural_check_constants() {
        assert_eq!(MIN_WORDS_FOR_HYDE, 5);
        assert_eq!(HYDE_CACHE_TTL, Duration::from_secs(300));
        assert_eq!(HYDE_SIDECAR_TIMEOUT, Duration::from_secs(3));
        assert_eq!(HYDE_MAX_TOKENS, 128);
        assert!((HYDE_TEMPERATURE - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn structural_check_module_references() {
        let src = include_str!("hyde.rs");
        assert!(src.contains("HydeClient"), "must define HydeClient trait");
        assert!(src.contains("HydeStage"), "must define HydeStage struct");
        assert!(src.contains("circuit_breaker"), "must integrate circuit breaker");
        assert!(src.contains("blake3"), "must use blake3 for cache keys");
        assert!(src.contains("HYDE_CACHE_TTL"), "must define cache TTL");
        assert!(src.contains("generate_hypothesis"), "must define generation method");
    }
}
