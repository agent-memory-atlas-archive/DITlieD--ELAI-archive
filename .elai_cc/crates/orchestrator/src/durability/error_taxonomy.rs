//! E-44 (Tranche 7C) — tool error taxonomy.
//!
//! A regex / keyword classifier that buckets a free-form error string
//! into one of six [`ErrorClass`] arms. The classifier is intentionally
//! deterministic + stdlib-only (no LLM call) so it can run on every
//! failed dispatch without blowing the latency budget. Mirrors the
//! `.claude/tools/failure_taxonomy.py` regex classifier used by the
//! auditor side.
//!
//! Producer wiring: each [`ToolErrorClassifier::classify`] call,
//! when paired with [`emit_classification`], emits one
//! [`DurabilityEventKind::ToolErrorClassified`] row (reserved at
//! SCHEMA_VERSION 186; no further bump needed).

use crate::durability::observability::notify_tool_error_classified;
use crate::ssb::{Ssb, SsbError};

/// Coarse error class. Ordered by recovery cost (cheapest first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorClass {
    /// Network blip, DNS hiccup, transient 5xx. Retry the same op.
    TransientNetwork,
    /// 429 / Retry-After / quota exceeded. Back off and retry.
    RateLimited,
    /// 401 / 403 / token expired. Re-auth and retry.
    AuthFailure,
    /// Malformed argument / contract violation. Fix the call shape.
    InvalidInput,
    /// Tool itself crashed / unhandled exception / panic. Compensate.
    ToolBug,
    /// Did not match any heuristic. Abort + escalate.
    Unknown,
}

impl ErrorClass {
    /// Stable snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TransientNetwork => "transient_network",
            Self::RateLimited => "rate_limited",
            Self::AuthFailure => "auth_failure",
            Self::InvalidInput => "invalid_input",
            Self::ToolBug => "tool_bug",
            Self::Unknown => "unknown",
        }
    }

    /// Recommended recovery action for this class. Mirrors the 3-bucket
    /// recovery taxonomy in the E-44 plan (retry / compensate / abort).
    #[must_use]
    pub fn recovery_action(&self) -> &'static str {
        match self {
            Self::TransientNetwork | Self::RateLimited | Self::AuthFailure => "retry",
            Self::ToolBug => "compensate",
            Self::InvalidInput | Self::Unknown => "abort",
        }
    }
}

/// Heuristic classifier. Holds no state beyond the compiled keyword
/// tables (built into `classify` via simple lowercase contains checks
/// to avoid pulling in `regex` for what amounts to substring matching).
#[derive(Debug, Default, Clone, Copy)]
pub struct ToolErrorClassifier;

