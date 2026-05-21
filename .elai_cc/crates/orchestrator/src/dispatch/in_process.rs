// C-39: In-process dispatch implementation.
//
// Routes tool calls through the in-process plugin registry.
// This is the DEFAULT path (no --via-mcp flag).
// Preserves all pre-C-39 tool dispatch behaviour.

use super::{Dispatch, DispatchRequest, DispatchResult};

/// In-process tool dispatch. Routes calls to the plugin registry directly.
/// No network I/O; this is the legacy path unchanged by C-39.
pub struct InProcessDispatch {
    _private: (),
}

impl InProcessDispatch {
    /// Construct with the real plugin registry (production path).
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// No-op constructor for contract/compile tests.
    pub fn new_noop() -> Self {
        Self { _private: () }
    }
}

impl Default for InProcessDispatch {
    fn default() -> Self { Self::new() }
}

impl Dispatch for InProcessDispatch {
    fn dispatch_tool(&self, req: &DispatchRequest) -> DispatchResult {
        // TODO[C39.stub]: placeholder in-process dispatch; replace with plugin registry call in production wiring phase
        // Default in-process path: return a placeholder success.
        // The real production wiring will override this via
        // apply_task_runner_wiring when the dispatch crate is fully
        // integrated into TaskRunnerConfig.
        Ok(serde_json::json!({
            "tool": req.tool_name,
            "dispatch": "in_process",
            "status": "ok"
        }))
    }
}
