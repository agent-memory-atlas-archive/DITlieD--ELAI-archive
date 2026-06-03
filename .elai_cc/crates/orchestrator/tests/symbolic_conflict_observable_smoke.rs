//! F-8 / Slice A4 — symbolic-conflict observable smoke (the W-5 real-path
//! harness).
//!
//! Runs the REAL SCF detector
//! ([`elai_orchestrator::detect_conflicts`] over
//! [`elai_orchestrator::intent_from_config_variant`] +
//! [`elai_orchestrator::current_process_model`]) across the four shipped
//! `tests/fixtures/*_variant.json` proposal fixtures — the SAME fixtures the
//! `elai self-mod apply --proposal-fixture` CLI entrypoint loads. Each test
//! decodes a fixture exactly as the CLI handler does (the operator-friendly
//! `config` object is re-encoded to the `config_blob` byte form the detector
//! decodes, and `active_tripwires` are the snake-case `TripwireKind` labels) and
//! asserts the expected `ConflictKind` set (or none for the clean variant).
//!
//! POLICY observed (not asserted as a hard fail): WARN-then-escalate. A conflict
//! is a SIGNAL — these tests assert the detector FLAGS the right kind, they do
//! NOT assert any fail-closed BLOCK (there is none). SCF's published 27.9%
//! precision is a simulation-only NON-target and is deliberately never asserted.
//!
//! This is a pure-function harness (no SSB, no runtime) — it exercises the exact
//! detector the CLI drives, so a regression in the fixtures OR the detector
//! breaks here loudly. The SSB-emitting compose path is covered by
//! `crates/contracts/tests/symbolic_conflict_apply_proposal_contract.rs`.

use std::path::PathBuf;

use elai_orchestrator::self_mod_layer1a::PipelineMode;
use elai_orchestrator::self_mod_types::{ConfigVariant, VariantStatus};
use elai_orchestrator::tripwire::TripwireKind;
use elai_orchestrator::{
    current_process_model, detect_conflicts, intent_from_config_variant, ConflictKind,
};
use serde::Deserialize;

/// Mirror of the CLI `ProposalFixture` shape (crates/cli/src/self_mod_cmd.rs).
/// Kept local to the test so the harness validates the on-disk fixture schema
/// the CLI consumes without depending on the binary crate's private type.
#[derive(Debug, Deserialize)]
struct ProposalFixture {
    variant_id: String,
    #[serde(default)]
    config: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    active_tripwires: Vec<TripwireKind>,
    #[serde(default)]
    #[allow(dead_code)]
    fsm_substate: Option<String>,
}

/// Load a fixture from `tests/fixtures/<name>` relative to the orchestrator
/// crate manifest dir (the canonical `CARGO_MANIFEST_DIR` fixture pattern used
/// by the rest of the orchestrator test suite).
fn load_fixture(name: &str) -> ProposalFixture {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse fixture {}: {e}", path.display()))
}

/// Decode a fixture into a real `ConfigVariant` exactly as the CLI handler does:
/// the operator-friendly `config` object is re-encoded to the `config_blob`
/// byte form the detector decodes.
fn config_variant(fixture: &ProposalFixture) -> ConfigVariant {
    let config_blob =
        serde_json::to_vec(&serde_json::Value::Object(fixture.config.clone()))
            .expect("encode config_blob");
    ConfigVariant {
        variant_id: fixture.variant_id.clone(),
        parent_id: None,
        score: 0.0,
        children_count: 0,
        config_blob,
        created_at: 0,
        pipeline_outcomes: Vec::new(),
        pipeline_mode: PipelineMode::Surgical,
        status: VariantStatus::Active,
    }
}

/// Run the real detector over a fixture and return the detected conflict kinds.
fn detect_kinds(name: &str) -> Vec<ConflictKind> {
    let fixture = load_fixture(name);
    let cv = config_variant(&fixture);
    let intent = intent_from_config_variant(&cv);
    let model = current_process_model(&fixture.active_tripwires);
    detect_conflicts(&intent, &model)
        .into_iter()
        .map(|c| c.kind)
        .collect()
}

