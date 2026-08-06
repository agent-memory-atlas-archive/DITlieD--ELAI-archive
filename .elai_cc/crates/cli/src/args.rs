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
    /// J-2: local Tier-1 read-only MCP control-plane server over stdio.
    McpServe {
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
    /// J-30 T5: `elai providers configure [--backend <name>]
    /// [--key <key>] [--base-url <url>] [--model <model>]`. Direct
    /// (non-REPL) invocation; the existing `/providers` slash command
    /// (commands.rs) is listing-only and is NOT touched. Flags-shaped
    /// struct lives in providers_cmd.rs; the args layer stays free of
    /// any providers type dependency by carrying the four optional
    /// strings verbatim.
    Providers {
        backend: Option<String>,
        key: Option<String>,
        base_url: Option<String>,
        model: Option<String>,
        output_format: CliOutputFormat,
    },
    /// J-30 T6: `elai providers status [--backend <name>] [--json]`.
    /// Read-side companion to T5's configure flow: prints per-backend
    /// inventory (configured / verified_at / models / stale) from
    /// `provider_status.json` + descriptor + key_store + model catalog.
    /// `--backend` filters to one row; absent = all backends. Handler
    /// in providers_cmd.rs::handle_providers_status.
    ProvidersStatus {
        backend: Option<String>,
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
    /// E-12 — `elai production-checklist [--project-root PATH]
    /// [--output-format text|json|toml]`. Runs the production security
    /// checklist against a target directory and reports results.
    ProductionChecklist {
        project_root: PathBuf,
        output_format: CliOutputFormat,
    },
    /// B-43 Phase 7 — `elai install [--skip-models] [--reset-config]`.
    /// Re-entry from the already-installed CLI; spawns scripts/install/install.{sh,ps1}
    /// with forwarded flags. Flag-bearing forms route through `parse_install_subcommand`.
    Install {
        skip_models: bool,
        reset_config: bool,
        project_root: Option<PathBuf>,
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
        /// G-2 -- `--emit-rra [<path>]`. When set, force-pins the
        /// Reproducible-Run Artifact (`.rra`) output path. The RRA is sealed
        /// by default at run end (kill-switch `ELAI_RRA_DISABLE`); this flag
        /// lets the operator pin a custom output path. `None` => default
        /// `<sessions>/<sid>/run.rra`.
        emit_rra: Option<PathBuf>,
        /// I-42 — `--profile orchestrate|critique-only`. Distinct axis from
        /// `--mode` (gate autonomy) and `--provider` (where work runs):
        /// `--profile` controls WHO writes and whether executor/fixer agents
        /// are spawned at all. Resolved to a
        /// [`elai_orchestrator::RunProfile`] by
        /// [`elai_orchestrator::RunProfile::from_cli_args`] at the production
        /// wiring point; in `critique-only` the executor/fixer spawn is
        /// structurally refused. `None` => default `orchestrate`.
        profile: Option<String>,
        /// I-44 — `--zero-retention`. When set (AND the default-OFF
        /// `ELAI_ZERO_RETENTION_ENABLE` master kill-switch is truthy), the run
        /// purges its own raw managed-transcript files after the sealed verdict
        /// (G-2 RRA + G-8 attestation) commits, keeping only the redacted,
        /// hash-checkable evidence. Destructive + irreversible; `false` (flag
        /// absent) is the fail-safe default (retain everything). Resolved to an
        /// [`elai_orchestrator::retention::ExportRetentionMode`] at the seal seam.
        zero_retention: bool,
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
    /// F-5 / Slice A4: `elai dkg staleness-tick --workspace <path>` — the W-5
    /// trigger. Forces one memory decay tick, collects facts past the
    /// staleness-quarantine threshold, pushes them onto a local cascade queue,
    /// emits one `StalenessQuarantineBroadcast` SSB row per record, drains the
    /// queue, applies the DKG (`apply_staleness_broadcast`) + cosmos
    /// (`flag_on_staleness_broadcast`) consumers, and prints
    /// `dependents_demoted: N`. Single-process full cycle. Handler:
    /// `crate::dkg_staleness_cmd::run_dkg_staleness_tick`.
    DkgStalenessTick {
        workspace: PathBuf,
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
    /// B-29 Phase 5b: `elai symbol_impact --target <symbol-or-file>
    /// [--depth <u32>] [--project-root <path>] [--output-format text|json]`
    /// Thin shell wrapper around
    /// `knowledge::symbol_graph::traversal::compute_impact_radius`. Exposes
    /// the B-29 SymbolGraphCache to fork-exec callers (Python eval, agents)
    /// without going through the MCP/IPC bridge.
    SymbolImpact {
        project_root: PathBuf,
        target: String,
        depth: u32,
        output_format: CliOutputFormat,
    },
    /// J-39 T3: `elai cone_token_floor --target <file-or-symbol>
    /// [--depth <u32>] [--project-root <path>] [--output-format text|json]`
    /// Thin shell wrapper around
    /// `elai_orchestrator::derive_cone_token_floor`.
    ConeTokenFloor {
        project_root: PathBuf,
        target: String,
        depth: u32,
        output_format: CliOutputFormat,
    },
    /// B-29 Phase 5b: `elai symbol_context --name <symbol-name>
    /// [--file <path>] [--project-root <path>] [--output-format text|json]`
    /// Thin shell wrapper around
    /// `knowledge::symbol_graph::traversal::compute_symbol_context`.
    SymbolContext {
        project_root: PathBuf,
        name: String,
        file: Option<String>,
        output_format: CliOutputFormat,
    },
    /// B-34 Phase 5b: `elai entry_points [--project-root <path>]
    /// [--output-format text|json] [--limit <usize>]`
    /// Thin shell wrapper around
    /// `knowledge::symbol_graph::entry_points::detect_entry_points`.
    /// Mirrors the TOOL_ENTRY_POINTS dispatcher in production_agent_rpc.rs
    /// (single-process, no MCP bridge).
    EntryPoints {
        project_root: PathBuf,
        limit: usize,
        output_format: CliOutputFormat,
    },
    /// E-49 SWE-Bench Pro v5 driver surface — `elai task <problem>
    /// --repo-root PATH --mode local|hybrid|cloud --output-format json|text
    /// --max-wall-seconds N [--executor-runtime <backend>]
    /// [--auditor-runtime <backend>]`.
    /// Non-interactive one-shot task submission: applies per-role runtime
    /// overrides (B-37 toml-edit), boots the FSM-aware task_runner inline,
    /// writes a `TaskSubmitted` SSB row matching the dashboard's
    /// `submit_task` Tauri command, tails the broadcast until terminal
    /// signal (`PhaseComplete` or runner-exit) or wall-budget elapses, and
    /// emits one JSONL summary line on stdout. Driven by the v5 SWE-Bench
    /// Pro Python driver via subprocess-wrap. Handler:
    /// `crate::task_cmd::run_task_subcommand`.
    Task {
        problem_statement: String,
        repo_root: PathBuf,
        mode: crate::task_cmd::TaskMode,
        output_format: crate::task_cmd::TaskOutputFormat,
        max_wall_seconds: u64,
        executor_runtime: Option<String>,
        auditor_runtime: Option<String>,
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
    /// F-7 / Slice A4: `elai template-ranking list [--json]` — rank
    /// compiled-pipeline templates by Laplace-smoothed outcome utility (HALF B
    /// CBR ranking). Reads the persisted `CompiledPipelineStore`, prints
    /// utility alongside the raw `first_try_ratio` baseline (the before/after),
    /// and emits a `template_utility_ranked` SSB row per template (default-ON;
    /// `ELAI_TEMPLATE_CBR_DISABLED` kill-switch). Top-level subcommand (not a
    /// REPL slash command) parsed by
    /// `parse_template_ranking_subcommand_args`; handler lives in
    /// `crate::template_ranking_cmd::run_template_ranking_subcommand`.
    TemplateRanking {
        action: Option<String>,
        json: bool,
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
    /// WIRING-FIX-ELAI Phase 2 (SP3): `elai wiring-coverage [--manifest PATH]
    /// [--diff-base REF] [--output json]` — Layer-3 entrypoint-coverage gate.
    /// Builds the instrumented `elai` binary, drives the auditor-owned smoke
    /// scenarios against the REAL binary under production-default env, and
    /// BLOCKs any NEW (diff-introduced) function whose execution count is 0
    /// unless cfg(test) or a registered `// WIRE-DARK[B-N]:` marker suppresses
    /// it. Exits 0 on clean, 1 on any violation OR un-measurable evidence
    /// (R2 fail-closed). Handler at `crate::wiring_coverage_cmd::run`.
    WiringCoverage {
        manifest_path: Option<String>,
        diff_base: Option<String>,
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
    /// D-1: `elai template migrate [--source DIR] [--path PATH] [--json]`.
    /// Bootstraps the append-only reasoning-template archive
    /// (`<cwd>/archive/reasoning-templates.jsonl`) from the built-in canonical
    /// seed set (+ optional `--source DIR` files), writing one Frozen
    /// ArchiveEntry per pattern and emitting one ArchiveEntryAdded SSB row per
    /// appended entry. Idempotent: re-running on unchanged input appends zero.
    TemplateMigrate {
        source: Option<String>,
        path: Option<String>,
        json: bool,
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
    /// G-2: `elai rra show <path>` — render a Reproducible-Run Artifact
    /// (`.rra`) summary (seed + determinism pins + model pin + honest
    /// guarantee level + per-gate verdict manifest + blake3 root). Driver at
    /// crates/cli/src/rra_cmd.rs. Read-only; never re-executes the run.
    Rra {
        action: RraAction,
        output_format: CliOutputFormat,
    },
    /// G-2: `elai replay --verify <path>` — verify a `.rra` offline.
    /// Recomputes the blake3 root (tamper check) + re-derives every event
    /// content hash (divergence check) and prints the [`ReplayVerdict`]
    /// (Verified / Divergence@event / Tampered). Driver at
    /// crates/cli/src/rra_cmd.rs. Read-only; honest verify-divergence scope
    /// (GD-5) — it does NOT bit-identically replay token streams.
    Replay {
        action: ReplayAction,
        output_format: CliOutputFormat,
    },
    /// H-35: `elai rewind <target> [--scope conversation|code|both] [--summarize]`
    /// — restore session state to a prior SSB seq. Drives compact_session
    /// (conversation trim), recover_stubs (stub rehydrate), ssb_resume_from
    /// (code restore, fail-closed). Emits one `session_rewound` SSB beacon per
    /// successful rewind. Idempotence guard: target==head emits NO row.
    /// Driver at crates/cli/src/rewind_cmd.rs. DEFAULT-ON;
    /// kill-switch ELAI_REWIND_DISABLE.
    Rewind {
        action: RewindAction,
        output_format: CliOutputFormat,
    },
    /// H-49: `elai undo / redo / history / jump` — CAS-backed reversible LLM
    /// edits. undo/redo/jump walk the undo-tree HEAD over the SessionDag.
    /// Default-ON; kill-switch ELAI_TIMETRAVEL_DISABLE.
    /// Driver at crates/cli/src/timetravel_cmd.rs.
    TimeTravel {
        action: TimeTravelAction,
        output_format: CliOutputFormat,
    },
    /// G-5: `elai review --cockpit [--session <id>] [--base <ref>] [--json]` —
    /// the reviewer-cockpit Diff-Surface-to-Review (DSR) producer. Parses the
    /// working-tree git diff against a deterministically pinned base, reads the
    /// session's SSB vouch stream, computes the DSR (un-vouched changed lines /
    /// total changed lines), prints the vouched/un-vouched bands + the headline
    /// DSR, and emits ONE `review_surface_scored` row through the live session
    /// `Arc<Ssb>`. Driver at crates/cli/src/review_cmd.rs. Default-ON; the only
    /// opt-out is the kill-switch `ELAI_REVIEW_COCKPIT_DISABLE`.
    Review {
        action: ReviewAction,
        output_format: CliOutputFormat,
    },
    /// H-53: `elai prebind [--demo-prebind] [--allow-freeform-citations]` —
    /// citation pre-binding + constrained synthesis on the /research|/ask
    /// answer path. `--demo-prebind` is the H-53 W-1 seeded-fixture trigger
    /// (the observable_effect proof): builds a deterministic evidence set,
    /// calls `freeze_anchors`, runs closed-set membership + blake3-support
    /// check (Bound/Ghost/Stale outcomes), and emits a genuine
    /// `CitationAnchorsBound` SSB row. `--allow-freeform-citations` is the
    /// BROKEN-ARM probe (R2 proof): bypasses closed-set check and fires the
    /// Blocked arm. Default-ON kill-switch: ELAI_CITATION_PREBIND_DISABLE.
    /// Handler at crates/cli/src/prebind_cmd.rs.
    Prebind {
        allow_freeform: bool,
        output_format: CliOutputFormat,
    },
    /// G-8: `elai verify-attestation <commit>` — re-verify a signed in-toto v1
    /// provenance attestation OFFLINE, without re-running ELAI (trust-without-
    /// re-running, SLSA-VSA). Reads the digest-only `ELAI-Provenance-Digest`
    /// commit trailer + the sidecar `attestation.intoto.json`, recomputes the
    /// blake3-Merkle root over the predicate, and confirms it equals both the
    /// subject digest and the trailer (the F-gate verdict hashes are
    /// Merkle-bound). Driver at crates/cli/src/verify_attestation_cmd.rs.
    /// READ-ONLY (opens no Ssb); exits non-zero on not-verified so CI can gate;
    /// a git-absent / missing-trailer / missing-sidecar condition is an explicit
    /// error, never a fabricated "verified".
    VerifyAttestation {
        commit: String,
        output_format: CliOutputFormat,
        /// I-18: `--chain` flag. When set, walks the per-row blake3 chain in
        /// the SSB alongside the attestation re-verification and reports the
        /// first broken link (sequence, expected_prev_hash, found_prev_hash).
        /// Default: false (backward-compatible; existing CI invocations without
        /// `--chain` continue to work on LEGACY receipts).
        chain: bool,
        /// I-45: `--current-objective <text>` threads the caller's current
        /// objective text into `verify_attestation_against_trailer_with_objective`
        /// so `AttestationVerdict::ReceiptStale` can fire on a live CLI run
        /// (GAP-1). `None` preserves the legacy `ObjectiveUnchecked` behavior
        /// (non-fatal, exit 0, per D-016).
        current_objective: Option<String>,
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
    /// E-43 -- `elai skills-mgmt <list|eval|publish>` agent skill systems
    /// surface (frontmatter + eval/ + publish dist). Sibling to the
    /// existing slash-command `Skills` variant. Handler:
    /// crate::skills_cmd. ABSORBs the E-22 SKILL.md export shim.
    SkillsManagement {
        action: SkillsAction,
        output_format: CliOutputFormat,
    },
    /// E-39 -- `elai cloud override-cap <USD> --confirm <STRING>`.
    /// Layered-cap manual override CLI surface. Requires a typed-
    /// confirmation string (typically the override USD amount as a
    /// word, e.g. "fifty-dollars") to prevent accidental override.
    /// Emits a [`CostEventKind::LayeredCapManualOverride`] SSB row
    /// carrying `user_session_id` + `override_amount_usd` +
    /// `confirmation_string_hash` -- composes with E-38 EU AI Act
    /// audit-trail liveness gate. Handler: `crate::cloud_cmd`. See
    /// `workflow/reasoning/E-39-decisions.md`.
    Cloud {
        action: CloudAction,
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
    /// E-35 — `elai structured-output <sub>` — structured-output substrate
    /// operator surface. Anchors the host-side substrate
    /// (`codeact_sandbox::execute_code_act`,
    /// `CloudGroundingClient::mark_strict_schema_applied`, the 10 E-35
    /// SSB beacons, the `GrammarCompilePanel` placeholder producer) as
    /// production-callable so they are not WIRE-DARK per
    /// E40_WIRE_DARK_AVOIDANCE_VIA_CLI. Python sidecar wiring
    /// (XGrammarLogitsProcessor + tool_parser.py) is RUNPOD-PENDING and
    /// owned by a separate session. Handler at
    /// crates/cli/src/structured_output_cmd.rs. See
    /// `workflow/reasoning/E-35-decisions.md`.
    StructuredOutput {
        action: StructuredOutputAction,
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
    /// E-53: `elai formal-verify <run|status>` — formal-verify
    /// pipeline CLI surface driven by `crate::formal_verify_cmd`.
    /// Composes the orchestrator's `formal_verify` module (which
    /// composes C-50 `contract_coding` + C-52 `dafny_verify`) and
    /// emits the `formal_verifier_completed` SSB beacon at the
    /// pipeline boundary. `run` dispatches a single verifier against
    /// a target; `status` prints the available verifier toolchains +
    /// each one's RUNPOD-PENDING status. Live Z3 / Kani / Verus
    /// dispatch is RUNPOD-PENDING; Dafny dispatch reuses C-52
    /// `invoke_dafny_cli` and surfaces toolchain absence as
    /// `RunpodPending`. Gemma-4 BLOCKED per memory
    /// `feedback_no_gemma4`.
    FormalVerify {
        action: FormalVerifyAction,
        output_format: CliOutputFormat,
    },
    /// E-8: `elai optimize template --target <agent_role> --train PATH
    /// --val PATH [--reflection-model sonnet-4.6] [--budget-tokens N]`.
    /// Self-evolving prompts driver. Spawns the Python
    /// `elai_optimizer` service (GEPA primary per arXiv:2507.19457 +
    /// DSPy MIPROv2 / OPRO surfaces); refuses local 7B as reflection
    /// LM per arXiv:2405.10276 (Zhang et al.). Live LLM calls
    /// RUNPOD-PENDING (CLI parses + dispatches; the Python side reads
    /// the spec + emits OptimizedTemplate to disk; the orchestrator
    /// admit path consumes the JSON). See
    /// workflow/reasoning/E-8-decisions.md.
    Optimize {
        action: OptimizeAction,
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
    /// D-23: `elai bench sycophancy [--fixtures PATH] [--output PATH]
    /// [--detection-floor 0.70] [--fp-ceiling 0.15]`. Runs the
    /// sycophancy detection benchmark against the labelled fixture
    /// corpus. Exits 0 on Pass, 1 on Fail. JSON report written to
    /// `--output` (default stdout). Owns its own tokio runtime
    /// (sync entrypoint pattern — mirrors `bench_prmbench`). Floor and
    /// ceiling carried as String (CliAction derives Eq; f64 does not)
    /// and parsed at the handler boundary.
    BenchSycophancy {
        fixtures: Option<String>,
        output: Option<String>,
        detection_floor: Option<String>,
        fp_ceiling: Option<String>,
    },
    /// D-24: `elai bench canary-regression [--fixtures PATH] [--output
    /// PATH] [--harmful-block-floor 20] [--helpful-block-ceiling 5]
    /// [--neutral-block-ceiling 2]`. Runs the cross-domain canary
    /// regression benchmark against the labelled fixture corpus
    /// (20 Harmful + 50 Helpful + 30 Neutral). Exits 0 on Pass, 1 on
    /// Fail. JSON report written to `--output` (default stdout). Owns
    /// its own tokio runtime (sync entrypoint pattern — mirrors
    /// `bench_sycophancy`). Thresholds carried as String (CliAction
    /// derives Eq) and parsed at the handler boundary.
    BenchCanary {
        fixtures: Option<String>,
        output: Option<String>,
        harmful_block_floor: Option<String>,
        helpful_block_ceiling: Option<String>,
        neutral_block_ceiling: Option<String>,
    },
    /// D-25: `elai bench tamper [--scenarios PATH] [--output PATH]
    /// [--fail-fast=<true|false>] [--max-wall-ms-per-scenario N]`.
    /// Runs the determinism + archive tamper-suite benchmark against
    /// the 10-scenario JSON-manifest matrix at
    /// `benchmarks/tamper/scenarios/` driving D-8 `verify_archive`
    /// IN-PROCESS. Exits 0 on Pass (every scenario matched expected
    /// outcome AND max_wall_ms <= 5000), 1 on Fail. JSON report
    /// written to `--output` (default stdout). Owns its own tokio
    /// runtime (sync entrypoint pattern — mirrors `bench_canary`).
    /// `fail_fast` is reserved (the runner selects ff mode per
    /// manifest); parsed for forward compat. Wall budget carried as
    /// String (CliAction derives Eq) and parsed at the handler
    /// boundary.
    BenchTamper {
        scenarios: Option<String>,
        output: Option<String>,
        fail_fast: Option<String>,
        max_wall_ms_per_scenario: Option<String>,
    },
    /// H-34 (SCHEMA_VERSION 213): `elai bench mcp-schema-budget [--json]`.
    /// MCP-schema token-budget MEASURE-FIRST benchmark. Drives the embedded
    /// multi-server MCP tool catalog (20 tools: local-rag / pod-lifecycle /
    /// notion) OFFLINE and computes baseline_schema_tokens_per_turn (all tools
    /// eager) vs. injected_schema_tokens_per_turn (floor-only defer-load minimum).
    /// Both token counts are COMPUTED from the corpus, never literals.
    /// Writes a timestamped mcp-schema-budget-<unix_ms>.json report under
    /// `<cwd>/benchmarks/mcp_schema_budget/` and emits exactly one
    /// `SsbActionType::McpSchemaBudgetMeasured` beacon (default-ON; no
    /// `ELAI_*_ENABLE` gate). `--json` prints the full report to stdout.
    /// Exits 0 on a clean run, 1 on error.
    BenchMcpSchemaBudget {
        json: bool,
    },
    /// F-2b: `elai bench seccodeplt [--json]`. Drives the REAL ELAI static
    /// write-gate (`vibegate::evaluate_vibegate`) over a SeCodePLT-structured
    /// CWE corpus OFFLINE (no model, no sandbox) and computes a before/after
    /// insecure-rate + gate recall. Writes a timestamped
    /// `seccodeplt-<unix_ms>.json` report under `<cwd>/benchmarks/seccodeplt/`
    /// and emits exactly one `SsbActionType::InsecureRateMeasured` beacon
    /// (W-2: always-emitted on run; no `ELAI_*_ENABLE` gate). With `--json`
    /// the full report JSON is also printed to stdout. Exits 0 on a clean
    /// run, 1 on infrastructural error. Owns its own tokio runtime
    /// (`Ssb::new` constructed inside `block_on` per the F-2b playbook).
    BenchSeccodeplt {
        json: bool,
    },
    /// I-9 (SCHEMA_VERSION 263): `elai bench error-amplification [--fixture PATH]
    /// [--seed <u64>] [--json] [--no-repair]`. The error-amplification (alpha_t)
    /// telemetry harness. Injects a deterministic byte-perturbation at one step
    /// of a frozen pipeline fixture (IN-HARNESS, NOT C-28 cargo-mutants), walks
    /// the downstream window classifying each step RECOVERED vs PROPAGATED,
    /// computes alpha_t (the downstream propagated fraction) + recovered fraction
    /// + the active-annealing temperature, writes a timestamped
    /// `error-amplification-<unix_ms>.json` report under
    /// `<cwd>/benchmarks/error_amplification/`, and emits exactly one
    /// `SsbActionType::ErrorAmplificationMeasured` beacon (default-ON; no
    /// `ELAI_*_ENABLE` gate). `--fixture` overrides the embedded fixture (default:
    /// the compile-time `pipeline_v1.json`); `--seed` pins the perturbation seed
    /// (default: the `ELAI_SEED` path); `--no-repair` runs the R2 negative control
    /// (recovery disabled => alpha == 1.0). With `--json` the full report JSON is
    /// also printed. Exits 0 on a clean run, 1 on infrastructural error. Owns its
    /// own tokio runtime (`Ssb::new` inside `block_on` per the F-2b playbook).
    BenchErrorAmplification {
        fixture: Option<String>,
        seed: Option<u64>,
        json: bool,
        no_repair: bool,
    },
    /// H-30 (SCHEMA_VERSION 239): `elai bench staleness [--json]
    /// [--broken-retrieval]`. The staleness / intent-drift INSTRUMENT. Drives
    /// the SHIPPED bi-temporal memory substrate (FactStore + audit_conflicts +
    /// get_relevant_facts, REUSE GD-1) over a labeled supersession fixture in an
    /// ISOLATED in-memory store and computes two grounded STALE
    /// (arXiv:2605.06527) rates: SR (recognition: the stale row was closed) and
    /// IPA (application: retrieval served the fresh value, never the stale one),
    /// invariant SR >= IPA. Writes a timestamped `staleness-<unix_ms>.json`
    /// report under `<cwd>/benchmarks/staleness/` and emits exactly one
    /// `SsbActionType::StalenessEvalMeasured` beacon (default-ON; no
    /// `ELAI_*_ENABLE` gate). `--broken-retrieval` runs the negative control (a
    /// retriever that reads include-closed rows) so `stale_fact_reached_prompt >
    /// 0` -- R2: the instrument screams. With `--json` the full report JSON is
    /// also printed. Exits 0 on a clean run, 1 on infrastructural error. Owns
    /// its own tokio runtime (`Ssb::new` inside `block_on`).
    BenchStaleness {
        json: bool,
        broken_retrieval: bool,
    },
    /// G-6 (SCHEMA_VERSION 217): `elai bench safedelta [--suites <csv>]
    /// [--seed <u64>] [--json]`. Runs the gate-ON-vs-gate-OFF SAFEDELTA
    /// ablation over the embedded SeCodePLT corpus under the pinned seed.
    /// DERIVES the delta from the SHIPPED SecCodePltRunner::run (GD-1);
    /// re-implements NO part of F-2. Emits ONE SafetyDeltaMeasured SSB beacon
    /// + writes a timestamped JSON manifest under benchmarks/safedelta/ + a
    /// signed replayable .rra manifest via G-2 rra.rs (blake3-only root).
    /// Tier-1 = static-gate only; Tier-2 live dynamic exploit-exec arm is
    /// RUNPOD-PENDING (BaxBench-392 arXiv:2502.11844, SUSVIBES-200 arXiv:2512.03262).
    BenchSafedelta {
        /// Comma-separated suite list (e.g. `seccodeplt`). Tier-1 only supports
        /// `seccodeplt`; additional suites require a live RunPod endpoint.
        suites: Vec<String>,
        /// Deterministic RNG seed (mirrors ELAI_SEED; default 0).
        seed: u64,
        /// Print the full report JSON to stdout.
        json: bool,
        /// P1.a Tier-2 skeleton: when present, drive one live SidecarRuntime
        /// completion against this endpoint before the static ablation.
        endpoint: Option<String>,
        /// P1.a Tier-2 skeleton: model id for the live leg. None = sidecar
        /// default.
        model: Option<String>,
        /// PS-1: drive the live 4B lane (:8181) instead of the static-gate
        /// ablation. Emits `SafetyDeltaMeasured` with `live=true` + lane tags.
        live: bool,
        /// PS-1: before/after protocol selector. Currently only
        /// `safety-preamble-off-on` is implemented; other values return a
        /// CLI parse error. `None` defaults to `safety-preamble-off-on`.
        protocol: Option<String>,
    },
    /// PS-3 (SCHEMA_VERSION 277): `elai bench hallucination-delta [--suites <csv>]
    /// [--seed <u64>] [--json] [--endpoint <url>] [--model <id>]`.
    /// FAITHFUL MIRROR of G-6 safedelta: gate-ON-vs-gate-OFF hallucination-delta
    /// ablation over the embedded fixture corpus. Emits ONE
    /// HallucinationDeltaMeasured SSB beacon + writes a timestamped JSON
    /// manifest under benchmarks/hallucination_delta/ + a signed replayable
    /// .rra manifest via G-2 rra.rs (blake3-only root). Tier-1 = fixture only;
    /// Tier-2 live CGG ablation is RUNPOD-PENDING.
    BenchHallucinationDelta {
        /// Comma-separated suite list (e.g. `cgg`). Tier-1 only supports `cgg`;
        /// additional suites require a live RunPod endpoint.
        suites: Vec<String>,
        /// Deterministic RNG seed (mirrors ELAI_SEED; default 0).
        seed: u64,
        /// Print the full report JSON to stdout.
        json: bool,
        /// Tier-2 skeleton: when present, rejected in this fixture-tier slice
        /// (live CGG ablation is RunPod-gated).
        endpoint: Option<String>,
        /// Tier-2 skeleton: model id for the live leg. None = sidecar default.
        model: Option<String>,
    },
    /// PS-READINESS-E2ERUNNER Slice B: `elai bench local-lane-e2e
    /// [--endpoint <url>] [--model <id>] [--json]`. The ONLY live consumer of
    /// the 4B sidecar at http://127.0.0.1:8181. Health-checks the sidecar,
    /// asks the 4B to generate a small Rust function, splices it into a fixed
    /// test harness, compiles + runs it, and emits ONE signed off-agent
    /// artifact (.rra) carrying the REAL compile + test exit codes + a lane
    /// tag (model + host). Emits one LocalLaneE2eMeasured SSB beacon. A
    /// missing/unreachable sidecar is a hard FAIL (R1), never a synthesized
    /// pass.
    BenchLocalLaneE2e {
        /// Print the full payload JSON to stdout.
        json: bool,
        /// Sidecar endpoint. Defaults to http://127.0.0.1:8181 (the 4B lane).
        endpoint: Option<String>,
        /// Model id for the 4B sidecar lane. Defaults to qwen3.5-4b-q4_k_m.
        model: Option<String>,
    },
    /// K-18a (PS-5 observe arm): `elai bench case-reuse [--suite fixture]
    /// [--seed <u64>] [--json]`. Recurrence-hit-rate over the SHIPPED BM25
    /// retrieval stack: for each fixture recurrence, does the shipped retrieval
    /// find the paired proven case (recurrence_hit_rate@{1,3,5}) + the
    /// false-positive control (false_match_rate@k). ZERO reuse-into-generation
    /// (K-18 Task 2 is BENCH-PENDING on the unbuilt VCG J-track). Writes a
    /// blake3-pinned JSON report under benchmarks/case_reuse/. Emits 1
    /// RecurrenceHitRateMeasured SSB flat beacon + a signed .rra manifest
    /// per clean run (SCHEMA_VERSION 270, K-18a follow-on).
    BenchCaseReuse {
        /// Suite to run. Only `fixture` is runnable today; the real-journal
        /// `journal` suite is F-2 deferred.
        suite: String,
        /// Deterministic RNG seed (mirrors ELAI_SEED; default 0).
        seed: u64,
        /// Print the full report JSON to stdout.
        json: bool,
    },
    /// K-2: `elai bench routing-economics [--seed <u64>] [--json]
    /// [--force-cloud-everything]`. Compares the shipped hybrid routing policy
    /// with a force-cloud control over deterministic UsageTurnEvent fixture rows,
    /// emits RoutingEconomicsMeasured, and seals an RRA. The live lane remains
    /// RunnerPresent with an explicit unavailable-cost label until funded.
    BenchRoutingEconomics {
        seed: u64,
        json: bool,
        force_cloud_everything: bool,
    },
    /// K-9: `elai bench model-refresh [--lane <coding|vision|ocr|embedding_reranker>] [--seed <u64>] [--json]`.
    BenchModelRefresh {
        lane: Option<String>,
        seed: u64,
        json: bool,
    },
    /// K-4 (PS-2 owner, KEYSTONE of K-T3): `elai bench gate-tax
    /// [--seed <u64>] [--json] [--enforce-ceiling]`. Composed gate-tax budget:
    /// the in-band CODING trust-spine gate friction (11 FSM EvidenceGate
    /// family + T1/T2/T3 hook tiers) plus a benign-corpus over-block sweep
    /// (FIX-4) and the RIDER-b gate-manifest coverage table. Falls to a
    /// labeled fixture composition (never a fabricated MEASURED) on a dry
    /// journal. Writes a JSON report under benchmarks/gate_tax/. Emits 1
    /// GateTaxMeasured SSB flat beacon + a signed .rra manifest per clean run
    /// (SCHEMA_VERSION 271). `--enforce-ceiling` (MOVE 3/6, SP2) is the
    /// actual "release gate" exit-code contract — CEILING-PENDING[K-4] fails
    /// closed until a human confirms a real ceiling (ABORT-5).
    BenchGateTax {
        /// Deterministic seed (plumbing parity with the bench-family CLI
        /// shape; does not drive RNG in this runner).
        seed: u64,
        /// Print the full report JSON to stdout.
        json: bool,
        /// MOVE 3/6 (SP2): apply the ceiling-breach exit-code contract.
        enforce_ceiling: bool,
    },
    /// G-10 (SCHEMA_VERSION 218): `elai model-drift baseline|check|show
    /// [--project-root PATH] [--json]` — exogenous model-version behavioral-
    /// regression gate CLI. DISTINCT from `elai drift` (E-45 cross-language
    /// SOURCE drift). Default-ON; kill-switch ELAI_MODEL_DRIFT_DISABLE only (W-2).
    /// Composes D-7 canary.rs (GD-1: re-implements NONE). See
    /// workflow/reasoning/G-10-decisions.md.
    ModelDrift {
        action: ModelDriftAction,
        output_format: CliOutputFormat,
    },
    /// H-41 (SCHEMA_VERSION 219): `elai checker-gaps [--limit N] [--json]` —
    /// aggregate `checker_gap_detected` SSB rows into a ranked backlog of
    /// claim-class x complexity-taxon pairs that fell to a non-Constructive
    /// grounding mode for lack of a Deterministic verifier. Uses the Xu
    /// arXiv:2401.11817 Table-3 taxonomy to flag NP-complete / undecidable
    /// taxa as HIGH-priority (never-LLM-grade, RD-3). Production read surface
    /// over the off-agent `CheckerGapDetected` beacon. Requires
    /// ELAI_CHECKER_GAP_ENABLE=1 on the writer side.
    CheckerGaps {
        /// Maximum backlog rows to return (default 50).
        limit: Option<usize>,
        /// Print as JSON instead of a human-readable table.
        json: bool,
    },
    /// J-28 T8: `elai usage [--json]` — grouped cloud/local token-usage
    /// rollup (per model/task/session/lane), cloud and local SEPARATELY
    /// LABELED. Production read surface over `UsageTurnEvent` SSB rows
    /// emitted at `grounding_client.rs` (cloud) + `runtime_orchestrator.rs`
    /// (local). Handler at `crates/cli/src/usage_cmd.rs`.
    Usage {
        /// Print as JSON instead of a human-readable table.
        json: bool,
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
    /// dkg-stream-rigorous Task 2: `elai index build [--project-root PATH] [--vectors]`.
    /// Non-interactive full index build (BM25 + DKG + optional streaming
    /// vectors) that writes to the CANONICAL `.elai_cc/indexes/projects/default/bm25`
    /// + `dkg.db` + `vector_index.json` paths the query-time stack (and the
    /// `bench rag` eval seam) read from. `--vectors` sets `ELAI_BULK_VECTOR_BUILD=1`
    /// for this run so the (memory-bounded, streaming) vector stage is populated.
    IndexBuild {
        project_root: PathBuf,
        vectors: bool,
        json: bool,
    },
    /// I-34: `elai bench crane [--crane-fixture PATH] [--seed <u64>]`.
    /// Deterministic CRANE hybrid grammar-pin gate fixture harness. Drives the
    /// three-layer I-34 gate (eligibility + blake3 pin verify + post-parse
    /// re-validation) over an embedded fixture without live inference (W-10
    /// PENDING checkbox). Emits `GrammarGateEvent` SSB rows for each of the
    /// three Kind arms (CranePassed / PinViolation / PostParseRejected) and
    /// exits 0 on a clean run or 1 on infrastructure error.
    BenchCrane {
        crane_fixture: Option<String>,
        seed: Option<u64>,
    },
    // I-27: `elai bench convergence --suite <PATH> [--seeds <u32>] [--seed <u64>] [--json]`
    // Verified-outcome-rate vs compute convergence curve fixture harness. Statistical
    // floor enforced mechanically (seed_count>=5, paired=true). Emits ONE
    // ConvergenceCurveMeasured SSB flat beacon per clean run. Live per-node
    // compute-spend leg pending I-4 (WIRE-DARK[I-27]).
    BenchConvergence {
        suite: std::path::PathBuf,
        seeds: u32,
        json: bool,
    },
    // J-34 leg-1: `elai bench context-fill --suite <PATH> [--seeds <u32>] [--json]`
    // Verified-outcome-rate vs context-window fill fraction curve fixture
    // harness. Statistical floor enforced mechanically (seed_count>=5,
    // paired=true). Builds a FillDegradationCurve, constructs a
    // CalibrationArtifact, and emits ONE ContextCalibrationEvent(CurveMeasured)
    // SSB row via CalibrationArtifactStore::insert_with_ssb (slice-2 seam).
    // Live per-model inference leg deferred (WIRE-DARK[J-34.leg1-live]).
    BenchContextFill {
        suite: std::path::PathBuf,
        seeds: u32,
        json: bool,
    },
    // J-34 leg-3: `elai bench positional-probe --suite <PATH> [--seeds <u32>] [--json]`
    // Positional-value curve fixture harness. Fills a PositionalValueCurve at
    // depth fractions 0.0 / 0.25 / 0.5 / 0.75 / 1.0. Scores deterministically
    // by gate verdicts (never LLM-judged). Emits ONE
    // ContextCalibrationEvent(CurveMeasured) SSB row via
    // CalibrationArtifactStore::insert_with_ssb (slice-2 seam). Live per-model
    // inference leg deferred (WIRE-DARK[J-34.leg3-live]).
    BenchPositionalProbe {
        suite: std::path::PathBuf,
        seeds: u32,
        json: bool,
    },
    // I-38: `elai bench qdelta --config <PATH> [--seed <u64>] [--json]`
    // QDELTA defective-diff rate fixture harness. Statistical floor enforced
    // mechanically (seed_count>=5, paired=true). Emits ONE QdeltaMeasured SSB
    // flat beacon per clean run. Live-model leg deferred (D-026).
    BenchQdelta {
        config: std::path::PathBuf,
        seed: u64,
        json: bool,
    },
    // C-37: `elai bench snowballing --pool A|B --output <PATH>` — Structure
    // Snowballing harness. Constrained-decoding 5-class Reflector replication.
    BenchSnowballing {
        pool: String,
        n_samples: Option<i64>,
        model: Option<String>,
        max_trials: Option<i64>,
        reflector_mode: Option<String>,
        sidecar_url: Option<String>,
        output: String,
        python_bin: Option<String>,
    },
    // I-43: `elai bench claim-scope [--suite <fixture>] [--seed <u64>] [--json]`
    // Reproducibility-claim scoper + cheap pre-falsification kill-gate.
    // Runs pilot (gate-ON vs gate-OFF, reviewer-reaching trap detector) and
    // divergence scoper (local vs cloud action-stream divergence). Fixture leg only;
    // live-model leg deferred (D-026). Emits ClaimScopeMeasured SSB beacons.
    BenchClaimScope {
        suite: Option<String>,
        seed: u64,
        json: bool,
    },
    // I-33: `elai bench governance-probe [--seed <u64>]`
    // Mechanical decision governance fixture harness (CEFL / I6Q / commit-reveal).
    // Exercises all three GovernancePrimitiveFired Kind arms without a live LLM:
    // framing-inversion (CeflExternalized), vacuous-rationale (I6qRejected),
    // out-of-order-commit (CommitRevealSealed). Exits 0 on a clean run or 1 on
    // infrastructure error. Wiring-enforcement R1/R4 entry point for I-33.
    BenchGovernanceProbe {
        seed: u64,
    },
    /// I-32: `elai role-ab-probe [--seed N]` — run the role-contract A/B harness.
    /// Loads fixtures from crates/contracts/tests/fixtures/role_contracts/,
    /// assembles system-prompt sections with and without role contracts for each
    /// affected role, computes pass-rates, emits `RoleContractApplied(AbDelta)` SSB rows.
    /// Evidence gate (ID-5): new_rate >= old_rate required; exits non-zero on regression.
    RoleAbProbe {
        seed: u64,
    },
    /// I-35: `elai bench sequence-probe [--seed N]` — run the tool-sequence enforcement
    /// bench harness. Replays 3 mock fixtures (compliant / order-violation / halt-enforced)
    /// against the builtin SequenceRuleSet, verifies expected verdicts, emits
    /// `ToolSequenceVerdict` SSB rows. Exits 0 on all-pass, non-zero on any mismatch.
    BenchSequenceProbe {
        seed: u64,
    },
    /// J-24 MS4: `elai bench steering-vs-prompt --task <fixture> [--seed <u64>] [--json]`
    /// Deterministic fixture scoring of a steered arm vs a role-prompt baseline.
    /// Emits `SteeringEffectMeasured` SSB row. No live LLM judge (TD-3).
    BenchSteeringVsPrompt {
        task: std::path::PathBuf,
        seed: u64,
        json: bool,
    },
    /// J-72 slice D: `elai bench retrieval-adapt --gold <path> [--k <N>]
    /// [--baseline-ranks <json>] [--adapted-ranks <json>] [--out <report.json>]
    /// [--json]` — Tier-1 MRR/Recall harness for baseline vs adapted slot
    /// delta over a gold set. Deterministic. Never LLM-judge.
    BenchRetrievalAdapt {
        gold: PathBuf,
        k: u32,
        baseline_ranks: Option<PathBuf>,
        adapted_ranks: Option<PathBuf>,
        out: Option<PathBuf>,
        json: bool,
    },
    /// J-75: deterministic measure-first RRF component ablation report.
    BenchRrfAblate {
        gold: PathBuf,
        k: u32,
        full_ranks: Option<PathBuf>,
        drop_pagerank_ranks: Option<PathBuf>,
        drop_second_dense_ranks: Option<PathBuf>,
        nav_only_graph_ranks: Option<PathBuf>,
        out: Option<PathBuf>,
        json: bool,
    },
    /// J-77 I11: `elai bench perception-faithfulness [--fixture PATH] [--json]
    /// [--enforce]` — 6-metric perception-faithfulness fixture harness.
    /// Drives the REAL `run_perception_faithfulness_on_fixture` orchestrator
    /// runner (fuse_detections + filter_ghost_regions). Writes a timestamped
    /// JSON report under `benchmarks/perception_faithfulness/`. No SSB beacon
    /// (fixture-only thin bench). `--enforce` exits non-zero on metric fail.
    BenchPerceptionFaithfulness {
        fixture: PathBuf,
        json: bool,
        enforce: bool,
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
        count_only: bool,
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
    /// F-8 / Slice A4: `elai self-mod apply --proposal-fixture PATH
    /// [--no-env-override]`. The SCF symbolic-conflict detector's W-3
    /// production entrypoint + W-5 live trigger. Drives a real `ConfigVariant`
    /// (loaded from the fixture) through the REAL
    /// `elai_orchestrator::screen_config_variant_for_conflicts` compose unit,
    /// emitting `symbolic_conflict_detected` SSB beacon(s) and HOLDING the
    /// candidate pending on conflict (WARN-then-escalate, never fail-closed).
    /// Handler: `crate::self_mod_cmd::run`. See
    /// `workflow/reasoning/F-8-decisions.md`.
    SelfMod {
        action: SelfModAction,
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
    /// D-28: `elai farm-adv-rm <run|verdict>` — adversarial example
    /// farming runner CLI surface. Drives
    /// `elai_orchestrator::farm::adv_rm::AdvRmFarmer` which walks a seed
    /// dataset, applies the Z>1.96 dual-filter, and writes
    /// `corpus.jsonl` with `gold_verdict: "pending"`. Live PPM scoring
    /// loop + Bradley-Terry retrain + dual `bench_prmbench` v1/v2 runs
    /// are RUNPOD-PENDING (see `python/elai_inference/ppm_adversarial.py`
    /// and `workflow/remote-setup/ppm_finetune.sh`). Emits one
    /// `SsbActionType::AdversarialFarmRunCompleted` SSB row at end-of-
    /// run. Single new variant per workflow/reasoning/D-28-decisions.md
    /// DECISION-2.
    FarmAdvRm { sub: FarmAdvRmSub },
    // E-10: hostile-critic structural simplifications -- bench ablation /
    // tier eval / critic report CLI dispatchers. See
    // workflow/reasoning/E-10-decisions.md.
    BenchAblation { tile: String, fixtures: String, budget: u32 },
    /// E-7 — `elai darwin-loop [--generations N] [--mutants K] [--seed S]
    /// [--traps T] [--gamma G]` — exercises the darwinian-evolve substrate
    /// (DiscountedTsConfig + SuccessiveRejects + trap_tasks) as a single
    /// CLI surface. Wire-dark: no SsbActionType emission, no live sidecar
    /// call. See workflow/reasoning/E-7-decisions.md DECISION-13.
    DarwinLoop {
        generations: u32,
        mutants: u32,
        seed: u64,
        traps: u32,
        gamma: String,
    },
    TierEval { suites: Vec<String>, budget: u32 },
    CriticReport { output: Option<String> },
    /// E-6: `elai mast report [--session <id>] [--limit N] [--json]` --
    /// deterministic MAST failure-mode report read from `audit_mast_tagged`
    /// SSB rows (read-only). `session` is an operator run label echoed in
    /// the header; `limit` caps the scan window. See mast_cmd.rs.
    Mast { session: Option<String>, limit: Option<usize>, json: bool },
    /// E-47: `elai lora-gate-check [--config PATH] [--json]` -- ship-gate
    /// dispatcher for the LoRA fine-tuning subsystem. Reads the
    /// `[finetune.lora]` block from `.elai_cc/elai_cc.toml`, evaluates
    /// the triple-AND gate, prints the outcome, and emits a
    /// `lora_ship_gate_event` SSB row when
    /// `ELAI_LORA_SHIP_GATE_ENABLE=1`. Producer-side LoRA training stack
    /// (Unsloth + QLoRA-NF4 + PiSSA + DoRA) is RUNPOD-PENDING.
    LoraGateCheck { config: Option<String>, json: bool },
    /// E-9: `elai eval run --benchmark=<NAME> [--harness-vary] [--p-target=0.05]
    /// [--list] [--json]` — external agent-eval benchmark dispatcher.
    /// Drives the host-side scaffolding for 11 external benchmark suites
    /// (swe-bench-pro / swe-bench-live / livecodebench-v6 / tau2-bench /
    /// bfcl-v3 / gaia-l1-l2 / clawsbench-split / refinebench /
    /// sycophancy-probes / math-floors / deep-planning). Live model evals
    /// RUNPOD-PENDING; the CLI surface ships the registry + power-
    /// calibration refusal contract + SSB emission boundary. See
    /// workflow/reasoning/E-9-decisions.md DECISION-8.
    /// `p_target` carried as String (CliAction derives Eq; f64 does not).
    EvalRun {
        benchmark: Option<String>,
        harness_vary: bool,
        p_target: Option<String>,
        list: bool,
        json: bool,
    },
    /// E-55: `elai rl train --recipe=<name>` — agentic RL recipe runner
    /// surface. Dispatches one of the deferred-ladder rungs by recipe
    /// name (rlaif_judge / gepa_evolve / reward_fusion / mona_canary /
    /// sft_rollout_collect / deferred_unlock_check). Substrate ships
    /// fully; the actual cloud Sonnet rollouts + LoRA training are
    /// RUNPOD-PENDING and gated behind `ELAI_AGENTIC_RL_ENABLE=1`.
    /// See workflow/reasoning/E-55-decisions.md.
    RlTrain {
        recipe: Option<String>,
        list: bool,
        json: bool,
    },
    /// E-25 Tranche C (C.4): `elai checkpoint <list|rewind>` -- edit
    /// snapshot store CLI surface. Lists existing per-session
    /// snapshots OR rewinds the last N edits in the current session
    /// from the on-disk content-addressed store at
    /// `.elai_cc/.session-checkpoints/`. Wire-dark default OFF;
    /// gated by `ELAI_E25_EDIT_CHECKPOINTS_ENABLE=1` for SSB emit.
    /// The CLI surface itself works regardless of the gate so the
    /// operator can inspect existing snapshots without enabling the
    /// emit cadence. See workflow/reasoning/E-25-trancheC-decisions.md.
    CheckpointSub {
        sub: CheckpointSub,
    },
    /// F-9 / Slice D4: `elai dep-check <path> [--offline]` — write-time
    /// anti-slopsquatting dep-check CLI surface. Reads the file at
    /// `path`, extracts dependency import lines, runs
    /// `DepWriteTimeGate.analyze_collect`, prints one BLOCK line per
    /// `NotFound`/`TooNew` finding, and emits `WriteTimeSlopsquatBlocked`
    /// SSB rows (durably) to the canonical session `ssb.db`.
    ///
    /// DEFAULT mode (no flag) = `SubprocessRegistryChecker` — the REAL live
    /// network detection leg (W-10/W-5 gated). `--offline` =
    /// `StubRegistryChecker` — the deterministic demo/contract harness that
    /// exercises the same gate code path (offline mode is safe on real code,
    /// only curated known-hallucinated names Block; WARN-3).
    /// Handler: `crate::dep_check_cmd`.
    DepCheck {
        /// File path to scan for dependency imports.
        path: String,
        /// When true, use `StubRegistryChecker` (deterministic offline demo).
        /// When false (default), use `SubprocessRegistryChecker` (the real
        /// live registry probe).
        offline: bool,
    },
    /// H-17 (SCHEMA_VERSION 223): `elai trial-search [--objective OBJECTIVE]
    /// [--window WINDOW] [--json]` — fixture strategy-search subcommand.
    ///
    /// Runs a small in-process fixture search loop that registers >=3 DISTINCT
    /// strategy configurations through `TrialLedger::register_trial`, producing
    /// >=3 `TrialRegistered` SSB rows and a monotone trial count N. This is the
    /// W-3 Tier-1 production entrypoint proof for H-17. The live autonomous
    /// trading-search integration (Tier-2, H-18) is a separate gated step.
    ///
    /// Default objective: "sharpe_1y". Default window: current year start..end.
    /// Default-ON; no env override required. Kill-switch: ELAI_TRIAL_LEDGER_DISABLE.
    ///
    /// Handler: `crate::trial_search_cmd::run_trial_search`.
    TrialSearch {
        /// Search objective key (e.g. "sharpe_1y"). Canonicalized before hashing.
        objective: Option<String>,
        /// Data window string "start_ms..end_ms" for the evaluation period.
        window: Option<String>,
        /// Print trial count + config hashes as JSON to stdout.
        json: bool,
    },
    /// H-18 — `elai deflation-check [--sr <f64>] [--t <u64>] [--n <u64>]
    ///   [--sharpe-std <f64>] [--pbo <f64>] [--seed <u64>] [--json]`
    ///
    /// W-3 production entrypoint for the Deflated Sharpe / PBO / MinBTL
    /// promotion gate (T-2, the #1 net-new trust gate). Registers a fixture
    /// strategy via trial-search, then calls evaluate_deflation and emits a
    /// DeflationGateEvaluated SSB row. The live trading-strategy promote hook
    /// is Tier-2 (registered stub, DEFERRED).
    ///
    /// Default scenario: a high-raw-Sharpe but high-N strategy that FAILS
    /// DSR deflation (demonstrates the gate blocks overfitted strategies).
    ///
    /// Handler: `crate::deflation_check_cmd::run`.
    ///
    /// Note: f64 fields carried as Option<String> because CliAction derives Eq
    /// and f64 does not implement Eq. Parsed to f64 in the handler.
    DeflationCheck {
        /// Per-period SR of the candidate strategy. Default: 0.10.
        sr: Option<String>,
        /// Number of return periods T. Default: 252 (one year daily).
        t_periods: Option<u64>,
        /// Number of trial strategies tested N (overrides ledger count for
        /// the fixture scenario; the ledger will have this many registered
        /// configs). Default: 50.
        n_fixture: Option<u64>,
        /// Cross-sectional std of SR across N trials. Default: 0.05.
        sharpe_std: Option<String>,
        /// PBO scalar from Tier-2 sidecar [0,1]. None = absent -> Block.
        pbo: Option<String>,
        /// Evaluation seed for GD-2 reproducibility. Default: 42.
        seed: Option<u64>,
        /// Print JSON summary instead of human text.
        json: bool,
    },

    /// H-20 — `elai leakage-check [--module <path>] [--no-split]
    ///   [--demo-fit-reaches-both] [--residue-temporal] [--seed <u64>] [--json]`
    ///
    /// W-1 production entrypoint for the leakage static-analysis COMPOSITE gate
    /// (T-3 trust-hardening). Runs evaluate_leakage over a target module against
    /// the live DKG (fail-closed Block if the DKG is absent/empty) and emits a
    /// LeakageBlocked SSB row (off-agent done-gate artifact). Sound for the
    /// Kapoor L1.1/L1.2/L1.3 classes only; the residue routes to human review.
    ///
    /// Handler: `crate::leakage_check_cmd::run`.
    LeakageCheck {
        /// Module path under analysis. Default: "pipeline.py".
        module: Option<String>,
        /// Omit the declared split boundary -> forces the L1.1 (no test set) block.
        no_split: bool,
        /// Drive the deterministic in-memory DemoLeakyDkg (fit reaches both
        /// sinks) so the sound-block path is exercisable without a built graph.
        demo_fit_both: bool,
        /// Set a heuristic L3.1 temporal residue flag -> routes to human review.
        residue_temporal: bool,
        /// Evaluation seed for GD-2 reproducibility. Default: 42.
        seed: Option<u64>,
        /// Print JSON summary instead of human text.
        json: bool,
    },

    /// H-21 — `elai lookahead-check (--demo-future-leak | --demo-causal)
    ///   [--seed <u64>] [--json]`
    ///
    /// W-1 production entrypoint for the runtime look-ahead / causality
    /// DIFFERENTIAL gate (T-3 trust-hardening; the un-gameable runtime backstop
    /// to the H-20 static leakage COMPOSITE). Diffs a full-data feature
    /// column-set against a future-truncated one; any column that changes under
    /// truncation proves a look-ahead leak (FutureLeak -> Block). Emits a
    /// LookaheadDifferential SSB row (off-agent done-gate artifact).
    ///
    /// Handler: `crate::lookahead_check_cmd::run`.
    LookaheadCheck {
        /// Drive the iloc[t+1] fixture whose leak column diverges under
        /// truncation -> BLOCK (FutureLeak).
        demo_future_leak: bool,
        /// Drive the backward-looking fixture that diffs to zero -> ALLOW (Causal).
        demo_causal: bool,
        /// Evaluation seed for GD-2 reproducibility. Default: 42.
        seed: Option<u64>,
        /// Print JSON summary instead of human text.
        json: bool,
    },

    /// H-22 — `elai null-test (--demo-overfit | --demo-genuine)
    ///   [--seed <u64>] [--json]`
    ///
    /// W-1 production entrypoint for the trading metamorphic NULL-TEST gate (T-6
    /// trust-hardening). Asks whether a strategy's edge survives a block-
    /// permutation null: a genuine edge sits in the tail (p=(C+1)/(K+1) <
    /// alpha/N, Bonferroni over N from H-17) -> ALLOW; an overfit edge sits
    /// inside the null band (p >= alpha/N) -> it survived the null -> BLOCK.
    /// Emits a TradingNullTest SSB row (off-agent done-gate artifact).
    ///
    /// Handler: `crate::null_test_cmd::run`.
    NullTest {
        /// Drive the flat/noise fixture whose edge survives the null -> BLOCK
        /// (EdgeSurvivedNull).
        demo_overfit: bool,
        /// Drive the trending fixture whose edge is significant vs the null ->
        /// ALLOW (EdgeCollapsedToNull).
        demo_genuine: bool,
        /// Evaluation seed for GD-2 reproducibility. Default: 42.
        seed: Option<u64>,
        /// Print JSON summary instead of human text.
        json: bool,
    },
    /// H-26 — `elai shadow-check (--demo-invariant-held | --demo-invariant-violated)
    ///   [--seed <u64>] [--json]`
    ///
    /// W-1 production entrypoint for the shadow-exec + financial-invariant
    /// metamorphic-relation gate (T-10 trust-hardening). Diffs two fixture-supplied
    /// result sets within a LOCKED float band AND checks textbook-correct financial
    /// invariants: accounting IDENTITIES EXACT in integer cents (share conservation,
    /// cash+position*price==equity, PnL additivity), and option-pricing / numeric MRs
    /// (PutCallParity European-only, NumericStability) within the band. A held run ->
    /// InvariantHeld (PASS, exit 0); a violated identity / beyond-band divergence ->
    /// ShadowDiverged / InvariantViolated (BLOCK, exit 1). Emits a
    /// DifferentialDivergence SSB row (off-agent done-gate artifact).
    ///
    /// Handler: `crate::shadow_check_cmd::run`.
    ShadowCheck {
        /// Drive the consistent fixture whose invariants all hold -> InvariantHeld
        /// (PASS, exit 0).
        demo_invariant_held: bool,
        /// Drive the fixture whose PnL-additivity / share-conservation identity is
        /// violated (integer cents) -> InvariantViolated (BLOCK, exit 1).
        demo_invariant_violated: bool,
        /// Evaluation seed for GD-2 reproducibility. Default: 42.
        seed: Option<u64>,
        /// Print JSON summary instead of human text.
        json: bool,
    },
    /// H-27 — `elai gate-selfcheck (--demo-all-valid | --demo-gate-broken) [--seed
    ///   <u64>] [--json]`
    ///
    /// W-1/W-3 production entrypoint for the verify-the-verifiers META gate (TD-4).
    /// Drives a FROZEN registry of the 8 Track-3 gates, replays each gate's OWN demo-
    /// violated fixture as a planted mutant through its REAL async evaluate_* fn, and
    /// asserts a BLOCK-class verdict (kill the mutant). All gates killing their mutants
    /// -> AllGatesValid (PASS, exit 0); a gate that fails to block its violation, an
    /// over-block, an un-runnable gate, or an empty registry -> a fail-closed Block
    /// (exit 1). Emits a GateSelfValidation SSB row (off-agent done-gate artifact).
    ///
    /// Handler: `crate::gate_selfcheck_cmd::run`.
    GateSelfcheck {
        /// Drive all 8 gates; expects every mutant killed -> AllGatesValid (PASS,
        /// exit 0).
        demo_all_valid: bool,
        /// Inject a synthetic WEAKENED gate whose violation fixture is not blocked ->
        /// GateFailedToBlock (BLOCK, exit 1), naming the broken gate.
        demo_gate_broken: bool,
        /// Evaluation seed for GD-2 reproducibility. Default: 42.
        seed: Option<u64>,
        /// Print JSON summary instead of human text.
        json: bool,
    },
    /// H-28 — `elai reward-check (--demo-clean | --demo-tampered | --demo-score-jump)
    ///   [--seed <u64>] [--json]`
    ///
    /// W-1/W-3 production entrypoint for the current-RF reward-tampering gate (Everitt
    /// arXiv:1908.04734). Drives check_reward_tampering over a planted fixture triple (a
    /// frozen-evaluator hash baseline, an observed hash, a claimed/prior reward + the
    /// locked plausibility band) and detects reward tampering via two sound legs: a
    /// frozen-evaluator blake3 MISMATCH -> EvaluatorModified HARD-BLOCK (leg 3a, reuses the
    /// ArchiveVerifyResult tripwire; also the fail-closed arm for a missing baseline), and
    /// an implausible reward delta with the hash INTACT -> ImplausibleRewardDelta FLAG ->
    /// human Layer-3 (leg 3b). TD-3: leg 3a is checked FIRST and short-circuits before 3b.
    /// NoTampering -> PASS (exit 0); any non-PASS verdict -> exit 1. Emits a RewardTampering
    /// SSB row (off-agent done-gate artifact).
    ///
    /// Handler: `crate::reward_tamper_cmd::run`.
    RewardCheck {
        /// Matching evaluator hash + in-band reward; expects NoTampering (PASS, exit 0).
        demo_clean: bool,
        /// Mismatching evaluator hash (the agent edited its own evaluator); expects
        /// EvaluatorModified (HARD-BLOCK, exit 1).
        demo_tampered: bool,
        /// Intact hash + an implausible reward jump beyond the locked band; expects
        /// ImplausibleRewardDelta (FLAG -> human Layer-3, exit 1).
        demo_score_jump: bool,
        /// Evaluation seed for GD-2 reproducibility. Default: 42.
        seed: Option<u64>,
        /// Print JSON summary instead of human text.
        json: bool,
    },
    /// H-19 — `elai holdout-check` runs the reusable-holdout query-budget gate (the
    /// Thresholdout DP gate, Dwork et al. arXiv:1506.02629) over a planted fixture holdout
    /// and emits HoldoutBudgetEvent SSB rows (the off-agent done-gate artifact). --demo-spend
    /// runs one above-threshold OOS query that charges 1 budget unit (QuerySpent, exit 0);
    /// --demo-exhausted provisions B=1, spends it, then runs one more query that is REFUSED
    /// (Exhausted, budget_exhausted, exit 1). Tier-1 = this always-compiled CLI (the live
    /// OOS-eval choke-point is Tier-2/deferred).
    ///
    /// Handler: `crate::holdout_check_cmd::run`.
    HoldoutCheck {
        /// Provision + one above-threshold OOS query; expects QuerySpent (charges budget,
        /// exit 0).
        demo_spend: bool,
        /// Provision B=1, spend it, then one more query; expects Exhausted (budget_exhausted,
        /// REFUSE, exit 1).
        demo_exhausted: bool,
        /// Evaluation seed for GD-2 reproducibility. Default: 42.
        seed: Option<u64>,
        /// Print JSON summary instead of human text.
        json: bool,
    },
    /// H-29 — `elai drift-check` runs the LOCKED fail-closed drift state machine (T-13, the
    /// master deployment-trust interlock) over a planted fixture strategy and emits
    /// DriftInterlock SSB rows (the off-agent done-gate artifact). --demo-stable plants a clean
    /// deployment (no drift across all four axes => Running, exit 0); --demo-drift plants a
    /// vendor-schema change + an output-PSI breach + a 14d-deviation breach (ANY axis breach
    /// latches DriftHalted, exit 1). The four axes are PSI on inputs+outputs, a blake3
    /// vendor-schema fingerprint, a rolling-14d live-vs-backtest deviation, and a 90d validation
    /// TTL; it composes the shipped G-10/D-7/H-18 signals (GD-1). The HALTED state is a
    /// persistent journal-derived latch that SURVIVES restart; the unlock is a human/frozen FREE
    /// function requiring fresh re-validation evidence. Tier-1 = this always-compiled CLI (the
    /// live-trading resume FSM edge is Tier-2/deferred — no such transition exists yet).
    ///
    /// Handler: `crate::drift_check_cmd::run`.
    DriftCheck {
        /// Plant a clean deployment; expects Running (no drift on any axis, exit 0).
        demo_stable: bool,
        /// Plant a schema change + PSI/deviation breach; expects DriftHalted (latched, exit 1).
        demo_drift: bool,
        /// Evaluation seed for GD-2 reproducibility. Default: 42.
        seed: Option<u64>,
        /// Print JSON summary instead of human text.
        json: bool,
    },
    /// H-31 — `elai remember "<text>"` captures a durable WORK preference into the bi-temporal
    /// user-memory ledger (activating the wire-dark elai_memory substrate). The text runs through
    /// the secret/PII reject FIRST (a credential is REJECTED, never stored), is folded into the
    /// FactStore via `dispatch_one` (ADD on a new key, UPDATE-via-supersede on a contradicting
    /// statement -- newest-wins soft-close, DELETE never issued), and emits a UserDecisionSuperseded
    /// SSB row (the off-agent done-gate artifact). Gated behind ELAI_USER_MEMORY_ENABLE (R9,
    /// default-OFF); a no-op when the flag is unset.
    ///
    /// Handler: `crate::user_memory_capture::capture_remember`.
    Remember {
        /// The preference text to remember (e.g. "prefer tracing over the log crate").
        text: String,
        /// Print JSON summary instead of human text.
        json: bool,
    },
    /// I-39 — `elai plan status [--letter <L>] [--json]` — derive per-plan state from
    /// three deterministic sources: (a) s-plan-audit-2026-06-04 FINISH fragment VERDICT
    /// lines, (b) `wiring_contract_check.py --json` verdicts, (c) SSB journal via
    /// read-only `SsbViewer::open`. Every output line carries an evidence pointer +
    /// "verified against <map-date/commit>" anchor. Output is byte-identical from the
    /// same inputs (deterministic). Emits one `PlanStateDerived` SSB row per invocation
    /// (fail-soft if no writable ssb.db). Default-ON; no kill-switch (pure read path).
    ///
    /// Handler: `crate::plan_status_cmd::run_plan_status`.
    PlanStatus {
        action: PlanStatusAction,
        output_format: CliOutputFormat,
    },
    /// I-41 — `elai critique <spec-path> [--lenses L1,L4,L8] [--risk simple|heavy] [--mock]`
    ///
    /// Runs spec-time critique lenses (L1-L8) over a plan/spec artifact.
    /// Advisory only; findings are written to workflow/reasoning/<stem>-critique.md
    /// and emitted as CritiqueLensFinding SSB rows (advisory=true).
    ///
    /// Kill-switch: `ELAI_CRITIQUE_LENSES_ENABLE=1` to activate LLM runs.
    /// `--mock` uses the inline stub runtime for fixture/demo mode.
    ///
    /// Handler: `crate::critique_cmd::run_critique`.
    Critique {
        action: CritiqueAction,
        output_format: CliOutputFormat,
    },
    /// I-40 — `elai wrap [--out <path>] -- <agent cmd...>` or
    /// `elai wrap --fixture [--out <path>]` — minimal trust-layer proxy.
    ///
    /// Wraps an external agent's stdio MCP tool traffic with the ELAI trust
    /// substrate (G-1 taint + F-2/G-3 write-gate + G-2 RRA emit + G-9
    /// redaction). Composes SHIPPED substrate (GD-1 — no new trust primitives).
    /// ONE controllable fixture agent tested Tier-1; full external-agent matrix
    /// is Tier-2 (user-approval gated, new-network-dep).
    ///
    /// `--fixture` runs the deterministic seeded fixture (GPU-free; W-1 trigger):
    ///   (1) untrusted ingest -> TaintFlow row (kind=Web, blocked=false)
    ///   (2) seeded destructive write -> evaluate_vibegate Block -> TaintFlow row
    ///       (kind=File, blocked=true, sink_tool=write_file)
    ///   (3) RRA bundle emitted (G-2 build_rra + write_rra + G-9 redaction;
    ///       reuses run_artifact_emitted SSB type -- NO new action_type, GD-1).
    ///
    /// Default-ON RRA emit; kill-switch: ELAI_WRAP_RRA_DISABLE=1 (R9).
    /// Handler: `crate::wrap_cmd::run`.
    Wrap {
        /// The external agent command and its arguments (everything after `--`).
        /// When `--fixture` is the first element, runs the deterministic fixture.
        agent_cmd: Vec<String>,
        /// Optional output path for the emitted `.rra` bundle.
        output_path: Option<PathBuf>,
        output_format: CliOutputFormat,
    },
    /// K-6: `elai export compliance` — redacted RRA + attestation + audit trail
    /// + retention statement bundle. Composition only (GD-1). Handler:
    /// `crate::compliance_export_cmd::run`.
    Compliance {
        session_id: String,
        /// Output directory (or path) for the compliance bundle.
        output_path: PathBuf,
        /// I-44 style flag (same bool as CliAction::Run.zero_retention).
        zero_retention: bool,
        output_format: CliOutputFormat,
    },
    /// J-26 S5 (slice S4): `elai eval-corpus-ingest --pack <dir>
    /// [--root <corpus-root>]` — load a TaskPack's
    /// `data/public/task.toml` (+ optional `task.md` overlay) and call
    /// `elai_eval_corpus::ingest::ingest_task`. Exit 0 Accepted / 1
    /// Rejected/Err.
    EvalCorpusIngest {
        pack: PathBuf,
        root: Option<PathBuf>,
    },
    /// J-72 slice C: `elai retrieval gold-build --out <path>
    /// [--rra-dir <dir>] [--history <path>] [--seed-queries <path>]
    /// [--max-hard-neg <N>] [--json]` — assemble a Tier-1 retrieval
    /// gold-set JSONL from RRA bundles + history JSONL + seed TOML.
    /// Parser in `args_retrieval.rs`; handler in `retrieval_cmd.rs`.
    RetrievalGoldBuild {
        out: PathBuf,
        rra_dir: Option<PathBuf>,
        history: Option<PathBuf>,
        seed_queries: Option<PathBuf>,
        max_hard_neg: usize,
        json: bool,
    },
}

/// I-41 — parameters for `elai critique`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CritiqueAction {
    /// Path to the spec/plan artifact to critique.
    pub spec_path: std::path::PathBuf,
    /// Which lenses to run (empty = use risk-level default).
    pub lenses: Vec<elai_orchestrator::critique::lenses::LensId>,
    /// Risk level determines default lens selection (Simple = L1/L4/L8, Heavy = all 8).
    pub risk: elai_orchestrator::critique::lenses::RiskLevel,
    /// Use inline stub runtime instead of a live LLM (fixture/demo mode).
    pub mock: bool,
}

/// E-25 Tranche C (C.4) -- `elai checkpoint <sub>` sub-verbs. Mirrors
/// the FarmAdvRmSub closed-sub-verb pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CheckpointSub {
    /// `elai checkpoint list [--session-id ID] [--json]` -- enumerate
    /// existing per-session snapshots.
    List {
        session_id: Option<String>,
        json: bool,
    },
    /// `elai checkpoint rewind --steps N [--session-id ID] [--dry-run]`
    /// -- rewind the last N snapshots in the current session.
    Rewind {
        steps: u32,
        session_id: Option<String>,
        dry_run: bool,
    },
}

/// D-28 — `elai farm-adv-rm <sub>` carrier struct. Mirrors the
/// DafnyAction / TddAction parent-variant pattern so the dispatcher
/// matches a single CliAction arm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FarmAdvRmArgs {
    pub(crate) sub: FarmAdvRmSub,
}

/// D-28 — closed sub-verb enum for `elai farm-adv-rm`. Two arms this
/// pass: `Run` (drive the farm loop end-to-end) + `Verdict` (re-evaluate
/// a previously-written JSON report). Numerical knobs on `Run` are
/// carried as Option<String> so CliAction derives Eq cleanly (f64
/// does not).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FarmAdvRmSub {
    /// `elai farm-adv-rm run --dataset PATH [...]` — drive the farm
    /// loop end-to-end. Validation of the numerical knobs lives in the
    /// `farm_adv_rm` handler so this layer stays free of orchestrator
    /// type deps.
    Run {
        dataset: String,
        corpus_dir: Option<String>,
        output: Option<String>,
        max_samples: Option<String>,
        max_rounds: Option<String>,
        z_threshold: Option<String>,
        paper_floor_pp: Option<String>,
        source_session: Option<String>,
    },
    /// `elai farm-adv-rm verdict --report PATH` — re-evaluate the
    /// verdict on a previously-written JSON report. No hook calls.
    Verdict { report: String },
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
    /// E-52 Tier-B — `elai scip lsp-probe --symbol NAME --language SLUG
    /// [--mode probe|full_handshake]`. Production caller for the
    /// `symbol_graph::lsp_oracle` JSON-RPC handshake path; defeats the
    /// E37_WIRE_DARK_GAP_PATTERN by giving the new
    /// `oracle_lookup_with_mode` function a real production call site.
    /// `--mode probe` (default) preserves the E-40 `--version` smoke
    /// probe; `--mode full_handshake` runs the LSP base-protocol v3.17
    /// initialize/initialized/shutdown/exit dance.
    LspProbe { symbol: String, language: String, mode: String },
    /// `elai scip refresh [--scip-dir <path>] [--project-root <path>]` —
    /// regenerates SCIP indices by invoking `codebase_mapper.py` and
    /// emits a `ScipRefreshCompleted` SSB row (gated on
    /// `ELAI_SCIP_REFRESH_EMIT_ENABLE`, default-OFF).
    Refresh { scip_dir: String, project_root: String },
    /// E-40 Task 2 (Tranche B-3) — `elai scip resolve --symbol NAME
    /// --language <rust|python|typescript|go> [--from-file PATH]
    /// [--rules-dir DIR]`. Production caller for
    /// `symbol_graph::scope_graph::ScopeGraph::resolve_name` — the push/pop
    /// scope-graph name resolver. Emits one `scope_graph_resolved` SSB row
    /// (gated on `ELAI_SCOPE_GRAPH_EMIT_ENABLE`, default-OFF).
    Resolve {
        symbol: String,
        language: String,
        from_file: String,
        rules_dir: String,
    },
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
    /// `elai safety tool-filter --fixture NAME` — commit a minimal
    /// researcher tool set (Read + Web), load the named AgentDojo-style
    /// fixture from `crates/contracts/tests/fixtures/tool_filter/`, replay
    /// each injection step, assert every out-of-set call is denied, emit
    /// `tool_filter_committed` + `untrusted_var_quarantined` SSB rows, and
    /// render a JSON report `{ commit_hash, denied_tool, asr_before,
    /// asr_after }` where `asr_after` must be strictly less than
    /// `asr_before`.
    ToolFilter { fixture: String },
    /// F-10 / Slice A4: `elai safety monotonicity --fixture PATH
    /// [--no-env-override]` — security-monotonicity iteration guard live
    /// trigger. Loads the JSON fixture
    /// (`{ lang, prior_reply, current_reply }`), classifies the prior reply's
    /// CWE-class set, constructs the REAL
    /// `SecurityMonotonicityGate::check_monotonicity` (`loop_kind=Refine`),
    /// and on `MonotonicityVerdict::Regressed` emits the
    /// `SecurityClassRegressionBlocked` SSB beacon + Finding (durably, via an
    /// own current-thread runtime so `Ssb::new` is constructed inside
    /// `block_on`) and prints the verdict. `--no-env-override` documents that
    /// the run must NOT be suppressed by an ambient kill-switch — when the
    /// flag is set and `ELAI_SECURITY_MONOTONICITY_DISABLE` is truthy the
    /// handler errors instead of silently degrading to `Monotone`.
    /// Handler: `crate::safety_cmd::run_monotonicity`.
    Monotonicity {
        /// Path to the JSON fixture (`{ lang, prior_reply, current_reply }`).
        fixture: String,
        /// When true, refuse to run if the kill-switch env var is set truthy
        /// (the run must observe the real verdict, not a suppressed one).
        no_env_override: bool,
    },
}

/// F-8 / Slice A4 — `elai self-mod <sub>` sub-verb. SCF symbolic-conflict
/// detector operator surface. Bundled with the parent `SelfMod` variant so the
/// dispatcher matches a single arm. See `workflow/reasoning/F-8-decisions.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SelfModAction {
    /// `elai self-mod apply --proposal-fixture PATH [--no-env-override]` — the
    /// W-3 production entrypoint + W-5 live trigger. Loads a `ConfigVariant`
    /// fixture (`{ variant_id, config, active_tripwires, fsm_substate }`), runs
    /// the REAL symbolic-conflict screen against the live `LOCKED_KEYS` +
    /// tripwire model, and on conflict emits the real `symbolic_conflict_detected`
    /// SSB beacon(s) (the held-pending record) + reports `status = pending`. The
    /// candidate is NOT applied and NOT hard-rejected — WARN-then-escalate, never
    /// fail-closed. `--no-env-override` refuses to run when
    /// `ELAI_SYMBOLIC_CONFLICT_DISABLE` is truthy (the caller demanded a real
    /// screen). Handler: `crate::self_mod_cmd::run_apply`.
    Apply {
        /// Path to the JSON proposal fixture.
        proposal_fixture: String,
        /// When true, refuse to run if the kill-switch env var is set truthy.
        no_env_override: bool,
    },
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
    /// `elai memory pin --fact-id <id>` — emit Pinned governance beacon (H-33).
    Pin,
    /// `elai memory unpin --fact-id <id>` — emit Unpinned governance beacon (H-33).
    Unpin,
    /// `elai memory provenance [--task <hint>] [--json]` — eligible-to-fire set (H-33).
    Provenance,
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

/// G-10 (SCHEMA_VERSION 218) — `elai model-drift <sub>` sub-verb.
/// DISTINCT from `elai drift` (E-45 cross-language SOURCE drift). Use the
/// full subcommand string "model-drift" to avoid clobbering the E-45 `drift`
/// arm (args.rs drift arm / drift_cmd.rs). See workflow/reasoning/G-10-decisions.md.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ModelDriftAction {
    /// `elai model-drift baseline [--project-root PATH] [--json]`
    /// Score the canary fixture set under the current identity and write the
    /// per-family baseline scores to benchmarks/model_drift/<identity_key>.json.
    Baseline { project_root: PathBuf, json: bool },
    /// `elai model-drift check [--project-root PATH] [--json]`
    /// Read the last-good baseline, re-run the canary, emit ModelDriftSuspected
    /// on regression, exit non-zero on drift.
    Check { project_root: PathBuf, json: bool },
    /// `elai model-drift show [--project-root PATH] [--json]`
    /// Display the latest baseline file.
    Show { project_root: PathBuf, json: bool },
}

/// G-2 — `elai rra <sub>` sub-verb. Bundled with the parent `Rra` variant so
/// the dispatcher can match a single arm (mirrors `FreezeAction`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RraAction {
    /// `elai rra show <path>` — render the `.rra` summary.
    Show { path: PathBuf },
    /// I-40 — `elai rra verify <path>` — spec-conformance check against
    /// rra-spec-v1 (docs/spec/rra-spec-v1.md, C1-C6 required + I1-I4
    /// informational). DISTINCT from `elai replay --verify` (which re-derives
    /// content hashes; GD-5). Exits nonzero on any C1-C6 failure so CI can
    /// gate on it. Handler: `crate::rra_verify_cmd::run_verify_conformance`.
    Verify { path: PathBuf },
    /// J-14 — `elai rra export-fixture <bundle> [--adapter <name>]` — export a
    /// content-keyed cassette (`.fixture`) from an RRA bundle. Scrubs auth
    /// headers, keys the cassette by `(model, content_blake3, tool_name)`, and
    /// returns the fixture path + SSB-ready `ReproPackPayload` (kind =
    /// `FixtureExtracted`). The `adapter` string is a passthrough label for
    /// J-91 (may be the empty string today).
    ExportFixture { bundle: PathBuf, adapter: String },
}

/// J-14 — CLI-side tier selector for `elai replay --verify --tier <t>`.
/// Maps 1:1 to [`elai_orchestrator::rra::TierVerdictLabel`]. Kept as a
/// distinct CLI-layer enum so the parser does not depend on the
/// orchestrator crate's wire types (the args module is parsed before any
/// orchestrator symbol is resolved).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReplayVerifyTier {
    /// `--tier t0` — action-stream replay (the G-2 binary verdict).
    T0,
    /// `--tier t1` — token-stream blake3 equality.
    T1,
    /// `--tier t2` — score/logprob equality.
    T2,
}

impl ReplayVerifyTier {
    /// Parse the `--tier` value string (`t0` | `t1` | `t2`,
    /// case-insensitive). Returns `None` on an unknown label so the caller
    /// can format the error with the accepted set.
    pub(crate) fn parse(raw: &str) -> Option<Self> {
        match raw.to_ascii_lowercase().as_str() {
            "t0" | "actionfaithful" | "action-faithful" => Some(Self::T0),
            "t1" | "tokenfaithful" | "token-faithful" => Some(Self::T1),
            "t2" | "scorefaithful" | "score-faithful" => Some(Self::T2),
            _ => None,
        }
    }
}

/// G-2 — `elai replay <sub>` sub-verb. The top-level `replay` verb is new; it
/// does NOT collide with the existing `replay-spill` / `fixture-replay`
/// sub-actions (those are distinct subcommand words).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReplayAction {
    /// `elai replay --verify <path>` — verify a `.rra` offline.
    /// I-45: `--current-objective <text>` threads the caller's current
    /// objective text into `verify_artifact_with_objective` so
    /// `ReplayVerdict::ObjectiveHashMismatch` can fire on a live CLI run
    /// (GAP-1). `None` preserves the legacy `ObjectiveUnchecked` behavior.
    /// J-14: `--tier t0|t1|t2` selects the graded verify tier
    /// (`ActionFaithful` / `TokenFaithful` / `ScoreFaithful`); `None`
    /// defaults to `ActionFaithful` (T0, the G-2 binary verdict).
    Verify {
        path: PathBuf,
        current_objective: Option<String>,
        tier: Option<ReplayVerifyTier>,
    },
    /// H-23 — `elai replay --verify --metric-digest <demo>` — same-seed re-run
    /// hash-equality METRIC_GATE. ADDITIVE to the offline `.rra` verifier above;
    /// it does NOT regress the `Verify { path }` arm. Drives
    /// `rerun_equality::verify_rerun_digest` over two fixture-supplied
    /// declared-metric-output digest lists and emits a `RerunHashEquality` SSB
    /// row (the off-agent artifact). `demo` selects the deterministic Tier-1
    /// fixture (`deterministic` => Equal twice; `nondeterministic` => a hidden
    /// unseeded RNG => Divergent). The live double-run of a real experiment is
    /// the deferred Tier-2 path.
    VerifyRerun { demo: RerunDemoMode },
    /// H-24 — `elai replay --verify --golden <demo>` — sim-to-real parity gate
    /// (Part C). ADDITIVE to the offline `.rra` verifier + the H-23 re-run gate;
    /// it does NOT regress either arm. Drives
    /// `sim_real_parity::verify_replay_golden` over a seeded replay event log and
    /// a FROZEN golden event log and emits a `SimRealParity` SSB row (the off-
    /// agent artifact). `demo` selects the deterministic Tier-1 fixture
    /// (`parity` => replay==golden => Parity PASS; `gap` => a divergent event log
    /// => ReplayGoldenMismatch BLOCK). The live paper-trade reconciliation
    /// (numeric Part B) is the deferred Tier-2 path (GD-5).
    VerifyGolden { demo: GoldenDemoMode },
    /// H-25 — `elai replay --verify --result-manifest <demo>` — result manifest
    /// re-derivation gate. ADDITIVE to the offline `.rra` verifier + the H-23
    /// re-run gate + the H-24 parity gate; it does NOT regress any of them. Drives
    /// `result_manifest::verify_result` over a fixture manifest + a set of
    /// orchestrator-captured outputs and emits a `ResultManifestVerified` SSB row
    /// (the off-agent artifact). `demo` selects the deterministic Tier-1 fixture
    /// (`complete` => every declared number re-derives => ReDerived PASS;
    /// `tampered` => a hand-edited value that does not re-parse =>
    /// FabricationDetected BLOCK; `incomplete` => an empty/trivial manifest =>
    /// FabricationDetected BLOCK, anti-sandbagging). The produced-vs-declared
    /// completeness diff is the deferred Tier-2 path.
    VerifyResultManifest { demo: ResultManifestDemoMode },
}

/// H-35 — rewind scope: which parts of the session state to restore.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RewindScope {
    /// Trim the message stream past target_seq; rehydrate stubs.
    Conversation,
    /// Code restore only via ssb_resume_from (fail-closed predicate).
    Code,
    /// Both: conversation trim THEN code restore (default).
    Both,
}

/// H-35 — parsed rewind action from `elai rewind <target> [flags]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RewindAction {
    /// Raw target string: an integer SSB seq, "turn:<n>", or "checkpoint:<label>".
    pub(crate) target: String,
    pub(crate) scope: RewindScope,
    pub(crate) summarize: bool,
    /// Override the active session_id (defaults to the env/config session).
    pub(crate) session_id: Option<String>,
    /// Dry-run: print what would be done without writing any SSB row.
    pub(crate) dry_run: bool,
}

/// H-49 — Scope selector for `elai undo / redo / jump`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum TimeTravelScope {
    /// Restore file content only (code restore).
    Code,
    /// Restore conversation context only.
    Conversation,
    /// Restore both files and conversation (default).
    #[default]
    Both,
}

/// H-49 — Grain selector for `elai undo / redo / jump`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum TimeTravelGrainArg {
    /// Single tool-call granularity.
    Edit,
    /// User-assistant turn granularity (default).
    #[default]
    Turn,
    /// Baby-step boundary granularity.
    BabyStep,
}

/// H-49 — Dispatched action from the `elai undo/redo/history/jump` sub-commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TimeTravelAction {
    Undo {
        grain: TimeTravelGrainArg,
        scope: TimeTravelScope,
        steps: u32,
        dry_run: bool,
        session_id: Option<String>,
    },
    Redo {
        grain: TimeTravelGrainArg,
        scope: TimeTravelScope,
        dry_run: bool,
        session_id: Option<String>,
    },
    History {
        format: String,
        session_id: Option<String>,
    },
    Jump {
        target: String,
        grain: TimeTravelGrainArg,
        scope: TimeTravelScope,
        dry_run: bool,
        session_id: Option<String>,
    },
}

/// H-23 — Tier-1 fixture selector for `elai replay --verify --metric-digest`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RerunDemoMode {
    /// A clean seeded experiment: both re-runs hash identical declared outputs
    /// (Equal twice).
    Deterministic,
    /// A hidden unseeded RNG: run #2's declared-output digest differs
    /// (Divergent, the first divergent metric named).
    Nondeterministic,
}

