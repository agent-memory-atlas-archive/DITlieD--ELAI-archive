# C-2: SSB Emission — Orchestration + Tree Navigator + Cosmos + Ambiguity + Anti-Spiral

Objective: close the SSB emission gap across five producer sites — pipeline tree
navigator, cosmos_worker, ambiguity scorer, autonomous gate stagnation detector,
and agent lifecycle — so that BubbleUpStrip.svelte and OrchestrationDrawer.svelte
display real events from a live cloud run.
Estimated effort: 1 session ~6-8h.
Pre-requisites: C-1 ships first; it establishes the SsbActionType variant pattern
and the emit helper used by all sites here.
Parallel-safe with: C-3, C-4, C-9.

## Context

B-7 (MISSING): phase_node_ready, step_node_ready, baby_step_ready, dag_repair_escalated
not in ssb.rs; no emit sites found.

B-13 (PARTIAL): BubbleUpStrip.svelte queries bubble_up_typed rows (commands.rs:2034)
but comment at :66 reads "LIVE: empty until B-7 wires bubble_up_event SSB emit".

B-6 (MISSING): agent_status_changed SsbActionType absent; get_agents (commands.rs:1659)
returns hardcoded IDLE list.

B-38 (PARTIAL): AmbiguityScored / AmbiguityFloorFailed / AmbiguityStreakAdvanced defined
at ssb.rs:914-921 but AmbiguityScorer not called in task_runner.rs; get_ambiguity_state
returns mock:true (commands.rs:6561); OrchestrationDrawer has no ambiguity section.

B-42 (PARTIAL): CosmosRefreshStarted/Updated/Decayed/GroundingUsed defined at ssb.rs:923-934
but cosmos_worker.rs lines 30-33 defer SSB integration; tracing::info! used as temp channel.

phase-4G (MISSING): stagnation_spinning_detected, stagnation_oscillation_detected,
lateral_thinking_activated, goal_drift_warning, watchdog_timeout entirely absent from ssb.rs.

## Reads

- .elai_cc/crates/orchestrator/src/ssb.rs (SsbActionType enum, SsbPayload, emit helper)
- .elai_cc/crates/orchestrator/src/ssb_viewer.rs (viewer query methods)
- .elai_cc/crates/orchestrator/src/task_runner.rs (AmbiguityScorer call site, gate wiring)
- .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs (bubble_up_typed emit at :1607)
- .elai_cc/crates/orchestrator/src/pipeline_builder.rs (dag_repair_round emit at :1100)
- .elai_cc/crates/orchestrator/src/autonomous_gate.rs (EscalationTrigger, GateContext)
- .elai_cc/crates/knowledge/src/cosmos_worker.rs (tick, refresh_one, record_grounding_used)
- .elai_cc/crates/reasoning/src/ambiguity.rs (AmbiguityScorer, score() async method)
- .elai_cc/crates/orchestrator/src/evidence.rs (GroundingSweepGate, TransitionContext)
- .elai_cc/crates/dashboard-v2/src/lib/orbit/BubbleUpStrip.svelte (placeholder comment :66)
- .elai_cc/crates/dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte (ambiguity gap)
- .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs (get_agents :1659, get_ambiguity_state :6561)
- verification/reports/B-plan/B-6, B-7, B-13, B-38, B-42 reports (grounding for all gaps above)
- verification/reports/steps/phase-4G.report.md (4G.9 SSB action types missing list)

## Tasks

1. **ssb.rs — define 4 missing tree-node variants** (B-7 MISSING)
   Add SsbActionType variants: PhaseNodeReady, StepNodeReady, BabyStepReady,
   DagRepairEscalated. Add corresponding payload structs
   (PhaseNodeReadyPayload, StepNodeReadyPayload, BabyStepReadyPayload,
   DagRepairEscalatedPayload) and string serialization arms following the
   existing C-1 pattern.
   File: .elai_cc/crates/orchestrator/src/ssb.rs

2. **ssb.rs — define agent_status_changed variant** (B-6 MISSING)
   Add SsbActionType::AgentStatusChanged + AgentStatusChangedPayload with fields:
   agent_id: String, role: String, old_lifecycle: String, new_lifecycle: String,
   timestamp_ms: u64.
   File: .elai_cc/crates/orchestrator/src/ssb.rs

3. **ssb.rs — define 5 stagnation/watchdog variants** (4G.9 MISSING)
   Add variants: StagnationSpinningDetected, StagnationOscillationDetected,
   LateralThinkingActivated, GoalDriftWarning, WatchdogTimeout.
   Add minimal payload structs (pattern: String, iteration: u32, detail: String).
   File: .elai_cc/crates/orchestrator/src/ssb.rs

