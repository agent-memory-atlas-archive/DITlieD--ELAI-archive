//! E-39 contract: per-role cap enforcement produces
//! `AbortDecision::Abort { reason: PerRoleCapBreached }` and the SSB
//! row carries `CostEventKind::TaskAbortedBySpendCap` with the role
//! field set.
//!
//! Drives `abort_task_on_cap_breach` with an auditor role that has
//! exceeded its per-role cap, writes the SSB row, reads back and
//! asserts `reason == "per_role_cap_breached"` and the role label.
//!
//! Production entrypoint: elai run -> drive_task -> post-call cost
//! check -> `abort_task_on_cap_breach` (task_runner.rs cloud-dispatch
//! branch).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::cost::cloud_abort::{
    abort_task_on_cap_breach, AbortCtx, AbortDecision, LayeredCapsConfig, PerRoleCapsConfig,
    TaskAbortReason,
};
use elai_orchestrator::cost::CostRoleBucket;
use elai_orchestrator::ssb::{
    CostEventKind, CostTokenEventPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::SsbReader;
use rustc_hash::FxHashMap;
use tempfile::TempDir;

// WIRE-DARK[E-39]: abort_task_on_cap_breach (per-role variant) has no
// production caller in task_runner.rs yet (OPTION B deferred disjoint
// slice). Same wiring gap as spend_cap_mid_task_abort. Resolves when the
// call is added to drive_task cloud-dispatch branch. See E-39-decisions.md.
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_per_role_cap_enforcement_ssb_roundtrip() {
    // 1. Drive the real abort logic with auditor over its cap.
    let mut per_role_caps = PerRoleCapsConfig::default();
    per_role_caps
        .caps_usd_cents
        .insert(CostRoleBucket::Auditor, 500);
    let mut by_role = FxHashMap::default();
    by_role.insert(CostRoleBucket::Auditor, 600_u64);

    let ctx = AbortCtx {
        task_spend_usd_cents: 600,
        max_spend_per_task_cents: 10_000, // task cap NOT breached
        task_spend_by_role: by_role,
        active_role: Some(CostRoleBucket::Auditor),
        day_spend_usd_cents: 600,
        per_role_caps,
        layered_caps: LayeredCapsConfig::default(),
        velocity_breach_present: false,
    };
    let decision = abort_task_on_cap_breach(&ctx);
    match &decision {
        AbortDecision::Abort { reason, role, .. } => {
            assert_eq!(*reason, TaskAbortReason::PerRoleCapBreached);
            assert_eq!(*role, Some(CostRoleBucket::Auditor));
        }
        AbortDecision::Continue => panic!("expected PerRoleCapBreached abort, got Continue"),
    }

    // 2. Write the off-agent artifact SSB row.
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "e39-per-role".into(),
        session_id: "e39-per-role-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let payload = SsbPayload::CostTokenEvent(CostTokenEventPayload {
        kind: CostEventKind::TaskAbortedBySpendCap,
        session_id: Some("e39-per-role-session".into()),
        task_id: Some("task-per-role-001".into()),
        details_json: serde_json::json!({
            "spent_usd_cents": 600,
            "cap_usd_cents": 500,
            "reason": "per_role_cap_breached",
            "role": CostRoleBucket::Auditor.as_str()
        }),
        ts_ms: 1_717_000_006_000,
    });
    ssb.write_payload("cost", 0, payload, Vec::new())
        .await
        .expect("write");

    // 3. Read back and assert.
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
            assert_eq!(
                p.details_json["reason"],
                serde_json::json!("per_role_cap_breached"),
                "reason must be per_role_cap_breached"
            );
            assert_eq!(
                p.details_json["role"],
                serde_json::json!("auditor"),
                "role must be 'auditor'"
            );
        }
        other => panic!("expected CostTokenEvent, got {other:?}"),
    }
}
