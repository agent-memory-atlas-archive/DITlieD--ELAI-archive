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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CliAction {
    DumpManifests {
        output_format: CliOutputFormat,
    },
    BootstrapPlan {
        output_format: CliOutputFormat,
    },
    Agents {
        args: Option<String>,
        output_format: CliOutputFormat,
    },
    Mcp {
        args: Option<String>,
        output_format: CliOutputFormat,
    },
    /// B-37D Phase 4M: direct (non-REPL) invocation of `/runtime`.
    /// Carries the three parser-string fields verbatim — validation
    /// lives in the runtime_cmd handler so the args layer stays free of
    /// any providers/orchestrator type dependency.
    Runtime {
        action: Option<String>,
        role: Option<String>,
        target: Option<String>,
        output_format: CliOutputFormat,
    },
    Skills {
        args: Option<String>,
        output_format: CliOutputFormat,
    },
    PrintSystemPrompt {
        cwd: PathBuf,
        date: String,
        output_format: CliOutputFormat,
    },
    Version {
        output_format: CliOutputFormat,
    },
    ResumeSession {
        session_path: PathBuf,
        commands: Vec<String>,
        output_format: CliOutputFormat,
    },
    Status {
        model: String,
        permission_mode: PermissionMode,
        output_format: CliOutputFormat,
    },
    Sandbox {
        output_format: CliOutputFormat,
    },
    Prompt {
        prompt: String,
        model: String,
        output_format: CliOutputFormat,
        allowed_tools: Option<AllowedToolSet>,
        permission_mode: PermissionMode,
        /// 1.1 — set when `--backend=local[:PORT]` is passed or
        /// ANTHROPIC_BASE_URL points to localhost. Routes through
        /// BudgetPolicy::Local instead of BudgetPolicy::ApiDefault.
        backend: Option<LocalBackendRef>,
    },
    Login {
        output_format: CliOutputFormat,
    },
    Logout {
        output_format: CliOutputFormat,
    },
    Doctor {
        output_format: CliOutputFormat,
    },
    /// E-17 SP1 task 1 — `elai doctor knowledge-models [--project-root PATH]
    /// [--output-format text|json|toml]`. Probes ONNX staging under
    /// `core::paths::models_dir_path(project_root)` and reports per-file
    /// existence, size, and blake3 hash. Nested as a sibling of
    /// `CliAction::Doctor` (not inside it) because the rest of the CLI
    /// dispatcher matches flat enum variants — mirrors `SymbolGraphBuild`
    /// vs `SymbolGraphStats` (args.rs:917-918).
    DoctorKnowledgeModels {
        project_root: PathBuf,
        output_format: CliOutputFormat,
    },
    Init {
        output_format: CliOutputFormat,
    },
    Repl {
        model: String,
        allowed_tools: Option<AllowedToolSet>,
        permission_mode: PermissionMode,
        /// B-9 cycle 2 (BLOCK-1 fix): when `false` (default), the REPL
        /// turn loop persists exclusively to the SessionDag. When `true`,
        /// it additionally writes the legacy JSONL transcript via
        /// [`Session::save_to_path`] for export tooling that still expects
        /// the on-disk JSONL renderer.
        export_jsonl: bool,
    },
    DepAudit {
        project_dir: PathBuf,
        output_format: CliOutputFormat,
    },
    Local {
        port: u16,
    },
    Dashboard,
    /// Phase A1a: `elai run [--project-root <path>]` — spawn the production
    /// task_runner with `feasibility_prober` AND `layer1a_processor` both
    /// wired to `Some(...)` so an SSB `TaskSubmitted` row reaches
    /// `drive_task` end-to-end. Closes the gap recorded in
    /// `metrics/phase_4F.toml:103`. Contract pinned by
    /// `crates/contracts/tests/task_runner_chokepoint_contract.rs`.
    Run {
        project_root: PathBuf,
        /// C-9 (Phase 4G) — `--mode autonomous|interactive|hybrid`. Resolved
        /// to a [`GateMode`] by [`elai_orchestrator::GateMode::from_cli_args`]
        /// at the production wiring point (`apply_task_runner_wiring`).
        /// `None` => default Interactive.
        mode: Option<String>,
        /// C-5 (cloud-mode hardening) — `--provider local|cloud|hybrid`.
        /// Distinct semantic axis from `--mode` (which is gate-mode, i.e.
        /// who decides to advance). `--provider` controls WHERE the work
        /// runs: pure local sidecar, pure cloud API, or hybrid (local-first
        /// with cloud-escalate). Maps to
        /// [`elai_orchestrator::dispatcher::BackendMode`] at the wiring
        /// site: `local` -> `Local`, `cloud` -> `Api`, `hybrid` -> `Hybrid`.
        /// `None` => use the toml default from `[cloud].default_mode`.
        provider: Option<String>,
        /// C-39 -- route tool calls through MCP bridge instead of in-process plugins.
        via_mcp: bool,
    },
    /// --remote-sidecar: SSH-boot the GPU box sidecar, start the local tunnel,
    /// health-poll until ready, then launch the V2 dashboard with
    /// ANTHROPIC_BASE_URL=http://localhost:8100.
    RemoteSidecar,
    HelpTopic(LocalHelpTopic),
    // prompt-mode formatting is only supported for non-interactive runs
    Help {
        output_format: CliOutputFormat,
    },
    /// B-29 Phase 4: `elai symbol_graph build [--project-root <path>]`
    /// Runs a full symbol-graph build for the given project root.
    SymbolGraphBuild {
        project_root: PathBuf,
    },
    /// B-29 Phase 4: `elai symbol_graph stats [--project-root <path>]`
    /// Prints SymbolGraphMeta (symbol_count, edge_count, unresolved_edge_pct, …).
    SymbolGraphStats {
        project_root: PathBuf,
    },
    /// E-21 Tier-A: `elai symbol_graph rank --top N [--project-root <path>]`
    /// Prints top-N PageRank symbols from the existing dkg_nodes_pagerank
    /// table (populated by knowledge::pipeline::compute_repo_map / C-3 ship).
    /// Graceful-empty when table is absent — same shape as the dashboard's
    /// get_pagerank_top Tauri command.
    SymbolGraphRank {
        project_root: PathBuf,
        top: u32,
    },
    /// C-13: `elai wip-squash [--project-root <path>]` — invoked by the
    /// Python FSM hook chain at AUDITING -> MEMORIZING transitions when
    /// the Rust task_runner is not in scope (the hook layer fires the
    /// transition from outside the runner process). Forwards to
    /// `elai_orchestrator::wip_squash::squash_wip_commits_if_any`.
    WipSquash {
        project_root: PathBuf,
    },
    /// C-20: `elai measure [--scope <name>] [--cloud] [--output toml|json]`
    /// — run the test-infra measurement suite. Direct (non-REPL) invocation
    /// of `/measure`; the three parser-string fields carry the slash-command
    /// shape verbatim (validation lives at the handler boundary in
    /// `crate::measure::run_measure_subcommand`).
    Measure {
        scope: String,
        cloud: bool,
        output_format: CliOutputFormat,
    },
    /// C-25: `elai pipeline [list|reset]` — inspect or reset the compiled-pipeline
    /// cache at `.elai/cache/compiled_pipelines.msgpack`. Direct (non-REPL)
    /// invocation of `/pipeline`. `action` is the lower-cased subcommand
    /// (`"list"` | `"reset"` | `"help"` | None=>list). Handler lives in
    /// `crate::pipeline_subcommand::run_pipeline_subcommand`.
    Pipeline {
        action: Option<String>,
        output_format: CliOutputFormat,
    },
    /// C-27: `elai wiring audit [--scope all|stub-ban|dkg-reachability|observable-effect]
    /// [--config PATH] [--contracts PATH] [--output toml|json]`
    /// — run the deterministic Tier T-0 wiring contract audit. Mirrors the
    /// Measure/Pipeline pattern (string-typed scope; validation lives at
    /// the handler boundary in `crate::wiring_audit::run`). Exits 0 on
    /// clean audit, 1 on any violation.
    WiringAudit {
        scope: Option<String>,
        config_path: Option<String>,
        contracts_dir: Option<String>,
        output_format: CliOutputFormat,
    },
    /// D-8: `elai archive verify [--path PATH] [--json] [--fail-fast]`.
    /// Surfaces D-1's ArchiveChain::verify_chain as a user-facing CLI +
    /// CI hook. Exits 0 on clean chain, 1 on any tamper / broken-parent /
    /// IO / malformed-JSON. Path default resolves to
    /// `<cwd>/archive/reasoning-templates.jsonl` (matches D-1 writer).
    ArchiveVerify {
        path: Option<String>,
        json: bool,
        fail_fast: bool,
    },
    /// D-9: `elai audit-locked-keys [--registry PATH] [--staged | --files A,B]
    /// [--json] [--fail-fast]` — run LockedKeysAuditor against the staged
    /// diff (or an explicit file list) and exit non-zero on any violation.
    /// Default `--staged` invokes `git diff --cached --name-only` to drive
    /// the inputs; `--files` accepts a comma-separated list for CI /
    /// scripted use. JSON output mirrors `archive verify --json`.
    AuditLockedKeys {
        registry_path: Option<String>,
        files: Option<String>,
        use_staged: bool,
        json: bool,
        fail_fast: bool,
    },
    /// D-19: `elai bench prmbench [--dataset PATH] [--hook mock|http://host:port]
    /// [--output PATH] [--gating-threshold 0.30] [--category NAME]`
    /// Runs the PRMBench adversarial-gaming benchmark against the PPM
    /// verifier (or its mock). Exits 0 on Pass, 1 on Fail. JSON report is
    /// written to `--output` (default stdout). Owns its own tokio runtime
    /// (sync entrypoint pattern — mirrors `archive_verify`). Threshold is
    /// carried as a String (CliAction derives Eq; f64 does not) and parsed
    /// at the handler boundary.
    BenchPrmbench {
        dataset: Option<String>,
        hook: String,
        output: Option<String>,
        gating_threshold: Option<String>,
        category: Option<String>,
    },
    /// D-21: `elai bench mode-routing [--dataset PATH] [--output PATH]
    /// [--macro-floor 0.85] [--per-mode-floor 0.70]`
    /// Replays the hand-labeled mode-routing fixture through D-10's
    /// `select_grounding_mode` predicate and gates pass iff macro F1 >=
    /// macro-floor AND every per-mode F1 >= per-mode-floor. Exits 0/1.
    /// Floor values are carried as String (CliAction derives Eq; f64 does
    /// not) and parsed at the handler boundary.
    BenchModeRouting {
        dataset: Option<String>,
        output: Option<String>,
        macro_floor: Option<String>,
        per_mode_floor: Option<String>,
    },
    /// D-22: `elai bench conformal [--dataset PATH] [--corpus PATH]
    /// [--output PATH] [--epsilon 0.1]`. Runs the conformal calibration
    /// validation benchmark against the calibration corpus + held-out
    /// validation fixture. Exits 0 on Pass, 1 on Fail. JSON report is
    /// written to `--output` (default stdout). Owns its own tokio
    /// runtime (sync entrypoint pattern — mirrors bench_mode_routing).
    /// Epsilon is carried as a String (CliAction derives Eq; f64 does
    /// not) and parsed at the handler boundary.
    BenchConformal {
        dataset: Option<String>,
        corpus: Option<String>,
        output: Option<String>,
        epsilon: Option<String>,
    },
    /// D-20: `elai bench arc [--dataset PATH] [--output PATH]
    /// [--novelty-config PATH]`. Replays an ARC-AGI-2 fixture through
    /// D-10's `select_grounding_mode` predicate and D-15's `classify`
    /// predicate, then aggregates the pair-wise outcomes into the
    /// closed [`ARCVerdict`] taxonomy (HonestlyEvaluated /
    /// GameWithMemorization / Refused). Exits 0 on HonestlyEvaluated,
    /// 1 otherwise. JSON report written to `--output` (default stdout).
    /// Owns its own tokio runtime (sync entrypoint pattern — mirrors
    /// `bench_mode_routing` + `bench_conformal`).
    BenchArc {
        dataset: Option<String>,
        output: Option<String>,
        novelty_config: Option<String>,
    },
    /// E-17 SP6: `elai bench rag --tier <STANDARD|DEEP> --top-k <N>
    /// [--project-root PATH] [--json] -- <QUERY...>`. Runs the local
    /// knowledge pipeline at the requested tier and emits the final-stage
    /// retrieval results as JSON (or a human-readable table). Used by
    /// `.elai_cc/eval/rag/run.py` as its observable interface; without
    /// this subcommand the eval harness has no clean way to drive the
    /// pipeline (the augment_with_knowledge code path is wrapped in
    /// full prompt-turn machinery and emits a chat response, not
    /// parseable retrieval JSON). Mirrors `bench_arc` sync entry pattern.
    BenchRag {
        tier: String,
        top_k: u32,
        query: String,
        json: bool,
        project_root: PathBuf,
    },
    // E-30: `elai incident record --title <TITLE> --date <DATE> --url <URL>
    //        [--id <ID>]` — store an incident postmortem entry in the SSB
    //        and emit IncidentRecordCreated so the dashboard can surface it.
    IncidentRecord {
        title: String,
        incident_date: String,
        source_url: String,
        incident_id: Option<String>,
    },
}