/// H-24 — Tier-1 fixture selector for `elai replay --verify --golden`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GoldenDemoMode {
    /// A clean single-entrypoint strategy: the seeded replay reproduces the frozen
    /// golden event log byte-for-byte (Parity PASS).
    Parity,
    /// A sim-real divergence: the replay's fill event differs from the golden (a
    /// different fill model in backtest) => ReplayGoldenMismatch BLOCK, first
    /// divergent event named.
    Gap,
}

/// H-25 — Tier-1 fixture selector for `elai replay --verify --result-manifest`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResultManifestDemoMode {
    /// A clean manifest: every declared empirical number re-parses from its
    /// orchestrator-captured output to its declared value AND re-hashes to its
    /// declared provenance (ReDerived PASS).
    Complete,
    /// A hand-edited number: the declared value does not re-parse from the captured
    /// output (FabricationDetected BLOCK, the offending result named).
    Tampered,
    /// An empty/trivial manifest: a reported number with a 0/empty manifest is the
    /// anti-fabrication signature (FabricationDetected BLOCK, anti-sandbagging).
    Incomplete,
}

/// G-5 — `elai review <sub>` sub-verb. Bundled with the parent `Review` variant
/// so the dispatcher can match a single arm (mirrors `RraAction`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReviewAction {
    /// `elai review --cockpit [--session <id>] [--base <ref>]
    ///   [--demo-anchor] [--trust-model-line-numbers]` — compute + print
    /// the reviewer-cockpit DSR and emit a `review_surface_scored` row.
    ///
    /// * `session` — the session whose SSB vouch stream is read + whose `ssb.db`
    ///   the row is written to. `None` resolves to the canonical session id the
    ///   driver derives from the project root (mirrors `dep-check`).
    /// * `base` — the git ref to diff the working tree against. `None` defaults
    ///   to `HEAD`. The driver resolves it to a concrete commit SHA via
    ///   `git rev-parse` and records that SHA in the row (deterministic pin).
    /// * `demo_anchor` — H-52 W-1 seeded-fixture trigger: injects deterministic
    ///   snippet-bearing ReviewFindings over a fixed diff fixture so the resolver
    ///   runs on REAL snippets and emits genuine FindingAnchored rows covering all
    ///   three observable-effect cases: (a) Anchored, (b) Ambiguous, (c) Unanchorable.
    ///   The W-1 live-CLI proof; the DEFERRED model-finding source (E-41/C-40) is
    ///   the future consumer. NOT a stub — it is the sanctioned observable_effect
    ///   pattern used by every other gate in this run (H-21/H-26/H-27 etc.).
    /// * `trust_model_line` — H-52 BROKEN-ARM (R2 proof): anchors case (b) to a
    ///   model-emitted line instead of the resolver, intentionally mis-anchoring
    ///   the Ambiguous finding to the wrong hunk. A gate using this mode fails R2
    ///   (a resolver that stays "right" when fed a model line is itself a FAIL).
    Cockpit {
        session: Option<String>,
        base: Option<String>,
        /// H-52 W-1 seeded-fixture trigger. Default: false.
        demo_anchor: bool,
        /// H-52 broken-arm flag (R2 proof). Default: false.
        trust_model_line: bool,
    },
}

