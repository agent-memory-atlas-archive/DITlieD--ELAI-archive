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

use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use elai_core::tool_dispatcher::{
    EditRequest, FailureInput, GitBranchPolicy, GitCommitRequest, IncrementalRequest,
    ToolDispatcher, ValidateRequest,
};
use elai_ipc_types::{
    EntryPointsRequest, ListArtifactsRequest, RemoveArtifactsRequest, SearchArtifactsRequest,
    SymbolContextRequest, SymbolFlowRequest, SymbolImpactRequest, SymbolListRequest,
    TOOL_CLASSIFY, TOOL_COMPILE_CHECK, TOOL_EDIT, TOOL_ENTRY_POINTS, TOOL_FILE_READ,
    TOOL_GIT_BRANCH, TOOL_GIT_COMMIT, TOOL_GIT_UNDO, TOOL_INDEX_ARTIFACTS, TOOL_LIST_ARTIFACTS,
    TOOL_REMOVE_ARTIFACTS, TOOL_SEARCH_ARTIFACTS, TOOL_SYMBOL_CONTEXT, TOOL_SYMBOL_FLOW,
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
use elai_providers::{AnthropicClient, InputMessage, MessageRequest, OutputContentBlock};

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
        }
    }

    /// Builder-style setter for the SSB writer used by symbol-graph tool arms.
    /// Returns `self` so call sites can chain.
    pub fn with_ssb(mut self, ssb: Arc<crate::ssb::Ssb>) -> Self {
        self.ssb = Some(ssb);
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
            let request = MessageRequest {
                model: self.model.clone(),
                max_tokens: effective_max_tokens,
                messages: vec![InputMessage::user_text(prompt)],
                system: None,
                tools: None,
                tool_choice: None,
                stream: false,
                seed: None,
            };

            match self.client.send_message(&request).await {
                Ok(response) => {
                    let text = response.content.iter().find_map(|block| {
                        if let OutputContentBlock::Text { text } = block {
                            Some(text.clone())
                        } else {
                            None
                        }
                    });
                    match text {
                        Some(t) => InferOutcome::Response(t),
                        None => InferOutcome::Failed(
                            "provider returned no text content block".to_string(),
                        ),
                    }
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
            match translate_tool_request(
                &tool_id,
                &arguments,
                self.dispatcher.as_ref(),
                self.ssb.as_ref(),
                self.artifact_vm.as_ref().map(Arc::as_ref),
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

async fn translate_tool_request(
    tool_id: &str,
    arguments: &[u8],
    dispatcher: &dyn ToolDispatcher,
    ssb: Option<&Arc<crate::ssb::Ssb>>,
    artifact_vm: Option<&(dyn elai_knowledge::vector::VectorModule + Send + Sync)>,
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
            let plan_id = uuid::Uuid::new_v4().to_string();
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
            let t0 = std::time::Instant::now();
            let result = compute_impact_radius(&mut cache, &wire.target, wire.depth)
                .map_err(|e| format!("tool symbol_impact: traversal error: {e}"))?;
            let latency_ms = t0.elapsed().as_millis().try_into().unwrap_or(u32::MAX);
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
            let t0 = std::time::Instant::now();
            let result = compute_call_flow(&mut cache, &wire.entrypoint_id, wire.depth)
                .map_err(|e| format!("tool symbol_flow: traversal error: {e}"))?;
            let latency_ms = t0.elapsed().as_millis().try_into().unwrap_or(u32::MAX);
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
            let t0 = std::time::Instant::now();
            let contexts = compute_symbol_context(
                &mut cache,
                &wire.name,
                wire.file_hint.as_deref(),
            )
            .map_err(|e| format!("tool symbol_context: traversal error: {e}"))?;
            let latency_ms = t0.elapsed().as_millis().try_into().unwrap_or(u32::MAX);
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
            let t0 = std::time::Instant::now();
            let symbols = list_symbols(&mut cache, filter)
                .map_err(|e| format!("tool symbol_list: traversal error: {e}"))?;
            let latency_ms = t0.elapsed().as_millis().try_into().unwrap_or(u32::MAX);
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
            // STUB: full artifact re-indexing is deferred to Phase C, which will wire
            // ArtifactIndexer + VectorModule via a service-locator pattern.
            // Returning Err prevents agents from calling this tool and silently
            // receiving an empty artifact store (previous "cleared" status was misleading).
            // Use the CLI `elai index-artifacts` command for on-demand indexing.
            return Err(
                "tool index_artifacts: full indexing deferred to Phase C; \
                 use the CLI `elai index-artifacts` command for now"
                    .to_string(),
            );
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
            let t0 = std::time::Instant::now();
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
            let latency_ms: u32 = t0.elapsed().as_millis().try_into().unwrap_or(u32::MAX);
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

        unknown => Err(format!("unknown tool_id: {unknown}")),
    }
}

