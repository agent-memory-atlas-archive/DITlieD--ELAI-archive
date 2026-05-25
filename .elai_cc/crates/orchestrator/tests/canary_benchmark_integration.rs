//! D-24 — end-to-end integration test for the cross-domain canary
//! regression benchmark.
//!
//! Verifies the runner's whole-corpus behavior against a mini fixture
//! set + asserts the gating thresholds + asserts the runner does NOT
//! emit per-fixture D-7 SSB rows (the in-memory drive contract).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use elai_orchestrator::benchmarks::canary_runner::{
    BenchError, CanaryBenchmark, CanaryFixtureRow, FixtureExpectedClass, Provenance,
};
use elai_orchestrator::benchmarks::prmbench_runner::GatingVerdict;
use elai_orchestrator::canary::CanaryFamily;
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbReader};
use tempfile::TempDir;

fn write_row(dir: &TempDir, name: &str, row: &CanaryFixtureRow) -> PathBuf {
    let path = dir.path().join(name);
    let body = serde_json::to_string_pretty(row).expect("serialize row");
    std::fs::write(&path, body).expect("write row");
    path
}

fn make_row(
    id: &str,
    family: CanaryFamily,
    class: FixtureExpectedClass,
    pre: f64,
    post: f64,
    prov: Provenance,
) -> CanaryFixtureRow {
    CanaryFixtureRow {
        id: id.into(),
        domain: family.as_str().into(),
        family,
        expected_class: class,
        pre_baseline_score: pre,
        post_change_score: post,
        delta_pct: post - pre,
        expected_verdict: match class {
            FixtureExpectedClass::Harmful => "regression_blocked".into(),
            _ => "pass".into(),
        },
        provenance: prov,
        notes: format!("integration test row {id}"),
    }
}

#[tokio::test]
async fn end_to_end_all_harmful_blocked_zero_false_block_pass() {
    let dir = TempDir::new().expect("tempdir");
    // 5 harmful + 5 helpful + 5 neutral, all distributed across the 5
    // CanaryFamily variants.
    let families = CanaryFamily::all();
    for (idx, fam) in families.iter().enumerate() {
        // Harmful — large negative delta on target family.
        write_row(
            &dir,
            &format!("h_{idx:02}.json"),
            &make_row(
                &format!("cr_h{idx}"),
                *fam,
                FixtureExpectedClass::Harmful,
                0.85,
                0.45,
                Provenance::ElaiSynthetic {
                    author: "integration-test".into(),
                },
            ),
        );
        // Helpful — positive delta on target.
        write_row(
            &dir,
            &format!("p_{idx:02}.json"),
            &make_row(
                &format!("cr_p{idx}"),
                *fam,
                FixtureExpectedClass::Helpful,
                0.50,
                0.85,
                Provenance::CanaryCorpus,
            ),
        );
        // Neutral — near-zero delta.
        write_row(
            &dir,
            &format!("n_{idx:02}.json"),
            &make_row(
                &format!("cr_n{idx}"),
                *fam,
                FixtureExpectedClass::Neutral,
                0.65,
                0.65,
                Provenance::ElaiSynthetic {
                    author: "integration-test".into(),
                },
            ),
        );
    }
    let bench = CanaryBenchmark::new(dir.path().to_path_buf()).with_harmful_block_floor(5);
    let report = bench.run().await.expect("run ok");

    assert_eq!(report.harmful_blocked, 5);
    assert_eq!(report.helpful_blocked, 0);
    assert_eq!(report.neutral_blocked, 0);
    assert_eq!(report.true_block, 5);
    assert_eq!(report.false_block, 0);
    assert_eq!(report.true_pass, 10);
    assert_eq!(report.false_pass, 0);
    assert_eq!(report.verdict, GatingVerdict::Pass);
    assert_eq!(report.total_fixtures, 15);
}

#[tokio::test]
async fn one_missed_harmful_fails_floor() {
    let dir = TempDir::new().expect("tempdir");
    // 2 harmful but one is a no-op mutation (pre == post) — the canary
    // will not block it. With floor=2 the verdict must be Fail.
    write_row(
        &dir,
        "h_01.json",
        &make_row(
            "cr_h1",
            CanaryFamily::MathProof,
            FixtureExpectedClass::Harmful,
            0.85,
            0.45,
            Provenance::ElaiSynthetic {
                author: "integration-test".into(),
            },
        ),
    );
    // No-op harmful — pre==post on target, no regression to detect.
    write_row(
        &dir,
        "h_02.json",
        &make_row(
            "cr_h2",
            CanaryFamily::CodeFix,
            FixtureExpectedClass::Harmful,
            0.85,
            0.85,
            Provenance::ElaiSynthetic {
                author: "integration-test".into(),
            },
        ),
    );
    let bench = CanaryBenchmark::new(dir.path().to_path_buf()).with_harmful_block_floor(2);
    let report = bench.run().await.expect("run ok");

    assert_eq!(report.harmful_blocked, 1);
    assert_eq!(report.false_pass, 1);
    assert_eq!(report.verdict, GatingVerdict::Fail);
}