/// E-39 -- `elai cloud override-cap` sub-verbs. The override-cap
/// subcommand bumps the daily / monthly layered cap and requires a
/// typed-confirmation string to prevent fat-finger overrides. Future
/// sub-verbs (status, reset, history) land here as siblings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CloudAction {
    /// `elai cloud override-cap <usd-amount> --confirm <typed-string>`.
    /// Records the override in SSB; the orchestrator's
    /// `cloud_abort::LayeredCapsConfig` consumes it on next session
    /// start.
    OverrideCap {
        amount_usd: u64,
        confirmation: String,
    },
    /// `elai cloud status` -- print the current layered-cap config +
    /// most recent SSB rows.
    Status,
}

/// E-43 -- `elai skills <list|eval|publish|export>` sub-verb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SkillsAction {
    /// `elai skills list` -- enumerate SKILL.md files via the C-41
    /// SkillRegistry walker.
    List,
    /// `elai skills eval [--skill <name>] [--dry-run]` -- run the
    /// lexical-match precision/recall harness against each skill's
    /// eval/positive.jsonl + eval/negative.jsonl.
    Eval { skill: Option<String>, dry_run: bool },
    /// `elai skills publish --target <claude|codex|gemini> [--dry-run]`
    /// -- emit dist/<target>/skills/<name>/SKILL.md (and optional
    /// agents/openai.yaml for Codex). ABSORBs the E-22 export shim.
    Publish { target: String, dry_run: bool },
    /// E-22 (SCHEMA_VERSION 170) -- `elai skills export
    /// [--target <claude|codex|gemini>] [--dry-run]`. Writes a
    /// normalised SKILL.md to dist/<target>/skills/<name>/SKILL.md
    /// (frontmatter keys sorted; body preserved byte-for-byte) and
    /// detects drift via a blake3 8-byte prefix diff against any prior
    /// dist copy. Emits one `GithubTrendingEvent::SkillExportNormalized`
    /// SSB row per skill processed (wire-dark unless
    /// `ELAI_GITHUB_TRENDING_ENABLE=1`). Differs from `publish` in two
    /// ways: (1) the frontmatter is normalized BEFORE write (publish
    /// emits the body verbatim), and (2) drift detection runs
    /// idempotently so re-runs report drift_count=0 on a clean tree.
    Export { target: String, dry_run: bool },
    /// F-6 / Slice A4 (C6c) -- `elai skills-mgmt library list` -- print every
    /// learned skill in the in-memory [`elai_orchestrator::SkillStore`] with
    /// its trust tier (S1..S4), invocation/failure counts, and staleness.
    LibraryList,
    /// F-6 / Slice A4 -- `elai skills-mgmt library promote <id> [--human-grant]`
    /// -- climb a learned skill one trust rung. The apex jump `S3 -> S4`
    /// requires `--human-grant`. Emits one `SkillLibraryEvent(TrustPromoted)`
    /// SSB row on success.
    LibraryPromote { id: String, human_grant: bool },
    /// F-6 / Slice A4 -- `elai skills-mgmt library show <id>` -- print one
    /// learned skill's full record (tier, counts, ages, staleness score).
    LibraryShow { id: String },
    /// F-6 / Slice A4 (C6c) -- `elai skills-mgmt library tick` -- run one
    /// staleness-demotion sweep (`SkillStore::demote_if_stale`) and emit one
    /// `SkillLibraryEvent(DemotedStale)` SSB row per demoted skill.
    LibraryTick,
    /// I-12 -- `elai skills-mgmt playbook export [--out <path>] [--emit-ssb]`
    /// -- export verified playbook entries to AGENTS.md / skills format with
    /// embedded evidence row IDs. Emits PlaybookEvent{Exported} SSB row when --emit-ssb.
    PlaybookExport { out_path: Option<String>, emit_ssb: bool },
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

