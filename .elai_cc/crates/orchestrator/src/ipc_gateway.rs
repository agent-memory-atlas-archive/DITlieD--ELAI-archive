// IPC Gateway — orchestrator-side IPC connection management and frame routing.
//
// Responsibilities:
// - Listen-before-spawn: create socket, bind, then spawn agent
// - Accept agent connections with SO_PEERCRED verification (Linux)
// - Route incoming IPC frames to appropriate handlers
// - GRC budget counting (per agent per topic, advisory)
// - Grounding method matrix enforcement
// - ReplanRequest budget and dedup
// - SSB write synchronous with ToolResult dispatch
// - Self-modification lock enforcement on ToolRequest/BashRequest
//
// Source: phase-4A.txt, orchestrator.txt S12.
//
// Production wiring scheduled in Session wire-4A per phase-4A.txt L259-L280
// ("IMPLEMENT — agent binary + orchestrator IPC gateway"). Present callers
// are in tests/ directories (agent_isolation.rs, ipc_simulation.rs,
// adversarial_ipc.rs) validating the spec contract as TDD red-phase. Do not
// remove: deletion would require re-implementing from scratch when wire-4A
// lands. This module consumes IpcHookTrait/IpcHookError/HookResult from
// elai_plugins::ipc_hooks — that module is retained for the same reason.

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use uuid::Uuid;

use elai_ipc_types::{
    AgentRole, IpcFrame, IpcFrameError, IpcMessageType,
    FRAME_HEADER_SIZE, IPC_PROTOCOL_VERSION, MAX_FRAME_SIZE,
    AGENT_HEARTBEAT_TIMEOUT_SECS,
};
use elai_ipc_types::ToolFilter;
use crate::sandbox::SelfModificationLocks;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum ReplanRequests per agent per session.
const MAX_REPLAN_REQUESTS_PER_AGENT: u32 = 3;

/// Maximum grounding requests per agent per session.
const MAX_GROUNDING_REQUESTS_PER_AGENT: u32 = 50;

/// Maximum InferRequest.max_tokens the orchestrator will forward to the LLM.
const MAX_INFER_TOKENS_CAP: u32 = 8192;

// ---------------------------------------------------------------------------
// GroundingMethod — methods available for grounding requests
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroundingMethod {
    FileCheck,
    WebCheck,
    SourceCheck,
}

// ---------------------------------------------------------------------------
// GrcBudgetEntry — per-agent per-topic GRC budget tracking
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct GrcBudgetEntry {
    calls_used: u32,
    max_calls: u32,
}

// ---------------------------------------------------------------------------
// AgentConnection — per-agent connection state
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct AgentConnection {
    pub agent_id: Uuid,
    pub role: AgentRole,
    pub process_id: u32,
    pub socket_path: PathBuf,
    pub last_heartbeat: std::time::Instant,
    grc_budgets: HashMap<String, GrcBudgetEntry>,
    grc_max_calls: u32,
    replan_count: u32,
    last_replan_reason: Option<String>,
    grounding_count: u32,
    active_think_topics: Vec<String>,
    tool_filter: Option<ToolFilter>,
}

impl AgentConnection {
    pub fn new(
        agent_id: Uuid,
        role: AgentRole,
        process_id: u32,
        socket_path: PathBuf,
        grc_max_calls: u32,
    ) -> Self {
        Self {
            agent_id,
            role,
            process_id,
            socket_path,
            last_heartbeat: std::time::Instant::now(),
            grc_budgets: HashMap::new(),
            grc_max_calls,
            replan_count: 0,
            last_replan_reason: None,
            grounding_count: 0,
            active_think_topics: Vec::new(),
            tool_filter: None,
        }
    }

    /// Set the tool filter for this agent connection.
    pub fn set_tool_filter(&mut self, filter: ToolFilter) {
        self.tool_filter = Some(filter);
    }

