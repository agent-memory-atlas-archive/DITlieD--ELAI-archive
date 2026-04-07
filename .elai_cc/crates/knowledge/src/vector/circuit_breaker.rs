use elai_core::{ClockProvider, LogicalInstant};
use std::sync::Arc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const FAILURE_THRESHOLD: u32 = 3;
const WINDOW_SECS: u64 = 60;
const HALF_OPEN_AFTER_SECS: u64 = 30;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

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
// CircuitBreaker
// ---------------------------------------------------------------------------

pub struct CircuitBreaker {
    state: BreakerState,
    failure_count: u32,
    window_start: LogicalInstant,
    clock: Arc<dyn ClockProvider>,
}

impl CircuitBreaker {
    pub fn new(clock: Arc<dyn ClockProvider>) -> Self {
        let now = clock.now();
        Self {
            state: BreakerState::Closed,
            failure_count: 0,
            window_start: now,
            clock,
        }
    }

    pub fn state(&self) -> BreakerState {
        self.state
    }

    pub fn call<F, T, E>(&mut self, f: F) -> Result<T, BreakerError>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Debug,
    {
        match self.state {
            BreakerState::Open => {
                let elapsed = self.clock.elapsed_since(self.window_start);
                if elapsed >= Duration::from_secs(HALF_OPEN_AFTER_SECS) {
                    self.state = BreakerState::HalfOpen;
                } else {
                    return Err(BreakerError::Open);
                }
                self.try_call(f)
            }
            BreakerState::HalfOpen => self.try_call(f),
            BreakerState::Closed => self.try_call(f),
        }
    }

    fn try_call<F, T, E>(&mut self, f: F) -> Result<T, BreakerError>
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
                let now = self.clock.now();
                self.record_failure(now);
                Err(BreakerError::CallFailed(format!("{e:?}")))
            }
        }
    }

    /// Check elapsed time and transition Open -> HalfOpen if timeout has passed.
    /// Does not execute any closure — purely updates internal state.
    pub fn sync_state(&mut self) {
        if self.state == BreakerState::Open {
            let elapsed = self.clock.elapsed_since(self.window_start);
            if elapsed >= Duration::from_secs(HALF_OPEN_AFTER_SECS) {
                self.state = BreakerState::HalfOpen;
            }
        }
    }

    pub fn record_success(&mut self) {
        self.state = BreakerState::Closed;
        self.failure_count = 0;
        self.window_start = self.clock.now();
    }

    pub fn record_failure(&mut self, clock_now: LogicalInstant) {
        let elapsed = self.clock.elapsed_since(self.window_start);
        if elapsed > Duration::from_secs(WINDOW_SECS) {
            self.window_start = clock_now;
            self.failure_count = 1;
        } else {
            self.failure_count += 1;
        }
        if self.failure_count >= FAILURE_THRESHOLD {
            self.state = BreakerState::Open;
            self.window_start = clock_now;
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

    fn make_clock() -> Arc<SeededClock> {
        Arc::new(SeededClock::new())
    }

    #[test]
    fn circuit_breaker_starts_closed() {
        let clock = make_clock();
        let cb = CircuitBreaker::new(clock);
        assert_eq!(cb.state(), BreakerState::Closed);
    }

    #[test]
    fn circuit_breaker_opens_after_threshold_failures() {
        let clock = make_clock();
        let mut cb = CircuitBreaker::new(clock.clone());

        for _ in 0..FAILURE_THRESHOLD {
            let _ = cb.call(|| Err::<(), &str>("fail"));
        }
        assert_eq!(cb.state(), BreakerState::Open);
    }

    #[test]
    fn circuit_breaker_rejects_calls_when_open() {
        let clock = make_clock();
        let mut cb = CircuitBreaker::new(clock.clone());

        for _ in 0..FAILURE_THRESHOLD {
            let _ = cb.call(|| Err::<(), &str>("fail"));
        }
        assert_eq!(cb.state(), BreakerState::Open);

        let result = cb.call(|| Ok::<(), &str>(()));
        assert!(result.is_err());
        match result.unwrap_err() {
            BreakerError::Open => {}
            other => panic!("expected Open, got {other:?}"),
        }
    }

    #[test]
    fn circuit_breaker_transitions_to_half_open_after_timeout() {
        let clock = make_clock();
        let mut cb = CircuitBreaker::new(clock.clone());

        for _ in 0..FAILURE_THRESHOLD {
            let _ = cb.call(|| Err::<(), &str>("fail"));
        }
        assert_eq!(cb.state(), BreakerState::Open);

        // SeededClock increments 1ms per now() call.
        // Need 30_000 ticks to reach 30s elapsed.
        for _ in 0..30_000 {
            let _ = clock.now();
        }

        let result = cb.call(|| Ok::<i32, &str>(42));
        assert!(result.is_ok());
        assert_eq!(cb.state(), BreakerState::Closed);
    }

    #[test]
    fn circuit_breaker_success_resets_to_closed() {
        let clock = make_clock();
        let mut cb = CircuitBreaker::new(clock.clone());

        // Fail twice (below threshold)
        let _ = cb.call(|| Err::<(), &str>("fail"));
        let _ = cb.call(|| Err::<(), &str>("fail"));
        assert_eq!(cb.state(), BreakerState::Closed);
        assert_eq!(cb.failure_count, 2);

        let result = cb.call(|| Ok::<i32, &str>(1));
        assert!(result.is_ok());
        assert_eq!(cb.failure_count, 0);
    }

    #[test]
    fn circuit_breaker_window_reset_after_60s() {
        let clock = make_clock();
        let mut cb = CircuitBreaker::new(clock.clone());

        // Fail twice
        let _ = cb.call(|| Err::<(), &str>("fail"));
        let _ = cb.call(|| Err::<(), &str>("fail"));
        assert_eq!(cb.failure_count, 2);

        // Advance past 60s window
        for _ in 0..61_000 {
            let _ = clock.now();
        }

        // Next failure should reset window, count = 1
        let _ = cb.call(|| Err::<(), &str>("fail"));
        assert_eq!(cb.failure_count, 1);
        assert_eq!(cb.state(), BreakerState::Closed);
    }

    // --- Step 3 tests ---

    #[test]
    fn test_circuit_breaker_opens_after_3_failures() {
        let clock = make_clock();
        let mut cb = CircuitBreaker::new(clock.clone());

        assert_eq!(cb.state(), BreakerState::Closed);

        // Exactly 3 failures within the 60s window
        for i in 0..3u32 {
            let _ = cb.call(|| Err::<(), String>(format!("fail-{i}")));
        }
        assert_eq!(cb.state(), BreakerState::Open);
        assert_eq!(cb.failure_count, 3);
    }

    #[test]
    fn test_circuit_breaker_half_open_probe() {
        let clock = make_clock();
        let mut cb = CircuitBreaker::new(clock.clone());

        // Trip the breaker with 3 failures
        for _ in 0..3 {
            let _ = cb.call(|| Err::<(), &str>("fail"));
        }
        assert_eq!(cb.state(), BreakerState::Open);

        // Advance 30s (30_000 ticks on SeededClock)
        for _ in 0..30_000 {
            let _ = clock.now();
        }

        // sync_state() transitions Open -> HalfOpen without calling a closure
        cb.sync_state();
        assert_eq!(cb.state(), BreakerState::HalfOpen);
    }
}