/// E-53 — `elai formal-verify <sub>` sub-verb. Drives the
/// orchestrator's `formal_verify` module (composing C-50
/// contract_coding + C-52 dafny_verify under a CSQ-classifier gate).
///
/// `run --target PATH --verifier KIND` dispatches one verifier
/// (z3_smt | dafny | kani | verus) against a target path. `status`
/// reports each toolchain's RUNPOD-PENDING posture (all 4 are
/// expected absent on local dev hosts per memory + C-52 substrate).
///
/// Citation discipline (per workflow/sessions/s-25120/mistakes.md):
/// the E-53 plan cites 14 arXiv IDs; only 3 (arXiv:2604.22601 NL2VC,
/// arXiv:2501.03124 PRMBench, AlphaProof Nature 2025) are inline-
/// cited in the formal_verify mod docstring. The other 11 are
/// deferred to mistakes.md cite-incident #11 (E-53-INGEST-pending).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FormalVerifyAction {
    /// `elai formal-verify run --target PATH --verifier KIND [--demo-escape-hatch]` —
    /// dispatch one verifier against a target file. KIND must be one
    /// of `z3_smt` / `dafny` / `kani` / `verus`. Z3 / Kani / Verus
    /// routes are RUNPOD-PENDING stubs returning Inapplicable in the
    /// lean E-53 ship; Dafny route reuses C-52 invoke_dafny_cli and
    /// surfaces RunpodPending when the binary is absent.
    ///
    /// `--demo-escape-hatch` is the H-54 W-1 seeded-fixture trigger: feeds a
    /// frozen goal containing a relocated sorry (AlphaProof failure mode 1:1)
    /// so the scan returns EscapeHatchAnywhere and emits the off-agent
    /// TrapBehaviorDetected(ClosureEscapeHatch) SSB row.
    Run {
        target: String,
        verifier: String,
        /// H-54: seeded-fixture demo trigger.
        demo_escape_hatch: bool,
    },
    /// `elai formal-verify status` — print the 4 verifier toolchains
    /// + each one's RUNPOD-PENDING posture + the substitute-models
    /// list inherited from C-52 (Gemma-4 BLOCKED per memory).
    Status,
}

/// E-8 — `elai optimize <sub>` sub-verb. Drives the self-evolving
/// prompts pipeline:
///   * `template` -- run one optimization pass for a given
///     `AgentRole` over train/val fixtures, dispatching the Python
///     `elai_optimizer` service.
///   * `status` -- print the optimizer's RUNPOD-PENDING posture +
///     the gepa PyPI version + which roles have shipped templates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OptimizeAction {
    /// `elai optimize template --target ROLE --train PATH --val PATH
    /// [--reflection-model NAME] [--budget-tokens N] [--include-cross-domain]`
    Template {
        target: String,
        train: String,
        val: String,
        reflection_model: String,
        budget_tokens: u64,
        include_cross_domain: bool,
    },
    /// `elai optimize status` — print optimizer status.
    Status,
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
            "--otlp-endpoint" => {
                // E-29 Tier-B: optional OTLP gRPC endpoint for the
                // ssb-otel-export crate. When provided, sets the
                // ELAI_OTLP_ENDPOINT env var the observability subtree
                // reads via observability::ELAI_OTLP_ENDPOINT_VAR.
                // Default: noop (no OTLP export). Live OTLP integration
                // is RUNPOD-PENDING -- the exporter constructor sees
                // the env var here but the live gRPC emit path needs
                // a spun-up otelcol-contrib instance. See
                // workflow/reasoning/E-29-decisions.md (Tier-B).
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --otlp-endpoint".to_string())?;
                std::env::set_var("ELAI_OTLP_ENDPOINT", value);
                index += 2;
            }
            flag if flag.starts_with("--otlp-endpoint=") => {
                std::env::set_var("ELAI_OTLP_ENDPOINT", &flag[16..]);
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
        // J-30 T5: `elai providers configure [...]` -- direct local parse
        // (FORK D: the existing /providers slash command is listing-only
        // and is NOT touched). Sub-verb is `configure`; future subactions
        // can extend parse_providers_subcommand_args. Handler at
        // crates/cli/src/providers_cmd.rs.
        "providers" => parse_providers_subcommand_args(&rest[1..], output_format),
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
        // F-9 / Slice D4: `elai dep-check <path> [--offline]`.
        // DEFAULT uses SubprocessRegistryChecker (real live registry probe);
        // --offline switches to StubRegistryChecker (deterministic demo).
        "dep-check" => {
            let path = rest
                .iter()
                .skip(1)
                .find(|a| !a.starts_with('-'))
                .cloned()
                .ok_or_else(|| "dep-check requires a <path> argument".to_string())?;
            let offline = rest.iter().any(|a| a == "--offline");
            Ok(CliAction::DepCheck { path, offline })
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
        // E-12 — `elai production-checklist [--project-root PATH]
        // [--output-format text|json|toml]`. Runs the production security
        // checklist against a target directory.
        "production-checklist" => {
            parse_production_checklist_subcommand_args(&rest[1..], output_format)
        }
        // B-43 Phase 7 — `elai install [--skip-models] [--reset-config]
        // [--project-root PATH] [--output-format text|json]`. Bare `elai install`
        // is also reachable via parse_single_word_command_alias.
        "install" => parse_install_subcommand(&rest[1..], output_format),
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
        // G-10 (SCHEMA_VERSION 218) — `elai model-drift baseline|check|show`.
        // Exogenous model-version behavioral-regression gate; handler at
        // crates/cli/src/model_drift_cmd.rs. DISTINCT from `elai drift`
        // (E-45 cross-language SOURCE drift). Default-ON (W-2); kill-switch
        // ELAI_MODEL_DRIFT_DISABLE only. See workflow/reasoning/G-10-decisions.md.
        "model-drift" => parse_model_drift_subcommand_args(&rest[1..], output_format),
        // H-41 (SCHEMA_VERSION 219) — `elai checker-gaps [--limit N] [--json]`.
        // Ranked backlog of claim-class x complexity-taxon gaps (Xu Table-3).
        // Read surface over CheckerGapDetected SSB rows. See H-41 plan.
        "checker-gaps" | "checker_gaps" => parse_checker_gaps_args(&rest[1..]),
        // J-28 T8 — `elai usage [--json]`. Grouped cloud/local token-usage
        // rollup read surface over UsageTurnEvent SSB rows. Handler at
        // crates/cli/src/usage_cmd.rs.
        "usage" => parse_usage_subcommand_args(&rest[1..]),
        // G-2 — `elai rra show <path>`. Reproducible-Run Artifact viewer;
        // handler at crates/cli/src/rra_cmd.rs.
        "rra" => parse_rra_subcommand_args(&rest[1..], output_format),
        // J-2 — local Tier-1 read-only MCP control plane (stdio only).
        "mcp-serve" => parse_mcp_serve_subcommand_args(&rest[1..], output_format),
        // G-2 — `elai replay --verify <path>`. Offline .rra verifier;
        // handler at crates/cli/src/rra_cmd.rs. Top-level `replay` is new and
        // distinct from the replay-spill / fixture-replay sub-actions.
        "replay" => parse_replay_subcommand_args(&rest[1..], output_format),
        // H-35 — `elai rewind <target> [--scope ...] [--summarize]`. Session
        // state restore to a prior SSB seq. Handler at crates/cli/src/rewind_cmd.rs.
        // Distinct from the OLD CheckpointSub::Rewind sub-verb under `checkpoint`.
        "rewind" => parse_rewind_subcommand_args(&rest[1..], output_format),
        // H-49 — `elai undo / redo / history / jump`. CAS-backed reversible LLM
        // edits over the undo-tree HEAD. Default-ON; kill-switch ELAI_TIMETRAVEL_DISABLE.
        // Handler at crates/cli/src/timetravel_cmd.rs.
        "undo" => parse_timetravel_undo_args(&rest[1..], output_format),
        "redo" => parse_timetravel_redo_args(&rest[1..], output_format),
        "history" | "timeline" => parse_timetravel_history_args(&rest[1..], output_format),
        "jump" => parse_timetravel_jump_args(&rest[1..], output_format),
        // G-5 — `elai review --cockpit [--session <id>] [--base <ref>]`. The
        // reviewer-cockpit DSR producer; handler at crates/cli/src/review_cmd.rs.
        "review" => parse_review_subcommand_args(&rest[1..], output_format),
        // H-53 — `elai prebind [--demo-prebind] [--allow-freeform-citations]`. Citation
        // pre-binding + constrained synthesis on the /research|/ask answer path.
        // --demo-prebind: H-53 W-1 seeded-fixture trigger (the observable_effect proof).
        // --allow-freeform-citations: BROKEN-ARM (R2 proof, emits Blocked SSB row).
        // Handler at crates/cli/src/prebind_cmd.rs.
        "prebind" => parse_prebind_subcommand_args(&rest[1..], output_format),
        // G-8 — `elai verify-attestation <commit>`. Offline in-toto provenance
        // attestation verifier; handler at crates/cli/src/verify_attestation_cmd.rs.
        // READ-ONLY; trust-without-re-running (SLSA-VSA).
        "verify-attestation" => parse_verify_attestation_subcommand_args(&rest[1..], output_format),
        // E-43 -- `elai skills-mgmt <list|eval|publish>`. Agent skill systems
        // surface; handler at crates/cli/src/skills_cmd.rs. ABSORBs the
        // E-22 SKILL.md export shim. Paper-grounded against
        // agentskills.io/specification. Renamed from `skills` to
        // avoid clash with the existing slash-command `Skills` variant.
        "skills-mgmt" => crate::args_skills::parse_skills_subcommand_args(&rest[1..], output_format),
        // E-39 -- `elai cloud <override-cap|status>`. Layered-cap
        // manual-override CLI surface. Handler: crate::cloud_cmd.
        "cloud" => parse_cloud_subcommand_args(&rest[1..], output_format),
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
        // E-35 — `elai structured-output status|codeact-run|strict-mode|
        // grammar emit-test|search-replace check|reject-engine`.
        // Structured-output substrate operator surface (XGrammar /
        // INSTRUCTOR / CodeAct / SEARCH/REPLACE / cloud strict-schema).
        // WIRE-DARK avoidance per E40_WIRE_DARK_AVOIDANCE_VIA_CLI:
        // every new SSB beacon variant in the +10 atomic bump has at
        // least one production caller via this subverb tree. Python
        // sidecar wiring is RUNPOD-PENDING; AgentRole::Runner DEFERRED
        // per E41_AGENTROLE_FROZEN_AT_17 (CodeAct routes through
        // existing role + RuntimeBackend). Handler at
        // crates/cli/src/structured_output_cmd.rs. See
        // workflow/reasoning/E-35-decisions.md.
        "structured-output" => {
            parse_structured_output_subcommand_args(&rest[1..], output_format)
        }
        // E-37 — `elai safety status|list-pins|verify-pins|fixture-replay`.
        // Prompt-injection defense operator surface; handler at
        // crates/cli/src/safety_cmd.rs. Layers additively atop E-24 +
        // E-30. See workflow/reasoning/E-37-decisions.md.
        "safety" => parse_safety_subcommand_args(&rest[1..], output_format),
        // F-8 / Slice A4 — `elai self-mod apply --proposal-fixture PATH`.
        // SCF symbolic-conflict detector W-3 production entrypoint + W-5 live
        // trigger; handler at crates/cli/src/self_mod_cmd.rs. See
        // workflow/reasoning/F-8-decisions.md.
        "self-mod" => parse_self_mod_subcommand_args(&rest[1..], output_format),
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
        // E-53 — `elai formal-verify run|status`. Formal-verify
        // pipeline CLI surface; handler at
        // crates/cli/src/formal_verify_cmd.rs. Composes
        // orchestrator::formal_verify (which composes C-50
        // contract_coding + C-52 dafny_verify). Live Z3 / Kani /
        // Verus dispatch is RUNPOD-PENDING; Dafny dispatch reuses
        // C-52 invoke_dafny_cli. Gemma-4 BLOCKED per memory
        // feedback_no_gemma4. Mirrors C-52 dafny subcommand shape +
        // the snake-case variant convention. Also accepts the
        // underscore variant (`formal_verify`) for caller ergonomics.
        "formal-verify" | "formal_verify" => {
            parse_formal_verify_subcommand_args(&rest[1..], output_format)
        }
        // E-8 — `elai optimize template|status`. Self-evolving prompts
        // driver. Spawns the Python `elai_optimizer` service. Live LLM
        // calls RUNPOD-PENDING. Refuses local 7B as reflection LM per
        // arXiv:2405.10276 (Zhang et al.) -- enforced at CLI parse
        // time. See workflow/reasoning/E-8-decisions.md.
        "optimize" => parse_optimize_subcommand_args(&rest[1..], output_format),
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
        "symbol_graph" | "symbol-graph" => crate::args_symbol::parse_symbol_graph_args(&rest[1..]),
        // F-5 / Slice A4: `elai dkg staleness-tick --workspace <path>` (W-5).
        "dkg" => parse_dkg_args(&rest[1..]),
        // B-29 Phase 5b: thin shell wrappers around the SymbolGraphCache
        // traversal API so external scripts (Python eval, agents) can call
        // compute_impact_radius / compute_symbol_context / detect_entry_points
        // by fork-exec with --output-format=json. No IPC bridge, no daemon.
        // Handlers at crates/cli/src/symbol_graph.rs.
        "symbol_impact" | "symbol-impact" => {
            crate::args_symbol::parse_symbol_impact_args(&rest[1..], output_format)
        }
        "cone_token_floor" | "cone-token-floor" => {
            crate::args_symbol::parse_cone_token_floor_args(&rest[1..], output_format)
        }
        "symbol_context" | "symbol-context" => {
            crate::args_symbol::parse_symbol_context_args(&rest[1..], output_format)
        }
        "entry_points" | "entry-points" => {
            crate::args_symbol::parse_entry_points_args(&rest[1..], output_format)
        }
        // E-49 SWE-Bench Pro v5 driver surface — non-interactive task
        // submission with FSM + per-role runtime overrides. Handler at
        // crates/cli/src/task_cmd.rs.
        "task" => parse_task_subcommand_args(&rest[1..]),
        "wip-squash" | "wip_squash" => parse_wip_squash_args(&rest[1..]),
        // D-19: `elai bench prmbench ...` — PRMBench adversarial-gaming benchmark.
        // B-15 Tranche O.2 (orchestrate s-22600): parser block moved to
        // crate::args_bench. CliAction variants unchanged.
        "bench" => crate::args_bench::parse_bench_subcommand_args(&rest[1..]),
        // dkg-stream-rigorous Task 2: `elai index build [--project-root PATH] [--vectors]`.
        "index" => parse_index_subcommand_args(&rest[1..]),
        // D-28: `elai farm-adv-rm <run|verdict> ...` — adversarial example
        // farming runner. Drives the AdvRmFarmer (orchestrator-side) and
        // writes corpus.jsonl with `gold_verdict: "pending"`. Single
        // SsbActionType::AdversarialFarmRunCompleted row at end-of-run.
        "farm-adv-rm" | "farm_adv_rm" => parse_farm_adv_rm_args(&rest[1..]),
        // E-10: hostile-critic structural simplifications.
        "bench-ablation" | "bench_ablation" => crate::args_critic::parse_bench_ablation_args(&rest[1..]),
        // E-7: darwinian-evolve substrate exerciser. See darwin_loop_cmd.rs.
        "darwin-loop" | "darwin_loop" => crate::args_critic::parse_darwin_loop_args(&rest[1..]),
        "tier-eval" | "tier_eval" => crate::args_critic::parse_tier_eval_args(&rest[1..]),
        "critic-report" | "critic_report" => crate::args_critic::parse_critic_report_args(&rest[1..]),
        // E-6: MAST failure-mode report. `elai mast report [--session <id>]
        // [--limit N] [--json]` reads audit_mast_tagged SSB rows read-only.
        "mast" => crate::args_critic::parse_mast_args(&rest[1..]),
        // E-47: LoRA ship-gate dispatcher. Reads [finetune.lora] from
        // elai_cc.toml + evaluates triple-AND gate + prints outcome +
        // emits SSB row when ELAI_LORA_SHIP_GATE_ENABLE=1.
        "lora-gate-check" | "lora_gate_check" => crate::args_critic::parse_lora_gate_check_args(&rest[1..]),
        // E-9: external agent-eval benchmark suite dispatcher. Top-level
        // `elai eval run --benchmark=<NAME> ...` per
        // workflow/reasoning/E-9-decisions.md DECISION-8 (kept separate
        // from the existing `bench *` D-19..D-25 surfaces).
        "eval" => crate::args_skills::parse_eval_subcommand_args(&rest[1..]),
        // E-55: `elai rl train --recipe=<name>` — agentic RL recipe
        // runner. Substrate ships fully; cloud Sonnet rollouts + LoRA
        // training RUNPOD-PENDING.
        "rl" => crate::args_skills::parse_rl_subcommand_args(&rest[1..]),
        // E-25 Tranche C (C.4): `elai checkpoint <list|rewind>` --
        // edit snapshot store CLI surface. Wire-dark default OFF for
        // SSB emit; CLI inspection always works.
        "checkpoint" => crate::args_skills::parse_checkpoint_subcommand_args(&rest[1..]),
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
        // F-7 / Slice A4: `elai template-ranking list [--json]` — top-level
        // subcommand (not a REPL slash command; mirrors `archive` / `ppm`
        // local-parser shape, no slash-command grammar entry needed).
        "template-ranking" => parse_template_ranking_subcommand_args(&rest[1..]),
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
        // WIRING-FIX-ELAI Phase 2 (SP3): `elai wiring-coverage [--manifest PATH]
        // [--diff-base REF]`. Top-level subcommand (not a REPL slash command):
        // local arg parse, mirrors `archive`. Validation lives in the handler.
        "wiring-coverage" => parse_wiring_coverage_subcommand_args(&rest[1..], output_format),
        // D-8: `elai archive verify [--path PATH] [--json] [--fail-fast]`
        // Surfaces ArchiveChain::verify_chain (D-1) as a CLI subcommand +
        // CI hook. Parser is local (no slash-command grammar needed --
        // this is a top-level `elai archive verify` invocation, not a
        // REPL slash command).
        "archive" => parse_archive_subcommand_args(&rest[1..]),
        // D-1: `elai template migrate [--source DIR] [--path PATH] [--json]`
        // Bootstraps the reasoning-template archive (JSONL) + emits one
        // ArchiveEntryAdded SSB row per appended template. Top-level subcommand
        // mirroring `archive verify` (operator / CI surface, not a slash
        // command). Sub-verb is `migrate`; the parser errors on unknown verbs.
        "template" => parse_template_subcommand_args(&rest[1..]),
        // C-40: `elai pr <open|list|status|close> [args...]` — GitHub PR
        // adapter CLI surface. Sub-action validation lives in the
        // handler (mirrors `Mcp` / `Runtime` string-typed pattern).
        "pr" => crate::args_pr::parse_pr_subcommand_args(&rest[1..]),
        // E-41: `elai pr-review <sub> [args...]` — PR review surface.
        // Distinct top-level subcommand so the args layer can stay
        // minimal; `pr` continues to dispatch the C-40 open/list/status/
        // close surface untouched.
        "pr-review" => crate::args_pr::parse_pr_review_subcommand_args(&rest[1..]),
        // H-17 — `elai trial-search` is the W-3 Tier-1 fixture entrypoint that
        // proves register_trial fires from a real CLI subcommand (not test-only).
        "trial-search" => parse_trial_search_args(&rest[1..]),
        // H-18 — `elai deflation-check` is the W-3 Tier-1 production entrypoint
        // for the Deflated Sharpe / PBO / MinBTL promotion gate. Registers a
        // fixture strategy, calls evaluate_deflation, emits DeflationGateEvaluated.
        "deflation-check" => parse_deflation_check_args(&rest[1..]),
        // H-20 — `elai leakage-check` is the W-1 production entrypoint for the
        // leakage static-analysis COMPOSITE gate. Runs evaluate_leakage over a
        // target module, emits a LeakageBlocked SSB row (off-agent artifact).
        "leakage-check" => parse_leakage_check_args(&rest[1..]),
        // H-21 — `elai lookahead-check` is the W-1 production entrypoint for the
        // runtime look-ahead / causality DIFFERENTIAL gate. Diffs full vs
        // future-truncated column-sets, emits a LookaheadDifferential SSB row.
        "lookahead-check" => parse_lookahead_check_args(&rest[1..]),
        // H-22 — `elai null-test` is the W-1 production entrypoint for the
        // trading metamorphic NULL-TEST gate (does the edge survive a
        // block-permutation null?). Emits a TradingNullTest SSB row.
        "null-test" => parse_null_test_args(&rest[1..]),
        // H-26 — `elai shadow-check` is the W-1 production entrypoint for the
        // shadow-exec + financial-invariant metamorphic-relation gate (does a
        // financial computation hold its textbook-correct invariants AND agree with a
        // shadow re-implementation within a LOCKED band?). Emits a
        // DifferentialDivergence SSB row.
        "shadow-check" => parse_shadow_check_args(&rest[1..]),
        // H-27 — `elai gate-selfcheck` is the W-1/W-3 production entrypoint for the
        // verify-the-verifiers META gate (do the 8 OTHER Track-3 gates each KILL their
        // own planted-violation fixture?). Emits a GateSelfValidation SSB row.
        "gate-selfcheck" => parse_gate_selfcheck_args(&rest[1..]),
        // H-28 — `elai reward-check` is the W-1/W-3 production entrypoint for the
        // current-RF reward-tampering gate (does a self-mod edit its own evaluator
        // (hash mismatch) or jump the reward implausibly with the hash intact?). Emits
        // a RewardTampering SSB row.
        "reward-check" => parse_reward_check_args(&rest[1..]),
        "holdout-check" => parse_holdout_check_args(&rest[1..]),
        // H-29 — `elai drift-check` is the W-1/W-3 production entrypoint for the LOCKED
        // fail-closed drift state machine (T-13, the master deployment-trust interlock). Plants
        // a clean OR drifted fixture deployment and emits a DriftInterlock SSB row; latches
        // DriftHalted (exit 1) on any axis breach. Naming: NOT `drift` (E-45 source drift) NOR
        // `model-drift` (G-10) — a new compound verb.
        "drift-check" => parse_drift_check_args(&rest[1..]),
        // H-31 — `elai remember "<text>"` captures a durable WORK preference into the bi-temporal
        // user-memory ledger and emits a UserDecisionSuperseded SSB row (the off-agent done-gate
        // artifact). Gated behind ELAI_USER_MEMORY_ENABLE (R9, default-OFF).
        "remember" => parse_remember_args(&rest[1..]),
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
        "incident" => crate::args_memory::parse_incident_subcommand_args(&rest[1..]),
        // E-5 session-2: `elai memory <sub>` — heuristic-only inspection
        // surface over the per-project memory_facts store.
        "memory" => crate::args_memory::parse_memory_subcommand_args(&rest[1..]),
        // I-39 — `elai plan status [--letter <L>] [--json]`. Per-plan state derived
        // from three deterministic sources: FINISH fragments + wiring_contract_check.py
        // + SSB journal. Emits one PlanStateDerived SSB row (fail-soft if no ssb.db).
        // Handler at crates/cli/src/plan_status_cmd.rs.
        "plan" => parse_plan_subcommand_args(&rest[1..], output_format),
        // I-41 — `elai critique <spec-path> [--lenses L1,...] [--risk simple|heavy] [--mock]`.
        // Spec-time critique lenses (L1-L8), advisory only, zero gate-verdict path.
        // Kill-switch: ELAI_CRITIQUE_LENSES_ENABLE=1. Handler at crates/cli/src/critique_cmd.rs.
        "critique" => parse_critique_args(&rest[1..], output_format),
        // I-32 — `elai role-ab-probe [--seed N]`. Role-contract A/B harness.
        // Loads fixtures from crates/contracts/tests/fixtures/role_contracts/,
        // assembles system-prompt sections with/without role contracts, computes
        // pass-rates, emits RoleContractApplied(AbDelta) SSB rows. Evidence gate
        // (ID-5): new_pass_rate >= old_pass_rate required; exits non-zero on regression.
        // Handler at crates/cli/src/role_ab_probe_cmd.rs.
        "role-ab-probe" => parse_role_ab_probe_args(&rest[1..]),
        // I-40 — `elai wrap [--output <path>] [--json] -- <agent-cmd...>` is the
        // trust-layer wrapper proxy. Spins up the taint + write-gate trust substrate
        // for an external agent command and emits a .rra bundle. FORK-1: this is a
        // NEW top-level CliAction::Wrap with its own Arc<Ssb>; it does NOT extend
        // run_subcommand. Kill-switch: ELAI_WRAP_RRA_DISABLE=1.
        "wrap" => parse_wrap_subcommand_args(&rest[1..], output_format),
        // K-6 — `elai export compliance [--session <id>] [--out <path>] [--zero-retention]`
        "export" => parse_export_subcommand_args(&rest[1..], output_format),
        // J-26 S5 (slice S4): `elai eval-corpus-ingest --pack <dir>
        // [--root <corpus-root>]` — load a TaskPack's task.toml and call
        // ingest_task. Exit 0 Accepted / 1 Rejected/Err.
        "eval-corpus-ingest" | "eval_corpus_ingest" => {
            parse_eval_corpus_ingest_args(&rest[1..])
        }
        // J-72 slice C: `elai retrieval gold-build ...` — Tier-1 gold-set
        // assembler. Parser in args_retrieval.rs.
        "retrieval" => crate::args_retrieval::parse_retrieval_subcommand_args(&rest[1..]),
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
    // I-42: --profile orchestrate|critique-only (run-profile axis: who writes
    // + whether executor/fixer agents are spawned). Default orchestrate.
    let mut profile: Option<String> = None;
    // G-2: --emit-rra is a flag that may optionally take a path. A bare
    // `--emit-rra` enables the default-path seal (and is harmless since the
    // RRA is default-ON); `--emit-rra <path>` / `--emit-rra=<path>` pins a path.
    let mut emit_rra: Option<PathBuf> = None;
    // I-44: --zero-retention is a boolean flag (no value). Default false =
    // fail-safe retain-everything.
    let mut zero_retention: bool = false;
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
            // I-42: --profile orchestrate|critique-only (run-profile axis).
            "--profile" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --profile".to_string())?;
                let lower = value.trim().to_ascii_lowercase();
                if !matches!(lower.as_str(), "orchestrate" | "critique-only") {
                    return Err(format!(
                        "unsupported value for --profile: {value} (expected orchestrate or critique-only)"
                    ));
                }
                profile = Some(lower);
                i += 2;
            }
            flag if flag.starts_with("--profile=") => {
                let value = &flag[10..];
                let lower = value.trim().to_ascii_lowercase();
                if !matches!(lower.as_str(), "orchestrate" | "critique-only") {
                    return Err(format!(
                        "unsupported value for --profile: {value} (expected orchestrate or critique-only)"
                    ));
                }
                profile = Some(lower);
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
            // G-2: --emit-rra [<path>] — flag with an OPTIONAL path. A following
            // token that does not look like a flag is consumed as the path.
            "--emit-rra" => {
                match rest.get(i + 1) {
                    Some(next) if !next.starts_with("--") => {
                        emit_rra = Some(PathBuf::from(next));
                        i += 2;
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            flag if flag.starts_with("--emit-rra=") => {
                emit_rra = Some(PathBuf::from(&flag[11..]));
                i += 1;
            }
            // I-44: --zero-retention (boolean flag, no value).
            "--zero-retention" => {
                zero_retention = true;
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
    Ok(CliAction::Run { project_root, mode, provider, via_mcp, empire, export_ssb, emit_rra, profile, zero_retention })
}

/// E-49 SWE-Bench Pro v5 driver — parse `elai task <problem_statement>
/// --repo-root PATH --mode local|hybrid|cloud --output-format json|text
/// --max-wall-seconds N [--executor-runtime BACKEND]
/// [--auditor-runtime BACKEND]`.
///
/// The first positional argument is the problem statement; all subsequent
/// `--flag value` pairs configure the run. Mirrors the symbol_impact
/// flag-parsing style (no `clap` dep) and validates values at parse time
/// so the dispatch arm sees a clean struct.
pub(crate) fn parse_task_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    if rest.is_empty() {
        return Err(
            "usage: elai task <problem_statement> --repo-root PATH \
             --mode local|hybrid|cloud --output-format json|text \
             --max-wall-seconds N [--executor-runtime BACKEND] \
             [--auditor-runtime BACKEND]"
                .to_string(),
        );
    }
    // Collect positional fragments + flagged args. We accept the
    // problem-statement as either a single quoted positional OR multiple
    // bare words before the first --flag (so `elai task fix the bug
    // --repo-root ...` works as a convenience form).
    let mut positional: Vec<String> = Vec::new();
    let mut repo_root: Option<PathBuf> = None;
    let mut mode: Option<crate::task_cmd::TaskMode> = None;
    let mut output_format: Option<crate::task_cmd::TaskOutputFormat> = None;
    let mut max_wall_seconds: Option<u64> = None;
    let mut executor_runtime: Option<String> = None;
    let mut auditor_runtime: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--repo-root" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --repo-root".to_string())?;
                repo_root = Some(PathBuf::from(value));
                i += 2;
            }
            flag if flag.starts_with("--repo-root=") => {
                repo_root = Some(PathBuf::from(&flag[12..]));
                i += 1;
            }
            "--mode" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --mode".to_string())?;
                mode = Some(crate::task_cmd::TaskMode::parse(value)?);
                i += 2;
            }
            flag if flag.starts_with("--mode=") => {
                mode = Some(crate::task_cmd::TaskMode::parse(&flag[7..])?);
                i += 1;
            }
            "--output-format" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --output-format".to_string())?;
                output_format = Some(crate::task_cmd::TaskOutputFormat::parse(value)?);
                i += 2;
            }
            flag if flag.starts_with("--output-format=") => {
                output_format =
                    Some(crate::task_cmd::TaskOutputFormat::parse(&flag[16..])?);
                i += 1;
            }
            "--max-wall-seconds" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --max-wall-seconds".to_string())?;
                max_wall_seconds = Some(value.parse::<u64>().map_err(|e| {
                    format!("--max-wall-seconds must be a positive integer: {e}")
                })?);
                i += 2;
            }
            flag if flag.starts_with("--max-wall-seconds=") => {
                max_wall_seconds = Some(flag[19..].parse::<u64>().map_err(|e| {
                    format!("--max-wall-seconds must be a positive integer: {e}")
                })?);
                i += 1;
            }
            "--executor-runtime" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --executor-runtime".to_string())?;
                executor_runtime = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--executor-runtime=") => {
                executor_runtime = Some(flag[19..].to_string());
                i += 1;
            }
            "--auditor-runtime" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --auditor-runtime".to_string())?;
                auditor_runtime = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--auditor-runtime=") => {
                auditor_runtime = Some(flag[18..].to_string());
                i += 1;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown `task` argument: {other}"));
            }
            other => {
                positional.push(other.to_string());
                i += 1;
            }
        }
    }
    let problem_statement = if positional.is_empty() {
        return Err("`task` requires a problem-statement positional argument".to_string());
    } else {
        positional.join(" ")
    };
    let repo_root = repo_root.ok_or_else(|| "missing required --repo-root".to_string())?;
    let mode = mode.ok_or_else(|| "missing required --mode".to_string())?;
    let output_format =
        output_format.unwrap_or(crate::task_cmd::TaskOutputFormat::Json);
    let max_wall_seconds = max_wall_seconds.unwrap_or(600);
    if max_wall_seconds == 0 {
        return Err("--max-wall-seconds must be > 0".to_string());
    }
    Ok(CliAction::Task {
        problem_statement,
        repo_root,
        mode,
        output_format,
        max_wall_seconds,
        executor_runtime,
        auditor_runtime,
    })
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

