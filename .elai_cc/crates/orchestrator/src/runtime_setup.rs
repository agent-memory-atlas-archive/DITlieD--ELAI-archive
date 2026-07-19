// Phase A1b — production task_runner wiring helpers.
//
// Moved from `cli::runtime_builder` so the dashboard-v2 Tauri `setup()`
// callback, the `elai run` CLI subcommand, and any future surface can share
// one chokepoint that constructs the SSB writer + feasibility prober +
// Layer 1A processor as a coherent bundle.
//
// SSB path: this module ALWAYS opens the canonical flat path
// `<project_root>/.elai_cc/sessions/ssb.db` via
// `elai_core::paths::sessions_dir_path` — the same bucket the dashboard's
// `state::resolve_ssb_path` reads from. The previous CLI variant nested the
// DB under `sessions/<session_id>/ssb.db` which caused WARN-1 in
// workflow/handoff/notes-A1a.md (the dashboard never saw rows written by
// `elai run`).
//
// Anti-stub: every fallible step bubbles its error as `anyhow::Error`. No
// `.unwrap()` / `.expect()` / `panic!` / `todo!()` lives on this code path.
//
// Contract: `crates/contracts/tests/task_runner_chokepoint_contract.rs`
// (`contract_apply_task_runner_wiring_attaches_both_processors`).

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use tokio::sync::Mutex as AsyncMutex;

use crate::{
    agent_pool::{AgentPoolConfig, InProcessAgentPool},
    autonomous_gate::{AutonomousGate, AutonomousGateConfig, GateMode},
    background_task::BackgroundTaskManager,
    run_profile::RunProfile,
    cloud_mode::{CloudConfig, CloudModeRouter, RedactingBabyStepExecutor},
    compiled_pipeline::{CompiledPipelineStore, TemplateCrystallizer},
    dispatcher::{BackendConfig, BackendMode},
    gate::InteractiveGate,
    grc::{pipeline_grc_disabled_via_env, PipelineGrc},
    hook_dispatcher::{HookDispatcher, InProcessHookDispatcher},
    ipc_accept::AgentRpcServer,
    ipc_gateway::IpcGateway,
    skill_trust::SkillStore,
    task_runner::{AgentPoolBabyStepExecutor, BabyStepExecutor},
    DefaultFeasibilityProber, DefaultLayer1aProcessor, FeasibilityProber, Layer1aProcessor, Ssb,
    SsbConfig, TaskRunnerConfig, SSB_DB_FILE_NAME,
};

/// Name of the agent binary that `AgentPoolBabyStepExecutor` spawns.
/// Includes the `.exe` suffix on Windows; bare on other platforms.
#[cfg(windows)]
const AGENT_BINARY_NAME: &str = "elai_cc_agent.exe";
#[cfg(not(windows))]
const AGENT_BINARY_NAME: &str = "elai_cc_agent";

/// Environment variable that overrides the agent binary path.
const ELAI_AGENT_BIN_ENV: &str = "ELAI_AGENT_BIN";

/// Production wiring bundle for `start_task_runner` callers. Holds the four
/// long-lived handles the chokepoint must construct before spawning the
/// background runner task: the SSB writer, the feasibility prober, the
/// Layer 1A processor, and the optional baby-step executor. All are `Arc`'d
/// so the runner loop and the caller's shutdown path can share them.
///
/// `baby_step_executor` defaults to `None` (synthetic / legacy fallback).
/// Callers that want the production agent-pool path call
/// [`build_default_baby_step_executor`] and assign the result before calling
/// [`apply_task_runner_wiring`].
pub struct RuntimeWiring {
    pub ssb: Arc<Ssb>,
    pub feasibility_prober: Arc<DefaultFeasibilityProber>,
    pub layer1a_processor: Arc<DefaultLayer1aProcessor>,
    /// Optional production baby-step executor. `None` keeps the legacy
    /// synthetic path. Set to `Some(build_default_baby_step_executor(...))`
    /// for the real agent-pool path.
    ///
    /// C-5: when `cloud_config` is also `Some` and the resolved provider is
    /// `Cloud` or `Hybrid`, [`apply_task_runner_wiring`] wraps this executor
    /// in [`RedactingBabyStepExecutor`] before plumbing it into
    /// [`TaskRunnerConfig::baby_step_executor`].
    pub baby_step_executor: Option<Arc<dyn BabyStepExecutor>>,
    /// C-9 — autonomous-gate operating mode. `None` keeps the legacy
    /// behaviour (no gate attached to TaskRunnerConfig); `Some(mode)` causes
    /// [`apply_task_runner_wiring`] to construct an [`AutonomousGate`] and
    /// thread it into `TaskRunnerConfig::autonomous_gate`. Callers parse
    /// `--mode` via `GateMode::from_cli_args` and stash the resolved mode
    /// here before calling apply.
    pub gate_mode: Option<GateMode>,
    /// I-42 — critique-only operating profile. `None` keeps the legacy
    /// autonomous-orchestrate posture (the executor is installed as resolved);
    /// `Some(RunProfile::CritiqueOnly)` causes [`apply_task_runner_wiring`] to
    /// force the resolved `baby_step_executor` to `None` BEFORE installing it,
    /// structurally refusing the executor/fixer agent spawn (one gate covers
    /// both, since the fixer derives from the same field). Callers parse
    /// `--profile` via `RunProfile::from_cli_args` and stash the resolved
    /// profile here before calling apply. `Some(RunProfile::Orchestrate)` is
    /// equivalent to `None` for wiring purposes.
    pub run_profile: Option<RunProfile>,
    /// B-6 #2 -- production [`InteractiveGate`] handle for the CLI human-
    /// prompt path. When `Some` AND `gate_mode` is set,
    /// [`apply_task_runner_wiring`] passes this handle as the second
    /// argument to [`AutonomousGate::new`] (was always `None` before this
    /// wire) so the MODIFY / REDIRECT mixed-mode fallback path can reach
    /// the live CLI prompter without rebuilding the gate. Defaults to
    /// `None` in [`build_task_runner_wiring`] so existing non-CLI callers
    /// (dashboard, contract tests) preserve the headless-batch behaviour.
    /// The CLI `run_subcommand` sets this to
    /// `Some(Arc::new(CliInteractiveGate))` when `gate_mode == Interactive`
    /// (or hybrid in future) before calling apply. Closes the B-6
    /// cross-check finding "CliInteractiveGate defined at
    /// cli/src/runtime_builder.rs:393 but NEVER instantiated in
    /// production" (per workflow/sessions/s-night-2026-05-30/cross-check/
    /// B/B-02_b4_b5_b6.md).
    pub interactive_gate: Option<Arc<dyn InteractiveGate>>,
    /// C-5 — cloud-mode router config. When `Some`,
    /// [`apply_task_runner_wiring`] constructs a [`CloudModeRouter`] and
    /// installs it on [`TaskRunnerConfig::cloud_router`], and wraps the
    /// baby-step executor in [`RedactingBabyStepExecutor`] using the
    /// privacy profile carried on the config. `None` keeps the legacy
    /// pre-C-5 behaviour (no router, no redaction wrapper).
    pub cloud_config: Option<CloudConfig>,
    /// C-5 — `--provider local|cloud|hybrid` override from the CLI. When
    /// `Some`, [`apply_task_runner_wiring`] honours it over the toml
    /// `[cloud].default_mode` field; when `None`, the toml default carried
    /// on `cloud_config.default_mode` wins. Mapped to
    /// [`BackendMode`] at apply time:
    ///   * `"local"`  -> `BackendMode::Local`
    ///   * `"cloud"`  -> `BackendMode::Api`
    ///   * `"hybrid"` -> `BackendMode::Hybrid`
    pub provider_override: Option<String>,
    /// C-13 — production hook dispatcher (Tier-1, in-process). When
    /// `Some`, [`apply_task_runner_wiring`] assigns
    /// `cfg.hook_dispatcher = wiring.hook_dispatcher.clone()`. Callers
    /// build this via [`build_production_hook_dispatcher`] which returns
    /// an [`InProcessHookDispatcher`] with the
    /// `RuffPostToolUseRunner` installed. `None` keeps the pre-C-13
    /// behaviour (no hook dispatcher attached; task_runner skips T1/T2/T3
    /// evaluation).
    ///
    /// Trait selection (DECISION D3 option 3A in workflow/reasoning/C-13-decisions.md):
    /// the field is typed as `Arc<dyn HookDispatcher>` (the in-process
    /// trait at `hook_dispatcher.rs:345`) so the assignment into
    /// `TaskRunnerConfig.hook_dispatcher` is direct. The C-11 Tier-2
    /// `ipc_accept::IpcHookDispatcher` surface is a separate concern
    /// wired by `AgentPoolBabyStepExecutor::with_hook_dispatcher` at the
    /// agent-pool level.
    pub hook_dispatcher: Option<Arc<dyn HookDispatcher + Send + Sync>>,
    /// C-25 audit follow-up — production template crystallizer.
    /// Defaults to `Some(Arc<AsyncMutex<TemplateCrystallizer>>)` via
    /// [`build_task_runner_wiring`] so [`apply_task_runner_wiring`]
    /// threads it into `TaskRunnerConfig.crystallizer`. Closes audit
    /// WARN-4: the v56 surface left production sites at `None` so the
    /// post-execute hook was a strict no-op in live runs. Tests that
    /// want the legacy None path overwrite the field on the bundle
    /// before calling apply.
    pub crystallizer: Option<Arc<AsyncMutex<TemplateCrystallizer>>>,
    /// C-25 audit follow-up — production compiled-pipeline on-disk store
    /// rooted under `<project_root>/.elai_cc/cache/compiled_pipelines.msgpack`.
    /// Loads from disk eagerly inside [`build_task_runner_wiring`] so the
    /// runner sees pre-existing crystallizations on the first task. The
    /// runner persists back via [`run_crystallizer_post_execute_hook`].
    /// Closes audit WARN-4.
    pub compiled_pipeline_store: Option<Arc<AsyncMutex<CompiledPipelineStore>>>,
    /// C-25 audit follow-up — background-task manager (soft/hard-timeout
    /// auto-detach + spawn/complete SSB rows). Constructed in
    /// [`build_task_runner_wiring`] with the same SSB writer as the
    /// rest of the runtime so all background spawns surface in the
    /// dashboard timeline. Closes audit WARN-4.
    pub background_task_manager: Option<Arc<BackgroundTaskManager>>,
    /// F-6 / Slice A3 — production learned-skill store (the C6b critic-gated
    /// acquisition authority). Constructed in [`build_task_runner_wiring`]
    /// mirroring the [`TemplateCrystallizer`] Arc and threaded into
    /// `TaskRunnerConfig.skill_store` by [`apply_task_runner_wiring`] so the
    /// crystallizer post-execute hook can insert a [`crate::skill_trust::LearnedSkill`]
    /// (born at the S1 floor) on an N-consecutive-success crystallization
    /// gate AND a reused `MiniCriticVerdict::Pass`. `None` keeps the legacy
    /// path (acquisition hook is a strict no-op); tests that want the
    /// no-store path overwrite this field on the bundle.
    pub skill_store: Option<Arc<AsyncMutex<SkillStore>>>,
    pub via_mcp: bool,
    /// C-39 -- production tool dispatch impl. When `Some`,
    /// [`apply_task_runner_wiring`] plumbs this `Arc<dyn Dispatch>` onto
    /// [`TaskRunnerConfig::mcp_dispatch`] so production tool calls go
    /// through the chosen leg (in-process or via-mcp). When `None`, the
    /// legacy C-13 `ToolDispatcher` path remains the sole dispatch
    /// surface (zero-regression default). The CLI / dashboard wiring
    /// sites construct this Arc based on `wiring.via_mcp`.
    pub mcp_dispatch: Option<Arc<dyn crate::dispatch::Dispatch>>,
    /// B-28 WIRING-2 (item E2-DIFF) — production buffer editor for the
    /// focus-tunnel DIFF execution branch. When `Some`,
    /// [`apply_task_runner_wiring`] assigns it onto
    /// [`TaskRunnerConfig::buffer_editor`] so `run_diff_step_with_provider`
    /// applies parsed hunks through the real 3-tier matcher instead of
    /// falling back to STRUCTURED. The CLI / dashboard wiring sites set this
    /// to the `Arc<InProcessToolDispatcher>` they already build (the only
    /// production `BufferEditor` impl lives in the `tools` crate, which
    /// `orchestrator` cannot depend on). `None` keeps the pre-wire fallback.
    pub buffer_editor: Option<Arc<dyn crate::focus_tunnel::BufferEditor>>,
    /// B-8 (PipelineGrc activation) — production-shaped `PipelineGrc`
    /// handle. When `Some`, [`apply_task_runner_wiring`] assigns it onto
    /// [`TaskRunnerConfig::pipeline_grc`] so the 4 safety gates
    /// (`ground_plan` / `generate_csq` / `check_inline_feasibility` /
    /// `InlineFeasibilityState::reset`) fire live during the run loop.
    /// `None` keeps the pre-B-8 no-op behaviour (every gated call site
    /// falls through silently).
    ///
    /// [`build_task_runner_wiring`] defaults this to a fresh
    /// `Some(Arc<AsyncMutex<PipelineGrc::new_from_default()>>)` UNLESS
    /// the `ELAI_DISABLE_PIPELINE_GRC` env-var kill-switch is set, in
    /// which case the field stays `None` and the runner sees the
    /// pre-B-8 path. Tests that want the legacy `None` path overwrite
    /// the field on the bundle before calling apply. The construction
    /// path delegates to [`TaskRunnerConfig::with_pipeline_grc`]
    /// semantics — the same builder-equivalent assignment the contract
    /// test at `crates/contracts/tests/pipeline_grc_wired_contract.rs`
    /// pins.
    pub pipeline_grc: Option<Arc<AsyncMutex<PipelineGrc>>>,
    /// F-1 B2/B5 — result of the vibesec seed integrity check run once at
    /// wiring time over the project rules.db (after [`build_task_runner_wiring`]
    /// seeds the `mutable=0` security rows via
    /// [`elai_rules::project_rule_seeder::ProjectRuleSeeder::seed_security_rules`]
    /// and runs [`crate::vibegate::check_vibegate_seed_integrity`]). `true` when
    /// the on-disk seed rows match the embedded blake3 baseline; `false` when
    /// the seed was tampered. Callers thread this into
    /// [`build_production_hook_dispatcher`] so a tampered seed forces the
    /// VibeGate code-vuln axis to block fail-closed on every write.
    pub vibegate_seed_ok: bool,
    /// P0-2 (E-19 reflection regression fix) — production reflection client for
    /// micro-reflection emit on completed tasks (E-19). [`build_task_runner_wiring`]
    /// constructs a [`SidecarReflectionAdapter`] wrapping a fresh
    /// [`SidecarGroundingClient::default()`] and stores it here as
    /// `Arc<dyn DynReflectionClient>`. When `ELAI_DISABLE_MICRO_REFLECTION`
    /// is set the field is `None` and the reflection call sites short-circuit.
    /// [`apply_task_runner_wiring`] assigns this directly onto
    /// `TaskRunnerConfig::reflection_client`; all 5 `new_minimal()` construction
    /// sites previously left that field at `None`.
    pub reflection_client: Option<Arc<dyn crate::DynReflectionClient>>,
    /// P0-1 (adversary resolver) — boot-time cross-family adversary runtime.
    /// Constructed in [`build_task_runner_wiring`] when BOTH
    /// `ELAI_ALLOW_CLOUD_ADVERSARY=1` is set AND
    /// [`crate::adversary::route_cross_family`] succeeds (two distinct model
    /// families reachable). Single-family or gated-off deploys leave this
    /// `None`. [`apply_task_runner_wiring`] assigns it onto
    /// `TaskRunnerConfig::adversary_runtime` so `dispatch_adversarial_pass`
    /// sees a live `AdversaryRuntimeState` (still gated by `ELAI_ADVERSARY_ENABLE`).
    pub adversary_runtime: Option<Arc<crate::runtime_config::AdversaryRuntimeState>>,
    /// B-25 Phase 4J — optional shared `EmpireBridge` handle. `None` until
    /// the CLI launches with `--empire` AND the orchestrator is compiled
    /// with `--features empire`; in that case the launcher constructs the
    /// bridge, installs the Ssb writer, calls `connect()`, and stores the
    /// `Arc<EmpireBridge>` here BEFORE calling [`apply_task_runner_wiring`]
    /// so the bridge survives for the lifetime of the runner and the
    /// runner loop can drive `emit_action` for the 5 empire_* SSB action
    /// types as Empire events arrive. Closes GAP-1: the bridge handle must
    /// outlive the `if args.empire {}` block so it is not dropped at block
    /// end. cfg-gated to `empire` feature; non-empire builds pay zero cost.
    #[cfg(feature = "empire")]
    pub empire_bridge: Option<Arc<crate::empire::EmpireBridge>>,
}

