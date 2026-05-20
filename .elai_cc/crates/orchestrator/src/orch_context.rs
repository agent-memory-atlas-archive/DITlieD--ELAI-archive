//! D-10 — `OrchContext`: typed inputs the mode-routing predicate reads.
//!
//! Pure value object. The orchestrator constructs an `OrchContext`
//! snapshot at each claim-routing site (cheap — small Copy/Clone
//! fields + already-cached optional summaries). The router
//! ([`crate::grounding_mode::select_grounding_mode`]) consumes it
//! immutably; the agent has no read or write surface on it.
//!
//! D-11 (checker registry), D-13 (Adversary role), D-16 (UniCR), D-18
//! (ObservableEffect sandbox) all extend this struct with their
//! concrete capabilities. To keep D-10 standalone and unblock the
//! producer wire, we ship the typed surface today and stub the
//! capability flags (`conformal_calibration_ready`,
//! `available_verifiers`) so downstream phases can flip them on
//! without re-shaping the public API.
//!
//! Grounding:
//!   * docs/specs/guide/real-problem-solving-grounded-2026-05-19.md §3.2.
//!   * arxiv:1908.04734 (Everitt current-RF) — context is orchestrator-
//!     owned, the agent cannot modify it.

use serde::{Deserialize, Serialize};

use crate::agent_pool::CanaryStatusSummary;
use crate::verification_result::VerifierKind;

/// Coarse role under which the current claim is being emitted. The
/// router's mode-selection table is parametric in `user_role` — e.g.
/// an `Adversary` claim is always routed `FalsifiabilityCrossFamily`,
/// never `Constructive`, regardless of `claim_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchUserRole {
    /// Primary policy model under evaluation (default).
    Executor,
    /// Auditor agent emitting a grading claim — never routed via
    /// constructive checkers because checkers are the auditor's own
    /// tools (Cohen Boxed Myopic — agent cannot grade itself).
    Auditor,
    /// Adversary agent — by definition routes to cross-family.
    Adversary,
    /// Surface-level user query (orchestrator reflection / meta).
    Meta,
}

impl OrchUserRole {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Executor => "executor",
            Self::Auditor => "auditor",
            Self::Adversary => "adversary",
            Self::Meta => "meta",
        }
    }
}

/// Coarse FSM phase the orchestrator is currently in. Mirror of a
/// subset of [`crate::fsm::FsmState`] kept private to D-10 so the
/// router does not depend on the full FSM module surface — keeps the
/// blast radius narrow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchPhase {
    Idle,
    Planning,
    FeasibilityProbe,
    TestingFirst,
    Executing,
    Auditing,
    Memorizing,
    Researching,
    Configuring,
    Complete,
}

impl OrchPhase {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Planning => "planning",
            Self::FeasibilityProbe => "feasibility_probe",
            Self::TestingFirst => "testing_first",
            Self::Executing => "executing",
            Self::Auditing => "auditing",
            Self::Memorizing => "memorizing",
            Self::Researching => "researching",
            Self::Configuring => "configuring",
            Self::Complete => "complete",
        }
    }
}

/// Snapshot the router uses to decide a [`crate::grounding_mode::GroundingMode`].
///
/// All fields are observation-only: the router does not mutate this
/// struct. Producers construct a fresh `OrchContext` per routing call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrchContext {
    pub user_role: OrchUserRole,
    pub current_phase: OrchPhase,
    /// Latest cross-domain canary verdict (D-7). The router uses this
    /// to refuse claims when canaries are blocking promotion — a
    /// failed canary means "do not let the optimizer grade itself
    /// under the same family". Wired by callers via
    /// `agent_pool::recent_canary_status`.
    pub recent_canary_status: Option<CanaryStatusSummary>,
    /// Number of `tripwire_fired` SSB rows observed in the recent
    /// audit window (D-4). High count → router is more conservative
    /// (refusal-biased) per Cohen Boxed Myopic.
    pub recent_tripwire_count: usize,
    /// Closed list of verifier kinds currently online (D-3
    /// consensus aggregator's input set). Used by the router to gate
    /// `FalsifiabilityCrossFamily` on actually having an alternate-
    /// family verifier registered.
    pub available_verifiers: Vec<VerifierKind>,
    /// True when the D-16 UniCR conformal calibration table is ready
    /// (n calibration samples ≥ threshold). When false, `Refuse`
    /// claims must downgrade to `ConservativeFallback` because the
    /// formal `Refuse` mode has no calibrated abstain threshold.
    pub conformal_calibration_ready: bool,
}

