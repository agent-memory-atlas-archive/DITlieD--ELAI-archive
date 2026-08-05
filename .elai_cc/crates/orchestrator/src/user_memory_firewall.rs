//! H-31 (SCHEMA_VERSION 240): the Personalization-Trap firewall -- the #1 load-bearing safety
//! constraint of the user-preference bi-temporal ledger.
//!
//! User-memory (durable WORK preferences captured via `/remember`) is injected into the model
//! prompt for an ALLOWLISTED role ONLY. The allowlist is a CLOSED, COMPILE-TIME exhaustive match
//! over BOTH role enums:
//!   * [`elai_ipc_types::AgentRole`] -- the agent-dispatch wire role threaded through the
//!     BabyStepExecutor. ALLOW = `Executor` ONLY. Every reviewer/auditor-shaped role
//!     (`Auditor` + the 6 per-level/production auditors + `IdeaCritic` + `InjectionClassifier`)
//!     and every non-prompt-assembling agent role (`Researcher`/`Designer`/... ) gets ZERO
//!     user-memory.
//!   * [`crate::agent_role::ConsensusRole`] -- the consensus-pass role. ALLOW = `Worker` ONLY.
//!     `Verifier`/`Adversary`/`Critic`/`Reflector`/`Auditor` all get ZERO user-memory.
//!
//! WHY A CLOSED MATCH, NOT A DENY-LIST: a default-inject-with-deny-list silently leaks user-memory
//! into any role a future contributor forgets to deny. The exhaustive match means a NEW role
//! variant added later FAILS TO COMPILE until an explicit allow/deny decision is made for it
//! (leak-by-omission is structurally impossible). The slot-fill site calls
//! [`agent_role_gets_user_memory`] / [`consensus_role_gets_user_memory`] BEFORE retrieving any
//! fact; when it returns `false` the Memory slot is forced EMPTY (not merely truncated).
//!
//! GROUNDING (citation discipline -- two bad cites were caught at grounding time):
//!   * Personalization has ROLE-DEPENDENT effects on epistemic independence
//!     (arXiv:2603.00024, Kelley & Riedl: personalization DECREASES epistemic independence in a
//!     social-peer role). The adversarial-reviewer/auditor user-memory exclusion is CONSERVATIVE
//!     DEFENSIVE ENGINEERING consistent with that mechanism -- the auditor role itself was NOT
//!     directly tested.
//!   * NOT cited: the "+68% token narration trap" number (FABRICATED, not in Zep arXiv:2501.13956).
//!   * NOT cited as judge-exclusion support: arXiv:2510.09905 (REAL but WRONG DOMAIN -- it studies
//!     emotional reasoning in a RESPONDENT role, not a judge/auditor evaluating external content).
//!
//! See `workflow/reasoning/H-31-decisions.md`.

use crate::agent_role::ConsensusRole;
use crate::ssb::{
    MemoryGovernanceEventPayload, MemoryGovernanceKind, SsbPayload,
};
use elai_ipc_types::AgentRole;

/// Compile-time ALLOWLIST gate for the agent-dispatch role enum.
///
/// Returns `true` ONLY for [`AgentRole::Executor`] (the worker/primary role that assembles a
/// production turn). Every other role -- the auditor family, the critic, the injection classifier,
/// and every non-prompt-assembling agent -- returns `false` and receives ZERO user-memory.
///
/// The match is exhaustive and has NO wildcard arm: adding a new [`AgentRole`] variant will fail
/// to compile here until an explicit allow/deny decision is made. This is the structural guarantee
/// against leak-by-omission.
#[must_use]
pub fn agent_role_gets_user_memory(role: AgentRole) -> bool {
    match role {
        // ALLOW: the worker/primary/executor role. This is the ONLY role that receives the
        // user's durable WORK preferences in its prompt.
        AgentRole::Executor => true,
        // DENY (reviewer/adversary-shaped roles -- Personalization-Trap exclusion): user-memory in
        // an evaluative/critical role would bias its judgment toward the user (arXiv:2603.00024
        // role-dependent epistemic independence; conservative defensive engineering).
        AgentRole::Auditor
        | AgentRole::IdeaCritic
        | AgentRole::AuditorConcepts
        | AgentRole::AuditorPhases
        | AgentRole::AuditorSteps
        | AgentRole::AuditorBabySteps
        | AgentRole::AuditorBabyStepDetail
        | AgentRole::AuditorProductionGrade
        | AgentRole::InjectionClassifier => false,
        // DENY (non-prompt-assembling agent roles -- allowlist default, not a leak): these roles do
        // not assemble a production user-facing turn, so they have no business carrying the user's
        // preferences. Explicit deny so a future reassignment is a conscious decision.
        AgentRole::Researcher
        | AgentRole::Designer
        | AgentRole::PromptEngineer
        | AgentRole::ContextCompiler
        | AgentRole::MemoryWriter
        | AgentRole::MediaValidator
        | AgentRole::IdeaFramer => false,
    }
}

