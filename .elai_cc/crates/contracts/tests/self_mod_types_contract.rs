//! Contract tests for B-23 Phase 4H Group A — self_mod_types.rs.
//!
//! Compile-level guard: every public type/enum the spec requires for
//! 4H.1 SKELETON must be reachable from `elai_orchestrator` re-exports.
//! Field-level invariants (score in [0,1], similarity formula weights, etc.)
//! land in algorithm-level groups (B/E). Group A only checks surface area.

use elai_orchestrator::{
    AcRegression, AllowedConfigKey, CatchLayer, ConfigVariant, ConfigVariantSimilarity,
    DataTable, DataTableValidation, DataTableVerdict, EvolutionGateResult, FullPipelineOutcome,
    OscVerdict, PipelineMode, ProjectScope, ProposedChange, ReasoningTraceRef, RegressionReport,
    SelfModProposal, StagedEvaluation, StaleApiCatch, StructuralChange, StructuralChangeType,
    TestOutcomeReport, UnclassifiedError, VariantStatus,
};
use std::path::PathBuf;

#[test]
fn pipeline_mode_has_oracle_variant() {
    // Phase-4H.txt:89,105 mandates SURGICAL | GREENFIELD | ORACLE.
    let modes = [
        PipelineMode::Surgical,
        PipelineMode::Greenfield,
        PipelineMode::Oracle,
    ];
    assert_eq!(modes.len(), 3);
    assert_eq!(PipelineMode::Oracle.as_str(), "oracle");
}

#[test]
fn test_outcome_report_has_four_categories() {
    // SWE-bench grading: F2P (resolution), P2P (maintenance), F2F (no fix),
    // P2F (REGRESSION). Phase-4H.txt:125-129.
    let report = TestOutcomeReport {
        fail_to_pass: vec!["t1".to_string()],
        pass_to_pass: vec!["t2".to_string()],
        fail_to_fail: vec![],
        pass_to_fail: vec![],
    };
    assert_eq!(report.fail_to_pass.len(), 1);
    assert_eq!(report.pass_to_pass.len(), 1);
    assert!(report.fail_to_fail.is_empty());
    assert!(report.pass_to_fail.is_empty());
}

#[test]
fn full_pipeline_outcome_has_self_mod_fields() {
    // Phase-4H.txt:86-103 — must carry pipeline_mode, etc_trigger_events,
    // stale_api_catches, context_budget_truncations, failure_taxonomy_misses.
    let outcome = FullPipelineOutcome {
        template_id: "t".into(),
        project_id: "p".into(),
        pipeline_mode: PipelineMode::Surgical,
        phases_total: 0,
        phases_passed_first_try: 0,
        audit_findings_total: 0,
        audit_findings_block: 0,
        test_coverage_final: 0.0,
        time_to_complete_seconds: 0,
        tokens_consumed_total: 0,
        human_interventions: 0,
        rules_triggered: vec![],
        test_outcomes: TestOutcomeReport::default(),
        etc_trigger_events: 0,
        stale_api_catches: vec![],
        context_budget_truncations: vec![],
        failure_taxonomy_misses: vec![],
    };
    assert_eq!(outcome.pipeline_mode, PipelineMode::Surgical);
}

#[test]
fn stale_api_catch_records_ecosystem_and_layer() {
    let catch = StaleApiCatch {
        file_path: PathBuf::from("foo.rs"),
        deprecated_name: "blake2".into(),
        replacement_name: "blake3".into(),
        ecosystem: "cargo".into(),
        caught_by: CatchLayer::Auditor,
    };
    assert_eq!(catch.ecosystem, "cargo");
    assert_eq!(catch.caught_by, CatchLayer::Auditor);
}

#[test]
fn unclassified_error_tracks_occurrences() {
    let err = UnclassifiedError {
        error_text: "foo".into(),
        file_path: PathBuf::from("a.rs"),
        baby_step_id: "bs1".into(),
        occurrence_count: 3,
    };
    assert!(err.occurrence_count >= 3, "Phase 3F self-discovery threshold");
}

#[test]
fn config_variant_carries_score_and_status() {
    let cv = ConfigVariant {
        variant_id: "v1".into(),
        parent_id: None,
        score: 0.5,
        children_count: 0,
        config_blob: vec![],
        created_at: 0,
        pipeline_outcomes: vec![],
        pipeline_mode: PipelineMode::Greenfield,
        status: VariantStatus::Active,
    };
    assert!((0.0..=1.0).contains(&cv.score));
    assert_eq!(cv.status, VariantStatus::Active);
}

#[test]
fn allowed_config_key_data_table_variants_present() {
    // Layer 1A's 4 DataTable variants (deprecated_patterns, FailureTaxonomy,
    // lazy-code, source-tiers) must remain reachable.
    for table in [
        DataTable::DeprecatedPatterns,
        DataTable::FailureTaxonomy,
        DataTable::LazyCodeRegex,
        DataTable::SourceTiers,
    ] {
        let key = AllowedConfigKey::DataTable(table);
        match key {
            AllowedConfigKey::DataTable(_) => (),
            AllowedConfigKey::Parameter(_) => panic!("expected DataTable"),
        }
    }
}

