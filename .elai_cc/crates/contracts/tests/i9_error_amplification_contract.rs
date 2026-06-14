//! I-9 — error-amplification (alpha_t) telemetry contract tests.
//!
//! Coverage (no #[ignore], no stubs, no placeholder asserts):
//!
//! (a) Production emit lands the ErrorAmplificationMeasured row: drives the REAL
//!     orchestrator emit seam `emit_error_amplification_measured` (the SAME fn
//!     the `elai bench error-amplification` CLI calls) against a temp SQLite SSB,
//!     then queries the real DB by action_type, decodes the payload, and asserts
//!     the beacon row landed with the harness-computed alpha_t (off-author
//!     artifact, R4). This is the load-bearing wiring arm.
//!
//! (b) Deterministic-injection alpha correctness on the frozen fixture: the
//!     repaired arm contracts (alpha_t < 1.0) and recovered_fraction + alpha_t
//!     are complementary; the run is byte-deterministic under a fixed seed (GD-2).
//!
//! (c) Broken arm FAILS (R2 discrimination): the `--no-repair` arm propagates
//!     fully (alpha_t == 1.0), strictly above the repaired arm. An instrument
//!     that could not tell the two apart would be wire-dark.
//!
//! (d) action_type registration + round-trip (as_sql_str / from_str) + the SSB
//!     payload exhaustiveness via the public wire-tag parser + schema floor.
//!
//! NOTE R7: this contract-test caller does NOT satisfy production reachability.
//! The production caller is the CLI `bench error-amplification` seam
//! (crates/cli/src/bench_error_amplification.rs::emit_beacon -> the same
//! orchestrator emit fn). This test only proves the emit path produces a real
//! DB row.
//!
//! Spec references: docs/thefinalglance/I-plan/I-9_error_amplification_telemetry.md
//! Reads (mandatory): wiring-enforcement.md R1/R2/R4/R7/R8; production-caller-gate.md;
//! CLAUDE.md GRC + ID-5 (baseline-before-threshold).

use std::sync::Arc;

use elai_orchestrator::benchmarks::error_amplification_runner::{
    emit_error_amplification_measured, ErrorPropagationHarness, ANNEAL_ALPHA_THRESHOLD,
};
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload,
    SsbReader, SCHEMA_VERSION_PUBLIC, SSB_ACTION_TYPES,
};

/// Build a real temp-DB SSB writer in a TempDir.
async fn build_ssb(session_id: &str) -> (Arc<Ssb>, tempfile::TempDir) {
    use elai_core::clock::OsClock;

    let proj = tempfile::TempDir::new().expect("project tempdir");
    let db_path = proj.path().join("ssb.db");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path,
                pool_size: Some(2),
                project_id: "i9-contract".into(),
                session_id: session_id.to_owned(),
            },
            clock,
        )
        .expect("ssb open"),
    );
    (ssb, proj)
}

// ---------------------------------------------------------------------------
// (a) Production emit lands the ErrorAmplificationMeasured row (R4 wiring)
// ---------------------------------------------------------------------------