/// Compile-time ALLOWLIST gate for the consensus-pass role enum.
///
/// Returns `true` ONLY for [`ConsensusRole::Worker`] (the candidate-generating role). The
/// `Verifier`/`Adversary`/`Critic`/`Reflector`/`Auditor` roles all return `false` -- user-memory
/// in a consensus verifier/adversary/critic biases the heterogeneous-verifier check.
///
/// Exhaustive, no wildcard: a new [`ConsensusRole`] variant fails to compile until an explicit
/// allow/deny decision is made.
#[must_use]
pub fn consensus_role_gets_user_memory(role: ConsensusRole) -> bool {
    match role {
        // ALLOW: the candidate-generating worker. The ONLY consensus role that receives user-memory.
        ConsensusRole::Worker => true,
        // DENY (every evaluative consensus role -- Personalization-Trap exclusion). Reflector
        // summarizes post-aggregation and does not need user-memory either.
        ConsensusRole::Verifier
        | ConsensusRole::Adversary
        | ConsensusRole::Critic
        | ConsensusRole::Reflector
        | ConsensusRole::Auditor => false,
    }
}

/// J-59: build a [`SsbPayload::MemoryGovernanceEvent`] for per-seat isolation
/// audit rows (`WriteStamped` / `RetrievalFiltered` / `FirewallBlocked`).
///
/// `fact_id` is `""` for isolation-only rows that are not bound to a single
/// fact. **NEVER** include memory content, preference text, or seat secrets in
/// the payload — only the opaque `seat_id` handle and the kind discriminator.
#[must_use]
pub fn memory_isolation_payload(
    kind: MemoryGovernanceKind,
    seat_id: &str,
    actor: &str,
    session_id: &str,
    created_at_ms: u64,
) -> SsbPayload {
    SsbPayload::MemoryGovernanceEvent(MemoryGovernanceEventPayload {
        fact_id: String::new(),
        kind,
        actor: actor.to_owned(),
        session_id: session_id.to_owned(),
        created_at_ms,
        seat_id: seat_id.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executor_is_the_only_allowed_agent_role() {
        assert!(
            agent_role_gets_user_memory(AgentRole::Executor),
            "Executor MUST receive user-memory (the worker/primary role)"
        );
    }

    #[test]
    fn every_auditor_family_role_gets_zero_user_memory() {
        // The load-bearing firewall: no reviewer/auditor-shaped role may carry user-memory.
        let excluded = [
            AgentRole::Auditor,
            AgentRole::IdeaCritic,
            AgentRole::AuditorConcepts,
            AgentRole::AuditorPhases,
            AgentRole::AuditorSteps,
            AgentRole::AuditorBabySteps,
            AgentRole::AuditorBabyStepDetail,
            AgentRole::AuditorProductionGrade,
            AgentRole::InjectionClassifier,
        ];
        for role in excluded {
            assert!(
                !agent_role_gets_user_memory(role),
                "reviewer/auditor role {role:?} MUST get ZERO user-memory (Personalization-Trap)"
            );
        }
    }

    #[test]
    fn non_prompt_assembling_agent_roles_get_zero_user_memory() {
        let excluded = [
            AgentRole::Researcher,
            AgentRole::Designer,
            AgentRole::PromptEngineer,
            AgentRole::ContextCompiler,
            AgentRole::MemoryWriter,
            AgentRole::MediaValidator,
            AgentRole::IdeaFramer,
        ];
        for role in excluded {
            assert!(
                !agent_role_gets_user_memory(role),
                "non-prompt-assembling role {role:?} MUST get ZERO user-memory (allowlist default)"
            );
        }
    }

    #[test]
    fn worker_is_the_only_allowed_consensus_role() {
        assert!(
            consensus_role_gets_user_memory(ConsensusRole::Worker),
            "Worker MUST receive user-memory (the candidate-generating role)"
        );
    }

    #[test]
    fn every_evaluative_consensus_role_gets_zero_user_memory() {
        for role in ConsensusRole::all() {
            if matches!(role, ConsensusRole::Worker) {
                continue;
            }
            assert!(
                !consensus_role_gets_user_memory(role),
                "evaluative consensus role {role:?} MUST get ZERO user-memory (Personalization-Trap)"
            );
        }
    }
}
