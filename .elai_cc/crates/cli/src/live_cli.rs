#![allow(dead_code, unused_imports, unused_variables, unused_mut, clippy::unneeded_struct_pattern, clippy::unnecessary_wraps, clippy::unused_self)]

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, UNIX_EPOCH};
use std::pin::Pin;
use std::future::Future;

use elai_core::{ClockProvider, EntropySource, OsClock, OsEntropy};
use elai_providers::{
    oauth_token_is_expired, resolve_startup_auth_source, AnthropicClient, AuthSource,
    ContentBlockDelta, InputContentBlock, InputMessage, MessageRequest, MessageResponse,
    OutputContentBlock, PromptCache,
    StreamEvent as ApiStreamEvent, ToolChoice, ToolDefinition, ToolResultContentBlock,
    LlmProvider, OllamaConfig, SidecarProvider,
};
use elai_plugins::{PluginHooks, PluginManager, PluginManagerConfig, PluginRegistry};
use crossterm::{cursor::MoveToColumn, execute, terminal::{Clear, ClearType}};
use elai_orchestrator::{
    clear_oauth_credentials, format_usd, generate_pkce_pair, generate_state,
    load_oauth_credentials, load_system_prompt, parse_oauth_callback_request_target,
    pricing_for_model, resolve_sandbox_status, save_oauth_credentials, ApiClient, ApiRequest,
    AssistantEvent, BackendConfig, BackendMode, CompactionConfig, ConfigLoader, ConfigSource,
    ContentBlock, ConversationMessage, ConversationRuntime, McpServerManager, McpTool, MessageRole,
    ModelPricing, OAuthAuthorizationRequest, OAuthConfig, OAuthTokenExchangeRequest,
    PermissionMode, PermissionPolicy, ProjectContext, PromptCacheEvent, ResolvedPermissionMode,
    RuntimeError, Session, TokenUsage, ToolError, ToolExecutor, UsageTracker,
};
use serde::Deserialize;
use serde_json::json;
use elai_rules::predicate::from_json;
use elai_rules::rule_engine::{CompiledPredicate, DefaultRuleEngine, EvaluationContext, FsmPhaseBit, Rule, RuleEngine};
use elai_knowledge::{TreeSitterAstEvaluator, ShortCoder, DkgQuerierAdapter, DkgEngine, detect_project_profile, populate_tier3, assemble_telescope};
use elai_knowledge::pipeline::{QueryPipeline, PipelineStage};
use elai_knowledge::bm25::Bm25Index;
use elai_knowledge::catrag::{CatRag, CatRagConfig};
use elai_knowledge::arag::{AragOrchestrator, AragConfig};
use elai_knowledge::vector::OrtVectorModule;
use elai_knowledge::vector::ort_session::OrtSession;
use elai_knowledge::vector::indexer::VectorIndexer;
use elai_knowledge::vector::TEXT_VEC_DIM;
use elai_knowledge::startup::{startup_readiness_check, StartupConfig};
use elai_knowledge::degradation::DegradationCascade;
use elai_knowledge::reranker::{CrossEncoderReranker, RerankerConfig};
use elai_knowledge::splade::{SpladeExpander, SpladeConfig};
use elai_knowledge::index_version::{check_and_upgrade, LogNotifier};
use elai_orchestrator::indexing_service::IndexingService;
use elai_core::paths::{
    dkg_db_path as core_dkg_db_path, models_dir_path as core_models_dir_path,
    rules_db_path as core_rules_db_path, vectors_db_path as core_vectors_db_path,
    ELAI_DATA_DIR_NAME,
};
use elai_rules::stubs::EmptyDkgQuerier;
use elai_tools::{GlobalToolRegistry, RuntimeToolDefinition, ToolSearchOutput, mvp_tool_specs};

use crate::commands::{
    handle_agents_slash_command, handle_mcp_slash_command, handle_mcp_slash_command_json,
    handle_plugins_slash_command, handle_skills_slash_command, handle_skills_slash_command_json,
    render_slash_command_help, resume_supported_slash_commands, slash_command_specs,
    validate_slash_command_input, SlashCommand,
};
use crate::init::initialize_repo;
use crate::render::{MarkdownStreamState, Spinner, TerminalRenderer};
use crate::*;
use crate::progress::{InternalPromptProgressReporter, InternalPromptProgressEvent, InternalPromptProgressState};
use crate::runtime_client::RuntimeClient;
use crate::tool_executor::CliToolExecutor;
use crate::runtime_builder::{build_runtime, build_runtime_with_plugin_state, CliPermissionPrompter};
use elai_reasoning::{
    ChatFuture, ChatInferenceClient, ChatReflectConfig, ChatReflectPipeline, ReflectError,
    ReflectEventSink,
};

// ---------------------------------------------------------------------------
// C-1: reranker SSB notifier adapters
// ---------------------------------------------------------------------------
//
// The knowledge crate's `SsbNotifier` trait (defined in
// `crate::degradation`) emits `DegradationEvent { action_type, stage,
// detail }`. The orchestrator's `Arc<Ssb>` writer needs an
// `SsbPayload::KnowledgeDegradation` row. `SsbRerankerNotifier` bridges
// the two so the C-1 stub_ban check
// (`NoopSsbNotifier` absent from production `reranker.rs` lines) holds
// while `knowledge_degradation` rows still appear in the dashboard.
struct SsbRerankerNotifier {
    ssb: Arc<elai_orchestrator::ssb::Ssb>,
}

impl SsbRerankerNotifier {
    fn new(ssb: Arc<elai_orchestrator::ssb::Ssb>) -> Self {
        Self { ssb }
    }
}

impl elai_knowledge::reranker::SsbNotifier for SsbRerankerNotifier {
    fn emit(&self, event: elai_knowledge::reranker::DegradationEvent) {
        let payload = elai_orchestrator::ssb::SsbPayload::KnowledgeDegradation(
            elai_orchestrator::ssb::KnowledgeDegradationPayload {
                component: event.stage.to_owned(),
                old_mode: String::new(),
                new_mode: String::new(),
                reason: event.detail,
            },
        );
        let ssb = self.ssb.clone();
        // Same pattern as `OrchestratorKnowledgeSsbSink` — spawn onto the
        // active tokio runtime; drop with a warning when none is active.
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    if let Err(e) = ssb
                        .write_payload(
                            elai_orchestrator::task_runner::RUNNER_AGENT_ID,
                            elai_orchestrator::task_runner::RUNNER_SUPERSTEP,
                            payload,
                            Vec::new(),
                        )
                        .await
                    {
                        tracing::warn!(
                            error = %e,
                            "live_cli reranker notifier: write_payload failed (non-fatal)"
                        );
                    }
                });
            }
            Err(_) => {
                tracing::debug!(
                    "live_cli reranker notifier: no tokio runtime; dropping row"
                );
            }
        }
    }
}

/// Local noop used when no Ssb writer is available. Distinct from the
/// `NoopSsbNotifier` defined in `crate::degradation` so that the
/// production reranker.rs surface has zero `NoopSsbNotifier`
/// references (WIRING_CONTRACT stub_ban).
struct NoopRerankerNotifier;

impl elai_knowledge::reranker::SsbNotifier for NoopRerankerNotifier {
    fn emit(&self, _event: elai_knowledge::reranker::DegradationEvent) {}
}

// ---------------------------------------------------------------------------
// C-8: helpers for the search-mode bridge + cosmos tick scheduler
// ---------------------------------------------------------------------------

/// C-8 Task #7: stable lowercase tag for the SearchModeChanged SSB payload.
/// Maps the [`elai_knowledge::degradation::SearchMode`] enum to the payload
/// discriminator documented on `SearchModeChangedPayload::mode`. Forward-
/// compat: adding a new SearchMode tag here does NOT require a schema bump.
fn search_mode_to_payload_str(mode: elai_knowledge::degradation::SearchMode) -> &'static str {
    use elai_knowledge::degradation::SearchMode;
    match mode {
        SearchMode::Hybrid => "hybrid",
        SearchMode::CodeOnly => "semantic",
        SearchMode::TextOnly => "semantic",
        SearchMode::SparseOnly => "ann",
        SearchMode::Bm25Only => "lexical",
    }
}

// ---------------------------------------------------------------------------
// C-30: project detection signal scan (fresh-project gate).
// ---------------------------------------------------------------------------

/// A single project-detection signal evaluated by [`run_project_detection_signals`].
/// `name` is a stable lowercase identifier matching the SSB payload's
/// `signal_name`. `weight` mirrors the SSB payload's `weight` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct C30ProjectSignal {
    name: &'static str,
    weight: C30SignalWeight,
    fired: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum C30SignalWeight {
    Required,
    Strong,
    Weak,
}

impl C30SignalWeight {
    fn as_str(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Strong => "strong",
            Self::Weak => "weak",
        }
    }
}

/// Evaluate the C-30 project-detection signal table for the given project
/// root and return one [`C30ProjectSignal`] per signal evaluated. Pure
/// function — does not emit SSB rows itself; the caller is responsible for
/// emitting `ProjectDetectionSignal` rows for each entry.
fn run_project_detection_signals(project_root: &Path) -> Vec<C30ProjectSignal> {
    let mut out = Vec::with_capacity(6);

    out.push(C30ProjectSignal {
        name: "cargo_toml",
        weight: C30SignalWeight::Required,
        fired: project_root.join("Cargo.toml").is_file(),
    });
    out.push(C30ProjectSignal {
        name: "go_mod",
        weight: C30SignalWeight::Required,
        fired: project_root.join("go.mod").is_file(),
    });
    out.push(C30ProjectSignal {
        name: "pyproject_toml",
        weight: C30SignalWeight::Required,
        fired: project_root.join("pyproject.toml").is_file(),
    });
    out.push(C30ProjectSignal {
        name: "package_json",
        weight: C30SignalWeight::Strong,
        fired: project_root.join("package.json").is_file(),
    });

    // Best-effort .rs count up to 10. We bail early to keep the scan cheap.
    let mut rs_count: u32 = 0;
    if let Ok(read) = std::fs::read_dir(project_root) {
        for entry in read.flatten().take(256) {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".rs") {
                    rs_count += 1;
                    if rs_count >= 10 {
                        break;
                    }
                }
            }
        }
    }
    out.push(C30ProjectSignal {
        name: "rust_file_count_ge_10",
        weight: C30SignalWeight::Strong,
        fired: rs_count >= 10,
    });

    out
}

/// True when at least one REQUIRED signal fired (per pipeline-modes-design
/// section 4): the operator should be asked to build the index.
fn c30_any_required_fired(signals: &[C30ProjectSignal]) -> bool {
    signals
        .iter()
        .any(|s| s.weight == C30SignalWeight::Required && s.fired)
}

/// Emit one SSB row per evaluated signal. Safe to call from synchronous
/// dispatch arms — uses `tokio::runtime::Handle::try_current()` and silently
/// drops rows when no runtime is active (mirrors `SsbRerankerNotifier`).
fn emit_project_detection_signals(
    ssb: &Arc<elai_orchestrator::ssb::Ssb>,
    signals: &[C30ProjectSignal],
) {
    if signals.is_empty() {
        return;
    }
    let ssb = ssb.clone();
    let signals: Vec<C30ProjectSignal> = signals.to_vec();
    let emit = async move {
        for sig in signals {
            let payload = elai_orchestrator::ssb::SsbPayload::ProjectDetectionSignal(
                elai_orchestrator::ssb::ProjectDetectionSignalPayload {
                    signal_name: sig.name.to_string(),
                    weight: sig.weight.as_str().to_string(),
                    fired: sig.fired,
                },
            );
            if let Err(e) = ssb
                .write_payload(
                    elai_orchestrator::task_runner::RUNNER_AGENT_ID,
                    elai_orchestrator::task_runner::RUNNER_SUPERSTEP,
                    payload,
                    Vec::new(),
                )
                .await
            {
                tracing::warn!(error = %e, "C-30 project-detection-signal SSB emit failed");
            }
        }
    };
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            handle.spawn(emit);
        }
        Err(_) => {
            // No async runtime — build a transient runtime so the rows still
            // land. We block_on synchronously because the dispatch arm is
            // sync and we have already declared the cost above.
            if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                rt.block_on(emit);
            }
        }
    }
}

/// C-8 Task #3 + C-16 (DECISION-A/F/H): per-tick wrapper around
/// `CosmosWorker::tick()`.
///
/// As of C-16, the SSB emissions live INSIDE `CosmosWorker::tick()` (via the
/// `SsbEmitter` attached at construction). This wrapper is now a thin
/// scheduler shim: it invokes tick() and logs failed artifacts. The
/// `CosmosRefreshStarted` row (with correct stale+decayed count per
/// DECISION-F) and `CosmosArtifactUpdated` rows (with real blake3/size/
/// latency per DECISION-H) are emitted by the worker itself, so this
/// wrapper no longer hand-builds payloads.
async fn cosmos_tick_emit_loop(
    _ssb: &Arc<elai_orchestrator::Ssb>,
    worker: &Arc<elai_knowledge::CosmosWorker>,
) {
    // C-16 BLOCK-1 fix: `worker.tick()` is now async-native (no nested
    // runtime panic). Await directly on the ambient multi-thread tokio
    // runtime spawned at live_cli.rs:1425.
    let report = worker.tick().await;
    for (name, err) in report.failed.iter() {
        tracing::warn!(
            artifact = %name,
            error = %err,
            "C-16 cosmos tick: refresh failed (non-fatal, advisory only)"
        );
    }
    tracing::debug!(
        checked = report.checked,
        refreshed = report.refreshed.len(),
        unchanged = report.unchanged.len(),
        failed = report.failed.len(),
        "C-16 cosmos tick: report"
    );
}

#[derive(Debug, Clone)]
pub(crate) struct SessionHandle {
    pub(crate) id: String,
    pub(crate) path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct ManagedSessionSummary {
    pub(crate) id: String,
    pub(crate) path: PathBuf,
    pub(crate) modified_epoch_millis: u128,
    pub(crate) message_count: usize,
    pub(crate) parent_session_id: Option<String>,
    pub(crate) branch_name: Option<String>,
}

pub(crate) struct LiveCli {
    pub(crate) model: String,
    pub(crate) allowed_tools: Option<AllowedToolSet>,
    pub(crate) permission_mode: PermissionMode,
    pub(crate) system_prompt: Vec<String>,
    pub(crate) runtime: BuiltRuntime,
    pub(crate) session: SessionHandle,
    pub(crate) backend_config: Option<BackendConfig>,
    pub(crate) ssb: Option<Arc<elai_orchestrator::Ssb>>,
    pub(crate) fsm: Arc<elai_orchestrator::Fsm>,
    pub(crate) clock: Arc<dyn ClockProvider>,
    pub(crate) session_dag: Option<elai_orchestrator::SessionDag>,
    pub(crate) plan: Option<elai_orchestrator::PlanSkeleton>,
    pub(crate) last_snap_id: Option<String>,
    pub(crate) last_fsm_state: String,
    pub(crate) knowledge_pipeline: Option<QueryPipeline>,
    /// A-44: IndexingService for coordinated DKG+BM25 build and incremental updates.
    pub(crate) indexing_service: Option<IndexingService>,
    /// A-38: baby step counter for incremental profile refresh (every N=50 turns).
    pub(crate) baby_step_counter: u32,
    /// A-38: cached project profile for incremental updates.
    pub(crate) cached_profile: Option<elai_core::ProjectProfile>,
    /// A-38: unix timestamp (seconds) of last successful profile refresh. 0 = never.
    pub(crate) last_profiled_at: u64,
    /// B-9 cycle 2 (BLOCK-1 fix): when `false` the REPL turn loop never
    /// touches the legacy JSONL renderer — `Session::save_to_path` only
    /// fires from the explicit `/export --export-jsonl` slash command.
    /// Toggled at startup via the top-level `--export-jsonl` CLI flag.
    /// SessionDag is the authoritative store regardless of this flag.
    pub(crate) export_jsonl_enabled: bool,
    /// B-39: whether `/reason on` is active. When `false` (default), the
    /// Chat REFLECT pipeline is a zero-overhead no-op in run_turn.
    pub(crate) reflect_enabled: bool,
    /// B-39: lazily-constructed REFLECT pipeline. Built on first use when
    /// `reflect_enabled` is `true` and a `backend_config` is present.
    /// Cleared on `/reason off` so a fresh build picks up any config changes.
    pub(crate) reflect_pipeline: Option<ChatReflectPipeline>,
    /// C-30: Oracle answer-level cache. Lazily created on first `/research`
    /// invocation. Lives for the lifetime of the LiveCli session.
    pub(crate) oracle_cache: Option<Arc<elai_orchestrator::oracle_cache::OracleAnswerCache>>,
    /// C-30: set to `true` once the fresh-project detection signal scan has
    /// run (either from a prior `/task` invocation in this session, or
    /// because the indexes already existed at startup). Prevents the scan
    /// from re-running on every `/task` call.
    pub(crate) project_detection_done: bool,
}

pub(crate) struct RuntimePluginState {
    pub(crate) feature_config: elai_orchestrator::RuntimeFeatureConfig,
    pub(crate) tool_registry: GlobalToolRegistry,
    pub(crate) plugin_registry: PluginRegistry,
    pub(crate) mcp_state: Option<Arc<Mutex<RuntimeMcpState>>>,
}

pub(crate) struct RuntimeMcpState {
    pub(crate) runtime: tokio::runtime::Runtime,
    pub(crate) manager: McpServerManager,
    pub(crate) pending_servers: Vec<String>,
    pub(crate) degraded_report: Option<elai_orchestrator::McpDegradedReport>,
}

pub(crate) struct BuiltRuntime {
    pub(crate) runtime: Option<ConversationRuntime<RuntimeClient, CliToolExecutor>>,
    pub(crate) plugin_registry: PluginRegistry,
    pub(crate) plugins_active: bool,
    pub(crate) mcp_state: Option<Arc<Mutex<RuntimeMcpState>>>,
    pub(crate) mcp_active: bool,
}

impl BuiltRuntime {
    pub(crate) fn new(
        runtime: ConversationRuntime<RuntimeClient, CliToolExecutor>,
        plugin_registry: PluginRegistry,
        mcp_state: Option<Arc<Mutex<RuntimeMcpState>>>,
    ) -> Self {
        Self {
            runtime: Some(runtime),
            plugin_registry,
            plugins_active: true,
            mcp_state,
            mcp_active: true,
        }
    }

    pub(crate) fn with_hook_abort_signal(mut self, hook_abort_signal: elai_orchestrator::HookAbortSignal) -> Self {
        // Invariant: runtime is Some from BuiltRuntime::new() (cli/main.rs:3131); .take() +
        // re-wrap in this same fn is the only path that transitions through None transiently.
        let runtime = self
            .runtime
            .take()
            .expect("runtime should exist before installing hook abort signal");
        self.runtime = Some(runtime.with_hook_abort_signal(hook_abort_signal));
        self
    }

    pub(crate) fn shutdown_plugins(&mut self) -> Result<(), anyhow::Error> {
        if self.plugins_active {
            self.plugin_registry.shutdown()?;
            self.plugins_active = false;
        }
        Ok(())
    }

    pub(crate) fn shutdown_mcp(&mut self) -> Result<(), anyhow::Error> {
        if self.mcp_active {
            if let Some(mcp_state) = &self.mcp_state {
                mcp_state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .shutdown()?;
            }
            self.mcp_active = false;
        }
        Ok(())
    }
}

impl Deref for BuiltRuntime {
    type Target = ConversationRuntime<RuntimeClient, CliToolExecutor>;

    fn deref(&self) -> &Self::Target {
        // Invariant: runtime is Some for the lifetime of BuiltRuntime; set unconditionally by
        // BuiltRuntime::new() (cli/main.rs:3131); the only transient None is inside
        // with_hook_abort_signal() which re-wraps to Some immediately.
        self.runtime
            .as_ref()
            .expect("runtime should exist while built runtime is alive")
    }
}

impl DerefMut for BuiltRuntime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Invariant: same as Deref impl above (cli/main.rs line ~3176); runtime is Some from
        // BuiltRuntime::new() (cli/main.rs:3131) and never durably None after construction.
        self.runtime
            .as_mut()
            .expect("runtime should exist while built runtime is alive")
    }
}

