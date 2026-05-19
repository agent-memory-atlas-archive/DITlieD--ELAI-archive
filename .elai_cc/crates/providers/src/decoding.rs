//! C-26: Constrained-decoding mode + per-role defaults.
//!
//! This module defines [`DecodingMode`] (the four-variant enum used by both
//! the sidecar and the cloud grounding client) and [`ConstrainedDecodingConfig`]
//! (the wire-form policy attached to a single completion request).
//!
//! Why this lives in `elai-providers` (not orchestrator): the sidecar client
//! and the Anthropic cloud client are both `elai-providers` types, and they
//! are the two consumers that translate `DecodingMode` into HTTP body fields
//! (`response_format` / `grammar_path` for the sidecar; Anthropic Tool API
//! `tools` + `tool_choice` for cloud Sonnet). Hosting the type in providers
//! keeps the orchestrator -> providers dependency edge one-way and avoids a
//! circular crate dep.
//!
//! Per-role defaults use a local [`RoleDecoding`] enum rather than re-importing
//! `elai_orchestrator::AgentRole` (decision in workflow/reasoning/
//! C-26-decisions.md DECISION-A).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// DecodingMode
// ---------------------------------------------------------------------------

/// Constrained-decoding mode for a single completion call.
///
/// Four variants:
/// * [`DecodingMode::JsonSchema`] — server-side JSON-schema enforcement
///   (sidecar: lm-format-enforcer / outlines; cloud: Anthropic Tool API
///   `input_schema`).
/// * [`DecodingMode::Grammar`] — EBNF grammar enforcement (sidecar-only;
///   cloud falls back to FreeText). `crane_adaptive` is a forward-compat
///   flag for the CRANE adaptive-grammar runtime (not yet implemented).
/// * [`DecodingMode::Regex`] — regex pattern enforcement (sidecar-only).
/// * [`DecodingMode::FreeText`] — no constraint; default for every role
///   that does not declare otherwise.
///
/// The variant carries the payload directly (no `Option<String>` wrapper)
/// because every constrained variant is meaningless without its payload —
/// representing "JsonSchema with no schema" is a bug, not a state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum DecodingMode {
    /// JSON-schema enforcement. `schema_json` is the schema as a JSON string
    /// (caller pre-validates that it parses; we do not re-parse here so the
    /// type stays `Eq`-derivable).
    JsonSchema { schema_json: String },
    /// EBNF-grammar enforcement. `ebnf_path` is resolved by the caller (see
    /// workflow/reasoning/C-26-decisions.md DECISION-B for the resolution
    /// strategy). `crane_adaptive` enables the future CRANE adaptive
    /// runtime; today it only affects [`crane_adaptive_eligible`].
    Grammar { ebnf_path: PathBuf, crane_adaptive: bool },
    /// Regex-pattern enforcement.
    Regex { pattern: String },
    /// No decoding constraint. Default for the policy.
    FreeText,
}

impl Default for DecodingMode {
    fn default() -> Self {
        Self::FreeText
    }
}

impl DecodingMode {
    /// Discriminant label used in SSB beacons and log lines. Stable
    /// snake-case strings; do NOT change without coordinating SSB readers.
    pub fn discriminant(&self) -> &'static str {
        match self {
            Self::JsonSchema { .. } => "json_schema",
            Self::Grammar { .. } => "grammar",
            Self::Regex { .. } => "regex",
            Self::FreeText => "free_text",
        }
    }
}

// ---------------------------------------------------------------------------
// ConstrainedDecodingConfig
// ---------------------------------------------------------------------------

/// Per-call constrained-decoding configuration. Attached to a sidecar /
/// cloud request by the orchestrator dispatcher; the receiving client
/// (`SidecarProvider::build_completion_body` or
/// `CloudGroundingClient::build_request_with_decoding`) translates the
/// payload into transport-specific request fields.
///
/// `Default` produces `FreeText` with `max_retries_on_invalid = 3` and
/// `strict = true` so callers that pass `ConstrainedDecodingConfig::default()`
/// observe the legacy un-constrained behaviour.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstrainedDecodingConfig {
    /// The decoding constraint to apply. `FreeText` disables the constraint.
    pub mode: DecodingMode,
    /// Maximum retries on validation failure (sidecar: schema/grammar reject;
    /// cloud: schema reject from Tool API). Caller-side retry loop. Default 3.
    pub max_retries_on_invalid: u8,
    /// When `true`, validation failures escalate after `max_retries_on_invalid`.
    /// When `false`, the client falls back to `FreeText` on the last attempt
    /// without escalating. Default `true`.
    pub strict: bool,
}

