//! E-39 -- cost + token economics dashboard surface.
//!
//! Four read-only Tauri commands feed the Svelte `CostPanel`:
//!
//! * `get_velocity_cap_status` -- snapshot of the 4-window
//!   (1m / 10m / 1h / 24h) rolling counters + the most recent
//!   `velocity_cap_breached` SSB row, if any.
//! * `get_cost_router_recent` -- recent `cost_router_decision` rows
//!   (one per cloud dispatch).
//! * `get_automix_verdicts_recent` -- recent
//!   `automix_verifier_verdict` rows.
//! * `get_free_tier_quotas` -- per-backend remaining-quota snapshot.
//!
//! Additive sibling of commands_e10 / commands_e38. The orchestrator
//! producer modules (`crate::cost::*`) are the sole writers of the
//! underlying SSB rows; this module only reads.
//!
//! Wire-dark unless `ELAI_COST_TOKEN_ECONOMICS_ENABLE=1` on the
//! orchestrator side -- commands return empty envelopes when no rows
//! exist. See `workflow/reasoning/E-39-decisions.md`.

use elai_orchestrator::ssb_viewer::SsbActionDetail;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 100;
const MAX_LIMIT: usize = 1_000;

/// One per-window snapshot for the velocity-cap panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VelocityWindowSnapshot {
    /// Window label: `one_min` / `ten_min` / `one_hour` / `twenty_four_hour`.
    pub window: String,
    /// Latest count (requests).
    pub count: u64,
    /// Latest limit (requests). Absent fields default to 0.
    pub limit: u64,
    /// Latest metric kind: `requests` or `usd_cents`.
    pub kind: String,
    /// Wall-clock ts of the most recent row that referenced this window.
    pub ts_ms: i64,
}

/// Aggregated velocity-cap panel response.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct VelocityCapStatus {
    pub windows: Vec<VelocityWindowSnapshot>,
    pub grand_total: u32,
}

/// One cost-router decision row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostRouterDecisionRow {
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub difficulty: f64,
    pub budget_remaining_usd: f64,
    pub chosen_backend: String,
    pub criticality: String,
    pub rationale: String,
    pub ts_ms: i64,
}

/// AutoMix verdict row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutomixVerdictRow {
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub verdict: String,
    pub reason: String,
    pub ts_ms: i64,
}

/// Per-backend free-tier quota status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FreeTierQuotaRow {
    pub backend: String,
    pub status: String,
    pub remaining_pct: u8,
    pub ts_ms: i64,
}

fn cap_limit(limit: Option<usize>) -> usize {
    limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT)
}

/// Filter `cost_token_event` rows to a particular `kind` discriminator
/// without depending on the orchestrator-private SsbPayload type.
fn filter_kind<'a>(
    rows: &'a [SsbActionDetail],
    target_kind: &str,
) -> Vec<&'a SsbActionDetail> {
    rows.iter()
        .filter(|r| {
            let outer = match r.payload_json.as_object() {
                Some(o) => o,
                None => return false,
            };
            let inner = outer
                .get("CostTokenEvent")
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_else(|| outer.clone());
            inner
                .get("kind")
                .and_then(|v| v.as_str())
                .map(|s| s == target_kind)
                .unwrap_or(false)
        })
        .collect()
}

fn details<'a>(row: &'a SsbActionDetail) -> serde_json::Map<String, serde_json::Value> {
    let outer = row
        .payload_json
        .as_object()
        .cloned()
        .unwrap_or_default();
    let inner = outer
        .get("CostTokenEvent")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or(outer);
    inner
        .get("details_json")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default()
}

