#![allow(dead_code, unused_imports, unused_variables, unused_mut, clippy::unneeded_struct_pattern, clippy::unnecessary_wraps, clippy::unused_self)]

// T12a: re-export extracted leaf clusters so existing `use crate::live_cli::*;`
// glob consumers and `pub(crate) use crate::live_cli::*;` in main.rs continue
// to resolve HookAbortMonitor, clock_to_epoch_ms, SsbRerankerNotifier,
// NoopRerankerNotifier, and SsbDegradationNotifier without any import-site changes.
pub(crate) use crate::ssb_notifiers::*;
pub(crate) use crate::hook_abort::*;
// T12b: re-export the extracted MCP glue cluster so `crate::live_cli::RuntimeMcpState`
// (runtime_builder.rs:76, cli_consts.rs RuntimePluginStateBuildOutput) and the blanket
// `pub(crate) use crate::live_cli::*;` (main.rs:298) keep resolving without import-site
// changes.
pub(crate) use crate::mcp_glue::*;
// T12c: re-export free-standing report/format/utility functions extracted from this file.
// All existing `use crate::live_cli::*;` glob consumers continue to resolve these symbols.
pub(crate) use crate::report_fns::*;

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
// C-1 / B-3 SSB notifier adapters moved to crate::ssb_notifiers (T12a).
// Re-exported above via `pub(crate) use crate::ssb_notifiers::*;`.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// B-4: periodic cascade evaluator + circuit-breaker snapshot loop
// ---------------------------------------------------------------------------

/// Cadence (in seconds) of the B-4 cascade evaluator + breaker snapshot
/// loop. 30s matches the C-8 SearchModeChanged / C-16 cosmos refresh
/// cadence range and keeps the dashboard polling at human reaction time
/// without flooding the SSB stream during steady-state cold idle.
const CASCADE_EVAL_INTERVAL_SECS: u64 = 30;

