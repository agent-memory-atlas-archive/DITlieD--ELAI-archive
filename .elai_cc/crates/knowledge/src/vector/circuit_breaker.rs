use elai_core::{ClockProvider, LogicalInstant};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Constants (defaults, overridable via Config)
// ---------------------------------------------------------------------------

const DEFAULT_FAILURE_THRESHOLD: u32 = 3;
const DEFAULT_WINDOW_SECS: u64 = 60;
const DEFAULT_HALF_OPEN_BASE_SECS: u64 = 30;
const DEFAULT_BACKOFF_CAP_SECS: u64 = 120;
const DEFAULT_HALF_OPEN_SUCCESS_THRESHOLD: u32 = 2;

// ---------------------------------------------------------------------------
// Error classification
// ---------------------------------------------------------------------------

/// Classifies an ONNX/model error to determine circuit breaker behavior.
/// TRANSIENT errors count toward the failure threshold (3 needed to open).
/// PERMANENT errors open the breaker immediately after 1 failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    /// Timeout, temporary OOM, transient network issue.
    Transient,
    /// Segfault, model file missing, invalid ONNX, unrecoverable corruption.
    Permanent,
}

// ---------------------------------------------------------------------------
// Breaker state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub window: Duration,
    pub half_open_base: Duration,
    pub backoff_cap: Duration,
    pub half_open_success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: DEFAULT_FAILURE_THRESHOLD,
            window: Duration::from_secs(DEFAULT_WINDOW_SECS),
            half_open_base: Duration::from_secs(DEFAULT_HALF_OPEN_BASE_SECS),
            backoff_cap: Duration::from_secs(DEFAULT_BACKOFF_CAP_SECS),
            half_open_success_threshold: DEFAULT_HALF_OPEN_SUCCESS_THRESHOLD,
        }
    }
}

// ---------------------------------------------------------------------------
// Breaker error
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum BreakerError {
    Open,
    CallFailed(String),
}

impl std::fmt::Display for BreakerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "circuit breaker is open"),
            Self::CallFailed(e) => write!(f, "call failed: {e}"),
        }
    }
}

impl std::error::Error for BreakerError {}

// ---------------------------------------------------------------------------
// EmbeddingCircuitBreaker
// ---------------------------------------------------------------------------

/// Per-ONNX-model circuit breaker with failure classification, exponential
/// backoff, deterministic jitter, and HALF_OPEN success counter.
///
/// Independent instance per model (GraphCodeBERT, Granite, cross-encoder, SPLADE).
/// State is NOT checkpointed — resets to CLOSED on process restart.
pub struct EmbeddingCircuitBreaker {
    model_id: String,
    state: BreakerState,
    config: CircuitBreakerConfig,
    clock: Arc<dyn ClockProvider>,

    // Failure tracking (CLOSED state)
    failure_count: u32,
    window_start: LogicalInstant,

    // OPEN state timing
    open_since: LogicalInstant,
    consecutive_opens: u32,

    // HALF_OPEN success counter
    half_open_successes: u32,

    // Deterministic jitter offset derived from model_id hash
    jitter: Duration,
}

impl EmbeddingCircuitBreaker {
    pub fn new(model_id: impl Into<String>, clock: Arc<dyn ClockProvider>, config: CircuitBreakerConfig) -> Self {
        let model_id = model_id.into();
        let jitter = compute_jitter(&model_id, &config);
        let now = clock.now();
        Self {
            model_id,
            state: BreakerState::Closed,
            config,
            clock,
            failure_count: 0,
            window_start: now,
            open_since: now,
            consecutive_opens: 0,
            half_open_successes: 0,
            jitter,
        }
    }

    pub fn with_defaults(model_id: impl Into<String>, clock: Arc<dyn ClockProvider>) -> Self {
        Self::new(model_id, clock, CircuitBreakerConfig::default())
    }

    pub fn state(&self) -> BreakerState {
        self.state
    }

    pub fn model_id(&self) -> &str {
        &self.model_id
    }

    pub fn jitter(&self) -> Duration {
        self.jitter
    }

    /// Returns the current OPEN timeout including exponential backoff and jitter.
    pub fn current_timeout(&self) -> Duration {
        let base = self.config.half_open_base;
        let multiplier = 1u64 << self.consecutive_opens.min(2);
        let backed_off = Duration::from_secs(
            (base.as_secs() * multiplier).min(self.config.backoff_cap.as_secs()),
        );
        backed_off + self.jitter
    }

