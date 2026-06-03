//! F-8 SLICE A2 contract — the LLM-free symbolic conflict detector.
//!
//! Asserts the interface-level guarantees of
//! `elai_orchestrator::symbolic_conflict`:
//!   1. each of the three [`ConflictKind`] variants fires on a crafted
//!      intent-vs-model pair,
//!   2. a clean (non-locked-key) config variant yields ZERO conflicts,
//!   3. the conflict fingerprint is deterministic and exactly 16 lowercase
//!      hex chars,
//!   4. `tripwire_quiescent_violated` is `!active.is_empty()` over the five
//!      [`TripwireKind`] variants,
//!   5. every [`RESOURCE_RAIL_KEYS`] entry is genuinely a [`LOCKED_KEYS`]
//!      member (the rail list cannot silently desync from the lock list).
//!
//! Policy under test is WARN-then-escalate: the detector RETURNS conflicts
//! as a signal; it never panics, blocks, or fails closed.
//!
//! NO cargo test execution by the main LLM (s-60824 mandate). The auditor
//! runs the suite under cargo test in the AUDITING substate.

use elai_orchestrator::self_mod_layer1a::PipelineMode;
use elai_orchestrator::self_mod_safety::LOCKED_KEYS;
use elai_orchestrator::self_mod_types::{ConfigVariant, ProposedChange, VariantStatus};
use elai_orchestrator::symbolic_conflict::{
    conflict_fingerprint, current_process_model, detect_conflicts, intent_from_config_variant,
    intent_from_proposed_change, tripwire_quiescent_violated, ConflictKind, IntentCondition,
    IntentOp, ProcessModel, SymbolicConflict, SymbolicIntent, CONFLICT_FINGERPRINT_HEX_LEN,
    RESOURCE_RAIL_KEYS,
};
use elai_orchestrator::tripwire::TripwireKind;

fn json_blob(pairs: &[(&str, serde_json::Value)]) -> Vec<u8> {
    let map: serde_json::Map<String, serde_json::Value> = pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), v.clone()))
        .collect();
    serde_json::to_vec(&serde_json::Value::Object(map)).expect("serialize test blob")
}

fn variant(blob: Vec<u8>) -> ConfigVariant {
    ConfigVariant {
        variant_id: "v-contract".to_string(),
        parent_id: None,
        score: 0.0,
        children_count: 0,
        config_blob: blob,
        created_at: 0,
        pipeline_outcomes: Vec::new(),
        pipeline_mode: PipelineMode::Surgical,
        status: VariantStatus::Active,
    }
}

#[test]
fn clean_variant_yields_zero_conflicts() {
    // A config variant touching only NON-locked keys, no active tripwire.
    let cv = variant(json_blob(&[
        ("freshness_half_life", serde_json::json!(7.0)),
        ("etc_threshold", serde_json::json!(0.5)),
    ]));
    let intent = intent_from_config_variant(&cv);
    let model = current_process_model(&[]);
    let conflicts = detect_conflicts(&intent, &model);
    assert!(
        conflicts.is_empty(),
        "clean variant must produce no conflicts, got: {conflicts:?}"
    );
}

#[test]
fn contradictory_fires_on_locked_key_mutation() {
    // Raising a LOCKED_KEYS-governed parameter (score_weights) is the
    // canonical Contradictory conflict — even with no tripwire active.
    let cv = variant(json_blob(&[("score_weights", serde_json::json!("tampered"))]));
    let intent = intent_from_config_variant(&cv);
    let model = current_process_model(&[]);
    let conflicts = detect_conflicts(&intent, &model);
    assert!(
        conflicts
            .iter()
            .any(|c| c.kind == ConflictKind::Contradictory && c.key == "score_weights"),
        "expected Contradictory on score_weights, got: {conflicts:?}"
    );
}

#[test]
fn causal_violation_fires_on_locked_mutation_during_active_tripwire() {
    // A locked-key mutation WHILE a tripwire is active violates quiescence
    // -> both Contradictory AND CausalViolation are raised.
    let cv = variant(json_blob(&[("auditor_config", serde_json::json!("swap"))]));
    let intent = intent_from_config_variant(&cv);
    let model = current_process_model(&[TripwireKind::KlDivergence]);
    let conflicts = detect_conflicts(&intent, &model);
    assert!(
        conflicts
            .iter()
            .any(|c| c.kind == ConflictKind::CausalViolation && c.key == "auditor_config"),
        "expected CausalViolation under active tripwire, got: {conflicts:?}"
    );
    assert!(
        conflicts
            .iter()
            .any(|c| c.kind == ConflictKind::Contradictory),
        "the same mutation must also be Contradictory, got: {conflicts:?}"
    );
}

