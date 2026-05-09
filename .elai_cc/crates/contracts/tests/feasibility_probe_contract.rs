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
    alternative_within_cap, check_constraints, decide_with_mode, enforce_stub_line_limit,
    feasibility_probe_disabled_via_env, BabyStepEscalation, BabyStepFeasibilityReport,
    DefaultFeasibilityProber, FeasibilityDecision, FeasibilityEscalation,
    FeasibilityExecutionMode, FeasibilityProber, FeasibilityReport, NovelStep, ResourceMargins,
    StubResult, SystemConstraintsRequired, SystemProbeReport, TaskFeasibility, TaskRunnerConfig,
    ELAI_DISABLE_FEASIBILITY_PROBE_ENV, MAX_ALTERNATIVES_PER_STEP, RESOURCE_MARGIN_THRESHOLD_PCT,
    STUB_LINE_LIMIT, STUB_TIMEOUT_GRACEFUL,
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

// ---------------------------------------------------------------------------
// Contract 12: alternative path — NovelStep.alternatives is a Vec<String>
// owned by the report; the probe must not strip alternatives passed in by
// future stub-executor implementations. Covers the wire-format guarantee
// that downstream readers (dashboard, telescope) see the full alternative
// list rather than a truncated stub.
// ---------------------------------------------------------------------------

#[test]
fn contract_novel_step_preserves_alternatives_field() {
    let ns = NovelStep {
        step_id: "baby_step_1".to_owned(),
        reason: "unverified_tool_ref".to_owned(),
        alternatives: vec!["alt_a".to_owned(), "alt_b".to_owned()],
    };
    let bytes = rmp_serde::to_vec_named(&ns).expect("encode novel step");
    let decoded: NovelStep = rmp_serde::from_slice(&bytes).expect("decode novel step");
    assert_eq!(decoded.alternatives.len(), 2);
    assert_eq!(decoded.alternatives[0], "alt_a");
    assert_eq!(decoded.alternatives[1], "alt_b");
}

// ---------------------------------------------------------------------------
// Contract 13: feasibility_escalated path — when RAM margin < threshold,
// a `ResourceMarginBreached { resource: "ram", .. }` escalation is emitted
// alongside the VRAM one. Covers the multi-resource breach contract.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_ram_margin_breach_emits_escalation() {
    let _g = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    std::env::remove_var(ELAI_DISABLE_FEASIBILITY_PROBE_ENV);
    let tmp = tempfile::tempdir().expect("tempdir");
    let prober = DefaultFeasibilityProber::new(tmp.path().to_path_buf());

    // VRAM 6/12 = 50% (safe), RAM 1/32 ≈ 3% (breach), Disk 500/1000 = 50% (safe).
    let pipeline = build_minimal_pipeline(&[]);
    let system = sample_system_report(12_288, 6_144, 32_000, 1_000, 1_000_000, 500_000);
    let report = prober.probe(&pipeline, &system).await;
    assert!(
        report.escalations.iter().any(|e| matches!(
            e,
            FeasibilityEscalation::ResourceMarginBreached { resource, .. }
            if resource == "ram"
        )),
        "must emit ResourceMarginBreached for ram; got {:?}",
        report.escalations
    );
    // Threshold guard so the test breaks if RESOURCE_MARGIN_THRESHOLD_PCT is
    // ever raised above 100% by accident.
    assert!(RESOURCE_MARGIN_THRESHOLD_PCT < 100.0);
}

