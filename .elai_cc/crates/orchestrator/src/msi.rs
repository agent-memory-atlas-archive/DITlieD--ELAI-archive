//! B-23 Phase 4H Group E — Modification Solidification Index (4H.7).
//!
//! Per-PipelineMode exponential-decay weighted moving average of variant
//! delta_score. Distinct from `self_mod_layer1a::MsiState` (apply-success
//! +/-1 over the last N applies); this module tracks the SCORE TRAJECTORY
//! of the DGM config archive across pipeline outcomes so the dashboard can
//! render IMPROVING/STABILIZING/DIVERGING per mode.
//!
//! Formula (spec phase-4H.txt:142-152, 738-768):
//!   delta_score_i = variant.score - parent.score   (in [-1, +1])
//!   w_i = exp(-lambda * (N - i))                   (lambda = 0.3, i = position)
//!   MSI = sum(w_i * delta_score_i) / sum(w_i)      (normalised, in [-1, +1])
//!   IMPROVING:    MSI > 0.30
//!   STABILIZING: -0.10 < MSI <= 0.30
//!   DIVERGING:    MSI <= -0.10
//!
//! Grounding: workflow/papers/darwinnet-interface.md (PSI is conceptual
//! ancestor; formula is OUR DESIGN per spec phase-4H.txt:794-800).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::self_mod_layer1a::PipelineMode;
use crate::self_mod_types::VariantId;

// ---------------------------------------------------------------------------
// Named constants (spec phase-4H.txt:741-746)
// ---------------------------------------------------------------------------

/// MSI decay constant. Smaller = more recency-weighted. Spec default 0.3.
pub const MSI_LAMBDA: f32 = 0.3;
/// Window size — number of variants considered in the moving average.
pub const MSI_WINDOW: usize = 10;
/// Group E audit BLOCK fix: re-export the canonical thresholds from
/// `self_mod_layer1a` so the two modules can never silently drift. The
/// two values were previously declared in both modules with identical
/// numeric content but no compile-time enforcement.
pub use crate::self_mod_layer1a::MSI_DIVERGING_THRESHOLD;
pub use crate::self_mod_layer1a::MSI_IMPROVING_THRESHOLD;

// ---------------------------------------------------------------------------
// Verdict
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MsiVerdict {
    Improving,
    Stabilizing,
    Diverging,
}

impl MsiVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            MsiVerdict::Improving => "improving",
            MsiVerdict::Stabilizing => "stabilizing",
            MsiVerdict::Diverging => "diverging",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsiSnapshot {
    pub pipeline_mode: PipelineMode,
    pub msi_value: f32,
    pub samples: u32,
    pub verdict: MsiVerdict,
}

// ---------------------------------------------------------------------------
// Per-mode tracker
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModeState {
    /// Newest delta_score at index 0; newest-first ring (capped to MSI_WINDOW).
    deltas: Vec<f32>,
    /// Latest variant id we have observed for this mode (debug/dashboard hint).
    latest_variant: Option<VariantId>,
}

impl Default for ModeState {
    fn default() -> Self {
        Self {
            deltas: Vec::with_capacity(MSI_WINDOW),
            latest_variant: None,
        }
    }
}

/// Per-PipelineMode MSI tracker. Modes are independent — DIVERGING in
/// SURGICAL does not pause GREENFIELD self-mod (spec phase-4H.txt:752-754).
#[derive(Debug, Default, Clone)]
pub struct MsiTracker {
    states: BTreeMap<PipelineMode, ModeState>,
    lambda: f32,
    window: usize,
    /// Group E audit BLOCK fix: EVO Withhold persistently overrides the
    /// computed verdict to STABILIZING for the named modes until cleared
    /// (spec phase-4H.txt:1007-1011 — "MSI verdict for the mode is FORCED
    /// to STABILIZING"). The underlying moving average is preserved so a
    /// subsequent `clear_force_stabilizing` restores the computed verdict.
    forced_stabilizing: BTreeSet<PipelineMode>,
}

impl MsiTracker {
    pub fn new() -> Self {
        Self {
            states: BTreeMap::new(),
            lambda: MSI_LAMBDA,
            window: MSI_WINDOW,
            forced_stabilizing: BTreeSet::new(),
        }
    }

    pub fn with_tuning(mut self, lambda: f32, window: usize) -> Self {
        self.lambda = lambda;
        self.window = window.max(1);
        self
    }

