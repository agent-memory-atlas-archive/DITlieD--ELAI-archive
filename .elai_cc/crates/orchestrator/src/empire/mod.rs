//! Phase 4J — Empire integration submodule (host-side scaffolding).
//!
//! Status: SCAFFOLDING ONLY. The Empire codebase (workspace/factory-brain)
//! is not currently accessible from this workspace. Per phase-4J pre-resolved
//! decision policy, this submodule ships the full host-side surface (types,
//! bridge state machine, factory-line client outline, brain manifest, Tauri
//! command stubs) so live activation is a one-line flip when Empire deploys.
//!
//! Every external-IO seam is annotated with the marker
//! `EMPIRE-PENDING: <what-to-wire>` so a grep-driven follow-up patch can
//! locate and flip every wire-point at once. Markers are NOT TODOs — each
//! seam returns a well-typed `EmpireBridgeError::NotConnected` or an empty
//! list so the bridge degrades gracefully when Empire is unreachable.
//!
//! Module layout:
//!   - `types`          — wire-format value objects (FactoryLine command /
//!                         response, FactoryCmd, BrainManifest, EngineTier,
//!                         FactoryLine line snapshot, capability,
//!                         EmpireEventKind for the 5 SSB variants).
//!   - `ipc`            — Empire IPC engine-protocol types (RegisterHook /
//!                         Emit / QueryHooks / Subscribe / Unsubscribe /
//!                         BusCreate / BusJoin / BusLeave / BusList).
//!                         Lives at `empire::ipc::Message` to avoid colliding
//!                         with the existing internal `elai_ipc_types::IpcMessageType`
//!                         (10+ call sites, separate concept entirely).
//!   - `factory_line`   — FactoryLine protocol client outline. The connect /
//!                         heartbeat / re-register / reconnect loop is wired
//!                         in shape; the actual `tokio::net::UnixStream` /
//!                         `TcpStream` boot is gated on Empire deployment.
//!   - `brain_manifest` — `BrainManifest::validate` rules (reject empty
//!                         brain_id / empty neurons / duplicate neuron_ids).
//!   - `bridge`         — `EmpireBridge` state machine
//!                         (Disconnected -> Connecting -> Connected ->
//!                         Disconnected) + `EmpireBridge::emit_action`
//!                         entrypoint that maps `EmpireEventKind` to one of
//!                         the 5 SSB action types when Empire is live.
//!
//! Feature gate: this module is only compiled when the `empire` feature is
//! active on the orchestrator crate (`crates/orchestrator/Cargo.toml`).
//! Non-empire builds pay zero compile-time or runtime cost.

pub mod brain_manifest;
pub mod bridge;
pub mod factory_line;
pub mod ipc;
pub mod repair;
pub mod types;

pub use brain_manifest::BrainManifest;
pub use bridge::{BridgeState, EmpireBridge, EmpireBridgeError};
pub use factory_line::{
    build_transport, FactoryLineClient, FactoryLineConfig, FactoryLineTransport, TcpTransport,
    TransportConfig,
};
#[cfg(unix)]
pub use factory_line::UnixTransport;
pub use ipc::EmpireIpcMessageType;
pub use repair::{detect_repair_kind, EmpireRepairError, EmpireRepairKind, EmpireRepairProposal};
pub use types::{
    Capability, ElaiNeuron, EmpireEvent, EmpireEventKind, EngineTier, FactoryCmd, FactoryLine,
    FactoryLineCommand, FactoryLineResponse, HookBinding, HookKind, NeuronRegistration,
};
