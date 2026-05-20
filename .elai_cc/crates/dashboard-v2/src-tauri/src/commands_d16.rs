//! D-16 — UniCR conformal-refusal calibration read-only Tauri command
//! surface.
//!
//! Two read-only commands feed the Svelte `ConformalCalibrationPanel`:
//!
//! - [`get_conformal_recent`] — last N joined
//!   `(conformal_tau_computed + conformal_refusal_decided)` rows for the
//!   panel's per-claim histogram and tau-over-time chart.
//! - [`get_calibration_corpus_state`] — a snapshot of the active
//!   calibration corpus state (size + source + epsilon). The MVP returns
//!   a static snapshot derived from
//!   `.elai_cc/calibration/seed/calibration_seed.jsonl`; a follow-up
//!   patch will wire this to the orchestrator's live corpus once the
//!   nightly TauUpdater is online (per D-16 task 3).
//!
//! Additive — sibling of `commands_d10` (mode routing) +
//! `commands_d11` (constructive checkers) + `commands_d13`
//! (adversary). The orchestrator producers
//! (`elai_orchestrator::conformal_calibration::compute_tau_and_emit` +
//! `decide_and_emit`, both reachable via
//! `agent_pool::dispatch_conformal_refusal`) are the sole writers of
//! the underlying SSB rows; this module only reads.
//!
//! Grounding: arXiv:2503.01345 (UniCR — Unified Conformal Refusal) +
//! Vovk et al. *Algorithmic Learning in a Random World* (2005) §2.2.
//!
//! See docs/thefinalglance/D-plan/D-16_unicr_conformal_refusal.md.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// Typed mirror of [`elai_orchestrator::conformal_calibration::ConformalTauRow`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformalTauRowDto {
    pub tau: f64,
    pub epsilon: f64,
    pub corpus_size: u32,
    pub source: String,
    pub ts_ms: i64,
}

/// Typed mirror of [`elai_orchestrator::conformal_calibration::ConformalRefusalRow`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformalRefusalRowDto {
    pub claim_id: String,
    pub claim_score: f64,
    pub tau: f64,
    pub decision: String,
    pub confidence: f64,
    pub ts_ms: i64,
}

/// Aggregated response for the panel: recent tau rows + recent refusal
/// decisions + a per-decision count (abstains vs answers across the
/// returned refusal rows) so the panel can render the histogram without
/// re-walking the rows in JS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentConformal {
    pub tau_rows: Vec<ConformalTauRowDto>,
    pub refusal_rows: Vec<ConformalRefusalRowDto>,
    pub abstain_count: u32,
    pub answer_count: u32,
}

/// Return the most recent `conformal_tau_computed` + `conformal_refusal_decided`
/// rows (capped at `MAX_LIMIT`). Empty response when no SSB viewer is
/// attached (Tauri-dev fresh boot, no DB).
#[tauri::command]
pub async fn get_conformal_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<RecentConformal, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(RecentConformal {
                tau_rows: Vec::new(),
                refusal_rows: Vec::new(),
                abstain_count: 0,
                answer_count: 0,
            });
        }
    };

    let tau_rows: Vec<ConformalTauRowDto> =
        elai_orchestrator::conformal_calibration::recent_tau_rows(&viewer, cap)
            .into_iter()
            .map(|r| ConformalTauRowDto {
                tau: r.tau,
                epsilon: r.epsilon,
                corpus_size: r.corpus_size,
                source: r.source,
                ts_ms: r.ts_ms,
            })
            .collect();

    let refusal_rows: Vec<ConformalRefusalRowDto> =
        elai_orchestrator::conformal_calibration::recent_refusal_rows(&viewer, cap)
            .into_iter()
            .map(|r| ConformalRefusalRowDto {
                claim_id: r.claim_id,
                claim_score: r.claim_score,
                tau: r.tau,
                decision: r.decision,
                confidence: r.confidence,
                ts_ms: r.ts_ms,
            })
            .collect();

    let abstain_count = u32::try_from(
        refusal_rows
            .iter()
            .filter(|r| r.decision == "abstain")
            .count(),
    )
    .unwrap_or(u32::MAX);
    let answer_count = u32::try_from(
        refusal_rows
            .iter()
            .filter(|r| r.decision == "answer")
            .count(),
    )
    .unwrap_or(u32::MAX);

    Ok(RecentConformal {
        tau_rows,
        refusal_rows,
        abstain_count,
        answer_count,
    })
}

/// Snapshot of the active calibration corpus state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationCorpusStateDto {
    /// Snake-case `CorpusSource` label.
    pub source: String,
    /// Number of tuples in the corpus.
    pub size: u32,
    /// Whether the corpus is frozen (production seed corpora are frozen
    /// by construction).
    pub frozen: bool,
    /// The UniCR operating point (epsilon) the orchestrator uses by
    /// default. Surfaced here so the panel renders the threshold without
    /// having to call a second endpoint.
    pub epsilon: f64,
}

/// Return a snapshot of the active calibration corpus. The MVP returns
/// a synthetic snapshot describing the shipped seed corpus
/// (`.elai_cc/calibration/seed/`). A follow-up patch will wire this to
/// the orchestrator's live corpus state once the nightly grower is
/// online.
#[tauri::command]
pub async fn get_calibration_corpus_state(
    _state: State<'_, AppState>,
) -> Result<CalibrationCorpusStateDto, String> {
    Ok(CalibrationCorpusStateDto {
        source: elai_orchestrator::conformal_calibration::CorpusSource::Synthetic
            .as_str()
            .to_string(),
        size: 20,
        frozen: true,
        epsilon: elai_orchestrator::conformal_calibration::DEFAULT_EPSILON,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tau_dto_serializes_with_expected_field_names() {
        let dto = ConformalTauRowDto {
            tau: 0.8,
            epsilon: 0.1,
            corpus_size: 40,
            source: "held_out".into(),
            ts_ms: 1_700_000_000_000,
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        for key in [
            "\"tau\"",
            "\"epsilon\"",
            "\"corpus_size\"",
            "\"source\"",
            "\"ts_ms\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn refusal_dto_serializes_with_expected_field_names() {
        let dto = ConformalRefusalRowDto {
            claim_id: "c-1".into(),
            claim_score: 0.5,
            tau: 0.7,
            decision: "abstain".into(),
            confidence: 0.0,
            ts_ms: 1_700_000_000_000,
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        for key in [
            "\"claim_id\"",
            "\"claim_score\"",
            "\"tau\"",
            "\"decision\"",
            "\"confidence\"",
            "\"ts_ms\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn corpus_state_dto_serializes_with_expected_field_names() {
        let dto = CalibrationCorpusStateDto {
            source: "synthetic".into(),
            size: 20,
            frozen: true,
            epsilon: 0.1,
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        for key in ["\"source\"", "\"size\"", "\"frozen\"", "\"epsilon\""] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }

    #[test]
    fn aggregated_response_serializes_with_expected_field_names() {
        let r = RecentConformal {
            tau_rows: Vec::new(),
            refusal_rows: Vec::new(),
            abstain_count: 0,
            answer_count: 0,
        };
        let s = serde_json::to_string(&r).expect("serialize");
        for key in [
            "\"tau_rows\"",
            "\"refusal_rows\"",
            "\"abstain_count\"",
            "\"answer_count\"",
        ] {
            assert!(s.contains(key), "missing key {key} in {s}");
        }
    }
}
