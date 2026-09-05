DASHBOARD-V2 DATA-BINDING CONTRACT SHEET
task_id: contract-sheet-v1
scope: F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\ ONLY (legacy crates\dashboard\ and crates\.bkp_old\dashboard\ IGNORED — dead)
purpose: a from-scratch REDESIGN re-wires every panel from this map. The wiring (command + return type + handler) is kept; presentation is thrown away.

ARCHITECTURE (verified)
- Backend handlers: Rust #[tauri::command] fns in F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands*.rs (commands.rs is the big one ~11500 lines; the rest are per-plan: commands_c35.rs, _c40..c47, _d1..d27, _e1..e54, _f2..f13, _g1..g10, _h11..h51, _i33/_i38, _empire.rs, _vibegate.rs, media_validation.rs). Registered in src-tauri\src\lib.rs via generate_handler!.
- Two front-end data paths:
  - STORE path: component imports a thin wrapper fn from src\lib\api\stub.ts (e.g. get_fsm_state, get_decomposition_tree) — usually via a state store in src\lib\state\*.svelte.ts. stub.ts's tryInvoke<T>() (stub.ts:256) calls Tauri invoke() and on absence/reject FALLS BACK to src\lib\api\mock-data.ts. So STORE-path panels render real data when the backend is present and FAKE data otherwise (on_mock = "fallback").
  - INLINE path: component imports invoke directly from @tauri-apps/api/core (often lazily via a local callInvoke<T>(cmd,args) helper that does `const mod = await import('@tauri-apps/api/core'); return mod.invoke<T>(cmd,args)`), passes the command string at the call site, and has NO mock fallback — it shows empty/error state when the backend is absent (on_mock = NO). This is the dominant pattern for the MonitorWindow leaf panels and the D/E/F/G/H bench drawers.
- The callInvoke / q / inv / mod.invoke<T>(cmd,args) helper is per-file boilerplate; the LOAD-BEARING fact is the command STRING literal passed in, captured per row below.
- ts-rs bindings: NO generated bindings dir exists (searched: no "This file was generated" / ts-rs header file under dashboard-v2/src except api/types.ts which is hand-authored). All TS return types are HAND-WRITTEN in src\lib\api\types.ts or declared inline at the call site (e.g. `invoke<{ mermaid: string }>` ). Rust handlers mostly return CmdResult<Value> (= serde_json::Value, untyped) — the TS type is the front-end's assertion, NOT a ts-rs-guaranteed contract. Newer handlers (e37/e41/e54/c47/h51/i33) return typed DTOs (e.g. SafetyStatusDto, TopicHandleDto).
- Events (Tauri listen()): 8 channels in use — symbol_graph_updated (GraphWindow), media_ssb (media_validation store), mini-critic-verdict, reflect-progress (chat reflect), reindex-progress (reindex store), remote-sidecar-progress (boot overlay), research-done + research-progress (drawers store research feed). No EventSource; streaming chat is via start_chat_stream command + emitted token events drained by frame_pacer.ts.

GROUNDING NOTE on return types: where the Rust handler returns CmdResult<Value> the "real" shape is the serde_json::Value the fn builds — the TS type is the front-end's expectation and is the right thing to keep for the redesign. Rows marked UNVERIFIED-TYPE could not have their TS type resolved from a read. file:line citations are from grep -n over the actual files this session.


================================================================================
SECTION 1 — PER-COMPONENT CONTRACT TABLE
================================================================================
Format per component:
component_file | ui_role | calls: command(args)->TSType | backend_handler file:line | events | data_path | feature_map_status | on_mock

--- WINDOWS (top-level surfaces) ---

src/lib/windows/CockpitWindow.svelte
  ui_role: primary cockpit — review-surface cards + hosts intervention/gate widgets
  calls: get_review_surface() -> ReviewSurface[]
  backend_handler: commands_g5.rs:52 get_review_surface -> CmdResult<Value> (review surface rows)
  events: none direct
  data_path: INLINE (callInvoke helper, line 34-46)
  feature_map_status: WIRED (G-5 review surface)
  on_mock: NO

src/lib/windows/MonitorWindow.svelte (1347 lines — the AGGREGATOR; hosts dozens of leaf panels listed below)
  ui_role: monitoring grid; imports + lays out ProposalsPanel, MsiPanel, MediaValidationPanel, MastFailurePanel, CriticDecisionPanel, RoadmapPanel, CostPanel, EvalCorpusPanel, SycophancyMatchedPairPanel, ContextStatePanel, TrustTelemetryPanel, LoraShipGatePanel (+ more). Each leaf panel does its own inline invoke.
  calls: (delegated to child panels — MonitorWindow itself is mostly layout)
  data_path: MIXED (composes INLINE leaf panels)
  feature_map_status: container — see children
  on_mock: NO (children are inline)

src/lib/windows/PlanWindow.svelte
  ui_role: plan / decomposition view + intent-pair (focus tunnel) editor
  calls:
    - query_focus_tunnel_per_baby_step(...) -> FocusTunnelPerStep  | handler commands.rs:8385 query_focus_tunnel_per_baby_step
    - get_intent_pairs({stepId}) -> IntentPairSetLite  | handler commands.rs:11067 get_intent_pairs
    - add_intent_pair({...}) -> ()  | handler commands.rs:10988 add_intent_pair
    - confirm_intent_pairs({...}) -> IntentPairSetLite | handler commands.rs:11017 confirm_intent_pairs
  also reads plan store (get_decomposition_tree)
  data_path: MIXED (INLINE invoke for intent pairs + STORE plan.svelte for tree)
  feature_map_status: WIRED (focus tunnel / intent pairs)
  on_mock: NO for intent-pair calls; tree via store has fallback

src/lib/windows/GraphWindow.svelte
  ui_role: 2D/symbol dependency graph + impact overlay + mermaid export
  calls:
    - get_graph_view_data({mode}) -> VizData  | handler commands.rs:2636 get_graph_view_data
    - get_impact_overlay({...}) -> ImpactOverlay | handler commands.rs:2687 get_impact_overlay
    - export_graph_mermaid({...}) -> string | handler commands.rs:2753 export_graph_mermaid
  events: listen('symbol_graph_updated') -> refetch (line 457)
  data_path: INLINE
  feature_map_status: WIRED (GD-1 graph view, symbol graph)
  on_mock: NO (but mock-orbit.ts fixtures exist for orbit views)

src/lib/windows/SymbolGraphWindow.svelte
  ui_role: SCIP symbol-graph view (query defs/refs/callers/callees + pagerank + status)
  calls (via api/stub.ts wrappers symbolGraphQuery/symbolGraphPageRank/symbolGraphStatus, imported line 20-27): symbol_graph_query -> SymbolGraphQueryReport (commands_e40.rs:67) ; symbol_graph_pagerank -> SymbolGraphPageRankReport (commands_e40.rs:104) ; symbol_graph_status -> SymbolGraphStatusReport (commands_e40.rs:127)
  events: symbol_graph_updated (shared with GraphWindow)
  data_path: STORE (api/stub.ts wrappers)
  feature_map_status: WIRED (E-40 symbol graph)
  on_mock: fallback (stub.ts)

src/lib/windows/CanvasWindow.svelte
  ui_role: canvas-workbench code-graph host
  calls:
    - get_graph_view_data({mode:'symbol'}) -> VizData | commands.rs:2636
    - get_pagerank_top({...}) -> {known, rows[]} | commands.rs:1728 get_pagerank_top
  data_path: INLINE
  feature_map_status: WIRED (canvas / GD-1 reuse)
  on_mock: NO

src/lib/windows/ReviewWindow.svelte
  ui_role: per-file change review (accept/decline/retry verdicts)
  calls:
    - get_file_changes({sessionId}) -> (file changes) | commands.rs:10660 get_file_changes
    - accept_change({path,hunkId?}) -> () | commands.rs:10740 accept_change
    - decline_change({path,hunkId?}) -> () | commands.rs:10752 decline_change
    - retry_audit({path,hunkId?,note?}) -> () | commands.rs:10764 retry_audit
  data_path: INLINE (4 invokes documented in file header lines 6-9)
  feature_map_status: WIRED (review/verdict surface)
  on_mock: NO

src/lib/windows/CommandWindow.svelte
  ui_role: command palette / time-travel commands
  calls: timetravel_undo() -> {head,divergence_detected} ; timetravel_redo() ; timetravel_restore_files_only({target}) | handlers commands_h49.rs:271/:331/:498 (also timetravel_jump :387, timetravel_history :449)
  data_path: INLINE (logStub wrappers around results)
  feature_map_status: WIRED behind flag — H-49 time-travel (verify ELAI_*_ENABLE gate)
  on_mock: NO (logStub is just console logging of real results)

src/lib/windows/ActivityWindow.svelte
  ui_role: orbit/activity visualization of FSM + plan
  calls: none direct — reads fsm + plan stores, renders OrbitCanvas; hosts InterventionControlBar
  data_path: STORE (fsm.svelte, plan.svelte) + child InterventionControlBar (inline)
  feature_map_status: WIRED (orbit activity)
  on_mock: fallback (stores) ; mock-orbit.ts fixtures

