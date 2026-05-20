//! D-13 — Consensus-role enum for adversarial verifier wiring.
//!
//! This is the closed enum of *consensus-pass roles* used by
//! [`run_adversarial_pass`](crate::agent_pool::run_adversarial_pass).
//! It is intentionally distinct from [`elai_ipc_types::AgentRole`] —
//! `elai_ipc_types::AgentRole` is the agent-dispatch wire role
//! (Executor / Auditor / PromptEngineer / …), while this enum tags
//! who is contributing a [`crate::verification_result::VerificationResult`]
//! to a consensus pass (worker generates, verifier scores, adversary
//! attempts to falsify, critic / reflector / auditor inspect).
//!
//! Closed enum, no `Other(String)` escape — adding a new consensus role
//! is a plan-level schema bump. The D-13 contribution is
//! [`ConsensusRole::Adversary`], which is required for any
//! [`crate::grounding_mode::GroundingMode::FalsifiabilityCrossFamily`]
//! pass per the literature (arxiv:2504.16760 lightweight latent
//! verifiers, T1; arxiv:2507.02778 Self-Correction Bench 64.5%
//! blind-spot rate, T1; arxiv:2403.07974 adversarial-verification
//! taxonomy, T1).
//!
//! ## Why a separate enum (not extend `elai_ipc_types::AgentRole`)
//!
//! `AgentRole` crosses the IPC wire on every `AgentDispatchConfig` and
//! is part of the locked 16-variant `IpcMessageType` surface. The
//! consensus-role tag never crosses the IPC boundary — it stays inside
//! the orchestrator's verifier-consensus path. Keeping them disjoint
//! preserves the IPC wire format and keeps the dispatch / consensus
//! concerns independent.

use serde::{Deserialize, Serialize};

/// Closed enum of consensus-pass roles. Tags the role a contributor
/// played in a multi-verifier consensus run.
///
/// L3-locked: agent self-modification must not introduce new variants —
/// doing so would silently re-shape the heterogeneous-verifier check.
/// New variants land via plan-level schema bumps only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusRole {
    /// Generates a candidate answer / chain. Default executor role.
    Worker,
    /// Scores a candidate against a verifier signal (PPM, intrinsic,
    /// deterministic, cross-context — see
    /// [`crate::verification_result::VerifierKind`]).
    Verifier,
    /// D-13 contribution. Generates structured counter-evidence
    /// against a worker claim from a DIFFERENT model family. Required
    /// for any `FalsifiabilityCrossFamily` mode dispatch. Per
    /// arxiv:2504.16760 (T1) heterogeneous-verifier is the documented
    /// defense against same-family verifier-transfer failure.
    Adversary,
    /// Inspects a worker chain post-hoc and points out gaps without
    /// scoring (constructive critique). Distinct from `Verifier`
    /// (numeric score) and `Adversary` (counter-evidence generation).
    Critic,
    /// Reflects on the consensus outcome (post-aggregation) and
    /// produces a textual summary. Does NOT vote in the consensus.
    Reflector,
    /// Plan-mode auditor — single-cycle integration pass. Mirrors the
    /// `elai_ipc_types::AgentRole::Auditor` semantics but at the
    /// consensus-pass level.
    Auditor,
}

impl ConsensusRole {
    /// Snake-case wire label used in SSB rows + dashboard rendering.
    /// Must match the `#[serde(rename_all = "snake_case")]` output so
    /// the enum and its label stay in lockstep.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Worker => "worker",
            Self::Verifier => "verifier",
            Self::Adversary => "adversary",
            Self::Critic => "critic",
            Self::Reflector => "reflector",
            Self::Auditor => "auditor",
        }
    }

    /// Full closed-enum enumeration — useful for tests / dashboard
    /// histogram seeding (so the panel renders zero-count columns for
    /// roles that have not yet been observed in a window).
    #[must_use]
    pub const fn all() -> [ConsensusRole; 6] {
        [
            Self::Worker,
            Self::Verifier,
            Self::Adversary,
            Self::Critic,
            Self::Reflector,
            Self::Auditor,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_matches_serde_snake_case() {
        for role in ConsensusRole::all() {
            let s = serde_json::to_string(&role).expect("serialize");
            // serde_json wraps unit variants in quotes -> "snake_case".
            let trimmed = s.trim_matches('"');
            assert_eq!(trimmed, role.as_str(), "label mismatch for {role:?}");
        }
    }

    #[test]
    fn adversary_present_in_all() {
        assert!(ConsensusRole::all().contains(&ConsensusRole::Adversary));
    }

    #[test]
    fn all_six_variants() {
        assert_eq!(ConsensusRole::all().len(), 6);
    }
}
