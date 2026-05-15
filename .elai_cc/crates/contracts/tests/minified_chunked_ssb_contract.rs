//! B-32 — Minified-file chunking SSB round-trip + bijection contract tests.
//!
//! Covers the new SSB surface introduced by B-32 Step B:
//!   1. SsbActionType::MinifiedFileChunked round-trips through
//!      `as_sql_str` + `ssb_action_type_from_str_public` (string bijection).
//!   2. SCHEMA_VERSION bumped 37 -> 38 to gate the additive enum + CHECK
//!      constraint change so cross-process consumers refuse mismatched DBs.
//!   3. MinifiedFileChunkedPayload survives serde round-trip (JSON used as
//!      a stable, human-debuggable witness; msgpack is exercised
//!      end-to-end by the orchestrator's existing write-time
//!      `validate_payload_matches_action_type` path).
//!
//! These tests fail-to-compile until B-32 Step B EXECUTING lands the
//! `MinifiedFileChunked` enum variant, the `MinifiedFileChunkedPayload`
//! struct, the `as_sql_str` / `ssb_action_type_from_str` arms, the
//! `SsbPayload::MinifiedFileChunked` discriminant, the
//! `payload.action_type()` mapper arm, and the SCHEMA_VERSION bump.

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, MinifiedFileChunkedPayload, SsbActionType, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
};

#[test]
fn test_minified_file_chunked_action_round_trips() {
    let original = SsbActionType::MinifiedFileChunked;
    let wire = original.as_sql_str();
    assert_eq!(
        wire, "minified_file_chunked",
        "B-32: SsbActionType::MinifiedFileChunked must serialize to the canonical wire string \
         used by the SQL CHECK constraint and the dashboard payload_json pipeline",
    );

    let parsed = ssb_action_type_from_str_public(wire)
        .expect("B-32: minified_file_chunked must parse back to the enum variant");
    assert_eq!(
        parsed, original,
        "B-32: as_sql_str and ssb_action_type_from_str_public must form a bijection \
         on the MinifiedFileChunked variant",
    );

    // Defense-in-depth: the SsbPayload mapper must agree with the enum.
    let payload = SsbPayload::MinifiedFileChunked(MinifiedFileChunkedPayload {
        relative_path: "vendor/bundle.min.js".into(),
        char_count: 50_000,
        chunk_count: 25,
        avg_line_length: 50_000,
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::MinifiedFileChunked,
        "B-32: SsbPayload::MinifiedFileChunked must map back to \
         SsbActionType::MinifiedFileChunked via .action_type()",
    );
}

#[test]
fn test_schema_version_bumped_to_38() {
    // Updated to 40 by B-33 follow-up (SymbolGraphUpdated, 39 -> 40).
    // Previous bumps that touched this test: 37 -> 38 (B-32, this variant)
    // and 38 -> 39 (B-31.5, 5 deferred B-31 SSB beacons).
    // The MinifiedFileChunked invariants (round-trip + payload schema)
    // remain pinned by the other two tests in this file; this test only
    // anchors the floor — SCHEMA_VERSION must never regress below 38.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 38,
        "B-32: SCHEMA_VERSION must be at least 38 (the bump that gated the \
         MinifiedFileChunked enum + CHECK constraint addition). Got {}.",
        SCHEMA_VERSION_PUBLIC,
    );
    // Defense-in-depth: pin the current expected value so any future bump
    // is intentional, not accidental. Update both this constant and the
    // ledger comment in ssb.rs when bumping.
    assert_eq!(
        SCHEMA_VERSION_PUBLIC, 40,
        "B-33 follow-up: SCHEMA_VERSION expected 40 after the SymbolGraphUpdated \
         variant landed. If this fires after another batch bump (41+), update both \
         constants AND verify the relevant ledger comments in ssb.rs are still present.",
    );
}

#[test]
fn test_minified_file_chunked_payload_round_trips() {
    let original = MinifiedFileChunkedPayload {
        relative_path: "static/bootstrap.min.css".into(),
        char_count: 150_000,
        chunk_count: 75,
        avg_line_length: 150_000,
    };

    // serde_json round-trip — uses serde::{Serialize, Deserialize} which the
    // SSB MessagePack pipeline also relies on. JSON is the lowest-common-
    // denominator witness for the payload schema (no msgpack-specific
    // gotchas like length-prefix mismatches).
    let json = serde_json::to_string(&original)
        .expect("B-32: MinifiedFileChunkedPayload must serialize to JSON");
    let restored: MinifiedFileChunkedPayload = serde_json::from_str(&json)
        .expect("B-32: MinifiedFileChunkedPayload must deserialize from JSON");

    assert_eq!(
        restored, original,
        "B-32: MinifiedFileChunkedPayload must survive serde round-trip — \
         field rename or type change here is a backward-incompatible schema break",
    );
}
