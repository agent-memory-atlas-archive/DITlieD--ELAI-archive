DASHBOARD MOCKUP INVENTORY
task_id: mockup-inventory-v1
purpose: ONE master, designer-oriented inventory of EVERYTHING across EVERY plan (B-K, shipped
  AND not-yet-built) that talks to the dashboard, and WHAT DATA each feature sends to the UI.
  This is the UNION of what is wired today and what every plan intends to surface. Grouped by what
  the USER SEES, not by plan letter. Built by REUSING the three companion artifacts:
    - CONTRACT-SHEET.md      (panel -> command -> handler, data_path, on_mock, status)  [shipped wiring]
    - HANDLER-SHAPES.md      (the real json!/DTO field shapes each shipped command sends)
    - GAP-ANALYSIS.md        (buckets A/B/C/D + the I-plan / J / K dashboard map)
  Plus a frontend_pair grep across B-K plan indexes for PLANNED additions.

HOW TO READ AN ENTRY
  name | display_type | target_surface | SENDS: data fed to UI | interaction | status | source
  display_type   = window | drawer | panel | tile | badge | chip | row | strip | overlay | gauge | bar | view
  target_surface = the window/drawer it lives in, or NEW
  SENDS          = shipped: field list summarized from HANDLER-SHAPES (cited); planned: what the plan
                   frontend_pair SAYS it will send (COARSE = plan is vague, transcribed verbatim, not invented)
  interaction    = read-only | has-actions: <list>
  status         = WIRED | DARK(flag) | BENCH-PENDING | NO-PANEL-YET | PLANNED(plan-id) | DEFERRED
  source         = plan id / handler file:line

GROUND-TRUTH RULE (from GAP-ANALYSIS): where the feature map and contract sheet disagree on a
  dashboard panel, the CONTRACT SHEET wins (live grep). Two known-stale feature-map notes already
  corrected: ContextStatePanel + TrustTelemetryPanel ARE mounted (empty unless ELAI_DEVELOPER_
  DISCOURSE_ENABLE=1); LoraShipGatePanel EXISTS + is mounted (bench-pending).


================================================================================
SECTION 1 — TOP-LEVEL WINDOWS (existing + planned)
================================================================================
Each window = its role + the panels/surfaces it hosts. (file paths under
src/lib/windows|screens unless noted.)

--- EXISTING (shipped) ---

CockpitWindow            | the primary cockpit: review-surface cards + hosts intervention/gate widgets.
  SENDS: get_review_surface -> ReviewSurfaceRow[] (seq, ts_ms, session_id, dsr[0..1], total_changed_lines,
    attention_lines, vouched_hunks, unvouched_hunks, base_commit, diff_available, diff_unavailable_reason)
    [HANDLER-SHAPES commands_g5.rs:52].
  interaction: read-only (hosts action widgets as children).  status: WIRED (G-5).  source: CockpitWindow.svelte / commands_g5.rs:52
  PLANNED additions land here heavily — see Section 3 (I-9 alpha-trend tile, I-24 canary chips, I-37 EARS chips, I-20 mutation chip).

MonitorWindow           | the monitoring AGGREGATOR (1347 lines); pure layout hosting dozens of leaf panels.
  Hosts: ProposalsPanel, MsiPanel, MediaValidationPanel, MastFailurePanel, CriticDecisionPanel, RoadmapPanel,
    CostPanel, EvalCorpusPanel, SycophancyMatchedPairPanel, ContextStatePanel, TrustTelemetryPanel,
    LoraShipGatePanel, + the GRC surfaces, + most leaf inspector panels in Section 2.
  SENDS: none itself (children inline-invoke).  interaction: read-only layout.  status: container.  source: MonitorWindow.svelte

PlanWindow              | plan / decomposition tree + intent-pair (focus tunnel) editor.
  SENDS: get_decomposition_tree (DecompositionTreeSnapshot, top-level concepts[] -> phases/steps/babysteps),
    get_plan_progress (phases[], current_phase, total_pct), query_focus_tunnel_per_baby_step (FocusTunnelPerStep),
    get_intent_pairs -> IntentPairSetLite.
  interaction: has-actions: add_intent_pair, confirm_intent_pairs.  status: WIRED.  source: PlanWindow.svelte / commands.rs:3768,400,8385,11067
  PLANNED: I-39 plan-state-from-repo surface lands here.

GraphWindow             | 2D/symbol dependency graph + impact overlay + mermaid export.
  SENDS: get_graph_view_data -> VizData (nodes/edges/layout; TS mirror api/types.ts), get_impact_overlay
    (files[], by_depth{}, total_files, max_depth_reached, truncated), export_graph_mermaid -> mermaid string.
  events: symbol_graph_updated -> refetch.  interaction: read-only (graph interactions client-side).
  status: WIRED (GD-1).  source: GraphWindow.svelte / commands.rs:2636,2687,2753

SymbolGraphWindow       | SCIP symbol-graph view (defs/refs/callers/callees + pagerank + status).
  SENDS: symbol_graph_query -> SymbolGraphQueryReport (kind, symbol, result_count, results[SymbolMatch],
    graph_path, node_count), symbol_graph_pagerank -> SymbolGraphPageRankReport (seed, k, ranked[RankedSymbol],
    node_count), symbol_graph_status -> {graph_path, graph_present, node_count, lru_capacity}.
  events: symbol_graph_updated.  interaction: has-actions: query input.  status: WIRED (E-40).  source: SymbolGraphWindow.svelte / commands_e40.rs:67,104,127

CanvasWindow            | canvas-workbench code-graph host (cytoscape/WebGL).
  SENDS: get_graph_view_data{mode:symbol} -> VizData, get_pagerank_top -> {known, rows[{symbol,path,score}]},
    get_canvas_activity -> CanvasActivityEventDto[], get_canvas_ops -> CanvasOpDto[], get_canvas_views ->
    CanvasViewEventDto[].
  interaction: has-actions: canvas_select_node (taint-gated handoff), emit_canvas_checkpoint.  status: WIRED (H-11..H-15).
  source: CanvasWindow.svelte + canvas*.svelte.ts / commands.rs:2636, commands_h11/h13/h15.rs
  PLANNED: J-31 "system" composition mode (Section 3); H-Track-2 canvas rebuild (demoted, Section 4).

ReviewWindow            | per-file change review (accept/decline/retry verdicts).
  SENDS: get_file_changes -> file-change rows; accept/decline/retry return {ok, seq}.
  interaction: has-actions: accept_change, decline_change, retry_audit{note}.  status: WIRED.  source: ReviewWindow.svelte / commands.rs:10660,10740,10752,10764

CommandWindow           | command palette / time-travel commands.
  SENDS: timetravel_undo/redo/restore_files_only -> TimeTravelResponse (disabled, no_op, beacon_seq, head,
    summary, divergence_detected); timetravel_history -> TimeTravelNode[] (snap_id, parent_id, is_head,
    children_count, tag, created_at_ms, manifest_hash).
  interaction: has-actions: undo, redo, jump, restore-files-only.  status: WIRED behind ELAI_TIMETRAVEL_DISABLE (H-49).  source: CommandWindow.svelte / commands_h49.rs:271,331,498
  PLANNED: H-51 resume-offer (tiered handoff load) + H-35 rewind land in CommandWindow (Section 3).

ActivityWindow          | orbit/activity visualization of FSM + plan; hosts InterventionControlBar.
  SENDS: reads fsm.svelte (get_fsm_state) + plan.svelte stores; renders OrbitCanvas.  interaction: hosts intervention bar.
  status: WIRED (orbit).  source: ActivityWindow.svelte  NOTE: orbit retirement rides canvas-default (H-Track-2, demoted, not killed).

SettingsWindow          | model routing, gate mode, sidecar, local budget, webhooks, oauth, execution mode.
  SENDS/actions: fetch_settings_get/set, set_gate_mode, set_model_routing (-> [{role,target}]),
    set_local_max_context, set_local_min_prompt, set_mcp_enabled, set/get_execution_mode, set/get_cloud_model,
    set/get_webhook_url, get_oauth_status/start_oauth_login/do_oauth_logout, get/set_sidecar_settings, restart_sidecar.
  interaction: has-actions: all of the above (settings mutations).  status: WIRED.  source: SettingsWindow.svelte / commands.rs (many)
  PLANNED additions here: I-36 per-role compiled-variant badge, I-49 bedrock/azure runtime rows, I-50 auth broker rows, J-30 BYOK config (Section 3).

ExploreWindow           | knowledge-engine / explore query console.
  SENDS: run_explore_query{query} -> Value, query_knowledge_engine{query} -> Value.
  interaction: has-actions: query input.  status: WIRED.  source: ExploreWindow.svelte / commands.rs:4706,4630

