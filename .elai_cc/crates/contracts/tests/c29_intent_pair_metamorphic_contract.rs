//! C-29 contract: Testing-Trust Tier T-2 (IntentPair) + Tier T-3
//! (basic Metamorphic relations).
//!
//! Pins:
//!   * SCHEMA_VERSION_PUBLIC >= 61 (atomic 9-tuple lockstep bump per
//!     workflow/reasoning/C-29-decisions.md DECISION-D1 / DRIFT-C29-1).
//!   * Two new SsbActionType variants — `IntentConfirmed` + `MrViolated` —
//!     round-trip via `ssb_action_type_from_str_public` /
//!     `SsbActionType::as_sql_str`.
//!   * `IntentConfirmedPayload` shape: { step_id, pair_count, confirmed_by }.
//!   * `MrViolatedPayload` shape: { step_id, relation, counterexample,
//!     sample_size }. `counterexample` MUST be non-empty per stub_ban
//!     (DRIFT-C29-11).
//!   * `IntentPair` constructor enforces char bounds 120/120/80; multi-byte
//!     UTF-8 characters count per `char`, not per byte (CSQ).
//!   * `RelationLibrary::default()` excludes `PermutationInvariance`
//!     (DRIFT-C29-6).
//!   * `IntentConfig::default()` returns `min_pairs = 3`.
//!   * `IntentPairsGate` is registered for PLANNING -> TESTING_FIRST in
//!     `all_gates()` (DRIFT-C29-3).
//!   * `IntentPairsGate::check` rejects when `intent_pairs.len() <
//!     min_intent_pairs` and passes when threshold met.
//!   * `evaluate_autonomous_intent_confirm` honors the threshold floor
//!     (auto-confirm does NOT bypass `min_pairs`, DRIFT-C29-3).

use elai_orchestrator::evidence::{
    all_gates, EvidenceGate, IntentPairsGate, TransitionContext,
};
use elai_orchestrator::fsm::FsmState;
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, IntentConfirmedPayload, MrViolatedPayload, SsbActionType,
    SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{
    evaluate_autonomous_intent_confirm, parse_relation, ConfirmedBy, IntentConfig, IntentError,
    IntentPair, IntentPairSet, MetamorphicConfig, MetamorphicRelation, MetamorphicTestRunner,
    MrResult, RelationLibrary, ASSERTION_MAX_CHARS, INTENT_MAX_CHARS, REF_MAX_CHARS,
};

// ---------------------------------------------------------------------------
// SSB schema + variant pins
// ---------------------------------------------------------------------------

#[test]
fn c29_schema_version_public_floor_is_61() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 61,
        "C-29 atomic 9-tuple lockstep bump: floor is 61 (see \
         workflow/reasoning/C-29-decisions.md DECISION-D1 / DRIFT-C29-1). \
         Current: {SCHEMA_VERSION_PUBLIC}",
    );
}

#[test]
fn c29_intent_confirmed_action_type_round_trips_through_str() {
    let variant = SsbActionType::IntentConfirmed;
    assert_eq!(variant.as_sql_str(), "intent_confirmed");
    let parsed = ssb_action_type_from_str_public("intent_confirmed")
        .unwrap_or_else(|e| panic!("from_str must accept 'intent_confirmed': {e}"));
    assert_eq!(parsed, variant);
}

#[test]
fn c29_mr_violated_action_type_round_trips_through_str() {
    let variant = SsbActionType::MrViolated;
    assert_eq!(variant.as_sql_str(), "mr_violated");
    let parsed = ssb_action_type_from_str_public("mr_violated")
        .unwrap_or_else(|e| panic!("from_str must accept 'mr_violated': {e}"));
    assert_eq!(parsed, variant);
}

#[test]
fn c29_intent_confirmed_payload_serde_roundtrip() {
    let p = IntentConfirmedPayload {
        step_id: 7,
        pair_count: 3,
        confirmed_by: "user".to_string(),
    };
    let bytes = serde_json::to_vec(&p).expect("serialize");
    let back: IntentConfirmedPayload = serde_json::from_slice(&bytes).expect("deserialize");
    assert_eq!(back, p);
}

