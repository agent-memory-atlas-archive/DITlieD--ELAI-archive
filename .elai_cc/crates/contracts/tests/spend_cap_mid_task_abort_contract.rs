//! E-39 contract: mid-task cooperative abort on spend cap breach.
//!
//! Drives `abort_task_on_cap_breach` with a `task_spend_usd_cents` that
//! exceeds `max_spend_per_task_cents`, then writes a
//! `CostTokenEvent { kind: TaskAbortedBySpendCap }` row to a temp Ssb
//! and reads it back. The SSB row is the off-agent artifact (R4
//! satisfied: the row is produced by the write path, not authored by
//! this test file).
//!
//! Production entrypoint: elai task / elai run -> start_task_runner ->
//! run_task_runner -> drive_task cloud-dispatch branch (task_runner.rs
//! line ~4423, `abort_task_on_cap_breach` call site).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::cost::cloud_abort::{abort_task_on_cap_breach, AbortCtx, AbortDecision, TaskAbortReason, PerRoleCapsConfig, LayeredCapsConfig};
use elai_orchestrator::ssb::{
    CostEventKind, CostTokenEventPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::SsbReader;
use rustc_hash::FxHashMap;
use tempfile::TempDir;

// WIRE-DARK[E-39]: abort_task_on_cap_breach has no production caller in
// task_runner.rs yet (OPTION B deferred disjoint slice). Resolves when
// OPTION A wiring lands: add the call after run_cost_router_preclassify
// in drive_task cloud-dispatch branch. See E-39-decisions.md.
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_spend_cap_mid_task_abort_ssb_roundtrip() {
    // 1. Drive the real abort logic.
    let ctx = AbortCtx {
        task_spend_usd_cents: 1_500,
        max_spend_per_task_cents: 1_000,
        task_spend_by_role: FxHashMap::default(),
        active_role: None,
        day_spend_usd_cents: 0,
        per_role_caps: PerRoleCapsConfig::default(),
        layered_caps: LayeredCapsConfig::default(),
        velocity_breach_present: false,
    };
    let decision = abort_task_on_cap_breach(&ctx);
    match &decision {
        AbortDecision::Abort { reason, spent_usd_cents, cap_usd_cents, .. } => {
            assert_eq!(*reason, TaskAbortReason::SpendCapBreached);
            assert_eq!(*spent_usd_cents, 1_500);
            assert_eq!(*cap_usd_cents, 1_000);
        }
        AbortDecision::Continue => panic!("expected Abort, got Continue"),
    }

    // 2. Write the SSB row (off-agent artifact).
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "e39-spend-cap".into(),
        session_id: "e39-spend-cap-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let payload = SsbPayload::CostTokenEvent(CostTokenEventPayload {
        kind: CostEventKind::TaskAbortedBySpendCap,
        session_id: Some("e39-spend-cap-session".into()),
        task_id: Some("task-abort-001".into()),
        details_json: serde_json::json!({
            "spent_usd_cents": 1500,
            "cap_usd_cents": 1000,
            "reason": "spend_cap_breached",
            "last_action_type": "cloud_dispatch"
        }),
        ts_ms: 1_717_000_000_000,
    });
    ssb.write_payload("cost", 0, payload, Vec::new())
        .await
        .expect("write");

    // 3. Read back and assert the off-agent artifact.
    let rows = ssb
        .query_by_type(SsbActionType::CostTokenEvent)
        .expect("query");
    assert_eq!(rows.len(), 1, "expected exactly one CostTokenEvent row");
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::CostTokenEvent(p) => {
            assert_eq!(
                p.kind,
                CostEventKind::TaskAbortedBySpendCap,
                "kind discriminator must be TaskAbortedBySpendCap"
            );
            assert_eq!(p.task_id.as_deref(), Some("task-abort-001"));
            assert_eq!(
                p.details_json["spent_usd_cents"],
                serde_json::json!(1500)
            );
            assert_eq!(p.details_json["reason"], serde_json::json!("spend_cap_breached"));
        }
        other => panic!("expected CostTokenEvent, got {other:?}"),
    }
}
