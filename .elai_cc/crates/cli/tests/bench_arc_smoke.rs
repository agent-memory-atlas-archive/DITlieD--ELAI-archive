//! D-20 observable_effect smoke test for the ARC-AGI-2 OOD probe.
//!
//! Plan WIRING_CONTRACT observable_effect:
//!   trigger: `elai bench arc`
//!   expect_ssb: ArcBenchmarkCompleted row
//!   expect_artifact: ARCReport JSON
//!
//! Strategy mirrors `bench_mode_routing_smoke.rs`: the CLI dispatcher
//! calls `std::process::exit` and the cli crate is binary-only, so the
//! test exercises the orchestrator-side runner
//! (`ARCAGI2Runner::run_all`) directly against the bundled mini
//! fixture. The CLI's own unit tests (in `src/bench_arc.rs::tests`)
//! cover the parser + threshold-override + render path.

use std::path::PathBuf;

use elai_orchestrator::benchmarks::arc_agi2_runner::{
    ARCAGI2Runner, ARCVerdict, VecARCSsbEmitter,
};
use elai_orchestrator::grounding_mode::GroundingMode;
use elai_orchestrator::novelty::NoveltyClass;

/// Locate the bundled mini-fixture relative to the cli crate's
/// `CARGO_MANIFEST_DIR`. The fixture lives at the repo root
/// (`.elai_cc/benchmarks/arc_agi2/arc_mini_fixture.jsonl`).
fn bundled_mini_fixture() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("..")
        .join("..")
        .join("benchmarks")
        .join("arc_agi2")
        .join("arc_mini_fixture.jsonl")
}

#[tokio::test]
async fn observable_effect_mini_fixture_produces_one_ssb_row_and_typed_verdict() {
    let fixture = bundled_mini_fixture();
    assert!(
        fixture.exists(),
        "expected bundled mini-fixture at {}",
        fixture.display()
    );
    let runner = ARCAGI2Runner::new(fixture, 1_700_000_400_000);
    let emitter = VecARCSsbEmitter::new();
    let report = runner.run_all(&emitter).await.expect("run ok");

    // Shape assertions — every published field is populated.
    assert!(report.total >= 20, "mini fixture is 20-30 rows");
    assert_eq!(
        report.fixture_blake3.len(),
        64,
        "fixture_blake3 is 64 hex chars"
    );
    assert_eq!(
        report.mode_distribution.len(),
        GroundingMode::all().len(),
        "mode_distribution covers all 5 GroundingMode variants"
    );
    assert_eq!(
        report.novelty_distribution.len(),
        NoveltyClass::all().len(),
        "novelty_distribution covers all 3 NoveltyClass variants"
    );

    // The mini fixture is engineered to hit the HonestlyEvaluated verdict
    // (heavy DiscoveryCandidate share + Hypothetical-class claims that
    // D-10 routes to OperationalDeferred).
    match &report.gating_verdict {
        ARCVerdict::HonestlyEvaluated { details } => {
            assert!(!details.is_empty(), "verdict details non-empty");
        }
        other => panic!("expected HonestlyEvaluated against mini fixture, got {other:?}"),
    }

    // Verdict-driven helper.
    assert!(report.passed());

    // Exactly one SSB row is emitted.
    assert_eq!(emitter.len().await, 1, "exactly one SSB row per run_all");
    let rows = emitter.rows().await;
    let row = &rows[0];
    assert_eq!(row.total, report.total);
    assert_eq!(row.fixture_blake3, report.fixture_blake3);
    assert_eq!(row.gating_verdict, "honestly_evaluated");
    assert_eq!(row.honestly_evaluated_count, report.honestly_evaluated_count);
    assert_eq!(row.dominant_mode, report.dominant_mode);
    assert_eq!(row.dominant_novelty_class, report.dominant_novelty_class);
}

#[tokio::test]
async fn mini_fixture_run_is_deterministic_across_repeats() {
    // Same fixture -> identical fixture_blake3 + identical aggregate
    // counts across runs. Locks the "deterministic predicate" contract
    // against drift in either D-10 or D-15.
    let fixture = bundled_mini_fixture();
    let runner1 = ARCAGI2Runner::new(fixture.clone(), 1_700_000_401_000);
    let runner2 = ARCAGI2Runner::new(fixture, 1_700_000_402_000);
    let emitter1 = VecARCSsbEmitter::new();
    let emitter2 = VecARCSsbEmitter::new();
    let report1 = runner1.run_all(&emitter1).await.expect("run1");
    let report2 = runner2.run_all(&emitter2).await.expect("run2");
    assert_eq!(report1.fixture_blake3, report2.fixture_blake3);
    assert_eq!(report1.total, report2.total);
    assert_eq!(report1.mode_distribution, report2.mode_distribution);
    assert_eq!(report1.novelty_distribution, report2.novelty_distribution);
    assert_eq!(
        report1.honestly_evaluated_count, report2.honestly_evaluated_count
    );
    // gating_verdict is a typed enum carrying details strings; we
    // compare the wire-form labels instead so the details string can
    // contain timestamps in future revisions without breaking the
    // contract.
    assert_eq!(
        report1.gating_verdict.as_str(),
        report2.gating_verdict.as_str()
    );
}
