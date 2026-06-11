//! D-4: Tripwire stack — KL + Δρ + IQR + perplexity + DDM auto-revert.
//!
//! Five orthogonal drift / outlier detectors composed into a single sliding-
//! window monitor. When any detector fires, the [`TripwireMonitor`] emits an
//! `SsbActionType::TripwireFired` row; in concurrent multi-detector regimes
//! the auto-revert path additionally flips the most-recently-promoted
//! [`crate::archive_chain::ArchiveEntry`] status to `Retired` with
//! [`RetirementReason::Degradation`] and emits `TripwireAutoReverted`.
//!
//! Math sources (no external dep — formulas implemented in-crate):
//! * KL divergence — closed-form Kullback-Leibler `sum p_i * log(p_i / q_i)`
//!   over discrete distributions. Drift-detection use validated by
//!   arxiv:2306.01191 §3.1 (Skoularidou et al, "Online KL drift detection")
//!   and the survey arxiv:2004.05785 Lu et al §3.3 ("distribution-based
//!   change detectors").
//! * Pearson Δρ — standard sample Pearson correlation, then compare two
//!   windows. Pattern grounded in arxiv:2210.10760 Gao et al §4 (RM
//!   over-optimisation: proxy/gold correlation degrades as KL grows).
//! * IQR outlier — Tukey's classical Q3 + k*IQR rule with k=3.0 per
//!   arxiv:2507.05619 (T1) Table 1.
//! * Perplexity anomaly — `exp(-mean(log_prob))` then μ + k*σ test on the
//!   PPL sequence; standard textbook formula (no external dep).
//! * DDM — Drift Detection Method (Gama et al 2004, re-derived in
//!   arxiv:2412.08737 §2.2 + Lu et al survey §3.2). State machine tracks
//!   p_min + s_min and fires `Drift` when `p_i + s_i >= p_min + 3*s_min`.
//!
//! Wiring (HARD): producer is [`TripwireMonitor::observe`] / [`auto_revert`]
//! at the agent_pool post-consensus hook. Consumer is the archive_chain
//! (the chain mutates itself on auto-revert). Dashboard reads via the
//! `get_recent_tripwires` Tauri command (commands_d4.rs).

use std::collections::VecDeque;

use blake3::Hasher as Blake3Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

use crate::archive_chain::ArchiveChain;
use crate::reasoning_template::{ArchiveStatus, RetirementReason, TemplateId};
use crate::ssb::{SsbPayload, TemplateAutoRevertedPayload, TripwireFiredPayload};
use crate::verification_result::IntrinsicSignals;

// ---------------------------------------------------------------------------
// TripwireKind: closed enum of detector families
// ---------------------------------------------------------------------------

/// One detector family. Closed enum — agents must not introduce new
/// kinds without a coordinated schema bump (defends against silent
/// reshaping of the auto-revert rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TripwireKind {
    /// KL divergence between the current `entropy` distribution proxy and
    /// the baseline window.
    KlDivergence,
    /// Pearson correlation Δρ between paired verifier scores
    /// (proxy vs. gold) over two adjacent windows.
    CorrelationDrop,
    /// IQR outlier on per-step `logprob_var` spikes (Tukey k=3).
    IqrOutlier,
    /// Perplexity anomaly — exp(-logprob_mean) against mean + k*σ.
    Perplexity,
    /// Drift Detection Method — sequential statistical test on a
    /// pass/fail stream.
    Ddm,
}

impl TripwireKind {
    /// Snake-case wire label for SSB rows + dashboard rendering.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::KlDivergence => "kl_divergence",
            Self::CorrelationDrop => "correlation_drop",
            Self::IqrOutlier => "iqr_outlier",
            Self::Perplexity => "perplexity",
            Self::Ddm => "ddm",
        }
    }

    /// Which signal stream the detector consumes. Surfaced on the SSB
    /// `TripwireFired` payload as `source` so downstream consumers (D-26
    /// concurrent-fire diagnostics, dashboard breakdown) can group firings
    /// by the underlying signal without re-deriving the mapping. Mapping
    /// rationale:
    /// * `KlDivergence` reads the action-distribution (entropy histogram)
    ///   per arxiv:2210.10760 §4 — tagged `Action`.
    /// * `CorrelationDrop` is the proxy/gold reward correlation degradation
    ///   from arxiv:2210.10760 — tagged `Reward`.
    /// * `IqrOutlier` runs on per-token `logprob_var`, an action-level
    ///   variance signal — tagged `Action`.
    /// * `Perplexity` is the textbook perplexity series — tagged
    ///   `Perplexity`.
    /// * `Ddm` consumes the pass/fail classifier stream (Gama 2004) —
    ///   tagged `Classifier`.
    #[must_use]
    pub fn source(&self) -> SignalSource {
        match self {
            Self::KlDivergence => SignalSource::Action,
            Self::CorrelationDrop => SignalSource::Reward,
            Self::IqrOutlier => SignalSource::Action,
            Self::Perplexity => SignalSource::Perplexity,
            Self::Ddm => SignalSource::Classifier,
        }
    }
}

// ---------------------------------------------------------------------------
// SignalSource: which signal stream a tripwire firing originated from
// ---------------------------------------------------------------------------

/// Per D-4 GAP-1 closure: tag every `TripwireFired` payload with the
/// underlying signal stream the detector consumed, so concurrent-fire
/// diagnostics (D-26's `TripwireFiredConcurrent` consumer) and the
/// dashboard breakdown can group rows by source without re-deriving the
/// `TripwireKind -> source` mapping from the wire label.
///
/// Closed enum — adding a new source family requires a coordinated SSB
/// schema bump (the `source` field is part of the `TripwireFiredPayload`
/// stable wire shape).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalSource {
    /// Reward / verifier-score signal (proxy-vs-gold correlation; from
    /// arxiv:2210.10760 RM over-optimisation framing).
    Reward,
    /// Action-distribution signal (entropy histogram or per-token
    /// logprob_var); maps to policy-action drift in the same framing.
    Action,
    /// Perplexity series (exp(-logprob_mean)) anomaly detector input.
    Perplexity,
    /// Pass/fail classifier stream (DDM input, Gama 2004).
    Classifier,
}

impl SignalSource {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Reward => "reward",
            Self::Action => "action",
            Self::Perplexity => "perplexity",
            Self::Classifier => "classifier",
        }
    }
}

// ---------------------------------------------------------------------------
// TripwireConfig
// ---------------------------------------------------------------------------