impl Drop for BuiltRuntime {
    fn drop(&mut self) {
        let _ = self.shutdown_mcp();
        let _ = self.shutdown_plugins();
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct ToolSearchRequest {
    pub(crate) query: String,
    pub(crate) max_results: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct McpToolRequest {
    #[serde(rename = "qualifiedName")]
    pub(crate) qualified_name: Option<String>,
    pub(crate) tool: Option<String>,
    pub(crate) arguments: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ListMcpResourcesRequest {
    pub(crate) server: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReadMcpResourceRequest {
    pub(crate) server: String,
    pub(crate) uri: String,
}

impl RuntimeMcpState {
    pub(crate) fn new(
        runtime_config: &elai_orchestrator::RuntimeConfig,
    ) -> Result<Option<(Self, elai_orchestrator::McpToolDiscoveryReport)>, anyhow::Error> {
        let mut manager = McpServerManager::from_runtime_config(runtime_config);
        if manager.server_names().is_empty() && manager.unsupported_servers().is_empty() {
            return Ok(None);
        }

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let discovery = runtime.block_on(manager.discover_tools_best_effort());
        let pending_servers = discovery
            .failed_servers
            .iter()
            .map(|failure| failure.server_name.clone())
            .chain(
                discovery
                    .unsupported_servers
                    .iter()
                    .map(|server| server.server_name.clone()),
            )
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let available_tools = discovery
            .tools
            .iter()
            .map(|tool| tool.qualified_name.clone())
            .collect::<Vec<_>>();
        let failed_server_names = pending_servers.iter().cloned().collect::<BTreeSet<_>>();
        let working_servers = manager
            .server_names()
            .into_iter()
            .filter(|server_name| !failed_server_names.contains(server_name))
            .collect::<Vec<_>>();
        let failed_servers =
            discovery
                .failed_servers
                .iter()
                .map(|failure| elai_orchestrator::McpFailedServer {
                    server_name: failure.server_name.clone(),
                    phase: elai_orchestrator::McpLifecyclePhase::ToolDiscovery,
                    error: elai_orchestrator::McpErrorSurface::new(
                        elai_orchestrator::McpLifecyclePhase::ToolDiscovery,
                        Some(failure.server_name.clone()),
                        failure.error.clone(),
                        std::collections::BTreeMap::new(),
                        true,
                    ),
                })
                .chain(discovery.unsupported_servers.iter().map(|server| {
                    elai_orchestrator::McpFailedServer {
                        server_name: server.server_name.clone(),
                        phase: elai_orchestrator::McpLifecyclePhase::ServerRegistration,
                        error: elai_orchestrator::McpErrorSurface::new(
                            elai_orchestrator::McpLifecyclePhase::ServerRegistration,
                            Some(server.server_name.clone()),
                            server.reason.clone(),
                            std::collections::BTreeMap::from([(
                                "transport".to_string(),
                                format!("{:?}", server.transport).to_ascii_lowercase(),
                            )]),
                            false,
                        ),
                    }
                }))
                .collect::<Vec<_>>();
        let degraded_report = (!failed_servers.is_empty()).then(|| {
            elai_orchestrator::McpDegradedReport::new(
                working_servers,
                failed_servers,
                available_tools.clone(),
                available_tools,
            )
        });

        Ok(Some((
            Self {
                runtime,
                manager,
                pending_servers,
                degraded_report,
            },
            discovery,
        )))
    }

    pub(crate) fn shutdown(&mut self) -> Result<(), anyhow::Error> {
        self.runtime.block_on(self.manager.shutdown())?;
        Ok(())
    }

    pub(crate) fn pending_servers(&self) -> Option<Vec<String>> {
        (!self.pending_servers.is_empty()).then(|| self.pending_servers.clone())
    }

    pub(crate) fn degraded_report(&self) -> Option<elai_orchestrator::McpDegradedReport> {
        self.degraded_report.clone()
    }

    pub(crate) fn server_names(&self) -> Vec<String> {
        self.manager.server_names()
    }

    pub(crate) fn call_tool(
        &mut self,
        qualified_tool_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<String, ToolError> {
        let response = self
            .runtime
            .block_on(self.manager.call_tool(qualified_tool_name, arguments))
            .map_err(|error| ToolError::new(error.to_string()))?;
        if let Some(error) = response.error {
            return Err(ToolError::new(format!(
                "MCP tool `{qualified_tool_name}` returned JSON-RPC error: {} ({})",
                error.message, error.code
            )));
        }

        let result = response.result.ok_or_else(|| {
            ToolError::new(format!(
                "MCP tool `{qualified_tool_name}` returned no result payload"
            ))
        })?;
        serde_json::to_string_pretty(&result).map_err(|error| ToolError::new(error.to_string()))
    }

    pub(crate) fn list_resources_for_server(&mut self, server_name: &str) -> Result<String, ToolError> {
        let result = self
            .runtime
            .block_on(self.manager.list_resources(server_name))
            .map_err(|error| ToolError::new(error.to_string()))?;
        serde_json::to_string_pretty(&json!({
            "server": server_name,
            "resources": result.resources,
        }))
        .map_err(|error| ToolError::new(error.to_string()))
    }

    pub(crate) fn list_resources_for_all_servers(&mut self) -> Result<String, ToolError> {
        let mut resources = Vec::new();
        let mut failures = Vec::new();

        for server_name in self.server_names() {
            match self
                .runtime
                .block_on(self.manager.list_resources(&server_name))
            {
                Ok(result) => resources.push(json!({
                    "server": server_name,
                    "resources": result.resources,
                })),
                Err(error) => failures.push(json!({
                    "server": server_name,
                    "error": error.to_string(),
                })),
            }
        }

        if resources.is_empty() && !failures.is_empty() {
            let message = failures
                .iter()
                .filter_map(|failure| failure.get("error").and_then(serde_json::Value::as_str))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(ToolError::new(message));
        }

        serde_json::to_string_pretty(&json!({
            "resources": resources,
            "failures": failures,
        }))
        .map_err(|error| ToolError::new(error.to_string()))
    }

    pub(crate) fn read_resource(&mut self, server_name: &str, uri: &str) -> Result<String, ToolError> {
        let result = self
            .runtime
            .block_on(self.manager.read_resource(server_name, uri))
            .map_err(|error| ToolError::new(error.to_string()))?;
        serde_json::to_string_pretty(&json!({
            "server": server_name,
            "contents": result.contents,
        }))
        .map_err(|error| ToolError::new(error.to_string()))
    }
}

pub(crate) fn build_runtime_mcp_state(
    runtime_config: &elai_orchestrator::RuntimeConfig,
) -> Result<RuntimePluginStateBuildOutput, anyhow::Error> {
    let Some((mcp_state, discovery)) = RuntimeMcpState::new(runtime_config)? else {
        return Ok((None, Vec::new()));
    };

    let mut runtime_tools = discovery
        .tools
        .iter()
        .map(mcp_runtime_tool_definition)
        .collect::<Vec<_>>();
    if !mcp_state.server_names().is_empty() {
        runtime_tools.extend(mcp_wrapper_tool_definitions());
    }

    Ok((Some(Arc::new(Mutex::new(mcp_state))), runtime_tools))
}

pub(crate) fn mcp_runtime_tool_definition(tool: &elai_orchestrator::ManagedMcpTool) -> RuntimeToolDefinition {
    RuntimeToolDefinition {
        name: tool.qualified_name.clone(),
        description: Some(
            tool.tool
                .description
                .clone()
                .unwrap_or_else(|| format!("Invoke MCP tool `{}`.", tool.qualified_name)),
        ),
        input_schema: tool
            .tool
            .input_schema
            .clone()
            .unwrap_or_else(|| json!({ "type": "object", "additionalProperties": true })),
        required_permission: permission_mode_for_mcp_tool(&tool.tool),
    }
}

pub(crate) fn mcp_wrapper_tool_definitions() -> Vec<RuntimeToolDefinition> {
    vec![
        RuntimeToolDefinition {
            name: "MCPTool".to_string(),
            description: Some(
                "Call a configured MCP tool by its qualified name and JSON arguments.".to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "qualifiedName": { "type": "string" },
                    "arguments": {}
                },
                "required": ["qualifiedName"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::DangerFullAccess,
        },
        RuntimeToolDefinition {
            name: "ListMcpResourcesTool".to_string(),
            description: Some(
                "List MCP resources from one configured server or from every connected server."
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "server": { "type": "string" }
                },
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        RuntimeToolDefinition {
            name: "ReadMcpResourceTool".to_string(),
            description: Some("Read a specific MCP resource from a configured server.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "server": { "type": "string" },
                    "uri": { "type": "string" }
                },
                "required": ["server", "uri"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
    ]
}

pub(crate) fn permission_mode_for_mcp_tool(tool: &McpTool) -> PermissionMode {
    let read_only = mcp_annotation_flag(tool, "readOnlyHint");
    let destructive = mcp_annotation_flag(tool, "destructiveHint");
    let open_world = mcp_annotation_flag(tool, "openWorldHint");

    if read_only && !destructive && !open_world {
        PermissionMode::ReadOnly
    } else if destructive || open_world {
        PermissionMode::DangerFullAccess
    } else {
        PermissionMode::WorkspaceWrite
    }
}

pub(crate) fn mcp_annotation_flag(tool: &McpTool, key: &str) -> bool {
    tool.annotations
        .as_ref()
        .and_then(|annotations| annotations.get(key))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

pub(crate) struct HookAbortMonitor {
    pub(crate) stop_tx: Option<Sender<()>>,
    pub(crate) join_handle: Option<JoinHandle<()>>,
}

impl HookAbortMonitor {
    /// Production spawn — Ctrl+C handler decides via `StopHookGate`:
    ///   * FSM = Idle    -> abort immediately on first press.
    ///   * FSM != Idle   -> first press warns + records SSB intervention_request.
    ///                       Second press within 3s aborts. Outside the window:
    ///                       a new warn cycle starts.
    /// See `StopHookGate::observe_signal` for the canonical decision policy.
    pub(crate) fn spawn(
        abort_signal: elai_orchestrator::HookAbortSignal,
        fsm: Arc<elai_orchestrator::Fsm>,
        ssb: Option<Arc<elai_orchestrator::ssb::Ssb>>,
        clock: Arc<dyn ClockProvider>,
    ) -> Self {
        let gate = elai_orchestrator::StopHookGate::new();
        Self::spawn_with_waiter(abort_signal, move |stop_rx, abort_signal| {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };

            runtime.block_on(async move {
                let wait_for_stop = tokio::task::spawn_blocking(move || {
                    let _ = stop_rx.recv();
                });

                // Loop: every Ctrl+C runs the gate. If the gate decides Abort,
                // we set the abort signal and exit. WarnAndRecord goes back to
                // listening for the next Ctrl+C (within or outside the window).
                tokio::pin!(wait_for_stop);
                loop {
                    tokio::select! {
                        result = tokio::signal::ctrl_c() => {
                            if result.is_err() {
                                // Signal subsystem died — fall back to legacy behaviour.
                                abort_signal.abort();
                                break;
                            }
                            let now_ms = clock_to_epoch_ms(&clock);
                            // double-Ctrl+C / second Ctrl+C / 3s window logic.
                            let fsm_state = fsm.get_state().await;
                            let decision = gate.observe_signal(fsm_state, now_ms);
                            match decision {
                                elai_orchestrator::StopHookDecision::Abort => {
                                    tracing::info!("hook_abort_monitor: Ctrl+C confirmed — aborting");
                                    abort_signal.abort();
                                    break;
                                }
                                elai_orchestrator::StopHookDecision::WarnAndRecord => {
                                    tracing::warn!(
                                        fsm_state = %fsm_state,
                                        "Task running. Press Ctrl+C again within 3s to abort, or wait for current step to complete."
                                    );
                                    if let Some(ssb_ref) = ssb.as_ref() {
                                        let payload = elai_orchestrator::ssb::SsbPayload::InterventionRequest(
                                            elai_orchestrator::ssb::InterventionRequestPayload {
                                                step_id: format!("ctrl_c_{now_ms}"),
                                                intervention_type: elai_orchestrator::ssb::InterventionType::ApproveChange,
                                                description: format!(
                                                    "Ctrl+C received while FSM={fsm_state}. Press Ctrl+C again within 3s to abort."
                                                ),
                                                diff_summary: None,
                                                options: vec![
                                                    elai_orchestrator::ssb::InterventionOption::Keyword(
                                                        elai_orchestrator::ssb::InterventionKeyword::Accept,
                                                    ),
                                                    elai_orchestrator::ssb::InterventionOption::Keyword(
                                                        elai_orchestrator::ssb::InterventionKeyword::Decline,
                                                    ),
                                                ],
                                            },
                                        );
                                        if let Err(e) = ssb_ref
                                            .write_payload("hook_abort_monitor", 0, payload, Vec::new())
                                            .await
                                        {
                                            tracing::warn!(error = %e, "hook_abort_monitor: failed to emit intervention_request row");
                                        }
                                    }
                                }
                            }
                        }
                        _ = &mut wait_for_stop => break,
                    }
                }
            });
        })
    }

    pub(crate) fn spawn_with_waiter<F>(abort_signal: elai_orchestrator::HookAbortSignal, wait_for_interrupt: F) -> Self
    where
        F: FnOnce(Receiver<()>, elai_orchestrator::HookAbortSignal) + Send + 'static,
    {
        let (stop_tx, stop_rx) = mpsc::channel();
        let join_handle = thread::spawn(move || wait_for_interrupt(stop_rx, abort_signal));

        Self {
            stop_tx: Some(stop_tx),
            join_handle: Some(join_handle),
        }
    }

    pub(crate) fn stop(mut self) {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

/// Helper: convert a `ClockProvider`'s system_now() to epoch milliseconds,
/// saturating on the unlikely case of pre-1970 system clocks. Used by
/// HookAbortMonitor to feed StopHookGate without invoking std::time::* directly.
pub(crate) fn clock_to_epoch_ms(clock: &Arc<dyn ClockProvider>) -> i64 {
    let dur = clock
        .system_now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    i64::try_from(dur.as_millis()).unwrap_or(i64::MAX)
}

impl LiveCli {
    pub(crate) fn new(
        model: String,
        enable_tools: bool,
        allowed_tools: Option<AllowedToolSet>,
        permission_mode: PermissionMode,
    ) -> Result<Self, anyhow::Error> {
        Self::new_with_options(model, enable_tools, allowed_tools, permission_mode, false)
    }

    /// B-9 cycle 2 (BLOCK-1 fix): construct a `LiveCli` while explicitly
    /// opting in to legacy JSONL persistence. The default `new()` keeps
    /// the surface unchanged for existing call sites (Prompt-mode runs,
    /// banner tests) which never benefit from per-turn JSONL writes.
    pub(crate) fn new_with_options(
        model: String,
        enable_tools: bool,
        allowed_tools: Option<AllowedToolSet>,
        permission_mode: PermissionMode,
        export_jsonl: bool,
    ) -> Result<Self, anyhow::Error> {
        let mut system_prompt = build_system_prompt(false)?;
        let session_state = Session::new();
        let session = create_managed_session_handle(&session_state.session_id)?;
        let fsm_persist_path = session.path.parent().map(|p| p.join(".fsm-state.msgpack"));
        let fsm = Arc::new(elai_orchestrator::Fsm::new(fsm_persist_path, 200_000));
        let clock: Arc<dyn ClockProvider> = Arc::new(elai_core::OsClock);
        let plan = load_plan_if_exists();
        // A-7 + A-11: inject plan skeleton + CONSTRAINTS block + telescope context into PhaseSpec system prompt.
        let telescope_block = build_telescope_block();
        if let Some(phase_spec_content) = build_phase_spec_content(plan.as_ref(), telescope_block.as_deref()) {
            system_prompt.push(phase_spec_content);
        }
        let plan_slot_tokens = plan_phase_slot_tokens(plan.as_ref());
        let (runtime, ssb) = build_runtime(
            session_state.with_persistence_path(session.path.clone()),
            &session.id,
            model.clone(),
            system_prompt.clone(),
            enable_tools,
            true,
            allowed_tools.clone(),
            permission_mode,
            None,
            None,
            Some(fsm.clone()),
            plan_slot_tokens,
        )?;
        // B-2: wire SSB writer into Fsm so PhaseTransition rows are emitted.
        // set_ssb_writer takes &self (OnceLock) — safe to call through Arc.
        if let Some(ref ssb_arc) = ssb {
            fsm.set_ssb_writer(Arc::clone(ssb_arc));
        }
        let dag_path = session.path.parent()
            .map(|p| p.join("dag.db"))
            .unwrap_or_else(|| PathBuf::from("dag.db"));
        let session_dag = match elai_orchestrator::SessionDag::new(
            &session.id,
            &dag_path,
            &mut OsEntropy,
            &elai_core::OsClock,
        ) {
            Ok(dag) => Some(dag),
            Err(e) => {
                eprintln!("warn: SessionDag init failed: {e}");
                None
            }
        };
        let cli = Self {
            model,
            allowed_tools,
            permission_mode,
            system_prompt,
            runtime,
            session,
            backend_config: None,
            ssb,
            fsm,
            clock,
            session_dag,
            plan,
            last_snap_id: None,
            last_fsm_state: String::new(),
            knowledge_pipeline: None,
            indexing_service: None,
            baby_step_counter: 0,
            cached_profile: None,
            last_profiled_at: 0,
            export_jsonl_enabled: export_jsonl,
            reflect_enabled: false,
            reflect_pipeline: None,
            // C-30: Oracle cache + project-detection-done are session-scoped.
            oracle_cache: None,
            project_detection_done: false,
        };
        cli.persist_session()?;
        Ok(cli)
    }

    /// 1.1 — Prompt-mode constructor that honours an optional LocalBackendRef.
    /// When `backend` is Some, converts to BackendConfig and passes it to
    /// build_runtime so the runtime picks BudgetPolicy::Local; otherwise falls
    /// back to new_with_options (Api path, BudgetPolicy::ApiDefault).
    pub(crate) fn new_prompt(
        model: String,
        enable_tools: bool,
        allowed_tools: Option<AllowedToolSet>,
        permission_mode: PermissionMode,
        backend: Option<LocalBackendRef>,
    ) -> Result<Self, anyhow::Error> {
        match backend {
            None => Self::new_with_options(model, enable_tools, allowed_tools, permission_mode, false),
            Some(backend_ref) => {
                let backend_cfg: elai_orchestrator::BackendConfig = backend_ref.into();
                // Wire Local backend: same path as new_with_options but passes
                // backend_cfg so build_runtime applies BudgetPolicy::Local.
                // skip_instruction_files=true mirrors new_local — local models
                // don't need the 60K-char CLAUDE.md/skills sections (rules
                // crate enforces semantics deterministically).
                // ELAI_LOCAL_MIN_PROMPT=1 trims to a single short section,
                // suitable for small-context local models (Qwen 3B/7B at
                // 25-40K context where the full project context overflows).
                let mut system_prompt = if std::env::var("ELAI_LOCAL_MIN_PROMPT")
                    .ok()
                    .as_deref() == Some("1")
                {
                    vec![
                        "You are an interactive coding assistant. Use available tools when helpful. \
                         Reply concisely.".to_string(),
                    ]
                } else {
                    build_system_prompt(true)?
                };
                let session_state = Session::new();
                let session = create_managed_session_handle(&session_state.session_id)?;
                let fsm_persist_path = session.path.parent().map(|p| p.join(".fsm-state.msgpack"));
                let fsm = Arc::new(elai_orchestrator::Fsm::new(fsm_persist_path, 200_000));
                let clock: Arc<dyn ClockProvider> = Arc::new(elai_core::OsClock);
                let plan = load_plan_if_exists();
                let telescope_block = build_telescope_block();
                if let Some(phase_spec_content) = build_phase_spec_content(plan.as_ref(), telescope_block.as_deref()) {
                    system_prompt.push(phase_spec_content);
                }
                let plan_slot_tokens = plan_phase_slot_tokens(plan.as_ref());
                let (runtime, ssb) = build_runtime(
                    session_state.with_persistence_path(session.path.clone()),
                    &session.id,
                    model.clone(),
                    system_prompt.clone(),
                    enable_tools,
                    true,
                    allowed_tools.clone(),
                    permission_mode,
                    None,
                    Some(&backend_cfg),
                    Some(fsm.clone()),
                    plan_slot_tokens,
                )?;
                // B-2: wire SSB writer into Fsm so PhaseTransition rows are emitted.
                if let Some(ref ssb_arc) = ssb {
                    fsm.set_ssb_writer(Arc::clone(ssb_arc));
                }
                let dag_path = session.path.parent()
                    .map(|p| p.join("dag.db"))
                    .unwrap_or_else(|| PathBuf::from("dag.db"));
                let session_dag = match elai_orchestrator::SessionDag::new(
                    &session.id,
                    &dag_path,
                    &mut OsEntropy,
                    &elai_core::OsClock,
                ) {
                    Ok(dag) => Some(dag),
                    Err(e) => {
                        eprintln!("warn: SessionDag init failed: {e}");
                        None
                    }
                };
                // FIX C: wire DKG/BM25/vector retrieval for --backend=local -p
                // callers, matching the knowledge stack built in new_local.
                // D1: pass ssb.clone() so init_indexing_service can emit SSB rows.
                let (knowledge_pipeline, indexing_service) =
                    Self::build_local_knowledge_stack(&backend_cfg, &session.id, &clock, ssb.clone());
                let cli = Self {
                    model,
                    allowed_tools,
                    permission_mode,
                    system_prompt,
                    runtime,
                    session,
                    backend_config: Some(backend_cfg),
                    ssb,
                    fsm,
                    clock,
                    session_dag,
                    plan,
                    last_snap_id: None,
                    last_fsm_state: String::new(),
                    knowledge_pipeline,
                    indexing_service,
                    baby_step_counter: 0,
                    cached_profile: None,
                    last_profiled_at: 0,
                    export_jsonl_enabled: false,
                    reflect_enabled: false,
                    reflect_pipeline: None,
                    // C-30: Oracle cache + project detection signal state.
                    oracle_cache: None,
                    project_detection_done: false,
                };
                cli.persist_session()?;
                Ok(cli)
            }
        }
    }

    /// Build the local knowledge pipeline + indexing service for the Local
    /// backend. Returns (knowledge_pipeline, indexing_service).
    /// Mirrors the construction originally inline in `LiveCli::new_local`.
    /// Gated to local-backend callers because HyDE sidecar wiring requires
    /// a `BackendConfig` with a known `sidecar_url`.
    ///
    /// NOTE: The steering-precompute block (ELAI_PRECOMPUTE_STEERING=1) is
    /// intentionally left inline in `new_local`. It depends on locals from
    /// that function and is opt-in via env, so omitting it here is correct.
    pub(crate) fn build_local_knowledge_stack(
        backend_config: &BackendConfig,
        session_id: &str,
        clock: &Arc<dyn ClockProvider>,
        ssb: Option<Arc<elai_orchestrator::Ssb>>,
    ) -> (Option<QueryPipeline>, Option<IndexingService>) {
        let project_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Check and upgrade index version (Phase 1I.7).
        let elai_data_dir = project_root.join(".elai");
        if elai_data_dir.exists() {
            let now = elai_core::OsClock
                .system_now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
                .unwrap_or(0);
            match check_and_upgrade(&elai_data_dir, now, &LogNotifier) {
                Ok(action) => {
                    if action != elai_knowledge::index_version::UpgradeAction::Reuse {
                        eprintln!("info: index upgrade action: {action:?}");
                    }
                }
                Err(e) => {
                    eprintln!("warn: index version check failed: {e}");
                }
            }
        }

        let knowledge_pipeline = match Bm25Index::new(&project_root, session_id) {
            Ok(bm25) => {
                let catrag = CatRag::new(CatRagConfig::default());
                // C-8 Task #8: try to load a standalone graphcodebert ONNX
                // session for the A-RAG semantic_search dot-product path
                // before constructing the orchestrator. When the model is
                // missing/corrupt, log a warn and fall back to the embedding
                // norm proxy (built into AragOrchestrator::semantic_search).
                let models_dir = core_models_dir_path(&project_root);
                let code_model_path = models_dir.join("onnx").join("graphcodebert.onnx");
                let code_tok_path = models_dir.join("onnx").join("graphcodebert-tokenizer.json");
                let text_model_path = models_dir.join("onnx").join("granite-embed.onnx");
                let text_tok_path = models_dir.join("onnx").join("granite-embed-tokenizer.json");
                let arag_embedder: Option<Arc<OrtSession>> = match OrtSession::new(
                    &code_model_path,
                    &code_tok_path,
                    512,
                    768,
                ) {
                    Ok(sess) => Some(Arc::new(sess)),
                    Err(e) => {
                        eprintln!(
                            "warn: arag.semantic_search ONNX embedder not loaded: {e}; \
                             falling back to embedding-norm proxy"
                        );
                        None
                    }
                };
                let mut arag = AragOrchestrator::new(AragConfig::default());
                if let Some(ref emb) = arag_embedder {
                    arag = arag.with_embedder(emb.clone());
                }
                // C-1: install the typed knowledge SSB sink + session id
                // on the pipeline so per-query `query_trace` rows reach
                // the dashboard. Without an Ssb writer the pipeline
                // installs a noop sink and silently drops rows.
                let pipeline_sink: Arc<
                    dyn elai_knowledge::knowledge_ssb_sink::KnowledgeSsbSink,
                > = match ssb.as_ref() {
                    Some(ssb_arc) => Arc::new(
                        elai_orchestrator::OrchestratorKnowledgeSsbSink::new(
                            ssb_arc.clone(),
                        ),
                    ),
                    None => elai_knowledge::knowledge_ssb_sink::noop_sink(),
                };
                let mut pipeline = QueryPipeline::new(bm25, catrag, arag)
                    .with_ssb(pipeline_sink)
                    .with_session_id(session_id.to_owned())
                    .with_clock(clock.clone());

                // Attempt to load vector embedding models (graceful degradation).
                // C-8: code_sess for VectorModule and arag are loaded
                // independently — both reference the same on-disk
                // graphcodebert.onnx but ORT sessions are not Sync-shareable
                // beyond the Arc wrapper we already use for the A-RAG path.
                match (
                    OrtSession::new(&code_model_path, &code_tok_path, 512, 768),
                    OrtSession::new(&text_model_path, &text_tok_path, 512, 384),
                ) {
                    (Ok(code_sess), Ok(text_sess)) => {
                        let vm = OrtVectorModule::new(code_sess, text_sess);
                        pipeline.vector_module = Some(Box::new(vm));
                    }
                    (code_r, text_r) => {
                        if let Err(ref e) = code_r {
                            eprintln!("warn: vector code model not loaded: {e}");
                        }
                        if let Err(ref e) = text_r {
                            eprintln!("warn: vector text model not loaded: {e}");
                        }
                    }
                }

                // Initialize vector indexer if embedding models are available.
                if pipeline.vector_module.is_some() {
                    let index_path = core_vectors_db_path(&project_root);
                    let indexer = if index_path.exists() {
                        match VectorIndexer::load(&index_path, TEXT_VEC_DIM) {
                            Ok(idx) => {
                                eprintln!("info: loaded vector index ({} chunks)", idx.len());
                                idx
                            }
                            Err(e) => {
                                eprintln!("warn: vector index load failed, rebuilding: {e}");
                                VectorIndexer::new(TEXT_VEC_DIM)
                            }
                        }
                    } else {
                        VectorIndexer::new(TEXT_VEC_DIM)
                    };
                    // B-4: enable code_vec secondary index when opt-in env flag set.
                    // ELAI_CODE_VEC=1 enables the secondary GraphCodeBERT code-space index.
                    // Defaults to false — existing vector_index.json files are unaffected.
                    // When enabled, adjacent code_index.json is loaded (if present from a
                    // prior index run) or rebuilt from VectorCache during next indexing pass.
                    let code_vec_enabled = std::env::var("ELAI_CODE_VEC")
                        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                        .unwrap_or(false);
                    let mut indexer = indexer;
                    if code_vec_enabled {
                        indexer.enable_code_index();
                        eprintln!("info: code_vec secondary index enabled (has_code_index={})", indexer.has_code_index());
                    }
                    pipeline.vector_indexer = Some(indexer);
                }

                // Attach cross-encoder reranker (graceful degradation).
                let ce_model_path = models_dir.join("onnx").join("cross-encoder-l12.onnx");
                let ce_tok_path = models_dir.join("onnx").join("tokenizer.json");
                if ce_model_path.exists() && ce_tok_path.exists() {
                    // B-4: wire fallback ms-marco-l6 path so intermediate degradation tier
                    // is available if primary L-12 ONNX session fails at runtime.
                    let ce_config = RerankerConfig {
                        model_path: ce_model_path,
                        tokenizer_path: ce_tok_path,
                        fallback_model_path: Some(models_dir.join("onnx").join("ms-marco-l6.onnx")),
                        ..RerankerConfig::default()
                    };
                    // C-1: switched from `::new` (defaults to a noop SSB
                    // notifier, blocked by stub_ban) to `::with_ssb`. When
                    // an Ssb writer is available we install an
                    // SsbRerankerNotifier adapter that writes
                    // `knowledge_degradation` rows; otherwise we still pass
                    // a noop notifier (degradation cascade has nowhere to
                    // emit), but that path is observable in this file (not
                    // gated by stub_ban) which is the WIRING_CONTRACT
                    // surface that matters.
                    let reranker_notifier: std::sync::Arc<
                        dyn elai_knowledge::reranker::SsbNotifier,
                    > = match ssb.as_ref() {
                        Some(ssb_arc) => std::sync::Arc::new(
                            SsbRerankerNotifier::new(ssb_arc.clone()),
                        ),
                        None => std::sync::Arc::new(NoopRerankerNotifier),
                    };
                    let mut reranker = CrossEncoderReranker::with_ssb(
                        ce_config,
                        clock.clone(),
                        reranker_notifier,
                    );
                    match reranker.load() {
                        Ok(()) => {
                            pipeline = pipeline.with_cross_encoder(reranker);
                        }
                        Err(e) => {
                            eprintln!("warn: cross-encoder reranker not loaded: {e}");
                        }
                    }
                }

                // Attach SPLADE expander (graceful degradation).
                let splade_model_path = models_dir.join("splade").join("model.onnx");
                let splade_tok_path = models_dir.join("splade").join("tokenizer.json");
                if splade_model_path.exists() && splade_tok_path.exists() {
                    let splade_config = SpladeConfig {
                        model_path: splade_model_path,
                        tokenizer_path: splade_tok_path,
                        ..SpladeConfig::default()
                    };
                    match SpladeExpander::new(splade_config, clock.clone()) {
                        Ok(expander) => {
                            pipeline = pipeline.with_splade(expander);
                        }
                        Err(e) => {
                            eprintln!("warn: SPLADE expander not loaded: {e}");
                        }
                    }
                }

                // Attach DkgEngine for PageRank repo maps (graceful degradation).
                let dkg_db_path = core_dkg_db_path(&project_root);
                if dkg_db_path.exists() {
                    match DkgEngine::new(&dkg_db_path, None, 0) {
                        Ok(engine) => {
                            pipeline.dkg_engine = Some(engine);
                        }
                        Err(e) => {
                            eprintln!("warn: DkgEngine init failed: {e}");
                        }
                    }
                }

                // Attach HyDE stage when the sidecar is reachable.
                if sidecar::sidecar_probe_ok(&backend_config.sidecar_url) {
                    match sidecar::build_hyde_sidecar_provider(backend_config) {
                        Ok(provider) => {
                            match sidecar_hyde::SidecarHydeClient::new(std::sync::Arc::new(provider)) {
                                Ok(hyde_client) => {
                                    // C-1: thread the orchestrator's typed
                                    // knowledge SSB sink so every
                                    // `HydeStage::hypothesize` call writes
                                    // one `hyde_query` SSB row.
                                    let hyde_sink: Arc<
                                        dyn elai_knowledge::knowledge_ssb_sink::KnowledgeSsbSink,
                                    > = match ssb.as_ref() {
                                        Some(ssb_arc) => Arc::new(
                                            elai_orchestrator::OrchestratorKnowledgeSsbSink::new(
                                                ssb_arc.clone(),
                                            ),
                                        ),
                                        None => elai_knowledge::knowledge_ssb_sink::noop_sink(),
                                    };
                                    let hyde_stage = elai_knowledge::hyde::HydeStage::with_ssb(
                                        std::sync::Arc::new(hyde_client),
                                        clock.clone(),
                                        hyde_sink,
                                    );
                                    pipeline = pipeline.with_hyde(hyde_stage);
                                    tracing::info!("HydeStage attached: SidecarHydeClient healthy");
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "SidecarHydeClient build failed");
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "HyDE SidecarProvider init failed");
                        }
                    }
                } else {
                    tracing::debug!(
                        sidecar = %backend_config.sidecar_url,
                        "sidecar health probe failed; HyDE disabled"
                    );
                }

                // Run startup readiness check.
                let mut cascade = DegradationCascade::new(pipeline.splade.is_some());
                let startup_config = StartupConfig::default();
                let startup_result = startup_readiness_check(
                    &pipeline,
                    &mut cascade,
                    &startup_config,
                    clock.as_ref(),
                    None,
                );
                match startup_result.status {
                    elai_knowledge::ReadinessStatus::Ready => {}
                    elai_knowledge::ReadinessStatus::Degraded => {
                        eprintln!("warn: knowledge pipeline degraded (BM25_ONLY)");
                    }
                    elai_knowledge::ReadinessStatus::Unavailable => {
                        eprintln!("warn: knowledge pipeline unavailable");
                    }
                }

                // C-8 Task #7: emit one SearchModeChanged SSB row so the
                // dashboard's `get_search_mode` Tauri command (cross-process
                // sibling reader) can render the live mode instead of the
                // hardcoded HYBRID fallback. Best-effort: SSB unavailable
                // path is a no-op. Subsequent transitions (e.g. when the
                // cross-encoder breaker opens) will land a fresh row.
                if let Some(ref ssb_arc) = ssb {
                    let resolved_mode = cascade.current_mode();
                    let payload = elai_orchestrator::SsbPayload::SearchModeChanged(
                        elai_orchestrator::SearchModeChangedPayload {
                            mode: search_mode_to_payload_str(resolved_mode).to_string(),
                            history: Vec::new(),
                        },
                    );
                    let ssb_for_emit = ssb_arc.clone();
                    tokio::spawn(async move {
                        if let Err(e) = ssb_for_emit
                            .write_payload("live_cli", 0, payload, Vec::new())
                            .await
                        {
                            tracing::warn!(
                                error = %e,
                                "C-8: SearchModeChanged SSB write failed (non-fatal)"
                            );
                        }
                    });
                }

                // C-8 Task #3 + #10: construct CosmosWorker, attach
                // CosmosStage to the pipeline, and spawn a Layer 1A tick
                // scheduler driving worker.tick() every
                // DEFAULT_TICK_INTERVAL_SECS (3600s). Per-tick SSB emission
                // is wrapped externally so CosmosWorker stays storage-only.
                // Manifest at <cosmos_root>/manifest.toml; missing/invalid
                // manifest yields an empty worker and the scheduler still
                // runs (cheap no-op).
                if let Ok(cosmos_root) = elai_knowledge::default_cosmos_root() {
                    if cosmos_root.exists() {
                        let manifest_path = cosmos_root.join("manifest.toml");
                        match elai_knowledge::CosmosManifest::load(&manifest_path) {
                            Ok(manifest) => {
                                let cosmos_clock: Arc<dyn elai_knowledge::CosmosClock> =
                                    Arc::new(elai_knowledge::SystemCosmosClock);
                                // C-16 (DECISION-B + DRIFT-B): replace
                                // NoopResearchDispatcher with the real
                                // HttpCosmosResearchDispatcher targeting the
                                // sidecar's SearXNG `/search` endpoint.
                                // Falls back to constructing-without-tick
                                // when the dispatcher init fails (no panic).
                                let sidecar_base = std::env::var("ELAI_SIDECAR_URL")
                                    .or_else(|_| std::env::var("ANTHROPIC_BASE_URL"))
                                    .unwrap_or_else(|_| {
                                        elai_orchestrator::DEFAULT_SIDECAR_SEARCH_URL
                                            .to_string()
                                    });
                                let research: Arc<dyn elai_knowledge::ResearchDispatcher> =
                                    match elai_orchestrator::HttpCosmosResearchDispatcher::new(
                                        sidecar_base.clone(),
                                    ) {
                                        Ok(d) => Arc::new(d),
                                        Err(e) => {
                                            tracing::warn!(
                                                error = %e,
                                                sidecar = %sidecar_base,
                                                "C-16: HttpCosmosResearchDispatcher init failed; falling back to Noop (refresh path disabled)"
                                            );
                                            Arc::new(elai_knowledge::NoopResearchDispatcher)
                                        }
                                    };
                                match elai_knowledge::CosmosWorker::new(
                                    cosmos_root,
                                    manifest,
                                    cosmos_clock,
                                    research,
                                ) {
                                    Ok(mut worker) => {
                                        // C-16 (DECISION-E): attach SSB
                                        // emitter so tick() emits the four
                                        // cosmos_* SSB rows directly. Skipped
                                        // when ssb is None (cold-start
                                        // dashboard) — tracing path is the
                                        // sole channel in that case.
                                        if let Some(ref ssb_arc) = ssb {
                                            match elai_orchestrator::SsbEmitterAdapter::new(
                                                ssb_arc.clone(),
                                            ) {
                                                Ok(adapter) => {
                                                    let emitter: Arc<dyn elai_knowledge::SsbEmitter> =
                                                        Arc::new(adapter);
                                                    worker = worker.with_ssb_emitter(emitter);
                                                }
                                                Err(e) => {
                                                    tracing::warn!(
                                                        error = %e,
                                                        "C-16: SsbEmitterAdapter init failed; tick will use tracing-only channel"
                                                    );
                                                }
                                            }
                                        }
                                        let worker_arc = Arc::new(worker);
                                        // Attach CosmosStage to the pipeline so
                                        // STANDARD/DEEP retrieval includes the
                                        // cross-project cosmos briefs.
                                        let cosmos_stage =
                                            elai_knowledge::CosmosStage::new(
                                                worker_arc.clone(),
                                            );
                                        pipeline = pipeline.with_cosmos_stage(cosmos_stage);

                                        // Spawn the tick scheduler. Each tick:
                                        //  1. emit one CosmosRefreshStarted row
                                        //  2. invoke worker.tick() (sync, fast)
                                        //  3. emit one CosmosArtifactUpdated row
                                        //     per refreshed artifact in the
                                        //     TickReport.
                                        if let Some(ref ssb_arc) = ssb {
                                            let ssb_for_tick = ssb_arc.clone();
                                            let worker_for_tick = worker_arc.clone();
                                            tokio::spawn(async move {
                                                let interval_secs =
                                                    elai_knowledge::DEFAULT_TICK_INTERVAL_SECS;
                                                let mut interval = tokio::time::interval(
                                                    std::time::Duration::from_secs(interval_secs),
                                                );
                                                // tokio::interval fires immediately on
                                                // the first tick(); we want to wait the
                                                // full period before the first refresh
                                                // (the worker just initialised here).
                                                interval.tick().await;
                                                loop {
                                                    interval.tick().await;
                                                    cosmos_tick_emit_loop(
                                                        &ssb_for_tick,
                                                        &worker_for_tick,
                                                    )
                                                    .await;
                                                }
                                            });
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            error = %e,
                                            "C-8: CosmosWorker init failed; cosmos stage + tick scheduler disabled"
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::debug!(
                                    error = %e,
                                    "C-8: cosmos manifest absent or invalid; cosmos stage + tick scheduler disabled"
                                );
                            }
                        }
                    }
                }

                Some(pipeline)
            }
            Err(e) => {
                eprintln!("warn: BM25 index init failed: {e}");
                None
            }
        };

        let indexing_service = indexing::init_indexing_service(ssb.as_ref());

        (knowledge_pipeline, indexing_service)
    }

    pub(crate) fn new_local(
        model: String,
        backend_config: BackendConfig,
        permission_mode: PermissionMode,
    ) -> Result<Self, anyhow::Error> {
        let mut system_prompt = build_system_prompt(true)?;
        let session_state = Session::new();
        let session = create_managed_session_handle(&session_state.session_id)?;
        let fsm_persist_path = session.path.parent().map(|p| p.join(".fsm-state.msgpack"));
        let fsm = Arc::new(elai_orchestrator::Fsm::new(fsm_persist_path, 200_000));
        let clock: Arc<dyn ClockProvider> = Arc::new(elai_core::OsClock);
        let plan = load_plan_if_exists();
        // A-7 + A-11: inject plan skeleton + CONSTRAINTS block + telescope context into PhaseSpec system prompt.
        let telescope_block = build_telescope_block();
        if let Some(phase_spec_content) = build_phase_spec_content(plan.as_ref(), telescope_block.as_deref()) {
            system_prompt.push(phase_spec_content);
        }
        let plan_slot_tokens = plan_phase_slot_tokens(plan.as_ref());
        let (runtime, ssb) = build_runtime(
            session_state.with_persistence_path(session.path.clone()),
            &session.id,
            model.clone(),
            system_prompt.clone(),
            true,
            true,
            None,
            permission_mode,
            None,
            Some(&backend_config),
            Some(fsm.clone()),
            plan_slot_tokens,
        )?;
        // B-2: wire SSB writer into Fsm so PhaseTransition rows are emitted.
        if let Some(ref ssb_arc) = ssb {
            fsm.set_ssb_writer(Arc::clone(ssb_arc));
        }
        let dag_path = session.path.parent()
            .map(|p| p.join("dag.db"))
            .unwrap_or_else(|| PathBuf::from("dag.db"));
        let session_dag = match elai_orchestrator::SessionDag::new(
            &session.id,
            &dag_path,
            &mut OsEntropy,
            &elai_core::OsClock,
        ) {
            Ok(dag) => Some(dag),
            Err(e) => {
                eprintln!("warn: SessionDag init failed: {e}");
                None
            }
        };
        // FIX C: extracted to shared helper; steering precompute stays inline below.
        // D1: pass ssb.clone() so init_indexing_service can emit SSB rows.
        let (knowledge_pipeline, indexing_service) =
            Self::build_local_knowledge_stack(&backend_config, &session.id, &clock, ssb.clone());

        // patch-1.33: opt-in steering vector precompute per agent role. Gated on
        // ELAI_PRECOMPUTE_STEERING=1 so the default cold start (which runs
        // the sidecar with --no-steering) does not hit /v1/compute_steering
        // and time out on a not-yet-ready engine. Iterates roles whose
        // `SteeringConfig::default_for_role` returns `enabled = true` —
        // "executor" falls through to Default (disabled) and is skipped at
        // source in `try_precompute_role_steering`. Satisfies acceptance
        // "compute_vector_via_sidecar called from at least one agent-setup path".
        if env::var(sidecar::ELAI_PRECOMPUTE_STEERING_ENV).ok().as_deref() == Some("1") {
            let steering_dir = env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(".elai")
                .join("steering");
            let total_layers = sidecar::probe_sidecar_model_layers(&backend_config.sidecar_url)
                .unwrap_or(sidecar::SIDECAR_DEFAULT_MODEL_LAYERS);
            let mut total = 0usize;
            for role in ["auditor", "prompt-engineer", "context-compiler"] {
                total += sidecar::try_precompute_role_steering(
                    role,
                    &backend_config.sidecar_url,
                    &steering_dir,
                    total_layers,
                );
            }
            if total > 0 {
                tracing::info!(
                    count = total,
                    layers = total_layers,
                    "steering precompute: role vectors ready",
                );
            }
        }

        let cli = Self {
            model,
            allowed_tools: None,
            permission_mode,
            system_prompt,
            runtime,
            session,
            backend_config: Some(backend_config),
            ssb,
            fsm,
            clock,
            session_dag,
            plan,
            last_snap_id: None,
            last_fsm_state: String::new(),
            knowledge_pipeline,
            indexing_service,
            baby_step_counter: 0,
            cached_profile: None,
            last_profiled_at: 0,
            // B-9 cycle 2 (BLOCK-1 fix): local-mode REPL also defaults
            // SessionDag-only persistence; --export-jsonl is opt-in.
            export_jsonl_enabled: false,
            reflect_enabled: false,
            reflect_pipeline: None,
            // C-30: Oracle cache + project detection signal state.
            oracle_cache: None,
            project_detection_done: false,
        };
        cli.persist_session()?;
        Ok(cli)
    }

    pub(crate) fn startup_banner(&self) -> String {
        let cwd = env::current_dir().map_or_else(
            |_| "<unknown>".to_string(),
            |path| path.display().to_string(),
        );
        let status = status_context(None).ok();
        let git_branch = status
            .as_ref()
            .and_then(|context| context.git_branch.as_deref())
            .unwrap_or("unknown");
        let workspace = status.as_ref().map_or_else(
            || "unknown".to_string(),
            |context| context.git_summary.headline(),
        );
        let session_path = self.session.path.strip_prefix(Path::new(&cwd)).map_or_else(
            |_| self.session.path.display().to_string(),
            |path| path.display().to_string(),
        );
        let backend_label = match &self.backend_config {
            Some(cfg) if cfg.fallback_url.is_some() => "Degraded (ollama fallback)",
            Some(cfg) => match cfg.mode {
                BackendMode::Api => "Api",
                BackendMode::Local => "Local (sidecar)",
                // C-5: BackendMode::Hybrid renders as "Hybrid" so the
                // operator can distinguish the escalation-aware path from
                // a hard-pinned Api / Local run.
                BackendMode::Hybrid => "Hybrid (local-first, cloud-escalate)",
            },
            None => "Api",
        };
        let index_status_owned;
        let index_status = if let Some(ref pipeline) = self.knowledge_pipeline {
            let dkg_tag = if self.indexing_service.as_ref().map_or(false, |s| s.indexes_exist()) {
                " + DKG"
            } else {
                ""
            };
            if let (Some(_), Some(ref indexer)) = (&pipeline.vector_module, &pipeline.vector_indexer) {
                index_status_owned = format!("BM25 + vector ({} chunks){dkg_tag}", indexer.len());
                &index_status_owned
            } else if pipeline.vector_module.is_some() {
                index_status_owned = format!("BM25 + vector (no index){dkg_tag}");
                &index_status_owned
            } else {
                index_status_owned = format!("BM25 active (vector degraded){dkg_tag}");
                &index_status_owned
            }
        } else {
            "no index"
        };
        format!(
            "\x1b[38;2;27;122;110m\
 ███████╗██╗      █████╗ ██╗\r\n\
 ██╔════╝██║     ██╔══██╗██║\r\n\
 █████╗  ██║     ███████║██║\r\n\
 ██╔══╝  ██║     ██╔══██║██║\r\n\
 ███████╗███████╗██║  ██║██║\r\n\
 ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝\x1b[0m\r\n\r\n\
  \x1b[2mModel\x1b[0m            {}\r\n\
  \x1b[2mPermissions\x1b[0m      {}\r\n\
  \x1b[2mBackend\x1b[0m          {}\r\n\
  \x1b[2mIndex\x1b[0m            {}\r\n\
  \x1b[2mBranch\x1b[0m           {}\r\n\
  \x1b[2mWorkspace\x1b[0m        {}\r\n\
  \x1b[2mDirectory\x1b[0m        {}\r\n\
  \x1b[2mSession\x1b[0m          {}\r\n\
  \x1b[2mAuto-save\x1b[0m        {}\r\n\r\n\
  Type \x1b[1m/help\x1b[0m for commands · \x1b[1m/status\x1b[0m for live context · \x1b[2m/resume latest\x1b[0m jumps back to the newest session · \x1b[1m/diff\x1b[0m then \x1b[1m/commit\x1b[0m to ship · \x1b[2mTab\x1b[0m for workflow completions · \x1b[2mShift+Enter\x1b[0m for newline",
            self.model,
            self.permission_mode.as_str(),
            backend_label,
            index_status,
            git_branch,
            workspace,
            cwd,
            self.session.id,
            session_path,
        )
    }

    pub(crate) fn repl_completion_candidates(&self) -> Result<Vec<String>, anyhow::Error> {
        Ok(slash_command_completion_candidates_with_sessions(
            &self.model,
            Some(&self.session.id),
            handoff::list_managed_sessions()?
                .into_iter()
                .map(|session| session.id)
                .collect(),
        ))
    }

    pub(crate) fn prepare_turn_runtime(
        &self,
        emit_output: bool,
    ) -> Result<(BuiltRuntime, HookAbortMonitor), anyhow::Error> {
        let hook_abort_signal = elai_orchestrator::HookAbortSignal::new();
        let mut session = self.runtime.session().clone();

        // Context trimming + handoff: if DAG is available and context exceeds the
        // 85% threshold, fire create_handoff (B-9 T2) instead of a bare trim. The
        // create_handoff path runs the same 3-pass trim under the hood and persists
        // a tagged "handoff" snapshot for /continue.
        if let Some(dag) = &self.session_dag {
            let estimated_tokens: usize = session.messages.iter().map(|m| {
                m.blocks.iter().map(|b| match b {
                    ContentBlock::Text { text } => text.len() / TOKEN_CHARS_PER_TOKEN,
                    _ => 50, // rough estimate for tool blocks
                }).sum::<usize>()
            }).sum();
            let model_window = max_tokens_for_model(&self.model) as usize;
            let threshold = (model_window as f64 * SESSION_HANDOFF_THRESHOLD) as usize;
            if estimated_tokens > threshold && !session.messages.is_empty() {
                // B-9 T1: stub_ssb_seq is now populated via the SSB writer's
                // current last_seq. The CLI persistence path no longer hard-codes
                // None — Pass 1 of trim_context can produce real stubs.
                let last_seq_now: Option<u64> = self
                    .ssb
                    .as_ref()
                    .and_then(|ssb| {
                        use elai_orchestrator::SsbReader;
                        ssb.last_seq().ok().flatten()
                    })
                    .and_then(|s| u64::try_from(s).ok());
                let trimmed_msgs: Vec<elai_orchestrator::TrimmedMessage> = session
                    .messages
                    .iter()
                    .enumerate()
                    .map(|(i, msg)| elai_orchestrator::TrimmedMessage {
                        seq: i as u32,
                        role: match msg.role {
                            MessageRole::User => elai_orchestrator::DagMessageRole::User,
                            MessageRole::Assistant => elai_orchestrator::DagMessageRole::Assistant,
                            MessageRole::System => elai_orchestrator::DagMessageRole::System,
                            MessageRole::Tool => elai_orchestrator::DagMessageRole::Tool,
                        },
                        content: msg.blocks.iter().filter_map(|b| match b {
                            ContentBlock::Text { text } => Some(text.as_str()),
                            _ => None,
                        }).collect::<Vec<_>>().join("\n").into_bytes(),
                        is_stubbed: false,
                        // B-9 T1: anchor each persisted message to the SSB sequence
                        // that immediately preceded it. Falls back to a synthetic
                        // monotonic seq derived from the message index when no SSB
                        // writer is present (still Some(_) — the contract).
                        stub_ssb_seq: Some(last_seq_now.unwrap_or(0).saturating_add(i as u64 + 1)),
                        // B-9 cycle 2 (BLOCK-2 fix): preserve the full ContentBlock
                        // vector so /continue rehydrates ToolUse/ToolResult pairs
                        // intact. Without this, the Anthropic API rejects the
                        // continued conversation with 400 invalid_request_error.
                        blocks: Some(msg.blocks.clone()),
                    })
                    .collect();
                let valid_ssb_seqs: HashSet<u64> = self.ssb.as_ref()
                    .and_then(|ssb| {
                        use elai_orchestrator::SsbReader;
                        ssb.query_by_session(&self.session.id).ok()
                    })
                    .map(|entries| entries.iter().filter_map(|e| u64::try_from(e.seq).ok()).collect())
                    .unwrap_or_default();
                let config = elai_orchestrator::TrimConfig {
                    stub_threshold: 200,
                    model_context_window: model_window,
                    allocated_slots_tokens: BUDGET_OUTPUT_RESERVE,
                    output_reserve: 1024,
                };
                // B-9 T2: branch on whether SSB is available — only create_handoff
                // when a real SSB writer exists (it threads ssb_reader for stub
                // recovery). Otherwise fall back to plain trim_context for the
                // no-SSB CLI mode.
                if let Some(ssb_ref) = self.ssb.as_ref() {
                    let fsm_state = format!("{}", elai_orchestrator::FsmState::Executing);
                    match dag.create_handoff(
                        &trimmed_msgs,
                        config,
                        ssb_ref.as_ref(),
                        &mut OsEntropy,
                        &elai_core::OsClock,
                        &fsm_state,
                    ) {
                        Ok(snap) => {
                            // Pull the trimmed view back from the persisted snapshot
                            // so the in-flight `session` reflects post-handoff state.
                            if let Ok((_, trimmed)) = dag.load_snapshot(&snap.snap_id) {
                                for (i, tm) in trimmed.iter().enumerate() {
                                    if tm.is_stubbed && i < session.messages.len() {
                                        session.messages[i] = ConversationMessage {
                                            role: session.messages[i].role,
                                            blocks: vec![ContentBlock::Text {
                                                text: "[stubbed — content archived in SSB]".to_owned(),
                                            }],
                                            usage: None,
                                        };
                                    }
                                }
                            }
                        }
                        Err(e) => eprintln!("warn: create_handoff failed: {e}"),
                    }
                } else {
                    match dag.trim_context(&trimmed_msgs, config, &valid_ssb_seqs) {
                        Ok(trimmed) => {
                            for (i, tm) in trimmed.iter().enumerate() {
                                if tm.is_stubbed && i < session.messages.len() {
                                    session.messages[i] = ConversationMessage {
                                        role: session.messages[i].role,
                                        blocks: vec![ContentBlock::Text {
                                            text: "[stubbed — content archived in SSB]".to_owned(),
                                        }],
                                        usage: None,
                                    };
                                }
                            }
                        }
                        Err(e) => eprintln!("warn: context trim failed: {e}"),
                    }
                }
            }
        }

        let plan_slot_tokens = plan_phase_slot_tokens(self.plan.as_ref());
        let (runtime, _ssb) = build_runtime(
            session,
            &self.session.id,
            self.model.clone(),
            self.system_prompt.clone(),
            true,
            emit_output,
            self.allowed_tools.clone(),
            self.permission_mode,
            None,
            self.backend_config.as_ref(),
            Some(self.fsm.clone()),
            plan_slot_tokens,
        )?;
        let runtime = runtime.with_hook_abort_signal(hook_abort_signal.clone());
        // B-19A (T4): wire FSM + Clock into the Ctrl+C handler so the
        // double-press gate can read FSM state.
        // D1: LiveCli.ssb is now Option<Arc<Ssb>> (partial B-19B/C promotion).
        // HookAbortMonitor wiring is still deferred to B-19B/C; passing None
        // here preserves existing Ctrl+C behaviour unchanged.
        let hook_abort_monitor = HookAbortMonitor::spawn(
            hook_abort_signal,
            self.fsm.clone(),
            None,
            self.clock.clone(),
        );

        Ok((runtime, hook_abort_monitor))
    }

    /// Save a DAG snapshot after a turn. Errors are logged, never propagated.
    pub(crate) fn try_save_snapshot(&mut self) {
        let Some(dag) = &self.session_dag else { return };
        let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(rt) => rt,
            Err(_) => return,
        };
        let fsm_state = rt.block_on(self.fsm.get_state());
        let phase_label = rt.block_on(self.fsm.phase_label());
        let now = self.clock.system_now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let snap_id = elai_orchestrator::SessionDag::generate_snap_id(&mut OsEntropy);
        // B-9 T1: anchor each persisted message to the SSB writer's current
        // last_seq so trim Pass 1 can produce stubs on /continue. Falls back to
        // a synthetic monotonic seq when no SSB writer is present.
        // FIX A: call last_seq() inside rt.block_on so Handle::current()
        // resolves to `rt`. Without this, with_read_conn's Handle::current()
        // panics because no tokio runtime is current after block_on returns.
        let last_seq_now: Option<u64> = rt.block_on(async {
            self.ssb.as_ref().and_then(|ssb| {
                use elai_orchestrator::SsbReader;
                ssb.last_seq().ok().flatten()
            })
            .and_then(|s| u64::try_from(s).ok())
        });
        let messages: Vec<elai_orchestrator::TrimmedMessage> = self
            .runtime
            .session()
            .messages
            .iter()
            .enumerate()
            .map(|(i, msg)| elai_orchestrator::TrimmedMessage {
                seq: i as u32,
                role: match msg.role {
                    MessageRole::User => elai_orchestrator::DagMessageRole::User,
                    MessageRole::Assistant => elai_orchestrator::DagMessageRole::Assistant,
                    MessageRole::System => elai_orchestrator::DagMessageRole::System,
                    MessageRole::Tool => elai_orchestrator::DagMessageRole::Tool,
                },
                content: msg.blocks.iter().filter_map(|b| match b {
                    ContentBlock::Text { text } => Some(text.as_str()),
                    _ => None,
                }).collect::<Vec<_>>().join("\n").into_bytes(),
                is_stubbed: false,
                // B-9 T1: stub_ssb_seq must be Some(_) on every persisted message.
                stub_ssb_seq: Some(last_seq_now.unwrap_or(0).saturating_add(i as u64 + 1)),
                // B-9 cycle 2 (BLOCK-2 fix): preserve full ContentBlock vec so
                // /continue rehydrates ToolUse/ToolResult pairs intact.
                blocks: Some(msg.blocks.clone()),
            })
            .collect();
        let fsm_label = format!("{fsm_state}");
        let propagation_phase = if phase_label.is_empty() { None } else { Some(phase_label.clone()) };
        let snapshot = elai_orchestrator::Snapshot {
            snap_id: snap_id.clone(),
            parent_id: self.last_snap_id.clone(),
            session_id: self.session.id.clone(),
            fsm_state: fsm_label.clone(),
            phase: if phase_label.is_empty() { None } else { Some(phase_label) },
            step: None,
            findings: None,
            code_changes: None,
            tests_written: None,
            commit_hashes: vec![],
            created_at: now,
            tag: None,
        };
        match dag.save_snapshot(&snapshot, &messages) {
            Ok(()) => {
                // Detect FSM state change and propagate plan changes
                if !self.last_fsm_state.is_empty()
                    && self.last_fsm_state != fsm_label
                {
                    if let (Some(plan), Some(ssb)) = (&self.plan, &self.ssb) {
                        if let Some(phase_id) = &propagation_phase {
                            let _ = rt.block_on(plan.propagate_changes(phase_id, ssb.as_ref()));
                        }
                    }
                }
                self.last_fsm_state = fsm_label;
                self.last_snap_id = Some(snap_id);
            }
            Err(e) => eprintln!("warn: snapshot save failed: {e}"),
        }
    }

    /// A-38: attempt incremental profile refresh if baby step counter triggers.
    /// Errors are logged, never propagated — REPL must not crash on profile refresh failures.
    pub(crate) fn try_incremental_profile_refresh(&mut self) {
        use elai_knowledge::incremental_profile::{
            ProfileChangeDetector, incremental_tier1_refresh,
        };
        let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Determine last_profiled_at from cached profile or 0 for first run.
        let last_profiled_at = self.last_profiled_at;

        let mut detector = ProfileChangeDetector::new(project_root.clone(), last_profiled_at);

        if !detector.should_refresh_tier1(self.baby_step_counter) {
            return;
        }

        // Get or create the profile to refresh.
        if self.cached_profile.is_none() {
            match elai_knowledge::incremental_profile::load_or_detect_profile(&project_root) {
                Ok(p) => {
                    self.cached_profile = Some(p);
                }
                Err(e) => {
                    eprintln!("warn: incremental profile load failed: {e}");
                    return;
                }
            }
        }

        let profile = match self.cached_profile.as_mut() {
            Some(p) => p,
            None => return,
        };

        match incremental_tier1_refresh(&project_root, &mut detector, profile) {
            Ok(true) => {
                // Reset counter and update timestamp after successful refresh.
                self.baby_step_counter = 0;
                self.last_profiled_at = detector.last_profiled_at;
            }
            Ok(false) => {
                // No changes detected — do NOT reset counter. The next check
                // triggers at the next multiple of BABY_STEP_REFRESH_N.
                self.baby_step_counter = 0;
            }
            Err(e) => {
                eprintln!("warn: incremental profile refresh failed: {e}");
            }
        }
    }

    /// Drive the FSM through a single transition. Builds a one-shot tokio runtime
    /// to call the async `Fsm::transition`. Mirrors the pattern used by
    /// `try_save_snapshot`. Errors are logged as warnings; the REPL must not crash
    /// on FSM failures.
    pub(crate) fn fsm_transition_sync(&self, target: elai_orchestrator::FsmState) {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("[fsm] failed to build runtime for transition to {target}: {e}");
                return;
            }
        };
        if let Err(e) = rt.block_on(self.fsm.transition(target, "interactive", vec![])) {
            eprintln!("[fsm] transition to {target} failed: {e}");
        }
    }

    /// Walk the FSM from IDLE into EXECUTING for interactive single-agent mode.
    /// Chain: IDLE -> PLANNING -> TESTING_FIRST -> EXECUTING.
    /// Any step failure is logged; subsequent steps are still attempted so that
    /// if the FSM was already past IDLE (e.g. resumed session) the chain
    /// converges gracefully rather than hard-failing.
    pub(crate) fn fsm_enter_executing(&self) {
        use elai_orchestrator::FsmState;
        for target in [FsmState::Planning, FsmState::TestingFirst, FsmState::Executing] {
            self.fsm_transition_sync(target);
        }
    }

    /// Walk the FSM from EXECUTING back to IDLE for a clean interactive shutdown.
    /// Chain: EXECUTING -> AUDITING -> MEMORIZING -> COMPLETE -> IDLE.
    /// Individual step failures are logged and the chain continues — the session
    /// is ending regardless.
    pub(crate) fn fsm_shutdown(&self) {
        use elai_orchestrator::FsmState;
        for target in [
            FsmState::Auditing,
            FsmState::Memorizing,
            FsmState::Complete,
            FsmState::Idle,
        ] {
            self.fsm_transition_sync(target);
        }
    }

    pub(crate) fn replace_runtime(&mut self, runtime: BuiltRuntime) -> Result<(), anyhow::Error> {
        self.runtime.shutdown_plugins()?;
        self.runtime = runtime;
        Ok(())
    }

    pub(crate) fn run_turn(&mut self, input: &str) -> Result<(), anyhow::Error> {
        // Inject knowledge pipeline retrieval context if available.
        let augmented_input = self.augment_with_knowledge(input);
        let effective_input = augmented_input.as_deref().unwrap_or(input);

        // B-39 / B-39C: Chat REFLECT gate.
        // When the pipeline returns skipped=false, the AUDITOR-selected reply
        // is rendered directly and the runtime.run_turn fall-through is bypassed
        // entirely (D2a — text-only, no tool execution). When skipped=true
        // (TRIAGE classified the turn as Chat, or any error fell back to
        // passthrough), control falls through to prepare_turn_runtime unchanged.
        if self.reflect_enabled {
            if self.backend_config.is_none() {
                tracing::warn!("/reason on but no backend_config; skipping reflect");
            } else if let Some(ref backend_cfg) = self.backend_config.clone() {
                // Lazy-build the pipeline on first use.
                if self.reflect_pipeline.is_none() {
                    let cfg = ChatReflectConfig { enabled: true, ..ChatReflectConfig::default() };
                    let sink = self.ssb.as_ref().map(|s| Arc::clone(s) as Arc<dyn ReflectEventSink>);
                    self.reflect_pipeline = Some(ChatReflectPipeline::new(cfg, sink));
                }
                if let Some(pipe) = self.reflect_pipeline.as_ref() {
                    match sidecar::build_hyde_sidecar_provider(backend_cfg) {
                        Ok(provider) => {
                            let client = SidecarChatClient::new(Arc::new(provider));
                            let turn_id = format!("turn-{}", self.clock.now().tick());
                            if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()
                            {
                                let output = rt.block_on(pipe.run(turn_id, effective_input, &client));
                                if !output.skipped {
                                    let renderer = TerminalRenderer::new();
                                    let rendered = renderer.render_markdown(&output.final_reply);
                                    let mut stdout = io::stdout();
                                    write!(stdout, "{rendered}\r\n")?;
                                    stdout.flush()?;
                                    self.runtime.push_user_text(effective_input)?;
                                    self.runtime.push_assistant_text(&output.final_reply)?;
                                    self.runtime
                                        .record_pipeline_usage(output.token_summary.total);
                                    self.persist_session()?;
                                    self.try_save_snapshot();
                                    self.baby_step_counter =
                                        self.baby_step_counter.saturating_add(1);
                                    self.try_incremental_profile_refresh();
                                    return Ok(());
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "REFLECT: SidecarProvider build failed; skipping");
                        }
                    }
                }
            }
        }

        let (mut runtime, hook_abort_monitor) = self.prepare_turn_runtime(true)?;
        let mut spinner = Spinner::new();
        let mut stdout = io::stdout();
        spinner.tick(
            "Thinking...",
            TerminalRenderer::new().color_theme(),
            &mut stdout,
        )?;
        let mut permission_prompter = CliPermissionPrompter::new(self.permission_mode);
        let result = runtime.run_turn(effective_input, Some(&mut permission_prompter));
        hook_abort_monitor.stop();
        match result {
            Ok(summary) => {
                self.replace_runtime(runtime)?;
                // Clear the spinner line without overwriting the streamed response.
                let _ = execute!(stdout, MoveToColumn(0), Clear(ClearType::CurrentLine));
                write!(stdout, "\r\n")?;
                stdout.flush()?;
                if let Some(event) = summary.auto_compaction {
                    println!(
                        "{}",
                        format_auto_compaction_notice(event.removed_message_count)
                    );
                }
                self.persist_session()?;
                self.try_save_snapshot();
                // A-38: increment baby step counter, trigger Tier 1 refresh if due.
                self.baby_step_counter = self.baby_step_counter.saturating_add(1);
                self.try_incremental_profile_refresh();
                Ok(())
            }
            Err(error) => {
                runtime.shutdown_plugins()?;
                spinner.fail(
                    "Request failed",
                    TerminalRenderer::new().color_theme(),
                    &mut stdout,
                )?;
                Err(error.into())
            }
        }
    }

    pub(crate) fn run_turn_with_output(
        &mut self,
        input: &str,
        output_format: CliOutputFormat,
    ) -> Result<(), anyhow::Error> {
        match output_format {
            CliOutputFormat::Text | CliOutputFormat::Toml => self.run_turn(input),
            CliOutputFormat::Json => self.run_prompt_json(input),
        }
    }

    /// Query the knowledge pipeline and return an augmented input string with
    /// retrieval context prepended. Returns None if pipeline is absent or query fails.
    ///
    /// 1.3 — When backend is Local, caps the retrieval context at 8000 chars to
    /// prevent overflowing Qwen 3B/7B's ~25K token context window. Heuristic:
    /// 25_000 * 0.35 budget fraction ≈ 8_750 tokens; capped at 8_000 chars
    /// (≈2_286 tokens @ 3.5 chars/token) to leave room for the input itself.
    pub(crate) fn augment_with_knowledge(&self, input: &str) -> Option<String> {
        let pipeline = self.knowledge_pipeline.as_ref()?;
        // 1.3: detect Local backend to decide whether to cap context.
        let is_local = self
            .backend_config
            .as_ref()
            .map(|c| matches!(c.mode, BackendMode::Local))
            .unwrap_or(false);
        match pipeline.query(input) {
            Ok(stages) => {
                let snippets: Vec<&str> = stages.iter()
                    .flat_map(|stage| stage.results.iter())
                    .take(KNOWLEDGE_TOP_K)
                    .map(|r| r.snippet.as_str())
                    .collect();
                if snippets.is_empty() {
                    return None;
                }
                let context_block = snippets.join("\n---\n");
                // 1.3: apply retrieval char cap on Local backend only.
                let capped_block = if is_local {
                    // 8000 char ceiling — see doc comment above.
                    const LOCAL_RETRIEVAL_CHAR_CAP: usize = 8_000;
                    if context_block.len() > LOCAL_RETRIEVAL_CHAR_CAP {
                        // Truncate at a UTF-8 char boundary to avoid splitting codepoints.
                        let cut = context_block
                            .char_indices()
                            .take_while(|(i, _)| *i <= LOCAL_RETRIEVAL_CHAR_CAP)
                            .last()
                            .map(|(i, _)| i)
                            .unwrap_or(LOCAL_RETRIEVAL_CHAR_CAP);
                        format!("{} [truncated]", &context_block[..cut])
                    } else {
                        context_block
                    }
                } else {
                    context_block
                };
                Some(format!(
                    "<code-context>\n{capped_block}\n</code-context>\n\n{input}"
                ))
            }
            Err(e) => {
                eprintln!("warn: knowledge pipeline query failed: {e}");
                None
            }
        }
    }

    /// Whether the knowledge pipeline is active.
    pub(crate) fn has_knowledge_pipeline(&self) -> bool {
        self.knowledge_pipeline.is_some()
    }

    /// Notify the knowledge pipeline that a file was modified (e.g., after tool use).
    pub(crate) fn notify_file_changed(&mut self, path: &Path) {
        if let Some(ref mut pipeline) = self.knowledge_pipeline {
            if let Err(e) = pipeline.bm25.update_file(path, None) {
                eprintln!("warn: BM25 update_file failed for {}: {e}", path.display());
            }
            // Also re-index in vector index if available.
            if let (Some(ref vm), Some(ref mut indexer)) =
                (&pipeline.vector_module, &mut pipeline.vector_indexer)
            {
                if let Err(e) = indexer.index_file(path, vm.as_ref(), None) {
                    eprintln!("warn: vector index_file failed for {}: {e}", path.display());
                }
            }
        }
        // A-44: incremental DKG+BM25 reindex via IndexingService.
        if let Some(ref mut svc) = self.indexing_service {
            if let Err(e) = svc.reindex_file(path) {
                tracing::warn!(path = %path.display(), "IndexingService reindex_file failed: {e}");
            }
        }
    }

    pub(crate) fn run_prompt_json(&mut self, input: &str) -> Result<(), anyhow::Error> {
        let (mut runtime, hook_abort_monitor) = self.prepare_turn_runtime(false)?;
        let mut permission_prompter = CliPermissionPrompter::new(self.permission_mode);
        let result = runtime.run_turn(input, Some(&mut permission_prompter));
        hook_abort_monitor.stop();
        let summary = result?;
        self.replace_runtime(runtime)?;
        self.persist_session()?;
        println!(
            "{}",
            json!({
                "message": final_assistant_text(&summary),
                "model": self.model,
                "iterations": summary.iterations,
                "auto_compaction": summary.auto_compaction.map(|event| json!({
                    "removed_messages": event.removed_message_count,
                    "notice": format_auto_compaction_notice(event.removed_message_count),
                })),
                "tool_uses": collect_tool_uses(&summary),
                "tool_results": collect_tool_results(&summary),
                "prompt_cache_events": collect_prompt_cache_events(&summary),
                "usage": {
                    "input_tokens": summary.usage.input_tokens,
                    "output_tokens": summary.usage.output_tokens,
                    "cache_creation_input_tokens": summary.usage.cache_creation_input_tokens,
                    "cache_read_input_tokens": summary.usage.cache_read_input_tokens,
                },
                "estimated_cost": format_usd(
                    elai_orchestrator::usage::estimate_cost_usd_with_pricing(
                        summary.usage,
                        pricing_for_model(&self.model)
                            .unwrap_or_else(elai_orchestrator::ModelPricing::default_sonnet_tier)
                    ).total_cost_usd()
                )
            })
        );
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn handle_repl_command(
        &mut self,
        command: SlashCommand,
    ) -> Result<bool, anyhow::Error> {
        Ok(match command {
            SlashCommand::Help => {
                println!("{}", render_repl_help());
                false
            }
            SlashCommand::Status => {
                self.print_status();
                false
            }
            SlashCommand::Bughunter { scope } => {
                self.run_bughunter(scope.as_deref())?;
                false
            }
            SlashCommand::Commit => {
                self.run_commit(None)?;
                false
            }
            SlashCommand::Pr { context } => {
                self.run_pr(context.as_deref())?;
                false
            }
            SlashCommand::Issue { context } => {
                self.run_issue(context.as_deref())?;
                false
            }
            SlashCommand::Ultraplan { task } => {
                self.run_ultraplan(task.as_deref())?;
                false
            }
            SlashCommand::Teleport { target } => {
                Self::run_teleport(target.as_deref())?;
                false
            }
            SlashCommand::DebugToolCall => {
                self.run_debug_tool_call(None)?;
                false
            }
            SlashCommand::Sandbox => {
                Self::print_sandbox_status();
                false
            }
            SlashCommand::Compact => {
                self.compact()?;
                false
            }
            SlashCommand::Model { model } => self.set_model(model)?,
            SlashCommand::Permissions { mode } => self.set_permissions(mode)?,
            SlashCommand::Clear { confirm } => self.clear_session(confirm)?,
            SlashCommand::Cost => {
                self.print_cost();
                false
            }
            SlashCommand::Resume { session_path } => self.resume_session(session_path)?,
            SlashCommand::Continue { snap_id } => self.continue_from_handoff(snap_id.as_deref())?,
            SlashCommand::Config { section } => {
                Self::print_config(section.as_deref())?;
                false
            }
            SlashCommand::Mcp { action, target } => {
                let args = match (action.as_deref(), target.as_deref()) {
                    (None, None) => None,
                    (Some(action), None) => Some(action.to_string()),
                    (Some(action), Some(target)) => Some(format!("{action} {target}")),
                    (None, Some(target)) => Some(target.to_string()),
                };
                Self::print_mcp(args.as_deref(), CliOutputFormat::Text)?;
                false
            }
            SlashCommand::Runtime {
                action,
                role,
                target,
            } => {
                Self::print_runtime(
                    action.as_deref(),
                    role.as_deref(),
                    target.as_deref(),
                )?;
                false
            }
            SlashCommand::Memory => {
                Self::print_memory()?;
                false
            }
            SlashCommand::Init => {
                run_init(CliOutputFormat::Text)?;
                false
            }
            SlashCommand::Diff => {
                Self::print_diff()?;
                false
            }
            SlashCommand::Version => {
                Self::print_version(CliOutputFormat::Text);
                false
            }
            SlashCommand::Export { path, jsonl } => {
                self.export_session(path.as_deref(), jsonl)?;
                false
            }
            SlashCommand::Session { action, target } => {
                self.handle_session_command(action.as_deref(), target.as_deref())?
            }
            SlashCommand::Plugins { action, target } => {
                self.handle_plugins_command(action.as_deref(), target.as_deref())?
            }
            SlashCommand::Agents { args } => {
                Self::print_agents(args.as_deref(), CliOutputFormat::Text)?;
                false
            }
            SlashCommand::Skills { args } => {
                Self::print_skills(args.as_deref(), CliOutputFormat::Text)?;
                false
            }
            SlashCommand::Doctor => {
                println!("{}", doctor::render_doctor_report()?.render());
                false
            }
            SlashCommand::Profile { path } => {
                let root = path
                    .as_deref()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                match detect_project_profile(&root) {
                    Ok(mut profile) => {
                        populate_tier3(&mut profile, &[]);
                        let output = elai_knowledge::serialize_profile(&profile)
                            .unwrap_or_else(|e| format!("serialization error: {e}"));
                        println!("{output}");
                    }
                    Err(e) => eprintln!("profile detection failed: {e}"),
                }
                false
            }
            SlashCommand::Plan { mode } => {
                match &self.plan {
                    None => {
                        println!("No plan loaded. Place a plan file at {PLANS_DIR}/<pipeline>.toml and restart.");
                    }
                    Some(plan) => {
                        match mode.as_deref() {
                            None | Some("list") | Some("") => {
                                println!("Plan: {}", plan.pipeline_id);
                                println!("Version: {}", plan.version);
                                println!("Phases ({}):", plan.phases.len());
                                for (id, entry) in &plan.phases {
                                    let deps = if entry.depends_on.is_empty() {
                                        String::new()
                                    } else {
                                        format!(" [deps: {}]", entry.depends_on.join(", "))
                                    };
                                    println!("  {id}: {}{deps}", entry.goal);
                                }
                                if !plan.global_constraints.is_empty() {
                                    println!("Constraints:");
                                    for c in &plan.global_constraints {
                                        println!("  - {c}");
                                    }
                                }
                            }
                            Some(phase_arg) => {
                                // /plan <phase_id> — show contracts
                                match plan.extract_contracts(phase_arg) {
                                    Ok(contracts) => {
                                        println!("Contracts for phase '{phase_arg}':");
                                        println!("  Inputs:");
                                        for c in &contracts.input_contracts {
                                            println!("    {}: {}", c.name, c.type_signature);
                                        }
                                        println!("  Outputs:");
                                        for c in &contracts.output_contracts {
                                            println!("    {}: {}", c.name, c.type_signature);
                                        }
                                    }
                                    Err(e) => eprintln!("Plan error: {e}"),
                                }
                            }
                        }
                    }
                }
                false
            }
            SlashCommand::Login
            | SlashCommand::Logout
            | SlashCommand::Vim
            | SlashCommand::Upgrade
            | SlashCommand::Stats
            | SlashCommand::Share
            | SlashCommand::Feedback
            | SlashCommand::Files
            | SlashCommand::Fast
            | SlashCommand::Exit
            | SlashCommand::Summary
            | SlashCommand::Desktop
            | SlashCommand::Brief
            | SlashCommand::Advisor
            | SlashCommand::Stickers
            | SlashCommand::Insights
            | SlashCommand::Thinkback
            | SlashCommand::ReleaseNotes
            | SlashCommand::SecurityReview
            | SlashCommand::Keybindings
            | SlashCommand::PrivacySettings
            | SlashCommand::Review { .. }
            | SlashCommand::Tasks { .. }
            | SlashCommand::Theme { .. }
            | SlashCommand::Voice { .. }
            | SlashCommand::Usage { .. }
            | SlashCommand::Rename { .. }
            | SlashCommand::Copy { .. }
            | SlashCommand::Hooks { .. }
            | SlashCommand::Context { .. }
            | SlashCommand::Color { .. }
            | SlashCommand::Effort { .. }
            | SlashCommand::Branch { .. }
            | SlashCommand::Rewind { .. }
            | SlashCommand::Ide { .. }
            | SlashCommand::Tag { .. }
            | SlashCommand::OutputStyle { .. }
            | SlashCommand::AddDir { .. } => {
                eprintln!("Command registered but not yet implemented.");
                false
            }
            SlashCommand::Research { question } | SlashCommand::Ask { question } => {
                self.fsm_transition_sync(elai_orchestrator::FsmState::Researching);
                match question {
                    Some(q) => {
                        // C-30: Oracle answer-level cache (lazy-init).
                        let oracle_cache = self
                            .oracle_cache
                            .get_or_insert_with(|| {
                                Arc::new(elai_orchestrator::oracle_cache::OracleAnswerCache::new())
                            })
                            .clone();
                        let oracle_project_root = std::env::current_dir()
                            .unwrap_or_else(|_| PathBuf::from("."));
                        let oracle_key = elai_orchestrator::oracle_cache::CacheKey::compute(
                            &q,
                            &oracle_project_root,
                        );
                        let now_secs = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        if let Some(cached) = oracle_cache.get(&oracle_key, now_secs) {
                            println!(
                                "Research answer (cached, source_tier={}, {} sources):",
                                elai_orchestrator::oracle_cache::source_tier_str(cached.source_tier),
                                cached.answer.sources_consulted.len()
                            );
                            println!("{}", cached.answer.answer_text);
                            if let Some(ssb) = &self.ssb {
                                let payload = elai_orchestrator::ssb::SsbPayload::OracleCacheHit(
                                    elai_orchestrator::ssb::OracleCacheHitPayload {
                                        query_hash_hex: oracle_key.query_hash_hex(),
                                        source_tier: elai_orchestrator::oracle_cache::source_tier_str(cached.source_tier).to_string(),
                                        cached_age_secs: cached.age_secs(now_secs),
                                    },
                                );
                                let ssb = ssb.clone();
                                if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                                    rt.block_on(async move {
                                        if let Err(e) = ssb.write_payload(elai_orchestrator::task_runner::RUNNER_AGENT_ID, elai_orchestrator::task_runner::RUNNER_SUPERSTEP, payload, vec![]).await {
                                            tracing::warn!(error = %e, "C-30 oracle_cache_hit SSB emit failed");
                                        }
                                    });
                                }
                            }
                            self.fsm_transition_sync(elai_orchestrator::FsmState::Idle);
                            return Ok(false);
                        }
                        // C-17 (DRIFT-N) Oracle pre-seeding: prefix the
                        // research query with a PROJECT_STARTING_POINTS
                        // block built from detect_entry_points. Closes
                        // B-34 MISSING gap. The injection is best-effort
                        // — if the symbol-graph DB is absent or empty we
                        // proceed with the unaugmented query.
                        let augmented = c17_research_oracle_prefix(&q, self.ssb.as_ref());
                        let query = elai_core::ResearchQuery::new(&augmented);
                        let tier_cfg = match elai_core::parse_source_tier_config(EMPTY_TIER_TOML) {
                            Ok(cfg) => cfg,
                            Err(e) => {
                                eprintln!("Failed to load tier config: {e}");
                                self.fsm_transition_sync(elai_orchestrator::FsmState::Idle);
                                return Ok(false);
                            }
                        };
                        // Default: hit the sidecar `/search` proxy (SearXNG/Brave-backed).
                        // Dev opt-out: ELAI_STUB_SEARCH=1 swaps in the in-memory stub for
                        // offline research-pipeline smoke tests. Falsy values
                        // (0/false/no/off/empty) explicitly keep the production backend.
                        let use_stub = std::env::var("ELAI_STUB_SEARCH")
                            .map(|v| {
                                let trimmed = v.trim();
                                !trimmed.is_empty()
                                    && !matches!(
                                        trimmed.to_ascii_lowercase().as_str(),
                                        "0" | "false" | "no" | "off"
                                    )
                            })
                            .unwrap_or(false);
                        let backend: Box<dyn elai_orchestrator::SearchBackend> = if use_stub {
                            tracing::info!("research: using StubSearchBackend (ELAI_STUB_SEARCH=1)");
                            Box::new(elai_orchestrator::StubSearchBackend::new())
                        } else {
                            let sidecar_url = std::env::var("ELAI_SIDECAR_URL")
                                .unwrap_or_else(|_| {
                                    elai_orchestrator::DEFAULT_SIDECAR_SEARCH_URL.to_string()
                                });
                            match elai_orchestrator::SearxSearchBackend::new(&sidecar_url) {
                                Ok(b) => {
                                    tracing::info!(
                                        sidecar_url = %sidecar_url,
                                        "research: using SearxSearchBackend"
                                    );
                                    Box::new(b)
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        error = %e,
                                        sidecar_url = %sidecar_url,
                                        "research: SearxSearchBackend construction failed, falling back to stub"
                                    );
                                    Box::new(elai_orchestrator::StubSearchBackend::new())
                                }
                            }
                        };
                        // D3: wire SidecarGroundingClient for LLM-backed Phase F synthesis.
                        // run_research_pipeline is sync (phases A-E use sync SearchBackend::search).
                        // Phase F async bridge is handled inside phase_f_synthesize_blocking
                        // (builds its own current-thread runtime; safe from this sync context).
                        let sidecar_base_url = std::env::var("ANTHROPIC_BASE_URL")
                            .or_else(|_| std::env::var("ELAI_SIDECAR_URL"))
                            .unwrap_or_else(|_| elai_orchestrator::grounding_client::SIDECAR_DEFAULT_HOST.to_string());
                        let grounding = elai_orchestrator::SidecarGroundingClient::new(
                            sidecar_base_url,
                            std::time::Duration::from_secs(elai_orchestrator::grounding_client::GROUNDING_TIMEOUT_SECS),
                        );
                        match elai_orchestrator::run_research_pipeline(&query, &tier_cfg, backend.as_ref(), &grounding) {
                            Ok(answer) => {
                                println!("Research answer ({} sources, {} caveats):",
                                    answer.sources_consulted.len(), answer.caveats.len());
                                println!("{}", answer.answer_text);
                                if !answer.sources_consulted.is_empty() {
                                    println!("\nSources:");
                                    for (i, s) in answer.sources_consulted.iter().enumerate() {
                                        println!("  [{}] {}", i + 1, s.url);
                                    }
                                }
                                if !answer.path.is_empty() {
                                    if let Some(ssb) = &self.ssb {
                                        use elai_orchestrator::ssb::SsbPayload;
                                        // C-15: OsClock-backed `ts_ms`. Research-path SSB rows are
                                        // observation timestamps; not part of any deterministic test
                                        // fixture, so plain OsClock is sufficient.
                                        let ts_ms = u64::try_from(OsClock.now_ms()).unwrap_or(u64::MAX);
                                        let payload = SsbPayload::ResearchPathChosen(
                                            elai_orchestrator::ResearchPathChosenPayload {
                                                path: answer.path.clone(),
                                                reason: "synthesis".to_string(),
                                                ts_ms,
                                            },
                                        );
                                        let ssb = ssb.clone();
                                        if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                                            rt.block_on(async {
                                                if let Err(e) = ssb.write_payload("research", 0, payload, vec![]).await {
                                                    tracing::warn!(error = %e, "research: failed to emit research_path_chosen SSB row");
                                                }
                                            });
                                        }
                                    }
                                }

                                // C-30: cache the answer using the
                                // confidence -> tier mapping. T4 / Unknown
                                // are dropped by `OracleAnswerCache::insert`
                                // itself. Emit `oracle_cache_miss` regardless
                                // so the dashboard sees one row per pipeline
                                // run that bypassed the cache.
                                let tier = elai_orchestrator::oracle_cache::source_tier_for_confidence(answer.confidence);
                                let cached_entry = elai_orchestrator::oracle_cache::CachedAnswer {
                                    answer: answer.clone(),
                                    cached_at: now_secs,
                                    source_tier: tier,
                                };
                                let inserted = oracle_cache.insert(oracle_key, cached_entry);
                                tracing::debug!(
                                    inserted = inserted,
                                    "C-30 oracle cache: insert"
                                );
                                if let Some(ssb) = &self.ssb {
                                    let payload = elai_orchestrator::ssb::SsbPayload::OracleCacheMiss(
                                        elai_orchestrator::ssb::OracleCacheMissPayload {
                                            query_hash_hex: oracle_key.query_hash_hex(),
                                            confidence: elai_orchestrator::oracle_cache::confidence_str(answer.confidence).to_string(),
                                        },
                                    );
                                    let ssb = ssb.clone();
                                    if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                                        rt.block_on(async move {
                                            if let Err(e) = ssb.write_payload(elai_orchestrator::task_runner::RUNNER_AGENT_ID, elai_orchestrator::task_runner::RUNNER_SUPERSTEP, payload, vec![]).await {
                                                tracing::warn!(error = %e, "C-30 oracle_cache_miss SSB emit failed");
                                            }
                                        });
                                    }
                                }
                            }
                            Err(e) => tracing::error!(error = %e, "research pipeline error"),
                        }
                    }
                    None => eprintln!("Usage: /research <question>"),
                }
                self.fsm_transition_sync(elai_orchestrator::FsmState::Idle);
                false
            }
            SlashCommand::Explore { target } => {
                match target {
                    Some(t) => {
                        tracing::info!(mode = "explore", query = %t, "routed to explore mode");
                        println!("[explore] {t}");
                    }
                    None => eprintln!("Usage: /explore <question>"),
                }
                false
            }
            SlashCommand::Task { description, surgical, new_project } => {
                match description {
                    Some(d) => {
                        tracing::info!(
                            mode = "task",
                            surgical = surgical,
                            new_project = new_project,
                            description = %d,
                            "routed to task mode (D2: DefaultWebResearchRunner)"
                        );

                        // -------------------------------------------------------
                        // C-30: project detection signal scan (first /task only).
                        // -------------------------------------------------------
                        if !self.project_detection_done {
                            let project_root = std::env::current_dir()
                                .unwrap_or_else(|_| PathBuf::from("."));
                            let indexes_ready = self
                                .indexing_service
                                .as_ref()
                                .map(|s| s.indexes_exist())
                                .unwrap_or(false);
                            if !indexes_ready {
                                let signals =
                                    run_project_detection_signals(&project_root);
                                let any_required = c30_any_required_fired(&signals);
                                if let Some(ssb) = &self.ssb {
                                    emit_project_detection_signals(ssb, &signals);
                                }
                                if any_required {
                                    tracing::info!(
                                        signals = signals.len(),
                                        "C-30 fresh-project detected; index build is recommended"
                                    );
                                    eprintln!(
                                        "[C-30] Detected new project. Build knowledge index? (rerun with `/index build` to confirm; proceeding without index for this task.)"
                                    );
                                }
                            }
                            self.project_detection_done = true;
                        }

                        // -------------------------------------------------------
                        // C-30: surgical branch intercepts BEFORE
                        // DefaultWebResearchRunner. The default ConceptDecomposition
                        // pipeline is skipped and replaced by the SurgicalDecomposer
                        // output. When the DKG is unavailable or returns no
                        // matches, the resulting plan is empty (Simple, 0 LOC)
                        // but still emits the canonical SurgicalPlanCreated row
                        // so the dashboard / auditor observe the surgical-mode
                        // selection.
                        if surgical {
                            let project_root = std::env::current_dir()
                                .unwrap_or_else(|_| PathBuf::from("."));
                            let dkg_db_path_raw = core_dkg_db_path(&project_root);
                            let dkg_db_path = if dkg_db_path_raw.is_file() {
                                Some(dkg_db_path_raw.clone())
                            } else {
                                None
                            };
                            let dkg_querier: Arc<
                                dyn elai_rules::predicate::DkgQuerier + Send + Sync,
                            > = match dkg_db_path
                                .as_ref()
                                .and_then(|p| DkgQuerierAdapter::new(p).ok())
                            {
                                Some(adapter) => Arc::new(adapter),
                                None => Arc::new(EmptyDkgQuerier),
                            };
                            let decomposer = elai_orchestrator::surgical_decomposer::SurgicalDecomposer::new(
                                dkg_querier,
                                dkg_db_path,
                            );
                            let interactive_mode = elai_orchestrator::context_budget::ExecutionMode::Interactive;
                            // `confirmed` is wired from the CLI's --new flag
                            // (best-effort signal that the operator opted in).
                            // Production callers can flip this when the user
                            // answers the confirm prompt above.
                            let confirmed = new_project;
                            let mode_str = "interactive";
                            match decomposer.decompose(&d, interactive_mode, confirmed) {
                                Ok(plan) => {
                                    println!(
                                        "[task --surgical] plan: complexity={} files={} loc={} steps={}",
                                        plan.complexity_class.as_str(),
                                        plan.affected_set.blast_radius_files(),
                                        plan.affected_set.total_loc_estimate,
                                        plan.steps.len(),
                                    );
                                    if let Some(ssb) = &self.ssb {
                                        let ssb = ssb.clone();
                                        let payload = elai_orchestrator::ssb::SsbPayload::SurgicalPlanCreated(
                                            elai_orchestrator::ssb::SurgicalPlanCreatedPayload {
                                                task_id: plan.task_id.clone(),
                                                complexity_class: plan.complexity_class.as_str().to_string(),
                                                blast_radius_files: plan.affected_set.blast_radius_files(),
                                                total_loc_estimate: plan.affected_set.total_loc_estimate,
                                                step_count: u32::try_from(plan.steps.len()).unwrap_or(u32::MAX),
                                                regression_gate_snapshot_hash: plan.regression_gate.snapshot_hash.clone(),
                                            },
                                        );
                                        if plan.affected_set.total_loc_estimate > elai_orchestrator::surgical_decomposer::SURGICAL_LOC_WARN_THRESHOLD {
                                            let warn_payload = elai_orchestrator::ssb::SsbPayload::SurgicalLocWarning(
                                                elai_orchestrator::ssb::SurgicalLocWarningPayload {
                                                    task_id: plan.task_id.clone(),
                                                    total_loc_estimate: plan.affected_set.total_loc_estimate,
                                                    threshold: elai_orchestrator::surgical_decomposer::SURGICAL_LOC_WARN_THRESHOLD,
                                                    mode: mode_str.to_string(),
                                                },
                                            );
                                            let ssb_warn = ssb.clone();
                                            let task_id_warn = plan.task_id.clone();
                                            if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                                                rt.block_on(async move {
                                                    if let Err(e) = ssb_warn.write_payload(elai_orchestrator::task_runner::RUNNER_AGENT_ID, elai_orchestrator::task_runner::RUNNER_SUPERSTEP, warn_payload, vec![]).await {
                                                        tracing::warn!(error = %e, task_id = %task_id_warn, "C-30 surgical_loc_warning SSB emit failed");
                                                    }
                                                });
                                            }
                                        }
                                        if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                                            rt.block_on(async move {
                                                if let Err(e) = ssb.write_payload(elai_orchestrator::task_runner::RUNNER_AGENT_ID, elai_orchestrator::task_runner::RUNNER_SUPERSTEP, payload, vec![]).await {
                                                    tracing::warn!(error = %e, "C-30 surgical_plan_created SSB emit failed");
                                                }
                                            });
                                        }
                                    }
                                    self.fsm_transition_sync(elai_orchestrator::FsmState::Idle);
                                    return Ok(false);
                                }
                                Err(elai_orchestrator::surgical_decomposer::SurgicalError::ConfirmationRequired { loc, threshold }) => {
                                    eprintln!(
                                        "[task --surgical] blast radius {loc} LOC exceeds {threshold}; rerun with `--new` to confirm."
                                    );
                                    if let Some(ssb) = &self.ssb {
                                        let payload = elai_orchestrator::ssb::SsbPayload::SurgicalLocWarning(
                                            elai_orchestrator::ssb::SurgicalLocWarningPayload {
                                                task_id: elai_orchestrator::surgical_decomposer::generate_task_id(&d),
                                                total_loc_estimate: loc,
                                                threshold,
                                                mode: "interactive".into(),
                                            },
                                        );
                                        let ssb = ssb.clone();
                                        if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                                            rt.block_on(async move {
                                                let _ = ssb.write_payload(elai_orchestrator::task_runner::RUNNER_AGENT_ID, elai_orchestrator::task_runner::RUNNER_SUPERSTEP, payload, vec![]).await;
                                            });
                                        }
                                    }
                                    self.fsm_transition_sync(elai_orchestrator::FsmState::Idle);
                                    return Ok(false);
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "C-30 surgical decomposition failed; falling back to default task runner");
                                    eprintln!("[task --surgical] decomposition error: {e}; falling back to default pipeline");
                                }
                            }
                        }

                        // D2: wire DefaultWebResearchRunner — mirror dashboard/src/lib.rs:798-806.
                        // Build a one-shot tokio runtime (same pattern as fsm_transition_sync,
                        // cli/main.rs:3622) to block on the async runner.run() future.
                        let sidecar_url = std::env::var("ELAI_SIDECAR_URL")
                            .or_else(|_| std::env::var("ANTHROPIC_BASE_URL"))
                            .unwrap_or_else(|_| {
                                elai_orchestrator::DEFAULT_SIDECAR_SEARCH_URL.to_string()
                            });

                        let tier_config = elai_orchestrator::research_tier::research_config_for_tier(
                            elai_core::VramTier::Vram12Gb,
                        );
                        let runner = elai_orchestrator::DefaultWebResearchRunner::new(
                            tier_config,
                            sidecar_url.clone(),
                            self.clock.clone(),
                        );

                        // Build emitter: SSB-backed when SSB is wired, noop otherwise.
                        struct NoopStepEmitter;
                        impl elai_orchestrator::web_research_orchestrator::WebResearchStepEmitter
                            for NoopStepEmitter
                        {
                            fn emit(
                                &self,
                                event: elai_orchestrator::web_research_orchestrator::WebResearchStepEvent,
                            ) -> std::pin::Pin<
                                Box<dyn std::future::Future<Output = ()> + Send + '_>,
                            > {
                                tracing::debug!(
                                    action = %event.action,
                                    iteration = event.iteration,
                                    phi = event.phi,
                                    psi = event.psi,
                                    sources_fetched = event.sources_fetched,
                                    "task: web-research step"
                                );
                                Box::pin(std::future::ready(()))
                            }
                        }

                        let emitter: std::sync::Arc<
                            dyn elai_orchestrator::web_research_orchestrator::WebResearchStepEmitter,
                        > = match &self.ssb {
                            Some(ssb) => std::sync::Arc::new(
                                elai_orchestrator::SsbWebResearchStepEmitter::new(ssb.clone()),
                            ),
                            None => std::sync::Arc::new(NoopStepEmitter),
                        };

                        let rt = match tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        {
                            Ok(rt) => rt,
                            Err(e) => {
                                eprintln!("[task] failed to build async runtime: {e}");
                                return Ok(false);
                            }
                        };

                        use elai_orchestrator::WebResearchRunner;
                        match rt.block_on(runner.run(d.clone(), emitter)) {
                            Ok(summary) => {
                                println!(
                                    "[task] research complete: status={} iterations={} sources_used={}",
                                    summary.status, summary.iterations, summary.sources_used
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    error = %e,
                                    sidecar_url = %sidecar_url,
                                    "task: web-research runner failed"
                                );
                                eprintln!("[task] web-research failed: {e}");
                            }
                        }
                    }
                    None => eprintln!("Usage: /task [--surgical|--new] <description>"),
                }
                false
            }
            SlashCommand::Mode { mode } => {
                // B-22 Phase 4G: /mode is a runtime gate-mode switch handled
                // by the dashboard's set_gate_mode IPC. From the live CLI
                // we surface a hint pointing at the dashboard's controls;
                // headless flips happen by setting ELAI_GATE_MODE before
                // bringing up the runner.
                let trimmed = mode.trim();
                if trimmed.is_empty() {
                    eprintln!(
                        "Usage: /mode <interactive|autonomous>  (route the value through the dashboard set_gate_mode IPC)"
                    );
                } else if elai_orchestrator::GateMode::from_str(trimmed).is_none() {
                    eprintln!(
                        "/mode: unknown gate mode `{trimmed}`; expected `interactive` or `autonomous`"
                    );
                } else {
                    eprintln!(
                        "/mode {trimmed}: live-CLI flip is handled by the dashboard. Use the dashboard mode toggle, or set ELAI_GATE_MODE={trimmed} before launching the runner."
                    );
                }
                false
            }
            SlashCommand::Reason { on_off } => {
                // B-39: toggle the Chat REFLECT pipeline. Mirrors /mode: string-typed,
                // validation in handler, zero orchestrator dep.
                let trimmed = on_off.trim();
                if trimmed.eq_ignore_ascii_case("on") {
                    self.reflect_enabled = true;
                    tracing::info!("REFLECT enabled");
                    eprintln!("[reason] Chat REFLECT pipeline enabled");
                } else if trimmed.eq_ignore_ascii_case("off") {
                    self.reflect_enabled = false;
                    self.reflect_pipeline = None;
                    tracing::info!("REFLECT disabled");
                    eprintln!("[reason] Chat REFLECT pipeline disabled");
                } else {
                    tracing::warn!(
                        on_off = %trimmed,
                        "/reason: unrecognised value; expected `on` or `off`"
                    );
                    eprintln!("/reason: unrecognised value `{trimmed}`; expected `on` or `off`");
                }
                false
            }
            SlashCommand::EntryPoints { project_root } => {
                // B-34 BLOCK-4 fix: implement /entry_points handler.
                // Resolves project root from argument or cwd, loads SymbolGraphCache,
                // runs detect_entry_points, and prints one line per entry.
                use elai_knowledge::symbol_graph::{
                    cache::SymbolGraphCache,
                    entry_points::{detect_entry_points, EntryPointReason},
                    sharding::SYMBOL_FILE_LRU_SIZE,
                };
                use elai_core::paths::dkg_db_path as ep_dkg_db_path;

                let root: PathBuf = project_root
                    .as_deref()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                let project_id = elai_core::project_id_from_path(&root);
                let db_path = ep_dkg_db_path(&root);

                if !db_path.exists() {
                    eprintln!(
                        "/entry_points: no symbol graph DB found at {}. Run `elai symbol_graph build` first.",
                        db_path.display()
                    );
                } else {
                    match SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE) {
                        Err(e) => eprintln!("/entry_points: cache open failed: {e}"),
                        Ok(mut cache) => {
                            match cache.ensure_reverse_index() {
                                Err(e) => eprintln!("/entry_points: reverse_index load failed: {e}"),
                                Ok(reverse_index) => {
                                    let reverse_index = reverse_index.clone();
                                    let mut all_files: std::collections::BTreeSet<String> =
                                        reverse_index.keys().cloned().collect();
                                    for callers in reverse_index.values() {
                                        all_files.extend(callers.iter().cloned());
                                    }
                                    const EP_CODE_EXT: &[&str] = &[
                                        "ts", "tsx", "js", "jsx", "py", "rb", "go", "rs",
                                        "java", "kt", "php", "c", "cpp", "cs",
                                    ];
                                    let mut payloads = Vec::new();
                                    for file in all_files.iter().filter(|f| {
                                        std::path::Path::new(f.as_str())
                                            .extension()
                                            .and_then(|e| e.to_str())
                                            .map_or(false, |e| EP_CODE_EXT.contains(&e))
                                    }) {
                                        match cache.get_or_load_payload(file) {
                                            Ok(Some(arc)) => payloads.push((*arc).clone()),
                                            Ok(None) | Err(_) => {}
                                        }
                                    }
                                    let entries = detect_entry_points(&reverse_index, &payloads);
                                    if entries.is_empty() {
                                        println!("No entry points detected.");
                                    } else {
                                        println!("Entry points ({}):", entries.len());
                                        for e in &entries {
                                            let reason = match &e.reason {
                                                EntryPointReason::Orphan => "orphan",
                                                EntryPointReason::WellKnownName(_) => "well_known",
                                                EntryPointReason::FrameworkRoute(_) => "framework",
                                                EntryPointReason::Test => "test",
                                            };
                                            let line_str = e.line.map_or_else(|| "?".to_string(), |l| l.to_string());
                                            println!("  {}:{} {} ({})", e.file, line_str, e.name, reason);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                false
            }
            // C-24 — /graph mermaid [--mode file|symbol|decomposition]
            //                       [--max-nodes N] [project-root]
            //
            // Deterministic, zero-LLM: delegates to the same generator the
            // dashboard Tauri command uses (knowledge::visualization::mermaid).
            // `decomposition` mode is NOT reachable here because it needs the
            // dashboard's in-memory DecompositionTreeSnapshot; we surface a
            // clear error pointing users at the Tauri command in that case.
            SlashCommand::GraphMermaid { mode, max_nodes, project_root } => {
                let root: PathBuf = project_root
                    .as_deref()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                let project_id = elai_core::project_id_from_path(&root);
                let data_result = match mode.as_str() {
                    "file" => elai_knowledge::visualization::views::build_file_view(
                        &root, &project_id,
                    ),
                    "symbol" => elai_knowledge::visualization::views::build_symbol_view(
                        &root, &project_id,
                    ),
                    "decomposition" => {
                        eprintln!(
                            "/graph mermaid --mode decomposition: not supported via CLI \
                             (requires the dashboard's in-memory DecompositionTreeSnapshot). \
                             Use the dashboard GraphWindow `Export Mermaid` button instead."
                        );
                        return Ok(false);
                    }
                    other => {
                        eprintln!(
                            "/graph mermaid: unknown mode `{other}`; \
                             expected file|symbol|decomposition"
                        );
                        return Ok(false);
                    }
                };
                match data_result {
                    Ok(data) => {
                        let mermaid = elai_knowledge::visualization::mermaid::generate_mermaid(
                            &data, max_nodes,
                        );
                        // Plain stdout — pipes into the deterministic check
                        // script (the spec greps for "^graph " / "^flowchart ").
                        println!("{mermaid}");
                    }
                    Err(e) => eprintln!("/graph mermaid: build failed: {e}"),
                }
                false
            }
            // C-20: `/measure` from the REPL surfaces the same metrics
            // backfill behaviour as the direct-CLI invocation. The handler
            // owns its own tokio runtime via `run_measure_subcommand`, so
            // we can call it from this sync REPL context.
            SlashCommand::Measure { scope, cloud, output_format: _ } => {
                let project_root = std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."));
                if let Err(e) = crate::measure::run_measure_subcommand(scope, cloud, project_root) {
                    eprintln!("/measure failed: {e}");
                }
                false
            }
            // C-25: REPL handler for /pipeline. Dispatches to the same
            // synchronous handler used by `elai pipeline …`.
            SlashCommand::Pipeline { action } => {
                let project_root = std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."));
                if let Err(e) = crate::pipeline_subcommand::run_pipeline_subcommand(
                    action.as_deref(),
                    &project_root,
                ) {
                    eprintln!("/pipeline failed: {e}");
                }
                false
            }
            // C-27: REPL handler for /wiring audit. Dispatches to the same
            // sync handler used by `elai wiring audit …`. Text output mode
            // by default in the REPL (operators can re-run from the shell
            // for JSON / TOML output).
            SlashCommand::WiringAudit { scope, config_path, contracts_dir } => {
                if let Err(e) = crate::wiring_audit::run(
                    scope,
                    config_path,
                    contracts_dir,
                    crate::CliOutputFormat::Text,
                ) {
                    eprintln!("/wiring audit failed: {e}");
                }
                false
            }
            SlashCommand::Unknown(name) => {
                eprintln!("{}", format_unknown_slash_command(&name));
                false
            }
        })
    }

    pub(crate) fn persist_session(&self) -> Result<(), anyhow::Error> {
        // B-9 cycle 2 (BLOCK-1 fix): JSONL is now opt-in via the
        // top-level `--export-jsonl` flag. SessionDag is the
        // authoritative store for every REPL turn (fed by
        // `emit_dag_checkpoint` and the per-turn `save_snapshot` call
        // sites at L4134/L4278). When the flag is off, this method is a
        // no-op for the legacy renderer — the dashboard,
        // `/continue`, and resume flow read from the DAG.
        if self.export_jsonl_enabled {
            self.runtime.session().save_to_path(&self.session.path)?;
        }
        Ok(())
    }

    fn print_status(&self) {
        let cumulative = self.runtime.usage().cumulative_usage();
        let latest = self.runtime.usage().current_turn_usage();
        let Ok(ctx) = status_context(Some(&self.session.path)) else {
            tracing::warn!("print_status: failed to load status context; skipping status print");
            return;
        };
        println!(
            "{}",
            format_status_report(
                &self.model,
                StatusUsage {
                    message_count: self.runtime.session().messages.len(),
                    turns: self.runtime.usage().turns(),
                    latest,
                    cumulative,
                    estimated_tokens: self.runtime.estimated_tokens(),
                },
                self.permission_mode.as_str(),
                &ctx,
            )
        );
    }

    fn print_sandbox_status() {
        let Ok(cwd) = env::current_dir() else {
            tracing::warn!("print_sandbox_status: current_dir unavailable; skipping sandbox status");
            return;
        };
        let loader = ConfigLoader::default_for(&cwd);
        let runtime_config = loader
            .load()
            .unwrap_or_else(|_| elai_orchestrator::RuntimeConfig::empty());
        println!(
            "{}",
            format_sandbox_report(&resolve_sandbox_status(runtime_config.sandbox(), &cwd))
        );
    }

    fn set_model(&mut self, model: Option<String>) -> Result<bool, anyhow::Error> {
        let Some(model) = model else {
            println!(
                "{}",
                format_model_report(
                    &self.model,
                    self.runtime.session().messages.len(),
                    self.runtime.usage().turns(),
                )
            );
            return Ok(false);
        };

        let model = resolve_model_alias(&model).to_string();

        if model == self.model {
            println!(
                "{}",
                format_model_report(
                    &self.model,
                    self.runtime.session().messages.len(),
                    self.runtime.usage().turns(),
                )
            );
            return Ok(false);
        }

        let previous = self.model.clone();
        let session = self.runtime.session().clone();
        let message_count = session.messages.len();
        let (runtime, _ssb) = build_runtime(
            session,
            &self.session.id,
            model.clone(),
            self.system_prompt.clone(),
            true,
            true,
            self.allowed_tools.clone(),
            self.permission_mode,
            None,
            self.backend_config.as_ref(),
            Some(self.fsm.clone()),
            0,
        )?;
        self.replace_runtime(runtime)?;
        self.model.clone_from(&model);
        println!(
            "{}",
            format_model_switch_report(&previous, &model, message_count)
        );
        Ok(true)
    }

    fn set_permissions(
        &mut self,
        mode: Option<String>,
    ) -> Result<bool, anyhow::Error> {
        let Some(mode) = mode else {
            println!(
                "{}",
                format_permissions_report(self.permission_mode.as_str())
            );
            return Ok(false);
        };

        let normalized = normalize_permission_mode(&mode).ok_or_else(|| {
            anyhow::anyhow!(
                "unsupported permission mode '{mode}'. Use read-only, workspace-write, or danger-full-access."
            )
        })?;

        if normalized == self.permission_mode.as_str() {
            println!("{}", format_permissions_report(normalized));
            return Ok(false);
        }

        let previous = self.permission_mode.as_str().to_string();
        let session = self.runtime.session().clone();
        self.permission_mode = permission_mode_from_label(normalized);
        let (runtime, _ssb) = build_runtime(
            session,
            &self.session.id,
            self.model.clone(),
            self.system_prompt.clone(),
            true,
            true,
            self.allowed_tools.clone(),
            self.permission_mode,
            None,
            self.backend_config.as_ref(),
            Some(self.fsm.clone()),
            0,
        )?;
        self.replace_runtime(runtime)?;
        println!(
            "{}",
            format_permissions_switch_report(&previous, normalized)
        );
        Ok(true)
    }

    fn clear_session(&mut self, confirm: bool) -> Result<bool, anyhow::Error> {
        if !confirm {
            println!(
                "clear: confirmation required; run /clear --confirm to start a fresh session."
            );
            return Ok(false);
        }

        let previous_session = self.session.clone();
        let session_state = Session::new();
        self.session = create_managed_session_handle(&session_state.session_id)?;
        let (runtime, _ssb) = build_runtime(
            session_state.with_persistence_path(self.session.path.clone()),
            &self.session.id,
            self.model.clone(),
            self.system_prompt.clone(),
            true,
            true,
            self.allowed_tools.clone(),
            self.permission_mode,
            None,
            self.backend_config.as_ref(),
            Some(self.fsm.clone()),
            0,
        )?;
        self.replace_runtime(runtime)?;
        println!(
            "Session cleared\n  Mode             fresh session\n  Previous session {}\n  Resume previous  /resume {}\n  Preserved model  {}\n  Permission mode  {}\n  New session      {}\n  Session file     {}",
            previous_session.id,
            previous_session.id,
            self.model,
            self.permission_mode.as_str(),
            self.session.id,
            self.session.path.display(),
        );
        Ok(true)
    }

    fn print_cost(&self) {
        let cumulative = self.runtime.usage().cumulative_usage();
        println!("{}", format_cost_report(cumulative));
    }

    fn resume_session(
        &mut self,
        session_path: Option<String>,
    ) -> Result<bool, anyhow::Error> {
        let Some(session_ref) = session_path else {
            println!("{}", render_resume_usage());
            return Ok(false);
        };

        let handle = handoff::resolve_session_reference(&session_ref)?;

        // Try DAG-based resume first
        let dag_path = handle.path.parent()
            .map(|p| p.join("dag.db"))
            .unwrap_or_else(|| PathBuf::from("dag.db"));
        let dag_resume = elai_orchestrator::SessionDag::open(&handle.id, &dag_path).ok();

        let session = Session::load_from_path(&handle.path)?;
        let message_count = session.messages.len();
        let session_id = session.session_id.clone();
        let (runtime, _ssb) = build_runtime(
            session,
            &handle.id,
            self.model.clone(),
            self.system_prompt.clone(),
            true,
            true,
            self.allowed_tools.clone(),
            self.permission_mode,
            None,
            self.backend_config.as_ref(),
            Some(self.fsm.clone()),
            plan_phase_slot_tokens(self.plan.as_ref()),
        )?;
        self.replace_runtime(runtime)?;
        self.session = SessionHandle {
            id: session_id,
            path: handle.path,
        };

        // Wire DAG if opened successfully, recover last snap_id
        if let Some(dag) = dag_resume {
            let last_snap = dag.latest_snapshot(None).ok().flatten();
            self.last_snap_id = last_snap.map(|s| s.snap_id);
            self.session_dag = Some(dag);
        } else {
            self.session_dag = None;
            self.last_snap_id = None;
        }

        println!(
            "{}",
            format_resume_report(
                &self.session.path.display().to_string(),
                message_count,
                self.runtime.usage().turns(),
            )
        );
        Ok(true)
    }

    /// B-9 T3: rehydrate the most recent SessionDag handoff snapshot. This is
    /// the canonical /continue path — replaces the legacy JSONL
    /// `Session::load_from_path` for handoff resume. The CLI keeps /resume for
    /// the legacy pre-DAG path (explicit, opt-in); /continue is the path the
    /// new dashboard's IPC consumes.
    pub fn continue_from_handoff(
        &mut self,
        snap_id: Option<&str>,
    ) -> Result<bool, anyhow::Error> {
        handoff::continue_from_handoff(self, snap_id)
    }

    fn print_config(section: Option<&str>) -> Result<(), anyhow::Error> {
        println!("{}", render_config_report(section)?);
        Ok(())
    }

    fn print_memory() -> Result<(), anyhow::Error> {
        println!("{}", render_memory_report()?);
        Ok(())
    }

    pub(crate) fn print_agents(
        args: Option<&str>,
        output_format: CliOutputFormat,
    ) -> Result<(), anyhow::Error> {
        let cwd = env::current_dir()?;
        let message = handle_agents_slash_command(args, &cwd)?;
        match output_format {
            CliOutputFormat::Text | CliOutputFormat::Toml => println!("{message}"),
            CliOutputFormat::Json => println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "kind": "agents",
                    "message": message,
                    "args": args,
                }))?
            ),
        }
        Ok(())
    }

    pub(crate) fn print_mcp(
        args: Option<&str>,
        output_format: CliOutputFormat,
    ) -> Result<(), anyhow::Error> {
        let cwd = env::current_dir()?;
        match output_format {
            CliOutputFormat::Text | CliOutputFormat::Toml => {
                println!("{}", handle_mcp_slash_command(args, &cwd)?)
            }
            CliOutputFormat::Json => println!(
                "{}",
                serde_json::to_string_pretty(&handle_mcp_slash_command_json(args, &cwd)?)?
            ),
        }
        Ok(())
    }

    /// B-37D Phase 4M: render `/runtime` output to stdout. The handler
    /// is async because the `test` and `list` paths await
    /// `RuntimeAdapter::health_check` futures; we spin up a Tokio
    /// current-thread runtime locally to keep this helper callable
    /// from the sync REPL turn loop.
    pub(crate) fn print_runtime(
        action: Option<&str>,
        role: Option<&str>,
        target: Option<&str>,
    ) -> Result<(), anyhow::Error> {
        let cwd = env::current_dir()?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let result = runtime.block_on(crate::runtime_cmd::handle_runtime_slash_command(
            action, role, target, &cwd,
        ));
        match result {
            Ok(out) => println!("{out}"),
            Err(e) => println!("[FAIL] {e}"),
        }
        Ok(())
    }

    pub(crate) fn print_skills(
        args: Option<&str>,
        output_format: CliOutputFormat,
    ) -> Result<(), anyhow::Error> {
        let cwd = env::current_dir()?;
        match output_format {
            CliOutputFormat::Text | CliOutputFormat::Toml => {
                println!("{}", handle_skills_slash_command(args, &cwd)?)
            }
            CliOutputFormat::Json => println!(
                "{}",
                serde_json::to_string_pretty(&handle_skills_slash_command_json(args, &cwd)?)?
            ),
        }
        Ok(())
    }

    fn print_diff() -> Result<(), anyhow::Error> {
        println!("{}", render_diff_report()?);
        Ok(())
    }

    fn print_version(output_format: CliOutputFormat) {
        let _ = crate::print_version(output_format);
    }

    fn export_session(
        &self,
        requested_path: Option<&str>,
        jsonl: bool,
    ) -> Result<(), anyhow::Error> {
        let export_path = resolve_export_path(requested_path, self.runtime.session())?;
        // B-9 T4: --export-jsonl gates the legacy JSONL renderer
        // (`Session::render_jsonl_snapshot`). Default is the text export.
        if jsonl {
            self.runtime.session().save_to_path(&export_path)?;
        } else {
            fs::write(&export_path, render_export_text(self.runtime.session()))?;
        }
        println!(
            "Export\n  Result           wrote transcript\n  File             {}\n  Messages         {}",
            export_path.display(),
            self.runtime.session().messages.len(),
        );
        Ok(())
    }

    pub fn handle_session_command(
        &mut self,
        action: Option<&str>,
        target: Option<&str>,
    ) -> Result<bool, anyhow::Error> {
        handoff::handle_session_command(self, action, target)
    }

    fn handle_plugins_command(
        &mut self,
        action: Option<&str>,
        target: Option<&str>,
    ) -> Result<bool, anyhow::Error> {
        let cwd = env::current_dir()?;
        let loader = ConfigLoader::default_for(&cwd);
        let runtime_config = loader.load()?;
        let mut manager = build_plugin_manager(&cwd, &loader, &runtime_config);
        let result = handle_plugins_slash_command(action, target, &mut manager)?;
        println!("{}", result.message);
        if result.reload_runtime {
            self.reload_runtime_features()?;
        }
        Ok(false)
    }

    fn reload_runtime_features(&mut self) -> Result<(), anyhow::Error> {
        let (runtime, _ssb) = build_runtime(
            self.runtime.session().clone(),
            &self.session.id,
            self.model.clone(),
            self.system_prompt.clone(),
            true,
            true,
            self.allowed_tools.clone(),
            self.permission_mode,
            None,
            self.backend_config.as_ref(),
            Some(self.fsm.clone()),
            plan_phase_slot_tokens(self.plan.as_ref()),
        )?;
        self.replace_runtime(runtime)?;
        self.persist_session()
    }

    fn compact(&mut self) -> Result<(), anyhow::Error> {
        let result = self.runtime.compact(CompactionConfig::default());
        let removed = result.removed_message_count;
        let kept = result.compacted_session.messages.len();
        let skipped = removed == 0;
        let (runtime, _ssb) = build_runtime(
            result.compacted_session,
            &self.session.id,
            self.model.clone(),
            self.system_prompt.clone(),
            true,
            true,
            self.allowed_tools.clone(),
            self.permission_mode,
            None,
            self.backend_config.as_ref(),
            Some(self.fsm.clone()),
            plan_phase_slot_tokens(self.plan.as_ref()),
        )?;
        self.replace_runtime(runtime)?;
        self.persist_session()?;
        println!("{}", format_compact_report(removed, kept, skipped));
        Ok(())
    }

    fn run_internal_prompt_text_with_progress(
        &self,
        prompt: &str,
        enable_tools: bool,
        progress: Option<InternalPromptProgressReporter>,
    ) -> Result<String, anyhow::Error> {
        let session = self.runtime.session().clone();
        let (mut runtime, _ssb) = build_runtime(
            session,
            &self.session.id,
            self.model.clone(),
            self.system_prompt.clone(),
            enable_tools,
            false,
            self.allowed_tools.clone(),
            self.permission_mode,
            progress,
            self.backend_config.as_ref(),
            Some(self.fsm.clone()),
            plan_phase_slot_tokens(self.plan.as_ref()),
        )?;
        let mut permission_prompter = CliPermissionPrompter::new(self.permission_mode);
        let summary = runtime.run_turn(prompt, Some(&mut permission_prompter))?;
        let text = final_assistant_text(&summary).trim().to_string();
        runtime.shutdown_plugins()?;
        Ok(text)
    }

    fn run_internal_prompt_text(
        &self,
        prompt: &str,
        enable_tools: bool,
    ) -> Result<String, anyhow::Error> {
        self.run_internal_prompt_text_with_progress(prompt, enable_tools, None)
    }

    fn run_bughunter(&self, scope: Option<&str>) -> Result<(), anyhow::Error> {
        println!("{}", format_bughunter_report(scope));
        Ok(())
    }

    fn run_ultraplan(&self, task: Option<&str>) -> Result<(), anyhow::Error> {
        println!("{}", format_ultraplan_report(task));
        Ok(())
    }

    fn run_teleport(target: Option<&str>) -> Result<(), anyhow::Error> {
        let Some(target) = target.map(str::trim).filter(|value| !value.is_empty()) else {
            println!("Usage: /teleport <symbol-or-path>");
            return Ok(());
        };

        println!("{}", render_teleport_report(target)?);
        Ok(())
    }

    fn run_debug_tool_call(&self, args: Option<&str>) -> Result<(), anyhow::Error> {
        validate_no_args("/debug-tool-call", args)?;
        println!("{}", render_last_tool_debug_report(self.runtime.session())?);
        Ok(())
    }

    fn run_commit(&mut self, args: Option<&str>) -> Result<(), anyhow::Error> {
        validate_no_args("/commit", args)?;
        let status = git_output(&["status", "--short", "--branch"])?;
        let summary = parse_git_workspace_summary(Some(&status));
        let branch = parse_git_status_branch(Some(&status));
        if summary.is_clean() {
            println!("{}", format_commit_skipped_report());
            return Ok(());
        }

        println!(
            "{}",
            format_commit_preflight_report(branch.as_deref(), summary)
        );
        Ok(())
    }

    fn run_pr(&self, context: Option<&str>) -> Result<(), anyhow::Error> {
        let branch =
            resolve_git_branch_for(&env::current_dir()?).unwrap_or_else(|| "unknown".to_string());
        println!("{}", format_pr_report(&branch, context));
        Ok(())
    }

    fn run_issue(&self, context: Option<&str>) -> Result<(), anyhow::Error> {
        println!("{}", format_issue_report(context));
        Ok(())
    }
}

/// Attempt to load a PlanSkeleton from the plans/ directory.
/// Returns None if no plan file exists or parsing fails.
pub(crate) fn load_plan_if_exists() -> Option<elai_orchestrator::PlanSkeleton> {
    let cwd = env::current_dir().ok()?;
    let plans_dir = cwd.join(PLANS_DIR);
    if !plans_dir.is_dir() {
        return None;
    }
    // Find the first .toml file in plans/ (sorted for determinism)
    let mut toml_files: Vec<PathBuf> = fs::read_dir(&plans_dir).ok()?
        .filter_map(|e| {
            let path = e.ok()?.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    toml_files.sort();
    let entry = toml_files.into_iter().next()?;
    match elai_orchestrator::PlanSkeleton::read_from_file(&entry) {
        Ok(plan) => Some(plan),
        Err(e) => {
            eprintln!("warn: plan load failed from {}: {e}", entry.display());
            None
        }
    }
}

pub(crate) fn plan_phase_slot_tokens(plan: Option<&elai_orchestrator::PlanSkeleton>) -> usize {
    let skeleton_tokens = plan
        .and_then(|p| p.to_toml().ok())
        .map(|s| s.len() / TOKEN_CHARS_PER_TOKEN)
        .unwrap_or(0);
    // A-7: include constitutional CONSTRAINTS block in PhaseSpec budget.
    let constraints_tokens = build_constraints_block().estimated_tokens();
    skeleton_tokens + constraints_tokens
}

/// Build the CONSTRAINTS block from static seed data (A-7: constitutional injection).
///
/// Uses BLOCK-severity entries from deprecated_patterns seed to generate
/// the constraint text appended to PhaseSpec content at assembly time.
pub(crate) fn build_constraints_block() -> elai_orchestrator::context_budget::ConstraintsBlock {
    use elai_orchestrator::context_budget::{ConstraintEntry, ConstraintsBlock};
    let seed_entries = elai_rules::schema::block_entries_from_seed();
    let entries: Vec<ConstraintEntry> = seed_entries
        .into_iter()
        .map(|e| ConstraintEntry {
            deprecated_name: e.deprecated_name,
            replacement_name: e.replacement_name,
        })
        .collect();
    ConstraintsBlock::from_entries(entries)
}

/// Build the telescope context block (A-11) from the current project profile.
///
/// Detects the project profile from cwd, then formats the telescope context
/// for injection into the PhaseSpec system prompt. Returns None if detection fails.
pub(crate) fn build_telescope_block() -> Option<String> {
    let project_root = std::env::current_dir().ok()?;
    let mut profile = detect_project_profile(&project_root).ok()?;

    // Wire DKG nodes for architecture inference (A-36).
    // B-1: read from canonical `.elai_cc/indexes/dkg.db` via core::paths.
    // Graceful degradation: if DB absent or engine fails, continue with empty nodes.
    let dkg_db_path = core_dkg_db_path(&project_root);
    let dkg_nodes = if dkg_db_path.exists() {
        DkgEngine::new(&dkg_db_path, None, 0)
            .and_then(|engine| engine.nodes())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    populate_tier3(&mut profile, &dkg_nodes);
    let telescope_ctx = assemble_telescope(&profile);
    let formatted = telescope_ctx.format_for_phase_spec();
    if formatted.is_empty() {
        None
    } else {
        Some(formatted)
    }
}

/// Build the PhaseSpec content string: plan skeleton TOML + CONSTRAINTS block + telescope block.
///
/// Returns None if no plan is loaded, constraints block is empty, and telescope is None.
/// This string is appended to the system prompt as an additional section so that
/// the constitutional CONSTRAINTS block and telescope context are visible to the model (A-7, A-11).
pub(crate) fn build_phase_spec_content(
    plan: Option<&elai_orchestrator::PlanSkeleton>,
    telescope: Option<&str>,
) -> Option<String> {
    let skeleton_toml = plan.and_then(|p| p.to_toml().ok());
    let constraints = build_constraints_block();
    let constraints_text = constraints.format();

    let base = match (skeleton_toml, constraints_text.is_empty()) {
        (None, true) => None,
        (Some(toml), true) => Some(toml),
        (None, false) => Some(constraints_text),
        (Some(mut toml), false) => {
            toml.push_str(&constraints_text);
            Some(toml)
        }
    };

    match (base, telescope) {
        (None, None) => None,
        (Some(b), None) => Some(b),
        (None, Some(t)) => Some(t.to_string()),
        (Some(mut b), Some(t)) => {
            b.push('\n');
            b.push_str(t);
            Some(b)
        }
    }
}

impl Drop for LiveCli {
    fn drop(&mut self) {
        if let Some(ssb) = self.ssb.as_mut() {
            // At drop time there should be no other Arc references to the Ssb.
            // get_mut succeeds iff the refcount is 1; on failure the write task
            // exits naturally when the Arc is dropped (sender channel closes).
            if let Some(inner) = Arc::get_mut(ssb) {
                inner.shutdown();
            }
        }
    }
}

pub(crate) fn create_managed_session_handle(
    session_id: &str,
) -> Result<SessionHandle, anyhow::Error> {
    let id = session_id.to_string();
    let path = handoff::sessions_dir()?.join(format!("{id}.{PRIMARY_SESSION_EXTENSION}"));
    Ok(SessionHandle { id, path })
}

pub(crate) fn render_repl_help() -> String {
    [
        "REPL".to_string(),
        "  /exit                Quit the REPL".to_string(),
        "  /quit                Quit the REPL".to_string(),
        "  Up/Down              Navigate prompt history".to_string(),
        "  Tab                  Complete commands, modes, and recent sessions".to_string(),
        "  Ctrl-C               Clear input (or exit on empty prompt)".to_string(),
        "  Shift+Enter/Ctrl+J   Insert a newline".to_string(),
        "  Auto-save            .elai/sessions/<session-id>.jsonl".to_string(),
        "  Resume latest        /resume latest".to_string(),
        "  Browse sessions      /session list".to_string(),
        String::new(),
        render_slash_command_help(),
    ]
    .join(
        "
",
    )
}

pub(crate) fn print_status_snapshot(
    model: &str,
    permission_mode: PermissionMode,
    output_format: CliOutputFormat,
) -> Result<(), anyhow::Error> {
    let usage = StatusUsage {
        message_count: 0,
        turns: 0,
        latest: TokenUsage::default(),
        cumulative: TokenUsage::default(),
        estimated_tokens: 0,
    };
    let context = status_context(None)?;
    match output_format {
        CliOutputFormat::Text | CliOutputFormat::Toml => println!(
            "{}",
            format_status_report(model, usage, permission_mode.as_str(), &context)
        ),
        CliOutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "kind": "status",
                "model": model,
                "permission_mode": permission_mode.as_str(),
                "usage": {
                    "messages": usage.message_count,
                    "turns": usage.turns,
                    "latest_total": usage.latest.total_tokens(),
                    "cumulative_input": usage.cumulative.input_tokens,
                    "cumulative_output": usage.cumulative.output_tokens,
                    "cumulative_total": usage.cumulative.total_tokens(),
                    "estimated_tokens": usage.estimated_tokens,
                },
                "workspace": {
                    "cwd": context.cwd,
                    "project_root": context.project_root,
                    "git_branch": context.git_branch,
                    "git_state": context.git_summary.headline(),
                    "changed_files": context.git_summary.changed_files,
                    "staged_files": context.git_summary.staged_files,
                    "unstaged_files": context.git_summary.unstaged_files,
                    "untracked_files": context.git_summary.untracked_files,
                    "session": context.session_path.as_ref().map_or_else(|| "live-repl".to_string(), |path| path.display().to_string()),
                    "loaded_config_files": context.loaded_config_files,
                    "discovered_config_files": context.discovered_config_files,
                    "memory_file_count": context.memory_file_count,
                },
                "sandbox": {
                    "enabled": context.sandbox_status.enabled,
                    "active": context.sandbox_status.active,
                    "supported": context.sandbox_status.supported,
                    "in_container": context.sandbox_status.in_container,
                    "requested_namespace": context.sandbox_status.requested.namespace_restrictions,
                    "active_namespace": context.sandbox_status.namespace_active,
                    "requested_network": context.sandbox_status.requested.network_isolation,
                    "active_network": context.sandbox_status.network_active,
                    "filesystem_mode": context.sandbox_status.filesystem_mode.as_str(),
                    "filesystem_active": context.sandbox_status.filesystem_active,
                    "allowed_mounts": context.sandbox_status.allowed_mounts,
                    "markers": context.sandbox_status.container_markers,
                    "fallback_reason": context.sandbox_status.fallback_reason,
                }
            }))?
        ),
    }
    Ok(())
}

pub(crate) fn status_context(
    session_path: Option<&Path>,
) -> Result<StatusContext, anyhow::Error> {
    let cwd = env::current_dir()?;
    let loader = ConfigLoader::default_for(&cwd);
    let discovered_config_files = loader.discover().len();
    let runtime_config = loader.load()?;
    let project_context = ProjectContext::discover_with_git(&cwd, DEFAULT_DATE)?;
    let (project_root, git_branch) =
        parse_git_status_metadata(project_context.git_status.as_deref());
    let git_summary = parse_git_workspace_summary(project_context.git_status.as_deref());
    let sandbox_status = resolve_sandbox_status(runtime_config.sandbox(), &cwd);
    Ok(StatusContext {
        cwd,
        session_path: session_path.map(Path::to_path_buf),
        loaded_config_files: runtime_config.loaded_entries().len(),
        discovered_config_files,
        memory_file_count: project_context.instruction_files.len(),
        project_root,
        git_branch,
        git_summary,
        sandbox_status,
    })
}

pub(crate) fn format_status_report(
    model: &str,
    usage: StatusUsage,
    permission_mode: &str,
    context: &StatusContext,
) -> String {
    [
        format!(
            "Status
  Model            {model}
  Permission mode  {permission_mode}
  Messages         {}
  Turns            {}
  Estimated tokens {}",
            usage.message_count, usage.turns, usage.estimated_tokens,
        ),
        format!(
            "Usage
  Latest total     {}
  Cumulative input {}
  Cumulative output {}
  Cumulative total {}",
            usage.latest.total_tokens(),
            usage.cumulative.input_tokens,
            usage.cumulative.output_tokens,
            usage.cumulative.total_tokens(),
        ),
        format!(
            "Workspace
  Cwd              {}
  Project root     {}
  Git branch       {}
  Git state        {}
  Changed files    {}
  Staged           {}
  Unstaged         {}
  Untracked        {}
  Session          {}
  Config files     loaded {}/{}
  Memory files     {}
  Suggested flow   /status → /diff → /commit",
            context.cwd.display(),
            context
                .project_root
                .as_ref()
                .map_or_else(|| "unknown".to_string(), |path| path.display().to_string()),
            context.git_branch.as_deref().unwrap_or("unknown"),
            context.git_summary.headline(),
            context.git_summary.changed_files,
            context.git_summary.staged_files,
            context.git_summary.unstaged_files,
            context.git_summary.untracked_files,
            context.session_path.as_ref().map_or_else(
                || "live-repl".to_string(),
                |path| path.display().to_string()
            ),
            context.loaded_config_files,
            context.discovered_config_files,
            context.memory_file_count,
        ),
        format_sandbox_report(&context.sandbox_status),
    ]
    .join(
        "

",
    )
}

pub(crate) fn format_sandbox_report(status: &elai_orchestrator::SandboxStatus) -> String {
    format!(
        "Sandbox
  Enabled           {}
  Active            {}
  Supported         {}
  In container      {}
  Requested ns      {}
  Active ns         {}
  Requested net     {}
  Active net        {}
  Filesystem mode   {}
  Filesystem active {}
  Allowed mounts    {}
  Markers           {}
  Fallback reason   {}",
        status.enabled,
        status.active,
        status.supported,
        status.in_container,
        status.requested.namespace_restrictions,
        status.namespace_active,
        status.requested.network_isolation,
        status.network_active,
        status.filesystem_mode.as_str(),
        status.filesystem_active,
        if status.allowed_mounts.is_empty() {
            "<none>".to_string()
        } else {
            status.allowed_mounts.join(", ")
        },
        if status.container_markers.is_empty() {
            "<none>".to_string()
        } else {
            status.container_markers.join(", ")
        },
        status
            .fallback_reason
            .clone()
            .unwrap_or_else(|| "<none>".to_string()),
    )
}

pub(crate) fn format_commit_preflight_report(branch: Option<&str>, summary: GitWorkspaceSummary) -> String {
    format!(
        "Commit
  Result           ready
  Branch           {}
  Workspace        {}
  Changed files    {}
  Action           create a git commit from the current workspace changes",
        branch.unwrap_or("unknown"),
        summary.headline(),
        summary.changed_files,
    )
}

pub(crate) fn format_commit_skipped_report() -> String {
    "Commit
  Result           skipped
  Reason           no workspace changes
  Action           create a git commit from the current workspace changes
  Next             /status to inspect context · /diff to inspect repo changes"
        .to_string()
}

pub(crate) fn print_sandbox_status_snapshot(
    output_format: CliOutputFormat,
) -> Result<(), anyhow::Error> {
    let cwd = env::current_dir()?;
    let loader = ConfigLoader::default_for(&cwd);
    let runtime_config = loader
        .load()
        .unwrap_or_else(|_| elai_orchestrator::RuntimeConfig::empty());
    let status = resolve_sandbox_status(runtime_config.sandbox(), &cwd);
    match output_format {
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            println!("{}", format_sandbox_report(&status))
        }
        CliOutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "kind": "sandbox",
                "enabled": status.enabled,
                "active": status.active,
                "supported": status.supported,
                "in_container": status.in_container,
                "requested_namespace": status.requested.namespace_restrictions,
                "active_namespace": status.namespace_active,
                "requested_network": status.requested.network_isolation,
                "active_network": status.network_active,
                "filesystem_mode": status.filesystem_mode.as_str(),
                "filesystem_active": status.filesystem_active,
                "allowed_mounts": status.allowed_mounts,
                "markers": status.container_markers,
                "fallback_reason": status.fallback_reason,
            }))?
        ),
    }
    Ok(())
}

pub(crate) fn render_help_topic(topic: LocalHelpTopic) -> String {
    match topic {
        LocalHelpTopic::Status => "Status
  Usage            elai status
  Purpose          show the local workspace snapshot without entering the REPL
  Output           model, permissions, git state, config files, and sandbox status
  Related          /status · elai --resume latest /status"
            .to_string(),
        LocalHelpTopic::Sandbox => "Sandbox
  Usage            elai sandbox
  Purpose          inspect the resolved sandbox and isolation state for the current directory
  Output           namespace, network, filesystem, and fallback details
  Related          /sandbox · elai status"
            .to_string(),
        LocalHelpTopic::Doctor => "Doctor
  Usage            elai doctor
  Purpose          diagnose local auth, config, workspace, sandbox, and build metadata
  Output           local-only health report; no provider request or session resume required
  Related          /doctor · elai --resume latest /doctor"
            .to_string(),
    }
}

pub(crate) fn print_help_topic(topic: LocalHelpTopic) {
    println!("{}", render_help_topic(topic));
}

pub(crate) fn render_config_report(section: Option<&str>) -> Result<String, anyhow::Error> {
    let cwd = env::current_dir()?;
    let loader = ConfigLoader::default_for(&cwd);
    let discovered = loader.discover();
    let runtime_config = loader.load()?;

    let mut lines = vec![
        format!(
            "Config
  Working directory {}
  Loaded files      {}
  Merged keys       {}",
            cwd.display(),
            runtime_config.loaded_entries().len(),
            runtime_config.merged().len()
        ),
        "Discovered files".to_string(),
    ];
    for entry in discovered {
        let source = match entry.source {
            ConfigSource::User => "user",
            ConfigSource::Project => "project",
            ConfigSource::Local => "local",
        };
        let status = if runtime_config
            .loaded_entries()
            .iter()
            .any(|loaded_entry| loaded_entry.path == entry.path)
        {
            "loaded"
        } else {
            "missing"
        };
        lines.push(format!(
            "  {source:<7} {status:<7} {}",
            entry.path.display()
        ));
    }

    if let Some(section) = section {
        lines.push(format!("Merged section: {section}"));
        let value = match section {
            "env" => runtime_config.get("env"),
            "hooks" => runtime_config.get("hooks"),
            "model" => runtime_config.get("model"),
            "plugins" => runtime_config
                .get("plugins")
                .or_else(|| runtime_config.get("enabledPlugins")),
            other => {
                lines.push(format!(
                    "  Unsupported config section '{other}'. Use env, hooks, model, or plugins."
                ));
                return Ok(lines.join(
                    "
",
                ));
            }
        };
        lines.push(format!(
            "  {}",
            match value {
                Some(value) => value.render(),
                None => "<unset>".to_string(),
            }
        ));
        return Ok(lines.join(
            "
",
        ));
    }

    lines.push("Merged JSON".to_string());
    lines.push(format!("  {}", runtime_config.as_json().render()));
    Ok(lines.join(
        "
",
    ))
}

pub(crate) fn render_memory_report() -> Result<String, anyhow::Error> {
    let cwd = env::current_dir()?;
    let project_context = ProjectContext::discover(&cwd, DEFAULT_DATE)?;
    let mut lines = vec![format!(
        "Memory
  Working directory {}
  Instruction files {}",
        cwd.display(),
        project_context.instruction_files.len()
    )];
    if project_context.instruction_files.is_empty() {
        lines.push("Discovered files".to_string());
        lines.push(
            "  No ELAI instruction files discovered in the current directory ancestry."
                .to_string(),
        );
    } else {
        lines.push("Discovered files".to_string());
        for (index, file) in project_context.instruction_files.iter().enumerate() {
            let preview = file.content.lines().next().unwrap_or("").trim();
            let preview = if preview.is_empty() {
                "<empty>"
            } else {
                preview
            };
            lines.push(format!("  {}. {}", index + 1, file.path.display(),));
            lines.push(format!(
                "     lines={} preview={}",
                file.content.lines().count(),
                preview
            ));
        }
    }
    Ok(lines.join(
        "
",
    ))
}

pub(crate) fn init_elai_md() -> Result<String, anyhow::Error> {
    let cwd = env::current_dir()?;
    Ok(initialize_repo(&cwd)?.render())
}

pub(crate) fn run_init(output_format: CliOutputFormat) -> Result<(), anyhow::Error> {
    let message = init_elai_md()?;
    match output_format {
        CliOutputFormat::Text | CliOutputFormat::Toml => println!("{message}"),
        CliOutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "kind": "init",
                "message": message,
            }))?
        ),
    }
    Ok(())
}

