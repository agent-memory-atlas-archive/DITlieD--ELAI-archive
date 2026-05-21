// Phase A1a — `elai run` subcommand.
//
// First production caller of `start_task_runner` outside the orchestrator
// crate. The dashboard never spawned the task_runner (the dashboard reads
// SSB rows but does not own the runner loop), so this module is the
// canonical entry that turns a SSB `TaskSubmitted` row into a live
// `drive_task` invocation.
//
// Contract test: `crates/contracts/tests/task_runner_chokepoint_contract.rs`.
// Spec gap closed: `metrics/phase_4F.toml:103`
// (`production_construction_paths_wired = false`).

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::OsEntropy;
use elai_orchestrator::start_task_runner;

use crate::runtime_builder::{apply_task_runner_wiring, build_task_runner_wiring};

/// Parsed `elai run` arguments.
#[derive(Debug, Clone)]
pub struct RunArgs {
    /// Project root the runner should bind to. Defaults to the CWD when
    /// the user does not pass `--project-root`.
    pub project_root: PathBuf,
    /// C-9 — `--mode autonomous|interactive|hybrid`. `None` resolves to
    /// `GateMode::Interactive` at the wiring site
    /// ([`elai_orchestrator::GateMode::from_cli_args`]). Unknown values are
    /// rejected at parse time in [`crate::args::parse_run_subcommand_args`].
    pub mode: Option<String>,
    /// C-5 — `--provider local|cloud|hybrid`. Maps to
    /// [`elai_orchestrator::dispatcher::BackendMode`] at the wiring site:
    ///   * `local`  -> `BackendMode::Local`
    ///   * `cloud`  -> `BackendMode::Api`
    ///   * `hybrid` -> `BackendMode::Hybrid` (local-first, cloud-escalate)
    /// `None` => use the toml default from `[cloud].default_mode`.
    /// Unknown values are rejected at parse time.
    pub provider: Option<String>,
    /// C-39 -- route tool calls through MCP bridge instead of in-process plugins.
    pub via_mcp: bool,
}

/// Synchronous entry consumed by the `CliAction::Run` match arm. Owns its
/// own tokio runtime — the parent `fn run()` in `main.rs` is sync, so we
/// cannot share the runtime that `runtime_client.rs` builds for the REPL
/// path.
pub(crate) fn run_task_runner_subcommand(args: RunArgs) -> Result<(), anyhow::Error> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_name("elai-runner")
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("elai run: tokio runtime init failed: {e}"))?;
    runtime.block_on(async move { run_async(args).await })
}

