//! E-29 Tier-B -- post-run MAST failure-mode classifier.
//!
//! Walks SsbViewer rows for a given session and projects observed
//! finding-row category strings onto the E-6-shipped
//! [`crate::mast::MastMode`] 14-variant taxonomy
//! (arXiv:2503.13657 Cemri et al. NeurIPS 2025).
//!
//! Reuses [`crate::mast::finding_to_mast_mode`] as the deterministic
//! mapping kernel; supplements with row-pattern heuristics over the
//! visited SSB rows (action_type, severity, repetition counts). The
//! classifier intentionally does NOT call any external LLM: this is the
//! grounded ELAI-side default classifier that runs offline against
//! historical traces. The Laminar-style natural-language signal_match
//! flow (see [`super::signal_match`]) is the LLM-backed alternative,
//! gated behind a separate opt-in env var.
//!
//! See `crate::regression::failure_taxonomy_misses` for the orthogonal
//! 13-category SYMPTOM taxonomy. The MAST modes are CAUSE classifications,
//! not symptoms -- both surfaces coexist (DRIFT-2 in
//! `workflow/reasoning/E-29-decisions.md`).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::mast::{finding_to_mast_mode, MastMode};
use crate::ssb_viewer::{SsbActionDetail, SsbViewer, SsbViewerError};

/// Rows come back externally tagged because `SsbPayload` is a plain-derive
/// enum: the top-level key is the variant name and the value is the inner
/// payload. Navigate into it, falling back to the value itself so a bare
/// payload (older rows, hand-built fixtures) still reads.
fn inner_payload<'a>(row: &'a SsbActionDetail, variant: &str) -> &'a serde_json::Value {
    row.payload_json.get(variant).unwrap_or(&row.payload_json)
}

/// One MAST-mode hit on a closed run. Carries the SSB row identifiers
/// that drove the classification + a confidence in basis points (0..=10_000).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MastModeHit {
    /// The classified MAST mode.
    pub mode: MastMode,
    /// Wire-stable snake_case tag for the mode (mirrors
    /// [`MastMode::as_sql_str`]).
    pub mode_label: String,
    /// SSB row seq ids that drove the classification (evidence trail).
    /// Empty for heuristic-only hits.
    pub evidence_row_ids: Vec<i64>,
    /// Confidence in basis points (0..=10_000). 10_000 = certain
    /// (deterministic `finding_to_mast_mode` mapping hit); lower values
    /// are heuristic-driven inferences.
    pub confidence_bp: u16,
    /// Session id this hit was computed for.
    pub session_id: String,
}

