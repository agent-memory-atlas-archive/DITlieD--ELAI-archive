//! E-39 -- velocity caps (rolling-window req/$ counters).
//!
//! Defends against the 2.3M-call weekend retry storm class of failure
//! (leanopstech.com/blog/agentic-ai-cost-runaway-token-budget-2026)
//! and the per-prompt-volume vs RPS asymmetry called out by
//! handsonarchitects.com/blog/2025/denial-of-wallet-cost-aware-rate-
//! limiting-part-1 (a single 200K-token prompt outspends 10K 1K-token
//! prompts, so RPS-only limits do not protect budgets).
//!
//! Four windows are tracked simultaneously:
//! * [`VelocityWindow::OneMin`]   -- last 60 seconds (burst protection)
//! * [`VelocityWindow::TenMin`]   -- last 10 minutes (sustained burst)
//! * [`VelocityWindow::OneHour`]  -- last 60 minutes (retry-storm defense)
//! * [`VelocityWindow::TwentyFourHour`] -- last 24 hours (daily ceiling)
//!
//! Each window has independent thresholds for req-count and $-spent.
//! A breach in any window aborts the in-flight task and writes a
//! [`crate::ssb::CostEventKind::VelocityCapBreached`] SSB row carrying
//! `window`, `count`, `limit`, and `kind` (req or usd).

use std::collections::VecDeque;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};

/// 4-arm rolling-window discriminator. Lives in a small enum so the
/// breach payload carries a stable snake_case label rather than a
/// raw duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VelocityWindow {
    OneMin,
    TenMin,
    OneHour,
    TwentyFourHour,
}

impl VelocityWindow {
    /// Snake-case wire label used in SSB details_json + dashboard
    /// rendering.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneMin => "one_min",
            Self::TenMin => "ten_min",
            Self::OneHour => "one_hour",
            Self::TwentyFourHour => "twenty_four_hour",
        }
    }

    /// Window duration in milliseconds. Used internally by the
    /// rolling-window pruning helper.
    #[must_use]
    pub fn duration_ms(&self) -> u64 {
        match self {
            Self::OneMin => 60_000,
            Self::TenMin => 600_000,
            Self::OneHour => 3_600_000,
            Self::TwentyFourHour => 86_400_000,
        }
    }
}

/// Caller-supplied configuration for the 4 windows. Each field carries
/// `Option<u64>` so an unconfigured window stays effectively-unbounded
/// (the C-5 hard ceiling still applies via [`crate::cost::cloud_abort`]).
#[derive(Debug, Clone, Default)]
pub struct VelocityCapsConfig {
    pub one_min_req_limit: Option<u64>,
    pub one_min_usd_cents_limit: Option<u64>,
    pub ten_min_req_limit: Option<u64>,
    pub ten_min_usd_cents_limit: Option<u64>,
    pub one_hour_req_limit: Option<u64>,
    pub one_hour_usd_cents_limit: Option<u64>,
    pub twenty_four_hour_req_limit: Option<u64>,
    pub twenty_four_hour_usd_cents_limit: Option<u64>,
}

impl VelocityCapsConfig {
    /// Default-safe ceiling pack derived from the report's "retry
    /// storm 2.3M calls / weekend" datapoint. Tuned conservative for
    /// solo-dev hardware: 30 req/min, $1/min, 200 req/10min, $5/10min,
    /// 600 req/hour, $25/hour, 5000 req/day, $100/day. Users override
    /// per project in `[cloud.velocity_caps]` of `elai_cc.toml`.
    #[must_use]
    pub fn default_conservative() -> Self {
        Self {
            one_min_req_limit: Some(30),
            one_min_usd_cents_limit: Some(100),
            ten_min_req_limit: Some(200),
            ten_min_usd_cents_limit: Some(500),
            one_hour_req_limit: Some(600),
            one_hour_usd_cents_limit: Some(2_500),
            twenty_four_hour_req_limit: Some(5_000),
            twenty_four_hour_usd_cents_limit: Some(10_000),
        }
    }
}

/// What kind of metric breached the cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VelocityMetricKind {
    Requests,
    UsdCents,
}

impl VelocityMetricKind {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requests => "requests",
            Self::UsdCents => "usd_cents",
        }
    }
}

/// Breach record returned from [`VelocityCapEnforcer::enforce`] when a
/// rolling window has exceeded its limit. The orchestrator forwards
/// this into the [`crate::ssb::CostEventKind::VelocityCapBreached`]
/// emit path + aborts the in-flight task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VelocityBreach {
    pub window: VelocityWindow,
    pub kind: VelocityMetricKind,
    pub count: u64,
    pub limit: u64,
}

#[derive(Debug, Clone, Copy)]
struct Sample {
    ts_ms: u64,
    usd_cents: u64,
}

/// Rolling-window counter for one (window, metric_kind) pair. Keeps a
/// FIFO of samples and prunes anything older than `window.duration_ms`
/// on every `record` / `current_*` call.
#[derive(Debug, Default)]
struct WindowState {
    samples: VecDeque<Sample>,
}

