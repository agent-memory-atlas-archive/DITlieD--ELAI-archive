// C-45 — AgentRoleDropper: agent-side role-drop policy for CANTANTE
// contrastive credit attribution.
//
// When the orchestrator's credit-attribution harness (C-45 STEP 3,
// `crates/orchestrator/src/credit_attribution/harness.rs`) runs a
// "drop one role" configuration, it sets the `ELAI_C45_DROPPED_ROLE`
// environment variable to the snake_case `AgentRole` name before
// spawning the agent binary via `agent_pool::launch_agent_process`.
// The agent reads this env var at startup and constructs an
// `AgentRoleDropper`. When the dropper is active and the agent
// receives an `AgentRequest` for the dropped role, the dispatch path
// in `main.rs::handle_message` returns a fallback `AgentResponse`
// instead of entering the full ReAct loop — the "fallback to
// merged-behavior" path described in the C-45 plan STEP 2.
//
// Paper grounding: arXiv:2605.13295 (Zehle 2026, "CANTANTE: Optimizing
// Agentic Systems via Contrastive Credit Attribution", Section 3.2).
// The contrastive method removes each agent in turn and measures the
// task-success delta. This module implements the agent-side "removal"
// primitive: a dropped role never enters its full ReAct loop; instead
// it returns a defined fallback so the pipeline observes a non-empty
// (but degraded) output — never a silent skip (C-45 stub_ban:
// "dropped-role fallback path MUST be DEFINED per role").
//
// Per-role fallback behavior table (C-45 plan STEP 2):
//
//   | Role             | Fallback           | Description                          |
//   |------------------|--------------------|--------------------------------------|
//   | Executor         | MergeIntoAuditor   | Auditor absorbs execution; the       |
//   |                  |                    | auditor-shaped agent takes over      |
//   |                  |                    | the executor's ReAct loop. Executor  |
//   |                  |                    | is core — dropping it is the most    |
//   |                  |                    | extreme ablation.                    |
//   | Auditor          | MergeIntoExecutor  | Executor self-verifies; the          |
//   |                  |                    | executor-shaped agent runs its own   |
//   |                  |                    | inline verification pass instead of  |
//   |                  |                    | a dedicated audit cycle.             |
//   | Researcher       | UseCachedPrior     | Use cached prior research only;      |
//   |                  |                    | never spawn a fresh researcher       |
//   |                  |                    | cycle. The pipeline reuses the       |
//   |                  |                    | most recent research artifacts.      |
//   | Designer         | SkipPhase          | Skip the design phase; proceed       |
//   |                  |                    | with the default / last-known        |
//   |                  |                    | architecture. No design artifacts    |
//   |                  |                    | are produced.                        |
//   | PromptEngineer   | SkipPhase          | Skip prompt engineering; use         |
//   |                  |                    | compiled / default prompt templates  |
//   |                  |                    | without per-task refinement.         |
//   | ContextCompiler  | UseCachedPrior     | Use cached context compilation       |
//   |                  |                    | from prior runs; do not re-compile  |
//   |                  |                    | the context package.                 |
//   | MemoryWriter     | SkipPhase          | Skip memory persistence; the         |
//   |                  |                    | pipeline proceeds without writing    |
//   |                  |                    | to the memory-bank. No artifacts     |
//   |                  |                    | are persisted.                       |
//   | MediaValidator   | NoOp               | Media validation is a no-op for      |
//   |                  |                    | non-media tasks; skip the validation |
//   |                  |                    | pass entirely.                       |
//
// Roles outside the 8-role pilot set (B-40 IdeaFramer/IdeaCritic,
// B-41 per-level Auditor*, E-12 AuditorProductionGrade, E-37
// InjectionClassifier) are NOT droppable in the C-45 pilot — the
// `fallback_for` method returns `None` for them, and the dispatch
// path runs the normal ReAct loop.

use elai_ipc_types::AgentRole;