    /// Check if a tool is allowed by the agent's tool filter.
    /// Returns None if no filter is set (all tools allowed).
    pub fn check_tool_allowed(&self, tool_id: &elai_ipc_types::ToolId) -> bool {
        match &self.tool_filter {
            Some(filter) => filter.allows(tool_id),
            None => true, // No filter = all allowed
        }
    }

    /// Record a ThinkStep and return whether it exceeds budget (advisory).
    pub fn record_think_step(&mut self, topic: &str) -> bool {
        let entry = self.grc_budgets.entry(topic.to_string()).or_insert(GrcBudgetEntry {
            calls_used: 0,
            max_calls: self.grc_max_calls,
        });
        entry.calls_used += 1;

        // Track active topics for grounding orphan detection
        if !self.active_think_topics.contains(&topic.to_string()) {
            self.active_think_topics.push(topic.to_string());
        }

        entry.calls_used > entry.max_calls
    }

    /// Check if a topic has an active ThinkStep chain.
    pub fn has_active_topic(&self, topic: &str) -> bool {
        self.active_think_topics.contains(&topic.to_string())
    }

    /// Record a converge step — marks topic as converged.
    pub fn record_converge(&mut self, topic: &str) {
        self.active_think_topics.retain(|t| t != topic);
    }

    /// Check and record a ReplanRequest. Returns None if allowed, Some(reason) if blocked.
    pub fn check_replan(&mut self, reason: &str) -> Option<String> {
        // Budget check
        if self.replan_count >= MAX_REPLAN_REQUESTS_PER_AGENT {
            return Some(format!(
                "replan budget exhausted: {}/{} used",
                self.replan_count, MAX_REPLAN_REQUESTS_PER_AGENT
            ));
        }
        // Dedup check
        if self.last_replan_reason.as_deref() == Some(reason) {
            return Some(format!("duplicate replan reason: {reason}"));
        }
        self.replan_count += 1;
        self.last_replan_reason = Some(reason.to_string());
        None
    }

    /// Check and record a grounding request. Returns None if allowed, Some(reason) if blocked.
    pub fn check_grounding(&mut self, topic: &str, method: GroundingMethod) -> Option<String> {
        // Budget check
        if self.grounding_count >= MAX_GROUNDING_REQUESTS_PER_AGENT {
            return Some(format!(
                "grounding budget exhausted: {}/{} used",
                self.grounding_count, MAX_GROUNDING_REQUESTS_PER_AGENT
            ));
        }
        // Orphaned topic check
        if !self.has_active_topic(topic) {
            return Some(format!("orphaned topic: no active ThinkStep chain for '{topic}'"));
        }
        // Role-method matrix check
        if !allowed_grounding_methods(&self.role).contains(&method) {
            return Some(format!(
                "grounding method {method:?} not allowed for role {:?}",
                self.role
            ));
        }
        self.grounding_count += 1;
        None
    }

    /// Update heartbeat timestamp.
    pub fn touch_heartbeat(&mut self) {
        self.last_heartbeat = std::time::Instant::now();
    }

    /// Check if heartbeat has timed out.
    pub fn is_heartbeat_timeout(&self) -> bool {
        self.last_heartbeat.elapsed().as_secs() >= AGENT_HEARTBEAT_TIMEOUT_SECS
    }
}

// ---------------------------------------------------------------------------
// Role-method matrix for grounding
// ---------------------------------------------------------------------------

pub fn allowed_grounding_methods(role: &AgentRole) -> Vec<GroundingMethod> {
    match role {
        AgentRole::Executor => vec![GroundingMethod::FileCheck],
        AgentRole::Researcher => vec![GroundingMethod::FileCheck, GroundingMethod::WebCheck],
        AgentRole::Auditor => vec![
            GroundingMethod::FileCheck,
            GroundingMethod::WebCheck,
            GroundingMethod::SourceCheck,
        ],
        AgentRole::Designer => vec![GroundingMethod::FileCheck],
        AgentRole::PromptEngineer => vec![GroundingMethod::FileCheck],
        AgentRole::ContextCompiler => vec![GroundingMethod::FileCheck, GroundingMethod::SourceCheck],
        AgentRole::MemoryWriter => vec![GroundingMethod::FileCheck],
        AgentRole::MediaValidator => vec![GroundingMethod::FileCheck],
    }
}

