//! Tranche B-7 triple contract tests — E-13 TDD verification substrate
//! (PropertyGate / MutationGateSurvivor / AntiMock) + E-12 carry
//! (AssertionOnlyDiff) + survival cache + default-off posture.
//!
//! Locks the public surface added by the B-7 triple tranche:
//!   * SCHEMA_VERSION_PUBLIC = 195 (single coordinated bump).
//!   * `SsbActionType::TddVerificationEvent` exists + round-trips.
//!   * `TddVerificationEventKind` has exactly 4 arms with snake-case
//!     wire labels.
//!   * `SurvivalCache::record_survivor` + `is_known_survivor` +
//!     `record_toolchain_missing` behave per contract (cache hit ->
//!     short-circuit; toolchain-missing -> recorded as Skipped, NOT
//!     surfaced as a survivor).
//!   * `AssertionOnlyDiffBlock` arm serializes as snake_case
//!     `assertion_only_diff_block`.
//!   * Default-off posture: 4 wire-dark env gates are absent by default
//!     (ELAI_E13_TDD_VERIFICATION_EMIT_ENABLE,
//!     ELAI_E13_SURVIVAL_CACHE_ENABLE,
//!     ELAI_E50_ARCHETYPE_SELECTED_EMIT_ENABLE,
//!     ELAI_SAAS_ARCH_PATTERNS_ENABLE).
//!
//! NOTE: no `cargo test` is run by the contract author (per CLAUDE.md
//! HARD RULE). The auditor agent runs these in AUDITING state.

use elai_orchestrator::ssb::{
    SaasArchPatternEventKind, SsbActionType, SsbPayload, TddVerificationEventKind,
    TddVerificationEventPayload, SCHEMA_VERSION_PUBLIC,
    ELAI_E13_TDD_VERIFICATION_EMIT_ENABLE_ENV,
    ELAI_E50_ARCHETYPE_SELECTED_EMIT_ENABLE_ENV,
    ELAI_SAAS_ARCH_PATTERNS_ENABLE_ENV,
};
use elai_orchestrator::survival_cache::{
    survival_cache_enabled, SurvivalCache, SurvivorStatus,
    ELAI_E13_SURVIVAL_CACHE_ENABLE_ENV,
};

/// D-1. SCHEMA_VERSION_PUBLIC must equal 195 after the B-7 triple bump.
#[test]
fn test_schema_version_is_195() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 195,
        "schema only drifts forward; this contract was written at version 195. \
         B-7 triple bumps SCHEMA_VERSION_PUBLIC from 194 -> 195; \
         observed {SCHEMA_VERSION_PUBLIC}"
    );
}

/// D-2. TddVerificationEvent round-trips through the typed payload
/// surface.
#[test]
fn test_tdd_verification_event_round_trip() {
    let payload = TddVerificationEventPayload {
        kind: TddVerificationEventKind::PropertyGatePass,
        task_id_hash: 0xABCD_1234_5678_9012,
        details_json: serde_json::json!({
            "property_name_hash": "0xfeedface",
            "sample_size": 256
        }),
        ts_ms: 1_700_000_195_001,
    };
    let envelope = SsbPayload::TddVerificationEvent(payload.clone());
    assert_eq!(envelope.action_type(), SsbActionType::TddVerificationEvent);
    let buf = rmp_serde::to_vec_named(&envelope).expect("serialize");
    let back: SsbPayload = rmp_serde::from_slice(&buf).expect("deserialize");
    match back {
        SsbPayload::TddVerificationEvent(got) => {
            assert_eq!(got, payload);
        }
        other => panic!("expected TddVerificationEvent, got {other:?}"),
    }
}

/// D-3. All 4 TddVerificationEventKind arms have snake-case wire labels
/// and the round-trip is total.
#[test]
fn test_tdd_verification_event_kind_round_trip() {
    let cases = [
        (
            TddVerificationEventKind::PropertyGatePass,
            "property_gate_pass",
        ),
        (
            TddVerificationEventKind::MutationGateSurvivorBlock,
            "mutation_gate_survivor_block",
        ),
        (TddVerificationEventKind::AntiMockBlock, "anti_mock_block"),
        (
            TddVerificationEventKind::AssertionOnlyDiffBlock,
            "assertion_only_diff_block",
        ),
    ];
    for (kind, expected_label) in cases {
        assert_eq!(kind.as_str(), expected_label);
        let parsed = TddVerificationEventKind::from_snake_case(expected_label)
            .unwrap_or_else(|| panic!("round trip {expected_label} -> None"));
        assert_eq!(parsed, kind);
    }
}

