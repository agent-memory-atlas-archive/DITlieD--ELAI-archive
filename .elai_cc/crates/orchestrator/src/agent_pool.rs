use rustc_hash::FxHashMap;
use std::path::PathBuf;
use std::process::Child;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use elai_core::EntropySource;
use elai_reasoning::{
    agent_grc_config_for_role, AgentGrcConfig, AgentGrcConfigLoader, AuditorBudgetReserve,
    BasicGrcEngine, GrcBudget, GrcEngine, GrcState, GroundingLevel, TomlConfigLoader,
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

    /// C-9 — Whether the pool can fit a model of the given size in bytes.
    /// Default impl compares against [`available_vram`]; concrete pools can
    /// override when they have richer accounting (e.g. fragmented VRAM
    /// reservation, per-device residency).
    ///
    /// Consumed by [`crate::autonomous_gate::AutonomousGate::decide`] before
    /// emitting [`crate::autonomous_gate::AutonomyDecision::SubstituteModel`]
    /// so a fallback that won't fit fires
    /// [`crate::autonomous_gate::EscalationTrigger::SubstituteModelFailed`]
    /// instead of an unconstrained swap. Source: phase-4G.txt §4G.5.
    fn can_fit_llm(&self, model_size_bytes: u64) -> bool {
        self.available_vram() >= model_size_bytes
    }
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

/// Env override for the agent sandbox default. When set to a truthy value
/// (`1`/`true`/`on`), `enable_sandbox` is forced ON regardless of platform;
/// when set to a falsy value (`0`/`false`/`off`), it is forced OFF. When the
/// var is UNSET the platform default (`true`) holds on every host. B-18B
/// promoted the Linux clone3+seccomp path to a platform default; the env
/// override remains the opt-out seam (`ELAI_SANDBOX_ENABLE=0`). On Linux hosts
/// without CAP_SYS_ADMIN, `LinuxSandbox::launch_agent` returns
/// `SandboxError::ProcessLaunchFailed` (EPERM) rather than silently falling
/// back to a plain `Command::spawn` — callers must propagate the error.
pub const ELAI_SANDBOX_ENABLE_ENV: &str = "ELAI_SANDBOX_ENABLE";

/// Resolve the sandbox default: explicit env override wins, else `true`.
/// B-18B: Linux default flipped ON (clone3+seccomp path activated). The env
/// override `ELAI_SANDBOX_ENABLE=0` remains the supported opt-out for hosts
/// where the unprivileged clone3 namespaces are unavailable.
pub(crate) fn sandbox_default_enabled() -> bool {
    match std::env::var(ELAI_SANDBOX_ENABLE_ENV) {
        Ok(v) => match v.trim() {
            "1" | "true" | "TRUE" | "on" | "ON" => true,
            "0" | "false" | "FALSE" | "off" | "OFF" => false,
            // Unrecognised value: ignore the override, keep default ON.
            _ => true,
        },
        Err(_) => true,
    }
}

impl Default for AgentPoolConfig {
    fn default() -> Self {
        Self {
            scratch_base: PathBuf::from(".elai_cc/sessions/default/agents"),
            vram_required_bytes: VRAM_MIN_BYTES,
            project_path: PathBuf::from("."), // Default to current directory
            // B-18A/B-18B: sandbox isolation is ON by default on every host.
            // Windows uses AppContainer (with a non-isolating Job+RestrictedToken
            // fallback when AppContainer is unavailable); Linux uses
            // clone3+seccomp namespaces (requires CAP_SYS_ADMIN or a userns
            // outer container; EPERM propagates as SandboxError, no silent
            // fallback). ELAI_SANDBOX_ENABLE=0 is the supported opt-out.
            enable_sandbox: sandbox_default_enabled(),
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

    /// H-42: spawn with an explicit decomposition level so the GRC budget is
    /// scaled to the real serial depth of the sub-task.  Default implementation
    /// delegates to `spawn` (d=0, no scaling) for backward compatibility.
    /// Override in `InProcessAgentPool` to apply the real depth factor.
    fn spawn_with_level(
        &mut self,
        role: AgentRole,
        task: String,
        level: crate::pipeline_tree_navigator::DecompositionLevel,
    ) -> Result<AgentHandle, AgentPoolError> {
        let _ = level; // base impl ignores level; InProcessAgentPool overrides
        self.spawn(role, task)
    }

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
    /// J-70 E2a: auditor financial-independence budget reserve. The
    /// orchestrator cannot raid this pool for executor work.
    pub auditor_budget_reserve: AuditorBudgetReserve,
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
            auditor_budget_reserve: AuditorBudgetReserve::default(),
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

    /// J-70 E2a: attempt to reallocate budget from the auditor reserve to
    /// executor work. This ALWAYS fails — the auditor budget is a HARD
    /// reservation the orchestrator cannot raid (NASA-STD financial
    /// independence).
    pub fn try_reallocate_from_auditor_reserve(
        &self,
        calls: u32,
        chars: u32,
    ) -> Result<(), &str> {
        self.auditor_budget_reserve.refuse_raid(calls, chars)
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

impl<E: EntropySource> InProcessAgentPool<E> {
    /// Shared implementation for `spawn` and `spawn_with_level`.
    ///
    /// H-42: `level` controls the GRC budget scale factor (1 + 0.25*d).
    /// - `spawn()` passes `DecompositionLevel::Concept` (d=0, no scaling, flat base).
    /// - `spawn_with_level()` passes the real level; BabyStep (d=3) yields 1.75x budget.
    ///
    /// All logic that was previously duplicated between the two public entry points
    /// lives here. Kill-switch: `ELAI_REASONING_BUDGET_SCALE_DISABLE=1` in the env
    /// makes `scale_grc_budget` return the base unchanged for any depth.
    fn spawn_inner(
        &mut self,
        role: AgentRole,
        task: String,
        level: crate::pipeline_tree_navigator::DecompositionLevel,
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

        let base_config = default_grc_config(&role);
        let depth = crate::reasoning_budget::estimate_serial_depth(level);
        let grc_config = crate::reasoning_budget::scale_grc_budget(base_config.clone(), depth);

        // Emit tracing::info when scaling actually changed the budget (non-zero depth,
        // kill-switch off). The SSB row is emitted by spawn_with_level callers that
        // own an Arc<Ssb>; this tracing row is the secondary observable artifact.
        if grc_config.budget.max_calls != base_config.budget.max_calls {
            let ceiling = base_config
                .budget
                .max_calls
                .saturating_mul(crate::reasoning_budget::LOCKED_BUDGET_CEILING);
            tracing::info!(
                target: "orchestrator::reasoning_budget",
                role = %grc_config.role,
                base_calls = base_config.budget.max_calls,
                serial_depth = depth,
                scaled_calls = grc_config.budget.max_calls,
                ceiling = ceiling,
                kind = "depth_bound",
                action_type = "reasoning_budget_scaled",
                "H-42: ReasoningBudgetScaled — depth-scaled GRC budget at dispatch"
            );
        }

        let scratch_dir = self.config.scratch_base.join(agent_id.to_string());

        // Instantiate a fresh reasoning engine and apply the role-specific GRC config.
        // with_agent_config resets all topic state, so each spawned agent starts clean.
        let mut grc_engine = BasicGrcEngine::new();
        grc_engine.with_agent_config(grc_config.clone());

        // C-37 — closes GAP-B (default_decoding_for had no production
        // callers): emit a tracing::info row at the role-dispatch site so the
        // per-role decoding policy table actually influences run config.
        //
        // We can NOT call SidecarProvider::apply_role_default_decoding here
        // because the orchestrator does not own the SidecarProvider — the
        // spawned child process owns it. The child reads its role from the
        // IPC AgentRequest frame and is the natural site to apply the
        // policy; this log makes the dispatch event observable so audit
        // tooling can correlate the child's policy application with the
        // parent's spawn decision.
        //
        // Paper grounding: arXiv:2604.06066 Section 4.3. See
        // providers/src/decoding.rs:PER_ROLE_DECODING_PARAMS_PAPER.
        let role_label = agent_role_to_snake_case(role);
        if let Some(decoding_role) = elai_providers::decoding::RoleDecoding::from_snake_case(role_label) {
            let cfg = elai_providers::decoding::default_decoding_for(decoding_role);
            tracing::info!(
                target: "orchestrator::agent_pool",
                agent_id = %agent_id,
                role = role_label,
                decoding_mode = cfg.mode.discriminant(),
                max_retries = cfg.max_retries_on_invalid,
                strict = cfg.strict,
                "C-37: per-role default decoding policy resolved at dispatch"
            );
        } else {
            tracing::debug!(
                target: "orchestrator::agent_pool",
                agent_id = %agent_id,
                role = role_label,
                "C-37: no decoding policy mapping for role; keeping legacy un-constrained behaviour"
            );
        }

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
                | AgentRole::AuditorProductionGrade
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
                Ok(_) => {
                    tracing::debug!(
                        agent_id = %agent_id,
                        "agent_pool: seeded auditor Observe for anti-rubber-stamp gate activation"
                    );
                    if let Err(e) = self.auditor_budget_reserve.charge(1, 0) {
                        tracing::warn!(
                            agent_id = %agent_id,
                            error = e,
                            "agent_pool: auditor budget reserve charge failed for Observe seed"
                        );
                    }
                }
                Err(e) => tracing::warn!(
                    agent_id = %agent_id,
                    error = ?e,
                    "agent_pool: auditor seed rejected; engine continues without seed"
                ),
            }
        }

        // J-70 E2a financial independence: could this executor draw from the
        // auditor reserve? The only transfer API must refuse a real non-zero
        // raid amount. Skip auditor roles (they own the reserve). Fail closed
        // if a future edit makes refuse_raid succeed on a positive amount.
        if !matches!(
            role,
            AgentRole::Auditor
                | AgentRole::AuditorConcepts
                | AgentRole::AuditorPhases
                | AgentRole::AuditorSteps
                | AgentRole::AuditorBabySteps
                | AgentRole::AuditorBabyStepDetail
                | AgentRole::AuditorProductionGrade
        ) {
            match self.try_reallocate_from_auditor_reserve(
                grc_config.budget.max_calls.max(1),
                grc_config.budget.max_chars.max(1),
            ) {
                Ok(()) => {
                    tracing::error!(
                        target: "orchestrator::j70_financial_independence",
                        "INVARIANT BROKEN: auditor reserve accepted a raid; refusing spawn funding from reserve"
                    );
                }
                Err(msg) => {
                    tracing::debug!(
                        target: "orchestrator::j70_financial_independence",
                        error = msg,
                        executor_calls = grc_config.budget.max_calls,
                        "executor funded from own GRC budget; auditor reserve untouched"
                    );
                }
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
}

impl<E: EntropySource> AgentPool for InProcessAgentPool<E> {
    fn spawn(
        &mut self,
        role: AgentRole,
        task: String,
    ) -> Result<AgentHandle, AgentPoolError> {
        // H-42: no depth scaling in the bare spawn(); use spawn_with_level() for
        // callers that know the real DecompositionLevel (e.g. BabyStep dispatch).
        self.spawn_inner(role, task, crate::pipeline_tree_navigator::DecompositionLevel::Concept)
    }

    fn spawn_with_level(
        &mut self,
        role: AgentRole,
        task: String,
        level: crate::pipeline_tree_navigator::DecompositionLevel,
    ) -> Result<AgentHandle, AgentPoolError> {
        self.spawn_inner(role, task, level)
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
        // E-12: production-grade auditor (E-11 prereq, fulfilled in E-12).
        // Plan-scope; runs the 15-item production checklist + framework
        // profile gates against the emitted FilePlan.
        AgentRole::AuditorProductionGrade => "auditor_production_grade",
        // E-37: prompt-injection classifier (off-model, PromptArmor-style).
        AgentRole::InjectionClassifier => "injection_classifier",
    }
}

// ---------------------------------------------------------------------------
// default_grc_config: per-role GRC budget mapping (reasoning.txt L88-124)
// ---------------------------------------------------------------------------

pub fn default_grc_config(role: &AgentRole) -> AgentGrcConfig {
    let config = match role {
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
        // WARN-attribution fix (H-41 fixer): delegate to the reasoning crate's
        // canonical config table so `transition_overrides` (specifically
        // Hypothesize -> [Verify], Verify -> [Verify, Observe]) are included.
        // The hardcoded arm was missing those overrides; the new validate_config
        // call in this function caught the latent bug. Pattern mirrors B-41
        // per-level auditor arms above.
        AgentRole::Auditor => agent_grc_config_for_role("auditor").unwrap_or_else(|| {
            let mut overrides = FxHashMap::default();
            overrides.insert(GrcState::Hypothesize, vec![GrcState::Verify]);
            overrides.insert(
                GrcState::Verify,
                vec![GrcState::Verify, GrcState::Observe],
            );
            AgentGrcConfig {
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
                transition_overrides: overrides,
                require_scope_enumeration: false,
                ..AgentGrcConfig::default()
            }
        }),
        // base-phase-2 (2C): Designer + PromptEngineer now delegate to the
        // reasoning crate's canonical config table (agent_grc_config_for_role),
        // exactly like the ContextCompiler / MediaValidator / IdeaFramer arms.
        // This removes the two-source-of-truth divergence and fixes the latent
        // Designer chain bug.
        //
        // Pre-fix the Designer arm hardcoded [Observe, PlanAction, Act, Verify]
        // with empty transition_overrides. Observe -> PlanAction is NOT in the
        // base allowed_next (Observe -> [Hypothesize, Converge, Status]), so the
        // first Designer agent to run its second think() got back
        // GrcError::InvalidTransition and could never complete its chain. The
        // canonical chain [Observe, Hypothesize, PlanAction, Verify, Converge]
        // (spec phase-2C.txt T14) with override {PlanAction: [Verify]} is valid.
        //
        // agent_grc_config_for_role("designer") returns Some by construction
        // (explicit arm in reasoning/src/config.rs); the unwrap_or_else fallback
        // is unreachable but required to satisfy no-.expect()-in-production.
        // The fallback deliberately carries the full valid chain (NOT
        // ..AgentGrcConfig::default(), which would yield the executor-shape
        // default and re-diverge) so even the unreachable branch stays correct.
        AgentRole::Designer => agent_grc_config_for_role("designer").unwrap_or_else(|| {
            let mut overrides = FxHashMap::default();
            overrides.insert(GrcState::PlanAction, vec![GrcState::Verify]);
            AgentGrcConfig {
                role: "designer".into(),
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
                    GrcState::Verify,
                    GrcState::Converge,
                ],
                grounding_level: GroundingLevel::High,
                csq_required: true,
                transition_overrides: overrides,
                require_scope_enumeration: true,
                ..AgentGrcConfig::default()
            }
        }),
        // base-phase-2 (2C): canonical chain [Observe, Hypothesize, Converge]
        // (spec phase-2C.txt T13), budget 5/800. Pre-fix this hardcoded the
        // executor-shape 5-step chain, diverging from the reasoning crate.
        AgentRole::PromptEngineer => {
            agent_grc_config_for_role("prompt_engineer").unwrap_or_else(|| AgentGrcConfig {
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
                    GrcState::Converge,
                ],
                grounding_level: GroundingLevel::High,
                csq_required: false,
                require_scope_enumeration: false,
                ..AgentGrcConfig::default()
            })
        }
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
        AgentRole::Researcher => {
            // The Researcher chain is [Observe, Hypothesize, PlanAction, Verify,
            // Converge]. PlanAction.allowed_next() = [Act, Converge, Status] by
            // default — Verify is absent, so the PlanAction -> Verify edge fails
            // validate_step_chain without an additive override. The Designer arm
            // has the identical chain shape and uses the same override; mirror it
            // exactly (effective_allowed_next is additive-union, so [Act,
            // Converge, Status, Verify] merges correctly without listing the
            // defaults here).
            let mut overrides = FxHashMap::default();
            overrides.insert(GrcState::PlanAction, vec![GrcState::Verify]);
            AgentGrcConfig {
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
                transition_overrides: overrides,
                require_scope_enumeration: false,
                ..AgentGrcConfig::default()
            }
        }
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
        // E-12: production-grade auditor. No reasoning::config table entry
        // exists yet — fall back to Plan-mode auditor budget (6/1000) which
        // is the closest published shape. When reasoning::config gains an
        // explicit `auditor_production_grade` arm the unwrap_or_else
        // fallback becomes unreachable. Per E-12-tier-b-decisions.md the
        // explicit table arm is RUNPOD-PENDING (lives alongside the live
        // 15-runner toolchain install on the remote box).
        AgentRole::AuditorProductionGrade => {
            agent_grc_config_for_role("auditor_production_grade").unwrap_or_else(|| {
                AgentGrcConfig {
                    role: "auditor_production_grade".into(),
                    ..AgentGrcConfig::default()
                }
            })
        }
        // E-37: injection classifier role — restricted GRC budget (small,
        // 1-shot classification, no chain-of-thought required). Falls back
        // to the published config or a conservative default.
        AgentRole::InjectionClassifier => {
            agent_grc_config_for_role("injection_classifier").unwrap_or_else(|| {
                AgentGrcConfig {
                    role: "injection_classifier".into(),
                    ..AgentGrcConfig::default()
                }
            })
        }
    };

    // base-phase-2 (2C): validate the hardcoded config EAGERLY at construction
    // instead of deferring to the first failing think() at runtime. Without
    // this, an invalid required_steps chain (e.g. the pre-fix Designer arm
    // whose Observe -> PlanAction edge is not in the base transition graph)
    // stays dormant until an agent of that role runs its second reasoning step,
    // at which point think() returns GrcError::InvalidTransition deep inside a
    // live run. Validating here makes any future drift fail fast and loud.
    //
    // No production panic (CLAUDE.md no-.unwrap()/.expect() rule): on failure
    // we emit a tracing::error! so the dead-evidence path screams, and a
    // debug_assert! so dev + test builds abort immediately while release builds
    // log-and-continue with the (still returned) config. The canonical configs
    // above all pass validation today; this guard exists to catch regressions.
    let loader = TomlConfigLoader::new();
    if let Err(err) = loader.validate_config(&config) {
        tracing::error!(
            target: "orchestrator::agent_pool",
            role = %config.role,
            error = %err,
            "default_grc_config produced an INVALID GRC config — this is a code \
             regression in the hardcoded per-role chain; fix the required_steps \
             / transition_overrides for this role"
        );
        debug_assert!(
            false,
            "default_grc_config({:?}) is invalid: {err}",
            role
        );
    }

    config
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
        | AgentRole::AuditorBabyStepDetail
        | AgentRole::AuditorProductionGrade => {
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
// D-3: Multi-verifier consensus call site
// ---------------------------------------------------------------------------

use crate::verification_result::{
    aggregate as aggregate_verdict, AggregateVerdict, ConsensusRule, VerificationResult,
};

/// Trait for objects that can emit consensus SSB rows. Mirrors the
/// `PpmSsbEmitter` shape from D-2 so the test harness can capture both PPM
/// rows and consensus rows through the same in-memory backing store.
///
/// Production callers wrap `Ssb::write_payload`; the test emitter
/// [`VecConsensusEmitter`] pushes payloads into a `Vec` so unit + smoke tests
/// can assert on row contents without spinning up SQLite.
pub trait ConsensusSsbEmitter: Send + Sync {
    fn emit(
        &self,
        payload: crate::ssb::SsbPayload,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;
}

/// One verifier-side adapter consumed by [`run_consensus_pass`]. Implementors
/// (PPM, intrinsic, deterministic, future verifier kinds) return a
/// [`VerificationResult`] for a given `candidate_id`. The trait uses native
/// async fn — `async-trait` is rejected per Anti-Stale.
///
/// The error type is `String` to keep the trait object-friendly and to allow
/// per-adapter error families to propagate without infecting the consensus
/// surface; the consensus pass treats any adapter error as "verifier
/// unavailable" and continues with the remaining adapters.
pub trait VerifierAdapter: Send + Sync {
    fn verify(
        &self,
        candidate_id: &str,
    ) -> impl std::future::Future<Output = Result<VerificationResult, String>> + Send;
}

/// Public summary of one [`run_consensus_pass`] invocation. Surfaced via the
/// `get_recent_verifications` Tauri command + the
/// `VerificationConsensusPanel` Svelte drawer.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierConsensusPass {
    pub candidate_id: String,
    pub template_id: String,
    pub verdict: AggregateVerdict,
    pub results: Vec<VerificationResult>,
    pub rule_id: String,
}

/// Run a multi-verifier consensus pass for `candidate_id` against the
/// supplied `adapters`, aggregate using `rule`, emit a single SSB row, and
/// return the aggregated summary.
///
/// Wiring contract (D-3):
/// - PRODUCER: this function. Reachable from `agent_pool` (its public free
///   functions are part of orchestrator's public surface) and called from
///   the verifier-consensus integration test under
///   `crates/orchestrator/tests/`.
/// - CONSUMER: the pre-promotion gate
///   `verification_result::gate_with_consensus` reads the returned verdict.
/// - SSB ROW: exactly one row per call. `Pass` -> `VerifierConsensusPass`;
///   anything else (`Fail` / `Inconclusive`) -> `VerifierConsensusFail`.
///
/// Adapter errors are treated as "verifier unavailable" — the failing
/// verifier is skipped, a debug log entry is emitted, and the consensus
/// runs over the remaining adapters. This matches the D-2 PPM unavailable
/// behaviour (defer rather than reject).
pub async fn run_consensus_pass<E>(
    candidate_id: impl Into<String>,
    template_id: impl Into<String>,
    adapters: &[&dyn DynVerifierAdapter],
    rule: &ConsensusRule,
    emitter: &E,
) -> Result<VerifierConsensusPass, String>
where
    E: ConsensusSsbEmitter + ?Sized,
{
    let candidate_id = candidate_id.into();
    let template_id = template_id.into();

    let mut results: Vec<VerificationResult> = Vec::with_capacity(adapters.len());
    for adapter in adapters {
        match adapter.verify_dyn(&candidate_id).await {
            Ok(r) => results.push(r),
            Err(e) => {
                tracing::debug!(
                    candidate_id = %candidate_id,
                    error = %e,
                    "run_consensus_pass: verifier adapter unavailable; skipping"
                );
            }
        }
    }

    let verdict = aggregate_verdict(rule, &results);
    let rule_id = rule.rule_id().to_string();

    // Emit one SSB row per call. Producer wiring is exhaustive: Pass on one
    // arm, Fail/Inconclusive on the other. Empty `results` AND empty
    // `failing_verifier_ids` is still emitted as a Fail row carrying the
    // verdict reason so downstream consumers see the rejection.
    let payload = if verdict.is_pass() {
        let pass_count = results.iter().filter(|r| r.passed).count() as u32;
        crate::ssb::SsbPayload::VerifierConsensusPass(
            crate::ssb::VerifierConsensusPassPayload {
                candidate_id: candidate_id.clone(),
                template_id: template_id.clone(),
                verdict: verdict.as_str().into(),
                verifier_count: results.len() as u32,
                pass_count,
                rule_id: rule_id.clone(),
            },
        )
    } else {
        let failing: Vec<String> = match &verdict {
            AggregateVerdict::Fail { reasons } => reasons.clone(),
            _ => Vec::new(),
        };
        let reason: String = match &verdict {
            AggregateVerdict::Fail { .. } => "fail".into(),
            AggregateVerdict::Inconclusive { reason } => reason.clone(),
            AggregateVerdict::Pass => unreachable!("Pass handled above"),
        };
        crate::ssb::SsbPayload::VerifierConsensusFail(
            crate::ssb::VerifierConsensusFailPayload {
                candidate_id: candidate_id.clone(),
                template_id: template_id.clone(),
                failing_verifier_ids: failing,
                reason,
            },
        )
    };
    emitter.emit(payload).await?;

    Ok(VerifierConsensusPass {
        candidate_id,
        template_id,
        verdict,
        results,
        rule_id,
    })
}

/// Object-safe wrapper trait that bridges [`VerifierAdapter`] (native async
/// fn in trait, not dyn-compatible) into a boxed-future shape callable
/// through `&dyn`. Implementors should NOT implement this trait directly —
/// implement [`VerifierAdapter`] and use the blanket [`AdapterErased`]
/// wrapper below.
pub trait DynVerifierAdapter: Send + Sync {
    fn verify_dyn<'a>(
        &'a self,
        candidate_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<VerificationResult, String>> + Send + 'a>,
    >;
}

/// Blanket wrapper that converts a [`VerifierAdapter`] (with native
/// async-fn-in-trait) into a [`DynVerifierAdapter`] (object-safe via boxed
/// future). Use `AdapterErased::new(adapter)` to pass a list of
/// heterogeneous adapters to `run_consensus_pass`.
pub struct AdapterErased<A: VerifierAdapter> {
    inner: A,
}

impl<A: VerifierAdapter> AdapterErased<A> {
    pub fn new(inner: A) -> Self {
        Self { inner }
    }
}

impl<A: VerifierAdapter + 'static> DynVerifierAdapter for AdapterErased<A> {
    fn verify_dyn<'a>(
        &'a self,
        candidate_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<VerificationResult, String>> + Send + 'a>,
    > {
        Box::pin(self.inner.verify(candidate_id))
    }
}

/// In-memory consensus emitter used by tests + the smoke test. Captures
/// SsbPayload rows in a Mutex<Vec<_>> so assertions can inspect them after
/// `run_consensus_pass` returns.
#[derive(Default)]
pub struct VecConsensusEmitter {
    rows: tokio::sync::Mutex<Vec<crate::ssb::SsbPayload>>,
}

impl VecConsensusEmitter {
    pub fn new() -> Self {
        Self {
            rows: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn rows(&self) -> Vec<crate::ssb::SsbPayload> {
        self.rows.lock().await.clone()
    }

    pub async fn len(&self) -> usize {
        self.rows.lock().await.len()
    }
}

impl ConsensusSsbEmitter for VecConsensusEmitter {
    async fn emit(&self, payload: crate::ssb::SsbPayload) -> Result<(), String> {
        self.rows.lock().await.push(payload);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// D-13: Adversarial-verification pass + heterogeneous-verifier gate
// ---------------------------------------------------------------------------
//
// `run_adversarial_pass` is the call-site that ties together the D-3
// consensus aggregator + the D-13 adversary generator. The sequence:
//
//   1. Run the verifier consensus on the original claim -> original_verdict.
//   2. Ask the adversary to emit a structured `CounterEvidenceSchema`.
//   3. Emit one `AdversaryGenerated` SSB row carrying counter_id,
//      claim_id, severity, model_family.
//   4. Re-run the verifier consensus on the same adapters, this time
//      attributing the counter-evidence as a fresh failing
//      `Deterministic` signal when `severity >= severity_threshold`.
//      Emit one `ConsensusReevaluatedWithCounter` SSB row carrying
//      (claim_id, original_verdict, post_counter_verdict).
//   5. Check `heterogeneous_verifier_check` on the verifier pool. If
//      `false`, emit a `tracing::warn!` with `HeterogeneousVerifierMissing`
//      so the caller can decide between retry-with-cross-family or Mode E.
//
// Grounding: arxiv:2403.07974 (T1) adversarial verification +
// arxiv:2504.16760 (T1) lightweight latent verifiers (heterogeneous-
// family verification as the documented defense). No external network
// calls in this function — the adversary trait is the I/O boundary.

use crate::adversary::{
    heterogeneous_verifier_check, AdversaryConfig, AdversaryError, CounterEvidenceSchema,
    DynAdversary, ModelFamily,
};
use crate::verification_result::VerifierKind;

/// Public summary of one [`run_adversarial_pass`] invocation. Mirrors
/// [`VerifierConsensusPass`] but carries the adversary + the verdict
/// transition for dashboard rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct AdversarialRunSummary {
    pub claim_id: String,
    pub counter: CounterEvidenceSchema,
    pub original_verdict: AggregateVerdict,
    pub post_counter_verdict: AggregateVerdict,
    pub heterogeneous: bool,
}

/// Run an adversarial pass for `claim` against the supplied
/// `adapters` + `adversary`. Emits two SSB rows
/// (`AdversaryGenerated` + `ConsensusReevaluatedWithCounter`) and
/// returns an [`AdversarialRunSummary`] for downstream gating.
///
/// Wiring contract (D-13):
/// - PRODUCER: this function. Reachable from `agent_pool` (its public
///   free functions are part of the orchestrator's public surface) and
///   exercised end-to-end by the integration test under
///   `crates/orchestrator/tests/adversary_pass_integration.rs`.
/// - CONSUMER: the gate helper
///   `verification_result::gate_with_adversarial_check` consumes the
///   summary's `post_counter_verdict` (NOT the original) when deciding
///   whether to promote.
/// - SSB ROWS: exactly two rows per call (one `AdversaryGenerated` +
///   one `ConsensusReevaluatedWithCounter`). The
///   `HeterogeneousVerifierMissing` warning is `tracing::warn!` only —
///   it does NOT add a third SSB row (no schema slot reserved per the
///   user brief), so dashboards rely on the `heterogeneous` field of
///   `AdversarialRunSummary` for that signal.
///
/// Adapter errors during the re-run consensus pass are skipped (same
/// behaviour as `run_consensus_pass`). Adversary errors propagate via
/// the `Result` — the caller decides Mode E vs. retry.
pub async fn run_adversarial_pass<E>(
    claim: &Claim,
    adversary: &dyn DynAdversary,
    config: &AdversaryConfig,
    adapters: &[&dyn DynVerifierAdapter],
    verifier_families: &[(VerifierKind, ModelFamily)],
    rule: &ConsensusRule,
    emitter: &E,
) -> Result<AdversarialRunSummary, AdversaryError>
where
    E: ConsensusSsbEmitter + ?Sized,
{
    // ----- Step 1: original consensus pass.
    let mut original_results: Vec<VerificationResult> = Vec::with_capacity(adapters.len());
    for adapter in adapters {
        match adapter.verify_dyn(&claim.claim_id).await {
            Ok(r) => original_results.push(r),
            Err(e) => {
                tracing::debug!(
                    claim_id = %claim.claim_id,
                    error = %e,
                    "run_adversarial_pass: original verifier adapter unavailable; skipping"
                );
            }
        }
    }
    let original_verdict = aggregate_verdict(rule, &original_results);

    // ----- Step 2: ask the adversary for structured counter-evidence.
    let counter = adversary
        .generate_counter_dyn(claim, config)
        .await?;

    // ----- Step 3: emit AdversaryGenerated SSB row.
    let adv_payload = crate::ssb::SsbPayload::AdversaryGenerated(
        crate::ssb::AdversaryGeneratedPayload {
            counter_id: counter.counter_id.clone(),
            claim_id: claim.claim_id.clone(),
            severity: counter.severity,
            model_family: config.adversary_model_family.as_str().to_string(),
        },
    );
    if let Err(e) = emitter.emit(adv_payload).await {
        tracing::warn!(
            claim_id = %claim.claim_id,
            error = %e,
            "run_adversarial_pass: AdversaryGenerated emit failed (pass continues)"
        );
    }

    // ----- Step 4: re-run consensus with the counter as a fresh
    // Deterministic signal when it clears the severity floor. The
    // counter is modeled as a `failing` deterministic result so the
    // aggregator sees one more "not passed" signal pulling the
    // verdict toward Fail / Inconclusive. When `severity` is below
    // the floor, the counter does NOT contribute (consistent with
    // `CounterEvidenceSchema::invalidates_claim`).
    let mut post_results = original_results.clone();
    if counter.invalidates_claim(config.severity_threshold) {
        let now_ms = original_results
            .last()
            .map(|r| r.ts_ms)
            .unwrap_or(0);
        post_results.push(VerificationResult::from_deterministic(
            claim.claim_id.clone(),
            format!("adversary:{}", counter.counter_id),
            false,
            now_ms,
        ));
    }
    let post_counter_verdict = aggregate_verdict(rule, &post_results);

    let consensus_payload = crate::ssb::SsbPayload::ConsensusReevaluatedWithCounter(
        crate::ssb::ConsensusReevaluatedWithCounterPayload {
            claim_id: claim.claim_id.clone(),
            original_verdict: original_verdict.as_str().to_string(),
            post_counter_verdict: post_counter_verdict.as_str().to_string(),
        },
    );
    if let Err(e) = emitter.emit(consensus_payload).await {
        tracing::warn!(
            claim_id = %claim.claim_id,
            error = %e,
            "run_adversarial_pass: ConsensusReevaluatedWithCounter emit failed"
        );
    }

    // ----- Step 5: heterogeneous-verifier gate.
    let heterogeneous = heterogeneous_verifier_check(claim, verifier_families);
    if !heterogeneous {
        tracing::warn!(
            claim_id = %claim.claim_id,
            verifier_count = verifier_families.len(),
            "HeterogeneousVerifierMissing: verifier pool spans <2 model families; \
             FalsifiabilityCrossFamily guarantees degraded (arxiv:2504.16760, T1)"
        );
    }

    Ok(AdversarialRunSummary {
        claim_id: claim.claim_id.clone(),
        counter,
        original_verdict,
        post_counter_verdict,
        heterogeneous,
    })
}

/// Composite gate: returns `Ok(())` only when the post-counter
/// consensus passes. The original verdict is ignored — the adversarial
/// pass exists precisely so that a claim which "passed" the first
/// consensus can still be rejected when cross-family counter-evidence
/// invalidates it. Per the user brief: the consensus is gated on
/// counter-evidence NOT invalidating the claim.
///
/// Wiring: extends D-3's `gate_with_consensus` with the adversarial
/// half. Production callers route Mode B claims through this gate
/// instead of the raw consensus gate.
#[must_use]
pub fn gate_with_adversarial_check(summary: &AdversarialRunSummary) -> bool {
    summary.post_counter_verdict.is_pass()
}

/// D-13 read surface — typed projection of an `adversary_generated` +
/// `consensus_reevaluated_with_counter` SSB row pair, joined by
/// `claim_id`. Consumed by the Tauri `get_recent_adversarial_runs`
/// command + the `AdversaryPanel` drawer.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdversaryRunRow {
    pub counter_id: String,
    pub claim_id: String,
    pub severity: f64,
    pub model_family: String,
    pub original_verdict: String,
    pub post_counter_verdict: String,
    pub ts_ms: i64,
}

/// Return the most recent `adversary_generated` + matching
/// `consensus_reevaluated_with_counter` row pairs (capped at `limit`)
/// joined on `claim_id`. Returns an empty `Vec` when the viewer has
/// no rows or errors (logged via `tracing::warn`).
#[must_use]
pub fn recent_adversarial_runs(
    viewer: &crate::ssb_viewer::SsbViewer,
    limit: usize,
) -> Vec<AdversaryRunRow> {
    let adv_rows = match viewer.recent_rows_of_type("adversary_generated", limit) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(
                action_type = "adversary_generated",
                error = %e,
                "recent_adversarial_runs: viewer error"
            );
            return Vec::new();
        }
    };
    let consensus_rows = match viewer
        .recent_rows_of_type("consensus_reevaluated_with_counter", limit)
    {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(
                action_type = "consensus_reevaluated_with_counter",
                error = %e,
                "recent_adversarial_runs: viewer error"
            );
            return Vec::new();
        }
    };

    // Index consensus rows by claim_id for the join. `BTreeMap` so the
    // iteration order is deterministic when multiple consensus rows
    // share a claim_id (latest-by-ts wins on insert).
    let mut consensus_by_claim: std::collections::BTreeMap<String, (String, String, i64)> =
        std::collections::BTreeMap::new();
    for d in consensus_rows {
        let outer = match d.payload_json.as_object() {
            Some(o) => o,
            None => continue,
        };
        let obj = if let Some(inner) =
            outer.get("ConsensusReevaluatedWithCounter").and_then(|v| v.as_object())
        {
            inner.clone()
        } else {
            outer.clone()
        };
        let claim_id = obj
            .get("claim_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let ov = obj
            .get("original_verdict")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let pv = obj
            .get("post_counter_verdict")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        consensus_by_claim
            .entry(claim_id)
            .and_modify(|(eov, epv, ets)| {
                if d.ts > *ets {
                    *eov = ov.clone();
                    *epv = pv.clone();
                    *ets = d.ts;
                }
            })
            .or_insert((ov, pv, d.ts));
    }

    let mut out = Vec::with_capacity(adv_rows.len());
    for d in adv_rows {
        let outer = match d.payload_json.as_object() {
            Some(o) => o,
            None => continue,
        };
        let obj = if let Some(inner) =
            outer.get("AdversaryGenerated").and_then(|v| v.as_object())
        {
            inner.clone()
        } else {
            outer.clone()
        };
        let claim_id = obj
            .get("claim_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let (ov, pv) = match consensus_by_claim.get(&claim_id) {
            Some((o, p, _)) => (o.clone(), p.clone()),
            None => (String::new(), String::new()),
        };
        out.push(AdversaryRunRow {
            counter_id: obj
                .get("counter_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            claim_id,
            severity: obj.get("severity").and_then(|v| v.as_f64()).unwrap_or(0.0),
            model_family: obj
                .get("model_family")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            original_verdict: ov,
            post_counter_verdict: pv,
            ts_ms: d.ts,
        });
    }
    out.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    out.truncate(limit);
    out
}

/// D-13 read surface — the [`AdversaryConfig`] currently loaded by the
/// orchestrator. Returned as a serializable DTO so the dashboard can
/// render the active adversary family + severity floor without
/// reaching into the orchestrator's internal config types.
#[must_use]
pub fn adversary_config_snapshot(config: &AdversaryConfig) -> AdversaryConfigDto {
    AdversaryConfigDto {
        adversary_model_family: config.adversary_model_family.as_str().to_string(),
        max_counter_attempts: config.max_counter_attempts,
        severity_threshold: config.severity_threshold,
    }
}

/// DTO mirror of [`AdversaryConfig`] for the Tauri / dashboard surface.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdversaryConfigDto {
    pub adversary_model_family: String,
    pub max_counter_attempts: u32,
    pub severity_threshold: f64,
}

// ---------------------------------------------------------------------------
// D-4: Tripwire monitor post-consensus hook
// ---------------------------------------------------------------------------

use crate::tripwire::{
    auto_revert as tripwire_auto_revert, build_auto_reverted_payload,
    build_tripwire_fired_payload, ObserveOutcome, TripwireFired, TripwireMonitor,
    TripwireSample, TripwireSsbEmitter,
};

/// Observation pump that feeds a single `TripwireSample` into the monitor,
/// emits one `TripwireFired` SSB row per detector firing, and (when the
/// caller provides an archive_chain + template_id matching a concurrent
/// fire ≥2 detectors) auto-reverts the template.
///
/// Producer wiring contract (D-4):
/// - This function is the production producer of `TripwireFired` +
///   `TemplateAutoReverted` SSB rows.
/// - Reachable from: agent_pool's post-consensus path. Callers invoke
///   `observe_post_consensus_tripwires` right after `run_consensus_pass`
///   returns a `Pass` verdict, passing the per-step IntrinsicSignals
///   bundle as the sample input.
/// - SSB emission ordering: `TripwireFired` rows are emitted BEFORE
///   `auto_revert` mutates the archive chain (see tripwire::auto_revert
///   stub_ban: audit trail must capture intent even if revert fails).
///
/// The function is best-effort with respect to SSB writes (failures are
/// logged); auto-revert errors propagate as `Err(String)` so the caller can
/// surface them or bail. When `archive_chain` is `None` no auto-revert is
/// attempted regardless of firings (used by tests + the smoke-test harness
/// that exercises firings without a live chain).
pub async fn observe_post_consensus_tripwires<E>(
    monitor: &mut TripwireMonitor,
    archive_chain: Option<&mut crate::archive_chain::ArchiveChain>,
    template_id: Option<&crate::reasoning_template::TemplateId>,
    candidate_id: &str,
    sample: TripwireSample,
    emitter: &E,
) -> Result<ObserveOutcome, String>
where
    E: TripwireSsbEmitter + ?Sized,
{
    let outcome = monitor.observe(candidate_id, sample);

    // Emit per-firing SSB rows BEFORE mutating the archive (D-4 stub_ban).
    for fired in &outcome.fired {
        let payload = build_tripwire_fired_payload(fired);
        if let Err(e) = emitter.emit(payload).await {
            tracing::warn!(
                event_id = fired.event_id.as_str(),
                kind = fired.kind.as_str(),
                error = %e,
                "observe_post_consensus_tripwires: TripwireFired SSB emit failed"
            );
        }
    }

    // Auto-revert when ≥2 concurrent firings AND an archive_chain +
    // template_id are supplied (the production path passes them; tests can
    // pass None to assert firings without revert side-effects).
    if outcome.fired.len() >= 2 {
        if let (Some(chain), Some(tid)) = (archive_chain, template_id) {
            // Use the first firing as the "trigger" event for lineage tracing;
            // the other firings still appear in SSB as standalone rows.
            let trigger: &TripwireFired = &outcome.fired[0];
            match tripwire_auto_revert(chain, tid, trigger) {
                Ok(revert_outcome) => {
                    let payload = build_auto_reverted_payload(&revert_outcome);
                    if let Err(e) = emitter.emit(payload).await {
                        tracing::warn!(
                            template_id = revert_outcome.template_id.as_str(),
                            error = %e,
                            "observe_post_consensus_tripwires: TemplateAutoReverted SSB emit failed"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        template_id = tid.as_hex(),
                        error = %e,
                        "observe_post_consensus_tripwires: auto_revert rejected"
                    );
                    return Err(format!("auto_revert: {e}"));
                }
            }
        }
    }

    Ok(outcome)
}

// ---------------------------------------------------------------------------
// D-2: PPM verifier consensus call site
// ---------------------------------------------------------------------------

/// Run PPM scoring on a candidate-step chain produced by an agent and
/// persist the result to SSB. This is the production CALL SITE that
/// satisfies the D-2 WIRING_CONTRACT — `emit_ppm_score` is reachable
/// from a non-test orchestrator path.
///
/// The function is additive: it does NOT alter any existing agent_pool
/// behaviour. Callers that have a `PpmClient` configured (i.e.
/// `ELAI_PPM_ENABLE=1` with the second sidecar online) invoke this
/// after the executor produces a step chain; consensus aggregation
/// (D-3) layers on top of the returned `PpmScore`.
///
/// When the PPM is unavailable the function returns
/// `Err(PpmClientError::Hook(PpmHookError::Unavailable))` and the caller
/// applies its own fallback policy (typically "defer promotion" — see
/// D-3 plan). No SSB row is emitted on transport failure; an empty
/// gating verdict is preserved as an absence rather than a misleading
/// `PpmGatingFailed` row.
pub async fn run_ppm_verifier_check<H, E>(
    client: &crate::ppm_client::PpmClient<H>,
    candidate_id: impl Into<String>,
    verifier_id: impl Into<String>,
    problem_statement: impl Into<String>,
    reasoning_steps: Vec<String>,
    mode: crate::sidecar_ppm_hook::PpmMode,
    emitter: &E,
) -> Result<crate::ppm_client::PpmScore, crate::ppm_client::PpmClientError>
where
    H: crate::sidecar_ppm_hook::SidecarPpmHook,
    E: crate::ppm_client::PpmSsbEmitter + ?Sized,
{
    let req = crate::sidecar_ppm_hook::PpmScoreRequest {
        candidate_id: candidate_id.into(),
        verifier_id: verifier_id.into(),
        problem_statement: problem_statement.into(),
        reasoning_steps,
        mode,
    };
    crate::ppm_client::emit_ppm_score(client, req, emitter).await
}

// ---------------------------------------------------------------------------
// D-6 — held-out task sampling for the BAI promotion-gate eval pool.
// ---------------------------------------------------------------------------

/// D-6 — sample one held-out task from `store` whose eligibility window has
/// elapsed at `now_ms`. Returns `None` when the active pool is empty (no
/// tasks ingested yet, or every ingested task is still inside its
/// eligibility window).
///
/// Selection is leftmost-active (oldest-ingest-first) — deterministic and
/// independent of the caller's RNG so two parallel BAI rounds can call
/// this in any order without leaking eval-order signal into the
/// promotion decision. Callers that need diversity sampling can layer
/// their own RNG-driven choice over the returned vec via
/// [`crate::held_out::HeldOutStore::active_tasks`] directly.
///
/// This function is the BAI-side wiring contract: `bai::promote_candidate`
/// consumes the returned [`crate::held_out::HeldOutTask`] as the source of
/// its `StagedEvalResult` Pass/Fail signal. See workflow/reasoning/D-6-decisions.md.
pub fn sample_held_out_task(
    store: &crate::held_out::HeldOutStore,
    now_ms: i64,
) -> Option<crate::held_out::HeldOutTask> {
    match store.active_tasks(now_ms) {
        Ok(mut tasks) => {
            if tasks.is_empty() {
                None
            } else {
                // Oldest-ingest-first: active_tasks already returns
                // ascending by ingested_ts_ms.
                Some(tasks.remove(0))
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "sample_held_out_task: store error");
            None
        }
    }
}

// ---------------------------------------------------------------------------
// D-7 — recent canary status (read surface for BAI gate + dashboard).
// ---------------------------------------------------------------------------

/// D-7 — fold the most recent `CanaryRunCompleted` SSB row into a typed
/// summary. The BAI promotion gate calls this BEFORE running the
/// promotion rule: if the most recent canary verdict is `Fail`, the
/// gate may block promotion (the BAI module dispatches on the
/// `dominant_family` to attribute the block).
///
/// `viewer` is the same `SsbReader`-backed `SsbViewer` the dashboard
/// commands consume; passing it in (rather than re-opening a fresh
/// connection) keeps the canary read surface zero-cost in the
/// promotion-gate hot path.
///
/// Returns `None` when no `CanaryRunCompleted` row has been written yet
/// (fresh install / canary not yet exercised).
///
/// See workflow/reasoning/D-7-decisions.md.
#[must_use]
pub fn recent_canary_status(viewer: &crate::ssb_viewer::SsbViewer) -> Option<CanaryStatusSummary> {
    let details = viewer
        .recent_rows_of_type("canary_run_completed", 1)
        .ok()?;
    let detail = details.into_iter().next()?;
    // The viewer JSON-encodes the externally-tagged `SsbPayload`, so
    // the value is shaped `{ "CanaryRunCompleted": { ...fields... } }`.
    // Unwrap one level to get at the payload struct's fields. We also
    // accept the un-wrapped shape for forward-compat in case the
    // viewer adopts a custom Serialize that flattens variants.
    let obj = match detail.payload_json.as_object() {
        Some(o) => {
            if let Some(inner) = o
                .get("CanaryRunCompleted")
                .and_then(|v| v.as_object())
            {
                inner.clone()
            } else {
                o.clone()
            }
        }
        None => return None,
    };
    Some(CanaryStatusSummary {
        report_id: obj
            .get("report_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        ts_ms: obj
            .get("ts_ms")
            .and_then(|v| v.as_i64())
            .unwrap_or(0),
        overall_transfer_score: obj
            .get("overall_transfer_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        decision_kind: obj
            .get("decision_kind")
            .and_then(|v| v.as_str())
            .unwrap_or("pass")
            .to_string(),
        dominant_fail_family: obj
            .get("dominant_fail_family")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

/// Typed projection of the most recent `CanaryRunCompleted` SSB row.
/// Consumed by the BAI gate (block-on-fail) + the dashboard
/// `get_canary_recent` command.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CanaryStatusSummary {
    pub report_id: String,
    pub ts_ms: i64,
    pub overall_transfer_score: f64,
    /// Snake-case: `pass` / `marginal_pass` / `fail`.
    pub decision_kind: String,
    /// Empty unless `decision_kind == "fail"`.
    pub dominant_fail_family: String,
}

impl CanaryStatusSummary {
    /// True when the BAI gate should block promotion on this run.
    #[must_use]
    pub fn blocks_promotion(&self) -> bool {
        self.decision_kind == "fail"
    }
}

// ---------------------------------------------------------------------------
// D-10 — mode-routing producer (`route_claim`).
// ---------------------------------------------------------------------------
//
// The orchestrator (NOT the policy model) picks the
// [`crate::grounding_mode::GroundingMode`] for every novel claim. The
// pure predicate `select_grounding_mode` lives in
// `crates/orchestrator/src/grounding_mode.rs` and is L3-locked via the
// D-9 locked_keys_registry — the agent cannot edit the routing logic.
//
// This block exposes the production producer surface:
//   1. `ClaimRoutedEmitter` trait — the SSB-write side, mockable in
//      tests so the routing decision table can be exercised without a
//      live Ssb database.
//   2. `route_claim` async fn — single public entry the future
//      task_runner-side dispatcher calls. Returns `(GroundingMode,
//      EmissionGuard)` so the caller can confirm the emission landed
//      before dispatching to the chosen verifier (RAII semantics: if
//      the guard is dropped without `confirm()`, the producer logs
//      via `tracing::warn` so the dashboard can detect a partial
//      route).

use crate::checker_registry::{CheckerKind, CheckerRegistry, CheckerResult};
use crate::claim::{Claim, ClaimType};
use crate::grounding_mode::{select_grounding_mode, GroundingMode};
use crate::orch_context::OrchContext;
use crate::ssb::{CheckerGapDetectedPayload, CheckerResultEmittedPayload, ClaimRoutedPayload};

/// Producer-side SSB emitter for `claim_routed` rows. The production
/// adapter wraps `Arc<Ssb>::write_payload` (callers in `task_runner`
/// own the Ssb writer); tests use [`VecClaimRoutedEmitter`] below.
///
/// Native async-fn-in-trait per the anti-stale-training rule — no
/// `async-trait` crate needed.
pub trait ClaimRoutedEmitter: Send + Sync {
    /// Emit one `claim_routed` row. Returning `Err(String)` does NOT
    /// abort the route — the orchestrator already picked the mode;
    /// the row is the durable audit signal, and a failed write is
    /// logged + surfaced via `EmissionGuard::status`.
    fn emit(
        &self,
        payload: ClaimRoutedPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>>;
}

/// Test-only emitter that accumulates rows in a `Vec` behind a
/// `tokio::sync::Mutex` (async-safe, no `std::sync::Mutex` hold across
/// await per the anti-stale-training rule).
#[derive(Default)]
pub struct VecClaimRoutedEmitter {
    rows: tokio::sync::Mutex<Vec<ClaimRoutedPayload>>,
}

impl VecClaimRoutedEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn rows(&self) -> Vec<ClaimRoutedPayload> {
        self.rows.lock().await.clone()
    }

    pub async fn len(&self) -> usize {
        self.rows.lock().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.rows.lock().await.is_empty()
    }
}

impl ClaimRoutedEmitter for VecClaimRoutedEmitter {
    fn emit(
        &self,
        payload: ClaimRoutedPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            self.rows.lock().await.push(payload);
            Ok(())
        })
    }
}

/// RAII guard returned by [`route_claim`]. Wraps the emission status
/// so the caller can:
///   * `confirm()` — mark the route as fully observed (no warning).
///   * `mode()` — re-read the selected mode without re-running the
///     predicate.
///   * `status()` — read the emission outcome (`Ok(())` or the
///     error string from the emitter).
///
/// On drop without `confirm()`, the guard logs via `tracing::warn` so
/// downstream code that ignores the route is surfaced in the SSB
/// stream (defensive — the predicate is the load-bearing piece, but
/// a silently-dropped row would still hide a routing decision).
#[must_use = "EmissionGuard records the routing outcome; drop without confirm() warns"]
pub struct EmissionGuard {
    mode: GroundingMode,
    status: Result<(), String>,
    confirmed: bool,
    claim_id: String,
}

impl EmissionGuard {
    /// The mode the orchestrator picked. Cheap copy.
    #[must_use]
    pub fn mode(&self) -> GroundingMode {
        self.mode
    }

    /// Borrow the emission outcome.
    #[must_use]
    pub fn status(&self) -> &Result<(), String> {
        &self.status
    }

    /// Mark the route as fully observed — suppresses the drop-time
    /// warning. Idempotent.
    pub fn confirm(mut self) -> GroundingMode {
        self.confirmed = true;
        self.mode
    }
}

impl Drop for EmissionGuard {
    fn drop(&mut self) {
        if !self.confirmed {
            tracing::warn!(
                claim_id = self.claim_id.as_str(),
                mode = self.mode.as_str(),
                status_ok = self.status.is_ok(),
                "EmissionGuard dropped without confirm(); routing decision may be lost"
            );
        }
    }
}

/// Compute a stable blake3 hex of the orchestrator-context snapshot
/// the router observed. Hashing the JSON wire form keeps the
/// pre-image canonical across Rust struct re-orderings (serde JSON
/// is field-name-keyed). Returns the empty string on serialize
/// failure (logged) — empty hash is a sentinel the dashboard
/// surfaces as "ctx-hash unavailable".
fn ctx_summary_blake3_hex(ctx: &OrchContext) -> String {
    match serde_json::to_vec(ctx) {
        Ok(bytes) => {
            let h = blake3::hash(&bytes);
            h.to_hex().to_string()
        }
        Err(e) => {
            tracing::warn!(error = %e, "ctx_summary_blake3_hex: serde_json::to_vec failed");
            String::new()
        }
    }
}

/// D-10 public producer surface. Calls the pure predicate
/// `select_grounding_mode`, emits a `claim_routed` SSB row, and
/// returns the `(mode, EmissionGuard)` pair.
///
/// `now_ms` is injected (no clock read inside the producer) per the
/// entropy-quarantine rule. Production callers feed the
/// `ClockProvider`-sourced time the rest of the orchestrator uses.
pub async fn route_claim<E: ClaimRoutedEmitter + ?Sized>(
    claim: &Claim,
    ctx: &OrchContext,
    emitter: &E,
    now_ms: i64,
) -> (GroundingMode, EmissionGuard) {
    let mode = select_grounding_mode(claim, ctx);
    let mode = {
        // J-94 T3-A: optional offline calibration tightening (process-wide install).
        let cal = crate::grounding_router_calibration::current_grounding_calibration();
        match cal.as_ref() {
            Some(c) => crate::grounding_router_calibration::apply_grounding_calibration(
                mode, claim.claim_type, c,
            ),
            None => mode,
        }
    };
    let payload = ClaimRoutedPayload {
        claim_id: claim.claim_id.clone(),
        claim_type: claim.claim_type.as_str().to_string(),
        mode: mode.as_str().to_string(),
        ts_ms: now_ms,
        ctx_summary_blake3: ctx_summary_blake3_hex(ctx),
    };
    let status = emitter.emit(payload).await;
    if let Err(ref e) = status {
        tracing::warn!(
            claim_id = claim.claim_id.as_str(),
            mode = mode.as_str(),
            error = %e,
            "route_claim: SSB emission failed (route still applies)"
        );
    }
    let guard = EmissionGuard {
        mode,
        status,
        confirmed: false,
        claim_id: claim.claim_id.clone(),
    };
    (mode, guard)
}

// ---------------------------------------------------------------------------
// H-41 — checker-gap emit surface.
// ---------------------------------------------------------------------------
//
// RISK-1 mitigation: route_claim signature is NOT changed. Instead we add:
//   1. CheckerGapEmitter trait — same pattern as ClaimRoutedEmitter above.
//   2. VecCheckerGapEmitter — test emitter accumulating rows in a Vec.
//   3. route_claim_with_gap — thin async wrapper over route_claim that adds
//      the gap-detect logic AFTER the routing decision, keeping all existing
//      callers untouched (the one production caller in task_runner.rs can
//      switch to route_claim_with_gap independently; this wrapper is the
//      H-41 entry point / PRODUCTION ENTRYPOINT for the gap instrument).
//
// The gap condition (per grounding_mode.rs:159-170):
//   claim is Factual|Computational AND
//   OrchContext.available_verifiers has NO VerifierKind::Deterministic
//   → route fell to FalsifiabilityCrossFamily / ConformalRefusal / ConservativeFallback
//
// Wire-dark: ELAI_CHECKER_GAP_ENABLE env var. Default-OFF per R9 / wire-dark-allowlist.toml.

/// Emitter trait for [`SsbActionType::CheckerGapDetected`] rows (H-41).
/// Production adapter wraps `Arc<Ssb>::write_payload`; tests use
/// [`VecCheckerGapEmitter`].
pub trait CheckerGapEmitter: Send + Sync {
    /// Emit one `checker_gap_detected` SSB row. Returning `Err(String)` is
    /// best-effort — the caller logs the error and continues routing.
    fn emit_gap(
        &self,
        payload: CheckerGapDetectedPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>>;
}

/// Test-only emitter that accumulates [`CheckerGapDetectedPayload`] rows in
/// a `Vec` behind a `tokio::sync::Mutex`.
#[derive(Default)]
pub struct VecCheckerGapEmitter {
    rows: tokio::sync::Mutex<Vec<CheckerGapDetectedPayload>>,
}

impl VecCheckerGapEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn rows(&self) -> Vec<CheckerGapDetectedPayload> {
        self.rows.lock().await.clone()
    }

    pub async fn len(&self) -> usize {
        self.rows.lock().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.rows.lock().await.is_empty()
    }
}

impl CheckerGapEmitter for VecCheckerGapEmitter {
    fn emit_gap(
        &self,
        payload: CheckerGapDetectedPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            self.rows.lock().await.push(payload);
            Ok(())
        })
    }
}

/// H-41 production entry point: calls [`route_claim`], then — when
/// `gap_enabled` is `true` — detects whether a Factual/Computational claim
/// fell to a non-Constructive mode for lack of a `VerifierKind::Deterministic`
/// verifier and emits a `CheckerGapDetected` SSB row via `gap_emitter`.
///
/// The gap-emit is BEST-EFFORT: if the emitter returns an error, the routing
/// decision is unchanged and the error is logged via `tracing::warn`.
///
/// Use [`route_claim_with_gap_env`] for the production caller that reads the
/// `ELAI_CHECKER_GAP_ENABLE` env var. This function accepts the flag directly
/// so tests can control it without env-var races (env vars are process-global
/// and unsafe across concurrent tokio tests).
///
/// Signature keeps `route_claim` untouched (RISK-1 mitigation: no existing
/// call site is broken). The single production caller in `task_runner.rs` can
/// switch to `route_claim_with_gap_env` independently; H-41 uses this as its
/// W-3 production entrypoint.
pub async fn route_claim_with_gap<E, G>(
    claim: &Claim,
    ctx: &OrchContext,
    emitter: &E,
    gap_emitter: &G,
    now_ms: i64,
    gap_enabled: bool,
) -> (GroundingMode, EmissionGuard)
where
    E: ClaimRoutedEmitter + ?Sized,
    G: CheckerGapEmitter + ?Sized,
{
    // Step 1: route exactly as before (no logic change — GD-1).
    let (mode, guard) = route_claim(claim, ctx, emitter, now_ms).await;

    // Step 2: gap detection — only when enabled.
    //
    // BLOCK-2(c) fix: two gap conditions are detected:
    //
    // Gap-A (original): Factual|Computational claim + no Deterministic verifier
    //   in OrchContext.available_verifiers. The claim fell to a non-Constructive
    //   mode because the D-11 checker registry has no registered adapter that can
    //   cover it.
    //
    // Gap-B (new, production path): Procedural|Hypothetical claim fell to
    //   OperationalDeferred. This represents the "no D-18 adapter registered"
    //   gap. D-18 is not yet plumbed into OrchContext (no
    //   `has_operational_adapter` field exists), so a Procedural claim that
    //   routes to OperationalDeferred is ALWAYS a gap until D-18 lands. The
    //   `dispatch_claim_routing` production site in task_runner.rs synthesises
    //   a Procedural claim, so Gap-B is the condition that fires on every real
    //   task when ELAI_CHECKER_GAP_ENABLE=1. Without Gap-B the instrument would
    //   NEVER fire from the production path (BLOCK-2 compounding bug).
    if gap_enabled {
        let emit_gap = if matches!(
            claim.claim_type,
            ClaimType::Factual | ClaimType::Computational
        ) {
            // Gap-A: Factual/Computational without a Deterministic verifier.
            let has_det = ctx
                .available_verifiers
                .iter()
                .any(|v| *v == crate::verification_result::VerifierKind::Deterministic);
            !has_det
        } else if matches!(
            claim.claim_type,
            ClaimType::Procedural | ClaimType::Hypothetical
        ) {
            // Gap-B: Procedural/Hypothetical fell to OperationalDeferred, which
            // means no D-18 sandbox adapter is registered. This IS the production
            // gap the H-41 instrument is designed to measure.
            mode == GroundingMode::OperationalDeferred
        } else {
            false
        };

        if emit_gap {
            let taxon = crate::checker_coverage::classify_complexity_taxon(claim);
            let gap_payload = CheckerGapDetectedPayload {
                claim_id: claim.claim_id.clone(),
                claim_type: claim.claim_type.as_str().to_string(),
                complexity_taxon: taxon.as_str().to_string(),
                fell_to_mode: mode.as_str().to_string(),
                ts_ms: now_ms,
            };
            if let Err(e) = gap_emitter.emit_gap(gap_payload).await {
                tracing::warn!(
                    claim_id = claim.claim_id.as_str(),
                    mode = mode.as_str(),
                    error = %e,
                    "route_claim_with_gap: gap emit failed (route still applies)"
                );
            }
        }
    }

    (mode, guard)
}

/// Production convenience wrapper over [`route_claim_with_gap`] that reads
/// `ELAI_CHECKER_GAP_ENABLE` from the environment. Default-OFF (R9 / W-2).
pub async fn route_claim_with_gap_env<E, G>(
    claim: &Claim,
    ctx: &OrchContext,
    emitter: &E,
    gap_emitter: &G,
    now_ms: i64,
) -> (GroundingMode, EmissionGuard)
where
    E: ClaimRoutedEmitter + ?Sized,
    G: CheckerGapEmitter + ?Sized,
{
    let gap_enabled = std::env::var("ELAI_CHECKER_GAP_ENABLE")
        .map(|v| v == "1")
        .unwrap_or(false);
    route_claim_with_gap(claim, ctx, emitter, gap_emitter, now_ms, gap_enabled).await
}

/// D-10 read surface — the most recent N `claim_routed` SSB rows the
/// dashboard `ModeRoutingPanel` renders. Empty `Vec` when the viewer
/// has no rows (or returns an error — logged via `tracing::warn`).
#[must_use]
pub fn recent_claim_routings(
    viewer: &crate::ssb_viewer::SsbViewer,
    limit: usize,
) -> Vec<ClaimRoutedRow> {
    let mut out = Vec::new();
    match viewer.recent_rows_of_type("claim_routed", limit) {
        Ok(details) => {
            for d in details {
                if let Some(row) = claim_routed_detail_to_row(d) {
                    out.push(row);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = "claim_routed",
                error = %e,
                "recent_claim_routings: viewer error"
            );
        }
    }
    out.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    out.truncate(limit);
    out
}

/// Typed projection of a single `claim_routed` SSB row. Consumed by
/// the Tauri `get_recent_claim_routings` command + dashboard tests.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ClaimRoutedRow {
    pub claim_id: String,
    pub claim_type: String,
    pub mode: String,
    pub ts_ms: i64,
    pub ctx_summary_blake3: String,
}

fn claim_routed_detail_to_row(d: crate::ssb_viewer::SsbActionDetail) -> Option<ClaimRoutedRow> {
    if d.action_type.as_str() != "claim_routed" {
        return None;
    }
    // The viewer JSON-encodes the externally-tagged `SsbPayload`,
    // so the value is shaped `{ "ClaimRouted": { ...fields... } }`.
    let outer = d.payload_json.as_object()?;
    let obj = if let Some(inner) = outer.get("ClaimRouted").and_then(|v| v.as_object()) {
        inner.clone()
    } else {
        outer.clone()
    };
    Some(ClaimRoutedRow {
        claim_id: obj
            .get("claim_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        claim_type: obj
            .get("claim_type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        mode: obj
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
        ctx_summary_blake3: obj
            .get("ctx_summary_blake3")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

// ---------------------------------------------------------------------------
// D-11 — constructive-checker dispatch producer.
// ---------------------------------------------------------------------------
//
// When the D-10 router picks GroundingMode::Constructive, the
// orchestrator routes the claim through the
// `crate::checker_registry::CheckerRegistry`. This block exposes the
// dispatch producer that:
//   1. invokes `CheckerRegistry::run_all_applicable`,
//   2. emits one `checker_result_emitted` SSB row per result,
//   3. returns the unmodified Vec of `(id, kind, result)` so the
//      caller can decide how to aggregate (any-Pass / all-Pass / etc).
//
// Grounding: arxiv:2310.04406 (constructive proof checking),
// arxiv:2211.12588 (PoT, T1), arxiv:2402.01030 (CodeAct, T1).

/// Producer-side SSB emitter for `checker_result_emitted` rows.
/// Mirrors [`ClaimRoutedEmitter`] in shape so production callers can
/// share a single Ssb writer adapter under both traits.
pub trait CheckerResultEmitter: Send + Sync {
    fn emit(
        &self,
        payload: CheckerResultEmittedPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>>;
}

/// Test-only emitter that records every emitted payload.
#[derive(Default)]
pub struct VecCheckerResultEmitter {
    rows: tokio::sync::Mutex<Vec<CheckerResultEmittedPayload>>,
}

impl VecCheckerResultEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn rows(&self) -> Vec<CheckerResultEmittedPayload> {
        self.rows.lock().await.clone()
    }

    pub async fn len(&self) -> usize {
        self.rows.lock().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.rows.lock().await.is_empty()
    }
}

impl CheckerResultEmitter for VecCheckerResultEmitter {
    fn emit(
        &self,
        payload: CheckerResultEmittedPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            self.rows.lock().await.push(payload);
            Ok(())
        })
    }
}

/// Map a [`CheckerResult`] to a `(result_kind, reason)` pair for SSB
/// payload construction. `reason` is bounded to 2048 chars at the
/// producer to keep SSB row sizes predictable.
fn checker_result_to_payload_fields(r: &CheckerResult) -> (&'static str, String) {
    let reason = match r {
        CheckerResult::Pass => String::new(),
        CheckerResult::Fail { reason } => bounded_reason(reason),
        CheckerResult::Inapplicable { reason } => bounded_reason(reason),
        CheckerResult::Error { source } => bounded_reason(source),
    };
    (r.result_kind(), reason)
}

fn bounded_reason(s: &str) -> String {
    const MAX: usize = 2048;
    if s.len() <= MAX {
        s.to_string()
    } else {
        // Round to a char boundary so we don't slice mid-codepoint.
        let mut end = MAX;
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        s[..end].to_string()
    }
}

/// D-11 public producer surface. Runs every applicable checker in the
/// registry, emits one `checker_result_emitted` SSB row per result,
/// and returns the raw vector for downstream aggregation.
///
/// `now_ms` is injected (no clock read inside the producer) per the
/// entropy-quarantine rule, matching [`route_claim`].
///
/// The emitter receives one payload per [`CheckerRegistry::run_all_applicable`]
/// triple. A failed emission is logged via `tracing::warn` but does
/// NOT abort dispatch — the checker results themselves are the
/// load-bearing data; the row is the durable audit signal.
pub async fn dispatch_constructive_checkers<E: CheckerResultEmitter + ?Sized>(
    claim: &Claim,
    registry: &CheckerRegistry,
    emitter: &E,
    now_ms: i64,
) -> Vec<(String, CheckerKind, CheckerResult)> {
    let triples = registry.run_all_applicable(claim).await;
    for (id, kind, result) in &triples {
        let (result_kind, reason) = checker_result_to_payload_fields(result);
        let payload = CheckerResultEmittedPayload {
            checker_id: id.clone(),
            checker_kind: kind.as_str().to_string(),
            result_kind: result_kind.to_string(),
            claim_id: claim.claim_id.clone(),
            ts_ms: now_ms,
            reason,
        };
        if let Err(e) = emitter.emit(payload).await {
            tracing::warn!(
                claim_id = claim.claim_id.as_str(),
                checker_id = id.as_str(),
                error = %e,
                "dispatch_constructive_checkers: SSB emission failed"
            );
        }
    }
    triples
}

/// D-11 read surface — the most recent N `checker_result_emitted` SSB
/// rows the dashboard `CheckerRegistryPanel` renders.
#[must_use]
pub fn recent_checker_results(
    viewer: &crate::ssb_viewer::SsbViewer,
    limit: usize,
) -> Vec<CheckerResultRow> {
    let mut out = Vec::new();
    match viewer.recent_rows_of_type("checker_result_emitted", limit) {
        Ok(details) => {
            for d in details {
                if let Some(row) = checker_result_detail_to_row(d) {
                    out.push(row);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = "checker_result_emitted",
                error = %e,
                "recent_checker_results: viewer error"
            );
        }
    }
    out.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    out.truncate(limit);
    out
}

/// Typed projection of a single `checker_result_emitted` SSB row. Same
/// shape as [`CheckerResultEmittedPayload`] but lives next to the
/// other agent_pool read surfaces ([`ClaimRoutedRow`]) for the Tauri
/// command layer.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CheckerResultRow {
    pub checker_id: String,
    pub checker_kind: String,
    pub result_kind: String,
    pub claim_id: String,
    pub ts_ms: i64,
    pub reason: String,
}

fn checker_result_detail_to_row(
    d: crate::ssb_viewer::SsbActionDetail,
) -> Option<CheckerResultRow> {
    if d.action_type.as_str() != "checker_result_emitted" {
        return None;
    }
    let outer = d.payload_json.as_object()?;
    let obj = if let Some(inner) = outer.get("CheckerResultEmitted").and_then(|v| v.as_object()) {
        inner.clone()
    } else {
        outer.clone()
    };
    Some(CheckerResultRow {
        checker_id: obj
            .get("checker_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        checker_kind: obj
            .get("checker_kind")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        result_kind: obj
            .get("result_kind")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        claim_id: obj
            .get("claim_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
        reason: obj
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

// ---------------------------------------------------------------------------
// D-12 — OperationalizationCheck baby-step predicate dispatch.
//
// When a baby step carries the three-field operationalisation triple
// (procedure + expected + claim) the auditor invokes
// `run_operational_check` to gate the MEMORIZING transition. The
// function wraps `baby_step::dispatch_check_for_baby_step`, emits one
// `baby_step_dispatched` SSB row per outcome via the
// `BabyStepDispatchedEmitter` trait, and returns the typed outcome.
//
// Grounding:
//   * arxiv:2305.16291 Voyager (T1) — skill-with-measurable-effect.
//   * arxiv:2407.01489 AGENTLESS (T1) — single-step verification.
// ---------------------------------------------------------------------------

/// Errors raised by [`run_operational_check`]. Closed enum so callers
/// can surface a single typed diagnostic per gate decision.
#[derive(Debug, thiserror::Error)]
pub enum OperationalError {
    /// Baby step failed the [`crate::baby_step::is_operationalized`]
    /// predicate (one of the three required fields was absent).
    /// MEMORIZING gate must BLOCK.
    #[error("baby step {step_id} is not operationalized")]
    NotOperationalized { step_id: String },
    /// Dispatch ran but every checker returned non-Pass — the
    /// auditor must BLOCK MEMORIZING. Carries the outcome label
    /// (`"fail"` or `"inapplicable"`).
    #[error("baby step {step_id} dispatch produced non-pass outcome: {outcome}")]
    NonPassOutcome { step_id: String, outcome: String },
}

/// Producer-side SSB emitter for `baby_step_dispatched` rows. Mirrors
/// [`CheckerResultEmitter`] in shape so production callers can share a
/// single Ssb writer adapter under both traits.
pub trait BabyStepDispatchedEmitter: Send + Sync {
    fn emit(
        &self,
        payload: crate::ssb::BabyStepDispatchedPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>>;
}

/// Test-only emitter that records every emitted payload.
#[derive(Default)]
pub struct VecBabyStepDispatchedEmitter {
    rows: tokio::sync::Mutex<Vec<crate::ssb::BabyStepDispatchedPayload>>,
}

impl VecBabyStepDispatchedEmitter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            rows: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    pub async fn rows(&self) -> Vec<crate::ssb::BabyStepDispatchedPayload> {
        self.rows.lock().await.clone()
    }

    pub async fn len(&self) -> usize {
        self.rows.lock().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.rows.lock().await.is_empty()
    }
}

impl BabyStepDispatchedEmitter for VecBabyStepDispatchedEmitter {
    fn emit(
        &self,
        payload: crate::ssb::BabyStepDispatchedPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async move {
            self.rows.lock().await.push(payload);
            Ok(())
        })
    }
}

/// Emit a `baby_step_dispatched` SSB row. Producer-only — the
/// dashboard reads via [`recent_baby_step_dispatches`]. Returns the
/// emitted payload so the caller can re-use the structured outcome
/// without re-deriving it.
pub async fn emit_baby_step_dispatched<E: BabyStepDispatchedEmitter + ?Sized>(
    step: &crate::baby_step::BabyStep,
    outcome: &crate::baby_step::DispatchOutcome,
    emitter: &E,
    now_ms: i64,
) -> crate::ssb::BabyStepDispatchedPayload {
    let payload = crate::ssb::BabyStepDispatchedPayload {
        step_id: step.step_id.clone(),
        parent_task_id: step.parent_task_id.clone(),
        checker_kinds_used: outcome.checker_kinds_used(),
        outcome: outcome.outcome_label().to_string(),
        ts_ms: now_ms,
    };
    if let Err(e) = emitter.emit(payload.clone()).await {
        tracing::warn!(
            step_id = step.step_id.as_str(),
            error = %e,
            "emit_baby_step_dispatched: SSB emission failed"
        );
    }
    payload
}

/// D-12 gating function. Calls
/// [`crate::baby_step::dispatch_check_for_baby_step`], emits the SSB
/// row via `emitter`, and converts the outcome to a `Result<(),
/// OperationalError>` the auditor can match on.
///
/// `Ok(outcome)` when every checker returned Pass (auditor allows
/// MEMORIZING). `Err(OperationalError::NotOperationalized)` when the
/// baby step is missing one of the three operationalisation fields
/// (auditor BLOCKs). `Err(OperationalError::NonPassOutcome)` when the
/// dispatch returned `Inapplicable` or any checker returned non-Pass.
///
/// `now_ms` is injected (no clock read inside the producer) per the
/// entropy-quarantine rule, matching [`route_claim`] +
/// [`dispatch_constructive_checkers`].
pub async fn run_operational_check<E: BabyStepDispatchedEmitter + ?Sized>(
    step: &crate::baby_step::BabyStep,
    registry: &crate::checker_registry::CheckerRegistry,
    emitter: &E,
    now_ms: i64,
) -> Result<crate::baby_step::DispatchOutcome, OperationalError> {
    let outcome = crate::baby_step::dispatch_check_for_baby_step(step, registry).await;
    let _ = emit_baby_step_dispatched(step, &outcome, emitter, now_ms).await;
    match &outcome {
        crate::baby_step::DispatchOutcome::NotOperationalized => {
            Err(OperationalError::NotOperationalized {
                step_id: step.step_id.clone(),
            })
        }
        crate::baby_step::DispatchOutcome::Inapplicable => Err(OperationalError::NonPassOutcome {
            step_id: step.step_id.clone(),
            outcome: "inapplicable".into(),
        }),
        crate::baby_step::DispatchOutcome::Dispatched { .. } => {
            if outcome.overall_pass() {
                Ok(outcome)
            } else {
                Err(OperationalError::NonPassOutcome {
                    step_id: step.step_id.clone(),
                    outcome: outcome.outcome_label().to_string(),
                })
            }
        }
    }
}

/// D-12 read surface — the most recent N `baby_step_dispatched` SSB
/// rows the dashboard `BabyStepPanel` renders.
#[must_use]
pub fn recent_baby_step_dispatches(
    viewer: &crate::ssb_viewer::SsbViewer,
    limit: usize,
) -> Vec<BabyStepDispatchedRow> {
    let mut out = Vec::new();
    match viewer.recent_rows_of_type("baby_step_dispatched", limit) {
        Ok(details) => {
            for d in details {
                if let Some(row) = baby_step_dispatched_detail_to_row(d) {
                    out.push(row);
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                action_type = "baby_step_dispatched",
                error = %e,
                "recent_baby_step_dispatches: viewer error"
            );
        }
    }
    out.sort_by(|a, b| b.ts_ms.cmp(&a.ts_ms));
    out.truncate(limit);
    out
}

/// Typed projection of a single `baby_step_dispatched` SSB row.
/// Same shape as [`crate::ssb::BabyStepDispatchedPayload`] but lives
/// next to the other agent_pool read surfaces ([`CheckerResultRow`])
/// for the Tauri command layer.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BabyStepDispatchedRow {
    pub step_id: String,
    pub parent_task_id: String,
    pub checker_kinds_used: Vec<String>,
    pub outcome: String,
    pub ts_ms: i64,
}

fn baby_step_dispatched_detail_to_row(
    d: crate::ssb_viewer::SsbActionDetail,
) -> Option<BabyStepDispatchedRow> {
    if d.action_type.as_str() != "baby_step_dispatched" {
        return None;
    }
    let outer = d.payload_json.as_object()?;
    let obj = if let Some(inner) = outer.get("BabyStepDispatched").and_then(|v| v.as_object()) {
        inner.clone()
    } else {
        outer.clone()
    };
    let kinds: Vec<String> = obj
        .get("checker_kinds_used")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    Some(BabyStepDispatchedRow {
        step_id: obj
            .get("step_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        parent_task_id: obj
            .get("parent_task_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        checker_kinds_used: kinds,
        outcome: obj
            .get("outcome")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        ts_ms: obj.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0),
    })
}

// ---------------------------------------------------------------------------
// D-16 — UniCR conformal-refusal dispatch (formal Mode E grounding).
//
// When the D-10 router picks GroundingMode::ConformalRefusal the
// orchestrator runs this producer. It accepts a pre-loaded
// `CalibrationCorpus`, computes tau via
// `crate::calibration::compute_tau_and_emit` (one
// `conformal_tau_computed` SSB row), then runs the per-claim decision
// via `decide_and_emit` (one `conformal_refusal_decided` SSB row), and
// returns the typed `RefusalDecision`.
// I-23: all calibration imports go through the `crate::calibration`
// facade — no consumer reaches `conformal_calibration` directly.
//
// Grounding: arXiv:2503.01345 (UniCR — unified conformal refusal) +
// Vovk et al. *Algorithmic Learning in a Random World* (2005) §2.2.
// ---------------------------------------------------------------------------

/// D-16 public producer surface. Single-shot dispatcher that wires the
/// `GroundingMode::ConformalRefusal` arm of the D-10 router to a live
/// calibration corpus.
///
/// Both emitters are injected so callers can share the production Ssb
/// writer adapter or supply test stubs. `now_ms` is injected per the
/// entropy-quarantine rule — matches [`route_claim`] +
/// [`dispatch_constructive_checkers`].
///
/// Returns the typed [`crate::calibration::RefusalDecision`]
/// on success. Tau-compute failures (insufficient corpus, bad epsilon)
/// propagate as [`crate::calibration::ConformalError`] — the
/// caller should fall back to `GroundingMode::ConservativeFallback`
/// per the D-10 routing table.
pub async fn dispatch_conformal_refusal<
    TE: crate::calibration::ConformalTauComputedEmitter + ?Sized,
    RE: crate::calibration::ConformalRefusalDecidedEmitter + ?Sized,
>(
    claim_id: &str,
    claim_score: f64,
    corpus: &crate::calibration::CalibrationCorpus,
    epsilon: f64,
    tau_emitter: &TE,
    decision_emitter: &RE,
    now_ms: i64,
) -> Result<
    crate::calibration::RefusalDecision,
    crate::calibration::ConformalError,
> {
    let (tau, calibration_id) = crate::calibration::compute_tau_and_emit(
        corpus,
        epsilon,
        tau_emitter,
        now_ms,
    )
    .await?;
    let decision = crate::calibration::decide_and_emit(
        claim_id,
        claim_score,
        tau,
        &calibration_id,
        decision_emitter,
        now_ms,
    )
    .await;
    Ok(decision)
}

#[cfg(test)]
mod d16_conformal_dispatch_tests {
    use super::*;
    use crate::calibration::{
        CalibrationCorpus, CalibrationTuple, CorpusSource, RefusalDecision,
        VecConformalRefusalDecidedEmitter, VecConformalTauComputedEmitter,
    };

    fn balanced_corpus(n_pairs: usize) -> CalibrationCorpus {
        let mut c = CalibrationCorpus::new(CorpusSource::HeldOut);
        for i in 0..n_pairs {
            c.append(CalibrationTuple::new(
                format!("t-{i}"),
                format!("c-{i}"),
                "true",
                true,
                0.9,
                1_700_000_000 + (i as i64),
            ))
            .expect("append");
            c.append(CalibrationTuple::new(
                format!("f-{i}"),
                format!("d-{i}"),
                "false",
                false,
                0.2,
                1_700_000_500 + (i as i64),
            ))
            .expect("append");
        }
        c
    }

    #[tokio::test]
    async fn low_score_routes_to_abstain_and_emits_two_rows() {
        let corpus = balanced_corpus(20);
        let tau_emitter = VecConformalTauComputedEmitter::new();
        let dec_emitter = VecConformalRefusalDecidedEmitter::new();
        let d = dispatch_conformal_refusal(
            "claim-low",
            0.02,
            &corpus,
            0.1,
            &tau_emitter,
            &dec_emitter,
            1_700_000_900_000,
        )
        .await
        .expect("dispatch ok");
        assert!(matches!(d, RefusalDecision::Abstain));
        assert_eq!(tau_emitter.len().await, 1);
        assert_eq!(dec_emitter.len().await, 1);
        let dec_rows = dec_emitter.rows().await;
        assert_eq!(dec_rows[0].claim_id, "claim-low");
        assert_eq!(dec_rows[0].decision, "abstain");
    }

    #[tokio::test]
    async fn high_score_routes_to_answer_and_emits_two_rows() {
        let corpus = balanced_corpus(20);
        let tau_emitter = VecConformalTauComputedEmitter::new();
        let dec_emitter = VecConformalRefusalDecidedEmitter::new();
        let d = dispatch_conformal_refusal(
            "claim-high",
            0.97,
            &corpus,
            0.1,
            &tau_emitter,
            &dec_emitter,
            1_700_000_901_000,
        )
        .await
        .expect("dispatch ok");
        match d {
            RefusalDecision::Answer { confidence } => {
                assert!((confidence - 0.97).abs() < 1e-9);
            }
            RefusalDecision::Abstain => panic!("expected Answer"),
        }
        let dec_rows = dec_emitter.rows().await;
        assert_eq!(dec_rows[0].decision, "answer");
        assert!((dec_rows[0].confidence - 0.97).abs() < 1e-9);
    }

    #[tokio::test]
    async fn insufficient_corpus_propagates_error() {
        let corpus = balanced_corpus(2); // 4 tuples, below the 20-floor
        let tau_emitter = VecConformalTauComputedEmitter::new();
        let dec_emitter = VecConformalRefusalDecidedEmitter::new();
        let err = dispatch_conformal_refusal(
            "claim-tiny",
            0.5,
            &corpus,
            0.1,
            &tau_emitter,
            &dec_emitter,
            0,
        )
        .await
        .expect_err("must fail on tiny corpus");
        assert!(matches!(
            err,
            crate::calibration::ConformalError::InsufficientCorpus { .. }
        ));
        // No SSB rows emitted (the tau emit never fired).
        assert_eq!(tau_emitter.len().await, 0);
        assert_eq!(dec_emitter.len().await, 0);
    }
}

// ---------------------------------------------------------------------------
// D-15 — Discovery-vs-search labeling dispatcher (consumer)
//
// When the D-10 router routes a retained Mode A or Mode D result, the
// orchestrator calls this producer to attach a closed-taxonomy
// `NoveltyClass` label. The dispatcher delegates to the pure
// `crate::novelty::emit_novelty_classified` (one `novelty_classified`
// SSB row) and returns the typed `NoveltyClass`. Promotion gating —
// DiscoveryCandidate may require an additional adversarial verification
// pass per D-13 — is the responsibility of the caller; this dispatcher
// only classifies + emits.
//
// Grounding: arXiv:2406.05761 (LLM novelty detection three-bucket
// taxonomy); docs/specs/guide/real-problem-solving-grounded-2026-05-19.md
// §1 Mode C + §4.8.
// ---------------------------------------------------------------------------

/// D-15 public producer surface. Single-shot dispatcher that wires a
/// retained reasoning template through the
/// [`crate::novelty::classify`] predicate and emits one
/// `novelty_classified` SSB row.
///
/// `emitter` is injected so callers can share the production Ssb writer
/// adapter or supply test stubs. `now_ms` is injected per the
/// entropy-quarantine rule.
///
/// Returns the typed [`crate::novelty::NoveltyClass`]. The caller
/// inspects the variant: `DiscoveryCandidate` should trigger downstream
/// adversarial verification (D-13) before any promotion claim is
/// surfaced to the user (per real-problem §1 Mode C honesty layer).
pub async fn dispatch_novelty<E: crate::novelty::NoveltyClassifiedEmitter + ?Sized>(
    template: &crate::reasoning_template::ReasoningTemplate,
    archive: &crate::archive_chain::ArchiveChain,
    evidence: crate::novelty::NoveltyEvidence,
    config: crate::novelty::NoveltyConfig,
    emitter: &E,
    now_ms: i64,
) -> crate::novelty::NoveltyClass {
    crate::novelty::emit_novelty_classified(
        template,
        archive,
        evidence,
        config,
        emitter,
        now_ms,
    )
    .await
}

// ---------------------------------------------------------------------------
// D-17 — Honest-refusal SLA monitoring dispatcher (consumer)
//
// Driven periodically by the orchestration loop (or the `task_runner`
// in the production deployment — see the D-17 plan section 4). Calls
// the monitor's `emit_snapshot`, returns the typed
// `RefusalRateSnapshot`. The dispatcher does NOT decide alarm
// thresholds — `RefusalSlaConfig::alarm_threshold_breaches` is the
// caller-side cap and is consumed by the breach-counter the dashboard
// surfaces. Idempotency over rapid back-to-back ticks is also a
// caller-side concern.
//
// Grounding: arXiv:2503.01345 (UniCR honest refusal — formal Mode E
// surface the SLA monitors); docs/specs/guide/real-problem-solving-
// grounded-2026-05-19.md §4.6.
// ---------------------------------------------------------------------------

/// D-17 public producer surface. Single-shot dispatcher that calls
/// [`crate::refusal_sla::RefusalRateMonitor::emit_snapshot`] on the
/// supplied monitor.
///
/// `snapshot_emitter` + `breach_emitter` are injected so callers can
/// share the production Ssb writer adapter or supply test stubs.
/// `now_ms` is injected per the entropy-quarantine rule.
///
/// Returns the typed [`crate::refusal_sla::RefusalRateSnapshot`]. The
/// caller inspects `snapshot.health_band`:
/// `HealthBand::Below` / `HealthBand::Above` indicate an SLA breach
/// — the dispatcher already emitted the breach SSB row; downstream
/// alert wiring (Bubble-Up) is the caller's concern.
pub async fn dispatch_refusal_sla_check<
    SE: crate::refusal_sla::RefusalRateSnapshotEmitter + ?Sized,
    BE: crate::refusal_sla::RefusalSlaBreachEmitter + ?Sized,
>(
    monitor: &crate::refusal_sla::RefusalRateMonitor,
    snapshot_emitter: &SE,
    breach_emitter: &BE,
    now_ms: i64,
) -> crate::refusal_sla::RefusalRateSnapshot {
    monitor
        .emit_snapshot(snapshot_emitter, breach_emitter, now_ms)
        .await
}

#[cfg(test)]
mod d17_refusal_sla_dispatch_tests {
    use super::*;
    use crate::calibration::RefusalDecision;
    use crate::refusal_sla::{
        HealthBand, RefusalRateMonitor, RefusalSlaConfig, VecRefusalRateSnapshotEmitter,
        VecRefusalSlaBreachEmitter,
    };

    #[tokio::test]
    async fn dispatch_emits_snapshot_only_when_healthy() {
        let mut monitor = RefusalRateMonitor::with_defaults();
        // 10/90 → 10% inside healthy band [5%, 20%].
        for _ in 0..10 {
            monitor.observe(RefusalDecision::Abstain);
        }
        for _ in 0..90 {
            monitor.observe(RefusalDecision::Answer { confidence: 0.9 });
        }
        let snap_em = VecRefusalRateSnapshotEmitter::new();
        let breach_em = VecRefusalSlaBreachEmitter::new();
        let snap = dispatch_refusal_sla_check(
            &monitor,
            &snap_em,
            &breach_em,
            1_700_000_000_000,
        )
        .await;
        assert!(matches!(snap.health_band, HealthBand::Healthy));
        assert_eq!(snap_em.len().await, 1);
        assert_eq!(breach_em.len().await, 0);
    }

    #[tokio::test]
    async fn dispatch_emits_breach_above_band() {
        let cfg = RefusalSlaConfig::default();
        let mut monitor = RefusalRateMonitor::new(cfg).expect("ok");
        // 50/50 = 50% — far above healthy_band_hi (0.20).
        for _ in 0..50 {
            monitor.observe(RefusalDecision::Abstain);
        }
        for _ in 0..50 {
            monitor.observe(RefusalDecision::Answer { confidence: 0.9 });
        }
        let snap_em = VecRefusalRateSnapshotEmitter::new();
        let breach_em = VecRefusalSlaBreachEmitter::new();
        let snap = dispatch_refusal_sla_check(
            &monitor,
            &snap_em,
            &breach_em,
            1_700_000_000_000,
        )
        .await;
        assert!(matches!(snap.health_band, HealthBand::Above { .. }));
        assert_eq!(snap_em.len().await, 1);
        assert_eq!(breach_em.len().await, 1);
        let rows = breach_em.rows().await;
        assert_eq!(rows[0].breach_kind, "above");
        assert!((rows[0].rate - 0.5).abs() < 1e-9);
    }
}

#[cfg(test)]
mod d15_novelty_dispatch_tests {
    use super::*;
    use crate::novelty::{
        NoveltyClass, NoveltyConfig, NoveltyEvidence, VecNoveltyClassifiedEmitter,
    };
    use crate::reasoning_template::{
        compute_content_hash, ArchiveStatus, Domain, PatternKind, Predicate, ReasoningTemplate,
        TemplateBody, TemplateId, VerifierKind,
    };

    fn make_template(body: &str) -> ReasoningTemplate {
        let bounded = TemplateBody::try_new(body).expect("body fits");
        let hash = compute_content_hash(bounded.as_str(), None);
        ReasoningTemplate {
            id: TemplateId::new_v4(),
            domain: Domain::Code,
            pattern_kind: PatternKind::Decomposition,
            template_body: bounded,
            eligibility_predicate: Predicate::Always,
            verifier_required: VerifierKind::Refuse,
            content_hash: hash,
            created_at_ms: 1_700_000_000_000,
            parent_id: None,
            archive_status: ArchiveStatus::Promoted,
        }
    }

    #[tokio::test]
    async fn dispatch_high_match_routes_to_search_and_emits_row() {
        let t = make_template("body-search");
        let a = crate::archive_chain::ArchiveChain::new(
            std::path::PathBuf::from("test-archive.jsonl"),
        );
        let emitter = VecNoveltyClassifiedEmitter::new();
        let class = dispatch_novelty(
            &t,
            &a,
            NoveltyEvidence::new(10, 0.95, 0.0, 0.0),
            NoveltyConfig::default(),
            &emitter,
            1_700_000_500_000,
        )
        .await;
        assert_eq!(class, NoveltyClass::Search);
        assert_eq!(emitter.len().await, 1);
        let rows = emitter.rows().await;
        assert_eq!(rows[0].class, "search");
    }

    #[tokio::test]
    async fn dispatch_medium_evidence_routes_to_distant_search() {
        let t = make_template("body-distant");
        let a = crate::archive_chain::ArchiveChain::new(
            std::path::PathBuf::from("test-archive.jsonl"),
        );
        let emitter = VecNoveltyClassifiedEmitter::new();
        let class = dispatch_novelty(
            &t,
            &a,
            NoveltyEvidence::new(3, 0.55, 0.0, 0.0),
            NoveltyConfig::default(),
            &emitter,
            1_700_000_500_001,
        )
        .await;
        assert_eq!(class, NoveltyClass::DistantSearch);
        let rows = emitter.rows().await;
        assert_eq!(rows[0].class, "distant_search");
    }

    #[tokio::test]
    async fn dispatch_zero_match_low_duplicate_routes_to_discovery_candidate() {
        let t = make_template("body-discovery");
        let a = crate::archive_chain::ArchiveChain::new(
            std::path::PathBuf::from("test-archive.jsonl"),
        );
        let emitter = VecNoveltyClassifiedEmitter::new();
        let class = dispatch_novelty(
            &t,
            &a,
            NoveltyEvidence::new(0, 0.1, 0.0, 0.0),
            NoveltyConfig::default(),
            &emitter,
            1_700_000_500_002,
        )
        .await;
        assert_eq!(class, NoveltyClass::DiscoveryCandidate);
        let rows = emitter.rows().await;
        assert_eq!(rows[0].class, "discovery_candidate");
        assert_eq!(rows[0].dkg_match_count, 0);
    }
}

#[cfg(test)]
mod d12_dispatch_tests {
    use super::*;
    use crate::baby_step::{BabyStep, ExpectedKind, ExpectedResult, MeasurableProcedure};
    use crate::checker_registry::{CheckerKind, CheckerRegistry, NumericRangeChecker};
    use crate::claim::ClaimType;

    fn mk_step(claim_text: &str) -> BabyStep {
        BabyStep {
            step_id: "bs-d12-1".into(),
            parent_task_id: "task-d12".into(),
            description: "dispatch fixture".into(),
            measurable_procedure: Some(MeasurableProcedure {
                steps: vec!["compute".into()],
                success_criterion: "checker Pass".into(),
                checker_kind_hint: Some(CheckerKind::NumericRange),
                max_runtime_ms: 5_000,
            }),
            expected_result: Some(ExpectedResult {
                kind: ExpectedKind::BooleanFlag,
                value: serde_json::json!(true),
                tolerance: None,
            }),
            claim_payload: Some(Claim {
                claim_id: "claim-d12-1".into(),
                text: claim_text.into(),
                claim_type: ClaimType::Computational,
                evidence: vec![],
                ts_ms: 1_700_000_060_000,
            }),
        }
    }

    #[tokio::test]
    async fn run_operational_check_pass_emits_ssb_row() {
        let mut reg = CheckerRegistry::new();
        reg.register("nr-0-10", Box::new(NumericRangeChecker::new(0.0, 10.0)));
        let emitter = VecBabyStepDispatchedEmitter::new();
        let step = mk_step("5");
        let out = run_operational_check(&step, &reg, &emitter, 1_700_000_061_000)
            .await
            .expect("must pass");
        assert!(out.overall_pass());
        let rows = emitter.rows().await;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].outcome, "pass");
        assert_eq!(rows[0].checker_kinds_used, vec!["numeric_range".to_string()]);
        assert_eq!(rows[0].step_id, "bs-d12-1");
        assert_eq!(rows[0].parent_task_id, "task-d12");
    }

    #[tokio::test]
    async fn run_operational_check_not_operationalized_emits_row_and_errors() {
        let reg = CheckerRegistry::new();
        let emitter = VecBabyStepDispatchedEmitter::new();
        let step = BabyStep {
            step_id: "bs-d12-2".into(),
            parent_task_id: "task-d12".into(),
            description: "missing fields".into(),
            measurable_procedure: None,
            expected_result: None,
            claim_payload: None,
        };
        let err = run_operational_check(&step, &reg, &emitter, 1_700_000_062_000)
            .await
            .expect_err("must error");
        assert!(matches!(
            err,
            OperationalError::NotOperationalized { .. }
        ));
        let rows = emitter.rows().await;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].outcome, "inapplicable");
        assert!(rows[0].checker_kinds_used.is_empty());
    }

    #[tokio::test]
    async fn run_operational_check_fail_returns_non_pass() {
        let mut reg = CheckerRegistry::new();
        reg.register("nr-0-10", Box::new(NumericRangeChecker::new(0.0, 10.0)));
        let emitter = VecBabyStepDispatchedEmitter::new();
        // claim text "42" is out of [0, 10] -> Fail.
        let step = mk_step("42");
        let err = run_operational_check(&step, &reg, &emitter, 1_700_000_063_000)
            .await
            .expect_err("must error");
        match err {
            OperationalError::NonPassOutcome { outcome, .. } => {
                assert_eq!(outcome, "fail");
            }
            other => panic!("expected NonPassOutcome, got {other:?}"),
        }
        let rows = emitter.rows().await;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].outcome, "fail");
    }
}

#[cfg(test)]
mod d11_dispatch_tests {
    use super::*;
    use crate::checker_registry::{Checker, CheckerKind, CheckerRegistry, CheckerResult};
    use crate::claim::ClaimType;

    struct StaticChecker {
        kind: CheckerKind,
        result: CheckerResult,
    }

    impl Checker for StaticChecker {
        fn kind(&self) -> CheckerKind {
            self.kind
        }
        fn applies_to(&self, _claim: &Claim) -> bool {
            true
        }
        fn check<'a>(
            &'a self,
            _claim: &'a Claim,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = CheckerResult> + Send + 'a>> {
            let r = self.result.clone();
            Box::pin(async move { r })
        }
    }

    fn mk_claim(id: &str) -> Claim {
        Claim {
            claim_id: id.into(),
            text: "x = 1".into(),
            claim_type: ClaimType::Computational,
            evidence: vec![],
            ts_ms: 1_700_000_000_000,
        }
    }

    #[tokio::test]
    async fn dispatch_emits_one_row_per_result() {
        let mut reg = CheckerRegistry::new();
        reg.register(
            "a-pass",
            Box::new(StaticChecker {
                kind: CheckerKind::RegexMatch,
                result: CheckerResult::Pass,
            }),
        );
        reg.register(
            "b-fail",
            Box::new(StaticChecker {
                kind: CheckerKind::NumericRange,
                result: CheckerResult::Fail {
                    reason: "bad".into(),
                },
            }),
        );
        reg.register(
            "c-inapplicable",
            Box::new(StaticChecker {
                kind: CheckerKind::Z3,
                result: CheckerResult::Inapplicable {
                    reason: "skip".into(),
                },
            }),
        );

        let emitter = VecCheckerResultEmitter::new();
        let claim = mk_claim("c-d11");
        let out = dispatch_constructive_checkers(&claim, &reg, &emitter, 1_700_000_077_000).await;
        assert_eq!(out.len(), 3);

        let rows = emitter.rows().await;
        assert_eq!(rows.len(), 3);
        // Sorted by id -> a-pass, b-fail, c-inapplicable.
        assert_eq!(rows[0].checker_id, "a-pass");
        assert_eq!(rows[0].result_kind, "pass");
        assert_eq!(rows[0].reason, "");
        assert_eq!(rows[1].checker_id, "b-fail");
        assert_eq!(rows[1].result_kind, "fail");
        assert_eq!(rows[1].reason, "bad");
        assert_eq!(rows[2].checker_id, "c-inapplicable");
        assert_eq!(rows[2].result_kind, "inapplicable");
        // Every row inherits the same ts_ms + claim_id.
        for r in &rows {
            assert_eq!(r.ts_ms, 1_700_000_077_000);
            assert_eq!(r.claim_id, "c-d11");
        }
    }

    #[tokio::test]
    async fn dispatch_emits_zero_when_registry_empty() {
        let reg = CheckerRegistry::new();
        let emitter = VecCheckerResultEmitter::new();
        let claim = mk_claim("c-empty");
        let out = dispatch_constructive_checkers(&claim, &reg, &emitter, 1).await;
        assert!(out.is_empty());
        assert!(emitter.is_empty().await);
    }

    #[test]
    fn bounded_reason_truncates_at_2048() {
        let huge = "x".repeat(5000);
        let bounded = bounded_reason(&huge);
        assert_eq!(bounded.len(), 2048);
    }
}

#[cfg(test)]
mod d10_route_claim_tests {
    use super::*;
    use crate::claim::{ClaimType, EvidenceKind, EvidenceRef};
    use crate::orch_context::{OrchContext, OrchPhase, OrchUserRole};
    use crate::verification_result::VerifierKind;

    fn mk_claim(claim_type: ClaimType, with_evidence: bool, id: &str) -> Claim {
        let evidence = if with_evidence {
            vec![EvidenceRef::from_content(
                EvidenceKind::Corpus,
                "src/main.rs:1-10",
                b"fn main() {}",
            )]
        } else {
            vec![]
        };
        Claim {
            claim_id: id.into(),
            text: "test".into(),
            claim_type,
            evidence,
            ts_ms: 1_700_000_000_000,
        }
    }

    #[tokio::test]
    async fn route_claim_emits_one_row_per_call() {
        let emitter = VecClaimRoutedEmitter::new();
        let claim = mk_claim(ClaimType::Factual, true, "c-1");
        let mut ctx = OrchContext::minimal(OrchUserRole::Executor, OrchPhase::Executing);
        ctx.available_verifiers = vec![VerifierKind::Deterministic, VerifierKind::Intrinsic];
        let (mode, guard) = route_claim(&claim, &ctx, &emitter, 1_700_000_000_000).await;
        assert_eq!(mode, GroundingMode::Constructive);
        assert!(guard.status().is_ok());
        assert_eq!(emitter.len().await, 1);
        let _ = guard.confirm();
        let rows = emitter.rows().await;
        assert_eq!(rows[0].claim_id, "c-1");
        assert_eq!(rows[0].mode, "constructive");
        assert_eq!(rows[0].claim_type, "factual");
        assert_eq!(rows[0].ts_ms, 1_700_000_000_000);
        assert_eq!(rows[0].ctx_summary_blake3.len(), 64);
    }

    #[tokio::test]
    async fn route_claim_five_modes_smoke() {
        // observable_effect smoke per D-10 WIRING_CONTRACT — exactly
        // one row per claim, one row per expected mode.
        let emitter = VecClaimRoutedEmitter::new();
        struct Row {
            claim_type: ClaimType,
            with_evidence: bool,
            verifiers: &'static [VerifierKind],
            calibration_ready: bool,
            user_role: OrchUserRole,
            expected: GroundingMode,
        }
        let rows = [
            Row {
                claim_type: ClaimType::Factual,
                with_evidence: true,
                verifiers: &[VerifierKind::Deterministic, VerifierKind::Intrinsic],
                calibration_ready: false,
                user_role: OrchUserRole::Executor,
                expected: GroundingMode::Constructive,
            },
            Row {
                claim_type: ClaimType::Procedural,
                with_evidence: false,
                verifiers: &[VerifierKind::Deterministic],
                calibration_ready: false,
                user_role: OrchUserRole::Executor,
                expected: GroundingMode::OperationalDeferred,
            },
            Row {
                claim_type: ClaimType::Speculative,
                with_evidence: false,
                verifiers: &[VerifierKind::Intrinsic, VerifierKind::Ppm],
                calibration_ready: false,
                user_role: OrchUserRole::Executor,
                expected: GroundingMode::FalsifiabilityCrossFamily,
            },
            Row {
                claim_type: ClaimType::Speculative,
                with_evidence: false,
                verifiers: &[VerifierKind::Intrinsic],
                calibration_ready: true,
                user_role: OrchUserRole::Executor,
                expected: GroundingMode::ConformalRefusal,
            },
            Row {
                claim_type: ClaimType::Normative,
                with_evidence: true,
                verifiers: &[VerifierKind::Deterministic, VerifierKind::Intrinsic],
                calibration_ready: false,
                user_role: OrchUserRole::Executor,
                expected: GroundingMode::ConservativeFallback,
            },
        ];
        for (i, row) in rows.iter().enumerate() {
            let claim = mk_claim(row.claim_type, row.with_evidence, &format!("c-{i}"));
            let mut ctx = OrchContext::minimal(row.user_role, OrchPhase::Executing);
            ctx.available_verifiers = row.verifiers.to_vec();
            ctx.conformal_calibration_ready = row.calibration_ready;
            let (mode, guard) =
                route_claim(&claim, &ctx, &emitter, 1_700_000_000_000 + i as i64).await;
            assert_eq!(mode, row.expected, "row {i} expected {:?}", row.expected);
            let _ = guard.confirm();
        }
        assert_eq!(emitter.len().await, 5);
        let by_mode = emitter.rows().await;
        assert_eq!(by_mode.len(), 5);
        // Each row carries the expected mode label.
        for (i, payload) in by_mode.iter().enumerate() {
            assert_eq!(payload.mode, rows[i].expected.as_str());
        }
    }

    #[tokio::test]
    async fn route_claim_ctx_blake3_is_64_hex_chars() {
        let emitter = VecClaimRoutedEmitter::new();
        let claim = mk_claim(ClaimType::Factual, true, "c-hash");
        let ctx = OrchContext::minimal(OrchUserRole::Executor, OrchPhase::Executing);
        let (_, guard) = route_claim(&claim, &ctx, &emitter, 1).await;
        let _ = guard.confirm();
        let rows = emitter.rows().await;
        let hex = &rows[0].ctx_summary_blake3;
        assert_eq!(hex.len(), 64, "blake3 hex must be 64 chars (got {})", hex.len());
        assert!(
            hex.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "blake3 hex must be lowercase: {hex}"
        );
    }
}

// ---------------------------------------------------------------------------
// E-46 — Hierarchical decomposition tuning producer wiring.
// ---------------------------------------------------------------------------
//
// Three additive free functions sit alongside D-7's `recent_canary_status`
// + D-10's `route_claim` to keep the orchestrator-side producer surface
// uniform. The pure ledger / shortcut / canary modules under
// `crate::dual_ledger`, `crate::agentless_shortcut`,
// `crate::goal_drift_canary` stay free of `Ssb` references — emission
// happens here at the agent_pool boundary.
//
// Grounding (per module docstrings):
//   * arxiv:2412.21154 (Devin task-ledger pattern)
//   * arxiv:2407.01489 (AGENTLESS leaf shortcut)
//   * arxiv:2505.02709 (goal-drift PRM supervision)

use crate::agentless_shortcut::ShortcutReason;
use crate::dual_ledger::{
    DualLedger, DualLedgerError, DualLedgerSnapshot, TaskAdvancement, TaskId,
};
use crate::goal_drift_canary::GoalDriftCanary;
use crate::ssb::{
    AgentlessShortcutTakenPayload, GoalDriftFiredPayload, Ssb, SsbPayload, TaskAdvancedPayload,
};

/// E-46 — emit a `task_advanced` SSB row from a [`TaskAdvancement`] view.
/// Caller-supplied `superstep` matches the rest of the orchestrator's
/// emission convention (the BSP layer threads the value through).
/// Failures to write are logged at warn and dropped — the ledger
/// mutation has already happened and the dashboard signal is advisory.
pub async fn emit_task_advanced(
    advancement: &TaskAdvancement,
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
) {
    let payload = SsbPayload::TaskAdvanced(TaskAdvancedPayload {
        task_id: advancement.task_id.0.clone(),
        parent_id: advancement
            .parent_id
            .as_ref()
            .map(|p| p.0.clone())
            .unwrap_or_default(),
        completion_pct: advancement.completion_pct,
        status: advancement.status.as_str().to_owned(),
        blocker: advancement.blocker.clone(),
        ts_ms: advancement.ts_ms,
    });
    if let Err(e) = ssb
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await
    {
        tracing::warn!(
            error = %e,
            task_id = %advancement.task_id.0,
            "agent_pool::emit_task_advanced: SSB write failed"
        );
    }
}

/// E-46 — emit an `agentless_shortcut_taken` SSB row. Caller has
/// already confirmed eligibility via
/// `agentless_shortcut::shortcut_eligible` returning `Yes(reason)`.
pub async fn emit_agentless_shortcut_taken(
    task_id: &TaskId,
    reason: ShortcutReason,
    ts_ms: i64,
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
) {
    let payload = SsbPayload::AgentlessShortcutTaken(AgentlessShortcutTakenPayload {
        task_id: task_id.0.clone(),
        reason: reason.as_str().to_owned(),
        ts_ms,
    });
    if let Err(e) = ssb
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await
    {
        tracing::warn!(
            error = %e,
            task_id = %task_id.0,
            "agent_pool::emit_agentless_shortcut_taken: SSB write failed"
        );
    }
}

/// E-46 — emit a `goal_drift_fired` SSB row. Caller confirmed the
/// canary's `fired()` predicate already returned `true`; we accept the
/// drift + step + threshold values from the caller so the SSB row
/// matches the canary's view exactly.
pub async fn emit_goal_drift_fired(
    canary: &GoalDriftCanary,
    drift: f64,
    step: u32,
    ts_ms: i64,
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
) {
    let payload = SsbPayload::GoalDriftFired(GoalDriftFiredPayload {
        initial_goal_blake3: canary.initial_goal_blake3.clone(),
        drift,
        threshold: canary.drift_threshold,
        step,
        ts_ms,
    });
    if let Err(e) = ssb
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await
    {
        tracing::warn!(
            error = %e,
            initial_goal_blake3 = %canary.initial_goal_blake3,
            "agent_pool::emit_goal_drift_fired: SSB write failed"
        );
    }
}

/// E-46 — composite producer: advance a task in the supplied ledger
/// AND emit the corresponding `task_advanced` SSB row. The single
/// public entry point the task_runner / TreeNavigator uses so callers
/// can't forget the emission. Returns the post-advance view (or the
/// ledger error if the task isn't registered).
///
/// Caller passes a `&mut DualLedger` (orchestrator session state) +
/// an `Arc<Ssb>` for emission. The function is otherwise pure async
/// glue.
pub async fn process_task_advancement(
    ledger: &mut DualLedger,
    task_id: &TaskId,
    completion_delta: f64,
    blocker: Option<String>,
    ts_ms: i64,
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
) -> Result<TaskAdvancement, DualLedgerError> {
    let advancement = ledger.advance_task(task_id, completion_delta, blocker, ts_ms)?;
    emit_task_advanced(&advancement, ssb, agent_id, superstep).await;
    Ok(advancement)
}

/// E-46 — read surface for the dashboard `get_dual_ledger_state`
/// command. Returns a cheap clone of the snapshot — the orchestrator
/// session owns the ledger; this helper just serialises it.
#[must_use]
pub fn ledger_snapshot(ledger: &DualLedger) -> DualLedgerSnapshot {
    ledger.snapshot()
}

/// E-46 — typed summary of promotion-eligible tasks. Used by the BAI
/// promotion-gate to filter candidate tasks. Returns the task ids
/// whose progress ledger has reached `100.0`.
#[must_use]
pub fn promotion_eligible_tasks(ledger: &DualLedger) -> Vec<TaskId> {
    ledger
        .iter()
        .filter(|(_, p)| p.completion_pct >= 100.0)
        .map(|(t, _)| t.task_id.clone())
        .collect()
}

/// E-46 — BAI consumer: returns true when goal-drift is currently
/// firing for the supplied canary + summary. Used by the BAI gate to
/// block promotion when drift is active (companion to
/// `CanaryStatusSummary::blocks_promotion`).
#[must_use]
pub fn goal_drift_blocks_promotion(canary: &GoalDriftCanary, current_summary: &str) -> bool {
    canary.fired(current_summary)
}

#[cfg(test)]
mod e46_producer_tests {
    use super::*;
    use crate::agentless_shortcut::{shortcut_eligible, Eligibility};
    use crate::dual_ledger::{TaskLedger, TaskStatus};
    use crate::ssb::{SsbActionType, SsbConfig, SsbReader};
    use elai_core::clock::{ClockProvider, SeededClock};
    use tempfile::TempDir;

    fn test_ssb() -> (Ssb, TempDir) {
        let dir = TempDir::new().expect("tempdir");
        let cfg = SsbConfig {
            db_path: dir.path().join("ssb.db"),
            pool_size: Some(2),
            project_id: "p-test".into(),
            session_id: "s-test".into(),
        };
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let ssb = Ssb::new(cfg, clock).expect("ssb init");
        (ssb, dir)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn process_task_advancement_emits_row() {
        let (ssb, _dir) = test_ssb();
        let ssb = Arc::new(ssb);
        let mut ledger = DualLedger::new();
        ledger.register_task(TaskLedger::new(
            TaskId::from("t-1"),
            None,
            vec!["criterion".into()],
            1,
        ));

        let adv = process_task_advancement(
            &mut ledger,
            &TaskId::from("t-1"),
            50.0,
            None,
            2,
            &ssb,
            "agent-test",
            0,
        )
        .await
        .expect("advancement ok");
        assert_eq!(adv.completion_pct, 50.0);
        assert_eq!(adv.status, TaskStatus::InProgress);
        let rows = ssb.query_by_type(SsbActionType::TaskAdvanced).expect("query");
        assert_eq!(rows.len(), 1, "exactly one task_advanced row");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn emit_agentless_shortcut_writes_row() {
        let (ssb, _dir) = test_ssb();
        let ssb = Arc::new(ssb);
        let t = TaskLedger::new(
            TaskId::from("t-1"),
            None,
            vec!["file=crates/foo/src/a.rs".into(), "loc<=10".into()],
            1,
        );
        let elig = shortcut_eligible(&t);
        let reason = match elig {
            Eligibility::Yes(r) => r,
            Eligibility::No(s) => panic!("expected Yes, got No({s})"),
        };
        emit_agentless_shortcut_taken(&t.task_id, reason, 2, &ssb, "agent-test", 0).await;
        let rows = ssb
            .query_by_type(SsbActionType::AgentlessShortcutTaken)
            .expect("query");
        assert_eq!(rows.len(), 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn emit_goal_drift_writes_row() {
        let (ssb, _dir) = test_ssb();
        let ssb = Arc::new(ssb);
        let canary = GoalDriftCanary::new("alpha beta gamma", vec![1, 5], 0.4);
        // Drifted summary — orthogonal tokens.
        let drifted = "delta epsilon zeta";
        assert!(canary.fired(drifted));
        let drift = canary.measure_drift(drifted);
        emit_goal_drift_fired(&canary, drift, 5, 2, &ssb, "agent-test", 0).await;
        let rows = ssb
            .query_by_type(SsbActionType::GoalDriftFired)
            .expect("query");
        assert_eq!(rows.len(), 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn observable_effect_smoke_three_tasks_one_shortcut_one_drift() {
        // E-46 observable_effect: create parent + 2 children, advance
        // them, take 1 agentless shortcut, run measure_drift on a
        // deliberately-drifted summary, assert all 3 SSB rows landed.
        let (ssb, _dir) = test_ssb();
        let ssb = Arc::new(ssb);
        let mut ledger = DualLedger::new();
        // Parent + 2 children.
        ledger.register_task(TaskLedger::new(
            TaskId::from("parent"),
            None,
            vec!["coordinate children".into()],
            1,
        ));
        ledger.register_task(TaskLedger::new(
            TaskId::from("child-1"),
            Some(TaskId::from("parent")),
            vec![
                "file=crates/orchestrator/src/agent_pool.rs".into(),
                "loc<=20".into(),
            ],
            1,
        ));
        ledger.register_task(TaskLedger::new(
            TaskId::from("child-2"),
            Some(TaskId::from("parent")),
            vec!["broad refactor across crates".into()],
            1,
        ));

        // Advance all three tasks — emits 3 task_advanced rows.
        for tid in ["parent", "child-1", "child-2"] {
            let _ = process_task_advancement(
                &mut ledger,
                &TaskId::from(tid),
                25.0,
                None,
                2,
                &ssb,
                "agent-test",
                0,
            )
            .await
            .expect("advance");
        }
        // child-1 is shortcut-eligible (single-file + small LOC).
        let child1 = ledger.task(&TaskId::from("child-1")).expect("present");
        match shortcut_eligible(child1) {
            Eligibility::Yes(r) => {
                emit_agentless_shortcut_taken(
                    &child1.task_id,
                    r,
                    3,
                    &ssb,
                    "agent-test",
                    0,
                )
                .await;
            }
            Eligibility::No(s) => panic!("expected Yes for child-1, got No({s})"),
        }
        // Drift canary on the parent goal vs an orthogonal summary.
        let canary = GoalDriftCanary::new("coordinate children", vec![1, 5], 0.4);
        let drifted_summary = "completely unrelated tokens";
        assert!(canary.fired(drifted_summary));
        let drift = canary.measure_drift(drifted_summary);
        emit_goal_drift_fired(&canary, drift, 5, 4, &ssb, "agent-test", 0).await;

        // Assert: 3 task_advanced + 1 agentless_shortcut_taken + 1
        // goal_drift_fired.
        let adv_rows = ssb
            .query_by_type(SsbActionType::TaskAdvanced)
            .expect("query");
        assert_eq!(adv_rows.len(), 3, "3 task_advanced rows");
        let sc_rows = ssb
            .query_by_type(SsbActionType::AgentlessShortcutTaken)
            .expect("query");
        assert_eq!(sc_rows.len(), 1, "1 agentless_shortcut_taken row");
        let gd_rows = ssb
            .query_by_type(SsbActionType::GoalDriftFired)
            .expect("query");
        assert_eq!(gd_rows.len(), 1, "1 goal_drift_fired row");

        // Ledger consumer surface: promotion_eligible_tasks
        // (none yet — all tasks are at 25%).
        assert!(promotion_eligible_tasks(&ledger).is_empty());
        // Advance child-1 to 100%.
        let _ = process_task_advancement(
            &mut ledger,
            &TaskId::from("child-1"),
            75.0,
            None,
            5,
            &ssb,
            "agent-test",
            0,
        )
        .await
        .expect("advance");
        let eligible = promotion_eligible_tasks(&ledger);
        assert_eq!(eligible, vec![TaskId::from("child-1")]);

        // goal_drift_blocks_promotion exercises the BAI consumer side.
        assert!(goal_drift_blocks_promotion(&canary, drifted_summary));
        assert!(!goal_drift_blocks_promotion(&canary, "coordinate children"));

        // Snapshot round-trip.
        let snap = ledger_snapshot(&ledger);
        assert_eq!(snap.tasks.len(), 3);
        assert_eq!(snap.progress.len(), 3);
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
        // base-phase-2 (2C): canonical Designer chain per spec phase-2C.txt T14
        // is [Observe, Hypothesize, PlanAction, Verify, Converge] (5 steps,
        // ends in Converge, no Act — Designer emits a PipelineSpec, it does not
        // mutate). Executor is [Observe, Hypothesize, PlanAction, Act, Verify]
        // (5 steps, has Act, no Converge). Still distinct: Designer has Converge
        // and no Act; Executor has Act and no Converge.
        assert_ne!(designer.required_steps, executor.required_steps);
        assert_eq!(
            designer.required_steps,
            vec![
                GrcState::Observe,
                GrcState::Hypothesize,
                GrcState::PlanAction,
                GrcState::Verify,
                GrcState::Converge,
            ]
        );
        assert_eq!(
            executor.required_steps,
            vec![
                GrcState::Observe,
                GrcState::Hypothesize,
                GrcState::PlanAction,
                GrcState::Act,
                GrcState::Verify,
            ]
        );
        // Designer ends in Converge and has no Act; Executor is the inverse.
        assert!(designer.required_steps.contains(&GrcState::Converge));
        assert!(!designer.required_steps.contains(&GrcState::Act));
        assert!(executor.required_steps.contains(&GrcState::Act));
        assert!(!executor.required_steps.contains(&GrcState::Converge));
        // Both spec chains must pass eager validation (2C
        // validate_config-at-construction). default_grc_config already
        // validates internally; assert explicitly here as a regression guard.
        let loader = TomlConfigLoader::new();
        loader
            .validate_config(&designer)
            .expect("canonical designer config must validate");
        loader
            .validate_config(&executor)
            .expect("canonical executor config must validate");
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

    #[test]
    fn auditor_reserve_refuses_orchestrator_raid() {
        let mut pool = test_pool(u64::MAX);
        let result = pool.try_reallocate_from_auditor_reserve(1, 0);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("financial independence") || msg.contains("cannot be raided"),
            "expected financial-independence message, got: {msg}"
        );
    }

    #[test]
    fn auditor_reserve_refuse_raid_zero_is_noop() {
        let pool = test_pool(u64::MAX);
        let result = pool.try_reallocate_from_auditor_reserve(0, 0);
        assert!(result.is_ok(), "zero-zero raid should be a no-op Ok, got: {result:?}");
    }

    #[test]
    fn auditor_reserve_charge_ok() {
        let mut pool = test_pool(u64::MAX);
        let before = pool.auditor_budget_reserve.remaining_calls();
        assert_eq!(before, 10);
        let result = pool.auditor_budget_reserve.charge(1, 0);
        assert!(result.is_ok());
        let after = pool.auditor_budget_reserve.remaining_calls();
        assert_eq!(after, 9);
    }
}
