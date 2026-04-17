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

use elai_core::clock::OsClock;
use elai_reasoning::BasicGrcEngine;
use uuid::Uuid;

use crate::agent_pool::{AgentHandle, AgentRole, AgentStatus, ModelPool};
use crate::checkpoint::{
    AgentCheckpointData, CheckpointManager, GrcHistoryEntry,
};
use crate::recovery::{AgentDeathClassification, RecoveryManager};
use crate::pipeline_builder::{
    PipelineBuilder, PipelineBuilderError, PredicateCompiler, PredicateRef, RuleId, ToolLookup,
};
use elai_rules::predicate::{Verdict, VerdictResult};
use elai_rules::rule_engine::{
    CompiledPredicate, EvaluationContext, FsmPhaseBit, Rule, RuleEngine,
    ALWAYS_INCLUDED_DOMAINS as RULES_ALWAYS_INCLUDED_DOMAINS,
};
use crate::pipeline_tree_navigator::{
    run_integration_checkpoint, BabyStepLocation, BabyStepOutcome, BabyStepResult,
    DecompositionLevel, DepthFirstNavigator, IntegrationRunResult, IntegrationRunner,
    NavigatorTree, NoOpIntegrationRunner, TreeNavigator, TreeNavigatorState,
};
use crate::grounding_client::SidecarGroundingClient;
use crate::research_tree::{
    research_decompose, ConceptDecomposition, ConceptId, ExecutionMode, ExitCriterion,
    IntegrationSpec, MediaSpec, NodeId, NodeStatus, OutputType, PerfSpec, PhaseId,
    PipelineSpec, ResearchCache, ResearchNode, StepId, DEFAULT_CACHE_MAX_ENTRIES,
    DEFAULT_CACHE_TTL_SECONDS,
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

/// Finding category for integration-checkpoint failures at any decomposition
/// level (baby step / step / phase / concept). The dashboard filter keys on
/// this string to surface checkpoint failures in the activity feed.
pub const INTEGRATION_CHECKPOINT_FAILED_CATEGORY: &str = "integration_checkpoint_failed";

/// Finding category for integration-checkpoint bubble-up escalations raised
/// by `run_{step,phase,concept}_integration` when an integration fails.
pub const INTEGRATION_BUBBLE_UP_CATEGORY: &str = "integration_bubble_up";

/// Finding category for rules-engine verdicts emitted at baby-step PRE_GATE.
/// Dashboard filter keys on this string to surface rule violations in the
/// activity feed. Session 4E.14.
pub const RULE_VIOLATION_CATEGORY: &str = "rule_violation";

/// Action label placed on every `EvaluationContext` built for a baby step
/// PRE_GATE evaluation. Matches the rules-engine contract: the `action`
/// string discriminates evaluation call sites (write/execute/modify/...).
/// "baby_step_pre_gate" is unambiguous — no existing rule keys on it.
pub const BABY_STEP_PRE_GATE_ACTION: &str = "baby_step_pre_gate";

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
// Runtime rule evaluation (Session 4E.14)
// ---------------------------------------------------------------------------

/// Bundles a [`RuleEngine`] with its compiled rule set for runtime evaluation
/// at baby-step PRE_GATE. Separate from [`RuntimePredicates`] (which lives on
/// the [`PipelineBuilder`] path for structural DAG validation) because the
/// [`RuleEngine`] trait evaluates against an [`EvaluationContext`] whereas
/// [`PredicateCompiler`] only maps phase names to rule IDs.
///
/// Wrapping in [`Arc`] is intentional: [`CompiledPredicate`] is not [`Clone`]
/// (the `SchemaRequired` variant holds a `jsonschema::Validator`), so we share
/// the rules slice by reference.
pub struct RuleEvaluator {
    engine: Arc<dyn RuleEngine + Send + Sync>,
    compiled_rules: Arc<Vec<(Rule, CompiledPredicate)>>,
}

impl RuleEvaluator {
    /// Construct a new evaluator. The caller keeps ownership of the engine
    /// and compiled rules; both are internally [`Arc`]-wrapped so the runner
    /// can share a single set across every task without re-compilation.
    pub fn new(
        engine: Arc<dyn RuleEngine + Send + Sync>,
        compiled_rules: Arc<Vec<(Rule, CompiledPredicate)>>,
    ) -> Self {
        Self {
            engine,
            compiled_rules,
        }
    }

    /// Number of compiled rules held by the evaluator. Used by observability
    /// callers (tests, dashboard wiring) to assert seed rules were loaded.
    pub fn rules_len(&self) -> usize {
        self.compiled_rules.len()
    }

    /// Evaluate every compiled rule whose `phase_mask` includes `fsm_phase`
    /// AND whose domain either (a) matches one of `file_domains` or
    /// (b) is in `ALWAYS_INCLUDED_DOMAINS` (engineering/security — see
    /// `rule_engine::get_active_rules` docs).
    ///
    /// Evaluation is one-rule-at-a-time via [`std::slice::from_ref`] to avoid
    /// cloning [`CompiledPredicate`]. This preserves the engine's locked
    /// evaluation order *within a single rule* (MetricGate → GrepPattern →
    /// AstPattern → Composite → GraphReachability) but NOT across rules; the
    /// runner's PRE_GATE call site does not need cross-rule ordering because
    /// (i) baby-step rules are independent and (ii) Block short-circuit is
    /// implemented here by breaking the outer loop on the first Block verdict.
    pub fn evaluate_at_pre_gate(
        &self,
        fsm_phase: FsmPhaseBit,
        file_domains: &[&str],
        ctx: &EvaluationContext<'_>,
    ) -> Vec<Verdict> {
        let phase_bit = fsm_phase.mask();
        let mut verdicts: Vec<Verdict> = Vec::new();
        for pair in self.compiled_rules.iter() {
            let (rule, _) = pair;
            let phase_match = (rule.phase_mask & phase_bit) != 0;
            let domain_match = file_domains.contains(&rule.domain.as_str())
                || RULES_ALWAYS_INCLUDED_DOMAINS.contains(&rule.domain.as_str());
            if !phase_match || !domain_match {
                continue;
            }
            match self.engine.evaluate(std::slice::from_ref(pair), ctx) {
                Ok(mut vs) => {
                    let has_block = vs
                        .iter()
                        .any(|v| matches!(v.result, VerdictResult::Block));
                    verdicts.append(&mut vs);
                    if has_block {
                        break;
                    }
                }
                Err(e) => {
                    // Audit BLOCK-fix (4E.14): engine errors MUST surface as a
                    // signal the operator can see. Emit a `Warn` verdict
                    // (picked up by `evaluate_pre_gate_and_emit_findings` →
                    // warning-severity Finding) AND log via `tracing::warn!`
                    // so both the dashboard activity feed and the log
                    // pipeline carry the evidence. Using `Warn` (not `Block`)
                    // keeps the baby step from failing on engine-internal
                    // bugs; the operator can still escalate if needed.
                    tracing::warn!(
                        rule_id = %rule.rule_id,
                        error = %e,
                        "rule engine error during baby-step PRE_GATE",
                    );
                    verdicts.push(Verdict {
                        rule_id: rule.rule_id.clone(),
                        result: VerdictResult::Warn,
                        evidence: format!("rule engine error: {e}"),
                    });
                }
            }
        }
        verdicts
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
// Web-scaffold routing (Session 4E.15)
// ---------------------------------------------------------------------------

/// Category used on `Finding` rows emitted when a web-scaffold task fails
/// or cannot be served (no applier configured).
pub const WEB_SCAFFOLD_FAILED_CATEGORY: &str = "web_scaffold_failed";

/// Built-in template names the detector recognises. Kept in one place so the
/// applier, detector, and tests agree.
pub const WEB_SCAFFOLD_TEMPLATE_REACT: &str = "vite-react";
pub const WEB_SCAFFOLD_TEMPLATE_SVELTE: &str = "vite-svelte";
pub const WEB_SCAFFOLD_TEMPLATE_ASTRO: &str = "astro";
pub const WEB_SCAFFOLD_TEMPLATE_VANILLA: &str = "vanilla";

/// Default template used when the task text contains a web keyword but does
/// not name a specific framework. `vanilla` has no build step so it is the
/// safest fallback across environments.
pub const DEFAULT_SCAFFOLD_TEMPLATE: &str = WEB_SCAFFOLD_TEMPLATE_VANILLA;

/// Lowercase prefixes that trigger the scaffold path. The list is narrow on
/// purpose: "scaffold a ..." is deliberately excluded because existing
/// integration tasks like "Scaffold a Vite hello-world SPA and serve it"
/// expect the dev-server pipeline, not a pure scaffold call. Users who want
/// the scaffold branch say "make me / create / build me / bootstrap / new".
const WEB_SCAFFOLD_PREFIXES: &[&str] = &[
    "make me a ",
    "make me an ",
    "create a ",
    "create an ",
    "build me a ",
    "build me an ",
    "bootstrap a ",
    "bootstrap an ",
    "new ",
];

/// Lowercase keywords that confirm the intent is a WEB scaffold (as opposed
/// to a generic "make me a function" request). The detector requires BOTH a
/// prefix match AND one of these keywords to classify the task.
const WEB_SCAFFOLD_KEYWORDS: &[&str] = &[
    "react",
    "svelte",
    "astro",
    "vue",
    "vanilla",
    "web app",
    "webapp",
    "website",
    "frontend",
    "landing page",
    "static site",
    "spa",
];

/// Heuristic: return true when the task text looks like a request to produce
/// a fresh web-scaffold project that should route through
/// [`ScaffoldApplier`] instead of the default decomposition-plus-navigator
/// pipeline.
///
/// This is the MVP "baby step with tool = scaffold_apply" detector called for
/// by `docs/mvp-wiring-gaps.md` Session 4E.15.
pub fn is_web_scaffold_task(task: &str) -> bool {
    let t = task.trim().to_ascii_lowercase();
    if t.is_empty() {
        return false;
    }
    let has_prefix = WEB_SCAFFOLD_PREFIXES.iter().any(|p| t.starts_with(p));
    if !has_prefix {
        return false;
    }
    WEB_SCAFFOLD_KEYWORDS.iter().any(|k| t.contains(k))
}

/// Map task text to one of the built-in template names. First match wins so
/// the ordering is significant — framework names are checked before the
/// generic `vanilla` fallback.
pub fn detect_scaffold_template(task: &str) -> &'static str {
    let t = task.trim().to_ascii_lowercase();
    if t.contains("react") {
        return WEB_SCAFFOLD_TEMPLATE_REACT;
    }
    if t.contains("svelte") {
        return WEB_SCAFFOLD_TEMPLATE_SVELTE;
    }
    if t.contains("astro") {
        return WEB_SCAFFOLD_TEMPLATE_ASTRO;
    }
    DEFAULT_SCAFFOLD_TEMPLATE
}

/// Pluggable applier that realises a template onto disk. Injected as a trait
/// so the `orchestrator` crate does not gain a direct dep on the `scaffold`
/// crate; the dashboard (which already depends on `scaffold`) supplies the
/// production impl at setup time.
///
/// The impl owns variable generation (project_name / colours / author) so the
/// task runner stays ignorant of template-specific substitution keys.
pub trait ScaffoldApplier: Send + Sync {
    fn apply(
        &self,
        template_name: &str,
        target_dir: &Path,
        task_text: &str,
    ) -> Result<(), String>;
}

/// Drive a task classified as a web-scaffold via [`is_web_scaffold_task`].
/// Resolves the project directory, picks a template, and hands off to the
/// injected [`ScaffoldApplier`]. Emits `PhaseComplete` on success, a
/// `Finding` with category [`WEB_SCAFFOLD_FAILED_CATEGORY`] on failure.
///
/// Failure modes (all non-fatal — the runner stays alive):
/// - `resolved.scaffold_applier` is `None`: emit an informational Finding.
/// - `resolver.resolve(project_id)` is `None`: emit an error Finding.
/// - `ScaffoldApplier::apply` returns `Err`: emit an error Finding with the
///   error message verbatim.
async fn drive_web_scaffold_task(
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
            "web-scaffold task seq={} text={:?}",
            seq,
            truncate_for_log(&task.task, TASK_LOG_TRUNCATE_CHARS)
        ),
    )
    .await;

    if cancel.is_cancelled() {
        return Ok(());
    }

    let Some(applier) = resolved.scaffold_applier.as_ref() else {
        warn!(seq, "task_runner: web-scaffold task but no applier configured");
        emit_finding(
            writer,
            "warning",
            WEB_SCAFFOLD_FAILED_CATEGORY,
            "web-scaffold task detected but no ScaffoldApplier configured on the runner",
        )
        .await;
        return Ok(());
    };

    let Some(project_dir) = resolved.resolver.resolve(&task.project_id) else {
        warn!(
            seq,
            project_id = %task.project_id,
            "task_runner: web-scaffold could not resolve project_id to a directory"
        );
        emit_finding(
            writer,
            "error",
            WEB_SCAFFOLD_FAILED_CATEGORY,
            &format!(
                "web-scaffold failed for task seq={seq}: project_id={:?} did not resolve",
                task.project_id
            ),
        )
        .await;
        return Ok(());
    };

    let template = detect_scaffold_template(&task.task);

    emit_reasoning(
        writer,
        1,
        "plan_action",
        &format!(
            "scaffold target={:?} template={}",
            project_dir.display(),
            template
        ),
    )
    .await;

    match applier.apply(template, &project_dir, &task.task) {
        Ok(()) => {
            emit_reasoning(
                writer,
                2,
                "verify",
                &format!("scaffold complete template={} dir={:?}", template, project_dir.display()),
            )
            .await;
            let complete = SsbPayload::PhaseComplete(PhaseCompletePayload {
                phase_id: format!("task_seq_{}", seq),
                agent_id: RUNNER_AGENT_ID.to_owned(),
                fsm_state: "complete".to_owned(),
                evidence_artifacts: Vec::new(),
                commit_hashes: Vec::new(),
                test_outcomes: vec![format!("web_scaffold:{}:ok", template)],
            });
            if let Err(e) = writer
                .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, complete, Vec::new())
                .await
            {
                error!(seq, error = %e, "task_runner: write PhaseComplete (web-scaffold) failed");
            }
            info!(seq, template, "task_runner: web-scaffold task complete");
        }
        Err(msg) => {
            error!(seq, template, error = %msg, "task_runner: scaffold applier failed");
            emit_finding(
                writer,
                "error",
                WEB_SCAFFOLD_FAILED_CATEGORY,
                &format!(
                    "web-scaffold failed for task seq={seq} template={template}: {msg}"
                ),
            )
            .await;
        }
    }
    Ok(())
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
    /// Optional applier that realises a scaffold template onto disk when
    /// `is_web_scaffold_task` matches. When `None`, scaffold-classified tasks
    /// emit an informational Finding instead of scaffolding.
    pub scaffold_applier: Option<Arc<dyn ScaffoldApplier>>,
    /// Optional runtime rules evaluator (Session 4E.14). When `Some`, the
    /// runner evaluates the compiled rule set against each baby step at
    /// PRE_GATE and emits `Finding` rows for every Block/Warn verdict. When
    /// `None`, the PRE_GATE is a no-op (pre-4E.14 behavior).
    pub rule_evaluator: Option<Arc<RuleEvaluator>>,
    /// Root directory under which per-task checkpoint subdirectories are
    /// written (Session 3C.REDO). When `Some`, the runner owns a
    /// [`CheckpointManager`] and writes a checkpoint row to both disk and the
    /// SSB after each baby-step boundary. When `None`, checkpoint/recovery is
    /// disabled (pre-3C.REDO behavior — task_runner does not checkpoint).
    ///
    /// Typical value: `.elai_cc/sessions/{session_id}/agents`. Each task seq
    /// becomes its own `agent_id` subdirectory under this root.
    pub checkpoint_scratch_dir: Option<PathBuf>,
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
            scaffold_applier: None,
            rule_evaluator: None,
            checkpoint_scratch_dir: None,
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
    scaffold_applier: Option<Arc<dyn ScaffoldApplier>>,
    rule_evaluator: Option<Arc<RuleEvaluator>>,
    /// Session 3C.REDO. Present iff [`TaskRunnerConfig::checkpoint_scratch_dir`]
    /// was supplied. When present, the runner owns a `CheckpointManager` and
    /// a `RecoveryManager` bound to the same clock, and writes checkpoint rows
    /// at each baby-step boundary. On `passed=false` it fires recovery.
    checkpoint: Option<Arc<CheckpointContext>>,
}

