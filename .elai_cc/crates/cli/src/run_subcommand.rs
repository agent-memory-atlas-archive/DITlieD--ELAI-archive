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

use elai_orchestrator::start_task_runner;

use crate::runtime_builder::{
    apply_task_runner_wiring, build_task_runner_wiring, install_buffer_editor,
    install_production_agent_pool, CliInteractiveGate,
};

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
    /// G-2 -- when `--emit-rra` is set, force-write a Reproducible-Run Artifact
    /// (`.rra`) bundle at run end to the session's `run.rra` (or to the path
    /// given by `--emit-rra <path>`). The RRA is emitted by default at run end
    /// (the kill-switch is `ELAI_RRA_DISABLE`); this flag additionally lets the
    /// operator pin a custom output path. `None` => default
    /// `<sessions>/<sid>/run.rra`.
    pub emit_rra: Option<PathBuf>,
    /// I-42 — `--profile orchestrate|critique-only`. Resolved to a
    /// [`elai_orchestrator::RunProfile`] via `RunProfile::from_cli_args` at
    /// the wiring site; `critique-only` structurally refuses the executor/
    /// fixer spawn. `None` => default orchestrate. Unknown values are
    /// rejected at parse time in [`crate::args::parse_run_subcommand_args`].
    pub profile: Option<String>,
    /// I-44 — `--zero-retention`. When set (AND `ELAI_ZERO_RETENTION_ENABLE` is
    /// truthy) the run purges its own raw managed-transcript files after the
    /// sealed verdict commits, via
    /// [`elai_orchestrator::retention::purge_after_verify`] at the seal seam.
    /// `false` (default) retains everything (fail-safe). Destructive when on.
    pub zero_retention: bool,
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

    // I-42 — resolve the run profile from the parsed `--profile` flag.
    // `RunProfile::from_cli_args(None)` returns Orchestrate so omitting the
    // flag preserves the autonomous-orchestrate default. With this assignment
    // the `apply_task_runner_wiring` chokepoint forces
    // `baby_step_executor = None` when the resolved profile is CritiqueOnly,
    // structurally refusing the executor/fixer spawn (one gate, both agents).
    wiring.run_profile = Some(elai_orchestrator::RunProfile::from_cli_args(
        args.profile.as_deref(),
    ));
    tracing::info!(
        run_profile = ?wiring.run_profile,
        "elai run: run profile resolved"
    );

    // B-6 #2: install CliInteractiveGate in production when the resolved
    // gate_mode is Interactive. This is the SINGLE production constructor
    // call for the struct defined at cli/src/runtime_builder.rs:393 + impl
    // elai_orchestrator::InteractiveGate. apply_task_runner_wiring passes
    // this handle as the second argument to AutonomousGate::new so the
    // MODIFY / REDIRECT mixed-mode fallback path can reach the live CLI
    // prompter without rebuilding the gate. Closes the cross-check finding
    // "CliInteractiveGate defined but NEVER instantiated in production"
    // (workflow/sessions/s-night-2026-05-30/cross-check/B/B-02_b4_b5_b6.md
    // B-6 row "InteractiveGate actually instantiated UNMET"). Autonomous
    // mode skips the construction because the gate is reserved for the
    // human-prompt path -- AutonomousGate makes its own decisions.
    if matches!(wiring.gate_mode, Some(elai_orchestrator::GateMode::Interactive)) {
        let cli_gate: Arc<dyn elai_orchestrator::InteractiveGate> =
            Arc::new(CliInteractiveGate);
        wiring.interactive_gate = Some(cli_gate);
        tracing::info!(
            "elai run: CliInteractiveGate installed on RuntimeWiring \
             (gate_mode=Interactive); B-6 #2 wiring closed"
        );
    }

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
        // C-47b CLI-arm: build the long-lived shared ToolCalibrationRegistry
        // with the cold-tier persist store when
        // ELAI_C47B_CASE_BASE_PERSIST_ENABLE is truthy. This mirrors the
        // dashboard arm verbatim (dashboard-v2/src-tauri/src/state.rs:597-634)
        // so CLI tool calls record + persist calibration into the SAME
        // case_base.sqlite3 the dashboard reads. Fail-open: if
        // CasePersistStore::open errors (disk permission, path missing,
        // sqlite locked) we log warn and fall back to in-memory-only (no
        // panic). When the gate is off the registry is None and
        // McpClientDispatch::dispatch_tool keeps the exact http_call_tool
        // path (zero behaviour change, wire-dark by default).
        #[cfg(feature = "mcp")]
        let calibration_registry: Option<Arc<elai_plugins::ToolCalibrationRegistry>> = {
            let gate = std::env::var("ELAI_C47B_CASE_BASE_PERSIST_ENABLE")
                .map(|v| {
                    let t = v.trim().to_ascii_lowercase();
                    t == "1" || t == "true" || t == "yes" || t == "on"
                })
                .unwrap_or(false);
            if gate {
                let persist_path = project_root
                    .join(".elai_cc")
                    .join("data")
                    .join("case_base")
                    .join("case_base.sqlite3");
                let reg = match elai_plugins::CasePersistStore::open(&persist_path) {
                    Ok(store) => {
                        tracing::info!(
                            path = %persist_path.display(),
                            "C-47b CLI-arm: CasePersistStore opened; calibration registry wired with persist"
                        );
                        elai_plugins::ToolCalibrationRegistry::new()
                            .with_persist(Arc::new(store))
                    }
                    Err(e) => {
                        tracing::warn!(
                            path = %persist_path.display(),
                            error = %e,
                            "C-47b CLI-arm: CasePersistStore::open failed; calibration registry in-memory-only (fail-open)"
                        );
                        elai_plugins::ToolCalibrationRegistry::new()
                    }
                };
                // Warm-start the hot LRU from the cold tier so the first
                // CLI dispatch consults persisted history. Non-fatal:
                // rehydrate Err (or no persist store) is logged + ignored,
                // mirroring the dashboard bootstrap spawn (lib.rs:366-382).
                match reg.rehydrate_all_from_persist().await {
                    Ok(n) => tracing::info!(
                        rehydrated = n,
                        "C-47b CLI-arm: rehydrated calibration cases from persist"
                    ),
                    Err(e) => tracing::warn!(
                        error = %e,
                        "C-47b CLI-arm: rehydrate_all_from_persist failed (non-fatal)"
                    ),
                }
                Some(Arc::new(reg))
            } else {
                None
            }
        };
        let mcp = elai_orchestrator::dispatch::McpClientDispatch::new(
            mcp_dispatch_port,
            wiring.ssb.clone(),
        );
        // C-47: wire the SSB emitter so tool_calibration_* rows land in the
        // shared SSB store on every MCP dispatch. The emitter is constructed
        // unconditionally (no extra env gate): when the broader case-base
        // gate is off the bridge itself is a noop for calibration logic, but
        // the SSB adapter rows still fire so the dashboard can observe the
        // dispatch surface. Mirror of the `OrchestratorToolGateSsbEmitter`
        // wiring in commands_c47.rs.
        #[cfg(feature = "mcp")]
        let mcp = mcp.with_calibration_ssb_emitter(std::sync::Arc::new(
            elai_orchestrator::OrchestratorCaseBaseSsbEmitter::new(wiring.ssb.clone()),
        ));
        #[cfg(feature = "mcp")]
        let mcp = match calibration_registry {
            Some(reg) => mcp.with_calibration_registry(reg),
            None => mcp,
        };
        // F-3p1 / Slice C3 (ENFORCE-B): attach a CapabilityFlowAdapter bound to
        // the SAME process-global committed-tool-set slot the producer commits
        // into, so every dispatch_tool call consults the committed set. Default-
        // ON: no env-gate; pass-through until the producer commits.
        #[cfg(feature = "mcp")]
        let mcp = mcp.with_capability_flow_adapter(std::sync::Arc::new(
            elai_orchestrator::CapabilityFlowAdapter::from_slot(
                elai_orchestrator::committed_slot(),
            ),
        ));
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

    // patch-1.14 — install the production agent pool via the shared helper
    // in runtime_builder. mirrors the prior inline block byte-for-byte; the
    // helper logs the same warn messages on soft failures (ipc dir create,
    // agent binary lookup) so the synthetic fallback semantics are preserved.
    install_production_agent_pool(&mut wiring, &project_root, &session_id).await?;

    // B-28 WIRING-2 (item E2-DIFF) — install the production buffer editor so the
    // focus-tunnel DIFF branch applies hunks through the real 3-tier matcher
    // instead of falling back to STRUCTURED. Independent of the agent-pool path
    // above (DIFF must work even when the agent binary is absent). The slot is
    // threaded onto TaskRunnerConfig by apply_task_runner_wiring below.
    install_buffer_editor(&mut wiring);

    // C-13 — install the production hook dispatcher BEFORE wiring is
    // applied so apply_task_runner_wiring threads it onto
    // TaskRunnerConfig.hook_dispatcher (Tier-1 surface). See
    // workflow/reasoning/C-13-decisions.md (DECISION D3 option 3A).
    // F-9 / Slice D3 — project_root threaded in so the gate can resolve
    // the registry_check.py path via SubprocessRegistryChecker::default_at_root.
    // H-38 BLOCK-1: thread interactive_gate into the stop-cost confirm path so
    // CLI Interactive mode can present the confirm prompt on high-spend stops.
    // Autonomous / headless callers have interactive_gate=None; the cost gate
    // still fires and defaults to HOLD (fail-closed).
    wiring.hook_dispatcher =
        Some(elai_orchestrator::runtime_setup::build_production_hook_dispatcher(
            wiring.ssb.clone(),
            wiring.vibegate_seed_ok,
            &project_root,
            wiring.interactive_gate.clone(),
        ));
    tracing::info!(
        "elai run: production InProcessHookDispatcher with RuffPostToolUseRunner + VibeGate SSB + DepWriteTimeGate + StopCostGate installed"
    );

    let ssb_writer: Arc<elai_orchestrator::Ssb> = wiring.ssb.clone();
    // G-2: retain a clone of the live run handle for the RRA seal at run end
    // (DD2: the same Arc<Ssb> reads the stream AND writes the beacon row;
    // start_task_runner consumes `ssb_writer` below).
    let rra_ssb: Arc<elai_orchestrator::Ssb> = ssb_writer.clone();
    let mut cfg = apply_task_runner_wiring(cursor_path, &wiring);
    // E-46 D-8: thread the `[planning.*]` TOML stanzas (already parsed into
    // `elai_config.planning` above) onto `TaskRunnerConfig::planning_config`
    // so the orchestrator uses operator-supplied thresholds at runtime instead
    // of the compile-time module constants. `elai_config` was loaded above for
    // the `[cloud]` section; reuse the same parse result here (zero extra I/O).
    // Default-safe: `PlanningConfig::default()` matches the module constants, so
    // an absent `[planning]` stanza in elai_cc.toml has zero behavioural effect.
    cfg.planning_config = elai_config.planning.clone();
    tracing::info!(
        ledger_reset_blocker_threshold = cfg.planning_config.ledger.ledger_reset_blocker_threshold,
        entropy_ratio_threshold = cfg.planning_config.entropy.entropy_ratio_threshold,
        jaccard_similarity_threshold = cfg.planning_config.goal_drift.jaccard_similarity_threshold,
        "elai run: E-46 D-8 planning_config threaded from elai_cc.toml [planning.*]"
    );

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

    // G-2 -- seal the Reproducible-Run Artifact (.rra) at run end, using the
    // SAME live `ssb_writer` handle that wrote the rest of the stream (DD2: the
    // emit + the run_artifact_emitted row MUST share the run's Arc<Ssb>; a fresh
    // Ssb::new would bind a different session_id and split the stream). Default-
    // ON; suppressed only by the ELAI_RRA_DISABLE kill-switch. Best-effort: an
    // RRA failure is logged and never masks the runner outcome.
    if !rra_disabled() {
        let rra_path = args.emit_rra.clone().unwrap_or_else(|| {
            project_root
                .join(elai_core::paths::ELAI_DATA_DIR_NAME)
                .join("sessions")
                .join(&session_id)
                .join("run.rra")
        });
        if let Err(e) = emit_run_artifact(&rra_ssb, &session_id, &rra_path).await {
            tracing::warn!(error = %e, "elai run: G-2 RRA emit failed (best-effort); run outcome unaffected");
        }
    } else {
        tracing::info!("elai run: G-2 RRA emit suppressed by ELAI_RRA_DISABLE kill-switch");
    }

    // G-8 -- build + sign + emit the in-toto provenance attestation at run end,
    // ALONGSIDE the G-2 RRA emit, using the SAME live `rra_ssb` handle (DD4: the
    // attestation row MUST share the run's Arc<Ssb> + read the SAME stream the
    // RRA does; a fresh Ssb::new would split the session_id). The attestation
    // re-expresses the G-2 RunArtifact manifest as a signed in-toto Statement +
    // writes the sidecar attestation.intoto.json + emits the
    // provenance_attestation_emitted beacon. Default-ON; suppressed only by the
    // ELAI_ATTESTATION_DISABLE kill-switch. Best-effort: a failure is logged and
    // never masks the runner outcome.
    if !attestation_disabled() {
        let attestation_path = project_root
            .join(elai_core::paths::ELAI_DATA_DIR_NAME)
            .join("sessions")
            .join(&session_id)
            .join("attestation.intoto.json");
        if let Err(e) = emit_attestation(&rra_ssb, &session_id, &attestation_path).await {
            tracing::warn!(error = %e, "elai run: G-8 attestation emit failed (best-effort); run outcome unaffected");
        }
    } else {
        tracing::info!(
            "elai run: G-8 attestation emit suppressed by ELAI_ATTESTATION_DISABLE kill-switch"
        );
    }

    // I-44 -- zero-retention-after-verification purge. Runs AFTER the G-2 RRA +
    // G-8 attestation seal (the verdict + evidence are committed and survive),
    // using the SAME live `rra_ssb` handle so the RetentionPurged rows land on
    // the run's stream. FAIL-SAFE: the purge fires ONLY when the operator passed
    // `--zero-retention` (resolved to ZeroAfterVerify) AND the default-OFF
    // `ELAI_ZERO_RETENTION_ENABLE` master kill-switch is truthy (checked inside
    // purge_after_verify). In the default Full mode this performs ZERO filesystem
    // mutation. Best-effort: a purge failure is logged and never masks the runner
    // outcome. `now_ms` is the injected OsClock wall-clock at the entrypoint
    // boundary (the lib path takes it as a parameter; never calls SystemTime).
    {
        use elai_core::clock::ClockProvider as _;
        let mode = elai_orchestrator::retention::ExportRetentionMode::from_cli_flag(
            args.zero_retention,
        );
        let now_ms = elai_core::clock::OsClock.now_ms();
        if let Err(e) = elai_orchestrator::retention::purge_after_verify(
            &rra_ssb,
            &session_id,
            &project_root,
            mode,
            now_ms,
        )
        .await
        {
            tracing::warn!(
                error = %e,
                "elai run: I-44 zero-retention purge failed (best-effort); run outcome unaffected"
            );
        }
    }

    // G-10 -- run the exogenous model-version behavioral-regression check at
    // run end, ALONGSIDE the G-2 RRA emit + G-8 attestation, using the SAME
    // live `rra_ssb` handle (DD4: the row MUST share the run's Arc<Ssb>). On
    // a detected identity change (fingerprint/model differs from the last-good
    // baseline) re-runs the canary and emits ModelDriftSuspected on regression.
    // Default-ON; suppressed only by the ELAI_MODEL_DRIFT_DISABLE kill-switch.
    // Best-effort: a failure is logged and never masks the runner outcome. See
    // workflow/reasoning/G-10-decisions.md (DD4: NOT runtime_setup.rs).
    if !model_drift_disabled() {
        if let Err(e) = emit_model_drift_check(&rra_ssb, &session_id, &project_root).await {
            tracing::warn!(
                error = %e,
                "elai run: G-10 model-drift check failed (best-effort); run outcome unaffected"
            );
        }
    } else {
        tracing::info!(
            "elai run: G-10 model-drift check suppressed by ELAI_MODEL_DRIFT_DISABLE kill-switch"
        );
    }

    outcome
}

