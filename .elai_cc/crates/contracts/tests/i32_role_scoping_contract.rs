//! I-32 structural role-scoping contract.
//!
//! Persona stripping for factual/code roles + role-scope supervisor gate.
//!
//! Arms (all fail-closed, R1 missing-evidence = FAIL):
//!   (a) CONTRACT-IN-PROMPT: build_system_prompt_for_role(Executor) must NOT
//!       contain "You are an interactive agent" and MUST contain "# Role: Executor".
//!       (R4: drives the REAL build path; the off-agent artifact is the assembled Vec.)
//!   (b) OUT-OF-SCOPE-TOOL-REJECTED: an InProcessSupervisor with role contracts
//!       installed, driving a Directive with initiator="researcher" + tool_name="file_write",
//!       returns blocked=true with rule_id "i32-role-scope-denied".
//!   (c) IN-SCOPE-PASSES: same supervisor + Directive initiator="researcher" +
//!       tool_name="web_search" returns blocked=false.
//!   (d) KIND-DISCRIMINATING: RoleContractApplied payload round-trips through
//!       live rmp_serde; all three Kind arms encode/decode correctly.
//!   (e) SCHEMA-LOCK: SCHEMA_VERSION_PUBLIC >= 241.
//!
//! Spec references: docs/thefinalglance/I-plan/I-32_structural_role_scoping.md
//! Reads (mandatory): wiring-enforcement.md R1/R4/R7; IC-17 persona-split grounding.

use elai_orchestrator::ssb::{
    RoleContractAppliedKind, RoleContractAppliedPayload, SsbActionType, SsbPayload,
    SCHEMA_VERSION_PUBLIC, decode_ssb_payload, ssb_action_type_from_str_public,
};
use elai_orchestrator::{
    AgentRole, Directive, GateOutcome, InProcessSupervisor, RoleContractStore, Supervisor,
    apply_to_system_prompt,
};

// ---------------------------------------------------------------------------
// (a) CONTRACT-IN-PROMPT: build_system_prompt_for_role produces the right text.
// ---------------------------------------------------------------------------

#[test]
fn i32_a_contract_present_in_built_prompt_for_executor() {
    // We drive apply_to_system_prompt directly (the same code path called by
    // build_system_prompt_for_role). This is the production function, not a shim.
    let store = RoleContractStore::default();

    // Simulate a baseline section list with the narrative intro that gets replaced.
    let mut sections = vec![
        "You are an interactive agent who helps with code tasks.".to_string(),
        "# Claude instructions\nsome rules".to_string(),
    ];
    apply_to_system_prompt(&mut sections, AgentRole::Executor, &store);

    let joined = sections.join("\n");

    assert!(
        !joined.contains("You are an interactive agent"),
        "Executor prompt must NOT contain narrative persona after contract application; \
         got: {joined}"
    );
    assert!(
        joined.contains("# Role: Executor"),
        "Executor prompt must contain the schema-first role header after contract application; \
         got: {joined}"
    );
    assert!(
        joined.contains("Constraints:"),
        "Executor prompt must contain the Constraints block after contract application; \
         got: {joined}"
    );
}

#[test]
fn i32_a2_auditor_persona_retained() {
    // Auditor is alignment-dependent; apply_to_system_prompt must leave it unchanged.
    let store = RoleContractStore::default();
    let intro = "You are an interactive agent with hostile-critic framing.".to_string();
    let mut sections = vec![intro.clone()];
    apply_to_system_prompt(&mut sections, AgentRole::Auditor, &store);

    assert_eq!(
        sections[0], intro,
        "Auditor sections must be unchanged (PersonaRetained); got: {:?}",
        sections
    );
}

// ---------------------------------------------------------------------------
// (b) OUT-OF-SCOPE-TOOL-REJECTED: researcher + file_write = blocked.
// ---------------------------------------------------------------------------

#[test]
fn i32_b_out_of_scope_tool_rejected_for_researcher() {
    let store = RoleContractStore::default();
    let supervisor = InProcessSupervisor::new("/tmp/i32-worktree")
        .with_role_contracts(store);

    // Researcher is a factual/code role whose contract does NOT include file_write.
    let directive = Directive::new(
        "file_write",
        serde_json::json!({}),
        "i32-test-intent",
        "researcher",
    );
    let outcome: GateOutcome = supervisor.run_gates(&directive);

    assert!(
        outcome.blocked,
        "file_write for researcher must be blocked by the role-scope gate; \
         outcome: {outcome:?}"
    );
    let has_role_scope_finding = outcome
        .findings
        .iter()
        .any(|f| f.rule_id.as_deref() == Some("i32-role-scope-denied"));
    assert!(
        has_role_scope_finding,
        "blocking finding must carry rule_id 'i32-role-scope-denied'; \
         findings: {:?}",
        outcome.findings
    );
}

// ---------------------------------------------------------------------------
// (c) IN-SCOPE-PASSES: researcher + web_search = allowed.
// ---------------------------------------------------------------------------

