// C-18 B-2 backfill — FSM PhaseTransition SSB emission contract.
//
// Three assertions, one test:
//   1. `Fsm::transition_with_metrics(IDLE -> PLANNING, ..)` succeeds.
//   2. After the transition, `SsbViewer::latest_fsm_state()` returns
//      `state == "PLANNING"`.
//   3. The most-recent `phase_transition` SSB row carries
//      `from_state == "IDLE"` and `to_state == "PLANNING"` (decoded via
//      `SsbViewer::recent_phase_transitions`).
//   4. `Fsm::reset()` after the transition does NOT emit a second
//      `phase_transition` row — documents the deliberate exclusion called out
//      in the B-2 verification report.
//
// SSB construction follows the exact in-tree precedent at
// `crates/orchestrator/src/fsm.rs:1554` (test_transition_emits_phase_transition_ssb)
// — file-backed SQLite under a `tempfile::TempDir`, `OsClock` for the writer,
// `viewer::SsbViewer::open` against the same db_path. No `.unwrap()` in setup.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::fsm::{Fsm, FsmState};
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig};
use elai_orchestrator::ssb_viewer::SsbViewer;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helper: deterministic SSB + tempdir setup with no .unwrap() in test setup.
// ---------------------------------------------------------------------------

/// Construct an empty SSB on disk + open a viewer pointed at the same db.
/// Returned `TempDir` must outlive the SSB/viewer (RAII deletes the tmpdir).
fn make_ssb_and_viewer() -> (TempDir, Arc<Ssb>, SsbViewer) {
    let dir = TempDir::new().expect("tempdir creation must succeed in test setup");
    let db_path = dir.path().join("c18_fsm_phase_transition.db");
    let config = SsbConfig {
        db_path: db_path.clone(),
        project_id: "c18-fsm-phase-transition".to_owned(),
        session_id: "s-c18-fsm".to_owned(),
        pool_size: Some(2),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(config, clock).expect("ssb::new must succeed for fresh tempdir"));
    let viewer = SsbViewer::open(&db_path).expect("ssb_viewer::open must succeed");
    (dir, ssb, viewer)
}

/// Poll the viewer until the expected number of `phase_transition` rows is
/// present, or the polling budget is exhausted.  The SSB write goes through an
/// async writer task; immediate reads after `transition_with_metrics` may
/// observe zero rows because the write has not yet flushed.
async fn wait_for_phase_transitions(
    viewer: &SsbViewer,
    expected: usize,
    max_polls: u32,
) -> Vec<(i64, String, String, i64)> {
    for _ in 0..max_polls {
        let rows = viewer
            .recent_phase_transitions(16)
            .expect("recent_phase_transitions read must succeed");
        if rows.len() >= expected {
            return rows;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    viewer
        .recent_phase_transitions(16)
        .expect("recent_phase_transitions read must succeed (final attempt)")
}

// ---------------------------------------------------------------------------
// Contract: IDLE -> PLANNING transition emits exactly one phase_transition row
// with the expected from/to fields, viewer agrees with the latest state, and
// `reset()` after the transition does NOT emit a second row.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fsm_phase_transition_contract_idle_to_planning() {
    let (_dir, ssb, viewer) = make_ssb_and_viewer();

    let fsm = Fsm::new(None, 8_000).with_ssb_writer(Arc::clone(&ssb));

    // 1. Successful transition.
    let record = fsm
        .transition_with_metrics(FsmState::Planning, "plan-phase", Vec::new(), None)
        .await
        .expect("IDLE -> PLANNING transition must succeed");
    assert_eq!(record.source, FsmState::Idle, "TransitionRecord.source must be Idle");
    assert_eq!(record.target, FsmState::Planning, "TransitionRecord.target must be Planning");

    // 2. Wait for the async SSB write to land, then read back the single row.
    let rows = wait_for_phase_transitions(&viewer, 1, 50).await;
    assert_eq!(
        rows.len(),
        1,
        "expected exactly 1 phase_transition row after one transition, got {}",
        rows.len(),
    );
    let (_seq, from_state, to_state, _ts_ms) = &rows[0];
    assert_eq!(from_state, "IDLE", "phase_transition.from_state must be IDLE");
    assert_eq!(to_state, "PLANNING", "phase_transition.to_state must be PLANNING");

    // 3. Viewer's latest_fsm_state must reflect the new target.
    let snapshot = viewer
        .latest_fsm_state()
        .expect("latest_fsm_state must succeed");
    assert_eq!(
        snapshot.state, "PLANNING",
        "latest_fsm_state.state must equal target.to_string() after a successful transition",
    );

    // 4. Reset must NOT emit a second phase_transition row.
    //
    // The Fsm::reset() path returns to IDLE without going through the normal
    // transition matrix (see fsm.rs §reset). The B-2 report explicitly calls
    // this out as a "deliberate exclusion" — only matrix-validated transitions
    // emit phase_transition rows. We lock that contract here.
    fsm.reset().await.expect("reset must succeed after a successful transition");

    // Poll a few times to allow a hypothetical reset-time write to land. We
    // require row count to stay at 1.
    tokio::time::sleep(std::time::Duration::from_millis(80)).await;
    let rows_after_reset = viewer
        .recent_phase_transitions(16)
        .expect("recent_phase_transitions after reset must succeed");
    assert_eq!(
        rows_after_reset.len(),
        1,
        "reset() must NOT emit a second phase_transition row (B-2 deliberate exclusion); \
         observed {} rows after reset",
        rows_after_reset.len(),
    );
}

// ---------------------------------------------------------------------------
// Compile-time anchor: PhaseTransition variant + action_type round-trip.
// Locks the SSB type-shape that the Fsm engine writes; if the variant goes
// away or the action_type string drifts, this test fails to compile.
// ---------------------------------------------------------------------------

#[test]
fn phase_transition_action_type_string_locked() {
    assert_eq!(
        SsbActionType::PhaseTransition.as_sql_str(),
        "phase_transition",
        "SsbActionType::PhaseTransition must serialise to the SQL string \
         'phase_transition' — the viewer query in latest_fsm_state depends on it",
    );
}