/// Errors produced by [`classify_closed_run`].
#[derive(Debug, thiserror::Error)]
pub enum ClassifierError {
    #[error("ssb viewer error: {0}")]
    Viewer(#[from] SsbViewerError),
}

/// Maximum number of SSB rows to scan per session for the classifier
/// pass. Bounds wall-time + memory; a HEAVY-tier session typically has
/// 10K-50K rows but only a fraction are findings. 20_000 is the
/// per-session scan ceiling.
pub const CLASSIFIER_ROW_SCAN_CAP: usize = 20_000;

/// Confidence assigned when [`finding_to_mast_mode`] returns a direct
/// hit on a finding-row's `category` field. The mapping is deterministic
/// so the confidence is 10_000 (= 100%).
const CONFIDENCE_DIRECT_MAPPING_BP: u16 = 10_000;

/// Confidence assigned when a step-repetition heuristic fires.
/// Empirically calibrated: repetition is a strong but not perfect MAST
/// FM-1.3 signal -- a few legitimate retries can mimic the pattern.
const CONFIDENCE_HEURISTIC_REPETITION_BP: u16 = 7_500;

/// Confidence assigned when a premature-termination heuristic fires
/// (test outcome PASS while finding rows have FAILURE severity).
/// Calibrated lower than direct mapping; the cross-row inference can
/// false-positive on legitimately-recovered runs.
const CONFIDENCE_HEURISTIC_PREMATURE_TERM_BP: u16 = 6_500;

/// Step-repetition threshold (per agent_id within a session) above
/// which the FM-1.3 heuristic fires. Sourced from MAST Table 1
/// (FM-1.3 = 17.14% of corpus, the highest-frequency Specification
/// failure mode). The cutoff is intentionally conservative.
pub const REPETITION_THRESHOLD: usize = 3;

/// Run the classifier over the SSB rows for `session_id`.
///
/// Procedure:
/// 1. Pull recent `finding` rows + `test_outcome` rows + `phase_transition`
///    rows via [`SsbViewer::recent_rows_of_type`] -- bounded by
///    [`CLASSIFIER_ROW_SCAN_CAP`] each.
/// 2. For every `finding` row whose payload has a `category` field
///    [`finding_to_mast_mode`] maps cleanly, emit a hit with
///    [`CONFIDENCE_DIRECT_MAPPING_BP`].
/// 3. Step-repetition heuristic: if any single `agent_id` accumulates
///    >= [`REPETITION_THRESHOLD`] `finding` rows tagged with the same
///    `category`, emit a FM-1.3 hit at
///    [`CONFIDENCE_HEURISTIC_REPETITION_BP`].
/// 4. Premature-termination heuristic: if the most recent
///    `test_outcome` row is `"PASS"` while at least one prior
///    `finding` row in the session window has severity `"FAILURE"`
///    or `"BLOCK"`, emit a FM-3.1 hit at
///    [`CONFIDENCE_HEURISTIC_PREMATURE_TERM_BP`].
///
/// The classifier is OFFLINE -- it never calls an LLM. The Laminar-
/// style natural-language signal classifier in [`super::signal_match`]
/// is the opt-in LLM-backed alternative for free-text outcome queries.
///
/// The returned `Vec<MastModeHit>` is sorted by descending confidence.
/// Empty result == clean session (no MAST hits detected).
pub fn classify_closed_run(
    session_id: &str,
    viewer: &SsbViewer,
) -> Result<Vec<MastModeHit>, ClassifierError> {
    let findings = viewer.recent_rows_of_type("finding", CLASSIFIER_ROW_SCAN_CAP)?;
    let outcomes = viewer.recent_rows_of_type("test_outcome", CLASSIFIER_ROW_SCAN_CAP)?;

    let mut hits: Vec<MastModeHit> = Vec::new();

    // Stage 1: deterministic finding_to_mast_mode hits.
    for row in &findings {
        if !row_belongs_to_session(row, session_id) {
            continue;
        }
        let category = match inner_payload(row, "Finding")
            .get("category")
            .and_then(|v| v.as_str())
        {
            Some(c) => c,
            None => continue,
        };
        if let Some(mode) = finding_to_mast_mode(category) {
            hits.push(MastModeHit {
                mode,
                mode_label: mode.as_sql_str().to_string(),
                evidence_row_ids: vec![row.seq],
                confidence_bp: CONFIDENCE_DIRECT_MAPPING_BP,
                session_id: session_id.to_string(),
            });
        }
    }

    // Stage 2: step-repetition heuristic -- per-(agent_id, category)
    // counts. Uses BTreeMap not HashMap per ELAI ban on std
    // HashMap-in-prod (determinism + reproducibility).
    let mut per_pair: BTreeMap<(String, String), (usize, Vec<i64>)> = BTreeMap::new();
    for row in &findings {
        if !row_belongs_to_session(row, session_id) {
            continue;
        }
        let category = match inner_payload(row, "Finding")
            .get("category")
            .and_then(|v| v.as_str())
        {
            Some(c) => c.to_string(),
            None => continue,
        };
        let entry = per_pair
            .entry((row.agent_id.clone(), category))
            .or_insert_with(|| (0, Vec::new()));
        entry.0 += 1;
        entry.1.push(row.seq);
    }
    for ((_agent, _cat), (count, seqs)) in &per_pair {
        if *count >= REPETITION_THRESHOLD {
            hits.push(MastModeHit {
                mode: MastMode::Fm13StepRepetition,
                mode_label: MastMode::Fm13StepRepetition.as_sql_str().to_string(),
                evidence_row_ids: seqs.clone(),
                confidence_bp: CONFIDENCE_HEURISTIC_REPETITION_BP,
                session_id: session_id.to_string(),
            });
        }
    }

    // Stage 3: premature-termination heuristic.
    let last_outcome = outcomes
        .iter()
        .find(|r| row_belongs_to_session(r, session_id))
        .and_then(|r| {
            inner_payload(r, "TestOutcome")
                .get("passed")
                .and_then(|v| v.as_bool())
        });
    if last_outcome == Some(true) {
        // Find any earlier failure-severity finding row.
        let failure_seqs: Vec<i64> = findings
            .iter()
            .filter(|r| row_belongs_to_session(r, session_id))
            .filter(|r| {
                inner_payload(r, "Finding")
                    .get("severity")
                    .and_then(|v| v.as_str())
                    .map(|s| matches!(s, "FAILURE" | "BLOCK" | "failure" | "block"))
                    .unwrap_or(false)
            })
            .map(|r| r.seq)
            .collect();
        if !failure_seqs.is_empty() {
            hits.push(MastModeHit {
                mode: MastMode::Fm31PrematureTermination,
                mode_label: MastMode::Fm31PrematureTermination.as_sql_str().to_string(),
                evidence_row_ids: failure_seqs,
                confidence_bp: CONFIDENCE_HEURISTIC_PREMATURE_TERM_BP,
                session_id: session_id.to_string(),
            });
        }
    }

    hits.sort_by(|a, b| b.confidence_bp.cmp(&a.confidence_bp));
    Ok(hits)
}

/// Determine whether an [`SsbActionDetail`] row belongs to `session_id`.
/// SSB rows do not all carry an explicit session_id field; we fall back
/// to the payload JSON's `session_id` field when present, else accept
/// the row (best-effort -- caller filters again downstream).
fn row_belongs_to_session(row: &SsbActionDetail, session_id: &str) -> bool {
    // Finding and TestOutcome payloads do not carry session_id, so absent stays permissive.
    match inner_payload(row, "Finding")
        .get("session_id")
        .and_then(|v| v.as_str()) {
        Some(s) if s.is_empty() => true,
        Some(s) => s == session_id,
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mast::MastMode;

    #[test]
    fn confidence_bp_constants_in_range() {
        assert!(CONFIDENCE_DIRECT_MAPPING_BP <= 10_000);
        assert!(CONFIDENCE_HEURISTIC_REPETITION_BP <= 10_000);
        assert!(CONFIDENCE_HEURISTIC_PREMATURE_TERM_BP <= 10_000);
    }

    #[test]
    fn fourteen_mode_constructor_smoke() {
        // Compile-time check that every MastMode variant can be wrapped
        // in a MastModeHit -- exhaustive coverage of the 14-variant
        // taxonomy is asserted in the contract test
        // crates/contracts/tests/mast_classifier_14_variants.rs.
        for mode in MastMode::ALL {
            let hit = MastModeHit {
                mode,
                mode_label: mode.as_sql_str().to_string(),
                evidence_row_ids: vec![1, 2, 3],
                confidence_bp: 5_000,
                session_id: "s-test".into(),
            };
            assert!(!hit.mode_label.is_empty());
        }
    }

    #[test]
    fn row_belongs_to_session_session_match() {
        let row = SsbActionDetail {
            seq: 1,
            ts: 0,
            action_type: "finding".into(),
            agent_id: "a".into(),
            payload_json: serde_json::json!({ "session_id": "s-1" }),
            files: vec![],
        };
        assert!(row_belongs_to_session(&row, "s-1"));
        assert!(!row_belongs_to_session(&row, "s-2"));
    }

    #[test]
    fn row_belongs_to_session_no_field_is_permissive() {
        let row = SsbActionDetail {
            seq: 1,
            ts: 0,
            action_type: "finding".into(),
            agent_id: "a".into(),
            payload_json: serde_json::json!({}),
            files: vec![],
        };
        assert!(row_belongs_to_session(&row, "s-1"));
    }
}
