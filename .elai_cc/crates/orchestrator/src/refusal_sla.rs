//! D-17 — Honest-refusal SLA monitoring.
//!
//! A sliding-window monitor on top of the
//! [`crate::conformal_calibration::RefusalDecision`] stream D-16 emits.
//! The monitor counts the fraction of `Abstain` decisions inside a
//! bounded recent-decisions window, classifies that fraction against an
//! operator-configured healthy band, and emits two SSB beacons per
//! snapshot:
//!
//!   * `refusal_rate_snapshot` — every snapshot, carrying
//!     [`crate::ssb::RefusalRateSnapshotPayload`].
//!   * `refusal_sla_breach`    — only when `health_band != Healthy`,
//!     carrying [`crate::ssb::RefusalSlaBreachPayload`].
//!
//! Why this matters: without an SLA, the formal Mode E refusal path
//! D-16 ships can be gamed — a model that abstains on everything
//! "looks honest" while delivering nothing. Conversely, a model that
//! never abstains is confabulating. The monitor surfaces both failure
//! modes structurally so the orchestrator can intervene (escalate to
//! the Bubble-Up alert system, freeze promotion, etc).
//!
//! Grounding sources (per CLAUDE.md grounded-by-default rule):
//!   * arXiv:2503.01345 — UniCR (Unified Conformal Refusal): the formal
//!     Mode E refusal surface this monitor watches.
//!   * docs/specs/guide/real-problem-solving-grounded-2026-05-19.md §4.6
//!     (lines 288-296): "treat honest 'i-don't-know' rate as a
//!     first-class quality metric next to accuracy."
//!   * docs/thefinalglance/D-plan/D-17_honest_refusal_sla.md.
//!
//! Scope (orchestrator-scoped MVP per the D-17 plan):
//!   * pure types + pure `compute_snapshot` + pure `sla_breach`;
//!   * an SSB-emitting producer surface (`emit_snapshot`) that mirrors
//!     [`crate::conformal_calibration::decide_and_emit`] in shape;
//!   * a `dispatch_refusal_sla_check` consumer in `agent_pool` that
//!     drives periodic snapshot emission;
//!   * a typed read-side projection of the recent SSB rows for the
//!     dashboard `RefusalRateGauge` drawer.
//!
//! Out-of-scope (deferred to a later D-17 follow-up):
//!   * Bubble-Up alert dispatch (the breach beacon is the wire signal
//!     the alert dispatcher will subscribe to);
//!   * per-session vs. global window — this MVP keeps a single global
//!     `VecDeque` per `RefusalRateMonitor` instance, the caller scopes
//!     by spawning multiple monitors when needed;
//!   * persisted breach idempotency — the periodic dispatcher caps
//!     emit cadence; idempotency lives at the consumer.
//!
//! No new external dep. `VecDeque` is stdlib, `blake3` / `tracing` /
//! `thiserror` / native async-fn-in-trait via `Pin<Box<dyn Future>>`
//! all per the CLAUDE.md anti-stale-training rule.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::warn;

use std::sync::Arc;

use crate::conformal_calibration::RefusalDecision;
use crate::ssb::{RefusalRateSnapshotPayload, RefusalSlaBreachPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default sliding-window size. Chosen so an early-session snapshot
/// already has enough samples to be informative without staying anchored
/// to ancient history. Caller can override via [`RefusalSlaConfig`].
pub const DEFAULT_WINDOW_SIZE: usize = 100;

/// Default target refusal rate. The system is expected to abstain on
/// ~10% of claims when honestly calibrated (per arXiv:2503.01345 §6
/// `epsilon = 0.1` operating point).
pub const DEFAULT_TARGET_RATE: f64 = 0.10;

/// Default lower bound of the healthy band. Rates below this floor
/// indicate the policy is confabulating — abstaining too rarely.
/// Per docs/specs/guide/real-problem-solving-grounded-2026-05-19.md
/// §4.6 (5-50% empirical band).
pub const DEFAULT_HEALTHY_BAND_LO: f64 = 0.05;

/// Default upper bound of the healthy band. Rates above this ceiling
/// indicate the system is over-abstaining — checkers misconfigured OR
/// task suite outside calibration range.
pub const DEFAULT_HEALTHY_BAND_HI: f64 = 0.20;

/// Default consecutive-breach count required before the dispatcher
/// considers the breach "alarmable". Single transient breaches are
/// noisy; sustained breaches need attention.
pub const DEFAULT_ALARM_THRESHOLD_BREACHES: u32 = 3;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Errors raised by D-17 producer surfaces.
#[derive(Debug, Error)]
pub enum RefusalSlaError {
    /// The [`RefusalSlaConfig`] healthy-band bounds are invalid: the
    /// low bound must be finite, in `[0.0, 1.0]`, and strictly less
    /// than the high bound, which must also be finite and in
    /// `[0.0, 1.0]`.
    #[error(
        "invalid healthy band: lo={lo}, hi={hi}; require 0.0 <= lo < hi <= 1.0 and both finite"
    )]
    InvalidBand { lo: f64, hi: f64 },
    /// Window size must be at least 1 — a zero-size window can never
    /// hold an observation.
    #[error("window_size must be >= 1; got {got}")]
    InvalidWindowSize { got: usize },
}

