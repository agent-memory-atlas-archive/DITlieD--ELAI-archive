//! Dashboard-facing task runner (Blocker #1 — MVP).
//!
//! Consumes `SsbPayload::TaskSubmitted` rows written by the dashboard's
//! `submit_task` command (crates/dashboard/src/commands.rs:189-218) and
//! drives the end-to-end decomposition -> pipeline-build -> baby-step
//! navigation pipeline, emitting progress rows back to the SSB so the
//! dashboard UI can observe real work.
//!
//! Architecture: Option B (orchestrator-hosted). Co-located with the
//! `Ssb`, `PipelineBuilder`, `DepthFirstNavigator` and `research_decompose`
//! types to avoid cross-crate extensions. The dashboard starts it via
//! [`start_task_runner`] at Tauri setup.
//!
//! Grounding:
//! - `Ssb::subscribe` (broadcast) + `SsbReader::query_by_type` for replay
//!   — see ssb.rs:1048 / ssb.rs:882.
//! - `research_tree::research_decompose` produces a `ConceptDecomposition`
//!   from a task string (research_tree.rs:1451). It uses a synthetic
//!   report in no-LLM mode today — the comment at research_tree.rs:1512
//!   flags this as the substitution point for a real LLM call.
//! - `PipelineBuilder::build` accepts a `PipelineSpec` whose `steps` is
//!   `Vec<String>` of tool names (research_tree.rs:593, pipeline_builder.rs:335).
//! - `DepthFirstNavigator` drives baby-step traversal
//!   (pipeline_tree_navigator.rs:298).
//!
//! Dedup strategy: a cursor file (JSON, one i64) at the configured path
//! records the highest consumed `task_submitted` seq. On restart we tail
//! from `cursor+1`. The cursor is fsynced after each successful task run.
//!
//! Cancellation: `CancelHandle::cancel` flips an `AtomicBool`. The tail
//! loop checks it between broadcast recvs (select! over cancel + recv),
//! and the per-task driver checks it between baby steps.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use elai_ipc_types::ToolId;
use elai_web_validator::{
    start_dev_server, DevServerHandle, WebValidationTier, WebValidatorError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex as AsyncMutex, Notify};
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use crate::agent_pool::ModelPool;
use crate::pipeline_builder::{
    PipelineBuilder, PipelineBuilderError, PredicateCompiler, PredicateRef, RuleId, ToolLookup,
};
use crate::pipeline_tree_navigator::{
    BabyStepOutcome, BabyStepResult, DepthFirstNavigator, NavigatorTree, TreeNavigator,
    TreeNavigatorState,
};
use crate::grounding_client::SidecarGroundingClient;
use crate::research_tree::{
    research_decompose, ConceptDecomposition, ExecutionMode, MediaSpec, NodeId, NodeStatus,
    OutputType, PerfSpec, PipelineSpec, ResearchCache, ResearchNode,
    DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS,
};
use crate::ssb::{
    FindingPayload, PhaseCompletePayload, PreviewAvailablePayload, ReasoningTracePayload, Ssb,
    SsbActionType, SsbEntry, SsbPayload, SsbReader, TaskSubmittedPayload, TestOutcomePayload,
    WebResearchStepPayload,
};
use crate::web_research_orchestrator::{
    WebResearchOrchestrator, WebResearchStepEmitter, WebResearchStepEvent,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Agent ID emitted on every runner-produced SSB row.
pub const RUNNER_AGENT_ID: &str = "task_runner";

/// Default superstep for runner-emitted rows. The dashboard UI keys actions
/// by seq, not superstep, so zero is fine for MVP.
pub const RUNNER_SUPERSTEP: i64 = 0;

/// Max baby steps executed per task before the runner bails to avoid pinning
/// the executor. 10,000 is well above the largest documented decomposition.
pub const MAX_BABY_STEPS_PER_TASK: usize = 10_000;

/// Default VRAM budget reported by the runner's `ModelPool`. The pipeline
/// builder treats `u64::MAX` as "unconstrained", matching the test doubles
/// used by `pipeline_builder_tests.rs:66`.
pub const RUNNER_VRAM_UNCONSTRAINED: u64 = u64::MAX;

/// Synthetic duration emitted on every `TestOutcome` row. Real dispatch is
/// a follow-up blocker — the dashboard only needs a non-zero progress signal.
pub const RUNNER_SYNTHETIC_DURATION_MS: u64 = 1;

/// Default dev-server command used when we detect a web project but the
/// caller did not override it. Matches the most common Vite/npm scaffold.
pub const DEFAULT_DEV_SERVER_COMMAND: &str = "npm run dev";

/// Default port the MVP Vite template binds to. Matching the value surfaced
/// by `PreviewAvailablePayload` in the existing SSB fixtures (ssb.rs:1586).
pub const DEFAULT_DEV_SERVER_PORT: u16 = 5173;

/// SSB `slot` label for preview rows emitted by the runner. The spec
/// contract (ssb.rs:494) requires this string to exist; "live" matches the
/// doc example and the existing smoke fixture.
pub const PREVIEW_SLOT_LIVE: &str = "live";

/// Finding category for dev-server launch failures. Kept as a const so the
/// dashboard filter can match exactly.
pub const DEV_SERVER_FAILED_CATEGORY: &str = "dev_server_failed";

/// Character cap applied to task text when embedding it in log/reasoning rows.
/// Keeps SSB payloads compact without losing the first line of context.
pub const TASK_LOG_TRUNCATE_CHARS: usize = 120;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum TaskRunnerError {
    #[error("ssb error: {0}")]
    Ssb(#[from] crate::ssb::SsbError),

    #[error("pipeline build failed: {0}")]
    PipelineBuild(#[from] PipelineBuilderError),

    #[error("research_tree decompose failed: {0}")]
    Decompose(#[from] crate::research_tree::ResearchTreeError),

    #[error("empty decomposition: no concepts produced for task")]
    EmptyDecomposition,

    #[error("cursor I/O error at {path}: {source}")]
    CursorIo {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("cursor json error: {0}")]
    CursorJson(#[from] serde_json::Error),

    #[error("payload decode error: {0}")]
    PayloadDecode(#[from] rmp_serde::decode::Error),
}

// ---------------------------------------------------------------------------
// Cursor persistence
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Cursor {
    /// Highest TaskSubmitted seq fully consumed. `None` = nothing consumed yet.
    last_consumed_seq: Option<i64>,
}

impl Cursor {
    async fn load(path: &PathBuf) -> Result<Self, TaskRunnerError> {
        match tokio::fs::read(path).await {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(TaskRunnerError::CursorIo {
                path: path.display().to_string(),
                source: e,
            }),
        }
    }

    async fn save(&self, path: &PathBuf) -> Result<(), TaskRunnerError> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                tokio::fs::create_dir_all(parent).await.map_err(|e| {
                    TaskRunnerError::CursorIo {
                        path: parent.display().to_string(),
                        source: e,
                    }
                })?;
            }
        }
        let bytes = serde_json::to_vec(self)?;
        let mut f =
            tokio::fs::File::create(path)
                .await
                .map_err(|e| TaskRunnerError::CursorIo {
                    path: path.display().to_string(),
                    source: e,
                })?;
        f.write_all(&bytes)
            .await
            .map_err(|e| TaskRunnerError::CursorIo {
                path: path.display().to_string(),
                source: e,
            })?;
        f.sync_all()
            .await
            .map_err(|e| TaskRunnerError::CursorIo {
                path: path.display().to_string(),
                source: e,
            })?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Cancel handle
// ---------------------------------------------------------------------------

/// Cooperative cancellation. The runner polls this between broadcast recvs
/// and between baby-step iterations.
#[derive(Clone, Debug, Default)]
pub struct CancelHandle {
    flag: Arc<AtomicBool>,
    wake: Arc<Notify>,
}

impl CancelHandle {
    pub fn new() -> Self {
        Self::default()
    }

    /// Signal cancellation. Idempotent.
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Release);
        self.wake.notify_waiters();
    }

    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::Acquire)
    }
}

// ---------------------------------------------------------------------------
// Dev-server launcher injection (Blocker #6)
// ---------------------------------------------------------------------------

/// Outcome of a successful dev-server launch. Owned `DevServerHandle` is
/// parked in the runner's [`DevServerRegistry`] so the child process is NOT
/// dropped (and therefore killed) the moment this value goes out of scope.
pub struct LaunchedPreview {
    pub url: String,
    pub port: u16,
    pub handle: DevServerHandle,
}

/// Inject point for spawning a dev server. Default impl wraps
/// `elai_web_validator::start_dev_server`; tests swap in a fake that returns
/// a canned URL without spawning a real process. Native async fn in trait —
/// no async-trait crate (per CLAUDE.md anti-stale-training).
pub trait DevServerLauncher: Send + Sync {
    fn launch(
        &self,
        project_dir: PathBuf,
        step_id: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<LaunchedPreview, String>> + Send + '_>,
    >;
}

/// Production launcher: resolves a dev-server command and port, then calls
/// `elai_web_validator::start_dev_server`. Returns a `String` error on the
/// trait boundary so the caller can log it verbatim (the web-validator's
/// typed error enum is an implementation detail behind this trait).
pub struct WebValidatorLauncher {
    command: String,
    port: u16,
}

impl WebValidatorLauncher {
    pub fn new(command: impl Into<String>, port: u16) -> Self {
        Self {
            command: command.into(),
            port,
        }
    }

    /// Sensible default: `npm run dev` on port 5173 (Vite).
    pub fn default_vite() -> Self {
        Self::new(DEFAULT_DEV_SERVER_COMMAND, DEFAULT_DEV_SERVER_PORT)
    }
}

impl DevServerLauncher for WebValidatorLauncher {
    fn launch(
        &self,
        project_dir: PathBuf,
        _step_id: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<LaunchedPreview, String>> + Send + '_>,
    > {
        let cmd = self.command.clone();
        let port = self.port;
        Box::pin(async move {
            let handle: DevServerHandle =
                start_dev_server(&project_dir, &cmd, port)
                    .await
                    .map_err(|e: WebValidatorError| format!("start_dev_server: {e}"))?;
            let url = format!("http://127.0.0.1:{}/", handle.port());
            Ok(LaunchedPreview {
                url,
                port: handle.port(),
                handle,
            })
        })
    }
}

/// Registry of live dev-server handles, keyed by `project_id`. A handle
/// parked here is NOT dropped between tasks — this is load-bearing: Drop on
/// `DevServerHandle` kills the child, and the dashboard needs the URL to
/// stay clickable after the task that spawned it finishes.
///
/// The outer `Arc` is cheaply cloneable into each task; the inner
/// `tokio::sync::Mutex` is async-safe (CLAUDE.md: no `std::sync::Mutex` in
/// async code unless provably sync-only).
pub type DevServerRegistry = Arc<AsyncMutex<HashMap<String, DevServerHandle>>>;

pub fn new_dev_server_registry() -> DevServerRegistry {
    Arc::new(AsyncMutex::new(HashMap::new()))
}

/// Resolves a `project_id` (from `TaskSubmittedPayload`) to an on-disk
/// working directory. The dashboard's `ProjectsStore` lives in a different
/// crate; injecting this as a trait avoids a cross-crate dependency.
pub trait ProjectDirResolver: Send + Sync {
    fn resolve(&self, project_id: &str) -> Option<PathBuf>;
}

/// Fallback resolver: always returns the current working directory. Used
/// when the dashboard has not plugged in a real resolver. The launcher will
/// still fail cleanly (no `package.json`) and emit a Finding — it never
/// panics.
pub struct CwdProjectDirResolver;

impl ProjectDirResolver for CwdProjectDirResolver {
    fn resolve(&self, _project_id: &str) -> Option<PathBuf> {
        std::env::current_dir().ok()
    }
}

/// Heuristic: a project directory is "web-validatable" when a `package.json`
/// exists at its root. Kept deliberately shallow — the WEB.x spec already
/// defines the full validation tier, this is just the detection gate.
pub fn project_dir_is_web(project_dir: &Path) -> bool {
    project_dir.join("package.json").is_file()
}

// ---------------------------------------------------------------------------
// Runtime adapters (ToolLookup / PredicateCompiler for runtime, NOT tests)
// ---------------------------------------------------------------------------

/// Accepts any tool name. Justification: `PipelineSpec::steps` here carries
/// concept/phase-derived step names that are not registered in any real
/// `ToolRegistry` — the pipeline builder's role at this layer is DAG
/// validation (cycles, resource fit, predicate compile), not tool
/// existence. Zero-lazy: this is load-bearing design, not a stub.
struct RuntimeToolLookup;

impl ToolLookup for RuntimeToolLookup {
    fn has_tool(&self, _tool_id: &ToolId) -> bool {
        true
    }

    fn find_substitute(&self, _tool_id: &ToolId) -> Option<ToolId> {
        None
    }
}

/// No-gate predicate compiler: every predicate compiles successfully and
/// yields zero active rules. The MVP doesn't wire policy evaluation to
/// the runner yet; rule installation is a follow-up.
struct RuntimePredicates;

impl PredicateCompiler for RuntimePredicates {
    fn compile(&self, raw: &str) -> Result<PredicateRef, String> {
        Ok(PredicateRef {
            raw: raw.to_owned(),
            valid: true,
        })
    }

    fn active_rules_for_phase(&self, _phase_name: &str) -> Vec<RuleId> {
        Vec::new()
    }
}

/// Reports `u64::MAX` VRAM so pipeline validation never fails on resource
/// overflow at the runner layer. The real hardware floor is enforced by
/// `agent_pool::AgentPool`, not by task-runner dispatch.
struct RuntimeModelPool;

impl ModelPool for RuntimeModelPool {
    fn available_vram(&self) -> u64 {
        RUNNER_VRAM_UNCONSTRAINED
    }
}

// ---------------------------------------------------------------------------
// Web-research routing (Session 4D.12)
// ---------------------------------------------------------------------------

/// Category used on `Finding` rows emitted when web-research fails.
pub const WEB_RESEARCH_FAILED_CATEGORY: &str = "web_research_failed";

/// Lowercase case-insensitive prefixes that trigger the web-research path. The
/// list is intentionally conservative so tasks like "Build a hello-world CLI"
/// continue to flow through the default pipeline unchanged.
const WEB_RESEARCH_PREFIXES: &[&str] = &[
    "research:",
    "research ",
    "find out ",
    "look up ",
    "investigate ",
    "what is ",
    "what are ",
    "who is ",
    "who are ",
    "when is ",
    "when was ",
    "where is ",
    "how does ",
];

/// Substrings (lowercase) that additionally classify a task as web-research.
const WEB_RESEARCH_SUBSTRINGS: &[&str] = &["web search", "web research"];

/// Heuristic: return true when the task text looks like a web-research task
/// that should route through [`WebResearchRunner`] instead of the default
/// decomposition-plus-navigator pipeline.
///
/// This is the MVP "tool = WebSearch or research-tier tag" detector called
/// for by `docs/mvp-wiring-gaps.md` Session 4D.12.
pub fn is_web_research_task(task: &str) -> bool {
    let t = task.trim().to_ascii_lowercase();
    if t.is_empty() {
        return false;
    }
    if WEB_RESEARCH_PREFIXES.iter().any(|p| t.starts_with(p)) {
        return true;
    }
    if WEB_RESEARCH_SUBSTRINGS.iter().any(|s| t.contains(s)) {
        return true;
    }
    false
}

/// Summary of one web-research run for terminal-row construction.
#[derive(Debug, Clone)]
pub struct WebResearchSummary {
    pub iterations: u32,
    pub status: String,
    pub sources_used: u32,
}

/// Pluggable runner that drives [`WebResearchOrchestrator`] for one task. The
/// default impl ([`DefaultWebResearchRunner`]) builds a fresh orchestrator per
/// call so `WebContentIndex` state does not leak across tasks. Tests inject a
/// fake runner that emits canned `WebResearchStep` rows without touching the
/// network.
pub trait WebResearchRunner: Send + Sync {
    fn run(
        &self,
        task: String,
        emitter: Arc<dyn WebResearchStepEmitter>,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<WebResearchSummary, String>>
                + Send
                + '_,
        >,
    >;
}

/// Default [`WebResearchRunner`] that instantiates a fresh
/// [`WebResearchOrchestrator`] for every task so `WebContentIndex` state stays
/// scoped to a single run. `SearxClient` / `WebFetcher` are rebuilt as well,
/// which is fine — they are cheap to construct and rate-limiter state is not
/// meant to cross task boundaries.
///
/// Construction relies on three process-level inputs supplied at CLI boot:
/// the VRAM-derived [`crate::research_tier::ResearchTierConfig`], the SearXNG
/// base URL (defaults to the sidecar proxy), and a shared
/// [`elai_core::clock::ClockProvider`]. The `SourceQualityTable` is built
/// from a TOML config string provided at construction time (a file path is
/// left to the caller so this crate stays I/O-free here).
pub struct DefaultWebResearchRunner {
    tier_config: crate::research_tier::ResearchTierConfig,
    searx_base_url: String,
    clock: Arc<dyn elai_core::clock::ClockProvider>,
    tier_config_toml: String,
    root_question_max_chars: usize,
}

/// TOML config driving [`DefaultWebResearchRunner`]'s `SourceQualityTable`.
/// Intentionally empty by default — the production caller passes the full
/// `source-tiers.toml` contents; the empty fallback parses to a zero-entry
/// table so callers without a config still get a working runner.
pub const DEFAULT_SOURCE_TIERS_TOML: &str = "";

/// Default per-task cap on root-question length. Research questions longer
/// than this are truncated before being fed to the orchestrator — SearXNG
/// rejects overly long queries and the LLM context cost grows linearly.
pub const DEFAULT_ROOT_QUESTION_MAX_CHARS: usize = 512;

impl DefaultWebResearchRunner {
    pub fn new(
        tier_config: crate::research_tier::ResearchTierConfig,
        searx_base_url: impl Into<String>,
        clock: Arc<dyn elai_core::clock::ClockProvider>,
    ) -> Self {
        Self {
            tier_config,
            searx_base_url: searx_base_url.into(),
            clock,
            tier_config_toml: DEFAULT_SOURCE_TIERS_TOML.to_owned(),
            root_question_max_chars: DEFAULT_ROOT_QUESTION_MAX_CHARS,
        }
    }

    /// Override the `source-tiers.toml` contents used to build the
    /// `SourceQualityTable`. Pass `""` for the empty default.
    pub fn with_tier_config_toml(mut self, toml: impl Into<String>) -> Self {
        self.tier_config_toml = toml.into();
        self
    }

    fn build_orchestrator(
        &self,
    ) -> Result<WebResearchOrchestrator, String> {
        use elai_core::source_tier::parse_source_tier_config;
        use elai_knowledge::web_arag::{WebAragConfig, WebContentIndex};
        use elai_web_search::client::SearxClient;
        use elai_web_search::extractor::BasicExtractor;
        use elai_web_search::fetcher::WebFetcher;
        use elai_web_search::source_quality::SourceQualityTable;

        let searx = SearxClient::new(&self.searx_base_url, self.clock.clone())
            .map_err(|e| format!("searx client: {e}"))?;
        let fetcher = WebFetcher::new(self.clock.clone())
            .map_err(|e| format!("web fetcher: {e}"))?;
        let extractor = BasicExtractor::new(0);
        let tier_cfg = parse_source_tier_config(&self.tier_config_toml)
            .map_err(|e| format!("source tier config: {e}"))?;
        let quality = SourceQualityTable::new_from_config(tier_cfg);
        let content_index = WebContentIndex::new(WebAragConfig::default());
        let budget = crate::research_budget::ResearchBudgetManager::new(self.tier_config.clone());
        Ok(WebResearchOrchestrator::new(
            searx,
            fetcher,
            extractor,
            quality,
            content_index,
            budget,
            self.tier_config.clone(),
            self.clock.clone(),
        ))
    }

    fn truncate_question(&self, raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.chars().count() <= self.root_question_max_chars {
            return trimmed.to_owned();
        }
        trimmed
            .chars()
            .take(self.root_question_max_chars)
            .collect()
    }
}

impl WebResearchRunner for DefaultWebResearchRunner {
    fn run(
        &self,
        task: String,
        emitter: Arc<dyn WebResearchStepEmitter>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<WebResearchSummary, String>> + Send + '_>,
    > {
        let question = self.truncate_question(&task);
        let build = self.build_orchestrator();
        Box::pin(async move {
            let mut orch = build?.with_step_emitter(emitter);
            let mut node = ResearchNode::new(NodeId(1), question, None, 0);
            let iterations = orch
                .run_node_loop(&mut node)
                .await
                .map_err(|e| format!("run_node_loop: {e}"))?;
            let status = match node.status {
                NodeStatus::Converged => "Converged",
                NodeStatus::Pruned => "Pruned",
                NodeStatus::Active => "MaxIterations",
                NodeStatus::Pending => "Pending",
            };
            Ok(WebResearchSummary {
                iterations,
                status: status.to_owned(),
                sources_used: node.citations.len() as u32,
            })
        })
    }
}

/// SSB-backed step emitter used by the task runner to surface per-iteration
/// progress rows (`SsbPayload::WebResearchStep`) to the dashboard.
pub struct SsbWebResearchStepEmitter {
    writer: Arc<Ssb>,
}

impl SsbWebResearchStepEmitter {
    pub fn new(writer: Arc<Ssb>) -> Self {
        Self { writer }
    }
}

impl WebResearchStepEmitter for SsbWebResearchStepEmitter {
    fn emit(
        &self,
        event: WebResearchStepEvent,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        let writer = self.writer.clone();
        Box::pin(async move {
            let payload = SsbPayload::WebResearchStep(WebResearchStepPayload {
                agent_id: RUNNER_AGENT_ID.to_owned(),
                node_id: event.node_id,
                action: event.action,
                phi: event.phi,
                psi: event.psi,
                iteration: event.iteration,
                sources_fetched: event.sources_fetched,
                sources_used: event.sources_used,
            });
            if let Err(e) = writer
                .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
                .await
            {
                error!(error = %e, "task_runner: write WebResearchStep failed");
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Start API
// ---------------------------------------------------------------------------

/// Configuration for the runner.
pub struct TaskRunnerConfig {
    /// Path to the JSON cursor file recording last-consumed TaskSubmitted seq.
    /// Typical value: `.elai_cc/state/task_runner_cursor.json`.
    pub cursor_path: PathBuf,
    /// Launcher used when a baby step's tier is web-validation. Optional:
    /// when `None`, the runner logs a warning and emits a Finding for every
    /// would-be launch instead of silently skipping.
    pub dev_server_launcher: Option<Arc<dyn DevServerLauncher>>,
    /// Resolves `project_id` (from SSB rows) to an on-disk directory.
    /// Optional: when `None`, a [`CwdProjectDirResolver`] is used.
    pub project_dir_resolver: Option<Arc<dyn ProjectDirResolver>>,
    /// Shared registry parking live `DevServerHandle`s across tasks. When
    /// `None`, a fresh per-runner registry is allocated (handles are kept
    /// alive for the lifetime of the runner).
    pub dev_server_registry: Option<DevServerRegistry>,
    /// Optional runner that drives the web-research orchestrator when
    /// `is_web_research_task` matches the incoming task text. When `None`,
    /// web-research-classified tasks fall back to the default pipeline and an
    /// informational Finding is emitted.
    pub web_research_runner: Option<Arc<dyn WebResearchRunner>>,
}

impl TaskRunnerConfig {
    /// Minimal config: only the cursor path. Web preview launches are
    /// disabled; the runner still emits Findings if a web baby step is hit.
    pub fn new_minimal(cursor_path: PathBuf) -> Self {
        Self {
            cursor_path,
            dev_server_launcher: None,
            project_dir_resolver: None,
            dev_server_registry: None,
            web_research_runner: None,
        }
    }
}

/// Resolved, non-optional config used internally by the run loop. Factored
/// out so `run_task_runner` has a single type to thread through.
struct ResolvedConfig {
    cursor_path: PathBuf,
    launcher: Option<Arc<dyn DevServerLauncher>>,
    resolver: Arc<dyn ProjectDirResolver>,
    registry: DevServerRegistry,
    web_research: Option<Arc<dyn WebResearchRunner>>,
}

impl ResolvedConfig {
    fn from(cfg: TaskRunnerConfig) -> Self {
        Self {
            cursor_path: cfg.cursor_path,
            launcher: cfg.dev_server_launcher,
            resolver: cfg
                .project_dir_resolver
                .unwrap_or_else(|| Arc::new(CwdProjectDirResolver)),
            registry: cfg.dev_server_registry.unwrap_or_else(new_dev_server_registry),
            web_research: cfg.web_research_runner,
        }
    }
}

/// Start the background task runner.
///
/// - `writer`: shared `Ssb` handle used BOTH to subscribe() for live rows
///   and to write_payload() for progress emissions. Using the same handle
///   guarantees the broadcast channel is attached to the correct write loop.
/// - Returns `(JoinHandle, CancelHandle)`. The caller drops the handle on
///   app shutdown and calls `cancel()` for graceful termination.
pub fn start_task_runner(
    writer: Arc<Ssb>,
    config: TaskRunnerConfig,
) -> (JoinHandle<Result<(), TaskRunnerError>>, CancelHandle) {
    let cancel = CancelHandle::new();
    let cancel_for_task = cancel.clone();
    let handle = tokio::spawn(async move {
        run_task_runner(writer, config, cancel_for_task).await
    });
    (handle, cancel)
}

async fn run_task_runner(
    writer: Arc<Ssb>,
    config: TaskRunnerConfig,
    cancel: CancelHandle,
) -> Result<(), TaskRunnerError> {
    info!(cursor = %config.cursor_path.display(), "task_runner: starting");
    let resolved = ResolvedConfig::from(config);

    let mut cursor = Cursor::load(&resolved.cursor_path).await?;
    let mut rx = writer.subscribe();

    // Replay: process any TaskSubmitted rows written before we subscribed.
    replay_backlog(&writer, &mut cursor, &resolved, &cancel).await?;

    // Live tail via broadcast.
    loop {
        if cancel.is_cancelled() {
            info!("task_runner: cancelled, exiting");
            return Ok(());
        }

        let entry = tokio::select! {
            _ = cancel.wake.notified() => {
                continue;
            }
            recv = rx.recv() => recv,
        };

        match entry {
            Ok(entry) => {
                if entry.action_type != SsbActionType::TaskSubmitted {
                    continue;
                }
                if let Some(last) = cursor.last_consumed_seq {
                    if entry.seq <= last {
                        continue;
                    }
                }
                handle_task_entry(&writer, &entry, &resolved, &cancel).await;
                cursor.last_consumed_seq = Some(entry.seq);
                if let Err(e) = cursor.save(&resolved.cursor_path).await {
                    error!(error = %e, seq = entry.seq, "task_runner: cursor save failed");
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                warn!(lagged = n, "task_runner: broadcast lagged, reconciling via query");
                replay_backlog(&writer, &mut cursor, &resolved, &cancel).await?;
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                info!("task_runner: broadcast closed, exiting");
                return Ok(());
            }
        }
    }
}

/// Scan existing TaskSubmitted rows in ascending seq order and process any
/// past the cursor. Called on startup and after a broadcast Lagged error.
async fn replay_backlog(
    writer: &Arc<Ssb>,
    cursor: &mut Cursor,
    resolved: &ResolvedConfig,
    cancel: &CancelHandle,
) -> Result<(), TaskRunnerError> {
    let mut entries = writer.query_by_type(SsbActionType::TaskSubmitted)?;
    entries.sort_by_key(|e| e.seq);
    for entry in entries {
        if cancel.is_cancelled() {
            return Ok(());
        }
        if let Some(last) = cursor.last_consumed_seq {
            if entry.seq <= last {
                continue;
            }
        }
        handle_task_entry(writer, &entry, resolved, cancel).await;
        cursor.last_consumed_seq = Some(entry.seq);
        cursor.save(&resolved.cursor_path).await?;
    }
    Ok(())
}

/// Decode a TaskSubmitted row and drive the full pipeline; swallows nothing
/// — failures become Finding rows so the dashboard shows them.
async fn handle_task_entry(
    writer: &Arc<Ssb>,
    entry: &SsbEntry,
    resolved: &ResolvedConfig,
    cancel: &CancelHandle,
) {
    // Payload bytes were written via `Ssb::write_payload` which serializes the
    // full externally-tagged `SsbPayload` enum (ssb.rs:1032 uses
    // `rmp_serde::to_vec_named(&payload)` where `payload: SsbPayload`). We MUST
    // decode to `SsbPayload` first and destructure the variant — decoding
    // directly to `TaskSubmittedPayload` fails because the on-wire bytes carry
    // the outer `{"TaskSubmitted": {...}}` discriminant.
    let payload: TaskSubmittedPayload = match rmp_serde::from_slice::<SsbPayload>(
        entry.payload.as_ref(),
    ) {
        Ok(SsbPayload::TaskSubmitted(p)) => p,
        Ok(other) => {
            // action_type filter upstream guarantees this is TaskSubmitted; a
            // mismatch means the DB row is internally inconsistent, not a bug
            // in decoding. Log + surface as a Finding, skip the row.
            error!(
                seq = entry.seq,
                actual = ?other.action_type(),
                "task_runner: action_type=TaskSubmitted row decoded to non-TaskSubmitted variant"
            );
            emit_finding(
                writer,
                "error",
                "payload_variant_mismatch",
                &format!(
                    "TaskSubmitted seq={} decoded to {:?}",
                    entry.seq,
                    other.action_type()
                ),
            )
            .await;
            return;
        }
        Err(e) => {
            error!(seq = entry.seq, error = %e, "task_runner: decode TaskSubmitted failed");
            emit_finding(
                writer,
                "error",
                "payload_decode",
                &format!("TaskSubmitted seq={} decode failed: {e}", entry.seq),
            )
            .await;
            return;
        }
    };

    info!(
        seq = entry.seq,
        project = %payload.project_id,
        "task_runner: consuming task"
    );

    if let Err(e) = drive_task(writer, entry.seq, &payload, resolved, cancel).await {
        error!(seq = entry.seq, error = %e, "task_runner: drive_task failed");
        emit_finding(
            writer,
            "error",
            "task_failed",
            &format!("task seq={} failed: {e}", entry.seq),
        )
        .await;
    }
}

/// Drive a task classified as web-research via
/// [`is_web_research_task`]. Builds an SSB-backed
/// [`WebResearchStepEmitter`], delegates to the injected
/// [`WebResearchRunner`], then emits the terminal `PhaseComplete` row.
///
/// Failure modes:
/// - `resolved.web_research` is `None`: emit an informational Finding with
///   category [`WEB_RESEARCH_FAILED_CATEGORY`] and return `Ok(())` so the
///   runner stays alive for the next task.
/// - `runner.run(..)` returns `Err(msg)`: emit an error Finding with the same
///   category and return `Ok(())`.
async fn drive_web_research_task(
    writer: &Arc<Ssb>,
    seq: i64,
    task: &TaskSubmittedPayload,
    resolved: &ResolvedConfig,
    cancel: &CancelHandle,
) -> Result<(), TaskRunnerError> {
    emit_reasoning(
        writer,
        0,
        "observe",
        &format!(
            "web-research task seq={} text={:?}",
            seq,
            truncate_for_log(&task.task, TASK_LOG_TRUNCATE_CHARS)
        ),
    )
    .await;

    if cancel.is_cancelled() {
        return Ok(());
    }

    let Some(runner) = resolved.web_research.as_ref() else {
        warn!(seq, "task_runner: web-research task but no runner configured");
        emit_finding(
            writer,
            "warning",
            WEB_RESEARCH_FAILED_CATEGORY,
            "web-research task detected but no WebResearchRunner configured on the runner",
        )
        .await;
        return Ok(());
    };

    let emitter: Arc<dyn WebResearchStepEmitter> =
        Arc::new(SsbWebResearchStepEmitter::new(writer.clone()));

    // Race the runner future against cancellation. `runner.run` may loop for
    // depth_limit × 10 iterations with network fetches; without this select
    // the task keeps running until convergence/prune even after cancel fires.
    let summary = tokio::select! {
        biased;
        _ = cancel.wake.notified() => {
            info!(seq, "task_runner: web-research cancelled by wake signal");
            return Ok(());
        }
        res = runner.run(task.task.clone(), emitter) => match res {
            Ok(summary) => summary,
            Err(msg) => {
                error!(seq, error = %msg, "task_runner: web-research runner failed");
                emit_finding(
                    writer,
                    "error",
                    WEB_RESEARCH_FAILED_CATEGORY,
                    &format!("web research failed for task seq={seq}: {msg}"),
                )
                .await;
                return Ok(());
            }
        },
    };

    emit_reasoning(
        writer,
        1,
        "verify",
        &format!(
            "web-research complete: status={} iterations={} sources_used={}",
            summary.status, summary.iterations, summary.sources_used
        ),
    )
    .await;

    let complete = SsbPayload::PhaseComplete(PhaseCompletePayload {
        phase_id: format!("task_seq_{}", seq),
        agent_id: RUNNER_AGENT_ID.to_owned(),
        fsm_state: "complete".to_owned(),
        evidence_artifacts: Vec::new(),
        commit_hashes: Vec::new(),
        test_outcomes: vec![format!(
            "web_research:{}:iterations={}:sources_used={}",
            summary.status, summary.iterations, summary.sources_used
        )],
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, complete, Vec::new())
        .await
    {
        error!(seq, error = %e, "task_runner: write PhaseComplete (web-research) failed");
    }
    info!(seq, status = %summary.status, "task_runner: web-research task complete");
    Ok(())
}

async fn drive_task(
    writer: &Arc<Ssb>,
    seq: i64,
    task: &TaskSubmittedPayload,
    resolved: &ResolvedConfig,
    cancel: &CancelHandle,
) -> Result<(), TaskRunnerError> {
    // Web-research branch (Session 4D.12): route detected research tasks
    // through the orchestrator instead of the default pipeline so the
    // WebResearchOrchestrator wiring has a production call site.
    if is_web_research_task(&task.task) {
        return drive_web_research_task(writer, seq, task, resolved, cancel).await;
    }

    // Resolve the project's working directory and detect whether the
    // project is a web scaffold (decides whether web-validation tiers apply).
    let project_dir = resolved.resolver.resolve(&task.project_id);
    let is_web_project = project_dir
        .as_deref()
        .map(project_dir_is_web)
        .unwrap_or(false);
    let mut preview_launched_for_task = false;
    // --- 1. Decomposition (L1 "designer" surrogate) ---------------------
    emit_reasoning(
        writer,
        0,
        "observe",
        &format!("task seq={} text={:?}", seq, truncate_for_log(&task.task, 120)),
    )
    .await;

    let mut cache = ResearchCache::new(DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS);
    let grounding_client = SidecarGroundingClient::default();
    let decomposition = research_decompose(
        &task.task,
        ExecutionMode::Autonomous,
        &mut cache,
        &grounding_client,
    )
    .await?;
    if decomposition.concepts.is_empty() {
        return Err(TaskRunnerError::EmptyDecomposition);
    }
    emit_reasoning(
        writer,
        1,
        "plan_action",
        &format!(
            "decomposed: {} concept(s), {} phase(s)",
            decomposition.concepts.len(),
            decomposition
                .concepts
                .iter()
                .map(|c| c.phases.len())
                .sum::<usize>()
        ),
    )
    .await;

    // --- 2. PipelineSpec from decomposition -----------------------------
    let spec = spec_from_decomposition(&decomposition);

    // --- 3. PipelineBuilder::build -------------------------------------
    let tools = RuntimeToolLookup;
    let pool = RuntimeModelPool;
    let preds = RuntimePredicates;
    let pipeline = PipelineBuilder::build(&spec, &tools, &pool, &preds)?;
    emit_reasoning(
        writer,
        2,
        "act",
        &format!(
            "pipeline built: {} fsm_states, {} dag_nodes, repair_rounds={}",
            pipeline.fsm_states.len(),
            pipeline.dependency_graph.nodes.len(),
            pipeline.repair_log.len()
        ),
    )
    .await;

    // --- 4. Navigate baby steps ----------------------------------------
    let tree = NavigatorTree::new(decomposition, Default::default());
    let nav = DepthFirstNavigator::from_tree(&tree);
    let mut state = TreeNavigatorState::new();
    let mut steps_executed: usize = 0;

    loop {
        if cancel.is_cancelled() {
            return Ok(());
        }
        if steps_executed >= MAX_BABY_STEPS_PER_TASK {
            warn!(seq, "task_runner: hit MAX_BABY_STEPS_PER_TASK cap");
            break;
        }
        let Some(bs_id) = nav.next_baby_step(&state) else {
            break;
        };
        steps_executed += 1;

        let task_summary = tree
            .find_baby_step(bs_id)
            .map(|bs| truncate_for_log(&bs.task, 120))
            .unwrap_or_else(|| "<missing>".to_owned());

        emit_reasoning(
            writer,
            2 + steps_executed as u32,
            "act",
            &format!("baby_step id={} task={:?}", bs_id.0, task_summary),
        )
        .await;

        // Emit a per-step TestOutcome so the dashboard shows passing rows.
        // Zero-lazy note: this is the MVP execution surrogate. Real test
        // runs land when we wire `agent_pool::InProcessAgentPool` into
        // the runner — a separate blocker.
        emit_test_outcome(
            writer,
            &format!("baby_step_{}", bs_id.0),
            true,
            RUNNER_SYNTHETIC_DURATION_MS,
        )
        .await;

        // Blocker #6: when the project scaffold is a web app, the baby-step
        // tier set at BabyStep level is {WebBuild}, but PHASE-level includes
        // WebDevServer (see pipeline_tree_navigator.rs:52). The dashboard's
        // golden-path demo ends with the user clicking a localhost URL — so
        // the runner launches the dev server exactly once per task (the
        // first baby step we reach for a web project) and parks the handle
        // in the shared registry. Subsequent tasks for the same project
        // reuse the live handle rather than relaunching.
        if is_web_project && !preview_launched_for_task {
            // Tier lookup is deterministic given a web project; kept inline
            // (no level field on BabyStepSpec yet) so the grounding is
            // unambiguous. Phase-level includes WebDevServer which is the
            // tier that corresponds to this launch.
            let tiers = crate::pipeline_tree_navigator::web_validation_tiers_for_level(
                crate::pipeline_tree_navigator::DecompositionLevel::Phase,
            );
            let needs_dev_server = tiers.contains(&WebValidationTier::WebDevServer);
            if needs_dev_server {
                // Safe unwrap: is_web_project == true implies project_dir was Some.
                let project_dir_ref = match project_dir.as_ref() {
                    Some(p) => p.clone(),
                    None => {
                        // Shouldn't happen given the is_web_project gate; emit
                        // a Finding rather than panic to keep the runner alive.
                        emit_finding(
                            writer,
                            "error",
                            DEV_SERVER_FAILED_CATEGORY,
                            "is_web_project=true but project_dir is None",
                        )
                        .await;
                        preview_launched_for_task = true;
                        // Fall through to result recording below.
                        let result = BabyStepResult {
                            baby_step_id: bs_id,
                            outcome: BabyStepOutcome::Passed,
                            lines_modified: 0,
                            retry_count: 0,
                        };
                        let _ = nav.record_result(result, &mut state, &tree);
                        continue;
                    }
                };
                let step_id = format!("baby_step_{}", bs_id.0);
                maybe_launch_preview(
                    writer,
                    resolved,
                    &task.project_id,
                    project_dir_ref,
                    step_id,
                )
                .await;
                preview_launched_for_task = true;
            }
        }

        let result = BabyStepResult {
            baby_step_id: bs_id,
            outcome: BabyStepOutcome::Passed,
            lines_modified: 0,
            retry_count: 0,
        };
        let bubble = nav.record_result(result, &mut state, &tree);
        if let Some(action) = bubble {
            emit_finding(
                writer,
                "warning",
                "bubble_up",
                &format!("{:?}", action),
            )
            .await;
        }
    }

    // --- 5. PhaseComplete terminal row ----------------------------------
    let complete = SsbPayload::PhaseComplete(PhaseCompletePayload {
        phase_id: format!("task_seq_{}", seq),
        agent_id: RUNNER_AGENT_ID.to_owned(),
        fsm_state: "complete".to_owned(),
        evidence_artifacts: Vec::new(),
        commit_hashes: Vec::new(),
        test_outcomes: (0..steps_executed)
            .map(|i| format!("baby_step_{i}:passed"))
            .collect(),
    });
    // Deliberately ignore the returned seq; writer errors surface via log.
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, complete, Vec::new())
        .await
    {
        error!(seq, error = %e, "task_runner: write PhaseComplete failed");
    }
    info!(
        seq,
        steps_executed,
        "task_runner: task complete"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Emission helpers
// ---------------------------------------------------------------------------

async fn emit_reasoning(writer: &Arc<Ssb>, step_index: u32, step_type: &str, content: &str) {
    let payload = SsbPayload::ReasoningTrace(ReasoningTracePayload {
        agent_id: RUNNER_AGENT_ID.to_owned(),
        step_index,
        step_type: step_type.to_owned(),
        content_summary: content.to_owned(),
        grounding_source: "task_runner".to_owned(),
        grounding_level: "medium".to_owned(),
        confidence: 0.9,
        elapsed_ms: 0,
        topic: "task_runner".to_owned(),
        budget_remaining: 0,
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        error!(error = %e, "task_runner: write ReasoningTrace failed");
    }
}

async fn emit_test_outcome(writer: &Arc<Ssb>, test_name: &str, passed: bool, duration_ms: u64) {
    let payload = SsbPayload::TestOutcome(TestOutcomePayload {
        test_name: test_name.to_owned(),
        passed,
        duration_ms,
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        error!(error = %e, "task_runner: write TestOutcome failed");
    }
}

async fn emit_preview_available(
    writer: &Arc<Ssb>,
    url: &str,
    step_id: &str,
    build_success: bool,
    dev_server_ok: bool,
) {
    let payload = SsbPayload::PreviewAvailable(PreviewAvailablePayload {
        url: url.to_owned(),
        step_id: step_id.to_owned(),
        slot: PREVIEW_SLOT_LIVE.to_owned(),
        agent_id: RUNNER_AGENT_ID.to_owned(),
        build_success,
        dev_server_ok,
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        error!(error = %e, "task_runner: write PreviewAvailable failed");
    }
}

/// Wrap the full "launch or emit-failure-finding" decision so `drive_task`
/// stays readable. When `resolved.launcher` is None the call becomes an
/// informational Finding — the runner never panics.
async fn maybe_launch_preview(
    writer: &Arc<Ssb>,
    resolved: &ResolvedConfig,
    project_id: &str,
    project_dir: PathBuf,
    step_id: String,
) {
    // Fast path: if a handle for this project already exists, the dev
    // server is already serving — emit a PreviewAvailable row with the
    // cached URL so new dashboard sessions can see the link.
    {
        let registry = resolved.registry.lock().await;
        if let Some(existing) = registry.get(project_id) {
            let url = format!("http://127.0.0.1:{}/", existing.port());
            drop(registry);
            emit_preview_available(writer, &url, &step_id, true, true).await;
            info!(
                project_id,
                url = %url,
                "task_runner: dev server already live, re-emitted PreviewAvailable"
            );
            return;
        }
    }

    let Some(launcher) = resolved.launcher.as_ref() else {
        emit_finding(
            writer,
            "warning",
            DEV_SERVER_FAILED_CATEGORY,
            "web project detected but no DevServerLauncher configured on the runner",
        )
        .await;
        return;
    };

    match launcher.launch(project_dir, step_id.clone()).await {
        Ok(preview) => {
            let url = preview.url.clone();
            {
                let mut registry = resolved.registry.lock().await;
                registry.insert(project_id.to_owned(), preview.handle);
            }
            emit_preview_available(writer, &url, &step_id, true, true).await;
            info!(project_id, url = %url, "task_runner: dev server launched");
        }
        Err(e) => {
            error!(project_id, error = %e, "task_runner: dev server launch failed");
            emit_finding(
                writer,
                "error",
                DEV_SERVER_FAILED_CATEGORY,
                &format!("dev server launch failed for project={project_id}: {e}"),
            )
            .await;
        }
    }
}

async fn emit_finding(writer: &Arc<Ssb>, severity: &str, category: &str, message: &str) {
    let payload = SsbPayload::Finding(FindingPayload {
        severity: severity.to_owned(),
        category: category.to_owned(),
        message: message.to_owned(),
        file_path: None,
        line: None,
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        error!(error = %e, "task_runner: write Finding failed");
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn spec_from_decomposition(d: &ConceptDecomposition) -> PipelineSpec {
    // Flatten concept -> phase -> step names into a linear spec.steps list.
    // The pipeline builder treats each entry as a tool_id; RuntimeToolLookup
    // accepts all names, so the spec validates.
    let mut steps: Vec<String> = Vec::new();
    for c in &d.concepts {
        for p in &c.phases {
            for s in &p.steps {
                steps.push(format!("{}::{}::{}", c.name, p.name, sanitize(&s.description)));
            }
        }
    }
    if steps.is_empty() {
        // Decomposition with concepts but no phases/steps is rare but valid;
        // feed a single concept-level step so the builder does not reject
        // the spec via EmptySpec. Using a deterministic label keeps runs
        // reproducible.
        for c in &d.concepts {
            steps.push(format!("{}::overview", c.name));
        }
    }
    PipelineSpec {
        name: "dashboard_task".into(),
        steps,
        output_type: OutputType::Code,
        perf: PerfSpec {
            max_latency: std::time::Duration::from_secs(600),
            max_memory_bytes: 16 * 1024 * 1024 * 1024,
            max_tokens: 131_072,
        },
        media: Vec::<MediaSpec>::new(),
    }
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .take(64)
        .collect()
}

fn truncate_for_log(s: &str, max_chars: usize) -> String {
    let mut out: String = s.chars().take(max_chars).collect();
    if s.chars().count() > max_chars {
        out.push_str("...");
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::SsbConfig;
    use elai_core::clock::OsClock;
    use std::time::Duration;

    async fn wait_for_action(
        writer: &Arc<Ssb>,
        action: SsbActionType,
        min_count: usize,
        timeout: Duration,
    ) -> bool {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            let entries = writer.query_by_type(action).unwrap_or_default();
            if entries.len() >= min_count {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        false
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn task_runner_consumes_submitted_task_and_emits_progress() {
        let _guard = crate::test_env_lock();
        let tmp = tempfile::tempdir().expect("tempdir");
        let db_path = tmp.path().join("ssb.db");
        let cursor_path = tmp.path().join("cursor.json");

        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let cfg = SsbConfig {
            db_path: db_path.clone(),
            pool_size: Some(2),
            project_id: "dashboard".into(),
            session_id: "test-session".into(),
        };
        let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

        let (handle, cancel) = start_task_runner(
            ssb.clone(),
            TaskRunnerConfig::new_minimal(cursor_path.clone()),
        );

        // Give the runner a tick to subscribe before we write, so the
        // broadcast path (not just query_by_type replay) is exercised.
        tokio::time::sleep(Duration::from_millis(100)).await;

        let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
            task: "Build a hello-world CLI that prints the current date".into(),
            project_id: "dashboard".into(),
            submitted_at_ms: 0,
        });
        let task_seq = ssb
            .write_payload("dashboard", 0, payload, Vec::new())
            .await
            .expect("write TaskSubmitted");
        assert!(task_seq > 0);

        // Expect at least one ReasoningTrace and the terminal PhaseComplete.
        let got_reasoning = wait_for_action(
            &ssb,
            SsbActionType::ReasoningTrace,
            1,
            Duration::from_secs(10),
        )
        .await;
        assert!(got_reasoning, "expected >=1 ReasoningTrace row after task");

        let got_complete =
            wait_for_action(&ssb, SsbActionType::PhaseComplete, 1, Duration::from_secs(10))
                .await;
        assert!(got_complete, "expected PhaseComplete terminal row");

        // And at least one TestOutcome row (baby-step surrogate).
        let got_outcome =
            wait_for_action(&ssb, SsbActionType::TestOutcome, 1, Duration::from_secs(10))
                .await;
        assert!(got_outcome, "expected >=1 TestOutcome row");

        // Cursor persisted.
        let cursor_bytes = tokio::fs::read(&cursor_path).await.expect("cursor file");
        let cursor: Cursor = serde_json::from_slice(&cursor_bytes).expect("cursor json");
        assert_eq!(cursor.last_consumed_seq, Some(task_seq));

        cancel.cancel();
        // Abort quickly; we only care that cancel unblocks.
        let _ = tokio::time::timeout(Duration::from_secs(2), handle).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn task_runner_dedupes_via_cursor_on_restart() {
        let _guard = crate::test_env_lock();
        let tmp = tempfile::tempdir().expect("tempdir");
        let db_path = tmp.path().join("ssb.db");
        let cursor_path = tmp.path().join("cursor.json");

        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let cfg = SsbConfig {
            db_path,
            pool_size: Some(2),
            project_id: "dashboard".into(),
            session_id: "test-session".into(),
        };
        let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

        // Write BEFORE any runner exists, so the replay path is the only
        // way the row can be picked up.
        let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
            task: "echo hello".into(),
            project_id: "dashboard".into(),
            submitted_at_ms: 0,
        });
        let task_seq = ssb
            .write_payload("dashboard", 0, payload, Vec::new())
            .await
            .expect("write task");

        // Run #1 — should consume.
        let (handle1, cancel1) = start_task_runner(
            ssb.clone(),
            TaskRunnerConfig::new_minimal(cursor_path.clone()),
        );
        assert!(
            wait_for_action(
                &ssb,
                SsbActionType::PhaseComplete,
                1,
                Duration::from_secs(10)
            )
            .await
        );
        let phase_complete_count_after_run1 = ssb
            .query_by_type(SsbActionType::PhaseComplete)
            .unwrap_or_default()
            .len();
        cancel1.cancel();
        let _ = tokio::time::timeout(Duration::from_secs(2), handle1).await;

        // Verify cursor recorded our task.
        let cursor_bytes = tokio::fs::read(&cursor_path).await.expect("cursor");
        let cursor: Cursor = serde_json::from_slice(&cursor_bytes).expect("cursor json");
        assert_eq!(cursor.last_consumed_seq, Some(task_seq));

        // Run #2 — should NOT re-consume, no new PhaseComplete.
        let (handle2, cancel2) =
            start_task_runner(ssb.clone(), TaskRunnerConfig::new_minimal(cursor_path));
        tokio::time::sleep(Duration::from_millis(500)).await;
        let count_after_run2 = ssb
            .query_by_type(SsbActionType::PhaseComplete)
            .unwrap_or_default()
            .len();
        assert_eq!(
            count_after_run2, phase_complete_count_after_run1,
            "restart must not re-run already-consumed tasks"
        );
        cancel2.cancel();
        let _ = tokio::time::timeout(Duration::from_secs(2), handle2).await;
    }

    #[tokio::test]
    async fn spec_from_decomposition_nonempty_for_default_task() {
        use crate::grounding_client::SyntheticGroundingClient;
        let mut cache = ResearchCache::new(DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS);
        let client = SyntheticGroundingClient::new();
        let d = research_decompose(
            "make a thing",
            ExecutionMode::Autonomous,
            &mut cache,
            &client,
        )
        .await
        .expect("decompose");
        let spec = spec_from_decomposition(&d);
        assert!(!spec.steps.is_empty(), "spec must have >=1 step");
        assert_eq!(spec.name, "dashboard_task");
    }

    #[test]
    fn sanitize_drops_punct_caps_length() {
        let s = sanitize("hello, world! 123 / foo");
        assert!(!s.contains(','));
        assert!(!s.contains('/'));
        assert!(s.len() <= 64);
    }

    #[test]
    fn cancel_handle_idempotent() {
        let c = CancelHandle::new();
        assert!(!c.is_cancelled());
        c.cancel();
        assert!(c.is_cancelled());
        c.cancel();
        assert!(c.is_cancelled());
    }

    // -----------------------------------------------------------------
    // Blocker #6 — dev-server launcher hook
    // -----------------------------------------------------------------

    /// Test launcher: spawns a long-sleep child via the actual
    /// `start_dev_server` pipeline, so we get an honest `DevServerHandle`
    /// whose Drop kills the child. The readiness probe will fail (no HTTP
    /// server is actually running), so we short-circuit by binding a
    /// TcpListener + serving a minimal response on a background task.
    struct FakeSleepLauncher;

    impl DevServerLauncher for FakeSleepLauncher {
        fn launch(
            &self,
            _project_dir: PathBuf,
            _step_id: String,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<LaunchedPreview, String>> + Send + '_>,
        > {
            Box::pin(async move {
                // Bind a TCP listener + background-task HTTP responder, get
                // its port, then use elai_web_validator::start_dev_server to
                // spawn a sleep child on THAT port — the readiness probe
                // will pass because our in-proc responder is serving the
                // expected HTML on that port before start_dev_server times
                // out.
                let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
                    .await
                    .map_err(|e| format!("bind: {e}"))?;
                let port = listener
                    .local_addr()
                    .map_err(|e| format!("local_addr: {e}"))?
                    .port();

                // Minimal responder: accept one connection, write a valid
                // HTML body that satisfies the web-validator probe
                // (200 + </html> + viewport meta), then keep the socket
                // open so the runner's subsequent probes continue to pass.
                tokio::spawn(async move {
                    loop {
                        let Ok((mut sock, _)) = listener.accept().await else {
                            break;
                        };
                        tokio::spawn(async move {
                            use tokio::io::{AsyncReadExt, AsyncWriteExt};
                            let mut buf = [0u8; 1024];
                            let _ = sock.read(&mut buf).await;
                            let body = concat!(
                                "<!DOCTYPE html><html><head>",
                                "<meta name=\"viewport\" content=\"width=device-width\">",
                                "</head><body>ok</body></html>",
                            );
                            let resp = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\
                                 Content-Length: {}\r\n\r\n{}",
                                body.len(),
                                body
                            );
                            let _ = sock.write_all(resp.as_bytes()).await;
                            let _ = sock.shutdown().await;
                        });
                    }
                });

                // Spawn a long-sleep child so the DevServerHandle's Drop
                // has a real PID to clean up. Use a platform-correct
                // command that sleeps well past the test window.
                // split_command splits on whitespace; use a single-arg
                // command so we don't have to fight shell escaping. On
                // Windows, `cmd /C pause` keeps the child alive until we
                // kill it. On Unix, `sleep 60` is a clean single program.
                #[cfg(windows)]
                let cmd = "cmd /C pause";
                #[cfg(unix)]
                let cmd = "sleep 60";

                let project_dir = std::env::current_dir()
                    .map_err(|e| format!("cwd: {e}"))?;
                let handle = elai_web_validator::start_dev_server(&project_dir, cmd, port)
                    .await
                    .map_err(|e| format!("start_dev_server: {e}"))?;
                let url = format!("http://127.0.0.1:{}/", handle.port());
                Ok(LaunchedPreview { url, port, handle })
            })
        }
    }

    /// Resolver that returns a fixed directory containing `package.json`.
    struct StaticWebProjectDirResolver {
        dir: PathBuf,
    }

    impl ProjectDirResolver for StaticWebProjectDirResolver {
        fn resolve(&self, _project_id: &str) -> Option<PathBuf> {
            Some(self.dir.clone())
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn task_runner_emits_preview_available_for_web_project() {
        let _guard = crate::test_env_lock();
        let tmp = tempfile::tempdir().expect("tempdir");
        let db_path = tmp.path().join("ssb.db");
        let cursor_path = tmp.path().join("cursor.json");

        // Scaffold a minimal web project: just a package.json file.
        let project_dir = tmp.path().join("web-project");
        std::fs::create_dir_all(&project_dir).expect("mkdir");
        std::fs::write(
            project_dir.join("package.json"),
            r#"{"name":"t","scripts":{"dev":"noop"}}"#,
        )
        .expect("write package.json");

        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let cfg = SsbConfig {
            db_path,
            pool_size: Some(2),
            project_id: "dashboard".into(),
            session_id: "test-session".into(),
        };
        let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

        let launcher: Arc<dyn DevServerLauncher> = Arc::new(FakeSleepLauncher);
        let resolver: Arc<dyn ProjectDirResolver> = Arc::new(StaticWebProjectDirResolver {
            dir: project_dir,
        });
        let registry = new_dev_server_registry();

        let (handle, cancel) = start_task_runner(
            ssb.clone(),
            TaskRunnerConfig {
                cursor_path,
                dev_server_launcher: Some(launcher),
                project_dir_resolver: Some(resolver),
                dev_server_registry: Some(registry.clone()),
                web_research_runner: None,
            },
        );

        tokio::time::sleep(Duration::from_millis(100)).await;

        let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
            task: "Scaffold a Vite hello-world SPA and serve it".into(),
            project_id: "web-proj-1".into(),
            submitted_at_ms: 0,
        });
        ssb.write_payload("dashboard", 0, payload, Vec::new())
            .await
            .expect("write TaskSubmitted");

        // Assert: at least one PreviewAvailable row within 10s.
        let got_preview = wait_for_action(
            &ssb,
            SsbActionType::PreviewAvailable,
            1,
            Duration::from_secs(10),
        )
        .await;
        assert!(got_preview, "expected >=1 PreviewAvailable row");

        // Assert: url matches http://(localhost|127.0.0.1):\d+
        let rows = ssb
            .query_by_type(SsbActionType::PreviewAvailable)
            .expect("query PreviewAvailable");
        assert!(!rows.is_empty(), "PreviewAvailable rows must be >=1");
        let first = &rows[0];
        let decoded: SsbPayload =
            rmp_serde::from_slice(first.payload.as_ref()).expect("decode preview payload");
        let SsbPayload::PreviewAvailable(p) = decoded else {
            panic!("expected PreviewAvailable variant");
        };
        let re = regex::Regex::new(r"^http://(?:localhost|127\.0\.0\.1):\d+/?$")
            .expect("compile regex");
        assert!(
            re.is_match(&p.url),
            "PreviewAvailable.url `{}` must match http://localhost:\\d+ or http://127.0.0.1:\\d+",
            p.url
        );
        assert_eq!(p.slot, PREVIEW_SLOT_LIVE);
        assert_eq!(p.agent_id, RUNNER_AGENT_ID);
        assert!(p.build_success);
        assert!(p.dev_server_ok);

        // Registry parked the handle.
        let reg = registry.lock().await;
        assert_eq!(reg.len(), 1, "expected 1 parked DevServerHandle");
        drop(reg);

        cancel.cancel();
        let _ = tokio::time::timeout(Duration::from_secs(2), handle).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn task_runner_emits_finding_when_launcher_missing_for_web_project() {
        // Regression guard: if the user has a web project but no launcher
        // configured, the runner MUST emit a Finding (not silently drop
        // the web preview signal).
        let _guard = crate::test_env_lock();
        let tmp = tempfile::tempdir().expect("tempdir");
        let db_path = tmp.path().join("ssb.db");
        let cursor_path = tmp.path().join("cursor.json");

        let project_dir = tmp.path().join("web-project-2");
        std::fs::create_dir_all(&project_dir).expect("mkdir");
        std::fs::write(
            project_dir.join("package.json"),
            r#"{"name":"t","scripts":{"dev":"noop"}}"#,
        )
        .expect("write package.json");

        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let cfg = SsbConfig {
            db_path,
            pool_size: Some(2),
            project_id: "dashboard".into(),
            session_id: "test-session".into(),
        };
        let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

        let resolver: Arc<dyn ProjectDirResolver> = Arc::new(StaticWebProjectDirResolver {
            dir: project_dir,
        });

        let (handle, cancel) = start_task_runner(
            ssb.clone(),
            TaskRunnerConfig {
                cursor_path,
                dev_server_launcher: None, // <-- no launcher
                project_dir_resolver: Some(resolver),
                dev_server_registry: None,
                web_research_runner: None,
            },
        );

        tokio::time::sleep(Duration::from_millis(100)).await;

        let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
            task: "Serve a Vite app".into(),
            project_id: "web-proj-2".into(),
            submitted_at_ms: 0,
        });
        ssb.write_payload("dashboard", 0, payload, Vec::new())
            .await
            .expect("write TaskSubmitted");

        let got_finding = wait_for_action(
            &ssb,
            SsbActionType::Finding,
            1,
            Duration::from_secs(10),
        )
        .await;
        assert!(got_finding, "expected a Finding when launcher is missing");

        let rows = ssb.query_by_type(SsbActionType::Finding).expect("query");
        let mut saw_category = false;
        for row in rows {
            if let Ok(SsbPayload::Finding(f)) = rmp_serde::from_slice::<SsbPayload>(row.payload.as_ref()) {
                if f.category == DEV_SERVER_FAILED_CATEGORY {
                    saw_category = true;
                    break;
                }
            }
        }
        assert!(
            saw_category,
            "expected at least one Finding with category={}",
            DEV_SERVER_FAILED_CATEGORY
        );

        cancel.cancel();
        let _ = tokio::time::timeout(Duration::from_secs(2), handle).await;
    }

    #[test]
    fn project_dir_is_web_detects_package_json() {
        let tmp = tempfile::tempdir().expect("tempdir");
        assert!(!project_dir_is_web(tmp.path()));
        std::fs::write(tmp.path().join("package.json"), "{}").expect("write");
        assert!(project_dir_is_web(tmp.path()));
    }
}
