//! G-6 (SCHEMA_VERSION 217): SafetyDeltaMeasured SSB read surface for the
//! Svelte `SafeDeltaPanel` drawer.
//!
//! One read-only command:
//!
//! - [`get_safety_delta_latest`] -- the most recent `safety_delta_measured`
//!   flat-beacon row emitted by `elai bench safedelta`. Surfaces the derived
//!   SAFEDELTA (gate-ON-vs-gate-OFF marginal residual rate) + 95% bootstrap CI
//!   + per-CWE delta breakdown + corpus_blake3 + pinned seed + gate_recall +
//!   runpod_pending status.
//!
//! Default-ON: `safety_delta_measured` rows emit when `elai bench safedelta`
//! runs. The bench command IS the entrypoint (W-2, no ELAI_*_ENABLE gate).
//!
//! HONESTY: Tier-1 runs the embedded static-gate corpus only; the live
//! dynamic exploit-exec arm (BaxBench-392 arXiv:2502.11844, SUSVIBES-200
//! arXiv:2512.03262) is RUNPOD-PENDING (`runpod_pending = true`).
//!
//! Mirror of commands_f2.rs::get_insecure_rate_latest (:171) per context-
//! package instruction (consumes_symbols SC-3 verified anchor).
//!
//! cross-lang-pair: dashboard/tauri/api

use serde::{Deserialize, Serialize};
use tauri::State;

use elai_orchestrator::ssb::{SsbActionType, SsbPayload};

use crate::state::AppState;

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// One row of the SafeDeltaPanel. Field names must match the TypeScript
/// `SafetyDeltaLatest` interface in `src/lib/api/types.ts`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafetyDeltaLatest {
    /// Suite label from the underlying SecCodePlt report.
    pub suite: String,
    /// Candidate-generation provenance label.
    pub model_id: String,
    /// Derived SAFEDELTA = insecure_rate_before - insecure_rate_after.
    pub safety_delta: f64,
    /// Seeded bootstrap-resample 95% CI lower bound.
    pub ci_low: f64,
    /// Seeded bootstrap-resample 95% CI upper bound.
    pub ci_high: f64,
    /// Number of candidate completions evaluated.
    pub n_tasks: u32,
    /// Deterministic RNG seed (ELAI_SEED default 0).
    pub seed: u64,
    /// blake3 hex of the corpus blob (provenance; anti-gaming).
    pub corpus_blake3: String,
    /// Derived gate recall: (before - after) / before.
    pub gate_recall: f64,
    /// `true` = Tier-1 static-gate only; Tier-2 is RunPod-gated.
    pub runpod_pending: bool,
    /// SSB row timestamp (unix milliseconds).
    pub ts: i64,
}

// ---------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------

/// Return the most recent `safety_delta_measured` flat-beacon row from SSB,
/// or `None` when no viewer is attached or no measurement has been recorded yet.
///
/// Read-only mirror of [`crate::commands_f2::get_insecure_rate_latest`]:
/// it queries the SSB viewer for the single newest `safety_delta_measured` row
/// and projects it into [`SafetyDeltaLatest`].
#[tauri::command]
pub async fn get_safety_delta_latest(
    state: State<'_, AppState>,
) -> Result<Option<SafetyDeltaLatest>, String> {
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(None),
    };

    let raw = viewer
        .recent_rows_of_type(SsbActionType::SafetyDeltaMeasured.as_sql_str(), 1)
        .map_err(|e| e.to_string())?;

    for entry in raw {
        let pl: SsbPayload = match serde_json::from_value(entry.payload_json.clone()) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let SsbPayload::SafetyDeltaMeasured(p) = pl {
            return Ok(Some(SafetyDeltaLatest {
                suite: p.suite.clone(),
                model_id: p.model_id.clone(),
                safety_delta: p.safety_delta,
                ci_low: p.ci_low,
                ci_high: p.ci_high,
                n_tasks: p.n_tasks,
                seed: p.seed,
                corpus_blake3: p.corpus_blake3.clone(),
                gate_recall: p.gate_recall,
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
    fn safety_delta_latest_serializes_expected_fields() {
        let row = SafetyDeltaLatest {
            suite: "secodeplt_static_gate".to_string(),
            model_id: "seccodeplt_curated_corpus".to_string(),
            safety_delta: 0.34375,
            ci_low: 0.21875,
            ci_high: 0.46875,
            n_tasks: 96,
            seed: 42,
            corpus_blake3: "0123456789abcdef".to_string(),
            gate_recall: 0.6875,
            runpod_pending: true,
            ts: 1_700_000_000_001,
        };
        let json = serde_json::to_string(&row).expect("serialise");
        assert!(json.contains("safety_delta"), "JSON must contain safety_delta");
        assert!(json.contains("ci_low"), "JSON must contain ci_low");
        assert!(json.contains("ci_high"), "JSON must contain ci_high");
        assert!(json.contains("runpod_pending"), "JSON must contain runpod_pending");
        assert!(json.contains("corpus_blake3"), "JSON must contain corpus_blake3");

        let parsed: SafetyDeltaLatest = serde_json::from_str(&json).expect("round-trip");
        assert_eq!(parsed.safety_delta, row.safety_delta);
        assert_eq!(parsed.runpod_pending, true);
    }
}