/// Environment variable name the orchestrator sets when spawning an
/// agent under a C-45 "drop one role" configuration. The value is the
/// snake_case `AgentRole` name (e.g. `"researcher"`). Unset or empty
/// means baseline (no role dropped).
pub const ELAI_C45_DROPPED_ROLE_ENV: &str = "ELAI_C45_DROPPED_ROLE";

/// Per-role fallback strategy when a role is dropped from a
/// contrastive credit-attribution run. Each variant describes what
/// the pipeline does INSTEAD of spawning the dropped role's full
/// agent cycle.
///
/// This is the "fallback to merged-behavior" path from the C-45 plan
/// STEP 2. The fallback is always DEFINED — `panic on drop` is
/// explicitly banned (C-45 stub_ban).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleFallback {
    /// Use cached prior results from earlier runs; do not spawn a
    /// fresh agent cycle for this role. (Researcher, ContextCompiler)
    UseCachedPrior,
    /// Skip the phase entirely; the pipeline proceeds without the
    /// role's output. (Designer, PromptEngineer, MemoryWriter)
    SkipPhase,
    /// Merge the dropped role's responsibility into the Executor's
    /// ReAct loop — the Executor picks up the slack inline.
    /// (Auditor)
    MergeIntoExecutor,
    /// Merge the dropped role's responsibility into the Auditor's
    /// pass — the Auditor takes over. (Executor)
    MergeIntoAuditor,
    /// No-op — the role contributes nothing on this task class.
    /// (MediaValidator for non-media tasks)
    NoOp,
}

impl RoleFallback {
    /// Human-readable description of the fallback strategy, suitable
    /// for inclusion in the fallback `AgentResponse` content so the
    /// orchestrator observes a non-empty, self-describing output
    /// (C-45 stub_ban: "fallback must be defined and observable").
    #[must_use]
    pub fn description(self) -> &'static str {
        match self {
            Self::UseCachedPrior => {
                "use cached prior results; no fresh agent cycle spawned"
            }
            Self::SkipPhase => {
                "skip phase; pipeline proceeds without this role's output"
            }
            Self::MergeIntoExecutor => {
                "merge responsibility into Executor's ReAct loop"
            }
            Self::MergeIntoAuditor => {
                "merge responsibility into Auditor's verification pass"
            }
            Self::NoOp => {
                "no-op; role contributes nothing on this task class"
            }
        }
    }
}

/// Agent-side role-drop policy for CANTANTE contrastive credit
/// attribution (C-45). Holds the `dropped_role` flag and provides
/// the per-role fallback behavior table.
///
/// Constructed at agent startup via [`Self::from_env`] (reads the
/// `ELAI_C45_DROPPED_ROLE` env var set by the orchestrator's
/// `agent_pool::launch_agent_process`). When the flag is set and the
/// agent receives an `AgentRequest` for the dropped role, the
/// dispatch path in `main.rs::handle_message` returns a fallback
/// `AgentResponse` instead of entering the full ReAct loop.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AgentRoleDropper {
    /// `Some(role)` to forcibly drop the role from the run; `None`
    /// for the baseline configuration (no role dropped).
    pub dropped_role: Option<AgentRole>,
}

impl AgentRoleDropper {
    /// Baseline constructor — no role dropped. Equivalent to
    /// `Default::default()`.
    #[must_use]
    pub fn none() -> Self {
        Self { dropped_role: None }
    }

    /// Drop a single role. The dropper will route around `role` for
    /// the lifetime of this agent process.
    #[must_use]
    pub fn drop(role: AgentRole) -> Self {
        Self { dropped_role: Some(role) }
    }

