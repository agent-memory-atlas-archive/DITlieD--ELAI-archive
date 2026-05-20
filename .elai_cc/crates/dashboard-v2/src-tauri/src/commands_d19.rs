//! D-19: PRMBench adversarial-gaming benchmark Tauri command surface.
//!
//! One read-only command feeds the Svelte `PrmBenchPanel`:
//!
//! - [`get_prmbench_recent`] — last N `prmbench_run_completed` rows from
//!   the SSB store, flattened to a typed DTO the frontend can render
//!   without re-loading the underlying payload.
//!
//! Additive — runs alongside the existing `commands::*` + `commands_d1::*`
//! + `commands_d2::*` + `commands_d3::*` handlers without touching them.
//! Aligns with the D-19 plan §panel: the panel renders the most recent
//! PRMBench runs with accuracy / precision / recall + per-category
//! verdict breakdown so the operator can see PPM adversarial-gaming
//! exposure at a glance.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// One row of the panel — `accuracy` / `precision` / `recall` mirror the
/// orchestrator-side `PRMBenchReport` aggregate. `gating_verdict` is the
/// snake-case label (`"pass"` / `"fail"`). `fixture_blake3` is the 64-hex
/// blake3 of the JSONL fixture the run was computed against, so dashboard
/// consumers can pin a row to a dataset revision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrmBenchRow {
    pub seq: i64,
    pub ts: i64,
    pub total: u32,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub gating_verdict: String,
    pub fixture_blake3: String,
    pub category_count: u32,
}

/// Return the last `limit` PRMBench run rows from SSB, newest first.
/// Empty Vec when the SSB store is missing (fresh repo) or contains no
/// PRMBench rows.
#[tauri::command]
pub async fn get_prmbench_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<PrmBenchRow>, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    let mut rows = Vec::new();
    match viewer.recent_rows_of_type("prmbench_run_completed", limit) {
        Ok(details) => {
            for detail in details {
                if let Some(row) = prmbench_detail_to_row(detail) {
                    rows.push(row);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = "prmbench_run_completed",
                error = %e,
                "get_prmbench_recent: viewer error"
            );
        }
    }
    rows.sort_by(|a, b| b.seq.cmp(&a.seq).then(Ordering::Equal));
    rows.truncate(limit);
    Ok(rows)
}

fn prmbench_detail_to_row(d: elai_orchestrator::SsbActionDetail) -> Option<PrmBenchRow> {
    if d.action_type != "prmbench_run_completed" {
        return None;
    }
    let obj = d.payload_json.as_object()?;
    Some(PrmBenchRow {
        seq: d.seq,
        ts: d.ts,
        total: u32::try_from(obj.get("total")?.as_u64()?).unwrap_or(0),
        accuracy: obj.get("accuracy")?.as_f64()?,
        precision: obj.get("precision")?.as_f64()?,
        recall: obj.get("recall")?.as_f64()?,
        gating_verdict: obj.get("gating_verdict")?.as_str()?.to_string(),
        fixture_blake3: obj.get("fixture_blake3")?.as_str()?.to_string(),
        category_count: u32::try_from(obj.get("category_count")?.as_u64()?).unwrap_or(0),
    })
}
