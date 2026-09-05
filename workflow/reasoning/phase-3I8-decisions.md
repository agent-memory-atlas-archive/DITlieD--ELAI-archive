Phase 3I.8: IPC fault injection wiring (green phase)

What changes:
- SimFault::NetworkPartition gains from_node:Option<u64> and to_node:Option<u64> for asymmetric partitions (None = broadcast).
- SimulatorRuntime gains fault_queue:VecDeque<(SimFault, Duration)> (expiry time) and sim_time:Duration.
- advance_time increments sim_time, retains only unexpired faults.
- inject_fault computes expiry = sim_time + fault_duration, pushes to queue.
- snapshot_state returns real tick derived from sim_time.
- check_invariants extended: asserts ssb_row_count and active_agents match between SUT and reference.
- IpcGateway::send/receive gain &mut SimulatorRuntime parameter (Option A from phase-3I6-decisions.md).
- send checks fault_queue for active NetworkPartition matching node_id direction.
- receive checks fault_queue for active AgentCrash.
- All 4 ipc_simulation.rs tests un-ignored, call sites updated.

Key decisions:
- Option A wiring (pass &mut sim) over Arc<Mutex> — simpler for in-process DST, no threading.
- fault_queue stores expiry_time (absolute) not remaining duration — avoids recomputation on each check.
- NetworkPartition duration extracted from enum variant to compute expiry at inject time.
- Broadcast partition (from_node=None): blocks all gateways regardless of node_id.
- Directional partition (from_node=Some(x), to_node=Some(y)): only blocks gateway with node_id==x sending to y.

Expected test outcome:
- All 4 IPC tests GREEN (previously ignored).
- Existing 2112 tests remain GREEN.
- 0 warnings.

Top failure modes:
1. Signature change in send/receive breaks callers outside ipc_simulation.rs — grep all call sites.
2. Expiry time arithmetic off-by-one: fault expires at exact sim_time boundary vs strictly-greater.