    /// Read the drop policy from the `ELAI_C45_DROPPED_ROLE`
    /// environment variable. The orchestrator sets this var when
    /// spawning the agent under a C-45 "drop one role" configuration.
    ///
    /// Returns [`Self::none()`] when:
    ///   - the env var is unset or empty (baseline run)
    ///   - the env var carries an unrecognized role name (safe
    ///     default — never drop a role we cannot identify)
    #[must_use]
    pub fn from_env() -> Self {
        let raw = std::env::var(ELAI_C45_DROPPED_ROLE_ENV)
            .unwrap_or_default();
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Self::none();
        }
        match parse_role_snake_case(trimmed) {
            Some(role) => Self::drop(role),
            None => {
                tracing::warn!(
                    env = ELAI_C45_DROPPED_ROLE_ENV,
                    raw = %trimmed,
                    "unrecognized role name; defaulting to baseline (no drop)"
                );
                Self::none()
            }
        }
    }

    /// True when `role` is the dropped role. The dispatch path checks
    /// this before entering the ReAct loop.
    #[must_use]
    pub fn is_dropped(&self, role: AgentRole) -> bool {
        self.dropped_role == Some(role)
    }

    /// True when no role is dropped (baseline configuration).
    #[must_use]
    pub fn is_baseline(&self) -> bool {
        self.dropped_role.is_none()
    }

    /// Returns the fallback strategy for `role` when `role` is the
    /// dropped role; `None` otherwise. This is the per-role fallback
    /// behavior table from the C-45 plan STEP 2.
    ///
    /// Returns `None` for roles outside the 8-role pilot set — those
    /// are not droppable in the C-45 pilot and the dispatch path
    /// runs the normal ReAct loop for them.
    #[must_use]
    pub fn fallback_for(&self, role: AgentRole) -> Option<RoleFallback> {
        if !self.is_dropped(role) {
            return None;
        }
        fallback_table(role)
    }

    /// Returns a human-readable fallback description for `role` when
    /// `role` is the dropped role; `None` otherwise. Used in the
    /// fallback `AgentResponse` content so the orchestrator observes
    /// a non-empty, self-describing output.
    #[must_use]
    pub fn fallback_description(&self, role: AgentRole) -> Option<&'static str> {
        self.fallback_for(role).map(|f| f.description())
    }
}

/// Per-role fallback table. Maps each of the 8 base `AgentRole`
/// variants to its defined fallback strategy. Returns `None` for
/// roles outside the pilot set (not droppable in C-45).
///
/// This is the documented per-role fallback behavior table from the
/// C-45 plan STEP 2, encoded as a match so the compiler catches any
/// missing arm when new `AgentRole` variants are added.
#[must_use]
fn fallback_table(role: AgentRole) -> Option<RoleFallback> {
    let fallback = match role {
        AgentRole::Executor => RoleFallback::MergeIntoAuditor,
        AgentRole::Auditor => RoleFallback::MergeIntoExecutor,
        AgentRole::Researcher => RoleFallback::UseCachedPrior,
        AgentRole::Designer => RoleFallback::SkipPhase,
        AgentRole::PromptEngineer => RoleFallback::SkipPhase,
        AgentRole::ContextCompiler => RoleFallback::UseCachedPrior,
        AgentRole::MemoryWriter => RoleFallback::SkipPhase,
        AgentRole::MediaValidator => RoleFallback::NoOp,
        // Roles outside the 8-role pilot set are not droppable in
        // C-45. The fallback is undefined for them — the dispatch
        // path runs the normal ReAct loop.
        AgentRole::IdeaFramer
        | AgentRole::IdeaCritic
        | AgentRole::AuditorConcepts
        | AgentRole::AuditorPhases
        | AgentRole::AuditorSteps
        | AgentRole::AuditorBabySteps
        | AgentRole::AuditorBabyStepDetail
        | AgentRole::AuditorProductionGrade
        | AgentRole::InjectionClassifier => return None,
    };
    Some(fallback)
}

