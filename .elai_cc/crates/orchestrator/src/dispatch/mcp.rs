// C-39: MCP bridge dispatch implementation.
//
// Routes tool calls through DefaultMcpBridge (plugins crate, feature=mcp).
// This is the --via-mcp path. Uses the same HTTP JSON-RPC 2.0 client that
// the dashboard uses (plugins/src/mcp_bridge.rs).
//
// STUB_BAN compliance (C-39): on network error, surfaces to SSB and fails
// closed -- no silent retry.
//
// Distinct-action-type contract: this impl emits SsbPayload::CliMcpDispatch
// rows; the legacy in-process / dashboard path emits McpToolCalled. The
// CI parity checker unions both action types (see scripts/ci_mcp_parity_check.py).

use std::sync::Arc;

use super::{Dispatch, DispatchRequest, DispatchResult};
use crate::ssb::{CliMcpDispatchPayload, Ssb, SsbPayload};

/// MCP bridge dispatch. Routes tool calls via DefaultMcpBridge HTTP client.
/// Emits CliMcpDispatch SSB rows on each call (when an SSB writer is wired).
/// Port is the bridge server port (default 8200 from DEFAULT_MCP_SERVER_URL).
pub struct McpClientDispatch {
    pub port: u16,
    /// Optional SSB writer. When `Some`, every dispatch emits one
    /// CliMcpDispatch row carrying tool_name + port. When `None` (test
    /// constructor), dispatch is observable only via the return value.
    ssb: Option<Arc<Ssb>>,
}

impl McpClientDispatch {
    /// Construct with the given bridge server port and SSB writer for
    /// observable telemetry. This is the production constructor.
    pub fn new(port: u16, ssb: Arc<Ssb>) -> Self {
        Self { port, ssb: Some(ssb) }
    }

    /// Construct with just a port; no SSB emission. Used by tests + the
    /// contract suite where the runtime is too lightweight to host an
    /// Ssb instance.
    pub fn new_noop() -> Self {
        // Default port matches DEFAULT_MCP_SERVER_URL in plugins/src/mcp_bridge.rs.
        Self { port: 8200, ssb: None }
    }

    /// Builder: attach an SSB writer post-construction. Used by the
    /// runtime wiring path when the writer becomes available after the
    /// dispatcher is constructed.
    pub fn with_ssb(mut self, ssb: Arc<Ssb>) -> Self {
        self.ssb = Some(ssb);
        self
    }

    fn emit_ssb(&self, tool_name: &str) {
        let Some(ssb) = self.ssb.as_ref() else { return };
        let payload = SsbPayload::CliMcpDispatch(CliMcpDispatchPayload {
            tool_name: tool_name.to_owned(),
            port: self.port,
        });
        // Fire-and-forget emit on a current-thread runtime. Errors are
        // logged but never propagated to the dispatch caller -- a missing
        // SSB row is a telemetry gap, not a dispatch failure.
        let ssb = ssb.clone();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let rt = match rt {
            Ok(rt) => rt,
            Err(e) => {
                tracing::warn!(error = %e, "cli_mcp_dispatch: ssb runtime build failed; skipping emit");
                return;
            }
        };
        let result = rt.block_on(async move {
            ssb.write_payload("cli-mcp-dispatch", 0, payload, Vec::new()).await
        });
        if let Err(e) = result {
            tracing::warn!(error = %e, "cli_mcp_dispatch: ssb emit failed");
        }
    }
}

impl Dispatch for McpClientDispatch {
    fn dispatch_tool(&self, req: &DispatchRequest) -> DispatchResult {
        // C-39: always emit one CliMcpDispatch row per call, even on
        // network failure -- the row records the ATTEMPT so dashboards
        // and CI parity scripts see the symmetric event stream.
        self.emit_ssb(&req.tool_name);

        // Build a current-thread runtime and round-trip the call through
        // DefaultMcpBridge::http_call_tool. The bridge handles JSON-RPC 2.0
        // framing + result sanitization. stub_ban: no silent retry; the
        // first error surfaces to the caller.
        #[cfg(feature = "mcp")]
        {
            use elai_plugins::mcp_bridge::DefaultMcpBridge;
            let bridge = DefaultMcpBridge::default();
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("cli_mcp_dispatch: runtime build failed: {}", e))?;
            let url = format!("http://localhost:{}", self.port);
            let tool_name = req.tool_name.clone();
            let args = req.args.clone();
            let result = rt.block_on(async move {
                bridge.http_call_tool(&url, &tool_name, args).await
            });
            result
                .map(|r| r.content)
                .map_err(|e| format!("cli_mcp_dispatch: bridge call failed: {}", e))
        }
        #[cfg(not(feature = "mcp"))]
        {
            // Without the mcp feature there is no DefaultMcpBridge in the
            // dependency graph. Surface a structured error -- callers
            // should activate the feature before selecting --via-mcp.
            Err(format!(
                "cli_mcp_dispatch: plugins/mcp feature not compiled in (port {})",
                self.port
            ))
        }
    }
}
