//! Contract tests for B-23 Phase 4H Group C — bridge from FullPipelineOutcome
//! to Layer 1A DataTableProposal stream + scope enforcement on apply
//! + DEFAULT_MAX_FPR pinning + REG pre-screen stub.

use elai_core::clock::OsClock;
use elai_orchestrator::{
    proposals_from_full_outcome, regression_pre_screen, AcRegression, AllowedConfigKey,
    AppliedProposalRecord, CatchLayer, DataTable, DataTableProposal, DefaultLayer1aProcessor,
    FullPipelineOutcome, Layer1aError, Layer1aProcessor, PipelineMode, ProjectScope,
    ProposalSource, RegressionReport, Ssb, SsbConfig, StaleApiCatch, TestOutcomeReport,
    UnclassifiedError, DEFAULT_MAX_FPR,
};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex as AsyncMutex;

const _: () = {
    // Compile-time pin: DEFAULT_MAX_FPR may not drift without a spec
    // change. Spec phase-4H.txt:245 + 460 lock the threshold at 0.05.
    // (Group C audit BLOCK fix corrected this from the batch-3C draft 0.10.)
    assert!(DEFAULT_MAX_FPR == 0.05);
};

struct Fixture {
    _tmp: TempDir,
    processor: DefaultLayer1aProcessor,
}

async fn fixture_with_eco(active_ecosystem: Option<&str>) -> Fixture {
    let tmp = tempfile::tempdir().expect("tempdir");
    let rules_db_path: PathBuf = tmp.path().join("rules.db");
    let conn = Connection::open(&rules_db_path).expect("open rules.db");
    let rules_db = Arc::new(AsyncMutex::new(conn));
    let ssb_path = tmp.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path: ssb_path,
        pool_size: Some(2),
        project_id: "proj".into(),
        session_id: "sess".into(),
    };
    let ssb = Arc::new(Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init"));
    let mut processor = DefaultLayer1aProcessor::new(rules_db, rules_db_path, ssb);
    if let Some(eco) = active_ecosystem {
        processor = processor.with_ecosystem(eco);
    }
    processor.ensure_tables().await.expect("ensure_tables");
    Fixture { _tmp: tmp, processor }
}

