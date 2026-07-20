//! I-21 — sprint contracts + ralph loop contract tests.
//!
//! Coverage (no #[ignore], no stubs, no placeholder asserts):
//!
//! (a) Contract validation: a well-formed contract approves; a vague /
//!     non-deterministic acceptance criterion is REJECTED with a concrete
//!     structured objection (the stub_ban-3 path).
//!
//! (b) RalphLoop cap: the bounded counter reinjects up to the cap then
//!     escalates, and never advances past the bound (stub_ban-2: bounded).
//!
//! (c) Acceptance check composes i-17: a real `TriLayerStack` verdict drives
//!     the accept/veto decision (GD-1 — the ralph trigger is a TriLayerStack
//!     verdict, not an ad-hoc check).
//!
//! (d) Adversary escalation composes D-13: `escalate_to_adversary` routes a
//!     real `MockAdversary` and returns the concrete counterexample text.
//!
//! (e) SSB round-trip (all 5 Kinds): each SprintContractEvent kind written to a
//!     real temp SQLite db, read back, decoded; field content verified
//!     (off-author artifact, wiring-enforcement R4).
//!
//! (f) Schema lock: SCHEMA_VERSION_PUBLIC >= 256 AND SSB_ACTION_TYPES.len() >=
//!     565 AND all 5 sql_str values round-trip via
//!     ssb_action_type_from_str_public.
//!
//! (g) Production dispatch SSB rows land: drives the PRODUCTION
//!     `task_runner::dispatch_sprint_contract` (via the `test-support`
//!     entrypoint) with `ELAI_SPRINT_CONTRACT_ENABLE=1` + a plumbed checker
//!     runtime, then queries the real DB and asserts the `Proposed` + `Approved`
//!     rows landed (off-author artifact; proves the new pub symbols are
//!     reachable from a real production entrypoint, R4/R7/D-030).
//!
//! Spec references: docs/thefinalglance/I-plan/I-21_sprint_contracts_ralph_loop.md
//! Reads (mandatory): wiring-enforcement.md R1/R4/R6/R7/R8; production-caller-gate.md.

use std::sync::{Arc, Mutex, OnceLock};

use elai_orchestrator::adversary::{
    AdversaryConfig, CounterEvidenceSchema, DynAdversary, AdversaryErased, ModelFamily,
    MockAdversary,
};
use elai_orchestrator::sprint_contract::{
    acceptance_check, escalate_to_adversary, sprint_contract_enabled, AcceptanceCriterion,
    ContractValidationError, RalphLoop, RalphStep, SealedContract, SprintContract,
    MAX_RALPH_REINJECTIONS,
};
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload,
    SprintContractEventKind, SprintContractEventPayload, SCHEMA_VERSION_PUBLIC, SSB_ACTION_TYPES,
};
use elai_orchestrator::verifier_stack::{Branch, TriLayerStack};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn good_contract() -> SprintContract {
    SprintContract {
        goal: "wire dispatch_sprint_contract into the post-task chain".into(),
        in_scope_files: vec!["crates/orchestrator/src/sprint_contract.rs".into()],
        acceptance_criteria: vec![
            AcceptanceCriterion::CompileStatus,
            AcceptanceCriterion::TestSymbol("i21_sprint_contract".into()),
        ],
        bounded_iteration_count: MAX_RALPH_REINJECTIONS,
        out_of_scope_marker: "no edits outside sprint_contract.rs".into(),
    }
}

fn open_ssb(dir: &tempfile::TempDir, session: &str) -> Arc<Ssb> {
    use elai_core::clock::OsClock;
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("i21-ssb.db"),
                pool_size: Some(2),
                project_id: "i21-contract".into(),
                session_id: session.into(),
            },
            clock,
        )
        .expect("ssb open"),
    )
}

// ---------------------------------------------------------------------------
// Env-var safety (CodeRabbit I-50): env vars are process-global and the
// default test harness runs tests in parallel. A static mutex serializes
// every test in this binary that reads or mutates
// ELAI_SPRINT_CONTRACT_ENABLE, and the RAII guard restores the prior value
// on drop (panic-safe). Mirrors the h42_serial_depth_budget_contract pattern.
// ---------------------------------------------------------------------------

fn sprint_contract_env_mutex() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct SprintContractEnvGuard {
    key: &'static str,
    prev: Option<String>,
}