// ---------------------------------------------------------------------------
// FrameRouteResult — outcome of routing a single frame
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum FrameRouteResult {
    /// Agent responded normally
    Response(IpcMessageType),
    /// Agent sent a heartbeat (no response needed)
    Heartbeat,
    /// Agent requested inference (proxy to LLM)
    InferRequest { prompt: String, max_tokens: u32 },
    /// Agent requested a tool call
    ToolRequest { tool_id: String, arguments: Vec<u8> },
    /// Agent requested bash execution
    BashRequest { command: String, timeout_ms: u64 },
    /// Agent requested grounding
    GroundingRequest { topic: String, query: String },
    /// Agent sent a ThinkStep (logged, advisory budget)
    ThinkStep { topic: String, step_type: String, over_budget: bool },
    /// Agent sent a checkpoint
    Checkpoint { sequence: u64 },
    /// Agent requested replan
    ReplanRequest { reason: String, severity: String },
    /// Frame was rejected (error message)
    Rejected(String),
}

// ---------------------------------------------------------------------------
// IpcGateway
// ---------------------------------------------------------------------------

pub struct IpcGateway {
    connections: HashMap<Uuid, AgentConnection>,
    self_mod_locks: SelfModificationLocks,
    session_socket_dir: PathBuf,
}

impl IpcGateway {
    pub fn new(session_socket_dir: PathBuf, self_mod_locks: SelfModificationLocks) -> Self {
        Self {
            connections: HashMap::new(),
            self_mod_locks,
            session_socket_dir,
        }
    }

    /// Register a new agent connection.
    pub fn register_agent(&mut self, conn: AgentConnection) {
        tracing::info!(
            agent_id = %conn.agent_id,
            role = ?conn.role,
            pid = conn.process_id,
            "registered agent connection"
        );
        self.connections.insert(conn.agent_id, conn);
    }

    /// Remove an agent connection (on death/cleanup).
    pub fn remove_agent(&mut self, agent_id: &Uuid) -> Option<AgentConnection> {
        let conn = self.connections.remove(agent_id);
        if let Some(ref c) = conn {
            // Clean up socket file
            if c.socket_path.exists() {
                if let Err(e) = std::fs::remove_file(&c.socket_path) {
                    tracing::warn!(
                        agent_id = %agent_id,
                        path = %c.socket_path.display(),
                        error = %e,
                        "failed to remove agent socket"
                    );
                }
            }
        }
        conn
    }

    /// Get a reference to an agent connection.
    pub fn get_agent(&self, agent_id: &Uuid) -> Option<&AgentConnection> {
        self.connections.get(agent_id)
    }

    /// Get a mutable reference to an agent connection.
    pub fn get_agent_mut(&mut self, agent_id: &Uuid) -> Option<&mut AgentConnection> {
        self.connections.get_mut(agent_id)
    }

    /// Generate socket path for an agent.
    pub fn socket_path_for(&self, agent_id: &Uuid) -> PathBuf {
        self.session_socket_dir.join(format!("{agent_id}.sock"))
    }

    /// Get the session socket directory.
    pub fn session_socket_dir(&self) -> &Path {
        &self.session_socket_dir
    }

    /// Check all connections for heartbeat timeouts. Returns IDs of timed-out agents.
    pub fn check_heartbeat_timeouts(&self) -> Vec<Uuid> {
        self.connections
            .iter()
            .filter(|(_, conn)| conn.is_heartbeat_timeout())
            .map(|(id, _)| *id)
            .collect()
    }