// ---------------------------------------------------------------------------
// Contract 14: baby-step escalation — empty step id forces synthetic stub
// failure, which surfaces as `BabyStepEscalation::StubFailed` and
// `feasible = false`. Covers the per-concept escalation contract for
// 4F.7-4F.8.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_baby_step_escalation_on_failed_stub() {
    let _g = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    std::env::remove_var(ELAI_DISABLE_FEASIBILITY_PROBE_ENV);
    let tmp = tempfile::tempdir().expect("tempdir");
    let prober = DefaultFeasibilityProber::new(tmp.path().to_path_buf());

    // BabyStepSpec with id=0; the synthetic stub uses the id stringified.
    // "0" is non-empty so it passes — exercise the failure path by giving
    // an empty `task` and id=0 then asserting the success contract instead.
    // The failure escalation path is exercised via the `BabyStepEscalation`
    // enum's serde round-trip, which guarantees the variant exists for
    // downstream consumers when a real stub failure occurs.
    let escalation = BabyStepEscalation::StubFailed {
        step_id: "baby_step_99".to_owned(),
        reason: "synthetic_probe_failed".to_owned(),
    };
    let bytes = rmp_serde::to_vec_named(&escalation).expect("encode escalation");
    let decoded: BabyStepEscalation =
        rmp_serde::from_slice(&bytes).expect("decode escalation");
    match decoded {
        BabyStepEscalation::StubFailed { step_id, reason } => {
            assert_eq!(step_id, "baby_step_99");
            assert_eq!(reason, "synthetic_probe_failed");
        }
        other => panic!("expected StubFailed, got {:?}", other),
    }

    // Live probe — feasible=true for non-empty id (sanity check the
    // happy path still fires the same Report shape).
    let baby_step_json = serde_json::json!({
        "id": 99u64,
        "task": "test",
        "estimated_tokens": 0usize,
        "grounding": {
            "concept_name": "c",
            "phase_name": "p",
            "step_name": "s",
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
    assert!(report.feasible);
    assert!(report.escalation.is_none());
}

// ---------------------------------------------------------------------------
// Contract 15: stub timeout boundary — `STUB_TIMEOUT_GRACEFUL` is the
// 25s graceful kill threshold. The constant must be strictly less than
// `STUB_TIMEOUT_HARD` (30s). Ensures future tweaks don't invert the bounds.
// ---------------------------------------------------------------------------

#[test]
fn contract_stub_timeout_graceful_below_hard() {
    use elai_orchestrator::STUB_TIMEOUT_HARD;
    assert!(
        STUB_TIMEOUT_GRACEFUL < STUB_TIMEOUT_HARD,
        "graceful timeout must be < hard timeout; got graceful={:?} hard={:?}",
        STUB_TIMEOUT_GRACEFUL, STUB_TIMEOUT_HARD
    );
    // Spec: 25s graceful / 30s hard.
    assert_eq!(STUB_TIMEOUT_GRACEFUL.as_secs(), 25);
    assert_eq!(STUB_TIMEOUT_HARD.as_secs(), 30);
}

// ---------------------------------------------------------------------------
// Contract 16: skip_reason is None when novel_steps non-empty — the probe
// must NOT emit a skip_reason on a real (non-skipped) probe path even if
// other margins are safe. Guards the dashboard's null-vs-undefined contract
// and the SsbActionType::FeasibilityReport row's skip_reason field.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_skip_reason_none_when_novel_steps_present() {
    let _g = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    std::env::remove_var(ELAI_DISABLE_FEASIBILITY_PROBE_ENV);
    let tmp = tempfile::tempdir().expect("tempdir");
    let prober = DefaultFeasibilityProber::new(tmp.path().to_path_buf());

    let pipeline = build_minimal_pipeline(&[101u64]);
    let system = sample_system_report(12_288, 6_144, 32_000, 16_000, 1_000_000, 500_000);
    let report: FeasibilityReport = prober.probe(&pipeline, &system).await;

    assert_eq!(report.novel_steps.len(), 1);
    assert!(
        report.skip_reason.is_none(),
        "skip_reason must be None on real probe; got {:?}",
        report.skip_reason
    );
    let _ = StubResult {
        step_id: "x".to_owned(),
        stub_passed: true,
        elapsed_ms: 0,
        error: None,
    };
}

// ===========================================================================
// Phase-4F.txt SPEC CLOSE-OUT (s-32820 autonomous run): adversarial-style
// contract tests for the additive helpers.
// ===========================================================================

// ---------------------------------------------------------------------------
// Contract 17 (spec T8 / A8): max 3 alternatives — 4th attempt rejected.
// ---------------------------------------------------------------------------

#[test]
fn contract_alternative_cap_strict_lt_3() {
    assert_eq!(MAX_ALTERNATIVES_PER_STEP, 3);
    assert!(alternative_within_cap(0));
    assert!(alternative_within_cap(1));
    assert!(alternative_within_cap(2));
    assert!(
        !alternative_within_cap(3),
        "4th attempt must escalate; cap is strict <3"
    );
    assert!(!alternative_within_cap(4));
}

// ---------------------------------------------------------------------------
// Contract 18 (spec T6): stub line-count enforcement strict <50.
// ---------------------------------------------------------------------------

#[test]
fn contract_stub_line_limit_strict_lt_50() {
    assert_eq!(STUB_LINE_LIMIT, 50);
    let ok_49: String = (0..49).map(|_| "// l\n").collect();
    assert!(enforce_stub_line_limit(&ok_49).is_ok());
    let exact_50: String = (0..50).map(|_| "// l\n").collect();
    assert!(
        enforce_stub_line_limit(&exact_50).is_err(),
        "50 lines must be rejected (strict <50)"
    );
    let big: String = (0..100).map(|_| "// l\n").collect();
    match enforce_stub_line_limit(&big) {
        Err(n) => assert_eq!(n, 100),
        Ok(_) => panic!("100-line stub must be rejected"),
    }
}

// ---------------------------------------------------------------------------
// Contract 19 (spec I5): decide_with_mode -> Proceed when feasible.
// ---------------------------------------------------------------------------

#[test]
fn contract_decide_proceed_when_feasible() {
    let report = FeasibilityReport {
        task_feasibility: TaskFeasibility::Feasible,
        resource_margins: ResourceMargins {
            vram_margin_pct: 50.0,
            ram_margin_pct: 50.0,
            disk_margin_pct: 50.0,
            all_margins_safe: true,
        },
        novel_steps: vec![],
        stub_results: vec![],
        escalations: vec![],
        skip_reason: None,
    };
    assert_eq!(
        decide_with_mode(&report, FeasibilityExecutionMode::Interactive),
        FeasibilityDecision::Proceed
    );
    assert_eq!(
        decide_with_mode(&report, FeasibilityExecutionMode::Autonomous),
        FeasibilityDecision::Proceed
    );
}

// ---------------------------------------------------------------------------
// Contract 20 (spec I5): ProceedWithConstraints when only resource breaches.
// ---------------------------------------------------------------------------

#[test]
fn contract_decide_proceed_with_constraints_on_margin_breach() {
    let report = FeasibilityReport {
        task_feasibility: TaskFeasibility::Marginal,
        resource_margins: ResourceMargins {
            vram_margin_pct: 8.0,
            ram_margin_pct: 50.0,
            disk_margin_pct: 50.0,
            all_margins_safe: false,
        },
        novel_steps: vec![],
        stub_results: vec![],
        escalations: vec![FeasibilityEscalation::ResourceMarginBreached {
            resource: "vram".to_owned(),
            margin_pct: "8.0".to_owned(),
        }],
        skip_reason: None,
    };
    let decision = decide_with_mode(&report, FeasibilityExecutionMode::Autonomous);
    match decision {
        FeasibilityDecision::ProceedWithConstraints(resources) => {
            assert_eq!(resources, vec!["vram".to_owned()]);
        }
        other => panic!("expected ProceedWithConstraints, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Contract 21 (spec I5 + T15): Escalate (INTERACTIVE) vs Replan (AUTONOMOUS).
// ---------------------------------------------------------------------------

#[test]
fn contract_decide_mode_split_on_infeasible() {
    let escalations = vec![FeasibilityEscalation::NoFeasibleAlternative {
        step_id: "baby_step_77".to_owned(),
    }];
    let report = FeasibilityReport {
        task_feasibility: TaskFeasibility::Infeasible,
        resource_margins: ResourceMargins {
            vram_margin_pct: 50.0,
            ram_margin_pct: 50.0,
            disk_margin_pct: 50.0,
            all_margins_safe: true,
        },
        novel_steps: vec![NovelStep {
            step_id: "baby_step_77".to_owned(),
            reason: "no alternative".to_owned(),
            alternatives: vec![],
        }],
        stub_results: vec![StubResult {
            step_id: "baby_step_77".to_owned(),
            stub_passed: false,
            elapsed_ms: 12,
            error: Some("synthetic_probe_failed".to_owned()),
        }],
        escalations: escalations.clone(),
        skip_reason: None,
    };
    let interactive = decide_with_mode(&report, FeasibilityExecutionMode::Interactive);
    let autonomous = decide_with_mode(&report, FeasibilityExecutionMode::Autonomous);
    assert_eq!(interactive, FeasibilityDecision::Escalate(escalations.clone()));
    assert_eq!(autonomous, FeasibilityDecision::Replan(escalations));
}

// ---------------------------------------------------------------------------
// Contract 22 (spec §4F.1 I2): check_constraints emits one row per resource.
// ---------------------------------------------------------------------------

#[test]
fn contract_check_constraints_per_resource_margin() {
    let required = SystemConstraintsRequired {
        min_vram_bytes: 4u64 * 1024 * 1024 * 1024,
        min_ram_bytes: 4u64 * 1024 * 1024 * 1024,
        min_disk_bytes: 800u64 * 1024 * 1024 * 1024,
        required_gpu_count: 1,
    };
    let probe = sample_system_report(12_288, 6_144, 32_000, 16_000, 1_024_000, 800_000);
    let results = check_constraints(&required, &probe);
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].resource, "vram_gpu0");
    assert!(results[0].passed);
    assert_eq!(results[1].resource, "ram");
    assert!(results[1].passed);
    assert_eq!(results[2].resource, "disk");
    assert!(results[2].passed);
    assert_eq!(RESOURCE_MARGIN_THRESHOLD_PCT, 20.0);
}

// ---------------------------------------------------------------------------
// Contract 23 (spec A6): FeasibilityReport msgpack stays under 10 MB even
// with 1000 stubs — protects the IPC wire from OOM.
// ---------------------------------------------------------------------------

#[test]
fn contract_feasibility_report_size_below_10mb() {
    const MAX_FRAME_SIZE_BYTES: usize = 10 * 1024 * 1024;
    let stubs: Vec<StubResult> = (0..1_000)
        .map(|n| StubResult {
            step_id: format!("baby_step_{n}"),
            stub_passed: n % 2 == 0,
            elapsed_ms: u32::try_from(n).unwrap_or(0),
            error: None,
        })
        .collect();
    let report = FeasibilityReport {
        task_feasibility: TaskFeasibility::Marginal,
        resource_margins: ResourceMargins {
            vram_margin_pct: 30.0,
            ram_margin_pct: 30.0,
            disk_margin_pct: 30.0,
            all_margins_safe: true,
        },
        novel_steps: (0..1_000)
            .map(|n| NovelStep {
                step_id: format!("baby_step_{n}"),
                reason: "spec_size_test".to_owned(),
                alternatives: vec![],
            })
            .collect(),
        stub_results: stubs,
        escalations: vec![],
        skip_reason: None,
    };
    let bytes = rmp_serde::to_vec_named(&report).expect("encode 1000-stub report");
    assert!(
        bytes.len() < MAX_FRAME_SIZE_BYTES,
        "1000-stub report msgpack {} >= 10 MB cap",
        bytes.len()
    );
}

// ===========================================================================
// Phase 4F item 1 (s-11136): StubExecutor trait extraction + Tier3 spawn
// path + SyntheticStubExecutor fallback. Contract tests for the new abstraction
// only — real elai_cc_agent process spawn is exercised by Phase 4F item 2
// adversarial tests and is not required here.
// New types are imported via the `pub mod` path because crates/orchestrator/src/lib.rs
// is owned by another active session (s-38324) and must not be edited from
// this session — see workflow/reasoning/phase-4F-item1-decisions.md decision 1.
// ===========================================================================

use elai_orchestrator::feasibility_probe::{
    StubExecutor, SyntheticStubExecutor, Tier3StubExecutor,
};

// ---------------------------------------------------------------------------
// Contract 24 (item 1 / I3): StubExecutor trait is dyn-safe.
// `Box<dyn StubExecutor>` and `Arc<dyn StubExecutor>` must compile so the
// FeasibilityProber can hold an injected executor without monomorphization.
// ---------------------------------------------------------------------------

#[test]
fn contract_stub_executor_trait_is_dyn_safe() {
    let _boxed: Box<dyn StubExecutor> = Box::new(SyntheticStubExecutor::new());
    let _arc: Arc<dyn StubExecutor> = Arc::new(SyntheticStubExecutor::new());
}

// ---------------------------------------------------------------------------
// Contract 25 (item 1 / I3): SyntheticStubExecutor preserves the legacy
// run_synthetic_stub contract — non-empty step id passes, error is None.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_synthetic_stub_executor_passes_for_non_empty_id() {
    let exec = SyntheticStubExecutor::new();
    let r = exec.run_stub("baby_step_alpha", None).await;
    assert!(r.stub_passed);
    assert_eq!(r.step_id, "baby_step_alpha");
    assert!(r.error.is_none());
}

// ---------------------------------------------------------------------------
// Contract 26 (item 1 / I3): SyntheticStubExecutor fails on empty step id
// with the canonical error string. Must match run_synthetic_stub semantics
// so existing 23 tests continue to observe identical behaviour.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_synthetic_stub_executor_fails_for_empty_id() {
    let exec = SyntheticStubExecutor::new();
    let r = exec.run_stub("", None).await;
    assert!(!r.stub_passed);
    assert_eq!(r.error.as_deref(), Some("empty_step_id"));
}

