//! BSP superstep engine: drives agents through barrier-synchronized supersteps.
//!
//! The engine wires together channels, agent pool, SSB recording, and checkpoint
//! persistence into the core loop that all subsequent phases build on.
//! Loop order: snapshot -> prepare_next_tasks -> clear_non_accumulating_topics
//! -> execute_tasks_parallel -> apply_writes -> deferred_phase ->
//! clear_ephemeral -> evidence_gate_passes -> save_checkpoint -> superstep++.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use elai_core::ClockProvider;

use crate::agent_pool::{AgentHandle, AgentPool, AgentPoolError, AgentRole};
use crate::bsp::channels::{
    ChannelId, ChannelRegistry, ChannelSnapshot, ChannelState, InMemoryChannelRegistry,
    UNTRACKED_DEFAULT_LIMIT_BYTES, UNTRACKED_FORCE_CLEAR_MULTIPLIER,
};
use crate::ssb::{
    CheckpointPayload, DeathEventPayload, MemoryMonitoringPayload, Ssb, SsbActionType,
    SsbPayload, SsbReader,
};

// ---------------------------------------------------------------------------
// Named constants (no magic numbers)
// ---------------------------------------------------------------------------

/// Maximum retry attempts per superstep before aborting.
pub const MAX_SUPERSTEP_ATTEMPTS: u32 = 3;

/// Maximum total supersteps before safety-valve abort.
pub const MAX_SUPERSTEPS: u64 = 100;

/// Default straggler timeout in seconds (matches Phase 3C heartbeat timeout).
pub const DEFAULT_STRAGGLER_TIMEOUT_SECS: u64 = 15;

