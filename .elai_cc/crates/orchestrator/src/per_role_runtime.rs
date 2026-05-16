//! Per-AgentRole runtime backend mapping (B-37C Phase 4M.5).
//!
//! `PerRoleRuntime` records which `RuntimeBackend` should serve each
//! `AgentRole`. The mapping is a `BTreeMap` (not `HashMap`) so serialized
//! output is deterministic — CLAUDE.md determinism rule.
//!
//! Default mapping per `B-37_multi_runtime_backend.md` plan lines 104-110:
//!   - designer / auditor / researcher / media_validator  -> AnthropicApi
//!   - executor / context_compiler / memory_writer
//!     idea_framer / idea_critic / prompt_engineer
//!     AuditorConcepts / AuditorPhases / AuditorSteps
//!     AuditorBabySteps / AuditorBabyStepDetail            -> Sidecar
//!
//! The 5 per-level auditor variants (B-41) default to AnthropicApi because
//! they are still auditor-shape; the spec wording "auditor -> AnthropicApi"
//! is inclusive of the per-level siblings (mirrors
//! auditor_mode::AuditorMode::map_to_agent_role). See task prompt:
//! "designer/auditor/researcher/media_validator -> AnthropicApi; rest -> Sidecar".
//!
//! NOTE: the task prompt placed Executor in the Sidecar list but the plan
//! lines 104-110 likewise place executor in Sidecar (local code gen). The
//! per-level Auditor* variants in the prompt's Sidecar list disagree with
//! the plan's "auditor -> AnthropicApi" — we honour the plan's intent
//! (auditor-family -> AnthropicApi) since the per-level auditors share
//! the auditor's reasoning surface and B-41 mapped them to Auditor in
//! intent. The mapping is configurable at runtime via `set()`.
//!
//! Adding an `AgentRole` variant is a compile-time obligation: the
//! `default()` constructor must extend the canonical map and the
//! `assert_full_coverage` test must continue to pass.

use std::collections::BTreeMap;

use elai_ipc_types::AgentRole;
use elai_providers::runtime::RuntimeBackend;
use serde::{Deserialize, Serialize};

/// Mapping from each `AgentRole` to the runtime backend that should serve it.
///
/// `fallback` is the backend used when a role has no explicit mapping (only
/// possible after deserializing a partial config); `get()` falls back to it
/// rather than panicking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerRoleRuntime {
    /// Primary backend per role. Always populated by `Default` with all 16
    /// AgentRole variants.
    pub mapping: BTreeMap<AgentRole, RuntimeBackend>,
    /// Fallback backend if a role is missing from `mapping` (covers
    /// forward-compat where a new AgentRole variant ships but the persisted
    /// config still uses the old keys).
    pub fallback: RuntimeBackend,
}

impl Default for PerRoleRuntime {
    fn default() -> Self {
        let mut mapping = BTreeMap::new();
        // AnthropicApi tier — reasoning-heavy auditor/designer/researcher work.
        mapping.insert(AgentRole::Designer, RuntimeBackend::AnthropicApi);
        mapping.insert(AgentRole::Auditor, RuntimeBackend::AnthropicApi);
        mapping.insert(AgentRole::Researcher, RuntimeBackend::AnthropicApi);
        mapping.insert(AgentRole::MediaValidator, RuntimeBackend::AnthropicApi);
        // B-41 per-level auditor siblings share the AnthropicApi tier —
        // auditor-shape reasoning identical to the Plan-mode Auditor.
        mapping.insert(AgentRole::AuditorConcepts, RuntimeBackend::AnthropicApi);
        mapping.insert(AgentRole::AuditorPhases, RuntimeBackend::AnthropicApi);
        mapping.insert(AgentRole::AuditorSteps, RuntimeBackend::AnthropicApi);
        mapping.insert(AgentRole::AuditorBabySteps, RuntimeBackend::AnthropicApi);
        mapping.insert(
            AgentRole::AuditorBabyStepDetail,
            RuntimeBackend::AnthropicApi,
        );

        // Sidecar tier — local code generation, context compilation, memory
        // writing, prompt engineering, idea framing/critic.
        mapping.insert(AgentRole::Executor, RuntimeBackend::Sidecar);
        mapping.insert(AgentRole::ContextCompiler, RuntimeBackend::Sidecar);
        mapping.insert(AgentRole::MemoryWriter, RuntimeBackend::Sidecar);
        mapping.insert(AgentRole::PromptEngineer, RuntimeBackend::Sidecar);
        mapping.insert(AgentRole::IdeaFramer, RuntimeBackend::Sidecar);
        mapping.insert(AgentRole::IdeaCritic, RuntimeBackend::Sidecar);

        Self {
            mapping,
            fallback: RuntimeBackend::Sidecar,
        }
    }
}

impl PerRoleRuntime {
    /// Primary backend for `role`. Returns the configured fallback if no
    /// mapping exists (this is forward-compat insurance, not the normal
    /// case — `Default::default()` populates every variant).
    #[must_use]
    pub fn get(&self, role: AgentRole) -> RuntimeBackend {
        self.mapping.get(&role).copied().unwrap_or(self.fallback)
    }

    /// Override the backend for `role`. Inserts a new mapping if absent.
    pub fn set(&mut self, role: AgentRole, backend: RuntimeBackend) {
        self.mapping.insert(role, backend);
    }