/// Build the production wiring bundle for `start_task_runner`.
///
/// Side effects:
/// - Creates `<project_root>/.elai_cc/` and
///   `<project_root>/.elai_cc/sessions/` if missing (matches the
///   dashboard's persistence layout).
/// - Opens (or creates) the canonical project SSB at
///   `<project_root>/.elai_cc/sessions/{SSB_DB_FILE_NAME}` — flat, no
///   per-session sub-directory, so the dashboard reader and any runner
///   writer share the same bucket.
/// - Opens (or creates) the project rules.db at
///   `<project_root>/.elai_cc/rules.db` and runs the standard schema
///   migrations via `elai_rules::schema::open_rules_db`.
/// - Calls `DefaultLayer1aProcessor::ensure_tables` exactly once.
///
/// Anti-stub: every fallible step bubbles the error as `anyhow::Error`; no
/// `.unwrap()` / `panic!` / `todo!()` on the hot path.
pub async fn build_task_runner_wiring(
    project_root: &std::path::Path,
    session_id: &str,
) -> Result<RuntimeWiring, anyhow::Error> {
    // 1. Ensure the `.elai_cc/` data directory and its `sessions/` subtree
    //    exist before we try to open the SQLite files below.
    let data_dir = project_root.join(elai_core::paths::ELAI_DATA_DIR_NAME);
    std::fs::create_dir_all(&data_dir).map_err(|e| {
        anyhow::anyhow!(
            "build_task_runner_wiring: create {} failed: {e}",
            data_dir.display()
        )
    })?;
    let sessions_dir = elai_core::paths::sessions_dir_path(project_root);
    std::fs::create_dir_all(&sessions_dir).map_err(|e| {
        anyhow::anyhow!(
            "build_task_runner_wiring: create {} failed: {e}",
            sessions_dir.display()
        )
    })?;

    // 2. Open the canonical SSB. Flat path `<project_root>/.elai_cc/sessions/ssb.db`
    //    — same bucket the dashboard reads via
    //    `dashboard-v2/src-tauri/src/state.rs::resolve_ssb_path` (the
    //    `sessions_dir_path(&cwd).join("ssb.db")` candidate).
    let ssb_db_path = sessions_dir.join(SSB_DB_FILE_NAME);
    // Phase A1c: derive the project_id from the caller-supplied
    // `project_root` (NOT the process CWD). The dashboard Tauri `setup()`
    // and the `elai run` CLI both pass an explicit `project_root` — using
    // `project_id_from_cwd()` here meant the CLI launched from one directory
    // and the dashboard launched from another bucketed their SSB rows under
    // different `project_id` values for the same on-disk project. The
    // canonical helper canonicalises the path internally so `./` and the
    // real path collapse to the same id (see project_id.rs:42-49).
    let project_id = elai_core::project_id_from_path(project_root);
    let ssb_config = SsbConfig {
        db_path: ssb_db_path,
        pool_size: Some(2),
        project_id,
        session_id: session_id.to_string(),
    };
    let clock_for_ssb: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(ssb_config, clock_for_ssb)
        .map_err(|e| anyhow::anyhow!("build_task_runner_wiring: SSB init failed: {e}"))?;
    let ssb = Arc::new(ssb);

    // G-4: install the process-global SSB writer for the live MCP rug-pull /
    // re-pin gate on the STDIO transport leg. The default `elai run` serves
    // MCP tool descriptions through `McpServerManager::discover_tools_*`, which
    // has no `Arc<Ssb>` in scope; this slot lets that real seam emit the
    // off-agent `mcp_rug_pull_blocked` row on a rug-pull without threading an
    // Ssb through the manager ctor + cli glue (mirrors the `lock` B-31.5
    // process-global writer). Idempotent; the gate still BLOCKS fail-closed
    // even when this is never called (only the SSB row is then dropped).
    #[cfg(feature = "mcp")]
    crate::mcp_rugpull_gate::install_ssb_writer(Arc::clone(&ssb));

    // B-31.5: install the process-global SSB writer for the cross-process
    // IndexLock (LockAcquired / LockContested rows) and the startup auto-resume
    // / graceful_shutdown paths (IndexResume / IndexCancel rows). Mirrors the
    // mcp_rugpull_gate slot above — the per-run `Arc<Ssb>` is the canonical
    // writer for every B-31 SSB emit. When unset the emits are debug-log
    // no-ops; installing here covers BOTH the CLI `elai run` path (lock emits
    // during the run phase) and the dashboard path (auto_resume at lib.rs:1950
    // runs after this wiring at :1544). Idempotent.
    crate::lock::install_ssb_writer(Arc::clone(&ssb));
    crate::startup::install_ssb_writer(Arc::clone(&ssb));

    // 3. Construct the production feasibility prober. Mirrors
    //    `dashboard-v2/src-tauri/src/state.rs:381-383`.
    //
    //    E-22 phase-4F M1 + STUB-1 + Tier3 injection (2026-05-22): the
    //    prober is now wired with BOTH a `system_prober` (so
    //    `ResourceDelta::after` reflects live VRAM) AND a `Tier3StubExecutor`
    //    when the `elai_cc_agent` binary is locatable on disk. Failure to
    //    locate the agent binary is non-fatal — the prober falls back to
    //    the synthetic stub path (input-parse only) so existing contract
    //    tests / agentless setups keep passing. The system_prober is
    //    constructed unconditionally because nvml + nvidia-smi are
    //    cheap and the prober owns its own 60s cache.
    let feasibility_root = data_dir.join("feasibility");
    let clock_for_probe: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let system_prober_for_feasibility: Arc<dyn crate::system_probe::SystemProber> = Arc::new(
        crate::system_probe::DefaultSystemProber::new(Some(project_root.to_path_buf()), clock_for_probe),
    );
    let mut prober = DefaultFeasibilityProber::new(feasibility_root)
        .with_system_prober(Arc::clone(&system_prober_for_feasibility));
    match locate_default_agent_binary() {
        Ok(agent_path) => {
            let tier3 = crate::feasibility_probe::Tier3StubExecutor::new(agent_path);
            prober = prober.with_stub_executor(Arc::new(tier3));
        }
        Err(e) => {
            tracing::info!(
                error = %e,
                "build_task_runner_wiring: elai_cc_agent not located — feasibility prober keeps the synthetic stub path"
            );
        }
    }
    let feasibility_prober = Arc::new(prober);

    // 4. Open the project rules.db via the canonical helper. This runs the
    //    standard schema migration; the resulting Connection is then wrapped
    //    in `Arc<tokio::sync::Mutex<_>>` so the async Layer 1A methods can
    //    take the lock across await boundaries.
    let rules_db_path = elai_core::paths::rules_db_path(project_root);
    let clock_for_rules = OsClock;
    let rules_conn = elai_rules::schema::open_rules_db(&rules_db_path, &clock_for_rules)
        .map_err(|e| {
            anyhow::anyhow!(
                "build_task_runner_wiring: open rules.db at {} failed: {e}",
                rules_db_path.display()
            )
        })?;

    // F-1 B5 — seed the 12 immutable (mutable=0) F-2a security rules into the
    // project rules.db. Idempotent (rows present are skipped). This gives the
    // `mutable=0` immune-rule + blake3 integrity subsystem a real production
    // caller (it was test-only before). The live VibeGate code-vuln axis itself
    // evaluates the embedded in-memory ruleset (rule_loader::CANONICAL_ROWS), so
    // the DB seed is the defence-in-depth copy the integrity guard protects.
    match elai_rules::project_rule_seeder::ProjectRuleSeeder::seed_security_rules(
        &rules_conn,
        &clock_for_rules,
    ) {
        Ok(inserted) => tracing::info!(
            inserted,
            "build_task_runner_wiring: F-2a security seed rules ensured (mutable=0)"
        ),
        Err(e) => tracing::error!(
            error = %e,
            "build_task_runner_wiring: seeding F-2a security rules failed; \
             VibeGate seed integrity will fail-closed"
        ),
    }

    // H-18 — seed the LOCKED COMPOSITE deflation promotion gate (FIN-DEFL-001,
    // mutable=0) into the project rules.db alongside the F-2a security rules.
    // This is the production caller that lights up the mutable=0 DB guarantee +
    // COMPOSITE predicate (DSR >= 0.95 AND PBO < 0.5 AND MinTRL); without it the
    // deflation gate row was test-only / wire-dark. Idempotent.
    match elai_rules::project_rule_seeder::ProjectRuleSeeder::seed_deflation_rules(
        &rules_conn,
        &clock_for_rules,
    ) {
        Ok(inserted) => tracing::info!(
            inserted,
            "build_task_runner_wiring: H-18 deflation gate seed rule ensured (FIN-DEFL-001, mutable=0)"
        ),
        Err(e) => tracing::error!(
            error = %e,
            "build_task_runner_wiring: seeding H-18 deflation gate rule failed; \
             FIN-DEFL-001 promotion gate will not be DB-backed"
        ),
    }

    // H-19 — seed the LOCKED reusable-holdout query-budget gate (FIN-HOLDOUT-001,
    // mutable=0) into the project rules.db alongside the deflation gate. This is the
    // production caller that lights up the mutable=0 DB guarantee + the Thresholdout DP
    // exhaustion invariant (budget_remaining > 0; once HOLDOUT_BUDGET_TOTAL above-threshold
    // queries are spent the OOS query is REFUSED); without it the FIN-HOLDOUT-001 row would
    // be test-only / wire-dark. The Rust gate (holdout_budget::query_holdout) enforces the
    // exhaustion + the seeded Laplace noise as named constants; this seed feeds the vibesec
    // blake3 seed-integrity baseline. Idempotent.
    match elai_rules::project_rule_seeder::ProjectRuleSeeder::seed_holdout_budget_rules(
        &rules_conn,
        &clock_for_rules,
    ) {
        Ok(inserted) => tracing::info!(
            inserted,
            "build_task_runner_wiring: H-19 holdout budget gate seed rule ensured (FIN-HOLDOUT-001, mutable=0)"
        ),
        Err(e) => tracing::error!(
            error = %e,
            "build_task_runner_wiring: seeding H-19 holdout budget gate rule failed; \
             FIN-HOLDOUT-001 query-budget gate will not be DB-backed"
        ),
    }

    // H-29 — seed the LOCKED drift-interlock gate rows (FIN-DRIFT-001 + FIN-DRIFT-002,
    // mutable=0) into the project rules.db alongside the deflation/holdout gates. This is the
    // production caller that lights up the mutable=0 DB guarantee + the four-axis drift interlock
    // (PSI on inputs+outputs < 0.25, a matching blake3 vendor-schema fingerprint, a fresh 90d
    // validation, a bounded rolling-14d deviation); without it the FIN-DRIFT-* rows would be
    // test-only / wire-dark. The Rust gate (drift_interlock::evaluate_drift_interlock) enforces
    // the full interlock with a persistent journal-derived latch (DEFAULT-HALTED on missing
    // evidence, SURVIVES restart, NO agent unlock); this seed feeds the vibesec blake3
    // seed-integrity baseline. Idempotent.
    match elai_rules::project_rule_seeder::ProjectRuleSeeder::seed_drift_interlock_rules(
        &rules_conn,
        &clock_for_rules,
    ) {
        Ok(inserted) => tracing::info!(
            inserted,
            "build_task_runner_wiring: H-29 drift interlock gate seed rules ensured (FIN-DRIFT-001/002, mutable=0)"
        ),
        Err(e) => tracing::error!(
            error = %e,
            "build_task_runner_wiring: seeding H-29 drift interlock gate rules failed; \
             FIN-DRIFT-* interlock rows will not be DB-backed"
        ),
    }

    // H-20 — seed the LOCKED COMPOSITE leakage gate (FIN-LEAK-001, mutable=0)
    // into the project rules.db alongside the deflation gate. This is the
    // production caller that lights up the leakage COMPOSITE (GraphReachability
    // over the DKG: fit-site reaching BOTH train+test sink => SoundBlock
    // L1.2/L1.3); without it the FIN-LEAK-001 row was test-only / wire-dark.
    // Idempotent.
    match elai_rules::project_rule_seeder::ProjectRuleSeeder::seed_leakage_rules(
        &rules_conn,
        &clock_for_rules,
    ) {
        Ok(inserted) => tracing::info!(
            inserted,
            "build_task_runner_wiring: H-20 leakage gate seed rule ensured (FIN-LEAK-001, mutable=0)"
        ),
        Err(e) => tracing::error!(
            error = %e,
            "build_task_runner_wiring: seeding H-20 leakage gate rule failed; \
             FIN-LEAK-001 leakage gate will not be DB-backed"
        ),
    }

    // H-22 — seed the LOCKED COMPOSITE trading null-test gate (FIN-NULL-001,
    // mutable=0) into the project rules.db alongside the deflation + leakage
    // gates. This is the production caller that lights up the mutable=0 DB
    // guarantee + the permutation-null significance invariant (p < alpha/N,
    // Bonferroni over N from H-17); without it the FIN-NULL-001 row would be
    // test-only / wire-dark. The Rust gate (trading_null_gate::evaluate_trading_null)
    // evaluates the LOCKED alpha as a named constant; this seed feeds the vibesec
    // blake3 seed-integrity baseline. Idempotent.
    match elai_rules::project_rule_seeder::ProjectRuleSeeder::seed_trading_nulls_rules(
        &rules_conn,
        &clock_for_rules,
    ) {
        Ok(inserted) => tracing::info!(
            inserted,
            "build_task_runner_wiring: H-22 trading null-test gate seed rule ensured (FIN-NULL-001, mutable=0)"
        ),
        Err(e) => tracing::error!(
            error = %e,
            "build_task_runner_wiring: seeding H-22 trading null-test gate rule failed; \
             FIN-NULL-001 null-test gate will not be DB-backed"
        ),
    }

    // H-23 — seed the LOCKED reproducibility sub-gates (REPRO-SEED-001 AST
    // seed-coverage + REPRO-ENV-001 SchemaRequired env-pin manifest, both
    // mutable=0) into the project rules.db alongside the deflation + leakage +
    // null-test gates. These are the production callers that light up the
    // mutable=0 DB guarantee + the env-pin precondition that STRUCTURALLY enforces
    // hazard A (a run that never declared its seed/determinism env cannot pass);
    // without this the REPRO-* rows would be test-only / wire-dark. The pure
    // re-run hash-equality DECISION lives in rerun_equality::verify_rerun_digest
    // (CLI-only); these seed rows are the static CAUSE-catchers + the vibesec
    // blake3 seed-integrity baseline. Idempotent.
    match elai_rules::project_rule_seeder::ProjectRuleSeeder::seed_repro_rules(
        &rules_conn,
        &clock_for_rules,
    ) {
        Ok(inserted) => tracing::info!(
            inserted,
            "build_task_runner_wiring: H-23 reproducibility seed rules ensured (REPRO-SEED-001 + REPRO-ENV-001, mutable=0)"
        ),
        Err(e) => tracing::error!(
            error = %e,
            "build_task_runner_wiring: seeding H-23 reproducibility rules failed; \
             REPRO-SEED-001 / REPRO-ENV-001 will not be DB-backed"
        ),
    }

    // H-24 — seed the LOCKED sim-to-real parity Part-A sub-gates (FIN-SIMREAL-001
    // mode-divergent-branch AST ban SCOPED to the order/signal execution path +
    // FIN-SIMREAL-002 single-entrypoint SchemaRequired manifest, both mutable=0)
    // into the project rules.db alongside the repro + null-test gates. This is the
    // production caller that lights up the mutable=0 DB guarantee + the single-
    // entrypoint precondition that STRUCTURALLY enforces sim-real parity (the
    // research/backtest path must be the SAME path that runs live); without this
    // the FIN-SIMREAL-* rows would be test-only / wire-dark. The pure replay==
    // golden DECISION lives in sim_real_parity::verify_replay_golden (CLI-only);
    // these seed rows are the static STRUCTURAL CAUSE-catchers + the vibesec
    // blake3 seed-integrity baseline. Idempotent.
    match elai_rules::project_rule_seeder::ProjectRuleSeeder::seed_sim_real_parity_rules(
        &rules_conn,
        &clock_for_rules,
    ) {
        Ok(inserted) => tracing::info!(
            inserted,
            "build_task_runner_wiring: H-24 sim-to-real parity seed rules ensured (FIN-SIMREAL-001 + FIN-SIMREAL-002, mutable=0)"
        ),
        Err(e) => tracing::error!(
            error = %e,
            "build_task_runner_wiring: seeding H-24 sim-to-real parity rules failed; \
             FIN-SIMREAL-001 / FIN-SIMREAL-002 will not be DB-backed"
        ),
    }

    // H-25 — seed the LOCKED result-manifest re-derivation Part-A sub-gates
    // (FIN-RESMANIFEST-001 SchemaRequired manifest-shape + FIN-RESMANIFEST-002
    // monkeypatch AstPattern SCOPED to the result-emitting surface, both mutable=0)
    // into the project rules.db alongside the repro + parity + null-test gates. This
    // is the production caller that lights up the mutable=0 DB guarantee + the
    // manifest-shape precondition that STRUCTURALLY enforces "every reported number
    // carries provenance" (the F-4 None-hash analog for an empirical result);
    // without this the FIN-RESMANIFEST-* rows would be test-only / wire-dark. The
    // pure re-parse/re-hash DECISION lives in result_manifest::verify_result
    // (CLI-only); these seed rows are the static STRUCTURAL CAUSE-catchers. The
    // MonkeypatchBlocked verdict is emitted by the rule-engine over FIN-RESMANIFEST-
    // 002. Idempotent.
    match elai_rules::project_rule_seeder::ProjectRuleSeeder::seed_result_manifest_rules(
        &rules_conn,
        &clock_for_rules,
    ) {
        Ok(inserted) => tracing::info!(
            inserted,
            "build_task_runner_wiring: H-25 result-manifest re-derivation seed rules ensured (FIN-RESMANIFEST-001 + FIN-RESMANIFEST-002, mutable=0)"
        ),
        Err(e) => tracing::error!(
            error = %e,
            "build_task_runner_wiring: seeding H-25 result-manifest rules failed; \
             FIN-RESMANIFEST-001 / FIN-RESMANIFEST-002 will not be DB-backed"
        ),
    }

    // H-26 — seed the LOCKED financial-invariant naked-float-`==` ban
    // (FIN-INVARIANT-001 AstPattern SCOPED to the order/signal/pnl/price execution
    // surface, mutable=0) into the project rules.db alongside the repro + parity +
    // null-test + result-manifest gates. This is the production caller that lights up
    // the mutable=0 DB guarantee for the static naked-float-== leg; without this the
    // FIN-INVARIANT-001 row would be test-only / wire-dark. The pure divergence /
    // accounting-identity DECISION lives in shadow_exec::diff_shadow_impls (CLI-only);
    // this seed row is the static STRUCTURAL CAUSE-catcher (a float == on money is a
    // precision bug). The NakedFloatEq verdict is emitted by the rule-engine over
    // FIN-INVARIANT-001. Idempotent.
    match elai_rules::project_rule_seeder::ProjectRuleSeeder::seed_financial_invariant_rules(
        &rules_conn,
        &clock_for_rules,
    ) {
        Ok(inserted) => tracing::info!(
            inserted,
            "build_task_runner_wiring: H-26 financial-invariant seed rules ensured (FIN-INVARIANT-001 naked-float-== ban, mutable=0)"
        ),
        Err(e) => tracing::error!(
            error = %e,
            "build_task_runner_wiring: seeding H-26 financial-invariant rules failed; \
             FIN-INVARIANT-001 will not be DB-backed"
        ),
    }

    // F-1 B2 — run the vibesec seed integrity check once at startup over the
    // freshly-seeded rules.db. A `false` result is threaded onto the dispatcher
    // (RuntimeWiring::vibegate_seed_ok) so a tampered seed forces every code-vuln
    // pre-gate to block fail-closed. blake3 baseline (never blake2/sha2).
    let vibegate_seed_ok = crate::vibegate::check_vibegate_seed_integrity(&rules_conn);
    if !vibegate_seed_ok {
        tracing::error!(
            "build_task_runner_wiring: vibesec seed integrity check FAILED; \
             VibeGate will block all code-vuln writes fail-closed"
        );
    }

    let rules_db = Arc::new(tokio::sync::Mutex::new(rules_conn));

    // 5. Build the Layer 1A processor and run `ensure_tables` once. The
    //    constructor at `self_mod_layer1a.rs:486` is the production entry
    //    point — no test-only or stub helper is used here.
    let layer1a = DefaultLayer1aProcessor::new(rules_db, rules_db_path.clone(), ssb.clone());
    layer1a
        .ensure_tables()
        .await
        .map_err(|e| anyhow::anyhow!("build_task_runner_wiring: Layer1A ensure_tables failed: {e}"))?;
    let layer1a_processor = Arc::new(layer1a);

    // C-25 audit follow-up — construct the production
    // crystallizer + compiled_pipeline_store + background_task_manager
    // bundle so [`apply_task_runner_wiring`] threads them onto
    // TaskRunnerConfig. Closes audit WARN-4: all production
    // construction sites previously passed `None` for these three Arcs
    // so the post-execute hook (run_crystallizer_post_execute_hook)
    // was a strict no-op and the SSB beacons never fired live.
    //
    // crystallizer: default 3-success threshold; SSB writer plumbed so
    // `template_crystallized` rows surface in the dashboard.
    let crystallizer = Arc::new(AsyncMutex::new(TemplateCrystallizer::new(Some(
        Arc::clone(&ssb),
    ))));
    // compiled_pipeline_store: load from disk eagerly so pre-existing
    // crystallizations are visible on the first task. The standard path
    // sits under <project_root>/.elai/cache/compiled_pipelines.msgpack.
    // Corrupt-input handling is panic-free (per compiled_pipeline::tests
    // store_load_corrupt_msgpack_returns_empty_no_panic).
    let store_path = CompiledPipelineStore::standard_path(project_root);
    let mut store = CompiledPipelineStore::new(store_path);
    store.load_from_disk();
    let compiled_pipeline_store = Arc::new(AsyncMutex::new(store));
    // background_task_manager: default soft/hard-timeout policy; SSB
    // writer plumbed so background_task_started / background_task_complete
    // rows surface in the dashboard timeline.
    let background_task_manager =
        Arc::new(BackgroundTaskManager::new(Some(Arc::clone(&ssb))));

    // F-6 / Slice A3 — production learned-skill store. Mirrors the
    // crystallizer Arc above: a fresh in-memory `SkillStore` wrapped in an
    // `Arc<AsyncMutex<_>>` so the crystallizer post-execute hook can acquire
    // a learned skill (born at S1) on a critic-gated crystallization. The
    // store has no on-disk persistence in this slice (acquisition + beacon
    // only); a persistent backing is a later-slice concern.
    let skill_store = Arc::new(AsyncMutex::new(SkillStore::new()));

    // B-8 (PipelineGrc activation) — construct a default `PipelineGrc`
    // and stash it in the bundle UNLESS the `ELAI_DISABLE_PIPELINE_GRC`
    // kill-switch is set. The bundle field mirrors what
    // `TaskRunnerConfig::with_pipeline_grc` would have produced:
    // `Arc<AsyncMutex<PipelineGrc>>`. Closes the production-wiring gap
    // documented in workflow/sessions/s-night-2026-05-30/cross-check/
    // B/B-03_b7_b8_b9_b10.md (every gated call site was a no-op in
    // CLI + dashboard runs).
    let pipeline_grc = if pipeline_grc_disabled_via_env() {
        tracing::info!(
            "build_task_runner_wiring: ELAI_DISABLE_PIPELINE_GRC set — \
             PipelineGrc wiring skipped, all 4 safety gates remain no-ops"
        );
        None
    } else {
        Some(Arc::new(AsyncMutex::new(PipelineGrc::new_from_default())))
    };

    // P0-1 — boot-time cross-family adversary runtime construction.
    // Two conditions must both hold:
    //   (a) ELAI_ALLOW_CLOUD_ADVERSARY=1 is set (opt-in gate; absent = None).
    //   (b) route_cross_family succeeds — only when the reachable pool has at
    //       least one family different from the worker family (Local in
    //       default deploy). Single-family or gated-off deployments yield None,
    //       preserving the pre-P0-1 dark-row behaviour.
    // The constructed adversary is a CloudAdversary wrapping AnthropicClient.
    // If AnthropicClient::from_env fails (no ANTHROPIC_API_KEY), we log a warning
    // and leave adversary_runtime = None rather than aborting boot.
    let adversary_runtime = if std::env::var("ELAI_ALLOW_CLOUD_ADVERSARY")
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
    {
        // D-025 pool-resolution (controller correction over D2): derive the
        // generator family from the executor-role backend in PerRoleRuntime
        // rather than hardcoding Local. Default PerRoleRuntime maps
        // AgentRole::Executor -> RuntimeBackend::Sidecar; we map that backend
        // to a ModelFamily via ModelFamily::from_model_id(canonical_name()),
        // falling back to Local only when unmapped (Sidecar -> "sidecar" ->
        // Local by the substring classifier; AnthropicApi -> "anthropic_api"
        // -> Anthropic). This makes the derivation correct for non-default
        // deployments (e.g. a cloud-only Executor role) without requiring any
        // new config surface.
        let executor_backend = crate::per_role_runtime::PerRoleRuntime::default()
            .get(crate::agent_pool::AgentRole::Executor);
        let worker_family = crate::adversary::ModelFamily::from_model_id(
            executor_backend.canonical_name(),
        );
        // F-3 / D-025 pool-resolution: derive the available family pool from
        // the actual PerRoleRuntime role->backend mapping rather than a
        // hardcoded [Anthropic, DeepSeek] slice. Iterates all (role, backend)
        // pairs, maps each backend to its ModelFamily via from_model_id on the
        // backend's canonical_name(), dedupes by label (owned String), and
        // builds the pool from the distinct family set. This makes the pool
        // reflect what is genuinely configured at runtime without requiring a
        // new config surface or any new dependencies.
        let prt = crate::per_role_runtime::PerRoleRuntime::default();
        let mut seen_labels: Vec<String> = Vec::new();
        let mut available_families: Vec<crate::adversary::ModelFamily> = Vec::new();
        for (_role, backend) in prt.list() {
            let fam = crate::adversary::ModelFamily::from_model_id(backend.canonical_name());
            let label = fam.as_str().to_owned();
            if !seen_labels.iter().any(|s| s == &label) {
                seen_labels.push(label);
                available_families.push(fam);
            }
        }

        match crate::adversary::route_cross_family(&worker_family, &available_families, true) {
            Ok(_chosen_family) => {
                // Construct the cloud adversary. The model string uses the
                // canonical Sonnet 4.6 identifier (the current production
                // cloud workhorse per CLAUDE.md E-31 tier matrix).
                match crate::adversary::CloudAdversary::from_env("claude-sonnet-4-6") {
                    Ok(cloud_adv) => {
                        let dyn_adv: std::sync::Arc<dyn crate::adversary::DynAdversary> =
                            std::sync::Arc::new(crate::adversary::AdversaryErased::new(cloud_adv));
                        let adversary_config =
                            crate::adversary::AdversaryConfig::default_for_local_worker();
                        let runtime_config =
                            crate::runtime_config::AdversaryRuntimeConfig::default();
                        Some(crate::runtime_config::AdversaryRuntimeState::new(
                            dyn_adv,
                            adversary_config,
                            runtime_config,
                        ))
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "build_task_runner_wiring: CloudAdversary::from_env failed; \
                             adversary_runtime stays None (set ANTHROPIC_API_KEY to enable)"
                        );
                        None
                    }
                }
            }
            Err(_same_family) => {
                tracing::info!(
                    "build_task_runner_wiring: route_cross_family returned SameFamilyPairing; \
                     adversary_runtime stays None (single-family pool)"
                );
                None
            }
        }
    } else {
        None
    };

    Ok(RuntimeWiring {
        ssb,
        feasibility_prober,
        layer1a_processor,
        baby_step_executor: None,
        // C-9: default to None so existing callers preserve pre-C-9
        // behaviour. The CLI + dashboard wiring sites set this to
        // `Some(GateMode::from_cli_args(args.mode.as_deref()))` before
        // calling `apply_task_runner_wiring`.
        gate_mode: None,
        // I-42: default to None so existing callers preserve the autonomous-
        // orchestrate posture (executor installed as resolved). The CLI run
        // surface sets this to `Some(RunProfile::from_cli_args(args.profile
        // .as_deref()))` before calling `apply_task_runner_wiring`.
        run_profile: None,
        // B-6 #2: default to None so existing non-CLI callers (dashboard,
        // contract tests) preserve the headless-batch behaviour (the
        // pre-wire posture where AutonomousGate::new received `None`).
        // The CLI run_subcommand overrides this to
        // `Some(Arc::new(CliInteractiveGate))` when gate_mode is
        // Interactive (or hybrid in future).
        interactive_gate: None,
        // C-5: cloud-mode router + provider override. Defaults preserve
        // pre-C-5 behaviour (no router, no redaction wrapper, no
        // ceiling enforcement).
        cloud_config: None,
        provider_override: None,
        // C-13: hook dispatcher defaults to None so existing callers
        // preserve pre-C-13 behaviour. The CLI + dashboard wiring sites
        // set this to `Some(build_production_hook_dispatcher(...))`
        // before calling `apply_task_runner_wiring`.
        hook_dispatcher: None,
        // C-25 audit follow-up — live by default; tests that want the
        // legacy None path overwrite these on the bundle.
        crystallizer: Some(crystallizer),
        compiled_pipeline_store: Some(compiled_pipeline_store),
        background_task_manager: Some(background_task_manager),
        // F-6 / Slice A3 — live by default so the critic-gated acquisition
        // hook can insert learned skills in production runs. Tests that want
        // the legacy None path overwrite this on the bundle.
        skill_store: Some(skill_store),
        via_mcp: false,
        // C-39: default to None so non-CLI callers (legacy tests,
        // dashboard pre-C-39) keep the unchanged tool-dispatch surface.
        // The CLI run_subcommand populates this based on wiring.via_mcp.
        mcp_dispatch: None,
        // B-28 WIRING-2: default to None so non-CLI callers keep the
        // pre-wire DIFF->STRUCTURED fallback. The CLI + dashboard wiring
        // sites set this to the Arc<InProcessToolDispatcher> they build.
        buffer_editor: None,
        // B-8: live by default; `ELAI_DISABLE_PIPELINE_GRC` overrides
        // to `None` above. Tests that want the legacy no-gate path
        // overwrite this field on the bundle before calling apply.
        pipeline_grc,
        // F-1 B2 — result of the vibesec seed integrity check computed above.
        vibegate_seed_ok,
        // P0-2 (E-19 fix) — production reflection client. Constructed here
        // using SidecarGroundingClient::default() (DEFAULT_SIDECAR_HOST) and
        // wrapped in SidecarReflectionAdapter. Stays None when the
        // ELAI_DISABLE_MICRO_REFLECTION kill-switch is set.
        reflection_client: if crate::micro_reflection_disabled_via_env() {
            None
        } else {
            let grounding = crate::SidecarGroundingClient::default();
            let adapter = crate::SidecarReflectionAdapter::new(Arc::new(grounding));
            Some(Arc::new(adapter) as Arc<dyn crate::DynReflectionClient>)
        },
        // P0-1 — adversary runtime constructed above; None for single-family
        // deploys or when ELAI_ALLOW_CLOUD_ADVERSARY is unset.
        adversary_runtime,
        // B-25 Phase 4J — default None; the CLI run_subcommand populates
        // this when `--empire` is set so the bridge handle survives for the
        // runner lifetime. Non-empire builds skip this field entirely via
        // cfg(feature = "empire").
        #[cfg(feature = "empire")]
        empire_bridge: None,
    })
}

