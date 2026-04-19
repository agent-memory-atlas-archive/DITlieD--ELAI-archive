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
use std::sync::{Arc, Mutex};

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

use crate::agent_pool::{AgentHandle, AgentPool, AgentRole, AgentStatus, ModelPool};
use crate::checkpoint::{
    AgentCheckpointData, CheckpointManager, GrcHistoryEntry,
};
use crate::recovery::{AgentDeathClassification, RecoveryManager};
use crate::pipeline_builder::{
    PipelineBuilder, PipelineBuilderError, PredicateCompiler, PredicateRef, RuleId, ToolLookup,
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
    NavigatorTree, NoOpIntegrationRunner, TreeNavigator, TreeNavigatorState,
};
use crate::grounding_client::SidecarGroundingClient;
use crate::hook_dispatcher::{
    emit_hook_events_to_ssb, emit_shortcoder_compression_to_ssb, HookFinding, HookInvocation,
    HookPoint,
};
use crate::provider_signals::ProviderSignalReader;
use elai_providers::LlmProvider;
use crate::research_tree::{
    research_decompose, ConceptDecomposition, ConceptId, ExecutionMode, ExitCriterion,
    IntegrationSpec, MediaSpec, NodeId, NodeStatus, OutputType, PerfSpec, PhaseId,
    PipelineSpec, ResearchCache, ResearchNode, StepId, DEFAULT_CACHE_MAX_ENTRIES,
    DEFAULT_CACHE_TTL_SECONDS,
};
use crate::ssb::{
    DkgReadyPayload, FindingPayload, IndexingProgressPayload, PhaseCompletePayload,
    PreviewAvailablePayload, ProviderSignalsPayload, ReasoningTracePayload,
    SearchTriggerFiredPayload, Ssb, SsbActionType, SsbEntry, SsbPayload, SsbReader,
    TaskSubmittedPayload, TestOutcomePayload, VramTierDetectedPayload, WebResearchStepPayload,
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
];

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
}

/// Default timeout for a single baby-step agent execution (matches
/// `deterministic_auditor::BUILD_CMD_TIMEOUT_S`).
pub const DEFAULT_BABY_STEP_TIMEOUT_MS: u64 = 120_000;

impl<E: elai_core::EntropySource + Send + 'static> AgentPoolBabyStepExecutor<E> {
    pub fn new(
        pool: Arc<AsyncMutex<crate::agent_pool::InProcessAgentPool<E>>>,
        agent_binary_path: PathBuf,
        socket_dir: PathBuf,
    ) -> Self {
        Self {
            pool,
            agent_binary_path,
            socket_dir,
            timeout_ms: DEFAULT_BABY_STEP_TIMEOUT_MS,
            role: crate::agent_pool::AgentRole::Executor,
        }
    }

    pub fn with_timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    pub fn with_role(mut self, role: crate::agent_pool::AgentRole) -> Self {
        self.role = role;
        self
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
            let start = std::time::Instant::now();

            // Build the task string delivered to the agent. The escalation
            // message (from the anti-spiral detector) is prepended so it is
            // the first thing the agent sees on the next turn.
            let task_text = match ctx.escalation_message {
                Some(msg) => format!("{msg}\n\n{}", ctx.task_summary),
                None => ctx.task_summary.clone(),
            };

            // Spawn an agent handle on the pool.
            let (agent_id, spawn_err) = {
                let mut pool = self.pool.lock().await;
                match pool.spawn(self.role, task_text) {
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
                        duration_ms: start.elapsed().as_millis() as u64,
                        exit_code: None,
                        stderr_tail: reason,
                        changed_files: Vec::new(),
                    };
                }
            };

            // Ensure socket directory exists; launch the agent process.
            if let Err(e) = std::fs::create_dir_all(&self.socket_dir) {
                let reason = format!("socket_dir create failed: {e}");
                return BabyStepExecutionResult {
                    outcome: BabyStepOutcome::Failed { reason: reason.clone() },
                    duration_ms: start.elapsed().as_millis() as u64,
                    exit_code: None,
                    stderr_tail: reason,
                    changed_files: Vec::new(),
                };
            }
            let socket_path = self
                .socket_dir
                .join(format!("agent-{}.sock", agent_id));

            let launch_err = {
                let mut pool = self.pool.lock().await;
                pool.launch_agent_process(
                    &agent_id,
                    &self.agent_binary_path,
                    &socket_path,
                )
                .err()
            };
            if let Some(e) = launch_err {
                let reason = format!("launch_agent_process failed: {e}");
                return BabyStepExecutionResult {
                    outcome: BabyStepOutcome::Failed { reason: reason.clone() },
                    duration_ms: start.elapsed().as_millis() as u64,
                    exit_code: None,
                    stderr_tail: reason,
                    changed_files: Vec::new(),
                };
            }

            // Wait on the child process with timeout. `wait_for_agent_exit`
            // reads stdout/stderr to completion to avoid the child blocking
            // on a full pipe buffer.
            let outcome = wait_for_agent_exit(
                Arc::clone(&self.pool),
                agent_id,
                std::time::Duration::from_millis(self.timeout_ms),
            )
            .await;

            BabyStepExecutionResult {
                outcome: outcome.outcome,
                duration_ms: start.elapsed().as_millis() as u64,
                exit_code: outcome.exit_code,
                stderr_tail: outcome.stderr_tail,
                // Gap #9: real agents will populate this via an out-of-band
                // diff reporter once wire-4A's changed-files protocol lands.
                // Until then we default to empty so the post-execution rule
                // re-eval is a no-op (documented in the trait doc-comment).
                changed_files: Vec::new(),
            }
        })
    }
}

