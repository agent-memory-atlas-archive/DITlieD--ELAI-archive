// IPC wire protocol types for agent process isolation (Phase 4A).
//
// Wire format: [1-byte version][4-byte LE length][postcard payload]
// All messages are serialized with postcard. The orchestrator and agent binary
// both depend on this module for type definitions.
//
// Source: phase-4A.txt L49-79, orchestrator.txt S12.

use crate::agent_pool::AgentRole;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

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

/// Opaque tool identifier used in ToolFilter and ToolRequest messages.
/// Wraps a String to maintain type safety at the IPC boundary.
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
// IpcMessageType — 15 message variants (must match orchestrator.txt S12)
// ---------------------------------------------------------------------------

/// All IPC message types exchanged between orchestrator and agent processes.
///
/// Exact variant list from phase-4A.txt L56-59:
///   AgentRequest, AgentResponse, InferRequest, InferResponse,
///   ToolRequest, ToolResult, BashRequest, BashResult,
///   GroundingRequest, GroundingResult, ThinkStep, CheckpointSync,
///   AgentHeartbeat, AgentKill, ReplanRequest
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
// Uses a discriminant-to-unit match. Adding or removing a variant causes a
// compile error (non-exhaustive match or unreachable arm).
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

/// Number of IpcMessageType variants. Used in round-trip tests.
pub const IPC_MESSAGE_TYPE_VARIANT_COUNT: usize = 15;

// ---------------------------------------------------------------------------
// IpcFrame — wire-level frame wrapping a serialized message
// ---------------------------------------------------------------------------

/// A single IPC frame ready for transmission.
/// Created per message send, consumed (moved) on write.
#[derive(Debug, Clone)]
pub struct IpcFrame {
    pub version: u8,
    pub payload: Vec<u8>,
}

impl IpcFrame {
    /// Serialize an IpcMessageType into a frame.
    /// Returns Err if serialization fails or payload exceeds MAX_FRAME_SIZE.
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

    /// Deserialize a frame payload back into an IpcMessageType.
    pub fn decode(payload: &[u8]) -> Result<IpcMessageType, IpcFrameError> {
        postcard::from_bytes(payload)
            .map_err(|e| IpcFrameError::DeserializationFailed(e.to_string()))
    }

    /// Parse wire bytes back into an IpcFrame.
    /// Validates version byte and length prefix before allocating payload.
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

    /// Produce the wire bytes: [version][4-byte LE length][payload].
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
// ToolFilter — whitelist of allowed tools per agent dispatch
// ---------------------------------------------------------------------------

/// Tool access whitelist enforced at the IPC boundary.
/// When an agent sends a ToolRequest, the orchestrator checks tool_id
/// against this filter. Not in the list -> rejected with generic error.
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
// SeccompProfile — seccomp filter profile identifier
// ---------------------------------------------------------------------------

/// Seccomp profile for agent sandboxing (Linux only).
/// IPC_BASELINE: all roles. NETWORK_EXTRA: researcher only (adds AF_INET).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeccompProfile {
    IpcBaseline,
    NetworkExtra,
}

// ---------------------------------------------------------------------------
// ResourceLimits — per-agent resource constraints
// ---------------------------------------------------------------------------

/// Resource limits applied to agent processes via OS mechanisms
/// (cgroups on Linux, Job Objects on Windows).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum resident set size in bytes.
    pub max_memory_bytes: u64,
    /// Maximum CPU time in seconds (wall clock).
    pub max_cpu_secs: u64,
    /// Maximum number of open file descriptors.
    pub max_open_files: u32,
}

// ---------------------------------------------------------------------------
// ProcessConfig — per-role sandbox configuration
// ---------------------------------------------------------------------------

/// Configuration for spawning an agent process in a sandbox.
/// Each role gets a distinct ProcessConfig with allowed paths and
/// security profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub allowed_paths: Vec<PathBuf>,
    pub seccomp_profile: SeccompProfile,
    pub resource_limits: ResourceLimits,
}

// ---------------------------------------------------------------------------
// AgentDispatchConfig — authoritative spawn configuration
// ---------------------------------------------------------------------------