GithubWindow            | GitHub PR management + PR-review status.
  SENDS: github_pr_open/list/status/close/get_active_url (PrSummary/PrStatus 1-hop; active-url ->
    {url, pr_number, ts_ms}); pr_review_status -> PrReviewStatusDto (recent_reviews[], last_watcher_tick,
    last_slopsquat, last_semgrep, last_auto_merge_block — each with pr_number_hash + ts_ms).
  interaction: has-actions: open/close PR.  status: WIRED (C-40, E-41).  source: GithubWindow.svelte / commands_c40.rs, commands_e41.rs:80
  PLANNED: K-2 evidence-pack-as-PR-comment surface rides github.svelte.ts (Section 3).

AuditTimelineWindow     | per-level audit timeline (B-41 auditor modes); LevelAuditVerdictJson[] with cascade arrows.
  SENDS: get_per_level_verdicts(taskId) -> LevelAuditVerdictJson[] (mode, tier_node_id{tier,id}, findings[]
    {category,severity,message,source}, verdict, respawned_once, cascade_to_parent); get_test_integrity_events.
  interaction: read-only.  status: WIRED (B-41), no-mock (empty not fake).  source: AuditTimelineWindow.svelte / commands.rs:9978,10024
  PLANNED additions: J-16 blast-radius panel, K-5 gate-intensity rows land on this surface (Section 3).

IdeaFramingWindow       | B-40 idea-framer (+ IdeaFramerSecurityPanel).
  SENDS: get_idea_framing_state -> IdeaFramingStateDto (session_id, framed_idea[json], critic_rounds_used,
    is_converged, is_active).
  interaction: read-only.  status: WIRED (B-40).  source: IdeaFramingWindow.svelte / commands.rs:8552
  PLANNED: I-41 spec-time critique-lens advisory list lands here (Section 3).

QdeltaPanel (window-level)| QDELTA statistical-floor tile (I-38).
  SENDS: get_qdelta_latest -> QdeltaLatest | null (suite, model_id, n_tasks, seed_count, paired,
    on/off_defective_rate, delta, ci_low, ci_high, dollar_per_task, contamination_probe_result,
    surviving_mutants_per_task, corpus_blake3, runpod_pending, ts).
  interaction: read-only.  status: WIRED behind flag / BENCH-PENDING (I-38).  source: QdeltaPanel.svelte / commands_i38.rs:81

RraViewerPanel (window) | RRA (reproducible-run-artifact) viewer (G-2).
  SENDS: get_run_artifact -> RunArtifactRow[] (seq, ts_ms, root_blake3, guarantee_level, gate_count, seed,
    session_id, objective_hash?).
  interaction: read-only.  status: WIRED (G-2).  source: RraViewerPanel.svelte / commands_g2.rs:50
  PLANNED additions: I-40 wrapped-run rows, I-44 RetentionPurged rows, I-45 objective-hash chain badge (Section 3).

ProjectSelector (screen)| project selection.
  SENDS/actions: list_projects, create_project, open_project_from_folder.  status: WIRED.  source: ProjectSelector.svelte
CloudPanel (screen)     | cloud routing + spend.
  SENDS: get_cloud_model/set_cloud_model, get_cloud_spend, get_cloud_route_status.  status: WIRED.  source: CloudPanel.svelte / commands.rs:5167..5242
  PLANNED: H-9 token-tracker activates CloudPanel to render NON-zero token counts (Section 3/4).
RemoteSidecarBootOverlay (screen) | remote sidecar boot.
  SENDS/actions: start_remote_sidecar, get_remote_sidecar_progress, cancel_remote_sidecar.  events: remote-sidecar-progress.
  status: WIRED.  source: RemoteSidecarBootOverlay.svelte

--- PLANNED NEW WINDOWS ---

UsageWindow             | NEW deep token-usage view: per-model/task/session token consumption, cache effectiveness,
    tool/file-op counts.  SENDS: per-turn token ledger (UsageTurnEvent: capture/semantics/pricing-corrected token
    counts per cloud+local turn) [COARSE — J-28/J-29 frontend_pair "the new token-tracker component"].
  interaction: read-only.  status: PLANNED(J-28/J-29).  source: J-SUMMARY_INDEX.md:653-655 + GAP-ANALYSIS bucket D
LiveUsageGauge          | NEW mid-stream live token meter (companion to UsageWindow).  SENDS: live in-flight token
    count + window pct [COARSE].  interaction: read-only.  status: PLANNED(J-29).  source: GAP-ANALYSIS bucket D / J-29

canvas "system" mode    | NEW mode on the shipped canvas: nodes = buildable units, edges = verified wiring claims.
  SENDS: system-composition graph from system_canvas.rs [COARSE].  interaction: read-only graph.  status: PLANNED(J-31).
  source: GAP-ANALYSIS bucket D / J-31
front-door cards        | NEW chat-as-ELAI surface: ACK / notice / RESULT cards + "Open in canvas ->" deep-link.
  SENDS: IntentRouted (Pattern-2: Classified/Confirmed/Rejected/Bypassed/Clarified) [from front_door.rs at
    start_chat_stream + live_cli run_turn seams].  interaction: has-actions: confirm/clarify/open-in-canvas.
  status: PLANNED(J-32).  source: J-SUMMARY_INDEX.md:741-743


================================================================================
SECTION 2 — PANELS / DRAWERS BY DOMAIN (shipped; SENDS shape from HANDLER-SHAPES)
================================================================================

--- 2a. RUN-STATE & ORCHESTRATION ---
GatesPanel              | panel | MonitorWindow | get_ambiguity_state (score, milestone, threshold, brownfield,
  components{goal_clarity/constraint_clarity/success_criteria_clarity/context_clarity each {score,weight,floor}},
  floor_failures[], streak{current,required}, qualifies_for_advance) | read-only | WIRED | commands.rs:9529
OrchestrationDrawer     | drawer | MonitorWindow | get_ambiguity_state + get_stagnation_events (StagnationEventDto[]:
  timestamp_ms, action_type, agent_id, pattern[spinning|oscillation|no_drift|diminishing_returns], persona,
  drift_score, elapsed_ms, detail) + credit_attribution_status ({rows[{role,score}], ts_ms, runpod_pending}) +
  routing_decisions_status ({rows[{task_id,difficulty,alpha,dispatched[{role,primitive}],is_lazy_mode}]}) | read-only |
  WIRED (credit/routing synthetic until runpod) | commands.rs:9529,8657,11090,11169
ProposalsPanel          | panel | MonitorWindow | get_pending_proposals + get_applied_proposals (SelfModProposalRow:
  proposal_id, table, fpr_estimate?, occurrences_seen?, pipeline_mode, applied_by?, ts_ms) | has-actions:
  apply_proposal_from_user, reject_proposal | WIRED/DARK (self-mod feature gated) | commands.rs:1144,1162,1274,1302
MsiPanel                | panel | MonitorWindow | get_msi_status (SelfModMsiSnapshotRow: pipeline_mode, msi_value,
  status, ts_ms) + get_steering_status (fixed placeholder: active, vector_label, last_computed_ts, magnitude,
  applied_count) | has-actions: recompute_steering_vector | DARK/PARTIAL (steering gated) | commands.rs:1315,1324,1335
OptimizerRunPanel       | panel | MonitorWindow | get_optimizer_run ({rows[{seq,ts,phase,iteration_index,
  val_lift_pct?,held_out_lift_pct?}], latest_phase?, latest_val_lift_pct?, latest_held_out_lift_pct?, overfit_risk,
  reflection_lm_too_small}) | read-only | WIRED | commands.rs:11425
PlanningLedgerPanel     | drawer | MonitorWindow | get_dual_ledger_state (tasks[{task_id,parent_id,completion_pct,
  status,latest_blocker,last_update_ms}], n_complete, n_blocked) + get_recent_agentless_shortcuts ([{task_id,reason,
  ts_ms}]) + get_goal_drift_status (firings[{initial_goal_blake3,drift,threshold,step,ts_ms}], firings_by_canary{},
  latest_drift, latest_threshold) | read-only | WIRED (E-46) | commands_e46.rs:99,166,200
DecompositionEntropyHistogram | drawer | MonitorWindow | get_entropy_history (EntropyHistogramRow[]: parent_node_key,
  entropy_ratio[0..1], h_parent, h_children, threshold, ts_ms) | read-only | WIRED (E-46) | commands_e46.rs:374
BabyStepPanel           | drawer | MonitorWindow | get_baby_step_dispatches_recent (rows[{step_id,parent_task_id,
  checker_kinds_used[],outcome,ts_ms}], counts_by_outcome{}, counts_by_kind{}, total) | read-only | WIRED (D-12) | commands_d12.rs:66
