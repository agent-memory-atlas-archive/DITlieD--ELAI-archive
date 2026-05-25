// D-28 contract: Adversarial example farming runner surface.
//
// Verifies (atomic SCHEMA bump 144 -> 145, +1 SsbActionType variant):
// 1. SCHEMA_VERSION_PUBLIC >= 145 (D-28 floor; AdversarialFarmRunCompleted
//    variant landed at this bump).
// 2. SsbActionType::AdversarialFarmRunCompleted + payload round-trip via
//    as_sql_str / ssb_action_type_from_str_public + JSON + msgpack
//    (binds the wire-format string + payload shape).
// 3. GatingVerdict 2-arm closed enum {Pass, Fail} reused from D-19 — pinned
//    so the D-28 verdict surface inherits the binary-gate design.
// 4. Z>1.96 invariant: DEFAULT_Z_THRESHOLD == 1.96 (paper arXiv:2504.06141
//    §3.2 95th-pct uncertainty filter). Hard-pinned constant; drift in
//    either direction trips this test.
// 5. max_samples <= MAX_SAMPLES_CAP (paper §5 saturation guard; the runner
//    surfaces InvalidMaxSamples beyond the cap).
// 6. max_rounds <= MAX_ROUNDS_CAP (paper §5 round-3 plateau; <=2 hard
//    ceiling).
// 7. paper_floor_pp default >= 1.0 (calibrated from paper-reported +0.7pp
//    overall delta -> 1.0pp conservative ELAI integer-pp floor; the plan
//    text's UNGROUNDED "+5pp" REJECTED per D-28-decisions.md DECISION-3).

use elai_orchestrator::benchmarks::prmbench_runner::GatingVerdict;
use elai_orchestrator::farm::adv_rm::{
    AdvRmFarmRunReport, DEFAULT_MAX_ROUNDS, DEFAULT_MAX_SAMPLES, DEFAULT_PAPER_FLOOR_PP,
    DEFAULT_Z_THRESHOLD, MAX_ROUNDS_CAP, MAX_SAMPLES_CAP,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, AdversarialFarmRunCompletedPayload, SsbActionType,
    SsbPayload, SCHEMA_VERSION_PUBLIC,
};

/// Floor for D-28: head SCHEMA_VERSION at completion-drive close.
/// Atomic +1 bump from D-25's 144 head (single new variant
/// AdversarialFarmRunCompleted; see D-28-decisions.md DECISION-1).
const D28_SCHEMA_FLOOR: i64 = 145;

#[test]
fn schema_version_includes_d28_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= D28_SCHEMA_FLOOR,
        "schema must include D-28 AdversarialFarmRunCompleted variant (got {SCHEMA_VERSION_PUBLIC}, floor {D28_SCHEMA_FLOOR})"
    );
}

#[test]
fn adversarial_farm_run_completed_payload_roundtrip() {
    // Bind the SQL wire tag + the round-trip identity. The variant tag is
    // the canonical snake_case wire format the lockstep cascade is pinned
    // to (action_type_lock.golden ordinal 0000).
    let tag = SsbActionType::AdversarialFarmRunCompleted.as_sql_str();
    assert_eq!(tag, "adversarial_farm_run_completed");
    let decoded = ssb_action_type_from_str_public(tag)
        .expect("ssb_action_type_from_str_public adversarial_farm_run_completed");
    assert_eq!(decoded, SsbActionType::AdversarialFarmRunCompleted);

    let p = AdversarialFarmRunCompletedPayload {
        samples_farmed: 1000,
        rounds: 2,
        prmbench_delta_pp: 1.5,
        verdict: "pass".into(),
        paper_floor_pp: 1.0,
        corpus_blake3: "a".repeat(64),
    };
    let payload = SsbPayload::AdversarialFarmRunCompleted(p.clone());
    assert_eq!(
        payload.action_type(),
        SsbActionType::AdversarialFarmRunCompleted
    );

    // JSON round-trip preserves payload byte semantics.
    let json = serde_json::to_string(&p).expect("payload json serialize");
    let decoded_p: AdversarialFarmRunCompletedPayload =
        serde_json::from_str(&json).expect("payload json deserialize");
    assert_eq!(decoded_p, p);

    // msgpack round-trip via SsbPayload outer envelope (mirrors
    // prmbench_contract msgpack pattern).
    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack serialize");
    let decoded_env: SsbPayload =
        rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    assert_eq!(
        decoded_env.action_type(),
        SsbActionType::AdversarialFarmRunCompleted
    );
    if let SsbPayload::AdversarialFarmRunCompleted(d) = decoded_env {
        assert_eq!(d, p);
    } else {
        panic!("expected AdversarialFarmRunCompleted payload");
    }
}