/// Closed-enum classification of a [`RefusalRateSnapshot::refusal_rate`]
/// against the operator-configured healthy band.
///
/// The `Below`/`Above` arms carry the absolute distance from the
/// nearest healthy-band boundary so dashboards can render the breach
/// magnitude without re-computing it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum HealthBand {
    /// The refusal rate is below `healthy_band_lo`. `gap = lo - rate`.
    Below { gap: f64 },
    /// The refusal rate is inside `[healthy_band_lo, healthy_band_hi]`.
    Healthy,
    /// The refusal rate is above `healthy_band_hi`. `gap = rate - hi`.
    Above { gap: f64 },
}

impl HealthBand {
    /// Stable snake-case wire label matching the serde
    /// `rename_all = "snake_case"` discriminant. Used by the SSB
    /// payload + the breach-kind selector.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Below { .. } => "below",
            Self::Healthy => "healthy",
            Self::Above { .. } => "above",
        }
    }

    /// `true` when the band is `Below` or `Above` — the dispatcher
    /// emits a `RefusalSlaBreach` row exactly for these cases.
    #[must_use]
    pub fn is_breach(&self) -> bool {
        !matches!(self, Self::Healthy)
    }
}

/// Operator-configured monitoring policy. Loaded from
/// `elai_cc.toml` `[refusal_sla]` (commented by default — the monitor
/// falls back to the `DEFAULT_*` constants when the section is absent).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RefusalSlaConfig {
    /// Sliding window size in number of recent decisions.
    pub window_size: usize,
    /// Expected refusal rate when honestly calibrated.
    pub target_rate: f64,
    /// Lower bound of the healthy band (inclusive).
    pub healthy_band_lo: f64,
    /// Upper bound of the healthy band (inclusive).
    pub healthy_band_hi: f64,
    /// Consecutive breaches needed before the dispatcher considers the
    /// breach alarmable (caller-side guard).
    pub alarm_threshold_breaches: u32,
}

impl Default for RefusalSlaConfig {
    fn default() -> Self {
        Self {
            window_size: DEFAULT_WINDOW_SIZE,
            target_rate: DEFAULT_TARGET_RATE,
            healthy_band_lo: DEFAULT_HEALTHY_BAND_LO,
            healthy_band_hi: DEFAULT_HEALTHY_BAND_HI,
            alarm_threshold_breaches: DEFAULT_ALARM_THRESHOLD_BREACHES,
        }
    }
}

impl RefusalSlaConfig {
    /// Validate the config. Returns `Err` when the band is malformed or
    /// the window size is zero. Pure check, no I/O.
    pub fn validate(&self) -> Result<(), RefusalSlaError> {
        if self.window_size == 0 {
            return Err(RefusalSlaError::InvalidWindowSize { got: 0 });
        }
        let lo = self.healthy_band_lo;
        let hi = self.healthy_band_hi;
        if !lo.is_finite()
            || !hi.is_finite()
            || lo < 0.0
            || hi > 1.0
            || lo >= hi
        {
            return Err(RefusalSlaError::InvalidBand { lo, hi });
        }
        Ok(())
    }
}

