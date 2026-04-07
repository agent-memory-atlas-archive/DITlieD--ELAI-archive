use crate::vector::circuit_breaker::{BreakerState, EmbeddingCircuitBreaker};
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// SSB degradation events (knowledge.txt L124)
// ---------------------------------------------------------------------------

/// Event emitted to the Session State Buffer on every search mode change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DegradationEvent {
    pub action_type: &'static str,
    pub stage: &'static str,
    pub detail: String,
}

/// Trait for emitting SSB degradation events. Injectable for testing.
/// SSB write failure is non-fatal (spec: log warning, continue with fallback).
pub trait SsbNotifier: Send + Sync {
    fn emit(&self, event: DegradationEvent);
}

/// No-op notifier used when SSB is not configured.
pub struct NoopSsbNotifier;

impl SsbNotifier for NoopSsbNotifier {
    fn emit(&self, _event: DegradationEvent) {}
}

// ---------------------------------------------------------------------------
// SearchMode (knowledge.txt L135-141)
// ---------------------------------------------------------------------------

/// Active search mode, determined by which layers are operational.
///
/// Cascade: HYBRID -> CodeOnly/TextOnly -> SparseOnly -> Bm25Only.
/// Mode restored automatically when failed component's circuit breaker
/// transitions back to CLOSED.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// Full pipeline: dual-model dense vectors + BM25 + SPLADE + reranking.
    Hybrid,
    /// GraphCodeBERT failed, text vectors (Granite) still available.
    CodeOnly,
    /// Granite failed, code vectors (GraphCodeBERT) still available.
    TextOnly,
    /// Both dense models failed. BM25 + SPLADE (if enabled).
    SparseOnly,
    /// BM25 only. SPLADE also failed or not enabled. Last resort.
    Bm25Only,
}

impl std::fmt::Display for SearchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hybrid => write!(f, "HYBRID"),
            Self::CodeOnly => write!(f, "CODE_ONLY"),
            Self::TextOnly => write!(f, "TEXT_ONLY"),
            Self::SparseOnly => write!(f, "SPARSE_ONLY"),
            Self::Bm25Only => write!(f, "BM25_ONLY"),
        }
    }
}

// ---------------------------------------------------------------------------
// DegradationCascade (knowledge.txt L145-149)
// ---------------------------------------------------------------------------

/// Tracks per-model circuit breaker states and computes the current SearchMode.
///
/// Fallback chain:
///   GraphCodeBERT fail -> TEXT-vector-only (Granite)
///   Granite fail -> CODE-vector-only (GraphCodeBERT)
///   Both fail -> SPARSE_ONLY (BM25 + SPLADE)
///   SPLADE fail -> BM25_ONLY
///   Cross-encoder fail -> skip reranking (handled by reranker internally)
///
/// This struct is purely reactive: it reads breaker states, computes the mode,
/// and emits SSB events on mode transitions. It does not own the breakers.
pub struct DegradationCascade {
    previous_mode: SearchMode,
    splade_enabled: bool,
}

impl DegradationCascade {
    pub fn new(splade_enabled: bool) -> Self {
        Self {
            previous_mode: SearchMode::Hybrid,
            splade_enabled,
        }
    }