fn session_id(row: &SsbActionDetail) -> Option<String> {
    let outer = row.payload_json.as_object()?;
    let inner = outer
        .get("CostTokenEvent")
        .and_then(|v| v.as_object())
        .unwrap_or(outer);
    inner
        .get("session_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn task_id(row: &SsbActionDetail) -> Option<String> {
    let outer = row.payload_json.as_object()?;
    let inner = outer
        .get("CostTokenEvent")
        .and_then(|v| v.as_object())
        .unwrap_or(outer);
    inner
        .get("task_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn ts_ms(row: &SsbActionDetail) -> i64 {
    let outer = match row.payload_json.as_object() {
        Some(o) => o,
        None => return 0,
    };
    let inner = outer
        .get("CostTokenEvent")
        .and_then(|v| v.as_object())
        .unwrap_or(outer);
    inner
        .get("ts_ms")
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
}

/// Get the latest velocity-cap snapshot from `velocity_cap_breached`
/// SSB rows. Empty windows envelope when no breaches have occurred.
#[tauri::command]
pub async fn get_velocity_cap_status(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<VelocityCapStatus, String> {
    let cap = cap_limit(limit);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(VelocityCapStatus::default()),
    };
    let raw = match viewer.recent_rows_of_type("cost_token_event", cap) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                action_type = "cost_token_event",
                error = %e,
                "get_velocity_cap_status: viewer error"
            );
            return Ok(VelocityCapStatus::default());
        }
    };
    let mut windows: Vec<VelocityWindowSnapshot> = Vec::new();
    let mut seen: std::collections::BTreeMap<String, VelocityWindowSnapshot> =
        std::collections::BTreeMap::new();
    let filtered = filter_kind(&raw, "velocity_cap_breached");
    let total = filtered.len() as u32;
    for r in filtered {
        let d = details(r);
        let window = d
            .get("window")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if window.is_empty() {
            continue;
        }
        let snap = VelocityWindowSnapshot {
            window: window.clone(),
            count: d.get("count").and_then(|v| v.as_u64()).unwrap_or(0),
            limit: d.get("limit").and_then(|v| v.as_u64()).unwrap_or(0),
            kind: d
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("requests")
                .to_string(),
            ts_ms: ts_ms(r),
        };
        match seen.get(&window) {
            Some(existing) if existing.ts_ms >= snap.ts_ms => {}
            _ => {
                seen.insert(window, snap);
            }
        }
    }
    windows.extend(seen.into_values());
    Ok(VelocityCapStatus {
        windows,
        grand_total: total,
    })
}

/// Get recent `cost_router_decision` rows.
#[tauri::command]
pub async fn get_cost_router_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<CostRouterDecisionRow>, String> {
    let cap = cap_limit(limit);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };
    let raw = match viewer.recent_rows_of_type("cost_token_event", cap) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                action_type = "cost_token_event",
                error = %e,
                "get_cost_router_recent: viewer error"
            );
            return Ok(Vec::new());
        }
    };
    let mut out = Vec::new();
    for r in filter_kind(&raw, "cost_router_decision") {
        let d = details(r);
        out.push(CostRouterDecisionRow {
            session_id: session_id(r),
            task_id: task_id(r),
            difficulty: d.get("difficulty").and_then(|v| v.as_f64()).unwrap_or(0.0),
            budget_remaining_usd: d
                .get("budget_remaining_usd")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            chosen_backend: d
                .get("chosen_backend")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            criticality: d
                .get("criticality")
                .and_then(|v| v.as_str())
                .unwrap_or("medium")
                .to_string(),
            rationale: d
                .get("rationale")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            ts_ms: ts_ms(r),
        });
    }
    Ok(out)
}

/// Get recent `automix_verifier_verdict` rows.
#[tauri::command]
pub async fn get_automix_verdicts_recent(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<AutomixVerdictRow>, String> {
    let cap = cap_limit(limit);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };
    let raw = match viewer.recent_rows_of_type("cost_token_event", cap) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                action_type = "cost_token_event",
                error = %e,
                "get_automix_verdicts_recent: viewer error"
            );
            return Ok(Vec::new());
        }
    };
    let mut out = Vec::new();
    for r in filter_kind(&raw, "automix_verifier_verdict") {
        let d = details(r);
        out.push(AutomixVerdictRow {
            session_id: session_id(r),
            task_id: task_id(r),
            verdict: d
                .get("verdict")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            reason: d
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            ts_ms: ts_ms(r),
        });
    }
    Ok(out)
}

/// Get the latest per-backend free-tier quota status.
#[tauri::command]
pub async fn get_free_tier_quotas(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<FreeTierQuotaRow>, String> {
    let cap = cap_limit(limit);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };
    let raw = match viewer.recent_rows_of_type("cost_token_event", cap) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(
                action_type = "cost_token_event",
                error = %e,
                "get_free_tier_quotas: viewer error"
            );
            return Ok(Vec::new());
        }
    };
    // Merge approaching + exceeded rows; keep the latest per backend.
    let mut seen: std::collections::BTreeMap<String, FreeTierQuotaRow> =
        std::collections::BTreeMap::new();
    for kind in ["free_tier_quota_approaching", "free_tier_quota_exceeded"] {
        for r in filter_kind(&raw, kind) {
            let d = details(r);
            let backend = d
                .get("backend")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if backend.is_empty() {
                continue;
            }
            let row = FreeTierQuotaRow {
                backend: backend.clone(),
                status: if kind == "free_tier_quota_exceeded" {
                    "exceeded".to_string()
                } else {
                    "approaching".to_string()
                },
                remaining_pct: d
                    .get("remaining_pct")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u8,
                ts_ms: ts_ms(r),
            };
            match seen.get(&backend) {
                Some(existing) if existing.ts_ms >= row.ts_ms => {}
                _ => {
                    seen.insert(backend, row);
                }
            }
        }
    }
    Ok(seen.into_values().collect())
}
