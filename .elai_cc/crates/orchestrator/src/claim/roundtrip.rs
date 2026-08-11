//! Graded evidence round-trip residual predictor (L-3).

use super::extract::{ClaimSet, ExtractedClaim};
use super::risk::{
    score_claim_risk_with, ClaimRisk, PredictorKind, PredictorSignal, PredictorSource,
};

/// ELAI-declared seed boundary, not a published constant.
const DEFAULT_RESIDUAL_FIRES_AT: f64 = 0.5;

/// Predictor for the fraction of evidence references that failed to round-trip.
#[derive(Debug, Clone, Copy)]
pub struct RoundTripPredictor {
    residual_fires_at: f64,
}

impl Default for RoundTripPredictor {
    fn default() -> Self {
        Self::new(DEFAULT_RESIDUAL_FIRES_AT)
    }
}

impl RoundTripPredictor {
    pub fn new(residual_fires_at: f64) -> Self {
        Self { residual_fires_at }
    }

    pub fn roundtrip_residual(claim: &ExtractedClaim) -> f64 {
        if claim.refs.is_empty() {
            return 0.0;
        }
        let unresolved_refs = claim.refs.iter().filter(|reference| !reference.resolved).count();
        unresolved_refs as f64 / claim.refs.len() as f64
    }

}

/// Production entry point for claim risk scoring with the round-trip signal.
pub fn score_claim_risk_wired(set: &ClaimSet) -> Vec<ClaimRisk> {
    let predictor = RoundTripPredictor::default();
    score_claim_risk_with(set, &[&predictor])
}

impl PredictorSource for RoundTripPredictor {
    fn signals(&self, claim: &ExtractedClaim) -> Vec<PredictorSignal> {
        let residual = Self::roundtrip_residual(claim);
        vec![PredictorSignal {
            source: PredictorKind::RoundTrip,
            value: residual,
            fired: residual >= self.residual_fires_at,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::extract::{ByteSpan, ClaimKind, ClaimRef};

    fn claim(refs: Vec<bool>) -> ExtractedClaim {
        ExtractedClaim {
            kind: ClaimKind::Behavior,
            span: ByteSpan { start: 0, end: 1 },
            refs: refs
                .into_iter()
                .enumerate()
                .map(|(index, resolved)| ClaimRef {
                    raw: format!("ref-{index}"),
                    resolved,
                })
                .collect(),
        }
    }

    #[test]
    fn fully_resolved_claim_has_zero_residual_and_does_not_fire() {
        let claim = claim(vec![true, true]);
        assert_eq!(RoundTripPredictor::roundtrip_residual(&claim), 0.0);
        assert!(!RoundTripPredictor::default().signals(&claim)[0].fired);
    }

    #[test]
    fn all_unresolved_claim_has_unit_residual_and_fires() {
        let claim = claim(vec![false, false]);
        assert_eq!(RoundTripPredictor::roundtrip_residual(&claim), 1.0);
        assert!(RoundTripPredictor::default().signals(&claim)[0].fired);
    }

    #[test]
    fn one_of_four_unresolved_has_quarter_residual_and_does_not_fire() {
        let claim = claim(vec![false, true, true, true]);
        assert_eq!(RoundTripPredictor::roundtrip_residual(&claim), 0.25);
        assert!(!RoundTripPredictor::default().signals(&claim)[0].fired);
    }

    #[test]
    fn refless_claim_has_zero_residual_without_panicking() {
        let claim = claim(Vec::new());
        assert_eq!(RoundTripPredictor::roundtrip_residual(&claim), 0.0);
        assert!(!RoundTripPredictor::default().signals(&claim)[0].fired);
    }
}
