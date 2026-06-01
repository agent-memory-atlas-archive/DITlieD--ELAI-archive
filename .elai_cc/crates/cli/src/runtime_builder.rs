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

use elai_core::{ClockProvider, EntropySource, OsEntropy};
use elai_providers::{
    oauth_token_is_expired, resolve_startup_auth_source, AnthropicClient, AuthSource,
    ContentBlockDelta, InputContentBlock, InputMessage, MessageRequest, MessageResponse,
    OutputContentBlock, PromptCache,
    StreamEvent as ApiStreamEvent, ToolChoice, ToolDefinition, ToolResultContentBlock,
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
use crate::live_cli::{RuntimeMcpState, RuntimePluginState};

pub(crate) fn build_runtime(
    session: Session,
    session_id: &str,
    model: String,
    system_prompt: Vec<String>,
    enable_tools: bool,
    emit_output: bool,
    allowed_tools: Option<AllowedToolSet>,
    permission_mode: PermissionMode,
    progress_reporter: Option<InternalPromptProgressReporter>,
    backend_config: Option<&BackendConfig>,
    fsm: Option<Arc<elai_orchestrator::Fsm>>,
    phase_slot_tokens: usize,
) -> Result<(BuiltRuntime, Option<Arc<elai_orchestrator::Ssb>>), anyhow::Error> {
    let runtime_plugin_state = build_runtime_plugin_state()?;
    build_runtime_with_plugin_state(
        session,
        session_id,
        model,
        system_prompt,
        enable_tools,
        emit_output,
        allowed_tools,
        permission_mode,
        progress_reporter,
        runtime_plugin_state,
        backend_config,
        fsm,
        phase_slot_tokens,
    )
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_runtime_with_plugin_state(
    session: Session,
    session_id: &str,
    model: String,
    system_prompt: Vec<String>,
    enable_tools: bool,
    emit_output: bool,
    allowed_tools: Option<AllowedToolSet>,
    permission_mode: PermissionMode,
    progress_reporter: Option<InternalPromptProgressReporter>,
    runtime_plugin_state: RuntimePluginState,
    backend_config: Option<&BackendConfig>,
    fsm: Option<Arc<elai_orchestrator::Fsm>>,
    phase_slot_tokens: usize,
) -> Result<(BuiltRuntime, Option<Arc<elai_orchestrator::Ssb>>), anyhow::Error> {
    let RuntimePluginState {
        feature_config,
        tool_registry,
        plugin_registry,
        mcp_state,
    } = runtime_plugin_state;
    plugin_registry.initialize()?;
    let policy = permission_policy(permission_mode, &feature_config, &tool_registry)
        .map_err(std::io::Error::other)?;

    let (backend, auth, base_url, tool_policy, budget_policy) = match backend_config {
        Some(config) => {
            let fsm_state_path = session
                .persistence_path()
                .map(|p| p.parent().unwrap_or(p).join(".fsm-state.msgpack"));
            (
                BackendMode::Local,
                AuthSource::None,
                config.sidecar_url.clone(),
                ToolPolicy::FsmFiltered {
                    max_tools: config.max_tools_local,
                    fsm_state_path,
                },
                BudgetPolicy::Local {
                    ceiling: elai_orchestrator::LOCAL_MODEL_MAX_CONTEXT,
                    fraction: elai_orchestrator::LOCAL_BUDGET_FRACTION,
                },
            )
        }
        None => (
            BackendMode::Api,
            resolve_cli_auth_source()?,
            elai_providers::read_base_url(),
            if enable_tools { ToolPolicy::Full } else { ToolPolicy::None },
            BudgetPolicy::ApiDefault,
        ),
    };

    let mut client_builder = AnthropicClient::from_auth(auth).with_base_url(base_url);
    if matches!(backend, BackendMode::Api) {
        client_builder = client_builder.with_prompt_cache(PromptCache::new(session_id));
    }

    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("elai-worker")
        .enable_all()
        .build()?;

    // Create Ssb inside tokio runtime context (Ssb::new calls tokio::spawn).
    let ssb = {
        let _guard = tokio_runtime.enter();
        let ssb_db_path = session
            .persistence_path()
            .map(|p| {
                p.parent()
                    .unwrap_or(p)
                    .join(elai_orchestrator::SSB_DB_FILE_NAME)
            })
            .unwrap_or_else(|| {
                let mut tmp = std::env::temp_dir();
                tmp.push(format!("elai-ssb-{}.db", session_id));
                tmp
            });
        // Canonical CWD-derived project_id. Shared helper (elai_core) so
        // the dashboard lands rows in the same bucket when launched from
        // the same directory. See `elai_core::project_id` module docs.
        let project_id = elai_core::project_id_from_cwd()
            .unwrap_or_else(|| format!("{}unknown", elai_core::PROJECT_ID_PREFIX));
        let ssb_config = elai_orchestrator::SsbConfig {
            db_path: ssb_db_path,
            pool_size: None,
            project_id,
            session_id: session_id.to_string(),
        };
        let clock: Arc<dyn ClockProvider> = Arc::new(elai_core::OsClock);
        match elai_orchestrator::Ssb::new(ssb_config, clock) {
            Ok(ssb) => Some(Arc::new(ssb)),
            Err(e) => {
                eprintln!("[warn] SSB init failed, telemetry disabled: {e}");
                None
            }
        }
    };

    let dkg_querier: Box<dyn elai_rules::predicate::DkgQuerier + Send + Sync> = {
        // B-1: phase-3J L141-146 documents the legacy expression
        // `session.persistence_path().parent()/dkg.db` resolving to
        // `.elai/sessions/dkg.db` — a per-session binding that was
        // never populated and silently fell through to EmptyDkgQuerier.
        // We deliberately sever the (buggy) per-session affordance and
        // bind the rule-engine DKG to the project-scoped canonical
        // database produced by IndexingService.
        let project_root_for_dkg =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let dkg_db_path = core_dkg_db_path(&project_root_for_dkg);
        if dkg_db_path.exists() {
            match DkgQuerierAdapter::new(&dkg_db_path) {
                Ok(adapter) => Box::new(adapter),
                Err(e) => {
                    eprintln!("[warn] DKG adapter init failed, using empty querier: {e}");
                    Box::new(EmptyDkgQuerier)
                }
            }
        } else {
            // DKG db not yet built (first run or no graph phase complete). Silent fallback.
            Box::new(EmptyDkgQuerier)
        }
    };
    let rule_engine = DefaultRuleEngine::new(
        Box::new(TreeSitterAstEvaluator::new()),
        dkg_querier,
    );
    let compiled_rules = load_seed_rules(&rule_engine);

    // PI Tier 2 + Rule Seeding: detect project profile, enrich with Tier 2
    // conventions, then seed project-specific rules into the rules DB.
    {
        let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        match detect_project_profile(&project_root) {
            Ok(mut profile) => {
                // Tier 2: DkgEngine not available at this point; regex fallback used.
                elai_knowledge::populate_tier2_conventions(&mut profile, None);
                tracing::info!(
                    project = %project_root.display(),
                    lang = %profile.languages.primary.id_str(),
                    "PI Tier 2 conventions populated"
                );

                // B-1: rules.db is project-scoped at
                // `<root>/.elai_cc/rules.db` (was `.elai/rules.db` in
                // the unbuilt fallback branch). The session-persistence
                // path was a per-session binding that mirrored the
                // rule-engine DKG bug; we collapse to a single
                // project-scoped database.
                let rules_db_path = core_rules_db_path(&project_root);
                let clock = elai_core::OsClock;
                match elai_rules::schema::open_rules_db(&rules_db_path, &clock) {
                    Ok(conn) => {
                        match elai_rules::project_rule_seeder::ProjectRuleSeeder::seed_rules(
                            &conn, &profile, &clock,
                        ) {
                            Ok(n) => tracing::info!(count = n, "PI rule seeding complete"),
                            Err(e) => tracing::warn!(error = %e, "PI rule seeding failed"),
                        }
                    }
                    Err(e) => tracing::warn!(
                        path = %rules_db_path.display(),
                        error = %e,
                        "rules.db open failed; PI rule seeding skipped"
                    ),
                }
            }
            Err(e) => tracing::warn!(error = %e, "project profile detection failed; PI Tier 2 + seeding skipped"),
        }
    }

    let backend_config_owned = backend_config.cloned();

    let runtime_client = RuntimeClient {
        runtime: tokio_runtime,
        client: client_builder,
        backend,
        model: model.clone(),
        enable_tools,
        emit_output,
        allowed_tools: allowed_tools.clone(),
        tool_registry: tool_registry.clone(),
        progress_reporter,
        tool_policy,
        budget_policy,
        fsm,
        rule_engine,
        compiled_rules,
        phase_slot_tokens,
        backend_config: backend_config_owned,
        // B-28 WIRING-2 (item G): focus-tunnel StreamingValidator is opt-in;
        // None preserves the legacy fast path. Production wire-up lands
        // when `drive_task` plumbs the validator through.
        streaming_validator: None,
    };

    // Load elai_cc.toml from the working directory. Defaults are used when absent.
    let elai_config = env::current_dir()
        .map(|cwd| runtime_config::load_elai_config(&cwd))
        .unwrap_or_default();

    // A-26/A-27: init routing and safety filter from elai_cc.toml (OnceLock — first call wins).
    elai_tools::routing_rules::init_routing_config(elai_config.tools.routing_rules.clone());
    elai_tools::safety_filter::init_safety_config(elai_config.tools.safety_filter.enabled);

    // A-31: apply action policy from elai_cc.toml [actions] to the tool registry.
    let tool_registry_with_policy = tool_registry
        .clone()
        .with_action_policy(elai_config.actions.clone());
    let mut runtime = ConversationRuntime::new_with_features(
        session,
        runtime_client,
        CliToolExecutor::new(
            allowed_tools.clone(),
            emit_output,
            tool_registry_with_policy,
            mcp_state.clone(),
        ),
        policy,
        system_prompt,
        &feature_config,
    );
    // A-30: wire OutputConfig from elai_cc.toml [output] section.
    runtime = runtime.with_output_config(elai_config.output);
    if emit_output {
        runtime = runtime.with_hook_progress_reporter(Box::new(CliHookProgressReporter));
    }
    // E-25 Tranche L (F.1.c): install ApprovalFatigueGuard substrate.
    // Field-Some is opt-in here; the guard only fires when the env gate
    // `ELAI_E25_APPROVAL_FATIGUE_GUARD_ENABLE` is also truthy. Default
    // OFF (env-gate dormant), so this builder call is wire-dark.
    runtime = runtime.with_approval_fatigue(
        elai_orchestrator::architecture::approval_fatigue_guard::ApprovalFatigueGuard::with_defaults(),
    );
    // E-25 stubs F1c / F1a: wire SSB writer so guard emits land in the
    // same DB the dashboard reads. Gated on ssb being Some -- when SSB
    // init failed the emitter is simply absent (existing behavior for
    // telemetry-disabled paths). Both guard env-gates (APPROVAL_FATIGUE
    // and BASH_COMPLEXITY) are checked at the emit site in run_turn;
    // no additional gate here.
    if let Some(ssb_arc) = ssb.as_ref() {
        runtime = runtime.with_ssb_writer(Arc::clone(ssb_arc));
    }
    // Wire SessionTracer with file-based telemetry sink (interim; full SSB sink in later step).
    if let Some(persist_path) = runtime.session().persistence_path() {
        let telemetry_path = persist_path
            .parent()
            .unwrap_or(persist_path)
            .join("telemetry.jsonl");
        match elai_orchestrator::JsonlTelemetrySink::new(&telemetry_path) {
            Ok(sink) => {
                let tracer = elai_orchestrator::SessionTracer::new(
                    session_id,
                    Arc::new(sink),
                );
                runtime = runtime.with_session_tracer(tracer);
            }
            Err(e) => eprintln!("[warn] session telemetry disabled: {e}"),
        }
    }
    Ok((BuiltRuntime::new(runtime, plugin_registry, mcp_state), ssb))
}

pub(crate) struct CliHookProgressReporter;

impl elai_orchestrator::HookProgressReporter for CliHookProgressReporter {
    fn on_event(&mut self, event: &elai_orchestrator::HookProgressEvent) {
        match event {
            elai_orchestrator::HookProgressEvent::Started {
                event,
                tool_name,
                command,
            } => eprintln!(
                "[hook {event_name}] {tool_name}: {command}",
                event_name = event.as_str()
            ),
            elai_orchestrator::HookProgressEvent::Completed {
                event,
                tool_name,
                command,
            } => eprintln!(
                "[hook done {event_name}] {tool_name}: {command}",
                event_name = event.as_str()
            ),
            elai_orchestrator::HookProgressEvent::Cancelled {
                event,
                tool_name,
                command,
            } => eprintln!(
                "[hook cancelled {event_name}] {tool_name}: {command}",
                event_name = event.as_str()
            ),
        }
    }
}

pub(crate) struct CliInteractiveGate;

impl elai_orchestrator::InteractiveGate for CliInteractiveGate {
    fn present_gate(
        &self,
        summary: &elai_orchestrator::EvidenceSummary,
        risk_level: elai_orchestrator::RiskLevel,
    ) -> elai_orchestrator::GateDecision {
        if risk_level == elai_orchestrator::RiskLevel::Low {
            return elai_orchestrator::GateDecision::Approve;
        }
        eprintln!(
            "[gate] risk={risk_level} phase={} gates={}",
            summary.phase_label, summary.gate_results.len()
        );
        for result in &summary.gate_results {
            let mark = if result.passed { "PASS" } else { "FAIL" };
            eprintln!("  [{mark}] {}: {}", result.gate_name, result.detail);
        }
        eprint!("[gate] approve / reject? > ");
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return elai_orchestrator::GateDecision::Reject;
        }
        match input.trim().to_lowercase().as_str() {
            "approve" | "a" | "yes" | "y" => elai_orchestrator::GateDecision::Approve,
            _ => elai_orchestrator::GateDecision::Reject,
        }
    }
}

pub(crate) struct CliPermissionPrompter {
    pub(crate) current_mode: PermissionMode,
}

impl CliPermissionPrompter {
    pub(crate) fn new(current_mode: PermissionMode) -> Self {
        Self { current_mode }
    }
}

impl elai_orchestrator::PermissionPrompter for CliPermissionPrompter {
    fn decide(
        &mut self,
        request: &elai_orchestrator::PermissionRequest,
    ) -> elai_orchestrator::PermissionPromptDecision {
        println!();
        println!("Permission approval required");
        println!("  Tool             {}", request.tool_name);
        println!("  Current mode     {}", self.current_mode.as_str());
        println!("  Required mode    {}", request.required_mode.as_str());
        if let Some(reason) = &request.reason {
            println!("  Reason           {reason}");
        }
        println!("  Input            {}", request.input);
        print!("Approve this tool call? [y/N]: ");
        let _ = io::stdout().flush();

        let mut response = String::new();
        match io::stdin().read_line(&mut response) {
            Ok(_) => {
                let normalized = response.trim().to_ascii_lowercase();
                if matches!(normalized.as_str(), "y" | "yes") {
                    elai_orchestrator::PermissionPromptDecision::Allow
                } else {
                    elai_orchestrator::PermissionPromptDecision::Deny {
                        reason: format!(
                            "tool '{}' denied by user approval prompt",
                            request.tool_name
                        ),
                    }
                }
            }
            Err(error) => elai_orchestrator::PermissionPromptDecision::Deny {
                reason: format!("permission approval failed: {error}"),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// RuntimeClient — unified API client with policy-based backend selection.
// Replaces AnthropicRuntimeClient, LocalRuntimeClient, and CliApiClient.
// Branch points in stream(): auth (constructor), tool filtering, prompt cache.
// ---------------------------------------------------------------------------

pub(crate) enum ToolPolicy {
    Full,
    FsmFiltered {
        max_tools: usize,
        fsm_state_path: Option<PathBuf>,
    },
    None,
}

pub(crate) enum BudgetPolicy {
    ApiDefault,
    Local {
        ceiling: usize,
        fraction: usize,
    },
    /// B-27 cloud-mode 200K profile. `ceiling` is the attention window
    /// (200_000 for Anthropic Sonnet 4.6 with the
    /// [`elai_orchestrator::context_budget::ModelCapabilityProfile::cloud_200k`]
    /// profile). The runtime client uses this verbatim instead of the
    /// model-keyed `max_tokens_for_model` table. Type matches the
    /// `MessageRequest.max_tokens` width so the match arm doesn't
    /// require a narrowing cast.
    Cloud {
        ceiling: u32,
    },
}

// ---------------------------------------------------------------------------
// Phase A1b — production wiring chokepoint moved to the orchestrator crate.
//
// The helpers below were authored here in Phase A1a and have now been
// promoted to `elai_orchestrator::runtime_setup` so the dashboard-v2 Tauri
// `setup()` callback can spawn `start_task_runner` with the same Some+Some
// wiring without re-implementing the bundle. The previous local definitions
// also nested the SSB under `sessions/<session_id>/ssb.db`, which broke the
// dashboard reader (WARN-1 in workflow/handoff/notes-A1a.md). The moved
// versions write the canonical flat path
// `<project_root>/.elai_cc/sessions/ssb.db`.
//
// Re-exported here under their old `pub(crate)` names so `run_subcommand.rs`
// keeps compiling without churn — every other call site reaches them through
// `elai_orchestrator::{...}` directly.
//
// Contract: `crates/contracts/tests/task_runner_chokepoint_contract.rs`.
// ---------------------------------------------------------------------------

pub(crate) use elai_orchestrator::{
    apply_task_runner_wiring, build_task_runner_wiring, RuntimeWiring,
};

// ---------------------------------------------------------------------------
// patch-1.14 — shared production agent pool installer.
//
// extracted from `run_subcommand.rs` so both `elai run` and `elai task` can
// install the same AgentPoolBabyStepExecutor wiring without copy-paste drift.
// without this helper the `elai task` subcommand fell through to
// SyntheticBabyStepExecutor for any non-baby-step orchestrator dispatch and
// the v5 swe-bench-pro driver could not exercise the real agent loop.
//
// the helper mutates `wiring.baby_step_executor` in place. on any soft
// failure (ipc dir create fails, agent binary missing, cloud creds absent in
// the cloud arm) it logs a tracing::warn! and returns Ok(()) so the caller
// keeps running with the synthetic fallback exactly as it did before. only
// hard failures (anyhow ? propagation) abort the wiring step.
// ---------------------------------------------------------------------------

/// install the production AgentPoolBabyStepExecutor into `wiring`. soft-fails
/// to synthetic when the agent binary or ipc dir is unavailable; returns Err
/// only on unexpected wiring failures the caller cannot recover from.
pub(crate) async fn install_production_agent_pool(
    wiring: &mut RuntimeWiring,
    project_root: &Path,
    session_id: &str,
) -> anyhow::Result<()> {
    let ipc_dir = project_root
        .join(elai_core::paths::ELAI_DATA_DIR_NAME)
        .join("sessions")
        .join(session_id)
        .join("ipc");
    if let Err(e) = std::fs::create_dir_all(&ipc_dir) {
        tracing::warn!(
            path = %ipc_dir.display(),
            error = %e,
            "agent pool: failed to create IPC socket dir; executor will fall back to synthetic"
        );
        return Ok(());
    }
    let agent_bin = match elai_orchestrator::locate_default_agent_binary() {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "agent pool: agent binary not found; using synthetic fallback executor"
            );
            return Ok(());
        }
    };
    let gateway = Arc::new(tokio::sync::Mutex::new(
        elai_orchestrator::ipc_gateway::IpcGateway::with_os_clock(
            ipc_dir.clone(),
            elai_orchestrator::sandbox::SelfModificationLocks::new(),
        ),
    ));
    let rpc_server: Arc<dyn elai_orchestrator::AgentRpcServer + Send + Sync + 'static> =
        if std::env::var("ELAI_GROUNDING_BACKEND").as_deref() == Ok("cloud") {
            match elai_providers::AnthropicClient::from_env() {
                Ok(client) => build_agent_pool_rpc_server(client, wiring.ssb.clone()),
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "agent pool: cloud credentials unavailable; using NoOpAgentRpcServer"
                    );
                    Arc::new(elai_orchestrator::NoOpAgentRpcServer)
                }
            }
        } else {
            let base_url = std::env::var("ANTHROPIC_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8100".to_string());
            let client =
                elai_providers::AnthropicClient::new("local-dummy").with_base_url(base_url);
            build_agent_pool_rpc_server(client, wiring.ssb.clone())
        };
    let executor = elai_orchestrator::build_default_baby_step_executor(
        OsEntropy,
        agent_bin.clone(),
        ipc_dir.clone(),
        gateway,
        rpc_server,
    );
    wiring.baby_step_executor = Some(executor);
    tracing::info!(
        agent_bin = %agent_bin.display(),
        "agent pool: production AgentPoolBabyStepExecutor installed"
    );
    Ok(())
}

/// build the ProductionAgentRpcServer + InProcessToolDispatcher chain shared
/// by the cloud and sidecar arms of `install_production_agent_pool`. mirrors
/// the original inline block from run_subcommand.rs byte-for-byte (wraps
/// InProcessToolDispatcher in IpcToolDispatcherAdapter per C-13 D1 Option B,
/// installs the canonical DEFAULT_BABY_STEP_SYSTEM_PROMPT per patch-R5).
fn build_agent_pool_rpc_server(
    client: elai_providers::AnthropicClient,
    ssb: Arc<elai_orchestrator::Ssb>,
) -> Arc<dyn elai_orchestrator::AgentRpcServer + Send + Sync + 'static> {
    let model =
        std::env::var("ELAI_MODEL").unwrap_or_else(|_| "claude-sonnet-4-6".to_string());
    let max_tokens = std::env::var("ELAI_MAX_TOKENS")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(4096);
    let dispatcher: Arc<dyn elai_core::tool_dispatcher::ToolDispatcher> = {
        struct NoOpIntegrationRunner;
        impl elai_tools::tiered_validation::IntegrationCommandRunner for NoOpIntegrationRunner {
            fn run_integration<'a>(
                &'a self,
                _level: &'a str,
                _dir: &'a Path,
            ) -> Pin<
                Box<
                    dyn Future<
                            Output = elai_tools::tiered_validation::IntegrationRunOutcome,
                        > + Send
                        + 'a,
                >,
            > {
                Box::pin(async {
                    elai_tools::tiered_validation::IntegrationRunOutcome {
                        passed: false,
                        rules_fired: 0,
                        stderr_snippet: Some(
                            "integration tier not wired (NoOpIntegrationRunner)".to_string(),
                        ),
                    }
                })
            }
        }
        let validator = elai_tools::dispatcher::default_validator(
            Arc::new(NoOpIntegrationRunner),
            "rust",
        );
        let inner: Arc<dyn elai_core::tool_dispatcher::ToolDispatcher> = Arc::new(
            elai_tools::dispatcher::InProcessToolDispatcher::new(Arc::new(validator)),
        );
        Arc::new(elai_tools::adaptive_tool::IpcToolDispatcherAdapter::new(
            inner, ssb,
        ))
    };
    // s-22600 Tranche G G.3 (audit B.4-CLI followup): wire
    // `SymbolGraphCacheAdapter` at CLI startup so the
    // TOOL_SYMBOL_DSL_QUERY + TOOL_SYMBOL_DATAFLOW_IMPACT dispatcher
    // arms hit real cache data instead of returning typed empty
    // responses (pre-B.4 behaviour).
    //
    // Cache choice: the orchestrator's adapter wraps the JSON-rooted
    // `elai_orchestrator::symbol_graph::SymbolGraphCache` (E-40),
    // distinct from the SQLite-rooted
    // `elai_knowledge::symbol_graph::cache::SymbolGraphCache` (B-29).
    // The JSON view is built by `.claude/tools/codebase_mapper.py`
    // (per CLAUDE.md "Codebase map" pointer); when the file is absent
    // (fresh checkout / never regenerated) we skip the adapter wire and
    // pre-B.4 behaviour (typed-empty responses) is preserved. No new
    // env gate -- the on-disk JSON file is the gate.
    let symbol_graph_adapter: Option<
        std::sync::Arc<dyn elai_knowledge::symbol_graph_adapter::SymbolGraphAdapter>,
    > = {
        // Mirror `scip_cmd.rs` DEFAULT_GRAPH_PATH so the CLI wire and
        // the `elai scip *` subcommands point at the same file.
        let graph_path = std::path::PathBuf::from(".claude/tools/elai_graph.json");
        if !graph_path.exists() {
            tracing::debug!(
                graph = %graph_path.display(),
                "G.3 elai_graph.json absent; symbol-graph tool arms degrade to empty"
            );
            None
        } else {
            match elai_orchestrator::symbol_graph::SymbolGraphCache::load(&graph_path) {
                Ok(cache) => {
                    let cache_arc =
                        std::sync::Arc::new(std::sync::Mutex::new(cache));
                    // B-29.5 soft p99 latency budget per the
                    // `SymbolGraphCacheAdapter::new` docstring.
                    let max_query_ms: u64 = 500;
                    Some(std::sync::Arc::new(
                        elai_orchestrator::symbol_graph::SymbolGraphCacheAdapter::new(
                            cache_arc,
                            max_query_ms,
                        ),
                    ))
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        graph = %graph_path.display(),
                        "G.3 SymbolGraphCache::load failed; symbol-graph tool arms degrade to empty"
                    );
                    None
                }
            }
        }
    };
    let mut server = elai_orchestrator::ProductionAgentRpcServer::new(
        client, dispatcher, model, max_tokens,
    )
    .with_system_prompt(
        elai_orchestrator::production_agent_rpc::DEFAULT_BABY_STEP_SYSTEM_PROMPT,
    );
    if let Some(adapter) = symbol_graph_adapter {
        server = server.with_symbol_graph_adapter(adapter);
    }
    Arc::new(server)
}
