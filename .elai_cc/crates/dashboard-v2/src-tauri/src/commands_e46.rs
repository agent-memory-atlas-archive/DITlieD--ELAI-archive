//! E-46: Hierarchical decomposition tuning Tauri command surface.
//!
//! Three read-only commands feed the Svelte `PlanningLedgerPanel`:
//!
//! - [`get_dual_ledger_state`] — most recent `task_advanced` SSB rows
//!   folded into a `DualLedgerStateDto` (tasks + progress) keyed by
//!   `task_id`. Supports filtering by status string.
//! - [`get_recent_agentless_shortcuts`] — last N
//!   `agentless_shortcut_taken` SSB rows for the dashboard taxonomy
//!   histogram.
//! - [`get_goal_drift_status`] — folded view of the most recent
//!   `goal_drift_fired` rows: latest drift / threshold / firing count.
//!
//! Additive — sibling of `commands_d9` (locked keys). The orchestrator
//! producer (`agent_pool::process_task_advancement` /
//! `emit_agentless_shortcut_taken` / `emit_goal_drift_fired`) is the
//! sole writer of these rows; this module only reads.
//!
//! See docs/thefinalglance/E-plan/E-46_planning_decomposition.md.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 100;
const MAX_LIMIT: usize = 1_000;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

/// One task in the dual-ledger view rendered by `PlanningLedgerPanel`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DualLedgerTaskDto {
    pub task_id: String,
    pub parent_id: String,
    pub completion_pct: f64,
    pub status: String,
    /// Latest blocker text emitted on this task (empty when none).
    pub latest_blocker: String,
    /// Wall-clock UNIX ms of the most recent advance event.
    pub last_update_ms: i64,
}

/// Aggregated dual-ledger snapshot returned by
/// [`get_dual_ledger_state`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DualLedgerStateDto {
    pub tasks: Vec<DualLedgerTaskDto>,
    /// Count of tasks at completion_pct >= 100 (promotion-eligible).
    pub n_complete: u32,
    /// Count of tasks currently blocked.
    pub n_blocked: u32,
}

/// One agentless-shortcut row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentlessShortcutRow {
    pub task_id: String,
    pub reason: String,
    pub ts_ms: i64,
}

/// One goal-drift-firing row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalDriftRow {
    pub initial_goal_blake3: String,
    pub drift: f64,
    pub threshold: f64,
    pub step: u32,
    pub ts_ms: i64,
}

