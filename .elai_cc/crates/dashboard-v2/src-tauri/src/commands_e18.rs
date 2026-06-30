//! E-18: Tool-use reliability gate dashboard surface (read-only).
//!
//! cross-lang-pair: dashboard/tauri/api
//!
//! Two read-only Tauri commands feed the Svelte `ToolReliabilityInspector`:
//!
//! - [`tool_reliability_status`] — gate configuration + recent SSB row
//!   counts (Pass / Reject / Spill / Escalate). Returns a snapshot
//!   with zeroed counts when the orchestrator has not wired a live
//!   SSB emitter yet (RELIABILITY-PENDING).
//! - [`list_tool_reliability_rows`] — last N
//!   tool_reliability_gate_passed / tool_reliability_gate_rejected /
//!   tool_call_spilled / tool_call_retry_escalated SSB rows,
//!   flattened to a typed DTO the frontend can render without
//!   re-loading the underlying payload.
//!
//! Additive — runs alongside `commands_c35.rs` (tool gate) /
//! `commands_c42.rs` (authz) / `commands_c47.rs` (calibration) without
//! touching them.
//!
//! Reuse-heavy posture: composes ABOVE C-47 (case-base) and BELOW
//! actual dispatch. Composition order:
//!   AuthzPolicy (C-42) -> ToolGate (C-35) -> Calibration consult (C-47)
//!   -> Reliability gate (E-18) -> dispatch.
//!
//! See `workflow/reasoning/E-18-decisions.md`.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 200;
const MAX_LIMIT: usize = 2000;

/// Inline-cap constant mirrored from
/// `elai_orchestrator::tool_reliability::RELIABILITY_GATE_INLINE_CAP`.
/// We restate it here so the dashboard can render the cap without a
/// cross-crate const import at the lib.rs `tauri::generate_handler!`
/// site. The contract test fences the two values can never drift —
/// see crates/contracts/tests/e18_tool_use_reliability_contract.rs.
const RELIABILITY_INLINE_CAP_BYTES: usize = 8 * 1024;

/// Default retry cap mirrored from
/// `elai_orchestrator::tool_reliability::retry::DEFAULT_RETRY_CAP`.
const RELIABILITY_DEFAULT_RETRY_CAP: u32 = 1;

/// Wire shape for the `tool_reliability_status` command. Mirrors the
/// `StatusRow` shape the CLI handler emits so operators can verify
/// the dashboard and `elai reliability status` show the same numbers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolReliabilityStatusDto {
    pub inline_cap_bytes: usize,
    pub default_retry_cap: u32,
    pub spill_dir: String,
    pub pass_count: u64,
    pub reject_count: u64,
    pub spill_count: u64,
    pub escalate_count: u64,
    pub spill_queue_bytes: u64,
}

/// One SSB row of the panel — maps an E-18 payload to the wire shape
/// the panel consumes. Optional fields are populated per row kind:
///   * `reject_reason_hash` on `tool_reliability_gate_rejected`
///   * `spill_path_hash` + `payload_bytes` on `tool_call_spilled`
///   * `attempt` + `escalate_to_role_hash` on `tool_call_retry_escalated`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolReliabilityRowDto {
    pub action_type: String,
    pub tool_name_hash: String,
    pub ts_ms: i64,
    pub reject_reason_hash: Option<String>,
    pub spill_path_hash: Option<String>,
    pub attempt: Option<u32>,
    pub escalate_to_role_hash: Option<String>,
    pub payload_bytes: Option<u64>,
}

/// Return the reliability gate snapshot. When the orchestrator has
/// not wired a live SSB emitter yet, returns the defaults snapshot
/// (cap + retry cap + spill dir) with zeroed counters — the Svelte
/// panel renders the "Reliability gate snapshot unavailable" empty
/// state alongside the config rows.
#[tauri::command]
pub async fn tool_reliability_status(
    state: State<'_, AppState>,
) -> Result<ToolReliabilityStatusDto, String> {
    // Resolve spill dir relative to project state root, sibling to
    // ssb.db + grammar_cache per E-35.
    let spill_dir = ".elai_cc/.elai_cc/spill".to_string();

    let (pass_count, reject_count, spill_count, escalate_count, spill_queue_bytes) =
        if let Some(viewer) = &state.viewer {
            let pass = viewer
                .recent_rows_of_type("tool_reliability_gate_passed", MAX_LIMIT)
                .map(|v| v.len() as u64)
                .unwrap_or(0);
            let rej = viewer
                .recent_rows_of_type("tool_reliability_gate_rejected", MAX_LIMIT)
                .map(|v| v.len() as u64)
                .unwrap_or(0);
            let sp = viewer
                .recent_rows_of_type("tool_call_spilled", MAX_LIMIT)
                .map(|v| v.len() as u64)
                .unwrap_or(0);
            let esc = viewer
                .recent_rows_of_type("tool_call_retry_escalated", MAX_LIMIT)
                .map(|v| v.len() as u64)
                .unwrap_or(0);
            let queue_bytes = viewer
                .recent_rows_of_type("tool_call_spilled", MAX_LIMIT)
                .map(|details| {
                    details
                        .into_iter()
                        .filter_map(|d| {
                            d.payload_json
                                .as_object()
                                .and_then(|m| m.get("payload_bytes"))
                                .and_then(|v| v.as_u64())
                        })
                        .sum::<u64>()
                })
                .unwrap_or(0);
            (pass, rej, sp, esc, queue_bytes)
        } else {
            (0, 0, 0, 0, 0)
        };

    Ok(ToolReliabilityStatusDto {
        inline_cap_bytes: RELIABILITY_INLINE_CAP_BYTES,
        default_retry_cap: RELIABILITY_DEFAULT_RETRY_CAP,
        spill_dir,
        pass_count,
        reject_count,
        spill_count,
        escalate_count,
        spill_queue_bytes,
    })
}