    /// Record a (variant_id, delta_score) pair for `mode`. `delta_score`
    /// is variant.score - parent.score (already in [-1, +1]).
    pub fn record(&mut self, mode: PipelineMode, variant: VariantId, delta: f32) {
        let state = self.states.entry(mode).or_default();
        let clamped = delta.clamp(-1.0, 1.0);
        state.deltas.insert(0, clamped);
        if state.deltas.len() > self.window {
            state.deltas.truncate(self.window);
        }
        state.latest_variant = Some(variant);
    }

    /// Compute the weighted moving average of recorded deltas for `mode`.
    /// Returns 0.0 when no samples exist for the mode.
    pub fn value(&self, mode: PipelineMode) -> f32 {
        let Some(state) = self.states.get(&mode) else {
            return 0.0;
        };
        if state.deltas.is_empty() {
            return 0.0;
        }
        let mut weight_sum = 0.0f32;
        let mut weighted = 0.0f32;
        for (i, d) in state.deltas.iter().enumerate() {
            // Newest at index 0; the spec's `w_i = exp(-lambda * (N - i))`
            // with i = position (newest = N) collapses to `exp(-lambda * i)`
            // when we walk newest-first. Identical recency ordering.
            let exp_arg = -(self.lambda * (i as f32));
            let w = exp_arg.exp();
            weighted += w * d;
            weight_sum += w;
        }
        if weight_sum <= 0.0 {
            return 0.0;
        }
        (weighted / weight_sum).clamp(-1.0, 1.0)
    }

    pub fn verdict(&self, mode: PipelineMode) -> MsiVerdict {
        if self.forced_stabilizing.contains(&mode) {
            return MsiVerdict::Stabilizing;
        }
        msi_verdict(self.value(mode))
    }

    /// Returns true when the EVO gate has forced this mode's verdict to
    /// STABILIZING (overriding the computed value).
    pub fn is_forced_stabilizing(&self, mode: PipelineMode) -> bool {
        self.forced_stabilizing.contains(&mode)
    }

    /// Clear the EVO Withhold override for `mode`. The next `verdict()`
    /// call will reflect the computed moving average again.
    pub fn clear_force_stabilizing(&mut self, mode: PipelineMode) {
        self.forced_stabilizing.remove(&mode);
    }

    pub fn samples(&self, mode: PipelineMode) -> u32 {
        self.states
            .get(&mode)
            .map(|s| s.deltas.len())
            .unwrap_or(0)
            .try_into()
            .unwrap_or(u32::MAX)
    }

    /// Snapshot for a single mode.
    pub fn snapshot(&self, mode: PipelineMode) -> MsiSnapshot {
        let value = self.value(mode);
        MsiSnapshot {
            pipeline_mode: mode,
            msi_value: value,
            samples: self.samples(mode),
            verdict: msi_verdict(value),
        }
    }

    /// Snapshot for ALL modes that have at least one recorded sample.
    pub fn all_snapshots(&self) -> Vec<MsiSnapshot> {
        self.states
            .keys()
            .copied()
            .map(|m| self.snapshot(m))
            .collect()
    }

    /// Force STABILIZING for `mode` (used by Group E's EVO evolution-gate
    /// Withhold path: forces STABILIZING regardless of the underlying MSI
    /// computation; spec phase-4H.txt:1007-1011). Subsequent `verdict()`
    /// calls return STABILIZING until [`Self::clear_force_stabilizing`].
    pub fn force_stabilizing(&mut self, mode: PipelineMode) -> MsiSnapshot {
        self.forced_stabilizing.insert(mode);
        MsiSnapshot {
            pipeline_mode: mode,
            msi_value: self.value(mode),
            samples: self.samples(mode),
            verdict: MsiVerdict::Stabilizing,
        }
    }
}

