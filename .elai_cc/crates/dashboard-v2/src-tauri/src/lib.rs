//! ELAI dashboard-v2 backend entry point. Constructs the Tauri builder,
//! attaches a single `AppState`, and registers every Tauri command consumed
//! by the Svelte frontend.
//!
//! See `state.rs` for the dashboard's runtime state and `commands.rs` for the
//! command-by-command implementation. We deliberately keep this file thin —
//! it should never grow beyond a builder + invoke_handler list + bootstrap.

mod commands;
// D-1: ReasoningTemplate ArchiveChain Tauri command surface. Additive — does
// not refactor `commands.rs`. See workflow/reasoning/D-1-decisions.md.
mod commands_d1;
// E-1 dashboard slice: chat-memory / MemoryNotes + CoreBlock read-only
// Tauri command surface. Separate module to avoid racing the other
// E-plan orchestration sessions that are concurrently editing
// commands.rs. See the W2-E1 fixer brief for the WIRING_CONTRACT
// rationale.
mod commands_e1;
// D-2: PPM (Process Preference Model) read-only command surface. Additive —
// does not refactor `commands.rs` (owned by other orchestration sessions).
// See workflow/reasoning/D-2-decisions.md.
mod commands_d2;
// D-3: multi-verifier consensus read-only command surface. Additive — does
// not refactor commands.rs / commands_d2.rs. See
// workflow/reasoning/D-3-decisions.md.
mod commands_d3;
// D-4: tripwire stack read-only command surface. Additive — does not
// refactor commands.rs / commands_d2.rs / commands_d3.rs. The orchestrator
// producer (tripwire::TripwireMonitor::observe + tripwire::auto_revert) is
// the sole writer of `tripwire_fired` + `template_auto_reverted` rows.
mod commands_d4;
// D-19: PRMBench adversarial-gaming benchmark read-only command surface.
// Additive — does not refactor commands.rs / commands_d2.rs /
// commands_d3.rs / commands_d4.rs. The orchestrator producer
// (`benchmarks::prmbench_runner::PRMBenchRunner::run_all`) is the sole
// writer of `prmbench_run_completed` rows.
mod commands_d19;
// D-5: BAI (Best-Arm Identification) Thompson-sampling promotion round
// read-only command surface. Additive — does not refactor commands.rs /
// commands_d{1,2,3,4,19}.rs. The orchestrator producer
// (`bai::emit_bai_round`) is the sole writer of `bai_arm_updated` rows.
mod commands_d5;
// D-6: Time-windowed held-out task store + 9-gram contamination filter
// read-only command surface. Additive — sibling of commands_d5. The
// orchestrator producer (`held_out::insert_and_emit` +
// `held_out::mark_used_and_emit`) is the sole writer of
// `held_out_task_added` / `held_out_task_used` rows.
mod commands_d6;
// D-7: Cross-domain transfer canary read-only command surface. Additive
// — sibling of commands_d6. The orchestrator producer
// (`canary::emit_canary_run`) is the sole writer of
// `canary_run_completed` / `canary_family_evaluated` rows.
mod commands_d7;
// D-9: L3 frozen-surface auditor read-only command surface. Additive
// — sibling of commands_d7. The orchestrator producer
// (`LockedKeysAuditor::audit_diff_ranged`) is the sole writer of
// `locked_keys_audit_run` / `locked_keys_violation_detected` rows.
mod commands_d9;
// D-10: mode-routing predicate read-only command surface. Sibling of
// commands_d9. The orchestrator producer (`agent_pool::route_claim`)
// is the sole writer of `claim_routed` SSB rows; this module only reads.
mod commands_d10;
// D-21: mode-routing coverage benchmark read-only command surface.
// Sibling of commands_d10. The orchestrator producer
// (`benchmarks::mode_routing_runner::ModeRoutingBenchmark::run_all`) is
// the sole writer of `mode_routing_bench_run_completed` rows; this
// module only reads.
mod commands_d21;
// D-11: constructive-checker registry read-only command surface.
// Sibling of commands_d10. The orchestrator producer
// (`elai_orchestrator::dispatch_constructive_checkers`) is the sole
// writer of `checker_result_emitted` SSB rows; this module only reads.
mod commands_d11;
// D-12: OperationalizationCheck baby-step predicate read-only command
// surface. Sibling of commands_d10 / commands_d11. The orchestrator
// producer (`elai_orchestrator::run_operational_check`) is the sole
// writer of `baby_step_dispatched` SSB rows; this module only reads.
mod commands_d12;
// E-46: Hierarchical decomposition tuning read-only command surface.
// Sibling of commands_d10. The orchestrator producers
// (`agent_pool::process_task_advancement` /
// `emit_agentless_shortcut_taken` / `emit_goal_drift_fired`) are the
// sole writers of `task_advanced` / `agentless_shortcut_taken` /
// `goal_drift_fired` rows; this module only reads.
mod commands_e46;
// E-35: grammar-compilation visibility command. New module to keep the
// boundary clean — `commands.rs` is owned by other orchestration sessions
// in parallel and merging there would race.
mod commands_e35;
// E-5: MemoryFacts read-only Tauri commands (get_memory_facts /
// get_trust_score). Separate module to avoid racing the other E-plan
// orchestration sessions that are concurrently editing commands.rs.
mod commands_e5;
// D-26: ReasoningTemplate + archive dashboard surface extension.
// Adds get_archive_lineage + get_template_archive_combined read-only
// composites on top of D-1 / D-4 / D-5. No producer logic; pure reads.
// See workflow/reasoning/D-26-decisions.md (if generated by the
// orchestrator's memory-writer pass).
mod commands_d26;
// D-27: Mode-routing + refusal-SLA dashboard composite read surface.
// Adds get_mode_routing_refusal_combined on top of D-10 / D-15 /
// D-16 / D-17. No producer logic; pure read composing the four
// Wave-2 surfaces into a single dashboard refresh payload.
// See workflow/reasoning/D-27-decisions.md.
mod commands_d27;
// D-13: adversary role + cross-family verifier read-only command surface.
// Sibling of commands_d10 / commands_d11. The orchestrator producer
// (`elai_orchestrator::agent_pool::run_adversarial_pass`) is the sole
// writer of `adversary_generated` + `consensus_reevaluated_with_counter`
// rows; this module only reads.
mod commands_d13;
// D-16: UniCR conformal-refusal calibration read-only command surface.
// Sibling of commands_d10 / commands_d11 / commands_d13. The
// orchestrator producers
// (`elai_orchestrator::conformal_calibration::compute_tau_and_emit` +
// `decide_and_emit`, both reachable via
// `agent_pool::dispatch_conformal_refusal`) are the sole writers of
// `conformal_tau_computed` + `conformal_refusal_decided` rows; this
// module only reads.
mod commands_d16;
// D-15: discovery-vs-search NoveltyClass read-only command surface.
// Sibling of commands_d11 / commands_d12 / commands_d13 / commands_d16.
// The orchestrator producers
// (`elai_orchestrator::novelty::emit_novelty_classified` +
// `agent_pool::dispatch_novelty`) are the sole writers of
// `novelty_classified` rows; this module only reads.
mod commands_d15;
// D-18 — typed-ObservableEffect predicate read surface. Carries the
// `observable_effect_verified` SSB rows the orchestrator producer
// (`elai_orchestrator::observable_effect::verify_observable_effect`)
// emits; this module only reads.
mod commands_d18;
// D-17 — honest-refusal SLA monitoring read surface. Carries the
// `refusal_rate_snapshot` + `refusal_sla_breach` SSB rows the
// orchestrator producer
// (`elai_orchestrator::refusal_sla::RefusalRateMonitor::emit_snapshot`,
// reachable via `agent_pool::dispatch_refusal_sla_check`) emits; this
// module only reads.
mod commands_d17;
// D-22 — conformal calibration validation read surface. Carries the
// `conformal_validation_completed` SSB rows the orchestrator producer
// (`elai_orchestrator::benchmarks::conformal_validation_runner::ConformalValidationRunner::run_all`)
// emits; this module only reads.
mod commands_d22;
// D-20 — ARC-AGI-2 OOD probe read surface. Carries the
// `arc_benchmark_completed` SSB rows the orchestrator producer
// (`elai_orchestrator::benchmarks::arc_agi2_runner::ARCAGI2Runner::run_all`)
// emits; this module only reads.
mod commands_d20;
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