/// Typed read-side projection of a single [`HealthBand`] classification.
/// Carries the same fields as
/// [`crate::ssb::RefusalRateSnapshotPayload`] plus the typed
/// [`HealthBand`] enum so consumers do not have to re-parse the
/// snake-case label.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefusalRateSnapshot {
    /// Wall-clock UNIX ms at snapshot-emit time.
    pub ts_ms: i64,
    /// Number of decisions inside the sliding window.
    pub window_total: u32,
    /// Number of `Abstain` decisions inside the window.
    pub abstain_count: u32,
    /// `abstain_count / window_total` clamped to `[0.0, 1.0]`; `0.0`
    /// when `window_total == 0` (the no-data / fail-safe band).
    pub refusal_rate: f64,
    /// Typed classification against the operator-configured band.
    pub health_band: HealthBand,
}

/// Sliding-window monitor over the [`RefusalDecision`] stream.
///
/// Construct via [`RefusalRateMonitor::new`] (validates the config) or
/// [`RefusalRateMonitor::with_defaults`] (infallible — uses the
/// `DEFAULT_*` constants which are guaranteed valid by construction).
/// Call [`observe`](Self::observe) on every decision and
/// [`compute_snapshot`](Self::compute_snapshot) at the dispatcher's
/// chosen cadence.
///
/// The monitor is `Send + Sync` (no interior mutability — the caller
/// chooses the synchronization strategy, see `agent_pool::dispatch_refusal_sla_check`).
#[derive(Debug, Clone)]
pub struct RefusalRateMonitor {
    config: RefusalSlaConfig,
    /// Bounded FIFO of recent decisions; size capped at
    /// `config.window_size`.
    recent_decisions: VecDeque<RefusalDecision>,
}

impl RefusalRateMonitor {
    /// Construct a new monitor. Validates the config; returns
    /// [`RefusalSlaError::InvalidBand`] or
    /// [`RefusalSlaError::InvalidWindowSize`] on malformed input.
    pub fn new(config: RefusalSlaConfig) -> Result<Self, RefusalSlaError> {
        config.validate()?;
        let window_size = config.window_size;
        Ok(Self {
            config,
            recent_decisions: VecDeque::with_capacity(window_size),
        })
    }

    /// Infallible constructor using [`RefusalSlaConfig::default`].
    /// Guaranteed to succeed because the `DEFAULT_*` constants form a
    /// valid configuration by construction.
    #[must_use]
    pub fn with_defaults() -> Self {
        // `RefusalSlaConfig::default()` is hand-curated to satisfy
        // `validate()`; the `expect` is therefore unreachable in
        // practice. Per CLAUDE.md the panic-on-bug rule applies: a
        // change to the defaults that broke `validate` would surface
        // immediately on the first `with_defaults` call site.
        Self::new(RefusalSlaConfig::default())
            .expect("RefusalSlaConfig::default is valid by construction")
    }

    /// Borrow the active configuration.
    #[must_use]
    pub fn config(&self) -> &RefusalSlaConfig {
        &self.config
    }

    /// Number of decisions currently in the sliding window.
    #[must_use]
    pub fn window_len(&self) -> usize {
        self.recent_decisions.len()
    }

    /// Add one decision to the sliding window. Evicts the oldest
    /// decision when the window is at capacity.
    pub fn observe(&mut self, decision: RefusalDecision) {
        if self.recent_decisions.len() == self.config.window_size {
            self.recent_decisions.pop_front();
        }
        self.recent_decisions.push_back(decision);
    }

    /// Compute a snapshot of the current window. Pure function — no
    /// I/O, no clock read (the caller supplies `now_ms`).
    #[must_use]
    pub fn compute_snapshot(&self, now_ms: i64) -> RefusalRateSnapshot {
        let window_total_usize = self.recent_decisions.len();
        let abstain_count_usize = self
            .recent_decisions
            .iter()
            .filter(|d| matches!(d, RefusalDecision::Abstain))
            .count();
        // Clamp + finite-guard the rate: `Abstain` count is bounded by
        // total which is bounded by window_size (a `usize` we control),
        // so the division is safe except for the `total == 0` case.
        let refusal_rate = if window_total_usize == 0 {
            0.0
        } else {
            let raw = (abstain_count_usize as f64) / (window_total_usize as f64);
            if raw.is_finite() {
                raw.clamp(0.0, 1.0)
            } else {
                0.0
            }
        };
        let health_band = classify_health_band(
            refusal_rate,
            self.config.healthy_band_lo,
            self.config.healthy_band_hi,
        );
        RefusalRateSnapshot {
            ts_ms: now_ms,
            window_total: u32::try_from(window_total_usize).unwrap_or(u32::MAX),
            abstain_count: u32::try_from(abstain_count_usize).unwrap_or(u32::MAX),
            refusal_rate,
            health_band,
        }
    }

