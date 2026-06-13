#![deny(dead_code, unused_imports)]

use rustc_hash::FxHashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use elai_ipc_types::{FsmState, ToolId, ToolResultPayload};
use elai_orchestrator::Ssb;
use elai_orchestrator::ssb::{MemoryGuardEventKind, MemoryGuardEventPayload, SsbPayload};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Target IPC dispatch overhead (p95) in milliseconds.
pub const IPC_OVERHEAD_TARGET_MS: u64 = 2;

// ---------------------------------------------------------------------------
// ToolError — typed errors for all tool operations
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("tool not found: {tool_id}")]
    NotFound { tool_id: String },

    #[error("tool not allowed in FSM state {state:?}: {tool_id}")]
    NotAllowedInState { tool_id: String, state: FsmState },

    #[error("deserialization failed for {tool_id}: {reason}")]
    DeserializationFailed { tool_id: String, reason: String },

    #[error("execution failed for {tool_id}: {reason}")]
    ExecutionFailed { tool_id: String, reason: String },

    #[error("SSB write lock denied for {tool_id}: {path}")]
    SsbWriteLockDenied { tool_id: String, path: String },

    #[error("subprocess spawn failed for {tool_id}: {reason}")]
    SubprocessFailed { tool_id: String, reason: String },

    #[error("io error in {tool_id}: {source}")]
    Io {
        tool_id: String,
        #[source]
        source: std::io::Error,
    },

    #[error("DKG query failed for {tool_id}: {reason}")]
    DkgQueryFailed { tool_id: String, reason: String },
}

// ---------------------------------------------------------------------------
// ToolHandler — individual tool execution
// ---------------------------------------------------------------------------

/// Async handler for a single tool. Implementations live in tool module files.
/// Uses boxed future return for dyn-compatibility (native async fn in traits
/// is not object-safe; this avoids the async-trait crate per project rules).
pub trait ToolHandler: Send + Sync {
    /// Execute the tool with postcard-deserialized arguments.
    fn execute<'a>(
        &'a self,
        arguments: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<ToolResultPayload, ToolError>> + Send + 'a>>;

    /// Tool ID string (must match TOOL_* constants in ipc-types).
    fn tool_id(&self) -> &str;

    /// Whether this tool performs write operations (used for FSM filtering).
    fn is_write_tool(&self) -> bool;
}

// ---------------------------------------------------------------------------
// IpcToolRegistry — dispatch hub for all agent-accessible tools
// ---------------------------------------------------------------------------

pub struct IpcToolRegistry {
    handlers: FxHashMap<String, Arc<dyn ToolHandler>>,
    /// I-13: optional SSB writer for `DualSerialized` event emission.
    /// When `Some`, every `dispatch()` call that returns a condensable payload
    /// emits a `MemoryGuardEvent(DualSerialized { token_delta })` row.
    ssb: Option<Arc<Ssb>>,
}

impl IpcToolRegistry {
    pub fn new() -> Self {
        Self {
            handlers: FxHashMap::default(),
            ssb: None,
        }
    }

    /// Attach an SSB writer so `dispatch()` can emit `DualSerialized` rows.
    ///
    /// I-13: production callers that hold an `Arc<Ssb>` pass it here so the
    /// dual-serialization token delta is recorded in the SSB stream.
    #[must_use]
    pub fn with_ssb(mut self, ssb: Arc<Ssb>) -> Self {
        self.ssb = Some(ssb);
        self
    }

    /// Register a tool handler.
    pub fn register(&mut self, handler: Arc<dyn ToolHandler>) {
        self.handlers.insert(handler.tool_id().to_string(), handler);
    }

    /// Dispatch a tool request. Checks FSM state filtering before execution.
    ///
    /// I-13: when an SSB writer is wired (see [`Self::with_ssb`]), emits a
    /// `MemoryGuardEvent(DualSerialized { token_delta })` row whenever the
    /// payload output exceeds [`crate::tool_result::LLM_OUTPUT_CHAR_LIMIT`].
    /// The `to_llm_content()` condensed form is NOT returned to callers here
    /// (that is the responsibility of the context-assembly path); the emit
    /// records the delta for the dashboard DualSerialized badge.
    pub async fn dispatch(
        &self,
        tool_id: &str,
        arguments: &[u8],
        fsm_state: FsmState,
    ) -> Result<ToolResultPayload, ToolError> {
        let handler = self.handlers.get(tool_id).ok_or_else(|| ToolError::NotFound {
            tool_id: tool_id.to_string(),
        })?;

        // Phase-aware filtering: block write tools in read-only states
        if handler.is_write_tool() && !Self::write_tools_allowed(fsm_state) {
            return Err(ToolError::NotAllowedInState {
                tool_id: tool_id.to_string(),
                state: fsm_state,
            });
        }

        // Block compile_check in TestingFirst
        if tool_id == elai_ipc_types::TOOL_COMPILE_CHECK && fsm_state == FsmState::TestingFirst {
            return Err(ToolError::NotAllowedInState {
                tool_id: tool_id.to_string(),
                state: fsm_state,
            });
        }

        let payload = handler.execute(arguments).await?;

        // I-13: emit DualSerialized SSB row when output exceeds LLM_OUTPUT_CHAR_LIMIT.
        // The token_delta is the character-count difference between the full output
        // (to_ui_dict) and the condensed output (to_llm_content). This is the
        // off-agent artifact (R4) proving the dual-serialization gate fired.
        if let Some(ssb) = self.ssb.as_ref() {
            let delta = crate::tool_result::dual_serialize_delta(&payload);
            if delta > 0 {
                let ts_ms = {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
                        .unwrap_or(0)
                };
                let emit_payload = SsbPayload::MemoryGuardEvent(MemoryGuardEventPayload {
                    kind: MemoryGuardEventKind::DualSerialized,
                    pattern_class: None,
                    token_delta: Some(delta),
                    byte_range: None,
                    ts_ms,
                });
                if let Err(e) = ssb
                    .write_payload("ipc_registry", 0, emit_payload, vec![tool_id.to_string()])
                    .await
                {
                    tracing::warn!(
                        error = %e,
                        tool_id = %tool_id,
                        "IpcToolRegistry: DualSerialized SSB write failed (non-fatal)"
                    );
                }
            }
        }

        Ok(payload)
    }

    /// Return tool IDs available in the given FSM state.
    pub fn available_tools(&self, fsm_state: FsmState) -> Vec<ToolId> {
        let write_allowed = Self::write_tools_allowed(fsm_state);
        self.handlers
            .values()
            .filter(|h| {
                if h.is_write_tool() && !write_allowed {
                    return false;
                }
                // compile_check blocked in TestingFirst
                if h.tool_id() == elai_ipc_types::TOOL_COMPILE_CHECK
                    && fsm_state == FsmState::TestingFirst
                {
                    return false;
                }
                true
            })
            .map(|h| ToolId::new(h.tool_id()))
            .collect()
    }

    /// Whether write tools are allowed in this FSM state.
    ///
    /// `pub` since C-13 so the [`crate::adaptive_tool`] adapter shim
    /// (`IpcToolDispatcherAdapter`) can replicate the registry's
    /// write-side FSM gate without going through the async `dispatch`
    /// path. The function is intentionally a pure check on `FsmState`;
    /// no per-instance state is consulted.
    pub fn write_tools_allowed(state: FsmState) -> bool {
        matches!(state, FsmState::Executing | FsmState::Auditing)
    }

    /// Number of registered handlers.
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }
}

impl Default for IpcToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
