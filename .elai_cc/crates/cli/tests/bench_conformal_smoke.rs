//! D-22 observable_effect smoke test for the conformal calibration
//! validation benchmark.
//!
//! Plan WIRING_CONTRACT observable_effect:
//!   trigger: `elai bench conformal`
//!   expect_ssb: ConformalValidationCompleted row
//!   expect_artifact: deterministic JSON report
//!
//! Strategy mirrors `bench_mode_routing_smoke.rs`: the CLI dispatcher
//! calls `std::process::exit` and the cli crate is binary-only, so the
//! test exercises the orchestrator-side runner
//! (`ConformalValidationRunner::run_all`) directly against the bundled
//! mini fixture. The CLI's own unit tests (in `src/bench_conformal.rs`)
//! cover the parser + epsilon-override + render path.

use std::path::PathBuf;

use elai_orchestrator::benchmarks::conformal_validation_runner::{
    ConformalValidationRunner, VecConformalValidationSsbEmitter,
};
use elai_orchestrator::benchmarks::prmbench_runner::GatingVerdict;
use elai_orchestrator::conformal_calibration::{
    CalibrationCorpus, CalibrationTuple, CorpusSource,
};

/// Locate the bundled mini-fixture relative to the cli crate's
/// `CARGO_MANIFEST_DIR`. The fixture lives at the repo root
/// (`.elai_cc/benchmarks/conformal/fixtures/`).
fn bundled_mini_fixture() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("..")
        .join("..")
        .join("benchmarks")
        .join("conformal")
        .join("fixtures")
        .join("conformal_validation_mini.jsonl")
}

/// Match the balanced 20-pair synthetic corpus the bench_conformal CLI
/// default-builds (and the orchestrator unit tests use). Keeps the smoke
/// test hermetic — no on-disk seed corpus required.
fn balanced_corpus() -> CalibrationCorpus {
    let mut c = CalibrationCorpus::new(CorpusSource::HeldOut);
    for i in 0..20 {
        c.append(CalibrationTuple::new(
            format!("t-{i}"),
            format!("c-{i}"),
            "true claim",
            true,
            0.9,
            1_700_000_000 + (i as i64),
        ))
        .expect("append true tuple");
        c.append(CalibrationTuple::new(
            format!("f-{i}"),
            format!("d-{i}"),
            "false claim",
            false,
            0.2,
            1_700_000_500 + (i as i64),
        ))
        .expect("append false tuple");
    }
    c.freeze();
    c
}

#[tokio::test]
async fn observable_effect_mini_fixture_passes_gate_and_emits_one_ssb_row() {
    let fixture = bundled_mini_fixture();
    assert!(
        fixture.exists(),
        "expected bundled mini-fixture at {}",
        fixture.display()
    );
    let runner = ConformalValidationRunner::new(balanced_corpus(), fixture);
    let emitter = VecConformalValidationSsbEmitter::new();
    let report = runner
        .run_all(&emitter, 1_700_000_200_000)
        .await
        .expect("run ok");

    // Shape assertions — every published field is populated.
    assert_eq!(report.total, 36, "mini fixture is 36 rows");
    assert!((0.0..=1.0).contains(&report.tau), "tau in [0, 1]");
    assert!(
        (0.0..=1.0).contains(&report.empirical_refusal_rate),
        "empirical refusal rate in [0, 1]"
    );
    assert_eq!(
        report.fixture_blake3.len(),
        64,
        "fixture_blake3 is 64 hex chars"
    );
    // Plan + fixture engineering: 1 low-score row per bucket -> per-bucket
    // refusal rate = 1/12 ~= 0.083, well inside [0.0, 0.2] band at
    // epsilon = 0.1.
    assert!(
        report.empirical_refusal_rate <= 0.2,
        "empirical refusal rate should be inside the target band, got {}",
        report.empirical_refusal_rate
    );
    assert_eq!(
        report.gating_verdict,
        GatingVerdict::Pass,
        "mini fixture is engineered to pass the per-bucket band gate",
    );

    // Per-domain breakdown surfaces every (domain, bucket) cell.
    assert!(!report.per_domain.is_empty(), "per_domain populated");
    // Aggregate per-bucket map covers all three buckets.
    assert!(report.per_bucket.contains_key("in_domain"));
    assert!(report.per_bucket.contains_key("near_ood"));
    assert!(report.per_bucket.contains_key("far_ood"));

    // Exactly one SSB row is emitted.
    assert_eq!(emitter.len().await, 1, "exactly one SSB row per run_all");
    let rows = emitter.rows().await;
    let row = &rows[0];
    assert_eq!(row.total, report.total);
    assert_eq!(row.fixture_blake3, report.fixture_blake3);
    assert_eq!(row.gating_verdict, "pass");
    assert_eq!(row.ts_ms, 1_700_000_200_000);
}

#[tokio::test]
async fn mini_fixture_run_is_deterministic_across_repeats() {
    // Same corpus + fixture -> identical report (modulo ts_ms which we
    // pin). Locks the D-22 ACCEPTANCE "deterministic" contract.
    let fixture = bundled_mini_fixture();
    let runner1 = ConformalValidationRunner::new(balanced_corpus(), fixture.clone());
    let runner2 = ConformalValidationRunner::new(balanced_corpus(), fixture);
    let emitter1 = VecConformalValidationSsbEmitter::new();
    let emitter2 = VecConformalValidationSsbEmitter::new();
    let report1 = runner1
        .run_all(&emitter1, 1_700_000_200_000)
        .await
        .expect("run1");
    let report2 = runner2
        .run_all(&emitter2, 1_700_000_200_000)
        .await
        .expect("run2");
    assert_eq!(report1.tau, report2.tau);
    assert_eq!(
        report1.empirical_refusal_rate,
        report2.empirical_refusal_rate
    );
    assert_eq!(report1.fixture_blake3, report2.fixture_blake3);
    assert_eq!(report1.per_domain, report2.per_domain);
    assert_eq!(report1.gating_verdict, report2.gating_verdict);
}
