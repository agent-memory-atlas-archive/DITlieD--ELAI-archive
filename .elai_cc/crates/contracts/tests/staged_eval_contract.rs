//! Contract tests for B-23 Phase 4H Group D — staged_eval.rs (4H.5).

use elai_orchestrator::{
    evaluate_stage, run_full_pipeline, AllowedConfigKey, BenchmarkRunner, DataTable, ParameterKey,
    PipelineMode, ProjectScope, ProposedChange, ReasoningTraceRef, SelfModProposal, StageOutcome,
    StageReport, StagedEvalError, StagedEvaluation, TestOutcomeReport, STAGE1_DEFAULT_TASKS,
    STAGE2_DEFAULT_TASKS, STAGE3_DEFAULT_TASKS,
};

fn class_b_proposal() -> SelfModProposal {
    SelfModProposal {
        variant_id: "v1".into(),
        parent_id: "v0".into(),
        // C-21 Task 7 — typed closed-enum ParameterKey.
        target_config_key: AllowedConfigKey::Parameter(ParameterKey::EtcThreshold),
        proposed_change: ProposedChange::UpdateKey {
            key: "etc_threshold".into(),
            old_value: "0.6".into(),
            new_value: "0.65".into(),
        },
        ssb_evidence: vec!["seq:1:reasoning_trace".into()],
        reasoning_trace_evidence: vec![ReasoningTraceRef {
            agent_id: "executor".into(),
            phase: "p".into(),
            grc_step_index: 0,
            step_type: "observe".into(),
            failure_mode: "stale".into(),
        }],
        rationale: "test".into(),
        scope: Some(ProjectScope {
            ecosystem: None,
            language: None,
            pipeline_mode: Some(PipelineMode::Surgical),
        }),
    }
}

fn no_regression() -> TestOutcomeReport {
    TestOutcomeReport {
        fail_to_pass: vec!["t1".into()],
        pass_to_pass: vec!["t2".into()],
        fail_to_fail: vec![],
        pass_to_fail: vec![],
    }
}

#[test]
fn pass_to_fail_at_stage1_discards_variant() {
    let r = StageReport {
        stage: StagedEvaluation::Stage1(STAGE1_DEFAULT_TASKS),
        score: 0.6,
        report: TestOutcomeReport {
            fail_to_pass: vec![],
            pass_to_pass: vec![],
            fail_to_fail: vec![],
            pass_to_fail: vec!["regressed".into()],
        },
        top_two: true,
        trigger_task_passed: true,
        safety_regression: false,
    };
    assert_eq!(evaluate_stage(&r).outcome, StageOutcome::DiscardStage1);
}

#[test]
fn stage3_promotion_strict_threshold() {
    // Score == 0.40 -> archive (strict greater-than).
    let r = StageReport {
        stage: StagedEvaluation::Stage3(STAGE3_DEFAULT_TASKS),
        score: 0.40,
        report: no_regression(),
        top_two: true,
        trigger_task_passed: true,
        safety_regression: false,
    };
    assert_eq!(evaluate_stage(&r).outcome, StageOutcome::ArchiveStage3);

    let mut promote = r.clone();
    promote.score = 0.41;
    assert_eq!(
        evaluate_stage(&promote).outcome,
        StageOutcome::PromoteActive
    );
}

#[test]
fn stage3_safety_regression_blocks_promotion() {
    let r = StageReport {
        stage: StagedEvaluation::Stage3(STAGE3_DEFAULT_TASKS),
        score: 0.55,
        report: no_regression(),
        top_two: true,
        trigger_task_passed: true,
        safety_regression: true,
    };
    assert_eq!(evaluate_stage(&r).outcome, StageOutcome::ArchiveStage3);
}

#[test]
fn stage2_below_threshold_archives() {
    let r = StageReport {
        stage: StagedEvaluation::Stage2(STAGE2_DEFAULT_TASKS),
        score: 0.30,
        report: no_regression(),
        top_two: true,
        trigger_task_passed: false,
        safety_regression: false,
    };
    let v = evaluate_stage(&r);
    assert_eq!(v.outcome, StageOutcome::ArchiveStage2);
    assert!(v.reason.contains("below threshold") || v.reason.contains("not top-2"));
}

struct AlwaysPass;
impl BenchmarkRunner for AlwaysPass {
    fn run_stage(
        &self,
        stage: StagedEvaluation,
        _: &SelfModProposal,
        _: PipelineMode,
    ) -> Result<StageReport, StagedEvalError> {
        Ok(StageReport {
            stage,
            score: 0.85,
            report: no_regression(),
            top_two: true,
            trigger_task_passed: true,
            safety_regression: false,
        })
    }
}

struct Stage1Crashes;
impl BenchmarkRunner for Stage1Crashes {
    fn run_stage(
        &self,
        stage: StagedEvaluation,
        _: &SelfModProposal,
        _: PipelineMode,
    ) -> Result<StageReport, StagedEvalError> {
        Ok(StageReport {
            stage,
            score: 0.5,
            report: TestOutcomeReport {
                fail_to_pass: vec![],
                pass_to_pass: vec![],
                fail_to_fail: vec![],
                pass_to_fail: vec!["crash".into()],
            },
            top_two: true,
            trigger_task_passed: false,
            safety_regression: false,
        })
    }
}

#[test]
fn full_pipeline_promotes_when_all_gates_pass() {
    let p = class_b_proposal();
    let v = run_full_pipeline(&AlwaysPass, &p, PipelineMode::Surgical).unwrap();
    assert_eq!(v.outcome, StageOutcome::PromoteActive);
}

#[test]
fn full_pipeline_short_circuits_on_stage1_pass_to_fail() {
    let p = class_b_proposal();
    let v = run_full_pipeline(&Stage1Crashes, &p, PipelineMode::Surgical).unwrap();
    assert_eq!(v.outcome, StageOutcome::DiscardStage1);
}

#[test]
fn class_a_rejected_at_entry() {
    let mut p = class_b_proposal();
    p.target_config_key = AllowedConfigKey::DataTable(DataTable::DeprecatedPatterns);
    p.proposed_change = ProposedChange::InsertRow {
        table: "deprecated_patterns".into(),
        row: vec![],
    };
    let err = run_full_pipeline(&AlwaysPass, &p, PipelineMode::Surgical).unwrap_err();
    assert!(matches!(err, StagedEvalError::ClassAProposalRejected));
}

/// C-21 Task 7: LOCKED_KEYS rejection migrated from runtime to COMPILE
/// time. `cloud_monthly_budget` has no variant in `ParameterKey`, so the
/// previous test construction does not compile. The compile-time check is
/// pinned by the c21_anti_stale_contract.rs trybuild fixture; the runtime
/// `is_locked` gate remains as defence-in-depth for JSON-decoded proposals.
#[test]
fn locked_parameter_check_runtime_consistent() {
    assert!(elai_orchestrator::is_locked("cloud_monthly_budget"));
    assert!(elai_orchestrator::is_locked("score_weights"));
    assert!(!elai_orchestrator::is_locked("etc_threshold"));
}

#[test]
fn mode_mismatch_rejected() {
    let mut p = class_b_proposal();
    p.scope = Some(ProjectScope {
        ecosystem: None,
        language: None,
        pipeline_mode: Some(PipelineMode::Greenfield),
    });
    let err = run_full_pipeline(&AlwaysPass, &p, PipelineMode::Surgical).unwrap_err();
    assert!(matches!(err, StagedEvalError::ModeMismatch { .. }));
}