/// 1.1 — Minimal PartialEq-capable carrier for a local-backend URL.
/// BackendConfig does not derive PartialEq (serialisation-only struct), so we
/// store just the sidecar URL here for CliAction equality in tests and convert
/// to a full BackendConfig when constructing the runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalBackendRef {
    pub(crate) sidecar_url: String,
}

impl From<LocalBackendRef> for elai_orchestrator::BackendConfig {
    fn from(r: LocalBackendRef) -> Self {
        elai_orchestrator::BackendConfig {
            mode: elai_orchestrator::BackendMode::Local,
            sidecar_url: r.sidecar_url,
            ..elai_orchestrator::BackendConfig::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocalHelpTopic {
    Status,
    Sandbox,
    Doctor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CliOutputFormat {
    Text,
    Json,
    /// C-21 DRIFT-6 — dep-audit `--output-format toml`. The TOML report is
    /// always written to `metrics/dep_audit_<YYYY-MM-DD>.toml`; when this
    /// variant is selected, the dep-audit subcommand additionally emits
    /// `metrics/dep_audit_latest.toml` (copy of the dated file) so callers
    /// like Layer1A DIVERGING-state pipelines can locate the most recent
    /// audit deterministically.
    Toml,
}

impl CliOutputFormat {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            "toml" => Ok(Self::Toml),
            other => Err(format!(
                "unsupported value for --output-format: {other} (expected text, json, or toml)"
            )),
        }
    }

    /// C-21 Task 6 — for consumers that only know Text/Json, fold the
    /// `Toml` variant down to `Text` so non-dep-audit subcommands don't
    /// have to grow Toml-specific renderers. The dep-audit consumer
    /// explicitly matches `Toml` (see doctor::run_dep_audit_command) so
    /// the demotion is invisible for the one subcommand that cares.
    #[allow(dead_code)]
    pub(crate) fn for_generic_consumer(self) -> Self {
        match self {
            Self::Text | Self::Toml => Self::Text,
            Self::Json => Self::Json,
        }
    }
}

/// 1.1: Resolve the optional backend config for Prompt actions.
/// Explicit --backend=local flag takes precedence; if absent, check
/// ANTHROPIC_BASE_URL for a localhost/127.0.0.1 base URL and infer Local mode.
pub(crate) fn resolve_prompt_backend(flag: Option<LocalBackendRef>) -> Option<LocalBackendRef> {
    flag.or_else(|| {
        std::env::var("ANTHROPIC_BASE_URL").ok().and_then(|url| {
            if url.contains("localhost") || url.contains("127.0.0.1") {
                let port: u16 = url
                    .split(':')
                    .last()
                    .and_then(|s| s.trim_end_matches('/').parse().ok())
                    .unwrap_or(8100);
                Some(LocalBackendRef {
                    sidecar_url: format!("http://localhost:{port}"),
                })
            } else {
                None
            }
        })
    })
}

#[allow(clippy::too_many_lines)]
pub(crate) fn parse_args(args: &[String]) -> Result<CliAction, String> {
    let mut model = DEFAULT_MODEL.to_string();
    let mut output_format = CliOutputFormat::Text;
    let mut permission_mode_override = None;
    let mut wants_help = false;
    let mut wants_version = false;
    let mut allowed_tool_values = Vec::new();
    let mut rest = Vec::new();
    let mut export_jsonl = false;
    // 1.1: --backend=local[:PORT] flag; None = use API path (default)
    let mut backend_flag: Option<LocalBackendRef> = None;
    // C-35: --tool-gate-off flag -- disables dynamic tool gating at runtime.
    let mut tool_gate_off = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" if rest.is_empty() => {
                wants_help = true;
                index += 1;
            }
            "--version" | "-V" => {
                wants_version = true;
                index += 1;
            }
            "--model" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --model".to_string())?;
                model = resolve_model_alias(value).to_string();
                index += 2;
            }
            flag if flag.starts_with("--model=") => {
                model = resolve_model_alias(&flag[8..]).to_string();
                index += 1;
            }
            "--output-format" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --output-format".to_string())?;
                output_format = CliOutputFormat::parse(value)?;
                index += 2;
            }
            "--permission-mode" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --permission-mode".to_string())?;
                permission_mode_override = Some(parse_permission_mode_arg(value)?);
                index += 2;
            }
            flag if flag.starts_with("--output-format=") => {
                output_format = CliOutputFormat::parse(&flag[16..])?;
                index += 1;
            }
            flag if flag.starts_with("--permission-mode=") => {
                permission_mode_override = Some(parse_permission_mode_arg(&flag[18..])?);
                index += 1;
            }
            "--dangerously-skip-permissions" => {
                permission_mode_override = Some(PermissionMode::DangerFullAccess);
                index += 1;
            }
            "--tool-gate-off" => {
                // C-35: escape hatch to disable dynamic tool gating at runtime.
                // Used for debugging or when gating regresses task-solve rate.
                tool_gate_off = true;
                index += 1;
            }
            "--export-jsonl" => {
                // B-9 cycle 2 (BLOCK-1 fix): opt-in legacy JSONL persistence.
                // Default off — SessionDag is the authoritative store for
                // every REPL turn. JSONL stays available for export
                // tooling that still consumes the on-disk transcript.
                export_jsonl = true;
                index += 1;
            }
            // 1.1: --backend=local or --backend=local:PORT routes Prompt
            // through BudgetPolicy::Local instead of BudgetPolicy::ApiDefault.
            flag if flag.starts_with("--backend=local") => {
                let port: u16 = flag
                    .strip_prefix("--backend=local:")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(8100);
                backend_flag = Some(LocalBackendRef {
                    sidecar_url: format!("http://localhost:{port}"),
                });
                index += 1;
            }
            "--local" => {
                // Local inference mode: auto-start sidecar, interactive chat
                let port = args.get(index + 1)
                    .and_then(|s| s.parse::<u16>().ok())
                    .unwrap_or(8100);
                return Ok(CliAction::Local { port });
            }
            "--dashboard" => {
                return Ok(CliAction::Dashboard);
            }
            "--remote-sidecar" => {
                return Ok(CliAction::RemoteSidecar);
            }
            "-p" => {
                // Compat: -p "prompt" = one-shot prompt
                let prompt = args[index + 1..].join(" ");
                if prompt.trim().is_empty() {
                    return Err("-p requires a prompt string".to_string());
                }
                return Ok(CliAction::Prompt {
                    prompt,
                    model: resolve_model_alias(&model).to_string(),
                    output_format,
                    allowed_tools: normalize_allowed_tools(&allowed_tool_values)?,
                    permission_mode: permission_mode_override
                        .unwrap_or_else(default_permission_mode),
                    backend: resolve_prompt_backend(backend_flag.clone()),
                });
            }
            "--print" => {
                // Compat: --print makes output non-interactive
                output_format = CliOutputFormat::Text;
                index += 1;
            }
            "--resume" if rest.is_empty() => {
                rest.push("--resume".to_string());
                index += 1;
            }
            flag if rest.is_empty() && flag.starts_with("--resume=") => {
                rest.push("--resume".to_string());
                rest.push(flag[9..].to_string());
                index += 1;
            }
            "--allowedTools" | "--allowed-tools" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --allowedTools".to_string())?;
                allowed_tool_values.push(value.clone());
                index += 2;
            }
            flag if flag.starts_with("--allowedTools=") => {
                allowed_tool_values.push(flag[15..].to_string());
                index += 1;
            }
            flag if flag.starts_with("--allowed-tools=") => {
                allowed_tool_values.push(flag[16..].to_string());
                index += 1;
            }
            other if rest.is_empty() && other.starts_with('-') => {
                return Err(format_unknown_option(other))
            }
            other => {
                rest.push(other.to_string());
                index += 1;
            }
        }
    }

    if wants_help {
        return Ok(CliAction::Help { output_format });
    }

    if wants_version {
        return Ok(CliAction::Version { output_format });
    }

    // 1.1: resolve backend (explicit flag wins, then ANTHROPIC_BASE_URL auto-detect).
    let backend_config = resolve_prompt_backend(backend_flag);

    let allowed_tools = normalize_allowed_tools(&allowed_tool_values)?;

    if rest.is_empty() {
        let permission_mode = permission_mode_override.unwrap_or_else(default_permission_mode);
        return Ok(CliAction::Repl {
            model,
            allowed_tools,
            permission_mode,
            export_jsonl,
        });
    }
    if rest.first().map(String::as_str) == Some("--resume") {
        return parse_resume_args(&rest[1..], output_format);
    }
    if let Some(action) = parse_local_help_action(&rest) {
        return action;
    }
    if let Some(action) =
        parse_single_word_command_alias(&rest, &model, permission_mode_override, output_format)
    {
        return action;
    }

    let permission_mode = permission_mode_override.unwrap_or_else(default_permission_mode);

    if tool_gate_off {
        // C-35: propagate the disable-gate flag via env var so downstream
        // config loaders (runtime_config.rs) can read it without a parse_args refactor.
        std::env::set_var("ELAI_TOOL_GATE_OFF", "1");
    }

    match rest[0].as_str() {
        "dump-manifests" => Ok(CliAction::DumpManifests { output_format }),
        "bootstrap-plan" => Ok(CliAction::BootstrapPlan { output_format }),
        "agents" => Ok(CliAction::Agents {
            args: join_optional_args(&rest[1..]),
            output_format,
        }),
        "mcp" => Ok(CliAction::Mcp {
            args: join_optional_args(&rest[1..]),
            output_format,
        }),
        // B-37D Phase 4M: route `elai runtime …` through the slash-command
        // parser so the strict grammar (list / test <backend> /
        // set --role <r> --backend <b> / help) stays a single source of truth.
        "runtime" => {
            let raw = std::iter::once("/runtime".to_string())
                .chain(rest[1..].iter().cloned())
                .collect::<Vec<_>>()
                .join(" ");
            match SlashCommand::parse(&raw) {
                Ok(Some(SlashCommand::Runtime {
                    action,
                    role,
                    target,
                })) => Ok(CliAction::Runtime {
                    action,
                    role,
                    target,
                    output_format,
                }),
                Ok(_) => Err(format!("unexpected slash command: {raw}")),
                Err(error) => Err(error.to_string()),
            }
        }
        "skills" => Ok(CliAction::Skills {
            args: join_optional_args(&rest[1..]),
            output_format,
        }),
        "system-prompt" => parse_system_prompt_args(&rest[1..], output_format),
        "dep-audit" => {
            let project_dir = if rest.len() > 1 {
                PathBuf::from(&rest[1])
            } else {
                std::env::current_dir().map_err(|e| format!("cannot determine cwd: {e}"))?
            };
            Ok(CliAction::DepAudit {
                project_dir,
                output_format,
            })
        }
        // E-17 SP1 task 1 — `elai doctor [knowledge-models ...]`.
        // `elai doctor` (bare) is also reachable via the single-word
        // command alias at line 1438, but the parse_args dispatch table
        // is the only path that sees sub-arguments (e.g. `doctor
        // knowledge-models --project-root PATH`), so route the
        // sub-subcommand here. Bare `doctor` falls through to the
        // existing CliAction::Doctor variant for byte-for-byte
        // backwards compatibility with the bare-slash entry point.
        "doctor" => parse_doctor_subcommand_args(&rest[1..], output_format),
        "login" => Ok(CliAction::Login { output_format }),
        "logout" => Ok(CliAction::Logout { output_format }),
        "init" => Ok(CliAction::Init { output_format }),
        "run" => parse_run_subcommand_args(&rest[1..]),
        "symbol_graph" | "symbol-graph" => parse_symbol_graph_args(&rest[1..]),
        "wip-squash" | "wip_squash" => parse_wip_squash_args(&rest[1..]),
        // D-19: `elai bench prmbench ...` — PRMBench adversarial-gaming benchmark.
        "bench" => parse_bench_subcommand_args(&rest[1..]),
        // C-20: route `elai measure …` through the slash-command parser so
        // the grammar is a single source of truth (mirrors the `runtime`
        // routing immediately above).
        "measure" => {
            let raw = std::iter::once("/measure".to_string())
                .chain(rest[1..].iter().cloned())
                .collect::<Vec<_>>()
                .join(" ");
            match SlashCommand::parse(&raw) {
                Ok(Some(SlashCommand::Measure {
                    scope,
                    cloud,
                    output_format: _of,
                })) => Ok(CliAction::Measure {
                    scope,
                    cloud,
                    output_format,
                }),
                Ok(_) => Err(format!("unexpected slash command: {raw}")),
                Err(error) => Err(error.to_string()),
            }
        }
        // C-25: `elai pipeline [list|reset]` routes through the slash-command
        // parser so the grammar stays single-sourced.
        "pipeline" => {
            let raw = std::iter::once("/pipeline".to_string())
                .chain(rest[1..].iter().cloned())
                .collect::<Vec<_>>()
                .join(" ");
            match SlashCommand::parse(&raw) {
                Ok(Some(SlashCommand::Pipeline { action })) => Ok(CliAction::Pipeline {
                    action,
                    output_format,
                }),
                Ok(_) => Err(format!("unexpected slash command: {raw}")),
                Err(error) => Err(error.to_string()),
            }
        }
        // C-27: `elai wiring audit ...` routes through the slash-command
        // parser. Mirrors the pipeline/measure shape (string-typed,
        // single-grammar source).
        "wiring" => {
            let raw = std::iter::once("/wiring".to_string())
                .chain(rest[1..].iter().cloned())
                .collect::<Vec<_>>()
                .join(" ");
            match SlashCommand::parse(&raw) {
                Ok(Some(SlashCommand::WiringAudit {
                    scope,
                    config_path,
                    contracts_dir,
                })) => Ok(CliAction::WiringAudit {
                    scope,
                    config_path,
                    contracts_dir,
                    output_format,
                }),
                Ok(_) => Err(format!("unexpected slash command: {raw}")),
                Err(error) => Err(error.to_string()),
            }
        }
        // D-8: `elai archive verify [--path PATH] [--json] [--fail-fast]`
        // Surfaces ArchiveChain::verify_chain (D-1) as a CLI subcommand +
        // CI hook. Parser is local (no slash-command grammar needed --
        // this is a top-level `elai archive verify` invocation, not a
        // REPL slash command).
        "archive" => parse_archive_subcommand_args(&rest[1..]),
        // D-9 — `elai audit-locked-keys` is a top-level subcommand, not a
        // slash command (CI + pre-commit hook want a simple argv shape).
        "audit-locked-keys" => parse_audit_locked_keys_args(&rest[1..]),
        "prompt" => {
            let prompt = rest[1..].join(" ");
            if prompt.trim().is_empty() {
                return Err("prompt subcommand requires a prompt string".to_string());
            }
            Ok(CliAction::Prompt {
                prompt,
                model,
                output_format,
                allowed_tools,
                permission_mode,
                backend: backend_config.clone(),
            })
        }
        // E-30: `elai incident record --title <TITLE> --date <DATE> --url <URL>
        // [--id <ID>]` — store an incident postmortem entry in the SSB.
        "incident" => parse_incident_subcommand_args(&rest[1..]),
        other if other.starts_with('/') => parse_direct_slash_cli_action(&rest, output_format),
        _other => Ok(CliAction::Prompt {
            prompt: rest.join(" "),
            model,
            output_format,
            allowed_tools,
            permission_mode,
            backend: backend_config,
        }),
    }
}

