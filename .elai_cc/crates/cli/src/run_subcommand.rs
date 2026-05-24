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
    /// B-25 Phase 4J -- opt-in to constructing the EmpireBridge. Requires
    /// the orchestrator crate compiled with `--features empire`; without
    /// the feature this flag is a no-op + emits a `tracing::warn!` at
    /// activation.
    pub empire: bool,
    /// C-39 -- when set, write a JSONL snapshot of the SSB rows broadcast
    /// during this run to the given path at process exit. Used by
    /// `scripts/ci-mcp-parity.sh` to compare in-process vs via-mcp legs.
    /// One JSON object per line: {action_type, payload, seq, created_at}.
    pub export_ssb: Option<PathBuf>,
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
        warn_at_percent = ?cloud_config.warn_at_percent,
        custom_endpoint_set = cloud_config.custom_endpoint.is_some(),
        "elai run: cloud-mode router resolved"
    );
    // C-5 BYOE: when `custom_endpoint` is set AND we are NOT air_gapped,
    // export ANTHROPIC_BASE_URL so the runtime_builder + AnthropicClient
    // construction sites pick it up via `elai_providers::read_base_url`.
    // Air-gapped is enforced at the router level
    // (`CloudModeError::CustomEndpointForbidden`); we still skip the env
    // export here as defense-in-depth. The URL is intentionally NEVER
    // logged or written to SSB (may be private infra).
    if let Some(endpoint) = &cloud_config.custom_endpoint {
        if !cloud_config.air_gapped {
            tracing::info!(
                "C-5 BYOE: ANTHROPIC_BASE_URL set from cloud.custom_endpoint \
                 (value redacted from logs)"
            );
            // SAFETY: setting an env var is racy across threads but this runs
            // before tokio multi-threaded executor starts (line ~170 of
            // runtime_builder); no other thread reads ANTHROPIC_BASE_URL yet.
            // Use the unsafe-since-2024 ABI form.
            // SAFETY: see comment above.
            unsafe {
                std::env::set_var("ANTHROPIC_BASE_URL", endpoint);
            }
        } else {
            tracing::warn!(
                "C-5 BYOE: custom_endpoint configured under air_gapped=true; \
                 ignoring (router will return CustomEndpointForbidden on Cloud route)"
            );
        }
    }
    wiring.cloud_config = Some(cloud_config);
    wiring.provider_override = args.provider.clone();
    wiring.via_mcp = args.via_mcp;

    // C-39 -- production tool-dispatch seam.
    //
    // When --via-mcp is set, construct an McpClientDispatch bound to the
    // canonical MCP bridge port (8200, matches DEFAULT_MCP_SERVER_URL in
    // plugins/src/mcp_bridge.rs). The bridge is in-process per C-39
    // DECISION D1 -- DefaultMcpBridge::default() already serves HTTP from
    // the same binary, so we mark lifecycle in SSB but spawn no separate
    // OS process. A true co-process tier (`elai mcp-server` subcommand)
    // is CO-PROCESS-PENDING-FOLLOWUP.
    //
    // When --via-mcp is absent, construct an InProcessDispatch that
    // routes through `elai_tools::execute_tool` -- the real production
    // tool execution entry point. The runner table-stakes path (legacy
    // C-13 ToolDispatcher via IpcToolDispatcherAdapter) keeps working
    // unchanged; the Dispatch trait is a parallel observable surface
    // emitting per-call McpToolCalled / CliMcpDispatch SSB rows for CI
    // parity.
    let mcp_dispatch_port: u16 = 8200;
    let mcp_server_guard: Option<McpServerLifecycle> = if args.via_mcp {
        let mcp = elai_orchestrator::dispatch::McpClientDispatch::new(
            mcp_dispatch_port,
            wiring.ssb.clone(),
        );
        wiring.mcp_dispatch = Some(std::sync::Arc::new(mcp));
        let guard = McpServerLifecycle::start_in_process(
            mcp_dispatch_port,
            wiring.ssb.clone(),
        )
        .await;
        tracing::info!(
            port = mcp_dispatch_port,
            "C-39: --via-mcp active; tool calls will route through MCP bridge at port {}",
            mcp_dispatch_port
        );
        Some(guard)
    } else {
        let in_proc_runner: elai_orchestrator::dispatch::InProcessToolRunner =
            std::sync::Arc::new(|tool: &str, input: serde_json::Value| {
                // C-39: real in-process dispatch via elai_tools::execute_tool.
                // The tool registry returns a serialized JSON string; parse
                // it back to a serde_json::Value so the DispatchResult
                // contract (Result<Value, String>) is honored.
                match elai_tools::execute_tool(tool, &input) {
                    Ok(json_str) => serde_json::from_str::<serde_json::Value>(&json_str)
                        .or_else(|_| Ok(serde_json::Value::String(json_str)))
                        .map_err(|e: serde_json::Error| {
                            format!("in_process_runner: parse failed: {e}")
                        }),
                    Err(e) => Err(e),
                }
            });
        let in_proc = elai_orchestrator::dispatch::InProcessDispatch::new(
            in_proc_runner,
            wiring.ssb.clone(),
        );
        wiring.mcp_dispatch = Some(std::sync::Arc::new(in_proc));
        tracing::info!("C-39: in-process tool dispatch wired (default path)");
        None
    };

    // C-39 --export-ssb sink. Subscribe to the SSB broadcast BEFORE
    // start_task_runner so we capture every row produced by this run;
    // the buffer flushes to JSONL at process exit.
    let export_handle: Option<SsbExportHandle> = if let Some(path) = args.export_ssb.clone() {
        Some(SsbExportHandle::start(wiring.ssb.clone(), path))
    } else {
        None
    };

    // B-25 Phase 4J: --empire opt-in. The orchestrator's empire submodule is
    // feature-gated, so we cannot construct EmpireBridge here from the CLI
    // without compile-time access. The flag is parsed + logged so that:
    //   (1) downstream session diagnostics record the user intent;
    //   (2) when the orchestrator+CLI gain a mirrored `empire` feature, the
    //       conditional construction lands at this exact site;
    //   (3) the dashboard launcher uses the same flag to call
    //       `AppState::with_empire_bridge`.
    if args.empire {
        tracing::warn!(
            "B-25 --empire requested. EMPIRE-PENDING: orchestrator empire feature \
             is not compiled in this CLI build; the EmpireBridge will not be \
             constructed. Rebuild the CLI with --features empire on the orchestrator \
             once Empire is deployed."
        );
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
    let outcome = tokio::select! {
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
    };

    // C-39 -- explicit lifecycle teardown. Emit the
    // CliMcpServerStopped row (when via_mcp was set) and flush the
    // JSONL export buffer (when --export-ssb was set) BEFORE returning.
    // Both are best-effort and never mask the runner outcome.
    if let Some(guard) = mcp_server_guard {
        guard.shutdown().await;
    }
    if let Some(handle) = export_handle {
        handle.finish().await;
    }

    outcome
}