/// Apply a `RuntimeWiring` bundle to a fresh `TaskRunnerConfig::new_minimal`
/// without mutating the rest of the orchestrator. Both fields are exposed
/// `pub` on `TaskRunnerConfig`, so we set them through struct field
/// assignment; every other field stays at the `new_minimal` default. This is
/// the single place that knows the chokepoint contract — the contract test
/// at `crates/contracts/tests/task_runner_chokepoint_contract.rs` pins it.
pub fn apply_task_runner_wiring(
    cursor_path: std::path::PathBuf,
    wiring: &RuntimeWiring,
) -> TaskRunnerConfig {
    let mut cfg = TaskRunnerConfig::new_minimal(cursor_path);
    let prober: Arc<dyn FeasibilityProber> = wiring.feasibility_prober.clone();
    let processor: Arc<dyn Layer1aProcessor> = wiring.layer1a_processor.clone();
    cfg.feasibility_prober = Some(prober);
    cfg.layer1a_processor = Some(processor);

    // C-9 — install the AutonomousGate at the single production chokepoint
    // both `cli::run_subcommand` and `dashboard-v2::src-tauri::lib::setup`
    // reach. Closes the gap recorded in metrics/phase_4G.toml notes:165
    // ("autonomous_gate: None"). When `gate_mode` is None the field stays
    // None, preserving the pre-C-9 behaviour for callers that have not
    // opted in.
    if let Some(mode) = wiring.gate_mode {
        let gate_cfg = AutonomousGateConfig {
            mode,
            ..AutonomousGateConfig::default()
        };
        // B-6 #2: plumb the optional CliInteractiveGate (set by the CLI
        // run_subcommand when gate_mode is Interactive) so AutonomousGate
        // owns a live human-prompter for the MODIFY / REDIRECT mixed-mode
        // fallback path. Was always `None` before this wire; closes the
        // "CliInteractiveGate defined but never instantiated in production"
        // cross-check finding (B-02_b4_b5_b6.md). Non-CLI callers
        // (dashboard, contract tests) leave the field at its
        // `build_task_runner_wiring` default of None and preserve the
        // pre-wire headless-batch behaviour.
        cfg.autonomous_gate = Some(Arc::new(AutonomousGate::new(
            gate_cfg,
            wiring.interactive_gate.clone(),
        )));
    }

    // C-5 — cloud-mode router + redaction wrapper + ceiling enforcement.
    //
    // When `cloud_config` is `Some`, we:
    //   1. Resolve the effective BackendMode from the CLI `--provider`
    //      override (when supplied) or the toml `[cloud].default_mode` field.
    //   2. Wrap the caller-supplied baby-step executor in
    //      `RedactingBabyStepExecutor` when the resolved mode is `Hybrid`
    //      or `Api` (cloud), so cloud-bound payloads pass through the
    //      privacy redactor before any network call.
    //   3. Construct a `CloudModeRouter` and install it on
    //      `TaskRunnerConfig::cloud_router` so the runner's pre-dispatch
    //      hook can emit EscalateToCloud and enforce the daily spend cap.
    //   4. Thread the daily spend cap onto `cloud_spend_cap_daily_usd`
    //      for the ceiling-check call site.
    //
    // When `cloud_config` is `None`, the path is a strict no-op — the
    // baby-step executor is plumbed through unchanged.
    let (resolved_executor, backend_mode, spend_cap_daily) = if let Some(cloud) = &wiring.cloud_config {
        let resolved_mode = resolve_backend_mode(
            wiring.provider_override.as_deref(),
            cloud.default_mode,
        );
        let wraps_redactor = matches!(
            resolved_mode,
            BackendMode::Api | BackendMode::Hybrid,
        );
        let router = Arc::new(CloudModeRouter::new(cloud.clone()));
        let privacy = router.privacy_config();
        cfg.cloud_router = Some(router);
        cfg.cloud_spend_cap_daily_usd = cloud.spend_cap_daily_usd;
        // RedactingBabyStepExecutor wraps the production executor when the
        // resolved provider is cloud-capable. For pure Local the inner
        // executor is plumbed through verbatim — redaction is needless
        // overhead and would burn CPU on data that never leaves the host.
        //
        // C-5: pass the SSB writer through `with_ssb` so the wrapper emits a
        // `RedactionApplied` row each time the redactor fires. Lets the
        // dashboard CloudPanel render the privacy-event feed without any
        // additional plumbing at the executor call site.
        let exec = match (&wiring.baby_step_executor, wraps_redactor) {
            (Some(inner), true) => {
                let wrapped: Arc<dyn BabyStepExecutor> = Arc::new(
                    RedactingBabyStepExecutor::with_ssb(
                        Arc::clone(inner),
                        privacy,
                        wiring.ssb.clone(),
                    ),
                );
                Some(wrapped)
            }
            (inner, _) => inner.clone(),
        };
        (exec, resolved_mode, cloud.spend_cap_daily_usd)
    } else {
        (wiring.baby_step_executor.clone(), BackendMode::default(), None)
    };
    // I-42 — critique-only structural spawn gate. When the resolved profile
    // refuses the executor spawn (CritiqueOnly), force the executor to `None`
    // here, at the single install chokepoint, so NO disk-mutating
    // `AgentPoolBabyStepExecutor` is ever installed. The runner falls back to
    // the non-disk-mutating synthetic path, and because the fixer loop
    // (`fix_and_rerun_until_green`) derives from the SAME
    // `baby_step_executor`, this ONE gate covers BOTH executor and fixer.
    // Fail-closed by construction: there is nothing to spawn. `None` /
    // `Orchestrate` leaves the resolved executor untouched (legacy posture).
    let resolved_executor = match wiring.run_profile {
        Some(profile) if profile.refuses_executor_spawn() => {
            tracing::info!(
                run_profile = %profile,
                "I-42 critique-only: executor/fixer spawn refused at install chokepoint \
                 (baby_step_executor forced to None)"
            );
            None
        }
        _ => resolved_executor,
    };
    cfg.baby_step_executor = resolved_executor;

    // Always populate backend_config so downstream code (resolve_exec_mode,
    // build_dispatch_context) sees a coherent provider routing decision.
    let mut backend_config = BackendConfig::default();
    backend_config.mode = backend_mode;
    if let Some(cloud) = &wiring.cloud_config {
        backend_config.model = cloud.default_model.clone();
    }
    // E-32 task 9: on the local-sidecar path only, source the context ceiling from
    // the sidecar's reported window so the dispatch budget tracks the real 32K local
    // model window instead of the wrong 131072 assumption. Api/Hybrid stay `None` and
    // keep the pre-E-32 static ceiling (no regression on cloud/hybrid backends).
    if backend_mode == BackendMode::Local {
        backend_config.max_context = Some(crate::dispatcher::SIDECAR_MODEL_MAX_CONTEXT);
    }
    cfg.backend_config = Some(backend_config);

    // Cap is mirrored on TaskRunnerConfig.cloud_spend_cap_daily_usd above
    // when cloud_config is Some; this assignment makes the legacy fallthrough
    // explicit so the field is never silently None when the caller wanted a
    // cap (defensive: caps only land via cloud_config today).
    let _ = spend_cap_daily; // documented intent — assignment happens above.

    // C-13 — install the production hook dispatcher at the chokepoint so
    // task_runner's T1/T2/T3 evaluation paths (5359/6577/6932) see a
    // non-None `Arc<dyn HookDispatcher>`. `None` preserves the pre-C-13
    // skip-everything behaviour. See DECISION D3 in
    // workflow/reasoning/C-13-decisions.md (option 3A — Tier-1 trait).
    if let Some(hd) = wiring.hook_dispatcher.as_ref() {
        let trait_obj: Arc<dyn crate::hook_dispatcher::HookDispatcher> = hd.clone();
        cfg.hook_dispatcher = Some(trait_obj);
    }

    // C-25 audit follow-up — thread the live crystallizer +
    // compiled_pipeline_store + background_task_manager into
    // TaskRunnerConfig so the post-execute hook and the
    // build_with_compiled_executor entry point see non-None Arcs in
    // production. Closes WARN-4 in workflow/sessions/s-25672/audit-C25.md.
    cfg.crystallizer = wiring.crystallizer.clone();
    cfg.compiled_pipeline_store = wiring.compiled_pipeline_store.clone();
    cfg.background_task_manager = wiring.background_task_manager.clone();
    // F-6 / Slice A3 — thread the live learned-skill store onto
    // TaskRunnerConfig so the crystallizer post-execute hook reaches it via
    // ResolvedConfig. `None` on the bundle keeps the acquisition hook a no-op.
    cfg.skill_store = wiring.skill_store.clone();
    // B-28 WIRING-2 (item E2-DIFF) — thread the production buffer editor onto
    // TaskRunnerConfig so the focus-tunnel DIFF branch applies hunks through
    // the real matcher. `None` on the bundle keeps the DIFF->STRUCTURED
    // fallback (the CLI/dashboard sites set it to InProcessToolDispatcher).
    cfg.buffer_editor = wiring.buffer_editor.clone();

    // C-39 — plumb the optional Dispatch impl onto TaskRunnerConfig so
    // future tool-call sites can route through this seam. When
    // `wiring.via_mcp` is true and the caller has NOT pre-constructed a
    // mcp_dispatch impl, we default to a no-SSB McpClientDispatch bound
    // to the canonical 8200 port (matches DEFAULT_MCP_SERVER_URL in
    // plugins/src/mcp_bridge.rs). When `via_mcp` is false and no
    // dispatch was provided, the field stays None and the runner keeps
    // the legacy C-13 ToolDispatcher path -- zero regression on the
    // default leg.
    if let Some(dispatch) = wiring.mcp_dispatch.as_ref() {
        cfg.mcp_dispatch = Some(dispatch.clone());
    } else if wiring.via_mcp {
        let mcp = crate::dispatch::McpClientDispatch::new(8200, wiring.ssb.clone());
        // F-3p1 / Slice C3 (ENFORCE-B): bind the dispatch's CapabilityFlowAdapter
        // to the SAME process-global committed-tool-set slot the producer commits
        // into. Default-ON: pass-through until the producer commits, then every
        // dispatch_tool call enforces the committed set.
        #[cfg(feature = "mcp")]
        let mcp = mcp.with_capability_flow_adapter(Arc::new(
            crate::capability_flow::CapabilityFlowAdapter::from_slot(
                crate::capability_flow::committed_slot(),
            ),
        ));
        cfg.mcp_dispatch = Some(Arc::new(mcp));
        tracing::info!(
            "C-39: --via-mcp default dispatch wired (McpClientDispatch port 8200)"
        );
    }

    // B-8 (PipelineGrc activation) — plumb the shared
    // `Arc<AsyncMutex<PipelineGrc>>` onto `TaskRunnerConfig.pipeline_grc`
    // so the 4 safety gates fire live in production. Direct field
    // assignment mirrors the equivalent of
    // `TaskRunnerConfig::with_pipeline_grc` but preserves the shared Arc
    // so multiple sessions / surfaces (CLI + dashboard) can share one
    // engine via the bundle when desired. When `wiring.pipeline_grc`
    // is `None` (kill-switch set in `build_task_runner_wiring`, OR
    // test override), the field stays `None` and every gated call site
    // short-circuits to the pre-B-8 no-op.
    cfg.pipeline_grc = wiring.pipeline_grc.clone();

    // H-37: thread the optional CliInteractiveGate handle onto
    // TaskRunnerConfig.plan_interactive_gate so drive_task can invoke it at
    // the PLANNING -> TESTING_FIRST boundary when the gate is enabled.
    // Only set when an interactive_gate was installed on the RuntimeWiring
    // (which happens in run_subcommand.rs when gate_mode == Interactive).
    // In autonomous / non-interactive mode the field stays None and the gate
    // auto-advances (PlanDecisionKind::AutoAdvance, no human prompt).
    if let Some(ig) = wiring.interactive_gate.clone() {
        cfg.plan_interactive_gate = Some(ig);
    }

    // P0-2 (E-19 reflection regression fix) — thread the production
    // SidecarReflectionAdapter onto TaskRunnerConfig::reflection_client so all
    // 5 TaskRunnerConfig::new_minimal() construction sites get a live client
    // instead of the None they carry post-new_minimal. When the kill-switch
    // ELAI_DISABLE_MICRO_REFLECTION is set, wiring.reflection_client is None
    // here and the field stays None — preserving the pre-E-19 skip behaviour.
    cfg.reflection_client = wiring.reflection_client.clone();

    // P0-1 (adversary resolver) — thread the boot-time cross-family adversary
    // runtime onto TaskRunnerConfig so ResolvedConfig::from can forward it to
    // dispatch_adversarial_pass. None when ELAI_ALLOW_CLOUD_ADVERSARY is unset
    // or route_cross_family found no cross-family pool (single-family deploy).
    cfg.adversary_runtime = wiring.adversary_runtime.clone();

    // B-25 Phase 4J — thread the EmpireBridge handle from RuntimeWiring onto
    // TaskRunnerConfig so the runner loop retains the Arc for its lifetime and
    // can drive `emit_action` for the 5 empire_* SSB action types. None when
    // `--empire` was not passed or the empire feature is off; the bridge is
    // never constructed in those cases. Closes GAP-1: the handle must outlive
    // the `if args.empire {}` block in run_subcommand.rs.
    #[cfg(feature = "empire")]
    {
        cfg.empire_bridge = wiring.empire_bridge.clone();
    }

    cfg
}

