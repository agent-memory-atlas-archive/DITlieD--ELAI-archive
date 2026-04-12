use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use elai_core::EntropySource;
use elai_reasoning::{AgentGrcConfig, BasicGrcEngine, GrcBudget, GrcEngine, GrcState, GroundingLevel};

use crate::sandbox::{AgentSandbox, AppContainerConfig, SandboxHandle};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum VRAM (bytes) required to spawn an agent with a loaded model.
/// Below this threshold, spawn() returns ResourceInsufficient.
pub const VRAM_MIN_BYTES: u64 = 512 * 1024 * 1024; // 512 MiB

// ---------------------------------------------------------------------------
// AgentRole: the 8 agent roles (architecture-overview.txt section 9)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    Executor,
    Auditor,
    Researcher,
    Designer,
    PromptEngineer,
    ContextCompiler,
    MemoryWriter,
    MediaValidator,
}

// ---------------------------------------------------------------------------
// AgentStatus: lifecycle states (phase-3C.txt L100-105)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Spawning,
    Ready,
    Running,
    Checkpointing,
    Finished,
    Failed,
}

// ---------------------------------------------------------------------------
// ModelPool: VRAM query abstraction (stub — real impl in providers/ later)
// ---------------------------------------------------------------------------

pub trait ModelPool: Send + Sync {
    /// Available VRAM in bytes. u64::MAX means unconstrained (no GPU check).
    fn available_vram(&self) -> u64;
}

// ---------------------------------------------------------------------------
// AgentHandle: per-agent metadata exposed to orchestrator + downstream phases
// ---------------------------------------------------------------------------

pub struct AgentHandle {
    pub agent_id: Uuid,
    pub role: AgentRole,
    pub status: AgentStatus,
    pub task: String,
    pub active_files: Vec<PathBuf>,
    pub grc_config: AgentGrcConfig,
    pub scratch_dir: PathBuf,
    /// Reasoning engine instance configured for this agent's role.
    /// Instantiated at spawn time with the role-specific GRC config applied via
    /// `GrcEngine::with_agent_config`. Each handle owns its own engine so that
    /// topic budgets are isolated per-agent.
    pub grc_engine: BasicGrcEngine,
    /// Sandbox handle for process isolation (Phase 3J.11 AppContainer)
    /// None for in-process agents (Phase 3 Tier A), Some for out-of-process (Phase 4A)
    pub sandbox_handle: Option<SandboxHandle>,
}

impl Clone for AgentHandle {
    /// Produces a structurally identical handle with a fresh `BasicGrcEngine`
    /// seeded from the same `grc_config`. Topic state (budget counters, step
    /// history) is intentionally NOT cloned — each cloned handle starts with a
    /// clean engine. This mirrors the intent of `with_agent_config`, which
    /// resets topic state.
    fn clone(&self) -> Self {
        let mut engine = BasicGrcEngine::new();
        engine.with_agent_config(self.grc_config.clone());
        Self {
            agent_id: self.agent_id,
            role: self.role,
            status: self.status,
            task: self.task.clone(),
            active_files: self.active_files.clone(),
            grc_config: self.grc_config.clone(),
            scratch_dir: self.scratch_dir.clone(),
            grc_engine: engine,
            sandbox_handle: self.sandbox_handle.clone(),
        }
    }
}

impl std::fmt::Debug for AgentHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentHandle")
            .field("agent_id", &self.agent_id)
            .field("role", &self.role)
            .field("status", &self.status)
            .field("task", &self.task)
            .field("active_files", &self.active_files)
            .field("grc_config", &self.grc_config)
            .field("scratch_dir", &self.scratch_dir)
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// AgentMessage: inter-agent communication payload
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AgentMessage {
    pub sender: Uuid,
    pub recipient: Uuid,
    pub payload: String,
}

// ---------------------------------------------------------------------------
// AgentPoolConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AgentPoolConfig {
    pub scratch_base: PathBuf,
    pub vram_required_bytes: u64,
    /// Project path for sandbox filesystem grants
    pub project_path: PathBuf,
    /// Enable sandbox isolation (Phase 3J.11)
    /// For Phase 3 (in-process), this is false but infrastructure is in place
    /// For Phase 4A (out-of-process), this will be true
    pub enable_sandbox: bool,
}

