//! C-47: Case-based calibration dashboard surface (read-only).
//!
//! cross-lang-pair: dashboard/tauri/api
//!
//! Two read-only Tauri commands feed the Svelte `CaseBaseInspector`:
//!
//! - [`tool_calibration_status`] — per-tool case-base stats
//!   (shape_count, total_calls, error_count, last_ts_ms). Returns
//!   an empty Vec when the orchestrator has not wired a live
//!   `ToolCalibrationRegistry` yet (CALIBRATION-PENDING).
//! - [`list_tool_calibration_rows`] — last N tool_call_logged /
//!   tool_calibration_hint_emitted / tool_calibration_drift_detected
//!   SSB rows, flattened to a typed DTO the frontend can render
//!   without re-loading the underlying payload.
//!
//! Additive — runs alongside the existing `commands_c35.rs` (tool
//! gate) / `commands_c42.rs` (authz) / `commands_c43.rs` (freeze) /
//! `commands_c46.rs` handlers without touching them.
//!
//! Paper grounding: arXiv:2605.15041 (CAST — Case-Based Calibration of
//! Adaptive Reasoning and Execution for LLM Tool Use). See
//! workflow/papers/case-based-calibration.md.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 200;
const MAX_LIMIT: usize = 2000;

/// Per-tool case-base status wire shape consumed by the Svelte
/// `CaseBaseInspector` panel. Mirrors the
/// `elai_plugins::ToolCalibrationStatus` Rust struct one-for-one so
/// the TS DTO at `dashboard-v2/src/lib/api/stub.ts` deserializes
/// without translation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCalibrationStatusDto {
    pub tool_name: String,
    pub shape_count: u32,
    pub total_calls: u32,
    pub error_count: u32,
    pub last_ts_ms: i64,
}

/// One SSB row of the panel — maps a C-47 payload to the wire shape
/// the panel consumes.
///
/// Note: omits `Eq` because `error_rate` is `Option<f32>` (NaN not
/// comparable for total Eq); structural equality uses `PartialEq`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCalibrationRowDto {
    /// blake3-prefix u64 of the tool name (rendered as hex).
    pub tool_name_hash: String,
    pub call_shape_hash: String,
    /// One of `"tool_call_logged"` / `"tool_calibration_hint_emitted"` /
    /// `"tool_calibration_drift_detected"`.
    pub action_type: String,
    pub ts_ms: i64,
    /// Present on `tool_call_logged` rows only.
    pub post_call_audit_ok: Option<bool>,
    /// Present on `tool_calibration_drift_detected` rows only.
    pub error_rate: Option<f32>,
    /// Present on `tool_call_logged` + `tool_calibration_drift_detected`.
    pub total_calls: Option<u32>,
}

/// Return per-tool calibration status. When the orchestrator has
/// not wired a live registry yet, returns an empty Vec — the Svelte
/// panel renders the `"Calibration registry unavailable"` empty
/// state.
#[tauri::command]
pub async fn tool_calibration_status(
    state: State<'_, AppState>,
) -> Result<Vec<ToolCalibrationStatusDto>, String> {
    // C-47b: when the calibration registry is wired (mcp feature + env gate),
    // read live statuses from it. Falls back to empty Vec when the registry is
    // absent (env gate off or feature compiled out) -- the Svelte panel renders
    // the "Calibration registry unavailable" empty state in that case.
    #[cfg(feature = "mcp")]
    {
        if let Some(reg) = state.calibration_registry.as_ref() {
            let statuses = reg.statuses().await;
            let mut rows: Vec<ToolCalibrationStatusDto> = statuses
                .into_iter()
                .map(|s| ToolCalibrationStatusDto {
                    tool_name: s.tool_name,
                    shape_count: s.shape_count,
                    total_calls: s.total_calls,
                    error_count: s.error_count,
                    last_ts_ms: s.last_ts_ms,
                })
                .collect();
            rows.sort_by(|a, b| a.tool_name.cmp(&b.tool_name));
            return Ok(rows);
        }
    }
    #[cfg(not(feature = "mcp"))]
    let _ = &state;
    Ok(Vec::new())
}

/// Return the last `limit` C-47 SSB rows, newest first. Reads from
/// the shared `SsbViewer` — the orchestrator-side emit path
/// (`OrchestratorCaseBaseSsbEmitter`) writes the matching rows.
///
/// Empty Vec when the SSB store is missing or contains no
/// calibration rows yet.
#[tauri::command]
pub async fn list_tool_calibration_rows(
    state: State<'_, AppState>,
    window_ms: Option<i64>,
    limit: Option<usize>,
) -> Result<Vec<ToolCalibrationRowDto>, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    let action_types = [
        "tool_call_logged",
        "tool_calibration_hint_emitted",
        "tool_calibration_drift_detected",
    ];
    let mut rows: Vec<ToolCalibrationRowDto> =
        Vec::with_capacity(action_types.len() * limit);
    for at in action_types {
        match viewer.recent_rows_of_type(at, limit) {
            Ok(details) => {
                for d in details {
                    if let Some(row) = detail_to_row(d) {
                        rows.push(row);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    action_type = at,
                    error = %e,
                    "list_tool_calibration_rows: viewer error"
                );
            }
        }
    }

    rows.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms).then(Ordering::Equal));
    if let Some(window) = window_ms {
        if window > 0 {
            let cutoff_ms = now_ms().saturating_sub(window);
            rows.retain(|r| r.ts_ms >= cutoff_ms);
        }
    }
    rows.truncate(limit);
    Ok(rows)
}

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now() // entropy-allow: observability ts-ms for SSB window cutoff
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn detail_to_row(d: elai_orchestrator::SsbActionDetail) -> Option<ToolCalibrationRowDto> {
    let obj = d.payload_json.as_object()?;
    let tool_name_hash = obj
        .get("tool_name_hash")
        .and_then(|v| v.as_u64())
        .map(|h| format!("{h:016x}"))
        .unwrap_or_default();
    let call_shape_hash = obj
        .get("call_shape_hash")
        .and_then(|v| v.as_u64())
        .map(|h| format!("{h:016x}"))
        .unwrap_or_default();
    let post_call_audit_ok = obj.get("post_call_audit_ok").and_then(|v| v.as_bool());
    let error_rate = obj
        .get("error_rate")
        .and_then(|v| v.as_f64())
        .map(|x| x as f32);
    let total_calls = obj
        .get("total_calls")
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok());
    Some(ToolCalibrationRowDto {
        tool_name_hash,
        call_shape_hash,
        action_type: d.action_type.clone(),
        ts_ms: d.ts,
        post_call_audit_ok,
        error_rate,
        total_calls,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_dto_round_trips_serde() {
        let s = ToolCalibrationStatusDto {
            tool_name: "fs.read".into(),
            shape_count: 3,
            total_calls: 10,
            error_count: 4,
            last_ts_ms: 1_700_000_000_000,
        };
        let j = serde_json::to_string(&s).expect("serialise");
        let back: ToolCalibrationStatusDto = serde_json::from_str(&j).expect("parse");
        assert_eq!(back, s);
    }

    #[test]
    fn row_dto_round_trips_serde() {
        let r = ToolCalibrationRowDto {
            tool_name_hash: "deadbeef".into(),
            call_shape_hash: "0123abcd".into(),
            action_type: "tool_call_logged".into(),
            ts_ms: 42,
            post_call_audit_ok: Some(true),
            error_rate: None,
            total_calls: Some(7),
        };
        let j = serde_json::to_string(&r).expect("serialise");
        let back: ToolCalibrationRowDto = serde_json::from_str(&j).expect("parse");
        assert_eq!(back, r);
    }
}