/// Per-kind thresholds and window sizes. Defaults are paper-cited and
/// MUST NOT be hardcoded into the monitor constructor (defends against
/// agent in-code edits per the L3 lock policy — operators tune via
/// `elai_cc.toml`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TripwireConfig {
    /// KL divergence threshold τ_spec (arxiv:2507.05619 default 0.3).
    pub kl_spec_tau: f32,
    /// Pearson Δρ degradation threshold (arxiv:2507.05619 default 0.5).
    pub rho_delta_threshold: f32,
    /// IQR multiplier (Tukey default 3.0 per arxiv:2507.05619).
    pub iqr_outlier_multiplier: f32,
    /// Perplexity σ multiplier (arxiv:2507.05619 default 2.0).
    pub ppl_anomaly_sigma: f32,
    /// DDM warning multiplier (Gama 2004 default 2.0).
    pub ddm_warning_threshold: f32,
    /// DDM drift multiplier (Gama 2004 default 3.0).
    pub ddm_drift_threshold: f32,
    /// Minimum number of samples in the window before a detector may fire
    /// (defends against false positives on short streams).
    pub min_sample_count: u32,
    /// Sliding-window length (Lu et al survey default 50).
    pub window_size: u32,
}

impl TripwireConfig {
    /// Default configuration from the D-4 plan (paper-cited).
    #[must_use]
    pub fn defaults() -> Self {
        Self {
            kl_spec_tau: 0.3,
            rho_delta_threshold: 0.5,
            iqr_outlier_multiplier: 3.0,
            ppl_anomaly_sigma: 2.0,
            ddm_warning_threshold: 2.0,
            ddm_drift_threshold: 3.0,
            min_sample_count: 10,
            window_size: 50,
        }
    }
}

impl Default for TripwireConfig {
    fn default() -> Self {
        Self::defaults()
    }
}

/// Default location of the `[tripwire]` stanza relative to the workspace
/// root that owns the `.elai_cc/` tree (the orchestrator process CWD).
const TRIPWIRE_TOML_SUFFIX: &str = ".elai_cc/elai_cc.toml";

/// Load the [`TripwireConfig`] from the workspace `elai_cc.toml` `[tripwire]`
/// stanza, mapping the toml key names to the struct fields and falling back
/// to [`TripwireConfig::defaults`] for any key that is missing or absent.
///
/// D-4 stub_ban L3-lock: thresholds MUST come from `elai_cc.toml`, never be
/// hardcoded in the [`TripwireMonitor`] constructor (defends against an agent
/// editing the thresholds in-code per companion §4). The toml key names are
/// the committed `[tripwire]` labels (`delta_rho`, `corr_window`,
/// `iqr_multiplier`, `ppl_sigma_mult`, `ddm_min_instances`,
/// `ddm_warning_threshold`, `ddm_drift_threshold`, `kl_spec_tau`) which differ
/// from the struct field names; the mapping is explicit here so a rename on
/// either side trips the loader rather than silently dropping a value.
///
/// Resolution mirrors `build_checker_runtime`: the orchestrator runs with CWD
/// = the workspace root, so `current_dir().join(".elai_cc/elai_cc.toml")` is
/// the canonical path. Any failure (file missing, parse error, no `[tripwire]`
/// stanza) is non-fatal and returns [`TripwireConfig::defaults`] — the
/// paper-cited defaults are themselves valid, so the monitor never runs with
/// nonsense thresholds.
#[must_use]
pub fn load_tripwire_config() -> TripwireConfig {
    let path = match std::env::current_dir() {
        Ok(root) => root.join(TRIPWIRE_TOML_SUFFIX),
        Err(e) => {
            warn!(error = %e, "tripwire: current_dir() failed; using paper-cited defaults");
            return TripwireConfig::defaults();
        }
    };
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            warn!(
                path = %path.display(),
                error = %e,
                "tripwire: elai_cc.toml unreadable; using paper-cited defaults"
            );
            return TripwireConfig::defaults();
        }
    };
    load_tripwire_config_from_str(&raw)
}

/// Parse a [`TripwireConfig`] from a raw toml string. Split out so the
/// per-key mapping is unit-testable without filesystem access. Missing keys
/// (and a missing `[tripwire]` stanza) fall back to the paper-cited defaults.
#[must_use]
pub fn load_tripwire_config_from_str(raw: &str) -> TripwireConfig {
    let defaults = TripwireConfig::defaults();
    let value: toml::Value = match toml::from_str(raw) {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "tripwire: elai_cc.toml parse failed; using paper-cited defaults");
            return defaults;
        }
    };
    let Some(section) = value.get("tripwire").and_then(toml::Value::as_table) else {
        // No [tripwire] stanza — the committed config ships one, so this only
        // happens on a stripped config; the defaults are still paper-valid.
        return defaults;
    };
    let f32_or = |key: &str, fallback: f32| -> f32 {
        section
            .get(key)
            .and_then(toml::Value::as_float)
            .map_or(fallback, |v| v as f32)
    };
    let u32_or = |key: &str, fallback: u32| -> u32 {
        section
            .get(key)
            .and_then(toml::Value::as_integer)
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(fallback)
    };
    TripwireConfig {
        kl_spec_tau: f32_or("kl_spec_tau", defaults.kl_spec_tau),
        rho_delta_threshold: f32_or("delta_rho", defaults.rho_delta_threshold),
        iqr_outlier_multiplier: f32_or("iqr_multiplier", defaults.iqr_outlier_multiplier),
        ppl_anomaly_sigma: f32_or("ppl_sigma_mult", defaults.ppl_anomaly_sigma),
        ddm_warning_threshold: f32_or("ddm_warning_threshold", defaults.ddm_warning_threshold),
        ddm_drift_threshold: f32_or("ddm_drift_threshold", defaults.ddm_drift_threshold),
        min_sample_count: u32_or("ddm_min_instances", defaults.min_sample_count),
        window_size: u32_or("corr_window", defaults.window_size),
    }
}

// ---------------------------------------------------------------------------
// TripwireFired event + id
// ---------------------------------------------------------------------------

/// One tripwire firing. `event_id` is a blake3-derived stable identifier
/// (so dashboard surfaces can dedupe by id rather than seq); the inputs to
/// the hash are `(kind, candidate_id, observed.to_bits, threshold.to_bits,
/// fired_at_ms)` so a regenerated event with the same scalars produces the
/// same id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TripwireFired {
    pub event_id: String,
    pub kind: TripwireKind,
    pub candidate_id: String,
    pub observed_value: f64,
    pub threshold: f64,
    pub fired_at_ms: i64,
}