/// Parse a snake_case role name string into the canonical `AgentRole`
/// enum. Returns `None` for unrecognized names.
///
/// The `AgentRole` enum uses `#[serde(rename_all = "snake_case")]`,
/// but the agent crate does not depend on `serde_json` (keeping the
/// sandboxed binary's dep surface minimal per the crate invariant
/// at `main.rs:1-9`). This manual matcher covers the 8 pilot roles
/// plus the newer variants so `from_env` never silently drops a
/// valid role.
#[must_use]
fn parse_role_snake_case(name: &str) -> Option<AgentRole> {
    match name {
        "executor" => Some(AgentRole::Executor),
        "auditor" => Some(AgentRole::Auditor),
        "researcher" => Some(AgentRole::Researcher),
        "designer" => Some(AgentRole::Designer),
        "prompt_engineer" => Some(AgentRole::PromptEngineer),
        "context_compiler" => Some(AgentRole::ContextCompiler),
        "memory_writer" => Some(AgentRole::MemoryWriter),
        "media_validator" => Some(AgentRole::MediaValidator),
        "idea_framer" => Some(AgentRole::IdeaFramer),
        "idea_critic" => Some(AgentRole::IdeaCritic),
        "auditor_concepts" => Some(AgentRole::AuditorConcepts),
        "auditor_phases" => Some(AgentRole::AuditorPhases),
        "auditor_steps" => Some(AgentRole::AuditorSteps),
        "auditor_baby_steps" => Some(AgentRole::AuditorBabySteps),
        "auditor_baby_step_detail" => Some(AgentRole::AuditorBabyStepDetail),
        "auditor_production_grade" => Some(AgentRole::AuditorProductionGrade),
        "injection_classifier" => Some(AgentRole::InjectionClassifier),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- AgentRoleDropper construction ----

    #[test]
    fn none_is_baseline() {
        let d = AgentRoleDropper::none();
        assert!(d.is_baseline());
        assert_eq!(d.dropped_role, None);
    }

    #[test]
    fn drop_carries_role() {
        let d = AgentRoleDropper::drop(AgentRole::Researcher);
        assert!(!d.is_baseline());
        assert_eq!(d.dropped_role, Some(AgentRole::Researcher));
    }

    #[test]
    fn default_is_none() {
        let d: AgentRoleDropper = Default::default();
        assert!(d.is_baseline());
    }

    // ---- is_dropped ----

    #[test]
    fn is_dropped_true_for_dropped_role() {
        let d = AgentRoleDropper::drop(AgentRole::Auditor);
        assert!(d.is_dropped(AgentRole::Auditor));
    }

    #[test]
    fn is_dropped_false_for_other_role() {
        let d = AgentRoleDropper::drop(AgentRole::Auditor);
        assert!(!d.is_dropped(AgentRole::Executor));
    }

    #[test]
    fn is_dropped_false_for_baseline() {
        let d = AgentRoleDropper::none();
        assert!(!d.is_dropped(AgentRole::Executor));
    }

    // ---- fallback_for: per-role fallback table ----

    #[test]
    fn fallback_for_executor_is_merge_into_auditor() {
        let d = AgentRoleDropper::drop(AgentRole::Executor);
        assert_eq!(
            d.fallback_for(AgentRole::Executor),
            Some(RoleFallback::MergeIntoAuditor)
        );
    }

    #[test]
    fn fallback_for_auditor_is_merge_into_executor() {
        let d = AgentRoleDropper::drop(AgentRole::Auditor);
        assert_eq!(
            d.fallback_for(AgentRole::Auditor),
            Some(RoleFallback::MergeIntoExecutor)
        );
    }

    #[test]
    fn fallback_for_researcher_is_use_cached_prior() {
        let d = AgentRoleDropper::drop(AgentRole::Researcher);
        assert_eq!(
            d.fallback_for(AgentRole::Researcher),
            Some(RoleFallback::UseCachedPrior)
        );
    }

    #[test]
    fn fallback_for_designer_is_skip_phase() {
        let d = AgentRoleDropper::drop(AgentRole::Designer);
        assert_eq!(
            d.fallback_for(AgentRole::Designer),
            Some(RoleFallback::SkipPhase)
        );
    }

    #[test]
    fn fallback_for_prompt_engineer_is_skip_phase() {
        let d = AgentRoleDropper::drop(AgentRole::PromptEngineer);
        assert_eq!(
            d.fallback_for(AgentRole::PromptEngineer),
            Some(RoleFallback::SkipPhase)
        );
    }

    #[test]
    fn fallback_for_context_compiler_is_use_cached_prior() {
        let d = AgentRoleDropper::drop(AgentRole::ContextCompiler);
        assert_eq!(
            d.fallback_for(AgentRole::ContextCompiler),
            Some(RoleFallback::UseCachedPrior)
        );
    }

    #[test]
    fn fallback_for_memory_writer_is_skip_phase() {
        let d = AgentRoleDropper::drop(AgentRole::MemoryWriter);
        assert_eq!(
            d.fallback_for(AgentRole::MemoryWriter),
            Some(RoleFallback::SkipPhase)
        );
    }

    #[test]
    fn fallback_for_media_validator_is_no_op() {
        let d = AgentRoleDropper::drop(AgentRole::MediaValidator);
        assert_eq!(
            d.fallback_for(AgentRole::MediaValidator),
            Some(RoleFallback::NoOp)
        );
    }

    // ---- fallback_for: non-pilot roles return None ----

    #[test]
    fn fallback_for_idea_framer_returns_none() {
        let d = AgentRoleDropper::drop(AgentRole::IdeaFramer);
        assert_eq!(d.fallback_for(AgentRole::IdeaFramer), None);
    }

    #[test]
    fn fallback_for_auditor_concepts_returns_none() {
        let d = AgentRoleDropper::drop(AgentRole::AuditorConcepts);
        assert_eq!(d.fallback_for(AgentRole::AuditorConcepts), None);
    }

    // ---- fallback_for: non-dropped role returns None ----

    #[test]
    fn fallback_for_non_dropped_role_returns_none() {
        let d = AgentRoleDropper::drop(AgentRole::Researcher);
        assert_eq!(d.fallback_for(AgentRole::Executor), None);
    }

    #[test]
    fn fallback_for_baseline_returns_none() {
        let d = AgentRoleDropper::none();
        assert_eq!(d.fallback_for(AgentRole::Executor), None);
    }

    // ---- fallback_description ----

    #[test]
    fn fallback_description_is_non_empty_for_each_pilot_role() {
        for role in [
            AgentRole::Executor,
            AgentRole::Auditor,
            AgentRole::Researcher,
            AgentRole::Designer,
            AgentRole::PromptEngineer,
            AgentRole::ContextCompiler,
            AgentRole::MemoryWriter,
            AgentRole::MediaValidator,
        ] {
            let d = AgentRoleDropper::drop(role);
            let desc = d.fallback_description(role).expect("desc defined");
            assert!(!desc.is_empty(), "fallback desc must be non-empty for {role:?}");
        }
    }

    // ---- RoleFallback::description ----

    #[test]
    fn role_fallback_descriptions_are_distinct_and_non_empty() {
        let descs: Vec<&str> = [
            RoleFallback::UseCachedPrior,
            RoleFallback::SkipPhase,
            RoleFallback::MergeIntoExecutor,
            RoleFallback::MergeIntoAuditor,
            RoleFallback::NoOp,
        ]
        .iter()
        .map(|f| f.description())
        .collect();
        for d in &descs {
            assert!(!d.is_empty());
        }
        // All distinct.
        let mut sorted = descs.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), descs.len(), "fallback descs must be distinct");
    }

    // ---- parse_role_snake_case ----

    #[test]
    fn parse_pilot_roles() {
        assert_eq!(
            parse_role_snake_case("executor"),
            Some(AgentRole::Executor)
        );
        assert_eq!(
            parse_role_snake_case("researcher"),
            Some(AgentRole::Researcher)
        );
        assert_eq!(
            parse_role_snake_case("media_validator"),
            Some(AgentRole::MediaValidator)
        );
    }

    #[test]
    fn parse_non_pilot_roles() {
        assert_eq!(
            parse_role_snake_case("idea_framer"),
            Some(AgentRole::IdeaFramer)
        );
        assert_eq!(
            parse_role_snake_case("auditor_production_grade"),
            Some(AgentRole::AuditorProductionGrade)
        );
    }

    #[test]
    fn parse_unknown_returns_none() {
        assert_eq!(parse_role_snake_case("nonexistent"), None);
        assert_eq!(parse_role_snake_case(""), None);
    }

    // ---- from_env (unset var = baseline) ----

    #[test]
    fn from_env_unset_returns_baseline() {
        // SAFETY: this test sets an env var in a single-threaded test.
        // Other tests in this module do not read ELAI_C45_DROPPED_ROLE_ENV
        // concurrently. std::env::set_var is safe in single-threaded
        // tests; the agent binary reads it once at startup.
        std::env::remove_var(ELAI_C45_DROPPED_ROLE_ENV);
        let d = AgentRoleDropper::from_env();
        assert!(d.is_baseline());
    }

    #[test]
    fn from_env_empty_returns_baseline() {
        std::env::set_var(ELAI_C45_DROPPED_ROLE_ENV, "");
        let d = AgentRoleDropper::from_env();
        assert!(d.is_baseline());
        std::env::remove_var(ELAI_C45_DROPPED_ROLE_ENV);
    }

    #[test]
    fn from_env_valid_role_drops_that_role() {
        std::env::set_var(ELAI_C45_DROPPED_ROLE_ENV, "researcher");
        let d = AgentRoleDropper::from_env();
        assert_eq!(d.dropped_role, Some(AgentRole::Researcher));
        assert!(d.is_dropped(AgentRole::Researcher));
        std::env::remove_var(ELAI_C45_DROPPED_ROLE_ENV);
    }

    #[test]
    fn from_env_whitespace_trimmed() {
        std::env::set_var(ELAI_C45_DROPPED_ROLE_ENV, "  auditor  ");
        let d = AgentRoleDropper::from_env();
        assert_eq!(d.dropped_role, Some(AgentRole::Auditor));
        std::env::remove_var(ELAI_C45_DROPPED_ROLE_ENV);
    }

    #[test]
    fn from_env_unknown_role_defaults_to_baseline() {
        std::env::set_var(ELAI_C45_DROPPED_ROLE_ENV, "fake_role");
        let d = AgentRoleDropper::from_env();
        assert!(d.is_baseline(), "unknown role must not drop anything");
        std::env::remove_var(ELAI_C45_DROPPED_ROLE_ENV);
    }

    // ---- C-45 plan STEP 2: all 8 pilot roles have a defined fallback ----

    #[test]
    fn all_eight_pilot_roles_have_defined_fallback() {
        // C-45 stub_ban: "dropped-role fallback path MUST be DEFINED
        // per role; panic on drop is not acceptable."
        let pilot_roles = [
            AgentRole::Executor,
            AgentRole::Auditor,
            AgentRole::Researcher,
            AgentRole::Designer,
            AgentRole::PromptEngineer,
            AgentRole::ContextCompiler,
            AgentRole::MemoryWriter,
            AgentRole::MediaValidator,
        ];
        for role in pilot_roles {
            let d = AgentRoleDropper::drop(role);
            let fallback = d.fallback_for(role);
            assert!(
                fallback.is_some(),
                "C-45 stub_ban: fallback MUST be defined for {role:?}"
            );
            let desc = d.fallback_description(role);
            assert!(
                desc.is_some() && !desc.unwrap().is_empty(),
                "C-45: fallback must be observable (non-empty description) for {role:?}"
            );
        }
    }
}
