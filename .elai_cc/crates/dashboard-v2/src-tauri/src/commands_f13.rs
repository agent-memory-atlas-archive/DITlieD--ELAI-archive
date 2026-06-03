//! F-13 / Slice A5: `SessionArchived` SSB read surface for the Svelte
//! `SessionArchiveDrawer`.
//!
//! One read-only command:
//!
//! - [`get_session_archives`] -- last N `session_archived` rows emitted at the
//!   `run_migration` destructive-DROP site (the `Ssb::new` production open path)
//!   when a stale-version DB is opened with `ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE`
//!   set; the stale `ssb.db` is cold-archived to
//!   `workflow/sessions/{id}/archive/ssb-v{N}.sqlite` BEFORE the DROP.
//!
//! The drawer renders the archived session id, the stale schema version, the
//! archive path, and the blake3 manifest prefix so an operator can locate and
//! integrity-check the cold archive.
//!
//! This feature is DEFAULT-OFF by design (forensic opt-in; the producer-side
//! `ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE` gate is wire-dark, registered in
//! wire-dark-allowlist.toml). The read command itself is NOT feature-gated: it
//! always queries the SSB viewer and returns whatever rows exist (an empty Vec
//! on the default install where the gate was never enabled).
//!
//! Sibling of `commands_f7` (TemplateRankingDrawer surface): one command per
//! distinct Svelte drawer key (W-1 wiring doctrine).
//!
//! cross-lang-pair: dashboard/tauri/api

use serde_json::{json, Value};
use tauri::State;

use elai_orchestrator::ssb_viewer::SessionArchiveRow;

use crate::state::AppState;

/// Local Tauri-command result alias (mirrors the sibling `commands_f*` modules;
/// `commands.rs::CmdResult` is module-private).
type CmdResult<T> = Result<T, String>;

/// F-13 / Slice A5 — dashboard read cap for the session-archive feed. The
/// viewer clamps to its own `DRAWER_QUERY_HARD_CAP` (500); this is the
/// dashboard-side request size, kept well under that ceiling so the IPC payload
/// stays small at the drawer's poll cadence.
const SESSION_ARCHIVES_FEED_CAP: usize = 200;

/// List the `session_archived` SSB rows newest-first. Each row records a
/// stale-version store that was cold-archived BEFORE a destructive
/// `run_migration` DROP.
///
/// Returns an empty array when no SSB DB is reachable (fresh install) or when
/// the archive gate was never enabled. Serde serializes `SessionArchiveRow`'s
/// field names verbatim, so the frontend `SessionArchive` DTO must mirror them.
#[tauri::command]
pub fn get_session_archives(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<SessionArchiveRow> = viewer
        .session_archives(SESSION_ARCHIVES_FEED_CAP)
        .unwrap_or_default();
    serde_json::to_value(rows).map_err(|e| e.to_string())
}