/// Parse `elai production-checklist [--project-root PATH]
/// [--output-format text|json|toml]`. Mirrors the structural pattern of
/// `parse_doctor_subcommand_args` (flag-loop with `--foo value` and `--foo=value`
/// shapes; unknown-flag rejection; `--json` alias).
pub(crate) fn parse_production_checklist_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut project_root: Option<PathBuf> = None;
    let mut output_format_override: Option<CliOutputFormat> = None;
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
            "--output-format" => {
                let value = rest
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
                output_format_override = Some(CliOutputFormat::Json);
                i += 1;
            }
            other => {
                return Err(format!(
                    "unknown `production-checklist` argument: {other}"
                ));
            }
        }
    }

    let project_root = match project_root {
        Some(p) => p,
        None => std::env::current_dir().map_err(|e| {
            format!(
                "`production-checklist` cannot determine cwd for --project-root default: {e}"
            )
        })?,
    };
    Ok(CliAction::ProductionChecklist {
        project_root,
        output_format: output_format_override.unwrap_or(output_format),
    })
}

/// Parse `elai install [--skip-models] [--reset-config] [--project-root PATH]
/// [--output-format text|json|toml]`. Mirrors the structural pattern of
/// `parse_doctor_subcommand_args` (flag-loop with `--foo value` and `--foo=value`
/// shapes; unknown-flag rejection; `--json` alias).
pub(crate) fn parse_install_subcommand(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut skip_models = false;
    let mut reset_config = false;
    let mut project_root: Option<PathBuf> = None;
    let mut output_format_override: Option<CliOutputFormat> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--skip-models" => {
                skip_models = true;
                i += 1;
            }
            "--reset-config" => {
                reset_config = true;
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
                project_root = Some(PathBuf::from(&flag[15..]));
                i += 1;
            }
            "--output-format" => {
                let value = rest
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
                output_format_override = Some(CliOutputFormat::Json);
                i += 1;
            }
            other => {
                return Err(format!("unknown `install` argument: {other}"));
            }
        }
    }
    Ok(CliAction::Install {
        skip_models,
        reset_config,
        project_root,
        output_format: output_format_override.unwrap_or(output_format),
    })
}

pub(crate) fn parse_output_format_value(value: &str) -> Result<CliOutputFormat, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "text" => Ok(CliOutputFormat::Text),
        "json" => Ok(CliOutputFormat::Json),
        "toml" => Ok(CliOutputFormat::Toml),
        other => Err(format!(
            "unsupported value for --output-format: {other} (expected text, json, or toml)"
        )),
    }
}

/// WIRING-FIX-ELAI Phase 2 (SP3): parse `elai wiring-coverage` args. Local
/// parser (mirrors `archive`). Recognises `--manifest <path>` / `--manifest=<p>`,
/// `--diff-base <ref>` / `--diff-base=<ref>`. Unknown args are an error.
///
/// ADR-6: `rest: &[String]` (NOT `&[&str]`) to match the dispatch-site `&rest[1..]`
/// slice type; indexes via `rest[i].as_str()` and missing flag values via
/// `rest.get(i + 1).ok_or(..)?` (no `.unwrap()` per CLAUDE.md no-unwrap rule).
fn parse_wiring_coverage_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut manifest_path: Option<String> = None;
    let mut diff_base: Option<String> = None;
    let mut i = 0usize;
    while i < rest.len() {
        match rest[i].as_str() {
            "--manifest" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "--manifest requires a path value".to_string())?;
                manifest_path = Some(v.clone());
                i += 2;
            }
            flag if flag.starts_with("--manifest=") => {
                manifest_path = Some(flag["--manifest=".len()..].to_string());
                i += 1;
            }
            "--diff-base" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "--diff-base requires a ref value".to_string())?;
                diff_base = Some(v.clone());
                i += 2;
            }
            flag if flag.starts_with("--diff-base=") => {
                diff_base = Some(flag["--diff-base=".len()..].to_string());
                i += 1;
            }
            other => {
                return Err(format!(
                    "elai wiring-coverage: unknown argument `{other}`. \
                     Use --manifest, --diff-base."
                ));
            }
        }
    }
    Ok(CliAction::WiringCoverage {
        manifest_path,
        diff_base,
        output_format,
    })
}

/// D-8 — parse `elai archive verify [--path PATH] [--json] [--fail-fast]`.
///
/// Only one sub-verb is currently supported (`verify`). Other archive
/// sub-verbs (e.g. compact, inspect) are out-of-scope; the parser errors out
/// on unknown sub-verbs so a typo doesn't silently fall through to the
/// fallback Prompt action.
/// J-30 T5/T6 -- `elai providers configure [...]` (T5) or
/// `elai providers status [--backend <name>] [--json]` (T6). Local
/// parser (FORK D: not a slash command; the existing `/providers`
/// slash command is listing-only and stays untouched). Supports both
/// `--flag value` and `--flag=value` forms. Validation of the backend
/// name / key requirement lives in the handler (providers_cmd.rs) so
/// this layer stays free of any providers/orchestrator type dependency.
pub(crate) fn parse_providers_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    match sub {
        "configure" => parse_providers_configure_args(rest, output_format),
        "status" => parse_providers_status_args(rest, output_format),
        _ => Err(format!(
            "unknown `providers` sub-command: '{sub}' (expected 'configure' or 'status')"
        )),
    }
}

fn parse_providers_configure_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut backend: Option<String> = None;
    let mut key: Option<String> = None;
    let mut base_url: Option<String> = None;
    let mut model: Option<String> = None;
    let mut i = 1;
    while i < rest.len() {
        let arg = rest[i].as_str();
        let (flag, inline) = if let Some(eq) = arg.find('=') {
            (&arg[..eq], Some(arg[eq + 1..].to_string()))
        } else {
            (arg, None)
        };
        let take_value =
            |inline: Option<String>, i: &mut usize| -> Result<String, String> {
                if let Some(v) = inline {
                    Ok(v)
                } else {
                    let v = rest
                        .get(*i + 1)
                        .ok_or_else(|| format!("missing value for {flag}"))?
                        .clone();
                    *i += 1;
                    Ok(v)
                }
            };
        match flag {
            "--backend" => backend = Some(take_value(inline, &mut i)?),
            "--key" => key = Some(take_value(inline, &mut i)?),
            "--base-url" => base_url = Some(take_value(inline, &mut i)?),
            "--model" => model = Some(take_value(inline, &mut i)?),
            _ => return Err(format!("unknown providers flag: {arg}")),
        }
        i += 1;
    }
    Ok(CliAction::Providers {
        backend,
        key,
        base_url,
        model,
        output_format,
    })
}

fn parse_providers_status_args(
    rest: &[String],
    mut output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut backend: Option<String> = None;
    let mut i = 1;
    while i < rest.len() {
        let arg = rest[i].as_str();
        let (flag, inline) = if let Some(eq) = arg.find('=') {
            (&arg[..eq], Some(arg[eq + 1..].to_string()))
        } else {
            (arg, None)
        };
        let take_value =
            |inline: Option<String>, i: &mut usize| -> Result<String, String> {
                if let Some(v) = inline {
                    Ok(v)
                } else {
                    let v = rest
                        .get(*i + 1)
                        .ok_or_else(|| format!("missing value for {flag}"))?
                        .clone();
                    *i += 1;
                    Ok(v)
                }
            };
        match flag {
            "--backend" => backend = Some(take_value(inline, &mut i)?),
            "--json" => output_format = CliOutputFormat::Json,
            _ => return Err(format!("unknown providers status flag: {arg}")),
        }
        i += 1;
    }
    Ok(CliAction::ProvidersStatus {
        backend,
        output_format,
    })
}

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

/// D-1 — parse `elai template migrate [--source DIR] [--path PATH] [--json]`.
///
/// Only one sub-verb is supported (`migrate`); the parser errors on unknown
/// sub-verbs so a typo does not silently fall through to the fallback Prompt
/// action. Mirrors `parse_archive_subcommand_args` (local parser, top-level
/// subcommand, no slash-command grammar).
pub(crate) fn parse_template_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    if sub != "migrate" {
        return Err(format!(
            "unknown `template` sub-command: '{sub}' (expected 'migrate')"
        ));
    }
    let mut source: Option<String> = None;
    let mut path: Option<String> = None;
    let mut json = false;
    let mut i = 1;
    while i < rest.len() {
        match rest[i].as_str() {
            "--source" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --source".to_string())?;
                source = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--source=") => {
                source = Some(flag["--source=".len()..].to_string());
                i += 1;
            }
            "--path" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --path".to_string())?;
                path = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--path=") => {
                path = Some(flag["--path=".len()..].to_string());
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `template migrate` argument: {other}"));
            }
        }
    }
    Ok(CliAction::TemplateMigrate { source, path, json })
}

/// F-7 / Slice A4 — parse `elai template-ranking <action> [--json]`.
///
/// `action` is the optional lower-cased subcommand (`list` | `help` | absent =>
/// list); validation of unknown actions lives at the handler boundary in
/// `crate::template_ranking_cmd::run_template_ranking_subcommand`. Mirrors the
/// `parse_archive_subcommand_args` local-parser shape (top-level subcommand, no
/// slash-command grammar).
pub(crate) fn parse_template_ranking_subcommand_args(
    rest: &[String],
) -> Result<CliAction, String> {
    let mut action: Option<String> = None;
    let mut json = false;
    for arg in rest {
        match arg.as_str() {
            "--json" => json = true,
            flag if flag.starts_with("--") => {
                return Err(format!("unknown `template-ranking` argument: {flag}"));
            }
            sub => {
                if action.is_some() {
                    return Err(format!(
                        "unexpected extra `template-ranking` argument: {sub}"
                    ));
                }
                action = Some(sub.to_ascii_lowercase());
            }
        }
    }
    Ok(CliAction::TemplateRanking { action, json })
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

/// H-17 — parse `elai trial-search [--objective OBJ] [--window WIN] [--json]`.
///
/// The W-3 Tier-1 fixture entrypoint for the persisted trial ledger. Registers
/// >=3 distinct strategy configurations through `TrialLedger::register_trial`
/// and prints the resulting trial count N. Default-ON; no env override needed.
pub(crate) fn parse_trial_search_args(rest: &[String]) -> Result<CliAction, String> {
    let mut objective: Option<String> = None;
    let mut window: Option<String> = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--objective" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --objective".to_string())?;
                objective = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--objective=") => {
                objective = Some(flag["--objective=".len()..].to_string());
                i += 1;
            }
            "--window" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --window".to_string())?;
                window = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--window=") => {
                window = Some(flag["--window=".len()..].to_string());
                i += 1;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `trial-search` argument: {other}"));
            }
        }
    }
    Ok(CliAction::TrialSearch { objective, window, json })
}

/// H-18 — parse `elai deflation-check [--sr <f64>] [--t <u64>] [--n <u64>]
/// [--sharpe-std <f64>] [--pbo <f64>] [--seed <u64>] [--json]`.
///
/// f64 fields stored as Option<String> (CliAction derives Eq; f64 does not).
/// Parsed to f64 in the handler. Values validated at parse time.
pub(crate) fn parse_deflation_check_args(rest: &[String]) -> Result<CliAction, String> {
    let mut sr: Option<String> = None;
    let mut t_periods: Option<u64> = None;
    let mut n_fixture: Option<u64> = None;
    let mut sharpe_std: Option<String> = None;
    let mut pbo: Option<String> = None;
    let mut seed: Option<u64> = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--sr" => {
                let v = rest.get(i + 1).ok_or_else(|| "missing value for --sr".to_string())?;
                v.parse::<f64>().map_err(|_| format!("invalid f64 for --sr: {v}"))?;
                sr = Some(v.clone());
                i += 2;
            }
            "--t" => {
                let v = rest.get(i + 1).ok_or_else(|| "missing value for --t".to_string())?;
                t_periods = Some(v.parse().map_err(|_| format!("invalid u64 for --t: {v}"))?);
                i += 2;
            }
            "--n" => {
                let v = rest.get(i + 1).ok_or_else(|| "missing value for --n".to_string())?;
                n_fixture = Some(v.parse().map_err(|_| format!("invalid u64 for --n: {v}"))?);
                i += 2;
            }
            "--sharpe-std" => {
                let v = rest.get(i + 1).ok_or_else(|| "missing value for --sharpe-std".to_string())?;
                v.parse::<f64>().map_err(|_| format!("invalid f64 for --sharpe-std: {v}"))?;
                sharpe_std = Some(v.clone());
                i += 2;
            }
            "--pbo" => {
                let v = rest.get(i + 1).ok_or_else(|| "missing value for --pbo".to_string())?;
                v.parse::<f64>().map_err(|_| format!("invalid f64 for --pbo: {v}"))?;
                pbo = Some(v.clone());
                i += 2;
            }
            "--seed" => {
                let v = rest.get(i + 1).ok_or_else(|| "missing value for --seed".to_string())?;
                seed = Some(v.parse().map_err(|_| format!("invalid u64 for --seed: {v}"))?);
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `deflation-check` argument: {other}"));
            }
        }
    }
    Ok(CliAction::DeflationCheck { sr, t_periods, n_fixture, sharpe_std, pbo, seed, json })
}

/// H-20 — parse `elai leakage-check [--module <path>] [--no-split]
/// [--demo-fit-reaches-both] [--residue-temporal] [--seed <u64>] [--json]`.
pub(crate) fn parse_leakage_check_args(rest: &[String]) -> Result<CliAction, String> {
    let mut module: Option<String> = None;
    let mut no_split = false;
    let mut demo_fit_both = false;
    let mut residue_temporal = false;
    let mut seed: Option<u64> = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--module" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --module".to_string())?;
                module = Some(v.clone());
                i += 2;
            }
            "--no-split" => {
                no_split = true;
                i += 1;
            }
            "--demo-fit-reaches-both" => {
                demo_fit_both = true;
                i += 1;
            }
            "--residue-temporal" => {
                residue_temporal = true;
                i += 1;
            }
            "--seed" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = Some(v.parse().map_err(|_| format!("invalid u64 for --seed: {v}"))?);
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `leakage-check` argument: {other}"));
            }
        }
    }
    Ok(CliAction::LeakageCheck {
        module,
        no_split,
        demo_fit_both,
        residue_temporal,
        seed,
        json,
    })
}