InterventionControlBar  | bar | ActivityWindow | (consumes pending-intervention props) | has-actions:
  submit_intervention_response, respond_to_confirmation, submit_credential_response | WIRED | commands.rs:532,563,585
GrammarCompilePanel     | panel | MonitorWindow | get_grammar_compilations (PLACEHOLDER until sidecar producer:
  rows:[], producer_status:"pending_sidecar_wiring", expected_total:64, note, pin_status:"not_applicable") | read-only |
  WIRED (producer-pending) | commands_e35.rs:39   PLANNED: I-34 grammar-pin status decorates this (Section 3).
EmpireBridgePanel       | panel | MonitorWindow | get_empire_status ({state[unavailable|disconnected|connecting|
  connected], feature_enabled, factory_line_count, note}) + list_empire_factory_lines (FactoryLineDto[]: brain_id,
  engine_tier[MKI|MKII|MKIII], neuron_count, last_heartbeat_ms, status) | read-only | WIRED/PARTIAL (mock fallback) | commands_empire.rs:79,92
IndexingProgressPanel   | panel | MonitorWindow | get_indexing_progress (status, project_id, project_path,
  indexed_files, total_files, batch_size, batches_processed, batches_total, last_checkpoint_at, schema_version) +
  get_indexing_resilience (auto_resume_enabled, watch_locks_held, last_checkpoint_unix_secs, last_resume_from_batch,
  last_resume_files_skipped, lock_contested_count, shutdown_deadline_secs, last_batch_duration_ms) | read-only +
  events reindex-progress | WIRED | commands.rs:6540,6627
SessionsDrawer          | drawer | MonitorWindow | get_session_branches (list) + fork/merge/close branch ops |
  has-actions: fork_session_branch, merge_session_branch, close_session_branch | WIRED | commands.rs:3413,3584,3677,3701
  PLANNED additions: I-2 branch workspace badges, I-11 belief-state inspector row, I-52 checkpoint rows, I-4 progress sparkline (Section 3).
SessionArchiveDrawer    | drawer | MonitorWindow | get_session_archives (SessionArchiveRow[]: seq, ts_ms, session_id,
  stale_version, archive_path, manifest_blake3, store_kind[ssb|session_dag]) | read-only | WIRED (F-13) | commands_f13.rs:52
ResearchDrawer          | drawer | MonitorWindow | get_research_node_history (research_node_stages rows, 1-hop) |
  events research-progress, research-done | read-only | WIRED | commands.rs:10703
  PLANNED: I-15 semantic-cache-source badge on research answers (Section 3).
QueryDrawer             | drawer | MonitorWindow | get_query_trace_recent ([{seq,ts,query_id,tier,stage_count,
  result_count,total_latency_ms}]) | read-only | WIRED | commands.rs:1409
PagerankPanel           | drawer | MonitorWindow | get_pagerank_top ({known, rows[{symbol,path,score}]}) | read-only | WIRED | commands.rs:1728

--- 2b. COST / ROUTING / EFFICIENCY ---
CostPanel               | panel | MonitorWindow | get_velocity_cap_status (windows[{window[one_min|ten_min|one_hour|
  twenty_four_hour],count,limit,kind[requests|usd_cents],ts_ms}], grand_total) + get_cost_router_recent
  (CostRouterDecisionRow[]: session_id?, task_id?, difficulty, budget_remaining_usd, chosen_backend, criticality,
  rationale, ts_ms) + get_automix_verdicts_recent ([{session_id?,task_id?,verdict,reason,ts_ms}]) +
  get_free_tier_quotas ([{backend,status,remaining_pct,ts_ms}]) | read-only | WIRED/PARTIAL (E-39, some runpod-pending) |
  commands_e39.rs:175,237,291,335
  PLANNED additions (heavy): I-7 escalation cascade rows, I-10 TTL/economics fields, I-25 route-explanation row,
  I-26 thinking-budget bar (Section 3).
ContextStatePanel       | panel | MonitorWindow | get_context_state (ContextStateDto: fsm_state, substate?,
  token_budget_pct, mcp_think_topics[{topic,remaining_calls,remaining_chars}], last_precompact_thresholds[],
  prompt_injection_check_status[passed|blocked|not_run], runtime_choice_badge[local|cloud_billed], current_role?,
  current_runtime?) | read-only, polled 5s | WIRED but EMPTY unless ELAI_DEVELOPER_DISCOURSE_ENABLE=1 (H-50) | commands.rs:8798
  PLANNED: K-6 active-threshold/curve tile lands in the ContextStatePanel/cockpit family (Section 3).
GRC surfaces            | panel(s) | MonitorWindow | get_grc_budgets ([{role,calls_remaining,chars_remaining,
  total_calls,total_chars,calls_used,chars_used}]) + get_grounding_sweep + get_csq_pending + get_csq_log |
  read-only | WIRED | commands.rs:2817,2868,2917,2986

--- 2c. TRUST & SAFETY ---
TrustTelemetryPanel     | panel | MonitorWindow | get_trust_telemetry (TrustReportDto: tasks_per_hour,
  findings_per_task, revert_rate, ungrounded_count, stagnation_dist{pattern:count}) | read-only | WIRED but EMPTY
  unless ELAI_DEVELOPER_DISCOURSE_ENABLE=1 | commands.rs:8981
PromptArmorPanel        | panel | MonitorWindow | safety_status (SafetyStatusDto: injection_detected/blocked_count,
  spotlight_wrapped/breached_count, pin_mismatch_count, capability_rejected_count, provenance_tagged_count,
  prompt_armor_verdict_count, fixture_replayed_count, pin_dir) + list_safety_rows ([{action_type,ts_ms}]) | read-only |
  WIRED (E-37) | commands_e37.rs:77,119
AuthzPanel              | panel | MonitorWindow | gateway_authz_status (AuthzStatusDto: policy_file, default[allow|deny|
  unavailable], roles[{role,tools[],doc_owners[]}], feature_enabled, note) + gateway_authz_recent_denies
  (AuthzDenyRowDto[]: role_hash, tool_hash, deny_reason_label, deny_reason_hash, ts_ms) + gateway_authz_run_fuzz
  (AuthzFuzzReportDto: passed, failed, failures[], budget, full_500k_status, paper) | has-actions: run_fuzz |
  WIRED (C-42) | commands_c42.rs:106,115,158
McpStatusPanel          | panel | MonitorWindow | get_mcp_security_status -> McpSecurityStatus[] (UNRESOLVED-SHAPE,
  body not read) | read-only | WIRED | commands.rs:261
McpRugPullDrawer        | drawer | MonitorWindow | get_mcp_rug_pulls (McpRugPullRow[]: seq, ts_ms, server_url,
  schema_blake3_before/after, changed_tools[], description_diff_count, ingest_origin) | read-only | WIRED (G-4) | commands_g4.rs:49
TaintFlowDrawer         | drawer | MonitorWindow | get_taint_flows (TaintFlowRow[]: seq, ts_ms, kind[web|tool_output|
  file|sandbox_output], origin_blake3, trust_level, sink_tool, blocked, task_id) | read-only | WIRED (G-1) | commands_g1.rs:48
BlastRadiusDrawer       | drawer | MonitorWindow | get_blast_radius_blocks (BlastRadiusBlockRow[]: seq, ts_ms,
  kind[fs_escape|egress_denied], pre/post_expansion_command, target_or_host, worktree_root_blake3, trust_label) |
  read-only | WIRED (G-3) | commands_g3.rs:53
ConfidenceDrawer        | drawer | MonitorWindow | get_claim_confidence (ClaimConfidenceRow[]: seq, ts_ms, claim_id,
  claim_score[0..1], logprob_component?, self_consistency_component, exec_agreement_component, lane[local|cloud], tau,
  conformal_verdict[abstain|answer], abstained) + get_trifecta_quarantines (TrifectaQuarantineRow[]: seq, ts_ms,
  task_id, untrusted_origin_blake3, sink_tool, secret_present, untrusted_span, egress_available, quarantined,
  sub_llm_verdict) | read-only | WIRED (G-7) | commands_g7.rs:55,73
RedactionPanel          | drawer | MonitorWindow | get_redaction_stats (RedactionStatRow[]: seq, ts_ms, session_id,
  field, tokens[], match_count, kind_counts[(label,count)]) | read-only | WIRED (G-9) | commands_g9.rs:51
TripwireStatusStrip / TripwirePanel | strip / drawer | MonitorWindow | get_recent_tripwires (TripwireRow[]: seq, ts,
  kind[fired|auto_reverted|fired_concurrent], event_id?, tripwire_kind?, source?, candidate_id?, observed_value?,
  threshold?, fired_at_ms?, template_id?, parent_template_id?, retirement_reason?, fired_tripwire_id?, tripwire_a/b?,
  fired_within_ms?) | read-only | WIRED (D-4) | commands_d4.rs:64