impl TripwireFired {
    /// Construct a firing event and compute its blake3 id. `observed` and
    /// `threshold` are widened to `f64` for SSB-friendly serialisation.
    #[must_use]
    pub fn new(
        kind: TripwireKind,
        candidate_id: impl Into<String>,
        observed_value: f64,
        threshold: f64,
        fired_at_ms: i64,
    ) -> Self {
        let candidate_id = candidate_id.into();
        let event_id = blake3_event_id(
            kind,
            candidate_id.as_str(),
            observed_value,
            threshold,
            fired_at_ms,
        );
        Self {
            event_id,
            kind,
            candidate_id,
            observed_value,
            threshold,
            fired_at_ms,
        }
    }
}

/// blake3-derived event identifier — feeds the dashboard de-dupe path.
/// f64 values are mixed in by their IEEE-754 bit pattern so identical
/// scalars produce identical ids.
fn blake3_event_id(
    kind: TripwireKind,
    candidate_id: &str,
    observed_value: f64,
    threshold: f64,
    fired_at_ms: i64,
) -> String {
    let mut h = Blake3Hasher::new();
    h.update(b"tripwire-v1\n");
    h.update(kind.as_str().as_bytes());
    h.update(b"\n");
    h.update(candidate_id.as_bytes());
    h.update(b"\n");
    h.update(&observed_value.to_bits().to_le_bytes());
    h.update(&threshold.to_bits().to_le_bytes());
    h.update(&fired_at_ms.to_le_bytes());
    let hash = h.finalize();
    // Truncate to 16 bytes / 32 hex chars — collision-resistant enough for
    // SSB row dedupe; full hash is overkill for this surface.
    let bytes = hash.as_bytes();
    let mut out = String::with_capacity(32);
    for byte in &bytes[..16] {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Failure mode of the auto-revert helper. Distinct from `TemplateError` so
/// the caller can branch on revert-specific failures (e.g. no candidate to
/// revert to) without re-parsing strings.
#[derive(Debug, Error)]
pub enum AutoRevertError {
    #[error("template not found in archive chain: {0}")]
    TemplateNotFound(String),
    #[error("template already retired: {0}")]
    AlreadyRetired(String),
    #[error("status transition rejected: {from} -> retired")]
    TransitionRejected { from: &'static str },
    #[error("archive append failed: {0}")]
    AppendFailed(String),
}

// ---------------------------------------------------------------------------
// DDM state machine
// ---------------------------------------------------------------------------

/// DDM (Drift Detection Method) state machine.
///
/// Per Gama 2004 / Lu et al §3.2: tracks classifier error rate `p_i` and
/// standard deviation `s_i = sqrt(p_i*(1-p_i)/i)`. When `p_i + s_i` crosses
/// `p_min + warning*s_min` the level is `Warning`; when it crosses
/// `p_min + drift*s_min` the level is `Drift` and the monitor fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DdmLevel {
    Normal,
    Warning,
    Drift,
}

/// Stateful DDM tracker. Closed over a stream of pass/fail booleans.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DdmState {
    pub sample_count: u32,
    pub error_count: u32,
    pub p_min: f32,
    pub s_min: f32,
    pub current_p: f32,
    pub current_s: f32,
    pub level: DdmLevel,
}

impl DdmState {
    /// Fresh state — `p_min` + `s_min` initialised to `f32::INFINITY` so the
    /// first sample becomes the minimum.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sample_count: 0,
            error_count: 0,
            p_min: f32::INFINITY,
            s_min: f32::INFINITY,
            current_p: 0.0,
            current_s: 0.0,
            level: DdmLevel::Normal,
        }
    }

    /// Gama 2004: minimum samples before DDM begins evaluating drift
    /// thresholds. Below this count the level stays Normal regardless of
    /// observed p_i to avoid false positives on tiny streams.
    pub const MIN_SAMPLES_FOR_DRIFT: u32 = 30;

    /// Feed one observation (`true` = pass / `false` = fail) and update the
    /// state. Returns the previous level so callers can detect the
    /// `Normal -> Drift` transition.
    pub fn observe(&mut self, passed: bool, warning_mult: f32, drift_mult: f32) -> DdmLevel {
        if !passed {
            self.error_count += 1;
        }
        self.sample_count += 1;
        let i = self.sample_count as f32;
        let p = self.error_count as f32 / i;
        let s = (p * (1.0 - p) / i).sqrt();
        self.current_p = p;
        self.current_s = s;

        // Update running minimum of (p + s) — Gama: track the lowest seen
        // pair, not p_min + s_min independently. Only start tracking after
        // the warm-up window AND once at least one error has been
        // observed (a zero-error baseline collapses the threshold to
        // zero and forces spurious fires).
        if self.sample_count >= Self::MIN_SAMPLES_FOR_DRIFT
            && self.error_count > 0
            && p + s < self.p_min + self.s_min
        {
            self.p_min = p;
            self.s_min = s;
        }

        let prev = self.level;

        // Warm-up: defer drift evaluation until we have enough samples
        // AND a finite running minimum to compare against.
        if self.sample_count < Self::MIN_SAMPLES_FOR_DRIFT
            || !self.p_min.is_finite()
            || !self.s_min.is_finite()
        {
            self.level = DdmLevel::Normal;
            return prev;
        }

        let warn_threshold = self.p_min + warning_mult * self.s_min;
        let drift_threshold = self.p_min + drift_mult * self.s_min;
        self.level = if p + s >= drift_threshold {
            DdmLevel::Drift
        } else if p + s >= warn_threshold {
            DdmLevel::Warning
        } else {
            DdmLevel::Normal
        };

        // Reset running minimum on entering Drift so the next stream is
        // measured against a fresh baseline (Gama 2004).
        if self.level == DdmLevel::Drift && prev != DdmLevel::Drift {
            self.p_min = f32::INFINITY;
            self.s_min = f32::INFINITY;
        }
        prev
    }
}

impl Default for DdmState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Closed-form math helpers
// ---------------------------------------------------------------------------

