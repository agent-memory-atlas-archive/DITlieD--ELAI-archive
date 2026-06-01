//! E-3 sub-phase 4B — predicted-next-role table + VRAM-tier gating.
//!
//! Pure mapping table used by
//! [`crate::runtime_orchestrator::RuntimeOrchestrator::predict_and_warm_next_role`]
//! to decide which adapter to pre-warm immediately after a successful
//! `try_complete`. The mapping reflects the canonical FSM transition
//! flow per `docs/specs/architecture-overview.txt`:
//!
//!   * Designer / Researcher / IdeaFramer / PromptEngineer (planner-shape)
//!     -> Executor (coder-shape) is the next move on PLANNING -> EXECUTING.
//!   * Executor (coder-shape) -> Auditor (critic-shape) per the
//!     EXECUTING -> AUDITING canonical flow.
//!   * Auditor (critic-shape) -> Designer (planner-shape) per
//!     AUDITING -> MEMORIZING -> PLANNING (planner regains control).
//!   * Everything else falls back to Designer (planner) which is the
//!     coldest backend in the typical cycle so a warm helps the most.
//!
//! VRAM-tier gating (see memory:project_vram_tier_plan.md):
//!   * `Low`  (8GB-12GB single GPU): pre-warm disabled — host cannot hold
//!     two model weight sets at once without thrashing.
//!   * `Medium` (16GB-24GB): pre-warm enabled.
//!   * `High` (24GB+ / 48GB+): pre-warm enabled (aggressive variant
//!     reserved for a future tier-aware policy that warms multiple
//!     next-likely backends in parallel — out of scope for 4B).
//!
//! Tier is sourced from `ELAI_VRAM_TIER` (case-insensitive: `low` /
//! `medium` / `high`). Default is `Low` when the env var is unset so
//! 12GB hosts stay safe by default. Cloud-only Anthropic adapters
//! treat warm as a connection-pool ping regardless of GPU tier, so the
//! tier gate only blocks heavyweight local backends (sidecar / vllm /
//! sglang / mlx / llama_cpp).

use elai_ipc_types::AgentRole;

/// VRAM tier for the host running ELAI. Drives the pre-warm gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VramTier {
    /// Single-GPU 8-12GB: cannot hold two weight sets; pre-warm is
    /// effectively a no-op (would force the prior weights out).
    Low,
    /// Single-GPU 16-24GB OR small multi-GPU: pre-warm enabled,
    /// single-backend variant.
    Medium,
    /// 24GB+ single-GPU OR 32GB+ multi-GPU: pre-warm enabled with
    /// headroom for aggressive variants in future sub-phases.
    High,
}

impl VramTier {
    /// Snake-case wire label used by SSB `details_json` and dashboards.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    /// Read the tier from `ELAI_VRAM_TIER`. Unset / unrecognised values
    /// default to `Low` so 12GB single-GPU hosts (the most common
    /// deployment per memory:project_vram_tier_plan.md) stay safe.
    #[must_use]
    pub fn from_env() -> Self {
        match std::env::var("ELAI_VRAM_TIER") {
            Ok(v) => match v.trim().to_ascii_lowercase().as_str() {
                "low" | "12gb" | "12" | "8gb" | "8" => Self::Low,
                "medium" | "med" | "16gb" | "16" | "24gb" | "24" => Self::Medium,
                "high" | "48gb" | "48" => Self::High,
                _ => Self::Low,
            },
            Err(_) => Self::Low,
        }
    }
}

/// Predict the most-likely next `AgentRole` to fire after the given
/// `current_role` completes. Returns `None` when the current role does
/// not have a documented next-role hint (the orchestrator then skips
/// the pre-warm cycle entirely).
///
/// The mapping is intentionally small and biased toward the
/// planner / coder / critic triangle that dominates plan execution.
/// Auditor variants collapse to the AUDITING -> PLANNING handoff.
/// Idea-framer / idea-critic (B-40) cycle back to the framer.
#[must_use]
pub fn predict_next_role(current_role: AgentRole) -> Option<AgentRole> {
    match current_role {
        // planner-shape -> coder-shape
        AgentRole::Designer
        | AgentRole::Researcher
        | AgentRole::PromptEngineer
        | AgentRole::ContextCompiler
        | AgentRole::IdeaFramer => Some(AgentRole::Executor),
        // coder-shape -> critic-shape
        AgentRole::Executor => Some(AgentRole::Auditor),
        // critic-shape -> planner-shape (next plan iteration)
        AgentRole::Auditor
        | AgentRole::AuditorConcepts
        | AgentRole::AuditorPhases
        | AgentRole::AuditorSteps
        | AgentRole::AuditorBabySteps
        | AgentRole::AuditorBabyStepDetail
        | AgentRole::AuditorProductionGrade
        | AgentRole::IdeaCritic => Some(AgentRole::Designer),
        // memory_writer fires at end-of-plan; the next likely move is
        // a fresh framing pass on the next idea.
        AgentRole::MemoryWriter => Some(AgentRole::IdeaFramer),
        // off-side roles have no canonical next; skip pre-warm.
        AgentRole::MediaValidator | AgentRole::InjectionClassifier => None,
    }
}

/// Tier-gate decision: should the orchestrator issue an
/// `adapter.warm()` for the predicted-next backend on this host ?
///
/// Returns `false` on `Low` (default) so 12GB hosts stay no-op-safe.
/// Returns `true` on `Medium` / `High`.
#[must_use]
pub fn tier_allows_prewarm(tier: VramTier) -> bool {
    matches!(tier, VramTier::Medium | VramTier::High)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predict_planner_to_coder() {
        assert_eq!(
            predict_next_role(AgentRole::Designer),
            Some(AgentRole::Executor)
        );
        assert_eq!(
            predict_next_role(AgentRole::Researcher),
            Some(AgentRole::Executor)
        );
        assert_eq!(
            predict_next_role(AgentRole::PromptEngineer),
            Some(AgentRole::Executor)
        );
    }

    #[test]
    fn predict_coder_to_critic() {
        assert_eq!(
            predict_next_role(AgentRole::Executor),
            Some(AgentRole::Auditor)
        );
    }

    #[test]
    fn predict_critic_to_planner() {
        assert_eq!(
            predict_next_role(AgentRole::Auditor),
            Some(AgentRole::Designer)
        );
        assert_eq!(
            predict_next_role(AgentRole::AuditorConcepts),
            Some(AgentRole::Designer)
        );
        assert_eq!(
            predict_next_role(AgentRole::IdeaCritic),
            Some(AgentRole::Designer)
        );
    }

    #[test]
    fn predict_offside_roles_return_none() {
        assert_eq!(predict_next_role(AgentRole::MediaValidator), None);
        assert_eq!(predict_next_role(AgentRole::InjectionClassifier), None);
    }

    #[test]
    fn tier_gate_blocks_low() {
        assert!(!tier_allows_prewarm(VramTier::Low));
        assert!(tier_allows_prewarm(VramTier::Medium));
        assert!(tier_allows_prewarm(VramTier::High));
    }
}