impl WindowState {
    fn prune(&mut self, now_ms: u64, horizon_ms: u64) {
        while let Some(front) = self.samples.front() {
            if now_ms.saturating_sub(front.ts_ms) > horizon_ms {
                self.samples.pop_front();
            } else {
                break;
            }
        }
    }

    fn count_requests(&self) -> u64 {
        self.samples.len() as u64
    }

    fn sum_usd_cents(&self) -> u64 {
        self.samples.iter().map(|s| s.usd_cents).sum()
    }

    fn record(&mut self, ts_ms: u64, usd_cents: u64) {
        self.samples.push_back(Sample { ts_ms, usd_cents });
    }
}

/// 4-window enforcer. Holds one `WindowState` per [`VelocityWindow`]
/// and a clock for `now_ms`. Single-task / single-thread by design;
/// the orchestrator wraps it in a per-task `Mutex` at the call site
/// rather than synchronizing here (keeps the hot path lock-free
/// otherwise and the audit row carries a clear thread story).
pub struct VelocityCapEnforcer {
    config: VelocityCapsConfig,
    clock: Arc<dyn ClockProvider>,
    one_min: WindowState,
    ten_min: WindowState,
    one_hour: WindowState,
    twenty_four_hour: WindowState,
}

impl VelocityCapEnforcer {
    /// Construct with the supplied config + clock. The clock injection
    /// follows C-15's `ClockProvider` precedent (CLAUDE.md HARD RULE:
    /// no direct `SystemTime::now()`). `OsClock` is acceptable in
    /// production; tests pass a deterministic stub.
    #[must_use]
    pub fn new(config: VelocityCapsConfig, clock: Arc<dyn ClockProvider>) -> Self {
        Self {
            config,
            clock,
            one_min: WindowState::default(),
            ten_min: WindowState::default(),
            one_hour: WindowState::default(),
            twenty_four_hour: WindowState::default(),
        }
    }