#[test]
fn gating_verdict_two_arm_serde_inherited_from_d19() {
    // D-28 reuses the D-19 GatingVerdict closed enum unchanged
    // (D-28-decisions.md DECISION-8). Pin the 2-variant surface so a
    // drift in either side trips this contract.
    let cases = [
        (GatingVerdict::Pass, "pass"),
        (GatingVerdict::Fail, "fail"),
    ];
    for (variant, wire) in cases {
        assert_eq!(variant.as_str(), wire);
        let json = serde_json::to_string(&variant).expect("verdict json serialize");
        assert_eq!(json, format!("\"{wire}\""));
        let decoded: GatingVerdict =
            serde_json::from_str(&json).expect("verdict json deserialize");
        assert_eq!(decoded, variant);
    }
    // Exhaustive match — if a 3rd variant lands without an ELAI-side
    // decision doc updating both D-19 and D-28 contracts, this fails to
    // compile.
    fn _exhaustive(v: GatingVerdict) -> &'static str {
        match v {
            GatingVerdict::Pass => "pass",
            GatingVerdict::Fail => "fail",
        }
    }
}

#[test]
fn z_threshold_invariant_pinned_at_one_point_nine_six() {
    // Paper arXiv:2504.06141 §3.2 — the dual-filter uses Z=1.96 (95th-pct
    // two-sided normal CDF) as the high-uncertainty cutoff for ensemble
    // std. Drift in either direction (a "tighter 2.58" or "looser 1.64"
    // override at the constant) breaks the paper-grounded invariant.
    let eps = 1e-9_f64;
    assert!(
        (DEFAULT_Z_THRESHOLD - 1.96_f64).abs() < eps,
        "DEFAULT_Z_THRESHOLD must be 1.96 (got {DEFAULT_Z_THRESHOLD})"
    );
}

#[test]
fn max_samples_cap_paper_saturation_guard() {
    // Paper §5: >~1000 examples/round hurts performance (diversity
    // saturation). DEFAULT_MAX_SAMPLES is the paper sweet spot (1000);
    // MAX_SAMPLES_CAP is the hard ceiling (1500 — gives callers up-to-
    // 1.5x headroom but no further).
    assert!(
        DEFAULT_MAX_SAMPLES <= MAX_SAMPLES_CAP,
        "DEFAULT_MAX_SAMPLES {DEFAULT_MAX_SAMPLES} must be <= MAX_SAMPLES_CAP {MAX_SAMPLES_CAP}"
    );
    // Hard-pin the cap at 1500 so drift trips this contract.
    assert!(
        MAX_SAMPLES_CAP <= 1500,
        "MAX_SAMPLES_CAP {MAX_SAMPLES_CAP} exceeds paper saturation guard (1500)"
    );
    assert_eq!(DEFAULT_MAX_SAMPLES, 1000);
}

#[test]
fn max_rounds_cap_paper_round_three_plateau() {
    // Paper §5: round 3+ yields near-zero new examples; the runner
    // enforces a hard ceiling at 2.
    assert!(
        MAX_ROUNDS_CAP <= 2,
        "MAX_ROUNDS_CAP {MAX_ROUNDS_CAP} exceeds paper round-3 plateau (2)"
    );
    assert!(
        DEFAULT_MAX_ROUNDS <= MAX_ROUNDS_CAP,
        "DEFAULT_MAX_ROUNDS {DEFAULT_MAX_ROUNDS} must be <= MAX_ROUNDS_CAP {MAX_ROUNDS_CAP}"
    );
    assert_eq!(DEFAULT_MAX_ROUNDS, 2);
    assert_eq!(MAX_ROUNDS_CAP, 2);
}

#[test]
fn paper_floor_pp_default_at_least_one_point_zero() {
    // D-28-decisions.md DECISION-3: plan-doc UNGROUNDED "+5pp" REJECTED.
    // Recalibrated floor pins to 1.0pp (paper +0.7pp rounded UP to
    // integer-pp). The contract test catches drift toward both ends:
    // a "tighter 5pp" overclaim re-introduces the rejected number; a
    // "looser 0.5pp" or 0.0pp default would silently ship the report
    // without honouring the paper's measured minimum effect.
    assert!(
        DEFAULT_PAPER_FLOOR_PP >= 1.0,
        "DEFAULT_PAPER_FLOOR_PP {DEFAULT_PAPER_FLOOR_PP} must be >= 1.0 (paper-grounded floor; +0.7pp rounded up)"
    );
    // Counter-clamp: must NOT carry the UNGROUNDED "+5pp" value.
    assert!(
        DEFAULT_PAPER_FLOOR_PP < 5.0,
        "DEFAULT_PAPER_FLOOR_PP {DEFAULT_PAPER_FLOOR_PP} reintroduces the rejected plan-doc 5pp value"
    );
    // Surface predicate on the report mirrors the same floor.
    let r = AdvRmFarmRunReport {
        samples_farmed: 100,
        rounds: 2,
        prmbench_delta_pp: 1.5,
        verdict: GatingVerdict::Pass,
        paper_floor_pp: DEFAULT_PAPER_FLOOR_PP,
        corpus_blake3: "a".repeat(64),
        corpus_path: "tests/c.jsonl".into(),
        provenance_counts: vec![("canary_corpus".into(), 100)],
    };
    assert!(r.passed());
}
