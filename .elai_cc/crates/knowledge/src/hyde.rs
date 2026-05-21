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

use crate::knowledge_ssb_sink::{
    noop_sink, HydeQueryEvent, KnowledgeSsbSink,
};
use crate::query_class::{classify, QueryClass};
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

/// E-17 SP2 task 5: env-var name controlling whether HyDE runs for
/// non-NaturalLanguage queries. Default = "1" (true) — when set the
/// `should_use_hyde` gate returns false for CodeSymbol / Mixed queries
/// even if the tier dispatch already enabled HyDE.
///
/// Honest framing: this is an extrapolation from arxiv:2212.10496 NL-focus
/// design intent. Not externally validated for code queries; SP6 measures
/// recall@5 with/without the gate.
pub const ELAI_HYDE_NL_ONLY_ENV: &str = "ELAI_HYDE_NL_ONLY";

/// E-17 SP2 task 5: NL-only gate for HyDE.
///
/// Returns `true` ONLY when:
///   1. Query has >= [`MIN_WORDS_FOR_HYDE`] words, AND
///   2. The NL-only gate is disabled (`ELAI_HYDE_NL_ONLY=0`), OR the
///      heuristic classifier ([`crate::query_class::classify`]) returns
///      [`QueryClass::NaturalLanguage`].
///
/// CodeSymbol queries and Mixed queries (when the gate is on, default)
/// return `false` — the pipeline skips HyDE and emits a
/// `hyde_skipped_code_query` SSB row. The original query still flows
/// through BM25 + vector search; only the LLM-roundtrip is skipped.
pub fn should_use_hyde(query: &str) -> bool {
    let word_count = query.split_whitespace().count();
    if word_count < MIN_WORDS_FOR_HYDE {
        return false;
    }
    if !nl_only_gate_enabled() {
        // Gate disabled -> respect tier-only routing (existing behaviour).
        return true;
    }
    matches!(classify(query), QueryClass::NaturalLanguage)
}

/// Read [`ELAI_HYDE_NL_ONLY_ENV`] and return `true` when the gate is on.
/// Default (unset / invalid) = `true` (NL-only by default per E-17 task 5).
fn nl_only_gate_enabled() -> bool {
    match std::env::var(ELAI_HYDE_NL_ONLY_ENV) {
        Ok(v) => {
            let t = v.trim();
            !(t == "0" || t.eq_ignore_ascii_case("false") || t.eq_ignore_ascii_case("off"))
        }
        Err(_) => true,
    }
}

// C-1: convert `Duration` to milliseconds clamped to `u64::MAX`. Used
// for HyDE latency payloads. Module-private to keep the helper close to
// its single call site.
fn duration_to_ms_clamped(d: Duration) -> u64 {
    let ms_u128 = d.as_millis();
    if ms_u128 > u128::from(u64::MAX) {
        u64::MAX
    } else {
        ms_u128 as u64
    }
}

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
    /// C-1: typed SSB sink for per-hypothesize beacons. Production wiring
    /// uses the orchestrator's `OrchestratorKnowledgeSsbSink`; tests and
    /// inert paths use `NoopKnowledgeSsbSink`.
    ssb_sink: Arc<dyn KnowledgeSsbSink>,
}

impl HydeStage {
    /// Create a new HyDE stage with default circuit breaker configuration
    /// and a no-op SSB sink. Production callers should prefer
    /// [`HydeStage::with_ssb`] to wire the orchestrator adapter so HyDE
    /// rows reach the dashboard.
    pub fn new(client: Arc<dyn HydeClient>, clock: Arc<dyn ClockProvider>) -> Self {
        Self::with_ssb(client, clock, noop_sink())
    }

    /// Create a new HyDE stage with a typed SSB sink installed. Every
    /// call to [`HydeStage::hypothesize`] will emit one
    /// `KnowledgeSsbSink::emit_hyde_query` row via this sink.
    pub fn with_ssb(
        client: Arc<dyn HydeClient>,
        clock: Arc<dyn ClockProvider>,
        ssb_sink: Arc<dyn KnowledgeSsbSink>,
    ) -> Self {
        let breaker = EmbeddingCircuitBreaker::with_defaults("hyde_sidecar", clock.clone());
        Self {
            client,
            cache: Mutex::new(FxHashMap::default()),
            breaker: Mutex::new(breaker),
            clock,
            ssb_sink,
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
    ///
    /// C-1: emits one [`HydeQueryEvent`] through the configured
    /// [`KnowledgeSsbSink`] for every call that passes the
    /// `MIN_WORDS_FOR_HYDE` guard. Short-query rejections are NOT
    /// emitted (they happen before the cache key is computed and would
    /// flood the dashboard for trivial queries).
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

        // C-1: capture start instant for latency measurement (entropy
        // quarantine: ClockProvider only, never std::time::Instant).
        let started = self.clock.now();

        // Cache lookup (blake3 key). The full 32-byte hash is hex-encoded
        // for the SSB payload so duplicate-query analysis on the dashboard
        // side can group by the full hash without truncation collisions.
        let raw_hash = blake3::hash(query.as_bytes());
        let key_bytes: [u8; 32] = *raw_hash.as_bytes();
        let query_hash_hex = raw_hash.to_hex().to_string();

        if let Ok(cache) = self.cache.lock() {
            if let Some(entry) = cache.get(&key_bytes) {
                let age = self.clock.elapsed_since(entry.created_at);
                if age < HYDE_CACHE_TTL {
                    // NARROWING-OK: cache age never exceeds u64::MAX ms (584M years).
                    tracing::debug!(
                        age_ms = age.as_millis().min(u64::MAX as u128) as u64,
                        "hyde: cache hit"
                    );
                    let text = entry.text.clone();
                    // Drop the cache lock before emitting so the sink
                    // dispatch path never holds the cache mutex.
                    drop(cache);
                    let latency_ms = duration_to_ms_clamped(self.clock.elapsed_since(started));
                    let hypothesis_len = u32::try_from(text.len()).unwrap_or(u32::MAX);
                    self.ssb_sink.emit_hyde_query(HydeQueryEvent {
                        query_hash: query_hash_hex,
                        hypothesis_len,
                        cache_hit: true,
                        latency_ms,
                    });
                    return Some(text);
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

        let latency_ms = duration_to_ms_clamped(self.clock.elapsed_since(started));
        let hypothesis_len = hypothesis
            .as_ref()
            .map(|t| u32::try_from(t.len()).unwrap_or(u32::MAX))
            .unwrap_or(0);
        self.ssb_sink.emit_hyde_query(HydeQueryEvent {
            query_hash: query_hash_hex,
            hypothesis_len,
            cache_hit: false,
            latency_ms,
        });

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