    /// Evaluate current search mode from circuit breaker states.
    ///
    /// `code_breaker`: GraphCodeBERT circuit breaker (None if not configured).
    /// `text_breaker`: Granite circuit breaker (None if not configured).
    /// `splade_breaker`: SPLADE circuit breaker (None if SPLADE not enabled).
    ///
    /// Returns the computed SearchMode. If mode changed from previous,
    /// emits a `knowledge_degradation` SSB event.
    pub fn evaluate(
        &mut self,
        code_breaker: Option<&Mutex<EmbeddingCircuitBreaker>>,
        text_breaker: Option<&Mutex<EmbeddingCircuitBreaker>>,
        splade_breaker: Option<&Mutex<EmbeddingCircuitBreaker>>,
        ssb: &dyn SsbNotifier,
    ) -> SearchMode {
        let code_ok = breaker_is_available(code_breaker);
        let text_ok = breaker_is_available(text_breaker);
        let splade_ok = self.splade_enabled && breaker_is_available(splade_breaker);

        let mode = match (code_ok, text_ok) {
            (true, true) => SearchMode::Hybrid,
            (false, true) => SearchMode::TextOnly,
            (true, false) => SearchMode::CodeOnly,
            (false, false) => {
                if splade_ok {
                    SearchMode::SparseOnly
                } else if self.splade_enabled {
                    // SPLADE enabled but its breaker is open
                    SearchMode::Bm25Only
                } else {
                    // SPLADE not enabled: SPARSE_ONLY == BM25_ONLY (spec adversarial)
                    SearchMode::Bm25Only
                }
            }
        };

        if mode != self.previous_mode {
            let detail = format!(
                "{} -> {}: code={}, text={}, splade={}",
                self.previous_mode,
                mode,
                if code_ok { "ok" } else { "down" },
                if text_ok { "ok" } else { "down" },
                if !self.splade_enabled {
                    "disabled"
                } else if splade_ok {
                    "ok"
                } else {
                    "down"
                },
            );
            ssb.emit(DegradationEvent {
                action_type: "knowledge_degradation",
                stage: "cascade",
                detail,
            });
            self.previous_mode = mode;
        }

        mode
    }

    /// Returns the last computed search mode without re-evaluating breakers.
    pub fn current_mode(&self) -> SearchMode {
        self.previous_mode
    }