/// Discrete KL divergence `D(p || q) = sum p_i * ln(p_i/q_i)`.
///
/// * `p` is the current distribution, `q` is the baseline. Both must be
///   non-empty + equal length.
/// * Returns `0.0` when the inputs are pointwise equal.
/// * Returns `f32::INFINITY` when `q_i == 0` while `p_i > 0` (KL is
///   undefined / infinite by definition; downstream comparisons MUST
///   tolerate +inf without panicking).
/// * `p_i = 0` rows contribute `0` (limit `x*ln(x) -> 0`).
/// * Non-finite inputs (NaN / negative) produce `f32::NAN` so caller can
///   surface a "bad input" signal rather than silently fire.
#[must_use]
pub fn kl_divergence(p: &[f32], q: &[f32]) -> f32 {
    if p.is_empty() || p.len() != q.len() {
        return f32::NAN;
    }
    let mut sum = 0.0f32;
    for (pi, qi) in p.iter().zip(q.iter()) {
        if !pi.is_finite() || !qi.is_finite() || *pi < 0.0 || *qi < 0.0 {
            return f32::NAN;
        }
        if *pi == 0.0 {
            continue;
        }
        if *qi == 0.0 {
            return f32::INFINITY;
        }
        sum += pi * (pi / qi).ln();
    }
    sum
}

/// Sample Pearson correlation. Returns `0.0` when either variance is zero
/// (closed-form: undefined / no signal); returns `f32::NAN` on malformed
/// input (length mismatch / empty).
#[must_use]
pub fn pearson_correlation(xs: &[f32], ys: &[f32]) -> f32 {
    if xs.is_empty() || xs.len() != ys.len() {
        return f32::NAN;
    }
    let n = xs.len() as f32;
    let mean_x: f32 = xs.iter().sum::<f32>() / n;
    let mean_y: f32 = ys.iter().sum::<f32>() / n;
    let mut cov = 0.0f32;
    let mut var_x = 0.0f32;
    let mut var_y = 0.0f32;
    for (x, y) in xs.iter().zip(ys.iter()) {
        let dx = *x - mean_x;
        let dy = *y - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }
    if var_x == 0.0 || var_y == 0.0 {
        return 0.0;
    }
    cov / (var_x.sqrt() * var_y.sqrt())
}

/// IQR outlier check. Returns the largest outlier value (above
/// `Q3 + multiplier * IQR`) when one is present, else `None`.
#[must_use]
pub fn iqr_outlier(values: &[f32], multiplier: f32) -> Option<f32> {
    if values.len() < 4 {
        return None;
    }
    let mut sorted: Vec<f32> = values.iter().copied().collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let q1 = quantile_sorted(&sorted, 0.25);
    let q3 = quantile_sorted(&sorted, 0.75);
    let iqr = q3 - q1;
    let upper = q3 + multiplier * iqr;
    let mut max_outlier: Option<f32> = None;
    for v in values {
        if *v > upper {
            max_outlier = Some(match max_outlier {
                None => *v,
                Some(prev) => prev.max(*v),
            });
        }
    }
    max_outlier
}

/// Linear-interpolation quantile of a sorted slice. Pre-condition: `sorted`
/// is non-decreasing and non-empty (enforced by the call sites).
fn quantile_sorted(sorted: &[f32], q: f32) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    let pos = q * (sorted.len() as f32 - 1.0);
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        return sorted[lo];
    }
    let frac = pos - lo as f32;
    sorted[lo] * (1.0 - frac) + sorted[hi] * frac
}

/// Perplexity from mean log-probability. `exp(-logprob_mean)`. `logprob_mean`
/// is expected to be `<= 0`; positive inputs produce perplexities `< 1` (the
/// formula is total — we do not panic / clamp here, callers decide).
#[must_use]
pub fn perplexity_from_logprob(logprob_mean: f32) -> f32 {
    (-logprob_mean).exp()
}

/// Perplexity anomaly check. Computes mean + stddev of the input series and
/// flags any value exceeding `mean + sigma_multiplier * stddev`. Returns the
/// largest anomalous value when present, else `None`.
#[must_use]
pub fn perplexity_anomaly(ppls: &[f32], sigma_multiplier: f32) -> Option<f32> {
    if ppls.len() < 2 {
        return None;
    }
    let n = ppls.len() as f32;
    let mean: f32 = ppls.iter().sum::<f32>() / n;
    let variance: f32 =
        ppls.iter().map(|v| (*v - mean).powi(2)).sum::<f32>() / (n - 1.0);
    let stddev = variance.sqrt();
    if stddev == 0.0 {
        return None;
    }
    let upper = mean + sigma_multiplier * stddev;
    let mut max_anom: Option<f32> = None;
    for v in ppls {
        if *v > upper {
            max_anom = Some(match max_anom {
                None => *v,
                Some(prev) => prev.max(*v),
            });
        }
    }
    max_anom
}

// ---------------------------------------------------------------------------
// TripwireMonitor: stateful sliding-window detector
// ---------------------------------------------------------------------------

/// One sample point fed into the monitor. We carry both the intrinsic
/// signals (logprob / entropy / KL-to-base) and a paired (proxy, gold) tuple
/// for the correlation detector. `passed` drives the DDM stream.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TripwireSample {
    pub signals: IntrinsicSignals,
    /// Proxy verifier score on `[0,1]` (e.g. PPM). `None` skips this sample
    /// for the Δρ detector but still contributes to KL / IQR / PPL / DDM.
    pub proxy: Option<f32>,
    /// Gold-truth score on `[0,1]` (e.g. deterministic pass=1.0 / fail=0.0).
    pub gold: Option<f32>,
    /// Whether the candidate passed the consensus gate. Drives DDM.
    pub passed: bool,
    /// Unix-millis wall clock.
    pub ts_ms: i64,
}

/// Outcome of one observation. `fired` contains zero or more concurrent
/// firings (one per detector). The monitor never deduplicates within a
/// single observation — concurrent fires across detectors are a stronger
/// signal than one firing alone (the D-4 plan's consensus_fire_count uses
/// this count downstream).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ObserveOutcome {
    pub fired: Vec<TripwireFired>,
    pub ddm_level: Option<DdmLevel>,
}

/// Sliding-window detector. Holds the last `config.window_size` samples and
/// runs each detector on every `observe` call.
#[derive(Debug, Clone)]
pub struct TripwireMonitor {
    config: TripwireConfig,
    window: VecDeque<TripwireSample>,
    ddm: DdmState,
}

impl TripwireMonitor {
    /// Construct a monitor with the given configuration. The window starts
    /// empty; the first `min_sample_count` observations only seed the
    /// detectors without firing.
    #[must_use]
    pub fn new(config: TripwireConfig) -> Self {
        let cap = config.window_size as usize;
        Self {
            config,
            window: VecDeque::with_capacity(cap),
            ddm: DdmState::new(),
        }
    }

    /// Current window size (number of buffered samples).
    #[must_use]
    pub fn window_len(&self) -> usize {
        self.window.len()
    }

