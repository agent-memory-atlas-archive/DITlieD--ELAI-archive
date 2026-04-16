use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use elai_core::clock::{ClockProvider, LogicalInstant};
use tracing::debug;

/// Type alias for injectable sleep function (deterministic testing support).
pub type SleepFn = Arc<dyn Fn(Duration) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// Default sleep using tokio::time::sleep.
pub fn tokio_sleep() -> SleepFn {
    Arc::new(|d| Box::pin(tokio::time::sleep(d)))
}

/// No-op sleep for deterministic tests.
pub fn noop_sleep() -> SleepFn {
    Arc::new(|_| Box::pin(async {}))
}

/// Rate limiter using ClockProvider for time measurement and injectable
/// sleep function for deterministic testing.
pub struct RateLimiter {
    clock: Arc<dyn ClockProvider>,
    interval: Duration,
    last_request: Option<LogicalInstant>,
    sleep_fn: SleepFn,
}

impl RateLimiter {
    /// Create with default tokio sleep.
    pub fn new(clock: Arc<dyn ClockProvider>, interval: Duration) -> Self {
        Self {
            clock,
            interval,
            last_request: None,
            sleep_fn: tokio_sleep(),
        }
    }

    /// Create with custom sleep function (for deterministic testing).
    pub fn with_sleep(clock: Arc<dyn ClockProvider>, interval: Duration, sleep_fn: SleepFn) -> Self {
        Self {
            clock,
            interval,
            last_request: None,
            sleep_fn,
        }
    }

    /// Wait if the minimum interval since the last request has not elapsed.
    pub async fn wait_if_needed(&mut self) {
        if let Some(last) = self.last_request {
            let elapsed = self.clock.elapsed_since(last);
            if elapsed < self.interval {
                let remaining = self.interval - elapsed;
                debug!(
                    remaining_ms = remaining.as_millis() as u64,
                    "rate limiter: sleeping"
                );
                (self.sleep_fn)(remaining).await;
            }
        }
        self.last_request = Some(self.clock.now());
    }
}
