//! Phase 4J — Empire bridge dashboard surface (read-only).
//!
//! cross-lang-pair: dashboard/tauri/api
//!
//! Two read-only commands feed the Svelte `EmpireBridgePanel`:
//!
//! - [`get_empire_status`] — current bridge state
//!   (`Unavailable | Disconnected | Connecting | Connected`).
//! - [`list_empire_factory_lines`] — Vec<FactoryLine> snapshot for the
//!   panel's connection list.
//!
//! Status mapping:
//!   - `Unavailable`   = the orchestrator was NOT built with the `empire`
//!                       feature; the panel renders "Empire bridge
//!                       unavailable; activate when factory-brain deployed".
//!   - `Disconnected`  = empire feature is on but the bridge has not yet
//!                       successfully connected (live until factory-brain
//!                       deploys; matches the host-side scaffolding state).
//!   - `Connecting`    = mid-connect (EMPIRE-PENDING; surfaced once
//!                       `EmpireBridge::connect` returns Ok).
//!   - `Connected`     = live link (EMPIRE-PENDING; surfaced once
//!                       `EmpireBridge::connect` returns Ok).
//!
//! Sibling of `commands_e35` — uses the same placeholder-response shape so
//! the wire format is stable across "feature off" and "feature on, bridge
//! Disconnected". When the producer side (the orchestrator's `EmpireBridge`)
//! lands, this module gains a `state.empire_bridge: Option<Arc<EmpireBridge>>`
//! lookup and the placeholders flip to live reads in one place.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

type CmdResult<T> = Result<T, String>;

/// Bridge status wire shape consumed by `EmpireBridgePanel.svelte`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmpireStatusDto {
    /// `"unavailable" | "disconnected" | "connecting" | "connected"`.
    pub state: String,
    /// True when the orchestrator was built with the `empire` feature.
    pub feature_enabled: bool,
    /// Number of FactoryLine rows the bridge currently sees.
    pub factory_line_count: u32,
    /// Free-form human note. Used to surface the EMPIRE-PENDING reason.
    pub note: String,
}

/// One FactoryLine row rendered by `EmpireBridgePanel.svelte`. Mirrors
/// `elai_orchestrator::empire::FactoryLine` but serde-stable so the wire
/// shape does not couple to the orchestrator-internal type when the feature
/// is enabled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactoryLineDto {
    pub brain_id: String,
    /// `"MKI" | "MKII" | "MKIII"`.
    pub engine_tier: String,
    pub neuron_count: u32,
    pub last_heartbeat_ms: i64,
    pub status: String,
}

#[cfg(not(feature = "empire"))]
const NOTE_UNAVAILABLE: &str =
    "Empire bridge unavailable: orchestrator built without the `empire` feature. \
     Enable feature + deploy factory-brain to activate.";

#[cfg(feature = "empire")]
const NOTE_DISCONNECTED: &str =
    "Empire bridge present but not connected. EMPIRE-PENDING: factory-brain socket boot.";

/// Return the current EmpireBridge status.
///
/// When the orchestrator is built without the `empire` feature, returns
/// `state = "unavailable"` and a zero factory-line count. The dashboard
/// renders this as an empty-state notice; no error is surfaced.
#[tauri::command]
pub async fn get_empire_status(state: State<'_, AppState>) -> CmdResult<EmpireStatusDto> {
    let (state_label, feature_enabled, count, note) = empire_status_inner(&state);
    Ok(EmpireStatusDto {
        state: state_label.to_string(),
        feature_enabled,
        factory_line_count: count,
        note: note.to_string(),
    })
}

/// Return the current FactoryLine snapshot. Empty when the bridge is
/// `Unavailable` or `Disconnected` (graceful degradation).
#[tauri::command]
pub async fn list_empire_factory_lines(
    state: State<'_, AppState>,
) -> CmdResult<Vec<FactoryLineDto>> {
    Ok(empire_factory_lines_inner(&state))
}

// ---------------------------------------------------------------------------
// Feature-conditional implementation seam
// ---------------------------------------------------------------------------

#[cfg(feature = "empire")]
fn empire_status_inner(state: &AppState) -> (&'static str, bool, u32, &'static str) {
    use elai_orchestrator::empire::{BridgeState, EmpireBridge};
    // B-25: read the shared AppState bridge handle when present. Falls back
    // to a fresh disconnected EmpireBridge if the launcher did not opt
    // into --empire so the dashboard still renders a deterministic empty
    // state. The fallback path is the EMPIRE-PENDING activation: once the
    // bridge is always present + always wired, this branch goes away.
    let bridge_arc = state.empire_bridge_snapshot();
    let label = match bridge_arc
        .as_ref()
        .map(|b| b.state())
        .unwrap_or(BridgeState::Disconnected)
    {
        BridgeState::Disconnected => "disconnected",
        BridgeState::Connecting => "connecting",
        BridgeState::Connected => "connected",
    };
    let factory_line_count = bridge_arc
        .as_ref()
        .map(|b| b.factory_lines().len())
        .unwrap_or_else(|| EmpireBridge::new().factory_lines().len()) as u32;
    (label, true, factory_line_count, NOTE_DISCONNECTED)
}

#[cfg(not(feature = "empire"))]
fn empire_status_inner(_state: &AppState) -> (&'static str, bool, u32, &'static str) {
    ("unavailable", false, 0, NOTE_UNAVAILABLE)
}

#[cfg(feature = "empire")]
fn empire_factory_lines_inner(state: &AppState) -> Vec<FactoryLineDto> {
    use elai_orchestrator::empire::EmpireBridge;
    let lines = match state.empire_bridge_snapshot() {
        Some(b) => b.factory_lines(),
        None => EmpireBridge::new().factory_lines(),
    };
    lines
        .into_iter()
        .map(|fl| FactoryLineDto {
            brain_id: fl.brain_id,
            engine_tier: format!("{:?}", fl.engine_tier).to_uppercase(),
            neuron_count: fl.neuron_count,
            last_heartbeat_ms: fl.last_heartbeat_ms,
            status: fl.status,
        })
        .collect()
}

#[cfg(not(feature = "empire"))]
fn empire_factory_lines_inner(_state: &AppState) -> Vec<FactoryLineDto> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_shape_has_expected_fields() {
        // Construct a fresh AppState via the canonical constructor so the
        // feature-conditional helpers see the same shape they'd see at
        // runtime (the empire_bridge field defaults to None).
        let state = AppState::new();
        let (label, enabled, count, note) = empire_status_inner(&state);
        // Either feature-on or feature-off path must populate label + note.
        assert!(!label.is_empty());
        assert!(!note.is_empty());
        // When feature is OFF count is always 0; when feature is ON count is
        // 0 until the bridge is actually wired into AppState (EMPIRE-PENDING).
        assert_eq!(count, 0);
        // feature_enabled is a bool — both branches are valid.
        let _ = enabled;
    }

    #[test]
    fn factory_lines_empty_in_scaffolding_state() {
        let state = AppState::new();
        let lines = empire_factory_lines_inner(&state);
        assert!(lines.is_empty());
    }
}