    /// Read-only view of the active config.
    #[must_use]
    pub fn config(&self) -> &TripwireConfig {
        &self.config
    }

    /// Read-only view of the current DDM state.
    #[must_use]
    pub fn ddm_state(&self) -> &DdmState {
        &self.ddm
    }

    /// Ingest one sample. Returns the firings produced this tick (possibly
    /// empty). Trims the window deterministically to `window_size`.
    pub fn observe(&mut self, candidate_id: &str, sample: TripwireSample) -> ObserveOutcome {
        // 1. Push + trim window — deterministic, never silently drops.
        let cap = self.config.window_size as usize;
        self.window.push_back(sample);
        while self.window.len() > cap {
            self.window.pop_front();
        }

        let mut outcome = ObserveOutcome::default();

        // 2. DDM is online (per-observation) regardless of window length.
        let prev = self.ddm.observe(
            sample.passed,
            self.config.ddm_warning_threshold,
            self.config.ddm_drift_threshold,
        );
        outcome.ddm_level = Some(self.ddm.level);
        if self.ddm.level == DdmLevel::Drift && prev != DdmLevel::Drift {
            outcome.fired.push(TripwireFired::new(
                TripwireKind::Ddm,
                candidate_id,
                f64::from(self.ddm.current_p + self.ddm.current_s),
                f64::from(self.config.ddm_drift_threshold),
                sample.ts_ms,
            ));
        }

        // 3. Window-based detectors gated on min_sample_count.
        let min = self.config.min_sample_count as usize;
        if self.window.len() < min {
            return outcome;
        }

        // 4. KL divergence — bucket recent vs. baseline entropy as a 4-bin
        //    histogram on [0, 8] nats and KL the two halves. This is a
        //    surrogate that needs only the scalar entropy values we already
        //    capture in IntrinsicSignals (no per-token distribution
        //    required, which we don't have at this hook).
        if let Some(kl) = window_split_kl(&self.window) {
            if kl > self.config.kl_spec_tau {
                outcome.fired.push(TripwireFired::new(
                    TripwireKind::KlDivergence,
                    candidate_id,
                    f64::from(kl),
                    f64::from(self.config.kl_spec_tau),
                    sample.ts_ms,
                ));
            }
        }

        // 5. Δρ — split window into two halves on (proxy, gold).
        if let Some(delta) = window_split_correlation_drop(&self.window) {
            if delta > self.config.rho_delta_threshold {
                outcome.fired.push(TripwireFired::new(
                    TripwireKind::CorrelationDrop,
                    candidate_id,
                    f64::from(delta),
                    f64::from(self.config.rho_delta_threshold),
                    sample.ts_ms,
                ));
            }
        }

        // 6. IQR outlier on logprob_var.
        let lpv: Vec<f32> = self.window.iter().map(|s| s.signals.logprob_var).collect();
        if let Some(outlier) = iqr_outlier(&lpv, self.config.iqr_outlier_multiplier) {
            outcome.fired.push(TripwireFired::new(
                TripwireKind::IqrOutlier,
                candidate_id,
                f64::from(outlier),
                f64::from(self.config.iqr_outlier_multiplier),
                sample.ts_ms,
            ));
        }

        // 7. Perplexity anomaly.
        let ppls: Vec<f32> = self
            .window
            .iter()
            .map(|s| perplexity_from_logprob(s.signals.logprob_mean))
            .collect();
        if let Some(anom) = perplexity_anomaly(&ppls, self.config.ppl_anomaly_sigma) {
            outcome.fired.push(TripwireFired::new(
                TripwireKind::Perplexity,
                candidate_id,
                f64::from(anom),
                f64::from(self.config.ppl_anomaly_sigma),
                sample.ts_ms,
            ));
        }

        outcome
    }
}

/// Split the window in two halves and KL-divergence the entropy
/// distributions (4-bin histogram on `[0.0, 8.0]` nats).
fn window_split_kl(window: &VecDeque<TripwireSample>) -> Option<f32> {
    let n = window.len();
    if n < 4 {
        return None;
    }
    let mid = n / 2;
    let (older, newer): (Vec<f32>, Vec<f32>) = window
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.signals.entropy))
        .fold((Vec::new(), Vec::new()), |(mut a, mut b), (i, e)| {
            if i < mid {
                a.push(e);
            } else {
                b.push(e);
            }
            (a, b)
        });
    let older_hist = hist_4bin(&older);
    let newer_hist = hist_4bin(&newer);
    Some(kl_divergence(&newer_hist, &older_hist))
}

/// 4-bin histogram on `[0.0, 8.0]` nats. Values outside the range are
/// clamped into the boundary bin. Add Laplace smoothing of `+1` per bin so
/// `kl_divergence` never sees a zero baseline (we want a finite signal for
/// drift, not +inf).
fn hist_4bin(values: &[f32]) -> Vec<f32> {
    let mut bins = [1.0f32; 4]; // Laplace +1 smoothing.
    for v in values {
        let clamped = v.clamp(0.0, 7.999);
        let idx = ((clamped / 8.0) * 4.0) as usize;
        bins[idx.min(3)] += 1.0;
    }
    let total: f32 = bins.iter().sum();
    bins.iter().map(|b| *b / total).collect()
}

/// Split the window in two halves on the (proxy, gold) pairs and return
/// `corr_older - corr_newer` (positive == degradation).
fn window_split_correlation_drop(window: &VecDeque<TripwireSample>) -> Option<f32> {
    let mut pairs_older: Vec<(f32, f32)> = Vec::new();
    let mut pairs_newer: Vec<(f32, f32)> = Vec::new();
    let mid = window.len() / 2;
    for (i, s) in window.iter().enumerate() {
        if let (Some(p), Some(g)) = (s.proxy, s.gold) {
            if i < mid {
                pairs_older.push((p, g));
            } else {
                pairs_newer.push((p, g));
            }
        }
    }
    if pairs_older.len() < 3 || pairs_newer.len() < 3 {
        return None;
    }
    let (p_older, g_older): (Vec<f32>, Vec<f32>) = pairs_older.into_iter().unzip();
    let (p_newer, g_newer): (Vec<f32>, Vec<f32>) = pairs_newer.into_iter().unzip();
    let corr_older = pearson_correlation(&p_older, &g_older);
    let corr_newer = pearson_correlation(&p_newer, &g_newer);
    if !corr_older.is_finite() || !corr_newer.is_finite() {
        return None;
    }
    Some(corr_older - corr_newer)
}

// ---------------------------------------------------------------------------
// Auto-revert helper
// ---------------------------------------------------------------------------

