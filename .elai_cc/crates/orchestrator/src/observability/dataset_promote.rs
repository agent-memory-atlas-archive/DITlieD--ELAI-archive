//! E-29 Tier-B -- promote production traces to eval datasets
//! (Langfuse / LangSmith pattern).
//!
//! `promote_traces_to_dataset(trace_ids, dataset_name)` records an
//! `EvalDatasetMember` SSB row per trace, tagging the trace as a
//! member of the named eval dataset. The optional CI-gate via
//! `ELAI_EVAL_GATE_REGRESSION_THRESHOLD` is DOCUMENTED but NOT
//! ENFORCED in this slice (deferred to a follow-up plan -- the
//! E-29-decisions.md Tier-B section lists this as RUNPOD-PENDING-A).
//!
//! Wire-dark: emission gated by the parent
//! [`super::ELAI_AGENTIC_OBSERVABILITY_ENABLE_VAR`].
//!
//! Grounding: github.com/langfuse/langfuse -- "traces become datasets"
//! is Langfuse's promote workflow. Self-host + LLM-as-judge -- the
//! E-29-decisions.md Tier-B section documents why ELAI's surface
//! tracks the on-disk SsbViewer rather than a Langfuse-shaped server.

use serde::{Deserialize, Serialize};

/// One eval-dataset member row. Stored as the per-trace payload of an
/// `AgenticObservabilityEventKind::EvalDatasetMember` SSB row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalDatasetMember {
    /// SSB trace id (sequence number or correlation hash) this member
    /// refers to.
    pub trace_id: i64,
    /// Dataset name (free-text, e.g. `"regression_2026_q2"`).
    pub dataset_name: String,
    /// Optional label (e.g. `"hard"`, `"sycophancy_probe"`).
    pub label: String,
    /// Scoring rubric -- one of `"binary_pass"`, `"likert_5"`,
    /// `"rubric_v1"` for now. Stored as plain string so consumers can
    /// extend without a schema bump.
    pub scoring_rubric: String,
}

/// Aggregate result of [`promote_traces_to_dataset`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalDatasetPromotion {
    /// Dataset the trace ids were promoted into.
    pub dataset_name: String,
    /// Per-trace member rows. Sorted ascending by `trace_id` for
    /// stable PR diffs.
    pub members: Vec<EvalDatasetMember>,
    /// Number of trace ids in the input set that were skipped because
    /// of de-duplication against an existing member row (best-effort;
    /// caller passes the prior member set explicitly).
    pub skipped_duplicates: usize,
}

/// Errors produced by the promotion helper.
#[derive(Debug, thiserror::Error)]
pub enum PromotionError {
    #[error("dataset name must be non-empty")]
    EmptyDatasetName,
    #[error("trace id list must be non-empty")]
    EmptyTraceIds,
    #[error("scoring rubric '{0}' not supported (use binary_pass / likert_5 / rubric_v1)")]
    UnsupportedScoringRubric(String),
}

/// Accepted scoring-rubric tags. Extending this list is an additive
/// change -- the dashboard surfaces the tag as a plain string.
pub const ACCEPTED_RUBRICS: &[&str] = &["binary_pass", "likert_5", "rubric_v1"];

/// Promote a set of trace ids to a named eval dataset.
///
/// Builds the [`EvalDatasetMember`] rows in canonical order. Skips
/// trace ids that already appear in `prior_members` (de-dup against
/// the existing on-disk dataset). The caller is responsible for
/// emitting the SSB rows (the wire-dark gate is checked by the Tauri
/// command).
pub fn promote_traces_to_dataset(
    trace_ids: &[i64],
    dataset_name: &str,
    label: &str,
    scoring_rubric: &str,
    prior_members: &[i64],
) -> Result<EvalDatasetPromotion, PromotionError> {
    if dataset_name.is_empty() {
        return Err(PromotionError::EmptyDatasetName);
    }
    if trace_ids.is_empty() {
        return Err(PromotionError::EmptyTraceIds);
    }
    if !ACCEPTED_RUBRICS.contains(&scoring_rubric) {
        return Err(PromotionError::UnsupportedScoringRubric(
            scoring_rubric.to_string(),
        ));
    }

    let mut members: Vec<EvalDatasetMember> = Vec::new();
    let mut sorted_ids: Vec<i64> = trace_ids.to_vec();
    sorted_ids.sort_unstable();
    sorted_ids.dedup();

    let mut skipped = 0usize;
    for tid in &sorted_ids {
        if prior_members.contains(tid) {
            skipped += 1;
            continue;
        }
        members.push(EvalDatasetMember {
            trace_id: *tid,
            dataset_name: dataset_name.to_string(),
            label: label.to_string(),
            scoring_rubric: scoring_rubric.to_string(),
        });
    }

    Ok(EvalDatasetPromotion {
        dataset_name: dataset_name.to_string(),
        members,
        skipped_duplicates: skipped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promotes_unique_ids() {
        let r = promote_traces_to_dataset(
            &[3, 1, 2, 1],
            "regression_2026",
            "hard",
            "binary_pass",
            &[],
        )
        .expect("ok");
        assert_eq!(r.members.len(), 3);
        assert_eq!(r.members[0].trace_id, 1);
        assert_eq!(r.members[1].trace_id, 2);
        assert_eq!(r.members[2].trace_id, 3);
        assert_eq!(r.skipped_duplicates, 0);
    }

    #[test]
    fn skips_prior_members() {
        let r = promote_traces_to_dataset(
            &[1, 2, 3, 4],
            "regression_2026",
            "hard",
            "likert_5",
            &[2, 4],
        )
        .expect("ok");
        assert_eq!(r.members.len(), 2);
        assert_eq!(r.skipped_duplicates, 2);
    }

    #[test]
    fn rejects_empty_name() {
        let r = promote_traces_to_dataset(&[1], "", "", "binary_pass", &[]);
        assert!(matches!(r, Err(PromotionError::EmptyDatasetName)));
    }

    #[test]
    fn rejects_empty_trace_ids() {
        let r = promote_traces_to_dataset(&[], "ds", "", "binary_pass", &[]);
        assert!(matches!(r, Err(PromotionError::EmptyTraceIds)));
    }

    #[test]
    fn rejects_bad_rubric() {
        let r = promote_traces_to_dataset(&[1], "ds", "", "made_up", &[]);
        assert!(matches!(r, Err(PromotionError::UnsupportedScoringRubric(_))));
    }
}