// C-18 D2: re-export the Tauri-free implementation extracted from the
// `export_graph_mermaid` command. Surfaces a single function from
// `commands.rs` without making the entire module `pub`, matching the minimum
// surface-area principle from the C-18 plan ("ZERO production code changes
// EXCEPT the 3 visibility/extraction refactors above"). Integration tests
// under `crates/dashboard-v2/src-tauri/tests/` call this directly because
// Tauri `State<>` cannot be constructed outside the Tauri runtime.
pub use commands::export_graph_mermaid_impl;
// C-24 (DECISION-10): same C-18 D2 extraction pattern applied to the two
// other Tauri State<>-bound graph commands so their integration tests can
// call the impl directly.
pub use commands::{get_graph_view_data_impl, get_impact_overlay_impl};

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

    // FIXME(tokio-runtime): `Ssb::new()` calls `tokio::spawn(write_loop(...))`. stub-ban-allow: documented Tauri/Tokio bootstrap workaround
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
            // C-8 Task #7: dashboard-originated SearchModeChanged emitter.
            commands::set_search_mode,
            commands::get_embedding_breakers,
            commands::get_recall_metrics,
            commands::get_dkg_stats,
            // B-30: language grammar registry status
            commands::get_language_grammar_status,
            // C-3: knowledge dashboard surface — pagerank / freshness /
            // canary drift / research budget / source-tier distribution.
            // cross-lang-pair: dashboard/tauri/api
            commands::get_pagerank_top,
            commands::get_freshness_state,
            commands::get_canary_drift,
            commands::get_research_budget,
            commands::get_source_tier_distribution,
            // E-17 SP4: RAG retrieval-quality SSB row surface. Four
            // read-only commands over the SCHEMA_VERSION 97 SP2 payload
            // variants (DeepPoolCapped / RrfQualityGateFired /
            // HydeSkippedCodeQuery / RrfComponentWeights). Producers ship
            // in E-17 SP2 (knowledge::pipeline / deep / hyde); this
            // surface only reads via SsbViewer::recent_rows_of_type.
            // cross-lang-pair: dashboard/tauri/api
            commands::get_deep_pool_capped_recent,
            commands::get_rrf_quality_gate_recent,
            commands::get_hyde_skipped_code_query_recent,
            commands::get_rrf_component_weights_recent,
            // REASONING / GRC
            commands::get_grc_budgets,
            commands::get_grounding_sweep,
            commands::get_csq_pending,
            commands::get_gg_signals,
            // C-3: reasoning dashboard surface — CSQ log (all verdicts),
            // feasibility log (raw rows), per-role GRC config.
            // cross-lang-pair: dashboard/tauri/api
            commands::get_csq_log,
            commands::get_feasibility_log,
            commands::get_agent_grc_config,
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
            // C-23: /continue surface — load latest handoff snapshot
            // from the active SessionDag.
            commands::load_handoff,
            commands::get_anti_spiral_counter,
            // C-10 — anti-spiral StagnationSection event feed.
            commands::get_stagnation_events,
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
            // MUTATION GAUGE (C-28)
            commands::get_mutation_gauge,
            // COMPLETENESS RATIO (C-32) -- Code-Completeness Layer 2 gauge.
            commands::get_completeness_ratio,
            // INTENT-PAIRS (C-29)
            // cross-lang-pair: dashboard/tauri/api
            commands::add_intent_pair,
            commands::confirm_intent_pairs,
            commands::get_intent_pairs,
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
            // B-31 Phase 4 (Cycle B) — INDEXING PROGRESS + CANCEL
            commands::cancel_indexing,
            commands::get_indexing_progress,
            // C-17 (DRIFT-J): indexing resilience snapshot.
            commands::get_indexing_resilience,
            // B-29 Phase 5 — IMPACT ANALYSIS
            commands::get_impact_analysis,
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
            // B-35 Phase C — CONTEXT ARTIFACTS + ENTRY POINTS
            commands::list_artifacts,
            commands::search_artifacts,
            commands::index_artifacts,
            commands::remove_artifacts,
            commands::list_entry_points,
            // REMOTE SIDECAR BOOT OVERLAY
            commands::start_remote_sidecar,
            commands::get_remote_sidecar_progress,
            commands::cancel_remote_sidecar,
            // B-39 — REFLECT STATE
            commands::get_reflect_state,
            // B-28 follow-up — FOCUS TUNNEL INLINE CHIPS (single per-baby-step
            // query; the dedicated FocusTunnelPanel surface was retired).
            commands::query_focus_tunnel_per_baby_step,
            // B-33 — INTERACTIVE GRAPH VIEWER (3 commands)
            commands::get_graph_view_data,
            commands::get_impact_overlay,
            commands::export_graph_mermaid,
            // B-24 Phase 4I Group C8 — MEDIA VALIDATION SURFACE
            media_validation::get_media_validations,
            media_validation::get_media_validation_by_step,
            media_validation::submit_media_validation_for_review,
            media_validation::subscribe_media_ssb_events,
            // B-40 — IDEA FRAMING SURFACE
            commands::get_idea_framing_state,
            commands::submit_tradeoff_responses,
            // B-38 — AMBIGUITY GATE SURFACE (mock data; real producer in follow-up).
            // cross-lang-pair: dashboard/tauri/api
            commands::get_ambiguity_state,
            // B-42 — COSMOS ARTIFACT TIER (read-only status; refresh trigger
            // returns explicit NotImplemented Err pending dispatcher wiring —
            // see workflow/reasoning/B-42-decisions.md BLOCK-WIRING).
            // cross-lang-pair: dashboard/tauri/api
            commands::get_cosmos_status,
            commands::trigger_cosmos_refresh,
            // C-16 (DECISION-C): cosmos SSB poll-subscription. Returns rows
            // since last_seq; frontend polls at 5s cadence for the drawer
            // live-update path.
            commands::subscribe_cosmos_events,
            // B-41 — Per-level auditor verdicts feed for AuditTimelineWindow.
            // cross-lang-pair: dashboard/tauri/api
            commands::get_per_level_verdicts,
            // B-37D — MULTI-RUNTIME BACKEND ABSTRACTION (Phase 4M, Wave-2A).
            // list_runtimes / test_runtime / set_role_runtime surface the
            // 11-backend RuntimeOrchestrator (per-role mapping + fallback
            // chain + per-backend health). set_role_runtime atomically
            // persists to elai_cc.toml via the same shape used by the CLI
            // `/runtime set` handler (cli/src/runtime_cmd.rs).
            // cross-lang-pair: dashboard/tauri/api
            commands::list_runtimes,
            commands::test_runtime,
            commands::set_role_runtime,
            // C-4 — ORCHESTRATION + CLOUD SPEND SURFACE.
            // 9 new commands wired over SsbViewer methods + a greenfield
            // projects.json persistence pair (list_projects/create_project).
            // get_executable_pipeline and get_cloud_spend are already
            // registered above; their bodies were rewritten in C-4 to
            // call real SsbViewer methods instead of returning stubs.
            // cross-lang-pair: dashboard/tauri/api
            commands::get_phase_detail,
            commands::get_baby_step_detail,
            commands::get_phase_progress,
            commands::get_recent_actions,
            commands::get_file_changes,
            commands::get_action_detail,
            commands::get_research_node_history,
            // C-7: review-window reviewer-verdict commands. Each emits one
            // ChangeDecision SSB row (SCHEMA_VERSION 49). Live diff itself
            // remains carried by FileWrite rows; these record only verdicts.
            commands::accept_change,
            commands::decline_change,
            commands::retry_audit,
            commands::list_projects,
            commands::create_project,
            // E-35 — GRAMMAR COMPILATION VISIBILITY (XGrammar substrate).
            // Producer (sidecar grammar_cache_loaded SSB rows) lands in
            // followups_for_other_sessions; the command surface is wired
            // now so the Svelte panel renders an empty state cleanly.
            commands_e35::get_grammar_compilations,
            // E-5 — USER MODELING + LONG-TERM MEMORY (bi-temporal
            // memory_facts store). Read-only surface for the Svelte
            // MemoryFactsDrawer; mutation commands (ADD/UPDATE/DELETE/
            // NOOP dispatcher) reserved for the follow-up vertical slice.
            commands_e5::get_memory_facts,
            commands_e5::get_trust_score,
            // E-1 dashboard slice — CHAT-MEMORY (MemoryNotes + CoreBlock)
            // read-only surface for the Svelte MemoryNotesPanel. Producer
            // wiring (NoteStore::insert + CoreBlockStore::upsert emitters
            // in task_runner.rs) is owned by another orchestration session
            // — see followups_for_other_sessions in W2-E1 brief.
            commands_e1::get_memory_notes,
            commands_e1::get_chat_memory_state,
            // D-1 — REASONING TEMPLATE ARCHIVE.
            // Read-only surface for the Svelte MutationArchivePanel. The
            // archive itself is appended by the orchestrator producer
            // (archive_chain::append_and_emit_ssb); these commands only read.
            commands_d1::get_archive_recent,
            commands_d1::get_reasoning_template_by_id,
            // D-2 — PPM (Process Preference Model) verifier scores.
            // Read-only surface for the Svelte PpmScoresPanel. The rows
            // are written by the orchestrator producer
            // (ppm_client::emit_ppm_score); this command only reads from SSB.
            commands_d2::get_ppm_recent_scores,
            // D-3 — multi-verifier consensus rows.
            // Read-only surface for the Svelte VerificationConsensusPanel.
            // Rows are written by the orchestrator producer
            // (agent_pool::run_consensus_pass).
            commands_d3::get_recent_verifications,
            // D-4 — tripwire stack rows.
            // Read-only surface for the Svelte TripwirePanel. Rows are
            // written by the orchestrator producer
            // (tripwire::TripwireMonitor + tripwire::auto_revert).
            commands_d4::get_recent_tripwires,
            // D-19 — PRMBench adversarial-gaming benchmark rows.
            // Read-only surface for the Svelte PrmBenchPanel. Rows are
            // written by the orchestrator producer
            // (benchmarks::prmbench_runner::PRMBenchRunner::run_all).
            commands_d19::get_prmbench_recent,
            // D-5 — BAI (Best-Arm Identification) Thompson-sampling
            // promotion round rows. Read-only surface for the Svelte
            // BaiArmsPanel. Rows are written by the orchestrator producer
            // (bai::emit_bai_round) carrying the Beta(alpha, beta)
            // posterior + n_pulls + mean_estimate + was_promoted.
            commands_d5::get_bai_arms,
            // D-6 — held-out task store rows + aggregate stats.
            // Read-only surface for the Svelte HeldOutPanel. Rows are
            // written by the orchestrator producer
            // (held_out::insert_and_emit + held_out::mark_used_and_emit).
            commands_d6::get_held_out_recent,
            commands_d6::get_held_out_stats,
            // D-7 — cross-domain transfer canary read surface. Rows
            // are written by the orchestrator producer
            // (canary::emit_canary_run); this surface only reads.
            commands_d7::get_canary_recent,
            commands_d7::get_canary_per_family_breakdown,
            // D-9 — L3 frozen-surface auditor read surface. Rows are
            // written by the orchestrator producer
            // (LockedKeysAuditor::audit_diff_ranged); this surface
            // only reads.
            commands_d9::get_locked_keys_status,
            commands_d9::get_locked_keys_recent_audits,
            // D-10 — mode-routing predicate read surface. Rows are
            // written by the orchestrator producer
            // (`elai_orchestrator::route_claim`); this surface only reads.
            commands_d10::get_recent_claim_routings,
            // D-21 — mode-routing coverage benchmark read surface. Rows
            // are written by the orchestrator producer
            // (`benchmarks::mode_routing_runner::ModeRoutingBenchmark::run_all`);
            // this surface only reads.
            commands_d21::get_mode_routing_bench_recent,
            // D-11 — constructive-checker registry read surface. Rows
            // are written by the orchestrator producer
            // (`elai_orchestrator::dispatch_constructive_checkers`);
            // this surface only reads.
            commands_d11::get_checker_results_recent,
            commands_d11::get_checker_registry_state,
            // D-12 — OperationalizationCheck baby-step predicate read
            // surface. Rows are written by the orchestrator producer
            // (`elai_orchestrator::run_operational_check`); this surface
            // only reads.
            commands_d12::get_baby_step_dispatches_recent,
            // D-13 — adversary-run + cross-family verifier read surface.
            // Rows are written by the orchestrator producer
            // (`elai_orchestrator::agent_pool::run_adversarial_pass`);
            // this surface only reads.
            commands_d13::get_adversarial_runs_recent,
            commands_d13::get_adversary_config,
            // D-16 — UniCR conformal-refusal calibration read surface.
            // Rows are written by the orchestrator producers
            // (`elai_orchestrator::conformal_calibration::compute_tau_and_emit`
            // + `decide_and_emit`); this surface only reads.
            commands_d16::get_conformal_recent,
            commands_d16::get_calibration_corpus_state,
            // D-15 — discovery-vs-search NoveltyClass read surface. Rows
            // are written by the orchestrator producers
            // (`elai_orchestrator::novelty::emit_novelty_classified` +
            // `agent_pool::dispatch_novelty`); this surface only reads.
            commands_d15::get_novelty_recent,
            commands_d15::get_novelty_distribution,
            // D-18 — typed-ObservableEffect predicate read surface.
            // Rows are written by the orchestrator producer
            // (`elai_orchestrator::observable_effect::verify_observable_effect`);
            // this surface only reads.
            commands_d18::get_observable_effects_recent,
            // D-17 — honest-refusal SLA monitoring read surface. Rows
            // are written by the orchestrator producer
            // (`elai_orchestrator::refusal_sla::RefusalRateMonitor::emit_snapshot`,
            // reachable via `agent_pool::dispatch_refusal_sla_check`);
            // this surface only reads.
            commands_d17::get_refusal_sla_recent,
            commands_d17::get_refusal_rate_current,
            // D-22 — Conformal calibration validation read surface. Rows
            // are written by the orchestrator producer
            // (`benchmarks::conformal_validation_runner::ConformalValidationRunner::run_all`);
            // this surface only reads.
            commands_d22::get_conformal_validation_recent,
            // D-20 — ARC-AGI-2 OOD probe read surface. Rows are written
            // by the orchestrator producer
            // (`benchmarks::arc_agi2_runner::ARCAGI2Runner::run_all`);
            // this surface only reads.
            commands_d20::get_arc_bench_recent,
            // E-46 — Hierarchical decomposition tuning read surface.
            // Rows are written by the orchestrator producers
            // (`agent_pool::process_task_advancement` /
            // `emit_agentless_shortcut_taken` / `emit_goal_drift_fired`);
            // this surface only reads.
            commands_e46::get_dual_ledger_state,
            commands_e46::get_recent_agentless_shortcuts,
            commands_e46::get_goal_drift_status,
            // D-26 — ReasoningTemplate archive dashboard composite surface.
            // get_archive_lineage walks the D-1 chain backwards from a
            // template id; get_template_archive_combined fuses archive +
            // tripwire + BAI summaries for the new ArchiveExplorerView.
            // No producer logic — pure read.
            commands_d26::get_archive_lineage,
            commands_d26::get_template_archive_combined,
            // D-27 — Mode-routing + refusal-SLA dashboard composite.
            // Single read returns mode_breakdown (D-10) +
            // refusal_rate_current (D-17) + discovery_queue (D-15
            // filtered to discovery_candidate) + recent_classifications
            // (D-15) so ModeRoutingDashboardView refreshes in one
            // round-trip. No producer.
            commands_d27::get_mode_routing_refusal_combined,
            // E-24 -- MCP bridge hardening status surface.
            commands::get_mcp_security_status,
            // E-51 -- Runtime capability probe (MLX / LlamaCpp platform detect).
            commands::runtime_capability_probe,
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
                media_validation::spawn_subscriber(handle.clone(), writer.clone(), media_state);

                // B-33 follow-up (SCHEMA_VERSION 40): tap the SSB broadcast
                // for SymbolGraphUpdated rows and re-emit them as a Tauri
                // event named "symbol_graph_updated" so dashboard-v2
                // GraphWindow can refresh without polling. Replaces the 30s
                // setInterval fallback that B-33 D3 shipped while ssb.rs
                // was locked. Defense-in-depth: GraphWindow keeps the 30s
                // poll as a backstop for hot-reload + dropped-event cases.
                spawn_symbol_graph_subscriber(handle.clone(), writer);

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
                                                    // C-13 — wrap the real
                                                    // InProcessToolDispatcher in the
                                                    // IpcToolDispatcherAdapter shim
                                                    // (dashboard cloud arm). See
                                                    // workflow/reasoning/C-13-decisions.md
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
                                                // patch-R5 — canonical baby-step
                                                // system prompt so Sonnet emits
                                                // ToolUse blocks (see
                                                // crates/orchestrator/src/production_agent_rpc.rs
                                                // DEFAULT_BABY_STEP_SYSTEM_PROMPT docstring).
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
                                            // C-13 — wrap the real
                                            // InProcessToolDispatcher in the
                                            // IpcToolDispatcherAdapter shim
                                            // (dashboard sidecar arm). See
                                            // workflow/reasoning/C-13-decisions.md
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
                                        // patch-R5 — same canonical baby-step
                                        // system prompt as the cloud arm above.
                                        std::sync::Arc::new(
                                            elai_orchestrator::ProductionAgentRpcServer::new(
                                                client, dispatcher, model, max_tokens,
                                            )
                                            .with_system_prompt(
                                                elai_orchestrator::production_agent_rpc::DEFAULT_BABY_STEP_SYSTEM_PROMPT,
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
                        // C-9 — install AutonomousGate via the wiring
                        // chokepoint. Dashboard always starts in Interactive
                        // mode; users can flip to Autonomous via the
                        // `/mode` slash command at runtime
                        // (set_gate_mode IPC, live_cli.rs:2690).
                        wiring.gate_mode =
                            Some(elai_orchestrator::GateMode::Interactive);

                        // C-5 — install the CloudModeRouter via the wiring
                        // chokepoint. The dashboard does NOT depend on the
                        // CLI crate, so we read `[cloud]` directly from the
                        // project-root elai_cc.toml here with a thin
                        // local helper. Falls back to CloudConfig::default()
                        // when the file is missing or `[cloud]` is empty so
                        // the router is always constructable; the dashboard
                        // `set_execution_mode` Tauri command flips between
                        // providers at runtime. provider_override is None
                        // because the dashboard has no CLI `--provider`
                        // flag — toml default carries the initial choice.
                        let cloud_cfg = load_cloud_config_from_toml(&project_root);
                        tracing::info!(
                            toml_default_mode = ?cloud_cfg.default_mode,
                            privacy_profile = ?cloud_cfg.privacy_profile,
                            air_gapped = cloud_cfg.air_gapped,
                            spend_cap_daily_usd = ?cloud_cfg.spend_cap_daily_usd,
                            "dashboard-v2: cloud-mode router resolved"
                        );
                        wiring.cloud_config = Some(cloud_cfg);
                        wiring.provider_override = None;

                        // C-13 — install the production hook dispatcher
                        // BEFORE wiring is applied so apply_task_runner_wiring
                        // threads it onto TaskRunnerConfig.hook_dispatcher
                        // (Tier-1 surface). See
                        // workflow/reasoning/C-13-decisions.md (DECISION D3
                        // option 3A).
                        wiring.hook_dispatcher = Some(
                            elai_orchestrator::runtime_setup::build_production_hook_dispatcher(),
                        );

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

                        // B-31 Phase 4 (Cycle B): fire-and-forget auto-resume
                        // of any project whose persisted indexing_status is
                        // InProgress. Mirrors SocratiCode startup.ts:37-150.
                        // Non-blocking: per-project tasks fan out under
                        // spawn_blocking; setup() returns immediately so the
                        // first window paints without waiting.
                        let auto_resume_indexes_root = project_root
                            .join(elai_core::paths::ELAI_DATA_DIR_NAME)
                            .join(elai_core::paths::INDEXES_DIR_NAME);
                        let auto_resume_clock = state.clock.clone();
                        tauri::async_runtime::spawn(async move {
                            let report =
                                elai_orchestrator::startup::auto_resume_indexed_projects(
                                    auto_resume_indexes_root,
                                    auto_resume_clock,
                                )
                                .await;
                            tracing::info!(
                                ?report,
                                "dashboard-v2: auto_resume_indexed_projects completed"
                            );
                        });
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

                    // B-31 Phase 4 (Cycle B): graceful shutdown of any active
                    // indexing run. Flips the process-global SHUTTING_DOWN
                    // flag, requests cancellation for every InProgress
                    // project, waits up to STARTUP_SHUTDOWN_DEADLINE_SECS,
                    // and force-releases any orphan locks on deadline expiry.
                    // RunEvent closure runs on the main thread with no async
                    // context; tauri::async_runtime::block_on is the only way
                    // to await here. The leaked tokio runtime from run()
                    // head (mem::forget(rt)) provides the active reactor.
                    let shutdown_indexes_root = state
                        .project_root_snapshot()
                        .join(elai_core::paths::ELAI_DATA_DIR_NAME)
                        .join(elai_core::paths::INDEXES_DIR_NAME);
                    let shutdown_clock = state.clock.clone();
                    let shutdown_deadline = std::time::Duration::from_secs(
                        elai_orchestrator::startup::STARTUP_SHUTDOWN_DEADLINE_SECS,
                    );
                    let shutdown_report = tauri::async_runtime::block_on(async move {
                        elai_orchestrator::startup::graceful_shutdown(
                            shutdown_indexes_root,
                            shutdown_deadline,
                            shutdown_clock,
                        )
                        .await
                    });
                    tracing::info!(
                        ?shutdown_report,
                        "dashboard-v2: graceful_shutdown completed on ExitRequested"
                    );
                }
            }
        });
}

