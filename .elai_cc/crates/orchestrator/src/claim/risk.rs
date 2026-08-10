//! Deterministic claim-risk convergence fuse (L-2).

use super::extract::{ClaimSet, ExtractedClaim};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RiskTier { Low, Medium, High, Critical }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PredictorKind {
    RefsUnresolved, RoundTrip, Oscillation, SemanticEntropy, Counterfactual, LogprobOptional,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PredictorSignal {
    pub source: PredictorKind,
    pub value: f64,
    pub fired: bool,
}

pub trait PredictorSource {
    fn signals(&self, claim: &ExtractedClaim) -> Vec<PredictorSignal>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClaimRisk {
    tier: RiskTier,
    pub signals: Vec<PredictorSignal>,
    pub consult: Vec<PredictorKind>,
    pub causal_supported: bool,
    pub claim_digest: [u8; 32],
}

impl ClaimRisk {
    pub fn tier(&self) -> RiskTier { self.tier }
}

// Seed boundaries are ELAI-measured at L-10 (ID-5), not paper constants.
const CRITICAL_MEAN: f64 = 0.90;

/// Return the expensive predictors worth consulting for a cheap tier. This is
/// deliberately only a set calculation; J-84 supplies the consultation loop.
pub fn expensive_predictors_for(tier: RiskTier) -> Vec<PredictorKind> {
    match tier {
        RiskTier::Low => Vec::new(),
        RiskTier::Medium => vec![PredictorKind::RoundTrip, PredictorKind::Oscillation],
        RiskTier::High | RiskTier::Critical => vec![
            PredictorKind::RoundTrip, PredictorKind::Oscillation,
            PredictorKind::SemanticEntropy, PredictorKind::Counterfactual,
        ],
    }
}

pub fn score_claim_risk(set: &ClaimSet) -> Vec<ClaimRisk> {
    score_claim_risk_with(set, &[])
}

pub fn score_claim_risk_with(
    set: &ClaimSet,
    sources: &[&dyn PredictorSource],
) -> Vec<ClaimRisk> {
    set.claims().iter().map(|claim| {
        let mut signals = Vec::new();
        let unresolved = claim.refs.iter().any(|reference| !reference.resolved);
        let cheap_signal = PredictorSignal {
            source: PredictorKind::RefsUnresolved,
            value: if unresolved { 1.0 } else { 0.0 },
            fired: unresolved,
        };
        // Consult is gated by the cheap tier before injected expensive signals affect the final tier.
        let cheap_tier = fuse(vec![cheap_signal.clone()], digest(claim)).tier();
        let consult = expensive_predictors_for(cheap_tier);
        signals.push(cheap_signal);
        for source in sources { signals.extend(source.signals(claim)); }
        let mut risk = fuse(signals, digest(claim));
        risk.consult = consult;
        risk
    }).collect()
}

fn fuse(signals: Vec<PredictorSignal>, claim_digest: [u8; 32]) -> ClaimRisk {
    let fired: Vec<&PredictorSignal> = signals.iter().filter(|signal| signal.fired).collect();
    let mut kinds = std::collections::HashSet::new();
    for signal in &fired { kinds.insert(signal.source); }
    let tier = match kinds.len() {
        0 => RiskTier::Low,
        1 => RiskTier::Medium,
        _ => {
            let mean = fired.iter().map(|signal| signal.value).sum::<f64>() / fired.len() as f64;
            if mean >= CRITICAL_MEAN { RiskTier::Critical } else { RiskTier::High }
        }
    };
    let causal_supported = fired.iter().any(|signal|
        signal.source == PredictorKind::Counterfactual);
    ClaimRisk { tier, signals, consult: Vec::new(), causal_supported, claim_digest }
}

fn digest(claim: &ExtractedClaim) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(claim.kind.as_str().as_bytes());
    hasher.update(&claim.span.start.to_le_bytes());
    hasher.update(&claim.span.end.to_le_bytes());
    for reference in &claim.refs { hasher.update(reference.raw.as_bytes()); }
    *hasher.finalize().as_bytes()
}
