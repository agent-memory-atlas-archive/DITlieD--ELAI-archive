# B-7: Producer Wiring — Pipeline Factory (4D + 4E)
Objective: visualize the DecompositionTree, ExecutablePipeline DAG, DAG
repair events, web research per-node activity, and IntegrationCheckpoint
successes. Activate PlanWindow.svelte and replace the empty stubs
get_decomposition_tree / get_phase_detail / get_baby_step_detail.
Estimated effort: 2-3 sessions.
Pre-requisites: B-1 (path unification — research tree state may need it).
Parallel-safe with: B-4, B-5, B-6.

## SHIPPED-ON-V2 STATUS (2026-05-22, session B-7-cont-2)

Producer-wire surface is GREEN on V2 (dashboard-v2 + orchestrator). The
following tasks ship in the current changeset:

  Task #1  PlanWindow register      SHIPPED ON V2 — mounted at
                                    crates/dashboard-v2/src/lib/layout/
                                    WindowHost.svelte:7 (import) :64
                                    (render). No WindowRegistry.ts on V2;
                                    WindowHost is the direct mount surface.
  Task #2  concept_decomposition_   SHIPPED ON V2 — ConceptDecompositionNode
           node producer            payload at ssb.rs:4184, emitted via
                                    research_tree.rs::SsbDecompositionEmitter
                                    (impl DecompositionTreeEmitter); wired
                                    on every decompose() child push.
  Task #3  get_decomposition_tree   SHIPPED ON V2 — commands.rs:3285 reads
           viewer reader            via viewer.decomposition_nodes(...).
  Task #4  get_phase_detail +       SHIPPED ON V2 — commands.rs:9194 +
           get_baby_step_detail     commands.rs:9218 (real SSB reads, not
                                    NotFound stubs).
  Task #5  PlanWindow tree+detail   SHIPPED ON V2 — PlanWindow.svelte
           pane                     renders Concept/Phase/Step/BabyStep
                                    with the 8-block BabyStep detail pane.
  Task #6  dag_repair_round log     SHIPPED ON V2 — get_dag_repair_log at
                                    commands.rs:3461; viewer reader
                                    recent_dag_repair_rounds(100).
  Task #7  get_executable_pipeline  SHIPPED ON V2 — commands.rs:3432.
  Task #9  tree_position_changed    SHIPPED ON V2 (this session) — emitter
                                    trait at pipeline_tree_navigator.rs +
                                    SsbTreePositionChangedEmitter; fired
                                    after every record_result() cursor
                                    mutation. SsbActionType +
                                    TreePositionChangedPayload at ssb.rs.
  Task #10 failure_recorded         SHIPPED ON V2 (this session) — emitter
                                    trait + SsbFailureRecordedEmitter; fired
                                    once per FailureRecord upsert inside
                                    handle_failure(). Shape mirrors the
                                    in-memory FailureRecord exactly
                                    (reason/attempts, not failure_kind /
                                    retry_count — see DECISION (b) below).
  Task #11 bubble_up_typed          SHIPPED ON V2 — SsbBubbleUpEmitter
                                    (pipeline_tree_navigator.rs:1890).
  Task #12 integration_checkpoint_  SHIPPED ON V2 — SsbIntegrationEmitter
           success                  (pipeline_tree_navigator.rs:1927).
  Task #13 research_node_stage      SHIPPED ON V2 (this session) — routed
                                    via existing SsbActionType::
                                    WebResearchStep rows (stage field
                                    populated), no new SsbActionType
                                    variant fabricated. Viewer reader:
                                    research_node_stages(node_id, limit);
                                    Tauri command: get_research_node_history
                                    rewired (no longer returns empty Vec).
                                    See DECISION (e) below.
  Task #14 package_ref_verified     EMITTER SUBSTRATE SHIPPED ON V2,
                                    PRODUCER DATA-FLOW DEFERRED TO
                                    B-7-followup — emitter trait +
                                    SsbPackageRefVerifiedEmitter +
                                    verify_packages_with_emitter() helper
                                    exist and are unit-tested in
                                    research_tree.rs, but drive_task's
                                    decomposition flow has no site where
                                    PackageRef data is materialized
                                    (ConceptDecomposition + PipelineSpec
                                    have no packages field). WIRE-DARK[B-7]
                                    marker at research_tree.rs:2688
                                    references B-7-followup as deferral
                                    home. Shape mirrors the in-memory
                                    PackageRef exactly — no fabricated
                                    method enum (ShapeOnly | RegistryHttp).
                                    See DECISION (c) below.
  Task #15 tech_stack_decided       EMITTER SUBSTRATE SHIPPED ON V2,
                                    PRODUCER DATA-FLOW DEFERRED TO
                                    B-7-followup — emitter trait +
                                    SsbTechStackDecidedEmitter +
                                    new_tech_stack_decision() constructor
                                    helper exist and are unit-tested in
                                    research_tree.rs, but drive_task's
                                    decomposition flow has no site where
                                    TechStackDecision data is materialized
                                    (ConceptDecomposition + PipelineSpec
                                    have no tech_stack field). WIRE-DARK
                                    [B-7] marker at research_tree.rs:2728
                                    references B-7-followup as deferral
                                    home. `source` is the snake_case
                                    DecisionSource discriminant (research |
                                    constraint | default) — no fabricated
                                    RESEARCH | EXISTING_PROJECT |
                                    USER_SPECIFIED | FALLBACK literals.

