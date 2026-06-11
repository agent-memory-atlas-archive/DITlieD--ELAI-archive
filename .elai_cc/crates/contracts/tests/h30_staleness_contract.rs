//! H-30 contract tests — the staleness / intent-drift INSTRUMENT (the
//! measure-before-trust keystone for the durable-user-memory write/read path).
//!
//! Covers (TEST-FIRST, authored against the live runner + SSB surface), over a
//! REAL Ssb SQLite db (off-agent artifact, Layer-4 / R4):
//!   (1) the eval over the labeled supersession fixture measures SR + IPA; the
//!       clean retrieval path returns X' and NEVER X (stale_fact_reached == 0).
//!   (2) the grounded invariant SR >= IPA holds (application cannot exceed
//!       recognition; STALE arXiv:2605.06527 Finding 1).
//!   (3) the --broken-retrieval arm makes stale_fact_reached_prompt > 0 (the
//!       instrument SCREAMS) and IPA drops.
//!   (4) THE R2 SELF-TEST: a broken-arm run reporting stale_fact_reached == 0
//!       is itself an instrument FAIL — the runner flags instrument_failed; a
//!       healthy broken arm leaks rows so instrument_failed stays false, AND a
//!       contrived broken arm with nothing to leak flips instrument_failed true.
//!   (5) seed-reproducible: same ELAI_SEED => byte-identical report (GD-2).
//!   (6) SCHEMA_VERSION >= 239, StalenessEvalMeasured in the action_type
//!       bijection, and a REAL StalenessEvalMeasured row lands + round-trips
//!       through the SSB single-writer (the off-agent artifact).
//!
//! SCHEMA_VERSION assertions use `>=` (never an exact global pin) per the
//! BRITTLE_GLOBAL_SCHEMA_PIN gotcha. TOTAL_VARIANTS / ALL_VARIANTS presence is
//! asserted by action_type_lock.rs (compile-time exhaustiveness + count-drift)
//! and the e45 contract.
//!
//! Grounding:
//!   - crates/orchestrator/src/benchmarks/staleness_runner.rs (the H-30 runner).
//!   - crates/memory/src/store.rs (FactStore.supersede / soft_close /
//!     list_current hard-filter / list_including_closed read-only accessor).
//!   - crates/memory/src/retrieval.rs (get_relevant_facts — the chokepoint).
//!   - workflow/papers/stale-2605.06527-interface.md (SR/IPA definitions +
//!     the SR >= IPA invariant).

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::benchmarks::staleness_runner::StalenessEvalRunner;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload,
    StalenessEvalMeasuredPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, project_id: &str) -> Arc<Ssb> {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: project_id.into(),
        session_id: format!("{project_id}-session"),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb"))
}

fn clean_runner() -> StalenessEvalRunner {
    StalenessEvalRunner::from_embedded(false).expect("embedded corpus parses (clean)")
}

fn broken_runner() -> StalenessEvalRunner {
    StalenessEvalRunner::from_embedded(true).expect("embedded corpus parses (broken)")
}

// ---------------------------------------------------------------------------
// (1) clean path: SR + IPA measured; X' returned, X never; stale_reached == 0
// ---------------------------------------------------------------------------

#[test]
fn clean_path_serves_fresh_never_stale_and_measures_both_rates() {
    let report = clean_runner().run().expect("clean run");
    assert!(report.n_cases >= 2, "fixture must have >=2 cases");
    assert!(!report.broken_arm, "clean arm");
    assert!(!report.instrument_failed, "clean arm is never instrument_failed");
    assert_eq!(
        report.stale_fact_reached_prompt, 0,
        "clean path: the list_current valid_to_ms hard-filter must keep stale rows out of the returned set"
    );
    // Both rates are measured fractions in (0,1] (computed, not literals).
    assert!((0.0..=1.0).contains(&report.sr_rate), "sr_rate in [0,1]");
    assert!((0.0..=1.0).contains(&report.ipa_rate), "ipa_rate in [0,1]");
    assert!(report.sr_rate > 0.0, "sr_rate must be > 0 (>=1 recognized case)");
    assert!(report.ipa_rate > 0.0, "ipa_rate must be > 0 (>=1 applied case)");
    // The honest unrecognized case keeps SR strictly below 1.0.
    assert!(report.sr_rate < 1.0, "sr_rate must be < 1.0 (the honest unrecognized case)");
}

