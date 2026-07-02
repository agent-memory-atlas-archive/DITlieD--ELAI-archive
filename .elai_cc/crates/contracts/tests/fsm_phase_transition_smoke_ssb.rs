// B-2 done-gate smoke: drive the real Fsm producer to write a phase_transition
// row into the project-root SSB so wiring_contract_check.py's expect_ssb leg
// finds off-agent proof. The contract test (fsm_phase_transition_contract.rs)
// writes to a tempdir and proves the producer+consumer path; this smoke writes
// to the SAME db the check reads (<project_root>/sessions/ssb.db or
// ELAI_SSB_DB env override) so the off-agent DB row is present after the
// trigger runs.
//
// Trigger: cargo test -p elai_cc_contracts --test fsm_phase_transition_smoke_ssb
// The test performs one IDLE -> PLANNING transition via the real
// Fsm::transition_with_metrics (fsm.rs:963) which emits SsbPayload::PhaseTransition
// at fsm.rs:1189-1210, then polls the viewer to confirm the row landed.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::fsm::{Fsm, FsmState};
use elai_orchestrator::ssb::{Ssb, SsbConfig};
use elai_orchestrator::ssb_viewer::SsbViewer;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn smoke_writes_phase_transition_to_project_root_ssb() {
    // Resolve the DB path: ELAI_SSB_DB env wins; otherwise the workspace
    // default <contracts_crate>/../../sessions/ssb.db == <project_root>/sessions/ssb.db
    // which is _ssb_candidates[0] in wiring_contract_check.py.
    let db_path = std::env::var("ELAI_SSB_DB")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            manifest.join("../../sessions/ssb.db")
        });

    // Ensure the parent dir exists (the SSB constructor does not create it).
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let config = SsbConfig {
        db_path: db_path.clone(),
        project_id: "b2-smoke".to_owned(),
        session_id: "s-b2-smoke".to_owned(),
        pool_size: Some(2),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(config, clock).expect("smoke: Ssb::new must succeed on project-root db"),
    );

    let fsm = Fsm::new(None, 8_000).with_ssb_writer(Arc::clone(&ssb));

    fsm.transition_with_metrics(FsmState::Planning, "b2-smoke", Vec::new(), None)
        .await
        .expect("smoke: IDLE -> PLANNING transition must succeed");

    // Poll the viewer until the row lands (async writer flush).
    let viewer = SsbViewer::open(&db_path).expect("smoke: SsbViewer::open must succeed");
    let mut snapshot = viewer
        .latest_fsm_state()
        .expect("smoke: latest_fsm_state must succeed");
    for _ in 0..50 {
        if snapshot.state != "IDLE" {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        snapshot = viewer
            .latest_fsm_state()
            .expect("smoke: latest_fsm_state must succeed");
    }
    assert_ne!(
        snapshot.state, "IDLE",
        "smoke: project-root SSB must contain a non-IDLE phase_transition row after the trigger",
    );
}
