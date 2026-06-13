//! D-16 closure contract test — UniCR conformal-refusal calibration.
//!
//! Pins the invariants the D-16 closure session shipped so future
//! refactors don't silently drift:
//!
//!   * SCHEMA_VERSION >= 127 (D-16 closure target after C-52 took
//!     head to 126).
//!   * `RefusalDecision` stays at 2 variants (Abstain, Answer) — the
//!     plan's 4-variant expansion was rejected per D-16-decisions.md
//!     DECISION-2 because D-22 imports the 2-variant shape.
//!   * `ConformalRefusalDecidedPayload` round-trips with the new
//!     optional `reason` field on Abstain rows AND with reason=None
//!     for backward compatibility.
//!   * `CalibrationCorpusUpdatedPayload` round-trips its full
//!     (corpus_size, new_tau, alpha, source, ts_ms) shape.
//!   * `TauUpdater::update_tau` is deterministic — same corpus on
//!     disk -> same tau on disk across two consecutive runs.
//!   * UniCR Eq.5 quantile formula: tau = the (1 - alpha) empirical
//!     quantile of `{1 - predicted_score_i}`. The closed-form check
//!     uses the (20 pairs of 0.9 / 0.2) balanced corpus the
//!     orchestrator's own unit test exercises.
//!
//! Paper grounding: arXiv:2509.01455 (UniCR Eq.5 CRC quantile
//! formula). alpha (miscoverage rate) and rho (selective risk budget)
//! are DISTINCT quantities per D-16-decisions.md DECISION-4 — only
//! alpha is on the SSB row; rho lives on the runtime config only.

use elai_orchestrator::calibration::{
    compute_tau, decide, decide_and_emit_with_reason, tau_path_default,
    update_tau_and_emit, CalibrationCorpus, CalibrationTuple, CorpusSource,
    RefusalDecision, RefusalReason, TauUpdater, VecCalibrationCorpusUpdatedEmitter,
    VecConformalRefusalDecidedEmitter, DEFAULT_EPSILON, TAU_DEFAULT_FALLBACK,
};
use elai_orchestrator::ssb::{
    CalibrationCorpusUpdatedPayload, ConformalRefusalDecidedPayload,
    SCHEMA_VERSION_PUBLIC,
};

#[test]
fn schema_version_floor_is_at_least_127() {
    // D-16 closure raised the schema from 126 -> 127 for the new
    // CalibrationCorpusUpdated variant. Future bumps must NOT slide
    // back below 127.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 127,
        "SCHEMA_VERSION_PUBLIC = {SCHEMA_VERSION_PUBLIC} regressed below 127"
    );
}

#[test]
fn refusal_decision_stays_two_variant() {
    // Closed enum sanity — `RefusalDecision` exists with exactly the
    // Abstain + Answer arms. Pattern-match exhaustiveness here means
    // a 3rd variant would fail to compile, which is the contract.
    let abstain = RefusalDecision::Abstain;
    let answer = RefusalDecision::Answer { confidence: 0.99 };
    fn arm_label(d: RefusalDecision) -> &'static str {
        match d {
            RefusalDecision::Abstain => "abstain",
            RefusalDecision::Answer { .. } => "answer",
        }
    }
    assert_eq!(arm_label(abstain), "abstain");
    assert_eq!(arm_label(answer), "answer");
}

#[test]
fn conformal_refusal_decided_payload_round_trips_with_reason() {
    let payload = ConformalRefusalDecidedPayload {
        claim_id: "c-contract-1".into(),
        claim_score: 0.05,
        tau: 0.8,
        decision: "abstain".into(),
        confidence: 0.0,
        ts_ms: 1_700_000_000_000,
        reason: Some(RefusalReason::LowEvidence.as_str().to_string()),
        calibration_id: None,
    };
    let s = serde_json::to_string(&payload).expect("serialize");
    assert!(s.contains("\"reason\""), "reason key must appear: {s}");
    assert!(
        s.contains("\"low_evidence\""),
        "low_evidence wire label must appear: {s}"
    );
    let round: ConformalRefusalDecidedPayload =
        serde_json::from_str(&s).expect("deserialize");
    assert_eq!(round, payload);
}

#[test]
fn conformal_refusal_decided_payload_round_trips_without_reason() {
    // Backward compatibility: rows emitted before the reason field
    // landed MUST deserialize as reason=None via #[serde(default)].
    let raw = r#"{
        "claim_id": "c-legacy",
        "claim_score": 0.05,
        "tau": 0.8,
        "decision": "abstain",
        "confidence": 0.0,
        "ts_ms": 1700000000000
    }"#;
    let p: ConformalRefusalDecidedPayload =
        serde_json::from_str(raw).expect("legacy row must parse");
    assert!(p.reason.is_none(), "legacy row must default reason=None");
    assert_eq!(p.claim_id, "c-legacy");
}

#[test]
fn calibration_corpus_updated_payload_round_trips() {
    let payload = CalibrationCorpusUpdatedPayload {
        corpus_size: 40,
        new_tau: 0.8,
        alpha: 0.1,
        source: "synthetic".into(),
        ts_ms: 1_700_000_127_000,
    };
    let s = serde_json::to_string(&payload).expect("serialize");
    for key in [
        "\"corpus_size\"",
        "\"new_tau\"",
        "\"alpha\"",
        "\"source\"",
        "\"ts_ms\"",
    ] {
        assert!(s.contains(key), "missing key {key} in {s}");
    }
    let round: CalibrationCorpusUpdatedPayload =
        serde_json::from_str(&s).expect("deserialize");
    assert_eq!(round, payload);
}

#[test]
fn tau_path_default_resolves_under_elai_data() {
    let p = tau_path_default();
    let s = p.to_string_lossy();
    assert!(
        s.contains(".elai_cc") && s.ends_with("conformal_tau.txt"),
        "tau path default must live under .elai_cc/data/: {s}"
    );
}

