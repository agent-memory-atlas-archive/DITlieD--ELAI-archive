//! E-29 Tier-B -- Laminar-style natural-language signal matching.
//!
//! User writes a free-text query (e.g. "agent asked user for
//! clarification and got useful answer") -- an LLM classifier writes
//! `signal_match` rows backfilled across SSB. The classifier itself
//! lives outside this module (the per-role runtime is the dispatch
//! surface); this module owns the payload shape + the deterministic
//! sweep helper that walks the candidate row set.
//!
//! Wire-dark: emission gated by the parent
//! [`super::ELAI_AGENTIC_OBSERVABILITY_ENABLE_VAR`].
//!
//! Grounding: laminar.sh/article/2026-04-23-top-6-agent-observability-
//! platforms -- Signals are Laminar's flagship feature; the user-driven
//! free-text query + LLM-side backfill is the SOTA "show me X across
//! all traces" UX in the 2026 agent-observability space.

use serde::{Deserialize, Serialize};

use crate::ssb_viewer::{SsbViewer, SsbViewerError};

/// One signal-match hit recorded against an SSB row.
///
/// `evidence_row_ids` is the list of SSB seq ids the classifier matched
/// against. The dashboard renders these as clickable jump-links into
/// the TranscriptDrawer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalMatchHit {
    /// Human-readable signal label (e.g. `"clarification_resolved"`).
    pub signal_label: String,
    /// SSB row seq ids that matched.
    pub evidence_row_ids: Vec<i64>,
    /// Classifier model id (e.g. `"qwen-2.5-coder-3b"`).
    pub classifier_model: String,
    /// Confidence in basis points (0..=10_000).
    pub confidence_bp: u16,
    /// Session id this backfill ran against.
    pub session_id: String,
}

/// Errors produced by the candidate-row sweep helper.
#[derive(Debug, thiserror::Error)]
pub enum SignalSweepError {
    #[error("ssb viewer error: {0}")]
    Viewer(#[from] SsbViewerError),
}

/// Maximum number of candidate rows the sweep returns per action_type.
/// Matches the classifier's per-batch input ceiling.
pub const SIGNAL_SWEEP_PER_TYPE_CAP: usize = 5_000;

/// Sweep candidate SSB rows for a signal-match backfill.
///
/// The classifier itself is OUT OF SCOPE for this Tier-B slice (gated
/// behind the per-role runtime + an explicit env opt-in). What this
/// helper does:
///
/// 1. Pulls the recent N rows of each candidate action_type
///    (`finding`, `phase_transition`, `tool_result`) via
///    [`SsbViewer::recent_rows_of_type`].
/// 2. Returns the merged set, sorted by seq descending, so the caller
///    can stream them into the classifier in temporal order.
///
/// This is the deterministic backfill PRIMITIVE -- the classifier
/// wrapping is the caller's responsibility (Tauri command
/// `ssb_backfill_signals` constructs it on top of the per-role
/// runtime). Wire-dark default: no SSB row is emitted by this helper
/// itself; the classifier owner emits the `signal_match` row per hit
/// once the LLM call returns.
pub fn backfill_signal_matches(
    viewer: &SsbViewer,
    candidate_action_types: &[&str],
) -> Result<Vec<crate::ssb_viewer::SsbActionDetail>, SignalSweepError> {
    let mut merged = Vec::new();
    for at in candidate_action_types {
        let rows = viewer.recent_rows_of_type(at, SIGNAL_SWEEP_PER_TYPE_CAP)?;
        merged.extend(rows);
    }
    merged.sort_by(|a, b| b.seq.cmp(&a.seq));
    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_hit_serializes() {
        let hit = SignalMatchHit {
            signal_label: "clarification_resolved".into(),
            evidence_row_ids: vec![10, 11, 12],
            classifier_model: "qwen-3b".into(),
            confidence_bp: 8_500,
            session_id: "s-test".into(),
        };
        let json = serde_json::to_string(&hit).expect("serialize");
        let back: SignalMatchHit = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(hit, back);
    }
}