    /// Compute a snapshot, emit one `refusal_rate_snapshot` SSB row
    /// always, and emit one `refusal_sla_breach` SSB row when
    /// `snapshot.health_band` is `Below` or `Above`.
    ///
    /// `now_ms` is injected (no clock read inside the producer) per the
    /// entropy-quarantine rule — mirrors the D-16
    /// `compute_tau_and_emit` shape.
    ///
    /// Failed SSB emissions are logged via `tracing::warn` but do NOT
    /// override the snapshot result — the snapshot is the load-bearing
    /// piece; the rows are the durable audit signal.
    pub async fn emit_snapshot<
        SE: RefusalRateSnapshotEmitter + ?Sized,
        BE: RefusalSlaBreachEmitter + ?Sized,
    >(
        &self,
        snapshot_emitter: &SE,
        breach_emitter: &BE,
        now_ms: i64,
    ) -> RefusalRateSnapshot {
        let snapshot = self.compute_snapshot(now_ms);
        let snapshot_payload = RefusalRateSnapshotPayload {
            refusal_rate: snapshot.refusal_rate,
            window_total: snapshot.window_total,
            abstain_count: snapshot.abstain_count,
            health_band: snapshot.health_band.as_str().to_string(),
            ts_ms: snapshot.ts_ms,
        };
        if let Err(e) = snapshot_emitter.emit(snapshot_payload).await {
            warn!(
                refusal_rate = snapshot.refusal_rate,
                window_total = snapshot.window_total,
                error = %e,
                "RefusalRateMonitor::emit_snapshot: snapshot SSB emission failed \
                 (snapshot still applies)"
            );
        }
        if snapshot.health_band.is_breach() {
            let (threshold, breach_kind) = match snapshot.health_band {
                HealthBand::Below { .. } => (self.config.healthy_band_lo, "below"),
                HealthBand::Above { .. } => (self.config.healthy_band_hi, "above"),
                HealthBand::Healthy => unreachable!("is_breach guards against Healthy"),
            };
            let breach_payload = RefusalSlaBreachPayload {
                rate: snapshot.refusal_rate,
                threshold,
                breach_kind: breach_kind.to_string(),
                ts_ms: snapshot.ts_ms,
            };
            if let Err(e) = breach_emitter.emit(breach_payload).await {
                warn!(
                    refusal_rate = snapshot.refusal_rate,
                    breach_kind = breach_kind,
                    error = %e,
                    "RefusalRateMonitor::emit_snapshot: breach SSB emission failed \
                     (breach still applies)"
                );
            }
        }
        snapshot
    }
}

/// Pure classifier — given a refusal rate + healthy-band bounds, return
/// the corresponding [`HealthBand`]. Used by
/// [`RefusalRateMonitor::compute_snapshot`] and exposed publicly so
/// callers can pre-classify without holding a monitor.
///
/// Robustness:
///   * NaN rate is fail-closed: classified as `Below { gap = lo }` so
///     the orchestrator never silently treats malformed input as
///     "Healthy" (per CLAUDE.md fail-closed reasoning rule).
///   * `lo == hi` is a degenerate band; the caller is supposed to
///     validate the config before constructing a monitor, but if this
///     classifier is called with `lo == hi`, only exact-match rates are
///     classified as Healthy.
#[must_use]
pub fn classify_health_band(rate: f64, lo: f64, hi: f64) -> HealthBand {
    if rate.is_nan() {
        return HealthBand::Below { gap: lo };
    }
    if rate < lo {
        return HealthBand::Below { gap: (lo - rate).max(0.0) };
    }
    if rate > hi {
        return HealthBand::Above { gap: (rate - hi).max(0.0) };
    }
    HealthBand::Healthy
}