Remaining DEFERRED to follow-up plan (B-7-follow-up):
  Task #8  phase_sized_split — emitter wire from pipeline_builder.rs::
                                phase_sizing_splitter call sites.
  Task #16 decomposition_alternative panel — Designer surface row +
                                intervention card.
  Task #17 research_cache_hit_rate gauge.
  Task #18 get_research_budget + get_source_tier_distribution real reads.

SCHEMA_VERSION 99 -> 100 in this session covers the 4 producer-wire
variants (TreePositionChanged, FailureRecorded, PackageRefVerified,
TechStackDecided). all_test_payloads bijection: 232 -> 236.

## V2 REBASELINE NOTE (2026-05-07, updated 2026-05-22)

Producer-side work (4D + 4E pipeline factory) is V2-agnostic. Dashboard
surface targets — V1 paths in the original plan are RETIRED, V2 paths
below are CANONICAL:

  V1 crates/dashboard/src/commands.rs        -> V2 crates/dashboard-v2/
                                                 src-tauri/src/commands.rs
  V1 src-frontend/src/shell/WindowRegistry
     .ts                                     -> RETIRED in V2 (routing via
                                                 AppShell + Panel +
                                                 WindowHost; PlanWindow is
                                                 mounted directly by
                                                 WindowHost.svelte:7/64,
                                                 no registry indirection).
  V1 src-frontend/src/windows/PlanWindow
     .svelte (272 LOC, unregistered)         -> crates/dashboard-v2/src/lib/
                                                 windows/PlanWindow.svelte
                                                 (live-wired; stub.ts fall-
                                                 back kept for cold-start:
                                                 get_decomposition_tree,
                                                 get_executable_pipeline,
                                                 get_dag_repair_log,
                                                 get_research_tree,
                                                 get_bubble_up_events,
                                                 get_package_verifications
                                                 NEW B-7-cont-2)
  V1 src-frontend/src/drawers/Research
     Drawer.svelte                           -> crates/dashboard-v2/src/lib/
                                                 drawers/ResearchDrawer
                                                 .svelte (B-7-cont-2 added
                                                 per-stage timeline rows
                                                 from rewired
                                                 get_research_node_history)
  V1 src-frontend/src/drawers/Orchestration
     Drawer.svelte                           -> crates/dashboard-v2/src/lib/
                                                 drawers/OrchestrationDrawer
                                                 .svelte

V2 ActivityWindow PLAN mode (Mode 1) IS the DecompositionTree visualizer
rendered as a constellation. PlanWindow is the structured-tree alternative;
ActivityWindow is the canvas alternative. Both consume the same producer
output. Per DOMAIN-CONTEXT §2 PLAN window: Concept -> Phase -> Step ->
BabyStep with 8-block BabyStep (GROUNDING, PRE_GATE, TASK, TDD,
DETERMINISTIC_CHECK, ACCEPTANCE, INTEGRATION, EXIT_CRITERIA).