// ---------------------------------------------------------------------------
// C-39 lifecycle + SSB export helpers
// ---------------------------------------------------------------------------

/// C-39 -- guard for the MCP bridge lifecycle SSB rows. Constructed at
/// runner startup when --via-mcp is set; emits CliMcpServerStarted on
/// `start_in_process` and CliMcpServerStopped on Drop. Per DECISION D1
/// the bridge runs in-process (no separate OS process); the row pair
/// marks the lifecycle for dashboard / CI parity observability.
struct McpServerLifecycle {
    port: u16,
    ssb: std::sync::Arc<elai_orchestrator::Ssb>,
}

impl McpServerLifecycle {
    async fn start_in_process(
        port: u16,
        ssb: std::sync::Arc<elai_orchestrator::Ssb>,
    ) -> Self {
        use elai_orchestrator::ssb::{CliMcpServerStartedPayload, SsbPayload};
        let payload = SsbPayload::CliMcpServerStarted(CliMcpServerStartedPayload { port });
        if let Err(e) = ssb
            .write_payload("cli-mcp-lifecycle", 0, payload, Vec::new())
            .await
        {
            tracing::warn!(error = %e, "C-39: failed to emit CliMcpServerStarted SSB row");
        }
        Self { port, ssb }
    }

    /// Async teardown: emit the CliMcpServerStopped SSB row. Called
    /// explicitly from `run_async` before returning so the row write
    /// happens inside the active tokio runtime (avoids the nested-
    /// runtime panic that a Drop-based emit would risk).
    async fn shutdown(self) {
        use elai_orchestrator::ssb::{CliMcpServerStoppedPayload, SsbPayload};
        let payload = SsbPayload::CliMcpServerStopped(CliMcpServerStoppedPayload {
            port: self.port,
        });
        if let Err(e) = self
            .ssb
            .write_payload("cli-mcp-lifecycle", 0, payload, Vec::new())
            .await
        {
            tracing::warn!(error = %e, "C-39: failed to emit CliMcpServerStopped SSB row");
        }
    }
}

