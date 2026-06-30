//! J-53: Deterministic tool-input repair shim for the OpenAI-compat
//! unmasked-decoder seam.
//!
//! ELAI cannot grammar-mask cloud OpenAI-compat providers (DeepSeek-V4-direct,
//! Azure, GLM, xAI) or any local model behind an OpenAI-compatible server
//! (Ollama, llama.cpp, local vLLM in OpenAI mode). On that lane the ONLY lever
//! is repairing the bytes the model already emitted. Today
//! `parse_tool_arguments` stuffs malformed args into a silent `{"raw": ...}`
//! blob that flows into tool execution and fails downstream with no
//! localization. This module replaces that with a validate-then-repair entry
//! that applies four ordered deterministic shape repairs + a typed-path
//! markdown-link unwrap BEFORE any re-prompt (E-18 one-shot retry is the
//! FALLBACK when repair genuinely cannot fix it).
//!
//! Design invariants (plan J-53, stub_ban):
//! - Validate-then-repair, NEVER preprocess-then-validate (a greedy preprocess
//!   corrupts valid-but-unusual content, e.g. a writeFile body that is itself
//!   JSON-shaped).
//! - Valid inputs ship byte-identical (ShipUnchanged, zero repair row).
//! - The repair ORDER is load-bearing: json-array-parse BEFORE bare-string-wrap
//!   (else `'["a","b"]'` double-wraps into `['["a","b"]']`).
//! - NO new dependency: a minimal structural type-matcher against the declared
//!   `input_schema` "type". The full `jsonschema` crate is DEFERRED
//!   (user-confirmation-flagged).
//! - NO `.unwrap()`/`.expect()` on production paths.
//! - Default-ON behind `ELAI_TOOLCALL_REPAIR_DISABLE` (W-2 kill-switch, NOT a
//!   new ENABLE). When disabled, the legacy passthrough is used and the
//!   disable is logged (never a silent flip).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tracing::{debug, warn};

/// Default limit when the tool's `input_schema` does not declare a `default`
/// for the `limit` field. Matches `file_read.rs::DEFAULT_LINE_LIMIT` (200)
/// so the relational default is consistent with the tool dispatcher's own
/// fallback.
const DEFAULT_LIMIT: u32 = 200;

/// The kind of deterministic repair applied to a malformed tool argument.
///
/// Within a single array-declared field: json-array-parse BEFORE bare-string-wrap
/// (load-bearing). `NullOfOptionalDrop` is a cross-field cleanup; `TypedPathUnwrap` a leak repair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepairKind {
    /// A stringified JSON array (`"[\"a\",\"b\"]"`) was parsed into an actual
    /// array where the schema declares `type: "array"`.
    JsonArrayParse,
    /// A bare string (`"foo"`) was wrapped into `["foo"]` where the schema
    /// declares `type: "array"`.
    BareStringWrap,
    /// A placeholder object (`{"items": ["a","b"]}`) was unwrapped into its
    /// single array value where the schema declares `type: "array"`.
    SingleArgObjectUnwrap,
    /// A `null` value on an optional field was dropped (omit, never send
    /// `null`).
    NullOfOptionalDrop,
    /// A degenerate markdown auto-link (`[/foo](/foo)`) on a path-typed field
    /// was stripped to the bare path. Real links (`[click](https://x.com)`)
    /// pass through unchanged.
    TypedPathUnwrap,
}

/// The outcome of `repair_tool_input`.
#[derive(Debug, Clone, PartialEq)]
pub enum RepairOutcome {
    /// The input parsed and structurally matched the schema — shipped
    /// byte-identical, zero repair row.
    ShipUnchanged { value: Value },
    /// The input was malformed but deterministically repaired. `kinds` records
    /// every repair applied (for telemetry / the SSB row).
    Repaired { value: Value, kinds: Vec<RepairKind> },
    /// The input could not be repaired to match the schema. `model_readable_msg`
    /// is a templated, model-readable error that rides E-18's one-shot retry
    /// path as the fallback AFTER deterministic repair.
    Unrepairable { model_readable_msg: String },
}

/// The relational-invariant default applied to an offset/limit-class tool.
///
/// J-53 step 4: when a tool declares both `offset` and `limit` integer
/// properties and the model provides only one, the other is defaulted:
/// `limit` alone -> `offset=0`, `offset` alone -> `limit=<tool default>`.
/// The `note` is a model-readable string (NO `Error:` prefix) intended to
/// be surfaced in the ToolResult so the model self-corrects next turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalDefault {
    /// The field that was defaulted (e.g. `"offset"` or `"limit"`).
    pub field: String,
    /// The default value that was applied, as a string for the note.
    pub default_value: String,
    /// A model-readable note about the default (NO `Error:` prefix).
    /// Surfacing this in the ToolResult requires the tool dispatcher
    /// (`crates/tools/src/dispatcher.rs`); the note is produced here
    /// and logged at the production seam.
    pub note: String,
}