/// Phase A1a — parse `run` subcommand args.
///
/// Supports `--project-root <path>` and `--project-root=<path>`; defaults to
/// `std::env::current_dir()` when the flag is absent.
///
/// C-9 — also supports `--mode <autonomous|interactive|hybrid>` (default
/// interactive). Validation lives at the gate construction site
/// (`GateMode::from_cli_args`); unknown values resolve to Interactive there.
/// Anything else is rejected with a typed error string so the top-level
/// dispatcher can format it. Keeping the parser tight here protects the
/// chokepoint contract test from drift introduced by a future flag.
pub fn parse_run_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let mut project_root: Option<PathBuf> = None;
    let mut mode: Option<String> = None;
    let mut provider: Option<String> = None;
    let mut via_mcp: bool = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--project-root" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --project-root".to_string())?;
                project_root = Some(PathBuf::from(value));
                i += 2;
            }
            flag if flag.starts_with("--project-root=") => {
                project_root = Some(PathBuf::from(&flag[15..]));
                i += 1;
            }
            "--mode" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --mode".to_string())?;
                let lower = value.trim().to_ascii_lowercase();
                if !matches!(lower.as_str(), "autonomous" | "interactive" | "hybrid") {
                    return Err(format!(
                        "unsupported value for --mode: {value} (expected autonomous, interactive, or hybrid)"
                    ));
                }
                mode = Some(lower);
                i += 2;
            }
            flag if flag.starts_with("--mode=") => {
                let value = &flag[7..];
                let lower = value.trim().to_ascii_lowercase();
                if !matches!(lower.as_str(), "autonomous" | "interactive" | "hybrid") {
                    return Err(format!(
                        "unsupported value for --mode: {value} (expected autonomous, interactive, or hybrid)"
                    ));
                }
                mode = Some(lower);
                i += 1;
            }
            // C-5: --provider local|cloud|hybrid (provider-routing axis).
            "--provider" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --provider".to_string())?;
                let lower = value.trim().to_ascii_lowercase();
                if !matches!(lower.as_str(), "local" | "cloud" | "hybrid") {
                    return Err(format!(
                        "unsupported value for --provider: {value} (expected local, cloud, or hybrid)"
                    ));
                }
                provider = Some(lower);
                i += 2;
            }
            flag if flag.starts_with("--provider=") => {
                let value = &flag[11..];
                let lower = value.trim().to_ascii_lowercase();
                if !matches!(lower.as_str(), "local" | "cloud" | "hybrid") {
                    return Err(format!(
                        "unsupported value for --provider: {value} (expected local, cloud, or hybrid)"
                    ));
                }
                provider = Some(lower);
                i += 1;
            }
            "--via-mcp" => {
                via_mcp = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `run` argument: {other}"));
            }
        }
    }
    let project_root = match project_root {
        Some(p) => p,
        None => std::env::current_dir()
            .map_err(|e| format!("`run` cannot determine cwd for --project-root default: {e}"))?,
    };
    Ok(CliAction::Run { project_root, mode, provider, via_mcp })
}