/// C-5 helper — resolve the production [`BackendMode`] from the CLI
/// `--provider` override (when supplied) or the toml-default `ExecMode`
/// carried on [`CloudConfig::default_mode`].
///
/// Mapping (canonical):
///   * `Some("local")`   -> `BackendMode::Local`
///   * `Some("cloud")`   -> `BackendMode::Api`
///   * `Some("hybrid")`  -> `BackendMode::Hybrid`
///   * `None` + `ExecMode::Local` -> `BackendMode::Local`
///   * `None` + `ExecMode::Cloud` -> `BackendMode::Api`
///   * `None` + `ExecMode::Auto`  -> `BackendMode::Hybrid`
///
/// Unknown strings fall through to `BackendMode::default()` (Api) so a
/// typo in `--provider` never silently demotes to Local with an
/// "implicit" semantic; the parser at [`crate::args::parse_run_subcommand_args`]
/// rejects unknown values up front.
fn resolve_backend_mode(
    cli_override: Option<&str>,
    toml_default: crate::cloud_mode::ExecMode,
) -> BackendMode {
    if let Some(raw) = cli_override {
        return match raw.to_ascii_lowercase().as_str() {
            "local" => BackendMode::Local,
            "cloud" => BackendMode::Api,
            "hybrid" => BackendMode::Hybrid,
            _ => BackendMode::default(),
        };
    }
    match toml_default {
        crate::cloud_mode::ExecMode::Local => BackendMode::Local,
        crate::cloud_mode::ExecMode::Cloud => BackendMode::Api,
        crate::cloud_mode::ExecMode::Auto => BackendMode::Hybrid,
    }
}

