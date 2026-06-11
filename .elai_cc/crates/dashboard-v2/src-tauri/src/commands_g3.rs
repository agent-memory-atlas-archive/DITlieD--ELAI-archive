//! G-3 / Slice A: `BlastRadiusBlocked` SSB read surface for the Svelte
//! `BlastRadiusDrawer`.
//!
//! One read-only command:
//!
//! - [`get_blast_radius_blocks`] -- last N `blast_radius_blocked` rows emitted by
//!   the deterministic runtime blast-radius gates. Each row records ONE
//!   fail-closed block: either a shell-EXPANDED filesystem-mutating target that
//!   resolved OUTSIDE the per-run worktree root (`kind = fs_escape`, emitted from
//!   `production_agent_rpc::translate_tool_request` / `run_bash`), or an outbound
//!   egress host that carried a NUL/control char or resolved outside the pinned
//!   allow-set (`kind = egress_denied`, emitted from
//!   `WebResearchOrchestrator::fetch_and_extract`). The drawer renders the
//!   pre- vs post-expansion command diff (the `#10077` forensics fix) so an
//!   operator can see exactly what the shell would have run / connected to.
//!
//! Default-ON producer: both gates ship armed; the only opt-out is the
//! kill-switch `ELAI_BLAST_RADIUS_DISABLE` (read at each gate call site). The
//! read command itself is NOT feature-gated: it always queries the SSB viewer
//! and returns whatever rows exist (an empty Vec on a fresh install / when no
//! block has occurred).
//!
//! Sibling of `commands_g4` (McpRugPullDrawer surface): one command per distinct
//! Svelte drawer key (W-1 wiring doctrine).
//!
//! cross-lang-pair: dashboard/tauri/api

use serde_json::{json, Value};
use tauri::State;

use elai_orchestrator::ssb_viewer::BlastRadiusBlockRow;

use crate::state::AppState;

/// Local Tauri-command result alias (mirrors the sibling `commands_g*` modules;
/// `commands.rs::CmdResult` is module-private).
type CmdResult<T> = Result<T, String>;

/// G-3 — dashboard read cap for the blast-radius feed. The viewer clamps to its
/// own `DRAWER_QUERY_HARD_CAP` (500); this is the dashboard-side request size,
/// kept well under that ceiling so the IPC payload stays small at the drawer's
/// poll cadence.
const BLAST_RADIUS_FEED_CAP: usize = 200;

/// List the `blast_radius_blocked` SSB rows newest-first. Each row records a
/// deterministic runtime gate block (FS escape or egress denial) that was
/// refused fail-closed.
///
/// Returns an empty array when no SSB DB is reachable (fresh install) or when no
/// block has been recorded. Serde serializes `BlastRadiusBlockRow`'s field names
/// verbatim, so the frontend `BlastRadiusBlock` DTO must mirror them.
#[tauri::command]
pub fn get_blast_radius_blocks(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<BlastRadiusBlockRow> = viewer
        .blast_radius_blocks(BLAST_RADIUS_FEED_CAP)
        .unwrap_or_default();
    serde_json::to_value(rows).map_err(|e| e.to_string())
}
