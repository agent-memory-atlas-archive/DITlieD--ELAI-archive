use rustc_hash::FxHashMap;
use std::path::PathBuf;
use std::process::Child;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use elai_core::EntropySource;
use elai_reasoning::{
    agent_grc_config_for_role, AgentGrcConfig, BasicGrcEngine, GrcBudget, GrcEngine, GrcState,
    GroundingLevel,
};

use crate::sandbox::{AgentSandbox, AppContainerConfig, SandboxHandle};

// Re-export AgentRole from the shared IPC types crate.
// This is the canonical definition used by both orchestrator and agent binary.
pub use elai_ipc_types::AgentRole;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum VRAM (bytes) required to spawn an agent with a loaded model.
/// Below this threshold, spawn() returns ResourceInsufficient.
pub const VRAM_MIN_BYTES: u64 = 512 * 1024 * 1024; // 512 MiB

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
    /// OS child process handle for out-of-process agents (Phase 4A).
    /// None for in-process agents (Phase 3 Tier A).
    /// Not cloned — only the original handle can kill/wait the process.
    pub child_process: Option<Child>,
    /// IPC socket path for this agent (Phase 4A).
    /// Used by the orchestrator to communicate with the agent process.
    pub socket_path: Option<PathBuf>,
    /// Per-agent AppContainer profile identifier (B-18 Phase 2).
    /// Format: "elai-agent-{agent_id}". None when sandbox is disabled.
    /// Phase-2 note: MVP uses a single shared profile; per-agent profile
    /// isolation (unique SID per agent) is deferred to B-18C.
    pub appcontainer_profile_id: Option<String>,
    /// Per-agent filesystem allow-list (B-18 Phase 3).
    /// Glob patterns that are explicitly allowed for this agent regardless of
    /// global HIDDEN_PATH_PATTERNS. Visibility filter checks this list first;
    /// if any pattern matches the path is NOT filtered.
    /// Empty = no extra allowances (applies global hidden-path rules).
    pub allowed_paths: Vec<String>,
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
            child_process: None, // Not cloned — only original handle owns the process
            socket_path: self.socket_path.clone(),
            appcontainer_profile_id: self.appcontainer_profile_id.clone(),
            allowed_paths: self.allowed_paths.clone(),
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
            .field("has_child_process", &self.child_process.is_some())
            .field("socket_path", &self.socket_path)
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
            // B-18A Task 7: enable AppContainer-based isolation on Windows by
            // default. Linux/macOS stay false until B-18B+ wires clone3+seccomp
            // through launch_agent_process. The factory in
            // create_sandbox() falls back to a non-isolating Job+RestrictedToken
            // path on hosts where AppContainer is unavailable, so this default
            // is safe even on older Windows.
            enable_sandbox: cfg!(target_os = "windows"),
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

    /// B-22 Phase 4G: snake-case role string for an agent identified by
    /// `agent_id`. Returns `None` when the UUID is unknown.
    ///
    /// The dashboard uses this to surface per-agent role badges in the
    /// supervision feed and the eventual /agents inspector. The mapping is
    /// the same canonical snake_case used in `AgentRole` SSB serialisation.
    fn get_role(&self, agent_id: Uuid) -> Option<String>;
}

// ---------------------------------------------------------------------------
// InProcessAgentPool: in-process implementation (Phase 3 — Tier A only)
// ---------------------------------------------------------------------------

