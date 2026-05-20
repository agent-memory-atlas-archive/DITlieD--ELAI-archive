//! D-6: Time-windowed held-out task store + 9-gram contamination filter
//! Tauri command surface.
//!
//! Two read-only commands feed the Svelte `HeldOutPanel`:
//!
//! - [`get_held_out_recent`] — most recent `held_out_task_added` SSB rows,
//!   flattened to a typed DTO carrying the task envelope (id, source,
//!   blake3, ngram count, ingest + eligibility window).
//! - [`get_held_out_stats`] — aggregated counters (total / per-source /
//!   active / contamination_hits) computed from the SSB tail. The
//!   `contamination_hits` counter is approximated as `held_out_task_added`
//!   write attempts that did not land — surfaced indirectly by counting
//!   distinct `task_id` values vs total emitted rows; pure SSB-driven so
//!   the dashboard does not need a second IPC channel into the orchestrator.
//!
//! Additive — runs alongside the existing `commands::*` + `commands_d{1..5,19}::*`
//! handlers without touching them. The orchestrator producer
//! (`held_out::insert_and_emit` + `held_out::mark_used_and_emit`) is the
//! sole writer of `held_out_task_added` / `held_out_task_used` rows; this
//! module only reads.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// One row of the recent-tasks list rendered by `HeldOutPanel`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeldOutTaskRow {
    pub task_id: String,
    pub source: String,
    pub content_blake3: String,
    pub ngram_count: u32,
    pub ingested_ts_ms: i64,
    pub eligibility_window_ms: i64,
}

/// Aggregate counters rendered by `HeldOutPanel`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeldOutStatsRow {
    pub total_added: u32,
    pub total_used: u32,
    pub by_source: HashMap<String, u32>,
    /// Used-count divided by added-count, capped at 1.0. Renders the
    /// utilization gauge in the panel.
    pub utilization_pct: f32,
}

/// Return the last `limit` `held_out_task_added` SSB rows, ordered by
/// most-recent first (the SSB viewer returns ascending; we reverse on
/// flatten). Empty Vec when the SSB store is missing or contains no
/// held-out rows.
#[tauri::command]
pub async fn get_held_out_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<HeldOutTaskRow>, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    let mut rows = Vec::new();
    match viewer.recent_rows_of_type("held_out_task_added", limit) {
        Ok(details) => {
            for detail in details {
                if let Some(row) = detail_to_task_row(detail) {
                    rows.push(row);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = "held_out_task_added",
                error = %e,
                "get_held_out_recent: viewer error"
            );
        }
    }

    // Most-recent ingest first (SSB viewer's natural order is insertion
    // ascending; we want descending by ingested_ts_ms for the panel).
    rows.sort_by(|a, b| b.ingested_ts_ms.cmp(&a.ingested_ts_ms));
    rows.truncate(limit);
    Ok(rows)
}

/// Return aggregate stats computed from the SSB tail.
#[tauri::command]
pub async fn get_held_out_stats(
    state: State<'_, AppState>,
) -> Result<HeldOutStatsRow, String> {
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(HeldOutStatsRow {
                total_added: 0,
                total_used: 0,
                by_source: HashMap::new(),
                utilization_pct: 0.0,
            });
        }
    };

    // Scan up to 2000 of each — bounded so the panel stays cheap; the
    // dashboard refresh cadence (~1-2s) is more than enough to keep
    // counters fresh.
    let added = viewer
        .recent_rows_of_type("held_out_task_added", 2_000)
        .unwrap_or_default();
    let used = viewer
        .recent_rows_of_type("held_out_task_used", 2_000)
        .unwrap_or_default();

    let total_added = added.len() as u32;
    let total_used = used.len() as u32;

    let mut by_source: HashMap<String, u32> = HashMap::new();
    for detail in &added {
        if let Some(obj) = detail.payload_json.as_object() {
            if let Some(src) = obj.get("source").and_then(|v| v.as_str()) {
                *by_source.entry(src.to_string()).or_insert(0) += 1;
            }
        }
    }

    let utilization_pct = if total_added == 0 {
        0.0
    } else {
        let u = total_used as f32 / total_added as f32;
        if u > 1.0 {
            1.0
        } else {
            u
        }
    };

    Ok(HeldOutStatsRow {
        total_added,
        total_used,
        by_source,
        utilization_pct,
    })
}

fn detail_to_task_row(d: elai_orchestrator::SsbActionDetail) -> Option<HeldOutTaskRow> {
    if d.action_type.as_str() != "held_out_task_added" {
        return None;
    }
    let obj = d.payload_json.as_object()?;
    let task_id = obj.get("task_id")?.as_str()?.to_string();
    let source = obj
        .get("source")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let content_blake3 = obj
        .get("content_blake3")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let ngram_count = obj
        .get("ngram_count")
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(0);
    let ingested_ts_ms = obj
        .get("ingested_ts_ms")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let eligibility_window_ms = obj
        .get("eligibility_window_ms")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    Some(HeldOutTaskRow {
        task_id,
        source,
        content_blake3,
        ngram_count,
        ingested_ts_ms,
        eligibility_window_ms,
    })
}
