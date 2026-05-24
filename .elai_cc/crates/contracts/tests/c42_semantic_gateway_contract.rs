//! C-42 (session 2) — Semantic Gateway zero-trust authorization contract.
//!
//! Paper grounding: arXiv:2604.25555 "From CRUD to Autonomous Agents:
//! Formal Validation and Zero-Trust Security for Semantic Gateways in
//! AI-Native Enterprise Systems" — Ignacio Peyrano, MEDIUM reliability
//! per `workflow/papers/semantic-gateway-fuzzer.md` (single-author
//! arXiv preprint, no peer-review label, eval is on one domain). DO NOT
//! cite the paper's 100% / 84.2% / 5.3x numbers as validated benchmarks
//! — those are design targets only.
//!
//! Paper-vs-ELAI design boundary (read FIRST before reading invariants):
//!   - The 3-layer architecture (L1 Semantic Firewall, L2 OPA/Rego RBAC,
//!     L3 Crypto-HITL) is PAPER-faithful.
//!   - Binary verdict ({allow, deny}) is PAPER-faithful (Section 4).
//!   - BOLA defense (agent claims ignored, DB-of-record consulted) is
//!     PAPER-faithful (Section 5).
//!   - SSB event names (`gateway_policy_allowed` / `gateway_policy_denied` /
//!     `gateway_hitl_pending`) are ELAI DESIGN — the paper does not
//!     define an event-bus contract.
//!   - Composition order (AuthzPolicy BEFORE C-35 ToolGate, short-circuit
//!     on deny) is ELAI DESIGN — paper does not address composition with
//!     prior gates (Option B per paper-interface "Compose-with-existing-
//!     ToolGate").
//!   - Fuzz sequence budget (256 in-process) is ELAI scope — paper's
//!     500K sequence run is RUNPOD-PENDING (`gateway_fuzz_500k_eval`).
//!   - L1 DeBERTa-v3 classifier is RUNPOD-PENDING (`gateway_l1_deberta_
//!     v3_eval`); L3 Crypto-HITL is DEFERRED-TO-FOLLOWUP.
//!
//! This contract test locks the wire surface for the 3 SSB variants, the
//! AuthzPolicy default-deny + role-table + BOLA + sentinel rules, and the
//! ComposedVerdict layer-attribution. Compile-only (no live MCP bridge).

#![cfg(feature = "mcp")]

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, GatewayHitlPendingPayload,
    GatewayPolicyAllowedPayload, GatewayPolicyDeniedPayload,
};
use elai_orchestrator::{SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC};

use elai_plugins::authz::{
    check_authz, compose_with_tool_gate, AuthzPolicy, AuthzVerdict, ComposedVerdict,
    DocContext,
};
use elai_plugins::mcp::gate::{ToolGate, ToolGateConfig, ToolGateStateConfig};

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// (E1) Default deny — policy with default = "deny" rejects undeclared
//      role/tool combos.
// ---------------------------------------------------------------------------

const POLICY_TOML: &str = r#"
default = "deny"

[roles.planner]
tools = ["knowledge.search", "fs.read"]
doc_owners = []

[roles.executor]
tools = ["knowledge.search", "fs.write", "fs.edit"]
doc_owners = ["self"]

[roles.admin]
tools = ["*"]
doc_owners = []
"#;

fn policy() -> AuthzPolicy {
    AuthzPolicy::load_from_str(POLICY_TOML).expect("policy parses")
}

#[test]
fn c42_e1_default_is_deny_paper_grounded() {
    let policy = policy();
    assert_eq!(
        policy.default_disposition(),
        "deny",
        "paper-grounded default per arXiv:2604.25555 Section 4 (deny-all)",
    );
    assert!(!policy.is_default_allow());
}

// ---------------------------------------------------------------------------
// (E2) Unlisted role/tool combo denies — covers the "deny-by-default"
//      structural invariant from paper Section 4.
// ---------------------------------------------------------------------------

#[test]
fn c42_e2_unlisted_role_tool_combo_denies() {
    let policy = policy();
    // ghost role not declared -> deny.
    let v = check_authz("ghost", "fs.read", None, &policy);
    assert!(!v.is_allow());
    assert!(v.deny_reason().unwrap().contains("role-not-declared"));

    // declared role but missing tool -> deny.
    let v = check_authz("planner", "fs.write", None, &policy);
    assert!(!v.is_allow());
    assert!(v
        .deny_reason()
        .unwrap()
        .contains("role-not-permitted-tool"));
}

