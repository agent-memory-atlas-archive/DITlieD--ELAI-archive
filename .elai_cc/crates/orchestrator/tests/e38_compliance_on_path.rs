//! E-38 on-path smoke -- drives the REAL `Fsm::transition` entrypoint
//! with a `DefaultComplianceProvider` installed and the
//! `ELAI_EU_AI_ACT_COMPLIANCE_GATE_ENABLE` opt-in set, then asserts:
//!
//!   * the PLANNING -> TESTING_FIRST transition is refused with
//!     `FsmError::ComplianceBlocked { assertion_label: "deterministic_seed" }`
//!     (the gate fails on `DeterministicSeed` because
//!     `ELAI_DETERMINISTIC_SEED` is not set in the test env);
//!   * an `AiRoadmapSignalEvent` SSB row with
//!     `kind = ComplianceBlocked` + `assertion_label = "deterministic_seed"`
//!     lands in the SSB (the off-agent artifact the executor did NOT
//!     author -- wiring-enforcement R4);
//!   * the FSM remains in PLANNING (rejection happens before state
//!     mutation);
//!   * a second transition with the gate opted OUT (env unset) proceeds
//!     and emits a `ComplianceVerified` row (default-OFF path is not
//!     bricked).
//!
//! Deterministic only; no GPU / network / live model. Mirrors the
//! `canary_observable_smoke.rs` SsbConfig + SeededClock + SsbViewer
//! pattern.

use std::sync::Arc;

use elai_orchestrator::compliance::{
    DefaultComplianceProvider, ELAI_DETERMINISTIC_SEED_ENV,
};
use elai_orchestrator::fsm::{Fsm, FsmError, FsmState};
use elai_orchestrator::ssb::{
    RoadmapSignalKind, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
};
use elai_orchestrator::ssb_viewer::SsbViewer;

/// Helper: decode the first `AiRoadmapSignalEvent` row in the SSB and
/// return its `(kind, assertion_label)` pair. Panics on decode failure
/// (the test is asserting the row landed, so a decode error is a test
/// failure, not a skip).
fn first_signal_row(
    ssb: &Ssb,
) -> (RoadmapSignalKind, String) {
    let rows = ssb
        .query_by_type(SsbActionType::AiRoadmapSignalEvent)
        .expect("query_by_type AiRoadmapSignalEvent");
    assert!(
        !rows.is_empty(),
        "expected at least one AiRoadmapSignalEvent row in the SSB"
    );
    let payload: SsbPayload = rmp_serde::from_slice(&rows[0].payload)
        .expect("decode AiRoadmapSignalEvent payload");
    match payload {
        SsbPayload::AiRoadmapSignalEvent(p) => (p.kind, p.assertion_label),
        other => panic!("expected AiRoadmapSignalEvent arm, got {other:?}"),
    }
}

/// Helper: count `PhaseTransition` rows via the read-only viewer (the
/// same path `DefaultComplianceProvider::ssb_row_count_in_planning_phase`
/// uses at verdict time).
fn phase_transition_count(viewer: &SsbViewer) -> usize {
    viewer
        .recent_rows_of_type(SsbActionType::PhaseTransition.as_sql_str(), 1024)
        .expect("recent_rows_of_type PhaseTransition")
        .len()
}