/// Session 3C.REDO: bundle the two managers so a single `Option` on
/// `ResolvedConfig` gates the whole checkpoint/recovery surface.
///
/// Both managers share the same `OsClock` so the `CheckpointManager::write`
/// -> `RecoveryManager::read_latest` pair stays in sync. `OsClock` is `Copy`,
/// so the managers do not share state — this struct exists only to keep the
/// `Option` footprint small and make "either both or neither" the only valid
/// runtime state.
struct CheckpointContext {
    checkpoint_manager: CheckpointManager<OsClock>,
    recovery_manager: RecoveryManager<OsClock>,
    scratch_dir: PathBuf,
}

impl ResolvedConfig {
    fn from(cfg: TaskRunnerConfig) -> Self {
        let checkpoint = cfg.checkpoint_scratch_dir.map(|dir| {
            Arc::new(CheckpointContext {
                checkpoint_manager: CheckpointManager::new(OsClock),
                recovery_manager: RecoveryManager::new(OsClock),
                scratch_dir: dir,
            })
        });
        Self {
            cursor_path: cfg.cursor_path,
            launcher: cfg.dev_server_launcher,
            resolver: cfg
                .project_dir_resolver
                .unwrap_or_else(|| Arc::new(CwdProjectDirResolver)),
            registry: cfg.dev_server_registry.unwrap_or_else(new_dev_server_registry),
            web_research: cfg.web_research_runner,
            scaffold_applier: cfg.scaffold_applier,
            rule_evaluator: cfg.rule_evaluator,
            checkpoint,
        }
    }
}