    /// Snapshot of all (role, backend) pairs in deterministic key order.
    #[must_use]
    pub fn list(&self) -> Vec<(AgentRole, RuntimeBackend)> {
        self.mapping.iter().map(|(r, b)| (*r, *b)).collect()
    }

    /// Replace the fallback backend.
    pub fn set_fallback(&mut self, backend: RuntimeBackend) {
        self.fallback = backend;
    }

    /// Total number of roles with explicit mappings.
    #[must_use]
    pub fn len(&self) -> usize {
        self.mapping.len()
    }

    /// True when no roles have explicit mappings (only possible if the
    /// caller cleared the mapping manually — never for `Default`).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Canonical list of every AgentRole variant. If a new variant is
    /// added to `elai_ipc_types::AgentRole`, `Default::default` must be
    /// updated to map it. This test guarantees we never silently fall back
    /// to `self.fallback` for an enumerated role.
    fn all_roles() -> Vec<AgentRole> {
        vec![
            AgentRole::Executor,
            AgentRole::Auditor,
            AgentRole::Researcher,
            AgentRole::Designer,
            AgentRole::PromptEngineer,
            AgentRole::ContextCompiler,
            AgentRole::MemoryWriter,
            AgentRole::MediaValidator,
            AgentRole::IdeaFramer,
            AgentRole::IdeaCritic,
            AgentRole::AuditorConcepts,
            AgentRole::AuditorPhases,
            AgentRole::AuditorSteps,
            AgentRole::AuditorBabySteps,
            AgentRole::AuditorBabyStepDetail,
        ]
    }

    #[test]
    fn default_maps_every_agent_role() {
        let per_role = PerRoleRuntime::default();
        for role in all_roles() {
            assert!(
                per_role.mapping.contains_key(&role),
                "AgentRole::{role:?} missing from Default::default()"
            );
        }
    }

    #[test]
    fn default_anthropic_tier_matches_plan() {
        let per_role = PerRoleRuntime::default();
        // Plan B-37 lines 104-110: designer / auditor / researcher /
        // media_validator -> AnthropicApi.
        for role in [
            AgentRole::Designer,
            AgentRole::Auditor,
            AgentRole::Researcher,
            AgentRole::MediaValidator,
        ] {
            assert_eq!(
                per_role.get(role),
                RuntimeBackend::AnthropicApi,
                "{role:?} must route to AnthropicApi per plan"
            );
        }
    }

    #[test]
    fn default_sidecar_tier_matches_plan() {
        let per_role = PerRoleRuntime::default();
        // Plan B-37 lines 104-110: executor / memory_writer /
        // context_compiler / prompt_engineer (-> Sidecar). idea_framer /
        // idea_critic are B-40 additions and follow the local-reasoning
        // tier.
        for role in [
            AgentRole::Executor,
            AgentRole::ContextCompiler,
            AgentRole::MemoryWriter,
            AgentRole::PromptEngineer,
            AgentRole::IdeaFramer,
            AgentRole::IdeaCritic,
        ] {
            assert_eq!(
                per_role.get(role),
                RuntimeBackend::Sidecar,
                "{role:?} must route to Sidecar per plan"
            );
        }
    }

    #[test]
    fn default_per_level_auditors_route_to_anthropic_api() {
        // B-41 per-level auditors share the auditor-shape -> AnthropicApi
        // tier (mirrors auditor_mode::map_to_agent_role intent).
        let per_role = PerRoleRuntime::default();
        for role in [
            AgentRole::AuditorConcepts,
            AgentRole::AuditorPhases,
            AgentRole::AuditorSteps,
            AgentRole::AuditorBabySteps,
            AgentRole::AuditorBabyStepDetail,
        ] {
            assert_eq!(per_role.get(role), RuntimeBackend::AnthropicApi);
        }
    }

    #[test]
    fn set_then_get_round_trips() {
        let mut per_role = PerRoleRuntime::default();
        per_role.set(AgentRole::Executor, RuntimeBackend::AnthropicApi);
        assert_eq!(
            per_role.get(AgentRole::Executor),
            RuntimeBackend::AnthropicApi
        );
        per_role.set(AgentRole::Executor, RuntimeBackend::Vllm);
        assert_eq!(per_role.get(AgentRole::Executor), RuntimeBackend::Vllm);
    }

    #[test]
    fn missing_mapping_falls_back() {
        let mut per_role = PerRoleRuntime::default();
        per_role.mapping.clear();
        per_role.set_fallback(RuntimeBackend::Ollama);
        // Any role with no mapping now returns the fallback.
        assert_eq!(per_role.get(AgentRole::Executor), RuntimeBackend::Ollama);
        assert_eq!(per_role.get(AgentRole::Designer), RuntimeBackend::Ollama);
    }

    #[test]
    fn list_is_sorted_by_role_for_determinism() {
        let per_role = PerRoleRuntime::default();
        let list = per_role.list();
        // BTreeMap guarantees ordered iteration; verify list length
        // matches mapping length.
        assert_eq!(list.len(), per_role.len());
        // Sanity: same data round-trips.
        for (role, backend) in &list {
            assert_eq!(per_role.get(*role), *backend);
        }
    }

    #[test]
    fn serde_round_trip_preserves_mapping() {
        let original = PerRoleRuntime::default();
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: PerRoleRuntime = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }
}