/// G-2 kill-switch check. The RRA emit is DEFAULT-ON; the only opt-out is the
/// `ELAI_RRA_DISABLE` kill-switch (truthy value). There is deliberately NO
/// default-OFF enable flag (W-2: default-off is not a terminal state; the
/// deterministic check asserts zero enable-flag references for this feature).
fn rra_disabled() -> bool {
    env_truthy("ELAI_RRA_DISABLE")
}

/// G-8 kill-switch check. The attestation emit is DEFAULT-ON; the only opt-out
/// is the `ELAI_ATTESTATION_DISABLE` kill-switch (truthy value). There is
/// deliberately NO default-OFF enable flag (W-2 / R9: default-off is not a
/// terminal state; the deterministic check asserts zero enable-flag references
/// for this feature -- the kill-switch above is the sole control).
fn attestation_disabled() -> bool {
    env_truthy("ELAI_ATTESTATION_DISABLE")
}

/// Shared truthy-env check for the G-2 / G-8 DEFAULT-ON kill-switches: a var set
/// to `1` / `true` / `yes` / `on` (case-insensitive) disables the feature.
fn env_truthy(var: &str) -> bool {
    match std::env::var(var) {
        Ok(raw) => {
            let v = raw.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes" || v == "on"
        }
        Err(_) => false,
    }
}