src/lib/windows/SettingsWindow.svelte
  ui_role: settings (model routing, gate mode, sidecar, local budget, webhooks, oauth, execution mode)
  calls (via stub.ts wrappers — STORE path): fetch_settings_get/set, set_gate_mode, set_model_routing, set_local_max_context, set_local_min_prompt, set_mcp_enabled, set_execution_mode/get_execution_mode, set_cloud_model/get_cloud_model, set_webhook_url/get_webhook_url, get_oauth_status/start_oauth_login/do_oauth_logout, get_sidecar_settings/set_sidecar_settings, restart_sidecar
  backend_handlers: commands.rs:7014 fetch_settings_set / :6999 get / :947 set_gate_mode / :1049 set_model_routing / :4769 set_local_max_context / :4787 set_local_min_prompt / :4801 set_mcp_enabled / :4807 set_execution_mode / :5181 set_cloud_model / :5354 set_webhook_url / :4952 get_oauth_status / :4982 start_oauth_login / :5141 do_oauth_logout / :7525 get_sidecar_settings / :7547 set_sidecar_settings / :7573 restart_sidecar
  events: has an unlisten on a settings channel (line 337) — verify channel name in file
  data_path: STORE (stub.ts) + session.svelte
  feature_map_status: WIRED (settings/runtime config)
  on_mock: fallback

src/lib/windows/ExploreWindow.svelte
  ui_role: knowledge-engine / explore query console
  calls: run_explore_query({query}) -> Value | commands.rs:4706 ; query_knowledge_engine({query}) -> Value | commands.rs:4630 (via stub.ts/commands store)
  data_path: STORE (commands.svelte imports query_knowledge_engine; run_explore_query in stub.ts)
  feature_map_status: WIRED (explore/knowledge query)
  on_mock: fallback

src/lib/windows/GithubWindow.svelte
  ui_role: GitHub PR management
  calls (STORE via github.svelte + stub.ts wrappers): github_pr_open, github_pr_list, github_pr_status, github_pr_close, github_pr_get_active_url, pr_review_status
  backend_handlers: commands_c40.rs:88/:120/:150/:178/:204 + commands_e41.rs:80 pr_review_status -> PrReviewStatusDto
  data_path: STORE (github.svelte.ts)
  feature_map_status: WIRED (C-40 GitHub PR, E-41 PR review)
  on_mock: fallback

src/lib/windows/AuditTimelineWindow.svelte
  ui_role: per-level audit timeline (B-41 auditor modes); Props {taskId='active'}; renders LevelAuditVerdictJson[] with cascade arrows
  calls (via api/stub.ts wrappers getPerLevelVerdicts/getTestIntegrityEvents, imported line 24, called line 67-68): get_per_level_verdicts(taskId) -> LevelAuditVerdictJson[] (commands.rs:9978) ; get_test_integrity_events(taskId) -> (commands.rs:10024)
  data_path: STORE (api/stub.ts wrappers — note: "live tryInvoke, NO mock per brief" per file comment line 11, so empty state not fake)
  feature_map_status: WIRED (B-41 per-level audit)
  on_mock: NO (stub wrapper intentionally has no mock fallback here)

src/lib/windows/IdeaFramingWindow.svelte / IdeaFramerSecurityPanel.svelte
  ui_role: B-40 idea-framer
  calls: get_idea_framing_state() -> (commands.rs:8552); via stub.get_idea_framing_state (line 34)
  data_path: STORE (api/stub.ts)
  feature_map_status: WIRED (B-40 idea framing)
  on_mock: fallback

src/lib/windows/QdeltaPanel.svelte
  ui_role: QDELTA statistical-floor tile (I-38)
  calls: get_qdelta_latest() -> QdeltaLatest | null | commands_i38.rs:81 get_qdelta_latest
  data_path: INLINE (line 36 invoke(cmd,args) wrapper, call line 41)
  feature_map_status: WIRED behind flag — I-38 (verify ELAI gate)
  on_mock: NO

src/lib/windows/RraViewerPanel.svelte
  ui_role: RRA (reproducible-run-artifact) viewer (G-2)
  calls: get_run_artifact() -> RunArtifact[] | commands_g2.rs:50 get_run_artifact
  data_path: INLINE (callInvoke, line 39)
  feature_map_status: WIRED (G-2 RRA)
  on_mock: NO

src/lib/windows/FileExplorer.svelte / GraphSidebarPanel.svelte / MermaidExportModal.svelte
  ui_role: file tree / graph sidebar / mermaid export modal
  calls: read_dir({root,relPath}) -> FsEntry[] | commands_h11.rs:159 ; read_file({path}) -> String commands_h11.rs:223 ; open_in_editor({path}) commands_h11.rs:242 ; mount_root({path}) commands_h11.rs:282 (FileExplorer via fileTree.svelte store)
  data_path: STORE (fileTree.svelte imports read_dir from stub.ts)
  feature_map_status: WIRED (H-11 filesystem mount)
  on_mock: fallback

--- MONITORWINDOW LEAF PANELS / COMPONENTS (src/lib/components/) — predominantly INLINE ---

src/lib/components/AuthzPanel.svelte
  ui_role: gateway authz status + recent denies + fuzz report
  calls: gateway_authz_status() -> AuthzStatus (commands_c42.rs:106) ; gateway_authz_recent_denies(...) -> AuthzDenyRow[] (commands_c42.rs:115) ; gateway_authz_run_fuzz() -> AuthzFuzzReport (commands_c42.rs:158)
  data_path: INLINE | feature_map_status: WIRED (C-42 authz gateway) | on_mock: NO

src/lib/components/CaseBaseInspector.svelte
  ui_role: tool-calibration case base
  calls: tool_calibration_status() -> ToolCalibrationStatusDto[] (commands_c47.rs:75) ; list_tool_calibration_rows(...) -> ToolCalibrationRowDto[] (commands_c47.rs:112)
  data_path: INLINE | feature_map_status: WIRED (C-47 tool calibration) | on_mock: NO

src/lib/components/CheckerCoveragePanel.svelte
  ui_role: checker coverage + gap backlog (H-41)
  calls: get_coverage_report({...}) -> CoverageReport (commands_h41.rs:92) ; get_checker_gap_backlog({limit}) -> CheckerGapBacklogRow[] (commands_h41.rs:65)
  data_path: INLINE | feature_map_status: WIRED (H-41 checker coverage) | on_mock: NO

src/lib/components/CheckerStatusPanel.svelte
  ui_role: recent checker results (D-11)
  calls: get_checker_results_recent({limit}) -> RecentCheckerResults (commands_d11.rs:124)
  data_path: INLINE | feature_map_status: WIRED (D-11 checker registry) | on_mock: NO

src/lib/components/CriticDecisionPanel.svelte
  ui_role: hostile-critic decisions (E-10)
  calls: get_critic_decision_state({limit:200}) -> CriticDecisionState (commands_e10.rs:61)
  data_path: INLINE | feature_map_status: DARK(ELAI_CRITIC_OF_DESIGN_ENABLE) — E-10 wire-dark per CLAUDE.md | on_mock: NO (empty unless flag ON)

src/lib/components/RoadmapPanel.svelte
  ui_role: 2026-posture roadmap signals (E-38)
  calls: get_roadmap_signals_state({limit:200}) -> RoadmapSignalsState (commands_e38.rs:87)
  data_path: INLINE | feature_map_status: DARK(ELAI_AI_ROADMAP_SIGNALS_ENABLE) per CLAUDE.md | on_mock: NO (empty unless flag ON)

src/lib/components/CostPanel.svelte
  ui_role: velocity cap + cost router + automix + free-tier quotas (E-39)
  calls: get_velocity_cap_status() (commands_e39.rs:175) ; get_cost_router_recent() (:237) ; get_automix_verdicts_recent() (:291) ; get_free_tier_quotas() (:335)
  data_path: INLINE | feature_map_status: WIRED/PARTIAL (E-39 cost router; some RUNPOD-PENDING) | on_mock: NO

src/lib/components/EvalCorpusPanel.svelte
  ui_role: eval-corpus status (E-54)
  calls: get_eval_corpus_status({...}) -> EvalCorpusStatus (commands_e54.rs:128)
  data_path: INLINE | feature_map_status: WIRED (E-54 eval firewall) | on_mock: NO

src/lib/components/SycophancyMatchedPairPanel.svelte
  ui_role: sycophancy matched-pair results (E-54)
  calls: get_sycophancy_matched_pair_results(...) -> SycophancyMatchedPairResults (commands_e54.rs:218)
  data_path: INLINE | feature_map_status: WIRED (E-54) | on_mock: NO

src/lib/components/MastFailurePanel.svelte
  ui_role: MAST failure breakdown + iso-budget (E-6)
  calls: get_mast_breakdown({...}) -> MastBreakdownResponse (commands_e6.rs:80) ; get_iso_budget_recent({...}) -> IsoBudgetMeasuredDto[] (commands_e6.rs:206)
  data_path: INLINE | feature_map_status: WIRED (E-6 MAST) | on_mock: NO

src/lib/components/McpStatusPanel.svelte
  ui_role: MCP security status
  calls: get_mcp_security_status() -> McpSecurityStatus[] (commands.rs:261)
  data_path: INLINE | feature_map_status: WIRED | on_mock: NO