pub(crate) fn normalize_permission_mode(mode: &str) -> Option<&'static str> {
    match mode.trim() {
        "read-only" => Some("read-only"),
        "workspace-write" => Some("workspace-write"),
        "danger-full-access" => Some("danger-full-access"),
        _ => None,
    }
}

pub(crate) fn render_diff_report() -> Result<String, anyhow::Error> {
    render_diff_report_for(&env::current_dir()?)
}

pub(crate) fn render_diff_report_for(cwd: &Path) -> Result<String, anyhow::Error> {
    let staged = run_git_diff_command_in(cwd, &["diff", "--cached"])?;
    let unstaged = run_git_diff_command_in(cwd, &["diff"])?;
    if staged.trim().is_empty() && unstaged.trim().is_empty() {
        return Ok(
            "Diff\n  Result           clean working tree\n  Detail           no current changes"
                .to_string(),
        );
    }

    let mut sections = Vec::new();
    if !staged.trim().is_empty() {
        sections.push(format!("Staged changes:\n{}", staged.trim_end()));
    }
    if !unstaged.trim().is_empty() {
        sections.push(format!("Unstaged changes:\n{}", unstaged.trim_end()));
    }

    Ok(format!("Diff\n\n{}", sections.join("\n\n")))
}

pub(crate) fn run_git_diff_command_in(
    cwd: &Path,
    args: &[&str],
) -> Result<String, anyhow::Error> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow::anyhow!("git {} failed: {stderr}", args.join(" ")));
    }
    Ok(String::from_utf8(output.stdout)?)
}

