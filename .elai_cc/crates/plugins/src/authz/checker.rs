//! C-42 (session 2) — pure-function authorization check.
//!
//! [`check_authz`] is the canonical entry point. It is deterministic,
//! synchronous, and side-effect-free: same inputs always produce the same
//! verdict. SSB emission is deliberately NOT done inside this function —
//! the caller (the MCP bridge wiring) emits the
//! `GatewayPolicyAllowed`/`GatewayPolicyDenied` row after consulting the
//! verdict, so the policy module stays free of orchestrator dependencies
//! (and the fuzz harness can exercise it without booting the SSB writer).

use super::policy::AuthzPolicy;
use super::types::{AuthzVerdict, DocContext};

/// Run the full L2 RBAC check for `(role, tool, doc_context)`.
///
/// Decision order:
///   1. Unknown role -> Deny with reason `role-not-declared`.
///   2. Role exists but tool not declared (and no wildcard) -> Deny with
///      reason `role-not-permitted-tool`.
///   3. Role permits tool but ownership constraint fails -> Deny with
///      reason `role-not-permitted-owner`.
///   4. All checks pass -> Allow.
///
/// BOLA defense: when `doc_context` carries an `agent_claimed_owner`, the
/// field is ignored entirely. Only `db_owner` (the authoritative DB-of-
/// record field) is consulted. See arXiv:2604.25555 Section 5.
pub fn check_authz(
    role: &str,
    tool: &str,
    doc_context: Option<&DocContext>,
    policy: &AuthzPolicy,
) -> AuthzVerdict {
    if policy.role(role).is_none() {
        return AuthzVerdict::Deny {
            reason: format!("role-not-declared: role '{role}' not in authz-policy.toml"),
        };
    }
    if !policy.is_role_tool_permitted(role, tool) {
        return AuthzVerdict::Deny {
            reason: format!(
                "role-not-permitted-tool: role '{role}' not granted tool '{tool}'"
            ),
        };
    }
    let db_owner = doc_context.and_then(|c| c.db_owner.as_deref());
    if !policy.is_role_owner_permitted(role, db_owner) {
        let owner_label = db_owner.unwrap_or("<none>");
        return AuthzVerdict::Deny {
            reason: format!(
                "role-not-permitted-owner: role '{role}' not granted access to docs owned by '{owner_label}'"
            ),
        };
    }
    AuthzVerdict::Allow
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY_TOML: &str = r#"
default = "deny"

[roles.planner]
tools = ["knowledge.search", "fs.read"]
doc_owners = []

[roles.executor]
tools = ["knowledge.search", "fs.write", "fs.edit"]
doc_owners = ["self"]
"#;

    fn policy() -> AuthzPolicy {
        AuthzPolicy::load_from_str(POLICY_TOML).expect("load")
    }

    #[test]
    fn unknown_role_denies() {
        let v = check_authz("ghost", "fs.read", None, &policy());
        assert!(!v.is_allow());
        assert!(v.deny_reason().unwrap().contains("role-not-declared"));
    }

    #[test]
    fn known_role_unlisted_tool_denies() {
        let v = check_authz("planner", "fs.write", None, &policy());
        assert!(!v.is_allow());
        assert!(v
            .deny_reason()
            .unwrap()
            .contains("role-not-permitted-tool"));
    }

    #[test]
    fn known_role_listed_tool_allows() {
        let v = check_authz("planner", "knowledge.search", None, &policy());
        assert!(v.is_allow());
    }

    #[test]
    fn bola_agent_claim_is_ignored() {
        // Agent claims to own the doc but DB says different owner. Self-
        // sentinel rule on executor demands db_owner == role. The agent
        // claim of "executor" must NOT promote the verdict to allow.
        let ctx = DocContext::new(
            "doc-42",
            Some("alice".to_string()), // DB authoritative
            Some("executor".to_string()), // agent-claimed (BOLA attempt)
        );
        let v = check_authz("executor", "fs.write", Some(&ctx), &policy());
        assert!(!v.is_allow(), "BOLA: agent claim was honoured -- ungrounded");
        assert!(v
            .deny_reason()
            .unwrap()
            .contains("role-not-permitted-owner"));
    }

    #[test]
    fn self_sentinel_allows_when_db_owner_matches() {
        let ctx = DocContext::new("doc-42", Some("executor".to_string()), None);
        let v = check_authz("executor", "fs.write", Some(&ctx), &policy());
        assert!(v.is_allow());
    }

    #[test]
    fn missing_doc_context_with_constraint_denies() {
        // executor.doc_owners = ["self"]. No DocContext provided -> deny.
        let v = check_authz("executor", "fs.write", None, &policy());
        assert!(!v.is_allow());
        assert!(v
            .deny_reason()
            .unwrap()
            .contains("role-not-permitted-owner"));
    }

    #[test]
    fn deny_all_blank_policy_denies_everything() {
        let policy = AuthzPolicy::deny_all();
        let v = check_authz("planner", "knowledge.search", None, &policy);
        assert!(!v.is_allow());
    }
}