/// B-29 Phase 4 — parse `symbol_graph` / `symbol-graph` subcommand args.
///
/// Usage:
///   elai symbol_graph build [--project-root <path>]
///   elai symbol_graph stats [--project-root <path>]
pub(crate) fn parse_symbol_graph_args(rest: &[String]) -> Result<CliAction, String> {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    let flag_rest = if rest.is_empty() { &rest[..] } else { &rest[1..] };

    let mut project_root: Option<PathBuf> = None;
    // E-21 Tier-A: `rank --top N` accepts an additional --top flag (default 20).
    let mut top: u32 = 0;
    let mut i = 0;
    while i < flag_rest.len() {
        match flag_rest[i].as_str() {
            "--project-root" => {
                let value = flag_rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --project-root".to_string())?;
                project_root = Some(PathBuf::from(value));
                i += 2;
            }
            flag if flag.starts_with("--project-root=") => {
                project_root = Some(PathBuf::from(&flag[15..]));
                i += 1;
            }
            "--top" => {
                let value = flag_rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --top".to_string())?;
                top = value
                    .parse::<u32>()
                    .map_err(|e| format!("--top must be a non-negative integer: {e}"))?;
                i += 2;
            }
            flag if flag.starts_with("--top=") => {
                top = flag[6..]
                    .parse::<u32>()
                    .map_err(|e| format!("--top must be a non-negative integer: {e}"))?;
                i += 1;
            }
            other => {
                return Err(format!("unknown `symbol_graph` argument: {other}"));
            }
        }
    }

    let project_root = match project_root {
        Some(p) => p,
        None => std::env::current_dir().map_err(|e| {
            format!("`symbol_graph` cannot determine cwd for --project-root default: {e}")
        })?,
    };

    match sub {
        "build" => Ok(CliAction::SymbolGraphBuild { project_root }),
        "stats" => Ok(CliAction::SymbolGraphStats { project_root }),
        "rank" => Ok(CliAction::SymbolGraphRank { project_root, top }),
        other => Err(format!(
            "unknown `symbol_graph` sub-command: '{other}' (expected 'build', 'stats', or 'rank')"
        )),
    }
}