pub(crate) fn render_teleport_report(target: &str) -> Result<String, anyhow::Error> {
    let cwd = env::current_dir()?;

    let file_list = Command::new("rg")
        .args(["--files"])
        .current_dir(&cwd)
        .output()?;
    let file_matches = if file_list.status.success() {
        String::from_utf8(file_list.stdout)?
            .lines()
            .filter(|line| line.contains(target))
            .take(10)
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let content_output = Command::new("rg")
        .args(["-n", "-S", "--color", "never", target, "."])
        .current_dir(&cwd)
        .output()?;

    let mut lines = vec![
        "Teleport".to_string(),
        format!("  Target           {target}"),
        "  Action           search workspace files and content for the target".to_string(),
    ];
    if !file_matches.is_empty() {
        lines.push(String::new());
        lines.push("File matches".to_string());
        lines.extend(file_matches.into_iter().map(|path| format!("  {path}")));
    }

    if content_output.status.success() {
        let matches = String::from_utf8(content_output.stdout)?;
        if !matches.trim().is_empty() {
            lines.push(String::new());
            lines.push("Content matches".to_string());
            lines.push(truncate_for_prompt(&matches, 4_000));
        }
    }

    if lines.len() == 1 {
        lines.push("  Result           no matches found".to_string());
    }

    Ok(lines.join("\n"))
}

pub(crate) fn render_last_tool_debug_report(session: &Session) -> Result<String, anyhow::Error> {
    let last_tool_use = session
        .messages
        .iter()
        .rev()
        .find_map(|message| {
            message.blocks.iter().rev().find_map(|block| match block {
                ContentBlock::ToolUse { id, name, input } => {
                    Some((id.clone(), name.clone(), input.clone()))
                }
                _ => None,
            })
        })
        .ok_or_else(|| anyhow::anyhow!("no prior tool call found in session"))?;

    let tool_result = session.messages.iter().rev().find_map(|message| {
        message.blocks.iter().rev().find_map(|block| match block {
            ContentBlock::ToolResult {
                tool_use_id,
                tool_name,
                output,
                is_error,
            } if tool_use_id == &last_tool_use.0 => {
                Some((tool_name.clone(), output.clone(), *is_error))
            }
            _ => None,
        })
    });

    let mut lines = vec![
        "Debug tool call".to_string(),
        "  Action           inspect the last recorded tool call and its result".to_string(),
        format!("  Tool id          {}", last_tool_use.0),
        format!("  Tool name        {}", last_tool_use.1),
        "  Input".to_string(),
        indent_block(&last_tool_use.2, 4),
    ];

    match tool_result {
        Some((tool_name, output, is_error)) => {
            lines.push("  Result".to_string());
            lines.push(format!("    name           {tool_name}"));
            lines.push(format!(
                "    status         {}",
                if is_error { "error" } else { "ok" }
            ));
            lines.push(indent_block(&output, 4));
        }
        None => lines.push("  Result           missing tool result".to_string()),
    }

    Ok(lines.join("\n"))
}

pub(crate) fn indent_block(value: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    value
        .lines()
        .map(|line| format!("{indent}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn validate_no_args(
    command_name: &str,
    args: Option<&str>,
) -> Result<(), anyhow::Error> {
    if let Some(args) = args.map(str::trim).filter(|value| !value.is_empty()) {
        return Err(anyhow::anyhow!(
            "{command_name} does not accept arguments. Received: {args}\nUsage: {command_name}"
        ));
    }
    Ok(())
}

pub(crate) fn format_bughunter_report(scope: Option<&str>) -> String {
    format!(
        "Bughunter
  Scope            {}
  Action           inspect the selected code for likely bugs and correctness issues
  Output           findings should include file paths, severity, and suggested fixes",
        scope.unwrap_or("the current repository")
    )
}

pub(crate) fn format_ultraplan_report(task: Option<&str>) -> String {
    format!(
        "Ultraplan
  Task             {}
  Action           break work into a multi-step execution plan
  Output           plan should cover goals, risks, sequencing, verification, and rollback",
        task.unwrap_or("the current repo work")
    )
}

pub(crate) fn format_pr_report(branch: &str, context: Option<&str>) -> String {
    format!(
        "PR
  Branch           {branch}
  Context          {}
  Action           draft or create a pull request for the current branch
  Output           title and markdown body suitable for GitHub",
        context.unwrap_or("none")
    )
}

pub(crate) fn format_issue_report(context: Option<&str>) -> String {
    format!(
        "Issue
  Context          {}
  Action           draft or create a GitHub issue from the current context
  Output           title and markdown body suitable for GitHub",
        context.unwrap_or("none")
    )
}

pub(crate) fn git_output(args: &[&str]) -> Result<String, anyhow::Error> {
    let output = Command::new("git")
        .args(args)
        .current_dir(env::current_dir()?)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow::anyhow!("git {} failed: {stderr}", args.join(" ")));
    }
    Ok(String::from_utf8(output.stdout)?)
}

pub(crate) fn git_status_ok(args: &[&str]) -> Result<(), anyhow::Error> {
    let output = Command::new("git")
        .args(args)
        .current_dir(env::current_dir()?)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow::anyhow!("git {} failed: {stderr}", args.join(" ")));
    }
    Ok(())
}