/// Apply relational-invariant defaults to offset/limit-class tools.
///
/// J-53 step 4: `limit` alone -> `offset=0`, `offset` alone ->
/// `limit=<tool default>`. A tool is "offset/limit-class" if its
/// `input_schema` declares both `offset` and `limit` properties of
/// integer type. The default for `limit` is read from the schema's
/// `default` field if present, else falls back to `DEFAULT_LIMIT` (200,
/// matching `file_read.rs::DEFAULT_LINE_LIMIT`).
///
/// This is a **post-repair** defaulting step, NOT a shape repair — it
/// runs after `repair_tool_input` has fixed any malformed shapes and
/// fills in missing relational counterparts. It is intentionally
/// separate from `repair_tool_input` (the plan places relational
/// defaults at the tool-dispatcher boundary; within the providers crate
/// the closest reachable seam is `parse_tool_arguments`, which calls
/// this function after shape repair).
///
/// Returns `Some(RelationalDefault)` if a default was applied, `None`
/// if both fields are present, both absent, or the tool is not
/// offset/limit-class.
pub fn apply_relational_defaults(
    tool_name: &str,
    value: &mut Value,
    input_schema: Option<&Value>,
) -> Option<RelationalDefault> {
    let schema = input_schema?;
    let properties = schema.get("properties").and_then(|p| p.as_object())?;

    // Check if this is an offset/limit-class tool.
    let offset_schema = properties.get("offset")?;
    let limit_schema = properties.get("limit")?;
    let offset_is_int = offset_schema.get("type").and_then(|t| t.as_str()) == Some("integer");
    let limit_is_int = limit_schema.get("type").and_then(|t| t.as_str()) == Some("integer");
    if !offset_is_int || !limit_is_int {
        return None;
    }

    let obj = value.as_object_mut()?;
    let has_offset = obj.get("offset").is_some_and(|v| !v.is_null());
    let has_limit = obj.get("limit").is_some_and(|v| !v.is_null());

    if has_offset && has_limit {
        return None; // both present — no default needed
    }
    if !has_offset && !has_limit {
        return None; // both absent — let the tool handle its own defaults
    }

    if !has_offset && has_limit {
        // `limit` alone -> `offset=0`
        obj.insert("offset".to_string(), Value::from(0u32));
        let default = RelationalDefault {
            field: "offset".to_string(),
            default_value: "0".to_string(),
            note: format!(
                "tool `{tool_name}`: offset defaulted to 0 (limit was specified without offset)"
            ),
        };
        debug!(tool = %tool_name, field = %default.field, "J-53: relational default applied");
        return Some(default);
    }

    // has_offset && !has_limit -> `limit=<tool default>`
    let default_limit = limit_schema
        .get("default")
        .and_then(|d| d.as_u64())
        .map(|v| v as u32)
        .unwrap_or(DEFAULT_LIMIT);
    obj.insert("limit".to_string(), Value::from(default_limit));
    let default = RelationalDefault {
        field: "limit".to_string(),
        default_value: default_limit.to_string(),
        note: format!(
            "tool `{tool_name}`: limit defaulted to {default_limit} (offset was specified without limit)"
        ),
    };
    debug!(tool = %tool_name, field = %default.field, "J-53: relational default applied");
    Some(default)
}

// ---------------------------------------------------------------------------
// J-53 step 5: ToolInputRepaired SSB action_type — providers-side event.
//
// The SSB action_type variant + 9-tuple lockstep lives in elai-orchestrator
// (crates/orchestrator/src/ssb/) — it cannot live here because elai-providers
// must not depend on elai-orchestrator (circular dep; see the auth_broker_emit
// pattern at crates/orchestrator/src/auth_broker_emit.rs). This module defines
// the providers-side event data the orchestrator emit wrapper maps to the
// `ToolInputRepaired` SSB payload, plus the Kind discriminator that becomes
// the SSB payload's `kind` field (Pattern-2 single-variant action_type).
//
// Kind discriminator (matches the plan's WIRING_CONTRACT):
//   ShapeRepaired         — a malformed arg was deterministically repaired
//                           (RepairOutcome::Repaired; carries the RepairKind
//                           detail in `repair_kinds`).
//   RelationalDefaulted   — a structurally-valid arg had a missing
//                           offset/limit counterpart filled by
//                           `apply_relational_defaults` (ShipUnchanged + a
//                           relational default; no shape repair).
//   UnrepairableSurfaced  — the arg could not be repaired and a model-readable
//                           `tool_input_invalid` value was surfaced
//                           (RepairOutcome::Unrepairable; rides E-18 retry).
// ---------------------------------------------------------------------------

/// The SSB `ToolInputRepaired` action_type Kind discriminator.
///
/// Pattern-2 single-variant: this is the `kind` field on the SSB payload. One
/// event is emitted per `parse_tool_arguments` call that is NOT a clean
/// `ShipUnchanged`-with-no-default. When a call is BOTH shape-repaired AND
/// relationally-defaulted, the primary kind is `ShapeRepaired` (the format
/// signal) and the relational detail is carried in `relational_default`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolInputRepairKind {
    /// A malformed tool arg was deterministically shape-repaired.
    ShapeRepaired,
    /// A structurally-valid arg had a missing offset/limit counterpart defaulted.
    RelationalDefaulted,
    /// An unrepairable arg surfaced a model-readable `tool_input_invalid` value.
    UnrepairableSurfaced,
}