/// H-21 — parse `elai lookahead-check (--demo-future-leak | --demo-causal)
/// [--seed <u64>] [--json]`.
pub(crate) fn parse_lookahead_check_args(rest: &[String]) -> Result<CliAction, String> {
    let mut demo_future_leak = false;
    let mut demo_causal = false;
    let mut seed: Option<u64> = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--demo-future-leak" => {
                demo_future_leak = true;
                i += 1;
            }
            "--demo-causal" => {
                demo_causal = true;
                i += 1;
            }
            "--seed" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = Some(v.parse().map_err(|_| format!("invalid u64 for --seed: {v}"))?);
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `lookahead-check` argument: {other}"));
            }
        }
    }
    if demo_future_leak && demo_causal {
        return Err(
            "--demo-future-leak and --demo-causal are mutually exclusive".to_string(),
        );
    }
    if !demo_future_leak && !demo_causal {
        return Err(
            "lookahead-check requires one of --demo-future-leak or --demo-causal".to_string(),
        );
    }
    Ok(CliAction::LookaheadCheck {
        demo_future_leak,
        demo_causal,
        seed,
        json,
    })
}

/// H-22 — parse `elai null-test (--demo-overfit | --demo-genuine)
/// [--seed <u64>] [--json]`.
pub(crate) fn parse_null_test_args(rest: &[String]) -> Result<CliAction, String> {
    let mut demo_overfit = false;
    let mut demo_genuine = false;
    let mut seed: Option<u64> = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--demo-overfit" => {
                demo_overfit = true;
                i += 1;
            }
            "--demo-genuine" => {
                demo_genuine = true;
                i += 1;
            }
            "--seed" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = Some(v.parse().map_err(|_| format!("invalid u64 for --seed: {v}"))?);
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `null-test` argument: {other}"));
            }
        }
    }
    if demo_overfit && demo_genuine {
        return Err("--demo-overfit and --demo-genuine are mutually exclusive".to_string());
    }
    if !demo_overfit && !demo_genuine {
        return Err("null-test requires one of --demo-overfit or --demo-genuine".to_string());
    }
    Ok(CliAction::NullTest {
        demo_overfit,
        demo_genuine,
        seed,
        json,
    })
}

/// H-26 — parse `elai shadow-check (--demo-invariant-held | --demo-invariant-violated)
/// [--seed <u64>] [--json]`.
pub(crate) fn parse_shadow_check_args(rest: &[String]) -> Result<CliAction, String> {
    let mut demo_invariant_held = false;
    let mut demo_invariant_violated = false;
    let mut seed: Option<u64> = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--demo-invariant-held" => {
                demo_invariant_held = true;
                i += 1;
            }
            "--demo-invariant-violated" => {
                demo_invariant_violated = true;
                i += 1;
            }
            "--seed" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = Some(v.parse().map_err(|_| format!("invalid u64 for --seed: {v}"))?);
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `shadow-check` argument: {other}"));
            }
        }
    }
    if demo_invariant_held && demo_invariant_violated {
        return Err(
            "--demo-invariant-held and --demo-invariant-violated are mutually exclusive".to_string(),
        );
    }
    if !demo_invariant_held && !demo_invariant_violated {
        return Err(
            "shadow-check requires one of --demo-invariant-held or --demo-invariant-violated"
                .to_string(),
        );
    }
    Ok(CliAction::ShadowCheck {
        demo_invariant_held,
        demo_invariant_violated,
        seed,
        json,
    })
}

/// H-27 — parse `elai gate-selfcheck (--demo-all-valid | --demo-gate-broken)
/// [--seed <u64>] [--json]`.
pub(crate) fn parse_gate_selfcheck_args(rest: &[String]) -> Result<CliAction, String> {
    let mut demo_all_valid = false;
    let mut demo_gate_broken = false;
    let mut seed: Option<u64> = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--demo-all-valid" => {
                demo_all_valid = true;
                i += 1;
            }
            "--demo-gate-broken" => {
                demo_gate_broken = true;
                i += 1;
            }
            "--seed" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = Some(v.parse().map_err(|_| format!("invalid u64 for --seed: {v}"))?);
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `gate-selfcheck` argument: {other}"));
            }
        }
    }
    if demo_all_valid && demo_gate_broken {
        return Err(
            "--demo-all-valid and --demo-gate-broken are mutually exclusive".to_string(),
        );
    }
    if !demo_all_valid && !demo_gate_broken {
        return Err(
            "gate-selfcheck requires one of --demo-all-valid or --demo-gate-broken".to_string(),
        );
    }
    Ok(CliAction::GateSelfcheck {
        demo_all_valid,
        demo_gate_broken,
        seed,
        json,
    })
}

/// H-28 — parse `elai reward-check (--demo-clean | --demo-tampered | --demo-score-jump)
/// [--seed <u64>] [--json]`.
pub(crate) fn parse_reward_check_args(rest: &[String]) -> Result<CliAction, String> {
    let mut demo_clean = false;
    let mut demo_tampered = false;
    let mut demo_score_jump = false;
    let mut seed: Option<u64> = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--demo-clean" => {
                demo_clean = true;
                i += 1;
            }
            "--demo-tampered" => {
                demo_tampered = true;
                i += 1;
            }
            "--demo-score-jump" => {
                demo_score_jump = true;
                i += 1;
            }
            "--seed" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = Some(v.parse().map_err(|_| format!("invalid u64 for --seed: {v}"))?);
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `reward-check` argument: {other}"));
            }
        }
    }
    let selected = u8::from(demo_clean) + u8::from(demo_tampered) + u8::from(demo_score_jump);
    if selected > 1 {
        return Err(
            "--demo-clean, --demo-tampered, and --demo-score-jump are mutually exclusive"
                .to_string(),
        );
    }
    if selected == 0 {
        return Err(
            "reward-check requires one of --demo-clean, --demo-tampered, or --demo-score-jump"
                .to_string(),
        );
    }
    Ok(CliAction::RewardCheck {
        demo_clean,
        demo_tampered,
        demo_score_jump,
        seed,
        json,
    })
}

/// H-19 — parse `elai holdout-check (--demo-spend | --demo-exhausted) [--seed <u64>] [--json]`.
pub(crate) fn parse_holdout_check_args(rest: &[String]) -> Result<CliAction, String> {
    let mut demo_spend = false;
    let mut demo_exhausted = false;
    let mut seed: Option<u64> = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--demo-spend" => {
                demo_spend = true;
                i += 1;
            }
            "--demo-exhausted" => {
                demo_exhausted = true;
                i += 1;
            }
            "--seed" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = Some(v.parse().map_err(|_| format!("invalid u64 for --seed: {v}"))?);
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `holdout-check` argument: {other}"));
            }
        }
    }
    let selected = u8::from(demo_spend) + u8::from(demo_exhausted);
    if selected > 1 {
        return Err("--demo-spend and --demo-exhausted are mutually exclusive".to_string());
    }
    if selected == 0 {
        return Err(
            "holdout-check requires one of --demo-spend or --demo-exhausted".to_string(),
        );
    }
    Ok(CliAction::HoldoutCheck {
        demo_spend,
        demo_exhausted,
        seed,
        json,
    })
}

/// H-29 — parse `elai drift-check (--demo-stable | --demo-drift) [--seed <u64>] [--json]`.
pub(crate) fn parse_drift_check_args(rest: &[String]) -> Result<CliAction, String> {
    let mut demo_stable = false;
    let mut demo_drift = false;
    let mut seed: Option<u64> = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--demo-stable" => {
                demo_stable = true;
                i += 1;
            }
            "--demo-drift" => {
                demo_drift = true;
                i += 1;
            }
            "--seed" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --seed".to_string())?;
                seed = Some(v.parse().map_err(|_| format!("invalid u64 for --seed: {v}"))?);
                i += 2;
            }
            "--json" => {
                json = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `drift-check` argument: {other}"));
            }
        }
    }
    let selected = u8::from(demo_stable) + u8::from(demo_drift);
    if selected > 1 {
        return Err("--demo-stable and --demo-drift are mutually exclusive".to_string());
    }
    if selected == 0 {
        return Err("drift-check requires one of --demo-stable or --demo-drift".to_string());
    }
    Ok(CliAction::DriftCheck {
        demo_stable,
        demo_drift,
        seed,
        json,
    })
}

/// H-31 — parse `elai remember "<text>" [--json]`. The text is the positional remainder (all
/// non-flag tokens joined with a space); `--json` selects machine output. An empty text is an error.
pub(crate) fn parse_remember_args(rest: &[String]) -> Result<CliAction, String> {
    let mut json = false;
    let mut text_parts: Vec<String> = Vec::new();
    for arg in rest {
        match arg.as_str() {
            "--json" => json = true,
            other => text_parts.push(other.to_string()),
        }
    }
    let text = text_parts.join(" ").trim().to_string();
    if text.is_empty() {
        return Err("remember requires non-empty text: `elai remember \"<text>\"`".to_string());
    }
    Ok(CliAction::Remember { text, json })
}

// B-15 Tranche O.2 (orchestrate s-22600): the eight `parse_bench_*_args`
// functions + `validate_bool_arg` helper that previously lived here have
// moved to crate::args_bench. CliAction enum is unchanged; the call site
// at L2018 dispatches directly to `crate::args_bench::parse_bench_subcommand_args`.
// Removed ~722 LOC; see workflow/reasoning/O-trancheO-decisions.md.


/// D-28 — parse `elai farm-adv-rm <run|verdict> [...]`. Mirrors the
/// `parse_bench_subcommand_args` sub-verb dispatch pattern. Numerical
/// knobs (`--max-samples`, `--max-rounds`, `--z-threshold`,
/// `--paper-floor-pp`) are carried as Option<String> so CliAction
/// derives Eq cleanly; validation lives in the `farm_adv_rm` handler
/// boundary.
pub(crate) fn parse_farm_adv_rm_args(rest: &[String]) -> Result<CliAction, String> {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    match sub {
        "run" => parse_farm_adv_rm_run_args(&rest[1..]),
        "verdict" => parse_farm_adv_rm_verdict_args(&rest[1..]),
        other => Err(format!(
            "unknown `farm-adv-rm` sub-command: '{other}' (expected 'run' | 'verdict')"
        )),
    }
}

/// D-28 — parse `elai farm-adv-rm run --dataset PATH [...]`.
fn parse_farm_adv_rm_run_args(rest: &[String]) -> Result<CliAction, String> {
    let mut dataset: Option<String> = None;
    let mut corpus_dir: Option<String> = None;
    let mut output: Option<String> = None;
    let mut max_samples: Option<String> = None;
    let mut max_rounds: Option<String> = None;
    let mut z_threshold: Option<String> = None;
    let mut paper_floor_pp: Option<String> = None;
    let mut source_session: Option<String> = None;
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
            "--corpus-dir" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --corpus-dir".to_string())?;
                corpus_dir = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--corpus-dir=") => {
                corpus_dir = Some(flag["--corpus-dir=".len()..].to_string());
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
            "--max-samples" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --max-samples".to_string())?;
                let _: u32 = value
                    .parse()
                    .map_err(|e| format!("invalid --max-samples {value:?}: {e}"))?;
                max_samples = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--max-samples=") => {
                let value = &flag["--max-samples=".len()..];
                let _: u32 = value
                    .parse()
                    .map_err(|e| format!("invalid --max-samples {value:?}: {e}"))?;
                max_samples = Some(value.to_string());
                i += 1;
            }
            "--max-rounds" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --max-rounds".to_string())?;
                let _: u32 = value
                    .parse()
                    .map_err(|e| format!("invalid --max-rounds {value:?}: {e}"))?;
                max_rounds = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--max-rounds=") => {
                let value = &flag["--max-rounds=".len()..];
                let _: u32 = value
                    .parse()
                    .map_err(|e| format!("invalid --max-rounds {value:?}: {e}"))?;
                max_rounds = Some(value.to_string());
                i += 1;
            }
            "--z-threshold" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --z-threshold".to_string())?;
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --z-threshold {value:?}: {e}"))?;
                z_threshold = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--z-threshold=") => {
                let value = &flag["--z-threshold=".len()..];
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --z-threshold {value:?}: {e}"))?;
                z_threshold = Some(value.to_string());
                i += 1;
            }
            "--paper-floor-pp" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --paper-floor-pp".to_string())?;
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --paper-floor-pp {value:?}: {e}"))?;
                paper_floor_pp = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--paper-floor-pp=") => {
                let value = &flag["--paper-floor-pp=".len()..];
                let _: f64 = value
                    .parse()
                    .map_err(|e| format!("invalid --paper-floor-pp {value:?}: {e}"))?;
                paper_floor_pp = Some(value.to_string());
                i += 1;
            }
            "--source-session" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --source-session".to_string())?;
                source_session = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--source-session=") => {
                source_session = Some(flag["--source-session=".len()..].to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown `farm-adv-rm run` argument: {other}"));
            }
        }
    }
    let dataset = dataset
        .ok_or_else(|| "missing required --dataset PATH for `farm-adv-rm run`".to_string())?;
    Ok(CliAction::FarmAdvRm {
        sub: FarmAdvRmSub::Run {
            dataset,
            corpus_dir,
            output,
            max_samples,
            max_rounds,
            z_threshold,
            paper_floor_pp,
            source_session,
        },
    })
}

/// D-28 — parse `elai farm-adv-rm verdict --report PATH`. Re-evaluates
/// the gate against a previously-written report without re-running the
/// hook loop. No hook calls; no SSB writes.
fn parse_farm_adv_rm_verdict_args(rest: &[String]) -> Result<CliAction, String> {
    let mut report: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--report" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --report".to_string())?;
                report = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--report=") => {
                report = Some(flag["--report=".len()..].to_string());
                i += 1;
            }
            other => {
                return Err(format!("unknown `farm-adv-rm verdict` argument: {other}"));
            }
        }
    }
    let report = report
        .ok_or_else(|| "missing required --report PATH for `farm-adv-rm verdict`".to_string())?;
    Ok(CliAction::FarmAdvRm {
        sub: FarmAdvRmSub::Verdict { report },
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

/// F-5 / Slice A4 — parse `elai dkg staleness-tick --workspace <path>` (W-5).
/// `--workspace` defaults to the current working directory when omitted.
pub(crate) fn parse_dkg_args(rest: &[String]) -> Result<CliAction, String> {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    let flag_rest = if rest.is_empty() { &rest[..] } else { &rest[1..] };

    let mut workspace: Option<PathBuf> = None;
    let mut i = 0;
    while i < flag_rest.len() {
        match flag_rest[i].as_str() {
            "--workspace" => {
                let value = flag_rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for --workspace".to_string())?;
                workspace = Some(PathBuf::from(value));
                i += 2;
            }
            flag if flag.starts_with("--workspace=") => {
                workspace = Some(PathBuf::from(&flag["--workspace=".len()..]));
                i += 1;
            }
            other => {
                return Err(format!("unknown `dkg` argument: {other}"));
            }
        }
    }

    let workspace = match workspace {
        Some(p) => p,
        None => std::env::current_dir().map_err(|e| {
            format!("`dkg` cannot determine cwd for --workspace default: {e}")
        })?,
    };

    match sub {
        "staleness-tick" | "staleness_tick" => Ok(CliAction::DkgStalenessTick { workspace }),
        other => Err(format!(
            "unknown `dkg` sub-command: '{other}' (expected 'staleness-tick')"
        )),
    }
}

/// C-43 — parse the `elai freeze` sub-verb. Grammar:
///   `elai freeze status`
///   `elai freeze verify`
///   `elai freeze ratify --ratifier <id>`
/// ELAI design hypothesis — no paper grounding.
/// dkg-stream-rigorous Task 2: parse `elai index build [--project-root PATH]
/// [--vectors] [--json]`. The only sub-action is `build`. `--project-root`
/// defaults to the current working directory; `--vectors` opts the
/// (memory-bounded, streaming) vector stage into the build.
pub(crate) fn parse_index_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| "missing `index` sub-action; expected: build".to_string())?
        .as_str();
    match sub {
        "build" => {
            let mut project_root: Option<PathBuf> = None;
            let mut vectors = false;
            let mut json = false;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--project-root" => {
                        let v = rest.get(i + 1).ok_or_else(|| {
                            "`--project-root` requires a path argument".to_string()
                        })?;
                        project_root = Some(PathBuf::from(v));
                        i += 2;
                    }
                    flag if flag.starts_with("--project-root=") => {
                        let v = flag.trim_start_matches("--project-root=");
                        project_root = Some(PathBuf::from(v));
                        i += 1;
                    }
                    "--vectors" => {
                        vectors = true;
                        i += 1;
                    }
                    "--json" => {
                        json = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!("unexpected argument to `index build`: {other}"));
                    }
                }
            }
            let project_root = match project_root {
                Some(p) => p,
                None => std::env::current_dir()
                    .map_err(|e| format!("could not resolve current dir for --project-root: {e}"))?,
            };
            Ok(CliAction::IndexBuild {
                project_root,
                vectors,
                json,
            })
        }
        other => Err(format!(
            "unknown `index` sub-action {other:?}; expected: build"
        )),
    }
}

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

/// G-10 (SCHEMA_VERSION 218) — parse `elai model-drift <sub>`. Grammar:
///   `elai model-drift baseline [--project-root PATH] [--json]`
///   `elai model-drift check [--project-root PATH] [--json]`
///   `elai model-drift show [--project-root PATH] [--json]`
///
/// DISTINCT from `elai drift` (E-45 cross-language SOURCE drift checker).
pub(crate) fn parse_model_drift_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .map(String::as_str)
        .ok_or_else(|| {
            "missing `model-drift` sub-action; expected: baseline|check|show".to_string()
        })?;

    let args = &rest[1..];
    let mut project_root = PathBuf::from(".");
    let mut json = matches!(output_format, CliOutputFormat::Json);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json = true;
                i += 1;
            }
            "--project-root" => {
                let value = args.get(i + 1).ok_or_else(|| {
                    "missing value for `--project-root`".to_string()
                })?;
                project_root = PathBuf::from(value);
                i += 2;
            }
            flag if flag.starts_with("--project-root=") => {
                project_root = PathBuf::from(&flag[15..]);
                i += 1;
            }
            other => {
                return Err(format!(
                    "unknown `model-drift {sub}` argument: {other}"
                ));
            }
        }
    }

    let action = match sub {
        "baseline" => ModelDriftAction::Baseline { project_root, json },
        "check" => ModelDriftAction::Check { project_root, json },
        "show" => ModelDriftAction::Show { project_root, json },
        other => {
            return Err(format!(
                "unknown `model-drift` sub-action `{other}`; expected: baseline|check|show"
            ));
        }
    };

    Ok(CliAction::ModelDrift {
        action,
        output_format,
    })
}

/// G-2 + I-40 — parse `elai rra <sub>`. Grammar:
///   `elai rra show <path>`
///   `elai rra verify <path>` (I-40: spec-conformance check, distinct from replay --verify)
pub(crate) fn parse_rra_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| "missing `rra` sub-action; expected: show <path> | verify <path>".to_string())?
        .as_str();
    match sub {
        "show" => {
            let path = rest
                .get(1)
                .ok_or_else(|| "`rra show` requires a <path> argument".to_string())?;
            if rest.len() > 2 {
                return Err(format!("unexpected argument after `rra show <path>`: {}", rest[2]));
            }
            Ok(CliAction::Rra {
                action: RraAction::Show {
                    path: PathBuf::from(path),
                },
                output_format,
            })
        }
        // I-40 — `elai rra verify <path>`: spec-conformance check against
        // rra-spec-v1 (docs/spec/rra-spec-v1.md). DISTINCT from
        // `elai replay --verify` (content-hash re-derivation). Exits nonzero
        // on C1-C6 failures so CI can gate on it.
        "verify" => {
            let path = rest
                .get(1)
                .ok_or_else(|| "`rra verify` requires a <path> argument".to_string())?;
            if rest.len() > 2 {
                return Err(format!(
                    "unexpected argument after `rra verify <path>`: {}",
                    rest[2]
                ));
            }
            Ok(CliAction::Rra {
                action: RraAction::Verify {
                    path: PathBuf::from(path),
                },
                output_format,
            })
        }
        // J-14 — `elai rra export-fixture <bundle> [--adapter <name>]`: export
        // a content-keyed cassette from an RRA bundle. The `--adapter` flag is
        // a passthrough label for J-91 (may be omitted; defaults to "").
        "export-fixture" => {
            let bundle = rest
                .get(1)
                .ok_or_else(|| "`rra export-fixture` requires a <bundle> argument".to_string())?;
            let mut adapter = String::new();
            let mut i = 2;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--adapter" => {
                        i += 1;
                        adapter = rest
                            .get(i)
                            .ok_or_else(|| "`rra export-fixture --adapter` requires a value".to_string())?
                            .clone();
                    }
                    other => {
                        return Err(format!(
                            "unexpected argument after `rra export-fixture <bundle>`: {other}"
                        ));
                    }
                }
                i += 1;
            }
            Ok(CliAction::Rra {
                action: RraAction::ExportFixture {
                    bundle: PathBuf::from(bundle),
                    adapter,
                },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `rra` sub-action `{other}`; expected: show <path> | verify <path> | export-fixture <bundle> [--adapter <name>]"
        )),
    }
}

fn parse_mcp_serve_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    if let Some(argument) = rest.first() {
        return Err(format!("unexpected `mcp-serve` argument: {argument}"));
    }
    Ok(CliAction::McpServe { output_format })
}

/// G-2 — parse `elai replay <sub>`. Grammar:
///   `elai replay --verify <path>`
pub(crate) fn parse_replay_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut verify_path: Option<PathBuf> = None;
    let mut verify_flag = false;
    let mut metric_digest: Option<String> = None;
    let mut golden: Option<String> = None;
    let mut result_manifest: Option<String> = None;
    let mut tier: Option<ReplayVerifyTier> = None;
    // I-45: optional `--current-objective <text>` for the offline `.rra`
    // verifier path. Threads the caller's current objective text into
    // `verify_artifact_with_objective` so `ObjectiveHashMismatch` can fire on
    // a live CLI run. Rejected on the demo gate paths (H-23/H-24/H-25) below
    // because mixing staleness with a demo gate is ambiguous.
    let mut current_objective: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            // H-23: `--verify` may stand alone (paired with `--metric-digest`)
            // OR take a `.rra` path value. Peek the next token: if it is another
            // flag (or absent), this is the bare metric-digest form.
            "--verify" => {
                verify_flag = true;
                match rest.get(i + 1) {
                    Some(next) if !next.starts_with("--") => {
                        verify_path = Some(PathBuf::from(next));
                        i += 2;
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            flag if flag.starts_with("--verify=") => {
                verify_flag = true;
                verify_path = Some(PathBuf::from(&flag[9..]));
                i += 1;
            }
            // H-23: same-seed re-run hash-equality METRIC_GATE (additive).
            "--metric-digest" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for `--metric-digest` (deterministic | nondeterministic)".to_string())?;
                metric_digest = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--metric-digest=") => {
                metric_digest = Some(flag["--metric-digest=".len()..].to_string());
                i += 1;
            }
            // H-24: sim-to-real parity gate path (additive; sibling to
            // `--metric-digest`). Does NOT regress the offline `.rra` verifier
            // or the H-23 re-run gate.
            "--golden" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for `--golden` (parity | gap)".to_string())?;
                golden = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--golden=") => {
                golden = Some(flag["--golden=".len()..].to_string());
                i += 1;
            }
            // H-25: result manifest re-derivation gate path (additive; sibling to
            // `--metric-digest` / `--golden`). Does NOT regress the offline `.rra`
            // verifier or the H-23/H-24 gates.
            "--result-manifest" => {
                let value = rest.get(i + 1).ok_or_else(|| {
                    "missing value for `--result-manifest` (complete | tampered | incomplete)"
                        .to_string()
                })?;
                result_manifest = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--result-manifest=") => {
                result_manifest = Some(flag["--result-manifest=".len()..].to_string());
                i += 1;
            }
            // I-45: `--current-objective <text>` for the offline `.rra`
            // verifier. Threads the caller's current objective text into
            // `verify_artifact_with_objective` so `ObjectiveHashMismatch` can
            // fire on a live CLI run. Rejected below when a demo gate flag
            // (`--metric-digest` / `--golden` / `--result-manifest`) is also
            // present, because mixing staleness with a demo gate is ambiguous.
            "--current-objective" => {
                let value = rest.get(i + 1).ok_or_else(|| {
                    "missing value for `--current-objective <text>`".to_string()
                })?;
                current_objective = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--current-objective=") => {
                current_objective = Some(flag["--current-objective=".len()..].to_string());
                i += 1;
            }
            // J-14: `--tier t0|t1|t2` for the offline `.rra` verifier. Selects
            // the graded verify tier (`ActionFaithful` / `TokenFaithful` /
            // `ScoreFaithful`). Defaults to T0 when absent. Rejected on the
            // demo gate paths below for the same reason as `--current-objective`.
            "--tier" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for `--tier <t0|t1|t2>`".to_string())?;
                tier = Some(ReplayVerifyTier::parse(value).ok_or_else(|| {
                    format!(
                        "unknown `--tier` value `{value}` (expected `t0`, `t1`, or `t2`)"
                    )
                })?);
                i += 2;
            }
            flag if flag.starts_with("--tier=") => {
                let value = &flag["--tier=".len()..];
                tier = Some(ReplayVerifyTier::parse(value).ok_or_else(|| {
                    format!(
                        "unknown `--tier` value `{value}` (expected `t0`, `t1`, or `t2`)"
                    )
                })?);
                i += 1;
            }
            other => {
                return Err(format!("unknown `replay` argument: {other}"));
            }
        }
    }

    // I-45: `--current-objective` is only meaningful on the offline `.rra`
    // verifier path (`ReplayAction::Verify`). Mixing it with any of the demo
    // gates (`--metric-digest` / `--golden` / `--result-manifest`) is
    // ambiguous — the demo arms do not carry the field and would silently
    // drop it. Fail closed at the parser instead.
    if current_objective.is_some()
        && (metric_digest.is_some() || golden.is_some() || result_manifest.is_some())
    {
        return Err(
            "`--current-objective` cannot be combined with `--metric-digest`, `--golden`, or `--result-manifest` (it only applies to `replay --verify <path>`)".to_string(),
        );
    }

    // J-14: `--tier` is only meaningful on the offline `.rra` verifier path
    // (`ReplayAction::Verify`). The demo gate paths do not carry a tier.
    if tier.is_some()
        && (metric_digest.is_some() || golden.is_some() || result_manifest.is_some())
    {
        return Err(
            "`--tier` cannot be combined with `--metric-digest`, `--golden`, or `--result-manifest` (it only applies to `replay --verify <path>`)".to_string(),
        );
    }

    // H-23: the metric-digest re-run gate path (additive). Requires `--verify`
    // to be present (the WIRING_CONTRACT pins `replay --verify --metric-digest`).
    if let Some(mode_raw) = metric_digest {
        if !verify_flag {
            return Err(
                "`--metric-digest` requires `--verify` (use `replay --verify --metric-digest <mode>`)"
                    .to_string(),
            );
        }
        let demo = match mode_raw.as_str() {
            "deterministic" | "demo-deterministic" => RerunDemoMode::Deterministic,
            "nondeterministic" | "demo-nondeterministic" => RerunDemoMode::Nondeterministic,
            other => {
                return Err(format!(
                    "unknown `--metric-digest` mode `{other}` (expected `deterministic` or `nondeterministic`)"
                ));
            }
        };
        return Ok(CliAction::Replay {
            action: ReplayAction::VerifyRerun { demo },
            output_format,
        });
    }

    // H-24: the sim-to-real parity gate path (additive). Requires `--verify` to
    // be present (the WIRING_CONTRACT pins `replay --verify --golden`).
    if let Some(mode_raw) = golden {
        if !verify_flag {
            return Err(
                "`--golden` requires `--verify` (use `replay --verify --golden <mode>`)"
                    .to_string(),
            );
        }
        let demo = match mode_raw.as_str() {
            "parity" | "demo-parity" => GoldenDemoMode::Parity,
            "gap" | "demo-gap" => GoldenDemoMode::Gap,
            other => {
                return Err(format!(
                    "unknown `--golden` mode `{other}` (expected `parity` or `gap`)"
                ));
            }
        };
        return Ok(CliAction::Replay {
            action: ReplayAction::VerifyGolden { demo },
            output_format,
        });
    }

    // H-25: the result manifest re-derivation gate path (additive). Requires
    // `--verify` to be present (the WIRING_CONTRACT pins
    // `replay --verify --result-manifest`).
    if let Some(mode_raw) = result_manifest {
        if !verify_flag {
            return Err(
                "`--result-manifest` requires `--verify` (use `replay --verify --result-manifest <mode>`)"
                    .to_string(),
            );
        }
        let demo = match mode_raw.as_str() {
            "complete" | "demo-complete" => ResultManifestDemoMode::Complete,
            "tampered" | "demo-tampered" => ResultManifestDemoMode::Tampered,
            "incomplete" | "demo-incomplete" => ResultManifestDemoMode::Incomplete,
            other => {
                return Err(format!(
                    "unknown `--result-manifest` mode `{other}` (expected `complete`, `tampered`, or `incomplete`)"
                ));
            }
        };
        return Ok(CliAction::Replay {
            action: ReplayAction::VerifyResultManifest { demo },
            output_format,
        });
    }

    let path = verify_path.ok_or_else(|| {
        "`replay` requires `--verify <path>` (the .rra to verify), or `--verify --metric-digest <mode>` (H-23 re-run hash-equality), or `--verify --golden <mode>` (H-24 sim-to-real parity), or `--verify --result-manifest <mode>` (H-25 result manifest re-derivation)".to_string()
    })?;
    Ok(CliAction::Replay {
        action: ReplayAction::Verify {
            path,
            current_objective,
            tier,
        },
        output_format,
    })
}

