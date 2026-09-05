# Verification Report: B-7_pipeline_factory_producer_wiring
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-7_pipeline_factory_producer_wiring.md
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary

B-7 is substantially implemented on the producer/emitter side: the SSB action types are defined, SsbDecompositionEmitter is wired in task_runner.rs, dag_repair_round and bubble_up_typed and integration_checkpoint_success are emitted from their respective sources, and get_decomposition_tree in the V2 commands.rs performs a real SSB read. PlanWindow is mounted in V2 via WindowHost.svelte (type='plan'), satisfying the V2 routing model. However, 7 of the 14 named SSB action types in WIRING_CONTRACT are absent from the codebase (no definition found), get_phase_detail and get_baby_step_detail exist only in the archived V1 crate (.bkp_old/), get_executable_pipeline remains a stub returning [], and get_research_budget/get_source_tier_distribution return hardcoded zeroes flagged known=false.

## Claims verified DONE

- PlanWindow.svelte exists (272+ LOC) in V2 — evidence: crates/dashboard-v2/src/lib/windows/PlanWindow.svelte:1
- PlanWindow mounted in V2 via WindowHost.svelte `{:else if type === 'plan'}` — evidence: crates/dashboard-v2/src/lib/layout/WindowHost.svelte:29-30
- SSB action type ConceptDecompositionNode defined — evidence: crates/orchestrator/src/ssb.rs:678
- concept_decomposition_node string mapping present — evidence: crates/orchestrator/src/ssb.rs:1050,1228
- ConceptDecompositionNodePayload struct defined — evidence: crates/orchestrator/src/ssb.rs:1791
- SsbDecompositionEmitter struct defined in research_tree.rs — evidence: crates/orchestrator/src/research_tree.rs:2358
- SsbDecompositionEmitter wired into task_runner.rs (two call sites) — evidence: crates/orchestrator/src/task_runner.rs:4375,4386
- SSB action type DagRepairRound defined — evidence: crates/orchestrator/src/ssb.rs:1227 ("dag_repair_round")
- DagRepairRoundPayload struct defined — evidence: crates/orchestrator/src/ssb.rs:1781
- dag_repair_round emitted from pipeline_builder.rs — evidence: crates/orchestrator/src/pipeline_builder.rs:1100
- SSB action type BubbleUpTyped defined — evidence: crates/orchestrator/src/ssb.rs:669,1225
- BubbleUpTypedPayload struct defined — evidence: crates/orchestrator/src/ssb.rs:1755
- bubble_up_typed emitted from pipeline_tree_navigator.rs — evidence: crates/orchestrator/src/pipeline_tree_navigator.rs:1607
- SSB action type IntegrationCheckpointSuccess defined — evidence: crates/orchestrator/src/ssb.rs:672,1048
- IntegrationCheckpointSuccessPayload struct defined — evidence: crates/orchestrator/src/ssb.rs:1769
- integration_checkpoint_success emitted from pipeline_tree_navigator.rs — evidence: crates/orchestrator/src/pipeline_tree_navigator.rs:1644
- get_decomposition_tree (V2 commands.rs) performs real SSB read via viewer.decomposition_tree() — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:1904-1921
- decomposition_tree() in ssb_viewer.rs reads concept_decomposition_node rows and builds real tree — evidence: crates/orchestrator/src/ssb_viewer.rs:1758-1784
- decomposition_tree() still returns default() when SSB has no rows (expected empty-state) — evidence: crates/orchestrator/src/ssb_viewer.rs:1784
- get_dag_repair_log (V2) queries recent_dag_repair_rounds(100) from SSB — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:2051-2059
- recent_dag_repair_rounds() defined in ssb_viewer.rs — evidence: crates/orchestrator/src/ssb_viewer.rs:3345
- get_bubble_up_events (V2 commands.rs) implemented — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:2034
- ResearchDrawer.svelte exists in V2 and imports ResearchNodeStage — evidence: crates/dashboard-v2/src/lib/drawers/ResearchDrawer.svelte:7
- ResearchNodeStage type defined in types.ts — evidence: crates/dashboard-v2/src/lib/api/types.ts:809
- OrchestrationDrawer.svelte exists in V2 — evidence: crates/dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte:1
- WebQualityDrawer.svelte exists in V2 — evidence: crates/dashboard-v2/src/lib/drawers/WebQualityDrawer.svelte:91
- research_budget_snapshot() and source_tier_distribution() exist in ssb_viewer.rs — evidence: crates/orchestrator/src/ssb_viewer.rs:3055,3073
- integration_checkpoint_success contract tests exist — evidence: crates/orchestrator/tests/integration_checkpoint_success.rs:67,100
- PlanWindow calls get_bubble_up_events — evidence: crates/dashboard-v2/src/lib/windows/PlanWindow.svelte:62
- PlanWindow calls get_decomposition_tree via plan state — evidence: crates/dashboard-v2/src/lib/state/plan.svelte.ts:40