#[test]
fn proposed_change_has_data_and_param_variants() {
    let _a = ProposedChange::InsertRow {
        table: "deprecated_patterns".into(),
        row: vec![1, 2, 3],
    };
    let _b = ProposedChange::UpdateKey {
        key: "etc_threshold".into(),
        old_value: "0.6".into(),
        new_value: "0.65".into(),
    };
}

#[test]
fn project_scope_default_is_unscoped() {
    let scope = ProjectScope {
        ecosystem: None,
        language: None,
        pipeline_mode: None,
    };
    assert!(scope.ecosystem.is_none());
    assert!(scope.language.is_none());
    assert!(scope.pipeline_mode.is_none());
}

#[test]
fn staged_evaluation_three_stages() {
    // Phase-4H.txt:238 — Stage1(10) | Stage2(50) | Stage3(200).
    let stages = [
        StagedEvaluation::Stage1(10),
        StagedEvaluation::Stage2(50),
        StagedEvaluation::Stage3(200),
    ];
    assert_eq!(stages.len(), 3);
}

#[test]
fn data_table_validation_has_threshold_and_window() {
    let v = DataTableValidation {
        false_positive_count: 0,
        total_evaluations: 0,
        false_positive_rate: 0.0,
        threshold: 0.05,
        window: 10,
        verdict: DataTableVerdict::InsufficientData,
    };
    assert!((v.threshold - 0.05).abs() < 1e-6);
    assert_eq!(v.window, 10);
}

#[test]
fn structural_change_requires_human_approved() {
    let sc = StructuralChange {
        change_type: StructuralChangeType::AllowedConfigKeyAddition,
        description: "add new key".into(),
        human_approved: false,
    };
    assert!(!sc.human_approved, "Layer 3 default must require explicit approval");
}

#[test]
fn config_variant_similarity_weights_documented() {
    // Phase-4H.txt:259-260, ouroboros 0.5/0.3/0.2.
    let s = ConfigVariantSimilarity {
        key_overlap: 1.0,
        type_match: 1.0,
        exact_match: 1.0,
        similarity: 1.0,
    };
    assert!((s.similarity - (0.5 * s.key_overlap + 0.3 * s.type_match + 0.2 * s.exact_match)).abs() < 1e-6);
}

#[test]
fn osc_verdict_has_four_states() {
    let _ = [
        OscVerdict::Stagnant { window: 3, similarity: 0.95 },
        OscVerdict::Oscillating { period: 2 },
        OscVerdict::Evolving,
        OscVerdict::Inconclusive { reason: "ndata".into() },
    ];
}

#[test]
fn ac_regression_carries_consecutive_count() {
    let r = AcRegression {
        ac_index: 0,
        ac_text: "ac".into(),
        passed_in_pipeline: "p1".into(),
        failed_in_pipeline: "p5".into(),
        consecutive_failures: 3,
    };
    assert_eq!(r.consecutive_failures, 3);
}

#[test]
fn regression_report_default_has_no_regressions() {
    let r = RegressionReport::default();
    assert!(!r.has_regressions);
    assert!(r.regressions.is_empty());
}

#[test]
fn evolution_gate_has_allow_and_withhold() {
    let _ = [
        EvolutionGateResult::Allow { evolved_count: 4 },
        EvolutionGateResult::Withhold {
            reason: "ontology never evolved".into(),
            identical_streak: 5,
        },
    ];
}

#[test]
fn reasoning_trace_ref_carries_grc_step_index() {
    let r = ReasoningTraceRef {
        agent_id: "a".into(),
        phase: "p".into(),
        grc_step_index: 4,
        step_type: "observe".into(),
        failure_mode: "stale".into(),
    };
    assert_eq!(r.grc_step_index, 4);
}

#[test]
fn self_mod_proposal_carries_evidence_and_scope() {
    let prop = SelfModProposal {
        variant_id: "v1".into(),
        parent_id: "v0".into(),
        target_config_key: AllowedConfigKey::Parameter("etc_threshold".into()),
        proposed_change: ProposedChange::UpdateKey {
            key: "etc_threshold".into(),
            old_value: "0.6".into(),
            new_value: "0.65".into(),
        },
        ssb_evidence: vec!["ssb-1".into()],
        reasoning_trace_evidence: vec![],
        rationale: "phase ETC trigger excessive".into(),
        scope: Some(ProjectScope {
            ecosystem: Some("cargo".into()),
            language: Some("Rust".into()),
            pipeline_mode: Some(PipelineMode::Surgical),
        }),
    };
    assert!(!prop.ssb_evidence.is_empty(), "ssb_evidence required");
    assert!(prop.scope.is_some());
}
