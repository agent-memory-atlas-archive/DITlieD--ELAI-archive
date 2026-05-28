//! E-48 Task 6 — conformal triple-threshold convergence (preemptive-unsolvable).
//!
//! Extends the existing dual-threshold convergence (phi_min upper +
//! psi_min quality floor) with a third lower bound -- preemptive
//! unsolvable. When confidence stays below the lower threshold for
//! `low_delta_streak` consecutive rounds AND the
//! intermediate-evidence delta stays below the calibration floor, the
//! orchestrator converges with
//! [`crate::research_tree::ConvergenceStatus::PreemptiveUnsolvable`].
//!
//! Grounding:
//!   * arXiv:2602.03814 -- Conformal Thinking (dual-threshold reasoning
//!     stop with distribution-free risk control). E-48 extends to a
//!     triple-threshold by adding the lower-bound preemptive-unsolvable
//!     arm.
//!
//! Calibration:
//!   * Per the plan open question #5, the lower threshold needs a
//!     held-out set of known-unsolvable research tasks to calibrate.
//!   * ELAI does not yet have this set. Default threshold is
//!     conservative (rarely fires) and the producer-side emit fn
//!     records `threshold_uncalibrated: true` on the SSB row so
//!     consumers can surface the warning.
//!   * Once ELAI accumulates 30+ tagged-unsolvable cases, a follow-up
//!     patch will tune the constants here and flip the
//!     `threshold_uncalibrated` flag default.

use crate::research_tree::ConvergenceStatus;

/// Default upper confidence threshold ("goal satisfied"). Mirrors the
/// existing B-7 dual-threshold `phi_min` default. Sourced from
/// [`crate::research_tier::ResearchTierConfig`] at runtime; this const
/// is the fallback when no tier config is available.
pub const DEFAULT_GOAL_SATISFIED_UPPER: f64 = 0.85;

/// Default quality floor threshold (B-7 `psi_min` default).
pub const DEFAULT_QUALITY_FLOOR: f64 = 0.60;

/// Default lower threshold below which a research task starts looking
/// preemptively unsolvable. Conservative -- requires confidence < 0.20
/// AND the rest of the gate. Calibration TODO when ELAI accumulates
/// tagged-unsolvable cases (open question #5).
pub const DEFAULT_PREEMPTIVE_UNSOLVABLE_LOWER: f64 = 0.20;

/// Minimum number of rounds before the preemptive-unsolvable gate may
/// fire. Avoids cold-start false positives (a research task may show
/// low confidence in its first 1-2 rounds simply because it has not
/// yet retrieved enough sources).
pub const DEFAULT_MIN_ROUNDS_BEFORE_PREEMPTIVE: u32 = 3;

/// Maximum intermediate-evidence delta (ratio) consistent with a
/// preemptive-unsolvable verdict. If the latest 3 rounds each delivered
/// `< this` new evidence, the gate fires.
pub const DEFAULT_PREEMPTIVE_DELTA_FLOOR: f64 = 0.05;

/// Number of consecutive low-delta rounds required before the gate fires.
pub const DEFAULT_LOW_DELTA_STREAK_REQUIRED: u32 = 3;

/// Configurable thresholds for the triple-threshold gate. Falls back to
/// the `DEFAULT_*` constants when callers do not override.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TripleThresholdConfig {
    pub goal_satisfied_upper: f64,
    pub quality_floor: f64,
    pub preemptive_unsolvable_lower: f64,
    pub min_rounds_before_preemptive: u32,
    pub preemptive_delta_floor: f64,
    pub low_delta_streak_required: u32,
    /// Whether the lower threshold has been calibrated against a
    /// tagged-unsolvable corpus. Defaults to `false` -- consumers
    /// surface the startup warning per the open-question note.
    pub threshold_calibrated: bool,
}

impl Default for TripleThresholdConfig {
    fn default() -> Self {
        Self {
            goal_satisfied_upper: DEFAULT_GOAL_SATISFIED_UPPER,
            quality_floor: DEFAULT_QUALITY_FLOOR,
            preemptive_unsolvable_lower: DEFAULT_PREEMPTIVE_UNSOLVABLE_LOWER,
            min_rounds_before_preemptive: DEFAULT_MIN_ROUNDS_BEFORE_PREEMPTIVE,
            preemptive_delta_floor: DEFAULT_PREEMPTIVE_DELTA_FLOOR,
            low_delta_streak_required: DEFAULT_LOW_DELTA_STREAK_REQUIRED,
            threshold_calibrated: false,
        }
    }
}

/// Inputs to the triple-threshold gate at the end of one REFLECT cycle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GateInputs {
    /// Current research-tree confidence in `[0.0, 1.0]`. Maps to phi
    /// in the B-7 dual-threshold convention.
    pub confidence: f64,
    /// Number of REFLECT rounds completed so far.
    pub rounds_completed: u32,
    /// Number of consecutive recent rounds where the
    /// intermediate-evidence delta stayed below
    /// `config.preemptive_delta_floor`.
    pub low_delta_streak: u32,
}