/// B-33 follow-up (SCHEMA_VERSION 40): SSB broadcast subscriber that taps
/// `SymbolGraphUpdated` rows and re-emits them as a Tauri event named
/// `symbol_graph_updated`. The frontend `GraphWindow.svelte` listens for
/// this event via `@tauri-apps/api/event::listen` and refreshes its view
/// without waiting for the 30 s polling fallback.
///
/// The pattern intentionally mirrors `media_validation::spawn_subscriber`:
/// tap the broadcast, filter by action_type, decode the payload, and emit
/// a small JSON event back to the webview. Decoding goes through
/// `decode_ssb_payload_bytes::<SsbPayload>` (Decision 92, B-24 audit) so
/// the externally-tagged enum variant is correctly matched.
///
/// `Lagged` is logged + skipped — falling behind a few rows is acceptable
/// for a refresh hint. `Closed` exits the loop cleanly (Ssb shutdown).
fn spawn_symbol_graph_subscriber(
    app: tauri::AppHandle,
    ssb_writer: std::sync::Arc<elai_orchestrator::ssb::Ssb>,
) {
    use tauri::Emitter;
    use tokio::sync::broadcast::error::RecvError;

    /// Tauri event name. Must match `GraphWindow.svelte` listener.
    const SYMBOL_GRAPH_UPDATED_EVENT: &str = "symbol_graph_updated";

    /// Compact event payload re-broadcast to the frontend. Carries the same
    /// 5 fields as `SymbolGraphUpdatedPayload` plus the SSB row sequence so
    /// the frontend can dedupe across rapid back-to-back rebuilds.
    #[derive(serde::Serialize, Clone)]
    struct SymbolGraphUpdatedEvent {
        seq: i64,
        ts_ms: i64,
        project_id: String,
        file_count: u32,
        symbol_count: u64,
        edge_count: u64,
        built_ms: i64,
    }

    let mut rx = ssb_writer.subscribe();
    tauri::async_runtime::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(entry) => {
                    if entry.action_type != elai_orchestrator::SsbActionType::SymbolGraphUpdated {
                        continue;
                    }
                    let payload: elai_orchestrator::SsbPayload =
                        match elai_orchestrator::decode_ssb_payload_bytes(&entry.payload) {
                            Ok(p) => p,
                            Err(e) => {
                                tracing::warn!(
                                    error = %e,
                                    "dashboard-v2: symbol_graph_updated payload decode failed"
                                );
                                continue;
                            }
                        };
                    let inner = match payload {
                        elai_orchestrator::SsbPayload::SymbolGraphUpdated(p) => p,
                        other => {
                            tracing::warn!(
                                actual = other.action_type().as_sql_str(),
                                "dashboard-v2: symbol_graph_updated action_type / payload mismatch"
                            );
                            continue;
                        }
                    };
                    let event = SymbolGraphUpdatedEvent {
                        seq: entry.seq,
                        ts_ms: entry.created_at,
                        project_id: inner.project_id,
                        file_count: inner.file_count,
                        symbol_count: inner.symbol_count,
                        edge_count: inner.edge_count,
                        built_ms: inner.built_ms,
                    };
                    if let Err(e) = app.emit(SYMBOL_GRAPH_UPDATED_EVENT, &event) {
                        tracing::warn!(
                            error = %e,
                            "dashboard-v2: emit symbol_graph_updated failed"
                        );
                    }
                }
                Err(RecvError::Lagged(n)) => {
                    tracing::warn!(
                        skipped = n,
                        "dashboard-v2: symbol_graph SSB subscriber lagged"
                    );
                    continue;
                }
                Err(RecvError::Closed) => {
                    tracing::debug!(
                        "dashboard-v2: symbol_graph SSB subscriber channel closed; exiting"
                    );
                    return;
                }
            }
        }
    });
}

