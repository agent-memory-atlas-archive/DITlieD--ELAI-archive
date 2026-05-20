//! D-5: BAI (Best-Arm Identification) Thompson-sampling promotion round
//! Tauri command surface.
//!
//! One read-only command feeds the Svelte `BaiArmsPanel`:
//!
//! - [`get_bai_arms`] — most recent BAI arm-update rows from the SSB
//!   store, flattened to a typed DTO carrying the Beta(alpha, beta)
//!   posterior + n_pulls + mean estimate + promotion flag. The frontend
//!   sorts by mean_estimate descending so the empirically-leading arm
//!   surfaces at the top of the panel.
//!
//! Additive — runs alongside the existing `commands::*` + `commands_d1::*` +
//! `commands_d2::*` + `commands_d3::*` + `commands_d4::*` handlers without
//! touching them. The orchestrator producer
//! (`bai::emit_bai_round`) is the sole writer of `bai_arm_updated` rows;
//! this module only reads.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// One row of the panel. `mean_estimate` is `alpha / (alpha + beta)` —
/// the Beta posterior mean. `was_promoted` mirrors the orchestrator's
/// `PromotionDecision::was_promoted()` flag for the round that produced
/// this row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaiArmRow {
    pub template_id: String,
    pub alpha: f64,
    pub beta: f64,
    pub n_pulls: u32,
    pub was_promoted: bool,
    /// alpha / (alpha + beta). Computed Tauri-side so the frontend
    /// doesn't have to repeat the divide-by-zero guard (Beta(1, 1)
    /// prior makes the denominator always positive in practice, but
    /// the explicit guard keeps the contract honest).
    pub mean_estimate: f64,
}

/// Return the last `limit` BAI arm-update rows from SSB, sorted by
/// `mean_estimate` descending (empirically leading arms surface first).
/// Empty Vec when the SSB store is missing (fresh repo) or contains no
/// BAI rows.
#[tauri::command]
pub async fn get_bai_arms(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<BaiArmRow>, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    let mut rows = Vec::new();
    match viewer.recent_rows_of_type("bai_arm_updated", limit) {
        Ok(details) => {
            for detail in details {
                if let Some(row) = bai_detail_to_row(detail) {
                    rows.push(row);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = "bai_arm_updated",
                error = %e,
                "get_bai_arms: viewer error"
            );
        }
    }

    // Sort descending by mean_estimate; NaNs sink to the bottom via the
    // partial_cmp -> Equal fallback (Beta posteriors are always finite
    // in practice but the explicit fallback keeps the comparator total).
    rows.sort_by(|a, b| {
        b.mean_estimate
            .partial_cmp(&a.mean_estimate)
            .unwrap_or(Ordering::Equal)
    });
    rows.truncate(limit);
    Ok(rows)
}

fn bai_detail_to_row(d: elai_orchestrator::SsbActionDetail) -> Option<BaiArmRow> {
    if d.action_type.as_str() != "bai_arm_updated" {
        return None;
    }
    let obj = d.payload_json.as_object()?;
    let template_id = obj.get("template_id")?.as_str()?.to_string();
    let alpha = obj.get("alpha").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let beta = obj.get("beta").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let n_pulls = obj
        .get("n_pulls")
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(0);
    let was_promoted = obj
        .get("was_promoted")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    // Prefer the payload-carried mean_estimate (bai::emit_bai_round writes
    // arm.mean() into the row directly); fall back to alpha/(alpha+beta)
    // when the field is missing for any reason. The denominator can only
    // be zero if both alpha and beta were zeroed, which the Beta(1, 1)
    // prior + monotonic update prevents — but the guard is cheap.
    let mean_estimate = obj
        .get("mean_estimate")
        .and_then(|v| v.as_f64())
        .unwrap_or_else(|| {
            let denom = alpha + beta;
            if denom > 0.0 {
                alpha / denom
            } else {
                0.5
            }
        });
    Some(BaiArmRow {
        template_id,
        alpha,
        beta,
        n_pulls,
        was_promoted,
        mean_estimate,
    })
}