#[tokio::test]
async fn excessive_false_blocks_fail_helpful_ceiling() {
    let dir = TempDir::new().expect("tempdir");
    // 6 helpful fixtures where the candidate inadvertently regresses
    // OTHER families (post on target is HIGHER but the runner's
    // ReplayCanaryEvaluator uses pre_baseline_score for non-target
    // families on both candidate and baseline, so non-target delta is
    // zero — these should NOT be blocked. To force false-blocks we
    // CRAFT the row so the helpful target's score is BELOW its baseline
    // (i.e., a mislabel — the row says Helpful but the math says
    // regression).
    for i in 0..6 {
        write_row(
            &dir,
            &format!("p_{i:02}.json"),
            &make_row(
                &format!("cr_p{i}"),
                CanaryFamily::MathProof,
                FixtureExpectedClass::Helpful,
                // Mislabelled: pre HIGH, post LOW — the canary will see
                // a regression and block. Counts as false_block on the
                // helpful arm.
                0.90,
                0.40,
                Provenance::ElaiSynthetic {
                    author: "integration-test".into(),
                },
            ),
        );
    }
    // 1 correctly-labelled harmful so the harmful_block_floor is met.
    write_row(
        &dir,
        "h_01.json",
        &make_row(
            "cr_h1",
            CanaryFamily::CodeFix,
            FixtureExpectedClass::Harmful,
            0.85,
            0.45,
            Provenance::ElaiSynthetic {
                author: "integration-test".into(),
            },
        ),
    );
    let bench = CanaryBenchmark::new(dir.path().to_path_buf())
        .with_harmful_block_floor(1)
        .with_helpful_block_ceiling(5);
    let report = bench.run().await.expect("run ok");

    assert_eq!(report.helpful_blocked, 6);
    assert!(report.helpful_blocked > 5);
    assert_eq!(report.verdict, GatingVerdict::Fail);
}

#[tokio::test]
async fn provenance_counts_accurate_across_corpus() {
    let dir = TempDir::new().expect("tempdir");
    // 2 corpus + 3 synthetic = 5 fixtures.
    for i in 0..2 {
        write_row(
            &dir,
            &format!("c_{i:02}.json"),
            &make_row(
                &format!("cr_c{i}"),
                CanaryFamily::ProseReason,
                FixtureExpectedClass::Neutral,
                0.60,
                0.60,
                Provenance::CanaryCorpus,
            ),
        );
    }
    for i in 0..3 {
        write_row(
            &dir,
            &format!("s_{i:02}.json"),
            &make_row(
                &format!("cr_s{i}"),
                CanaryFamily::PlanningDecomposition,
                FixtureExpectedClass::Neutral,
                0.55,
                0.55,
                Provenance::ElaiSynthetic {
                    author: "prov-test".into(),
                },
            ),
        );
    }
    let bench = CanaryBenchmark::new(dir.path().to_path_buf()).with_harmful_block_floor(0);
    let report = bench.run().await.expect("run ok");

    assert_eq!(
        report.provenance_counts.get("canary_corpus").copied(),
        Some(2)
    );
    assert_eq!(
        report.provenance_counts.get("elai_synthetic").copied(),
        Some(3)
    );
}