/// E-17 SP1 task 1 — parse `elai doctor [knowledge-models ...]`.
///
/// `elai doctor` (no sub-subcommand) returns `CliAction::Doctor` —
/// byte-for-byte identical to the bare-slash entry at args.rs:1438.
///
/// `elai doctor knowledge-models [--project-root PATH]
/// [--output-format text|json|toml]` returns
/// `CliAction::DoctorKnowledgeModels`. `--output-format` here is
/// consumed by the top-level flag parser (already parsed into
/// `output_format` by the caller); a `--output-format` flag mixed in
/// after `knowledge-models` is *also* honored so the subcommand
/// surface matches `bench prmbench`, which accepts both placements.
pub(crate) fn parse_doctor_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    if sub.is_empty() {
        // `elai doctor` with no further arg — same as bare slash.
        return Ok(CliAction::Doctor { output_format });
    }
    if sub != "knowledge-models" {
        return Err(format!(
            "unknown `doctor` sub-command: '{sub}' (expected 'knowledge-models')"
        ));
    }

    let flag_rest = &rest[1..];
    let mut project_root: Option<PathBuf> = None;
    let mut output_format_override: Option<CliOutputFormat> = None;
    let mut i = 0;
    while i < flag_rest.len() {
        match flag_rest[i].as_str() {
            "--project-root" => {
                let value = flag_rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --project-root".to_string())?;
                project_root = Some(PathBuf::from(value));
                i += 2;
            }
            flag if flag.starts_with("--project-root=") => {
                project_root = Some(PathBuf::from(&flag[15..]));
                i += 1;
            }
            "--output-format" => {
                let value = flag_rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --output-format".to_string())?;
                output_format_override = Some(parse_output_format_value(value)?);
                i += 2;
            }
            flag if flag.starts_with("--output-format=") => {
                output_format_override = Some(parse_output_format_value(&flag[16..])?);
                i += 1;
            }
            "--json" => {
                // Convenience alias matching the `archive verify --json`
                // shape — same as `--output-format json`.
                output_format_override = Some(CliOutputFormat::Json);
                i += 1;
            }
            other => {
                return Err(format!(
                    "unknown `doctor knowledge-models` argument: {other}"
                ));
            }
        }
    }

    let project_root = match project_root {
        Some(p) => p,
        None => std::env::current_dir().map_err(|e| {
            format!(
                "`doctor knowledge-models` cannot determine cwd for --project-root default: {e}"
            )
        })?,
    };
    Ok(CliAction::DoctorKnowledgeModels {
        project_root,
        output_format: output_format_override.unwrap_or(output_format),
    })
}

fn parse_output_format_value(value: &str) -> Result<CliOutputFormat, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "text" => Ok(CliOutputFormat::Text),
        "json" => Ok(CliOutputFormat::Json),
        "toml" => Ok(CliOutputFormat::Toml),
        other => Err(format!(
            "unsupported value for --output-format: {other} (expected text, json, or toml)"
        )),
    }
}

/// D-8 — parse `elai archive verify [--path PATH] [--json] [--fail-fast]`.
///
/// Only one sub-verb is currently supported (`verify`). Other archive
/// sub-verbs (e.g. compact, inspect) are out-of-scope; the parser errors out
/// on unknown sub-verbs so a typo doesn't silently fall through to the
/// fallback Prompt action.
pub(crate) fn parse_archive_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    if sub != "verify" {
        return Err(format!(
            "unknown `archive` sub-command: '{sub}' (expected 'verify')"
        ));
    }
    let mut path: Option<String> = None;
    let mut json = false;
    let mut fail_fast = false;
    let mut i = 1;
    while i < rest.len() {
        match rest[i].as_str() {
            "--path" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --path".to_string())?;
                path = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--path=") => {
                path = Some(flag[7..].to_string());
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            "--fail-fast" => {
                fail_fast = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `archive verify` argument: {other}"));
            }
        }
    }
    Ok(CliAction::ArchiveVerify {
        path,
        json,
        fail_fast,
    })
}

/// D-9 — parse `elai audit-locked-keys [--registry PATH] [--staged | --files A,B]
/// [--json] [--fail-fast]`.
///
/// `--staged` and `--files` are mutually exclusive; `--staged` is the
/// implicit default so pre-commit hooks can invoke the bare subcommand.
/// CI callers wanting full control pass `--files file1,file2,...`.
pub(crate) fn parse_audit_locked_keys_args(rest: &[String]) -> Result<CliAction, String> {
    let mut registry_path: Option<String> = None;
    let mut files: Option<String> = None;
    let mut use_staged = true;
    let mut explicit_staged = false;
    let mut json = false;
    let mut fail_fast = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--registry" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --registry".to_string())?;
                registry_path = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--registry=") => {
                registry_path = Some(flag["--registry=".len()..].to_string());
                i += 1;
            }
            "--files" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --files".to_string())?;
                files = Some(value.clone());
                use_staged = false;
                i += 2;
            }
            flag if flag.starts_with("--files=") => {
                files = Some(flag["--files=".len()..].to_string());
                use_staged = false;
                i += 1;
            }
            "--staged" => {
                explicit_staged = true;
                use_staged = true;
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            "--fail-fast" => {
                fail_fast = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `audit-locked-keys` argument: {other}"));
            }
        }
    }
    if explicit_staged && files.is_some() {
        return Err("--staged and --files are mutually exclusive".to_string());
    }
    Ok(CliAction::AuditLockedKeys {
        registry_path,
        files,
        use_staged,
        json,
        fail_fast,
    })
}

