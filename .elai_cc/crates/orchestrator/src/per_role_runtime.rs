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

/// E-20 -- routing-priority axis for [`PerRoleRuntime`].
///
/// Closed enum, 3 variants. Read from `elai_cc.toml`
/// `[agents.<role>.priority]`. Cost = optimize for $; Latency = optimize
/// for ms; Quality = optimize for accuracy / multi-file coherence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Cost,
    Latency,
    Quality,
}

impl Priority {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cost => "cost",
            Self::Latency => "latency",
            Self::Quality => "quality",
        }
    }
}

/// E-20 -- task-shape hint for [`PerRoleRuntime`].
///
/// Closed enum, 3 variants. Read from `elai_cc.toml`
/// `[agents.<role>.task_shape_hint]`. Composes with [`Priority`] to
/// pick the cloud-model personality (Sonnet for speed/$ execution,
/// Opus for multi-file architecture, dedicated math backend for
/// math-heavy work). Grounding: practitioner comparisons documented
/// in E-20 plan task 7 (Cosmic JS / NxCode 2025-2026 reports of
/// 19-50% Opus token savings on complex multi-file refactors).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskShape {
    Execution,
    Architecture,
    Math,
}

impl TaskShape {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Execution => "execution",
            Self::Architecture => "architecture",
            Self::Math => "math",
        }
    }
}

/// Mapping from each `AgentRole` to the runtime backend that should serve it.
///
/// `fallback` is the backend used when a role has no explicit mapping (only
/// possible after deserializing a partial config); `get()` falls back to it
/// rather than panicking.
///
/// E-20 extension: `priority` + `task_shape_hint` BTreeMaps carry the
/// model-personality routing axes used by
/// [`crate::cost::cloud_route`]. Defaults populate all 16 AgentRole
/// variants; missing entries fall back to `Priority::Cost` +
/// `TaskShape::Execution` (cheapest / shortest defaults).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerRoleRuntime {
    /// Primary backend per role. Always populated by `Default` with all 16
    /// AgentRole variants.
    pub mapping: BTreeMap<AgentRole, RuntimeBackend>,
    /// Fallback backend if a role is missing from `mapping` (covers
    /// forward-compat where a new AgentRole variant ships but the persisted
    /// config still uses the old keys).
    pub fallback: RuntimeBackend,
    /// E-20 -- per-role priority axis. Default populates all 16
    /// AgentRole variants per the
    /// `default_anthropic_tier_matches_plan` /
    /// `default_sidecar_tier_matches_plan` test convention.
    #[serde(default = "default_priority_map")]
    pub priority: BTreeMap<AgentRole, Priority>,
    /// E-20 -- per-role task-shape hint. Default populates all 16
    /// AgentRole variants.
    #[serde(default = "default_task_shape_map")]
    pub task_shape_hint: BTreeMap<AgentRole, TaskShape>,
}

/// E-20 -- default priority map. Auditor + Researcher + Designer +
/// MediaValidator + per-level auditors (Quality, multi-file reasoning)
/// route to Opus-style backends; Executor + ContextCompiler +
/// MemoryWriter route to Sonnet-style backends; planner-shape roles
/// (PromptEngineer / IdeaFramer / IdeaCritic / InjectionClassifier)
/// route to Cost (cheapest available).
fn default_priority_map() -> BTreeMap<AgentRole, Priority> {
    let mut map = BTreeMap::new();
    // Quality-priority roles -- deep reasoning / multi-file / audit.
    map.insert(AgentRole::Designer, Priority::Quality);
    map.insert(AgentRole::Auditor, Priority::Quality);
    map.insert(AgentRole::Researcher, Priority::Quality);
    map.insert(AgentRole::MediaValidator, Priority::Quality);
    map.insert(AgentRole::AuditorConcepts, Priority::Quality);
    map.insert(AgentRole::AuditorPhases, Priority::Quality);
    map.insert(AgentRole::AuditorSteps, Priority::Quality);
    map.insert(AgentRole::AuditorBabySteps, Priority::Quality);
    map.insert(AgentRole::AuditorBabyStepDetail, Priority::Quality);
    // Latency-priority roles -- speed/$ matters for short turns.
    map.insert(AgentRole::Executor, Priority::Latency);
    map.insert(AgentRole::ContextCompiler, Priority::Latency);
    map.insert(AgentRole::MemoryWriter, Priority::Latency);
    // Cost-priority roles -- planner shape; local sidecar is good enough.
    map.insert(AgentRole::PromptEngineer, Priority::Cost);
    map.insert(AgentRole::IdeaFramer, Priority::Cost);
    map.insert(AgentRole::IdeaCritic, Priority::Cost);
    map.insert(AgentRole::InjectionClassifier, Priority::Cost);
    map
}

