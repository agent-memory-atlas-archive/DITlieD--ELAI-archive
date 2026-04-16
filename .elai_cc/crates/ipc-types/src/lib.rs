// Shared IPC wire protocol types for agent <-> orchestrator communication.
//
// This crate exists to break the circular dependency between elai_cc_agent
// (which must NOT depend on elai-orchestrator) and elai-orchestrator (which
// defines the message types). Both crates depend on this shared crate.
//
// Wire format: [1-byte version][4-byte LE length][postcard payload]

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// AgentRole — the 8 agent roles
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
// Constants
// ---------------------------------------------------------------------------

/// Wire protocol version. Mismatch between orchestrator and agent -> kill.
pub const IPC_PROTOCOL_VERSION: u8 = 1;

/// Hard cap on any single IPC frame payload (10 MiB).
pub const MAX_FRAME_SIZE: usize = 10 * 1024 * 1024;

/// Per-type size limits (enforced at frame read time).
pub const MAX_AGENT_HEARTBEAT_SIZE: usize = 1024;
pub const MAX_THINK_STEP_SIZE: usize = 64 * 1024;
pub const MAX_TOOL_RESULT_SIZE: usize = MAX_FRAME_SIZE;

/// Agent heartbeat interval in seconds.
pub const AGENT_HEARTBEAT_INTERVAL_SECS: u64 = 5;

/// Timeout after which a missing heartbeat is treated as agent death.
pub const AGENT_HEARTBEAT_TIMEOUT_SECS: u64 = 15;

/// Length-prefix field size (4 bytes, little-endian u32).
pub const LENGTH_PREFIX_SIZE: usize = 4;

/// Total frame header size: 1 (version) + 4 (length).
pub const FRAME_HEADER_SIZE: usize = 1 + LENGTH_PREFIX_SIZE;

// ---------------------------------------------------------------------------
// ToolId — opaque tool identifier
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolId(pub String);

impl ToolId {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// SeccompProfile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeccompProfile {
    IpcBaseline,
    NetworkExtra,
}

// ---------------------------------------------------------------------------
// ResourceLimits
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_bytes: u64,
    pub max_cpu_secs: u64,
    pub max_open_files: u32,
}

// ---------------------------------------------------------------------------
// ProcessConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub allowed_paths: Vec<PathBuf>,
    pub seccomp_profile: SeccompProfile,
    pub resource_limits: ResourceLimits,
}

// ---------------------------------------------------------------------------
// ToolFilter
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolFilter {
    pub allowed_tools: Vec<ToolId>,
}

impl ToolFilter {
    pub fn allows(&self, tool_id: &ToolId) -> bool {
        self.allowed_tools.contains(tool_id)
    }
}

// ---------------------------------------------------------------------------
// AgentDispatchConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDispatchConfig {
    pub role: AgentRole,
    pub model_preference: String,
    pub tool_filter: ToolFilter,
    pub rule_domains: Vec<String>,
    pub grc_budget_max_calls: u32,
    pub grc_budget_max_chars: u32,
    pub sandbox_config: ProcessConfig,
}

// ---------------------------------------------------------------------------
// IpcMessageType — 15 message variants (LOCKED)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IpcMessageType {
    AgentRequest { task: String, dispatch_config: AgentDispatchConfig },
    AgentResponse { content: String, artifacts: Vec<String> },
    InferRequest { prompt: String, max_tokens: u32 },
    InferResponse { text: String, tokens_used: u32 },
    ToolRequest { tool_id: ToolId, arguments: Vec<u8> },
    ToolResult { tool_id: ToolId, output: Vec<u8>, success: bool },
    BashRequest { command: String, timeout_ms: u64 },
    BashResult { stdout: Vec<u8>, stderr: Vec<u8>, exit_code: i32 },
    GroundingRequest { topic: String, query: String },
    GroundingResult { topic: String, evidence: Vec<String>, confidence: f32 },
    ThinkStep { topic: String, step_type: String, content: String },
    CheckpointSync { agent_id: Uuid, sequence: u64, payload: Vec<u8> },
    AgentHeartbeat { agent_id: Uuid, uptime_secs: u64 },
    AgentKill { reason: String },
    ReplanRequest { reason: String, severity: String },
}

