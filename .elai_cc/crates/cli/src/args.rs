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
        /// B-25 Phase 4J -- opt-in to constructing the EmpireBridge so the
        /// dashboard EmpireBridgePanel can reflect live factory-brain
        /// state. Requires the orchestrator crate to be compiled with
        /// `--features empire`; without the feature this flag is a no-op
        /// and a warn-level tracing message is logged at runtime.
        empire: bool,
        /// C-39 -- when set via `--export-ssb <path>`, the runner writes a
        /// JSONL snapshot of the broadcast SSB rows for this run to the
        /// given path at process exit. Consumed by the CI parity script
        /// to compare in-process vs via-mcp dispatch sequences.
        export_ssb: Option<PathBuf>,
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
    /// C-43: `elai freeze status|verify|ratify --ratifier <id>` — Phase 4H
    /// scope-freeze CLI surface. `status` prints the parsed
    /// `<workspace>/scope-freeze.toml` manifest, `verify` walks every
    /// frozen_items entry against on-disk mtimes (exits non-zero on any
    /// violation), `ratify` writes a fresh manifest covering the default
    /// Phase 4H frozen set and emits a ScopeFreezeRatified SSB row.
    /// Driver at crates/cli/src/freeze_cmd.rs. ELAI design hypothesis —
    /// no paper grounding; see workflow/sessions/s-25120/mistakes.md
    /// for the bad-cite analysis of arXiv:2604.21725.
    Freeze {
        action: FreezeAction,
        output_format: CliOutputFormat,
    },
    /// C-45: `elai credit <attribute|table|export>` — contrastive
    /// credit-attribution surface driven by
    /// `crate::credit_cmd`. Paper-grounded against arXiv:2605.13295
    /// (Zehle 2026, CANTANTE Section 3.2). `attribute` runs the
    /// harness, `table` re-renders the most recent table, `export`
    /// writes the per-role JSON. The synthetic-fixture path is
    /// deterministic + GPU-free; the live runpod pilot is flagged
    /// `RUNPOD-PENDING: cantante_pilot_450runs_qwen3_8b`.
    Credit {
        action: CreditAction,
        output_format: CliOutputFormat,
    },
    /// C-46: `elai routing <decisions|pilot|status|export>` —
    /// Uno-Orchestra selective-delegation routing surface driven by
    /// `crate::routing_cmd`. Paper-grounded against arXiv:2605.05007
    /// (Cui et al 2026, Sections 5 + 6.1). `decisions` lists recent
    /// SelectiveDelegationDecision rows, `pilot` runs the
    /// difficulty-classifier pilot harness, `status` prints the
    /// active routing policy, `export` writes the pilot table to a
    /// JSON file. Live runpod pilot is flagged
    /// `RUNPOD-PENDING: uno_orchestra_pilot_8roles_qwen3_8b`.
    Routing {
        action: RoutingAction,
        output_format: CliOutputFormat,
    },
    /// C-47: `elai calibration <status|reset-hints|export>` —
    /// case-based tool-use calibration surface driven by
    /// `crate::calibration_cmd`. Paper-grounded against
    /// arXiv:2605.15041 (CAST). `status` renders per-tool case-base
    /// stats; `reset-hints` exercises hint suppression on a fresh
    /// registry; `export` writes the synthetic snapshot to JSON for
    /// archival. The synthetic-fixture path is deterministic + GPU-
    /// free; the live registry is queried only when the production
    /// task-runner has wired one (`RUNPOD-PENDING: live registry IPC
    /// seam`).
    Calibration {
        action: CalibrationAction,
        output_format: CliOutputFormat,
    },
    /// E-14: `elai browser <run|list-journeys|status>` — browser-e2e
    /// agent surface driven by `crate::browser_e2e_cmd`. `run` invokes
    /// the named journey via Playwright (wire-dark when `npx`/`playwright`
    /// absent — see `crates/orchestrator/src/browser_e2e/`).
    /// `list-journeys` enumerates `.elai_cc/journeys/*.toml`.
    /// `status` prints recent SSB rows from the `browser_e2e_*` action
    /// family. Live Playwright + VLM tier are RUNPOD-PENDING; the
    /// synthetic-fixture path is deterministic + GPU-free.
    Browser {
        action: BrowserAction,
        output_format: CliOutputFormat,
    },
    /// E-18: `elai reliability <status|list-spills|replay-spill>` —
    /// tool-use reliability gate surface driven by
    /// `crate::reliability_cmd`. `status` prints the current gate
    /// configuration + recent SSB row counts (Pass / Reject / Spill /
    /// Escalate). `list-spills` enumerates the
    /// `.elai_cc/.elai_cc/spill/*.json` overflow store.
    /// `replay-spill --path PATH` reads one spilled body back so the
    /// operator can verify a downstream tool consumed the right
    /// payload. Reuse-heavy posture: composes on D-11 RegistryCheck +
    /// E-35 SearchReplaceMatcher / SidecarXGrammarHook + C-47
    /// ToolCalibrationRegistry + C-42 AuthzPolicy + C-35 ToolGate
    /// rather than re-implementing those surfaces. See
    /// `workflow/reasoning/E-18-decisions.md`.
    Reliability {
        action: ReliabilityAction,
        output_format: CliOutputFormat,
    },
    /// E-44 — `elai durability <sub>` — durability layer operator
    /// surface. Anchors the orchestrator's `durability::*` module
    /// as production callable so the new IdempotentDispatcher +
    /// CompactLock + checkpoint_migrator + continue_as_new wrappers
    /// + resume_validator are not WIRE-DARK per E-41
    /// WIRE-DARK-AVOIDANCE-VIA-CLI precedent. See
    /// `workflow/reasoning/E-44-decisions.md`.
    Durability {
        action: DurabilityAction,
        output_format: CliOutputFormat,
    },
    /// E-45 — `elai drift <sub>` — cross-language drift detector
    /// operator surface. Anchors the orchestrator's
    /// `cross_lang_drift::*` module as production callable so
    /// `DriftDetector` + `PairRegistry` are not WIRE-DARK per E-41
    /// precedent (E40_WIRE_DARK_AVOIDANCE_VIA_CLI). Handler at
    /// crates/cli/src/drift_cmd.rs. See
    /// `workflow/reasoning/E-45-decisions.md`.
    Drift {
        action: DriftAction,
        output_format: CliOutputFormat,
    },
    /// E-49 — `elai metrics <sub>` — coding-agent metric suite operator
    /// surface (scope-lean ship). Anchors `coding_metrics::*` as
    /// production callable so MetricsCollector + aggregator + the
    /// SWE-Bench Pro / RefineBench live-run boundaries (RUNPOD-PENDING)
    /// are not WIRE-DARK per E-41 precedent. Handler at
    /// crates/cli/src/metrics_cmd.rs. See
    /// `workflow/reasoning/E-49-decisions.md`.
    Metrics {
        action: MetricsAction,
        output_format: CliOutputFormat,
    },
    /// D-16: `elai conformal-calibration <status|update>` — durable
    /// UniCR tau threshold surface driven by
    /// `crate::conformal_calibration_cli`. Paper-grounded against
    /// arXiv:2509.01455 (UniCR Eq.5). The namespace is DISTINCT from
    /// C-47's `elai calibration` (which owns the case-based tool-use
    /// calibration registry); D-16-decisions.md DECISION-5 documents
    /// the disambiguation.
    ///
    /// Sub-actions:
    ///   * `status` — prints the current tau scalar, corpus size,
    ///     and last-update wall-clock from the configured
    ///     `[conformal].tau_path` (default
    ///     `.elai_cc/data/conformal_tau.txt`).
    ///   * `update` — invokes `TauUpdater::update_tau`, persists
    ///     the new scalar atomically, and emits one
    ///     `calibration_corpus_updated` SSB row.
    ConformalCalibration {
        action: ConformalCalibrationAction,
        output_format: CliOutputFormat,
    },
    /// C-49: `elai experience <distill|list|export|reset>` —
    /// iterative-distillation experience surface driven by
    /// `crate::experience_cmd`. ELAI heuristic adaptation of
    /// arXiv:2605.14443 (Sayana, Todi, Jash 2026). The synthetic-
    /// fixture path is deterministic + GPU-free; the live trace
    /// recorder is `RUNPOD-PENDING: iterative_distillation_prompter_rl_qwen3_8b`.
    Experience {
        action: ExperienceAction,
        output_format: CliOutputFormat,
    },
    /// C-50: `elai contract <scaffold|audit|list>` — Contract-Coding
    /// STRUCTURAL contract test scaffolding surface driven by
    /// `crate::contract_cmd`. Paper-grounded against arXiv:2604.13100
    /// (Lin, Zhao, Shi 2026 — kernel K = <N, Sigma, Delta> §4.2).
    /// `scaffold` emits a Rust contract test for a function name;
    /// `audit` runs the heuristic Discriminator against a code path
    /// + spec path; `list` prints the known target kernel anchors.
    /// Full Generator -> Discriminator RL pipeline (Algorithm 1) is
    /// `RUNPOD-PENDING: contract_coding_gen_disc_rl_qwen3_8b`.
    Contract {
        action: ContractAction,
        output_format: CliOutputFormat,
    },
    /// C-52: `elai dafny <verify|status|list-tiers>` — Dafny Formal
    /// Verification Spike CLI surface driven by `crate::dafny_cmd`.
    /// Paper-grounded against arXiv:2604.22601 (NL2VC). `verify`
    /// invokes the `dafny verify` binary on a `.dfy` spec (or
    /// surfaces the toolchain-not-found error); `status` reports
    /// `dafny --version` + ELAI substitute model selection; `list-
    /// tiers` prints the three paper tiers (Contextless / Signature
    /// / SelfHealing per RQ1/RQ2/RQ3). Live execution is
    /// `RUNPOD-PENDING: live_dafny_verification_with_non_gemma_substitution`
    /// — model substitution MANDATORY (Gemma 4 BLOCKED per memory
    /// feedback_no_gemma4; ELAI substitutes from Codestral-22B-v0.1
    /// / GPT-OSS-120B / Qwen 3 Coder 30B per paper eval set).
    Dafny {
        action: DafnyAction,
        output_format: CliOutputFormat,
    },
    /// E-13: `elai tdd <run|status|mutation-score>` — TDD verifier
    /// agent CLI surface driven by `crate::tdd_cmd`. Composes with
    /// the orchestrator's `tdd_verifier` module (Red/Green/Refactor
    /// cycle wrapper + anti-mock heuristic + cargo-mutants targeted
    /// subprocess) and emits the `tdd_cycle_started` /
    /// `tdd_test_failed` / `tdd_test_passed` SSB beacons.
    /// `run` drives one cycle for a task; `status` reports the
    /// most recent cycle outcome; `mutation-score` shells out to
    /// `cargo-mutants` on a single test path and prints the parsed
    /// `MutationOutcome`. Live `cargo-mutants` execution is
    /// RUNPOD-PENDING — the binary may be absent on the local dev
    /// host (wire-dark posture per E-12 Tier-B + C-52 dafny
    /// precedents).
    Tdd {
        action: TddAction,
        output_format: CliOutputFormat,
    },
    /// D-2: `elai ppm health [--endpoint URL] [--timeout SECS] [--json]`.
    /// Three-step health probe against the PPM sidecar (TCP probe of the
    /// configured port -> /health GET -> small /v1/ppm/score sanity round-
    /// trip). Mirrors `--remote-sidecar`'s probe pattern. Exits 0 on
    /// reachable + loaded, 1 on any failure. JSON output mirrors
    /// `archive verify --json` for CI consumption.
    PpmHealth {
        endpoint: Option<String>,
        timeout_secs: Option<u64>,
        json: bool,
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
    /// E-5 session-2: `elai memory <sub> [--project-root PATH]
    /// [--user-id ID] [--scope SCOPE] [--fact-type TYPE]
    /// [--fact-id ID] [--limit N] [--json]` — heuristic-only inspection
    /// surface over the per-project `memory_facts` sqlite store.
    /// Subcommands shipped this slice: `list`, `inspect`, `insert`,
    /// `delete`. Adjudication / promotion / Stage-2 classifier are
    /// DEFERRED to E-5-followup per workflow/reasoning/E-5-decisions.md.
    Memory {
        sub: MemorySubcommandKind,
        project_root: PathBuf,
        user_id: String,
        scope: Option<String>,
        fact_type: Option<String>,
        fact_id: Option<String>,
        key: Option<String>,
        value_json: Option<String>,
        limit: usize,
        json: bool,
    },
    /// C-40: `elai pr <open|list|status|close> [args...]` — GitHub PR
    /// adapter CLI surface. Shells out to the `gh` CLI via the
    /// orchestrator's `github_pr::GhCliAdapter`. Each subcommand maps
    /// to one trait method. Dispatcher lives in
    /// `crate::github_pr_cmd::run_github_pr_subcommand`; validation of
    /// the `sub` string + per-sub argument shape lives at the handler
    /// boundary (mirrors the `Mcp` / `Runtime` / `Mode` string-typed
    /// pattern so the args layer stays free of orchestrator types).
    Pr {
        /// One of `"open"`, `"list"`, `"status"`, `"close"`. Case-
        /// insensitive at parse time; validation lives in the handler.
        sub: String,
        /// `--repo owner/name`. None => gh infers from cwd's git remote.
        repo: Option<String>,
        /// `--title <text>` for `open`. Required for `open`; ignored
        /// elsewhere.
        title: Option<String>,
        /// `--body <text>` for `open`. Defaults to empty string.
        body: Option<String>,
        /// `--head <branch>` for `open`. Required for `open`.
        head: Option<String>,
        /// `--base <branch>` for `open`. Defaults to `main` at handler.
        base: Option<String>,
        /// `--draft` flag for `open`.
        draft: bool,
        /// Positional PR number for `status` / `close`. Required for
        /// those subverbs.
        pr_number: Option<u32>,
        /// `--state open|closed|merged|all` for `list`. None => default.
        state_filter: Option<String>,
        /// `--limit N` for `list`. 0 => handler-default (20).
        limit: u32,
        /// `--json` flag — emit machine-readable JSON to stdout.
        json: bool,
    },
    /// E-41: `elai pr review <compose-body|comment|watch|verify-deps|
    /// semgrep-gate|auto-merge-assess|apply-branch-protection|gen-codeowners|
    /// suggest-squash|detect-force-push>`. CLI surface for the E-41
    /// github_pr review extension modules. Every E-41 helper has at
    /// least one subverb here (WIRE-DARK avoidance). See
    /// `crate::pr_review_cmd::PrReviewAction` for the structured shape;
    /// the args layer carries a string + opaque key=value pairs so the
    /// orchestrator types stay out of the args crate.
    PrReview {
        /// One of `"compose-body" | "comment" | "watch" | "verify-deps"
        /// | "semgrep-gate" | "auto-merge-assess" |
        /// "apply-branch-protection" | "gen-codeowners" |
        /// "suggest-squash" | "detect-force-push"`. Case-insensitive at
        /// parse time; validation lives in the handler.
        sub: String,
        /// Positional PR number for review subverbs that need one.
        pr: Option<u32>,
        /// `--body <text>` for `comment` / `suggest-squash`.
        body: Option<String>,
        /// `--file <path>` for `comment`.
        file: Option<String>,
        /// `--line <n>` for `comment`.
        line: Option<u32>,
        /// `--template <text>` for `compose-body`.
        template: Option<String>,
        /// `--kv key=value` repeated for `compose-body`.
        kv_pairs: Vec<(String, String)>,
        /// `--since <ms>` for `watch`.
        since: Option<i64>,
        /// `--ruleset <id>` for `semgrep-gate` (default "auto").
        ruleset: Option<String>,
        /// `--loc-added <n>` for `auto-merge-assess`.
        loc_added: Option<u32>,
        /// `--loc-removed <n>` for `auto-merge-assess`.
        loc_removed: Option<u32>,
        /// `--files-changed <n>` for `auto-merge-assess`.
        files_changed: Option<u32>,
        /// `--touches-owned` for `auto-merge-assess`.
        touches_owned: bool,
        /// `--repo owner/name` for `apply-branch-protection` /
        /// `detect-force-push`.
        repo: Option<String>,
        /// `--branch <name>` for `apply-branch-protection` (default "main").
        branch: Option<String>,
        /// `--touched <path>` repeated for `gen-codeowners`.
        touched_files: Vec<String>,
        /// `--default-owner @handle` for `gen-codeowners`.
        default_owner: Option<String>,
        /// `--title <text>` for `suggest-squash`.
        title: Option<String>,
        /// `--json` to emit machine-readable output.
        json: bool,
    },
    /// E-37: `elai safety <status|list-pins|verify-pins|fixture-replay>`.
    /// Prompt-injection defense operator surface (OWASP LLM Top 10 LLM01).
    /// Read-only — no destructive sub-verbs. Reuses the safety crate's
    /// pin / classifier surface so no orchestrator deps are pulled into
    /// the CLI bin's compile graph beyond what already lands via
    /// `elai-orchestrator`. See `workflow/reasoning/E-37-decisions.md`.
    Safety {
        action: SafetyAction,
        output_format: CliOutputFormat,
    },
    /// E-40: `elai scip <defs|refs|callers|callees|pagerank|status|import>`.
    /// Orchestrator-rooted symbol-graph operator surface. Loads
    /// `.claude/tools/elai_graph.json` on dispatch + invokes the
    /// `elai_orchestrator::symbol_graph` query helpers / personalized
    /// PageRank / SCIP import wrapper. Read-only — no destructive
    /// sub-verbs. Defends against E37_WIRE_DARK_GAP_PATTERN by being
    /// the canonical production caller for the new module. See
    /// `workflow/reasoning/E-40-decisions.md`.
    SymbolGraph {
        action: SymbolGraphAction,
        output_format: CliOutputFormat,
    },
}

