//! `EmpireBridge` — state machine + SSB action-kind dispatcher.
//!
//! The bridge owns the connection lifecycle (Disconnected -> Connecting ->
//! Connected -> Disconnected) and exposes a single `emit_action` entrypoint
//! that maps an `EmpireEventKind` to the matching SSB action_type when
//! Empire is live (EMPIRE-PENDING on the SSB dispatcher).
//!
//! Until Empire deploys:
//!   - `connect()` returns `EmpireBridgeError::NotConnected`
//!   - the state machine stays at `Disconnected`
//!   - `emit_action()` returns `EmpireBridgeError::NotConnected`
//!   - `factory_lines()` returns an empty list (graceful degradation)
//!
//! All transitions are tracing-logged so the dashboard / log analyzer can
//! distinguish "Empire unavailable" from "bridge crashed".

use std::sync::Mutex;

use thiserror::Error;

use super::types::{EmpireEventKind, FactoryLine};

/// State machine for the bridge connection lifecycle.
///
/// Transitions: Disconnected -> Connecting -> (Connected | Disconnected).
/// Idempotent — calling `connect()` while already `Connected` is a no-op.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeState {
    Disconnected,
    Connecting,
    Connected,
}

impl BridgeState {
    pub fn as_str(&self) -> &'static str {
        match self {
            BridgeState::Disconnected => "disconnected",
            BridgeState::Connecting => "connecting",
            BridgeState::Connected => "connected",
        }
    }
}

/// Failure modes the bridge surfaces. All variants are non-panicking —
/// the bridge never aborts the orchestrator.
#[derive(Debug, Clone, Error)]
pub enum EmpireBridgeError {
    #[error("Empire bridge is not connected (host-side scaffold; pending Empire deployment)")]
    NotConnected,
    #[error("Empire bridge: encode failed: {0}")]
    Encode(String),
    #[error("Empire bridge: decode failed: {0}")]
    Decode(String),
    #[error("Empire bridge: I/O error: {0}")]
    Io(String),
    #[error("Empire bridge: protocol violation: {0}")]
    Protocol(String),
}

/// Host-side bridge between elai_cc and the Empire factory-brain bus.
pub struct EmpireBridge {
    state: Mutex<BridgeState>,
}

impl Default for EmpireBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl EmpireBridge {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(BridgeState::Disconnected),
        }
    }

    /// Current bridge state. Used by `get_empire_status` Tauri command.
    pub fn state(&self) -> BridgeState {
        match self.state.lock() {
            Ok(g) => *g,
            // A poisoned mutex shouldn't happen in practice (no panic-while-held
            // paths inside the bridge), but if it does we want a deterministic
            // observable: report Disconnected and log loudly.
            Err(e) => {
                tracing::error!(error = %e, "EmpireBridge::state — mutex poisoned, reporting Disconnected");
                BridgeState::Disconnected
            }
        }
    }

    fn transition(&self, next: BridgeState) {
        if let Ok(mut g) = self.state.lock() {
            let prev = *g;
            if prev == next {
                return; // idempotent
            }
            *g = next;
            tracing::info!(
                from = prev.as_str(),
                to = next.as_str(),
                "EmpireBridge state transition"
            );
        }
    }

    /// EMPIRE-PENDING: drive `FactoryLineClient::connect()` and on success
    /// transition to `Connected`. Until Empire deploys this transitions to
    /// `Connecting` then immediately back to `Disconnected` so log
    /// analyzers see the attempt + failure pair.
    pub async fn connect(&self) -> Result<(), EmpireBridgeError> {
        self.transition(BridgeState::Connecting);
        tracing::debug!(
            "EmpireBridge::connect — EMPIRE-PENDING: factory-brain socket boot deferred until Empire deploys"
        );
        self.transition(BridgeState::Disconnected);
        Err(EmpireBridgeError::NotConnected)
    }

    /// EMPIRE-PENDING: write one of the 5 SSB action_type rows
    /// (empire_connected / empire_disconnected / empire_event_received /
    /// empire_capability_invoked / empire_repair_proposed) via the
    /// orchestrator's `Arc<Ssb>::write_payload`. Returns `NotConnected`
    /// until Empire deploys + the SSB lockstep patch lands.
    pub fn emit_action(&self, kind: EmpireEventKind) -> Result<(), EmpireBridgeError> {
        let label = match &kind {
            EmpireEventKind::Connected { .. } => "empire_connected",
            EmpireEventKind::Disconnected { .. } => "empire_disconnected",
            EmpireEventKind::EventReceived { .. } => "empire_event_received",
            EmpireEventKind::CapabilityInvoked { .. } => "empire_capability_invoked",
            EmpireEventKind::RepairProposed { .. } => "empire_repair_proposed",
        };
        tracing::debug!(
            action_type = label,
            "EmpireBridge::emit_action — EMPIRE-PENDING: SSB lockstep (SCHEMA_VERSION bump) deferred until Empire deploys"
        );
        Err(EmpireBridgeError::NotConnected)
    }

    /// EMPIRE-PENDING: list FactoryLine rows the dashboard can render. When
    /// the bridge is `Disconnected`, returns an empty list so the panel
    /// shows the empty-state message (graceful degradation).
    pub fn factory_lines(&self) -> Vec<FactoryLine> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::empire::types::EmpireEventKind;

    #[test]
    fn new_bridge_starts_disconnected() {
        let b = EmpireBridge::new();
        assert_eq!(b.state(), BridgeState::Disconnected);
    }

    #[tokio::test]
    async fn connect_round_trips_through_connecting_and_back() {
        let b = EmpireBridge::new();
        let res = b.connect().await;
        assert!(matches!(res, Err(EmpireBridgeError::NotConnected)));
        // After connect() returns Err, the bridge should be back in Disconnected.
        assert_eq!(b.state(), BridgeState::Disconnected);
    }

    #[test]
    fn emit_action_labels_all_five_variants() {
        let b = EmpireBridge::new();
        let variants = [
            EmpireEventKind::Connected {
                brain_id: "elai_cc".to_string(),
                protocol_version: 1,
            },
            EmpireEventKind::Disconnected {
                brain_id: "elai_cc".to_string(),
                reason: "test".to_string(),
            },
            EmpireEventKind::EventReceived {
                source: "factory".to_string(),
                event_type: "deploy".to_string(),
                payload: serde_json::Value::Null,
            },
            EmpireEventKind::CapabilityInvoked {
                capability: "task_submit".to_string(),
                caller: "factory".to_string(),
                allowed: true,
            },
            EmpireEventKind::RepairProposed {
                issue: "CircularDep".to_string(),
                target_path: "brainctl.py".to_string(),
                fix_kind: "Refactor".to_string(),
            },
        ];
        for v in variants {
            // All five must return NotConnected until Empire deploys.
            let r = b.emit_action(v);
            assert!(matches!(r, Err(EmpireBridgeError::NotConnected)));
        }
    }

    #[test]
    fn factory_lines_empty_when_disconnected() {
        let b = EmpireBridge::new();
        assert!(b.factory_lines().is_empty());
    }
}