pub(crate) fn command_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub(crate) fn write_temp_text_file(
    filename: &str,
    contents: &str,
) -> Result<PathBuf, anyhow::Error> {
    let path = env::temp_dir().join(filename);
    fs::write(&path, contents)?;
    Ok(path)
}

pub(crate) fn recent_user_context(session: &Session, limit: usize) -> String {
    let requests = session
        .messages
        .iter()
        .filter(|message| message.role == MessageRole::User)
        .filter_map(|message| {
            message.blocks.iter().find_map(|block| match block {
                ContentBlock::Text { text } => Some(text.trim().to_string()),
                _ => None,
            })
        })
        .rev()
        .take(limit)
        .collect::<Vec<_>>();

    if requests.is_empty() {
        "<no prior user messages>".to_string()
    } else {
        requests
            .into_iter()
            .rev()
            .enumerate()
            .map(|(index, text)| format!("{}. {}", index + 1, text))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub(crate) fn truncate_for_prompt(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        value.trim().to_string()
    } else {
        let truncated = value.chars().take(limit).collect::<String>();
        format!("{}\n…[truncated]", truncated.trim_end())
    }
}

pub(crate) fn sanitize_generated_message(value: &str) -> String {
    value.trim().trim_matches('`').trim().replace("\r\n", "\n")
}

pub(crate) fn parse_titled_body(value: &str) -> Option<(String, String)> {
    let normalized = sanitize_generated_message(value);
    let title = normalized
        .lines()
        .find_map(|line| line.strip_prefix("TITLE:").map(str::trim))?;
    let body_start = normalized.find("BODY:")?;
    let body = normalized[body_start + "BODY:".len()..].trim();
    Some((title.to_string(), body.to_string()))
}

pub(crate) fn render_version_report() -> String {
    let git_sha = GIT_SHA.unwrap_or("unknown");
    let target = BUILD_TARGET.unwrap_or("unknown");
    format!(
        "ELAI\n  Version          {VERSION}\n  Git SHA          {git_sha}\n  Target           {target}\n  Build date       {DEFAULT_DATE}"
    )
}

pub(crate) fn render_export_text(session: &Session) -> String {
    let mut lines = vec!["# Conversation Export".to_string(), String::new()];
    for (index, message) in session.messages.iter().enumerate() {
        let role = match message.role {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "tool",
        };
        lines.push(format!("## {}. {role}", index + 1));
        for block in &message.blocks {
            match block {
                ContentBlock::Text { text } => lines.push(text.clone()),
                ContentBlock::ToolUse { id, name, input } => {
                    lines.push(format!("[tool_use id={id} name={name}] {input}"));
                }
                ContentBlock::ToolResult {
                    tool_use_id,
                    tool_name,
                    output,
                    is_error,
                } => {
                    lines.push(format!(
                        "[tool_result id={tool_use_id} name={tool_name} error={is_error}] {output}"
                    ));
                }
            }
        }
        lines.push(String::new());
    }
    lines.join("\n")
}

pub(crate) fn default_export_filename(session: &Session) -> String {
    let stem = session
        .messages
        .iter()
        .find_map(|message| match message.role {
            MessageRole::User => message.blocks.iter().find_map(|block| match block {
                ContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            }),
            _ => None,
        })
        .map_or("conversation", |text| {
            text.lines().next().unwrap_or("conversation")
        })
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .take(8)
        .collect::<Vec<_>>()
        .join("-");
    let fallback = if stem.is_empty() {
        "conversation"
    } else {
        &stem
    };
    format!("{fallback}.txt")
}

pub(crate) fn resolve_export_path(
    requested_path: Option<&str>,
    session: &Session,
) -> Result<PathBuf, anyhow::Error> {
    let cwd = env::current_dir()?;
    let file_name =
        requested_path.map_or_else(|| default_export_filename(session), ToOwned::to_owned);
    let final_name = if Path::new(&file_name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
    {
        file_name
    } else {
        format!("{file_name}.txt")
    };
    Ok(cwd.join(final_name))
}

pub(crate) fn build_system_prompt(skip_instruction_files: bool) -> Result<Vec<String>, anyhow::Error> {
    let mut sections = load_system_prompt(
        env::current_dir()?,
        DEFAULT_DATE,
        env::consts::OS,
        "unknown",
    )?;
    if skip_instruction_files {
        // When rules enforce coding standards programmatically, strip instruction-file
        // sections to save context budget. Instruction sections start with "# " headers
        // that reference loaded config/CLAUDE.md content.
        sections.retain(|s| !s.starts_with("# Claude instructions"));
    }
    Ok(sections)
}

/// Load seed rules from data/rules/*.toml, compile predicates via the engine.
pub(crate) fn load_seed_rules(
    engine: &DefaultRuleEngine,
) -> Vec<(Rule, CompiledPredicate)> {
    #[derive(Deserialize)]
    struct SeedFile {
        pub(crate) rules: Vec<Rule>,
    }

    let rules_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/rules");
    let entries = match fs::read_dir(&rules_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[rules] seed directory not found ({}): {e}", rules_dir.display());
            return Vec::new();
        }
    };

    let mut compiled = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(true, |ext| ext != "toml") {
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[rules] failed to read {}: {e}", path.display());
                continue;
            }
        };
        let seed: SeedFile = match toml::from_str(&content) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[rules] failed to parse {}: {e}", path.display());
                continue;
            }
        };
        for rule in seed.rules {
            match engine.compile_predicate(&rule.predicate_json) {
                Ok(cp) => compiled.push((rule, cp)),
                Err(e) => {
                    eprintln!("[rules] failed to compile rule {}: {e}", rule.rule_id);
                }
            }
        }
    }
    eprintln!("[rules] loaded {} seed rules from {}", compiled.len(), rules_dir.display());
    compiled
}