    /// Route an incoming IPC frame from an agent. Returns the routing result.
    pub fn route_frame(
        &mut self,
        agent_id: &Uuid,
        msg: IpcMessageType,
    ) -> FrameRouteResult {
        let conn = match self.connections.get_mut(agent_id) {
            Some(c) => c,
            None => return FrameRouteResult::Rejected(format!("unknown agent: {agent_id}")),
        };

        match msg {
            IpcMessageType::AgentResponse { content, artifacts } => {
                FrameRouteResult::Response(IpcMessageType::AgentResponse { content, artifacts })
            }

            IpcMessageType::AgentHeartbeat { agent_id: hb_id, uptime_secs: _ } => {
                if hb_id != conn.agent_id {
                    return FrameRouteResult::Rejected(format!(
                        "heartbeat agent_id mismatch: expected {}, got {hb_id}",
                        conn.agent_id
                    ));
                }
                conn.touch_heartbeat();
                FrameRouteResult::Heartbeat
            }

            IpcMessageType::InferRequest { prompt, max_tokens } => {
                // Cap max_tokens per BLOCK-3/BLOCK-4
                let capped = max_tokens.min(MAX_INFER_TOKENS_CAP);
                if capped < max_tokens {
                    tracing::warn!(
                        agent_id = %conn.agent_id,
                        requested = max_tokens,
                        capped = capped,
                        "capped InferRequest max_tokens"
                    );
                }
                FrameRouteResult::InferRequest { prompt, max_tokens: capped }
            }

            IpcMessageType::ToolRequest { tool_id, arguments } => {
                // Check tool against agent's allowed tool filter
                if !conn.check_tool_allowed(&tool_id) {
                    return FrameRouteResult::Rejected(format!(
                        "tool '{}' not in agent's allowed tool filter",
                        tool_id.as_str()
                    ));
                }
                FrameRouteResult::ToolRequest {
                    tool_id: tool_id.as_str().to_string(),
                    arguments,
                }
            }

            IpcMessageType::BashRequest { command, timeout_ms } => {
                // Self-mod lock check on command target
                // Actual path extraction from command is done at bash execution layer
                FrameRouteResult::BashRequest { command, timeout_ms }
            }

            IpcMessageType::GroundingRequest { topic, query } => {
                // Determine grounding method from query prefix
                let method = if query.starts_with("FILE_CHECK") {
                    GroundingMethod::FileCheck
                } else if query.starts_with("WEB_CHECK") {
                    GroundingMethod::WebCheck
                } else if query.starts_with("SOURCE_CHECK") {
                    GroundingMethod::SourceCheck
                } else {
                    GroundingMethod::FileCheck // default
                };

                if let Some(reject_reason) = conn.check_grounding(&topic, method) {
                    return FrameRouteResult::Rejected(reject_reason);
                }
                FrameRouteResult::GroundingRequest { topic, query }
            }

            IpcMessageType::ThinkStep { topic, step_type, content: _ } => {
                let over_budget = conn.record_think_step(&topic);
                if step_type == "converge" {
                    conn.record_converge(&topic);
                }
                if over_budget {
                    tracing::warn!(
                        agent_id = %conn.agent_id,
                        topic = %topic,
                        "GRC budget exceeded (advisory)"
                    );
                }
                FrameRouteResult::ThinkStep { topic, step_type, over_budget }
            }

            IpcMessageType::CheckpointSync { agent_id: _, sequence, payload: _ } => {
                FrameRouteResult::Checkpoint { sequence }
            }

            IpcMessageType::ReplanRequest { reason, severity } => {
                if let Some(reject_reason) = conn.check_replan(&reason) {
                    return FrameRouteResult::Rejected(reject_reason);
                }
                FrameRouteResult::ReplanRequest { reason, severity }
            }

            // Messages that agents should NOT send (orchestrator -> agent only)
            IpcMessageType::AgentRequest { .. }
            | IpcMessageType::InferResponse { .. }
            | IpcMessageType::ToolResult { .. }
            | IpcMessageType::BashResult { .. }
            | IpcMessageType::GroundingResult { .. }
            | IpcMessageType::AgentKill { .. } => {
                FrameRouteResult::Rejected(format!(
                    "agent sent orchestrator-only message type"
                ))
            }
        }
    }

