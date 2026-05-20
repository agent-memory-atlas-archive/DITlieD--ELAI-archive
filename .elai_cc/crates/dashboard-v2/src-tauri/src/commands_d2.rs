//! D-2: PPM (Process Preference Model) Tauri command surface.
//!
//! One read-only command feeds the Svelte `PpmScoresPanel`:
//!
//! - [`get_ppm_recent_scores`] — last N PPM verifier rows (mix of
//!   `ppm_scored` + `ppm_gating_failed`) from the SSB store, flattened
//!   to a typed DTO the frontend can render without a second lookup.
//!
//! Additive — runs alongside the existing `commands::*` + `commands_d1::*`
//! handlers without touching them. Aligns with the D-2 plan §panel: the
//! panel renders a chronological list of recent verifier decisions so
//! the operator can see PPM gating health at a glance.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// One row of the panel. `kind` is either `"scored"` (gate passed) or
/// `"gating_failed"` (gate tripped). `score_mean` / `score_var` are echoed
/// in both shapes; `threshold` is `None` for `scored` rows and `Some(x)`
/// for `gating_failed` rows. `cache_hit` / `latency_ms` are only populated
/// on `scored` rows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PpmScoreRow {
    pub seq: i64,
    pub ts: i64,
    pub kind: &'static str,
    pub candidate_id: String,
    pub verifier_id: String,
    pub mode: String,
    pub score_mean: f64,
    pub score_var: Option<f64>,
    pub num_steps: u32,
    pub threshold: Option<f64>,
    pub latency_ms: Option<u64>,
    pub cache_hit: Option<bool>,
}

/// Return the last `limit` PPM verifier rows from SSB, newest first.
/// Empty Vec when the SSB store is missing (fresh repo) or contains no
/// PPM rows.
#[tauri::command]
pub async fn get_ppm_recent_scores(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<PpmScoreRow>, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    // Fetch both action types and merge — `recent_rows_of_type` already
    // sorts DESC by seq, so concatenate + re-sort + truncate.
    let mut combined = Vec::new();
    for at in ["ppm_scored", "ppm_gating_failed"] {
        match viewer.recent_rows_of_type(at, limit) {
            Ok(rows) => {
                for detail in rows {
                    if let Some(row) = ppm_detail_to_row(detail) {
                        combined.push(row);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(action_type = at, error = %e, "get_ppm_recent_scores: viewer error");
            }
        }
    }
    combined.sort_by(|a, b| b.seq.cmp(&a.seq).then(Ordering::Equal));
    combined.truncate(limit);
    Ok(combined)
}

fn ppm_detail_to_row(
    d: elai_orchestrator::SsbActionDetail,
) -> Option<PpmScoreRow> {
    match d.action_type.as_str() {
        "ppm_scored" => {
            let obj = d.payload_json.as_object()?;
            Some(PpmScoreRow {
                seq: d.seq,
                ts: d.ts,
                kind: "scored",
                candidate_id: obj.get("candidate_id")?.as_str()?.to_string(),
                verifier_id: obj.get("verifier_id")?.as_str()?.to_string(),
                mode: obj.get("mode")?.as_str()?.to_string(),
                score_mean: obj.get("score_mean")?.as_f64()?,
                score_var: obj.get("score_var").and_then(|v| v.as_f64()),
                num_steps: u32::try_from(obj.get("num_steps")?.as_u64()?).unwrap_or(0),
                threshold: None,
                latency_ms: obj.get("latency_ms").and_then(|v| v.as_u64()),
                cache_hit: obj.get("cache_hit").and_then(|v| v.as_bool()),
            })
        }
        "ppm_gating_failed" => {
            let obj = d.payload_json.as_object()?;
            Some(PpmScoreRow {
                seq: d.seq,
                ts: d.ts,
                kind: "gating_failed",
                candidate_id: obj.get("candidate_id")?.as_str()?.to_string(),
                verifier_id: obj.get("verifier_id")?.as_str()?.to_string(),
                mode: obj.get("mode")?.as_str()?.to_string(),
                score_mean: obj.get("score_mean")?.as_f64()?,
                score_var: None,
                num_steps: u32::try_from(obj.get("num_steps")?.as_u64()?).unwrap_or(0),
                threshold: obj.get("threshold").and_then(|v| v.as_f64()),
                latency_ms: None,
                cache_hit: None,
            })
        }
        _ => None,
    }
}