// Compile-time assertion: exactly 15 variants.
#[doc(hidden)]
pub const fn _ipc_variant_tag(msg: &IpcMessageType) -> u8 {
    match msg {
        IpcMessageType::AgentRequest { .. } => 0,
        IpcMessageType::AgentResponse { .. } => 1,
        IpcMessageType::InferRequest { .. } => 2,
        IpcMessageType::InferResponse { .. } => 3,
        IpcMessageType::ToolRequest { .. } => 4,
        IpcMessageType::ToolResult { .. } => 5,
        IpcMessageType::BashRequest { .. } => 6,
        IpcMessageType::BashResult { .. } => 7,
        IpcMessageType::GroundingRequest { .. } => 8,
        IpcMessageType::GroundingResult { .. } => 9,
        IpcMessageType::ThinkStep { .. } => 10,
        IpcMessageType::CheckpointSync { .. } => 11,
        IpcMessageType::AgentHeartbeat { .. } => 12,
        IpcMessageType::AgentKill { .. } => 13,
        IpcMessageType::ReplanRequest { .. } => 14,
    }
}

/// Number of IpcMessageType variants.
pub const IPC_MESSAGE_TYPE_VARIANT_COUNT: usize = 15;

// ---------------------------------------------------------------------------
// IpcFrame — wire-level frame
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct IpcFrame {
    pub version: u8,
    pub payload: Vec<u8>,
}

impl IpcFrame {
    pub fn encode(msg: &IpcMessageType) -> Result<Self, IpcFrameError> {
        let payload = postcard::to_allocvec(msg)
            .map_err(|e| IpcFrameError::SerializationFailed(e.to_string()))?;
        if payload.len() > MAX_FRAME_SIZE {
            return Err(IpcFrameError::PayloadTooLarge {
                size: payload.len(),
                limit: MAX_FRAME_SIZE,
            });
        }
        Ok(Self {
            version: IPC_PROTOCOL_VERSION,
            payload,
        })
    }

    pub fn decode(payload: &[u8]) -> Result<IpcMessageType, IpcFrameError> {
        postcard::from_bytes(payload)
            .map_err(|e| IpcFrameError::DeserializationFailed(e.to_string()))
    }

    pub fn from_wire_bytes(data: &[u8]) -> Result<Self, IpcFrameError> {
        if data.len() < FRAME_HEADER_SIZE {
            return Err(IpcFrameError::FrameTooShort);
        }
        let version = data[0];
        if version != IPC_PROTOCOL_VERSION {
            return Err(IpcFrameError::VersionMismatch {
                expected: IPC_PROTOCOL_VERSION,
                actual: version,
            });
        }
        let len = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
        if len > MAX_FRAME_SIZE {
            return Err(IpcFrameError::PayloadTooLarge {
                size: len,
                limit: MAX_FRAME_SIZE,
            });
        }
        let total = FRAME_HEADER_SIZE + len;
        if data.len() < total {
            return Err(IpcFrameError::FrameTooShort);
        }
        Ok(Self {
            version,
            payload: data[FRAME_HEADER_SIZE..total].to_vec(),
        })
    }

    pub fn to_wire_bytes(&self) -> Result<Vec<u8>, IpcFrameError> {
        let len: u32 = self.payload.len().try_into().map_err(|_| {
            IpcFrameError::PayloadTooLarge {
                size: self.payload.len(),
                limit: u32::MAX as usize,
            }
        })?;
        let mut buf = Vec::with_capacity(FRAME_HEADER_SIZE + self.payload.len());
        buf.push(self.version);
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(&self.payload);
        Ok(buf)
    }
}

