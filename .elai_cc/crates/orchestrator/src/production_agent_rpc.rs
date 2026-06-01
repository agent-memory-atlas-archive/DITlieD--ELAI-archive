// Phase B2c — ProductionAgentRpcServer
//
// Implements the 2-method `AgentRpcServer` trait:
//   - `handle_infer_request`: wraps `AnthropicClient::send_message`.
//   - `handle_tool_request`: translates wire-format `ToolRequest.tool_id` +
//     postcard-encoded `arguments` into the matching `ToolDispatcher::dispatch_*`
//     call via an `Arc<dyn ToolDispatcher>`.
//
// WIRING BACKEND (ELAI_GROUNDING_BACKEND env var, mirrors task_runner.rs:4133):
//   "cloud" -> AnthropicClient::from_env() (reads ANTHROPIC_BASE_URL / auth).
//   Any other value (unset / "sidecar" / empty) ->
//       AnthropicClient::new("local-dummy").with_base_url(ANTHROPIC_BASE_URL
//       env, default http://localhost:8100).
//
// No .unwrap() / .expect() outside #[cfg(test)] or const init.
// All error paths propagate via ToolOutcome::Failed / InferOutcome::Failed.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_core::tool_dispatcher::{
    EditRequest, FailureInput, GitBranchPolicy, GitCommitRequest, IncrementalRequest,
    ToolDispatcher, ValidateRequest,
};
use elai_ipc_types::{
    EntryPointsRequest, ExportGraphMermaidRequest, IndexArtifactsRequest, ListArtifactsRequest,
    RemoveArtifactsRequest, SearchArtifactsRequest, SymbolContextRequest,
    SymbolDataflowImpactRequest, SymbolDslQueryRequest, SymbolFlowRequest,
    SymbolImpactRequest, SymbolListRequest, TOOL_CLASSIFY, TOOL_COMPILE_CHECK,
    TOOL_DKG_CALLERS, TOOL_DKG_DATA_FLOW, TOOL_DKG_DEPENDENTS, TOOL_DKG_SUBGRAPH, TOOL_EDIT,
    TOOL_ENTRY_POINTS, TOOL_EXPORT_GRAPH_MERMAID, TOOL_FILE_READ, TOOL_GIT_BRANCH,
    TOOL_GIT_COMMIT, TOOL_GIT_UNDO, TOOL_INDEX_ARTIFACTS, TOOL_LIST_ARTIFACTS,
    TOOL_REMOVE_ARTIFACTS, TOOL_SEARCH_ARTIFACTS, TOOL_SYMBOL_CONTEXT,
    TOOL_SYMBOL_DATAFLOW_IMPACT, TOOL_SYMBOL_DSL_QUERY, TOOL_SYMBOL_FLOW,
    TOOL_SYMBOL_IMPACT, TOOL_SYMBOL_LIST, TOOL_VALIDATE,
};
use elai_knowledge::artifacts::{
    store::ArtifactStore,
    search::artifact_search_with_precomputed_embedding,
};
use elai_knowledge::symbol_graph::{
    cache::SymbolGraphCache,
    entry_points::{detect_entry_points, DetectedEntryPoint, EntryPointReason, EntryPointSummary},
    sharding::SYMBOL_FILE_LRU_SIZE,
    traversal::{compute_call_flow, compute_impact_radius, compute_symbol_context, list_symbols, SymbolFilter},
};
use elai_providers::{
    AnthropicClient, InputMessage, MessageRequest, OutputContentBlock, SystemBlock, ToolChoice,
    ToolDefinition,
};
use serde_json::Value;

use crate::ipc_accept::{AgentRpcServer, InferOutcome, ToolOutcome};


/// Default sidecar base URL when ANTHROPIC_BASE_URL is unset.
const DEFAULT_SIDECAR_URL: &str = "http://localhost:8100";

/// Default model used when constructing a `MessageRequest`.
const DEFAULT_MODEL: &str = "claude-sonnet-4-6";

/// Default max_tokens used when the caller does not override.
const DEFAULT_MAX_TOKENS: u32 = 4096;

/// Outline-mode threshold: files above this many lines get a signatures-only
/// view from `dispatch_file_read_outline`. Matches
/// `elai_tools::dispatcher::OUTLINE_LINE_THRESHOLD`.
const OUTLINE_LINE_THRESHOLD: usize = 300;

/// patch-R5 — canonical system prompt biasing Sonnet into invoking tools
/// from the 6-tool wire catalog. Without this, B3 cloud E2E observed Sonnet
/// returning text-only responses and the agent's `run_react_turn` exiting
/// after one iteration with zero `TOOL_USE` markers parsed and no
/// `FileWrite` SSB row written (see R4 handoff `e2e_cloud_sonnet_run_v9.log`,
/// L737 panic `fw_rows=0`).
///
/// Production call sites (B3 e2e, CLI `elai task`, dashboard "Run" button)
/// pass this constant via `ProductionAgentRpcServer::new(...).with_system_prompt(
/// DEFAULT_BABY_STEP_SYSTEM_PROMPT)`. The wire catalog (R3) only publishes
/// the six tools enumerated below; their exact names match the
/// `elai_ipc_types::TOOL_*` wire IDs.
pub const DEFAULT_BABY_STEP_SYSTEM_PROMPT: &str = "You are an autonomous coding agent embedded in the ELAI orchestrator runtime. \
Your single job is to complete the user's coding task by invoking tools from the catalog provided in this request. \
The catalog has fourteen tools (wire IDs in lowercase): edit, file_read, validate, compile_check, classify, git_commit, symbol_impact, symbol_flow, symbol_context, symbol_list, dkg_callers, dkg_dependents, dkg_data_flow, dkg_subgraph. \
The four symbol_* tools query the project's symbol-graph (call edges + reverse-dependency index) — use them when you need to know all callers of a function before editing it, or to discover symbols in a file. \
The four dkg_* tools query the dependency knowledge graph (callers / dependents / data-flow edges / BFS subgraph) when you need symbol-level traversal beyond the symbol-graph reverse index. \
Hard rules: (1) every turn that performs work MUST include a tool call -- prose-only replies produce no file changes and waste the turn; \
(2) the edit tool requires the target file to already exist on disk and replaces an exact `old_text` fragment with `new_text` -- so the first thing you usually need is a file_read of the existing scaffold (e.g. src/App.jsx) to learn what to replace; \
(3) provide complete arguments matching each tool's input_schema (file_path, old_text, new_text for edit; file_path for file_read; etc.); \
(4) issue at most one tool call per turn -- after a tool result comes back, decide the next call; \
(5) when the requested artifact exists with the correct content, reply WITHOUT any tool call and the loop will terminate. \
Project paths are repo-relative unless absolute. Do not call validate / compile_check / git_commit until at least one edit has succeeded.";

// ---------------------------------------------------------------------------
// ProductionAgentRpcServer
// ---------------------------------------------------------------------------

/// Production implementation of `AgentRpcServer`.
///
/// - `client`: AnthropicClient pointed at either the remote sidecar or the
///   Anthropic cloud API, depending on `ELAI_GROUNDING_BACKEND`.
/// - `dispatcher`: `Arc<dyn ToolDispatcher>` for in-process tool calls
///   (type-erased so production code does not pull in `elai-tools`).
/// - `model`: model identifier forwarded on every `MessageRequest`.
/// - `max_tokens`: token budget forwarded on every `MessageRequest`.
pub struct ProductionAgentRpcServer {
    client: AnthropicClient,
    dispatcher: Arc<dyn ToolDispatcher>,
    model: String,
    max_tokens: u32,
    /// Optional SSB handle used by the B-29 Phase 5 symbol-graph tools to
    /// append SymbolQuery action rows for the dashboard ActivityFeed.
    /// `None` for harness/test paths that do not wire an SSB instance.
    ssb: Option<Arc<crate::ssb::Ssb>>,
    /// Optional VectorModule used by the `search_artifacts` dispatcher arm
    /// (B-35 Phase B). When `None`, artifact search falls back to returning
    /// an empty result set (mirrors `artifact_search` graceful-degrade behavior).
    /// Deferred wiring (B-35 Phase C2): inject via a proper service-locator so
    /// per-project VectorModules can be selected by db_path. Set via
    /// `with_artifact_vm` at server startup.
    artifact_vm: Option<Arc<dyn elai_knowledge::vector::VectorModule + Send + Sync>>,
    /// patch-R5 — optional system prompt forwarded as a one-block
    /// `SystemBlock` array on every `MessageRequest`. `None` (the default)
    /// preserves R3.1 behaviour (`request.system = None`, no leak into the
    /// 10+ existing 4-arg call sites). Production wires
    /// `DEFAULT_BABY_STEP_SYSTEM_PROMPT` via `with_system_prompt(...)` so
    /// Sonnet emits `ToolUse` blocks instead of text-only replies.
    system_prompt: Option<String>,
    /// patch-R3 — per-server stash of `OutputContentBlock::ToolUse.input` JSON
    /// values, keyed by tool name and ordered FIFO for repeated names.
    /// `handle_infer_request` populates this when Sonnet returns ToolUse
    /// blocks (emitting `"TOOL_USE: <name>"` markers in the response text per
    /// agent_loop.rs:79 contract); `handle_tool_request` pops a value when the
    /// agent sends `ToolRequest.arguments = Vec::new()` (agent_loop.rs:170) and
    /// converts JSON -> postcard via `convert_json_input_to_postcard`.
    /// `tokio::sync::Mutex` mandated because the lock is held across `.await`
    /// in `handle_infer_request` (push loop after `send_message` resolves);
    /// `std::sync::Mutex` would prevent `ProductionAgentRpcServer` from being
    /// usable behind `Arc<dyn AgentRpcServer>` whose methods are async.
    pub pending_tool_args: Arc<tokio::sync::Mutex<HashMap<String, VecDeque<Value>>>>,
    /// C-15: ClockProvider used for all timing inside tool dispatch (latency
    /// measurement, artifact indexing `now_secs`). Defaults to `OsClock`;
    /// callers can override via `with_clock()` for deterministic tests.
    clock: Arc<dyn ClockProvider>,
    /// E-40 / s-22600 Tranche B ITEM B.4 — optional SymbolGraph adapter
    /// used by the `TOOL_SYMBOL_DSL_QUERY` + `TOOL_SYMBOL_DATAFLOW_IMPACT`
    /// dispatcher arms. When `None`, both arms continue to return typed
    /// empty responses (preserves the pre-B.4 wire shape for harness /
    /// test paths that do not wire a real cache). When `Some`, the arms
    /// dispatch the DSL / impact query through the adapter and return
    /// real results, while ALSO emitting the existing `emit_symbol_query`
    /// row (additive observability) plus a new `SymbolGraphEvent` row
    /// (SymbolDslExecuted / DataflowImpactComputed) for the dashboard.
    /// Set via `with_symbol_graph_adapter` at CLI startup
    /// (`live_cli::build_local_knowledge_stack`).
    symbol_graph_adapter:
        Option<Arc<dyn elai_knowledge::symbol_graph_adapter::SymbolGraphAdapter>>,
}

