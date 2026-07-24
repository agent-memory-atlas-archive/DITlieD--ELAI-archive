//! J-38 Unit 1: deterministic context-sufficiency gate.
//!
//! Computes whether the retrieved context symbols cover the intent symbols
//! at a sufficient coverage ratio (default 80%). Used by task_runner to
//! gate dispatch when context is insufficient.

use std::collections::HashSet;

/// Maximum number of retrieval rounds allowed before the gate blocks.
pub const MAX_RETRIEVAL_ROUNDS: usize = 3;

/// Coverage threshold for sufficiency (0.8 = 80%).
const COVERAGE_THRESHOLD: f64 = 0.8;

/// Report of context sufficiency analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct SufficiencyReport {
    /// Ratio of intent symbols covered by context (0.0 to 1.0).
    pub coverage_ratio: f64,
    /// Threshold used for the sufficiency check.
    pub threshold: f64,
    /// Whether coverage meets or exceeds the threshold.
    pub is_sufficient: bool,
    /// Intent symbols not present in the context.
    pub missing: Vec<String>,
}

impl SufficiencyReport {
    /// Create a new sufficiency report from the given coverage data.
    fn new(coverage_ratio: f64, threshold: f64, missing: Vec<String>) -> Self {
        let is_sufficient = coverage_ratio >= threshold;
        Self {
            coverage_ratio,
            threshold,
            is_sufficient,
            missing,
        }
    }
}

/// Compute context sufficiency by comparing intent symbols against context symbols.
///
/// # Arguments
/// * `intent_symbols` - Symbols required by the intent (e.g., from IntentPair or task spec).
/// * `context_symbols` - Symbols present in the retrieved context (e.g., from ContextBudget slots).
///
/// # Returns
/// A `SufficiencyReport` containing coverage ratio, threshold, sufficiency flag, and missing symbols.
///
/// # Algorithm
/// 1. Convert both symbol lists to HashSets for O(1) lookup.
/// 2. Compute missing = intent_symbols not in context_symbols.
/// 3. Compute coverage_ratio = 1.0 - (missing.len() / intent_symbols.len()).
/// 4. is_sufficient = coverage_ratio >= 0.8.
///
/// # Edge cases
/// - Empty intent_symbols: coverage_ratio = 1.0 (vacuously sufficient).
/// - Empty context_symbols: coverage_ratio = 0.0 (all intent symbols missing).
pub fn compute_context_sufficiency(
    intent_symbols: &[String],
    context_symbols: &[String],
) -> SufficiencyReport {
    let intent_set: HashSet<&String> = intent_symbols.iter().collect();
    let context_set: HashSet<&String> = context_symbols.iter().collect();

    let missing: Vec<String> = intent_symbols
        .iter()
        .filter(|sym| !context_set.contains(sym))
        .cloned()
        .collect();

    let coverage_ratio = if intent_symbols.is_empty() {
        1.0
    } else {
        1.0 - (missing.len() as f64 / intent_symbols.len() as f64)
    };

    SufficiencyReport::new(coverage_ratio, COVERAGE_THRESHOLD, missing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_coverage_is_sufficient() {
        let intent = vec!["foo".to_string(), "bar".to_string()];
        let context = vec!["foo".to_string(), "bar".to_string()];
        let report = compute_context_sufficiency(&intent, &context);
        assert_eq!(report.coverage_ratio, 1.0);
        assert!(report.is_sufficient);
        assert!(report.missing.is_empty());
    }

    #[test]
    fn partial_coverage_below_threshold() {
        let intent = vec!["foo".to_string(), "bar".to_string(), "baz".to_string()];
        let context = vec!["foo".to_string()];
        let report = compute_context_sufficiency(&intent, &context);
        assert_eq!(report.coverage_ratio, 1.0 / 3.0);
        assert!(!report.is_sufficient);
        assert_eq!(report.missing.len(), 2);
    }

    #[test]
    fn partial_coverage_at_threshold() {
        let intent = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];
        let context = vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()];
        let report = compute_context_sufficiency(&intent, &context);
        assert_eq!(report.coverage_ratio, 0.8);
        assert!(report.is_sufficient);
        assert_eq!(report.missing.len(), 1);
    }

    #[test]
    fn empty_intent_is_sufficient() {
        let intent: Vec<String> = vec![];
        let context = vec!["foo".to_string()];
        let report = compute_context_sufficiency(&intent, &context);
        assert_eq!(report.coverage_ratio, 1.0);
        assert!(report.is_sufficient);
        assert!(report.missing.is_empty());
    }

    #[test]
    fn empty_context_yields_zero_coverage() {
        let intent = vec!["foo".to_string(), "bar".to_string()];
        let context: Vec<String> = vec![];
        let report = compute_context_sufficiency(&intent, &context);
        assert_eq!(report.coverage_ratio, 0.0);
        assert!(!report.is_sufficient);
        assert_eq!(report.missing.len(), 2);
    }

    #[test]
    fn threshold_is_constant() {
        let intent = vec!["foo".to_string()];
        let context = vec![];
        let report = compute_context_sufficiency(&intent, &context);
        assert_eq!(report.threshold, 0.8);
    }
}