#[test]
fn tau_updater_is_deterministic_on_same_corpus() {
    let dir = tempfile::tempdir().expect("tempdir");
    let corpus_path = dir.path().join("seed.jsonl");
    let tau_path = dir.path().join("tau.txt");
    let mut raw = String::new();
    for i in 0..20 {
        raw.push_str(&format!(
            "{{\"tuple_id\":\"t-{i}\",\"claim_id\":\"c-{i}\",\"claim_text\":\"x\",\"ground_truth\":true,\"predicted_score\":0.9,\"ts_ms\":{}}}\n",
            1_700_000_000 + (i as i64)
        ));
        raw.push_str(&format!(
            "{{\"tuple_id\":\"f-{i}\",\"claim_id\":\"d-{i}\",\"claim_text\":\"y\",\"ground_truth\":false,\"predicted_score\":0.2,\"ts_ms\":{}}}\n",
            1_700_000_500 + (i as i64)
        ));
    }
    std::fs::write(&corpus_path, raw).expect("write");
    let updater = TauUpdater::new(corpus_path, tau_path, 0.1);
    let a = updater.update_tau().expect("first update");
    let b = updater.update_tau().expect("second update");
    assert!(
        (a - b).abs() < 1e-12,
        "same corpus must yield same tau: {a} vs {b}"
    );
}

#[test]
fn tau_updater_missing_corpus_returns_fallback() {
    let dir = tempfile::tempdir().expect("tempdir");
    let updater = TauUpdater::new(
        dir.path().join("missing.jsonl"),
        dir.path().join("tau.txt"),
        DEFAULT_EPSILON,
    );
    let t = updater.update_tau().expect("update_tau");
    assert!((t - TAU_DEFAULT_FALLBACK).abs() < 1e-12);
}

#[test]
fn unicr_eq5_quantile_formula_matches_closed_form() {
    // 20 pairs (40 tuples): 20 with predicted_score=0.9
    // (non-conformity 0.1) and 20 with predicted_score=0.2
    // (non-conformity 0.8). At alpha=0.1 the (1-alpha) quantile sits
    // in the high-non-conformity tail, so tau = 0.8 by the UniCR
    // Eq.5 formula (Vovk §2.2 closed form: idx =
    // ceil((1 - alpha) * (n + 1)) - 1, capped at n - 1).
    let mut c = CalibrationCorpus::new(CorpusSource::HeldOut);
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
    let tau = compute_tau(&c, 0.1).expect("tau");
    assert!(
        (tau - 0.8).abs() < 1e-9,
        "UniCR Eq.5 closed form expected tau=0.8; got {tau}"
    );
    // And the decide rule rejects a low-confidence claim against
    // this tau.
    assert!(matches!(decide(0.05, tau), RefusalDecision::Abstain));
    assert!(matches!(
        decide(0.95, tau),
        RefusalDecision::Answer { .. }
    ));
}

#[tokio::test]
async fn decide_and_emit_with_reason_stamps_reason_on_abstain_only() {
    let emitter = VecConformalRefusalDecidedEmitter::new();
    let _ = decide_and_emit_with_reason(
        "c-abstain",
        0.01,
        0.5,
        "aaaaaaaaaaaaaaaa",
        Some(RefusalReason::ToolInconsistency),
        &emitter,
        100,
    )
    .await;
    let _ = decide_and_emit_with_reason(
        "c-answer",
        0.99,
        0.5,
        "aaaaaaaaaaaaaaaa",
        Some(RefusalReason::VerifierRejection),
        &emitter,
        200,
    )
    .await;
    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].decision, "abstain");
    assert_eq!(rows[0].reason.as_deref(), Some("tool_inconsistency"));
    assert_eq!(rows[1].decision, "answer");
    assert!(
        rows[1].reason.is_none(),
        "Answer rows must drop the reason regardless of caller intent"
    );
}

#[tokio::test]
async fn update_tau_and_emit_emits_calibration_corpus_updated_row() {
    let dir = tempfile::tempdir().expect("tempdir");
    let corpus_path = dir.path().join("seed.jsonl");
    let tau_path = dir.path().join("tau.txt");
    let mut raw = String::new();
    for i in 0..20 {
        raw.push_str(&format!(
            "{{\"tuple_id\":\"t-{i}\",\"claim_id\":\"c-{i}\",\"claim_text\":\"x\",\"ground_truth\":true,\"predicted_score\":0.9,\"ts_ms\":{}}}\n",
            1_700_000_000 + (i as i64)
        ));
        raw.push_str(&format!(
            "{{\"tuple_id\":\"f-{i}\",\"claim_id\":\"d-{i}\",\"claim_text\":\"y\",\"ground_truth\":false,\"predicted_score\":0.2,\"ts_ms\":{}}}\n",
            1_700_000_500 + (i as i64)
        ));
    }
    std::fs::write(&corpus_path, raw).expect("write");
    let updater = TauUpdater::new(corpus_path, tau_path, 0.1);
    let emitter = VecCalibrationCorpusUpdatedEmitter::new();
    let tau = update_tau_and_emit(&updater, "auto", &emitter, 1_700_000_999_000)
        .await
        .expect("update_tau_and_emit");
    assert!((tau - 0.8).abs() < 1e-9);
    let rows = emitter.rows().await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].corpus_size, 40);
    assert!((rows[0].new_tau - 0.8).abs() < 1e-9);
    // DECISION-4: only alpha (miscoverage rate) lives on the row;
    // rho (selective risk budget) is decision-time only.
    assert!((rows[0].alpha - 0.1).abs() < 1e-9);
    assert_eq!(rows[0].source, "auto");
}