/// E-40 — `elai scip <sub>` sub-verb. Bundled with the parent
/// `SymbolGraph` variant so the dispatcher can match a single arm. Paper
/// grounding deferred per workflow/sessions/s-25120/mistakes.md cluster
/// #6 (Stack Graphs + GraphCoder + CodexGraph + RANGER + Glean Angle).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SymbolGraphAction {
    /// `elai scip query --symbol NAME --kind <defs|refs|callers|callees>`.
    Query { symbol: String, kind: String },
    /// `elai scip defs --symbol NAME` — shorthand for `query --kind defs`.
    Defs { symbol: String },
    /// `elai scip refs --symbol NAME` — shorthand for `query --kind refs`.
    Refs { symbol: String },
    /// `elai scip pagerank --seed NAME --k N` — personalized PageRank
    /// rooted on the seed.
    PageRank { seed: String, k: u32 },
    /// `elai scip status` — print graph-path + node count + LRU stats.
    Status,
    /// `elai scip import --dir <path>` — enumerate SCIP shards under
    /// `<path>` (default `.claude/tools/scip_indices/`).
    ImportScip { dir: String },
}

/// E-37 — `elai safety <sub>` sub-verb. Prompt-injection defense
/// operator surface. Bundled with the parent `Safety` variant so the
/// dispatcher can match a single arm. See
/// `workflow/reasoning/E-37-decisions.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SafetyAction {
    /// `elai safety status` — print recent SSB row counts across the 9
    /// E-37 `injection_*` / `spotlight_*` / `capability_gate_*` /
    /// `tool_description_pin_*` / `prompt_armor_*` / `provenance_*`
    /// action types. GPU-free synthetic snapshot when no live SSB is
    /// wired.
    Status,
    /// `elai safety list-pins` — enumerate the per-tool blake3
    /// description pins under `.elai_cc/.elai/mcp/tool_descriptions/`.
    /// Each row shows server, tool, pinned_hash prefix, pinned_at_ms.
    ListPins,
    /// `elai safety verify-pins` — re-walk the per-tool pin store and
    /// surface any `DescPinStatus::Drift` records WITHOUT auto-repinning
    /// (operator must explicitly approve via a follow-on `pin update`
    /// — deferred to follow-on patch-N).
    VerifyPins,
    /// `elai safety fixture-replay --fixture NAME` — replay the named
    /// adversarial fixture (`agentdojo` / `echoleak` / `cometjacking`)
    /// through the heuristic classifier + spotlight + capability gate
    /// stack and emit `injection_fixture_replayed` SSB row. Live
    /// model integration RUNPOD-PENDING.
    FixtureReplay { fixture: String },
}

/// E-5 session-2 — `elai memory <sub>` sub-verb. Bundled with the
/// parent `Memory` variant so the dispatcher can match a single arm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MemorySubcommandKind {
    /// `elai memory list` — paginated listing of currently-valid rows.
    List,
    /// `elai memory inspect --fact-id <id>` — full row + evidence_pointers.
    Inspect,
    /// `elai memory insert --fact-type <t> --key <k> --value-json <json>`
    /// — manual T0 user-direct insert (mirrors the dispatcher's ADD path
    /// with `Provenance::user_direct("cli")`).
    Insert,
    /// `elai memory delete --fact-id <id>` — soft-close (sets valid_to_ms).
    Delete,
}

/// C-43 — `elai freeze <sub>` sub-verb. Bundled with the parent
/// `Freeze` variant so the dispatcher can match a single arm. ELAI
/// design hypothesis — no paper grounding; see
/// workflow/sessions/s-25120/mistakes.md.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FreezeAction {
    /// `elai freeze status` — prints the parsed
    /// `<workspace>/scope-freeze.toml` manifest.
    Status,
    /// `elai freeze verify` — walks every `frozen_items` entry against
    /// on-disk mtimes and exits non-zero on any violation.
    Verify,
    /// `elai freeze ratify --ratifier <id>` — writes a fresh manifest
    /// covering the default Phase 4H frozen set and emits a
    /// ScopeFreezeRatified SSB row.
    Ratify { ratifier: String },
}

/// C-45 — `elai credit <sub>` sub-verb. Paper-grounded against
/// arXiv:2605.13295 (Zehle 2026). Bundled with the parent `Credit`
/// variant so the dispatcher can match a single arm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CreditAction {
    /// `elai credit attribute [--role <name>]` — run the contrastive
    /// credit-attribution harness (synthetic-fixture path). Without
    /// `--role`, ablates the 8-role pilot set; with `--role`, ablates a
    /// single role.
    Attribute { dropped_role: Option<String> },
    /// `elai credit table` — render the most-recent credit table
    /// (re-runs the synthetic-fixture harness so the surface is
    /// observable without state).
    Table,
    /// `elai credit export --path <PATH>` — write the credit table to a
    /// JSON file at `PATH`.
    Export { path: String },
}

/// C-46 — `elai routing <sub>` sub-verb. Paper-grounded against
/// arXiv:2605.05007 (Cui et al 2026). Bundled with the parent
/// `Routing` variant so the dispatcher can match a single arm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RoutingAction {
    /// `elai routing decisions [--since-ms <MS>]` — render the most
    /// recent SelectiveDelegationDecision rows. `--since-ms` filters
    /// rows whose `ts_ms` is greater than the supplied value (0 = no
    /// filter). Without the flag, the surface re-runs the pilot
    /// harness once so the CLI always has a verifiable observable
    /// effect even before live SSB rows exist.
    Decisions { since_ms: Option<u64> },
    /// `elai routing pilot --task <DESCRIPTION>` — run the difficulty
    /// classifier + selective-delegator pilot harness over the
    /// 8-base-role pilot scope and print the chosen `(role,
    /// primitive)` pairs. The synthetic-fixture path is deterministic
    /// + GPU-free; live runs are RUNPOD-PENDING.
    Pilot { task_template: String },
    /// `elai routing status` — print the resolved RoutingPolicy
    /// (enabled, alpha, s_max, max_turns, max_workers_per_query).
    Status,
    /// `elai routing export --path <PATH>` — write the pilot table to
    /// a JSON file at `PATH` for the uno-orchestra-pilot.md artifact.
    Export { path: String },
}