/// Evaluate the triple-threshold gate.
///
/// Returns:
///   * `Some(ConvergenceStatus::Converged)` when confidence has crossed
///     the upper bound.
///   * `Some(ConvergenceStatus::PreemptiveUnsolvable {..})` when all
///     four preemptive-unsolvable conditions hold:
///       1. `rounds_completed >= config.min_rounds_before_preemptive`
///       2. `confidence < config.preemptive_unsolvable_lower`
///       3. `low_delta_streak >= config.low_delta_streak_required`
///       4. `!config.threshold_calibrated`-derived warning rides the
///          payload but does NOT veto the gate (consumers must accept
///          the uncalibrated verdict and surface the warning).
///   * `None` when the gate is undecided -- caller should continue the
///     loop until a verdict surfaces or the iteration cap is hit.
///
/// NaN-safe: NaN confidence is treated as 0.0 (drives the gate toward
/// PreemptiveUnsolvable if the streak / round conditions hold, matching
/// the conservative-default-fires-rarely posture).
#[must_use]
pub fn evaluate(inputs: GateInputs, config: TripleThresholdConfig) -> Option<ConvergenceStatus> {
    let confidence = if inputs.confidence.is_nan() {
        0.0
    } else {
        inputs.confidence
    };

    // Upper bound -- goal satisfied.
    if confidence >= config.goal_satisfied_upper {
        return Some(ConvergenceStatus::Converged);
    }

    // Lower bound -- preemptive unsolvable.
    let preemptive = inputs.rounds_completed >= config.min_rounds_before_preemptive
        && confidence < config.preemptive_unsolvable_lower
        && inputs.low_delta_streak >= config.low_delta_streak_required;
    if preemptive {
        return Some(ConvergenceStatus::PreemptiveUnsolvable {
            rounds: inputs.rounds_completed,
            low_delta_streak: inputs.low_delta_streak,
            final_confidence: confidence,
            threshold_uncalibrated: !config.threshold_calibrated,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upper_bound_converges() {
        let cfg = TripleThresholdConfig::default();
        let v = evaluate(
            GateInputs {
                confidence: 0.95,
                rounds_completed: 5,
                low_delta_streak: 0,
            },
            cfg,
        );
        assert!(matches!(v, Some(ConvergenceStatus::Converged)));
    }

    #[test]
    fn lower_bound_fires_preemptive_unsolvable() {
        let cfg = TripleThresholdConfig::default();
        let v = evaluate(
            GateInputs {
                confidence: 0.10,
                rounds_completed: 4,
                low_delta_streak: 3,
            },
            cfg,
        );
        match v {
            Some(ConvergenceStatus::PreemptiveUnsolvable {
                rounds,
                low_delta_streak,
                final_confidence,
                threshold_uncalibrated,
            }) => {
                assert_eq!(rounds, 4);
                assert_eq!(low_delta_streak, 3);
                assert!((final_confidence - 0.10).abs() < f64::EPSILON);
                assert!(threshold_uncalibrated, "default config is uncalibrated");
            }
            other => panic!("expected PreemptiveUnsolvable, got {other:?}"),
        }
    }

    #[test]
    fn cold_start_does_not_fire_preemptive() {
        let cfg = TripleThresholdConfig::default();
        let v = evaluate(
            GateInputs {
                confidence: 0.05,
                rounds_completed: 1,
                low_delta_streak: 1,
            },
            cfg,
        );
        // rounds_completed below min_rounds_before_preemptive -> undecided.
        assert!(v.is_none());
    }

    #[test]
    fn moderate_confidence_undecided() {
        let cfg = TripleThresholdConfig::default();
        let v = evaluate(
            GateInputs {
                confidence: 0.50,
                rounds_completed: 5,
                low_delta_streak: 3,
            },
            cfg,
        );
        assert!(v.is_none());
    }

    #[test]
    fn calibrated_config_flips_uncalibrated_flag() {
        let cfg = TripleThresholdConfig {
            threshold_calibrated: true,
            ..Default::default()
        };
        let v = evaluate(
            GateInputs {
                confidence: 0.10,
                rounds_completed: 4,
                low_delta_streak: 3,
            },
            cfg,
        );
        match v {
            Some(ConvergenceStatus::PreemptiveUnsolvable {
                threshold_uncalibrated,
                ..
            }) => assert!(!threshold_uncalibrated),
            other => panic!("expected PreemptiveUnsolvable, got {other:?}"),
        }
    }

    #[test]
    fn nan_confidence_treated_as_zero() {
        let cfg = TripleThresholdConfig::default();
        let v = evaluate(
            GateInputs {
                confidence: f64::NAN,
                rounds_completed: 4,
                low_delta_streak: 3,
            },
            cfg,
        );
        match v {
            Some(ConvergenceStatus::PreemptiveUnsolvable {
                final_confidence, ..
            }) => assert_eq!(final_confidence, 0.0),
            other => panic!("expected PreemptiveUnsolvable, got {other:?}"),
        }
    }
}
