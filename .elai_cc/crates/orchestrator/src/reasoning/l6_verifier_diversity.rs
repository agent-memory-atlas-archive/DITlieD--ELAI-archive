//! E-19 task 7 — L6 verifier diversity (fresh-seeded L1 + rotated L3).
//!
//! Paper grounding: arXiv:2502.08788 (Stop Overvaluing Multi-Agent Debate —
//! diversity / heterogeneity matters; same-model re-runs are insufficient).
//! Note the E-19 grounding triage flagged the original "diversity +
//! calibration" framing as WRONG_REGIME-minor: the paper's recommendation is
//! MODEL HETEROGENEITY, not confidence calibration. ELAI's L6 diversity
//! probe uses the same model with different (a) RNG seed and (b)
//! InterrogationTree question set — this is the closest local-pipeline
//! approximation, and D-13 (Adversary role, cross-family probe) is the
//! orthogonal model-heterogeneity axis.
//!
//! IO-free at the data-type level. The orchestrator wrapper supplies the
//! actual L1 / L3 results; this module decides agreement and emits the
//! verdict.

use serde::{Deserialize, Serialize};

/// Outcome of the verifier-diversity probe.
///
/// `Pass` is the green-light state (both fresh-seed L1 and rotated L3 agree
/// with the L6 output). `Disagree` is the red-flag state (at least one
/// disagrees); the caller MUST NOT promote the L6 output until disagreement
/// is resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiversityVerdict {
    Pass,
    Disagree,
}

impl DiversityVerdict {
    /// Snake-case wire tag.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Disagree => "disagree",
        }
    }
}

/// Pure-function verifier-diversity rule.
///
/// `fresh_seed_l1_agrees` is true when a fresh-seeded L1 pass produces an
/// output consistent with the L6 output (string-equal answers OR
/// AST-equivalent code, decided by the caller).
///
/// `rotated_l3_agrees` is true when a different L3 question set produces
/// an output consistent with the L6 output.
///
/// Both must hold for `Pass`; either failure yields `Disagree`.
#[must_use]
pub fn verifier_diversity(fresh_seed_l1_agrees: bool, rotated_l3_agrees: bool) -> DiversityVerdict {
    if fresh_seed_l1_agrees && rotated_l3_agrees {
        DiversityVerdict::Pass
    } else {
        DiversityVerdict::Disagree
    }
}

/// Payload-friendly bundle for the SSB
/// `ReasoningQualityProbe { kind: ProbeKind::L6VerifierDiversity }` row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiversityProbe {
    /// L1 fresh-seed agreement boolean.
    pub fresh_seed_l1_agrees: bool,
    /// L3 rotated-question-set agreement boolean.
    pub rotated_l3_agrees: bool,
    /// Computed verdict.
    pub verdict: DiversityVerdict,
    /// Short reason slug — e.g. "fresh_seed_disagreed" / "rotated_l3_disagreed"
    /// / "both_disagreed" / "both_agreed". Snake_case.
    pub reason: String,
}

impl DiversityProbe {
    /// Compute the probe payload from the two agreement booleans.
    #[must_use]
    pub fn compute(fresh_seed_l1_agrees: bool, rotated_l3_agrees: bool) -> Self {
        let verdict = verifier_diversity(fresh_seed_l1_agrees, rotated_l3_agrees);
        let reason = match (fresh_seed_l1_agrees, rotated_l3_agrees) {
            (true, true) => "both_agreed",
            (false, true) => "fresh_seed_disagreed",
            (true, false) => "rotated_l3_disagreed",
            (false, false) => "both_disagreed",
        }
        .to_string();
        Self {
            fresh_seed_l1_agrees,
            rotated_l3_agrees,
            verdict,
            reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_agree_passes() {
        let p = DiversityProbe::compute(true, true);
        assert_eq!(p.verdict, DiversityVerdict::Pass);
        assert_eq!(p.reason, "both_agreed");
    }

    #[test]
    fn fresh_seed_disagrees_blocks() {
        let p = DiversityProbe::compute(false, true);
        assert_eq!(p.verdict, DiversityVerdict::Disagree);
        assert_eq!(p.reason, "fresh_seed_disagreed");
    }

    #[test]
    fn rotated_l3_disagrees_blocks() {
        let p = DiversityProbe::compute(true, false);
        assert_eq!(p.verdict, DiversityVerdict::Disagree);
        assert_eq!(p.reason, "rotated_l3_disagreed");
    }

    #[test]
    fn both_disagree_blocks() {
        let p = DiversityProbe::compute(false, false);
        assert_eq!(p.verdict, DiversityVerdict::Disagree);
        assert_eq!(p.reason, "both_disagreed");
    }

    #[test]
    fn verdict_wire_tags() {
        assert_eq!(DiversityVerdict::Pass.as_str(), "pass");
        assert_eq!(DiversityVerdict::Disagree.as_str(), "disagree");
    }
}
