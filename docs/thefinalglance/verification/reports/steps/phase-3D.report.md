# Verification Report: phase-3D.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-3D.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Phase 3D defines BSP superstep execution, 4 channel types, deferred node execution, checkpoint integration with memory monitoring, and true-parallel spawn_parallel. The core implementations are present and well-tested in orchestrator/src/bsp/. The primary gaps are: crash recovery is stub-only (logs only, no actual state restoration), there is no elai_cc.toml [orchestrator.bsp] config section, and the spec-required "all channel lifecycle behaviors correct across 5+ supersteps" integration test does not exist. The "checkpoint_roundtrip byte-identical" acceptance criterion exists for Topic round-trip but not for the full BSP checkpoint load/save cycle that spec 3D.4 demands.

## Deliverables verified DONE

### 3D.1 Channel Type Definitions
- LastValue<T> struct defined with SSB-seq last-writer-wins — evidence: .elai_cc/crates/orchestrator/src/bsp/channels.rs:203
- Topic<T>(accumulate: bool) struct with two-phase clear — evidence: channels.rs:211, clear_non_accumulating_topics() at channels.rs:470
- EphemeralValue<T> with write_count (u32, saturating_add) — evidence: channels.rs:217, EphemeralEntry at channels.rs:415
- UntrackedValue<T: DeepSizeOf> in-memory only — evidence: channels.rs:223
- ChannelId<T> phantom typed key — evidence: channels.rs:106
- ChannelState enum with exactly 3 variants (LastValue, Topic, Ephemeral, no UntrackedValue) — evidence: channels.rs:193
- ChannelRegistry trait with 6+ methods (read, write, snapshot_for_scheduling, clear_ephemeral, force_clear_untracked, deep_size_untracked) — evidence: channels.rs:324
- ChannelWrite, ChannelValue, ChannelError, ChannelSnapshot types — evidence: channels.rs:233, 181, 273, 247
- DeepSizeOf trait hand-rolled (no external dep) with Vec<T> heap measurement — evidence: channels.rs:34
- UNTRACKED_DEFAULT_LIMIT_BYTES = 10MB, UNTRACKED_FORCE_CLEAR_MULTIPLIER = 2 constants — evidence: channels.rs:22, 25
- Contract test: channel_registry_contract.rs covers all 6 registry methods, SSB-seq ordering, DeepSizeOf, ChannelState 3-variant — evidence: .elai_cc/crates/contracts/tests/channel_registry_contract.rs:1
- size_of_val absent from implementation code (only in comment) — evidence: channels.rs:32 (comment only)

### 3D.2 BSP Superstep Loop
- BspEngine::run() implementing the full corrected loop order — evidence: engine.rs:180
- snapshot_for_scheduling before clear_non_accumulating_topics before execute — evidence: engine.rs:204, 213
- clear_non_accumulating_topics() two-phase Topic reset — evidence: engine.rs:213
- execute_tasks_with_timeout via tokio::task::JoinSet — evidence: engine.rs:541
- evidence_gate_passes function — evidence: engine.rs:698
- save_checkpoint AFTER gate passes — evidence: engine.rs:427
- superstep counter u64, monotonically incrementing — evidence: engine.rs:186, 471
- MAX_SUPERSTEP_ATTEMPTS=3, MAX_SUPERSTEPS=100, DEFAULT_STRAGGLER_TIMEOUT_SECS=15 — evidence: engine.rs:34, 37, 40
- BspCheckpoint msgpack serialization with BTreeMap (deterministic order) — evidence: engine.rs:138
- Straggler classification as AgentDeath::TIMEOUT with SSB death event — evidence: engine.rs:225
- Checkpoint file: superstep_{N}.msgpack, atomic write via tmp+rename — evidence: engine.rs:770
- Tests: superstep_counter_increments, max_supersteps_terminates_loop, checkpoint_count_matches_supersteps, checkpoint_written_after_gate, straggler_timeout_infrastructure — evidence: engine.rs:1031, 1047, 1074, 1114, 1148
- Contract test: determinism_contract (byte-identical checkpoints same seed) — evidence: bsp_engine_contract.rs:124
- Contract test: bsp_engine_runs_3_supersteps_with_2_agents — evidence: bsp_engine_contract.rs:82

### 3D.3 Deferred Node Execution
- is_deferred: bool on NodeSpec — evidence: engine.rs:115
- Deferred phase between apply_writes and clear_ephemeral — evidence: engine.rs:288
- all_inputs_satisfied via depends_on.all(Complete) check — evidence: engine.rs:298
- Single-pass only (no cascading) — evidence: engine.rs:309 (separate JoinSet per deferred phase)
- Tests: test_deferred_fires_after_barrier, test_deferred_unsatisfied_does_not_fire, test_cascading_deferred_impossible — evidence: engine.rs:1240, 1296, 1345