IncidentsDrawer         | drawer | MonitorWindow | get_recent_incidents (IncidentRow[]: seq, ts, kind[8 arms:
  argv_destructive_denied|argv_approval_required|trust_dialog_*_deferred|mcp_pin_version_refused|auditor_lineage_
  collision|untrusted_memory_write_blocked|incident_record_created], + per-kind optionals argv_hash/reason/citation/
  defer_reason/server_name/pinned_version/observed_version/auditor_lineage/executor_lineage/text_source/incident_id/
  title/source_url) | read-only | WIRED (E-30) | commands_e30.rs:80
LockedKeysPanel         | drawer | MonitorWindow | get_locked_keys_status (LockedKeysStatusDto: registry_path,
  n_entries, registry_blake3, loaded_ts_ms, entries[{path,line_range[],reason}], error?) + get_locked_keys_recent_audits
  (runs[{audit_id,ts_ms,n_audited,n_violations,registry_blake3,registry_path}], violations_by_audit{}) | read-only |
  WIRED (D-9) | commands_d9.rs:86,125
ToolGateInspector       | panel | MonitorWindow | get_tool_gate_snapshot (ToolGateSnapshotDto: fsm_state, eager[],
  gated[], denied[], feature_enabled, note) + list_tool_gate_rows (ToolGateRowDto[]: tool_name, fsm_state,
  prompt_tokens_saved, action_type, ts_ms, + optionals commit_hash/allowed_categories/denied_tool/taint_source) |
  read-only | WIRED (C-35) | commands_c35.rs:102,130
  PLANNED additions: I-35 sequence-verdict rows, I-46 bash-interceptor block reasons (Section 3).
ToolReliabilityInspector| panel | MonitorWindow | tool_reliability_status (ToolReliabilityStatusDto: inline_cap_bytes,
  default_retry_cap, spill_dir, pass/reject/spill/escalate_count, spill_queue_bytes) + list_tool_reliability_rows
  (ToolReliabilityRowDto[]: action_type, tool_name_hash, ts_ms, + optionals reject_reason_hash/spill_path_hash/
  attempt/escalate_to_role_hash/payload_bytes) | read-only | WIRED (E-18) | commands_e18.rs:88,151
CaseBaseInspector       | panel | MonitorWindow | tool_calibration_status (ToolCalibrationStatusDto[]: tool_name,
  shape_count, total_calls, error_count, last_ts_ms) + list_tool_calibration_rows (ToolCalibrationRowDto[]:
  tool_name_hash, call_shape_hash, action_type, ts_ms, + optionals post_call_audit_ok/error_rate/total_calls) |
  read-only | WIRED (C-47) | commands_c47.rs:75,112
VibeGateDrawer          | drawer | MonitorWindow | list_vibegate_blocks (VibeGateBlocksResponse: rows[{seq,ts,
  kind[CodeVuln|Secret|DepHallucinated|Skill],rule_id?,cwe?,file_path,source_blake3,citation?,fsm_state?}],
  env_active) | read-only | WIRED | commands_vibegate.rs:83
SecurityRulesPanel      | drawer | MonitorWindow | get_security_blocks_recent (SecurityBlocksResponse: rows[{seq,ts,
  rule_id,cwe,file_path,language,candidate_blake3}], env_active) + get_insecure_rate_latest (InsecureRateRow|null:
  suite, model_id, n_samples, insecure_rate_before/after, gate_recall, ts) | read-only | WIRED (F-2) | commands_f2.rs:79,171
SecurityMonotonicityDrawer | drawer | MonitorWindow | get_security_monotonicity_blocks (rows[{seq,ts,loop_kind[refine|
  auto_advance|self_mod],loop_id,introduced_classes[],prior_classes[],delta_blake3,lang}], env_active) | read-only |
  WIRED (F-10) | commands_f10.rs:102
StalenessQuarantineDrawer | drawer | MonitorWindow | get_staleness_quarantine (rows[{seq,ts,fact_id,staleness_ratio,
  dependents_demoted,cosmos_artifacts_flagged,dropped_count}]) | read-only | WIRED (F-5) | commands_f5.rs:84
FreezeStatusBar         | widget/bar | global | scope_freeze_status (FreezeStatusDto: clean, ratified_at_ms,
  verified_at_ms, violation_count, violations[{item_path,item_path_hash,frozen_since_ms,modified_at_ms}], error?) |
  read-only | WIRED (C-43) | commands_c43.rs:84
SycophancyFlagStrip     | strip | MonitorWindow | get_sycophancy_flags_recent (SycophancyFlagRow[]: seq, ts_ms,
  claim_id, flip_detected, delta, verdict[convergent|divergent_sycophancy_flag|inconclusive]) | read-only | WIRED (D-27) | commands_d27.rs:288
ClaimConfidenceChip / SycophancyGuardBadge | chip / badge | inline (parent-driven) | (prop-driven: evidence_ref
  {kind:ssb_seq,value}) | read-only | WIRED (G-7/E-54 data) | presentational
RefusalRateGauge        | gauge | MonitorWindow | get_refusal_sla_recent (RecentRefusalSla: snapshot_rows[{refusal_rate,
  window_total,abstain_count,health_band,ts_ms}], breach_rows[{rate,threshold,breach_kind,ts_ms}], is_breaching) +
  get_refusal_rate_current (UNRESOLVED) | read-only | WIRED (D-17) | commands_d17.rs:70,127
CriticDecisionPanel     | panel | MonitorWindow | get_critic_decision_state (CriticDecisionState: tiles[{phase,
  baseline_score,challenger_score,paired_bootstrap_p?,sample_size,verdict_code[0=Keep,1=Simplify],ts_ms}],
  grand_total) | read-only | DARK(ELAI_CRITIC_OF_DESIGN_ENABLE) | commands_e10.rs:61
RoadmapPanel            | panel | MonitorWindow | get_roadmap_signals_state (RoadmapSignalsState: last_compliance
  {kind,assertion_label,hint,assertions_checked,plan_session_id,ts_ms}?, mcp_transport{current,target,ts_ms}?,
  posture_stale{posture_doc_path,mtime_ms,age_days,ts_ms}?, mrcr_unverified_count, grand_total) | read-only |
  DARK(ELAI_AI_ROADMAP_SIGNALS_ENABLE) | commands_e38.rs:87

--- 2d. VERIFICATION & BENCHMARKS ---
MastFailurePanel / FailureModesDrawer | panel / drawer | MonitorWindow | get_mast_breakdown (MastBreakdownResponse:
  rows[{mast_mode,agent_role,count}], totals[{mast_mode,total}], grand_total) + get_iso_budget_recent
  (IsoBudgetMeasuredDto[]: task_fixture, budget_tokens, full_pipeline_pass_rate, single_agent_wait_pass_rate,
  insufficient_lift, runpod_pending, mast_mode_counts[], unique_catch_rate{}, decorrelation_score{},
  insufficient_contribution{}) | read-only | WIRED (E-6, H-40 fields) | commands_e6.rs:80,206
VerificationConsensusPanel | drawer | MonitorWindow | get_recent_verifications (VerificationConsensusRow[]: seq, ts,
  kind[pass|fail|ppm_unavailable|verifier_disagreement|cross_context_dispatched], candidate_id, template_id, verdict,
  rule_id, verifier_count?, pass_count?, failing_verifier_ids[], reason?) | read-only | WIRED (D-3) | commands_d3.rs:52
PpmScoresPanel          | drawer | MonitorWindow | get_ppm_recent_scores (PpmScoreRow[]: seq, ts, kind[scored|
  gating_failed], candidate_id, verifier_id, mode, score_mean, score_var?, num_steps, threshold?, latency_ms?,
  cache_hit?) | read-only | WIRED (D-2) | commands_d2.rs:49
CheckerStatusPanel / CheckerRegistryPanel | panel / drawer | MonitorWindow | get_checker_results_recent
  (RecentCheckerResults: rows[{checker_id,checker_kind,result_kind,claim_id,ts_ms,reason}], counts_by_kind{}, total) +
  get_checker_registry_state (kinds[{kind,description,builtin_available}]) | read-only | WIRED (D-11) | commands_d11.rs:124,179
CheckerCoveragePanel    | panel | MonitorWindow | get_coverage_report (CoverageReportDto: mode_a_fraction,
  constructive_count, total_factual_computational, session_id) + get_checker_gap_backlog (CheckerGapBacklogRowDto[]:
  claim_type, complexity_taxon, count, priority[1|2]) | read-only | WIRED (H-41) | commands_h41.rs:65,92