impl ProductionAgentRpcServer {
    /// Construct a server with an already-built `AnthropicClient`.
    pub fn new(
        client: AnthropicClient,
        dispatcher: Arc<dyn ToolDispatcher>,
        model: String,
        max_tokens: u32,
    ) -> Self {
        Self {
            client,
            dispatcher,
            model,
            max_tokens,
            ssb: None,
            artifact_vm: None,
            system_prompt: None,
            pending_tool_args: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            clock: Arc::new(OsClock),
            symbol_graph_adapter: None,
        }
    }

    /// C-15: builder-style setter for the `ClockProvider`. Production keeps the
    /// `OsClock` default; deterministic tests pass `SeededClock`.
    #[must_use]
    pub fn with_clock(mut self, clock: Arc<dyn ClockProvider>) -> Self {
        self.clock = clock;
        self
    }

    /// Builder-style setter for the SSB writer used by symbol-graph tool arms.
    /// Returns `self` so call sites can chain.
    pub fn with_ssb(mut self, ssb: Arc<crate::ssb::Ssb>) -> Self {
        self.ssb = Some(ssb);
        self
    }

    /// patch-R5 — builder-style setter for the system prompt forwarded on
    /// every `MessageRequest.system`. Production passes
    /// `DEFAULT_BABY_STEP_SYSTEM_PROMPT`; tests pass whatever string they
    /// want to assert on. Existing 4-arg `new()` call sites that don't
    /// chain `.with_system_prompt(...)` keep shipping `system: None`,
    /// preserving R3.1 wire behaviour.
    #[must_use]
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Builder-style setter for the VectorModule used by the `search_artifacts`
    /// dispatcher arm (B-35 Phase B). When not set, artifact search returns
    /// an empty result set (graceful degrade — no embedding available).
    pub fn with_artifact_vm(
        mut self,
        vm: Arc<dyn elai_knowledge::vector::VectorModule + Send + Sync>,
    ) -> Self {
        self.artifact_vm = Some(vm);
        self
    }

    /// E-40 / s-22600 Tranche B ITEM B.4 — builder-style setter for the
    /// SymbolGraph adapter used by the `TOOL_SYMBOL_DSL_QUERY` +
    /// `TOOL_SYMBOL_DATAFLOW_IMPACT` dispatcher arms. When not set, both
    /// arms return typed empty responses (pre-B.4 behaviour preserved).
    /// CLI startup chains this on the
    /// `ProductionAgentRpcServer::with_*` builder to plumb a real
    /// `SymbolGraphCacheAdapter` (orchestrator/src/symbol_graph/mod.rs).
    #[must_use]
    pub fn with_symbol_graph_adapter(
        mut self,
        adapter: Arc<dyn elai_knowledge::symbol_graph_adapter::SymbolGraphAdapter>,
    ) -> Self {
        self.symbol_graph_adapter = Some(adapter);
        self
    }

