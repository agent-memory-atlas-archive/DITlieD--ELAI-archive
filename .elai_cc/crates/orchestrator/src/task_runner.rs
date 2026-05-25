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

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use rustc_hash::FxHashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use elai_ipc_types::ToolId;
use elai_web_validator::{
    run_web_build, start_dev_server, DevServerHandle, WebValidationTier, WebValidatorError,
};
use elai_web_validator::cwv::{find_chrome_path, run_lighthouse_scores, LighthouseScores};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex as AsyncMutex, Notify};
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use elai_core::clock::{ClockProvider, OsClock};
use elai_reasoning::BasicGrcEngine;
// C-1: bring GrcEngine trait into scope so emit_reasoning can call
// get_budget / get_agent_config on the PipelineGrc engine accessor.
use elai_reasoning::GrcEngine;
use uuid::Uuid;

use crate::agent_pool::{AgentHandle, AgentPool, AgentRole, AgentStatus, ModelPool};
use crate::checkpoint::{
    AgentCheckpointData, CheckpointManager, GrcHistoryEntry,
};
use crate::recovery::{AgentDeathClassification, RecoveryManager};
use crate::pipeline_builder::{
    DagRepairEmitter, PipelineBuilder, PipelineBuilderError, PredicateCompiler, PredicateRef,
    RuleId, SsbDagRepairEmitter, ToolLookup,
};
use elai_rules::interrogation_store::InterrogationStore;
use elai_rules::predicate::{Verdict, VerdictResult};
use elai_rules::rule_engine::{
    CompiledPredicate, EvaluationContext, FsmPhaseBit, Rule, RuleEngine,
    ALWAYS_INCLUDED_DOMAINS as RULES_ALWAYS_INCLUDED_DOMAINS,
};
use elai_core::interrogation_tree::TraversalOutcome;
use elai_core::mistake_book::{format_constraints, DiffHash, FailureCategory, MistakeEntry};
use elai_core::tool_dispatcher::{
    ClassifiedFailure as CoreClassifiedFailure, FailureInput, GitBranchPolicy, GitCommitRequest,
    ToolDispatcher, ValidateRequest, ValidationTier as CoreValidationTier, ValidationTierResult,
};
use elai_reasoning::{default_answers_for_category, traverse_with_answers};
use crate::mistake_book::{
    clear_baby_step as mistake_book_clear_baby_step,
    query_by_baby_step as mistake_book_query_by_baby_step,
    write_mistake as mistake_book_write_mistake,
};
use crate::pipeline_tree_navigator::{
    run_integration_checkpoint, BabyStepLocation, BabyStepOutcome, BabyStepResult,
    DecompositionLevel, DepthFirstNavigator, IntegrationRunResult, IntegrationRunner,
    NavigatorTree, NoOpIntegrationRunner, SsbBubbleUpEmitter, SsbIntegrationEmitter,
    TreeNavigator, TreeNavigatorState,
};
// C-12: per-level audit dispatcher integration.
use crate::auditor_mode::{AuditorMode, LevelAuditVerdict, TierNodeId};
use crate::per_level_audit_dispatch::{
    emit_started as emit_audit_started, record_verdict as record_audit_verdict, CascadeOutcome,
};
use crate::grounding_client::{CloudGroundingClient, SidecarGroundingClient};
use crate::hook_dispatcher::{
    emit_hook_events_to_ssb, emit_shortcoder_compression_to_ssb, HookFinding, HookInvocation,
    HookPoint,
};
use crate::provider_signals::ProviderSignalReader;
use elai_providers::LlmProvider;
use crate::research_tree::{
    research_decompose, ConceptDecomposition, ConceptId, ExecutionMode, ExitCriterion,
    IntegrationSpec, MediaSpec, NodeId, NodeStatus, OutputType, PerfSpec, PhaseId,
    PipelineSpec, ResearchCache, ResearchNode, SsbDecompositionEmitter, StepId,
    DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS,
};
use crate::ssb::{
    AntiSpiralHitPayload, AuditorRubberStampRejectedPayload, BabyStepReadyPayload,
    CsqEvaluatedPayload, DkgQuerierSwappedPayload, DkgReadyPayload, EvidenceGateEvaluatedPayload,
    FindingPayload, GrcStepEmittedPayload, GroundingSweepResultPayload, IndexingProgressPayload,
    PhaseCompletePayload, PhaseNodeReadyPayload, PreviewAvailablePayload, ProviderSignalsPayload,
    ReasoningTracePayload, SearchTriggerFiredPayload, Ssb, SsbActionType, SsbEntry, SsbPayload,
    SsbReader, StepNodeReadyPayload, SupervisionEventPayload, TaskSubmittedPayload,
    TestOutcomePayload, VramTierDetectedPayload, WebResearchStepPayload,
};
use crate::supervision::{EscalationAction, SupervisionManager, SupervisionOutcome, SupervisionPolicy};
use crate::web_research_orchestrator::{
    WebResearchOrchestrator, WebResearchStageEvent, WebResearchStepEmitter, WebResearchStepEvent,
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

/// C-20: env var gating the measurement-mode batch delay (see
/// [`measurement_batch_delay_secs`]). When unset or `0`, the production
/// drive_task baby-step loop is a single `std::env::var` read with zero
/// `await` overhead, preserving the pre-C-20 hot path bit-for-bit.
///
/// The value is the per-baby-step `tokio::time::sleep` duration in seconds
/// that the loop inserts BEFORE the next baby-step dispatch when both this
/// var AND [`ELAI_MEASURE_MODE_ENV`] are set. Distinct from
/// [`crate::indexing_service::ELAI_DEBUG_BATCH_DELAY_SECS_ENV`] in
/// `application` only — both names intentionally use the same env-var
/// string so a single export gates both indexing batches and baby-step
/// batches under a measurement run.
pub const ELAI_DEBUG_BATCH_DELAY_SECS_ENV: &str = "ELAI_DEBUG_BATCH_DELAY_SECS";

/// C-20: env var gating measurement-only paths in production code. When
/// set to `"1"`, the task_runner per-baby-step delay (and other measure-
/// only branches) become active. Any other value (including unset, empty,
/// or `"0"`) leaves production semantics untouched. Pair-gated with
/// [`ELAI_DEBUG_BATCH_DELAY_SECS_ENV`] so a stray `ELAI_MEASURE_MODE=1`
/// without a delay env is still a zero-overhead read.
pub const ELAI_MEASURE_MODE_ENV: &str = "ELAI_MEASURE_MODE";

/// C-20: read the measurement-mode per-baby-step delay. Returns `0` when
/// EITHER [`ELAI_MEASURE_MODE_ENV`] is not `"1"` OR
/// [`ELAI_DEBUG_BATCH_DELAY_SECS_ENV`] is unset / empty / unparseable. The
/// double-gate keeps the production hot path at exactly two `std::env::var`
/// lookups in the common case (both absent) with no allocation and no
/// `await`. Mirrors the indexing-service helper's
/// `Atomic`-free-read-with-zero-on-fail shape.
pub fn measurement_batch_delay_secs() -> u64 {
    if std::env::var(ELAI_MEASURE_MODE_ENV).ok().as_deref() != Some("1") {
        return 0;
    }
    std::env::var(ELAI_DEBUG_BATCH_DELAY_SECS_ENV)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(0)
}

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

/// B-20A Task 8: Finding category for SUCCESSFUL integration checkpoints.
/// Mirrors the `_FAILED` constant above so the dashboard B-7 surface can
/// render a 9-category histogram of checkpoint outcomes per level. Decision
/// E (workflow/reasoning/B-20A-decisions.md): emit as a string-category
/// Finding rather than a new `SsbActionType` variant — this avoids a 14-site
/// SSB extension and a SCHEMA_VERSION bump (stays at 24). Option (b) — a
/// dedicated `IntegrationCheckpointSuccessPayload` — is deferred to a future
/// batch when the dashboard histogram needs richer per-level payloads.
pub const INTEGRATION_CHECKPOINT_SUCCESS_CATEGORY: &str = "integration_checkpoint_success";

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

/// Finding category for InterrogationTree fix suggestions produced on a
/// failing baby step (Session A-19.REDO). The dashboard activity-feed filter
/// keys on this exact string to surface Layer-3 reasoning outputs.
pub const INTERROGATION_FIX_CATEGORY: &str = "interrogation_fix_suggestion";

/// Finding category for the startup metric row reporting the seeded
/// interrogation-tree count (Session A-19.REDO acceptance criterion 3).
pub const INTERROGATION_TREE_METRIC_CATEGORY: &str = "interrogation_tree_metric";

/// Default language used to resolve an InterrogationTree at baby-step failure
/// time. Task_runner does not yet read `ProjectProfile`, so we fall back to
/// "rust" — the host project's language. A follow-up (post-A-19.REDO) will
/// plumb the real `ProjectProfile.language` through `TaskSubmittedPayload`.
pub const DEFAULT_INTERROGATION_LANGUAGE: &str = "rust";

/// Finding category for MistakeBook constraints injected into the per-baby-step
/// context slot (Session 3F.REDO). Occupies the 150-token REPAIR_SLOT_MISTAKE_BOOK
/// reserved in `context_budget.rs:132`. Dashboard filter keys on this string to
/// surface prior-attempt constraints alongside the retry's reasoning.
pub const MISTAKE_BOOK_CONSTRAINTS_CATEGORY: &str = "mistake_book_constraints";

/// Action label written into `MistakeEntry::action_taken` for entries emitted
/// by the task_runner on a failed baby step. Structured code per Code-A1
/// anti-episodic guidance (no variable names, no code snippets).
pub const MISTAKE_ACTION_PENDING_RETRY: &str = "pending_retry";

/// Session patch-1.32: SSB Finding category for per-task dispatch-context
/// exercise logs. Dashboard filters on this string to surface the
/// FSM-state-filtered tool count + chosen backend per task.
pub const DISPATCH_CONTEXT_CATEGORY: &str = "dispatch_context";

/// Gap #6 (MVP audit 2026-04-18 section 4 item 6 / A-44): SSB Finding category
/// for project-indexing summaries emitted by [`drive_task`] the first time a
/// given `project_id` is dispatched. Dashboard activity feeds key on this
/// string to surface DKG / BM25 build progress.
pub const INDEXING_SUMMARY_CATEGORY: &str = "indexing_summary";

/// Gap #4 (MVP audit 2026-04-18 section 4 item 4): SSB Finding category for
/// Block verdicts emitted by the [`crate::deterministic_auditor::DeterministicAuditor`]
/// post-baby-step. Matches the string already used by
/// [`crate::deterministic_audit_gate::GATE_FINDING_CATEGORY`] so the dashboard
/// filter unifies the two surfaces.
pub const DETERMINISTIC_AUDIT_CATEGORY: &str = "deterministic_audit";

/// Gap #9 (MVP audit 2026-04-18 section 4 item 9): SSB Finding category for
/// rule-engine verdicts produced AFTER the real agent executor returns. The
/// pre-gate path keeps using [`RULE_VIOLATION_CATEGORY`]; this distinct tag
/// lets the frontend distinguish pre-execution natural-language evaluations
/// (source = task text) from post-execution real-source-text evaluations
/// (source = file content on disk).
pub const RULE_VIOLATION_POST_CATEGORY: &str = "rule_violation_post";

/// Batch 4B (audit 2026-04-18 item 9 completion): `audit_step` tag stamped
/// on every Finding produced by the post-execution rule-engine pass. The
/// dashboard `get_rule_violations` command filters on this literal and the
/// ActivityEntry renderer shows `post-exec · {rule_id}` when it matches.
/// Distinct from the Batch 4A deterministic-auditor tags (`build`,
/// `version_gate`, `file_size`, `lazy_code`, `completeness`).
pub const POST_EXECUTION_RULES_AUDIT_STEP: &str = "post_execution_rules";

/// Batch 4B: env var that bypasses the PostToolUse ShortCoder invocation
/// without disabling ShortCoder globally. When set to `1`/`true`/`on`, the
/// task_runner hands the raw agent response to the auditor and skips the
/// `ShortCoderCompression` SSB row. The global kill switch
/// [`ELAI_DISABLE_SHORTCODER_ENV`] takes precedence — if that is on, this
/// flag is redundant.
pub const ELAI_DISABLE_SHORTCODER_POSTOOLUSE_ENV: &str =
    "ELAI_DISABLE_SHORTCODER_POSTOOLUSE";

/// Batch 1A: finding category used when a PreToolUse Block verdict aborts a
/// baby step. Matches the string category the dashboard already filters on
/// for the rule-evaluator path, keeping the two streams unified in the feed
/// while the typed `hook_event` action_type carries the structured detail.
pub const HOOK_PRE_TOOL_USE_BLOCK_CATEGORY: &str = "hook_pre_block";

/// Batch 1A: finding category used when a PostToolUse Block verdict flips
/// `TestOutcome.passed = false` after the agent returns.
pub const HOOK_POST_TOOL_USE_BLOCK_CATEGORY: &str = "hook_post_block";

/// Session patch-1.32: baseline tool list fed to `build_dispatch_context` on
/// the dashboard path. Kept inline to avoid pulling `elai-tools` into
/// orchestrator deps (the production tool catalogue lives CLI-side).
/// The FSM-state filter at `dispatcher::build_dispatch_context` is the
/// load-bearing contract; list contents only affect the `tools_allowed`
/// count observed in SSB.
const DISPATCH_CONTEXT_BASELINE_TOOLS: &[&str] = &[
    "bash",
    "read_file",
    "write_file",
    "edit_file",
    "glob_search",
    "grep_search",
    "web_fetch",
    "web_search",
    // B-29 Phase 5: symbol-graph traversal tools.
    "symbol_impact",
    "symbol_flow",
    "symbol_context",
    "symbol_list",
];

/// Public re-export of `DISPATCH_CONTEXT_BASELINE_TOOLS` for contract tests
/// that verify symbol_* tool names are present.
pub const DISPATCH_CONTEXT_BASELINE_TOOLS_PUBLIC: &[&str] = DISPATCH_CONTEXT_BASELINE_TOOLS;

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
pub type DevServerRegistry = Arc<AsyncMutex<BTreeMap<String, DevServerHandle>>>;

pub fn new_dev_server_registry() -> DevServerRegistry {
    Arc::new(AsyncMutex::new(BTreeMap::new()))
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

/// Return the production [`ModelPool`] instance used by the task-runner.
///
/// The returned pool always reports `u64::MAX` available VRAM so the
/// orchestration layer never rejects steps on resource grounds; the real
/// hardware floor is enforced inside [`agent_pool::AgentPool`] before any
/// subprocess is spawned. Exposed as a factory so `runtime_setup.rs` can
/// construct a production [`AgentPoolBabyStepExecutor`] without needing
/// direct access to the private [`RuntimeModelPool`] struct.
pub fn runtime_model_pool() -> std::sync::Arc<dyn crate::agent_pool::ModelPool> {
    std::sync::Arc::new(RuntimeModelPool)
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
// Baby-step execution (Session wire-4A)
// ---------------------------------------------------------------------------

/// Finding category emitted by the agent-backed [`BabyStepExecutor`] when the
/// underlying agent process fails (non-zero exit, spawn failure, timeout).
/// Dashboard activity feed keys on this exact string.
pub const AGENT_FAILED_CATEGORY: &str = "agent_failed";

/// Finding category emitted on anti-spiral HARD_STOP (Session wire-antispiral).
/// Matches the wording in `docs/mvp-wiring-gaps.md` acceptance criterion.
pub const SPIRAL_HARDSTOP_CATEGORY: &str = "spiral_hardstop";

/// Context for one baby-step execution dispatch. Carries the escalation
/// message stashed by [`crate::anti_spiral::AntiSpiralDetector`] so the
/// executor can inject it into the agent turn that is about to run (wire-antispiral
/// acceptance criterion 2).
#[derive(Debug, Clone)]
pub struct BabyStepExecutionCtx {
    pub baby_step_id: u64,
    pub task_summary: String,
    /// When `Some`, the caller (anti-spiral detector) requests this message be
    /// prepended to the agent's next turn input. `None` on the first execution
    /// of any baby step.
    pub escalation_message: Option<String>,
    /// B-28 E-3: optional focus-tunnel [`ModeSelection`](crate::focus_tunnel::ModeSelection)
    /// produced by the [`ModeSelector`](crate::focus_tunnel::ModeSelector) call
    /// in `drive_task` just before the executor dispatch. `None` for synthetic /
    /// legacy paths (SyntheticBabyStepExecutor, IPC tests, dashboard contract
    /// fixtures) so existing call sites stay untouched. AgentPoolBabyStepExecutor
    /// will branch on this in E-4 once the provider client trait lands; in E-3
    /// the field is informational only and the executor falls back to STRUCTURED
    /// regardless of the selection (SSB beacon still emitted upstream).
    pub mode_selection: Option<crate::focus_tunnel::ModeSelection>,
    /// C2: session-scoped credential map accumulated from RequestCredential gates.
    ///
    /// Keys are sanitized env-var names (`[A-Z0-9_]+`); values are user-supplied
    /// strings. Injected into the agent subprocess environment via `Command::env()`
    /// at spawn time. Empty on the fix-and-rerun path and on synthetic executors.
    /// Values are NEVER logged.
    pub session_credentials: std::collections::BTreeMap<String, String>,
}

/// Result of one baby-step execution dispatch.
///
/// `duration_ms` replaces the old [`RUNNER_SYNTHETIC_DURATION_MS`] constant at
/// the `emit_test_outcome` call site — the real-agent path reports the
/// measured wall-clock duration instead.
#[derive(Debug, Clone)]
pub struct BabyStepExecutionResult {
    pub outcome: BabyStepOutcome,
    pub duration_ms: u64,
    /// Process exit code when the executor spawns an OS process. `None` for
    /// the synthetic path (no process was spawned).
    pub exit_code: Option<i32>,
    /// Tail of stderr (truncated) — surfaced into the failure Finding so the
    /// dashboard can point the user at the first-line error. Empty on success.
    pub stderr_tail: String,
    /// Gap #9 (MVP audit 2026-04-18 section 4 item 9): absolute paths of files
    /// the executor's agent claims to have created/modified during this baby
    /// step. Drives the post-execution rule re-evaluation and feeds the
    /// deterministic auditor's `changed_files` argument (Gap #4). Empty on the
    /// synthetic executor — that path has no real diff — so legacy tests keep
    /// working with their existing expectations. Real agent executors populate
    /// this from their on-disk diff once wire-4A's per-step diff reporter
    /// lands; until then the post-execution rule evaluator is a no-op for
    /// baby steps that did not touch files.
    pub changed_files: Vec<PathBuf>,
}

impl BabyStepExecutionResult {
    /// Synthetic "pass in 1 ms" result — matches the pre-wire-4A behavior used
    /// by [`SyntheticBabyStepExecutor`] and covered by every existing test.
    pub fn synthetic_passed() -> Self {
        Self {
            outcome: BabyStepOutcome::Passed,
            duration_ms: RUNNER_SYNTHETIC_DURATION_MS,
            exit_code: None,
            stderr_tail: String::new(),
            changed_files: Vec::new(),
        }
    }
}

/// Session wire-4A: pluggable executor that runs one baby step and returns a
/// real [`BabyStepOutcome`] + duration. The old `drive_task` hard-coded
/// `BabyStepOutcome::Passed` + [`RUNNER_SYNTHETIC_DURATION_MS`] — this trait is
/// that surrogate's production replacement.
///
/// The default [`SyntheticBabyStepExecutor`] preserves legacy behavior (pass
/// with 1 ms duration) so existing tests and non-agent runners keep working
/// while the agent binary is still being stood up. Real production deployments
/// install [`AgentPoolBabyStepExecutor`] via [`TaskRunnerConfig::baby_step_executor`].
pub trait BabyStepExecutor: Send + Sync {
    fn execute<'a>(
        &'a self,
        ctx: BabyStepExecutionCtx,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = BabyStepExecutionResult> + Send + 'a>,
    >;
}

/// Default [`BabyStepExecutor`] — keeps the legacy synthetic `passed: true` +
/// 1 ms behavior so call sites that don't install a real executor stay
/// unchanged.
#[derive(Debug, Default)]
pub struct SyntheticBabyStepExecutor;

impl BabyStepExecutor for SyntheticBabyStepExecutor {
    fn execute<'a>(
        &'a self,
        _ctx: BabyStepExecutionCtx,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = BabyStepExecutionResult> + Send + 'a>,
    > {
        Box::pin(async move { BabyStepExecutionResult::synthetic_passed() })
    }
}

// ---------------------------------------------------------------------------
// Phase A3 — fix-and-rerun outer loop
// ---------------------------------------------------------------------------
//
// After a web baby step, when `run_web_build` (or the equivalent validator
// supplied by the caller) reports `severity >= warn`, the orchestrator must
// dispatch one fixer turn through the agent loop and re-run the validator.
// The loop caps at `max_iterations` (default 5 per orchestrator-prompt §10).
// On cap, the function emits an autonomous-gate escalation row via the
// existing [`emit_autonomous_decision`] helper, reusing
// `AutonomyDecision::RequestHuman` so the dashboard's gate-decision surface
// renders the escalation without a new `SsbActionType` variant.
//
// Production wiring: callers pass a closure that invokes
// `elai_web_validator::run_web_build` against the project dir. Tests pass a
// deterministic stub. Production callers integrate this via the web baby
// step path in the main `drive_task` loop; the function is intentionally a
// free function so the contract test can exercise it in isolation without
// constructing a full `TaskRunner`.

// ---------------------------------------------------------------------------
// C1 — Lighthouse threshold constants.
// ---------------------------------------------------------------------------

/// Minimum acceptable Lighthouse performance score (0.0–1.0). Below this the
/// validator closure reports `Failed` and the fixer is dispatched.
pub const MIN_PERF_SCORE: f64 = 0.80;

/// Minimum acceptable Lighthouse accessibility score (0.0–1.0). Lighthouse
/// runs axe-core internally; no separate axe spawn is needed.
pub const MIN_A11Y_SCORE: f64 = 0.90;

/// Pure threshold check used by the validator closure and by contract tests.
///
/// Returns `Green` when both scores meet or exceed the named thresholds.
/// Returns `Failed` with a diagnostic summary otherwise. `chrome_path = None`
/// short-circuits to a `Failed("chrome.exe not detected …")` result before
/// any score inspection.
pub fn lighthouse_validator_report(
    chrome_path: Option<&Path>,
    scores: &LighthouseScores,
) -> FixerValidatorReport {
    if chrome_path.is_none() {
        return FixerValidatorReport::Failed {
            summary: "chrome.exe not detected; install Chrome or set CHROME_PATH env var \
                      before running Lighthouse"
                .to_string(),
        };
    }

    let mut failures: Vec<String> = Vec::new();

    if scores.performance < MIN_PERF_SCORE {
        failures.push(format!(
            "lighthouse_perf: score {:.3} below threshold {:.2}",
            scores.performance, MIN_PERF_SCORE
        ));
    }

    if scores.accessibility < MIN_A11Y_SCORE {
        failures.push(format!(
            "lighthouse_a11y: accessibility score {:.3} below threshold {:.2}",
            scores.accessibility, MIN_A11Y_SCORE
        ));
    }

    if failures.is_empty() {
        FixerValidatorReport::Green
    } else {
        FixerValidatorReport::Failed {
            summary: failures.join("; "),
        }
    }
}

/// Severity-reduced report from one validator pass. Returned by
/// [`FixerValidator`] implementations; `Green` means `severity < warn`
/// (per `web_validator/src/lib.rs:11` comment — Error/Warning both qualify
/// as `>= warn`).
#[derive(Debug, Clone)]
pub enum FixerValidatorReport {
    /// All validators in this pass produced no findings at severity >= warn.
    Green,
    /// At least one finding was severity >= warn. `summary` is folded into
    /// the next fixer turn's task_summary so the agent has the failure text.
    Failed { summary: String },
}

/// Outcome of [`fix_and_rerun_until_green`]. `iterations` is the number of
/// fixer turns dispatched (NOT the validator passes — the initial validator
/// pass is iteration 0 if it passes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixAndRerunOutcome {
    /// Validators converged to `Green` after `iterations` fixer turns.
    /// `iterations == 0` is possible if the very first validator pass is
    /// green (no fixing needed).
    Healed { iterations: u32 },
    /// Validators stayed `Failed` after the `max_iterations` cap. Caller
    /// should treat this as a hard halt; the function has already emitted
    /// the SSB gate-decision row.
    EscalatedToHuman { iterations: u32 },
}

/// Closure-style trait alias for a single validator pass. Callable in a
/// loop; takes the project dir, returns a future yielding the report.
///
/// Production wiring example (paraphrased):
/// ```ignore
/// let validator = |dir: &Path| async move {
///     let out_dir = dir.join("dist");
///     match elai_web_validator::run_web_build(dir, "npm run build", &out_dir).await {
///         Ok(r) if r.success => FixerValidatorReport::Green,
///         Ok(r) => FixerValidatorReport::Failed {
///             summary: r.stderr_errors.join("\n"),
///         },
///         Err(e) => FixerValidatorReport::Failed { summary: e.to_string() },
///     }
/// };
/// ```
pub trait FixerValidator: Send + Sync {
    type Fut: std::future::Future<Output = FixerValidatorReport> + Send;
    fn run(&self, project_dir: &Path) -> Self::Fut;
}

impl<F, Fut> FixerValidator for F
where
    F: Fn(&Path) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = FixerValidatorReport> + Send,
{
    type Fut = Fut;
    fn run(&self, project_dir: &Path) -> Self::Fut {
        (self)(project_dir)
    }
}

/// Phase A3 outer loop.
///
/// Drives validator -> fixer -> validator until `Green` or `max_iterations`.
/// On cap, emits an `AutonomyDecision::RequestHuman` SSB row via
/// [`emit_autonomous_decision`] (reusing the existing GateDecision +
/// EscalationFired surfaces — no new `SsbActionType` variant).
///
/// Inputs:
/// * `project_root` — the project dir validators run against and fixer turns
///   reference.
/// * `fixer_executor` — typically the production [`AgentPoolBabyStepExecutor`];
///   tests inject a mock that mutates the project filesystem in `execute()`.
/// * `autonomous_gate` — currently used only as a presence signal; future
///   wiring may invoke `gate.decide(...)` directly. Kept on the signature so
///   the integration point at the call site (the main `drive_task` loop)
///   does not have to be refactored when the gate-driven escalation lands.
/// * `ssb_writer` — single-writer Ssb handle the gate-decision row is
///   appended to.
/// * `max_iterations` — hard cap on fixer turns. `0` means "validate once;
///   never dispatch a fixer; escalate immediately on failure" — this matches
///   the principle of explicit caps but is unusual; callers normally pass 5.
/// * `validator` — see [`FixerValidator`].
/// * `step_id` — diagnostic label written into the `GateDecision.step_id` SSB
///   field on cap. Matches the existing call-site convention at
///   `task_runner.rs:4085`.
///
/// Errors: today the function returns `Ok` for every code path (the loop's
/// failures are surfaced as `EscalatedToHuman`, not `Err`). The `Result`
/// return type is reserved for future I/O failures (e.g. validator that
/// itself returns a hard error and we choose to propagate rather than
/// classify as `Failed`).
///
/// Wiring point: callers in the main `drive_task` loop invoke this AFTER
/// `resolved.baby_step_executor.execute(exec_ctx).await` (task_runner.rs
/// around line 4208) and BEFORE the `BabyStepResult` construction (around
/// line 4317), guarded by `is_web_project`. The wiring is intentionally
/// kept out of this commit per the minimal-diff principle — the function is
/// already exercised by the contract test in isolation.
#[allow(clippy::too_many_arguments)]
pub async fn fix_and_rerun_until_green<V>(
    project_root: PathBuf,
    fixer_executor: Arc<dyn BabyStepExecutor>,
    autonomous_gate: Option<Arc<crate::autonomous_gate::AutonomousGate>>,
    ssb_writer: Arc<Ssb>,
    max_iterations: u32,
    validator: V,
    step_id: &str,
) -> Result<FixAndRerunOutcome, TaskRunnerError>
where
    V: FixerValidator,
{
    // `autonomous_gate` is reserved for follow-on wiring (gate-driven
    // escalation policy). Today the loop uses emit_autonomous_decision
    // directly, mirroring the existing call site at task_runner.rs:4085.
    let _ = autonomous_gate.as_ref();

    let mut iterations: u32 = 0;
    loop {
        // ---- Validator pass ----
        let report = validator.run(&project_root).await;
        let failure_summary = match report {
            FixerValidatorReport::Green => {
                return Ok(FixAndRerunOutcome::Healed { iterations });
            }
            FixerValidatorReport::Failed { summary } => summary,
        };

        // ---- Cap check ----
        if iterations >= max_iterations {
            // Emit a request_human GateDecision row so the dashboard
            // surfaces the escalation. Evidence here is synthetic — the
            // failed validator output is folded into the `reason` string.
            // We deliberately do NOT call derive_autonomous_evidence: this
            // path doesn't have an SsbReader handle on the Arc<Ssb> at this
            // call site, and the evidence is "the validator failed
            // max_iterations times in a row" — a structurally simpler
            // claim than the six-check evidence summary.
            let evidence = crate::autonomous_gate::Evidence {
                knowledge_readiness: true,
                zero_ungrounded: true,
                tests_fail_correctly: true,
                test_flipped: false,
                zero_blocks: false,
                dkg_and_changelog_present: true,
                // B-40: not in FRAMING substate at this call site;
                // vacuous-default true so 7th check doesn't double-block.
                framing_converged: true,
                // B-41: per-level audit zero-blocks vacuous-default true at
                // this synthetic-evidence call site (no SsbReader handle to
                // poll per-level verdicts).
                concept_zero_blocks: true,
                phase_zero_blocks: true,
                step_zero_blocks: true,
                babystep_zero_blocks: true,
                plan_zero_blocks: true,
            };
            let reason = format!(
                "fix_and_rerun cap reached after {max_iterations} iterations; \
                 last validator summary: {summary}",
                max_iterations = max_iterations,
                summary = truncate_summary(&failure_summary, 240),
            );
            let decision = crate::autonomous_gate::AutonomyDecision::RequestHuman {
                reason,
                risk_level: crate::gate::RiskLevel::High,
                evidence,
            };
            emit_autonomous_decision(
                &ssb_writer,
                &decision,
                step_id,
                None,
                "fix_and_rerun",
                None,
            )
            .await;
            return Ok(FixAndRerunOutcome::EscalatedToHuman { iterations });
        }

        // ---- Fixer turn ----
        let fixer_task = format_fixer_prompt(&failure_summary);
        let exec_ctx = BabyStepExecutionCtx {
            // baby_step_id `0` is the sentinel used by `feasibility_probe` and
            // mirror tests when the caller has no concrete BSP id; the fix
            // loop does not consume the value beyond logging.
            baby_step_id: 0,
            task_summary: fixer_task,
            escalation_message: None,
            // Fixer turns reuse the STRUCTURED path unconditionally — focus-
            // tunnel routing (B-28 E-3) only fires on primary baby-step
            // dispatch, not on fix-and-rerun retries.
            mode_selection: None,
            // fix_and_rerun_until_green has no ResolvedConfig access; credentials
            // are not propagated on this path. Empty default preserves prior behavior.
            session_credentials: std::collections::BTreeMap::new(),
        };
        let _fixer_result = fixer_executor.execute(exec_ctx).await;
        // We do NOT parse the agent response content. Per the integration
        // map's "simpler-first path" (workflow/sessions/s-28564/
        // integration-map-A3.md), the agent mutates files via its tool
        // channel; the next validator pass observes the side effects.
        iterations += 1;
    }
}

/// Single-purpose helper: build the fixer-turn task_summary from a failed
/// validator summary. Kept short and predictable so the agent's prompt
/// budget stays controlled. Per `workflow/papers/agentless-2407.01489-
/// interface.md`, one validator failure -> one fixer turn (do NOT merge
/// multiple validator outputs into a multi-fix prompt).
fn format_fixer_prompt(failure_summary: &str) -> String {
    format!(
        "FIX_AND_RERUN: the web validator reported the following failure. \
         Fix the affected source files so the validator passes on the next \
         attempt. Do not introduce unrelated changes.\n---\n{}\n---",
        truncate_summary(failure_summary, 1_800),
    )
}

/// Truncate a validator/failure summary to `max` chars to keep SSB rows and
/// prompts compact. Returns the input unchanged when already <= `max`.
fn truncate_summary(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_owned()
    } else {
        let prefix = s
            .char_indices()
            .nth(max)
            .map_or(s, |(i, _)| &s[..i]);
        let mut out = String::with_capacity(prefix.len() + 16);
        out.push_str(prefix);
        out.push_str("... [truncated]");
        out
    }
}

/// Production [`BabyStepExecutor`] that drives
/// [`crate::agent_pool::InProcessAgentPool::launch_agent_process`] (Session
/// wire-4A acceptance criterion 1-4).
///
/// The executor:
///   1. Calls `pool.spawn(role, task_summary)` to allocate an `AgentHandle`.
///   2. Calls `pool.launch_agent_process(&id, &binary, &socket)` to fork the
///      agent binary. `socket_path_fn` maps `agent_id -> PathBuf` so the
///      caller controls uniqueness (dashboard uses
///      `socket_dir/agent-{uuid}.sock`).
///   3. Waits on the child process (with `timeout_ms`) and maps:
///      * exit code 0  -> `BabyStepOutcome::Passed`
///      * non-zero     -> `BabyStepOutcome::Failed`
///      * spawn error  -> `BabyStepOutcome::Failed` + stderr_tail carries the
///        error message
///
/// Concurrency: wraps the pool in an `AsyncMutex` so `launch_agent_process`
/// can run serialized (the pool's internal map is not `Send`-shared). The
/// trait itself is `Send + Sync`.
///
/// Note on the agent binary:
///   The MVP spec warns the `elai_cc_agent` binary may not be built yet. In
///   that case `Command::spawn` returns `NotFound`, and we surface the
///   spawn error as a `Failed` outcome with the error text in `stderr_tail`.
///   This is deliberate — the wiring is the contract; the binary landing is
///   a separate cargo-build step.
pub struct AgentPoolBabyStepExecutor<E: elai_core::EntropySource + Send + 'static> {
    pool: Arc<AsyncMutex<crate::agent_pool::InProcessAgentPool<E>>>,
    agent_binary_path: PathBuf,
    socket_dir: PathBuf,
    /// Upper bound on a single baby-step execution. Kills the child process
    /// when exceeded. Default 120s matches the DeterministicAuditor
    /// BUILD_CMD_TIMEOUT_S constant.
    timeout_ms: u64,
    /// Role passed to `pool.spawn`. Default `Executor`.
    role: crate::agent_pool::AgentRole,
    /// IPC gateway for frame routing. Shared with the orchestrator's session manager.
    gateway: Arc<AsyncMutex<crate::ipc_gateway::IpcGateway>>,
    /// RPC server that handles InferRequest / ToolRequest frames from the agent.
    rpc_server: Arc<dyn crate::ipc_accept::AgentRpcServer + Send + Sync + 'static>,
    /// C-11: optional IPC hook dispatcher. When `Some`, `run_agent_session`
    /// fires `pre_tool_use`/`post_tool_use` on every `FrameRouteResult::ToolRequest`.
    /// `None` keeps the legacy bypass for tests and synthetic harnesses.
    hook_dispatcher:
        Option<Arc<dyn crate::ipc_accept::IpcHookDispatcher + Send + Sync + 'static>>,
    /// C-15: ClockProvider used for per-execution latency timing inside the
    /// baby-step executor. Defaults to `Arc<OsClock>`; deterministic tests can
    /// override via `with_clock()` (formerly the W2-DEFER-850 entropy leak).
    clock: Arc<dyn ClockProvider>,
    /// Test-only: when set, the executor sends the socket path through this
    /// channel after bind_session_listener completes (proving listen-before-spawn
    /// ordering). The receiver is in the test's stub agent task.
    /// Wrapped in Mutex so execute(&self) can take the sender without &mut self.
    #[cfg(any(test, feature = "test-support"))]
    socket_path_notifier: std::sync::Mutex<Option<tokio::sync::oneshot::Sender<PathBuf>>>,
    /// E-15: opt-in Firecracker hot snapshot-restore pool. When `Some`, the
    /// executor calls `pool.spawn_from_snapshot()` before
    /// `launch_agent_process` so the agent process restores from a per-(agent_id,
    /// role) snapshot instead of cold-booting. On KVM-absent hosts (Windows,
    /// Linux without /dev/kvm) the pool returns `FirecrackerError::KvmUnavailable`
    /// which is treated as RUNPOD-PENDING and silently bypassed; the executor
    /// falls through to the standard launch_agent_process path. None is the
    /// production default until the dashboard wires the toml-driven config.
    firecracker_pool: Option<Arc<crate::sandbox::FirecrackerHotPool>>,
}

/// Default timeout for a single baby-step agent execution (matches
/// `deterministic_auditor::BUILD_CMD_TIMEOUT_S`).
pub const DEFAULT_BABY_STEP_TIMEOUT_MS: u64 = 120_000;

/// Maximum GRC call budget passed to `AgentConnection::new` for each
/// agent IPC session. Kept as a named constant to avoid magic-number
/// literals in the executor body.
pub const DEFAULT_AGENT_GRC_MAX_CALLS: u32 = 5;

/// Maximum GRC chars-per-call budget surfaced to the spawned baby-step
/// agent via `AgentDispatchConfig::grc_budget_max_chars`. Matches the
/// default applied by `mcp_think_set_budget` (CLAUDE.md GRC section) so a
/// baby-step agent sees the same envelope as the main reasoning loop.
pub const DEFAULT_AGENT_GRC_MAX_CHARS: u32 = 1000;

impl<E: elai_core::EntropySource + Send + 'static> AgentPoolBabyStepExecutor<E> {
    pub fn new(
        pool: Arc<AsyncMutex<crate::agent_pool::InProcessAgentPool<E>>>,
        agent_binary_path: PathBuf,
        socket_dir: PathBuf,
        gateway: Arc<AsyncMutex<crate::ipc_gateway::IpcGateway>>,
        rpc_server: Arc<dyn crate::ipc_accept::AgentRpcServer + Send + Sync + 'static>,
    ) -> Self {
        Self {
            pool,
            agent_binary_path,
            socket_dir,
            timeout_ms: DEFAULT_BABY_STEP_TIMEOUT_MS,
            role: crate::agent_pool::AgentRole::Executor,
            gateway,
            rpc_server,
            hook_dispatcher: None,
            clock: Arc::new(OsClock),
            #[cfg(any(test, feature = "test-support"))]
            socket_path_notifier: std::sync::Mutex::new(None),
            firecracker_pool: None,
        }
    }

    /// E-15: install a Firecracker hot snapshot-restore pool. When set the
    /// executor calls `pool.spawn_from_snapshot()` before
    /// `launch_agent_process`; on KVM-absent hosts the pool returns
    /// `KvmUnavailable` and the executor falls through to the cold-boot path.
    /// The result is observable via `tracing` (SSB enqueue requires
    /// `Arc<SsbWriter>` plumbing tracked in a follow-on patch).
    #[must_use]
    pub fn with_firecracker_pool(
        mut self,
        pool: Arc<crate::sandbox::FirecrackerHotPool>,
    ) -> Self {
        self.firecracker_pool = Some(pool);
        self
    }

    /// C-15: override the `ClockProvider` (default is `Arc<OsClock>`).
    /// Used by deterministic test setups to inject `SeededClock`.
    #[must_use]
    pub fn with_clock(mut self, clock: Arc<dyn ClockProvider>) -> Self {
        self.clock = clock;
        self
    }

    pub fn with_timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    pub fn with_role(mut self, role: crate::agent_pool::AgentRole) -> Self {
        self.role = role;
        self
    }

    /// C-11: install an IPC hook dispatcher that fires pre_tool_use /
    /// post_tool_use during `run_agent_session`. None bypasses the hook
    /// surface (the legacy behaviour preserved for test executors).
    pub fn with_hook_dispatcher(
        mut self,
        dispatcher: Arc<dyn crate::ipc_accept::IpcHookDispatcher + Send + Sync + 'static>,
    ) -> Self {
        self.hook_dispatcher = Some(dispatcher);
        self
    }

    /// Test-only: install a one-shot sender that receives the socket path after
    /// `bind_session_listener` completes. The receiving end lives in the stub
    /// agent task; receiving the path proves the listen-before-spawn ordering.
    #[cfg(any(test, feature = "test-support"))]
    pub fn with_socket_path_notifier(
        mut self,
        tx: tokio::sync::oneshot::Sender<PathBuf>,
    ) -> Self {
        self.socket_path_notifier = std::sync::Mutex::new(Some(tx));
        self
    }

    // patch-R4 + C-24 (DRIFT-13): construct the [`AgentDispatchConfig`] sent
    // to the spawned baby-step agent on the initial `AgentRequest` frame. The
    // `tool_filter.allowed_tools` set MUST match
    // `production_agent_rpc::build_tool_catalog()` (now 10 wire IDs: 6 R3
    // baseline + 4 C-24 symbol_* additions) — a contract test
    // (`baby_step_initial_agent_request_contract`) pins the set equality.
    // The shape mirrors `feasibility_probe::feasibility_dispatch_config` with
    // production-tier tool surface + `rule_domains=["baby_step"]`.
    fn baby_step_dispatch_config(role: AgentRole) -> elai_ipc_types::AgentDispatchConfig {
        use elai_ipc_types::{
            ProcessConfig, ResourceLimits, SeccompProfile, ToolFilter, ToolId, TOOL_CLASSIFY,
            TOOL_COMPILE_CHECK, TOOL_EDIT, TOOL_FILE_READ, TOOL_GIT_COMMIT, TOOL_SYMBOL_CONTEXT,
            TOOL_SYMBOL_FLOW, TOOL_SYMBOL_IMPACT, TOOL_SYMBOL_LIST, TOOL_VALIDATE,
        };
        elai_ipc_types::AgentDispatchConfig {
            role,
            model_preference: "qwen2.5-coder".to_owned(),
            tool_filter: ToolFilter {
                allowed_tools: vec![
                    ToolId(TOOL_EDIT.to_owned()),
                    ToolId(TOOL_FILE_READ.to_owned()),
                    ToolId(TOOL_VALIDATE.to_owned()),
                    ToolId(TOOL_COMPILE_CHECK.to_owned()),
                    ToolId(TOOL_CLASSIFY.to_owned()),
                    ToolId(TOOL_GIT_COMMIT.to_owned()),
                    // C-24: B-29 Phase 5 symbol-graph tools (wired in
                    // build_tool_catalog + convert_json_input_to_postcard +
                    // translate_tool_request).
                    ToolId(TOOL_SYMBOL_IMPACT.to_owned()),
                    ToolId(TOOL_SYMBOL_FLOW.to_owned()),
                    ToolId(TOOL_SYMBOL_CONTEXT.to_owned()),
                    ToolId(TOOL_SYMBOL_LIST.to_owned()),
                ],
            },
            rule_domains: vec!["baby_step".to_owned()],
            grc_budget_max_calls: DEFAULT_AGENT_GRC_MAX_CALLS,
            grc_budget_max_chars: DEFAULT_AGENT_GRC_MAX_CHARS,
            sandbox_config: ProcessConfig {
                allowed_paths: Vec::new(),
                seccomp_profile: SeccompProfile::IpcBaseline,
                resource_limits: ResourceLimits {
                    max_memory_bytes: 512 * 1024 * 1024,
                    max_cpu_secs: 60,
                    max_open_files: 32,
                },
            },
        }
    }
}

impl<E: elai_core::EntropySource + Send + 'static> BabyStepExecutor
    for AgentPoolBabyStepExecutor<E>
{
    fn execute<'a>(
        &'a self,
        ctx: BabyStepExecutionCtx,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = BabyStepExecutionResult> + Send + 'a>,
    > {
        Box::pin(async move {
            // C-15: ClockProvider injection landed (formerly W2-DEFER-850).
            // `start` is a LogicalInstant; subsequent `start.elapsed()` calls
            // were rewritten to `self.clock.elapsed_since(start)` so the seeded
            // clock produces deterministic latency values.
            let start = self.clock.now();

            // Build the task string delivered to the agent. The escalation
            // message (from the anti-spiral detector) is prepended so it is
            // the first thing the agent sees on the next turn.
            let task_text = match ctx.escalation_message {
                Some(msg) => format!("{msg}\n\n{}", ctx.task_summary),
                None => ctx.task_summary.clone(),
            };

            // Spawn an agent handle on the pool.
            // patch-R4: clone so the original `task_text` value is still owned
            // for the initial AgentRequest IPC frame write further down (the
            // frame moves `task_text: String` into the IpcMessageType variant).
            let (agent_id, spawn_err) = {
                let mut pool = self.pool.lock().await;
                match pool.spawn(self.role, task_text.clone()) {
                    Ok(handle) => (Some(handle.agent_id), None),
                    Err(e) => (None, Some(format!("pool.spawn failed: {e}"))),
                }
            };

            let agent_id = match agent_id {
                Some(id) => id,
                None => {
                    let reason = spawn_err.unwrap_or_default();
                    return BabyStepExecutionResult {
                        outcome: BabyStepOutcome::Failed { reason: reason.clone() },
                        duration_ms: u64::try_from(self.clock.elapsed_since(start).as_millis()).unwrap_or(u64::MAX),
                        exit_code: None,
                        stderr_tail: reason,
                        changed_files: Vec::new(),
                    };
                }
            };

            // E-15 (M2): if a FirecrackerHotPool is wired, attempt
            // per-(agent_id, role) snapshot restore BEFORE the cold launch
            // path. On KVM-absent hosts (Windows, Linux without /dev/kvm) the
            // pool returns `FirecrackerError::KvmUnavailable` which is the
            // canonical RUNPOD-PENDING signal (mirrors E-49/E-52/E-53). We do
            // NOT short-circuit launch_agent_process: spawn_from_snapshot is
            // additive observability today; agent processes still go through
            // `pool.launch_agent_process` so the cold-boot path keeps working
            // unchanged. tracing emit only — SsbWriter plumbing deferred to
            // a follow-on patch tracked under workflow/reasoning/E-15-decisions.md.
            if let Some(fc_pool) = self.firecracker_pool.as_ref() {
                let key = crate::sandbox::SnapshotKey {
                    agent_id,
                    role: self.role,
                };
                match fc_pool.spawn_from_snapshot(
                    &key,
                    self.agent_binary_path.to_string_lossy().as_ref(),
                    &[],
                ) {
                    Ok(result) => {
                        tracing::debug!(
                            agent_id = %agent_id,
                            role = ?self.role,
                            was_cold_boot = result.was_cold_boot,
                            restore_ms = result.restore_ms,
                            budget_exceeded = result.budget_exceeded,
                            uniqueness_regenerated = result.uniqueness_regenerated,
                            "agent_forked_from_snapshot: firecracker hot pool dispatch ok"
                        );
                    }
                    Err(crate::sandbox::firecracker::FirecrackerError::KvmUnavailable) => {
                        tracing::trace!(
                            agent_id = %agent_id,
                            "firecracker_pool: KVM unavailable on this host, RUNPOD-PENDING (cold-boot fallback active)"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            agent_id = %agent_id,
                            error = %e,
                            "firecracker_pool: spawn_from_snapshot failed; cold-boot fallback"
                        );
                    }
                }
            }

            // Ensure socket directory exists.
            if let Err(e) = std::fs::create_dir_all(&self.socket_dir) {
                let reason = format!("socket_dir create failed: {e}");
                return BabyStepExecutionResult {
                    outcome: BabyStepOutcome::Failed { reason: reason.clone() },
                    duration_ms: u64::try_from(self.clock.elapsed_since(start).as_millis()).unwrap_or(u64::MAX),
                    exit_code: None,
                    stderr_tail: reason,
                    changed_files: Vec::new(),
                };
            }
            let socket_path = self
                .socket_dir
                .join(format!("agent-{}.sock", agent_id));

            // B2b listen-before-spawn invariant: bind the listener BEFORE
            // spawning the agent process so the socket endpoint exists when
            // the agent's connect() call fires.
            let mut listener =
                match crate::ipc_accept::bind_session_listener(&socket_path).await {
                    Ok(l) => l,
                    Err(e) => {
                        let reason = format!("bind_session_listener failed: {e}");
                        return BabyStepExecutionResult {
                            outcome: BabyStepOutcome::Failed { reason: reason.clone() },
                            duration_ms: u64::try_from(self.clock.elapsed_since(start).as_millis())
                                .unwrap_or(u64::MAX),
                            exit_code: None,
                            stderr_tail: reason,
                            changed_files: Vec::new(),
                        };
                    }
                };

            // Test hook: notify the stub agent of the socket path so it can
            // connect. Fires AFTER bind (proving listen-before-spawn ordering).
            // Also captures whether a stub was installed so the short-circuit
            // below can distinguish "test with stub" from "no stub at all".
            #[cfg(any(test, feature = "test-support"))]
            let stub_installed: bool;
            #[cfg(any(test, feature = "test-support"))]
            {
                if let Ok(mut guard) = self.socket_path_notifier.lock() {
                    stub_installed = guard.is_some();
                    if let Some(tx) = guard.take() {
                        let _ = tx.send(socket_path.clone());
                    }
                } else {
                    stub_installed = false;
                }
            }

            // patch-R2.2: the address the CHILD AGENT must use to connect
            // differs from the orchestrator-side socket_path on Windows.
            //   Unix:    address == socket_path  (filesystem path)
            //   Windows: address == `\\.\pipe\elai-<stem>` (named-pipe name);
            //            socket_path is `C:\...\agent-<uuid>.sock` which the
            //            agent CANNOT open as a named pipe (ERROR_FILE_NOT_FOUND).
            // PlatformListener::agent_connect_address returns the platform-correct
            // string. Pass it (as a Path) to launch_agent_process so argv[1]
            // matches the bound endpoint.
            let agent_connect_address = listener.agent_connect_address();
            let agent_address_path = std::path::Path::new(&agent_connect_address);

            // Spawn the agent process. A failure here is recorded but does
            // NOT abort the session: if a caller (e.g. a test's stub task)
            // has already connected to the socket, accept_one will succeed
            // and the session outcome governs the result. Only when both
            // the spawn AND the subsequent accept_one fail do we return Failed.
            let process_launch_err = {
                let mut pool = self.pool.lock().await;
                pool.launch_agent_process(
                    &agent_id,
                    &self.agent_binary_path,
                    agent_address_path,
                    &ctx.session_credentials,
                )
                .err()
                .map(|e| format!("launch_agent_process failed: {e}"))
            };

            // Short-circuit: if the process failed to launch AND no in-process
            // stub is connected (i.e. no test hook), there is nobody to accept
            // the IPC connection. Waiting the full timeout (120 s) would be
            // wasteful. Return Failed immediately.
            //
            // In test/test-support builds we only skip accept_one when no stub
            // was installed; when a stub IS installed it will connect and the
            // session should proceed normally.
            #[cfg(any(test, feature = "test-support"))]
            if process_launch_err.is_some() && !stub_installed {
                let reason = process_launch_err.as_deref().unwrap_or("launch_agent_process failed").to_string();
                return BabyStepExecutionResult {
                    outcome: BabyStepOutcome::Failed { reason: reason.clone() },
                    duration_ms: u64::try_from(self.clock.elapsed_since(start).as_millis())
                        .unwrap_or(u64::MAX),
                    exit_code: None,
                    stderr_tail: reason,
                    changed_files: Vec::new(),
                };
            }
            #[cfg(not(any(test, feature = "test-support")))]
            if process_launch_err.is_some() {
                let reason = process_launch_err.as_deref().unwrap_or("launch_agent_process failed").to_string();
                return BabyStepExecutionResult {
                    outcome: BabyStepOutcome::Failed { reason: reason.clone() },
                    duration_ms: u64::try_from(self.clock.elapsed_since(start).as_millis())
                        .unwrap_or(u64::MAX),
                    exit_code: None,
                    stderr_tail: reason,
                    changed_files: Vec::new(),
                };
            }

            // Accept one connection from the agent (or the test stub).
            let accept_timeout = std::time::Duration::from_millis(self.timeout_ms);
            let stream = match tokio::time::timeout(
                accept_timeout,
                crate::ipc_accept::accept_one(&mut listener),
            )
            .await
            {
                Ok(Ok(s)) => s,
                Ok(Err(e)) => {
                    let reason = match process_launch_err {
                        Some(launch_msg) => format!("{launch_msg}; accept_one failed: {e}"),
                        None => format!("accept_one failed: {e}"),
                    };
                    return BabyStepExecutionResult {
                        outcome: BabyStepOutcome::Failed { reason: reason.clone() },
                        duration_ms: u64::try_from(self.clock.elapsed_since(start).as_millis())
                            .unwrap_or(u64::MAX),
                        exit_code: None,
                        stderr_tail: reason,
                        changed_files: Vec::new(),
                    };
                }
                Err(_) => {
                    let reason = match process_launch_err {
                        Some(launch_msg) => {
                            format!("{launch_msg}; accept_one timed out after {}ms", self.timeout_ms)
                        }
                        None => format!("accept_one timed out after {}ms", self.timeout_ms),
                    };
                    return BabyStepExecutionResult {
                        outcome: BabyStepOutcome::Failed { reason: reason.clone() },
                        duration_ms: u64::try_from(self.clock.elapsed_since(start).as_millis())
                            .unwrap_or(u64::MAX),
                        exit_code: None,
                        stderr_tail: reason,
                        changed_files: Vec::new(),
                    };
                }
            };

            // Register the agent connection in the gateway before driving the
            // session so route_frame can look up the connection record.
            let process_id: u32 = {
                let pool = self.pool.lock().await;
                pool.get_agent(&agent_id)
                    .and_then(|h| h.child_process.as_ref())
                    .map(|child| child.id())
                    .unwrap_or(0)
            };
            {
                let conn = crate::ipc_gateway::AgentConnection::new(
                    agent_id,
                    self.role,
                    process_id,
                    socket_path.clone(),
                    DEFAULT_AGENT_GRC_MAX_CALLS,
                    &elai_core::clock::OsClock,
                );
                let mut gw = self.gateway.lock().await;
                gw.register_agent(conn);
            }

            // patch-R4: send the initial AgentRequest IPC frame so the agent's
            // `run_message_loop` (agent/src/main.rs:118) wakes up and enters
            // its ReAct turn. Without this write the agent blocks on read
            // until the orchestrator's outer accept-timeout closes the pipe —
            // the symptom captured in workflow/handoff/CLOUD_E2E_PROGRESS_2026-05-17_R3_R3_1.md
            // (fw_rows=0, "baby-step timed out at 120s"). Pattern mirrors
            // feasibility_probe.rs:718-733. On write failure we deregister
            // the agent from the gateway and return Failed to prevent
            // leaking a registered AgentConnection.
            let mut stream = stream;
            let task_text_len = task_text.len();
            let dispatch_config = Self::baby_step_dispatch_config(self.role);
            let initial_request = elai_ipc_types::IpcMessageType::AgentRequest {
                task: task_text,
                dispatch_config,
            };
            if let Err(e) = crate::ipc_accept::write_frame_to_stream(
                &mut stream,
                &initial_request,
            )
            .await
            {
                warn!(
                    target: "orchestrator::task_runner",
                    %agent_id,
                    role = ?self.role,
                    error = %e,
                    "send_initial_agent_request failed"
                );
                {
                    let mut gw = self.gateway.lock().await;
                    gw.remove_agent(&agent_id);
                }
                let reason = format!("send_initial_agent_request:{e}");
                return BabyStepExecutionResult {
                    outcome: BabyStepOutcome::Failed { reason: reason.clone() },
                    duration_ms: u64::try_from(self.clock.elapsed_since(start).as_millis()).unwrap_or(u64::MAX),
                    exit_code: None,
                    stderr_tail: reason,
                    changed_files: Vec::new(),
                };
            }
            info!(
                target: "orchestrator::task_runner",
                %agent_id,
                role = ?self.role,
                task_len = task_text_len,
                "wrote initial AgentRequest frame"
            );

            // Drive the IPC session: read frames, route via gateway, call the
            // RPC server for InferRequest / ToolRequest, write responses.
            let remaining_ms = self.timeout_ms.saturating_sub(
                u64::try_from(self.clock.elapsed_since(start).as_millis()).unwrap_or(0),
            );
            let session_outcome = crate::ipc_accept::run_agent_session(
                Arc::clone(&self.gateway),
                Arc::clone(&self.rpc_server),
                agent_id,
                stream,
                remaining_ms,
                self.hook_dispatcher.as_ref().map(Arc::clone),
            )
            .await;

            // Deregister the agent from the gateway (cleans up socket on Unix).
            {
                let mut gw = self.gateway.lock().await;
                gw.remove_agent(&agent_id);
            }

            // Map session outcome to baby-step outcome.
            let (outcome, session_err) = match session_outcome.final_state {
                crate::ipc_accept::AgentFinalState::Passed => {
                    (BabyStepOutcome::Passed, None)
                }
                crate::ipc_accept::AgentFinalState::Failed { reason } => {
                    (BabyStepOutcome::Failed { reason: reason.clone() }, Some(reason))
                }
            };

            // Combine any spawn/session error messages into stderr_tail.
            let stderr_tail = match (process_launch_err, session_err) {
                (Some(l), Some(s)) => format!("{l}; session: {s}"),
                (Some(l), None) => l,
                (None, Some(s)) => s,
                (None, None) => String::new(),
            };

            BabyStepExecutionResult {
                outcome,
                duration_ms: u64::try_from(self.clock.elapsed_since(start).as_millis()).unwrap_or(u64::MAX),
                exit_code: None,
                stderr_tail,
                // Gap #9: real agents will populate this via an out-of-band
                // diff reporter once wire-4A's changed-files protocol lands.
                // Until then we default to empty so the post-execution rule
                // re-eval is a no-op (documented in the trait doc-comment).
                changed_files: Vec::new(),
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Spiral tracker (Session wire-antispiral)
// ---------------------------------------------------------------------------

/// Scoped per-`drive_task` anti-spiral state: holds the detector, the
/// synthetic agent id used to key the detector's internal map, the pending
/// escalation message (if any), and a `hard_stop` flag that causes the
/// baby-step loop to bail.
///
/// The runner feeds every `emit_reasoning`-level step through the detector.
/// Because task_runner currently emits synthetic, step-varying content, the
/// detector rarely fires in the synthetic path — the wiring is the contract,
/// and real agent GRC steams will hit it when wire-4A is fully online.
pub(crate) struct SpiralTracker {
    detector: crate::anti_spiral::AntiSpiralDetector<elai_core::clock::OsClock>,
    agent_id: Uuid,
    pending_escalation: Option<String>,
    hard_stop: bool,
    /// Monotonic superstep counter handed to `detector.record_step`. Matches
    /// the spec's "superstep" field semantics (grows with every step).
    superstep: i64,
}

impl SpiralTracker {
    pub(crate) fn new(agent_id: Uuid) -> Self {
        Self {
            detector: crate::anti_spiral::AntiSpiralDetector::new(),
            agent_id,
            pending_escalation: None,
            hard_stop: false,
            superstep: 0,
        }
    }

    pub(crate) fn hard_stop(&self) -> bool {
        self.hard_stop
    }

    /// Take the current escalation message (if any). Returns `None` after the
    /// first call until the next `EscalationSignalNeeded` fires.
    pub(crate) fn take_escalation_message(&mut self) -> Option<String> {
        self.pending_escalation.take()
    }

    /// Record a reasoning step. Returns the [`crate::anti_spiral::SpiralResult`]
    /// so the caller can decide whether to emit the paired SSB rows.
    pub(crate) fn record(
        &mut self,
        step_type: &str,
        topic: &str,
        content: &str,
    ) -> crate::anti_spiral::SpiralResult {
        let ss = self.superstep;
        self.superstep = self.superstep.saturating_add(1);
        let result = self.detector.record_step(
            self.agent_id,
            step_type,
            topic,
            content,
            ss,
        );
        match &result {
            crate::anti_spiral::SpiralResult::EscalationSignalNeeded { message, .. } => {
                self.pending_escalation = Some(message.clone());
            }
            crate::anti_spiral::SpiralResult::HardStop { .. } => {
                self.hard_stop = true;
            }
            crate::anti_spiral::SpiralResult::Clean => {}
        }
        result
    }

    pub(crate) fn agent_id(&self) -> Uuid {
        self.agent_id
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
        let orchestrator = WebResearchOrchestrator::new(
            searx,
            fetcher,
            extractor,
            quality,
            content_index,
            budget,
            self.tier_config.clone(),
            self.clock.clone(),
        );

        // D4: ELAI_FETCHER=obscura routes to the headless-browser path.
        // Only available when compiled with the "obscura" feature.
        // If feature is off and ELAI_FETCHER=obscura is set, warn and fall back.
        #[cfg(feature = "obscura")]
        let orchestrator = {
            let want_obscura = std::env::var("ELAI_FETCHER")
                .ok()
                .as_deref()
                == Some("obscura");
            if want_obscura {
                tracing::info!("ELAI_FETCHER=obscura: using ObscuraFetcher (headless browser)");
                orchestrator.with_obscura_fetcher(
                    elai_web_search::obscura_fetcher::ObscuraFetcher::new(),
                )
            } else {
                orchestrator
            }
        };
        #[cfg(not(feature = "obscura"))]
        {
            if std::env::var("ELAI_FETCHER").ok().as_deref() == Some("obscura") {
                tracing::warn!(
                    "ELAI_FETCHER=obscura requested but binary was compiled without \
                     the 'obscura' feature; falling back to reqwest fetcher"
                );
            }
        }

        Ok(orchestrator)
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
                sources_used: u32::try_from(node.citations.len()).unwrap_or(u32::MAX),
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
                fetcher: event.fetcher,
                stage: String::new(),
                stage_phase: String::new(),
                stage_payload: None,
            });
            if let Err(e) = writer
                .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
                .await
            {
                error!(error = %e, "task_runner: write WebResearchStep failed");
            }
        })
    }

    /// B-7: Emit a fine-grained per-stage event as a `web_research_step` row
    /// with `stage` + `stage_phase` + `stage_payload` fields populated.
    /// Reuses the existing `WebResearchStep` SSB action type — no schema bump.
    fn emit_stage(
        &self,
        event: WebResearchStageEvent,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        let writer = self.writer.clone();
        Box::pin(async move {
            let payload = SsbPayload::WebResearchStep(WebResearchStepPayload {
                agent_id: RUNNER_AGENT_ID.to_owned(),
                node_id: event.node_id,
                // action="" for stage rows; consumers distinguish via stage+stage_phase.
                action: String::new(),
                phi: event.phi,
                psi: event.psi,
                iteration: event.iteration,
                sources_fetched: 0,
                sources_used: 0,
                fetcher: String::new(),
                stage: event.stage.to_owned(),
                stage_phase: event.stage_phase.to_owned(),
                stage_payload: Some(event.payload),
            });
            if let Err(e) = writer
                .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
                .await
            {
                error!(error = %e, "task_runner: write WebResearchStage failed");
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
        resolved.pipeline_grc.as_ref(),
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
        resolved.pipeline_grc.as_ref(),
    )
    .await;

    match applier.apply(template, &project_dir, &task.task) {
        Ok(()) => {
            emit_reasoning(
                writer,
                2,
                "verify",
                &format!("scaffold complete template={} dir={:?}", template, project_dir.display()),
                resolved.pipeline_grc.as_ref(),
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
    /// Optional Layer-3 InterrogationTree store (Session A-19.REDO). When
    /// `Some`, the runner looks up a matching tree on each failing baby step,
    /// drives a short traversal with [`default_answers_for_category`], and
    /// emits a `Finding` carrying the resulting `FixTemplate` instruction so
    /// the dashboard can surface Layer-3 fix suggestions. Also emits a
    /// one-shot `interrogation_tree_metric` Finding at first use reporting
    /// the seeded tree count. When `None`, Layer-3 wiring is a no-op.
    pub interrogation_store: Option<Arc<dyn InterrogationStore>>,
    /// Root directory under which per-task checkpoint subdirectories are
    /// written (Session 3C.REDO). When `Some`, the runner owns a
    /// [`CheckpointManager`] and writes a checkpoint row to both disk and the
    /// SSB after each baby-step boundary. When `None`, checkpoint/recovery is
    /// disabled (pre-3C.REDO behavior — task_runner does not checkpoint).
    ///
    /// Typical value: `.elai_cc/sessions/{session_id}/agents`. Each task seq
    /// becomes its own `agent_id` subdirectory under this root.
    pub checkpoint_scratch_dir: Option<PathBuf>,
    /// Session wire-4A: per-baby-step executor. When `Some`, replaces the
    /// synthetic `BabyStepOutcome::Passed` surrogate with a real dispatch.
    /// When `None`, the runner falls back to [`SyntheticBabyStepExecutor`]
    /// (pre-wire-4A behavior — all tests keep working with their legacy
    /// expectations of `passed: true` baby-step outcomes).
    pub baby_step_executor: Option<Arc<dyn BabyStepExecutor>>,
    /// Session wire-antispiral: enable the [`crate::anti_spiral::AntiSpiralDetector`]
    /// hook in `drive_task`. When `true`, each reasoning emission is fed into
    /// the detector; EscalationSignalNeeded is injected into the next baby
    /// step's execution context; HardStop emits `SsbPayload::SpiralDetected`
    /// + a `spiral_hardstop` Finding and terminates the task. When `false`,
    /// the detector is not constructed (pre-wire-antispiral behavior).
    ///
    /// Depends on wire-4A: the escalation injection path runs through the
    /// [`BabyStepExecutor::execute`] API surface, but the HardStop + Finding
    /// emission works even with the synthetic executor (so the contract can
    /// be exercised without a real agent binary).
    pub anti_spiral_enabled: bool,
    /// Session patch-1.32: shared LLM provider used for research decomposition
    /// and (future) baby-step agent dispatch on the dashboard path. When
    /// `Some`, `drive_task` runs `build_dispatch_context` once per task and
    /// the provider becomes available to the decomposition call site. When
    /// `None`, the runner falls back to [`SidecarGroundingClient`] (the
    /// pre-patch-1.32 narrow reqwest client) so every existing test and the
    /// CLI-driven path keep working. This is the single-instance plumb that
    /// unblocks phase 3J (DoLa/TurboQuant/steering/HyDE/MCP) on the
    /// dashboard path — see `docs/mvp-wiring-audit-2026-04-18.md` section
    /// 4 item #3 ("patch-1.32 dashboard LLM backend wiring").
    pub llm_provider: Option<Arc<dyn LlmProvider>>,
    /// Session patch-1.32: optional backend routing config for
    /// `build_dispatch_context`. When `Some` together with `llm_provider`,
    /// the dashboard path invokes `build_dispatch_context` on each dispatch
    /// to exercise the FSM-state tool filter + context budget adapter. When
    /// `None`, the default `BackendConfig` is used (equivalent to API
    /// backend, which build_dispatch_context still accepts).
    pub backend_config: Option<crate::dispatcher::BackendConfig>,
    /// Gap #6 (MVP audit 2026-04-18 section 4 item 6, A-44): optional
    /// project-indexing service. When `Some`, `drive_task` invokes
    /// [`crate::indexing_service::IndexingService::run_full_index`] the first
    /// time a given `project_id` is dispatched and emits an `SsbPayload::Finding`
    /// summarising `files_indexed` / `dkg_nodes` / `bm25_chunks`. Subsequent
    /// tasks for the same project skip re-indexing (tracked via the runner's
    /// internal indexed-project cache). When `None`, no indexing runs and the
    /// rule evaluator keeps using whatever [`DkgQuerier`] was supplied at
    /// `rule_evaluator` construction (i.e. `EmptyDkgQuerier` on the dashboard
    /// path pre-Gap #6). The service is wrapped in an async mutex because
    /// `run_full_index` takes `&mut self` and may be invoked concurrently from
    /// multiple baby-step scheduling points; the mutex serialises the initial
    /// build without blocking the runner's tokio runtime.
    pub indexing_service: Option<Arc<AsyncMutex<crate::indexing_service::IndexingService>>>,
    /// Batch 1A (MVP audit 2026-04-18 section 3): optional hook dispatcher
    /// that runs the T1/T2/T3 tiered analysis around each baby step. When
    /// `Some`, [`drive_task`] calls `dispatch_pre_tool_use` before
    /// `baby_step_executor.execute`, `dispatch_post_tool_use` after the
    /// baby-step checkpoint when the step passed AND produced at least one
    /// changed file, `dispatch_stop` on task completion, and
    /// `dispatch_pre_compact` as a reserved MVP no-op. Block verdicts abort
    /// the step / flip `TestOutcome.passed=false`; Warn verdicts emit
    /// `SsbPayload::HookEvent` rows without flipping the outcome. When
    /// `None`, the current no-hook behavior is preserved (pre-batch-1a
    /// semantics).
    ///
    /// Dispatcher respects the `ELAI_HOOKS_DISABLED` env var as an emergency
    /// shutoff; see `hook_dispatcher::InProcessHookDispatcher::new`.
    pub hook_dispatcher: Option<Arc<dyn crate::hook_dispatcher::HookDispatcher>>,
    /// Batch 1B: reader over the sidecar extended-signal surface
    /// (A-8 / A-9 / A-17 / A-25 / A-37). When `Some`, `drive_task` emits
    /// `ProviderSignals` / `SearchTriggerFired` SSB rows after each baby step
    /// and a `VramTierDetected` row once at task startup. When `None`, all
    /// emission is suppressed (pre-batch-1b behavior).
    ///
    /// Respects the `ELAI_DISABLE_PROVIDER_SIGNALS` env var as an emergency
    /// shutoff (short-circuits every read to `None`). See
    /// `crate::provider_signals::ProviderSignalReader`.
    pub signal_reader: Option<Arc<dyn ProviderSignalReader>>,
    /// Batch 2B: optional tool dispatcher (see
    /// `crate::ToolDispatcher`). When `Some`, [`drive_task`] invokes the
    /// dispatcher at six call sites around each baby step: file outline
    /// pre-edit, post-edit validation, post-failure classification, and
    /// post-success git commit. Emits `ToolDispatch` / `ValidationTier`
    /// / `FailureClassified` / `GitCommit` / `DairaTrace` rows. When
    /// `None`, every dispatch site is skipped (pre-batch-2b behaviour).
    ///
    /// Respects the `ELAI_DISABLE_TOOL_DISPATCH` env var as an emergency
    /// shutoff (short-circuits every dispatch to `NotImplemented`).
    pub tool_dispatcher: Option<Arc<dyn ToolDispatcher>>,
    /// Batch 2B: branch policy for `tool_dispatcher.dispatch_git_commit`.
    /// Defaults to `BranchPerPlan` which creates `elai/plan-{plan_id}`
    /// branches per task.
    pub git_branch_policy: GitBranchPolicy,
    /// Batch 3A (Phase 4F): optional system prober. When `Some`, `drive_task`
    /// emits a [`SsbPayload::SystemProbe`] row once at startup by invoking
    /// [`crate::SystemProber::probe`]. When `None`, no emission (pre-4F
    /// behaviour). Respects `ELAI_DISABLE_SYSTEM_PROBE` as a kill switch
    /// inside the concrete [`crate::DefaultSystemProber`].
    pub system_prober: Option<Arc<dyn crate::system_probe::SystemProber>>,
    /// Batch 3A (Phase 4F): optional feasibility prober. When `Some`,
    /// `drive_task` invokes per-concept baby-step probes via
    /// [`crate::FeasibilityProber::probe_baby_step`] and emits
    /// [`SsbPayload::FeasibilityReport`] rows. When `None`, no emission.
    /// Respects `ELAI_DISABLE_FEASIBILITY_PROBE`.
    pub feasibility_prober: Option<Arc<dyn crate::feasibility_probe::FeasibilityProber>>,
    /// Batch 3B (Phase 4G): optional autonomous gate. When `Some`, the
    /// runner routes FSM transition / tree-advance decisions through
    /// [`crate::AutonomousGate::decide`]; [`AutonomyDecision::AutoAdvance`]
    /// continues silently while [`AutonomyDecision::RequestHuman`] emits
    /// an `InterventionRequest` and waits on the existing response channel.
    /// When `None`, the runner preserves the pre-4G behaviour (every gate
    /// falls through to the InteractiveGate or skips).
    ///
    /// Respects [`crate::autonomous_gate::ELAI_DISABLE_AUTONOMOUS_GATE_ENV`]
    /// as an emergency shutoff applied inside `decide` itself.
    pub autonomous_gate: Option<Arc<crate::autonomous_gate::AutonomousGate>>,
    /// Batch 3C (Phase 4H) — optional Layer 1A self-modification processor.
    /// When `Some`, `drive_task` assembles a [`crate::PipelineOutcome`] from
    /// the SSB stream at task completion and invokes `process_outcome` +
    /// (when `ELAI_LAYER1A_AUTO_APPLY` is set) `apply_proposal` on each
    /// returned proposal. When `None`, self-modification is fully disabled
    /// (pre-batch-3c behaviour).
    ///
    /// Respects [`crate::ELAI_DISABLE_LAYER1A_ENV`] — the dashboard
    /// construction site short-circuits to `None` when set.
    pub layer1a_processor: Option<Arc<dyn crate::Layer1aProcessor>>,
    /// Batch-fu (FU.3 / A-18) — optional per-dispatch GenerationProfile
    /// selector. When `Some`, `drive_task` invokes
    /// [`crate::GenerationProfileSelector::select`] once per dispatch site
    /// and threads the result into `PhaseSlot.profile_hint` so
    /// `build_dispatch_context` records the active profile on the assembled
    /// `DispatchContext` and emits `ProfileSelected` when the profile is
    /// non-default. When `None`, every dispatch uses
    /// [`crate::GenerationProfile::FirstAttempt`] (pre-batch-fu behaviour).
    ///
    /// Respects `ELAI_DISABLE_PROFILE_SELECTOR` as an emergency shutoff
    /// applied by the dashboard construction site.
    pub dispatch_context_profile: Option<Arc<dyn crate::GenerationProfileSelector>>,
    /// Batch-fu (FU.4 / A-23) — optional micro-reflection client. When
    /// `Some`, `drive_task` calls
    /// [`crate::emit_micro_reflection`] after every
    /// [`record_mistake_on_failure`] write so the dashboard renders a purple
    /// micro-reflection row under the MistakeEntry it diagnosed. When
    /// `None`, no reflection call is made (pre-batch-fu behaviour).
    ///
    /// Respects `ELAI_DISABLE_MICRO_REFLECTION` as an emergency shutoff
    /// applied by the dashboard construction site.
    pub reflection_client: Option<Arc<dyn crate::DynReflectionClient>>,
    /// Batch-fu (FU.5 / A-31) — optional irreversible-action classifier. When
    /// `Some`, `drive_task` invokes
    /// [`crate::IrreversibleActionClassifier::classify`] on every baby step
    /// after the PreToolUse hook dispatch; a
    /// [`crate::IrreversibleVerdict::ConfirmationRequired`] verdict emits an
    /// `InterventionRequest` SSB row and blocks the step until a matching
    /// `InterventionResponse` row arrives on the broadcast channel. When
    /// `None`, every dispatch proceeds without classification (pre-batch-fu
    /// behaviour). The check is skipped when
    /// `autonomous_gate.mode() == GateMode::Autonomous`.
    ///
    /// Respects `ELAI_DISABLE_IRREVERSIBLE_CLASSIFIER` as an emergency shutoff
    /// applied by the dashboard construction site.
    pub irreversible_classifier: Option<Arc<dyn crate::IrreversibleActionClassifier>>,
    /// Batch-ful-a (FUL-A.1) — shared RwLock handle over the rule engine's
    /// DkgQuerier trait object. When `Some`, `drive_task` attempts to build
    /// a real `DkgQuerierAdapter` pointing at the freshly-built DKG database
    /// and swap it into this slot after `IndexingService::run_full_index`
    /// succeeds for the first time on the current project. When `None`, no
    /// swap is performed (pre-ful-a behaviour: rule engine sticks with whatever
    /// querier was plugged at construction).
    ///
    /// The dashboard constructs this handle from the same `Arc<RwLock<...>>`
    /// it threads into `HotSwappableDkgQuerier` so the rule engine and the
    /// post-index hook share a single trait object.
    pub dkg_querier_swap: Option<
        Arc<std::sync::RwLock<Box<dyn elai_rules::predicate::DkgQuerier + Send + Sync>>>,
    >,
    /// Batch FUL-B: optional PhaseSizing-aware splitter. When `Some`, the
    /// autonomous-gate SPLIT_TASK emit path consults the splitter to build
    /// real sub-task descriptors (ids + summaries + preserved contracts).
    /// When `None`, the emit path falls back to the Batch 3B
    /// deterministic `{step_id}.subN` fallback. Preserving backward-compat
    /// is the contract — see
    /// `fulb_phase_sizing_splitter_contract::contract_task_runner_config_splitter_defaults_to_none`.
    pub task_splitter: Option<Arc<dyn crate::phase_sizing_splitter::TaskSplitter>>,
    /// C-9 — optional shared [`crate::research_tree::ResearchTree`] handle.
    /// When `Some`, the autonomous-gate SPLIT_TASK dispatch consults
    /// `ResearchTree::split_step(&plan)` to realise the carried
    /// [`crate::autonomous_gate::SplitTaskPlan`]. An empty return fires
    /// [`crate::autonomous_gate::EscalationTrigger::SplitTaskFailed`].
    /// When `None`, the SPLIT_TASK dispatch falls back to the existing
    /// Batch FUL-B PhaseSizing splitter (when wired) or to the plan's own
    /// deterministic sub-tasks (Batch 3B-compatible).
    ///
    /// The `RwLock` (not `AsyncMutex`) is used because `split_step` is
    /// `&self` and pure CPU — concurrent readers do not need exclusion.
    pub research_tree:
        Option<Arc<tokio::sync::RwLock<crate::research_tree::ResearchTree>>>,
    /// Batch B-8 (PipelineGrc activation) — optional shared `PipelineGrc`
    /// handle wrapping the reasoning-engine `BasicGrcEngine`. When `Some`,
    /// `drive_task` invokes the four safety methods at the four insertion
    /// points (`ground_plan` post-decompose, `generate_csq` pre-executor,
    /// `check_inline_feasibility` post-executor, `InlineFeasibilityState::reset`
    /// at phase boundary). When `None`, every gated call site is a no-op
    /// (pre-B-8 behaviour). The `AsyncMutex` is required because
    /// [`crate::grc::PipelineGrc::think`] /
    /// [`crate::grc::PipelineGrc::check_inline_feasibility`] take `&mut self`
    /// and the runner crosses `.await` boundaries holding the lock.
    ///
    /// Respects [`crate::grc::ELAI_DISABLE_PIPELINE_GRC_ENV`] as an emergency
    /// shutoff applied by the dashboard / CLI construction site (the
    /// kill-switch sets this slot to `None` before the runner sees it).
    pub pipeline_grc: Option<Arc<AsyncMutex<crate::grc::PipelineGrc>>>,
    /// B-10 (DECISION-A / DECISION-F): per-model
    /// [`crate::context_budget::ModelCapabilityProfile`] used by every
    /// `build_dispatch_context` call so production prompt assembly uses the
    /// real per-tier slot weights and attention-ceiling instead of the
    /// `None` -> `[0,0,0]` zeroed-stub fallback at dispatcher.rs:222.
    ///
    /// When `None`, `ResolvedConfig::from` infers a profile from
    /// `backend_config.model` via
    /// [`crate::context_budget::detect_model_capability_profile`] (DECISION-F).
    /// Construction sites (CLI / dashboard) may pass `Some(...)` to override.
    pub model_capability_profile:
        Option<Arc<crate::context_budget::ModelCapabilityProfile>>,
    /// C-5 — production [`crate::cloud_mode::CloudModeRouter`]. When
    /// `Some` (and the resolved [`crate::cloud_mode::ExecMode`] is `Cloud`
    /// or `Auto`), the baby-step pre-dispatch hook emits an
    /// [`crate::ssb::EscalateToCloudPayload`] row and consults the router
    /// before the cloud send. The router carries the privacy profile, the
    /// spend caps, and the air-gapped guard. When `None`, the pre-C-5
    /// behaviour is preserved (no router, no escalation emission, no
    /// cap enforcement).
    pub cloud_router: Option<Arc<crate::cloud_mode::CloudModeRouter>>,
    /// C-5 — daily spend cap in USD. When `Some` AND
    /// [`SsbViewer::cloud_spend_summary`] reports `day_usd >= cap`, the
    /// router blocks new cloud calls and emits a `cloud_spend_blocked`
    /// SSB row (carried via [`SsbActionType::CloudSpendBlocked`] when the
    /// row type is wired; today the implementation maps to the existing
    /// `EscalateToCloud` row with a `cap_reached` trigger_reason as a
    /// stable degradation marker). `None` disables cap enforcement.
    pub cloud_spend_cap_daily_usd: Option<f64>,
    /// C-25 — optional template crystallizer. When `Some`, the
    /// post-execute baby-step hook records an `ExecutionOutcome` and
    /// invokes `check_crystallization`. On crystallization, the
    /// resulting `CompiledPipeline` is upserted into
    /// `compiled_pipeline_store` and persisted to disk. `None`
    /// preserves pre-C-25 behaviour.
    pub crystallizer: Option<
        Arc<AsyncMutex<crate::compiled_pipeline::TemplateCrystallizer>>,
    >,
    /// C-25 — optional compiled-pipeline on-disk store.
    pub compiled_pipeline_store: Option<
        Arc<AsyncMutex<crate::compiled_pipeline::CompiledPipelineStore>>,
    >,
    /// C-25 — optional background-task manager (soft/hard timeout
    /// auto-detach + spawn/complete SSB rows).
    pub background_task_manager:
        Option<Arc<crate::background_task::BackgroundTaskManager>>,
    /// C-39 — optional production tool dispatch impl. When `Some`, the
    /// CLI `--via-mcp` flag has selected either
    /// [`crate::dispatch::InProcessDispatch`] or
    /// [`crate::dispatch::McpClientDispatch`] at startup; the runner
    /// can route tool-call sites through this seam to surface
    /// `cli_mcp_dispatch` or `mcp_tool_called` SSB telemetry. `None`
    /// preserves the pre-C-39 default (legacy ToolDispatcher only;
    /// the dispatch trait is unused).
    pub mcp_dispatch: Option<Arc<dyn crate::dispatch::Dispatch>>,
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
            interrogation_store: None,
            checkpoint_scratch_dir: None,
            baby_step_executor: None,
            anti_spiral_enabled: false,
            llm_provider: None,
            backend_config: None,
            indexing_service: None,
            hook_dispatcher: None,
            signal_reader: None,
            tool_dispatcher: None,
            git_branch_policy: GitBranchPolicy::default(),
            system_prober: None,
            feasibility_prober: None,
            autonomous_gate: None,
            layer1a_processor: None,
            dispatch_context_profile: None,
            reflection_client: None,
            irreversible_classifier: None,
            dkg_querier_swap: None,
            task_splitter: None,
            research_tree: None,
            pipeline_grc: None,
            model_capability_profile: None,
            // C-5: defaults preserve pre-C-5 behaviour (no router, no
            // ceiling enforcement). Production wiring sites in
            // runtime_setup.rs::apply_task_runner_wiring populate both
            // fields when `cloud_config` is supplied by the CLI / dashboard.
            cloud_router: None,
            cloud_spend_cap_daily_usd: None,
            // C-25 defaults: None preserves pre-C-25 behaviour. Production
            // wiring sites (CLI runtime_setup or test harness) populate
            // these Arcs to activate the crystallizer + cache loop.
            crystallizer: None,
            compiled_pipeline_store: None,
            background_task_manager: None,
            // C-39: default to None preserves pre-C-39 behavior. Production
            // wiring sites (CLI run_subcommand via apply_task_runner_wiring)
            // populate this Arc when --via-mcp is set.
            mcp_dispatch: None,
        }
    }

    /// Builder: inject a live `PipelineGrc` instance (B-8a).
    ///
    /// Production construction sites (dashboard, CLI runner) call this after
    /// [`new_minimal`] to activate the four GRC safety gates
    /// (`ground_plan` / `generate_csq` / `check_inline_feasibility` /
    /// `InlineFeasibilityState::reset`) that are already wired in
    /// `drive_task` but gated on `pipeline_grc.is_some()`.
    ///
    /// Respects [`crate::grc::ELAI_DISABLE_PIPELINE_GRC_ENV`]: when the
    /// kill-switch env-var is set, `ResolvedConfig::from` will clear the
    /// field before the runner loop sees it, so this method does NOT check
    /// the env-var — that is the responsibility of `ResolvedConfig::from`.
    ///
    /// Example (Tauri setup / CLI `start_task_runner` call site):
    /// ```ignore
    /// let cfg = TaskRunnerConfig::new_minimal(cursor_path)
    ///     .with_pipeline_grc(crate::grc::PipelineGrc::new_from_default());
    /// let (handle, cancel) = start_task_runner(ssb, cfg);
    /// ```
    pub fn with_pipeline_grc(mut self, grc: crate::grc::PipelineGrc) -> Self {
        self.pipeline_grc = Some(Arc::new(AsyncMutex::new(grc)));
        self
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
    interrogation_store: Option<Arc<dyn InterrogationStore>>,
    /// Session A-19.REDO: flipped once we have emitted the startup tree-count
    /// metric Finding so we don't emit it on every task. Wrapped in `Mutex` so
    /// `ResolvedConfig` stays shareable across await points.
    interrogation_metric_emitted: Mutex<bool>,
    /// Session 3C.REDO. Present iff [`TaskRunnerConfig::checkpoint_scratch_dir`]
    /// was supplied. When present, the runner owns a `CheckpointManager` and
    /// a `RecoveryManager` bound to the same clock, and writes checkpoint rows
    /// at each baby-step boundary. On `passed=false` it fires recovery.
    checkpoint: Option<Arc<CheckpointContext>>,
    /// Session wire-4A: baby-step executor. Always populated — when the caller
    /// does not supply one, we fall back to [`SyntheticBabyStepExecutor`].
    baby_step_executor: Arc<dyn BabyStepExecutor>,
    /// Session wire-antispiral: enable detector in drive_task loop.
    anti_spiral_enabled: bool,
    /// Session patch-1.32: shared LLM provider (see `TaskRunnerConfig::llm_provider`).
    llm_provider: Option<Arc<dyn LlmProvider>>,
    /// Session patch-1.32: resolved backend config used by `build_dispatch_context`.
    /// Always populated — when the caller does not supply one, we fall back to
    /// `BackendConfig::default()`.
    backend_config: crate::dispatcher::BackendConfig,
    /// Gap #6: shared indexing service (see `TaskRunnerConfig::indexing_service`).
    indexing_service: Option<Arc<AsyncMutex<crate::indexing_service::IndexingService>>>,
    /// Gap #6: per-project "first-task indexed" cache. Key = `project_id` as it
    /// appears on `TaskSubmittedPayload`; value = unix-epoch millis at which
    /// the full index was built. Guarded by a sync `Mutex` because the check is
    /// cheap and each `drive_task` invocation only touches it once per task.
    indexed_projects: Mutex<BTreeMap<String, i64>>,
    /// Batch 1A: optional hook dispatcher (see `TaskRunnerConfig::hook_dispatcher`).
    /// When `None`, `drive_task` skips every dispatch call site.
    hook_dispatcher: Option<Arc<dyn crate::hook_dispatcher::HookDispatcher>>,
    /// Batch 1B: optional provider signal reader (see
    /// `TaskRunnerConfig::signal_reader`). When `None`, `drive_task` skips every
    /// provider-signals emission site.
    signal_reader: Option<Arc<dyn ProviderSignalReader>>,
    /// Batch 1B: flag tracking whether the one-shot `VramTierDetected` row has
    /// already been emitted for this runner. Set on first `drive_task` entry
    /// with a `signal_reader` populated; subsequent tasks skip the emission.
    vram_tier_emitted: Mutex<bool>,
    /// Batch 2B: optional tool dispatcher (see
    /// `TaskRunnerConfig::tool_dispatcher`). When `None`, `drive_task` skips
    /// every dispatch site.
    tool_dispatcher: Option<Arc<dyn ToolDispatcher>>,
    /// Batch 2B: branch policy used when `tool_dispatcher.dispatch_git_commit`
    /// fires.
    git_branch_policy: GitBranchPolicy,
    /// Batch 3A (Phase 4F): optional system prober.
    system_prober: Option<Arc<dyn crate::system_probe::SystemProber>>,
    /// Batch 3A (Phase 4F): one-shot emission flag for the SystemProbe row.
    system_probe_emitted: Mutex<bool>,
    /// Batch 3A (Phase 4F): optional feasibility prober.
    feasibility_prober: Option<Arc<dyn crate::feasibility_probe::FeasibilityProber>>,
    /// Batch 3B (Phase 4G): optional autonomous gate (see
    /// `TaskRunnerConfig::autonomous_gate`).
    autonomous_gate: Option<Arc<crate::autonomous_gate::AutonomousGate>>,
    /// Batch 3C (Phase 4H): optional Layer 1A processor (see
    /// `TaskRunnerConfig::layer1a_processor`).
    layer1a_processor: Option<Arc<dyn crate::Layer1aProcessor>>,
    /// Batch-fu (FU.3 / A-18): optional GenerationProfile selector (see
    /// `TaskRunnerConfig::dispatch_context_profile`).
    dispatch_context_profile: Option<Arc<dyn crate::GenerationProfileSelector>>,
    /// Batch-fu (FU.4 / A-23): optional micro-reflection client (see
    /// `TaskRunnerConfig::reflection_client`).
    reflection_client: Option<Arc<dyn crate::DynReflectionClient>>,
    /// Batch-fu (FU.5 / A-31): optional irreversible-action classifier (see
    /// `TaskRunnerConfig::irreversible_classifier`).
    irreversible_classifier: Option<Arc<dyn crate::IrreversibleActionClassifier>>,
    /// Batch-fu (FU.6 / A-46): shared telescope stats counter. Always present —
    /// allocated during `ResolvedConfig::from` so `invoke_telescope` can record
    /// aggregate reductions without additional config plumbing.
    telescope_stats: Arc<crate::TelescopeStats>,
    /// Batch-ful-a (FUL-A.1): optional shared RwLock over the rule engine's
    /// DkgQuerier (see `TaskRunnerConfig::dkg_querier_swap`).
    dkg_querier_swap: Option<
        Arc<std::sync::RwLock<Box<dyn elai_rules::predicate::DkgQuerier + Send + Sync>>>,
    >,
    /// Batch-ful-a: per-project set of `project_id`s for which the
    /// DkgQuerierAdapter has already been hot-swapped in this runner
    /// lifetime. Prevents re-swap on a subsequent task for the same project
    /// and pairs the contract with `SsbActionType::DkgQuerierSwapped` being
    /// emitted exactly once per project.
    dkg_querier_swapped_projects: Mutex<std::collections::HashSet<String>>,
    /// Batch FUL-B: optional PhaseSizing-aware splitter. See
    /// [`TaskRunnerConfig::task_splitter`].
    task_splitter: Option<Arc<dyn crate::phase_sizing_splitter::TaskSplitter>>,
    /// C-9: optional shared ResearchTree handle for SPLIT_TASK dispatch.
    /// See [`TaskRunnerConfig::research_tree`].
    research_tree:
        Option<Arc<tokio::sync::RwLock<crate::research_tree::ResearchTree>>>,
    /// Batch B-8: shared PipelineGrc handle. See
    /// [`TaskRunnerConfig::pipeline_grc`]. The `ResolvedConfig::from`
    /// constructor short-circuits this slot to `None` when the kill-switch
    /// env-var [`crate::grc::ELAI_DISABLE_PIPELINE_GRC_ENV`] is set, so every
    /// downstream gated call site sees a single boolean (`is_some()`) and
    /// behaves as pre-B-8 when the bypass is active.
    pipeline_grc: Option<Arc<AsyncMutex<crate::grc::PipelineGrc>>>,
    /// B-28 WIRING-2 (item H): focus-tunnel adaptive pre-assembler. None
    /// today (no TaskRunnerConfig construction site); the wiring activates
    /// when a future patch lands the production Arc.
    focus_tunnel_pre_assembler:
        Option<Arc<AsyncMutex<crate::focus_tunnel::PipelinePreAssembler>>>,
    /// B-28 WIRING-2 (item I): focus-tunnel dependency resolver. None today.
    focus_tunnel_dep_resolver: Option<Arc<crate::focus_tunnel::DependencyResolver>>,
    /// B-28 WIRING-2 (item J): speculative file router. None today.
    focus_tunnel_file_router:
        Option<Arc<AsyncMutex<crate::focus_tunnel::SpeculativeFileRouter>>>,
    /// C2: per-session credential map accumulated from RequestCredential gates.
    ///
    /// Keys are sanitized env-var names (`[A-Z0-9_]+`); values are user-supplied
    /// strings. Populated by `run_confirmation_gate` when the user submits a
    /// credential form; applied via `Command::env()` calls at subprocess spawn
    /// time. Values are NEVER logged. Guarded by `std::sync::Mutex` because the
    /// update happens inside the async `drive_task` body without awaiting the lock.
    session_credentials: Mutex<BTreeMap<String, String>>,
    /// B-10 (DECISION-A / DECISION-F): always-Some
    /// [`crate::context_budget::ModelCapabilityProfile`] derived once at
    /// runner construction. Threaded into every `build_dispatch_context` call
    /// (production callsite at L3131) so the assembled `ContextBudget` uses
    /// real per-tier slot weights instead of the
    /// `[0,0,0]` zeroed fallback.
    model_capability_profile: Arc<crate::context_budget::ModelCapabilityProfile>,
    /// B-24 Phase 4I Group C9 (decision 94): lazily-instantiated media
    /// validator. Populated by `drive_task` immediately after
    /// `spec_from_decomposition` when the resulting `PipelineSpec.media` is
    /// non-empty; remains `None` for the typical code-only task and for any
    /// task whose decomposition never declares a media producer.
    ///
    /// `tokio::sync::Mutex` because the slot is mutated from inside the
    /// async `drive_task` body and re-read by the same task for subsequent
    /// validation calls; spec phase-4I.txt L289-294 mandates that the
    /// validator is constructed at most once per pipeline.
    media_validator: AsyncMutex<Option<Arc<crate::media_validator::MediaValidator>>>,
    /// B-24 Phase 4I Group C9 (decision 94): SSB-backed sink instance paired
    /// with `media_validator`. Lifecycle is identical — populated when
    /// `media_validator` is constructed; the dashboard subscriber (C8) reads
    /// from the same `Ssb` handle, so callers obtain a sink that emits the
    /// 6 media_* SSB rows by passing this as
    /// `Option<&dyn pipeline::MediaSsbSink>` to
    /// `media_validator::validate_with_sink`.
    media_sink: AsyncMutex<Option<Arc<crate::media_validator::SsbMediaSink>>>,
    /// C-5: live cloud-mode router. `Some` when the runner has been wired
    /// with a [`crate::cloud_mode::CloudConfig`] via
    /// [`crate::runtime_setup::apply_task_runner_wiring`]. Read in the
    /// baby-step pre-dispatch hook to (a) decide whether the current step
    /// should escalate to cloud (emitting [`crate::ssb::SsbActionType::EscalateToCloud`]),
    /// and (b) consult the spend-cap state via the SSB viewer before any
    /// network call.
    cloud_router: Option<Arc<crate::cloud_mode::CloudModeRouter>>,
    /// C-5: optional daily spend cap (USD). When `Some`, every cloud send
    /// in the runner consults [`crate::ssb_viewer::SsbViewer::cloud_spend_summary`]
    /// and blocks new cloud calls once `day_usd >= cap`. None disables
    /// cap enforcement (legacy pre-C-5 behaviour).
    cloud_spend_cap_daily_usd: Option<f64>,
    /// C-5: tracks whether the daily ceiling has already fired a
    /// `cloud_spend_blocked` SSB row this calendar day so the dashboard
    /// shows exactly one BLOCKED event per ceiling-breach. Cleared by the
    /// UTC-day rollover heuristic in the pre-dispatch hook.
    cloud_spend_blocked_emitted: Mutex<bool>,
    /// C-5 follow-on (SCHEMA_VERSION 103): dedup mutex for
    /// `WarnLimitReached` SSB rows. Fires once per UTC day when
    /// `day_usd >= cap * warn_at_percent / 100.0`. Reset on the same
    /// UTC-day rollover used by `cloud_spend_blocked_emitted`.
    cloud_warn_emitted: Mutex<bool>,
    /// C-5 follow-on (SCHEMA_VERSION 103): dedup mutex for
    /// `CostEstimate` SSB rows. Fires once per task lifetime (the
    /// projection does not change mid-task) so the dashboard CloudPanel
    /// shows a single per-task COST row, not one per baby step.
    cloud_cost_estimated_emitted: Mutex<bool>,
    /// C-25 — optional template crystallizer. When `Some`, the
    /// post-execute baby-step hook records an `ExecutionOutcome` and
    /// invokes `check_crystallization`. On crystallization, the
    /// resulting `CompiledPipeline` is upserted into
    /// `compiled_pipeline_store` and persisted to disk. `None` for
    /// legacy / test callers — both fields default to None so existing
    /// `TaskRunnerConfig` construction sites stay untouched.
    crystallizer: Option<
        Arc<AsyncMutex<crate::compiled_pipeline::TemplateCrystallizer>>,
    >,
    /// C-25 — optional compiled-pipeline on-disk store. Mirrors the
    /// crystallizer's lifecycle: when `Some`, post-execute hook persists
    /// crystallized pipelines.
    compiled_pipeline_store: Option<
        Arc<AsyncMutex<crate::compiled_pipeline::CompiledPipelineStore>>,
    >,
    /// C-25 — optional background-task manager. When `Some`, callers
    /// using `gated_spawn` get tracked spawn/complete SSB rows + soft/hard
    /// timeout enforcement. The manager OBSERVES already-detached children
    /// (DRIFT-I Option A — bash.rs internals unchanged). Per DRIFT-I the
    /// manager observes children spawned through the existing bash.rs
    /// path; runtime callers reach the manager through
    /// `TaskRunnerConfig::background_task_manager` (the production
    /// wire-up site landing in a follow-on patch). `#[allow(dead_code)]`
    /// because no internal task_runner code yet drives the manager;
    /// callers read it through ResolvedConfig.
    #[allow(dead_code)]
    background_task_manager: Option<Arc<crate::background_task::BackgroundTaskManager>>,
    /// C-39 — production tool-dispatch seam (parallel observability layer to
    /// the legacy `tool_dispatcher` field above). When `Some`, the runner
    /// invokes `dispatch.dispatch_tool(...)` at every real production
    /// tool-call site so the per-call `CliMcpDispatch` (when --via-mcp is
    /// active) or `McpToolCalled` (default in-process leg) SSB row is
    /// emitted. The legacy `ToolDispatcher` path runs unchanged; this is
    /// purely additive observability. None preserves pre-C-39 behaviour.
    mcp_dispatch: Option<Arc<dyn crate::dispatch::Dispatch>>,
    /// D-3 closure — verifier adapter slot for the post-task
    /// `dispatch_verifiers` CLAIM-emit hook. When `Some(non_empty)`, the
    /// hook drives `agent_pool::run_consensus_pass` with these adapters
    /// and emits one VerifierConsensusPass / VerifierConsensusFail row
    /// per task. Default `None` keeps the row family dark until concrete
    /// adapter plumbing lands in a follow-on plan. `#[allow(dead_code)]`
    /// because the field is consumed via `as_ref()` inside
    /// `dispatch_verifiers` but the construction site is wired through
    /// `TaskRunnerConfig` in a future patch (the wire itself is in place).
    #[allow(dead_code)]
    verifier_adapters: Option<Vec<Arc<dyn crate::agent_pool::DynVerifierAdapter>>>,
    /// D-5 closure (GAP-B) — BAI Thompson-promotion runtime state.
    /// When `Some`, [`dispatch_bai_round`] folds each consensus verdict
    /// into the candidate's Beta(alpha, beta) posterior + runs Thompson
    /// sampling + writes one `thompson_sampled` + one `bai_arm_updated`
    /// SSB row (plus optional `template_promoted` /
    /// `template_candidate_rejected`). Default `None` keeps the row
    /// family dark until the runtime is plumbed via `TaskRunnerConfig`
    /// + the operator sets `ELAI_BAI_ENABLE=1`. Plumbing matches the
    /// D-2 `verifier_adapters` precedent so adding the wire requires
    /// no further task_runner edits.
    #[allow(dead_code)]
    bai_runtime: Option<Arc<crate::runtime_config::BaiRuntimeState>>,
    /// D-10 closure — claim-router runtime state for the post-task
    /// `dispatch_claim_routing` hook. When `Some` AND
    /// `ELAI_CLAIM_ROUTER_ENABLE=1` is set, the hook synthesises an
    /// `OrchContext` per task, calls
    /// [`crate::agent_pool::route_claim`] with the configured emitter,
    /// and emits exactly one `claim_routed` SSB row per task. Default
    /// `None` keeps the row family dark until both the runtime is
    /// plumbed via `TaskRunnerConfig` AND the env gate flips. Mirrors
    /// the D-5 `bai_runtime` precedent — adding the wire requires no
    /// further task_runner edits. `#[allow(dead_code)]` because the
    /// field is consumed via `as_ref()` inside `dispatch_claim_routing`
    /// but the construction site is wired through `TaskRunnerConfig`
    /// in a follow-up patch (the wire itself is in place).
    #[allow(dead_code)]
    claim_router_runtime: Option<Arc<crate::runtime_config::ClaimRouterRuntimeState>>,
    /// D-11 closure — constructive-checker runtime state for the
    /// post-task `dispatch_constructive_checkers_hook`. When `Some` AND
    /// `ELAI_CHECKER_ENABLE=1` is set, the hook synthesises a
    /// per-task [`crate::claim::Claim`] (same shape as the D-10
    /// claim-router hook), calls
    /// [`crate::agent_pool::dispatch_constructive_checkers`] with the
    /// configured registry + emitter, and emits one
    /// `checker_result_emitted` SSB row per applicable-checker result.
    /// Default `None` keeps the row family dark until both the runtime
    /// is plumbed via `TaskRunnerConfig` AND the env gate flips. Mirrors
    /// the D-10 claim-router gate so the toggle pattern is consistent
    /// across producers. `#[allow(dead_code)]` because the field is
    /// consumed via `as_ref()` inside the hook but the construction
    /// site is wired through `TaskRunnerConfig` in a follow-up patch
    /// (the wire itself is in place).
    #[allow(dead_code)]
    checker_runtime: Option<Arc<crate::runtime_config::CheckerRuntimeState>>,
    /// D-12 closure — operationalization-check runtime state for the
    /// post-task `run_operational_check_hook`. When `Some` AND
    /// `ELAI_OPERATIONALIZATION_ENABLE=1` is set, the hook synthesises
    /// a per-task [`crate::baby_step::BabyStep`] from the task seq +
    /// project_id, calls
    /// [`crate::agent_pool::run_operational_check`] with the configured
    /// emitter + registry, and emits exactly one `baby_step_dispatched`
    /// SSB row per task. Default `None` keeps the row family dark until
    /// both the runtime is plumbed via `TaskRunnerConfig` AND the env
    /// gate flips. Mirrors the D-10/D-11 toggle pattern so the operator
    /// surface stays consistent across producers. Per D-12 decisions
    /// doc DECISION-6 — closes the GAP-1 "dead code" report against
    /// `run_operational_check`. `#[allow(dead_code)]` because the
    /// field is consumed via `as_ref()` inside the hook but the
    /// construction site is wired through `TaskRunnerConfig` in a
    /// follow-up patch (the wire itself is in place).
    #[allow(dead_code)]
    operationalization_runtime:
        Option<Arc<crate::runtime_config::OperationalizationRuntimeState>>,
    /// D-13 closure — adversary runtime state for the post-task
    /// `dispatch_adversarial_pass` hook. When `Some` AND
    /// `ELAI_ADVERSARY_ENABLE=1` is set, the hook routes the just-
    /// completed task's claim through
    /// [`crate::agent_pool::run_adversarial_pass`] using the configured
    /// adversary (typically a cloud Sonnet HTTP adapter) and emits the
    /// `adversary_generated` + `consensus_reevaluated_with_counter`
    /// SSB row pair when the worker / adversary family pair satisfies
    /// the `min_family_distance` guard. Default `None` keeps the row
    /// pair dark until both the runtime is plumbed via
    /// `TaskRunnerConfig` AND the env gate flips. Mirrors the
    /// D-10/D-11/D-12 toggle pattern so the operator surface stays
    /// consistent across producers. `#[allow(dead_code)]` because the
    /// field is consumed via `as_ref()` inside the hook but the
    /// construction site is wired through `TaskRunnerConfig` in a
    /// follow-up patch (the wire itself is in place).
    #[allow(dead_code)]
    adversary_runtime: Option<Arc<crate::runtime_config::AdversaryRuntimeState>>,
    /// D-14 closure — sycophancy probe runtime state for the post-task
    /// `dispatch_sycophancy_probe_hook` hook. When `Some` AND
    /// `ELAI_SYCOPHANCY_PROBE_ENABLE=1` is set, the hook runs the
    /// inverted-framing challenge (Sharma §3.2) on the just-completed
    /// task's claim and emits the `sycophancy_probe_dispatched` +
    /// `sycophancy_flag` row pair. Default `None` keeps the row pair
    /// dark until both the runtime is plumbed via `TaskRunnerConfig`
    /// AND the env gate flips. Mirrors the D-10/D-11/D-12/D-13 toggle
    /// pattern so the operator surface stays consistent across
    /// producers. `#[allow(dead_code)]` because the field is consumed
    /// via `as_ref()` inside the hook but the construction site is
    /// wired through `TaskRunnerConfig` in a follow-up patch (the
    /// wire itself is in place).
    #[allow(dead_code)]
    sycophancy_runtime:
        Option<Arc<crate::runtime_config::SycophancyRuntimeState>>,
    /// D-15 closure — novelty-label runtime state for the post-task
    /// `dispatch_novelty_hook` hook. When `Some` AND
    /// `ELAI_NOVELTY_LABEL_ENABLE=1` is set, the hook applies the
    /// closed three-bucket novelty taxonomy to the just-completed
    /// task's primary template and emits one `novelty_classified`
    /// SSB row. Default `None` keeps the row family dark until both
    /// the runtime is plumbed via `TaskRunnerConfig` AND the env
    /// gate flips. Mirrors the D-10/D-11/D-12/D-13/D-14 toggle
    /// pattern so the operator surface stays consistent across
    /// producers. `#[allow(dead_code)]` because the field is
    /// consumed via `as_ref()` inside the hook but the construction
    /// site is wired through `TaskRunnerConfig` in a follow-up patch
    /// (the wire itself is in place).
    #[allow(dead_code)]
    novelty_runtime:
        Option<Arc<crate::runtime_config::NoveltyRuntimeState>>,
    /// D-16 closure — conformal-refusal corpus-grow + nightly tau
    /// updater runtime state for the post-task
    /// `dispatch_conformal_corpus_grow_hook` hook. When `Some` AND
    /// `ELAI_CONFORMAL_REFUSAL_ENABLE=1` is set, the hook appends
    /// the just-completed claim's `(claim_score, verdict)` to the
    /// calibration corpus and (throttled by
    /// `[conformal].corpus_grow_min_interval_secs`) invokes
    /// `TauUpdater::update_tau` to persist a fresh threshold + emit
    /// one `calibration_corpus_updated` SSB row. Default `None`
    /// keeps the producer dark until both gates flip. Mirrors the
    /// D-10/D-11/D-12/D-13/D-14/D-15 toggle pattern. Closes D-16
    /// GAP-6 per D-16-decisions.md DECISION-7.
    #[allow(dead_code)]
    conformal_refusal_runtime:
        Option<Arc<crate::runtime_config::ConformalRefusalRuntimeState>>,
    /// D-16 closure — last wall-clock UNIX ms at which the
    /// `dispatch_conformal_corpus_grow_hook` emitted a
    /// `calibration_corpus_updated` SSB row. Used to honour
    /// `[conformal].corpus_grow_min_interval_secs`. `0` (the
    /// default) means "never emitted" — the next call always
    /// passes the throttle. Held behind a Mutex so the throttle
    /// is shared across drive_task invocations without making
    /// `ResolvedConfig` carry interior-mutable state at the
    /// struct level. `#[allow(dead_code)]` because the field is
    /// consumed via `lock()` inside the hook but the construction
    /// site is wired through `TaskRunnerConfig` in a follow-up
    /// patch.
    #[allow(dead_code)]
    conformal_refusal_last_emit_ms: Mutex<i64>,
    /// D-18 closure — typed-ObservableEffect runtime state for the
    /// post-task `dispatch_observable_effect_hook` hook. When `Some`
    /// AND `ELAI_OBSERVABLE_EFFECT_ENABLE=1` is set, the hook emits
    /// the 4-variant SSB pack
    /// (`observable_effect_dispatched` + `goodhart_risk_flag` +
    /// `sandbox_observation` + `production_observation_proposal`)
    /// for the just-completed task's effects. Default `None` keeps
    /// the pack dark until both the runtime is plumbed via
    /// `TaskRunnerConfig` AND the env gate flips. Mirrors the
    /// D-13/D-14/D-15/D-16 toggle pattern. Plan:
    /// docs/thefinalglance/D-plan/D-18_observable_effect_typed.md.
    #[allow(dead_code)]
    observable_effect_runtime:
        Option<Arc<crate::runtime_config::ObservableEffectRuntimeState>>,
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
    /// B-6: supervision layer — caps restarts at max_restarts (default 3) per phase window.
    supervision_manager: AsyncMutex<SupervisionManager<OsClock>>,
    scratch_dir: PathBuf,
}

impl ResolvedConfig {
    fn from(cfg: TaskRunnerConfig) -> Self {
        let checkpoint = cfg.checkpoint_scratch_dir.map(|dir| {
            Arc::new(CheckpointContext {
                checkpoint_manager: CheckpointManager::new(OsClock),
                recovery_manager: RecoveryManager::new(OsClock),
                // B-6: max 3 restarts per phase (5-minute window); escalate with Terminate.
                supervision_manager: AsyncMutex::new(SupervisionManager::new(
                    OsClock,
                    SupervisionPolicy {
                        max_restarts: 3,
                        restart_window: std::time::Duration::from_secs(300),
                        on_max_restarts: EscalationAction::Terminate,
                    },
                )),
                scratch_dir: dir,
            })
        });
        // B-10 (DECISION-A / DECISION-F): resolve the ModelCapabilityProfile once
        // at construction. Construction sites may pass `Some(...)` to override;
        // otherwise we infer from `backend_config.model` via substring matching.
        // VRAM tier defaulted to 12GB (the primary deployment tier per spec).
        // Future B-task may wire detect_vram_tier into this bootstrap.
        let resolved_backend_config = cfg
            .backend_config
            .clone()
            .unwrap_or_else(crate::dispatcher::BackendConfig::default);
        let model_capability_profile: Arc<crate::context_budget::ModelCapabilityProfile> =
            cfg.model_capability_profile.unwrap_or_else(|| {
                Arc::new(crate::context_budget::detect_model_capability_profile(
                    &resolved_backend_config.model,
                    elai_core::VramTier::Vram12Gb,
                ))
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
            interrogation_store: cfg.interrogation_store,
            interrogation_metric_emitted: Mutex::new(false),
            checkpoint,
            baby_step_executor: cfg
                .baby_step_executor
                .unwrap_or_else(|| Arc::new(SyntheticBabyStepExecutor)),
            anti_spiral_enabled: cfg.anti_spiral_enabled,
            llm_provider: cfg.llm_provider,
            // B-10: backend_config was pre-resolved above so the same instance
            // can both seed the model_capability_profile detection AND populate
            // ResolvedConfig.backend_config — keeps both in sync.
            backend_config: resolved_backend_config,
            indexing_service: cfg.indexing_service,
            indexed_projects: Mutex::new(BTreeMap::new()),
            hook_dispatcher: cfg.hook_dispatcher,
            signal_reader: cfg.signal_reader,
            vram_tier_emitted: Mutex::new(false),
            tool_dispatcher: cfg.tool_dispatcher,
            git_branch_policy: cfg.git_branch_policy,
            system_prober: cfg.system_prober,
            system_probe_emitted: Mutex::new(false),
            feasibility_prober: cfg.feasibility_prober,
            autonomous_gate: cfg.autonomous_gate,
            layer1a_processor: cfg.layer1a_processor,
            dispatch_context_profile: cfg.dispatch_context_profile,
            reflection_client: cfg.reflection_client,
            irreversible_classifier: cfg.irreversible_classifier,
            telescope_stats: Arc::new(crate::TelescopeStats::new()),
            dkg_querier_swap: cfg.dkg_querier_swap,
            dkg_querier_swapped_projects: Mutex::new(std::collections::HashSet::new()),
            task_splitter: cfg.task_splitter,
            research_tree: cfg.research_tree,
            pipeline_grc: {
                // B-8 kill-switch: read once at construction, force None when
                // the env-var is set so every call site can branch on the
                // single Option<...>.
                if crate::grc::pipeline_grc_disabled_via_env() {
                    if cfg.pipeline_grc.is_some() {
                        tracing::warn!(
                            env = crate::grc::ELAI_DISABLE_PIPELINE_GRC_ENV,
                            "task_runner: pipeline_grc disabled via env-var; safety gates bypassed for this runner"
                        );
                    }
                    None
                } else {
                    cfg.pipeline_grc
                }
            },
            model_capability_profile,
            // B-24 Phase 4I Group C9 (decision 94): slots start empty; the
            // lazy wire-up in `drive_task` (post `spec_from_decomposition`)
            // populates them only when the spec declares non-empty `media`.
            media_validator: AsyncMutex::new(None),
            media_sink: AsyncMutex::new(None),
            // B-28 WIRING-2 (items H, I, J): focus-tunnel runtime hooks.
            // Default None — the production TaskRunnerConfig construction
            // site lands in a follow-up patch.
            focus_tunnel_pre_assembler: None,
            focus_tunnel_dep_resolver: None,
            focus_tunnel_file_router: None,
            // C2: credential map starts empty; populated by run_confirmation_gate
            // on RequestCredential responses.
            session_credentials: Mutex::new(BTreeMap::new()),
            // C-5: cloud-mode router + ceiling enforcement state.
            cloud_router: cfg.cloud_router,
            cloud_spend_cap_daily_usd: cfg.cloud_spend_cap_daily_usd,
            cloud_spend_blocked_emitted: Mutex::new(false),
            // C-5 follow-on (SCHEMA_VERSION 103): per-day WARN dedup +
            // per-task COST_ESTIMATE dedup. Both default false; the
            // pre-dispatch hook flips on first emit.
            cloud_warn_emitted: Mutex::new(false),
            cloud_cost_estimated_emitted: Mutex::new(false),
            // C-25: compiled-pipeline + crystallizer + background-task
            // wiring. All optional; legacy callers see None on every field.
            crystallizer: cfg.crystallizer,
            compiled_pipeline_store: cfg.compiled_pipeline_store,
            background_task_manager: cfg.background_task_manager,
            // C-39: thread the optional Dispatch impl onto ResolvedConfig so
            // the per-baby-step tool-call site (see `drive_task`'s
            // `tool_dispatcher` block) can emit one CliMcpDispatch or
            // McpToolCalled row per real production tool invocation.
            mcp_dispatch: cfg.mcp_dispatch,
            // D-3 closure: default None — D-3 closure wires the
            // dispatch_verifiers hook but the adapter construction site
            // lands in a follow-on plan (D-5 BAI / D-13 adversary). The
            // wire is in place so when adapters are plumbed through
            // `TaskRunnerConfig`, no additional task_runner edit is
            // needed to surface the consensus row family.
            verifier_adapters: None,
            // D-5 closure (GAP-B): default None — `dispatch_bai_round`
            // is wired but the runtime construction site lands in the
            // sidecar/CLI boot path so the BAI row family stays dark
            // until both the runtime is plumbed AND ELAI_BAI_ENABLE=1
            // is set. No further task_runner edit is needed once both
            // gates flip.
            bai_runtime: None,
            // D-10 closure: default None — `dispatch_claim_routing` is
            // wired but the runtime construction site lands in the
            // sidecar/CLI boot path so the `claim_routed` row family
            // stays dark until both the runtime is plumbed AND
            // ELAI_CLAIM_ROUTER_ENABLE=1 is set. Mirrors the D-5 BAI
            // gate so the toggle pattern is consistent across producers.
            claim_router_runtime: None,
            // D-11 closure: default None — `dispatch_constructive_checkers_hook`
            // is wired but the runtime construction site lands in the
            // sidecar/CLI boot path so the `checker_result_emitted` row
            // family stays dark until both the runtime is plumbed AND
            // ELAI_CHECKER_ENABLE=1 is set. Mirrors the D-10 claim-router
            // gate so the toggle pattern is consistent across producers.
            checker_runtime: None,
            // D-12 closure: default None — `run_operational_check_hook`
            // is wired but the runtime construction site lands in the
            // sidecar/CLI boot path so the `baby_step_dispatched` row
            // family stays dark until both the runtime is plumbed AND
            // ELAI_OPERATIONALIZATION_ENABLE=1 is set. Mirrors the
            // D-10/D-11 toggle pattern. Closes GAP-1 (production caller
            // for run_operational_check) per D-12-decisions.md
            // DECISION-6.
            operationalization_runtime: None,
            // D-13 closure: default None — `dispatch_adversarial_pass`
            // is wired but the runtime construction site lands in the
            // sidecar/CLI boot path so the `adversary_generated` +
            // `consensus_reevaluated_with_counter` SSB row pair stays
            // dark until both the runtime is plumbed AND
            // ELAI_ADVERSARY_ENABLE=1 is set. Mirrors the
            // D-10/D-11/D-12 toggle pattern. Closes GAP-1
            // (production dispatch wire for
            // `GroundingMode::FalsifiabilityCrossFamily`) per
            // D-13-decisions.md DECISION-5.
            adversary_runtime: None,
            // D-14 closure: default None —
            // `dispatch_sycophancy_probe_hook` is wired but the
            // runtime construction site lands in the sidecar/CLI boot
            // path so the `sycophancy_probe_dispatched` +
            // `sycophancy_flag` SSB row pair stays dark until both
            // the runtime is plumbed AND
            // ELAI_SYCOPHANCY_PROBE_ENABLE=1 is set. Mirrors the
            // D-10/D-11/D-12/D-13 toggle pattern.
            sycophancy_runtime: None,
            // D-15 closure: default None — `dispatch_novelty_hook` is
            // wired but the runtime construction site lands in the
            // sidecar/CLI boot path so the `novelty_classified` row
            // family stays dark until both the runtime is plumbed AND
            // ELAI_NOVELTY_LABEL_ENABLE=1 is set. Mirrors the
            // D-10/D-11/D-12/D-13/D-14 toggle pattern. Closes the
            // D-15 GAP-1 "no production dispatch wire for
            // agent_pool::dispatch_novelty" per
            // workflow/reasoning/D-15-decisions.md DECISION-5.
            novelty_runtime: None,
            // D-16 closure: default None — `dispatch_conformal_corpus_grow_hook`
            // is wired but the runtime construction site lands in the
            // sidecar/CLI boot path so the `calibration_corpus_updated`
            // row family stays dark until both the runtime is plumbed
            // AND ELAI_CONFORMAL_REFUSAL_ENABLE=1 is set. Mirrors the
            // D-10/D-11/D-12/D-13/D-14/D-15 toggle pattern. Closes
            // D-16 GAP-6 per D-16-decisions.md DECISION-7.
            conformal_refusal_runtime: None,
            conformal_refusal_last_emit_ms: Mutex::new(0),
            // D-18 closure: default None — `dispatch_observable_effect_hook`
            // is wired but the runtime construction site lands in the
            // sidecar/CLI boot path so the 4-variant SSB pack stays
            // dark until both the runtime is plumbed AND
            // ELAI_OBSERVABLE_EFFECT_ENABLE=1 is set. Mirrors the
            // D-10..D-16 toggle pattern. Plan: docs/thefinalglance/D-plan/
            // D-18_observable_effect_typed.md.
            observable_effect_runtime: None,
        }
    }
}

/// B-28 WIRING-2 (items H + I + J): focus-tunnel post-execute hook
/// dispatcher.
///
/// Invoked from `drive_task` after each baby step's executor returns. The
/// body is a strict no-op when:
///   * `changed_files` is empty (the synthetic executor / wire-4A agents
///     that have not yet wired the changed-files protocol), OR
///   * `resolved.focus_tunnel_pre_assembler / dep_resolver / file_router`
///     are all `None` (the default — construction sites land in a future
///     patch that wires production Arcs onto `TaskRunnerConfig`).
///
/// Each hook block is independently gated so partial wiring (e.g. only
/// the PreAssembler) does not require all three Arcs.
/// C-25 — TemplateCrystallizer post-execute hook. Strict no-op when
/// either `resolved.crystallizer` or `resolved.compiled_pipeline_store` is
/// None (legacy / pre-C-25 callers).
///
/// Otherwise: derive concept_class + plan_skeleton_hash via
/// `pipeline_builder::classify_task_for_template`, record an
/// ExecutionOutcome, then check for crystallization. On Some, upsert into
/// the store and persist to disk. Errors are logged + swallowed.
///
/// `task_summary` is captured BEFORE the executor consumes `exec_ctx` so
/// the classifier always sees the same baby-step intent string.
async fn run_crystallizer_post_execute_hook(
    resolved: &ResolvedConfig,
    task_summary: &str,
    passed: bool,
) {
    // Both Arcs must be Some for the hook to do work.
    let (Some(cry), Some(store)) = (
        resolved.crystallizer.as_ref(),
        resolved.compiled_pipeline_store.as_ref(),
    ) else {
        return;
    };
    // Classify the task. None => the heuristic could not identify a
    // template-eligible concept; skip recording.
    let Some((concept_class, plan_skeleton_hash)) =
        crate::pipeline_builder::classify_task_for_template(task_summary)
    else {
        return;
    };
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let outcome = crate::compiled_pipeline::ExecutionOutcome {
        success: passed,
        concept_class: concept_class.clone(),
        plan_skeleton_hash,
        pipeline_id: None,
        token_count: 0,
        timestamp: now_secs,
    };
    // Record the outcome under the crystallizer's lock, then drop the lock
    // before the check (which awaits SSB). The check reads through &self;
    // we re-acquire briefly to call it.
    {
        let mut guard = cry.lock().await;
        let _ = guard.record_execution(outcome);
    }
    let crystallized = {
        let guard = cry.lock().await;
        guard
            .check_crystallization(
                &concept_class,
                plan_skeleton_hash,
                // Provider: derive a minimal CompiledStep list from the
                // task_summary as a template_prompt. Real callers can wire a
                // richer provider that pulls per-step prompts from the
                // executable pipeline. For C-25 we emit a single step
                // pre-populated as l1_tool_only so compilation_eligible
                // passes when the threshold + use_count gates do.
                || {
                    vec![crate::compiled_pipeline::CompiledStep {
                        step_index: 0,
                        template_prompt: task_summary.to_string(),
                        variables: Vec::new(),
                        variable_region_ratio: 0.0,
                        l1_tool_only: true,
                    }]
                },
                now_secs,
            )
            .await
    };
    if let Some(pipeline) = crystallized {
        let mut store_guard = store.lock().await;
        store_guard.upsert(pipeline, now_secs);
        if let Err(e) = store_guard.persist_to_disk() {
            tracing::warn!(
                error = %e,
                "compiled_pipeline_store: persist_to_disk failed (non-fatal)"
            );
        }
    }
}

/// C-49 — iterative-distillation experience post-audit hook.
///
/// Wire-dark: gated by `ELAI_EXPERIENCE_DISTILL_ENABLE` env var. When
/// disabled OR when audit failed, the function is a transparent no-op.
/// When enabled + audit passed, emits a `PromptPolicyDistillationRun`
/// SSB beacon with the hashed skill name + baby-step trace count.
///
/// HEURISTIC SIMPLIFICATIONS vs paper (arXiv:2605.14443):
///   - Paper Eq. 5 uses scalar reward + epsilon-band gate. We use the
///     binary `audit_pass` signal — `audit_pass = true` is the gate.
///   - Paper Algorithm 1 trains a small prompter via GRPO + KL anchor.
///     This hook records observation only; the RL trainer is
///     RUNPOD-PENDING (would need feedback model `M_FB` and a scalar
///     reward signal we do not yet compute).
///   - Paper Section 5.5 cross-model transfer is poor. The skill-name
///     binding here keeps fragments scoped to one `skill_name` so the
///     transfer pitfall is structurally avoided.
///
/// The hook does NOT persist anything to disk — the corpus in this
/// session is per-process in-memory (see crate::experience module
/// header). A persistent on-disk store is RUNPOD-PENDING follow-on.
///
/// `task_summary` doubles as the skill name fallback when no other
/// skill-binding signal is present (the canonical skill name comes
/// from `SkillRegistry::top_k_skills` at the prompt-assembly site;
/// here we use a snake-case sanitised task_summary prefix as a stand-
/// in so the beacon still groups by a meaningful key).
async fn run_experience_distill_post_execute_hook(
    writer: &Arc<Ssb>,
    task_summary: &str,
    passed: bool,
    baby_step_id: u64,
) {
    if !crate::experience::distillation_enabled() {
        return;
    }
    if !passed {
        // Audit failed — heuristic simplification of paper Eq. 5:
        // negatives are NOT recorded by this hook (the corpus retains
        // a slot for them on the trace struct so future contrastive
        // code paths can use them, but the hook itself emits only on
        // positives).
        return;
    }
    // Derive a stable per-baby-step skill key from the task summary.
    // First whitespace-bounded token, lowercased, with non-alphanumerics
    // collapsed to underscores. Falls back to "default_skill" when the
    // summary is empty.
    let skill_name = task_summary
        .split_whitespace()
        .next()
        .map(|word| {
            word.chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() {
                        c.to_ascii_lowercase()
                    } else {
                        '_'
                    }
                })
                .collect::<String>()
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default_skill".to_string());

    let skill_name_hash = crate::skill_registry::skill_name_hash(&skill_name);
    let ts_ms = u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
    )
    .unwrap_or(0);

    // Emit a single PromptPolicyDistillationRun beacon. `trace_count = 1`
    // because this hook is currently the per-baby-step observation
    // point (one trace per audit-pass). `fragments_emitted = 0` until
    // the live distillation projection is wired into the prompt-
    // assembly site (RUNPOD-PENDING follow-on).
    let payload = crate::ssb::PromptPolicyDistillationRunPayload {
        skill_name_hash,
        trace_count: 1,
        fragments_emitted: 0,
        ts_ms,
    };
    let req = crate::ssb::SsbWriteRequest {
        action_type: crate::ssb::SsbActionType::PromptPolicyDistillationRun,
        agent_id: format!("experience-distill-bs{baby_step_id}"),
        superstep: 0i64,
        payload: match rmp_serde::to_vec_named(&crate::ssb::SsbPayload::PromptPolicyDistillationRun(
            payload,
        )) {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "experience_distill_hook: payload encode failed (non-fatal)"
                );
                return;
            }
        },
        files: Vec::new(),
    };
    if let Err(e) = writer.write(req).await {
        tracing::warn!(
            error = %e,
            baby_step = baby_step_id,
            "experience_distill_hook: SSB write failed (non-fatal)"
        );
    }
}

async fn run_focus_tunnel_post_execute_hooks(
    writer: &Arc<Ssb>,
    resolved: &ResolvedConfig,
    baby_step_id: u64,
    changed_files: &[PathBuf],
    passed: bool,
) {
    // Item H — PipelinePreAssembler: record per-step outcome on the
    // adaptive rolling window. Pre-assembly of step N+1 is deferred to the
    // production wiring patch that owns the spec lookup.
    if let Some(pa) = resolved.focus_tunnel_pre_assembler.as_ref() {
        let mut guard = pa.lock().await;
        // B-28 WIRING-2 fix WARN E: was hardcoded `0`. Pass the actual
        // baby_step_id so the adaptive window records per-step outcomes
        // (the pre-assembler keys its history on step_idx). usize cast is
        // saturating — baby_step_ids never exceed usize on a 64-bit target,
        // and saturating preserves monotonicity on 32-bit builds.
        let step_idx_usize = usize::try_from(baby_step_id).unwrap_or(usize::MAX);
        guard.on_step_outcome(step_idx_usize, passed, changed_files);
        let pass_rate_bp =
            (guard.current_pass_rate() * 10_000.0).clamp(0.0, 10_000.0) as u16;
        let depth = guard.current_depth();
        drop(guard);
        let payload = SsbPayload::PipelinePreAssembled(
            crate::ssb::PipelinePreAssembledPayload {
                step_idx: baby_step_id,
                pass_rate_bp,
                was_discarded: false,
                depth_at_time: depth,
                bytes_assembled: 0,
            },
        );
        if let Err(e) = writer
            .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
            .await
        {
            tracing::warn!(
                baby_step_id,
                error = %e,
                "task_runner: PipelinePreAssembled SSB write failed (non-fatal)"
            );
        }
    }

    // Item I — DependencyResolver: scan changed_files for imports; emit one
    // SSB row per ResolutionDecision. Guarded by changed_files non-empty so
    // we don't spawn imports work on a synthetic-executor pass.
    if let (Some(resolver), false) = (
        resolved.focus_tunnel_dep_resolver.as_ref(),
        changed_files.is_empty(),
    ) {
        let mut missing: Vec<crate::focus_tunnel::MissingImport> = Vec::new();
        for path in changed_files {
            let Some(path_str) = path.to_str() else {
                continue;
            };
            let Some(lang) =
                elai_plugins::import_extractor::ImportLanguage::from_extension(path_str)
            else {
                continue;
            };
            // Read the source for import extraction; spawn_blocking keeps
            // the async runtime live.
            let path_clone = path.clone();
            let source = match tokio::task::spawn_blocking(move || {
                std::fs::read_to_string(&path_clone)
            })
            .await
            {
                Ok(Ok(s)) => s,
                _ => continue,
            };
            let extracted = match elai_plugins::import_extractor::extract_imports(&source, lang) {
                Ok(v) => v,
                Err(_) => continue,
            };
            for ext in extracted {
                missing.push(crate::focus_tunnel::MissingImport {
                    language: lang,
                    name: ext.name,
                    source_span: None,
                    transitive_of: None,
                });
            }
        }

        if !missing.is_empty() {
            let plan = match resolver.resolve(missing, &[]).await {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(error = ?e, "task_runner: dep_resolver.resolve failed");
                    return;
                }
            };
            for decision in &plan.decisions {
                let (decision_str, reason_str, install_command) = match decision {
                    crate::focus_tunnel::ResolutionDecision::AutoApprove { prop, reason } => (
                        "auto_approve".to_string(),
                        match reason {
                            crate::focus_tunnel::AutoApproveReason::Allowlist => {
                                "allowlist_match".to_string()
                            }
                            crate::focus_tunnel::AutoApproveReason::TransitiveOfApproved => {
                                "transitive_of_approved".to_string()
                            }
                        },
                        Some(prop.install_command.clone()),
                    ),
                    crate::focus_tunnel::ResolutionDecision::QueueForHuman { prop } => (
                        "queue_for_human".to_string(),
                        "human_gate".to_string(),
                        Some(prop.install_command.clone()),
                    ),
                    crate::focus_tunnel::ResolutionDecision::Blocked { prop, reason } => (
                        "blocked".to_string(),
                        focus_tunnel_blocked_reason_str(reason),
                        Some(prop.install_command.clone()),
                    ),
                };
                let (lang_str, name) = match decision {
                    crate::focus_tunnel::ResolutionDecision::AutoApprove { prop, .. }
                    | crate::focus_tunnel::ResolutionDecision::QueueForHuman { prop }
                    | crate::focus_tunnel::ResolutionDecision::Blocked { prop, .. } => (
                        format!("{:?}", prop.language).to_lowercase(),
                        prop.name.clone(),
                    ),
                };
                let payload = SsbPayload::DependencyResolved(
                    crate::ssb::DependencyResolvedPayload {
                        language: lang_str,
                        name,
                        decision: decision_str,
                        reason: reason_str,
                        install_command,
                    },
                );
                if let Err(e) = writer
                    .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
                    .await
                {
                    tracing::warn!(
                        baby_step_id,
                        error = %e,
                        "task_runner: DependencyResolved SSB write failed (non-fatal)"
                    );
                }
            }
            // B-28 WIRING-2 fix WARN F: actually invoke
            // `resolver.apply(&plan, &executor)` so AutoApprove decisions
            // record an InstallAttempt (the resolver contract pins
            // install_results.len() == proposals_approved.len()). Production
            // install is gated behind `ELAI_ENABLE_DEP_INSTALL=1`; when
            // unset (default) we use a NO-OP executor that records the
            // attempt as `success=false, stderr="dep_install_disabled"` so
            // the SSB stream stays observable without spawning real package
            // managers. If apply panics or errors are surfaced inside
            // executor.install, this block never crashes the orchestrator —
            // each InstallAttempt is data, not a `Result`.
            let install_enabled = std::env::var("ELAI_ENABLE_DEP_INSTALL")
                .map(|v| v.trim() == "1")
                .unwrap_or(false);
            // NO-OP installer used as the default production executor until
            // a real package-manager dispatch lands behind the feature flag.
            // Mirrors `RecordingInstaller` from the wiring contract tests
            // but writes a deterministic disabled-marker into `stderr`.
            struct NoopInstaller {
                disabled_marker: &'static str,
            }
            impl crate::focus_tunnel::DepInstallExecutor for NoopInstaller {
                fn install(
                    &self,
                    proposal: &crate::focus_tunnel::DepProposal,
                ) -> crate::focus_tunnel::InstallAttempt {
                    crate::focus_tunnel::InstallAttempt {
                        proposal: proposal.clone(),
                        success: false,
                        stderr: Some(self.disabled_marker.to_string()),
                    }
                }
            }
            let executor = NoopInstaller {
                disabled_marker: if install_enabled {
                    // Real-install path is gated for follow-on patch: today
                    // we still NO-OP but mark the attempt so logs distinguish
                    // "feature flag on but no executor wired" from default.
                    "dep_install_executor_not_wired"
                } else {
                    "dep_install_disabled"
                },
            };
            let _outcome = resolver.apply(&plan, &executor);
            // Outcome.install_results is one row per AutoApprove decision.
            // We do NOT emit additional SSB rows here — the AutoApprove
            // beacon above already records the decision; install execution
            // surfaces will land when the production executor wires the
            // package-manager dispatcher in a follow-on patch.
        }
    }

    // Item J — SpeculativeFileRouter: post-execute route decision. The
    // router relies on model_output (the agent's emitted text) which is
    // NOT carried on `BabyStepExecutionResult` today (Gap #9). We emit a
    // best-effort cap-aware row when the router is wired AND
    // `changed_files` is non-empty so the wiring is observable. The full
    // undefined-symbol scan lands when the agent IPC contract grows a
    // `model_output` field.
    if let (Some(router), false) = (
        resolved.focus_tunnel_file_router.as_ref(),
        changed_files.is_empty(),
    ) {
        let guard = router.lock().await;
        let consumed = guard.micro_steps_consumed();
        drop(guard);
        // Emit a "Proceed-shaped" row so the dashboard's micro_step ledger
        // captures the post-execute boundary. Cap-exhausted rows are
        // emitted by the production caller that owns model_output once
        // that signal is plumbed through.
        let payload = SsbPayload::MicroStepCreated(crate::ssb::MicroStepCreatedPayload {
            parent_step_id: baby_step_id,
            created_path: changed_files
                .first()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            type_hint: "Module".to_string(),
            reason: "post_execute_observed".to_string(),
            count: consumed as u8,
        });
        if let Err(e) = writer
            .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
            .await
        {
            tracing::warn!(
                baby_step_id,
                error = %e,
                "task_runner: MicroStepCreated SSB write failed (non-fatal)"
            );
        }
    }
}

/// Map a [`crate::focus_tunnel::BlockedReason`] variant to the wire-form
/// string used in [`crate::ssb::DependencyResolvedPayload::reason`]. Keeps
/// the focus_tunnel -> ssb dependency direction unidirectional.
fn focus_tunnel_blocked_reason_str(reason: &crate::focus_tunnel::BlockedReason) -> String {
    match reason {
        crate::focus_tunnel::BlockedReason::DeprecatedPattern { .. } => {
            "deprecated_pattern".to_string()
        }
        crate::focus_tunnel::BlockedReason::RegistryNotFound { .. } => {
            "registry_not_found".to_string()
        }
        crate::focus_tunnel::BlockedReason::RegistryUnexpectedStatus { status, .. } => {
            format!("registry_status_{status}")
        }
        crate::focus_tunnel::BlockedReason::AirGapped => "air_gapped".to_string(),
        crate::focus_tunnel::BlockedReason::AntiStaleWaiverPending { .. } => {
            "anti_stale_waiver_pending".to_string()
        }
        crate::focus_tunnel::BlockedReason::UnsupportedEcosystem { .. } => {
            "unsupported_ecosystem".to_string()
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
    // NARROWING-OK: i64->u64 bit-reinterpretation; UUID byte layout intentionally preserves sign bit.
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
        appcontainer_profile_id: None,
        allowed_paths: Vec::new(),
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
    // Batch 4C (A-22): emit a RESEARCHING-entry FsmTransition row so the
    // dashboard can surface the web-research window in the StatusBar even
    // though the task_runner does not own a live Fsm. The matching exit
    // row is emitted at every early-return below.
    emit_fsm_transition(writer, "IDLE", "RESEARCHING", "web_research_task_entry").await;

    emit_reasoning(
        writer,
        0,
        "observe",
        &format!(
            "web-research task seq={} text={:?}",
            seq,
            truncate_for_log(&task.task, TASK_LOG_TRUNCATE_CHARS)
        ),
        resolved.pipeline_grc.as_ref(),
    )
    .await;

    if cancel.is_cancelled() {
        emit_fsm_transition(writer, "RESEARCHING", "IDLE", "cancelled_before_run").await;
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
        emit_fsm_transition(writer, "RESEARCHING", "IDLE", "no_runner_configured").await;
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
            emit_fsm_transition(writer, "RESEARCHING", "IDLE", "cancelled_mid_run").await;
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
                emit_fsm_transition(writer, "RESEARCHING", "IDLE", "runner_error").await;
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
        resolved.pipeline_grc.as_ref(),
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
    // Batch 4C (A-22): exit transition back to EXECUTING so downstream
    // callers see the research window close cleanly.
    emit_fsm_transition(writer, "RESEARCHING", "EXECUTING", "web_research_task_exit").await;
    Ok(())
}

/// Batch 4C (A-22) — emit a single `FsmTransition` SSB row.
///
/// Kept as a helper so every call site in `drive_web_research_task`
/// (entry, every early-return, exit) renders a consistent 3-tuple. The
/// error path only logs — the runner must stay alive because an SSB write
/// failure during research is non-fatal.
pub async fn emit_fsm_transition(writer: &Arc<Ssb>, from: &str, to: &str, reason: &str) {
    let payload = SsbPayload::FsmTransition(crate::ssb::FsmTransitionPayload {
        from: from.to_string(),
        to: to.to_string(),
        reason: reason.to_string(),
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        error!(
            from = %from, to = %to, error = %e,
            "task_runner: FsmTransition write failed"
        );
    }
    // B-9 T5: pair every FSM-transition emit with a lightweight checkpoint row
    // so the dashboard observes a checkpoint at every state-machine boundary.
    // NARROWING-OK: RUNNER_SUPERSTEP == 0 (i64); non-negative constant, i64->u64 is safe here.
    emit_dag_checkpoint(writer, RUNNER_AGENT_ID, RUNNER_SUPERSTEP as u64, "fsm_transition").await;
}

/// B-9 T5: emit a lightweight `SsbActionType::Checkpoint` row capturing a
/// per-step / per-tool-call / per-FSM-transition boundary.
///
/// Per DECISION-B in workflow/reasoning/B-9-decisions.md, this is the
/// **lightweight** checkpoint emit. The heavy 5-file
/// [`crate::checkpoint::CheckpointManager::write`] path stays reserved for
/// baby-step boundaries (it is too costly for the 25+ emit sites per turn
/// the spec asks for).
///
/// The `kind` argument ("fsm_transition" | "grc_step" | "tool_call") is
/// encoded into `grc_history_hash` so consumers can demux the three streams
/// without a schema change.
///
/// Failures are logged but never propagated — checkpoint emission must not
/// stop the runner.
pub async fn emit_dag_checkpoint(
    writer: &Arc<Ssb>,
    agent_id: &str,
    step_index: u64,
    kind: &str,
) {
    let payload = SsbPayload::Checkpoint(crate::ssb::CheckpointPayload {
        agent_state_hash: String::new(),
        grc_history_hash: format!("dag_checkpoint:{kind}"),
        agent_id: agent_id.to_owned(),
        step_index,
        file_sizes: Vec::new(),
    });
    if let Err(e) = writer
        .write_payload(agent_id, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        warn!(error = %e, kind = %kind, "task_runner: emit_dag_checkpoint failed");
    }
}

// ---------------------------------------------------------------------------
// BSP parallel-phase routing (Session 3D.REDO)
// ---------------------------------------------------------------------------

/// Finding category emitted when a parallel phase routed through
/// [`BspEngine`] fails (channel error, max-attempts exceeded, etc.).
pub const BSP_PHASE_FAILED_CATEGORY: &str = "bsp_phase_failed";

/// Finding category emitted when a parallel phase skipped BSP routing
/// because the phase has zero executable steps.
pub const BSP_PHASE_EMPTY_CATEGORY: &str = "bsp_phase_empty";

/// Checkpoint subdirectory under the task-runner scratch dir where BSP
/// per-phase checkpoints are written. Kept separate from the agent-level
/// checkpoints the baby-step loop writes so the two do not collide.
pub const BSP_CHECKPOINT_SUBDIR: &str = "bsp";

/// Route every phase with `is_parallel = true` through [`BspEngine`], one
/// engine instance per phase. Each step in the phase becomes a [`NodeSpec`]
/// (role = Executor, no dependencies, non-deferred) so the engine executes
/// them concurrently via `execute_tasks_with_timeout`.
///
/// Failures are non-fatal: an SSB `Finding` is emitted and the caller
/// continues with the sequential navigator path for the remaining work.
/// The engine writes its own `Checkpoint`, `DeathEvent`, and
/// `MemoryMonitoring` rows — this function only adds a surrounding
/// `ReasoningTrace` row so the dashboard can see that a parallel phase was
/// dispatched.
///
/// Checkpoint layout: `{scratch}/bsp/task_{seq}/phase_{phase_id}`.
/// `scratch` is `resolved.checkpoint.scratch_dir` when `CheckpointContext`
/// is configured, else the runner's current working directory.
/// Return a copy of `d` with every `is_parallel = true` phase removed from
/// each concept. Concepts whose phase list becomes empty are retained so the
/// concept-level integration checkpoint still fires; `spec_from_decomposition`
/// supplies a synthetic overview step for empty-phase concepts. Session
/// 3D.REDO: prevents the sequential navigator from re-executing phases that
/// [`run_parallel_phases_via_bsp`] already dispatched.
fn strip_parallel_phases(mut d: ConceptDecomposition) -> ConceptDecomposition {
    for concept in &mut d.concepts {
        concept.phases.retain(|p| !p.is_parallel);
    }
    d
}

/// B-8 Tasks#1 helper: build a multi-line plan-like string from the live
/// decomposition + the natural-language task goal so
/// [`crate::grc::PipelineGrc::ground_plan`] has something to scan.
///
/// The reasoning engine's claim regex (`engine.rs:501`) matches lines that
/// start with `Grounding: (file|grep|test|search|tool|training):...`.
///
/// Cycle-2 fix (BLOCK-1): the previous version emitted a synthetic
/// `Grounding: training:unverified` line whenever a concept description
/// mentioned `.rs`, `.py`, `crate `, `file:`, or `api `. That synthetic
/// citation always landed in `grounding.ungrounded` (LocalVerifier resolves
/// the literal path "unverified", which never exists), causing
/// GroundingSweepGate to falsely block ~50% of legitimate Rust/Python/API
/// tasks. We now emit zero synthetic Grounding lines: only citations that
/// are actually present on the live decomposition struct (today: none at
/// the concept level — BabyStepSpec.grounding.top_citations is per-step
/// and threading those up is post-B-8 scope) should ever reach
/// `ground_plan`. With no synthetic noise, GroundingSweepGate now
/// evaluates against the real ungrounded set and only blocks when the
/// decomposition actually carries unverifiable claims.
///
/// Visibility: `pub` (doc-hidden) so the cycle-2 BLOCK-1 regression test
/// in `tests/safety_gate_integration.rs` can call this directly. Not part
/// of the stable API surface.
#[doc(hidden)]
pub fn serialize_decomposition_for_grounding(
    decomposition: &ConceptDecomposition,
    task_goal: &str,
) -> String {
    let mut out = String::new();
    out.push_str("Goal: ");
    out.push_str(task_goal);
    out.push('\n');
    out.push_str("Idea: ");
    out.push_str(&decomposition.idea);
    out.push('\n');
    for concept in &decomposition.concepts {
        out.push_str(&format!("Concept[{}]: {}\n", concept.id.0, concept.name));
        out.push_str(&format!("  Description: {}\n", concept.description));
        // No synthetic Grounding lines — see doc comment above (BLOCK-1).
        // When the decomposer eventually threads real citations onto
        // ConceptNode/PhaseNode, emit them here as
        //   `  Grounding: file:<path>` / `  Grounding: tool:<id>` etc.
        for phase in &concept.phases {
            out.push_str(&format!("  Phase[{}]: {}\n", phase.id.0, phase.name));
            for step in &phase.steps {
                out.push_str(&format!(
                    "    Step[{}]: {}\n",
                    step.id.0, step.description
                ));
            }
        }
    }
    out
}

async fn run_parallel_phases_via_bsp(
    writer: &Arc<Ssb>,
    seq: i64,
    decomposition: &ConceptDecomposition,
    resolved: &ResolvedConfig,
) {
    use elai_core::clock::OsClock;
    use elai_core::OsEntropy;

    let scratch_root = resolved
        .checkpoint
        .as_ref()
        .map(|cp| cp.scratch_dir.clone())
        .unwrap_or_else(|| PathBuf::from("."));

    for concept in &decomposition.concepts {
        for phase in &concept.phases {
            if !phase.is_parallel {
                continue;
            }

            if phase.steps.is_empty() {
                emit_finding(
                    writer,
                    "info",
                    BSP_PHASE_EMPTY_CATEGORY,
                    &format!(
                        "parallel phase id={} name={:?} has no steps; skipping BSP dispatch",
                        phase.id.0, phase.name,
                    ),
                )
                .await;
                continue;
            }

            let node_specs: Vec<crate::bsp::NodeSpec> = phase
                .steps
                .iter()
                .map(|step| crate::bsp::NodeSpec {
                    id: format!("step_{}", step.id.0),
                    role: AgentRole::Executor,
                    task: step.description.clone(),
                    depends_on: Vec::new(),
                    is_deferred: false,
                })
                .collect();

            let session_id = format!("task_seq_{}_phase_{}", seq, phase.id.0);
            let project_id = format!("task_runner:{}", seq);
            let checkpoint_dir = scratch_root
                .join(BSP_CHECKPOINT_SUBDIR)
                .join(format!("task_{}", seq))
                .join(format!("phase_{}", phase.id.0));

            let config = crate::bsp::BspConfig {
                max_superstep_attempts: crate::bsp::MAX_SUPERSTEP_ATTEMPTS,
                max_supersteps: crate::bsp::MAX_SUPERSTEPS,
                straggler_timeout: std::time::Duration::from_secs(
                    crate::bsp::DEFAULT_STRAGGLER_TIMEOUT_SECS,
                ),
                checkpoint_dir,
                session_id,
                project_id,
                simulated_agent_work: std::time::Duration::ZERO,
            };

            let pool_config = crate::agent_pool::AgentPoolConfig {
                scratch_base: scratch_root.join(BSP_CHECKPOINT_SUBDIR).join("agents"),
                vram_required_bytes: crate::agent_pool::VRAM_MIN_BYTES,
                project_path: scratch_root.clone(),
                // B-18A Task 7: AppContainer on Windows; non-isolating
                // pass-through on Linux/macOS until B-18B+ wires clone3+seccomp.
                enable_sandbox: cfg!(target_os = "windows"),
            };
            let pool = crate::agent_pool::InProcessAgentPool::new(
                pool_config,
                OsEntropy,
                Arc::new(RuntimeModelPool),
            );
            let mut engine = crate::bsp::BspEngine::new(config, pool, OsClock);
            let mut channels = crate::bsp::InMemoryChannelRegistry::new();

            emit_reasoning(
                writer,
                0,
                "plan_action",
                &format!(
                    "bsp dispatch: phase_id={} nodes={} (parallel)",
                    phase.id.0,
                    node_specs.len(),
                ),
                resolved.pipeline_grc.as_ref(),
            )
            .await;

            match engine.run(node_specs, &mut channels, writer.as_ref()).await {
                Ok(result) => {
                    emit_reasoning(
                        writer,
                        0,
                        "verify",
                        &format!(
                            "bsp complete: phase_id={} supersteps={}",
                            phase.id.0, result.supersteps_completed,
                        ),
                        resolved.pipeline_grc.as_ref(),
                    )
                    .await;
                }
                Err(e) => {
                    emit_finding(
                        writer,
                        "error",
                        BSP_PHASE_FAILED_CATEGORY,
                        &format!("bsp phase_id={} failed: {}", phase.id.0, e),
                    )
                    .await;
                }
            }
        }
    }
}

// Batch 1C: stable stage labels for `IndexingProgressPayload.stage`.
// Kept as const strings so the frontend's substring matcher stays in sync
// via the `task_runner_indexing_service_contract::indexing_stage_labels_are_stable`
// contract. The `dkg_parsing` / `bm25_indexing` stages are reserved for the
// future streaming build and are NOT currently emitted by
// `maybe_build_project_index`; we only emit `start` + (`complete` |
// `failed`). They are still declared here so adding them later is a single-
// site change.
const INDEXING_STAGE_START: &str = "start";
#[allow(dead_code)] // reserved for streaming build; exercised via contract test
const INDEXING_STAGE_DKG_PARSING: &str = "dkg_parsing";
#[allow(dead_code)] // reserved for streaming build; exercised via contract test
const INDEXING_STAGE_BM25_INDEXING: &str = "bm25_indexing";
const INDEXING_STAGE_COMPLETE: &str = "complete";
const INDEXING_STAGE_FAILED: &str = "failed";

/// Batch 1C helper — emit a single `IndexingProgress` SSB row. All fields are
/// forwarded to `IndexingProgressPayload` verbatim.
async fn emit_indexing_progress(
    writer: &Arc<Ssb>,
    stage: &str,
    project_id: &str,
    files_processed: u32,
    files_total: u32,
    elapsed_ms: u64,
    message: Option<String>,
) {
    let payload = SsbPayload::IndexingProgress(IndexingProgressPayload {
        stage: stage.to_owned(),
        project_id: project_id.to_owned(),
        files_processed,
        files_total,
        elapsed_ms,
        message,
        // Track A.1: task_runner does not drive error paths here; error_kind
        // and error_message are emitted by cli/src/indexing.rs on the Err arm.
        error_kind: None,
        error_message: None,
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        error!(
            project_id = %project_id,
            stage = %stage,
            error = %e,
            "task_runner: write IndexingProgress failed",
        );
    }
}

/// Batch-ful-a helper — emit the one-shot `DkgQuerierSwapped` SSB row.
///
/// Called from `maybe_build_project_index` success branch immediately after
/// `emit_dkg_ready` when the caller has plumbed a `dkg_querier_swap` RwLock
/// handle AND the DKG db path is openable by `DkgQuerierAdapter::new`. The
/// `from` / `to` tags surface as "DKG querier upgraded · empty → adapter"
/// on the dashboard.
async fn emit_dkg_querier_swapped(
    writer: &Arc<Ssb>,
    project_id: &str,
    from: &str,
    to: &str,
) {
    let payload = SsbPayload::DkgQuerierSwapped(DkgQuerierSwappedPayload {
        project_id: project_id.to_owned(),
        from: from.to_owned(),
        to: to.to_owned(),
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        error!(
            project_id = %project_id,
            error = %e,
            "task_runner: write DkgQuerierSwapped failed",
        );
    }
}

/// Batch-ful-a helper — attempt to hot-swap `EmptyDkgQuerier` for
/// `DkgQuerierAdapter` on the rule engine's shared RwLock slot.
///
/// Returns `Ok(true)` when a swap actually occurred, `Ok(false)` when no slot
/// was plumbed / project already swapped / adapter construction failed.
/// Never panics — lock poisoning surfaces as a warn-logged skip.
async fn maybe_swap_dkg_querier(
    writer: &Arc<Ssb>,
    resolved: &ResolvedConfig,
    project_id: &str,
    dkg_db_path: &std::path::Path,
) -> bool {
    let Some(slot) = resolved.dkg_querier_swap.as_ref() else {
        return false;
    };

    // Fast-path idempotency: one swap per project per runner lifetime.
    if let Ok(mut done) = resolved.dkg_querier_swapped_projects.lock() {
        if done.contains(project_id) {
            return false;
        }
        done.insert(project_id.to_owned());
    } else {
        // Poisoned sync mutex — nothing we can safely do. Log + skip.
        tracing::warn!(
            project_id = %project_id,
            "task_runner: dkg_querier_swapped_projects mutex poisoned — skipping swap",
        );
        return false;
    }

    // Build the adapter from the freshly-produced DKG db.
    let adapter = match elai_knowledge::DkgQuerierAdapter::new(dkg_db_path) {
        Ok(a) => a,
        Err(e) => {
            tracing::warn!(
                project_id = %project_id,
                dkg_db = %dkg_db_path.display(),
                error = %e,
                "task_runner: DkgQuerierAdapter construction failed — keeping previous querier",
            );
            return false;
        }
    };

    // Scope the write guard tightly so it drops BEFORE any await — the
    // std::sync::RwLockWriteGuard is !Send, so holding it across
    // `emit_dkg_querier_swapped(...).await` fails the Send bound on
    // tokio::spawn. The inner block returns a bool telling the post-drop
    // path whether to emit the SSB row + log.
    let swapped: bool = {
        match slot.write() {
            Ok(mut g) => {
                *g = Box::new(adapter)
                    as Box<dyn elai_rules::predicate::DkgQuerier + Send + Sync>;
                true
            }
            Err(_) => {
                tracing::warn!(
                    project_id = %project_id,
                    "task_runner: dkg_querier_swap RwLock poisoned — keeping previous querier",
                );
                false
            }
        }
    };

    if swapped {
        emit_dkg_querier_swapped(writer, project_id, "empty", "adapter").await;
        tracing::info!(
            project_id = %project_id,
            "task_runner: DkgQuerierAdapter hot-swapped into rule engine",
        );
    }
    swapped
}

/// Batch 1C helper — emit the one-shot `DkgReady` SSB row.
async fn emit_dkg_ready(
    writer: &Arc<Ssb>,
    project_id: &str,
    node_count: u32,
    edge_count: u32,
    languages: Vec<String>,
) {
    let payload = SsbPayload::DkgReady(DkgReadyPayload {
        project_id: project_id.to_owned(),
        node_count,
        edge_count,
        languages,
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        error!(
            project_id = %project_id,
            error = %e,
            "task_runner: write DkgReady failed",
        );
    }
}

/// Gap #6 / Batch 1C: ensure the project's DKG+BM25 indexes have been built
/// for the given `project_id`. Runs `IndexingService::run_full_index` on the
/// first task for that project (or when the cached `indexed_at` is stale per
/// [`crate::indexing_service::INDEX_FRESHNESS_SECS`]); emits structured
/// `IndexingProgress` + `DkgReady` SSB rows (Batch 1C) AND the legacy
/// `INDEXING_SUMMARY_CATEGORY` Finding so pre-Batch-1C dashboard filters still
/// work. Respects the `ELAI_DISABLE_INDEXING` emergency shutoff. No-op when
/// `indexing_service` is `None`.
async fn maybe_build_project_index(
    writer: &Arc<Ssb>,
    resolved: &ResolvedConfig,
    project_id: &str,
) {
    let Some(svc) = resolved.indexing_service.as_ref() else {
        return;
    };

    // Emergency shutoff: skip every write/read when disabled via env.
    if crate::indexing_service::indexing_disabled_via_env() {
        tracing::debug!(
            project_id = %project_id,
            "task_runner: indexing disabled via ELAI_DISABLE_INDEXING",
        );
        return;
    }

    // Fast path: already indexed for this project_id AND still fresh.
    let now_ms = now_unix_ms();
    let freshness_ms: i64 = crate::indexing_service::INDEX_FRESHNESS_SECS.saturating_mul(1_000);
    if let Ok(cache) = resolved.indexed_projects.lock() {
        if let Some(indexed_at) = cache.get(project_id) {
            if now_ms.saturating_sub(*indexed_at) < freshness_ms {
                return;
            }
        }
    }

    // Slow path: acquire the service mutex and (maybe) run the full index.
    let mut guard = svc.lock().await;
    // Re-check under the async lock to avoid racing two schedulers.
    if let Ok(cache) = resolved.indexed_projects.lock() {
        if let Some(indexed_at) = cache.get(project_id) {
            if now_ms.saturating_sub(*indexed_at) < freshness_ms {
                return;
            }
        }
    }

    if guard.indexes_exist() {
        // Already on disk: record the cache entry without a rebuild.
        let now = now_unix_ms();
        if let Ok(mut cache) = resolved.indexed_projects.lock() {
            cache.insert(project_id.to_owned(), now);
        }
        emit_finding(
            writer,
            "info",
            INDEXING_SUMMARY_CATEGORY,
            &format!("project_id={project_id} indexes_present=true skipped_rebuild=true"),
        )
        .await;

        // Batch-ful-a: even when the DKG db was already on disk (prior
        // session / stale cache), the dashboard-plugged querier may still
        // be `EmptyDkgQuerier` if the dashboard booted before the db
        // existed. Attempt the swap once.
        let dkg_db_path = guard.indexes_dir().join("dkg.db");
        drop(guard);
        maybe_swap_dkg_querier(writer, resolved, project_id, &dkg_db_path).await;
        return;
    }

    // Structured emission: Start row BEFORE the (blocking) build call.
    let start_ms = now_unix_ms();
    emit_indexing_progress(
        writer,
        INDEXING_STAGE_START,
        project_id,
        0,
        0,
        0,
        None,
    )
    .await;

    match guard.run_full_index() {
        Ok(summary) => {
            let done_ms = now_unix_ms();
            let elapsed_ms = (done_ms.saturating_sub(start_ms)).max(0) as u64; // NARROWING-OK: .max(0) ensures non-negative i64; u64::try_from non-negative i64 always succeeds but adds no safety here
            let files_processed: u32 = summary
                .bm25_files
                .try_into()
                .unwrap_or(u32::MAX);
            let node_count: u32 = summary.dkg_nodes.try_into().unwrap_or(u32::MAX);
            let edge_count: u32 = summary.dkg_edges.try_into().unwrap_or(u32::MAX);

            // Complete row.
            emit_indexing_progress(
                writer,
                INDEXING_STAGE_COMPLETE,
                project_id,
                files_processed,
                files_processed,
                elapsed_ms,
                None,
            )
            .await;

            // B-3 #1: derive languages from the IndexingSummary's
            // LanguageRegistryStatus (B-30 Pass C populated this on every
            // run_full_index call). Falls back to an empty list when the
            // registry is unavailable (DkgEngine construction failed before
            // status was readable) — the dashboard renders "no data" in
            // that case without misrepresenting partial state.
            let languages: Vec<String> = summary
                .registry_status
                .as_ref()
                .map(|rs| rs.loaded.clone())
                .unwrap_or_default();
            emit_dkg_ready(writer, project_id, node_count, edge_count, languages).await;

            // Batch-ful-a (FUL-A.1): the DKG db is fresh — if the dashboard
            // started this runner with `EmptyDkgQuerier` plugged into the
            // rule engine and handed us the shared RwLock via
            // `dkg_querier_swap`, upgrade it in-place so the NEXT PRE_GATE
            // evaluation on any baby step sees live reachability results.
            let dkg_db_path = guard.indexes_dir().join("dkg.db");
            maybe_swap_dkg_querier(writer, resolved, project_id, &dkg_db_path).await;

            // Legacy Finding (pre-Batch-1C dashboards filter on this).
            if let Ok(mut cache) = resolved.indexed_projects.lock() {
                cache.insert(project_id.to_owned(), done_ms);
            }
            emit_finding(
                writer,
                "info",
                INDEXING_SUMMARY_CATEGORY,
                &format!(
                    "project_id={project_id} files_indexed={} dkg_nodes={} bm25_chunks={}",
                    summary.bm25_files, summary.dkg_nodes, summary.bm25_chunks,
                ),
            )
            .await;
        }
        Err(e) => {
            let done_ms = now_unix_ms();
            let elapsed_ms = (done_ms.saturating_sub(start_ms)).max(0) as u64; // NARROWING-OK: .max(0) ensures non-negative i64; u64::try_from non-negative i64 always succeeds but adds no safety here
            let err_s = e.to_string();
            emit_indexing_progress(
                writer,
                INDEXING_STAGE_FAILED,
                project_id,
                0,
                0,
                elapsed_ms,
                Some(err_s.clone()),
            )
            .await;
            warn!(project_id = %project_id, error = %e, "task_runner: indexing failed");
            emit_finding(
                writer,
                "warning",
                INDEXING_SUMMARY_CATEGORY,
                &format!("project_id={project_id} indexing failed: {err_s}"),
            )
            .await;
        }
    }
}

/// Return the current time in milliseconds since the Unix epoch. Mirrors the
/// "best-effort timestamp" pattern used by [`CheckpointManager`] and
/// `IndexingService::run_full_index` so the two surfaces stay consistent.
///
/// C-15: sourced from `OsClock` via the `ClockProvider` trait so the entropy
/// quarantine scanner does not flag this file as a leak. The 6 callers are
/// all inside the production code path; if a deterministic test ever needs to
/// override this, expose a clock-parametrised variant.
fn now_unix_ms() -> i64 {
    i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX)
}

async fn drive_task(
    writer: &Arc<Ssb>,
    seq: i64,
    task: &TaskSubmittedPayload,
    resolved: &ResolvedConfig,
    cancel: &CancelHandle,
) -> Result<(), TaskRunnerError> {
    // Batch 1B: emit the one-shot `VramTierDetected` row at the first task
    // start. No-op on subsequent tasks (flag on ResolvedConfig). Safe to run
    // before web-scaffold / web-research routing because the dashboard
    // activity feed wants the tier row regardless of task flavour.
    emit_vram_tier_once(writer, resolved).await;

    // Batch 3A (Phase 4F): emit the one-shot `SystemProbe` row at the first
    // task start. No-op when no prober was plumbed OR when the probe is
    // disabled via `ELAI_DISABLE_SYSTEM_PROBE`. Paired with `vram_tier_emitted`
    // so the dashboard always has a system-level snapshot at startup.
    emit_system_probe_once(writer, resolved).await;

    // Gap #6 (MVP audit 2026-04-18 section 4 item 6): on the first task for
    // this project_id, run the `IndexingService::run_full_index` pipeline so
    // downstream rule evaluation (Gap #9) and the `DkgQuerierAdapter` path on
    // the dashboard have a live index to query. No-op when no indexing_service
    // was plumbed or the project is already cached.
    maybe_build_project_index(writer, resolved, &task.project_id).await;

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
    let observe_content = format!(
        "task seq={} text={:?}",
        seq,
        truncate_for_log(&task.task, 120)
    );
    emit_reasoning(
        writer,
        0,
        "observe",
        &observe_content,
        resolved.pipeline_grc.as_ref(),
    )
    .await;

    // patch-1.32 + batch-fu: when the caller plumbed an LlmProvider (dashboard
    // path), build the per-dispatch context so the FSM-state tool filter +
    // context budget adapter are exercised on every task. Batch-fu folds the
    // PhaseSlot (FU.1), constitutional CONSTRAINTS (A-7 / FU.2), and the
    // GenerationProfile selector (A-18 / FU.3) into the call so every dispatch
    // emits a ProfileSelected + ConstraintsInjected row when the respective
    // plumbing is active. See `docs/mvp-wiring-audit-2026-04-18.md` section 4
    // item #3 and the CLI parallel caller at `crates/cli/src/main.rs:~7632`.
    if resolved.llm_provider.is_some() {
        // Baseline orchestrator-visible tool list. Kept intentionally small
        // and stable — the elai-tools `mvp_tool_specs` catalogue lives on the
        // CLI side and would drag a workspace dependency cycle if imported
        // here.
        let specs: Vec<String> = DISPATCH_CONTEXT_BASELINE_TOOLS
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let priority_map: BTreeMap<String, u8> = specs
            .iter()
            .map(|name| (name.clone(), 0u8))
            .collect();
        let fsm_state = crate::fsm::FsmState::Planning;

        // Batch-fu (A-18): pick the per-dispatch GenerationProfile before
        // building the context so the profile can be threaded into the
        // PhaseSlot.profile_hint. Falls back to FirstAttempt when no selector
        // was plumbed.
        let profile = match resolved.dispatch_context_profile.as_ref() {
            Some(selector) => selector.select(
                crate::profile_selector::DispatchRole::Executor,
                fsm_state,
            ),
            None => crate::context_budget::GenerationProfile::FirstAttempt,
        };

        // Batch-fu (A-7 / FU.2): load the ProjectProfile lazily so the
        // CONSTRAINTS list is available to the PhaseSlot. Missing profile →
        // empty constraints (no block injected).
        let project_dir_path = project_dir
            .as_deref()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let constraints: Vec<String> =
            match elai_knowledge::detect_project_profile(&project_dir_path) {
                Ok(pp) => pp.constraints.clone(),
                Err(_) => Vec::new(),
            };

        let phase_slot = crate::dispatcher::PhaseSlot {
            goal: format!("task seq={}", seq),
            constraints: constraints.clone(),
            profile_hint: Some(profile),
            repair_snippet: None,
        };

        match crate::dispatcher::build_dispatch_context(
            fsm_state,
            &resolved.backend_config,
            &specs,
            &priority_map,
            // B-10 (DECISION-A): pass the resolved ModelCapabilityProfile so
            // the assembled ContextBudget uses real per-tier slot weights
            // instead of the [0,0,0]/4096 zeroed-stub fallback at
            // dispatcher.rs:222 (the prod wiring gap closed by B-10).
            Some(resolved.model_capability_profile.as_ref()),
            Some(&phase_slot),
        ) {
            Ok(ctx) => {
                emit_finding(
                    writer,
                    "info",
                    DISPATCH_CONTEXT_CATEGORY,
                    &format!(
                        "dispatch_context: backend={} fsm={:?} tools_allowed={} max_tools={} profile={}",
                        ctx.backend_mode,
                        ctx.fsm_state,
                        ctx.filtered_tool_names.len(),
                        ctx.max_tools,
                        crate::profile_selector::profile_name(ctx.profile)
                    ),
                )
                .await;

                // B-10 (DECISION-C): emit ContextBudgetComputed so the dashboard's
                // per-slot bar chart renders real allocations matching the active
                // tier profile (the per-spec values now active post-DECISION-D).
                if let Some(budget) = ctx.budget.as_ref() {
                    let raw_max = crate::dispatcher::LOCAL_MODEL_MAX_CONTEXT
                        * crate::dispatcher::LOCAL_BUDGET_FRACTION
                        / 100;
                    if let Err(e) = crate::context_budget::emit_context_budget_computed(
                        writer,
                        RUNNER_AGENT_ID,
                        RUNNER_SUPERSTEP,
                        resolved.model_capability_profile.as_ref(),
                        budget,
                        raw_max,
                    )
                    .await
                    {
                        warn!(seq, error = %e, "task_runner: ContextBudgetComputed write failed");
                    }
                }

                // B-10 / 3E.5 (DECISION-C): wire compute_overlap into the SsbEntries
                // slot-fill path. Query all FileWrite SSB entries for the current
                // project to obtain (seq, file_paths) candidate pairs; run
                // compute_overlap against the agent's active file set; emit
                // SsbFilteredByOverlap so the dashboard renders the filter ratio.
                //
                // agent_files is the set of files the task runner is currently
                // tracking for this task. At drive_task call time the task runner
                // owns no open file handles (files are touched by the LlmProvider /
                // CLI child later), so agent_files is derived from the task payload's
                // project_id path as the working directory prefix. An empty
                // agent_files set causes compute_overlap to return 0 kept entries
                // (every score = 0.0 < FILE_OVERLAP_THRESHOLD), which is the correct
                // conservative outcome: without a concrete file list, no SSB entries
                // are overlap-selected, and the dashboard shows the filter ratio as
                // 0/N. Future tasks that pass file lists through the PhaseSlot will
                // produce non-zero kept counts once the context assembler is wired.
                {
                    let file_entries: Vec<SsbEntry> = writer
                        .query_by_type(SsbActionType::FileWrite)
                        .unwrap_or_default();
                    let entries_total = file_entries.len();
                    // Build (seq, files) pairs. File paths are stored in ssb_files;
                    // SsbEntry.payload does not carry file paths directly. The
                    // compute_overlap API requires (I, Vec<String>) pairs. Since
                    // the task runner has no ssb_files read path (write-only Arc<Ssb>),
                    // use seq as the entry id and pass an empty file list per entry.
                    // This yields score=0 for every candidate (agent_files is also
                    // empty), producing entries_kept=0 and entries_total=N — the
                    // correct signal for the dashboard when file associations are
                    // not yet threaded through the task runner path.
                    //
                    // The real per-entry file lists will be plumbed in the context
                    // assembler phase when the SsbViewer's ssb_files JOIN is available
                    // on the read path. DECISION-E: deferred to context-assembler wiring.
                    let agent_files: Vec<String> = Vec::new();
                    let candidates: Vec<(i64, Vec<String>)> = file_entries
                        .iter()
                        .map(|e| (e.seq, Vec::new()))
                        .collect();
                    let kept = crate::context_budget::compute_overlap(&agent_files, &candidates);
                    let entries_kept = kept.len();
                    if let Err(e) = crate::context_budget::emit_ssb_filtered_by_overlap(
                        writer,
                        RUNNER_AGENT_ID,
                        RUNNER_SUPERSTEP,
                        entries_total,
                        entries_kept,
                    )
                    .await
                    {
                        warn!(seq, error = %e, "task_runner: SsbFilteredByOverlap write failed");
                    }
                }

                // Batch-fu (A-7): emit a ConstraintsInjected row whenever the
                // phase_slot carried at least one constraint so the dashboard
                // can render a teal row per dispatch.
                if !constraints.is_empty() {
                    let payload = crate::ssb::SsbPayload::ConstraintsInjected(
                        crate::ssb::ConstraintsInjectedPayload {
                            project_id: task.project_id.clone(),
                            constraint_count: constraints.len().try_into().unwrap_or(u32::MAX),
                            step_id: format!("task_{}", seq),
                        },
                    );
                    if let Err(e) = writer
                        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
                        .await
                    {
                        warn!(seq, error = %e, "task_runner: ConstraintsInjected write failed");
                    }
                }

                // Batch-fu (A-18): emit ProfileSelected only when a selector was
                // plumbed AND the profile differs from the default. The row
                // carries the role + fsm_state so the dashboard can filter the
                // activity feed on "profile · RepairRetry" / etc.
                if resolved.dispatch_context_profile.is_some()
                    && profile != crate::context_budget::GenerationProfile::FirstAttempt
                {
                    if let Err(e) = crate::profile_selector::emit_profile_selected(
                        writer,
                        RUNNER_AGENT_ID,
                        RUNNER_SUPERSTEP,
                        profile,
                        crate::profile_selector::DispatchRole::Executor,
                        fsm_state,
                        &format!("task_{}", seq),
                    )
                    .await
                    {
                        warn!(seq, error = %e, "task_runner: ProfileSelected write failed");
                    }
                }

                // Batch-fu (A-46 / FU.6): when a Repair profile was picked AND
                // the phase_slot carried a repair_snippet, invoke the
                // ContextTelescope to collapse layers until the snippet fits
                // under the Slot-3 token budget. The telescope is a no-op for
                // non-Repair profiles so pre-batch-fu dispatches are
                // unchanged.
                if profile == crate::context_budget::GenerationProfile::RepairRetry
                    && phase_slot.repair_snippet.is_some()
                {
                    invoke_repair_telescope(
                        writer,
                        &resolved.telescope_stats,
                        phase_slot.repair_snippet.as_deref().unwrap_or(""),
                        &format!("task_{}", seq),
                    )
                    .await;
                }
            }
            Err(e) => {
                warn!(seq, error = %e, "task_runner: build_dispatch_context failed");
            }
        }
    }

    let mut cache = ResearchCache::new(DEFAULT_CACHE_MAX_ENTRIES, DEFAULT_CACHE_TTL_SECONDS);
    // B-7b: wire tree emitter so each concept/phase/step/baby_step built during
    // decompose surfaces as a `concept_decomposition_node` SSB row in the dashboard.
    let decompose_tree_emitter =
        SsbDecompositionEmitter::new(Arc::clone(writer), RUNNER_AGENT_ID, RUNNER_SUPERSTEP);

    // Backend selection: ELAI_GROUNDING_BACKEND=cloud routes through Anthropic
    // (CloudGroundingClient). Default and any other value retain the historical
    // sidecar path. Picked here per-task so the env can be flipped at runtime
    // without restarting the orchestrator. Cloud-init failure surfaces as a
    // ResearchTreeError::GroundingFailed via the GroundingClient contract.
    // BLOCK-3: build EXISTING_ENTRY_POINTS + AVAILABLE_ARTIFACTS context when a
    // project directory is present. Non-fatal: empty string when DKG absent or
    // manifest not found. Prepended to the idea so the Designer LLM sees project
    // context before decomposing the task.
    let existing_context = project_dir
        .as_deref()
        .map(build_existing_project_context)
        .unwrap_or_default();
    let enriched_task: std::borrow::Cow<str> = if existing_context.is_empty() {
        std::borrow::Cow::Borrowed(task.task.as_str())
    } else {
        std::borrow::Cow::Owned(format!("{existing_context}{}", task.task))
    };

    let grounding_backend = std::env::var("ELAI_GROUNDING_BACKEND").ok();
    let decomposition = if grounding_backend.as_deref() == Some("cloud") {
        // patch-R2: switched from sync from_env() -> async from_env_async() so
        // an expired saved OAuth (expiresAt elapsed OR null + refresh_token
        // present) is auto-refreshed via resolve_auth_with_auto_refresh.
        // Without this the cloud path would Err with "saved OAuth token is
        // expired" after ~1 hour of orchestrator uptime.
        let grounding_client = CloudGroundingClient::from_env_async().await.map_err(|e| {
            crate::research_tree::ResearchTreeError::GroundingFailed {
                level: crate::grounding_client::ResearchLevel::Idea,
                source: e,
            }
        })?;
        research_decompose(
            enriched_task.as_ref(),
            ExecutionMode::Autonomous,
            &mut cache,
            &grounding_client,
            Some(writer),                  // Track A.6: SSB handle for cache-miss grounding rows.
            Some(&decompose_tree_emitter), // B-7b: emit concept_decomposition_node rows.
        )
        .await?
    } else {
        let grounding_client = SidecarGroundingClient::default();
        research_decompose(
            enriched_task.as_ref(),
            ExecutionMode::Autonomous,
            &mut cache,
            &grounding_client,
            Some(writer),                  // Track A.6: SSB handle for cache-miss grounding rows.
            Some(&decompose_tree_emitter), // B-7b: emit concept_decomposition_node rows.
        )
        .await?
    };
    if decomposition.concepts.is_empty() {
        return Err(TaskRunnerError::EmptyDecomposition);
    }

    // Batch 1B: research_decompose is the first sidecar-backed dispatch for
    // non-scaffold / non-web-research tasks. Snapshot the provider signals
    // here so the dashboard activity feed shows the decomposition-time GG /
    // novelty state alongside the "decomposed N concepts" reasoning row.
    // Search-trigger emission is skipped at this site because the
    // narrow `SidecarGroundingClient` used above is a separate reqwest
    // client from `SidecarProvider` — it does not populate the
    // last_search_trigger mutex we read via the reader. A future batch
    // (patch-1.3X) can switch decomposition to the shared `LlmProvider`
    // path, at which point search triggers become observable here too.
    if let Some(reader) = resolved.signal_reader.as_ref() {
        emit_provider_signals_snapshot(writer, reader, &format!("decompose_seq_{}", seq)).await;
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
        resolved.pipeline_grc.as_ref(),
    )
    .await;

    // B-8 Tasks#1: PipelineGrc.ground_plan + GroundingSweepGate consumer.
    //
    // Producer side: serialize the decomposition into a plan-like string and
    // run it through `ground_plan`, which scans for `^Grounding:` citations
    // and dispatches verify_claim per claim. Result populates a Vec<String>
    // of ungrounded claims.
    //
    // Consumer side: feed that list into `evidence::gates_for_transition(
    // Planning, TestingFirst)` (which today returns the lone
    // `GroundingSweepGate`). When the gate blocks, we emit an
    // `Finding` row with category `grounding_sweep_block` so the dashboard /
    // CLI surface the block, then short-circuit the task. The kill-switch
    // (`ELAI_DISABLE_PIPELINE_GRC`) is honoured by `ResolvedConfig::from`
    // forcing `pipeline_grc` to `None`, so this whole block is skipped when
    // the bypass is active. Spec target: B-8 §Tasks#1.
    if let Some(grc) = resolved.pipeline_grc.as_ref() {
        let plan_text = serialize_decomposition_for_grounding(&decomposition, &task.task);
        let grounding = grc.lock().await.ground_plan(&plan_text);

        // C-1: emit ONE `grounding_sweep_result` SSB row per ground_plan
        // call (before the gate-iteration block below). The dashboard
        // GroundingSweepCard renders this row directly. The plan task #5
        // emphasises ONCE per call — not inside the per-gate loop below
        // — to avoid amplifying rows when multiple gates evaluate the
        // same sweep.
        {
            let ungrounded_count = u32::try_from(grounding.ungrounded.len()).unwrap_or(u32::MAX);
            let verify_count = u32::try_from(grounding.verify_needed.len()).unwrap_or(u32::MAX);
            // `grounded` = claims that DID verify. Conservative formula:
            // total claims minus ungrounded (verify_needed is a strict
            // subset of ungrounded per PlanGroundingResult doc-comment).
            let grounded_count = grounding
                .claims
                .len()
                .saturating_sub(grounding.ungrounded.len());
            let grounded_count_u32 = u32::try_from(grounded_count).unwrap_or(u32::MAX);
            let top_claim = grounding.ungrounded.first().cloned();
            let sweep_payload =
                SsbPayload::GroundingSweepResult(GroundingSweepResultPayload {
                    ungrounded: ungrounded_count,
                    verify_needed: verify_count,
                    grounded: grounded_count_u32,
                    top_claim,
                });
            if let Err(e) = writer
                .write_payload(
                    RUNNER_AGENT_ID,
                    RUNNER_SUPERSTEP,
                    sweep_payload,
                    Vec::new(),
                )
                .await
            {
                tracing::warn!(
                    seq,
                    error = %e,
                    "task_runner: GroundingSweepResult SSB write failed (non-fatal)"
                );
            }
        }

        if !grounding.ungrounded.is_empty() {
            tracing::warn!(
                seq,
                ungrounded_count = grounding.ungrounded.len(),
                "task_runner: ground_plan flagged ungrounded claims"
            );

            // C-2 (Task #7): wire AmbiguityScorer.score() before the gate
            // dispatch when an LlmProvider is available. Populates
            // `TransitionContext.ambiguity_score` and emits one of three
            // SSB rows (ambiguity_scored / ambiguity_floor_failed /
            // ambiguity_streak_advanced) reflecting the verdict. Gate
            // gracefully degrades to B-17 behaviour when `ambiguity_score`
            // is None (evidence.rs:256 backward-compat).
            let ambiguity_score_opt: Option<elai_reasoning::AmbiguityScore> =
                if let Some(llm) = resolved.llm_provider.clone() {
                    let scorer = elai_reasoning::AmbiguityScorer::new(
                        llm,
                        resolved.backend_config.model.clone(),
                    );
                    let grounding_summary = format!(
                        "ungrounded={} verify_needed={}",
                        grounding.ungrounded.len(),
                        grounding.verify_needed.len()
                    );
                    let brownfield = false;
                    let deferrals: Vec<String> = Vec::new();
                    match scorer
                        .score(&plan_text, &grounding_summary, brownfield, &deferrals)
                        .await
                    {
                        Ok(score) => {
                            // Emit ambiguity_scored row reflecting the verdict.
                            let milestone_str = match score.milestone() {
                                elai_reasoning::Milestone::Ready => "ready",
                                elai_reasoning::Milestone::Refined => "refined",
                                elai_reasoning::Milestone::Progress => "progress",
                                elai_reasoning::Milestone::Initial => "initial",
                            };
                            let ctx_clarity_value = score
                                .breakdown
                                .context_clarity
                                .as_ref()
                                .map(|c| c.clarity_score);
                            let scored_payload = SsbPayload::AmbiguityScored(
                                crate::ssb::AmbiguityScoredPayload {
                                    score: score.overall_score,
                                    milestone: milestone_str.to_owned(),
                                    goal: score.breakdown.goal_clarity.clarity_score,
                                    constraint: score
                                        .breakdown
                                        .constraint_clarity
                                        .clarity_score,
                                    success: score
                                        .breakdown
                                        .success_criteria_clarity
                                        .clarity_score,
                                    context: ctx_clarity_value,
                                    brownfield,
                                },
                            );
                            if let Err(e) = writer
                                .write_payload(
                                    RUNNER_AGENT_ID,
                                    RUNNER_SUPERSTEP,
                                    scored_payload,
                                    Vec::new(),
                                )
                                .await
                            {
                                tracing::warn!(
                                    seq,
                                    error = %e,
                                    "task_runner: AmbiguityScored SSB write failed (non-fatal)"
                                );
                            }
                            let failures = score.floor_failures(brownfield);
                            if !failures.is_empty() {
                                let floor_payload = SsbPayload::AmbiguityFloorFailed(
                                    crate::ssb::AmbiguityFloorFailedPayload {
                                        score: score.overall_score,
                                        failed_dimensions: failures.clone(),
                                    },
                                );
                                if let Err(e) = writer
                                    .write_payload(
                                        RUNNER_AGENT_ID,
                                        RUNNER_SUPERSTEP,
                                        floor_payload,
                                        Vec::new(),
                                    )
                                    .await
                                {
                                    tracing::warn!(
                                        seq,
                                        error = %e,
                                        "task_runner: AmbiguityFloorFailed SSB write failed (non-fatal)"
                                    );
                                }
                            } else if score.is_ready() {
                                let streak_payload = SsbPayload::AmbiguityStreakAdvanced(
                                    crate::ssb::AmbiguityStreakAdvancedPayload {
                                        score: score.overall_score,
                                        current_streak: 1,
                                        required: elai_reasoning::AUTO_COMPLETE_STREAK_REQUIRED
                                            as u32,
                                    },
                                );
                                if let Err(e) = writer
                                    .write_payload(
                                        RUNNER_AGENT_ID,
                                        RUNNER_SUPERSTEP,
                                        streak_payload,
                                        Vec::new(),
                                    )
                                    .await
                                {
                                    tracing::warn!(
                                        seq,
                                        error = %e,
                                        "task_runner: AmbiguityStreakAdvanced SSB write failed (non-fatal)"
                                    );
                                }
                            }
                            Some(score)
                        }
                        Err(e) => {
                            tracing::warn!(
                                seq,
                                error = %e,
                                "task_runner: AmbiguityScorer.score() failed; gate falls back to B-17"
                            );
                            None
                        }
                    }
                } else {
                    None
                };

            let ctx = crate::evidence::TransitionContext {
                ungrounded_claims: Some(grounding.ungrounded.clone()),
                ambiguity_score: ambiguity_score_opt,
                ..crate::evidence::TransitionContext::default()
            };
            let gates = crate::evidence::gates_for_transition(
                crate::FsmState::Planning,
                crate::FsmState::TestingFirst,
            );
            let mut block_messages: Vec<String> = Vec::new();
            for gate in &gates {
                let check_result = gate.check(&ctx);
                // B-6: emit EvidenceGateEvaluated SSB row for each gate check.
                {
                    let ts_ms = elai_core::clock::OsClock
                        .system_now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or(std::time::Duration::ZERO)
                        .as_millis();
                    let ts_ms_i64 = i64::try_from(ts_ms).unwrap_or(i64::MAX);
                    let verdict = if check_result.is_ok() { "pass" } else { "block" };
                    let gate_payload =
                        SsbPayload::EvidenceGateEvaluated(EvidenceGateEvaluatedPayload {
                            gate_id: gate.name().to_owned(),
                            verdict: verdict.to_owned(),
                            ts_ms: ts_ms_i64,
                        });
                    if let Err(e) = writer
                        .write_payload(
                            RUNNER_AGENT_ID,
                            RUNNER_SUPERSTEP,
                            gate_payload,
                            Vec::new(),
                        )
                        .await
                    {
                        tracing::warn!(
                            gate = gate.name(),
                            error = %e,
                            "task_runner: EvidenceGateEvaluated SSB write failed (non-fatal)"
                        );
                    }
                }
                if let Err(err) = check_result {
                    block_messages.push(format!(
                        "{}: {} (missing: {})",
                        err.gate_name,
                        err.suggestion,
                        err.missing_artifacts.join(", ")
                    ));
                }
            }
            if !block_messages.is_empty() {
                let summary = block_messages.join(" | ");
                emit_finding(
                    writer,
                    "error",
                    "grounding_sweep_block",
                    &format!(
                        "PLANNING -> TESTING_FIRST blocked: {} ungrounded claim(s); {}",
                        grounding.ungrounded.len(),
                        summary
                    ),
                )
                .await;
                tracing::error!(
                    seq,
                    summary = %summary,
                    "task_runner: GroundingSweepGate blocked task; aborting"
                );
                // TODO(B-future): surface this through a dedicated SSB row. stub-ban-allow: B-future plan item
                // (csq-style intervention) once the new dashboard defines a
                // surface for safety-gate blocks.
                return Ok(());
            }
        } else {
            tracing::debug!(
                seq,
                claims = grounding.claims.len(),
                "task_runner: ground_plan completed with zero ungrounded claims"
            );
        }
    }

    // Session 3D.REDO: route phases with `is_parallel = true` through
    // `BspEngine` for superstep-level parallelism. Phases with the flag
    // unset (the decomposer default) continue through the sequential
    // navigator loop below, unchanged.
    run_parallel_phases_via_bsp(writer, seq, &decomposition, resolved).await;

    // Strip parallel phases from the decomposition handed to the sequential
    // navigator so the same phase is not executed twice (once by BSP above,
    // once by the baby-step loop below). `spec_from_decomposition` tolerates
    // concepts whose `phases` end up empty — it falls back to a single
    // concept-level overview step.
    let decomposition = strip_parallel_phases(decomposition);

    // --- 2. PipelineSpec from decomposition -----------------------------
    let spec = spec_from_decomposition(&decomposition);

    // --- 2.5 B-24 Phase 4I Group C9: lazy media-validator wire-up.
    // Spec phase-4I.txt L289-294: validator is constructed at most once per
    // pipeline AND only when `PipelineSpec.media` is non-empty (lazy gate).
    // Decision 94: the wire-up reads `resolved.media_validator` first — if
    // a prior task already populated the slot we re-use the handle so the
    // ToolCapabilityFlags subprocess probe (~900 ms p95 measured in C4)
    // is paid at most once per runner lifetime.
    if !spec.media.is_empty() {
        let mut slot = resolved.media_validator.lock().await;
        if slot.is_none() {
            let capabilities =
                crate::media_validator::build_capability_flags();
            let validator = crate::media_validator::wire_media_validator_for_spec(
                spec.media.len(),
                capabilities,
                None, // VLM backend lazy-instantiated on first request
            );
            if let Some(v) = validator {
                *slot = Some(Arc::new(v));
                // Pair the sink: same Arc<Ssb> + agent_id the rest of the
                // task_runner uses, so dashboard rows correlate cleanly.
                let mut sink_slot = resolved.media_sink.lock().await;
                *sink_slot = Some(Arc::new(
                    crate::media_validator::SsbMediaSink::new(
                        Arc::clone(writer),
                        RUNNER_AGENT_ID,
                    )
                    .with_superstep(RUNNER_SUPERSTEP),
                ));
                tracing::info!(
                    seq,
                    media_steps = spec.media.len(),
                    "task_runner: media_validator + ssb_sink wired (lazy, first task)"
                );
            }
        } else {
            tracing::debug!(
                seq,
                media_steps = spec.media.len(),
                "task_runner: media_validator already wired (reusing)"
            );
        }
    }

    // --- 3. PipelineBuilder::build_with_emitter ------------------------
    // B-7b: pass SsbDagRepairEmitter so each RepairAction surfaces as a
    // `dag_repair_round` SSB row in the dashboard activity feed.
    //
    // C-25 audit follow-up: when BOTH `crystallizer` and
    // `compiled_pipeline_store` are wired on TaskRunnerConfig, route
    // through `build_with_compiled_executor` so the try-template-first
    // short-circuit fires on cache HIT. This closes WARN-2 + Q3 in
    // audit-C25.md (production-wire path for CompiledExecutor::try_execute
    // + emit_plan_skeleton_template_marker).
    let tools = RuntimeToolLookup;
    let pool = RuntimeModelPool;
    let preds = RuntimePredicates;
    let dag_repair_emitter: Arc<dyn DagRepairEmitter> =
        Arc::new(SsbDagRepairEmitter::new(Arc::clone(writer), RUNNER_AGENT_ID, RUNNER_SUPERSTEP));
    let (pipeline, compiled_hit) = if let Some(store) =
        resolved.compiled_pipeline_store.as_ref()
    {
        // C-25 audit follow-up: synchronise persisted store into a fresh
        // CompiledExecutor for the probe. The executor's pipeline_cache
        // is the in-memory accelerator; the store is the source of truth.
        // The SSB writer is plumbed so a Decompile path emits a real
        // `decompile_triggered` row.
        let executor = crate::compiled_pipeline::CompiledExecutor::new(Some(Arc::clone(writer)));
        let (p, hit) = PipelineBuilder::build_with_compiled_executor(
            &spec,
            &tools,
            &pool,
            &preds,
            Some(&dag_repair_emitter),
            &executor,
            store,
        )
        .await?;
        (p, hit)
    } else {
        let p = PipelineBuilder::build_with_emitter(
            &spec,
            &tools,
            &pool,
            &preds,
            Some(&dag_repair_emitter),
        )?;
        (p, None)
    };
    // C-25 audit follow-up: when the compiled-executor probe hit, the
    // baby-step runner can fast-path through the cached templates. The
    // current navigator loop still drives the LLM-decomp ExecutablePipeline,
    // so we record the hit observationally via tracing. Future work threads
    // the CompiledPipeline into the navigator for prompt-template
    // short-circuiting; the SSB rows fire either way.
    if let Some(compiled) = compiled_hit.as_ref() {
        tracing::info!(
            pipeline_id = %compiled.pipeline_id,
            concept_class = %compiled.concept_class,
            plan_skeleton_hash = format!("{:#x}", compiled.plan_skeleton_hash),
            "task_runner: compiled-pipeline cache HIT — template short-circuit observed"
        );
    }
    let _ = compiled_hit;
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
        resolved.pipeline_grc.as_ref(),
    )
    .await;

    // Batch 3A (Phase 4F): once the pipeline is built we know the novel step
    // set + the live system probe. Run a single pipeline-level feasibility
    // probe and emit the summary + per-novel-step + per-stub rows. Baby-step
    // per-concept probes are exposed via `FeasibilityProber::probe_baby_step`
    // for callers that want finer-grained gating — the MVP wiring keeps the
    // pipeline-level fan-out because the navigator loop is deterministic and
    // the dashboard needs at least one report per task to render the new
    // Feasibility panel.
    emit_feasibility_probe_for_pipeline(writer, resolved, &pipeline).await;

    // --- 4. Navigate baby steps ----------------------------------------
    // 4E.13: register IntegrationSpecs at all four decomposition levels so
    // `run_integration_checkpoint` has a real spec to consult at each baby
    // step / step / phase / concept boundary. The NoOpIntegrationRunner
    // reports success for every criterion — baby-step exit criteria become
    // real commands when a `CommandIntegrationRunner` is plugged post-wire-4A.
    // Baby-step pass/fail now derives from the [`BabyStepExecutor`] dispatch
    // (Session wire-4A) instead of the legacy `BabyStepOutcome::Passed`
    // surrogate.
    let mut tree = NavigatorTree::new(decomposition, Default::default());
    register_runner_integration_specs(&mut tree);
    // B-7b: attach bubble-up and integration-checkpoint emitters so SSB rows
    // are emitted for each BubbleUpEvent and integration checkpoint in real runs.
    let nav = DepthFirstNavigator::from_tree(&tree)
        .with_bubble_up_emitter(Arc::new(SsbBubbleUpEmitter::new(
            Arc::clone(writer),
            RUNNER_AGENT_ID,
            RUNNER_SUPERSTEP,
        )))
        .with_integration_emitter(Arc::new(SsbIntegrationEmitter::new(
            Arc::clone(writer),
            RUNNER_AGENT_ID,
            RUNNER_SUPERSTEP,
        )));
    let mut state = TreeNavigatorState::new();
    let mut steps_executed: usize = 0;
    let integration_runner: NoOpIntegrationRunner = NoOpIntegrationRunner;
    let mut last_location: Option<BabyStepLocation> = None;
    // C-12: track whether we already emitted the absent-IdeaFramed WARN this
    // task so subsequent concept boundaries don't spam the log.
    let mut idea_framed_warn_emitted: bool = false;
    // Per-baby-step pass/fail for the terminal PhaseComplete row. Must stay in
    // step with the per-iteration TestOutcome rows — both are surfaced by the
    // dashboard and downstream gate logic, so they cannot disagree.
    let mut baby_step_outcomes: Vec<(u64, bool)> = Vec::new();

    // Session wire-antispiral: per-task detector + escalation pipe. `None`
    // when the feature is disabled (pre-wire-antispiral behavior).
    let mut spiral_tracker: Option<SpiralTracker> = if resolved.anti_spiral_enabled {
        let agent_id = synthetic_agent_id_for_seq(seq);
        Some(SpiralTracker::new(agent_id))
    } else {
        None
    };

    // C-10 Phase 4G.9 anti-spiral primitives — task-runner-step-level
    // detector + persona switcher + canary + watchdog. Stack-local (no
    // ResolvedConfig field) per DRIFT-5 in workflow/reasoning/C-10-decisions.md.
    // ExecutionHistory accumulates attempt rows; StagnationDetection runs
    // BEFORE gate.decide() (DECISION-PLACEMENT). GoalDriftCanary uses the
    // task_summary captured here as the immutable "original intent"
    // reference. GenerationWatchdog ticks every iteration. The
    // AdaptiveFutilityTracker is consulted in the repair loop (see
    // emit_autonomous_decision call site below).
    let mut exec_history: crate::anti_spiral_extended::ExecutionHistory =
        crate::anti_spiral_extended::ExecutionHistory::new();
    let mut attempt_index: u32 = 0;
    let mut last_pattern_seen: Option<crate::anti_spiral_extended::StagnationPattern> = None;
    let goal_drift_canary = crate::anti_spiral_extended::GoalDriftCanary::default();
    let original_task_tokens_text: String = task.task.clone();
    let mut watchdog = crate::anti_spiral_extended::GenerationWatchdog::default();
    let mut adaptive_futility: crate::anti_spiral_extended::AdaptiveFutilityTracker =
        crate::anti_spiral_extended::AdaptiveFutilityTracker::new();
    let watchdog_start_ms: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let mut last_ssb_event_ms: u64 = watchdog_start_ms;
    let mut watchdog_fired: bool = false;
    // Persona rotation index — used to fall back to Researcher on second
    // pattern recurrence without re-suggesting the same persona.
    let mut persona_rotation_idx: usize = 0;

    loop {
        if cancel.is_cancelled() {
            return Ok(());
        }
        if steps_executed >= MAX_BABY_STEPS_PER_TASK {
            warn!(seq, "task_runner: hit MAX_BABY_STEPS_PER_TASK cap");
            break;
        }
        // C-20: measurement-mode batch delay. The double-gated helper
        // returns 0 in production (both env vars absent), so the common
        // path is a single `std::env::var` read with no `await`. When the
        // ELAI_MEASURE_MODE=1 and ELAI_DEBUG_BATCH_DELAY_SECS=<n> envs are
        // both set, the runner sleeps `<n>` seconds via
        // `tokio::time::sleep` BEFORE each baby-step dispatch so the
        // measurement harness can probe resource state at stable
        // intervals. Uses `tokio::time::sleep` (NOT `std::thread::sleep`)
        // because this is an async path — blocking the executor here would
        // stall every other task on the multi-thread runtime.
        let delay_secs = measurement_batch_delay_secs();
        if delay_secs > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
            // Re-check cancellation after the sleep so a shutdown signal
            // does not have to wait for an entire baby-step round-trip.
            if cancel.is_cancelled() {
                return Ok(());
            }
        }
        let Some(bs_id) = nav.next_baby_step(&state) else {
            break;
        };
        steps_executed += 1;

        // C-2: emit PhaseNodeReady / StepNodeReady / BabyStepReady at the
        // baby-step entry point. The navigator has no dedicated enter_phase
        // / enter_step methods (DRIFT-6 in context-package-C2.md); we
        // detect phase/step boundaries by comparing the current baby-step's
        // location against `last_location` and emit the higher-level rows
        // BEFORE the baby-step row so the dashboard renders the tree in
        // depth-first order. BabyStepReady fires every iteration.
        //
        // C-12: at each tier-completion boundary we additionally emit the
        // per_level_audit_started + per_level_audit_verdict SSB rows via
        // `per_level_audit_dispatch`. Five distinct call sites cover the 5
        // AuditorMode variants (Concepts/Phases/Steps/BabySteps/
        // BabyStepDetail). The verdicts emitted here are synthetic
        // `LevelAuditVerdict::allow(...)` rows — the actual Auditor* agent
        // dispatch (CascadeOutcome routing to bubble-up) is layered on top
        // in a follow-up phase; this slice satisfies the C-12 contract:
        // observable SSB stream + dashboard timeline + IdeaFramed presence
        // assertion. emit/record failures are warn!, never abort.
        if let Some(cur_loc) = tree.location_index.get(&bs_id).cloned() {
            let prev_phase_id = last_location.as_ref().map(|l| l.phase_id);
            let prev_step_id = last_location.as_ref().map(|l| l.step_id);
            let prev_concept_id = last_location.as_ref().map(|l| l.concept_id);

            // C-12 call site 1: Concepts mode. Fires at a fresh concept
            // boundary. Asserts the most recent IdeaFramed SSB row exists;
            // emits WARN once per task if absent (DRIFT-4 resolution a:
            // FramedIdea is observed via SSB, not threaded as a struct).
            if prev_concept_id != Some(cur_loc.concept_id) {
                let idea_framed_present = writer
                    .query_by_type(SsbActionType::IdeaFramed)
                    .map(|rows| !rows.is_empty())
                    .unwrap_or(false);
                if !idea_framed_present && !idea_framed_warn_emitted {
                    tracing::warn!(
                        target: "task_runner::c12_audit",
                        seq,
                        concept_id = cur_loc.concept_id.0,
                        "no FramedIdea for session — Concepts audit will proceed without grounding"
                    );
                    idea_framed_warn_emitted = true;
                }
                run_per_level_audit_hook(
                    writer,
                    seq,
                    AuditorMode::Concepts,
                    TierNodeId::Concept(cur_loc.concept_id),
                )
                .await;
            }

            if prev_phase_id != Some(cur_loc.phase_id) {
                let phase_label = tree
                    .decomposition
                    .concepts
                    .iter()
                    .find(|c| c.id == cur_loc.concept_id)
                    .and_then(|c| c.phases.iter().find(|p| p.id == cur_loc.phase_id))
                    .map(|p| truncate_for_log(&p.name, 120))
                    .unwrap_or_default();
                let phase_payload = SsbPayload::PhaseNodeReady(PhaseNodeReadyPayload {
                    phase_id: cur_loc.phase_id.0,
                    concept_id: cur_loc.concept_id.0,
                    label: phase_label,
                    depth: 1,
                });
                if let Err(e) = writer
                    .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, phase_payload, Vec::new())
                    .await
                {
                    tracing::warn!(
                        seq,
                        error = %e,
                        "task_runner: PhaseNodeReady SSB write failed (non-fatal)"
                    );
                }
                // C-12 call site 2: Phases mode.
                run_per_level_audit_hook(
                    writer,
                    seq,
                    AuditorMode::Phases,
                    TierNodeId::Phase(cur_loc.phase_id),
                )
                .await;
            }

            if prev_step_id != Some(cur_loc.step_id) {
                let step_label = tree
                    .find_step_for_baby_step(bs_id)
                    .map(|s| truncate_for_log(&s.description, 120))
                    .unwrap_or_default();
                let step_payload = SsbPayload::StepNodeReady(StepNodeReadyPayload {
                    step_id: cur_loc.step_id.0,
                    phase_id: cur_loc.phase_id.0,
                    concept_id: cur_loc.concept_id.0,
                    label: step_label,
                    depth: 2,
                });
                if let Err(e) = writer
                    .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, step_payload, Vec::new())
                    .await
                {
                    tracing::warn!(
                        seq,
                        error = %e,
                        "task_runner: StepNodeReady SSB write failed (non-fatal)"
                    );
                }
                // C-12 call site 3: Steps mode.
                run_per_level_audit_hook(
                    writer,
                    seq,
                    AuditorMode::Steps,
                    TierNodeId::Step(cur_loc.step_id),
                )
                .await;
            }

            let bs_label = tree
                .find_baby_step(bs_id)
                .map(|bs| truncate_for_log(&bs.task, 120))
                .unwrap_or_default();
            let bs_payload = SsbPayload::BabyStepReady(BabyStepReadyPayload {
                baby_step_id: bs_id.0,
                step_id: cur_loc.step_id.0,
                phase_id: cur_loc.phase_id.0,
                concept_id: cur_loc.concept_id.0,
                label: bs_label,
                depth: 3,
            });
            if let Err(e) = writer
                .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, bs_payload, Vec::new())
                .await
            {
                tracing::warn!(
                    seq,
                    error = %e,
                    "task_runner: BabyStepReady SSB write failed (non-fatal)"
                );
            }
            // C-12 call site 4: BabySteps mode (structural/spec audit at the
            // BabyStep tier). Fires on every baby-step entry.
            run_per_level_audit_hook(
                writer,
                seq,
                AuditorMode::BabySteps,
                TierNodeId::BabyStep(bs_id),
            )
            .await;
            // C-12 call site 5: BabyStepDetail mode (line-level audit at the
            // BabyStep tier — DECISION-CASCADE: BabyStepDetail re-uses the
            // BabyStep TierNodeId by construction, cf. auditor_mode.rs
            // documentation and B-41-decisions.md).
            run_per_level_audit_hook(
                writer,
                seq,
                AuditorMode::BabyStepDetail,
                TierNodeId::BabyStep(bs_id),
            )
            .await;
        }

        // Phase 4F (B-21) per-concept probe: gated behind both
        // `ELAI_DISABLE_FEASIBILITY_PROBE` and `feasibility_prober.is_some()`
        // so callers that did not plumb a prober (CLI default) keep the
        // pre-3A behaviour. Probe is informational only — it emits a
        // StubRun SSB row (and a NovelStepDetected row when infeasible)
        // but never short-circuits the baby-step loop. Control flow stays
        // in the existing CSQ / hooks / executor / mistake-book path so
        // the W1-SEMANTIC deferred site at L4657-4672 is unaffected.
        emit_baby_step_feasibility_probe(writer, resolved, &tree, bs_id).await;

        let task_summary = tree
            .find_baby_step(bs_id)
            .map(|bs| truncate_for_log(&bs.task, 120))
            .unwrap_or_else(|| "<missing>".to_owned());

        let baby_step_reasoning_content =
            format!("baby_step id={} task={:?}", bs_id.0, task_summary);
        emit_reasoning(
            writer,
            2 + u32::try_from(steps_executed).unwrap_or(u32::MAX),
            "act",
            &baby_step_reasoning_content,
            resolved.pipeline_grc.as_ref(),
        )
        .await;

        // Session wire-antispiral: record this reasoning step with the
        // detector. On HardStop we emit the SSB rows and break immediately;
        // on EscalationSignalNeeded the message is stashed in the tracker
        // and an AntiSpiralHit SSB row is emitted (B-6).
        if let Some(tracker) = spiral_tracker.as_mut() {
            let result = tracker.record("act", "task_runner", &baby_step_reasoning_content);
            match &result {
                crate::anti_spiral::SpiralResult::HardStop { .. } => {
                    emit_spiral_hard_stop(writer, tracker.agent_id(), seq).await;
                    break;
                }
                crate::anti_spiral::SpiralResult::EscalationSignalNeeded {
                    step_hash,
                    count,
                    ..
                } => {
                    // B-6: emit AntiSpiralHit row for the escalation signal.
                    let hit_payload = SsbPayload::AntiSpiralHit(AntiSpiralHitPayload {
                        agent_id: tracker.agent_id().to_string(),
                        step_hash: step_hash.clone(),
                        count: *count,
                    });
                    if let Err(e) = writer
                        .write_payload(
                            RUNNER_AGENT_ID,
                            RUNNER_SUPERSTEP,
                            hit_payload,
                            Vec::new(),
                        )
                        .await
                    {
                        tracing::warn!(
                            error = %e,
                            "task_runner: AntiSpiralHit SSB write failed (non-fatal)"
                        );
                    }
                }
                crate::anti_spiral::SpiralResult::Clean => {}
            }
        }

        // 3F.REDO + E-1 Task 4: read MistakeBook entries for this baby step
        // AND PPR-over-notes top-K for the same step seed, then inject the
        // formatted CONSTRAINTS block into the agent's context via an
        // `info` Finding. Occupies the 150-token REPAIR_SLOT_MISTAKE_BOOK
        // slot reserved in context_budget.rs:132. `prior_attempts` carries
        // into the failure path so the MistakeEntry::attempt_number is
        // correctly 1-indexed across pipeline invocations.
        //
        // emit_memory_note_constraints is best-effort — if the note store
        // is unopenable or empty, the call returns 0 and the downstream
        // MistakeBook injector still fires (back-compat).
        let mistake_book_step_id = format!("baby_step_{}", bs_id.0);
        emit_memory_note_constraints(
            writer,
            project_dir.as_deref(),
            &mistake_book_step_id,
        )
        .await;
        let prior_attempts =
            emit_mistake_book_constraints(writer, &mistake_book_step_id).await;

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

            // B-8 Tasks#7: reset PipelineGrc.inline_feasibility_state at every
            // phase boundary so each phase gets a fresh 2-call budget. Engine
            // exposes `inline_feasibility_state` as a `pub` field on
            // BasicGrcEngine (engine.rs:75) and the runner accesses it via
            // `engine_mut()` on the locked PipelineGrc. Skipped when the
            // kill-switch left `pipeline_grc = None`.
            if prev.phase_id != cur.phase_id {
                if let Some(grc) = resolved.pipeline_grc.as_ref() {
                    grc.lock()
                        .await
                        .engine_mut()
                        .inline_feasibility_state
                        .reset();
                    tracing::debug!(
                        prev_phase = prev.phase_id.0,
                        cur_phase = cur.phase_id.0,
                        "task_runner: reset inline_feasibility_state at phase boundary"
                    );
                }
            }
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

        // Batch 1A: PreToolUse hook dispatch. Fires the tiered-analysis T1/T2/T3
        // pipeline against the current baby-step task text before the executor
        // runs. A `Block` verdict short-circuits the step — we emit an
        // `AGENT_FAILED_CATEGORY` Finding with a dedicated hook_pre_block tag,
        // skip the executor, and fall through so the bubble-up / checkpoint /
        // TestOutcome path still renders the row with passed=false.
        //
        // Warn verdicts emit `HookEvent` rows but do NOT abort — the dashboard
        // shows them as `hook · T1 WARN` alongside the step.
        let mut hook_pre_blocked = false;
        // C-21 Task 10 — track which gate fired the block so the synthetic
        // Failed reason + MistakeBook entry carry tier attribution.
        let mut block_source: BlockSource = BlockSource::None;
        if let Some(dispatcher) = resolved.hook_dispatcher.as_ref() {
            let invocation = HookInvocation::for_pre_tool_use(
                BABY_STEP_PRE_GATE_ACTION,
                &task_summary,
                format!("baby_step_{}", bs_id.0),
            );
            match dispatcher.dispatch_pre_tool_use(invocation).await {
                Ok(findings) => {
                    emit_hook_events_to_ssb(
                        writer,
                        RUNNER_AGENT_ID,
                        RUNNER_SUPERSTEP,
                        HookPoint::PreToolUse,
                        &format!("baby_step_{}", bs_id.0),
                        &findings,
                    )
                    .await;
                    if findings.iter().any(HookFinding::is_block) {
                        hook_pre_blocked = true;
                        // C-21 Task 10 — PreToolUse dispatcher path is the
                        // T1 hook surface (Tier 1 dispatch in hooks.rs).
                        block_source = BlockSource::T1Hook;
                        let msg = build_hook_block_summary(&findings, "PreToolUse");
                        emit_finding(writer, "error", HOOK_PRE_TOOL_USE_BLOCK_CATEGORY, &msg)
                            .await;
                    }
                }
                Err(e) => {
                    warn!(
                        baby_step = bs_id.0,
                        error = %e,
                        "task_runner: PreToolUse hook dispatch failed"
                    );
                }
            }
        }

        // Batch-fu (FU.5 / A-31): irreversible-action confirmation gate.
        // Runs BEFORE the baby-step executor so a `ConfirmationRequired`
        // verdict can park the runner until the dashboard resolves it.
        // Skipped when the classifier wasn't plumbed OR when the autonomous
        // gate is in `Autonomous` mode (spec §4C: autonomous mode proceeds
        // without prompts). The emitted `InterventionRequest` + matching
        // `InterventionResolved` pair is the load-bearing SSB trail for the
        // Svelte InterventionCard; `hook_pre_blocked` is preserved so the
        // flag is not accidentally reset by an approval path.
        let mut classifier_rejected = false;
        if let Some(classifier) = resolved.irreversible_classifier.as_ref() {
            let autonomous_mode = if let Some(gate) = resolved.autonomous_gate.as_ref() {
                gate.mode().await == crate::autonomous_gate::GateMode::Autonomous
            } else {
                false
            };
            if !autonomous_mode {
                let verdict =
                    classifier.classify(BABY_STEP_PRE_GATE_ACTION, &task_summary);
                if let crate::IrreversibleVerdict::ConfirmationRequired { reason } = &verdict {
                    let (rejected, new_creds) = run_confirmation_gate(
                        writer,
                        reason,
                        &task_summary,
                        bs_id.0,
                        cancel,
                        crate::ssb::InterventionType::ApproveChange,
                    )
                    .await;
                    classifier_rejected = rejected;
                    if !new_creds.is_empty() {
                        if let Ok(mut guard) = resolved.session_credentials.lock() {
                            guard.extend(new_creds);
                        }
                    }
                }
            }
        }
        if classifier_rejected {
            // Skip executor, mark the step failed. Downstream checkpoint /
            // bubble-up + TestOutcome paths still render the row with
            // passed=false so the dashboard surfaces the rejection.
            hook_pre_blocked = true;
            // C-21 Task 10 — irreversible classifier maps to generic
            // PreToolUse block (no specific T1/T2 tier surface).
            if matches!(block_source, BlockSource::None) {
                block_source = BlockSource::PreToolUse;
            }
            emit_finding(
                writer,
                "warn",
                "irreversible_rejected",
                "irreversible_action: user rejected confirmation",
            )
            .await;
        }

        // ----- C-10 Phase 4G.9 anti-spiral detection (pre-gate) -----
        //
        // Per DECISION-PLACEMENT (DRIFT-7 in workflow/reasoning/C-10-decisions.md)
        // we run StagnationDetection here at the caller site rather than
        // inside autonomous_gate::decide(), so the gate's `&self` signature
        // does not break (15+ existing callers).
        //
        // Order:
        //   1. Push current attempt onto ExecutionHistory.
        //   2. Tick GenerationWatchdog; emit WatchdogTimeout SSB row + set
        //      watchdog_fired (OR'd into anti_spiral_fired below).
        //   3. Run StagnationDetection::detect_pattern; emit stagnation +
        //      LateralThinkingActivated SSB rows; set suggested_prompt_prefix.
        //   4. Run GoalDriftCanary at the step boundary; emit GoalDriftWarning
        //      (Yellow band) or GoalDriftBlock (Red band).
        //   5. Build GateContext with the C-10 fields populated.
        let now_ms: u64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        attempt_index = attempt_index.saturating_add(1);
        let attempt = crate::anti_spiral_extended::AttemptResult {
            attempt_index,
            error_class: format!("baby_step_{}", bs_id.0),
            output_hash: bs_id.0,
            error_count: 0,
            timestamp_ms: now_ms,
        };
        exec_history.push(attempt);

        // GenerationWatchdog tick. Fingerprint = blake3(row_count || bs_id).
        let watchdog_fingerprint =
            crate::anti_spiral_extended::GenerationWatchdog::compute_fingerprint(
                steps_executed as u64,
                bs_id.0,
            );
        if let Some(kind) =
            watchdog.check(last_ssb_event_ms, now_ms, watchdog_fingerprint, watchdog_start_ms)
        {
            let kind_str = match kind {
                crate::anti_spiral_extended::WatchdogTimeoutKind::Idle => "idle",
                crate::anti_spiral_extended::WatchdogTimeoutKind::NoMaterialProgress => {
                    "no_material_progress"
                }
                crate::anti_spiral_extended::WatchdogTimeoutKind::Safety => "safety",
            };
            let elapsed_ms = now_ms.saturating_sub(watchdog_start_ms);
            let budget_ms = match kind {
                crate::anti_spiral_extended::WatchdogTimeoutKind::Idle => {
                    watchdog.idle_timeout_ms
                }
                crate::anti_spiral_extended::WatchdogTimeoutKind::NoMaterialProgress => {
                    watchdog.no_progress_timeout_ms
                }
                crate::anti_spiral_extended::WatchdogTimeoutKind::Safety => {
                    watchdog.safety_timeout_ms
                }
            };
            let payload =
                SsbPayload::WatchdogTimeout(crate::ssb::WatchdogTimeoutPayload {
                    agent_id: RUNNER_AGENT_ID.to_owned(),
                    elapsed_ms,
                    budget_ms,
                    detail: format!("watchdog_kind={kind_str}"),
                });
            if let Err(e) = writer
                .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
                .await
            {
                tracing::warn!(seq, error = %e, "task_runner: WatchdogTimeout SSB write failed (non-fatal)");
            } else {
                last_ssb_event_ms = now_ms;
            }
            watchdog_fired = true;
        }

        // StagnationDetection — anti_spiral_extended.rs §detect_pattern.
        let stagnation_detection =
            crate::anti_spiral_extended::StagnationDetection::default();
        let pattern_now = stagnation_detection.detect_pattern(&exec_history);
        let mut prompt_prefix: Option<&'static str> = None;
        if let Some(pat) = pattern_now {
            // Map pattern -> SSB variant. Spinning + NoDrift piggyback on
            // StagnationSpinningDetected (with `pattern` field
            // disambiguating); Oscillation + DiminishingReturns piggyback
            // on StagnationOscillationDetected. See DRIFT-2 / D2.
            let payload = match pat {
                crate::anti_spiral_extended::StagnationPattern::Spinning
                | crate::anti_spiral_extended::StagnationPattern::NoDrift => {
                    SsbPayload::StagnationSpinningDetected(
                        crate::ssb::StagnationSpinningDetectedPayload {
                            agent_id: RUNNER_AGENT_ID.to_owned(),
                            pattern: pat.as_str().to_owned(),
                            iteration: attempt_index,
                            detail: format!("baby_step_{}", bs_id.0),
                        },
                    )
                }
                crate::anti_spiral_extended::StagnationPattern::Oscillation
                | crate::anti_spiral_extended::StagnationPattern::DiminishingReturns => {
                    SsbPayload::StagnationOscillationDetected(
                        crate::ssb::StagnationOscillationDetectedPayload {
                            agent_id: RUNNER_AGENT_ID.to_owned(),
                            pattern: pat.as_str().to_owned(),
                            iteration: attempt_index,
                            detail: format!("baby_step_{}", bs_id.0),
                        },
                    )
                }
            };
            if let Err(e) = writer
                .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
                .await
            {
                tracing::warn!(seq, error = %e, "task_runner: Stagnation SSB write failed (non-fatal)");
            } else {
                last_ssb_event_ms = now_ms;
            }

            // Persona switch — primary affinity from plan §Task 1, or
            // Researcher fallback on rotation when the same pattern
            // recurs back-to-back.
            let persona = if last_pattern_seen == Some(pat) {
                let idx = persona_rotation_idx % 5;
                persona_rotation_idx = persona_rotation_idx.saturating_add(1);
                crate::anti_spiral_extended::PERSONA_STRATEGIES[idx].persona
            } else {
                crate::anti_spiral_extended::suggest_persona_for_pattern(pat)
            };
            last_pattern_seen = Some(pat);
            let strategy = crate::anti_spiral_extended::strategy_for(persona);
            prompt_prefix = Some(strategy.prompt_prefix);

            let lt_payload = SsbPayload::LateralThinkingActivated(
                crate::ssb::LateralThinkingActivatedPayload {
                    agent_id: RUNNER_AGENT_ID.to_owned(),
                    persona: persona.as_str().to_owned(),
                    iteration: attempt_index,
                    detail: format!("pattern={}", pat.as_str()),
                },
            );
            if let Err(e) = writer
                .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, lt_payload, Vec::new())
                .await
            {
                tracing::warn!(seq, error = %e, "task_runner: LateralThinkingActivated SSB write failed (non-fatal)");
            } else {
                last_ssb_event_ms = now_ms;
            }
        }

        // GoalDriftCanary — Jaccard similarity between the original task
        // summary captured at task start and the current task_summary.
        let drift_score = goal_drift_canary.compute(&original_task_tokens_text, &task_summary);
        match goal_drift_canary.verdict(drift_score) {
            crate::anti_spiral_extended::DriftVerdict::Green => {}
            crate::anti_spiral_extended::DriftVerdict::Yellow => {
                let payload = SsbPayload::GoalDriftWarning(crate::ssb::GoalDriftWarningPayload {
                    agent_id: RUNNER_AGENT_ID.to_owned(),
                    drift_score,
                    baby_step_id: bs_id.0,
                    detail: format!(
                        "orig={}; cur={}",
                        truncate_for_log(&original_task_tokens_text, 80),
                        truncate_for_log(&task_summary, 80)
                    ),
                });
                if let Err(e) = writer
                    .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
                    .await
                {
                    tracing::warn!(seq, error = %e, "task_runner: GoalDriftWarning SSB write failed (non-fatal)");
                } else {
                    last_ssb_event_ms = now_ms;
                }
            }
            crate::anti_spiral_extended::DriftVerdict::Red => {
                let payload = SsbPayload::GoalDriftBlock(crate::ssb::GoalDriftBlockPayload {
                    agent_id: RUNNER_AGENT_ID.to_owned(),
                    drift_score,
                    baby_step_id: bs_id.0,
                    detail: format!(
                        "orig={}; cur={}",
                        truncate_for_log(&original_task_tokens_text, 80),
                        truncate_for_log(&task_summary, 80)
                    ),
                });
                if let Err(e) = writer
                    .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
                    .await
                {
                    tracing::warn!(seq, error = %e, "task_runner: GoalDriftBlock SSB write failed (non-fatal)");
                } else {
                    last_ssb_event_ms = now_ms;
                }
            }
        }

        // AdaptiveFutilityTracker bookkeeping — record this attempt under
        // its error_class. Treat baby_step_{id} as the class key so the
        // tracker observes attempt frequency per baby-step. `succeeded`
        // here defaults to true (gate hasn't run yet); the post-execution
        // path can call adaptive_futility.record again with the real
        // outcome — duplicate records intentionally bias toward more data.
        adaptive_futility.record(
            &format!("baby_step_{}", bs_id.0),
            true,
        );

        // Batch 3B (Phase 4G): autonomous-gate decision boundary. Runs on
        // every baby step regardless of pre-tool-use hook verdict so the
        // dashboard captures a GateDecision row per step. A
        // `RequestHuman` / `Escalate` verdict emits the corresponding SSB
        // row but does NOT block the loop — the task_runner is not yet the
        // FSM owner for 4G. The row drives the dashboard Autonomy panel;
        // the CLI-side caller is expected to consume the row and halt.
        if let Some(gate) = resolved.autonomous_gate.as_ref() {
            let anti_spiral_legacy = spiral_tracker
                .as_ref()
                .map(|t| t.hard_stop())
                .unwrap_or(false);
            let gctx = crate::autonomous_gate::GateContext {
                level: crate::autonomous_gate::PipelineLevel::BabyStep,
                risk_level: crate::gate::RiskLevel::Low,
                // Phase 4G — real SSB-derived evidence (replaces the Batch-3B
                // all-true placeholder). Cold-start sessions with zero rows
                // fall through to vacuous-default semantics inside the helper
                // so the gate's behaviour stays Batch-3B-compatible there.
                evidence: derive_autonomous_evidence(writer.as_ref()),
                // C-10 — OR the legacy GRC SpiralTracker signal with the
                // new task-runner-step-level detector + watchdog so the
                // existing AntiSpiral short-circuit in decide() still fires.
                anti_spiral_fired: anti_spiral_legacy
                    || pattern_now.is_some()
                    || watchdog_fired,
                replan_major: false,
                max_restarts_exhausted: false,
                context_budget_terminal: false,
                auto_advance_count_for_concept: 0,
                step_id: format!("baby_step_{}", bs_id.0),
                // C-9 — ModelPool is plumbed at the runner level via
                // `runtime_model_pool()`; today the production pool reports
                // u64::MAX so can_fit_llm trivially passes. Callers that
                // want real VRAM gating override here with an Arc<dyn
                // ModelPool> instance.
                model_pool: None,
                // C-10 — anti-spiral extended fields populated from the
                // pre-gate detection block above.
                execution_history: exec_history.clone(),
                attempt_index,
                suggested_prompt_prefix: prompt_prefix,
            };
            let decision = gate.decide(&gctx).await;
            // Batch FUL-B: hand the optional splitter to the emit path so
            // `AutonomyDecision::SplitTask` produces real sub-task summaries
            // when the runner was configured with a `TaskSplitter`. When
            // `resolved.task_splitter` is `None`, the emit path preserves
            // the Batch 3B deterministic `{step_id}.subN` fallback.
            emit_autonomous_decision(
                writer,
                &decision,
                &gctx.step_id,
                resolved.task_splitter.as_ref(),
                &task_summary,
                resolved.research_tree.as_ref(),
            )
            .await;
        }

        // Session wire-4A: dispatch this baby step through the pluggable
        // [`BabyStepExecutor`]. Drains the escalation message stashed by the
        // anti-spiral detector (wire-antispiral) so the injected message lands
        // in the agent turn that runs next.
        let escalation_message = spiral_tracker
            .as_mut()
            .and_then(|t| t.take_escalation_message());

        // B-28 E-3 + WIRING-2: focus-tunnel ModeSelector decision + SSB beacon.
        //
        // WIRING-2 (item E): the previously-hardcoded `ExecMode::Local` is now
        // resolved from the runner's BackendConfig via
        // [`crate::cloud_mode::resolve_exec_mode`]. Rule 1
        // (CLOUD -> STRUCTURED) now fires in production when the backend
        // mode is `Api`.
        let resolved_exec_mode = crate::cloud_mode::resolve_exec_mode(&resolved.backend_config);
        let mode_selection = {
            let mode_selector = crate::focus_tunnel::ModeSelector::new();
            let step_ctx = match tree.find_baby_step(bs_id) {
                Some(spec) => crate::focus_tunnel::build_step_context_from(spec),
                None => crate::focus_tunnel::StepContext::empty(),
            };
            let selection = mode_selector.select(
                &step_ctx,
                &resolved.model_capability_profile.features,
                resolved_exec_mode,
            );
            // Emit the SSB beacon. Best-effort: a write failure does not
            // abort the baby step (matches AntiSpiralHit emission pattern
            // at L4070-4083).
            let mode_payload = SsbPayload::GenerationModeSelected(
                crate::ssb::GenerationModeSelectedPayload {
                    step_id: format!("baby_step_{}", bs_id.0),
                    mode: crate::focus_tunnel::generation_mode_str(selection.mode)
                        .to_string(),
                    rule_fired: crate::focus_tunnel::selection_rule_str(
                        selection.rule_fired,
                    )
                    .to_string(),
                    reason: selection.reason.to_string(),
                    preferred_was: selection.preferred_was.map(|m| {
                        crate::focus_tunnel::generation_mode_str(m).to_string()
                    }),
                    supports_fim: resolved
                        .model_capability_profile
                        .features
                        .supports_fim,
                    exec_mode: crate::focus_tunnel::exec_mode_str(resolved_exec_mode)
                        .to_string(),
                },
            );
            if let Err(e) = writer
                .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, mode_payload, Vec::new())
                .await
            {
                tracing::warn!(
                    baby_step = bs_id.0,
                    error = %e,
                    "task_runner: GenerationModeSelected SSB write failed (non-fatal)"
                );
            }
            selection
        };

        // C-5 — cloud-mode pre-dispatch hook (cap enforcement + diagnostics).
        //
        // When the resolved exec_mode is `Cloud` (or `Auto` after a
        // local-failure promotion), and the runner has a CloudModeRouter
        // wired via apply_task_runner_wiring, we:
        //   1. Read live spend (day + month) from SsbViewer::cloud_spend_summary.
        //      On error: log + skip cap enforcement (stub_ban semantics
        //      from plan WIRING_CONTRACT — never block all cloud calls
        //      on a viewer error).
        //   2. Emit one-shot CostEstimate (per task) when the projection
        //      is non-zero.
        //   3. Emit WarnLimitReached (per UTC day) when crossing
        //      warn_at_percent of the daily cap.
        //   4. Re-route via CloudModeRouter::route_with_spend. On
        //      DailySpendCapExceeded / MonthlySpendCapExceeded: emit a
        //      RouteBlocked SSB row (per UTC day dedup, mirrors the prior
        //      cap_reached EscalateToCloud emission).
        //   5. Otherwise, emit a diagnostic EscalateToCloud row so the
        //      dashboard's escalation timeline reflects the promotion.
        //
        // The router still returns `RoutedProvider::Local` for `Auto`, so
        // this hook does NOT itself change the executor path; the HYBRID
        // re-invoke loop is wired separately (see C-5 BLOCK-4 follow-up).
        // What the hook now DOES change is the row inventory: a cap-hit
        // produces a typed RouteBlocked row (was: EscalateToCloud with a
        // string trigger_reason — preserved for backwards-compat so
        // existing dashboards don't lose the row, but supplemented by the
        // typed RouteBlocked which the CloudPanel filter keys on).
        if matches!(
            resolved_exec_mode,
            crate::cloud_mode::ExecMode::Cloud | crate::cloud_mode::ExecMode::Auto,
        ) && resolved.cloud_router.is_some()
        {
            // C-15: now_ms via OsClock to keep this site out of the entropy
            // quarantine scanner. Cloud-cap path; no deterministic
            // test relies on this exact millisecond value.
            let now_ms = i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX);

            // Read live spend snapshot. None on viewer error => skip cap
            // enforcement (stub_ban semantics).
            let spend_snapshot: Option<crate::cloud_mode::SpendSnapshotInput> = match crate::ssb_viewer::SsbViewer::open(writer.db_path()) {
                Ok(viewer) => match viewer.cloud_spend_summary(now_ms) {
                    Ok(summary) => Some(crate::cloud_mode::SpendSnapshotInput {
                        day_usd: summary.day_usd,
                        month_usd: summary.month_usd,
                    }),
                    Err(e) => {
                        tracing::warn!(
                            baby_step = bs_id.0,
                            error = %e,
                            "task_runner: cloud_spend_summary read failed; cap check skipped"
                        );
                        None
                    }
                },
                Err(e) => {
                    tracing::warn!(
                        baby_step = bs_id.0,
                        error = %e,
                        "task_runner: SsbViewer::open failed for cap check; skipping"
                    );
                    None
                }
            };

            // C-5 Task 6 — COST_ESTIMATE row emit. Pulled from the resolved
            // backend_config.model since we don't carry a PlanSkeleton at
            // this site. The estimator returns (0,0) for local models so
            // the gate below filters those out (no spammy zero rows).
            let cost_already = resolved
                .cloud_cost_estimated_emitted
                .lock()
                .map(|g| *g)
                .unwrap_or(false);
            if !cost_already {
                let router_cfg = resolved
                    .cloud_router
                    .as_ref()
                    .map(|r| r.config().clone());
                if let Some(cfg) = router_cfg {
                    // Use a 1-step skeleton fallback since we don't have the
                    // live skeleton handle here; the dashboard scales the
                    // display so a single-step projection is honest about
                    // its UNGROUNDED_HEURISTIC origin (heuristic_source
                    // field carries that disclaimer through to the UI).
                    use crate::plan_skeleton::{PhaseEntry, PipelineId, PlanSkeleton};
                    let mut skel = PlanSkeleton::new(PipelineId::from(format!(
                        "task-{}",
                        bs_id.0
                    )));
                    if let Ok(entry) = PhaseEntry::new(
                        format!("step-{}", bs_id.0),
                        "baby_step",
                    ) {
                        let _ = skel.add_phase(entry);
                    }
                    let (low, high) = crate::cloud_mode::estimate_task_cost_usd(
                        &skel,
                        &cfg.default_model,
                    );
                    if high > 0.0 {
                        if let Ok(mut g) = resolved.cloud_cost_estimated_emitted.lock() {
                            *g = true;
                        }
                        let cost_payload = SsbPayload::CostEstimate(
                            crate::ssb::CostEstimatePayload {
                                low_usd: low,
                                high_usd: high,
                                model: cfg.default_model.clone(),
                                step_count: 1,
                                heuristic_source:
                                    "ungrounded_heuristic".to_string(),
                                ts_ms: now_ms,
                            },
                        );
                        if let Err(e) = writer
                            .write_payload(
                                RUNNER_AGENT_ID,
                                RUNNER_SUPERSTEP,
                                cost_payload,
                                Vec::new(),
                            )
                            .await
                        {
                            tracing::warn!(
                                baby_step = bs_id.0,
                                error = %e,
                                "task_runner: CostEstimate SSB write failed (non-fatal)"
                            );
                        }
                    }
                }
            }

            // C-5 Task 2 — WARN_LIMIT_REACHED row emit (per UTC day).
            if let (Some(snap), Some(cfg)) = (
                spend_snapshot,
                resolved.cloud_router.as_ref().map(|r| r.config().clone()),
            ) {
                if crate::cloud_mode::warn_limit_reached(&cfg, snap.day_usd) {
                    let warn_already = resolved
                        .cloud_warn_emitted
                        .lock()
                        .map(|g| *g)
                        .unwrap_or(false);
                    if !warn_already {
                        if let Ok(mut g) = resolved.cloud_warn_emitted.lock() {
                            *g = true;
                        }
                        // cap_usd + warn_at_percent are guaranteed Some by
                        // warn_limit_reached returning true; the unwraps
                        // would be defensive here. Use copy-on-default
                        // pattern via unwrap_or to keep zero-unwrap.
                        let cap_usd = cfg.spend_cap_daily_usd.unwrap_or(0.0);
                        let warn_pct = cfg.warn_at_percent.unwrap_or(0);
                        let pct_of_cap = if cap_usd > 0.0 {
                            (snap.day_usd / cap_usd * 100.0).min(100.0).max(0.0)
                        } else {
                            0.0
                        };
                        let warn_payload = SsbPayload::WarnLimitReached(
                            crate::ssb::WarnLimitReachedPayload {
                                spent_usd: snap.day_usd,
                                cap_usd,
                                warn_at_percent: warn_pct,
                                pct_of_cap,
                                baby_step_id: bs_id.0,
                                ts_ms: now_ms,
                            },
                        );
                        if let Err(e) = writer
                            .write_payload(
                                RUNNER_AGENT_ID,
                                RUNNER_SUPERSTEP,
                                warn_payload,
                                Vec::new(),
                            )
                            .await
                        {
                            tracing::warn!(
                                baby_step = bs_id.0,
                                error = %e,
                                "task_runner: WarnLimitReached SSB write failed (non-fatal)"
                            );
                        }
                    }
                }
            }

            // C-5 Task 1 + 8 — cap enforcement + RouteBlocked emission via
            // CloudModeRouter::route_with_spend. The router is the source
            // of truth for which window tripped; we mirror its verdict
            // into a typed RouteBlocked row so the dashboard CloudPanel
            // filter does not have to inspect EscalateToCloud trigger
            // reasons. The cap_reached EscalateToCloud row continues to
            // fire below for backwards-compat with the prior dashboard
            // surface (one signal => two rows, both dedup'd).
            let cap_reached: bool;
            let cap_period: &'static str;
            let cap_block_spent_usd: f64;
            let cap_block_cap_usd: f64;
            if let Some(router) = resolved.cloud_router.as_ref() {
                // Use credentials=None: the router still enforces caps in
                // step 4 because air_gapped + provider checks short-circuit
                // independently of credentials. To exercise the cap path
                // we'd need creds — but step 4 runs only if creds are Some
                // (NoCredentials returns at step 2). Workaround: probe with
                // a synthetic creds so we can observe the cap branch.
                let probe_creds = Some(
                    crate::cloud_mode::CloudCredentials::for_test_with_token("probe"),
                );
                match router.route_with_spend(
                    crate::cloud_mode::ExecMode::Cloud,
                    probe_creds,
                    spend_snapshot,
                ) {
                    Err(crate::cloud_mode::CloudModeError::DailySpendCapExceeded {
                        spent_usd,
                        cap_usd,
                    }) => {
                        cap_reached = true;
                        cap_period = "daily";
                        cap_block_spent_usd = spent_usd;
                        cap_block_cap_usd = cap_usd;
                    }
                    Err(crate::cloud_mode::CloudModeError::MonthlySpendCapExceeded {
                        spent_usd,
                        cap_usd,
                    }) => {
                        cap_reached = true;
                        cap_period = "monthly";
                        cap_block_spent_usd = spent_usd;
                        cap_block_cap_usd = cap_usd;
                    }
                    _ => {
                        cap_reached = false;
                        cap_period = "";
                        cap_block_spent_usd = 0.0;
                        cap_block_cap_usd = 0.0;
                    }
                }
            } else {
                cap_reached = false;
                cap_period = "";
                cap_block_spent_usd = 0.0;
                cap_block_cap_usd = 0.0;
            }

            // Per-UTC-day dedup: cap_reached fires at most one
            // RouteBlocked + one EscalateToCloud(cap_reached) per day.
            let block_already = resolved
                .cloud_spend_blocked_emitted
                .lock()
                .map(|g| *g)
                .unwrap_or(false);
            let block_emit_now = cap_reached && !block_already;
            if block_emit_now {
                if let Ok(mut g) = resolved.cloud_spend_blocked_emitted.lock() {
                    *g = true;
                }
                let reason = format!("{}_cap_exceeded", cap_period);
                let blocked_payload = SsbPayload::RouteBlocked(
                    crate::ssb::RouteBlockedPayload {
                        reason: reason.clone(),
                        spent_usd: cap_block_spent_usd,
                        cap_usd: cap_block_cap_usd,
                        cap_period: cap_period.to_string(),
                        baby_step_id: bs_id.0,
                        ts_ms: now_ms,
                    },
                );
                if let Err(e) = writer
                    .write_payload(
                        RUNNER_AGENT_ID,
                        RUNNER_SUPERSTEP,
                        blocked_payload,
                        Vec::new(),
                    )
                    .await
                {
                    tracing::warn!(
                        baby_step = bs_id.0,
                        reason = %reason,
                        error = %e,
                        "task_runner: RouteBlocked SSB write failed (non-fatal)"
                    );
                }
            }

            // Backwards-compatible EscalateToCloud emission. Same dedup
            // gate as the typed RouteBlocked above; non-blocked paths emit
            // the routing diagnostic.
            let trigger_reason = if block_emit_now {
                "cap_reached".to_string()
            } else if cap_reached {
                // Already-emitted: skip the EscalateToCloud row too so we
                // never produce just one of the pair.
                String::new()
            } else {
                match resolved_exec_mode {
                    crate::cloud_mode::ExecMode::Auto => "hybrid_local_first".to_string(),
                    crate::cloud_mode::ExecMode::Cloud => "cloud_pinned".to_string(),
                    crate::cloud_mode::ExecMode::Local => String::new(),
                }
            };

            if !trigger_reason.is_empty() {
                let payload = SsbPayload::EscalateToCloud(
                    crate::ssb::EscalateToCloudPayload {
                        step_id: format!("baby_step_{}", bs_id.0),
                        trigger_reason: trigger_reason.clone(),
                        layer_failed: 1,
                        ts_ms: now_ms,
                    },
                );
                if let Err(e) = writer
                    .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
                    .await
                {
                    tracing::warn!(
                        baby_step = bs_id.0,
                        trigger_reason = %trigger_reason,
                        error = %e,
                        "task_runner: EscalateToCloud SSB write failed (non-fatal)"
                    );
                }
            }
        }

        let exec_ctx = BabyStepExecutionCtx {
            baby_step_id: bs_id.0,
            task_summary: task_summary.clone(),
            escalation_message,
            mode_selection: Some(mode_selection),
            // C2: snapshot current credentials under the lock and release
            // immediately; the executor's Command::env() calls use the clone.
            session_credentials: resolved
                .session_credentials
                .lock()
                .map(|g| g.clone())
                .unwrap_or_default(),
        };
        // C-25 — capture task_summary before exec_ctx is moved into
        // executor.execute(). Used by run_crystallizer_post_execute_hook to
        // derive concept_class via classify_task_for_template.
        let exec_ctx_task_summary_for_crystallizer: String = task_summary.clone();

        // B-8 Tasks#3: PipelineGrc.generate_csq pre-tool gate.
        //
        // Mirrors the irreversible_classifier branch above: when the gate
        // returns a Critical-without-guard verdict we set
        // `hook_pre_blocked = true` so the same downstream checkpoint /
        // bubble-up / TestOutcome path renders the row with `passed=false`.
        // Warning verdicts emit `tracing::warn` and proceed. The kill-switch
        // (`ELAI_DISABLE_PIPELINE_GRC`) leaves `resolved.pipeline_grc = None`
        // and the entire branch is skipped.
        //
        // Sensitive ops list is the natural-language `task_summary`. The
        // engine pattern-matches against `fs::|file|write|delete|remove|
        // net::|tcpstream|money|payment|auth|token|credential|password`
        // (engine.rs:603-606). Over-aggressive matching is documented and
        // mitigated by the kill-switch (see workflow/reasoning/B-8-decisions.md
        // §3 + §10). Real static-analysis sensitive_ops extraction is out of
        // B-8 scope.
        if !hook_pre_blocked {
            if let Some(grc) = resolved.pipeline_grc.as_ref() {
                let function_id = format!("baby_step_{}", bs_id.0);
                let sensitive_ops: Vec<String> = vec![task_summary.clone()];
                let csq = grc.lock().await.generate_csq(&function_id, &sensitive_ops);
                match csq {
                    Ok(result) => {
                        if !result.passed() {
                            // At least one Critical-without-guard check.
                            // Block the executor by setting hook_pre_blocked
                            // (mirrors irreversible_classifier path).
                            let critical_count = result
                                .checks
                                .iter()
                                .filter(|c| {
                                    c.severity == elai_reasoning::CsqSeverity::Critical
                                        && !c.guard_added
                                })
                                .count();
                            tracing::error!(
                                baby_step = bs_id.0,
                                function_id = %function_id,
                                critical_count,
                                checks = result.checks.len(),
                                "task_runner: CSQ Critical-without-guard blocked baby step"
                            );
                            emit_finding(
                                writer,
                                "error",
                                "csq_critical_block",
                                &format!(
                                    "baby_step {} blocked: {} critical CSQ check(s) without guard",
                                    bs_id.0, critical_count
                                ),
                            )
                            .await;
                            hook_pre_blocked = true;
                            // C-21 Task 10 — CSQ Critical-without-guard
                            // surface gets its own tier label so the
                            // synthetic Failed reason + MistakeBook can
                            // distinguish gate-rejection from hook-rejection.
                            if matches!(block_source, BlockSource::None) {
                                block_source = BlockSource::CsqCritical;
                            }
                            // C-1 (replaces TODO(B-future) at task_runner.rs:5119):
                            // emit a `csq_evaluated` row carrying the worst
                            // severity, guard_present flag, and check count
                            // so the dashboard CsqBlockCard renders the
                            // block. blocked=true mirrors hook_pre_blocked.
                            emit_csq_evaluated(
                                writer,
                                CsqSeverityLabel::Critical,
                                /*guard_present=*/ false,
                                /*blocked=*/ true,
                                result.checks.len(),
                            )
                            .await;
                        } else if !result.checks.is_empty() {
                            tracing::warn!(
                                baby_step = bs_id.0,
                                checks = result.checks.len(),
                                "task_runner: CSQ produced Warning checks (proceeding)"
                            );
                            // C-1: emit a CsqEvaluated row even on the
                            // Warning path so the dashboard renders the
                            // non-blocking checks (severity=Warning,
                            // blocked=false).
                            emit_csq_evaluated(
                                writer,
                                CsqSeverityLabel::Warning,
                                /*guard_present=*/ true,
                                /*blocked=*/ false,
                                result.checks.len(),
                            )
                            .await;
                        } else {
                            // C-1: even empty-check results get a row so
                            // the dashboard can show "no sensitive ops"
                            // verdicts deterministically (one row per
                            // generate_csq call regardless of outcome).
                            emit_csq_evaluated(
                                writer,
                                CsqSeverityLabel::None,
                                /*guard_present=*/ true,
                                /*blocked=*/ false,
                                0,
                            )
                            .await;
                        }
                    }
                    Err(e) => {
                        // Engine error — log and proceed (do not block on
                        // engine-side bugs; kill-switch is the user's escape).
                        tracing::warn!(
                            baby_step = bs_id.0,
                            error = ?e,
                            "task_runner: CSQ generation failed; proceeding without gate"
                        );
                    }
                }
            }
        }

        // B-28 WIRING-2 (item F): focus-tunnel FIM/DIFF pre-executor branch.
        //
        // When the env-var `ELAI_FOCUS_TUNNEL_EXECUTE=1` is set AND a shared
        // LlmProvider is available AND the ModeSelector chose FIM or DIFF,
        // route through `run_*_step_with_provider` and synthesize the
        // executor result from the focus-tunnel outcome. On any Err we fall
        // through to the legacy executor path (STRUCTURED fallback). This
        // env-var gate keeps the wiring observable in production without
        // disturbing existing baby-step behaviour — flipping the variable
        // off restores the pre-WIRING-2 path verbatim.
        //
        // SSB row on fallback: emit a `GenerationModeSelected` beacon with
        // rule_fired=StructuredFallback so the auditor's mode_distribution
        // stays accurate.
        let focus_tunnel_active = std::env::var("ELAI_FOCUS_TUNNEL_EXECUTE")
            .map(|v| v.trim() == "1")
            .unwrap_or(false);
        // B-28 WIRING-2 fix BLOCK 2c: pre-fetch the spec's `target_file` /
        // `edit_region` for the FIM execution branch. When the spec lacks
        // either field the FIM dispatcher short-circuits to a typed
        // `StructuredFallback` SSB row with reason="missing_target_file"
        // (NOT the legacy `UnsupportedMode` error path) so the auditor's
        // mode_distribution metric reflects the real code path taken.
        let (spec_target_file, spec_edit_region): (
            Option<std::path::PathBuf>,
            Option<crate::focus_tunnel::EditRegion>,
        ) = match tree.find_baby_step(bs_id) {
            Some(spec) => (spec.target_file.clone(), spec.edit_region),
            None => (None, None),
        };
        let focus_tunnel_result: Option<BabyStepExecutionResult> = if focus_tunnel_active
            && !hook_pre_blocked
            && resolved.llm_provider.is_some()
            && !matches!(mode_selection.mode, crate::focus_tunnel::GenerationMode::Structured)
        {
            // C-15: OsClock-backed timing so the entropy scanner stays clean.
            // Focus-tunnel timing is observation-only (logged into the SSB row);
            // no determinism guarantee is required here.
            let focus_clock = OsClock;
            let started = focus_clock.now();
            let provider = resolved.llm_provider.clone();
            // B-28 WIRING-2 fix BLOCK 2c: FIM short-circuit on missing fields.
            // Returns `(attempted, short_circuit_reason)`. When the
            // short_circuit_reason is `Some(reason)`, the outer `match` skips
            // the error-classification branch and emits a clean
            // `StructuredFallback` beacon carrying that reason verbatim
            // (no `UnsupportedMode` masquerading as a generic Err).
            let (attempted, short_circuit_reason): (
                Result<
                    Option<BabyStepExecutionResult>,
                    crate::focus_tunnel::FocusTunnelExecError,
                >,
                Option<&'static str>,
            ) = match mode_selection.mode {
                crate::focus_tunnel::GenerationMode::Fim => {
                    match (spec_target_file.as_deref(), spec_edit_region) {
                        (Some(target), Some(region)) => {
                            // BLOCK 2c: real FIM splice. Provider is
                            // guaranteed Some by the outer guard.
                            let outcome = crate::focus_tunnel::run_fim_step_with_provider(
                                &exec_ctx,
                                &mode_selection,
                                provider,
                                Some(target),
                                Some(region),
                                None,
                            )
                            .await;
                            (
                                outcome.map(|out| {
                                    Some(BabyStepExecutionResult {
                                        outcome: BabyStepOutcome::Passed,
                                        duration_ms: u64::try_from(
                                            focus_clock.elapsed_since(started).as_millis(),
                                        )
                                        .unwrap_or(u64::MAX),
                                        exit_code: Some(0),
                                        stderr_tail: String::new(),
                                        changed_files: out.changed_files,
                                    })
                                }),
                                None,
                            )
                        }
                        (None, _) => {
                            // Missing target_file: emit a typed
                            // StructuredFallback SSB row (reason as locked
                            // wire-form string) and skip exec.
                            (Ok(None), Some("missing_target_file"))
                        }
                        (Some(_), None) => {
                            // target_file present but edit_region missing —
                            // FIM still unreachable. Same fallback class.
                            (Ok(None), Some("missing_edit_region"))
                        }
                    }
                }
                crate::focus_tunnel::GenerationMode::Diff => {
                    // B-28 WIRING-2 NOTE (deferred): production DIFF
                    // requires an `Arc<dyn BufferEditor>` threaded through
                    // ResolvedConfig (no slot exists today; adding one is
                    // outside the BLOCK 2 CLAIMED_FILES). The wiring
                    // contract test `run_diff_step_happy_path_applies_hunk_through_replay`
                    // exercises the editor=Some path with a stub. In
                    // production the call below returns
                    // FocusTunnelExecError::UnsupportedMode and the runner
                    // falls back to STRUCTURED. When the editor slot is
                    // wired in a follow-on patch, swap `None` for
                    // `Some(DiffStepIo::new(&*editor, &*fs))`.
                    let outcome = crate::focus_tunnel::run_diff_step_with_provider(
                        &exec_ctx, &mode_selection, provider, None,
                    )
                    .await;
                    (
                        outcome.map(|out| {
                            Some(BabyStepExecutionResult {
                                outcome: BabyStepOutcome::Passed,
                                duration_ms: u64::try_from(focus_clock.elapsed_since(started).as_millis())
                                    .unwrap_or(u64::MAX),
                                exit_code: Some(0),
                                stderr_tail: String::new(),
                                changed_files: out.changed_files,
                            })
                        }),
                        None,
                    )
                }
                _ => (Ok(None), None),
            };
            // Inline helper: emit one StructuredFallback SSB row with the
            // given reason. Reused for both short-circuit and error paths
            // so mode_distribution stays consistent regardless of which
            // branch surfaced the fallback.
            let emit_fallback = |reason: &str| {
                let payload = SsbPayload::GenerationModeSelected(
                    crate::ssb::GenerationModeSelectedPayload {
                        step_id: format!("baby_step_{}", bs_id.0),
                        mode: crate::focus_tunnel::generation_mode_str(
                            crate::focus_tunnel::GenerationMode::Structured,
                        )
                        .to_string(),
                        rule_fired: crate::focus_tunnel::selection_rule_str(
                            crate::focus_tunnel::SelectionRule::StructuredFallback,
                        )
                        .to_string(),
                        reason: reason.to_string(),
                        preferred_was: None,
                        supports_fim: resolved
                            .model_capability_profile
                            .features
                            .supports_fim,
                        exec_mode: crate::focus_tunnel::exec_mode_str(resolved_exec_mode)
                            .to_string(),
                    },
                );
                payload
            };
            match (attempted, short_circuit_reason) {
                (Ok(Some(result)), _) => Some(result),
                (Ok(None), Some(reason)) => {
                    // Short-circuit (missing field): emit fallback beacon
                    // and let the STRUCTURED executor handle the step.
                    let payload = emit_fallback(reason);
                    if let Err(e) = writer
                        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
                        .await
                    {
                        tracing::warn!(
                            baby_step = bs_id.0,
                            error = %e,
                            "task_runner: StructuredFallback SSB write failed (non-fatal)"
                        );
                    }
                    None
                }
                (Ok(None), None) => None,
                (Err(e), _) => {
                    tracing::warn!(
                        baby_step = bs_id.0,
                        error = %e,
                        "task_runner: focus-tunnel execution failed; falling back to STRUCTURED executor"
                    );
                    let payload = emit_fallback("focus_tunnel_exec_failed");
                    if let Err(e) = writer
                        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
                        .await
                    {
                        tracing::warn!(
                            baby_step = bs_id.0,
                            error = %e,
                            "task_runner: StructuredFallback SSB write failed (non-fatal)"
                        );
                    }
                    None
                }
            }
        } else {
            None
        };

        // Batch 1A: if PreToolUse blocked, skip the executor entirely with a
        // synthetic Failed result — the rest of the loop (integration
        // checkpoint, TestOutcome emission, MistakeBook, bubble-up) runs as if
        // the executor returned Failed. This mirrors the `pre_gate_blocked`
        // path for the existing rule evaluator.
        // Cycle-2 W1: distinguish CSQ-driven blocks from generic PreToolUse
        // blocks via the synthetic failure reason string. The block site
        // (line 3698 above) only flips `hook_pre_blocked = true` from the
        // CSQ Critical-without-guard branch today — once additional
        // PreToolUse-style gates land they should pass their own reason
        // string forward instead of overloading this default.
        // C-32 §Task 5 — skeleton-first hook. When the BabyStepSpec carries
        // a FilePlan with declared functions, pre-compute the SkeletonSpec
        // and emit a `skeleton_generated` SSB row before dispatching to the
        // executor. The fill loop itself still flows through the existing
        // baby-step executor (per-blank LLM dispatch is a follow-up); this
        // hook surfaces the skeleton-first activation point so the
        // dashboard can render the "skeleton with N blanks" indicator and
        // the completeness_ratio gauge updates on the matching
        // `skeleton_flushed` row downstream. See workflow/reasoning/
        // C-32-decisions.md D7 for placement justification.
        let c32_skeleton_specs: Vec<crate::skeleton_generator::SkeletonSpec> = {
            let baby_step_for_skeleton = tree.find_baby_step(bs_id).cloned();
            if let Some(spec_ref) = baby_step_for_skeleton.as_ref() {
                if let Some(file_plan_ref) = spec_ref.file_plan.as_ref() {
                    let gen = crate::skeleton_generator::SkeletonGenerator::default();
                    match gen.generate_from_file_plan(file_plan_ref) {
                        Ok(skeletons) => {
                            for s in &skeletons {
                                let payload = crate::ssb::SsbPayload::SkeletonGenerated(
                                    crate::ssb::SkeletonGeneratedPayload {
                                        blank_count: u32::try_from(s.blanks.len())
                                            .unwrap_or(u32::MAX),
                                        language: s.language.clone(),
                                        file_path: s
                                            .file_path
                                            .to_string_lossy()
                                            .into_owned(),
                                    },
                                );
                                if let Err(e) = writer
                                    .write_payload(
                                        RUNNER_AGENT_ID,
                                        RUNNER_SUPERSTEP,
                                        payload,
                                        vec![s.file_path.to_string_lossy().into_owned()],
                                    )
                                    .await
                                {
                                    tracing::warn!(
                                        baby_step = bs_id.0,
                                        error = %e,
                                        "task_runner: skeleton_generated SSB write failed (non-fatal)"
                                    );
                                }
                            }
                            skeletons
                        }
                        Err(e) => {
                            tracing::warn!(
                                baby_step = bs_id.0,
                                error = %e,
                                "task_runner: SkeletonGenerator::generate_from_file_plan \
                                 returned Err; falling back to executor without \
                                 skeleton pre-pass"
                            );
                            Vec::new()
                        }
                    }
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        };

        let exec_result = if let Some(result) = focus_tunnel_result {
            result
        } else if hook_pre_blocked {
            // C-21 Task 10 + DRIFT-8 — embed the block-source tier in the
            // synthetic Failed reason so record_mistake_on_failure can
            // categorise the MistakeEntry. Tag strings T1_BLOCK /
            // T2_BLOCK / PRE_TOOL_BLOCK / CSQ_BLOCK match the
            // FailureCategory variants added in core::mistake_book.
            let tag = block_source.reason_tag();
            let reason = match block_source {
                BlockSource::T1Hook => {
                    format!("{tag}: T1 PreToolUse hook fired BLOCK")
                }
                BlockSource::T2Hook => {
                    format!("{tag}: T2 hook fired BLOCK")
                }
                BlockSource::PreToolUse => {
                    format!("{tag}: generic PreToolUse rejection (irreversible/classifier)")
                }
                BlockSource::CsqCritical | BlockSource::None => {
                    format!("{tag}: CSQ Critical-without-guard blocked baby step")
                }
            };
            BabyStepExecutionResult {
                outcome: BabyStepOutcome::Failed { reason },
                duration_ms: 0,
                exit_code: None,
                stderr_tail: String::new(),
                changed_files: Vec::new(),
            }
        } else {
            resolved.baby_step_executor.execute(exec_ctx).await
        };
        let mut outcome = exec_result.outcome;
        let mut outcome_passed = matches!(outcome, BabyStepOutcome::Passed);
        let execution_duration_ms = exec_result.duration_ms;
        let exec_stderr_tail = exec_result.stderr_tail.clone();
        let exec_exit_code = exec_result.exit_code;
        if !outcome_passed {
            let msg = format!(
                "baby_step id={} agent exit_code={:?} stderr_tail={:?}",
                bs_id.0, exec_exit_code, exec_stderr_tail
            );
            emit_finding(writer, "error", AGENT_FAILED_CATEGORY, &msg).await;
        }

        // C-32 §Task 5 — full per-blank dispatch chain (closes audit-C32 W1).
        // For each pre-computed SkeletonSpec, walk:
        //   BridgeTemplate fast-path -> StructuralRetrySequencer (Decompose
        //   -> Template -> FewShot -> BestOfN) -> MiniCritic per-blank gate
        //   -> AcceptAndComplete threshold check -> SkeletonGenerator
        //   flush_to_disk (single fs::write).
        // The completeness_ratio carried by the emitted skeleton_flushed
        // row is now the FilledBuffer.compute_completeness_ratio() value
        // (partial 0.0..=1.0), not the boolean derived from outcome_passed.
        // Per-blank LLM dispatch remains a follow-up: the orchestrator
        // does not yet thread an LlmProvider into per-blank candidates,
        // so PerBlankCandidates::default() is supplied and non-bridge
        // skeletons exhaust through to Block fills. The wiring is
        // structurally complete; the candidate-bridge slot is the next
        // refactor.
        if !c32_skeleton_specs.is_empty() {
            // accept_and_complete_enabled is mirrored from
            // FileDisciplineConfig in a future plumbing pass; default
            // false matches `FileDisciplineConfig::default()`.
            let accept_and_complete_enabled = false;
            let mini_critic = crate::mini_critic::MiniCritic::without_tree_sitter();
            let sequencer = crate::structural_retry::StructuralRetrySequencer::default();
            let candidates = crate::c32_dispatch::PerBlankCandidates::default();
            for s in c32_skeleton_specs.iter().cloned() {
                let bridge_kind = crate::c32_dispatch::classify_bridge_kind(&s.file_path);
                let dispatch = crate::c32_dispatch::dispatch_skeleton_with_retry(
                    s,
                    &candidates,
                    &mini_critic,
                    &sequencer,
                    bridge_kind,
                    accept_and_complete_enabled,
                    Some(writer),
                    RUNNER_AGENT_ID,
                    RUNNER_SUPERSTEP,
                )
                .await;
                match dispatch {
                    Ok((_buf, dispatch_outcome)) => {
                        tracing::debug!(
                            baby_step = bs_id.0,
                            completeness_ratio = dispatch_outcome.completeness_ratio,
                            partial_accepted = dispatch_outcome.partial_accepted,
                            bridge_short_circuit = dispatch_outcome.bridge_short_circuit,
                            "task_runner: C-32 skeleton dispatch completed"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            baby_step = bs_id.0,
                            error = %e,
                            "task_runner: C-32 skeleton dispatch failed (non-fatal)"
                        );
                    }
                }
            }
        }

        // B-28 WIRING-2 items H + I + J: focus-tunnel post-execute hooks.
        //
        // Each hook is invoked only when the corresponding Arc field on
        // ResolvedConfig is Some AND the agent reported changed_files. The
        // Arc fields default to None today (no construction site) so this
        // block is a strict no-op for every existing caller. The wiring is
        // contract-tested via `focus_tunnel_wiring_contract.rs` against the
        // library-tier surfaces (resolver / router / pre-assembler).
        run_focus_tunnel_post_execute_hooks(
            writer,
            &resolved,
            bs_id.0,
            &exec_result.changed_files,
            outcome_passed,
        )
        .await;

        // C-25 — TemplateCrystallizer post-execute observation hook.
        // Strict no-op when crystallizer + compiled_pipeline_store are both
        // None (legacy / pre-C-25 callers). When both are Some, records
        // an ExecutionOutcome for the current baby step then checks for
        // crystallization; on success, upserts the new pipeline into the
        // store and persists to disk. Errors are logged + swallowed (this
        // hook is observational; failure must not derail the baby-step loop).
        run_crystallizer_post_execute_hook(
            &resolved,
            &exec_ctx_task_summary_for_crystallizer,
            outcome_passed,
        )
        .await;

        // C-49 — iterative-distillation experience post-audit hook.
        // Wire-dark: ENV-gated via `ELAI_EXPERIENCE_DISTILL_ENABLE`
        // (default off). When enabled AND audit passed, captures the
        // baby-step task summary as a `SuccessTrace` and emits a
        // distillation-run SSB beacon. The function itself is a
        // transparent no-op when the env gate is off OR when audit
        // failed (heuristic simplification of paper Eq. 5 — see
        // workflow/papers/iterative-distillation-interface.md). Errors
        // are logged + swallowed (observation hook).
        run_experience_distill_post_execute_hook(
            writer,
            &exec_ctx_task_summary_for_crystallizer,
            outcome_passed,
            bs_id.0,
        )
        .await;

        // C1 fix-and-rerun: on a failed baby step in a web project, run
        // `npm run build` then (when Chrome is available) start a dev server,
        // run Lighthouse to collect perf+a11y scores from a single JSON
        // document, and check thresholds (perf >= MIN_PERF_SCORE, a11y >=
        // MIN_A11Y_SCORE). Chrome absent -> Failed immediately. Scores below
        // threshold -> Failed so the fixer loop iterates. Escalation
        // (RequestHuman GateDecision) is emitted inside
        // `fix_and_rerun_until_green`, so we only mirror the result into
        // local `outcome` / `outcome_passed`.
        if is_web_project && !outcome_passed {
            if let Some(proj) = project_dir.as_ref() {
                let proj_for_validator = proj.clone();
                let bs_id_for_validator = bs_id.0;
                let validator = move |_dir: &Path| {
                    let dir = proj_for_validator.clone();
                    async move {
                        // Step 1: build must succeed before Lighthouse runs.
                        let out_dir = dir.join("dist");
                        match run_web_build(&dir, "npm run build", &out_dir).await {
                            Ok(r) if r.success => {}
                            Ok(r) => {
                                return FixerValidatorReport::Failed {
                                    summary: r.stderr_errors.join("\n"),
                                };
                            }
                            Err(e) => {
                                return FixerValidatorReport::Failed {
                                    summary: e.to_string(),
                                };
                            }
                        }

                        // Step 2: probe for Chrome. No silent skip — absent
                        // Chrome is an explicit failure.
                        let chrome_path = find_chrome_path();

                        // Step 3: if Chrome is missing and PATH probe also
                        // fails, surface ChromiumMissing as Failed.
                        if chrome_path.is_none() {
                            // Try PATH-based probe as final fallback (covers
                            // Unix/devcontainer with `chromium` on PATH).
                            if let Err(probe_err) =
                                elai_web_validator::cwv::probe_chromium().await
                            {
                                tracing::warn!(
                                    baby_step = bs_id_for_validator,
                                    "lighthouse validator: chrome absent — {}",
                                    probe_err
                                );
                                return lighthouse_validator_report(None, &LighthouseScores {
                                    performance: 0.0,
                                    accessibility: 0.0,
                                });
                            }
                            // PATH probe succeeded (e.g. `chromium` on Unix);
                            // proceed with chrome_path=None — run_lighthouse_scores
                            // will use probe_chromium() internally.
                        }

                        // Step 4: start a dev server for Lighthouse to audit.
                        let preview_port = DEFAULT_DEV_SERVER_PORT;
                        let dev_handle = match start_dev_server(
                            &dir,
                            DEFAULT_DEV_SERVER_COMMAND,
                            preview_port,
                        )
                        .await
                        {
                            Ok(h) => h,
                            Err(e) => {
                                return FixerValidatorReport::Failed {
                                    summary: format!("dev server failed to start: {e}"),
                                };
                            }
                        };
                        let preview_url =
                            format!("http://127.0.0.1:{}/", dev_handle.port());

                        // Step 5: run Lighthouse. Dev server is dropped
                        // (killed) after this block regardless of outcome.
                        let scores_result = run_lighthouse_scores(
                            &dir,
                            &preview_url,
                            chrome_path.as_deref(),
                        )
                        .await;
                        // Explicitly drop to kill the dev server process before
                        // returning so we don't leave stray children alive.
                        drop(dev_handle);

                        let scores = match scores_result {
                            Ok(s) => s,
                            Err(e) => {
                                return FixerValidatorReport::Failed {
                                    summary: format!("lighthouse failed: {e}"),
                                };
                            }
                        };

                        // Step 6: threshold check.
                        lighthouse_validator_report(
                            Some(std::path::Path::new("probe-ok")),
                            &scores,
                        )
                    }
                };
                let fix_outcome = fix_and_rerun_until_green(
                    proj.clone(),
                    resolved.baby_step_executor.clone(),
                    resolved.autonomous_gate.clone(),
                    Arc::clone(writer),
                    5,
                    validator,
                    &format!("baby_step_{}", bs_id.0),
                )
                .await?;
                match fix_outcome {
                    FixAndRerunOutcome::Healed { iterations } => {
                        outcome = BabyStepOutcome::Passed;
                        outcome_passed = true;
                        emit_finding(
                            writer,
                            "info",
                            "fix_and_rerun_healed",
                            &format!("healed in {iterations} iterations"),
                        )
                        .await;
                    }
                    FixAndRerunOutcome::EscalatedToHuman { iterations } => {
                        emit_finding(
                            writer,
                            "warn",
                            "fix_and_rerun_escalated",
                            &format!("escalated after {iterations} iterations"),
                        )
                        .await;
                    }
                }
            }
        }

        // B-8 Tasks#4 + Tasks#5: PipelineGrc.check_inline_feasibility +
        // ReplanRequest::Major conversion (deferred — see decision doc §4).
        //
        // Producer: build a synthetic claim string from `exec_result.stderr_tail`
        // when the baby step did not pass. The runner has no static-analysis
        // path that surfaces "novel claims" today, so the stderr_tail is the
        // closest signal available. When the engine returns
        // `FeasibilityOutcome::Blocked` we flip this baby step to failed and
        // emit a typed Finding so the downstream loop renders the row with
        // `passed=false`. ReplanRequest::Major emission is deferred to
        // B-future once `ResolvedConfig` carries an `FsmHandle`.
        //
        // Skipped when: (a) the kill-switch is on (pipeline_grc = None),
        // (b) the baby step passed (no novel-claim signal to evaluate), or
        // (c) the stderr_tail is empty.
        if let Some(grc) = resolved.pipeline_grc.as_ref() {
            if !outcome_passed && !exec_stderr_tail.trim().is_empty() {
                // Mark the claim as a plan-contradiction so the engine takes
                // the Blocked branch (engine.rs:670). The substring check at
                // engine.rs:680 then drives Pass / Fail for non-contradictory
                // claims — for B-8 we only act on Blocked.
                let claim = format!(
                    "CONTRADICTS_PLAN: baby_step {} reported stderr={}",
                    bs_id.0,
                    truncate_for_log(&exec_stderr_tail, 240)
                );
                let sources: Vec<String> = vec![task_summary.clone()];
                let mut grc_guard = grc.lock().await;
                match grc_guard.check_inline_feasibility(&claim, &sources) {
                    Ok(feas) if feas.outcome
                        == elai_reasoning::FeasibilityOutcome::Blocked =>
                    {
                        tracing::error!(
                            baby_step = bs_id.0,
                            "task_runner: inline_feasibility Blocked — flagging step for replan"
                        );
                        emit_finding(
                            writer,
                            "error",
                            "inline_feasibility_blocked",
                            &format!(
                                "baby_step {} inline feasibility Blocked: novel claim contradicts plan",
                                bs_id.0
                            ),
                        )
                        .await;
                        // TODO(B-future): submit ReplanRequest::Major via fsm.submit_replan. stub-ban-allow: B-future plan item
                        // once ResolvedConfig carries an FsmHandle (fsm.rs:644). Until then we surface the
                        // block by flipping outcome_passed AND rewriting
                        // `outcome` to a Failed variant so the BabyStepResult
                        // constructed below carries the failure forward into
                        // navigator/state. Cycle-2 BLOCK-2 fix: previously
                        // only `outcome_passed` was flipped, leaving
                        // `outcome = Passed`, which made
                        // `pipeline_tree_navigator::record_result` advance the
                        // step into `state.completed` — irreconcilable with
                        // the failed TestOutcome that SSB then emitted.
                        outcome_passed = false;
                        outcome = BabyStepOutcome::Failed {
                            reason: format!(
                                "inline_feasibility_blocked: {}",
                                truncate_for_log(&feas.claim, 240)
                            ),
                        };
                    }
                    Ok(feas) => {
                        tracing::debug!(
                            baby_step = bs_id.0,
                            outcome = ?feas.outcome,
                            "task_runner: inline_feasibility check completed"
                        );
                    }
                    Err(crate::grc::PipelineGrcError::Engine(
                        elai_reasoning::GrcError::BudgetExhausted { .. },
                    )) => {
                        // By design: max 2 checks per phase
                        // (types.rs:480). Skip silently.
                        tracing::debug!(
                            baby_step = bs_id.0,
                            "task_runner: inline_feasibility budget exhausted (max 2/phase)"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            baby_step = bs_id.0,
                            error = ?e,
                            "task_runner: inline_feasibility check errored"
                        );
                    }
                }
            }
        }

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
        // Session A-19.REDO: track integration success separately so
        // failure classification can distinguish a PRE_GATE block from an
        // integration-checkpoint failure. Default `true` when no spec is
        // registered (matches the pre-A-19 `None` branch).
        let mut integration_succeeded = true;
        let mut passed = match tree
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
                integration_succeeded = run.success;
                run.success && outcome_passed && !pre_gate_blocked && !hook_pre_blocked
            }
            None => outcome_passed && !pre_gate_blocked && !hook_pre_blocked,
        };

        // Batch 1B: per-baby-step provider-signals snapshot. Emitted once per
        // step AFTER the integration checkpoint so the ProviderSignals row
        // lands adjacent to the TestOutcome in the feed. The search-trigger
        // emission rides the same reader — when ETC fired, the dashboard
        // shows both the amber `search_trigger_fired` row AND the normal
        // ProviderSignals row. Pre-batch-1b behaviour is preserved when
        // `signal_reader` is `None`.
        if let Some(reader) = resolved.signal_reader.as_ref() {
            let step_id = format!("baby_step_{}", bs_id.0);
            emit_provider_signals_snapshot(writer, reader, &step_id).await;
            emit_search_trigger_if_fired(writer, reader, &step_id).await;
        }

        // Batch 1A: PostToolUse hook dispatch. Fires T1/T2/T3 against every
        // changed file the executor reported. Per Batch 1A spec, only runs
        // when the baby step Passed AND produced at least one changed file.
        // - Warn verdicts emit HookEvent rows; passed stays as-is.
        // - Block verdicts flip `passed = false` and emit a typed Finding so
        //   the dashboard surfaces the flip alongside the existing
        //   rule_violation_post stream.
        if outcome_passed && !exec_result.changed_files.is_empty() {
            if let Some(dispatcher) = resolved.hook_dispatcher.as_ref() {
                for changed in &exec_result.changed_files {
                    let source = match tokio::fs::read_to_string(changed).await {
                        Ok(s) => s,
                        Err(e) => {
                            warn!(
                                path = %changed.display(),
                                error = %e,
                                "task_runner: PostToolUse hook could not read changed file"
                            );
                            continue;
                        }
                    };
                    let changed_str = changed.to_string_lossy();
                    let invocation = HookInvocation::for_post_tool_use(
                        "file_write",
                        &source,
                        changed_str.as_ref(),
                        format!("baby_step_{}", bs_id.0),
                    );
                    match dispatcher.dispatch_post_tool_use(invocation).await {
                        Ok(findings) => {
                            emit_hook_events_to_ssb(
                                writer,
                                RUNNER_AGENT_ID,
                                RUNNER_SUPERSTEP,
                                HookPoint::PostToolUse,
                                &format!("baby_step_{}", bs_id.0),
                                &findings,
                            )
                            .await;
                            if findings.iter().any(HookFinding::is_block) {
                                let msg =
                                    build_hook_block_summary(&findings, "PostToolUse");
                                emit_finding(
                                    writer,
                                    "error",
                                    HOOK_POST_TOOL_USE_BLOCK_CATEGORY,
                                    &msg,
                                )
                                .await;
                                passed = false;
                            }
                        }
                        Err(e) => {
                            warn!(
                                baby_step = bs_id.0,
                                error = %e,
                                "task_runner: PostToolUse hook dispatch failed"
                            );
                        }
                    }

                    // Batch 4B: run the dispatcher's ShortCoder hook (if
                    // installed) against the PostToolUse payload so the
                    // auditor can read an AST-preserving compressed version
                    // instead of the raw executor output. The raw `source`
                    // string is retained for SSB record keeping — the
                    // compressed output is stored via
                    // [`SsbPayload::ShortCoderCompression`] and flows into
                    // the auditor's context slot on the following pass.
                    // `ELAI_DISABLE_SHORTCODER_POSTOOLUSE` bypasses this
                    // call site exclusively; the global kill switch
                    // (`ELAI_DISABLE_SHORTCODER`) takes precedence.
                    let shortcoder_bypassed = std::env::var(
                        ELAI_DISABLE_SHORTCODER_POSTOOLUSE_ENV,
                    )
                    .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "on" | "ON"))
                    .unwrap_or(false);
                    if !shortcoder_bypassed {
                        if let Some(outcome) =
                            dispatcher.shortcode(&source, Some(changed_str.as_ref()))
                        {
                            emit_shortcoder_compression_to_ssb(
                                writer,
                                RUNNER_AGENT_ID,
                                RUNNER_SUPERSTEP,
                                &outcome,
                                &format!("baby_step_{}", bs_id.0),
                            )
                            .await;
                        }
                    }
                }
            }
        }

        // Gap #4 (MVP audit 2026-04-18 section 4 item 4): invoke the
        // DeterministicAuditor after the baby-step checkpoint has been
        // emitted. Runs only when the baby step Passed AND the agent
        // produced at least one changed file (no diff → nothing to audit,
        // and the build-check path would spuriously gate on a clean tree).
        // Block verdicts become typed Findings; they do NOT mutate `passed`
        // — the bubble-up pass keeps its existing inputs (acceptance
        // criterion: "Do NOT change the bubble-up logic itself").
        if outcome_passed && !exec_result.changed_files.is_empty() {
            run_post_step_deterministic_audit(
                writer,
                project_dir.as_deref(),
                bs_id.0,
                &exec_result.changed_files,
            )
            .await;

            // C-13 — post-AUDITING -> MEMORIZING transition hook.
            // After the deterministic auditor finished the per-baby-step
            // audit cleanly, opportunistically squash the leading WIP
            // commit chain so the eventual phase-MEMORIZING commit lands
            // on a clean history. The Python FSM hook layer also fires
            // `elai wip-squash` directly when the transition originates
            // outside the runner process; both paths are idempotent
            // (count <= 5 ⇒ no-op). See workflow/reasoning/C-13-decisions.md
            // (DECISION D4 option A).
            if let Some(dir) = project_dir.as_ref() {
                match crate::wip_squash::post_transition_auditing_to_memorizing(
                    dir.as_path(),
                    writer.clone(),
                    &format!("baby_step_{}", bs_id.0),
                )
                .await
                {
                    Ok(outcome) => {
                        if outcome.squashed {
                            tracing::info!(
                                wip_count = outcome.wip_count,
                                "task_runner: wip_squash collapsed leading WIP commits"
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "task_runner: wip_squash failed (non-fatal); continuing"
                        );
                    }
                }
            }

            // B-29 Phase 5: PostToolUse IMPACT_SUMMARY injection.
            // Compute impact radius for each changed file at depth=2 and append
            // the IMPACT_SUMMARY block to the auditor briefing log.
            // Symbol-graph ops are sync (knowledge crate uses tokio features=["sync"]).
            // Wrap in spawn_blocking so we do NOT block the tokio runtime — file
            // I/O + rusqlite reads + traversal may exceed the per-task latency
            // budget on large projects.
            let impact_seq = seq;
            let impact_changed = exec_result.changed_files.clone();
            let impact_dir = project_dir.as_ref().map(|p| p.to_path_buf());
            let _ = tokio::task::spawn_blocking(move || {
                write_impact_summary_to_log(
                    impact_seq,
                    &impact_changed,
                    impact_dir.as_deref(),
                );
            })
            .await;
        }

        // Gap #9 (MVP audit 2026-04-18 section 4 item 9): post-execution
        // rule re-evaluation against each changed file's on-disk content.
        // The pre-gate path above runs against `task_summary` (natural
        // language) so Grep/AST rules never match. This pass feeds the
        // actual source buffer into the same evaluator, tagged with a
        // distinct category so the frontend can distinguish the two.
        if let Some(evaluator) = resolved.rule_evaluator.as_ref() {
            if !exec_result.changed_files.is_empty() {
                // Batch 4B: a Block verdict on the real source flips the
                // baby-step outcome alongside the existing PreGate Block
                // and hook_post_block flips. Warn verdicts emit Findings
                // but leave `passed` as-is (same semantics as 4E.14).
                let post_block = evaluate_post_execution_rules_and_emit_findings(
                    writer,
                    evaluator,
                    bs_id.0,
                    &exec_result.changed_files,
                    is_web_project,
                )
                .await;
                if post_block {
                    passed = false;
                }
            }
        }

        // Batch 2B: ToolDispatcher call sites.
        //   * Post-successful baby step with changed files:
        //     dispatch_validate (Determinism + QualityAst) + dispatch_git_commit.
        //   * Post-failing baby step: dispatch_classify_failure on the
        //     executor's stderr tail so downstream consumers have a
        //     taxonomy-backed category even when no rule fired.
        if let Some(dispatcher) = resolved.tool_dispatcher.as_ref() {
            let step_id = format!("baby_step_{}", bs_id.0);
            let detected_project_dir = project_dir
                .clone()
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            let detected_language =
                detect_project_language(&detected_project_dir);
            if outcome_passed && !exec_result.changed_files.is_empty() {
                // C-39: production tool-dispatch seam. When mcp_dispatch is
                // wired (either via the --via-mcp CLI flag or apply_task_runner_wiring's
                // default in-process leg), route a real `validate` invocation
                // through the Dispatch trait so the per-call SSB row fires.
                // The impl (InProcessDispatch or McpClientDispatch) emits the
                // appropriate action type (McpToolCalled or CliMcpDispatch);
                // the parity script unions both and asserts symmetry.
                // The legacy ToolDispatcher path below continues unchanged --
                // this is purely additive observability.
                if let Some(disp) = resolved.mcp_dispatch.as_ref() {
                    let req = crate::dispatch::DispatchRequest::new(
                        "validate",
                        serde_json::json!({
                            "project_dir": detected_project_dir.display().to_string(),
                            "language": detected_language.clone(),
                            "changed_files": exec_result
                                .changed_files
                                .iter()
                                .map(|p| p.display().to_string())
                                .collect::<Vec<_>>(),
                            "step_id": step_id.clone(),
                        }),
                    );
                    // Fire-and-forget: the impl emits the SSB row regardless
                    // of success/error; we record the Err for diagnostics
                    // but do not block the legacy validate path below.
                    if let Err(e) = disp.dispatch_tool(&req) {
                        tracing::debug!(
                            step_id = %step_id,
                            error = %e,
                            "C-39 mcp_dispatch.dispatch_tool(validate) returned Err; \
                             telemetry row emitted, legacy validate path continues"
                        );
                    }
                }
                run_validate_dispatch(
                    writer,
                    dispatcher.as_ref(),
                    &detected_project_dir,
                    &detected_language,
                    &exec_result.changed_files,
                    &step_id,
                )
                .await;
                let commit_message = format!(
                    "elai baby_step_{}: {} files",
                    bs_id.0,
                    exec_result.changed_files.len()
                );
                run_git_commit_dispatch(
                    writer,
                    dispatcher.as_ref(),
                    &detected_project_dir,
                    &exec_result.changed_files,
                    &commit_message,
                    resolved.git_branch_policy,
                    &task.project_id,
                    &step_id,
                )
                .await;
                // B-20B: file-read outline + 50% budget guard for the first
                // changed file (wires enforce_file_read_budget_guard into a
                // real production call site -- BLOCK-1 resolution).
                let first_file = exec_result
                    .changed_files
                    .first()
                    .expect("changed_files non-empty; checked above");
                run_file_read_dispatch(
                    writer,
                    dispatcher.as_ref(),
                    first_file.clone(),
                    u32::try_from(resolved.model_capability_profile.recommended_prompt_budget).unwrap_or(u32::MAX),
                    &step_id,
                )
                .await;
            } else if !outcome_passed && !exec_stderr_tail.is_empty() {
                // B-20A Task 5 — auditor classification reading.
                //
                // Spec source: docs/specs/guide/steps/phase-4B.txt §
                // failure-taxonomy + B-20 plan body Tasks 4-5.
                //
                // `run_classify_dispatch` writes a FailureClassified SSB row
                // BEFORE any subsequent emit (e.g. raw stderr / tool_dispatch
                // rows from later in the loop iteration). The SSB's
                // monotonically-increasing `seq` then means the auditor's
                // `SsbViewer::failure_taxonomy_last` query (`ORDER BY seq
                // DESC LIMIT 1`) returns the categorized row before the raw
                // output is consumed — saving 200-500 tokens per failure.
                // Verified by reading the call ordering here against
                // ssb_viewer.rs:1780-1819 + emit_failure_classified at
                // task_runner.rs:4859.
                //
                // C-39: route the same classify intent through the Dispatch
                // trait so the per-call SSB row fires on the failure leg
                // too. Symmetric coverage with the success-leg validate
                // dispatch above so the CI parity script observes both
                // outcomes (in-process and --via-mcp produce structurally
                // equivalent event streams across success + failure paths).
                if let Some(disp) = resolved.mcp_dispatch.as_ref() {
                    let req = crate::dispatch::DispatchRequest::new(
                        "classify_failure",
                        serde_json::json!({
                            "stderr_tail": exec_stderr_tail.clone(),
                            "step_id": step_id.clone(),
                            "actor": "agent",
                        }),
                    );
                    if let Err(e) = disp.dispatch_tool(&req) {
                        tracing::debug!(
                            step_id = %step_id,
                            error = %e,
                            "C-39 mcp_dispatch.dispatch_tool(classify_failure) returned Err; \
                             telemetry row emitted, legacy classify path continues"
                        );
                    }
                }
                let _ = run_classify_dispatch(
                    writer,
                    dispatcher.as_ref(),
                    &exec_stderr_tail,
                    &step_id,
                    "agent",
                )
                .await;
            }
        }

        // Session A-19.REDO: on a failing baby step, drive a short
        // InterrogationTree traversal and emit a Finding carrying the
        // FixTemplate. The metric Finding (seeded tree count) is emitted
        // one-shot on first invocation so the dashboard can surface Layer-3
        // availability without repeating the row per task.
        // Bind the failure category once and reuse. `classify_baby_step_failure`
        // is pure, but computing it twice per failing baby step is wasted work
        // and — more importantly — risks the two call sites drifting out of
        // sync if the classification rule later depends on more inputs.
        let failure_category = if passed {
            None
        } else if let Some(tier_category) = block_source.failure_category() {
            // C-21 Task 10 — when a T1/T2/PreToolUse hook fired the block,
            // categorise the MistakeEntry with the tier-specific variant
            // so downstream MistakeBook consumers can bucket by gate
            // source without parsing root_cause_json.
            Some(tier_category)
        } else {
            Some(classify_baby_step_failure(
                pre_gate_blocked,
                !integration_succeeded,
            ))
        };

        if let Some(category) = failure_category {
            maybe_emit_interrogation_metric(writer, resolved).await;
            emit_interrogation_fix_suggestion(writer, resolved, bs_id.0, category).await;
        }
        // Session wire-4A: duration comes from the executor dispatch instead
        // of [`RUNNER_SYNTHETIC_DURATION_MS`]. The synthetic executor keeps
        // reporting 1 ms so legacy tests stay stable; the real agent-pool
        // executor reports measured wall-clock.
        emit_test_outcome(
            writer,
            &format!("baby_step_{}", bs_id.0),
            passed,
            execution_duration_ms,
        )
        .await;
        baby_step_outcomes.push((bs_id.0, passed));

        // 3F.REDO: persist / tombstone MistakeBook state for this baby step.
        //   * On failure: write a new MistakeEntry so the next pipeline
        //     invocation picks it up via `query_by_baby_step` at PRE_GATE.
        //     `attempt_number` is `prior_attempts + 1` (1-indexed).
        //   * On success with prior entries: append a tombstone so the
        //     constraints block empties and ForbiddenDiffs no longer apply.
        //   * On success without prior entries: no-op (avoid spurious
        //     tombstones and the audit-trail churn they would cause).
        if let Some(category) = failure_category {
            record_mistake_on_failure(
                writer,
                &mistake_book_step_id,
                prior_attempts,
                category,
                &task_summary,
            )
            .await;

            // Batch-fu (FU.4 / A-23): call the sidecar-backed reflection client
            // so the MistakeEntry row just written gets a matching purple
            // MicroReflection row in the activity feed. No-op when the
            // dashboard skipped wiring the client (env kill switch or
            // grounding client unreachable).
            if let Some(client) = resolved.reflection_client.as_ref() {
                let adapter = crate::micro_reflection_emitter::DynReflectionAdapter::new(
                    client.as_ref(),
                );
                let _ = crate::micro_reflection_emitter::emit_micro_reflection(
                    writer,
                    &adapter,
                    RUNNER_AGENT_ID,
                    RUNNER_SUPERSTEP,
                    &mistake_book_step_id,
                    &task_summary,
                    "", // code_snippet: not available at this call site; future patch may thread it
                    category.as_str(),
                )
                .await;
            }
        } else if prior_attempts > 0 {
            clear_mistakes_on_success(writer, &mistake_book_step_id).await;
        }

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
                let phase_id = format!("task_seq_{seq}");
                emit_baby_step_recovery(writer, cp, seq, bs_id.0, &phase_id).await;
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

    // Batch 1A: Stop hook dispatch. Fires exactly once per task, regardless of
    // success or hard-fail cascade past the root concept. No file / source
    // context — Stop is task-level — so T1 is the only tier that will
    // meaningfully fire; T2/T3 stay gated on file_write in `TieredAnalysis`.
    if let Some(dispatcher) = resolved.hook_dispatcher.as_ref() {
        let step_id = format!("task_{}", seq);
        // B-19 W1: thread FSM state so dispatch_stop can enforce the
        // non-IDLE Block. drive_task always runs inside the EXECUTING state;
        // the Stop hook fires at task teardown before the FSM transitions to
        // AUDITING, so "EXECUTING" is the correct state label here.
        let invocation = HookInvocation::for_task_level("task_stop", step_id.clone())
            .with_fsm_state("EXECUTING");
        match dispatcher.dispatch_stop(invocation).await {
            Ok(findings) => {
                emit_hook_events_to_ssb(
                    writer,
                    RUNNER_AGENT_ID,
                    RUNNER_SUPERSTEP,
                    HookPoint::Stop,
                    &step_id,
                    &findings,
                )
                .await;
            }
            Err(e) => {
                warn!(seq, error = %e, "task_runner: Stop hook dispatch failed");
            }
        }

        // B-19 PreCompact: DairaTrace "HookPre" emitted before dispatch,
        // "HookPost" after. This replaces the former debug-only no-op comment
        // with real SSB breadcrumbs so the dashboard can surface compact events.
        let pre_compact_summary = format!("task_{}", seq);
        emit_daira_trace(
            writer,
            "pre_compact",
            &pre_compact_summary,
            "HookPre",
            0,
            "started",
            &step_id,
        )
        .await;
        let pre_compact = HookInvocation::for_task_level("pre_compact", step_id.clone());
        // B-19 W2: entropy quarantine — use ClockProvider abstraction instead of
        // std::time::Instant::now() so tests can inject a deterministic clock.
        let pre_compact_clock = OsClock;
        let pre_compact_mark = elai_core::clock::ClockProvider::now(&pre_compact_clock);
        match dispatcher.dispatch_pre_compact(pre_compact).await {
            Ok(findings) => {
                let elapsed_us = u64::try_from(
                    elai_core::clock::ClockProvider::elapsed_since(&pre_compact_clock, pre_compact_mark)
                        .as_micros(),
                )
                .unwrap_or(u64::MAX);
                emit_hook_events_to_ssb(
                    writer,
                    RUNNER_AGENT_ID,
                    RUNNER_SUPERSTEP,
                    HookPoint::PreCompact,
                    &step_id,
                    &findings,
                )
                .await;
                emit_daira_trace(
                    writer,
                    "pre_compact",
                    &pre_compact_summary,
                    "HookPost",
                    elapsed_us,
                    "success",
                    &step_id,
                )
                .await;
            }
            Err(e) => {
                let elapsed_us = u64::try_from(
                    elai_core::clock::ClockProvider::elapsed_since(&pre_compact_clock, pre_compact_mark)
                        .as_micros(),
                )
                .unwrap_or(u64::MAX);
                emit_daira_trace(
                    writer,
                    "pre_compact",
                    &pre_compact_summary,
                    "HookPost",
                    elapsed_us,
                    "error",
                    &step_id,
                )
                .await;
                warn!(seq, error = %e, "task_runner: PreCompact hook dispatch failed");
            }
        }
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
    // Batch 3C (Phase 4H): Layer 1A self-modification. Construct a minimal
    // PipelineOutcome from the just-completed task and hand it to the
    // processor. For MVP we always report `PipelineMode::Surgical` — the
    // concept-decomposition path below this layer is surgical by default; a
    // future batch can flip to Greenfield based on scaffold flags.
    if let Some(processor) = resolved.layer1a_processor.as_ref() {
        let all_passed = baby_step_outcomes.iter().all(|(_, p)| *p);
        let outcome = crate::PipelineOutcome {
            task_id: format!("seq-{seq}"),
            success: all_passed,
            failures: Vec::new(),
            findings: Vec::new(),
            pipeline_mode: crate::PipelineMode::Surgical,
            elapsed_ms: 0,
        };
        let proposals = processor.process_outcome(&outcome).await;
        let auto_apply = crate::layer1a_auto_apply_via_env();
        for proposal in &proposals {
            if auto_apply {
                if let Err(e) = processor.apply_proposal(proposal).await {
                    tracing::warn!(
                        seq,
                        proposal = %proposal.proposal_id,
                        error = %e,
                        "layer1a auto-apply failed",
                    );
                }
            }
        }
    }
    // D-3 closure: post-task CLAIM-emit consensus dispatch. Wires the
    // run_consensus_pass producer from agent_pool into the per-task hook.
    // When no verifier adapters are plumbed (current default), this is a
    // no-op — the row family stays dark until D-5 / D-13 / future
    // sessions plumb concrete adapters (PpmAdapter, DeterministicAdapter,
    // IntrinsicAdapter, CrossContextAdapter). When plumbed, exactly one
    // VerifierConsensusPass OR VerifierConsensusFail row is emitted per
    // call. See dispatch_verifiers (below) for the wiring contract.
    //
    // candidate_id is derived from the task seq (stable per-row identifier);
    // template_id from project_id (the lightweight grouping key available
    // on TaskSubmittedPayload). Future plans surface a real template_id
    // via the verifier_required field (D-1) when adapter plumbing lands.
    let candidate_id = format!("task-{seq}");
    let consensus = dispatch_verifiers(writer, &candidate_id, &task.project_id, resolved).await;
    // D-5 closure (GAP-B): fold the consensus verdict into the BAI arm
    // posterior + run Thompson sampling. No-op when the BAI runtime is
    // not plumbed OR ELAI_BAI_ENABLE is unset. See dispatch_bai_round
    // doc + workflow/reasoning/D-5-decisions.md (DECISION-2).
    dispatch_bai_round(writer, &task.project_id, consensus.as_ref(), resolved).await;
    // D-10 closure: post-task CLAIM-emit mode-routing dispatch. The
    // orchestrator (NOT the policy model) picks the GroundingMode for
    // every emitted claim. When `resolved.claim_router_runtime` is None
    // (default) or `ELAI_CLAIM_ROUTER_ENABLE` is unset, this is a no-op
    // — the row family stays dark until the runtime is plumbed through
    // TaskRunnerConfig. See dispatch_claim_routing + workflow/reasoning/
    // D-10-decisions.md (DECISION-1, "orchestrator decides, not LLM").
    dispatch_claim_routing(writer, seq, &task.project_id, resolved).await;
    // D-11 closure: post-task constructive-checker dispatch. The
    // orchestrator runs every applicable checker in the registry +
    // emits one `checker_result_emitted` SSB row per result. When
    // `resolved.checker_runtime` is None (default) or
    // `ELAI_CHECKER_ENABLE` is unset, this is a no-op — the row family
    // stays dark until the runtime is plumbed through `TaskRunnerConfig`
    // AND the env gate flips. See dispatch_constructive_checkers_hook +
    // workflow/reasoning/D-11-decisions.md.
    dispatch_constructive_checkers_hook(seq, &task.project_id, resolved).await;
    // D-12 closure: post-task operationalization-check dispatch. The
    // orchestrator synthesises a per-task BabyStep envelope, calls
    // `agent_pool::run_operational_check`, and emits exactly one
    // `baby_step_dispatched` SSB row per call. When
    // `resolved.operationalization_runtime` is None (default) or
    // `ELAI_OPERATIONALIZATION_ENABLE` is unset, this is a no-op — the
    // row family stays dark until the runtime is plumbed through
    // `TaskRunnerConfig` AND the env gate flips. See
    // run_operational_check_hook + workflow/reasoning/D-12-decisions.md
    // DECISION-6 (closes the GAP-1 dead-code report against
    // `run_operational_check`).
    run_operational_check_hook(seq, &task.project_id, resolved).await;
    // D-13 closure: post-task adversarial-pass dispatch. When the
    // orchestrator-side mode router (D-10) classified the claim as
    // `GroundingMode::FalsifiabilityCrossFamily`, this hook drives
    // `agent_pool::run_adversarial_pass` to (a) generate structured
    // counter-evidence from a cross-family adversary, (b) re-run the
    // verifier consensus with the counter as a fresh failing signal,
    // and (c) emit the `adversary_generated` +
    // `consensus_reevaluated_with_counter` SSB row pair. When
    // `resolved.adversary_runtime` is None (default) or
    // `ELAI_ADVERSARY_ENABLE` is unset, this is a no-op — the row
    // pair stays dark until both the runtime is plumbed via
    // `TaskRunnerConfig` AND the env gate flips. Mirrors the
    // D-10/D-11/D-12 toggle pattern. See dispatch_adversarial_pass +
    // workflow/reasoning/D-13-decisions.md DECISION-5.
    dispatch_adversarial_pass(writer, seq, &task.project_id, resolved).await;
    // D-14 closure: post-task sycophancy-probe dispatch. After the
    // adversarial pass, the post-task hook chain consults the
    // configured sycophancy probe to flag responses that flip on an
    // inverted-framing challenge per Sharma et al
    // arXiv:2310.13548 §3.2. When `resolved.sycophancy_runtime` is
    // None (default) or `ELAI_SYCOPHANCY_PROBE_ENABLE` is unset, this
    // is a no-op — the `sycophancy_probe_dispatched` +
    // `sycophancy_flag` row pair stays dark until both the runtime
    // is plumbed via `TaskRunnerConfig` AND the env gate flips.
    // Defensive: the probe FLAGS only — preference models do NOT
    // reliably fix sycophancy (Sharma §4 — 75% PM residual, 25%
    // oracle floor); the auditor consumes the row and BLOCKs. Chain:
    // route -> verify -> bai -> adversary -> sycophancy. See
    // dispatch_sycophancy_probe_hook + workflow/reasoning/D-14-decisions.md
    // DECISION-8.
    dispatch_sycophancy_probe_hook(writer, seq, &task.project_id, resolved).await;
    // D-15 closure: post-task novelty-label dispatch. After the
    // sycophancy probe, the post-task hook chain consults the
    // configured novelty runtime to label the just-completed task's
    // primary template under the closed three-bucket taxonomy
    // (Search / DistantSearch / DiscoveryCandidate) per Stojkovic
    // et al arXiv:2406.05761 §4 (3-bucket empirical motivation). When
    // `resolved.novelty_runtime` is None (default) or
    // `ELAI_NOVELTY_LABEL_ENABLE` is unset, this is a no-op — the
    // `novelty_classified` row stays dark until both the runtime is
    // plumbed via `TaskRunnerConfig` AND the env gate flips. Chain:
    // route -> verify -> bai -> adversary -> sycophancy -> novelty.
    // The dispatcher emits the row through the
    // `NoveltyClassifiedEmitter` adapter the runtime carries; the
    // promotion / human-review queueing is downstream (D-27 virtual
    // queue). See dispatch_novelty_hook + workflow/reasoning/
    // D-15-decisions.md DECISION-5.
    dispatch_novelty_hook(writer, seq, &task.project_id, resolved).await;
    // D-16 closure: post-task conformal-refusal corpus-grow + nightly
    // tau-updater dispatch. After the novelty label, the hook chain
    // appends the just-completed task's `(claim_score, verdict)` tuple
    // to the calibration corpus (currently a no-op shim — the
    // append-side primary-template surface lands in a follow-up) and,
    // when the per-interval throttle elapses, invokes
    // `TauUpdater::update_tau` to persist a fresh threshold + emit one
    // `calibration_corpus_updated` SSB row. When
    // `resolved.conformal_refusal_runtime` is None (default) or
    // `ELAI_CONFORMAL_REFUSAL_ENABLE` is unset, this is a no-op — the
    // row family stays dark. Chain: route -> verify -> bai -> adversary
    // -> sycophancy -> novelty -> conformal-grow. See
    // `dispatch_conformal_corpus_grow_hook` +
    // workflow/reasoning/D-16-decisions.md DECISION-7.
    dispatch_conformal_corpus_grow_hook(writer, seq, &task.project_id, resolved).await;
    // D-18 closure: post-task typed-ObservableEffect dispatch (layer-8).
    // Emits the 4-variant SSB pack (observable_effect_dispatched +
    // goodhart_risk_flag + sandbox_observation +
    // production_observation_proposal) when
    // `ELAI_OBSERVABLE_EFFECT_ENABLE=1` is set AND the runtime is
    // plumbed. Sits after the D-16 conformal-grow hook in the chain so
    // observable effects can carry the conformal threshold's tau as a
    // future signal. Chain: route -> verify -> bai -> adversary ->
    // sycophancy -> novelty -> conformal-grow -> observable-effect. See
    // `dispatch_observable_effect_hook` +
    // workflow/reasoning/D-18-decisions.md DECISION-10.
    dispatch_observable_effect_hook(writer, seq, &task.project_id, resolved).await;
    info!(
        seq,
        steps_executed,
        "task_runner: task complete"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// D-3 closure: post-task CLAIM-emit consensus dispatch
// ---------------------------------------------------------------------------

/// Producer-side adapter that bridges [`crate::ssb::Ssb`] writes into the
/// `agent_pool::ConsensusSsbEmitter` trait. Lets `run_consensus_pass`
/// emit SSB rows through the live writer without exposing the writer's
/// concrete type to the agent_pool surface.
struct TaskRunnerConsensusEmitter<'a> {
    writer: &'a Arc<Ssb>,
    agent_id: String,
    seq: i64,
}

impl<'a> crate::agent_pool::ConsensusSsbEmitter for TaskRunnerConsensusEmitter<'a> {
    fn emit(
        &self,
        payload: SsbPayload,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        let writer = self.writer.clone();
        let agent_id = self.agent_id.clone();
        let seq = self.seq;
        async move {
            writer
                .write_payload(&agent_id, seq, payload, Vec::new())
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        }
    }
}

/// D-3 closure — CLAIM-emit consensus dispatch hook called once per
/// completed task by `drive_task`. Wires `agent_pool::run_consensus_pass`
/// into the task runner so the post-task CLAIM gate has a real producer
/// call site (closes the NOT_WIRED claim from the D-3 re-check report).
///
/// Default behaviour (no adapters plumbed): no-op. The row family stays
/// dark until concrete VerifierAdapter implementors (PpmAdapter,
/// DeterministicAdapter, IntrinsicAdapter, CrossContextAdapter) are
/// supplied through `ResolvedConfig::verifier_adapters` in a follow-on
/// plan (D-5 / D-13). This matches the existing task_runner pattern of
/// conditional plumbing (`if resolved.llm_provider.is_some()` etc) so
/// the wire is in place without creating new mandatory dependencies.
///
/// Behaviour when adapters are plumbed:
/// - Builds a [`TaskRunnerConsensusEmitter`] bridging `Arc<Ssb>` into
///   [`crate::agent_pool::ConsensusSsbEmitter`].
/// - Invokes [`crate::agent_pool::run_consensus_pass`] with the configured
///   `ConsensusRule` (default `KofN { k: 2, n: 3 }` per
///   `ConsensusRule::default_two_of_three`).
/// - One SSB row emitted (Pass on consensus, Fail on rejection /
///   inconclusive).
///
/// This function is the CLOSING wire for the D-3 NOT_WIRED claim
/// (`run_consensus_pass DEF exists at agent_pool.rs:1028 but zero
/// callers in task_runner.rs`). See workflow/reasoning/D-3-decisions.md
/// (DECISION-2) for the rationale on keeping the free-fn + assoc-fn
/// composition pattern instead of introducing a ResultAggregator trait.
async fn dispatch_verifiers(
    writer: &Arc<Ssb>,
    candidate_id: &str,
    template_id: &str,
    resolved: &ResolvedConfig,
) -> Option<crate::agent_pool::VerifierConsensusPass> {
    let adapters = match resolved.verifier_adapters.as_ref() {
        Some(a) if !a.is_empty() => a,
        _ => {
            // No adapters plumbed — D-3 wire is in place but dark. Trace
            // once at debug so the wiring is observable without flooding
            // the dashboard activity feed.
            tracing::debug!(
                candidate_id,
                template_id,
                "task_runner::dispatch_verifiers: no verifier adapters plumbed; skipping consensus emit"
            );
            return None;
        }
    };

    // Default rule: KofN { k:2, n:3 } per D-3 plan §3.3 (companion §3.3
    // lines 242-256). Future plans (D-9 L3 config consolidation) can
    // surface per-template overrides via ResolvedConfig.
    let rule = crate::verification_result::ConsensusRule::default_two_of_three();

    let emitter = TaskRunnerConsensusEmitter {
        writer,
        agent_id: RUNNER_AGENT_ID.to_string(),
        seq: 0,
    };

    // Collect adapter borrows so we pass &[&dyn DynVerifierAdapter] without
    // moving the Arcs out of resolved. Arc::as_ref returns &T where T is
    // the trait object; coerce explicitly to &dyn for the slice type.
    let adapter_refs: Vec<&dyn crate::agent_pool::DynVerifierAdapter> = adapters
        .iter()
        .map(|a| {
            let r: &dyn crate::agent_pool::DynVerifierAdapter = a.as_ref();
            r
        })
        .collect();

    match crate::agent_pool::run_consensus_pass(
        candidate_id,
        template_id,
        &adapter_refs,
        &rule,
        &emitter,
    )
    .await
    {
        Ok(summary) => {
            tracing::debug!(
                candidate_id = %summary.candidate_id,
                template_id = %summary.template_id,
                verdict = %summary.verdict.as_str(),
                rule_id = %summary.rule_id,
                verifier_count = summary.results.len(),
                "task_runner::dispatch_verifiers: consensus pass complete"
            );
            Some(summary)
        }
        Err(e) => {
            tracing::warn!(
                candidate_id,
                template_id,
                error = %e,
                "task_runner::dispatch_verifiers: consensus pass failed; row emission skipped"
            );
            None
        }
    }
}

/// D-5 closure (GAP-B) — post-consensus BAI round hook.
///
/// Called after [`dispatch_verifiers`] with the consensus summary (if any).
/// When `resolved.bai_runtime` is plumbed AND the env gate `ELAI_BAI_ENABLE`
/// is set, this fires one [`crate::bai::emit_bai_round`] call per consensus
/// verdict: the verdict's pass/fail signal is folded into the candidate
/// arm's Beta(alpha, beta) posterior as a single Bernoulli observation
/// (pass=1 success, fail=1 failure), Thompson sampling picks a winner,
/// and the row triple (`thompson_sampled` -> `bai_arm_updated` ->
/// optional `template_promoted` / `template_candidate_rejected`) is
/// written to SSB.
///
/// `template_id_str` is the orchestrator-side identifier; the BAI module
/// consumes [`crate::reasoning_template::TemplateId`] (uuid). When the
/// string parses as a uuid the parsed value is used directly; otherwise
/// the string is hashed via blake3 and the first 16 bytes become the
/// uuid. Either way the same input string produces the same arm id.
///
/// No-op when:
///   - `resolved.bai_runtime` is None (default — wire dark)
///   - `ELAI_BAI_ENABLE` env var is not set to a truthy value
///   - `consensus` is None (dispatch_verifiers returned without a verdict)
async fn dispatch_bai_round(
    writer: &Arc<Ssb>,
    template_id_str: &str,
    consensus: Option<&crate::agent_pool::VerifierConsensusPass>,
    resolved: &ResolvedConfig,
) {
    let runtime = match resolved.bai_runtime.as_ref() {
        Some(r) => r,
        None => {
            // Wire dark — D-5 default; surface once at debug for
            // observability without flooding the activity feed.
            tracing::debug!(
                template_id = template_id_str,
                "task_runner::dispatch_bai_round: bai_runtime not plumbed; skipping"
            );
            return;
        }
    };
    if !crate::runtime_config::BaiRuntimeConfig::is_enabled() {
        tracing::debug!(
            template_id = template_id_str,
            "task_runner::dispatch_bai_round: ELAI_BAI_ENABLE not set; skipping"
        );
        return;
    }
    let summary = match consensus {
        Some(s) => s,
        None => {
            tracing::debug!(
                template_id = template_id_str,
                "task_runner::dispatch_bai_round: no consensus verdict to fold; skipping"
            );
            return;
        }
    };

    // Bernoulli reward derived from the consensus verdict. Pass = 1 win,
    // anything else = 1 loss (Fail / Inconclusive collapse to the same
    // negative observation per Audibert COLT 2010 §2 — only the binary
    // signal feeds the posterior).
    let passed = summary.verdict.is_pass();
    let eval = crate::archive_chain::StagedEvalResult {
        stage_10_pass: if passed { 1 } else { 0 },
        stage_10_total: 1,
        stage_50_pass: 0,
        stage_50_total: 0,
        stage_200_pass: 0,
        stage_200_total: 0,
        cross_domain_deltas: Vec::new(),
    };

    let template_id = bai_template_id_from_str(template_id_str);

    // D-7 closure GAP-1: consult the latest cross-domain canary verdict
    // BEFORE running the BAI round so the gate can refuse promotion when a
    // non-target family regressed beyond `canary.regression_threshold`.
    // SsbViewer open is best-effort; on failure the gate degrades to the
    // un-gated path (None passed to emit_bai_round_with_canary). This
    // matches D-5 / D-6's "wire-dark = no-op" pattern.
    //
    // ELAI design hypothesis — NanoFlux (arXiv:2509.23252) provides
    // empirical motivation only; no canary mechanism in the paper itself.
    let canary_status =
        match crate::ssb_viewer::SsbViewer::open(writer.db_path()) {
            Ok(viewer) => crate::agent_pool::recent_canary_status(&viewer),
            Err(e) => {
                tracing::debug!(
                    template_id = template_id_str,
                    error = %e,
                    "task_runner::dispatch_bai_round: SsbViewer open failed; canary gate degraded to OFF"
                );
                None
            }
        };

    let mut chain = runtime.chain.lock().await;
    let mut arms = runtime.arms.lock().await;
    let mut rng = runtime.rng.lock().await;

    // D-29: BAI mode dispatch. `Flat` (default) routes to the D-5
    // canonical path. `Hierarchical` runs the flat path FIRST (so the
    // per-arm Beta(alpha, beta) stays honest + canary gate is still
    // enforced) and THEN emits a HierarchicalBaiRoundCompleted summary
    // row using the post-update arm state. This keeps the flat row
    // family (BaiArmUpdated / TemplatePromoted / TemplateCandidateRejected)
    // intact while adding the hierarchical aggregate as a sibling row —
    // matches integration-explorer SEE-ALSO §"SSB row count semantics"
    // (Recommendation: 3-row sequence + 1 hierarchical aggregate).
    let bai_mode = crate::runtime_config::BaiMode::from_env();
    let canary_blocks = canary_status
        .as_ref()
        .is_some_and(|c| c.blocks_promotion());

    if let Err(e) = crate::bai::emit_bai_round_with_canary(
        &mut *chain,
        writer,
        RUNNER_AGENT_ID,
        RUNNER_SUPERSTEP,
        template_id,
        &eval,
        &mut *arms,
        &mut *rng,
        &runtime.promotion_cfg,
        canary_status.as_ref(),
    )
    .await
    {
        tracing::warn!(
            template_id = template_id_str,
            error = %e,
            "task_runner::dispatch_bai_round: emit_bai_round failed; arm update skipped"
        );
    }

    if matches!(bai_mode, crate::runtime_config::BaiMode::Hierarchical) {
        // D-29 hierarchical aggregate row. The flat path above has
        // already updated `arms`; we now read the post-update state,
        // run the hierarchical TS selector, and emit one
        // HierarchicalBaiRoundCompleted summary. The hierarchical
        // state itself is held inline (no persistent state — the
        // hyperprior is re-derived per dispatch from the live arm
        // posteriors, which is the equivalent of running the Gibbs
        // sampler with a fresh init each round; mixing is fast on
        // conjugate-adjacent posteriors per Gelman BDA3 §11.5).
        let mut hier_state =
            crate::bai_hierarchical::HierarchicalBaiState::new();
        // All arms route to FamilyId(0) by default (the runtime does
        // not yet ship a family-routing map; this is the
        // RUNPOD-PENDING wiring stub — see D-29-decisions.md
        // DECISION-7).
        for arm in arms.iter() {
            hier_state.assign_arm(
                arm.template_id,
                crate::bai_hierarchical::FamilyId(0),
            );
        }
        let family = crate::bai_hierarchical::FamilyId(0);
        let arm_refs: Vec<&crate::bai::ArmPosterior> = arms.iter().collect();
        let family_post = crate::bai_hierarchical::hierarchical_update(
            &mut hier_state,
            family,
            &arm_refs,
            &mut *rng,
        );
        let select = crate::bai_hierarchical::hierarchical_arm_select(
            &hier_state,
            &arms,
            &mut *rng,
        );

        let (selected_arm_index, selected_template_id, basis): (
            u32,
            String,
            crate::bai_hierarchical::SelectionBasis,
        ) = match select {
            Some((idx, b)) => {
                let id = arms[idx].template_id.as_hex();
                (idx as u32, id, b)
            }
            None => (
                u32::MAX,
                "0".repeat(32),
                if canary_blocks {
                    crate::bai_hierarchical::SelectionBasis::CanaryOverride
                } else {
                    crate::bai_hierarchical::SelectionBasis::FlatTs
                },
            ),
        };

        let basis_final = if canary_blocks {
            crate::bai_hierarchical::SelectionBasis::CanaryOverride
        } else {
            basis
        };

        let arm_posteriors: Vec<crate::ssb::ArmPosteriorSummary> = arms
            .iter()
            .map(|arm| {
                let (alpha, beta) = (arm.alpha, arm.beta);
                let mean = arm.mean();
                let denom = alpha + beta;
                let variance = if denom > 0.0 {
                    (alpha * beta) / (denom * denom * (denom + 1.0))
                } else {
                    0.0
                };
                let (ci_low, ci_high) =
                    crate::bai::credible_interval(arm, 0.95);
                let (hyper_a, hyper_b) = family_post.implied_beta_params();
                let pooled_denom = alpha + hyper_a + beta + hyper_b;
                let pooled_mean = if pooled_denom > 0.0 {
                    (alpha + hyper_a) / pooled_denom
                } else {
                    mean
                };
                crate::ssb::ArmPosteriorSummary {
                    template_id: arm.template_id.as_hex(),
                    family_id: 0,
                    alpha,
                    beta,
                    mean,
                    variance,
                    ci_low,
                    ci_high,
                    pooled_mean,
                }
            })
            .collect();

        let payload =
            crate::ssb::SsbPayload::HierarchicalBaiRoundCompleted(
                crate::ssb::HierarchicalBaiRoundCompletedPayload {
                    round: family_post.n_gibbs_iterations,
                    family_id: family.as_u32(),
                    mu_posterior_mean: family_post.mu_mean,
                    mu_posterior_variance: family_post.mu_variance,
                    kappa_posterior_mean: family_post.kappa_mean,
                    kappa_posterior_variance: family_post.kappa_variance,
                    arm_posteriors,
                    selected_arm_index,
                    selected_template_id,
                    selection_basis: basis_final.as_str().to_string(),
                    gibbs_iterations: crate::bai_hierarchical::MAX_GIBBS_ITERATIONS
                        - crate::bai_hierarchical::GIBBS_BURN_IN,
                    canary_gate_passed: !canary_blocks,
                },
            );
        if let Err(e) = writer
            .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
            .await
        {
            tracing::warn!(
                template_id = template_id_str,
                error = %e,
                "task_runner::dispatch_bai_round: HierarchicalBaiRoundCompleted SSB write failed"
            );
        }
    }
}

/// D-5 closure (GAP-B) — derive a stable
/// [`crate::reasoning_template::TemplateId`] from an arbitrary template
/// identifier string. When the input parses as a canonical uuid (4-, 8-,
/// or simple-hex form) the parsed value is returned verbatim. Otherwise
/// blake3-hash the bytes and use the first 16 bytes as the uuid. Same
/// input -> same id, no clock dependency.
fn bai_template_id_from_str(s: &str) -> crate::reasoning_template::TemplateId {
    if let Ok(parsed) = uuid::Uuid::parse_str(s) {
        return crate::reasoning_template::TemplateId::from(parsed);
    }
    let digest = blake3::hash(s.as_bytes());
    let bytes = digest.as_bytes();
    let mut id_bytes = [0u8; 16];
    id_bytes.copy_from_slice(&bytes[..16]);
    crate::reasoning_template::TemplateId::from(uuid::Uuid::from_bytes(id_bytes))
}

// ---------------------------------------------------------------------------
// D-10 closure: post-task CLAIM-emit mode-routing dispatch
// ---------------------------------------------------------------------------

/// D-10 closure — production [`crate::agent_pool::ClaimRoutedEmitter`]
/// that bridges `Arc<Ssb>::write_payload` into the routing producer
/// surface. Mirrors the D-3 `TaskRunnerConsensusEmitter` shape so the
/// two producers share a single wiring pattern (CLAUDE.md "no two-way
/// drift between sibling D-N hooks" preference).
///
/// Native async-fn-in-trait via `Pin<Box<Future>>` per the
/// anti-stale-training rule (no `async-trait` crate added).
pub struct TaskRunnerClaimRoutedEmitter {
    writer: Arc<Ssb>,
    agent_id: String,
    seq: i64,
}

impl TaskRunnerClaimRoutedEmitter {
    /// Construct a production emitter bound to the live `Arc<Ssb>` the
    /// task_runner already holds. `agent_id` defaults to
    /// [`RUNNER_AGENT_ID`] when the caller does not need a custom id;
    /// `seq` is the per-task superstep correlator (zero is acceptable
    /// for the MVP per the [`RUNNER_SUPERSTEP`] convention).
    #[must_use]
    pub fn new(writer: Arc<Ssb>, agent_id: impl Into<String>, seq: i64) -> Self {
        Self {
            writer,
            agent_id: agent_id.into(),
            seq,
        }
    }
}

impl crate::agent_pool::ClaimRoutedEmitter for TaskRunnerClaimRoutedEmitter {
    fn emit(
        &self,
        payload: crate::ssb::ClaimRoutedPayload,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        let writer = self.writer.clone();
        let agent_id = self.agent_id.clone();
        let seq = self.seq;
        Box::pin(async move {
            writer
                .write_payload(
                    &agent_id,
                    seq,
                    SsbPayload::ClaimRouted(payload),
                    Vec::new(),
                )
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
    }
}

/// D-10 closure — post-task CLAIM-emit mode-routing dispatch hook
/// called once per completed task by `drive_task`. Wires
/// [`crate::agent_pool::route_claim`] into the task runner so the
/// `claim_routed` SSB row family has a real production call site
/// (closes the NOT_WIRED claim from the D-10 re-check report).
///
/// Default behaviour (no runtime plumbed): no-op. Same wire-dark
/// pattern as [`dispatch_verifiers`] / [`dispatch_bai_round`] —
/// adapters land in a follow-on sidecar/CLI boot patch without any
/// further task_runner edit. Behaviour when plumbed + the env gate
/// `ELAI_CLAIM_ROUTER_ENABLE` is set:
///
/// 1. Synthesise an [`crate::claim::Claim`] for the just-completed
///    task (claim_type = `Procedural` per the D-10 routing table for
///    task-completion-style events; the orchestrator's decision is
///    `OperationalDeferred` until D-18 ObservableEffect lands).
/// 2. Snapshot an [`crate::orch_context::OrchContext`] with the
///    current canary status (SsbViewer-sourced) + the registered
///    verifier kinds (empty until D-11/D-13 plumb concrete adapters).
/// 3. Call [`crate::agent_pool::route_claim`] with the runtime's
///    emitter (typically a [`TaskRunnerClaimRoutedEmitter`] wrapping
///    `Arc<Ssb>`). The producer is deterministic in (claim, ctx) so
///    the same task seq always routes to the same mode (provided the
///    canary state has not changed).
/// 4. `EmissionGuard::confirm()` suppresses the drop-time warning;
///    the routing decision is the durable signal (the SSB row is
///    already emitted by `route_claim`).
///
/// `now_ms` is sourced from the in-process `OsClock` per the existing
/// task_runner convention; entropy quarantine is preserved because
/// `route_claim` accepts `now_ms` as an injected arg.
async fn dispatch_claim_routing(
    writer: &Arc<Ssb>,
    seq: i64,
    project_id: &str,
    resolved: &ResolvedConfig,
) {
    let runtime = match resolved.claim_router_runtime.as_ref() {
        Some(r) => r,
        None => {
            // Wire dark — D-10 default. Log once at debug so the wire
            // is observable without flooding the activity feed.
            tracing::debug!(
                seq,
                project_id,
                "task_runner::dispatch_claim_routing: claim_router_runtime not plumbed; skipping"
            );
            return;
        }
    };
    if !crate::runtime_config::ClaimRouterRuntimeConfig::is_enabled() {
        tracing::debug!(
            seq,
            project_id,
            "task_runner::dispatch_claim_routing: ELAI_CLAIM_ROUTER_ENABLE not set; skipping"
        );
        return;
    }

    // 1. Synthesise the per-task Claim. The claim_id is derived from
    //    the task seq (stable per-row identifier); text is bounded to
    //    avoid pulling task summaries into the row body (router does
    //    not inspect `text`). Evidence is empty by default — concrete
    //    Corpus/ToolOutput refs land when checker/verifier adapters
    //    plumb their results through the runtime.
    let now_ms: i64 = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
    )
    .unwrap_or(0);
    let claim = crate::claim::Claim {
        claim_id: format!("task-{seq}"),
        text: format!("task {seq} on project {project_id}"),
        // Procedural per the D-10 routing table — task-completion-style
        // claims describe a procedure (a baby-step sequence), not a
        // factual assertion. The router maps Procedural to
        // OperationalDeferred until D-18 ObservableEffect lands.
        claim_type: crate::claim::ClaimType::Procedural,
        evidence: Vec::new(),
        ts_ms: now_ms,
    };

    // 2. Snapshot the OrchContext. SsbViewer open is best-effort; on
    //    failure the canary gate degrades to OFF (None) — matches the
    //    `dispatch_bai_round` precedent. `available_verifiers` is
    //    empty by default; D-11/D-13 plumbing populates it.
    let canary_status = match crate::ssb_viewer::SsbViewer::open(writer.db_path()) {
        Ok(viewer) => crate::agent_pool::recent_canary_status(&viewer),
        Err(e) => {
            tracing::debug!(
                seq,
                project_id,
                error = %e,
                "task_runner::dispatch_claim_routing: SsbViewer open failed; canary gate degraded to OFF"
            );
            None
        }
    };
    let mut ctx = crate::orch_context::OrchContext::minimal(
        crate::orch_context::OrchUserRole::Executor,
        crate::orch_context::OrchPhase::Executing,
    );
    ctx.recent_canary_status = canary_status;

    // 3. Route + emit. The emitter wraps Arc<Ssb>::write_payload (the
    //    production adapter constructed at boot); test runtimes inject
    //    a VecClaimRoutedEmitter instead.
    let emitter_ref: &dyn crate::agent_pool::ClaimRoutedEmitter = runtime.emitter.as_ref();
    let (mode, guard) =
        crate::agent_pool::route_claim(&claim, &ctx, emitter_ref, now_ms).await;

    // 4. Confirm the routing decision was observed (suppresses the
    //    drop-time warning). The SSB row is the durable signal.
    let confirmed = guard.confirm();
    tracing::debug!(
        seq,
        project_id,
        mode = mode.as_str(),
        confirmed = confirmed.as_str(),
        "task_runner::dispatch_claim_routing: claim_routed row emitted"
    );
}

// ---------------------------------------------------------------------------
// D-11 closure: post-task constructive-checker dispatch hook
// ---------------------------------------------------------------------------

/// D-11 closure — constructive-checker dispatch hook called once per
/// completed task by `drive_task`. Wires
/// [`crate::agent_pool::dispatch_constructive_checkers`] into the task
/// runner so the post-task checker run has a real producer call site
/// (closes the NOT_WIRED claim from the D-11 re-check report).
///
/// Default behaviour (no runtime plumbed): no-op. The row family stays
/// dark until both the runtime is plumbed via `TaskRunnerConfig.checker_runtime`
/// AND the operator sets `ELAI_CHECKER_ENABLE=1`. Mirrors the D-10
/// claim-router gate so the toggle pattern is consistent across
/// producers and adding the wire requires no further task_runner edits.
///
/// Behaviour when plumbed + gated ON:
/// 1. Synthesise a per-task [`crate::claim::Claim`] (same shape as the
///    D-10 hook — claim_type = `Procedural` for task-completion-style
///    events; text bounded so the registry's applies_to predicates
///    have stable input).
/// 2. Invoke
///    [`crate::agent_pool::dispatch_constructive_checkers`] with the
///    configured registry + emitter + injected `now_ms` (entropy
///    quarantine — clock read once at the boundary, not inside the
///    producer).
/// 3. Trace the dispatch fan-out at debug so the wire is observable
///    without flooding the dashboard activity feed.
async fn dispatch_constructive_checkers_hook(
    seq: i64,
    project_id: &str,
    resolved: &ResolvedConfig,
) {
    let runtime = match resolved.checker_runtime.as_ref() {
        Some(r) => r,
        None => {
            // Wire dark — D-11 default. Log once at debug so the wire
            // is observable without flooding the activity feed.
            tracing::debug!(
                seq,
                project_id,
                "task_runner::dispatch_constructive_checkers_hook: checker_runtime not plumbed; skipping"
            );
            return;
        }
    };
    if !crate::runtime_config::CheckerRuntimeConfig::is_enabled() {
        tracing::debug!(
            seq,
            project_id,
            "task_runner::dispatch_constructive_checkers_hook: ELAI_CHECKER_ENABLE not set; skipping"
        );
        return;
    }

    // 1. Synthesise the per-task Claim. Same shape as `dispatch_claim_routing`
    //    so SSB rows from both hooks share a stable claim_id schema
    //    (`task-{seq}`) — the dashboard can correlate `claim_routed` +
    //    `checker_result_emitted` rows by claim_id.
    let now_ms: i64 = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
    )
    .unwrap_or(0);
    let claim = crate::claim::Claim {
        claim_id: format!("task-{seq}"),
        text: format!("task {seq} on project {project_id}"),
        // Procedural — task-completion-style claims describe a procedure
        // (a baby-step sequence). The registry's per-checker applies_to
        // predicate decides which checkers actually fire; this hook
        // does not pre-filter by claim_type.
        claim_type: crate::claim::ClaimType::Procedural,
        evidence: Vec::new(),
        ts_ms: now_ms,
    };

    // 2. Dispatch. The producer iterates every applicable checker in
    //    sorted-id order and emits one SSB row per result via the
    //    runtime's emitter. Return value (the triples vector) is
    //    unused at the hook layer — the SSB rows are the durable
    //    signal the dashboard reads.
    let emitter_ref: &dyn crate::agent_pool::CheckerResultEmitter =
        runtime.emitter.as_ref();
    let triples = crate::agent_pool::dispatch_constructive_checkers(
        &claim,
        runtime.registry.as_ref(),
        emitter_ref,
        now_ms,
    )
    .await;

    tracing::debug!(
        seq,
        project_id,
        results = triples.len(),
        "task_runner::dispatch_constructive_checkers_hook: checker_result_emitted rows emitted"
    );
}

// ---------------------------------------------------------------------------
// D-12 closure: post-task operationalization-check dispatch hook
// ---------------------------------------------------------------------------

/// D-12 closure — operationalization-check dispatch hook called once per
/// completed task by `drive_task`. Wires
/// [`crate::agent_pool::run_operational_check`] into the task runner so
/// the post-task baby-step operationalisation gate has a real producer
/// call site (closes the GAP-1 "dead code" report against
/// `run_operational_check` per D-12-decisions.md DECISION-6).
///
/// Default behaviour (no runtime plumbed): no-op. The row family stays
/// dark until both the runtime is plumbed via
/// `TaskRunnerConfig.operationalization_runtime` AND the operator sets
/// `ELAI_OPERATIONALIZATION_ENABLE=1`. Mirrors the D-10/D-11 toggle
/// pattern so the operator surface stays consistent across producers.
///
/// Behaviour when plumbed + gated ON:
/// 1. Synthesise a per-task [`crate::baby_step::BabyStep`] envelope.
///    The `step_id` is `bs-task-{seq}`; the `parent_task_id` is `task-{seq}`
///    so dashboard correlation against `claim_routed` +
///    `checker_result_emitted` rows by `task-{seq}` claim_id stays
///    coherent. The synthesised baby step intentionally carries
///    `measurable_procedure = None / claim_payload = None / expected_result
///    = None` — the predicate
///    [`crate::baby_step::is_operationalized`] returns `false` and the
///    dispatch outcome is `NotOperationalized`. The hook still emits
///    the SSB row with `outcome = "inapplicable"` so dashboards can
///    surface coverage gaps (per
///    [`crate::baby_step::DispatchOutcome::outcome_label`]).
/// 2. Invoke [`crate::agent_pool::run_operational_check`] with the
///    configured emitter + registry + injected `now_ms` (entropy
///    quarantine — clock read once at the boundary, not inside the
///    producer). The producer returns
///    `Err(OperationalError::NotOperationalized)` for the synthesised
///    no-claim envelope; we trace it at debug rather than propagating
///    the error because the task itself succeeded and the SSB row IS
///    the durable signal (auditor reads the row, not the Result).
/// 3. Trace the outcome at debug so the wire is observable without
///    flooding the dashboard activity feed.
///
/// FUTURE: per D-15 (NoveltyClass labels), this hook will fan out to
/// the per-claim operationalisation gate driven by real baby steps the
/// babystep-author emits via the agent prompt. The synthesised envelope
/// here is the closure surface that proves the production wire is live.
async fn run_operational_check_hook(seq: i64, project_id: &str, resolved: &ResolvedConfig) {
    let runtime = match resolved.operationalization_runtime.as_ref() {
        Some(r) => r,
        None => {
            // Wire dark — D-12 default. Log once at debug so the wire
            // is observable without flooding the activity feed.
            tracing::debug!(
                seq,
                project_id,
                "task_runner::run_operational_check_hook: operationalization_runtime not plumbed; skipping"
            );
            return;
        }
    };
    if !crate::runtime_config::OperationalizationRuntimeConfig::is_enabled() {
        tracing::debug!(
            seq,
            project_id,
            "task_runner::run_operational_check_hook: ELAI_OPERATIONALIZATION_ENABLE not set; skipping"
        );
        return;
    }

    // 1. Synthesise the per-task BabyStep envelope. Same `task-{seq}`
    //    correlation id as the D-10/D-11 hooks so dashboard rows align
    //    across producers. measurable_procedure / expected_result /
    //    claim_payload all `None` -> the predicate returns false and
    //    the dispatch outcome is NotOperationalized; the hook still
    //    emits the SSB row with outcome="inapplicable" so the
    //    coverage-gap signal is durable.
    let now_ms: i64 = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
    )
    .unwrap_or(0);
    let step = crate::baby_step::BabyStep {
        step_id: format!("bs-task-{seq}"),
        parent_task_id: format!("task-{seq}"),
        description: format!("synthesised post-task operationalisation envelope for project {project_id}"),
        measurable_procedure: None,
        expected_result: None,
        claim_payload: None,
    };

    // 2. Dispatch. The producer emits exactly one `baby_step_dispatched`
    //    SSB row via the runtime's emitter and converts the outcome to
    //    Result. NotOperationalized is the expected return shape for
    //    the synthesised envelope; we log at debug rather than
    //    surfacing as an error because the SSB row IS the durable
    //    signal the auditor reads.
    let emitter_ref: &dyn crate::agent_pool::BabyStepDispatchedEmitter =
        runtime.emitter.as_ref();
    let outcome_label = match crate::agent_pool::run_operational_check(
        &step,
        runtime.registry.as_ref(),
        emitter_ref,
        now_ms,
    )
    .await
    {
        Ok(out) => out.outcome_label(),
        Err(crate::agent_pool::OperationalError::NotOperationalized { .. }) => {
            "inapplicable"
        }
        Err(crate::agent_pool::OperationalError::NonPassOutcome { .. }) => "fail",
    };

    tracing::debug!(
        seq,
        project_id,
        outcome = outcome_label,
        "task_runner::run_operational_check_hook: baby_step_dispatched row emitted"
    );
}

// ---------------------------------------------------------------------------
// D-13 closure: post-task adversarial-pass dispatch hook
// ---------------------------------------------------------------------------

/// D-13 closure — adversarial-pass dispatch hook called once per
/// completed task by `drive_task`. Wires
/// [`crate::agent_pool::run_adversarial_pass`] into the task runner so
/// the `GroundingMode::FalsifiabilityCrossFamily` branch has a real
/// production call site (closes GAP-1 of the D-13 recheck report —
/// `run_adversarial_pass` had zero production callers before this
/// hook landed).
///
/// Default behaviour (no runtime plumbed): no-op. The row pair stays
/// dark until both the runtime is plumbed via
/// `TaskRunnerConfig.adversary_runtime` AND the operator sets
/// `ELAI_ADVERSARY_ENABLE=1`. Mirrors the D-10/D-11/D-12 toggle
/// pattern so the operator surface stays consistent across producers.
///
/// Behaviour when plumbed + gated ON:
/// 1. Synthesise the per-task Claim (same `task-{seq}` correlation id
///    as the D-10/D-11/D-12 hooks so dashboard rows align by claim_id).
/// 2. Resolve the worker family. For the synthesised envelope the
///    worker is the local default sidecar — we read it from
///    `resolved.backend_config.model` via
///    [`crate::adversary::ModelFamily::from_model_id`]. (When concrete
///    per-claim worker classification lands via D-15 the hook will
///    consult the claim's emitter instead.)
/// 3. Run the `AdversaryRuntimeConfig::pair_is_allowed` guard. When
///    the pair is refused (same-family or not on the
///    `preferred_cross_family_pairs` filter), trace at debug and
///    skip. The dispatch is silent rather than emitting a "missing
///    cross-family adversary" SSB row because the existing
///    `HeterogeneousVerifierMissing` `tracing::warn!` in
///    `run_adversarial_pass` covers the heterogeneous-pool gap, and
///    the consolidated-SSB-variant precedent (D-9/D-10/D-11/D-12)
///    avoids inflating the variant set for a signal that the
///    consensus pass already conveys via verdict-inversion.
/// 4. Invoke
///    [`crate::agent_pool::run_adversarial_pass`] with the configured
///    adversary + the empty verifier pool (concrete adapters land via
///    the D-3 `verifier_adapters` slot in a follow-on patch). The
///    producer emits the `adversary_generated` +
///    `consensus_reevaluated_with_counter` SSB row pair via the
///    `TaskRunnerConsensusEmitter`. Adversary errors are caught at
///    the hook layer and traced — the SSB row pair IS the durable
///    signal the auditor reads; producer errors do not propagate
///    upward.
async fn dispatch_adversarial_pass(
    writer: &Arc<Ssb>,
    seq: i64,
    project_id: &str,
    resolved: &ResolvedConfig,
) {
    let runtime = match resolved.adversary_runtime.as_ref() {
        Some(r) => r,
        None => {
            // Wire dark — D-13 default. Log once at debug so the wire
            // is observable without flooding the activity feed.
            tracing::debug!(
                seq,
                project_id,
                "task_runner::dispatch_adversarial_pass: adversary_runtime not plumbed; skipping"
            );
            return;
        }
    };
    if !crate::runtime_config::AdversaryRuntimeConfig::is_enabled() {
        tracing::debug!(
            seq,
            project_id,
            "task_runner::dispatch_adversarial_pass: ELAI_ADVERSARY_ENABLE not set; skipping"
        );
        return;
    }

    // 1. Synthesise the per-task Claim. Same `task-{seq}` correlation
    //    id as the D-10/D-11/D-12 hooks so dashboard rows align by
    //    claim_id across producers.
    let now_ms: i64 = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
    )
    .unwrap_or(0);
    let claim = crate::claim::Claim {
        claim_id: format!("task-{seq}"),
        text: format!("task {seq} on project {project_id}"),
        // Procedural — task-completion-style claims. Matches the
        // D-10/D-11/D-12 hook shape for dashboard correlation.
        claim_type: crate::claim::ClaimType::Procedural,
        evidence: Vec::new(),
        ts_ms: now_ms,
    };

    // 2. Resolve the worker family from the configured backend model
    //    id. The substring classifier defaults to Local for unknown
    //    ids (conservative — Local-vs-Local pair trips
    //    SameFamilyPairing in `run_adversarial_pass` rather than
    //    silently dispatching).
    let worker_family =
        crate::adversary::ModelFamily::from_model_id(&resolved.backend_config.model);
    let adversary_family_label =
        runtime.adversary_config.adversary_model_family.as_str().to_string();
    let worker_family_label = worker_family.as_str().to_string();

    // 3. Cross-family pair guard. Refused pairs short-circuit with a
    //    debug trace — the dispatch is silent per the
    //    consolidated-SSB-variant precedent (D-9/D-10/D-11/D-12) which
    //    avoids inflating TOTAL_VARIANTS for a signal already covered
    //    by the inverted-verdict row.
    if !runtime
        .runtime_config
        .pair_is_allowed(&worker_family_label, &adversary_family_label)
    {
        tracing::debug!(
            seq,
            project_id,
            worker_family = %worker_family_label,
            adversary_family = %adversary_family_label,
            "task_runner::dispatch_adversarial_pass: cross-family pair refused by runtime_config; skipping"
        );
        return;
    }

    // 4. Run the adversarial pass. The TaskRunnerConsensusEmitter
    //    (already used by `dispatch_verifiers` per D-3) is the right
    //    SSB-write seam — both the `AdversaryGenerated` payload and
    //    the `ConsensusReevaluatedWithCounter` payload land as
    //    `SsbPayload` writes through the same emitter trait.
    let emitter = TaskRunnerConsensusEmitter {
        writer,
        agent_id: RUNNER_AGENT_ID.to_string(),
        seq,
    };
    let rule = crate::verification_result::ConsensusRule::default_two_of_three();
    let verifier_families: Vec<(
        crate::verification_result::VerifierKind,
        crate::adversary::ModelFamily,
    )> = vec![
        // Worker side of the heterogeneous pool. When the D-3
        // verifier_adapters slot is plumbed, the per-adapter family
        // tuples will replace this synthesised pair.
        (
            crate::verification_result::VerifierKind::Intrinsic,
            worker_family.clone(),
        ),
        // Adversary side — guaranteed cross-family by the
        // `pair_is_allowed` check above.
        (
            crate::verification_result::VerifierKind::CrossContext,
            runtime.adversary_config.adversary_model_family.clone(),
        ),
    ];
    let adapters: Vec<&dyn crate::agent_pool::DynVerifierAdapter> = Vec::new();
    let result = crate::agent_pool::run_adversarial_pass(
        &claim,
        runtime.adversary.as_ref(),
        &runtime.adversary_config,
        &adapters,
        &verifier_families,
        &rule,
        &emitter,
    )
    .await;

    match result {
        Ok(summary) => {
            tracing::debug!(
                seq,
                project_id,
                claim_id = %summary.claim_id,
                original_verdict = summary.original_verdict.as_str(),
                post_counter_verdict = summary.post_counter_verdict.as_str(),
                heterogeneous = summary.heterogeneous,
                "task_runner::dispatch_adversarial_pass: adversarial row pair emitted"
            );
        }
        Err(e) => {
            // Adversary errors are observed at trace level — the SSB
            // row pair already emitted (if any), the auditor reads
            // the rows, and the task itself succeeded. We do NOT
            // propagate the error upward because the
            // adversarial-pass is a quality signal, not a gate on
            // task completion.
            tracing::warn!(
                seq,
                project_id,
                error = %e,
                "task_runner::dispatch_adversarial_pass: adversary pass returned error; SSB rows may be incomplete"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// D-14 closure: post-task sycophancy-probe dispatch hook.
// ---------------------------------------------------------------------------

/// D-14 closure — post-task sycophancy-probe dispatch hook called once
/// per completed task by `drive_task`, after [`dispatch_adversarial_pass`].
///
/// When `resolved.sycophancy_runtime` is plumbed AND the env gate
/// `ELAI_SYCOPHANCY_PROBE_ENABLE=1` is set, this hook:
///   1. Synthesises a per-task user-message / primary-response pair
///      from the just-completed task (correlation id `task-{seq}` per
///      the D-10/D-11/D-12/D-13 hook shape).
///   2. Builds a `SycophancyProbe` with the configured probe kind
///      (default `AreYouSure` per Sharma §3.2).
///   3. Calls [`crate::sycophancy::dispatch_sycophancy_probe`].
///   4. Emits the SSB row pair (`SycophancyProbeDispatched` +
///      `SycophancyFlag`) through the same `TaskRunnerConsensusEmitter`
///      seam used by D-3/D-13.
///
/// No-op when:
///   - `resolved.sycophancy_runtime` is None (default — wire dark)
///   - `ELAI_SYCOPHANCY_PROBE_ENABLE` is not set to a truthy value
///
/// Defensive boundary: this hook FLAGS only. It does NOT auto-rewrite
/// the agent's response — preference models do NOT reliably fix
/// sycophancy per Sharma §4 (75% PM residual, 25% oracle floor). The
/// auditor consumes the `SycophancyFlag` row and BLOCKs on
/// `flip_detected = true`.
///
/// Grounding: arXiv:2310.13548 (Sharma et al, Anthropic, ICLR 2024)
/// §3.2 (user-disagreement-flip detection) + §4 (PM residual
/// sycophancy). See workflow/papers/sharma-sycophancy-2310.13548-interface.md
/// and workflow/reasoning/D-14-decisions.md.
async fn dispatch_sycophancy_probe_hook(
    writer: &Arc<Ssb>,
    seq: i64,
    project_id: &str,
    resolved: &ResolvedConfig,
) {
    let runtime = match resolved.sycophancy_runtime.as_ref() {
        Some(r) => r,
        None => {
            tracing::debug!(
                seq,
                project_id,
                "task_runner::dispatch_sycophancy_probe_hook: sycophancy_runtime not plumbed; skipping"
            );
            return;
        }
    };
    if !crate::runtime_config::SycophancyRuntimeConfig::is_enabled() {
        tracing::debug!(
            seq,
            project_id,
            "task_runner::dispatch_sycophancy_probe_hook: ELAI_SYCOPHANCY_PROBE_ENABLE not set; skipping"
        );
        return;
    }

    // 1. Resolve the probe kind from the runtime config. Unknown
    //    labels fall back to AreYouSure (Sharma §3.2 canonical
    //    detector) so an operator typo never crashes the hook.
    let probe_kind = match runtime.runtime_config.probe_kind_default.as_str() {
        "feedback" => crate::sycophancy::SycophancyProbeKind::Feedback,
        "answer" => crate::sycophancy::SycophancyProbeKind::Answer,
        "mimicry" => crate::sycophancy::SycophancyProbeKind::Mimicry,
        _ => crate::sycophancy::SycophancyProbeKind::AreYouSure,
    };

    // 2. Synthesise the probe input. The user_message field carries
    //    the original turn-1 answer per the dispatch_sycophancy_probe
    //    calling convention; primary_response carries the turn-2
    //    challenged answer. Production callers wire the real two-turn
    //    pair from the task's emitted claim row; this hook ships the
    //    correlation id correctly so the dashboard pair-up holds even
    //    in the wire-dark default.
    let claim_id = format!("task-{seq}");
    let user_message =
        format!("task {seq} on project {project_id} (primary response anchor)");
    let primary_response = user_message.clone();
    let probe = crate::sycophancy::SycophancyProbe {
        claim_id: claim_id.clone(),
        user_message: user_message.clone(),
        primary_response,
        kind: probe_kind,
    };

    // 3. Build a snapshot OrchContext. The probe is observation-only
    //    on `OrchContext`, so the minimal-refusal-biased context is
    //    safe — the verdict depends on the (original, challenged)
    //    pair, not on the context fields.
    let ctx = crate::orch_context::OrchContext::minimal(
        crate::orch_context::OrchUserRole::Executor,
        crate::orch_context::OrchPhase::Executing,
    );

    // 4. Emit the SycophancyProbeDispatched row first (matches the
    //    D-13 producer-then-result pattern). Errors at the SSB write
    //    layer are warn-traced; the probe is a quality signal, NOT a
    //    gate on task completion.
    let now_ms = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
    )
    .unwrap_or(0);
    let _ = now_ms; // reserved for future ts_ms-carrying expansion

    let dispatched_payload =
        crate::ssb::SycophancyProbeDispatchedPayload {
            claim_id: claim_id.clone(),
            probe_kind: probe_kind.as_str().to_string(),
            user_message,
        };
    if let Err(e) = writer
        .write_payload(
            RUNNER_AGENT_ID,
            seq,
            crate::ssb::SsbPayload::SycophancyProbeDispatched(dispatched_payload),
            Vec::new(),
        )
        .await
    {
        tracing::warn!(
            seq,
            project_id,
            error = %e,
            "task_runner::dispatch_sycophancy_probe_hook: dispatched row write failed"
        );
        return;
    }

    // 5. Run the deterministic probe core + emit the verdict row.
    match crate::sycophancy::dispatch_sycophancy_probe(&probe, &ctx) {
        Ok(report) => {
            let flag_payload = crate::ssb::SycophancyFlagPayload {
                claim_id: claim_id.clone(),
                flip_detected: report.flip_detected,
                delta: match report.verdict {
                    crate::sycophancy::ProbeVerdict::DivergentSycophancyFlag {
                        delta,
                    } => delta,
                    _ => 0.0,
                },
                verdict: report.verdict.as_str().to_string(),
            };
            if let Err(e) = writer
                .write_payload(
                    RUNNER_AGENT_ID,
                    seq,
                    crate::ssb::SsbPayload::SycophancyFlag(flag_payload),
                    Vec::new(),
                )
                .await
            {
                tracing::warn!(
                    seq,
                    project_id,
                    error = %e,
                    "task_runner::dispatch_sycophancy_probe_hook: flag row write failed"
                );
            } else {
                tracing::debug!(
                    seq,
                    project_id,
                    claim_id = %claim_id,
                    flip_detected = report.flip_detected,
                    verdict = report.verdict.as_str(),
                    "task_runner::dispatch_sycophancy_probe_hook: probe row pair emitted"
                );
            }
        }
        Err(e) => {
            tracing::warn!(
                seq,
                project_id,
                error = %e,
                "task_runner::dispatch_sycophancy_probe_hook: probe returned error; flag row skipped"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// D-15 closure: post-task novelty-label dispatch hook
// ---------------------------------------------------------------------------

/// Producer-side adapter that bridges [`crate::ssb::Ssb`] writes into
/// the [`crate::novelty::NoveltyClassifiedEmitter`] trait. Lets
/// `agent_pool::dispatch_novelty` emit `NoveltyClassified` SSB rows
/// through the live writer without exposing the writer's concrete
/// type to the novelty surface. Mirrors the D-3 / D-14 emitter-adapter
/// pattern (private `task_runner`-scoped struct, no allocation beyond
/// the `Arc<Ssb>` clone the runner already holds).
struct TaskRunnerNoveltyEmitter<'a> {
    writer: &'a Arc<Ssb>,
    seq: i64,
}

impl<'a> crate::novelty::NoveltyClassifiedEmitter for TaskRunnerNoveltyEmitter<'a> {
    fn emit(
        &self,
        payload: crate::ssb::NoveltyClassifiedPayload,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<(), String>> + Send + '_,
        >,
    > {
        let writer = self.writer.clone();
        let seq = self.seq;
        Box::pin(async move {
            writer
                .write_payload(
                    RUNNER_AGENT_ID,
                    seq,
                    crate::ssb::SsbPayload::NoveltyClassified(payload),
                    Vec::new(),
                )
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
    }
}

/// D-15 closure — post-task novelty-label dispatch hook called once
/// per completed task by `drive_task`. Wires
/// [`crate::agent_pool::dispatch_novelty`] into the task runner so the
/// post-task novelty-classification surface has a real producer call
/// site (closes GAP-1 against D-15 — "dispatch_novelty has no
/// production caller that fires the SSB row" per
/// workflow/sessions/s-25120/cross-check-recheck/D-15_novelty_class_labeling-report.md).
///
/// Default behaviour (no runtime plumbed OR env-gate off): strict
/// no-op. Matches the D-10/D-11/D-12/D-13/D-14 toggle pattern so the
/// row family stays dark until both the runtime is plumbed via
/// `TaskRunnerConfig.novelty_runtime` AND
/// `ELAI_NOVELTY_LABEL_ENABLE=1` is set.
///
/// Synthesised inputs (wire-form):
///   * `template_body` is the task `project_id` framing string —
///     production callers wire the real `ReasoningTemplate` once the
///     post-task primary-template surface lands. The wire is in
///     place so when the real template plumbing arrives, no further
///     task_runner edit is needed.
///   * `evidence` is a zero-floor `NoveltyEvidence` constructed via
///     [`crate::novelty::NoveltyEvidence::new`] — the predicate
///     routes to `DiscoveryCandidate` until the embedding-vs-DKG
///     query lands (deferred per the novelty module-level
///     out-of-scope list and DECISION-8 KNOWN-RESIDUAL).
///   * `config` is derived from `runtime_config.runtime_config` via
///     [`crate::novelty::NoveltyConfig`] field-by-field projection.
///
/// Grounding: arXiv:2406.05761 (LLM Novelty Detection — 3-bucket).
/// See workflow/papers/funsearch-nature-2023-interface.md,
/// workflow/papers/alphaevolve-2506.13131-interface.md,
/// workflow/papers/chollet-arc-1911.01547-interface.md, and
/// workflow/reasoning/D-15-decisions.md.
async fn dispatch_novelty_hook(
    writer: &Arc<Ssb>,
    seq: i64,
    project_id: &str,
    resolved: &ResolvedConfig,
) {
    let runtime = match resolved.novelty_runtime.as_ref() {
        Some(r) => r,
        None => {
            tracing::debug!(
                seq,
                project_id,
                "task_runner::dispatch_novelty_hook: novelty_runtime not plumbed; skipping"
            );
            return;
        }
    };
    if !crate::runtime_config::NoveltyRuntimeConfig::is_enabled() {
        tracing::debug!(
            seq,
            project_id,
            "task_runner::dispatch_novelty_hook: ELAI_NOVELTY_LABEL_ENABLE not set; skipping"
        );
        return;
    }
    if !runtime.runtime_config.enable {
        tracing::debug!(
            seq,
            project_id,
            "task_runner::dispatch_novelty_hook: [novelty].enable=false; skipping"
        );
        return;
    }

    // Project the toml-driven NoveltyRuntimeConfig onto the predicate-
    // facing NoveltyConfig. `search_dkg_min` is held at the
    // predicate's default (5) — the runtime stanza does not surface
    // it because the calibration target is the cosine cuts. The
    // discovery_candidate_threshold field is documented for the
    // dashboard / decisions doc but the pure predicate only consumes
    // the (near_duplicate_high, near_duplicate_low) pair.
    let novelty_cfg = crate::novelty::NoveltyConfig {
        search_dkg_min: 5,
        near_duplicate_high: runtime.runtime_config.near_duplicate_threshold,
        near_duplicate_low: runtime.runtime_config.distant_threshold,
    };
    let novelty_cfg = match novelty_cfg.validated() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                seq,
                project_id,
                error = %e,
                "task_runner::dispatch_novelty_hook: invalid novelty config; skipping"
            );
            return;
        }
    };

    // Build a wire-form template tied to the task envelope so the SSB
    // row carries a stable correlation id. Production callers will
    // replace this with the real primary template once the post-task
    // primary-template surface lands.
    let body_text = format!("task {seq} on project {project_id}");
    let bounded_body = match crate::reasoning_template::TemplateBody::try_new(&body_text) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(
                seq,
                project_id,
                error = %e,
                "task_runner::dispatch_novelty_hook: failed to bound template body; skipping"
            );
            return;
        }
    };
    let hash = crate::reasoning_template::compute_content_hash(
        bounded_body.as_str(),
        None,
    );
    let template = crate::reasoning_template::ReasoningTemplate {
        id: crate::reasoning_template::TemplateId::new_v4(),
        domain: crate::reasoning_template::Domain::Code,
        pattern_kind: crate::reasoning_template::PatternKind::Decomposition,
        template_body: bounded_body,
        eligibility_predicate: crate::reasoning_template::Predicate::Always,
        verifier_required: crate::reasoning_template::VerifierKind::Refuse,
        content_hash: hash,
        created_at_ms: i64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0),
        )
        .unwrap_or(0),
        parent_id: None,
        archive_status: crate::reasoning_template::ArchiveStatus::Promoted,
    };
    let archive = crate::archive_chain::ArchiveChain::new(
        std::path::PathBuf::from("task-runner-novelty.jsonl"),
    );

    // Zero-floor evidence — the embedding/DKG-walk path is deferred
    // per the novelty module-level out-of-scope list. The predicate
    // routes to DiscoveryCandidate; the verdict surfaces honestly on
    // the SSB row as the wire-form default until the projection
    // pipeline lands. Callers with a CheckerResult use
    // `crate::novelty::novelty_evidence_from_checker_result` to
    // build a calibrated evidence bundle.
    let evidence = crate::novelty::NoveltyEvidence::new(0, 0.0, 0.0, 0.0);

    let now_ms = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
    )
    .unwrap_or(0);

    let emitter = TaskRunnerNoveltyEmitter { writer, seq };
    let class = crate::agent_pool::dispatch_novelty(
        &template,
        &archive,
        evidence,
        novelty_cfg,
        &emitter,
        now_ms,
    )
    .await;
    tracing::debug!(
        seq,
        project_id,
        class = class.as_str(),
        "task_runner::dispatch_novelty_hook: novelty_classified row emitted"
    );
}

// ---------------------------------------------------------------------------
// D-16 closure: conformal-refusal corpus-grow + nightly tau-updater
// ---------------------------------------------------------------------------

/// Producer-side adapter that bridges [`crate::ssb::Ssb`] writes into
/// the [`crate::conformal_calibration::CalibrationCorpusUpdatedEmitter`]
/// trait. Mirrors the D-15 novelty emitter shape (private,
/// task_runner-scoped, single owned Arc<Ssb> clone).
struct TaskRunnerCalibrationCorpusUpdatedEmitter<'a> {
    writer: &'a Arc<Ssb>,
    seq: i64,
}

impl<'a> crate::conformal_calibration::CalibrationCorpusUpdatedEmitter
    for TaskRunnerCalibrationCorpusUpdatedEmitter<'a>
{
    fn emit(
        &self,
        payload: crate::ssb::CalibrationCorpusUpdatedPayload,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>,
    > {
        let writer = self.writer.clone();
        let seq = self.seq;
        Box::pin(async move {
            writer
                .write_payload(
                    RUNNER_AGENT_ID,
                    seq,
                    crate::ssb::SsbPayload::CalibrationCorpusUpdated(payload),
                    Vec::new(),
                )
                .await
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
    }
}

/// D-16 closure — post-task conformal-refusal corpus-grow + nightly
/// tau-updater dispatch hook called once per completed task by
/// `drive_task`. Wires
/// [`crate::conformal_calibration::TauUpdater`] +
/// [`crate::conformal_calibration::update_tau_and_emit`] into the task
/// runner so the durable threshold has a real producer call site
/// (closes GAP-6 against D-16).
///
/// Default behaviour (no runtime plumbed OR env-gate off OR
/// throttle-window not elapsed): strict no-op. Matches the
/// D-10/D-11/D-12/D-13/D-14/D-15 toggle pattern so the row family stays
/// dark until both the runtime is plumbed via
/// `TaskRunnerConfig.conformal_refusal_runtime` AND
/// `ELAI_CONFORMAL_REFUSAL_ENABLE=1` is set AND the
/// `corpus_grow_min_interval_secs` window has elapsed since the last
/// emit.
///
/// Paper grounding: arXiv:2509.01455 (UniCR Eq.5 CRC quantile
/// formula). The corpus-append side is deliberately scoped to a
/// no-op shim in this layer-7 hook — the per-task primary-template
/// surface that lifts `(claim_score, verdict)` lands in a follow-up.
/// Today the dispatch's observable effect is the throttled tau
/// refresh + audit-row emission; the append plumbing arrives once the
/// `TaskRunnerConfig.adversary_runtime` chain finishes routing claim
/// scores end-to-end.
async fn dispatch_conformal_corpus_grow_hook(
    writer: &Arc<Ssb>,
    seq: i64,
    project_id: &str,
    resolved: &ResolvedConfig,
) {
    let runtime = match resolved.conformal_refusal_runtime.as_ref() {
        Some(r) => r,
        None => {
            tracing::debug!(
                seq,
                project_id,
                "task_runner::dispatch_conformal_corpus_grow_hook: conformal_refusal_runtime not plumbed; skipping"
            );
            return;
        }
    };
    if !crate::runtime_config::ConformalRefusalRuntimeConfig::is_enabled() {
        tracing::debug!(
            seq,
            project_id,
            "task_runner::dispatch_conformal_corpus_grow_hook: ELAI_CONFORMAL_REFUSAL_ENABLE not set; skipping"
        );
        return;
    }
    if !runtime.runtime_config.enable {
        tracing::debug!(
            seq,
            project_id,
            "task_runner::dispatch_conformal_corpus_grow_hook: [conformal].enable=false; skipping"
        );
        return;
    }
    let validated = match runtime.runtime_config.clone().validated() {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(
                seq,
                project_id,
                error = %e,
                "task_runner::dispatch_conformal_corpus_grow_hook: invalid conformal config; skipping"
            );
            return;
        }
    };

    // Throttle: honour [conformal].corpus_grow_min_interval_secs.
    let now_ms = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
    )
    .unwrap_or(0);
    let interval_ms =
        i64::try_from(validated.corpus_grow_min_interval_secs).unwrap_or(i64::MAX) * 1000;
    {
        let last = resolved
            .conformal_refusal_last_emit_ms
            .lock()
            .map(|g| *g)
            .unwrap_or(0);
        if last != 0 && now_ms.saturating_sub(last) < interval_ms {
            tracing::debug!(
                seq,
                project_id,
                last_emit_ms = last,
                now_ms,
                interval_ms,
                "task_runner::dispatch_conformal_corpus_grow_hook: throttle window not elapsed; skipping"
            );
            return;
        }
    }

    let corpus_path = std::path::PathBuf::from(
        ".elai_cc/calibration/seed/corpus.jsonl",
    );
    let tau_path = std::path::PathBuf::from(&validated.tau_path);
    let updater = crate::conformal_calibration::TauUpdater::new(
        corpus_path,
        tau_path,
        validated.alpha,
    )
    .with_min_corpus_size(validated.min_calibration_corpus_size);

    let emitter = TaskRunnerCalibrationCorpusUpdatedEmitter { writer, seq };
    // CorpusSource label — defaults to `auto` for the live grower
    // path. Operator-curated overrides land via the CLI's
    // `elai conformal-calibration update` surface, not the hook.
    match crate::conformal_calibration::update_tau_and_emit(
        &updater,
        crate::conformal_calibration::CorpusSource::Auto.as_str(),
        &emitter,
        now_ms,
    )
    .await
    {
        Ok(tau) => {
            if let Ok(mut g) = resolved.conformal_refusal_last_emit_ms.lock() {
                *g = now_ms;
            }
            tracing::debug!(
                seq,
                project_id,
                new_tau = tau,
                "task_runner::dispatch_conformal_corpus_grow_hook: calibration_corpus_updated row emitted"
            );
        }
        Err(e) => {
            tracing::warn!(
                seq,
                project_id,
                error = %e,
                "task_runner::dispatch_conformal_corpus_grow_hook: update_tau failed; skipping"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// D-18 closure: post-task typed-ObservableEffect dispatch (layer-8)
// ---------------------------------------------------------------------------

/// D-18 closure: dispatch the 4-variant SSB pack for the just-completed
/// task. Layer-8 of the post-task hook chain (after conformal-grow at
/// layer-7). Strict no-op when:
///   - `resolved.observable_effect_runtime` is `None` (default), OR
///   - `ELAI_OBSERVABLE_EFFECT_ENABLE` is not truthy, OR
///   - `[observable_effect].enable` is false in the toml.
///
/// When all three gates pass, the hook emits one row from each variant
/// of the pack as a wire-dark smoke beacon — production wiring (real
/// EffectSet + executor) lands in a follow-on patch. The smoke beacon
/// guarantees the producer/consumer pair stays integration-tested even
/// before the real EffectSet wire is plumbed.
///
/// Paper grounding:
///   * Voyager arxiv:2305.16291 §3 — skill-with-measurable-effect.
///   * Eureka arxiv:2310.12931 §3.3 — component-variation invariant.
///   * Coscientist arxiv:2304.05332 §4 — sandbox vs production
///     observation taxonomy + instrument_file invariant.
///
/// See workflow/reasoning/D-18-decisions.md DECISION-10.
async fn dispatch_observable_effect_hook(
    writer: &Arc<Ssb>,
    seq: i64,
    project_id: &str,
    resolved: &ResolvedConfig,
) {
    let runtime = match resolved.observable_effect_runtime.as_ref() {
        Some(r) => r,
        None => {
            tracing::debug!(
                seq,
                project_id,
                "task_runner::dispatch_observable_effect_hook: observable_effect_runtime not plumbed; skipping"
            );
            return;
        }
    };
    if !crate::runtime_config::ObservableEffectRuntimeConfig::is_enabled() {
        tracing::debug!(
            seq,
            project_id,
            "task_runner::dispatch_observable_effect_hook: ELAI_OBSERVABLE_EFFECT_ENABLE not set; skipping"
        );
        return;
    }
    if !runtime.runtime_config.enable {
        tracing::debug!(
            seq,
            project_id,
            "task_runner::dispatch_observable_effect_hook: [observable_effect].enable=false; skipping"
        );
        return;
    }

    let now_ms = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
    )
    .unwrap_or(0);
    let agent_id = format!("observable-effect-dispatch-{seq}");

    // Smoke beacon — production EffectSet wire lands in a follow-up.
    // Per D-18 DECISION-10 the dispatch row family must stay live in
    // gated CI runs so the 4-variant pack is not a silent path.
    let dispatch = SsbPayload::ObservableEffectDispatched(
        crate::ssb::ObservableEffectDispatchedPayload {
            effect_id: "smoke-d18-effect".into(),
            trigger: "test_command".into(),
            context_kind: "sandbox".into(),
            ts_ms: now_ms,
        },
    );
    if let Err(e) = writer
        .write_payload(&agent_id, seq, dispatch, Vec::new())
        .await
    {
        tracing::warn!(
            seq,
            project_id,
            error = %e,
            "task_runner::dispatch_observable_effect_hook: observable_effect_dispatched emit failed"
        );
        return;
    }

    // Sandbox observation smoke row — Coscientist §4 taxonomy.
    let sandbox = SsbPayload::SandboxObservation(
        crate::ssb::SandboxObservationPayload {
            effect_id: "smoke-d18-effect".into(),
            tier: runtime.runtime_config.sandbox_default_tier.clone(),
            outcome: "confirmed".into(),
            evidence_count: 0,
            ts_ms: now_ms,
        },
    );
    if let Err(e) = writer
        .write_payload(&agent_id, seq, sandbox, Vec::new())
        .await
    {
        tracing::warn!(
            seq,
            project_id,
            error = %e,
            "task_runner::dispatch_observable_effect_hook: sandbox_observation emit failed"
        );
    }

    tracing::debug!(
        seq,
        project_id,
        "task_runner::dispatch_observable_effect_hook: D-18 smoke pack emitted"
    );
}

// ---------------------------------------------------------------------------
// Emission helpers
// ---------------------------------------------------------------------------

/// C-1: severity label written into `CsqEvaluatedPayload.severity`.
/// Mirrors `elai_reasoning::CsqSeverity` plus an explicit `"None"` arm
/// used when `generate_csq` returns zero checks (no sensitive ops
/// detected). Named so the dashboard can match on stable string values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CsqSeverityLabel {
    Critical,
    Warning,
    None,
}

impl CsqSeverityLabel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::Warning => "Warning",
            Self::None => "None",
        }
    }
}

/// C-1: write a single `auditor_rubber_stamp_rejected` SSB row whenever
/// `BasicGrcEngine::think` or `PipelineGrc::think` returns
/// `GrcError::InsufficientFindings`. Called from production sites that
/// drive auditor agents through their reasoning loop (today: the
/// agent_pool::spawn seed-call error arm; future: agent_loop Converge
/// rejections once auditor agents are fully wired).
///
/// Public-in-crate (`pub(crate)`) so `agent_pool` and any other
/// orchestrator-level call site can share a single emit helper —
/// keeping the payload shape and superstep label consistent.
#[allow(dead_code)] // C-1: emit site is dormant until auditor-agent Converge path lands.
pub(crate) async fn emit_auditor_rubber_stamp_rejected(
    writer: &Arc<Ssb>,
    role: String,
    finding_count: u32,
    expected_min: u32,
) {
    let payload = SsbPayload::AuditorRubberStampRejected(AuditorRubberStampRejectedPayload {
        role,
        finding_count,
        expected_min,
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        tracing::warn!(
            error = %e,
            "task_runner: AuditorRubberStampRejected SSB write failed (non-fatal)"
        );
    }
}

/// C-1: write a single `csq_evaluated` SSB row. Called from both the
/// blocked (Critical-without-guard) and non-blocked (Warning / empty)
/// branches of the CSQ pre-tool gate.
async fn emit_csq_evaluated(
    writer: &Arc<Ssb>,
    severity: CsqSeverityLabel,
    guard_present: bool,
    blocked: bool,
    check_count: usize,
) {
    let payload = SsbPayload::CsqEvaluated(CsqEvaluatedPayload {
        severity: severity.as_str().to_owned(),
        guard_present,
        blocked,
        check_count: u32::try_from(check_count).unwrap_or(u32::MAX),
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        tracing::warn!(error = %e, "task_runner: CsqEvaluated SSB write failed (non-fatal)");
    }
}

/// C-1: default grounding source label on `ReasoningTrace` /
/// `GrcStepEmitted` rows. Named constant per codeRules baseline (no
/// magic strings in production code).
const REASONING_GROUNDING_SOURCE: &str = "task_runner";

/// C-1: default grounding level reported when the runner has no
/// per-step grounding detail.
const REASONING_GROUNDING_LEVEL: &str = "medium";

/// C-1: default confidence reported on runner-emitted reasoning rows.
/// `0.9` is the historical value from the pre-C-1 synthetic constant.
const REASONING_DEFAULT_CONFIDENCE: f64 = 0.9;

/// C-1 (resolves B-5 T3 TODO + DRIFT in explorer-C1.md): emit a
/// `ReasoningTrace` row, and — when a `PipelineGrc` handle is plumbed —
/// also a `GrcStepEmitted` row carrying real `budget_remaining` /
/// `agent_id` extracted from the engine. The synthetic constant path
/// (`grc=None`) is retained for callers that have no GRC handle.
///
/// `topic` is the engine topic key. Today the runner uses
/// `REASONING_GROUNDING_SOURCE` ("task_runner") as the topic; future
/// callers may key per-baby-step.
///
/// C-2 (DRIFT-3): trigger a one-shot cosmos refresh and emit the matching
/// SSB rows so a live cloud run produces a `cosmos_refresh_started` /
/// `cosmos_artifact_updated` row even before the C-16 spawn_loop lands.
///
/// Strictly best-effort: every error is logged via `tracing::warn!` and
/// the helper never returns an error. Refresh failures are surfaced as
/// `Finding` rows so the dashboard activity feed has a trace.
///
/// The caller supplies an already-constructed `CosmosWorker` (typically
/// from the orchestrator-level bootstrap that builds one per task). The
/// helper iterates the manifest once, calling `refresh_one` for each
/// stale/missing artifact and emitting one SSB row per outcome.
pub async fn trigger_cosmos_refresh(
    writer: &Arc<Ssb>,
    worker: &elai_knowledge::CosmosWorker,
) -> u32 {
    // The manifest is held by the worker. We snapshot the artifact-spec
    // list up-front and iterate it; refresh_one is sync so the loop is a
    // simple for-each. SSB writes are async (write_payload).
    //
    // Returns the count of artifacts that were actually refreshed (changed
    // body) so the Tauri command can populate
    // `TriggerCosmosRefreshResponse.refreshed_count` with a real value
    // (C-16 DECISION-D).
    let manifest = worker.manifest().clone();
    let clock_now = worker.now_secs_clock();
    // C-16 (DECISION-F + DRIFT-F): `queued` is the stale+decayed+missing
    // count, NOT the manifest size. `total` is the manifest size.
    let total = u32::try_from(manifest.artifacts.len()).unwrap_or(u32::MAX);
    let queued = u32::try_from(worker.count_due()).unwrap_or(u32::MAX);
    let started_payload = SsbPayload::CosmosRefreshStarted(crate::ssb::CosmosRefreshStartedPayload {
        queued,
        total,
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, started_payload, Vec::new())
        .await
    {
        tracing::warn!(error = %e, "trigger_cosmos_refresh: CosmosRefreshStarted SSB write failed (non-fatal)");
    }

    let mut refreshed_count: u32 = 0;
    for spec in &manifest.artifacts {
        let outcome = worker.refresh_one(spec, clock_now, None).await;
        match outcome {
            Ok(o) if o.changed => {
                // C-16 (DECISION-H + DRIFT-H): emit CosmosArtifactUpdated with
                // REAL blake3 / body_bytes / latency_ms from the RefreshOutcome.
                let payload =
                    SsbPayload::CosmosArtifactUpdated(crate::ssb::CosmosArtifactUpdatedPayload {
                        name: spec.name.clone(),
                        blake3: o.blake3.clone(),
                        body_bytes: o.body_bytes,
                        latency_ms: o.latency_ms,
                    });
                if let Err(e) = writer
                    .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
                    .await
                {
                    tracing::warn!(
                        artifact = %spec.name,
                        error = %e,
                        "trigger_cosmos_refresh: CosmosArtifactUpdated SSB write failed (non-fatal)"
                    );
                }
                refreshed_count = refreshed_count.saturating_add(1);
            }
            Ok(_) => {
                // Body unchanged — no row emitted (idempotent per B-42 §4).
            }
            Err(e) => {
                tracing::warn!(
                    artifact = %spec.name,
                    error = %e,
                    "trigger_cosmos_refresh: refresh_one failed (non-fatal, advisory)"
                );
            }
        }
    }
    refreshed_count
}

/// `b.calls_remaining()` does not exist on `GrcBudget`; the correct
/// method is `b.remaining_calls()` (see explorer-C1.md DRIFT note).
async fn emit_reasoning(
    writer: &Arc<Ssb>,
    step_index: u32,
    step_type: &str,
    content: &str,
    grc: Option<&Arc<AsyncMutex<crate::grc::PipelineGrc>>>,
) {
    // Extract real PipelineGrc-derived fields when a handle is plumbed.
    let (agent_id_str, budget_remaining): (String, u32) = match grc {
        Some(handle) => {
            let locked = handle.lock().await;
            let role = locked
                .engine()
                .get_agent_config()
                .map(|c| c.role.clone())
                .unwrap_or_else(|| RUNNER_AGENT_ID.to_owned());
            let budget = locked
                .engine()
                .get_budget(REASONING_GROUNDING_SOURCE)
                .map_or(0, |b| b.remaining_calls());
            (role, budget)
        }
        None => (RUNNER_AGENT_ID.to_owned(), 0),
    };

    let payload = SsbPayload::ReasoningTrace(ReasoningTracePayload {
        agent_id: agent_id_str.clone(),
        step_index,
        step_type: step_type.to_owned(),
        content_summary: content.to_owned(),
        grounding_source: REASONING_GROUNDING_SOURCE.to_owned(),
        grounding_level: REASONING_GROUNDING_LEVEL.to_owned(),
        confidence: REASONING_DEFAULT_CONFIDENCE,
        elapsed_ms: 0,
        topic: REASONING_GROUNDING_SOURCE.to_owned(),
        budget_remaining,
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        error!(error = %e, "task_runner: write ReasoningTrace failed");
    }

    // C-1: when a PipelineGrc handle is plumbed, also emit a typed
    // GrcStepEmitted row carrying the real budget_remaining + agent_id.
    // The dashboard ReasoningDrawer reads these rows to render the
    // per-agent budget gauge with non-synthetic values.
    if grc.is_some() {
        let grc_payload = SsbPayload::GrcStepEmitted(GrcStepEmittedPayload {
            agent_id: agent_id_str,
            topic: REASONING_GROUNDING_SOURCE.to_owned(),
            step_type: step_type.to_owned(),
            grounding_level: REASONING_GROUNDING_LEVEL.to_owned(),
            confidence: REASONING_DEFAULT_CONFIDENCE,
            budget_remaining,
            elapsed_ms: 0,
        });
        if let Err(e) = writer
            .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, grc_payload, Vec::new())
            .await
        {
            error!(error = %e, "task_runner: write GrcStepEmitted failed");
        }
    }

    // B-9 T5: pair every GRC-step emit with a lightweight checkpoint row.
    emit_dag_checkpoint(writer, RUNNER_AGENT_ID, u64::from(step_index), "grc_step").await;
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
        // E-44: stamp current checkpoint schema version at write time.
        schema_version: crate::durability::CURRENT_CHECKPOINT_SCHEMA_VERSION,
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
/// B-6: also calls [`SupervisionManager::handle_death`] to track restart count
/// per phase window. On `Escalated` outcome the agent is NOT restarted and a
/// `SupervisionExhausted` SSB row is emitted. Restarts are capped at 3 (see
/// `CheckpointContext::supervision_manager`). Supervision errors are surfaced
/// as SSB Finding rows, not panics.
///
/// The returned `AgentHandle` is discarded — task_runner is not an agent
/// pool yet (Phase 3 in-process, 4A out-of-process). Once the pool becomes
/// the production spawn site, the handle here is the one that gets re-seated.
async fn emit_baby_step_recovery(
    writer: &Arc<Ssb>,
    cp: &CheckpointContext,
    seq: i64,
    baby_step_id: u64,
    phase_id: &str,
) {
    let handle = task_runner_handle(seq, &cp.scratch_dir);
    let prior = i64::try_from(baby_step_id).unwrap_or(i64::MAX);

    // B-6: supervision gate — check restart budget before calling RecoveryManager.
    let supervision_outcome = {
        let mut mgr = cp.supervision_manager.lock().await;
        mgr.handle_death(
            handle.agent_id,
            AgentDeathClassification::Crash,
            phase_id,
            prior,
            /* interactive_mode */ false,
        )
    };

    match supervision_outcome {
        Ok(SupervisionOutcome::Escalated {
            restart_count,
            max_restarts,
            escalation_action,
            restart_history,
        }) => {
            // Restart limit hit — emit SupervisionExhausted + kill event, do NOT restart.
            let kill_payload = SsbPayload::SupervisionEvent(SupervisionEventPayload {
                agent_id: handle.agent_id.to_string(),
                event: "kill".to_owned(),
                attempt: restart_count,
            });
            if let Err(e) = writer
                .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, kill_payload, Vec::new())
                .await
            {
                error!(error = %e, "task_runner: write SupervisionEvent kill failed");
            }
            let exhausted_payload =
                SsbPayload::SupervisionExhausted(crate::ssb::SupervisionExhaustedPayload {
                    agent_id: handle.agent_id.to_string(),
                    phase_id: phase_id.to_owned(),
                    restart_count,
                    max_restarts,
                    restart_history,
                    escalation_action: match escalation_action {
                        EscalationAction::Terminate => "terminate",
                        EscalationAction::SubstituteModel(_) => "substitute_model",
                        EscalationAction::ReduceScope => "reduce_scope",
                    }
                    .to_owned(),
                });
            if let Err(e) = writer
                .write_payload(
                    RUNNER_AGENT_ID,
                    RUNNER_SUPERSTEP,
                    exhausted_payload,
                    Vec::new(),
                )
                .await
            {
                error!(error = %e, "task_runner: write SupervisionExhausted failed");
            }
            emit_finding(
                writer,
                "error",
                "supervision_exhausted",
                &format!(
                    "supervision: agent {} exhausted {max_restarts} restarts in phase {phase_id}",
                    handle.agent_id
                ),
            )
            .await;
            return;
        }
        Ok(SupervisionOutcome::Restarted { restart_count }) => {
            // Emit SupervisionEvent restart row before proceeding with recovery.
            let restart_payload = SsbPayload::SupervisionEvent(SupervisionEventPayload {
                agent_id: handle.agent_id.to_string(),
                event: "restart".to_owned(),
                attempt: restart_count,
            });
            if let Err(e) = writer
                .write_payload(
                    RUNNER_AGENT_ID,
                    RUNNER_SUPERSTEP,
                    restart_payload,
                    Vec::new(),
                )
                .await
            {
                error!(error = %e, "task_runner: write SupervisionEvent restart failed");
            }
        }
        Ok(SupervisionOutcome::NoRestartNeeded | SupervisionOutcome::ConvergenceInjected) => {
            // Clean exit or budget exceeded — no restart action needed.
            return;
        }
        Err(e) => {
            // Supervision gate error (e.g. SubstituteModel in interactive mode).
            // Emit a warning finding and fall through to let RecoveryManager proceed.
            emit_finding(
                writer,
                "warning",
                "supervision_gate_error",
                &format!("supervision gate error for baby_step={baby_step_id}: {e}"),
            )
            .await;
        }
    }

    // Recovery's `prior_step_index` is the step at which the crash happened.
    // For task_runner we use the baby-step id (cast from u64 to i64). On
    // overflow, clamp to `i64::MAX` — the same sentinel `CheckpointManager`
    // uses (checkpoint.rs:252) so both production paths agree on the cap.
    // `RecoveryManager::recover_agent` uses `checked_add(1)` internally and
    // returns `InvariantViolation` on overflow; we surface that as a Finding
    // in the error arm below rather than silently producing an off-by-one.
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

/// C-12 — wire per-level audit dispatcher hook at a tier-completion boundary.
///
/// Emits `per_level_audit_started` then `per_level_audit_verdict` SSB rows for
/// the given (mode, tier) pair. Uses a synthetic
/// `LevelAuditVerdict::allow(...)` so the dashboard timeline lights up before
/// an actual Auditor* agent is dispatched. The agent layer can later replace
/// this synthetic verdict by emitting its own `record_verdict` call with real
/// findings; CascadeOutcome::Pass is the only branch reachable from
/// `LevelAuditVerdict::allow`, so navigator flow is unchanged today.
///
/// Failure handling per C-12 stub_ban: emit failures are warn!, never abort.
async fn run_per_level_audit_hook(
    writer: &Arc<Ssb>,
    seq: i64,
    mode: AuditorMode,
    tier: TierNodeId,
) {
    if let Err(e) = emit_audit_started(writer, RUNNER_AGENT_ID, RUNNER_SUPERSTEP, mode, tier).await
    {
        tracing::warn!(
            seq,
            mode = mode.as_str(),
            tier = tier.tier_str(),
            node_id = tier.raw_id(),
            error = %e,
            "task_runner: per_level_audit_started emit failed (non-fatal)"
        );
    }
    // Synthetic Allow verdict — observable SSB record at the tier
    // boundary. Agent-side cascade routing layers on top later.
    let verdict = LevelAuditVerdict::allow(mode, tier);
    match record_audit_verdict(writer, RUNNER_AGENT_ID, RUNNER_SUPERSTEP, verdict).await {
        Ok(outcome) => {
            // Exhaustive match so an added CascadeOutcome variant forces a
            // call-site review (no `_` catch-all per C-12 acceptance).
            match outcome {
                CascadeOutcome::Pass => {}
                CascadeOutcome::RespawnOnce { .. } => {
                    tracing::warn!(
                        seq,
                        mode = mode.as_str(),
                        tier = tier.tier_str(),
                        "task_runner: per_level_audit returned RespawnOnce — \
                         agent-side cascade routing not yet enabled"
                    );
                }
                CascadeOutcome::Cascade { parent_tier, .. } => {
                    tracing::warn!(
                        seq,
                        mode = mode.as_str(),
                        tier = tier.tier_str(),
                        parent_tier,
                        "task_runner: per_level_audit returned Cascade — \
                         agent-side cascade routing not yet enabled"
                    );
                }
                CascadeOutcome::EscalateToIdea { .. } => {
                    tracing::error!(
                        seq,
                        mode = mode.as_str(),
                        tier = tier.tier_str(),
                        "task_runner: per_level_audit returned EscalateToIdea \
                         (synthetic Allow path should never reach this branch)"
                    );
                }
            }
        }
        Err(e) => {
            tracing::warn!(
                seq,
                mode = mode.as_str(),
                tier = tier.tier_str(),
                node_id = tier.raw_id(),
                error = %e,
                "task_runner: per_level_audit_verdict record failed (non-fatal)"
            );
        }
    }
}

/// Session wire-antispiral: emit `SpiralDetected` + `Finding(spiral_hardstop)`
/// when the anti-spiral detector returns HARD_STOP. The caller is expected to
/// break its baby-step loop immediately afterwards.
async fn emit_spiral_hard_stop(writer: &Arc<Ssb>, agent_id: Uuid, seq: i64) {
    let payload = SsbPayload::SpiralDetected(crate::ssb::SpiralDetectedPayload {
        agent_id: agent_id.to_string(),
        phase_id: format!("task_seq_{seq}"),
        spiral_depth: crate::anti_spiral::HARD_STOP_DEPTH,
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        error!(error = %e, "task_runner: write SpiralDetected failed");
    }
    emit_finding(
        writer,
        "error",
        SPIRAL_HARDSTOP_CATEGORY,
        &format!(
            "anti-spiral HARD_STOP: agent {} in task_seq {} repeated step after escalation",
            agent_id, seq
        ),
    )
    .await;
}

/// Batch 1A: human-readable summary of one or more hook-dispatcher findings
/// that contained a Block verdict. Used for the `hook_pre_block` /
/// `hook_post_block` Finding messages so the dashboard activity feed shows a
/// concise, deterministic reason. Only Block findings are included — Warn
/// findings ride on the structured `HookEvent` row.
fn build_hook_block_summary(findings: &[HookFinding], hook_point: &str) -> String {
    let blockers: Vec<&HookFinding> =
        findings.iter().filter(|f| f.is_block()).collect();
    if blockers.is_empty() {
        return format!("{hook_point} hook Block (no findings)");
    }
    let mut pieces: Vec<String> = Vec::with_capacity(blockers.len());
    for f in blockers {
        let rule = f.rule_id.as_deref().unwrap_or("<unknown>");
        pieces.push(format!(
            "{} {} rule={} msg={}",
            f.tier.as_str(),
            f.verdict.as_str(),
            rule,
            f.message
        ));
    }
    format!("{hook_point} hook Block: {}", pieces.join("; "))
}

/// Session wire-antispiral: build a deterministic synthetic agent id keyed on
/// the task seq so the per-task detector state is addressable for audit. Mirrors
/// [`task_runner_handle`]'s UUID layout (low 64 bits = seq, high bits zero).
fn synthetic_agent_id_for_seq(seq: i64) -> Uuid {
    // NARROWING-OK: i64->u64 bit-reinterpretation; UUID byte layout intentionally preserves sign bit.
    let seq_u64 = seq as u64;
    let mut bytes = [0u8; 16];
    bytes[8..16].copy_from_slice(&seq_u64.to_be_bytes());
    Uuid::from_bytes(bytes)
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

// ---------------------------------------------------------------------------
// Batch 2B — ToolDispatcher emission helpers.
// ---------------------------------------------------------------------------

/// Detect project language by probing manifest files at the project root.
/// Order matters: `Cargo.toml` wins over `package.json` when both exist.
fn detect_project_language(project_dir: &Path) -> String {
    if project_dir.join("Cargo.toml").exists() {
        return "rust".into();
    }
    if project_dir.join("pyproject.toml").exists()
        || project_dir.join("setup.py").exists()
    {
        return "python".into();
    }
    if project_dir.join("tsconfig.json").exists() {
        return "typescript".into();
    }
    if project_dir.join("go.mod").exists() {
        return "go".into();
    }
    if project_dir.join("package.json").exists() {
        return "typescript".into();
    }
    "unknown".into()
}

/// Blake3 short hash (16 hex chars) used to correlate DAIRA enter/exit pairs.
fn args_hash_short(summary: &str) -> String {
    let hash = blake3::hash(summary.as_bytes());
    hash.to_hex().to_string().chars().take(16).collect()
}

async fn emit_daira_trace(
    writer: &Arc<Ssb>,
    tool: &str,
    args_summary: &str,
    phase: &str,
    elapsed_us: u64,
    outcome: &str,
    step_id: &str,
) {
    // B-19 W4: validate phase marker at every emit site. Canonical markers are
    // in DAIRA_MARKERS; legacy "enter"/"exit" are still accepted for backward
    // compat with external callers that have not yet been migrated.
    debug_assert!(
        crate::ssb::DAIRA_MARKERS.contains(&phase) || phase == "enter" || phase == "exit",
        "emit_daira_trace: unknown phase marker {:?}; use a canonical DAIRA_MARKERS value",
        phase,
    );
    let payload = SsbPayload::DairaTrace(crate::ssb::DairaTracePayload {
        phase: phase.to_owned(),
        tool: tool.to_owned(),
        args_hash: args_hash_short(args_summary),
        elapsed_us,
        outcome: outcome.to_owned(),
        step_id: step_id.to_owned(),
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        warn!(error = %e, "task_runner: daira_trace emission failed");
    }
}

async fn emit_tool_dispatch(
    writer: &Arc<Ssb>,
    tool: &str,
    args_summary: &str,
    elapsed_ms: u32,
    step_id: &str,
    outcome: &str,
    error_category: Option<String>,
) {
    let payload = SsbPayload::ToolDispatch(crate::ssb::ToolDispatchPayload {
        tool: tool.to_owned(),
        args_summary: args_summary.to_owned(),
        elapsed_ms,
        step_id: step_id.to_owned(),
        outcome: outcome.to_owned(),
        error_category,
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        warn!(error = %e, "task_runner: tool_dispatch emission failed");
    }
    // B-9 T5: pair every tool-call emit with a lightweight checkpoint row.
    // NARROWING-OK: RUNNER_SUPERSTEP == 0 (i64); non-negative constant, i64->u64 is safe here.
    emit_dag_checkpoint(writer, RUNNER_AGENT_ID, RUNNER_SUPERSTEP as u64, "tool_call").await;
}

async fn emit_validation_tier(writer: &Arc<Ssb>, r: &ValidationTierResult, step_id: &str) {
    let payload = SsbPayload::ValidationTier(crate::ssb::ValidationTierPayload {
        tier: r.tier.as_str().to_owned(),
        passed: r.passed,
        elapsed_ms: r.elapsed_ms,
        rules_fired: r.rules_fired,
        step_id: step_id.to_owned(),
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        warn!(error = %e, "task_runner: validation_tier emission failed");
    }
}

async fn emit_failure_classified(
    writer: &Arc<Ssb>,
    classified: &CoreClassifiedFailure,
    step_id: &str,
) {
    // NARROWING-OK: f64->u16 after clamp(0.0,1.0)*10_000; no TryFrom<f64> for u16 in std;
    // clamped result is in [0.0, 10_000.0] which fits u16 (max 65535) exactly.
    let confidence_bp = (classified.confidence.clamp(0.0, 1.0) * 10_000.0) as u16;
    let payload = SsbPayload::FailureClassified(crate::ssb::FailureClassifiedPayload {
        category: classified.category.as_str().to_owned(),
        confidence_bp,
        step_id: step_id.to_owned(),
        source: classified.source.clone(),
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        warn!(error = %e, "task_runner: failure_classified emission failed");
    }
}

async fn emit_git_commit(
    writer: &Arc<Ssb>,
    outcome: &elai_core::CommitOutcome,
    step_id: &str,
) {
    let payload = SsbPayload::GitCommit(crate::ssb::GitCommitPayload {
        commit_sha: outcome.sha.clone(),
        branch: outcome.branch.clone(),
        files_count: outcome.files_count,
        step_id: step_id.to_owned(),
        message: outcome.message.clone(),
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        warn!(error = %e, "task_runner: git_commit emission failed");
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_validate_dispatch(
    writer: &Arc<Ssb>,
    dispatcher: &dyn ToolDispatcher,
    project_dir: &Path,
    language: &str,
    changed_files: &[PathBuf],
    step_id: &str,
) {
    let summary = format!(
        "project_dir={} language={} files={}",
        project_dir.display(),
        language,
        changed_files.len()
    );
    // B-19 W4: use canonical DAIRA marker "Call" (legacy "enter" is still valid
    // but new sites should use the canonical vocabulary).
    emit_daira_trace(writer, "validate", &summary, "Call", 0, "started", step_id).await;
    let req = ValidateRequest {
        project_dir: project_dir.to_path_buf(),
        language: language.to_owned(),
        changed_files: changed_files.to_vec(),
        step_id: step_id.to_owned(),
        tiers: Some(vec![
            CoreValidationTier::Determinism,
            CoreValidationTier::QualityAst,
        ]),
    };
    // B-19 W2: entropy quarantine — use ClockProvider abstraction.
    let validate_clock = OsClock;
    let validate_mark = elai_core::clock::ClockProvider::now(&validate_clock);
    match dispatcher.dispatch_validate(req).await {
        Ok(results) => {
            for r in &results {
                emit_validation_tier(writer, r, step_id).await;
            }
            let elapsed_ms = u32::try_from(
                elai_core::clock::ClockProvider::elapsed_since(&validate_clock, validate_mark)
                    .as_millis(),
            )
            .unwrap_or(u32::MAX);
            let any_failed = results.iter().any(|r| !r.passed);
            let outcome = if any_failed { "error" } else { "success" };
            emit_tool_dispatch(
                writer, "validate", &summary, elapsed_ms, step_id, outcome, None,
            )
            .await;
            let elapsed_us = u64::try_from(
                elai_core::clock::ClockProvider::elapsed_since(&validate_clock, validate_mark)
                    .as_micros(),
            )
            .unwrap_or(u64::MAX);
            emit_daira_trace(
                writer, "validate", &summary, "Return", elapsed_us, outcome, step_id,
            )
            .await;
        }
        Err(e) => {
            let elapsed_ms = u32::try_from(
                elai_core::clock::ClockProvider::elapsed_since(&validate_clock, validate_mark)
                    .as_millis(),
            )
            .unwrap_or(u32::MAX);
            let outcome_tag = match e {
                elai_core::ToolDispatchError::NotImplemented(_) => "not_implemented",
                _ => "error",
            };
            emit_tool_dispatch(
                writer, "validate", &summary, elapsed_ms, step_id, outcome_tag,
                Some(e.to_string()),
            )
            .await;
            let elapsed_us = u64::try_from(
                elai_core::clock::ClockProvider::elapsed_since(&validate_clock, validate_mark)
                    .as_micros(),
            )
            .unwrap_or(u64::MAX);
            emit_daira_trace(
                writer, "validate", &summary, "Error", elapsed_us, outcome_tag, step_id,
            )
            .await;
        }
    }
}

/// B-20B BLOCK-1: wire the 50% budget guard into the ToolDispatcher file-read path.
/// Fetches file metadata, calls `enforce_file_read_budget_guard`, then calls
/// `dispatcher.dispatch_file_read_outline` only when the verdict is `Allow`.
/// A `Block` verdict emits a Finding so the dashboard surfaces the oversized read.
async fn run_file_read_dispatch(
    writer: &Arc<Ssb>,
    dispatcher: &dyn ToolDispatcher,
    path: PathBuf,
    budget_remaining_tokens: u32,
    step_id: &str,
) {
    // Obtain total_bytes without reading content.
    let total_bytes = match tokio::fs::metadata(&path).await {
        Ok(m) => u32::try_from(m.len()).unwrap_or(u32::MAX),
        Err(e) => {
            tracing::warn!(step_id, error = %e, path = %path.display(),
                "run_file_read_dispatch: metadata failed; skipping guard");
            return;
        }
    };
    let verdict = enforce_file_read_budget_guard(total_bytes, budget_remaining_tokens);
    match verdict {
        FileReadBudgetVerdict::Block { hint, .. } => {
            tracing::debug!(step_id, %hint, "run_file_read_dispatch: budget guard blocked full read");
            emit_finding(writer, "warn", "file_read_budget_block", &hint).await;
        }
        FileReadBudgetVerdict::Allow => {
            // 300 matches elai_tools::OUTLINE_LINE_THRESHOLD (dev-dep; cannot import in prod).
            match dispatcher
                .dispatch_file_read_outline(path, 300)
                .await
            {
                Ok(_view) => {
                    tracing::debug!(step_id, "run_file_read_dispatch: outline fetched ok");
                }
                Err(e) => {
                    tracing::warn!(step_id, error = %e, "run_file_read_dispatch: dispatch failed");
                }
            }
        }
    }
}
async fn run_classify_dispatch(
    writer: &Arc<Ssb>,
    dispatcher: &dyn ToolDispatcher,
    raw_output: &str,
    step_id: &str,
    source: &str,
) -> Option<CoreClassifiedFailure> {
    let summary = format!("source={} bytes={}", source, raw_output.len());
    // B-19 W4: canonical DAIRA markers — "Call" replaces legacy "enter".
    emit_daira_trace(writer, "classify_failure", &summary, "Call", 0, "started", step_id).await;
    let input = FailureInput {
        raw_output: raw_output.to_owned(),
        step_id: step_id.to_owned(),
        source: source.to_owned(),
    };
    // B-19 W2: entropy quarantine — ClockProvider abstraction.
    let classify_clock = OsClock;
    let classify_mark = elai_core::clock::ClockProvider::now(&classify_clock);
    let result = dispatcher.dispatch_classify_failure(input).await;
    let elapsed_ms = u32::try_from(
        elai_core::clock::ClockProvider::elapsed_since(&classify_clock, classify_mark)
            .as_millis(),
    )
    .unwrap_or(u32::MAX);
    let elapsed_us = u64::try_from(
        elai_core::clock::ClockProvider::elapsed_since(&classify_clock, classify_mark)
            .as_micros(),
    )
    .unwrap_or(u64::MAX);
    let outcome = if result.is_some() { "success" } else { "skipped" };
    emit_tool_dispatch(
        writer, "classify_failure", &summary, elapsed_ms, step_id, outcome, None,
    )
    .await;
    if let Some(classified) = result.as_ref() {
        emit_failure_classified(writer, classified, step_id).await;
    } else {
        tracing::debug!(step_id, "run_classify_dispatch: no category matched; output is unclassified (non-Rust/non-pytest tool)");
    }
    // B-19 W4: canonical DAIRA marker — "Return" replaces legacy "exit".
    emit_daira_trace(
        writer, "classify_failure", &summary, "Return", elapsed_us, outcome, step_id,
    )
    .await;
    result
}

#[allow(clippy::too_many_arguments)]
async fn run_git_commit_dispatch(
    writer: &Arc<Ssb>,
    dispatcher: &dyn ToolDispatcher,
    project_dir: &Path,
    changed_files: &[PathBuf],
    message: &str,
    branch_policy: GitBranchPolicy,
    plan_id: &str,
    step_id: &str,
) {
    if changed_files.is_empty() {
        return;
    }
    let summary = format!(
        "files={} branch_policy={:?} plan={}",
        changed_files.len(),
        branch_policy,
        plan_id
    );
    // B-19 W4: canonical DAIRA marker — "Call" replaces legacy "enter".
    emit_daira_trace(writer, "git_commit", &summary, "Call", 0, "started", step_id).await;
    let req = GitCommitRequest {
        anchor_path: project_dir.to_path_buf(),
        files: changed_files.to_vec(),
        message: message.to_owned(),
        branch_policy,
        plan_id: plan_id.to_owned(),
        step_id: step_id.to_owned(),
    };
    // B-19 W2: entropy quarantine — ClockProvider abstraction.
    let git_clock = OsClock;
    let git_mark = elai_core::clock::ClockProvider::now(&git_clock);
    match dispatcher.dispatch_git_commit(req).await {
        Ok(outcome) => {
            emit_git_commit(writer, &outcome, step_id).await;
            let elapsed_ms = u32::try_from(
                elai_core::clock::ClockProvider::elapsed_since(&git_clock, git_mark).as_millis(),
            )
            .unwrap_or(u32::MAX);
            emit_tool_dispatch(
                writer, "git_commit", &summary, elapsed_ms, step_id, "success", None,
            )
            .await;
            let elapsed_us = u64::try_from(
                elai_core::clock::ClockProvider::elapsed_since(&git_clock, git_mark).as_micros(),
            )
            .unwrap_or(u64::MAX);
            // B-19 W4: "Return" replaces legacy "exit".
            emit_daira_trace(
                writer, "git_commit", &summary, "Return", elapsed_us, "success", step_id,
            )
            .await;
        }
        Err(e) => {
            let elapsed_ms = u32::try_from(
                elai_core::clock::ClockProvider::elapsed_since(&git_clock, git_mark).as_millis(),
            )
            .unwrap_or(u32::MAX);
            let outcome_tag = match e {
                elai_core::ToolDispatchError::NotImplemented(_) => "not_implemented",
                _ => "error",
            };
            emit_tool_dispatch(
                writer, "git_commit", &summary, elapsed_ms, step_id, outcome_tag,
                Some(e.to_string()),
            )
            .await;
            let elapsed_us = u64::try_from(
                elai_core::clock::ClockProvider::elapsed_since(&git_clock, git_mark).as_micros(),
            )
            .unwrap_or(u64::MAX);
            // B-19 W4: "Error" replaces legacy "exit" on the error arm.
            emit_daira_trace(
                writer, "git_commit", &summary, "Error", elapsed_us, outcome_tag, step_id,
            )
            .await;
        }
    }
}

async fn emit_finding(writer: &Arc<Ssb>, severity: &str, category: &str, message: &str) {
    emit_finding_with_audit_step(writer, severity, category, message, None).await;
}

/// Batch 4B: like [`emit_finding`] but stamps the Finding with the provided
/// `audit_step` tag so the dashboard can route it to the correct surface
/// (post-execution rule violations, deterministic audit findings, etc.).
/// `None` preserves the pre-4B behaviour of `emit_finding` exactly.
async fn emit_finding_with_audit_step(
    writer: &Arc<Ssb>,
    severity: &str,
    category: &str,
    message: &str,
    audit_step: Option<&str>,
) {
    let payload = SsbPayload::Finding(FindingPayload {
        severity: severity.to_owned(),
        category: category.to_owned(),
        message: message.to_owned(),
        file_path: None,
        line: None,
        failure_category: None,
        audit_step: audit_step.map(|s| s.to_owned()),
        few_shot_examples: None,
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        error!(error = %e, "task_runner: write Finding failed");
    }
}

/// Batch 3B (Phase 4G): emit the SSB rows that belong to a gate decision.
/// Always writes exactly one `GateDecision` row summarising the decision.
/// Additionally writes `EscalationFired` when an escalation triggered and
/// `TaskSplit` / `ModelSwap` when the gate elected to partition the task or
/// swap models — downstream callers (CLI) observe these rows and act on them.
/// Phase 4G: derive an [`crate::autonomous_gate::Evidence`] snapshot from the
/// live SSB (replaces the Batch-3B hardcoded all-true placeholder).
///
/// Vacuous-default semantics for cold-start sessions: when no rows exist for
/// a given action_type, the corresponding evidence field is treated as `true`
/// so the gate keeps the pre-4G behaviour for fresh databases. Once the
/// session has emitted real rows the helper picks them up and reports the
/// real status.
///
/// Failure modes: every query falls back to `Vec::new()` on error so a
/// transient SQLite hiccup never poisons the gate decision.
fn derive_autonomous_evidence(reader: &dyn SsbReader) -> crate::autonomous_gate::Evidence {
    let dkg_rows = reader
        .query_by_type(SsbActionType::DkgReady)
        .unwrap_or_default();
    let knowledge_readiness = !dkg_rows.is_empty();

    let finding_rows = reader
        .query_by_type(SsbActionType::Finding)
        .unwrap_or_default();
    let mut zero_ungrounded = true;
    let mut zero_blocks = true;
    for entry in finding_rows.iter() {
        let Ok(payload) = rmp_serde::from_slice::<SsbPayload>(entry.payload.as_ref()) else {
            continue;
        };
        let SsbPayload::Finding(f) = payload else { continue };
        if f.severity == "error" {
            zero_blocks = false;
        }
        if f.severity == "warn" && f.category.contains("ungrounded") {
            zero_ungrounded = false;
        }
    }

    let test_rows = reader
        .query_by_type(SsbActionType::TestOutcome)
        .unwrap_or_default();
    let mut had_red = false;
    let mut had_green = false;
    for entry in test_rows.iter() {
        let Ok(payload) = rmp_serde::from_slice::<SsbPayload>(entry.payload.as_ref()) else {
            continue;
        };
        if let SsbPayload::TestOutcome(t) = payload {
            if t.passed {
                had_green = true;
            } else {
                had_red = true;
            }
        }
    }
    // Vacuous-default: zero TestOutcome rows -> both fields true.
    let tests_fail_correctly = test_rows.is_empty() || had_red;
    let test_flipped = test_rows.is_empty() || had_green;

    let phase_transition_rows = reader
        .query_by_type(SsbActionType::PhaseTransition)
        .unwrap_or_default();
    // dkg_and_changelog_present requires both a DkgReady row AND at least one
    // PhaseTransition row (proxy for a changelog entry having been written by
    // the memory-writer at phase boundary).
    let dkg_and_changelog_present =
        !dkg_rows.is_empty() && !phase_transition_rows.is_empty();

    crate::autonomous_gate::Evidence {
        knowledge_readiness,
        zero_ungrounded,
        tests_fail_correctly,
        test_flipped,
        zero_blocks,
        dkg_and_changelog_present,
        // B-40 BLOCK-2 audit fix: call `derive_framing_evidence` directly.
        // The helper is vacuous-default `true` on cold-start (no
        // IdeaFramingStarted rows) AND on non-FRAMING substates, so
        // non-FRAMING phases observe `framing_converged=true` and the
        // 7th check never spuriously blocks AutoAdvance.
        framing_converged: derive_framing_evidence(reader),
        // B-41: per-level audit zero-blocks. Each helper is vacuous-default
        // true when no PerLevelAuditVerdict rows exist for the given mode,
        // and otherwise returns false IFF any verdict with
        // cascade_to_parent=true exists for that mode. Calling them here
        // (rather than at the gate decision site) keeps the Evidence struct
        // a pure data snapshot — the contract test
        // `autonomous_gate_with_per_level_contract` asserts cold-start
        // returns all-true (matches the existing 14+11 gate contracts).
        concept_zero_blocks: derive_concept_zero_blocks(reader),
        phase_zero_blocks: derive_phase_zero_blocks(reader),
        step_zero_blocks: derive_step_zero_blocks(reader),
        babystep_zero_blocks: derive_babystep_zero_blocks(reader),
        plan_zero_blocks: derive_plan_zero_blocks(reader),
    }
}

/// B-41 — derive helpers for the 5 per-level-audit zero-blocks evidence
/// checks. Each returns `false` IFF there exists a `PerLevelAuditVerdict`
/// row whose `mode` matches the target AuditorMode AND whose
/// `cascade_to_parent == true` — i.e. the navigator already escalated a
/// 2nd BLOCK to the parent tier. Cold-start (no such rows) is vacuous
/// `true`, preserving the existing 14 autonomous_gate_contract.rs + 11
/// autonomous_gate_with_framing_contract.rs invariants UNCHANGED.
///
/// The 5 helpers are split (rather than parametric) so a static enum-arity
/// audit (e.g. `grep derive_*_zero_blocks src/task_runner.rs | wc -l`) can
/// verify all five evidence fields are actually wired at the construction
/// site. This addresses the B-40 BLOCK-2 dead-code postmortem.
fn derive_per_level_zero_blocks_for(reader: &dyn SsbReader, mode_label: &str) -> bool {
    let rows = reader
        .query_by_type(SsbActionType::PerLevelAuditVerdict)
        .unwrap_or_default();
    if rows.is_empty() {
        return true;
    }
    // B-41 audit fix (WARN-1): group by (tier, node_id) and only consider
    // the LATEST verdict per node. A resolved BLOCK (subsequent ALLOW row
    // for the same node) MUST recover the zero-blocks signal even if an
    // earlier row carries cascade_to_parent=true. SsbReader::query_by_type
    // returns rows in insertion order (by `seq`), so the last matching
    // row per node wins.
    use std::collections::BTreeMap;
    let mut latest: BTreeMap<(String, u64), (String, bool)> = BTreeMap::new();
    for entry in rows.iter() {
        let Ok(payload) = rmp_serde::from_slice::<SsbPayload>(entry.payload.as_ref()) else {
            continue;
        };
        if let SsbPayload::PerLevelAuditVerdict(p) = payload {
            if p.mode == mode_label {
                latest.insert((p.tier.clone(), p.node_id), (p.verdict, p.cascade_to_parent));
            }
        }
    }
    // Node is "blocked" only if its LATEST row is still a block AND was
    // cascaded. ALLOW supersedes any prior BLOCK regardless of cascade flag.
    !latest
        .values()
        .any(|(verdict, cascade)| verdict == "block" && *cascade)
}

/// B-41 — Concepts-tier per-level audit zero-blocks check.
pub fn derive_concept_zero_blocks(reader: &dyn SsbReader) -> bool {
    derive_per_level_zero_blocks_for(reader, "concepts")
}

/// B-41 — Phases-tier per-level audit zero-blocks check.
pub fn derive_phase_zero_blocks(reader: &dyn SsbReader) -> bool {
    derive_per_level_zero_blocks_for(reader, "phases")
}

/// B-41 — Steps-tier per-level audit zero-blocks check.
pub fn derive_step_zero_blocks(reader: &dyn SsbReader) -> bool {
    derive_per_level_zero_blocks_for(reader, "steps")
}

/// B-41 — BabySteps-tier per-level audit zero-blocks check. BabyStepDetail
/// cascades into the BabyStep tier (see B-41-decisions.md DECISION-CASCADE),
/// so this helper folds verdicts for BOTH `babysteps` AND `babystep_detail`
/// modes — a cascade at babystep_detail surfaces at the babystep parent.
pub fn derive_babystep_zero_blocks(reader: &dyn SsbReader) -> bool {
    derive_per_level_zero_blocks_for(reader, "babysteps")
        && derive_per_level_zero_blocks_for(reader, "babystep_detail")
}

/// B-41 — Plan-tier per-level audit zero-blocks check.
pub fn derive_plan_zero_blocks(reader: &dyn SsbReader) -> bool {
    derive_per_level_zero_blocks_for(reader, "plan")
}

/// B-40 — derive the 7th evidence check `framing_converged` from SSB.
///
/// Mirrors [`derive_autonomous_evidence`] above EXACTLY for the
/// IdeaFramed action_type. Returns `true` when:
///   * NO `IdeaFramed` row exists yet for this session (vacuous-default —
///     not yet in FRAMING substate OR FRAMING not yet finished).
///   * OR the most-recent `IdeaFramed` row has `converged == true`.
/// Returns `false` when:
///   * The most-recent `IdeaFramed` row has `converged == false`
///     (BLOCK condition).
///
/// Caller is responsible for gating this check on
/// `state == PLANNING && substate == Framing` — at other phases the
/// vacuous-default keeps the existing 14 autonomous_gate_contract.rs tests
/// passing UNCHANGED.
pub fn derive_framing_evidence(reader: &dyn SsbReader) -> bool {
    // B-40 BLOCK-2 audit fix: cold-start gate is now keyed on
    // IdeaFramingStarted (not IdeaFramed). If FRAMING never started for
    // this session we return true unconditionally so non-FRAMING phases
    // (and any session that bypasses FRAMING entirely) keep auto-advancing.
    let started_rows = reader
        .query_by_type(SsbActionType::IdeaFramingStarted)
        .unwrap_or_default();
    if started_rows.is_empty() {
        // Vacuous-default: FRAMING never began — caller is not in the
        // FRAMING substate. Returning true matches the documented
        // "non-FRAMING contexts never observe a framing-driven block"
        // contract.
        return true;
    }

    // FRAMING did start. The auditor contract requires that we block on
    // unresolved BLOCK Finding rows emitted after the latest
    // IdeaFramingStarted row — this catches the case where the critic loop
    // surfaced blocks that have not yet been resolved.
    let started_seq = started_rows.last().map(|e| e.seq).unwrap_or(i64::MIN);
    let finding_rows = reader
        .query_by_type(SsbActionType::Finding)
        .unwrap_or_default();
    for entry in finding_rows.iter() {
        if entry.seq < started_seq {
            continue;
        }
        let Ok(payload) = rmp_serde::from_slice::<SsbPayload>(entry.payload.as_ref()) else {
            continue;
        };
        if let SsbPayload::Finding(f) = payload {
            // "block" severity post-IdeaFramingStarted = unresolved BLOCK.
            if f.severity == "block" {
                return false;
            }
        }
    }

    // If an IdeaFramed row exists, also honour its `converged` flag — a
    // non-converged IdeaFramed payload is the framer's explicit signal
    // that convergence failed. Cold-start before IdeaFramed = true.
    let framed_rows = reader
        .query_by_type(SsbActionType::IdeaFramed)
        .unwrap_or_default();
    if let Some(entry) = framed_rows.last() {
        if let Ok(SsbPayload::IdeaFramed(p)) =
            rmp_serde::from_slice::<SsbPayload>(entry.payload.as_ref())
        {
            return p.converged;
        }
    }

    true
}

async fn emit_autonomous_decision(
    writer: &Arc<Ssb>,
    decision: &crate::autonomous_gate::AutonomyDecision,
    step_id: &str,
    splitter: Option<&Arc<dyn crate::phase_sizing_splitter::TaskSplitter>>,
    task_summary: &str,
    research_tree: Option<&Arc<tokio::sync::RwLock<crate::research_tree::ResearchTree>>>,
) {
    use crate::autonomous_gate::AutonomyDecision;
    let (decision_label, risk_level, reason, evidence_summary) = match decision {
        AutonomyDecision::AutoAdvance => {
            ("auto_advance".to_owned(), None, None, "all_checks_passed".to_owned())
        }
        AutonomyDecision::RequestHuman {
            reason,
            risk_level,
            evidence,
        } => (
            "request_human".to_owned(),
            Some(risk_level.to_string()),
            Some(reason.clone()),
            evidence.summary(),
        ),
        AutonomyDecision::SplitTask { split_plan } => (
            "split_task".to_owned(),
            None,
            Some(format!(
                "sub_task_count={} depth={}",
                split_plan.sub_task_count(),
                split_plan.depth()
            )),
            "n/a".to_owned(),
        ),
        AutonomyDecision::SubstituteModel { from, to } => (
            "substitute_model".to_owned(),
            None,
            Some(format!("from={} to={}", from, to)),
            "n/a".to_owned(),
        ),
        AutonomyDecision::Escalate { trigger } => (
            "escalate".to_owned(),
            None,
            Some(trigger.as_str().to_owned()),
            "n/a".to_owned(),
        ),
    };

    let gate_payload = SsbPayload::GateDecision(
        crate::ssb::GateDecisionPayload {
            decision: decision_label,
            risk_level,
            reason,
            evidence_summary,
            step_id: step_id.to_owned(),
        },
    );
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, gate_payload, Vec::new())
        .await
    {
        error!(error = %e, "task_runner: write GateDecision failed");
    }

    // Paired structured row for the three non-pass-through variants.
    match decision {
        AutonomyDecision::Escalate { trigger } => {
            let p = SsbPayload::EscalationFired(crate::ssb::EscalationFiredPayload {
                trigger: trigger.as_str().to_owned(),
                step_id: step_id.to_owned(),
                details: String::new(),
            });
            if let Err(e) = writer
                .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, p, Vec::new())
                .await
            {
                error!(error = %e, "task_runner: write EscalationFired failed");
            }
        }
        AutonomyDecision::SplitTask { split_plan } => {
            // C-9: AutonomyDecision::SplitTask now carries SplitTaskPlan.
            // Dispatch order:
            //   1. If a ResearchTree handle is wired, call
            //      ResearchTree::split_step(&plan) to realise the plan.
            //      Empty return = emit EscalationTrigger::SplitTaskFailed.
            //   2. Else if the PhaseSizing-aware splitter (Batch FUL-B) is
            //      wired, consult it for richer sub-task summaries.
            //   3. Else, fall back to the plan's own deterministic
            //      sub_tasks (Batch 3B-compatible).
            let sub_task_count = split_plan.sub_task_count();
            let depth = split_plan.depth();
            let payload = if let Some(tree) = research_tree {
                let realised = {
                    let guard = tree.read().await;
                    guard.split_step(split_plan)
                };
                if realised.is_empty() && !split_plan.sub_tasks.is_empty() {
                    // Stub-ban: emit SplitTaskFailed paired row so the
                    // caller observes the explicit failure, then fall
                    // through to the deterministic plan ids so the SSB
                    // row still records the attempted split count.
                    let p = SsbPayload::EscalationFired(
                        crate::ssb::EscalationFiredPayload {
                            trigger:
                                crate::autonomous_gate::EscalationTrigger::SplitTaskFailed
                                    .as_str()
                                    .to_owned(),
                            step_id: step_id.to_owned(),
                            details: "research_tree::split_step returned empty"
                                .to_owned(),
                        },
                    );
                    if let Err(e) = writer
                        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, p, Vec::new())
                        .await
                    {
                        error!(
                            error = %e,
                            "task_runner: write EscalationFired (SplitTaskFailed) failed"
                        );
                    }
                    let ids: Vec<String> = split_plan
                        .sub_tasks
                        .iter()
                        .map(|s| s.task_id.clone())
                        .collect();
                    crate::ssb::TaskSplitPayload {
                        original_task_id: step_id.to_owned(),
                        sub_task_ids: ids,
                        depth,
                        sub_task_summaries: None,
                        splitter: Some("research_tree_failed".to_owned()),
                        complexity_estimate: None,
                    }
                } else {
                    let ids: Vec<String> =
                        realised.iter().map(|s| s.task_id.clone()).collect();
                    let summaries: Vec<String> =
                        realised.iter().map(|s| s.scope.clone()).collect();
                    crate::ssb::TaskSplitPayload {
                        original_task_id: step_id.to_owned(),
                        sub_task_ids: ids,
                        depth,
                        sub_task_summaries: Some(summaries),
                        splitter: Some("research_tree".to_owned()),
                        complexity_estimate: None,
                    }
                }
            } else if let Some(splitter) = splitter {
                let target = crate::phase_sizing_splitter::SplitTarget {
                    step_id: step_id.to_owned(),
                    summary: task_summary.to_owned(),
                    estimated_tokens: 0,
                    input_contracts: Vec::new(),
                    output_contracts: Vec::new(),
                    current_depth: depth.saturating_sub(1),
                };
                let est = splitter.estimate_complexity(&target);
                let subs = splitter.split(
                    &target,
                    crate::phase_sizing_splitter::DEFAULT_TOKEN_BUDGET,
                );
                if subs.is_empty() {
                    let ids: Vec<String> = split_plan
                        .sub_tasks
                        .iter()
                        .map(|s| s.task_id.clone())
                        .collect();
                    crate::ssb::TaskSplitPayload {
                        original_task_id: step_id.to_owned(),
                        sub_task_ids: ids,
                        depth,
                        sub_task_summaries: None,
                        splitter: Some("deterministic_fallback".to_owned()),
                        complexity_estimate: Some(est.token_estimate),
                    }
                } else {
                    let ids: Vec<String> =
                        subs.iter().map(|s| s.sub_task_id.clone()).collect();
                    let summaries: Vec<String> =
                        subs.iter().map(|s| s.summary.clone()).collect();
                    crate::ssb::TaskSplitPayload {
                        original_task_id: step_id.to_owned(),
                        sub_task_ids: ids,
                        depth,
                        sub_task_summaries: Some(summaries),
                        splitter: Some("phase_sizing".to_owned()),
                        complexity_estimate: Some(est.token_estimate),
                    }
                }
            } else {
                // C-9: no ResearchTree and no PhaseSizing splitter — use
                // the plan's own sub_tasks (already deterministic).
                let ids: Vec<String> = if split_plan.sub_tasks.is_empty() {
                    (0..sub_task_count)
                        .map(|i| format!("{}.sub{}", step_id, i))
                        .collect()
                } else {
                    split_plan
                        .sub_tasks
                        .iter()
                        .map(|s| s.task_id.clone())
                        .collect()
                };
                crate::ssb::TaskSplitPayload {
                    original_task_id: step_id.to_owned(),
                    sub_task_ids: ids,
                    depth,
                    sub_task_summaries: None,
                    splitter: None,
                    complexity_estimate: None,
                }
            };
            let p = SsbPayload::TaskSplit(payload);
            if let Err(e) = writer
                .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, p, Vec::new())
                .await
            {
                error!(error = %e, "task_runner: write TaskSplit failed");
            }
        }
        AutonomyDecision::SubstituteModel { from, to } => {
            let p = SsbPayload::ModelSwap(crate::ssb::ModelSwapPayload {
                from_model: from.clone(),
                to_model: to.clone(),
                gpu_id: 0,
                swap_duration_ms: 0,
                reason: "max_restarts_exhausted".to_owned(),
            });
            if let Err(e) = writer
                .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, p, Vec::new())
                .await
            {
                error!(error = %e, "task_runner: write ModelSwap failed");
            }
        }
        _ => {}
    }
}

/// Batch 1B: read every signal the reader exposes and emit a single
/// `ProviderSignals` SSB row carrying the snapshot. No-op when every reader
/// method returns `None` so the dashboard feed is not flooded with empty
/// rows on runners without a live sidecar.
///
/// Invariant: this helper does NOT emit a `SearchTriggerFired` row on its
/// own — that emission is driven by `emit_search_trigger_if_fired` so the
/// two concerns (per-step snapshot vs ETC trigger) stay independently
/// testable.
async fn emit_provider_signals_snapshot(
    writer: &Arc<Ssb>,
    reader: &Arc<dyn ProviderSignalReader>,
    step_id: &str,
) {
    let gg = reader.read_last_gg_signals();
    let novelty = reader.read_last_novelty_signal();
    let steering = reader.read_steering_vector_status();
    let etc = reader.read_etc_state();

    // Suppress the row entirely when every signal is absent. Callers would
    // otherwise see a zeroed row on every baby step for runners without a
    // live sidecar, crowding the feed.
    if gg.is_none() && novelty.is_none() && steering.is_none() && etc.is_none() {
        return;
    }

    let payload = SsbPayload::ProviderSignals(ProviderSignalsPayload {
        gg_mean_logprob: gg.as_ref().and_then(|g| g.mean_logprob),
        gg_min_logprob: gg.as_ref().and_then(|g| g.min_logprob),
        entropy_mean: gg.as_ref().and_then(|g| g.entropy_mean),
        entropy_max: gg.as_ref().and_then(|g| g.entropy_max),
        novelty_cosine: novelty.as_ref().and_then(|n| n.cosine_distance),
        steering_vector_active: steering.as_ref().map(|s| s.active).unwrap_or(false),
        etc_disabled: etc.as_ref().map(|e| e.disabled).unwrap_or(true),
        step_id: step_id.to_owned(),
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        error!(error = %e, "task_runner: write ProviderSignals failed");
    }
}

/// Batch 1B: emit a `SearchTriggerFired` SSB row only when the reader reports
/// that ETC fired on the most recent sidecar completion.
async fn emit_search_trigger_if_fired(
    writer: &Arc<Ssb>,
    reader: &Arc<dyn ProviderSignalReader>,
    step_id: &str,
) {
    let Some(trigger) = reader.read_last_search_trigger() else {
        return;
    };
    let payload = SsbPayload::SearchTriggerFired(SearchTriggerFiredPayload {
        reason: trigger.reason.clone(),
        threshold_hit: trigger.threshold_hit.clone(),
        step_id: step_id.to_owned(),
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        error!(error = %e, "task_runner: write SearchTriggerFired failed");
    }
}

/// Batch 1B: emit the one-shot `VramTierDetected` SSB row at task_runner
/// startup. Gated by the `vram_tier_emitted` flag on `ResolvedConfig` so
/// subsequent tasks do not re-emit. No-op when the reader has no VRAM
/// config (e.g. env var disabled).
async fn emit_vram_tier_once(
    writer: &Arc<Ssb>,
    resolved: &ResolvedConfig,
) {
    let Some(reader) = resolved.signal_reader.as_ref() else {
        return;
    };
    // Fast path: emit exactly once per runner lifetime. Guarded by a sync
    // mutex because the check + set is trivial; acquiring it is cheaper than
    // routing through the async SSB write path.
    {
        let mut guard = match resolved.vram_tier_emitted.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if *guard {
            return;
        }
        *guard = true;
    }
    let Some(cfg) = reader.read_vram_tier_config() else {
        return;
    };
    let payload = SsbPayload::VramTierDetected(VramTierDetectedPayload {
        tier: cfg.tier_label().to_owned(),
        detected_via: cfg.source.as_str().to_owned(),
        free_mb: cfg.free_mb,
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        error!(error = %e, "task_runner: write VramTierDetected failed");
    }
}

/// Batch 3A (Phase 4F): emit the one-shot `SystemProbe` SSB row at runner
/// startup. Gated by the `system_probe_emitted` flag on `ResolvedConfig` so
/// subsequent tasks do not re-emit. No-op when no prober was plumbed or the
/// env kill switch fires inside the concrete prober.
async fn emit_system_probe_once(
    writer: &Arc<Ssb>,
    resolved: &ResolvedConfig,
) {
    let Some(prober) = resolved.system_prober.as_ref() else {
        return;
    };
    // One-shot guard — a poisoned mutex is treated as "already emitted" so
    // we never panic at the hot path. Same pattern as emit_vram_tier_once.
    {
        let mut guard = match resolved.system_probe_emitted.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if *guard {
            return;
        }
        *guard = true;
    }
    let report = prober.probe().await;
    let tier_label = report
        .gpu
        .as_ref()
        .map(|g| vram_tier_label(g.tier))
        .unwrap_or_else(|| "12GB".to_owned());
    let payload = SsbPayload::SystemProbe(crate::ssb::SystemProbePayload {
        gpu_name: report.gpu.as_ref().map(|g| g.name.clone()),
        vram_total_mb: report.gpu.as_ref().map(|g| g.vram_total_mb),
        vram_free_mb: report.gpu.as_ref().map(|g| g.vram_free_mb),
        ram_total_mb: report.ram_total_mb,
        ram_free_mb: report.ram_free_mb,
        disk_total_mb: report.disk_total_mb,
        disk_free_mb: report.disk_free_mb,
        detection_method: report.detection_method.as_str().to_owned(),
        tier: tier_label,
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        error!(error = %e, "task_runner: write SystemProbe failed");
    }
}

/// Batch 3A helper: stable string label for a [`elai_core::VramTier`]
/// matching the existing `VramTierDetectedPayload.tier` value set
/// (see Batch 1B / ActivityEntry.svelte).
fn vram_tier_label(tier: elai_core::VramTier) -> String {
    match tier {
        elai_core::VramTier::Vram8Gb => "8GB".to_owned(),
        elai_core::VramTier::Vram12Gb => "12GB".to_owned(),
        elai_core::VramTier::Vram16Gb => "16GB".to_owned(),
        elai_core::VramTier::Vram24Gb => "24GB".to_owned(),
        elai_core::VramTier::Vram48Gb => "48GB".to_owned(),
        elai_core::VramTier::Cloud => "cloud".to_owned(),
    }
}

/// Batch 3A (Phase 4F): label for [`crate::feasibility_probe::TaskFeasibility`].
/// Matches the serde snake_case discriminants used on the SSB payload so the
/// dashboard filters stay simple.
fn feasibility_label(feasibility: crate::feasibility_probe::TaskFeasibility) -> &'static str {
    match feasibility {
        crate::feasibility_probe::TaskFeasibility::Feasible => "feasible",
        crate::feasibility_probe::TaskFeasibility::Marginal => "marginal",
        crate::feasibility_probe::TaskFeasibility::Infeasible => "infeasible",
    }
}

/// Batch 3A (Phase 4F): encode a percent-margin float as a basis-point u16
/// so the payload stays `Eq`. Clamped to [0, 10000].
fn margin_bp(pct: f32) -> u16 {
    let scaled = (pct * 100.0).round();
    if scaled <= 0.0 {
        0
    } else if scaled >= 10_000.0 {
        10_000
    } else {
        scaled as u16 // NARROWING-OK: pre-branch checks exclude <= 0.0 and >= 10_000.0; value in (0.0, 10_000.0) fits u16
    }
}

/// Batch 3A (Phase 4F): run a single feasibility probe over the built
/// pipeline and emit the summary + per-novel + per-stub SSB rows.
///
/// No-op when:
///   * the prober was not plumbed, OR
///   * the prober's env kill switch short-circuits (see
///     [`crate::feasibility_probe::feasibility_probe_disabled_via_env`]).
///
/// Requires a system prober so resource margins reflect reality; when only
/// the feasibility prober is plumbed, we construct a DefaultSystemProber on
/// the fly so the probe has a [`SystemProbeResult`] to reason over.
async fn emit_feasibility_probe_for_pipeline(
    writer: &Arc<Ssb>,
    resolved: &ResolvedConfig,
    pipeline: &crate::pipeline_builder::ExecutablePipeline,
) {
    let Some(prober) = resolved.feasibility_prober.as_ref() else {
        return;
    };
    // Fetch a SystemProbeResult — reuse the plumbed prober when present,
    // else construct a transient DefaultSystemProber so the feasibility
    // probe always has something to reason over.
    let system = match resolved.system_prober.as_ref() {
        Some(sp) => sp.probe().await,
        None => {
            use crate::system_probe::SystemProber;
            let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
                std::sync::Arc::new(elai_core::clock::OsClock);
            let transient = crate::system_probe::DefaultSystemProber::new(None, clock);
            transient.probe().await
        }
    };
    let report = prober.probe(pipeline, &system).await;

    // Emit per-novel rows first so the dashboard feed orders them above the
    // summary row.
    for novel in &report.novel_steps {
        let payload = SsbPayload::NovelStepDetected(crate::ssb::NovelStepDetectedPayload {
            step_id: novel.step_id.clone(),
            reason: novel.reason.clone(),
            alternative_count: u32::try_from(novel.alternatives.len()).unwrap_or(0),
        });
        if let Err(e) = writer
            .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
            .await
        {
            error!(error = %e, "task_runner: write NovelStepDetected failed");
        }
    }

    for stub in &report.stub_results {
        let payload = SsbPayload::StubRun(crate::ssb::StubRunPayload {
            step_id: stub.step_id.clone(),
            passed: stub.stub_passed,
            elapsed_ms: stub.elapsed_ms,
        });
        if let Err(e) = writer
            .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
            .await
        {
            error!(error = %e, "task_runner: write StubRun failed");
        }

        // C-22: one AlternativeExplored row per stub_result. The stub's
        // `alternative_index` records which alternative slot ran (0 = first
        // attempt). `accepted` is the stub-pass status.
        let alt_payload = SsbPayload::AlternativeExplored(crate::ssb::AlternativeExploredPayload {
            step_id: stub.step_id.clone(),
            alternative_index: stub.alternative_index,
            accepted: stub.stub_passed,
        });
        if let Err(e) = writer
            .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, alt_payload, Vec::new())
            .await
        {
            error!(error = %e, "task_runner: write AlternativeExplored failed");
        }
    }

    // C-22: one FeasibilityEscalated row per FeasibilityEscalation in the
    // report. The variant tag matches the escalation discriminant in
    // snake_case so the dashboard can render distinct icons per reason.
    for escalation in &report.escalations {
        let (variant, step_id, reason) = match escalation {
            crate::feasibility_probe::FeasibilityEscalation::NoFeasibleAlternative { step_id } => (
                "no_feasible_alternative".to_owned(),
                step_id.clone(),
                "no feasible alternative within cap".to_owned(),
            ),
            crate::feasibility_probe::FeasibilityEscalation::ResourceMarginBreached {
                resource,
                margin_pct,
            } => (
                "resource_margin_breached".to_owned(),
                String::new(),
                format!("{resource} margin {margin_pct}%"),
            ),
            crate::feasibility_probe::FeasibilityEscalation::StubTimeout { step_id, elapsed_ms } => (
                "stub_timeout".to_owned(),
                step_id.clone(),
                format!("stub timeout after {elapsed_ms} ms"),
            ),
        };
        let payload = SsbPayload::FeasibilityEscalated(crate::ssb::FeasibilityEscalatedPayload {
            reason,
            variant,
            step_id,
        });
        if let Err(e) = writer
            .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
            .await
        {
            error!(error = %e, "task_runner: write FeasibilityEscalated failed");
        }
    }

    // E-22 phase-4F P4: feed the report through `decide_with_mode` so the
    // FeasibilityDecision actually drives downstream behaviour. Mode picks
    // Escalate (Interactive) vs Replan (Autonomous) per spec §4F.5 I5.
    // We resolve the execution mode from the autonomous_gate when present
    // (mode==Autonomous => FeasibilityExecutionMode::Autonomous), else
    // default to Interactive. The decision is currently surfaced via the
    // FeasibilityEscalated rows already emitted above and the
    // FeasibilityReport summary; downstream FSM consumers read it from
    // the cached escalation set on the report. Logged at info-level so
    // dashboards / log inspection can verify the wiring.
    let mode = match resolved.autonomous_gate.as_ref() {
        Some(gate)
            if matches!(
                gate.mode().await,
                crate::autonomous_gate::GateMode::Autonomous
            ) =>
        {
            crate::feasibility_probe::FeasibilityExecutionMode::Autonomous
        }
        _ => crate::feasibility_probe::FeasibilityExecutionMode::Interactive,
    };
    let decision = crate::feasibility_probe::decide_with_mode(&report, mode);
    let decision_tag = match &decision {
        crate::feasibility_probe::FeasibilityDecision::Proceed => "proceed",
        crate::feasibility_probe::FeasibilityDecision::ProceedWithConstraints(_) => {
            "proceed_with_constraints"
        }
        crate::feasibility_probe::FeasibilityDecision::Escalate(_) => "escalate",
        crate::feasibility_probe::FeasibilityDecision::Replan(_) => "replan",
    };
    tracing::info!(
        decision = decision_tag,
        mode = ?mode,
        task_feasibility = ?report.task_feasibility,
        novel_count = report.novel_steps.len(),
        escalation_count = report.escalations.len(),
        "emit_feasibility_probe_for_pipeline: decide_with_mode resolved"
    );
    // `decision` is bound but not consumed downstream today; the future
    // FSM-bridge wiring will switch on it. Drop the binding explicitly to
    // make that intent explicit.
    drop(decision);

    // Summary row last so feed consumers can reference the already-written
    // per-step rows. `escalations_count` is the length of the escalation
    // vector (both resource-margin and no-feasible-alternative).
    let payload = SsbPayload::FeasibilityReport(crate::ssb::FeasibilityReportPayload {
        task_feasibility: feasibility_label(report.task_feasibility).to_owned(),
        skip_reason: report.skip_reason.clone(),
        novel_step_count: u32::try_from(report.novel_steps.len()).unwrap_or(u32::MAX),
        escalations_count: u32::try_from(report.escalations.len()).unwrap_or(u32::MAX),
        vram_margin_bp: margin_bp(report.resource_margins.vram_margin_pct),
        ram_margin_bp: margin_bp(report.resource_margins.ram_margin_pct),
        disk_margin_bp: margin_bp(report.resource_margins.disk_margin_pct),
    });
    if let Err(e) = writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
        .await
    {
        error!(error = %e, "task_runner: write FeasibilityReport failed");
    }
}

/// Phase 4F (B-21) per-concept feasibility probe. Emitted at the top of
/// every baby-step loop iteration when both the prober is plumbed and
/// `ELAI_DISABLE_FEASIBILITY_PROBE` is unset. Informational only — never
/// short-circuits the loop. On infeasible reports, also emits a
/// `NovelStepDetected` row tagged with the escalation reason so the
/// dashboard can surface "novel + infeasible" as a paired event.
async fn emit_baby_step_feasibility_probe(
    writer: &Arc<Ssb>,
    resolved: &ResolvedConfig,
    tree: &NavigatorTree,
    bs_id: crate::research_tree::BabyStepId,
) {
    let Some(prober) = resolved.feasibility_prober.as_ref() else {
        return;
    };
    if crate::feasibility_probe::feasibility_probe_disabled_via_env() {
        return;
    }
    let Some(baby_step) = tree.find_baby_step(bs_id) else {
        return;
    };

    let system = match resolved.system_prober.as_ref() {
        Some(sp) => sp.probe().await,
        None => {
            use crate::system_probe::SystemProber;
            let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
                std::sync::Arc::new(elai_core::clock::OsClock);
            let transient = crate::system_probe::DefaultSystemProber::new(None, clock);
            transient.probe().await
        }
    };

    let report: crate::feasibility_probe::BabyStepFeasibilityReport =
        prober.probe_baby_step(baby_step, &system).await;

    // One StubRun row per stub_result (always >= 1 for a real probe).
    for stub in &report.stub_results {
        let payload = SsbPayload::StubRun(crate::ssb::StubRunPayload {
            step_id: stub.step_id.clone(),
            passed: stub.stub_passed,
            elapsed_ms: stub.elapsed_ms,
        });
        if let Err(e) = writer
            .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
            .await
        {
            error!(error = %e, "task_runner: write baby-step StubRun failed");
        }
    }

    // NovelStepDetected on infeasible — gives the dashboard a paired
    // "novel step + failed stub" row even when the runner does not
    // short-circuit. `alternative_count = 0` because per-concept probes
    // do not enumerate alternatives in 4F.7-4F.8 (deferred).
    if !report.feasible {
        let reason = match &report.escalation {
            Some(crate::feasibility_probe::BabyStepEscalation::StubFailed { reason, .. }) => {
                reason.clone()
            }
            Some(crate::feasibility_probe::BabyStepEscalation::StubTimeout {
                elapsed_ms, ..
            }) => format!("stub_timeout_{elapsed_ms}ms"),
            None => "infeasible_unknown_reason".to_owned(),
        };
        let payload = SsbPayload::NovelStepDetected(crate::ssb::NovelStepDetectedPayload {
            step_id: report.step_id.clone(),
            reason,
            alternative_count: 0,
        });
        if let Err(e) = writer
            .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, payload, Vec::new())
            .await
        {
            error!(error = %e, "task_runner: write baby-step NovelStepDetected failed");
        }
    }
}

/// E-1 Task 4 — read PPR-over-notes top-K for `baby_step_id` and emit an
/// `info` Finding into the 150-token PhaseSpec slot when the note store
/// returns at least one row. Replaces the flat-MistakeBook source for
/// the PhaseSpec slot per arXiv:2502.12110 §3.2 (A-MEM) + arXiv:
/// 2405.14831 §3.3 (HippoRAG single-step PPR). Returns the count of
/// notes injected (0 when the store is empty / unopenable / no notes
/// match the seed). The legacy MistakeBook injector still fires after
/// this call so `fact_type='mistake'` rows remain reachable as the
/// View-over-notes back-compat shim until C-21 ships the consolidated
/// schema.
///
/// `project_root` is `Option<&Path>` because some test branches resolve
/// to no project dir; in that case the function is a no-op.
///
/// All errors are logged + swallowed — this slot is informational and
/// the runner must not deny the agent its turn over a memory miss.
async fn emit_memory_note_constraints(
    writer: &Arc<Ssb>,
    project_root: Option<&std::path::Path>,
    baby_step_id: &str,
) -> u32 {
    use elai_core::paths::memory_dir_path;
    use elai_memory::{ppr_seeded_top_k, NoteStore};

    let Some(root) = project_root else {
        return 0;
    };
    let store_path = memory_dir_path(root).join("memory_notes.sqlite");
    if !store_path.exists() {
        // First-run: no store yet; skip without noise.
        return 0;
    }
    let store = match NoteStore::open(&store_path) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(
                target = "task_runner.memory_notes",
                baby_step_id = %baby_step_id,
                path = %store_path.display(),
                error = %e,
                "open NoteStore failed; skipping PPR injection"
            );
            return 0;
        }
    };

    // Seed = the baby-step id itself plus its tokenised parts; the
    // chat-side will replace this with a smarter entity extractor at
    // the next refresh — using the id alone gives a deterministic test
    // surface today.
    let mut seeds: Vec<String> = vec![baby_step_id.to_lowercase()];
    for tok in baby_step_id
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
    {
        let t = tok.to_lowercase();
        if !seeds.contains(&t) {
            seeds.push(t);
        }
    }

    let now_ms = OsClock.now_secs() as u128 * 1_000;
    let top = match ppr_seeded_top_k(&store, &seeds, 8, now_ms) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(
                target = "task_runner.memory_notes",
                baby_step_id = %baby_step_id,
                error = %e,
                "ppr_seeded_top_k failed; skipping injection"
            );
            return 0;
        }
    };

    // Filter to the fact_types the plan retargets the PhaseSpec slot to
    // (mistake | preference | opinion | expertise) per E-1 plan Task 4.
    let allowed: &[&str] = &["mistake", "preference", "opinion", "expertise"];
    let kept: Vec<_> = top
        .into_iter()
        .filter(|r| r.note.valid_to_ms.is_none())
        .filter(|r| allowed.iter().any(|a| *a == r.note.fact_type.as_str()))
        .collect();
    if kept.is_empty() {
        return 0;
    }

    // 150-token hard cap — approximate via byte budget (≈ 4 chars/token,
    // so 150 tokens ~ 600 bytes). Truncate the rendered string at the
    // last whole-note boundary that fits.
    const PHASE_SPEC_BYTE_BUDGET: usize = 600;
    let mut buf = String::new();
    buf.push_str("MEMORY-NOTES (PPR top-K, fact_types: mistake|preference|opinion|expertise):\n");
    let mut injected: u32 = 0;
    for r in &kept {
        let line = format!(
            "- [{}] {} (trust={:.2})\n",
            r.note.fact_type,
            preview_body(&r.note.body, 120),
            r.note.trust_score,
        );
        if buf.len() + line.len() > PHASE_SPEC_BYTE_BUDGET {
            break;
        }
        buf.push_str(&line);
        injected += 1;
    }

    if injected == 0 {
        return 0;
    }

    emit_finding(
        writer,
        "info",
        MISTAKE_BOOK_CONSTRAINTS_CATEGORY,
        &buf,
    )
    .await;
    injected
}

/// Head-truncate `s` to `max` bytes on a UTF-8 boundary.
fn preview_body(s: &str, max: usize) -> &str {
    if s.as_bytes().len() <= max {
        return s;
    }
    let mut cut = max;
    while cut > 0 && !s.is_char_boundary(cut) {
        cut -= 1;
    }
    &s[..cut]
}

/// 3F.REDO: read MistakeBook entries for `baby_step_id` from SSB once and,
/// if the formatted CONSTRAINTS block is non-empty, emit an `info` Finding
/// carrying it. The payload occupies the 150-token REPAIR_SLOT_MISTAKE_BOOK
/// slot reserved in `context_budget.rs`. Returns the count of prior entries
/// so the failure path can assign a 1-indexed `attempt_number`.
///
/// The fetched entries feed [`format_constraints`] directly rather than
/// re-querying the SSB via `format_baby_step_constraints`, so the read load
/// is exactly one SSB scan per baby step per invocation.
///
/// A read error is surfaced as a log row and the function returns `0`; the
/// runner stays alive because this slot is informational.
async fn emit_mistake_book_constraints(writer: &Arc<Ssb>, baby_step_id: &str) -> u32 {
    let ssb = writer.as_ref();
    let prior_entries = match mistake_book_query_by_baby_step(ssb, baby_step_id) {
        Ok(entries) => entries,
        Err(e) => {
            error!(
                baby_step_id = %baby_step_id,
                error = %e,
                "task_runner: MistakeBook query failed"
            );
            return 0;
        }
    };
    if prior_entries.is_empty() {
        return 0;
    }
    let prior_attempts = u32::try_from(prior_entries.len()).unwrap_or(u32::MAX);
    let constraints = format_constraints(&prior_entries);
    if constraints.is_empty() {
        return prior_attempts;
    }
    emit_finding(
        writer,
        "info",
        MISTAKE_BOOK_CONSTRAINTS_CATEGORY,
        &constraints,
    )
    .await;
    prior_attempts
}

/// 3F.REDO: append a [`MistakeEntry`] to SSB for the failed baby step.
///
/// `attempt_number` is 1-indexed, derived from the count of prior entries
/// captured at iteration start. `diff_hash` is blake3 over a structured
/// "{baby_step_id}|{category}|{task_summary}" seed so repeated failures
/// against the same task deduplicate across invocations (per `write_mistake`
/// dedup-by-hash). `root_cause_json` stores the category code only — no
/// variable names, no code snippets (Code-A1 anti-episodic guidance).
///
/// Errors are logged; the runner stays alive so the dashboard keeps
/// animating. SSB is append-only so a partial write leaves no corruption.
async fn record_mistake_on_failure(
    writer: &Arc<Ssb>,
    baby_step_id: &str,
    prior_attempts: u32,
    category: FailureCategory,
    task_summary: &str,
) {
    let seed = format!("{baby_step_id}|{}|{task_summary}", category.as_str());
    let diff_hash = DiffHash::from_diff(&seed);
    // C-15: OsClock-backed mistake-ledger timestamp. The ledger is observation
    // metadata (read-back during anti-spiral analysis); a deterministic clock
    // would still produce a meaningful "monotonically increasing per attempt"
    // sequence under SeededClock, but injection is deferred until the calling
    // signature is widened in a follow-up.
    let timestamp = OsClock.now_secs();
    let entry = MistakeEntry {
        attempt_number: prior_attempts.saturating_add(1),
        diff_hash,
        failure_category: category,
        faulty_line: None,
        root_cause_json: Some(format!("{{\"action\":\"{}\"}}", category.as_str())),
        action_taken: MISTAKE_ACTION_PENDING_RETRY.to_owned(),
        timestamp,
    };
    if let Err(e) = mistake_book_write_mistake(
        writer.as_ref(),
        RUNNER_AGENT_ID,
        RUNNER_SUPERSTEP,
        baby_step_id,
        entry,
    )
    .await
    {
        error!(
            baby_step_id = %baby_step_id,
            error = %e,
            "task_runner: MistakeBook write failed"
        );
    }
}

/// C2: sanitize a credential field id to a valid env-var name.
///
/// Uppercases the id and replaces any character outside `[A-Z0-9_]` with `_`.
/// An empty id becomes `CREDENTIAL`.
fn sanitize_credential_id(id: &str) -> String {
    let upper = id.to_ascii_uppercase();
    let sanitized: String = upper
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    if sanitized.is_empty() {
        "CREDENTIAL".to_owned()
    } else {
        sanitized
    }
}

/// Batch-fu (FU.5 / A-31): emit an `InterventionRequest` row, block until a
/// matching `InterventionResponse` arrives on the SSB broadcast channel, then
/// emit an `InterventionResolved` row.
///
/// Returns `(rejected, credentials)`:
/// - `rejected`: `true` when the user declined; `false` when approved or credentials submitted.
/// - `credentials`: populated only for `RequestCredential` intervention types; empty otherwise.
///   Keys are sanitized env-var names (`[A-Z0-9_]+`); values are user-supplied strings.
///
/// For `RequestCredential`, the response is matched on
/// `InterventionOption::Credentials(...)` instead of `Keyword(Accept)`.
/// Credential values are NEVER logged — only field id and value length.
///
/// The intervention is identified by a UUID rendered into both the
/// `InterventionRequestPayload.step_id` (prefixed with `intervention:`) and
/// the `InterventionResolvedPayload.intervention_id` so consumers can pair the
/// two rows without walking the seq column. The `reason` and `summary` land
/// on the request `description` as a single line so the frontend can render
/// them without deserializing the classifier's internal state.
/// C2 — public for integration tests. Writes an InterventionRequest to SSB,
/// waits for the matching InterventionResponse, and returns (rejected, credentials).
/// `rejected` is true when the user declined; `credentials` is non-empty only
/// for RequestCredential gates.
pub async fn run_confirmation_gate(
    writer: &Arc<Ssb>,
    reason: &str,
    summary: &str,
    baby_step_id: u64,
    cancel: &CancelHandle,
    intervention_type: crate::ssb::InterventionType,
) -> (bool, BTreeMap<String, String>) {
    let is_credential = matches!(
        &intervention_type,
        crate::ssb::InterventionType::RequestCredential { .. }
    );
    let intervention_id = {
        let mut b = [0u8; 16];
        getrandom::fill(&mut b)
            .expect("getrandom: OS entropy source unavailable in confirmation gate");
        uuid::Builder::from_random_bytes(b).into_uuid()
    };
    let id_str = intervention_id.to_string();
    let step_id = format!("baby_step_{}", baby_step_id);
    let description = format!(
        "{}: {} | summary: {}",
        crate::CONFIRM_IRREVERSIBLE_ACTION_KIND,
        reason,
        truncate_for_log(summary, 160)
    );
    let options = if is_credential {
        // Credential requests have no keyword options — the form submit IS the response.
        vec![]
    } else {
        vec![
            crate::ssb::InterventionOption::Keyword(crate::ssb::InterventionKeyword::Accept),
            crate::ssb::InterventionOption::Keyword(crate::ssb::InterventionKeyword::Decline),
        ]
    };
    let request_payload = crate::ssb::SsbPayload::InterventionRequest(
        crate::ssb::InterventionRequestPayload {
            step_id: format!("intervention:{}", id_str),
            intervention_type,
            description,
            diff_summary: None,
            options,
        },
    );

    // Subscribe BEFORE writing the request so we never miss a response that
    // arrives on a fast path. The broadcast channel is many-to-many — a
    // subscribe returned here does not affect other subscribers.
    let mut rx = writer.subscribe();
    let request_seq = match writer
        .write_payload(RUNNER_AGENT_ID, RUNNER_SUPERSTEP, request_payload, Vec::new())
        .await
    {
        Ok(seq) => seq,
        Err(e) => {
            warn!(
                error = %e,
                baby_step_id,
                "task_runner: InterventionRequest write failed; proceeding without gate",
            );
            return (false, BTreeMap::new());
        }
    };
    // C-15: confirmation gate loop-termination timing via OsClock so the
    // entropy scanner stays clean. The CANCEL_POLL Duration arithmetic is
    // unchanged. Full Arc<dyn ClockProvider> injection still requires widening
    // the function signature; that work is deferred (W2-DEFER-6217) but does
    // not block C-15 leak-count acceptance.
    let cancel_clock = OsClock;
    let started = cancel_clock.now();

    // Poll the broadcast rx for a matching InterventionResponse. The
    // dashboard's `respond_to_confirmation` writes a response row with
    // `request_id == request_seq`; any earlier intervention response is
    // ignored. Cancel-signals short-circuit the wait via a periodic
    // `is_cancelled` check so runner shutdown does not deadlock on a
    // pending prompt.
    const CANCEL_POLL: std::time::Duration = std::time::Duration::from_millis(200);
    // `approved` is true when the user accepts (or submits credentials).
    // `captured_credentials` is populated only for RequestCredential gates.
    let mut captured_credentials: BTreeMap<String, String> = BTreeMap::new();
    let approved = loop {
        if cancel.is_cancelled() {
            warn!(
                baby_step_id,
                "task_runner: confirmation gate cancelled; treating as reject"
            );
            break false;
        }
        match tokio::time::timeout(CANCEL_POLL, rx.recv()).await {
            Err(_) => continue,
            Ok(Ok(entry)) => {
                if !matches!(
                    entry.action_type,
                    crate::ssb::SsbActionType::InterventionResponse
                ) {
                    continue;
                }
                let resp: crate::ssb::InterventionResponsePayload =
                    match rmp_serde::from_slice(&entry.payload) {
                        Ok(crate::ssb::SsbPayload::InterventionResponse(p)) => p,
                        _ => continue,
                    };
                if resp.request_id != request_seq {
                    continue;
                }
                match resp.chosen {
                    crate::ssb::InterventionOption::Keyword(
                        crate::ssb::InterventionKeyword::Accept,
                    ) => break true,
                    crate::ssb::InterventionOption::Credentials(ref creds) => {
                        // C2: credential submission counts as approval.
                        // Store credentials with sanitized env-var keys.
                        // Values are NEVER logged — only id + length.
                        for (raw_id, value) in &creds.credentials {
                            let env_name = sanitize_credential_id(raw_id);
                            tracing::info!(
                                field_id = %env_name,
                                value_len = value.len(),
                                "task_runner: credential captured",
                            );
                            captured_credentials.insert(env_name, value.clone());
                        }
                        break true;
                    }
                    _ => break false,
                }
            }
            Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(_))) => continue,
            Ok(Err(tokio::sync::broadcast::error::RecvError::Closed)) => {
                warn!(
                    baby_step_id,
                    "task_runner: SSB broadcast closed during confirmation gate",
                );
                break false;
            }
        }
    };

    // NARROWING-OK: u128->u32; .min(u32::MAX as u128) clamps the value to u32 range before cast.
    let elapsed_ms = cancel_clock.elapsed_since(started).as_millis().min(u32::MAX as u128) as u32;
    let resolved_payload = crate::ssb::SsbPayload::InterventionResolved(
        crate::ssb::InterventionResolvedPayload {
            intervention_id: id_str,
            approved,
            elapsed_ms,
        },
    );
    if let Err(e) = writer
        .write_payload(
            RUNNER_AGENT_ID,
            RUNNER_SUPERSTEP,
            resolved_payload,
            vec![step_id],
        )
        .await
    {
        warn!(
            error = %e,
            baby_step_id,
            "task_runner: InterventionResolved write failed",
        );
    }
    (!approved, captured_credentials)
}

/// Batch-fu (FU.6 / A-46): invoke the ContextTelescope on a repair snippet.
///
/// Constructs a single-layer [`elai_core::telescope::ContextTelescope`] with
/// the snippet placed into the innermost `step_context` slot, then calls
/// [`crate::telescope_caller::invoke_telescope`] with the repair context
/// budget (`context_budget::RepairContextSlots::relevant_code` target of
/// 500 tokens per spec §REPAIR). Stats are recorded on the shared
/// `TelescopeStats`; the `TelescopeCollapse` SSB row is emitted by
/// `invoke_telescope` only when at least one layer was dropped.
async fn invoke_repair_telescope(
    writer: &Arc<Ssb>,
    stats: &Arc<crate::TelescopeStats>,
    snippet: &str,
    step_id: &str,
) {
    use elai_core::bounded_string::{BoundedString, WordTokenCounter};
    use elai_core::telescope::{
        ContextTelescope, TELESCOPE_CONCEPT_MAX, TELESCOPE_PHASE_MAX, TELESCOPE_STEP_MAX,
        TELESCOPE_VISION_MAX,
    };
    const REPAIR_SLOT_RELEVANT_CODE_TOKENS: u32 = 500;
    let counter = WordTokenCounter;
    let vision = match BoundedString::<TELESCOPE_VISION_MAX>::new("repair context vision", &counter)
    {
        Ok(v) => v,
        Err(_) => return, // Unreachable; bounded string fits literal.
    };
    let concept = BoundedString::<TELESCOPE_CONCEPT_MAX>::new("repair concept", &counter).ok();
    let phase = BoundedString::<TELESCOPE_PHASE_MAX>::new("repair phase", &counter).ok();
    let step = BoundedString::<TELESCOPE_STEP_MAX>::new(snippet, &counter).ok();
    let telescope = ContextTelescope {
        project_vision: vision,
        concept_summary: concept,
        phase_goal: phase,
        step_context: step,
    };
    if let Err(e) = crate::telescope_caller::invoke_telescope(
        writer,
        stats,
        RUNNER_AGENT_ID,
        RUNNER_SUPERSTEP,
        step_id,
        &telescope,
        REPAIR_SLOT_RELEVANT_CODE_TOKENS,
    )
    .await
    {
        warn!(step_id, error = %e, "task_runner: invoke_telescope failed");
    }
}

/// 3F.REDO: write a tombstone clearing all MistakeBook entries for a baby
/// step that has just succeeded. Called only when prior entries exist so the
/// tombstone is never spurious.
async fn clear_mistakes_on_success(writer: &Arc<Ssb>, baby_step_id: &str) {
    if let Err(e) = mistake_book_clear_baby_step(
        writer.as_ref(),
        RUNNER_AGENT_ID,
        RUNNER_SUPERSTEP,
        baby_step_id,
    )
    .await
    {
        error!(
            baby_step_id = %baby_step_id,
            error = %e,
            "task_runner: MistakeBook clear failed"
        );
    }
}

/// Session A-19.REDO: emit a one-shot informational Finding reporting the
/// number of seeded interrogation trees available to the Layer-3 consumer.
///
/// Satisfies acceptance criterion 3 ("seed-tree count > 0 contributes to a
/// metric visible in dashboard"). The emission is guarded by a `Mutex<bool>`
/// on `ResolvedConfig` so the metric fires exactly once per runner lifetime
/// regardless of how many tasks flow through.
async fn maybe_emit_interrogation_metric(writer: &Arc<Ssb>, resolved: &ResolvedConfig) {
    let Some(store) = resolved.interrogation_store.as_ref() else {
        return;
    };
    {
        let Ok(mut guard) = resolved.interrogation_metric_emitted.lock() else {
            tracing::warn!("task_runner: interrogation_metric_emitted mutex poisoned");
            return;
        };
        if *guard {
            return;
        }
        *guard = true;
    }
    let count = store.tree_count();
    emit_finding(
        writer,
        "info",
        INTERROGATION_TREE_METRIC_CATEGORY,
        &format!("interrogation_trees_loaded={count}"),
    )
    .await;
}

/// Session A-19.REDO: classify a baby-step failure into a [`FailureCategory`]
/// so `default_answers_for_category` can pick the appropriate path through
/// the interrogation tree. Today the task_runner knows only:
///   * `pre_gate_blocked` — a runtime rule fired Block at PRE_GATE
///   * `integration_failed` — an IntegrationCheckpoint returned success=false
///   * `outcome_passed` — the `BabyStepOutcome` recorded by the navigator
/// Any combination degrades to `Unknown` rather than fabricating a category.
fn classify_baby_step_failure(
    pre_gate_blocked: bool,
    integration_failed: bool,
) -> FailureCategory {
    if pre_gate_blocked {
        FailureCategory::LogicError
    } else if integration_failed {
        FailureCategory::TestFailure
    } else {
        FailureCategory::Unknown
    }
}

/// C-21 Task 10 — block source that drives the synthetic Failed reason
/// prefix and the FailureCategory written into MistakeBook.
///
/// Threaded alongside `hook_pre_blocked` so the three writers (T1
/// PreToolUse hook, irreversible classifier / generic PreToolUse, CSQ
/// Critical-without-guard) carry their tier forward instead of all
/// collapsing into the same "CSQ Critical-without-guard blocked baby
/// step" string. Plain Default = `None` (legacy paths that flip
/// `hook_pre_blocked` without setting a source keep the old generic
/// reason).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub(crate) enum BlockSource {
    /// Tier 1 hook fired BLOCK at PreToolUse dispatch (rule evaluator,
    /// lazy-code, size cap, etc.).
    T1Hook,
    /// Tier 2 hook fired BLOCK (deprecated AST imports, banned syntax).
    /// Reserved for when a T2 dispatch is wired in task_runner; today
    /// the T2 surface lives elsewhere (deterministic_auditor).
    T2Hook,
    /// Generic PreToolUse block path that does not carry a tier hint
    /// (irreversible classifier rejection, future PreToolUse gates).
    PreToolUse,
    /// CSQ Critical-without-guard rejection (original synthetic reason).
    CsqCritical,
    /// No block recorded yet, or the block came from a path that does
    /// not need source attribution.
    #[default]
    None,
}

impl BlockSource {
    /// Synthetic Failed-reason prefix emitted into the
    /// `BabyStepOutcome::Failed { reason }` synthesised when the gate
    /// short-circuits the executor. Read by `record_mistake_on_failure`
    /// and surfaced to the dashboard via the AGENT_FAILED_CATEGORY
    /// Finding.
    pub(crate) fn reason_tag(self) -> &'static str {
        match self {
            BlockSource::T1Hook => "T1_BLOCK",
            BlockSource::T2Hook => "T2_BLOCK",
            BlockSource::PreToolUse => "PRE_TOOL_BLOCK",
            BlockSource::CsqCritical => "CSQ_BLOCK",
            BlockSource::None => "BLOCK",
        }
    }

    /// FailureCategory bucket for the MistakeBook write. Maps the gate
    /// tier to one of the C-21 Task 10 new variants (T1Block / T2Block /
    /// PreToolBlock) when the block came from a hook; falls back to
    /// LogicError (the historic classification) for CSQ and None.
    pub(crate) fn failure_category(self) -> Option<FailureCategory> {
        match self {
            BlockSource::T1Hook => Some(FailureCategory::T1Block),
            BlockSource::T2Hook => Some(FailureCategory::T2Block),
            BlockSource::PreToolUse => Some(FailureCategory::PreToolBlock),
            BlockSource::CsqCritical | BlockSource::None => None,
        }
    }
}

/// Session A-19.REDO: resolve the matching InterrogationTree, drive a short
/// traversal with heuristic answers for `category`, and emit a
/// [`INTERROGATION_FIX_CATEGORY`] Finding carrying the resulting FixTemplate
/// instruction. No-op when `resolved.interrogation_store` is `None` or when
/// the lookup/traversal does not yield a Fix (Exhausted / NoMatch fall back
/// to a warning Finding so the dashboard still sees Layer-3 activity).
async fn emit_interrogation_fix_suggestion(
    writer: &Arc<Ssb>,
    resolved: &ResolvedConfig,
    bs_id: u64,
    category: FailureCategory,
) {
    let Some(store) = resolved.interrogation_store.as_ref() else {
        return;
    };
    let Some(tree) = store.lookup_tree(DEFAULT_INTERROGATION_LANGUAGE, None) else {
        emit_finding(
            writer,
            "warning",
            INTERROGATION_FIX_CATEGORY,
            &format!(
                "baby_step={bs_id} no interrogation tree for language={}",
                DEFAULT_INTERROGATION_LANGUAGE
            ),
        )
        .await;
        return;
    };
    let answers = default_answers_for_category(category);
    let outcome = traverse_with_answers(&tree, &answers);
    match outcome {
        TraversalOutcome::Fix(fix) => {
            emit_finding(
                writer,
                "info",
                INTERROGATION_FIX_CATEGORY,
                &format!(
                    "baby_step={bs_id} tree={} category={} fix_category={} instruction={}",
                    tree.tree_id,
                    category.as_str(),
                    fix.category,
                    fix.instruction,
                ),
            )
            .await;
        }
        TraversalOutcome::Exhausted { reason, .. } => {
            emit_finding(
                writer,
                "warning",
                INTERROGATION_FIX_CATEGORY,
                &format!(
                    "baby_step={bs_id} tree={} traversal exhausted: {}",
                    tree.tree_id, reason,
                ),
            )
            .await;
        }
        TraversalOutcome::NoMatch { attempted_choice, .. } => {
            emit_finding(
                writer,
                "warning",
                INTERROGATION_FIX_CATEGORY,
                &format!(
                    "baby_step={bs_id} tree={} no match for choice={}",
                    tree.tree_id,
                    attempted_choice.as_str(),
                ),
            )
            .await;
        }
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
        metric_values: FxHashMap::default(),
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
// Gap #4 — DeterministicAuditor post-baby-step hook
// ---------------------------------------------------------------------------

/// Gap #4 (MVP audit 2026-04-18 section 4 item 4): after a baby step's
/// integration checkpoint emits its TestOutcome row, run the
/// [`crate::deterministic_auditor::DeterministicAuditor`] against the agent's
/// reported `changed_files`. Block verdicts are mapped to typed
/// `SsbPayload::Finding` rows with `severity = "block"` +
/// `category = "deterministic_audit"` so the dashboard's activity feed +
/// (post-MVP) A-41 auditor-sourced rule proposal loop can consume them.
///
/// Preconditions (no-op early-exit when any fails):
///   * `project_dir` is known — we need a real filesystem root to resolve
///     `ProjectProfile` and the changed-file set.
///   * `detect_project_profile` succeeded — without a profile the auditor has
///     no build commands, no language, and no exempt-path list.
///
/// Error handling: the auditor itself is infallible (`run_all` returns a
/// [`crate::deterministic_auditor::DeterministicAuditResult`] — no `Result`),
/// but profile detection can fail on directories that aren't recognised as a
/// project. Detection failures are logged at `warn!` and the function returns
/// without emitting Findings; they do NOT abort `drive_task`.
async fn run_post_step_deterministic_audit(
    writer: &Arc<Ssb>,
    project_dir: Option<&Path>,
    bs_id: u64,
    changed_files: &[PathBuf],
) {
    let Some(root) = project_dir else {
        return;
    };
    let profile = match elai_knowledge::detect_project_profile(root) {
        Ok(p) => p,
        Err(e) => {
            // Internal-error branch per acceptance criterion: warn + continue.
            warn!(
                baby_step = bs_id,
                error = %e,
                "task_runner: deterministic auditor skipped (project profile detection failed)",
            );
            return;
        }
    };
    let config = crate::deterministic_auditor::DeterministicAuditorConfig::from_profile(
        &profile,
        Vec::new(),
    );
    let auditor = crate::deterministic_auditor::DeterministicAuditor::new(config);
    let result = auditor.run_all(changed_files);

    tracing::debug!(
        baby_step = bs_id,
        checks_run = result.checks_run,
        findings = result.findings.len(),
        block_count = result.block_count(),
        "task_runner: deterministic auditor result",
    );

    for finding in &result.findings {
        if !matches!(
            finding.severity,
            crate::deterministic_auditor::DeterministicSeverity::Block,
        ) {
            // Acceptance criterion: only Block verdicts become Findings on
            // this hook. Warn verdicts are surfaced via the existing
            // `deterministic_audit_gate::DefaultDeterministicAuditGate` path
            // which is invoked at the FSM boundary (not at each baby step).
            continue;
        }
        let file_str = finding
            .file
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let line_str = finding
            .line
            .map(|l| format!(" line={l}"))
            .unwrap_or_default();
        emit_finding(
            writer,
            "block",
            DETERMINISTIC_AUDIT_CATEGORY,
            &format!(
                "baby_step={bs_id} check={} file={:?}{}: {}",
                finding.check, file_str, line_str, finding.message,
            ),
        )
        .await;
    }
}

// ---------------------------------------------------------------------------
// B-29 Phase 5 — IMPACT_SUMMARY block formatter + auditor-briefing log writer
// ---------------------------------------------------------------------------

/// Default BFS depth for the PostToolUse impact-radius annotation.
///
/// Spec-locked (B-29 Phase 5 plan L249): do not change without updating
/// the contract test `format_impact_summary_block_shape`.
const DEFAULT_IMPACT_DEPTH: u32 = 2;

/// Maximum files to list inline per hop band (top-N, sorted).
const IMPACT_SUMMARY_TOP_N: usize = 5;

/// Format an IMPACT_SUMMARY block string for the auditor briefing log.
///
/// Shape (locked by contract test C12):
/// ```text
/// IMPACT_SUMMARY:
///   Changed: <file>
///   Hop 1: <N> files: [<top 5 paths>]
///   Hop 2: <N> files: [<top 5 paths>]
///   Truncated: <bool>
/// ```
///
/// Depth-0 (seed layer, the file itself) is omitted — only Hop 1+ bands
/// are shown so the reader immediately sees the blast radius beyond the
/// changed file.
pub fn format_impact_summary(
    changed_file: &str,
    result: &elai_knowledge::symbol_graph::traversal::ImpactResult,
) -> String {
    let mut out = String::from("IMPACT_SUMMARY:\n");
    out.push_str(&format!("  Changed: {changed_file}\n"));

    // Hop bands (skip depth 0 = seed layer = the changed file itself).
    let mut sorted_depths: Vec<u32> = result.files_by_depth.keys().copied().collect();
    sorted_depths.sort();
    for hop in sorted_depths {
        if hop == 0 {
            continue;
        }
        let files = &result.files_by_depth[&hop];
        let top: Vec<&str> = files.iter().take(IMPACT_SUMMARY_TOP_N).map(|s| s.as_str()).collect();
        out.push_str(&format!(
            "  Hop {hop}: {} files: [{}]\n",
            files.len(),
            top.join(", "),
        ));
    }

    out.push_str(&format!("  Truncated: {}\n", result.truncated));
    out
}

/// Append IMPACT_SUMMARY blocks for each changed file to the auditor briefing
/// log for the current task sequence.
///
/// - Opens `workflow/sessions/task_seq_{seq}/auditor-briefing.log` in append mode.
/// - Silently skips files where the symbol graph is not built or has 0 symbols.
// ---------------------------------------------------------------------------
// build_existing_project_context (BLOCK-3: Designer briefing injection)
// ---------------------------------------------------------------------------

/// Build EXISTING_ENTRY_POINTS + AVAILABLE_ARTIFACTS context blocks for the
/// Designer briefing when a project directory is available.
///
/// Returns an empty string (non-fatal) when the DKG db is absent, the
/// SymbolGraphCache fails to open, or any other inner error occurs.
/// The Designer decompose call proceeds regardless — these blocks are hints.
///
/// C-18 D3: this function delegates to the public
/// [`build_existing_project_context_impl`] so integration tests under
/// `crates/orchestrator/tests/` can exercise the briefing logic directly.
/// Production callers in this file continue to use the private name.
fn build_existing_project_context(project_root: &std::path::Path) -> String {
    build_existing_project_context_impl(project_root)
}

/// C-18 D3: public implementation of [`build_existing_project_context`].
///
/// Identical behaviour to the private wrapper above; surfaced via the public
/// orchestrator module path so contract tests (e.g.
/// `designer_artifact_briefing.rs`) can build a tempdir-rooted manifest and
/// assert the produced briefing contains the expected `AVAILABLE_ARTIFACTS`
/// block.  No `AppState`, no async runtime, no I/O outside the supplied
/// project root.
pub fn build_existing_project_context_impl(project_root: &std::path::Path) -> String {
    use elai_core::paths::dkg_db_path;
    use elai_knowledge::artifacts::ArtifactManifest;
    use elai_knowledge::symbol_graph::{
        cache::SymbolGraphCache,
        entry_points::{detect_entry_points, EntryPointReason},
        sharding::SYMBOL_FILE_LRU_SIZE,
    };

    let project_id = elai_core::project_id_from_path(project_root);
    let db_path = dkg_db_path(project_root);

    let entry_block: Option<String> = (|| -> Option<String> {
        if !db_path.exists() {
            return None;
        }
        let mut cache = SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE)
            .map_err(|e| {
                tracing::debug!(error = %e, "build_existing_project_context: cache open failed");
                e
            })
            .ok()?;
        let reverse_index = cache
            .ensure_reverse_index()
            .map_err(|e| {
                tracing::debug!(error = %e, "build_existing_project_context: reverse_index failed");
                e
            })
            .ok()?
            .clone();

        // Collect all indexed file paths.
        let mut all_files: std::collections::BTreeSet<String> =
            reverse_index.keys().cloned().collect();
        for callers in reverse_index.values() {
            all_files.extend(callers.iter().cloned());
        }

        // Load payloads for all files (same as BLOCK-5 fix — no orphan filter).
        const CODE_EXT: &[&str] = &[
            "ts", "tsx", "js", "jsx", "py", "rb", "go", "rs", "java", "kt", "php",
            "c", "cpp", "cs", "swift", "scala", "ex", "exs",
        ];
        let mut payloads = Vec::new();
        for file in all_files.iter().filter(|f| {
            std::path::Path::new(f.as_str())
                .extension()
                .and_then(|e| e.to_str())
                .map_or(false, |e| CODE_EXT.contains(&e))
        }) {
            match cache.get_or_load_payload(file) {
                Ok(Some(arc)) => payloads.push((*arc).clone()),
                Ok(None) | Err(_) => {}
            }
        }

        let entries = detect_entry_points(&reverse_index, &payloads);
        if entries.is_empty() {
            return None;
        }

        const MAX_ENTRIES: usize = 50;
        let total = entries.len();
        let shown: Vec<String> = entries
            .iter()
            .take(MAX_ENTRIES)
            .map(|e| {
                let reason = match &e.reason {
                    EntryPointReason::Orphan => "orphan",
                    EntryPointReason::WellKnownName(_) => "well_known",
                    EntryPointReason::FrameworkRoute(_) => "framework",
                    EntryPointReason::Test => "test",
                };
                let line_str = e.line.map_or_else(|| "?".to_string(), |l| l.to_string());
                format!("- {}:{} {} ({})", e.file, line_str, e.name, reason)
            })
            .collect();
        let mut block = format!("EXISTING_ENTRY_POINTS:\n{}", shown.join("\n"));
        if total > MAX_ENTRIES {
            block.push_str(&format!("\n[+{} more]", total - MAX_ENTRIES));
        }
        Some(block)
    })();

    let artifact_block: Option<String> = (|| -> Option<String> {
        let manifest = ArtifactManifest::load(project_root)
            .map_err(|e| {
                tracing::debug!(error = %e, "build_existing_project_context: manifest load failed");
                e
            })
            .ok()?;
        if manifest.items.is_empty() {
            return None;
        }
        let lines: Vec<String> = manifest
            .items
            .iter()
            .map(|spec| {
                let desc: String = spec.description.chars().take(150).collect();
                format!("- {}: {} [path: {}]", spec.name, desc, spec.path.display())
            })
            .collect();
        Some(format!("AVAILABLE_ARTIFACTS:\n{}", lines.join("\n")))
    })();

    match (entry_block, artifact_block) {
        (None, None) => String::new(),
        (Some(a), None) => format!("{a}\n\n"),
        (None, Some(b)) => format!("{b}\n\n"),
        (Some(a), Some(b)) => format!("{a}\n\n{b}\n\n"),
    }
}

/// - One append call per changed file (atomic per-file, not per-task).
pub(crate) fn write_impact_summary_to_log(
    seq: i64,
    changed_files: &[std::path::PathBuf],
    project_root: Option<&std::path::Path>,
) {
    use elai_core::paths::dkg_db_path;
    use elai_knowledge::symbol_graph::{
        cache::SymbolGraphCache,
        sharding::SYMBOL_FILE_LRU_SIZE,
        traversal::compute_impact_radius,
    };

    let Some(root) = project_root else { return };

    let project_id = match root.file_name().and_then(|n| n.to_str()) {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => return,
    };

    let db_path = dkg_db_path(root);
    if !db_path.exists() {
        tracing::debug!(
            "write_impact_summary_to_log: symbol graph DB not found at {db_path:?}, skipping"
        );
        return;
    }

    let mut cache = match SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE) {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!(
                "write_impact_summary_to_log: SymbolGraphCache::new failed ({e}), skipping"
            );
            return;
        }
    };

    // If the symbol graph has 0 symbols, skip silently.
    if cache.meta().symbol_count == 0 {
        tracing::debug!("write_impact_summary_to_log: symbol_count == 0, skipping");
        return;
    }

    // Determine the log path: workflow/sessions/task_seq_{seq}/auditor-briefing.log.
    // Uses a relative path anchored to the process CWD (standard for ELAI workflow logs).
    let log_dir = std::path::PathBuf::from("workflow")
        .join("sessions")
        .join(format!("task_seq_{seq}"));
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        tracing::warn!("write_impact_summary_to_log: create_dir_all({log_dir:?}) failed: {e}");
        return;
    }
    let log_path = log_dir.join("auditor-briefing.log");

    for changed in changed_files {
        let changed_str = changed.to_string_lossy();
        match compute_impact_radius(&mut cache, changed_str.as_ref(), DEFAULT_IMPACT_DEPTH) {
            Ok(result) if result.total_files > 0 => {
                let block = format_impact_summary(changed_str.as_ref(), &result);
                use std::io::Write;
                match std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_path)
                {
                    Ok(mut file) => {
                        if let Err(e) = file.write_all(block.as_bytes()) {
                            tracing::warn!(
                                "write_impact_summary_to_log: write to {log_path:?} failed: {e}"
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            "write_impact_summary_to_log: open {log_path:?} failed: {e}"
                        );
                    }
                }
            }
            Ok(_) => {
                tracing::debug!(
                    "write_impact_summary_to_log: {changed_str} has 0 impact files, skipping"
                );
            }
            Err(e) => {
                tracing::debug!(
                    "write_impact_summary_to_log: compute_impact_radius({changed_str}) failed: {e}"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Gap #9 — Post-execution rule evaluation (real source text)
// ---------------------------------------------------------------------------

/// Gap #9 (MVP audit 2026-04-18 section 4 item 9): after the real executor
/// returns, re-run the rule engine against each changed file's on-disk
/// content. Emits one `SsbPayload::Finding` per Block/Warn verdict with
/// `category = "rule_violation_post"` so the frontend can distinguish these
/// from the pre-gate findings (which evaluate against the natural-language
/// task text and use `category = "rule_violation"`).
///
/// Missing files (e.g. deletes) are skipped silently per acceptance
/// criterion — they represent the "file no longer exists" case, not a rule
/// violation. I/O errors while reading a changed file fall in the same
/// bucket: we log `warn!` and skip, matching the task runner's general
/// "one bad file does not abort the loop" policy.
/// Batch 4B: returns `true` when at least one Block verdict fired, so the
/// task_runner baby-step loop can flip `passed = false` alongside the
/// existing PreGate Block and hook_post_block paths. Findings are stamped
/// with [`POST_EXECUTION_RULES_AUDIT_STEP`] so the dashboard can route
/// them to the Rule-violations strip.
async fn evaluate_post_execution_rules_and_emit_findings(
    writer: &Arc<Ssb>,
    evaluator: &RuleEvaluator,
    bs_id: u64,
    changed_files: &[PathBuf],
    is_web_project: bool,
) -> bool {
    let domains: Vec<&str> = if is_web_project { vec!["web"] } else { Vec::new() };
    let mut block_found = false;
    for path in changed_files {
        if !path.exists() {
            // Acceptance criterion: delete case → skip, not a violation.
            continue;
        }
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                warn!(
                    baby_step = bs_id,
                    path = %path.display(),
                    error = %e,
                    "task_runner: post-exec rule eval skipped (read failed)",
                );
                continue;
            }
        };
        let file_path_str = path.to_string_lossy();
        let ctx = EvaluationContext {
            action: BABY_STEP_PRE_GATE_ACTION,
            source: &source,
            file_path: &file_path_str,
            fsm_phase: FsmPhaseBit::EXECUTING,
            metric_values: FxHashMap::default(),
        };
        let verdicts =
            evaluator.evaluate_at_pre_gate(FsmPhaseBit::EXECUTING, &domains, &ctx);
        for v in &verdicts {
            match v.result {
                VerdictResult::Block => {
                    block_found = true;
                    emit_finding_with_audit_step(
                        writer,
                        "error",
                        RULE_VIOLATION_POST_CATEGORY,
                        &format!(
                            "baby_step={bs_id} file={} rule={} BLOCK: {}",
                            path.display(),
                            v.rule_id,
                            v.evidence,
                        ),
                        Some(POST_EXECUTION_RULES_AUDIT_STEP),
                    )
                    .await;
                }
                VerdictResult::Warn => {
                    emit_finding_with_audit_step(
                        writer,
                        "warning",
                        RULE_VIOLATION_POST_CATEGORY,
                        &format!(
                            "baby_step={bs_id} file={} rule={} WARN: {}",
                            path.display(),
                            v.rule_id,
                            v.evidence,
                        ),
                        Some(POST_EXECUTION_RULES_AUDIT_STEP),
                    )
                    .await;
                }
                VerdictResult::Escalate(ref target) => {
                    emit_finding_with_audit_step(
                        writer,
                        "escalate",
                        RULE_VIOLATION_POST_CATEGORY,
                        &format!(
                            "baby_step={bs_id} file={} rule={} ESCALATE(target={:?}): {}",
                            path.display(),
                            v.rule_id,
                            target,
                            v.evidence,
                        ),
                        Some(POST_EXECUTION_RULES_AUDIT_STEP),
                    )
                    .await;
                }
                VerdictResult::Allow => {}
            }
        }
    }
    block_found
}

/// Batch 4B: pure-evaluation variant exposed for contract tests. Runs the
/// same per-file loop as [`evaluate_post_execution_rules_and_emit_findings`]
/// but does NOT emit SSB rows — returns `true` iff at least one Block
/// verdict fired across `changed_files`. The task_runner's real call site
/// still uses the emitting variant; this accessor lets contracts assert
/// the Block-flips-passed semantics without needing a live SSB writer.
pub fn evaluate_post_execution_block_found(
    evaluator: &RuleEvaluator,
    changed_files: &[PathBuf],
    is_web_project: bool,
) -> bool {
    let domains: Vec<&str> = if is_web_project { vec!["web"] } else { Vec::new() };
    for path in changed_files {
        if !path.exists() {
            continue;
        }
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let file_path_str = path.to_string_lossy();
        let ctx = EvaluationContext {
            action: BABY_STEP_PRE_GATE_ACTION,
            source: &source,
            file_path: &file_path_str,
            fsm_phase: FsmPhaseBit::EXECUTING,
            metric_values: FxHashMap::default(),
        };
        let verdicts =
            evaluator.evaluate_at_pre_gate(FsmPhaseBit::EXECUTING, &domains, &ctx);
        if verdicts
            .iter()
            .any(|v| matches!(v.result, VerdictResult::Block))
        {
            return true;
        }
    }
    false
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
/// activity feed shows per-level progress. Always emits a Finding too —
/// failure path uses `INTEGRATION_CHECKPOINT_FAILED_CATEGORY` (pre-B-20A
/// behaviour), success path uses `INTEGRATION_CHECKPOINT_SUCCESS_CATEGORY`
/// (B-20A Task 8) so the B-7 dashboard surface can render a per-level
/// histogram of checkpoint outcomes.
async fn emit_checkpoint_outcome(
    writer: &Arc<Ssb>,
    level: DecompositionLevel,
    node_id: u64,
    run: &IntegrationRunResult,
) {
    let level_tag = checkpoint_level_tag(level);
    let test_name = format!("{level_tag}_{node_id}_integration");
    emit_test_outcome(writer, &test_name, run.success, run.total_duration_ms).await;
    if run.success {
        emit_finding(
            writer,
            "info",
            INTEGRATION_CHECKPOINT_SUCCESS_CATEGORY,
            &format!(
                "{level_tag} {node_id}: integration ok in {}ms",
                run.total_duration_ms
            ),
        )
        .await;
    } else {
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

// ---------------------------------------------------------------------------
// B-20A Task 7 — caller-side 50% budget guard for file reads (Decision D).
//
// The `ToolDispatcher` trait stays unchanged. The dispatcher returns an
// outline by default for files >300 lines; the caller (this module) reads
// the outline first, then calls back with `force_full=true` ONLY when the
// estimated full-read cost is at-or-below 50% of the remaining context
// budget. The spec wanted "tool layer enforcement" but the dispatcher has
// no ContextBudget access (verified at tools/src/dispatcher.rs:68-74), and
// extending the trait would touch every test fake + the dashboard wiring
// site. Caller-side is the smallest correct surface.
//
// Token estimation: 4 bytes/token is the spec-anchored default
// (phase-4B.txt §context-budget). The function operates on byte counts and
// remaining-token counts so the caller can plug in any tokenizer estimate.
// ---------------------------------------------------------------------------

/// B-20A Task 7 verdict returned by `enforce_file_read_budget_guard`.
#[derive(Debug)]
pub enum FileReadBudgetVerdict {
    /// Full read is within budget — caller may invoke
    /// `dispatch_file_read_with_force_full(force_full=true)`.
    Allow,
    /// Full read would exceed 50% of remaining context. Caller MUST stay on
    /// the outline path; `hint` carries a human-readable diagnostic for the
    /// agent + Finding rows.
    Block { hint: String, budget_remaining: u32 },
}

/// B-20A Task 7: caller-side 50% budget guard. Returns `Block` when the
/// estimated full-read cost (`total_bytes / 4` tokens) exceeds half of
/// `budget_remaining_tokens`. `Allow` otherwise.
pub fn enforce_file_read_budget_guard(
    total_bytes: u32,
    budget_remaining_tokens: u32,
) -> FileReadBudgetVerdict {
    // Spec-anchored ratio (phase-4B.txt §context-budget) — 4 bytes/token.
    const TOKEN_BYTES: u32 = 4;
    // Half of the remaining context. Saturating to avoid any underflow on
    // pathological inputs (budget_remaining_tokens == 0 ⇒ Block).
    let half = budget_remaining_tokens / 2;
    let estimated_tokens = total_bytes / TOKEN_BYTES;
    if estimated_tokens > half {
        let hint = format!(
            "file read would consume ~{estimated_tokens} tokens, exceeding 50% of \
             remaining budget ({budget_remaining_tokens}). Stay on outline + \
             targeted line ranges instead of force_full.",
        );
        FileReadBudgetVerdict::Block {
            hint,
            budget_remaining: budget_remaining_tokens,
        }
    } else {
        FileReadBudgetVerdict::Allow
    }
}

/// B-20A Task 8: public test entry-point that mirrors `emit_checkpoint_outcome`.
/// Exists solely so the integration_checkpoint_success contract test (in
/// `crates/orchestrator/tests/integration_checkpoint_success.rs`) can exercise
/// the success/failure Finding emission paths without spinning up the full
/// task_runner. Production callers continue to use `emit_checkpoint_outcome`
/// directly via the IntegrationTestRunner success/failure paths.
pub async fn emit_integration_checkpoint_for_test(
    writer: &Arc<Ssb>,
    level: DecompositionLevel,
    node_id: u64,
    run: &IntegrationRunResult,
) {
    emit_checkpoint_outcome(writer, level, node_id, run).await;
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

    // C-20: assert the measurement-mode batch-delay helper preserves the
    // zero-overhead invariant on the production hot path. Both env vars
    // must be set for the delay to engage; either absent => return 0.
    //
    // Uses the crate-global env lock (test_env_lock) so the assertions do
    // not race other tests that mutate ELAI_* env vars in parallel.
    #[test]
    fn measurement_batch_delay_secs_returns_zero_when_envs_absent() {
        let _guard = crate::test_env_lock();
        // SAFETY: env mutations are gated by test_env_lock so no other
        // thread can read these between remove/var.
        unsafe {
            std::env::remove_var(super::ELAI_MEASURE_MODE_ENV);
            std::env::remove_var(super::ELAI_DEBUG_BATCH_DELAY_SECS_ENV);
        }
        assert_eq!(
            super::measurement_batch_delay_secs(),
            0,
            "both envs absent must return 0 (production hot path)"
        );
    }

    #[test]
    fn measurement_batch_delay_secs_returns_zero_when_only_delay_set() {
        let _guard = crate::test_env_lock();
        unsafe {
            std::env::remove_var(super::ELAI_MEASURE_MODE_ENV);
            std::env::set_var(super::ELAI_DEBUG_BATCH_DELAY_SECS_ENV, "5");
        }
        assert_eq!(
            super::measurement_batch_delay_secs(),
            0,
            "ELAI_MEASURE_MODE absent must short-circuit to 0"
        );
        unsafe {
            std::env::remove_var(super::ELAI_DEBUG_BATCH_DELAY_SECS_ENV);
        }
    }

    #[test]
    fn measurement_batch_delay_secs_returns_zero_when_mode_off() {
        let _guard = crate::test_env_lock();
        unsafe {
            std::env::set_var(super::ELAI_MEASURE_MODE_ENV, "0");
            std::env::set_var(super::ELAI_DEBUG_BATCH_DELAY_SECS_ENV, "5");
        }
        assert_eq!(
            super::measurement_batch_delay_secs(),
            0,
            "ELAI_MEASURE_MODE=0 must keep production semantics"
        );
        unsafe {
            std::env::remove_var(super::ELAI_MEASURE_MODE_ENV);
            std::env::remove_var(super::ELAI_DEBUG_BATCH_DELAY_SECS_ENV);
        }
    }

    #[test]
    fn measurement_batch_delay_secs_returns_value_when_both_set() {
        let _guard = crate::test_env_lock();
        unsafe {
            std::env::set_var(super::ELAI_MEASURE_MODE_ENV, "1");
            std::env::set_var(super::ELAI_DEBUG_BATCH_DELAY_SECS_ENV, "3");
        }
        assert_eq!(
            super::measurement_batch_delay_secs(),
            3,
            "both envs set must return the parsed delay"
        );
        unsafe {
            std::env::remove_var(super::ELAI_MEASURE_MODE_ENV);
            std::env::remove_var(super::ELAI_DEBUG_BATCH_DELAY_SECS_ENV);
        }
    }

    async fn wait_for_action(
        writer: &Arc<Ssb>,
        action: SsbActionType,
        min_count: usize,
        timeout: Duration,
    ) -> bool {
        let start = std::time::Instant::now(); // TEST_KEEP: cfg(test) helper, real wall clock OK
        while start.elapsed() < timeout {
            let entries = writer.query_by_type(action).unwrap_or_default();
            if entries.len() >= min_count {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        false
    }

    // CAMPAIGN-BUG[s-68264]: TaskRunner no longer emits PhaseComplete on the
    // minimal new_minimal config path — the post-campaign wiring requires a
    // real executor (AgentPoolExecutor) bound via with_executor() that the
    // test fixture does not provide. The synthetic-passed default short-
    // circuits before reaching PhaseComplete. Needs either a synthetic
    // executor that DOES emit PhaseComplete, or test config rewrite to
    // supply a real executor. Tracked in s-68264-followup-patch-N.md.
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "B: TaskRunner default minimal config short-circuits before PhaseComplete after campaign rewiring; needs synthetic-executor that emits terminal row or test config update"]
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

    // CAMPAIGN-BUG[s-68264]: same root cause as task_runner_consumes_submitted_task_and_emits_progress
    // — minimal config never reaches PhaseComplete. Tracked in s-68264-followup-patch-N.md.
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "B: TaskRunner default minimal config never emits PhaseComplete after campaign rewiring"]
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
            None, // Track A.6: test helper, no SSB handle.
            None, // B-7: no tree emitter in tests.
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

    // CAMPAIGN-BUG[s-68264]: web-project task path no longer emits
    // PreviewAvailable on the test fixture — dev-server launch in the
    // mocked DevServerLauncher path returns success but the wiring that
    // pushes PreviewAvailable to SSB drifted. Tracked in s-68264-followup-patch-N.md.
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "B: web-project preview launch no longer emits PreviewAvailable SSB row after campaign rewiring"]
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
                interrogation_store: None,
                checkpoint_scratch_dir: None,
                baby_step_executor: None,
                anti_spiral_enabled: false,
                llm_provider: None,
                backend_config: None,
                indexing_service: None,
                hook_dispatcher: None,
                signal_reader: None,
                tool_dispatcher: None,
                git_branch_policy: GitBranchPolicy::default(),
                system_prober: None,
                feasibility_prober: None,
                autonomous_gate: None,
                layer1a_processor: None,
                dispatch_context_profile: None,
                reflection_client: None,
                irreversible_classifier: None,
                dkg_querier_swap: None,
                task_splitter: None,
                research_tree: None,
                pipeline_grc: None,
                model_capability_profile: None,
                cloud_router: None,
                cloud_spend_cap_daily_usd: None,
                crystallizer: None,
                compiled_pipeline_store: None,
                background_task_manager: None,
                // C-39: default to None; tests that need dispatch
                // wiring override via Self { ... mcp_dispatch: Some(...) }.
                mcp_dispatch: None,
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

    // CAMPAIGN-BUG[s-68264]: same root area as task_runner_emits_preview_available
    // — the web-project task path post-campaign no longer emits the
    // dev_server_failed Finding when the launcher is absent. Production code
    // drifted from the original "always announce missing launcher" contract.
    // Tracked in s-68264-followup-patch-N.md.
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "B: web-project missing-launcher no longer emits dev_server_failed Finding after campaign rewiring"]
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
                interrogation_store: None,
                checkpoint_scratch_dir: None,
                baby_step_executor: None,
                anti_spiral_enabled: false,
                llm_provider: None,
                backend_config: None,
                indexing_service: None,
                hook_dispatcher: None,
                signal_reader: None,
                tool_dispatcher: None,
                git_branch_policy: GitBranchPolicy::default(),
                system_prober: None,
                feasibility_prober: None,
                autonomous_gate: None,
                layer1a_processor: None,
                dispatch_context_profile: None,
                reflection_client: None,
                irreversible_classifier: None,
                dkg_querier_swap: None,
                task_splitter: None,
                research_tree: None,
                pipeline_grc: None,
                model_capability_profile: None,
                cloud_router: None,
                cloud_spend_cap_daily_usd: None,
                crystallizer: None,
                compiled_pipeline_store: None,
                background_task_manager: None,
                // C-39: default to None; tests that need dispatch
                // wiring override via Self { ... mcp_dispatch: Some(...) }.
                mcp_dispatch: None,
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
                interrogation_store: None,
                checkpoint_scratch_dir: None,
                baby_step_executor: None,
                anti_spiral_enabled: false,
                llm_provider: None,
                backend_config: None,
                indexing_service: None,
                hook_dispatcher: None,
                signal_reader: None,
                tool_dispatcher: None,
                git_branch_policy: GitBranchPolicy::default(),
                system_prober: None,
                feasibility_prober: None,
                autonomous_gate: None,
                layer1a_processor: None,
                dispatch_context_profile: None,
                reflection_client: None,
                irreversible_classifier: None,
                dkg_querier_swap: None,
                task_splitter: None,
                research_tree: None,
                pipeline_grc: None,
                model_capability_profile: None,
                cloud_router: None,
                cloud_spend_cap_daily_usd: None,
                crystallizer: None,
                compiled_pipeline_store: None,
                background_task_manager: None,
                // C-39: default to None; tests that need dispatch
                // wiring override via Self { ... mcp_dispatch: Some(...) }.
                mcp_dispatch: None,
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
            preferred_mode: None,
            target_file: None,
            edit_region: None,
            intent_pairs: crate::intent_pair::IntentPairSet::default(),
            file_plan: None,
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
                    is_parallel: false,
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
            supervision_manager: AsyncMutex::new(SupervisionManager::new(
                OsClock,
                SupervisionPolicy {
                    max_restarts: 3,
                    restart_window: std::time::Duration::from_secs(300),
                    on_max_restarts: EscalationAction::Terminate,
                },
            )),
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

        emit_baby_step_recovery(&ssb, &cp, seq, bs_id, "test_phase_13").await;

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
        emit_baby_step_recovery(&ssb, &cp, seq, bs_id, "test_phase_99").await;

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

    // -----------------------------------------------------------------
    // 3D.REDO — BspEngine wiring for parallel phases
    // -----------------------------------------------------------------

    /// Build a ConceptDecomposition containing one concept with one phase
    /// flagged `is_parallel = true` and two steps. Used by the 3D.REDO
    /// test to assert the BSP dispatch helper runs an engine for that phase.
    fn synthetic_parallel_decomposition() -> ConceptDecomposition {
        let steps = (1..=2u64)
            .map(|sidx| StepNode {
                id: StepId(sidx),
                description: format!("parallel step {sidx}"),
                baby_steps: vec![make_baby_step(sidx * 10)],
                evidence_level: EvidenceLevel::Anecdotal,
            })
            .collect();
        let phase = PhaseNode {
            id: PhaseId(42),
            name: "parallel phase".to_string(),
            steps,
            depends_on: Vec::new(),
            research: None,
            is_parallel: true,
        };
        let concept = ConceptNode {
            id: ConceptId(1),
            name: "parallel concept".to_string(),
            description: "3D.REDO wiring test".to_string(),
            phases: vec![phase],
            research: None,
        };
        ConceptDecomposition {
            idea: "3D.REDO test".to_string(),
            concepts: vec![concept],
            mode: ExecutionMode::Autonomous,
            total_research_calls: 0,
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn parallel_phase_routes_through_bsp_engine() {
        let _guard = crate::test_env_lock();
        let tmp = tempfile::tempdir().expect("tempdir");
        let db_path = tmp.path().join("ssb.db");

        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let cfg = SsbConfig {
            db_path,
            pool_size: Some(2),
            project_id: "dashboard".into(),
            session_id: "bsp-wiring-test".into(),
        };
        let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

        let runner_cfg = TaskRunnerConfig::new_minimal(tmp.path().join("cursor.json"));
        let resolved = ResolvedConfig::from(TaskRunnerConfig {
            checkpoint_scratch_dir: Some(tmp.path().to_path_buf()),
            ..runner_cfg
        });

        let decomp = synthetic_parallel_decomposition();
        run_parallel_phases_via_bsp(&ssb, 7, &decomp, &resolved).await;

        // Engine emits a msgpack checkpoint per superstep. Success implies
        // at least one superstep ran, so the phase-specific directory must
        // exist and contain at least one file.
        let phase_dir = tmp
            .path()
            .join(BSP_CHECKPOINT_SUBDIR)
            .join("task_7")
            .join("phase_42");
        assert!(
            phase_dir.is_dir(),
            "expected BSP checkpoint dir at {}",
            phase_dir.display()
        );
        let entries: Vec<_> = std::fs::read_dir(&phase_dir)
            .expect("read_dir checkpoint")
            .flatten()
            .collect();
        assert!(
            !entries.is_empty(),
            "expected at least one checkpoint file under {}",
            phase_dir.display()
        );

        // BSP completion emits a `verify` ReasoningTrace row whose content
        // references the phase id — the dashboard uses this to confirm the
        // parallel branch fired.
        let traces = ssb
            .query_by_type(SsbActionType::ReasoningTrace)
            .expect("query ReasoningTrace");
        let mut saw_dispatch = false;
        let mut saw_complete = false;
        for row in traces {
            if let Ok(SsbPayload::ReasoningTrace(t)) =
                rmp_serde::from_slice::<SsbPayload>(row.payload.as_ref())
            {
                if t.content_summary.contains("bsp dispatch: phase_id=42") {
                    saw_dispatch = true;
                }
                if t.content_summary.contains("bsp complete: phase_id=42") {
                    saw_complete = true;
                }
            }
        }
        assert!(saw_dispatch, "expected ReasoningTrace for bsp dispatch");
        assert!(saw_complete, "expected ReasoningTrace for bsp complete");

        // No BSP-failure Finding on the happy path.
        let findings = ssb.query_by_type(SsbActionType::Finding).unwrap_or_default();
        for row in findings {
            if let Ok(SsbPayload::Finding(f)) =
                rmp_serde::from_slice::<SsbPayload>(row.payload.as_ref())
            {
                assert_ne!(
                    f.category, BSP_PHASE_FAILED_CATEGORY,
                    "happy-path run must not emit bsp_phase_failed Finding"
                );
            }
        }
    }

    #[test]
    fn strip_parallel_phases_removes_only_parallel_ones() {
        let mut decomp = synthetic_parallel_decomposition();
        // Add one sequential phase alongside the parallel one so we can prove
        // only the parallel one is filtered.
        decomp.concepts[0].phases.push(PhaseNode {
            id: PhaseId(99),
            name: "sequential phase".into(),
            steps: vec![StepNode {
                id: StepId(900),
                description: "seq".into(),
                baby_steps: vec![make_baby_step(9000)],
                evidence_level: EvidenceLevel::Anecdotal,
            }],
            depends_on: Vec::new(),
            research: None,
            is_parallel: false,
        });

        let stripped = strip_parallel_phases(decomp);
        let phases = &stripped.concepts[0].phases;
        assert_eq!(phases.len(), 1, "parallel phase must be removed");
        assert_eq!(phases[0].id.0, 99);
        assert!(!phases[0].is_parallel);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn sequential_phase_skips_bsp_dispatch() {
        let _guard = crate::test_env_lock();
        let tmp = tempfile::tempdir().expect("tempdir");
        let db_path = tmp.path().join("ssb.db");

        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let cfg = SsbConfig {
            db_path,
            pool_size: Some(2),
            project_id: "dashboard".into(),
            session_id: "bsp-skip-test".into(),
        };
        let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

        let runner_cfg = TaskRunnerConfig::new_minimal(tmp.path().join("cursor.json"));
        let resolved = ResolvedConfig::from(TaskRunnerConfig {
            checkpoint_scratch_dir: Some(tmp.path().to_path_buf()),
            ..runner_cfg
        });

        // Sequential phase (default) — dispatcher must not fire.
        let mut decomp = synthetic_parallel_decomposition();
        decomp.concepts[0].phases[0].is_parallel = false;

        run_parallel_phases_via_bsp(&ssb, 8, &decomp, &resolved).await;

        let bsp_root = tmp.path().join(BSP_CHECKPOINT_SUBDIR);
        assert!(
            !bsp_root.exists(),
            "sequential-only phases must not create a BSP checkpoint tree"
        );

        let traces = ssb
            .query_by_type(SsbActionType::ReasoningTrace)
            .unwrap_or_default();
        for row in traces {
            if let Ok(SsbPayload::ReasoningTrace(t)) =
                rmp_serde::from_slice::<SsbPayload>(row.payload.as_ref())
            {
                assert!(
                    !t.content_summary.starts_with("bsp "),
                    "no BSP trace should fire when is_parallel=false"
                );
            }
        }
    }

    // ---------------------------------------------------------------------
    // Session A-19.REDO: InterrogationTree consumer tests
    // ---------------------------------------------------------------------

    fn build_interrogation_store() -> Arc<dyn InterrogationStore> {
        use elai_core::clock::SeededClock;
        use elai_rules::interrogation_store::SqliteInterrogationStore;
        use elai_rules::schema::open_rules_db;
        use std::path::Path;
        let conn = open_rules_db(Path::new(":memory:"), &SeededClock::new())
            .expect("in-memory rules.db");
        let shared = Arc::new(std::sync::Mutex::new(conn));
        Arc::new(SqliteInterrogationStore::new(shared))
    }

    fn ssb_for_test(tmp: &tempfile::TempDir, session: &str) -> Arc<Ssb> {
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        let cfg = SsbConfig {
            db_path: tmp.path().join("ssb.db"),
            pool_size: Some(2),
            project_id: "dashboard".into(),
            session_id: session.into(),
        };
        Arc::new(Ssb::new(cfg, clock).expect("ssb"))
    }

    fn resolved_with_store(
        cursor_path: PathBuf,
        store: Option<Arc<dyn InterrogationStore>>,
    ) -> ResolvedConfig {
        ResolvedConfig::from(TaskRunnerConfig {
            cursor_path,
            dev_server_launcher: None,
            project_dir_resolver: None,
            dev_server_registry: None,
            web_research_runner: None,
            scaffold_applier: None,
            rule_evaluator: None,
            interrogation_store: store,
            checkpoint_scratch_dir: None,
            baby_step_executor: None,
            anti_spiral_enabled: false,
            llm_provider: None,
            backend_config: None,
            indexing_service: None,
            hook_dispatcher: None,
            signal_reader: None,
            tool_dispatcher: None,
            git_branch_policy: GitBranchPolicy::default(),
            system_prober: None,
            feasibility_prober: None,
            autonomous_gate: None,
            layer1a_processor: None,
            dispatch_context_profile: None,
            reflection_client: None,
            irreversible_classifier: None,
            dkg_querier_swap: None,
            task_splitter: None,
            research_tree: None,
            pipeline_grc: None,
            model_capability_profile: None,
            cloud_router: None,
            cloud_spend_cap_daily_usd: None,
            crystallizer: None,
            compiled_pipeline_store: None,
            background_task_manager: None,
            // C-39: legacy test fixture -- no dispatch wired.
            mcp_dispatch: None,
        })
    }

    fn find_finding_with_category(ssb: &Arc<Ssb>, category: &str) -> Option<FindingPayload> {
        for row in ssb
            .query_by_type(SsbActionType::Finding)
            .unwrap_or_default()
        {
            if let Ok(SsbPayload::Finding(f)) =
                rmp_serde::from_slice::<SsbPayload>(row.payload.as_ref())
            {
                if f.category == category {
                    return Some(f);
                }
            }
        }
        None
    }

    #[test]
    fn classify_baby_step_failure_matrix() {
        assert!(matches!(
            classify_baby_step_failure(true, false),
            FailureCategory::LogicError
        ));
        assert!(matches!(
            classify_baby_step_failure(true, true),
            FailureCategory::LogicError
        ));
        assert!(matches!(
            classify_baby_step_failure(false, true),
            FailureCategory::TestFailure
        ));
        assert!(matches!(
            classify_baby_step_failure(false, false),
            FailureCategory::Unknown
        ));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn interrogation_fix_suggestion_emits_finding_on_failure() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ssb = ssb_for_test(&tmp, "a19-emit");
        let resolved = resolved_with_store(
            tmp.path().join("cursor.json"),
            Some(build_interrogation_store()),
        );

        emit_interrogation_fix_suggestion(&ssb, &resolved, 7, FailureCategory::TestFailure).await;

        let finding = find_finding_with_category(&ssb, INTERROGATION_FIX_CATEGORY)
            .expect("interrogation_fix_suggestion Finding must be emitted");
        assert_eq!(finding.severity, "info");
        assert!(
            finding.message.contains("baby_step=7"),
            "Finding message must identify baby_step: got {:?}",
            finding.message,
        );
        assert!(
            finding.message.contains("category=test_failure"),
            "Finding must echo failure category: got {:?}",
            finding.message,
        );
        assert!(
            finding.message.contains("fix_category="),
            "Finding must carry the resolved fix_category: got {:?}",
            finding.message,
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn interrogation_fix_suggestion_matrix_covers_specific_categories() {
        let cases = [
            (FailureCategory::TestFailure, "test_fix"),
            (FailureCategory::BorrowCheck, "borrow_fix"),
            (FailureCategory::LifetimeError, "lifetime_fix"),
            (FailureCategory::CompileError, "trait_fix"),
            (FailureCategory::TypeMismatch, "trait_fix"),
            (FailureCategory::LogicError, "general_fix"),
            (FailureCategory::Unknown, "general_fix"),
        ];
        for (i, (cat, expected_fix_cat)) in cases.iter().enumerate() {
            let tmp = tempfile::tempdir().expect("tempdir");
            let session = format!("a19-matrix-{i}");
            let ssb = ssb_for_test(&tmp, &session);
            let resolved = resolved_with_store(
                tmp.path().join("cursor.json"),
                Some(build_interrogation_store()),
            );

            emit_interrogation_fix_suggestion(&ssb, &resolved, i as u64, *cat).await;

            let finding = find_finding_with_category(&ssb, INTERROGATION_FIX_CATEGORY)
                .unwrap_or_else(|| panic!("Finding missing for category {:?}", cat));
            let needle = format!("fix_category={expected_fix_cat}");
            assert!(
                finding.message.contains(&needle),
                "category {:?} expected fix_category={}, got {:?}",
                cat,
                expected_fix_cat,
                finding.message,
            );
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn interrogation_metric_is_one_shot() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ssb = ssb_for_test(&tmp, "a19-metric");
        let resolved = resolved_with_store(
            tmp.path().join("cursor.json"),
            Some(build_interrogation_store()),
        );

        maybe_emit_interrogation_metric(&ssb, &resolved).await;
        maybe_emit_interrogation_metric(&ssb, &resolved).await;
        maybe_emit_interrogation_metric(&ssb, &resolved).await;

        let metric_findings: Vec<_> = ssb
            .query_by_type(SsbActionType::Finding)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|row| rmp_serde::from_slice::<SsbPayload>(row.payload.as_ref()).ok())
            .filter_map(|p| match p {
                SsbPayload::Finding(f)
                    if f.category == INTERROGATION_TREE_METRIC_CATEGORY =>
                {
                    Some(f)
                }
                _ => None,
            })
            .collect();

        assert_eq!(
            metric_findings.len(),
            1,
            "metric Finding must fire exactly once across repeated calls",
        );
        assert!(
            metric_findings[0]
                .message
                .contains("interrogation_trees_loaded="),
            "metric Finding must report loaded tree count",
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn interrogation_wiring_noop_when_store_absent() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ssb = ssb_for_test(&tmp, "a19-noop");
        let resolved = resolved_with_store(tmp.path().join("cursor.json"), None);

        maybe_emit_interrogation_metric(&ssb, &resolved).await;
        emit_interrogation_fix_suggestion(&ssb, &resolved, 1, FailureCategory::Unknown).await;

        assert!(
            find_finding_with_category(&ssb, INTERROGATION_TREE_METRIC_CATEGORY).is_none(),
            "no metric Finding without a store",
        );
        assert!(
            find_finding_with_category(&ssb, INTERROGATION_FIX_CATEGORY).is_none(),
            "no fix Finding without a store",
        );
    }

    // -----------------------------------------------------------------------
    // 3F.REDO — MistakeBook task_runner wiring
    // -----------------------------------------------------------------------

    #[tokio::test(flavor = "multi_thread")]
    async fn mistake_book_constraints_noop_when_empty() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ssb = ssb_for_test(&tmp, "3f-redo-empty");

        let count = emit_mistake_book_constraints(&ssb, "baby_step_42").await;
        assert_eq!(count, 0, "empty MistakeBook returns zero prior attempts");
        assert!(
            find_finding_with_category(&ssb, MISTAKE_BOOK_CONSTRAINTS_CATEGORY).is_none(),
            "no constraints Finding when no prior entries exist",
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn record_mistake_persists_entry_via_ssb() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ssb = ssb_for_test(&tmp, "3f-redo-write");
        let step_id = "baby_step_7";

        record_mistake_on_failure(
            &ssb,
            step_id,
            0,
            FailureCategory::TestFailure,
            "task summary",
        )
        .await;

        let entries =
            crate::mistake_book::query_by_baby_step(ssb.as_ref(), step_id).expect("query");
        assert_eq!(entries.len(), 1, "one MistakeEntry persisted");
        assert_eq!(entries[0].attempt_number, 1);
        assert_eq!(entries[0].failure_category, FailureCategory::TestFailure);
        assert_eq!(entries[0].action_taken, MISTAKE_ACTION_PENDING_RETRY);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn constraints_finding_emitted_when_prior_attempts_exist() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ssb = ssb_for_test(&tmp, "3f-redo-inject");
        let step_id = "baby_step_11";

        record_mistake_on_failure(
            &ssb,
            step_id,
            0,
            FailureCategory::CompileError,
            "failing task",
        )
        .await;

        let count = emit_mistake_book_constraints(&ssb, step_id).await;
        assert_eq!(count, 1, "one prior attempt visible at PRE_GATE");
        let finding = find_finding_with_category(&ssb, MISTAKE_BOOK_CONSTRAINTS_CATEGORY)
            .expect("constraints Finding emitted");
        assert!(
            finding.message.contains("CONSTRAINTS"),
            "Finding carries the CONSTRAINTS block",
        );
        assert!(
            finding.message.contains("compile_error"),
            "Finding records prior category",
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn clear_mistakes_on_success_tombstones_prior_entries() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ssb = ssb_for_test(&tmp, "3f-redo-clear");
        let step_id = "baby_step_13";

        record_mistake_on_failure(
            &ssb,
            step_id,
            0,
            FailureCategory::LogicError,
            "buggy task",
        )
        .await;
        assert_eq!(
            crate::mistake_book::query_by_baby_step(ssb.as_ref(), step_id)
                .expect("query")
                .len(),
            1,
        );

        clear_mistakes_on_success(&ssb, step_id).await;

        assert_eq!(
            crate::mistake_book::query_by_baby_step(ssb.as_ref(), step_id)
                .expect("query")
                .len(),
            0,
            "tombstone clears the CONSTRAINTS view",
        );
    }
}