ConformalCalibrationPanel | drawer | MonitorWindow | get_conformal_recent (RecentConformal: tau_rows[{tau,epsilon,
  corpus_size,source,ts_ms}], refusal_rows[{claim_id,claim_score,tau,decision,confidence,ts_ms}], abstain_count,
  answer_count) + get_calibration_corpus_state (UNRESOLVED) | read-only | WIRED (D-16) | commands_d16.rs:74,191
  PLANNED: I-23 scorer-health badge (Section 3).
ObservableEffectPanel   | drawer | MonitorWindow | get_observable_effects_recent (RecentObservableEffects: rows[{effect_id,
  trigger_kind,observation_kind,sandbox_tier,outcome,confound_flags,elapsed_us,ts_ms}], counts_by_outcome{},
  counts_by_sandbox_tier{}, total, dispatched_rows[], goodhart_flags[], sandbox_observations[], production_proposals[]) |
  read-only | WIRED (D-18) | commands_d18.rs:139
EvalCorpusPanel         | panel | MonitorWindow | get_eval_corpus_status (EvalCorpusStatus: known, families[{family,
  shard_created_count,rejected_count}], ngram/cosine/schema_violation/test_fails_reference/test_useless/
  generator_rotation_violation/perf_delta counts, ngram_n, cosine_threshold, grand_total, last_ts_ms) | read-only |
  WIRED (E-54) | commands_e54.rs:128
SycophancyMatchedPairPanel | panel | MonitorWindow | get_sycophancy_matched_pair_results (SycophancyMatchedPairResults:
  known, probes[{task_id,family,phase,gap_pp?,sandbagging_anomaly,ts_ms}], framing_gap_count,
  sandbagging_anomaly_count, grand_total) | read-only | WIRED (E-54) | commands_e54.rs:218
AdversaryPanel          | drawer | MonitorWindow | get_adversarial_runs_recent (RecentAdversarialRuns: rows[{counter_id,
  claim_id,severity,model_family,original_verdict,post_counter_verdict,ts_ms}], total, verdict_inversions) +
  get_adversary_config ({adversary_model_family,max_counter_attempts,severity_threshold}) | read-only | WIRED (D-13) | commands_d13.rs:68,129
CanaryPanel             | drawer | MonitorWindow | get_canary_recent (CanaryRunRow[]: report_id, ts_ms,
  overall_transfer_score, decision_kind, dominant_fail_family) + get_canary_per_family_breakdown (by_report{report_id:
  [{family,n_evaluated,pass_count,fail_count,transfer_score,baseline_score}]}) | read-only | WIRED (D-7) | commands_d7.rs:63,100
  PLANNED: I-24 per-branch verdict chips (Section 3).
HeldOutPanel            | drawer | MonitorWindow | get_held_out_recent (HeldOutTaskRow[]: task_id, source,
  content_blake3, ngram_count, ingested_ts_ms, eligibility_window_ms) + get_held_out_stats ({total_added,total_used,
  by_source{},utilization_pct}) | read-only | WIRED (D-6) | commands_d6.rs:61,98
NoveltyBreakdown / NoveltyPanel | panel / drawer | MonitorWindow | get_novelty_breakdown ({total,search,distant_search,
  discovery_candidate,*_pct,dominant_class?,limit_applied}) + get_novelty_recent (RecentNovelty: rows[{template_id,
  class,dkg_match_count,near_duplicate_threshold,ts_ms,candidate_status?}], distribution{total,search,distant_search,
  discovery_candidate}) | read-only | WIRED (D-15) | commands_d15.rs:236,72
ModeRoutingPanel        | drawer | MonitorWindow | get_recent_claim_routings (RecentClaimRoutings: rows[{claim_id,
  claim_type,mode,ts_ms,ctx_summary_blake3}], counts_by_mode{}, total) | read-only | WIRED (D-10) | commands_d10.rs:57
DiscoveryQueueDrawer / ModeRoutingDashboardView | drawer / view | MonitorWindow | get_mode_routing_refusal_combined
  (ModeRoutingRefusalView: mode_breakdown{5 modes}, refusal_rate_current, refusal_health_band, discovery_queue[{
  template_id,class,dkg_match_count,near_duplicate_threshold,ts_ms,days_waiting}], recent_classifications[],
  dominant_mode?, limit_applied) | has-actions: ratify_discovery, reject_discovery | WIRED (D-27) | commands_d27.rs:171,363,400
ArchiveVerifyGauge      | gauge | MonitorWindow | get_archive_verify_status ({verified, total_entries, error_count,
  last_verified_ts_ms?, errors[]}) | read-only | WIRED (D-26) | commands_d26.rs:459
MutationArchivePanel    | drawer | MonitorWindow | get_archive_recent (ArchiveRecentRow[]: template[ReasoningTemplateDto],
  self_hash, parent_hash?, promoted_at_ms?, retired_at_ms?, retirement_reason?) | read-only | WIRED (D-1) | commands_d1.rs:52
BaiArmsPanel            | drawer | MonitorWindow | get_bai_arms (BaiArmRow[]: template_id, alpha, beta, n_pulls,
  was_promoted, mean_estimate) | read-only | WIRED (D-5) | commands_d5.rs:51
ArchiveExplorerView / LineageGraphView | view / view | (archive explorer) | get_template_archive_combined
  (CombinedArchiveView: templates[{id,domain,pattern_kind,archive_status,content_hash_hex,parent_hash_hex?,
  created_at_ms,body_len}], recent_tripwires[], bai_arms[]) + get_archive_lineage (ArchiveLineage: entries[LineageNode],
  root_id?, depth) | read-only | WIRED (D-26) | commands_d26.rs:213,200
ARCBenchPanel           | drawer | MonitorWindow | get_arc_bench_recent (RecentARCBenchRuns: rows[{total,
  honestly_evaluated_count,gating_verdict,fixture_blake3,dominant_mode,dominant_novelty_class,ts_ms}], total,
  last_verdict, last_dominant_mode, last_dominant_novelty_class) | read-only | BENCH-PENDING (D-20) | commands_d20.rs:62
PrmBenchPanel           | drawer | MonitorWindow | get_prmbench_recent (PrmBenchRow[]: seq, ts, total, accuracy,
  precision, recall, gating_verdict, fixture_blake3, category_count) | read-only | BENCH-PENDING (D-19) | commands_d19.rs:48
ModeRoutingBenchPanel   | drawer | MonitorWindow | get_mode_routing_bench_recent (ModeRoutingBenchRow[]: seq, ts, total,
  accuracy, f1_macro, gating_verdict, fixture_blake3, fixture_row_count) | read-only | BENCH-PENDING (D-21) | commands_d21.rs:45
SycophancyBenchPanel    | drawer | MonitorWindow | get_sycophancy_bench_recent (SycophancyBenchRow[]: seq, ts, tp, fp,
  tn, fn_, inconclusive, detection_rate, fp_rate, verdict, total_fixtures, fixture_blake3, runner_name) | read-only |
  BENCH-PENDING (D-23) | commands_d23.rs:51
ConformalValidationPanel| drawer | MonitorWindow | get_conformal_validation_recent (ConformalValidationRow[]: seq, ts,
  tau, epsilon, empirical_refusal_rate, gating_verdict, fixture_blake3, total) | read-only | BENCH-PENDING (D-22) | commands_d22.rs:44
SafeDeltaPanel          | drawer | MonitorWindow | get_safety_delta_latest (SafetyDeltaLatest|null: suite, model_id,
  safety_delta, ci_low, ci_high, n_tasks, seed, corpus_blake3, gate_recall, runpod_pending, ts) | read-only |
  BENCH-PENDING (G-6 SAFEDELTA, the release-gating number) | commands_g6.rs:74
LoraShipGatePanel       | panel | MonitorWindow | get_lora_ship_gate_status ({gate_status[wire_dark|<kind>],
  gate_enabled, ship_gate_token, confirm_user_understanding, rows[]}) | read-only | BENCH-PENDING (mounted; data empty) | commands.rs:11527
ModelDriftPanel         | drawer | MonitorWindow | get_model_drift_status (ModelDriftStatus|null: identity_key, kind,
  signals[], per_family_delta[(family,delta)], affected_template_ids[], confirmed_by_fingerprint, baseline_captured_at,
  ts) | read-only | WIRED (G-10) | commands_g10.rs:72
AttestationDrawer       | drawer | MonitorWindow | get_attestations (AttestationRow[]: seq, ts_ms, session_id,
  subject_blake3, run_artifact_root, verified_levels, all_passed, signing_model) | read-only | WIRED (G-8) | commands_g8.rs:54
TemplateRankingDrawer   | drawer | MonitorWindow | get_template_ranking (rows[{seq,ts,concept_class,plan_skeleton_hash,
  utility,first_try_ratio,sample_n,chosen_pipeline_id}]) | read-only | WIRED (F-7) | commands_f7.rs:89

