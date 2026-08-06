//! J-78 (context-access spec): OBSERVE-ONLY access-spec composition for agent
//! context assembly.
//!
//! Composes the H-31 Personalization-Trap firewall
//! ([`crate::user_memory_firewall::agent_role_gets_user_memory`]) into a
//! declarative [`AccessSpec`] that is emitted as an
//! [`SsbActionType::ContextAccessSpec`] row BEFORE context injection. Tier-1
//! is OBSERVE-ONLY: the spec is declared and out-of-spec reads are flagged,
//! but nothing is hard-enforced (no [`ContextAccessSpecKind::SpecEnforced`]
//! emission).
//!
//! The firewall match itself is NOT re-authored here —
//! [`filter_memory_allowed`] delegates to the existing compile-time allowlist.

use std::sync::Arc;

use elai_core::clock::ClockProvider;
use elai_ipc_types::AgentRole;

use crate::ssb::{
    ContextAccessSpecKind, ContextAccessSpecPayload, Ssb, SsbPayload,
};
use crate::task_runner::{RUNNER_AGENT_ID, RUNNER_SUPERSTEP};

/// Declarative access spec for a single agent role: which SSB row ids,
/// sibling output ids, and memory namespaces the role is allowed to read.
///
/// Tier-1 populates only `allowed_memory_namespaces` (the H-31 firewall
/// surface). `allowed_ssb_row_ids` and `allowed_sibling_output_ids` are
/// empty in Tier-1 (reserved for later enforcement slices).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessSpec {
    /// SSB row ids the role is allowed to read (Tier-1: empty — reserved).
    pub allowed_ssb_row_ids: Vec<String>,
    /// Sibling output ids the role is allowed to read (Tier-1: empty — reserved).
    pub allowed_sibling_output_ids: Vec<String>,
    /// Memory namespaces the role is allowed to read. Populated from the
    /// H-31 firewall: `["user_memory"]` for Executor, empty for all others.
    pub allowed_memory_namespaces: Vec<String>,
    /// Human-readable role label for the emitted payload.
    pub role_label: String,
}

/// Compute the [`AccessSpec`] for an agent role.
///
/// * Auditor family + IdeaCritic + InjectionClassifier: empty memory
///   namespaces (H-31 declared deny case).
/// * Executor: memory namespace `["user_memory"]` allowed.
/// * Other non-prompt-assembling roles: minimal (empty memory).
///
/// This does NOT re-author the firewall match — it delegates to
/// [`filter_memory_allowed`] which calls the H-31 compile-time allowlist.
#[must_use]
pub fn compute_access_spec_for_agent_role(role: AgentRole) -> AccessSpec {
    let role_label = format!("{role:?}");
    let spec_empty = AccessSpec {
        allowed_ssb_row_ids: Vec::new(),
        allowed_sibling_output_ids: Vec::new(),
        allowed_memory_namespaces: Vec::new(),
        role_label: role_label.clone(),
    };
    let allowed_memory_namespaces = if filter_memory_allowed(&spec_empty, role) {
        vec!["user_memory".to_string()]
    } else {
        Vec::new()
    };
    AccessSpec {
        allowed_ssb_row_ids: Vec::new(),
        allowed_sibling_output_ids: Vec::new(),
        allowed_memory_namespaces,
        role_label,
    }
}

/// Delegate to the H-31 firewall to decide whether `role` is allowed
/// user-memory. Does NOT re-author the match — calls
/// [`crate::user_memory_firewall::agent_role_gets_user_memory`].
#[must_use]
pub fn filter_memory_allowed(_spec: &AccessSpec, role: AgentRole) -> bool {
    crate::user_memory_firewall::agent_role_gets_user_memory(role)
}

/// OBSERVE-ONLY: returns `true` if `requested_kind` is NOT in the spec's
/// allowed memory namespaces. The caller flags this via
/// [`emit_out_of_spec_read_flagged`] but does NOT hard-block.
#[must_use]
pub fn flag_out_of_spec_read(spec: &AccessSpec, requested_kind: &str) -> bool {
    !spec.allowed_memory_namespaces.iter().any(|ns| ns == requested_kind)
}

/// Emit a [`ContextAccessSpecKind::SpecDeclared`] OBSERVE-ONLY row.
///
/// `observe_only` MUST be `true` in Tier-1 (no hard enforcement).
pub async fn emit_context_access_spec_declared(
    ssb: &Ssb,
    spec: &AccessSpec,
    observe_only: bool,
    clock: &Arc<dyn ClockProvider>,
) {
    let payload = SsbPayload::ContextAccessSpec(ContextAccessSpecPayload {
        kind: ContextAccessSpecKind::SpecDeclared,
        role: spec.role_label.clone(),
        allowed_ssb_row_ids: spec.allowed_ssb_row_ids.clone(),
        allowed_sibling_output_ids: spec.allowed_sibling_output_ids.clone(),
        allowed_memory_namespaces: spec.allowed_memory_namespaces.clone(),
        observe_only,
        out_of_spec_flagged: Vec::new(),
        ts_ms: i64::try_from(clock.now_ms()).unwrap_or(0),
    });
    if let Err(e) = ssb
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        tracing::warn!(error = %e, "access_spec: emit SpecDeclared failed");
    }
}

/// Emit a [`ContextAccessSpecKind::OutOfSpecReadFlagged`] OBSERVE-ONLY row
/// recording that `requested` was read but is not in the spec.
pub async fn emit_out_of_spec_read_flagged(
    ssb: &Ssb,
    spec: &AccessSpec,
    requested: &str,
    clock: &Arc<dyn ClockProvider>,
) {
    let payload = SsbPayload::ContextAccessSpec(ContextAccessSpecPayload {
        kind: ContextAccessSpecKind::OutOfSpecReadFlagged,
        role: spec.role_label.clone(),
        allowed_ssb_row_ids: spec.allowed_ssb_row_ids.clone(),
        allowed_sibling_output_ids: spec.allowed_sibling_output_ids.clone(),
        allowed_memory_namespaces: spec.allowed_memory_namespaces.clone(),
        observe_only: true,
        out_of_spec_flagged: vec![requested.to_string()],
        ts_ms: i64::try_from(clock.now_ms()).unwrap_or(0),
    });
    if let Err(e) = ssb
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        tracing::warn!(error = %e, "access_spec: emit OutOfSpecReadFlagged failed");
    }
}

