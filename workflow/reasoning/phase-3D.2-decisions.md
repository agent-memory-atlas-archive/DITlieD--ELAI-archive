Phase 3D.2: BSP Superstep Engine

What: BspEngine<P: AgentPool, C: ClockProvider> implementing the corrected BSP superstep loop.
New file: crates/orchestrator/src/bsp/engine.rs + bsp/mod.rs update + contract tests.

Key decisions:
1. BspEngine takes &mut InMemoryChannelRegistry concretely (not trait-generic) because
   clear_non_accumulating_topics() and reset_ephemeral_write_counts() are inherent methods
   on InMemoryChannelRegistry, not on the ChannelRegistry trait.
2. Superstep checkpoint uses tokio::fs + rmp_serde directly (not CheckpointManager which
   is for agent checkpoints with 5-file atomic writes). Format: superstep_{N}.msgpack.
3. Straggler timeout: tokio::time::timeout wrapping a future::ready() since spawn_parallel
   is synchronous in 3D.2. Infrastructure present for 3D.5 upgrade.
4. evidence_gate_passes: all non-deferred nodes must be Complete or TimedOut. Simple for 3D.2.
5. Ephemeral-clear recording uses existing CheckpointPayload with agent_id="bsp_engine".
6. No new dependencies needed. rmp-serde, tokio, thiserror, serde, uuid already in Cargo.toml.

Expected test outcomes:
- 9 unit tests in engine.rs #[cfg(test)]
- 3 contract tests in contracts/tests/bsp_engine_contract.rs
- cargo check --workspace: 0 warnings

Unknowns:
- Whether evidence_gate_passes needs richer logic for 3D.3 deferred nodes (answer: no, deferred
  nodes are filtered out in prepare_next_tasks; gate only checks non-deferred).
- Whether BspCheckpoint serialization order is stable across HashMap iteration (answer: yes,
  BTreeMap used for determinism instead of HashMap).
