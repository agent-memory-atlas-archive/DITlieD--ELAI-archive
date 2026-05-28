//! E-20 task 2 -- HybridEscalationGate contract test.
//!
//! Asserts the calibrated escalation gate behaves correctly per
//! per-L-layer threshold config. Replaces the C-5
//! `retry_cap == 1 && reason == "L3_failed"` escalation logic with the
//! confidence-threshold cascade from arXiv:2511.07396 C3PO +
//! arXiv:2310.12963 AutoMix.
//!
//! Paper anchor: arXiv:2511.07396 (conformal-cost-bounded cascade) +
//! arXiv:2310.12963 (AutoMix POMDP self-verify; >50% cost reduction
//! at iso-quality).

use elai_orchestrator::cost::cloud_route::{
    EscalationReason, HybridEscalationGate, HybridEscalationVerdict, LLayerThresholds,
    L_HYBRID_THRESHOLD_DEFAULT,
};

#[test]
fn default_threshold_matches_plan_value() {
    // Plan task 2: "Initial threshold 0.55". Documented in
    // workflow/reasoning/E-20-decisions.md DECISION-5.
    assert!((L_HYBRID_THRESHOLD_DEFAULT - 0.55).abs() < 1e-6);
}

#[test]
fn high_confidence_keeps_local() {
    let gate = HybridEscalationGate::default();
    let (verdict, reason) = gate.evaluate(0.9, 3, false);
    assert_eq!(verdict, HybridEscalationVerdict::SkipCloud);
    assert_eq!(reason, EscalationReason::Confidence);
}

#[test]
fn low_confidence_escalates_cloud() {
    let gate = HybridEscalationGate::default();
    let (verdict, _) = gate.evaluate(0.4, 3, false);
    assert_eq!(verdict, HybridEscalationVerdict::EscalateCloud);
}

#[test]
fn very_low_confidence_critical_escalates_opus() {
    let gate = HybridEscalationGate::default();
    let (verdict, reason) = gate.evaluate(0.05, 5, true);
    assert_eq!(verdict, HybridEscalationVerdict::EscalateOpus);
    assert_eq!(reason, EscalationReason::Confidence);
}

#[test]
fn per_layer_threshold_override_respected() {
    let mut thresholds = LLayerThresholds::default();
    thresholds.per_layer[2] = Some(0.8); // L3 stricter
    let gate = HybridEscalationGate {
        thresholds,
        ..HybridEscalationGate::default()
    };
    // Score 0.7 is above L1/L2/L4/L5 default 0.55 but below L3 override.
    let (verdict_l1, _) = gate.evaluate(0.7, 1, false);
    let (verdict_l3, _) = gate.evaluate(0.7, 3, false);
    assert_eq!(verdict_l1, HybridEscalationVerdict::SkipCloud);
    assert_eq!(verdict_l3, HybridEscalationVerdict::EscalateCloud);
}

#[test]
fn retry_exhausted_path_returns_cloud_with_reason() {
    let gate = HybridEscalationGate::default();
    let (verdict, reason) = gate.retry_exhausted_path();
    assert_eq!(verdict, HybridEscalationVerdict::EscalateCloud);
    assert_eq!(reason, EscalationReason::RetryExhausted);
}

#[test]
fn privacy_block_path_keeps_local_with_reason() {
    let gate = HybridEscalationGate::default();
    let (verdict, reason) = gate.privacy_block_path();
    // Privacy block KEEPS execution local (refuses to dispatch).
    assert_eq!(verdict, HybridEscalationVerdict::SkipCloud);
    assert_eq!(reason, EscalationReason::PrivacyBlock);
}

#[test]
fn opus_threshold_below_default() {
    let gate = HybridEscalationGate::default();
    assert!(gate.opus_threshold < L_HYBRID_THRESHOLD_DEFAULT);
}
