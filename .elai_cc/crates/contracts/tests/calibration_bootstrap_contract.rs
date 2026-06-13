// D-30 contract: calibration bootstrap warm-start surface.
//
// Verifies (atomic SCHEMA bump 146 -> 147, +1 SsbActionType variant):
// 1. SCHEMA_VERSION_PUBLIC >= 147 (D-30 floor; CalibrationBootstrapCompleted
//    variant landed at this bump).
// 2. SsbActionType::CalibrationBootstrapCompleted + payload round-trip via
//    as_sql_str / ssb_action_type_from_str_public + JSON + msgpack
//    (binds the wire-format string + payload shape).
// 3. Option fields (bootstrap_replicates / bootstrap_ci_low /
//    bootstrap_ci_high / block_length_p) serialise as None for the
//    warm-start-only path AND as Some(_) for SECONDARY block-bootstrap
//    runs without bumping the schema.
// 4. PublishedCalibrationRow -> CalibrationTuple projection clamps
//    predicted_score to [0.0, 1.0] and truncates claim_text to
//    MAX_CLAIM_TEXT_CHARS (256) via CalibrationTuple::new.
// 5. CalibrationBootstrapMode::from_env defaults to Off when unset
//    (wire-dark default per pattern-4) and flips On for "1"/"on"/"true"/
//    "yes" (case-insensitive). ENV_LOCK Mutex used to avoid parallel-
//    test drift on the process-wide env.
// 6. default_block_p(n) returns 1/sqrt(n) (Politis-Romano 1994 JASA Vol 89
//    §3 default) for n > 0 and a safe 1.0 fallback for n = 0.
// 7. MIN_BOOTSTRAP_REPLICATES = 200 floor + MAX_BOOTSTRAP_REPLICATES =
//    10_000 ceiling enforced by bootstrap_resample; below the floor or
//    above the ceiling errors with InvalidReplicateCount.
// 8. bootstrap_resample seed determinism: same seed + corpus + params
//    produces byte-identical sorted tau replicate vectors across two
//    independent SeededRng instances.

use std::sync::Mutex;

use elai_core::entropy::SeededRng;
use elai_orchestrator::calibration::{
    bootstrap_resample, ci_90_from_sorted, default_block_p, BootstrapError,
    CalibrationBootstrapCorpus, PublishedCalibrationRow, DEFAULT_BOOTSTRAP_EPSILON,
    DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE, MAX_BOOTSTRAP_REPLICATES, MIN_BOOTSTRAP_REPLICATES,
    CalibrationTuple, MAX_CLAIM_TEXT_CHARS,
};
use elai_orchestrator::runtime_config::{
    CalibrationBootstrapMode, ELAI_CALIBRATION_BOOTSTRAP_ENABLE_ENV,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, CalibrationBootstrapCompletedPayload, SsbActionType,
    SsbPayload, SCHEMA_VERSION_PUBLIC,
};

/// Floor for D-30: head SCHEMA_VERSION at completion-drive close.
/// Atomic +1 bump from D-29's 146 head (single new variant
/// CalibrationBootstrapCompleted; see D-30-decisions.md DECISION-1).
const D30_SCHEMA_FLOOR: i64 = 147;

/// Process-wide lock used by every env-touching test in this file. Mirrors
/// the ENV_LOCK precedent in hierarchical_bai_contract.rs so parallel
/// `cargo test` runs do not race on `ELAI_CALIBRATION_BOOTSTRAP_ENABLE`.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn make_row(idx: usize, gt: bool, score: f64) -> PublishedCalibrationRow {
    PublishedCalibrationRow {
        tuple_id: format!("truthfulqa-{idx:04}"),
        claim_id: format!("claim-{idx}"),
        claim_text: format!("synthetic test claim {idx}"),
        ground_truth: gt,
        predicted_score: score,
        ts_ms: 1_700_000_000_000 + idx as i64,
    }
}

