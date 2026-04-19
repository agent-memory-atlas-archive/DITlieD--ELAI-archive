//! Batch 4C (A-18) — per-dispatch GenerationProfile selector.
//!
//! Picks one of the 7 profiles declared by
//! [`crate::context_budget::GenerationProfile`] based on the dispatch role
//! and the current FSM state. Emission is decoupled from selection so the
//! selector trait stays testable without an SSB writer.
//!
//! Production path: `DefaultGenerationProfileSelector` applies the mapping
//! documented in reasoning-architecture-design.txt §GENERATION PARAMETERS —
//! repair retries use `RepairRetry` (temperature 1.0), first attempts use
//! `FirstAttempt`, research/interrogation picks `Interrogation` to stay
//! near-deterministic, AB-MCTS branches pick `TreeBranchA/B/C`.
//!
//! The selector is plumbed onto `TaskRunnerConfig::dispatch_context_profile`
//! as `Option<Arc<dyn GenerationProfileSelector>>`; the `None` default is
//! backwards-compatible and leaves existing call sites on the pre-4C
//! `GenerationProfile::FirstAttempt` implicit behavior.

use std::sync::Arc;

use crate::context_budget::GenerationProfile;
use crate::fsm::FsmState;
use crate::ssb::{ProfileSelectedPayload, Ssb, SsbError, SsbPayload};

/// Free-form role label for the dispatch site.
///
/// Kept as a narrow enum so the selector does not become a string-taxonomy
/// bag. Each variant maps 1:1 to a task_runner call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DispatchRole {
    /// Primary baby-step executor (first attempt).
    Executor,
    /// Repair retry after a failure (`MistakeBook` entry replayed).
    Repair,
    /// Auditor pass.
    Auditor,
    /// Research decomposition / web-research loop.
    Research,
    /// Interrogation-tree traversal.
    Interrogation,
    /// AB-MCTS branch A.
    TreeBranchA,
    /// AB-MCTS branch B.
    TreeBranchB,
    /// AB-MCTS branch C.
    TreeBranchC,
}

impl DispatchRole {
    /// Stable string name for SSB emission.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Executor => "executor",
            Self::Repair => "repair",
            Self::Auditor => "auditor",
            Self::Research => "research",
            Self::Interrogation => "interrogation",
            Self::TreeBranchA => "tree_branch_a",
            Self::TreeBranchB => "tree_branch_b",
            Self::TreeBranchC => "tree_branch_c",
        }
    }
}

/// Selector trait — chooses a [`GenerationProfile`] for a dispatch.
///
/// Object-safe on purpose so it can be held as `Arc<dyn
/// GenerationProfileSelector>` on `TaskRunnerConfig`.
pub trait GenerationProfileSelector: Send + Sync {
    fn select(&self, role: DispatchRole, fsm_state: FsmState) -> GenerationProfile;
}

/// Default selector matching the spec mapping.
///
/// Grounding: reasoning-architecture-design.txt §GENERATION PARAMETERS;
/// code: [`GenerationProfile::temperature`] / `top_p` tables in
/// `context_budget.rs`.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultGenerationProfileSelector;

impl GenerationProfileSelector for DefaultGenerationProfileSelector {
    fn select(&self, role: DispatchRole, fsm_state: FsmState) -> GenerationProfile {
        // RESEARCHING state overrides role-based pick for every call site
        // except AB-MCTS branches, which keep their fixed profile so their
        // diversity budget is not accidentally flattened.
        if matches!(fsm_state, FsmState::Researching) {
            match role {
                DispatchRole::TreeBranchA => return GenerationProfile::TreeBranchA,
                DispatchRole::TreeBranchB => return GenerationProfile::TreeBranchB,
                DispatchRole::TreeBranchC => return GenerationProfile::TreeBranchC,
                _ => return GenerationProfile::Interrogation,
            }
        }
        match role {
            DispatchRole::Executor => GenerationProfile::FirstAttempt,
            DispatchRole::Repair => GenerationProfile::RepairRetry,
            DispatchRole::Auditor => GenerationProfile::Interrogation,
            DispatchRole::Research => GenerationProfile::Interrogation,
            DispatchRole::Interrogation => GenerationProfile::Interrogation,
            DispatchRole::TreeBranchA => GenerationProfile::TreeBranchA,
            DispatchRole::TreeBranchB => GenerationProfile::TreeBranchB,
            DispatchRole::TreeBranchC => GenerationProfile::TreeBranchC,
        }
    }
}

/// Stable identifier name for a [`GenerationProfile`].
///
/// Used by the SSB emitter so the dashboard can filter `ProfileSelected`
/// rows on the profile name without deserializing the full payload.
pub fn profile_name(profile: GenerationProfile) -> &'static str {
    match profile {
        GenerationProfile::FirstAttempt => "FirstAttempt",
        GenerationProfile::MicroReflection => "MicroReflection",
        GenerationProfile::Interrogation => "Interrogation",
        GenerationProfile::RepairRetry => "RepairRetry",
        GenerationProfile::TreeBranchA => "TreeBranchA",
        GenerationProfile::TreeBranchB => "TreeBranchB",
        GenerationProfile::TreeBranchC => "TreeBranchC",
    }
}

/// Emit a single `ProfileSelected` SSB row.
///
/// Kept as a free function so call sites do not have to build the payload
/// by hand; errors are surfaced to the caller so the task_runner can log
/// them without aborting the dispatch.
pub async fn emit_profile_selected(
    writer: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    profile: GenerationProfile,
    role: DispatchRole,
    fsm_state: FsmState,
    step_id: &str,
) -> Result<i64, SsbError> {
    let payload = SsbPayload::ProfileSelected(ProfileSelectedPayload {
        profile: profile_name(profile).to_string(),
        role: role.as_str().to_string(),
        fsm_state: fsm_state.to_string(),
        step_id: step_id.to_string(),
    });
    writer
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_selector_picks_repair_retry_on_repair_role() {
        let s = DefaultGenerationProfileSelector;
        assert_eq!(
            s.select(DispatchRole::Repair, FsmState::Executing),
            GenerationProfile::RepairRetry
        );
    }

    #[test]
    fn default_selector_picks_first_attempt_on_executor_role() {
        let s = DefaultGenerationProfileSelector;
        assert_eq!(
            s.select(DispatchRole::Executor, FsmState::Executing),
            GenerationProfile::FirstAttempt
        );
    }

    #[test]
    fn default_selector_overrides_to_interrogation_in_researching() {
        let s = DefaultGenerationProfileSelector;
        assert_eq!(
            s.select(DispatchRole::Executor, FsmState::Researching),
            GenerationProfile::Interrogation
        );
    }

    #[test]
    fn default_selector_preserves_tree_branch_in_researching() {
        let s = DefaultGenerationProfileSelector;
        assert_eq!(
            s.select(DispatchRole::TreeBranchB, FsmState::Researching),
            GenerationProfile::TreeBranchB
        );
    }

    #[test]
    fn profile_name_covers_all_seven_profiles() {
        let all = [
            GenerationProfile::FirstAttempt,
            GenerationProfile::MicroReflection,
            GenerationProfile::Interrogation,
            GenerationProfile::RepairRetry,
            GenerationProfile::TreeBranchA,
            GenerationProfile::TreeBranchB,
            GenerationProfile::TreeBranchC,
        ];
        for p in all {
            assert!(!profile_name(p).is_empty());
        }
    }
}