// ---------------------------------------------------------------------------
// C-5 — thin elai_cc.toml `[cloud]` reader.
//
// The dashboard does not depend on the CLI crate (which owns the full
// runtime_config loader). Rather than add that dep purely to read 6
// fields, we parse the toml ourselves into the orchestrator's typed
// `CloudConfig`. Missing fields fall through to the orchestrator
// `CloudConfig::default()` so the router is always constructable.
// ---------------------------------------------------------------------------

use serde::Deserialize;

/// Mirror of the `[cloud]` toml section used purely for parsing.
/// Mirrors `elai-cli::runtime_config::CloudConfigToml` shape so the wire
/// format stays in lockstep — the CLI is the canonical writer.
#[derive(Debug, Default, Deserialize)]
struct CloudTomlForDashboard {
    #[serde(default)]
    cloud: Option<CloudSectionToml>,
}

#[derive(Debug, Deserialize)]
struct CloudSectionToml {
    #[serde(default)]
    default_provider: Option<String>,
    #[serde(default)]
    default_model: Option<String>,
    #[serde(default)]
    default_mode: Option<String>,
    #[serde(default)]
    air_gapped: Option<bool>,
    #[serde(default)]
    privacy: Option<CloudPrivacyTomlSection>,
    #[serde(default)]
    spend: Option<CloudSpendTomlSection>,
}