    /// Check elapsed time and transition Open -> HalfOpen if timeout has passed.
    pub fn sync_state(&mut self) {
        if self.state == BreakerState::Open {
            let elapsed = self.clock.elapsed_since(self.open_since);
            if elapsed >= self.current_timeout() {
                self.state = BreakerState::HalfOpen;
                self.half_open_successes = 0;
            }
        }
    }

    /// Execute a fallible closure through the breaker. On OPEN, returns
    /// `Err(BreakerError::Open)` without calling the closure.
    pub fn call<F, T, E>(&mut self, f: F, failure_kind: FailureKind) -> Result<T, BreakerError>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Debug,
    {
        match self.state {
            BreakerState::Open => {
                let elapsed = self.clock.elapsed_since(self.open_since);
                if elapsed >= self.current_timeout() {
                    self.state = BreakerState::HalfOpen;
                    self.half_open_successes = 0;
                    self.try_call(f, failure_kind)
                } else {
                    Err(BreakerError::Open)
                }
            }
            BreakerState::HalfOpen => self.try_call(f, failure_kind),
            BreakerState::Closed => self.try_call(f, failure_kind),
        }
    }

    /// Convenience: call with `FailureKind::Transient` (most common case).
    pub fn call_transient<F, T, E>(&mut self, f: F) -> Result<T, BreakerError>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Debug,
    {
        self.call(f, FailureKind::Transient)
    }

    /// Record a success externally (e.g., after a HALF_OPEN probe with a fresh session).
    pub fn record_success(&mut self) {
        match self.state {
            BreakerState::Closed => {
                self.failure_count = 0;
                self.window_start = self.clock.now();
            }
            BreakerState::HalfOpen => {
                self.half_open_successes += 1;
                if self.half_open_successes >= self.config.half_open_success_threshold {
                    self.state = BreakerState::Closed;
                    self.failure_count = 0;
                    self.consecutive_opens = 0;
                    self.window_start = self.clock.now();
                }
            }
            BreakerState::Open => {
                // Shouldn't happen in normal flow, but handle gracefully.
                // A success while OPEN means something probed outside the breaker.
            }
        }
    }

