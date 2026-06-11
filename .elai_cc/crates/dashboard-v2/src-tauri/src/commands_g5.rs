//! G-5 / Slice A: `ReviewSurfaceScored` SSB read surface for the Svelte
//! `CockpitWindow`.
//!
//! One read-only command:
//!
//! - [`get_review_surface`] -- the most recent `review_surface_scored` rows the
//!   `elai review --cockpit` producer emits. Each row records ONE reviewer-
//!   cockpit Diff-Surface-to-Review (DSR) score: the DSR ratio (un-vouched
//!   changed lines / total changed lines), the vouched / un-vouched / total
//!   changed-line counts, and the pinned diff-base commit. The window renders
//!   the headline DSR + the vouched/un-vouched split so a reviewer can read the
//!   review surface without re-running the score.
//!
//! The PRODUCER (`elai review --cockpit`) is DEFAULT-ON; the only opt-out is the
//! kill-switch `ELAI_REVIEW_COCKPIT_DISABLE`. This READ command is NOT feature-
//! gated: it always queries the SSB viewer and returns whatever rows exist (an
//! empty Vec on a fresh install / when no review has been scored yet).
//!
//! This is the read-only CONSUMER of the off-agent `review_surface_scored` row
//! (DD4): the CLI is the producer that holds the live `Arc<Ssb>`; this command
//! never writes.
//!
//! Sibling of `commands_g2` (RraViewerPanel surface) and `commands_g3`
//! (BlastRadiusDrawer surface): one command per distinct Svelte panel key.
//!
//! cross-lang-pair: dashboard/tauri/api

use serde_json::{json, Value};
use tauri::State;

use elai_orchestrator::ssb_viewer::ReviewSurfaceRow;

use crate::state::AppState;

/// Local Tauri-command result alias (mirrors the sibling `commands_g*` modules).
type CmdResult<T> = Result<T, String>;

/// G-5 — dashboard read cap for the review-surface feed. The viewer clamps to
/// its own `DRAWER_QUERY_HARD_CAP` (500); this is the dashboard-side request
/// size, kept well under that ceiling so the IPC payload stays small at the
/// window's poll cadence.
const REVIEW_SURFACE_FEED_CAP: usize = 200;

/// List the `review_surface_scored` SSB rows newest-first. Each row records ONE
/// reviewer-cockpit DSR score: the DSR ratio, the vouched / un-vouched / total
/// changed-line counts, and the pinned diff-base commit.
///
/// Returns an empty array when no SSB DB is reachable (fresh install) or when no
/// review has been scored. Serde serializes `ReviewSurfaceRow`'s field names
/// verbatim, so the frontend `ReviewSurface` DTO must mirror them.
#[tauri::command]
pub fn get_review_surface(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<ReviewSurfaceRow> = viewer
        .review_surfaces(REVIEW_SURFACE_FEED_CAP)
        .unwrap_or_default();
    serde_json::to_value(rows).map_err(|e| e.to_string())
}