/// C-49 — `elai experience <sub>` sub-verb. ELAI heuristic adaptation
/// of arXiv:2605.14443 (Sayana, Todi, Jash 2026). Bundled with the
/// parent `Experience` variant so the dispatcher can match a single
/// arm. The full RL-trained prompter (Algorithm 1 — GRPO + KL anchor)
/// is RUNPOD-PENDING; this CLI surface only inspects + drives the
/// heuristic distiller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExperienceAction {
    /// `elai experience distill [--skill NAME]` — re-run the heuristic
    /// distiller over the in-memory synthetic corpus and print the
    /// emitted fragments. Without `--skill`, distills for every skill
    /// the synthetic harness populates. The synthetic-fixture path is
    /// deterministic + GPU-free; live runs are RUNPOD-PENDING.
    Distill { skill: Option<String> },
    /// `elai experience list` — list every skill with a non-empty
    /// trace bin (synthetic harness output).
    List,
    /// `elai experience export --path PATH` — write the per-skill
    /// distilled fragments to a JSON file at PATH.
    Export { path: String },
    /// `elai experience reset` — clear the in-memory synthetic corpus
    /// so the next distill call starts from a clean slate.
    Reset,
}

/// C-50 — `elai contract <sub>` sub-verb. Paper-grounded against
/// arXiv:2604.13100 (Lin, Zhao, Shi 2026 — Contract-Coding kernel
/// `K = <N, Sigma, Delta>` §4.2). Bundled with the parent
/// `Contract` variant so the dispatcher can match a single arm.
/// The full RL Generator -> Discriminator pipeline (Algorithm 1) is
/// `RUNPOD-PENDING`; this CLI surface drives the heuristic
/// scaffolder + Discriminator only. Citation discipline: the
/// headline "47%" is the Roguelike subset of Greenfield-5 (Table 1)
/// and TIES Trae; the differentiator is structural integrity per
/// Section 5.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ContractAction {
    /// `elai contract scaffold --fn-name NAME [--output PATH]` —
    /// emit a Rust contract test scaffolding for the given function
    /// name. The signature is synthesized from the function name
    /// (no signature parser yet); the operator hand-edits the
    /// emitted file before locking it.
    Scaffold {
        /// Canonical function name. Required; the scaffolder
        /// refuses empty / whitespace input.
        fn_name: String,
        /// Optional output path; when `None`, the scaffolded body
        /// is printed to stdout.
        output: Option<String>,
    },
    /// `elai contract audit --code-path PATH --spec-path PATH` —
    /// run the heuristic Discriminator against the code body at
    /// `--code-path` using the kernel at `--spec-path`. Spec file
    /// is `serde_json` (deterministic round-trip).
    Audit {
        /// Path to the code body to audit.
        code_path: String,
        /// Path to the JSON-encoded `StructuralContract` spec.
        spec_path: String,
    },
    /// `elai contract list` — emit a deterministic sample of the
    /// canonical kernel anchors the scaffolder emits (N/Sigma/Delta
    /// names) for documentation / smoke purposes.
    List,
}

/// C-52 — `elai dafny <sub>` sub-verb. Paper-grounded against
/// arXiv:2604.22601 (NL2VC). Bundled with the parent `Dafny` variant
/// so the dispatcher can match a single arm. Live `dafny verify`
/// execution is `RUNPOD-PENDING: live_dafny_verification_with_non_gemma_substitution`
/// — the local dev host lacks the `dafny` binary on PATH per the
/// C-52 cross-check report. Model substitution is MANDATORY: Gemma
/// 4 is BLOCKED per memory rule `feedback_no_gemma4`; ELAI
/// substitutes from the paper's evaluation set: Codestral-22B-v0.1
/// / GPT-OSS-120B / Qwen 3 Coder 30B.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DafnyAction {
    /// `elai dafny verify --spec-path PATH [--tier signature]
    /// [--max-iter 10]` — shell out to `dafny verify <spec>` and
    /// print the parsed result (dafny_pass / udebug_pass /
    /// heal_iter). Tier defaults to `signature` per paper Section 3
    /// (minimum viable tier; contextless near-universal-failure).
    Verify {
        spec_path: String,
        tier: Option<String>,
        max_iter: Option<u32>,
    },
    /// `elai dafny status` — report the local `dafny` binary
    /// availability + the current ELAI substitute model selection.
    /// Exits 0 even if the toolchain is missing (the absence is the
    /// canonical RUNPOD-PENDING signal).
    Status,
    /// `elai dafny list-tiers` — emit the canonical paper tier
    /// list (Contextless / Signature / SelfHealing per RQ1/RQ2/RQ3)
    /// so operators can see what `--tier` accepts.
    ListTiers,
}

/// E-13 — `elai tdd <sub>` sub-verb. Drives the orchestrator's
/// `tdd_verifier` module. Three sub-actions:
///   - `run --task-path PATH` — drive one Red/Green/Refactor cycle
///     for the task at PATH. Reads the failing test source from
///     the file, runs the agent's state machine, and prints the
///     `CycleResult` (test_flipped / anti_mock_clean / iterations).
///   - `status` — report `cargo-mutants --version` availability +
///     the orchestrator's TDD config defaults. Exits 0 even when
///     the toolchain is missing (the absence is the canonical
///     `RUNPOD-PENDING` signal).
///   - `mutation-score --test-path PATH` — shell out to
///     `cargo-mutants --file <PATH>` and print the parsed
///     `MutationOutcome`. Live execution is RUNPOD-PENDING when
///     the binary is absent.
///
/// Citation discipline (per workflow/sessions/s-25120/mistakes.md):
/// the E-13 plan cites 11 arXiv IDs; only LLMorpheus
/// (arXiv:2404.09952) and ThinkPRM (arXiv:2504.16828) remain
/// inline-cited in the agent docstrings. The other 9 are named
/// by author/title in workflow/reasoning/E-13-decisions.md only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TddAction {
    /// `elai tdd run --task-path PATH` — drive one TDD cycle.
    Run { task_path: String },
    /// `elai tdd status` — report toolchain availability +
    /// TDD config defaults.
    Status,
    /// `elai tdd mutation-score --test-path PATH` — run
    /// `cargo-mutants` on a single file and print the outcome.
    MutationScore { test_path: String },
}

/// E-14 — `elai browser <sub>` sub-verb. Bundled with the parent
/// `Browser` variant so the dispatcher can match a single arm.
/// Wire-dark posture for absent npx/playwright — see
/// `crates/orchestrator/src/browser_e2e/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BrowserAction {
    /// `elai browser run --journey <NAME>` — load
    /// `.elai_cc/journeys/<NAME>.toml` and invoke the Playwright runner.
    /// Returns the assertion pass/fail counts on stdout.
    Run { journey: String },
    /// `elai browser list-journeys` — print every TOML file in
    /// `.elai_cc/journeys/` with its parsed `name` + `url`.
    ListJourneys,
    /// `elai browser status` — print the most recent `browser_e2e_*`
    /// SSB rows (run started + assertion pass/fail + snapshot captured)
    /// so operators can verify wiring without waiting for the
    /// dashboard.
    Status,
}

/// E-18 — `elai reliability <sub>` sub-verb. Tool-use reliability gate
/// CLI surface. Bundled with the parent `Reliability` variant so the
/// dispatcher can match a single arm. See
/// `workflow/reasoning/E-18-decisions.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReliabilityAction {
    /// `elai reliability status` — print the current gate inline cap,
    /// configured retry budget, and recent SSB row counts across the 4
    /// `tool_reliability_*` / `tool_call_*` action types. GPU-free
    /// synthetic snapshot when no live SSB is wired.
    Status,
    /// `elai reliability list-spills [--dir DIR]` — enumerate the
    /// JSON spill files under `.elai_cc/.elai_cc/spill/` (or the
    /// supplied DIR). Each row shows filename, payload bytes, and
    /// blake3 digest.
    ListSpills { dir: Option<String> },
    /// `elai reliability replay-spill --path PATH` — read one spilled
    /// body back from PATH and print the JSON to stdout. Operator-side
    /// verification primitive.
    ReplaySpill { path: String },
}

/// C-47 — `elai calibration <sub>` sub-verb. Paper-grounded against
/// arXiv:2605.15041 (CAST). Bundled with the parent `Calibration`
/// variant so the dispatcher can match a single arm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CalibrationAction {
    /// `elai calibration status [--tool NAME]` — render per-tool
    /// case-base stats (shape_count, total_calls, error_count,
    /// last_ts_ms). `--tool` filters to one tool; otherwise prints
    /// every tool the synthetic registry observed.
    Status { tool_name: Option<String> },
    /// `elai calibration reset-hints` — toggle hint suppression on a
    /// freshly constructed registry. Useful for testing the gate.
    ResetHints,
    /// `elai calibration export --path PATH` — write the synthetic
    /// snapshot to a JSON file at PATH.
    Export { path: String },
}