/// D-19 — parse `elai bench prmbench [--dataset PATH] [--hook mock|http://host:port]
/// [--output PATH] [--gating-threshold 0.30] [--category NAME]`.
///
/// Only one sub-verb is supported (`prmbench`). The flag grammar mirrors
/// `archive verify`: long-form `--flag VALUE` and `--flag=VALUE` both
/// accepted. Validation of the values (URL parsing for `--hook`, range
/// check on `--gating-threshold`) lives in the bench_prmbench handler
/// boundary so this parser stays free of orchestrator type deps.
pub(crate) fn parse_bench_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    if sub == "mode-routing" {
        return parse_bench_mode_routing_args(&rest[1..]);
    }
    if sub == "conformal" {
        return parse_bench_conformal_args(&rest[1..]);
    }
    if sub == "arc" {
        return parse_bench_arc_args(&rest[1..]);
    }
    if sub == "rag" {
        return parse_bench_rag_args(&rest[1..]);
    }
    if sub != "prmbench" {
        return Err(format!(
            "unknown `bench` sub-command: '{sub}' (expected 'prmbench' | 'mode-routing' | 'conformal' | 'arc' | 'rag')"
        ));
    }
    let mut dataset: Option<String> = None;
    let mut hook: String = "mock".to_string();
    let mut output: Option<String> = None;
    let mut gating_threshold: Option<String> = None;
    let mut category: Option<String> = None;
    let mut i = 1;
    while i < rest.len() {
        match rest[i].as_str() {
            "--dataset" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --dataset".to_string())?;
                dataset = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--dataset=") => {
                dataset = Some(flag["--dataset=".len()..].to_string());
                i += 1;
            }
            "--hook" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --hook".to_string())?;
                hook = value.clone();
                i += 2;
            }
            flag if flag.starts_with("--hook=") => {
                hook = flag["--hook=".len()..].to_string();
                i += 1;
            }
            "--output" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --output".to_string())?;
                output = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--output=") => {
                output = Some(flag["--output=".len()..].to_string());
                i += 1;
            }
            "--gating-threshold" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --gating-threshold".to_string())?;
                // Validate parseability up front so a malformed flag fails
                // at the parser boundary instead of mid-run.
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --gating-threshold {value:?}: {e}"))?;
                gating_threshold = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--gating-threshold=") => {
                let value = &flag["--gating-threshold=".len()..];
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --gating-threshold {value:?}: {e}"))?;
                gating_threshold = Some(value.to_string());
                i += 1;
            }
            "--category" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --category".to_string())?;
                category = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--category=") => {
                category = Some(flag["--category=".len()..].to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench prmbench` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchPrmbench {
        dataset,
        hook,
        output,
        gating_threshold,
        category,
    })
}

/// D-21 — parse `elai bench mode-routing [--dataset PATH] [--output PATH]
/// [--macro-floor 0.85] [--per-mode-floor 0.70]`. Mirrors the prmbench
/// parser; hand-rolled per the CLAUDE.md "no new clap dep" rule.
/// Validation of the floor values lives at the handler boundary.
pub(crate) fn parse_bench_mode_routing_args(rest: &[String]) -> Result<CliAction, String> {
    let mut dataset: Option<String> = None;
    let mut output: Option<String> = None;
    let mut macro_floor: Option<String> = None;
    let mut per_mode_floor: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--dataset" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --dataset".to_string())?;
                dataset = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--dataset=") => {
                dataset = Some(flag["--dataset=".len()..].to_string());
                i += 1;
            }
            "--output" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --output".to_string())?;
                output = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--output=") => {
                output = Some(flag["--output=".len()..].to_string());
                i += 1;
            }
            "--macro-floor" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --macro-floor".to_string())?;
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --macro-floor {value:?}: {e}"))?;
                macro_floor = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--macro-floor=") => {
                let value = &flag["--macro-floor=".len()..];
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --macro-floor {value:?}: {e}"))?;
                macro_floor = Some(value.to_string());
                i += 1;
            }
            "--per-mode-floor" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --per-mode-floor".to_string())?;
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --per-mode-floor {value:?}: {e}"))?;
                per_mode_floor = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--per-mode-floor=") => {
                let value = &flag["--per-mode-floor=".len()..];
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --per-mode-floor {value:?}: {e}"))?;
                per_mode_floor = Some(value.to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench mode-routing` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchModeRouting {
        dataset,
        output,
        macro_floor,
        per_mode_floor,
    })
}

/// D-22 — parse `elai bench conformal [--dataset PATH] [--corpus PATH]
/// [--output PATH] [--epsilon 0.1]`. Mirrors the mode-routing parser;
/// hand-rolled per the CLAUDE.md "no new clap dep" rule. Validation of
/// the epsilon value lives at the handler boundary.
pub(crate) fn parse_bench_conformal_args(rest: &[String]) -> Result<CliAction, String> {
    let mut dataset: Option<String> = None;
    let mut corpus: Option<String> = None;
    let mut output: Option<String> = None;
    let mut epsilon: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--dataset" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --dataset".to_string())?;
                dataset = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--dataset=") => {
                dataset = Some(flag["--dataset=".len()..].to_string());
                i += 1;
            }
            "--corpus" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --corpus".to_string())?;
                corpus = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--corpus=") => {
                corpus = Some(flag["--corpus=".len()..].to_string());
                i += 1;
            }
            "--output" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --output".to_string())?;
                output = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--output=") => {
                output = Some(flag["--output=".len()..].to_string());
                i += 1;
            }
            "--epsilon" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --epsilon".to_string())?;
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --epsilon {value:?}: {e}"))?;
                epsilon = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--epsilon=") => {
                let value = &flag["--epsilon=".len()..];
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --epsilon {value:?}: {e}"))?;
                epsilon = Some(value.to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench conformal` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchConformal {
        dataset,
        corpus,
        output,
        epsilon,
    })
}

/// D-20 — parse `elai bench arc [--dataset PATH] [--output PATH]
/// [--novelty-config PATH]`. Hand-rolled per the CLAUDE.md "no new clap
/// dep" rule.
pub(crate) fn parse_bench_arc_args(rest: &[String]) -> Result<CliAction, String> {
    let mut dataset: Option<String> = None;
    let mut output: Option<String> = None;
    let mut novelty_config: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--dataset" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --dataset".to_string())?;
                dataset = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--dataset=") => {
                dataset = Some(flag["--dataset=".len()..].to_string());
                i += 1;
            }
            "--output" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --output".to_string())?;
                output = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--output=") => {
                output = Some(flag["--output=".len()..].to_string());
                i += 1;
            }
            "--novelty-config" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --novelty-config".to_string())?;
                novelty_config = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--novelty-config=") => {
                novelty_config = Some(flag["--novelty-config=".len()..].to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown `bench arc` argument: {other}"));
            }
        }
    }
    Ok(CliAction::BenchArc {
        dataset,
        output,
        novelty_config,
    })
}

/// E-17 SP6 — parse `elai bench rag --tier <STANDARD|DEEP> --top-k <N>
/// [--project-root PATH] [--json] -- <QUERY...>`. Hand-rolled per the
/// CLAUDE.md "no new clap dep" rule. The query is taken from positional
/// arguments after the `--` sentinel (joined with spaces); flags before
/// `--` are parsed as named options. `--tier` is required; everything
/// else has a default (top-k=10, json=false, project-root=cwd).
pub(crate) fn parse_bench_rag_args(rest: &[String]) -> Result<CliAction, String> {
    let mut tier: Option<String> = None;
    let mut top_k: u32 = 10;
    let mut project_root: Option<PathBuf> = None;
    let mut json: bool = false;
    let mut query_parts: Vec<String> = Vec::new();
    let mut i = 0;
    let mut saw_sentinel = false;
    while i < rest.len() {
        let arg = rest[i].as_str();
        if saw_sentinel {
            query_parts.push(rest[i].clone());
            i += 1;
            continue;
        }
        match arg {
            "--" => {
                saw_sentinel = true;
                i += 1;
            }
            "--tier" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --tier".to_string())?;
                tier = Some(value.to_uppercase());
                i += 2;
            }
            flag if flag.starts_with("--tier=") => {
                tier = Some(flag["--tier=".len()..].to_uppercase());
                i += 1;
            }
            "--top-k" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --top-k".to_string())?;
                top_k = value
                    .parse()
                    .map_err(|e| format!("invalid --top-k {value:?}: {e}"))?;
                i += 2;
            }
            flag if flag.starts_with("--top-k=") => {
                let value = &flag["--top-k=".len()..];
                top_k = value
                    .parse()
                    .map_err(|e| format!("invalid --top-k {value:?}: {e}"))?;
                i += 1;
            }
            "--project-root" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --project-root".to_string())?;
                project_root = Some(PathBuf::from(value));
                i += 2;
            }
            flag if flag.starts_with("--project-root=") => {
                project_root = Some(PathBuf::from(&flag["--project-root=".len()..]));
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                // Pre-sentinel positional: treat as the start of an
                // unsentineled query. Permissive grammar: `elai bench rag
                // --tier STANDARD --top-k 5 my query here` is equivalent
                // to `... -- my query here` once we hit a non-flag arg.
                if other.starts_with("--") {
                    return Err(format!("unknown `bench rag` argument: {other}"));
                }
                saw_sentinel = true;
                query_parts.push(rest[i].clone());
                i += 1;
            }
        }
    }
    let tier_value = tier
        .ok_or_else(|| "missing required --tier (STANDARD or DEEP)".to_string())?;
    if tier_value != "STANDARD" && tier_value != "DEEP" && tier_value != "FAST" {
        return Err(format!(
            "invalid --tier {tier_value:?} (expected STANDARD | DEEP | FAST)"
        ));
    }
    let query = query_parts.join(" ").trim().to_string();
    if query.is_empty() {
        return Err("missing query (pass after `--` or as trailing positional args)".to_string());
    }
    if top_k == 0 {
        return Err("--top-k must be >= 1".to_string());
    }
    let resolved_project_root = match project_root {
        Some(p) => p,
        None => std::env::current_dir()
            .map_err(|e| format!("cannot determine cwd for --project-root: {e}"))?,
    };
    Ok(CliAction::BenchRag {
        tier: tier_value,
        top_k,
        query,
        json,
        project_root: resolved_project_root,
    })
}

/// C-13 — parse `elai wip-squash [--project-root <path>]` arguments.
/// Same `--project-root` resolution scheme as `parse_symbol_graph_args`.
pub(crate) fn parse_wip_squash_args(rest: &[String]) -> Result<CliAction, String> {
    let mut project_root: Option<PathBuf> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--project-root" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --project-root".to_string())?;
                project_root = Some(PathBuf::from(value));
                i += 2;
            }
            flag if flag.starts_with("--project-root=") => {
                project_root = Some(PathBuf::from(&flag[15..]));
                i += 1;
            }
            other => {
                return Err(format!("unknown `wip-squash` argument: {other}"));
            }
        }
    }
    let project_root = match project_root {
        Some(p) => p,
        None => std::env::current_dir().map_err(|e| {
            format!("`wip-squash` cannot determine cwd for --project-root default: {e}")
        })?,
    };
    Ok(CliAction::WipSquash { project_root })
}

pub(crate) fn parse_local_help_action(rest: &[String]) -> Option<Result<CliAction, String>> {
    if rest.len() != 2 || !is_help_flag(&rest[1]) {
        return None;
    }

    let topic = match rest[0].as_str() {
        "status" => LocalHelpTopic::Status,
        "sandbox" => LocalHelpTopic::Sandbox,
        "doctor" => LocalHelpTopic::Doctor,
        _ => return None,
    };
    Some(Ok(CliAction::HelpTopic(topic)))
}

