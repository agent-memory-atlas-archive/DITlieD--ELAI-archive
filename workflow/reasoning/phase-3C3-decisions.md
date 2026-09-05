Phase 3C.3 Decisions — Checkpoint Module

What: CheckpointManager<C: ClockProvider> with 5-file atomic writes, SSB metadata recording,
ChannelKind::UntrackedValue validation, and read_latest for recovery.

Key decisions:
1. sha2 over blake3: blake3 not in orchestrator deps, sha2 already present. Hashes are for
   SSB metadata audit trail, not performance-critical. No new dep needed.
2. Serializable GRC snapshot via GrcHistoryEntry: GrcState enum lacks Serialize/Deserialize.
   Created GrcHistoryEntry with string fields instead of trying to modify the reasoning crate.
3. Vec<(String, u64)> for file_sizes: lighter than HashMap for ordered, small (5-element) list.
   Matches SSB payload serialization expectations.
4. tokio::fs for all file I/O: no std::fs in async context. Atomic write pattern uses
   write -> remove_existing -> rename (Windows NTFS safety).
5. InvalidTimestamp error variant: added to handle checkpoint_ts parse failures cleanly
   instead of forcing rmp_serde::decode::Error into a string parse context.
6. SsbTraceable impl on AgentCheckpointData: minimal impl with empty agent_id/step_index
   since the trait is for ad-hoc conversions; real writes go through CheckpointManager::write().

Expected test outcome: 17 checkpoint + 22 SSB + 20 agent_pool + 3 contract = 62 all green.
34 pre-existing failures in hooks/prompt (outside blast radius).

Unknowns:
- 3C.4 recovery will consume these files. Path convention locked: scratch_dir/{agent_id}/.
- No size limit on checkpoint data (spec defers to 3C.5 concern).