fn make_corpus(n: usize) -> CalibrationBootstrapCorpus {
    let tuples: Vec<CalibrationTuple> = (0..n)
        .map(|i| {
            let gt = i % 2 == 0;
            let score = if gt { 0.9 } else { 0.4 };
            CalibrationTuple::from(make_row(i, gt, score))
        })
        .collect();
    CalibrationBootstrapCorpus {
        corpus_slug: "truthfulqa".into(),
        corpus_blake3: "0".repeat(64),
        tuples,
    }
}

#[test]
fn schema_version_includes_d30_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= D30_SCHEMA_FLOOR,
        "schema must include D-30 CalibrationBootstrapCompleted variant (got {SCHEMA_VERSION_PUBLIC}, floor {D30_SCHEMA_FLOOR})"
    );
}

#[test]
fn calibration_bootstrap_completed_warmstart_payload_roundtrip() {
    // Bind the SQL wire tag + the round-trip identity for the warm-start
    // (Option fields None) path.
    let tag = SsbActionType::CalibrationBootstrapCompleted.as_sql_str();
    assert_eq!(tag, "calibration_bootstrap_completed");
    let decoded = ssb_action_type_from_str_public(tag)
        .expect("ssb_action_type_from_str_public calibration_bootstrap_completed");
    assert_eq!(decoded, SsbActionType::CalibrationBootstrapCompleted);

    let p = CalibrationBootstrapCompletedPayload {
        corpus_slug: "truthfulqa".into(),
        corpus_size: 50,
        corpus_blake3: "a".repeat(64),
        tau_warm_start: 0.8,
        alpha: 0.1,
        tau_floor_active: false,
        bootstrap_replicates: None,
        bootstrap_ci_low: None,
        bootstrap_ci_high: None,
        block_length_p: None,
        ts_ms: 1_700_000_147_000,
        calibration_id: None,
    };
    let payload = SsbPayload::CalibrationBootstrapCompleted(p.clone());
    assert_eq!(
        payload.action_type(),
        SsbActionType::CalibrationBootstrapCompleted
    );

    // JSON round-trip preserves payload byte semantics (Option::None
    // serialises to `null` and survives round-trip).
    let json = serde_json::to_string(&p).expect("payload json serialize");
    let decoded_p: CalibrationBootstrapCompletedPayload =
        serde_json::from_str(&json).expect("payload json deserialize");
    assert_eq!(decoded_p, p);
    assert!(decoded_p.bootstrap_replicates.is_none());
    assert!(decoded_p.bootstrap_ci_low.is_none());
    assert!(decoded_p.bootstrap_ci_high.is_none());
    assert!(decoded_p.block_length_p.is_none());

    // msgpack round-trip via SsbPayload outer envelope.
    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack serialize");
    let decoded_env: SsbPayload =
        rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    assert_eq!(
        decoded_env.action_type(),
        SsbActionType::CalibrationBootstrapCompleted
    );
    if let SsbPayload::CalibrationBootstrapCompleted(d) = decoded_env {
        assert_eq!(d, p);
    } else {
        panic!("expected CalibrationBootstrapCompleted payload");
    }
}