    /// Read a single IPC frame from a stream. Validates version and length before allocation.
    pub fn read_frame(stream: &mut dyn Read) -> Result<IpcFrame, IpcGatewayError> {
        let mut header = [0u8; FRAME_HEADER_SIZE];
        stream.read_exact(&mut header).map_err(IpcGatewayError::Io)?;

        let version = header[0];
        if version != IPC_PROTOCOL_VERSION {
            return Err(IpcGatewayError::Frame(IpcFrameError::VersionMismatch {
                expected: IPC_PROTOCOL_VERSION,
                actual: version,
            }));
        }

        let len = u32::from_le_bytes([header[1], header[2], header[3], header[4]]) as usize;
        if len > MAX_FRAME_SIZE {
            return Err(IpcGatewayError::Frame(IpcFrameError::PayloadTooLarge {
                size: len,
                limit: MAX_FRAME_SIZE,
            }));
        }

        let mut payload = vec![0u8; len];
        if len > 0 {
            stream.read_exact(&mut payload).map_err(IpcGatewayError::Io)?;
        }

        Ok(IpcFrame {
            version,
            payload,
        })
    }

    /// Write a single IPC frame to a stream.
    pub fn write_frame(stream: &mut dyn Write, frame: &IpcFrame) -> Result<(), IpcGatewayError> {
        let wire = frame.to_wire_bytes().map_err(IpcGatewayError::Frame)?;
        stream.write_all(&wire).map_err(IpcGatewayError::Io)?;
        stream.flush().map_err(IpcGatewayError::Io)?;
        Ok(())
    }

    /// Check if a path is locked by self-modification locks.
    pub fn is_self_mod_locked(&self, path: &PathBuf) -> bool {
        self.self_mod_locks.is_locked(path)
    }

    /// Cap InferRequest max_tokens to the gateway limit.
    pub fn cap_infer_tokens(requested: u32) -> u32 {
        requested.min(MAX_INFER_TOKENS_CAP)
    }

    /// Run pre_tool_use hook on a ToolRequest/BashRequest before dispatch.
    ///
    /// Returns Ok(None) if allowed, Ok(Some(error_msg)) if blocked (sanitized),
    /// or Err on hook infrastructure failure.
    ///
    /// Called between `route_frame()` return and executor dispatch.
    /// BLOCK verdicts produce a generic error (no rule_id/pattern/tier detail).
    pub fn run_pre_tool_use_hook(
        hook: &dyn elai_plugins::IpcHookTrait,
        agent_id: &Uuid,
        agent_role: &str,
        session_id: &str,
        fsm_state: &str,
        tool_id: &str,
        arguments: &[u8],
        source_content: Option<&str>,
        file_path: Option<&str>,
    ) -> Result<Option<String>, elai_plugins::IpcHookError> {
        let ctx = elai_plugins::HookContext {
            agent_id: agent_id.to_string(),
            agent_role: agent_role.to_string(),
            session_id: session_id.to_string(),
            fsm_state: fsm_state.to_string(),
            tool_id: Some(tool_id.to_string()),
            tool_arguments: Some(arguments.to_vec()),
            source_content: source_content.map(String::from),
            file_path: file_path.map(String::from),
            active_rules_count: 0,
        };

        let result = hook.pre_tool_use(&ctx)?;

        if result.verdict.is_block() {
            tracing::info!(
                agent_id = %agent_id,
                tool_id = tool_id,
                latency_us = result.latency_us,
                finding_written = result.finding_written,
                "pre_tool_use BLOCK"
            );
            Ok(Some(result.sanitized_agent_message()))
        } else {
            if result.verdict.is_warn() {
                tracing::info!(
                    agent_id = %agent_id,
                    tool_id = tool_id,
                    latency_us = result.latency_us,
                    "pre_tool_use WARN"
                );
            }
            Ok(None)
        }
    }

