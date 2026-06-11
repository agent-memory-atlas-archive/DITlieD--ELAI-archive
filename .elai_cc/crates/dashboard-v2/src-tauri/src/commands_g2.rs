//! G-2 / Slice A: `RunArtifactEmitted` SSB read surface for the Svelte
//! `RraViewerPanel`.
//!
//! One read-only command:
//!
//! - [`get_run_artifact`] -- last N `run_artifact_emitted` rows emitted at the
//!   `elai run` exit (the `run_async` flush, mirroring `--export-ssb`) when a
//!   Reproducible-Run Artifact (`.rra`) bundle is sealed. The panel renders the
//!   blake3 root, the honest per-lane guarantee level, the F-gate evidence-
//!   manifest entry count, the pinned `ELAI_SEED`, and the session id so an
//!   operator can read the artifact's trust posture without opening the file.
//!
//! Default-ON producer: the RRA is sealed at run end; the only opt-out is the
//! kill-switch `ELAI_RRA_DISABLE`. The read command itself is NOT feature-
//! gated: it always queries the SSB viewer and returns whatever rows exist (an
//! empty Vec on a fresh install / when no run has sealed an artifact yet).
//!
//! Sibling of `commands_g4` (McpRugPullDrawer surface) and `commands_f13`
//! (SessionArchiveDrawer surface): one command per distinct Svelte panel key
//! (W-1 wiring doctrine).
//!
//! cross-lang-pair: dashboard/tauri/api

use serde_json::{json, Value};
use tauri::State;

use elai_orchestrator::ssb_viewer::RunArtifactRow;

use crate::state::AppState;

/// Local Tauri-command result alias (mirrors the sibling `commands_f*` /
/// `commands_g*` modules; `commands.rs::CmdResult` is module-private).
type CmdResult<T> = Result<T, String>;

/// G-2 — dashboard read cap for the run-artifact feed. The viewer clamps to its
/// own `DRAWER_QUERY_HARD_CAP` (500); this is the dashboard-side request size,
/// kept well under that ceiling so the IPC payload stays small at the panel's
/// poll cadence.
const RUN_ARTIFACTS_FEED_CAP: usize = 200;

/// List the `run_artifact_emitted` SSB rows newest-first. Each row records ONE
/// sealed Reproducible-Run Artifact (`.rra`): the blake3 root, the honest
/// per-lane guarantee level, the F-gate evidence-manifest count, and the pinned
/// seed.
///
/// Returns an empty array when no SSB DB is reachable (fresh install) or when
/// no artifact has been sealed. Serde serializes `RunArtifactRow`'s field names
/// verbatim, so the frontend `RunArtifact` DTO must mirror them.
#[tauri::command]
pub fn get_run_artifact(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<RunArtifactRow> = viewer.run_artifacts(RUN_ARTIFACTS_FEED_CAP).unwrap_or_default();
    serde_json::to_value(rows).map_err(|e| e.to_string())
}
