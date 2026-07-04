// B-19 done-gate smoke: drive the real hook-event producer to write a
// hook_event row into the project-root SSB so wiring_contract_check.py's
// expect_ssb leg finds off-agent proof. The contract test
// (hook_dispatcher_contract.rs) proves the round-trip invariant; this smoke
// writes to the SAME db the check reads (<project_root>/sessions/ssb.db or
// ELAI_SSB_DB env override) so the off-agent DB row is present after the
// trigger runs.
//
// Trigger: cargo test -p elai_cc_contracts --test hook_smoke_ssb -- --nocapture
// The test builds a real Ssb, emits a HookEvent SSB row via
// emit_hook_events_to_ssb (hook_dispatcher.rs:1780), then polls the reader to
// confirm the row landed.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::hook_dispatcher::{
    emit_hook_events_to_ssb, HookFinding, HookPoint, HookTier, HookVerdictKind,
    HOOK_EVENT_CATEGORY,
};
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbReader};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn smoke_writes_hook_event_to_project_root_ssb() {
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
        project_id: "b19-smoke".to_owned(),
        session_id: "s-b19-smoke".to_owned(),
        pool_size: Some(2),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(config, clock).expect("smoke: Ssb::new must succeed on project-root db"),
    );

    // Real B-19 producer path: emit_hook_events_to_ssb writes one HookEvent
    // row per finding through the standard write_payload path.
    let findings = vec![HookFinding {
        tier: HookTier::T1,
        verdict: HookVerdictKind::Warn,
        rule_id: Some("b19-smoke-rule".into()),
        category: HOOK_EVENT_CATEGORY.into(),
        message: "b19 smoke: hook_event row emitted".into(),
        fix_template: None,
    }];
    let count = emit_hook_events_to_ssb(
        &ssb,
        "b19-smoke",
        0,
        HookPoint::PostToolUse,
        "b19_smoke_step",
        &findings,
    )
    .await;
    assert_eq!(count, 1, "smoke: exactly one hook_event row emitted");

    // Poll the reader until the row lands (async writer flush).
    let mut rows: Vec<_> = ssb
        .query_by_type(SsbActionType::HookEvent)
        .expect("smoke: query_by_type must succeed");
    for _ in 0..50 {
        if !rows.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        rows = ssb
            .query_by_type(SsbActionType::HookEvent)
            .expect("smoke: query_by_type must succeed");
    }
    assert!(
        !rows.is_empty(),
        "smoke: project-root SSB must contain a hook_event row after the trigger",
    );
}