## Claims PARTIAL

- get_decomposition_tree stub eliminated (WIRING_CONTRACT stub_ban)
  - exists: V2 commands.rs does a real SSB read (crates/dashboard-v2/src-tauri/src/commands.rs:1904)
  - gap: ssb_viewer.rs still returns DecompositionTreeSnapshot::default() when no rows found (crates/orchestrator/src/ssb_viewer.rs:1784). This is an empty-state fallback, not a hardcoded default; stub_ban is functionally met for populated runs but the viewer-level early-exit is still ::default().

- get_phase_detail (V2 Tauri command)
  - exists: ssb_viewer.rs:1872 has phase_detail() method with real SQL read; V1 .bkp_old commands.rs:2645 has the Tauri command
  - gap: V2 commands.rs (crates/dashboard-v2/src-tauri/src/commands.rs) has no get_phase_detail command. Only archived V1 exposes it.

- get_baby_step_detail (V2 Tauri command)
  - exists: ssb_viewer.rs:1916 has baby_step_detail() method; V1 .bkp_old commands.rs:2660 has Tauri command
  - gap: baby_step_detail() still returns NotFound for every call ("No BabyStepSpec persistence yet"); V2 commands.rs has no get_baby_step_detail. Both the V2 command wiring and the actual persistence are missing.

- get_research_budget + get_source_tier_distribution (real reads)
  - exists: ssb_viewer.rs methods exist at lines 3055 and 3073
  - gap: both return hardcoded zeroes with known=false; no V2 Tauri commands expose them; no writer-side SSB row type persists the underlying data

- OrchestrationDrawer shows DAG repair log
  - exists: get_dag_repair_log command is wired in V2 (crates/dashboard-v2/src-tauri/src/commands.rs:2051); OrchestrationDrawer.svelte exists
  - gap: OrchestrationDrawer.svelte does not import or call get_dag_repair_log; the repair log section is not rendered there (no evidence in elai.map for that import)

## Claims MISSING

- SSB action type phase_node_ready — searched: "phase_node_ready", "PhaseNodeReady"; no evidence in codebase
- SSB action type step_node_ready — searched: "step_node_ready", "StepNodeReady"; no evidence in codebase
- SSB action type baby_step_ready — searched: "baby_step_ready", "BabyStepReady"; no evidence in codebase
- SSB action type dag_repair_escalated — searched: "dag_repair_escalated", "DagRepairEscalated"; no evidence in codebase
- SSB action type phase_sized_split — searched: "phase_sized_split", "PhaseSizedSplit"; no evidence in codebase
- SSB action type tree_position_changed — searched: "tree_position_changed", "TreePositionChanged"; no evidence in codebase
- SSB action type failure_recorded (as SSB row, not struct) — searched: "failure_recorded", "FailureRecorded" in ssb.rs; no SSB emit of failure_recorded action_type; FailureRecord struct exists in pipeline_tree_navigator.rs but is not wired to SSB
- SSB action type research_node_stage (as SSB row) — searched: "research_node_stage" in ssb.rs; no SSB action type; ResearchNodeStage type exists in types.ts frontend but no emitter found in web_research_orchestrator.rs or ssb.rs
- SSB action type package_ref_verified — searched: "package_ref_verified", "PackageRefVerified"; no evidence
- SSB action type tech_stack_decided — searched: "tech_stack_decided", "TechStackDecided"; no evidence
- get_executable_pipeline real implementation — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:2046 returns Ok(json!([])) stub
- get_research_node_history Tauri command — searched: "get_research_node_history"; no evidence in any commands.rs
- get_package_verifications Tauri command — searched: "get_package_verifications"; no evidence in any commands.rs
- PhaseSizingSplitter SSB emit (phase_sized_split payload) — searched: "PhaseSizingSplitter", "phase_sized_split"; no evidence
- TreeNavigator position cursor SSB emit — searched: "tree_position_changed"; no evidence
- DecompositionAlternative intervention card — searched: "DecompositionAlternative"; no evidence in dashboard-v2
- ResearchCache hit-rate gauge (frontend) — searched: dashboard-v2 for cache_hit gauge; no evidence
- PackageRef.verified badges in PlanWindow — searched: "package_ref_verified" in PlanWindow.svelte; no evidence
- TechStackDecision badges in PlanWindow — searched: "tech_stack_decided" in PlanWindow.svelte; no evidence
- BubbleUp cascade visualization with arrows in PlanWindow — searched: PlanWindow for cascade/arrow render; PlanWindow calls get_bubble_up_events and has bubbleColor() but no arrow cascade visualization