/// The providers-side event data for the `ToolInputRepaired` SSB action_type.
///
/// Built at the production seam (`parse_tool_arguments_with_telemetry` in
/// `openai_compat.rs`) and emitted via `tracing` here; the orchestrator emit
/// wrapper (a separate, orchestrator-scoped task following the
/// `auth_broker_emit` pattern) maps this to the SSB journal row. Carries
/// provider + model + tool + kind so repair-RATE per (provider, model, tool)
/// — the sibling FORMAT signal to J-51's SEMANTIC adherence — can be derived
/// from the journal without merging the two metrics (plan COMPOSES-WITH).
///
/// No raw argument bytes, no PII, no secrets — only the repair classification
/// and the (provider, model, tool) key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolInputRepairEvent {
    /// The OpenAI-compat provider name (e.g. `"xAI"`, `"OpenAI"`, `"DeepSeek"`).
    pub provider: String,
    /// The model identifier the repair-rate is keyed on.
    pub model: String,
    /// The tool name whose input was repaired/surfaced.
    pub tool: String,
    /// The primary Kind discriminator (SSB payload `kind`).
    pub kind: ToolInputRepairKind,
    /// The detailed shape-repair kinds (empty for `RelationalDefaulted` /
    /// `UnrepairableSurfaced`; non-empty for `ShapeRepaired`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repair_kinds: Vec<RepairKind>,
    /// The relational default applied, if any (carried alongside
    /// `ShapeRepaired` when both fired, or as the sole detail for
    /// `RelationalDefaulted`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relational_default: Option<RelationalDefault>,
    /// Unix-epoch milliseconds when the event was produced.
    pub ts_ms: i64,
}

impl ToolInputRepairEvent {
    /// Construct a new repair event with a server-side timestamp.
    #[must_use]
    pub fn new(
        provider: &str,
        model: &str,
        tool: &str,
        kind: ToolInputRepairKind,
        repair_kinds: Vec<RepairKind>,
        relational_default: Option<RelationalDefault>,
    ) -> Self {
        let ts_ms = std::time::SystemTime::now() // entropy-allow: ToolInputRepairEvent::new is a free constructor, no ClockProvider threaded through the OpenAI-compat repair shim; observability ts_ms for the repair event, not a replayed value
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        Self {
            provider: provider.to_owned(),
            model: model.to_owned(),
            tool: tool.to_owned(),
            kind,
            repair_kinds,
            relational_default,
            ts_ms,
        }
    }
}

/// Kill-switch env var. When set, the legacy passthrough (`{"raw": ...}`) is
/// used and the disable is logged. W-2: default-ON, this is a DISABLE not an
/// ENABLE.
const REPAIR_DISABLE_ENV: &str = "ELAI_TOOLCALL_REPAIR_DISABLE";

/// Returns `true` if the repair shim is disabled via the kill-switch env var.
fn repair_disabled() -> bool {
    match std::env::var(REPAIR_DISABLE_ENV) {
        Ok(value) => !value.is_empty()
            && !value.eq_ignore_ascii_case("false")
            && !value.eq_ignore_ascii_case("0"),
        Err(std::env::VarError::NotPresent) => false,
        Err(std::env::VarError::NotUnicode(_)) => false,
    }
}