## Context

Per audit-dashboard/pipeline-factory-wiring.md: "HEAVY ENGINE, FLAT
WINDSHIELD." 4D + 4E ship 30 sub-steps of substantively-implemented
production code; the dashboard projects ~5% of that surface. PlanWindow.svelte
(272 LOC) is fully written but UNREGISTERED. get_decomposition_tree
hardcodes Ok(DecompositionTreeSnapshot::default()).

## Reads

- audit-dashboard/pipeline-factory-wiring.md (the full inventory)
- audit-implementation/phase-4D.md, 4E.md
- summary/phase-4-summary.md (4D = 18 sub-steps; 4E = 12 sub-steps)
- crates/orchestrator/src/research_tree.rs (Concept/Phase/Step/BabyStep nodes)
- crates/orchestrator/src/pipeline_builder.rs + pipeline_tree_navigator.rs
- crates/dashboard/src-frontend/src/windows/PlanWindow.svelte (272 LOC, unregistered)

## Tasks

1. **Register PlanWindow.** WindowRegistry.ts:27 currently routes
   WindowType.PLAN to Workspace. Decision (per B-3): either register
   PlanWindow as PLAN, or merge them. Recommended: register PlanWindow as
   PLAN; Workspace becomes WindowType.HOME or default landing.

2. **DecompositionTree producer (renamed: concept_decomposition_node).**
   Wire research_tree's ConceptDecomposition production to write SSB
   action `concept_decomposition_node` payload {node_kind, node_id,
   parent_id, name}. SHIPPED ON V2: one row per concept | phase | step |
   baby_step constructed during decompose(), all routed through one
   SsbActionType (no separate phase_node_ready / step_node_ready /
   baby_step_ready variants — single discriminator with `node_kind`
   suffices for the dashboard tree builder).

3. **Implement get_decomposition_tree(plan_id) → tree** in ssb_viewer.rs.
   Read concept_decomposition + child rows, assemble tree struct. Return
   real DecompositionTreeSnapshot.

4. **Implement get_phase_detail(phase_id) → PhaseDetail** + 
   **get_baby_step_detail(baby_step_id) → BabyStepDetail.** SQL reads from
   the SSB rows. Replace NotFound stubs.

5. **PlanWindow content.** PlanWindow renders the full tree:
   `Concept (3) → Phase (8) → Step (24) → BabyStep (87)` with progress
   per node (PASS / PENDING / FAILED counts as bar). Click node for detail
   pane (TASK/GROUNDING/DETERMINISTIC_CHECK/ACCEPTANCE blocks).

6. **DAG repair events log.** Add SSB `dag_repair_round` payload
   {round_index, action: Reorder|Substitute, target_node, success}.
   Also `dag_repair_escalated` when T_DAG=10 exhausted. Dashboard surface:
   add a section to OrchestrationDrawer (or a new RepairLogPanel).

7. **ExecutablePipeline DAG visualizer.** New Tauri command
   `get_executable_pipeline(plan_id)` returns the full DAG +
   ValidationErrorKind history + repair_log. Render with d3 / cytoscape.js
   in Plan window detail pane.

8. **PhaseSizingSplitter events.** When Phase 3.5 splits a phase that
   exceeds available_for_work * 0.6, emit SSB `phase_sized_split` payload
   {original_phase_id, new_phase_ids[], complexity_reason}. Dashboard
   shows in tree as split badge.

9. **TreeNavigator state cursor.** Add SSB `tree_position_changed` per
   navigator step: {current_node, visited[], blocked[], skipped[]}. Plan
   window highlights the current node.

10. **FailureRecord per baby step.** Emit SSB `failure_recorded` payload
    {baby_step_id, failure_kind, retry_count, last_error}. PlanWindow
    detail pane shows per-baby-step retry history.