## Claims UNVERIFIABLE

- "PlanWindow.svelte (272 LOC, unregistered)" in V1 — V1 src-frontend path (crates/dashboard/src-frontend/src/windows/PlanWindow.svelte) does not exist on disk (only .bkp_old backup); cannot verify V1 state. V2 path is verified.
- D3 / cytoscape.js DAG visualizer added — plan says "ADD dashboard d3 or cytoscape.js dep (with confirmation)"; cannot verify without package.json read and no elai.map entry found; get_executable_pipeline is a stub so the visualizer is definitively not present.
- "web_research_orchestrator.rs:166-396 emits only 5 coarse tokens" — line numbers may have shifted; claim is about pre-B-7 state; cannot verify the exact line range without full read.

## What must be done (action list)

- [ ] Define SSB action types: phase_node_ready, step_node_ready, baby_step_ready in crates/orchestrator/src/ssb.rs
- [ ] Define SSB action type dag_repair_escalated in crates/orchestrator/src/ssb.rs and emit from pipeline_builder.rs when T_DAG=10 exhausted
- [ ] Define SSB action types: phase_sized_split, tree_position_changed, failure_recorded, research_node_stage, package_ref_verified, tech_stack_decided in crates/orchestrator/src/ssb.rs
- [ ] Wire emitters for all 7 missing SSB action types in their respective producer modules
- [ ] Add get_phase_detail Tauri command to crates/dashboard-v2/src-tauri/src/commands.rs (forwarding to ssb_viewer.phase_detail)
- [ ] Add get_baby_step_detail Tauri command to crates/dashboard-v2/src-tauri/src/commands.rs (and fix ssb_viewer.baby_step_detail stub — currently always NotFound)
- [ ] Implement get_executable_pipeline in crates/dashboard-v2/src-tauri/src/commands.rs (currently returns Ok(json!([])))
- [ ] Add get_research_node_history and get_package_verifications Tauri commands to V2 commands.rs
- [ ] Implement real reads for research_budget_snapshot() and source_tier_distribution() in ssb_viewer.rs (currently zeroes/known=false); expose via V2 Tauri commands
- [ ] Wire get_dag_repair_log into OrchestrationDrawer.svelte to render repair log section
- [ ] Add granular research_node_stage emits to web_research_orchestrator.rs per Formulate/Search/Extract/Reflect/Converge stage
- [ ] Add PhaseSizingSplitter SSB emit in the phase-sizing code
- [ ] Add tree_position_changed emit in pipeline_tree_navigator.rs per navigator step
- [ ] Surface PackageRef.verified badges and TechStackDecision badges in PlanWindow.svelte
- [ ] Implement BubbleUp cascade visualization with arrows (BabyStep->Step->Phase->Concept) in PlanWindow.svelte
- [ ] Implement ResearchCache hit-rate gauge in ResearchDrawer.svelte or dedicated widget
- [ ] Create workflow/reasoning/B-7-decisions.md (required by plan, not found)

## Notes

The V2 rebaseline (noted in plan §V2 REBASELINE NOTE) is in effect. V1 WindowRegistry.ts maps WindowType.PLAN to Workspace (not PlanWindow), consistent with "unregistered" claim; V2 WindowHost.svelte routes type='plan' to PlanWindow directly — the V2 routing satisfies the registration requirement per the rebaseline. The most critical gaps are: 7 of 14 SSB action types entirely absent, get_phase_detail/get_baby_step_detail missing from V2 commands, and get_executable_pipeline is a complete stub. The emitter-side work for concept_decomposition_node, dag_repair_round, bubble_up_typed, and integration_checkpoint_success is complete and tested.
