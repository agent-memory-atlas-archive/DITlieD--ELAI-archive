//! D-23: Sycophancy detection benchmark Tauri command surface.
//!
//! One read-only command feeds the Svelte `SycophancyBenchPanel`:
//!
//! - [`get_sycophancy_bench_recent`] — last N
//!   `sycophancy_bench_run_completed` rows from the SSB store, flattened
//!   to a typed DTO the frontend can render without re-loading the
//!   underlying payload.
//!
//! Additive — runs alongside the existing `commands::*` + `commands_d19::*`
//! handlers without touching them. Aligns with the D-23 plan:
//! the panel renders the most recent sycophancy bench runs with detection
//! rate / FP rate + verdict color so the operator can see D-14 substrate
//! exposure at a glance.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// One row of the panel — mirrors the orchestrator-side
/// `SycophancyBenchRunCompletedPayload`. `verdict` is the snake-case
/// label (`"pass"` / `"fail"`). `fixture_blake3` is the 64-hex blake3
/// of the canonicalized fixture corpus the run was computed against.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SycophancyBenchRow {
    pub seq: i64,
    pub ts: i64,
    pub tp: u32,
    pub fp: u32,
    pub tn: u32,
    pub fn_: u32,
    pub inconclusive: u32,
    pub detection_rate: f64,
    pub fp_rate: f64,
    pub verdict: String,
    pub total_fixtures: u32,
    pub fixture_blake3: String,
    pub runner_name: String,
}

/// Return the last `limit` sycophancy bench run rows from SSB, newest
/// first. Empty Vec when the SSB store is missing (fresh repo) or
/// contains no sycophancy_bench rows.
#[tauri::command]
pub async fn get_sycophancy_bench_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<SycophancyBenchRow>, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    let mut rows = Vec::new();
    match viewer.recent_rows_of_type("sycophancy_bench_run_completed", limit) {
        Ok(details) => {
            for detail in details {
                if let Some(row) = sycophancy_bench_detail_to_row(detail) {
                    rows.push(row);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = "sycophancy_bench_run_completed",
                error = %e,
                "get_sycophancy_bench_recent: viewer error"
            );
        }
    }
    rows.sort_by(|a, b| b.seq.cmp(&a.seq).then(Ordering::Equal));
    rows.truncate(limit);
    Ok(rows)
}

fn sycophancy_bench_detail_to_row(
    d: elai_orchestrator::SsbActionDetail,
) -> Option<SycophancyBenchRow> {
    if d.action_type != "sycophancy_bench_run_completed" {
        return None;
    }
    let obj = d.payload_json.as_object()?;
    Some(SycophancyBenchRow {
        seq: d.seq,
        ts: d.ts,
        tp: u32::try_from(obj.get("tp")?.as_u64()?).unwrap_or(0),
        fp: u32::try_from(obj.get("fp")?.as_u64()?).unwrap_or(0),
        tn: u32::try_from(obj.get("tn")?.as_u64()?).unwrap_or(0),
        fn_: u32::try_from(obj.get("fn_")?.as_u64()?).unwrap_or(0),
        inconclusive: u32::try_from(obj.get("inconclusive")?.as_u64()?).unwrap_or(0),
        detection_rate: obj.get("detection_rate")?.as_f64()?,
        fp_rate: obj.get("fp_rate")?.as_f64()?,
        verdict: obj.get("verdict")?.as_str()?.to_string(),
        total_fixtures: u32::try_from(obj.get("total_fixtures")?.as_u64()?).unwrap_or(0),
        fixture_blake3: obj.get("fixture_blake3")?.as_str()?.to_string(),
        runner_name: obj.get("runner_name")?.as_str()?.to_string(),
    })
}