src/lib/components/MediaValidationPanel.svelte
  ui_role: browser-e2e / media validation status
  calls: browser_e2e_status() -> BrowserE2eStatus (commands_e14.rs:68)
  events: media_ssb (via media_validation store)
  data_path: MIXED (inline browser_e2e_status + media_validation store for ssb) | feature_map_status: WIRED (E-14 browser e2e) | on_mock: NO

src/lib/components/OptimizerRunPanel.svelte
  ui_role: optimizer run state
  calls: get_optimizer_run({n}) -> OptimizerRunStateDto (commands.rs:11425)
  data_path: INLINE | feature_map_status: WIRED | on_mock: NO

src/lib/components/PromptArmorPanel.svelte
  ui_role: safety/prompt-armor status + rows (E-37)
  calls: safety_status() -> SafetyStatusDto (commands_e37.rs:77) ; list_safety_rows({...}) -> SafetyRowDto[] (commands_e37.rs:119)
  data_path: INLINE | feature_map_status: WIRED (E-37 safety) | on_mock: NO

src/lib/components/ToolGateInspector.svelte
  ui_role: tool-gate snapshot + rows (C-35)
  calls: get_tool_gate_snapshot({...}) -> GateSnapshot (commands_c35.rs:102) ; list_tool_gate_rows({...}) -> ToolGateRow[] (commands_c35.rs:130)
  data_path: INLINE | feature_map_status: WIRED (C-35 tool gate) | on_mock: NO

src/lib/components/ToolReliabilityInspector.svelte
  ui_role: tool reliability status + rows (E-18)
  calls: tool_reliability_status() -> ReliabilityStatusDto (commands_e18.rs:88) ; list_tool_reliability_rows(...) -> ReliabilityRowDto[] (commands_e18.rs:151)
  data_path: INLINE | feature_map_status: WIRED (E-18) | on_mock: NO

src/lib/components/GrammarCompilePanel.svelte
  ui_role: grammar compilations (E-35)
  calls: get_grammar_compilations() -> GrammarCompilationsResponse (commands_e35.rs:39)
  data_path: INLINE | feature_map_status: WIRED (E-35 grammar) | on_mock: NO

src/lib/components/ArchiveVerifyGauge.svelte
  ui_role: archive verify gauge (D-26)
  calls: get_archive_verify_status() -> ArchiveVerifyStatus (commands_d26.rs:459)
  data_path: INLINE | feature_map_status: WIRED (D-26 archive) | on_mock: NO

src/lib/components/NoveltyBreakdown.svelte
  ui_role: novelty breakdown + recent (D-15)
  calls: get_novelty_breakdown({...}) -> NoveltyBreakdown (commands_d15.rs:236) ; get_novelty_recent({...}) -> RecentNovelty (commands_d15.rs:72)
  data_path: INLINE | feature_map_status: WIRED (D-15 novelty) | on_mock: NO