#[test]
fn c29_mr_violated_payload_serde_roundtrip() {
    let p = MrViolatedPayload {
        step_id: 11,
        relation: "inverse".to_string(),
        counterexample: "sample_index=0 input_hex=01 encoded_hex=ff decoded_hex=01".to_string(),
        sample_size: 100,
    };
    let bytes = serde_json::to_vec(&p).expect("serialize");
    let back: MrViolatedPayload = serde_json::from_slice(&bytes).expect("deserialize");
    assert_eq!(back, p);
    assert!(
        !back.counterexample.is_empty(),
        "MrViolated counterexample MUST be non-empty (stub_ban / DRIFT-C29-11)"
    );
}

// ---------------------------------------------------------------------------
// IntentPair shape + char bounds
// ---------------------------------------------------------------------------

#[test]
fn c29_intent_pair_bounded_chars_enforced() {
    assert_eq!(INTENT_MAX_CHARS, 120);
    assert_eq!(ASSERTION_MAX_CHARS, 120);
    assert_eq!(REF_MAX_CHARS, 80);
    let long_intent = "x".repeat(INTENT_MAX_CHARS + 1);
    let err = IntentPair::new(long_intent, "ok assertion", "bs-1").unwrap_err();
    assert!(matches!(err, IntentError::IntentTooLong { .. }));
}

#[test]
fn c29_intent_pair_rejects_empty_strings() {
    let err = IntentPair::new("", "non-empty", "bs-1").unwrap_err();
    assert!(matches!(err, IntentError::IntentEmpty));
    let err = IntentPair::new("non-empty", "", "bs-1").unwrap_err();
    assert!(matches!(err, IntentError::AssertionEmpty));
}

#[test]
fn c29_intent_pair_char_bound_counts_chars_not_bytes() {
    // CSQ from context-package-C29 — 4-byte emoji must count per `char`,
    // not per byte; 119 emoji must pass the 120-char ceiling even though
    // their byte-count is 476.
    let intent: String = "\u{1F600}".repeat(119);
    let pair = IntentPair::new(intent.clone(), "ok", "bs-1")
        .expect("119 emoji must pass char-budget");
    assert!(pair.natural_language_intent.len() > INTENT_MAX_CHARS);
    assert_eq!(pair.natural_language_intent.chars().count(), 119);
}

#[test]
fn c29_intent_pair_set_default_is_empty_unconfirmed() {
    let set = IntentPairSet::default();
    assert!(set.is_empty());
    assert!(!set.is_confirmed());
    assert!(set.confirmed_at_ms.is_none());
}

#[test]
fn c29_intent_config_default_min_pairs_is_3() {
    let cfg = IntentConfig::default();
    assert_eq!(cfg.min_pairs, 3);
}

#[test]
fn c29_confirmed_by_snake_case_strings() {
    assert_eq!(ConfirmedBy::User.as_str(), "user");
    assert_eq!(ConfirmedBy::AutonomousGate.as_str(), "autonomous_gate");
}

// ---------------------------------------------------------------------------
// Metamorphic library shape
// ---------------------------------------------------------------------------

#[test]
fn c29_relation_library_default_excludes_permutation() {
    let lib = RelationLibrary::default();
    assert!(lib.contains(MetamorphicRelation::Identity));
    assert!(lib.contains(MetamorphicRelation::Inverse));
    assert!(
        !lib.contains(MetamorphicRelation::PermutationInvariance),
        "PermutationInvariance must be opt-in (DRIFT-C29-6)"
    );
}

#[test]
fn c29_relation_library_from_config_skips_unknown_names() {
    let cfg = MetamorphicConfig {
        enabled: true,
        relations: vec![
            "identity".to_owned(),
            "totally_fictional_relation".to_owned(),
        ],
        sample_size: 5,
    };
    let lib = RelationLibrary::from_config(&cfg);
    assert!(lib.contains(MetamorphicRelation::Identity));
    assert!(!lib.contains(MetamorphicRelation::Inverse));
}

#[test]
fn c29_parse_relation_round_trip() {
    for rel in [
        MetamorphicRelation::Identity,
        MetamorphicRelation::Inverse,
        MetamorphicRelation::PermutationInvariance,
    ] {
        assert_eq!(parse_relation(rel.as_str()), Some(rel));
    }
    assert!(parse_relation("not_a_real_relation").is_none());
}