/// D-16 — `elai conformal-calibration <sub>` sub-verb. Paper-grounded
/// against arXiv:2509.01455 (UniCR Eq.5). Bundled with the parent
/// `ConformalCalibration` variant so the dispatcher can match a single
/// arm. The namespace is DISTINCT from C-47's `elai calibration`
/// (case-based tool-use registry) per D-16-decisions.md DECISION-5.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConformalCalibrationAction {
    /// `elai conformal-calibration status [--tau-path PATH]
    /// [--corpus-path PATH]` — render current tau, corpus size, and
    /// the last-modified wall-clock of the tau file. Both flags
    /// default to the `[conformal]` toml values when omitted.
    Status {
        tau_path: Option<String>,
        corpus_path: Option<String>,
    },
    /// `elai conformal-calibration update [--tau-path PATH]
    /// [--corpus-path PATH] [--alpha F]` — invoke `TauUpdater` once,
    /// persist the new scalar atomically, and emit one
    /// `calibration_corpus_updated` SSB row. `--alpha` overrides the
    /// `[conformal].alpha` default (UniCR Eq.5 miscoverage rate).
    Update {
        tau_path: Option<String>,
        corpus_path: Option<String>,
        alpha: Option<String>,
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
        // C-43 — `elai freeze status|verify|ratify --ratifier <id>`.
        // Phase 4H scope-freeze CLI surface; handler at
        // crates/cli/src/freeze_cmd.rs. ELAI design hypothesis — no
        // paper grounding (see workflow/sessions/s-25120/mistakes.md).
        "freeze" => parse_freeze_subcommand_args(&rest[1..], output_format),
        // C-45 — `elai credit attribute|table|export`. CANTANTE contrastive
        // credit-attribution CLI surface; handler at
        // crates/cli/src/credit_cmd.rs. Paper-grounded against
        // arXiv:2605.13295 (Zehle 2026); see
        // workflow/papers/cantante-credit-attribution.md.
        "credit" => parse_credit_subcommand_args(&rest[1..], output_format),
        // C-46 — `elai routing decisions|pilot|status|export`. Uno-Orchestra
        // selective-delegation routing CLI surface; handler at
        // crates/cli/src/routing_cmd.rs. Paper-grounded against
        // arXiv:2605.05007 (Cui et al 2026); see
        // workflow/papers/uno-orchestra-interface.md.
        "routing" => parse_routing_subcommand_args(&rest[1..], output_format),
        // C-47 — `elai calibration status|reset-hints|export`. Case-based
        // tool-use calibration CLI surface; handler at
        // crates/cli/src/calibration_cmd.rs. Paper-grounded against
        // arXiv:2605.15041 (CAST); see
        // workflow/papers/case-based-calibration.md.
        "calibration" => parse_calibration_subcommand_args(&rest[1..], output_format),
        // E-14 — `elai browser run|list-journeys|status`. Browser-e2e
        // agent surface; handler at crates/cli/src/browser_e2e_cmd.rs.
        // Runner at crates/orchestrator/src/browser_e2e/. Wire-dark for
        // absent npx/playwright; live VLM tier RUNPOD-PENDING.
        "browser" => parse_browser_subcommand_args(&rest[1..], output_format),
        // E-18 — `elai reliability status|list-spills|replay-spill`.
        // Tool-use reliability gate CLI surface; handler at
        // crates/cli/src/reliability_cmd.rs. Composes on D-11 +
        // E-35 + C-47 + C-42 + C-35 (reuse-heavy posture). See
        // workflow/reasoning/E-18-decisions.md.
        "reliability" => parse_reliability_subcommand_args(&rest[1..], output_format),
        // E-44 — `elai durability status|validate-checkpoint|migrate|
        //                       continue-as-new-preview|acquire-lock`.
        // Durability layer operator surface; handler at
        // crates/cli/src/durability_cmd.rs. Anchors the durability/
        // module as production callable per E-44-decisions.md
        // WIRE-DARK-AVOIDANCE. See workflow/reasoning/E-44-decisions.md.
        "durability" => parse_durability_subcommand_args(&rest[1..], output_format),
        // E-45 — `elai drift detect|list-pairs|baseline`. Cross-language
        // drift detector operator surface; handler at
        // crates/cli/src/drift_cmd.rs. Anchors cross_lang_drift::* out
        // of WIRE-DARK status per E-41 precedent
        // (E40_WIRE_DARK_AVOIDANCE_VIA_CLI). See
        // workflow/reasoning/E-45-decisions.md.
        "drift" => parse_drift_subcommand_args(&rest[1..], output_format),
        // E-49 — `elai metrics capture|swe-bench-pro|refinebench|aggregate|status`.
        // Coding-agent metric suite operator surface (scope-lean ship);
        // handler at crates/cli/src/metrics_cmd.rs. WIRE-DARK avoidance
        // per E-41 precedent. See workflow/reasoning/E-49-decisions.md.
        "metrics" => parse_metrics_subcommand_args(&rest[1..], output_format),
        // E-37 — `elai safety status|list-pins|verify-pins|fixture-replay`.
        // Prompt-injection defense operator surface; handler at
        // crates/cli/src/safety_cmd.rs. Layers additively atop E-24 +
        // E-30. See workflow/reasoning/E-37-decisions.md.
        "safety" => parse_safety_subcommand_args(&rest[1..], output_format),
        "scip" => parse_scip_subcommand_args(&rest[1..], output_format),
        // D-16 — `elai conformal-calibration status|update`. Durable
        // UniCR tau persistence CLI surface; handler at
        // crates/cli/src/conformal_calibration_cli.rs. Paper-grounded
        // against arXiv:2509.01455 (UniCR Eq.5). DISTINCT namespace
        // from C-47 `elai calibration` per D-16-decisions.md
        // DECISION-5 (C-47 owns the case-based tool-use registry; D-16
        // owns the conformal-refusal calibration scalar).
        "conformal-calibration" => {
            parse_conformal_calibration_subcommand_args(&rest[1..], output_format)
        }
        // C-49 — `elai experience distill|list|export|reset`.
        // Iterative-distillation experience CLI surface; handler at
        // crates/cli/src/experience_cmd.rs. ELAI heuristic adaptation
        // of arXiv:2605.14443 (Sayana, Todi, Jash 2026); see
        // workflow/papers/iterative-distillation-interface.md.
        "experience" => parse_experience_subcommand_args(&rest[1..], output_format),
        // C-50 — `elai contract scaffold|audit|list`. Contract-Coding
        // STRUCTURAL contract test scaffolding CLI surface; handler
        // at crates/cli/src/contract_cmd.rs. Paper-grounded against
        // arXiv:2604.13100 (Lin, Zhao, Shi 2026 — kernel
        // K = <N, Sigma, Delta> §4.2); see
        // workflow/papers/contract-coding-interface.md.
        "contract" => parse_contract_subcommand_args(&rest[1..], output_format),
        // C-52 — `elai dafny verify|status|list-tiers`. Dafny Formal
        // Verification Spike CLI surface; handler at
        // crates/cli/src/dafny_cmd.rs. Paper-grounded against
        // arXiv:2604.22601 (NL2VC). Live execution is RUNPOD-PENDING
        // (model substitution mandatory — Gemma 4 BLOCKED per
        // memory feedback_no_gemma4).
        "dafny" => parse_dafny_subcommand_args(&rest[1..], output_format),
        // E-13 — `elai tdd run|status|mutation-score`. TDD verifier
        // agent CLI surface; handler at crates/cli/src/tdd_cmd.rs.
        // Composes with orchestrator::tdd_verifier (Red/Green/
        // Refactor cycle + anti-mock heuristic + cargo-mutants
        // wrapper). Live cargo-mutants execution is RUNPOD-PENDING
        // when the binary is absent (wire-dark per E-12 / C-52).
        "tdd" => parse_tdd_subcommand_args(&rest[1..], output_format),
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
        // C-40: `elai pr <open|list|status|close> [args...]` — GitHub PR
        // adapter CLI surface. Sub-action validation lives in the
        // handler (mirrors `Mcp` / `Runtime` string-typed pattern).
        "pr" => parse_pr_subcommand_args(&rest[1..]),
        // E-41: `elai pr-review <sub> [args...]` — PR review surface.
        // Distinct top-level subcommand so the args layer can stay
        // minimal; `pr` continues to dispatch the C-40 open/list/status/
        // close surface untouched.
        "pr-review" => parse_pr_review_subcommand_args(&rest[1..]),
        // D-9 — `elai audit-locked-keys` is a top-level subcommand, not a
        // slash command (CI + pre-commit hook want a simple argv shape).
        "audit-locked-keys" => parse_audit_locked_keys_args(&rest[1..]),
        // D-2 — `elai ppm health` 3-step PPM sidecar probe. Top-level
        // subcommand mirroring `archive verify` (CI / operator surface, not
        // a slash command). Sub-action is `health`; future subactions
        // (e.g. `ppm score-once`) can extend `parse_ppm_subcommand_args`.
        "ppm" => parse_ppm_subcommand_args(&rest[1..]),
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
        // E-5 session-2: `elai memory <sub>` — heuristic-only inspection
        // surface over the per-project memory_facts store.
        "memory" => parse_memory_subcommand_args(&rest[1..]),
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
    let mut empire: bool = false;
    let mut export_ssb: Option<PathBuf> = None;
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
            "--empire" => {
                empire = true;
                i += 1;
            }
            // C-39: --export-ssb <path> writes the run's SSB row stream to
            // a JSONL file at process exit. Used by the CI parity job.
            "--export-ssb" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --export-ssb".to_string())?;
                export_ssb = Some(PathBuf::from(value));
                i += 2;
            }
            flag if flag.starts_with("--export-ssb=") => {
                export_ssb = Some(PathBuf::from(&flag[13..]));
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
    Ok(CliAction::Run { project_root, mode, provider, via_mcp, empire, export_ssb })
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

/// C-40 — parse `elai pr <open|list|status|close> [args...]`.
///
/// Each subverb maps to one [`crate::github_pr_cmd`] handler entry.
/// Argument validation is intentionally permissive at parse time — the
/// handler boundary re-validates (mirrors `Mcp` / `Runtime` string-
/// typed pattern). Returns [`CliAction::Pr`] with the parsed shape.
pub(crate) fn parse_pr_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .map(String::as_str)
        .ok_or_else(|| {
            "missing `pr` sub-command (expected one of: open, list, status, close)".to_string()
        })?
        .to_ascii_lowercase();
    if !matches!(sub.as_str(), "open" | "list" | "status" | "close") {
        return Err(format!(
            "unknown `pr` sub-command: '{sub}' (expected one of: open, list, status, close)"
        ));
    }

    let mut repo: Option<String> = None;
    let mut title: Option<String> = None;
    let mut body: Option<String> = None;
    let mut head: Option<String> = None;
    let mut base: Option<String> = None;
    let mut draft = false;
    let mut pr_number: Option<u32> = None;
    let mut state_filter: Option<String> = None;
    let mut limit: u32 = 0;
    let mut json = false;

    let mut i = 1;
    while i < rest.len() {
        let arg = rest[i].as_str();
        match arg {
            "--repo" => {
                repo = Some(
                    rest.get(i + 1)
                        .ok_or_else(|| "missing value for --repo".to_string())?
                        .clone(),
                );
                i += 2;
            }
            "--title" => {
                title = Some(
                    rest.get(i + 1)
                        .ok_or_else(|| "missing value for --title".to_string())?
                        .clone(),
                );
                i += 2;
            }
            "--body" => {
                body = Some(
                    rest.get(i + 1)
                        .ok_or_else(|| "missing value for --body".to_string())?
                        .clone(),
                );
                i += 2;
            }
            "--head" => {
                head = Some(
                    rest.get(i + 1)
                        .ok_or_else(|| "missing value for --head".to_string())?
                        .clone(),
                );
                i += 2;
            }
            "--base" => {
                base = Some(
                    rest.get(i + 1)
                        .ok_or_else(|| "missing value for --base".to_string())?
                        .clone(),
                );
                i += 2;
            }
            "--draft" => {
                draft = true;
                i += 1;
            }
            "--state" => {
                state_filter = Some(
                    rest.get(i + 1)
                        .ok_or_else(|| "missing value for --state".to_string())?
                        .clone(),
                );
                i += 2;
            }
            "--limit" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --limit".to_string())?;
                limit = v
                    .parse::<u32>()
                    .map_err(|e| format!("invalid --limit value '{v}': {e}"))?;
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other if !other.starts_with("--") && pr_number.is_none() => {
                // Positional PR number for status / close.
                pr_number = Some(
                    other
                        .parse::<u32>()
                        .map_err(|e| format!("invalid PR number '{other}': {e}"))?,
                );
                i += 1;
            }
            other => {
                return Err(format!("unknown `pr {sub}` argument: {other}"));
            }
        }
    }

    Ok(CliAction::Pr {
        sub,
        repo,
        title,
        body,
        head,
        base,
        draft,
        pr_number,
        state_filter,
        limit,
        json,
    })
}

