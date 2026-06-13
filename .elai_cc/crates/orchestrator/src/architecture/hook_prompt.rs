//! E-25 Tranche C (C.2) -- hook_prompt module: LLM-decision hook
//! handler with NEW `PromptDecision` enum.
//!
//! Per the E-25 plan (Task 4) the LLM-decision hook handler must return
//! one of three outcomes: `Allow`, `Deny(reason)`, `Ask(reason)`.
//! Existing enums are not a fit:
//!   * `crate::hook_dispatcher::HookVerdictKind` is `{Allow, Warn, Block}`
//!     (wrong semantics) with 9+ existing use sites at
//!     hook_dispatcher.rs:179/185/221/746/758/781/797/801/828/...; adding
//!     an `Ask` arm would cascade through every match-arm.
//!   * `crate::permissions::PermissionPromptDecision` is `{Allow, Deny}`
//!     (missing `Ask`).
//!
//! Per session-3 OPEN-QUESTIONS F-3 + Tranche-C explorer GAP-1, this
//! module mints a NEW [`PromptDecision`] at
//! `architecture::hook_prompt::PromptDecision` so the naming is local
//! and the 9+ HookVerdictKind use sites stay untouched. Downstream
//! cascades to E-24 Task 5 (pre-MCP-call guard) consume this type
//! directly.
//!
//! Wire-dark gate: [`ELAI_E25_PROMPT_HANDLER_ENABLE_VAR`]. When OFF the
//! handler short-circuits to `Allow` without invoking the runtime so the
//! sidecar / cloud Sonnet path stays cold by default.
//!
//! Emission: reuses the SHIPPED
//! [`crate::architecture::claude_code_arch::ClaudeCodeArchitectureEventKind::PromptDecisionMade`]
//! arm via the Pattern-2 single-variant SSB cascade. ZERO new
//! SsbActionType. ZERO schema bump.
//!
//! Default runtime: per CLAUDE.md remote-sidecar block and
//! Tranche-C explorer GAP-2, the canonical fallback runtime is the
//! Qwen2.5-Coder-3B-abliterated sidecar at 127.0.0.1:8100 via the
//! B-37 `PerRoleRuntime::assistant_decision` role. The runtime is
//! supplied by the caller as a `&dyn RuntimeAdapter` so this module
//! does not pull a hard dep on the orchestrator's
//! `RuntimeOrchestrator`; future plumbing (E-24 Task 5) selects the
//! correct role before calling.

use std::fmt;
use std::time::Duration;

use elai_providers::runtime::{
    CompletionConfig, Message, MessageRole, RuntimeAdapter, RuntimeAdapterError,
};

use crate::architecture::claude_code_arch::{
    ClaudeCodeArchPrimitive, ClaudeCodeArchitectureEventKind,
    ClaudeCodeArchitectureEventPayload,
};

/// Environment-variable name controlling the wire-dark gate. Set to a
/// truthy value (`"1"` / `"true"` / `"yes"` / `"on"`) to enable.
/// Default OFF.
pub const ELAI_E25_PROMPT_HANDLER_ENABLE_VAR: &str = "ELAI_E25_PROMPT_HANDLER_ENABLE";

/// Per-call sidecar timeout for the LLM-decision invocation. Plan
/// specifies 5s; exceeding it returns
/// [`PromptHandlerError::TimedOut`].
pub const PROMPT_HANDLER_TIMEOUT: Duration = Duration::from_secs(5);

/// Max tokens budgeted for the strict-JSON decision payload. The
/// handler returns a single line of JSON; 256 tokens is generous.
pub const PROMPT_HANDLER_MAX_TOKENS: u32 = 256;

/// Canonical system prompt the handler prepends. Keeps the sidecar
/// from emitting prose -- only strict JSON is accepted by
/// [`parse_decision_payload`].
pub const PROMPT_HANDLER_SYSTEM_PROMPT: &str =
    include_str!("../prompts/hook_prompt_handler.md");

/// NEW 3-arm enum minted for E-25 prompt-handler dispatch. Distinct
/// from `crate::hook_dispatcher::HookVerdictKind` and
/// `crate::permissions::PermissionPromptDecision` -- see module-level
/// doc for the naming rationale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptDecision {
    /// LLM granted the operation.
    Allow,
    /// LLM refused the operation. The reason is intended for surfacing
    /// to the user / audit log; it is NOT machine-parsed downstream.
    Deny(String),
    /// LLM deferred to the user. The reason is shown verbatim in the
    /// follow-up reconfirmation prompt.
    Ask(String),
}