    /// patch-R3 — Tool catalog published in `MessageRequest.tools` so Sonnet
    /// can decide which tool to call. Names are the wire IDs from
    /// `elai_ipc_types::TOOL_*` constants (NOT dispatcher method names).
    /// `input_schema` keys mirror the matching `elai_ipc_types::*Request`
    /// struct fields verbatim so `convert_json_input_to_postcard` can
    /// `serde_json::from_value::<*Request>` without rename overhead.
    ///
    /// Included with reduced schema (rationale per context-package):
    ///   - `TOOL_FILE_READ`: schema omits `offset`/`limit` because the
    ///     dispatcher arm at `translate_tool_request` rejects either being
    ///     `Some` (windowed reads unsupported).
    ///
    /// C-24 (DECISION-14): the 4 B-29 Phase 5 symbol-graph tools
    /// (`symbol_impact`, `symbol_flow`, `symbol_context`, `symbol_list`) are
    /// NOW INCLUDED. Their dispatcher arms at translate_tool_request
    /// (lines ~907-1072) are wired and `convert_json_input_to_postcard`
    /// also has matching arms — the agent can self-select them and the
    /// JSON->postcard bridge will succeed.
    ///
    /// Excluded entirely (would yield guaranteed-failure tool calls):
    ///   - `TOOL_INDEX_ARTIFACTS`: dispatcher arm returns `Err` (Phase C stub).
    ///   - `TOOL_GIT_UNDO`, `TOOL_GIT_BRANCH`: dispatcher arms return `Err`
    ///     ("not implemented in ProductionAgentRpcServer (B2c)").
    ///   - Artifact-store tools (`list_artifacts`, `search_artifacts`,
    ///     `remove_artifacts`, `entry_points`): deferred to a follow-up
    ///     catalog patch — their dispatcher arms exist but no matching
    ///     `convert_json_input_to_postcard` mapping yet.
    fn build_tool_catalog(&self) -> Vec<ToolDefinition> {
        let edit = ToolDefinition {
            name: TOOL_EDIT.to_string(),
            description: Some(
                "Apply a literal text replacement to a file. Requires a single \
                 exact match of `old_text` inside the file at `file_path`."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Absolute or repo-relative path to the file to edit."
                    },
                    "old_text": {
                        "type": "string",
                        "description": "Exact text fragment to be replaced (must occur once)."
                    },
                    "new_text": {
                        "type": "string",
                        "description": "Replacement text."
                    }
                },
                "required": ["file_path", "old_text", "new_text"]
            }),
            cache_control: None,
        };
        let file_read = ToolDefinition {
            name: TOOL_FILE_READ.to_string(),
            description: Some(
                "Read a file as an outline (signatures + imports for files larger \
                 than 300 lines, otherwise the full content). Windowed reads \
                 (offset/limit) are not supported by this dispatcher."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to the file to read."
                    }
                },
                "required": ["file_path"]
            }),
            cache_control: None,
        };
        let validate = ToolDefinition {
            name: TOOL_VALIDATE.to_string(),
            description: Some(
                "Run the tiered validator (determinism / quality-AST / unit) for \
                 the project directory that contains `file_path`."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path inside the project directory to validate."
                    },
                    "language": {
                        "type": "string",
                        "description": "Source language (e.g. rust, python, typescript, go)."
                    }
                },
                "required": ["file_path", "language"]
            }),
            cache_control: None,
        };
        let compile_check = ToolDefinition {
            name: TOOL_COMPILE_CHECK.to_string(),
            description: Some(
                "Run an incremental compile check for the project directory that \
                 contains `file_path` (cargo check / tsc --noEmit / equivalent)."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path inside the project directory to check."
                    },
                    "language": {
                        "type": "string",
                        "description": "Source language (e.g. rust, python, typescript)."
                    },
                    "retry_round": {
                        "type": "integer",
                        "minimum": 0,
                        "maximum": 255,
                        "description": "Retry counter; 0 on first invocation."
                    }
                },
                "required": ["file_path", "language", "retry_round"]
            }),
            cache_control: None,
        };
        let classify = ToolDefinition {
            name: TOOL_CLASSIFY.to_string(),
            description: Some(
                "Classify a raw build / test failure string into one of the known \
                 failure categories. Returns `null` when no classifier matches."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "error_output": {
                        "type": "string",
                        "description": "Raw build / test failure output to classify."
                    },
                    "file_path": {
                        "type": ["string", "null"],
                        "description": "Optional source file the failure refers to."
                    }
                },
                "required": ["error_output"]
            }),
            cache_control: None,
        };
        let git_commit = ToolDefinition {
            name: TOOL_GIT_COMMIT.to_string(),
            description: Some(
                "Commit the supplied paths under the active per-plan branch. \
                 Requires at least one path."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Commit message."
                    },
                    "paths": {
                        "type": "array",
                        "items": {"type": "string"},
                        "minItems": 1,
                        "description": "Repo-relative paths to stage and commit."
                    }
                },
                "required": ["message", "paths"]
            }),
            cache_control: None,
        };
        // C-24 (DECISION-1): the 4 B-29 Phase 5 symbol-graph tools.
        // Schema field names mirror the wire structs in
        // `elai_ipc_types::Symbol*Request` (lines 542-571) verbatim so
        // `convert_json_input_to_postcard` can `serde_json::from_value`
        // without rename overhead — same patch-R3 contract as the 6
        // tools above.
        //
        // `project_root` is required on every request — the dispatcher arms
        // (translate_tool_request L907-1072) Err immediately if the file_name
        // of the path is empty, because that's how `project_id` is derived
        // before opening the SymbolGraphCache. The agent's system prompt
        // surface (DEFAULT_BABY_STEP_SYSTEM_PROMPT) does not currently
        // inject `project_root`; the spawned agent is expected to mirror
        // the absolute project root surfaced via the task brief.
        let symbol_impact = ToolDefinition {
            name: TOOL_SYMBOL_IMPACT.to_string(),
            description: Some(
                "Compute the reverse-dependency blast radius of a symbol (BFS \
                 over the symbol call-graph). Returns the set of files that \
                 transitively call `target`, partitioned by hop distance up \
                 to `depth`."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root \
                         (used to derive project_id + locate the symbol-graph DB)."
                    },
                    "target": {
                        "type": "string",
                        "description": "Qualified symbol name to start the reverse traversal at."
                    },
                    "depth": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 16,
                        "description": "BFS depth cap (recommended default: 2)."
                    }
                },
                "required": ["project_root", "target", "depth"]
            }),
            cache_control: None,
        };
        let symbol_flow = ToolDefinition {
            name: TOOL_SYMBOL_FLOW.to_string(),
            description: Some(
                "Compute the forward call-flow from an entrypoint symbol (DFS \
                 over the symbol call-graph). Returns the call tree rooted at \
                 `entrypoint_id` truncated to `depth` levels."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root \
                         (used to derive project_id + locate the symbol-graph DB)."
                    },
                    "entrypoint_id": {
                        "type": "string",
                        "description": "Stable symbol id (typically qualified symbol name) \
                         to start the call-flow traversal at."
                    },
                    "depth": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 16,
                        "description": "DFS depth cap (recommended default: 5)."
                    }
                },
                "required": ["project_root", "entrypoint_id", "depth"]
            }),
            cache_control: None,
        };
        let symbol_context = ToolDefinition {
            name: TOOL_SYMBOL_CONTEXT.to_string(),
            description: Some(
                "Return a 360-degree view of a symbol definition: matching \
                 symbol(s) (by simple or qualified name) with file/line, kind, \
                 and a short callers/callees neighbourhood. Useful before edits \
                 to a symbol whose call-site count you do not yet know."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root."
                    },
                    "name": {
                        "type": "string",
                        "description": "Symbol name (simple or qualified) to look up."
                    },
                    "file_hint": {
                        "type": ["string", "null"],
                        "description": "Optional file path to disambiguate when multiple \
                         symbols share the same name."
                    }
                },
                "required": ["project_root", "name"]
            }),
            cache_control: None,
        };
        let symbol_list = ToolDefinition {
            name: TOOL_SYMBOL_LIST.to_string(),
            description: Some(
                "List symbols defined in the project, optionally filtered by \
                 file and/or fuzzy query string. Use when planning before edits \
                 to know which symbols already exist. Server caps `limit` at 500."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root."
                    },
                    "file": {
                        "type": ["string", "null"],
                        "description": "Optional file path to scope the listing."
                    },
                    "query": {
                        "type": ["string", "null"],
                        "description": "Optional substring filter on symbol name."
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 500,
                        "description": "Result cap (server clamps at 500)."
                    }
                },
                "required": ["project_root", "limit"]
            }),
            cache_control: None,
        };
        // C-30: four new DKG query tools (callers / dependents / data_flow /
        // subgraph). Input schema mirrors the helper signatures on
        // `DkgQuerierAdapter`: `target` (string) + `max_depth` (u8) for
        // callers/dependents/data_flow; `root` (string) + `depth` (u8) for
        // subgraph. `project_root` is included so the dispatcher arm
        // (translate_tool_request) can locate the per-project DKG db.
        let dkg_callers = ToolDefinition {
            name: TOOL_DKG_CALLERS.to_string(),
            description: Some(
                "Return the symbols that CALL `target` (inbound CALLS edges \
                 in the dependency knowledge graph). Useful for change-impact \
                 analysis before editing a function. Depth is capped at 8."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root \
                         (used to locate the per-project DKG database)."
                    },
                    "target": {
                        "type": "string",
                        "description": "Qualified symbol name to start inbound traversal at."
                    },
                    "max_depth": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 8,
                        "description": "BFS depth cap (recommended default: 3)."
                    }
                },
                "required": ["project_root", "target", "max_depth"]
            }),
            cache_control: None,
        };
        let dkg_dependents = ToolDefinition {
            name: TOOL_DKG_DEPENDENTS.to_string(),
            description: Some(
                "Return the symbols / files that `target` depends on (outbound \
                 IMPORTS / USES edges). Use when planning to refactor a module \
                 to see what it touches transitively. Depth is capped at 8."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root."
                    },
                    "target": {
                        "type": "string",
                        "description": "Qualified symbol name to start outbound traversal at."
                    },
                    "max_depth": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 8,
                        "description": "BFS depth cap (recommended default: 3)."
                    }
                },
                "required": ["project_root", "target", "max_depth"]
            }),
            cache_control: None,
        };
        let dkg_data_flow = ToolDefinition {
            name: TOOL_DKG_DATA_FLOW.to_string(),
            description: Some(
                "Return the data-flow edges (PARAMETER / RETURN / ASSIGNMENT / \
                 FIELD_ACCESS) reachable from `target`. Each result is a \
                 (source, target, variable, flow_type) tuple. Depth is capped \
                 at 8."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root."
                    },
                    "target": {
                        "type": "string",
                        "description": "Symbol to start data-flow traversal at."
                    },
                    "max_depth": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 8,
                        "description": "BFS depth cap (recommended default: 3)."
                    }
                },
                "required": ["project_root", "target", "max_depth"]
            }),
            cache_control: None,
        };
        let dkg_subgraph = ToolDefinition {
            name: TOOL_DKG_SUBGRAPH.to_string(),
            description: Some(
                "BFS-extract the subgraph rooted at `root` up to `depth` hops. \
                 Returns nodes + edges for compact agent-context rendering. \
                 Depth is capped at 8."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root."
                    },
                    "root": {
                        "type": "string",
                        "description": "Symbol to seed the BFS at."
                    },
                    "depth": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 8,
                        "description": "BFS depth cap (recommended default: 2)."
                    }
                },
                "required": ["project_root", "root", "depth"]
            }),
            cache_control: None,
        };
        // E-40 Tranche B-2: SymbolGraph DSL + dataflow tools. Distinct
        // from the B-29 symbol_* family above (those query the
        // SQLite-rooted cache; these query the orchestrator-rooted
        // JSON cache via the new SymbolGraphAdapter). Mirrors the
        // TOOL_SYMBOL_IMPACT input-schema shape (project_root + bounded
        // depth) plus a `dsl` / `symbol` field per tool.
        let symbol_dsl_query = ToolDefinition {
            name: TOOL_SYMBOL_DSL_QUERY.to_string(),
            description: Some(
                "Execute a Datalog-style DSL query (DEF/USE/IMPL/CALL \
                 selectors with lang/file/depth filters) over the \
                 orchestrator-rooted JSON symbol graph. Returns matching \
                 symbols plus any dataflow side-context edges, bounded by \
                 `depth_cap` (default 2)."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root."
                    },
                    "dsl": {
                        "type": "string",
                        "description": "DSL query string (e.g. DEF(foo) | lang(rust))."
                    },
                    "depth_cap": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 5,
                        "description": "Dataflow side-context hop cap (default 2)."
                    }
                },
                "required": ["project_root", "dsl", "depth_cap"]
            }),
            cache_control: None,
        };
        let symbol_dataflow_impact = ToolDefinition {
            name: TOOL_SYMBOL_DATAFLOW_IMPACT.to_string(),
            description: Some(
                "Compute the dataflow / reverse-call impact radius of a \
                 symbol via BFS over the JSON symbol graph's reverse-call \
                 index, bounded by `depth_cap` (default 2). Returns the set \
                 of transitively-impacted symbol names plus the actual \
                 depth reached and edge count."
                    .to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "project_root": {
                        "type": "string",
                        "description": "Absolute path to the project root."
                    },
                    "symbol": {
                        "type": "string",
                        "description": "Symbol name to start the reverse traversal at."
                    },
                    "depth_cap": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 5,
                        "description": "BFS depth cap (default 2)."
                    }
                },
                "required": ["project_root", "symbol", "depth_cap"]
            }),
            cache_control: None,
        };
        vec![
            edit, file_read, validate, compile_check, classify, git_commit,
            // C-24: B-29 Phase 5 symbol-graph tools, now agent-visible.
            symbol_impact, symbol_flow, symbol_context, symbol_list,
            // C-30: dependency-knowledge-graph query tools, now agent-visible.
            dkg_callers, dkg_dependents, dkg_data_flow, dkg_subgraph,
            // E-40 Tranche B-2: SymbolGraph DSL + dataflow tools.
            symbol_dsl_query, symbol_dataflow_impact,
        ]
    }

    /// Construct from environment variables, mirroring the `task_runner.rs`
    /// `ELAI_GROUNDING_BACKEND` switch (L4133):
    ///   - `Some("cloud")` -> `AnthropicClient::from_env()`.
    ///   - Any other value -> sidecar default.
    ///
    /// Returns `None` when the cloud path is requested but credentials are
    /// missing; the caller should fall back to `NoOpAgentRpcServer`.
    pub fn from_env(dispatcher: Arc<dyn ToolDispatcher>) -> Option<Self> {
        let backend = std::env::var("ELAI_GROUNDING_BACKEND").ok();
        let client = if backend.as_deref() == Some("cloud") {
            match AnthropicClient::from_env() {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "ProductionAgentRpcServer: cloud credentials unavailable; \
                         caller should fall back to NoOpAgentRpcServer"
                    );
                    return None;
                }
            }
        } else {
            let base_url = std::env::var("ANTHROPIC_BASE_URL")
                .unwrap_or_else(|_| DEFAULT_SIDECAR_URL.to_string());
            AnthropicClient::new("local-dummy").with_base_url(base_url)
        };

        let model = std::env::var("ELAI_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
        let max_tokens = std::env::var("ELAI_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(DEFAULT_MAX_TOKENS);

        Some(Self::new(client, dispatcher, model, max_tokens))
    }
}

// ---------------------------------------------------------------------------
// AgentRpcServer implementation
// ---------------------------------------------------------------------------

