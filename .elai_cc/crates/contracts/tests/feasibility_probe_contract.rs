//! Batch 3A (Phase 4F) — FeasibilityProbe contract tests.
//!
//! Covers:
//!   1. `FeasibilityProber` trait is dyn-safe (`Arc<dyn FeasibilityProber>` holds).
//!   2. Skip criteria: zero novel + all margins >= 20% -> `skip_reason`
//!      populated, `stub_results` empty.
//!   3. Resource-margin breach: <20% VRAM -> `ResourceMarginBreached` escalation.
//!   4. Novel step probe: at least one stub_result per novel step.
//!   5. Baby-step probe: returns a well-formed `BabyStepFeasibilityReport`.
//!   6. `stub_results.msgpack` is written to disk.
//!   7. `ELAI_DISABLE_FEASIBILITY_PROBE` short-circuits `probe()`.
//!   8. Absence (None) preserves current behaviour: `TaskRunnerConfig::new_minimal`
//!      leaves `feasibility_prober = None`.
//!   9. `FeasibilityReportPayload` / `NovelStepDetectedPayload` / `StubRunPayload`
//!      round-trip through msgpack.
//!  10. Action-type strings resolve.

use std::path::PathBuf;
use std::sync::Arc;

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, FeasibilityReportPayload, NovelStepDetectedPayload,
    SsbActionType, SsbPayload, StubRunPayload,
};
use elai_orchestrator::{
    feasibility_probe_disabled_via_env, BabyStepFeasibilityReport, DefaultFeasibilityProber,
    FeasibilityEscalation, FeasibilityProber, FeasibilityReport, ResourceMargins, StubResult,
    SystemProbeReport, TaskFeasibility, TaskRunnerConfig, ELAI_DISABLE_FEASIBILITY_PROBE_ENV,
    RESOURCE_MARGIN_THRESHOLD_PCT,
};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn sample_system_report(
    vram_total: u32,
    vram_free: u32,
    ram_total: u64,
    ram_free: u64,
    disk_total: u64,
    disk_free: u64,
) -> SystemProbeReport {
    use elai_orchestrator::{DetectionMethod, GpuInfo};
    use elai_core::VramTier;
    SystemProbeReport {
        gpu: Some(GpuInfo {
            name: "test-gpu".to_owned(),
            vram_total_mb: vram_total,
            vram_free_mb: vram_free,
            tier: VramTier::Vram12Gb,
            driver_version: None,
        }),
        ram_total_mb: ram_total,
        ram_free_mb: ram_free,
        disk_total_mb: disk_total,
        disk_free_mb: disk_free,
        detection_method: DetectionMethod::ShellOut,
        probed_at_ms: 1_700_000_000_000,
    }
}

/// Build a minimal `ExecutablePipeline` via `serde_json::from_value`. The
/// struct's fields are all `pub` + `Deserialize` so a JSON fixture is the
/// lowest-friction way to avoid the real `PipelineBuilder::build` surface
/// (which requires Tool/Model/Predicate mocks). Field names mirror
/// `ExecutablePipeline` in crates/orchestrator/src/pipeline_builder.rs:256.
fn build_minimal_pipeline(novel_ids: &[u64]) -> elai_orchestrator::ExecutablePipeline {
    use elai_orchestrator::ExecutablePipeline;
    let ids_json: Vec<serde_json::Value> = novel_ids
        .iter()
        .map(|id| serde_json::json!(id))
        .collect();
    let json = serde_json::json!({
        "pipeline_id": 1u64,
        "spec": {
            "name": "test",
            "steps": [],
            "output_type": "Code",
            "perf": {
                "max_latency": { "secs": 10, "nanos": 0 },
                "max_memory_bytes": 0u64,
                "max_tokens": 0u64
            },
            "media": []
        },
        "fsm_states": [],
        "dependency_graph": { "nodes": [], "edges": [] },
        "novel_step_ids": ids_json,
        "agent_configs": [],
        "rule_configs": [],
        "resource_estimate": {
            "total_vram_mb": 0u64,
            "peak_vram_mb": 0u64,
            "estimated_disk_mb": 0u64,
            "estimated_duration_secs": 0u64,
            "fits_hardware": true
        },
        "test_harness": {
            "test_cases": [],
            "fixture_stubs": []
        },
        "repair_log": [],
        "output_dir": "/tmp/feasibility_test"
    });
    let pipeline: ExecutablePipeline = serde_json::from_value(json)
        .expect("construct minimal ExecutablePipeline via JSON fixture");
    pipeline
}

// ---------------------------------------------------------------------------
// Contract 1: trait is dyn-safe
// ---------------------------------------------------------------------------

#[test]
fn contract_feasibility_prober_trait_is_dyn_safe() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let prober: Arc<dyn FeasibilityProber> =
        Arc::new(DefaultFeasibilityProber::new(tmp.path().to_path_buf()));
    assert!(Arc::strong_count(&prober) >= 1);
}

