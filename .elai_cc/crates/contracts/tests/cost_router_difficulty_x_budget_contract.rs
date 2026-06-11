//! E-39 contract: cost router difficulty-x-budget matrix produces the
//! correct `RouteDecision` and the SSB row carries
//! `CostEventKind::CostRouterDecision` with the `chosen_backend` field.
//!
//! Drives `route_for_difficulty_and_budget` across 3 representative
//! cells of the decision matrix (easy/ample -> Local, hard/ample ->
//! Sonnet, very_hard/high_criticality -> Opus), writes to temp Ssb,
//! reads back and asserts `chosen_backend` in details_json.
//!
//! Production entrypoint: elai run -> drive_task -> cost_token_economics
//! path -> `route_for_difficulty_and_budget` (task_runner.rs:4423,
//! `run_cost_router_preclassify`).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::cost::cost_router::{
    route_for_difficulty_and_budget, Criticality, RouteDecision,
};
use elai_orchestrator::ssb::{
    CostEventKind, CostTokenEventPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_cost_router_difficulty_x_budget_ssb_roundtrip() {
    // 1. Drive the 3 representative matrix cells.
    let easy_ample = route_for_difficulty_and_budget(0.10, 50.0, Criticality::Low);
    assert_eq!(easy_ample, RouteDecision::Local, "easy+ample must route Local");

    let hard_ample = route_for_difficulty_and_budget(0.50, 50.0, Criticality::Medium);
    assert_eq!(hard_ample, RouteDecision::Sonnet, "hard+ample must route Sonnet");

    let very_hard_high =
        route_for_difficulty_and_budget(0.90, 100.0, Criticality::High);
    match very_hard_high {
        RouteDecision::Opus { requires_confirmation } => {
            assert!(requires_confirmation, "very_hard+High must require confirmation");
        }
        other => panic!("very_hard+High must route Opus, got {other:?}"),
    }

    // 2. Write the off-agent artifact SSB row for the hard_ample (Sonnet)
    //    cell — it carries a `chosen_backend` field that the spec calls
    //    out explicitly as the required assertion target.
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "e39-cost-router".into(),
        session_id: "e39-cost-router-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let payload = SsbPayload::CostTokenEvent(CostTokenEventPayload {
        kind: CostEventKind::CostRouterDecision,
        session_id: Some("e39-cost-router-session".into()),
        task_id: Some("task-router-001".into()),
        details_json: serde_json::json!({
            "difficulty": 0.50_f64,
            "budget_remaining_usd": 50.0_f64,
            "chosen_backend": hard_ample.as_str(),
            "criticality": "medium",
            "rationale": "hard+ample => sonnet"
        }),
        ts_ms: 1_717_000_003_000,
    });
    ssb.write_payload("cost", 0, payload, Vec::new())
        .await
        .expect("write");

    // 3. Read back and assert the `chosen_backend` field.
    let rows = ssb
        .query_by_type(SsbActionType::CostTokenEvent)
        .expect("query");
    assert_eq!(rows.len(), 1, "expected exactly one CostTokenEvent row");
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::CostTokenEvent(p) => {
            assert_eq!(
                p.kind,
                CostEventKind::CostRouterDecision,
                "kind discriminator must be CostRouterDecision"
            );
            assert_eq!(
                p.details_json["chosen_backend"],
                serde_json::json!("sonnet"),
                "chosen_backend must be 'sonnet' for hard+ample cell"
            );
        }
        other => panic!("expected CostTokenEvent, got {other:?}"),
    }
}