impl PromptDecision {
    /// Canonical snake_case wire label for the variant (without
    /// payload). Used for SSB `details_json.decision` + dashboard
    /// filtering.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny(_) => "deny",
            Self::Ask(_) => "ask",
        }
    }

    /// Reason text, if any. `Allow` has no reason; `Deny` and `Ask`
    /// carry one.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Allow => None,
            Self::Deny(r) | Self::Ask(r) => Some(r.as_str()),
        }
    }
}

impl fmt::Display for PromptDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allow => write!(f, "Allow"),
            Self::Deny(reason) => write!(f, "Deny({reason})"),
            Self::Ask(reason) => write!(f, "Ask({reason})"),
        }
    }
}

/// Failure modes for [`prompt_decision`]. Constructed via `thiserror`
/// per CLAUDE.md HARD RULE Anti-Stale-Training (no `Box<dyn Error>`).
#[derive(Debug, thiserror::Error)]
pub enum PromptHandlerError {
    /// The sidecar / cloud call returned an error.
    #[error("runtime adapter error: {0}")]
    Runtime(#[from] RuntimeAdapterError),
    /// The sidecar replied within the budget but the body did not
    /// parse as strict JSON. Plan-locked semantic: NO graceful fallback
    /// to `Allow` (E-25 plan L60).
    #[error("invalid decision format from runtime: {reason}")]
    InvalidDecisionFormat { reason: String },
    /// The sidecar replied with valid JSON whose `decision` field is
    /// not one of `allow` / `deny` / `ask`.
    #[error("unknown decision variant `{variant}` from runtime")]
    UnknownVariant { variant: String },
    /// The sidecar did not return within
    /// [`PROMPT_HANDLER_TIMEOUT`].
    #[error("prompt-handler runtime call timed out after {millis} ms")]
    TimedOut { millis: u64 },
    /// The wire-dark gate is OFF. Surfaced so callers can distinguish
    /// "gate off" from "gate on, decision = allow".
    #[error("prompt-handler wire-dark gate {gate} unset; short-circuited to allow")]
    GateDark { gate: &'static str },
}

/// Returns true iff the wire-dark gate is enabled.
#[must_use]
pub fn prompt_handler_enabled() -> bool {
    matches_truthy_env(ELAI_E25_PROMPT_HANDLER_ENABLE_VAR)
}

fn matches_truthy_env(name: &str) -> bool {
    match std::env::var(name) {
        Ok(v) => {
            let trimmed = v.trim();
            trimmed.eq_ignore_ascii_case("1")
                || trimmed.eq_ignore_ascii_case("true")
                || trimmed.eq_ignore_ascii_case("yes")
                || trimmed.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

/// Parse a strict-JSON decision payload returned by the runtime.
/// Public so callers / tests can exercise the parse path without
/// standing up a runtime mock.
///
/// Accepts EXACTLY:
///   `{"decision": "allow" | "deny" | "ask", "reason": "..."}`
///
/// On parse failure returns [`PromptHandlerError::InvalidDecisionFormat`].
/// On unknown variant returns [`PromptHandlerError::UnknownVariant`].
pub fn parse_decision_payload(raw: &str) -> Result<PromptDecision, PromptHandlerError> {
    let trimmed = raw.trim();
    let v: serde_json::Value =
        serde_json::from_str(trimmed).map_err(|e| PromptHandlerError::InvalidDecisionFormat {
            reason: format!("json parse: {e}"),
        })?;
    let obj = v
        .as_object()
        .ok_or_else(|| PromptHandlerError::InvalidDecisionFormat {
            reason: "top-level value is not a JSON object".to_owned(),
        })?;
    let decision_raw = obj
        .get("decision")
        .and_then(|d| d.as_str())
        .ok_or_else(|| PromptHandlerError::InvalidDecisionFormat {
            reason: "missing string field `decision`".to_owned(),
        })?;
    let reason = obj
        .get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("")
        .to_owned();
    match decision_raw {
        "allow" => Ok(PromptDecision::Allow),
        "deny" => Ok(PromptDecision::Deny(reason)),
        "ask" => Ok(PromptDecision::Ask(reason)),
        other => Err(PromptHandlerError::UnknownVariant {
            variant: other.to_owned(),
        }),
    }
}

/// Invoke the LLM-decision hook handler.
///
/// When the wire-dark gate is OFF this short-circuits to
/// [`PromptHandlerError::GateDark`] WITHOUT touching the runtime.
/// Callers in production sites should map `GateDark` to
/// [`PromptDecision::Allow`] (fail-open under the wire-dark posture);
/// this distinction is surfaced via the error type so audit logs can
/// tell apart "gate dark" from a real `Allow` decision.
///
/// When the gate is ON: builds the strict-JSON system prompt + user
/// prompt, calls `runtime.complete` under [`PROMPT_HANDLER_TIMEOUT`],
/// and parses the response via [`parse_decision_payload`].
pub async fn prompt_decision(
    prompt: &str,
    runtime: &dyn RuntimeAdapter,
) -> Result<PromptDecision, PromptHandlerError> {
    if !prompt_handler_enabled() {
        return Err(PromptHandlerError::GateDark {
            gate: ELAI_E25_PROMPT_HANDLER_ENABLE_VAR,
        });
    }
    let messages = vec![Message {
        role: MessageRole::User,
        content: prompt.to_owned(),
    }];
    let config = CompletionConfig {
        model: None,
        max_tokens: PROMPT_HANDLER_MAX_TOKENS,
        temperature: 0.0,
        seed: None,
        stop_sequences: Vec::new(),
        system: Some(PROMPT_HANDLER_SYSTEM_PROMPT.to_owned()),
        decoding_config: None,
    };
    let completion_fut = runtime.complete(messages, config);
    let response =
        match tokio::time::timeout(PROMPT_HANDLER_TIMEOUT, completion_fut).await {
            Ok(result) => result?,
            Err(_) => {
                let millis = u64::try_from(PROMPT_HANDLER_TIMEOUT.as_millis())
                    .unwrap_or(u64::MAX);
                return Err(PromptHandlerError::TimedOut { millis });
            }
        };
    parse_decision_payload(&response.text)
}

/// Build the SSB payload for an evaluated decision. Always emits (the
/// PromptDecisionMade kind covers Allow / Deny / Ask uniformly per
/// substrate doc at claude_code_arch.rs:231-233).
#[must_use]
pub fn build_decision_payload(
    decision: &PromptDecision,
    latency_ms: u64,
    session_id: Option<String>,
    ts_ms: i64,
) -> ClaudeCodeArchitectureEventPayload {
    let details_json = serde_json::json!({
        "prompt_event": "prompt_decision_made",
        "decision": decision.as_str(),
        "reason": decision.reason(),
        "latency_ms": latency_ms,
    });
    ClaudeCodeArchitectureEventPayload {
        kind: ClaudeCodeArchitectureEventKind::PromptDecisionMade,
        primitive: ClaudeCodeArchPrimitive::PromptHandler,
        session_id,
        details_json,
        ts_ms,
    }
}

/// Map the production-canonical result of [`prompt_decision`] into a
/// `PromptDecision`. Only the `GateDark` variant is downgraded to
/// `Allow` (fail-open under the wire-dark posture per the module-doc
/// at L213-216); every other `PromptHandlerError` variant is propagated
/// unchanged so audit logs preserve the failure cause.
///
/// Use this adapter at production call sites that want a single
/// `PromptDecision` instead of branching on the `GateDark` special case
/// themselves. Test code that wants to assert on the `GateDark`
/// variant should call [`prompt_decision`] directly.
///
/// Note: this adapter is intentionally NOT named `unwrap_or_allow`
/// because that would imply the `Runtime` / `InvalidDecisionFormat` /
/// `UnknownVariant` / `TimedOut` variants also map to `Allow` -- they
/// MUST NOT (fail-closed on real errors).
///
/// s-22600 Tranche G G.1: ships the adapter as audit WARN-1 followup.
/// The F.2 mcp_bridge.rs site does NOT consume this adapter (different
/// error type: `llm_guard::GuardError` has no `GateDark` variant); the
/// adapter is shipped for future `hook_prompt::prompt_decision` callers
/// (post-E-24 consolidation per DECISION-T-F-5).
pub fn to_decision_or_err(
    result: Result<PromptDecision, PromptHandlerError>,
) -> Result<PromptDecision, PromptHandlerError> {
    match result {
        Ok(d) => Ok(d),
        Err(PromptHandlerError::GateDark { .. }) => Ok(PromptDecision::Allow),
        Err(other) => Err(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_allow_no_reason() {
        let raw = r#"{"decision":"allow","reason":""}"#;
        let d = parse_decision_payload(raw).expect("parse");
        assert_eq!(d, PromptDecision::Allow);
        assert_eq!(d.as_str(), "allow");
        assert_eq!(d.reason(), None);
    }

    #[test]
    fn parse_deny_carries_reason() {
        let raw = r#"{"decision":"deny","reason":"path escapes workspace"}"#;
        let d = parse_decision_payload(raw).expect("parse");
        assert_eq!(d, PromptDecision::Deny("path escapes workspace".into()));
        assert_eq!(d.as_str(), "deny");
        assert_eq!(d.reason(), Some("path escapes workspace"));
    }

    #[test]
    fn parse_ask_carries_reason() {
        let raw = r#"{"decision":"ask","reason":"confirm overwrite"}"#;
        let d = parse_decision_payload(raw).expect("parse");
        assert_eq!(d, PromptDecision::Ask("confirm overwrite".into()));
    }

    #[test]
    fn parse_invalid_json_returns_invalid_decision_format() {
        let raw = "not json at all";
        let err = parse_decision_payload(raw).unwrap_err();
        assert!(matches!(
            err,
            PromptHandlerError::InvalidDecisionFormat { .. }
        ));
    }

    #[test]
    fn parse_missing_decision_field_returns_invalid_decision_format() {
        let raw = r#"{"reason":"x"}"#;
        let err = parse_decision_payload(raw).unwrap_err();
        assert!(matches!(
            err,
            PromptHandlerError::InvalidDecisionFormat { .. }
        ));
    }

    #[test]
    fn parse_unknown_variant_returns_unknown_variant() {
        let raw = r#"{"decision":"maybe","reason":"x"}"#;
        let err = parse_decision_payload(raw).unwrap_err();
        assert!(matches!(err, PromptHandlerError::UnknownVariant { .. }));
    }

    #[test]
    fn build_payload_uses_shipped_kind() {
        let d = PromptDecision::Allow;
        let p = build_decision_payload(&d, 12, Some("s-test".into()), 1_700_000_000_000);
        assert_eq!(p.kind, ClaudeCodeArchitectureEventKind::PromptDecisionMade);
        assert_eq!(p.primitive, ClaudeCodeArchPrimitive::PromptHandler);
        assert_eq!(p.session_id.as_deref(), Some("s-test"));
        assert_eq!(p.details_json.get("decision").and_then(|v| v.as_str()), Some("allow"));
    }

    #[test]
    fn gate_default_off() {
        // The production gate is checked by `prompt_handler_enabled()`;
        // unset env => false. Use an alternate var name to avoid races.
        assert!(!matches_truthy_env("ELAI_E25_PROMPT_HANDLER_ENABLE_TEST_UNSET"));
    }

    // s-22600 Tranche G G.1: adapter pins (audit WARN-1 followup).

    #[test]
    fn to_decision_or_err_maps_gate_dark_to_allow() {
        let err = Err::<PromptDecision, _>(PromptHandlerError::GateDark {
            gate: ELAI_E25_PROMPT_HANDLER_ENABLE_VAR,
        });
        let d = to_decision_or_err(err).expect("gate dark must downgrade to Allow");
        assert_eq!(d, PromptDecision::Allow);
    }

    #[test]
    fn to_decision_or_err_propagates_real_errors() {
        let err = Err::<PromptDecision, _>(PromptHandlerError::UnknownVariant {
            variant: "maybe".to_owned(),
        });
        let propagated = to_decision_or_err(err).unwrap_err();
        assert!(matches!(
            propagated,
            PromptHandlerError::UnknownVariant { .. }
        ));
    }

    #[test]
    fn to_decision_or_err_passes_ok_decisions_through() {
        let ok_allow = Ok(PromptDecision::Allow);
        assert_eq!(
            to_decision_or_err(ok_allow).expect("ok allow passes through"),
            PromptDecision::Allow
        );
        let ok_deny = Ok(PromptDecision::Deny("nope".to_owned()));
        let out = to_decision_or_err(ok_deny).expect("ok deny passes through");
        assert!(matches!(out, PromptDecision::Deny(_)));
    }
}