impl Default for AgentPoolConfig {
    fn default() -> Self {
        Self {
            scratch_base: PathBuf::from(".elai_cc/sessions/default/agents"),
            vram_required_bytes: VRAM_MIN_BYTES,
            project_path: PathBuf::from("."), // Default to current directory
            enable_sandbox: false, // Disabled for Phase 3 (in-process)
        }
    }
}

// ---------------------------------------------------------------------------
// AgentPoolError
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum AgentPoolError {
    #[error("insufficient VRAM: {0}")]
    ResourceInsufficient(String),

    #[error("role not found: {0}")]
    RoleNotFound(String),

    #[error("agent not found: {0}")]
    AgentNotFound(Uuid),

    #[error("spawn failed: {0}")]
    SpawnFailed(String),
}

// ---------------------------------------------------------------------------
// AgentPool trait (object-safe for Phase 4A IPC transition)
// ---------------------------------------------------------------------------

pub trait AgentPool {
    fn spawn(
        &mut self,
        role: AgentRole,
        task: String,
    ) -> Result<AgentHandle, AgentPoolError>;

    fn spawn_parallel(
        &mut self,
        tasks: Vec<(AgentRole, String)>,
    ) -> Result<Vec<AgentHandle>, AgentPoolError>;

    /// Maximum number of agents that can execute concurrently based on VRAM budget.
    /// Engine uses this to limit JoinSet fan-out width.
    fn max_concurrent(&self) -> usize;

    fn communicate(
        &mut self,
        from: Uuid,
        to: Uuid,
        message: AgentMessage,
    ) -> Result<(), AgentPoolError>;
}

// ---------------------------------------------------------------------------
// InProcessAgentPool: in-process implementation (Phase 3 — Tier A only)
// ---------------------------------------------------------------------------

pub struct InProcessAgentPool<E: EntropySource> {
    agents: HashMap<Uuid, AgentHandle>,
    config: AgentPoolConfig,
    entropy: E,
    model_pool: Arc<dyn ModelPool>,
    /// Sandbox instance for process isolation (Phase 3J.11)
    /// None when sandbox is disabled (Phase 3 in-process)
    sandbox: Option<Box<dyn AgentSandbox>>,
}

impl<E: EntropySource> InProcessAgentPool<E> {
    pub fn new(config: AgentPoolConfig, entropy: E, model_pool: Arc<dyn ModelPool>) -> Self {
        let sandbox = if config.enable_sandbox {
            match crate::sandbox::create_sandbox() {
                Ok(sb) => Some(sb),
                Err(e) => {
                    eprintln!("Failed to create sandbox: {}, proceeding without isolation", e);
                    None
                }
            }
        } else {
            None
        };

        Self {
            agents: HashMap::new(),
            config,
            entropy,
            model_pool,
            sandbox,
        }
    }

    /// Generate a UUID v4 via EntropySource (entropy quarantine: no raw uuid::Uuid::new_v4()).
    fn generate_uuid(&mut self) -> Uuid {
        let mut bytes = [0u8; 16];
        self.entropy.fill_bytes(&mut bytes);
        // RFC 4122 version 4: set version bits (byte 6, high nibble = 0100)
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        // RFC 4122 variant 1: set variant bits (byte 8, high 2 bits = 10)
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        Uuid::from_bytes(bytes)
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn get_agent(&self, id: &Uuid) -> Option<&AgentHandle> {
        self.agents.get(id)
    }
}

impl<E: EntropySource> AgentPool for InProcessAgentPool<E> {
    fn spawn(
        &mut self,
        role: AgentRole,
        task: String,
    ) -> Result<AgentHandle, AgentPoolError> {
        // VRAM check: model_pool.available_vram() must meet minimum
        let available = self.model_pool.available_vram();
        if available < self.config.vram_required_bytes {
            return Err(AgentPoolError::ResourceInsufficient(format!(
                "need {} bytes, available {} bytes",
                self.config.vram_required_bytes, available,
            )));
        }

        let agent_id = self.generate_uuid();
        let grc_config = default_grc_config(&role);
        let scratch_dir = self.config.scratch_base.join(agent_id.to_string());

        // Instantiate a fresh reasoning engine and apply the role-specific GRC config.
        // with_agent_config resets all topic state, so each spawned agent starts clean.
        let mut grc_engine = BasicGrcEngine::new();
        grc_engine.with_agent_config(grc_config.clone());

        // Create sandbox handle if sandbox is enabled (Phase 3J.11)
        let sandbox_handle = if let Some(ref sandbox) = self.sandbox {
            let app_config = AppContainerConfig::new(
                agent_id,
                role,
                self.config.project_path.clone(),
                scratch_dir.clone(),
            );
            match sandbox.create(&app_config) {
                Ok(mut handle) => {
                    handle.agent_id = agent_id;
                    Some(handle)
                }
                Err(e) => {
                    eprintln!("Failed to create sandbox for agent {}: {}, proceeding without isolation", agent_id, e);
                    None
                }
            }
        } else {
            None
        };

        // task stored in handle for lifecycle use (3C.2 wires actual execution)
        let handle = AgentHandle {
            agent_id,
            role,
            status: AgentStatus::Spawning,
            task,
            active_files: Vec::new(),
            grc_config,
            scratch_dir,
            grc_engine,
            sandbox_handle,
        };

        self.agents.insert(agent_id, handle.clone());
        Ok(handle)
    }