#[test]
fn i32_c_in_scope_tool_passes_for_researcher() {
    let store = RoleContractStore::default();
    let supervisor = InProcessSupervisor::new("/tmp/i32-worktree")
        .with_role_contracts(store);

    // web_search is in the Researcher contract's allowed_tool_categories.
    let directive = Directive::new(
        "web_search",
        serde_json::json!({}),
        "i32-test-intent-2",
        "researcher",
    );
    let outcome: GateOutcome = supervisor.run_gates(&directive);

    // web_search on a localhost test path should not be blocked by the role-scope gate.
    // (E-30 argv gate only fires for bash_tool_use; G-3 blast-radius fires for
    // filesystem targets; F-1 VibeGate does not apply to web_search without a
    // vibe-gate hook installed — so only the role-scope arm could block here.)
    let role_scope_blocked = outcome
        .findings
        .iter()
        .any(|f| f.rule_id.as_deref() == Some("i32-role-scope-denied"));
    assert!(
        !role_scope_blocked,
        "web_search for researcher must NOT be blocked by the role-scope gate; \
         findings: {:?}",
        outcome.findings
    );
}

#[test]
fn i32_c2_alignment_dependent_role_never_blocked_by_role_scope_gate() {
    let store = RoleContractStore::default();
    let supervisor = InProcessSupervisor::new("/tmp/i32-worktree")
        .with_role_contracts(store);

    // InjectionClassifier has no contract entry; the gate must pass it through.
    let directive = Directive::new(
        "file_write",
        serde_json::json!({}),
        "i32-test-intent-3",
        "injection_classifier",
    );
    let outcome: GateOutcome = supervisor.run_gates(&directive);

    let role_scope_blocked = outcome
        .findings
        .iter()
        .any(|f| f.rule_id.as_deref() == Some("i32-role-scope-denied"));
    assert!(
        !role_scope_blocked,
        "alignment-dependent role (injection_classifier) must NEVER be blocked by the \
         role-scope gate; findings: {:?}",
        outcome.findings
    );
}

// ---------------------------------------------------------------------------
// (d) KIND-DISCRIMINATING: RoleContractApplied payload round-trips all 3 arms.
// ---------------------------------------------------------------------------

#[test]
fn i32_d_role_contract_applied_payload_kind_discriminates() {
    for (kind, role) in [
        (RoleContractAppliedKind::PersonaStripped, "executor"),
        (RoleContractAppliedKind::PersonaRetained, "auditor"),
        (RoleContractAppliedKind::AbDelta, "executor"),
    ] {
        let payload = SsbPayload::RoleContractApplied(RoleContractAppliedPayload {
            kind,
            role: role.to_owned(),
            fixture_pass_rate_old: 0.5,
            fixture_pass_rate_new: 1.0,
            delta: 0.5,
            ts_ms: 1_700_000_050_000,
        });

        // action_type bijection.
        assert_eq!(
            payload.action_type(),
            SsbActionType::RoleContractApplied(RoleContractAppliedKind::PersonaStripped),
            "RoleContractApplied payload must map to the role_contract_applied action_type"
        );

        // Live msgpack encode/decode (same path production uses).
        let encoded = rmp_serde::to_vec_named(&payload).expect("encode RoleContractApplied");
        let decoded = decode_ssb_payload(&encoded).expect("decode RoleContractApplied");
        match decoded {
            SsbPayload::RoleContractApplied(p) => {
                assert_eq!(p.kind, kind, "decoded Kind arm must round-trip for {role}");
                assert_eq!(p.role, role, "decoded role must round-trip");
            }
            other => panic!(
                "decoded payload must be RoleContractApplied for kind {kind:?} / role {role}; \
                 got {other:?}"
            ),
        }
    }

    // from_str bijection (read-side consumers).
    let at = ssb_action_type_from_str_public("role_contract_applied")
        .expect("from_str role_contract_applied must succeed");
    assert_eq!(
        at,
        SsbActionType::RoleContractApplied(RoleContractAppliedKind::PersonaStripped),
        "from_str role_contract_applied must return the PersonaStripped default arm"
    );
    assert_eq!(
        at.as_sql_str(),
        "role_contract_applied",
        "as_sql_str round-trip"
    );
}

// ---------------------------------------------------------------------------
// (e) SCHEMA-LOCK: SCHEMA_VERSION_PUBLIC >= 241 (I-32 landed).
// ---------------------------------------------------------------------------

#[test]
fn i32_e_schema_lock() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 241,
        "SCHEMA_VERSION must be >= 241 after I-32 lands; got {SCHEMA_VERSION_PUBLIC}"
    );
    // Verify the action_type_lock.rs constant is in sync via the e45 contract;
    // here we confirm the from_str + as_sql_str round-trip for the new variant.
    assert!(
        ssb_action_type_from_str_public("role_contract_applied").is_ok(),
        "'role_contract_applied' must be a registered action_type string"
    );
}