impl SprintContractEnvGuard {
    fn set(key: &'static str, val: &str) -> Self {
        let prev = std::env::var(key).ok();
        // SAFETY: the caller holds sprint_contract_env_mutex() for the
        // duration of the mutation window, serializing all readers/writers
        // of this env var in this binary.
        unsafe {
            std::env::set_var(key, val);
        }
        Self { key, prev }
    }
}

impl Drop for SprintContractEnvGuard {
    fn drop(&mut self) {
        // SAFETY: same mutex scope; drop runs as the test unwinds, still
        // holding the lock via the `_mutex` binding in the caller.
        unsafe {
            match &self.prev {
                Some(v) => std::env::set_var(self.key, v),
                None => std::env::remove_var(self.key),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// (a) Contract validation
// ---------------------------------------------------------------------------

#[test]
fn i21_good_contract_approves() {
    let sealed = SealedContract::seal(good_contract(), "task: build i-21");
    assert!(sealed.evaluate().is_ok(), "well-formed contract must approve");
}

#[test]
fn i21_vague_criterion_rejected_with_concrete_objection() {
    let mut c = good_contract();
    // A whitespace-only test symbol is the "implementation feels complete"
    // analogue: not deterministically checkable.
    c.acceptance_criteria = vec![AcceptanceCriterion::TestSymbol("   ".into())];
    match c.validate() {
        Err(e @ ContractValidationError::NonDeterministicCriterion { .. }) => {
            let objection = e.objection();
            assert!(
                objection.contains("not deterministically checkable"),
                "objection must name the ambiguity; got: {objection}"
            );
        }
        other => panic!("expected NonDeterministicCriterion rejection; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (b) RalphLoop cap
// ---------------------------------------------------------------------------

#[test]
fn i21_ralph_loop_caps_then_escalates() {
    let c = good_contract(); // effective bound = MAX_RALPH_REINJECTIONS (3)
    let mut loop_ = RalphLoop::new(&c);
    assert_eq!(loop_.bound(), MAX_RALPH_REINJECTIONS);
    assert_eq!(loop_.on_failure(), RalphStep::Reinjected { iteration: 1 });
    assert_eq!(loop_.on_failure(), RalphStep::Reinjected { iteration: 2 });
    assert_eq!(
        loop_.on_failure(),
        RalphStep::Escalated {
            iteration: MAX_RALPH_REINJECTIONS
        }
    );
    // Never advances past the bound (stub_ban-2: bounded).
    assert_eq!(
        loop_.on_failure(),
        RalphStep::Escalated {
            iteration: MAX_RALPH_REINJECTIONS
        }
    );
    assert_eq!(loop_.iteration(), MAX_RALPH_REINJECTIONS);
}

#[test]
fn i21_iteration_bound_never_exceeds_hard_cap() {
    let mut c = good_contract();
    c.bounded_iteration_count = 99;
    assert_eq!(c.effective_iteration_bound(), MAX_RALPH_REINJECTIONS);
}

// ---------------------------------------------------------------------------
// (c) Acceptance check composes i-17 TriLayerStack
// ---------------------------------------------------------------------------

/// An empty-registry, empty-source branch is NOT vetoed by L1 (no checkers fire,
/// no dep imports), so `acceptance_check` returns Ok(true) — proving the i-17
/// compose path is the accept/veto authority (GD-1).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i21_acceptance_check_composes_tri_layer_stack() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let ssb = open_ssb(&dir, "i21-c");

    let dep_gate = Arc::new(
        elai_orchestrator::vibesec::dep_writetime_gate::DepWriteTimeGate::new(
            None,
            Arc::new(elai_orchestrator::vibesec::dep_writetime_gate::StubRegistryChecker),
        ),
    );
    let registry = Arc::new(elai_orchestrator::checker_registry::CheckerRegistry::new());
    let stack = TriLayerStack::new(
        registry,
        dep_gate,
        None,
        None,
        AdversaryConfig::default_for_local_worker(),
        Arc::clone(&ssb),
    );

    let branch = Branch::new("sprint-c", String::new());
    let accepted = acceptance_check(&stack, &branch)
        .await
        .expect("acceptance_check must not error on the no-source branch");
    assert!(
        accepted,
        "an empty-registry/empty-source branch is not vetoed; acceptance must pass"
    );
}

// ---------------------------------------------------------------------------
// (d) Adversary escalation composes D-13
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i21_adversary_escalation_returns_counterexample() {
    let contract = good_contract();
    let counter = CounterEvidenceSchema {
        counter_id: "i21-counter".into(),
        original_claim_id: format!("sprint-{}", contract.contract_id()),
        counter_text: "the goal is unbounded: no scope file is named".into(),
        counter_evidence: vec![],
        severity: 0.9,
    };
    let mock = MockAdversary::with_counter(ModelFamily::Anthropic, counter.clone());
    let adversary: Arc<dyn DynAdversary> = Arc::new(AdversaryErased::new(mock));
    let config = AdversaryConfig::default_for_local_worker();

    let text = escalate_to_adversary(&adversary, &config, &contract)
        .await
        .expect("escalate_to_adversary must succeed with a mock counter");
    assert_eq!(
        text, counter.counter_text,
        "escalation must surface the concrete D-13 counterexample text"
    );
}

// ---------------------------------------------------------------------------
// (e) SSB round-trip — all 5 Kinds
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i21_ssb_round_trip_all_five_kinds() {
    use elai_orchestrator::ssb::SsbReader;

    let dir = tempfile::TempDir::new().expect("tempdir");
    let ssb = open_ssb(&dir, "i21-e");
    let contract = good_contract();

    let kinds = [
        SprintContractEventKind::Proposed,
        SprintContractEventKind::Approved,
        SprintContractEventKind::Rejected,
        SprintContractEventKind::RalphReinjected,
        SprintContractEventKind::AdversaryEscalated,
    ];

    for (i, kind) in kinds.into_iter().enumerate() {
        let payload = SprintContractEventPayload {
            kind,
            contract_id: contract.contract_id(),
            goal: contract.goal.clone(),
            iteration: i as u32,
            objection: if kind == SprintContractEventKind::Rejected {
                "vague criterion".into()
            } else {
                String::new()
            },
            counterexample: if kind == SprintContractEventKind::AdversaryEscalated {
                "concrete counter".into()
            } else {
                String::new()
            },
            ts_ms: 1,
        };
        ssb.write_payload(
            "i21-agent",
            i as i64,
            SsbPayload::SprintContractEvent(payload),
            Vec::new(),
        )
        .await
        .expect("write_payload");
    }

    for kind in kinds {
        let rows = ssb
            .query_by_type(SsbActionType::SprintContractEvent(kind))
            .expect("query_by_type");
        assert!(
            !rows.is_empty(),
            "expected >=1 SprintContractEvent({kind:?}) row; found 0 (R1: missing evidence = FAIL)"
        );
        let decoded = decode_ssb_payload(&rows[0].payload).expect("decode_ssb_payload");
        match decoded {
            SsbPayload::SprintContractEvent(p) => {
                assert_eq!(p.kind, kind, "decoded kind must match the queried kind");
                assert_eq!(
                    p.contract_id,
                    contract.contract_id(),
                    "decoded contract_id must match"
                );
                if kind == SprintContractEventKind::Rejected {
                    assert_eq!(p.objection, "vague criterion", "objection must round-trip");
                }
                if kind == SprintContractEventKind::AdversaryEscalated {
                    assert_eq!(
                        p.counterexample, "concrete counter",
                        "counterexample must round-trip"
                    );
                }
            }
            other => panic!("expected SsbPayload::SprintContractEvent; got {other:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// (f) Schema lock
// ---------------------------------------------------------------------------

#[test]
fn i21_schema_version_floor_256() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 256,
        "SCHEMA_VERSION_PUBLIC must be >= 256 (I-21 floor: SprintContractEvent added); got {}",
        SCHEMA_VERSION_PUBLIC,
    );
}

#[test]
fn i21_total_variants_floor_565() {
    const I21_TV_FLOOR: usize = 565;
    let total = SSB_ACTION_TYPES.len();
    assert!(
        total >= I21_TV_FLOOR,
        "SSB_ACTION_TYPES.len() must be >= {I21_TV_FLOOR} (I-21 floor); got {total}",
    );
    let has_sprint = SSB_ACTION_TYPES
        .iter()
        .any(|t| matches!(t, SsbActionType::SprintContractEvent(_)));
    assert!(
        has_sprint,
        "SSB_ACTION_TYPES must include the SprintContractEvent representative"
    );
}

#[test]
fn i21_action_type_round_trips_all_five() {
    let arms = [
        ("sprint_contract_proposed", SprintContractEventKind::Proposed),
        ("sprint_contract_approved", SprintContractEventKind::Approved),
        ("sprint_contract_rejected", SprintContractEventKind::Rejected),
        (
            "sprint_contract_ralph_reinjected",
            SprintContractEventKind::RalphReinjected,
        ),
        (
            "sprint_contract_adversary_escalated",
            SprintContractEventKind::AdversaryEscalated,
        ),
    ];
    for (wire_str, expected_kind) in arms {
        let parsed = ssb_action_type_from_str_public(wire_str)
            .unwrap_or_else(|e| panic!("ssb_action_type_from_str_public({wire_str:?}) failed: {e}"));
        match parsed {
            SsbActionType::SprintContractEvent(kind) => {
                assert_eq!(kind, expected_kind, "round-trip kind mismatch for {wire_str:?}");
                assert_eq!(
                    kind.as_sql_str(),
                    wire_str,
                    "as_sql_str() must produce the original wire_str for {wire_str:?}"
                );
            }
            other => panic!("expected SprintContractEvent for {wire_str:?}; got {other:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// (g) Production dispatch SSB rows land (D-030 wire-dark fix proof)
// ---------------------------------------------------------------------------

/// Drives the PRODUCTION `task_runner::dispatch_sprint_contract` (via the
/// `test-support` entrypoint) with `ELAI_SPRINT_CONTRACT_ENABLE=1` and a plumbed
/// checker runtime, then asserts the `Proposed` + `Approved` rows landed in a
/// real temp SQLite db. This proves the new sprint_contract pub symbols are
/// reachable from a real production entrypoint (wiring-enforcement R4/R7;
/// production-caller-gate D-030).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i21_production_dispatch_ssb_row_lands() {
    use elai_orchestrator::ssb::SsbReader;

    // Serialize against any other test in this binary that reads or mutates
    // ELAI_SPRINT_CONTRACT_ENABLE (e.g. i21_gate_off_by_default). The mutex
    // is held for the whole test body; the RAII env guard restores the
    // previous value on drop, including on panic.
    let _mutex = sprint_contract_env_mutex()
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let dir = tempfile::TempDir::new().expect("tempdir");
    let ssb = open_ssb(&dir, "i21-g");

    let _env_guard = SprintContractEnvGuard::set("ELAI_SPRINT_CONTRACT_ENABLE", "1");
    assert!(
        sprint_contract_enabled(),
        "gate must read enabled after set_var"
    );

    // Drive the REAL production dispatch with the checker runtime plumbed.
    elai_orchestrator::task_runner::dispatch_sprint_contract_for_test(&ssb, 7, "i21-contract", true)
        .await;

    // _env_guard drops here, restoring the env var; _mutex releases after.

    // Off-author artifact (R4): the Proposed row must have landed.
    let proposed = ssb
        .query_by_type(SsbActionType::SprintContractEvent(
            SprintContractEventKind::Proposed,
        ))
        .expect("query_by_type Proposed");
    assert!(
        !proposed.is_empty(),
        "expected >=1 SprintContractEvent(Proposed) row from the production dispatch; found 0 \
         (R1: missing evidence = FAIL; proves the emit path is reachable from dispatch_sprint_contract)"
    );

    // The well-formed contract approves, so the Approved row must also land.
    let approved = ssb
        .query_by_type(SsbActionType::SprintContractEvent(
            SprintContractEventKind::Approved,
        ))
        .expect("query_by_type Approved");
    assert!(
        !approved.is_empty(),
        "expected >=1 SprintContractEvent(Approved) row from the production dispatch; found 0"
    );

    // Decode the Proposed row and verify the contract goal round-trips.
    let decoded = decode_ssb_payload(&proposed[0].payload).expect("decode");
    match decoded {
        SsbPayload::SprintContractEvent(p) => {
            assert_eq!(p.kind, SprintContractEventKind::Proposed);
            assert!(
                p.goal.contains("task-7"),
                "the production dispatch goal must reference the task; got: {}",
                p.goal
            );
        }
        other => panic!("expected SprintContractEvent; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (h) Default-OFF guard
// ---------------------------------------------------------------------------

#[test]
fn i21_gate_off_by_default() {
    // Hold the env mutex so i21_production_dispatch_ssb_row_lands cannot
    // mutate ELAI_SPRINT_CONTRACT_ENABLE while this test reads it.
    let _mutex = sprint_contract_env_mutex()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let expected = std::env::var("ELAI_SPRINT_CONTRACT_ENABLE")
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false);
    assert_eq!(sprint_contract_enabled(), expected);
}