11. **BubbleUpEvent typed surface.** Today only debug-string flows. Add
    SSB `bubble_up_typed` payload {from_level, to_level, action:
    ResearchAlternative|RedecomposeStep|ReplanPhase|ReplanConcept|Escalate,
    reason}. Dashboard shows cascade hierarchy visualization with arrows
    from BabyStep → Step → Phase → Concept.

12. **IntegrationCheckpoint successes.** Today only failures emit. Add
    `integration_checkpoint_success` per level (BabyStep/Step/Phase/Concept)
    with timing. Dashboard shows green checkmark trail per node.

13. **Research tree per-node FORMULATE/SEARCH/EXTRACT/REFLECT events.**
    Currently web_research_orchestrator.rs:166-396 emits only 5 coarse
    end-of-iteration tokens (iterate/no_results/converged/pruned/max_iterations).
    Add granular emit per stage: `research_node_stage` payload
    {node_id, stage: Formulate|Search|Extract|Reflect|Converge,
    duration_ms, success}. ResearchDrawer shows per-node stage timeline.

14. **PackageRef.verified anti-hallucination badges.** When 4D verifies
    a package, emit `package_ref_verified` payload {name, ecosystem,
    method: ShapeOnly|RegistryHttp, verified}. Dashboard shows badges
    next to each PackageRef in PlanWindow detail.

15. **TechStackDecision visualization.** PackageRef chosen via
    RESEARCH/EXISTING_PROJECT/USER_SPECIFIED/FALLBACK. Surface as
    `tech_stack_decided` SSB rows. Plan window shows decision-source
    badges.

16. **DecompositionAlternative panel.** When Designer surfaces alternatives
    (per audit phase-4D R "INTERACTIVE mode logged not blocking"), surface
    as intervention card asking user to pick.

17. **ResearchCache hit-rate gauge.** Surface session-level cache hit %.

18. **get_research_budget + get_source_tier_distribution** — implement
    real reads.

## WIRING_CONTRACT

produces_symbols:
  - SsbDecompositionEmitter
  - SsbPackageRefVerifiedEmitter
  - SsbTechStackDecidedEmitter
  - SsbDagRepairEmitter
  - SsbBubbleUpEmitter
  - SsbIntegrationEmitter
  - SsbFailureRecordedEmitter
  - SsbTreePositionChangedEmitter
  - WebResearchStepEmitter
  - get_decomposition_tree
  - get_phase_detail
  - get_baby_step_detail
  - get_executable_pipeline
  - get_dag_repair_log
  - get_research_node_history
  - get_package_verifications
  - get_research_budget
  - get_source_tier_distribution
consumes_symbols:
  - PlanWindow.svelte
  - ResearchDrawer.svelte
  - OrchestrationDrawer.svelte
reachable_from:
  - task_runner::drive_task
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_decomposition_tree
stub_ban:
  - get_decomposition_tree returns non-default
  - get_phase_detail not NotFound
  - get_baby_step_detail not NotFound
  - PlanWindow registered
frontend_pair: same B-step
observable_effect:
  trigger: elai task "test decomposition trigger" --repo-root . --mode local --output-format json --max-wall-seconds 30
  expect_ssb: concept_decomposition_node
  expect_artifact: PlanWindow shows tree with real nodes; click drills down

## DETERMINISTIC_CHECK

```bash
# 1. PlanWindow mounted (V2 — WindowRegistry.ts is RETIRED; WindowHost
#    .svelte is the direct mount surface).
rg "import PlanWindow" .elai_cc/crates/dashboard-v2/src/lib/layout/WindowHost.svelte
# Expected: >=1 hit (mount line at WindowHost.svelte:7).

# 2. Stubs eliminated (NotFound replaced with real SSB reads).
rg "DecompositionTreeSnapshot::default\\(\\)" .elai_cc/crates/orchestrator/src/ssb_viewer.rs
# Expected: zero hits in get_decomposition_tree (replaced with real read).

# 3. B-7-cont-2: 4 new SsbActionType variants serialize bijectively.
#    test_typed_payloads asserts 232 -> 236 (+ 4 B-7).
cargo test -p elai-orchestrator --lib test_typed_payloads
# Expected: ok. 1 passed; 0 failed.

# 4. Producer-wire dashboard surface compiles + renders cleanly.
cd .elai_cc/crates/dashboard-v2 && npm run check
# Expected: COMPLETED ... 0 ERRORS

# 5. Smoke: submit task, verify tree populates within 60s; check the
#    PACKAGES VERIFIED row in PlanWindow detail pane after the first
#    research run completes.
```