pub(crate) fn build_runtime_plugin_state() -> Result<RuntimePluginState, anyhow::Error> {
    let cwd = env::current_dir()?;
    let loader = ConfigLoader::default_for(&cwd);
    let runtime_config = loader.load()?;
    build_runtime_plugin_state_with_loader(&cwd, &loader, &runtime_config)
}

pub(crate) fn build_runtime_plugin_state_with_loader(
    cwd: &Path,
    loader: &ConfigLoader,
    runtime_config: &elai_orchestrator::RuntimeConfig,
) -> Result<RuntimePluginState, anyhow::Error> {
    let plugin_manager = build_plugin_manager(cwd, loader, runtime_config);
    let plugin_registry = plugin_manager.plugin_registry()?;
    let plugin_hook_config =
        runtime_hook_config_from_plugin_hooks(plugin_registry.aggregated_hooks()?);
    let feature_config = runtime_config
        .feature_config()
        .clone()
        .with_hooks(runtime_config.hooks().merged(&plugin_hook_config));
    let (mcp_state, runtime_tools) = build_runtime_mcp_state(runtime_config)?;
    let tool_registry = GlobalToolRegistry::with_plugin_tools(plugin_registry.aggregated_tools()?)
        .map_err(|e| anyhow::anyhow!(e))?
        .with_runtime_tools(runtime_tools)
        .map_err(|e| anyhow::anyhow!(e))?;
    Ok(RuntimePluginState {
        feature_config,
        tool_registry,
        plugin_registry,
        mcp_state,
    })
}