/// E-41 — parse `elai pr-review <sub> [args]`.
///
/// Sub-action is validated downstream in `pr_review_cmd::run_pr_review_subcommand`.
/// Flag parsing is permissive: per-subverb required flags are checked by
/// the handler so the args layer stays free of orchestrator types.
pub(crate) fn parse_pr_review_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .map(String::as_str)
        .ok_or_else(|| "pr-review requires a sub-command".to_string())?
        .to_ascii_lowercase();
    let mut pr: Option<u32> = None;
    let mut body: Option<String> = None;
    let mut file: Option<String> = None;
    let mut line: Option<u32> = None;
    let mut template: Option<String> = None;
    let mut kv_pairs: Vec<(String, String)> = Vec::new();
    let mut since: Option<i64> = None;
    let mut ruleset: Option<String> = None;
    let mut loc_added: Option<u32> = None;
    let mut loc_removed: Option<u32> = None;
    let mut files_changed: Option<u32> = None;
    let mut touches_owned = false;
    let mut repo: Option<String> = None;
    let mut branch: Option<String> = None;
    let mut touched_files: Vec<String> = Vec::new();
    let mut default_owner: Option<String> = None;
    let mut title: Option<String> = None;
    let mut json = false;

    let mut i = 1;
    while i < rest.len() {
        let arg = rest[i].as_str();
        match arg {
            "--json" => {
                json = true;
                i += 1;
            }
            "--touches-owned" => {
                touches_owned = true;
                i += 1;
            }
            "--body" => {
                body = Some(require_value(rest, i, "--body")?);
                i += 2;
            }
            "--file" => {
                file = Some(require_value(rest, i, "--file")?);
                i += 2;
            }
            "--line" => {
                let v = require_value(rest, i, "--line")?;
                line = Some(
                    v.parse::<u32>()
                        .map_err(|e| format!("invalid --line '{v}': {e}"))?,
                );
                i += 2;
            }
            "--template" => {
                template = Some(require_value(rest, i, "--template")?);
                i += 2;
            }
            "--kv" => {
                let v = require_value(rest, i, "--kv")?;
                let (k, val) = v
                    .split_once('=')
                    .ok_or_else(|| format!("--kv expects key=value, got '{v}'"))?;
                kv_pairs.push((k.to_string(), val.to_string()));
                i += 2;
            }
            "--since" => {
                let v = require_value(rest, i, "--since")?;
                since = Some(
                    v.parse::<i64>()
                        .map_err(|e| format!("invalid --since '{v}': {e}"))?,
                );
                i += 2;
            }
            "--ruleset" => {
                ruleset = Some(require_value(rest, i, "--ruleset")?);
                i += 2;
            }
            "--loc-added" => {
                let v = require_value(rest, i, "--loc-added")?;
                loc_added = Some(
                    v.parse::<u32>()
                        .map_err(|e| format!("invalid --loc-added '{v}': {e}"))?,
                );
                i += 2;
            }
            "--loc-removed" => {
                let v = require_value(rest, i, "--loc-removed")?;
                loc_removed = Some(
                    v.parse::<u32>()
                        .map_err(|e| format!("invalid --loc-removed '{v}': {e}"))?,
                );
                i += 2;
            }
            "--files-changed" => {
                let v = require_value(rest, i, "--files-changed")?;
                files_changed = Some(
                    v.parse::<u32>()
                        .map_err(|e| format!("invalid --files-changed '{v}': {e}"))?,
                );
                i += 2;
            }
            "--repo" => {
                repo = Some(require_value(rest, i, "--repo")?);
                i += 2;
            }
            "--branch" => {
                branch = Some(require_value(rest, i, "--branch")?);
                i += 2;
            }
            "--touched" => {
                touched_files.push(require_value(rest, i, "--touched")?);
                i += 2;
            }
            "--default-owner" => {
                default_owner = Some(require_value(rest, i, "--default-owner")?);
                i += 2;
            }
            "--title" => {
                title = Some(require_value(rest, i, "--title")?);
                i += 2;
            }
            other if !other.starts_with("--") && pr.is_none() => {
                pr = Some(
                    other
                        .parse::<u32>()
                        .map_err(|e| format!("invalid PR number '{other}': {e}"))?,
                );
                i += 1;
            }
            other => {
                return Err(format!("unknown `pr-review {sub}` argument: {other}"));
            }
        }
    }

    Ok(CliAction::PrReview {
        sub,
        pr,
        body,
        file,
        line,
        template,
        kv_pairs,
        since,
        ruleset,
        loc_added,
        loc_removed,
        files_changed,
        touches_owned,
        repo,
        branch,
        touched_files,
        default_owner,
        title,
        json,
    })
}

fn require_value(args: &[String], index: usize, flag: &str) -> Result<String, String> {
    args.get(index + 1)
        .cloned()
        .ok_or_else(|| format!("{flag} requires a value"))
}

/// D-2 — parse `elai ppm <sub> [args]`.
///
/// Currently `health` is the only subcommand. Future subcommands (e.g.
/// `score-once`) extend the match arm. Mirrors `parse_archive_subcommand_args`.
pub(crate) fn parse_ppm_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    if sub != "health" {
        return Err(format!(
            "unknown `ppm` sub-command: '{sub}' (expected 'health')"
        ));
    }
    let mut endpoint: Option<String> = None;
    let mut timeout_secs: Option<u64> = None;
    let mut json = false;
    let mut i = 1;
    while i < rest.len() {
        match rest[i].as_str() {
            "--endpoint" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --endpoint".to_string())?;
                endpoint = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--endpoint=") => {
                endpoint = Some(flag["--endpoint=".len()..].to_string());
                i += 1;
            }
            "--timeout" | "--timeout-secs" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --timeout".to_string())?;
                let parsed: u64 = value
                    .parse()
                    .map_err(|e| format!("invalid --timeout value '{value}': {e}"))?;
                timeout_secs = Some(parsed);
                i += 2;
            }
            flag if flag.starts_with("--timeout=") => {
                let value = &flag["--timeout=".len()..];
                let parsed: u64 = value
                    .parse()
                    .map_err(|e| format!("invalid --timeout value '{value}': {e}"))?;
                timeout_secs = Some(parsed);
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `ppm health` argument: {other}"));
            }
        }
    }
    Ok(CliAction::PpmHealth {
        endpoint,
        timeout_secs,
        json,
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

/// C-43 — parse the `elai freeze` sub-verb. Grammar:
///   `elai freeze status`
///   `elai freeze verify`
///   `elai freeze ratify --ratifier <id>`
/// ELAI design hypothesis — no paper grounding.
pub(crate) fn parse_freeze_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `freeze` sub-action; expected one of: status, verify, ratify".to_string()
        })?
        .as_str();
    match sub {
        "status" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `freeze status`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Freeze {
                action: FreezeAction::Status,
                output_format,
            })
        }
        "verify" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `freeze verify`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Freeze {
                action: FreezeAction::Verify,
                output_format,
            })
        }
        "ratify" => {
            let mut ratifier: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--ratifier" => {
                        let value = rest.get(i + 1).ok_or_else(|| {
                            "missing value for `--ratifier`".to_string()
                        })?;
                        ratifier = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--ratifier=") => {
                        ratifier = Some(flag[11..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `freeze ratify` argument: {other}"
                        ));
                    }
                }
            }
            let ratifier = ratifier.ok_or_else(|| {
                "`freeze ratify` requires --ratifier <id>".to_string()
            })?;
            Ok(CliAction::Freeze {
                action: FreezeAction::Ratify { ratifier },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `freeze` sub-action `{other}`; expected one of: status, verify, ratify"
        )),
    }
}

/// C-45 — parse `elai credit <sub>` arguments. Three sub-actions:
///   `elai credit attribute [--role <name>]`
///   `elai credit table`
///   `elai credit export --path <PATH>`
/// Paper-grounded against arXiv:2605.13295 (Zehle 2026, CANTANTE
/// Section 3.2). The CLI surface drives the synthetic-fixture path;
/// the live runpod pilot is flagged
/// `RUNPOD-PENDING: cantante_pilot_450runs_qwen3_8b`.
pub(crate) fn parse_credit_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `credit` sub-action; expected one of: attribute, table, export".to_string()
        })?
        .as_str();
    match sub {
        "attribute" => {
            let mut dropped_role: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--role" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--role`".to_string())?;
                        dropped_role = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--role=") => {
                        dropped_role = Some(flag[7..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `credit attribute` argument: {other}"
                        ));
                    }
                }
            }
            Ok(CliAction::Credit {
                action: CreditAction::Attribute { dropped_role },
                output_format,
            })
        }
        "table" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `credit table`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Credit {
                action: CreditAction::Table,
                output_format,
            })
        }
        "export" => {
            let mut path: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--path" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--path`".to_string())?;
                        path = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--path=") => {
                        path = Some(flag[7..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `credit export` argument: {other}"
                        ));
                    }
                }
            }
            let path = path.ok_or_else(|| {
                "`credit export` requires --path <PATH>".to_string()
            })?;
            Ok(CliAction::Credit {
                action: CreditAction::Export { path },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `credit` sub-action `{other}`; expected one of: attribute, table, export"
        )),
    }
}

/// C-46 — `elai routing <decisions|pilot|status|export>` parser.
///
/// Paper-grounded against arXiv:2605.05007 (Cui et al 2026,
/// Uno-Orchestra Sections 5 + 6.1). The CLI surface drives the
/// synthetic-fixture path; live runpod pilot is flagged
/// `RUNPOD-PENDING: uno_orchestra_pilot_8roles_qwen3_8b`.
pub(crate) fn parse_routing_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `routing` sub-action; expected one of: decisions, pilot, status, export"
                .to_string()
        })?
        .as_str();
    match sub {
        "decisions" => {
            let mut since_ms: Option<u64> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--since-ms" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--since-ms`".to_string())?;
                        since_ms = Some(
                            value
                                .parse::<u64>()
                                .map_err(|e| format!("invalid --since-ms: {e}"))?,
                        );
                        i += 2;
                    }
                    flag if flag.starts_with("--since-ms=") => {
                        since_ms = Some(
                            flag[11..]
                                .parse::<u64>()
                                .map_err(|e| format!("invalid --since-ms: {e}"))?,
                        );
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `routing decisions` argument: {other}"
                        ));
                    }
                }
            }
            Ok(CliAction::Routing {
                action: RoutingAction::Decisions { since_ms },
                output_format,
            })
        }
        "pilot" => {
            let mut task_template: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--task" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--task`".to_string())?;
                        task_template = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--task=") => {
                        task_template = Some(flag[7..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!("unknown `routing pilot` argument: {other}"));
                    }
                }
            }
            let task_template = task_template.ok_or_else(|| {
                "`routing pilot` requires --task <DESCRIPTION>".to_string()
            })?;
            Ok(CliAction::Routing {
                action: RoutingAction::Pilot { task_template },
                output_format,
            })
        }
        "status" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `routing status`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Routing {
                action: RoutingAction::Status,
                output_format,
            })
        }
        "export" => {
            let mut path: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--path" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--path`".to_string())?;
                        path = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--path=") => {
                        path = Some(flag[7..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `routing export` argument: {other}"
                        ));
                    }
                }
            }
            let path = path.ok_or_else(|| {
                "`routing export` requires --path <PATH>".to_string()
            })?;
            Ok(CliAction::Routing {
                action: RoutingAction::Export { path },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `routing` sub-action `{other}`; \
             expected one of: decisions, pilot, status, export"
        )),
    }
}

/// C-47 — `elai calibration <status|reset-hints|export>` argument
/// parser. Paper-grounded against arXiv:2605.15041 (CAST). Mirrors
/// the [`parse_routing_subcommand_args`] / [`parse_credit_subcommand_args`]
/// flag-walking pattern for consistency.
pub(crate) fn parse_calibration_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `calibration` sub-action; expected one of: status, reset-hints, export"
                .to_string()
        })?
        .as_str();
    match sub {
        "status" => {
            let mut tool_name: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--tool" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--tool`".to_string())?;
                        tool_name = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--tool=") => {
                        tool_name = Some(flag[7..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `calibration status` argument: {other}"
                        ));
                    }
                }
            }
            Ok(CliAction::Calibration {
                action: CalibrationAction::Status { tool_name },
                output_format,
            })
        }
        "reset-hints" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `calibration reset-hints`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Calibration {
                action: CalibrationAction::ResetHints,
                output_format,
            })
        }
        "export" => {
            let mut path: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--path" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--path`".to_string())?;
                        path = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--path=") => {
                        path = Some(flag[7..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `calibration export` argument: {other}"
                        ));
                    }
                }
            }
            let path = path.ok_or_else(|| {
                "`calibration export` requires --path <PATH>".to_string()
            })?;
            Ok(CliAction::Calibration {
                action: CalibrationAction::Export { path },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `calibration` sub-action `{other}`; \
             expected one of: status, reset-hints, export"
        )),
    }
}

/// E-14 — `elai browser <run|list-journeys|status>` argument parser.
/// Wire-dark posture inside the runner — see
/// `crates/orchestrator/src/browser_e2e/`.
pub(crate) fn parse_browser_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `browser` sub-action; expected one of: run, list-journeys, status"
                .to_string()
        })?
        .as_str();
    match sub {
        "run" => {
            let mut journey: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--journey" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--journey`".to_string())?;
                        journey = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--journey=") => {
                        journey = Some(flag["--journey=".len()..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!("unknown `browser run` argument: {other}"));
                    }
                }
            }
            let journey = journey
                .ok_or_else(|| "`browser run` requires --journey <NAME>".to_string())?;
            Ok(CliAction::Browser {
                action: BrowserAction::Run { journey },
                output_format,
            })
        }
        "list-journeys" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `browser list-journeys`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Browser {
                action: BrowserAction::ListJourneys,
                output_format,
            })
        }
        "status" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `browser status`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Browser {
                action: BrowserAction::Status,
                output_format,
            })
        }
        other => Err(format!(
            "unknown `browser` sub-action `{other}`; \
             expected one of: run, list-journeys, status"
        )),
    }
}