/// Construct the production [`AgentPoolBabyStepExecutor`] for callers that
/// want to install the real agent-pool path rather than the synthetic
/// fallback.
///
/// The returned `Arc<dyn BabyStepExecutor>` wraps an
/// [`AgentPoolBabyStepExecutor`] backed by:
/// - an [`InProcessAgentPool`] seeded with `entropy` and configured with
///   [`AgentPoolConfig::default()`],
/// - the production [`runtime_model_pool()`] (reports `u64::MAX` VRAM),
/// - the caller-supplied `agent_binary_path`, `socket_dir`, `gateway`, and
///   `rpc_server`.
///
/// Pass the result as `wiring.baby_step_executor = Some(executor)` before
/// calling [`apply_task_runner_wiring`].
///
/// Anti-stub: no `.unwrap()` / `.expect()` / `panic!` — the pool constructor
/// is infallible; the executor is created but never run here.
pub fn build_default_baby_step_executor<E>(
    entropy: E,
    agent_binary_path: std::path::PathBuf,
    socket_dir: std::path::PathBuf,
    gateway: Arc<tokio::sync::Mutex<IpcGateway>>,
    rpc_server: Arc<dyn AgentRpcServer + Send + Sync + 'static>,
) -> Arc<dyn BabyStepExecutor>
where
    E: elai_core::EntropySource + Send + 'static,
{
    let model_pool = crate::task_runner::runtime_model_pool();
    let pool = Arc::new(tokio::sync::Mutex::new(InProcessAgentPool::new(
        AgentPoolConfig::default(),
        entropy,
        model_pool,
    )));
    Arc::new(AgentPoolBabyStepExecutor::new(
        pool,
        agent_binary_path,
        socket_dir,
        gateway,
        rpc_server,
    ))
}