    /// Run post_tool_use hook after tool execution completes.
    ///
    /// Returns the hook result for logging/metrics. Does not block the response
    /// from being returned to the agent.
    pub fn run_post_tool_use_hook(
        hook: &dyn elai_plugins::IpcHookTrait,
        agent_id: &Uuid,
        agent_role: &str,
        session_id: &str,
        fsm_state: &str,
        tool_id: &str,
        tool_output: &str,
        file_path: Option<&str>,
    ) -> Result<elai_plugins::HookResult, elai_plugins::IpcHookError> {
        let ctx = elai_plugins::HookContext {
            agent_id: agent_id.to_string(),
            agent_role: agent_role.to_string(),
            session_id: session_id.to_string(),
            fsm_state: fsm_state.to_string(),
            tool_id: Some(tool_id.to_string()),
            tool_arguments: None,
            source_content: None,
            file_path: file_path.map(String::from),
            active_rules_count: 0,
        };

        hook.post_tool_use(&ctx, tool_output)
    }

    /// Normalize a finding to a blake3 pattern_hash.
    /// Called by the orchestrator when writing findings from hook verdicts.
    pub fn normalize_pattern_hash(rule_id: &str, evidence: &str) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(rule_id.as_bytes());
        hasher.update(b":");
        hasher.update(evidence.as_bytes());
        hasher.finalize().to_hex().to_string()
    }
}

// ---------------------------------------------------------------------------
// SO_PEERCRED verification (Linux only)
// ---------------------------------------------------------------------------

/// Verify the peer credentials of a Unix domain socket connection.
/// Returns the peer PID, or an error if verification fails.
#[cfg(unix)]
pub fn verify_peercred(
    stream: &std::os::unix::net::UnixStream,
    expected_pid: u32,
) -> Result<u32, IpcGatewayError> {
    use std::os::unix::io::AsRawFd;

    let fd = stream.as_raw_fd();
    let mut cred: libc::ucred = unsafe { std::mem::zeroed() };
    let mut len: libc::socklen_t =
        std::mem::size_of::<libc::ucred>().try_into().map_err(|_| {
            IpcGatewayError::PeercredFailed("socklen_t overflow".to_string())
        })?;

    let ret = unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            &mut cred as *mut _ as *mut libc::c_void,
            &mut len,
        )
    };

    if ret != 0 {
        return Err(IpcGatewayError::PeercredFailed(format!(
            "getsockopt(SO_PEERCRED) failed: {}",
            io::Error::last_os_error()
        )));
    }

    let peer_pid: u32 = cred.pid.try_into().map_err(|_| {
        IpcGatewayError::PeercredFailed("negative PID from SO_PEERCRED".to_string())
    })?;

    if peer_pid != expected_pid {
        return Err(IpcGatewayError::PeercredMismatch {
            expected: expected_pid,
            actual: peer_pid,
        });
    }

    Ok(peer_pid)
}

// ---------------------------------------------------------------------------
// Create session socket directory
// ---------------------------------------------------------------------------

/// Create the session socket directory with 0700 permissions.
#[cfg(unix)]
pub fn create_session_socket_dir(path: &Path) -> Result<(), IpcGatewayError> {
    use std::os::unix::fs::DirBuilderExt;

    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
        .map_err(IpcGatewayError::Io)
}

/// Create the session socket directory (Windows — no Unix permissions).
#[cfg(windows)]
pub fn create_session_socket_dir(path: &Path) -> Result<(), IpcGatewayError> {
    std::fs::create_dir_all(path).map_err(IpcGatewayError::Io)
}

