# ELAI Verification — Master Punch List
Generated: 2026-05-17
Sources: 103 verification reports under verification/reports/

## Status summary
- DONE: 5 (B-40, phase-1B, phase-1D, phase-2D, phase-3A)
- MOSTLY_DONE: 77
- PARTIAL: 20
- NOT_DONE: 3 (B-25, guide/testing-trust-design, phase-4J)
- UNVERIFIABLE: 0 (individual unverifiable claims exist within reports but no report is wholly UNVERIFIABLE)

Total reports: 103 (45 B-plan + 18 guide + 40 steps)

## Punch list by source

### B-plan (45 files)

- [B-0_INDEX](reports/B-plan/B-0_INDEX.report.md) — MOSTLY_DONE — Master index stale; B-37/38/40/41/42 not recorded, SCHEMA_VERSION shows 26 not 46
  - [ ] Update B-0_INDEX.md with status blocks for B-37/38/40/41/42 shipped phases
  - [ ] Add current SCHEMA_VERSION (46) or pointer to ssb.rs
  - [ ] Resolve B-15 remaining items (HashMap ban, entropy leaks) in INDEX
  - [ ] Verify B-5 emit_reasoning per-agent budget wiring still open or closed
  - [ ] Confirm oracle-gate patch was committed (INDEX says "No commit yet")
  - [ ] Clarify B-36 obscura/web_search dep status; verify stub.ts run_planning_task calls Tauri

- [B-1_path_unification](reports/B-plan/B-1_path_unification.report.md) — MOSTLY_DONE — Path unification mostly done; minor pagerank tracing and AragOrchestrator add_chunk gaps
  - [ ] Replace guard.indexes_dir().join("dkg.db") hardcoded path with canonical helper
  - [ ] Add tracing::info!(nodes=...) to pagerank stage in knowledge crate
  - [ ] Add startup log lines for BM25, vectors, sessions, models canonical paths
  - [ ] Extend regression scan in path_unification.rs contract test
  - [ ] Decide on AragOrchestrator add_chunk canonical path wiring

- [B-2_fsm_phasetransition_wiring](reports/B-plan/B-2_fsm_phasetransition_wiring.report.md) — MOSTLY_DONE — FSM wiring solid; fsm.rs reset() uses direct mutation, missing B-2 contract test
  - [ ] Fix crates/orchestrator/src/fsm.rs reset() — replace direct state mutation with validated transition
  - [ ] Add crates/contracts/tests/fsm_phase_transition_contract.rs
  - [ ] Verify status-bar--executing CSS class triggers pulse animation
  - [ ] Confirm ActivityWindow/BubbleUpStrip surfaces FSM phase_transition rows

- [B-3_stub_command_eradication](reports/B-plan/B-3_stub_command_eradication.report.md) — PARTIAL — Multiple ssb_viewer stubs remain: pagerank_top, query_tier_stats, freshness_state; dashboard items unwired
  - [ ] Implement pagerank_top SQL in ssb_viewer.rs:2699
  - [ ] Implement query_tier_stats aggregate in ssb_viewer.rs:2708 from QueryTrace SSB rows
  - [ ] Implement freshness_state real DB read in ssb_viewer.rs:2722
  - [ ] Add rule_transition SSB subscription in drawers.svelte.ts; render in RulesDrawer.svelte
  - [ ] Wire /continue to actual SessionDag::load_handoff Tauri command
  - [ ] Refactor emit_reasoning to accept Option<Arc<AsyncMutex<PipelineGrc>>> for real budgets
  - [ ] Add ts-rs dep; generate types.gen.ts from Rust SessionHandoffEntry
  - [ ] Create workflow/reasoning/B-3-decisions.md

- [B-4_knowledge_producer_wiring](reports/B-plan/B-4_knowledge_producer_wiring.report.md) — PARTIAL — SSB emits use tracing::debug not real rows; 10+ Tauri commands missing; HyDE not wired
  - [ ] Write query_trace SSB row in pipeline.rs after each query (replace tracing::debug)
  - [ ] Wire real SsbNotifier into live_cli.rs query path (CrossEncoderReranker uses Noop)
  - [ ] Implement get_query_trace_recent Tauri command; set unwired.recentTraces=false
  - [ ] Add hyde_query SSB action type; emit from SidecarHydeClient
  - [ ] Implement get_pagerank_top, get_freshness_state, get_canary_drift Tauri commands
  - [ ] Add HyDE hypothesis preview panel to KnowledgeDrawer
  - [ ] Fix per-stage latency_ms by threading Clock ref into run_query
  - [ ] Add [models.cross_encoder] section to elai_cc.toml with blake3 hashes

- [B-5_reasoning_producer_wiring](reports/B-plan/B-5_reasoning_producer_wiring.report.md) — PARTIAL — GrcStepEmitted/CsqEvaluated/FeasibilityCheck SSB variants missing; emit_reasoning TODO at line 6218
  - [ ] Add SsbActionType variants GrcStepEmitted/GroundingSweepResult/CsqEvaluated/FeasibilityCheck/AuditorRubberStampRejected with payloads
  - [ ] Refactor emit_reasoning (task_runner.rs:6218) to accept Option<PipelineGrc> with real budget/agent_id
  - [ ] Add real grc_step_emitted SSB emit at all 11 emit_reasoning call sites
  - [ ] Implement csq_evaluated SSB emit (currently TODO at task_runner.rs:5020)
  - [ ] Add get_csq_log, get_feasibility_log, get_agent_grc_config Tauri commands
  - [ ] Create CsqBlockCard.svelte intervention card; wire to ChatWindow on CSQ Critical-without-guard
  - [ ] Add red bar to ReasoningDrawer.svelte at <20% calls/chars remaining
  - [ ] Create workflow/reasoning/B-5-decisions.md

- [B-6_orchestration_producer_wiring](reports/B-plan/B-6_orchestration_producer_wiring.report.md) — MOSTLY_DONE — CliInteractiveGate not wired into ConversationRuntime; get_agents returns hardcoded IDLE
  - [ ] Wire CliInteractiveGate into ConversationRuntime in runtime_builder.rs
  - [ ] Implement live agent status query in get_agents() using SSB AgentLifecycle rows
  - [ ] Add SSB action type agent_status_changed or document AgentLifecycle subsumes it
  - [ ] Resolve Tauri command name mismatches (get_recent_evidence_gates vs get_evidence_gates etc.)
  - [ ] Wire intervention overlay in frontend for InteractiveGate prompts

- [B-7_pipeline_factory_producer_wiring](reports/B-plan/B-7_pipeline_factory_producer_wiring.report.md) — PARTIAL — 7 SSB action types missing; 5 Tauri commands stub/missing; research_node_stage emits absent
  - [ ] Define SSB action types: phase_node_ready, step_node_ready, baby_step_ready in ssb.rs
  - [ ] Define dag_repair_escalated SSB type; emit when T_DAG=10 exhausted
  - [ ] Add get_phase_detail and get_baby_step_detail Tauri commands; fix baby_step_detail stub
  - [ ] Implement get_executable_pipeline (currently returns Ok(json!([])))
  - [ ] Add granular research_node_stage emits (Formulate/Search/Extract/Reflect/Converge stages)
  - [ ] Wire get_dag_repair_log into OrchestrationDrawer.svelte
  - [ ] Implement BubbleUp cascade visualization in PlanWindow.svelte
  - [ ] Create workflow/reasoning/B-7-decisions.md

- [B-8_pipelinegrc_activation](reports/B-plan/B-8_pipelinegrc_activation.report.md) — MOSTLY_DONE — ReplanRequest::Major on Blocked not wired; csq_evaluated SSB row stub; verify_claim stub
  - [ ] Implement ReplanRequest::Major submission when FeasibilityOutcome::Blocked (task_runner.rs:5583)
  - [ ] Implement dedicated SSB csq_evaluated row with severity/guard_present/blocked fields
  - [ ] Implement dashboard intervention card SSB row following csq_evaluated
  - [ ] Wire verify_claim into ContextCompiler pre-fetch (agent_pool.rs:559 currently debug-log only)

- [B-9_sessiondag_activation](reports/B-plan/B-9_sessiondag_activation.report.md) — MOSTLY_DONE — BranchTree unmounted; /session merge/close missing; CLI-delegation stubs not wired
  - [ ] Add /session merge <branch_id> to parse_session_command in cli/src/commands.rs
  - [ ] Add /session close <branch_id> to parse_session_command
  - [ ] Import and mount BranchTree.svelte in a live screen (SessionsDrawer or drawers panel)
  - [ ] Wire BranchTree onfork/onmerge/onclose to dispatch CLI commands via IPC
  - [ ] Verify /continue handler in CommandWindow.svelte triggers load_handoff

