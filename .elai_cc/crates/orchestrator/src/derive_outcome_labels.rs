//! J-93 LEG-F — non-LLM ground-truth labels from the SSB outcome stream.
//! Deterministic. No LLM. OBSERVE-ONLY under kill-switch.

/// Env kill-switch. Default OFF.
pub const ELAI_DERIVE_OUTCOME_LABELS_ENABLE: &str = "ELAI_DERIVE_OUTCOME_LABELS_ENABLE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutcomeLabel {
    TruePositive,
    FalsePositive,
    Unlabeled,
}

/// One labeled evaluation unit for a rule (pure data; no I/O).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcomeRecord {
    pub rule_id: String,
    pub blocked: bool,
    pub later_gates_all_pass: bool,
    pub shipped: bool,
    pub downstream_regression: bool,
    pub journal_complete: bool,
}

/// Aggregated counts for lifecycle MetricsSnapshot feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DerivedLabelCounts {
    pub true_positives: i64,
    pub false_positives: i64,
    pub unlabeled: i64,
}

fn truthy_env_value(value: &str) -> bool {
    matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
}

pub fn derive_outcome_labels_enabled() -> bool {
    std::env::var(ELAI_DERIVE_OUTCOME_LABELS_ENABLE)
        .map(|value| truthy_env_value(&value))
        .unwrap_or(false)
}

/// Pure classify (no env). Incomplete journal => Unlabeled always.
pub fn classify_outcome_record(rec: &OutcomeRecord) -> OutcomeLabel {
    if !rec.journal_complete || !rec.blocked {
        return OutcomeLabel::Unlabeled;
    }
    if rec.later_gates_all_pass && rec.shipped && !rec.downstream_regression {
        OutcomeLabel::FalsePositive
    } else {
        OutcomeLabel::TruePositive
    }
}

/// Pure aggregate (no env).
pub fn aggregate_labels(records: &[OutcomeRecord]) -> DerivedLabelCounts {
    let mut counts = DerivedLabelCounts::default();
    for record in records {
        match classify_outcome_record(record) {
            OutcomeLabel::TruePositive => counts.true_positives += 1,
            OutcomeLabel::FalsePositive => counts.false_positives += 1,
            OutcomeLabel::Unlabeled => counts.unlabeled += 1,
        }
    }
    counts
}

/// Env-gated: Disabled/zeros when flag OFF; else aggregate_labels.
pub fn derive_outcome_labels(records: &[OutcomeRecord]) -> DerivedLabelCounts {
    if !derive_outcome_labels_enabled() {
        return DerivedLabelCounts::default();
    }
    aggregate_labels(records)
}

/// Build a lifecycle MetricsSnapshot from derived counts.
pub fn metrics_snapshot_from_labels(
    counts: &DerivedLabelCounts,
) -> elai_rules::lifecycle::MetricsSnapshot {
    let tp = counts.true_positives;
    let fp = counts.false_positives;
    let total = tp + fp;
    let fpr = if total > 0 { fp as f64 / total as f64 } else { 0.0 };
    let tpr = if tp > 0 { 1.0 } else { 0.0 };
    elai_rules::lifecycle::MetricsSnapshot {
        true_positives: tp,
        false_positives: fp,
        causal_catches: 0,
        spurious_catches: 0,
        false_positive_rate: fpr,
        true_positive_rate: tpr,
        counterfactual_causal_rate: None,
    }
}

/// OBSERVE helper for the curator: log counts when ON; zeros when OFF.
pub fn observe_labels_for_rule(
    rule_id: &str,
    records: &[OutcomeRecord],
) -> elai_rules::lifecycle::MetricsSnapshot {
    let counts = derive_outcome_labels(records);
    if derive_outcome_labels_enabled() {
        tracing::info!(
            target: "derive_outcome_labels",
            rule_id = %rule_id,
            tp = counts.true_positives,
            fp = counts.false_positives,
            unlabeled = counts.unlabeled,
            "J-93 LEG-F outcome labels derived (non-LLM)"
        );
    }
    metrics_snapshot_from_labels(&counts)
}

#[cfg(test)]
mod j93_derive_outcome_labels_tests {
    use super::*;

    fn rec(blocked: bool, gates: bool, shipped: bool, reg: bool, complete: bool) -> OutcomeRecord {
        OutcomeRecord {
            rule_id: "r1".into(), blocked, later_gates_all_pass: gates, shipped,
            downstream_regression: reg, journal_complete: complete,
        }
    }

    #[test]
    fn incomplete_journal_is_unlabeled() {
        assert_eq!(classify_outcome_record(&rec(true, true, true, false, false)), OutcomeLabel::Unlabeled);
    }
    #[test]
    fn block_then_clean_ship_is_false_positive() {
        assert_eq!(classify_outcome_record(&rec(true, true, true, false, true)), OutcomeLabel::FalsePositive);
    }
    #[test]
    fn block_with_regression_is_true_positive() {
        assert_eq!(classify_outcome_record(&rec(true, true, true, true, true)), OutcomeLabel::TruePositive);
    }
    #[test]
    fn block_gates_fail_is_true_positive() {
        assert_eq!(classify_outcome_record(&rec(true, false, false, false, true)), OutcomeLabel::TruePositive);
    }
    #[test]
    fn non_block_is_unlabeled() {
        assert_eq!(classify_outcome_record(&rec(false, true, true, false, true)), OutcomeLabel::Unlabeled);
    }
    #[test]
    fn aggregate_counts_fp_and_tp() {
        let records = vec![rec(true, true, true, false, true), rec(true, false, false, false, true), rec(true, true, true, false, false)];
        let c = aggregate_labels(&records);
        assert_eq!(c.false_positives, 1);
        assert_eq!(c.true_positives, 1);
        assert_eq!(c.unlabeled, 1);
    }
    #[test]
    fn metrics_snapshot_rates() {
        let counts = DerivedLabelCounts { true_positives: 3, false_positives: 1, unlabeled: 0 };
        let m = metrics_snapshot_from_labels(&counts);
        assert!((m.false_positive_rate - 0.25).abs() < 1e-9);
        assert_eq!(m.true_positives, 3);
        assert_eq!(m.false_positives, 1);
    }
    #[test]
    fn flag_name_is_stable() { assert_eq!(ELAI_DERIVE_OUTCOME_LABELS_ENABLE, "ELAI_DERIVE_OUTCOME_LABELS_ENABLE"); }
    #[test]
    fn truthy_env_values_recognized() {
        assert!(truthy_env_value("1")); assert!(truthy_env_value("true")); assert!(truthy_env_value("YES"));
        assert!(truthy_env_value("on")); assert!(!truthy_env_value("0")); assert!(!truthy_env_value("")); assert!(!truthy_env_value("no"));
    }
}
