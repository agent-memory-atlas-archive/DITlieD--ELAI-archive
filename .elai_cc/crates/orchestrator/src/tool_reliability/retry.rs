//! E-18 — Retry escalation policy.
//!
//! Pure decision function over a `(error_kind, attempt)` pair. Returns
//! one of:
//!
//!   * `Retry` — caller should re-dispatch ONCE with an error-embedded
//!     reprompt (plan §4, arxiv:2509.18847 Structured Reflection). The
//!     plan's "NEVER retry with the same prompt" rule is enforced by
//!     the caller, not this module (we just say "another attempt is
//!     still allowed").
//!   * `Escalate { to_role }` — retry budget exhausted; hand off to a
//!     different agent role. We pick `Auditor` for ELAI because the
//!     project's AgentRole enum (elai_ipc_types) does NOT carry a
//!     `Fixer` variant. The Auditor cycles the audit pass which is the
//!     closest semantic to "human in the loop diagnoses why".
//!   * `GiveUp` — non-retryable error (auth, schema, syntax). The
//!     dispatcher records the failure and returns the error.
//!
//! Anti-stale: we do NOT model RetryDecision via `Box<dyn Error>`. The
//! `error_kind` is a leaf enum the caller chooses based on the actual
//! `McpError` / `ToolDispatchError` variant.

use elai_ipc_types::AgentRole;
use serde::{Deserialize, Serialize};

/// Classification of the dispatcher-observed error. Kept narrow on
/// purpose — callers map their concrete error enum into one of these
/// before consulting [`should_escalate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolErrorKind {
    /// Sidecar returned malformed JSON / Hermes envelope / fence-block
    /// (E-35 SearchReplaceMatcher refused). Retryable with an error-
    /// embedded reprompt.
    MalformedOutput,
    /// Sidecar / cloud provider transient error (HTTP 5xx, connection
    /// reset, timeout). Retryable.
    Transient,
    /// Schema or signature mismatch — the tool refused the args at
    /// dispatch time. Retryable ONCE with the schema error embedded.
    SchemaMismatch,
    /// Auth / authz / permission failure — never retryable; hand off
    /// (the operator must fix the policy).
    AuthzFailure,
    /// Hard syntax error (e.g. invalid Rust in an Edit). Retryable
    /// once with the compiler message embedded.
    SyntaxError,
    /// The downstream tool reported an internal failure (e.g. cargo
    /// build red). Not retryable — escalate.
    DownstreamFailure,
}

/// Verdict from [`should_escalate`]. Carries the next-attempt count
/// for the `Retry` arm so the caller can plumb it into the next
/// dispatch without a separate +1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetryDecision {
    /// Caller may dispatch again. `next_attempt` is the value to pass
    /// to the gate on the next call (so the gate can in turn refuse
    /// further retries when the budget is up).
    Retry { next_attempt: u32 },
    /// Caller must escalate to a different role. `to_role` is the
    /// destination AgentRole.
    Escalate { to_role: AgentRole },
    /// Caller must surface the error and stop trying.
    GiveUp,
}

/// Default retry budget — matches plan §4 "one-shot retry with
/// error-embedded reprompt".
pub const DEFAULT_RETRY_CAP: u32 = 1;

/// Decide what the dispatcher should do next for `(error_kind, attempt)`.
/// `attempt` is the number of attempts ALREADY made (so 0 = "we just
/// failed once"). `cap` is the retry budget; we surface `Escalate` when
/// `attempt >= cap` and the error is otherwise retryable.
pub fn should_escalate(error_kind: ToolErrorKind, attempt: u32, cap: u32) -> RetryDecision {
    match error_kind {
        // Non-retryable: bail immediately.
        ToolErrorKind::AuthzFailure => RetryDecision::GiveUp,

        // Downstream failure escalates without a retry — we don't burn
        // a second attempt on something the agent can't fix.
        ToolErrorKind::DownstreamFailure => RetryDecision::Escalate {
            to_role: AgentRole::Auditor,
        },

        // Retryable family — one shot then escalate.
        ToolErrorKind::MalformedOutput
        | ToolErrorKind::Transient
        | ToolErrorKind::SchemaMismatch
        | ToolErrorKind::SyntaxError => {
            if attempt >= cap {
                RetryDecision::Escalate {
                    to_role: AgentRole::Auditor,
                }
            } else {
                RetryDecision::Retry {
                    next_attempt: attempt + 1,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_output_retries_once_then_escalates() {
        // attempt=0: first failure, expect Retry with next_attempt=1.
        match should_escalate(ToolErrorKind::MalformedOutput, 0, DEFAULT_RETRY_CAP) {
            RetryDecision::Retry { next_attempt } => assert_eq!(next_attempt, 1),
            other => panic!("expected Retry(1), got {other:?}"),
        }
        // attempt=1: budget exhausted, expect Escalate to Auditor.
        match should_escalate(ToolErrorKind::MalformedOutput, 1, DEFAULT_RETRY_CAP) {
            RetryDecision::Escalate { to_role } => assert_eq!(to_role, AgentRole::Auditor),
            other => panic!("expected Escalate, got {other:?}"),
        }
    }

    #[test]
    fn transient_failure_follows_same_pattern() {
        match should_escalate(ToolErrorKind::Transient, 0, DEFAULT_RETRY_CAP) {
            RetryDecision::Retry { .. } => {}
            other => panic!("expected Retry for Transient, got {other:?}"),
        }
        match should_escalate(ToolErrorKind::Transient, 1, DEFAULT_RETRY_CAP) {
            RetryDecision::Escalate { .. } => {}
            other => panic!("expected Escalate for Transient over cap, got {other:?}"),
        }
    }

    #[test]
    fn authz_failure_gives_up_immediately() {
        match should_escalate(ToolErrorKind::AuthzFailure, 0, DEFAULT_RETRY_CAP) {
            RetryDecision::GiveUp => {}
            other => panic!("expected GiveUp for AuthzFailure, got {other:?}"),
        }
    }

    #[test]
    fn downstream_failure_skips_retry_and_escalates() {
        match should_escalate(ToolErrorKind::DownstreamFailure, 0, DEFAULT_RETRY_CAP) {
            RetryDecision::Escalate { to_role } => assert_eq!(to_role, AgentRole::Auditor),
            other => panic!("expected Escalate for DownstreamFailure, got {other:?}"),
        }
    }

    #[test]
    fn schema_mismatch_retries_under_cap() {
        match should_escalate(ToolErrorKind::SchemaMismatch, 0, 2) {
            RetryDecision::Retry { next_attempt } => assert_eq!(next_attempt, 1),
            other => panic!("expected Retry for SchemaMismatch, got {other:?}"),
        }
        match should_escalate(ToolErrorKind::SchemaMismatch, 2, 2) {
            RetryDecision::Escalate { .. } => {}
            other => panic!("expected Escalate at cap, got {other:?}"),
        }
    }

    #[test]
    fn syntax_error_retries_then_escalates() {
        match should_escalate(ToolErrorKind::SyntaxError, 0, DEFAULT_RETRY_CAP) {
            RetryDecision::Retry { next_attempt } => assert_eq!(next_attempt, 1),
            other => panic!("expected Retry for SyntaxError, got {other:?}"),
        }
    }

    #[test]
    fn default_cap_is_one() {
        assert_eq!(DEFAULT_RETRY_CAP, 1);
    }

    #[test]
    fn retry_decision_serde_roundtrip() {
        let v = RetryDecision::Escalate {
            to_role: AgentRole::Auditor,
        };
        let s = serde_json::to_string(&v).expect("serialize");
        let back: RetryDecision = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(v, back);
    }
}
