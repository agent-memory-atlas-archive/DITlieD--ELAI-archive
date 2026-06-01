// C-39: In-process dispatch implementation.
//
// Routes tool calls through the in-process plugin / tool registry. This is
// the DEFAULT path (no --via-mcp flag). The actual tool execution function
// lives in the `elai-tools` crate (which depends on `elai-orchestrator`,
// so the orchestrator cannot import it without a cycle); the CLI wiring
// site injects a closure carrying `elai_tools::execute_tool` here. This
// keeps the dispatch crate dependency-clean while still routing every
// invocation through real production code.
//
// Distinct-action-type contract: this impl emits McpToolCalled rows so
// the CI parity script can union in-process telemetry with the via-mcp
// CliMcpDispatch rows. See scripts/ci_mcp_parity_check.py:9.

use std::sync::Arc;

use super::{Dispatch, DispatchRequest, DispatchResult};
use crate::ssb::{McpToolCalledPayload, Ssb, SsbPayload};

/// Closure-based tool runner. The CLI wiring constructs one wrapping
/// `elai_tools::execute_tool` and passes it to `InProcessDispatch::new`.
/// Returns the serialized JSON output on success, or a structured error
/// string on failure. Send + Sync + 'static so the dispatcher is safe to
/// share across the multi-threaded runtime.
pub type InProcessToolRunner =
    Arc<dyn Fn(&str, serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync + 'static>;

/// In-process tool dispatch. Routes calls to a closure-based runner
/// constructed at startup by the CLI / dashboard wiring sites.
pub struct InProcessDispatch {
    /// Optional production tool runner. When `Some`, dispatch_tool
    /// delegates to it. When `None` (test constructor), dispatch returns
    /// a structured "not wired" error so test callers can see the seam
    /// without panicking.
    runner: Option<InProcessToolRunner>,
    /// Optional SSB writer. When `Some`, every dispatch emits one
    /// McpToolCalled row. Telemetry-only -- a missing writer never fails
    /// the dispatch.
    ssb: Option<Arc<Ssb>>,
}

impl InProcessDispatch {
    /// Construct with a production tool runner + SSB writer.
    pub fn new(runner: InProcessToolRunner, ssb: Arc<Ssb>) -> Self {
        Self {
            runner: Some(runner),
            ssb: Some(ssb),
        }
    }

    /// No-op constructor for contract/compile tests. dispatch_tool returns
    /// a structured error indicating the runner seam is not wired.
    pub fn new_noop() -> Self {
        Self {
            runner: None,
            ssb: None,
        }
    }

    /// Builder: attach an SSB writer post-construction (parallel to
    /// McpClientDispatch::with_ssb).
    pub fn with_ssb(mut self, ssb: Arc<Ssb>) -> Self {
        self.ssb = Some(ssb);
        self
    }

    fn emit_ssb(&self, tool_name: &str, outcome: &str, elapsed_ms: u32) {
        let Some(ssb) = self.ssb.as_ref() else { return };
        let payload = SsbPayload::McpToolCalled(McpToolCalledPayload {
            tool_name: tool_name.to_owned(),
            server_url: "in-process".to_string(),
            elapsed_ms,
            outcome: outcome.to_owned(),
        });
        let ssb = ssb.clone();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let rt = match rt {
            Ok(rt) => rt,
            Err(e) => {
                tracing::warn!(error = %e, "in_process_dispatch: ssb runtime build failed; skipping emit");
                return;
            }
        };
        let result = rt.block_on(async move {
            ssb.write_payload("in-process-dispatch", 0, payload, Vec::new()).await
        });
        if let Err(e) = result {
            tracing::warn!(error = %e, "in_process_dispatch: ssb emit failed");
        }
    }
}

impl Default for InProcessDispatch {
    fn default() -> Self {
        Self::new_noop()
    }
}

impl Dispatch for InProcessDispatch {
    fn dispatch_tool(&self, req: &DispatchRequest) -> DispatchResult {
        let Some(runner) = self.runner.as_ref() else {
            // new_noop / no production wiring: explicit error rather than
            // a silent success. Tests and the contract suite use new_noop
            // when they only need trait-bound assertions. Still emit a
            // row so the parity script sees symmetric coverage.
            self.emit_ssb(&req.tool_name, "not_wired", 0);
            return Err(format!(
                "in_process_dispatch: no production tool runner wired (tool={})",
                req.tool_name
            ));
        };
        // dispatch latency measurement is benchmark-intrinsic; injecting a
        // ClockProvider would defeat the purpose (we are measuring the
        // runner closure's wall-clock time so the dashboard surfaces real
        // elapsed_ms in the McpToolCalled SSB row).
        let start = std::time::Instant::now(); // entropy-allow: benchmark-intrinsic-dispatch-latency-measurement
        let result = runner(&req.tool_name, req.args.clone());
        let elapsed_ms = u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
        // Emit one McpToolCalled row per call regardless of success/error.
        // The parity script unions ("mcp_tool_called", "cli_mcp_dispatch")
        // so the two dispatch legs produce equivalent observable streams.
        let outcome = if result.is_ok() { "success" } else { "error" };
        self.emit_ssb(&req.tool_name, outcome, elapsed_ms);
        result
    }
}
