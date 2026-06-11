//! G-7 / Slice A: `ClaimConfidenceScored` + `TrifectaQuarantined` SSB read
//! surface for the Svelte `ConfidenceDrawer`.
//!
//! Two read-only commands:
//!
//! - [`get_claim_confidence`] -- last N `claim_confidence_scored` rows. Each row
//!   records ONE live per-change conformal decision carrying the REAL fused
//!   `claim_score` (log-prob LOCAL-only + self-consistency + exec-agreement
//!   components), the lane, tau, the `decide()` verdict and the abstained flag.
//!   The cockpit confidence-triage band reads these to surface low-confidence
//!   hunks first (the G-5 link).
//! - [`get_trifecta_quarantines`] -- last N `trifecta_quarantined` rows. Each
//!   row records ONE tool-call chokepoint where the secret-present +
//!   untrusted-span + egress legs co-occurred and the untrusted span was held
//!   back from the tool-caller. Carries the blake3 of the untrusted origin
//!   (NEVER the raw span), the sink tool, the three component flags, the
//!   quarantined flag and the no-tool sub-LLM verdict.
//!
//! Default state: the per-change confidence producer is gated by the EXISTING
//! conformal-refusal triple gate; the quarantine gate is default-ON (kill-switch
//! `ELAI_TRIFECTA_QUARANTINE_DISABLE` only). The read commands are NOT
//! feature-gated: they always query the SSB viewer and return whatever rows
//! exist (an empty array on a fresh install / when neither has fired).
//!
//! Sibling of `commands_g9` (RedactionPanel surface): one command per distinct
//! read surface (W-1 wiring doctrine).
//!
//! cross-lang-pair: dashboard/tauri/api

use serde_json::{json, Value};
use tauri::State;

use elai_orchestrator::ssb_viewer::{ClaimConfidenceRow, TrifectaQuarantineRow};

use crate::state::AppState;

/// Local Tauri-command result alias (mirrors the sibling `commands_g*` modules;
/// `commands.rs::CmdResult` is module-private).
type CmdResult<T> = Result<T, String>;

/// G-7 — dashboard read cap for the confidence feed. The viewer clamps to its
/// own `DRAWER_QUERY_HARD_CAP` (500); this is the dashboard-side request size,
/// kept well under that ceiling so the IPC payload stays small at the panel's
/// poll cadence.
const CONFIDENCE_FEED_CAP: usize = 200;

/// List the `claim_confidence_scored` SSB rows newest-first. Each row records
/// ONE live per-change conformal decision: the REAL fused claim_score + the
/// three per-signal components + the lane + tau + the verdict + abstained flag.
///
/// Returns an empty array when no SSB DB is reachable (fresh install) or when no
/// per-change decision has fired. Serde serializes `ClaimConfidenceRow`'s field
/// names verbatim, so the frontend `ClaimConfidence` DTO must mirror them.
#[tauri::command]
pub fn get_claim_confidence(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<ClaimConfidenceRow> =
        viewer.claim_confidences(CONFIDENCE_FEED_CAP).unwrap_or_default();
    serde_json::to_value(rows).map_err(|e| e.to_string())
}

/// List the `trifecta_quarantined` SSB rows newest-first. Each row records ONE
/// lethal-trifecta quarantine: the untrusted-origin blake3 + sink tool + the
/// three component flags + quarantined flag + the no-tool sub-LLM verdict.
/// NEVER the raw secret or untrusted span.
///
/// Returns an empty array when no SSB DB is reachable or when no quarantine has
/// fired. Serde serializes `TrifectaQuarantineRow`'s field names verbatim, so
/// the frontend `TrifectaQuarantine` DTO must mirror them.
#[tauri::command]
pub fn get_trifecta_quarantines(state: State<'_, AppState>) -> CmdResult<Value> {
    let Some(viewer) = state.viewer.as_ref() else {
        return Ok(json!([]));
    };
    let rows: Vec<TrifectaQuarantineRow> = viewer
        .trifecta_quarantines(CONFIDENCE_FEED_CAP)
        .unwrap_or_default();
    serde_json::to_value(rows).map_err(|e| e.to_string())
}