// ---------------------------------------------------------------------------
// (E3) Listed role/tool combo allows.
// ---------------------------------------------------------------------------

#[test]
fn c42_e3_listed_role_tool_combo_allows() {
    let policy = policy();
    let v = check_authz("planner", "knowledge.search", None, &policy);
    assert!(matches!(v, AuthzVerdict::Allow));
}

// ---------------------------------------------------------------------------
// (E4) BOLA defense — agent_claimed_owner is IGNORED. The checker only
//      consults DocContext::db_owner. Paper Section 5 invariant.
// ---------------------------------------------------------------------------

#[test]
fn c42_e4_bola_agent_claim_ignored() {
    let policy = policy();
    // executor.doc_owners = ["self"] -> db_owner must equal "executor".
    // Agent claims to be "executor" but DB says "alice" -> deny.
    let ctx = DocContext::new(
        "doc-42",
        Some("alice".to_string()),
        Some("executor".to_string()),
    );
    let v = check_authz("executor", "fs.write", Some(&ctx), &policy);
    assert!(
        !v.is_allow(),
        "BOLA: agent_claimed_owner promoted verdict — paper Section 5 violated",
    );
    assert!(v
        .deny_reason()
        .unwrap()
        .contains("role-not-permitted-owner"));

    // Sanity: same call without the agent claim must produce the same
    // verdict (claim-vs-no-claim does not change the decision).
    let ctx_no_claim = DocContext::new("doc-42", Some("alice".to_string()), None);
    let v2 = check_authz("executor", "fs.write", Some(&ctx_no_claim), &policy);
    assert_eq!(v, v2, "BOLA invariant: agent claim must not influence verdict");
}

// ---------------------------------------------------------------------------
// (E5) Three new SsbActionType::Gateway* variants round-trip through
//      as_sql_str + ssb_action_type_from_str_public.
// ---------------------------------------------------------------------------

#[test]
fn c42_e5_gateway_policy_allowed_round_trips() {
    let tag = SsbActionType::GatewayPolicyAllowed.as_sql_str();
    assert_eq!(tag, "gateway_policy_allowed");
    let decoded = ssb_action_type_from_str_public(tag).expect("known tag");
    assert_eq!(decoded, SsbActionType::GatewayPolicyAllowed);
}

#[test]
fn c42_e5_gateway_policy_denied_round_trips() {
    let tag = SsbActionType::GatewayPolicyDenied.as_sql_str();
    assert_eq!(tag, "gateway_policy_denied");
    let decoded = ssb_action_type_from_str_public(tag).expect("known tag");
    assert_eq!(decoded, SsbActionType::GatewayPolicyDenied);
}

#[test]
fn c42_e5_gateway_hitl_pending_round_trips() {
    let tag = SsbActionType::GatewayHitlPending.as_sql_str();
    assert_eq!(tag, "gateway_hitl_pending");
    let decoded = ssb_action_type_from_str_public(tag).expect("known tag");
    assert_eq!(decoded, SsbActionType::GatewayHitlPending);
}

#[test]
fn c42_e5_gateway_payloads_action_type_matches() {
    // Round-trip via SsbPayload::action_type().
    let allowed = SsbPayload::GatewayPolicyAllowed(GatewayPolicyAllowedPayload {
        role_hash: 0x1,
        tool_hash: 0x2,
        deny_reason_label: "allow".into(),
        ts_ms: 1_700_000_000_000,
    });
    assert_eq!(allowed.action_type(), SsbActionType::GatewayPolicyAllowed);

    let denied = SsbPayload::GatewayPolicyDenied(GatewayPolicyDeniedPayload {
        role_hash: 0x1,
        tool_hash: 0x2,
        deny_reason_label: "authz".into(),
        deny_reason_hash: 0x3,
        ts_ms: 1_700_000_000_000,
    });
    assert_eq!(denied.action_type(), SsbActionType::GatewayPolicyDenied);

    let pending = SsbPayload::GatewayHitlPending(GatewayHitlPendingPayload {
        role_hash: 0x1,
        tool_hash: 0x2,
        challenge_hash: 0,
        ts_ms: 1_700_000_000_000,
    });
    assert_eq!(pending.action_type(), SsbActionType::GatewayHitlPending);
}