--- 2e. MEMORY ---
MemoryFactsDrawer       | drawer | MonitorWindow | get_memory_facts / get_memory_facts_with_history (MemoryFactDto[]:
  fact_id, user_id, project_id?, scope, fact_type, key, value[json], confidence, trust_score, trust_threshold,
  ingested_at_ms, valid_from_ms, valid_to_ms?, decay_rate_per_day, last_observed_at_ms, evidence_pointers[],
  superseded_by?) + get_semantic_contradictions (viewer_attached, pairs[{fact_id_a,fact_id_b,conflict_kind,seq,ts_ms}]) |
  has-actions: memory_forget, memory_pin | WIRED (E-5 user memory) | commands_e5.rs:138,456,262,350,403
  PLANNED: I-13 guard badges (Section 3).
MemoryNotesPanel        | drawer | MonitorWindow | get_memory_notes (MemoryNotesResponse: store_path,
  store_just_created, notes[{note_id,scope[session|project|global],tags[],body,links[],link_count,created_at_ms,
  last_accessed_ms,trust_score}]) + get_chat_memory_state ({core_block_text,core_block_bytes,last_reflection_ms?,
  sleep_compute_active,recent_notes_count}) | read-only | WIRED (E-1) | commands_e1.rs:174,225

--- 2f. CODE-INTELLIGENCE / CANVAS ---
ArtifactsPanel          | panel | MonitorWindow | list_artifacts ({artifact_names[]}) + search/index/remove_artifacts |
  has-actions: search/index/remove | WIRED (B-42 cosmos) | commands.rs:9232,9264,9334,9366
FileExplorer / GraphSidebarPanel / MermaidExportModal | tree / panel / modal | (file UI) | read_dir (FsEntry[]:
  name,path,kind,size?) + read_file -> String | has-actions: open_in_editor, mount_root | WIRED (H-11) | commands_h11.rs:159,223,242,282
ArchMode (canvas)       | canvas screen | CanvasWindow | export_graph_mermaid -> {mermaid} | read-only | WIRED | commands.rs:2753
DesignPageNode (canvas) | canvas node | CanvasWindow | resolve_oid -> {oid,file,range[4],scope} | has-actions:
  apply_recast_patch (recast) | WIRED (H-14) | commands_h14.rs:109
entrypoints_map (canvas)| ts module | CanvasWindow | list_entry_points -> {entry_points[BackendEntryPoint],summary} |
  read-only | WIRED | commands.rs:9396

--- 2g. REVIEW / PR / MEDIA / TRANSCRIPT ---
MediaValidationPanel    | panel | MonitorWindow | browser_e2e_status (BrowserE2eStatusDto: runs_started,
  assertions_passed/failed, snapshots_captured, last_journey_name_hash?, last_ts_ms?) + get_media_validations
  (MediaValidationEntry[]: step_id, request_id, media_type, file_path, tier1?, tier2?, render_frames[],
  feedback_iterations[], final_verdict?, updated_at, axtree?, sycophancy?, swap_order_disagreement?,
  convergence_history[]) | has-actions: submit_media_validation_for_review | events media_ssb | WIRED (E-14/E-16) |
  commands_e14.rs:68 + media_validation.rs:419
TranscriptDrawer        | drawer | MonitorWindow | ssb_transcript_paged (TranscriptPage: rows[{seq,ts_ms,action_type,
  agent_id,role,body,redacted}], total_count?, first_seq?, has_older, has_newer) | has-actions: page nav | WIRED (E-29) |
  commands_e29.rs:257   PLANNED: I-14 per-transform receipt rows (Section 3).


================================================================================
SECTION 3 — PLANNED ADDITIONS TO EXISTING SURFACES (design slots)
================================================================================
addition -> target surface -> data -> plan id. Sources: CONTRACT-SHEET Section 4 (I-plan
frontend_pair map) + J/K/H frontend_pair grep. COARSE = plan wording transcribed, fields not invented.

--- I-PLAN (rows/badges/chips on EXISTING surfaces — NO new top-level windows) ---
I-1  supervisor verdict ROWS              -> verdict drawer / OrchestrationDrawer family | COARSE | PLANNED(I-1)
I-2  branch workspace BADGES              -> SessionsDrawer / branch board | COARSE | PLANNED(I-2)
I-4  progress sparkline per branch        -> SessionsDrawer / branch board | COARSE | PLANNED(I-4)
I-7  escalation cascade ROWS              -> ModeRoutingPanel / CostPanel family | COARSE | PLANNED(I-7)
I-8  abort/dead-end STRIP                 -> ReasoningDrawer (NOT YET PRESENT — add reasoning drawer) | COARSE | PLANNED(I-8)
I-9  error-amplification alpha-trend TILE -> CockpitWindow | COARSE | PLANNED(I-9)
I-10 prefix-invariance TTL/economics FIELDS -> CostPanel (H-9 cost tile) | COARSE | PLANNED(I-10)
I-11 belief-state inspector ROW           -> SessionsDrawer | COARSE | PLANNED(I-11)
I-12 playbook curator panel ROWS          -> RulesDrawer (closest: PinnedRulesPanel — see Section 5 unresolved) | COARSE | PLANNED(I-12)
I-13 memory-guard BADGES                  -> MemoryFactsDrawer | COARSE | PLANNED(I-13)
I-14 per-transform receipt ROWS           -> TranscriptDrawer | COARSE | PLANNED(I-14)
I-15 semantic-cache-source BADGE          -> ResearchDrawer / command cards | COARSE | PLANNED(I-15)
I-20 per-hunk mutation CHIP               -> CockpitWindow / MutationArchivePanel | COARSE | PLANNED(I-20)
I-21 sprint-contract panel/ROWS           -> OrchestrationDrawer family | COARSE | PLANNED(I-21)
I-22 reward surface                       -> OrchestrationDrawer / flywheel panel | COARSE | PLANNED(I-22)
I-23 scorer-health BADGE                  -> ConformalCalibrationPanel | COARSE | PLANNED(I-23)
I-24 per-branch canary verdict CHIPS      -> CockpitWindow / CanaryPanel | COARSE | PLANNED(I-24)
I-25 route-explanation ROW                -> CostPanel (cost cockpit) | COARSE | PLANNED(I-25)
I-26 thinking-budget BAR                  -> CostPanel / command cards | COARSE | PLANNED(I-26)
I-27 budgeted-convergence curve TILE      -> NEW cockpit tile | COARSE | PLANNED(I-27)
I-28 dag credit ROWS/TILE                 -> OrchestrationDrawer (already shows credit) | COARSE | PLANNED(I-28)
I-29 diffusion draft-lane BADGE           -> command cards | COARSE | PLANNED(I-29)
I-30 verifier-ranked SFT dataset CARD     -> NEW training/dataset panel | COARSE | PLANNED(I-30)
I-31 grpo flywheel status PANEL           -> NEW panel (admits/blocks/KL trend; relates to ProposalsPanel) | COARSE | PLANNED(I-31)
I-33 governance metrics TILE              -> NEW tile (handler get_governance_metrics SHIPPED: {cdl,diu,fsr,row_count}) | PLANNED(I-33) | commands_i33.rs:56
I-34 grammar-pin status                   -> GrammarCompilePanel | COARSE | PLANNED(I-34)
I-35 sequence-verdict ROWS                -> ToolGateInspector | COARSE | PLANNED(I-35)
I-36 per-role compiled-variant BADGE      -> SettingsWindow | COARSE | PLANNED(I-36)
I-37 per-requirement EARS CHIPS           -> CockpitWindow (G-5) | COARSE | PLANNED(I-37)
I-38 QDELTA TILE                          -> beside SafeDeltaPanel (QdeltaPanel SHIPPED) | PLANNED(I-38) | commands_i38.rs:81
I-39 plan-state-from-repo surface         -> PlanWindow | COARSE | PLANNED(I-39)
I-40 wrapped-run ROWS                     -> RraViewerPanel | COARSE | PLANNED(I-40)
I-41 spec-time critique-lens advisory LIST-> IdeaFramingWindow / new ReasoningDrawer | COARSE | PLANNED(I-41)
I-42 active critique-profile indicator    -> status bar mode cluster (no new component) | COARSE | PLANNED(I-42)
I-44 RetentionPurged ROWS                 -> RraViewerPanel | COARSE | PLANNED(I-44)
I-45 objective-hash chain BADGE           -> RraViewerPanel | COARSE | PLANNED(I-45)
I-46 bash-interceptor block reasons       -> ToolGateInspector / VibeGateDrawer | COARSE | PLANNED(I-46)
I-49 bedrock/azure runtime backend ROWS   -> SettingsWindow runtime (list_runtimes) | COARSE | PLANNED(I-49)
I-50 subscription-auth-broker ROWS/BADGE  -> SettingsWindow auth cluster | COARSE | PLANNED(I-50)
I-52 checkpoint/rewind ROWS               -> SessionsDrawer / SessionArchiveDrawer | COARSE | PLANNED(I-52)
I-53 recipe-runner preview/apply ROWS     -> tool-call drawer | COARSE | PLANNED(I-53)