// ---------------------------------------------------------------------------
// contradictory_locked_variant.json — locked-key mutation, no tripwire
// ---------------------------------------------------------------------------

#[test]
fn contradictory_locked_variant_flags_contradictory() {
    let kinds = detect_kinds("contradictory_locked_variant.json");
    assert!(
        kinds.contains(&ConflictKind::Contradictory),
        "a locked-key mutation must flag Contradictory; got {kinds:?}"
    );
    // No tripwire active ⇒ no CausalViolation.
    assert!(
        !kinds.contains(&ConflictKind::CausalViolation),
        "no active tripwire ⇒ no CausalViolation; got {kinds:?}"
    );
}

// ---------------------------------------------------------------------------
// resource_contention_variant.json — Require on a locked resource rail
// ---------------------------------------------------------------------------

#[test]
fn resource_contention_variant_flags_resource_contention() {
    let kinds = detect_kinds("resource_contention_variant.json");
    assert!(
        kinds.contains(&ConflictKind::ResourceContention),
        "a Require on a locked resource rail must flag ResourceContention; got {kinds:?}"
    );
    // A locked-key Set on the same rail is ALSO contradictory (the rail is a
    // locked key); the resource-rail variant therefore raises both.
    assert!(
        kinds.contains(&ConflictKind::Contradictory),
        "a locked resource rail is also a locked key ⇒ Contradictory; got {kinds:?}"
    );
}

// ---------------------------------------------------------------------------
// causal_violation_tripwire_variant.json — locked mutation WHILE a tripwire
// is active ⇒ Contradictory + CausalViolation
// ---------------------------------------------------------------------------

#[test]
fn causal_violation_variant_flags_causal_and_contradictory() {
    let kinds = detect_kinds("causal_violation_tripwire_variant.json");
    assert!(
        kinds.contains(&ConflictKind::CausalViolation),
        "a locked mutation during an active tripwire must flag CausalViolation; got {kinds:?}"
    );
    assert!(
        kinds.contains(&ConflictKind::Contradictory),
        "the same locked mutation is also Contradictory; got {kinds:?}"
    );
}

// ---------------------------------------------------------------------------
// clean_variant.json — non-locked key, no tripwire ⇒ NO conflict
// ---------------------------------------------------------------------------

#[test]
fn clean_variant_flags_nothing() {
    let kinds = detect_kinds("clean_variant.json");
    assert!(
        kinds.is_empty(),
        "a non-locked key with no active tripwire must be clean (admissible); got {kinds:?}"
    );
}

// ---------------------------------------------------------------------------
// Cross-fixture: the causal fixture's active tripwire is what flips the model
// (same key, drift toggled) — proves the tripwire half is wired, not constant
// ---------------------------------------------------------------------------

#[test]
fn tripwire_presence_is_what_adds_causal_violation() {
    // The causal fixture carries an active tripwire; clearing it (re-screening
    // the SAME config against a quiescent model) must drop the CausalViolation
    // while keeping the Contradictory flag — proving the causal rule is driven
    // by the live tripwire set, not hard-coded.
    let fixture = load_fixture("causal_violation_tripwire_variant.json");
    assert!(
        !fixture.active_tripwires.is_empty(),
        "the causal fixture must carry at least one active tripwire"
    );
    let cv = config_variant(&fixture);
    let intent = intent_from_config_variant(&cv);

    let with_tripwire = current_process_model(&fixture.active_tripwires);
    let quiescent = current_process_model(&[]);

    let kinds_with: Vec<ConflictKind> = detect_conflicts(&intent, &with_tripwire)
        .into_iter()
        .map(|c| c.kind)
        .collect();
    let kinds_quiescent: Vec<ConflictKind> = detect_conflicts(&intent, &quiescent)
        .into_iter()
        .map(|c| c.kind)
        .collect();

    assert!(kinds_with.contains(&ConflictKind::CausalViolation));
    assert!(!kinds_quiescent.contains(&ConflictKind::CausalViolation));
    // Contradictory is invariant to quiescence (the locked-key mutation stands).
    assert!(kinds_with.contains(&ConflictKind::Contradictory));
    assert!(kinds_quiescent.contains(&ConflictKind::Contradictory));
}