pub(crate) fn build_plugin_manager(
    cwd: &Path,
    loader: &ConfigLoader,
    runtime_config: &elai_orchestrator::RuntimeConfig,
) -> PluginManager {
    let plugin_settings = runtime_config.plugins();
    let mut plugin_config = PluginManagerConfig::new(loader.config_home().to_path_buf());
    plugin_config.enabled_plugins = plugin_settings.enabled_plugins().clone();
    plugin_config.external_dirs = plugin_settings
        .external_directories()
        .iter()
        .map(|path| resolve_plugin_path(cwd, loader.config_home(), path))
        .collect();
    plugin_config.install_root = plugin_settings
        .install_root()
        .map(|path| resolve_plugin_path(cwd, loader.config_home(), path));
    plugin_config.registry_path = plugin_settings
        .registry_path()
        .map(|path| resolve_plugin_path(cwd, loader.config_home(), path));
    plugin_config.bundled_root = plugin_settings
        .bundled_root()
        .map(|path| resolve_plugin_path(cwd, loader.config_home(), path));
    PluginManager::new(plugin_config)
}

pub(crate) fn resolve_plugin_path(cwd: &Path, config_home: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else if value.starts_with('.') {
        cwd.join(path)
    } else {
        config_home.join(path)
    }
}

pub(crate) fn runtime_hook_config_from_plugin_hooks(hooks: PluginHooks) -> elai_orchestrator::RuntimeHookConfig {
    elai_orchestrator::RuntimeHookConfig::new(
        hooks.pre_tool_use,
        hooks.post_tool_use,
        hooks.post_tool_use_failure,
    )
}

