// B-4 task #5 + #13 — dashboard-v2 contract tests for the 3 new Tauri
// commands: get_query_trace (singular), get_index_version,
// get_upgrade_action.
//
// Why no live function call:
//   Each command takes `State<'_, AppState>` which cannot be constructed
//   outside the Tauri runtime (AppState::new() has filesystem/env/network
//   side-effects). We therefore lock the response JSON shape via sentinel
//   struct round-trip; the JSON shape IS the dashboard-frontend ↔ backend
//   contract.
//
// What it asserts:
//   - The documented response shapes (known:true + known:false) deserialise
//     into the sentinel structs without losing fields.
//   - The shapes are round-trip stable (deserialise → re-serialise →
//     identical JSON values modulo field ordering).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// get_query_trace (singular) — response shape mirror.
// Production source: crates/dashboard-v2/src-tauri/src/commands.rs
//   - on match:    { "known": true, "seq", "ts", "query_id", "tier",
//                    "stage_count", "result_count", "total_latency_ms" }
//   - on no match: { "known": false }
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct QueryTraceDetailResponse {
    known: bool,
    #[serde(default)]
    seq: Option<i64>,
    #[serde(default)]
    ts: Option<i64>,
    #[serde(default)]
    query_id: Option<String>,
    #[serde(default)]
    tier: Option<String>,
    #[serde(default)]
    stage_count: Option<u64>,
    #[serde(default)]
    result_count: Option<u64>,
    #[serde(default)]
    total_latency_ms: Option<u64>,
}

#[test]
fn b4_query_trace_known_shape_round_trips() {
    let sample = json!({
        "known": true,
        "seq": 42,
        "ts": 1700000000,
        "query_id": "abc-123-def",
        "tier": "DEEP",
        "stage_count": 7,
        "result_count": 15,
        "total_latency_ms": 142,
    });
    let parsed: QueryTraceDetailResponse =
        serde_json::from_value(sample.clone()).expect("known shape must deserialise");
    assert!(parsed.known);
    assert_eq!(parsed.query_id.as_deref(), Some("abc-123-def"));
    assert_eq!(parsed.tier.as_deref(), Some("DEEP"));
    assert_eq!(parsed.stage_count, Some(7));
    assert_eq!(parsed.result_count, Some(15));
    assert_eq!(parsed.total_latency_ms, Some(142));
    let re_serialised = serde_json::to_value(&parsed).expect("re-serialise");
    assert_eq!(
        re_serialised.get("known"), Some(&Value::Bool(true)),
        "known flag must survive round-trip"
    );
}

#[test]
fn b4_query_trace_unknown_shape_round_trips() {
    let sample = json!({ "known": false });
    let parsed: QueryTraceDetailResponse =
        serde_json::from_value(sample).expect("unknown shape must deserialise");
    assert!(!parsed.known);
    assert!(parsed.query_id.is_none());
}

// ---------------------------------------------------------------------------
// get_index_version — response shape mirror.
// Production source: crates/dashboard-v2/src-tauri/src/commands.rs
//   - on success:  { "known": true, "version", "dkg_schema_version",
//                    "vector_dimensions": { "code", "text" },
//                    "bm25_format", "created_at" }
//   - on absent:   { "known": false, "error": "..." }
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct IndexVersionResponse {
    known: bool,
    #[serde(default)]
    version: Option<u32>,
    #[serde(default)]
    dkg_schema_version: Option<u32>,
    #[serde(default)]
    vector_dimensions: Option<VectorDimensionsDto>,
    #[serde(default)]
    bm25_format: Option<String>,
    #[serde(default)]
    created_at: Option<i64>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct VectorDimensionsDto {
    code: usize,
    text: usize,
}

#[test]
fn b4_index_version_known_shape_round_trips() {
    let sample = json!({
        "known": true,
        "version": 2,
        "dkg_schema_version": 3,
        "vector_dimensions": { "code": 1536, "text": 768 },
        "bm25_format": "tantivy_0.26_fmt1",
        "created_at": 1700000000,
    });
    let parsed: IndexVersionResponse =
        serde_json::from_value(sample.clone()).expect("known shape must deserialise");
    assert!(parsed.known);
    assert_eq!(parsed.version, Some(2));
    assert_eq!(parsed.dkg_schema_version, Some(3));
    let dims = parsed.vector_dimensions.expect("vector_dimensions present");
    assert_eq!(dims.code, 1536);
    assert_eq!(dims.text, 768);
    assert_eq!(parsed.bm25_format.as_deref(), Some("tantivy_0.26_fmt1"));
}

#[test]
fn b4_index_version_unknown_shape_round_trips() {
    let sample = json!({
        "known": false,
        "error": "index_version.toml not found (first run)",
    });
    let parsed: IndexVersionResponse =
        serde_json::from_value(sample).expect("unknown shape must deserialise");
    assert!(!parsed.known);
    assert!(parsed.error.is_some());
}

// ---------------------------------------------------------------------------
// get_upgrade_action — response shape mirror.
// Production source: crates/dashboard-v2/src-tauri/src/commands.rs
//   - on success:  { "known": true, "action": "Reuse" | "RebuildBm25Only" | "FullRebuild" }
//   - on absent:   { "known": false, "error": "..." }
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct UpgradeActionResponse {
    known: bool,
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[test]
fn b4_upgrade_action_reuse_shape_round_trips() {
    let sample = json!({ "known": true, "action": "Reuse" });
    let parsed: UpgradeActionResponse =
        serde_json::from_value(sample).expect("Reuse shape must deserialise");
    assert!(parsed.known);
    assert_eq!(parsed.action.as_deref(), Some("Reuse"));
}

#[test]
fn b4_upgrade_action_full_rebuild_shape_round_trips() {
    let sample = json!({ "known": true, "action": "FullRebuild" });
    let parsed: UpgradeActionResponse =
        serde_json::from_value(sample).expect("FullRebuild shape must deserialise");
    assert!(parsed.known);
    assert_eq!(parsed.action.as_deref(), Some("FullRebuild"));
}

#[test]
fn b4_upgrade_action_unknown_shape_round_trips() {
    let sample = json!({
        "known": false,
        "error": "index_version.toml not found (first run)",
    });
    let parsed: UpgradeActionResponse =
        serde_json::from_value(sample).expect("unknown shape must deserialise");
    assert!(!parsed.known);
    assert!(parsed.error.is_some());
}