impl ToolErrorClassifier {
    /// Construct a classifier. Cheap; safe to call per dispatch.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Bucket the supplied error string. Case-insensitive substring
    /// match; first matching bucket wins (cheapest-recovery first).
    #[must_use]
    pub fn classify(&self, err: &str) -> ErrorClass {
        let lower = err.to_ascii_lowercase();

        // Rate-limited needs to win over generic "network" / "500" so
        // that a "429 too many requests" string is bucketed correctly.
        if contains_any(
            &lower,
            &[
                "rate limit",
                "rate-limit",
                "too many requests",
                "429",
                "retry-after",
                "quota exceeded",
            ],
        ) {
            return ErrorClass::RateLimited;
        }

        if contains_any(
            &lower,
            &[
                "401",
                "403",
                "unauthorized",
                "unauthorised",
                "forbidden",
                "auth failed",
                "auth_failure",
                "token expired",
                "invalid token",
                "permission denied",
            ],
        ) {
            return ErrorClass::AuthFailure;
        }

        if contains_any(
            &lower,
            &[
                "connection refused",
                "connection reset",
                "timed out",
                "timeout",
                "dns",
                "name resolution",
                "network is unreachable",
                "no route to host",
                "502 bad gateway",
                "503 service unavailable",
                "504 gateway",
                "tls handshake",
            ],
        ) {
            return ErrorClass::TransientNetwork;
        }

        if contains_any(
            &lower,
            &[
                "invalid argument",
                "invalid input",
                "missing field",
                "schema mismatch",
                "parse error",
                "validation failed",
                "bad request",
                "400",
                "malformed",
            ],
        ) {
            return ErrorClass::InvalidInput;
        }

        if contains_any(
            &lower,
            &[
                "panic",
                "panicked",
                "unhandled exception",
                "segmentation fault",
                "internal server error",
                "500",
                "nullpointer",
                "null pointer",
                "tool bug",
                "assertion failed",
            ],
        ) {
            return ErrorClass::ToolBug;
        }

        ErrorClass::Unknown
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}

/// Truncate the error message for SSB emission (keeps the wire row
/// small + matches the docstring contract on
/// [`crate::ssb::DurabilityEventKind::ToolErrorClassified`]).
#[must_use]
pub fn truncate_message(msg: &str, limit: usize) -> String {
    if msg.len() <= limit {
        msg.to_owned()
    } else {
        msg.chars().take(limit).collect()
    }
}

/// Emit one [`DurabilityEventKind::ToolErrorClassified`] row.
#[allow(clippy::too_many_arguments)]
pub async fn emit_classification(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    tool_id: &str,
    class: ErrorClass,
    error_message: &str,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    let truncated = truncate_message(error_message, 200);
    notify_tool_error_classified(
        ssb,
        agent_id,
        superstep,
        session_id,
        tool_id,
        class.as_str(),
        truncated.as_str(),
        class.recovery_action(),
        ts_ms,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_transient_network() {
        let c = ToolErrorClassifier::new();
        assert_eq!(c.classify("connection refused"), ErrorClass::TransientNetwork);
        assert_eq!(c.classify("dial tcp: i/o timeout"), ErrorClass::TransientNetwork);
        assert_eq!(c.classify("503 Service Unavailable"), ErrorClass::TransientNetwork);
    }

    #[test]
    fn classify_rate_limited_and_auth() {
        let c = ToolErrorClassifier::new();
        assert_eq!(c.classify("HTTP 429 too many requests"), ErrorClass::RateLimited);
        assert_eq!(c.classify("Retry-After: 60"), ErrorClass::RateLimited);
        assert_eq!(c.classify("401 Unauthorized"), ErrorClass::AuthFailure);
        assert_eq!(c.classify("403 forbidden: token expired"), ErrorClass::AuthFailure);
    }

    #[test]
    fn classify_invalid_tool_bug_unknown() {
        let c = ToolErrorClassifier::new();
        assert_eq!(c.classify("schema mismatch on field foo"), ErrorClass::InvalidInput);
        assert_eq!(c.classify("400 bad request"), ErrorClass::InvalidInput);
        assert_eq!(c.classify("thread 'main' panicked at ..."), ErrorClass::ToolBug);
        assert_eq!(c.classify("500 internal server error"), ErrorClass::ToolBug);
        assert_eq!(c.classify("the moon ate my homework"), ErrorClass::Unknown);
    }

    #[test]
    fn recovery_action_per_class() {
        assert_eq!(ErrorClass::TransientNetwork.recovery_action(), "retry");
        assert_eq!(ErrorClass::RateLimited.recovery_action(), "retry");
        assert_eq!(ErrorClass::AuthFailure.recovery_action(), "retry");
        assert_eq!(ErrorClass::ToolBug.recovery_action(), "compensate");
        assert_eq!(ErrorClass::InvalidInput.recovery_action(), "abort");
        assert_eq!(ErrorClass::Unknown.recovery_action(), "abort");
    }

    #[test]
    fn truncate_message_limit() {
        assert_eq!(truncate_message("abc", 10), "abc");
        let long = "x".repeat(300);
        assert_eq!(truncate_message(&long, 200).len(), 200);
    }
}