#[test]
fn c42_e5_schema_version_at_least_113() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 113,
        "C-42 expects SCHEMA_VERSION >= 113, got {}",
        SCHEMA_VERSION_PUBLIC,
    );
}

// ---------------------------------------------------------------------------
// (E6) Composition order: AuthzPolicy runs BEFORE C-35 ToolGate. When
//      AuthzPolicy denies, the gate is never consulted (the gate could
//      allow but the composed verdict still denies and attributes
//      "authz").
// ---------------------------------------------------------------------------

#[test]
fn c42_e6_authz_runs_before_tool_gate() {
    // Build a gate that allows EVERY tool in EVERY state.
    let mut states = HashMap::new();
    states.insert(
        "planning".to_string(),
        ToolGateStateConfig {
            eager: vec!["fs.write".into()],
            gated: vec![],
            denied: vec![],
        },
    );
    let gate = ToolGate::from_config(ToolGateConfig {
        states,
        max_eager_schemas: 16,
    });

    // planner does NOT have fs.write in its tool list -> authz denies.
    // The gate would have allowed (fs.write is in eager) but compose must
    // short-circuit on authz and attribute the denial to "authz".
    let v = compose_with_tool_gate(
        "planner",
        "fs.write",
        None,
        "planning",
        &policy(),
        Some(&gate),
    );
    assert!(!v.is_allow(), "authz must deny even when gate would allow");
    assert_eq!(
        v.rejecting_layer_label(),
        "authz",
        "composed verdict must attribute the rejection to authz (paper Section 5 layer field)",
    );
}

#[test]
fn c42_e6_tool_gate_runs_after_authz_allow() {
    // Build a gate that DENIES every tool in "planning" state.
    let mut states = HashMap::new();
    states.insert(
        "planning".to_string(),
        ToolGateStateConfig {
            eager: vec![],
            gated: vec![],
            denied: vec!["knowledge.search".into()],
        },
    );
    let gate = ToolGate::from_config(ToolGateConfig {
        states,
        max_eager_schemas: 16,
    });
    // planner IS permitted to use knowledge.search by authz, but the gate
    // denies it for this FSM state. Composed verdict must attribute to
    // tool_gate.
    let v = compose_with_tool_gate(
        "planner",
        "knowledge.search",
        None,
        "planning",
        &policy(),
        Some(&gate),
    );
    assert!(!v.is_allow());
    assert_eq!(v.rejecting_layer_label(), "tool_gate");
}

#[test]
fn c42_e6_both_allow_composes_to_allow() {
    let mut states = HashMap::new();
    states.insert(
        "planning".to_string(),
        ToolGateStateConfig {
            eager: vec!["knowledge.search".into()],
            gated: vec![],
            denied: vec![],
        },
    );
    let gate = ToolGate::from_config(ToolGateConfig {
        states,
        max_eager_schemas: 16,
    });
    let v = compose_with_tool_gate(
        "planner",
        "knowledge.search",
        None,
        "planning",
        &policy(),
        Some(&gate),
    );
    assert_eq!(v, ComposedVerdict::Allow);
}

// ---------------------------------------------------------------------------
// (E7) Fuzz harness presence — assert the module compiles + the
//      reference-policy parse path works at minimum invocation budget.
//      The full proptest-state-machine run lives under
//      `cargo test -p elai-plugins authz::fuzz` (test-only module). Here
//      we only verify the harness's policy fixture parses and the
//      AuthzPolicy public surface is intact.
// ---------------------------------------------------------------------------

#[test]
fn c42_e7_fuzz_reference_policy_parses() {
    // Smoke-check the same policy shape the fuzz harness uses.
    let toml = r#"
default = "deny"

[roles.planner]
tools = ["knowledge.search"]
doc_owners = []
"#;
    let p = AuthzPolicy::load_from_str(toml).expect("fuzz reference policy parses");
    assert!(!p.is_default_allow());
    let v = check_authz("planner", "knowledge.search", None, &p);
    assert!(v.is_allow());
}

// ---------------------------------------------------------------------------
// (E8) Wildcard tool sentinel — admin role with "*" permits any tool.
//      Catches accidental wildcard-handling regressions.
// ---------------------------------------------------------------------------

#[test]
fn c42_e8_admin_wildcard_permits_any_tool() {
    let policy = policy();
    let v = check_authz("admin", "irreversible.format_disk", None, &policy);
    assert!(
        v.is_allow(),
        "wildcard tool sentinel '*' must permit any tool name",
    );
}