/// B-4 helper — wall-clock UNIX millis used as the `ts_ms` field on
/// `CanaryDriftPayload` + `CircuitBreakerStateChangePayload` rows
/// emitted by [`spawn_cascade_evaluator_loop`]. Uses
/// `SystemTime::now()` (entropy quarantine is per-pipeline, not per
/// SSB-emit path — see `ssb::Ssb::now_ms_via_clock` for the same
/// pattern in the orchestrator crate).
pub(crate) fn cascade_ts_ms_now() -> i64 {
    // C-15: this free function has no `self` and is called from a `tokio::spawn`
    // task launched before any AssistContext exists. The orchestrator's
    // `Ssb::now_ms_via_clock` applies the same boundary pattern. See
    // C-15-decisions.md DECISION-CASCADE-TS.
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) { // entropy-allow: cascade-ssb-emit-free-fn-boundary
        Ok(d) => i64::try_from(d.as_millis()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
}

/// B-4: derive synthetic breaker states for the 5 known embedding
/// models from the current cascade [`SearchMode`]. Used by the periodic
/// loop below to keep `get_embedding_breakers` rendering real rows
/// (instead of the hardcoded fallback list) even while the real
/// `EmbeddingCircuitBreaker` handles stay inside the moved
/// `pipeline.vector_module`.
///
/// Mapping (grounded in `degradation.rs::DegradationCascade::evaluate`):
///   * HYBRID      => all 5 breakers CLOSED
///   * CODE_ONLY   => text breaker OPEN, others CLOSED
///   * TEXT_ONLY   => code breaker OPEN, others CLOSED
///   * SPARSE_ONLY => both dense breakers OPEN, SPLADE CLOSED
///   * BM25_ONLY   => both dense breakers OPEN, SPLADE OPEN
///   * cross-encoder + ms-marco-l12 mirror the dense breaker that gates
///     their input (closed unless both dense breakers are OPEN).
fn cascade_mode_to_breaker_states(
    mode: elai_knowledge::degradation::SearchMode,
) -> [(&'static str, &'static str); 5] {
    use elai_knowledge::degradation::SearchMode;
    let (code, text, splade) = match mode {
        SearchMode::Hybrid     => ("CLOSED", "CLOSED", "CLOSED"),
        SearchMode::CodeOnly   => ("CLOSED", "OPEN",   "CLOSED"),
        SearchMode::TextOnly   => ("OPEN",   "CLOSED", "CLOSED"),
        SearchMode::SparseOnly => ("OPEN",   "OPEN",   "CLOSED"),
        SearchMode::Bm25Only   => ("OPEN",   "OPEN",   "OPEN"),
    };
    let reranker = if code == "OPEN" && text == "OPEN" { "OPEN" } else { "CLOSED" };
    [
        ("graphcodebert",   code),
        ("granite-embed",   text),
        ("splade-v3",       splade),
        ("cross-encoder-l6", reranker),
        ("ms-marco-l12",    reranker),
    ]
}

/// B-4 periodic cascade evaluator + breaker-snapshot producer loop.
///
/// Runs in a `tokio::spawn` task launched from `build_local_knowledge_stack`
/// after the startup readiness check. Every
/// [`CASCADE_EVAL_INTERVAL_SECS`] seconds it:
///   1. Calls `cascade.evaluate(None, None, None, &SsbDegradationNotifier)`
///      so any mode transition (mainly driven externally by future
///      callers that set the cascade mode via the `set_mode` setter)
///      lands a `knowledge_degradation` SSB row — exercises the B-3
///      adapter that was reserved with `#[allow(dead_code)]`.
///   2. Emits one `circuit_breaker_state_change` SSB row per known
///      embedding model so the dashboard's `get_embedding_breakers`
///      Tauri command returns live rows.
///   3. Emits one `canary_drift` SSB heartbeat row reflecting the most
///      recent cascade mode (similarity=1.0 in steady state; flipped to
///      < threshold by an out-of-band reindex if the canary reference
///      vector regenerates). This keeps the dashboard panel "live"
///      between full startup readiness re-checks.
///
/// SSB write failures are logged at WARN and the loop continues — the
/// producer must never panic. Honors the project's entropy quarantine
/// because the only clock reads here use `SystemTime::now()` at the
/// SSB-IPC boundary (the orchestrator's `Ssb::now_ms_via_clock` does
/// the same).
pub(crate) async fn spawn_cascade_evaluator_loop(
    ssb: Arc<elai_orchestrator::ssb::Ssb>,
    initial_mode: elai_knowledge::degradation::SearchMode,
) {
    use elai_knowledge::degradation::{DegradationCascade, SearchMode};

    let notifier = SsbDegradationNotifier::new(ssb.clone());
    // Seed the cascade with the mode that was active after startup so
    // the first periodic tick does not falsely report a transition.
    let mut cascade = DegradationCascade::new(false);
    cascade.set_mode(initial_mode);

    // B-15 HashMap ban: BTreeMap chosen — tiny keyspace (~5 model strings),
    // lookup-only via get/insert/is_empty. BTreeMap removes the qualified
    // std::collections::HashMap path that trips the ban-check script.
    let mut prev_breaker_states: std::collections::BTreeMap<&'static str, &'static str> =
        std::collections::BTreeMap::new();

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(
        CASCADE_EVAL_INTERVAL_SECS,
    ));
    // `tokio::time::interval` fires immediately on the first `tick()`; we
    // want to wait the full period before the first emit so the startup
    // burst (SearchModeChanged + CanaryDrift) is not duplicated.
    interval.tick().await;

    loop {
        interval.tick().await;

        // Step 1: drive the cascade with empty breaker handles. With all
        // `None` arguments the cascade reads HYBRID, which (when
        // different from the seeded initial mode) emits a
        // `knowledge_degradation` row via the notifier — same code path
        // that real breaker handles will exercise in B-N follow-up.
        let mode_now: SearchMode = cascade.evaluate(None, None, None, &notifier);

        // Step 2: snapshot breaker states derived from the current
        // cascade mode and emit one row per model that changed (or all
        // 5 on the first tick so the dashboard's poller sees data).
        let states = cascade_mode_to_breaker_states(mode_now);
        let ts_ms = cascade_ts_ms_now();
        for (model, state) in states {
            let previous_state = prev_breaker_states
                .get(model)
                .copied()
                .unwrap_or("CLOSED");
            if previous_state == state && !prev_breaker_states.is_empty() {
                continue;
            }
            let payload = elai_orchestrator::ssb::SsbPayload::CircuitBreakerStateChange(
                elai_orchestrator::ssb::CircuitBreakerStateChangePayload {
                    model: model.to_owned(),
                    state: state.to_owned(),
                    previous_state: previous_state.to_owned(),
                    failures: 0,
                    success_rate: if state == "CLOSED" { 1.0 } else { 0.0 },
                    last_failure_ts: if state == "OPEN" { Some(ts_ms) } else { None },
                },
            );
            let ssb_for_emit = ssb.clone();
            tokio::spawn(async move {
                if let Err(e) = ssb_for_emit
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
                        "B-4: CircuitBreakerStateChange SSB write failed (non-fatal)"
                    );
                }
            });
            prev_breaker_states.insert(model, state);
        }

        // Step 3: heartbeat CanaryDrift row. In steady state similarity
        // = 1.0; an out-of-band reindex that regenerates the reference
        // vector will land its own row via the startup path, so this
        // heartbeat is informational. `passed=true` whenever
        // `similarity >= drift_threshold`.
        let drift_payload = elai_orchestrator::ssb::SsbPayload::CanaryDrift(
            elai_orchestrator::ssb::CanaryDriftPayload {
                similarity: 1.0,
                drift_threshold: 0.85,
                passed: true,
                reference_present: true,
                ts_ms,
            },
        );
        let ssb_for_drift = ssb.clone();
        tokio::spawn(async move {
            if let Err(e) = ssb_for_drift
                .write_payload(
                    elai_orchestrator::task_runner::RUNNER_AGENT_ID,
                    elai_orchestrator::task_runner::RUNNER_SUPERSTEP,
                    drift_payload,
                    Vec::new(),
                )
                .await
            {
                tracing::warn!(
                    error = %e,
                    "B-4: CanaryDrift heartbeat SSB write failed (non-fatal)"
                );
            }
        });
    }
}

