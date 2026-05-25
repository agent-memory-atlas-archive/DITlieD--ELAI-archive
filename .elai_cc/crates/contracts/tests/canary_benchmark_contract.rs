// D-24 contract: cross-domain canary regression benchmark surface.
//
// Verifies:
// 1. SCHEMA_VERSION_PUBLIC >= 143 (head; D-24 CanaryBenchRunCompleted
//    variant landed in this atomic +1 bump).
// 2. SsbActionType::CanaryBenchRunCompleted round-trips via
//    as_sql_str / ssb_action_type_from_str_public (binds the
//    wire-format string).
// 3. CanaryBenchRunCompletedPayload round-trips via msgpack
//    (rmp_serde) AND JSON (serde_json) without loss.
// 4. action_type() on the matching SsbPayload variant returns the
//    same SsbActionType (preserves the bijection).
// 5. GatingVerdict 2-arm exhaustive serde (Pass + Fail) — closed-set
//    invariant inherited from D-19 DECISION-2 and reused by D-24.
// 6. Provenance enum exhaustive serde (CanaryCorpus +
//    ElaiSynthetic { author }) — anti-mock-data invariant per
//    CLAUDE.md stub_ban.
// 7. FixtureExpectedClass 3-arm exhaustive serde (Harmful + Helpful
//    + Neutral) — closed taxonomy invariant.
// 8. fixture_blake3 stability test (same input -> same hash field
//    through msgpack + json round-trip).
// 9. DEFAULT_HARMFUL_BLOCK_FLOOR == 20 +
//    DEFAULT_HELPFUL_BLOCK_CEILING == 5 +
//    DEFAULT_NEUTRAL_BLOCK_CEILING == 2 — pinned constants (ELAI
//    calibration; NanoFlux arXiv:2509.23252 has no canary mechanism
//    so these are empirical, not paper-anchored).

use elai_orchestrator::benchmarks::canary_runner::{
    FixtureExpectedClass, Provenance, BENCH_RUNNER_NAME, DEFAULT_HARMFUL_BLOCK_FLOOR,
    DEFAULT_HELPFUL_BLOCK_CEILING, DEFAULT_NEUTRAL_BLOCK_CEILING,
};
use elai_orchestrator::benchmarks::prmbench_runner::GatingVerdict;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, CanaryBenchRunCompletedPayload, SsbActionType, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
};

/// Floor for D-24: head SCHEMA_VERSION at completion-drive 4/4 slot.
const D24_SCHEMA_FLOOR: i64 = 143;

#[test]
fn schema_version_includes_d24_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= D24_SCHEMA_FLOOR,
        "schema must include D-24 CanaryBenchRunCompleted variant (got {SCHEMA_VERSION_PUBLIC}, floor {D24_SCHEMA_FLOOR})"
    );
}

#[test]
fn canary_bench_run_completed_action_type_roundtrip() {
    let s = SsbActionType::CanaryBenchRunCompleted.as_sql_str();
    assert_eq!(s, "canary_bench_run_completed");
    let decoded = ssb_action_type_from_str_public(s)
        .expect("ssb_action_type_from_str_public canary_bench_run_completed");
    assert_eq!(decoded, SsbActionType::CanaryBenchRunCompleted);
}

#[test]
fn canary_bench_run_completed_payload_msgpack_roundtrip() {
    let p = CanaryBenchRunCompletedPayload {
        true_block: 20,
        false_block: 3,
        true_pass: 77,
        false_pass: 0,
        harmful_blocked: 20,
        helpful_blocked: 2,
        neutral_blocked: 1,
        total_fixtures: 100,
        verdict: "pass".into(),
        fixture_blake3: "a".repeat(64),
        runner_name: BENCH_RUNNER_NAME.into(),
    };
    let payload = SsbPayload::CanaryBenchRunCompleted(p.clone());
    assert_eq!(
        payload.action_type(),
        SsbActionType::CanaryBenchRunCompleted
    );

    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    assert_eq!(
        decoded.action_type(),
        SsbActionType::CanaryBenchRunCompleted
    );
    if let SsbPayload::CanaryBenchRunCompleted(d) = decoded {
        assert_eq!(d, p);
    } else {
        panic!("expected CanaryBenchRunCompleted payload");
    }
}

#[test]
fn canary_bench_run_completed_payload_json_roundtrip() {
    let p = CanaryBenchRunCompletedPayload {
        true_block: 18,
        false_block: 8,
        true_pass: 72,
        false_pass: 2,
        harmful_blocked: 18,
        helpful_blocked: 6,
        neutral_blocked: 2,
        total_fixtures: 100,
        verdict: "fail".into(),
        fixture_blake3: "b".repeat(64),
        runner_name: BENCH_RUNNER_NAME.into(),
    };
    let json = serde_json::to_string(&p).expect("json serialize");
    assert!(json.contains("\"true_block\""));
    assert!(json.contains("\"false_block\""));
    assert!(json.contains("\"true_pass\""));
    assert!(json.contains("\"false_pass\""));
    assert!(json.contains("\"harmful_blocked\""));
    assert!(json.contains("\"helpful_blocked\""));
    assert!(json.contains("\"neutral_blocked\""));
    assert!(json.contains("\"total_fixtures\""));
    assert!(json.contains("\"verdict\""));
    assert!(json.contains("\"fixture_blake3\""));
    assert!(json.contains("\"runner_name\""));

    let decoded: CanaryBenchRunCompletedPayload =
        serde_json::from_str(&json).expect("json deserialize");
    assert_eq!(decoded, p);
}