impl AgentRpcServer for ProductionAgentRpcServer {
    fn handle_infer_request<'a>(
        &'a self,
        prompt: String,
        max_tokens: u32,
    ) -> Pin<Box<dyn std::future::Future<Output = InferOutcome> + Send + 'a>> {
        Box::pin(async move {
            // Use the frame-supplied max_tokens; fall back to server default
            // when the frame carries 0 (not yet set by caller).
            let effective_max_tokens = if max_tokens == 0 {
                self.max_tokens
            } else {
                max_tokens
            };
            // patch-R5 — wire system_prompt (when set) as a one-block
            // `SystemBlock` array, and pin `tool_choice` to `Auto`. R3 always
            // publishes the 6-tool catalog, so `tool_choice` must always be
            // explicit; `Auto` lets Sonnet interleave reasoning text with
            // ToolUse blocks (per Anthropic docs, `Any` would prefill the
            // assistant turn and suppress natural-language reasoning).
            let system = self
                .system_prompt
                .as_ref()
                .map(|p| vec![SystemBlock::text(p.clone())]);
            let request = MessageRequest {
                model: self.model.clone(),
                max_tokens: effective_max_tokens,
                messages: vec![InputMessage::user_text(prompt)],
                system,
                tools: Some(self.build_tool_catalog()),
                tool_choice: Some(ToolChoice::Auto),
                stream: false,
                seed: None,
            };

            match self.client.send_message(&request).await {
                Ok(response) => {
                    // patch-R3 — collapse `response.content` into the agent's
                    // text-based ReAct surface:
                    //   - Each `Text` block contributes its text verbatim.
                    //   - Each `ToolUse` block contributes a `"TOOL_USE: <name>"`
                    //     marker line (agent_loop.rs:79 marker contract) and
                    //     stashes its `input` under `name` for the upcoming
                    //     `ToolRequest` whose `arguments` are empty
                    //     (agent_loop.rs:170).
                    //   - `Thinking` / `RedactedThinking` blocks are dropped
                    //     (the agent has no parser for them today).
                    let mut text_blocks: Vec<String> = Vec::new();
                    let mut markers: Vec<String> = Vec::new();
                    let mut to_stash: Vec<(String, Value)> = Vec::new();

                    for block in response.content.into_iter() {
                        match block {
                            OutputContentBlock::Text { text } => text_blocks.push(text),
                            OutputContentBlock::ToolUse { id: _, name, input } => {
                                markers.push(format!("TOOL_USE: {name}"));
                                to_stash.push((name, input));
                            }
                            OutputContentBlock::Thinking { .. }
                            | OutputContentBlock::RedactedThinking { .. } => {}
                        }
                    }

                    if text_blocks.is_empty() && markers.is_empty() {
                        return InferOutcome::Failed(
                            "provider returned no text or tool_use content blocks"
                                .to_string(),
                        );
                    }

                    // Single lock acquisition for all pushes; dropped before
                    // we build the response string. Never held across the
                    // earlier `send_message().await`.
                    if !to_stash.is_empty() {
                        let mut pending = self.pending_tool_args.lock().await;
                        for (name, value) in to_stash {
                            pending.entry(name).or_default().push_back(value);
                        }
                    }

                    let separator = if !text_blocks.is_empty() && !markers.is_empty() {
                        "\n"
                    } else {
                        ""
                    };
                    let combined = format!(
                        "{}{}{}",
                        text_blocks.join("\n"),
                        separator,
                        markers.join("\n")
                    );
                    InferOutcome::Response(combined)
                }
                Err(e) => InferOutcome::Failed(format!("{e:?}")),
            }
        })
    }

    fn handle_tool_request<'a>(
        &'a self,
        tool_id: String,
        arguments: Vec<u8>,
    ) -> Pin<Box<dyn std::future::Future<Output = ToolOutcome> + Send + 'a>> {
        Box::pin(async move {
            // patch-R3 — `agent_loop` sends `ToolRequest.arguments = Vec::new()`
            // (agent_loop.rs:170) because its in-text `TOOL_USE: <id>` marker
            // carries no payload. When `arguments` is empty, try to pop the
            // corresponding stash entry seeded by `handle_infer_request` and
            // convert the JSON input to the postcard bytes the dispatcher arms
            // expect. A stash miss falls through to `translate_tool_request`
            // with the original empty `arguments`, which yields either
            // `"unknown tool_id:"` (preserves the test-c contract) or a
            // postcard decode error for a real-but-empty tool_id.
            let effective_arguments: Vec<u8> = if arguments.is_empty() {
                let popped = {
                    let mut pending = self.pending_tool_args.lock().await;
                    pending.get_mut(&tool_id).and_then(|q| q.pop_front())
                };
                match popped {
                    Some(value) => match convert_json_input_to_postcard(&tool_id, value) {
                        Ok(bytes) => bytes,
                        Err(reason) => return ToolOutcome::Failed(reason),
                    },
                    None => arguments,
                }
            } else {
                arguments
            };

            match translate_tool_request(
                &tool_id,
                &effective_arguments,
                self.dispatcher.as_ref(),
                self.ssb.as_ref(),
                self.artifact_vm.as_ref().map(Arc::as_ref),
                &self.clock,
                self.symbol_graph_adapter.as_ref(),
            )
            .await
            {
                Ok(bytes) => ToolOutcome::Result(bytes),
                Err(reason) => ToolOutcome::Failed(reason),
            }
        })
    }
}

// ---------------------------------------------------------------------------
// convert_json_input_to_postcard — patch-R3
// ---------------------------------------------------------------------------