#[test]
fn resource_contention_fires_on_require_of_locked_rail() {
    // A variant naming a locked spend rail yields a Require condition the
    // detector flags as ResourceContention.
    let cv = variant(json_blob(&[(
        "cloud_monthly_budget",
        serde_json::json!(1_000_000),
    )]));
    let intent = intent_from_config_variant(&cv);
    assert!(
        intent
            .requires
            .iter()
            .any(|c| c.op == IntentOp::Require && c.key == "cloud_monthly_budget"),
        "config variant naming a rail must emit a Require condition"
    );
    let model = current_process_model(&[]);
    let conflicts = detect_conflicts(&intent, &model);
    assert!(
        conflicts
            .iter()
            .any(|c| c.kind == ConflictKind::ResourceContention && c.key == "cloud_monthly_budget"),
        "expected ResourceContention on cloud_monthly_budget, got: {conflicts:?}"
    );
}

#[test]
fn proposed_change_update_key_on_locked_is_contradictory() {
    // The structured ProposedChange path screens equally.
    let change = ProposedChange::UpdateKey {
        key: "lock_list".to_string(),
        old_value: "a".to_string(),
        new_value: "b".to_string(),
    };
    let intent = intent_from_proposed_change(&change);
    let model = current_process_model(&[]);
    let conflicts = detect_conflicts(&intent, &model);
    assert!(
        conflicts
            .iter()
            .any(|c| c.kind == ConflictKind::Contradictory && c.key == "lock_list"),
        "UpdateKey on a locked key must be Contradictory, got: {conflicts:?}"
    );
}

#[test]
fn fingerprint_is_deterministic_and_16_lower_hex() {
    let a = conflict_fingerprint(ConflictKind::Contradictory, "score_weights", "detail-x");
    let b = conflict_fingerprint(ConflictKind::Contradictory, "score_weights", "detail-x");
    assert_eq!(a, b, "fingerprint must be deterministic");
    assert_eq!(
        a.len(),
        CONFLICT_FINGERPRINT_HEX_LEN,
        "fingerprint length must equal the declared constant"
    );
    assert_eq!(a.len(), 16, "fingerprint must be 16 hex chars");
    assert!(
        a.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "fingerprint must be lowercase hex, got: {a}"
    );
    // Distinct inputs -> distinct fingerprints (collision-resistance proxy).
    let c = conflict_fingerprint(ConflictKind::CausalViolation, "score_weights", "detail-x");
    assert_ne!(a, c, "different kind must change the fingerprint");
}

#[test]
fn fingerprint_on_constructed_conflict_matches_free_function() {
    let conflict = SymbolicConflict::new(ConflictKind::ResourceContention, "cloud_opus_allowed", "d");
    let expected = conflict_fingerprint(ConflictKind::ResourceContention, "cloud_opus_allowed", "d");
    assert_eq!(conflict.fingerprint, expected);
}

#[test]
fn tripwire_quiescence_is_any_nonempty_over_five_variants() {
    assert!(
        !tripwire_quiescent_violated(&[]),
        "empty active set -> quiescent (not violated)"
    );
    // Each of the five variants on its own must violate quiescence.
    for k in [
        TripwireKind::KlDivergence,
        TripwireKind::CorrelationDrop,
        TripwireKind::IqrOutlier,
        TripwireKind::Perplexity,
        TripwireKind::Ddm,
    ] {
        assert!(
            tripwire_quiescent_violated(&[k]),
            "active {k:?} must violate quiescence"
        );
    }
}

#[test]
fn resource_rails_are_all_locked_keys() {
    // The rail subset cannot silently desync from the canonical lock list.
    for rail in RESOURCE_RAIL_KEYS {
        assert!(
            LOCKED_KEYS.contains(rail),
            "resource rail {rail} must be a LOCKED_KEYS entry"
        );
    }
}

#[test]
fn empty_intent_against_empty_model_is_clean() {
    // Pure-function boundary: an empty intent + default model never panics
    // and never conflicts.
    let intent = SymbolicIntent::default();
    let model = ProcessModel::default();
    assert!(intent.is_empty());
    assert!(detect_conflicts(&intent, &model).is_empty());
}

#[test]
fn non_mutating_require_on_non_rail_locked_key_does_not_contend() {
    // A Require on a locked key that is NOT a resource rail is not a
    // ResourceContention (the rule is rail-scoped).
    let intent = SymbolicIntent {
        asserts: Vec::new(),
        requires: vec![IntentCondition::new("safety_locks", IntentOp::Require, "x")],
    };
    let model = current_process_model(&[]);
    let conflicts = detect_conflicts(&intent, &model);
    assert!(
        conflicts.is_empty(),
        "Require on a non-rail locked key must not contend, got: {conflicts:?}"
    );
}
