//! E-45 Tranche 8A -- JSON Schema snapshot lock for the 5 sidecar-boundary
//! types in `elai_providers::types`.
//!
//! Asserts byte-equality (after canonicalization) between the schemars-
//! generated schema for each of the 5 boundary types and a checked-in
//! fixture under `tests/schemas/<TypeName>.json`. The fixtures drive the
//! Pydantic mirror regeneration via
//! `.claude/tools/regen_sidecar_schemas.py` -- a drift here means either
//! (a) the Rust struct shape moved without a Pydantic mirror update or
//! (b) the schemars proc-macro output changed across a version bump,
//! both of which require deliberate review.
//!
//! Two tests per type:
//!   * `<name>_schema_matches_snapshot` -- runtime schema canonicalized
//!     and compared to the on-disk fixture (`#[test]`, runs in CI).
//!   * `regen_<name>_schema` -- writes the canonicalized runtime schema
//!     to disk (`#[test] #[ignore]`, manual invocation after review).
//!
//! Canonicalization: serde_json::Value with sorted keys at every level.
//! Schemars 1.2 emits `$defs` in declaration order; sorting at the
//! Value level produces deterministic byte output independent of derive
//! macro field-order quirks.
//!
//! Regen one type:
//!   cargo test -p elai-orchestrator --test schemas_snapshot \
//!       regen_message_request_schema -- --ignored --nocapture
//!
//! Regen all 5 (the helper in
//! `.claude/tools/regen_sidecar_schemas.py` shells the cargo command 5x
//! and then runs `datamodel-codegen` on the resulting directory).

use std::fs;
use std::path::PathBuf;

use elai_providers::{
    InputMessage, MessageRequest, MessageResponse, SystemBlock, ToolDefinition,
};
use schemars::schema_for;
use serde_json::Value;

fn schemas_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("schemas")
}

fn schema_path(type_name: &str) -> PathBuf {
    schemas_dir().join(format!("{type_name}.json"))
}

/// Canonicalize a serde_json::Value: sort all object keys recursively.
/// Schemars 1.2 emits `$defs` in derive-order; sorting here makes the
/// fixture byte-stable across derive-macro reorderings.
fn canonicalize(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted: Vec<(String, Value)> = map.into_iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            let mut out = serde_json::Map::new();
            for (k, v) in sorted {
                out.insert(k, canonicalize(v));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(canonicalize).collect()),
        other => other,
    }
}

fn runtime_schema_canonical<T: schemars::JsonSchema>() -> String {
    let schema = schema_for!(T);
    let value: Value = serde_json::to_value(&schema)
        .unwrap_or_else(|e| panic!("schema serialize to Value failed: {e}"));
    let canonical = canonicalize(value);
    let mut out = serde_json::to_string_pretty(&canonical)
        .unwrap_or_else(|e| panic!("canonical schema pretty-print failed: {e}"));
    out.push('\n');
    out
}

fn assert_schema_matches(type_name: &str, runtime: &str) {
    let path = schema_path(type_name);
    let on_disk = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "missing snapshot fixture {}: {}. Regen via:\n  cargo test -p elai-orchestrator --test schemas_snapshot regen_{}_schema -- --ignored --nocapture",
            path.display(),
            e,
            type_name.to_ascii_lowercase()
        )
    });
    // Re-canonicalize the on-disk file so historical whitespace / key order
    // differences do not produce false negatives. The runtime string is
    // already canonical.
    let on_disk_value: Value = serde_json::from_str(&on_disk)
        .unwrap_or_else(|e| panic!("on-disk fixture {} not valid JSON: {}", path.display(), e));
    let on_disk_canonical = canonicalize(on_disk_value);
    let mut on_disk_pretty = serde_json::to_string_pretty(&on_disk_canonical)
        .unwrap_or_else(|e| panic!("on-disk re-canonicalize failed: {e}"));
    on_disk_pretty.push('\n');
    assert_eq!(
        runtime, on_disk_pretty,
        "schemars-generated schema for {type_name} drifted from on-disk fixture {}.\n\
         Regen via:\n  cargo test -p elai-orchestrator --test schemas_snapshot regen_{}_schema -- --ignored --nocapture\n\
         Then review the diff and run `.claude/tools/regen_sidecar_schemas.py` to update the Pydantic mirror.",
        path.display(),
        type_name.to_ascii_lowercase()
    );
}

fn write_snapshot(type_name: &str, runtime: &str) {
    let dir = schemas_dir();
    if let Err(e) = fs::create_dir_all(&dir) {
        panic!("create schemas dir {}: {}", dir.display(), e);
    }
    let path = schema_path(type_name);
    if let Err(e) = fs::write(&path, runtime) {
        panic!("write {}: {}", path.display(), e);
    }
    eprintln!("regen: wrote {}", path.display());
}

// --- snapshot tests (run in CI) ----------------------------------------

#[test]
fn message_request_schema_matches_snapshot() {
    let runtime = runtime_schema_canonical::<MessageRequest>();
    assert_schema_matches("MessageRequest", &runtime);
}

#[test]
fn system_block_schema_matches_snapshot() {
    let runtime = runtime_schema_canonical::<SystemBlock>();
    assert_schema_matches("SystemBlock", &runtime);
}

#[test]
fn input_message_schema_matches_snapshot() {
    let runtime = runtime_schema_canonical::<InputMessage>();
    assert_schema_matches("InputMessage", &runtime);
}

#[test]
fn tool_definition_schema_matches_snapshot() {
    let runtime = runtime_schema_canonical::<ToolDefinition>();
    assert_schema_matches("ToolDefinition", &runtime);
}

#[test]
fn message_response_schema_matches_snapshot() {
    let runtime = runtime_schema_canonical::<MessageResponse>();
    assert_schema_matches("MessageResponse", &runtime);
}

// --- regen helpers (manual, --ignored) ---------------------------------

#[test]
#[ignore]
fn regen_message_request_schema() {
    let runtime = runtime_schema_canonical::<MessageRequest>();
    write_snapshot("MessageRequest", &runtime);
}

#[test]
#[ignore]
fn regen_system_block_schema() {
    let runtime = runtime_schema_canonical::<SystemBlock>();
    write_snapshot("SystemBlock", &runtime);
}

#[test]
#[ignore]
fn regen_input_message_schema() {
    let runtime = runtime_schema_canonical::<InputMessage>();
    write_snapshot("InputMessage", &runtime);
}

#[test]
#[ignore]
fn regen_tool_definition_schema() {
    let runtime = runtime_schema_canonical::<ToolDefinition>();
    write_snapshot("ToolDefinition", &runtime);
}

#[test]
#[ignore]
fn regen_message_response_schema() {
    let runtime = runtime_schema_canonical::<MessageResponse>();
    write_snapshot("MessageResponse", &runtime);
}

#[test]
#[ignore]
fn regen_all_schemas() {
    write_snapshot(
        "MessageRequest",
        &runtime_schema_canonical::<MessageRequest>(),
    );
    write_snapshot("SystemBlock", &runtime_schema_canonical::<SystemBlock>());
    write_snapshot("InputMessage", &runtime_schema_canonical::<InputMessage>());
    write_snapshot(
        "ToolDefinition",
        &runtime_schema_canonical::<ToolDefinition>(),
    );
    write_snapshot(
        "MessageResponse",
        &runtime_schema_canonical::<MessageResponse>(),
    );
}