// ---------------------------------------------------------------------------
// Contract 2: skip criteria
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_skip_criteria_zero_novel_and_safe_margins() {
    let _g = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    std::env::remove_var(ELAI_DISABLE_FEASIBILITY_PROBE_ENV);
    let tmp = tempfile::tempdir().expect("tempdir");
    let prober = DefaultFeasibilityProber::new(tmp.path().to_path_buf());

    let pipeline = build_minimal_pipeline(&[]);
    // VRAM 6/12 = 50% free, RAM 16/32 = 50% free, Disk 500/1000 = 50% free.
    let system = sample_system_report(12_288, 6_144, 32_000, 16_000, 1_000_000, 500_000);
    let report: FeasibilityReport = prober.probe(&pipeline, &system).await;

    assert_eq!(report.task_feasibility, TaskFeasibility::Feasible);
    assert_eq!(
        report.skip_reason.as_deref(),
        Some("no_novel_steps_margins_ok")
    );
    assert!(report.stub_results.is_empty());
    assert!(report.novel_steps.is_empty());
    assert!(report.escalations.is_empty());
}

// ---------------------------------------------------------------------------
// Contract 3: resource margin breach
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_resource_margin_breach_raises_escalation() {
    let _g = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    std::env::remove_var(ELAI_DISABLE_FEASIBILITY_PROBE_ENV);
    let tmp = tempfile::tempdir().expect("tempdir");
    let prober = DefaultFeasibilityProber::new(tmp.path().to_path_buf());

    // VRAM 1/12 = 8% free => below threshold.
    let pipeline = build_minimal_pipeline(&[]);
    let system = sample_system_report(12_288, 1_000, 32_000, 16_000, 1_000_000, 500_000);
    let report = prober.probe(&pipeline, &system).await;
    assert!(
        report.escalations.iter().any(|e| matches!(
            e,
            FeasibilityEscalation::ResourceMarginBreached { resource, .. }
            if resource == "vram"
        )),
        "must emit ResourceMarginBreached for vram; got {:?}",
        report.escalations
    );
}

// ---------------------------------------------------------------------------
// Contract 4: novel step probe — at least one stub_result per novel step
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_novel_step_emits_stub_result() {
    let _g = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    std::env::remove_var(ELAI_DISABLE_FEASIBILITY_PROBE_ENV);
    let tmp = tempfile::tempdir().expect("tempdir");
    let prober = DefaultFeasibilityProber::new(tmp.path().to_path_buf());

    let pipeline = build_minimal_pipeline(&[42u64, 43u64]);
    // Margins safe so skip doesn't trigger.
    let system = sample_system_report(12_288, 6_144, 32_000, 16_000, 1_000_000, 500_000);
    let report = prober.probe(&pipeline, &system).await;
    assert_eq!(report.novel_steps.len(), 2);
    assert_eq!(report.stub_results.len(), 2);
    // Synthetic stub returns stub_passed=true for non-empty ids.
    for stub in &report.stub_results {
        assert!(stub.stub_passed, "synthetic stub passes for non-empty id");
    }
}

// ---------------------------------------------------------------------------
// Contract 5: baby-step probe shape
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_baby_step_probe_well_formed() {
    let _g = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    std::env::remove_var(ELAI_DISABLE_FEASIBILITY_PROBE_ENV);
    let tmp = tempfile::tempdir().expect("tempdir");
    let prober = DefaultFeasibilityProber::new(tmp.path().to_path_buf());

    // Construct a BabyStepSpec via its public JSON shape.
    let baby_step_json = serde_json::json!({
        "id": 7u64,
        "task": "test baby step",
        "estimated_tokens": 100usize,
        "grounding": {
            "concept_name": "concept-1",
            "phase_name": "phase-1",
            "step_name": "step-1",
            "top_citations": [],
            "compressed_tokens": 0usize
        },
        "tdd": null,
        "acceptance": null
    });
    let baby_step: elai_orchestrator::research_tree::BabyStepSpec =
        serde_json::from_value(baby_step_json).expect("baby step spec");

    let system = sample_system_report(12_288, 6_144, 32_000, 16_000, 1_000_000, 500_000);
    let report: BabyStepFeasibilityReport =
        prober.probe_baby_step(&baby_step, &system).await;
    assert_eq!(report.step_id, "7");
    assert!(report.feasible);
    assert_eq!(report.stub_results.len(), 1);
    assert!(report.escalation.is_none());
}

