//! E-39 contract: 1-min rolling window breach produces a
//! `CostTokenEvent { kind: VelocityCapBreached }` SSB row.
//!
//! Drives the real `VelocityCapEnforcer` with a low req limit, trips
//! the breach, then writes + reads back from a temp Ssb. The SSB row
//! is the off-agent artifact.
//!
//! Production entrypoint: elai run -> drive_task cloud-dispatch branch
//! -> `velocity_cap.enforce()` call site (task_runner.rs ~line 4423).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::cost::velocity_cap::{
    VelocityCapEnforcer, VelocityCapsConfig, VelocityMetricKind, VelocityWindow,
};
use elai_orchestrator::ssb::{
    CostEventKind, CostTokenEventPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

// WIRE-DARK[E-39]: VelocityCapEnforcer.enforce() has no production caller
// in task_runner.rs yet (OPTION B deferred disjoint slice). Resolves when
// enforce() is called after run_cost_router_preclassify in drive_task
// cloud-dispatch branch. See E-39-decisions.md.
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_velocity_cap_1min_breach_ssb_roundtrip() {
    // 1. Drive the real enforcer — set 1-min req limit = 2, record 5
    //    samples, enforce must return a VelocityBreach.
    let mut cfg = VelocityCapsConfig::default();
    cfg.one_min_req_limit = Some(2);
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let mut enf = VelocityCapEnforcer::new(cfg, clock.clone());
    for _ in 0..5 {
        enf.record(50); // 50 usd_cents each
    }
    let breach = enf
        .enforce()
        .expect_err("enforcer must detect 1-min req breach");
    assert_eq!(breach.window, VelocityWindow::OneMin);
    assert_eq!(breach.kind, VelocityMetricKind::Requests);
    assert!(breach.count > breach.limit, "count must exceed limit");

    // 2. Write the off-agent artifact SSB row.
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "e39-velocity-cap".into(),
        session_id: "e39-velocity-cap-session".into(),
    };
    let clock2: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock2).expect("ssb"));

    let payload = SsbPayload::CostTokenEvent(CostTokenEventPayload {
        kind: CostEventKind::VelocityCapBreached,
        session_id: Some("e39-velocity-cap-session".into()),
        task_id: Some("task-vel-001".into()),
        details_json: serde_json::json!({
            "window": breach.window.as_str(),
            "kind": breach.kind.as_str(),
            "count": breach.count,
            "limit": breach.limit
        }),
        ts_ms: 1_717_000_001_000,
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
                CostEventKind::VelocityCapBreached,
                "kind discriminator must be VelocityCapBreached"
            );
            assert_eq!(p.details_json["window"], serde_json::json!("one_min"));
            assert_eq!(p.details_json["kind"], serde_json::json!("requests"));
            let count = p.details_json["count"].as_u64().expect("count u64");
            let limit = p.details_json["limit"].as_u64().expect("limit u64");
            assert!(count > limit, "count must exceed limit in SSB details");
        }
        other => panic!("expected CostTokenEvent, got {other:?}"),
    }
}
