//! J-78 -- context-access spec contract (source-wiring gate).
//!
//! Verifies the structural claims from the J-78 plan for the Tier-1
//! OBSERVE-ONLY access-spec slice:
//!
//! D1: `access_spec` module exists in elai_orchestrator and is declared in
//!     lib.rs (module reachable from the crate root).
//! D2: `compute_access_spec_for_agent_role` + `emit_context_access_spec_declared`
//!     + `emit_out_of_spec_read_flagged` + `flag_out_of_spec_read` are public
//!     production API (callable from outside the orchestrator crate).
//! D3: Auditor role AccessSpec denies memory (composes H-31 firewall).
//! D4: Executor role AccessSpec allows the `user_memory` namespace.
//! D5: `flag_out_of_spec_read` returns true for a namespace not in the spec.
//! D6: `ContextAccessSpecKind` has the three expected variants
//!     (SpecDeclared, OutOfSpecReadFlagged, SpecEnforced).
//! D7: Production caller exists in task_runner.rs (non-test call site).

use std::fs;
use std::path::PathBuf;

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/contracts has a parent")
        .to_path_buf()
}

fn read_src(rel: &str) -> String {
    let path = crates_dir().join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// D1: access_spec module exists + declared in lib.rs
// ---------------------------------------------------------------------------

#[test]
fn d1a_access_spec_module_exists() {
    let src = read_src("orchestrator/src/access_spec.rs");
    assert!(
        src.contains("pub struct AccessSpec"),
        "access_spec.rs must export AccessSpec"
    );
}

#[test]
fn d1b_access_spec_declared_in_lib() {
    let src = read_src("orchestrator/src/lib.rs");
    assert!(
        src.contains("pub mod access_spec"),
        "lib.rs must declare pub mod access_spec"
    );
}

// ---------------------------------------------------------------------------
// D2: compute/emit symbols are public production API
// ---------------------------------------------------------------------------

#[test]
fn d2_compute_and_emit_symbols_are_public() {
    let src = read_src("orchestrator/src/access_spec.rs");
    assert!(
        src.contains("pub fn compute_access_spec_for_agent_role"),
        "compute_access_spec_for_agent_role must be pub"
    );
    assert!(
        src.contains("pub async fn emit_context_access_spec_declared"),
        "emit_context_access_spec_declared must be pub async"
    );
    assert!(
        src.contains("pub async fn emit_out_of_spec_read_flagged"),
        "emit_out_of_spec_read_flagged must be pub async"
    );
    assert!(
        src.contains("pub fn flag_out_of_spec_read"),
        "flag_out_of_spec_read must be pub"
    );
    assert!(
        src.contains("pub fn filter_memory_allowed"),
        "filter_memory_allowed must be pub"
    );
}

// ---------------------------------------------------------------------------
// D3: Auditor role AccessSpec denies memory (composes H-31)
// ---------------------------------------------------------------------------

#[test]
fn d3_auditor_denies_memory() {
    use elai_orchestrator::access_spec::compute_access_spec_for_agent_role;
    use elai_ipc_types::AgentRole;

    let spec = compute_access_spec_for_agent_role(AgentRole::Auditor);
    assert!(
        spec.allowed_memory_namespaces.is_empty(),
        "Auditor must have ZERO memory namespaces (H-31 firewall deny)"
    );

    let sc = compute_access_spec_for_agent_role(AgentRole::IdeaCritic);
    let sc_empty = sc.allowed_memory_namespaces.is_empty();
    assert!(
        sc_empty,
        "IdeaCritic must have ZERO memory namespaces"
    );

    let si = compute_access_spec_for_agent_role(AgentRole::InjectionClassifier);
    let si_empty = si.allowed_memory_namespaces.is_empty();
    assert!(
        si_empty,
        "InjectionClassifier must have ZERO memory namespaces"
    );
}

// ---------------------------------------------------------------------------
// D4: Executor role AccessSpec allows user_memory namespace
// ---------------------------------------------------------------------------

#[test]
fn d4_executor_allows_user_memory() {
    use elai_orchestrator::access_spec::compute_access_spec_for_agent_role;
    use elai_ipc_types::AgentRole;

    let spec = compute_access_spec_for_agent_role(AgentRole::Executor);
    assert!(
        spec.allowed_memory_namespaces.contains(&"user_memory".to_string()),
        "Executor must allow the user_memory namespace"
    );
}

// ---------------------------------------------------------------------------
// D5: flag_out_of_spec_read returns true for a namespace not in the spec
// ---------------------------------------------------------------------------

#[test]
fn d5_out_of_spec_flag() {
    use elai_orchestrator::access_spec::{
        compute_access_spec_for_agent_role, flag_out_of_spec_read,
    };
    use elai_ipc_types::AgentRole;

    let auditor_spec = compute_access_spec_for_agent_role(AgentRole::Auditor);
    assert!(
        flag_out_of_spec_read(&auditor_spec, "user_memory"),
        "Auditor requesting user_memory must be flagged out-of-spec"
    );

    let executor_spec = compute_access_spec_for_agent_role(AgentRole::Executor);
    assert!(
        !flag_out_of_spec_read(&executor_spec, "user_memory"),
        "Executor requesting user_memory must NOT be flagged"
    );
}

// ---------------------------------------------------------------------------
// D6: ContextAccessSpecKind has the three expected variants
// ---------------------------------------------------------------------------

#[test]
fn d6_context_access_spec_kind_variants() {
    use elai_orchestrator::ssb::ContextAccessSpecKind;

    // Verify all three variants exist and are distinct.
    let declared = ContextAccessSpecKind::SpecDeclared;
    let flagged = ContextAccessSpecKind::OutOfSpecReadFlagged;
    let enforced = ContextAccessSpecKind::SpecEnforced;

    assert_ne!(declared, flagged);
    assert_ne!(declared, enforced);
    assert_ne!(flagged, enforced);
}

// ---------------------------------------------------------------------------
// D7: Production caller exists in task_runner.rs (non-test)
// ---------------------------------------------------------------------------

#[test]
fn d7_production_caller_in_task_runner() {
    let src = read_src("orchestrator/src/task_runner.rs");
    assert!(
        src.contains("crate::access_spec::compute_access_spec_for_agent_role"),
        "task_runner.rs must call compute_access_spec_for_agent_role in production"
    );
    assert!(
        src.contains("crate::access_spec::emit_context_access_spec_declared"),
        "task_runner.rs must call emit_context_access_spec_declared in production"
    );
}

// ---------------------------------------------------------------------------
// D8: SSB action type + payload wired (Pattern-2 flat variant)
// ---------------------------------------------------------------------------

#[test]
fn d8_ssb_action_type_wired() {
    let action_type_src = read_src("orchestrator/src/ssb/action_type.rs");
    assert!(
        action_type_src.contains("ContextAccessSpec"),
        "SsbActionType must have a ContextAccessSpec variant"
    );
    assert!(
        action_type_src.contains("\"context_access_spec\""),
        "ContextAccessSpec as_sql_str must return \"context_access_spec\""
    );

    let payloads_src = read_src("orchestrator/src/ssb/payloads.rs");
    assert!(
        payloads_src.contains("ContextAccessSpecPayload"),
        "SsbPayload must have a ContextAccessSpec arm"
    );
    assert!(
        payloads_src.contains("pub enum ContextAccessSpecKind"),
        "ContextAccessSpecKind enum must exist"
    );

    let schema_src = read_src("orchestrator/src/ssb/schema.rs");
    assert!(
        schema_src.contains("context_access_spec"),
        "SQL CHECK constraint must include 'context_access_spec'"
    );

    let ssb_src = read_src("orchestrator/src/ssb.rs");
    assert!(
        ssb_src.contains("SsbActionType::ContextAccessSpec"),
        "SSB_ACTION_TYPES must include ContextAccessSpec"
    );
}