    /// Record a failure externally.
    pub fn record_failure(&mut self, kind: FailureKind) {
        let now = self.clock.now();
        match self.state {
            BreakerState::Closed => {
                if kind == FailureKind::Permanent {
                    self.trip_open(now);
                    return;
                }
                // Transient: count within window
                let elapsed = self.clock.elapsed_since(self.window_start);
                if elapsed > self.config.window {
                    self.window_start = now;
                    self.failure_count = 1;
                } else {
                    self.failure_count += 1;
                }
                if self.failure_count >= self.config.failure_threshold {
                    self.trip_open(now);
                }
            }
            BreakerState::HalfOpen => {
                // Any failure in HALF_OPEN -> back to OPEN, reset success counter
                self.half_open_successes = 0;
                self.trip_open(now);
            }
            BreakerState::Open => {
                // Already open; update timestamp for fresh backoff
                self.open_since = now;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Internal
    // -----------------------------------------------------------------------

    fn try_call<F, T, E>(&mut self, f: F, failure_kind: FailureKind) -> Result<T, BreakerError>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Debug,
    {
        match f() {
            Ok(val) => {
                self.record_success();
                Ok(val)
            }
            Err(e) => {
                self.record_failure(failure_kind);
                Err(BreakerError::CallFailed(format!("{e:?}")))
            }
        }
    }

    fn trip_open(&mut self, now: LogicalInstant) {
        let was_half_open = self.state == BreakerState::HalfOpen;
        self.state = BreakerState::Open;
        self.open_since = now;
        if was_half_open {
            // Re-opening after a failed HALF_OPEN probe: escalate backoff
            self.consecutive_opens = self.consecutive_opens.saturating_add(1);
        } else {
            // First open from CLOSED: start at base timeout
            self.consecutive_opens = 0;
        }
        self.failure_count = 0;
        self.half_open_successes = 0;
    }
}

// ---------------------------------------------------------------------------
// Deterministic jitter
// ---------------------------------------------------------------------------

/// Compute deterministic jitter from model_id hash. The jitter is a fraction
/// of the half_open_base timeout, ensuring different models recover at
/// different times to prevent thundering herd.
fn compute_jitter(model_id: &str, config: &CircuitBreakerConfig) -> Duration {
    let mut hasher = DefaultHasher::new();
    model_id.hash(&mut hasher);
    let hash = hasher.finish();
    // Jitter range: 0..half_open_base_secs (in milliseconds for finer granularity)
    let base_ms = u64::try_from(config.half_open_base.as_millis()).unwrap_or(u64::MAX);
    let jitter_ms = hash % base_ms.max(1);
    Duration::from_millis(jitter_ms)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::SeededClock;

    fn make_clock() -> Arc<SeededClock> {
        Arc::new(SeededClock::new())
    }

    fn default_breaker(model_id: &str, clock: Arc<dyn ClockProvider>) -> EmbeddingCircuitBreaker {
        EmbeddingCircuitBreaker::with_defaults(model_id, clock)
    }

    // --- Basic state tests ---

    #[test]
    fn circuit_breaker_starts_closed() {
        let clock = make_clock();
        let cb = default_breaker("graphcodebert", clock);
        assert_eq!(cb.state(), BreakerState::Closed);
    }

    #[test]
    fn circuit_breaker_opens_after_3_transient_failures() {
        let clock = make_clock();
        let mut cb = default_breaker("graphcodebert", clock);

        for _ in 0..3 {
            let _ = cb.call(|| Err::<(), &str>("timeout"), FailureKind::Transient);
        }
        assert_eq!(cb.state(), BreakerState::Open);
    }

    #[test]
    fn circuit_breaker_permanent_failure_opens_immediately() {
        let clock = make_clock();
        let mut cb = default_breaker("graphcodebert", clock);

        let _ = cb.call(|| Err::<(), &str>("segfault"), FailureKind::Permanent);
        assert_eq!(cb.state(), BreakerState::Open);
    }

    #[test]
    fn circuit_breaker_rejects_calls_when_open() {
        let clock = make_clock();
        let mut cb = default_breaker("graphcodebert", clock);

        for _ in 0..3 {
            let _ = cb.call(|| Err::<(), &str>("fail"), FailureKind::Transient);
        }
        assert_eq!(cb.state(), BreakerState::Open);

        let result = cb.call(|| Ok::<(), &str>(()), FailureKind::Transient);
        assert!(matches!(result, Err(BreakerError::Open)));
    }

    // --- Failure window tests ---

    #[test]
    fn circuit_breaker_failures_spread_across_window_stay_closed() {
        let clock = make_clock();
        let mut cb = default_breaker("graphcodebert", clock.clone());

        // Fail at t=0
        let _ = cb.call(|| Err::<(), &str>("fail"), FailureKind::Transient);

        // Advance past 60s window
        for _ in 0..61_000 {
            let _ = clock.now();
        }

        // Fail at t=61s — window resets, count=1
        let _ = cb.call(|| Err::<(), &str>("fail"), FailureKind::Transient);
        assert_eq!(cb.state(), BreakerState::Closed);

        // Advance past another 60s
        for _ in 0..61_000 {
            let _ = clock.now();
        }

        // Fail at t=122s — window resets again, count=1
        let _ = cb.call(|| Err::<(), &str>("fail"), FailureKind::Transient);
        assert_eq!(cb.state(), BreakerState::Closed);
    }

    // --- HALF_OPEN success counter tests ---

    #[test]
    fn circuit_breaker_half_open_requires_2_consecutive_successes() {
        let clock = make_clock();
        let mut cb = default_breaker("graphcodebert", clock.clone());

        // Trip breaker
        for _ in 0..3 {
            let _ = cb.call(|| Err::<(), &str>("fail"), FailureKind::Transient);
        }
        assert_eq!(cb.state(), BreakerState::Open);

        // Advance past timeout (30s + jitter)
        let timeout_ms = u64::try_from(cb.current_timeout().as_millis()).unwrap_or(u64::MAX);
        for _ in 0..=timeout_ms {
            let _ = clock.now();
        }

        // First success in HALF_OPEN
        let r = cb.call(|| Ok::<i32, &str>(1), FailureKind::Transient);
        assert!(r.is_ok());
        assert_eq!(cb.state(), BreakerState::HalfOpen); // Still HALF_OPEN, need 2

        // Second success -> CLOSED
        let r = cb.call(|| Ok::<i32, &str>(2), FailureKind::Transient);
        assert!(r.is_ok());
        assert_eq!(cb.state(), BreakerState::Closed);
    }

    #[test]
    fn circuit_breaker_half_open_failure_after_1_success_resets_to_open() {
        let clock = make_clock();
        let mut cb = default_breaker("graphcodebert", clock.clone());

        // Trip breaker
        for _ in 0..3 {
            let _ = cb.call(|| Err::<(), &str>("fail"), FailureKind::Transient);
        }

        // Advance past timeout
        let timeout_ms = u64::try_from(cb.current_timeout().as_millis()).unwrap_or(u64::MAX);
        for _ in 0..=timeout_ms {
            let _ = clock.now();
        }

        // 1 success
        let _ = cb.call(|| Ok::<i32, &str>(1), FailureKind::Transient);
        assert_eq!(cb.state(), BreakerState::HalfOpen);

        // Then failure -> back to OPEN
        let _ = cb.call(|| Err::<(), &str>("flaky"), FailureKind::Transient);
        assert_eq!(cb.state(), BreakerState::Open);
        assert_eq!(cb.half_open_successes, 0); // counter reset
    }

    // --- Exponential backoff tests ---

    #[test]
    fn circuit_breaker_exponential_backoff() {
        let clock = make_clock();
        let config = CircuitBreakerConfig::default();
        let jitter = compute_jitter("test_model", &config);
        let mut cb = EmbeddingCircuitBreaker::new("test_model", clock.clone(), config);

        // First open: base=30s
        for _ in 0..3 {
            let _ = cb.call(|| Err::<(), &str>("fail"), FailureKind::Transient);
        }
        assert_eq!(cb.state(), BreakerState::Open);
        assert_eq!(
            cb.current_timeout(),
            Duration::from_secs(30) + jitter
        );

        // Advance past timeout, enter HALF_OPEN, then fail -> second OPEN
        let timeout_ms = u64::try_from(cb.current_timeout().as_millis()).unwrap_or(u64::MAX);
        for _ in 0..=timeout_ms {
            let _ = clock.now();
        }
        let _ = cb.call(|| Err::<(), &str>("fail again"), FailureKind::Transient);
        assert_eq!(cb.state(), BreakerState::Open);
        assert_eq!(
            cb.current_timeout(),
            Duration::from_secs(60) + jitter
        );

        // Third OPEN: advance, fail again
        let timeout_ms = u64::try_from(cb.current_timeout().as_millis()).unwrap_or(u64::MAX);
        for _ in 0..=timeout_ms {
            let _ = clock.now();
        }
        let _ = cb.call(|| Err::<(), &str>("fail again"), FailureKind::Transient);
        assert_eq!(cb.state(), BreakerState::Open);
        assert_eq!(
            cb.current_timeout(),
            Duration::from_secs(120) + jitter
        );

        // Fourth OPEN: capped at 120s
        let timeout_ms = u64::try_from(cb.current_timeout().as_millis()).unwrap_or(u64::MAX);
        for _ in 0..=timeout_ms {
            let _ = clock.now();
        }
        let _ = cb.call(|| Err::<(), &str>("fail"), FailureKind::Transient);
        assert_eq!(cb.state(), BreakerState::Open);
        assert_eq!(
            cb.current_timeout(),
            Duration::from_secs(120) + jitter,
            "backoff capped at 120s"
        );
    }

    // --- Jitter tests ---

    #[test]
    fn circuit_breaker_jitter_is_deterministic_for_same_model() {
        let config = CircuitBreakerConfig::default();
        let j1 = compute_jitter("graphcodebert", &config);
        let j2 = compute_jitter("graphcodebert", &config);
        assert_eq!(j1, j2, "same model_id must produce same jitter");
    }

    #[test]
    fn circuit_breaker_different_models_get_different_jitter() {
        let config = CircuitBreakerConfig::default();
        let models = ["graphcodebert", "granite", "cross_encoder", "splade"];
        let jitters: Vec<Duration> = models.iter().map(|m| compute_jitter(m, &config)).collect();

        // With 4 models and a 30s jitter range, collisions are astronomically unlikely.
        // We verify all 4 are distinct.
        for i in 0..jitters.len() {
            for j in (i + 1)..jitters.len() {
                assert_ne!(
                    jitters[i], jitters[j],
                    "models {} and {} should have different jitter",
                    models[i], models[j]
                );
            }
        }
    }

    // --- Independence tests ---

    #[test]
    fn circuit_breaker_independent_per_model() {
        let clock = make_clock();
        let mut gcb = default_breaker("graphcodebert", clock.clone());
        let granite = default_breaker("granite", clock.clone());
        let cross = default_breaker("cross_encoder", clock.clone());
        let splade = default_breaker("splade", clock);

        // Trip only graphcodebert
        for _ in 0..3 {
            let _ = gcb.call(|| Err::<(), &str>("fail"), FailureKind::Transient);
        }
        assert_eq!(gcb.state(), BreakerState::Open);

        // Others remain CLOSED
        assert_eq!(granite.state(), BreakerState::Closed);
        assert_eq!(cross.state(), BreakerState::Closed);
        assert_eq!(splade.state(), BreakerState::Closed);
    }

    // --- Permanent failure tests ---

    #[test]
    fn circuit_breaker_permanent_opens_after_1_failure_not_3() {
        let clock = make_clock();
        let mut cb = default_breaker("graphcodebert", clock);

        // Single permanent failure should open immediately
        let _ = cb.call(|| Err::<(), &str>("model file missing"), FailureKind::Permanent);
        assert_eq!(cb.state(), BreakerState::Open);
    }

    // --- Sync state test ---

    #[test]
    fn circuit_breaker_sync_state_transitions_open_to_half_open() {
        let clock = make_clock();
        let mut cb = default_breaker("graphcodebert", clock.clone());

        for _ in 0..3 {
            let _ = cb.call(|| Err::<(), &str>("fail"), FailureKind::Transient);
        }
        assert_eq!(cb.state(), BreakerState::Open);

        let timeout_ms = u64::try_from(cb.current_timeout().as_millis()).unwrap_or(u64::MAX);
        for _ in 0..=timeout_ms {
            let _ = clock.now();
        }

        cb.sync_state();
        assert_eq!(cb.state(), BreakerState::HalfOpen);
    }

    // --- Recovery resets consecutive_opens ---

    #[test]
    fn circuit_breaker_full_recovery_resets_backoff() {
        let clock = make_clock();
        let mut cb = default_breaker("test_model", clock.clone());

        // Trip once
        for _ in 0..3 {
            let _ = cb.call(|| Err::<(), &str>("fail"), FailureKind::Transient);
        }

        // Recover via HALF_OPEN
        let timeout_ms = u64::try_from(cb.current_timeout().as_millis()).unwrap_or(u64::MAX);
        for _ in 0..=timeout_ms {
            let _ = clock.now();
        }
        let _ = cb.call(|| Ok::<i32, &str>(1), FailureKind::Transient);
        let _ = cb.call(|| Ok::<i32, &str>(2), FailureKind::Transient);
        assert_eq!(cb.state(), BreakerState::Closed);
        assert_eq!(cb.consecutive_opens, 0);

        // Trip again — should start from 30s base, not 60s
        for _ in 0..3 {
            let _ = cb.call(|| Err::<(), &str>("fail"), FailureKind::Transient);
        }
        let jitter = cb.jitter();
        assert_eq!(
            cb.current_timeout(),
            Duration::from_secs(30) + jitter,
            "after full recovery, backoff resets to base"
        );
    }

    // --- Success in CLOSED resets failure count ---

    #[test]
    fn circuit_breaker_success_in_closed_resets_failure_count() {
        let clock = make_clock();
        let mut cb = default_breaker("graphcodebert", clock);

        // 2 failures (below threshold)
        let _ = cb.call(|| Err::<(), &str>("fail"), FailureKind::Transient);
        let _ = cb.call(|| Err::<(), &str>("fail"), FailureKind::Transient);
        assert_eq!(cb.state(), BreakerState::Closed);
        assert_eq!(cb.failure_count, 2);

        // Success resets count
        let _ = cb.call(|| Ok::<i32, &str>(1), FailureKind::Transient);
        assert_eq!(cb.failure_count, 0);
    }
}