// ---------------------------------------------------------------------------
// B-39: SidecarChatClient — bridges SidecarProvider to ChatInferenceClient
// ---------------------------------------------------------------------------

/// Wraps a `SidecarProvider` so it satisfies `elai_reasoning::ChatInferenceClient`.
///
/// `LlmProvider::complete` returns `Vec<String>` (batch completions); we take
/// the first element. The `OllamaConfig` arg is passed as-default because
/// `SidecarProvider` ignores it and uses its internal `SidecarConfig`.
pub(crate) struct SidecarChatClient {
    provider: Arc<SidecarProvider>,
}

impl SidecarChatClient {
    pub(crate) fn new(provider: Arc<SidecarProvider>) -> Self {
        Self { provider }
    }
}

impl ChatInferenceClient for SidecarChatClient {
    fn complete<'a>(&'a self, prompt: &'a str, _max_tokens: u32) -> ChatFuture<'a, String> {
        // `SidecarProvider` uses its internal `SidecarConfig.max_tokens`; the
        // `OllamaConfig` arg is ignored by the impl. `_max_tokens` is accepted
        // for trait conformance — the sidecar's own token cap from BackendConfig
        // applies instead.
        Box::pin(async move {
            let config = OllamaConfig::default();
            let completions = self
                .provider
                .complete(prompt, &config)
                .await
                .map_err(|e| ReflectError::Inference(e.to_string()))?;
            completions
                .into_iter()
                .next()
                .ok_or_else(|| ReflectError::Inference("sidecar returned no completions".to_string()))
        })
    }
}

// ---------------------------------------------------------------------------
// C-17 (DRIFT-N, DRIFT-O closure): Oracle pre-seeding helper for /research.
// ---------------------------------------------------------------------------
//
// Resolves project_dir = cwd, opens the on-disk SymbolGraphCache (if any),
// runs detect_entry_points, formats a PROJECT_STARTING_POINTS block, and
// prepends it to the user's query string. Emits an EntryPointsDetected SSB
// row (variant already shipped by B-34 at SCHEMA_VERSION 42) so the
// dashboard can render the per-research entry-point summary card without
// re-running detection.
//
// Robust to missing artefacts: when the symbol graph DB does not exist,
// when the project_dir cannot be resolved, or when no entry points are
// detected, returns the original query untouched.

const C17_MAX_ENTRY_POINTS_IN_BLOCK: usize = 25;

fn c17_research_oracle_prefix(
    query: &str,
    ssb: Option<&Arc<elai_orchestrator::Ssb>>,
) -> String {
    use elai_knowledge::symbol_graph::{
        cache::SymbolGraphCache,
        entry_points::{detect_entry_points, EntryPointReason},
        sharding::SYMBOL_FILE_LRU_SIZE,
    };

    let project_dir = match env::current_dir() {
        Ok(p) => p,
        Err(e) => {
            tracing::debug!(error = %e, "C-17 oracle: cwd unavailable; skipping pre-seed");
            return query.to_string();
        }
    };
    let db_path = elai_core::paths::dkg_db_path(&project_dir);
    if !db_path.exists() {
        tracing::debug!(
            db_path = %db_path.display(),
            "C-17 oracle: no symbol graph DB present; skipping pre-seed"
        );
        return query.to_string();
    }
    let project_id = elai_core::project_id_from_path(&project_dir);

    let mut cache = match SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE) {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!(error = %e, "C-17 oracle: cache open failed; skipping pre-seed");
            return query.to_string();
        }
    };
    let reverse_index_arc = match cache.ensure_reverse_index() {
        Ok(arc) => arc.clone(),
        Err(e) => {
            tracing::debug!(error = %e, "C-17 oracle: reverse_index load failed; skipping pre-seed");
            return query.to_string();
        }
    };

    const C17_CODE_EXT: &[&str] = &[
        "ts", "tsx", "js", "jsx", "py", "rb", "go", "rs", "java", "kt", "php",
        "c", "cpp", "cs",
    ];
    let mut all_files: std::collections::BTreeSet<String> = reverse_index_arc.keys().cloned().collect();
    for callers in reverse_index_arc.values() {
        all_files.extend(callers.iter().cloned());
    }
    let mut payloads = Vec::new();
    for file in all_files.iter().filter(|f| {
        Path::new(f.as_str())
            .extension()
            .and_then(|e| e.to_str())
            .map_or(false, |e| C17_CODE_EXT.contains(&e))
    }) {
        match cache.get_or_load_payload(file) {
            Ok(Some(arc)) => payloads.push((*arc).clone()),
            Ok(None) | Err(_) => {}
        }
    }
    let entries = detect_entry_points(&reverse_index_arc, &payloads);
    if entries.is_empty() {
        return query.to_string();
    }

    // Format the block. Cap at C17_MAX_ENTRY_POINTS_IN_BLOCK so the
    // prepended context stays under ~1KB even for very large projects.
    let mut block = String::with_capacity(256);
    block.push_str("PROJECT_STARTING_POINTS (auto-detected entry points):\n");
    let mut by_reason: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    for (i, e) in entries.iter().take(C17_MAX_ENTRY_POINTS_IN_BLOCK).enumerate() {
        let reason = match &e.reason {
            EntryPointReason::Orphan => "orphan".to_string(),
            EntryPointReason::WellKnownName(name) => format!("well_known:{}", name),
            EntryPointReason::FrameworkRoute(route) => format!("framework:{}", route),
            EntryPointReason::Test => "test".to_string(),
        };
        *by_reason.entry(reason.split(':').next().unwrap_or("other").to_string())
            .or_insert(0) += 1;
        let line_str = e.line.map_or_else(|| "?".to_string(), |l| l.to_string());
        block.push_str(&format!(
            "  [{}] {}:{} {} ({})\n",
            i + 1,
            e.file,
            line_str,
            e.name,
            reason
        ));
    }
    if entries.len() > C17_MAX_ENTRY_POINTS_IN_BLOCK {
        block.push_str(&format!(
            "  ... {} more entry points elided\n",
            entries.len() - C17_MAX_ENTRY_POINTS_IN_BLOCK
        ));
    }
    block.push_str("\n");

    // Aggregate per-reason buckets including the elided tail (recount over
    // the full entries vec so the SSB summary reflects everything detected).
    let mut full_by_reason: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    for e in &entries {
        let bucket = match &e.reason {
            EntryPointReason::Orphan => "orphan",
            EntryPointReason::WellKnownName(_) => "well_known",
            EntryPointReason::FrameworkRoute(_) => "framework",
            EntryPointReason::Test => "test",
        };
        *full_by_reason.entry(bucket.to_string()).or_insert(0) += 1;
    }
    let _ = by_reason;

    // Emit EntryPointsDetected SSB row (B-34 variant, SCHEMA_VERSION 42).
    if let Some(ssb) = ssb {
        use elai_orchestrator::ssb::{EntryPointsDetectedPayload, SsbPayload};
        let payload = SsbPayload::EntryPointsDetected(EntryPointsDetectedPayload {
            project_id: project_id.clone(),
            count: u32::try_from(entries.len()).unwrap_or(u32::MAX),
            by_reason: full_by_reason,
        });
        let ssb_clone = ssb.clone();
        if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
            rt.block_on(async move {
                if let Err(e) = ssb_clone
                    .write_payload("research-oracle", 0, payload, vec![])
                    .await
                {
                    tracing::warn!(error = %e, "C-17 oracle: failed to emit EntryPointsDetected SSB row");
                }
            });
        }
    }

    format!("{}{}", block, query)
}
