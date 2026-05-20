//! D-21 observable_effect smoke test for the mode-routing coverage benchmark.
//!
//! Plan WIRING_CONTRACT observable_effect:
//!   trigger: `elai bench mode-routing`
//!   expect_ssb: ModeRoutingBenchRunCompleted row
//!   expect_artifact: report.json
//!
//! Strategy mirrors `bench_prmbench_smoke.rs`: the CLI dispatcher calls
//! `std::process::exit` and the cli crate is binary-only, so the test
//! exercises the orchestrator-side runner
//! (`ModeRoutingBenchmark::run_all`) directly against the bundled mini
//! fixture. The CLI's own unit tests (in `src/bench_mode_routing.rs::tests`)
//! cover the parser + threshold-override + render path.

use std::path::PathBuf;

use elai_orchestrator::benchmarks::mode_routing_runner::{
    ModeRoutingBenchmark, VecModeRoutingBenchEmitter,
};
use elai_orchestrator::benchmarks::prmbench_runner::GatingVerdict;

/// Locate the bundled mini-fixture relative to the cli crate's
/// `CARGO_MANIFEST_DIR`. The fixture lives at the repo root
/// (`.elai_cc/benchmarks/mode_routing/fixtures/`).
fn bundled_mini_fixture() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("..")
        .join("..")
        .join("benchmarks")
        .join("mode_routing")
        .join("fixtures")
        .join("mode_routing_mini.jsonl")
}

#[tokio::test]
async fn observable_effect_mini_fixture_passes_gate_and_emits_one_ssb_row() {
    let fixture = bundled_mini_fixture();
    assert!(
        fixture.exists(),
        "expected bundled mini-fixture at {}",
        fixture.display()
    );
    let runner = ModeRoutingBenchmark::new(fixture);
    let emitter = VecModeRoutingBenchEmitter::new();
    let report = runner.run_all(&emitter).await.expect("run ok");

    // Shape assertions — every published field is populated.
    assert!(report.total >= 1, "expected at least one fixture row");
    assert!((0.0..=1.0).contains(&report.accuracy), "accuracy in [0, 1]");
    assert!((0.0..=1.0).contains(&report.f1_macro), "f1_macro in [0, 1]");
    assert_eq!(
        report.fixture_blake3.len(),
        64,
        "fixture_blake3 is 64 hex chars"
    );
    assert_eq!(
        report.per_mode_f1.len(),
        5,
        "per_mode_f1 covers all 5 GroundingMode variants"
    );
    assert!(
        !report.per_category.is_empty(),
        "at least one category surfaced"
    );

    // Mini fixture is engineered to land every row on its expected mode ->
    // accuracy must clear the plan's observable_effect floor (0.5).
    assert!(
        report.accuracy >= 0.5,
        "expected accuracy >= 0.5 against the mini fixture, got {}",
        report.accuracy
    );

    // Mini fixture pre-engineering means the gate should pass; if a future
    // edit to the predicate regresses coverage this assertion catches it.
    assert_eq!(
        report.gating_verdict,
        GatingVerdict::Pass,
        "mini fixture is engineered to pass the macro F1 >= 0.85 + per-mode F1 >= 0.70 gate"
    );

    // Exactly one SSB row is emitted.
    assert_eq!(emitter.len().await, 1, "exactly one SSB row per run_all");
    let rows = emitter.rows().await;
    let row = &rows[0];
    assert_eq!(row.total, report.total);
    assert_eq!(row.fixture_blake3, report.fixture_blake3);
    assert_eq!(row.gating_verdict, "pass");
    assert_eq!(row.fixture_row_count, report.total);
}

#[tokio::test]
async fn mini_fixture_run_is_deterministic_across_repeats() {
    // Same OrchContext mocks -> identical confusion matrix across runs.
    // Locks the D-21 ACCEPTANCE "deterministic (same inputs -> identical
    // confusion matrix)" contract against drift in the predicate or the
    // fixture loader.
    let fixture = bundled_mini_fixture();
    let runner1 = ModeRoutingBenchmark::new(fixture.clone());
    let runner2 = ModeRoutingBenchmark::new(fixture);
    let emitter1 = VecModeRoutingBenchEmitter::new();
    let emitter2 = VecModeRoutingBenchEmitter::new();
    let report1 = runner1.run_all(&emitter1).await.expect("run1");
    let report2 = runner2.run_all(&emitter2).await.expect("run2");
    assert_eq!(report1.confusion_matrix, report2.confusion_matrix);
    assert_eq!(report1.f1_macro, report2.f1_macro);
    assert_eq!(report1.fixture_blake3, report2.fixture_blake3);
}
