//! C-26 + C-37: Constrained-decoding mode + per-role defaults.
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
//!
//! # C-37 paper-mapping (Structure Snowballing, arXiv:2604.06066)
//!
//! Per-role decoding hyperparameters (temperature + max_new_tokens +
//! constrained-flag) for Critic + Reflection are grounded against
//! arXiv:2604.06066 Section 4.3. The paper's role taxonomy (Actor /
//! Evaluator / Reflector, all running on Qwen3-8B) maps onto the ELAI
//! 4-role taxonomy as follows:
//!
//!   Actor      <=> CODER       (free-text reasoning generator)
//!   Evaluator  <=> CRITIC      (free-text semantic-equivalence judge)
//!   Reflector  <=> REFLECTION  (constrained 5-class error diagnoser)
//!
//! The paper does NOT define a PLANNER role; ELAI's PLANNER is an internal
//! extension and its decoding defaults remain UNGROUNDED-pending-eval (kept
//! at the existing values).
//!
//! Numbers (paper Section 4.3, verbatim):
//!   All roles: temperature=0.1, max_new_tokens=1024
//!   Reflector: additionally constrained=YES via Outlines + pydantic FSM
//!
//! See [`PER_ROLE_DECODING_PARAMS_PAPER`] for the table and
//! [`paper_decoding_params_for`] for the lookup. The existing C-26 surface
//! ([`PER_ROLE_DECODING_DEFAULTS`] + [`default_decoding_for`]) is unchanged
//! — those govern decoding MODE (JsonSchema / Grammar / FreeText etc),
//! whereas the C-37 surface governs decoding HYPERPARAMETERS (temperature
//! / max_new_tokens). The two are orthogonal; the orchestrator dispatcher
//! consults both per spawn.

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

    /// C-37 — map a snake_case role string (the canonical label used by
    /// `orchestrator::agent_pool::agent_role_to_snake_case`) onto the
    /// [`RoleDecoding`] surface this crate uses for per-role defaults.
    ///
    /// Returns `None` for roles that have no decoding policy mapping
    /// today (e.g. `idea_framer`, `idea_critic`, the per-level auditors).
    /// Callers should treat `None` as "use the legacy un-constrained
    /// behaviour" so adding a new orchestrator role does not require a
    /// providers-crate touch.
    ///
    /// The B-41 per-level auditor siblings collapse to `Auditor` so the
    /// JsonSchema default applies uniformly — matches the spirit of
    /// `auditor_mode::map_to_agent_role`.
    pub fn from_snake_case(role: &str) -> Option<Self> {
        match role {
            "designer" => Some(Self::Designer),
            "executor" => Some(Self::Executor),
            "critic" => Some(Self::Critic),
            "auditor"
            | "auditor_concepts"
            | "auditor_phases"
            | "auditor_steps"
            | "auditor_babysteps"
            | "auditor_babystep_detail" => Some(Self::Auditor),
            "researcher" => Some(Self::Researcher),
            _ => None,
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
// C-37: Per-role decoding HYPERPARAMETERS (temperature / max_new_tokens /
// constrained-flag).
//
// Distinct from the C-26 MODE table above: this surface answers "what
// temperature + token budget + constrained-flag does this role use", not
// "which constraint shape does this role use". Both are consulted per
// spawn.
//
// Paper grounding (arXiv:2604.06066 Section 4.3): all roles use the same
// uniform decoding hyperparameters (temperature=0.1, max_new_tokens=1024);
// only the Reflector flips `constrained=true`. Paper role mapping
// (Actor=CODER, Evaluator=CRITIC, Reflector=REFLECTION) documented at
// module-level above.
// ---------------------------------------------------------------------------

/// Per-role decoding hyperparameters (paper-grounded for Critic +
/// Reflection per arXiv:2604.06066 Section 4.3; PLANNER + CODER kept at
/// current values pending grounded eval).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PaperDecodingParams {
    /// Sampling temperature. Paper Section 4.3: 0.1 ("minimise random
    /// variance"). Higher temps invalidate the alignment-tax claim
    /// because token thrash becomes confounded with sampling stochasticity.
    pub temperature: f32,
    /// Per-trial generation hard ceiling. Paper Section 4.3: 1024.
    pub max_new_tokens: u32,
    /// Whether to apply constrained decoding (Outlines + pydantic FSM in
    /// the paper; the sidecar's `response_format=json_schema` hint here).
    pub constrained: bool,
    /// Grounding tag carried into SSB beacons. `"paper-2604.06066-S4.3"`
    /// for paper-grounded rows; `"ungrounded-pending-eval"` for the
    /// PLANNER + CODER defaults that have not yet been re-evaluated.
    pub grounding: &'static str,
}

/// ELAI's internal 4-role taxonomy for decoding-hyperparameter selection.
///
/// This mirrors the design role used in plan / step decomposition; it is
/// orthogonal to [`RoleDecoding`] (which captures the 5-role wire-policy
/// surface). Paper mapping documented at module-level: Actor=CODER,
/// Evaluator=CRITIC, Reflector=REFLECTION.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaperRole {
    /// Plan-generation role. NOT in the paper; ELAI extension. Decoding
    /// defaults kept at FreeText / temperature=0.7 pending grounded eval.
    Planner,
    /// Reasoning-step generator (paper: Actor / M_a). Free-text.
    Coder,
    /// Semantic-equivalence judge (paper: Evaluator / M_e). Free-text.
    Critic,
    /// Root-cause diagnoser (paper: Reflector / M_r). Constrained via
    /// Outlines + 5-class pydantic FSM.
    Reflection,
}