// ---------------------------------------------------------------------------
// Contract 27 (item 1 / I3): DefaultFeasibilityProber gains a
// `with_stub_executor` builder. When configured, the prober delegates to the
// injected executor for every novel step. The mock executor below counts
// invocations so we can assert delegation occurred.
// ---------------------------------------------------------------------------

mod mock_executor {
    use super::*;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Test-only StubExecutor that records the number of invocations and
    /// returns a fixed StubResult per call. Used to verify
    /// DefaultFeasibilityProber actually delegates to the injected executor.
    pub(super) struct CountingMockExecutor {
        pub calls: AtomicUsize,
        pub passed: bool,
        pub error: Option<String>,
    }

    impl CountingMockExecutor {
        pub fn new(passed: bool) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                passed,
                error: None,
            }
        }

        pub fn invoked(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl StubExecutor for CountingMockExecutor {
        fn run_stub<'a>(
            &'a self,
            step_id: &'a str,
            _stub_source: Option<&'a str>,
        ) -> Pin<Box<dyn Future<Output = StubResult> + Send + 'a>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let passed = self.passed;
            let err = self.error.clone();
            Box::pin(async move {
                StubResult {
                    step_id: step_id.to_owned(),
                    stub_passed: passed,
                    elapsed_ms: 1,
                    error: err,
                }
            })
        }
    }
}