#[test]
fn calibration_bootstrap_completed_secondary_ci_payload_roundtrip() {
    // SECONDARY path: Option fields populated. Schema MUST NOT bump just
    // because the Option arms went from None to Some — that is the
    // entire point of the single-variant + Option-extension design
    // (DECISION-8).
    let p = CalibrationBootstrapCompletedPayload {
        corpus_slug: "truthfulqa".into(),
        corpus_size: 50,
        corpus_blake3: "b".repeat(64),
        tau_warm_start: 0.82,
        alpha: 0.1,
        tau_floor_active: false,
        bootstrap_replicates: Some(1000),
        bootstrap_ci_low: Some(0.78),
        bootstrap_ci_high: Some(0.86),
        block_length_p: Some(0.1414213562373095),
        ts_ms: 1_700_000_147_001,
        calibration_id: None,
    };
    let json = serde_json::to_string(&p).expect("payload json serialize");
    let decoded_p: CalibrationBootstrapCompletedPayload =
        serde_json::from_str(&json).expect("payload json deserialize");
    assert_eq!(decoded_p, p);
    assert_eq!(decoded_p.bootstrap_replicates, Some(1000));
    assert!((decoded_p.bootstrap_ci_low.unwrap() - 0.78).abs() < 1e-12);
    assert!((decoded_p.bootstrap_ci_high.unwrap() - 0.86).abs() < 1e-12);
    assert!((decoded_p.block_length_p.unwrap() - 0.1414213562373095).abs() < 1e-12);

    let bytes = rmp_serde::to_vec_named(&p).expect("msgpack serialize");
    let decoded_mp: CalibrationBootstrapCompletedPayload =
        rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    assert_eq!(decoded_mp, p);
}

#[test]
fn published_row_projection_clamps_score_and_truncates_text() {
    // predicted_score above 1.0 clamps to 1.0 via CalibrationTuple::new
    // (D-16 invariant). claim_text > MAX_CLAIM_TEXT_CHARS truncates.
    let long_text: String = "x".repeat(MAX_CLAIM_TEXT_CHARS + 10);
    let row = PublishedCalibrationRow {
        tuple_id: "tid-001".into(),
        claim_id: "claim-1".into(),
        claim_text: long_text,
        ground_truth: true,
        predicted_score: 1.5,
        ts_ms: 1_700_000_000_000,
    };
    let tup: CalibrationTuple = row.into();
    assert!(tup.predicted_score <= 1.0 && tup.predicted_score >= 0.0);
    assert_eq!(tup.predicted_score, 1.0);
    assert!(tup.claim_text.chars().count() <= MAX_CLAIM_TEXT_CHARS);
}

#[test]
fn published_row_projection_collapses_non_finite_score_to_zero() {
    let row = PublishedCalibrationRow {
        tuple_id: "tid-002".into(),
        claim_id: "claim-2".into(),
        claim_text: "test".into(),
        ground_truth: false,
        predicted_score: f64::NAN,
        ts_ms: 1_700_000_000_000,
    };
    let tup: CalibrationTuple = row.into();
    assert_eq!(tup.predicted_score, 0.0);
}

#[test]
fn calibration_bootstrap_mode_defaults_off_when_unset() {
    let _g = ENV_LOCK.lock().expect("env lock");
    // Save + clear + restore so we never leak state to sibling tests.
    // ENV_LOCK + 2021-edition env::set_var precedent matches
    // hierarchical_bai_contract.rs (D-29).
    let prior = std::env::var(ELAI_CALIBRATION_BOOTSTRAP_ENABLE_ENV).ok();
    std::env::remove_var(ELAI_CALIBRATION_BOOTSTRAP_ENABLE_ENV);
    let mode = CalibrationBootstrapMode::from_env();
    assert_eq!(mode, CalibrationBootstrapMode::Off);
    assert!(!mode.is_on());
    assert_eq!(mode.as_str(), "off");
    if let Some(v) = prior {
        std::env::set_var(ELAI_CALIBRATION_BOOTSTRAP_ENABLE_ENV, v);
    }
}

