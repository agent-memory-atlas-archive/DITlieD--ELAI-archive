use elai_orchestrator::claim::counterfactual::CounterfactualPredictor;
use elai_orchestrator::claim::extract::{ByteSpan, ClaimKind, ClaimRef, ExtractedClaim};
use elai_orchestrator::claim::risk::{PredictorKind, PredictorSource};
use elai_orchestrator::evidence::{
    AntiFabricationGate, EvidenceGate, IntegrationTestRowSummary, TransitionContext,
};

fn claim(kind: ClaimKind, raw: &str, resolved: bool) -> ExtractedClaim {
    ExtractedClaim {
        kind,
        span: ByteSpan { start: 0, end: 1 },
        refs: vec![ClaimRef { raw: raw.into(), resolved }],
    }
}

fn fired(predictor: &CounterfactualPredictor, claim: &ExtractedClaim) -> bool {
    predictor
        .signals(claim)
        .iter()
        .any(|signal| signal.source == PredictorKind::Counterfactual && signal.fired)
}

#[test]
fn production_predictor_is_present_and_non_causal_without_live_verdict() {
    let predictor = CounterfactualPredictor::new();
    assert!(!fired(&predictor, &claim(ClaimKind::Causal, "src/fix.rs", true)));
    assert!(!fired(&predictor, &claim(ClaimKind::Dependency, "dep-b", true)));
    assert!(!fired(&predictor, &claim(ClaimKind::File, "README.md", true)));
}

#[test]
fn predictor_emits_counterfactual_signal_through_public_api() {
    let predictor = CounterfactualPredictor::new();
    let signals = predictor.signals(&claim(ClaimKind::Behavior, "src/fix.rs", true));
    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0].source, PredictorKind::Counterfactual);
    assert!(!signals[0].fired);
}

#[test]
fn fabricated_pass_is_rejected() {
    let mut ctx = TransitionContext::default();
    ctx.integration_test_rows = Some(vec![IntegrationTestRowSummary {
        test_name: "x".into(),
        passed: true,
        stdout_hash: None,
    }]);
    assert!(AntiFabricationGate.check(&ctx).is_err());
}
