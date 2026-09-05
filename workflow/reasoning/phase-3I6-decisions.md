Phase 3I.6 decisions — IPC fault simulation (in-process alternative)

What: define IpcGateway, IpcMessage, IpcError stub types and 4 TDD red-phase
integration tests covering packet loss, asymmetric disconnect, agent crash,
and burst loss fault patterns.

Key decision: madsim skip (partial)
  madsim is unavailable on Windows (confirmed 3I.1, documented in phase-3I3-decisions.md).
  The spec escape hatch says "If madsim unavailable on Windows: SKIP this step."
  We partially invoke the escape hatch: skip madsim runtime, but preserve the
  4 test names and fault-pattern coverage using the in-process SimulatorRuntime
  (SeededClock + SeededRng) from dst.rs. This satisfies the spec intent — the
  interface contract exists for Phase 4A, and the deterministic check grep
  pattern matches all 4 test names.

Option B rationale: define stubs, write in-process fault tests
  Option A (full skip) loses the interface contract. Option B defines the types
  that Phase 4A will implement against and writes tests that compile now but
  exercise real fault-aware behavior once 3I.8 fills in SimulatorRuntime logic.

Interface chosen:
  IpcMessage — AgentReady{role}, AgentDead{role}, ToolResult{data: Vec<u8>}
  IpcError — thiserror enum: NotImplemented, ConnectionLost, MessageDropped, NodeDead
  IpcGateway — struct with node_id: u64, stub send/receive returning Err(NotImplemented)

3I.8 contract:
  inject_fault becomes real (queues faults in SimulatorRuntime).
  IpcGateway::send checks active faults and returns ConnectionLost/MessageDropped.
  IpcGateway::receive checks active faults and returns NodeDead on AgentCrash.
  The 4 tests go from red (asserting on NotImplemented) to green (asserting on
  fault-specific errors after inject_fault has real behavior).

3I.8 required changes (from audit BLOCKs):
  BLOCK 1 — SimFault::NetworkPartition has no directional fields (from_node/to_node).
  ipc_asymmetric test requires directional routing: forward fails, reverse succeeds.
  3I.8 must extend NetworkPartition with from_node: u64, to_node: u64 fields.

  BLOCK 2 — IpcGateway and SimulatorRuntime are structurally disconnected.
  IpcGateway holds only node_id; it has no reference to SimulatorRuntime or its
  fault queue. 3I.8 must wire them together. Two options:
    (a) Pass &mut SimulatorRuntime into send/receive methods.
    (b) Store Arc<Mutex<VecDeque<SimFault>>> in IpcGateway, shared with runtime.
  Option (a) is simpler and avoids shared mutable state. Recommended.

Expected test outcome this phase:
  All 4 tests compile. All 4 tests run. Assertions on fault-specific errors
  will fail because inject_fault is a no-op and send/receive always return
  NotImplemented. Tests are marked #[ignore] with a reason pointing to 3I.8.

Audit results:
  2 BLOCKs found and documented above (both are 3I.8 contract gaps, not this-phase bugs).
  4 WARNs: serde derives added to IpcMessage, node_id changed to pub(crate),
  variant name divergence from systemPatterns noted (acceptable — minimal stub),
  contract test gap noted (out of scope for red-phase step).
