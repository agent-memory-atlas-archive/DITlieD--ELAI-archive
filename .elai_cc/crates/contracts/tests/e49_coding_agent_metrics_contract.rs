// E-49 — Coding-agent metric suite contract test (scope-lean ship).
//
// Locks the public surface added by this plan:
//   * elai_orchestrator::coding_metrics::types::CodingMetric (capture point)
//   * elai_orchestrator::coding_metrics::collector::MetricsCollector
//   * elai_orchestrator::coding_metrics::aggregator::aggregate_session_metrics
//   * elai_orchestrator::coding_metrics::swebench_pro::{run_swebench_pro,
//     SweBenchProConfig, SweBenchProError::RunpodPending}
//   * elai_orchestrator::coding_metrics::refinebench::{run_refinebench,
//     RefineBenchConfig, RefineBenchError::RunpodPending}
//   * 5 SSB action types round-trip via msgpack:
//       - SsbActionType::CodingAgentMetricsCaptured
//       - SsbActionType::SweBenchProRunStarted
//       - SsbActionType::SweBenchProRunCompleted
//       - SsbActionType::RefineBenchScoreEmitted
//       - SsbActionType::MetricsAggregationRun
//   * SCHEMA_VERSION_PUBLIC floor at 138.
//   * 10th lockstep site (TOTAL_VARIANTS) included via the
//     action_type_lock golden test in elai-orchestrator's own test
//     suite (cross-checked here only by the SsbActionType variants
//     existing and round-tripping).
//   * WIRE-DARK avoidance: metrics_cmd is the production caller anchor
//     for MetricsCollector + aggregate_session_metrics + the SWE-Bench
//     Pro / RefineBench dry-run boundaries.
//
// Design source: SWE-bench Pro (claimed arXiv:2509.16941) + RefineBench
// (claimed arXiv:2511.22173). Citations DEFERRED to mistakes.md
// cluster #9 (cite-deferral discipline) — the contract asserts
// wire-shape + module-shape + SSB round-trip + RUNPOD-PENDING boundary
// independently of paper grounding.

use std::collections::BTreeMap;
use std::path::PathBuf;

use elai_orchestrator::coding_metrics::{
    aggregate_session_metrics, run_refinebench, run_swebench_pro, AggregatorError, CodingMetric,
    MetricsCollector, RefineBenchConfig, RefineBenchError, SweBenchProConfig, SweBenchProError,
};
use elai_orchestrator::ssb::{
    CodingAgentMetricsCapturedPayload, MetricsAggregationRunPayload,
    RefineBenchScoreEmittedPayload, SsbActionType, SsbPayload, SweBenchProRunCompletedPayload,
    SweBenchProRunStartedPayload, SCHEMA_VERSION_PUBLIC,
};

// ---------------------------------------------------------------------------
// (F1) SCHEMA_VERSION_PUBLIC floor: E-49 atomic-bumped to 138.
// ---------------------------------------------------------------------------

#[test]
fn e49_f1_schema_version_floor_is_138_or_higher() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 138,
        "E-49 atomic-bump puts SCHEMA_VERSION_PUBLIC at >= 138, found {SCHEMA_VERSION_PUBLIC}"
    );
}

// ---------------------------------------------------------------------------
// (F2) CodingMetric shape + MetricsCollector capture path.
// ---------------------------------------------------------------------------

#[test]
fn e49_f2_coding_metric_empty_shape() {
    let m = CodingMetric::empty("s-x", 1_700_000_000_000);
    assert_eq!(m.session_id, "s-x");
    assert_eq!(m.refine_rate, 0.0);
    assert!(m.pass_at_k.is_empty());
    assert_eq!(m.cost_per_task_cents, None);
    assert_eq!(m.ts_ms, 1_700_000_000_000);
    assert!(!m.has_pass_samples());
    assert_eq!(m.pass_at_1(), None);
}

#[test]
fn e49_f2b_metrics_collector_capture_returns_zero_observation_snapshot() {
    let collector = MetricsCollector::new();
    let snap = collector
        .capture_metrics("s-contract")
        .expect("capture ok");
    assert_eq!(snap.session_id, "s-contract");
    assert!(snap.pass_at_k.is_empty());
    assert_eq!(snap.refine_rate, 0.0);
    assert_eq!(snap.cost_per_task_cents, None);
}