## ACCEPTANCE

- [ ] PlanWindow registered + renders real tree
- [ ] 14 new SSB action_types added
- [ ] 9 new/replaced Tauri commands
- [ ] DAG repair log visible
- [ ] PackageRef verification badges visible
- [ ] Research per-node stage timeline visible
- [ ] BubbleUp cascade visualization works
- [ ] Tree position cursor highlights current node

## Files touched

- MODIFY: crates/orchestrator/src/research_tree.rs (granular emits)
- MODIFY: crates/orchestrator/src/web_research_orchestrator.rs (stage emits)
- MODIFY: crates/orchestrator/src/pipeline_builder.rs (repair emit)
- MODIFY: crates/orchestrator/src/pipeline_tree_navigator.rs (tree position emit)
- MODIFY: crates/orchestrator/src/ssb.rs (new action_types + payloads)
- MODIFY: crates/orchestrator/src/ssb_viewer.rs (real tree readers)
- MODIFY: crates/dashboard/src/commands.rs (new commands)
- MODIFY: crates/dashboard/src-frontend/src/shell/WindowRegistry.ts
- MODIFY: crates/dashboard/src-frontend/src/windows/PlanWindow.svelte
  (already 272 LOC; consolidate)
- MODIFY: crates/dashboard/src-frontend/src/drawers/ResearchDrawer.svelte
- MODIFY: crates/dashboard/src-frontend/src/drawers/OrchestrationDrawer.svelte
- ADD: dashboard d3 or cytoscape.js dep (with confirmation) for DAG render
- CREATE: workflow/reasoning/B-7-decisions.md

## Grounded constants

T_DAG=10 (max DAG repair rounds before escalation) is documented in the
canonical phase spec at
  docs/specs/guide/steps/phase-4E.txt:273
and used by pipeline_tree_navigator + pipeline_builder repair-loop sites.
No magic-number drift — the value is referenced via a named constant.

SPLIT_RATIO=0.6 (PhaseSizingSplitter trigger threshold relative to
available_for_work) is defined at
  .elai_cc/crates/orchestrator/src/pipeline_builder.rs:53
and consumed at
  .elai_cc/crates/orchestrator/src/phase_sizing_splitter.rs:13.
Task #8 (phase_sized_split emitter) will hook the splitter at the
consumer site; the constant itself is already grounded and stable.

## Extends-vs-forks decision for TaskSplitPayload

This session extends the EXISTING TaskSplitPayload (in ssb.rs, used by
the B-6 supervision layer) with two NEW fields:
  - phase_id: Option<u64>
  - complexity_reason: Option<String>
Both are `#[serde(default)]` so existing rows deserialize without
mutation (msgpack defaults are null/None for Option). No new SsbActionType
variant is forked. Rationale:
  (1) The semantic shape — "a unit of work was split, with a reason" —
      is identical between supervision-layer kills/restarts and the
      pipeline-factory PhaseSizingSplitter. Forking a near-duplicate
      payload would double the producer-side rmp_serde marshalling code
      with no semantic gain.
  (2) Backwards-compatibility: the existing supervision rows continue to
      deserialize cleanly because the two new fields default to None on
      pre-patch rows.
  (3) Dashboard fan-out: PlanWindow + a future SupervisionDrawer both
      consume the same payload struct, no shape negotiation needed.
Trade-off accepted: producers must populate either `agent_role` (existing
B-6 field) OR `phase_id` (new) but never both — a single payload with
optional fields beats two near-duplicate variants.

## Next

B-8 (PipelineGrc activation in production hot paths).
B-7-follow-up (phase_sized_split + decomposition_alternative panel +
research_cache_hit_rate gauge + get_research_budget +
get_source_tier_distribution).
