//! G-4 / Slice A: `McpRugPullBlocked` SSB read surface for the Svelte
//! `McpRugPullDrawer`.
//!
//! One read-only command:
//!
//! - [`get_mcp_rug_pulls`] -- last N `mcp_rug_pull_blocked` rows emitted by the
//!   live MCP bridge (`DefaultMcpBridge::http_list_tools` re-pin gate) when a
//!   reconnect-time `tools/list` re-fetch from a previously-approved server no
//!   longer matches the blake3 schema pin. The drawer renders the server, the
//!   before/after schema hashes, the changed tool names, and the description-
//!   diff count so an operator can decide whether to re-approve (`update_pin`).
//!
//! Default-ON producer: the re-pin gate ships armed; the only opt-out is the
//! kill-switch `ELAI_MCP_REPIN_DISABLE` (read at bridge construction). The read
//! command itself is NOT feature-gated: it always queries the SSB viewer and
//! returns whatever rows exist (an empty Vec on a fresh install / when no
//! rug-pull has occurred).
//!
//! Sibling of `commands_f13` (SessionArchiveDrawer surface): one command per
//! distinct Svelte drawer key (W-1 wiring doctrine).
//!
//! cross-lang-pair: dashboard/tauri/api

use serde_json::{json, Value};
use tauri::State;

use elai_orchestrator::ssb_viewer::McpRugPullRow;

use crate::state::AppState;

/// Local Tauri-command result alias (mirrors the sibling `commands_f*` modules;
/// `commands.rs::CmdResult` is module-private).
type CmdResult<T> = Result<T, String>;

/// G-4 — dashboard read cap for the rug-pull feed. The viewer clamps to its own
/// `DRAWER_QUERY_HARD_CAP` (500); this is the dashboard-side request size, kept
/// well under that ceiling so the IPC payload stays small at the drawer's poll
/// cadence.
const MCP_RUG_PULLS_FEED_CAP: usize = 200;

/// List the `mcp_rug_pull_blocked` SSB rows newest-first. Each row records a
/// live MCP rug-pull / line-jump that was blocked fail-closed until operator
/// re-approval.
///
/// Returns an empty array when no SSB DB is reachable (fresh install) or when
/// no rug-pull has been recorded. Serde serializes `McpRugPullRow`'s field
/// names verbatim, so the frontend `McpRugPull` DTO must mirror them.
#[tauri::command]
pub fn get_mcp_rug_pulls(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<McpRugPullRow> = viewer.mcp_rug_pulls(MCP_RUG_PULLS_FEED_CAP).unwrap_or_default();
    serde_json::to_value(rows).map_err(|e| e.to_string())
}