impl PaperRole {
    /// Stable snake_case label written into SSB beacons.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planner => "planner",
            Self::Coder => "coder",
            Self::Critic => "critic",
            Self::Reflection => "reflection",
        }
    }
}

/// Per-role paper-grounded decoding hyperparameter accessor.
///
/// Returns a [`PaperDecodingParams`] keyed on the 4-role taxonomy. Critic +
/// Reflection values are grounded against arXiv:2604.06066 Section 4.3;
/// Planner + Coder are flagged UNGROUNDED-pending-eval and retain the
/// pre-C-37 ELAI defaults.
pub fn paper_decoding_params_for(role: PaperRole) -> PaperDecodingParams {
    match role {
        PaperRole::Planner => PaperDecodingParams {
            // UNGROUNDED-pending grounded eval: keep ELAI's plan-generation
            // defaults; tracked for future re-evaluation.
            temperature: 0.7,
            max_new_tokens: 2048,
            constrained: false,
            grounding: "ungrounded-pending-eval",
        },
        PaperRole::Coder => PaperDecodingParams {
            // UNGROUNDED-pending grounded eval: ELAI's coder runs warmer
            // than the paper's Actor; paper uses 0.1 across all roles but
            // C-37 scope explicitly defers CODER until measured.
            temperature: 0.4,
            max_new_tokens: 2048,
            constrained: false,
            grounding: "ungrounded-pending-eval",
        },
        PaperRole::Critic => PaperDecodingParams {
            // Paper Section 4.3 (Evaluator decoding params, verbatim).
            temperature: 0.1,
            max_new_tokens: 1024,
            constrained: false,
            grounding: "paper-2604.06066-S4.3",
        },
        PaperRole::Reflection => PaperDecodingParams {
            // Paper Section 3.3 + 4.3 (Reflector decoding params + Outlines
            // FSM trigger). Constrained=true is the structure-snowballing
            // trigger condition under measurement; do not flip without
            // pilot data per C-37 plan §4.
            temperature: 0.1,
            max_new_tokens: 1024,
            constrained: true,
            grounding: "paper-2604.06066-S3.3-S4.3",
        },
    }
}

/// Compile-time per-role paper-grounded hyperparameter table. Used by the
/// contract tests to assert the four role -> params invariants without
/// re-running `paper_decoding_params_for` at every call site. Order
/// matches [`PaperRole`] declaration.
pub const PER_ROLE_DECODING_PARAMS_PAPER: &[(PaperRole, f32, u32, bool); 4] = &[
    (PaperRole::Planner, 0.7, 2048, false),
    (PaperRole::Coder, 0.4, 2048, false),
    (PaperRole::Critic, 0.1, 1024, false),
    (PaperRole::Reflection, 0.1, 1024, true),
];

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
    fn c37_critic_params_match_paper_section_43() {
        let p = paper_decoding_params_for(PaperRole::Critic);
        assert_eq!(p.temperature, 0.1);
        assert_eq!(p.max_new_tokens, 1024);
        assert!(!p.constrained);
        assert_eq!(p.grounding, "paper-2604.06066-S4.3");
    }

    #[test]
    fn c37_reflection_params_match_paper_section_33() {
        let p = paper_decoding_params_for(PaperRole::Reflection);
        assert_eq!(p.temperature, 0.1);
        assert_eq!(p.max_new_tokens, 1024);
        assert!(p.constrained);
        assert_eq!(p.grounding, "paper-2604.06066-S3.3-S4.3");
    }

    #[test]
    fn c37_planner_and_coder_marked_ungrounded() {
        for role in [PaperRole::Planner, PaperRole::Coder] {
            let p = paper_decoding_params_for(role);
            assert_eq!(
                p.grounding, "ungrounded-pending-eval",
                "{role:?} must carry ungrounded marker until measured"
            );
        }
    }

    #[test]
    fn c37_paper_table_matches_lookup_fn() {
        for &(role, temp, mnt, con) in PER_ROLE_DECODING_PARAMS_PAPER.iter() {
            let p = paper_decoding_params_for(role);
            assert_eq!(p.temperature, temp, "{role:?} temperature");
            assert_eq!(p.max_new_tokens, mnt, "{role:?} max_new_tokens");
            assert_eq!(p.constrained, con, "{role:?} constrained");
        }
    }

    #[test]
    fn c37_paper_role_str_labels_stable() {
        assert_eq!(PaperRole::Planner.as_str(), "planner");
        assert_eq!(PaperRole::Coder.as_str(), "coder");
        assert_eq!(PaperRole::Critic.as_str(), "critic");
        assert_eq!(PaperRole::Reflection.as_str(), "reflection");
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