    fn max_concurrent(&self) -> usize {
        let available = self.model_pool.available_vram();
        let per_agent = self.config.vram_required_bytes;
        // Guard: either zero → fallback to sequential (1 at a time)
        if available == 0 || per_agent == 0 {
            return 1;
        }
        // floor division, clamped to at least 1; safe on 32-bit targets
        let ratio = available / per_agent;
        let computed = usize::try_from(ratio).unwrap_or(usize::MAX);
        computed.max(1)
    }

    fn spawn_parallel(
        &mut self,
        tasks: Vec<(AgentRole, String)>,
    ) -> Result<Vec<AgentHandle>, AgentPoolError> {
        // Creates handles synchronously in input order. Actual concurrent
        // dispatch happens in BspEngine::execute_tasks_with_timeout via JoinSet.
        // VRAM concurrency limit exposed via max_concurrent().
        let mut handles = Vec::with_capacity(tasks.len());
        for (role, task) in tasks {
            handles.push(self.spawn(role, task)?);
        }
        Ok(handles)
    }

    fn communicate(
        &mut self,
        from: Uuid,
        to: Uuid,
        _message: AgentMessage,
    ) -> Result<(), AgentPoolError> {
        if from == to {
            return Err(AgentPoolError::SpawnFailed(
                "self-communication not allowed".into(),
            ));
        }
        if !self.agents.contains_key(&from) {
            return Err(AgentPoolError::AgentNotFound(from));
        }
        if !self.agents.contains_key(&to) {
            return Err(AgentPoolError::AgentNotFound(to));
        }
        // In-process message delivery wired in 3C.2 lifecycle
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// default_grc_config: per-role GRC budget mapping (reasoning.txt L88-124)
// ---------------------------------------------------------------------------

pub fn default_grc_config(role: &AgentRole) -> AgentGrcConfig {
    match role {
        AgentRole::Executor => AgentGrcConfig {
            role: "executor".into(),
            budget: GrcBudget {
                max_calls: 8,
                max_chars: 1200,
                calls_used: 0,
                chars_used: 0,
            },
            required_steps: vec![
                GrcState::Observe,
                GrcState::Hypothesize,
                GrcState::PlanAction,
                GrcState::Act,
                GrcState::Verify,
            ],
            grounding_level: GroundingLevel::High,
            csq_required: true,
            require_scope_enumeration: true,
            ..AgentGrcConfig::default()
        },
        AgentRole::Auditor => AgentGrcConfig {
            role: "auditor".into(),
            budget: GrcBudget {
                max_calls: 10,
                max_chars: 1200,
                calls_used: 0,
                chars_used: 0,
            },
            required_steps: vec![
                GrcState::Observe,
                GrcState::Hypothesize,
                GrcState::Verify,
                GrcState::Verify,
            ],
            grounding_level: GroundingLevel::Verified,
            csq_required: true,
            require_scope_enumeration: false,
            ..AgentGrcConfig::default()
        },
        AgentRole::Designer => AgentGrcConfig {
            role: "designer".into(),
            budget: GrcBudget {
                max_calls: 8,
                max_chars: 1200,
                calls_used: 0,
                chars_used: 0,
            },
            // Designer: Observe -> PlanAction -> Act -> Verify (no Hypothesize —
            // designer produces PipelineSpec from Research Report, not investigating)
            required_steps: vec![
                GrcState::Observe,
                GrcState::PlanAction,
                GrcState::Act,
                GrcState::Verify,
            ],
            grounding_level: GroundingLevel::High,
            csq_required: true,
            require_scope_enumeration: true,
            ..AgentGrcConfig::default()
        },
        AgentRole::PromptEngineer => AgentGrcConfig {
            role: "prompt_engineer".into(),
            budget: GrcBudget {
                max_calls: 5,
                max_chars: 800,
                calls_used: 0,
                chars_used: 0,
            },
            required_steps: vec![
                GrcState::Observe,
                GrcState::Hypothesize,
                GrcState::PlanAction,
                GrcState::Act,
                GrcState::Verify,
            ],
            grounding_level: GroundingLevel::High,
            csq_required: false,
            require_scope_enumeration: false,
            ..AgentGrcConfig::default()
        },
        AgentRole::MemoryWriter => AgentGrcConfig {
            role: "memory_writer".into(),
            budget: GrcBudget {
                max_calls: 2,
                max_chars: 400,
                calls_used: 0,
                chars_used: 0,
            },
            required_steps: vec![GrcState::Observe, GrcState::Converge],
            grounding_level: GroundingLevel::Uncertain,
            csq_required: false,
            require_scope_enumeration: false,
            ..AgentGrcConfig::default()
        },
        AgentRole::ContextCompiler => AgentGrcConfig {
            role: "context_compiler".into(),
            ..AgentGrcConfig::default()
        },
        AgentRole::Researcher => AgentGrcConfig {
            role: "researcher".into(),
            ..AgentGrcConfig::default()
        },
        AgentRole::MediaValidator => AgentGrcConfig {
            role: "media_validator".into(),
            ..AgentGrcConfig::default()
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::SeededRng;
    use std::collections::HashSet;

    struct MockModelPool {
        vram: u64,
    }

    impl ModelPool for MockModelPool {
        fn available_vram(&self) -> u64 {
            self.vram
        }
    }

    fn test_pool(vram: u64) -> InProcessAgentPool<SeededRng> {
        InProcessAgentPool::new(
            AgentPoolConfig::default(),
            SeededRng::new(42),
            Arc::new(MockModelPool { vram }),
        )
    }

    #[test]
    fn spawn_returns_valid_handle() {
        let mut pool = test_pool(u64::MAX);
        let handle = pool.spawn(AgentRole::Executor, "test task".into()).unwrap();
        assert_eq!(handle.role, AgentRole::Executor);
        assert_eq!(handle.status, AgentStatus::Spawning);
        assert!(handle.active_files.is_empty());
        assert!(!handle.agent_id.is_nil());
    }

    #[test]
    fn spawn_vram_zero_returns_error() {
        let mut pool = test_pool(0);
        let result = pool.spawn(AgentRole::Executor, "test task".into());
        assert!(result.is_err());
        match result.unwrap_err() {
            AgentPoolError::ResourceInsufficient(msg) => {
                assert!(msg.contains("available 0 bytes"));
            }
            other => panic!("expected ResourceInsufficient, got: {other:?}"),
        }
    }

    #[test]
    fn spawn_parallel_empty() {
        let mut pool = test_pool(u64::MAX);
        let handles = pool.spawn_parallel(vec![]).unwrap();
        assert!(handles.is_empty());
    }

    #[test]
    fn spawn_parallel_returns_all_handles() {
        let mut pool = test_pool(u64::MAX);
        let tasks = vec![
            (AgentRole::Executor, "task1".into()),
            (AgentRole::Auditor, "task2".into()),
        ];
        let handles = pool.spawn_parallel(tasks).unwrap();
        assert_eq!(handles.len(), 2);
        assert_ne!(handles[0].agent_id, handles[1].agent_id);
        assert_eq!(handles[0].role, AgentRole::Executor);
        assert_eq!(handles[1].role, AgentRole::Auditor);
    }

    #[test]
    fn spawn_parallel_fail_fast() {
        let mut pool = test_pool(0);
        let tasks = vec![
            (AgentRole::Executor, "task1".into()),
            (AgentRole::Auditor, "task2".into()),
        ];
        let result = pool.spawn_parallel(tasks);
        assert!(result.is_err());
    }

    #[test]
    fn grc_config_executor() {
        let mut pool = test_pool(u64::MAX);
        let handle = pool.spawn(AgentRole::Executor, "task".into()).unwrap();
        assert_eq!(handle.grc_config.budget.max_calls, 8);
        assert_eq!(handle.grc_config.budget.max_chars, 1200);
        assert!(handle.grc_config.csq_required);
        assert!(handle.grc_config.require_scope_enumeration);
    }

    #[test]
    fn grc_config_memory_writer() {
        let mut pool = test_pool(u64::MAX);
        let handle = pool.spawn(AgentRole::MemoryWriter, "task".into()).unwrap();
        assert_eq!(handle.grc_config.budget.max_calls, 2);
        assert_eq!(handle.grc_config.budget.max_chars, 400);
        assert!(!handle.grc_config.csq_required);
        assert!(!handle.grc_config.require_scope_enumeration);
    }

    #[test]
    fn grc_config_designer_distinct_from_executor() {
        let designer = default_grc_config(&AgentRole::Designer);
        let executor = default_grc_config(&AgentRole::Executor);
        assert_eq!(designer.role, "designer");
        assert_eq!(executor.role, "executor");
        assert_ne!(designer.role, executor.role);
        // Designer has fewer required_steps (no Hypothesize)
        assert_ne!(designer.required_steps, executor.required_steps);
        assert_eq!(designer.required_steps.len(), 4); // Observe, PlanAction, Act, Verify
        assert_eq!(executor.required_steps.len(), 5); // Observe, Hypothesize, PlanAction, Act, Verify
        assert!(!designer.required_steps.contains(&GrcState::Hypothesize));
        assert!(executor.required_steps.contains(&GrcState::Hypothesize));
    }

    #[test]
    fn all_roles_serializable() {
        let roles = [
            AgentRole::Executor,
            AgentRole::Auditor,
            AgentRole::Researcher,
            AgentRole::Designer,
            AgentRole::PromptEngineer,
            AgentRole::ContextCompiler,
            AgentRole::MemoryWriter,
            AgentRole::MediaValidator,
        ];
        for role in &roles {
            let json = serde_json::to_string(role).unwrap();
            let deserialized: AgentRole = serde_json::from_str(&json).unwrap();
            assert_eq!(*role, deserialized);
        }
        assert_eq!(roles.len(), 8);
    }

    #[test]
    fn communicate_unknown_agent() {
        let mut pool = test_pool(u64::MAX);
        let unknown_a = Uuid::from_bytes([1; 16]);
        let unknown_b = Uuid::from_bytes([2; 16]);
        let msg = AgentMessage {
            sender: unknown_a,
            recipient: unknown_b,
            payload: "hello".into(),
        };
        let result = pool.communicate(unknown_a, unknown_b, msg);
        match result.unwrap_err() {
            AgentPoolError::AgentNotFound(id) => assert_eq!(id, unknown_a),
            other => panic!("expected AgentNotFound, got: {other:?}"),
        }
    }

    #[test]
    fn communicate_self_rejected() {
        let mut pool = test_pool(u64::MAX);
        let handle = pool.spawn(AgentRole::Executor, "t".into()).unwrap();
        let msg = AgentMessage {
            sender: handle.agent_id,
            recipient: handle.agent_id,
            payload: "self".into(),
        };
        assert!(pool.communicate(handle.agent_id, handle.agent_id, msg).is_err());
    }

    #[test]
    fn communicate_valid_agents() {
        let mut pool = test_pool(u64::MAX);
        let h1 = pool.spawn(AgentRole::Executor, "t1".into()).unwrap();
        let h2 = pool.spawn(AgentRole::Auditor, "t2".into()).unwrap();
        let msg = AgentMessage {
            sender: h1.agent_id,
            recipient: h2.agent_id,
            payload: "check this".into(),
        };
        assert!(pool.communicate(h1.agent_id, h2.agent_id, msg).is_ok());
    }

    #[test]
    fn uuid_via_entropy_produces_valid_v4() {
        let mut pool = test_pool(u64::MAX);
        let handle = pool.spawn(AgentRole::Executor, "task".into()).unwrap();
        let bytes = handle.agent_id.as_bytes();
        // RFC 4122 version 4: byte 6 high nibble = 0x4_
        assert_eq!(bytes[6] >> 4, 4);
        // RFC 4122 variant 1: byte 8 high 2 bits = 10
        assert_eq!(bytes[8] >> 6, 2);
    }

    #[test]
    fn consecutive_spawns_different_uuids() {
        let mut pool = test_pool(u64::MAX);
        let mut ids = HashSet::new();
        for i in 0..10 {
            let handle = pool
                .spawn(AgentRole::Executor, format!("task_{i}"))
                .unwrap();
            assert!(ids.insert(handle.agent_id), "duplicate UUID at spawn {i}");
        }
        assert_eq!(ids.len(), 10);
    }

    #[test]
    fn agent_count_tracks_spawns() {
        let mut pool = test_pool(u64::MAX);
        assert_eq!(pool.agent_count(), 0);
        pool.spawn(AgentRole::Executor, "t".into()).unwrap();
        assert_eq!(pool.agent_count(), 1);
        pool.spawn(AgentRole::Auditor, "t".into()).unwrap();
        assert_eq!(pool.agent_count(), 2);
    }

    #[test]
    fn get_agent_returns_handle() {
        let mut pool = test_pool(u64::MAX);
        let handle = pool.spawn(AgentRole::Executor, "t".into()).unwrap();
        let retrieved = pool.get_agent(&handle.agent_id).unwrap();
        assert_eq!(retrieved.agent_id, handle.agent_id);
        assert_eq!(retrieved.role, AgentRole::Executor);
    }

    #[test]
    fn model_pool_max_vram_allows_spawn() {
        let mut pool = test_pool(u64::MAX);
        assert!(pool.spawn(AgentRole::Executor, "t".into()).is_ok());
    }

    #[test]
    fn model_pool_exact_threshold_allows_spawn() {
        let mut pool = test_pool(VRAM_MIN_BYTES);
        assert!(pool.spawn(AgentRole::Executor, "t".into()).is_ok());
    }

    #[test]
    fn model_pool_below_threshold_rejects() {
        let mut pool = test_pool(VRAM_MIN_BYTES - 1);
        assert!(pool.spawn(AgentRole::Executor, "t".into()).is_err());
    }

    // -- 3D.5: VRAM-aware max_concurrent tests --

    #[test]
    fn max_concurrent_floor_division() {
        // 2 GiB available, 1 GiB per agent → max_concurrent = 2
        let available = 2 * 1024 * 1024 * 1024_u64;
        let per_agent = 1024 * 1024 * 1024_u64;
        let pool = InProcessAgentPool::new(
            AgentPoolConfig {
                vram_required_bytes: per_agent,
                ..AgentPoolConfig::default()
            },
            SeededRng::new(42),
            Arc::new(MockModelPool { vram: available }),
        );
        assert_eq!(pool.max_concurrent(), 2);
    }

    #[test]
    fn max_concurrent_vram_zero_fallback() {
        // available_vram = 0 → fallback to 1
        let pool = InProcessAgentPool::new(
            AgentPoolConfig::default(),
            SeededRng::new(42),
            Arc::new(MockModelPool { vram: 0 }),
        );
        assert_eq!(pool.max_concurrent(), 1);
    }

    #[test]
    fn max_concurrent_per_agent_zero_guard() {
        // per_agent_vram = 0 → no division by zero, fallback to 1
        let pool = InProcessAgentPool::new(
            AgentPoolConfig {
                vram_required_bytes: 0,
                ..AgentPoolConfig::default()
            },
            SeededRng::new(42),
            Arc::new(MockModelPool { vram: u64::MAX }),
        );
        assert_eq!(pool.max_concurrent(), 1);
    }

    #[test]
    fn all_statuses_serializable() {
        let statuses = [
            AgentStatus::Spawning,
            AgentStatus::Ready,
            AgentStatus::Running,
            AgentStatus::Checkpointing,
            AgentStatus::Finished,
            AgentStatus::Failed,
        ];
        for status in &statuses {
            let json = serde_json::to_string(status).unwrap();
            let deserialized: AgentStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(*status, deserialized);
        }
        assert_eq!(statuses.len(), 6);
    }
}