    /// Construct with the [`VelocityCapsConfig::default_conservative`]
    /// ceiling pack and an [`OsClock`] for live use.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(
            VelocityCapsConfig::default_conservative(),
            Arc::new(OsClock),
        )
    }

    fn now_ms(&self) -> u64 {
        // ClockProvider::now_ms returns u128 (since UNIX_EPOCH).
        // Truncate to u64 with saturation rather than panic (CLAUDE.md
        // HARD RULE: no .unwrap() in production paths).
        u64::try_from(self.clock.now_ms()).unwrap_or(u64::MAX)
    }

    fn window_mut(&mut self, w: VelocityWindow) -> &mut WindowState {
        match w {
            VelocityWindow::OneMin => &mut self.one_min,
            VelocityWindow::TenMin => &mut self.ten_min,
            VelocityWindow::OneHour => &mut self.one_hour,
            VelocityWindow::TwentyFourHour => &mut self.twenty_four_hour,
        }
    }

    fn req_limit(&self, w: VelocityWindow) -> Option<u64> {
        match w {
            VelocityWindow::OneMin => self.config.one_min_req_limit,
            VelocityWindow::TenMin => self.config.ten_min_req_limit,
            VelocityWindow::OneHour => self.config.one_hour_req_limit,
            VelocityWindow::TwentyFourHour => self.config.twenty_four_hour_req_limit,
        }
    }

    fn usd_limit(&self, w: VelocityWindow) -> Option<u64> {
        match w {
            VelocityWindow::OneMin => self.config.one_min_usd_cents_limit,
            VelocityWindow::TenMin => self.config.ten_min_usd_cents_limit,
            VelocityWindow::OneHour => self.config.one_hour_usd_cents_limit,
            VelocityWindow::TwentyFourHour => self.config.twenty_four_hour_usd_cents_limit,
        }
    }

    /// Record a cloud-call sample (1 request + `usd_cents` spent) at
    /// the current clock time. Call AFTER the cloud call completes so
    /// the spend is banked (cooperative-abort design: drains the
    /// in-flight call's already-paid token cost before deciding to
    /// abort).
    pub fn record(&mut self, usd_cents: u64) {
        let ts = self.now_ms();
        self.one_min.record(ts, usd_cents);
        self.ten_min.record(ts, usd_cents);
        self.one_hour.record(ts, usd_cents);
        self.twenty_four_hour.record(ts, usd_cents);
    }

    /// Returns `Ok(())` if no window has breached its limit; returns
    /// `Err(VelocityBreach)` describing the first breach found if any.
    /// Probe order: OneMin -> TenMin -> OneHour -> TwentyFourHour for
    /// each metric (requests first, then usd_cents).
    pub fn enforce(&mut self) -> Result<(), VelocityBreach> {
        let now = self.now_ms();
        for window in [
            VelocityWindow::OneMin,
            VelocityWindow::TenMin,
            VelocityWindow::OneHour,
            VelocityWindow::TwentyFourHour,
        ] {
            let horizon = window.duration_ms();
            let state = self.window_mut(window);
            state.prune(now, horizon);
            let req_count = state.count_requests();
            let usd_count = state.sum_usd_cents();
            if let Some(limit) = self.req_limit(window) {
                if req_count > limit {
                    return Err(VelocityBreach {
                        window,
                        kind: VelocityMetricKind::Requests,
                        count: req_count,
                        limit,
                    });
                }
            }
            if let Some(limit) = self.usd_limit(window) {
                if usd_count > limit {
                    return Err(VelocityBreach {
                        window,
                        kind: VelocityMetricKind::UsdCents,
                        count: usd_count,
                        limit,
                    });
                }
            }
        }
        Ok(())
    }

    /// Snapshot the current request count for each window. Used by the
    /// dashboard's `get_velocity_cap_status` Tauri command.
    #[must_use]
    pub fn snapshot_requests(&mut self) -> [(VelocityWindow, u64); 4] {
        let now = self.now_ms();
        [
            VelocityWindow::OneMin,
            VelocityWindow::TenMin,
            VelocityWindow::OneHour,
            VelocityWindow::TwentyFourHour,
        ]
        .map(|w| {
            let horizon = w.duration_ms();
            let s = self.window_mut(w);
            s.prune(now, horizon);
            (w, s.count_requests())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Deterministic clock for testing the rolling-window pruner. The
    /// orchestrator's `ClockProvider` trait requires `system_now`,
    /// `now`, `elapsed_since`, and `instant_now`; we provide minimal
    /// satisfying impls that drive `now_ms()` from a stored ms counter
    /// while leaving the other surfaces deterministic-enough for tests.
    struct StubClock {
        ms: AtomicU64,
    }

    impl StubClock {
        fn new(ms: u64) -> Self {
            Self {
                ms: AtomicU64::new(ms),
            }
        }
        fn advance_ms(&self, ms: u64) {
            self.ms.fetch_add(ms, Ordering::SeqCst);
        }
    }

    impl ClockProvider for StubClock {
        fn now(&self) -> elai_core::clock::LogicalInstant {
            elai_core::clock::LogicalInstant::from_tick(self.ms.load(Ordering::SeqCst))
        }
        fn elapsed_since(&self, _mark: elai_core::clock::LogicalInstant) -> std::time::Duration {
            std::time::Duration::from_millis(0)
        }
        fn system_now(&self) -> std::time::SystemTime {
            std::time::UNIX_EPOCH + std::time::Duration::from_millis(self.ms.load(Ordering::SeqCst))
        }
        fn instant_now(&self) -> std::time::Instant {
            std::time::Instant::now()
        }
    }

    #[test]
    fn window_labels_stable() {
        assert_eq!(VelocityWindow::OneMin.as_str(), "one_min");
        assert_eq!(VelocityWindow::TenMin.as_str(), "ten_min");
        assert_eq!(VelocityWindow::OneHour.as_str(), "one_hour");
        assert_eq!(VelocityWindow::TwentyFourHour.as_str(), "twenty_four_hour");
    }

    #[test]
    fn unbounded_config_never_breaches() {
        let cfg = VelocityCapsConfig::default();
        let clock: Arc<dyn ClockProvider> = Arc::new(StubClock::new(0));
        let mut enf = VelocityCapEnforcer::new(cfg, clock);
        for _ in 0..1000 {
            enf.record(10_000);
        }
        assert!(enf.enforce().is_ok());
    }

    #[test]
    fn one_min_req_breach_detected() {
        let mut cfg = VelocityCapsConfig::default();
        cfg.one_min_req_limit = Some(3);
        let clock: Arc<dyn ClockProvider> = Arc::new(StubClock::new(0));
        let mut enf = VelocityCapEnforcer::new(cfg, clock);
        for _ in 0..5 {
            enf.record(1);
        }
        let breach = enf.enforce().expect_err("must breach");
        assert_eq!(breach.window, VelocityWindow::OneMin);
        assert_eq!(breach.kind, VelocityMetricKind::Requests);
        assert_eq!(breach.count, 5);
        assert_eq!(breach.limit, 3);
    }

    #[test]
    fn one_hour_usd_breach_detected() {
        let mut cfg = VelocityCapsConfig::default();
        cfg.one_hour_usd_cents_limit = Some(500);
        let clock: Arc<dyn ClockProvider> = Arc::new(StubClock::new(0));
        let mut enf = VelocityCapEnforcer::new(cfg, clock);
        enf.record(600);
        let breach = enf.enforce().expect_err("must breach");
        assert_eq!(breach.window, VelocityWindow::OneHour);
        assert_eq!(breach.kind, VelocityMetricKind::UsdCents);
        assert_eq!(breach.count, 600);
        assert_eq!(breach.limit, 500);
    }

    #[test]
    fn pruning_releases_old_samples() {
        let mut cfg = VelocityCapsConfig::default();
        cfg.one_min_req_limit = Some(3);
        let stub = Arc::new(StubClock::new(0));
        let clock: Arc<dyn ClockProvider> = stub.clone();
        let mut enf = VelocityCapEnforcer::new(cfg, clock);
        for _ in 0..2 {
            enf.record(1);
        }
        // Advance well past the 1-min horizon.
        stub.advance_ms(120_000);
        for _ in 0..2 {
            enf.record(1);
        }
        // Only 2 samples should be inside the 1-min window now.
        assert!(enf.enforce().is_ok());
    }
}
