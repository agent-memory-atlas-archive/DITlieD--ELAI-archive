//! Offline calibration for the D-10 grounding router.
//!
//! Calibration is deliberately sample-scoped: it measures disagreement on a
//! declared held-out sample and never asserts global sufficiency.

use std::sync::{OnceLock, RwLock};

use crate::claim::ClaimType;
use crate::grounding_mode::GroundingMode;

/// One held-out sample row for offline calibration.
#[derive(Debug, Clone, PartialEq)]
pub struct GroundingCalibrationSample {
    pub claim_type: ClaimType,
    /// Mode the cheap D-10 router produced.
    pub cheap_mode: GroundingMode,
    /// Mode from the expensive path (the oracle label for this sample).
    pub expensive_mode: GroundingMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroundingRouterCalibration {
    pub sample_size: usize,
    pub disagreement_count: usize,
    pub disagreement_rate: f64,
    /// Human-readable declaration of the held-out sample.
    pub sample_declaration: String,
    /// Whether measured disagreement is sufficient to tighten this router.
    pub tighten_router: bool,
}

/// Disagreement rate at/above this value enables tightening.
pub const DISAGREE_TIGHTEN_THRESHOLD: f64 = 0.25;
/// Minimum samples before tightening may be enabled.
pub const MIN_SAMPLES_FOR_TIGHTEN: usize = 4;

/// Offline calibration over one declared held-out sample.
#[must_use]
pub fn calibrate_grounding_router(
    samples: &[GroundingCalibrationSample],
    sample_declaration: &str,
) -> GroundingRouterCalibration {
    let sample_size = samples.len();
    let disagreement_count = samples
        .iter()
        .filter(|sample| sample.cheap_mode != sample.expensive_mode)
        .count();
    let disagreement_rate = if sample_size == 0 {
        0.0
    } else {
        disagreement_count as f64 / sample_size as f64
    };
    let declaration = sample_declaration.trim();
    let declared = !declaration.is_empty();
    let tighten_router = declared
        && sample_size >= MIN_SAMPLES_FOR_TIGHTEN
        && disagreement_rate >= DISAGREE_TIGHTEN_THRESHOLD;
    GroundingRouterCalibration {
        sample_size,
        disagreement_count,
        disagreement_rate,
        sample_declaration: if declared {
            declaration.to_owned()
        } else {
            "UNDECLARED_SAMPLE".to_owned()
        },
        tighten_router,
    }
}

/// Apply measured calibration tightening to a live routing decision.
#[must_use]
pub fn apply_grounding_calibration(
    mode: GroundingMode,
    claim_type: ClaimType,
    cal: &GroundingRouterCalibration,
) -> GroundingMode {
    if cal.tighten_router
        && mode == GroundingMode::Constructive
        && matches!(claim_type, ClaimType::Factual | ClaimType::Computational)
    {
        GroundingMode::FalsifiabilityCrossFamily
    } else {
        mode
    }
}

fn calibration_slot() -> &'static RwLock<Option<GroundingRouterCalibration>> {
    static SLOT: OnceLock<RwLock<Option<GroundingRouterCalibration>>> = OnceLock::new();
    SLOT.get_or_init(|| RwLock::new(None))
}

/// Install offline calibration for subsequent route_claim calls.
pub fn install_grounding_router_calibration(cal: GroundingRouterCalibration) {
    if let Ok(mut slot) = calibration_slot().write() {
        *slot = Some(cal);
    }
}

/// Clear the process-wide offline calibration.
pub fn clear_grounding_router_calibration() {
    if let Ok(mut slot) = calibration_slot().write() {
        *slot = None;
    }
}