impl OrchContext {
    /// Convenience constructor for tests + production call sites that
    /// don't yet wire every field — defaults bias toward refusal
    /// (safer than biasing toward constructive when no verifiers are
    /// registered).
    #[must_use]
    pub fn minimal(user_role: OrchUserRole, current_phase: OrchPhase) -> Self {
        Self {
            user_role,
            current_phase,
            recent_canary_status: None,
            recent_tripwire_count: 0,
            available_verifiers: Vec::new(),
            conformal_calibration_ready: false,
        }
    }

    /// True iff at least one verifier of a family OTHER than `kind`
    /// is registered. Used by the router to decide
    /// `FalsifiabilityCrossFamily` eligibility — Cohen Boxed Myopic
    /// requires the falsifier to be a different family from the
    /// claimer.
    #[must_use]
    pub fn has_cross_family_verifier(&self, kind: VerifierKind) -> bool {
        self.available_verifiers.iter().any(|v| *v != kind)
    }

    /// True when the canary record indicates a blocking failure
    /// (delegates to `CanaryStatusSummary::blocks_promotion`). When
    /// `recent_canary_status` is None the router treats canary status
    /// as "unknown" and does NOT block (canary is advisory at this
    /// layer; D-5 BAI gate is the hard stop).
    #[must_use]
    pub fn canary_blocks(&self) -> bool {
        self.recent_canary_status
            .as_ref()
            .is_some_and(|c| c.blocks_promotion())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_context_is_refusal_biased() {
        let ctx = OrchContext::minimal(OrchUserRole::Executor, OrchPhase::Executing);
        assert!(!ctx.conformal_calibration_ready);
        assert!(ctx.available_verifiers.is_empty());
        assert!(!ctx.canary_blocks());
        assert!(!ctx.has_cross_family_verifier(VerifierKind::Intrinsic));
    }

    #[test]
    fn cross_family_requires_different_kind() {
        let mut ctx = OrchContext::minimal(OrchUserRole::Executor, OrchPhase::Executing);
        ctx.available_verifiers = vec![VerifierKind::Intrinsic];
        assert!(!ctx.has_cross_family_verifier(VerifierKind::Intrinsic));
        ctx.available_verifiers
            .push(VerifierKind::Deterministic);
        assert!(ctx.has_cross_family_verifier(VerifierKind::Intrinsic));
    }

    #[test]
    fn role_as_str_matches_serde() {
        for r in [
            OrchUserRole::Executor,
            OrchUserRole::Auditor,
            OrchUserRole::Adversary,
            OrchUserRole::Meta,
        ] {
            let json = serde_json::to_string(&r).expect("ser");
            let stripped = json.trim_matches('"');
            assert_eq!(stripped, r.as_str());
        }
    }

    #[test]
    fn phase_as_str_matches_serde() {
        for p in [
            OrchPhase::Idle,
            OrchPhase::Planning,
            OrchPhase::FeasibilityProbe,
            OrchPhase::TestingFirst,
            OrchPhase::Executing,
            OrchPhase::Auditing,
            OrchPhase::Memorizing,
            OrchPhase::Researching,
            OrchPhase::Configuring,
            OrchPhase::Complete,
        ] {
            let json = serde_json::to_string(&p).expect("ser");
            let stripped = json.trim_matches('"');
            assert_eq!(stripped, p.as_str());
        }
    }
}
