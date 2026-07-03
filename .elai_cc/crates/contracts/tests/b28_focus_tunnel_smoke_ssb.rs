// B-28 done-gate smoke: drive the real focus-tunnel SSB producer to write
// a generation_mode_selected row into the project-root SSB so
// wiring_contract_check.py's expect_ssb leg finds off-agent proof.
// The contract tests (focus_tunnel_e3_contract.rs,
// focus_tunnel_wiring_contract.rs) prove the round-trip invariants
// against temp DBs; this smoke writes to the SAME db the check reads
// (<project_root>/sessions/ssb.db or ELAI_SSB_DB env override) so the
// off-agent DB row is present after the trigger runs.
//
// Trigger: cargo test -p elai_cc_contracts --test b28_focus_tunnel_smoke_ssb -- --nocapture
// The test builds a real Ssb, emits one GenerationModeSelected SSB row via
// the standard write_payload path (the same payload shape task_runner.rs:8835
// emits unconditionally on every baby step), then polls the reader to
// confirm the row landed.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::ssb::{
    GenerationModeSelectedPayload, Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn smoke_writes_generation_mode_selected_to_project_root_ssb() {
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
        project_id: "b28-smoke".to_owned(),
        session_id: "s-b28-smoke".to_owned(),
        pool_size: Some(2),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(config, clock).expect("smoke: Ssb::new must succeed on project-root db"),
    );

    // Real B-28 producer path: write_payload with
    // SsbPayload::GenerationModeSelected (the same payload shape
    // task_runner.rs:8835-8858 emits unconditionally on every baby step
    // via writer.write_payload).
    let mode_selected = SsbPayload::GenerationModeSelected(GenerationModeSelectedPayload {
        step_id: "baby_step_b28_smoke".into(),
        mode: "fim".into(),
        rule_fired: "fim_all_conditions_met".into(),
        reason: "rule_3_fim_all_conditions_met".into(),
        preferred_was: None,
        supports_fim: true,
        exec_mode: "local".into(),
    });
    ssb.write_payload("b28_smoke", 0, mode_selected, Vec::new())
        .await
        .expect("smoke: write GenerationModeSelected must succeed");

    // Poll the reader until the generation_mode_selected row lands (async
    // writer flush). expect_ssb in the B-28 contract is
    // "generation_mode_selected" so this is the load-bearing row.
    let mut rows: Vec<_> = ssb
        .query_by_type(SsbActionType::GenerationModeSelected)
        .expect("smoke: query_by_type GenerationModeSelected must succeed");
    for _ in 0..50 {
        if !rows.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        rows = ssb
            .query_by_type(SsbActionType::GenerationModeSelected)
            .expect("smoke: query_by_type GenerationModeSelected must succeed");
    }
    assert!(
        !rows.is_empty(),
        "smoke: project-root SSB must contain a generation_mode_selected row after the trigger",
    );
}