pub struct InProcessAgentPool<E: EntropySource> {
    agents: FxHashMap<Uuid, AgentHandle>,
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
                    tracing::warn!("Failed to create sandbox: {}, proceeding without isolation", e);
                    None
                }
            }
        } else {
            None
        };

        Self {
            agents: FxHashMap::default(),
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

    pub fn get_agent_mut(&mut self, id: &Uuid) -> Option<&mut AgentHandle> {
        self.agents.get_mut(id)
    }

    /// Launch an agent as an OS process (Phase 4A).
    ///
    /// Creates scratch directory, IPC socket, spawns the agent binary,
    /// and updates the AgentHandle with child process and socket path.
    ///
    /// Requires: agent_binary_path points to the elai_cc_agent binary.
    /// The socket_dir must exist with 0700 permissions (created by IpcGateway).
    ///
    /// B-18A Task 8: when `enable_sandbox=true` AND a sandbox + sandbox_handle
    /// were successfully created at spawn() time, the binary is launched via
    /// `AgentSandbox::launch_agent` so that AppContainer / Job-object /
    /// restricted-token isolation is actually applied. The sandbox tracks the
    /// process internally; agent_pool stores no `Child` in that branch and
    /// kill_agent_process delegates to `AgentSandbox::kill_agent`. Falls back
    /// to plain `Command::spawn` when the sandbox is unavailable so older
    /// Windows hosts (and Linux until B-18B) keep working.
    pub fn launch_agent_process(
        &mut self,
        agent_id: &Uuid,
        agent_binary_path: &std::path::Path,
        socket_path: &std::path::Path,
        session_credentials: &std::collections::BTreeMap<String, String>,
    ) -> Result<(), AgentPoolError> {
        let handle = self.agents.get_mut(agent_id).ok_or_else(|| {
            AgentPoolError::AgentNotFound(*agent_id)
        })?;

        // Create scratch directory
        std::fs::create_dir_all(&handle.scratch_dir).map_err(|e| {
            AgentPoolError::SpawnFailed(format!("scratch dir creation failed: {e}"))
        })?;

        // B-18A Task 8: if a sandbox + sandbox_handle exist, launch through the
        // sandbox so AppContainer / Job / restricted-token actually apply.
        let scratch_dir = handle.scratch_dir.clone();
        let sandbox_owned = if let (Some(sandbox), Some(sb_handle)) = (
            self.sandbox.as_ref(),
            handle.sandbox_handle.as_ref(),
        ) {
            let command = agent_binary_path.to_string_lossy().into_owned();
            let args = vec![
                socket_path.to_string_lossy().into_owned(),
                agent_id.to_string(),
            ];
            match sandbox.launch_agent(sb_handle, &command, &args, &scratch_dir) {
                Ok(proc) => {
                    tracing::info!(
                        agent_id = %agent_id,
                        pid = proc.process_id,
                        socket_path = %socket_path.display(),
                        "launched sandboxed agent process via AgentSandbox::launch_agent"
                    );
                    true
                }
                Err(e) => {
                    tracing::warn!(
                        agent_id = %agent_id,
                        error = %e,
                        "sandbox launch_agent failed, falling back to plain Command::spawn"
                    );
                    false
                }
            }
        } else {
            false
        };

        if !sandbox_owned {
            // Fallback path (Linux until B-18B, or Windows when sandbox creation
            // failed): plain Command::spawn with sanitized env.
            let env = crate::sandbox::AgentEnvironment::new(scratch_dir);
            let mut cmd = std::process::Command::new(agent_binary_path);
            cmd.arg(socket_path.to_string_lossy().as_ref())
                .arg(agent_id.to_string())
                .env_clear()
                .envs(env.as_env_map());
            // C2: inject session credentials as additional env vars. Keys are
            // already sanitized to [A-Z0-9_]+ by sanitize_credential_id in
            // task_runner.rs. Credentials overlay the base AgentEnvironment;
            // on collision credentials win (sanitized ids are unique by design).
            // Values are NEVER logged — only key names and value lengths.
            for (key, value) in session_credentials {
                tracing::debug!(
                    agent_id = %agent_id,
                    credential_key = %key,
                    value_len = value.len(),
                    "agent spawn: injecting session credential into env"
                );
                cmd.env(key, value);
            }
            let child = cmd
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| {
                    AgentPoolError::SpawnFailed(format!("process spawn failed: {e}"))
                })?;
            tracing::info!(
                agent_id = %agent_id,
                pid = child.id(),
                socket_path = %socket_path.display(),
                "launched agent process (no sandbox)"
            );
            handle.child_process = Some(child);
        }
        // When sandbox owns the process, child_process stays None — the
        // sandbox's per-agent process map is the authoritative tracker, and
        // kill_agent_process / wait below routes through it.

        handle.socket_path = Some(socket_path.to_path_buf());
        handle.status = AgentStatus::Ready;

        Ok(())
    }

    /// Kill an agent's OS process and clean up resources.
    ///
    /// B-18A Task 8: routes the kill through `AgentSandbox::kill_agent` /
    /// `wait_agent` when the sandbox owns the process; falls back to direct
    /// `Child::kill` when launch_agent_process took the no-sandbox branch.
    pub fn kill_agent_process(&mut self, agent_id: &Uuid) -> Result<(), AgentPoolError> {
        let handle = self.agents.get_mut(agent_id).ok_or_else(|| {
            AgentPoolError::AgentNotFound(*agent_id)
        })?;

        if let Some(ref mut child) = handle.child_process {
            if let Err(e) = child.kill() {
                tracing::warn!(
                    agent_id = %agent_id,
                    error = %e,
                    "failed to kill agent process"
                );
            }
            // Reap to avoid zombie
            let _ = child.wait();
        } else if let Some(sandbox) = self.sandbox.as_ref() {
            // Sandbox-owned process: delegate kill+reap to the sandbox.
            if let Err(e) = sandbox.kill_agent(*agent_id) {
                tracing::warn!(
                    agent_id = %agent_id,
                    error = %e,
                    "AgentSandbox::kill_agent failed"
                );
            }
            // wait_agent reaps and releases the per-agent kernel handle / pid
            // entry. Errors are logged and swallowed so cleanup proceeds.
            if let Err(e) = sandbox.wait_agent(*agent_id) {
                tracing::warn!(
                    agent_id = %agent_id,
                    error = %e,
                    "AgentSandbox::wait_agent failed"
                );
            }
        }

        // Clean up socket file
        if let Some(ref path) = handle.socket_path {
            let _ = std::fs::remove_file(path);
        }

        // Clean up scratch directory
        let _ = std::fs::remove_dir_all(&handle.scratch_dir);

        handle.status = AgentStatus::Failed;
        handle.child_process = None;

        Ok(())
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

        // B-8 Tasks#6: anti-rubber-stamp activation for the Auditor role.
        //
        // The reasoning engine enforces "Auditor must record >= 1 FINDING:
        // Verify step before Converge" at engine.rs:297-315. The enforcement
        // is correct as-is (any auditor that calls `Converge` without a prior
        // `Verify` whose content starts with `FINDING:` is rejected with
        // `GrcError::InsufficientFindings`); the unit-test suite already
        // covers the rejection path (engine.rs tests).
        //
        // What was missing pre-B-8 was a production caller of
        // `engine.think()` on the auditor's engine — without one, the engine
        // is dormant and the gate is unreachable. We seed one Observe step
        // here. Observe is the legal first step in the state machine
        // (engine.rs:271-282) and consumes one budget slot from the auditor's
        // 10-call budget, leaving the real audit flow nine slots to record
        // its own Observe/Hypothesize/PlanAction/Act/Verify(FINDING:...)
        // chain before reaching Converge. With this seed in place, the
        // engine has live topic state and any subsequent Converge call
        // exercises the anti-rubber-stamp branch — turning the dormant gate
        // into a reachable production guard.
        //
        // The kill-switch (`ELAI_DISABLE_PIPELINE_GRC`) skips the seed so the
        // engine reverts to the unenforced baseline.
        //
        // Seed is restricted to Auditor because engine.rs:300 gates the
        // FINDING check on `role == "auditor"`; seeding non-auditor engines
        // would burn budget without a corresponding production benefit.
        // B-41: anti-rubber-stamp seed activates for every auditor variant
        // (Plan-mode + 5 per-level). engine.rs FINDING gate is keyed on the
        // role string ("auditor*" prefix); seeding all auditor variants keeps
        // the GRC budget initialised consistently across tiers.
        if matches!(
            role,
            AgentRole::Auditor
                | AgentRole::AuditorConcepts
                | AgentRole::AuditorPhases
                | AgentRole::AuditorSteps
                | AgentRole::AuditorBabySteps
                | AgentRole::AuditorBabyStepDetail
        ) && !crate::grc::pipeline_grc_disabled_via_env()
        {
            let seed = elai_reasoning::ThinkStep {
                step_type: GrcState::Observe,
                content: "auditor engine bootstrapped via agent_pool::spawn".into(),
                topic: format!("audit-{}", agent_id),
                grounding: Some("tool:agent_pool::spawn".into()),
                conclusion: None,
                scope: None,
            };
            match grc_engine.think(seed) {
                Ok(_) => tracing::debug!(
                    agent_id = %agent_id,
                    "agent_pool: seeded auditor Observe for anti-rubber-stamp gate activation"
                ),
                Err(e) => tracing::warn!(
                    agent_id = %agent_id,
                    error = ?e,
                    "agent_pool: auditor seed rejected; engine continues without seed"
                ),
            }
        }

        // B-8 Tasks#2: ContextCompiler verify_claim pre-fetch — out of scope.
        // The orchestrator-side ContextCompiler is currently a spawn stub
        // with no Rust prefetch path; the real ContextCompiler is the
        // Claude-Code sub-agent driven from CLAUDE.md session-protocol.
        // Wiring `verify_claim` into the sub-agent flow is a workflow-tooling
        // change tracked separately. See workflow/reasoning/B-8-decisions.md
        // §6.
        if matches!(role, AgentRole::ContextCompiler) {
            tracing::debug!(
                agent_id = %agent_id,
                "agent_pool: ContextCompiler verify_claim wiring deferred to B-future"
            );
        }

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
                    tracing::warn!("Failed to create sandbox for agent {}: {}, proceeding without isolation", agent_id, e);
                    None
                }
            }
        } else {
            None
        };

        // B-18 P2: derive per-agent AppContainer profile id from agent_id.
        // MVP: single shared profile in create_sandbox(); per-agent SID isolation is B-18C.
        let appcontainer_profile_id = if self.config.enable_sandbox {
            Some(format!("elai-agent-{}", agent_id))
        } else {
            None
        };

        // B-18 P3: per-agent allowed_paths from role-based defaults.
        // Executor: project source broad read; Auditor: read-only everything;
        // ContextCompiler: read-only except memory-bank.
        let allowed_paths = role_default_allowed_paths(&role, &self.config.project_path);

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
            child_process: None, // Set by launch_agent_process() after spawn
            socket_path: None,   // Set by launch_agent_process() after spawn
            appcontainer_profile_id,
            allowed_paths,
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

    fn get_role(&self, agent_id: Uuid) -> Option<String> {
        self.agents
            .get(&agent_id)
            .map(|h| agent_role_to_snake_case(h.role).to_owned())
    }
}

