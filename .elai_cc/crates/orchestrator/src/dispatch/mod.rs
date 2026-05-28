// C-39: Tool dispatch trait and implementations.
//
// Provides a lightweight Dispatch trait that wraps the tool-call layer only
// (not BabyStepExecutor). Two implementations:
//   - InProcessDispatch: routes to the in-process plugin registry (default).
//   - McpClientDispatch: routes via DefaultMcpBridge HTTP JSON-RPC 2.0 client.
//
// The CLI selects between them at startup based on the --via-mcp flag.
// Both emit SSB rows: InProcessDispatch emits existing mcp_tool_called rows;
// McpClientDispatch emits cli_mcp_dispatch rows.

mod in_process;
mod mcp;

pub use in_process::{InProcessDispatch, InProcessToolRunner};
pub use mcp::McpClientDispatch;
// E-38 (SCHEMA_VERSION 175): re-export the MCP transport-version log
// constants so contract tests + future external consumers can verify the
// current vs roadmap-target strings without poking into the private
// dispatch::mcp module.
pub use mcp::{MCP_TRANSPORT_CURRENT_DEFAULT, MCP_TRANSPORT_ROADMAP_TARGET};

// B-15 HashMap ban: prior unused std::collections imports removed.
// (52-line file, no map type appears.) If a future edit adds a real
// map, choose FxHashMap (lookup hot path) or BTreeMap (serialization)
// per B-15-decisions.md per-file rationale.

/// A tool dispatch request carrying the tool name and JSON arguments.
/// Both InProcessDispatch and McpClientDispatch accept this type.
#[derive(Debug, Clone)]
pub struct DispatchRequest {
    pub tool_name: String,
    pub args: serde_json::Value,
}

impl DispatchRequest {
    pub fn new(tool_name: impl Into<String>, args: serde_json::Value) -> Self {
        Self { tool_name: tool_name.into(), args }
    }

    /// No-op constructor for compile-time contract tests.
    pub fn new_noop(tool_name: impl Into<String>) -> Self {
        Self { tool_name: tool_name.into(), args: serde_json::Value::Null }
    }
}

/// Result of a tool dispatch: the JSON output or an error string.
pub type DispatchResult = Result<serde_json::Value, String>;

/// Core dispatch trait. Both in-process and MCP-bridge implementations
/// satisfy this interface. The CLI selects the impl at startup.
///
/// All implementations must:
/// - Never panic; surface errors via DispatchResult::Err.
/// - Never silently retry on network errors (C-39 stub_ban rule).
/// - Emit the appropriate SSB action type on each call.
pub trait Dispatch: Send + Sync {
    fn dispatch_tool(&self, req: &DispatchRequest) -> DispatchResult;
}