/// G-5 — parse `elai review <sub>`. Grammar:
///   `elai review --cockpit [--session <id>] [--base <ref>]
///                [--demo-anchor] [--trust-model-line-numbers]`
///
/// `--cockpit` is mandatory (it is the only sub-action today; a future
/// `--summary` / `--export` lands as a sibling flag). `--session` / `--base`
/// are optional. `--demo-anchor` drives the H-52 W-1 seeded-fixture trigger.
/// `--trust-model-line-numbers` drives the broken-arm R2 proof. Unknown flags
/// are a hard error (fail-closed parse).
pub(crate) fn parse_review_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut cockpit = false;
    let mut session: Option<String> = None;
    let mut base: Option<String> = None;
    let mut demo_anchor = false;
    let mut trust_model_line = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--cockpit" => {
                cockpit = true;
                i += 1;
            }
            "--session" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for `--session`".to_string())?;
                session = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--session=") => {
                session = Some(flag["--session=".len()..].to_string());
                i += 1;
            }
            "--base" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for `--base`".to_string())?;
                base = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--base=") => {
                base = Some(flag["--base=".len()..].to_string());
                i += 1;
            }
            // H-52 W-1 seeded-fixture trigger: injects deterministic snippet-bearing
            // ReviewFindings over a fixed diff fixture so anchor_finding runs on REAL
            // snippets and emits genuine FindingAnchored rows for all three observable-
            // effect cases (Anchored / Ambiguous / Unanchorable). This is the W-1 live-
            // CLI proof; the deferred model-finding source (E-41/C-40) is the future consumer.
            "--demo-anchor" => {
                demo_anchor = true;
                i += 1;
            }
            // H-52 broken-arm flag (R2 proof): forces a model-emitted line for the Ambiguous
            // case, intentionally mis-anchoring it to prove the resolver is necessary.
            "--trust-model-line-numbers" => {
                trust_model_line = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `review` argument: {other}"));
            }
        }
    }
    if !cockpit {
        return Err(
            "`review` requires `--cockpit` (the reviewer-cockpit DSR producer)".to_string(),
        );
    }
    Ok(CliAction::Review {
        action: ReviewAction::Cockpit {
            session,
            base,
            demo_anchor,
            trust_model_line,
        },
        output_format,
    })
}

/// H-53 — parse `elai prebind [--demo-prebind] [--allow-freeform-citations]`.
///
/// `--demo-prebind` (default if no flags supplied): H-53 W-1 seeded-fixture
/// trigger. Builds a deterministic evidence set, calls `freeze_anchors`, runs
/// closed-set check (Bound/Ghost/Stale), emits a genuine
/// `CitationAnchorsBound` SSB row (off-agent artifact, R4).
///
/// `--allow-freeform-citations`: BROKEN-ARM probe (R2 proof). Bypasses the
/// closed-set check; fires the Blocked arm of `CitationAnchorsBoundKind`.
///
/// Unknown flags are a hard error (fail-closed parse).
pub(crate) fn parse_prebind_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut allow_freeform = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--demo-prebind" => {
                // H-53 W-1 seeded-fixture trigger. Accepted but redundant (it is the
                // default behaviour); present so `elai prebind --demo-prebind` reads
                // naturally and matches the plan's documented CLI surface exactly.
                i += 1;
            }
            // BROKEN-ARM probe: bypasses closed-set check; fires Blocked SSB arm.
            "--allow-freeform-citations" => {
                allow_freeform = true;
                i += 1;
            }
            other => {
                return Err(format!("unknown `prebind` argument: {other}"));
            }
        }
    }
    Ok(CliAction::Prebind {
        allow_freeform,
        output_format,
    })
}

/// G-8 — parse `elai verify-attestation <commit>`. Grammar:
///   `elai verify-attestation <commit> [--chain]`
///
/// The commit is a single positional argument (mirrors `rra show <path>`).
/// `--chain` (I-18) is an optional flag that enables the per-row blake3 chain
/// walk alongside attestation re-verification. A missing commit is a hard
/// error; an unexpected extra argument (after consuming `--chain`) is a hard
/// error (fail-closed parse).
pub(crate) fn parse_verify_attestation_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    // I-18: strip any `--chain` flag from the positional argument list so it
    // doesn't break the existing single-positional-arg grammar. The flag is
    // optional and position-independent relative to <commit>.
    //
    // I-45: also strip `--current-objective <text>` and
    // `--current-objective=<text>` from the positional stream. The value is
    // threaded into `verify_attestation_against_trailer_with_objective` so
    // `AttestationVerdict::ReceiptStale` can fire on a live CLI run. A stateful
    // walk is required so the space-separated form's value token is also
    // removed from `positional` (a plain `filter` would keep it and trip the
    // "unexpected argument after <commit>" guard below).
    let mut chain = false;
    let mut current_objective: Option<String> = None;
    let mut positional: Vec<&String> = Vec::with_capacity(rest.len());
    let mut k = 0;
    while k < rest.len() {
        let a = &rest[k];
        if a.as_str() == "--chain" {
            chain = true;
            k += 1;
            continue;
        }
        if let Some(value) = a.strip_prefix("--current-objective=") {
            if current_objective.is_some() {
                return Err(
                    "`--current-objective` given more than once".to_string(),
                );
            }
            current_objective = Some(value.to_string());
            k += 1;
            continue;
        }
        if a.as_str() == "--current-objective" {
            let value = rest.get(k + 1).ok_or_else(|| {
                "missing value for `--current-objective <text>`".to_string()
            })?;
            if current_objective.is_some() {
                return Err(
                    "`--current-objective` given more than once".to_string(),
                );
            }
            current_objective = Some(value.clone());
            k += 2;
            continue;
        }
        positional.push(a);
        k += 1;
    }

    let commit = positional
        .first()
        .ok_or_else(|| {
            "`verify-attestation` requires a <commit> argument (SHA / ref / HEAD)".to_string()
        })?
        .to_string();
    if positional.len() > 1 {
        return Err(format!(
            "unexpected argument after `verify-attestation <commit>`: {}",
            positional[1]
        ));
    }
    Ok(CliAction::VerifyAttestation {
        commit,
        output_format,
        chain,
        current_objective,
    })
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

/// E-39 -- `elai cloud <override-cap|status>` parser.
///
/// `override-cap <USD> --confirm <typed-string>` requires both
/// arguments. The confirmation string is hashed at emit time
/// (blake3-prefix per CLAUDE.md HARD RULE) before being stored in
/// the SSB row, so the audit trail does not leak the raw secret.
pub(crate) fn parse_cloud_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `cloud` sub-action; expected one of: override-cap, status".to_string()
        })?
        .as_str();
    match sub {
        "override-cap" => {
            let mut amount_usd: Option<u64> = None;
            let mut confirmation: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--confirm" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--confirm`".to_string())?;
                        confirmation = Some(v.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--confirm=") => {
                        confirmation = Some(flag[10..].to_string());
                        i += 1;
                    }
                    bare if !bare.starts_with("--") => {
                        let parsed: u64 = bare
                            .parse()
                            .map_err(|e| format!("override-cap amount must be u64 USD: {e}"))?;
                        amount_usd = Some(parsed);
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `cloud override-cap` argument: {other}"
                        ));
                    }
                }
            }
            let amount_usd = amount_usd.ok_or_else(|| {
                "`cloud override-cap` requires a USD amount as the first positional argument"
                    .to_string()
            })?;
            let confirmation = confirmation.ok_or_else(|| {
                "`cloud override-cap` requires --confirm <typed-string> (typed confirmation to prevent accidental override)"
                    .to_string()
            })?;
            Ok(CliAction::Cloud {
                action: CloudAction::OverrideCap {
                    amount_usd,
                    confirmation,
                },
                output_format,
            })
        }
        "status" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `cloud status`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Cloud {
                action: CloudAction::Status,
                output_format,
            })
        }
        other => Err(format!(
            "unknown `cloud` sub-action `{other}`; expected one of: override-cap, status"
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
            "missing `safety` sub-action; expected one of: status, list-pins, verify-pins, fixture-replay, tool-filter, monotonicity"
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
        "tool-filter" => {
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
                            "unknown `safety tool-filter` argument: {other}"
                        ));
                    }
                }
            }
            let fixture = fixture.ok_or_else(|| {
                "`safety tool-filter` requires --fixture <NAME|all>; NAME resolves <NAME>.json under crates/contracts/tests/fixtures/tool_filter/, `all` replays every fixture in that dir".to_string()
            })?;
            Ok(CliAction::Safety {
                action: SafetyAction::ToolFilter { fixture },
                output_format,
            })
        }
        "monotonicity" => {
            let mut fixture: Option<String> = None;
            let mut no_env_override = false;
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
                    "--no-env-override" => {
                        no_env_override = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `safety monotonicity` argument: {other}"
                        ));
                    }
                }
            }
            let fixture = fixture.ok_or_else(|| {
                "`safety monotonicity` requires --fixture <PATH> (a JSON fixture with lang/prior_reply/current_reply)".to_string()
            })?;
            Ok(CliAction::Safety {
                action: SafetyAction::Monotonicity { fixture, no_env_override },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `safety` sub-action `{other}`; \
             expected one of: status, list-pins, verify-pins, fixture-replay, tool-filter, monotonicity"
        )),
    }
}

/// F-8 / Slice A4 — `elai self-mod apply --proposal-fixture PATH
/// [--no-env-override]` argument parser. Mirrors the `safety monotonicity`
/// flag-walking pattern. Handler at crates/cli/src/self_mod_cmd.rs. See
/// workflow/reasoning/F-8-decisions.md.
pub(crate) fn parse_self_mod_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest.first().map(String::as_str).ok_or_else(|| {
        "missing `self-mod` sub-action; expected: apply".to_string()
    })?;
    match sub {
        "apply" => {
            let mut proposal_fixture: Option<String> = None;
            let mut no_env_override = false;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--proposal-fixture" => {
                        let value = rest.get(i + 1).ok_or_else(|| {
                            "missing value for `--proposal-fixture`".to_string()
                        })?;
                        proposal_fixture = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--proposal-fixture=") => {
                        proposal_fixture =
                            Some(flag["--proposal-fixture=".len()..].to_string());
                        i += 1;
                    }
                    "--no-env-override" => {
                        no_env_override = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `self-mod apply` argument: {other}"
                        ));
                    }
                }
            }
            let proposal_fixture = proposal_fixture.ok_or_else(|| {
                "`self-mod apply` requires --proposal-fixture <PATH> \
                 (a ConfigVariant JSON fixture: { variant_id, config, active_tripwires })"
                    .to_string()
            })?;
            Ok(CliAction::SelfMod {
                action: SelfModAction::Apply {
                    proposal_fixture,
                    no_env_override,
                },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `self-mod` sub-action `{other}`; expected one of: apply"
        )),
    }
}

/// E-40 — `elai scip <query|defs|refs|callers|callees|pagerank|status|import|lsp-probe|refresh>`
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
            "missing `scip` sub-action; expected one of: query, defs, refs, callers, callees, pagerank, status, import, lsp-probe, refresh, resolve"
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
        "lsp-probe" => {
            // E-52 Tier-B — production caller for the lsp_oracle JSON-RPC
            // handshake path. Flag-walking pattern mirrors `scip query`
            // for consistency.
            let mut symbol: Option<String> = None;
            let mut language: Option<String> = None;
            let mut mode: String = "probe".to_string();
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
                    "--language" => {
                        language = Some(
                            args.get(i + 1)
                                .ok_or_else(|| "missing value for `--language`".to_string())?
                                .clone(),
                        );
                        i += 2;
                    }
                    flag if flag.starts_with("--language=") => {
                        language = Some(flag["--language=".len()..].to_string());
                        i += 1;
                    }
                    "--mode" => {
                        mode = args
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--mode`".to_string())?
                            .clone();
                        i += 2;
                    }
                    flag if flag.starts_with("--mode=") => {
                        mode = flag["--mode=".len()..].to_string();
                        i += 1;
                    }
                    other => return Err(format!("unknown `scip lsp-probe` argument: {other}")),
                }
            }
            let symbol = symbol.ok_or_else(|| {
                "`scip lsp-probe` requires --symbol <NAME>".to_string()
            })?;
            let language = language.ok_or_else(|| {
                "`scip lsp-probe` requires --language <rust|python|typescript|go>".to_string()
            })?;
            Ok(CliAction::SymbolGraph {
                action: SymbolGraphAction::LspProbe { symbol, language, mode },
                output_format,
            })
        }
        "refresh" => {
            // `elai scip refresh [--scip-dir <path>] [--project-root <path>]`
            // Mirrors the `import` flag-walking pattern for consistency.
            let mut scip_dir: String = String::new();
            let mut project_root: String = String::new();
            let mut i = 0;
            while i < args.len() {
                match args[i].as_str() {
                    "--scip-dir" => {
                        scip_dir = args
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--scip-dir`".to_string())?
                            .clone();
                        i += 2;
                    }
                    flag if flag.starts_with("--scip-dir=") => {
                        scip_dir = flag["--scip-dir=".len()..].to_string();
                        i += 1;
                    }
                    "--project-root" => {
                        project_root = args
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--project-root`".to_string())?
                            .clone();
                        i += 2;
                    }
                    flag if flag.starts_with("--project-root=") => {
                        project_root = flag["--project-root=".len()..].to_string();
                        i += 1;
                    }
                    other => return Err(format!("unknown `scip refresh` argument: {other}")),
                }
            }
            Ok(CliAction::SymbolGraph {
                action: SymbolGraphAction::Refresh { scip_dir, project_root },
                output_format,
            })
        }
        "resolve" => {
            // `elai scip resolve --symbol NAME --language SLUG
            //  [--from-file PATH] [--rules-dir DIR]`
            // Mirrors the `lsp-probe` flag-walking pattern.
            let mut symbol: Option<String> = None;
            let mut language: Option<String> = None;
            let mut from_file: String = String::new();
            let mut rules_dir: String = String::new();
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
                    "--language" => {
                        language = Some(
                            args.get(i + 1)
                                .ok_or_else(|| "missing value for `--language`".to_string())?
                                .clone(),
                        );
                        i += 2;
                    }
                    flag if flag.starts_with("--language=") => {
                        language = Some(flag["--language=".len()..].to_string());
                        i += 1;
                    }
                    "--from-file" => {
                        from_file = args
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--from-file`".to_string())?
                            .clone();
                        i += 2;
                    }
                    flag if flag.starts_with("--from-file=") => {
                        from_file = flag["--from-file=".len()..].to_string();
                        i += 1;
                    }
                    "--rules-dir" => {
                        rules_dir = args
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--rules-dir`".to_string())?
                            .clone();
                        i += 2;
                    }
                    flag if flag.starts_with("--rules-dir=") => {
                        rules_dir = flag["--rules-dir=".len()..].to_string();
                        i += 1;
                    }
                    other => return Err(format!("unknown `scip resolve` argument: {other}")),
                }
            }
            let symbol = symbol.ok_or_else(|| {
                "`scip resolve` requires --symbol <NAME>".to_string()
            })?;
            let language = language.ok_or_else(|| {
                "`scip resolve` requires --language <rust|python|typescript|go>".to_string()
            })?;
            Ok(CliAction::SymbolGraph {
                action: SymbolGraphAction::Resolve {
                    symbol,
                    language,
                    from_file,
                    rules_dir,
                },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `scip` sub-action `{other}`; \
             expected one of: query, defs, refs, callers, callees, pagerank, status, import, lsp-probe, refresh, resolve"
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

/// E-53 — parse `elai formal-verify <sub>` arguments. Two sub-actions:
///   `elai formal-verify run --target PATH --verifier KIND`
///   `elai formal-verify status`
///
/// Mirrors the C-52 dafny parser shape for consistency. KIND must be
/// one of `z3_smt` / `dafny` / `kani` / `verus`. Live Z3 / Kani /
/// Verus dispatch is RUNPOD-PENDING in the lean E-53 ship (the
/// pipeline returns `Inapplicable` with a `<verifier>_stub` reason);
/// Dafny dispatch reuses C-52 `invoke_dafny_cli` and surfaces
/// `RunpodPending` when the binary is absent on the local dev host.
/// Gemma-4 BLOCKED per memory `feedback_no_gemma4`.
pub(crate) fn parse_formal_verify_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `formal-verify` sub-action; expected one of: run, status".to_string()
        })?
        .as_str();
    match sub {
        "run" => {
            let mut target: Option<String> = None;
            let mut verifier: Option<String> = None;
            let mut demo_escape_hatch = false;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--target" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--target`".to_string())?;
                        target = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--target=") => {
                        target = Some(flag["--target=".len()..].to_string());
                        i += 1;
                    }
                    "--verifier" => {
                        let value = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--verifier`".to_string())?;
                        verifier = Some(value.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--verifier=") => {
                        verifier = Some(flag["--verifier=".len()..].to_string());
                        i += 1;
                    }
                    // H-54 W-1: seeded-fixture demo trigger.
                    "--demo-escape-hatch" => {
                        demo_escape_hatch = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!("unknown `formal-verify run` argument: {other}"));
                    }
                }
            }
            let target = target.ok_or_else(|| {
                "`formal-verify run` requires --target <PATH>".to_string()
            })?;
            let verifier = verifier.ok_or_else(|| {
                "`formal-verify run` requires --verifier <z3_smt|dafny|kani|verus>".to_string()
            })?;
            Ok(CliAction::FormalVerify {
                action: FormalVerifyAction::Run { target, verifier, demo_escape_hatch },
                output_format,
            })
        }
        "status" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `formal-verify status`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::FormalVerify {
                action: FormalVerifyAction::Status,
                output_format,
            })
        }
        other => Err(format!(
            "unknown `formal-verify` sub-action `{other}`; \
             expected one of: run, status"
        )),
    }
}

/// E-8 — parse `elai optimize <sub>` arguments. Two sub-actions:
///   * `template --target ROLE --train PATH --val PATH
///       [--reflection-model NAME] [--budget-tokens N]
///       [--include-cross-domain]`
///   * `status`
///
/// Reflection model is REJECTED with a clear error if the caller
/// passes `qwen` / `qwen2.5-coder` / `local` (per arXiv:2405.10276
/// small-LMs-as-optimizers limitation). The CLI surface enforces the
/// cloud-only refusal so an honest user cannot misconfigure the
/// expensive optimization loop.
pub(crate) fn parse_optimize_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `optimize` sub-action; expected one of: template, status".to_string()
        })?
        .as_str();
    match sub {
        "template" => {
            let mut target: Option<String> = None;
            let mut train: Option<String> = None;
            let mut val: Option<String> = None;
            let mut reflection_model: Option<String> = None;
            let mut budget_tokens: Option<u64> = None;
            let mut include_cross_domain = false;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--target" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--target`".to_string())?;
                        target = Some(v.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--target=") => {
                        target = Some(flag["--target=".len()..].to_string());
                        i += 1;
                    }
                    "--train" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--train`".to_string())?;
                        train = Some(v.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--train=") => {
                        train = Some(flag["--train=".len()..].to_string());
                        i += 1;
                    }
                    "--val" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--val`".to_string())?;
                        val = Some(v.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--val=") => {
                        val = Some(flag["--val=".len()..].to_string());
                        i += 1;
                    }
                    "--reflection-model" => {
                        let v = rest.get(i + 1).ok_or_else(|| {
                            "missing value for `--reflection-model`".to_string()
                        })?;
                        reflection_model = Some(v.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--reflection-model=") => {
                        reflection_model =
                            Some(flag["--reflection-model=".len()..].to_string());
                        i += 1;
                    }
                    "--budget-tokens" => {
                        let v = rest.get(i + 1).ok_or_else(|| {
                            "missing value for `--budget-tokens`".to_string()
                        })?;
                        budget_tokens = Some(v.parse::<u64>().map_err(|e| {
                            format!("--budget-tokens parse: {e}")
                        })?);
                        i += 2;
                    }
                    flag if flag.starts_with("--budget-tokens=") => {
                        let raw = &flag["--budget-tokens=".len()..];
                        budget_tokens = Some(
                            raw.parse::<u64>()
                                .map_err(|e| format!("--budget-tokens parse: {e}"))?,
                        );
                        i += 1;
                    }
                    "--include-cross-domain" => {
                        include_cross_domain = true;
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `optimize template` argument: {other}"
                        ));
                    }
                }
            }
            let target = target.ok_or_else(|| {
                "`optimize template` requires --target <agent_role>".to_string()
            })?;
            let train = train.ok_or_else(|| {
                "`optimize template` requires --train <PATH>".to_string()
            })?;
            let val = val
                .ok_or_else(|| "`optimize template` requires --val <PATH>".to_string())?;
            let reflection_model =
                reflection_model.unwrap_or_else(|| "sonnet-4.6".to_string());
            // arXiv:2405.10276 -- small LLMs are ineffective as the OPRO
            // optimizer. ELAI's local Qwen2.5-Coder-7B falls in that
            // regime; the CLI hard-refuses it as the reflection LM. The
            // exact match list mirrors the rejection messages from
            // workflow/reasoning/E-8-decisions.md.
            let lower = reflection_model.to_ascii_lowercase();
            let is_local = lower.contains("qwen")
                || lower.contains("local")
                || lower == "7b"
                || lower == "3b";
            if is_local {
                return Err(format!(
                    "reflection-model must be cloud (sonnet / gpt / opus class); \
                     refused `{reflection_model}` per arXiv:2405.10276 small-LMs-as-optimizers limitation"
                ));
            }
            let budget_tokens = budget_tokens.unwrap_or(200_000);
            Ok(CliAction::Optimize {
                action: OptimizeAction::Template {
                    target,
                    train,
                    val,
                    reflection_model,
                    budget_tokens,
                    include_cross_domain,
                },
                output_format,
            })
        }
        "status" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `optimize status`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::Optimize {
                action: OptimizeAction::Status,
                output_format,
            })
        }
        other => Err(format!(
            "unknown `optimize` sub-action `{other}`; \
             expected one of: template, status"
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
        "install" => Some(Ok(CliAction::Install {
            skip_models: false,
            reset_config: false,
            project_root: None,
            output_format,
        })),
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
            | "usage"
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

/// E-35 — `elai structured-output <sub>` sub-verb. Anchors the
/// host-side structured-output substrate (codeact_sandbox + cloud
/// strict-schema marker + SearchReplaceMatcher smoke + grammar cache
/// emit-test) as production callable. WIRE-DARK avoidance per
/// E40_WIRE_DARK_AVOIDANCE_VIA_CLI. Each subverb emits at least one
/// of the 10 new E-35 SSB beacons so no variant ships unwired.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StructuredOutputAction {
    /// `elai structured-output status` — print the 10 E-35 SSB action
    /// types + host-side wired primitives + RUNPOD-PENDING boundaries.
    /// GPU-free synthetic snapshot.
    Status,
    /// `elai structured-output codeact-run --python CODE [--allow TOOL]...`
    /// — invoke `codeact_sandbox::execute_code_act` against the
    /// supplied Python snippet (Runner-role-FREE). Returns
    /// SandboxAbsent when no sandbox is wired (RUNPOD-PENDING).
    /// Emits a `CodeactActionExecuted` SSB row regardless of outcome.
    CodeactRun {
        python: String,
        allow: Vec<String>,
    },
    /// `elai structured-output strict-mode --provider P --schema-id S
    /// --mode MODE` — invoke
    /// `CloudGroundingClient::mark_strict_schema_applied`. Emits a
    /// `CloudStrictSchemaApplied` SSB row when `mode != json_object`;
    /// refuses + emits `LegacyJsonObjectRefused` SSB row when
    /// `mode == json_object`.
    StrictModeMark {
        provider: String,
        schema_id: String,
        mode: String,
    },
    /// `elai structured-output grammar emit-test` — emit a synthetic
    /// `GrammarCacheLoaded` SSB row (host-side smoke). Real producer
    /// is the Python sidecar's `compile_all_schemas_at_boot`
    /// (RUNPOD-PENDING).
    GrammarEmitTest,
    /// `elai structured-output search-replace check --path PATH
    /// --buffer-text TEXT` — exercise
    /// `SearchReplaceMatcher`/`DelimiterCollisionGuard` against the
    /// supplied buffer text. Emits either
    /// `SearchReplaceTier1Hit` or `SearchReplaceCollisionFallback`
    /// SSB row depending on whether ELAI fences collide with the buffer.
    SearchReplaceCheck { path: String, buffer_text: String },
    /// `elai structured-output reject-engine --engine ENGINE
    /// --reason REASON` — emit a `StructuredEngineRejected` SSB row
    /// recording that an alternative engine (lm-format-enforcer) is
    /// being refused in favour of canonical XGrammar.
    RejectEngine { engine: String, reason: String },
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

pub(crate) fn parse_structured_output_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest
        .first()
        .ok_or_else(|| {
            "missing `structured-output` sub-action; expected one of: status, \
             codeact-run, strict-mode, grammar, search-replace, reject-engine"
                .to_string()
        })?
        .as_str();
    match sub {
        "status" => {
            if rest.len() > 1 {
                return Err(format!(
                    "unexpected argument after `structured-output status`: {}",
                    rest[1]
                ));
            }
            Ok(CliAction::StructuredOutput {
                action: StructuredOutputAction::Status,
                output_format,
            })
        }
        "codeact-run" => {
            let mut python: Option<String> = None;
            let mut allow: Vec<String> = Vec::new();
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--python" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--python`".to_string())?;
                        python = Some(v.clone());
                        i += 2;
                    }
                    "--allow" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--allow`".to_string())?;
                        allow.push(v.clone());
                        i += 2;
                    }
                    flag if flag.starts_with("--python=") => {
                        python = Some(flag["--python=".len()..].to_string());
                        i += 1;
                    }
                    flag if flag.starts_with("--allow=") => {
                        allow.push(flag["--allow=".len()..].to_string());
                        i += 1;
                    }
                    other => {
                        return Err(format!(
                            "unknown `structured-output codeact-run` arg: {other}"
                        ))
                    }
                }
            }
            let python = python.ok_or_else(|| {
                "`structured-output codeact-run` requires --python".to_string()
            })?;
            Ok(CliAction::StructuredOutput {
                action: StructuredOutputAction::CodeactRun { python, allow },
                output_format,
            })
        }
        "strict-mode" => {
            let mut provider: Option<String> = None;
            let mut schema_id: Option<String> = None;
            let mut mode: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--provider" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--provider`".to_string())?;
                        provider = Some(v.clone());
                        i += 2;
                    }
                    "--schema-id" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--schema-id`".to_string())?;
                        schema_id = Some(v.clone());
                        i += 2;
                    }
                    "--mode" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--mode`".to_string())?;
                        mode = Some(v.clone());
                        i += 2;
                    }
                    other => {
                        return Err(format!(
                            "unknown `structured-output strict-mode` arg: {other}"
                        ))
                    }
                }
            }
            let provider = provider.ok_or_else(|| {
                "`structured-output strict-mode` requires --provider".to_string()
            })?;
            let schema_id = schema_id.ok_or_else(|| {
                "`structured-output strict-mode` requires --schema-id".to_string()
            })?;
            let mode = mode.ok_or_else(|| {
                "`structured-output strict-mode` requires --mode".to_string()
            })?;
            Ok(CliAction::StructuredOutput {
                action: StructuredOutputAction::StrictModeMark {
                    provider,
                    schema_id,
                    mode,
                },
                output_format,
            })
        }
        "grammar" => {
            let sub2 = rest
                .get(1)
                .ok_or_else(|| {
                    "missing `grammar` sub-sub-action; expected: emit-test".to_string()
                })?
                .as_str();
            match sub2 {
                "emit-test" => {
                    if rest.len() > 2 {
                        return Err(format!(
                            "unexpected argument after `structured-output grammar \
                             emit-test`: {}",
                            rest[2]
                        ));
                    }
                    Ok(CliAction::StructuredOutput {
                        action: StructuredOutputAction::GrammarEmitTest,
                        output_format,
                    })
                }
                other => Err(format!(
                    "unknown `structured-output grammar` sub-action: {other}; \
                     expected: emit-test"
                )),
            }
        }
        "search-replace" => {
            let sub2 = rest
                .get(1)
                .ok_or_else(|| {
                    "missing `search-replace` sub-sub-action; expected: check".to_string()
                })?
                .as_str();
            match sub2 {
                "check" => {
                    let mut path: Option<String> = None;
                    let mut buffer_text: Option<String> = None;
                    let mut i = 2;
                    while i < rest.len() {
                        match rest[i].as_str() {
                            "--path" => {
                                let v = rest.get(i + 1).ok_or_else(|| {
                                    "missing value for `--path`".to_string()
                                })?;
                                path = Some(v.clone());
                                i += 2;
                            }
                            "--buffer-text" => {
                                let v = rest.get(i + 1).ok_or_else(|| {
                                    "missing value for `--buffer-text`".to_string()
                                })?;
                                buffer_text = Some(v.clone());
                                i += 2;
                            }
                            other => {
                                return Err(format!(
                                    "unknown `structured-output search-replace check` \
                                     arg: {other}"
                                ))
                            }
                        }
                    }
                    let path = path.ok_or_else(|| {
                        "`structured-output search-replace check` requires --path"
                            .to_string()
                    })?;
                    let buffer_text = buffer_text.ok_or_else(|| {
                        "`structured-output search-replace check` requires \
                         --buffer-text"
                            .to_string()
                    })?;
                    Ok(CliAction::StructuredOutput {
                        action: StructuredOutputAction::SearchReplaceCheck {
                            path,
                            buffer_text,
                        },
                        output_format,
                    })
                }
                other => Err(format!(
                    "unknown `structured-output search-replace` sub-action: {other}; \
                     expected: check"
                )),
            }
        }
        "reject-engine" => {
            let mut engine: Option<String> = None;
            let mut reason: Option<String> = None;
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--engine" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--engine`".to_string())?;
                        engine = Some(v.clone());
                        i += 2;
                    }
                    "--reason" => {
                        let v = rest
                            .get(i + 1)
                            .ok_or_else(|| "missing value for `--reason`".to_string())?;
                        reason = Some(v.clone());
                        i += 2;
                    }
                    other => {
                        return Err(format!(
                            "unknown `structured-output reject-engine` arg: {other}"
                        ))
                    }
                }
            }
            let engine = engine.ok_or_else(|| {
                "`structured-output reject-engine` requires --engine".to_string()
            })?;
            let reason = reason.ok_or_else(|| {
                "`structured-output reject-engine` requires --reason".to_string()
            })?;
            Ok(CliAction::StructuredOutput {
                action: StructuredOutputAction::RejectEngine { engine, reason },
                output_format,
            })
        }
        other => Err(format!(
            "unknown `structured-output` sub-action: {other}; expected one of: \
             status, codeact-run, strict-mode, grammar, search-replace, reject-engine"
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

/// H-35 — Parse `elai rewind <target> [--scope conversation|code|both]
/// [--summarize] [--session-id <id>] [--dry-run]`.
///
/// `<target>` is a positional: an integer SSB seq, "turn:<n>", or
/// "checkpoint:<label>". All flags are optional.
pub(crate) fn parse_rewind_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut target: Option<String> = None;
    let mut scope = RewindScope::Both;
    let mut summarize = false;
    let mut session_id: Option<String> = None;
    let mut dry_run = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--scope" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for `--scope` (conversation|code|both)".to_string())?;
                scope = parse_rewind_scope(value)?;
                i += 2;
            }
            flag if flag.starts_with("--scope=") => {
                scope = parse_rewind_scope(&flag["--scope=".len()..])?;
                i += 1;
            }
            "--summarize" => {
                summarize = true;
                i += 1;
            }
            "--session-id" => {
                let value = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value for `--session-id`".to_string())?;
                session_id = Some(value.clone());
                i += 2;
            }
            flag if flag.starts_with("--session-id=") => {
                session_id = Some(flag["--session-id=".len()..].to_string());
                i += 1;
            }
            "--dry-run" => {
                dry_run = true;
                i += 1;
            }
            flag if flag.starts_with("--") => {
                return Err(format!("unknown `rewind` flag: {flag}"));
            }
            positional => {
                if target.is_some() {
                    return Err(format!(
                        "unexpected extra positional argument to `rewind`: `{positional}` \
                         (target already set to `{}`)",
                        target.as_deref().unwrap_or("")
                    ));
                }
                target = Some(positional.to_string());
                i += 1;
            }
        }
    }
    let target = target.ok_or_else(|| {
        "`rewind` requires a target: an SSB seq (integer), `turn:<n>`, or `checkpoint:<label>`"
            .to_string()
    })?;
    Ok(CliAction::Rewind {
        action: RewindAction {
            target,
            scope,
            summarize,
            session_id,
            dry_run,
        },
        output_format,
    })
}

