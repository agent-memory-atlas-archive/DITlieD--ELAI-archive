//! D-19 observable_effect smoke test.
//!
//! Plan WIRING_CONTRACT observable_effect:
//!
//!   "invokes the CLI with --hook mock against the small fixture,
//!    asserts JSON report shape + accuracy>=0.5 (mock hook returns
//!    consistent score)."
//!
//! Strategy: this test exercises the orchestrator-side runner
//! (`PRMBenchRunner::run_all`) directly, since the CLI dispatcher
//! (`bench_prmbench::run`) calls `std::process::exit` and the cli crate
//! is binary-only (no lib target to import). The CLI's own unit tests
//! (in `src/bench_prmbench.rs::tests`) cover the report-shape + category
//! filter + threshold-override semantics in-process. Together the two
//! test surfaces close the producer/consumer loop without spawning a
//! subprocess.
//!
//! Two additional scenarios (full-fixture round-trip + per-category
//! filter) lock the contract surface against future regressions.

use std::path::PathBuf;
use std::sync::Arc;

use elai_orchestrator::benchmarks::prmbench_runner::{
    GatingVerdict, PRMBenchRunner, VecPrmBenchSsbEmitter,
};
use elai_orchestrator::sidecar_ppm_hook::MockSidecarPpmHook;

/// Locate the bundled mini-fixture relative to the cli crate's
/// `CARGO_MANIFEST_DIR`. The fixture lives at the repo root
/// (`.elai_cc/benchmarks/prmbench/`), so we walk up two parents from
/// `crates/cli/`.
fn bundled_mini_fixture() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("..")
        .join("..")
        .join("benchmarks")
        .join("prmbench")
        .join("prmbench_mini_fixture.jsonl")
}

#[tokio::test]
async fn observable_effect_mock_hook_against_mini_fixture() {
    let fixture = bundled_mini_fixture();
    assert!(
        fixture.exists(),
        "expected bundled mini-fixture at {}",
        fixture.display()
    );
    // Mock with forced score 0.85 -> every candidate predicted "correct";
    // ground-truth mix is 50% correct / 50% incorrect -> aggregate
    // accuracy lands at exactly 0.5 (the lower bound the plan asks for).
    let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.85));
    let runner = PRMBenchRunner::new(hook, fixture).expect("ctor");
    let emitter = VecPrmBenchSsbEmitter::new();
    let report = runner.run_all(&emitter).await.expect("run ok");

    // Shape assertions — every published field is populated.
    assert!(report.total >= 1, "expected at least one fixture row");
    assert!(
        (0.0..=1.0).contains(&report.accuracy),
        "accuracy in [0, 1]"
    );
    assert!(
        (0.0..=1.0).contains(&report.precision),
        "precision in [0, 1]"
    );
    assert!((0.0..=1.0).contains(&report.recall), "recall in [0, 1]");
    assert_eq!(
        report.fixture_blake3.len(),
        64,
        "fixture_blake3 is 64 hex chars"
    );
    assert!(
        report.per_category.len() >= 1,
        "at least one category surfaced"
    );

    // Accuracy floor per the plan WIRING_CONTRACT observable_effect.
    assert!(
        report.accuracy >= 0.5,
        "expected accuracy >= 0.5 with mock forced score 0.85, got {}",
        report.accuracy
    );

    // Exactly one SSB row is emitted.
    assert_eq!(emitter.len().await, 1, "exactly one SSB row per run_all");
    let rows = emitter.rows().await;
    let row = &rows[0];
    assert_eq!(row.total, report.total);
    assert_eq!(row.fixture_blake3, report.fixture_blake3);
    assert_eq!(row.gating_verdict, report.gating_verdict.as_str());
    assert!(row.category_count >= 1);
}

#[tokio::test]
async fn observable_effect_per_category_filter_runs_one_category() {
    let fixture = bundled_mini_fixture();
    let hook = Arc::new(MockSidecarPpmHook::with_forced_score(0.85));
    let runner = PRMBenchRunner::new(hook, fixture).expect("ctor");
    let emitter = VecPrmBenchSsbEmitter::new();
    let report = runner
        .run_category("deception_resistance", &emitter)
        .await
        .expect("run ok");

    assert_eq!(report.per_category.len(), 1);
    assert_eq!(report.per_category[0].category, "deception_resistance");
    assert!(report.per_category[0].total >= 1);
    assert_eq!(emitter.len().await, 1);
}

#[tokio::test]
async fn observable_effect_unavailable_hook_marks_insufficient_and_fails() {
    let fixture = bundled_mini_fixture();
    let hook = Arc::new(MockSidecarPpmHook::always_unavailable());
    let runner = PRMBenchRunner::new(hook, fixture).expect("ctor");
    let emitter = VecPrmBenchSsbEmitter::new();
    let report = runner.run_all(&emitter).await.expect("run ok");
    // Every row gets bucketed as `insufficient` -> aggregate scored = 0 ->
    // accuracy = 0 -> gate fails. This is the desired behaviour: a sidecar
    // outage cannot silently report Pass.
    assert_eq!(report.gating_verdict, GatingVerdict::Fail);
    for cat in &report.per_category {
        assert_eq!(cat.correct_predictions, 0);
        assert!(cat.insufficient >= 1);
    }
    assert_eq!(emitter.len().await, 1);
}