4. **pipeline_tree_navigator.rs — emit phase_node_ready / step_node_ready /
   baby_step_ready** (B-7 MISSING)
   At the node-entry points in pipeline_tree_navigator.rs (near existing
   bubble_up_typed emit at :1607), emit PhaseNodeReady / StepNodeReady /
   BabyStepReady rows with node_id, label, depth, parent_id. Use the same
   ssb_tx sender already present.
   File: .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs

5. **pipeline_builder.rs — emit dag_repair_escalated** (B-7 MISSING)
   At the dag-repair exhaustion path in pipeline_builder.rs (near dag_repair_round
   emit at :1100, when T_DAG=10 retry limit is reached), emit DagRepairEscalated
   with attempt_count and last_error fields.
   File: .elai_cc/crates/orchestrator/src/pipeline_builder.rs

6. **cosmos_worker.rs — emit 4 cosmos SSB rows** (B-42 PARTIAL)
   Replace tracing::info!(target="cosmos") placeholder calls in tick(),
   refresh_one(), and record_grounding_used() with real SsbRow inserts:
   CosmosRefreshStarted (on tick entry per artifact), CosmosArtifactUpdated
   (on successful write), CosmosArtifactDecayed (on decay detection),
   CosmosGroundingUsed (in record_grounding_used). Pass ssb_tx via CosmosWorker
   constructor or method argument following the pattern in bsp/engine.rs.
   File: .elai_cc/crates/knowledge/src/cosmos_worker.rs

7. **task_runner.rs — wire AmbiguityScorer call** (B-38 PARTIAL)
   Before GroundingSweepGate fires, call AmbiguityScorer::score() and populate
   TransitionContext.ambiguity_score. Evidence.rs already consumes the field when
   Some (evidence.rs:255-258 backward-compat confirmed). Emit ambiguity_scored /
   ambiguity_floor_failed / ambiguity_streak_advanced SSB rows from the gate
   result per the existing payload structs at ssb.rs:4851-4882.
   File: .elai_cc/crates/orchestrator/src/task_runner.rs

8. **task_runner.rs — emit agent_status_changed on AgentLifecycle transitions**
   (B-6 MISSING)
   At the AgentLifecycle SSB emit sites already present in bsp/engine.rs
   (:242, :264, :353, :509, :580), mirror an AgentStatusChanged row into the
   main ssb_tx so ssb_viewer can query it without touching bsp internals.
   Alternatively wire directly in task_runner.rs agent spawn/complete/crash paths.
   File: .elai_cc/crates/orchestrator/src/task_runner.rs

9. **ssb_viewer.rs — add recent_agent_status_changes() query**
   Add a viewer method that reads AgentStatusChanged rows and returns the latest
   lifecycle per agent_id. Used by get_agents Tauri command in step 10.
   File: .elai_cc/crates/orchestrator/src/ssb_viewer.rs

10. **commands.rs — replace get_agents hardcoded list** (B-6 PARTIAL)
    Replace the 8-role hardcoded IDLE response (commands.rs:1659-1668) with a
    real query to ssb_viewer.recent_agent_status_changes() so live lifecycle
    state propagates to OrchestrationDrawer.svelte.
    File: .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs

11. **commands.rs — replace get_ambiguity_state mock** (B-38 PARTIAL)
    Replace mock:true response (commands.rs:6561) with real SSB reads of
    ambiguity_scored / ambiguity_floor_failed / ambiguity_streak_advanced rows
    from ssb_viewer. Requires task 7 landing first.
    File: .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs

12. **OrchestrationDrawer.svelte — render ambiguity section** (B-38 PARTIAL)
    Add a collapsed AmbiguitySection block inside OrchestrationDrawer.svelte
    that calls get_ambiguity_state and renders overall_score, per-dimension
    breakdown bars, floor-failure flags, and streak progress — matching the
    GatesPanel.svelte layout already proven at :96-144.
    File: .elai_cc/crates/dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte

13. **Contract test: ssb_c2_tree_node_contract.rs**
    Verify PhaseNodeReady, StepNodeReady, BabyStepReady, DagRepairEscalated
    serialize to the expected action_type strings and round-trip through ssb_viewer.
    File: .elai_cc/crates/contracts/tests/ssb_c2_tree_node_contract.rs