pub(crate) fn is_help_flag(value: &str) -> bool {
    matches!(value, "--help" | "-h")
}

pub(crate) fn parse_single_word_command_alias(
    rest: &[String],
    model: &str,
    permission_mode_override: Option<PermissionMode>,
    output_format: CliOutputFormat,
) -> Option<Result<CliAction, String>> {
    if rest.len() != 1 {
        return None;
    }

    match rest[0].as_str() {
        "help" => Some(Ok(CliAction::Help { output_format })),
        "version" => Some(Ok(CliAction::Version { output_format })),
        "status" => Some(Ok(CliAction::Status {
            model: model.to_string(),
            permission_mode: permission_mode_override.unwrap_or_else(default_permission_mode),
            output_format,
        })),
        "sandbox" => Some(Ok(CliAction::Sandbox { output_format })),
        "doctor" => Some(Ok(CliAction::Doctor { output_format })),
        other => bare_slash_command_guidance(other).map(Err),
    }
}

pub(crate) fn bare_slash_command_guidance(command_name: &str) -> Option<String> {
    if matches!(
        command_name,
        "dump-manifests"
            | "bootstrap-plan"
            | "agents"
            | "mcp"
            | "skills"
            | "system-prompt"
            | "dep-audit"
            | "login"
            | "logout"
            | "init"
            | "prompt"
    ) {
        return None;
    }
    let slash_command = slash_command_specs()
        .iter()
        .find(|spec| spec.name == command_name)?;
    let guidance = if slash_command.resume_supported {
        format!(
            "`elai {command_name}` is a slash command. Use `elai --resume SESSION.jsonl /{command_name}` or start `elai` and run `/{command_name}`."
        )
    } else {
        format!(
            "`elai {command_name}` is a slash command. Start `elai` and run `/{command_name}` inside the REPL."
        )
    };
    Some(guidance)
}

pub(crate) fn join_optional_args(args: &[String]) -> Option<String> {
    let joined = args.join(" ");
    let trimmed = joined.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

pub(crate) fn parse_direct_slash_cli_action(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let raw = rest.join(" ");
    match SlashCommand::parse(&raw) {
        Ok(Some(SlashCommand::Help)) => Ok(CliAction::Help { output_format }),
        Ok(Some(SlashCommand::Agents { args })) => Ok(CliAction::Agents {
            args,
            output_format,
        }),
        Ok(Some(SlashCommand::Mcp { action, target })) => Ok(CliAction::Mcp {
            args: match (action, target) {
                (None, None) => None,
                (Some(action), None) => Some(action),
                (Some(action), Some(target)) => Some(format!("{action} {target}")),
                (None, Some(target)) => Some(target),
            },
            output_format,
        }),
        Ok(Some(SlashCommand::Runtime {
            action,
            role,
            target,
        })) => Ok(CliAction::Runtime {
            action,
            role,
            target,
            output_format,
        }),
        Ok(Some(SlashCommand::Measure {
            scope,
            cloud,
            output_format: _of,
        })) => Ok(CliAction::Measure {
            scope,
            cloud,
            output_format,
        }),
        // C-25: route /pipeline through the same direct-slash path so
        // `elai /pipeline list` works from the shell.
        Ok(Some(SlashCommand::Pipeline { action })) => Ok(CliAction::Pipeline {
            action,
            output_format,
        }),
        // C-27: route /wiring through the same direct-slash path so
        // `elai /wiring audit` works from the shell.
        Ok(Some(SlashCommand::WiringAudit {
            scope,
            config_path,
            contracts_dir,
        })) => Ok(CliAction::WiringAudit {
            scope,
            config_path,
            contracts_dir,
            output_format,
        }),
        Ok(Some(SlashCommand::Skills { args })) => Ok(CliAction::Skills {
            args,
            output_format,
        }),
        Ok(Some(SlashCommand::Unknown(name))) => Err(format_unknown_direct_slash_command(&name)),
        Ok(Some(command)) => Err({
            let _ = command;
            format!(
                "slash command {command_name} is interactive-only. Start `elai` and run it there, or use `elai --resume SESSION.jsonl {command_name}` / `elai --resume {latest} {command_name}` when the command is marked [resume] in /help.",
                command_name = rest[0],
                latest = LATEST_SESSION_REFERENCE,
            )
        }),
        Ok(None) => Err(format!("unknown subcommand: {}", rest[0])),
        Err(error) => Err(error.to_string()),
    }
}

pub(crate) fn format_unknown_option(option: &str) -> String {
    let mut message = format!("unknown option: {option}");
    if let Some(suggestion) = suggest_closest_term(option, CLI_OPTION_SUGGESTIONS) {
        message.push_str("\nDid you mean ");
        message.push_str(suggestion);
        message.push('?');
    }
    message.push_str("\nRun `elai--help` for usage.");
    message
}

pub(crate) fn format_unknown_direct_slash_command(name: &str) -> String {
    let mut message = format!("unknown slash command outside the REPL: /{name}");
    if let Some(suggestions) = render_suggestion_line("Did you mean", &suggest_slash_commands(name))
    {
        message.push('\n');
        message.push_str(&suggestions);
    }
    message.push_str("\nRun `elai--help` for CLI usage, or start `elai` and use /help.");
    message
}

pub(crate) fn format_unknown_slash_command(name: &str) -> String {
    let mut message = format!("Unknown slash command: /{name}");
    if let Some(suggestions) = render_suggestion_line("Did you mean", &suggest_slash_commands(name))
    {
        message.push('\n');
        message.push_str(&suggestions);
    }
    message.push_str("\n  Help             /help lists available slash commands");
    message
}

pub(crate) fn render_suggestion_line(label: &str, suggestions: &[String]) -> Option<String> {
    (!suggestions.is_empty()).then(|| format!("  {label:<16} {}", suggestions.join(", "),))
}

pub(crate) fn suggest_slash_commands(input: &str) -> Vec<String> {
    let mut candidates = slash_command_specs()
        .iter()
        .flat_map(|spec| {
            std::iter::once(spec.name)
                .chain(spec.aliases.iter().copied())
                .map(|name| format!("/{name}"))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.dedup();
    let candidate_refs = candidates.iter().map(String::as_str).collect::<Vec<_>>();
    ranked_suggestions(input.trim_start_matches('/'), &candidate_refs)
        .into_iter()
        .map(str::to_string)
        .collect()
}

pub(crate) fn suggest_closest_term<'a>(input: &str, candidates: &'a [&'a str]) -> Option<&'a str> {
    ranked_suggestions(input, candidates).into_iter().next()
}

pub(crate) fn ranked_suggestions<'a>(input: &str, candidates: &'a [&'a str]) -> Vec<&'a str> {
    let normalized_input = input.trim_start_matches('/').to_ascii_lowercase();
    let mut ranked = candidates
        .iter()
        .filter_map(|candidate| {
            let normalized_candidate = candidate.trim_start_matches('/').to_ascii_lowercase();
            let distance = levenshtein_distance(&normalized_input, &normalized_candidate);
            let prefix_bonus = usize::from(
                !(normalized_candidate.starts_with(&normalized_input)
                    || normalized_input.starts_with(&normalized_candidate)),
            );
            let score = distance + prefix_bonus;
            (score <= 4).then_some((score, *candidate))
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| left.cmp(right).then_with(|| left.1.cmp(right.1)));
    ranked
        .into_iter()
        .map(|(_, candidate)| candidate)
        .take(3)
        .collect()
}

pub(crate) fn levenshtein_distance(left: &str, right: &str) -> usize {
    if left.is_empty() {
        return right.chars().count();
    }
    if right.is_empty() {
        return left.chars().count();
    }

    let right_chars = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right_chars.len()).collect::<Vec<_>>();
    let mut current = vec![0; right_chars.len() + 1];

    for (left_index, left_char) in left.chars().enumerate() {
        current[0] = left_index + 1;
        for (right_index, right_char) in right_chars.iter().enumerate() {
            let substitution_cost = usize::from(left_char != *right_char);
            current[right_index + 1] = (previous[right_index + 1] + 1)
                .min(current[right_index] + 1)
                .min(previous[right_index] + substitution_cost);
        }
        previous.clone_from(&current);
    }

    previous[right_chars.len()]
}

pub(crate) fn resolve_model_alias(model: &str) -> &str {
    match model {
        "opus" => "claude-opus-4-6",
        "sonnet" => "claude-sonnet-4-6",
        "haiku" => "claude-haiku-4-5-20251213",
        _ => model,
    }
}

pub(crate) fn normalize_allowed_tools(values: &[String]) -> Result<Option<AllowedToolSet>, String> {
    if values.is_empty() {
        return Ok(None);
    }
    current_tool_registry()?.normalize_allowed_tools(values)
}

pub(crate) fn current_tool_registry() -> Result<GlobalToolRegistry, String> {
    let cwd = env::current_dir().map_err(|error| error.to_string())?;
    let loader = ConfigLoader::default_for(&cwd);
    let runtime_config = loader.load().map_err(|error| error.to_string())?;
    let state = build_runtime_plugin_state_with_loader(&cwd, &loader, &runtime_config)
        .map_err(|error| error.to_string())?;
    let registry = state.tool_registry.clone();
    if let Some(mcp_state) = state.mcp_state {
        mcp_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .shutdown()
            .map_err(|error| error.to_string())?;
    }
    Ok(registry)
}

pub(crate) fn parse_permission_mode_arg(value: &str) -> Result<PermissionMode, String> {
    normalize_permission_mode(value)
        .ok_or_else(|| {
            format!(
                "unsupported permission mode '{value}'. Use read-only, workspace-write, or danger-full-access."
            )
        })
        .map(permission_mode_from_label)
}

pub(crate) fn permission_mode_from_label(mode: &str) -> PermissionMode {
    match mode {
        "read-only" => PermissionMode::ReadOnly,
        "workspace-write" => PermissionMode::WorkspaceWrite,
        "danger-full-access" => PermissionMode::DangerFullAccess,
        other => panic!("unsupported permission mode label: {other}"),
    }
}

pub(crate) fn permission_mode_from_resolved(mode: ResolvedPermissionMode) -> PermissionMode {
    match mode {
        ResolvedPermissionMode::ReadOnly => PermissionMode::ReadOnly,
        ResolvedPermissionMode::WorkspaceWrite => PermissionMode::WorkspaceWrite,
        ResolvedPermissionMode::DangerFullAccess => PermissionMode::DangerFullAccess,
    }
}

pub(crate) fn default_permission_mode() -> PermissionMode {
    env::var("ELAI_PERMISSION_MODE")
        .ok()
        .as_deref()
        .and_then(normalize_permission_mode)
        .map(permission_mode_from_label)
        .or_else(config_permission_mode_for_current_dir)
        .unwrap_or(PermissionMode::DangerFullAccess)
}

pub(crate) fn config_permission_mode_for_current_dir() -> Option<PermissionMode> {
    let cwd = env::current_dir().ok()?;
    let loader = ConfigLoader::default_for(&cwd);
    loader
        .load()
        .ok()?
        .permission_mode()
        .map(permission_mode_from_resolved)
}

/// 1.2 — Tool names that must never appear in the model-visible tool list.
/// These are agent-runtime, testing, and MCP-bridge internals that the
/// assistant model should not call directly; exposing them wastes ~15K tokens
/// of context on the local sidecar and can trigger unintended agent spawns.
pub(crate) const MODEL_HIDDEN_TOOL_NAMES: &[&str] = &[
    "TestingPermission",
    "RunTaskPacket",
    "WorkerCreate",
    "WorkerGet",
    "WorkerObserve",
    "WorkerResolveTrust",
    "WorkerAwaitReady",
    "WorkerSendPrompt",
    "WorkerRestart",
    "WorkerTerminate",
    "TeamCreate",
    "TeamDelete",
    "CronCreate",
    "CronDelete",
    "CronList",
    "RemoteTrigger",
    "McpAuth",
    "MCP",
    "SendUserMessage",
    "ListMcpResources",
    "ReadMcpResource",
    "NotebookEdit",
    "ToolSearch",
];

pub(crate) fn filter_tool_specs(
    tool_registry: &GlobalToolRegistry,
    allowed_tools: Option<&AllowedToolSet>,
) -> Vec<ToolDefinition> {
    tool_registry
        .definitions(allowed_tools)
        .into_iter()
        .filter(|def| {
            !MODEL_HIDDEN_TOOL_NAMES
                .iter()
                .any(|hidden| hidden.eq_ignore_ascii_case(def.name.as_str()))
        })
        .collect()
}

pub(crate) fn parse_system_prompt_args(
    args: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut cwd = env::current_dir().map_err(|error| error.to_string())?;
    let mut date = DEFAULT_DATE.to_string();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--cwd" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --cwd".to_string())?;
                cwd = PathBuf::from(value);
                index += 2;
            }
            "--date" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --date".to_string())?;
                date.clone_from(value);
                index += 2;
            }
            other => return Err(format!("unknown system-prompt option: {other}")),
        }
    }

    Ok(CliAction::PrintSystemPrompt {
        cwd,
        date,
        output_format,
    })
}