/// E-18 — `elai reliability <status|list-spills|replay-spill>` argument
/// parser. Composes on D-11 RegistryCheck + E-35 SearchReplaceMatcher +
/// C-47 ToolCalibrationRegistry + C-42 AuthzPolicy + C-35 ToolGate.
pub(crate) fn parse_reliability_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `reliability` sub-action; expected one of: status, list-spills, replay-spill"
                .to_string()
        })?
        .as_str();
    match sub {
        "status" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `reliability status`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Reliability {
                action: ReliabilityAction::Status,
                output_format,
            })
        }
        "list-spills" => {
            let mut dir: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--dir" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--dir`".to_string())?;
                        dir = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--dir=") => {
                        dir = Some(flag["--dir=".len()..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `reliability list-spills` argument: {other}"
                        ));
                    }
                }
            }
            Ok(CliAction::Reliability {
                action: ReliabilityAction::ListSpills { dir },
                output_format,
            })
        }
        "replay-spill" => {
            let mut path: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--path" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--path`".to_string())?;
                        path = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--path=") => {
                        path = Some(flag["--path=".len()..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `reliability replay-spill` argument: {other}"
                        ));
                    }
                }
            }
            let path = path.ok_or_else(|| {
                "`reliability replay-spill` requires --path <PATH>".to_string()
            })?;
            Ok(CliAction::Reliability {
                action: ReliabilityAction::ReplaySpill { path },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `reliability` sub-action `{other}`; \
             expected one of: status, list-spills, replay-spill"
        )),
    }
}

/// E-37 — `elai safety <status|list-pins|verify-pins|fixture-replay>`
/// argument parser. Prompt-injection defense operator surface (OWASP
/// LLM Top 10 LLM01). Mirrors the E-18 reliability parser flag-walking
/// pattern. Read-only — no destructive sub-verbs.
pub(crate) fn parse_safety_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `safety` sub-action; expected one of: status, list-pins, verify-pins, fixture-replay"
                .to_string()
        })?
        .as_str();
    match sub {
        "status" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `safety status`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Safety { action: SafetyAction::Status, output_format })
        }
        "list-pins" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `safety list-pins`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Safety { action: SafetyAction::ListPins, output_format })
        }
        "verify-pins" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `safety verify-pins`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Safety { action: SafetyAction::VerifyPins, output_format })
        }
        "fixture-replay" => {
            let mut fixture: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--fixture" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--fixture`".to_string())?;
                        fixture = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--fixture=") => {
                        fixture = Some(flag["--fixture=".len()..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `safety fixture-replay` argument: {other}"
                        ));
                    }
                }
            }
            let fixture = fixture.ok_or_else(|| {
                "`safety fixture-replay` requires --fixture <NAME> (agentdojo|echoleak|cometjacking)".to_string()
            })?;
            Ok(CliAction::Safety {
                action: SafetyAction::FixtureReplay { fixture },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `safety` sub-action `{other}`; \
             expected one of: status, list-pins, verify-pins, fixture-replay"
        )),
    }
}

/// E-40 — `elai scip <query|defs|refs|callers|callees|pagerank|status|import>`
/// argument parser. Mirrors the safety_cmd flag-walking pattern. Paper
/// grounding deferred per workflow/sessions/s-25120/mistakes.md cluster
/// #6 (Stack Graphs / GraphCoder / CodexGraph / RANGER / Glean Angle).
pub(crate) fn parse_scip_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `scip` sub-action; expected one of: query, defs, refs, callers, callees, pagerank, status, import"
                .to_string()
        })?
        .as_str();
    let args = &rest[1..];
    match sub {
        "status" => {
            if !args.is_empty() {
                return Err(format!("unexpected argument after `scip status`: {}", args[0]));
            }
            Ok(CliAction::SymbolGraph {
                action: SymbolGraphAction::Status,
                output_format,
            })
        }
        "query" => {
            let mut symbol: Option<String> = None;
            let mut kind: Option<String> = None;
            let mut i = 0;
            while i < args.len() {
                match args[i].as_str() {
                    "--symbol" => {
                        symbol = Some(
                            args.get(i + 1)
                                .ok_or_else(|| "missing value for `--symbol`".to_string())?
                                .clone(),
                        );
                        i += 2;
                    }
                    flag if flag.starts_with("--symbol=") => {
                        symbol = Some(flag["--symbol=".len()..].to_string());
                        i += 1;
                    }
                    "--kind" => {
                        kind = Some(
                            args.get(i + 1)
                                .ok_or_else(|| "missing value for `--kind`".to_string())?
                                .clone(),
                        );
                        i += 2;
                    }
                    flag if flag.starts_with("--kind=") => {
                        kind = Some(flag["--kind=".len()..].to_string());
                        i += 1;
                    }
                    other => return Err(format!("unknown `scip query` argument: {other}")),
                }
            }
            let symbol = symbol.ok_or_else(|| {
                "`scip query` requires --symbol <NAME>".to_string()
            })?;
            let kind = kind.ok_or_else(|| {
                "`scip query` requires --kind <defs|refs|callers|callees>".to_string()
            })?;
            Ok(CliAction::SymbolGraph {
                action: SymbolGraphAction::Query { symbol, kind },
                output_format,
            })
        }
        verb @ ("defs" | "refs") => {
            let mut symbol: Option<String> = None;
            let mut i = 0;
            while i < args.len() {
                match args[i].as_str() {
                    "--symbol" => {
                        symbol = Some(
                            args.get(i + 1)
                                .ok_or_else(|| "missing value for `--symbol`".to_string())?
                                .clone(),
                        );
                        i += 2;
                    }
                    flag if flag.starts_with("--symbol=") => {
                        symbol = Some(flag["--symbol=".len()..].to_string());
                        i += 1;
                    }
                    other => return Err(format!("unknown `scip {verb}` argument: {other}")),
                }
            }
            let symbol = symbol.ok_or_else(|| {
                format!("`scip {verb}` requires --symbol <NAME>")
            })?;
            let action = match verb {
                "defs" => SymbolGraphAction::Defs { symbol },
                _ => SymbolGraphAction::Refs { symbol },
            };
            Ok(CliAction::SymbolGraph { action, output_format })
        }
        "callers" | "callees" => {
            // Both `callers` and `callees` route through the canonical
            // Query variant with --kind set to the sub-verb. Parsing
            // mirrors `scip query` minus the --kind flag.
            let mut symbol: Option<String> = None;
            let mut i = 0;
            while i < args.len() {
                match args[i].as_str() {
                    "--symbol" => {
                        symbol = Some(
                            args.get(i + 1)
                                .ok_or_else(|| "missing value for `--symbol`".to_string())?
                                .clone(),
                        );
                        i += 2;
                    }
                    flag if flag.starts_with("--symbol=") => {
                        symbol = Some(flag["--symbol=".len()..].to_string());
                        i += 1;
                    }
                    other => return Err(format!("unknown `scip {sub}` argument: {other}")),
                }
            }
            let symbol = symbol.ok_or_else(|| {
                format!("`scip {sub}` requires --symbol <NAME>")
            })?;
            Ok(CliAction::SymbolGraph {
                action: SymbolGraphAction::Query {
                    symbol,
                    kind: sub.to_string(),
                },
                output_format,
            })
        }
        "pagerank" => {
            let mut seed: Option<String> = None;
            let mut k: u32 = 10;
            let mut i = 0;
            while i < args.len() {
                match args[i].as_str() {
                    "--seed" => {
                        seed = Some(
                            args.get(i + 1)
                                .ok_or_else(|| "missing value for `--seed`".to_string())?
                                .clone(),
                        );
                        i += 2;
                    }
                    flag if flag.starts_with("--seed=") => {
                        seed = Some(flag["--seed=".len()..].to_string());
                        i += 1;
                    }
                    "--k" => {
                        let raw = args
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--k`".to_string())?;
                        k = raw
                            .parse::<u32>()
                            .map_err(|e| format!("invalid `--k` value `{raw}`: {e}"))?;
                        i += 2;
                    }
                    flag if flag.starts_with("--k=") => {
                        let raw = &flag["--k=".len()..];
                        k = raw
                            .parse::<u32>()
                            .map_err(|e| format!("invalid `--k` value `{raw}`: {e}"))?;
                        i += 1;
                    }
                    other => return Err(format!("unknown `scip pagerank` argument: {other}")),
                }
            }
            let seed = seed.ok_or_else(|| {
                "`scip pagerank` requires --seed <NAME>".to_string()
            })?;
            Ok(CliAction::SymbolGraph {
                action: SymbolGraphAction::PageRank { seed, k },
                output_format,
            })
        }
        "import" => {
            let mut dir: String = String::new();
            let mut i = 0;
            while i < args.len() {
                match args[i].as_str() {
                    "--dir" => {
                        dir = args
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--dir`".to_string())?
                            .clone();
                        i += 2;
                    }
                    flag if flag.starts_with("--dir=") => {
                        dir = flag["--dir=".len()..].to_string();
                        i += 1;
                    }
                    other => return Err(format!("unknown `scip import` argument: {other}")),
                }
            }
            Ok(CliAction::SymbolGraph {
                action: SymbolGraphAction::ImportScip { dir },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `scip` sub-action `{other}`; \
             expected one of: query, defs, refs, callers, callees, pagerank, status, import"
        )),
    }
}

/// D-16 — `elai conformal-calibration <status|update>` argument
/// parser. Paper-grounded against arXiv:2509.01455 (UniCR Eq.5).
/// Mirrors the C-47 [`parse_calibration_subcommand_args`] flag-walking
/// pattern for consistency.
pub(crate) fn parse_conformal_calibration_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `conformal-calibration` sub-action; expected one of: status, update"
                .to_string()
        })?
        .as_str();
    match sub {
        "status" => {
            let mut tau_path: Option<String> = None;
            let mut corpus_path: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--tau-path" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--tau-path`".to_string())?;
                        tau_path = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--tau-path=") => {
                        tau_path = Some(flag["--tau-path=".len()..].to_string());
                        i += 1;
                    }
                    "--corpus-path" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--corpus-path`".to_string())?;
                        corpus_path = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--corpus-path=") => {
                        corpus_path = Some(flag["--corpus-path=".len()..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `conformal-calibration status` argument: {other}"
                        ));
                    }
                }
            }
            Ok(CliAction::ConformalCalibration {
                action: ConformalCalibrationAction::Status {
                    tau_path,
                    corpus_path,
                },
                output_format,
            })
        }
        "update" => {
            let mut tau_path: Option<String> = None;
            let mut corpus_path: Option<String> = None;
            let mut alpha: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--tau-path" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--tau-path`".to_string())?;
                        tau_path = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--tau-path=") => {
                        tau_path = Some(flag["--tau-path=".len()..].to_string());
                        i += 1;
                    }
                    "--corpus-path" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--corpus-path`".to_string())?;
                        corpus_path = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--corpus-path=") => {
                        corpus_path = Some(flag["--corpus-path=".len()..].to_string());
                        i += 1;
                    }
                    "--alpha" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--alpha`".to_string())?;
                        alpha = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--alpha=") => {
                        alpha = Some(flag["--alpha=".len()..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `conformal-calibration update` argument: {other}"
                        ));
                    }
                }
            }
            Ok(CliAction::ConformalCalibration {
                action: ConformalCalibrationAction::Update {
                    tau_path,
                    corpus_path,
                    alpha,
                },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `conformal-calibration` sub-action `{other}`; \
             expected one of: status, update"
        )),
    }
}