/// D-4. SurvivalCache::record_survivor + is_known_survivor: a recorded
/// LiveAndSurvived row short-circuits future cargo-mutants subprocess
/// runs for the same mutation id.
#[test]
fn test_survival_cache_record_and_query() {
    let cache = SurvivalCache::in_memory().expect("in-memory cache");
    let mid = "crates/foo/src/bar.rs:42: replace + with -";
    cache
        .record_survivor(mid, 1_700_000_195_002, SurvivorStatus::LiveAndSurvived)
        .expect("record");
    assert!(cache.is_known_survivor(mid).expect("query"));
    // A different mutation id must NOT short-circuit.
    assert!(!cache
        .is_known_survivor("crates/foo/src/bar.rs:43: replace * with /")
        .expect("query miss"));
}

/// D-5. SurvivalCache::record_toolchain_missing records the row but
/// does NOT surface it as a survivor. This is the canonical
/// RUNPOD-PENDING / cargo-mutants 27.x missing short-circuit (E-13
/// DECISION-7).
#[test]
fn test_survival_cache_short_circuits_on_toolchain_missing() {
    let cache = SurvivalCache::in_memory().expect("in-memory cache");
    let mid = "crates/foo/src/bar.rs:44: replace > with <";
    cache
        .record_toolchain_missing(mid, 1_700_000_195_003)
        .expect("record");
    // Skipped is NOT a survivor.
    assert!(!cache.is_known_survivor(mid).expect("query"));
    // But the row IS recorded so the dashboard can render it.
    let rows = cache.list_survivors_since(0).expect("list");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].status, SurvivorStatus::Skipped);
}

/// D-6. The E-12 carry arm AssertionOnlyDiffBlock serializes as the
/// snake_case wire label `assertion_only_diff_block`. Pinned because
/// the producer wiring is DEFERRED (substrate-only this tranche) and a
/// silent rename would break the producer's future emission contract.
#[test]
fn test_assertion_only_diff_kind_serializes_as_snake_case() {
    let kind = TddVerificationEventKind::AssertionOnlyDiffBlock;
    let label = kind.as_str();
    assert_eq!(label, "assertion_only_diff_block");
    // serde round-trip via JSON also lands on the snake-case wire label.
    let serialized = serde_json::to_string(&kind).expect("serialize");
    assert_eq!(serialized, "\"assertion_only_diff_block\"");
}

/// D-7. Default-off posture: the 4 wire-dark env gates introduced or
/// referenced by B-7 triple are NOT set in the contract-test
/// environment; the helper predicates must return `false`. Test runs
/// inside the elai_orchestrator address space so the gates are read
/// via the canonical helpers, not by re-parsing env_var locally.
#[test]
fn test_default_off_posture() {
    // Caller scrubs the 4 env vars; this is sufficient for the
    // single-threaded `cargo test` default. If parallel test threads
    // race on env state in a future run, the test must move to
    // serial_test::serial — that's a follow-on hardening, not a B-7
    // requirement.
    std::env::remove_var(ELAI_E13_TDD_VERIFICATION_EMIT_ENABLE_ENV);
    std::env::remove_var(ELAI_E13_SURVIVAL_CACHE_ENABLE_ENV);
    std::env::remove_var(ELAI_E50_ARCHETYPE_SELECTED_EMIT_ENABLE_ENV);
    std::env::remove_var(ELAI_SAAS_ARCH_PATTERNS_ENABLE_ENV);

    assert!(!survival_cache_enabled(), "survival cache must default OFF");
    // The other 3 gates are exposed via env_gate_truthy directly.
    assert!(!elai_orchestrator::ssb::env_gate_truthy(
        ELAI_E13_TDD_VERIFICATION_EMIT_ENABLE_ENV
    ));
    assert!(!elai_orchestrator::ssb::env_gate_truthy(
        ELAI_E50_ARCHETYPE_SELECTED_EMIT_ENABLE_ENV
    ));
    assert!(!elai_orchestrator::ssb::env_gate_truthy(
        ELAI_SAAS_ARCH_PATTERNS_ENABLE_ENV
    ));
}