/// G-10 kill-switch check. The model-drift check is DEFAULT-ON; the only
/// opt-out is the `ELAI_MODEL_DRIFT_DISABLE` kill-switch (truthy value).
/// There is deliberately NO default-OFF enable flag (W-2: default-off is not
/// a terminal state; the deterministic check asserts zero enable-flag
/// references for this feature — the kill-switch above is the sole control).
fn model_drift_disabled() -> bool {
    env_truthy("ELAI_MODEL_DRIFT_DISABLE")
}

/// G-10 -- run the exogenous model-version behavioral-regression check at run
/// end. Reads the full ordered stream via the LIVE `ssb` handle (same handle
/// G-2 + G-8 use), resolves the identity tuple from the DeterminismConfig
/// pins + the model_swap stream, reads the last-good baseline file (if any),
/// re-runs the canary fixture set, and emits ONE `model_drift_suspected` row
/// on regression. Best-effort: on any failure logs a warn and returns Err.
async fn emit_model_drift_check(
    ssb: &Arc<elai_orchestrator::Ssb>,
    session_id: &str,
    project_root: &std::path::Path,
) -> Result<(), anyhow::Error> {
    use elai_orchestrator::{
        canary::{CanaryFamily, CanaryFamilyBundle, MarkerEvaluator, DEFAULT_MIN_PER_FAMILY},
        held_out::{ExpectedOutcome, HeldOutTask, TaskId, TaskSource},
        model_drift::{
            check_model_drift, record_baseline, now_ms, DriftSignal, DriftVerdict, Lane, ModelIdentity,
        },
        ssb::{DriftSignalKind, ModelDriftSuspectedPayload, SsbPayload},
    };
    use elai_orchestrator::ssb::SsbReader as _;

    // 1. Read the session stream to derive the identity tuple.
    let stream = ssb
        .query_session_stream_asc(session_id)
        .map_err(|e| anyhow::anyhow!("G-10: reading session stream: {e}"))?;

    let pins = elai_orchestrator::determinism::DeterminismConfig::from_env()
        .map_err(|e| anyhow::anyhow!("G-10: resolving determinism pins: {e}"))?;

    // build_rra derives model_pin + system_fingerprint_pin + has_cloud from stream.
    let artifact = elai_orchestrator::rra::build_rra(session_id, &stream, &pins);

    let identity = ModelIdentity::from_rra_fields(
        artifact.model_pin.as_deref(),
        artifact.system_fingerprint_pin.as_deref(),
        // has_cloud heuristic: GuaranteeLevel reflects the cloud/local split.
        matches!(
            artifact.guarantee_level,
            elai_orchestrator::rra::GuaranteeLevel::VerifyDivergence { .. }
        ),
    );

    // 2. Build fixture canary bundles (same as CLI baseline/check commands).
    let bundles: Vec<CanaryFamilyBundle> = CanaryFamily::all()
        .iter()
        .map(|f| {
            let tasks: Vec<HeldOutTask> = (0..DEFAULT_MIN_PER_FAMILY)
                .map(|i| HeldOutTask {
                    task_id: TaskId(format!("pass_g10_{}_{}", f.as_str(), i)),
                    source: TaskSource::Synthetic { seed: i as u64 },
                    content_blake3: {
                        let h = blake3::hash(
                            format!("g10_fixture_{}_{}", f.as_str(), i).as_bytes(),
                        );
                        format!("{:.16}", h.to_hex())
                    },
                    expected_outcome: ExpectedOutcome::Code("ok".to_owned()),
                    ingested_ts_ms: 0,
                    eligibility_window_ms: 0,
                    ngram_signature: vec![],
                    body: format!("g10 fixture task {} {}", f.as_str(), i),
                    template_id: None,
                })
                .collect();
            CanaryFamilyBundle { family: *f, tasks }
        })
        .collect();

    let evaluator = MarkerEvaluator {
        pass_marker: "pass_g10".to_owned(),
    };

    // 3. Try to load the last-good baseline. If absent, capture one and skip emit.
    let baseline_dir = project_root
        .join("benchmarks")
        .join("model_drift");
    let latest_baseline = std::fs::read_dir(&baseline_dir)
        .ok()
        .and_then(|entries| {
            let mut files: Vec<(std::time::SystemTime, std::path::PathBuf)> = entries
                .filter_map(|e| {
                    let e = e.ok()?;
                    let p = e.path();
                    if p.extension().and_then(|s| s.to_str()) == Some("json") {
                        let mt = e.metadata().ok()?.modified().ok()?;
                        Some((mt, p))
                    } else {
                        None
                    }
                })
                .collect();
            files.sort_by(|a, b| b.0.cmp(&a.0));
            files.into_iter().next().map(|(_, p)| p)
        });

    let baseline = match latest_baseline {
        Some(path) => {
            let raw = std::fs::read_to_string(&path)
                .map_err(|e| anyhow::anyhow!("G-10: read baseline {}: {e}", path.display()))?;
            serde_json::from_str::<elai_orchestrator::model_drift::DriftBaseline>(&raw)
                .map_err(|e| anyhow::anyhow!("G-10: parse baseline: {e}"))?
        }
        None => {
            // No baseline yet — capture one and skip drift check this run.
            tracing::info!(
                identity_key = %identity.identity_key(),
                "G-10: no baseline found; capturing baseline for current identity (no drift check this run)"
            );
            let ts = now_ms();
            let baseline = record_baseline(&identity, &bundles, &evaluator, ts)
                .await
                .map_err(|e| anyhow::anyhow!("G-10: record_baseline: {e}"))?;
            // Write baseline file.
            let _ = std::fs::create_dir_all(&baseline_dir);
            let key = identity.identity_key();
            let prefix = &key[..key.len().min(16)];
            let path = baseline_dir.join(format!("baseline-{prefix}.json"));
            if let Ok(json) = serde_json::to_string_pretty(&baseline) {
                let _ = std::fs::write(&path, json);
            }
            return Ok(());
        }
    };

    // 4. Re-run the canary and check for drift.
    let ts = now_ms();
    let verdict = check_model_drift(&identity, &baseline, &bundles, &evaluator, ts)
        .await
        .map_err(|e| anyhow::anyhow!("G-10: check_model_drift: {e}"))?;

    match verdict {
        DriftVerdict::NoDrift | DriftVerdict::BaselineCaptured => {
            tracing::debug!(
                identity_key = %identity.identity_key(),
                "G-10: model-drift check: NoDrift"
            );
        }
        DriftVerdict::BehavioralDriftSuspected {
            ref signals,
            ref per_family_delta,
            ref affected_template_ids,
            confirmed_by_fingerprint,
        } => {
            // 5. Emit the off-agent SSB row (R4): the real run entrypoint detected
            //    drift and this row landed through the SAME live Arc<Ssb>.
            let primary_kind = if signals.iter().any(|s| matches!(s, DriftSignal::FingerprintChanged)) {
                DriftSignalKind::FingerprintChanged
            } else if signals.iter().any(|s| matches!(s, DriftSignal::CanaryRegressed)) {
                DriftSignalKind::CanaryRegressed
            } else {
                DriftSignalKind::MalformedOutputSpike
            };

            let payload = ModelDriftSuspectedPayload {
                identity_key: identity.identity_key(),
                kind: primary_kind,
                signals: signals.iter().map(|s| s.label().to_owned()).collect(),
                per_family_delta: per_family_delta
                    .iter()
                    .map(|(f, d)| (f.as_str().to_owned(), *d))
                    .collect(),
                affected_template_ids: affected_template_ids.clone(),
                confirmed_by_fingerprint,
                baseline_captured_at: baseline.captured_at_ms,
            };

            ssb.write_payload(
                "cli:run-model-drift",
                0,
                SsbPayload::ModelDriftSuspected(payload),
                Vec::new(),
            )
            .await
            .map_err(|e| anyhow::anyhow!("G-10: writing model_drift_suspected row: {e}"))?;

            tracing::warn!(
                identity_key = %identity.identity_key(),
                confirmed_by_fingerprint,
                "G-10: BehavioralDriftSuspected emitted -- model_drift_suspected row written"
            );
        }
    }

    Ok(())
}