/// Return the last `limit` E-18 SSB rows, newest first. Reads from
/// the shared `SsbViewer` — the orchestrator-side emit path
/// (`OrchestratorReliabilitySsbEmitter`) writes the matching rows.
///
/// Empty Vec when the SSB store is missing or contains no reliability
/// rows yet.
#[tauri::command]
pub async fn list_tool_reliability_rows(
    state: State<'_, AppState>,
    window_ms: Option<i64>,
    limit: Option<usize>,
) -> Result<Vec<ToolReliabilityRowDto>, String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    let action_types = [
        "tool_reliability_gate_passed",
        "tool_reliability_gate_rejected",
        "tool_call_spilled",
        "tool_call_retry_escalated",
    ];
    let mut rows: Vec<ToolReliabilityRowDto> = Vec::with_capacity(action_types.len() * limit);
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
                    "list_tool_reliability_rows: viewer error"
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
    SystemTime::now() // entropy-allow: free-standing window-filter helper, no AppState/ClockProvider in scope; used only to compute a relative cutoff_ms for retain(), not a replayed/persisted value
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn detail_to_row(d: elai_orchestrator::SsbActionDetail) -> Option<ToolReliabilityRowDto> {
    let obj = d.payload_json.as_object()?;
    let tool_name_hash = obj
        .get("tool_name_hash")
        .and_then(|v| v.as_u64())
        .map(|h| format!("{h:016x}"))
        .unwrap_or_default();
    let reject_reason_hash = obj
        .get("reject_reason_hash")
        .and_then(|v| v.as_u64())
        .map(|h| format!("{h:016x}"));
    let spill_path_hash = obj
        .get("spill_path_hash")
        .and_then(|v| v.as_u64())
        .map(|h| format!("{h:016x}"));
    let attempt = obj
        .get("attempt")
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok());
    let escalate_to_role_hash = obj
        .get("escalate_to_role_hash")
        .and_then(|v| v.as_u64())
        .map(|h| format!("{h:016x}"));
    let payload_bytes = obj.get("payload_bytes").and_then(|v| v.as_u64());
    Some(ToolReliabilityRowDto {
        action_type: d.action_type.clone(),
        tool_name_hash,
        ts_ms: d.ts,
        reject_reason_hash,
        spill_path_hash,
        attempt,
        escalate_to_role_hash,
        payload_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_dto_round_trips_serde() {
        let s = ToolReliabilityStatusDto {
            inline_cap_bytes: RELIABILITY_INLINE_CAP_BYTES,
            default_retry_cap: RELIABILITY_DEFAULT_RETRY_CAP,
            spill_dir: ".elai_cc/.elai_cc/spill".into(),
            pass_count: 12,
            reject_count: 3,
            spill_count: 1,
            escalate_count: 0,
            spill_queue_bytes: 16_384,
        };
        let j = serde_json::to_string(&s).expect("serialise");
        let back: ToolReliabilityStatusDto = serde_json::from_str(&j).expect("parse");
        assert_eq!(back, s);
    }

    #[test]
    fn row_dto_round_trips_serde() {
        let r = ToolReliabilityRowDto {
            action_type: "tool_reliability_gate_rejected".into(),
            tool_name_hash: "deadbeef".into(),
            ts_ms: 42,
            reject_reason_hash: Some("0123abcd".into()),
            spill_path_hash: None,
            attempt: None,
            escalate_to_role_hash: None,
            payload_bytes: None,
        };
        let j = serde_json::to_string(&r).expect("serialise");
        let back: ToolReliabilityRowDto = serde_json::from_str(&j).expect("parse");
        assert_eq!(back, r);
    }
}