// ---------------------------------------------------------------------------
// (2) the grounded invariant SR >= IPA
// ---------------------------------------------------------------------------

#[test]
fn sr_is_at_least_ipa_application_cannot_exceed_recognition() {
    let report = clean_runner().run().expect("clean run");
    assert!(
        report.sr_rate >= report.ipa_rate,
        "STALE invariant: SR ({}) must be >= IPA ({}) -- application cannot exceed recognition",
        report.sr_rate,
        report.ipa_rate
    );
    // The gap is exactly sr - ipa.
    let gap = report.sr_rate - report.ipa_rate;
    assert!(
        (report.recognition_application_gap - gap).abs() < 1e-12,
        "recognition_application_gap must equal sr_rate - ipa_rate"
    );
    assert!(report.recognition_application_gap >= 0.0, "the gap is non-negative");
}

// ---------------------------------------------------------------------------
// (3) the broken arm SCREAMS: stale_fact_reached > 0 and IPA drops
// ---------------------------------------------------------------------------

#[test]
fn broken_arm_screams_and_ipa_drops() {
    let clean = clean_runner().run().expect("clean run");
    let broken = broken_runner().run().expect("broken run");
    assert!(broken.broken_arm, "broken arm flag set");
    assert!(
        broken.stale_fact_reached_prompt > 0,
        "the --broken-retrieval arm MUST leak >=1 superseded row (R2: the instrument screams)"
    );
    assert!(
        broken.ipa_rate <= clean.ipa_rate,
        "broken IPA ({}) must be <= clean IPA ({}) -- the stale row reaching the set drops application",
        broken.ipa_rate,
        clean.ipa_rate
    );
    // SR is recognition-side; a broken RETRIEVER does not change it.
    assert!(
        (broken.sr_rate - clean.sr_rate).abs() < 1e-12,
        "SR is recognition-side and must be identical across arms"
    );
}

// ---------------------------------------------------------------------------
// (4) THE R2 SELF-TEST — a broken arm reporting zero is itself a FAIL
// ---------------------------------------------------------------------------

#[test]
fn r2_self_test_broken_arm_reporting_zero_is_instrument_failure() {
    // A healthy broken arm leaks rows, so instrument_failed stays false.
    let broken = broken_runner().run().expect("broken run");
    assert!(
        broken.stale_fact_reached_prompt > 0 && !broken.instrument_failed,
        "a healthy broken arm leaks rows (>0) and is NOT an instrument failure"
    );

    // Contrive a broken arm with NOTHING to leak (a corpus whose only case is
    // unrecognized, so no stale row is ever closed). The broken retriever then
    // reports stale_reached == 0 and the runner MUST flag instrument_failed —
    // the dead-evidence path must scream (R2).
    let only_unrecognized = r#"[
      {
        "id":"r2-only-unrec-001","fact_type":"preference","key":"k",
        "stale_value":"a","fresh_value":"b","stale_tier":"T0","fresh_tier":"T0",
        "stale_evidence":["e1"],"fresh_evidence":["e2"],"expect_recognized":false
      }
    ]"#;
    let contrived = StalenessEvalRunner::from_json_with_seed(only_unrecognized, 0, true)
        .expect("parse contrived broken corpus");
    let rep = contrived.run().expect("contrived run");
    assert_eq!(
        rep.stale_fact_reached_prompt, 0,
        "no row was recognized-stale, so even the broken retriever leaks nothing"
    );
    assert!(
        rep.instrument_failed,
        "a broken-arm run reporting stale_fact_reached_prompt == 0 MUST be flagged instrument_failed (R2)"
    );
}