/// B-22 Phase 4G: canonical snake_case for an [`AgentRole`].
/// Mirrors `bsp::engine::agent_role_str` so SSB rows and the dashboard
/// see the same labels (kept inline rather than re-exported to avoid a
/// cross-module dep on the bsp submodule from agent_pool.rs).
fn agent_role_to_snake_case(role: AgentRole) -> &'static str {
    match role {
        AgentRole::Executor => "executor",
        AgentRole::Auditor => "auditor",
        AgentRole::Researcher => "researcher",
        AgentRole::Designer => "designer",
        AgentRole::PromptEngineer => "prompt_engineer",
        AgentRole::ContextCompiler => "context_compiler",
        AgentRole::MemoryWriter => "memory_writer",
        AgentRole::MediaValidator => "media_validator",
        // B-40: FRAMING substate agents (pre-decomposition).
        AgentRole::IdeaFramer => "idea_framer",
        AgentRole::IdeaCritic => "idea_critic",
        // B-41: per-level auditors (Plan-mode stays on AgentRole::Auditor).
        AgentRole::AuditorConcepts => "auditor_concepts",
        AgentRole::AuditorPhases => "auditor_phases",
        AgentRole::AuditorSteps => "auditor_steps",
        AgentRole::AuditorBabySteps => "auditor_babysteps",
        AgentRole::AuditorBabyStepDetail => "auditor_babystep_detail",
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
        // B-5: delegate to reasoning crate's canonical config table so this
        // arm stays in sync with TomlConfigLoader without duplicating the spec.
        // agent_grc_config_for_role returns Some("context_compiler") by construction;
        // the unwrap_or_else fallback is unreachable in practice but satisfies
        // the no-.expect()-in-production rule.
        AgentRole::ContextCompiler => agent_grc_config_for_role("context_compiler")
            .unwrap_or_else(|| AgentGrcConfig {
                role: "context_compiler".into(),
                ..AgentGrcConfig::default()
            }),
        AgentRole::Researcher => AgentGrcConfig {
            role: "researcher".into(),
            budget: GrcBudget {
                max_calls: 10,
                max_chars: 1200,
                calls_used: 0,
                chars_used: 0,
            },
            required_steps: vec![
                GrcState::Observe,
                GrcState::Hypothesize,
                GrcState::PlanAction,
                GrcState::Verify,
                GrcState::Converge,
            ],
            grounding_level: GroundingLevel::High,
            csq_required: true,
            require_scope_enumeration: false,
            ..AgentGrcConfig::default()
        },
        // B-5: delegate to reasoning crate's canonical config table.
        // agent_grc_config_for_role returns Some("media_validator") by construction;
        // the unwrap_or_else fallback is unreachable in practice but satisfies
        // the no-.expect()-in-production rule.
        AgentRole::MediaValidator => agent_grc_config_for_role("media_validator")
            .unwrap_or_else(|| AgentGrcConfig {
                role: "media_validator".into(),
                ..AgentGrcConfig::default()
            }),
        // B-40: FRAMING substate agents. Both delegate to reasoning::config
        // (explicit arms added there — see reasoning/src/config.rs:336).
        // Designer-shape budget 5/800 with Observe->PlanAction->Act->Verify
        // (idea_framer) or Auditor-shape Observe->Verify->Verify (idea_critic).
        // The unwrap_or_else fallback satisfies no-.expect()-in-production but
        // is unreachable: the reasoning table has explicit arms (B-5 rule (a)
        // requires non-empty required_steps; relying on `_=>None` fallback
        // here would yield AgentGrcConfig::default() with required_steps=[]).
        AgentRole::IdeaFramer => agent_grc_config_for_role("idea_framer")
            .unwrap_or_else(|| AgentGrcConfig {
                role: "idea_framer".into(),
                ..AgentGrcConfig::default()
            }),
        AgentRole::IdeaCritic => agent_grc_config_for_role("idea_critic")
            .unwrap_or_else(|| AgentGrcConfig {
                role: "idea_critic".into(),
                ..AgentGrcConfig::default()
            }),
        // B-41: per-level auditor budgets. All delegate to reasoning::config
        // (explicit arms added there). Tiered budgets per plan §3: Concepts
        // 6/1000 semantic-heavy, Phases 5/800, Steps 4/700, BabySteps 4/700,
        // BabyStepDetail 3/600 line-level fast.
        AgentRole::AuditorConcepts => agent_grc_config_for_role("auditor_concepts")
            .unwrap_or_else(|| AgentGrcConfig {
                role: "auditor_concepts".into(),
                ..AgentGrcConfig::default()
            }),
        AgentRole::AuditorPhases => agent_grc_config_for_role("auditor_phases")
            .unwrap_or_else(|| AgentGrcConfig {
                role: "auditor_phases".into(),
                ..AgentGrcConfig::default()
            }),
        AgentRole::AuditorSteps => agent_grc_config_for_role("auditor_steps")
            .unwrap_or_else(|| AgentGrcConfig {
                role: "auditor_steps".into(),
                ..AgentGrcConfig::default()
            }),
        AgentRole::AuditorBabySteps => agent_grc_config_for_role("auditor_babysteps")
            .unwrap_or_else(|| AgentGrcConfig {
                role: "auditor_babysteps".into(),
                ..AgentGrcConfig::default()
            }),
        AgentRole::AuditorBabyStepDetail => {
            agent_grc_config_for_role("auditor_babystep_detail").unwrap_or_else(|| {
                AgentGrcConfig {
                    role: "auditor_babystep_detail".into(),
                    ..AgentGrcConfig::default()
                }
            })
        }
    }
}

