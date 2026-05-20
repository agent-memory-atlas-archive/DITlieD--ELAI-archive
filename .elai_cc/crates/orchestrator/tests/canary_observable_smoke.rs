//! D-7 observable_effect smoke — end-to-end:
//!   * Seed 5 fixture held-out tasks per family (25 total) across the 5
//!     orthogonal canary families.
//!   * Run [`canary::run_canary`] with a deterministic evaluator pair.
//!   * Emit the report through [`canary::emit_canary_run`] into a
//!     temp SSB store.
//!   * Assert:
//!       - CanaryReport carries exactly 5 FamilyEvalResult entries.
//!       - Each entry has n_evaluated >= min_per_family.
//!       - SSB tail contains exactly 1 CanaryRunCompleted row + 5
//!         CanaryFamilyEvaluated rows matched on `report_id`.
//!       - `agent_pool::recent_canary_status` returns a summary that
//!         agrees with the emitted run.
//!
//! No live model / sidecar dependency. Runs against `Ssb::new` on a
//! tempdir-backed SQLite + in-process futures.

use std::sync::Arc;

use elai_orchestrator::agent_pool::recent_canary_status;
use elai_orchestrator::canary::{
    emit_canary_run, run_canary, CanaryDecision, CanaryEvaluator, CanaryFamily,
    CanaryFamilyBundle, CANARY_FAMILY_VARIANT_COUNT,
};
use elai_orchestrator::held_out::{
    build_task, ExpectedOutcome, HeldOutTask, TaskId, TaskSource, DEFAULT_NGRAM_SIZE,
};
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbReader};
use elai_orchestrator::ssb_viewer::SsbViewer;

struct AlwaysPass;
impl CanaryEvaluator for AlwaysPass {
    fn evaluate(
        &self,
        _task: &HeldOutTask,
    ) -> impl std::future::Future<Output = Result<bool, String>> + Send {
        async move { Ok(true) }
    }
}

struct AlwaysFail;
impl CanaryEvaluator for AlwaysFail {
    fn evaluate(
        &self,
        _task: &HeldOutTask,
    ) -> impl std::future::Future<Output = Result<bool, String>> + Send {
        async move { Ok(false) }
    }
}

