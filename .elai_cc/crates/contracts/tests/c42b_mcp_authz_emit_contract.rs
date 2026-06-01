//! Tranche D (s-22600) C-42b contract -- McpAuthz emit wire-through.
//!
//! Five contract cases (Phase A + Phase B):
//!   1. `compose_with_tool_gate_emit` with `Some(emitter)` fires the
//!      composed-verdict hook EXACTLY ONCE (Phase A + Phase B baseline;
//!      mirrors the in-crate `emit_hook_fires_once_on_allow` test that
//!      lives in compose.rs).
//!   2. `compose_with_tool_gate_emit` with `None` is byte-identical to
//!      the legacy `compose_with_tool_gate` surface (back-compat).
//!   3. Naming-collision: `with_authz_ssb_emitter` (existing, 4-arg
//!      GatewayAuthzSsbEmitter) and `with_mcp_authz_ssb_emitter` (NEW,
//!      5-arg McpAuthzSsbEmitter) are DISTINCT methods on
//!      DefaultMcpBridge; both compile and chain.
//!   4. Composed emitter call survives an emit-side Err (test by
//!      construction: the trait method is fn (sync, no Result) so by
//!      definition cannot return Err. We assert the trait surface stays
//!      sync-fn so a future "make it fallible" refactor would break
//!      this test deliberately).
//!   5. McpAuthzEvent Pattern-2 reuse: the new wire path uses
//!      [`SsbActionType::McpAuthzEvent`] -- NO new SsbActionType
//!      variant was added in Tranche D. Round-trip via
//!      `ssb_action_type_from_str_public("mcp_authz_event")`.
//!
//! Schema floor: asserts SCHEMA_VERSION_PUBLIC >= 195 (no bump in
//! Tranche D per D9).
//!
//! Dual-emit semantic (D7): the bridge fires BOTH gateway_*
//! (per-layer attribution) AND mcp_authz_event (composed verdict)
//! rows per `dispatch_tool_call_with_authz` call. Asserting both fire
//! in a single integration setup requires constructing a full Ssb
//! pool + tokio runtime + production policy, which lives in the
//! contract-test suite under
//! `tests/c42_semantic_gateway_contract.rs`. This file asserts the
//! INTEGRATION POINTS -- the trait + the bridge surface -- are wired
//! correctly so the dual-emit can fire.
//!
//! NO cargo test execution by the main LLM (s-22600 mandate). Auditor
//! runs the suite under cargo test in AUDITING substate.

use std::sync::{Arc, Mutex};

use elai_orchestrator::ssb::ELAI_C42_MCP_AUTHZ_EMIT_ENABLE_ENV;
use elai_orchestrator::{
    ssb_action_type_from_str_public, SsbActionType, SCHEMA_VERSION_PUBLIC,
};
use elai_plugins::authz::{
    compose_with_tool_gate, compose_with_tool_gate_emit, AuthzPolicy, ComposedVerdict,
    McpAuthzSsbEmitter,
};

// ---------------------------------------------------------------------------
// Contract -- schema-version floor (Tranche D reuses, does NOT bump)
// ---------------------------------------------------------------------------

#[test]
fn contract_schema_version_is_195_after_tranche_d_no_bump() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 195,
        "Tranche D C-42b reuses existing McpAuthzEvent Pattern-2 and \
         does NOT bump SCHEMA_VERSION. CONTRACT: SCHEMA_VERSION_PUBLIC \
         must be >= 195."
    );
}

// ---------------------------------------------------------------------------
// Contract -- D.2 case 5 (Pattern-2 reuse confirmed): no new variant
// ---------------------------------------------------------------------------

#[test]
fn contract_mcp_authz_event_pattern2_round_trip_still_works() {
    let actual =
        ssb_action_type_from_str_public("mcp_authz_event").expect("from_str");
    assert_eq!(
        actual,
        SsbActionType::McpAuthzEvent,
        "CONTRACT: Tranche D wires the orchestrator-side emitter onto \
         the existing McpAuthzEvent Pattern-2 variant -- NO new variant. \
         If from_str(\"mcp_authz_event\") stops resolving, the substrate \
         has regressed."
    );
}

#[test]
fn contract_env_gate_const_stable() {
    assert_eq!(
        ELAI_C42_MCP_AUTHZ_EMIT_ENABLE_ENV,
        "ELAI_C42_MCP_AUTHZ_EMIT_ENABLE",
        "CONTRACT: env-var name MUST be the canonical \
         ELAI_C42_MCP_AUTHZ_EMIT_ENABLE string. The orchestrator-side \
         OrchestratorMcpAuthzSsbEmitter reads this on every emit."
    );
}

// ---------------------------------------------------------------------------
// Test fixture -- a capturing McpAuthzSsbEmitter implementation
// ---------------------------------------------------------------------------

#[derive(Default)]
struct CapturingMcpAuthzEmitter {
    rows: Mutex<Vec<(String, String, String, String, Option<String>)>>,
}

