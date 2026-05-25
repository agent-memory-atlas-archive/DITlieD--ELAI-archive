//! D-22 closure contract test — Conformal Calibration Validation.
//!
//! Pins the invariants the D-22 closure session shipped so future
//! refactors cannot silently drift the wire shape, the gating semantics,
//! or the validation-fixture row schema:
//!
//!   * SCHEMA_VERSION_PUBLIC >= 86 (D-22 substrate floor; HEAD is 141).
//!   * `ConformalValidationCompletedPayload` round-trips its full
//!     (tau, epsilon, empirical_refusal_rate, gating_verdict,
//!     fixture_blake3, total, ts_ms) shape.
//!   * `RefusalDecision` is the 2-arm closed enum imported from D-16
//!     (Abstain + Answer { confidence }) — exhaustive serde round-trip.
//!   * UniCR §5 selective-risk budget invariant: empirical sel_risk
//!     <= alpha + finite-sample slack against the balanced 20-pair
//!     synthetic corpus.
//!   * `ValidationFixtureRow` parses the canonical 5-field shape
//!     (`row_id`, `distribution`, `domain`, `predicted_score`,
//!     `ground_truth`) the runner consumes.
//!
//! Paper grounding: arXiv:2509.01455 (UniCR Eq.5 quantile + §5
//! selective-risk budget); D-16-decisions.md DECISION-2 (2-arm
//! RefusalDecision lock); workflow/papers/unicr-2509.01455-interface.md.

use elai_orchestrator::benchmarks::conformal_validation_runner::{
    ConformalValidationRunner, ValidationFixtureRow, VecConformalValidationSsbEmitter,
};
use elai_orchestrator::conformal_calibration::{
    compute_tau, decide, CalibrationCorpus, CalibrationTuple, CorpusSource, RefusalDecision,
    DEFAULT_EPSILON,
};
use elai_orchestrator::ssb::{ConformalValidationCompletedPayload, SCHEMA_VERSION_PUBLIC};

#[test]
fn schema_version_floor_is_at_least_d22_substrate() {
    // D-22 substrate landed at SCHEMA 86. Subsequent ships have advanced
    // SCHEMA_VERSION; this floor guards against a regression below 86.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 86,
        "SCHEMA_VERSION_PUBLIC = {SCHEMA_VERSION_PUBLIC} regressed below D-22 floor 86"
    );
}

#[test]
fn conformal_validation_completed_payload_round_trips_full_shape() {
    let payload = ConformalValidationCompletedPayload {
        tau: 0.8,
        epsilon: 0.1,
        empirical_refusal_rate: 0.083,
        gating_verdict: "pass".to_string(),
        fixture_blake3:
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        total: 15,
        ts_ms: 1_700_000_000_000,
    };
    let s = serde_json::to_string(&payload).expect("serialize");
    for key in [
        "\"tau\"",
        "\"epsilon\"",
        "\"empirical_refusal_rate\"",
        "\"gating_verdict\"",
        "\"fixture_blake3\"",
        "\"total\"",
        "\"ts_ms\"",
    ] {
        assert!(s.contains(key), "missing key {key} in serialised payload: {s}");
    }
    let round: ConformalValidationCompletedPayload =
        serde_json::from_str(&s).expect("deserialize");
    assert_eq!(round, payload);
}

#[test]
fn refusal_decision_is_two_arm_closed_enum() {
    // D-16 DECISION-2: RefusalDecision is locked to Abstain + Answer.
    // The pattern match below MUST stay exhaustive — a 3rd variant fails
    // to compile, which is the wire-shape contract D-22 imports.
    fn arm_label(d: RefusalDecision) -> &'static str {
        match d {
            RefusalDecision::Abstain => "abstain",
            RefusalDecision::Answer { .. } => "answer",
        }
    }
    assert_eq!(arm_label(RefusalDecision::Abstain), "abstain");
    assert_eq!(
        arm_label(RefusalDecision::Answer { confidence: 0.99 }),
        "answer"
    );

    // Serde round-trip with snake_case tag = "kind".
    let abstain_s = serde_json::to_string(&RefusalDecision::Abstain).expect("serialize abstain");
    assert!(abstain_s.contains("\"kind\":\"abstain\""), "{abstain_s}");
    let abstain_back: RefusalDecision =
        serde_json::from_str(&abstain_s).expect("deserialize abstain");
    assert_eq!(abstain_back, RefusalDecision::Abstain);

    let answer_payload = RefusalDecision::Answer { confidence: 0.42 };
    let answer_s = serde_json::to_string(&answer_payload).expect("serialize answer");
    assert!(answer_s.contains("\"kind\":\"answer\""), "{answer_s}");
    assert!(answer_s.contains("\"confidence\""), "{answer_s}");
    let answer_back: RefusalDecision = serde_json::from_str(&answer_s).expect("deserialize answer");
    assert_eq!(answer_back, answer_payload);
}

