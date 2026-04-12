use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use elai_core::ClockProvider;

use elai_reasoning::BasicGrcEngine;
use crate::agent_pool::{AgentHandle, AgentStatus};
use crate::checkpoint::{CheckpointManager, GrcHistoryEntry};
use crate::ssb::{
    DeathEventPayload, RecoveryEventPayload, Ssb, SsbPayload,
};

// ---------------------------------------------------------------------------
// AgentDeathClassification
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentDeathClassification {
    Crash,
    CleanExit,
    // STUB(3C.2): heartbeat-based death detection not yet implemented.
    // Timeout is accepted but treated identically to Crash for recovery.
    Timeout,
    BudgetExceeded,
}

impl AgentDeathClassification {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Crash => "crash",
            Self::CleanExit => "clean_exit",
            Self::Timeout => "timeout",
            Self::BudgetExceeded => "budget_exceeded",
        }
    }
}

// ---------------------------------------------------------------------------
// RecoveryError
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum RecoveryError {
    #[error("checkpoint read error: {0}")]
    CheckpointRead(#[from] crate::checkpoint::CheckpointError),

    #[error("SSB write error during recovery: {0}")]
    SsbWrite(#[from] crate::ssb::SsbError),

    #[error("recovery denied: GRC budget exceeded, no restart permitted")]
    BudgetExceeded,

    #[error("recovery invariant violated: {0}")]
    InvariantViolation(String),
}

// ---------------------------------------------------------------------------
// AgentRecoveryContext: local data assembled during recover_agent()
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AgentRecoveryContext {
    pub classification: AgentDeathClassification,
    pub had_checkpoint: bool,
    pub degraded: bool,
    pub prior_step_index: i64,
    pub first_grounding_source: String,
    pub restored_grc_history: Vec<GrcHistoryEntry>,
    pub restored_active_files: Vec<PathBuf>,
}

// ---------------------------------------------------------------------------
// RecoveryManager<C: ClockProvider>
// ---------------------------------------------------------------------------

pub struct RecoveryManager<C: ClockProvider> {
    _clock: C,
}

impl<C: ClockProvider> RecoveryManager<C> {
    pub fn new(clock: C) -> Self {
        Self { _clock: clock }
    }

    /// Recover a dead agent. Returns a new AgentHandle with the same agent_id
    /// and status=Spawning, plus a AgentRecoveryContext with restoration metadata.
    ///
    /// BudgetExceeded classification returns Err immediately (no SSB writes,
    /// no agent spawn). All other classifications attempt checkpoint recovery.
    pub async fn recover_agent(
        &self,
        dead_handle: &AgentHandle,
        classification: AgentDeathClassification,
        prior_step_index: i64,
        ssb: &Ssb,
    ) -> Result<(AgentHandle, AgentRecoveryContext), RecoveryError> {
        // Budget exceeded: no restart per spec (GRC overflow -> convergence)
        if classification == AgentDeathClassification::BudgetExceeded {
            return Err(RecoveryError::BudgetExceeded);
        }

        let agent_id_str = dead_handle.agent_id.to_string();
        let checkpoint_dir = dead_handle
            .scratch_dir
            .join(dead_handle.agent_id.to_string());
        let checkpoint_path_fwd = checkpoint_dir
            .to_string_lossy()
            .replace('\\', "/");

        // Read checkpoint (static method, no &self on CheckpointManager)
        let checkpoint_data =
            CheckpointManager::<C>::read_latest(dead_handle).await?;

        let had_checkpoint = checkpoint_data.is_some();
        let degraded = !had_checkpoint;

        // Write death_event to SSB
        let death_payload = SsbPayload::DeathEvent(DeathEventPayload {
            agent_id: agent_id_str.clone(),
            classification: classification.as_str().to_owned(),
            prior_step_index,
            checkpoint_path: checkpoint_path_fwd.clone(),
            had_checkpoint,
        });
        ssb.write_payload(&agent_id_str, prior_step_index, death_payload, vec![])
            .await?;

        // Build recovery context from checkpoint or fresh start
        let (restored_grc_history, restored_active_files, first_grounding_source) =
            match checkpoint_data {
                Some(data) => {
                    let source =
                        format!("CHECKPOINT:{checkpoint_path_fwd}");
                    (data.grc_history, data.active_files, source)
                }
                None => {
                    let source = format!(
                        "CHECKPOINT:{checkpoint_path_fwd}:MISSING"
                    );
                    (Vec::new(), Vec::new(), source)
                }
            };

        let recovery_step_index = prior_step_index
            .checked_add(1)
            .ok_or_else(|| {
                RecoveryError::InvariantViolation(
                    "superstep overflow: prior_step_index is i64::MAX".into(),
                )
            })?;

        // Write recovery_event to SSB (superstep = prior + 1, monotonic)
        let recovery_payload =
            SsbPayload::RecoveryEvent(RecoveryEventPayload {
                agent_id: agent_id_str.clone(),
                classification: classification.as_str().to_owned(),
                recovered_from_checkpoint: had_checkpoint,
                degraded,
                first_grounding_source: first_grounding_source.clone(),
                prior_step_index,
                recovery_step_index,
            });
        ssb.write_payload(
            &agent_id_str,
            recovery_step_index,
            recovery_payload,
            vec![],
        )
        .await?;

        // Build recovered handle: same agent_id, status = Spawning.
        // A fresh reasoning engine is created from the cloned grc_config so that
        // the recovered agent starts with a clean topic budget (no leftover state
        // from the failed execution leaks into the recovery run).
        let recovered_handle = AgentHandle {
            agent_id: dead_handle.agent_id,
            role: dead_handle.role,
            status: AgentStatus::Spawning,
            task: dead_handle.task.clone(),
            active_files: restored_active_files.clone(),
            grc_engine: BasicGrcEngine::with_config(dead_handle.grc_config.clone()),
            grc_config: dead_handle.grc_config.clone(),
            scratch_dir: dead_handle.scratch_dir.clone(),
            sandbox_handle: dead_handle.sandbox_handle.clone(),
        };

        let context = AgentRecoveryContext {
            classification,
            had_checkpoint,
            degraded,
            prior_step_index,
            first_grounding_source,
            restored_grc_history,
            restored_active_files,
        };

        Ok((recovered_handle, context))
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_pool::{AgentRole, AgentStatus};
    use crate::checkpoint::{AgentCheckpointData, CheckpointManager};
    use crate::ssb::{Ssb, SsbActionType, SsbConfig};
    use elai_core::SeededClock;
    use elai_reasoning::{AgentGrcConfig, BasicGrcEngine, GrcBudget, GrcState, GroundingLevel};
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    fn test_grc_config() -> AgentGrcConfig {
        AgentGrcConfig {
            role: "executor".into(),
            budget: GrcBudget {
                max_calls: 5,
                max_chars: 1000,
                calls_used: 0,
                chars_used: 0,
            },
            required_steps: vec![GrcState::Observe, GrcState::PlanAction, GrcState::Verify],
            grounding_level: GroundingLevel::Uncertain,
            csq_required: false,
            allowed_methods: vec![],
            transition_overrides: HashMap::new(),
            require_scope_enumeration: false,
        }
    }

    fn test_handle(scratch_dir: &std::path::Path) -> AgentHandle {
        let grc_config = test_grc_config();
        AgentHandle {
            agent_id: Uuid::from_bytes([
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x47, 0x08, 0x89, 0x0a, 0x0b, 0x0c, 0x0d,
                0x0e, 0x0f, 0x10,
            ]),
            role: AgentRole::Executor,
            status: AgentStatus::Running,
            task: "test-recovery-task".into(),
            active_files: vec![PathBuf::from("src/main.rs")],
            grc_engine: BasicGrcEngine::with_config(grc_config.clone()),
            grc_config,
            scratch_dir: scratch_dir.to_path_buf(),
        }
    }

    fn test_checkpoint_data() -> AgentCheckpointData {
        AgentCheckpointData {
            agent_status: "running".into(),
            grc_history: vec![GrcHistoryEntry {
                step_type: "observe".into(),
                content: "prior observation".into(),
                grounding: "file:src/main.rs:L1".into(),
                topic: "default".into(),
            }],
            active_files: vec![PathBuf::from("src/main.rs")],
            tool_results: vec!["result-1".into()],
            output_draft: "draft".into(),
            channel_references: vec![],
        }
    }

    async fn make_ssb(clock: SeededClock) -> Ssb {
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
            Arc::new(clock),
        )
        .expect("ssb creation")
    }

    async fn write_checkpoint(
        handle: &AgentHandle,
        data: &AgentCheckpointData,
        step_index: u64,
        ssb: &Ssb,
    ) {
        let clock = SeededClock::new();
        let mgr = CheckpointManager::new(clock);
        mgr.write(handle, data, step_index, ssb)
            .await
            .expect("checkpoint write");
    }

    // -----------------------------------------------------------------------
    // Unit tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn recover_agent_crash_with_checkpoint() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let ssb = make_ssb(SeededClock::new()).await;
        let ckpt_data = test_checkpoint_data();

        // Write a valid checkpoint at step 5
        write_checkpoint(&handle, &ckpt_data, 5, &ssb).await;

        let mgr = RecoveryManager::new(SeededClock::new());
        let mut rx = ssb.subscribe();

        let (recovered, ctx) = mgr
            .recover_agent(&handle, AgentDeathClassification::Crash, 5, &ssb)
            .await
            .expect("recovery should succeed");

        // agent_id preserved
        assert_eq!(recovered.agent_id, handle.agent_id);
        assert_eq!(recovered.status, AgentStatus::Spawning);
        assert!(ctx.had_checkpoint);
        assert!(!ctx.degraded);
        assert_eq!(ctx.restored_grc_history.len(), 1);
        assert!(ctx.first_grounding_source.starts_with("CHECKPOINT:"));

        // SSB should have death_event then recovery_event
        let entry1 = rx.recv().await.expect("death_event");
        assert_eq!(entry1.action_type, SsbActionType::DeathEvent);
        let entry2 = rx.recv().await.expect("recovery_event");
        assert_eq!(entry2.action_type, SsbActionType::RecoveryEvent);
        assert_eq!(entry2.superstep, 6); // prior(5) + 1
    }

    #[tokio::test]
    async fn recover_agent_crash_no_checkpoint() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let ssb = make_ssb(SeededClock::new()).await;

        // No checkpoint written — scratch dir exists but no checkpoint files
        let mgr = RecoveryManager::new(SeededClock::new());

        let (recovered, ctx) = mgr
            .recover_agent(&handle, AgentDeathClassification::Crash, 3, &ssb)
            .await
            .expect("recovery should succeed even without checkpoint");

        assert_eq!(recovered.agent_id, handle.agent_id);
        assert_eq!(recovered.status, AgentStatus::Spawning);
        assert!(!ctx.had_checkpoint);
        assert!(ctx.degraded);
        assert!(ctx.restored_grc_history.is_empty());
        assert!(ctx.restored_active_files.is_empty());
        assert!(ctx.first_grounding_source.contains("MISSING"));
    }

    #[tokio::test]
    async fn recover_agent_budget_exceeded() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let ssb = make_ssb(SeededClock::new()).await;
        let mut rx = ssb.subscribe();

        let mgr = RecoveryManager::new(SeededClock::new());
        let result = mgr
            .recover_agent(
                &handle,
                AgentDeathClassification::BudgetExceeded,
                10,
                &ssb,
            )
            .await;

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), RecoveryError::BudgetExceeded),
            "should be BudgetExceeded variant"
        );

        // No SSB writes should have occurred
        assert!(
            rx.try_recv().is_err(),
            "no SSB entries should exist for BudgetExceeded"
        );
    }

    #[tokio::test]
    async fn recovery_preserves_agent_id() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let ssb = make_ssb(SeededClock::new()).await;

        let mgr = RecoveryManager::new(SeededClock::new());
        let (recovered, _ctx) = mgr
            .recover_agent(&handle, AgentDeathClassification::Crash, 0, &ssb)
            .await
            .expect("recovery");

        assert_eq!(
            recovered.agent_id, handle.agent_id,
            "recovered agent_id must equal original agent_id"
        );
    }

    #[tokio::test]
    async fn ssb_step_index_monotonic() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let ssb = make_ssb(SeededClock::new()).await;
        let mut rx = ssb.subscribe();

        let mgr = RecoveryManager::new(SeededClock::new());
        let prior = 42_i64;

        mgr.recover_agent(&handle, AgentDeathClassification::Crash, prior, &ssb)
            .await
            .expect("recovery");

        // Skip death_event
        let _death = rx.recv().await.expect("death");
        let recovery = rx.recv().await.expect("recovery");
        assert!(
            recovery.superstep > prior,
            "recovery_event superstep ({}) must be > prior_step_index ({prior})",
            recovery.superstep
        );
        assert_eq!(recovery.superstep, prior + 1);
    }

    #[tokio::test]
    async fn schema_migration_v2() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("migration_test.db");
        let clock = Arc::new(SeededClock::new());

        let _ssb = Ssb::new(
            SsbConfig {
                db_path: db_path.clone(),
                project_id: "test".into(),
                session_id: "test".into(),
                pool_size: None,
            },
            clock,
        )
        .expect("ssb v2");

        // Verify the CHECK constraint includes death_event and recovery_event
        let conn =
            rusqlite::Connection::open(&db_path).expect("open db");
        let sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name='ssb'",
                [],
                |row| row.get(0),
            )
            .expect("schema query");

        assert!(
            sql.contains("death_event"),
            "CHECK constraint must include death_event, got: {sql}"
        );
        assert!(
            sql.contains("recovery_event"),
            "CHECK constraint must include recovery_event, got: {sql}"
        );
    }

    #[tokio::test]
    async fn schema_migration_idempotent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("idempotent_test.db");
        let clock = Arc::new(SeededClock::new());

        // Create twice on same path
        let ssb1 = Ssb::new(
            SsbConfig {
                db_path: db_path.clone(),
                project_id: "test".into(),
                session_id: "test".into(),
                pool_size: None,
            },
            clock.clone(),
        );
        assert!(ssb1.is_ok(), "first creation should succeed");
        drop(ssb1);

        let ssb2 = Ssb::new(
            SsbConfig {
                db_path: db_path.clone(),
                project_id: "test".into(),
                session_id: "test".into(),
                pool_size: None,
            },
            clock,
        );
        assert!(ssb2.is_ok(), "second creation should succeed (idempotent)");
    }

    #[tokio::test]
    async fn recover_agent_timeout_treated_as_crash() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let ssb = make_ssb(SeededClock::new()).await;

        let mgr = RecoveryManager::new(SeededClock::new());
        // STUB(3C.2): Timeout treated same as Crash
        let (recovered, ctx) = mgr
            .recover_agent(&handle, AgentDeathClassification::Timeout, 1, &ssb)
            .await
            .expect("timeout recovery should succeed");

        assert_eq!(recovered.agent_id, handle.agent_id);
        assert_eq!(recovered.status, AgentStatus::Spawning);
        assert_eq!(ctx.classification, AgentDeathClassification::Timeout);
    }

    #[tokio::test]
    async fn recover_agent_clean_exit() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let ssb = make_ssb(SeededClock::new()).await;

        let mgr = RecoveryManager::new(SeededClock::new());
        let (recovered, ctx) = mgr
            .recover_agent(&handle, AgentDeathClassification::CleanExit, 2, &ssb)
            .await
            .expect("clean_exit recovery should succeed");

        assert_eq!(recovered.agent_id, handle.agent_id);
        assert_eq!(ctx.classification, AgentDeathClassification::CleanExit);
    }

    #[tokio::test]
    async fn recovery_context_grounding_source_format() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let ssb = make_ssb(SeededClock::new()).await;
        let ckpt_data = test_checkpoint_data();

        write_checkpoint(&handle, &ckpt_data, 1, &ssb).await;

        let mgr = RecoveryManager::new(SeededClock::new());
        let (_recovered, ctx) = mgr
            .recover_agent(&handle, AgentDeathClassification::Crash, 1, &ssb)
            .await
            .expect("recovery");

        // Must start with "CHECKPOINT:" and use forward slashes
        assert!(
            ctx.first_grounding_source.starts_with("CHECKPOINT:"),
            "grounding source must start with CHECKPOINT:"
        );
        assert!(
            !ctx.first_grounding_source.contains('\\'),
            "grounding source must use forward slashes, got: {}",
            ctx.first_grounding_source
        );
    }
}