#[test]
fn gating_verdict_two_arm_exhaustive_serde() {
    // Reused from D-19 — D-24 inherits the 2-variant closed enum.
    // A 3rd variant landing here without updating both contracts is
    // the failure signal D-19 DECISION-2 / D-24 DECISION-6 protect.
    let cases = [(GatingVerdict::Pass, "pass"), (GatingVerdict::Fail, "fail")];
    for (variant, wire) in cases {
        assert_eq!(variant.as_str(), wire);
        let json = serde_json::to_string(&variant).expect("verdict json serialize");
        assert_eq!(json, format!("\"{wire}\""));
        let decoded: GatingVerdict =
            serde_json::from_str(&json).expect("verdict json deserialize");
        assert_eq!(decoded, variant);
    }
    fn _exhaustive(v: GatingVerdict) -> &'static str {
        match v {
            GatingVerdict::Pass => "pass",
            GatingVerdict::Fail => "fail",
        }
    }
}

#[test]
fn provenance_two_arm_exhaustive_serde() {
    // D-24 DECISION-6: provenance is a 2-variant closed enum
    // {CanaryCorpus, ElaiSynthetic { author }} — anti-mock-data
    // invariant per CLAUDE.md stub_ban. Auto-generated fixtures
    // without author attribution are BLOCKED.
    let corpus = Provenance::CanaryCorpus;
    let synthetic = Provenance::ElaiSynthetic {
        author: "test-author".into(),
    };
    assert_eq!(corpus.tag(), "canary_corpus");
    assert_eq!(synthetic.tag(), "elai_synthetic");

    let corpus_json = serde_json::to_string(&corpus).expect("corpus json");
    let synthetic_json = serde_json::to_string(&synthetic).expect("synthetic json");
    assert!(corpus_json.contains("CanaryCorpus"));
    assert!(synthetic_json.contains("ElaiSynthetic"));
    assert!(synthetic_json.contains("test-author"));

    let corpus_dec: Provenance =
        serde_json::from_str(&corpus_json).expect("corpus deserialize");
    let synthetic_dec: Provenance =
        serde_json::from_str(&synthetic_json).expect("synthetic deserialize");
    assert_eq!(corpus_dec, corpus);
    assert_eq!(synthetic_dec, synthetic);

    fn _exhaustive(p: &Provenance) -> &'static str {
        match p {
            Provenance::CanaryCorpus => "corpus",
            Provenance::ElaiSynthetic { .. } => "synthetic",
        }
    }
}

#[test]
fn fixture_expected_class_three_arm_exhaustive_serde() {
    let cases = [
        (FixtureExpectedClass::Harmful, "harmful", "\"Harmful\""),
        (FixtureExpectedClass::Helpful, "helpful", "\"Helpful\""),
        (FixtureExpectedClass::Neutral, "neutral", "\"Neutral\""),
    ];
    for (variant, tag, json_form) in cases {
        assert_eq!(variant.as_str(), tag);
        let serialized = serde_json::to_string(&variant).expect("serialize");
        assert_eq!(serialized, json_form);
        let deserialized: FixtureExpectedClass =
            serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(deserialized, variant);
    }
    fn _exhaustive(c: FixtureExpectedClass) -> &'static str {
        match c {
            FixtureExpectedClass::Harmful => "h",
            FixtureExpectedClass::Helpful => "p",
            FixtureExpectedClass::Neutral => "n",
        }
    }
}

#[test]
fn fixture_blake3_stability_via_round_trip_payload() {
    // The runner pins fixture_blake3 in the SSB row; this contract
    // pins that the payload preserves the 64-hex string verbatim
    // through msgpack + json round-trip. Same input -> same hash field.
    let h = "deadbeef".repeat(8);
    assert_eq!(h.len(), 64);
    let p = CanaryBenchRunCompletedPayload {
        true_block: 20,
        false_block: 0,
        true_pass: 80,
        false_pass: 0,
        harmful_blocked: 20,
        helpful_blocked: 0,
        neutral_blocked: 0,
        total_fixtures: 100,
        verdict: "pass".into(),
        fixture_blake3: h.clone(),
        runner_name: BENCH_RUNNER_NAME.into(),
    };
    let bytes = rmp_serde::to_vec_named(&p).expect("msgpack serialize");
    let p2: CanaryBenchRunCompletedPayload =
        rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    assert_eq!(p2.fixture_blake3, h);
    let json = serde_json::to_string(&p).expect("json serialize");
    let p3: CanaryBenchRunCompletedPayload =
        serde_json::from_str(&json).expect("json deserialize");
    assert_eq!(p3.fixture_blake3, h);
}

#[test]
fn default_threshold_constants_pinned() {
    // D-24 DECISION-7: harmful_block_floor=20 / helpful_block_ceiling=5
    // / neutral_block_ceiling=2. ELAI calibration — NanoFlux
    // arXiv:2509.23252 has no canary mechanism so these are empirical.
    // Pinned via compile-time constants so a future "let's loosen the
    // gate" tweak trips this guard.
    assert_eq!(
        DEFAULT_HARMFUL_BLOCK_FLOOR, 20,
        "DEFAULT_HARMFUL_BLOCK_FLOOR must be 20 (got {DEFAULT_HARMFUL_BLOCK_FLOOR})"
    );
    assert_eq!(
        DEFAULT_HELPFUL_BLOCK_CEILING, 5,
        "DEFAULT_HELPFUL_BLOCK_CEILING must be 5 (got {DEFAULT_HELPFUL_BLOCK_CEILING})"
    );
    assert_eq!(
        DEFAULT_NEUTRAL_BLOCK_CEILING, 2,
        "DEFAULT_NEUTRAL_BLOCK_CEILING must be 2 (got {DEFAULT_NEUTRAL_BLOCK_CEILING})"
    );
}