/// G-2 -- build + write the `.rra` bundle and emit ONE `run_artifact_emitted`
/// SSB row through the LIVE run handle (DD2). Reads the full ordered stream via
/// the same handle, builds the artifact from the determinism pins, writes it
/// uncompressed (DD1), then writes the beacon row carrying the blake3 root.
async fn emit_run_artifact(
    ssb: &Arc<elai_orchestrator::Ssb>,
    session_id: &str,
    rra_path: &std::path::Path,
) -> Result<(), anyhow::Error> {
    use elai_orchestrator::ssb::SsbReader as _;

    // 1. Read the full, ordered, untruncated action stream for this session.
    let stream = ssb
        .query_session_stream_asc(session_id)
        .map_err(|e| anyhow::anyhow!("reading session stream for RRA: {e}"))?;

    // 2. Resolve the determinism pins (seed / hardware class / fingerprint).
    let pins = elai_orchestrator::determinism::DeterminismConfig::from_env()
        .map_err(|e| anyhow::anyhow!("resolving determinism pins for RRA: {e}"))?;

    // 3. Build + write the artifact.
    let artifact = elai_orchestrator::rra::build_rra(session_id, &stream, &pins);
    elai_orchestrator::rra::write_rra(&artifact, rra_path)
        .map_err(|e| anyhow::anyhow!("writing .rra to {}: {e}", rra_path.display()))?;

    // 4. Emit the off-agent beacon row through the SAME live handle (R4): a
    //    real run entrypoint sealed the artifact and this row landed.
    let gate_count = u32::try_from(artifact.manifest.len()).unwrap_or(u32::MAX);
    let payload = elai_orchestrator::ssb::SsbPayload::RunArtifactEmitted(
        elai_orchestrator::RunArtifactEmittedPayload {
            root_blake3: artifact.root_blake3.clone(),
            guarantee_level: artifact.guarantee_level.as_sql_str().to_owned(),
            gate_count,
            seed: artifact.seed,
            session_id: session_id.to_owned(),
            objective_hash: artifact.objective_hash.clone(),
        },
    );
    ssb.write_payload("cli:run-rra", 0, payload, Vec::new())
        .await
        .map_err(|e| anyhow::anyhow!("writing run_artifact_emitted row: {e}"))?;

    // 5. I-45: emit ObjectiveSealed(Sealed) when objective hash is present AND
    //    the kill-switch is not set. Default-ON; set ELAI_OBJECTIVE_SEAL_DISABLE=1
    //    to suppress (no allowlist row required per R9 for default-ON flags).
    let objective_seal_disabled =
        std::env::var("ELAI_OBJECTIVE_SEAL_DISABLE").unwrap_or_default() == "1";
    if !objective_seal_disabled {
        if let Some(ref obj_hash) = artifact.objective_hash {
            let seal_payload = elai_orchestrator::ssb::SsbPayload::ObjectiveSealed(
                elai_orchestrator::ssb::ObjectiveSealedPayload {
                    kind: elai_orchestrator::ssb::ObjectiveSealedKind::Sealed,
                    objective_hash: obj_hash.clone(),
                    current_hash: None,
                    session_id: session_id.to_owned(),
                    ts_ms: std::time::SystemTime::now() // entropy-allow: ssb-objective-seal-ts-ms-cli-entrypoint
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as i64,
                },
            );
            ssb.write_payload("cli:run-objective-seal", 0, seal_payload, Vec::new())
                .await
                .map_err(|e| anyhow::anyhow!("writing objective_sealed row: {e}"))?;
        }
    }

    // 6. I-18: emit RowChainSealed(ChainAdvanced) to record the per-row rolling
    //    blake3 chain beacon in the SSB. This is the PRODUCTION CALLER that
    //    wires chain_prev_hash (computed by build_rra) into the live SSB stream.
    //    The chain_prev_hash field on the artifact is None when this is the first
    //    RRA in the session (GENESIS marker); ChainAdvanced is still emitted so
    //    verifiers have a stable beacon to count and walk.
    let chain_seq = ssb
        .query_session_stream_asc(session_id)
        .map(|s| s.last().map(|e| e.seq).unwrap_or(0))
        .unwrap_or(0);
    let chain_payload = elai_orchestrator::ssb::SsbPayload::RowChainSealed(
        elai_orchestrator::ssb::RowChainSealedPayload {
            kind: elai_orchestrator::ssb::RowChainSealedKind::ChainAdvanced,
            seq: chain_seq,
            session_id: session_id.to_owned(),
            ts_ms: i64::try_from(
                std::time::SystemTime::now() // entropy-allow: ssb-row-chain-seal-ts-ms-cli-entrypoint
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis(),
            )
            .unwrap_or(i64::MAX),
            break_seq: None,
        },
    );
    ssb.write_payload("cli:run-chain-advanced", 0, chain_payload, Vec::new())
        .await
        .map_err(|e| anyhow::anyhow!("writing row_chain_sealed(chain_advanced) row: {e}"))?;

    // I-18 STEP-1: stamp the single-writer rolling chain_prev_hash into the
    // sealing log. This is the PRODUCTION CALLER of Ssb::chain_prev_hash()
    // (R7 production reachability). The rolling hash is maintained by the
    // single-writer on every SSB flush; stamping it here gives the off-agent
    // verifier a deterministic chain link without re-deriving the full stream.
    let rolling_chain_prev_hash = ssb.chain_prev_hash();

    tracing::info!(
        rra_path = %rra_path.display(),
        root_blake3 = %artifact.root_blake3,
        guarantee_level = %artifact.guarantee_level.as_sql_str(),
        gate_count,
        objective_hash = ?artifact.objective_hash,
        chain_prev_hash = ?artifact.chain_prev_hash,
        rolling_chain_prev_hash = ?rolling_chain_prev_hash,
        "elai run: G-2 RRA sealed + run_artifact_emitted + objective_sealed + row_chain_sealed rows written"
    );
    Ok(())
}