--- J-PLAN ---
J-1  perception ROWS                      -> existing drawer (decide at edit, W-4); +PerceptionScaffoldEvent | COARSE | PLANNED(J-1)
J-6  control-plane session ROWS           -> existing drawer (or W-6 absence); +HarnessInterventionEvent | COARSE | PLANNED(J-6 family)
J-9  HardwareModelCatalogPanel            -> NEW panel (deferred WITH Tier-2 CLI); +HardwareModelCatalogEvent | COARSE | DEFERRED(J-9)
J-10 insight/lineage COLUMN               -> plan-or-branch drawer; +InsightPropagated | COARSE | PLANNED(J-10)
J-13 local-determinism status LINE        -> RuntimePanel; +LocalDeterminismEvent | COARSE | PLANNED(J-13)
J-14 repro-pack rows                      -> existing RRA/review surface; +ReproPackEvent | COARSE | PLANNED(J-14)
J-16 blast-radius PANEL                   -> audit-timeline surface; +BlastRadiusEvent (KEYSTONE) | COARSE | PLANNED(J-16)
J-17 capability-fragmentation FINDING     -> compliance panel; +CompositionalPolicyEvent | COARSE | PLANNED(J-17)
J-18 retrieval channel BREAKDOWN          -> KnowledgeDrawer; +RetrievalAugmentEvent | COARSE | PLANNED(J-18)
J-19 HF-vs-vLLM TABLE                     -> RuntimePanel/CloudPanel; reuse RuntimeBackendSelected | COARSE | PLANNED(J-19)
J-20 security-invariant gate rows         -> ProposalsPanel/MutationArchivePanel (ELAI_DARWINIAN_LOOP_ENABLE) | COARSE | PLANNED(J-20)
J-21 memory-provenance                    -> memory-hub panel; +MemoryValidationEvent | COARSE | PLANNED(J-21)
J-23 review findings                      -> vibesec/audit panel; +ReviewGateCatalogEvent | COARSE | PLANNED(J-23)
J-24 steering-effect + token-budget       -> runtime/context panel; +BehaviorSteeringEvent | COARSE | PLANNED(J-24)
J-30 BYOK config + ROWS                   -> RuntimePanel extension; +ProviderByokEvent (never carries key material) | COARSE | PLANNED(J-30)
J-33 parsed-artifact BADGES + parse-failure -> KnowledgeDrawer; reuse E-36 / +DocumentParsedEvent | COARSE | PLANNED(J-33)
J-34 curve + knee + active-threshold TILE -> ContextStatePanel/cockpit family; +ThresholdApplied/PlacementApplied/CalibrationInvalidated | COARSE | PLANNED(J-34)
(J-28/J-29/J-31/J-32 are NEW windows -> Section 1.)

--- K-PLAN (almost no new dashboard — mostly CLI/evidence; these are the few UI touches) ---
K-2  evidence-pack-as-PR-comment surface  -> github state surface (github.svelte.ts) (decide at edit, W-4) | COARSE | PLANNED(K-2)
K-4  gate-tax friction TILE               -> cost/audit surface; +GateTaxMeasured (or extend I-38) | COARSE | PLANNED(K-4)
K-5  gate-intensity tier-decision ROWS     -> audit-timeline; +GateIntensitySelected (or reuse F-1) | COARSE | PLANNED(K-5)
K-3  RoutingEconomics HF-vs-hybrid TABLE   -> runtime/cost panel; +RoutingEconomicsMeasured | COARSE | PLANNED(K-3)
K-7  model-refresh-decision TILE           -> (refresh-decision tile); +ModelRefreshCadence | COARSE | PLANNED(K-7)
(K-1/K-6/K-8/K-9 and most of K = N/A dashboard -> Section 6.)

--- H-PLAN (deferred/planned UI not yet on a shipped surface) ---
H-35 rewind controls + session_rewound    -> CommandWindow + BranchTree + TranscriptDrawer (commands_h35.rs SHIPPED) | PLANNED(H-35)
H-36 notification surface                  -> NotificationManager.svelte (commands_h36.rs get_recent_notifications SHIPPED) | PLANNED(H-36)
H-37 plan-approval-gate decisions          -> plan-approval surface (commands_h37.rs get_plan_approval_decisions SHIPPED) | PLANNED(H-37)
H-38 stop-cost-confirm                     -> stop-confirmation surface (stop_confirmation_event) | COARSE | PLANNED(H-38)
H-50 per-regime context gauge + "unloaded N (reloadable)" -> ContextStatePanel family | COARSE | PLANNED(H-50)
H-51 resume-offer (tiered handoff load)    -> CommandWindow (commands_h51.rs resume_offer/tiered_load_confirm SHIPPED) | PLANNED(H-51)
H-Track-2 canvas rebuild pieces            -> Canvas.svelte / WindowHost / DesignPageNode / comparison board -> see Section 4 + Section 1 CanvasWindow

--- H-PLAN TRADING-TRUST GATE FAMILY (H-18..H-29; finance/trading verdict drawers, PLANNED, gate-driven) ---
These each ship an SSB gate + a frontend_pair drawer; all COARSE (plan names the drawer, not fields). Likely a NEW
"trading-trust" panel cluster in MonitorWindow (or a dedicated trading window) if the trading vertical is surfaced.
H-17 trial-ledger N                        -> H-18/H-29 cockpit; +TrialRegistered | PLANNED(H-17)
H-18 {DSR,PBO,N} deflation-gate DRAWER      -> {DSR,PBO,N} drawer; +DeflationGateEvaluated | PLANNED(H-18)
H-19 holdout-budget GAUGE                   -> budget gauge; +HoldoutBudgetEvent | PLANNED(H-19)
H-20 leakage tainting-fit-site DRAWER       -> tainting fit-site drawer; +LeakageBlocked | PLANNED(H-20)
H-21 lookahead first-changing-column        -> leakage drawer; +LookaheadDifferential | PLANNED(H-21)
H-22 trading-null null-band                 -> null-band surface; +TradingNullTest | PLANNED(H-22)
H-23 rerun-equality digest BADGE            -> RRA viewer digest badge; +RerunHashEquality | PLANNED(H-23)
H-24 sim-real parity DRAWER                 -> parity drawer; +SimRealParity | PLANNED(H-24)
H-25 result-manifest reviewer cockpit       -> reviewer cockpit (composes G-5); +ResultManifestVerified | PLANNED(H-25)
H-26 differential-divergence DRAWER         -> divergence drawer; +DifferentialDivergence | PLANNED(H-26)
H-27 gate-self-validation meta cockpit      -> meta cockpit; +GateSelfValidation | PLANNED(H-27)
H-28 reward-tampering self-mod cockpit      -> self-mod cockpit; +RewardTampering | PLANNED(H-28)
H-29 drift-interlock deployment cockpit     -> deployment cockpit; +DriftInterlock | PLANNED(H-29)


================================================================================
SECTION 4 — WIRED BACKEND, NO PANEL YET (candidate NEW panels — GAP-ANALYSIS bucket B)
================================================================================
Features with LIVE backend data and NO surface. Each is a candidate NEW panel; confirm the command
exists before designing (GAP-ANALYSIS note).
5 coding-metrics panels      | E-49 STABLE-Pass@1, $/CTS, the 7-tuple — NO panel | NO-PANEL-YET | feature-map:1190
BackendCostPanel             | C-44 BackendCostAccrued not surfaced — NO panel | NO-PANEL-YET | feature-map:970
VisionRAGDrawer              | E-36 vision-RAG — NO panel | NO-PANEL-YET | feature-map:119
branch-UI banner renderer    | C-23 session-branch frontend banner — NO renderer | NO-PANEL-YET | feature-map:274
ExternalBenchPanel           | E-9 external bench — deferred, no panel | DEFERRED / NO-PANEL-YET | feature-map:1187
HardwareModelCatalogPanel    | J-9 hardware/model catalog — deferred | DEFERRED | J-9
get_governance_metrics tile  | I-33 handler SHIPPED ({cdl,diu,fsr,row_count}) but no tile mounted | NO-PANEL-YET | commands_i33.rs:56
get_kor_orthogonality_status | E-54 handler SHIPPED (known, pairs[{family_a,family_b,passed,delta_pp?,ts_ms}],
  violated/passed_count, grand_total) but no Section-1 panel | NO-PANEL-YET | commands_e54.rs:294
get_compaction_ladder_state / recent_compaction_events | E-25 handlers exist (commands_e25.rs:127,151) but no
  Section-1 panel | NO-PANEL-YET | commands_e25.rs:127,151