impl Default for ConstrainedDecodingConfig {
    fn default() -> Self {
        Self {
            mode: DecodingMode::FreeText,
            max_retries_on_invalid: 3,
            strict: true,
        }
    }
}

impl ConstrainedDecodingConfig {
    /// Convenience constructor for a JSON-schema config.
    pub fn json_schema(schema_json: impl Into<String>) -> Self {
        Self {
            mode: DecodingMode::JsonSchema { schema_json: schema_json.into() },
            max_retries_on_invalid: 3,
            strict: true,
        }
    }

    /// Convenience constructor for an EBNF-grammar config.
    pub fn grammar(ebnf_path: impl Into<PathBuf>, crane_adaptive: bool) -> Self {
        Self {
            mode: DecodingMode::Grammar { ebnf_path: ebnf_path.into(), crane_adaptive },
            max_retries_on_invalid: 3,
            strict: true,
        }
    }

    /// Convenience constructor for a regex config.
    pub fn regex(pattern: impl Into<String>) -> Self {
        Self {
            mode: DecodingMode::Regex { pattern: pattern.into() },
            max_retries_on_invalid: 3,
            strict: true,
        }
    }
}

// ---------------------------------------------------------------------------
// CRANE adaptive helpers
// ---------------------------------------------------------------------------

/// Returns `true` when the supplied [`DecodingMode`] is eligible for the
/// CRANE adaptive-grammar runtime. Only `Grammar { crane_adaptive: true }`
/// qualifies today; every other mode returns `false`.
///
/// CRANE enforcement itself is out-of-scope for C-26 — this helper is the
/// gate the future CRANE patch will read.
pub fn crane_adaptive_eligible(mode: &DecodingMode) -> bool {
    matches!(mode, DecodingMode::Grammar { crane_adaptive: true, .. })
}

// ---------------------------------------------------------------------------
// Per-role defaults
// ---------------------------------------------------------------------------

/// Local role enum used only by the per-role default table.
///
/// We deliberately do NOT import `elai_orchestrator::AgentRole` here: that
/// would create a `providers -> orchestrator` dependency edge that already
/// exists in the opposite direction. The five roles below cover the
/// constrained-decoding policy surface — extending it requires touching
/// only this enum + [`PER_ROLE_DECODING_DEFAULTS`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleDecoding {
    Designer,
    Executor,
    Critic,
    Auditor,
    Researcher,
}

impl RoleDecoding {
    /// Stable snake_case label written into SSB beacons.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Designer => "designer",
            Self::Executor => "executor",
            Self::Critic => "critic",
            Self::Auditor => "auditor",
            Self::Researcher => "researcher",
        }
    }
}

/// Per-role default decoding configuration accessor.
///
/// Returns a [`ConstrainedDecodingConfig`] keyed on role. Centralised here
/// so the orchestrator dispatcher can look up "what decoding mode does role
/// X use by default" without hardcoding the policy at each call site.
///
/// Defaults (from C-26 plan §Task 1):
/// * Designer  -> FreeText (writes prose plans; no schema)
/// * Executor  -> Grammar (function-signature EBNF; CRANE-eligible)
/// * Critic    -> JsonSchema (returns structured verdict objects)
/// * Auditor   -> JsonSchema (returns structured findings objects)
/// * Researcher-> FreeText (returns prose research notes)
pub fn default_decoding_for(role: RoleDecoding) -> ConstrainedDecodingConfig {
    match role {
        RoleDecoding::Designer => ConstrainedDecodingConfig::default(),
        RoleDecoding::Executor => ConstrainedDecodingConfig {
            // Resolution strategy: caller (orchestrator dispatcher) joins this
            // relative path against the grammars/ dir before forwarding to
            // the sidecar. See workflow/reasoning/C-26-decisions.md
            // DECISION-B for the caller-resolves rationale.
            mode: DecodingMode::Grammar {
                ebnf_path: PathBuf::from("file_plan.ebnf"),
                crane_adaptive: true,
            },
            max_retries_on_invalid: 3,
            strict: true,
        },
        RoleDecoding::Critic => ConstrainedDecodingConfig {
            mode: DecodingMode::JsonSchema {
                schema_json: CRITIC_VERDICT_SCHEMA.to_owned(),
            },
            max_retries_on_invalid: 3,
            strict: true,
        },
        RoleDecoding::Auditor => ConstrainedDecodingConfig {
            mode: DecodingMode::JsonSchema {
                schema_json: AUDITOR_FINDING_SCHEMA.to_owned(),
            },
            max_retries_on_invalid: 3,
            strict: true,
        },
        RoleDecoding::Researcher => ConstrainedDecodingConfig::default(),
    }
}