- [B-10_contextbudget_profile_activation](reports/B-plan/B-10_contextbudget_profile_activation.report.md) — MOSTLY_DONE — VramTier hardcoded at 12Gb; agent_files not populated for overlap computation
  - [ ] Replace VramTier::Vram12Gb literal at task_runner.rs:2724 with detect_vram_tier()
  - [ ] Populate agent_files from PhaseSlot/task payload in SsbEntries overlap path (task_runner.rs:4246)
  - [ ] Run cargo test -p elai-orchestrator + -p elai_cc_contracts to confirm zero-sum invariant

- [B-11_design_system_restoration](reports/B-plan/B-11_design_system_restoration.report.md) — PARTIAL — CSS design tokens misaligned: bg-void, teal, text, semantic values differ from spec; font wrong
  - [ ] Align all --bg-* token hex values in tokens.css to spec (bg-void=#0a0e12 etc.)
  - [ ] Align teal tokens: --teal=#1B7A6E, --teal-bright=#24a08e, etc.
  - [ ] Fix body font: change --font-sans to 'JetBrains Mono' per spec
  - [ ] Sweep 11px occurrences across 7 Svelte components; replace with 10px per spec
  - [ ] Sweep off-spec sizes (9px->10px, 14px->13px, 17px->15px, 20px->18px)
  - [ ] Add visual.spec.ts Playwright/Vitest visual regression test
  - [ ] Fix DETERMINISTIC_CHECK paths from dashboard/src-frontend to dashboard-v2/src

- [B-12_shell_vs_workspace_decision](reports/B-plan/B-12_shell_vs_workspace_decision.report.md) — MOSTLY_DONE — Sidebar width 64px vs spec 48px; 7 icons vs spec 5; Ctrl+Shift+W unresolved
  - [ ] Fix Sidebar.svelte width: change from 64px to 48px if spec requires
  - [ ] Reconcile sidebar icon count: spec 5, current 7 — document or align
  - [ ] Decide on Ctrl+Shift+W: remove from WIRING_CONTRACT or document N/A

- [B-13_activity_feed_restoration](reports/B-plan/B-13_activity_feed_restoration.report.md) — MOSTLY_DONE — BubbleUpStrip shows placeholder "no events"; OrbitEvidenceGates mount not confirmed
  - [ ] Wire bubble_up_event SSB rows so BubbleUpStrip shows real events (not placeholder)
  - [ ] Verify OrbitEvidenceGates is imported into OrbitConcept.svelte or correct parent
  - [ ] Decide whether ActivityWindow concept-name header hover-reveal should be restored

- [B-14_live_token_streaming_chat](reports/B-plan/B-14_live_token_streaming_chat.report.md) — MOSTLY_DONE — Stop button missing in CommandWindow; AMBIGUOUS intent falls back to TASK without LLM
  - [ ] Add Stop button to CommandWindow.svelte when commands.isStreaming is true
  - [ ] Add LLM fallback for AMBIGUOUS intent or document TASK fallback as deliberate design
  - [ ] Add SsbActionType::ChatMessageStreamed to ssb.rs if cross-window sync desired

- [B-15_phase0_debt_burndown](reports/B-plan/B-15_phase0_debt_burndown.report.md) — PARTIAL — compact.rs/oauth.rs/prompt.rs (~1959 LOC dead code) not deleted; 50 SystemTime::now() sites not migrated; HashMap ban open
  - [ ] Delete crates/orchestrator/src/compact.rs (624 LOC) after confirming zero production callers
  - [ ] Delete crates/providers/src/oauth.rs (591 LOC) after confirming zero callers
  - [ ] Delete crates/orchestrator/src/prompt.rs (744 LOC) after confirming role_prompt wired
  - [ ] Replace SystemTime::now/Instant::now in ~50 production src files with ClockProvider
  - [ ] Replace std::collections::HashMap in 10 remaining production files with BTreeMap/FxHashMap
  - [ ] Re-enable entropy quarantine check in .github/workflows/windows.yml
  - [ ] Add instant_now() method to ClockProvider trait in core/src/clock.rs

- [B-16_anti_stale_enforcement](reports/B-plan/B-16_anti_stale_enforcement.report.md) — MOSTLY_DONE — check_anti_stale.sh not wired to CI; B-16B Chunk C unwrap sites pending approval; ort 2.0 waiver
  - [ ] Wire check_anti_stale.sh into GitHub Actions workflow (anti-stale.yml)
  - [ ] Execute B-16B Chunk C: ~12 .unwrap() sites in deterministic_auditor+dkg (user approval required)
  - [ ] Resolve B-16E-HIGH: self_discovery.rs L247/258/266 f64->u8 score clamping
  - [ ] Monitor crates.io for ort 2.0.0 stable; execute bump when published

- [B-17_ground_vacuous_gates](reports/B-plan/B-17_ground_vacuous_gates.report.md) — MOSTLY_DONE — Gate verdict-count aggregation missing; historical frequency view not in GatesPanel
  - [ ] Add per-gate verdict-count aggregation to get_evidence_gates or new command
  - [ ] Update GatesPanel.svelte to display per-gate PASS/BLOCK frequency (historical view)
  - [ ] Flip search_mode_cascade_pending_b4_wiring from #[ignore] when B-4 lands
  - [ ] Flip cli_interactive_gate_pending_b2_wiring from #[ignore] when B-2 wires CliInteractiveGate

- [B-18_tier_c_sandbox_activation](reports/B-plan/B-18_tier_c_sandbox_activation.report.md) — PARTIAL — B-18B setrlimit/prlimit64 calls not implemented; hidepid mount absent; IpcHookDispatcher not wired
  - [ ] Implement setrlimit calls in LinuxSandbox::launch_agent for CPU/memory/files/processes
  - [ ] Mount /proc with hidepid=2 inside CLONE_NEWNS namespace
  - [ ] Add PR_CAPBSET_DROP loop to drop all capabilities in clone_cb
  - [ ] Wire IpcHookDispatcher into run_agent_session in ipc_accept.rs
  - [ ] Extend test_4a_isolation_pid_namespace_real to read /proc/{pid}/status; assert NSpid!=1
  - [ ] Add Windows real isolation test using NtQueryInformationProcess

- [B-19_hook_system_unification](reports/B-plan/B-19_hook_system_unification.report.md) — PARTIAL — InProcessHookDispatcher not deleted; IpcHookDispatcher not wired to production; 10 DAIRA markers missing
  - [ ] Delete crates/orchestrator/src/hook_dispatcher.rs (InProcessHookDispatcher) after migration
  - [ ] Wire IpcHookDispatcher to production in runtime_setup.rs
  - [ ] Install ruff runner in production: instantiate RuffPostToolUseRunner
  - [ ] Implement AdaptiveTool<T> wrapper: auto-emit 10 remaining DAIRA markers
  - [ ] Implement MarkerReport struct and SSB storage per spec
  - [ ] Couple PreCompact hook findings to create_handoff trigger

- [B-20_tool_registry_unification](reports/B-plan/B-20_tool_registry_unification.report.md) — PARTIAL — IpcToolRegistry not wired; InProcessToolDispatcher still in production call sites; WIP squash missing
  - [ ] Wire IpcToolRegistry as production dispatcher in cli/src/run_subcommand.rs:149,221
  - [ ] Delete/stub-ban InProcessToolDispatcher from production after IpcToolRegistry wired
  - [ ] Implement WIP commit squash: >5 WIP commits -> 1 phase-level commit on AUDITING->MEMORIZING
  - [ ] Verify FailureCategory taxonomy alignment (14-variant vs 9-variant enum divergence)

- [B-21_phase_4F_feasibility_probe](reports/B-plan/B-21_phase_4F_feasibility_probe.report.md) — MOSTLY_DONE — SSB action type names mismatched; FeasibilityPanel.svelte missing; 3-failures escalation missing
  - [ ] Rename/alias SSB types to match plan: feasibility_probe_started, feasibility_probe_complete
  - [ ] Add FeasibilityPanel.svelte at dashboard-v2/src/lib/components/FeasibilityPanel.svelte
  - [ ] Wire d.unwired.feasibility=false in drawers.svelte.ts once FeasibilityPanel renders real data
  - [ ] Add 3-failures -> Escalate/Replan escalation path in task_runner or feasibility_probe
  - [ ] Confirm sysinfo dep for RAM/disk or document shell-out as approved approach

- [B-22_phase_4G_autonomous_mode](reports/B-plan/B-22_phase_4G_autonomous_mode.report.md) — MOSTLY_DONE — agents.{role}.backend stanza missing from elai_cc.toml; SSB action_type naming drift
  - [ ] Add [agents.{role}.backend] stanza to elai_cc.toml for local/api/hybrid
  - [ ] Reconcile SSB action_type names: auto_advance_decision vs actual names
  - [ ] Update B-22 plan rebaseline note: stub.ts get_auto_advance_stats line reference

- [B-23_phase_4H_self_modification](reports/B-plan/B-23_phase_4H_self_modification.report.md) — MOSTLY_DONE — SSB action_type names mismatched; nightly dep audit trigger not wired; cron script missing
  - [ ] Rename/alias SSB action_types to match WIRING_CONTRACT (self_mod_proposed etc.)
  - [ ] Wire nightly dep audit trigger from Layer1A DIVERGING state
  - [ ] Create scripts/self_mod_cron.sh
  - [ ] Confirm DarwinNet Gene Pool is conceptual only or implement gene_pool.rs

- [B-24_phase_4I_media_validator](reports/B-plan/B-24_phase_4I_media_validator.report.md) — PARTIAL — CLAP audio model, OllamaTier2, SglangTier2 not implemented; constrained decoding absent
  - [ ] Implement ClapAudioModel::load_model, embed_audio, embed_text in media_validator/clap.rs
  - [ ] Implement OllamaTier2::validate HTTP calls to ollama /api/generate in tier2.rs
  - [ ] Implement SglangTier2::validate HTTP calls to SGLang in tier2.rs
  - [ ] Add constrained decoding for VlmValidationOutput JSON schema enforcement in Tier 2
  - [ ] Create scripts/media_models_setup.sh for VLM model download opt-in

- [B-25_phase_4J_empire_integration](reports/B-plan/B-25_phase_4J_empire_integration.report.md) — NOT_DONE — Empire codebase not accessible; no implementation exists
  - [ ] Gain access to Empire codebase before any implementation
  - [ ] Create crates/orchestrator/src/empire/ module with client.rs/transport.rs/neuron.rs/repair.rs
  - [ ] Add empire SSB action types (empire_connected, empire_disconnected, empire_event_received etc.)
  - [ ] Implement ElaiNeuron MCP server with 4 capabilities + ToolFilter
  - [ ] Add cfg(feature="empire") gate; add EmpireBridgePanel.svelte
  - [ ] Wire --empire CLI flag in cli/src/args.rs
  - [ ] Create workflow/reasoning/B-25-decisions.md

- [B-26_phase_4K_6_to_13_dashboard_completion](reports/B-plan/B-26_phase_4K_6_to_13_dashboard_completion.report.md) — PARTIAL — webhook.ts missing; MonitorWindow static phase map; hamburger/responsive missing
  - [ ] CREATE dashboard-v2/src/lib/notifications/webhook.ts
  - [ ] MODIFY NotificationManager.svelte to import webhook.ts
  - [ ] MODIFY SettingsWindow.svelte: add webhook URL input field
  - [ ] MODIFY shortcuts.ts: add Ctrl+Shift+A (accept-all) and Ctrl+Shift+R (reject-all)
  - [ ] MODIFY MonitorWindow.svelte: replace static 5-phase map with get_plan_progress data
  - [ ] MODIFY Sidebar.svelte: add hamburger toggle shown below 1000px viewport width
  - [ ] MODIFY Workspace.svelte: auto-set secondaryCollapsed=true when viewport <800px

- [B-27_cloud_mode](reports/B-plan/B-27_cloud_mode.report.md) — MOSTLY_DONE — cloud_spend_summary stub returns zero; spend cap not enforced; CloudPanel.svelte missing
  - [ ] Implement SsbViewer::cloud_spend_summary() replacing hardcoded-zero stub in commands.rs:3306
  - [ ] Implement hard ceiling enforcement in CloudModeRouter::route() (currently deferred)
  - [ ] Implement warn_at_percent (80%) ceiling warning injection into COMMAND
  - [ ] Add CLI --mode cloud|hybrid|local flag to /task command
  - [ ] Create CloudPanel.svelte dashboard component per WIRING_CONTRACT
  - [ ] Implement pre-task cost estimation user dialog before cloud task submission
  - [ ] Create workflow/reasoning/B-27-decisions.md

- [B-28_focus_tunnel](reports/B-plan/B-28_focus_tunnel.report.md) — MOSTLY_DONE — StarCoder/DeepSeek supports_fim detection unverified; token-budget claim ungrounded without benchmark
  - [ ] Verify StarCoder/DeepSeek-Coder supports_fim=true detection in context_budget.rs
  - [ ] Add benchmark or deterministic test asserting token counts if savings claims are load-bearing

- [B-29_symbol_level_impact_api](reports/B-plan/B-29_symbol_level_impact_api.report.md) — MOSTLY_DONE — symbol_ tools not in ToolCatalog; update_changed_files_symbol_graph not in incremental path
  - [ ] Add ToolDefinition for symbol_impact/symbol_flow/symbol_context/symbol_list to build_tool_catalog in production_agent_rpc.rs:312
  - [ ] Wire update_changed_files_symbol_graph into indexing_service.rs incremental-update path
  - [ ] Add /symbol_impact input form to KnowledgeDrawer.svelte IMPACT ANALYSIS panel
  - [ ] Create crates/orchestrator/tests/symbol_query_dispatch.rs end-to-end IPC test

- [B-29.5_symbol_graph_scale_benchmark](reports/B-plan/B-29.5_symbol_graph_scale_benchmark.report.md) — MOSTLY_DONE — Scale fixtures not run; TOML sections for linux_kernel/chromium/vscode not populated
  - [ ] Run clone_scale_fixtures.sh for all 4 tiers; populate metrics/phase_b29_5_scale.toml
  - [ ] Create workflow/reasoning/B-29.5-decisions.md documenting hardware/p99 rationale
  - [ ] Add unresolved_edge_pct to scale.toml schema
  - [ ] Extend scale_determinism test to cover cross-codebase targets once fixtures indexed

- [B-30_multilang_dkg_hardening](reports/B-plan/B-30_multilang_dkg_hardening.report.md) — MOSTLY_DONE — Kotlin grammar not registered; svelte_extraction.rs test missing; multilang fixture absent
  - [ ] Add KotlinGrammar struct to crates/knowledge/src/dkg/mod.rs
  - [ ] Add "kt" and "kts" to ext_to_lang_id in dkg/mod.rs:3233
  - [ ] Register kotlin grammar in LanguageRegistry::default()
  - [ ] Create crates/knowledge/tests/svelte_extraction.rs
  - [ ] Create docs/fixtures/multilang_project/ with one source file per language

- [B-31_resumable_indexing_lock_lifecycle](reports/B-plan/B-31_resumable_indexing_lock_lifecycle.report.md) — MOSTLY_DONE — auto_resume_indexed_projects not wired to cli/main; SIGINT handler missing; Watch wiring absent
  - [ ] Wire auto_resume_indexed_projects into cli::main (crates/cli/src/main.rs)
  - [ ] Install SIGINT/SIGTERM signal handlers calling graceful_shutdown
  - [ ] Implement LockScope::Watch wiring in watcher service
  - [ ] Create tests/contracts/indexing_resilience_contract.rs
  - [ ] Implement migrateAbsolutePathKeys equivalent in knowledge::metadata

- [B-31.5_resumable_indexing_scale_benchmark](reports/B-plan/B-31.5_resumable_indexing_scale_benchmark.report.md) — MOSTLY_DONE — scale fixtures not run against real codebases; p50/p99 per-batch timing outputs 0
  - [ ] Run scripts/scale_test/run_all_tiers.sh against real ELAI_SCALE_FIXTURE_ROOT
  - [ ] Implement per-batch p50/p99 timing in run_tier_a.sh (currently outputs 0)
  - [ ] Run contract tests against production fixture metrics before declaring production-ready

- [B-32_minified_file_chunking](reports/B-plan/B-32_minified_file_chunking.report.md) — MOSTLY_DONE — SSB emit test missing; char-cap assertion missing; ELAI_SKIP_MINIFIED env opt-out not coded
  - [ ] Add test_minified_file_emits_ssb_action to knowledge/tests/chunking_minified.rs
  - [ ] Add test_apply_char_cap_called_in_every_branch to chunking_dispatch.rs
  - [ ] Implement ELAI_SKIP_MINIFIED=1 env opt-out in bm25.rs is_minified call site
  - [ ] Add IndexingProgressPanel Svelte display for "Detected N minified files" when SSB rows exist

- [B-33_interactive_graph_viewer](reports/B-plan/B-33_interactive_graph_viewer.report.md) — MOSTLY_DONE — /graph mermaid slash command missing; graph_export_dispatch test missing; decomposition-mode styles absent
  - [ ] Add /graph mermaid slash command to crates/cli/src/commands.rs
  - [ ] Wire export_graph_mermaid as dispatcher tool in crates/orchestrator/src/dispatcher.rs
  - [ ] Create crates/orchestrator/tests/graph_export_dispatch.rs
  - [ ] Add decomposition-mode cytoscape style selectors to GraphWindow.svelte
  - [ ] Add "Show blast radius" action to ActivityWindow.svelte file-write SSB row entries

- [B-34_entry_point_detection](reports/B-plan/B-34_entry_point_detection.report.md) — MOSTLY_DONE — /research oracle pre-seeding not wired; performance/selectivity tests missing
  - [ ] Implement /research Oracle pre-seeding: call detect_entry_points in SlashCommand::Research arm
  - [ ] Create crates/knowledge/tests/entry_points_performance.rs with <500ms assertion
  - [ ] Create crates/knowledge/tests/entry_points_payload_selectivity.rs
  - [ ] Add ELAI_DEBUG_PAYLOAD_LOAD=1 logging in TOOL_ENTRY_POINTS arm

- [B-35_context_artifacts_pipeline](reports/B-plan/B-35_context_artifacts_pipeline.report.md) — MOSTLY_DONE — index_all_artifacts not wired into IndexingService; TOOL_INDEX_ARTIFACTS stub returns Err
  - [ ] Wire index_all_artifacts into IndexingService::build_index_resumable after code phase
  - [ ] Implement TOOL_INDEX_ARTIFACTS in production_agent_rpc.rs (currently Err stub at line 1014)
  - [ ] Create crates/reasoning/tests/grounding_artifact_check.rs
  - [ ] Create crates/orchestrator/tests/designer_artifact_briefing.rs
  - [ ] Verify ActivityFeed renders ArtifactsIndexed/ArtifactSearched/ArtifactStale events

- [B-36_dashboard_v2_src_tauri_wiring](reports/B-plan/B-36_dashboard_v2_src_tauri_wiring.report.md) — MOSTLY_DONE — stub.ts ELAI_DASHBOARD_OFFLINE not explicit; planning-progress emit name mismatch; contract tests absent
  - [ ] Add ELAI_DASHBOARD_OFFLINE=1 flag gating to stub.ts (not always-on error fallback)
  - [ ] Expand one_shot_prompt signature to accept backend/port/allowed_tools
  - [ ] Rename planning-progress Tauri emit to web_research_step or document deviation
  - [ ] Add contract tests per D-command in crates/contracts/tests/dashboard_v2_<cmd>_contract.rs

- [B-37_multi_runtime_backend](reports/B-plan/B-37_multi_runtime_backend.report.md) — MOSTLY_DONE — /runtime fallback subcommand missing; list_runtimes/test_runtime Tauri name mismatch; Tier C isolation tests incomplete
  - [ ] Implement /runtime fallback <role> <b1>,<b2> subcommand
  - [ ] Rename list_runtimes->get_runtime_status and test_runtime->check_runtime_health or document deviation
  - [ ] Verify Tier C isolation contract tests cover all 7 CLI adapters

- [B-38_ambiguity_score_gate](reports/B-plan/B-38_ambiguity_score_gate.report.md) — MOSTLY_DONE — AmbiguityScorer not wired into live prompt-flow; get_ambiguity_state returns mock data
  - [ ] Wire AmbiguityScorer into task_runner.rs before GroundingSweepGate fires
  - [ ] Replace get_ambiguity_state mock data with real SSB row reads
  - [ ] Add ambiguity_scored/floor_failed/streak_advanced rendering to OrchestrationDrawer.svelte
  - [ ] Run cargo test -p elai_reasoning -- ambiguity + cargo test -p elai_contracts -- ambiguity

- [B-39_chat_reflect_pipeline](reports/B-plan/B-39_chat_reflect_pipeline.report.md) — MOSTLY_DONE — REFINE_CAP discrepancy (plan=2, code=3); elai-providers dep status unclear; test_payload coverage unverified
  - [ ] Verify test_payload() match arm covers ReflectTriaged/ReflectCandidate/ReflectAuditVerdict/ReflectRefined
  - [ ] Reconcile REFINE_CAP: plan says 2, code ships 3; update plan or document deviation
  - [ ] Clarify elai-providers dep status in reasoning/Cargo.toml (pre-B39 or added by B-39?)
  - [ ] Run cargo test -p elai-contracts -- chat_reflect to confirm all 16 tests pass

- [B-40_idea_framer_research_presets](reports/B-plan/B-40_idea_framer_research_presets.report.md) — DONE — All core types/functions/tests confirmed; minor plan doc stale references
  - [ ] Update plan spec line 15 to reflect actual SCHEMA_VERSION bump (40->41, not 28->29)
  - [ ] Update plan spec line 34-37 to replace bad arXiv:2407.21787 with arXiv:2409.04109

- [B-41_per_level_auditor_modes](reports/B-plan/B-41_per_level_auditor_modes.report.md) — MOSTLY_DONE — per_level_audit_dispatch not called from pipeline_tree_navigator.rs at 5 tier-completion points
  - [ ] Wire per_level_audit_dispatch::record_verdict into pipeline_tree_navigator.rs at 5 tier-completion points
  - [ ] Confirm auditor.md MODE DISPATCH has 6 mode-specific REVIEW prompt sections (~80 LOC each)
  - [ ] Confirm FramedIdea is passed as context to mode=Concepts auditor invocation

- [B-42_cosmos_artifact_tier](reports/B-plan/B-42_cosmos_artifact_tier.report.md) — MOSTLY_DONE — CosmosStage not wired into QueryPipeline; CosmosWorker not scheduled; SSB rows deferred to tracing
  - [ ] Wire CosmosStage into QueryPipeline between ArtifactStage and ARagStage
  - [ ] Wire CosmosWorker::tick() into task_runner.rs at hourly cadence
  - [ ] Emit actual SSB rows (CosmosRefreshStarted/CosmosArtifactUpdated/CosmosArtifactDecayed/CosmosGroundingUsed)
  - [ ] Create crates/knowledge/tests/cosmos_worker_tick.rs (~200 LOC)
  - [ ] Create crates/knowledge/tests/cosmos_pipeline_stage.rs (~180 LOC)
  - [ ] Clarify decay threshold: plan says 2x cadence, code uses 3x; align docs or update plan

---

### Guide (18 files)

- [anti-staleness-enforcement](reports/guide/anti-staleness-enforcement.report.md) — MOSTLY_DONE — ConstraintsBlock auto-population not wired; dep-audit automation not scheduled; modern_api.npz missing
  - [ ] Wire ConstraintsBlock auto-population from rules.db deprecated_patterns table
  - [ ] Implement periodic dep-audit automation (Phase 4H trigger or OS scheduler)
  - [ ] Compute and commit modern_api.npz via metrics/compute_steering_vector.py batch mode
  - [ ] Implement pruning for freshness_meta.db: add prune_below_min_decay() to FreshnessDb
  - [ ] Implement Layer 1 reasoning fix templates using deprecated_patterns

- [cloud-mode-design](reports/guide/cloud-mode-design.report.md) — MOSTLY_DONE — CLI per-task --mode flag missing; HYBRID escalation retry loop missing; spend cap hard enforcement deferred
  - [ ] Implement CLI per-task --mode flag in crates/cli/src/args.rs; bridge to BackendConfig
  - [ ] Wire HYBRID escalation retry loop in task_runner.rs: after local retries emit EscalateToCloud
  - [ ] Implement spend cap hard enforcement in CloudModeRouter::route_cloud
  - [ ] Replace get_cloud_spend stub with real SSB aggregation (sum today's CloudSpend rows)
  - [ ] Add pre-task cost estimation display ("$X-$Y. Proceed?") before first cloud call
  - [ ] Extend CloudPrivacyToml with blocked_paths/allow_source_upload/redact_env_vars

- [code-completeness-enforcement-design](reports/guide/code-completeness-enforcement-design.report.md) — PARTIAL — FilePlan/FileSpec/SkeletonGenerator/MiniCritic/BridgeTemplate all absent; FILE_PLAN not in BabyStepSpec
  - [ ] Define FilePlan/FileSpec/FileModSpec/FunctionSpec structs; add FILE_PLAN as 9th BabyStepSpec block
  - [ ] Implement FileLineTracker in TreeNavigator with cumulative line count per baby step
  - [ ] Implement skeleton-first generation with fill protocol (SkeletonGenerator, BLANK_N placeholders)
  - [ ] Implement mini-critic between fills (MiniCriticResult, tree-sitter fragment parse)
  - [ ] Implement AcceptAndComplete recovery (TriageResult, completeness_ratio >= 0.5)
  - [ ] Wire MistakeBook recording from T1/T2 BLOCK events into SSB

- [compiled-pipeline-design](reports/guide/compiled-pipeline-design.report.md) — PARTIAL — CompiledPipeline/TemplateCrystallizer/BackgroundTaskManager absent; constrained decoding stub in phase_d_extract_facts
  - [ ] Create CompiledPipeline/CompiledStep/TemplateVariable types in orchestrator/src/
  - [ ] Create TemplateCrystallizer with N=3 consecutive success threshold
  - [ ] Create CompiledExecutor with decompilation logic
  - [ ] Create BackgroundTaskManager with BACKGROUND_TASK_STARTED/COMPLETE SSB types
  - [ ] Wire constrained decoding into phase_d_extract_facts (upgrade from stub to real LLM call)
  - [ ] Create CumulativeTokenTracker; wire into cloud provider response parsing

- [dashboard-design](reports/guide/dashboard-design.report.md) — MOSTLY_DONE — ExploreWindow missing; Ctrl+Shift+H stub; per-hunk accept/decline missing; responsive breakpoints absent
  - [ ] Add EXPLORE window: create ExploreWindow.svelte; register 'explore' in WindowType and SHORTCUTS
  - [ ] Implement window recovery history: closedTabs stack; wire Ctrl+Shift+H to recovery modal
  - [ ] Add Ctrl+Enter send binding in CommandWindow.svelte textarea
  - [ ] Add Ctrl+Shift+A/R in shortcuts.ts wired to ReviewWindow
  - [ ] Add responsive CSS breakpoints: sidebar hamburger <1000px, auto-collapse <800px
  - [ ] Add per-hunk accept/decline controls to ReviewWindow.svelte

- [deterministic-harness-design](reports/guide/deterministic-harness-design.report.md) — PARTIAL — LINTER_GATE/CONVENTION_GATE Predicate variants absent; domain rule packs not implemented; elai rules CLI missing
  - [ ] Add LINTER_GATE and CONVENTION_GATE variants to Predicate enum
  - [ ] Create lint_cache.db schema and evaluation logic (blake3-keyed per file)
  - [ ] Create domain_rule_packs table in rules.db schema
  - [ ] Implement 5 built-in domain rule packs (trading, gamedev, security, ml, web)
  - [ ] Implement CLI commands: elai rules import <path> and elai rules install <pack_name>
  - [ ] Reduce MICRO_SLOT_RULES from 150 to 50-80 tokens target

- [development-stones-analysis](reports/guide/development-stones-analysis.report.md) — MOSTLY_DONE — Progressive indexing BM25_READY gate missing; git-aware file prioritization absent; BYOE not implemented
  - [ ] Implement staged progressive indexing: expose BM25_READY before DKG completes
  - [ ] Implement git-aware file prioritization in IndexingService.build_index_resumable
  - [ ] Add BYOE support: user-configured base_url override in CloudConfig
  - [ ] Wire adaptive futility threshold from SSB per-error-category retry success rates
  - [ ] Implement weaker evidence gates for zero-test codebases

- [focus-tunnel-design](reports/guide/focus-tunnel-design.report.md) — MOSTLY_DONE — StreamingValidator not wired; focus_tunnel_pre_assembler Arc unpopulated; ReplanRequest::Minor not wired
  - [ ] Wire StreamingValidator into run_fim_step_with_provider and run_diff_step_with_provider
  - [ ] Populate focus_tunnel_pre_assembler Arc in production TaskRunnerConfig
  - [ ] Populate focus_tunnel_dep_resolver Arc in production TaskRunnerConfig
  - [ ] Wire ReplanRequest::Minor emission from SpeculativeFileRouter into Fsm::submit_replan

- [implementation-guide](reports/guide/implementation-guide.report.md) — MOSTLY_DONE — DispatchContext name mismatch; proptest 10K-op config absent; Layer3Processor stub absent
  - [ ] Rename/alias DispatchContext+RoleDispatch or update guide spec to match actual names
  - [ ] Add explicit 10,000-op proptest config in dst.rs or dst_reference_model.rs
  - [ ] Implement or stub Layer3Processor for Phase 4H self-modification
  - [ ] Verify scripts/stage_onnx_models.py uses blake3 (not blake2) for hash verification

- [pipeline-modes-design](reports/guide/pipeline-modes-design.report.md) — PARTIAL — SurgicalPlan/SurgicalDecomposer absent; project detection signal logic absent; Oracle cache TTL absent
  - [ ] Implement SurgicalPlan/AffectedSet/ComplexityClass/RegressionGate/TestBaseline types
  - [ ] Implement SurgicalDecomposer: DKG blast radius traversal
  - [ ] Wire surgical flag through parse_task_flags to dispatcher
  - [ ] Add project detection signal logic (weighted-signal table)
  - [ ] Add Oracle Mode answer-level cache with tier-based TTL (T1=30d, T2=7d, T3=1d)
  - [ ] Expose DKG query tools to agents (dkg_callers/dkg_dependents/dkg_data_flow)

- [post-phase4-implementation-order](reports/guide/post-phase4-implementation-order.report.md) — PARTIAL — H-1 WiringContract types absent; G-1 CumulativeTokenTracker absent; F-1 adaptive model selection absent
  - [ ] Implement H-1: WiringContract/StubBanEntry/ObservableEffect/WiringEvidence/WiringViolation types
  - [ ] Implement H-1: DkgReachabilityAuditor/StubBanAuditor with SQL-backed DKG path queries
  - [ ] Implement G-1: CumulativeTokenTracker with prompt/completion tokens; wire to provider response.usage
  - [ ] Implement G-2: CompiledPipeline/TemplateCrystallizer; G-3: CompiledExecutor
  - [ ] Implement F-1: adaptive model selection; F-2: 4-phase progressive indexing
  - [ ] Implement C-1: ExploreWindow.svelte routing queries through Oracle Mode pipeline

- [project-intelligence-design](reports/guide/project-intelligence-design.report.md) — MOSTLY_DONE — DependencyProfile not populated; few-shot DKG examples not wired into prompt assembly; LLM Tier3 not implemented
  - [ ] Populate DependencyProfile in detect_project_profile (parse Cargo.toml/package.json)
  - [ ] Wire few-shot DKG examples into prompt assembly via build_telescope_block
  - [ ] Implement LLM-assisted Tier 3 vision or document as out-of-scope
  - [ ] Create workflow/reasoning/phase-3L-decisions.md
  - [ ] Add adversarial tests: Windows abs paths, large monorepo >100 members, vendored deps

- [reasoning-architecture-design](reports/guide/reasoning-architecture-design.report.md) — MOSTLY_DONE — AB-MCTS executor absent; CloudTier enum absent; Layer 5 diagnostic-only path absent
  - [ ] Implement AB-MCTS executor in crates/reasoning/src/tree_search.rs
  - [ ] Add CloudTier enum (Haiku/Sonnet/Opus) and CloudDiagnosticRequest/Response types
  - [ ] Implement Layer 5 diagnostic-only cloud call path separate from Layer 6 generation
  - [ ] Add CloudSpendTracker with per_task_max_generations enforcement
  - [ ] Implement offline Layer 5 model-swap execution using SwapStrategy::SwapModel

- [testing-trust-design](reports/guide/testing-trust-design.report.md) — NOT_DONE — 14 testing trust tiers (T-(-1) through T-13) entirely unimplemented
  - [ ] Implement T-(-1): DeterminismConfig type + hardware-class lock + blake3 prefix-cache keys
  - [ ] Implement T-0: WiringContract type + StubBanAuditor + DkgReachabilityAuditor + ObservableEffectRunner
  - [ ] Implement T-1: MutationRunner wrapping cargo-mutants + mutmut; add Mutation tier to ValidationTier
  - [ ] Implement T-2: IntentPair/IntentPairSet/IntentConfirmation; INTENT_CONFIRMED SSB type
  - [ ] Implement T-3: Metamorphic relation seed library in rules.db
  - [ ] Implement T-4 through T-13 progressively per spec tier ordering
  - [ ] Add [wiring.entrypoints] section to elai_cc.toml with closed production entrypoint set

- [token-efficiency-design](reports/guide/token-efficiency-design.report.md) — PARTIAL — ConstrainedDecodingConfig absent; RetryStrategy/EscalationChain absent; benchmark fixture bug (0% measurements 3/4/5)
  - [ ] Define ConstrainedDecodingConfig struct and DecodingMode enum in providers crate
  - [ ] Implement CRANE-style adaptive grammar or vLLM guided-decoding via outlines
  - [ ] Define RetryStrategy struct and wire to BubbleUpPolicy
  - [ ] Implement futility detection in NavigatorTree (same-error 3x, output-similarity >90%)
  - [ ] Fix benchmark harness fixture validation bug (measurements 3/4/5 report 0%)
  - [ ] Implement or wire PreloadManager.trigger_preload() to actual background loading

- [update-implementation-order](reports/guide/update-implementation-order.report.md) — MOSTLY_DONE — B-1/B-2/B-3 constrained decoding unimplemented; 4D/4E design additions absent; maintenance automation not wired
  - [ ] Implement B-1 CONSTRAINED DECODING: ConstrainedDecodingConfig/DecodingMode/grammar-guided gen
  - [ ] Implement B-2 CONSTRAINED DECODING FOR IMPORTS via JSON Schema/EBNF allowlist
  - [ ] Add DESIGN ADDITIONS TO 4B: FailureTaxonomy 3 new categories in failure_taxonomy.rs
  - [ ] Add DESIGN ADDITIONS TO 4E: FileLineTracker/skeleton-first/fill protocol/MiniCritic
  - [ ] Implement B-3 MAINTENANCE AUTOMATION: scheduled nightly trigger for freshness + steering vector

- [update-implementation-order-summary](reports/guide/update-implementation-order-summary.report.md) — MOSTLY_DONE — A-26 Tool Routing Layer absent; A-2 T1Hook rules.db loading unverified
  - [ ] Implement A-26 Tool Routing Layer: interceptor rejecting bash cat/sed/echo with hints (~150-200 LOC)
  - [ ] Verify A-2 T1Hook::new() loads rules from rules.db at construction (not just via set_rules)
  - [ ] Confirm B-1/B-2/B-3 scope or mark explicitly deferred with phase-4 dependency note

- [update-phase-map](reports/guide/update-phase-map.report.md) — PARTIAL — phase-3L.txt/phase-4D-SURGICAL.txt/phase-4D-ORACLE.txt not created; 10+ phase files need updates
  - [ ] Create steps/phase-3L.txt (8-step Project Intelligence spec)
  - [ ] Create steps/phase-3J-BASELINE.txt (Token Efficiency benchmark harness spec)
  - [ ] Create steps/phase-4D-SURGICAL.txt and steps/phase-4D-ORACLE.txt
  - [ ] Update steps/phase-1A.txt: add extract_conventions() API and ConventionStats types
  - [ ] Update steps/phase-3J.txt: add ETCLogitsProcessor/GG signals/constrained decoding/tool routing
  - [ ] Add LAZY_CODE/EMPTY_BODY/FILE_SIZE_EXCEEDED variants to FailureCategory enum

---

### Steps (40 files)

- [pre-phase-0](reports/steps/pre-phase-0.report.md) — MOSTLY_DONE — rusty-claude-cli exclusion unresolved; test_total field missing from pre_phase_0.toml
  - [ ] Determine whether rusty-claude-cli was intentionally excluded; apply P0.1 cfg-gates if exists
  - [ ] Add test_total field to metrics/pre_phase_0.toml
  - [ ] Add cfg_gated_tests list to pre_phase_0.toml documenting gated tests

- [phase-0](reports/steps/phase-0.report.md) — MOSTLY_DONE — compact.rs/prompt.rs/usage.rs dead code survives; migration_manifest.toml incomplete; determinism_proof.sh scoped wrong
  - [ ] Remove compact.rs (compact_session/CompactionConfig are billing concepts)
  - [ ] Strip UsageTracker/format_usd/ModelPricing from core/src/usage.rs
  - [ ] Add per-command KEEP/STRIP breakdown for 28+ slash commands to migration_manifest.toml
  - [ ] Fix determinism_proof.sh: use --test seeded_determinism not --workspace
  - [ ] Resolve SeededClock ELAI_SEED: clock starts at tick 0, only RNG seeded
  - [ ] Document or close elai_cc.toml ConfigLoader gap

- [phase-1A](reports/steps/phase-1A.report.md) — MOSTLY_DONE — SymbolKind::Use/Other variants missing; concept=NULL default test not confirmed
  - [ ] Add SymbolKind::Use and SymbolKind::Other to knowledge/src/lib.rs or document substitution
  - [ ] Verify concept column test exists in dkg_tests.rs (concept=NULL default assertion)

- [phase-1B](reports/steps/phase-1B.report.md) — DONE — BM25 implementation fully confirmed
  - [ ] (Optional) Confirm all 12 bm25_tests pass: cargo test -p knowledge --test bm25_tests

- [phase-1C](reports/steps/phase-1C.report.md) — MOSTLY_DONE — vector_cache lacks file_path column; invalidate_file returns NotSupported; measurements missing
  - [ ] Extend vector_cache schema: add file_path TEXT column; implement invalidate_file()
  - [ ] Register SSB file_write event handler to call cache.invalidate_file() on file_write
  - [ ] Execute Step 5 MEASURE: embed 100 chunks; write phase-1C-measurements.md with 4 entries
  - [ ] Decide on content_hash vs content_blake2s column naming (spec vs implementation)

- [phase-1D](reports/steps/phase-1D.report.md) — DONE — PageRank fully confirmed; optional naming and metric tighten only
  - [ ] Optional: rename PageRankNodeInfo (cosmetic)
  - [ ] Confirm cargo test --package knowledge -- pagerank with PROPTEST_CASES=100

- [phase-1E](reports/steps/phase-1E.report.md) — MOSTLY_DONE — measure_1e.rs in examples/ not src/bin/; metrics reflect degraded-mode only
  - [ ] Move measure_1e.rs from examples/ to src/bin/ to satisfy DETERMINISTIC_CHECK path
  - [ ] Re-run measure_1e with vector module configured so stages_skipped is empty

- [phase-1F](reports/steps/phase-1F.report.md) — MOSTLY_DONE — Phase dropped at 1F.2 (NOT_AVAILABLE); decision doc incomplete; metrics TOMLs absent
  - [ ] Add phase_1F_drop.toml to metrics/ recording the DROP outcome
  - [ ] Add decision note to workflow/reasoning/phase-1F-decisions.md documenting NOT_AVAILABLE

- [phase-1G](reports/steps/phase-1G.report.md) — MOSTLY_DONE — ONNX model files not staged; phase_1G.toml absent; SSB recovery event on circuit breaker close is stub
  - [ ] Create scripts/quantize_cross_encoder.py with onnxruntime INT8 quantization
  - [ ] Stage ONNX models at models/onnx/cross-encoder-l12.onnx and tokenizer.json
  - [ ] Commit metrics/phase_1G.toml after optimization_sweep run
  - [ ] Add SSB recovery event in emit_recovery_if_needed() (currently no-op stub)

- [phase-1H](reports/steps/phase-1H.report.md) — MOSTLY_DONE — cli/Cargo.toml forces splade feature unconditionally; latency assertion weak (eprintln only)
  - [ ] Fix cli/Cargo.toml: change features=["splade"] to no features; gate import behind cfg(feature="splade")
  - [ ] Add hard assert in splade_tests.rs for latency <2000ms (not just eprintln)

- [phase-1I](reports/steps/phase-1I.report.md) — MOSTLY_DONE — canary reference vector storage canonical location not decided; FSM UNAVAILABLE gates incomplete
  - [ ] Decide canonical storage for canary_reference_vec (canary_vec.bin or rules.db)
  - [ ] Add FSM gates for UNAVAILABLE on transitions beyond IDLE->PLANNING
  - [ ] Add acceptance test for fresh ONNX session on HALF_OPEN probe

- [phase-2A](reports/steps/phase-2A.report.md) — MOSTLY_DONE — GrcEngine has 10 methods vs spec 6; elai-providers dep in reasoning/ needs justification
  - [ ] Run cargo test -p elai-reasoning at current HEAD to confirm 0 failures
  - [ ] Decide whether GrcEngine 10-method count is acceptable or spec needs update
  - [ ] Decide whether elai-providers dep in reasoning/ needs feature flag

- [phase-2B](reports/steps/phase-2B.report.md) — MOSTLY_DONE — elai-tools dep missing from reasoning/Cargo.toml; I5 auto-resolution in think() absent; CargoTestRunner stub
  - [ ] Add elai-tools dep to reasoning/Cargo.toml or document FileReader trait as accepted alternative
  - [ ] Implement I5 auto-resolution in think(): attempt FILE_CHECK/SOURCE_CHECK for Uncertain
  - [ ] Verify or implement CargoTestRunner in tools/ and inject into LocalVerifier

- [phase-2C](reports/steps/phase-2C.report.md) — MOSTLY_DONE — routing_overhead_pct=28.5 exceeds <10% spec target; bench needs stable clock or spec update
  - [ ] Resolve routing_overhead_pct=28.5 vs <10% spec target (re-run, amend spec, or restructure bench)

- [phase-2D](reports/steps/phase-2D.report.md) — DONE — All 62 reasoning tests pass per metrics file
  - [ ] Run cargo test -p reasoning -- --nocapture to confirm all 62 tests still pass

- [phase-3A](reports/steps/phase-3A.report.md) — DONE — FSM fully confirmed; RESEARCHING state formally documented
  - [ ] Optional: update orchestrator.txt §2 to document RESEARCHING formally if not already done

- [phase-3B](reports/steps/phase-3B.report.md) — MOSTLY_DONE — test_payload_mismatch test missing; proptest round-trip test missing; determinism test missing
  - [ ] Add test_payload_mismatch test to ssb.rs tests module
  - [ ] Confirm or add property-based tests (seq monotonic, msgpack round-trip, conflict symmetry)
  - [ ] Confirm determinism test exists (byte-identical DB output with fixed-seed ClockProvider)

- [phase-3C](reports/steps/phase-3C.report.md) — MOSTLY_DONE — In-process heartbeat not implemented; interactive human gate for spiral escalation not wired; EscalationAction name mismatch
  - [ ] Implement in-process heartbeat for Tier A agents (tokio::sync::watch channel)
  - [ ] Wire interactive human gate for spiral escalation: pause + surface evidence to CLI/dashboard
  - [ ] Rename EscalationAction::Terminate -> EscalationAction::Escalate per spec

- [phase-3D](reports/steps/phase-3D.report.md) — MOSTLY_DONE — recover_from_checkpoint() does not restore channel state; checkpoint_roundtrip test missing
  - [ ] Implement actual crash recovery in recover_from_checkpoint() (restore LastValue/Topic/Ephemeral)
  - [ ] Add checkpoint_roundtrip test: write BspCheckpoint; serialize/deserialize; assert byte-identical
  - [ ] Add 5+ superstep integration test covering all channel types through lifecycle
  - [ ] Add [orchestrator.bsp] section to elai_cc.toml with untracked_memory_limit_bytes

- [phase-3E](reports/steps/phase-3E.report.md) — MOSTLY_DONE — providers::token_count_estimate not wired into context_budget; CacheBreakpoint.valid not consumed by provider
  - [ ] Wire providers::token_count_estimate into context_budget compute path
  - [ ] Wire CacheBreakpoint.valid to provider prefix cache (update_breakpoints result consumed)
  - [ ] Replace hardcoded MODEL_MAX_128K/MODEL_MAX_4K with ModelCapabilityProfile-driven values

- [phase-3F](reports/steps/phase-3F.report.md) — MOSTLY_DONE — Global seed loader missing; engineering/security domain seed TOMLs absent; criterion benchmark absent
  - [ ] Create global seed loader reading from seed/*.toml; insert rules with status=ACTIVE source=SEED
  - [ ] Add engineering domain seed TOML with >=10 rules (no-.unwrap(), tracing, etc.)
  - [ ] Add security domain seed TOML with mutable=0 rules
  - [ ] Add [rules.domain_mapping] config key to elai_cc.toml
  - [ ] Create crates/rules/benches/eval_latency.rs with Criterion

- [phase-3G](reports/steps/phase-3G.report.md) — MOSTLY_DONE — CoherenceReport violations not wired to SSB FindingPayload; integration test assertion missing
  - [ ] Wire CoherenceReport violations to SSB FindingPayload writes after run_coherence_checks()
  - [ ] Add integration test: verify SSB Finding entries appear after coherence violations in lifecycle_full_5phase_integration

- [phase-3H](reports/steps/phase-3H.report.md) — MOSTLY_DONE — parent_chain returns tip-to-root not root-to-tip; batch insert test uses 500ms not 100ms spec
  - [ ] Fix parent_chain to return root-to-tip order in session_dag.rs:861
  - [ ] Add exact boundary test: 85% -> handoff fires; 84.9% -> no handoff
  - [ ] Add 100-run determinism test for snap_ids and trimming
  - [ ] Change batch insert performance test from <500ms to <100ms per spec

- [phase-3I](reports/steps/phase-3I.report.md) — MOSTLY_DONE — madsim dev-dep not added; VCR cassette layer missing; TraceOps absent; SQLite VFS spike not evaluated
  - [ ] Add madsim v0.2.34 dev-dep with cfg(madsim) gate OR document DstIpcGateway stub justification
  - [ ] Write madsim spike test or document Windows incompatibility with failure evidence
  - [ ] Implement VCR cassette layer (HTTP transport intercept, blake3 hash, msgpack storage)
  - [ ] Complete SQLite custom VFS spike evaluation

- [phase-3J](reports/steps/phase-3J.report.md) — MOSTLY_DONE — DolaDecoder absent; MCP bridge stubs not real; CliInteractiveGate not wired; EmptyDkgQuerier in production
  - [ ] Implement DolaDecoder/DolaConfig in providers/src/layer_config/dola.rs or document permanently superseded
  - [ ] Replace DefaultMcpBridge stubs with real stdio/SSE/HTTP transport
  - [ ] Wire CliInteractiveGate into ConversationRuntime (runtime_builder.rs)
  - [ ] Replace EmptyDkgQuerier in runtime_builder.rs:228,233 with DkgQuerierAdapter
  - [ ] Run vLLM latency benchmark; populate vllm_spike.toml and phase_3J.toml latency fields

- [phase-3K](reports/steps/phase-3K.report.md) — MOSTLY_DONE — Heading/Paragraph ChunkKind variants missing; AragOrchestrator::semantic_search uses embedding-norm proxy
  - [ ] Add Heading and Paragraph variants to ChunkKind enum in bm25.rs
  - [ ] Wire real query embedding into AragOrchestrator::semantic_search (dot-product vs norm proxy)
  - [ ] Add regression-gate assertion in benchmark_eval_improved.rs (delta_pct >= -5.0 per tier)

- [phase-index-pre0-2D](reports/steps/phase-index-pre0-2D.report.md) — MOSTLY_DONE — Phase 1F decision doc missing; TomlConfigLoader stub comment unresolved
  - [ ] Write phase-1F decision doc with measured throughput ratios and ADOPT/DROP outcome
  - [ ] Verify TomlConfigLoader stub at reasoning/src/config.rs:30 re: full TOML section parsing

- [phase-index-3A-4J](reports/steps/phase-index-3A-4J.report.md) — MOSTLY_DONE — 4J Empire not started; 3I proptest-state-machine dep missing; 3K AST boundary chunking unverified
  - [ ] 4J: Implement Empire Integration starting with FactoryLine protocol client
  - [ ] 3I: Add proptest-state-machine dep; add stateful PBT harness in dst_proptest.rs
  - [ ] 3K: Verify chunk_file uses tree-sitter AST boundaries; fix if fixed-window in use

- [phase-4A](reports/steps/phase-4A.report.md) — MOSTLY_DONE — ReplanRequest missing evidence field; setrlimit calls missing; 10+ adversarial tests missing
  - [ ] Add evidence field to IpcMessageType::ReplanRequest in ipc-types/src/lib.rs
  - [ ] Implement setrlimit RLIMIT_NPROC=1/RLIMIT_FSIZE=100MB/RLIMIT_AS=512MB in sandbox.rs
  - [ ] Add /proc hidepid=2 bind-mount in Linux child namespace setup
  - [ ] Add test_4a_adv_ipc_u32_max_length_rejected (length=u32::MAX -> PayloadTooLarge)
  - [ ] Add test_4a_adv_selfmod_path_traversal, bash_write_locked, git_commit_locked, symlink_resolved
  - [ ] Add test_4a_adv_replan_minor_fast_track and test_4a_adv_replan_major_human_gate

- [phase-4B](reports/steps/phase-4B.report.md) — MOSTLY_DONE — force_full/tier_hint fields missing in IPC types; DAIRA markers structural; AdaptiveTool<T> absent
  - [ ] Add force_full: bool field to FileReadRequest in ipc-types/src/lib.rs
  - [ ] Add tier_hint: Option<u8> field to EditRequest in ipc-types/src/lib.rs
  - [ ] Implement structured DAIRA markers in TraceHandler ([CALL]/[RETURN]/[MUTATION] etc.)
  - [ ] Add AdaptiveTool<T> generic wrapper in tools crate with max 3 scope reductions
  - [ ] Create metrics/phase_4B.toml once 4B.5 MEASURE is executed

- [phase-4C](reports/steps/phase-4C.report.md) — MOSTLY_DONE — phase_4C.toml missing 9 metric sections; criterion bench missing; adversarial tests missing
  - [ ] Add 9 quantified metric sections to metrics/phase_4C.toml
  - [ ] Create plugins/benches/tiered_analysis_bench.rs using Criterion
  - [ ] Add path traversal adversarial test (Unix "../../../etc/passwd" + Windows variant)
  - [ ] Add T1 rule-count stress test (1000 AST_PATTERN rules, p99 <1ms)
  - [ ] Create workflow/reasoning/phase-4C-decisions.md

- [phase-4D](reports/steps/phase-4D.report.md) — MOSTLY_DONE — VRAM tier TOML section missing; adversarial tests absent; SystemConstraintsRequired struct fields unverified
  - [ ] Add [research.vram_tiers] TOML section to elai_cc.toml with all 5+ tier entries
  - [ ] Add adversarial tests: filesystem-escape, direct-network-access, GRC-budget-exhaustion
  - [ ] Verify SystemConstraintsRequired struct includes avg_file_tokens/model_max_context
  - [ ] Resolve InferenceRequest: add to ipc-types or document existing sidecar RPC satisfies it

- [phase-4E](reports/steps/phase-4E.report.md) — MOSTLY_DONE — assembly_time_ms all 0 in phase_4E.toml; BabyStepOutcome variant names mismatch; on_concept_fail field absent
  - [ ] Run cargo run --example measure_4e_tree to populate phase_4E.toml decomp_tree section
  - [ ] Rename BabyStepOutcome variants to match spec (Pass/FailRetry/FailBubbleUp/Skipped/Deferred)
  - [ ] Add on_concept_fail: BubbleUpAction field to BubbleUpPolicy
  - [ ] Add build_decomposition_tree and assemble_baby_step_context stubs per 4E.7

- [phase-4F](reports/steps/phase-4F.report.md) — PARTIAL — SystemProbeReport name mismatch; GpuInfo.uuid missing; ResourceDelta absent; 8 adversarial tests missing; 6 metrics unmeasured
  - [ ] Rename SystemProbeReport -> SystemProbeResult (DETERMINISTIC_CHECK grep)
  - [ ] Add GpuInfo.uuid field and shell-out parsing
  - [ ] Add ResourceDelta struct (vram/ram before/after bytes)
  - [ ] Wire production construction paths to pass real DefaultFeasibilityProber
  - [ ] Author 8 adversarial tests (A1-A8) per 4F.4 TASK
  - [ ] Measure and populate 6 unmeasured metric fields in phase_4F.toml

- [phase-4G](reports/steps/phase-4G.report.md) — PARTIAL — AutoAdvanceEvaluator/EscalationPolicy/StagnationDetection/GoalDriftCanary all absent; persona agent files missing
  - [ ] Define AutoAdvanceEvaluator with gate_checks: HashMap<FsmTransition, EvidenceCheck>
  - [ ] Define EscalationPolicy struct (anti_spiral_threshold/high_risk_pauses/max_auto_advances)
  - [ ] Implement StagnationPattern enum + StagnationDetection + ExecutionHistory
  - [ ] Implement GoalDriftCanary with Jaccard compute() and 3-band verdict logic
  - [ ] Create 5 persona agent files: persona-hacker/researcher/simplifier/architect/contrarian.md
  - [ ] Wire TreeAutoAdvancePolicy into cli/main.rs production construction (currently autonomous_gate: None)
  - [ ] Add 10 SSB stagnation/lateral_thinking action types
  - [ ] Populate metrics/phase_4G.toml with 4G.6 runtime measurements

- [phase-4H](reports/steps/phase-4H.report.md) — MOSTLY_DONE — AllowedConfigKey has only generic Parameter(String); SWE-bench fixtures missing; git pre-commit hook missing
  - [ ] Replace AllowedConfigKey::Parameter(String) with closed enum of 12 named Parameter variants
  - [ ] Create benchmarks/swe_bench/ with at least 10 SWE-bench task fixtures
  - [ ] Create benchmarks/trigger_task.toml with known-answer canary task
  - [ ] Implement git-level enforcement: pre-commit hook rejecting LOCKED_KEYS file modifications
  - [ ] Wire dep-audit output to FullPipelineOutcome.stale_api_catches
  - [ ] Wire steering vector refresh after deprecated_patterns Layer 1A promote

- [phase-4I](reports/steps/phase-4I.report.md) — MOSTLY_DONE — RCE metric key paths mismatched; vlm_sandbox_escape_blocked blocked by B-22; comfyui test skipped
  - [ ] Rename RCE metric keys in phase_4I.toml to match spec L592-593 exactly
  - [ ] Resolve a5 vlm_sandbox_escape_blocked (blocked by B-22 sandbox) or document
  - [ ] Resolve a7 comfyui_timeout_cleanup: enable on non-Windows CI or document skip rationale

- [phase-4J](reports/steps/phase-4J.report.md) — NOT_DONE — Empire Integration entirely unimplemented; requires Empire codebase access
  - [ ] Confirm Empire codebase accessible at elai_cc.toml [empire.codebase_path]
  - [ ] Create crates/empire/ directory; add elai_empire as workspace member
  - [ ] Implement FactoryLineTransport trait with UnixTransport + TcpTransport
  - [ ] Implement FactoryLineClient with heartbeat loop and reconnect backoff
  - [ ] Implement all Empire wire types (FactoryLineCommand/Response, IpcMessage, BrainManifest)
  - [ ] Write all 4J.3-4J.7 tests (TDD first)
  - [ ] Create metrics/phase_4J.toml with >=40 tests and all metric fields

- [phase-4K](reports/steps/phase-4K.report.md) — MOSTLY_DONE — 6 Tauri commands missing (get_phase_progress, get_recent_actions, get_file_changes, get_action_detail, list_projects, create_project); ReviewWindow unwired
  - [ ] Add get_phase_progress Tauri command wrapping viewer.phase_progress()
  - [ ] Add get_recent_actions(limit, offset) Tauri command
  - [ ] Add get_file_changes(session_id) and get_action_detail(action_id) Tauri commands
  - [ ] Add list_projects() and create_project() Tauri commands
  - [ ] Wire ReviewWindow.svelte to live SSB data via get_action_detail + submit_intervention_response
  - [ ] Adjust polling interval from 5000ms to 500ms per spec

- [phase-4M](reports/steps/phase-4M.report.md) — MOSTLY_DONE — /runtime fallback subcommand missing; 3 Tauri commands not wired; runtime telemetry not populated
  - [ ] Implement /runtime fallback <role> <b1,b2> subcommand
  - [ ] Wire list_runtimes/test_runtime/set_role_runtime Tauri commands in lib.rs
  - [ ] Populate runtime telemetry fields in metrics/phase_4M.toml
  - [ ] Run auditor gate: cargo test -p elai_providers + elai_orchestrator

- [phase-index-3A-4J](reports/steps/phase-index-3A-4J.report.md) — MOSTLY_DONE (see above)

- [phase-index-pre0-2D](reports/steps/phase-index-pre0-2D.report.md) — MOSTLY_DONE (see above)