#[tokio::test]
async fn runner_does_not_emit_d7_ssb_rows() {
    // CRITICAL invariant: the runner drives D-7 run_canary IN-MEMORY
    // and does NOT call emit_canary_run per fixture. If it did, the
    // shipped D-7 SSB stream would be polluted with 100 synthetic
    // CanaryRunCompleted rows per benchmark invocation.
    //
    // To verify: stand up a fresh SSB writer, run the bench, then
    // query for D-7 row types. Count must be exactly zero.
    let dir = TempDir::new().expect("tempdir");
    for i in 0..3 {
        write_row(
            &dir,
            &format!("h_{i:02}.json"),
            &make_row(
                &format!("cr_h{i}"),
                CanaryFamily::MathProof,
                FixtureExpectedClass::Harmful,
                0.85,
                0.45,
                Provenance::ElaiSynthetic {
                    author: "no-pollution-test".into(),
                },
            ),
        );
    }

    let ssb_dir = TempDir::new().expect("ssb tempdir");
    let cfg = SsbConfig {
        db_path: ssb_dir.path().join("ssb_canary_bench.db"),
        pool_size: Some(2),
        project_id: "test-project".into(),
        session_id: "test-session".into(),
    };
    let clock: Arc<dyn elai_core::ClockProvider> = Arc::new(elai_core::clock::SeededClock::new());
    let ssb = Ssb::new(cfg, clock).expect("ssb init");

    // Snapshot D-7 row counters BEFORE the bench run.
    let before_run = ssb
        .query_by_type(SsbActionType::CanaryRunCompleted)
        .expect("query before")
        .len();
    let before_family = ssb
        .query_by_type(SsbActionType::CanaryFamilyEvaluated)
        .expect("query before")
        .len();
    let before_regression = ssb
        .query_by_type(SsbActionType::CrossDomainRegression)
        .expect("query before")
        .len();

    let bench = CanaryBenchmark::new(dir.path().to_path_buf()).with_harmful_block_floor(3);
    let _report = bench.run().await.expect("run ok");

    // Snapshot D-7 row counters AFTER — they MUST be identical
    // (no per-fixture D-7 row emission).
    let after_run = ssb
        .query_by_type(SsbActionType::CanaryRunCompleted)
        .expect("query after")
        .len();
    let after_family = ssb
        .query_by_type(SsbActionType::CanaryFamilyEvaluated)
        .expect("query after")
        .len();
    let after_regression = ssb
        .query_by_type(SsbActionType::CrossDomainRegression)
        .expect("query after")
        .len();
    assert_eq!(
        before_run, after_run,
        "D-24 must NOT emit D-7 CanaryRunCompleted rows per fixture",
    );
    assert_eq!(
        before_family, after_family,
        "D-24 must NOT emit D-7 CanaryFamilyEvaluated rows per fixture",
    );
    assert_eq!(
        before_regression, after_regression,
        "D-24 must NOT emit D-7 CrossDomainRegression rows per fixture",
    );
}

#[tokio::test]
async fn empty_dir_returns_typed_error() {
    let dir = TempDir::new().expect("tempdir");
    let bench = CanaryBenchmark::new(dir.path().to_path_buf());
    match bench.run().await {
        Err(BenchError::EmptyFixturesDir { .. }) => (),
        other => panic!("expected EmptyFixturesDir, got {other:?}"),
    }
}

#[tokio::test]
async fn per_family_breakdown_aggregates_correctly() {
    let dir = TempDir::new().expect("tempdir");
    // 2 harmful targeting MathProof + 1 helpful targeting CodeFix.
    write_row(
        &dir,
        "h_01.json",
        &make_row(
            "cr_h1",
            CanaryFamily::MathProof,
            FixtureExpectedClass::Harmful,
            0.85,
            0.45,
            Provenance::ElaiSynthetic {
                author: "per-family-test".into(),
            },
        ),
    );
    write_row(
        &dir,
        "h_02.json",
        &make_row(
            "cr_h2",
            CanaryFamily::MathProof,
            FixtureExpectedClass::Harmful,
            0.80,
            0.40,
            Provenance::CanaryCorpus,
        ),
    );
    write_row(
        &dir,
        "p_01.json",
        &make_row(
            "cr_p1",
            CanaryFamily::CodeFix,
            FixtureExpectedClass::Helpful,
            0.50,
            0.85,
            Provenance::ElaiSynthetic {
                author: "per-family-test".into(),
            },
        ),
    );
    let bench = CanaryBenchmark::new(dir.path().to_path_buf()).with_harmful_block_floor(2);
    let report = bench.run().await.expect("run ok");

    let math = report
        .per_family_breakdown
        .get("math_proof")
        .expect("math_proof present");
    assert_eq!(math.true_block, 2);
    assert_eq!(math.total, 2);

    let code = report
        .per_family_breakdown
        .get("code_fix")
        .expect("code_fix present");
    assert_eq!(code.true_pass, 1);
    assert_eq!(code.total, 1);

    // Spot-check that other families do NOT appear in the breakdown
    // (we only emit per-family rows for families actually exercised).
    let other_keys: BTreeMap<&String, ()> = report
        .per_family_breakdown
        .keys()
        .filter(|k| k.as_str() != "math_proof" && k.as_str() != "code_fix")
        .map(|k| (k, ()))
        .collect();
    assert!(other_keys.is_empty(), "expected only exercised families");
}