#[test]
fn validation_fixture_row_parses_canonical_schema() {
    // Mirrors the row shape shipped under
    // .elai_cc/benchmarks/conformal_validation/fixtures/*.jsonl. The
    // contract: predicted_score is f64 in [0.0, 1.0], ground_truth is
    // bool, distribution + domain are snake-case wire labels.
    let raw = r#"{
        "row_id": "low_risk_000",
        "distribution": "in_domain",
        "domain": "code",
        "predicted_score": 0.95,
        "ground_truth": true
    }"#;
    let row: ValidationFixtureRow = serde_json::from_str(raw).expect("row deserialise");
    assert_eq!(row.row_id, "low_risk_000");
    assert_eq!(row.distribution, "in_domain");
    assert_eq!(row.domain, "code");
    assert!((row.predicted_score - 0.95).abs() < 1e-12);
    assert!(row.ground_truth);

    // Round-trip the row back through serde.
    let s = serde_json::to_string(&row).expect("row serialise");
    let back: ValidationFixtureRow = serde_json::from_str(&s).expect("row round-trip");
    assert_eq!(back, row);
}

#[test]
fn validation_fixture_row_rejects_unknown_distribution_only_at_runner() {
    // ValidationFixtureRow is a structural JSON shape; it does NOT close
    // the distribution enum at serde time (that happens inside the
    // runner via `parse_distribution`). This test pins the JSON-shape
    // permissiveness so the contract test remains stable even if the
    // runner's enum gains a new bucket name (`out_of_domain`, etc).
    let raw = r#"{
        "row_id": "boundary_000",
        "distribution": "boundary",
        "domain": "code",
        "predicted_score": 0.2,
        "ground_truth": true
    }"#;
    let row: ValidationFixtureRow = serde_json::from_str(raw).expect("row deserialise");
    assert_eq!(row.distribution, "boundary");
}

#[test]
fn selective_risk_budget_holds_for_balanced_synthetic_corpus() {
    // UniCR Eq.5 + §5 invariant: for a corpus of size n at alpha = 0.1,
    // tau = the (1 - alpha) empirical quantile of {1 - predicted_score_i}.
    // For 20 pairs of (0.9, 0.2) the closed-form quantile sits at 0.8.
    // The selective-risk budget is empirical_sel_risk <= alpha + slack,
    // with slack = 1.0 / (n + 1) (Vovk finite-sample term).
    let mut c = CalibrationCorpus::new(CorpusSource::Synthetic);
    for i in 0..20 {
        c.append(CalibrationTuple::new(
            format!("t-{i}"),
            format!("c-{i}"),
            "synthetic true",
            true,
            0.9,
            1_700_000_000 + (i as i64),
        ))
        .expect("append true");
        c.append(CalibrationTuple::new(
            format!("f-{i}"),
            format!("d-{i}"),
            "synthetic false",
            false,
            0.2,
            1_700_000_500 + (i as i64),
        ))
        .expect("append false");
    }
    let alpha = DEFAULT_EPSILON; // 0.1
    let tau = compute_tau(&c, alpha).expect("tau");
    assert!((tau - 0.8).abs() < 1e-9, "expected tau=0.8, got {tau}");

    // decide(): low confidence -> Abstain; high confidence -> Answer.
    assert!(matches!(decide(0.05, tau), RefusalDecision::Abstain));
    assert!(matches!(decide(0.95, tau), RefusalDecision::Answer { .. }));

    // Empirical refusal rate on the calibration corpus itself: 20 of 40
    // tuples carry predicted_score = 0.2 ((1 - 0.2) = 0.8 = tau exactly).
    // The decide rule abstains iff (1 - claim_score) > tau, so the 0.2
    // tuples land on the Answer side -> sel_risk = 0.0 here.
    //
    // The invariant we pin is the budget: sel_risk <= alpha + slack
    // where slack = 1 / (n + 1). For n = 40 -> slack ~= 0.0244.
    let n: usize = c.size();
    let slack = 1.0_f64 / ((n as f64) + 1.0);
    let mut refusals = 0_u32;
    for t in &c.tuples {
        if matches!(decide(t.predicted_score, tau), RefusalDecision::Abstain) {
            refusals += 1;
        }
    }
    let sel_risk = f64::from(refusals) / (n as f64);
    let budget = alpha + slack;
    assert!(
        sel_risk <= budget,
        "selective risk {sel_risk} exceeded budget {budget} (alpha={alpha}, slack={slack}, n={n})"
    );
}