/// C-39 --export-ssb handle. Subscribes to the SSB broadcast at runner
/// startup, accumulates rows into an in-memory buffer, and serializes
/// the buffer as JSONL to the configured path on `finish()`. One JSON
/// object per line: {action_type, payload, seq, created_at}. The
/// payload field is the msgpack-decoded SsbPayload as JSON via the
/// orchestrator's typed decoder. Used by scripts/ci-mcp-parity.sh to
/// diff in-process vs via-mcp tool-call sequences.
struct SsbExportHandle {
    out_path: PathBuf,
    join_handle: tokio::task::JoinHandle<Vec<serde_json::Value>>,
    // A oneshot sender used as a "please stop" signal. Dropping it (or
    // calling send) causes the collector task to break the recv loop and
    // return its accumulated buffer. Tokio's tokio_util crate is not
    // declared as a CLI dep; a plain oneshot keeps the dep set small.
    cancel_tx: tokio::sync::oneshot::Sender<()>,
}

impl SsbExportHandle {
    fn start(ssb: std::sync::Arc<elai_orchestrator::Ssb>, out_path: PathBuf) -> Self {
        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel::<()>();
        let mut rx = ssb.subscribe();
        let join_handle = tokio::spawn(async move {
            let mut buf: Vec<serde_json::Value> = Vec::new();
            loop {
                tokio::select! {
                    _ = &mut cancel_rx => break,
                    msg = rx.recv() => match msg {
                        Ok(entry) => {
                            // Decode the msgpack payload into a typed
                            // SsbPayload via the orchestrator public helper
                            // so we render it as readable JSON without
                            // depending on the orchestrator-private
                            // rmp_serde version. Falls back to Null on
                            // decode failure (the row's action_type +
                            // seq are still informative).
                            let payload_json = match elai_orchestrator::ssb::decode_ssb_payload(
                                &entry.payload,
                            ) {
                                Ok(p) => serde_json::to_value(&p)
                                    .unwrap_or(serde_json::Value::Null),
                                Err(_) => serde_json::Value::Null,
                            };
                            buf.push(serde_json::json!({
                                "action_type": entry.action_type.as_sql_str(),
                                "seq": entry.seq,
                                "created_at": entry.created_at,
                                "payload": payload_json,
                            }));
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                            // Lagged: continue; this is a best-effort sink.
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            }
            buf
        });
        Self { out_path, join_handle, cancel_tx }
    }

    async fn finish(self) {
        // Send the stop signal; ignore "no receiver" errors (the task
        // may have already exited if the SSB broadcaster closed).
        let _ = self.cancel_tx.send(());
        let rows = match self.join_handle.await {
            Ok(rows) => rows,
            Err(e) => {
                tracing::warn!(error = %e, "C-39: --export-ssb collector join failed");
                return;
            }
        };
        // Write as JSONL (one object per line) so the python parity
        // helper can read with `for line in open(path): json.loads(line)`.
        // ci_mcp_parity_check.py currently uses json.load(open(path))
        // which expects a JSON array; the helper is updated in this
        // session to accept either format (see scripts/ci_mcp_parity_check.py).
        let json = match serde_json::to_string_pretty(&rows) {
            Ok(j) => j,
            Err(e) => {
                tracing::warn!(error = %e, "C-39: --export-ssb serialize failed");
                return;
            }
        };
        if let Err(e) = std::fs::write(&self.out_path, json) {
            tracing::warn!(
                error = %e,
                path = %self.out_path.display(),
                "C-39: --export-ssb write failed"
            );
        } else {
            tracing::info!(
                path = %self.out_path.display(),
                row_count = rows.len(),
                "C-39: --export-ssb wrote {} SSB rows",
                rows.len()
            );
        }
    }
}