/// The AUTHORITATIVE configuration source for agent process spawning.
/// Contains everything the orchestrator needs to set up and communicate
/// with an agent process.
///
/// Code that checks role == Executor must also check tool_filter and
/// grc_config overrides in this config.
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
// Tests — postcard round-trip for all 15 variants
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn all_15_variants() -> Vec<IpcMessageType> {
        let tool_id = ToolId::new("test_tool");
        let agent_id = Uuid::nil();
        vec![
            IpcMessageType::AgentRequest {
                task: "test".into(),
                dispatch_config: AgentDispatchConfig {
                    role: AgentRole::Executor,
                    model_preference: "local".into(),
                    tool_filter: ToolFilter { allowed_tools: vec![tool_id.clone()] },
                    rule_domains: vec!["default".into()],
                    grc_budget_max_calls: 5,
                    grc_budget_max_chars: 1000,
                    sandbox_config: ProcessConfig {
                        allowed_paths: vec![],
                        seccomp_profile: SeccompProfile::IpcBaseline,
                        resource_limits: ResourceLimits {
                            max_memory_bytes: 1_073_741_824,
                            max_cpu_secs: 300,
                            max_open_files: 256,
                        },
                    },
                },
            },
            IpcMessageType::AgentResponse {
                content: "done".into(),
                artifacts: vec!["a.rs".into()],
            },
            IpcMessageType::InferRequest {
                prompt: "hello".into(),
                max_tokens: 100,
            },
            IpcMessageType::InferResponse {
                text: "world".into(),
                tokens_used: 5,
            },
            IpcMessageType::ToolRequest {
                tool_id: tool_id.clone(),
                arguments: vec![1, 2, 3],
            },
            IpcMessageType::ToolResult {
                tool_id: tool_id.clone(),
                output: vec![4, 5, 6],
                success: true,
            },
            IpcMessageType::BashRequest {
                command: "echo hi".into(),
                timeout_ms: 5000,
            },
            IpcMessageType::BashResult {
                stdout: b"hi\n".to_vec(),
                stderr: vec![],
                exit_code: 0,
            },
            IpcMessageType::GroundingRequest {
                topic: "test".into(),
                query: "is X true?".into(),
            },
            IpcMessageType::GroundingResult {
                topic: "test".into(),
                evidence: vec!["yes".into()],
                confidence: 0.95,
            },
            IpcMessageType::ThinkStep {
                topic: "analysis".into(),
                step_type: "observe".into(),
                content: "data observed".into(),
            },
            IpcMessageType::CheckpointSync {
                agent_id,
                sequence: 42,
                payload: vec![7, 8, 9],
            },
            IpcMessageType::AgentHeartbeat {
                agent_id,
                uptime_secs: 120,
            },
            IpcMessageType::AgentKill {
                reason: "timeout".into(),
            },
            IpcMessageType::ReplanRequest {
                reason: "test failure".into(),
                severity: "warn".into(),
            },
        ]
    }

    #[test]
    fn variant_count_is_15() {
        assert_eq!(all_15_variants().len(), IPC_MESSAGE_TYPE_VARIANT_COUNT);
    }

    #[test]
    fn postcard_round_trip_all_variants() {
        for (i, msg) in all_15_variants().into_iter().enumerate() {
            let frame = IpcFrame::encode(&msg)
                .unwrap_or_else(|e| panic!("encode failed for variant {i}: {e}"));
            let decoded = IpcFrame::decode(&frame.payload)
                .unwrap_or_else(|e| panic!("decode failed for variant {i}: {e}"));
            assert_eq!(msg, decoded, "round-trip mismatch for variant {i}");
        }
    }

    #[test]
    fn wire_bytes_format() {
        let msg = IpcMessageType::AgentHeartbeat {
            agent_id: Uuid::nil(),
            uptime_secs: 1,
        };
        let frame = IpcFrame::encode(&msg).unwrap_or_else(|e| panic!("encode failed: {e}"));
        let wire = frame.to_wire_bytes().unwrap_or_else(|e| panic!("to_wire failed: {e}"));

        // First byte is version
        assert_eq!(wire[0], IPC_PROTOCOL_VERSION);

        // Next 4 bytes are LE length of payload
        let len = u32::from_le_bytes([wire[1], wire[2], wire[3], wire[4]]);
        assert_eq!(len as usize, frame.payload.len());

        // Remaining bytes are payload
        assert_eq!(&wire[FRAME_HEADER_SIZE..], &frame.payload[..]);
    }

    #[test]
    fn payload_too_large_rejected() {
        // Create a message with a payload that would exceed MAX_FRAME_SIZE
        // We test the check in encode by creating a large ToolResult
        let large_data = vec![0u8; MAX_FRAME_SIZE + 1];
        let msg = IpcMessageType::ToolResult {
            tool_id: ToolId::new("big"),
            output: large_data,
            success: true,
        };
        let result = IpcFrame::encode(&msg);
        assert!(
            matches!(result, Err(IpcFrameError::PayloadTooLarge { .. })),
            "expected PayloadTooLarge error"
        );
    }

    #[test]
    fn tool_filter_allows() {
        let filter = ToolFilter {
            allowed_tools: vec![ToolId::new("read_file"), ToolId::new("grep")],
        };
        assert!(filter.allows(&ToolId::new("read_file")));
        assert!(!filter.allows(&ToolId::new("write_file")));
    }

    #[test]
    fn wire_round_trip_via_from_wire_bytes() {
        let msg = IpcMessageType::AgentHeartbeat {
            agent_id: Uuid::nil(),
            uptime_secs: 42,
        };
        let frame = IpcFrame::encode(&msg).unwrap_or_else(|e| panic!("encode: {e}"));
        let wire = frame.to_wire_bytes().unwrap_or_else(|e| panic!("to_wire: {e}"));
        let parsed = IpcFrame::from_wire_bytes(&wire)
            .unwrap_or_else(|e| panic!("from_wire: {e}"));
        let decoded = IpcFrame::decode(&parsed.payload)
            .unwrap_or_else(|e| panic!("decode: {e}"));
        assert_eq!(msg, decoded);
        assert_eq!(parsed.version, IPC_PROTOCOL_VERSION);
    }

    #[test]
    fn version_mismatch_rejected() {
        let msg = IpcMessageType::AgentKill { reason: "test".into() };
        let frame = IpcFrame::encode(&msg).unwrap_or_else(|e| panic!("encode: {e}"));
        let mut wire = frame.to_wire_bytes().unwrap_or_else(|e| panic!("to_wire: {e}"));
        wire[0] = IPC_PROTOCOL_VERSION + 1; // corrupt version
        let result = IpcFrame::from_wire_bytes(&wire);
        assert!(
            matches!(result, Err(IpcFrameError::VersionMismatch { .. })),
            "expected VersionMismatch error"
        );
    }

    #[test]
    fn frame_too_short_rejected() {
        let result = IpcFrame::from_wire_bytes(&[1, 2]);
        assert!(
            matches!(result, Err(IpcFrameError::FrameTooShort)),
            "expected FrameTooShort error"
        );
    }
}