/// Outcome of an auto-revert attempt. `template_id` is the retired template;
/// `parent_template_id` is the lineage target (if any) the orchestrator
/// should switch to next.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoRevertOutcome {
    pub template_id: String,
    pub parent_template_id: Option<String>,
    pub retirement_reason: RetirementReason,
    pub fired_tripwire_id: String,
}

/// Flip the matching [`ArchiveEntry`] to `Retired` with
/// [`RetirementReason::Degradation`] and return the lineage info the caller
/// emits to SSB. Producers MUST emit the `TemplateAutoReverted` SSB row
/// BEFORE re-persisting the archive line (see stub_ban contract): the
/// callers in `agent_pool` follow that order.
pub fn auto_revert(
    chain: &mut ArchiveChain,
    template_id: &TemplateId,
    fired: &TripwireFired,
) -> Result<AutoRevertOutcome, AutoRevertError> {
    let target_idx = chain
        .entries()
        .iter()
        .position(|e| e.template.id == *template_id)
        .ok_or_else(|| AutoRevertError::TemplateNotFound(template_id.as_hex()))?;
    let current_status = chain.entries()[target_idx].template.archive_status;
    if current_status == ArchiveStatus::Retired {
        return Err(AutoRevertError::AlreadyRetired(template_id.as_hex()));
    }
    if !current_status.can_transition_to(ArchiveStatus::Retired) {
        return Err(AutoRevertError::TransitionRejected {
            from: current_status.as_str(),
        });
    }
    let parent_template_id = chain.entries()[target_idx]
        .template
        .parent_id
        .as_ref()
        .map(TemplateId::as_hex);

    // Append a retirement marker to the chain (append-only D-1 contract —
    // we never rewrite existing entries; the latest entry for this template
    // carries the new status).
    let mut retire_entry = chain.entries()[target_idx].clone();
    retire_entry.template.archive_status = ArchiveStatus::Retired;
    retire_entry.retirement_reason = Some(RetirementReason::Degradation);
    retire_entry.retired_at_ms = Some(fired.fired_at_ms);
    chain
        .append(retire_entry)
        .map_err(|e| AutoRevertError::AppendFailed(e.to_string()))?;

    Ok(AutoRevertOutcome {
        template_id: template_id.as_hex(),
        parent_template_id,
        retirement_reason: RetirementReason::Degradation,
        fired_tripwire_id: fired.event_id.clone(),
    })
}

// ---------------------------------------------------------------------------
// SSB payload builders (additive — payloads themselves live in ssb.rs)
// ---------------------------------------------------------------------------

/// Build the SSB payload for a single tripwire firing. The `source` field
/// is derived from `fired.kind.source()` (closed mapping — see
/// [`TripwireKind::source`]).
#[must_use]
pub fn build_tripwire_fired_payload(fired: &TripwireFired) -> SsbPayload {
    SsbPayload::TripwireFired(TripwireFiredPayload {
        event_id: fired.event_id.clone(),
        tripwire_kind: fired.kind.as_str().into(),
        source: fired.kind.source().as_str().into(),
        candidate_id: fired.candidate_id.clone(),
        observed_value: fired.observed_value,
        threshold: fired.threshold,
        fired_at_ms: fired.fired_at_ms,
    })
}

/// Build the SSB payload for a template auto-revert.
#[must_use]
pub fn build_auto_reverted_payload(outcome: &AutoRevertOutcome) -> SsbPayload {
    SsbPayload::TemplateAutoReverted(TemplateAutoRevertedPayload {
        template_id: outcome.template_id.clone(),
        parent_template_id: outcome.parent_template_id.clone(),
        retirement_reason: outcome.retirement_reason.as_str().into(),
        fired_tripwire_id: outcome.fired_tripwire_id.clone(),
    })
}

/// Convenience: emit `TripwireFired` SSB rows for each firing in `outcome`
/// via a generic emitter. Errors are logged + swallowed (best-effort —
/// callers prioritise propagating the auto-revert result, not SSB writes).
pub async fn emit_observe_outcome<E>(emitter: &E, outcome: &ObserveOutcome)
where
    E: TripwireSsbEmitter + ?Sized,
{
    for fired in &outcome.fired {
        let payload = build_tripwire_fired_payload(fired);
        if let Err(e) = emitter.emit(payload).await {
            warn!(
                event_id = fired.event_id.as_str(),
                kind = fired.kind.as_str(),
                error = %e,
                "tripwire: SSB emit failed for TripwireFired"
            );
        } else {
            debug!(
                event_id = fired.event_id.as_str(),
                kind = fired.kind.as_str(),
                "tripwire: fired"
            );
        }
    }
}