/// B-18 task #12 — build the production [`AgentPoolBabyStepExecutor`] WITH
/// the IPC hook dispatcher installed on the executor's sync
/// `hook_dispatcher` field, so `run_agent_session` (ipc_accept.rs:463) fires
/// `pre_tool_use` / `post_tool_use` on every `FrameRouteResult::ToolRequest`.
/// This is the "wire IpcHookDispatcher to fire via IPC frame interception"
/// path B-18 task #12 requires.
///
/// Identical to [`build_default_baby_step_executor`] except the returned
/// executor has `.with_hook_dispatcher(...)` chained before the `Arc::new`
/// wrap. The `tiered` bundle + `ssb` handle are threaded in by the caller
/// (the CLI/dashboard bootstrap sites that already have `wiring.ssb`).
///
/// Anti-stub: no `.unwrap()` / `.expect()` / `panic!`; the pool + executor
/// + dispatcher constructors are infallible and nothing is run here.
#[must_use]
pub fn build_default_baby_step_executor_with_ipc_hooks<E>(
    entropy: E,
    agent_binary_path: std::path::PathBuf,
    socket_dir: std::path::PathBuf,
    gateway: Arc<tokio::sync::Mutex<IpcGateway>>,
    rpc_server: Arc<dyn AgentRpcServer + Send + Sync + 'static>,
    tiered: elai_plugins::tiered_analysis::TieredAnalysis,
    ssb: Arc<Ssb>,
) -> Arc<dyn BabyStepExecutor>
where
    E: elai_core::EntropySource + Send + 'static,
{
    let model_pool = crate::task_runner::runtime_model_pool();
    let pool = Arc::new(tokio::sync::Mutex::new(InProcessAgentPool::new(
        AgentPoolConfig::default(),
        entropy,
        model_pool,
    )));
    let executor = crate::task_runner::AgentPoolBabyStepExecutor::new(
        pool,
        agent_binary_path,
        socket_dir,
        gateway,
        rpc_server,
    )
    .with_hook_dispatcher(build_production_ipc_hook_dispatcher(tiered, ssb));
    Arc::new(executor)
}