#[derive(Debug, Default, Deserialize)]
struct CloudPrivacyTomlSection {
    #[serde(default)]
    profile: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct CloudSpendTomlSection {
    #[serde(default)]
    hard_cap_daily_usd: Option<f64>,
    #[serde(default)]
    hard_cap_monthly_usd: Option<f64>,
}

/// Read `<project_root>/elai_cc.toml` and convert the `[cloud]` section
/// into [`elai_orchestrator::cloud_mode::CloudConfig`].
///
/// Missing file / unparseable toml / missing `[cloud]` table all fall
/// through to `CloudConfig::default()` rather than failing the dashboard
/// boot — the operator can fix the toml at any time and `set_execution_mode`
/// flips the provider without restart.
fn load_cloud_config_from_toml(
    project_root: &std::path::Path,
) -> elai_orchestrator::cloud_mode::CloudConfig {
    use elai_orchestrator::cloud_mode::{CloudConfig, ExecMode};
    use elai_orchestrator::privacy_redactor::RedactionProfile;

    let path = project_root.join("elai_cc.toml");
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            tracing::debug!(
                path = %path.display(),
                error = %e,
                "dashboard-v2: elai_cc.toml not found; using CloudConfig::default()"
            );
            return CloudConfig::default();
        }
    };
    let parsed: CloudTomlForDashboard = match toml::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(
                path = %path.display(),
                error = %e,
                "dashboard-v2: elai_cc.toml parse failed; using CloudConfig::default()"
            );
            return CloudConfig::default();
        }
    };
    let Some(section) = parsed.cloud else {
        return CloudConfig::default();
    };
    let default = CloudConfig::default();
    CloudConfig {
        default_provider: section.default_provider.unwrap_or(default.default_provider),
        default_model: section.default_model.unwrap_or(default.default_model),
        default_mode: section
            .default_mode
            .as_deref()
            .map(ExecMode::from_config_str)
            .unwrap_or(default.default_mode),
        air_gapped: section.air_gapped.unwrap_or(default.air_gapped),
        privacy_profile: section
            .privacy
            .and_then(|p| p.profile)
            .as_deref()
            .map(RedactionProfile::from_config_str)
            .unwrap_or(default.privacy_profile),
        spend_cap_daily_usd: section
            .spend
            .as_ref()
            .and_then(|s| s.hard_cap_daily_usd),
        spend_cap_monthly_usd: section
            .spend
            .as_ref()
            .and_then(|s| s.hard_cap_monthly_usd),
    }
}