14. **Contract test: ssb_c2_cosmos_emit_contract.rs**
    Insert mock CosmosRefreshStarted + CosmosArtifactUpdated rows and verify
    ssb_viewer returns them with correct payloads.
    File: .elai_cc/crates/contracts/tests/ssb_c2_cosmos_emit_contract.rs

15. **Contract test: ssb_c2_agent_status_contract.rs**
    Emit AgentStatusChanged rows for 3 roles, verify recent_agent_status_changes()
    returns latest lifecycle per role and get_agents Tauri command reflects them.
    File: .elai_cc/crates/contracts/tests/ssb_c2_agent_status_contract.rs

## WIRING_CONTRACT

produces_symbols:
  - SsbActionType::{PhaseNodeReady, StepNodeReady, BabyStepReady, DagRepairEscalated}
  - SsbActionType::AgentStatusChanged + AgentStatusChangedPayload
  - SsbActionType::{StagnationSpinningDetected, StagnationOscillationDetected,
    LateralThinkingActivated, GoalDriftWarning, WatchdogTimeout}
  - cosmos SSB emit in CosmosWorker::{tick, refresh_one, record_grounding_used}
  - AmbiguityScorer wired into task_runner pre-gate path
  - ssb_viewer::recent_agent_status_changes()
  - get_agents live SSB response (no hardcoded IDLE)
  - get_ambiguity_state live SSB response (no mock:true)
  - OrchestrationDrawer ambiguity section
  - 3 new contract test files

consumes_symbols:
  - C-1 SsbActionType variant pattern + emit helper (pre-requisite)
  - AmbiguityScorer::score() at .elai_cc/crates/reasoning/src/ambiguity.rs:244
  - TransitionContext.ambiguity_score at .elai_cc/crates/orchestrator/src/evidence.rs:98
  - existing dag_repair_round emit pattern at pipeline_builder.rs:1100
  - existing bubble_up_typed emit at pipeline_tree_navigator.rs:1607
  - ssb_tx sender already threaded into task_runner and bsp/engine

reachable_from:
  - elai /task --mode cloud (all tree-node, cosmos, ambiguity emit sites trigger)
  - BubbleUpStrip.svelte via get_bubble_up_events (bubble_up_typed rows already
    wired; phase/step/baby_step_ready rows visible once defined)
  - OrchestrationDrawer.svelte agents section (live get_agents)
  - OrchestrationDrawer.svelte ambiguity section (new, task 12)
  - CosmosDrawer.svelte refresh flow

stub_ban:
  - get_agents MUST NOT return hardcoded IDLE list after this session
  - get_ambiguity_state MUST NOT return mock:true after this session
  - cosmos SSB rows MUST use SsbRow INSERT, not tracing::info! as channel
  - no .unwrap() in new emit paths; use ? or .ok() with logged error

observable_effect:
  trigger: elai /task --mode cloud "build a three-function Rust CLI with add/sub/mul"
  expect_ssb: >=1 phase_node_ready row, >=3 step_node_ready rows,
              >=1 ambiguity_scored row with overall_score field non-null,
              >=1 cosmos_refresh_started row (if cosmos tick fires in window),
              >=1 agent_status_changed row per spawned agent role
  expect_dashboard: BubbleUpStrip.svelte renders >=1 bubble row (not placeholder),
                    OrchestrationDrawer agents section shows non-IDLE lifecycle
                    for at least one role during execution,
                    OrchestrationDrawer ambiguity section visible with real score

## DETERMINISTIC_CHECK

```bash
# 1. All 12 new SsbActionType variants present in ssb.rs (>= 12 lines)
rg "PhaseNodeReady|StepNodeReady|BabyStepReady|DagRepairEscalated|AgentStatusChanged|\
StagnationSpinningDetected|StagnationOscillationDetected|LateralThinkingActivated|\
GoalDriftWarning|WatchdogTimeout|CosmosRefreshStarted|CosmosArtifactUpdated" \
  .elai_cc/crates/orchestrator/src/ssb.rs | wc -l

# 2. Cosmos emit calls in cosmos_worker (>= 4 matches)
rg "SsbPayload::Cosmos" .elai_cc/crates/knowledge/src/cosmos_worker.rs | wc -l

# 3. AmbiguityScorer called from task_runner (>= 1)
rg "AmbiguityScorer" .elai_cc/crates/orchestrator/src/task_runner.rs | wc -l

# 4. get_agents not hardcoded IDLE (expect 0)
rg "lifecycle.*IDLE" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs | wc -l

# 5. get_ambiguity_state not mock (expect 0)
rg '"mock"' .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs | wc -l

# 6. Tree-node emits in pipeline_tree_navigator (>= 3)
rg "PhaseNodeReady|StepNodeReady|BabyStepReady" \
  .elai_cc/crates/orchestrator/src/pipeline_tree_navigator.rs | wc -l

# 7. dag_repair_escalated in pipeline_builder (>= 1)
rg "DagRepairEscalated" .elai_cc/crates/orchestrator/src/pipeline_builder.rs | wc -l

# 8. Contract tests pass
cargo test -p elai_contracts -- ssb_c2 2>&1 | grep "test result"

# 9. Cargo check clean
cargo check -p elai_orchestrator -p elai_knowledge -p elai_contracts 2>&1 | tail -2
```

