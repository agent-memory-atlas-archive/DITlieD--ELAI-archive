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
}