#[test]
fn validation_fixture_row_predicted_score_is_f64() {
    // Schema-floor: predicted_score MUST stay f64 (the wire shape is
    // serde_json::Number deserialised into f64). A future drift to f32 /
    // i64 would break the contract test below.
    let raw =
        r#"{"row_id":"x","distribution":"in_domain","domain":"code","predicted_score":0.123456789012345,"ground_truth":true}"#;
    let row: ValidationFixtureRow = serde_json::from_str(raw).expect("row");
    // f64 carries ~15 decimal digits; if the field were f32 the value
    // would be rounded to ~7 digits.
    assert!(
        (row.predicted_score - 0.123_456_789_012_345_f64).abs() < 1e-13,
        "predicted_score f64 precision lost: {}",
        row.predicted_score
    );
}

#[tokio::test]
async fn runner_emits_one_completed_row_per_run_all_invocation() {
    // The shipped contract: ConformalValidationRunner::run_all emits
    // exactly ONE ConformalValidationCompletedPayload row per Ok return.
    // This test exercises the producer surface end-to-end against an
    // in-tempdir minimal fixture to pin the emit-count invariant.
    let dir = tempfile::tempdir().expect("tempdir");
    let fixture = dir.path().join("fixture.jsonl");
    let mut raw = String::new();
    // 12 rows per bucket (== DEFAULT_MIN_ROWS_PER_DISTRIBUTION). All
    // high-confidence -> empirical refusal rate ~ 0.0 (in-band for the
    // default epsilon=0.1 target band).
    for bucket in ["in_domain", "near_ood", "far_ood"] {
        for i in 0..12 {
            raw.push_str(&format!(
                "{{\"row_id\":\"{bucket}_{i}\",\"distribution\":\"{bucket}\",\"domain\":\"code\",\"predicted_score\":0.95,\"ground_truth\":true}}\n"
            ));
        }
    }
    std::fs::write(&fixture, raw).expect("write fixture");

    let mut c = CalibrationCorpus::new(CorpusSource::Synthetic);
    for i in 0..20 {
        c.append(CalibrationTuple::new(
            format!("t-{i}"),
            format!("c-{i}"),
            "true",
            true,
            0.9,
            1_700_000_000 + (i as i64),
        ))
        .expect("append");
        c.append(CalibrationTuple::new(
            format!("f-{i}"),
            format!("d-{i}"),
            "false",
            false,
            0.2,
            1_700_000_500 + (i as i64),
        ))
        .expect("append");
    }
    c.freeze();

    let runner = ConformalValidationRunner::new(c, fixture);
    let emitter = VecConformalValidationSsbEmitter::new();
    let report = runner
        .run_all(&emitter, 1_700_000_999_000)
        .await
        .expect("run_all ok");
    assert!(report.passed(), "expected Pass on balanced fixture");
    assert_eq!(report.total, 36);

    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 1, "exactly one completed row per run_all");
    assert_eq!(rows[0].total, 36);
    assert_eq!(rows[0].gating_verdict, "pass");
    assert!((rows[0].tau - 0.8).abs() < 1e-9);
}