/// patch-R3 — Convert a Sonnet `OutputContentBlock::ToolUse.input` JSON value
/// into the postcard-encoded wire bytes the matching dispatcher arm expects.
///
/// Each known `tool_id` is decoded into its `elai_ipc_types::*Request` wire
/// struct first (the typed deserialisation is itself the input-validation
/// gate — attacker-controlled JSON cannot synthesise an arbitrary postcard
/// frame because every field must satisfy the struct's `serde` schema), then
/// `postcard::to_allocvec` produces the byte payload that
/// `translate_tool_request` re-decodes.
///
/// Returns `Err(String)` for an unknown `tool_id`, JSON decode failure, or
/// postcard encode failure. Errors propagate as `ToolOutcome::Failed` so the
/// agent records the failure and continues its ReAct loop.
fn convert_json_input_to_postcard(tool_id: &str, input: Value) -> Result<Vec<u8>, String> {
    match tool_id {
        TOOL_EDIT => {
            let req: elai_ipc_types::EditRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_FILE_READ => {
            let req: elai_ipc_types::FileReadRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_VALIDATE => {
            let req: elai_ipc_types::ValidateRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_COMPILE_CHECK => {
            let req: elai_ipc_types::CompileCheckRequest =
                serde_json::from_value(input).map_err(|e| {
                    format!(
                        "convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}"
                    )
                })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_CLASSIFY => {
            let req: elai_ipc_types::ClassifyRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_GIT_COMMIT => {
            let req: elai_ipc_types::GitCommitRequest =
                serde_json::from_value(input).map_err(|e| {
                    format!(
                        "convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}"
                    )
                })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        // C-24 (DRIFT-12): JSON->postcard bridge for the 4 B-29 Phase 5
        // symbol-graph tools now exposed in the catalog. Field names
        // match the wire structs in ipc-types verbatim — see DECISION-1.
        TOOL_SYMBOL_IMPACT => {
            let req: SymbolImpactRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_SYMBOL_FLOW => {
            let req: SymbolFlowRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_SYMBOL_CONTEXT => {
            let req: SymbolContextRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_SYMBOL_LIST => {
            let req: SymbolListRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        // E-40 Tranche B-2: SymbolGraph DSL + dataflow tools.
        TOOL_SYMBOL_DSL_QUERY => {
            let req: SymbolDslQueryRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        TOOL_SYMBOL_DATAFLOW_IMPACT => {
            let req: SymbolDataflowImpactRequest = serde_json::from_value(input).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' input decode error: {e}")
            })?;
            postcard::to_allocvec(&req).map_err(|e| {
                format!("convert_json_input_to_postcard: tool '{tool_id}' postcard encode error: {e}")
            })
        }
        unknown => Err(format!(
            "convert_json_input_to_postcard: no mapping for tool_id '{unknown}'"
        )),
    }
}

// ---------------------------------------------------------------------------
// translate_tool_request — private dispatch helper
// ---------------------------------------------------------------------------

/// Deserialise `arguments` (postcard-encoded ipc-types wire struct) for the
/// given `tool_id`, call the matching `ToolDispatcher` method, and serialise
/// the result to JSON bytes.
///
/// Returns `Err(String)` for:
///   - unknown `tool_id`
///   - postcard decode failure
///   - dispatcher error
///   - JSON serialisation failure
/// Symbol-graph tool agent_id used when appending SymbolQuery SSB rows.
/// Distinct from per-baby-step agent IDs so dashboard filters can isolate
/// symbol-graph activity from regular agent tool calls.
const SSB_AGENT_SYMBOL_GRAPH: &str = "symbol_graph_tool";

/// C-8 Task #5: walk up from a `context_*.db` path to the project root.
/// Typical layout is `<project_root>/.elai/indexes/context_*.db`, so the
/// project root is the grandparent's parent. Returns `None` if the path
/// lacks at least three ancestor levels.
fn derive_project_root_from_db_path(db_path: &std::path::Path) -> Option<std::path::PathBuf> {
    db_path
        .parent()? // .elai/indexes
        .parent()? // .elai
        .parent() // <project_root>
        .map(|p| p.to_path_buf())
}

/// C-8 Task #5: derive a stable project_id from the canonical project root
/// path. Mirrors `metadata::project_id_from_path` semantics without pulling
/// the helper into the public surface — we just need the file_name as the
/// project_id label on the SSB payload.
fn derive_project_id_from_root(project_root: &std::path::Path) -> String {
    project_root
        .file_name()
        .and_then(|os| os.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Best-effort SSB write — never fails the tool call. Logs a warn on error.
async fn emit_symbol_query(
    ssb: Option<&Arc<crate::ssb::Ssb>>,
    tool_id: &str,
    target: String,
    hop_count: u32,
    total_files: u32,
    latency_ms: u32,
) {
    let Some(ssb) = ssb else { return };
    use crate::ssb::{SsbPayload, SymbolQueryPayload};
    let payload = SsbPayload::SymbolQuery(SymbolQueryPayload {
        tool: tool_id.to_string(),
        target,
        hop_count,
        total_files,
        latency_ms,
    });
    if let Err(e) = ssb
        .write_payload(SSB_AGENT_SYMBOL_GRAPH, 0, payload, Vec::new())
        .await
    {
        tracing::warn!(
            tool = tool_id,
            error = %e,
            "symbol_query SSB write failed (continuing tool result)"
        );
    }
}

// C-18 D1: visibility raised from private to `pub(crate)` so the in-crate
// integration tests under `crates/orchestrator/tests/` can reach it via
// the public test-helper alias `translate_tool_request_for_test` (gated by
// `test-support` feature). No change to the function body. The crate-public
// surface stays the same for downstream crates (they cannot name `pub(crate)`
// items either way), and the production agent loop continues to call this
// function directly from within the same crate.
pub(crate) async fn translate_tool_request(
    tool_id: &str,
    arguments: &[u8],
    dispatcher: &dyn ToolDispatcher,
    ssb: Option<&Arc<crate::ssb::Ssb>>,
    artifact_vm: Option<&(dyn elai_knowledge::vector::VectorModule + Send + Sync)>,
    clock: &Arc<dyn ClockProvider>,
    symbol_graph_adapter: Option<
        &Arc<dyn elai_knowledge::symbol_graph_adapter::SymbolGraphAdapter>,
    >,
) -> Result<Vec<u8>, String> {
    match tool_id {
        TOOL_EDIT => {
            let wire: elai_ipc_types::EditRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool edit: decode error: {e}"))?;
            let req = EditRequest {
                file_path: PathBuf::from(wire.file_path),
                old_text: wire.old_text,
                new_text: wire.new_text,
            };
            let outcome = dispatcher
                .dispatch_edit(req)
                .await
                .map_err(|e| format!("tool edit: dispatch error: {e:?}"))?;
            let value = serde_json::json!({
                "tier": format!("{:?}", outcome.tier),
                "bytes_written": outcome.bytes_written,
                "file_path": outcome.file_path.to_string_lossy(),
            });
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool edit: serialise error: {e}"))
        }

        TOOL_VALIDATE => {
            let wire: elai_ipc_types::ValidateRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool validate: decode error: {e}"))?;
            // WARN-1: derive project_dir as the deepest containing directory
            // of file_path, not the file path itself. This matches what
            // dispatch_validate expects (a directory, not a file).
            let file_path = PathBuf::from(&wire.file_path);
            let project_dir = file_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            let req = ValidateRequest {
                project_dir,
                language: wire.language,
                changed_files: Vec::new(),
                step_id: String::new(),
                tiers: None,
            };
            let outcomes = dispatcher
                .dispatch_validate(req)
                .await
                .map_err(|e| format!("tool validate: dispatch error: {e:?}"))?;
            let arr: Vec<serde_json::Value> = outcomes
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "tier": format!("{:?}", r.tier),
                        "passed": r.passed,
                        "elapsed_ms": r.elapsed_ms,
                        "rules_fired": r.rules_fired,
                    })
                })
                .collect();
            serde_json::to_vec(&serde_json::Value::Array(arr))
                .map_err(|e| format!("tool validate: serialise error: {e}"))
        }

        TOOL_CLASSIFY => {
            let wire: elai_ipc_types::ClassifyRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool classify: decode error: {e}"))?;
            let input = FailureInput {
                raw_output: wire.error_output,
                step_id: String::new(),
                source: wire.file_path.unwrap_or_default(),
            };
            let outcome = dispatcher.dispatch_classify_failure(input).await;
            let value = match outcome {
                Some(cf) => serde_json::json!({
                    "category": format!("{:?}", cf.category),
                    "confidence": cf.confidence,
                    "source": cf.source,
                }),
                None => serde_json::Value::Null,
            };
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool classify: serialise error: {e}"))
        }

        TOOL_GIT_COMMIT => {
            let wire: elai_ipc_types::GitCommitRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool git_commit: decode error: {e}"))?;
            // WARN-5: explicit error message that names the constraint.
            let anchor_path = wire
                .paths
                .first()
                .map(PathBuf::from)
                .ok_or_else(|| {
                    "tool git_commit: git commit requires at least one path in `paths`; \
                     received empty list"
                        .to_string()
                })?;
            let files: Vec<PathBuf> = wire.paths.into_iter().map(PathBuf::from).collect();
            // WARN-3: use GitBranchPolicy::default() (BranchPerPlan) instead of Main.
            // plan_id: wire GitCommitRequest has no plan_id field; fall back to a
            // UUIDv4 string so the per-plan branch has a unique non-empty name.
            let plan_id = uuid::Uuid::new_v4().to_string(); // entropy-allow: branch-name only, no EntropySource threaded
            let req = GitCommitRequest {
                anchor_path,
                files,
                message: wire.message,
                branch_policy: GitBranchPolicy::default(),
                plan_id,
                step_id: String::new(),
            };
            let outcome = dispatcher
                .dispatch_git_commit(req)
                .await
                .map_err(|e| format!("tool git_commit: dispatch error: {e:?}"))?;
            let value = serde_json::json!({
                "sha": outcome.sha,
                "branch": outcome.branch,
                "files_count": outcome.files_count,
                "message": outcome.message,
            });
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool git_commit: serialise error: {e}"))
        }

        TOOL_FILE_READ => {
            let wire: elai_ipc_types::FileReadRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool file_read: decode error: {e}"))?;
            // WARN-2: do NOT silently drop offset/limit — return an explicit error
            // so agents know windowed reads are unsupported and must retry without them.
            if wire.offset.is_some() || wire.limit.is_some() {
                return Err(
                    "windowed file reads (offset/limit) not yet supported by \
                     dispatch_file_read_outline; pass without offset/limit for full outline"
                        .to_string(),
                );
            }
            let path = PathBuf::from(wire.file_path);
            let view = dispatcher
                .dispatch_file_read_outline(path, OUTLINE_LINE_THRESHOLD)
                .await
                .map_err(|e| format!("tool file_read: dispatch error: {e:?}"))?;
            let value = serde_json::json!({
                "outline_only": view.outline_only,
                "total_lines": view.total_lines,
                "signatures": view.signatures,
                "imports": view.imports,
                "full_content": view.full_content,
            });
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool file_read: serialise error: {e}"))
        }

        TOOL_COMPILE_CHECK => {
            let wire: elai_ipc_types::CompileCheckRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool compile_check: decode error: {e}"))?;
            // WARN-1: derive project_dir as the deepest containing directory
            // of file_path, not the file path itself.
            let file_path = PathBuf::from(&wire.file_path);
            let project_dir = file_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            let req = IncrementalRequest {
                project_dir,
                language: wire.language,
                retry_round: wire.retry_round,
                step_id: String::new(),
            };
            let outcome = dispatcher
                .dispatch_incremental_compile(req)
                .await
                .map_err(|e| format!("tool compile_check: dispatch error: {e:?}"))?;
            let value = serde_json::json!({
                "success": outcome.success,
                "attempts": outcome.attempts,
                "elapsed_ms": outcome.elapsed_ms,
            });
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool compile_check: serialise error: {e}"))
        }

        // B-29 Phase 5 — Symbol-graph traversal tools.
        // Arguments are postcard-encoded typed request structs from ipc-types.
        // Each arm: open SymbolGraphCache, call the matching traversal fn,
        // serialize the result as JSON.

        TOOL_SYMBOL_IMPACT => {
            let wire: SymbolImpactRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool symbol_impact: decode error: {e}"))?;
            let project_root = std::path::PathBuf::from(&wire.project_root);
            let project_id = project_root
                .file_name()
                .and_then(|n| n.to_str())
                .filter(|n| !n.is_empty())
                .ok_or_else(|| {
                    format!(
                        "tool symbol_impact: cannot derive project_id from project_root={:?}",
                        wire.project_root
                    )
                })?
                .to_string();
            let db_path = elai_core::paths::dkg_db_path(&project_root);
            let mut cache = SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE)
                .map_err(|e| format!("tool symbol_impact: cache open failed: {e}"))?;
            let t0 = clock.now();
            let result = compute_impact_radius(&mut cache, &wire.target, wire.depth)
                .map_err(|e| format!("tool symbol_impact: traversal error: {e}"))?;
            let latency_ms = clock.elapsed_since(t0).as_millis().try_into().unwrap_or(u32::MAX);
            let total_files_u32: u32 = result.total_files.try_into().unwrap_or(u32::MAX);
            emit_symbol_query(
                ssb,
                TOOL_SYMBOL_IMPACT,
                wire.target.clone(),
                wire.depth,
                total_files_u32,
                latency_ms,
            )
            .await;
            serde_json::to_vec(&result)
                .map_err(|e| format!("tool symbol_impact: serialise error: {e}"))
        }

        TOOL_SYMBOL_FLOW => {
            let wire: SymbolFlowRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool symbol_flow: decode error: {e}"))?;
            let project_root = std::path::PathBuf::from(&wire.project_root);
            let project_id = project_root
                .file_name()
                .and_then(|n| n.to_str())
                .filter(|n| !n.is_empty())
                .ok_or_else(|| {
                    format!(
                        "tool symbol_flow: cannot derive project_id from project_root={:?}",
                        wire.project_root
                    )
                })?
                .to_string();
            let db_path = elai_core::paths::dkg_db_path(&project_root);
            let mut cache = SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE)
                .map_err(|e| format!("tool symbol_flow: cache open failed: {e}"))?;
            let t0 = clock.now();
            let result = compute_call_flow(&mut cache, &wire.entrypoint_id, wire.depth)
                .map_err(|e| format!("tool symbol_flow: traversal error: {e}"))?;
            let latency_ms = clock.elapsed_since(t0).as_millis().try_into().unwrap_or(u32::MAX);
            // For flow queries, total_files is set to the number of unique files
            // referenced in the flow tree (root + descendants); 0 if Option<FlowNode> is None.
            let total_files_u32: u32 = match &result {
                Some(_root) => 1u32, // Flow tree starts at one entrypoint file; deeper count via traversal optional in v1
                None => 0u32,
            };
            emit_symbol_query(
                ssb,
                TOOL_SYMBOL_FLOW,
                wire.entrypoint_id.clone(),
                wire.depth,
                total_files_u32,
                latency_ms,
            )
            .await;
            serde_json::to_vec(&result)
                .map_err(|e| format!("tool symbol_flow: serialise error: {e}"))
        }

        TOOL_SYMBOL_CONTEXT => {
            let wire: SymbolContextRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool symbol_context: decode error: {e}"))?;
            let project_root = std::path::PathBuf::from(&wire.project_root);
            let project_id = project_root
                .file_name()
                .and_then(|n| n.to_str())
                .filter(|n| !n.is_empty())
                .ok_or_else(|| {
                    format!(
                        "tool symbol_context: cannot derive project_id from project_root={:?}",
                        wire.project_root
                    )
                })?
                .to_string();
            let db_path = elai_core::paths::dkg_db_path(&project_root);
            let mut cache = SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE)
                .map_err(|e| format!("tool symbol_context: cache open failed: {e}"))?;
            let t0 = clock.now();
            let contexts = compute_symbol_context(
                &mut cache,
                &wire.name,
                wire.file_hint.as_deref(),
            )
            .map_err(|e| format!("tool symbol_context: traversal error: {e}"))?;
            let latency_ms = clock.elapsed_since(t0).as_millis().try_into().unwrap_or(u32::MAX);
            let total_files_u32: u32 = contexts.len().try_into().unwrap_or(u32::MAX);
            emit_symbol_query(
                ssb,
                TOOL_SYMBOL_CONTEXT,
                wire.name.clone(),
                0,
                total_files_u32,
                latency_ms,
            )
            .await;
            serde_json::to_vec(&contexts)
                .map_err(|e| format!("tool symbol_context: serialise error: {e}"))
        }

        TOOL_SYMBOL_LIST => {
            let wire: SymbolListRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool symbol_list: decode error: {e}"))?;
            let project_root = std::path::PathBuf::from(&wire.project_root);
            let project_id = project_root
                .file_name()
                .and_then(|n| n.to_str())
                .filter(|n| !n.is_empty())
                .ok_or_else(|| {
                    format!(
                        "tool symbol_list: cannot derive project_id from project_root={:?}",
                        wire.project_root
                    )
                })?
                .to_string();
            let db_path = elai_core::paths::dkg_db_path(&project_root);
            let mut cache = SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE)
                .map_err(|e| format!("tool symbol_list: cache open failed: {e}"))?;
            // Bound the limit defensively — agents must not be able to force an
            // unbounded scan of every symbol in the cache.
            const MAX_SYMBOL_LIST_LIMIT: u32 = 500;
            let capped_limit = wire.limit.min(MAX_SYMBOL_LIST_LIMIT);
            let filter = SymbolFilter {
                file: wire.file.clone(),
                query: wire.query.clone(),
                limit: capped_limit as usize,
            };
            let t0 = clock.now();
            let symbols = list_symbols(&mut cache, filter)
                .map_err(|e| format!("tool symbol_list: traversal error: {e}"))?;
            let latency_ms = clock.elapsed_since(t0).as_millis().try_into().unwrap_or(u32::MAX);
            let total_files_u32: u32 = symbols.len().try_into().unwrap_or(u32::MAX);
            let target_label = wire
                .query
                .clone()
                .or(wire.file.clone())
                .unwrap_or_else(|| "<all>".to_string());
            emit_symbol_query(
                ssb,
                TOOL_SYMBOL_LIST,
                target_label,
                0,
                total_files_u32,
                latency_ms,
            )
            .await;
            serde_json::to_vec(&symbols)
                .map_err(|e| format!("tool symbol_list: serialise error: {e}"))
        }

        // E-40 Tranche B-2 + s-22600 Tranche B ITEM B.4: SymbolGraph DSL +
        // dataflow tools. When `symbol_graph_adapter` is Some (CLI startup
        // wired a `SymbolGraphCacheAdapter`), dispatch the query through
        // the adapter and return real results. When None, preserve the
        // pre-B.4 empty-response behaviour so harness / test paths that
        // do not wire a cache continue to see a consistent wire shape.
        // Adapter errors (cache unavailable, DSL parse fail, etc) degrade
        // to empty responses with `tracing::warn` -- the agent loop is
        // best-effort over symbol graph tools.
        TOOL_SYMBOL_DSL_QUERY => {
            let wire: SymbolDslQueryRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool symbol_dsl_query: decode error: {e}"))?;
            let t0 = clock.now();
            let resp = match symbol_graph_adapter {
                Some(adapter) => {
                    let project_root = PathBuf::from(&wire.project_root);
                    match adapter.dsl_query(&wire.dsl, &project_root, wire.depth_cap) {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::warn!(
                                tool = TOOL_SYMBOL_DSL_QUERY,
                                error = %e,
                                "symbol_dsl_query adapter error -- returning empty response"
                            );
                            elai_ipc_types::SymbolDslQueryResponse {
                                matches: Vec::new(),
                                dataflow_edges: Vec::new(),
                                total_scanned: 0,
                            }
                        }
                    }
                }
                None => elai_ipc_types::SymbolDslQueryResponse {
                    matches: Vec::new(),
                    dataflow_edges: Vec::new(),
                    total_scanned: 0,
                },
            };
            let latency_ms = clock.elapsed_since(t0).as_millis().try_into().unwrap_or(u32::MAX);
            // PRESERVE the existing emit_symbol_query beacon (additive
            // observability per s-22300 B-2 contract).
            let result_count: u32 = resp.matches.len().try_into().unwrap_or(u32::MAX);
            emit_symbol_query(
                ssb,
                TOOL_SYMBOL_DSL_QUERY,
                wire.dsl.clone(),
                wire.depth_cap as u32,
                result_count,
                latency_ms,
            )
            .await;
            // s-22600 Tranche B ITEM B.4: also emit the SymbolGraphEvent
            // row (Pattern-2 reuse, SymbolDslExecuted kind, SCHEMA 193).
            // Best-effort: failure to write is logged + swallowed so the
            // tool dispatch result is unaffected.
            if let Some(writer) = ssb {
                let ts_ms = i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX);
                if let Err(e) = crate::symbol_graph::notify_symbol_graph_event(
                    writer.as_ref(),
                    "production_agent_rpc",
                    0,
                    crate::ssb::SymbolGraphEventKind::SymbolDslExecuted,
                    None,
                    serde_json::json!({
                        "dsl": wire.dsl,
                        "depth_cap": wire.depth_cap,
                        "matches": resp.matches.len(),
                        "edges": resp.dataflow_edges.len(),
                    }),
                    ts_ms,
                )
                .await
                {
                    tracing::warn!(
                        tool = TOOL_SYMBOL_DSL_QUERY,
                        error = %e,
                        "symbol_dsl_query: SymbolGraphEvent SSB write failed"
                    );
                }
            }
            serde_json::to_vec(&resp)
                .map_err(|e| format!("tool symbol_dsl_query: serialise error: {e}"))
        }

        TOOL_SYMBOL_DATAFLOW_IMPACT => {
            let wire: SymbolDataflowImpactRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool symbol_dataflow_impact: decode error: {e}"))?;
            let t0 = clock.now();
            let resp = match symbol_graph_adapter {
                Some(adapter) => {
                    let project_root = PathBuf::from(&wire.project_root);
                    match adapter.dataflow_impact(&wire.symbol, &project_root, wire.depth_cap) {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::warn!(
                                tool = TOOL_SYMBOL_DATAFLOW_IMPACT,
                                error = %e,
                                "symbol_dataflow_impact adapter error -- returning empty response"
                            );
                            elai_ipc_types::SymbolDataflowImpactResponse {
                                impacted_symbols: Vec::new(),
                                depth_actual: 0,
                                total_edges: 0,
                            }
                        }
                    }
                }
                None => elai_ipc_types::SymbolDataflowImpactResponse {
                    impacted_symbols: Vec::new(),
                    depth_actual: 0,
                    total_edges: 0,
                },
            };
            let latency_ms = clock.elapsed_since(t0).as_millis().try_into().unwrap_or(u32::MAX);
            let result_count: u32 = resp.impacted_symbols.len().try_into().unwrap_or(u32::MAX);
            emit_symbol_query(
                ssb,
                TOOL_SYMBOL_DATAFLOW_IMPACT,
                wire.symbol.clone(),
                wire.depth_cap as u32,
                result_count,
                latency_ms,
            )
            .await;
            // s-22600 Tranche B ITEM B.4: SymbolGraphEvent row,
            // DataflowImpactComputed kind. Best-effort emit.
            if let Some(writer) = ssb {
                let ts_ms = i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX);
                if let Err(e) = crate::symbol_graph::notify_symbol_graph_event(
                    writer.as_ref(),
                    "production_agent_rpc",
                    0,
                    crate::ssb::SymbolGraphEventKind::DataflowImpactComputed,
                    None,
                    serde_json::json!({
                        "symbol": wire.symbol,
                        "depth_cap": wire.depth_cap,
                        "depth_actual": resp.depth_actual,
                        "impacted_symbols": resp.impacted_symbols.len(),
                        "total_edges": resp.total_edges,
                    }),
                    ts_ms,
                )
                .await
                {
                    tracing::warn!(
                        tool = TOOL_SYMBOL_DATAFLOW_IMPACT,
                        error = %e,
                        "symbol_dataflow_impact: SymbolGraphEvent SSB write failed"
                    );
                }
            }
            serde_json::to_vec(&resp)
                .map_err(|e| format!("tool symbol_dataflow_impact: serialise error: {e}"))
        }

        TOOL_GIT_UNDO | TOOL_GIT_BRANCH => {
            Err(format!(
                "tool {tool_id}: not implemented in ProductionAgentRpcServer (B2c)"
            ))
        }

        // B-35 Phase B — Context artifact dispatcher arms.
        // ArtifactStore is sync (std::sync::Mutex); all calls are wrapped in
        // tokio::task::spawn_blocking so the async runtime is not blocked.

        TOOL_LIST_ARTIFACTS => {
            let wire: ListArtifactsRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool list_artifacts: decode error: {e}"))?;
            let db_path = std::path::PathBuf::from(&wire.db_path);
            let chunks = tokio::task::spawn_blocking(move || {
                let store = ArtifactStore::open(&db_path)
                    .map_err(|e| format!("tool list_artifacts: store open failed: {e}"))?;
                store.list_all_chunks()
                    .map_err(|e| format!("tool list_artifacts: list failed: {e}"))
            })
            .await
            .map_err(|e| format!("tool list_artifacts: spawn_blocking error: {e}"))??;
            // Return artifact names (de-duplicated, sorted) rather than raw chunks.
            let mut names: Vec<String> = chunks
                .into_iter()
                .map(|c| c.artifact_name)
                .collect();
            names.sort_unstable();
            names.dedup();
            let value = serde_json::json!({ "artifact_names": names });
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool list_artifacts: serialise error: {e}"))
        }

        TOOL_INDEX_ARTIFACTS => {
            // C-8 Task #5: real implementation (replaces the previous deferred
            // Err stub). Singular per wire frame (IndexArtifactsRequest carries
            // one artifact_name), but the handler treats it as a one-element
            // Vec<IndexArtifactsRequest> internally per C-8 directive so future
            // bulk-frame upgrades only need to change the decode step.
            //
            // Pipeline:
            //   1. Decode IndexArtifactsRequest.
            //   2. Derive project_root from db_path (parent-of-parent walk —
            //      typical layout is <project_root>/.elai/indexes/context_*.db).
            //   3. Load ArtifactManifest::load(project_root). Filter to the
            //      named artifact; missing name => empty result, not an error.
            //   4. Open ArtifactStore + ArtifactIndexer, invoke index_artifact
            //      per filtered spec on a spawn_blocking thread.
            //   5. Emit one ArtifactsIndexed SSB row carrying the indexed +
            //      up_to_date partition + total chunk count + elapsed_ms.
            //   6. Return Ok JSON { project_id, artifact_count, chunk_count,
            //      elapsed_ms, indexed[], up_to_date[] }.
            //
            // Errors from individual specs are non-fatal (mirror
            // index_all_artifacts semantics). A genuine setup error (manifest
            // load failure, store open failure) maps to ToolError so the
            // agent sees the failure.
            let wire: IndexArtifactsRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool index_artifacts: decode error: {e}"))?;
            let pending: Vec<IndexArtifactsRequest> = vec![wire];

            let db_path = std::path::PathBuf::from(&pending[0].db_path);
            let project_root = derive_project_root_from_db_path(&db_path)
                .ok_or_else(|| {
                    format!(
                        "tool index_artifacts: cannot derive project_root from db_path \
                         {} (expected <project_root>/.elai/indexes/context_*.db layout)",
                        pending[0].db_path
                    )
                })?;

            let project_id = derive_project_id_from_root(&project_root);
            let names: Vec<String> = pending.iter().map(|r| r.artifact_name.clone()).collect();

            let start_ms = clock.now();
            // C-15: capture `now_secs` BEFORE spawning the blocking task so the
            // injected `ClockProvider` is the source of truth even inside the
            // `move` closure (we cannot move &Arc<dyn ClockProvider> across the
            // spawn_blocking boundary cheaply).
            let now_secs_for_indexer = clock.now_secs().try_into().unwrap_or(i64::MAX);
            let (indexed_names, up_to_date_names, total_chunks) =
                tokio::task::spawn_blocking({
                    let project_root = project_root.clone();
                    let db_path = db_path.clone();
                    let names = names.clone();
                    move || -> Result<(Vec<String>, Vec<String>, usize), String> {
                        let manifest = elai_knowledge::artifacts::ArtifactManifest::load(
                            &project_root,
                        )
                        .map_err(|e| {
                            format!("tool index_artifacts: manifest load failed: {e}")
                        })?;

                        let store = elai_knowledge::artifacts::store::ArtifactStore::open(
                            &db_path,
                        )
                        .map_err(|e| {
                            format!("tool index_artifacts: store open failed: {e}")
                        })?;
                        // No VectorModule on this RPC path (agents do not pass
                        // it through); embedding-less indexing is the
                        // documented graceful-degrade behaviour (chunks stored
                        // with embedding=None per artifacts/indexer.rs:18-21).
                        let indexer = elai_knowledge::artifacts::indexer::ArtifactIndexer::new(
                            &store, None,
                        );
                        // C-15: now_secs sourced from injected ClockProvider above
                        // (see `now_secs_for_indexer`). Keeps determinism under
                        // SeededClock.
                        let now_secs = now_secs_for_indexer;

                        let target_specs: Vec<&elai_knowledge::artifacts::ArtifactSpec> = manifest
                            .items
                            .iter()
                            .filter(|s| names.iter().any(|n| n == &s.name))
                            .collect();

                        let mut indexed = Vec::new();
                        let mut up_to_date = Vec::new();
                        let mut total = 0usize;
                        for spec in target_specs {
                            match indexer.index_artifact(spec, None, now_secs) {
                                Ok(state) => {
                                    total = total.saturating_add(state.chunk_count);
                                    match state.status {
                                        elai_knowledge::artifacts::ArtifactStatus::Indexed => {
                                            indexed.push(state.name);
                                        }
                                        _ => {
                                            up_to_date.push(state.name);
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        artifact = %spec.name,
                                        error = %e,
                                        "tool index_artifacts: index_artifact failed (non-fatal)"
                                    );
                                }
                            }
                        }
                        Ok((indexed, up_to_date, total))
                    }
                })
                .await
                .map_err(|e| format!("tool index_artifacts: spawn_blocking error: {e}"))??;

            let elapsed_ms = u64::try_from(clock.elapsed_since(start_ms).as_millis()).unwrap_or(u64::MAX);
            let artifact_count = u32::try_from(indexed_names.len() + up_to_date_names.len())
                .unwrap_or(u32::MAX);

            // Emit ONE ArtifactsIndexed SSB row (existing B-34 variant —
            // payload struct fields {project_id, indexed, up_to_date,
            // total_chunks, latency_ms} match exactly). Non-fatal SSB
            // write — log on failure but still return Ok to the agent.
            if let Some(ssb_arc) = ssb {
                let payload = crate::ssb::SsbPayload::ArtifactsIndexed(
                    crate::ssb::ArtifactsIndexedPayload {
                        project_id: project_id.clone(),
                        indexed: indexed_names.clone(),
                        up_to_date: up_to_date_names.clone(),
                        total_chunks,
                        latency_ms: elapsed_ms,
                    },
                );
                if let Err(e) = ssb_arc
                    .write_payload("agent_rpc", 0, payload, Vec::new())
                    .await
                {
                    tracing::warn!(
                        error = %e,
                        "tool index_artifacts: ArtifactsIndexed SSB write failed (non-fatal)"
                    );
                }
            }

            let value = serde_json::json!({
                "project_id": project_id,
                "artifact_count": artifact_count,
                "chunk_count": total_chunks,
                "elapsed_ms": elapsed_ms,
                "indexed": indexed_names,
                "up_to_date": up_to_date_names,
            });
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool index_artifacts: serialise error: {e}"))
        }

        TOOL_SEARCH_ARTIFACTS => {
            let wire: SearchArtifactsRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool search_artifacts: decode error: {e}"))?;
            let limit = wire.limit
                .unwrap_or(elai_ipc_types::DEFAULT_ARTIFACT_SEARCH_LIMIT);
            let db_path = std::path::PathBuf::from(&wire.db_path);
            let query = wire.query.clone();
            let artifact_name = wire.artifact_name.clone();

            if let Some(vm) = artifact_vm {
                // Phase: embed in async context (VectorModule is not Send, cannot move
                // into spawn_blocking).  The embedding call is CPU-bound but short
                // relative to the store I/O that follows.
                let query_embedding = vm
                    .embed_text(&query)
                    .map_err(|e| format!("tool search_artifacts: embed_text failed: {e}"))?;

                // Offload SQLite I/O + cosine scan to a blocking thread.
                // query_embedding is Vec<f32> which is Send.
                let results = tokio::task::spawn_blocking(move || {
                    let store = ArtifactStore::open(&db_path)
                        .map_err(|e| format!("tool search_artifacts: store open failed: {e}"))?;
                    artifact_search_with_precomputed_embedding(
                        &store,
                        &query_embedding,
                        artifact_name.as_deref(),
                        limit,
                    )
                    .map_err(|e| format!("tool search_artifacts: search failed: {e}"))
                })
                .await
                .map_err(|e| format!("tool search_artifacts: spawn_blocking error: {e}"))??;

                // Strip embedding vectors before JSON serialisation: raw f32 arrays
                // bloat the IPC frame (~150 KB for 10 results at 1536-dim) and callers
                // do not need the raw vectors — scores and content are sufficient.
                let response_items: Vec<serde_json::Value> = results
                    .into_iter()
                    .map(|r| {
                        serde_json::json!({
                            "artifact_name": r.chunk.artifact_name,
                            "relative_path": r.chunk.relative_path,
                            "content": r.chunk.content,
                            "start_line": r.chunk.start_line,
                            "end_line": r.chunk.end_line,
                            "score": r.score,
                        })
                    })
                    .collect();
                let value = serde_json::json!({ "results": response_items });
                serde_json::to_vec(&value)
                    .map_err(|e| format!("tool search_artifacts: serialise error: {e}"))
            } else {
                // No VectorModule available — return empty results (graceful degrade).
                tracing::warn!(
                    "tool search_artifacts: no VectorModule injected — \
                     returning empty results (wire with_artifact_vm to enable search)"
                );
                let value = serde_json::json!({ "results": [] });
                serde_json::to_vec(&value)
                    .map_err(|e| format!("tool search_artifacts: serialise error: {e}"))
            }
        }

        TOOL_REMOVE_ARTIFACTS => {
            let wire: RemoveArtifactsRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool remove_artifacts: decode error: {e}"))?;
            let db_path = std::path::PathBuf::from(&wire.db_path);
            let artifact_name = wire.artifact_name.clone();
            let dropped = tokio::task::spawn_blocking(move || {
                let store = ArtifactStore::open(&db_path)
                    .map_err(|e| format!("tool remove_artifacts: store open failed: {e}"))?;
                store.drop_chunks_for_artifact(&artifact_name)
                    .map_err(|e| format!("tool remove_artifacts: drop failed: {e}"))
            })
            .await
            .map_err(|e| format!("tool remove_artifacts: spawn_blocking error: {e}"))??;
            let value = serde_json::json!({
                "artifact_name": wire.artifact_name,
                "chunks_removed": dropped,
            });
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool remove_artifacts: serialise error: {e}"))
        }

        // B-34 — Entry-point detection tool.
        // Loads reverse_file_index once (lazy), then selectively loads payloads
        // for orphan-candidate files + conventional entry-point filenames only.
        // This preserves the B-29 lazy-loading invariant.
        TOOL_ENTRY_POINTS => {
            let wire: EntryPointsRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool entry_points: decode error: {e}"))?;
            let project_root = std::path::PathBuf::from(&wire.project_root);
            // Use canonical blake3-based project_id (WARN-2: replaces basename heuristic).
            let project_id = elai_core::project_id_from_path(&project_root);
            let db_path = elai_core::paths::dkg_db_path(&project_root);

            // All SymbolGraphCache operations are synchronous (no async I/O).
            let t0 = clock.now();
            let result: Result<Vec<DetectedEntryPoint>, String> = (|| {
                let mut cache =
                    SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE)
                        .map_err(|e| format!("tool entry_points: cache open failed: {e}"))?;

                // Step 1: load reverse index (lazy, ~5ms first call).
                let reverse_index = cache
                    .ensure_reverse_index()
                    .map_err(|e| format!("tool entry_points: reverse_index load failed: {e}"))?
                    .clone();

                // Step 2: collect all known files from the reverse index
                // (keys = callee files; values = caller files).
                // Union gives every indexed file without loading name shards.
                let mut all_files: std::collections::BTreeSet<String> =
                    reverse_index.keys().cloned().collect();
                for callers in reverse_index.values() {
                    all_files.extend(callers.iter().cloned());
                }

                // Step 3: coarse extension filter — exclude non-code files that can
                // never contain entry points (BLOCK-5: the old orphan+filename filter
                // broke Heuristic 3 by excluding imported route files; we now load
                // payloads for all code-language files; non-code files are excluded for
                // performance. See workflow/reasoning/B-34-decisions.md DECISION-B34-9).
                const CODE_EXTENSIONS: &[&str] = &[
                    "ts", "tsx", "js", "jsx", "py", "rb", "go", "rs", "java", "kt", "php",
                    "c", "cpp", "cs", "swift", "scala", "ex", "exs",
                ];
                let candidate_files: Vec<String> = all_files
                    .into_iter()
                    .filter(|f| {
                        let ext = std::path::Path::new(f)
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("");
                        CODE_EXTENSIONS.contains(&ext)
                    })
                    .collect();

                // Step 4: selectively load payloads for candidates only.
                let mut payloads = Vec::with_capacity(candidate_files.len());
                for file in &candidate_files {
                    match cache.get_or_load_payload(file) {
                        Ok(Some(arc)) => payloads.push((*arc).clone()),
                        Ok(None) => {} // file not in DB — skip
                        Err(e) => {
                            tracing::warn!(
                                file = %file,
                                error = %e,
                                "tool entry_points: payload load failed — skipping"
                            );
                        }
                    }
                }

                // Step 5: run detection (pure fn, no I/O).
                let mut entries = detect_entry_points(&reverse_index, &payloads);

                // Step 6: filter tests if caller requested.
                if !wire.include_tests {
                    entries.retain(|e| !matches!(e.reason, EntryPointReason::Test));
                }

                Ok(entries)
            })();

            let entries = result?;
            let latency_ms: u32 = clock.elapsed_since(t0).as_millis().try_into().unwrap_or(u32::MAX);
            let summary = EntryPointSummary::from_entries(&entries);

            // Convert to wire DTOs.
            let dto_entries: Vec<serde_json::Value> = entries
                .iter()
                .map(|e| {
                    let (reason_str, reason_detail) = match &e.reason {
                        EntryPointReason::Orphan => ("orphan".to_string(), None),
                        EntryPointReason::WellKnownName(n) => {
                            ("well_known".to_string(), Some(n.clone()))
                        }
                        EntryPointReason::FrameworkRoute(r) => {
                            ("framework".to_string(), Some(r.clone()))
                        }
                        EntryPointReason::Test => ("test".to_string(), None),
                    };
                    serde_json::json!({
                        "id": e.id,
                        "name": e.name,
                        "file": e.file,
                        "line": e.line,
                        "reason": reason_str,
                        "reason_detail": reason_detail,
                    })
                })
                .collect();

            let value = serde_json::json!({
                "entries": dto_entries,
                "summary": {
                    "total": summary.total,
                    "by_reason": summary.by_reason,
                    "latency_ms": latency_ms,
                },
            });

            // BLOCK-2: emit EntryPointsDetected SSB row (best-effort, non-fatal).
            if let Some(ssb) = ssb {
                use crate::ssb::{EntryPointsDetectedPayload, SsbPayload};
                let ep_payload = SsbPayload::EntryPointsDetected(EntryPointsDetectedPayload {
                    project_id: project_id.clone(),
                    count: summary.total,
                    by_reason: summary.by_reason.clone(),
                });
                if let Err(e) = ssb
                    .write_payload(SSB_AGENT_SYMBOL_GRAPH, 0, ep_payload, Vec::new())
                    .await
                {
                    tracing::warn!(
                        error = %e,
                        "entry_points SSB write failed (continuing tool result)"
                    );
                }
            }

            serde_json::to_vec(&value)
                .map_err(|e| format!("tool entry_points: serialise error: {e}"))
        }

        // C-24 (DRIFT-5, DECISION-4): Mermaid graph-export tool. Mirrors the
        // Tauri command `export_graph_mermaid` (dashboard-v2 commands.rs:1805)
        // and the shared `export_graph_mermaid_impl` (C-18 D2 extraction).
        // Only `file` and `symbol` modes are reachable here: `decomposition`
        // mode requires in-memory `DecompositionTreeSnapshot` from the
        // dashboard `AppState` (no orchestrator dep). Callers that need
        // decomposition mode must use the Tauri command.
        TOOL_EXPORT_GRAPH_MERMAID => {
            let wire: ExportGraphMermaidRequest = postcard::from_bytes(arguments)
                .map_err(|e| format!("tool export_graph_mermaid: decode error: {e}"))?;
            let project_root = std::path::PathBuf::from(&wire.project_root);
            let project_id = project_root
                .file_name()
                .and_then(|n| n.to_str())
                .filter(|n| !n.is_empty())
                .ok_or_else(|| {
                    format!(
                        "tool export_graph_mermaid: cannot derive project_id from project_root={:?}",
                        wire.project_root
                    )
                })?
                .to_string();
            // Build VizData via the knowledge crate's shared view builders
                // (extracted from dashboard-v2/src-tauri/src/commands.rs in C-24).
            let data = match wire.mode.as_str() {
                "file" => elai_knowledge::visualization::views::build_file_view(
                    &project_root,
                    &project_id,
                )?,
                "symbol" => elai_knowledge::visualization::views::build_symbol_view(
                    &project_root,
                    &project_id,
                )?,
                "decomposition" => {
                    return Err(
                        "tool export_graph_mermaid: `decomposition` mode requires \
                         in-memory DecompositionTreeSnapshot from the dashboard \
                         AppState; use the Tauri export_graph_mermaid command \
                         instead of the IPC tool surface for that mode"
                            .to_string(),
                    );
                }
                other => {
                    return Err(format!(
                        "tool export_graph_mermaid: unknown mode `{other}`; \
                         expected file/symbol/decomposition"
                    ));
                }
            };
            // Match the B-33 default cap when the caller passes None.
            const B33_DEFAULT_MERMAID_MAX_NODES: usize = 200;
            let cap = wire
                .max_nodes
                .map(|n| n as usize)
                .unwrap_or(B33_DEFAULT_MERMAID_MAX_NODES);
            let mermaid = elai_knowledge::visualization::mermaid::generate_mermaid(&data, cap);
            // Return as JSON `{"mermaid": "..."}` so the agent can parse a
            // single field without dealing with a raw string envelope.
            let value = serde_json::json!({ "mermaid": mermaid });
            serde_json::to_vec(&value)
                .map_err(|e| format!("tool export_graph_mermaid: serialise error: {e}"))
        }

        unknown => Err(format!("unknown tool_id: {unknown}")),
    }
}