/// Drives the REAL orchestrator emit seam against a temp DB; asserts the row
/// lands AND its decoded payload carries the harness-computed alpha_t.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i9_production_emit_lands_error_amplification_row() {
    let (ssb, _proj) = build_ssb("i9-prod").await;

    // Drive the REAL harness over the embedded frozen fixture (repaired arm).
    let harness = ErrorPropagationHarness::from_embedded().expect("embedded fixture parses");
    let report = harness.run(true);

    // Emit via the SAME fn the CLI bench seam calls (the off-author artifact).
    emit_error_amplification_measured(&ssb, "i9-contract", &report)
        .await
        .expect("emit_error_amplification_measured");

    // OFF-AUTHOR ARTIFACT (R4): query the real DB by type, decode, verify.
    let rows = ssb
        .query_by_type(SsbActionType::ErrorAmplificationMeasured)
        .expect("query_by_type ErrorAmplificationMeasured");
    assert!(
        !rows.is_empty(),
        "expected >=1 ErrorAmplificationMeasured row; found 0 (R1: missing evidence = FAIL)"
    );

    let decoded = decode_ssb_payload(&rows[0].payload).expect("decode_ssb_payload");
    match decoded {
        SsbPayload::ErrorAmplificationMeasured(p) => {
            assert_eq!(p.suite, report.suite, "suite must round-trip");
            assert_eq!(p.alpha_t, report.alpha_t, "alpha_t must round-trip");
            assert_eq!(
                p.recovered_fraction, report.recovered_fraction,
                "recovered_fraction must round-trip"
            );
            assert_eq!(p.injected_step, report.injected_step);
            assert_eq!(p.divergence_step, report.divergence_step);
            assert_eq!(p.repaired, report.repaired);
            assert_eq!(p.seed, report.seed);
            assert_eq!(
                p.fixture_blake3.len(),
                64,
                "fixture_blake3 must be a 32-byte blake3 hex; got {}",
                p.fixture_blake3
            );
            // The repaired arm contracts: alpha below the amplifying threshold.
            assert!(
                p.alpha_t < 1.0,
                "repaired-arm alpha ({}) must contract below 1.0",
                p.alpha_t
            );
        }
        other => panic!("decoded payload must be ErrorAmplificationMeasured; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// (b) Deterministic-injection alpha correctness on the frozen fixture
// ---------------------------------------------------------------------------

#[test]
fn i9_repaired_arm_alpha_is_correct_and_deterministic() {
    let harness = ErrorPropagationHarness::from_embedded().expect("embedded fixture parses");
    let a = harness.run(true);
    let b = harness.run(true);

    // GD-2: byte-deterministic under a fixed seed.
    assert_eq!(a, b, "same fixture + seed + arm => identical report");

    // alpha_t + recovered_fraction == 1.0 (complementary by construction).
    assert!(
        (a.alpha_t + a.recovered_fraction - 1.0).abs() < 1e-9,
        "alpha_t ({}) + recovered_fraction ({}) must equal 1.0",
        a.alpha_t,
        a.recovered_fraction
    );
    assert!((0.0..=1.0).contains(&a.alpha_t), "alpha_t must be in [0,1]");

    // The frozen fixture has injected_step=1 with 4 downstream steps, 3 of which
    // recover => recovered_fraction = 0.75, alpha_t = 0.25 (COMPUTED, not a
    // literal in the harness — asserted here against the known fixture shape).
    assert!(
        (a.alpha_t - 0.25).abs() < 1e-9,
        "frozen fixture repaired alpha must be 0.25; got {}",
        a.alpha_t
    );
    assert_eq!(
        a.divergence_step,
        Some(a.injected_step),
        "divergence first appears at the injected step (DriftComparator shape)"
    );
    // Repaired arm (alpha 0.25 < 0.5 threshold) does NOT anneal.
    assert!(
        a.alpha_t < ANNEAL_ALPHA_THRESHOLD,
        "repaired alpha must be below the annealing threshold"
    );
    assert!(
        a.annealed_temperature.is_none(),
        "repaired arm below threshold must not fire the anneal tighten"
    );
}

// ---------------------------------------------------------------------------
// (c) Broken arm FAILS — R2 discrimination
// ---------------------------------------------------------------------------

#[test]
fn i9_no_repair_arm_amplifies_fully_and_anneals() {
    let harness = ErrorPropagationHarness::from_embedded().expect("embedded fixture parses");
    let repaired = harness.run(true);
    let no_repair = harness.run(false);

    // R2: the no-repair arm propagates through EVERY downstream step.
    assert_eq!(
        no_repair.alpha_t, 1.0,
        "no-repair arm must propagate fully (alpha == 1.0); got {}",
        no_repair.alpha_t
    );
    // The metric DISCRIMINATES: repaired strictly contracts below no-repair.
    assert!(
        repaired.alpha_t < no_repair.alpha_t,
        "repaired alpha ({}) must be strictly below no-repair alpha ({})",
        repaired.alpha_t,
        no_repair.alpha_t
    );
    // The amplifying arm fires the active-annealing tighten (alpha >= threshold).
    assert!(
        no_repair.alpha_t >= ANNEAL_ALPHA_THRESHOLD,
        "no-repair alpha must be at/above the annealing threshold"
    );
    assert!(
        no_repair.annealed_temperature.is_some(),
        "no-repair arm (amplifying) must fire the anneal tighten (C-26 temperature)"
    );
}

// ---------------------------------------------------------------------------
// (d) action_type registration + round-trip + schema floor
// ---------------------------------------------------------------------------

#[test]
fn i9_action_type_round_trips() {
    let tag = "error_amplification_measured";
    let parsed = ssb_action_type_from_str_public(tag)
        .unwrap_or_else(|e| panic!("wire tag {tag} must round-trip; got {e:?}"));
    assert_eq!(parsed, SsbActionType::ErrorAmplificationMeasured);
    assert_eq!(parsed.as_sql_str(), tag, "round-trip mismatch for {tag}");
}

#[test]
fn i9_schema_lock_error_amplification_registered() {
    // I-9 appends ONE flat beacon: SSB_ACTION_TYPES len -> 572, SCHEMA -> 263.
    // Floors (not exact — the registry climbs as later plans land).
    assert!(
        SCHEMA_VERSION_PUBLIC >= 263,
        "SCHEMA_VERSION_PUBLIC must be >= 263 after I-9; got {SCHEMA_VERSION_PUBLIC}"
    );
    assert!(
        SSB_ACTION_TYPES.len() >= 572,
        "SSB_ACTION_TYPES must have >= 572 variants after I-9; got {}",
        SSB_ACTION_TYPES.len()
    );
    // The new action_type is present in the canonical registry exactly once.
    let count = SSB_ACTION_TYPES
        .iter()
        .filter(|a| **a == SsbActionType::ErrorAmplificationMeasured)
        .count();
    assert_eq!(
        count, 1,
        "ErrorAmplificationMeasured must appear exactly once in SSB_ACTION_TYPES; got {count}"
    );
}