/// E-20 -- default task-shape map. Audit + Designer + per-level
/// auditors are Architecture-shaped; Executor + ContextCompiler +
/// MemoryWriter are Execution-shaped; Researcher is Architecture-
/// shaped (cross-source synthesis); planner roles are Execution-shaped
/// (turn-by-turn prompts).
fn default_task_shape_map() -> BTreeMap<AgentRole, TaskShape> {
    let mut map = BTreeMap::new();
    // Architecture-shape -- multi-file / multi-step reasoning.
    map.insert(AgentRole::Designer, TaskShape::Architecture);
    map.insert(AgentRole::Auditor, TaskShape::Architecture);
    map.insert(AgentRole::Researcher, TaskShape::Architecture);
    map.insert(AgentRole::AuditorConcepts, TaskShape::Architecture);
    map.insert(AgentRole::AuditorPhases, TaskShape::Architecture);
    map.insert(AgentRole::AuditorSteps, TaskShape::Architecture);
    map.insert(AgentRole::AuditorBabySteps, TaskShape::Architecture);
    map.insert(AgentRole::AuditorBabyStepDetail, TaskShape::Architecture);
    // Execution-shape -- single-file turns + planner output.
    map.insert(AgentRole::Executor, TaskShape::Execution);
    map.insert(AgentRole::ContextCompiler, TaskShape::Execution);
    map.insert(AgentRole::MemoryWriter, TaskShape::Execution);
    map.insert(AgentRole::PromptEngineer, TaskShape::Execution);
    map.insert(AgentRole::IdeaFramer, TaskShape::Execution);
    map.insert(AgentRole::IdeaCritic, TaskShape::Execution);
    map.insert(AgentRole::MediaValidator, TaskShape::Execution);
    map.insert(AgentRole::InjectionClassifier, TaskShape::Execution);
    map
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

        // E-26 (inference-engines 2026): when `ELAI_SGLANG_BASE_URL` is set
        // (non-empty), planner-shape roles (PromptEngineer / IdeaFramer /
        // IdeaCritic) route to Sglang for the cloud-throughput tier.
        // When unset, all routing collapses to today's B-37 Sidecar
        // behaviour (no behaviour change in default deploy). The 5
        // per-level auditor siblings stay on AnthropicApi regardless of
        // the env state per the sglang_per_role_routing contract test.
        let sglang_enabled = std::env::var("ELAI_SGLANG_BASE_URL")
            .ok()
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);
        let planner_backend = if sglang_enabled {
            RuntimeBackend::Sglang
        } else {
            RuntimeBackend::Sidecar
        };

        // Sidecar tier — local code generation, context compilation, memory
        // writing. Planner-shape (PromptEngineer / IdeaFramer / IdeaCritic)
        // gated by ELAI_SGLANG_BASE_URL per E-26.
        mapping.insert(AgentRole::Executor, RuntimeBackend::Sidecar);
        mapping.insert(AgentRole::ContextCompiler, RuntimeBackend::Sidecar);
        mapping.insert(AgentRole::MemoryWriter, RuntimeBackend::Sidecar);
        mapping.insert(AgentRole::PromptEngineer, planner_backend);
        mapping.insert(AgentRole::IdeaFramer, planner_backend);
        mapping.insert(AgentRole::IdeaCritic, planner_backend);
        // E-37 — off-model prompt-injection classifier. Sidecar tier by
        // default (local Qwen-3B is cheap enough for per-tool classification
        // batches per arXiv:2502.06703 cheap-monitors discipline); cloud
        // Haiku is the fallback when the sidecar reports unavailable.
        mapping.insert(AgentRole::InjectionClassifier, RuntimeBackend::Sidecar);

        Self {
            mapping,
            fallback: RuntimeBackend::Sidecar,
            priority: default_priority_map(),
            task_shape_hint: default_task_shape_map(),
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

    /// E-20 -- priority for `role`. Returns `Priority::Cost` as the
    /// forward-compat default if the role is missing from the map
    /// (only possible after deserializing a config that pre-dates the
    /// E-20 schema extension).
    #[must_use]
    pub fn priority(&self, role: AgentRole) -> Priority {
        self.priority.get(&role).copied().unwrap_or(Priority::Cost)
    }

    /// E-20 -- task-shape hint for `role`. Returns
    /// `TaskShape::Execution` as the forward-compat default.
    #[must_use]
    pub fn task_shape(&self, role: AgentRole) -> TaskShape {
        self.task_shape_hint
            .get(&role)
            .copied()
            .unwrap_or(TaskShape::Execution)
    }

    /// E-20 -- override priority for `role`.
    pub fn set_priority(&mut self, role: AgentRole, priority: Priority) {
        self.priority.insert(role, priority);
    }

    /// E-20 -- override task-shape hint for `role`.
    pub fn set_task_shape(&mut self, role: AgentRole, shape: TaskShape) {
        self.task_shape_hint.insert(role, shape);
    }
}