### 3D.4 Checkpoint + Memory Management Integration
- Memory monitoring: deep_size_untracked per-channel at each superstep boundary — evidence: engine.rs:374
- SSB MemoryMonitoringPayload emitted (not isolation_metric) — evidence: engine.rs:388
- force-clear when size >= 2x threshold — evidence: engine.rs:382
- EphemeralValue write_count reset via reset_ephemeral_write_counts() — evidence: channels.rs:450, engine.rs:196
- clear_ephemeral() returns cleared names for SSB recording — evidence: channels.rs:387, engine.rs:405
- EphemeralValue clears recorded in SSB (CheckpointPayload.agent_state_hash = JSON channel names) — evidence: engine.rs:711
- Topic(accumulate=false) two-phase clear implemented — evidence: engine.rs:213, channels.rs:470
- UntrackedValue excluded from BspCheckpoint — evidence: engine.rs:138 (BTreeMap<String, ChannelState>, ChannelState has no Untracked variant)
- recover_from_checkpoint function defined (validates SSB + logs ephemeral-clear records) — evidence: engine.rs:806
- isolation_metric absent from bsp/ — evidence: no grep hits
- Tests: memory_monitoring_measure_only, memory_monitoring_force_clear, memory_monitoring_empty_untracked, memory_monitoring_multi_channel_mixed, clear_ephemeral_recorded — evidence: engine.rs:1411, 1463, 1525, 1560, 1167
- Contract test: memory_monitoring_force_clear_contract — evidence: bsp_engine_contract.rs:219

### 3D.5 Parallel Agent Execution (spawn_parallel upgrade)
- spawn_parallel uses tokio::task::JoinSet for concurrent dispatch — evidence: engine.rs:541
- VRAM-aware max_concurrent via floor(available_vram / per_agent_vram) — evidence: agent_pool.rs:622
- Semaphore limits JoinSet fan-out to max_concurrent — evidence: engine.rs:530
- Fallback to max_concurrent=1 when VRAM=0 or per_agent=0 — evidence: agent_pool.rs:626
- Results sorted by input index (SSB seq ordering) — evidence: engine.rs:618
- Tests: test_spawn_parallel_true_concurrency (wall-time <250ms for 2x100ms tasks), test_straggler_timeout_reachable, test_result_seq_order — evidence: engine.rs:1652, 1699, 1734
- VRAM tests in agent_pool.rs: max_concurrent_floor_division, max_concurrent_vram_zero_fallback, max_concurrent_per_agent_zero_guard — evidence: agent_pool.rs:1213, 1229, 1240

## Deliverables PARTIAL

- Crash recovery (3D.4 acceptance: "Crash recovery produces identical state to clean run")
  - exists: recover_from_checkpoint() at engine.rs:806
  - gap: Implementation is validation+logging only. Doc comment says "actual state restoration into channels is a future-phase concern." No test exists that simulates crash at superstep N, calls recover_from_checkpoint, and asserts channel state is identical to a clean run. The spec's crash recovery acceptance criterion is unmet.

- checkpoint_roundtrip byte-identical test (3D.4 acceptance)
  - exists: adversarial_topic_round_trip_identical at channel_registry_contract.rs:144 (Topic msgpack only)
  - gap: No test that saves a full BspCheckpoint (with all 3 channel types), loads it back via rmp_serde::from_slice, and asserts byte-identity. The spec requires "cargo test checkpoint_roundtrip PASS".

- "All channel lifecycle behaviors correct across 5+ supersteps" (3D.4 acceptance)
  - exists: individual per-feature tests cover 1-3 supersteps each
  - gap: No single integration test runs 5+ supersteps with mixed LastValue/Topic/Ephemeral/Untracked channels across all supersteps. This is the spec's capstone integration criterion.

## Deliverables MISSING

- elai_cc.toml [orchestrator.bsp.untracked_memory_limit_bytes] config — searched: "untracked_memory_limit", "[orchestrator", "bsp.*limit" in elai_cc.toml; no results. UNTRACKED_DEFAULT_LIMIT_BYTES is a hard-coded Rust constant (channels.rs:22), not toml-configurable as R19 specifies.

## Deliverables UNVERIFIABLE

- Spec amendments 1-4 applied to orchestrator.txt — the spec requires four amendments to orchestrator.txt before implementation. Cannot verify spec file modifications (workflow docs are outside codebase scope). Implementation matches the amended behavior (accumulate=false defined, ChannelRegistry trait defined, checkpoint after gate), so the code is consistent with the amendments having been made.

## What must be done (action list)

- [ ] Implement actual crash recovery in recover_from_checkpoint(): restore LastValue/Topic/Ephemeral channel state from BspCheckpoint into InMemoryChannelRegistry, replay EphemeralValue clearing decisions from SSB, and write a test that crashes at superstep N and asserts recovered state == clean-run state (engine.rs:806)
- [ ] Add checkpoint_roundtrip test: write BspCheckpoint with all 3 channel types, serialize/deserialize, assert byte-identical (engine.rs tests section or bsp_engine_contract.rs)
- [ ] Add 5+ superstep integration test covering all channel types through their lifecycle (LastValue last-writer-wins, Topic accumulate=true/false, Ephemeral cleared on zero-write, Untracked force-clear) across 5 consecutive supersteps
- [ ] Add [orchestrator.bsp] section to elai_cc.toml with untracked_memory_limit_bytes = 10485760 (10MB) so BspConfig reads from toml rather than hard-coded constant (elai_cc.toml, BspConfig in engine.rs:76)

## Notes
The core BSP machinery (all 4 channel types, superstep loop, deferred nodes, memory monitoring, VRAM-aware parallelism) is fully implemented and has strong test coverage. The gaps are in crash recovery completeness, a missing checkpoint roundtrip test, one missing multi-superstep integration test, and the toml config binding for the memory limit. These are acceptance-criterion gaps, not architectural gaps.
