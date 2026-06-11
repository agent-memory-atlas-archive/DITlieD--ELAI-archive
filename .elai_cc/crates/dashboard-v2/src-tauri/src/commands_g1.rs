//! G-1 / Slice A: `TaintFlow` SSB read surface for the Svelte `TaintFlowDrawer`.
//!
//! One read-only command:
//!
//! - [`get_taint_flows`] -- last N `taint_flow` rows emitted by the live
//!   data-plane taint gate (`production_agent_rpc::translate_tool_request`
//!   git_commit / edit arms) when an untrusted ingest (web body / MCP tool
//!   output / external file read / sandbox stdout) flowed into a destructive
//!   sink and the `allow_tool_with_origin` capability gate REJECTED the call.
//!   The drawer renders the ingest Kind, the origin blake3, the trust level,
//!   and the blocked sink so an operator can see the data flow that was stopped.
//!
//! Default-ON producer: the taint gate ships armed; the only opt-out is the
//! kill-switch `ELAI_TAINT_PROPAGATE_DISABLE` (read at the gate call site). The
//! read command itself is NOT feature-gated: it always queries the SSB viewer
//! and returns whatever rows exist (an empty Vec on a fresh install / when no
//! tainted flow has been blocked).
//!
//! Sibling of `commands_g4` (McpRugPullDrawer surface): one command per distinct
//! Svelte drawer key (W-1 wiring doctrine).
//!
//! cross-lang-pair: dashboard/tauri/api

use serde_json::{json, Value};
use tauri::State;

use elai_orchestrator::ssb_viewer::TaintFlowRow;

use crate::state::AppState;

/// Local Tauri-command result alias (mirrors the sibling `commands_g4` module).
type CmdResult<T> = Result<T, String>;

/// G-1 — dashboard read cap for the taint-flow feed. The viewer clamps to its
/// own `DRAWER_QUERY_HARD_CAP` (500); this is the dashboard-side request size,
/// kept well under that ceiling so the IPC payload stays small at the drawer's
/// poll cadence.
const TAINT_FLOWS_FEED_CAP: usize = 200;

/// List the `taint_flow` SSB rows newest-first. Each row records a live
/// data-plane taint propagation that was BLOCKED at a destructive sink by the
/// capability gate.
///
/// Returns an empty array when no SSB DB is reachable (fresh install) or when
/// no tainted flow has been blocked. Serde serializes `TaintFlowRow`'s field
/// names verbatim, so the frontend `TaintFlow` DTO must mirror them.
#[tauri::command]
pub fn get_taint_flows(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<TaintFlowRow> = viewer.taint_flows(TAINT_FLOWS_FEED_CAP).unwrap_or_default();
    serde_json::to_value(rows).map_err(|e| e.to_string())
}