/// Returns `true` when the snapshot represents an SLA breach (the
/// `health_band` is not `Healthy`). Equivalent to
/// `snapshot.health_band.is_breach()` — exported as a free function
/// because the D-17 plan calls for it as a top-level helper.
#[must_use]
pub fn sla_breach(snapshot: &RefusalRateSnapshot) -> bool {
    snapshot.health_band.is_breach()
}

// ---------------------------------------------------------------------------
// SSB producer surfaces
// ---------------------------------------------------------------------------

/// Producer-side SSB emitter for `refusal_rate_snapshot` rows. Mirrors
/// [`crate::conformal_calibration::ConformalTauComputedEmitter`] in
/// shape so production callers can share a single Ssb writer adapter
/// under all of these traits.
///
/// Native async-fn-in-trait per the anti-stale-training rule.
pub trait RefusalRateSnapshotEmitter: Send + Sync {
    fn emit(
        &self,
        payload: RefusalRateSnapshotPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>>;
}

/// Producer-side SSB emitter for `refusal_sla_breach` rows.
pub trait RefusalSlaBreachEmitter: Send + Sync {
    fn emit(
        &self,
        payload: RefusalSlaBreachPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>>;
}

/// Test-only emitter that records every `refusal_rate_snapshot`
/// payload behind a `tokio::sync::Mutex` (no `std::sync::Mutex` held
/// across `.await` per the anti-stale-training rule).
#[derive(Default)]
pub struct VecRefusalRateSnapshotEmitter {
    rows: tokio::sync::Mutex<Vec<RefusalRateSnapshotPayload>>,
}

impl VecRefusalRateSnapshotEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn rows(&self) -> Vec<RefusalRateSnapshotPayload> {
        self.rows.lock().await.clone()
    }

    pub async fn len(&self) -> usize {
        self.rows.lock().await.len()
    }
}

impl RefusalRateSnapshotEmitter for VecRefusalRateSnapshotEmitter {
    fn emit(
        &self,
        payload: RefusalRateSnapshotPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            self.rows.lock().await.push(payload);
            Ok(())
        })
    }
}

/// Test-only emitter that records every `refusal_sla_breach` payload.
#[derive(Default)]
pub struct VecRefusalSlaBreachEmitter {
    rows: tokio::sync::Mutex<Vec<RefusalSlaBreachPayload>>,
}

impl VecRefusalSlaBreachEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn rows(&self) -> Vec<RefusalSlaBreachPayload> {
        self.rows.lock().await.clone()
    }

    pub async fn len(&self) -> usize {
        self.rows.lock().await.len()
    }
}

impl RefusalSlaBreachEmitter for VecRefusalSlaBreachEmitter {
    fn emit(
        &self,
        payload: RefusalSlaBreachPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            self.rows.lock().await.push(payload);
            Ok(())
        })
    }
}

// ---------------------------------------------------------------------------
// F2 / s-22600 Tranche B -- production SSB-backed emitter adapters
//
// Bridge the `RefusalRate{Snapshot|SlaBreach}Emitter` traits onto a shared
// `Arc<Ssb>` writer so the `run_refusal_sla_tick` periodic loop can land
// `refusal_rate_snapshot` + `refusal_sla_breach` rows on the canonical
// substrate. Mirrors the shape of
// `task_runner::TaskRunnerConformalTauComputedEmitter` but owns the
// `Arc<Ssb>` (not a borrow) so the emitter can be moved into a
// `tokio::spawn`ed task without lifetime drama.
//
// Naming: the F2 decision doc resolution path (a) names these
// `SsbBackedRefusalRateSnapshotEmitter` / `SsbBackedRefusalSlaBreachEmitter`.
// They live in `refusal_sla.rs` next to the trait + Vec test doubles so the
// emitter family stays co-located.
// ---------------------------------------------------------------------------

/// `agent_id` string emitted on every SSB row produced by these adapters.
/// Mirrors `task_runner::RUNNER_AGENT_ID = "task_runner"` (the canonical
/// task-runner-originated row author) so the existing dashboard filters
/// pick the rows up without extra plumbing.
pub const REFUSAL_SLA_AGENT_ID: &str = "task_runner";