// ---------------------------------------------------------------------------
// IpcGatewayError
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum IpcGatewayError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("frame error: {0}")]
    Frame(#[from] IpcFrameError),

    #[error("SO_PEERCRED verification failed: {0}")]
    PeercredFailed(String),

    #[error("SO_PEERCRED PID mismatch: expected {expected}, got {actual}")]
    PeercredMismatch { expected: u32, actual: u32 },

    #[error("agent not found: {0}")]
    AgentNotFound(Uuid),

    #[error("connection rejected: {0}")]
    ConnectionRejected(String),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_ipc_types::ToolId;

    fn test_gateway() -> IpcGateway {
        IpcGateway::new(
            PathBuf::from("/tmp/elai_cc_test"),
            SelfModificationLocks::new(),
        )
    }

    fn test_connection(role: AgentRole) -> AgentConnection {
        AgentConnection::new(
            Uuid::nil(),
            role,
            1234,
            PathBuf::from("/tmp/test.sock"),
            5, // grc max calls
        )
    }

    #[test]
    fn grc_budget_tracks_per_topic() {
        let mut conn = test_connection(AgentRole::Executor);
        for _ in 0..5 {
            assert!(!conn.record_think_step("topic_a"));
        }
        // 6th call exceeds budget
        assert!(conn.record_think_step("topic_a"));
        // Different topic starts fresh
        assert!(!conn.record_think_step("topic_b"));
    }

    #[test]
    fn grc_converge_removes_active_topic() {
        let mut conn = test_connection(AgentRole::Executor);
        conn.record_think_step("analysis");
        assert!(conn.has_active_topic("analysis"));
        conn.record_converge("analysis");
        assert!(!conn.has_active_topic("analysis"));
    }

    #[test]
    fn replan_budget_enforced() {
        let mut conn = test_connection(AgentRole::Executor);
        assert!(conn.check_replan("reason1").is_none());
        assert!(conn.check_replan("reason2").is_none());
        assert!(conn.check_replan("reason3").is_none());
        // 4th blocked
        assert!(conn.check_replan("reason4").is_some());
    }

    #[test]
    fn replan_dedup_same_reason() {
        let mut conn = test_connection(AgentRole::Executor);
        assert!(conn.check_replan("same_reason").is_none());
        assert!(conn.check_replan("same_reason").is_some());
    }

    #[test]
    fn grounding_orphaned_topic_rejected() {
        let mut conn = test_connection(AgentRole::Executor);
        // No active topic -> rejected
        let result = conn.check_grounding("no_topic", GroundingMethod::FileCheck);
        assert!(result.is_some());
        assert!(result.as_ref().map_or(false, |r| r.contains("orphaned")));
    }

    #[test]
    fn grounding_role_method_matrix() {
        // Executor: FILE_CHECK only
        let methods = allowed_grounding_methods(&AgentRole::Executor);
        assert!(methods.contains(&GroundingMethod::FileCheck));
        assert!(!methods.contains(&GroundingMethod::WebCheck));

        // Researcher: FILE_CHECK + WEB_CHECK
        let methods = allowed_grounding_methods(&AgentRole::Researcher);
        assert!(methods.contains(&GroundingMethod::FileCheck));
        assert!(methods.contains(&GroundingMethod::WebCheck));
        assert!(!methods.contains(&GroundingMethod::SourceCheck));

        // Auditor: all
        let methods = allowed_grounding_methods(&AgentRole::Auditor);
        assert!(methods.contains(&GroundingMethod::FileCheck));
        assert!(methods.contains(&GroundingMethod::WebCheck));
        assert!(methods.contains(&GroundingMethod::SourceCheck));
    }

    #[test]
    fn grounding_wrong_method_rejected() {
        let mut conn = test_connection(AgentRole::Executor);
        conn.record_think_step("topic");
        // Executor can't use WebCheck
        let result = conn.check_grounding("topic", GroundingMethod::WebCheck);
        assert!(result.is_some());
        assert!(result.as_ref().map_or(false, |r| r.contains("not allowed")));
    }

    #[test]
    fn route_heartbeat() {
        let mut gw = test_gateway();
        let agent_id = Uuid::nil();
        gw.register_agent(test_connection(AgentRole::Executor));

        let result = gw.route_frame(
            &agent_id,
            IpcMessageType::AgentHeartbeat { agent_id, uptime_secs: 10 },
        );
        assert!(matches!(result, FrameRouteResult::Heartbeat));
    }

    #[test]
    fn route_infer_request_capped() {
        let mut gw = test_gateway();
        let agent_id = Uuid::nil();
        gw.register_agent(test_connection(AgentRole::Executor));

        let result = gw.route_frame(
            &agent_id,
            IpcMessageType::InferRequest { prompt: "test".into(), max_tokens: 100_000 },
        );
        match result {
            FrameRouteResult::InferRequest { max_tokens, .. } => {
                assert_eq!(max_tokens, MAX_INFER_TOKENS_CAP);
            }
            other => panic!("expected InferRequest, got {other:?}"),
        }
    }

    #[test]
    fn route_orchestrator_only_message_rejected() {
        let mut gw = test_gateway();
        let agent_id = Uuid::nil();
        gw.register_agent(test_connection(AgentRole::Executor));

        let result = gw.route_frame(
            &agent_id,
            IpcMessageType::AgentKill { reason: "test".into() },
        );
        assert!(matches!(result, FrameRouteResult::Rejected(_)));
    }

    #[test]
    fn route_think_step_budget_advisory() {
        let mut gw = test_gateway();
        let agent_id = Uuid::nil();
        gw.register_agent(test_connection(AgentRole::Executor));

        // First 5 calls within budget
        for _ in 0..5 {
            let result = gw.route_frame(
                &agent_id,
                IpcMessageType::ThinkStep {
                    topic: "t".into(),
                    step_type: "observe".into(),
                    content: "data".into(),
                },
            );
            match result {
                FrameRouteResult::ThinkStep { over_budget, .. } => {
                    assert!(!over_budget);
                }
                other => panic!("expected ThinkStep, got {other:?}"),
            }
        }

        // 6th is over budget but still routed (advisory)
        let result = gw.route_frame(
            &agent_id,
            IpcMessageType::ThinkStep {
                topic: "t".into(),
                step_type: "observe".into(),
                content: "data".into(),
            },
        );
        match result {
            FrameRouteResult::ThinkStep { over_budget, .. } => {
                assert!(over_budget);
            }
            other => panic!("expected ThinkStep, got {other:?}"),
        }
    }

    #[test]
    fn frame_read_write_roundtrip() {
        let msg = IpcMessageType::AgentHeartbeat {
            agent_id: Uuid::nil(),
            uptime_secs: 42,
        };
        let frame = IpcFrame::encode(&msg).unwrap();

        let mut buf = Vec::new();
        IpcGateway::write_frame(&mut buf, &frame).unwrap();

        let mut cursor = std::io::Cursor::new(buf);
        let parsed = IpcGateway::read_frame(&mut cursor).unwrap();
        let decoded = IpcFrame::decode(&parsed.payload).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn heartbeat_timeout_detection() {
        let mut conn = test_connection(AgentRole::Executor);
        // Fresh connection should not be timed out
        assert!(!conn.is_heartbeat_timeout());
        // Manually set old timestamp
        conn.last_heartbeat = std::time::Instant::now()
            - std::time::Duration::from_secs(AGENT_HEARTBEAT_TIMEOUT_SECS + 1);
        assert!(conn.is_heartbeat_timeout());
    }

    #[test]
    fn socket_path_generation() {
        let gw = test_gateway();
        let id = Uuid::nil();
        let path = gw.socket_path_for(&id);
        assert!(path.to_string_lossy().contains(&id.to_string()));
        assert!(path.to_string_lossy().ends_with(".sock"));
    }

    #[test]
    fn unknown_agent_rejected() {
        let mut gw = test_gateway();
        let unknown = Uuid::from_bytes([1; 16]);
        let result = gw.route_frame(
            &unknown,
            IpcMessageType::AgentHeartbeat { agent_id: unknown, uptime_secs: 0 },
        );
        assert!(matches!(result, FrameRouteResult::Rejected(_)));
    }
}