pub(crate) fn parse_resume_args(args: &[String], output_format: CliOutputFormat) -> Result<CliAction, String> {
    let (session_path, command_tokens): (PathBuf, &[String]) = match args.first() {
        None => (PathBuf::from(LATEST_SESSION_REFERENCE), &[]),
        Some(first) if looks_like_slash_command_token(first) => {
            (PathBuf::from(LATEST_SESSION_REFERENCE), args)
        }
        Some(first) => (PathBuf::from(first), &args[1..]),
    };
    let mut commands = Vec::new();
    let mut current_command = String::new();

    for token in command_tokens {
        if token.trim_start().starts_with('/') {
            if resume_command_can_absorb_token(&current_command, token) {
                current_command.push(' ');
                current_command.push_str(token);
                continue;
            }
            if !current_command.is_empty() {
                commands.push(current_command);
            }
            current_command = String::from(token.as_str());
            continue;
        }

        if current_command.is_empty() {
            return Err("--resume trailing arguments must be slash commands".to_string());
        }

        current_command.push(' ');
        current_command.push_str(token);
    }

    if !current_command.is_empty() {
        commands.push(current_command);
    }

    Ok(CliAction::ResumeSession {
        session_path,
        commands,
        output_format,
    })
}

pub(crate) fn resume_command_can_absorb_token(current_command: &str, token: &str) -> bool {
    matches!(
        SlashCommand::parse(current_command),
        Ok(Some(SlashCommand::Export { path: None, jsonl: _ }))
    ) && !looks_like_slash_command_token(token)
}

pub(crate) fn looks_like_slash_command_token(token: &str) -> bool {
    let trimmed = token.trim_start();
    let Some(name) = trimmed.strip_prefix('/').and_then(|value| {
        value
            .split_whitespace()
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
    }) else {
        return false;
    };

    slash_command_specs()
        .iter()
        .any(|spec| spec.name == name || spec.aliases.contains(&name))
}

/// E-30 — parse `elai incident record [--title T] [--date D] [--url U] [--id I]`.
///
/// Stores an incident postmortem entry in the SSB and emits
/// SsbActionType::IncidentRecordCreated. The `record` sub-verb is mandatory
/// (leaves room for `list` / `search` in future E-plan phases).
pub(crate) fn parse_incident_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let Some(sub) = rest.first().map(String::as_str) else {
        return Err("incident: expected sub-command 'record'".to_string());
    };
    if sub != "record" {
        return Err(format!(
            "unknown `incident` sub-command: '{sub}' (expected 'record')"
        ));
    }
    let rest = &rest[1..];
    let mut title: Option<String> = None;
    let mut incident_date: Option<String> = None;
    let mut source_url: Option<String> = None;
    let mut incident_id: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--title" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --title".to_string())?;
                title = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--title=") => {
                title = Some(flag["--title=".len()..].to_string());
                i += 1;
            }
            "--date" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --date".to_string())?;
                incident_date = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--date=") => {
                incident_date = Some(flag["--date=".len()..].to_string());
                i += 1;
            }
            "--url" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --url".to_string())?;
                source_url = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--url=") => {
                source_url = Some(flag["--url=".len()..].to_string());
                i += 1;
            }
            "--id" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --id".to_string())?;
                incident_id = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--id=") => {
                incident_id = Some(flag["--id=".len()..].to_string());
                i += 1;
            }
            other => return Err(format!("unknown `incident record` argument: {other}")),
        }
    }
    let title = title.ok_or_else(|| "`incident record` requires --title".to_string())?;
    let incident_date =
        incident_date.ok_or_else(|| "`incident record` requires --date".to_string())?;
    let source_url =
        source_url.ok_or_else(|| "`incident record` requires --url".to_string())?;
    Ok(CliAction::IncidentRecord {
        title,
        incident_date,
        source_url,
        incident_id,
    })
}
