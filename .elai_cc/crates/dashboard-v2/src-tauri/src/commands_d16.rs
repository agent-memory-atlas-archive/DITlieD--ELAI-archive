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
        elai_orchestrator::calibration::recent_tau_rows(&viewer, cap)
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
        elai_orchestrator::calibration::recent_refusal_rows(&viewer, cap)
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
///
/// D-16 closure: extended with `corpus_size`, `last_updated_ts`, and
/// `current_tau` so the panel renders the durable nightly tau scalar
/// without re-decoding SSB rows. These fields are read from the live
/// `TauUpdater` over the configured corpus + tau paths
/// (default `.elai_cc/calibration/seed/corpus.jsonl` +
/// `.elai_cc/data/conformal_tau.txt`). When the tau file is missing,
/// `current_tau` falls back to `TAU_DEFAULT_FALLBACK` (0.9) and
/// `last_updated_ts` is `None`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationCorpusStateDto {
    /// Snake-case `CorpusSource` label.
    pub source: String,
    /// Number of tuples in the corpus (legacy field).
    pub size: u32,
    /// Whether the corpus is frozen (production seed corpora are frozen
    /// by construction).
    pub frozen: bool,
    /// The UniCR operating point (epsilon) the orchestrator uses by
    /// default. Surfaced here so the panel renders the threshold without
    /// having to call a second endpoint.
    pub epsilon: f64,
    /// D-16 closure: number of tuples in the live calibration corpus
    /// as observed by the dashboard's `TauUpdater::read_corpus_size`.
    /// Mirrors `size` semantically; kept as a separate field for
    /// forward compatibility with the per-tier corpus split.
    pub corpus_size: u32,
    /// D-16 closure: durable tau scalar currently persisted on disk
    /// (`TauUpdater::read_persisted_tau`). Reflects the most recent
    /// `calibration_corpus_updated` SSB row's `new_tau`. Defaults to
    /// `TAU_DEFAULT_FALLBACK` when the file does not exist.
    pub current_tau: f64,
    /// D-16 closure: UNIX-ms wall-clock of the tau file's last
    /// modification. `None` when the file does not exist yet (e.g.
    /// fresh boot before the first updater run).
    pub last_updated_ts: Option<i64>,
}

/// Default seed corpus path the dashboard reads when no override
/// configuration is plumbed. Matches the CLI handler's default at
/// crates/cli/src/conformal_calibration_cli.rs.
const DASHBOARD_DEFAULT_CORPUS_PATH: &str = ".elai_cc/calibration/seed/corpus.jsonl";

/// Return a snapshot of the active calibration corpus. Reads the live
/// `TauUpdater` over the project-local corpus + tau paths so the panel
/// renders the durable threshold + corpus size in real time.
///
/// Backward compatibility: legacy `size` / `epsilon` fields preserved;
/// new `corpus_size` / `current_tau` / `last_updated_ts` fields surface
/// the D-16 closure values per GAP-7.
#[tauri::command]
pub async fn get_calibration_corpus_state(
    _state: State<'_, AppState>,
) -> Result<CalibrationCorpusStateDto, String> {
    use std::path::PathBuf;
    use std::time::UNIX_EPOCH;

    let corpus_path = PathBuf::from(DASHBOARD_DEFAULT_CORPUS_PATH);
    let tau_path = elai_orchestrator::calibration::tau_path_default();
    let updater = elai_orchestrator::calibration::TauUpdater::new(
        corpus_path,
        tau_path.clone(),
        elai_orchestrator::calibration::DEFAULT_EPSILON,
    );
    let corpus_size_usize = updater.read_corpus_size();
    let corpus_size = u32::try_from(corpus_size_usize).unwrap_or(u32::MAX);
    let current_tau = updater.read_persisted_tau();
    let last_updated_ts = std::fs::metadata(&tau_path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .and_then(|d| i64::try_from(d.as_millis()).ok());

    // `source` reflects the corpus provenance the live updater is
    // reading from. When the corpus file is empty / missing, fall back
    // to the synthetic-seed label so the panel doesn't render a
    // misleading "auto" for a yet-empty grower.
    let source = if corpus_size_usize == 0 {
        elai_orchestrator::calibration::CorpusSource::Synthetic
            .as_str()
            .to_string()
    } else {
        elai_orchestrator::calibration::CorpusSource::Auto
            .as_str()
            .to_string()
    };

    Ok(CalibrationCorpusStateDto {
        source,
        size: corpus_size,
        frozen: false,
        epsilon: elai_orchestrator::calibration::DEFAULT_EPSILON,
        corpus_size,
        current_tau,
        last_updated_ts,
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
            corpus_size: 20,
            current_tau: 0.9,
            last_updated_ts: Some(1_700_000_000_000),
        };
        let s = serde_json::to_string(&dto).expect("serialize");
        for key in [
            "\"source\"",
            "\"size\"",
            "\"frozen\"",
            "\"epsilon\"",
            "\"corpus_size\"",
            "\"current_tau\"",
            "\"last_updated_ts\"",
        ] {
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