// ---------------------------------------------------------------------------
// Contract 6: stub_results.msgpack written to disk
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_stub_results_msgpack_is_written() {
    let _g = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    std::env::remove_var(ELAI_DISABLE_FEASIBILITY_PROBE_ENV);
    let tmp = tempfile::tempdir().expect("tempdir");
    let root: PathBuf = tmp.path().to_path_buf();
    let prober = DefaultFeasibilityProber::new(root.clone());

    let pipeline = build_minimal_pipeline(&[77u64]);
    let system = sample_system_report(12_288, 6_144, 32_000, 16_000, 1_000_000, 500_000);
    let _report = prober.probe(&pipeline, &system).await;

    // Pipeline id hardcoded in fixture to 1.
    let expected = root.join("pipeline-1").join("stub_results.msgpack");
    assert!(
        expected.exists(),
        "stub_results.msgpack must be written at {}",
        expected.display()
    );
    let bytes = std::fs::read(&expected).expect("read stub_results.msgpack");
    let decoded: Vec<StubResult> =
        rmp_serde::from_slice(&bytes).expect("decode stub_results msgpack");
    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0].step_id, "77");
    assert!(decoded[0].stub_passed);
}

// ---------------------------------------------------------------------------
// Contract 7: env kill switch short-circuits probe()
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_env_kill_switch_short_circuits_probe() {
    let _g = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    std::env::set_var(ELAI_DISABLE_FEASIBILITY_PROBE_ENV, "1");
    assert!(feasibility_probe_disabled_via_env());
    let tmp = tempfile::tempdir().expect("tempdir");
    let prober = DefaultFeasibilityProber::new(tmp.path().to_path_buf());
    let pipeline = build_minimal_pipeline(&[1u64, 2u64]);
    let system = sample_system_report(12_288, 1_000, 32_000, 1_000, 1_000_000, 10_000);
    let report = prober.probe(&pipeline, &system).await;
    std::env::remove_var(ELAI_DISABLE_FEASIBILITY_PROBE_ENV);
    assert_eq!(report.task_feasibility, TaskFeasibility::Feasible);
    assert_eq!(report.skip_reason.as_deref(), Some("disabled_via_env"));
    assert!(report.novel_steps.is_empty());
    assert!(report.stub_results.is_empty());
}

// ---------------------------------------------------------------------------
// Contract 8: Absence preserves pre-3A behaviour
// ---------------------------------------------------------------------------

#[test]
fn contract_new_minimal_keeps_feasibility_prober_none() {
    let cfg = TaskRunnerConfig::new_minimal(PathBuf::from("/tmp/x.json"));
    assert!(cfg.feasibility_prober.is_none());
}

// ---------------------------------------------------------------------------
// Contract 9: payloads round-trip through msgpack
// ---------------------------------------------------------------------------

#[test]
fn contract_payloads_roundtrip_msgpack() {
    let fr = FeasibilityReportPayload {
        task_feasibility: "marginal".to_owned(),
        skip_reason: None,
        novel_step_count: 2,
        escalations_count: 1,
        vram_margin_bp: 1_900, // 19.00%
        ram_margin_bp: 5_000,
        disk_margin_bp: 7_500,
    };
    let ns = NovelStepDetectedPayload {
        step_id: "baby_step_1".to_owned(),
        reason: "unverified".to_owned(),
        alternative_count: 0,
    };
    let sr = StubRunPayload {
        step_id: "baby_step_1".to_owned(),
        passed: true,
        elapsed_ms: 12,
    };
    for payload in [
        SsbPayload::FeasibilityReport(fr.clone()),
        SsbPayload::NovelStepDetected(ns.clone()),
        SsbPayload::StubRun(sr.clone()),
    ] {
        let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
        let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
        assert_eq!(decoded.action_type(), payload.action_type());
    }
}

// ---------------------------------------------------------------------------
// Contract 10: action-type string mapping
// ---------------------------------------------------------------------------

#[test]
fn contract_action_types_resolve() {
    assert_eq!(SsbActionType::FeasibilityReport.as_sql_str(), "feasibility_report");
    assert_eq!(SsbActionType::NovelStepDetected.as_sql_str(), "novel_step_detected");
    assert_eq!(SsbActionType::StubRun.as_sql_str(), "stub_run");
    assert_eq!(
        ssb_action_type_from_str_public("feasibility_report").expect("resolves"),
        SsbActionType::FeasibilityReport
    );
    assert_eq!(
        ssb_action_type_from_str_public("novel_step_detected").expect("resolves"),
        SsbActionType::NovelStepDetected
    );
    assert_eq!(
        ssb_action_type_from_str_public("stub_run").expect("resolves"),
        SsbActionType::StubRun
    );
}

// ---------------------------------------------------------------------------
// Contract 11: ResourceMargins::from_report produces zero margins on
// missing totals (forces caller to escalate).
// ---------------------------------------------------------------------------

#[test]
fn contract_resource_margins_zero_on_missing_totals() {
    let mut r = sample_system_report(0, 0, 0, 0, 0, 0);
    r.gpu = None;
    let margins = ResourceMargins::from_report(&r);
    assert_eq!(margins.vram_margin_pct, 0.0);
    assert_eq!(margins.ram_margin_pct, 0.0);
    assert_eq!(margins.disk_margin_pct, 0.0);
    assert!(!margins.all_margins_safe);
    assert!(RESOURCE_MARGIN_THRESHOLD_PCT > 0.0);
}