/// Return the currently installed offline calibration, if any.
#[must_use]
pub fn current_grounding_calibration() -> Option<GroundingRouterCalibration> {
    calibration_slot().read().ok().and_then(|slot| slot.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(cheap_mode: GroundingMode, expensive_mode: GroundingMode) -> GroundingCalibrationSample {
        GroundingCalibrationSample {
            claim_type: ClaimType::Factual,
            cheap_mode,
            expensive_mode,
        }
    }

    #[test]
    fn empty_declared_sample_has_zero_rate_and_no_tightening() {
        let result = calibrate_grounding_router(&[], "ids=none; source=fixture");
        assert_eq!(result.sample_size, 0);
        assert_eq!(result.disagreement_count, 0);
        assert_eq!(result.disagreement_rate, 0.0);
        assert!(!result.tighten_router);
    }

    #[test]
    fn undeclared_sample_never_tightens() {
        let samples = (0..4)
            .map(|_| sample(GroundingMode::Constructive, GroundingMode::ConservativeFallback))
            .collect::<Vec<_>>();
        let result = calibrate_grounding_router(&samples, "  \n\t");
        assert_eq!(result.sample_declaration, "UNDECLARED_SAMPLE");
        assert_eq!(result.disagreement_rate, 1.0);
        assert!(!result.tighten_router);
    }

    #[test]
    fn four_agreeing_samples_do_not_tighten() {
        let samples = (0..4)
            .map(|_| sample(GroundingMode::Constructive, GroundingMode::Constructive))
            .collect::<Vec<_>>();
        let result = calibrate_grounding_router(&samples, "ids=1-4; source=held-out");
        assert_eq!(result.disagreement_rate, 0.0);
        assert!(!result.tighten_router);
    }

    #[test]
    fn two_of_four_disagreements_tighten_declared_router() {
        let samples = vec![
            sample(GroundingMode::Constructive, GroundingMode::ConservativeFallback),
            sample(GroundingMode::Constructive, GroundingMode::Constructive),
            sample(GroundingMode::Constructive, GroundingMode::FalsifiabilityCrossFamily),
            sample(GroundingMode::Constructive, GroundingMode::Constructive),
        ];
        let result = calibrate_grounding_router(&samples, "ids=1-4; source=held-out");
        assert_eq!(result.disagreement_count, 2);
        assert_eq!(result.disagreement_rate, 0.5);
        assert!(result.tighten_router);
    }

    #[test]
    fn tightening_downgrades_factual_constructive_mode() {
        let cal = GroundingRouterCalibration {
            sample_size: 4,
            disagreement_count: 2,
            disagreement_rate: 0.5,
            sample_declaration: "held-out ids=1-4".to_owned(),
            tighten_router: true,
        };
        assert_eq!(
            apply_grounding_calibration(GroundingMode::Constructive, ClaimType::Factual, &cal),
            GroundingMode::FalsifiabilityCrossFamily
        );
    }

    #[test]
    fn non_tightening_leaves_constructive_mode_unchanged() {
        let cal = GroundingRouterCalibration {
            sample_size: 4,
            disagreement_count: 0,
            disagreement_rate: 0.0,
            sample_declaration: "held-out ids=1-4".to_owned(),
            tighten_router: false,
        };
        assert_eq!(
            apply_grounding_calibration(GroundingMode::Constructive, ClaimType::Factual, &cal),
            GroundingMode::Constructive
        );
    }

    #[test]
    fn conservative_fallback_is_unchanged_by_tightening() {
        let cal = GroundingRouterCalibration {
            sample_size: 4,
            disagreement_count: 4,
            disagreement_rate: 1.0,
            sample_declaration: "held-out ids=1-4".to_owned(),
            tighten_router: true,
        };
        assert_eq!(
            apply_grounding_calibration(
                GroundingMode::ConservativeFallback,
                ClaimType::Factual,
                &cal
            ),
            GroundingMode::ConservativeFallback
        );
    }

    #[test]
    fn install_current_and_clear_round_trip() {
        clear_grounding_router_calibration();
        assert_eq!(current_grounding_calibration(), None);
        let cal = calibrate_grounding_router(&[], "source=unit-test");
        install_grounding_router_calibration(cal.clone());
        assert_eq!(current_grounding_calibration(), Some(cal));
        clear_grounding_router_calibration();
        assert_eq!(current_grounding_calibration(), None);
    }
}
