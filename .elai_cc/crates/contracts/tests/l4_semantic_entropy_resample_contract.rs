//! Cross-crate contract for consult-gated semantic-entropy resampling.
//!
//! Mutation not covered: the flag-ON path through the whole
//! `AgentPoolBabyStepExecutor::execute`, which needs agent answer text crafted to
//! score High through the real workspace resolver. That path is deferred, not covered.

use elai_orchestrator::claim::extract::{ByteSpan, ClaimKind, ClaimRef, ClaimSet, ExtractedClaim};
use elai_orchestrator::claim::risk::RiskTier;
use elai_orchestrator::claim::roundtrip::score_claim_risk_wired;
use elai_orchestrator::claim::semantic_entropy::{collect_extra_samples, resample_is_indicated};
use elai_orchestrator::ipc_accept::test_support::ScriptedAgentRpcServer;
use std::sync::atomic::Ordering;

fn risks(resolved: bool) -> Vec<elai_orchestrator::claim::risk::ClaimRisk> {
    let set = ClaimSet::from_claims(vec![ExtractedClaim {
        kind: ClaimKind::Behavior,
        span: ByteSpan { start: 0, end: 1 },
        refs: vec![ClaimRef { raw: "src/nope.rs".into(), resolved }],
    }]);
    score_claim_risk_wired(&set, "")
}

#[test]
fn resample_is_indicated_only_above_medium() {
    let severe = risks(false);
    let cheap = risks(true);
    assert!(severe.iter().any(|risk| risk.tier() == RiskTier::Critical));
    assert!(cheap.iter().all(|risk| risk.tier() == RiskTier::Low));
    assert!(resample_is_indicated(&severe));
    assert!(!resample_is_indicated(&cheap));
}

#[tokio::test]
async fn disabled_gate_never_touches_inference() {
    let severe = risks(false);
    let scripted = ScriptedAgentRpcServer::new("answer");
    assert!(collect_extra_samples(false, &severe, &scripted, "p").await.is_empty());
    assert_eq!(scripted.infer_calls.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn armed_gate_collects_the_bounded_extra_samples() {
    let severe = risks(false);
    let scripted = ScriptedAgentRpcServer::new("answer");
    assert_eq!(collect_extra_samples(true, &severe, &scripted, "p").await.len(), 2);
    assert_eq!(scripted.infer_calls.load(Ordering::Relaxed), 2);
}
