//! D-21: mode-routing coverage benchmark Tauri command surface.
//!
//! One read-only command feeds the Svelte `ModeRoutingBenchPanel`:
//!
//! - [`get_mode_routing_bench_recent`] — last N
//!   `mode_routing_bench_run_completed` rows from the SSB store, flattened to
//!   a typed DTO the frontend can render without reloading the underlying
//!   payload.
//!
//! Additive — sibling of `commands_d19` (PRMBench). The orchestrator
//! producer
//! (`crates/orchestrator/src/benchmarks/mode_routing_runner.rs::ModeRoutingBenchmark::run_all`)
//! is the sole writer of `mode_routing_bench_run_completed` rows; this
//! module only reads.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// One row of the panel. Mirrors the
/// `ModeRoutingBenchRunCompletedPayload` shape from the orchestrator crate
/// but flattened for direct serde-to-TypeScript transport.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModeRoutingBenchRow {
    pub seq: i64,
    pub ts: i64,
    pub total: u32,
    pub accuracy: f64,
    pub f1_macro: f64,
    pub gating_verdict: String,
    pub fixture_blake3: String,
    pub fixture_row_count: u32,
}

/// Return the last `limit` mode-routing benchmark rows from SSB, newest
/// first. Empty Vec when the SSB store is missing (fresh repo) or contains
/// no mode-routing rows.
#[tauri::command]
pub async fn get_mode_routing_bench_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<ModeRoutingBenchRow>, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    let mut rows = Vec::new();
    match viewer.recent_rows_of_type("mode_routing_bench_run_completed", limit) {
        Ok(details) => {
            for detail in details {
                if let Some(row) = detail_to_row(detail) {
                    rows.push(row);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = "mode_routing_bench_run_completed",
                error = %e,
                "get_mode_routing_bench_recent: viewer error"
            );
        }
    }
    rows.sort_by(|a, b| b.seq.cmp(&a.seq).then(Ordering::Equal));
    rows.truncate(limit);
    Ok(rows)
}

fn detail_to_row(d: elai_orchestrator::SsbActionDetail) -> Option<ModeRoutingBenchRow> {
    if d.action_type != "mode_routing_bench_run_completed" {
        return None;
    }
    let obj = d.payload_json.as_object()?;
    Some(ModeRoutingBenchRow {
        seq: d.seq,
        ts: d.ts,
        total: u32::try_from(obj.get("total")?.as_u64()?).unwrap_or(0),
        accuracy: obj.get("accuracy")?.as_f64()?,
        f1_macro: obj.get("f1_macro")?.as_f64()?,
        gating_verdict: obj.get("gating_verdict")?.as_str()?.to_string(),
        fixture_blake3: obj.get("fixture_blake3")?.as_str()?.to_string(),
        fixture_row_count: u32::try_from(obj.get("fixture_row_count")?.as_u64()?).unwrap_or(0),
    })
}