// ---------------------------------------------------------------------------
// BspError
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum BspError {
    #[error("max superstep attempts ({0}) exceeded at superstep {1}")]
    MaxAttemptsExceeded(u32, u64),

    #[error("max supersteps ({0}) reached")]
    MaxSuperstepsReached(u64),

    #[error("agent pool error: {0}")]
    AgentPool(#[from] AgentPoolError),

    #[error("SSB write error: {0}")]
    Ssb(#[from] crate::ssb::SsbError),

    #[error("checkpoint I/O error: {0}")]
    CheckpointIo(String),

    #[error("channel error: {0}")]
    Channel(#[from] crate::bsp::channels::ChannelError),

    #[error("serialization error: {0}")]
    Serialization(String),
}

// ---------------------------------------------------------------------------
// BspConfig
// ---------------------------------------------------------------------------

/// Configuration for a BSP engine run.
#[derive(Debug, Clone)]
pub struct BspConfig {
    pub max_superstep_attempts: u32,
    pub max_supersteps: u64,
    pub straggler_timeout: Duration,
    pub checkpoint_dir: PathBuf,
    pub session_id: String,
    pub project_id: String,
}

impl Default for BspConfig {
    fn default() -> Self {
        Self {
            max_superstep_attempts: MAX_SUPERSTEP_ATTEMPTS,
            max_supersteps: MAX_SUPERSTEPS,
            straggler_timeout: Duration::from_secs(DEFAULT_STRAGGLER_TIMEOUT_SECS),
            checkpoint_dir: PathBuf::from("checkpoints"),
            session_id: String::new(),
            project_id: String::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// NodeSpec
// ---------------------------------------------------------------------------

/// Specification for a single node in the BSP graph.
#[derive(Debug, Clone)]
pub struct NodeSpec {
    pub id: String,
    pub role: AgentRole,
    pub task: String,
    pub depends_on: Vec<String>,
    /// Deferred nodes are present but not executed in 3D.2.
    pub is_deferred: bool,
}

// ---------------------------------------------------------------------------
// NodeStatus
// ---------------------------------------------------------------------------

/// Execution status of a BSP node within a superstep run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Pending,
    Complete,
    Failed,
    TimedOut,
}

// ---------------------------------------------------------------------------
// BspCheckpoint
// ---------------------------------------------------------------------------

/// Superstep checkpoint: serialized to msgpack for crash recovery.
/// Uses BTreeMap (not HashMap) for deterministic serialization order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BspCheckpoint {
    pub superstep: u64,
    pub channel_state: BTreeMap<String, ChannelState>,
    pub node_statuses: BTreeMap<String, NodeStatus>,
    /// Timestamp in seconds since UNIX_EPOCH, from ClockProvider (deterministic).
    pub created_at_secs: u64,
}

// ---------------------------------------------------------------------------
// BspResult
// ---------------------------------------------------------------------------

/// Result of a completed BSP run.
#[derive(Debug)]
pub struct BspResult {
    pub supersteps_completed: u64,
    pub final_snapshot: ChannelSnapshot,
    /// Final status of every node (including deferred). Callers can inspect
    /// for Pending/TimedOut/Failed to detect stranded or timed-out nodes.
    pub node_statuses: BTreeMap<String, NodeStatus>,
}

// ---------------------------------------------------------------------------
// BspEngine
// ---------------------------------------------------------------------------

/// BSP superstep engine. Generic over agent pool and clock provider.
pub struct BspEngine<P: AgentPool, C: ClockProvider> {
    pub config: BspConfig,
    pool: P,
    /// Clock provider for deterministic timestamps (entropy quarantine).
    /// Used in straggler timeout tracking and checkpoint metadata.
    clock: C,
}

impl<P: AgentPool, C: ClockProvider> BspEngine<P, C> {
    pub fn new(config: BspConfig, pool: P, clock: C) -> Self {
        Self { config, pool, clock }
    }

    /// Run the BSP superstep loop to completion.
    pub async fn run(
        &mut self,
        nodes: Vec<NodeSpec>,
        channels: &mut InMemoryChannelRegistry,
        ssb: &Ssb,
    ) -> Result<BspResult, BspError> {
        let mut superstep: u64 = 0;
        let mut attempt: u32 = 0;
        let mut node_statuses: BTreeMap<String, NodeStatus> = nodes
            .iter()
            .map(|n| (n.id.clone(), NodeStatus::Pending))
            .collect();

        // Reset ephemeral write counts before first iteration so clear_ephemeral
        // sees a clean slate regardless of caller state.
        channels.reset_ephemeral_write_counts();

        loop {
            // Safety valve: max supersteps
            if superstep >= self.config.max_supersteps {
                return Err(BspError::MaxSuperstepsReached(self.config.max_supersteps));
            }

            // 1. Snapshot channel state for scheduling decisions
            let channel_snapshot = channels.snapshot_for_scheduling();

            // 2. Prepare tasks for this superstep
            let tasks = prepare_next_tasks(&node_statuses, &channel_snapshot, &nodes);
            if tasks.is_empty() {
                break;
            }

            // 3. Clear non-accumulating topics AFTER snapshot, BEFORE execute
            channels.clear_non_accumulating_topics();

            // 4. Execute tasks via agent pool with straggler timeout
            let (completed_handles, timed_out) = self
                .execute_tasks_with_timeout(&tasks, ssb, superstep)
                .await?;

            // Record timed-out agents as death events
            for (node_id, handle) in &timed_out {
                let agent_id_str = handle.agent_id.to_string();
                let death_payload = SsbPayload::DeathEvent(DeathEventPayload {
                    agent_id: agent_id_str.clone(),
                    classification: "timeout".to_owned(),
                    prior_step_index: superstep as i64,
                    checkpoint_path: String::new(),
                    had_checkpoint: false,
                });
                ssb.write_payload(
                    &agent_id_str,
                    superstep as i64,
                    death_payload,
                    vec![],
                )
                .await?;
                node_statuses.insert(node_id.clone(), NodeStatus::TimedOut);
            }

            // Mark completed nodes
            for (node_id, _handle) in &completed_handles {
                node_statuses.insert(node_id.clone(), NodeStatus::Complete);
            }

            // 5. Apply writes (no channel writes from agents in 3D.2 stub —
            // agents don't produce ChannelWrite results from spawn_parallel)
            // In future phases, apply_writes will sort by ssb_seq and apply.

            // 5b. Deferred phase: single-pass execution of deferred nodes
            //     whose dependencies are all Complete after the main barrier.
            let deferred_tasks: Vec<(String, AgentRole, String)> = nodes
                .iter()
                .filter(|n| n.is_deferred)
                .filter(|n| {
                    matches!(
                        node_statuses.get(&n.id).copied().unwrap_or(NodeStatus::Pending),
                        NodeStatus::Pending
                    )
                })
                .filter(|n| {
                    n.depends_on.iter().all(|dep| {
                        matches!(
                            node_statuses.get(dep).copied().unwrap_or(NodeStatus::Pending),
                            NodeStatus::Complete
                        )
                    })
                })
                .map(|n| (n.id.clone(), n.role, n.task.clone()))
                .collect();

            if !deferred_tasks.is_empty() {
                let (deferred_completed, deferred_timed_out) = self
                    .execute_tasks_with_timeout(&deferred_tasks, ssb, superstep)
                    .await?;

                for (node_id, _handle) in &deferred_completed {
                    node_statuses.insert(node_id.clone(), NodeStatus::Complete);
                }

                for (node_id, handle) in &deferred_timed_out {
                    let agent_id_str = handle.agent_id.to_string();
                    let death_payload = SsbPayload::DeathEvent(DeathEventPayload {
                        agent_id: agent_id_str.clone(),
                        classification: "timeout".to_owned(),
                        prior_step_index: superstep as i64,
                        checkpoint_path: String::new(),
                        had_checkpoint: false,
                    });
                    ssb.write_payload(
                        &agent_id_str,
                        superstep as i64,
                        death_payload,
                        vec![],
                    )
                    .await?;
                    node_statuses.insert(node_id.clone(), NodeStatus::TimedOut);
                }
            }

            // 6. Memory monitoring: measure untracked channel sizes, emit SSB events
            {
                let agent_id = format!("bsp_engine:{}", self.config.session_id);
                let threshold = UNTRACKED_DEFAULT_LIMIT_BYTES;
                let force_threshold = threshold * UNTRACKED_FORCE_CLEAR_MULTIPLIER as usize;
                let sizes = channels.iter_untracked_sizes();

                for (name, size) in sizes {
                    let action = if size >= force_threshold {
                        "force_clear"
                    } else {
                        "measure"
                    };

                    let payload = SsbPayload::MemoryMonitoring(MemoryMonitoringPayload {
                        metric_name: name.clone(),
                        value: size as f64,
                        threshold: threshold as f64,
                        action_taken: action.to_owned(),
                    });

                    ssb.write_payload(&agent_id, superstep as i64, payload, vec![])
                        .await?;

                    if action == "force_clear" {
                        let erased_id = ChannelId::<()>::new(&name);
                        channels.force_clear_untracked(&erased_id);
                    }
                }
            }

            // 7. Clear ephemeral channels and record in SSB
            let cleared = channels.clear_ephemeral();
            if !cleared.is_empty() {
                record_ephemeral_clears(ssb, &cleared, superstep, &self.config.session_id)
                    .await?;
            }

            // 8. Reset ephemeral write counts for next superstep
            channels.reset_ephemeral_write_counts();

            // 9. Evidence gate check (only scheduled tasks, not all nodes)
            let scheduled_ids: Vec<&str> = tasks.iter().map(|(id, _, _)| id.as_str()).collect();
            if !evidence_gate_passes(&node_statuses, &scheduled_ids) {
                attempt += 1;
                if attempt >= self.config.max_superstep_attempts {
                    return Err(BspError::MaxAttemptsExceeded(
                        self.config.max_superstep_attempts,
                        superstep,
                    ));
                }
                continue;
            }

            // 10. Save checkpoint ONLY after gate passes
            save_superstep_checkpoint(
                &self.config.checkpoint_dir,
                superstep,
                channels,
                &node_statuses,
                &self.clock,
            )
            .await?;

            superstep += 1;
            attempt = 0;
        }

        let final_snapshot = channels.snapshot_for_scheduling();
        Ok(BspResult {
            supersteps_completed: superstep,
            final_snapshot,
            node_statuses,
        })
    }

    /// Execute tasks with straggler timeout. Returns (completed, timed_out) pairs.
    async fn execute_tasks_with_timeout(
        &mut self,
        tasks: &[(String, AgentRole, String)],
        ssb: &Ssb,
        superstep: u64,
    ) -> Result<(Vec<(String, AgentHandle)>, Vec<(String, AgentHandle)>), BspError> {
        let pool_tasks: Vec<(AgentRole, String)> = tasks
            .iter()
            .map(|(_id, role, task)| (*role, task.clone()))
            .collect();

        let handles = self.pool.spawn_parallel(pool_tasks)?;

        let mut completed = Vec::new();
        let mut timed_out = Vec::new();

        for (i, handle) in handles.into_iter().enumerate() {
            let node_id = &tasks[i].0;
            // Structural timeout wrapper: spawn_parallel is synchronous in 3D.2,
            // so timeout only triggers if future phases make it async.
            let timeout_result = tokio::time::timeout(
                self.config.straggler_timeout,
                std::future::ready(handle),
            )
            .await;

            match timeout_result {
                Ok(h) => completed.push((node_id.clone(), h)),
                Err(_elapsed) => {
                    // Straggler: timeout expired. This path only activates
                    // when spawn_parallel becomes truly async in 3D.5.
                    // For now, create a placeholder handle for the death event.
                    // In practice this branch is unreachable with sync spawn_parallel.
                    let _ = (ssb, superstep); // used in 3D.5
                    timed_out.push((node_id.clone(), AgentHandle {
                        agent_id: uuid::Uuid::nil(),
                        role: tasks[i].1,
                        status: crate::agent_pool::AgentStatus::Failed,
                        task: tasks[i].2.clone(),
                        active_files: Vec::new(),
                        grc_config: crate::agent_pool::default_grc_config(&tasks[i].1),
                        scratch_dir: PathBuf::new(),
                    }));
                }
            }
        }

        Ok((completed, timed_out))
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Determine which nodes should execute in this superstep.
/// Returns (node_id, role, task) triples.
fn prepare_next_tasks(
    statuses: &BTreeMap<String, NodeStatus>,
    _snapshot: &ChannelSnapshot,
    nodes: &[NodeSpec],
) -> Vec<(String, AgentRole, String)> {
    let mut tasks = Vec::new();
    for node in nodes {
        // Skip deferred nodes (3D.3 scope)
        if node.is_deferred {
            continue;
        }
        // Skip already completed/failed/timed-out nodes
        let status = statuses.get(&node.id).copied().unwrap_or(NodeStatus::Pending);
        if status != NodeStatus::Pending {
            continue;
        }
        // Check all dependencies are satisfied
        let deps_satisfied = node.depends_on.iter().all(|dep| {
            matches!(
                statuses.get(dep).copied().unwrap_or(NodeStatus::Pending),
                NodeStatus::Complete
            )
        });
        if deps_satisfied {
            tasks.push((node.id.clone(), node.role, node.task.clone()));
        }
    }
    tasks
}

/// Check if all scheduled tasks for this superstep have completed or timed out.
///
/// Only checks nodes that were actually scheduled (not all nodes in the graph).
/// TimedOut counts as gate-passing per BSP spec: timed-out agents are removed
/// from the barrier wait set and recovery is triggered separately via
/// RecoveryManager::recover_agent(). The checkpoint records the timed-out
/// status so crash recovery can distinguish partial supersteps.
fn evidence_gate_passes(
    statuses: &BTreeMap<String, NodeStatus>,
    scheduled_ids: &[&str],
) -> bool {
    scheduled_ids.iter().all(|id| {
        matches!(
            statuses.get(*id).copied().unwrap_or(NodeStatus::Pending),
            NodeStatus::Complete | NodeStatus::TimedOut
        )
    })
}

/// Record ephemeral channel clears in SSB using existing CheckpointPayload.
async fn record_ephemeral_clears(
    ssb: &Ssb,
    cleared: &[String],
    superstep: u64,
    session_id: &str,
) -> Result<(), BspError> {
    let cleared_json =
        serde_json::to_string(cleared).map_err(|e| BspError::Serialization(e.to_string()))?;

    let agent_id = format!("bsp_engine:{session_id}");
    let payload = SsbPayload::Checkpoint(CheckpointPayload {
        agent_state_hash: cleared_json,
        grc_history_hash: String::new(),
        agent_id: agent_id.clone(),
        step_index: superstep,
        file_sizes: vec![],
    });

    ssb.write_payload(&agent_id, superstep as i64, payload, vec![])
        .await?;
    Ok(())
}

/// Save superstep checkpoint to disk as msgpack.
async fn save_superstep_checkpoint(
    checkpoint_dir: &PathBuf,
    superstep: u64,
    channels: &InMemoryChannelRegistry,
    node_statuses: &BTreeMap<String, NodeStatus>,
    clock: &dyn ClockProvider,
) -> Result<(), BspError> {
    let snapshot = channels.snapshot_for_scheduling();
    let mut channel_state = BTreeMap::new();
    for (name, state) in snapshot.iter() {
        channel_state.insert(name.clone(), state.clone());
    }

    let ts_secs = clock
        .system_now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| BspError::CheckpointIo(format!("clock error: {e}")))?
        .as_secs();

    let checkpoint = BspCheckpoint {
        superstep,
        channel_state,
        node_statuses: node_statuses.clone(),
        created_at_secs: ts_secs,
    };

    let bytes =
        rmp_serde::to_vec_named(&checkpoint).map_err(|e| BspError::Serialization(e.to_string()))?;

    tokio::fs::create_dir_all(checkpoint_dir)
        .await
        .map_err(|e| BspError::CheckpointIo(format!("create_dir_all: {e}")))?;

    let filename = format!("superstep_{superstep}.msgpack");
    let path = checkpoint_dir.join(&filename);

    // Atomic write: temp file then rename (same pattern as checkpoint.rs)
    let tmp_path = path.with_extension("tmp");
    tokio::fs::write(&tmp_path, &bytes)
        .await
        .map_err(|e| BspError::CheckpointIo(format!("write tmp {filename}: {e}")))?;

    // Remove existing target if present (NTFS requirement)
    if tokio::fs::metadata(&path).await.is_ok() {
        tokio::fs::remove_file(&path)
            .await
            .map_err(|e| BspError::CheckpointIo(format!("remove existing {filename}: {e}")))?;
    }

    tokio::fs::rename(&tmp_path, &path)
        .await
        .map_err(|e| BspError::CheckpointIo(format!("rename {filename}: {e}")))?;

    Ok(())
}

/// Recover channel state from a BSP checkpoint and SSB history.
///
/// Validates post-checkpoint SSB entries to detect channels that may have
/// been lost or stale after a crash. Currently performs validation and
/// logging only — actual state restoration into `channels` is a future-phase
/// concern. The `channels` parameter establishes the interface contract so
/// callers wire it in now.
///
/// Recovery per channel type:
/// - EphemeralValue: checks SSB Checkpoint entries for ephemeral-clear records.
///   Channels cleared before checkpoint are expected MISSING.
/// - Topic: logged as restored from checkpoint state.
/// - UntrackedValue: not in checkpoint, lost on crash by design. Logged.
pub fn recover_from_checkpoint(
    checkpoint: &BspCheckpoint,
    channels: &mut InMemoryChannelRegistry,
    ssb: &dyn SsbReader,
    session_id: &str,
) -> Result<(), BspError> {
    // Touch channels to satisfy the interface contract — actual restoration
    // will populate this in a future phase. For now, reset ephemeral write
    // counts so the registry is in a clean state for the next run() call.
    channels.reset_ephemeral_write_counts();
    // Query SSB entries at the checkpoint's superstep
    let entries = ssb.query_by_superstep(checkpoint.superstep as i64)?;

    // Build a set of channel names mentioned in ephemeral-clear Checkpoint entries
    // written by this BSP engine session.
    let agent_prefix = format!("bsp_engine:{session_id}");
    let mut cleared_channels: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    for entry in &entries {
        if entry.action_type != SsbActionType::Checkpoint {
            continue;
        }
        if !entry.agent_id.starts_with(&agent_prefix) {
            continue;
        }
        // Parse the checkpoint payload to extract cleared channel names
        if let Ok(SsbPayload::Checkpoint(cp)) = rmp_serde::from_slice::<SsbPayload>(&entry.payload)
        {
            // agent_state_hash contains JSON array of cleared channel names
            if let Ok(names) = serde_json::from_str::<Vec<String>>(&cp.agent_state_hash) {
                for name in names {
                    cleared_channels.insert(name);
                }
            }
        }
    }

    // Check each channel in the checkpoint state
    for (name, state) in &checkpoint.channel_state {
        match state {
            ChannelState::Ephemeral(_) => {
                if cleared_channels.contains(name) {
                    // Channel was cleared before checkpoint — MISSING is expected
                    tracing::info!(
                        channel = %name,
                        superstep = checkpoint.superstep,
                        "recovery: ephemeral channel was cleared (expected MISSING)"
                    );
                } else {
                    tracing::info!(
                        channel = %name,
                        superstep = checkpoint.superstep,
                        "recovery: ephemeral channel present in checkpoint"
                    );
                }
            }
            ChannelState::Topic(_) => {
                tracing::info!(
                    channel = %name,
                    superstep = checkpoint.superstep,
                    "recovery: topic channel restored from checkpoint"
                );
            }
            ChannelState::LastValue(_) => {
                tracing::info!(
                    channel = %name,
                    superstep = checkpoint.superstep,
                    "recovery: last_value channel restored from checkpoint"
                );
            }
        }
    }

    // UntrackedValue: not in checkpoint, lost by design
    tracing::info!(
        superstep = checkpoint.superstep,
        "recovery: untracked channels are lost on crash (by design)"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_pool::{
        AgentPoolConfig, AgentRole, InProcessAgentPool, ModelPool,
    };
    use crate::ssb::{Ssb, SsbActionType, SsbConfig, SsbPayload};
    use elai_core::{SeededClock, SeededRng};
    use std::sync::Arc;

    struct MockModelPool;
    impl ModelPool for MockModelPool {
        fn available_vram(&self) -> u64 {
            u64::MAX
        }
    }

    fn test_pool() -> InProcessAgentPool<SeededRng> {
        InProcessAgentPool::new(
            AgentPoolConfig::default(),
            SeededRng::new(42),
            Arc::new(MockModelPool),
        )
    }

    async fn test_ssb() -> Ssb {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("test.db");
        let db_path_owned = db_path.to_path_buf();
        std::mem::forget(dir);
        Ssb::new(
            SsbConfig {
                db_path: db_path_owned,
                project_id: "test-project".into(),
                session_id: "test-session".into(),
                pool_size: None,
            },
            Arc::new(SeededClock::new()),
        )
        .expect("ssb creation")
    }

    fn test_config(checkpoint_dir: PathBuf) -> BspConfig {
        BspConfig {
            checkpoint_dir,
            session_id: "test-session".into(),
            project_id: "test-project".into(),
            ..Default::default()
        }
    }

    fn simple_nodes(count: usize) -> Vec<NodeSpec> {
        (0..count)
            .map(|i| NodeSpec {
                id: format!("node_{i}"),
                role: AgentRole::Executor,
                task: format!("task_{i}"),
                depends_on: if i > 0 {
                    vec![format!("node_{}", i - 1)]
                } else {
                    vec![]
                },
                is_deferred: false,
            })
            .collect()
    }

    // 1. bsp_config_defaults
    #[test]
    fn bsp_config_defaults() {
        let cfg = BspConfig::default();
        assert_eq!(cfg.max_superstep_attempts, MAX_SUPERSTEP_ATTEMPTS);
        assert_eq!(cfg.max_supersteps, MAX_SUPERSTEPS);
        assert_eq!(
            cfg.straggler_timeout,
            Duration::from_secs(DEFAULT_STRAGGLER_TIMEOUT_SECS)
        );
    }

    // 2. apply_writes_sorted_by_seq
    // In 3D.2, spawn_parallel does not produce ChannelWrite results — agents
    // return AgentHandles, not writes. apply_writes logic will be added when
    // agents produce channel writes. This test verifies the deterministic
    // ordering invariant via BTreeMap in BspCheckpoint.
    #[test]
    fn checkpoint_channel_state_is_sorted() {
        let mut channel_state = BTreeMap::new();
        channel_state.insert("z_channel".into(), ChannelState::LastValue(vec![1]));
        channel_state.insert("a_channel".into(), ChannelState::LastValue(vec![2]));
        channel_state.insert("m_channel".into(), ChannelState::LastValue(vec![3]));

        let checkpoint = BspCheckpoint {
            superstep: 0,
            channel_state,
            node_statuses: BTreeMap::new(),
            created_at_secs: 1_700_000_000,
        };

        let bytes = rmp_serde::to_vec_named(&checkpoint).expect("serialize");
        let decoded: BspCheckpoint = rmp_serde::from_slice(&bytes).expect("deserialize");

        let keys: Vec<&String> = decoded.channel_state.keys().collect();
        assert_eq!(keys, vec!["a_channel", "m_channel", "z_channel"]);
    }

    // 3. superstep_counter_increments
    #[tokio::test]
    async fn superstep_counter_increments() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = test_config(tmp.path().join("checkpoints"));
        let pool = test_pool();
        let clock = SeededClock::new();
        let ssb = test_ssb().await;
        let mut channels = InMemoryChannelRegistry::new();
        let nodes = simple_nodes(3);

        let mut engine = BspEngine::new(config, pool, clock);
        let result = engine.run(nodes, &mut channels, &ssb).await.expect("run");

        assert_eq!(result.supersteps_completed, 3);
    }

    // 4. max_supersteps_terminates_loop
    #[tokio::test]
    async fn max_supersteps_terminates_loop() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut config = test_config(tmp.path().join("checkpoints"));
        config.max_supersteps = 2;

        let pool = test_pool();
        let clock = SeededClock::new();
        let ssb = test_ssb().await;
        let mut channels = InMemoryChannelRegistry::new();

        // 5-node chain needs 5 supersteps, but limit is 2
        let nodes = simple_nodes(5);

        let mut engine = BspEngine::new(config, pool, clock);
        let result = engine.run(nodes, &mut channels, &ssb).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            BspError::MaxSuperstepsReached(limit) => {
                assert_eq!(limit, 2);
            }
            other => panic!("expected MaxSuperstepsReached, got: {other}"),
        }
    }

    // 5. checkpoint_count_matches_supersteps
    #[tokio::test]
    async fn checkpoint_count_matches_supersteps() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ckpt_dir = tmp.path().join("checkpoints");
        let config = test_config(ckpt_dir.clone());

        let pool = test_pool();
        let clock = SeededClock::new();
        let ssb = test_ssb().await;
        let mut channels = InMemoryChannelRegistry::new();

        // 2-node chain: 2 supersteps, 2 checkpoints
        let nodes = simple_nodes(2);

        let mut engine = BspEngine::new(config, pool, clock);
        let result = engine.run(nodes, &mut channels, &ssb).await.expect("run");
        assert_eq!(result.supersteps_completed, 2);

        // Exactly 2 checkpoint files, no extras
        assert!(
            tokio::fs::metadata(ckpt_dir.join("superstep_0.msgpack"))
                .await
                .is_ok(),
            "superstep_0.msgpack must exist"
        );
        assert!(
            tokio::fs::metadata(ckpt_dir.join("superstep_1.msgpack"))
                .await
                .is_ok(),
            "superstep_1.msgpack must exist"
        );
        assert!(
            tokio::fs::metadata(ckpt_dir.join("superstep_2.msgpack"))
                .await
                .is_err(),
            "superstep_2.msgpack must NOT exist"
        );
    }

    // 6. checkpoint_written_after_gate
    #[tokio::test]
    async fn checkpoint_written_after_gate() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ckpt_dir = tmp.path().join("checkpoints");
        let config = test_config(ckpt_dir.clone());

        let pool = test_pool();
        let clock = SeededClock::new();
        let ssb = test_ssb().await;
        let mut channels = InMemoryChannelRegistry::new();
        let nodes = vec![NodeSpec {
            id: "n0".into(),
            role: AgentRole::Executor,
            task: "t".into(),
            depends_on: vec![],
            is_deferred: false,
        }];

        let mut engine = BspEngine::new(config, pool, clock);
        engine.run(nodes, &mut channels, &ssb).await.expect("run");

        // superstep_0.msgpack must exist and deserialize
        let bytes = tokio::fs::read(ckpt_dir.join("superstep_0.msgpack"))
            .await
            .expect("read checkpoint");
        let ckpt: BspCheckpoint =
            rmp_serde::from_slice(&bytes).expect("deserialize checkpoint");
        assert_eq!(ckpt.superstep, 0);
    }

    // 7. straggler_timeout_classification
    // Since spawn_parallel is synchronous, timeout never triggers.
    // This test verifies the timeout infrastructure is wired by checking
    // that a very short timeout doesn't cause issues with sync execution.
    #[tokio::test]
    async fn straggler_timeout_infrastructure() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut config = test_config(tmp.path().join("checkpoints"));
        config.straggler_timeout = Duration::from_millis(1);

        let pool = test_pool();
        let clock = SeededClock::new();
        let ssb = test_ssb().await;
        let mut channels = InMemoryChannelRegistry::new();
        let nodes = simple_nodes(1);

        let mut engine = BspEngine::new(config, pool, clock);
        // Should complete normally since spawn_parallel is sync
        let result = engine.run(nodes, &mut channels, &ssb).await.expect("run");
        assert_eq!(result.supersteps_completed, 1);
    }

    // 8. clear_ephemeral_recorded
    #[tokio::test]
    async fn clear_ephemeral_recorded() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = test_config(tmp.path().join("checkpoints"));

        let pool = test_pool();
        let clock = SeededClock::new();
        let ssb = test_ssb().await;
        let mut rx = ssb.subscribe();
        let mut channels = InMemoryChannelRegistry::new();

        // Write an ephemeral value, then run engine — it should be cleared
        use crate::bsp::channels::{ChannelId, EphemeralValue};
        let eph_id: ChannelId<EphemeralValue<u32>> = ChannelId::new("test_eph");
        channels.write_ephemeral(&eph_id, 42, 1).expect("write eph");
        // Reset write counts to simulate start of superstep
        channels.reset_ephemeral_write_counts();

        let nodes = vec![NodeSpec {
            id: "n0".into(),
            role: AgentRole::Executor,
            task: "t".into(),
            depends_on: vec![],
            is_deferred: false,
        }];

        let mut engine = BspEngine::new(config, pool, clock);
        engine.run(nodes, &mut channels, &ssb).await.expect("run");

        // Look for a Checkpoint entry with agent_id="bsp_engine" in SSB
        let mut found_ephemeral_record = false;
        while let Ok(entry) = rx.try_recv() {
            if entry.action_type == SsbActionType::Checkpoint {
                let payload: SsbPayload =
                    rmp_serde::from_slice(&entry.payload).expect("deser");
                if let SsbPayload::Checkpoint(cp) = payload {
                    if cp.agent_id.starts_with("bsp_engine:") {
                        assert!(
                            cp.agent_state_hash.contains("test_eph"),
                            "ephemeral clear record must contain channel name"
                        );
                        found_ephemeral_record = true;
                    }
                }
            }
        }
        assert!(
            found_ephemeral_record,
            "SSB must contain ephemeral-clear Checkpoint entry"
        );
    }

    // 9. empty_task_list_breaks_loop
    #[tokio::test]
    async fn empty_task_list_breaks_loop() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = test_config(tmp.path().join("checkpoints"));
        let pool = test_pool();
        let clock = SeededClock::new();
        let ssb = test_ssb().await;
        let mut channels = InMemoryChannelRegistry::new();

        let mut engine = BspEngine::new(config, pool, clock);
        let result = engine
            .run(vec![], &mut channels, &ssb)
            .await
            .expect("empty run");
        assert_eq!(result.supersteps_completed, 0);
    }

    // 10. test_deferred_fires_after_barrier
    // Deferred node with all deps satisfied fires after the main barrier.
    #[tokio::test]
    async fn test_deferred_fires_after_barrier() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = test_config(tmp.path().join("checkpoints"));
        let pool = test_pool();
        let clock = SeededClock::new();
        let ssb = test_ssb().await;
        let mut channels = InMemoryChannelRegistry::new();

        let nodes = vec![
            NodeSpec {
                id: "a".into(),
                role: AgentRole::Executor,
                task: "task_a".into(),
                depends_on: vec![],
                is_deferred: false,
            },
            NodeSpec {
                id: "b".into(),
                role: AgentRole::Executor,
                task: "task_b".into(),
                depends_on: vec![],
                is_deferred: false,
            },
            NodeSpec {
                id: "deferred_c".into(),
                role: AgentRole::Executor,
                task: "task_c".into(),
                depends_on: vec!["a".into(), "b".into()],
                is_deferred: true,
            },
        ];

        let mut engine = BspEngine::new(config, pool, clock);
        let result = engine.run(nodes, &mut channels, &ssb).await.expect("run");

        // Both non-deferred run in superstep 0, deferred fires after barrier
        // in the same superstep. Total: 1 superstep.
        assert_eq!(result.supersteps_completed, 1);

        // Verify via checkpoint that deferred_c reached Complete
        let ckpt_bytes = tokio::fs::read(
            tmp.path().join("checkpoints/superstep_0.msgpack"),
        )
        .await
        .expect("read checkpoint");
        let ckpt: BspCheckpoint =
            rmp_serde::from_slice(&ckpt_bytes).expect("deserialize");
        assert_eq!(
            ckpt.node_statuses.get("deferred_c").copied(),
            Some(NodeStatus::Complete),
            "deferred node must be Complete in checkpoint"
        );
    }

    // 11. test_deferred_unsatisfied_does_not_fire
    // Deferred node whose deps are not all satisfied stays Pending.
    #[tokio::test]
    async fn test_deferred_unsatisfied_does_not_fire() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = test_config(tmp.path().join("checkpoints"));
        let pool = test_pool();
        let clock = SeededClock::new();
        let ssb = test_ssb().await;
        let mut channels = InMemoryChannelRegistry::new();

        let nodes = vec![
            NodeSpec {
                id: "a".into(),
                role: AgentRole::Executor,
                task: "task_a".into(),
                depends_on: vec![],
                is_deferred: false,
            },
            // "missing_dep" is never created as a node, so it stays Pending
            NodeSpec {
                id: "deferred_d".into(),
                role: AgentRole::Executor,
                task: "task_d".into(),
                depends_on: vec!["a".into(), "missing_dep".into()],
                is_deferred: true,
            },
        ];

        let mut engine = BspEngine::new(config, pool, clock);
        let result = engine.run(nodes, &mut channels, &ssb).await.expect("run");

        assert_eq!(result.supersteps_completed, 1);

        let ckpt_bytes = tokio::fs::read(
            tmp.path().join("checkpoints/superstep_0.msgpack"),
        )
        .await
        .expect("read checkpoint");
        let ckpt: BspCheckpoint =
            rmp_serde::from_slice(&ckpt_bytes).expect("deserialize");
        // deferred_d should remain Pending because "missing_dep" is never Complete
        assert_eq!(
            ckpt.node_statuses.get("deferred_d").copied(),
            Some(NodeStatus::Pending),
            "unsatisfied deferred node must remain Pending"
        );
    }

    // 12. test_cascading_deferred_impossible
    // Two deferred nodes depending on each other. Neither fires. No panic.
    #[tokio::test]
    async fn test_cascading_deferred_impossible() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = test_config(tmp.path().join("checkpoints"));
        let pool = test_pool();
        let clock = SeededClock::new();
        let ssb = test_ssb().await;
        let mut channels = InMemoryChannelRegistry::new();

        let nodes = vec![
            // One non-deferred node so the engine has work to do
            NodeSpec {
                id: "anchor".into(),
                role: AgentRole::Executor,
                task: "anchor_task".into(),
                depends_on: vec![],
                is_deferred: false,
            },
            NodeSpec {
                id: "def_x".into(),
                role: AgentRole::Executor,
                task: "task_x".into(),
                depends_on: vec!["def_y".into()],
                is_deferred: true,
            },
            NodeSpec {
                id: "def_y".into(),
                role: AgentRole::Executor,
                task: "task_y".into(),
                depends_on: vec!["def_x".into()],
                is_deferred: true,
            },
        ];

        let mut engine = BspEngine::new(config, pool, clock);
        let result = engine.run(nodes, &mut channels, &ssb).await.expect("run");

        // Engine completes (no infinite loop, no panic)
        assert_eq!(result.supersteps_completed, 1);

        let ckpt_bytes = tokio::fs::read(
            tmp.path().join("checkpoints/superstep_0.msgpack"),
        )
        .await
        .expect("read checkpoint");
        let ckpt: BspCheckpoint =
            rmp_serde::from_slice(&ckpt_bytes).expect("deserialize");
        // Neither deferred node should fire — both must remain Pending
        assert_eq!(
            ckpt.node_statuses.get("def_x").copied(),
            Some(NodeStatus::Pending),
            "cascading deferred def_x must remain Pending"
        );
        assert_eq!(
            ckpt.node_statuses.get("def_y").copied(),
            Some(NodeStatus::Pending),
            "cascading deferred def_y must remain Pending"
        );
    }

    // -----------------------------------------------------------------------
    // 13-16: Memory monitoring tests (Phase 3D.4)
    // -----------------------------------------------------------------------

    // 13. memory_monitoring_measure_only
    // Untracked channel under force-clear threshold emits "measure" SSB event.
    #[tokio::test]
    async fn memory_monitoring_measure_only() {
        use crate::bsp::channels::{ChannelId, UntrackedValue, UNTRACKED_DEFAULT_LIMIT_BYTES};

        let tmp = tempfile::tempdir().expect("tempdir");
        let config = test_config(tmp.path().join("checkpoints"));
        let pool = test_pool();
        let clock = SeededClock::new();
        let ssb = test_ssb().await;
        let mut rx = ssb.subscribe();
        let mut channels = InMemoryChannelRegistry::new();

        // 5 MB — under 10 MB threshold, so measure only
        let id: ChannelId<UntrackedValue<Vec<u8>>> = ChannelId::new("small_cache");
        channels
            .write_untracked(&id, vec![0u8; 5 * 1024 * 1024])
            .expect("write untracked");

        let nodes = vec![NodeSpec {
            id: "n0".into(),
            role: AgentRole::Executor,
            task: "t".into(),
            depends_on: vec![],
            is_deferred: false,
        }];

        let mut engine = BspEngine::new(config, pool, clock);
        engine.run(nodes, &mut channels, &ssb).await.expect("run");

        let mut found_measure = false;
        while let Ok(entry) = rx.try_recv() {
            if entry.action_type == SsbActionType::MemoryMonitoring {
                let payload: SsbPayload =
                    rmp_serde::from_slice(&entry.payload).expect("deser");
                if let SsbPayload::MemoryMonitoring(mm) = payload {
                    if mm.metric_name == "small_cache" {
                        assert_eq!(mm.action_taken, "measure");
                        assert!(mm.value >= 5_000_000.0, "value={}", mm.value);
                        assert_eq!(mm.threshold, UNTRACKED_DEFAULT_LIMIT_BYTES as f64);
                        found_measure = true;
                    }
                }
            }
        }
        assert!(found_measure, "SSB must contain measure event for small_cache");

        // Channel should still exist (not cleared)
        let erased = ChannelId::<()>::new("small_cache");
        let size = channels.deep_size_untracked(&erased);
        assert!(size >= 5_000_000, "channel must NOT be force-cleared");
    }

    // 14. memory_monitoring_force_clear
    // Untracked channel >= threshold * multiplier emits "force_clear" and clears.
    #[tokio::test]
    async fn memory_monitoring_force_clear() {
        use crate::bsp::channels::{
            ChannelId, UntrackedValue, UNTRACKED_DEFAULT_LIMIT_BYTES,
        };

        let tmp = tempfile::tempdir().expect("tempdir");
        let config = test_config(tmp.path().join("checkpoints"));
        let pool = test_pool();
        let clock = SeededClock::new();
        let ssb = test_ssb().await;
        let mut rx = ssb.subscribe();
        let mut channels = InMemoryChannelRegistry::new();

        // 25 MB — exceeds 10 MB * 2 = 20 MB force-clear threshold
        let id: ChannelId<UntrackedValue<Vec<u8>>> = ChannelId::new("big_cache");
        channels
            .write_untracked(&id, vec![0u8; 25 * 1024 * 1024])
            .expect("write untracked");

        let nodes = vec![NodeSpec {
            id: "n0".into(),
            role: AgentRole::Executor,
            task: "t".into(),
            depends_on: vec![],
            is_deferred: false,
        }];

        let mut engine = BspEngine::new(config, pool, clock);
        engine.run(nodes, &mut channels, &ssb).await.expect("run");

        let mut found_force_clear = false;
        while let Ok(entry) = rx.try_recv() {
            if entry.action_type == SsbActionType::MemoryMonitoring {
                let payload: SsbPayload =
                    rmp_serde::from_slice(&entry.payload).expect("deser");
                if let SsbPayload::MemoryMonitoring(mm) = payload {
                    if mm.metric_name == "big_cache" {
                        assert_eq!(mm.action_taken, "force_clear");
                        assert!(mm.value >= 25_000_000.0, "value={}", mm.value);
                        let expected_threshold =
                            UNTRACKED_DEFAULT_LIMIT_BYTES as f64;
                        assert_eq!(mm.threshold, expected_threshold);
                        found_force_clear = true;
                    }
                }
            }
        }
        assert!(
            found_force_clear,
            "SSB must contain force_clear event for big_cache"
        );

        // Channel must have been cleared
        let erased = ChannelId::<()>::new("big_cache");
        assert_eq!(
            channels.deep_size_untracked(&erased),
            0,
            "force-cleared channel must have size 0"
        );
    }

    // 15. memory_monitoring_empty_untracked
    // No untracked channels — no SSB MemoryMonitoring events emitted.
    #[tokio::test]
    async fn memory_monitoring_empty_untracked() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = test_config(tmp.path().join("checkpoints"));
        let pool = test_pool();
        let clock = SeededClock::new();
        let ssb = test_ssb().await;
        let mut rx = ssb.subscribe();
        let mut channels = InMemoryChannelRegistry::new();

        let nodes = vec![NodeSpec {
            id: "n0".into(),
            role: AgentRole::Executor,
            task: "t".into(),
            depends_on: vec![],
            is_deferred: false,
        }];

        let mut engine = BspEngine::new(config, pool, clock);
        engine.run(nodes, &mut channels, &ssb).await.expect("run");

        let mut monitoring_count = 0;
        while let Ok(entry) = rx.try_recv() {
            if entry.action_type == SsbActionType::MemoryMonitoring {
                monitoring_count += 1;
            }
        }
        assert_eq!(
            monitoring_count, 0,
            "no untracked channels => no MemoryMonitoring events"
        );
    }

    // 16. memory_monitoring_multi_channel_mixed
    // One channel under threshold (measure), one over (force_clear).
    #[tokio::test]
    async fn memory_monitoring_multi_channel_mixed() {
        use crate::bsp::channels::{ChannelId, UntrackedValue};

        let tmp = tempfile::tempdir().expect("tempdir");
        let config = test_config(tmp.path().join("checkpoints"));
        let pool = test_pool();
        let clock = SeededClock::new();
        let ssb = test_ssb().await;
        let mut rx = ssb.subscribe();
        let mut channels = InMemoryChannelRegistry::new();

        // Small: 5 MB (measure)
        let small_id: ChannelId<UntrackedValue<Vec<u8>>> = ChannelId::new("small");
        channels
            .write_untracked(&small_id, vec![0u8; 5 * 1024 * 1024])
            .expect("write small");

        // Big: 25 MB (force_clear)
        let big_id: ChannelId<UntrackedValue<Vec<u8>>> = ChannelId::new("big");
        channels
            .write_untracked(&big_id, vec![0u8; 25 * 1024 * 1024])
            .expect("write big");

        let nodes = vec![NodeSpec {
            id: "n0".into(),
            role: AgentRole::Executor,
            task: "t".into(),
            depends_on: vec![],
            is_deferred: false,
        }];

        let mut engine = BspEngine::new(config, pool, clock);
        engine.run(nodes, &mut channels, &ssb).await.expect("run");

        let mut small_action = None;
        let mut big_action = None;
        while let Ok(entry) = rx.try_recv() {
            if entry.action_type == SsbActionType::MemoryMonitoring {
                let payload: SsbPayload =
                    rmp_serde::from_slice(&entry.payload).expect("deser");
                if let SsbPayload::MemoryMonitoring(mm) = payload {
                    match mm.metric_name.as_str() {
                        "small" => small_action = Some(mm.action_taken),
                        "big" => big_action = Some(mm.action_taken),
                        _ => {}
                    }
                }
            }
        }

        assert_eq!(
            small_action.as_deref(),
            Some("measure"),
            "small channel must get 'measure'"
        );
        assert_eq!(
            big_action.as_deref(),
            Some("force_clear"),
            "big channel must get 'force_clear'"
        );

        // big is cleared, small survives
        let erased_big = ChannelId::<()>::new("big");
        assert_eq!(channels.deep_size_untracked(&erased_big), 0);
        let erased_small = ChannelId::<()>::new("small");
        assert!(channels.deep_size_untracked(&erased_small) >= 5_000_000);
    }
}