/// The validate-then-repair entry point.
///
/// Call this from the OpenAI-compat response -> `tool_use` conversion
/// (`parse_tool_arguments` seam) with the tool's name, the raw argument string
/// the model emitted, and the tool's declared `input_schema` (from the request
/// `tools` list). When `input_schema` is `None` (no schema available), the
/// input is parsed as-is and shipped unchanged if valid, else `Unrepairable`.
///
/// # Algorithm (plan J-53 leg 1)
///
/// 1. Parse as-is. If it parses AND structurally matches the schema, ship
///    UNTOUCHED (`ShipUnchanged`).
/// 2. On parse/shape failure, walk the schema's declared type at each failing
///    field and apply the four repairs: null-of-optional DROP (cleanup pass,
///    independent fields), then per array-declared field the load-bearing order
///    json-array-parse BEFORE bare-string-wrap, then single-arg-`{}`-unwrap.
/// 3. Typed-path unwrap: for path-typed fields, strip the degenerate markdown
///    auto-link `[text](url)` ONLY where link-text == url-without-protocol.
/// 4. Re-validate. If it now matches, `Repaired`; else `Unrepairable` with a
///    model-readable message.
pub fn repair_tool_input(
    tool_name: &str,
    raw_args: &str,
    input_schema: Option<&Value>,
) -> RepairOutcome {
    if repair_disabled() {
        warn!(
            env = REPAIR_DISABLE_ENV,
            tool = %tool_name,
            "J-53 tool-input repair disabled; using legacy passthrough"
        );
        return legacy_passthrough(raw_args);
    }

    // (a) parse as-is
    let mut value: Value = match serde_json::from_str(raw_args) {
        Ok(parsed) => parsed,
        Err(_) => {
            return RepairOutcome::Unrepairable {
                model_readable_msg: format!(
                    "tool `{tool_name}` was called but its arguments are not valid JSON. \
                     Raw arguments: {raw_args}. \
                     Please re-issue the tool call with valid JSON arguments matching the tool's input schema."
                ),
            };
        }
    };

    let Some(schema) = input_schema else {
        // No schema available — ship as-is if it parsed.
        debug!(tool = %tool_name, "J-53: no input_schema available; shipping parsed value unchanged");
        return RepairOutcome::ShipUnchanged { value };
    };

    // (a) structurally matches? — but still check typed-path unwrap (c),
    // which is a content-leak repair that fires even on structurally-valid
    // inputs (a path field containing a degenerate markdown auto-link IS a
    // valid string, but it's a chat-distribution leak that should be stripped).
    let structurally_valid = structurally_matches(&value, schema);

    // (b) + (c) apply repairs
    let mut kinds = Vec::new();
    if let Some(obj) = value.as_object_mut() {
        if !structurally_valid {
            apply_field_repairs(obj, schema, &mut kinds);
        }
        // typed-path unwrap runs regardless of structural validity — it's a
        // content-leak repair, not a structural-type repair (plan step (c)).
        apply_typed_path_unwrap(obj, schema, &mut kinds);
    }

    // (d) re-validate
    if structurally_matches(&value, schema) {
        if kinds.is_empty() {
            // No repairs fired — ship unchanged (byte-identical).
            return RepairOutcome::ShipUnchanged { value };
        }
        debug!(tool = %tool_name, kinds = ?kinds, "J-53: tool input repaired");
        return RepairOutcome::Repaired { value, kinds };
    }

    // Repairs applied but still doesn't match — surface a readable error.
    RepairOutcome::Unrepairable {
        model_readable_msg: format!(
            "tool `{tool_name}` was called with arguments that do not match its input schema and \
             could not be automatically repaired. Raw arguments: {raw_args}. \
             Please re-issue the tool call with arguments matching the declared schema."
        ),
    }
}

/// Legacy passthrough: parse as-is, fall back to `{"raw": arguments}` on
/// failure. Used when the kill-switch is set.
fn legacy_passthrough(raw_args: &str) -> RepairOutcome {
    match serde_json::from_str::<Value>(raw_args) {
        Ok(value) => RepairOutcome::ShipUnchanged { value },
        Err(_) => RepairOutcome::ShipUnchanged {
            value: serde_json::json!({ "raw": raw_args }),
        },
    }
}

/// Minimal structural type-matcher against a JSON Schema's declared "type".
///
/// Checks the shape (object/array/scalar) and recurses into `properties` and
/// `items`. Does NOT validate constraints like `minLength`, `pattern`,
/// `enum`, etc. — those are the deferred `jsonschema` crate's job
/// (user-confirmation-flagged, not this step).
fn structurally_matches(value: &Value, schema: &Value) -> bool {
    let schema_type = schema.get("type").and_then(|t| t.as_str());
    match schema_type {
        Some("object") => matches_object(value, schema),
        Some("array") => matches_array(value, schema),
        Some("string") => value.is_string(),
        Some("integer") => value.is_i64() || value.is_u64() || (value.is_f64() && value.as_f64().is_some_and(|f| f.fract() == 0.0)),
        Some("number") => value.is_number(),
        Some("boolean") => value.is_boolean(),
        Some("null") => value.is_null(),
        // Unknown or missing type constraint — accept anything.
        _ => true,
    }
}

fn matches_object(value: &Value, schema: &Value) -> bool {
    let Some(obj) = value.as_object() else {
        return false;
    };
    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        for (key, prop_schema) in properties {
            if let Some(field_val) = obj.get(key) {
                if !structurally_matches(field_val, prop_schema) {
                    return false;
                }
            }
        }
    }
    if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
        for req in required {
            if let Some(req_str) = req.as_str() {
                if !obj.contains_key(req_str) {
                    return false;
                }
            }
        }
    }
    true
}

fn matches_array(value: &Value, schema: &Value) -> bool {
    let Some(arr) = value.as_array() else {
        return false;
    };
    if let Some(items_schema) = schema.get("items") {
        for elem in arr {
            if !structurally_matches(elem, items_schema) {
                return false;
            }
        }
    }
    true
}