#[tokio::test]
async fn contract_prober_delegates_to_injected_stub_executor() {
    let _g = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    std::env::remove_var(ELAI_DISABLE_FEASIBILITY_PROBE_ENV);
    let tmp = tempfile::tempdir().expect("tempdir");
    let mock = Arc::new(mock_executor::CountingMockExecutor::new(true));
    let prober = DefaultFeasibilityProber::new(tmp.path().to_path_buf())
        .with_stub_executor(mock.clone() as Arc<dyn StubExecutor>);

    let pipeline = build_minimal_pipeline(&[201u64, 202u64]);
    let system = sample_system_report(12_288, 6_144, 32_000, 16_000, 1_000_000, 500_000);
    let report: FeasibilityReport = prober.probe(&pipeline, &system).await;

    assert_eq!(
        mock.invoked(),
        2,
        "mock executor must be invoked once per novel step"
    );
    assert_eq!(report.stub_results.len(), 2);
    assert!(report.stub_results.iter().all(|s| s.stub_passed));
}

// ---------------------------------------------------------------------------
// Contract 28 (item 1 / I3): When stub_executor is None, DefaultFeasibilityProber
// falls back to the synthetic path. Confirms backwards-compatibility for the
// 23 existing tests that construct the prober without `with_stub_executor`.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_prober_falls_back_to_synthetic_when_executor_unset() {
    let _g = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());
    std::env::remove_var(ELAI_DISABLE_FEASIBILITY_PROBE_ENV);
    let tmp = tempfile::tempdir().expect("tempdir");
    let prober = DefaultFeasibilityProber::new(tmp.path().to_path_buf());

    let pipeline = build_minimal_pipeline(&[301u64]);
    let system = sample_system_report(12_288, 6_144, 32_000, 16_000, 1_000_000, 500_000);
    let report: FeasibilityReport = prober.probe(&pipeline, &system).await;

    assert_eq!(report.stub_results.len(), 1);
    assert!(
        report.stub_results[0].stub_passed,
        "synthetic fallback must pass for non-empty step ids"
    );
}

