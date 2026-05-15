//! ELAI dashboard-v2 backend entry point. Constructs the Tauri builder,
//! attaches a single `AppState`, and registers every Tauri command consumed
//! by the Svelte frontend.
//!
//! See `state.rs` for the dashboard's runtime state and `commands.rs` for the
//! command-by-command implementation. We deliberately keep this file thin —
//! it should never grow beyond a builder + invoke_handler list + bootstrap.

mod commands;
mod media_validation;
mod reflect_bridge;
mod remote_sidecar_helpers;
mod scaffold_applier;
mod state;

// Phase A (s-69820): re-export `DashboardScaffoldApplier` so any future
// task-runner construction in CLI/agent can pull it via `elai_dashboard_v2_lib`
// without re-implementing the trait. The applier is defined in
// `scaffold_applier.rs` and wraps `scaffold::apply_template_from_root`.
pub use scaffold_applier::{default_templates_dir, DashboardScaffoldApplier, TEMPLATES_DIR_ENV};

use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialise tracing subscriber once. Errors here mean another subscriber
    // is already global; that's harmless — keep going.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .try_init();

    // FIXME(tokio-runtime): `Ssb::new()` calls `tokio::spawn(write_loop(...))`
    // at orchestrator/src/ssb.rs:2459.  That spawn requires an active Tokio
    // runtime, but we are still inside a plain synchronous `pub fn run()` —
    // Tauri's runtime has not started yet.  We build a dedicated multi-thread
    // runtime here solely to give the spawn a valid reactor context.
    //
    // Why `mem::forget(rt)` instead of dropping it?
    //   The write_loop JoinHandle is stored inside Ssb (bound to this runtime).
    //   Dropping `rt` would shut the runtime down and kill the write_loop task.
    //   We intentionally leak the runtime so it outlives `run()` and survives
    //   for the entire process lifetime.  This is safe: one runtime, one leak,
    //   no resource growth.  If the write_loop task design is ever refactored to
    //   be runtime-agnostic this block can be removed.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime for AppState::new() (write_loop spawn)");
    let app_state = rt.block_on(async { AppState::new() });
    // SAFETY: intentional process-lifetime leak — see comment above.
    std::mem::forget(rt);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // greeting
            commands::greet,
            // FSM / SYSTEM STATUS
            commands::get_fsm_state,
            commands::get_system_status,
            // TASK / INTERVENTION
            commands::submit_task,
            commands::submit_intervention_response,
            commands::respond_to_confirmation,
            commands::submit_credential_response,
            commands::open_project_from_folder,
            commands::get_feasibility_report,
            // GATE
            commands::get_gate_status,
            commands::set_gate_mode,
            commands::get_auto_advance_stats,
            // MODEL ROUTING
            commands::set_model_routing,
            // PROPOSALS / MSI / STEERING / MCP
            commands::get_active_optimizations,
            commands::get_pending_proposals,
            commands::get_applied_proposals,
            commands::apply_proposal_from_user,
            commands::reject_proposal,
            commands::get_msi_status,
            commands::get_steering_status,
            commands::recompute_steering_vector,
            commands::get_mcp_status,
            commands::set_mcp_enabled,
            // KNOWLEDGE
            commands::get_query_trace_recent,
            commands::get_search_mode,
            commands::get_embedding_breakers,
            commands::get_recall_metrics,
            commands::get_dkg_stats,
            // REASONING / GRC
            commands::get_grc_budgets,
            commands::get_grounding_sweep,
            commands::get_csq_pending,
            commands::get_gg_signals,
            // ORCHESTRATION
            commands::get_evidence_gates,
            commands::get_agents,
            commands::get_supervision_log,
            commands::get_context_budget,
            commands::get_bsp_supersteps,
            commands::get_session_branches,
            commands::fork_session_branch,
            commands::merge_session_branch,
            commands::close_session_branch,
            commands::get_anti_spiral_counter,
            commands::get_heartbeats,
            // PIPELINE / DECOMPOSITION
            commands::get_decomposition_tree,
            commands::get_bubble_up_events,
            commands::get_executable_pipeline,
            commands::get_dag_repair_log,
            // RESEARCH
            commands::get_research_tree,
            commands::get_source_quality_tiers,
            // PROVIDERS / INFERENCE
            commands::get_vram,
            commands::get_inference_metrics,
            commands::get_sidecar_status,
            // RULES
            commands::get_rules_lifecycle,
            commands::get_active_rules,
            commands::get_deprecated_patterns,
            // WEB / BUILD
            commands::get_web_arag_state,
            commands::get_build_slots,
            commands::get_dev_servers,
            // PHASE A — WEB SCAFFOLD + DEV SERVER + FIM (s-69820)
            commands::list_scaffold_templates,
            commands::apply_scaffold_template,
            commands::fim_fill_section,
            commands::run_web_build_check,
            commands::start_dev_server_for_project,
            commands::stop_dev_server_for_project,
            // PHASE B — WEB LINT + WEB PERFORMANCE / LIGHTHOUSE (s-44680)
            commands::run_web_lint_check,
            commands::run_lighthouse_audit,
            commands::probe_chromium_runtime,
            // PROJECT INTELLIGENCE
            commands::get_project_intelligence,
            // MISTAKE / INTERROGATION / FAILURE TAXONOMY
            commands::get_mistake_book,
            commands::get_interrogation_tree,
            commands::get_failure_taxonomy_last,
            // BASELINE / DEP AUDIT
            commands::get_baseline_results,
            commands::get_dep_audit_results,
            // HOOKS
            commands::get_hook_executions,
            commands::get_shortcoder_stats,
            // EXPLORE
            commands::run_explore_query,
            commands::query_knowledge_engine,
            commands::continue_task,
            // SETTINGS
            commands::set_local_max_context,
            commands::get_local_budget,
            commands::set_local_min_prompt,
            commands::get_local_min_prompt,
            commands::set_execution_mode,
            commands::get_execution_mode,
            // CLOUD AUTH (Anthropic OAuth login surface)
            commands::get_oauth_status,
            commands::start_oauth_login,
            commands::do_oauth_logout,
            commands::open_url_in_browser,
            // CLOUD MODEL PICKER
            commands::get_cloud_model,
            commands::set_cloud_model,
            // CLOUD SPEND (B-27)
            commands::get_cloud_spend,
            // WEBHOOK CONFIG
            commands::set_webhook_url,
            commands::get_webhook_url,
            // CHAT
            commands::submit_chat_message,
            commands::get_chat_history,
            commands::emit_chat_response,
            commands::classify_intent,
            commands::get_recent_tasks,
            // GROUNDING CACHE
            commands::get_grounding_cache_stats,
            // SIDECAR HEALTH (D.6)
            commands::get_sidecar_health,
            // STREAMING CHAT (B-14)
            commands::start_chat_stream,
            commands::cancel_chat_stream,
            // D.1 — REINDEX NOW
            commands::reindex_now,
            // D.2 — PLANNING TASK (SSB-actuated)
            commands::run_planning_task,
            commands::cancel_planning_task,
            // D.3 — RESEARCH (direct DefaultWebResearchRunner)
            commands::run_research,
            commands::cancel_research,
            // D.4 — SETTINGS READ/WRITE
            commands::fetch_settings_get,
            commands::fetch_settings_set,
            commands::obscura_test_fetch,
            // D.5 — LIVE CONTRACT TESTS (ELAI_LIVE_SIDECAR=1)
            commands::run_d5_live_contract,
            commands::cancel_d5_live_contract,
            // D.7 — ONE-SHOT PROMPT
            commands::one_shot_prompt,
            // D.8 — TOOL VISIBILITY
            commands::get_tool_visibility,
            // B-18 — SANDBOX TIER STATUS
            commands::get_sandbox_status,
            // D.11 — SIDECAR SETTINGS + RESTART
            commands::get_sidecar_settings,
            commands::set_sidecar_settings,
            commands::restart_sidecar,
            // D.12 — SIDECAR SEARCH
            commands::sidecar_search,
            // REMOTE SIDECAR BOOT OVERLAY
            commands::start_remote_sidecar,
            commands::get_remote_sidecar_progress,
            commands::cancel_remote_sidecar,
            // B-39 — REFLECT STATE
            commands::get_reflect_state,
            // B-28 follow-up — FOCUS TUNNEL INLINE CHIPS (single per-baby-step
            // query; the dedicated FocusTunnelPanel surface was retired).
            commands::query_focus_tunnel_per_baby_step,
            // B-24 Phase 4I Group C8 — MEDIA VALIDATION SURFACE
            media_validation::get_media_validations,
            media_validation::get_media_validation_by_step,
            media_validation::submit_media_validation_for_review,
            media_validation::subscribe_media_ssb_events,
        ])
        .setup(|app| {
            // B-24 Phase 4I Group C8: spawn the SSB subscriber that taps the
            // media_* action types and folds rows into AppState.media_validation.
            // When the writer is None (fresh install, no SSB DB), the subscriber
            // is skipped — Tauri commands still return empty entries and the
            // frontend renders the "no validations yet" empty state.
            use tauri::Manager;
            let state: tauri::State<'_, crate::state::AppState> = app.state();
            if let Some(writer) = state.writer.clone() {
                let media_state = state.media_validation.clone();
                let handle = app.handle().clone();
                media_validation::spawn_subscriber(handle, writer, media_state);

                // Phase A1b: spawn `start_task_runner` from the dashboard,
                // wired with the SAME `RuntimeWiring` chokepoint helpers the
                // `elai run` CLI uses (orchestrator::runtime_setup). Closes
                // the gap recorded at `metrics/phase_4F.toml:103`
                // (`production_construction_paths_wired = false`) on the
                // dashboard side and pairs with Phase A1a's CLI fix.
                //
                // Order of operations:
                // 1. `build_task_runner_wiring` re-uses the SAME canonical
                //    flat SSB path the dashboard reader already opened
                //    (`<project_root>/.elai_cc/sessions/ssb.db`). It opens
                //    its own writer handle inside the bundle because the
                //    Layer1A processor needs an Arc<Ssb> that owns the
                //    project_id/session_id pair used for runner-emitted
                //    rows. The dashboard `state.writer` and the wiring
                //    `wiring.ssb` are two writers on the same DB file with
                //    distinct session_ids — this is the same pattern used
                //    by CLI + dashboard co-existence and is safe under
                //    SQLite WAL mode (ssb.rs sets WAL on open).
                // 2. `apply_task_runner_wiring` produces a TaskRunnerConfig
                //    with both `feasibility_prober` and `layer1a_processor`
                //    populated as Some — chokepoint contract pinned in
                //    `contracts/tests/task_runner_chokepoint_contract.rs`.
                // 3. `start_task_runner` returns (JoinHandle, CancelHandle);
                //    we stash both in AppState for the
                //    `RunEvent::ExitRequested` shutdown closure below.
                let project_root = state.project_root_snapshot();
                let session_id = format!("dash-v2-runner-{}", std::process::id());
                let cursor_path = project_root
                    .join(elai_core::paths::ELAI_DATA_DIR_NAME)
                    .join("sessions")
                    .join(&session_id)
                    .join("cursor.json");
                // `build_task_runner_wiring` is async (opens SQLite + runs
                // Layer1A ensure_tables); the leaked tokio runtime created
                // at `run()` head is the active context, so block_on is
                // safe here. Failure path is warn-and-continue — the
                // dashboard remains usable without a runner loop, matching
                // the existing "writer missing -> skip subscriber" pattern.
                let wiring_result = tauri::async_runtime::block_on(async {
                    elai_orchestrator::build_task_runner_wiring(&project_root, &session_id).await
                });
                match wiring_result {
                    Ok(mut wiring) => {
                        let runner_ssb = wiring.ssb.clone();
                        // Phase A4: publish the Layer1A processor to AppState
                        // BEFORE apply_task_runner_wiring borrows `wiring`, so
                        // apply_proposal_from_user / reject_proposal can
                        // delegate to it. Clone the Arc<DefaultLayer1aProcessor>
                        // and widen to Arc<dyn Layer1aProcessor + Send + Sync>;
                        // the trait already requires Send + Sync
                        // (self_mod_layer1a.rs:317 `trait Layer1aProcessor: Send + Sync`).
                        let layer1a_for_state: std::sync::Arc<
                            dyn elai_orchestrator::Layer1aProcessor + Send + Sync,
                        > = wiring.layer1a_processor.clone();
                        match state.layer1a_processor.lock() {
                            Ok(mut g) => *g = Some(layer1a_for_state),
                            Err(e) => tracing::error!(
                                error = %e,
                                "dashboard-v2: layer1a_processor mutex poisoned; processor handle dropped"
                            ),
                        }
                        // B1: attempt to install the production
                        // AgentPoolBabyStepExecutor. IPC socket dir lives
                        // under the session subtree so artefacts are co-located.
                        let ipc_dir = project_root
                            .join(elai_core::paths::ELAI_DATA_DIR_NAME)
                            .join("sessions")
                            .join(&session_id)
                            .join("ipc");
                        if std::fs::create_dir_all(&ipc_dir).is_ok() {
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
                                    > = if std::env::var("ELAI_GROUNDING_BACKEND")
                                        .as_deref()
                                        == Ok("cloud")
                                    {
                                        match elai_providers::AnthropicClient::from_env() {
                                            Ok(client) => {
                                                let model = std::env::var("ELAI_MODEL")
                                                    .unwrap_or_else(|_| "claude-sonnet-4-6".to_string());
                                                let max_tokens = std::env::var("ELAI_MAX_TOKENS")
                                                    .ok()
                                                    .and_then(|v| v.parse::<u32>().ok())
                                                    .unwrap_or(4096);
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
                                                                         (NoOpIntegrationRunner in dashboard cloud path)"
                                                                            .to_string(),
                                                                    ),
                                                                }
                                                            })
                                                        }
                                                    }
                                                    let validator =
                                                        elai_tools::dispatcher::default_validator(
                                                            std::sync::Arc::new(
                                                                NoOpIntegrationRunner,
                                                            ),
                                                            "rust",
                                                        );
                                                    std::sync::Arc::new(
                                                        elai_tools::dispatcher::InProcessToolDispatcher::new(
                                                            std::sync::Arc::new(validator),
                                                        ),
                                                    )
                                                };
                                                std::sync::Arc::new(
                                                    elai_orchestrator::ProductionAgentRpcServer::new(
                                                        client, dispatcher, model, max_tokens,
                                                    ),
                                                )
                                            }
                                            Err(e) => {
                                                tracing::warn!(
                                                    error = %e,
                                                    "dashboard-v2: cloud credentials unavailable; \
                                                     using NoOpAgentRpcServer"
                                                );
                                                std::sync::Arc::new(
                                                    elai_orchestrator::NoOpAgentRpcServer,
                                                )
                                            }
                                        }
                                    } else {
                                        let base_url = std::env::var("ANTHROPIC_BASE_URL")
                                            .unwrap_or_else(|_| "http://localhost:8100".to_string());
                                        let client =
                                            elai_providers::AnthropicClient::new("local-dummy")
                                                .with_base_url(base_url);
                                        let model = std::env::var("ELAI_MODEL")
                                            .unwrap_or_else(|_| "claude-sonnet-4-6".to_string());
                                        let max_tokens = std::env::var("ELAI_MAX_TOKENS")
                                            .ok()
                                            .and_then(|v| v.parse::<u32>().ok())
                                            .unwrap_or(4096);
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
                                                                 (NoOpIntegrationRunner in dashboard sidecar path)"
                                                                    .to_string(),
                                                            ),
                                                        }
                                                    })
                                                }
                                            }
                                            let validator =
                                                elai_tools::dispatcher::default_validator(
                                                    std::sync::Arc::new(NoOpIntegrationRunner),
                                                    "rust",
                                                );
                                            std::sync::Arc::new(
                                                elai_tools::dispatcher::InProcessToolDispatcher::new(
                                                    std::sync::Arc::new(validator),
                                                ),
                                            )
                                        };
                                        std::sync::Arc::new(
                                            elai_orchestrator::ProductionAgentRpcServer::new(
                                                client, dispatcher, model, max_tokens,
                                            ),
                                        )
                                    };
                                    let executor =
                                        elai_orchestrator::build_default_baby_step_executor(
                                            elai_core::OsEntropy,
                                            agent_bin.clone(),
                                            ipc_dir,
                                            gateway,
                                            rpc_server,
                                        );
                                    wiring.baby_step_executor = Some(executor);
                                    tracing::info!(
                                        agent_bin = %agent_bin.display(),
                                        "dashboard-v2: production AgentPoolBabyStepExecutor installed"
                                    );
                                }
                                Err(e) => tracing::warn!(
                                    error = %e,
                                    "dashboard-v2: agent binary not found; \
                                     using synthetic fallback executor"
                                ),
                            }
                        } else {
                            tracing::warn!(
                                path = %ipc_dir.display(),
                                "dashboard-v2: failed to create IPC socket dir; \
                                 using synthetic fallback executor"
                            );
                        }
                        let cfg = elai_orchestrator::apply_task_runner_wiring(
                            cursor_path,
                            &wiring,
                        );
                        let (join_handle, cancel) =
                            elai_orchestrator::start_task_runner(runner_ssb, cfg);
                        match state.runner_join.lock() {
                            Ok(mut g) => *g = Some(join_handle),
                            Err(e) => tracing::error!(
                                error = %e,
                                "dashboard-v2: runner_join mutex poisoned; runner handle dropped"
                            ),
                        }
                        match state.runner_cancel.lock() {
                            Ok(mut g) => *g = Some(cancel),
                            Err(e) => tracing::error!(
                                error = %e,
                                "dashboard-v2: runner_cancel mutex poisoned; cancel handle dropped"
                            ),
                        }
                        tracing::info!(
                            project_root = %project_root.display(),
                            session_id = %session_id,
                            baby_step_executor = wiring.baby_step_executor.is_some(),
                            "dashboard-v2: spawned start_task_runner with full wiring \
                             (feasibility_prober=Some, layer1a_processor=Some)"
                        );
                    }
                    Err(err) => tracing::warn!(
                        error = %err,
                        project_root = %project_root.display(),
                        "dashboard-v2: build_task_runner_wiring failed; runner not started"
                    ),
                }
            } else {
                tracing::warn!(
                    "dashboard-v2: media SSB subscriber + task_runner not spawned (no writer); \
                     media validations will be empty until a session writes the db"
                );
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // Phase A1b: graceful task_runner shutdown on app exit.
            // Tauri 2.11 `RunEvent::ExitRequested` (tauri::app L225 in
            // tauri-2.11.1/src/app.rs) fires when the user closes the
            // window OR `AppHandle::exit` is called programmatically.
            // We take the `CancelHandle` out of AppState and flip its
            // AtomicBool — the runner loop observes the flip on its next
            // poll and returns from `start_task_runner` cleanly.
            // The JoinHandle is intentionally NOT awaited here: the
            // closure is `FnMut + 'static` and `RunEvent::ExitRequested`
            // arrives on the main thread without an async context, so
            // awaiting would block the event loop. Tauri proceeds to
            // tear down the runtime regardless; the runner has at most
            // one tick-period to observe the cancel before the process
            // exits, which is the same shutdown contract `elai run`
            // uses (run_subcommand.rs ctrl_c arm).
            if let tauri::RunEvent::ExitRequested { .. } = event {
                use tauri::Manager;
                if let Some(state) = app_handle.try_state::<AppState>() {
                    let cancel_opt = match state.runner_cancel.lock() {
                        Ok(mut g) => g.take(),
                        Err(e) => {
                            tracing::error!(
                                error = %e,
                                "dashboard-v2: runner_cancel mutex poisoned on shutdown; \
                                 task_runner will exit when its tokio runtime is dropped"
                            );
                            None
                        }
                    };
                    if let Some(cancel) = cancel_opt {
                        cancel.cancel();
                        tracing::info!("dashboard-v2: task_runner cancel signal sent on ExitRequested");
                    }
                }
            }
        });
}
