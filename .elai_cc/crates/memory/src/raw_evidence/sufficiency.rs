//! Deterministic sufficiency gate for the authorized fact layer.
//!
//! See [`super`] for the architectural overview and grounding.

use std::fmt;

/// The deterministic sufficiency verdict for the authorized fact layer.
/// When `Insufficient`, the raw-evidence tier is queried as a fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SufficiencyVerdict {
    /// The authorized fact layer is sufficient — no escalation needed.
    Sufficient,
    /// The authorized fact layer is insufficient — escalate to raw evidence.
    /// The carried reason is the deterministic trigger condition.
    Insufficient(SufficiencyReason),
}

/// The deterministic reason the fact layer was deemed insufficient.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SufficiencyReason {
    /// Zero authorized facts AND at least one candidate was excluded —
    /// facts existed for the query but every one was rejected by the
    /// retrieval-time authorization gate.
    AllExcluded,
    /// Zero authorized facts AND zero candidates — the query returned no
    /// facts at all (the fact layer has nothing for this query).
    NoCandidates,
}

impl fmt::Display for SufficiencyReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AllExcluded => f.write_str("all_excluded"),
            Self::NoCandidates => f.write_str("no_candidates"),
        }
    }
}

impl fmt::Display for SufficiencyVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sufficient => f.write_str("sufficient"),
            Self::Insufficient(r) => write!(f, "insufficient:{}", r),
        }
    }
}

/// Assess whether the authorized fact layer is sufficient for the query.
///
/// **Deterministic sufficiency gate**: the fact layer is `Insufficient`
/// when zero authorized facts were returned. The `reason` distinguishes
/// `AllExcluded` (candidates existed but were all rejected — the raw
/// evidence tier may carry the dropped detail) from `NoCandidates` (no
/// facts existed at all — the raw evidence tier is the only source).
///
/// When `authorized_count > 0`, the verdict is always `Sufficient` — the
/// fact layer has at least one valid, authorized fact for the query.
pub fn assess_sufficiency(
    authorized_count: usize,
    exclusion_count: usize,
) -> SufficiencyVerdict {
    if authorized_count > 0 {
        return SufficiencyVerdict::Sufficient;
    }
    if exclusion_count > 0 {
        SufficiencyVerdict::Insufficient(SufficiencyReason::AllExcluded)
    } else {
        SufficiencyVerdict::Insufficient(SufficiencyReason::NoCandidates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sufficiency_sufficient_when_authorized_nonzero() {
        assert_eq!(assess_sufficiency(1, 0), SufficiencyVerdict::Sufficient);
        assert_eq!(assess_sufficiency(3, 5), SufficiencyVerdict::Sufficient);
    }

    #[test]
    fn sufficiency_all_excluded_when_zero_authorized_with_exclusions() {
        assert_eq!(
            assess_sufficiency(0, 3),
            SufficiencyVerdict::Insufficient(SufficiencyReason::AllExcluded)
        );
    }

    #[test]
    fn sufficiency_no_candidates_when_zero_authorized_zero_exclusions() {
        assert_eq!(
            assess_sufficiency(0, 0),
            SufficiencyVerdict::Insufficient(SufficiencyReason::NoCandidates)
        );
    }

    #[test]
    fn sufficiency_verdict_display() {
        assert_eq!(SufficiencyVerdict::Sufficient.to_string(), "sufficient");
        assert_eq!(
            SufficiencyVerdict::Insufficient(SufficiencyReason::AllExcluded).to_string(),
            "insufficient:all_excluded"
        );
        assert_eq!(
            SufficiencyVerdict::Insufficient(SufficiencyReason::NoCandidates).to_string(),
            "insufficient:no_candidates"
        );
    }
}
