//! D-22: conformal calibration validation Tauri command surface.
//!
//! One read-only command feeds the Svelte `ConformalValidationPanel`:
//!
//! - [`get_conformal_validation_recent`] — last N
//!   `conformal_validation_completed` rows from the SSB store, flattened
//!   to a typed DTO the frontend can render without reloading the
//!   underlying payload.
//!
//! Additive — sibling of `commands_d21`. The orchestrator producer
//! (`crates/orchestrator/src/benchmarks/conformal_validation_runner.rs::ConformalValidationRunner::run_all`)
//! is the sole writer of `conformal_validation_completed` rows; this
//! module only reads.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// One row of the panel. Mirrors the
/// `ConformalValidationCompletedPayload` shape from the orchestrator
/// crate but flattened for direct serde-to-TypeScript transport.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConformalValidationRow {
    pub seq: i64,
    pub ts: i64,
    pub tau: f64,
    pub epsilon: f64,
    pub empirical_refusal_rate: f64,
    pub gating_verdict: String,
    pub fixture_blake3: String,
    pub total: u32,
}

/// Return the last `limit` conformal-validation benchmark rows from SSB,
/// newest first. Empty Vec when the SSB store is missing (fresh repo) or
/// contains no conformal-validation rows.
#[tauri::command]
pub async fn get_conformal_validation_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<ConformalValidationRow>, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    let mut rows = Vec::new();
    match viewer.recent_rows_of_type("conformal_validation_completed", limit) {
        Ok(details) => {
            for detail in details {
                if let Some(row) = detail_to_row(detail) {
                    rows.push(row);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = "conformal_validation_completed",
                error = %e,
                "get_conformal_validation_recent: viewer error"
            );
        }
    }
    rows.sort_by(|a, b| b.seq.cmp(&a.seq).then(Ordering::Equal));
    rows.truncate(limit);
    Ok(rows)
}

fn detail_to_row(d: elai_orchestrator::SsbActionDetail) -> Option<ConformalValidationRow> {
    if d.action_type != "conformal_validation_completed" {
        return None;
    }
    let obj = d.payload_json.as_object()?;
    Some(ConformalValidationRow {
        seq: d.seq,
        ts: d.ts,
        tau: obj.get("tau")?.as_f64()?,
        epsilon: obj.get("epsilon")?.as_f64()?,
        empirical_refusal_rate: obj.get("empirical_refusal_rate")?.as_f64()?,
        gating_verdict: obj.get("gating_verdict")?.as_str()?.to_string(),
        fixture_blake3: obj.get("fixture_blake3")?.as_str()?.to_string(),
        total: u32::try_from(obj.get("total")?.as_u64()?).unwrap_or(0),
    })
}
