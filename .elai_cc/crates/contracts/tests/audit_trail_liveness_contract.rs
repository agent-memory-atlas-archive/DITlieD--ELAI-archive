// E-38 Task 8c -- audit-trail liveness contract (EU AI Act art. 12).
//
// Drive 5 FSM transitions through the real Fsm::transition_with_metrics
// producer (fsm.rs:1208-1229 emits SsbPayload::PhaseTransition on every
// successful transition). Assert a phase_transition SSB row lands for EACH
// transition (5 rows total, matching from/to pairs), read back via
// SsbViewer::recent_phase_transitions. The twin at
// fsm_phase_transition_contract.rs proves 1 transition emits 1 row; this
// test proves the emit is not single-shot and survives a 5-step chain.
//
// SSB construction mirrors the twin: file-backed SQLite under tempfile::TempDir,
// OsClock for the writer, SsbViewer::open against the same db_path. No .unwrap()
// in setup. No compliance_provider installed -- the PLANNING/FEASIBILITY_PROBE
// -> TESTING_FIRST gate (fsm.rs:1034) is a no-op without a provider, so every
// transition reaches the emit site.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::fsm::{Fsm, FsmState};
use elai_orchestrator::ssb::{Ssb, SsbConfig};
use elai_orchestrator::ssb_viewer::SsbViewer;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helper: deterministic SSB + tempdir setup with no .unwrap() in test setup.
// ---------------------------------------------------------------------------

/// Construct an empty SSB on disk + open a viewer pointed at the same db.
/// Returned `TempDir` must outlive the SSB/viewer (RAII deletes the tmpdir).
fn make_ssb_and_viewer() -> (TempDir, Arc<Ssb>, SsbViewer) {
    let dir = TempDir::new().expect("tempdir creation must succeed in test setup");
    let db_path = dir.path().join("e38_audit_trail_liveness.db");
    let config = SsbConfig {
        db_path: db_path.clone(),
        project_id: "e38-audit-trail-liveness".to_owned(),
        session_id: "s-e38-audit".to_owned(),
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
            .recent_phase_transitions(32)
            .expect("recent_phase_transitions read must succeed");
        if rows.len() >= expected {
            return rows;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    viewer
        .recent_phase_transitions(32)
        .expect("recent_phase_transitions read must succeed (final attempt)")
}

// ---------------------------------------------------------------------------
// Contract: a 5-transition chain through the real Fsm producer leaves exactly
// 5 phase_transition SSB rows, one per transition, with the expected from/to
// pairs. EU AI Act art. 12 record-keeping: every FSM transition must leave a
// trace; the emit is not single-shot and survives a sequence.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn audit_trail_liveness_emits_on_each_of_five_transitions() {
    let (_dir, ssb, viewer) = make_ssb_and_viewer();
    let fsm = Fsm::new(None, 8_000).with_ssb_writer(Arc::clone(&ssb));

    // Chain (valid per fsm.rs valid_targets matrix, avoids the
    // Auditing -> Memorizing audit-block gate at fsm.rs:1002):
    //   1. IDLE             -> PLANNING
    //   2. PLANNING         -> FEASIBILITY_PROBE
    //   3. FEASIBILITY_PROBE-> TESTING_FIRST
    //   4. TESTING_FIRST    -> EXECUTING
    //   5. EXECUTING        -> AUDITING
    let chain: [(FsmState, FsmState, &str); 5] = [
        (FsmState::Idle, FsmState::Planning, "e38-8c-1"),
        (FsmState::Planning, FsmState::FeasibilityProbe, "e38-8c-2"),
        (FsmState::FeasibilityProbe, FsmState::TestingFirst, "e38-8c-3"),
        (FsmState::TestingFirst, FsmState::Executing, "e38-8c-4"),
        (FsmState::Executing, FsmState::Auditing, "e38-8c-5"),
    ];

    for (source, target, label) in chain.iter().copied() {
        let record = fsm
            .transition_with_metrics(target, label, Vec::new(), None)
            .await
            .expect("transition must succeed");
        assert_eq!(record.source, source, "TransitionRecord.source mismatch");
        assert_eq!(record.target, target, "TransitionRecord.target mismatch");
    }

    // Wait for all 5 async SSB writes to land.
    let rows = wait_for_phase_transitions(&viewer, 5, 100).await;
    assert_eq!(
        rows.len(),
        5,
        "expected exactly 5 phase_transition rows after 5 transitions, got {}",
        rows.len(),
    );

    // recent_phase_transitions returns newest-first; reverse for chronological.
    let mut chronological: Vec<(String, String)> =
        rows.iter().map(|(_, from, to, _)| (from.clone(), to.clone())).collect();
    chronological.reverse();

    let expected: Vec<(String, String)> = chain
        .iter()
        .map(|(s, t, _)| (s.to_string(), t.to_string()))
        .collect();

    for (i, (got, want)) in chronological.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            got, want,
            "phase_transition row {} from/to mismatch: got {:?} want {:?}",
            i, got, want,
        );
    }

    // Viewer's latest_fsm_state must reflect the final target (AUDITING).
    let snapshot = viewer
        .latest_fsm_state()
        .expect("latest_fsm_state must succeed");
    assert_eq!(
        snapshot.state, "AUDITING",
        "latest_fsm_state.state must be AUDITING after the 5-transition chain",
    );
}
