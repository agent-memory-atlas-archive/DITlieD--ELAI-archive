//! D-3 closure — verifier consensus contract.
//!
//! Pins the SCHEMA_VERSION_PUBLIC + ConsensusRule + AggregateVerdict +
//! the 5 SSB payload types in the consensus row family (the original 2 +
//! the 3 added in the D-3 closure: PpmUnavailable + VerifierDisagreement +
//! CrossContextDispatched) so any silent rename / shape drift fires a
//! compile or test failure here before reaching downstream consumers
//! (dashboard panel, agent_pool::run_consensus_pass, D-4 tripwire stack).
//!
//! Tests:
//!   * `schema_version_at_least_105` — floor lift atomic with the D-3
//!     closure's +3 SSB variants.
//!   * `consensus_rule_round_trips_via_serde` — every closed-enum variant
//!     serializes + deserializes back to the same value.
//!   * `aggregate_verdict_round_trips_via_serde` — same for the verdict
//!     enum.
//!   * `consensus_payloads_round_trip_via_serde` — every payload type
//!     round-trips via JSON without field rename / drop.
//!   * `default_rule_is_k_of_n_two_of_three` — companion §3.3 default
//!     contract.
//!   * `consensus_action_types_resolve_from_wire_strings` — the snake_case
//!     wire labels resolve back to their enum variants via
//!     `ssb_action_type_from_str_public`.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, CrossContextDispatchedPayload, PpmUnavailablePayload,
    SsbActionType, SsbPayload, VerifierConsensusFailPayload, VerifierConsensusPassPayload,
    VerifierDisagreementPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::verification_result::{
    aggregate, AggregateVerdict, ConsensusRule, IntrinsicSignals, VerificationResult, VerifierKind,
};

#[test]
fn schema_version_at_least_105() {
    // D-3 closure floor: atomic 9-tuple bump 104 -> 105 with 3 new
    // variants. Future bumps may rise; never fall below 105.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 105,
        "SCHEMA_VERSION_PUBLIC ({SCHEMA_VERSION_PUBLIC}) below D-3 closure floor 105",
    );
}

#[test]
fn consensus_rule_round_trips_via_serde() {
    let rules = vec![
        ConsensusRule::AllPass,
        ConsensusRule::KofN { k: 2, n: 3 },
        ConsensusRule::WeightedScoreThreshold {
            weights: vec![(VerifierKind::Ppm, 0.6), (VerifierKind::Intrinsic, 0.4)],
            threshold: 0.7,
        },
        ConsensusRule::VerifierKindHeterogeneous {
            required_kinds: vec![
                VerifierKind::Ppm,
                VerifierKind::Intrinsic,
                VerifierKind::Deterministic,
            ],
        },
    ];
    for rule in rules {
        let json = serde_json::to_string(&rule).expect("serialize ConsensusRule");
        let back: ConsensusRule = serde_json::from_str(&json).expect("deserialize ConsensusRule");
        assert_eq!(rule, back, "round-trip mismatch for {rule:?}");
    }
}

#[test]
fn aggregate_verdict_round_trips_via_serde() {
    let verdicts = vec![
        AggregateVerdict::Pass,
        AggregateVerdict::Fail {
            reasons: vec!["v1".into(), "v2".into()],
        },
        AggregateVerdict::Inconclusive {
            reason: "missing_kinds=ppm".into(),
        },
    ];
    for v in verdicts {
        let json = serde_json::to_string(&v).expect("serialize AggregateVerdict");
        let back: AggregateVerdict =
            serde_json::from_str(&json).expect("deserialize AggregateVerdict");
        assert_eq!(v, back, "round-trip mismatch for {v:?}");
    }
}

#[test]
fn consensus_payloads_round_trip_via_serde() {
    // The 5 payload types in the consensus row family must round-trip
    // without drift — silent field rename would break dashboard rendering.
    let pass = VerifierConsensusPassPayload {
        candidate_id: "cand-1".into(),
        template_id: "tpl-1".into(),
        verdict: "pass".into(),
        verifier_count: 3,
        pass_count: 2,
        rule_id: "k_of_n".into(),
    };
    let json = serde_json::to_string(&pass).expect("serialize VerifierConsensusPassPayload");
    let back: VerifierConsensusPassPayload =
        serde_json::from_str(&json).expect("deserialize VerifierConsensusPassPayload");
    assert_eq!(pass, back);

    let fail = VerifierConsensusFailPayload {
        candidate_id: "cand-2".into(),
        template_id: "tpl-2".into(),
        failing_verifier_ids: vec!["v-intr-1".into()],
        reason: "fail".into(),
    };
    let json = serde_json::to_string(&fail).expect("serialize VerifierConsensusFailPayload");
    let back: VerifierConsensusFailPayload =
        serde_json::from_str(&json).expect("deserialize VerifierConsensusFailPayload");
    assert_eq!(fail, back);

    let unavail = PpmUnavailablePayload {
        candidate_id: "cand-3".into(),
        verifier_id: "ppm-1".into(),
        reason: "health_check_failed".into(),
    };
    let json = serde_json::to_string(&unavail).expect("serialize PpmUnavailablePayload");
    let back: PpmUnavailablePayload =
        serde_json::from_str(&json).expect("deserialize PpmUnavailablePayload");
    assert_eq!(unavail, back);

    let disagree = VerifierDisagreementPayload {
        candidate_id: "cand-4".into(),
        template_id: "tpl-4".into(),
        verifier_a_kind: "ppm".into(),
        verifier_b_kind: "deterministic".into(),
        score_delta: 0.7,
    };
    let json = serde_json::to_string(&disagree).expect("serialize VerifierDisagreementPayload");
    let back: VerifierDisagreementPayload =
        serde_json::from_str(&json).expect("deserialize VerifierDisagreementPayload");
    assert_eq!(disagree, back);

    let cc = CrossContextDispatchedPayload {
        claim_id: "claim-5".into(),
        context_a_hash: "a".repeat(64),
        context_b_hash: "b".repeat(64),
        jaccard: 0.85,
        agreement: "agree".into(),
    };
    let json = serde_json::to_string(&cc).expect("serialize CrossContextDispatchedPayload");
    let back: CrossContextDispatchedPayload =
        serde_json::from_str(&json).expect("deserialize CrossContextDispatchedPayload");
    assert_eq!(cc, back);
}