/// Production SSB-backed `RefusalRateSnapshotEmitter`. Owns an
/// `Arc<Ssb>` so the emitter is `'static` + `Send + Sync` and can be
/// freely shared across the tokio task boundary in
/// `run_refusal_sla_tick`.
///
/// Best-effort emit: `Ssb::write_payload` errors are surfaced via
/// `tracing::warn` and returned as `Err(String)` so the upstream
/// `RefusalRateMonitor::emit_snapshot` warn-and-continue contract stays
/// intact (see refusal_sla.rs::emit_snapshot lines 353-361).
pub struct SsbBackedRefusalRateSnapshotEmitter {
    writer: Arc<Ssb>,
    superstep: i64,
}

impl SsbBackedRefusalRateSnapshotEmitter {
    #[must_use]
    pub fn new(writer: Arc<Ssb>, superstep: i64) -> Self {
        Self { writer, superstep }
    }
}

impl RefusalRateSnapshotEmitter for SsbBackedRefusalRateSnapshotEmitter {
    fn emit(
        &self,
        payload: RefusalRateSnapshotPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        let writer = self.writer.clone();
        let superstep = self.superstep;
        Box::pin(async move {
            writer
                .write_payload(
                    REFUSAL_SLA_AGENT_ID,
                    superstep,
                    SsbPayload::RefusalRateSnapshot(payload),
                    Vec::new(),
                )
                .await
                .map(|_| ())
                .map_err(|e| {
                    let msg = e.to_string();
                    warn!(
                        error = %msg,
                        "SsbBackedRefusalRateSnapshotEmitter: \
                         Ssb::write_payload failed; row dropped"
                    );
                    msg
                })
        })
    }
}

/// Production SSB-backed `RefusalSlaBreachEmitter`. Same shape as the
/// snapshot emitter above; emits `SsbPayload::RefusalSlaBreach` instead.
pub struct SsbBackedRefusalSlaBreachEmitter {
    writer: Arc<Ssb>,
    superstep: i64,
}

impl SsbBackedRefusalSlaBreachEmitter {
    #[must_use]
    pub fn new(writer: Arc<Ssb>, superstep: i64) -> Self {
        Self { writer, superstep }
    }
}

impl RefusalSlaBreachEmitter for SsbBackedRefusalSlaBreachEmitter {
    fn emit(
        &self,
        payload: RefusalSlaBreachPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        let writer = self.writer.clone();
        let superstep = self.superstep;
        Box::pin(async move {
            writer
                .write_payload(
                    REFUSAL_SLA_AGENT_ID,
                    superstep,
                    SsbPayload::RefusalSlaBreach(payload),
                    Vec::new(),
                )
                .await
                .map(|_| ())
                .map_err(|e| {
                    let msg = e.to_string();
                    warn!(
                        error = %msg,
                        "SsbBackedRefusalSlaBreachEmitter: \
                         Ssb::write_payload failed; row dropped"
                    );
                    msg
                })
        })
    }
}

// ---------------------------------------------------------------------------
// Read-surface projection — typed rows the dashboard consumes
// ---------------------------------------------------------------------------

/// Typed projection of a single `refusal_rate_snapshot` SSB row.
///
/// Mirrors [`RefusalRateSnapshotPayload`] but carries the snake-case
/// `health_band` as a `String` since the SSB row is the wire surface;
/// the typed [`HealthBand`] is reconstructable via
/// [`HealthBand::as_str`] round-trip.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefusalRateSnapshotRow {
    pub refusal_rate: f64,
    pub window_total: u32,
    pub abstain_count: u32,
    pub health_band: String,
    pub ts_ms: i64,
}

/// Typed projection of a single `refusal_sla_breach` SSB row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefusalSlaBreachRow {
    pub rate: f64,
    pub threshold: f64,
    pub breach_kind: String,
    pub ts_ms: i64,
}

/// D-17 read surface — the most recent N `refusal_rate_snapshot` SSB
/// rows the dashboard `RefusalRateGauge` renders.
#[must_use]
pub fn recent_snapshot_rows(
    viewer: &crate::ssb_viewer::SsbViewer,
    limit: usize,
) -> Vec<RefusalRateSnapshotRow> {
    let mut out = Vec::new();
    match viewer.recent_rows_of_type("refusal_rate_snapshot", limit) {
        Ok(details) => {
            for d in details {
                if let Some(row) = snapshot_detail_to_row(d) {
                    out.push(row);
                }
            }
        }
        Err(e) => {
            warn!(
                action_type = "refusal_rate_snapshot",
                error = %e,
                "recent_snapshot_rows: viewer error"
            );
        }
    }
    out.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    out.truncate(limit);
    out
}