## ACCEPTANCE

- [ ] All 12 new SsbActionType variants compile and serialize to correct snake_case strings
- [ ] pipeline_tree_navigator.rs emits PhaseNodeReady, StepNodeReady, BabyStepReady
      at node-entry points; DagRepairEscalated emitted in pipeline_builder.rs at T_DAG limit
- [ ] cosmos_worker.rs emits CosmosRefreshStarted / CosmosArtifactUpdated /
      CosmosArtifactDecayed / CosmosGroundingUsed via SsbRow INSERT (not tracing)
- [ ] AmbiguityScorer.score() called in task_runner pre-gate path; TransitionContext
      carries real AmbiguityScore; ambiguity_scored SSB row written per gate eval
- [ ] get_agents Tauri command returns live lifecycle from ssb_viewer, not hardcoded IDLE
- [ ] get_ambiguity_state Tauri command returns real SSB rows, not mock:true
- [ ] OrchestrationDrawer.svelte renders ambiguity section with overall_score + breakdown
- [ ] 3 contract test files pass: ssb_c2_tree_node_contract, ssb_c2_cosmos_emit_contract,
      ssb_c2_agent_status_contract (total >= 9 test functions across the 3 files)
- [ ] cargo check -p elai_orchestrator -p elai_knowledge -p elai_contracts: 0 errors
- [ ] Cloud-Sonnet smoke test PROVEN:
      Run: elai /task --mode cloud "build a three-function Rust CLI with add/sub/mul"
      Assert after run completes:
      (a) Query SSB: SELECT action_type, payload FROM ssb_rows WHERE action_type IN
          ('phase_node_ready','step_node_ready','baby_step_ready','ambiguity_scored',
           'agent_status_changed','cosmos_refresh_started') — must return >= 5 distinct
          action_type values with non-null payloads
      (b) BubbleUpStrip.svelte in the live dashboard shows >= 1 bubble row (not the
          "no bubble-up events" placeholder)
      (c) OrchestrationDrawer agents section shows at least one role with lifecycle
          != IDLE during or after task execution
      (d) OrchestrationDrawer ambiguity section renders overall_score as a number,
          not a loading spinner or zero-state

## TESTS

Contract tests to add (3 files, >=9 functions total):

crates/contracts/tests/ssb_c2_tree_node_contract.rs
  test_phase_node_ready_serializes_to_action_type_string
  test_step_node_ready_round_trips_through_viewer
  test_baby_step_ready_payload_fields_non_null
  test_dag_repair_escalated_emitted_at_t_dag_limit

crates/contracts/tests/ssb_c2_cosmos_emit_contract.rs
  test_cosmos_refresh_started_insert_and_retrieve
  test_cosmos_artifact_updated_payload_has_artifact_id
  test_cosmos_grounding_used_increments_count

crates/contracts/tests/ssb_c2_agent_status_contract.rs
  test_agent_status_changed_latest_lifecycle_per_role
  test_get_agents_returns_live_not_idle

Each test uses an in-process SSB (SQLite temp-db) following the pattern in
crates/contracts/tests/ssb_4k3_4k4_4k5_contract.rs.

## Out-of-scope

- AmbiguityScorer interrogation-round integration — defer to C-8
- CosmosWorker spawn_loop tick wiring + CosmosStage in QueryPipeline — defer to C-16
- StagnationDetection / ThinkingPersona / GenerationWatchdog / GoalDriftCanary logic —
  defer to C-10; this session adds SSB variant definitions only
- 4G.9 persona agent files — defer to C-10
- BubbleUp cascade arrows in PlanWindow.svelte — defer to C-3
- get_phase_detail / get_baby_step_detail V2 Tauri commands — defer to C-3
- research_node_stage, package_ref_verified, tech_stack_decided SSB types — defer to C-5
- trigger_cosmos_refresh unblocking — defer to C-16