// ---------------------------------------------------------------------------
// C-18 D1: public test-helper alias for `translate_tool_request`.
//
// Production callers stay inside the orchestrator crate and use the
// `pub(crate)` symbol directly. Contract/integration tests in
// `crates/orchestrator/tests/` link against the orchestrator as an external
// crate (rustc treats `tests/*.rs` as a separate compilation unit), so they
// cannot reach `pub(crate)` items.
//
// Gating on `any(test, feature = "test-support")`:
//   * `test` is set inside `lib.rs` integration tests (in-crate `#[cfg(test)]`).
//   * `test-support` is the orchestrator feature already used by
//     `tests/production_agent_rpc_contract.rs` (Cargo.toml dev-dep
//     `elai-orchestrator = { path = ".", features = ["test-support"] }`),
//     so this alias surfaces automatically for the C-18 dispatch test.
//
// The alias is a thin pass-through — no behaviour duplication, no extra
// allocation. If the underlying signature changes, the build breaks here
// first, which is the desired locking property.
#[cfg(any(test, feature = "test-support"))]
pub async fn translate_tool_request_for_test(
    tool_id: &str,
    arguments: &[u8],
    dispatcher: &dyn ToolDispatcher,
    ssb: Option<&Arc<crate::ssb::Ssb>>,
    artifact_vm: Option<&(dyn elai_knowledge::vector::VectorModule + Send + Sync)>,
    clock: &Arc<dyn ClockProvider>,
) -> Result<Vec<u8>, String> {
    translate_tool_request(tool_id, arguments, dispatcher, ssb, artifact_vm, clock, None).await
}

