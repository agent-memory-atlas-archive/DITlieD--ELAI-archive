//! C-42 (session 2) — composed gating order: AuthzPolicy BEFORE ToolGate.
//!
//! ELAI design decision (paper arXiv:2604.25555 does not specify
//! composition with prior gates — see workflow/papers/semantic-gateway-
//! fuzzer.md "Compose-with-existing-ToolGate"). We pick **Option B
//! (compose)** from the paper interface: AuthzPolicy (RBAC, paper L2) runs
//! first; the C-35 ToolGate (FSM-state allowlist) runs second; both must
//! allow.
//!
//! Rationale (recorded here, not in the paper):
//!   - AuthzPolicy is a per-role policy (planner cannot run shell ever),
//!     ToolGate is a per-FSM-state allowlist (no tool can run shell during
//!     AUDITING). The two questions are independent.
//!   - Running AuthzPolicy first short-circuits role-denial cases without
//!     touching the gate, which keeps fuzz inputs deterministic (the gate
//!     manifest can be absent and AuthzPolicy still denies cleanly).
//!   - The composed verdict carries which layer denied so SSB consumers
//!     can attribute the rejection (paper Section 5: "reject_reason
//!     populated on deny, includes which layer rejected").

use super::checker::check_authz;
use super::policy::AuthzPolicy;
use super::types::{AuthzVerdict, DocContext};
use crate::mcp::gate::ToolGate;

/// Three-state composed verdict carrying the rejecting layer when denied.
/// The string variants are stable wire labels suitable for SSB row
/// attribution (`gateway_policy_denied.deny_reason_hash` source).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposedVerdict {
    /// Both layers allow the call.
    Allow,
    /// AuthzPolicy (paper L2) denied. Carries the underlying verdict's
    /// reason string for downstream attribution.
    DeniedByAuthz { reason: String },
    /// AuthzPolicy allowed but ToolGate (C-35 FSM-state allowlist) denied.
    /// Carries the gate's denial reason (tool name + fsm_state).
    DeniedByToolGate { reason: String },
}

impl ComposedVerdict {
    pub fn is_allow(&self) -> bool {
        matches!(self, Self::Allow)
    }

    /// Stable layer-attribution label suitable for SSB payload fields
    /// ("authz" / "tool_gate" / "allow"). Mirrors paper Section 5's
    /// "reject_reason layer" wire field.
    pub fn rejecting_layer_label(&self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::DeniedByAuthz { .. } => "authz",
            Self::DeniedByToolGate { .. } => "tool_gate",
        }
    }

    pub fn deny_reason(&self) -> Option<&str> {
        match self {
            Self::Allow => None,
            Self::DeniedByAuthz { reason } | Self::DeniedByToolGate { reason } => {
                Some(reason.as_str())
            }
        }
    }
}

/// Run the composed gate in order: AuthzPolicy first, then ToolGate.
///
/// AuthzPolicy short-circuits — if it denies, the ToolGate is never
/// consulted. This is the canonical composition order; do NOT reverse it
/// without updating the contract test + the design-decision comment
/// above.
///
/// `fsm_state` is the lowercase FSM state string the gate is configured
/// against (matches the gate manifest's `[gate.<state>]` table key).
pub fn compose_with_tool_gate(
    role: &str,
    tool: &str,
    doc_context: Option<&DocContext>,
    fsm_state: &str,
    policy: &AuthzPolicy,
    gate: Option<&ToolGate>,
) -> ComposedVerdict {
    let authz = check_authz(role, tool, doc_context, policy);
    match authz {
        AuthzVerdict::Deny { reason } => {
            return ComposedVerdict::DeniedByAuthz { reason };
        }
        AuthzVerdict::Allow => {}
    }
    let Some(gate) = gate else {
        // No gate wired — authz alone decided. Allow.
        return ComposedVerdict::Allow;
    };
    if let Err(e) = gate.check_tool_allowed(fsm_state, tool) {
        return ComposedVerdict::DeniedByToolGate {
            reason: e.to_string(),
        };
    }
    ComposedVerdict::Allow
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY_TOML: &str = r#"
default = "deny"

[roles.planner]
tools = ["knowledge.search", "fs.read"]
doc_owners = []
"#;

    fn policy() -> AuthzPolicy {
        AuthzPolicy::load_from_str(POLICY_TOML).expect("load")
    }

    #[test]
    fn authz_deny_short_circuits_before_gate() {
        // Pass None for the gate. Even if the gate were not None, the
        // role-not-declared check fires first.
        let v = compose_with_tool_gate(
            "ghost",
            "fs.read",
            None,
            "executing",
            &policy(),
            None,
        );
        assert!(!v.is_allow());
        assert_eq!(v.rejecting_layer_label(), "authz");
        assert!(v.deny_reason().unwrap().contains("role-not-declared"));
    }

    #[test]
    fn authz_allow_with_no_gate_returns_allow() {
        let v = compose_with_tool_gate(
            "planner",
            "knowledge.search",
            None,
            "planning",
            &policy(),
            None,
        );
        assert!(v.is_allow());
        assert_eq!(v.rejecting_layer_label(), "allow");
    }

    // Gate-aware composition is exercised in the contract test
    // (crates/contracts/tests/c42_semantic_gateway_contract.rs) which has
    // the production ToolGate fixture available.
}