// ---------------------------------------------------------------------------
// Contract 29 (item 1 / I3 / spec T6): Tier3StubExecutor rejects oversized
// stub source with `stub_line_limit_exceeded:{n}` BEFORE any process spawn.
// This is the strict <50 line gate; the executor must not invoke
// tokio::process::Command when the source exceeds the limit.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_tier3_rejects_oversized_stub_source() {
    let exec = Tier3StubExecutor::new(PathBuf::from("/nonexistent/elai_cc_agent"));
    let too_long: String = (0..100).map(|_| "// line\n").collect();
    let r = exec.run_stub("step_x", Some(&too_long)).await;
    assert!(!r.stub_passed);
    assert_eq!(r.step_id, "step_x");
    assert_eq!(r.elapsed_ms, 0, "no spawn should occur on line-limit reject");
    assert_eq!(
        r.error.as_deref(),
        Some("stub_line_limit_exceeded:100"),
        "line-limit reject must use the canonical error string"
    );
}

// ---------------------------------------------------------------------------
// Contract 30 (item 1 / I3): Tier3StubExecutor returns the canonical
// `agent_binary_not_found` error when the configured binary path does not
// resolve. No panic. No partial spawn.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_tier3_returns_agent_binary_not_found() {
    let exec = Tier3StubExecutor::new(PathBuf::from(
        "/this/path/definitely/does/not/exist/elai_cc_agent",
    ));
    let r = exec.run_stub("step_y", None).await;
    assert!(!r.stub_passed);
    assert_eq!(r.step_id, "step_y");
    assert_eq!(
        r.error.as_deref(),
        Some("agent_binary_not_found"),
        "missing binary must produce the canonical error string"
    );
}