/// H-35 — Parse the `--scope` value for `elai rewind`.
fn parse_rewind_scope(raw: &str) -> Result<RewindScope, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "conversation" => Ok(RewindScope::Conversation),
        "code" => Ok(RewindScope::Code),
        "both" => Ok(RewindScope::Both),
        other => Err(format!(
            "unknown `--scope` value `{other}` (expected `conversation`, `code`, or `both`)"
        )),
    }
}

// ---------------------------------------------------------------------------
// H-41 — `elai checker-gaps` parser
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// J-28 T8 — `elai usage` parser
// ---------------------------------------------------------------------------

/// J-28 T8: parse `elai usage [--json]`.
pub(crate) fn parse_usage_subcommand_args(rest: &[String]) -> Result<CliAction, String> {
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--json" => json = true,
            other => return Err(format!("usage: unknown flag `{other}`")),
        }
        i += 1;
    }
    Ok(CliAction::Usage { json })
}

/// H-41: parse `elai checker-gaps [--limit N] [--json]`.
pub(crate) fn parse_checker_gaps_args(rest: &[String]) -> Result<CliAction, String> {
    let mut limit: Option<usize> = None;
    let mut json = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--limit" => {
                i += 1;
                let n = rest
                    .get(i)
                    .ok_or("--limit requires a value")?
                    .parse::<usize>()
                    .map_err(|e| format!("--limit: {e}"))?;
                limit = Some(n);
            }
            "--json" => {
                json = true;
            }
            other => return Err(format!("checker-gaps: unknown flag `{other}`")),
        }
        i += 1;
    }
    Ok(CliAction::CheckerGaps { limit, json })
}

// ---------------------------------------------------------------------------
// I-32 — role-ab-probe parser
// ---------------------------------------------------------------------------

/// I-32: parse `elai role-ab-probe [--seed <u64>]`.
pub(crate) fn parse_role_ab_probe_args(rest: &[String]) -> Result<CliAction, String> {
    let mut seed: u64 = 42;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--seed" => {
                i += 1;
                seed = rest
                    .get(i)
                    .ok_or("--seed requires a value")?
                    .parse::<u64>()
                    .map_err(|e| format!("--seed: {e}"))?;
            }
            other => return Err(format!("role-ab-probe: unknown flag `{other}`")),
        }
        i += 1;
    }
    Ok(CliAction::RoleAbProbe { seed })
}

// ---------------------------------------------------------------------------
// H-49 — time-travel parsers
// ---------------------------------------------------------------------------

fn parse_tt_grain(s: &str) -> Result<TimeTravelGrainArg, String> {
    match s {
        "edit" => Ok(TimeTravelGrainArg::Edit),
        "turn" => Ok(TimeTravelGrainArg::Turn),
        "babystep" | "baby_step" | "baby-step" => Ok(TimeTravelGrainArg::BabyStep),
        other => Err(format!(
            "unknown grain `{other}`; expected edit|turn|babystep"
        )),
    }
}

fn parse_tt_scope(s: &str) -> Result<TimeTravelScope, String> {
    match s {
        "code" => Ok(TimeTravelScope::Code),
        "conversation" => Ok(TimeTravelScope::Conversation),
        "both" => Ok(TimeTravelScope::Both),
        other => Err(format!(
            "unknown scope `{other}`; expected code|conversation|both"
        )),
    }
}

fn parse_tt_session_flag(
    rest: &[String],
    i: &mut usize,
    flag: &str,
) -> Result<String, String> {
    let val = rest
        .get(*i + 1)
        .ok_or_else(|| format!("missing value for `{flag}`"))?
        .clone();
    *i += 2;
    Ok(val)
}

/// H-49: parse `elai undo [--grain ...] [--scope ...] [--steps N] [--dry-run]
///         [--session-id <id>]`.
pub(crate) fn parse_timetravel_undo_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut grain = TimeTravelGrainArg::Turn;
    let mut scope = TimeTravelScope::Both;
    let mut steps: u32 = 1;
    let mut dry_run = false;
    let mut session_id: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--grain" => {
                grain = parse_tt_grain(rest.get(i + 1).ok_or("--grain needs a value")?)?;
                i += 2;
            }
            f if f.starts_with("--grain=") => {
                grain = parse_tt_grain(&f["--grain=".len()..])?;
                i += 1;
            }
            "--scope" => {
                scope = parse_tt_scope(rest.get(i + 1).ok_or("--scope needs a value")?)?;
                i += 2;
            }
            f if f.starts_with("--scope=") => {
                scope = parse_tt_scope(&f["--scope=".len()..])?;
                i += 1;
            }
            "--steps" => {
                steps = rest
                    .get(i + 1)
                    .ok_or("--steps needs a value")?
                    .parse::<u32>()
                    .map_err(|e| format!("--steps: {e}"))?;
                i += 2;
            }
            f if f.starts_with("--steps=") => {
                steps = f["--steps=".len()..]
                    .parse::<u32>()
                    .map_err(|e| format!("--steps: {e}"))?;
                i += 1;
            }
            "--dry-run" => { dry_run = true; i += 1; }
            "--session-id" => {
                session_id = Some(parse_tt_session_flag(rest, &mut i, "--session-id")?);
            }
            f if f.starts_with("--session-id=") => {
                session_id = Some(f["--session-id=".len()..].to_owned());
                i += 1;
            }
            f if f.starts_with("--") => {
                return Err(format!("unknown `undo` flag: {f}"));
            }
            _ => { i += 1; } // ignore unknown positionals
        }
    }
    Ok(CliAction::TimeTravel {
        action: TimeTravelAction::Undo { grain, scope, steps, dry_run, session_id },
        output_format,
    })
}

/// H-49: parse `elai redo [--grain ...] [--scope ...] [--dry-run] [--session-id <id>]`.
pub(crate) fn parse_timetravel_redo_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut grain = TimeTravelGrainArg::Turn;
    let mut scope = TimeTravelScope::Both;
    let mut dry_run = false;
    let mut session_id: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--grain" => {
                grain = parse_tt_grain(rest.get(i + 1).ok_or("--grain needs a value")?)?;
                i += 2;
            }
            f if f.starts_with("--grain=") => {
                grain = parse_tt_grain(&f["--grain=".len()..])?;
                i += 1;
            }
            "--scope" => {
                scope = parse_tt_scope(rest.get(i + 1).ok_or("--scope needs a value")?)?;
                i += 2;
            }
            f if f.starts_with("--scope=") => {
                scope = parse_tt_scope(&f["--scope=".len()..])?;
                i += 1;
            }
            "--dry-run" => { dry_run = true; i += 1; }
            "--session-id" => {
                session_id = Some(parse_tt_session_flag(rest, &mut i, "--session-id")?);
            }
            f if f.starts_with("--session-id=") => {
                session_id = Some(f["--session-id=".len()..].to_owned());
                i += 1;
            }
            f if f.starts_with("--") => {
                return Err(format!("unknown `redo` flag: {f}"));
            }
            _ => { i += 1; }
        }
    }
    Ok(CliAction::TimeTravel {
        action: TimeTravelAction::Redo { grain, scope, dry_run, session_id },
        output_format,
    })
}

/// H-49: parse `elai history [--format tree|list] [--session-id <id>]`.
pub(crate) fn parse_timetravel_history_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut format = "list".to_owned();
    let mut session_id: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--format" => {
                format = rest.get(i + 1).ok_or("--format needs a value")?.clone();
                i += 2;
            }
            f if f.starts_with("--format=") => {
                format = f["--format=".len()..].to_owned();
                i += 1;
            }
            "--session-id" => {
                session_id = Some(parse_tt_session_flag(rest, &mut i, "--session-id")?);
            }
            f if f.starts_with("--session-id=") => {
                session_id = Some(f["--session-id=".len()..].to_owned());
                i += 1;
            }
            f if f.starts_with("--") => {
                return Err(format!("unknown `history` flag: {f}"));
            }
            _ => { i += 1; }
        }
    }
    Ok(CliAction::TimeTravel {
        action: TimeTravelAction::History { format, session_id },
        output_format,
    })
}

/// H-49: parse `elai jump <target> [--grain ...] [--scope ...] [--dry-run]
///         [--session-id <id>]`.
pub(crate) fn parse_timetravel_jump_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut target: Option<String> = None;
    let mut grain = TimeTravelGrainArg::Turn;
    let mut scope = TimeTravelScope::Both;
    let mut dry_run = false;
    let mut session_id: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--grain" => {
                grain = parse_tt_grain(rest.get(i + 1).ok_or("--grain needs a value")?)?;
                i += 2;
            }
            f if f.starts_with("--grain=") => {
                grain = parse_tt_grain(&f["--grain=".len()..])?;
                i += 1;
            }
            "--scope" => {
                scope = parse_tt_scope(rest.get(i + 1).ok_or("--scope needs a value")?)?;
                i += 2;
            }
            f if f.starts_with("--scope=") => {
                scope = parse_tt_scope(&f["--scope=".len()..])?;
                i += 1;
            }
            "--dry-run" => { dry_run = true; i += 1; }
            "--session-id" => {
                session_id = Some(parse_tt_session_flag(rest, &mut i, "--session-id")?);
            }
            f if f.starts_with("--session-id=") => {
                session_id = Some(f["--session-id=".len()..].to_owned());
                i += 1;
            }
            f if f.starts_with("--") => {
                return Err(format!("unknown `jump` flag: {f}"));
            }
            positional => {
                if target.is_some() {
                    return Err(format!(
                        "`jump` only accepts one target argument; got extra: `{positional}`"
                    ));
                }
                target = Some(positional.to_owned());
                i += 1;
            }
        }
    }
    let target = target.ok_or_else(|| {
        "`jump` requires a target: a snap_id, `turn:<n>`, or node label".to_owned()
    })?;
    Ok(CliAction::TimeTravel {
        action: TimeTravelAction::Jump { target, grain, scope, dry_run, session_id },
        output_format,
    })
}

/// I-39 — parsed plan action from `elai plan status [--letter <L>] [--json]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlanStatusAction {
    /// Filter to a single plan letter (e.g. "B", "H"). `None` means all letters.
    pub(crate) letter: Option<String>,
    /// Override fragments directory (default: workflow/sessions/s-plan-audit-2026-06-04/).
    pub(crate) fragments_dir: Option<std::path::PathBuf>,
}

/// I-39 — parse `elai plan status [--letter <L>] [--fragments-dir <path>]`.
///
/// `--letter <L>`: filter to one plan letter. Accepts upper/lower; normalised to uppercase.
/// `--fragments-dir <path>`: override the default FINISH-fragment directory.
/// Unknown flags are a hard error (fail-closed parse).
pub(crate) fn parse_plan_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    // `elai plan` requires at least one subverb.
    let sub = rest
        .first()
        .ok_or_else(|| "`plan` requires a subcommand: `status`".to_string())?;
    match sub.as_str() {
        "status" => parse_plan_status_args(&rest[1..], output_format),
        other => Err(format!("unknown `plan` subcommand: {other}; expected `status`")),
    }
}

fn parse_plan_status_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut letter: Option<String> = None;
    let mut fragments_dir: Option<std::path::PathBuf> = None;
    let mut i = 0usize;
    while i < rest.len() {
        match rest[i].as_str() {
            "--letter" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "`--letter` requires a value (e.g. B or H)".to_string())?;
                letter = Some(v.to_uppercase());
                i += 2;
            }
            flag if flag.starts_with("--letter=") => {
                let v = &flag["--letter=".len()..];
                letter = Some(v.to_uppercase());
                i += 1;
            }
            "--fragments-dir" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "`--fragments-dir` requires a path value".to_string())?;
                fragments_dir = Some(std::path::PathBuf::from(v));
                i += 2;
            }
            flag if flag.starts_with("--fragments-dir=") => {
                let v = &flag["--fragments-dir=".len()..];
                fragments_dir = Some(std::path::PathBuf::from(v));
                i += 1;
            }
            other => {
                return Err(format!("unknown `plan status` argument: {other}"));
            }
        }
    }
    Ok(CliAction::PlanStatus {
        action: PlanStatusAction { letter, fragments_dir },
        output_format,
    })
}

/// I-41 — parse `elai critique <spec-path> [--lenses L1,...] [--risk simple|heavy] [--mock]`.
///
/// `<spec-path>` is positional and required.
/// `--lenses` accepts a comma-separated list of lens IDs (L1..L8).
/// `--risk` accepts `simple` (L1/L4/L8 trio) or `heavy` (all 8). Default: heavy.
/// `--mock` activates the inline stub runtime.
/// Unknown flags are a hard error (fail-closed parse).
pub(crate) fn parse_critique_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    use elai_orchestrator::critique::lenses::{LensId, RiskLevel};

    let mut spec_path: Option<std::path::PathBuf> = None;
    let mut lenses: Vec<LensId> = Vec::new();
    let mut risk = RiskLevel::Heavy;
    let mut mock = false;
    let mut i = 0usize;

    while i < rest.len() {
        match rest[i].as_str() {
            "--lenses" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "`--lenses` requires a comma-separated list (e.g. L1,L4,L8)".to_string())?;
                lenses = parse_lens_ids(v)?;
                i += 2;
            }
            flag if flag.starts_with("--lenses=") => {
                let v = &flag["--lenses=".len()..];
                lenses = parse_lens_ids(v)?;
                i += 1;
            }
            "--risk" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "`--risk` requires `simple` or `heavy`".to_string())?;
                risk = parse_risk_level(v)?;
                i += 2;
            }
            flag if flag.starts_with("--risk=") => {
                let v = &flag["--risk=".len()..];
                risk = parse_risk_level(v)?;
                i += 1;
            }
            "--mock" => {
                mock = true;
                i += 1;
            }
            arg if !arg.starts_with('-') && spec_path.is_none() => {
                spec_path = Some(std::path::PathBuf::from(arg));
                i += 1;
            }
            other => {
                return Err(format!("unknown `critique` argument: {other}"));
            }
        }
    }

    let spec_path = spec_path.ok_or_else(|| {
        "`critique` requires a spec file path (e.g. `elai critique path/to/spec.md`)".to_string()
    })?;

    Ok(CliAction::Critique {
        action: crate::args::CritiqueAction { spec_path, lenses, risk, mock },
        output_format,
    })
}

fn parse_lens_ids(s: &str) -> Result<Vec<elai_orchestrator::critique::lenses::LensId>, String> {
    use elai_orchestrator::critique::lenses::LensId;
    let mut ids = Vec::new();
    for part in s.split(',') {
        let id = match part.trim().to_uppercase().as_str() {
            "L1" => LensId::L1AssumptionExcavator,
            "L2" => LensId::L2ImpliedCompleteness,
            "L3" => LensId::L3AmbiguityMapper,
            "L4" => LensId::L4GapAnalyzer,
            "L5" => LensId::L5PreImplArchitect,
            "L6" => LensId::L6ApiContractValidator,
            "L7" => LensId::L7DocumentationValidator,
            "L8" => LensId::L8AnxietyReader,
            other => return Err(format!("unknown lens id `{other}`; expected L1..L8")),
        };
        ids.push(id);
    }
    Ok(ids)
}

fn parse_risk_level(s: &str) -> Result<elai_orchestrator::critique::lenses::RiskLevel, String> {
    use elai_orchestrator::critique::lenses::RiskLevel;
    match s.to_lowercase().as_str() {
        "simple" => Ok(RiskLevel::Simple),
        "heavy" => Ok(RiskLevel::Heavy),
        other => Err(format!("unknown risk level `{other}`; expected `simple` or `heavy`")),
    }
}

/// I-40 — parse `elai wrap [--output <path>] [--json] [--fixture] -- <agent-cmd...>`.
///
/// Grammar:
///   `elai wrap -- <agent-cmd> [args...]`      — subprocess mode
///   `elai wrap --fixture`                      — deterministic Tier-1 fixture
///   `elai wrap [--output <path>] -- <cmd...>` — pin .rra output path
///
/// `--` is REQUIRED before the agent command to prevent flag mis-parsing.
/// `--fixture` runs the deterministic W-1 fixture without a real subprocess.
/// Kill-switch: ELAI_WRAP_RRA_DISABLE=1 disables RRA emit (R9, default-ON).
pub(crate) fn parse_wrap_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut output_path: Option<PathBuf> = None;
    let mut fixture_mode = false;
    let mut i = 0;
    // Parse flags before `--` or `--fixture`.
    while i < rest.len() {
        match rest[i].as_str() {
            "--fixture" => {
                fixture_mode = true;
                i += 1;
            }
            "--output" => {
                let val = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing path after --output".to_string())?;
                output_path = Some(PathBuf::from(val));
                i += 2;
            }
            flag if flag.starts_with("--output=") => {
                output_path = Some(PathBuf::from(&flag[9..]));
                i += 1;
            }
            "--" => {
                i += 1;
                break;
            }
            other => {
                return Err(format!(
                    "unknown `wrap` flag `{other}`; use `--` to separate the agent command"
                ));
            }
        }
    }
    if fixture_mode {
        // Fixture mode: no agent command required.
        return Ok(CliAction::Wrap {
            agent_cmd: vec!["--fixture".to_string()],
            output_path,
            output_format,
        });
    }
    // The remaining args after `--` are the agent command.
    let agent_cmd: Vec<String> = rest[i..].to_vec();
    if agent_cmd.is_empty() {
        return Err(
            "`elai wrap` requires an agent command after `--`; \
             use `--fixture` for the deterministic fixture"
                .to_string(),
        );
    }
    Ok(CliAction::Wrap {
        agent_cmd,
        output_path,
        output_format,
    })
}

/// K-6 — parse `elai export <sub-action> ...`.
///
/// Currently the only sub-action is `compliance`.
pub(crate) fn parse_export_subcommand_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let sub = rest.first().map(String::as_str).ok_or_else(|| {
        "missing `export` sub-action; expected: compliance".to_string()
    })?;
    match sub {
        "compliance" => parse_export_compliance_args(&rest[1..], output_format),
        other => Err(format!(
            "unknown `export` sub-action `{other}`; expected: compliance"
        )),
    }
}

/// K-6 — parse `elai export compliance [--session <id>] [--out|--output <path>] [--zero-retention]`.
fn parse_export_compliance_args(
    rest: &[String],
    output_format: CliOutputFormat,
) -> Result<CliAction, String> {
    let mut session_id: Option<String> = None;
    let mut output_path: Option<PathBuf> = None;
    let mut zero_retention = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--session" => {
                let val = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value after --session".to_string())?;
                session_id = Some(val.clone());
                i += 2;
            }
            flag if flag.starts_with("--session=") => {
                session_id = Some(flag[10..].to_string());
                i += 1;
            }
            "--out" | "--output" => {
                let val = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing path after --out/--output".to_string())?;
                output_path = Some(PathBuf::from(val));
                i += 2;
            }
            flag if flag.starts_with("--out=") => {
                output_path = Some(PathBuf::from(&flag[6..]));
                i += 1;
            }
            flag if flag.starts_with("--output=") => {
                output_path = Some(PathBuf::from(&flag[9..]));
                i += 1;
            }
            "--zero-retention" => {
                zero_retention = true;
                i += 1;
            }
            other => {
                return Err(format!(
                    "unknown `export compliance` flag `{other}`; expected --session, --out, --zero-retention"
                ));
            }
        }
    }
    let session_id = session_id.ok_or_else(|| {
        "`elai export compliance` requires `--session <id>`".to_string()
    })?;
    let output_path = output_path.ok_or_else(|| {
        "`elai export compliance` requires `--out <path>` (or `--output <path>`)".to_string()
    })?;
    Ok(CliAction::Compliance {
        session_id,
        output_path,
        zero_retention,
        output_format,
    })
}

/// J-26 S5 (slice S4) — parse `elai eval-corpus-ingest --pack <dir>
/// [--root <corpus-root>]`. `--pack` is required; `--root` defaults to
/// `{cwd}/.elai_cc` inside the handler.
fn parse_eval_corpus_ingest_args(rest: &[String]) -> Result<CliAction, String> {
    let mut pack: Option<PathBuf> = None;
    let mut root: Option<PathBuf> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--pack" => {
                let val = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value after --pack".to_string())?;
                pack = Some(PathBuf::from(val));
                i += 2;
            }
            flag if flag.starts_with("--pack=") => {
                pack = Some(PathBuf::from(&flag[7..]));
                i += 1;
            }
            "--root" => {
                let val = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value after --root".to_string())?;
                root = Some(PathBuf::from(val));
                i += 2;
            }
            flag if flag.starts_with("--root=") => {
                root = Some(PathBuf::from(&flag[7..]));
                i += 1;
            }
            other => {
                return Err(format!(
                    "unknown `eval-corpus-ingest` flag `{other}`; expected --pack, --root"
                ));
            }
        }
    }
    let pack = pack.ok_or_else(|| {
        "`elai eval-corpus-ingest` requires `--pack <dir>`".to_string()
    })?;
    Ok(CliAction::EvalCorpusIngest { pack, root })
}