/// Apply the four field-level repairs to an object's properties.
///
/// Two passes: (1) null-of-optional DROP (cleanup, independent fields); (2) per
/// array-declared field, load-bearing: json-array-parse BEFORE bare-string-wrap, then single-arg-`{}`-unwrap.
fn apply_field_repairs(obj: &mut Map<String, Value>, schema: &Value, kinds: &mut Vec<RepairKind>) {
    let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) else {
        return;
    };
    let required: Vec<String> = schema
        .get("required")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default();

    // null-of-optional DROP: collect keys to remove (can't remove while
    // iterating the map mutably).
    let null_optional_keys: Vec<String> = obj
        .iter()
        .filter(|(key, val)| {
            val.is_null() && !required.iter().any(|r| r == *key) && properties.contains_key(*key)
        })
        .map(|(key, _)| key.clone())
        .collect();

    for key in &null_optional_keys {
        obj.remove(key);
        kinds.push(RepairKind::NullOfOptionalDrop);
    }

    // Now walk remaining fields for array-declared repairs.
    for (key, prop_schema) in properties {
        let Some(field_val) = obj.get_mut(key) else {
            continue;
        };
        let declared_array = prop_schema
            .get("type")
            .and_then(|t| t.as_str())
            == Some("array");
        if !declared_array || field_val.is_array() {
            continue;
        }

        // json-array-parse: a `"[\"a\",\"b\"]"` string where an array is declared.
        if let Some(s) = field_val.as_str() {
            if let Ok(Value::Array(parsed)) = serde_json::from_str::<Value>(s) {
                *field_val = Value::Array(parsed);
                kinds.push(RepairKind::JsonArrayParse);
                continue;
            }
            // bare-string-wrap: `"foo"` -> `["foo"]` where an array is declared.
            *field_val = Value::Array(vec![Value::String(s.to_string())]);
            kinds.push(RepairKind::BareStringWrap);
            continue;
        }

        // single-arg-{}-unwrap: a placeholder object where an array is declared.
        if let Some(inner_obj) = field_val.as_object() {
            if inner_obj.len() == 1 {
                if let Some(inner_val) = inner_obj.values().next() {
                    if inner_val.is_array() {
                        *field_val = inner_val.clone();
                        kinds.push(RepairKind::SingleArgObjectUnwrap);
                    }
                }
            }
        }
    }
}

/// Apply typed-path unwrap: for path-typed fields, strip the degenerate
/// markdown auto-link `[text](url)` ONLY where link-text == url-without-protocol.
fn apply_typed_path_unwrap(obj: &mut Map<String, Value>, schema: &Value, kinds: &mut Vec<RepairKind>) {
    let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) else {
        return;
    };
    for (key, prop_schema) in properties {
        if !is_path_typed(key, prop_schema) {
            continue;
        }
        let Some(field_val) = obj.get_mut(key) else {
            continue;
        };
        let Some(s) = field_val.as_str() else {
            continue;
        };
        if let Some(stripped) = strip_degenerate_markdown_link(s) {
            *field_val = Value::String(stripped);
            kinds.push(RepairKind::TypedPathUnwrap);
        }
    }
}

/// Heuristic: is this field path-typed? Checks the field name (contains
/// "path", "file", or "dir") or a `format: "path"` annotation in the schema.
fn is_path_typed(key: &str, prop_schema: &Value) -> bool {
    let key_lower = key.to_lowercase();
    if key_lower.contains("path") || key_lower.contains("file") || key_lower.contains("dir") {
        return true;
    }
    prop_schema
        .get("format")
        .and_then(|f| f.as_str())
        .is_some_and(|f| f == "path")
}

