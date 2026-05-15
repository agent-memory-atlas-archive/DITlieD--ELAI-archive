//! B-33 follow-up — SymbolGraphUpdated SSB round-trip + bijection contract.
//!
//! Mirrors the B-32 `minified_chunked_ssb_contract.rs` shape. Covers the new
//! SSB surface introduced when the SymbolGraphUpdated variant was un-deferred:
//!   1. SsbActionType::SymbolGraphUpdated round-trips through `as_sql_str` +
//!      `ssb_action_type_from_str_public` (string bijection).
//!   2. SCHEMA_VERSION bumped 39 -> 40 to gate the additive enum + CHECK
//!      constraint change so cross-process consumers refuse mismatched DBs.
//!   3. SymbolGraphUpdatedPayload survives serde round-trip — JSON is the
//!      lowest-common-denominator witness; msgpack is exercised end-to-end by
//!      the orchestrator's existing `validate_payload_matches_action_type`
//!      path at ssb.rs write time.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, SsbActionType, SsbPayload, SymbolGraphUpdatedPayload,
    SCHEMA_VERSION_PUBLIC,
};

#[test]
fn test_symbol_graph_updated_action_round_trips() {
    let original = SsbActionType::SymbolGraphUpdated;
    let wire = original.as_sql_str();
    assert_eq!(
        wire, "symbol_graph_updated",
        "B-33 follow-up: SsbActionType::SymbolGraphUpdated must serialize to the canonical wire \
         string used by the SQL CHECK constraint and the dashboard payload_json pipeline",
    );

    let parsed = ssb_action_type_from_str_public(wire)
        .expect("B-33 follow-up: symbol_graph_updated must parse back to the enum variant");
    assert_eq!(
        parsed, original,
        "B-33 follow-up: as_sql_str and ssb_action_type_from_str_public must form a bijection \
         on the SymbolGraphUpdated variant",
    );

    let payload = SsbPayload::SymbolGraphUpdated(SymbolGraphUpdatedPayload {
        project_id: "p-1".into(),
        file_count: 16,
        symbol_count: 1_024,
        edge_count: 4_096,
        built_ms: 1_730_000_000_000,
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::SymbolGraphUpdated,
        "B-33 follow-up: SsbPayload::SymbolGraphUpdated must map back to \
         SsbActionType::SymbolGraphUpdated via .action_type()",
    );
}

#[test]
fn test_schema_version_bumped_for_b33() {
    assert_eq!(
        SCHEMA_VERSION_PUBLIC, 40,
        "B-33 follow-up: SCHEMA_VERSION must be bumped from 39 to 40 to gate the additive \
         SymbolGraphUpdated enum + CHECK constraint change. If this fires after a later batch \
         bump (41+), update the constant here AND verify the B-33 ledger comment in ssb.rs is \
         still present.",
    );
}

#[test]
fn test_symbol_graph_updated_payload_round_trips() {
    let original = SymbolGraphUpdatedPayload {
        project_id: "elai-cc".into(),
        file_count: 256,
        symbol_count: 32_768,
        edge_count: 131_072,
        built_ms: 1_730_000_000_123,
    };

    // serde_json round-trip — uses serde::{Serialize, Deserialize} which the
    // SSB MessagePack pipeline also relies on. JSON is the lowest-common-
    // denominator witness for the payload schema (no msgpack-specific
    // gotchas like length-prefix mismatches).
    let json = serde_json::to_string(&original)
        .expect("B-33 follow-up: SymbolGraphUpdatedPayload must serialize to JSON");
    let restored: SymbolGraphUpdatedPayload = serde_json::from_str(&json)
        .expect("B-33 follow-up: SymbolGraphUpdatedPayload must deserialize from JSON");

    assert_eq!(
        restored.project_id, original.project_id,
        "B-33 follow-up: SymbolGraphUpdatedPayload.project_id must round-trip"
    );
    assert_eq!(
        restored.file_count, original.file_count,
        "B-33 follow-up: SymbolGraphUpdatedPayload.file_count must round-trip"
    );
    assert_eq!(
        restored.symbol_count, original.symbol_count,
        "B-33 follow-up: SymbolGraphUpdatedPayload.symbol_count must round-trip"
    );
    assert_eq!(
        restored.edge_count, original.edge_count,
        "B-33 follow-up: SymbolGraphUpdatedPayload.edge_count must round-trip"
    );
    assert_eq!(
        restored.built_ms, original.built_ms,
        "B-33 follow-up: SymbolGraphUpdatedPayload.built_ms must round-trip"
    );
}