/// Aggregate goal-drift status — recent firings + per-canary count.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalDriftStatusDto {
    pub firings: Vec<GoalDriftRow>,
    /// Number of firings grouped by canary id (initial_goal_blake3).
    pub firings_by_canary: HashMap<String, u32>,
    /// Drift of the most recent firing (NaN replaced with 0.0).
    pub latest_drift: f64,
    /// Threshold of the most recent firing.
    pub latest_threshold: f64,
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Return the most recent dual-ledger state derived from
/// `task_advanced` SSB rows. Latest row per `task_id` wins.
#[tauri::command]
pub async fn get_dual_ledger_state(
    state: State<'_, AppState>,
    filter: Option<String>,
) -> Result<DualLedgerStateDto, String> {
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(DualLedgerStateDto {
                tasks: Vec::new(),
                n_complete: 0,
                n_blocked: 0,
            });
        }
    };

    let mut latest: HashMap<String, DualLedgerTaskDto> = HashMap::new();
    match viewer.recent_rows_of_type("task_advanced", MAX_LIMIT) {
        Ok(details) => {
            for d in details {
                if let Some(row) = detail_to_task_dto(d) {
                    let prev_ts = latest
                        .get(&row.task_id)
                        .map(|r| r.last_update_ms)
                        .unwrap_or(i64::MIN);
                    if row.last_update_ms >= prev_ts {
                        latest.insert(row.task_id.clone(), row);
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = "task_advanced",
                error = %e,
                "get_dual_ledger_state: viewer error"
            );
        }
    }

    let filter_status = filter.as_deref();
    let mut tasks: Vec<DualLedgerTaskDto> = latest
        .into_values()
        .filter(|t| match filter_status {
            None => true,
            Some("") => true,
            Some(f) => t.status == f,
        })
        .collect();
    // Sort: most-recently updated first.
    tasks.sort_by(|a, b| b.last_update_ms.cmp(&a.last_update_ms));

    let n_complete = tasks
        .iter()
        .filter(|t| t.completion_pct >= 100.0)
        .count() as u32;
    let n_blocked = tasks.iter().filter(|t| t.status == "blocked").count() as u32;

    Ok(DualLedgerStateDto {
        tasks,
        n_complete,
        n_blocked,
    })
}

/// Return the last `limit` `agentless_shortcut_taken` rows, sorted
/// most-recent first.
#[tauri::command]
pub async fn get_recent_agentless_shortcuts(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<AgentlessShortcutRow>, String> {
    let cap = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    let mut rows: Vec<AgentlessShortcutRow> = Vec::new();
    match viewer.recent_rows_of_type("agentless_shortcut_taken", cap) {
        Ok(details) => {
            for d in details {
                if let Some(r) = detail_to_shortcut_row(d) {
                    rows.push(r);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = "agentless_shortcut_taken",
                error = %e,
                "get_recent_agentless_shortcuts: viewer error"
            );
        }
    }
    rows.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    rows.truncate(cap);
    Ok(rows)
}

/// Return the aggregate goal-drift status.
#[tauri::command]
pub async fn get_goal_drift_status(
    state: State<'_, AppState>,
) -> Result<GoalDriftStatusDto, String> {
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => {
            return Ok(GoalDriftStatusDto {
                firings: Vec::new(),
                firings_by_canary: HashMap::new(),
                latest_drift: 0.0,
                latest_threshold: 0.0,
            });
        }
    };

    let mut firings: Vec<GoalDriftRow> = Vec::new();
    match viewer.recent_rows_of_type("goal_drift_fired", MAX_LIMIT) {
        Ok(details) => {
            for d in details {
                if let Some(r) = detail_to_drift_row(d) {
                    firings.push(r);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = "goal_drift_fired",
                error = %e,
                "get_goal_drift_status: viewer error"
            );
        }
    }
    firings.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));

    let mut by_canary: HashMap<String, u32> = HashMap::new();
    for f in &firings {
        *by_canary.entry(f.initial_goal_blake3.clone()).or_insert(0) += 1;
    }
    let (latest_drift, latest_threshold) = firings
        .first()
        .map(|r| (r.drift, r.threshold))
        .unwrap_or((0.0, 0.0));

    Ok(GoalDriftStatusDto {
        firings,
        firings_by_canary: by_canary,
        latest_drift,
        latest_threshold,
    })
}

// ---------------------------------------------------------------------------
// Detail -> DTO helpers
// ---------------------------------------------------------------------------

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

fn detail_to_task_dto(d: elai_orchestrator::SsbActionDetail) -> Option<DualLedgerTaskDto> {
    if d.action_type.as_str() != "task_advanced" {
        return None;
    }
    let obj = unwrap_variant(&d.payload_json, "TaskAdvanced")?;
    Some(DualLedgerTaskDto {
        task_id: obj
            .get("task_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        parent_id: obj
            .get("parent_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        completion_pct: obj
            .get("completion_pct")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        status: obj
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("pending")
            .to_string(),
        latest_blocker: obj
            .get("blocker")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        last_update_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
    })
}

fn detail_to_shortcut_row(d: elai_orchestrator::SsbActionDetail) -> Option<AgentlessShortcutRow> {
    if d.action_type.as_str() != "agentless_shortcut_taken" {
        return None;
    }
    let obj = unwrap_variant(&d.payload_json, "AgentlessShortcutTaken")?;
    Some(AgentlessShortcutRow {
        task_id: obj
            .get("task_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        reason: obj
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
    })
}

fn detail_to_drift_row(d: elai_orchestrator::SsbActionDetail) -> Option<GoalDriftRow> {
    if d.action_type.as_str() != "goal_drift_fired" {
        return None;
    }
    let obj = unwrap_variant(&d.payload_json, "GoalDriftFired")?;
    Some(GoalDriftRow {
        initial_goal_blake3: obj
            .get("initial_goal_blake3")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        drift: obj
            .get("drift")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        threshold: obj
            .get("threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        step: obj.get("step").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
    })
}
