//! ELAI dashboard-v2 backend entry point. Constructs the Tauri builder,
//! attaches a single `AppState`, and registers every Tauri command consumed
//! by the Svelte frontend.
//!
//! See `state.rs` for the dashboard's runtime state and `commands.rs` for the
//! command-by-command implementation. We deliberately keep this file thin —
//! it should never grow beyond a builder + invoke_handler list + bootstrap.

mod commands;
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
            // CLOUD MODEL PICKER
            commands::get_cloud_model,
            commands::set_cloud_model,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
