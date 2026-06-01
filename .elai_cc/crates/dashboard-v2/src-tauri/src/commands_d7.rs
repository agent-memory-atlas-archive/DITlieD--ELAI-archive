//! D-7: Cross-domain transfer canary Tauri command surface.
//!
//! Two read-only commands feed the Svelte `CanaryPanel`:
//!
//! - [`get_canary_recent`] — most recent `canary_run_completed` SSB rows
//!   flattened to a typed DTO (report_id, ts_ms, overall_transfer_score,
//!   decision_kind, dominant_fail_family).
//! - [`get_canary_per_family_breakdown`] — most recent
//!   `canary_family_evaluated` SSB rows grouped by `report_id` so the
//!   panel can render per-family pass/fail / transfer-score bars for
//!   each run.
//!
//! Additive — sibling of `commands_d6` (held-out store). The
//! orchestrator producer (`canary::emit_canary_run`) is the sole writer
//! of `canary_run_completed` / `canary_family_evaluated` rows; this
//! module only reads.
//!
//! See workflow/reasoning/D-7-decisions.md.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// One row of the recent canary-run list rendered by `CanaryPanel`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanaryRunRow {
    pub report_id: String,
    pub ts_ms: i64,
    pub overall_transfer_score: f64,
    pub decision_kind: String,
    pub dominant_fail_family: String,
}

/// One per-family row of the breakdown view rendered by `CanaryPanel`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanaryFamilyRow {
    pub report_id: String,
    pub family: String,
    pub n_evaluated: u32,
    pub pass_count: u32,
    pub fail_count: u32,
    pub transfer_score: f64,
    pub baseline_score: f64,
}

/// Aggregate per-family rows grouped under their parent run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanaryBreakdown {
    // B-15 Tranche E E.3 / DE7: BTreeMap for serialization-observed
    // iteration determinism (per gotchas.md B15_DETERMINISM_MAP_TYPES).
    pub by_report: BTreeMap<String, Vec<CanaryFamilyRow>>,
}

/// Return the last `limit` `canary_run_completed` SSB rows, sorted by
/// most-recent `ts_ms` first.
#[tauri::command]
pub async fn get_canary_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<CanaryRunRow>, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    let mut rows = Vec::new();
    match viewer.recent_rows_of_type("canary_run_completed", limit) {
        Ok(details) => {
            for detail in details {
                if let Some(row) = detail_to_run_row(detail) {
                    rows.push(row);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = "canary_run_completed",
                error = %e,
                "get_canary_recent: viewer error"
            );
        }
    }

    rows.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    rows.truncate(limit);
    Ok(rows)
}

/// Return per-family rows for the most recent canary runs, grouped by
/// `report_id`. The Svelte panel joins this with the run list to render
/// per-family bars.
#[tauri::command]
pub async fn get_canary_per_family_breakdown(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<CanaryBreakdown, String> {
    // We over-fetch by 5x because every run emits up to 5 per-family
    // rows; this keeps the grouped view consistent with `limit` runs.
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let fetch = cap.saturating_mul(5).min(MAX_LIMIT * 5);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(CanaryBreakdown {
                by_report: BTreeMap::new(),
            });
        }
    };

    let mut by_report: BTreeMap<String, Vec<CanaryFamilyRow>> = BTreeMap::new();
    match viewer.recent_rows_of_type("canary_family_evaluated", fetch) {
        Ok(details) => {
            for detail in details {
                if let Some(row) = detail_to_family_row(detail) {
                    by_report
                        .entry(row.report_id.clone())
                        .or_default()
                        .push(row);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = "canary_family_evaluated",
                error = %e,
                "get_canary_per_family_breakdown: viewer error"
            );
        }
    }

    Ok(CanaryBreakdown { by_report })
}

/// Unwrap the externally-tagged SsbPayload shape (`{"VariantName": {...}}`)
/// so consumers can read the inner fields directly. Falls back to the
/// outer object when the wrapper is absent (forward-compat with a
/// hypothetical custom-Serialize that flattens variants).
fn unwrap_variant<'a>(
    payload: &'a serde_json::Value,
    variant: &str,
) -> Option<&'a serde_json::Map<String, serde_json::Value>> {
    let outer = payload.as_object()?;
    if let Some(inner) = outer.get(variant).and_then(|v| v.as_object()) {
        return Some(inner);
    }
    Some(outer)
}

fn detail_to_run_row(d: elai_orchestrator::SsbActionDetail) -> Option<CanaryRunRow> {
    if d.action_type.as_str() != "canary_run_completed" {
        return None;
    }
    let obj = unwrap_variant(&d.payload_json, "CanaryRunCompleted")?;
    Some(CanaryRunRow {
        report_id: obj.get("report_id")?.as_str()?.to_string(),
        ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
        overall_transfer_score: obj
            .get("overall_transfer_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        decision_kind: obj
            .get("decision_kind")
            .and_then(|v| v.as_str())
            .unwrap_or("pass")
            .to_string(),
        dominant_fail_family: obj
            .get("dominant_fail_family")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

fn detail_to_family_row(d: elai_orchestrator::SsbActionDetail) -> Option<CanaryFamilyRow> {
    if d.action_type.as_str() != "canary_family_evaluated" {
        return None;
    }
    let obj = unwrap_variant(&d.payload_json, "CanaryFamilyEvaluated")?;
    Some(CanaryFamilyRow {
        report_id: obj.get("report_id")?.as_str()?.to_string(),
        family: obj
            .get("family")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        n_evaluated: obj
            .get("n_evaluated")
            .and_then(|v| v.as_u64())
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(0),
        pass_count: obj
            .get("pass_count")
            .and_then(|v| v.as_u64())
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(0),
        fail_count: obj
            .get("fail_count")
            .and_then(|v| v.as_u64())
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(0),
        transfer_score: obj
            .get("transfer_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        baseline_score: obj
            .get("baseline_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
    })
}