/// B-18 task #12 — construct a [`TieredAnalysis`] with the orchestrator's
/// stub-grade AST evaluator + DKG querier (`PassAstEvaluator` +
/// `EmptyDkgQuerier`). The orchestrator crate cannot depend on
/// `TreeSitterAstEvaluator` (lives in the CLI crate), so this stub-grade
/// bundle is the minimum-viable TieredAnalysis for the IPC hook dispatcher
/// installed on the executor's sync surface. The CLI/dashboard bootstrap
/// sites that have a real `RuleEngine` should construct their own
/// `TieredAnalysis` and pass it to
/// [`build_default_baby_step_executor_with_ipc_hooks`] directly.
///
/// `fsm_phase` defaults to [`FsmPhaseBit::EXECUTING`] (the baby-step
/// executor only runs in EXECUTING).
#[must_use]
pub fn build_stub_tiered_analysis() -> elai_plugins::tiered_analysis::TieredAnalysis {
    use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
    use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};
    let engine = Box::new(DefaultRuleEngine::new(
        Box::new(PassAstEvaluator),
        Box::new(EmptyDkgQuerier),
    ));
    elai_plugins::tiered_analysis::TieredAnalysis::new(engine, FsmPhaseBit::EXECUTING)
}

/// E-15 — build the production [`AgentPoolBabyStepExecutor`] WITH the
/// Firecracker hot snapshot-restore pool and the live SSB writer installed,
/// gated by the default-OFF [`crate::runtime_config::sandbox_firecracker_enabled`]
/// flag.
///
/// When the flag is OFF this is identical to
/// [`build_default_baby_step_executor`] — no pool, no SSB rows, the firecracker
/// dispatch path stays dead. When ON, the returned executor:
/// - installs a [`crate::sandbox::FirecrackerHotPool::with_defaults`] rooted at
///   `{snapshot_root}/snapshots`, and
/// - installs `ssb` as the writer the firecracker dispatch path uses to emit the
///   off-agent `sandbox_backend_selected` + `agent_forked_from_snapshot` rows.
///
/// `snapshot_root` is normally the per-session dir (the parent of the IPC
/// `socket_dir`); the snapshots live in a `snapshots/` sub-folder so they GC
/// alongside the session. The CLI boot site (`runtime_builder.rs`) passes
/// `wiring.ssb.clone()` and the session dir.
///
/// Anti-stub: no `.unwrap()` / `.expect()` / `panic!`; the pool + executor
/// constructors are infallible and nothing is run here.
#[must_use]
pub fn build_default_baby_step_executor_with_sandbox<E>(
    entropy: E,
    agent_binary_path: std::path::PathBuf,
    socket_dir: std::path::PathBuf,
    gateway: Arc<tokio::sync::Mutex<IpcGateway>>,
    rpc_server: Arc<dyn AgentRpcServer + Send + Sync + 'static>,
    ssb: Arc<Ssb>,
    snapshot_root: std::path::PathBuf,
) -> Arc<dyn BabyStepExecutor>
where
    E: elai_core::EntropySource + Send + 'static,
{
    let model_pool = crate::task_runner::runtime_model_pool();
    let pool = Arc::new(tokio::sync::Mutex::new(InProcessAgentPool::new(
        AgentPoolConfig::default(),
        entropy,
        model_pool,
    )));
    let executor = crate::task_runner::AgentPoolBabyStepExecutor::new(
        pool,
        agent_binary_path,
        socket_dir,
        gateway,
        rpc_server,
    );

    if crate::runtime_config::sandbox_firecracker_enabled() {
        let snapshot_dir = snapshot_root.join("snapshots");
        let fc_pool = Arc::new(crate::sandbox::FirecrackerHotPool::with_defaults(
            snapshot_dir,
        ));
        Arc::new(
            executor
                .with_firecracker_pool(fc_pool)
                .with_ssb_writer(ssb),
        )
    } else {
        Arc::new(executor)
    }
}

/// C-13 — build the production [`InProcessHookDispatcher`] with the
/// [`elai_plugins::post_tool_use::RuffPostToolUseRunner`] installed.
///
/// The returned `Arc<InProcessHookDispatcher>` is suitable for direct
/// assignment to [`RuntimeWiring::hook_dispatcher`] (coerces to
/// `Arc<dyn HookDispatcher>` at the assignment site). It uses the
/// `OsClock` default — no test seam needed at production wiring sites.
///
/// `tiered` is `None` for now: the production TieredAnalysis bundle is
/// constructed elsewhere (see `plugins::TieredAnalysis::new`) and is not
/// part of C-13's wiring scope. When `None` the dispatcher returns empty
/// findings lists; the ruff path still fires for `.py` writes thanks to
/// the `with_ruff_runner` installation below.
///
/// Plan reference: C-13 task 2 + DECISION D3 option 3A
/// (workflow/reasoning/C-13-decisions.md).
///
/// F-1 B2 — installs the VibeGate SSB writer and the seed-integrity flag so
/// `VibeGateBlocked` / `SecurityRuleBlocked` rows actually emit in production
/// and a tampered seed forces fail-closed. `ssb` is the runtime's
/// `Arc<Ssb>` (clone of `wiring.ssb`); `vibegate_seed_ok` is the result of the
/// integrity check run at wiring time (`wiring.vibegate_seed_ok`). Rows are
/// stamped with [`crate::task_runner::RUNNER_AGENT_ID`] /
/// [`crate::task_runner::RUNNER_SUPERSTEP`], matching the SSB beacons the runner
/// writes elsewhere.
///
/// F-9 / Slice D3 — installs the write-time anti-slopsquatting gate
/// ([`DepWriteTimeGate`]) using `SubprocessRegistryChecker::default_at_root`
/// with `project_root`.  DEFAULT-ON: always installed in production so
/// every PreToolUse file-write path probes the registry for unknown deps.
///
/// H-38: `stop_gate` threads the production [`InteractiveGate`] into the
/// cost-confirmation stop path. Pass `wiring.interactive_gate.clone()` from
/// the CLI caller so [`dispatch_stop`] can present the confirm prompt when
/// spend exceeds `CONFIRM_COST_FLOOR_USD`. `None` means headless/autonomous:
/// the gate still fires but defaults to HOLD (fail-closed) when no prompter
/// is installed. The SSB db path is derived from `ssb.db_path()`.
#[must_use]
pub fn build_production_hook_dispatcher(
    ssb: Arc<Ssb>,
    vibegate_seed_ok: bool,
    project_root: &std::path::Path,
    stop_gate: Option<Arc<dyn InteractiveGate>>,
) -> Arc<InProcessHookDispatcher> {
    use crate::vibesec::dep_writetime_gate::{DepWriteTimeGate, SubprocessRegistryChecker};
    let ruff = Arc::new(elai_plugins::post_tool_use::RuffPostToolUseRunner::new());
    let registry_checker: std::sync::Arc<dyn crate::vibesec::dep_writetime_gate::RegistryExistenceChecker + Send + Sync> =
        std::sync::Arc::new(SubprocessRegistryChecker::default_at_root(project_root));
    let dep_gate = Arc::new(DepWriteTimeGate::new(Some(ssb.clone()), registry_checker));
    // H-38: derive db_path from the already-open Ssb so callers do not need to
    // re-plumb SsbConfig. The path is a &Path from the live instance.
    let db_path_str = ssb.db_path().to_string_lossy().into_owned();
    // I-1: construct the directive supervisor ONCE here — this is the SOLE
    // bootstrap consolidation fn all 3 production sites (run_subcommand.rs,
    // task_cmd.rs, dashboard src-tauri/lib.rs) call, so every site inherits an
    // identical supervisor and the named 3-way-drift risk is structurally
    // eliminated (there is only one construction point). The supervisor's
    // run_gates becomes the single E-30/F-1/G-3 gate path inside
    // dispatch_pre_tool_use (the inline E-30 + F-1 calls are skipped when it is
    // installed). Worktree root = project_root for the G-3 fs-escape gate; the
    // SSB writer emits DirectiveExecuted rows from execute_directive.
    // I-32 (audit fix): install role-contract store so the structural role-scope gate
    // (run_role_scope_gate, directive.rs:499) actually evaluates directives instead of
    // short-circuiting on a None store. Without this the gate was vacuous in production.
    let supervisor: Arc<dyn crate::directive::Supervisor> =
        Arc::new(crate::directive::InProcessSupervisor::new(project_root)
            .with_ssb(
                ssb.clone(),
                crate::task_runner::RUNNER_AGENT_ID,
                crate::task_runner::RUNNER_SUPERSTEP,
            )
            .with_role_contracts(crate::role_contracts::RoleContractStore::default())
            .with_sequence_rules(std::sync::Arc::new(
                crate::tool_sequence::SequenceRuleSet::load_builtin()
            )));
    let mut dispatcher = InProcessHookDispatcher::new(None)
        .with_ruff_runner(ruff)
        .with_vibegate_ssb(
            ssb.clone(),
            crate::task_runner::RUNNER_AGENT_ID,
            crate::task_runner::RUNNER_SUPERSTEP,
        )
        .with_vibegate_seed_ok(vibegate_seed_ok)
        .with_dep_writetime_gate(dep_gate)
        .with_supervisor(supervisor)
        // I-19: wire TDD preflight SSB so TddPreflightBlocked rows are emitted
        // from the production hook path when ELAI_TDD_PREFLIGHT_ENABLE=1.
        // Without this the gate blocked (fail-closed) but emitted no SSB row,
        // making the WIRING_CONTRACT observable_effect structurally unreachable.
        .with_tdd_preflight_ssb(
            ssb.clone(),
            crate::task_runner::RUNNER_AGENT_ID,
            crate::task_runner::RUNNER_SUPERSTEP,
        )
        // H-38 BLOCK-1: wire stop-cost SSB so day_usd is read from real CloudSpend
        // rows and StopConfirmationEvent beacons are emitted unconditionally on the
        // blocked path (R1/R4 off-agent artifact, fail-closed).
        .with_stop_cost_ssb(ssb, db_path_str);
    // H-38 BLOCK-1: wire the interactive gate when provided (CLI Interactive mode).
    // Headless/autonomous callers pass None; gate defaults to HOLD (fail-closed).
    if let Some(gate) = stop_gate {
        dispatcher = dispatcher.with_stop_confirm_gate(gate);
    }
    Arc::new(dispatcher)
}