fn dt_proposal(scope: Option<ProjectScope>) -> DataTableProposal {
    DataTableProposal {
        proposal_id: "scope-test".into(),
        table: DataTable::FailureTaxonomy,
        new_row: serde_json::json!({"category": "novel_error", "pattern": "MISSING_FLAG_QQQ"}),
        source: ProposalSource::Manual,
        fpr_estimate: 0.0,
        occurrences_seen: 3,
        pipeline_mode: PipelineMode::Surgical,
        scope,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scope_mismatch_rejects_cross_ecosystem_proposal() {
    let f = fixture_with_eco(Some("cargo")).await;
    let proposal = dt_proposal(Some(ProjectScope {
        ecosystem: Some("npm".into()),
        language: Some("TypeScript".into()),
        pipeline_mode: Some(PipelineMode::Surgical),
    }));
    let err = f.processor.apply_proposal(&proposal).await.unwrap_err();
    match err {
        Layer1aError::ScopeMismatch { expected, actual } => {
            assert_eq!(expected, "npm");
            assert_eq!(actual, "cargo");
        }
        other => panic!("expected ScopeMismatch, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scope_match_allows_apply() {
    let f = fixture_with_eco(Some("cargo")).await;
    let proposal = dt_proposal(Some(ProjectScope {
        ecosystem: Some("cargo".into()),
        language: Some("Rust".into()),
        pipeline_mode: Some(PipelineMode::Surgical),
    }));
    f.processor.apply_proposal(&proposal).await.expect("apply");
    // Round-trip: scope persisted on AppliedProposalRecord.
    let applied: Vec<AppliedProposalRecord> = f.processor.list_applied(10).await;
    assert_eq!(applied.len(), 1);
    let scope = applied[0].scope.as_ref().expect("scope persisted");
    assert_eq!(scope.ecosystem.as_deref(), Some("cargo"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unscoped_proposal_passes_through_any_active_ecosystem() {
    let f = fixture_with_eco(Some("cargo")).await;
    let proposal = dt_proposal(None);
    f.processor.apply_proposal(&proposal).await.expect("apply unscoped");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn no_active_ecosystem_skips_scope_check() {
    // Legacy callers that never call with_ecosystem() must not see false rejections.
    let f = fixture_with_eco(None).await;
    let proposal = dt_proposal(Some(ProjectScope {
        ecosystem: Some("npm".into()),
        language: None,
        pipeline_mode: None,
    }));
    f.processor.apply_proposal(&proposal).await.expect("apply with no active eco");
}

// ---------------------------------------------------------------------------
// Bridge contracts
// ---------------------------------------------------------------------------

fn bare_outcome(mode: PipelineMode) -> FullPipelineOutcome {
    FullPipelineOutcome {
        template_id: "t".into(),
        project_id: "p".into(),
        pipeline_mode: mode,
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
    }
}

#[test]
fn bridge_builds_deprecated_patterns_with_scope() {
    let mut outcome = bare_outcome(PipelineMode::Surgical);
    for _ in 0..3 {
        outcome.stale_api_catches.push(StaleApiCatch {
            file_path: PathBuf::from("src/lib.rs"),
            deprecated_name: "blake2".into(),
            replacement_name: "blake3".into(),
            ecosystem: "cargo".into(),
            caught_by: CatchLayer::Auditor,
        });
    }
    let props = proposals_from_full_outcome(&outcome, 3);
    assert_eq!(props.len(), 1);
    assert_eq!(props[0].table, DataTable::DeprecatedPatterns);
    let scope = props[0].scope.as_ref().expect("scope set");
    assert_eq!(scope.ecosystem.as_deref(), Some("cargo"));
    assert_eq!(scope.language.as_deref(), Some("Rust"));
    assert_eq!(props[0].source, ProposalSource::AutoFromOutcome);
}

#[test]
fn bridge_failure_taxonomy_threshold_uses_summed_count() {
    let mut outcome = bare_outcome(PipelineMode::Greenfield);
    outcome.failure_taxonomy_misses.push(UnclassifiedError {
        error_text: "MISSING_FLAG_X".into(),
        file_path: PathBuf::from("src/a.rs"),
        baby_step_id: "bs1".into(),
        occurrence_count: 2,
    });
    outcome.failure_taxonomy_misses.push(UnclassifiedError {
        error_text: "MISSING_FLAG_X".into(),
        file_path: PathBuf::from("src/b.rs"),
        baby_step_id: "bs2".into(),
        occurrence_count: 1,
    });
    let props = proposals_from_full_outcome(&outcome, 3);
    assert_eq!(props.len(), 1);
    assert_eq!(props[0].table, DataTable::FailureTaxonomy);
    let scope = props[0].scope.as_ref().expect("mode-only scope per audit fix");
    assert!(scope.ecosystem.is_none(), "failure_taxonomy stays language-agnostic on ecosystem");
    assert!(scope.language.is_none());
    assert_eq!(scope.pipeline_mode, Some(PipelineMode::Greenfield));
    assert_eq!(props[0].pipeline_mode, PipelineMode::Greenfield);
}

#[test]
fn bridge_below_threshold_no_emit() {
    let mut outcome = bare_outcome(PipelineMode::Surgical);
    outcome.stale_api_catches.push(StaleApiCatch {
        file_path: PathBuf::from("src/lib.rs"),
        deprecated_name: "blake2".into(),
        replacement_name: "blake3".into(),
        ecosystem: "cargo".into(),
        caught_by: CatchLayer::Auditor,
    });
    let props = proposals_from_full_outcome(&outcome, 3);
    assert!(props.is_empty());
}

#[test]
fn default_max_fpr_pinned_at_0_05() {
    // Runtime guard alongside the const_assert. Spec phase-4H.txt:245+460
    // lock the threshold at 5%; FPR threshold is a locked key (audit BLOCK-2).
    assert!((DEFAULT_MAX_FPR - 0.05).abs() < 1e-6);
}

#[test]
fn datatable_only_proposal_cannot_target_max_fpr() {
    // Sanity: the AllowedConfigKey enum has only DataTable + Parameter
    // variants. The "max_fpr" field is NOT a DataTable, so a DataTable
    // proposal carrying it never reaches the apply path with side effects.
    let key = AllowedConfigKey::DataTable(DataTable::FailureTaxonomy);
    match key {
        AllowedConfigKey::DataTable(_) => (),
        AllowedConfigKey::Parameter(_) => panic!("DataTable variant only"),
    }
}

#[test]
fn regression_pre_screen_passes_clean_report() {
    let report = RegressionReport::default();
    assert!(regression_pre_screen(&report));
}

#[test]
fn regression_pre_screen_blocks_on_regression() {
    let report = RegressionReport::from_regressions(vec![AcRegression {
        ac_index: 0,
        ac_text: "ac".into(),
        passed_in_pipeline: "p1".into(),
        failed_in_pipeline: "p5".into(),
        consecutive_failures: 1,
    }]);
    assert!(!regression_pre_screen(&report));
}
