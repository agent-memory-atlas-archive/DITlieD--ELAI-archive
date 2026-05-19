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
        // C-24 (DRIFT-4): default None on full-build fixture path.
        is_incremental: None,
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::SymbolGraphUpdated,
        "B-33 follow-up: SsbPayload::SymbolGraphUpdated must map back to \
         SsbActionType::SymbolGraphUpdated via .action_type()",
    );
}

#[test]
fn test_schema_version_at_or_above_b33_baseline() {
    // B-33 follow-up bumped SCHEMA_VERSION 39 -> 40 to gate the additive
    // SymbolGraphUpdated enum + CHECK constraint change. Later phases
    // (B-37, B-40, B-41, B-42 + C-24) added more variants and bumped
    // further. The contract is "no regression below 40" — anything 40+
    // includes the SymbolGraphUpdated row.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 40,
        "B-33 floor: SCHEMA_VERSION must be >= 40 (got {})",
        SCHEMA_VERSION_PUBLIC,
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
        // C-24 (DRIFT-4): default None on full-build fixture path.
        is_incremental: None,
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
    assert_eq!(
        restored.is_incremental, original.is_incremental,
        "C-24 (DRIFT-4): SymbolGraphUpdatedPayload.is_incremental must round-trip \
         (None default preserved via #[serde(default)])"
    );
}

#[test]
fn test_symbol_graph_updated_payload_round_trips_incremental_flag() {
    // C-24 (DRIFT-4): verifies the additive Option<bool> field
    // serializes/deserializes correctly when set to Some(true) — the
    // incremental-path branch in indexing_service.rs emits this shape on
    // every resume.
    let original = SymbolGraphUpdatedPayload {
        project_id: "elai-cc".into(),
        file_count: 7,
        symbol_count: 0,
        edge_count: 0,
        built_ms: 1_730_000_000_456,
        is_incremental: Some(true),
    };

    let json = serde_json::to_string(&original)
        .expect("C-24: SymbolGraphUpdatedPayload (incremental) must serialize to JSON");
    let restored: SymbolGraphUpdatedPayload = serde_json::from_str(&json)
        .expect("C-24: SymbolGraphUpdatedPayload (incremental) must deserialize from JSON");
    assert_eq!(
        restored.is_incremental,
        Some(true),
        "C-24 (DRIFT-4): is_incremental=Some(true) must round-trip through JSON"
    );

    // Defense-in-depth: also exercise the rmp-serde named-map path used by
    // the live SSB pipeline. The additive Option follows the C-17 DECISION-8
    // precedent (batch_duration_ms) — rmp-serde tolerates extra/missing
    // optional fields under named-map encoding.
    let bytes = rmp_serde::to_vec_named(&original)
        .expect("C-24: SymbolGraphUpdatedPayload (incremental) must rmp-serialize");
    let restored_mp: SymbolGraphUpdatedPayload = rmp_serde::from_slice(&bytes)
        .expect("C-24: SymbolGraphUpdatedPayload (incremental) must rmp-deserialize");
    assert_eq!(restored_mp.is_incremental, Some(true));
    assert_eq!(restored_mp.project_id, "elai-cc");
}