impl McpAuthzSsbEmitter for CapturingMcpAuthzEmitter {
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

const POLICY_TOML: &str = r#"
default = "deny"

[roles.planner]
tools = ["knowledge.search", "fs.read"]
doc_owners = []
"#;

fn policy() -> AuthzPolicy {
    AuthzPolicy::load_from_str(POLICY_TOML).expect("load")
}

// ---------------------------------------------------------------------------
// Contract -- D.2 case 1: emit hook fires once on allow
// ---------------------------------------------------------------------------

#[test]
fn contract_emit_hook_fires_once_on_compose_allow() {
    let em = CapturingMcpAuthzEmitter::default();
    let v = compose_with_tool_gate_emit(
        "planner",
        "knowledge.search",
        None,
        "planning",
        &policy(),
        None,
        Some(&em),
    );
    assert!(matches!(v, ComposedVerdict::Allow));
    let rows = em.rows.lock().expect("lock");
    assert_eq!(
        rows.len(),
        1,
        "CONTRACT: exactly one composed-emit row per allow decision"
    );
    assert_eq!(rows[0].3, "allow", "layer label must be 'allow'");
    assert!(rows[0].4.is_none(), "allow rows carry no deny_reason");
}

// ---------------------------------------------------------------------------
// Contract -- D.2 case 1b: emit hook fires once on authz deny
// ---------------------------------------------------------------------------

#[test]
fn contract_emit_hook_fires_once_on_compose_authz_deny() {
    let em = CapturingMcpAuthzEmitter::default();
    let v = compose_with_tool_gate_emit(
        "ghost",
        "fs.read",
        None,
        "executing",
        &policy(),
        None,
        Some(&em),
    );
    assert!(!matches!(v, ComposedVerdict::Allow));
    let rows = em.rows.lock().expect("lock");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].3, "authz", "deny layer must be 'authz'");
    assert!(
        rows[0].4.as_deref().unwrap_or("").contains("role-not-declared"),
        "deny_reason must carry the role-not-declared signal"
    );
}

// ---------------------------------------------------------------------------
// Contract -- D.2 case 2: None emitter is byte-identical to legacy surface
// ---------------------------------------------------------------------------

#[test]
fn contract_none_emitter_matches_legacy_compose() {
    // Same inputs, two different surfaces (legacy vs emit-with-None);
    // verdict must be identical so callers can swap without observing
    // behaviour change.
    let v_legacy = compose_with_tool_gate(
        "planner",
        "knowledge.search",
        None,
        "planning",
        &policy(),
        None,
    );
    let v_emit_none = compose_with_tool_gate_emit(
        "planner",
        "knowledge.search",
        None,
        "planning",
        &policy(),
        None,
        None,
    );
    assert_eq!(
        v_legacy, v_emit_none,
        "CONTRACT: compose_with_tool_gate_emit with None emitter MUST \
         be byte-identical to the legacy compose_with_tool_gate"
    );
}

// ---------------------------------------------------------------------------
// Contract -- D.2 case 3: naming collision avoidance
// (with_authz_ssb_emitter vs with_mcp_authz_ssb_emitter are distinct
// methods on DefaultMcpBridge; both compile and chain).
// ---------------------------------------------------------------------------

#[test]
fn contract_naming_collision_distinct_builder_methods() {
    // Static type-level check: we Arc-cast both trait objects so the
    // builder chain compiles. This test would FAIL TO COMPILE if the
    // two builders were merged into one method with the same name.
    use elai_plugins::mcp_bridge::DefaultMcpBridge;
    use elai_plugins::mcp_bridge::NoopGatewayAuthzSsbEmitter;

    struct NullMcpAuthz;
    impl McpAuthzSsbEmitter for NullMcpAuthz {
        fn emit_authz_decision(
            &self,
            _tool: &str,
            _fsm_state: &str,
            _actor: &str,
            _layer: &str,
            _deny_reason: Option<&str>,
        ) {
            // No-op for the collision-detection test.
        }
    }

    let _bridge = DefaultMcpBridge::default()
        .with_authz_ssb_emitter(Arc::new(NoopGatewayAuthzSsbEmitter))
        .with_mcp_authz_ssb_emitter(Arc::new(NullMcpAuthz));
    // Reaching this point means both builders exist with the expected
    // signatures. Contract satisfied.
}

// ---------------------------------------------------------------------------
// Contract -- D.2 case 4: emitter call non-fatal (trait is sync fn, no Result)
// ---------------------------------------------------------------------------

#[test]
fn contract_emit_authz_decision_signature_is_sync_fn_no_result() {
    // Compile-time check: if a future refactor changes the trait method
    // signature to return Result<_, _> or add an async marker, this
    // line stops compiling (the closure binds the exact signature).
    // The contract "the bridge can call emit_authz_decision without
    // worrying about a fail-stop" depends on the sync-fn-returning-unit
    // shape; breaking it requires a deliberate update to this test.
    fn _typecheck(em: &dyn McpAuthzSsbEmitter) {
        let _: () = em.emit_authz_decision("t", "s", "a", "l", None);
    }
    let em = CapturingMcpAuthzEmitter::default();
    _typecheck(&em);
}