/// Pure verdict mapping function. Exported so callers (e.g. EVO) can
/// reuse the threshold logic without needing a tracker instance.
pub fn msi_verdict(value: f32) -> MsiVerdict {
    if value > MSI_IMPROVING_THRESHOLD {
        MsiVerdict::Improving
    } else if value <= MSI_DIVERGING_THRESHOLD {
        MsiVerdict::Diverging
    } else {
        MsiVerdict::Stabilizing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tracker_returns_zero() {
        let t = MsiTracker::new();
        assert_eq!(t.value(PipelineMode::Surgical), 0.0);
        assert_eq!(t.verdict(PipelineMode::Surgical), MsiVerdict::Stabilizing);
    }

    #[test]
    fn improving_after_consecutive_gains() {
        let mut t = MsiTracker::new();
        for i in 0..10 {
            t.record(PipelineMode::Surgical, format!("v{i}"), 0.5);
        }
        let v = t.value(PipelineMode::Surgical);
        assert!(v > MSI_IMPROVING_THRESHOLD, "expected IMPROVING got {v}");
        assert_eq!(t.verdict(PipelineMode::Surgical), MsiVerdict::Improving);
    }

    #[test]
    fn diverging_after_consecutive_losses() {
        let mut t = MsiTracker::new();
        for i in 0..10 {
            t.record(PipelineMode::Surgical, format!("v{i}"), -0.5);
        }
        let v = t.value(PipelineMode::Surgical);
        assert!(v <= MSI_DIVERGING_THRESHOLD, "expected DIVERGING got {v}");
        assert_eq!(t.verdict(PipelineMode::Surgical), MsiVerdict::Diverging);
    }

    #[test]
    fn modes_are_independent() {
        let mut t = MsiTracker::new();
        for i in 0..10 {
            t.record(PipelineMode::Surgical, format!("v{i}"), -0.6);
            t.record(PipelineMode::Greenfield, format!("g{i}"), 0.6);
        }
        assert_eq!(t.verdict(PipelineMode::Surgical), MsiVerdict::Diverging);
        assert_eq!(t.verdict(PipelineMode::Greenfield), MsiVerdict::Improving);
    }

    #[test]
    fn window_caps_at_ten() {
        let mut t = MsiTracker::new();
        for i in 0..50 {
            t.record(PipelineMode::Oracle, format!("v{i}"), 0.5);
        }
        assert_eq!(t.samples(PipelineMode::Oracle), MSI_WINDOW as u32);
    }

    #[test]
    fn anti_fragility_recovery_within_window() {
        // 5 declining variants then 5 improving; assert recovery from DIVERGING.
        let mut t = MsiTracker::new();
        for i in 0..5 {
            t.record(PipelineMode::Surgical, format!("bad{i}"), -0.6);
        }
        assert_eq!(t.verdict(PipelineMode::Surgical), MsiVerdict::Diverging);
        for i in 0..5 {
            t.record(PipelineMode::Surgical, format!("good{i}"), 0.7);
        }
        // After 5 improving variants, MSI should not be DIVERGING any more
        // (recovery within window per spec phase-4H.txt:761-767).
        assert_ne!(t.verdict(PipelineMode::Surgical), MsiVerdict::Diverging);
    }

    #[test]
    fn force_stabilizing_overrides_verdict() {
        let mut t = MsiTracker::new();
        for i in 0..10 {
            t.record(PipelineMode::Surgical, format!("v{i}"), 0.6);
        }
        // Without override the verdict is IMPROVING.
        assert_eq!(t.verdict(PipelineMode::Surgical), MsiVerdict::Improving);
        let snap = t.force_stabilizing(PipelineMode::Surgical);
        assert_eq!(snap.verdict, MsiVerdict::Stabilizing);
    }

    #[test]
    fn delta_clamped_to_unit_range() {
        let mut t = MsiTracker::new();
        t.record(PipelineMode::Surgical, "v0".into(), 5.0); // out of range
        t.record(PipelineMode::Surgical, "v1".into(), -5.0);
        let v = t.value(PipelineMode::Surgical);
        assert!((-1.0..=1.0).contains(&v));
    }

    #[test]
    fn random_sequence_msi_in_unit_range() {
        // Lightweight 1000-call simulation with a deterministic LCG. Verifies
        // the MSI invariant ([-1, 1]) for any input sequence.
        let mut t = MsiTracker::new();
        let mut x: u32 = 0x1234_5678;
        for i in 0..1_000 {
            // xorshift-ish; deterministic, not cryptographic.
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            let d = ((x as i32) as f32 / i32::MAX as f32).clamp(-1.0, 1.0);
            t.record(PipelineMode::Surgical, format!("v{i}"), d);
        }
        let v = t.value(PipelineMode::Surgical);
        assert!((-1.0..=1.0).contains(&v));
    }
}