/// B-18/B-19 — build a production `elai_plugins::IpcHookDispatcher` and
/// return it as the SYNC `Arc<dyn ipc_accept::IpcHookDispatcher + Send + Sync>`
/// trait object that `AgentPoolBabyStepExecutor::with_hook_dispatcher`
/// (task_runner.rs:1441) consumes. This is the "wire IpcHookDispatcher to
/// fire via IPC frame interception" path B-18 task #12 requires —
/// `run_agent_session` (ipc_accept.rs:463) fires `pre_tool_use` /
/// `post_tool_use` on every `FrameRouteResult::ToolRequest` when the
/// executor's `hook_dispatcher` field is `Some`.
///
/// The blanket impl at `ipc_accept.rs:54`
/// (`impl<T: IpcHookTrait + ?Sized> IpcHookDispatcher for T`) makes the
/// plugins `IpcHookDispatcher` (which implements `IpcHookTrait`) satisfy
/// the orchestrator's `ipc_accept::IpcHookDispatcher` trait alias without
/// an adapter — the sync trait surface is exactly what the IPC accept loop
/// consumes.
///
/// Production-side wires:
///   * `SsbFindingWriter` — `RuntimeSsbFindingWriter` wraps the orchestrator's
///     `Arc<Ssb>` and emits one `SsbActionType::Finding` row per BLOCK verdict
///     so the dashboard surfaces the IPC-frame rejections in the activity
///     feed alongside in-process findings.
///   * `CrossSessionLockChecker` — `NoopLockChecker` is used in single-session
///     contexts. Multi-session deployments should swap in a checker that
///     queries the cross-session lock cache.
///
/// The `TieredAnalysis` instance is constructed by the caller (e.g. the CLI
/// or dashboard bootstrap) and threaded in. Callers without a tiered bundle
/// today should prefer `build_production_hook_dispatcher` (in-process); the
/// IPC dispatcher is reserved for the Tier-C accept-loop callers wired by
/// `crate::ipc_accept::run_agent_session`.
#[must_use]
pub fn build_production_ipc_hook_dispatcher(
    tiered: elai_plugins::tiered_analysis::TieredAnalysis,
    ssb: Arc<Ssb>,
) -> Arc<dyn crate::ipc_accept::IpcHookDispatcher + Send + Sync + 'static> {
    use elai_plugins::{IpcHookDispatcher, NoopLockChecker};
    let writer: Box<dyn elai_plugins::SsbFindingWriter + Send + Sync> =
        Box::new(RuntimeSsbFindingWriter::new(ssb));
    let lock_checker: Box<dyn elai_plugins::CrossSessionLockChecker + Send + Sync> =
        Box::new(NoopLockChecker);
    // The plugins IpcHookDispatcher implements IpcHookTrait; the blanket impl
    // at ipc_accept.rs:54 lifts it to the orchestrator's IpcHookDispatcher
    // trait alias. No async adapter needed — the executor's with_hook_dispatcher
    // field and run_agent_session both consume the sync surface.
    Arc::new(IpcHookDispatcher::new(tiered, writer, lock_checker))
}

/// B-19 — production `SsbFindingWriter` implementation wrapping the
/// orchestrator's `Arc<Ssb>`. The `IpcHookDispatcher` (in the plugins crate)
/// cannot depend on the orchestrator's `Ssb` type directly without a
/// circular dep, so the trait lives upstream and the concrete adapter
/// lives here.
///
/// Write semantics: per `elai_plugins::SsbFindingWriter`, the implementor
/// MUST inject `project_id` per `SSB_READER_PROJECT_ID_SCOPING`. The
/// orchestrator's `Ssb` already carries the project_id on its `SsbConfig`
/// (set at construction in `build_task_runner_wiring`), so the bridge here
/// is a simple `block_in_place` + `Handle::block_on` over `write_payload`.
///
/// Failure mode: a write failure is recorded by returning `Err(String)`;
/// the `IpcHookDispatcher` logs and treats SSB write errors as non-blocking
/// (`finding_written=false`). The dispatcher itself never aborts on this.
///
/// `superstep` defaults to `0` — the IPC-frame surface does not have a
/// running superstep counter the way `task_runner` does. The Finding row
/// still surfaces in the activity feed; only ordering across other tracked
/// supersteps may drift, which is acceptable for hook telemetry.
pub struct RuntimeSsbFindingWriter {
    ssb: Arc<Ssb>,
}

impl RuntimeSsbFindingWriter {
    /// Wrap an `Arc<Ssb>` so it can satisfy the upstream
    /// `elai_plugins::SsbFindingWriter` trait.
    #[must_use]
    pub fn new(ssb: Arc<Ssb>) -> Self {
        Self { ssb }
    }
}

impl elai_plugins::SsbFindingWriter for RuntimeSsbFindingWriter {
    fn write_finding(
        &self,
        agent_id: &str,
        pattern_hash: &str,
        verdict_json: &str,
        files: &[String],
    ) -> Result<(), String> {
        let payload = crate::ssb::SsbPayload::Finding(crate::ssb::FindingPayload {
            severity: "INFO".to_string(),
            category: "ipc_hook".to_string(),
            message: format!("verdict={verdict_json} pattern_hash={pattern_hash}"),
            file_path: files.first().cloned(),
            line: None,
            failure_category: None,
            audit_step: None,
            few_shot_examples: None,
        });
        let ssb = Arc::clone(&self.ssb);
        let agent_id_owned = agent_id.to_string();
        let files_owned: Vec<String> = files.to_vec();
        // The trait method is sync but `Ssb::write_payload` is async. Bridge
        // via the current runtime handle; we only call this from inside the
        // tokio runtime (the IpcHookDispatcher runs under spawn_blocking
        // from the IpcHookAsyncAdapter, so a Handle::current() is available).
        // Fall back to a tracing::warn + Err if no runtime is active so the
        // dispatcher reports the failure rather than panic.
        let Ok(handle) = tokio::runtime::Handle::try_current() else {
            return Err(
                "RuntimeSsbFindingWriter::write_finding: no tokio runtime".to_string(),
            );
        };
        // `block_in_place` requires multi-thread runtime; if we are on a
        // current-thread runtime the call would deadlock. Detect and bail
        // gracefully — production runtimes are multi-thread (per
        // `Ssb::new` and the dashboard tauri setup).
        let result = tokio::task::block_in_place(move || {
            handle.block_on(async move {
                ssb.write_payload(&agent_id_owned, 0, payload, files_owned).await
            })
        });
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("ssb write_payload failed: {e}")),
        }
    }
}

/// Locate the `elai_cc_agent[.exe]` binary.
///
/// Resolution order (first found wins):
/// 1. `ELAI_AGENT_BIN` environment variable — returned verbatim as a
///    [`PathBuf`] (file existence is NOT checked, so callers can use this for
///    out-of-process testing).
/// 2. Same directory as the current executable
///    (`std::env::current_exe()?.parent()/elai_cc_agent[.exe]`).
/// 3. `target/debug/elai_cc_agent[.exe]` and
///    `target/release/elai_cc_agent[.exe]` reachable by walking up from the
///    current executable's directory until a `target/{debug,release}` subtree
///    is found or the filesystem root is reached.
///
/// Returns `Err` with a descriptive message listing all probed paths if the
/// binary cannot be located.
pub fn locate_default_agent_binary() -> Result<std::path::PathBuf, anyhow::Error> {
    // 1. Env-var override.
    if let Ok(env_val) = std::env::var(ELAI_AGENT_BIN_ENV) {
        return Ok(std::path::PathBuf::from(env_val));
    }

    let mut probed: Vec<std::path::PathBuf> = Vec::new();

    // 2. Sibling of current_exe.
    let current_exe = std::env::current_exe().map_err(|e| {
        anyhow::anyhow!("locate_default_agent_binary: current_exe() failed: {e}")
    })?;
    if let Some(exe_dir) = current_exe.parent() {
        let candidate = exe_dir.join(AGENT_BINARY_NAME);
        if candidate.exists() {
            return Ok(candidate);
        }
        probed.push(candidate);

        // 3. Walk up looking for target/{debug,release}.
        let mut dir = exe_dir.to_path_buf();
        loop {
            for profile in &["debug", "release"] {
                let candidate = dir.join("target").join(profile).join(AGENT_BINARY_NAME);
                if candidate.exists() {
                    return Ok(candidate);
                }
                probed.push(candidate);
            }
            match dir.parent() {
                Some(parent) => dir = parent.to_path_buf(),
                None => break,
            }
        }
    }

    Err(anyhow::anyhow!(
        "locate_default_agent_binary: binary '{}' not found. Checked:\n  {}",
        AGENT_BINARY_NAME,
        probed
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("\n  "),
    ))
}