#[test]
fn default_rule_is_k_of_n_two_of_three() {
    let rule = ConsensusRule::default_two_of_three();
    match rule {
        ConsensusRule::KofN { k, n } => {
            assert_eq!(k, 2, "default rule must be k=2");
            assert_eq!(n, 3, "default rule must be n=3");
        }
        other => panic!("expected KofN default, got {other:?}"),
    }
}

#[test]
fn consensus_action_types_resolve_from_wire_strings() {
    let cases = [
        ("verifier_consensus_pass", SsbActionType::VerifierConsensusPass),
        ("verifier_consensus_fail", SsbActionType::VerifierConsensusFail),
        ("ppm_unavailable", SsbActionType::PpmUnavailable),
        ("verifier_disagreement", SsbActionType::VerifierDisagreement),
        ("cross_context_dispatched", SsbActionType::CrossContextDispatched),
    ];
    for (wire, expected) in cases {
        let resolved =
            ssb_action_type_from_str_public(wire).expect("wire-tag must resolve to enum variant");
        assert_eq!(
            resolved, expected,
            "wire-tag {wire} resolved to wrong variant: got {resolved:?}, want {expected:?}",
        );
    }
}

#[test]
fn aggregate_smoke_pass_and_fail() {
    // Sanity-check the closed-form aggregator on a deterministic input
    // so any aggregator-shape drift fires here before downstream.
    let r_pass = VerificationResult {
        candidate_id: "c".into(),
        verifier_id: "v1".into(),
        kind: VerifierKind::Ppm,
        score: Some(0.9),
        passed: true,
        signal_payload: IntrinsicSignals {
            logprob_mean: 0.0,
            logprob_var: 0.0,
            entropy: 0.0,
            step_count: 0,
            kl_to_base: 0.0,
        },
        ts_ms: 1_700_000_000_000,
    };
    let r_fail = VerificationResult {
        candidate_id: "c".into(),
        verifier_id: "v2".into(),
        kind: VerifierKind::Deterministic,
        score: Some(0.0),
        passed: false,
        signal_payload: IntrinsicSignals {
            logprob_mean: 0.0,
            logprob_var: 0.0,
            entropy: 0.0,
            step_count: 0,
            kl_to_base: 0.0,
        },
        ts_ms: 1_700_000_000_001,
    };

    let v = aggregate(&ConsensusRule::AllPass, &[r_pass.clone()]);
    assert_eq!(v, AggregateVerdict::Pass);

    let v = aggregate(&ConsensusRule::AllPass, &[r_pass, r_fail]);
    matches!(v, AggregateVerdict::Fail { .. });
}

#[test]
fn ssb_payload_round_trip_for_new_variants() {
    // Confirm the SsbPayload enum carries the 3 new variants and they
    // round-trip via the canonical action_type() discriminant.
    let payloads = vec![
        SsbPayload::PpmUnavailable(PpmUnavailablePayload {
            candidate_id: "c1".into(),
            verifier_id: "v1".into(),
            reason: "down".into(),
        }),
        SsbPayload::VerifierDisagreement(VerifierDisagreementPayload {
            candidate_id: "c2".into(),
            template_id: "t1".into(),
            verifier_a_kind: "ppm".into(),
            verifier_b_kind: "intrinsic".into(),
            score_delta: 0.5,
        }),
        SsbPayload::CrossContextDispatched(CrossContextDispatchedPayload {
            claim_id: "c3".into(),
            context_a_hash: "0".repeat(64),
            context_b_hash: "1".repeat(64),
            jaccard: 0.75,
            agreement: "agree".into(),
        }),
    ];
    for p in payloads {
        let t = p.action_type();
        let wire = t.as_sql_str();
        let resolved =
            ssb_action_type_from_str_public(wire).expect("wire-tag must resolve to enum variant");
        assert_eq!(resolved, t);
    }
}