/// G-8 -- build + sign + write the in-toto provenance attestation and emit ONE
/// `provenance_attestation_emitted` SSB row through the LIVE run handle (DD4).
/// Reads the SAME full ordered stream the RRA reads via the same handle, builds
/// the G-2 [`RunArtifact`] (GD-1: CONSUMES the F-gate manifest, re-implements
/// none), re-expresses it as a signed in-toto Statement (a LOCAL blake3-Merkle
/// proof; no new dep), writes the sidecar `attestation.intoto.json`, then writes
/// the off-agent beacon row carrying the blake3-Merkle subject digest.
async fn emit_attestation(
    ssb: &Arc<elai_orchestrator::Ssb>,
    session_id: &str,
    attestation_path: &std::path::Path,
) -> Result<(), anyhow::Error> {
    use elai_orchestrator::ssb::SsbReader as _;

    // 1. Read the full, ordered, untruncated action stream for this session
    //    (the SAME stream G-2 reads — shared handle, DD4).
    let stream = ssb
        .query_session_stream_asc(session_id)
        .map_err(|e| anyhow::anyhow!("reading session stream for attestation: {e}"))?;

    // 2. Resolve the determinism pins (seed / hardware class / fingerprint).
    let pins = elai_orchestrator::determinism::DeterminismConfig::from_env()
        .map_err(|e| anyhow::anyhow!("resolving determinism pins for attestation: {e}"))?;

    // 3. Build the G-2 RunArtifact (GD-1: CONSUME the manifest, re-implement no
    //    F-gate), then re-express + sign it as an in-toto Statement.
    let artifact = elai_orchestrator::rra::build_rra(session_id, &stream, &pins);
    let statement = elai_orchestrator::attestation::build_attestation(&artifact)
        .map_err(|e| anyhow::anyhow!("building in-toto attestation: {e}"))?;

    // 4. Write the sidecar attestation.intoto.json next to the run artifacts.
    elai_orchestrator::attestation::write_sidecar(&statement, attestation_path).map_err(|e| {
        anyhow::anyhow!(
            "writing attestation sidecar to {}: {e}",
            attestation_path.display()
        )
    })?;

    // 5. Emit the off-agent beacon row through the SAME live handle (R4): a real
    //    run entrypoint built + signed the attestation and this row landed.
    let subject_blake3 = statement
        .subject_digest()
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("attestation has no subject digest"))?;
    let verified_levels = u32::try_from(statement.predicate.verified_levels.len()).unwrap_or(u32::MAX);
    let all_passed = statement.predicate.verified_levels.iter().all(|v| v.passed);
    let payload = elai_orchestrator::ssb::SsbPayload::ProvenanceAttestationEmitted(
        elai_orchestrator::ProvenanceAttestationEmittedPayload {
            session_id: session_id.to_owned(),
            subject_blake3: subject_blake3.clone(),
            run_artifact_root: artifact.root_blake3.clone(),
            verified_levels,
            all_passed,
            signing_model: ATTESTATION_SIGNING_MODEL.to_owned(),
        },
    );
    ssb.write_payload("cli:run-attestation", 0, payload, Vec::new())
        .await
        .map_err(|e| anyhow::anyhow!("writing provenance_attestation_emitted row: {e}"))?;

    // 6. I-18: emit RowChainSealed(AibomEmitted) after the AI-BOM was emitted
    //    into the attestation sidecar. This is the PRODUCTION CALLER that wires
    //    the AibomEntry (built by build_attestation) into the live SSB stream.
    let aibom_seq = ssb
        .query_session_stream_asc(session_id)
        .map(|s| s.last().map(|e| e.seq).unwrap_or(0))
        .unwrap_or(0);
    let aibom_payload = elai_orchestrator::ssb::SsbPayload::RowChainSealed(
        elai_orchestrator::ssb::RowChainSealedPayload {
            kind: elai_orchestrator::ssb::RowChainSealedKind::AibomEmitted,
            seq: aibom_seq,
            session_id: session_id.to_owned(),
            ts_ms: i64::try_from(
                std::time::SystemTime::now() // entropy-allow: ssb-aibom-emit-ts-ms-cli-entrypoint
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis(),
            )
            .unwrap_or(i64::MAX),
            break_seq: None,
        },
    );
    ssb.write_payload("cli:run-aibom-emitted", 0, aibom_payload, Vec::new())
        .await
        .map_err(|e| anyhow::anyhow!("writing row_chain_sealed(aibom_emitted) row: {e}"))?;

    tracing::info!(
        attestation_path = %attestation_path.display(),
        subject_blake3 = %subject_blake3,
        run_artifact_root = %artifact.root_blake3,
        verified_levels,
        all_passed,
        "elai run: G-8 in-toto attestation signed + provenance_attestation_emitted + aibom_emitted rows written"
    );
    Ok(())
}

/// G-8 -- the honest signing-model label recorded on every emitted attestation
/// beacon (GD-5): the default is a LOCAL blake3-Merkle proof (offline recompute),
/// NOT a Sigstore-Rekor transparency-log witness. The deferred cosign-keyless
/// path (feature `cosign-attest`) would record a different label.
const ATTESTATION_SIGNING_MODEL: &str = "local_blake3_merkle";

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