// ---------------------------------------------------------------------------
// (5) seed-reproducible — same ELAI_SEED => byte-identical report (GD-2)
// ---------------------------------------------------------------------------

#[test]
fn report_is_byte_identical_under_same_seed() {
    let a = StalenessEvalRunner::from_json_with_seed(
        include_str_corpus(),
        7,
        false,
    )
    .expect("parse a")
    .run()
    .expect("run a");
    let b = StalenessEvalRunner::from_json_with_seed(include_str_corpus(), 7, false)
        .expect("parse b")
        .run()
        .expect("run b");
    assert_eq!(a, b, "same corpus + same seed => byte-identical report (GD-2)");
    assert_eq!(a.seed, 7, "seed round-trips into the report");
    assert_eq!(a.corpus_blake3.len(), 64, "blake3 hex is 64 chars");
}

/// Re-derive the embedded corpus bytes the runner ships with, so the
/// determinism test pins the SAME content the binary uses.
fn include_str_corpus() -> &'static str {
    include_str!("../../orchestrator/src/benchmarks/staleness_fixtures/corpus.json")
}

// ---------------------------------------------------------------------------
// (6) SCHEMA bijection + a REAL StalenessEvalMeasured row lands (off-agent)
// ---------------------------------------------------------------------------

#[test]
fn schema_version_and_action_type_bijection() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 239,
        "H-30 requires SCHEMA_VERSION >= 239, got {SCHEMA_VERSION_PUBLIC}"
    );
    let at = ssb_action_type_from_str_public("staleness_eval_measured")
        .expect("from_str staleness_eval_measured");
    assert_eq!(at, SsbActionType::StalenessEvalMeasured);
    assert_eq!(
        SsbActionType::StalenessEvalMeasured.as_sql_str(),
        "staleness_eval_measured"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_staleness_eval_measured_row_lands_and_round_trips() {
    let dir = TempDir::new().unwrap();
    let ssb = test_ssb(&dir, "h30-contract");

    // Drive the REAL runner, then emit the beacon through the single-writer.
    let report = clean_runner().run().expect("clean run");
    let payload = StalenessEvalMeasuredPayload {
        suite: report.suite.clone(),
        n_cases: report.n_cases,
        sr_rate: report.sr_rate,
        ipa_rate: report.ipa_rate,
        recognition_application_gap: report.recognition_application_gap,
        stale_fact_reached_prompt: report.stale_fact_reached_prompt,
        total_retrievals: report.total_retrievals,
        broken_arm: report.broken_arm,
        instrument_failed: report.instrument_failed,
        seed: report.seed,
        corpus_blake3: report.corpus_blake3.clone(),
    };
    ssb.write_payload(
        "h30-contract",
        0,
        SsbPayload::StalenessEvalMeasured(payload.clone()),
        Vec::new(),
    )
    .await
    .expect("write StalenessEvalMeasured row");

    // The off-agent artifact: read it back and verify the round-trip.
    let rows = ssb
        .query_by_type(SsbActionType::StalenessEvalMeasured)
        .expect("query StalenessEvalMeasured");
    assert_eq!(
        rows.len(),
        1,
        "exactly one StalenessEvalMeasured row must land (the off-agent artifact)"
    );
    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("decode payload");
    match decoded {
        SsbPayload::StalenessEvalMeasured(p) => {
            assert_eq!(p, payload, "round-trip mismatch on the StalenessEvalMeasured payload");
            assert_eq!(p.stale_fact_reached_prompt, 0, "clean-path row reports stale_reached == 0");
            assert!(p.sr_rate >= p.ipa_rate, "the row carries the SR >= IPA invariant");
        }
        other => panic!("expected StalenessEvalMeasured, got {other:?}"),
    }
}