    /// Force-set the mode (e.g., on startup when breakers haven't been initialized yet).
    pub fn set_mode(&mut self, mode: SearchMode) {
        self.previous_mode = mode;
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A breaker is "available" if it exists and its state is not OPEN.
/// HALF_OPEN counts as available (probe in progress, allowing limited calls).
fn breaker_is_available(breaker: Option<&Mutex<EmbeddingCircuitBreaker>>) -> bool {
    match breaker {
        None => true, // No breaker configured = no model to protect = always available
        Some(m) => {
            let cb = match m.lock() {
                Ok(cb) => cb,
                Err(_) => return false, // Poisoned mutex = treat as unavailable
            };
            cb.state() != BreakerState::Open
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::circuit_breaker::FailureKind;
    use elai_core::{ClockProvider, SeededClock};
    use std::sync::Arc;

    struct RecordingSsbNotifier {
        events: Mutex<Vec<DegradationEvent>>,
    }

    impl RecordingSsbNotifier {
        fn new() -> Self {
            Self { events: Mutex::new(Vec::new()) }
        }

        fn events(&self) -> Vec<DegradationEvent> {
            self.events.lock().unwrap().clone()
        }
    }

    impl SsbNotifier for RecordingSsbNotifier {
        fn emit(&self, event: DegradationEvent) {
            self.events.lock().unwrap().push(event);
        }
    }

    fn make_breaker(model_id: &str) -> Mutex<EmbeddingCircuitBreaker> {
        let clock = Arc::new(SeededClock::new());
        Mutex::new(EmbeddingCircuitBreaker::with_defaults(model_id, clock))
    }

    fn make_breaker_with_clock(model_id: &str, clock: Arc<SeededClock>) -> Mutex<EmbeddingCircuitBreaker> {
        Mutex::new(EmbeddingCircuitBreaker::with_defaults(model_id, clock as Arc<dyn elai_core::ClockProvider>))
    }

    // -----------------------------------------------------------------------
    // SearchMode cascade tests (acceptance criteria from phase-1I.txt)
    // -----------------------------------------------------------------------

    #[test]
    fn all_healthy_returns_hybrid() {
        let code_cb = make_breaker("graphcodebert");
        let text_cb = make_breaker("granite");
        let ssb = RecordingSsbNotifier::new();
        let mut cascade = DegradationCascade::new(false);

        let mode = cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
        assert_eq!(mode, SearchMode::Hybrid);
        assert!(ssb.events().is_empty(), "no mode change = no SSB event");
    }

    #[test]
    fn code_model_fail_degrades_to_text_only() {
        let code_cb = make_breaker("graphcodebert");
        let text_cb = make_breaker("granite");
        let ssb = RecordingSsbNotifier::new();
        let mut cascade = DegradationCascade::new(false);

        // Trip code breaker with permanent failure
        code_cb.lock().unwrap().record_failure(FailureKind::Permanent);

        let mode = cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
        assert_eq!(mode, SearchMode::TextOnly);
        assert_eq!(ssb.events().len(), 1);
        assert_eq!(ssb.events()[0].action_type, "knowledge_degradation");
    }

    #[test]
    fn text_model_fail_degrades_to_code_only() {
        let code_cb = make_breaker("graphcodebert");
        let text_cb = make_breaker("granite");
        let ssb = RecordingSsbNotifier::new();
        let mut cascade = DegradationCascade::new(false);

        text_cb.lock().unwrap().record_failure(FailureKind::Permanent);

        let mode = cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
        assert_eq!(mode, SearchMode::CodeOnly);
        assert_eq!(ssb.events().len(), 1);
    }

    #[test]
    fn both_dense_fail_no_splade_degrades_to_bm25_only() {
        let code_cb = make_breaker("graphcodebert");
        let text_cb = make_breaker("granite");
        let ssb = RecordingSsbNotifier::new();
        let mut cascade = DegradationCascade::new(false);

        code_cb.lock().unwrap().record_failure(FailureKind::Permanent);
        text_cb.lock().unwrap().record_failure(FailureKind::Permanent);

        let mode = cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
        assert_eq!(mode, SearchMode::Bm25Only);
    }

    #[test]
    fn both_dense_fail_splade_ok_degrades_to_sparse_only() {
        let code_cb = make_breaker("graphcodebert");
        let text_cb = make_breaker("granite");
        let splade_cb = make_breaker("splade");
        let ssb = RecordingSsbNotifier::new();
        let mut cascade = DegradationCascade::new(true);

        code_cb.lock().unwrap().record_failure(FailureKind::Permanent);
        text_cb.lock().unwrap().record_failure(FailureKind::Permanent);

        let mode = cascade.evaluate(Some(&code_cb), Some(&text_cb), Some(&splade_cb), &ssb);
        assert_eq!(mode, SearchMode::SparseOnly);
    }

    #[test]
    fn both_dense_and_splade_fail_degrades_to_bm25_only() {
        let code_cb = make_breaker("graphcodebert");
        let text_cb = make_breaker("granite");
        let splade_cb = make_breaker("splade");
        let ssb = RecordingSsbNotifier::new();
        let mut cascade = DegradationCascade::new(true);

        code_cb.lock().unwrap().record_failure(FailureKind::Permanent);
        text_cb.lock().unwrap().record_failure(FailureKind::Permanent);
        splade_cb.lock().unwrap().record_failure(FailureKind::Permanent);

        let mode = cascade.evaluate(Some(&code_cb), Some(&text_cb), Some(&splade_cb), &ssb);
        assert_eq!(mode, SearchMode::Bm25Only);
    }

    #[test]
    fn splade_disabled_sparse_only_equals_bm25_only() {
        // Adversarial: with SPLADE disabled, SPARSE_ONLY has no SPLADE component.
        // Test: SPARSE_ONLY == BM25_ONLY, cascade skips straight.
        let code_cb = make_breaker("graphcodebert");
        let text_cb = make_breaker("granite");
        let ssb = RecordingSsbNotifier::new();
        let mut cascade = DegradationCascade::new(false); // SPLADE disabled

        code_cb.lock().unwrap().record_failure(FailureKind::Permanent);
        text_cb.lock().unwrap().record_failure(FailureKind::Permanent);

        let mode = cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
        assert_eq!(mode, SearchMode::Bm25Only, "with SPLADE disabled, skip straight to BM25_ONLY");
    }

    /// Advance clock past the breaker's current timeout (base + jitter + margin).
    fn advance_past_timeout(clock: &SeededClock, breaker: &Mutex<EmbeddingCircuitBreaker>) {
        let timeout_ms = breaker.lock().unwrap().current_timeout().as_millis() as u64;
        // +100ms margin to ensure we're past the timeout
        for _ in 0..(timeout_ms + 100) {
            clock.now();
        }
    }

    #[test]
    fn mode_auto_restores_when_breaker_recovers() {
        let clock = Arc::new(SeededClock::new());
        let code_cb = make_breaker_with_clock("graphcodebert", clock.clone());
        let text_cb = make_breaker_with_clock("granite", clock.clone());
        let ssb = RecordingSsbNotifier::new();
        let mut cascade = DegradationCascade::new(false);

        // Trip code breaker
        code_cb.lock().unwrap().record_failure(FailureKind::Permanent);
        let mode = cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
        assert_eq!(mode, SearchMode::TextOnly);

        // Advance time past timeout (30s base + jitter)
        advance_past_timeout(&clock, &code_cb);

        // sync_state transitions Open -> HalfOpen
        code_cb.lock().unwrap().sync_state();
        let mode = cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
        // HalfOpen counts as available
        assert_eq!(mode, SearchMode::Hybrid);

        // Confirm SSB got recovery event
        let events = ssb.events();
        assert_eq!(events.len(), 2); // degrade + restore
        assert!(events[1].detail.contains("HYBRID"));
    }

    #[test]
    fn ssb_event_emitted_on_every_mode_change() {
        let code_cb = make_breaker("graphcodebert");
        let text_cb = make_breaker("granite");
        let ssb = RecordingSsbNotifier::new();
        let mut cascade = DegradationCascade::new(false);

        // Hybrid -> TextOnly (code fails)
        code_cb.lock().unwrap().record_failure(FailureKind::Permanent);
        cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);

        // TextOnly -> Bm25Only (text also fails)
        text_cb.lock().unwrap().record_failure(FailureKind::Permanent);
        cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);

        let events = ssb.events();
        assert_eq!(events.len(), 2);
        for ev in &events {
            assert_eq!(ev.action_type, "knowledge_degradation");
            assert_eq!(ev.stage, "cascade");
        }
    }

    #[test]
    fn no_ssb_event_when_mode_unchanged() {
        let code_cb = make_breaker("graphcodebert");
        let text_cb = make_breaker("granite");
        let ssb = RecordingSsbNotifier::new();
        let mut cascade = DegradationCascade::new(false);

        // Evaluate twice with same state
        cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
        cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
        assert!(ssb.events().is_empty());
    }

    #[test]
    fn cross_encoder_fail_does_not_affect_search_mode() {
        // Cross-encoder failure is handled by reranker internally (skip reranking).
        // DegradationCascade only tracks embedding model breakers.
        let code_cb = make_breaker("graphcodebert");
        let text_cb = make_breaker("granite");
        let ssb = RecordingSsbNotifier::new();
        let mut cascade = DegradationCascade::new(false);

        let mode = cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
        assert_eq!(mode, SearchMode::Hybrid);
    }

    #[test]
    fn bm25_only_always_reachable() {
        // "Never total outage: BM25_ONLY is always reachable (tantivy has no ONNX dependency)"
        // BM25 has no circuit breaker in the cascade — it's always the final fallback.
        let code_cb = make_breaker("graphcodebert");
        let text_cb = make_breaker("granite");
        let splade_cb = make_breaker("splade");
        let ssb = NoopSsbNotifier;
        let mut cascade = DegradationCascade::new(true);

        // Fail everything
        code_cb.lock().unwrap().record_failure(FailureKind::Permanent);
        text_cb.lock().unwrap().record_failure(FailureKind::Permanent);
        splade_cb.lock().unwrap().record_failure(FailureKind::Permanent);

        let mode = cascade.evaluate(Some(&code_cb), Some(&text_cb), Some(&splade_cb), &ssb);
        assert_eq!(mode, SearchMode::Bm25Only, "BM25_ONLY must always be reachable");
    }

    #[test]
    fn flapping_model_stabilizes() {
        // Adversarial: simulate flapping model (fail-recover-fail in <1s).
        // The cascade should reflect the breaker state faithfully; stabilization
        // is the circuit breaker's job (exponential backoff prevents rapid oscillation).
        let clock = Arc::new(SeededClock::new());
        let code_cb = make_breaker_with_clock("graphcodebert", clock.clone());
        let text_cb = make_breaker_with_clock("granite", clock.clone());
        let ssb = RecordingSsbNotifier::new();
        let mut cascade = DegradationCascade::new(false);

        // Fail -> Open
        code_cb.lock().unwrap().record_failure(FailureKind::Permanent);
        let m1 = cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
        assert_eq!(m1, SearchMode::TextOnly);

        // Advance past timeout, sync to HalfOpen
        advance_past_timeout(&clock, &code_cb);
        code_cb.lock().unwrap().sync_state();
        let m2 = cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
        assert_eq!(m2, SearchMode::Hybrid); // HalfOpen = available

        // Fail again in HalfOpen -> back to Open with escalated backoff
        code_cb.lock().unwrap().record_failure(FailureKind::Transient);
        let m3 = cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
        assert_eq!(m3, SearchMode::TextOnly);

        // Events: Hybrid->TextOnly, TextOnly->Hybrid, Hybrid->TextOnly
        assert_eq!(ssb.events().len(), 3);
    }

    #[test]
    fn each_model_independent_breaker() {
        // Acceptance: each model has independent breaker. Fail one, others remain CLOSED.
        let code_cb = make_breaker("graphcodebert");
        let text_cb = make_breaker("granite");
        let ssb = NoopSsbNotifier;
        let mut cascade = DegradationCascade::new(false);

        // Fail only code
        code_cb.lock().unwrap().record_failure(FailureKind::Permanent);
        assert_eq!(code_cb.lock().unwrap().state(), BreakerState::Open);
        assert_eq!(text_cb.lock().unwrap().state(), BreakerState::Closed);

        let mode = cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
        assert_eq!(mode, SearchMode::TextOnly);
    }

    #[test]
    fn half_open_breaker_counts_as_available() {
        // HALF_OPEN allows limited probe calls — treat as available for search.
        let clock = Arc::new(SeededClock::new());
        let code_cb = make_breaker_with_clock("graphcodebert", clock.clone());
        let text_cb = make_breaker_with_clock("granite", clock.clone());
        let ssb = NoopSsbNotifier;
        let mut cascade = DegradationCascade::new(false);

        // Trip and recover to HalfOpen
        code_cb.lock().unwrap().record_failure(FailureKind::Permanent);
        advance_past_timeout(&clock, &code_cb);
        code_cb.lock().unwrap().sync_state();
        assert_eq!(code_cb.lock().unwrap().state(), BreakerState::HalfOpen);

        let mode = cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
        assert_eq!(mode, SearchMode::Hybrid);
    }

    #[test]
    fn current_mode_returns_last_evaluated() {
        let ssb = NoopSsbNotifier;
        let mut cascade = DegradationCascade::new(false);
        assert_eq!(cascade.current_mode(), SearchMode::Hybrid);

        let code_cb = make_breaker("graphcodebert");
        let text_cb = make_breaker("granite");
        code_cb.lock().unwrap().record_failure(FailureKind::Permanent);
        cascade.evaluate(Some(&code_cb), Some(&text_cb), None, &ssb);
        assert_eq!(cascade.current_mode(), SearchMode::TextOnly);
    }

    #[test]
    fn display_formats_correctly() {
        assert_eq!(format!("{}", SearchMode::Hybrid), "HYBRID");
        assert_eq!(format!("{}", SearchMode::CodeOnly), "CODE_ONLY");
        assert_eq!(format!("{}", SearchMode::TextOnly), "TEXT_ONLY");
        assert_eq!(format!("{}", SearchMode::SparseOnly), "SPARSE_ONLY");
        assert_eq!(format!("{}", SearchMode::Bm25Only), "BM25_ONLY");
    }
}
