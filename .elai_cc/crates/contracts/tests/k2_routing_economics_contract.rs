//! K-2 routing-economics vertical-slice contract.

use std::sync::Arc;

use elai_core::clock::ClockProvider;
use elai_core::SeededClock;
use elai_orchestrator::benchmarks::routing_economics_runner::{
    compose_routing_economics, run_routing_economics, RunnerStatus,
    LIVE_COST_UNAVAILABLE,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, RoutingEconomicsMeasuredPayload, SsbActionType,
    SsbReader as _, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{Ssb, SsbConfig, SsbPayload};

fn status_str(status: RunnerStatus) -> String {
    match status {
        RunnerStatus::Measured => "measured".to_string(),
        RunnerStatus::RunnerPresent => "runner_present".to_string(),
    }
}

fn payload_from_report(
    report: &elai_orchestrator::benchmarks::routing_economics_runner::RoutingEconomicsReport,
) -> RoutingEconomicsMeasuredPayload {
    RoutingEconomicsMeasuredPayload {
        seed: report.seed,
        runner_status: status_str(report.runner_status),
        fixture_status: status_str(report.fixture_status),
        selected_arm: report.selected_arm.clone(),
        suite: report.suite.clone(),
        fixture_blake3: report.fixture_blake3.clone(),
        report_blake3: blake3::hash(
            &serde_json::to_vec(report).expect("serialize report for beacon hash"),
        )
        .to_hex()
        .to_string(),
        hybrid_n_tasks: report.hybrid.n_tasks,
        hybrid_n_offloaded: report.hybrid.n_offloaded,
        hybrid_percent_offloaded: report.hybrid.percent_offloaded,
        hybrid_n_verified: report.hybrid.n_verified,
        hybrid_total_cost_micros: report.hybrid.total_cost_micros,
        hybrid_cost_per_verified_task_micros: report.hybrid.cost_per_verified_task_micros,
        force_cloud_n_tasks: report.force_cloud_everything.n_tasks,
        force_cloud_n_offloaded: report.force_cloud_everything.n_offloaded,
        force_cloud_percent_offloaded: report.force_cloud_everything.percent_offloaded,
        force_cloud_n_verified: report.force_cloud_everything.n_verified,
        force_cloud_total_cost_micros: report.force_cloud_everything.total_cost_micros,
        force_cloud_cost_per_verified_task_micros: report
            .force_cloud_everything
            .cost_per_verified_task_micros,
        live_cost: report.live_cost.clone(),
        live_lane_funded: report.live_lane_funded,
    }
}

#[test]
fn hybrid_offloads_strictly_more_than_force_cloud() {
    let report = run_routing_economics(42, false).expect("runner");
    assert!(
        report.hybrid.percent_offloaded
            > report.force_cloud_everything.percent_offloaded,
        "hybrid must offload strictly more work than the force-cloud control"
    );
    assert_eq!(report.force_cloud_everything.percent_offloaded, 0.0);
    assert!(
        report.hybrid.total_cost_micros
            < report.force_cloud_everything.total_cost_micros,
        "the hybrid fixture must cost less while retaining verified outcomes"
    );
}

#[test]
fn unfunded_live_lane_reports_runner_present_without_fabricated_cost() {
    let report = run_routing_economics(7, false).expect("runner");
    assert_eq!(report.runner_status, RunnerStatus::RunnerPresent);
    assert_eq!(report.fixture_status, RunnerStatus::Measured);
    assert_eq!(report.live_cost, LIVE_COST_UNAVAILABLE);
    assert!(!report.live_lane_funded);
}

#[test]
fn same_seed_report_is_byte_identical() {
    let first = compose_routing_economics(1234, false).expect("first");
    let second = compose_routing_economics(1234, false).expect("second");
    assert_eq!(
        serde_json::to_vec(&first).expect("serialize first"),
        serde_json::to_vec(&second).expect("serialize second")
    );
}

#[test]
fn force_cloud_flag_selects_the_control_without_changing_the_comparison() {
    let hybrid_selected = run_routing_economics(99, false).expect("hybrid selected");
    let cloud_selected = run_routing_economics(99, true).expect("cloud selected");
    assert_eq!(hybrid_selected.selected_arm, "hybrid");
    assert_eq!(cloud_selected.selected_arm, "force_cloud_everything");
    assert_eq!(hybrid_selected.hybrid, cloud_selected.hybrid);
    assert_eq!(
        hybrid_selected.force_cloud_everything,
        cloud_selected.force_cloud_everything
    );
}

#[test]
fn routing_economics_beacon_is_queryable_and_recomputable() {
    assert!(SCHEMA_VERSION_PUBLIC >= 274);
    assert_eq!(
        ssb_action_type_from_str_public("routing_economics_measured")
            .expect("parse action type"),
        SsbActionType::RoutingEconomicsMeasured
    );

    let report = run_routing_economics(2026, false).expect("runner");
    let payload = payload_from_report(&report);
    let dir = tempfile::TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb.db"),
        pool_size: None,
        project_id: "k2-contract".to_string(),
        session_id: "k2-contract-session".to_string(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");

    runtime.block_on(async {
        let ssb = Ssb::new(cfg, clock).expect("ssb");
        ssb.write_payload(
            "contract:k2",
            0,
            SsbPayload::RoutingEconomicsMeasured(payload.clone()),
            Vec::new(),
        )
        .await
        .expect("write payload");
        let rows = ssb
            .query_by_type(SsbActionType::RoutingEconomicsMeasured)
            .expect("query beacon");
        assert_eq!(rows.len(), 1);
        let decoded: SsbPayload =
            rmp_serde::from_slice(&rows[0].payload).expect("decode beacon");
        assert_eq!(decoded, SsbPayload::RoutingEconomicsMeasured(payload));
    });

    let recomputed = run_routing_economics(report.seed, false).expect("recompute");
    let report_bytes = serde_json::to_vec(&report).expect("serialize report");
    let recomputed_bytes = serde_json::to_vec(&recomputed).expect("serialize recomputed");
    assert_eq!(report_bytes, recomputed_bytes);
    assert_eq!(
        blake3::hash(&report_bytes).to_hex().to_string(),
        payload_from_report(&report).report_blake3
    );
}
