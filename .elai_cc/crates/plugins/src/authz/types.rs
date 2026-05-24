//! C-42 (session 2) authorization types.
//!
//! All structs here are deterministic value types (no I/O, no async). The
//! [`AuthzVerdict`] enum is binary per arXiv:2604.25555 Section 4 (verdict
//! in {allow, deny}); there is no Quarantine or Audit variant.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Binary RBAC verdict, paper-faithful (arXiv:2604.25555 Section 4:
/// "default_policy_value == False" + "verdict in {allow, deny}").
///
/// `Deny` carries a structured reason so downstream SSB rows (the
/// `GatewayPolicyDenied` action type) can attribute the rejection without
/// re-running the policy check. The reason string is plain-text suitable
/// for operator-facing dashboards; it MUST NOT carry secrets (role tokens,
/// session ids) because SSB rows are subscribed by the dashboard panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthzVerdict {
    /// Role is authorized to invoke the tool in this document context.
    /// L1 (semantic firewall) and L3 (crypto-HITL) are out of scope for
    /// the L2-only C-42 session 2 — callers MUST NOT interpret `Allow`
    /// as a full 3-layer-passed verdict. The composed gate only asserts
    /// "RBAC permits this tool for this role".
    Allow,
    /// Role denied. `reason` is a short human-readable phrase
    /// ("role 'planner' not permitted tool 'file_write'", "default-deny
    /// policy: role 'unknown' not declared in policy"). Producers MUST
    /// keep reasons stable for downstream regex-grep workflows.
    Deny {
        /// Stable human-readable reason. Stable wording so SSB dashboards
        /// can group identical denials. Do not template variable
        /// identifiers into this string without a hash — operators read
        /// these raw.
        reason: String,
    },
}

impl AuthzVerdict {
    /// Returns true when the verdict is [`AuthzVerdict::Allow`].
    pub fn is_allow(&self) -> bool {
        matches!(self, Self::Allow)
    }

    /// Returns the deny reason if the verdict is [`AuthzVerdict::Deny`].
    pub fn deny_reason(&self) -> Option<&str> {
        match self {
            Self::Deny { reason } => Some(reason.as_str()),
            Self::Allow => None,
        }
    }
}

/// Document-context tuple supplied by the caller. Paper L2 (arXiv:2604.25555
/// Section 5) treats ownership / department as a BOLA-defense input:
/// **ownership comes from the authoritative DB, not from agent claims**.
///
/// This struct carries `db_owner` (the field the DB-of-record returns) and
/// `agent_claimed_owner` (the field the agent purports to own). The
/// [`crate::authz::checker::check_authz`] function ignores
/// `agent_claimed_owner` and only consults `db_owner`. The agent claim is
/// kept on the struct purely so callers (and tests) can demonstrate the
/// claim was supplied and ignored — never read it for an authorization
/// decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocContext {
    /// Stable id of the document under access (DB primary key, file path,
    /// or other route identifier the policy can match against).
    pub doc_id: String,
    /// Authoritative owner from the DB-of-record. `None` means the doc
    /// has no recorded owner (treated as public for read intents; deny
    /// for write intents — see [`AuthzPolicy`] rules).
    pub db_owner: Option<String>,
    /// Field the agent claims as the owner. ALWAYS ignored by the checker
    /// (BOLA defense, paper Section 5). Kept on the struct so the fuzz
    /// invariant can verify it was supplied and discarded.
    pub agent_claimed_owner: Option<String>,
}

impl DocContext {
    /// Public construction helper. Forces callers to think about BOLA at
    /// the type level — `db_owner` is the load-bearing field, the
    /// `agent_claimed_owner` is structural-only.
    pub fn new(
        doc_id: impl Into<String>,
        db_owner: Option<String>,
        agent_claimed_owner: Option<String>,
    ) -> Self {
        Self {
            doc_id: doc_id.into(),
            db_owner,
            agent_claimed_owner,
        }
    }
}

/// Role claim presented by the calling agent. Paper L2 (arXiv:2604.25555
/// Section 5) treats the role as coming from a **signed token**, not from
/// agent-emitted JSON. C-42 session 2 ships the type but token-signature
/// validation is deferred (no Crypto-HITL yet — L3 is followup scope).
///
/// In the L2-only path, the role string is trusted-by-caller — the bridge
/// caller (CLI, dashboard, internal task runner) is responsible for
/// supplying the right role. The fuzz harness exercises both correct
/// caller behavior and adversarial role-spoofing attempts (which deny
/// because the spoofed role is not in the policy).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleClaim {
    /// Role name. Lowercase snake-case; one of the keys declared in
    /// `authz-policy.toml` `[roles.*]`. Unknown roles always deny.
    pub role: String,
}

impl RoleClaim {
    pub fn new(role: impl Into<String>) -> Self {
        Self { role: role.into() }
    }
}

/// Error type for authz policy I/O + parse paths. Runtime check_authz
/// itself never errors (it returns [`AuthzVerdict`] directly); errors here
/// surface during [`crate::authz::AuthzPolicy::load`] or
/// [`crate::authz::AuthzPolicy::load_from_str`].
#[derive(Debug, Error)]
pub enum AuthzError {
    #[error("authz-policy.toml read error: {0}")]
    Io(#[from] std::io::Error),
    #[error("authz-policy.toml parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error(
        "authz-policy.toml default = {default:?} is invalid; must be \"allow\" or \"deny\". \
         deny-all is the paper-grounded recommendation (arXiv:2604.25555 Section 4)."
    )]
    InvalidDefault { default: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_verdict_is_allow() {
        assert!(AuthzVerdict::Allow.is_allow());
        assert!(AuthzVerdict::Allow.deny_reason().is_none());
    }

    #[test]
    fn deny_verdict_carries_reason() {
        let v = AuthzVerdict::Deny {
            reason: "test reason".to_string(),
        };
        assert!(!v.is_allow());
        assert_eq!(v.deny_reason(), Some("test reason"));
    }

    #[test]
    fn doc_context_keeps_agent_claim_but_does_not_use_it() {
        let ctx = DocContext::new(
            "doc-42",
            Some("alice".to_string()),
            Some("eve".to_string()),
        );
        // The struct stores both; the checker MUST only read db_owner.
        // This test asserts the surface contract; checker.rs has the
        // behavioral test.
        assert_eq!(ctx.db_owner.as_deref(), Some("alice"));
        assert_eq!(ctx.agent_claimed_owner.as_deref(), Some("eve"));
    }
}
