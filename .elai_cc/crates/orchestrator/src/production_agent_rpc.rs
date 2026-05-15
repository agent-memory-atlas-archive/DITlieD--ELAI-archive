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
    TOOL_CLASSIFY, TOOL_COMPILE_CHECK, TOOL_EDIT, TOOL_FILE_READ, TOOL_GIT_BRANCH,
    TOOL_GIT_COMMIT, TOOL_GIT_UNDO, TOOL_VALIDATE,
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
        }
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
            match translate_tool_request(&tool_id, &arguments, self.dispatcher.as_ref()).await {
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
async fn translate_tool_request(
    tool_id: &str,
    arguments: &[u8],
    dispatcher: &dyn ToolDispatcher,
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

        TOOL_GIT_UNDO | TOOL_GIT_BRANCH => {
            Err(format!(
                "tool {tool_id}: not implemented in ProductionAgentRpcServer (B2c)"
            ))
        }

        unknown => Err(format!("unknown tool_id: {unknown}")),
    }
}