// ---------------------------------------------------------------------------
// role_default_allowed_paths: per-role filesystem allow-list (B-18 P3)
// ---------------------------------------------------------------------------

/// Returns glob patterns explicitly allowed for the given role.
/// These patterns are stored on AgentHandle.allowed_paths and consulted by
/// the visibility filter before falling through to global HIDDEN_PATH_PATTERNS.
///
/// Role policy (incremental — each level adds to executor base):
///   Executor:        project/** broad read/write (normal ELAI_CC workflow)
///   Auditor:         everything readable (needs full project visibility for audit)
///   ContextCompiler: project/** read-only; memory-bank excluded (context delegation rule)
///   Researcher:      project/** + system temp for download staging
///   Others:          executor default
pub fn role_default_allowed_paths(role: &AgentRole, project_path: &std::path::Path) -> Vec<String> {
    let project_glob = format!("{}/**", project_path.display());
    match role {
        // B-41: all auditor variants (Plan-mode + 5 per-level) share the same
        // broad-read scope — audit needs full project visibility regardless of tier.
        AgentRole::Auditor
        | AgentRole::AuditorConcepts
        | AgentRole::AuditorPhases
        | AgentRole::AuditorSteps
        | AgentRole::AuditorBabySteps
        | AgentRole::AuditorBabyStepDetail => {
            // Auditor needs read access to everything in the project
            vec![project_glob, "**".to_string()]
        }
        AgentRole::ContextCompiler => {
            // Context compiler reads project but not memory-bank (context delegation rule)
            let mem_bank_glob = format!("{}/workflow/memory-bank/**", project_path.display());
            // Allowed = project glob; memory-bank is a sub-path but the visibility
            // filter applies HIDDEN_PATH_PATTERNS AFTER the allow check, so we
            // only add the base project allow and rely on hidden-path rules for
            // sensitive sub-paths.
            vec![project_glob, format!("!{}", mem_bank_glob)]
        }
        AgentRole::Researcher => {
            // Researcher gets project access + temp for download staging
            #[cfg(target_os = "windows")]
            let tmp = std::env::var("TEMP")
                .unwrap_or_else(|_| "C:\\Windows\\Temp".to_string());
            #[cfg(not(target_os = "windows"))]
            let tmp = "/tmp".to_string();
            vec![project_glob, format!("{}/**", tmp)]
        }
        // Executor, Designer, PromptEngineer, MemoryWriter, MediaValidator
        _ => vec![project_glob],
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
            // B-40: FRAMING substate agents.
            AgentRole::IdeaFramer,
            AgentRole::IdeaCritic,
        ];
        for role in &roles {
            let json = serde_json::to_string(role).unwrap();
            let deserialized: AgentRole = serde_json::from_str(&json).unwrap();
            assert_eq!(*role, deserialized);
        }
        assert_eq!(roles.len(), 10);
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

    // B-22 Phase 4G: AgentPool::get_role exposes the snake-case role label
    // used by the dashboard's per-agent inspector.
    #[test]
    fn get_role_returns_snake_case_for_spawned_agent() {
        let mut pool = test_pool(u64::MAX);
        let handle = pool.spawn(AgentRole::Executor, "t".into()).unwrap();
        let role = pool.get_role(handle.agent_id).expect("role present");
        assert_eq!(role, "executor");

        let h2 = pool.spawn(AgentRole::ContextCompiler, "t2".into()).unwrap();
        let role2 = pool.get_role(h2.agent_id).expect("role present");
        assert_eq!(role2, "context_compiler");
    }

    #[test]
    fn get_role_returns_none_for_unknown_uuid() {
        let pool = test_pool(u64::MAX);
        assert!(pool.get_role(Uuid::nil()).is_none());
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