/// Build a deterministic [`AgentHandle`] representing the task_runner itself
/// for a given task `seq`. The `agent_id` is derived from `seq` (UUID low 64
/// bits = seq, high 64 bits = 0) so checkpoints are addressable by seq and
/// the dashboard can recover them on restart.
///
/// Session 3C.REDO: task_runner is not yet a real agent (in-process, no GRC
/// loop), so this handle is a wiring placeholder. Once
/// [`crate::agent_pool::InProcessAgentPool`] has a production spawn site, the
/// handle returned by that path replaces this one.
fn task_runner_handle(seq: i64, scratch_dir: &Path) -> AgentHandle {
    // UUID low 64 bits = seq (as u64 via cast — sign bit preserved). High
    // bits zero. Deterministic and collision-free within a single runner.
    let seq_u64 = seq as u64;
    let mut bytes = [0u8; 16];
    bytes[8..16].copy_from_slice(&seq_u64.to_be_bytes());
    let agent_id = Uuid::from_bytes(bytes);

    let grc_config = elai_reasoning::AgentGrcConfig::default();
    AgentHandle {
        agent_id,
        role: AgentRole::Executor,
        status: AgentStatus::Running,
        task: format!("task_runner_seq_{seq}"),
        active_files: Vec::new(),
        grc_engine: BasicGrcEngine::with_config(grc_config.clone()),
        grc_config,
        scratch_dir: scratch_dir.to_path_buf(),
        sandbox_handle: None,
        child_process: None,
        socket_path: None,
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
    // Web-scaffold branch (Session 4E.15): route detected scaffold tasks
    // through the scaffold crate's apply_template via the injected
    // [`ScaffoldApplier`]. Checked before web-research so a phrase like
    // "make me a React app that compares blake3 vs sha3" falls on the
    // scaffold side (prefix + framework keyword), not the research side.
    if is_web_scaffold_task(&task.task) {
        return drive_web_scaffold_task(writer, seq, task, resolved, cancel).await;
    }

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
    // 4E.13: register IntegrationSpecs at all four decomposition levels so
    // `run_integration_checkpoint` has a real spec to consult at each baby
    // step / step / phase / concept boundary. The NoOpIntegrationRunner
    // reports success for every criterion — baby-step exit criteria become
    // real commands once `agent_pool::InProcessAgentPool` is wired (wire-4A).
    // Until then the wiring is the contract; pass/fail derives from the
    // recorded `BabyStepResult.outcome`.
    let mut tree = NavigatorTree::new(decomposition, Default::default());
    register_runner_integration_specs(&mut tree);
    let nav = DepthFirstNavigator::from_tree(&tree);
    let mut state = TreeNavigatorState::new();
    let mut steps_executed: usize = 0;
    let integration_runner: NoOpIntegrationRunner = NoOpIntegrationRunner;
    let mut last_location: Option<BabyStepLocation> = None;
    // Per-baby-step pass/fail for the terminal PhaseComplete row. Must stay in
    // step with the per-iteration TestOutcome rows — both are surfaced by the
    // dashboard and downstream gate logic, so they cannot disagree.
    let mut baby_step_outcomes: Vec<(u64, bool)> = Vec::new();

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

        // 4E.14: runtime PRE_GATE rules evaluation. Emits one Finding per
        // Block/Warn verdict so the dashboard can surface violations. A
        // `None` evaluator matches pre-4E.14 behavior (no-op).
        let pre_gate_blocked = if let Some(evaluator) = resolved.rule_evaluator.as_ref() {
            evaluate_pre_gate_and_emit_findings(
                writer,
                evaluator,
                bs_id.0,
                &task_summary,
                is_web_project,
            )
            .await
        } else {
            false
        };

        // 4E.13: fire higher-level integration checkpoints when the traversal
        // crosses a step / phase / concept boundary. `last_location` is the
        // *previous* baby step's location; comparing it to the current one
        // tells us which boundaries were just closed off. We fire the outgoing
        // level's checkpoint BEFORE emitting the new baby step's TestOutcome
        // so the dashboard orders rows by (old-boundary-close, new-step-open).
        let current_location = tree.location_index.get(&bs_id).cloned();
        if let (Some(prev), Some(cur)) = (last_location.as_ref(), current_location.as_ref()) {
            emit_level_transitions(writer, &nav, prev, cur, &integration_runner, &mut state, &tree).await;
        }

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
                match project_dir.as_ref() {
                    Some(project_dir_ref) => {
                        let step_id = format!("baby_step_{}", bs_id.0);
                        maybe_launch_preview(
                            writer,
                            resolved,
                            &task.project_id,
                            project_dir_ref.clone(),
                            step_id,
                        )
                        .await;
                        preview_launched_for_task = true;
                    }
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
                        // Fall through to the shared checkpoint path below.
                    }
                }
            }
        }

        let outcome = BabyStepOutcome::Passed;
        let outcome_passed = matches!(outcome, BabyStepOutcome::Passed);
        let result = BabyStepResult {
            baby_step_id: bs_id,
            outcome,
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

        // 4E.13: run the baby-step integration checkpoint and derive the
        // TestOutcome from its aggregate `success`. Replaces the previous
        // unconditional `TestOutcome { passed: true }` emission. If no spec
        // is registered for this baby step the checkpoint is skipped and
        // we fall back to the recorded outcome so the dashboard still sees
        // progress (matches "no integration, no checkpoint" semantics of
        // pipeline_tree_navigator.rs:820).
        // 4E.14: a PRE_GATE Block verdict forces `passed=false`; Warn
        // verdicts are informational and do not flip the outcome.
        let passed = match tree
            .integration_spec(DecompositionLevel::BabyStep, bs_id.0)
            .cloned()
        {
            Some(spec) => {
                let run = run_integration_checkpoint(
                    DecompositionLevel::BabyStep,
                    bs_id.0,
                    &spec,
                    &integration_runner,
                    &mut state,
                );
                emit_checkpoint_outcome(writer, DecompositionLevel::BabyStep, bs_id.0, &run).await;
                run.success && outcome_passed && !pre_gate_blocked
            }
            None => outcome_passed && !pre_gate_blocked,
        };
        emit_test_outcome(
            writer,
            &format!("baby_step_{}", bs_id.0),
            passed,
            RUNNER_SYNTHETIC_DURATION_MS,
        )
        .await;
        baby_step_outcomes.push((bs_id.0, passed));

        // Session 3C.REDO: per-baby-step checkpoint + recovery-on-failure.
        //   * Always write a checkpoint at the boundary so recovery has fresh
        //     grc_history to resume from. Failures inside `write` are logged
        //     and surfaced as a Finding but do NOT fail the task — the runner
        //     stays alive and the dashboard keeps animating.
        //   * When `passed` is false AND the checkpoint write succeeded, fire
        //     `recover_agent` with `Crash` classification. Gating on
        //     `checkpoint_ok` avoids a misleading `recovery_event` row with
        //     `degraded=true` when there is no on-disk checkpoint to
        //     restore from (disk full / rename failure). Recovery emits
        //     `death_event` + `recovery_event` rows to the SSB so the
        //     dashboard can surface the crash/resume pair on a single baby
        //     step. The recovered handle is discarded here: task_runner is
        //     not yet an agent pool, so there is nothing to re-seat. Phase
        //     4A replaces this discard with a real respawn.
        if let Some(cp) = resolved.checkpoint.as_ref() {
            let checkpoint_ok =
                emit_baby_step_checkpoint(writer, cp, seq, bs_id.0, &task_summary, passed).await;
            if !passed && checkpoint_ok {
                emit_baby_step_recovery(writer, cp, seq, bs_id.0).await;
            }
        }

        last_location = current_location;
    }

    // 4E.13: after the last baby step we still need to close off the final
    // step / phase / concept boundaries (no "next" baby step will trigger
    // `emit_level_transitions`). Fire step → phase → concept so step
    // results propagate into the navigator's phase failure counters before
    // the phase integration runs, and phase results into concept counters
    // before the concept integration. `run_integration_checkpoint` is append-
    // only — each call records exactly one `IntegrationCheckpoint`. Because
    // `emit_level_transitions` fires on the OUTGOING location and flush fires
    // on the FINAL location, the two are disjoint for every N >= 1 — no
    // double-recording can occur for a given (level, node_id) pair inside a
    // single `drive_task` invocation.
    if let Some(last) = last_location.as_ref() {
        flush_final_integration_checkpoints(writer, &nav, last, &integration_runner, &mut state, &tree).await;
    }

    // --- 5. PhaseComplete terminal row ----------------------------------
    // `test_outcomes` is derived from the per-baby-step `passed` booleans
    // collected during the loop so the terminal row never disagrees with the
    // individual TestOutcome SSB rows it summarises. Pre-4E.13 this was a
    // hardcoded `:passed` string regardless of outcome.
    let test_outcomes: Vec<String> = baby_step_outcomes
        .iter()
        .map(|(id, passed)| {
            let tag = if *passed { "passed" } else { "failed" };
            format!("baby_step_{id}:{tag}")
        })
        .collect();
    let complete = SsbPayload::PhaseComplete(PhaseCompletePayload {
        phase_id: format!("task_seq_{}", seq),
        agent_id: RUNNER_AGENT_ID.to_owned(),
        fsm_state: "complete".to_owned(),
        evidence_artifacts: Vec::new(),
        commit_hashes: Vec::new(),
        test_outcomes,
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

/// Session 3C.REDO: write a checkpoint row for a single baby step and
/// persist the five msgpack/txt files under the task's scratch subdirectory.
/// Failures are logged and emitted as a Finding so the dashboard surfaces
/// the problem; the runner stays alive.
///
/// Returns `true` iff the checkpoint write + SSB record both succeeded.
/// The caller uses this to gate `emit_baby_step_recovery` — recovery with
/// no on-disk checkpoint would produce a `recovery_event` with
/// `degraded=true`, which misleads the dashboard into believing resumable
/// state exists when it does not.
async fn emit_baby_step_checkpoint(
    writer: &Arc<Ssb>,
    cp: &CheckpointContext,
    seq: i64,
    baby_step_id: u64,
    task_summary: &str,
    passed: bool,
) -> bool {
    let handle = task_runner_handle(seq, &cp.scratch_dir);
    let status_str = if passed { "running" } else { "failed" };
    let data = AgentCheckpointData {
        agent_status: status_str.to_owned(),
        grc_history: vec![GrcHistoryEntry {
            step_type: "act".to_owned(),
            content: format!(
                "baby_step_{baby_step_id} {status}: {summary}",
                status = if passed { "passed" } else { "failed" },
                summary = task_summary,
            ),
            grounding: format!("task_runner:seq={seq}"),
            topic: "task_runner".to_owned(),
        }],
        active_files: Vec::new(),
        tool_results: vec![format!(
            "baby_step_{baby_step_id}:{tag}",
            tag = if passed { "passed" } else { "failed" }
        )],
        output_draft: String::new(),
        channel_references: Vec::new(),
    };
    match cp
        .checkpoint_manager
        .write(&handle, &data, baby_step_id, writer.as_ref())
        .await
    {
        Ok(()) => true,
        Err(e) => {
            error!(
                seq,
                baby_step = baby_step_id,
                error = %e,
                "task_runner: checkpoint write failed"
            );
            emit_finding(
                writer,
                "warning",
                "checkpoint_write_failed",
                &format!("checkpoint write failed for baby_step={baby_step_id}: {e}"),
            )
            .await;
            false
        }
    }
}

/// Session 3C.REDO: fire [`RecoveryManager::recover_agent`] with a `Crash`
/// classification when a baby step's aggregate `passed` is false. Recovery
/// emits `death_event` and `recovery_event` rows to the SSB so the dashboard
/// can pair the crash with its resume point on the same baby step.
///
/// The returned `AgentHandle` is discarded — task_runner is not an agent
/// pool yet (Phase 3 in-process, 4A out-of-process). Once the pool becomes
/// the production spawn site, the handle here is the one that gets re-seated.
async fn emit_baby_step_recovery(
    writer: &Arc<Ssb>,
    cp: &CheckpointContext,
    seq: i64,
    baby_step_id: u64,
) {
    let handle = task_runner_handle(seq, &cp.scratch_dir);
    // Recovery's `prior_step_index` is the step at which the crash happened.
    // For task_runner we use the baby-step id (cast from u64 to i64). On
    // overflow, clamp to `i64::MAX` — the same sentinel `CheckpointManager`
    // uses (checkpoint.rs:252) so both production paths agree on the cap.
    // `RecoveryManager::recover_agent` uses `checked_add(1)` internally and
    // returns `InvariantViolation` on overflow; we surface that as a Finding
    // in the error arm below rather than silently producing an off-by-one.
    let prior = i64::try_from(baby_step_id).unwrap_or(i64::MAX);
    if let Err(e) = cp
        .recovery_manager
        .recover_agent(
            &handle,
            AgentDeathClassification::Crash,
            prior,
            writer.as_ref(),
        )
        .await
    {
        error!(
            seq,
            baby_step = baby_step_id,
            error = %e,
            "task_runner: recover_agent failed"
        );
        emit_finding(
            writer,
            "warning",
            "recovery_failed",
            &format!("recovery failed for baby_step={baby_step_id}: {e}"),
        )
        .await;
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

/// 4E.14: evaluate runtime rules at a baby-step PRE_GATE and emit one
/// `Finding` row per Block/Warn verdict. Returns `true` when any Block
/// fired so the caller can fail the baby step accordingly.
///
/// Contract:
/// - `action` = [`BABY_STEP_PRE_GATE_ACTION`].
/// - `source` = the truncated baby-step task text (what the LLM would have
///   been asked to do). Rules that grep/ast-match rely on this string;
///   today it is a natural-language instruction, not code, which limits
///   what GrepPattern rules can catch. Future wiring replaces `source` with
///   the actual code buffer once `agent_pool::InProcessAgentPool` lands
///   (see task_runner.rs docstring).
/// - `file_path` = derived from the project's web-ness: `web_project.md`
///   for web projects (triggers the "web" domain in
///   [`RuleEvaluator::evaluate_at_pre_gate`]) and an empty string otherwise
///   (engineering/security always-included domains still fire).
/// - `fsm_phase` = EXECUTING, matching the state the runner is in while
///   driving baby steps.
async fn evaluate_pre_gate_and_emit_findings(
    writer: &Arc<Ssb>,
    evaluator: &RuleEvaluator,
    bs_id: u64,
    task_summary: &str,
    is_web_project: bool,
) -> bool {
    let file_path_stub = if is_web_project { "web_project.md" } else { "" };
    let domains: Vec<&str> = if is_web_project { vec!["web"] } else { Vec::new() };
    let ctx = EvaluationContext {
        action: BABY_STEP_PRE_GATE_ACTION,
        source: task_summary,
        file_path: file_path_stub,
        fsm_phase: FsmPhaseBit::EXECUTING,
        metric_values: HashMap::new(),
    };
    let verdicts = evaluator.evaluate_at_pre_gate(FsmPhaseBit::EXECUTING, &domains, &ctx);
    let mut any_block = false;
    for v in &verdicts {
        match v.result {
            VerdictResult::Block => {
                any_block = true;
                emit_finding(
                    writer,
                    "error",
                    RULE_VIOLATION_CATEGORY,
                    &format!("baby_step={bs_id} rule={} BLOCK: {}", v.rule_id, v.evidence),
                )
                .await;
            }
            VerdictResult::Warn => {
                emit_finding(
                    writer,
                    "warning",
                    RULE_VIOLATION_CATEGORY,
                    &format!("baby_step={bs_id} rule={} WARN: {}", v.rule_id, v.evidence),
                )
                .await;
            }
            VerdictResult::Escalate(ref target) => {
                // Audit BLOCK-fix (4E.14): Escalate at PRE_GATE MUST produce
                // an observable signal. Spec 3F.2 defines Escalate as the
                // "route to auditor/human" signal; without a Finding here
                // the verdict is silently dropped. Emit a dedicated Finding
                // with severity="escalate" so downstream consumers (dashboard
                // activity feed, auditor agent router) can filter on it.
                emit_finding(
                    writer,
                    "escalate",
                    RULE_VIOLATION_CATEGORY,
                    &format!(
                        "baby_step={bs_id} rule={} ESCALATE(target={:?}): {}",
                        v.rule_id, target, v.evidence,
                    ),
                )
                .await;
            }
            VerdictResult::Allow => {
                // Allow is the default non-event; no Finding is emitted.
            }
        }
    }
    any_block
}

// ---------------------------------------------------------------------------
// 4E.13 — Integration checkpoint helpers
// ---------------------------------------------------------------------------

/// Walk the tree's `location_index` and register a default IntegrationSpec at
/// every BabyStep / Step / Phase / Concept level. The criteria have no
/// `check_command` (NoOpIntegrationRunner is what we execute them with) —
/// registering them still drives the 4E.11 bookkeeping path: integration
/// checkpoints are appended to `state.integration_checkpoints`, bubble-up
/// counters increment on failure, and the per-level `run_*_integration`
/// convenience methods have a spec to look up. When wire-4A lands we swap
/// in a CommandIntegrationRunner and populate `check_command` with real
/// `cargo check` / `cargo test` invocations.
fn register_runner_integration_specs(tree: &mut NavigatorTree) {
    use std::collections::BTreeSet;

    let mut baby_step_ids: Vec<u64> = Vec::new();
    let mut step_ids: BTreeSet<StepId> = BTreeSet::new();
    let mut phase_ids: BTreeSet<PhaseId> = BTreeSet::new();
    let mut concept_ids: BTreeSet<ConceptId> = BTreeSet::new();
    for loc in tree.location_index.values() {
        baby_step_ids.push(loc.baby_step_id.0);
        step_ids.insert(loc.step_id);
        phase_ids.insert(loc.phase_id);
        concept_ids.insert(loc.concept_id);
    }
    for bs_u64 in &baby_step_ids {
        tree.register_integration_spec(
            DecompositionLevel::BabyStep,
            *bs_u64,
            default_integration_spec("baby_step", *bs_u64),
        );
    }
    for sid in &step_ids {
        tree.register_integration_spec(
            DecompositionLevel::Step,
            sid.0,
            default_integration_spec("step", sid.0),
        );
    }
    for pid in &phase_ids {
        tree.register_integration_spec(
            DecompositionLevel::Phase,
            pid.0,
            default_integration_spec("phase", pid.0),
        );
    }
    for cid in &concept_ids {
        tree.register_integration_spec(
            DecompositionLevel::Concept,
            cid.0,
            default_integration_spec("concept", cid.0),
        );
    }
}

fn default_integration_spec(level_tag: &str, node_id: u64) -> IntegrationSpec {
    IntegrationSpec {
        wiring_target: format!("{level_tag}_{node_id}"),
        callers: vec![RUNNER_AGENT_ID.to_string()],
        exit_criteria: vec![ExitCriterion {
            description: format!("{level_tag} {node_id} integration"),
            check_command: None,
        }],
    }
}

/// Compare the previous baby step's location to the current one and fire
/// the outgoing level's integration checkpoints. Concept-boundary crossings
/// also imply phase- and step-boundary crossings; phase implies step.
/// Order: step → phase → concept, so step failures propagate into phase
/// bubble-up counters before the phase integration runs (and phase into
/// concept). This is NOT "outer-to-inner" or "inner-to-outer" in the tree
/// sense — it is bottom-up, matching the spec's bubble-up cascade.
async fn emit_level_transitions(
    writer: &Arc<Ssb>,
    nav: &DepthFirstNavigator,
    prev: &BabyStepLocation,
    cur: &BabyStepLocation,
    runner: &dyn IntegrationRunner,
    state: &mut TreeNavigatorState,
    tree: &NavigatorTree,
) {
    let concept_changed = prev.concept_id != cur.concept_id;
    let phase_changed = concept_changed || prev.phase_id != cur.phase_id;
    let step_changed = phase_changed || prev.step_id != cur.step_id;

    if step_changed {
        let (result, action) = nav.run_step_integration(prev.step_id, runner, state, tree);
        if let Some(r) = result.as_ref() {
            emit_checkpoint_outcome(writer, DecompositionLevel::Step, prev.step_id.0, r).await;
        }
        if let Some(a) = action.as_ref() {
            emit_integration_bubble_up(writer, DecompositionLevel::Step, prev.step_id.0, a).await;
        }
    }
    if phase_changed {
        let (result, action) = nav.run_phase_integration(prev.phase_id, runner, state, tree);
        if let Some(r) = result.as_ref() {
            emit_checkpoint_outcome(writer, DecompositionLevel::Phase, prev.phase_id.0, r).await;
        }
        if let Some(a) = action.as_ref() {
            emit_integration_bubble_up(writer, DecompositionLevel::Phase, prev.phase_id.0, a).await;
        }
    }
    if concept_changed {
        let (result, action) = nav.run_concept_integration(prev.concept_id, runner, state, tree);
        if let Some(r) = result.as_ref() {
            emit_checkpoint_outcome(writer, DecompositionLevel::Concept, prev.concept_id.0, r).await;
        }
        if let Some(a) = action.as_ref() {
            emit_integration_bubble_up(writer, DecompositionLevel::Concept, prev.concept_id.0, a).await;
        }
    }
}

/// Fire step/phase/concept integration checkpoints for the last baby step's
/// location. Called once after the loop so the final level boundaries close
/// without waiting for a non-existent "next" baby step to trigger them.
async fn flush_final_integration_checkpoints(
    writer: &Arc<Ssb>,
    nav: &DepthFirstNavigator,
    last: &BabyStepLocation,
    runner: &dyn IntegrationRunner,
    state: &mut TreeNavigatorState,
    tree: &NavigatorTree,
) {
    let (step_result, step_action) =
        nav.run_step_integration(last.step_id, runner, state, tree);
    if let Some(r) = step_result.as_ref() {
        emit_checkpoint_outcome(writer, DecompositionLevel::Step, last.step_id.0, r).await;
    }
    if let Some(a) = step_action.as_ref() {
        emit_integration_bubble_up(writer, DecompositionLevel::Step, last.step_id.0, a).await;
    }
    let (phase_result, phase_action) =
        nav.run_phase_integration(last.phase_id, runner, state, tree);
    if let Some(r) = phase_result.as_ref() {
        emit_checkpoint_outcome(writer, DecompositionLevel::Phase, last.phase_id.0, r).await;
    }
    if let Some(a) = phase_action.as_ref() {
        emit_integration_bubble_up(writer, DecompositionLevel::Phase, last.phase_id.0, a).await;
    }
    let (concept_result, concept_action) =
        nav.run_concept_integration(last.concept_id, runner, state, tree);
    if let Some(r) = concept_result.as_ref() {
        emit_checkpoint_outcome(writer, DecompositionLevel::Concept, last.concept_id.0, r).await;
    }
    if let Some(a) = concept_action.as_ref() {
        emit_integration_bubble_up(writer, DecompositionLevel::Concept, last.concept_id.0, a).await;
    }
}

/// Emit a TestOutcome row for an integration-checkpoint run so the dashboard
/// activity feed shows per-level progress. On failure, additionally emit a
/// Finding so the category filter surfaces it outside the test-outcome pane.
async fn emit_checkpoint_outcome(
    writer: &Arc<Ssb>,
    level: DecompositionLevel,
    node_id: u64,
    run: &IntegrationRunResult,
) {
    let level_tag = checkpoint_level_tag(level);
    let test_name = format!("{level_tag}_{node_id}_integration");
    emit_test_outcome(writer, &test_name, run.success, run.total_duration_ms).await;
    if !run.success {
        let reason = run
            .failure_reason
            .as_deref()
            .unwrap_or("integration checkpoint failed");
        emit_finding(
            writer,
            "error",
            INTEGRATION_CHECKPOINT_FAILED_CATEGORY,
            &format!("{level_tag} {node_id}: {reason}"),
        )
        .await;
    }
}

/// Emit a Finding describing an integration-triggered bubble-up action so the
/// dashboard filter can distinguish it from baby-step cascade bubble-ups.
async fn emit_integration_bubble_up(
    writer: &Arc<Ssb>,
    level: DecompositionLevel,
    node_id: u64,
    action: &crate::pipeline_tree_navigator::BubbleUpAction,
) {
    let level_tag = checkpoint_level_tag(level);
    emit_finding(
        writer,
        "warning",
        INTEGRATION_BUBBLE_UP_CATEGORY,
        &format!("{level_tag} {node_id}: {:?}", action),
    )
    .await;
}

fn checkpoint_level_tag(level: DecompositionLevel) -> &'static str {
    match level {
        DecompositionLevel::BabyStep => "baby_step",
        DecompositionLevel::Step => "step",
        DecompositionLevel::Phase => "phase",
        DecompositionLevel::Concept => "concept",
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
                scaffold_applier: None,
                rule_evaluator: None,
                checkpoint_scratch_dir: None,
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
                scaffold_applier: None,
                rule_evaluator: None,
                checkpoint_scratch_dir: None,
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

    // Session 4E.15: scaffold detector + template picker.
    #[test]
    fn is_web_scaffold_task_matches_documented_triggers() {
        assert!(is_web_scaffold_task("Make me a React app"));
        assert!(is_web_scaffold_task("make me an svelte landing page"));
        assert!(is_web_scaffold_task("create a vanilla website"));
        assert!(is_web_scaffold_task("create an astro frontend"));
        assert!(is_web_scaffold_task("build me a simple React SPA"));
        assert!(is_web_scaffold_task("bootstrap a new webapp"));
        assert!(is_web_scaffold_task("new React landing page"));
        assert!(is_web_scaffold_task("   Make me an Astro website   "));

        // Must NOT match — prefix OK, but no web keyword.
        assert!(!is_web_scaffold_task("make me a function"));
        assert!(!is_web_scaffold_task("build me a cli tool"));
        assert!(!is_web_scaffold_task("create a rust crate"));

        // Must NOT match — keyword alone without scaffold prefix.
        assert!(!is_web_scaffold_task("research react apps"));
        assert!(!is_web_scaffold_task("Refactor the frontend"));

        // Must NOT match — existing dev-server integration task stays on the
        // default pipeline because "scaffold a" was deliberately excluded
        // from the prefix list.
        assert!(!is_web_scaffold_task(
            "Scaffold a Vite hello-world SPA and serve it"
        ));

        // Edge cases.
        assert!(!is_web_scaffold_task(""));
        assert!(!is_web_scaffold_task("   "));
    }

    #[test]
    fn detect_scaffold_template_picks_framework_or_vanilla() {
        assert_eq!(
            detect_scaffold_template("Make me a React app"),
            WEB_SCAFFOLD_TEMPLATE_REACT
        );
        assert_eq!(
            detect_scaffold_template("Create a Svelte frontend"),
            WEB_SCAFFOLD_TEMPLATE_SVELTE
        );
        assert_eq!(
            detect_scaffold_template("bootstrap an astro site"),
            WEB_SCAFFOLD_TEMPLATE_ASTRO
        );
        assert_eq!(
            detect_scaffold_template("make me a website"),
            DEFAULT_SCAFFOLD_TEMPLATE
        );
        // First match wins: react appears before svelte in the task text.
        assert_eq!(
            detect_scaffold_template("a react app with a svelte sidecar"),
            WEB_SCAFFOLD_TEMPLATE_REACT
        );
    }

    // Fake applier that records its arguments so the integration test can
    // assert the task runner routed scaffold tasks correctly without
    // touching the real scaffold crate. Uses `std::sync::Mutex` — the
    // trait's `apply` is sync and `tokio::sync::Mutex::blocking_lock`
    // panics when called from an async runtime.
    struct RecordingScaffoldApplier {
        calls: Arc<std::sync::Mutex<Vec<(String, PathBuf, String)>>>,
    }

    impl ScaffoldApplier for RecordingScaffoldApplier {
        fn apply(
            &self,
            template_name: &str,
            target_dir: &Path,
            task_text: &str,
        ) -> Result<(), String> {
            let key = (
                template_name.to_owned(),
                target_dir.to_path_buf(),
                task_text.to_owned(),
            );
            self.calls
                .lock()
                .map_err(|e| format!("recorder mutex poisoned: {e}"))?
                .push(key);
            Ok(())
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn task_runner_routes_web_scaffold_task_through_applier() {
        let _guard = crate::test_env_lock();
        let tmp = tempfile::tempdir().expect("tempdir");
        let db_path = tmp.path().join("ssb.db");
        let cursor_path = tmp.path().join("cursor.json");
        let project_dir = tmp.path().join("new-react-app");

        let clock: Arc<dyn elai_core::clock::ClockProvider> =
            Arc::new(elai_core::clock::OsClock);
        let cfg = SsbConfig {
            db_path,
            pool_size: Some(2),
            project_id: "dashboard".into(),
            session_id: "test-4e15".into(),
        };
        let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

        let resolver: Arc<dyn ProjectDirResolver> = Arc::new(StaticWebProjectDirResolver {
            dir: project_dir.clone(),
        });
        let calls = Arc::new(std::sync::Mutex::new(Vec::<(String, PathBuf, String)>::new()));
        let applier: Arc<dyn ScaffoldApplier> = Arc::new(RecordingScaffoldApplier {
            calls: calls.clone(),
        });

        let (handle, cancel) = start_task_runner(
            ssb.clone(),
            TaskRunnerConfig {
                cursor_path,
                dev_server_launcher: None,
                project_dir_resolver: Some(resolver),
                dev_server_registry: None,
                web_research_runner: None,
                scaffold_applier: Some(applier),
                rule_evaluator: None,
                checkpoint_scratch_dir: None,
            },
        );

        tokio::time::sleep(Duration::from_millis(100)).await;

        let payload = SsbPayload::TaskSubmitted(TaskSubmittedPayload {
            task: "Make me a React app".into(),
            project_id: "web-scaffold-proj-1".into(),
            submitted_at_ms: 0,
        });
        ssb.write_payload("dashboard", 0, payload, Vec::new())
            .await
            .expect("write TaskSubmitted");

        let got = wait_for_action(
            &ssb,
            SsbActionType::PhaseComplete,
            1,
            Duration::from_secs(10),
        )
        .await;
        assert!(got, "expected PhaseComplete row for scaffold task");

        let recorded = calls.lock().expect("calls mutex").clone();
        assert_eq!(recorded.len(), 1, "applier must be invoked exactly once");
        assert_eq!(recorded[0].0, WEB_SCAFFOLD_TEMPLATE_REACT);
        assert_eq!(recorded[0].1, project_dir);
        assert_eq!(recorded[0].2, "Make me a React app");

        // No Finding with scaffold-failed category on the happy path.
        let findings = ssb
            .query_by_type(SsbActionType::Finding)
            .unwrap_or_default();
        for row in findings {
            if let Ok(SsbPayload::Finding(f)) =
                rmp_serde::from_slice::<SsbPayload>(row.payload.as_ref())
            {
                assert_ne!(
                    f.category, WEB_SCAFFOLD_FAILED_CATEGORY,
                    "scaffold happy path must not emit scaffold-failed Finding"
                );
            }
        }

        cancel.cancel();
        let _ = tokio::time::timeout(Duration::from_secs(2), handle).await;
    }

    // -----------------------------------------------------------------
    // 4E.13 — Integration checkpoint wiring
    // -----------------------------------------------------------------

    use crate::research_tree::{
        BabyStepGrounding, ConceptNode, EvidenceLevel, PhaseNode, SourceCitation, SourceQuality,
        StepNode,
    };

    fn make_citation() -> SourceCitation {
        SourceCitation {
            url: "https://example.com".to_string(),
            title: "test".to_string(),
            quality: SourceQuality::Tier1,
            evidence_level: EvidenceLevel::Anecdotal,
            excerpt: "test excerpt".to_string(),
        }
    }

    fn make_grounding() -> BabyStepGrounding {
        BabyStepGrounding {
            concept_name: "c".to_string(),
            phase_name: "p".to_string(),
            step_name: "s".to_string(),
            top_citations: vec![make_citation()],
            compressed_tokens: 10,
        }
    }

    fn make_baby_step(id: u64) -> crate::research_tree::BabyStepSpec {
        crate::research_tree::BabyStepSpec {
            id: crate::research_tree::BabyStepId(id),
            task: format!("bs {id}"),
            estimated_tokens: 100,
            grounding: make_grounding(),
            tdd: None,
            acceptance: None,
        }
    }

    /// Build a tree with 1 concept, 2 phases, 2 steps per phase, 2 baby steps
    /// per step → 8 baby steps spanning 4 steps / 2 phases / 1 concept.
    fn synthetic_tree() -> NavigatorTree {
        let concept = ConceptNode {
            id: ConceptId(1),
            name: "root".to_string(),
            description: "synthetic".to_string(),
            phases: (1..=2u64)
                .map(|pid| PhaseNode {
                    id: PhaseId(pid),
                    name: format!("phase {pid}"),
                    steps: (1..=2u64)
                        .map(|sidx| StepNode {
                            id: StepId(pid * 10 + sidx),
                            description: format!("step p{pid}s{sidx}"),
                            baby_steps: vec![
                                make_baby_step(pid * 100 + sidx * 10 + 1),
                                make_baby_step(pid * 100 + sidx * 10 + 2),
                            ],
                            evidence_level: EvidenceLevel::Anecdotal,
                        })
                        .collect(),
                    depends_on: Vec::new(),
                    research: None,
                })
                .collect(),
            research: None,
        };
        let decomp = ConceptDecomposition {
            idea: "synthetic".to_string(),
            concepts: vec![concept],
            mode: ExecutionMode::Autonomous,
            total_research_calls: 0,
        };
        NavigatorTree::new(decomp, Default::default())
    }

    #[test]
    fn register_runner_integration_specs_populates_all_four_levels() {
        let mut tree = synthetic_tree();
        assert!(
            tree.integration_specs.is_empty(),
            "fresh tree must have no specs registered"
        );

        register_runner_integration_specs(&mut tree);

        // 8 baby steps + 4 steps + 2 phases + 1 concept = 15 specs
        assert_eq!(tree.integration_specs.len(), 15);

        assert!(tree
            .integration_spec(DecompositionLevel::Concept, 1)
            .is_some());
        for pid in [1u64, 2] {
            assert!(tree
                .integration_spec(DecompositionLevel::Phase, pid)
                .is_some());
        }
        for sid in [11u64, 12, 21, 22] {
            assert!(tree
                .integration_spec(DecompositionLevel::Step, sid)
                .is_some());
        }

        let bs_spec = tree
            .integration_spec(DecompositionLevel::BabyStep, 111)
            .expect("baby step 111 spec");
        assert_eq!(bs_spec.wiring_target, "baby_step_111");
        assert_eq!(bs_spec.callers, vec![RUNNER_AGENT_ID.to_string()]);
        assert_eq!(bs_spec.exit_criteria.len(), 1);
        assert!(bs_spec.exit_criteria[0].check_command.is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn emit_level_transitions_fires_step_phase_concept_on_crossings() {
        let _guard = crate::test_env_lock();
        let tmp = tempfile::tempdir().expect("tempdir");
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let ssb = Arc::new(
            Ssb::new(
                SsbConfig {
                    db_path: tmp.path().join("ssb.db"),
                    pool_size: Some(2),
                    project_id: "dashboard".into(),
                    session_id: "4e13".into(),
                },
                clock,
            )
            .expect("ssb"),
        );

        let mut tree = synthetic_tree();
        register_runner_integration_specs(&mut tree);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        let runner = NoOpIntegrationRunner;

        // Cross step only (phase/concept same).
        let prev = BabyStepLocation {
            concept_id: ConceptId(1),
            phase_id: PhaseId(1),
            step_id: StepId(11),
            baby_step_id: crate::research_tree::BabyStepId(111),
        };
        let cur_same_phase = BabyStepLocation {
            step_id: StepId(12),
            ..prev
        };
        emit_level_transitions(&ssb, &nav, &prev, &cur_same_phase, &runner, &mut state, &tree)
            .await;

        // Cross phase (implies step).
        let cur_new_phase = BabyStepLocation {
            phase_id: PhaseId(2),
            step_id: StepId(21),
            ..prev
        };
        emit_level_transitions(&ssb, &nav, &prev, &cur_new_phase, &runner, &mut state, &tree)
            .await;

        assert!(state
            .integration_checkpoints
            .iter()
            .any(|c| c.level == DecompositionLevel::Step));
        assert!(state
            .integration_checkpoints
            .iter()
            .any(|c| c.level == DecompositionLevel::Phase));

        let outcomes = ssb
            .query_by_type(SsbActionType::TestOutcome)
            .unwrap_or_default();
        assert!(
            outcomes.len() >= 2,
            "expected >=2 TestOutcome rows from checkpoints, got {}",
            outcomes.len()
        );

        let mut saw_step = false;
        for row in &outcomes {
            if let Ok(SsbPayload::TestOutcome(p)) =
                rmp_serde::from_slice::<SsbPayload>(row.payload.as_ref())
            {
                if p.test_name.starts_with("step_") && p.test_name.ends_with("_integration") {
                    saw_step = true;
                }
            }
        }
        assert!(saw_step, "expected a step_{{id}}_integration TestOutcome row");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn flush_final_integration_checkpoints_covers_all_three_levels() {
        let _guard = crate::test_env_lock();
        let tmp = tempfile::tempdir().expect("tempdir");
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let ssb = Arc::new(
            Ssb::new(
                SsbConfig {
                    db_path: tmp.path().join("ssb.db"),
                    pool_size: Some(2),
                    project_id: "dashboard".into(),
                    session_id: "4e13-flush".into(),
                },
                clock,
            )
            .expect("ssb"),
        );

        let mut tree = synthetic_tree();
        register_runner_integration_specs(&mut tree);
        let nav = DepthFirstNavigator::from_tree(&tree);
        let mut state = TreeNavigatorState::new();
        let runner = NoOpIntegrationRunner;

        let last = BabyStepLocation {
            concept_id: ConceptId(1),
            phase_id: PhaseId(2),
            step_id: StepId(22),
            baby_step_id: crate::research_tree::BabyStepId(222),
        };
        flush_final_integration_checkpoints(&ssb, &nav, &last, &runner, &mut state, &tree).await;

        let step_count = state
            .integration_checkpoints
            .iter()
            .filter(|c| c.level == DecompositionLevel::Step)
            .count();
        let phase_count = state
            .integration_checkpoints
            .iter()
            .filter(|c| c.level == DecompositionLevel::Phase)
            .count();
        let concept_count = state
            .integration_checkpoints
            .iter()
            .filter(|c| c.level == DecompositionLevel::Concept)
            .count();
        assert_eq!(step_count, 1);
        assert_eq!(phase_count, 1);
        assert_eq!(concept_count, 1);

        let outcomes = ssb
            .query_by_type(SsbActionType::TestOutcome)
            .unwrap_or_default();
        let mut names: Vec<String> = Vec::new();
        for row in &outcomes {
            if let Ok(SsbPayload::TestOutcome(p)) =
                rmp_serde::from_slice::<SsbPayload>(row.payload.as_ref())
            {
                names.push(p.test_name.clone());
            }
        }
        assert!(names.iter().any(|n| n == "step_22_integration"));
        assert!(names.iter().any(|n| n == "phase_2_integration"));
        assert!(names.iter().any(|n| n == "concept_1_integration"));
    }

    #[test]
    fn default_integration_spec_has_expected_shape() {
        let spec = default_integration_spec("phase", 42);
        assert_eq!(spec.wiring_target, "phase_42");
        assert_eq!(spec.callers, vec![RUNNER_AGENT_ID.to_string()]);
        assert_eq!(spec.exit_criteria.len(), 1);
        assert_eq!(
            spec.exit_criteria[0].description,
            "phase 42 integration"
        );
        assert!(spec.exit_criteria[0].check_command.is_none());
    }

    #[test]
    fn checkpoint_level_tag_maps_each_variant() {
        assert_eq!(checkpoint_level_tag(DecompositionLevel::BabyStep), "baby_step");
        assert_eq!(checkpoint_level_tag(DecompositionLevel::Step), "step");
        assert_eq!(checkpoint_level_tag(DecompositionLevel::Phase), "phase");
        assert_eq!(checkpoint_level_tag(DecompositionLevel::Concept), "concept");
    }

    // -----------------------------------------------------------------------
    // Session 3C.REDO: checkpoint + recovery wiring
    // -----------------------------------------------------------------------

    async fn new_ssb_for_3c(tmp: &std::path::Path, session: &str) -> Arc<Ssb> {
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        Arc::new(
            Ssb::new(
                SsbConfig {
                    db_path: tmp.join(format!("{session}.db")),
                    pool_size: Some(2),
                    project_id: "3c-redo".into(),
                    session_id: session.into(),
                },
                clock,
            )
            .expect("ssb"),
        )
    }

    fn new_checkpoint_context(scratch_dir: std::path::PathBuf) -> CheckpointContext {
        CheckpointContext {
            checkpoint_manager: CheckpointManager::new(OsClock),
            recovery_manager: RecoveryManager::new(OsClock),
            scratch_dir,
        }
    }

    #[test]
    fn task_runner_handle_uuid_is_deterministic_from_seq() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let h1 = task_runner_handle(42, tmp.path());
        let h2 = task_runner_handle(42, tmp.path());
        assert_eq!(h1.agent_id, h2.agent_id, "same seq must map to same uuid");
        let h3 = task_runner_handle(43, tmp.path());
        assert_ne!(h1.agent_id, h3.agent_id, "distinct seq must map to distinct uuid");
        assert_eq!(h1.role, AgentRole::Executor);
        assert_eq!(h1.status, AgentStatus::Running);
        assert!(h1.task.contains("task_runner_seq_42"));
    }

    #[test]
    fn resolved_config_without_scratch_dir_has_no_checkpoint() {
        let cfg = TaskRunnerConfig::new_minimal(std::path::PathBuf::from("/tmp/cursor.json"));
        let resolved = ResolvedConfig::from(cfg);
        assert!(resolved.checkpoint.is_none());
    }

    #[test]
    fn resolved_config_with_scratch_dir_owns_checkpoint_context() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut cfg = TaskRunnerConfig::new_minimal(std::path::PathBuf::from("/tmp/cursor.json"));
        cfg.checkpoint_scratch_dir = Some(tmp.path().to_path_buf());
        let resolved = ResolvedConfig::from(cfg);
        let cp = resolved.checkpoint.as_ref().expect("checkpoint context present");
        assert_eq!(cp.scratch_dir, tmp.path());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn emit_baby_step_checkpoint_writes_ssb_row_and_disk_files() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ssb = new_ssb_for_3c(tmp.path(), "cp-row").await;
        let cp = new_checkpoint_context(tmp.path().to_path_buf());
        let seq = 7_i64;
        let bs_id = 101_u64;

        emit_baby_step_checkpoint(&ssb, &cp, seq, bs_id, "task summary", true).await;

        let rows = ssb
            .query_by_type(SsbActionType::Checkpoint)
            .expect("query checkpoint");
        assert_eq!(rows.len(), 1, "expected exactly one Checkpoint row");
        let payload: SsbPayload =
            rmp_serde::from_slice(rows[0].payload.as_ref()).expect("decode payload");
        match payload {
            SsbPayload::Checkpoint(cp_payload) => {
                assert_eq!(cp_payload.step_index, bs_id);
                assert_eq!(cp_payload.file_sizes.len(), 5);
            }
            other => panic!("expected Checkpoint, got {:?}", other.action_type()),
        }

        // 5 files on disk under scratch/{agent_id}/
        let handle = task_runner_handle(seq, &cp.scratch_dir);
        let dir = cp.scratch_dir.join(handle.agent_id.to_string());
        for name in [
            "state.msgpack",
            "grc_history.msgpack",
            "tool_results.msgpack",
            "output_draft.msgpack",
            "checkpoint_ts",
        ] {
            assert!(
                dir.join(name).exists(),
                "expected {name} under {}",
                dir.display()
            );
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn emit_baby_step_recovery_writes_death_and_recovery_rows() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ssb = new_ssb_for_3c(tmp.path(), "recov-row").await;
        let cp = new_checkpoint_context(tmp.path().to_path_buf());
        let seq = 13_i64;
        let bs_id = 42_u64;

        emit_baby_step_recovery(&ssb, &cp, seq, bs_id).await;

        let death = ssb
            .query_by_type(SsbActionType::DeathEvent)
            .expect("query death_event");
        assert_eq!(death.len(), 1, "expected one death_event row");
        let recov = ssb
            .query_by_type(SsbActionType::RecoveryEvent)
            .expect("query recovery_event");
        assert_eq!(recov.len(), 1, "expected one recovery_event row");

        let SsbPayload::DeathEvent(d) =
            rmp_serde::from_slice(death[0].payload.as_ref()).expect("decode death")
        else {
            panic!("expected DeathEvent");
        };
        assert_eq!(d.classification, "crash");
        assert_eq!(d.prior_step_index, bs_id as i64);

        let SsbPayload::RecoveryEvent(r) =
            rmp_serde::from_slice(recov[0].payload.as_ref()).expect("decode recovery")
        else {
            panic!("expected RecoveryEvent");
        };
        assert_eq!(r.classification, "crash");
        assert_eq!(r.recovery_step_index, bs_id as i64 + 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn checkpoint_then_recovery_preserves_agent_id_and_reads_history() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ssb = new_ssb_for_3c(tmp.path(), "cp-recov").await;
        let cp = new_checkpoint_context(tmp.path().to_path_buf());
        let seq = 99_i64;
        let bs_id = 5_u64;

        emit_baby_step_checkpoint(&ssb, &cp, seq, bs_id, "work", false).await;
        emit_baby_step_recovery(&ssb, &cp, seq, bs_id).await;

        // Recovery row's checkpoint_path should match the handle's agent_id
        // derived from the same seq — proves recovery reads the SAME checkpoint
        // the prior call wrote, so post-crash resume is not degraded.
        let handle = task_runner_handle(seq, &cp.scratch_dir);
        let death_rows = ssb
            .query_by_type(SsbActionType::DeathEvent)
            .expect("query death_event");
        let SsbPayload::DeathEvent(d) =
            rmp_serde::from_slice(death_rows[0].payload.as_ref()).expect("decode")
        else {
            panic!("expected DeathEvent");
        };
        assert!(
            d.had_checkpoint,
            "death_event should record that a checkpoint existed at crash time"
        );
        assert!(
            d.checkpoint_path.contains(&handle.agent_id.to_string()),
            "checkpoint_path should reference the same agent_id as the write (agent_id={}, path={})",
            handle.agent_id,
            d.checkpoint_path,
        );
    }
}