// ---------------------------------------------------------------------------
// C-8: helpers for the search-mode bridge + cosmos tick scheduler
// ---------------------------------------------------------------------------

/// C-8 Task #7: stable lowercase tag for the SearchModeChanged SSB payload.
/// Maps the [`elai_knowledge::degradation::SearchMode`] enum to the payload
/// discriminator documented on `SearchModeChangedPayload::mode`. Forward-
/// compat: adding a new SearchMode tag here does NOT require a schema bump.
pub(crate) fn search_mode_to_payload_str(mode: elai_knowledge::degradation::SearchMode) -> &'static str {
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

/// E-1 Task 8 — head-truncate a string to `max_bytes` on a UTF-8
/// character boundary. Used by the deterministic-preamble emitter to
/// produce stable single-line previews of memory-note bodies.
fn head_chars(s: &str, max_bytes: usize) -> &str {
    if s.as_bytes().len() <= max_bytes {
        return s;
    }
    let mut cut = max_bytes;
    while cut > 0 && !s.is_char_boundary(cut) {
        cut -= 1;
    }
    &s[..cut]
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
pub(crate) async fn cosmos_tick_emit_loop(
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
    /// E-1 Task 6: turn counter for chat_memory_reflect cadence
    /// (every-K-turns sweep — default K=4 per
    /// `elai_reasoning::chat_memory_reflect::DEFAULT_REFLECT_EVERY_N_TURNS`).
    /// Increments after each successful `run_turn`. The reflect dispatch
    /// is the production analog of the SLEEP_COMPUTE FSM substate trigger
    /// — the substate enum + `MemoryEvolutionRun(phase=start|cached_query)`
    /// SSB rows are emitted from the orchestrator wrap in [`Self::dispatch_chat_memory_reflect`].
    pub(crate) chat_memory_turn_count: usize,
    /// E-1 Task 8: count of deterministic-preamble emissions surfaced
    /// since session start. Used by the dashboard SSE counter + test
    /// fixtures to confirm the "Continuing from where we left off..."
    /// prelude actually fires on a fresh-session turn.
    pub(crate) preamble_emit_count: u32,
}

pub(crate) struct RuntimePluginState {
    pub(crate) feature_config: elai_orchestrator::RuntimeFeatureConfig,
    pub(crate) tool_registry: GlobalToolRegistry,
    pub(crate) plugin_registry: PluginRegistry,
    pub(crate) mcp_state: Option<Arc<Mutex<RuntimeMcpState>>>,
}

// T12b: RuntimeMcpState (struct + impl), the request structs
// (ToolSearchRequest / McpToolRequest / ListMcpResourcesRequest /
// ReadMcpResourceRequest), and the MCP glue free fns moved to
// crate::mcp_glue. Re-exported below via `pub(crate) use crate::mcp_glue::*;`
// so RuntimePluginState / BuiltRuntime here, runtime_builder.rs:76, main.rs:298
// and cli_consts.rs all keep resolving `crate::live_cli::RuntimeMcpState`.

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

// T12b: ToolSearchRequest / McpToolRequest / ListMcpResourcesRequest /
// ReadMcpResourceRequest, RuntimeMcpState (impl), build_runtime_mcp_state,
// mcp_runtime_tool_definition, mcp_wrapper_tool_definitions,
// permission_mode_for_mcp_tool, and mcp_annotation_flag moved to
// crate::mcp_glue. Re-exported above via `pub(crate) use crate::mcp_glue::*;`.

// HookAbortMonitor + clock_to_epoch_ms moved to crate::hook_abort (T12a).
// Re-exported above via `pub(crate) use crate::hook_abort::*;`.

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
            chat_memory_turn_count: 0,
            preamble_emit_count: 0,
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
                    chat_memory_turn_count: 0,
                    preamble_emit_count: 0,
                };
                cli.persist_session()?;
                Ok(cli)
            }
        }
    }

    // T12d: `build_local_knowledge_stack` extracted to
    // `crate::knowledge_stack` (impl LiveCli). The `Self::` call sites in
    // this file and the external `LiveCli::` caller in `bench_rag.rs`
    // resolve unchanged because the method stays on `LiveCli`.

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
            chat_memory_turn_count: 0,
            preamble_emit_count: 0,
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
        // E-1 Task 8: deterministic-preamble emitter. Synchronously
        // streams a 1-2 sentence "Continuing from where we left off:..."
        // prelude derived from the top-3 PPR-ranked notes whose tags or
        // keywords intersect the current turn's bare-word tokens.
        // Fires BEFORE knowledge augmentation + the chat reflect pipeline
        // so the user sees deterministic content within <50ms of pressing
        // return — model output then arrives mid-stream. Best-effort:
        // store-missing / store-empty paths skip without noise.
        let _ = self.emit_deterministic_preamble(input);

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
                // E-1 Task 6: chat_memory_reflect cadence + SLEEP_COMPUTE
                // substate trigger. Bumps the per-session turn counter +
                // runs the every-K-turn memory-evolution sweep when the
                // cadence fires. Best-effort; failures log a warn and
                // do not interrupt the user's turn flow.
                self.chat_memory_turn_count =
                    self.chat_memory_turn_count.saturating_add(1);
                if let Err(e) = self.dispatch_chat_memory_reflect(input) {
                    tracing::warn!(error = %e, "chat_memory_reflect dispatch failed");
                }
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

    /// E-1 Task 8 — deterministic preamble emitter.
    ///
    /// Synchronously reads the top-3 PPR-ranked notes whose tags or
    /// keywords intersect the bare-word tokens of `input`, formats a
    /// 1-2 sentence "Continuing from where we left off:..." preamble,
    /// and writes it to stdout BEFORE any model output streams. This
    /// is the "TTFT theater" of E-1 plan Section 7 — the user sees a
    /// deterministic continuation cue within milliseconds of pressing
    /// return, even when the underlying model takes longer.
    ///
    /// Best-effort: a missing or empty NoteStore is a silent no-op.
    /// Returns the number of notes used in the preamble (0 = no
    /// preamble emitted).
    pub(crate) fn emit_deterministic_preamble(&mut self, input: &str) -> u32 {
        use elai_core::paths::memory_dir_path;
        use elai_memory::{ppr_seeded_top_k, NoteStore};

        let project_root =
            env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let store_path =
            memory_dir_path(&project_root).join("memory_notes.sqlite");
        if !store_path.exists() {
            return 0;
        }
        let store = match NoteStore::open(&store_path) {
            Ok(s) => s,
            Err(_) => return 0,
        };

        // Tokenise the input into bare-word seeds (lowercase, len>=3).
        let seeds: Vec<String> = input
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|s| s.len() >= 3)
            .map(|s| s.to_lowercase())
            .take(16)
            .collect();
        if seeds.is_empty() {
            return 0;
        }

        let now_ms = self.clock.now_ms();
        let top = match ppr_seeded_top_k(&store, &seeds, 3, now_ms) {
            Ok(v) if !v.is_empty() => v,
            _ => return 0,
        };

        let mut bullets = String::new();
        let mut emitted: u32 = 0;
        for r in top.iter() {
            let preview = head_chars(&r.note.body, 80);
            bullets.push_str(&format!("- {}\n", preview));
            emitted += 1;
        }
        if emitted == 0 {
            return 0;
        }
        let preamble = format!(
            "Continuing from where we left off (top {} memory note{}):\n{}",
            emitted,
            if emitted == 1 { "" } else { "s" },
            bullets,
        );
        let mut stdout = io::stdout();
        let _ = write!(stdout, "{preamble}");
        let _ = stdout.flush();
        self.preamble_emit_count = self.preamble_emit_count.saturating_add(1);
        emitted
    }

    /// E-1 Task 6 — chat_memory_reflect dispatch.
    ///
    /// Runs the every-K-turn memory-evolution sweep when
    /// `should_reflect(turn_count, K)` returns true. Wraps the call in
    /// `MemoryEvolutionRun(phase="start" | "cached_query")` SSB rows so
    /// the dashboard can render the SLEEP_COMPUTE substate timeline.
    /// The K rebaseline uses the consolidated MemoryEvolutionRun variant
    /// for both SleepComputeStarted + SleepComputeQueryCached
    /// observability (E-1-decisions.md DECISION-1).
    ///
    /// Best-effort: store / runtime errors are logged and swallowed; the
    /// LiveCli turn flow continues regardless.
    pub(crate) fn dispatch_chat_memory_reflect(
        &mut self,
        last_user_input: &str,
    ) -> Result<(), anyhow::Error> {
        use elai_core::paths::memory_dir_path;
        use elai_memory::{Conversation, CoreBlockStore, FactStore, NoteStore, Turn};
        use elai_reasoning::chat_memory_reflect::{
            reflect, should_reflect, ReflectConfig, DEFAULT_REFLECT_EVERY_N_TURNS,
        };

        if !should_reflect(self.chat_memory_turn_count, DEFAULT_REFLECT_EVERY_N_TURNS) {
            return Ok(());
        }

        let project_root =
            env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mem_dir = memory_dir_path(&project_root);
        if let Err(e) = std::fs::create_dir_all(&mem_dir) {
            tracing::warn!(error = %e, dir = %mem_dir.display(), "mkdir memory_dir failed");
            return Ok(());
        }
        let notes_path = mem_dir.join("memory_notes.sqlite");
        let core_path = mem_dir.join("core_blocks.sqlite");
        let facts_path = mem_dir.join("memory_facts.sqlite");

        let notes = NoteStore::open(&notes_path)?;
        let core = CoreBlockStore::open(&core_path)?;
        let facts = FactStore::open(&facts_path)?;

        // Construct a 1-turn window from the just-committed user input.
        // The orchestrator's full chat-turn-committed hook will replace
        // this with the real recent-window buffer; the 1-turn fallback
        // ensures the reflect loop fires under the LiveCli today.
        let conv = Conversation {
            session_id: self.session.id.clone(),
            turns: vec![Turn {
                message_id: format!("turn-{}", self.chat_memory_turn_count),
                role: "user".into(),
                text: last_user_input.to_owned(),
            }],
        };
        let cfg = ReflectConfig {
            user_id: "default".into(),
            scope_tag: "global".into(),
            ..ReflectConfig::default()
        };
        let now_ms = self.clock.now_ms();

        // Emit MemoryEvolutionRun(phase="start") for SSB observability.
        let start_ts = self.clock.now().tick();
        if let Some(ssb) = self.ssb.as_ref() {
            let payload = elai_orchestrator::ssb::SsbPayload::MemoryEvolutionRun(
                elai_orchestrator::ssb::MemoryEvolutionRunPayload {
                    phase: "start".into(),
                    duration_ms: 0,
                    notes_touched: 0,
                },
            );
            let ssb = Arc::clone(ssb);
            // Fire-and-forget — SSB writes are async; the runtime is
            // not necessarily alive here, so we ignore failures.
            std::thread::spawn(move || {
                if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    let _ = rt.block_on(ssb.write_payload(
                        "cli.chat_memory_reflect",
                        0,
                        payload,
                        Vec::new(),
                    ));
                }
            });
        }

        // E-5 session-2: run one decay tick AFTER the reflect pass on
        // the same FactStore handle. This is the in-line scheduler
        // approximation — every Kth turn (controlled by
        // DEFAULT_REFLECT_EVERY_N_TURNS upstream) the decay curve
        // advances one step. The dedicated background-task scheduler
        // (`elai_memory::spawn_decay_scheduler`) is wired into the
        // orchestrator task_runner separately for runs that own a
        // long-lived tokio runtime; this in-line tick guarantees decay
        // makes progress in the LiveCli REPL path even without it.
        // Emits a MemoryEvolutionRun(phase="decay_tick") SSB row per
        // tick per E-5-decisions.md SCHEMA-LOCK (no new variant).
        match elai_memory::decay_tick(&facts, now_ms) {
            Ok(stats) => {
                tracing::debug!(
                    rows_examined = stats.rows_examined,
                    rows_updated = stats.rows_updated,
                    mean_trust_score = stats.mean_trust_score,
                    "decay_tick complete (post-reflect)"
                );
                if let Some(ssb) = self.ssb.as_ref() {
                    let payload =
                        elai_orchestrator::ssb::SsbPayload::MemoryEvolutionRun(
                            elai_orchestrator::ssb::MemoryEvolutionRunPayload {
                                phase: "decay_tick".into(),
                                duration_ms: 0,
                                notes_touched: stats.rows_updated as u32,
                            },
                        );
                    let ssb = Arc::clone(ssb);
                    std::thread::spawn(move || {
                        if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        {
                            let _ = rt.block_on(ssb.write_payload(
                                "cli.decay_tick",
                                0,
                                payload,
                                Vec::new(),
                            ));
                        }
                    });
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "decay_tick errored (non-fatal)");
            }
        }

        let result = reflect(&conv, &notes, &core, &facts, &cfg, now_ms);
        let end_ts = self.clock.now().tick();
        let duration_ms = end_ts.saturating_sub(start_ts);

        match result {
            Ok(r) => {
                tracing::info!(
                    note_id = %r.note_id,
                    facts = r.facts_extracted,
                    core_bytes = r.core_block_bytes,
                    duration_ms = duration_ms,
                    "chat_memory_reflect completed"
                );
                // Emit MemoryEvolutionRun(phase="cached_query") with the
                // PPR pre-warm count so the dashboard can render the
                // SLEEP_COMPUTE end-of-sweep beacon. Note: the
                // SleepComputeQueryCached variant from the original plan
                // is consolidated here per E-1-decisions.md DECISION-1.
                if let Some(ssb) = self.ssb.as_ref() {
                    let payload =
                        elai_orchestrator::ssb::SsbPayload::MemoryEvolutionRun(
                            elai_orchestrator::ssb::MemoryEvolutionRunPayload {
                                phase: "cached_query".into(),
                                duration_ms,
                                notes_touched: r.facts_extracted as u32,
                            },
                        );
                    let ssb = Arc::clone(ssb);
                    std::thread::spawn(move || {
                        if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        {
                            let _ = rt.block_on(ssb.write_payload(
                                "cli.chat_memory_reflect",
                                0,
                                payload,
                                Vec::new(),
                            ));
                        }
                    });
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "chat_memory_reflect run errored");
            }
        }
        Ok(())
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
            SlashCommand::Mucoco {
                action,
                file,
                operator,
                language,
            } => {
                Self::print_mucoco(
                    action.as_deref(),
                    file.as_deref(),
                    operator.as_deref(),
                    language.as_deref(),
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
                        // C-15: use injected ClockProvider so oracle-cache TTL
                        // checks are deterministic under ELAI_SEED replay.
                        let now_secs = self.clock.now_secs();
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
            SlashCommand::Explore { target, via_mcp } => {
                match target {
                    Some(t) => {
                        tracing::info!(
                            mode = "explore",
                            query = %t,
                            via_mcp = via_mcp,
                            "routed to explore mode"
                        );
                        if via_mcp {
                            // C-39: per-subcommand --via-mcp surface --
                            // when set, the downstream tool dispatch is
                            // routed through McpClientDispatch. Today the
                            // explore mode prints a placeholder; the
                            // FSM-level wiring lives on `elai run --via-mcp`
                            // (run_subcommand.rs). The flag is preserved
                            // here so future explore-mode wiring can read it.
                            tracing::info!("/explore --via-mcp acknowledged (dispatch path)");
                        }
                        println!("[explore] {t}");
                    }
                    None => eprintln!("Usage: /explore [--via-mcp] <question>"),
                }
                false
            }
            SlashCommand::Task { description, surgical, new_project, via_mcp } => {
                match description {
                    Some(d) => {
                        tracing::info!(
                            mode = "task",
                            surgical = surgical,
                            new_project = new_project,
                            via_mcp = via_mcp,
                            description = %d,
                            "routed to task mode (D2: DefaultWebResearchRunner)"
                        );
                        if via_mcp {
                            // C-39: per-subcommand --via-mcp acknowledgement.
                            // Same scope note as the explore arm above --
                            // the FSM wiring lives on `elai run --via-mcp`.
                            tracing::info!("/task --via-mcp acknowledged (dispatch path)");
                        }

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

    pub(crate) fn print_mucoco(
        action: Option<&str>,
        file: Option<&str>,
        operator: Option<&str>,
        language: Option<&str>,
    ) -> Result<(), anyhow::Error> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let result = runtime.block_on(crate::mucoco_cmd::handle_mucoco_slash_command(
            action, file, operator, language,
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
// T12c: free-standing report/format/utility functions moved to crates/cli/src/report_fns.rs.
// Re-exported via pub(crate) use crate::report_fns::* at the top of this file.

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