src/lib/components/PinnedRulesPanel.svelte
  ui_role: pinned rules
  calls: get_pinned_rules({}) -> PinnedRule[]  | backend_handler: UNVERIFIED — grep over src-tauri/src/*.rs found NO get_pinned_rules #[tauri::command]. Either the command does not exist (panel will error / show empty) or it is registered under a different name. See SECTION 5 UNVERIFIED list.
  data_path: INLINE (callInvoke) | feature_map_status: unmatched | on_mock: NO

src/lib/components/SycophancyFlagStrip.svelte
  ui_role: sycophancy flag strip (D-27)
  calls: get_sycophancy_flags_recent({limit}) -> SycophancyFlagRow[] (commands_d27.rs:288)
  data_path: INLINE | feature_map_status: WIRED (D-27) | on_mock: NO

src/lib/components/TripwireStatusStrip.svelte
  ui_role: tripwire status strip (D-4)
  calls: get_recent_tripwires({limit}) -> TripwireRow[] (commands_d4.rs:64)
  data_path: INLINE | feature_map_status: WIRED (D-4 tripwires) | on_mock: NO

src/lib/components/InterventionControlBar.svelte
  ui_role: intervention / confirm / credential response bar
  calls: submit_intervention_response (commands.rs:532) ; respond_to_confirmation (:563) ; submit_credential_response (:585) — via callInvoke + possibly stub
  data_path: INLINE (callInvoke line 61) | feature_map_status: WIRED (intervention plane) | on_mock: NO

src/lib/components/EmpireBridgePanel.svelte
  ui_role: empire-bridge factory status
  calls: get_empire_status() -> EmpireStatusDto (commands_empire.rs:79) ; list_empire_factory_lines(...) (commands_empire.rs:92) — via api/empire.ts wrappers (try-invoke-else-mock, empire.ts:84)
  data_path: STORE (api/empire.ts) | feature_map_status: WIRED/PARTIAL (empire bridge) | on_mock: fallback (empire.ts mock fallback)

src/lib/components/ArtifactsPanel.svelte
  ui_role: cosmos/artifact DB browser
  calls: list_artifacts({db_path}) -> {artifact_names[]} (commands.rs:9232) ; search_artifacts (:9264) ; index_artifacts (:9334); remove_artifacts (:9366)
  data_path: INLINE (inv helper) | feature_map_status: WIRED (B-42 cosmos artifacts) | on_mock: NO

src/lib/components/IndexingProgressPanel.svelte
  ui_role: reindex progress + resilience
  calls: get_indexing_progress({}) -> IndexingProgressDto (commands.rs:6540) ; get_indexing_resilience({}) -> IndexingResilienceDto (commands.rs:6627)
  events: reindex-progress (via reindex store)
  data_path: INLINE + reindex store | feature_map_status: WIRED | on_mock: NO

src/lib/components/LoraShipGatePanel.svelte
  ui_role: LoRA ship-gate status
  calls: get_lora_ship_gate_status() -> LoraShipGateStatus (commands.rs:11527)
  data_path: INLINE | feature_map_status: WIRED/BENCH-PENDING (LoRA gate) | on_mock: NO

src/lib/components/GatesPanel.svelte
  ui_role: ambiguity / gate state
  calls: get_ambiguity_state() -> AmbiguityState (commands.rs:9529)
  data_path: INLINE (inv helper) | feature_map_status: WIRED | on_mock: NO

src/lib/components/ContextStatePanel.svelte
  ui_role: context-state / window budget, polled every 5s (H-50 context lifecycle)
  calls: get_context_state() -> ContextStateDto (commands.rs:8798); imported from api/stub.ts (line 26, call line 55)
  data_path: STORE (api/stub.ts wrapper) | feature_map_status: WIRED/PARTIAL (H-50) | on_mock: fallback

src/lib/components/TrustTelemetryPanel.svelte
  ui_role: trust telemetry report (F-plan trust)
  calls: get_trust_telemetry() -> TrustReportDto (commands.rs:8981); imported from api/stub.ts (line 28, call line 48)
  data_path: STORE (api/stub.ts wrapper) | feature_map_status: WIRED/PARTIAL (trust spine; some wire-dark) | on_mock: fallback

src/lib/components/ProposalsPanel.svelte
  ui_role: self-improvement proposals (pending/applied)
  calls: get_pending_proposals (commands.rs:1144) ; get_applied_proposals (:1162) ; apply_proposal_from_user (:1274) ; reject_proposal (:1302) — via stub.ts
  data_path: STORE (stub.ts) | feature_map_status: WIRED/DARK (self-improve, mostly gated) | on_mock: fallback

src/lib/components/MsiPanel.svelte
  ui_role: MSI (model-self-improvement) status
  calls: get_msi_status() (commands.rs:1315) ; get_steering_status (:1324) ; recompute_steering_vector (:1335) — verify which; via stub
  data_path: STORE (stub.ts) | feature_map_status: DARK/PARTIAL (steering gated) | on_mock: fallback

src/lib/components/ClaimConfidenceChip.svelte / SycophancyGuardBadge.svelte
  ui_role: inline badge chips (render an evidence_ref prop; no own backend call — consume parent data)
  calls: NONE (presentational; read props with {kind:'ssb_seq', value} evidence refs)
  data_path: n/a (prop-driven) | feature_map_status: WIRED (badge of G-7/E-54 data) | on_mock: NO

--- DRAWERS (src/lib/drawers/) — predominantly INLINE via callInvoke ---

src/lib/drawers/AdversaryPanel.svelte
  calls: get_adversarial_runs_recent({...}) -> RecentAdversarialRuns (commands_d13.rs:68) ; get_adversary_config() -> AdversaryConfigDto (commands_d13.rs:129)
  data_path: INLINE | status: WIRED (D-13 adversary) | on_mock: NO

src/lib/drawers/ARCBenchPanel.svelte
  calls: get_arc_bench_recent({limit:25}) -> RecentARCBenchRuns (commands_d20.rs:62)
  data_path: INLINE | status: WIRED/BENCH-PENDING (D-20 ARC bench) | on_mock: NO

src/lib/drawers/AttestationDrawer.svelte
  calls: get_attestations() -> Attestation[] (commands_g8.rs:54)
  data_path: INLINE | status: WIRED (G-8 attestation) | on_mock: NO

src/lib/drawers/BabyStepPanel.svelte
  calls: get_baby_step_dispatches_recent(...) -> (commands_d12.rs:66)
  data_path: INLINE | status: WIRED (D-12 baby-step dispatch) | on_mock: NO

src/lib/drawers/BaiArmsPanel.svelte
  calls: get_bai_arms({limit:50}) -> BaiArmRow[] (commands_d5.rs:51)
  data_path: INLINE | status: WIRED (D-5 BAI arms) | on_mock: NO

src/lib/drawers/BlastRadiusDrawer.svelte
  calls: get_blast_radius_blocks() -> BlastRadiusBlock[] (commands_g3.rs:53)
  data_path: INLINE | status: WIRED (G-3 blast radius) | on_mock: NO

src/lib/drawers/CanaryPanel.svelte
  calls: get_canary_recent({limit:50}) -> CanaryRunRow[] (commands_d7.rs:63) ; get_canary_per_family_breakdown({limit:50}) -> CanaryBreakdown (commands_d7.rs:100)
  data_path: INLINE | status: WIRED (D-7 canary) | on_mock: NO

src/lib/drawers/CheckerRegistryPanel.svelte
  calls: get_checker_results_recent({...}) -> RecentCheckerResults (commands_d11.rs:124) ; get_checker_registry_state() -> CheckerRegistryState (commands_d11.rs:179)
  data_path: INLINE | status: WIRED (D-11) | on_mock: NO

src/lib/drawers/ConfidenceDrawer.svelte
  calls: get_claim_confidence() -> ClaimConfidence[] (commands_g7.rs:55) ; get_trifecta_quarantines() -> TrifectaQuarantine[] (commands_g7.rs:73)
  data_path: INLINE | status: WIRED (G-7 claim confidence) | on_mock: NO

src/lib/drawers/ConformalCalibrationPanel.svelte
  calls: get_conformal_recent({...}) -> RecentConformal (commands_d16.rs:74) ; get_calibration_corpus_state() -> CalibrationCorpusStateDto (commands_d16.rs:191)
  data_path: INLINE | status: WIRED (D-16 conformal) | on_mock: NO

src/lib/drawers/ConformalValidationPanel.svelte
  calls: get_conformal_validation_recent(...) -> (commands_d22.rs:44)
  data_path: INLINE | status: WIRED (D-22) | on_mock: NO

src/lib/drawers/DecompositionEntropyHistogram.svelte
  calls: get_entropy_history({limit:50}) -> EntropyHistogramRow[] (commands_e46.rs:374)
  data_path: INLINE | status: WIRED (E-46 entropy) | on_mock: NO

src/lib/drawers/DiscoveryQueueDrawer.svelte
  calls: get_mode_routing_refusal_combined(...) -> (commands_d27.rs:171) ; ratify_discovery (commands_d27.rs:363) ; reject_discovery (commands_d27.rs:400)
  data_path: INLINE | status: WIRED (D-27 discovery queue) | on_mock: NO

src/lib/drawers/FailureModesDrawer.svelte
  calls: get_mast_breakdown({limit}) -> BreakdownResponse (commands_e6.rs:80)
  data_path: INLINE | status: WIRED (E-6) | on_mock: NO

src/lib/drawers/HeldOutPanel.svelte
  calls: get_held_out_recent({limit:50}) -> HeldOutTaskRow[] (commands_d6.rs:61) ; get_held_out_stats() -> HeldOutStatsRow (commands_d6.rs:98)
  data_path: INLINE | status: WIRED (D-6 held-out) | on_mock: NO

src/lib/drawers/IncidentsDrawer.svelte
  calls: get_recent_incidents({...}) -> IncidentRow[] (commands_e30.rs:80)
  data_path: INLINE | status: WIRED (E-30 incidents) | on_mock: NO

src/lib/drawers/LockedKeysPanel.svelte
  calls: get_locked_keys_status() -> LockedKeysStatus (commands_d9.rs:86) ; get_locked_keys_recent_audits({limit:50}) -> RecentAudits (commands_d9.rs:125)
  data_path: INLINE | status: WIRED (D-9 locked keys / action_type lock) | on_mock: NO

src/lib/drawers/McpRugPullDrawer.svelte
  calls: get_mcp_rug_pulls() -> McpRugPull[] (commands_g4.rs:49)
  data_path: INLINE | status: WIRED (G-4 mcp rug pull) | on_mock: NO

src/lib/drawers/MemoryFactsDrawer.svelte
  calls: get_memory_facts_with_history({...}) -> MemoryFactsHistoryResponse (commands_e5.rs:456) ; get_memory_facts({...}) -> MemoryFactsResponse (commands_e5.rs:138) ; get_semantic_contradictions(...) -> SemanticContradictionsResponse (commands_e5.rs:262) ; memory_forget({factId}) (commands_e5.rs:350) ; memory_pin({factId,pinned}) (commands_e5.rs:403)
  data_path: INLINE | status: WIRED (E-5 memory facts / user memory) | on_mock: NO

src/lib/drawers/MemoryNotesPanel.svelte
  calls: get_memory_notes({...}) -> MemoryNotesResponse (commands_e1.rs:174) ; get_chat_memory_state() -> ChatMemoryState (commands_e1.rs:225)
  data_path: INLINE | status: WIRED (E-1 memory notes) | on_mock: NO

src/lib/drawers/ModelDriftPanel.svelte
  calls: get_model_drift_status() -> (commands_g10.rs:72)
  data_path: INLINE (invoke wrapper line 42) | status: WIRED (G-10 model drift) | on_mock: NO

src/lib/drawers/ModeRoutingPanel.svelte
  calls: get_recent_claim_routings({...}) -> RecentClaimRoutings (commands_d10.rs:57)
  data_path: INLINE | status: WIRED (D-10 claim routing) | on_mock: NO

src/lib/drawers/ModeRoutingBenchPanel.svelte
  calls: get_mode_routing_bench_recent({limit:50}) -> ModeRoutingBenchRow[] (commands_d21.rs:45)
  data_path: INLINE | status: WIRED/BENCH-PENDING (D-21) | on_mock: NO

src/lib/drawers/MutationArchivePanel.svelte
  calls: get_archive_recent({limit:50}) -> ArchiveRecentRow[] (commands_d1.rs:52)
  data_path: INLINE | status: WIRED (D-1 mutation archive) | on_mock: NO

src/lib/drawers/NoveltyPanel.svelte
  calls: get_novelty_recent({limit:50}) -> RecentNovelty (commands_d15.rs:72)
  data_path: INLINE | status: WIRED (D-15) | on_mock: NO

src/lib/drawers/ObservableEffectPanel.svelte
  calls: get_observable_effects_recent(...) -> (commands_d18.rs:139)
  data_path: INLINE | status: WIRED (D-18 observable effect) | on_mock: NO

src/lib/drawers/OrchestrationDrawer.svelte
  calls: get_ambiguity_state() -> AmbiguityState (commands.rs:9529) ; get_stagnation_events() -> StagnationEvent[] (commands.rs:8657) ; credit_attribution_status() -> CreditAttributionSnapshot (commands.rs:11090) ; routing_decisions_status() -> RoutingDecisionsSnapshot (commands.rs:11169)
  data_path: INLINE | status: WIRED (orchestration: ambiguity/stagnation/credit/routing) | on_mock: NO

src/lib/drawers/PagerankPanel.svelte
  calls: get_pagerank_top({n}) -> PagerankResponse (commands.rs:1728)
  data_path: INLINE | status: WIRED (pagerank) | on_mock: NO

src/lib/drawers/PlanningLedgerPanel.svelte
  calls: get_dual_ledger_state({...}) -> DualLedgerStateDto (commands_e46.rs:99) ; get_recent_agentless_shortcuts({...}) -> AgentlessShortcutRow[] (commands_e46.rs:166) ; get_goal_drift_status() -> GoalDriftStatusDto (commands_e46.rs:200)
  data_path: INLINE | status: WIRED (E-46 dual ledger) | on_mock: NO

src/lib/drawers/PpmScoresPanel.svelte
  calls: get_ppm_recent_scores({limit:50}) -> PpmScoreRow[] (commands_d2.rs:49)
  data_path: INLINE | status: WIRED (D-2 PPM) | on_mock: NO

src/lib/drawers/PrmBenchPanel.svelte
  calls: get_prmbench_recent({limit:50}) -> PrmBenchRow[] (commands_d19.rs:48)
  data_path: INLINE | status: WIRED/BENCH-PENDING (D-19 PRM bench) | on_mock: NO

src/lib/drawers/QueryDrawer.svelte
  calls: get_query_trace_recent({n}) -> QueryTraceRow[] (commands.rs:1409)
  data_path: INLINE | status: WIRED (query trace) | on_mock: NO

src/lib/drawers/RedactionPanel.svelte
  calls: get_redaction_stats() -> RedactionStat[] (commands_g9.rs:51)
  data_path: INLINE | status: WIRED (G-9 redaction) | on_mock: NO

src/lib/drawers/RefusalRateGauge.svelte
  calls: get_refusal_sla_recent({...}) -> RecentRefusalSla (commands_d17.rs:70) [also get_refusal_rate_current commands_d17.rs:127]
  data_path: INLINE | status: WIRED (D-17 refusal SLA) | on_mock: NO

src/lib/drawers/ResearchDrawer.svelte
  calls: get_research_node_history(...) -> ResearchNodeHistoryRow[] (commands.rs:10703)
  events: research-progress, research-done (via drawers store)
  data_path: MIXED (inline history + STORE drawers research feed) | status: WIRED (research tree) | on_mock: fallback for feed

src/lib/drawers/SafeDeltaPanel.svelte
  calls: get_safety_delta_latest() -> (commands_g6.rs:74)
  data_path: INLINE (invoke wrapper line 37) | status: WIRED/BENCH-PENDING (G-6 SAFEDELTA) | on_mock: NO

src/lib/drawers/SecurityMonotonicityDrawer.svelte
  calls: get_security_monotonicity_blocks() -> (commands_f10.rs:102)
  data_path: INLINE | status: WIRED (F-10 security monotonicity) | on_mock: NO

src/lib/drawers/SecurityRulesPanel.svelte
  calls: get_security_blocks_recent({...}) -> SecurityBlocksResponse (commands_f2.rs:79) ; get_insecure_rate_latest() -> InsecureRateRow|null (commands_f2.rs:171)
  data_path: INLINE | status: WIRED (F-2 security blocks) | on_mock: NO

src/lib/drawers/SessionArchiveDrawer.svelte
  calls: get_session_archives() -> SessionArchive[] (commands_f13.rs:52)
  data_path: INLINE | status: WIRED (F-13 session archive) | on_mock: NO

src/lib/drawers/SessionsDrawer.svelte
  calls: fork_session_branch({name}) (commands.rs:3584) ; merge_session_branch({name}) (commands.rs:3677) ; close_session_branch({name}) (commands.rs:3701) [+ get_session_branches commands.rs:3413 via store]
  data_path: MIXED (inline branch ops + STORE for branch list) | status: WIRED (session branches) | on_mock: branch list fallback

src/lib/drawers/StalenessQuarantineDrawer.svelte
  calls: get_staleness_quarantine() -> (commands_f5.rs:84)
  data_path: INLINE | status: WIRED (F-5 staleness quarantine) | on_mock: NO

src/lib/drawers/SycophancyBenchPanel.svelte
  calls: get_sycophancy_bench_recent({limit:50}) -> SycophancyBenchRow[] (commands_d23.rs:51)
  data_path: INLINE | status: WIRED/BENCH-PENDING (D-23) | on_mock: NO

src/lib/drawers/TaintFlowDrawer.svelte
  calls: get_taint_flows() -> TaintFlow[] (commands_g1.rs:48)
  data_path: INLINE | status: WIRED (G-1 taint flow) | on_mock: NO

src/lib/drawers/TemplateRankingDrawer.svelte
  calls: get_template_ranking() -> (commands_f7.rs:89)
  data_path: INLINE | status: WIRED (F-7 template ranking) | on_mock: NO

src/lib/drawers/TranscriptDrawer.svelte
  calls: ssb_transcript_paged({...}) -> TranscriptPage (commands_e29.rs:257)  [related: ssb_transcript :170, ssb_resume_from :388, ssb_promote_to_dataset :442, ssb_backfill_signals :468]
  data_path: INLINE | status: WIRED (E-29 paged transcript) | on_mock: NO

src/lib/drawers/TripwirePanel.svelte
  calls: get_recent_tripwires({limit:50}) -> TripwireRow[] (commands_d4.rs:64)
  data_path: INLINE | status: WIRED (D-4) | on_mock: NO

src/lib/drawers/VerificationConsensusPanel.svelte
  calls: get_recent_verifications({limit:50}) -> VerificationConsensusRow[] (commands_d3.rs:52)
  data_path: INLINE | status: WIRED (D-3 verification consensus) | on_mock: NO

src/lib/drawers/VibeGateDrawer.svelte
  calls: list_vibegate_blocks({...}) -> VibeGateBlocksResponse (commands_vibegate.rs:83)
  data_path: INLINE | status: WIRED (vibegate) | on_mock: NO

--- VIEWS (src/lib/views/) ---

src/lib/views/ArchiveExplorerView.svelte
  calls: get_template_archive_combined({limit:100}) -> CombinedArchiveView (commands_d26.rs:213)
  data_path: INLINE (callInvoke + a 2nd mod.invoke) | status: WIRED (D-26) | on_mock: NO

src/lib/views/LineageGraphView.svelte
  calls: get_archive_lineage({templateId}) -> ArchiveLineage (commands_d26.rs:200)
  data_path: INLINE | status: WIRED (D-26 lineage) | on_mock: NO

src/lib/views/ModeRoutingDashboardView.svelte
  calls: get_mode_routing_refusal_combined(...) -> (commands_d27.rs:171)
  data_path: INLINE | status: WIRED (D-27) | on_mock: NO

--- WIDGETS / CANVAS / SCREENS ---

src/lib/widgets/FreezeStatusBar.svelte
  calls: scope_freeze_status({}) -> FreezeStatus (commands_c43.rs:84)
  data_path: INLINE | status: WIRED (C-43 scope freeze) | on_mock: NO

src/lib/canvas/ArchMode.svelte
  calls: export_graph_mermaid({...}) -> {mermaid:string} (commands.rs:2753)
  data_path: INLINE | status: WIRED (canvas arch mode) | on_mock: NO

src/lib/canvas/DesignPageNode.svelte
  calls: resolve_oid(...) -> {oid,file,range[],scope} (commands_h14.rs:109)
  data_path: INLINE | status: WIRED (H-14 canvas recast) | on_mock: NO

src/lib/canvas/entrypoints_map.ts
  calls: list_entry_points(...) -> {entry_points:BackendEntryPoint[], summary} (commands.rs:9396)
  data_path: INLINE (ts file) | status: WIRED (entrypoints map) | on_mock: NO

src/lib/screens/RemoteSidecarBootOverlay.svelte
  events: remote-sidecar-progress ; calls start_remote_sidecar (commands.rs:7692) / get_remote_sidecar_progress (:7784) / cancel_remote_sidecar (:7800) via stub
  data_path: STORE/INLINE | status: WIRED (remote sidecar boot) | on_mock: NO (real boot)

src/lib/screens/CloudPanel.svelte
  calls: get_cloud_model/set_cloud_model (commands.rs:5167/:5181) ; get_cloud_spend (:5217) ; get_cloud_route_status (:5242) via stub
  data_path: STORE (stub.ts) | status: WIRED (cloud routing) | on_mock: fallback

src/lib/screens/ProjectSelector.svelte
  calls: list_projects (commands.rs:10842) ; create_project (:10880) ; open_project_from_folder (:616) via stub
  data_path: STORE (stub.ts) | status: WIRED (project selection) | on_mock: fallback

--- STATE STORES that issue commands (see SECTION 3 for full ownership) ---
(these are .ts files in the STORE path; the windows above read them)

src/lib/state/fsm.svelte.ts        -> get_fsm_state (commands.rs:296)
src/lib/state/plan.svelte.ts       -> get_decomposition_tree (commands.rs:3768) [+ get_plan_progress :400, get_phase_detail :10580, get_baby_step_detail :10604]
src/lib/state/session.svelte.ts    -> get_gate_status (commands.rs:925), get_execution_mode (:4840), set_execution_mode (:4807)
src/lib/state/commands.svelte.ts   -> streamChat/start_chat_stream (commands.rs:5839), cancel_chat_stream (:6242), run_research (:6857), submit_task (:499), query_knowledge_engine (:4630), continue_task (:4757), get_media_validations/by_step/submit (media_validation.rs:419/:431/:448)
src/lib/state/github.svelte.ts     -> github_pr_* (commands_c40.rs) + pr_review_status (commands_e41.rs:80)
src/lib/state/fileTree.svelte.ts   -> read_dir (commands_h11.rs:159)
src/lib/state/feature_flags.svelte.ts -> get_feature_flags (commands.rs:1031)
src/lib/state/reindex.svelte.ts    -> reindex-progress event + reindex_now (commands.rs:6388) / cancel_indexing (:6505)
src/lib/state/media_validation.svelte.ts -> media_ssb event + subscribe_media_ssb_events (media_validation.rs:476)
src/lib/state/drawers.svelte.ts    -> aggregates many stub.ts fns + research-progress/research-done events
src/lib/state/grc.svelte.ts        -> get_grc_budgets (commands.rs:2817), get_grounding_sweep (:2868), get_csq_pending (:2917), get_csq_log (:2986)
src/lib/state/permission_prompts.svelte.ts -> stub.ts permission wrappers
src/lib/state/workspace.svelte.ts  -> logStub only (UI workspace layout state; no backend command)
src/lib/state/activity.svelte.ts   -> logStub only (UI activity-feed buffer)
src/lib/state/canvas*.svelte.ts (canvas, canvasActivity, canvasDispatcher, canvasViewState) -> get_graph_view_data (commands.rs:2636), get_canvas_activity (commands_h11.rs:311), get_canvas_ops (commands_h13.rs:66), get_canvas_views (commands_h15.rs:115), canvas_select_node (commands_h15.rs:167), emit_canvas_checkpoint (commands_h15.rs:220)
src/lib/state/graph.svelte.ts      -> pure client-side graph layout state (reads VizData from canvas/graph windows)
src/lib/state/review.svelte.ts     -> client-side review verdict buffer (ReviewWindow does the invokes inline)
src/lib/state/motion.svelte.ts / ux_prefs.svelte.ts / shortcuts.ts -> UI prefs only, no backend


================================================================================
SECTION 2 — COMMAND INDEX (reverse map)
================================================================================
NOTE: every command also appears next to its caller in SECTION 1 with TS return type. This section is the REVERSE lens: shared commands first (the ones the redesign must not duplicate), then the full handler list grouped by file. Untyped handlers return CmdResult<Value> (serde_json::Value) on the Rust side; the load-bearing front-end type is the one in SECTION 1.

--- SHARED commands (called by >1 component — redesign should wrap ONCE in a store) ---
get_mast_breakdown             commands_e6.rs:80      -> MastFailurePanel + FailureModesDrawer
get_graph_view_data            commands.rs:2636       -> GraphWindow + CanvasWindow + canvasActivity store
export_graph_mermaid           commands.rs:2753       -> GraphWindow + ArchMode
get_pagerank_top               commands.rs:1728       -> CanvasWindow + PagerankPanel
get_checker_results_recent     commands_d11.rs:124    -> CheckerStatusPanel + CheckerRegistryPanel
get_recent_tripwires           commands_d4.rs:64      -> TripwireStatusStrip + TripwirePanel
get_novelty_recent             commands_d15.rs:72     -> NoveltyBreakdown + NoveltyPanel
get_ambiguity_state            commands.rs:9529       -> GatesPanel + OrchestrationDrawer
get_mode_routing_refusal_combined commands_d27.rs:171 -> DiscoveryQueueDrawer + ModeRoutingDashboardView
get_template_archive_combined  commands_d26.rs:213    -> ArchiveExplorerView (two call sites — callInvoke + mod.invoke)

--- HANDLER FILE -> exported commands (file:line in SECTION 1 / per-handler grep) ---
commands.rs (the core, ~280 fns; key ones consumed by the dashboard):
  get_fsm_state:296, get_plan_progress:400, get_system_status:465, submit_task:499, submit_intervention_response:532,
  respond_to_confirmation:563, submit_credential_response:585, open_project_from_folder:616, get_gate_status:925,
  set_gate_mode:947, get_feature_flags:1031, set_model_routing:1049, get_pending_proposals:1144, get_applied_proposals:1162,
  apply_proposal_from_user:1274, reject_proposal:1302, get_msi_status:1315, get_steering_status:1324, recompute_steering_vector:1335,
  get_query_trace_recent:1409, get_pagerank_top:1728, get_grc_budgets:2817, get_grounding_sweep:2868, get_csq_pending:2917,
  get_csq_log:2986, get_session_branches:3413, fork_session_branch:3584, merge_session_branch:3677, close_session_branch:3701,
  get_decomposition_tree:3768, get_graph_view_data:2636, get_impact_overlay:2687, export_graph_mermaid:2753,
  query_knowledge_engine:4630, run_explore_query:4706, get_execution_mode:4840, set_execution_mode:4807, get_cloud_model:5167,
  set_cloud_model:5181, get_cloud_spend:5217, get_cloud_route_status:5242, fetch_settings_get:6999, fetch_settings_set:7014,
  start_chat_stream:5839, cancel_chat_stream:6242, reindex_now:6388, cancel_indexing:6505, get_indexing_progress:6540,
  get_indexing_resilience:6627, run_research:6857, query_focus_tunnel_per_baby_step:8385, get_idea_framing_state:8552,
  get_stagnation_events:8657, get_context_state:8798, get_trust_telemetry:8981, list_artifacts:9232, search_artifacts:9264,
  index_artifacts:9334, remove_artifacts:9366, list_entry_points:9396, get_ambiguity_state:9529, get_per_level_verdicts:9978,
  get_test_integrity_events:10024, get_file_changes:10660, accept_change:10740, decline_change:10752, retry_audit:10764,
  get_research_node_history:10703, list_projects:10842, create_project:10880, add_intent_pair:10988, confirm_intent_pairs:11017,
  get_intent_pairs:11067, credit_attribution_status:11090, routing_decisions_status:11169, get_optimizer_run:11425,
  get_lora_ship_gate_status:11527, start_remote_sidecar:7692, get_remote_sidecar_progress:7784, cancel_remote_sidecar:7800,
  get_sidecar_settings:7525, set_sidecar_settings:7547, restart_sidecar:7573, get_oauth_status:4952, start_oauth_login:4982,
  do_oauth_logout:5141, list_runtimes:10228, test_runtime:10311, set_role_runtime:10378, get_reflect_state:8340,
  get_mcp_security_status:261, get_phase_detail:10580, get_baby_step_detail:10604, get_phase_progress:10629
commands_c35.rs: get_tool_gate_snapshot:102, list_tool_gate_rows:130
commands_c40.rs: github_pr_open:88, github_pr_list:120, github_pr_status:150, github_pr_close:178, github_pr_get_active_url:204
commands_c41.rs: list_application_skills:114, recent_skill_routing_events:154
commands_c42.rs: gateway_authz_status:106, gateway_authz_recent_denies:115, gateway_authz_run_fuzz:158
commands_c43.rs: scope_freeze_status:84
commands_c47.rs: tool_calibration_status:75, list_tool_calibration_rows:112
commands_d1.rs: get_archive_recent:52, get_reasoning_template_by_id:80
commands_d2.rs: get_ppm_recent_scores:49
commands_d3.rs: get_recent_verifications:52
commands_d4.rs: get_recent_tripwires:64
commands_d5.rs: get_bai_arms:51
commands_d6.rs: get_held_out_recent:61, get_held_out_stats:98
commands_d7.rs: get_canary_recent:63, get_canary_per_family_breakdown:100
commands_d9.rs: get_locked_keys_status:86, get_locked_keys_recent_audits:125
commands_d10.rs: get_recent_claim_routings:57
commands_d11.rs: get_checker_results_recent:124, get_checker_registry_state:179
commands_d12.rs: get_baby_step_dispatches_recent:66
commands_d13.rs: get_adversarial_runs_recent:68, get_adversary_config:129
commands_d15.rs: get_novelty_recent:72, get_novelty_distribution:124, get_novelty_breakdown:236
commands_d16.rs: get_conformal_recent:74, get_calibration_corpus_state:191
commands_d17.rs: get_refusal_sla_recent:70, get_refusal_rate_current:127
commands_d18.rs: get_observable_effects_recent:139
commands_d19.rs: get_prmbench_recent:48
commands_d20.rs: get_arc_bench_recent:62
commands_d21.rs: get_mode_routing_bench_recent:45
commands_d22.rs: get_conformal_validation_recent:44
commands_d23.rs: get_sycophancy_bench_recent:51
commands_d26.rs: get_archive_lineage:200, get_template_archive_combined:213, get_archive_verify_status:459
commands_d27.rs: get_mode_routing_refusal_combined:171, get_sycophancy_flags_recent:288, ratify_discovery:363, reject_discovery:400
commands_e1.rs: get_memory_notes:174, get_chat_memory_state:225
commands_e5.rs: get_memory_facts:138, get_trust_score:164, get_semantic_contradictions:262, memory_forget:350, memory_pin:403, get_memory_facts_with_history:456
commands_e6.rs: get_mast_breakdown:80, get_iso_budget_recent:206
commands_e10.rs: get_critic_decision_state:61
commands_e14.rs: browser_e2e_status:68
commands_e18.rs: tool_reliability_status:88, list_tool_reliability_rows:151
commands_e25.rs: get_compaction_ladder_state:127, recent_compaction_events:151
commands_e29.rs: ssb_transcript:170, ssb_transcript_paged:257, ssb_resume_from:388, ssb_promote_to_dataset:442, ssb_backfill_signals:468
commands_e30.rs: get_recent_incidents:80
commands_e35.rs: get_grammar_compilations:39
commands_e37.rs: safety_status:77, list_safety_rows:119
commands_e38.rs: get_roadmap_signals_state:87
commands_e39.rs: get_velocity_cap_status:175, get_cost_router_recent:237, get_automix_verdicts_recent:291, get_free_tier_quotas:335
commands_e40.rs: symbol_graph_query:67, symbol_graph_pagerank:104, symbol_graph_status:127
commands_e41.rs: pr_review_status:80
commands_e46.rs: get_dual_ledger_state:99, get_recent_agentless_shortcuts:166, get_goal_drift_status:200, get_entropy_history:374
commands_e54.rs: get_eval_corpus_status:128, get_sycophancy_matched_pair_results:218, get_kor_orthogonality_status:294
commands_empire.rs: get_empire_status:79, list_empire_factory_lines:92
commands_f2.rs: get_security_blocks_recent:79, get_insecure_rate_latest:171
commands_f5.rs: get_staleness_quarantine:84
commands_f6.rs: get_skill_library:119
commands_f7.rs: get_template_ranking:89
commands_f9.rs: write_time_slop_status:143
commands_f10.rs: get_security_monotonicity_blocks:102
commands_f13.rs: get_session_archives:52
commands_g1.rs: get_taint_flows:48
commands_g2.rs: get_run_artifact:50
commands_g3.rs: get_blast_radius_blocks:53
commands_g4.rs: get_mcp_rug_pulls:49
commands_g5.rs: get_review_surface:52
commands_g6.rs: get_safety_delta_latest:74
commands_g7.rs: get_claim_confidence:55, get_trifecta_quarantines:73
commands_g8.rs: get_attestations:54
commands_g9.rs: get_redaction_stats:51
commands_g10.rs: get_model_drift_status:72
commands_h11.rs: read_dir:159, read_file:223, open_in_editor:242, mount_root:282, get_canvas_activity:311
commands_h13.rs: get_canvas_ops:66, capture_canvas_screenshot:118
commands_h14.rs: resolve_oid:109, apply_recast_patch:145, start_dev_proxy:182, stop_dev_proxy:208
commands_h15.rs: get_canvas_views:115, canvas_select_node:167, emit_canvas_checkpoint:220
commands_h35.rs: rewind_session:73
commands_h36.rs: get_recent_notifications:66
commands_h37.rs: get_plan_approval_decisions:68
commands_h41.rs: get_checker_gap_backlog:65, get_coverage_report:92
commands_h49.rs: timetravel_undo:271, timetravel_redo:331, timetravel_jump:387, timetravel_history:449, timetravel_restore_files_only:498
commands_h51.rs: resume_offer:59, tiered_load_confirm:107
commands_i33.rs: get_governance_metrics:56
commands_i38.rs: get_qdelta_latest:81
commands_vibegate.rs: list_vibegate_blocks:83
media_validation.rs: get_media_validations:419, get_media_validation_by_step:431, submit_media_validation_for_review:448, subscribe_media_ssb_events:476


================================================================================
SECTION 3 — STATE-LAYER MAP (the "good pattern" the redesign should EXTEND)
================================================================================
Each src/lib/state/*.svelte.ts store: what it owns | commands/events it wraps | readers.
The redesign should MOVE the inline-invoke panels (Section 5b) INTO this layer.

fsm.svelte.ts          | owns: live FSM snapshot | wraps: get_fsm_state (commands.rs:296) via stub.ts | readers: ActivityWindow, MonitorWindow, status surfaces
plan.svelte.ts         | owns: decomposition tree + plan/phase/baby-step progress | wraps: get_decomposition_tree:3768, get_plan_progress:400, get_phase_detail:10580, get_baby_step_detail:10604 | readers: PlanWindow, ActivityWindow
session.svelte.ts      | owns: gate mode + execution mode + session meta | wraps: get_gate_status:925, get_execution_mode:4840, set_execution_mode:4807 | readers: SettingsWindow, CockpitWindow, commands store
commands.svelte.ts     | owns: command-card history + chat/research/task input | wraps: start_chat_stream:5839 (streamChat), cancel_chat_stream:6242, run_research:6857, submit_task:499, query_knowledge_engine:4630, continue_task:4757, get_media_validations/by_step/submit (media_validation.rs) | readers: CommandWindow, ExploreWindow, chat surfaces
github.svelte.ts       | owns: PR list/status/review | wraps: github_pr_* (commands_c40.rs) + pr_review_status (commands_e41.rs:80) | readers: GithubWindow
fileTree.svelte.ts     | owns: mounted FS tree | wraps: read_dir (commands_h11.rs:159) | readers: FileExplorer
feature_flags.svelte.ts| owns: runtime feature flags + offline signal | wraps: get_feature_flags (commands.rs:1031) | readers: everywhere (gates DARK panels)
reindex.svelte.ts      | owns: indexing progress | wraps: reindex_now:6388, cancel_indexing:6505 + listen('reindex-progress') | readers: IndexingProgressPanel
media_validation.svelte.ts | owns: media-validation SSB feed | wraps: subscribe_media_ssb_events (media_validation.rs:476) + listen('media_ssb') | readers: MediaValidationPanel, commands store
drawers.svelte.ts      | owns: drawer-open state + research feed aggregation | wraps: many stub.ts fns + listen('research-progress'/'research-done') | readers: all drawers
grc.svelte.ts          | owns: GRC budget/grounding/CSQ | wraps: get_grc_budgets:2817, get_grounding_sweep:2868, get_csq_pending:2917, get_csq_log:2986 | readers: GRC surfaces in MonitorWindow
permission_prompts.svelte.ts | owns: pending permission prompts | wraps: stub.ts permission wrappers | readers: permission UI
canvas.svelte.ts / canvasActivity / canvasDispatcher / canvasViewState | owns: canvas-workbench graph + ops + views | wraps: get_graph_view_data:2636, get_canvas_activity (h11:311), get_canvas_ops (h13:66), get_canvas_views (h15:115), canvas_select_node (h15:167), emit_canvas_checkpoint (h15:220) | readers: CanvasWindow, ArchMode, DesignPageNode
graph.svelte.ts        | owns: client-side 2D graph layout (NO backend command — consumes VizData)
review.svelte.ts       | owns: client-side review verdict buffer (ReviewWindow does invokes INLINE — candidate to fold here)
workspace.svelte.ts    | owns: window/panel layout (NO backend)
activity.svelte.ts     | owns: activity-feed ring buffer (NO backend; fed by other stores)
motion.svelte.ts / ux_prefs.svelte.ts / shortcuts.ts | owns: motion prefs, UX prefs, keybinds (NO backend)

api/ layer (the wrappers the stores call):
api/stub.ts      | the central try-invoke-else-mock wrapper (tryInvoke:256); exports a named fn per command; falls back to api/mock-data.ts when TAURI_AVAILABLE is false or DASHBOARD_OFFLINE is set or invoke rejects. ~150 command wrappers live here.
api/mock-data.ts | the fake fixtures used as fallback (FSM_SNAPSHOT, DECOMP_TREE, ACTIVITY_ENTRIES, VRAM_GAUGES, SIDECAR_STATUS, etc. — ~40 exports).
api/mock-orbit.ts| orbit/graph fixtures (ORBIT_FIXTURE_SMALL/CONSTELLATION/RESEARCH/MAP, ORBIT_PAGERANK_NODES/EDGES).
api/empire.ts    | empire-bridge wrappers with their OWN try-invoke-else-mock (empire.ts:84) for get_empire_status / list_empire_factory_lines.
api/types.ts     | ALL hand-authored TS types (VizData, FsmSnapshot, DecompNode, and the DTOs). NOT ts-rs-generated.


================================================================================
SECTION 4 — I-PLAN PLANNED DASHBOARD ADDITIONS (design-against-spec)
================================================================================
From docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md frontend_pair fields. Each ADDS a row/badge/chip/tile to an EXISTING surface — the redesign must leave room. (N/A / W-6-justified entries omitted.)

I-1  directive supervisor enforcement -> supervisor verdict ROWS in an existing drawer (target: a verdict drawer / OrchestrationDrawer-family)
I-2  cow workspace isolation          -> branch workspace BADGES on BranchTree (target: SessionsDrawer / branch board)
I-7  cascading escalation + offline batch -> escalation cascade rows (target: ModeRoutingPanel / CostPanel family)
I-8  aletheia voluntary abort         -> abort/dead-end STRIP in ReasoningDrawer (target: a ReasoningDrawer — NOT YET PRESENT; redesign should add the reasoning drawer surface)
I-9  error amplification telemetry    -> alpha trend TILE on cockpit (target: CockpitWindow)
I-10 prefix invariance + ttl tiering  -> TTL/economics FIELDS on the H-9 cost tile (target: CostPanel)
I-11 pabu belief-state handoff        -> BeliefState inspector ROW in SessionsDrawer (target: SessionsDrawer)
I-12 evidence-gated playbook curator  -> playbook panel ROWS in RulesDrawer (target: a RulesDrawer — verify present; PinnedRulesPanel is the closest)
I-13 memory guard + dual serialization-> guard BADGES on MemoryFactsDrawer (target: MemoryFactsDrawer)
I-14 deterministic context compression-> per-transform receipt ROWS in TranscriptDrawer (target: TranscriptDrawer)
I-15 semantic cache + blake3          -> cache-source BADGE on research answers (target: ResearchDrawer / command cards)
I-23 unified conformal calibration    -> scorer-health BADGE (target: ConformalCalibrationPanel)
I-24 isomorphic canary anti-gaming    -> per-branch verdict CHIPS (cockpit family) (target: CockpitWindow / CanaryPanel)
I-25 ledger-trained router            -> route-explanation ROW on the cost cockpit (target: CostPanel)
I-26 dynamic thinking budgets         -> budget BAR on generation (target: CostPanel / command cards)
I-27 budgeted convergence             -> the curve TILE (target: a new cockpit tile)
I-28 dag credit assignment            -> credit (rows/tile) (target: OrchestrationDrawer — already shows credit_attribution_status)
I-29 diffusion draft lane             -> draft-lane BADGE on generation cards (target: command cards)
I-30 verifier-ranked sft exhaust      -> dataset CARD (counts + provenance) in a panel (target: a new training/dataset panel)
I-31 grpo flywheel                    -> flywheel status PANEL (admits/blocks/KL trend) (target: new panel; relates to ProposalsPanel/darwinian_admits)
I-33 mechanical decision governance   -> governance metrics TILE (target: new tile; handler get_governance_metrics commands_i33.rs:56 SHIPPED)
I-34 crane hybrid + grammar pinning   -> grammar-pin status in the existing decode panel (target: GrammarCompilePanel)
I-35 temporal tool-sequence enforce   -> sequence-verdict ROWS in the gate panel (target: ToolGateInspector)
I-36 cross-model prompt compilation   -> per-role compiled-variant BADGE in SettingsWindow (target: SettingsWindow)
I-37 ears requirement objects         -> per-requirement CHIPS in the cockpit (G-5) (target: CockpitWindow)
I-38 qdelta + statistical floor       -> QDELTA TILE next to SAFEDELTA tile (target: QdeltaPanel exists; place beside SafeDeltaPanel) [handler get_qdelta_latest commands_i38.rs:81 SHIPPED]
I-39 plan state from repo             -> (plan state surface; target PlanWindow)
I-40 trust-layer wrapper + rra spec   -> wrapped-run ROWS render in the existing RRA viewer (target: RraViewerPanel)
I-41 spec-time critique lenses        -> advisory findings LIST in ReasoningDrawer / B-40 framing panel (target: IdeaFramingWindow / new ReasoningDrawer)
I-42 critique-only operating profile  -> active profile in the StatusBar mode cluster (target: status bar; no new component)
I-44 zero-retention export mode       -> RetentionPurged ROWS in the RRA viewer (target: RraViewerPanel)
I-45 objective-hash sealed receipts   -> chain BADGE on the RRA viewer (target: RraViewerPanel)
I-46 bash interceptor + command fixup -> block reasons in the existing gate panel (target: ToolGateInspector / VibeGateDrawer)
I-49 bedrock + azure direct lanes     -> runtime backend rows (target: SettingsWindow runtime / list_runtimes)
I-50 subscription auth broker         -> auth (rows/badge) (target: SettingsWindow auth cluster)
I-52 checkpoint/rewind agent tool     -> checkpoint ROWS in the existing session drawer (target: SessionsDrawer / SessionArchiveDrawer)
I-53 recipe runners + utility pack    -> preview/apply ROWS in the existing tool-call drawer (target: a tool-call drawer)
I-21 sprint contracts + ralph loop    -> contract (panel/rows) (target: OrchestrationDrawer family)
I-20 budgeted mutation scheduler      -> per-hunk mutation CHIP (cockpit) (target: CockpitWindow / MutationArchivePanel)
I-4  evidence-ranked search ladder    -> progress sparkline per branch (branch board) (target: branch board / SessionsDrawer)
I-22 progress-shaped rewards          -> (reward surface; target OrchestrationDrawer / flywheel panel)
I-37/G-5 EARS                         -> per-requirement chips in cockpit (dup of I-37 above)
REDESIGN IMPLICATION: most additions land on CockpitWindow, CostPanel, SessionsDrawer, RraViewerPanel, the (not-yet-built) ReasoningDrawer, MemoryFactsDrawer, ToolGateInspector. Build those surfaces with row/badge/chip slots that scale.


================================================================================
SECTION 5 — SUMMARY + REDESIGN NOTES
================================================================================

5a. COUNTS (grounded)
- Components touching the backend (Section 1 rows, excl. pure-prop chips): ~63 .svelte/.ts components + ~16 state stores.
- invoke()/command call sites found: 77 invoke occurrences across 63 files (grep `invoke[<(]`). Plus ~150 named wrappers in api/stub.ts and ~6 in api/empire.ts.
- Distinct backend commands consumed by the dashboard: ~190 (full list Section 2). Total #[tauri::command] fns defined in src-tauri (incl. dev/unused): ~280.
- DATA-PATH SPLIT:
    INLINE (invoke directly in component, NO mock fallback): ~46 components — ALL the MonitorWindow leaf inspector panels + D/E/F/G/H bench drawers + GraphWindow/CanvasWindow/PlanWindow(intent)/ReviewWindow/CommandWindow + canvas nodes.
    STORE (via api/stub.ts wrapper; mostly mock fallback, a few no-mock): ~17 surfaces — SettingsWindow, CloudPanel, ProjectSelector, GithubWindow, ExploreWindow, ContextStatePanel, TrustTelemetryPanel, ProposalsPanel, MsiPanel, FileExplorer, SymbolGraphWindow, AuditTimelineWindow(no-mock), IdeaFramingWindow, IndexingProgressPanel(partial), RemoteSidecarBootOverlay(partial), EmpireBridgePanel(via empire.ts).
    MIXED: PlanWindow, ReviewWindow(reads via inline but has review store), MediaValidationPanel, ResearchDrawer, SessionsDrawer, ActivityWindow.
  => DOMINANT pattern is INLINE (~75% of feature panels call the backend inline), confirming the orchestrator's prior. This is the #1 redesign refactor target.
- ON MOCK DATA (renders api/mock-data.ts fallback when backend absent): the ~13 STORE-path surfaces + empire panel. INLINE panels do NOT mock — they show empty/error. mock-orbit.ts fixtures back the orbit/graph views (ActivityWindow, GraphWindow when no live VizData).
- DARK / unmatched backings:
    DARK(flag) per CLAUDE.md: CriticDecisionPanel (ELAI_CRITIC_OF_DESIGN_ENABLE), RoadmapPanel (ELAI_AI_ROADMAP_SIGNALS_ENABLE). These render empty unless the env flag is ON.
    BENCH-PENDING (handler exists, data populated only after a bench run): ARCBenchPanel, ModeRoutingBenchPanel, PrmBenchPanel, SycophancyBenchPanel, SafeDeltaPanel (G-6 SAFEDELTA), LoraShipGatePanel.
    UNMATCHED handler: PinnedRulesPanel calls get_pinned_rules — NO Rust #[tauri::command] of that name exists (UNVERIFIED, see 5d).

5b. PANELS WHOSE BACKEND IS DARK OR ON MOCK (redesign around REAL data, gate the rest)
- Flag-gated dark: CriticDecisionPanel, RoadmapPanel — keep the panel but show a clear "feature off (set ELAI_*_ENABLE=1)" empty state, don't fake data.
- Bench-pending: the *BenchPanel set + SafeDeltaPanel + LoraShipGatePanel — design a "no run yet" empty state; data is real once a bench runs.
- Mock-fallback (STORE path): every stub.ts-backed surface silently shows mock-data.ts in a browser/offline build. Redesign should make the mock-vs-live state VISIBLE (a dev-only "MOCK" ribbon) so a browser preview is never mistaken for live.

5c. INLINE PANELS THAT SHOULD HAVE invoke() EXTRACTED TO A STORE (the big refactor)
ALL ~48 INLINE panels duplicate the same `callInvoke<T>(cmd,args)` boilerplate (lazy `import('@tauri-apps/api/core')` + try/catch + console.warn). The redesign should:
  1. Add one typed wrapper per command in a single api layer (extend api/stub.ts or a fresh api/commands.ts), mirroring the existing stub.ts pattern but WITHOUT the mock fallback for bench/telemetry reads (or with an explicit opt-in mock).
  2. Give each feature family a thin store (e.g. benchStore, securityStore, memoryStore, canaryStore) that owns polling + caching, like fsm/plan/session do today.
  3. Highest-value extractions (most duplicated / most-shared): get_mast_breakdown, get_graph_view_data, get_checker_results_recent, get_recent_tripwires, get_novelty_recent, the whole D-bench family, OrchestrationDrawer's 4 inline calls.
  This removes ~48 copies of the same helper and makes the mock/live + loading/error/empty states uniform (the Section 5b visibility goal).

5d. UNVERIFIED (do NOT guess a shape in the redesign — resolve first)
- get_pinned_rules (PinnedRulesPanel.svelte:74) -> claimed PinnedRule[]. NO #[tauri::command] get_pinned_rules found in src-tauri/src/*.rs (grep returned nothing). Either the command is missing/renamed or registered elsewhere. RESOLVE before wiring PinnedRulesPanel.
- get_isolation_metric_status (referenced in api/stub.ts) -> NO Rust handler found. Likely a stub-only/mock-only wrapper. Verify before relying on it.
- Untyped return shapes: every handler returning CmdResult<Value> (serde_json::Value) — the front-end TS type in Section 1 is the front-end's ASSERTION, not a ts-rs contract. The redesign keeps the TS type but should treat it as unverified-against-Rust; the actual JSON shape is whatever the handler's serde_json::json!{...} builds. Notable untyped: most commands.rs get_* (get_grc_budgets, get_csq_*, get_evidence_gates, get_agents, get_supervision_log, get_context_budget, get_dkg_stats, get_recall_metrics, etc.) and all the get_*_status returning Value (get_safety_delta_latest, get_model_drift_status, get_staleness_quarantine, get_template_ranking, get_security_monotonicity_blocks, get_observable_effects_recent, get_conformal_validation_recent, get_mode_routing_refusal_combined). To get exact shapes, read each handler body (the json! macro) — out of scope for this sheet, flagged here.
- VERIFIED post-pass (call site + command + stub-wrapper confirmed): SymbolGraphWindow (symbolGraphQuery/PageRank/Status), AuditTimelineWindow (getPerLevelVerdicts/getTestIntegrityEvents), IdeaFramingWindow (get_idea_framing_state), ProposalsPanel (get_pending/applied_proposals + apply/reject), MsiPanel (get_msi_status). All STORE-path via api/stub.ts. The remaining UNVERIFIED items are only the get_pinned_rules / get_isolation_metric_status missing handlers and the CmdResult<Value> JSON shapes (read each handler's json! body to pin exact fields).

5e. EVENT CHANNELS (8 — the redesign must re-subscribe these)
symbol_graph_updated (GraphWindow refetch) | media_ssb (media_validation store) | reindex-progress (reindex store) | research-progress + research-done (drawers store) | remote-sidecar-progress (boot overlay) | reflect-progress (chat reflect) | mini-critic-verdict (critic strip). Chat token streaming is command-driven (start_chat_stream emits token events drained by lib/streaming/frame_pacer.ts), not a named listen channel.
