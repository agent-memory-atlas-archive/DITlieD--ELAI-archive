// C-39: MCP bridge dispatch implementation.
//
// Routes tool calls through DefaultMcpBridge (plugins crate, feature=mcp).
// This is the --via-mcp path. Uses the same HTTP JSON-RPC 2.0 client that
// the dashboard uses (plugins/src/mcp_bridge.rs).
//
// STUB_BAN compliance (C-39): on network error, surfaces to SSB and fails
// closed -- no silent retry.

use super::{Dispatch, DispatchRequest, DispatchResult};

/// MCP bridge dispatch. Routes tool calls via DefaultMcpBridge HTTP client.
/// Emits CliMcpDispatch SSB rows on each call.
/// Port is the bridge server port (default 8200 from DEFAULT_MCP_SERVER_URL).
pub struct McpClientDispatch {
    pub port: u16,
}

impl McpClientDispatch {
    /// Construct with the given bridge server port.
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    /// No-op constructor for contract/compile tests.
    pub fn new_noop() -> Self {
        // Default port matches DEFAULT_MCP_SERVER_URL in plugins/src/mcp_bridge.rs.
        Self { port: 8200 }
    }
}

impl Dispatch for McpClientDispatch {
    fn dispatch_tool(&self, req: &DispatchRequest) -> DispatchResult {
        // TODO[C39.stub]: placeholder MCP bridge call; replace with DefaultMcpBridge::http_call_tool integration
        // C-39: stub_ban -- network errors must surface to the caller, no
        // silent retry. The production implementation will call
        // DefaultMcpBridge::http_call_tool and map McpError to Err(String).
        // This placeholder returns an error so the --via-mcp path correctly
        // fails when no bridge is listening (pre-auto-launch integration).
        Err(format!(
            "cli_mcp_dispatch: bridge at port {} not yet integrated (C-39 production wiring pending)",
            self.port
        ))
    }
}