#[test]
fn calibration_bootstrap_mode_flips_on_for_truthy_tokens() {
    let _g = ENV_LOCK.lock().expect("env lock");
    let prior = std::env::var(ELAI_CALIBRATION_BOOTSTRAP_ENABLE_ENV).ok();
    let truthy = ["1", "on", "On", "TRUE", "yes"];
    for raw in truthy {
        std::env::set_var(ELAI_CALIBRATION_BOOTSTRAP_ENABLE_ENV, raw);
        let mode = CalibrationBootstrapMode::from_env();
        assert_eq!(
            mode,
            CalibrationBootstrapMode::On,
            "expected On for token {raw:?}, got {mode:?}"
        );
        assert!(mode.is_on());
        assert_eq!(mode.as_str(), "on");
    }
    let falsy = ["0", "off", "no", "garbage", ""];
    for raw in falsy {
        std::env::set_var(ELAI_CALIBRATION_BOOTSTRAP_ENABLE_ENV, raw);
        let mode = CalibrationBootstrapMode::from_env();
        assert_eq!(
            mode,
            CalibrationBootstrapMode::Off,
            "expected Off for token {raw:?}, got {mode:?}"
        );
    }
    match prior {
        Some(v) => std::env::set_var(ELAI_CALIBRATION_BOOTSTRAP_ENABLE_ENV, v),
        None => std::env::remove_var(ELAI_CALIBRATION_BOOTSTRAP_ENABLE_ENV),
    }
}

#[test]
fn default_block_p_matches_politis_romano_default() {
    // E[L] = 1 / p; Politis-Romano §3 default p = 1/sqrt(N).
    assert!((default_block_p(100) - 0.1).abs() < 1e-12);
    assert!((default_block_p(400) - 0.05).abs() < 1e-12);
    assert!((default_block_p(2500) - 0.02).abs() < 1e-12);
    // degenerate input -> safe 1.0 fallback (no resampling, but no panic)
    assert_eq!(default_block_p(0), 1.0);
}

#[test]
fn bootstrap_resample_enforces_replicate_window() {
    let corpus = make_corpus(40);
    let mut rng = SeededRng::new(1);
    // Below floor.
    let too_low = bootstrap_resample(
        &corpus,
        DEFAULT_BOOTSTRAP_EPSILON,
        DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE,
        MIN_BOOTSTRAP_REPLICATES - 1,
        default_block_p(40),
        &mut rng,
    )
    .expect_err("must reject below floor");
    assert!(matches!(too_low, BootstrapError::InvalidReplicateCount { .. }));
    // Above ceiling.
    let too_high = bootstrap_resample(
        &corpus,
        DEFAULT_BOOTSTRAP_EPSILON,
        DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE,
        MAX_BOOTSTRAP_REPLICATES + 1,
        default_block_p(40),
        &mut rng,
    )
    .expect_err("must reject above ceiling");
    assert!(matches!(too_high, BootstrapError::InvalidReplicateCount { .. }));
}

#[test]
fn bootstrap_resample_seed_determinism_contract() {
    let corpus = make_corpus(40);
    let p = default_block_p(corpus.size());
    let mut rng_a = SeededRng::new(424242);
    let mut rng_b = SeededRng::new(424242);
    let taus_a = bootstrap_resample(
        &corpus,
        DEFAULT_BOOTSTRAP_EPSILON,
        DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE,
        MIN_BOOTSTRAP_REPLICATES,
        p,
        &mut rng_a,
    )
    .expect("resample A");
    let taus_b = bootstrap_resample(
        &corpus,
        DEFAULT_BOOTSTRAP_EPSILON,
        DEFAULT_BOOTSTRAP_MIN_CORPUS_SIZE,
        MIN_BOOTSTRAP_REPLICATES,
        p,
        &mut rng_b,
    )
    .expect("resample B");
    assert_eq!(taus_a, taus_b);
    assert_eq!(taus_a.len(), MIN_BOOTSTRAP_REPLICATES as usize);
    // Output sorted ascending (Politis-Romano percentile-CI convention).
    for w in taus_a.windows(2) {
        assert!(w[0] <= w[1]);
    }
    // CI helper round-trips through the sorted output.
    let (lo, hi) = ci_90_from_sorted(&taus_a).expect("ci_90 over non-empty");
    assert!(lo <= hi);
    assert!((0.0..=1.0).contains(&lo));
    assert!((0.0..=1.0).contains(&hi));
}