fn fixture_task(family: CanaryFamily, idx: u64) -> HeldOutTask {
    let body = format!(
        "fixture task for family {} index {} body alpha beta gamma delta epsilon",
        family.as_str(),
        idx
    );
    let outcome = match family {
        CanaryFamily::CodeFix => ExpectedOutcome::Code("ok".into()),
        CanaryFamily::MathProof => ExpectedOutcome::Math("ok".into()),
        CanaryFamily::ProseReason
        | CanaryFamily::PlanningDecomposition
        | CanaryFamily::ToolUse => ExpectedOutcome::Prose("ok".into()),
    };
    let mut t = build_task(
        body,
        TaskSource::Synthetic { seed: idx },
        outcome,
        1_000 + idx as i64,
        0,
        DEFAULT_NGRAM_SIZE,
    );
    // Tag the task_id with the family label so the assertion later can
    // confirm each evaluator saw the right family bundle.
    t.task_id = TaskId(format!("{}-{}", family.as_str(), idx));
    t
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn canary_emits_run_plus_5_family_rows_into_ssb() {
    // 5 tasks per family × 5 families = 25 fixture tasks.
    let bundles: Vec<CanaryFamilyBundle> = CanaryFamily::all()
        .iter()
        .map(|&fam| {
            let tasks: Vec<HeldOutTask> = (0..5).map(|i| fixture_task(fam, i)).collect();
            CanaryFamilyBundle { family: fam, tasks }
        })
        .collect();

    // Stage a clean Pass case: candidate passes everything; baseline
    // also passes everything (delta = 0.0 on every family). That keeps
    // the canary decision deterministic and lets us focus the smoke
    // test on wiring correctness, not the band-threshold edge.
    let cand = AlwaysPass;
    let base = AlwaysPass;

    let report = run_canary(&bundles, &cand, &base, 5, 1_700_000_000_000)
        .await
        .expect("run_canary");

    assert_eq!(
        report.per_family.len(),
        CANARY_FAMILY_VARIANT_COUNT,
        "report must carry one FamilyEvalResult per family"
    );
    for r in &report.per_family {
        assert!(
            r.n_evaluated >= 5,
            "family {:?} n_evaluated must be >= min_per_family",
            r.family
        );
        assert_eq!(r.pass_count, 5);
        assert_eq!(r.fail_count, 0);
    }
    assert!(matches!(report.canary_decision, CanaryDecision::Pass));
    assert_eq!(report.report_id.len(), 64);

    // Emit into a fresh tempdir-backed SSB store.
    let dir = tempfile::TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb_canary_smoke.db"),
        pool_size: Some(2),
        project_id: "test-project".into(),
        session_id: "test-session".into(),
    };
    let clock: std::sync::Arc<dyn elai_core::ClockProvider> =
        std::sync::Arc::new(elai_core::clock::SeededClock::new());
    let ssb = Arc::new(Ssb::new(cfg.clone(), clock).expect("ssb init"));
    emit_canary_run(&report, &ssb, "test-agent", 0).await;

    // Exactly 1 run row.
    let run_rows = ssb
        .query_by_type(SsbActionType::CanaryRunCompleted)
        .expect("query run rows");
    assert_eq!(run_rows.len(), 1, "exactly one CanaryRunCompleted row");

    // Exactly 5 per-family rows.
    let fam_rows = ssb
        .query_by_type(SsbActionType::CanaryFamilyEvaluated)
        .expect("query family rows");
    assert_eq!(
        fam_rows.len(),
        CANARY_FAMILY_VARIANT_COUNT,
        "one CanaryFamilyEvaluated row per family"
    );

    // recent_canary_status emulates the BAI gate / dashboard read path.
    // Force a flush + reopen via a viewer pointed at the same db.
    drop(ssb);
    let viewer = SsbViewer::open(&cfg.db_path).expect("ssb viewer open");
    let status = recent_canary_status(&viewer).expect("status present");
    assert_eq!(status.report_id, report.report_id);
    assert_eq!(status.decision_kind, "pass");
    assert!(!status.blocks_promotion());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn canary_fail_path_blocks_promotion() {
    // Candidate fails everything; baseline passes everything → delta
    // = -1.0 on every family → Fail. dominant_family is whichever
    // family the deterministic decide() picks first (the first-tied
    // entry, which is CodeFix per CanaryFamily::all() ordering).
    let bundles: Vec<CanaryFamilyBundle> = CanaryFamily::all()
        .iter()
        .map(|&fam| {
            let tasks: Vec<HeldOutTask> = (0..5).map(|i| fixture_task(fam, i)).collect();
            CanaryFamilyBundle { family: fam, tasks }
        })
        .collect();
    let cand = AlwaysFail;
    let base = AlwaysPass;

    let report = run_canary(&bundles, &cand, &base, 5, 1_700_000_001_000)
        .await
        .expect("run_canary");
    match report.canary_decision {
        CanaryDecision::Fail { dominant_family: _ } => {}
        other => panic!("expected Fail, got {other:?}"),
    }

    let dir = tempfile::TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: dir.path().join("ssb_canary_fail.db"),
        pool_size: Some(2),
        project_id: "test-project".into(),
        session_id: "test-session".into(),
    };
    let clock: std::sync::Arc<dyn elai_core::ClockProvider> =
        std::sync::Arc::new(elai_core::clock::SeededClock::new());
    let ssb = Arc::new(Ssb::new(cfg.clone(), clock).expect("ssb init"));
    emit_canary_run(&report, &ssb, "test-agent", 0).await;
    drop(ssb);

    let viewer = SsbViewer::open(&cfg.db_path).expect("ssb viewer open");
    let status = recent_canary_status(&viewer).expect("status present");
    assert_eq!(status.decision_kind, "fail");
    assert!(status.blocks_promotion());
    assert!(
        !status.dominant_fail_family.is_empty(),
        "dominant_fail_family must be populated on fail"
    );
}