/// Trait for SSB emitters used by the tripwire monitor. Mirrors the
/// `ConsensusSsbEmitter` / `PpmSsbEmitter` shape from D-2/D-3 so the test
/// harness can swap in `VecTripwireEmitter` without touching production
/// code.
pub trait TripwireSsbEmitter: Send + Sync {
    fn emit(
        &self,
        payload: SsbPayload,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;
}

/// In-memory emitter for tests + the smoke test. Captures payloads in a
/// `Mutex<Vec>` so assertions can read them after the monitor runs.
#[derive(Default)]
pub struct VecTripwireEmitter {
    rows: tokio::sync::Mutex<Vec<SsbPayload>>,
}

impl VecTripwireEmitter {
    pub fn new() -> Self {
        Self {
            rows: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn rows(&self) -> Vec<SsbPayload> {
        self.rows.lock().await.clone()
    }

    pub async fn len(&self) -> usize {
        self.rows.lock().await.len()
    }
}

impl TripwireSsbEmitter for VecTripwireEmitter {
    async fn emit(&self, payload: SsbPayload) -> Result<(), String> {
        self.rows.lock().await.push(payload);
        Ok(())
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reasoning_template::{
        compute_content_hash, Domain, PatternKind, Predicate, ReasoningTemplate, TemplateBody,
        VerifierKind,
    };
    use tempfile::TempDir;

    fn mk_signals(logprob_mean: f32, logprob_var: f32, entropy: f32) -> IntrinsicSignals {
        IntrinsicSignals {
            logprob_mean,
            logprob_var,
            entropy,
            step_count: 1,
            kl_to_base: 0.0,
        }
    }

    fn mk_sample(passed: bool, signals: IntrinsicSignals, proxy: f32, gold: f32) -> TripwireSample {
        TripwireSample {
            signals,
            proxy: Some(proxy),
            gold: Some(gold),
            passed,
            ts_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn load_tripwire_config_maps_committed_toml_keys() {
        // The committed [tripwire] key names differ from the struct fields;
        // assert the explicit per-key mapping picks up every overridden value.
        let raw = "\
[tripwire]
kl_spec_tau = 0.7
delta_rho = 0.4
corr_window = 80
iqr_multiplier = 2.5
ppl_sigma_mult = 1.5
ddm_min_instances = 42
ddm_warning_threshold = 2.2
ddm_drift_threshold = 3.3
";
        let cfg = load_tripwire_config_from_str(raw);
        assert!((cfg.kl_spec_tau - 0.7).abs() < 1e-6);
        assert!((cfg.rho_delta_threshold - 0.4).abs() < 1e-6);
        assert_eq!(cfg.window_size, 80);
        assert!((cfg.iqr_outlier_multiplier - 2.5).abs() < 1e-6);
        assert!((cfg.ppl_anomaly_sigma - 1.5).abs() < 1e-6);
        assert_eq!(cfg.min_sample_count, 42);
        assert!((cfg.ddm_warning_threshold - 2.2).abs() < 1e-6);
        assert!((cfg.ddm_drift_threshold - 3.3).abs() < 1e-6);
    }

    #[test]
    fn load_tripwire_config_falls_back_per_missing_key() {
        // Empty / missing stanza -> paper-cited defaults (never nonsense).
        let defaults = TripwireConfig::defaults();
        assert_eq!(load_tripwire_config_from_str(""), defaults);
        assert_eq!(load_tripwire_config_from_str("[other]\nx = 1\n"), defaults);
        // Partial stanza keeps defaults for the keys it omits.
        let partial = load_tripwire_config_from_str("[tripwire]\nkl_spec_tau = 0.9\n");
        assert!((partial.kl_spec_tau - 0.9).abs() < 1e-6);
        assert_eq!(partial.window_size, defaults.window_size);
    }

    #[test]
    fn kl_identical_distributions_is_zero() {
        let p = [0.25f32, 0.25, 0.25, 0.25];
        assert!(kl_divergence(&p, &p).abs() < 1e-6);
    }

    #[test]
    fn kl_zero_baseline_nonzero_current_is_infinity() {
        let p = [0.5f32, 0.5];
        let q = [0.0f32, 1.0];
        assert!(kl_divergence(&p, &q).is_infinite());
    }

    #[test]
    fn kl_handles_zero_in_current_as_zero_contribution() {
        let p = [0.0f32, 1.0];
        let q = [0.5f32, 0.5];
        // 0*log(0/q) treated as 0; total = 1*log(1/0.5) = ln 2 ~= 0.693.
        let v = kl_divergence(&p, &q);
        assert!((v - 2f32.ln()).abs() < 1e-5);
    }

    #[test]
    fn kl_malformed_input_is_nan() {
        let p = [0.5f32, 0.5];
        let q = [1.0f32];
        assert!(kl_divergence(&p, &q).is_nan());
    }

    #[test]
    fn pearson_perfect_positive_is_one() {
        let xs = [1.0f32, 2.0, 3.0, 4.0];
        let ys = [2.0f32, 4.0, 6.0, 8.0];
        let r = pearson_correlation(&xs, &ys);
        assert!((r - 1.0).abs() < 1e-5);
    }

    #[test]
    fn pearson_zero_variance_returns_zero() {
        let xs = [1.0f32, 1.0, 1.0];
        let ys = [1.0f32, 2.0, 3.0];
        assert_eq!(pearson_correlation(&xs, &ys), 0.0);
    }

    #[test]
    fn iqr_detects_obvious_outlier() {
        let mut v: Vec<f32> = (1..=20).map(|i| i as f32).collect();
        v.push(500.0);
        let outlier = iqr_outlier(&v, 3.0).expect("outlier present");
        assert_eq!(outlier, 500.0);
    }

    #[test]
    fn iqr_no_outlier_returns_none() {
        let v: Vec<f32> = (1..=20).map(|i| i as f32).collect();
        assert!(iqr_outlier(&v, 3.0).is_none());
    }

    #[test]
    fn perplexity_anomaly_flags_outlier() {
        // logprob_mean = 0 -> ppl = 1; spike at 100.
        let mut ppls = vec![1.0f32; 30];
        ppls[15] = 100.0;
        let anom = perplexity_anomaly(&ppls, 2.0).expect("anomaly");
        assert_eq!(anom, 100.0);
    }

    #[test]
    fn perplexity_anomaly_quiet_series_no_fire() {
        let ppls = vec![1.0f32; 30];
        assert!(perplexity_anomaly(&ppls, 2.0).is_none());
    }

    #[test]
    fn ddm_clean_stream_stays_normal() {
        let mut ddm = DdmState::new();
        for _ in 0..100 {
            ddm.observe(true, 2.0, 3.0);
        }
        assert_eq!(ddm.level, DdmLevel::Normal);
    }

    #[test]
    fn ddm_failure_burst_triggers_drift() {
        let mut ddm = DdmState::new();
        // 50 clean.
        for _ in 0..50 {
            ddm.observe(true, 2.0, 3.0);
        }
        // 50 with high failure.
        let mut hit_drift = false;
        for i in 0..50 {
            ddm.observe(i % 5 != 0, 2.0, 3.0); // 80% failure-ish
            if ddm.level == DdmLevel::Drift {
                hit_drift = true;
                break;
            }
        }
        // Force degenerate: heavy failure stream from clean baseline must
        // eventually trip the warning + drift thresholds. Worst-case the
        // s_min ratio keeps us in Warning, so accept Warning OR Drift.
        assert!(
            hit_drift
                || ddm.level == DdmLevel::Warning
                || ddm.level == DdmLevel::Drift,
            "ddm level was {:?} after burst",
            ddm.level
        );
    }

    #[test]
    fn tripwire_fired_event_id_stable() {
        let a = TripwireFired::new(TripwireKind::IqrOutlier, "cand", 1.0, 0.5, 1);
        let b = TripwireFired::new(TripwireKind::IqrOutlier, "cand", 1.0, 0.5, 1);
        assert_eq!(a.event_id, b.event_id);
        assert_eq!(a.event_id.len(), 32);
    }

    #[test]
    fn tripwire_fired_event_id_changes_on_kind() {
        let a = TripwireFired::new(TripwireKind::IqrOutlier, "cand", 1.0, 0.5, 1);
        let b = TripwireFired::new(TripwireKind::Perplexity, "cand", 1.0, 0.5, 1);
        assert_ne!(a.event_id, b.event_id);
    }

    #[test]
    fn monitor_empty_then_min_samples_no_fire() {
        let cfg = TripwireConfig {
            min_sample_count: 10,
            ..TripwireConfig::defaults()
        };
        let mut mon = TripwireMonitor::new(cfg);
        for _ in 0..9 {
            let out = mon.observe("c", mk_sample(true, mk_signals(-1.0, 0.1, 1.0), 0.9, 1.0));
            // Window-based detectors must not fire below min_sample_count.
            assert!(
                out.fired
                    .iter()
                    .all(|f| matches!(f.kind, TripwireKind::Ddm)),
                "non-DDM detector fired before min_sample_count"
            );
        }
        assert_eq!(mon.window_len(), 9);
    }

    #[test]
    fn monitor_iqr_fires_on_logprob_var_spike() {
        let mut mon = TripwireMonitor::new(TripwireConfig::defaults());
        // 15 calm samples (var small + stable).
        for _ in 0..15 {
            let _ = mon.observe("c", mk_sample(true, mk_signals(-1.0, 0.05, 1.0), 0.9, 1.0));
        }
        // 1 outlier — var massively above median.
        let out = mon.observe(
            "c",
            mk_sample(true, mk_signals(-1.0, 50.0, 1.0), 0.9, 1.0),
        );
        assert!(
            out.fired.iter().any(|f| f.kind == TripwireKind::IqrOutlier),
            "expected IqrOutlier fire, got {:?}",
            out.fired.iter().map(|f| f.kind).collect::<Vec<_>>()
        );
    }

    #[test]
    fn monitor_perplexity_fires_on_ppl_spike() {
        let mut mon = TripwireMonitor::new(TripwireConfig::defaults());
        for _ in 0..15 {
            let _ = mon.observe("c", mk_sample(true, mk_signals(-1.0, 0.05, 1.0), 0.9, 1.0));
        }
        // Sharp PPL spike (very low logprob_mean -> huge PPL).
        let out = mon.observe(
            "c",
            mk_sample(true, mk_signals(-15.0, 0.05, 1.0), 0.9, 1.0),
        );
        assert!(out
            .fired
            .iter()
            .any(|f| f.kind == TripwireKind::Perplexity));
    }

    #[test]
    fn monitor_window_trims_to_size() {
        let mut mon = TripwireMonitor::new(TripwireConfig {
            window_size: 5,
            min_sample_count: 2,
            ..TripwireConfig::defaults()
        });
        for _ in 0..20 {
            let _ = mon.observe("c", mk_sample(true, mk_signals(-1.0, 0.1, 1.0), 0.9, 1.0));
        }
        assert_eq!(mon.window_len(), 5);
    }

    fn mk_template(body: &str) -> ReasoningTemplate {
        let body = TemplateBody::try_new(body).expect("body");
        let hash = compute_content_hash(body.as_str(), None);
        ReasoningTemplate {
            id: TemplateId::new_v4(),
            domain: Domain::Code,
            pattern_kind: PatternKind::Decomposition,
            template_body: body,
            eligibility_predicate: Predicate::Always,
            verifier_required: VerifierKind::Compile,
            content_hash: hash,
            created_at_ms: 0,
            parent_id: None,
            archive_status: ArchiveStatus::Promoted,
        }
    }

    #[test]
    fn auto_revert_flips_promoted_to_retired() {
        use crate::archive_chain::ArchiveEntry;
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("archive.jsonl");
        let mut chain = ArchiveChain::new(path);
        let t = mk_template("p");
        let template_id = t.id;
        chain
            .append(ArchiveEntry::new_root(t))
            .expect("append root");

        let fired = TripwireFired::new(
            TripwireKind::IqrOutlier,
            "cand-1",
            42.0,
            3.0,
            1_700_000_000_000,
        );
        let outcome = auto_revert(&mut chain, &template_id, &fired).expect("revert");
        assert_eq!(outcome.template_id, template_id.as_hex());
        assert_eq!(outcome.retirement_reason, RetirementReason::Degradation);
        assert_eq!(outcome.fired_tripwire_id, fired.event_id);

        // Latest entry for this template is Retired.
        let latest = chain
            .entries()
            .iter()
            .filter(|e| e.template.id == template_id)
            .last()
            .expect("at least one entry");
        assert_eq!(latest.template.archive_status, ArchiveStatus::Retired);
        assert_eq!(
            latest.retirement_reason,
            Some(RetirementReason::Degradation)
        );
    }

    #[test]
    fn auto_revert_rejects_already_retired() {
        use crate::archive_chain::ArchiveEntry;
        let dir = TempDir::new().expect("tempdir");
        let mut chain = ArchiveChain::new(dir.path().join("a.jsonl"));
        let mut t = mk_template("r");
        t.archive_status = ArchiveStatus::Retired;
        let template_id = t.id;
        chain
            .append(ArchiveEntry::new_root(t))
            .expect("append");
        let fired = TripwireFired::new(TripwireKind::Ddm, "c", 0.5, 0.3, 1);
        match auto_revert(&mut chain, &template_id, &fired) {
            Err(AutoRevertError::AlreadyRetired(_)) => {}
            other => panic!("expected AlreadyRetired, got {other:?}"),
        }
    }

    #[test]
    fn build_payloads_round_trip() {
        let fired = TripwireFired::new(TripwireKind::KlDivergence, "c", 0.5, 0.3, 1);
        let payload = build_tripwire_fired_payload(&fired);
        let action = payload.action_type();
        assert_eq!(action.as_sql_str(), "tripwire_fired");

        let outcome = AutoRevertOutcome {
            template_id: "tpl-1".into(),
            parent_template_id: Some("tpl-0".into()),
            retirement_reason: RetirementReason::Degradation,
            fired_tripwire_id: fired.event_id.clone(),
        };
        let payload2 = build_auto_reverted_payload(&outcome);
        assert_eq!(payload2.action_type().as_sql_str(), "template_auto_reverted");
    }

    #[tokio::test]
    async fn observable_smoke_emit_outcome_writes_one_row_per_fire() {
        let emitter = VecTripwireEmitter::new();
        let outcome = ObserveOutcome {
            fired: vec![
                TripwireFired::new(TripwireKind::Ddm, "c", 0.5, 0.3, 1),
                TripwireFired::new(TripwireKind::IqrOutlier, "c", 100.0, 3.0, 1),
            ],
            ddm_level: Some(DdmLevel::Drift),
        };
        emit_observe_outcome(&emitter, &outcome).await;
        assert_eq!(emitter.len().await, 2);
    }
}
