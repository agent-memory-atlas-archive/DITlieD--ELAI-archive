//! D-4: Tripwire stack Tauri command surface.
//!
//! One read-only command feeds the Svelte `TripwirePanel`:
//!
//! - [`get_recent_tripwires`] — last N tripwire rows (mix of
//!   `tripwire_fired` + `template_auto_reverted`) from the SSB store,
//!   flattened to a typed DTO the frontend can render without a second
//!   lookup.
//!
//! Additive — runs alongside the existing `commands::*` + `commands_d1::*` +
//! `commands_d2::*` + `commands_d3::*` handlers without touching them.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// One row of the panel. `kind` is one of:
/// * `"fired"`            — a detector tripped (TripwireFired row)
/// * `"auto_reverted"`    — an archived template was retired in response
///   (TemplateAutoReverted row)
/// * `"fired_concurrent"` — >=2 detectors fired in the same `observe()`
///   tick (TripwireFiredConcurrent row — D-4 GAP-2, SCHEMA_VERSION 108)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TripwireRow {
    pub seq: i64,
    pub ts: i64,
    pub kind: &'static str,
    pub event_id: Option<String>,
    pub tripwire_kind: Option<String>,
    /// D-4 GAP-1: signal-stream tag (`reward` / `action` / `perplexity`
    /// / `classifier`). None on legacy rows pre-GAP-1 + on
    /// auto_reverted / fired_concurrent rows that do not carry a single
    /// kind-derived source.
    #[serde(default)]
    pub source: Option<String>,
    pub candidate_id: Option<String>,
    pub observed_value: Option<f64>,
    pub threshold: Option<f64>,
    pub fired_at_ms: Option<i64>,
    pub template_id: Option<String>,
    pub parent_template_id: Option<String>,
    pub retirement_reason: Option<String>,
    pub fired_tripwire_id: Option<String>,
    /// D-4 GAP-2 (SCHEMA_VERSION 108): concurrent-fire columns. None on
    /// 'fired' / 'auto_reverted' rows.
    #[serde(default)]
    pub tripwire_a: Option<String>,
    #[serde(default)]
    pub tripwire_b: Option<String>,
    #[serde(default)]
    pub fired_within_ms: Option<u64>,
}

/// Return the last `limit` tripwire rows from SSB, newest first.
/// Empty Vec when the SSB store is missing (fresh repo) or contains no
/// tripwire rows.
#[tauri::command]
pub async fn get_recent_tripwires(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<TripwireRow>, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    let mut combined = Vec::new();
    // D-4 GAP-2: "tripwire_fired_concurrent" added so the dashboard
    // surfaces the new variant without a second poll path.
    for at in [
        "tripwire_fired",
        "template_auto_reverted",
        "tripwire_fired_concurrent",
    ] {
        match viewer.recent_rows_of_type(at, limit) {
            Ok(rows) => {
                for detail in rows {
                    if let Some(row) = tripwire_detail_to_row(detail) {
                        combined.push(row);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    action_type = at,
                    error = %e,
                    "get_recent_tripwires: viewer error"
                );
            }
        }
    }
    combined.sort_by(|a, b| b.seq.cmp(&a.seq).then(Ordering::Equal));
    combined.truncate(limit);
    Ok(combined)
}

fn tripwire_detail_to_row(d: elai_orchestrator::SsbActionDetail) -> Option<TripwireRow> {
    match d.action_type.as_str() {
        "tripwire_fired" => {
            let obj = d.payload_json.as_object()?;
            Some(TripwireRow {
                seq: d.seq,
                ts: d.ts,
                kind: "fired",
                event_id: obj.get("event_id").and_then(|v| v.as_str()).map(String::from),
                tripwire_kind: obj
                    .get("tripwire_kind")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                // D-4 GAP-1: signal-stream tag (None on legacy rows).
                source: obj.get("source").and_then(|v| v.as_str()).map(String::from),
                candidate_id: obj
                    .get("candidate_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                observed_value: obj.get("observed_value").and_then(|v| v.as_f64()),
                threshold: obj.get("threshold").and_then(|v| v.as_f64()),
                fired_at_ms: obj.get("fired_at_ms").and_then(|v| v.as_i64()),
                template_id: None,
                parent_template_id: None,
                retirement_reason: None,
                fired_tripwire_id: None,
                tripwire_a: None,
                tripwire_b: None,
                fired_within_ms: None,
            })
        }
        "template_auto_reverted" => {
            let obj = d.payload_json.as_object()?;
            Some(TripwireRow {
                seq: d.seq,
                ts: d.ts,
                kind: "auto_reverted",
                event_id: None,
                tripwire_kind: None,
                source: None,
                candidate_id: None,
                observed_value: None,
                threshold: None,
                fired_at_ms: None,
                template_id: obj
                    .get("template_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                parent_template_id: obj
                    .get("parent_template_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                retirement_reason: obj
                    .get("retirement_reason")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                fired_tripwire_id: obj
                    .get("fired_tripwire_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                tripwire_a: None,
                tripwire_b: None,
                fired_within_ms: None,
            })
        }
        // D-4 GAP-2 (SCHEMA_VERSION 108): concurrent-detector row.
        "tripwire_fired_concurrent" => {
            let obj = d.payload_json.as_object()?;
            Some(TripwireRow {
                seq: d.seq,
                ts: d.ts,
                kind: "fired_concurrent",
                event_id: None,
                tripwire_kind: None,
                source: None,
                candidate_id: obj
                    .get("candidate_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                observed_value: None,
                threshold: None,
                fired_at_ms: obj.get("fired_at_ms").and_then(|v| v.as_i64()),
                template_id: None,
                parent_template_id: None,
                retirement_reason: None,
                fired_tripwire_id: None,
                tripwire_a: obj
                    .get("tripwire_a")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                tripwire_b: obj
                    .get("tripwire_b")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                fired_within_ms: obj.get("fired_within_ms").and_then(|v| v.as_u64()),
            })
        }
        _ => None,
    }
}