/// Test 1: gate opted IN, no `ELAI_DETERMINISTIC_SEED` -> the
/// PLANNING -> TESTING_FIRST transition is refused with
/// `ComplianceBlocked { assertion_label: "deterministic_seed" }` and a
/// `ComplianceBlocked` SSB row lands. The FSM stays in PLANNING.
///
/// Race-free: uses `with_gate_enabled(true)` so the provider does NOT
/// probe `ELAI_EU_AI_ACT_COMPLIANCE_GATE_ENABLE` at verdict time. The
/// two on-path tests in this binary are both `#[tokio::test]` and cargo
/// runs them in parallel; a process-wide `set_var`/`remove_var` pair
/// races between Test 1's opt-in and Test 2's opt-out, so Test 1's
/// provider could see the gate as unset and incorrectly short-circuit
/// to `Pass`. The explicit `force_enabled` flag bypasses the env probe
/// and makes each test independent of the other's env mutations.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn gate_opted_in_blocks_on_missing_deterministic_seed() {
    // `ELAI_DETERMINISTIC_SEED` is read by `build_plan_context` at
    // verdict time; this is a per-test assertion that the seed is NOT
    // set, so remove it defensively. The gate ENABLE flag is NOT
    // probed here -- `with_gate_enabled(true)` bypasses it.
    std::env::remove_var(ELAI_DETERMINISTIC_SEED_ENV);
    // SSB emission leg is still env-gated by `ELAI_AI_ROADMAP_SIGNALS_ENABLE`.
    // This is a separate concern from the gate opt-in and does not race
    // with Test 2 (both tests set it to "1", never remove it).
    std::env::set_var("ELAI_AI_ROADMAP_SIGNALS_ENABLE", "1");

    let dir = tempfile::TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb_e38_on_path.db"),
        pool_size: Some(2),
        project_id: "test-project".into(),
        session_id: "s-e38-on-path".into(),
    };
    let clock: Arc<dyn elai_core::ClockProvider> =
        Arc::new(elai_core::clock::SeededClock::new());
    let ssb = Arc::new(Ssb::new(cfg.clone(), clock).expect("ssb init"));

    let fsm = Fsm::new(None, 8000).with_ssb_writer(Arc::clone(&ssb));
    // Install the production DefaultComplianceProvider through the
    // same `set_compliance_provider` seam the CLI uses, with an
    // explicit `enabled=true` so the verdict-time env probe is
    // bypassed (race-free against Test 2's opt-out).
    let provider = Arc::new(DefaultComplianceProvider::with_gate_enabled(
        "s-e38-on-path".to_string(),
        Arc::clone(&ssb),
        true,
    ));
    fsm.set_compliance_provider(provider);

    // Drive IDLE -> PLANNING (produces 1 PhaseTransition SSB row, so
    // the AuditTrailLive assertion would pass; the gate still fails on
    // DeterministicSeed first).
    fsm.transition(FsmState::Planning, "plan-phase", vec![])
        .await
        .expect("IDLE -> PLANNING must succeed (gate is not in scope on this edge)");
    assert_eq!(fsm.get_state().await, FsmState::Planning);

    // Give the SSB async write task time to flush the PhaseTransition
    // row so the provider's viewer read at verdict time sees it.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Confirm the provider's read path sees >= 1 PhaseTransition row
    // (this is the AuditTrailLive precondition; it would pass, so the
    // block must come from DeterministicSeed, not AuditTrailLive).
    let viewer = SsbViewer::open(&cfg.db_path).expect("ssb viewer open");
    assert!(
        phase_transition_count(&viewer) >= 1,
        "IDLE -> PLANNING must emit >= 1 PhaseTransition row"
    );

    // Drive PLANNING -> TESTING_FIRST: gate must refuse.
    let err = fsm
        .transition(FsmState::TestingFirst, "plan-phase", vec![])
        .await
        .expect_err("PLANNING -> TESTING_FIRST must be blocked by the compliance gate");
    match err {
        FsmError::ComplianceBlocked { assertion_label, hint } => {
            assert_eq!(
                assertion_label, "deterministic_seed",
                "expected DeterministicSeed assertion to fire first"
            );
            assert!(
                !hint.is_empty(),
                "remediation hint must be populated"
            );
        }
        other => panic!("expected ComplianceBlocked, got {other:?}"),
    }

    // FSM must remain in PLANNING (rejection happens before mutation).
    assert_eq!(fsm.get_state().await, FsmState::Planning);

    // Give the SSB async write task time to flush the signal row.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // The off-agent artifact: a ComplianceBlocked SSB row with the
    // deterministic_seed assertion label. This is the wiring-enforcement
    // R4 proof -- a row the executor did NOT author, read back through
    // the real SsbReader.
    let (kind, assertion_label) = first_signal_row(&ssb);
    assert_eq!(kind, RoadmapSignalKind::ComplianceBlocked);
    assert_eq!(assertion_label, "deterministic_seed");

    // No gate-env cleanup needed: `with_gate_enabled(true)` bypassed
    // the env probe, so Test 2's opt-out is independent of this test.
    // `ELAI_AI_ROADMAP_SIGNALS_ENABLE` is left set; Test 2 also sets
    // it, so no race.
}

/// Test 2: gate opted OUT (env unset) -> the provider short-circuits to
/// `Pass { assertions_checked: 0 }` and the transition proceeds. A
/// `ComplianceVerified` SSB row lands with `assertions_checked = 0`.
/// This proves the default-OFF path is not bricked (the C-28 mirror).
///
/// Race-free: uses `with_gate_enabled(false)` so the provider does NOT
/// probe `ELAI_EU_AI_ACT_COMPLIANCE_GATE_ENABLE` at verdict time (see
/// Test 1's doc comment for the race rationale).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn gate_opted_out_passes_through_and_emits_verified_row() {
    // SSB emission leg is env-gated by `ELAI_AI_ROADMAP_SIGNALS_ENABLE`.
    std::env::set_var("ELAI_AI_ROADMAP_SIGNALS_ENABLE", "1");

    let dir = tempfile::TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb_e38_off_path.db"),
        pool_size: Some(2),
        project_id: "test-project".into(),
        session_id: "s-e38-off-path".into(),
    };
    let clock: Arc<dyn elai_core::ClockProvider> =
        Arc::new(elai_core::clock::SeededClock::new());
    let ssb = Arc::new(Ssb::new(cfg.clone(), clock).expect("ssb init"));

    let fsm = Fsm::new(None, 8000).with_ssb_writer(Arc::clone(&ssb));
    let provider = Arc::new(DefaultComplianceProvider::with_gate_enabled(
        "s-e38-off-path".to_string(),
        Arc::clone(&ssb),
        false,
    ));
    fsm.set_compliance_provider(provider);

    fsm.transition(FsmState::Planning, "plan-phase", vec![])
        .await
        .expect("IDLE -> PLANNING");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // PLANNING -> TESTING_FIRST with the gate OFF must succeed.
    fsm.transition(FsmState::TestingFirst, "plan-phase", vec![])
        .await
        .expect("PLANNING -> TESTING_FIRST must succeed when gate is opted out");
    assert_eq!(fsm.get_state().await, FsmState::TestingFirst);

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // A ComplianceVerified row with assertions_checked = 0 (the
    // short-circuit Pass) lands in the SSB.
    let rows = ssb
        .query_by_type(SsbActionType::AiRoadmapSignalEvent)
        .expect("query_by_type AiRoadmapSignalEvent");
    assert!(
        !rows.is_empty(),
        "expected at least one AiRoadmapSignalEvent row"
    );
    let payload: SsbPayload = rmp_serde::from_slice(&rows[0].payload)
        .expect("decode AiRoadmapSignalEvent payload");
    match payload {
        SsbPayload::AiRoadmapSignalEvent(p) => {
            assert_eq!(p.kind, RoadmapSignalKind::ComplianceVerified);
            assert_eq!(p.assertions_checked, 0);
        }
        other => panic!("expected AiRoadmapSignalEvent arm, got {other:?}"),
    }

    std::env::remove_var("ELAI_AI_ROADMAP_SIGNALS_ENABLE");
}