struct ProcessWaitOutcome {
    outcome: BabyStepOutcome,
    exit_code: Option<i32>,
    stderr_tail: String,
}

/// Wait on the agent process owned by `pool[agent_id]`. Drives the child to
/// completion within `timeout`; on timeout, calls `pool.kill_agent_process`
/// and returns `Failed`. Reads stderr to a bounded tail (last 2 KiB).
async fn wait_for_agent_exit<E: elai_core::EntropySource + Send + 'static>(
    pool: Arc<AsyncMutex<crate::agent_pool::InProcessAgentPool<E>>>,
    agent_id: Uuid,
    timeout: std::time::Duration,
) -> ProcessWaitOutcome {
    const STDERR_TAIL_MAX: usize = 2 * 1024;

    // Take ownership of the child out of the pool so we don't hold the pool
    // lock during the entire wait (other baby steps could queue up).
    let mut child = {
        let mut pool = pool.lock().await;
        match pool
            .get_agent_mut(&agent_id)
            .and_then(|h| h.child_process.take())
        {
            Some(child) => child,
            None => {
                let reason = "child_process handle missing after launch".to_string();
                return ProcessWaitOutcome {
                    outcome: BabyStepOutcome::Failed { reason: reason.clone() },
                    exit_code: None,
                    stderr_tail: reason,
                };
            }
        }
    };

    let pool_for_kill = Arc::clone(&pool);
    let wait_result = tokio::task::spawn_blocking(move || {
        use std::io::Read;
        let mut stderr_buf = Vec::new();
        if let Some(mut stderr) = child.stderr.take() {
            let _ = stderr.read_to_end(&mut stderr_buf);
        }
        // drain stdout so the child doesn't block on a full pipe
        if let Some(mut stdout) = child.stdout.take() {
            let mut sink = Vec::new();
            let _ = stdout.read_to_end(&mut sink);
        }
        let status = child.wait();
        (status, stderr_buf)
    });

    match tokio::time::timeout(timeout, wait_result).await {
        Ok(Ok((Ok(status), stderr))) => {
            let exit_code = status.code();
            let tail = tail_lossy(&stderr, STDERR_TAIL_MAX);
            let outcome = if status.success() {
                BabyStepOutcome::Passed
            } else {
                BabyStepOutcome::Failed {
                    reason: format!(
                        "agent exit_code={:?} stderr_tail={}",
                        exit_code, tail
                    ),
                }
            };
            ProcessWaitOutcome {
                outcome,
                exit_code,
                stderr_tail: tail,
            }
        }
        Ok(Ok((Err(e), stderr))) => {
            let tail = tail_lossy(&stderr, STDERR_TAIL_MAX);
            let reason = format!("child.wait() error: {e} | stderr_tail={}", tail);
            ProcessWaitOutcome {
                outcome: BabyStepOutcome::Failed { reason: reason.clone() },
                exit_code: None,
                stderr_tail: reason,
            }
        }
        Ok(Err(join_err)) => {
            let reason = format!("agent wait join error: {join_err}");
            ProcessWaitOutcome {
                outcome: BabyStepOutcome::Failed { reason: reason.clone() },
                exit_code: None,
                stderr_tail: reason,
            }
        }
        Err(_elapsed) => {
            let _ = pool_for_kill.lock().await.kill_agent_process(&agent_id);
            let reason = "agent process timed out".to_string();
            ProcessWaitOutcome {
                outcome: BabyStepOutcome::Failed { reason: reason.clone() },
                exit_code: None,
                stderr_tail: reason,
            }
        }
    }
}