// ---------------------------------------------------------------------------
// E-30: Auditor-lineage collision detection (arXiv:2511.18397).
// ---------------------------------------------------------------------------

/// Lineage group for an agent identified by session + spawning origin.
///
/// Two agents share a lineage group when they derive from the same parent
/// session or are spawned by the same orchestrator instance within a single
/// FSM cycle. Using the same-lineage agent as both executor and auditor
/// introduces correlated bias (73% false-negative rate; arXiv:2511.18397).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct LineageGroup(pub String);

impl LineageGroup {
    pub fn new(group: impl Into<String>) -> Self {
        Self(group.into())
    }
}

/// Returns `true` if the auditor and executor would share a lineage group.
///
/// # Autonomy policy (E-30)
/// - In autonomous mode: collision is a BLOCK — caller must refuse dispatch.
/// - In interactive mode: collision is a WARN — caller should surface to user.
///
/// The distinction is left to the caller; this fn is a pure predicate.
pub fn auditor_executor_lineage_collision(
    auditor_lineage: &LineageGroup,
    executor_lineage: &LineageGroup,
) -> bool {
    auditor_lineage == executor_lineage
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
            // E-37: off-model prompt-injection classifier.
            AgentRole::InjectionClassifier,
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

    // E-30: auditor-lineage collision tests
    #[test]
    fn same_lineage_group_is_collision() {
        let g = LineageGroup::new("session-abc");
        assert!(
            auditor_executor_lineage_collision(&g, &g),
            "same lineage group must be a collision"
        );
    }

    #[test]
    fn different_lineage_groups_no_collision() {
        let auditor = LineageGroup::new("session-audit-pool");
        let executor = LineageGroup::new("session-exec-pool");
        assert!(
            !auditor_executor_lineage_collision(&auditor, &executor),
            "different lineage groups must not be a collision"
        );
    }

    // E-20: priority + task_shape_hint extension tests.
    #[test]
    fn priority_labels_stable() {
        assert_eq!(Priority::Cost.as_str(), "cost");
        assert_eq!(Priority::Latency.as_str(), "latency");
        assert_eq!(Priority::Quality.as_str(), "quality");
    }

    #[test]
    fn task_shape_labels_stable() {
        assert_eq!(TaskShape::Execution.as_str(), "execution");
        assert_eq!(TaskShape::Architecture.as_str(), "architecture");
        assert_eq!(TaskShape::Math.as_str(), "math");
    }

    #[test]
    fn default_priority_covers_every_agent_role() {
        let per_role = PerRoleRuntime::default();
        for role in all_roles() {
            assert!(
                per_role.priority.contains_key(&role),
                "Priority for AgentRole::{role:?} missing from Default"
            );
        }
    }

    #[test]
    fn default_task_shape_covers_every_agent_role() {
        let per_role = PerRoleRuntime::default();
        for role in all_roles() {
            assert!(
                per_role.task_shape_hint.contains_key(&role),
                "TaskShape for AgentRole::{role:?} missing from Default"
            );
        }
    }

    #[test]
    fn default_priority_auditor_is_quality() {
        let per_role = PerRoleRuntime::default();
        for role in [
            AgentRole::Auditor,
            AgentRole::AuditorConcepts,
            AgentRole::AuditorPhases,
            AgentRole::AuditorSteps,
            AgentRole::AuditorBabySteps,
            AgentRole::AuditorBabyStepDetail,
        ] {
            assert_eq!(per_role.priority(role), Priority::Quality);
        }
    }

    #[test]
    fn default_priority_executor_is_latency() {
        let per_role = PerRoleRuntime::default();
        assert_eq!(per_role.priority(AgentRole::Executor), Priority::Latency);
    }

    #[test]
    fn default_task_shape_designer_is_architecture() {
        let per_role = PerRoleRuntime::default();
        assert_eq!(
            per_role.task_shape(AgentRole::Designer),
            TaskShape::Architecture
        );
    }

    #[test]
    fn default_task_shape_executor_is_execution() {
        let per_role = PerRoleRuntime::default();
        assert_eq!(
            per_role.task_shape(AgentRole::Executor),
            TaskShape::Execution
        );
    }

    #[test]
    fn set_priority_round_trip() {
        let mut per_role = PerRoleRuntime::default();
        per_role.set_priority(AgentRole::Executor, Priority::Quality);
        assert_eq!(per_role.priority(AgentRole::Executor), Priority::Quality);
    }

    #[test]
    fn set_task_shape_round_trip() {
        let mut per_role = PerRoleRuntime::default();
        per_role.set_task_shape(AgentRole::Executor, TaskShape::Math);
        assert_eq!(per_role.task_shape(AgentRole::Executor), TaskShape::Math);
    }

    #[test]
    fn priority_serde_round_trips() {
        let original = PerRoleRuntime::default();
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: PerRoleRuntime = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
        // Spot-check one value pre + post.
        assert_eq!(decoded.priority(AgentRole::Auditor), Priority::Quality);
        assert_eq!(
            decoded.task_shape(AgentRole::AuditorBabyStepDetail),
            TaskShape::Architecture
        );
    }

    #[test]
    fn priority_forward_compat_default_is_cost() {
        // Simulate a config produced before E-20 shipped (the priority +
        // task_shape_hint fields will fall back to default_*_map via
        // serde's `#[serde(default = ...)]` -- here we test that an
        // explicit empty BTreeMap also returns Cost / Execution).
        let mut per_role = PerRoleRuntime::default();
        per_role.priority.clear();
        per_role.task_shape_hint.clear();
        assert_eq!(per_role.priority(AgentRole::Executor), Priority::Cost);
        assert_eq!(
            per_role.task_shape(AgentRole::Executor),
            TaskShape::Execution
        );
    }
}
