use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

/// Opaque monotonic timestamp. Comparable but not constructable outside this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogicalInstant {
    tick: u64,
    inner: Option<Instant>,
}

impl LogicalInstant {
    fn from_instant(instant: Instant, tick: u64) -> Self {
        Self {
            tick,
            inner: Some(instant),
        }
    }

    /// Create a tick-only instant (no wall clock). Used by SeededClock
    /// and custom test clocks that don't need real Instant backing.
    pub fn from_tick(tick: u64) -> Self {
        Self { tick, inner: None }
    }

    /// Returns the raw tick value (for serialization / logging only).
    pub fn tick(&self) -> u64 {
        self.tick
    }
}

/// Abstraction over monotonic + wall-clock time.
/// All methods take `&self` for object safety (Arc<dyn ClockProvider>).
pub trait ClockProvider: Send + Sync {
    fn now(&self) -> LogicalInstant;
    fn elapsed_since(&self, mark: LogicalInstant) -> Duration;
    fn system_now(&self) -> SystemTime;
}

// ---------------------------------------------------------------------------
// Production implementation
// ---------------------------------------------------------------------------

/// Delegates to std::time::Instant and std::time::SystemTime.
#[derive(Debug, Clone, Copy)]
pub struct OsClock;

static OS_CLOCK_TICK: AtomicU64 = AtomicU64::new(0);

impl ClockProvider for OsClock {
    fn now(&self) -> LogicalInstant {
        let tick = OS_CLOCK_TICK.fetch_add(1, Ordering::Relaxed);
        LogicalInstant::from_instant(Instant::now(), tick)
    }

    fn elapsed_since(&self, mark: LogicalInstant) -> Duration {
        debug_assert!(
            mark.inner.is_some(),
            "OsClock::elapsed_since called with a SeededClock LogicalInstant"
        );
        match mark.inner {
            Some(instant) => instant.elapsed(),
            None => Duration::ZERO,
        }
    }

    fn system_now(&self) -> SystemTime {
        SystemTime::now()
    }
}

// ---------------------------------------------------------------------------
// Test / deterministic implementation
// ---------------------------------------------------------------------------

/// Tick-based clock for deterministic tests. Each `now()` call increments by 1ms.
pub struct SeededClock {
    tick: AtomicU64,
    epoch: SystemTime,
}

impl SeededClock {
    pub fn new() -> Self {
        Self {
            tick: AtomicU64::new(0),
            epoch: SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000),
        }
    }

    pub fn with_epoch(epoch: SystemTime) -> Self {
        Self {
            tick: AtomicU64::new(0),
            epoch,
        }
    }
}

impl Default for SeededClock {
    fn default() -> Self {
        Self::new()
    }
}

impl ClockProvider for SeededClock {
    fn now(&self) -> LogicalInstant {
        let tick = self.tick.fetch_add(1, Ordering::Relaxed);
        LogicalInstant::from_tick(tick)
    }

    fn elapsed_since(&self, mark: LogicalInstant) -> Duration {
        let current = self.tick.load(Ordering::Relaxed);
        let elapsed_ticks = current.saturating_sub(mark.tick);
        Duration::from_millis(elapsed_ticks)
    }

    fn system_now(&self) -> SystemTime {
        let tick = self.tick.load(Ordering::Relaxed);
        self.epoch + Duration::from_millis(tick)
    }
}
