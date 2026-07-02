// B-10 done-gate smoke: drive the real ContextBudget producer to write a
// context_budget_computed row into the project-root SSB so wiring_contract_check.py's
// expect_ssb leg finds off-agent proof. The contract test (budget_parity.rs)
// proves the TaskRunner-vs-REPL parity invariant; this smoke writes to the SAME
// db the check reads (<project_root>/sessions/ssb.db or ELAI_SSB_DB env override)
// so the off-agent DB row is present after the trigger runs.
//
// Trigger: cargo test -p elai_cc_contracts --test context_budget_smoke_ssb
// The test builds a real ModelCapabilityProfile (micro_7b_12gb), computes a
// ContextBudget via compute_with_profile (context_budget.rs:1379), and emits a
// ContextBudgetComputed SSB row via emit_context_budget_computed
// (context_budget.rs:1798), then polls the viewer to confirm the row landed.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::context_budget::{
    emit_context_budget_computed, ContextBudget, ModelCapabilityProfile,
};
use elai_orchestrator::ssb::{Ssb, SsbConfig};
use elai_orchestrator::ssb_viewer::SsbViewer;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn smoke_writes_context_budget_computed_to_project_root_ssb() {
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
        project_id: "b10-smoke".to_owned(),
        session_id: "s-b10-smoke".to_owned(),
        pool_size: Some(2),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(config, clock).expect("smoke: Ssb::new must succeed on project-root db"),
    );

    // Real B-10 producer path: profile -> compute_with_profile -> emit SSB row.
    let profile = ModelCapabilityProfile::micro_7b_12gb();
    let budget = ContextBudget::compute_with_profile(32_768, [0, 0, 0], &profile)
        .expect("smoke: compute_with_profile must succeed for micro_7b_12gb at 32k");
    emit_context_budget_computed(&ssb, "b10-smoke", 0, &profile, &budget, 32_768)
        .await
        .expect("smoke: emit_context_budget_computed must succeed");

    // Poll the viewer until the row lands (async writer flush).
    let viewer = SsbViewer::open(&db_path).expect("smoke: SsbViewer::open must succeed");
    let mut snapshot = viewer
        .context_budget_computed_snapshot()
        .expect("smoke: context_budget_computed_snapshot must succeed");
    for _ in 0..50 {
        if snapshot.is_some() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        snapshot = viewer
            .context_budget_computed_snapshot()
            .expect("smoke: context_budget_computed_snapshot must succeed");
    }
    assert!(
        snapshot.is_some(),
        "smoke: project-root SSB must contain a context_budget_computed row after the trigger",
    );
}