/// C-49 — `elai experience <distill|list|export|reset>` parser.
///
/// Mirrors the C-46 `routing` + C-47 `calibration` shapes for
/// uniformity. All sub-actions are GPU-free synthetic-fixture paths;
/// the live trace recorder is RUNPOD-PENDING.
pub(crate) fn parse_experience_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `experience` sub-action; expected one of: distill, list, export, reset"
                .to_string()
        })?
        .as_str();
    match sub {
        "distill" => {
            let mut skill: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--skill" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--skill`".to_string())?;
                        skill = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--skill=") => {
                        skill = Some(flag[8..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `experience distill` argument: {other}"
                        ));
                    }
                }
            }
            Ok(CliAction::Experience {
                action: ExperienceAction::Distill { skill },
                output_format,
            })
        }
        "list" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `experience list`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Experience {
                action: ExperienceAction::List,
                output_format,
            })
        }
        "export" => {
            let mut path: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--path" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--path`".to_string())?;
                        path = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--path=") => {
                        path = Some(flag[7..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `experience export` argument: {other}"
                        ));
                    }
                }
            }
            let path = path.ok_or_else(|| {
                "`experience export` requires --path <PATH>".to_string()
            })?;
            Ok(CliAction::Experience {
                action: ExperienceAction::Export { path },
                output_format,
            })
        }
        "reset" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `experience reset`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Experience {
                action: ExperienceAction::Reset,
                output_format,
            })
        }
        other => Err(format!(
            "unknown `experience` sub-action `{other}`; \
             expected one of: distill, list, export, reset"
        )),
    }
}

/// C-50 — `elai contract <scaffold|audit|list>` parser.
///
/// Paper-grounded against arXiv:2604.13100 (Contract-Coding §4.2
/// kernel K = <N, Sigma, Delta>). The CLI surface drives the
/// heuristic structural Discriminator + scaffolder; the full RL
/// Generator -> Discriminator pipeline is RUNPOD-PENDING.
pub(crate) fn parse_contract_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `contract` sub-action; expected one of: scaffold, audit, list".to_string()
        })?
        .as_str();
    match sub {
        "scaffold" => {
            let mut fn_name: Option<String> = None;
            let mut output: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--fn-name" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--fn-name`".to_string())?;
                        fn_name = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--fn-name=") => {
                        fn_name = Some(flag[10..].to_string());
                        i += 1;
                    }
                    "--output" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--output`".to_string())?;
                        output = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--output=") => {
                        output = Some(flag[9..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `contract scaffold` argument: {other}"
                        ));
                    }
                }
            }
            let fn_name = fn_name.ok_or_else(|| {
                "`contract scaffold` requires --fn-name <NAME>".to_string()
            })?;
            Ok(CliAction::Contract {
                action: ContractAction::Scaffold { fn_name, output },
                output_format,
            })
        }
        "audit" => {
            let mut code_path: Option<String> = None;
            let mut spec_path: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--code-path" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--code-path`".to_string())?;
                        code_path = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--code-path=") => {
                        code_path = Some(flag[12..].to_string());
                        i += 1;
                    }
                    "--spec-path" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--spec-path`".to_string())?;
                        spec_path = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--spec-path=") => {
                        spec_path = Some(flag[12..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!("unknown `contract audit` argument: {other}"));
                    }
                }
            }
            let code_path = code_path.ok_or_else(|| {
                "`contract audit` requires --code-path <PATH>".to_string()
            })?;
            let spec_path = spec_path.ok_or_else(|| {
                "`contract audit` requires --spec-path <PATH>".to_string()
            })?;
            Ok(CliAction::Contract {
                action: ContractAction::Audit { code_path, spec_path },
                output_format,
            })
        }
        "list" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `contract list`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Contract {
                action: ContractAction::List,
                output_format,
            })
        }
        other => Err(format!(
            "unknown `contract` sub-action `{other}`; \
             expected one of: scaffold, audit, list"
        )),
    }
}

/// C-52 — parse `elai dafny <sub>` arguments. Three sub-actions:
///   `elai dafny verify --spec-path PATH [--tier signature] [--max-iter 10]`
///   `elai dafny status`
///   `elai dafny list-tiers`
/// Paper-grounded against arXiv:2604.22601 (NL2VC). Live `dafny
/// verify` execution is `RUNPOD-PENDING: live_dafny_verification_with_non_gemma_substitution`
/// — the binary is not on PATH on the local dev host (per the C-52
/// cross-check report). Model substitution mandatory — Gemma 4
/// BLOCKED per memory rule `feedback_no_gemma4`.
pub(crate) fn parse_dafny_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `dafny` sub-action; expected one of: verify, status, list-tiers".to_string()
        })?
        .as_str();
    match sub {
        "verify" => {
            let mut spec_path: Option<String> = None;
            let mut tier: Option<String> = None;
            let mut max_iter: Option<u32> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--spec-path" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--spec-path`".to_string())?;
                        spec_path = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--spec-path=") => {
                        spec_path = Some(flag[12..].to_string());
                        i += 1;
                    }
                    "--tier" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--tier`".to_string())?;
                        tier = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--tier=") => {
                        tier = Some(flag[7..].to_string());
                        i += 1;
                    }
                    "--max-iter" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--max-iter`".to_string())?;
                        let n: u32 = value
                            .parse()
                            .map_err(|e| format!("invalid `--max-iter` value `{value}`: {e}"))?;
                        max_iter = Some(n);
                        i += 2;
                    }
                    flag if flag.starts_with("--max-iter=") => {
                        let value = &flag[11..];
                        let n: u32 = value
                            .parse()
                            .map_err(|e| format!("invalid `--max-iter` value `{value}`: {e}"))?;
                        max_iter = Some(n);
                        i += 1;
                    }
                    other => {
                        return Err(format!("unknown `dafny verify` argument: {other}"));
                    }
                }
            }
            let spec_path = spec_path.ok_or_else(|| {
                "`dafny verify` requires --spec-path <PATH>".to_string()
            })?;
            Ok(CliAction::Dafny {
                action: DafnyAction::Verify {
                    spec_path,
                    tier,
                    max_iter,
                },
                output_format,
            })
        }
        "status" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `dafny status`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Dafny {
                action: DafnyAction::Status,
                output_format,
            })
        }
        "list-tiers" | "list_tiers" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `dafny list-tiers`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Dafny {
                action: DafnyAction::ListTiers,
                output_format,
            })
        }
        other => Err(format!(
            "unknown `dafny` sub-action `{other}`; \
             expected one of: verify, status, list-tiers"
        )),
    }
}

/// E-13 — parse `elai tdd <sub>` arguments. Three sub-actions:
///   `elai tdd run --task-path PATH`
///   `elai tdd status`
///   `elai tdd mutation-score --test-path PATH`
///
/// Mirrors the C-52 dafny parser shape for consistency. Live
/// cargo-mutants execution is RUNPOD-PENDING when the binary is
/// absent on PATH (wire-dark posture per E-12 Tier-B + C-52 dafny
/// precedents). Citation discipline per
/// workflow/sessions/s-25120/mistakes.md: 11 arXiv IDs in the plan,
/// only LLMorpheus + ThinkPRM inline-cited; rest deferred.
pub(crate) fn parse_tdd_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    if rest.is_empty() {
        return Err(
            "`tdd` requires a sub-action: run | status | mutation-score".to_string(),
        );
    }
    match rest[0].as_str() {
        "run" => {
            let mut task_path: Option<String> = None;
            let mut i = 1usize;
            while i < rest.len() {
                let arg = rest[i].as_str();
                match arg {
                    "--task-path" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--task-path`".to_string())?;
                        task_path = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--task-path=") => {
                        task_path = Some(flag[12..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!("unknown `tdd run` argument: {other}"));
                    }
                }
            }
            let task_path = task_path
                .ok_or_else(|| "`tdd run` requires --task-path <PATH>".to_string())?;
            Ok(CliAction::Tdd {
                action: TddAction::Run { task_path },
                output_format,
            })
        }
        "status" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `tdd status`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Tdd {
                action: TddAction::Status,
                output_format,
            })
        }
        "mutation-score" | "mutation_score" => {
            let mut test_path: Option<String> = None;
            let mut i = 1usize;
            while i < rest.len() {
                let arg = rest[i].as_str();
                match arg {
                    "--test-path" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--test-path`".to_string())?;
                        test_path = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--test-path=") => {
                        test_path = Some(flag[12..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!("unknown `tdd mutation-score` argument: {other}"));
                    }
                }
            }
            let test_path = test_path.ok_or_else(|| {
                "`tdd mutation-score` requires --test-path <PATH>".to_string()
            })?;
            Ok(CliAction::Tdd {
                action: TddAction::MutationScore { test_path },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `tdd` sub-action `{other}`; \
             expected one of: run, status, mutation-score"
        )),
    }
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

/// E-5 session-2 — parse `elai memory <sub> [...flags]`.
///
/// Subcommands: `list`, `inspect`, `insert`, `delete`. Adjudication +
/// promotion + Stage-2 classifier are DEFERRED per
/// workflow/reasoning/E-5-decisions.md.
///
/// All flags are optional at parse time; per-subcommand required-flag
/// validation runs in the handler so the error message can name the
/// subcommand (e.g. `"inspect requires --fact-id"`).
pub(crate) fn parse_memory_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let Some(sub_raw) = rest.first().map(String::as_str) else {
        return Err(
            "memory: expected sub-command (list | inspect | insert | delete)".to_string(),
        );
    };
    let sub = match sub_raw {
        "list" => MemorySubcommandKind::List,
        "inspect" | "show" => MemorySubcommandKind::Inspect,
        "insert" | "add" => MemorySubcommandKind::Insert,
        "delete" | "forget" => MemorySubcommandKind::Delete,
        other => {
            return Err(format!(
                "unknown `memory` sub-command: '{other}' (expected list | inspect | insert | delete)"
            ));
        }
    };
    let rest = &rest[1..];
    let mut project_root: Option<PathBuf> = None;
    let mut user_id: Option<String> = None;
    let mut scope: Option<String> = None;
    let mut fact_type: Option<String> = None;
    let mut fact_id: Option<String> = None;
    let mut key: Option<String> = None;
    let mut value_json: Option<String> = None;
    let mut limit: Option<usize> = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--project-root" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --project-root".to_string())?;
                project_root = Some(PathBuf::from(v));
                i += 2;
            }
            flag if flag.starts_with("--project-root=") => {
                project_root = Some(PathBuf::from(&flag["--project-root=".len()..]));
                i += 1;
            }
            "--user-id" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --user-id".to_string())?;
                user_id = Some(v.clone());
                i += 2;
            }
            flag if flag.starts_with("--user-id=") => {
                user_id = Some(flag["--user-id=".len()..].to_string());
                i += 1;
            }
            "--scope" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --scope".to_string())?;
                scope = Some(v.clone());
                i += 2;
            }
            flag if flag.starts_with("--scope=") => {
                scope = Some(flag["--scope=".len()..].to_string());
                i += 1;
            }
            "--fact-type" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --fact-type".to_string())?;
                fact_type = Some(v.clone());
                i += 2;
            }
            flag if flag.starts_with("--fact-type=") => {
                fact_type = Some(flag["--fact-type=".len()..].to_string());
                i += 1;
            }
            "--fact-id" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --fact-id".to_string())?;
                fact_id = Some(v.clone());
                i += 2;
            }
            flag if flag.starts_with("--fact-id=") => {
                fact_id = Some(flag["--fact-id=".len()..].to_string());
                i += 1;
            }
            "--key" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --key".to_string())?;
                key = Some(v.clone());
                i += 2;
            }
            flag if flag.starts_with("--key=") => {
                key = Some(flag["--key=".len()..].to_string());
                i += 1;
            }
            "--value-json" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --value-json".to_string())?;
                value_json = Some(v.clone());
                i += 2;
            }
            flag if flag.starts_with("--value-json=") => {
                value_json = Some(flag["--value-json=".len()..].to_string());
                i += 1;
            }
            "--limit" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --limit".to_string())?;
                limit = Some(
                    v.parse::<usize>()
                        .map_err(|e| format!("--limit must be a non-negative integer: {e}"))?,
                );
                i += 2;
            }
            flag if flag.starts_with("--limit=") => {
                limit = Some(
                    flag["--limit=".len()..]
                        .parse::<usize>()
                        .map_err(|e| format!("--limit must be a non-negative integer: {e}"))?,
                );
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => return Err(format!("unknown `memory {sub_raw}` argument: {other}")),
        }
    }
    let project_root = match project_root {
        Some(p) => p,
        None => std::env::current_dir().map_err(|e| format!("cannot determine cwd: {e}"))?,
    };
    let user_id = user_id.unwrap_or_else(|| "default".to_string());
    let limit = limit.unwrap_or(50).max(1).min(500);
    Ok(CliAction::Memory {
        sub,
        project_root,
        user_id,
        scope,
        fact_type,
        fact_id,
        key,
        value_json,
        limit,
        json,
    })
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

