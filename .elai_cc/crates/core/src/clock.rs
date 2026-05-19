use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime};

/// Baseline `Instant` captured once, used by `SeededClock::instant_now()` so the
/// returned `Instant` is monotonically derived from tick (no `lazy_static`,
/// no new dependency — see C-15 D3).
static SEEDED_INSTANT_BASE: OnceLock<Instant> = OnceLock::new();

fn seeded_instant_base() -> Instant {
    *SEEDED_INSTANT_BASE.get_or_init(Instant::now)
}

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

    /// Return a raw `Instant`. For `OsClock` this is `Instant::now()`. For
    /// `SeededClock` it is a synthetic instant derived from a process-wide
    /// baseline plus tick milliseconds — monotonic but unrelated to wall clock.
    /// Added in C-15.1 so that callers holding only an `Arc<dyn ClockProvider>`
    /// can still produce an `Instant` for legacy APIs (e.g., `Instant`-keyed
    /// caches) without leaking `Instant::now()` calls.
    fn instant_now(&self) -> Instant;

    /// Convenience: milliseconds since UNIX epoch, derived from `system_now()`.
    /// Returns 0 if the clock is somehow before UNIX_EPOCH (impossible for
    /// `OsClock` and `SeededClock`, but defensive for custom impls).
    fn now_ms(&self) -> u128 {
        self.system_now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    }

    /// Convenience: seconds since UNIX epoch, derived from `system_now()`.
    fn now_secs(&self) -> u64 {
        self.system_now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
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

    fn instant_now(&self) -> Instant {
        Instant::now()
    }
}

// ---------------------------------------------------------------------------
// Test / deterministic implementation
// ---------------------------------------------------------------------------

/// Tick-based clock for deterministic tests. Each `now()` call increments by 1ms.
///
/// ELAI_SEED seeds the random number generator only (SeedableRng path in
/// entropy.rs). The tick counter always starts at 0 in seeded mode — the first
/// call to `now()` returns `tick=0`. Timestamps in derived/non-clock fields are
/// therefore byte-identical across runs with the same ELAI_SEED when
/// `ClockProvider` is properly injected (C-15.2).
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

impl Clone for SeededClock {
    fn clone(&self) -> Self {
        Self {
            tick: AtomicU64::new(self.tick.load(Ordering::Relaxed)),
            epoch: self.epoch,
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

    fn instant_now(&self) -> Instant {
        // Synthetic Instant: process-wide baseline + tick ms. Monotonic in tick.
        // Not related to wall clock — callers that need wall clock must use
        // `system_now()`.
        let tick = self.tick.load(Ordering::Relaxed);
        seeded_instant_base() + Duration::from_millis(tick)
    }
}
