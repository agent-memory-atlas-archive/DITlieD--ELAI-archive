//! C-35: MCP submodules -- gate (dynamic gating) + lazy (lazy schema cache) +
//! plan-compatible alias surface.
//!
//! WIRING_CONTRACT produces clause is satisfied via two free-function wrappers
//! that delegate to the underlying struct methods, plus a re-export of
//! `mcp_bridge` under the plan-quoted `server` path. See
//! workflow/reasoning/C-35-decisions.md (M1, M2, M3) for the rationale.

pub mod gate;
// F-3 Slice C2: CommittedSetChecker object-safe trait seam. Keeps elai-plugins
// free of an elai-orchestrator compile-time dependency while allowing ToolGate
// to enforce commitment checks at call time.
pub mod committed_set;
pub mod lazy;
// E-30: MCP server version pin loader + enforcement backend.
// See workflow/reasoning/E-30-decisions.md M3 for the destination decision
// (extend plugins instead of scaffolding a new crates/mcp-loader crate).
pub mod pins;

/// C-35 M3: plan-quoted path `plugins/src/mcp/server.rs` lands as a re-export
/// of `crate::mcp_bridge`. The live HTTP/stdio MCP transport already ships in
/// mcp_bridge.rs; introducing a duplicate server.rs would split the surface.
/// The re-export means `crate::mcp::server::DefaultMcpBridge` resolves
/// correctly and any audit grep on the plan path finds the right type set.
pub use crate::mcp_bridge as server;

use crate::mcp_bridge::McpToolSpec;
use gate::{GatedCatalog, ToolGate};
use lazy::{LazyCacheError, LazySchemaCache};
use rustc_hash::FxHashMap;

/// C-35 M1: plan-named alias for `ToolGate::filter_for_state`. Renders the
/// gated catalog for a given FSM state. When the gate is `SafeDefault`
/// (manifest missing), every tool falls into `gated_summaries` with
/// `eager_schemas` empty -- the stub_ban in the WIRING_CONTRACT forbids a
/// silent fall-through to the full catalog.
pub fn render_gated_catalog(
    gate: &ToolGate,
    fsm_state: &str,
    full_catalog: &[McpToolSpec],
) -> GatedCatalog {
    gate.filter_for_state(fsm_state, full_catalog)
}

/// C-35 M2: plan-named alias for `LazySchemaCache::resolve`. Fetches the full
/// schema for `tool_name` from `registry`, populating the LRU on miss. Errors
/// when the tool name is not in the registry (no panic).
pub async fn resolve_schema_on_demand(
    cache: &LazySchemaCache,
    tool_name: &str,
    registry: &FxHashMap<String, McpToolSpec>,
) -> Result<McpToolSpec, LazyCacheError> {
    cache.resolve(tool_name, registry).await
}

/// C-35 N3: emitter trait for the 4 tool_* SSB action types. Defined in the
/// plugins crate to avoid a circular dependency on the orchestrator crate.
/// Real implementors live in the orchestrator (and in dashboard-v2 bootstrap
/// where the live `Ssb` writer is wired). `NoopToolGateEmitter` is the
/// default fallback so DefaultMcpBridge stays usable without orchestrator
/// wiring (e.g. in unit tests).
///
/// stub_ban: when `ToolGate::reason() == "manifest_absent"` (the SafeDefault
/// path), callers MUST skip emission entirely rather than write a partial
/// payload. The 3 existing ToolGateEventPayload fields (tool_name, fsm_state,
/// prompt_tokens_saved) are sufficient for the loaded-config path; SafeDefault
/// has no manifest-driven tokens_saved to report. See
/// workflow/reasoning/C-35-decisions.md (schema lock).
pub trait ToolGateSsbEmitter: Send + Sync {
    /// Emit one of the 4 tool_* rows. `action_kind` is one of
    /// "tool_gated" | "tool_eager" | "tool_lazy_resolved" | "tool_denied".
    /// Returns Ok(()) on a successful emit; Err(reason) when the SSB writer
    /// is unavailable -- callers log + continue (never panic).
    fn emit(
        &self,
        action_kind: &str,
        tool_name: &str,
        fsm_state: &str,
        prompt_tokens_saved: i64,
    ) -> Result<(), String>;
}

/// No-op emitter for contexts without an SSB writer (default construction
/// path, unit tests). All 4 emit kinds return Ok(()) without doing anything.
pub struct NoopToolGateEmitter;

impl ToolGateSsbEmitter for NoopToolGateEmitter {
    fn emit(
        &self,
        _action_kind: &str,
        _tool_name: &str,
        _fsm_state: &str,
        _prompt_tokens_saved: i64,
    ) -> Result<(), String> {
        Ok(())
    }
}

/// C-35 emit kinds. Kept as `&'static str` constants because the SsbActionType
/// enum lives in the orchestrator crate (out of reach without circular dep);
/// the orchestrator-side trait impl parses these to the matching variant.
pub mod emit_kinds {
    pub const TOOL_GATED: &str = "tool_gated";
    pub const TOOL_EAGER: &str = "tool_eager";
    pub const TOOL_LAZY_RESOLVED: &str = "tool_lazy_resolved";
    pub const TOOL_DENIED: &str = "tool_denied";
}