/// Compile-time per-role default table. Used by the contract tests to assert
/// the five role -> mode invariants without re-running `default_decoding_for`
/// at every call site. Order matches [`RoleDecoding`] declaration.
pub const PER_ROLE_DECODING_DEFAULTS: &[(RoleDecoding, &str); 5] = &[
    (RoleDecoding::Designer, "free_text"),
    (RoleDecoding::Executor, "grammar"),
    (RoleDecoding::Critic, "json_schema"),
    (RoleDecoding::Auditor, "json_schema"),
    (RoleDecoding::Researcher, "free_text"),
];

/// Minimal JSON-schema string for the critic verdict object.
/// Carries verdict (Allow/Block) + findings array.
const CRITIC_VERDICT_SCHEMA: &str = r#"{
  "type": "object",
  "properties": {
    "verdict": { "type": "string", "enum": ["allow", "block"] },
    "findings": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "severity": { "type": "string" },
          "message": { "type": "string" }
        },
        "required": ["severity", "message"]
      }
    }
  },
  "required": ["verdict", "findings"]
}"#;

/// Minimal JSON-schema string for the auditor finding object.
const AUDITOR_FINDING_SCHEMA: &str = r#"{
  "type": "object",
  "properties": {
    "block_count": { "type": "integer", "minimum": 0 },
    "warn_count": { "type": "integer", "minimum": 0 },
    "findings": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "level": { "type": "string", "enum": ["block", "warn", "info"] },
          "message": { "type": "string" }
        },
        "required": ["level", "message"]
      }
    }
  },
  "required": ["block_count", "warn_count", "findings"]
}"#;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_free_text() {
        let cfg = ConstrainedDecodingConfig::default();
        assert!(matches!(cfg.mode, DecodingMode::FreeText));
        assert_eq!(cfg.max_retries_on_invalid, 3);
        assert!(cfg.strict);
    }

    #[test]
    fn crane_adaptive_true_eligible() {
        let m = DecodingMode::Grammar {
            ebnf_path: PathBuf::from("file_plan.ebnf"),
            crane_adaptive: true,
        };
        assert!(crane_adaptive_eligible(&m));
    }

    #[test]
    fn crane_adaptive_false_not_eligible() {
        let m = DecodingMode::Grammar {
            ebnf_path: PathBuf::from("json_response.ebnf"),
            crane_adaptive: false,
        };
        assert!(!crane_adaptive_eligible(&m));
    }

    #[test]
    fn json_schema_constructor_round_trips() {
        let cfg = ConstrainedDecodingConfig::json_schema(r#"{"type":"object"}"#);
        match &cfg.mode {
            DecodingMode::JsonSchema { schema_json } => {
                assert!(schema_json.contains("object"));
            }
            other => panic!("expected JsonSchema, got {other:?}"),
        }
    }

    #[test]
    fn per_role_table_matches_default_fn() {
        for (role, expected) in PER_ROLE_DECODING_DEFAULTS.iter() {
            let cfg = default_decoding_for(*role);
            assert_eq!(cfg.mode.discriminant(), *expected, "role {role:?}");
        }
    }

    #[test]
    fn discriminant_labels_stable() {
        assert_eq!(DecodingMode::FreeText.discriminant(), "free_text");
        assert_eq!(
            DecodingMode::Regex { pattern: "x".into() }.discriminant(),
            "regex"
        );
        assert_eq!(
            DecodingMode::JsonSchema { schema_json: "{}".into() }.discriminant(),
            "json_schema"
        );
        assert_eq!(
            DecodingMode::Grammar {
                ebnf_path: PathBuf::from("x.ebnf"),
                crane_adaptive: false,
            }
            .discriminant(),
            "grammar"
        );
    }
}
