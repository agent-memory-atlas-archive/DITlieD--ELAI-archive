//! J-95 LEG C contract: I-7 all-fail waterfall must journal BeliefState condensation.
//!
//! Drives public `EscalationWaterfall::run` (same entry task_runner uses) with an
//! always-veto fixture. Fails if condense or BeliefStateEvent(Condensed) wire is deleted.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::adversary::AdversaryConfig;
use elai_orchestrator::checker_registry::{CheckerRegistry, RegexMatchChecker};
use elai_orchestrator::cloud_mode::{CloudCredentials, RoutedProvider};
use elai_orchestrator::escalation_waterfall::{default_escalation_router, EscalationWaterfall};
use elai_orchestrator::mutation_budget::MutationBudget;
use elai_orchestrator::context_budget::{
    j95_pristine_on_budget_enabled, observe_pristine_slot_candidates, ContextBudget,
    ELAI_J95_PRISTINE_ON_BUDGET_VAR,
};
use elai_orchestrator::dispatcher::BackendMode;
use elai_orchestrator::pristine_path_filter::PristinePathFilter;
use elai_orchestrator::search::{BranchInput, SearchLadder};
use elai_orchestrator::ssb::{
    decode_ssb_payload, BeliefStateEventKind, EscalationCascadeKind, Ssb, SsbActionType, SsbConfig,
    SsbPayload,
};
use elai_orchestrator::verifier_stack::{Branch, FlakeQuarantineLedger, TriLayerStack};
use elai_orchestrator::vibesec::dep_writetime_gate::{DepWriteTimeGate, StubRegistryChecker};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

const KILL_RATE_FLOOR: f64 = 0.5;

fn test_clock() -> Arc<dyn ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir) -> Arc<Ssb> {
    Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("ssb.db"),
                pool_size: Some(2),
                project_id: "j95-contract".into(),
                session_id: "j95-c-escalation-condense".into(),
            },
            test_clock(),
        )
        .expect("ssb init"),
    )
}

fn always_veto_registry() -> Arc<CheckerRegistry> {
    let mut registry = CheckerRegistry::new();
    let checker = RegexMatchChecker::new("ZZZ_J95_NEVER_PRESENT_SENTINEL_ZZZ", "j95-veto", false)
        .expect("regex compiles");
    registry.register("j95-always-veto", Box::new(checker));
    Arc::new(registry)
}

fn build_all_fail_waterfall(ssb: &Arc<Ssb>) -> EscalationWaterfall {
    let stack = Arc::new(TriLayerStack::new(
        always_veto_registry(),
        Arc::new(DepWriteTimeGate::new(None, Arc::new(StubRegistryChecker))),
        None,
        None,
        AdversaryConfig::default_for_local_worker(),
        Arc::clone(ssb),
    ));
    let pristine = Arc::new(PristinePathFilter::new(Arc::clone(ssb)));
    let quarantine = Arc::new(FlakeQuarantineLedger::new(Arc::clone(ssb)));
    let mutation_budget = MutationBudget::with_threshold(KILL_RATE_FLOOR);
    let ladder = Arc::new(SearchLadder::new(
        stack,
        pristine,
        quarantine,
        mutation_budget,
        Arc::clone(ssb),
    ));
    EscalationWaterfall::new(ladder, default_escalation_router(), Arc::clone(ssb))
}

fn cascade_rows(ssb: &Arc<Ssb>, kind: EscalationCascadeKind) -> Vec<elai_orchestrator::ssb::SsbEntry> {
    ssb.query_by_type(SsbActionType::EscalationCascade(kind))
        .expect("query_by_type must succeed")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn j95_c_escalation_condense_wire_present() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir);
    let waterfall = build_all_fail_waterfall(&ssb);
    let candidates = vec![
        BranchInput::from_branch(Branch::new("j95-fail-a", "let a = 1;")),
        BranchInput::from_branch(Branch::new("j95-fail-b", "let b = 2;")),
    ];
    let creds = Some(CloudCredentials::for_test_with_token("j95-fixture-token"));

    let outcome = waterfall
        .run(
            &candidates,
            "j95-c-session",
            vec!["j95 deterministic veto".to_string()],
            creds,
        )
        .await
        .expect("all-fail waterfall run must succeed");

    assert!(!outcome.local_win, "the vetoed fixture must take the escalation path");
    match outcome.routed {
        Some(RoutedProvider::Cloud { .. }) => {}
        other => panic!("all-fail must route to the cloud lane, got {other:?}"),
    }
    assert!(
        outcome.condensed_bytes > 0,
        "the all-fail escalation must carry a non-empty BeliefState wire"
    );

    assert_eq!(
        cascade_rows(&ssb, EscalationCascadeKind::Condensed).len(),
        1,
        "exactly one EscalationCascade(Condensed) row"
    );

    let belief_rows = ssb
        .query_by_type(SsbActionType::BeliefStateEvent(BeliefStateEventKind::Condensed))
        .expect("query BeliefStateEvent(Condensed)");
    assert!(
        !belief_rows.is_empty(),
        "the production condense path must emit a typed Condensed receipt"
    );
    match decode_ssb_payload(&belief_rows[0].payload).expect("decode condensed receipt") {
        SsbPayload::BeliefStateEvent(payload) => {
            assert_eq!(
                payload.kind,
                BeliefStateEventKind::Condensed,
                "the receipt must retain the Condensed discriminator"
            );
        }
        other => panic!(
            "expected BeliefStateEvent payload, got {:?}",
            other.action_type()
        ),
    }
}


#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn j95_b_pristine_budget_observe_wire_present() {
    // Default-OFF: gate reads env, unset => false. No env mutation (MR-2).
    // If a concurrent test polluted the var, skip asserts that depend on it.
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir);
    let filter = PristinePathFilter::new(Arc::clone(&ssb));
    let traceback = vec![
        b'x';
        elai_orchestrator::pristine_path_filter::TRACEBACK_COMPRESS_THRESHOLD_BYTES,
    ];
    let mut budget = ContextBudget::compute(8192, [0, 0, 0], 512).expect("budget");
    let outcome = observe_pristine_slot_candidates(
        &filter,
        &mut budget,
        "j95-b",
        "j95-b-session",
        Some(&traceback),
        BackendMode::Local,
    )
    .await
    .expect("observe must succeed");
    if !j95_pristine_on_budget_enabled() {
        assert_eq!(
            outcome.original_traceback_bytes, 0,
            "default-OFF must not run traceback contrast"
        );
        assert!(outcome.constraint_phrase.is_none());
    }
    // Symbol reachability: helpers are public and callable (production_caller
    // separately proves task_runner WIRED for observe_pristine_slot_candidates).
    let _ = ELAI_J95_PRISTINE_ON_BUDGET_VAR;
}
