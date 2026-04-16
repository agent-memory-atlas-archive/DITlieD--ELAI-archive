// IPC wire protocol types for agent process isolation (Phase 4A).
//
// All types are defined in the shared elai-ipc-types crate and re-exported
// here for backwards compatibility. The shared crate allows the agent binary
// to use the same types without depending on the orchestrator.
//
// Wire format: [1-byte version][4-byte LE length][postcard payload]
// All messages are serialized with postcard. The orchestrator and agent binary
// both depend on elai-ipc-types for type definitions.
//
// Source: phase-4A.txt L49-79, orchestrator.txt S12.

pub use elai_ipc_types::*;