/// Strip a degenerate markdown auto-link `[text](url)` to `text` ONLY when
/// link-text == url-without-protocol. Real links (`[click](https://x.com)`)
/// pass through unchanged (return `None`).
fn strip_degenerate_markdown_link(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if !trimmed.starts_with('[') {
        return None;
    }
    let close_bracket = trimmed.find(']')?;
    let text = &trimmed[1..close_bracket];
    let rest = &trimmed[close_bracket + 1..];
    if !rest.starts_with('(') {
        return None;
    }
    let close_paren = rest.find(')')?;
    let url = &rest[1..close_paren];
    let after = &rest[close_paren + 1..];
    if !after.is_empty() {
        return None; // the link is not the entire string
    }
    let url_no_proto = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap_or(url);
    // Degenerate when text == url (exact) OR text == url-without-protocol.
    // e.g. `[/foo](/foo)`, `[https://x.com](https://x.com)`, `[x.com](https://x.com)`.
    if text == url || text == url_no_proto {
        Some(text.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Serializes tests that touch the `ELAI_TOOLCALL_REPAIR_DISABLE` env var.
    /// Without this, the kill-switch test's `set_var`/`remove_var` races with
    /// other tests running in parallel, causing flaky failures.
    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        use std::sync::{Mutex, OnceLock};
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    // --- structurally_matches ---

    #[test]
    fn valid_object_matches() {
        let schema = json!({"type": "object", "properties": {"city": {"type": "string"}}});
        let value = json!({"city": "Paris"});
        assert!(structurally_matches(&value, &schema));
    }

    #[test]
    fn string_where_array_expected_does_not_match() {
        let schema = json!({"type": "object", "properties": {"items": {"type": "array"}}});
        let value = json!({"items": "foo"});
        assert!(!structurally_matches(&value, &schema));
    }

    #[test]
    fn null_where_string_expected_does_not_match() {
        let schema = json!({"type": "object", "properties": {"path": {"type": "string"}}, "required": ["path"]});
        let value = json!({"path": null});
        assert!(!structurally_matches(&value, &schema));
    }

    // --- repair_tool_input: ShipUnchanged ---

    #[test]
    fn valid_input_ships_unchanged() {
        let _lock = env_lock();
        std::env::remove_var("ELAI_TOOLCALL_REPAIR_DISABLE");
        let schema = json!({"type": "object", "properties": {"city": {"type": "string"}}, "required": ["city"]});
        let outcome = repair_tool_input("weather", r#"{"city":"Paris"}"#, Some(&schema));
        assert_eq!(outcome, RepairOutcome::ShipUnchanged { value: json!({"city": "Paris"}) });
    }

    #[test]
    fn valid_input_with_array_ships_unchanged() {
        let _lock = env_lock();
        std::env::remove_var("ELAI_TOOLCALL_REPAIR_DISABLE");
        let schema = json!({"type": "object", "properties": {"items": {"type": "array", "items": {"type": "string"}}}});
        let outcome = repair_tool_input("list", r#"{"items":["a","b"]}"#, Some(&schema));
        assert_eq!(outcome, RepairOutcome::ShipUnchanged { value: json!({"items": ["a", "b"]}) });
    }

    #[test]
    fn no_schema_ships_parsed_value_unchanged() {
        let _lock = env_lock();
        std::env::remove_var("ELAI_TOOLCALL_REPAIR_DISABLE");
        let outcome = repair_tool_input("tool", r#"{"key":"val"}"#, None);
        assert_eq!(outcome, RepairOutcome::ShipUnchanged { value: json!({"key": "val"}) });
    }

    // --- repair_tool_input: Repaired (json-array-parse) ---

    #[test]
    fn stringified_array_is_parsed() {
        let _lock = env_lock();
        std::env::remove_var("ELAI_TOOLCALL_REPAIR_DISABLE");
        let schema = json!({"type": "object", "properties": {"items": {"type": "array", "items": {"type": "string"}}}});
        let raw = r#"{"items":"[\"a\",\"b\"]"}"#;
        let outcome = repair_tool_input("list", raw, Some(&schema));
        match outcome {
            RepairOutcome::Repaired { value, kinds } => {
                assert_eq!(value, json!({"items": ["a", "b"]}));
                assert!(kinds.contains(&RepairKind::JsonArrayParse));
            }
            other => panic!("expected Repaired, got {other:?}"),
        }
    }

    // --- repair_tool_input: Repaired (bare-string-wrap) ---

    #[test]
    fn bare_string_is_wrapped_into_array() {
        let _lock = env_lock();
        std::env::remove_var("ELAI_TOOLCALL_REPAIR_DISABLE");
        let schema = json!({"type": "object", "properties": {"items": {"type": "array", "items": {"type": "string"}}}});
        let raw = r#"{"items":"foo"}"#;
        let outcome = repair_tool_input("list", raw, Some(&schema));
        match outcome {
            RepairOutcome::Repaired { value, kinds } => {
                assert_eq!(value, json!({"items": ["foo"]}));
                assert!(kinds.contains(&RepairKind::BareStringWrap));
            }
            other => panic!("expected Repaired, got {other:?}"),
        }
    }

    /// The ORDER invariant: a stringified array must be PARSED, not
    /// double-wrapped. If bare-string-wrap ran first, `'["a","b"]'` would
    /// become `['["a","b"]']` — a corruption. This test locks the order.
    #[test]
    fn array_parse_before_bare_string_wrap_order_invariant() {
        let _lock = env_lock();
        std::env::remove_var("ELAI_TOOLCALL_REPAIR_DISABLE");
        let schema = json!({"type": "object", "properties": {"items": {"type": "array", "items": {"type": "string"}}}});
        let raw = r#"{"items":"[\"a\",\"b\"]"}"#;
        let outcome = repair_tool_input("list", raw, Some(&schema));
        match outcome {
            RepairOutcome::Repaired { value, kinds } => {
                assert_eq!(value, json!({"items": ["a", "b"]}));
                assert!(kinds.contains(&RepairKind::JsonArrayParse));
                assert!(!kinds.contains(&RepairKind::BareStringWrap));
            }
            other => panic!("expected Repaired, got {other:?}"),
        }
    }

    // --- repair_tool_input: Repaired (single-arg-{}-unwrap) ---

    #[test]
    fn placeholder_object_is_unwrapped_into_array() {
        let _lock = env_lock();
        std::env::remove_var("ELAI_TOOLCALL_REPAIR_DISABLE");
        let schema = json!({"type": "object", "properties": {"items": {"type": "array", "items": {"type": "string"}}}});
        let raw = r#"{"items":{"placeholder":["a","b"]}}"#;
        let outcome = repair_tool_input("list", raw, Some(&schema));
        match outcome {
            RepairOutcome::Repaired { value, kinds } => {
                assert_eq!(value, json!({"items": ["a", "b"]}));
                assert!(kinds.contains(&RepairKind::SingleArgObjectUnwrap));
            }
            other => panic!("expected Repaired, got {other:?}"),
        }
    }

    // --- repair_tool_input: Repaired (null-of-optional-drop) ---

    #[test]
    fn null_on_optional_field_is_dropped() {
        let _lock = env_lock();
        std::env::remove_var("ELAI_TOOLCALL_REPAIR_DISABLE");
        let schema = json!({
            "type": "object",
            "properties": {"path": {"type": "string"}, "content": {"type": "string"}},
            "required": ["content"]
        });
        let raw = r#"{"path":null,"content":"hello"}"#;
        let outcome = repair_tool_input("write", raw, Some(&schema));
        match outcome {
            RepairOutcome::Repaired { value, kinds } => {
                assert_eq!(value, json!({"content": "hello"}));
                assert!(kinds.contains(&RepairKind::NullOfOptionalDrop));
            }
            other => panic!("expected Repaired, got {other:?}"),
        }
    }

    #[test]
    fn null_on_required_field_is_unrepairable() {
        let _lock = env_lock();
        std::env::remove_var("ELAI_TOOLCALL_REPAIR_DISABLE");
        let schema = json!({
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"]
        });
        let raw = r#"{"path":null}"#;
        let outcome = repair_tool_input("write", raw, Some(&schema));
        assert!(matches!(outcome, RepairOutcome::Unrepairable { .. }));
    }

    // --- repair_tool_input: Repaired (typed-path-unwrap) ---

    #[test]
    fn degenerate_markdown_link_on_path_is_stripped() {
        let _lock = env_lock();
        std::env::remove_var("ELAI_TOOLCALL_REPAIR_DISABLE");
        let schema = json!({"type": "object", "properties": {"file_path": {"type": "string"}}, "required": ["file_path"]});
        let raw = r#"{"file_path":"[/foo/bar](/foo/bar)"}"#;
        let outcome = repair_tool_input("read", raw, Some(&schema));
        match outcome {
            RepairOutcome::Repaired { value, kinds } => {
                assert_eq!(value, json!({"file_path": "/foo/bar"}));
                assert!(kinds.contains(&RepairKind::TypedPathUnwrap));
            }
            other => panic!("expected Repaired, got {other:?}"),
        }
    }

    #[test]
    fn real_markdown_link_on_path_passes_through() {
        let _lock = env_lock();
        std::env::remove_var("ELAI_TOOLCALL_REPAIR_DISABLE");
        let schema = json!({"type": "object", "properties": {"file_path": {"type": "string"}}, "required": ["file_path"]});
        // This is a valid string matching the schema — should ship unchanged.
        let raw = r#"{"file_path":"[click](https://x.com)"}"#;
        let outcome = repair_tool_input("read", raw, Some(&schema));
        assert_eq!(outcome, RepairOutcome::ShipUnchanged { value: json!({"file_path": "[click](https://x.com)"}) });
    }

    // --- repair_tool_input: Unrepairable ---

    #[test]
    fn unparseable_args_are_unrepairable() {
        let _lock = env_lock();
        std::env::remove_var("ELAI_TOOLCALL_REPAIR_DISABLE");
        let schema = json!({"type": "object"});
        let outcome = repair_tool_input("tool", "not-json", Some(&schema));
        match outcome {
            RepairOutcome::Unrepairable { model_readable_msg } => {
                assert!(model_readable_msg.contains("not valid JSON"));
                assert!(model_readable_msg.contains("tool `tool`"));
            }
            other => panic!("expected Unrepairable, got {other:?}"),
        }
    }

    #[test]
    fn irreparable_shape_is_unrepairable_with_readable_msg() {
        let _lock = env_lock();
        std::env::remove_var("ELAI_TOOLCALL_REPAIR_DISABLE");
        let schema = json!({"type": "object", "properties": {"count": {"type": "integer"}}, "required": ["count"]});
        // count is an object, not an integer — no repair applies.
        let raw = r#"{"count":{"nested":"bad"}}"#;
        let outcome = repair_tool_input("tool", raw, Some(&schema));
        assert!(matches!(outcome, RepairOutcome::Unrepairable { .. }));
    }

    // --- kill-switch ---

    #[test]
    fn kill_switch_uses_legacy_passthrough() {
        let _lock = env_lock();
        std::env::set_var("ELAI_TOOLCALL_REPAIR_DISABLE", "1");
        let outcome = repair_tool_input("tool", "not-json", None);
        std::env::remove_var("ELAI_TOOLCALL_REPAIR_DISABLE");
        // Legacy passthrough: {"raw": "not-json"}
        assert_eq!(
            outcome,
            RepairOutcome::ShipUnchanged { value: json!({"raw": "not-json"}) }
        );
    }

    // --- strip_degenerate_markdown_link ---

    #[test]
    fn strip_link_text_equals_url() {
        assert_eq!(
            strip_degenerate_markdown_link("[/foo](/foo)"),
            Some("/foo".to_string())
        );
    }

    #[test]
    fn strip_link_text_equals_url_with_protocol() {
        assert_eq!(
            strip_degenerate_markdown_link("[https://x.com](https://x.com)"),
            Some("https://x.com".to_string())
        );
    }

    #[test]
    fn strip_link_text_not_equal_url_returns_none() {
        assert_eq!(strip_degenerate_markdown_link("[click](https://x.com)"), None);
    }

    #[test]
    fn strip_non_link_returns_none() {
        assert_eq!(strip_degenerate_markdown_link("/foo/bar"), None);
    }

    #[test]
    fn strip_link_with_trailing_text_returns_none() {
        assert_eq!(strip_degenerate_markdown_link("[/foo](/foo) extra"), None);
    }

    // --- apply_relational_defaults (J-53 step 4) ---

    /// Schema for an offset/limit-class tool (matches `read_file` shape).
    fn read_file_schema() -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "offset": { "type": "integer", "minimum": 0 },
                "limit": { "type": "integer", "minimum": 1 }
            },
            "required": ["path"]
        })
    }

    #[test]
    fn limit_alone_defaults_offset_to_zero() {
        let schema = read_file_schema();
        let mut value = json!({"path": "/foo", "limit": 50});
        let default = apply_relational_defaults("read_file", &mut value, Some(&schema));
        assert!(default.is_some(), "expected a relational default");
        let default = default.unwrap();
        assert_eq!(default.field, "offset");
        assert_eq!(default.default_value, "0");
        assert_eq!(value, json!({"path": "/foo", "limit": 50, "offset": 0}));
        // Note must NOT have an `Error:` prefix (plan J-53).
        assert!(!default.note.starts_with("Error:"));
        assert!(default.note.contains("offset defaulted to 0"));
    }

    #[test]
    fn offset_alone_defaults_limit_to_default() {
        let schema = read_file_schema();
        let mut value = json!({"path": "/foo", "offset": 10});
        let default = apply_relational_defaults("read_file", &mut value, Some(&schema));
        assert!(default.is_some(), "expected a relational default");
        let default = default.unwrap();
        assert_eq!(default.field, "limit");
        // DEFAULT_LIMIT = 200 (matches file_read.rs::DEFAULT_LINE_LIMIT)
        assert_eq!(default.default_value, "200");
        assert_eq!(value, json!({"path": "/foo", "offset": 10, "limit": 200}));
        assert!(!default.note.starts_with("Error:"));
        assert!(default.note.contains("limit defaulted to 200"));
    }

    #[test]
    fn offset_alone_uses_schema_default_when_present() {
        let schema = json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "offset": { "type": "integer", "minimum": 0 },
                "limit": { "type": "integer", "minimum": 1, "default": 100 }
            },
            "required": ["path"]
        });
        let mut value = json!({"path": "/foo", "offset": 5});
        let default = apply_relational_defaults("read_file", &mut value, Some(&schema));
        assert!(default.is_some(), "expected a relational default");
        let default = default.unwrap();
        assert_eq!(default.default_value, "100");
        assert_eq!(value, json!({"path": "/foo", "offset": 5, "limit": 100}));
    }

    #[test]
    fn both_offset_and_limit_present_no_default() {
        let schema = read_file_schema();
        let mut value = json!({"path": "/foo", "offset": 10, "limit": 50});
        let default = apply_relational_defaults("read_file", &mut value, Some(&schema));
        assert!(default.is_none(), "both present -> no default needed");
        assert_eq!(value, json!({"path": "/foo", "offset": 10, "limit": 50}));
    }

    #[test]
    fn neither_offset_nor_limit_no_default() {
        let schema = read_file_schema();
        let mut value = json!({"path": "/foo"});
        let default = apply_relational_defaults("read_file", &mut value, Some(&schema));
        assert!(default.is_none(), "neither present -> let tool handle defaults");
        assert_eq!(value, json!({"path": "/foo"}));
    }

    #[test]
    fn non_offset_limit_tool_no_default() {
        // A tool without offset/limit properties should not be affected.
        let schema = json!({
            "type": "object",
            "properties": {
                "city": { "type": "string" }
            },
            "required": ["city"]
        });
        let mut value = json!({"city": "Paris"});
        let default = apply_relational_defaults("weather", &mut value, Some(&schema));
        assert!(default.is_none());
        assert_eq!(value, json!({"city": "Paris"}));
    }

    #[test]
    fn no_schema_no_default() {
        let mut value = json!({"path": "/foo", "limit": 50});
        let default = apply_relational_defaults("read_file", &mut value, None);
        assert!(default.is_none(), "no schema -> cannot determine offset/limit class");
        assert_eq!(value, json!({"path": "/foo", "limit": 50}));
    }

    #[test]
    fn null_offset_treated_as_absent() {
        let schema = read_file_schema();
        // If offset is explicitly null (and wasn't dropped by shape repair),
        // it should be treated as absent and defaulted to 0.
        let mut value = json!({"path": "/foo", "offset": null, "limit": 50});
        let default = apply_relational_defaults("read_file", &mut value, Some(&schema));
        assert!(default.is_some());
        assert_eq!(value, json!({"path": "/foo", "offset": 0, "limit": 50}));
    }
}