#[test]
fn e49_f2c_metrics_collector_pass_sample_path() {
    let mut collector = MetricsCollector::new();
    collector.record_pass_sample(1, 5, 10).expect("ok");
    let snap = collector.snapshot("s-cov", 1_700_000_000_000);
    let mut want_keys: BTreeMap<u32, ()> = BTreeMap::new();
    want_keys.insert(1u32, ());
    assert_eq!(
        snap.pass_at_k.keys().copied().collect::<Vec<_>>(),
        want_keys.keys().copied().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// (F3) Aggregator: exists + handles zero sessions + one session smoke.
// ---------------------------------------------------------------------------

#[test]
fn e49_f3_aggregator_rejects_zero_sessions() {
    let err = aggregate_session_metrics(&[]).unwrap_err();
    assert!(matches!(err, AggregatorError::NoSessions));
}

#[test]
fn e49_f3b_aggregator_smokes_one_session() {
    let snap = CodingMetric::empty("s-1", 1);
    let report = aggregate_session_metrics(&[snap]).expect("agg ok");
    assert_eq!(report.sessions.len(), 1);
    assert_eq!(report.refine_rate_avg, 0.0);
    assert_eq!(report.cost_per_task_total_cents, None);
}

// ---------------------------------------------------------------------------
// (F4) RUNPOD-PENDING markers: SWE-Bench Pro + RefineBench live runs.
// ---------------------------------------------------------------------------

#[test]
fn e49_f4_swebench_pro_live_run_is_runpod_pending() {
    let cfg = SweBenchProConfig::pending("pro-v1", "qwen2.5-coder-7b");
    let err = run_swebench_pro(&cfg).unwrap_err();
    assert!(
        matches!(err, SweBenchProError::RunpodPending),
        "SWE-Bench Pro live transport MUST be RUNPOD-PENDING in this ship"
    );
}

#[test]
fn e49_f4b_refinebench_live_run_is_runpod_pending() {
    let cfg = RefineBenchConfig::pending("rb-v1", "qwen2.5-coder-7b");
    let err = run_refinebench(&cfg).unwrap_err();
    assert!(
        matches!(err, RefineBenchError::RunpodPending),
        "RefineBench live transport MUST be RUNPOD-PENDING in this ship"
    );
}

// ---------------------------------------------------------------------------
// (F5) 5 SSB variants round-trip via msgpack.
// ---------------------------------------------------------------------------

#[test]
fn e49_f5_coding_agent_metrics_captured_round_trips() {
    let payload = SsbPayload::CodingAgentMetricsCaptured(CodingAgentMetricsCapturedPayload {
        session_id_hash: 0xDEAD_BEEF_0000_0049,
        pass_at_1_x10000: 7500,
        refine_rate_x10000: 6000,
        cost_per_task_cents: Some(42),
        ts_ms: 1_700_000_138_999,
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::CodingAgentMetricsCaptured
    );
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(payload, decoded);
}

#[test]
fn e49_f5b_swe_bench_pro_run_started_round_trips() {
    let payload = SsbPayload::SweBenchProRunStarted(SweBenchProRunStartedPayload {
        task_pool_version_hash: 0xCAFE_0049_0000_0001,
        model_id_hash: 0xCAFE_0049_0000_0002,
        max_tasks: Some(50),
        ts_ms: 1_700_000_138_001,
    });
    assert_eq!(payload.action_type(), SsbActionType::SweBenchProRunStarted);
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(payload, decoded);
}

#[test]
fn e49_f5c_swe_bench_pro_run_completed_round_trips() {
    let payload = SsbPayload::SweBenchProRunCompleted(SweBenchProRunCompletedPayload {
        task_pool_version_hash: 0xCAFE_0049_0000_0003,
        model_id_hash: 0xCAFE_0049_0000_0004,
        tasks_attempted: 50,
        tasks_solved: 33,
        pass_at_1_x10000: 6600,
        ts_ms: 1_700_000_138_002,
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::SweBenchProRunCompleted
    );
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(payload, decoded);
}

#[test]
fn e49_f5d_refinebench_score_emitted_round_trips() {
    let payload = SsbPayload::RefineBenchScoreEmitted(RefineBenchScoreEmittedPayload {
        suite_version_hash: 0xCAFE_0049_0000_0005,
        model_id_hash: 0xCAFE_0049_0000_0006,
        refine_rate_x10000: 5500,
        ts_ms: 1_700_000_138_003,
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::RefineBenchScoreEmitted
    );
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(payload, decoded);
}

#[test]
fn e49_f5e_metrics_aggregation_run_round_trips() {
    let payload = SsbPayload::MetricsAggregationRun(MetricsAggregationRunPayload {
        session_count: 3,
        pass_at_1_avg_x10000: 7000,
        refine_rate_avg_x10000: 5800,
        ts_ms: 1_700_000_138_004,
    });
    assert_eq!(payload.action_type(), SsbActionType::MetricsAggregationRun);
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(payload, decoded);
}

// ---------------------------------------------------------------------------
// (F6) WIRE-DARK avoidance: metrics_cmd has collector/aggregator anchors.
// ---------------------------------------------------------------------------

#[test]
fn e49_f6_metrics_cmd_has_collector_and_aggregator_call_sites() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // CARGO_MANIFEST_DIR -> .../crates/contracts. Sibling: crates/cli/src.
    let cli_src = workspace
        .parent()
        .and_then(|p| Some(p.join("cli").join("src").join("metrics_cmd.rs")))
        .expect("metrics_cmd path");
    let body = std::fs::read_to_string(&cli_src).expect("metrics_cmd.rs reads");
    assert!(
        body.contains("MetricsCollector"),
        "metrics_cmd.rs must reference MetricsCollector (WIRE-DARK anchor)"
    );
    assert!(
        body.contains("aggregate_session_metrics"),
        "metrics_cmd.rs must reference aggregate_session_metrics (WIRE-DARK anchor)"
    );
    assert!(
        body.contains("run_swebench_pro"),
        "metrics_cmd.rs must reference run_swebench_pro (RUNPOD-PENDING anchor)"
    );
    assert!(
        body.contains("run_refinebench"),
        "metrics_cmd.rs must reference run_refinebench (RUNPOD-PENDING anchor)"
    );
}
