//! L-8 quarantine projection contract tests.
//! The second production populate site, the I-11 arm in `task_runner.rs`, is NOT
//! covered here: reaching it needs a real decomposition turn, so reverting that
//! site alone is an uncovered mutation.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::belief_state::{AtomKind, BeliefState, CriticalAtom, TriedApproach};
use elai_orchestrator::checker_registry::{CheckerRegistry, RegexMatchChecker};
use elai_orchestrator::cloud_mode::CloudCredentials;
use elai_orchestrator::escalation_waterfall::{default_escalation_router, EscalationWaterfall};
use elai_orchestrator::claim::extract::extract_claims;
use elai_orchestrator::claim::prune::{claim_fact_id, content_key};
use elai_orchestrator::handoff_faithfulness::{enforce_handoff_faithfulness, projected_context_from_belief};
use elai_orchestrator::mutation_budget::MutationBudget;
use elai_orchestrator::pristine_path_filter::PristinePathFilter;
use elai_orchestrator::search::{BranchInput, SearchLadder};
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbPayload, StalenessQuarantineBroadcastPayload};
use elai_orchestrator::verifier_stack::{Branch, FlakeQuarantineLedger, TriLayerStack};
use elai_orchestrator::vibesec::dep_writetime_gate::{DepWriteTimeGate, StubRegistryChecker};
use elai_orchestrator::adversary::AdversaryConfig;
use tempfile::TempDir;

fn clock() -> Arc<dyn ClockProvider> { Arc::new(OsClock) }
fn ssb(dir: &TempDir, session: &str) -> Arc<Ssb> {
    Arc::new(Ssb::new(SsbConfig { db_path: dir.path().join("ssb.db"), pool_size: Some(2), project_id: "l8-contract".into(), session_id: session.into() }, clock()).expect("ssb"))
}
fn waterfall(ssb: &Arc<Ssb>) -> EscalationWaterfall {
    let mut registry = CheckerRegistry::new();
    registry.register("always-veto", Box::new(RegexMatchChecker::new("ZZZ_NEVER_PRESENT_ZZZ", "veto", false).expect("regex")));
    let ssb2 = Arc::clone(ssb);
    let stack = Arc::new(TriLayerStack::new(Arc::new(registry), Arc::new(DepWriteTimeGate::new(None, Arc::new(StubRegistryChecker))), None, None, AdversaryConfig::default_for_local_worker(), Arc::clone(ssb)));
    let ladder = Arc::new(SearchLadder::new(stack, Arc::new(PristinePathFilter::new(Arc::clone(ssb))), Arc::new(FlakeQuarantineLedger::new(Arc::clone(ssb))), MutationBudget::with_threshold(0.5), ssb2));
    EscalationWaterfall::new(ladder, default_escalation_router(), Arc::clone(ssb))
}
fn belief(reasons: Vec<String>) -> BeliefState {
    BeliefState::new(vec![TriedApproach { label: "attempt".into(), token_cost: 0 }], reasons.clone(), Vec::new(), reasons, Vec::new(), 0)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn b1_real_db_quarantine_populates_condensate() {
    let candidates = vec![BranchInput::from_branch(Branch::new("fail", "let x = 1;"))];
    let d1 = TempDir::new().expect("tempdir");
    let s1 = ssb(&d1, "l8-b1");
    let a = waterfall(&s1).run(&candidates, "l8-b1", vec!["reason".into()], Some(CloudCredentials::for_test_with_token("token"))).await.expect("run");
    let d2 = TempDir::new().expect("tempdir");
    let s2 = ssb(&d2, "l8-b1");
    s2.write_payload("slice-a", 0, SsbPayload::StalenessQuarantineBroadcast(StalenessQuarantineBroadcastPayload { fact_id: "claim:quarantined".into(), staleness_ratio: 1.0, dependents_demoted: 0, cosmos_artifacts_flagged: 0, tick_ms: 1, broadcast_id: "broadcast".into(), dropped_count: 0 }), Vec::new()).await.expect("seed");
    let b = waterfall(&s2).run(&candidates, "l8-b1", vec!["reason".into()], Some(CloudCredentials::for_test_with_token("token"))).await.expect("run");
    // WaterfallOutcome has no belief and Condensed SSB payload_bytes is zero; this byte delta is the production-path observable.
    assert!(b.condensed_bytes > a.condensed_bytes);
    assert!(!b.local_win && b.routed.is_some());
}

#[test]
fn b2_consult_excludes_quarantined_part() {
    let quarantined_reason = "dep-gate veto at `src/l8_absent_thing.rs`";
    let claims = extract_claims(quarantined_reason);
    assert!(!claims.is_empty());
    let fact_id = claim_fact_id(&content_key(&claims.claims()[0]));
    let b = belief(vec![quarantined_reason.into(), "retained reason".into()])
        .with_quarantined_claims(vec![fact_id]);
    let p = projected_context_from_belief(&b);
    assert!(!p.contains(quarantined_reason));
    assert!(p.contains("retained reason"));
}

#[test]
fn b3_consult_preserve_wins() {
    let quarantined_reason = "dep-gate veto at `src/l8_absent_thing.rs`";
    let claims = extract_claims(quarantined_reason);
    assert!(!claims.is_empty());
    let fact_id = claim_fact_id(&content_key(&claims.claims()[0]));
    let b = belief(vec![quarantined_reason.into(), "retained reason".into()])
        .with_quarantined_claims(vec![fact_id])
        .with_critical_atoms(vec![CriticalAtom { kind: AtomKind::BugLocation, value: quarantined_reason.into(), must_preserve: true }]);
    let p = projected_context_from_belief(&b);
    assert!(p.contains(quarantined_reason));
    assert!(enforce_handoff_faithfulness(&b, &p).is_ok());
}

#[test]
fn b4_old_payload_defaults_quarantine_field() {
    #[derive(serde::Serialize)]
    struct Old<'a> { version: u32, tried: &'a Vec<TriedApproach>, failed_because: &'a Vec<String>, current_ast_state: &'a Vec<u8>, constraints: &'a Vec<String>, dkg_entities: &'a Vec<i64>, token_delta: u32, critical_atoms: &'a Vec<CriticalAtom> }
    let b = belief(vec!["reason".into()]);
    let bytes = rmp_serde::to_vec_named(&Old { version: 1, tried: &b.tried, failed_because: &b.failed_because, current_ast_state: &b.current_ast_state, constraints: &b.constraints, dkg_entities: &b.dkg_entities, token_delta: b.token_delta, critical_atoms: &b.critical_atoms }).expect("encode old");
    let decoded = BeliefState::decode(&bytes).expect("old payload decodes");
    assert!(decoded.quarantined_claims.is_empty());
}
