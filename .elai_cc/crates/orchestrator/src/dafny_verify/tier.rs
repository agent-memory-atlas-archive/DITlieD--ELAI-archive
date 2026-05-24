//! C-52 — Dafny prompting tier per arXiv:2604.22601 RQ1/RQ2/RQ3.
//!
//! Three tiers in order of increasing scaffolding:
//!   - `Contextless` (RQ1): only the natural-language problem. The
//!     paper reports near-universal failure (6 of 7 models at 0%
//!     verify@5; Codestral-22B 27.27%, Gemma 4-31B 54.55%).
//!   - `Signature` (RQ2): NL problem + Dafny method signature with
//!     declared preconditions / postconditions. The paper's
//!     strongest non-self-healing tier.
//!   - `SelfHealing` (RQ3): NL problem + signature + Dafny verifier
//!     feedback from a prior attempt. Iterates up to `MAX_HEAL_ITER`
//!     rounds.
//!
//! Wire-tag string is snake_case; matches the paper's `tier` field
//! in result tables.

use serde::{Deserialize, Serialize};

/// Per-attempt prompting tier. Three variants tracking
/// arXiv:2604.22601 RQ1 / RQ2 / RQ3.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
pub enum DafnyTier {
    /// RQ1 — natural-language problem only. No signature, no
    /// scaffolding. Paper baseline.
    Contextless,
    /// RQ2 — NL problem + Dafny method signature with declared
    /// preconditions / postconditions. The minimum viable tier
    /// for any non-trivial Dafny generation per the paper.
    Signature,
    /// RQ3 — NL problem + signature + prior verifier feedback.
    /// Drives the self-healing loop.
    SelfHealing,
}

impl DafnyTier {
    /// Snake-case wire tag. Matches paper result-table convention.
    pub fn as_str(self) -> &'static str {
        match self {
            DafnyTier::Contextless => "contextless",
            DafnyTier::Signature => "signature",
            DafnyTier::SelfHealing => "self_healing",
        }
    }

    /// Parse the snake-case wire tag back to the variant.
    pub fn from_str(s: &str) -> Option<DafnyTier> {
        match s {
            "contextless" => Some(DafnyTier::Contextless),
            "signature" => Some(DafnyTier::Signature),
            "self_healing" => Some(DafnyTier::SelfHealing),
            _ => None,
        }
    }

    /// Canonical iteration order (RQ1 -> RQ2 -> RQ3). Used by the
    /// contract test G1 and the CLI `list-tiers` surface.
    pub const ALL: [DafnyTier; 3] = [
        DafnyTier::Contextless,
        DafnyTier::Signature,
        DafnyTier::SelfHealing,
    ];

    /// Whether the tier requires a method signature input. Only the
    /// Contextless tier omits the signature; the other two REQUIRE it
    /// per paper Section 3 (mandatory for RQ2/RQ3).
    pub fn requires_signature(self) -> bool {
        match self {
            DafnyTier::Contextless => false,
            DafnyTier::Signature | DafnyTier::SelfHealing => true,
        }
    }

    /// Whether the tier requires prior-attempt verifier feedback as
    /// input. Only SelfHealing requires this.
    pub fn requires_prior_feedback(self) -> bool {
        matches!(self, DafnyTier::SelfHealing)
    }
}

impl std::fmt::Display for DafnyTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_round_trip() {
        for t in DafnyTier::ALL {
            let s = t.as_str();
            let back = DafnyTier::from_str(s).expect("from_str");
            assert_eq!(back, t);
        }
    }

    #[test]
    fn tier_count_matches_paper_rq_count() {
        // Paper RQ1 + RQ2 + RQ3 = 3 distinct tiers.
        assert_eq!(DafnyTier::ALL.len(), 3);
    }

    #[test]
    fn signature_required_by_rq2_rq3() {
        assert!(!DafnyTier::Contextless.requires_signature());
        assert!(DafnyTier::Signature.requires_signature());
        assert!(DafnyTier::SelfHealing.requires_signature());
    }

    #[test]
    fn prior_feedback_required_only_by_rq3() {
        assert!(!DafnyTier::Contextless.requires_prior_feedback());
        assert!(!DafnyTier::Signature.requires_prior_feedback());
        assert!(DafnyTier::SelfHealing.requires_prior_feedback());
    }
}