/// D-17 read surface — the most recent N `refusal_sla_breach` SSB rows.
#[must_use]
pub fn recent_breach_rows(
    viewer: &crate::ssb_viewer::SsbViewer,
    limit: usize,
) -> Vec<RefusalSlaBreachRow> {
    let mut out = Vec::new();
    match viewer.recent_rows_of_type("refusal_sla_breach", limit) {
        Ok(details) => {
            for d in details {
                if let Some(row) = breach_detail_to_row(d) {
                    out.push(row);
                }
            }
        }
        Err(e) => {
            warn!(
                action_type = "refusal_sla_breach",
                error = %e,
                "recent_breach_rows: viewer error"
            );
        }
    }
    out.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    out.truncate(limit);
    out
}

fn snapshot_detail_to_row(
    d: crate::ssb_viewer::SsbActionDetail,
) -> Option<RefusalRateSnapshotRow> {
    if d.action_type.as_str() != "refusal_rate_snapshot" {
        return None;
    }
    let outer = d.payload_json.as_object()?;
    let obj = if let Some(inner) = outer
        .get("RefusalRateSnapshot")
        .and_then(|v| v.as_object())
    {
        inner.clone()
    } else {
        outer.clone()
    };
    Some(RefusalRateSnapshotRow {
        refusal_rate: obj
            .get("refusal_rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        window_total: obj
            .get("window_total")
            .and_then(|v| v.as_u64())
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(0),
        abstain_count: obj
            .get("abstain_count")
            .and_then(|v| v.as_u64())
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(0),
        health_band: obj
            .get("health_band")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
    })
}

fn breach_detail_to_row(
    d: crate::ssb_viewer::SsbActionDetail,
) -> Option<RefusalSlaBreachRow> {
    if d.action_type.as_str() != "refusal_sla_breach" {
        return None;
    }
    let outer = d.payload_json.as_object()?;
    let obj = if let Some(inner) = outer.get("RefusalSlaBreach").and_then(|v| v.as_object()) {
        inner.clone()
    } else {
        outer.clone()
    };
    Some(RefusalSlaBreachRow {
        rate: obj.get("rate").and_then(|v| v.as_f64()).unwrap_or(0.0),
        threshold: obj
            .get("threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        breach_kind: obj
            .get("breach_kind")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
    })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn observe_n(
        monitor: &mut RefusalRateMonitor,
        abstain: usize,
        answer: usize,
    ) {
        for _ in 0..abstain {
            monitor.observe(RefusalDecision::Abstain);
        }
        for _ in 0..answer {
            monitor.observe(RefusalDecision::Answer { confidence: 0.9 });
        }
    }

    #[test]
    fn default_config_validates() {
        RefusalSlaConfig::default().validate().expect("default valid");
    }

    #[test]
    fn invalid_band_rejected() {
        let bad = RefusalSlaConfig {
            healthy_band_lo: 0.5,
            healthy_band_hi: 0.2,
            ..RefusalSlaConfig::default()
        };
        assert!(matches!(
            bad.validate(),
            Err(RefusalSlaError::InvalidBand { .. })
        ));

        let nan = RefusalSlaConfig {
            healthy_band_lo: f64::NAN,
            ..RefusalSlaConfig::default()
        };
        assert!(matches!(
            nan.validate(),
            Err(RefusalSlaError::InvalidBand { .. })
        ));
    }

    #[test]
    fn invalid_window_size_rejected() {
        let bad = RefusalSlaConfig {
            window_size: 0,
            ..RefusalSlaConfig::default()
        };
        assert!(matches!(
            bad.validate(),
            Err(RefusalSlaError::InvalidWindowSize { got: 0 })
        ));
    }

    #[test]
    fn empty_window_yields_healthy_or_below_by_floor() {
        // rate = 0.0 → below band_lo = 0.05 → Below.
        let m = RefusalRateMonitor::with_defaults();
        let snap = m.compute_snapshot(1_700_000_000_000);
        assert_eq!(snap.window_total, 0);
        assert_eq!(snap.abstain_count, 0);
        assert!((snap.refusal_rate - 0.0).abs() < 1e-12);
        assert!(matches!(snap.health_band, HealthBand::Below { .. }));
    }

    #[test]
    fn healthy_band_classification() {
        let cfg = RefusalSlaConfig::default();
        let mut m = RefusalRateMonitor::new(cfg).expect("ok");
        // 10/90 = 10% inside healthy band [5%, 20%].
        observe_n(&mut m, 10, 90);
        let snap = m.compute_snapshot(1_700_000_000_000);
        assert_eq!(snap.window_total, 100);
        assert_eq!(snap.abstain_count, 10);
        assert!((snap.refusal_rate - 0.10).abs() < 1e-9);
        assert!(matches!(snap.health_band, HealthBand::Healthy));
        assert!(!sla_breach(&snap));
    }

    #[test]
    fn above_band_classification() {
        let mut m = RefusalRateMonitor::with_defaults();
        // 50/50 = 50% — far above healthy_band_hi (0.20).
        observe_n(&mut m, 50, 50);
        let snap = m.compute_snapshot(1_700_000_000_000);
        assert_eq!(snap.window_total, 100);
        assert_eq!(snap.abstain_count, 50);
        assert!((snap.refusal_rate - 0.5).abs() < 1e-9);
        match snap.health_band {
            HealthBand::Above { gap } => {
                assert!((gap - 0.30).abs() < 1e-9, "gap = {gap}");
            }
            other => panic!("expected Above; got {other:?}"),
        }
        assert!(sla_breach(&snap));
    }

    #[test]
    fn below_band_classification() {
        let mut m = RefusalRateMonitor::with_defaults();
        // 1/99 ≈ 1% — below healthy_band_lo (5%).
        observe_n(&mut m, 1, 99);
        let snap = m.compute_snapshot(1_700_000_000_000);
        assert!(matches!(snap.health_band, HealthBand::Below { .. }));
        assert!(sla_breach(&snap));
    }

    #[test]
    fn window_eviction_caps_at_window_size() {
        let cfg = RefusalSlaConfig {
            window_size: 10,
            ..RefusalSlaConfig::default()
        };
        let mut m = RefusalRateMonitor::new(cfg).expect("ok");
        observe_n(&mut m, 0, 20); // 20 Answers
        assert_eq!(m.window_len(), 10);
        // All 10 retained should be Answer (rate = 0%).
        let snap = m.compute_snapshot(0);
        assert_eq!(snap.window_total, 10);
        assert_eq!(snap.abstain_count, 0);
    }

    #[test]
    fn classify_health_band_nan_is_fail_closed() {
        let band = classify_health_band(f64::NAN, 0.05, 0.20);
        assert!(matches!(band, HealthBand::Below { .. }));
    }

    #[tokio::test]
    async fn emit_snapshot_always_emits_snapshot_row_and_breach_only_on_breach() {
        let mut m = RefusalRateMonitor::with_defaults();
        let snap_em = VecRefusalRateSnapshotEmitter::new();
        let breach_em = VecRefusalSlaBreachEmitter::new();

        // Healthy snapshot → 1 snapshot row, 0 breach rows.
        observe_n(&mut m, 10, 90);
        let snap = m.emit_snapshot(&snap_em, &breach_em, 100).await;
        assert!(matches!(snap.health_band, HealthBand::Healthy));
        assert_eq!(snap_em.len().await, 1);
        assert_eq!(breach_em.len().await, 0);

        // Now push the rate above the band → 1 more snapshot + 1 breach.
        observe_n(&mut m, 90, 0); // window now 100/100 abstain
        let snap2 = m.emit_snapshot(&snap_em, &breach_em, 200).await;
        assert!(matches!(snap2.health_band, HealthBand::Above { .. }));
        assert_eq!(snap_em.len().await, 2);
        assert_eq!(breach_em.len().await, 1);
        let breach_rows = breach_em.rows().await;
        assert_eq!(breach_rows[0].breach_kind, "above");
        assert!((breach_rows[0].threshold - 0.20).abs() < 1e-9);
    }

    #[test]
    fn health_band_wire_labels() {
        assert_eq!(HealthBand::Below { gap: 0.1 }.as_str(), "below");
        assert_eq!(HealthBand::Healthy.as_str(), "healthy");
        assert_eq!(HealthBand::Above { gap: 0.1 }.as_str(), "above");
    }
}