write_time_slop_status       | F-9 handler exists (commands_f9.rs:143) but no Section-1 panel | NO-PANEL-YET | commands_f9.rs:143
get_skill_library            | F-6 handler exists (commands_f6.rs:119) but no Section-1 panel | NO-PANEL-YET | commands_f6.rs:119
list_application_skills / recent_skill_routing_events | C-41 handlers exist (commands_c41.rs:114,154) but no
  Section-1 panel | NO-PANEL-YET | commands_c41.rs:114,154


================================================================================
SECTION 5 — EVENT / STREAM CHANNELS the dashboard subscribes
================================================================================
The 8 Tauri listen() channels any redesign MUST re-subscribe (CONTRACT-SHEET 5e):
1. symbol_graph_updated   -> GraphWindow / SymbolGraphWindow refetch
2. media_ssb              -> media_validation store (MediaValidationPanel)
3. reindex-progress       -> reindex store (IndexingProgressPanel)
4. research-progress      -> drawers store (ResearchDrawer feed)
5. research-done          -> drawers store (ResearchDrawer feed)
6. remote-sidecar-progress-> RemoteSidecarBootOverlay
7. reflect-progress       -> chat reflect surface
8. mini-critic-verdict    -> critic strip
PLUS: chat token streaming is command-driven (start_chat_stream emits token events drained by
  lib/streaming/frame_pacer.ts) — NOT a named listen channel; the redesign must keep the frame_pacer path.
PLANNED stream additions: LiveUsageGauge (J-29) mid-stream meter likely needs a usage-tick channel;
  front-door (J-32) cards consume IntentRouted; verify channel shape at build.


================================================================================
SECTION 6 — EXCLUDED (frontend_pair = N/A — explicitly NO dashboard surface)
================================================================================
Considered and intentionally surfaceless (so the designer knows they were not missed). All from the
J/K frontend_pair grep with "N/A" / "W-6 justified" / "harness tooling".
J-3  per-session edit-backup undo tool     | N/A — harness tooling (.claude/tools/undo.py, stdlib) | J-SUMMARY:113
J-4  handoff backup tool                    | N/A — harness tooling | J-SUMMARY:133
J-5  liteGAME propagation                   | N/A — cross-root harness | J-SUMMARY:152
J-7  incremental-eval skill                 | N/A — new .claude/skills/incremental-eval | J-SUMMARY:171
J-8(skill-import) skill-imports.log work     | N/A — harness log + policy amendment | J-SUMMARY:196
J-11 cross-plan amendment blocks            | N/A — doc amendments, no .elai_cc change, no SSB | J-SUMMARY:295
J-12(boundary) context_boundary.rs          | N/A — existing H-51/H-9 panels cover it | J-SUMMARY:577
J-15 amendment blocks                       | N/A — doc amendments, no SSB | J-SUMMARY:373
J-22(evaluator-abi) evaluator_abi module     | N/A — reuse H-43 FitnessEvaluated + E-54 rows | J-SUMMARY:597
J-25 annotated-scratch harness tool          | N/A — harness, settings user-reviewed | J-SUMMARY:620
J-27(payload-guard) SSB payload guard         | N/A (W-6) — test-layer derive + snapshot, guards existing 542 payloads | J-SUMMARY:509
K-1  positioning dossier + claims-policy lint | N/A — docs/strategy + lint, no SSB | K-SUMMARY:41
K-6  compliance export CLI arm               | N/A (W-6) — CLI export (or a tile, decide at edit) | K-SUMMARY:143
K-8  positioning brief                       | N/A — docs/strategy brief, no SSB | K-SUMMARY:157
K-9  feature kill-list                       | N/A — workflow/reasoning decision artifact | K-SUMMARY:177
NOTE: B/C/D/E/F/G summary indexes do NOT use the frontend_pair field (convention introduced in the
  H/I/J/K era). Their dashboard surfaces are fully captured as SHIPPED panels in Section 2 or as
  wired-no-panel gaps in Section 4; none are "excluded" — they predate the field.


================================================================================
SECTION 7 — SUMMARY COUNTS
================================================================================
TOP-LEVEL WINDOWS: 17 existing + 4 planned new (UsageWindow, LiveUsageGauge, canvas-system-mode, front-door) = 21.

DASHBOARD-TOUCHING FEATURES (de-duplicated, by status):
- SHIPPED panels/drawers/views (Section 2, the wired surface set): ~80 distinct components consuming ~190 commands.
- WIRED (live data, renders today):                                  ~62 surfaces
- DARK(flag) (empty unless ELAI_*_ENABLE set):                       4  (CriticDecisionPanel, RoadmapPanel,
                                                                        ContextStatePanel*, TrustTelemetryPanel*
                                                                        — *gated by ELAI_DEVELOPER_DISCOURSE_ENABLE)
- BENCH-PENDING (handler exists, empty until a run):                 7  (SafeDeltaPanel, ARCBench, ModeRoutingBench,
                                                                        PrmBench, SycophancyBench, ConformalValidation,
                                                                        LoraShipGate; QdeltaPanel also bench-pending)
- NO-PANEL-YET (backend wired, no surface — Section 4):              ~12 (5 coding-metrics + BackendCost + VisionRAG +
                                                                        branch-banner + governance-tile + KOR + compaction
                                                                        + write-time-slop + skill-library + skill-routing)
- PLANNED (decorates existing or new; Sections 1+3):                 I-plan ~41 + J-plan ~16 + K-plan ~5 +
                                                                        H-plan deferred ~6 + H trading-trust ~13 = ~81 planned additions
- DEFERRED:                                                          3  (J-9 HardwareModelCatalogPanel, E-9 ExternalBenchPanel,
                                                                        H-Track-2 canvas pieces partially-shipped/demoted)

NEW SURFACES TO DESIGN (net-new components the redesign should plan room for):
- 4 new windows: UsageWindow, LiveUsageGauge, canvas "system" mode, front-door cards.
- ~12 new panels from Section 4 (wired-no-panel): coding-metrics(x5), BackendCostPanel, VisionRAGDrawer,
  branch-banner, governance tile (I-33), KOR panel, compaction-ladder panel, skill-library/routing panel.
- New drawers implied by I-plan: ReasoningDrawer (I-8/I-41), training/dataset panel (I-30), grpo flywheel panel (I-31),
  budgeted-convergence cockpit tile (I-27).
- H trading-trust gate cluster: up to ~13 finance/trading verdict drawers/cockpits (H-18..H-29) IF the trading
  vertical is surfaced — likely a dedicated trading window or panel cluster.

EVENT CHANNELS to re-subscribe: 8 (Section 5) + frame_pacer token stream + planned usage-tick/IntentRouted.

KEY DESIGN IMPLICATIONS (carried from GAP-ANALYSIS cross-cutting findings):
- ~75% of feature panels invoke INLINE (no mock fallback) -> show empty/error offline; ~17 STORE-path panels
  silently render mock-data.ts fixtures -> a browser preview LOOKS live but is not. Make mock-vs-live VISIBLE.
- Most handlers return CmdResult<Value> (untyped) — the TS type is the front-end's ASSERTION; ssb_viewer.rs is the
  canonical wire-shape source for ~70% of panels (shapes in HANDLER-SHAPES).
- The single biggest design surfaces the redesign must plan around: MonitorWindow (hosts ~60 leaf panels),
  CockpitWindow + CostPanel + SessionsDrawer + RraViewerPanel (the heaviest planned-addition targets), the canvas,
  and the new UsageWindow.
- Design ONE uniform empty-state vocabulary: "feature off (set ELAI_*_ENABLE=1)" (DARK), "no run yet" (BENCH-PENDING),
  "no data" (WIRED-empty), "unavailable" (handler/diff error). Do not fake data.

UNRESOLVED (do NOT invent shapes — resolve before wiring; from CONTRACT-SHEET 5d + HANDLER-SHAPES):
- get_pinned_rules (PinnedRulesPanel) — NO #[tauri::command] of that name exists. PinnedRulesPanel will error/empty.
- get_isolation_metric_status (referenced in api/stub.ts) — NO Rust handler found; likely stub/mock-only.
- UNRESOLVED-SHAPE handlers (body not read): get_mcp_security_status, get_calibration_corpus_state,
  get_refusal_rate_current, + the settings/cloud/oauth/sidecar/runtime/chat/intent-pair/plan-detail/sessions/projects/
  grc-sweep/csq/reflect setters & getters.
- UNRESOLVED-SUBOBJECT (1-hop nested types): VizData, DecompositionTreeSnapshot, ReasoningTemplateDto, SymbolMatch/
  RankedSymbol, PrSummary/PrStatus, BackendEntryPoint, research_node_stages row, MediaValidationEntry sub-snapshots.
