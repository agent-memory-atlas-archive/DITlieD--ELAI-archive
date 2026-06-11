//! G-8 / Slice A: `ProvenanceAttestationEmitted` SSB read surface for the Svelte
//! `AttestationDrawer`.
//!
//! One read-only command:
//!
//! - [`get_attestations`] -- last N `provenance_attestation_emitted` rows emitted
//!   at the `elai run` exit when a signed in-toto v1 provenance attestation is
//!   built from the sealed G-2 Reproducible-Run Artifact. Each row records ONE
//!   attestation: the blake3-Merkle subject digest (the value the git trailer
//!   carries + `elai verify-attestation` recomputes offline), the G-2
//!   run-artifact root the attestation chains to, the F-gate `verified_levels`
//!   rollup count, an all-passed flag, and the honest signing model
//!   (`local_blake3_merkle`). The drawer renders the verifiedLevels rollup so an
//!   operator can read the run's off-machine trust posture without re-running
//!   `elai verify-attestation`.
//!
//! Default-ON producer: the attestation is emitted at run end; the only opt-out
//! is the kill-switch `ELAI_ATTESTATION_DISABLE`. The read command itself is NOT
//! feature-gated: it always queries the SSB viewer and returns whatever rows
//! exist (an empty Vec on a fresh install / when no run has sealed one yet).
//!
//! Sibling of `commands_g2` (RraViewerPanel surface) and `commands_g3`
//! (BlastRadiusDrawer surface): one command per distinct Svelte panel key (W-1
//! wiring doctrine).
//!
//! cross-lang-pair: dashboard/tauri/api

use serde_json::{json, Value};
use tauri::State;

use elai_orchestrator::ssb_viewer::AttestationRow;

use crate::state::AppState;

/// Local Tauri-command result alias (mirrors the sibling `commands_g*` modules;
/// `commands.rs::CmdResult` is module-private).
type CmdResult<T> = Result<T, String>;

/// G-8 — dashboard read cap for the attestation feed. The viewer clamps to its
/// own `DRAWER_QUERY_HARD_CAP` (500); this is the dashboard-side request size,
/// kept well under that ceiling so the IPC payload stays small at the drawer's
/// poll cadence.
const ATTESTATION_FEED_CAP: usize = 200;

/// List the `provenance_attestation_emitted` SSB rows newest-first. Each row
/// records ONE signed in-toto provenance attestation: the blake3-Merkle subject
/// digest, the G-2 run-artifact root, the verified-levels rollup count, an
/// all-passed flag, and the honest signing model.
///
/// Returns an empty array when no SSB DB is reachable (fresh install) or when no
/// attestation has been emitted. Serde serializes `AttestationRow`'s field names
/// verbatim, so the frontend `Attestation` DTO must mirror them.
#[tauri::command]
pub fn get_attestations(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<AttestationRow> = viewer.attestations(ATTESTATION_FEED_CAP).unwrap_or_default();
    serde_json::to_value(rows).map_err(|e| e.to_string())
}