// ---------------------------------------------------------------------------
// IpcFrameError
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IpcFrameError {
    #[error("serialization failed: {0}")]
    SerializationFailed(String),
    #[error("deserialization failed: {0}")]
    DeserializationFailed(String),
    #[error("payload too large: {size} bytes exceeds limit of {limit} bytes")]
    PayloadTooLarge { size: usize, limit: usize },
    #[error("protocol version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u8, actual: u8 },
    #[error("frame too short: need at least {FRAME_HEADER_SIZE} bytes")]
    FrameTooShort,
}

// ---------------------------------------------------------------------------
// FsmState — mirrors orchestrator FSM states for tool filtering
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FsmState {
    Idle,
    Planning,
    TestingFirst,
    Executing,
    Auditing,
    Memorizing,
    Complete,
    Configuring,
}

// ---------------------------------------------------------------------------
// Typed tool argument structs (postcard-serialized in ToolRequest.arguments)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitCommitRequest {
    pub message: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitUndoRequest {
    pub commit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitBranchRequest {
    pub branch_name: String,
    pub from_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub file_path: String,
    pub language: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceRequest {
    pub target_function: String,
    pub file_path: String,
    pub depth: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditRequest {
    pub file_path: String,
    pub old_text: String,
    pub new_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimplifyRequest {
    pub file_path: String,
    pub target_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassifyRequest {
    pub error_output: String,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileReadRequest {
    pub file_path: String,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompileCheckRequest {
    pub file_path: String,
    pub language: String,
    pub retry_round: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DkgCallersRequest {
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DkgDependentsRequest {
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DkgDataFlowRequest {
    pub variable: String,
    pub direction: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DkgSubgraphRequest {
    pub entry_point: String,
    pub depth: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrationCheckRequest {
    pub target_symbol: String,
    pub project_root: String,
}

// ---------------------------------------------------------------------------
// Tool result payload (generic success/error envelope)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResultPayload {
    pub output: String,
    pub diagnostics: Vec<String>,
}

// ---------------------------------------------------------------------------
// Well-known tool IDs (constants to avoid string typos)
// ---------------------------------------------------------------------------

pub const TOOL_GIT_COMMIT: &str = "git_commit";
pub const TOOL_GIT_UNDO: &str = "git_undo";
pub const TOOL_GIT_BRANCH: &str = "git_branch";
pub const TOOL_VALIDATE: &str = "validate";
pub const TOOL_TRACE: &str = "trace";
pub const TOOL_EDIT: &str = "edit";
pub const TOOL_SIMPLIFY: &str = "simplify";
pub const TOOL_CLASSIFY: &str = "classify";
pub const TOOL_FILE_READ: &str = "file_read";
pub const TOOL_COMPILE_CHECK: &str = "compile_check";
pub const TOOL_DKG_CALLERS: &str = "dkg_callers";
pub const TOOL_DKG_DEPENDENTS: &str = "dkg_dependents";
pub const TOOL_DKG_DATA_FLOW: &str = "dkg_data_flow";
pub const TOOL_DKG_SUBGRAPH: &str = "dkg_subgraph";
pub const TOOL_INTEGRATION_CHECK: &str = "integration_check";

// Web research tool IDs (4D.17 — for future Tier C agent IPC proxy).
pub const TOOL_SEARCH_REQUEST: &str = "search_request";
pub const TOOL_FETCH_REQUEST: &str = "fetch_request";
pub const TOOL_EMBED_REQUEST: &str = "embed_request";

// ---------------------------------------------------------------------------
// Web research typed request structs (postcard-serialized in ToolRequest.arguments)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebSearchRequest {
    pub query: String,
    pub max_results: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebFetchRequest {
    pub url: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbedRequest {
    pub texts: Vec<String>,
    pub model: String,
}