#[test]
fn c29_metamorphic_config_default_relations_excludes_permutation() {
    let cfg = MetamorphicConfig::default();
    assert!(cfg.enabled);
    assert_eq!(cfg.relations, vec!["identity".to_owned(), "inverse".to_owned()]);
    assert_eq!(cfg.sample_size, 100);
}

#[test]
fn c29_run_inverse_emits_non_empty_counterexample_on_failure() {
    let runner = MetamorphicTestRunner::new(RelationLibrary::default(), 1);
    // Adversarial encoder drops the last byte; decode cannot recover.
    let bad_encode = |x: &[u8]| -> Vec<u8> {
        if x.is_empty() {
            Vec::new()
        } else {
            x[..x.len() - 1].to_vec()
        }
    };
    let identity_decode = |x: &[u8]| -> Vec<u8> { x.to_vec() };
    let samples: Vec<Vec<u8>> = vec![vec![1, 2, 3]];
    let r: MrResult = runner.run_inverse(bad_encode, identity_decode, &samples);
    assert!(!r.passed);
    let ce = r
        .counterexample
        .expect("MrResult::failed must populate counterexample");
    assert!(
        !ce.is_empty(),
        "counterexample MUST be non-empty (DRIFT-C29-11)"
    );
    assert!(ce.contains("sample_index=0"));
}

// ---------------------------------------------------------------------------
// IntentPairsGate registration + behaviour
// ---------------------------------------------------------------------------

#[test]
fn c29_intent_pairs_gate_registered_for_planning_to_testing_first() {
    let gates = all_gates();
    let found = gates.iter().any(|g| {
        g.name() == "intent_pairs"
            && g.transition() == (FsmState::Planning, FsmState::TestingFirst)
    });
    assert!(
        found,
        "IntentPairsGate must be registered for PLANNING -> TESTING_FIRST \
         (DRIFT-C29-3)"
    );
}

#[test]
fn c29_intent_pairs_gate_rejects_below_threshold() {
    let mut set = IntentPairSet::new();
    set.push(IntentPair::new("a", "b", "bs-1").unwrap());
    set.push(IntentPair::new("c", "d", "bs-1").unwrap());
    let ctx = TransitionContext {
        intent_pairs: Some(set),
        min_intent_pairs: 3,
        ..Default::default()
    };
    let err = IntentPairsGate.check(&ctx).unwrap_err();
    assert!(
        err.missing_artifacts
            .iter()
            .any(|s| s == "intent_pairs_below_threshold"),
        "missing_artifacts must contain intent_pairs_below_threshold: {:?}",
        err.missing_artifacts
    );
}

#[test]
fn c29_intent_pairs_gate_passes_at_threshold() {
    let mut set = IntentPairSet::new();
    for _ in 0..3 {
        set.push(IntentPair::new("a", "b", "bs-1").unwrap());
    }
    let ctx = TransitionContext {
        intent_pairs: Some(set),
        min_intent_pairs: 3,
        ..Default::default()
    };
    IntentPairsGate.check(&ctx).expect("gate must pass at threshold");
}

#[test]
fn c29_intent_pairs_gate_default_degrades_when_threshold_zero() {
    // min_pairs = 0 disables the gate per IntentConfig semantics.
    let ctx = TransitionContext {
        intent_pairs: None,
        min_intent_pairs: 0,
        ..Default::default()
    };
    IntentPairsGate.check(&ctx).expect("gate must pass when disabled");
}

// ---------------------------------------------------------------------------
// Autonomous-gate auto-confirm semantics
// ---------------------------------------------------------------------------

#[test]
fn c29_autonomous_confirm_honors_threshold_floor() {
    let mut below = IntentPairSet::new();
    below.push(IntentPair::new("a", "b", "bs-1").unwrap());
    // Below threshold -> auto-confirm refuses.
    assert!(
        evaluate_autonomous_intent_confirm(&below, 3).is_none(),
        "auto-confirm MUST NOT bypass pair-count threshold (DRIFT-C29-3)"
    );
    let mut at = IntentPairSet::new();
    for _ in 0..3 {
        at.push(IntentPair::new("a", "b", "bs-1").unwrap());
    }
    assert_eq!(evaluate_autonomous_intent_confirm(&at, 3), Some(3));
}
