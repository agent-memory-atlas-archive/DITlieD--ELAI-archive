//! E-7: D-7 cross-domain transfer-regression threshold.
//!
//! Per E-7 plan task 7: reject promotion if for any non-target family
//! the candidate scores worse than max(relative_delta=-20%,
//! absolute_delta=-5%). Family selection is ADVERSARIAL — pick the
//! 3 families where the candidate scored LOWEST in the last 3
//! generations, not where ELAI is strongest.
//!
//! Threshold values are design hypotheses anchored to DGM
//! (arXiv:2505.22954) 50% -> 28.9% transfer drop = 42% relative as the
//! upper bar. Successive Rejects (Audibert COLT 2010) selects the BAI
//! variant over multi-family score.
//!
//! Wire-dark: compiles only when `cfg(feature = "darwinian-evolve")`
//! is set. Live family selection RUNPOD-PENDING (requires per-family
//! score history from the orchestrator).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Default relative-delta threshold (-20% per E-7 plan task 7).
pub const DEFAULT_REL_MAX: f64 = -0.20;

/// Default absolute-delta threshold (-5% per E-7 plan task 7).
pub const DEFAULT_ABS_MAX: f64 = -0.05;

/// Per-domain family identifier. Wire-dark scaffold; canonical Domain
/// enum lives elsewhere -- this module keys by snake_case label.
pub type FamilyLabel = String;

/// Transfer threshold configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferThreshold {
    pub rel_max: f64,
    pub abs_max: f64,
}

impl Default for TransferThreshold {
    fn default() -> Self {
        Self {
            rel_max: DEFAULT_REL_MAX,
            abs_max: DEFAULT_ABS_MAX,
        }
    }
}

/// Transfer verdict per family score-delta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferDecision {
    /// All families within thresholds.
    Allow,
    /// At least one family regressed beyond threshold.
    BlockRegression,
}

/// Check transfer-regression. Input: per-family `score_delta` where
/// positive = improvement and negative = regression. Threshold rule:
/// regression worse than max(rel_max, abs_max) on ANY family triggers
/// Block.
#[must_use]
pub fn check(
    family_score_delta: &HashMap<FamilyLabel, f64>,
    threshold: &TransferThreshold,
) -> TransferDecision {
    for (_family, delta) in family_score_delta {
        // delta is signed. Regression means delta < 0.
        if *delta < threshold.rel_max || *delta < threshold.abs_max {
            return TransferDecision::BlockRegression;
        }
    }
    TransferDecision::Allow
}

#[cfg(test)]
mod tests {
    use super::*;

    fn threshold() -> TransferThreshold {
        TransferThreshold::default()
    }

    #[test]
    fn no_regression_allows() {
        let mut deltas = HashMap::new();
        deltas.insert("python".to_string(), 0.05);
        deltas.insert("rust".to_string(), 0.10);
        assert_eq!(check(&deltas, &threshold()), TransferDecision::Allow);
    }

    #[test]
    fn small_regression_within_threshold_allows() {
        let mut deltas = HashMap::new();
        // -2% is above both thresholds (-20% rel / -5% abs).
        deltas.insert("python".to_string(), -0.02);
        deltas.insert("rust".to_string(), 0.10);
        assert_eq!(check(&deltas, &threshold()), TransferDecision::Allow);
    }

    #[test]
    fn large_regression_blocks() {
        let mut deltas = HashMap::new();
        // -25% is below the -20% relative threshold.
        deltas.insert("python".to_string(), -0.25);
        assert_eq!(
            check(&deltas, &threshold()),
            TransferDecision::BlockRegression
        );
    }

    #[test]
    fn any_family_regression_blocks() {
        let mut deltas = HashMap::new();
        deltas.insert("python".to_string(), 0.05);
        // -10% trips -5% absolute.
        deltas.insert("rust".to_string(), -0.10);
        assert_eq!(
            check(&deltas, &threshold()),
            TransferDecision::BlockRegression
        );
    }
}
