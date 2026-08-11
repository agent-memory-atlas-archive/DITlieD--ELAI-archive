use elai_orchestrator::claim::extract::{ByteSpan, ClaimKind, ClaimRef, ClaimSet, ExtractedClaim};
use elai_orchestrator::claim::risk::{PredictorKind, RiskTier};
use elai_orchestrator::claim::roundtrip::score_claim_risk_wired;

fn set(resolved: &[bool]) -> ClaimSet {
    ClaimSet::from_claims(vec![ExtractedClaim {
        kind: ClaimKind::Behavior,
        span: ByteSpan { start: 3, end: 9 },
        refs: resolved
            .iter()
            .enumerate()
            .map(|(index, &resolved)| ClaimRef {
                raw: format!("src/ref-{index}.rs"),
                resolved,
            })
            .collect(),
    }])
}

fn signal_fired(risk: &elai_orchestrator::claim::risk::ClaimRisk, source: PredictorKind) -> bool {
    risk.signals.iter().any(|signal| signal.source == source && signal.fired)
}

#[test]
fn all_resolved_is_low_and_roundtrip_does_not_fire() {
    let risk = &score_claim_risk_wired(&set(&[true, true]), "")[0];
    assert_eq!(risk.tier(), RiskTier::Low);
    assert!(!signal_fired(risk, PredictorKind::RoundTrip));
}

#[test]
fn all_unresolved_fires_both_and_moves_tier() {
    let risk = &score_claim_risk_wired(&set(&[false, false, false, false]), "")[0];
    assert!(signal_fired(risk, PredictorKind::RefsUnresolved));
    assert!(signal_fired(risk, PredictorKind::RoundTrip));
    assert!(matches!(risk.tier(), RiskTier::High | RiskTier::Critical));
}

#[test]
fn graded_residual_below_threshold_stays_medium() {
    let risk = &score_claim_risk_wired(&set(&[false, true, true, true]), "")[0];
    assert!(signal_fired(risk, PredictorKind::RefsUnresolved));
    assert!(!signal_fired(risk, PredictorKind::RoundTrip));
    assert_eq!(risk.tier(), RiskTier::Medium);
}

#[test]
fn refless_claim_returns_a_tier() {
    let risks = score_claim_risk_wired(&set(&[]), "");
    assert_eq!(risks.len(), 1);
    assert_eq!(risks[0].tier(), RiskTier::Low);
}


#[test]
fn outside_closed_set_citation_fires_roundtrip() {
    let text = "Behavior passes [2].";
    let claims = elai_orchestrator::claim::extract::extract_claims(text);
    let risks = score_claim_risk_wired(&claims, text);
    assert!(risks.iter().any(|risk| signal_fired(risk, PredictorKind::RoundTrip)));
    assert!(risks.iter().any(|risk| matches!(risk.tier(), RiskTier::Medium | RiskTier::High | RiskTier::Critical)));
}

#[test]
fn inside_closed_set_citation_stays_low() {
    let text = "Behavior passes [1].";
    let claims = ClaimSet::from_claims(vec![ExtractedClaim {
        kind: ClaimKind::Behavior,
        span: ByteSpan { start: 0, end: text.len() },
        refs: vec![ClaimRef { raw: "resolved".into(), resolved: true }],
    }]);
    let risks = score_claim_risk_wired(&claims, text);
    assert_eq!(risks.len(), 1);
    assert_eq!(risks[0].tier(), RiskTier::Low);
    assert!(!signal_fired(&risks[0], PredictorKind::RoundTrip));
}

#[test]
fn resolved_claim_without_citation_has_only_neutral_roundtrip() {
    let text = "Behavior passes.";
    let claims = ClaimSet::from_claims(vec![ExtractedClaim {
        kind: ClaimKind::Behavior,
        span: ByteSpan { start: 0, end: text.len() },
        refs: vec![ClaimRef { raw: "resolved".into(), resolved: true }],
    }]);
    let risks = score_claim_risk_wired(&claims, text);
    assert_eq!(risks[0].tier(), RiskTier::Low);
    assert_eq!(risks[0].signals.iter().filter(|s| s.source == PredictorKind::RoundTrip).count(), 1);
    assert!(!signal_fired(&risks[0], PredictorKind::RoundTrip));
}
