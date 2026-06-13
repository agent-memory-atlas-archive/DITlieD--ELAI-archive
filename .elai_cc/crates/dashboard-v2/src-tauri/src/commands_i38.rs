//! I-38 (SCHEMA_VERSION 235): QdeltaMeasured SSB read surface for the
//! Svelte `QdeltaPanel` drawer.
//!
//! One read-only command:
//!
//! - [`get_qdelta_latest`] -- the most recent `qdelta_measured` flat-beacon
//!   row emitted by `elai bench qdelta`. Surfaces QDELTA (off_rate - on_rate),
//!   paired 95% bootstrap CI, corpus_blake3, and runpod_pending status.
//!
//! Default-ON: `qdelta_measured` rows emit when `elai bench qdelta` runs.
//! The bench command IS the entrypoint (W-2, no ELAI_*_ENABLE gate).
//!
//! HONESTY: Tier-1 runs the embedded static-gate fixture corpus only; the
//! live dynamic leg is RUNPOD-PENDING (`runpod_pending = true`).
//!
//! Mirror of commands_g6.rs::get_safety_delta_latest per context-package
//! instruction (consumes_symbols I-38-C1 verified anchor).
//!
//! cross-lang-pair: dashboard/tauri/api

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::ssb::{SsbActionType, SsbPayload};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// One row of the QdeltaPanel. Field names must match the TypeScript
/// `QdeltaLatest` interface in `src/lib/api/types.ts`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QdeltaLatest {
    /// Suite label (e.g. "qdelta_static_fixture_v1").
    pub suite: String,
    /// Candidate-generation provenance label.
    pub model_id: String,
    /// Number of task pairs evaluated (gates-ON arm vs gates-OFF arm).
    pub n_tasks: u32,
    /// Number of bootstrap seeds used.
    pub seed_count: u32,
    /// Whether the evaluation was paired (always true after floor_check).
    pub paired: bool,
    /// Defective-diff rate with gates ON.
    pub on_defective_rate: f64,
    /// Defective-diff rate with gates OFF.
    pub off_defective_rate: f64,
    /// QDELTA = off_defective_rate - on_defective_rate (positive = gates help).
    pub delta: f64,
    /// Paired bootstrap 95% CI lower bound.
    pub ci_low: f64,
    /// Paired bootstrap 95% CI upper bound.
    pub ci_high: f64,
    /// Cost per task in USD (informational).
    pub dollar_per_task: f64,
    /// Contamination probe result string ("not_run"|"honestly_evaluated"|"game_with_memorization").
    pub contamination_probe_result: String,
    /// Mutation survival rate per task (I-20 stub; 0.0 until I-20 ships).
    pub surviving_mutants_per_task: f64,
    /// blake3 hex of the corpus blob (provenance; anti-gaming).
    pub corpus_blake3: String,
    /// `true` = Tier-1 fixture only; live-model leg is RunPod-gated (D-026).
    pub runpod_pending: bool,
    /// SSB row timestamp (unix milliseconds).
    pub ts: i64,
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

/// Return the most recent `qdelta_measured` flat-beacon row from SSB,
/// or `None` when no viewer is attached or no measurement has been recorded.
///
/// Read-only mirror of [`crate::commands_g6::get_safety_delta_latest`]:
/// queries the SSB viewer for the single newest `qdelta_measured` row
/// and projects it into [`QdeltaLatest`].
#[tauri::command]
pub async fn get_qdelta_latest(
    state: State<'_, AppState>,
) -> Result<Option<QdeltaLatest>, String> {
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(None),
    };

    let raw = viewer
        .recent_rows_of_type(SsbActionType::QdeltaMeasured.as_sql_str(), 1)
        .map_err(|e| e.to_string())?;

    for entry in raw {
        let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let SsbPayload::QdeltaMeasured(p) = pl {
            return Ok(Some(QdeltaLatest {
                suite: p.suite.clone(),
                model_id: p.model_id.clone(),
                n_tasks: p.n_tasks,
                seed_count: p.seed_count,
                paired: p.paired,
                on_defective_rate: p.on_defective_rate,
                off_defective_rate: p.off_defective_rate,
                delta: p.delta,
                ci_low: p.ci_low,
                ci_high: p.ci_high,
                dollar_per_task: p.dollar_per_task,
                contamination_probe_result: p.contamination_probe_result.clone(),
                surviving_mutants_per_task: p.surviving_mutants_per_task,
                corpus_blake3: p.corpus_blake3.clone(),
                runpod_pending: p.runpod_pending,
                ts: entry.ts,
            }));
        }
    }

    Ok(None)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qdelta_latest_serializes_expected_fields() {
        let row = QdeltaLatest {
            suite: "qdelta_static_fixture_v1".to_string(),
            model_id: "fixture_corpus".to_string(),
            n_tasks: 5,
            seed_count: 5,
            paired: true,
            on_defective_rate: 0.2,
            off_defective_rate: 0.6,
            delta: 0.4,
            ci_low: 0.1,
            ci_high: 0.7,
            dollar_per_task: 0.0,
            contamination_probe_result: "not_run".to_string(),
            surviving_mutants_per_task: 0.0,
            corpus_blake3: "0123456789abcdef".to_string(),
            runpod_pending: true,
            ts: 1_700_000_000_001,
        };
        let json = serde_json::to_string(&row).expect("serialise");
        assert!(json.contains("delta"), "JSON must contain delta");
        assert!(json.contains("ci_low"), "JSON must contain ci_low");
        assert!(json.contains("ci_high"), "JSON must contain ci_high");
        assert!(json.contains("runpod_pending"), "JSON must contain runpod_pending");
        assert!(json.contains("corpus_blake3"), "JSON must contain corpus_blake3");
        assert!(json.contains("contamination_probe_result"), "JSON must contain contamination_probe_result");
        assert!(json.contains("surviving_mutants_per_task"), "JSON must contain surviving_mutants_per_task");
    }

    #[test]
    fn qdelta_latest_delta_field_positive_for_valid_fixture() {
        let row = QdeltaLatest {
            suite: "s".to_string(),
            model_id: "m".to_string(),
            n_tasks: 5,
            seed_count: 5,
            paired: true,
            on_defective_rate: 0.2,
            off_defective_rate: 0.6,
            delta: 0.4,
            ci_low: 0.05,
            ci_high: 0.75,
            dollar_per_task: 0.0,
            contamination_probe_result: "not_run".to_string(),
            surviving_mutants_per_task: 0.0,
            corpus_blake3: "abc".to_string(),
            runpod_pending: true,
            ts: 0,
        };
        assert!(row.delta > 0.0, "delta must be positive when gates help");
    }
}