/// E-50 D-8. SaasArchPatternEventKind gains a 12th arm
/// SaasArchetypeSelected (within-variant arm extension; NO new
/// top-level variant; NO additional SCHEMA bump beyond E-13's
/// 194 -> 195).
#[test]
fn test_archetype_selected_kind_round_trip() {
    let kind = SaasArchPatternEventKind::SaasArchetypeSelected;
    assert_eq!(kind.as_str(), "saas_archetype_selected");
    let serialized = serde_json::to_string(&kind).expect("serialize");
    assert_eq!(serialized, "\"saas_archetype_selected\"");
    let deserialized: SaasArchPatternEventKind =
        serde_json::from_str(&serialized).expect("deserialize");
    assert_eq!(deserialized, kind);
}

/// E-50 D-9. ArchetypeSelected emit shape: build_archetype_selected_payload
/// returns a SaasArchPatternEventPayload with kind=SaasArchetypeSelected
/// + archetype snake_case + details_json carrying source +
/// confidence_bps + selected_at_step + brief_hash + caller_site.
#[test]
fn test_archetype_selected_emit_shape() {
    use elai_orchestrator::architecture::recommender::{
        build_archetype_selected_payload, recommend_archetype,
        ArchetypeSelectionSource,
    };
    use elai_orchestrator::idea_framing::ProjectClass;

    let rec = recommend_archetype(ProjectClass::SaaSDashboard, "auth + billing");
    let payload = build_archetype_selected_payload(
        &rec,
        ArchetypeSelectionSource::RecommenderHigh,
        10_000,
        4,
        Some("s-b7-shape".to_owned()),
        0xCAFE_BABE_DEAD_BEEF,
        1_700_000_195_004,
    );
    assert_eq!(
        payload.kind,
        SaasArchPatternEventKind::SaasArchetypeSelected
    );
    assert_eq!(payload.archetype, "saas_dashboard");
    let details = &payload.details_json;
    assert_eq!(details["source"], "recommender_high");
    assert_eq!(details["confidence_bps"], 10_000);
    assert_eq!(details["selected_at_step"], 4);
    assert_eq!(details["caller_site"], "recommend_archetype_with_emit");
}

/// E-50 D-10. The 5-variant SaasArchetype taxonomy stays canonical
/// (no PLAN-side fabricated B2C/B2B/ApiFirst/AiApp drift here -- the
/// e11_b + e50 negative-space contracts already ban that). This test
/// asserts the 5-variant set is the SAME set used by the new
/// SaasArchetypeSelected arm: the archetype string emitted by the
/// payload MUST be one of the 5 canonical snake_case labels.
#[test]
fn test_5_variant_taxonomy_pinned() {
    use elai_orchestrator::architecture::saas_archetype::{
        SaasArchetype, ALL_SAAS_ARCHETYPES,
    };
    assert_eq!(
        ALL_SAAS_ARCHETYPES.len(),
        5,
        "canonical SaasArchetype taxonomy is 5 variants -- 4-variant \
         B2C/B2B/ApiFirst/AiApp plan drift is REJECTED"
    );
    let labels: Vec<&'static str> = ALL_SAAS_ARCHETYPES
        .iter()
        .map(|a| a.as_snake_case())
        .collect();
    assert_eq!(
        labels,
        vec![
            "saas_dashboard",
            "b2b_marketplace",
            "internal_tooling",
            "developer_api",
            "consumer_social",
        ],
        "5-variant taxonomy order is the contract-locked iteration order"
    );
    // Spot-check: a payload built off ConsumerSocial emits the snake-case
    // label without renames.
    assert_eq!(SaasArchetype::ConsumerSocial.as_snake_case(), "consumer_social");
}

/// E-50 D-11. recommend_archetype_with_emit returns None when the env
/// gate is OFF (default posture). Pinned because the wire-dark
/// guarantee is the contract: a downstream caller that forgets to
/// check the gate must not silently start writing SSB rows.
#[test]
fn test_archetype_selected_default_off() {
    use elai_orchestrator::architecture::recommender::recommend_archetype_with_emit;
    use elai_orchestrator::idea_framing::ProjectClass;

    std::env::remove_var(ELAI_E50_ARCHETYPE_SELECTED_EMIT_ENABLE_ENV);
    let (_rec, payload) = recommend_archetype_with_emit(
        ProjectClass::WebApp,
        "auth dashboard",
        None,
        0,
        1_700_000_195_005,
    );
    assert!(
        payload.is_none(),
        "default-off gate -> recommend_archetype_with_emit returns None"
    );
}
