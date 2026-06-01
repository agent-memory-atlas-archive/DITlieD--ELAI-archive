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

/// Tranche B-6 quad C-42 SHIPPABLE-NOW -- per-call SSB emit hook.
///
/// The caller-side runtime guard. Implementations route to
/// `crates/orchestrator/src/authz_ssb_adapter.rs` (or equivalent) which
/// translates the tuple into an [`McpAuthzEvent`] SSB row gated by
/// `ELAI_C42_MCP_AUTHZ_EMIT_ENABLE`. `None` is the default (plugins-only
/// builds, tests, the offline fuzz harness).
///
/// Defense-in-depth boundary: this trait keeps the SSB-emit responsibility
/// on the CALLER side of [`check_authz`], preserving the documented purity
/// of the checker function (see `checker.rs:1-9`). Same pattern as the
/// existing `CaseBaseSsbEmitter` (C-47) and `ToolGateSsbEmitter` (C-35).
///
/// Proc-macro attribute `#[authz(states="...", actors="...")]` declarative
/// surface is DEFERRED to C-42b -- the runtime guard achieves the same
/// outcome without the first-proc-macro friction.
pub trait McpAuthzSsbEmitter: Send + Sync {
    /// Emit one [`McpAuthzEvent`] SSB row for the just-decided composed
    /// verdict. `layer` is one of `"allow"` / `"authz"` / `"tool_gate"`
    /// (mirrors [`ComposedVerdict::rejecting_layer_label`]).
    /// `deny_reason` is `None` on allow rows.
    fn emit_authz_decision(
        &self,
        tool: &str,
        fsm_state: &str,
        actor: &str,
        layer: &str,
        deny_reason: Option<&str>,
    );
}

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
    compose_with_tool_gate_emit(role, tool, doc_context, fsm_state, policy, gate, None)
}

/// Tranche B-6 quad C-42 SHIPPABLE-NOW -- composed gate with optional
/// per-call SSB emit.
///
/// Identical decision logic to [`compose_with_tool_gate`] but threads an
/// optional [`McpAuthzSsbEmitter`] through. When `Some(emitter)`, the
/// composed verdict is mirrored into an [`McpAuthzEvent`] SSB row
/// (Kind::AuthzCheckPassed on Allow, Kind::AuthzCheckFailed on either
/// deny). When `None`, behaviour is byte-identical to the legacy
/// [`compose_with_tool_gate`] surface -- preserves zero-overhead for
/// plugins-only builds and the offline fuzz harness.
///
/// Wire-dark unless the orchestrator-side adapter is configured with
/// `ELAI_C42_MCP_AUTHZ_EMIT_ENABLE=1`; the emitter itself is the env-gate
/// site (this function is purely a router).
pub fn compose_with_tool_gate_emit(
    role: &str,
    tool: &str,
    doc_context: Option<&DocContext>,
    fsm_state: &str,
    policy: &AuthzPolicy,
    gate: Option<&ToolGate>,
    emitter: Option<&dyn McpAuthzSsbEmitter>,
) -> ComposedVerdict {
    let authz = check_authz(role, tool, doc_context, policy);
    let verdict = match authz {
        AuthzVerdict::Deny { reason } => ComposedVerdict::DeniedByAuthz { reason },
        AuthzVerdict::Allow => match gate {
            None => ComposedVerdict::Allow,
            Some(gate) => match gate.check_tool_allowed(fsm_state, tool) {
                Ok(()) => ComposedVerdict::Allow,
                Err(e) => ComposedVerdict::DeniedByToolGate {
                    reason: e.to_string(),
                },
            },
        },
    };
    if let Some(em) = emitter {
        em.emit_authz_decision(
            tool,
            fsm_state,
            role,
            verdict.rejecting_layer_label(),
            verdict.deny_reason(),
        );
    }
    verdict
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

    // === Tranche B-6 quad C-42 SHIPPABLE-NOW: emitter-hook coverage ===

    use std::sync::Mutex;

    struct CapturingEmitter {
        rows: Mutex<Vec<(String, String, String, String, Option<String>)>>,
    }

    impl CapturingEmitter {
        fn new() -> Self {
            Self { rows: Mutex::new(Vec::new()) }
        }
    }

    impl McpAuthzSsbEmitter for CapturingEmitter {
        fn emit_authz_decision(
            &self,
            tool: &str,
            fsm_state: &str,
            actor: &str,
            layer: &str,
            deny_reason: Option<&str>,
        ) {
            let mut g = self.rows.lock().expect("lock");
            g.push((
                tool.to_string(),
                fsm_state.to_string(),
                actor.to_string(),
                layer.to_string(),
                deny_reason.map(|s| s.to_string()),
            ));
        }
    }

    #[test]
    fn emit_hook_fires_once_on_allow() {
        let em = CapturingEmitter::new();
        let v = compose_with_tool_gate_emit(
            "planner",
            "knowledge.search",
            None,
            "planning",
            &policy(),
            None,
            Some(&em),
        );
        assert!(v.is_allow());
        let rows = em.rows.lock().expect("lock");
        assert_eq!(rows.len(), 1, "exactly one emit per composed decision");
        assert_eq!(rows[0].3, "allow");
        assert!(rows[0].4.is_none(), "allow rows carry no deny_reason");
    }

    #[test]
    fn emit_hook_fires_once_on_authz_deny() {
        let em = CapturingEmitter::new();
        let v = compose_with_tool_gate_emit(
            "ghost",
            "fs.read",
            None,
            "executing",
            &policy(),
            None,
            Some(&em),
        );
        assert!(!v.is_allow());
        let rows = em.rows.lock().expect("lock");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].3, "authz");
        assert!(rows[0].4.as_deref().unwrap_or("").contains("role-not-declared"));
    }

    #[test]
    fn legacy_wrapper_is_emit_free() {
        // The original `compose_with_tool_gate` surface stays byte-identical
        // for callers that don't opt into the SSB emit hook.
        let v = compose_with_tool_gate(
            "planner",
            "knowledge.search",
            None,
            "planning",
            &policy(),
            None,
        );
        assert!(v.is_allow());
    }
}