// ---------------------------------------------------------------------------
// E-44 — `elai durability <sub>` argument surface.
// ---------------------------------------------------------------------------

/// E-44 — `elai durability <sub>` sub-verb. Anchors the orchestrator's
/// `durability::*` module as the production callable for
/// IdempotentDispatcher + CompactLock + checkpoint_migrator +
/// continue_as_new wrappers + resume_validator. WIRE-DARK avoidance
/// per E-41 precedent (E40_WIRE_DARK_AVOIDANCE_VIA_CLI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DurabilityAction {
    /// `elai durability status` — render the current durability
    /// configuration: CURRENT_CHECKPOINT_SCHEMA_VERSION,
    /// DEFAULT_EVENT_COUNT_CEILING, DEFAULT_PHASE_COUNT_CEILING, and
    /// the 5 SSB action types this surface emits.
    Status,
    /// `elai durability validate-checkpoint --path PATH
    /// [--expected-blake3 HEX]` — invoke
    /// `durability::resume_validator::validate_checkpoint` against
    /// PATH and render the CheckpointValidity verdict.
    ValidateCheckpoint {
        path: String,
        expected_blake3: Option<String>,
    },
    /// `elai durability migrate --path PATH --from-version V` — read
    /// raw msgpack bytes from PATH and run
    /// `durability::migrate_checkpoint(bytes, V)`. Prints the
    /// migrated byte length + blake3 of the result. Never mutates
    /// the source file.
    Migrate {
        path: String,
        from_version: u32,
    },
    /// `elai durability continue-as-new-preview --session-id S
    /// --events N --phases M` — invoke
    /// `durability::continue_as_new_if_needed` against the supplied
    /// state and render the BelowThreshold / Rollover verdict
    /// without performing any snapshot write.
    ContinueAsNewPreview {
        session_id: String,
        events: usize,
        phases: usize,
    },
    /// `elai durability acquire-lock --path PATH` — call
    /// `CompactLock::try_acquire` and immediately release. Operator
    /// primitive to verify a path is uncontended before booting the
    /// PreCompact-affecting paths.
    AcquireLock { path: String },
}

/// E-45 — `elai drift <sub>` sub-verb. Anchors the orchestrator's
/// `cross_lang_drift::*` module as the production callable for the
/// host-side drift detector. WIRE-DARK avoidance per E-41 precedent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DriftAction {
    /// `elai drift detect [--json]` — load the pair registry, run the
    /// detector, and print the report. `--json` switches to machine
    /// output.
    Detect { json: bool },
    /// `elai drift list-pairs` — render the loaded
    /// `cross-lang-pairs.toml` rows. GPU-free, registry-only.
    ListPairs,
    /// `elai drift baseline --commit HASH` — compare current drift
    /// totals to `workflow/audits/E-45-drift-baseline-<HASH>.json`.
    Baseline { commit: String },
}

pub(crate) fn parse_drift_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `drift` sub-action; expected one of: detect, list-pairs, baseline"
                .to_string()
        })?
        .as_str();
    match sub {
        "detect" => {
            let mut json = false;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--json" => {
                        json = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!("unknown `drift detect` arg: {other}"));
                    }
                }
            }
            Ok(CliAction::Drift {
                action: DriftAction::Detect { json },
                output_format,
            })
        }
        "list-pairs" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `drift list-pairs`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Drift {
                action: DriftAction::ListPairs,
                output_format,
            })
        }
        "baseline" => {
            let mut commit: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--commit" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--commit`".to_string())?;
                        commit = Some(v.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--commit=") => {
                        commit = Some(flag["--commit=".len()..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!("unknown `drift baseline` arg: {other}"));
                    }
                }
            }
            let commit =
                commit.ok_or_else(|| "`drift baseline` requires --commit".to_string())?;
            Ok(CliAction::Drift {
                action: DriftAction::Baseline { commit },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `drift` sub-action: {other}; expected one of: detect, list-pairs, baseline"
        )),
    }
}

/// E-49 — `elai metrics <sub>` sub-verb. Anchors the orchestrator's
/// `coding_metrics::*` module as the production callable for the
/// coding-agent metric suite. WIRE-DARK avoidance per E-41 precedent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MetricsAction {
    /// `elai metrics capture --session-id S` — run MetricsCollector and
    /// print the resulting CodingMetric capture.
    Capture { session_id: String },
    /// `elai metrics swe-bench-pro` — dry-run the SWE-Bench Pro live
    /// transport (RUNPOD-PENDING in this scope-lean ship).
    SweBenchPro,
    /// `elai metrics refinebench` — dry-run the RefineBench live transport
    /// (RUNPOD-PENDING in this scope-lean ship).
    RefineBench,
    /// `elai metrics aggregate` — exercise the aggregator over a single
    /// zero-observation capture (smoke test). The dashboard surface will
    /// supply real per-session snapshots in a follow-up patch.
    Aggregate,
    /// `elai metrics status` — print the SSB action types + wired primitives
    /// + RUNPOD-PENDING boundaries.
    Status,
}

pub(crate) fn parse_metrics_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `metrics` sub-action; expected one of: capture, swe-bench-pro, \
             refinebench, aggregate, status"
                .to_string()
        })?
        .as_str();
    match sub {
        "capture" => {
            let mut session_id: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--session-id" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--session-id`".to_string())?;
                        session_id = Some(v.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--session-id=") => {
                        session_id = Some(flag["--session-id=".len()..].to_string());
                        i += 1;
                    }
                    other => return Err(format!("unknown `metrics capture` arg: {other}")),
                }
            }
            let session_id = session_id
                .ok_or_else(|| "`metrics capture` requires --session-id".to_string())?;
            Ok(CliAction::Metrics {
                action: MetricsAction::Capture { session_id },
                output_format,
            })
        }
        "swe-bench-pro" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `metrics swe-bench-pro`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Metrics {
                action: MetricsAction::SweBenchPro,
                output_format,
            })
        }
        "refinebench" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `metrics refinebench`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Metrics {
                action: MetricsAction::RefineBench,
                output_format,
            })
        }
        "aggregate" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `metrics aggregate`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Metrics {
                action: MetricsAction::Aggregate,
                output_format,
            })
        }
        "status" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `metrics status`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Metrics {
                action: MetricsAction::Status,
                output_format,
            })
        }
        other => Err(format!(
            "unknown `metrics` sub-action: {other}; expected one of: \
             capture, swe-bench-pro, refinebench, aggregate, status"
        )),
    }
}

pub(crate) fn parse_durability_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `durability` sub-action; expected one of: \
             status, validate-checkpoint, migrate, \
             continue-as-new-preview, acquire-lock"
                .to_string()
        })?
        .as_str();
    match sub {
        "status" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `durability status`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Durability {
                action: DurabilityAction::Status,
                output_format,
            })
        }
        "validate-checkpoint" => {
            let mut path: Option<String> = None;
            let mut expected: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--path" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--path`".to_string())?;
                        path = Some(v.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--path=") => {
                        path = Some(flag["--path=".len()..].to_string());
                        i += 1;
                    }
                    "--expected-blake3" => {
                        let v = rest.get(i + 1).ok_or_else(|| {
                            "missing value for `--expected-blake3`".to_string()
                        })?;
                        expected = Some(v.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--expected-blake3=") => {
                        expected = Some(flag["--expected-blake3=".len()..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `durability validate-checkpoint` arg: {other}"
                        ));
                    }
                }
            }
            let path =
                path.ok_or_else(|| "`durability validate-checkpoint` requires --path".to_string())?;
            Ok(CliAction::Durability {
                action: DurabilityAction::ValidateCheckpoint {
                    path,
                    expected_blake3: expected,
                },
                output_format,
            })
        }
        "migrate" => {
            let mut path: Option<String> = None;
            let mut from_version: Option<u32> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--path" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--path`".to_string())?;
                        path = Some(v.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--path=") => {
                        path = Some(flag["--path=".len()..].to_string());
                        i += 1;
                    }
                    "--from-version" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--from-version`".to_string())?;
                        from_version = Some(
                            v.parse::<u32>().map_err(|e| {
                                format!("invalid value for `--from-version`: {e}")
                            })?,
                        );
                        i += 2;
                    }
                    flag if flag.starts_with("--from-version=") => {
                        let raw = &flag["--from-version=".len()..];
                        from_version = Some(
                            raw.parse::<u32>().map_err(|e| {
                                format!("invalid value for `--from-version`: {e}")
                            })?,
                        );
                        i += 1;
                    }
                    other => {
                        return Err(format!("unknown `durability migrate` arg: {other}"));
                    }
                }
            }
            let path =
                path.ok_or_else(|| "`durability migrate` requires --path".to_string())?;
            let from_version = from_version
                .ok_or_else(|| "`durability migrate` requires --from-version".to_string())?;
            Ok(CliAction::Durability {
                action: DurabilityAction::Migrate { path, from_version },
                output_format,
            })
        }
        "continue-as-new-preview" => {
            let mut session_id: Option<String> = None;
            let mut events: Option<usize> = None;
            let mut phases: Option<usize> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--session-id" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--session-id`".to_string())?;
                        session_id = Some(v.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--session-id=") => {
                        session_id = Some(flag["--session-id=".len()..].to_string());
                        i += 1;
                    }
                    "--events" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--events`".to_string())?;
                        events = Some(
                            v.parse::<usize>()
                                .map_err(|e| format!("invalid `--events`: {e}"))?,
                        );
                        i += 2;
                    }
                    flag if flag.starts_with("--events=") => {
                        events = Some(
                            flag["--events=".len()..]
                                .parse::<usize>()
                                .map_err(|e| format!("invalid `--events`: {e}"))?,
                        );
                        i += 1;
                    }
                    "--phases" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--phases`".to_string())?;
                        phases = Some(
                            v.parse::<usize>()
                                .map_err(|e| format!("invalid `--phases`: {e}"))?,
                        );
                        i += 2;
                    }
                    flag if flag.starts_with("--phases=") => {
                        phases = Some(
                            flag["--phases=".len()..]
                                .parse::<usize>()
                                .map_err(|e| format!("invalid `--phases`: {e}"))?,
                        );
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `durability continue-as-new-preview` arg: {other}"
                        ));
                    }
                }
            }
            let session_id = session_id.ok_or_else(|| {
                "`durability continue-as-new-preview` requires --session-id".to_string()
            })?;
            let events = events.ok_or_else(|| {
                "`durability continue-as-new-preview` requires --events".to_string()
            })?;
            let phases = phases.ok_or_else(|| {
                "`durability continue-as-new-preview` requires --phases".to_string()
            })?;
            Ok(CliAction::Durability {
                action: DurabilityAction::ContinueAsNewPreview {
                    session_id,
                    events,
                    phases,
                },
                output_format,
            })
        }
        "acquire-lock" => {
            let mut path: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--path" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--path`".to_string())?;
                        path = Some(v.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--path=") => {
                        path = Some(flag["--path=".len()..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `durability acquire-lock` arg: {other}"
                        ));
                    }
                }
            }
            let path =
                path.ok_or_else(|| "`durability acquire-lock` requires --path".to_string())?;
            Ok(CliAction::Durability {
                action: DurabilityAction::AcquireLock { path },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `durability` sub-action `{other}`; \
             expected one of: status, validate-checkpoint, migrate, \
             continue-as-new-preview, acquire-lock"
        )),
    }
}
