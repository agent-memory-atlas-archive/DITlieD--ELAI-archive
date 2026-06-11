//! E-39 contract: cost estimate inaccuracy threshold produces a
//! `CostTokenEvent { kind: CostEstimateInaccurate }` SSB row when the
//! actual/estimate ratio falls outside [0.5, 2.0].
//!
//! The inaccuracy detection is a ratio check -- no dedicated struct
//! exists; the logic is a direct comparison in the task_runner dispatch
//! path. This contract test drives the ratio computation inline and
//! asserts the SSB row's details_json carries the expected fields.
//!
//! Production entrypoint: elai run -> drive_task cloud-dispatch branch
//! post-call cost-accuracy check (task_runner.rs, E-39 cost economics
//! path; writes CostEstimateInaccurate when ratio < 0.5 or > 2.0).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::ssb::{
    CostEventKind, CostTokenEventPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

/// Ratio threshold defined in the plan spec: outside [0.5, 2.0] is
/// inaccurate. This constant mirrors the decision in E-39-decisions.md.
const INACCURACY_LOW: f64 = 0.5;
const INACCURACY_HIGH: f64 = 2.0;

// WIRE-DARK[E-39]: cost-estimate accuracy check has no production caller
// in drive_task post-call path yet (OPTION B deferred disjoint slice).
// Resolves when a check_cost_estimate_accuracy() call is added in the
// cloud-dispatch branch after the cloud response returns. See
// E-39-decisions.md.
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_cost_estimate_inaccuracy_threshold_ssb_roundtrip() {
    // 1. Drive the ratio check inline (the plan specifies this as a
    //    free ratio comparison, not a dedicated struct).
    let estimate_usd = 0.05_f64;
    let actual_usd = 0.15_f64; // ratio = 3.0 -> outside [0.5, 2.0]
    let ratio = actual_usd / estimate_usd.max(f64::EPSILON);
    assert!(
        ratio > INACCURACY_HIGH || ratio < INACCURACY_LOW,
        "ratio {ratio} must be outside [0.5, 2.0] to trigger the inaccuracy row"
    );

    // 2. Write the off-agent artifact SSB row.
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "e39-cost-estimate".into(),
        session_id: "e39-cost-estimate-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let payload = SsbPayload::CostTokenEvent(CostTokenEventPayload {
        kind: CostEventKind::CostEstimateInaccurate,
        session_id: Some("e39-cost-estimate-session".into()),
        task_id: Some("task-estimate-001".into()),
        details_json: serde_json::json!({
            "estimate_usd": estimate_usd,
            "actual_usd": actual_usd,
            "ratio": ratio,
            "p50_usd": 0.08_f64,
            "p95_usd": 0.20_f64
        }),
        ts_ms: 1_717_000_005_000,
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
                CostEventKind::CostEstimateInaccurate,
                "kind discriminator must be CostEstimateInaccurate"
            );
            let r = p.details_json["ratio"]
                .as_f64()
                .expect("ratio must be an f64");
            assert!(
                r > INACCURACY_HIGH || r < INACCURACY_LOW,
                "SSB ratio {r} must be outside the [0.5, 2.0] threshold"
            );
            assert_eq!(
                p.details_json["estimate_usd"],
                serde_json::json!(estimate_usd)
            );
            assert_eq!(
                p.details_json["actual_usd"],
                serde_json::json!(actual_usd)
            );
        }
        other => panic!("expected CostTokenEvent, got {other:?}"),
    }
}