/// Async body of `elai run`. Builds the production wiring, spawns the task
/// runner, and waits for either runner completion or a Ctrl-C.
async fn run_async(args: RunArgs) -> Result<(), anyhow::Error> {
    // session_id matches the dashboard's convention (`dash-v2-<pid>` at
    // state.rs:452). The CLI variant uses a different prefix so the two
    // surfaces are distinguishable in SSB row diagnostics.
    let session_id = format!("cli-run-{}", std::process::id());
    let project_root = args.project_root.clone();
    tracing::info!(
        project_root = %project_root.display(),
        session_id = %session_id,
        "elai run: starting task_runner"
    );

    let mut wiring = build_task_runner_wiring(&project_root, &session_id).await?;
    // C-9 — resolve the autonomous-gate mode from the parsed `--mode` flag.
    // `GateMode::from_cli_args(None)` returns Interactive so omitting the
    // flag preserves the pre-C-9 default. With this assignment the
    // `apply_task_runner_wiring` chokepoint installs a real
    // `AutonomousGate` on TaskRunnerConfig.autonomous_gate (was always
    // `None` before C-9 per phase_4G.toml notes:165).
    wiring.gate_mode = Some(elai_orchestrator::GateMode::from_cli_args(
        args.mode.as_deref(),
    ));
    tracing::info!(
        gate_mode = ?wiring.gate_mode,
        "elai run: autonomous-gate mode resolved"
    );

    // C-5 — load `[cloud]` from `elai_cc.toml` and stash on the wiring so
    // `apply_task_runner_wiring` can construct the CloudModeRouter +
    // RedactingBabyStepExecutor + spend-cap enforcement. The CLI
    // `--provider` flag overrides the toml `[cloud].default_mode`; absent
    // a flag and absent a toml file we still ship a default CloudConfig
    // so the router is always constructable and the dashboard CloudPanel
    // never sees a "router not wired" state.
    let elai_config = crate::runtime_config::load_elai_config(&project_root);
    let cloud_config = elai_config.cloud.to_cloud_config();
    tracing::info!(
        provider_override = ?args.provider,
        toml_default_mode = ?cloud_config.default_mode,
        privacy_profile = ?cloud_config.privacy_profile,
        air_gapped = cloud_config.air_gapped,
        spend_cap_daily_usd = ?cloud_config.spend_cap_daily_usd,
        "elai run: cloud-mode router resolved"
    );
    wiring.cloud_config = Some(cloud_config);
    wiring.provider_override = args.provider.clone();
    wiring.via_mcp = args.via_mcp;
    if args.via_mcp {
        tracing::info!(port = 8200, "C-39: --via-mcp active; tool calls will route through MCP bridge at port {}", 8200);
    }

    // Cursor lives under <project_root>/.elai_cc/sessions/<sid>/cursor.json
    // so the runner can resume across CLI invocations from the same
    // project. The build_task_runner_wiring helper already created the
    // session dir, so this path is guaranteed to be writable.
    let cursor_path = project_root
        .join(elai_core::paths::ELAI_DATA_DIR_NAME)
        .join("sessions")
        .join(&session_id)
        .join("cursor.json");

    // Attempt to install the production agent-pool executor. The IPC socket
    // directory lives alongside the cursor so all runner artefacts are
    // co-located under the session subtree.
    let ipc_dir = project_root
        .join(elai_core::paths::ELAI_DATA_DIR_NAME)
        .join("sessions")
        .join(&session_id)
        .join("ipc");
    if let Err(e) = std::fs::create_dir_all(&ipc_dir) {
        tracing::warn!(
            path = %ipc_dir.display(),
            error = %e,
            "elai run: failed to create IPC socket dir; executor will fall back to synthetic"
        );
    } else {
        match elai_orchestrator::locate_default_agent_binary() {
            Ok(agent_bin) => {
                let gateway = std::sync::Arc::new(tokio::sync::Mutex::new(
                    elai_orchestrator::ipc_gateway::IpcGateway::with_os_clock(
                        ipc_dir.clone(),
                        elai_orchestrator::sandbox::SelfModificationLocks::new(),
                    ),
                ));
                let rpc_server: std::sync::Arc<
                    dyn elai_orchestrator::AgentRpcServer + Send + Sync + 'static,
                > = if std::env::var("ELAI_GROUNDING_BACKEND").as_deref()
                    == Ok("cloud")
                {
                    // Cloud path: AnthropicClient reads ANTHROPIC_BASE_URL +
                    // auth from env. If credentials are absent, fall back to
                    // NoOp so the executor still starts (inference will fail
                    // gracefully per InferOutcome::Failed).
                    match elai_providers::AnthropicClient::from_env() {
                        Ok(client) => {
                            let model = std::env::var("ELAI_MODEL")
                                .unwrap_or_else(|_| "claude-sonnet-4-6".to_string());
                            let max_tokens = std::env::var("ELAI_MAX_TOKENS")
                                .ok()
                                .and_then(|v| v.parse::<u32>().ok())
                                .unwrap_or(4096);
                            // Build a real InProcessToolDispatcher via elai-tools.
                            // NoOpIntegrationCommandRunner returns honest "not wired" failures
                            // for the integration tier; the 4 other dispatch methods are real.
                            let dispatcher: std::sync::Arc<
                                dyn elai_core::tool_dispatcher::ToolDispatcher,
                            > = {
                                struct NoOpIntegrationRunner;
                                impl elai_tools::tiered_validation::IntegrationCommandRunner
                                    for NoOpIntegrationRunner
                                {
                                    fn run_integration<'a>(
                                        &'a self,
                                        _level: &'a str,
                                        _dir: &'a std::path::Path,
                                    ) -> std::pin::Pin<
                                        Box<
                                            dyn std::future::Future<
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
                                                    "integration tier not wired \
                                                     (NoOpIntegrationRunner in CLI)"
                                                        .to_string(),
                                                ),
                                            }
                                        })
                                    }
                                }
                                let validator = elai_tools::dispatcher::default_validator(
                                    std::sync::Arc::new(NoOpIntegrationRunner),
                                    "rust",
                                );
                                // C-13 — wrap the real InProcessToolDispatcher in the
                                // IpcToolDispatcherAdapter shim so every production
                                // tool call (a) passes the IpcToolRegistry FSM gate
                                // and (b) emits the 10 missing DAIRA markers via
                                // AdaptiveTool. See workflow/reasoning/C-13-decisions.md
                                // (D1 Option B).
                                let inner: std::sync::Arc<
                                    dyn elai_core::tool_dispatcher::ToolDispatcher,
                                > = std::sync::Arc::new(
                                    elai_tools::dispatcher::InProcessToolDispatcher::new(
                                        std::sync::Arc::new(validator),
                                    ),
                                );
                                std::sync::Arc::new(
                                    elai_tools::adaptive_tool::IpcToolDispatcherAdapter::new(
                                        inner,
                                        wiring.ssb.clone(),
                                    ),
                                )
                            };
                            // patch-R5 — wire the canonical baby-step system
                            // prompt so Sonnet emits ToolUse blocks instead of
                            // text-only replies (otherwise the agent's
                            // run_react_turn parses zero TOOL_USE markers,
                            // exits, and writes no files).
                            std::sync::Arc::new(
                                elai_orchestrator::ProductionAgentRpcServer::new(
                                    client, dispatcher, model, max_tokens,
                                )
                                .with_system_prompt(
                                    elai_orchestrator::production_agent_rpc::DEFAULT_BABY_STEP_SYSTEM_PROMPT,
                                ),
                            )
                        }
                        Err(e) => {
                            tracing::warn!(
                                error = %e,
                                "elai run: cloud credentials unavailable; \
                                 using NoOpAgentRpcServer"
                            );
                            std::sync::Arc::new(elai_orchestrator::NoOpAgentRpcServer)
                        }
                    }
                } else {
                    // Sidecar path (default).
                    let base_url = std::env::var("ANTHROPIC_BASE_URL")
                        .unwrap_or_else(|_| "http://localhost:8100".to_string());
                    let client = elai_providers::AnthropicClient::new("local-dummy")
                        .with_base_url(base_url);
                    let model = std::env::var("ELAI_MODEL")
                        .unwrap_or_else(|_| "claude-sonnet-4-6".to_string());
                    let max_tokens = std::env::var("ELAI_MAX_TOKENS")
                        .ok()
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(4096);
                    // Build a real InProcessToolDispatcher via elai-tools.
                    // NoOpIntegrationCommandRunner returns honest "not wired" failures
                    // for the integration tier; the 4 other dispatch methods are real.
                    let dispatcher: std::sync::Arc<
                        dyn elai_core::tool_dispatcher::ToolDispatcher,
                    > = {
                        struct NoOpIntegrationRunner;
                        impl elai_tools::tiered_validation::IntegrationCommandRunner
                            for NoOpIntegrationRunner
                        {
                            fn run_integration<'a>(
                                &'a self,
                                _level: &'a str,
                                _dir: &'a std::path::Path,
                            ) -> std::pin::Pin<
                                Box<
                                    dyn std::future::Future<
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
                                            "integration tier not wired \
                                             (NoOpIntegrationRunner in CLI sidecar path)"
                                                .to_string(),
                                        ),
                                    }
                                })
                            }
                        }
                        let validator = elai_tools::dispatcher::default_validator(
                            std::sync::Arc::new(NoOpIntegrationRunner),
                            "rust",
                        );
                        // C-13 — wrap the real InProcessToolDispatcher in the
                        // IpcToolDispatcherAdapter shim (sidecar arm). See
                        // workflow/reasoning/C-13-decisions.md (D1 Option B).
                        let inner: std::sync::Arc<
                            dyn elai_core::tool_dispatcher::ToolDispatcher,
                        > = std::sync::Arc::new(
                            elai_tools::dispatcher::InProcessToolDispatcher::new(
                                std::sync::Arc::new(validator),
                            ),
                        );
                        std::sync::Arc::new(
                            elai_tools::adaptive_tool::IpcToolDispatcherAdapter::new(
                                inner,
                                wiring.ssb.clone(),
                            ),
                        )
                    };
                    // patch-R5 — same canonical baby-step system prompt as
                    // the cloud-credentials arm above (see L154 comment).
                    std::sync::Arc::new(
                        elai_orchestrator::ProductionAgentRpcServer::new(
                            client, dispatcher, model, max_tokens,
                        )
                        .with_system_prompt(
                            elai_orchestrator::production_agent_rpc::DEFAULT_BABY_STEP_SYSTEM_PROMPT,
                        ),
                    )
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
                    "elai run: production AgentPoolBabyStepExecutor installed"
                );
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "elai run: agent binary not found; using synthetic fallback executor"
                );
            }
        }
    }

    // C-13 — install the production hook dispatcher BEFORE wiring is
    // applied so apply_task_runner_wiring threads it onto
    // TaskRunnerConfig.hook_dispatcher (Tier-1 surface). See
    // workflow/reasoning/C-13-decisions.md (DECISION D3 option 3A).
    wiring.hook_dispatcher =
        Some(elai_orchestrator::runtime_setup::build_production_hook_dispatcher());
    tracing::info!(
        "elai run: production InProcessHookDispatcher with RuffPostToolUseRunner installed"
    );

    let ssb_writer: Arc<elai_orchestrator::Ssb> = wiring.ssb.clone();
    let cfg = apply_task_runner_wiring(cursor_path, &wiring);

    let (mut handle, cancel) = start_task_runner(ssb_writer, cfg);

    tracing::info!("elai run: task_runner: subscribed to SSB, awaiting tasks");

    // Race the runner future against Ctrl-C. The `tokio::signal::ctrl_c`
    // future resolves when SIGINT (Unix) or CTRL_C_EVENT (Windows) is
    // delivered to the process. Borrow `handle` mutably in the first arm
    // so the second arm can still await it during graceful shutdown.
    tokio::select! {
        runner_result = &mut handle => {
            match runner_result {
                Ok(Ok(())) => {
                    tracing::info!("elai run: task_runner: exited cleanly");
                    Ok(())
                }
                Ok(Err(runner_err)) => {
                    Err(anyhow::anyhow!("elai run: task_runner: {runner_err}"))
                }
                Err(join_err) => {
                    Err(anyhow::anyhow!(
                        "elai run: task_runner: join failed: {join_err}"
                    ))
                }
            }
        }
        ctrl_c = tokio::signal::ctrl_c() => {
            if let Err(e) = ctrl_c {
                tracing::warn!(error = %e, "elai run: ctrl_c handler error; cancelling anyway");
            }
            tracing::info!("elai run: SIGINT received, cancelling task_runner");
            cancel.cancel();
            match tokio::time::timeout(std::time::Duration::from_secs(5), handle).await {
                Ok(Ok(Ok(()))) => {
                    tracing::info!("elai run: task_runner: graceful shutdown complete");
                }
                Ok(Ok(Err(runner_err))) => {
                    tracing::warn!(error = %runner_err, "elai run: task_runner: error during shutdown");
                }
                Ok(Err(join_err)) => {
                    tracing::warn!(error = %join_err, "elai run: task_runner: join error during shutdown");
                }
                Err(_) => {
                    tracing::warn!(
                        "elai run: task_runner: shutdown timed out after 5s; dropping handle"
                    );
                }
            }
            Ok(())
        }
    }
}
