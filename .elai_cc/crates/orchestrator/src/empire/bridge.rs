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
//!   - `emit_action()` writes the SSB row when an `Arc<Ssb>` handle is
//!     installed (fire-and-forget via `tokio::spawn`), or returns
//!     `EmpireBridgeError::NotConnected` when no handle is installed
//!   - `factory_lines()` returns an empty list (graceful degradation)
//!
//! All transitions are tracing-logged so the dashboard / log analyzer can
//! distinguish "Empire unavailable" from "bridge crashed".

use std::sync::{Arc, Mutex};

use thiserror::Error;

use super::types::{EmpireEventKind, FactoryLine};
use crate::ssb::{Ssb, SsbPayload};

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
    /// Optional SSB writer handle. When `Some`, `emit_action` writes the
    /// matching `empire_*` SSB row through the channel API. When `None`,
    /// `emit_action` is a graceful no-op (returns `NotConnected`). Set via
    /// [`EmpireBridge::with_ssb`] before the bridge is shared.
    ssb: Mutex<Option<Arc<Ssb>>>,
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
            ssb: Mutex::new(None),
        }
    }

    /// Install an `Arc<Ssb>` writer handle so `emit_action` can write
    /// `empire_*` rows through the SSB channel. Chainable — returns
    /// `&self` so the caller can construct + install in one expression
    /// before wrapping in `Arc`.
    pub fn with_ssb(&self, ssb: Arc<Ssb>) {
        if let Ok(mut g) = self.ssb.lock() {
            *g = Some(ssb);
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

    /// Write one of the 5 SSB action_type rows
    /// (empire_connected / empire_disconnected / empire_event_received /
    /// empire_capability_invoked / empire_repair_proposed) via the
    /// orchestrator's `Arc<Ssb>::write_payload` channel API.
    ///
    /// When the Ssb handle is `None` (not installed), returns
    /// `EmpireBridgeError::NotConnected` — graceful degradation, no panic.
    /// When the handle is `Some`, maps the `EmpireEventKind` to the matching
    /// `SsbPayload` variant and fires the write through the channel using
    /// `tokio::runtime::Handle::try_current()` + `spawn` (fire-and-forget,
    /// same pattern as `auth_broker_emit::emit_auth_broker_event`). If no
    /// tokio runtime is active, the write is skipped with a debug log.
    ///
    /// NOTE: `connect()` itself is still EMPIRE-PENDING — this method only
    /// wires the SSB writer call shape. The transport boot bodies in
    /// `factory_line.rs` remain stubs gated on Empire deployment.
    pub fn emit_action(&self, kind: EmpireEventKind) -> Result<(), EmpireBridgeError> {
        let label = match &kind {
            EmpireEventKind::Connected { .. } => "empire_connected",
            EmpireEventKind::Disconnected { .. } => "empire_disconnected",
            EmpireEventKind::EventReceived { .. } => "empire_event_received",
            EmpireEventKind::CapabilityInvoked { .. } => "empire_capability_invoked",
            EmpireEventKind::RepairProposed { .. } => "empire_repair_proposed",
        };

        // Snapshot the Ssb handle under a short-lived lock (Rule 8: no lock
        // across .await). Clone the Arc, drop the guard, then spawn the write.
        let ssb = {
            let g = self.ssb.lock();
            match g {
                Ok(guard) => guard.clone(),
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        "EmpireBridge::emit_action — ssb mutex poisoned, reporting NotConnected"
                    );
                    return Err(EmpireBridgeError::NotConnected);
                }
            }
        };

        let ssb = match ssb {
            Some(handle) => handle,
            None => {
                tracing::debug!(
                    action_type = label,
                    "EmpireBridge::emit_action — no Ssb handle installed, graceful no-op"
                );
                return Err(EmpireBridgeError::NotConnected);
            }
        };

        let payload = match &kind {
            EmpireEventKind::Connected {
                brain_id,
                protocol_version,
            } => SsbPayload::EmpireConnected(crate::ssb::EmpireConnectedPayload {
                brain_id: brain_id.clone(),
                protocol_version: *protocol_version,
            }),
            EmpireEventKind::Disconnected { brain_id, reason } => {
                SsbPayload::EmpireDisconnected(crate::ssb::EmpireDisconnectedPayload {
                    brain_id: brain_id.clone(),
                    reason: reason.clone(),
                })
            }
            EmpireEventKind::EventReceived {
                source,
                event_type,
                payload,
            } => SsbPayload::EmpireEventReceived(crate::ssb::EmpireEventReceivedPayload {
                source: source.clone(),
                event_type: event_type.clone(),
                payload: payload.clone(),
            }),
            EmpireEventKind::CapabilityInvoked {
                capability,
                caller,
                allowed,
            } => SsbPayload::EmpireCapabilityInvoked(
                crate::ssb::EmpireCapabilityInvokedPayload {
                    capability: capability.clone(),
                    caller: caller.clone(),
                    allowed: *allowed,
                },
            ),
            EmpireEventKind::RepairProposed {
                issue,
                target_path,
                fix_kind,
            } => SsbPayload::EmpireRepairProposed(crate::ssb::EmpireRepairProposedPayload {
                issue: issue.clone(),
                target_path: target_path.clone(),
                fix_kind: fix_kind.clone(),
            }),
        };

        // Fire-and-forget the SSB write through the existing mpsc channel
        // API. The Ssb write_loop owns the single writer; this is NOT a
        // second writer. agent_id="empire_bridge" identifies the bridge as
        // the producer; superstep=0 because Empire events are not tied to a
        // GRC superstep. Same pattern as auth_broker_emit.
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    if let Err(e) =
                        ssb.write_payload("empire_bridge", 0, payload, Vec::new()).await
                    {
                        tracing::warn!(
                            action_type = label,
                            error = %e,
                            "EmpireBridge::emit_action — SSB write failed"
                        );
                    }
                });
                tracing::debug!(
                    action_type = label,
                    "EmpireBridge::emit_action — SSB write spawned"
                );
                Ok(())
            }
            Err(_) => {
                tracing::debug!(
                    action_type = label,
                    "EmpireBridge::emit_action — no tokio runtime, skipping SSB write"
                );
                Err(EmpireBridgeError::NotConnected)
            }
        }
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
            // All five must return NotConnected when no Ssb handle is installed.
            let r = b.emit_action(v);
            assert!(matches!(r, Err(EmpireBridgeError::NotConnected)));
        }
    }

    #[test]
    fn factory_lines_empty_when_disconnected() {
        let b = EmpireBridge::new();
        assert!(b.factory_lines().is_empty());
    }

    // B-25 GAP-3: emit_action with no Ssb handle returns NotConnected
    // (graceful degradation — no panic, no SSB write attempted).
    #[test]
    fn emit_action_no_ssb_handle_returns_not_connected() {
        let b = EmpireBridge::new();
        let kind = EmpireEventKind::Connected {
            brain_id: "elai_cc".to_string(),
            protocol_version: 1,
        };
        let r = b.emit_action(kind);
        assert!(matches!(r, Err(EmpireBridgeError::NotConnected)));
    }

    // B-25 GAP-3: emit_action with an Ssb handle installed spawns the
    // write through the tokio runtime. Uses a real Ssb instance with a
    // temp-dir DB so the write_loop accepts the row. The call should
    // return Ok because the payload matches the action_type per the
    // QUINTUPLE-UPDATE lockstep and a tokio runtime is active.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn emit_action_with_ssb_handle_spawns_write() {
        use crate::ssb::{Ssb, SsbConfig};
        use elai_core::clock::{ClockProvider, SeededClock};
        let tmp = tempfile::tempdir().expect("tempdir for SSB test DB");
        let cfg = SsbConfig {
            db_path: tmp.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "p-test".into(),
            session_id: "s-test".into(),
        };
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let ssb = Ssb::new(cfg, clock).expect("Ssb::new for test");
        let ssb: Arc<Ssb> = Arc::new(ssb);

        let b = EmpireBridge::new();
        b.with_ssb(ssb.clone());

        let kind = EmpireEventKind::Connected {
            brain_id: "elai_cc".to_string(),
            protocol_version: 1,
        };
        let r = b.emit_action(kind);
        assert!(r.is_ok(), "emit_action with Ssb handle should return Ok");
        // Give the spawned write task time to complete so the SSB row lands.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}