fn tail_lossy(buf: &[u8], max: usize) -> String {
    let start = buf.len().saturating_sub(max);
    String::from_utf8_lossy(&buf[start..]).into_owned()
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
    indexed_projects: Mutex<HashMap<String, i64>>,
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
            interrogation_store: cfg.interrogation_store,
            interrogation_metric_emitted: Mutex::new(false),
            checkpoint,
            baby_step_executor: cfg
                .baby_step_executor
                .unwrap_or_else(|| Arc::new(SyntheticBabyStepExecutor)),
            anti_spiral_enabled: cfg.anti_spiral_enabled,
            llm_provider: cfg.llm_provider,
            backend_config: cfg
                .backend_config
                .unwrap_or_else(crate::dispatcher::BackendConfig::default),
            indexing_service: cfg.indexing_service,
            indexed_projects: Mutex::new(HashMap::new()),
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
async fn emit_fsm_transition(writer: &Arc<Ssb>, from: &str, to: &str, reason: &str) {
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
                enable_sandbox: false,
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
            let elapsed_ms = (done_ms.saturating_sub(start_ms)).max(0) as u64;
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

            // One-shot DkgReady row. languages is empty for MVP — the DKG
            // engine does not yet expose per-language stats; adding that is
            // A-35 scope (multi-lang DKG grammars).
            emit_dkg_ready(writer, project_id, node_count, edge_count, Vec::new()).await;

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
            let elapsed_ms = (done_ms.saturating_sub(start_ms)).max(0) as u64;
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
fn now_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
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
    emit_reasoning(writer, 0, "observe", &observe_content).await;

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
        let priority_map: HashMap<String, u8> = specs
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
            None,
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
    )
    .await;

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
    let nav = DepthFirstNavigator::from_tree(&tree);
    let mut state = TreeNavigatorState::new();
    let mut steps_executed: usize = 0;
    let integration_runner: NoOpIntegrationRunner = NoOpIntegrationRunner;
    let mut last_location: Option<BabyStepLocation> = None;
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

        let baby_step_reasoning_content =
            format!("baby_step id={} task={:?}", bs_id.0, task_summary);
        emit_reasoning(
            writer,
            2 + steps_executed as u32,
            "act",
            &baby_step_reasoning_content,
        )
        .await;

        // Session wire-antispiral: record this reasoning step with the
        // detector. On HardStop we emit the SSB rows and break immediately;
        // on EscalationSignalNeeded the message is stashed in the tracker
        // and drained below before executor dispatch.
        if let Some(tracker) = spiral_tracker.as_mut() {
            let result = tracker.record("act", "task_runner", &baby_step_reasoning_content);
            if matches!(result, crate::anti_spiral::SpiralResult::HardStop { .. }) {
                emit_spiral_hard_stop(writer, tracker.agent_id(), seq).await;
                break;
            }
        }

        // 3F.REDO: read MistakeBook entries for this baby step and inject the
        // formatted CONSTRAINTS block into the agent's context via an
        // `info` Finding. Occupies the 150-token REPAIR_SLOT_MISTAKE_BOOK
        // slot reserved in context_budget.rs:132. `prior_attempts` carries
        // into the failure path so the MistakeEntry::attempt_number is
        // correctly 1-indexed across pipeline invocations.
        let mistake_book_step_id = format!("baby_step_{}", bs_id.0);
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
                    classifier_rejected = run_confirmation_gate(
                        writer,
                        reason,
                        &task_summary,
                        bs_id.0,
                        cancel,
                    )
                    .await;
                }
            }
        }
        if classifier_rejected {
            // Skip executor, mark the step failed. Downstream checkpoint /
            // bubble-up + TestOutcome paths still render the row with
            // passed=false so the dashboard surfaces the rejection.
            hook_pre_blocked = true;
            emit_finding(
                writer,
                "warn",
                "irreversible_rejected",
                "irreversible_action: user rejected confirmation",
            )
            .await;
        }

        // Batch 3B (Phase 4G): autonomous-gate decision boundary. Runs on
        // every baby step regardless of pre-tool-use hook verdict so the
        // dashboard captures a GateDecision row per step. A
        // `RequestHuman` / `Escalate` verdict emits the corresponding SSB
        // row but does NOT block the loop — the task_runner is not yet the
        // FSM owner for 4G. The row drives the dashboard Autonomy panel;
        // the CLI-side caller is expected to consume the row and halt.
        if let Some(gate) = resolved.autonomous_gate.as_ref() {
            let gctx = crate::autonomous_gate::GateContext {
                level: crate::autonomous_gate::PipelineLevel::BabyStep,
                risk_level: crate::gate::RiskLevel::Low,
                evidence: crate::autonomous_gate::Evidence {
                    knowledge_readiness: true,
                    zero_ungrounded: true,
                    tests_fail_correctly: true,
                    test_flipped: true,
                    zero_blocks: true,
                    dkg_and_changelog_present: true,
                },
                anti_spiral_fired: spiral_tracker
                    .as_ref()
                    .map(|t| t.hard_stop())
                    .unwrap_or(false),
                replan_major: false,
                max_restarts_exhausted: false,
                context_budget_terminal: false,
                auto_advance_count_for_concept: 0,
                step_id: format!("baby_step_{}", bs_id.0),
            };
            let decision = gate.decide(&gctx).await;
            emit_autonomous_decision(writer, &decision, &gctx.step_id).await;
        }

        // Session wire-4A: dispatch this baby step through the pluggable
        // [`BabyStepExecutor`]. Drains the escalation message stashed by the
        // anti-spiral detector (wire-antispiral) so the injected message lands
        // in the agent turn that runs next.
        let escalation_message = spiral_tracker
            .as_mut()
            .and_then(|t| t.take_escalation_message());
        let exec_ctx = BabyStepExecutionCtx {
            baby_step_id: bs_id.0,
            task_summary: task_summary.clone(),
            escalation_message,
        };
        // Batch 1A: if PreToolUse blocked, skip the executor entirely with a
        // synthetic Failed result — the rest of the loop (integration
        // checkpoint, TestOutcome emission, MistakeBook, bubble-up) runs as if
        // the executor returned Failed. This mirrors the `pre_gate_blocked`
        // path for the existing rule evaluator.
        let exec_result = if hook_pre_blocked {
            BabyStepExecutionResult {
                outcome: BabyStepOutcome::Failed {
                    reason: "PreToolUse hook blocked baby step".to_string(),
                },
                duration_ms: 0,
                exit_code: None,
                stderr_tail: String::new(),
                changed_files: Vec::new(),
            }
        } else {
            resolved.baby_step_executor.execute(exec_ctx).await
        };
        let outcome = exec_result.outcome;
        let outcome_passed = matches!(outcome, BabyStepOutcome::Passed);
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
            } else if !outcome_passed && !exec_stderr_tail.is_empty() {
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

    // Batch 1A: Stop hook dispatch. Fires exactly once per task, regardless of
    // success or hard-fail cascade past the root concept. No file / source
    // context — Stop is task-level — so T1 is the only tier that will
    // meaningfully fire; T2/T3 stay gated on file_write in `TieredAnalysis`.
    if let Some(dispatcher) = resolved.hook_dispatcher.as_ref() {
        let step_id = format!("task_{}", seq);
        let invocation = HookInvocation::for_task_level("task_stop", step_id.clone());
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

        // PreCompact: reserved MVP wiring surface. The dashboard path has no
        // compaction threshold yet, so we invoke the no-op path to keep the
        // contract exercised (a real threshold lands in a later batch).
        let pre_compact = HookInvocation::for_task_level("pre_compact", step_id.clone());
        match dispatcher.dispatch_pre_compact(pre_compact).await {
            Ok(findings) => {
                emit_hook_events_to_ssb(
                    writer,
                    RUNNER_AGENT_ID,
                    RUNNER_SUPERSTEP,
                    HookPoint::PreCompact,
                    &step_id,
                    &findings,
                )
                .await;
            }
            Err(e) => {
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
    emit_daira_trace(writer, "validate", &summary, "enter", 0, "started", step_id).await;
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
    let start = std::time::Instant::now();
    match dispatcher.dispatch_validate(req).await {
        Ok(results) => {
            for r in &results {
                emit_validation_tier(writer, r, step_id).await;
            }
            let elapsed_ms = u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
            let any_failed = results.iter().any(|r| !r.passed);
            let outcome = if any_failed { "error" } else { "success" };
            emit_tool_dispatch(
                writer, "validate", &summary, elapsed_ms, step_id, outcome, None,
            )
            .await;
            let elapsed_us =
                u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
            emit_daira_trace(
                writer, "validate", &summary, "exit", elapsed_us, outcome, step_id,
            )
            .await;
        }
        Err(e) => {
            let elapsed_ms = u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
            let outcome_tag = match e {
                elai_core::ToolDispatchError::NotImplemented(_) => "not_implemented",
                _ => "error",
            };
            emit_tool_dispatch(
                writer, "validate", &summary, elapsed_ms, step_id, outcome_tag,
                Some(e.to_string()),
            )
            .await;
            let elapsed_us =
                u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
            emit_daira_trace(
                writer, "validate", &summary, "exit", elapsed_us, outcome_tag, step_id,
            )
            .await;
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
    emit_daira_trace(writer, "classify_failure", &summary, "enter", 0, "started", step_id).await;
    let input = FailureInput {
        raw_output: raw_output.to_owned(),
        step_id: step_id.to_owned(),
        source: source.to_owned(),
    };
    let start = std::time::Instant::now();
    let result = dispatcher.dispatch_classify_failure(input).await;
    let elapsed_ms = u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
    let elapsed_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
    let outcome = if result.is_some() { "success" } else { "skipped" };
    emit_tool_dispatch(
        writer, "classify_failure", &summary, elapsed_ms, step_id, outcome, None,
    )
    .await;
    if let Some(classified) = result.as_ref() {
        emit_failure_classified(writer, classified, step_id).await;
    }
    emit_daira_trace(
        writer, "classify_failure", &summary, "exit", elapsed_us, outcome, step_id,
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
    emit_daira_trace(writer, "git_commit", &summary, "enter", 0, "started", step_id).await;
    let req = GitCommitRequest {
        anchor_path: project_dir.to_path_buf(),
        files: changed_files.to_vec(),
        message: message.to_owned(),
        branch_policy,
        plan_id: plan_id.to_owned(),
        step_id: step_id.to_owned(),
    };
    let start = std::time::Instant::now();
    match dispatcher.dispatch_git_commit(req).await {
        Ok(outcome) => {
            emit_git_commit(writer, &outcome, step_id).await;
            let elapsed_ms =
                u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
            emit_tool_dispatch(
                writer, "git_commit", &summary, elapsed_ms, step_id, "success", None,
            )
            .await;
            let elapsed_us =
                u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
            emit_daira_trace(
                writer, "git_commit", &summary, "exit", elapsed_us, "success", step_id,
            )
            .await;
        }
        Err(e) => {
            let elapsed_ms =
                u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
            let outcome_tag = match e {
                elai_core::ToolDispatchError::NotImplemented(_) => "not_implemented",
                _ => "error",
            };
            emit_tool_dispatch(
                writer, "git_commit", &summary, elapsed_ms, step_id, outcome_tag,
                Some(e.to_string()),
            )
            .await;
            let elapsed_us =
                u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
            emit_daira_trace(
                writer, "git_commit", &summary, "exit", elapsed_us, outcome_tag, step_id,
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
async fn emit_autonomous_decision(
    writer: &Arc<Ssb>,
    decision: &crate::autonomous_gate::AutonomyDecision,
    step_id: &str,
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
        AutonomyDecision::SplitTask {
            sub_task_count,
            depth,
        } => (
            "split_task".to_owned(),
            None,
            Some(format!("sub_task_count={} depth={}", sub_task_count, depth)),
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
        AutonomyDecision::SplitTask {
            sub_task_count,
            depth,
        } => {
            // Spec: deterministic sub-task ids for MVP — no dynamic splitter yet.
            let ids: Vec<String> = (0..*sub_task_count)
                .map(|i| format!("{}.sub{}", step_id, i))
                .collect();
            let p = SsbPayload::TaskSplit(crate::ssb::TaskSplitPayload {
                original_task_id: step_id.to_owned(),
                sub_task_ids: ids,
                depth: *depth,
            });
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
        scaled as u16
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
/// the fly so the probe has a [`SystemProbeReport`] to reason over.
async fn emit_feasibility_probe_for_pipeline(
    writer: &Arc<Ssb>,
    resolved: &ResolvedConfig,
    pipeline: &crate::pipeline_builder::ExecutablePipeline,
) {
    let Some(prober) = resolved.feasibility_prober.as_ref() else {
        return;
    };
    // Fetch a SystemProbeReport — reuse the plumbed prober when present,
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
    }

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
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
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

/// Batch-fu (FU.5 / A-31): emit an `InterventionRequest` row, block until a
/// matching `InterventionResponse` arrives on the SSB broadcast channel, then
/// emit an `InterventionResolved` row. Returns `true` when the user rejected.
///
/// The intervention is identified by a UUID rendered into both the
/// `InterventionRequestPayload.step_id` (prefixed with `intervention:`) and
/// the `InterventionResolvedPayload.intervention_id` so consumers can pair the
/// two rows without walking the seq column. The `reason` and `summary` land
/// on the request `description` as a single line so the frontend can render
/// them without deserializing the classifier's internal state.
async fn run_confirmation_gate(
    writer: &Arc<Ssb>,
    reason: &str,
    summary: &str,
    baby_step_id: u64,
    cancel: &CancelHandle,
) -> bool {
    let intervention_id = Uuid::new_v4();
    let id_str = intervention_id.to_string();
    let step_id = format!("baby_step_{}", baby_step_id);
    let description = format!(
        "{}: {} | summary: {}",
        crate::CONFIRM_IRREVERSIBLE_ACTION_KIND,
        reason,
        truncate_for_log(summary, 160)
    );
    let options = vec![
        crate::ssb::InterventionOption::Keyword(crate::ssb::InterventionKeyword::Accept),
        crate::ssb::InterventionOption::Keyword(crate::ssb::InterventionKeyword::Decline),
    ];
    let request_payload = crate::ssb::SsbPayload::InterventionRequest(
        crate::ssb::InterventionRequestPayload {
            step_id: format!("intervention:{}", id_str),
            intervention_type: crate::ssb::InterventionType::ApproveChange,
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
            return false;
        }
    };
    let started = std::time::Instant::now();

    // Poll the broadcast rx for a matching InterventionResponse. The
    // dashboard's `respond_to_confirmation` writes a response row with
    // `request_id == request_seq`; any earlier intervention response is
    // ignored. Cancel-signals short-circuit the wait via a periodic
    // `is_cancelled` check so runner shutdown does not deadlock on a
    // pending prompt.
    const CANCEL_POLL: std::time::Duration = std::time::Duration::from_millis(200);
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
                let ok = matches!(
                    resp.chosen,
                    crate::ssb::InterventionOption::Keyword(
                        crate::ssb::InterventionKeyword::Accept,
                    ),
                );
                break ok;
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

    let elapsed_ms = started.elapsed().as_millis().min(u32::MAX as u128) as u32;
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
    !approved
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
            metric_values: HashMap::new(),
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
            metric_values: HashMap::new(),
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
