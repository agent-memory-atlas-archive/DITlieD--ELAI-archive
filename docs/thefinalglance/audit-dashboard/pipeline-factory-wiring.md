# Pipeline Factory L1 + L2 Dashboard Wiring Audit
Date: 2026-04-25
Auditor stance: HOSTILE
Scope: Phase 4D (Pipeline Factory L1 — Designer + FlashResearch +
       Recursive Decomposition + Web Research, 18 sub-steps) and
       Phase 4E (Pipeline Factory L2 — Builder + DAG repair +
       DecompositionTree + TreeNavigator + BubbleUpHandler, 12 sub-steps)
       — surfaced through Tauri dashboard backend
       (F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\) and Svelte
       frontend (F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\).

================================================================
OVERALL VERDICT: HEAVY ENGINE, FLAT WINDSHIELD
================================================================

Phases 4D + 4E ship 30 sub-steps of substantively-implemented
production logic (substantiated in audit-implementation/phase-4D.md and
phase-4E.md): real ResearchTree per-node FORMULATE/SEARCH/EXTRACT/
REFLECT/CONVERGE loops, real ConceptDecomposition/PhaseNode/StepNode/
BabyStepSpec types, real ExecutablePipeline + DependencyGraph +
DagRepairLoop with REORDER/SUBSTITUTE strategies and T_DAG=10 fixpoint
detection, real DepthFirstNavigator + BubbleUpHandler with cascading
3/2/1/1 thresholds across 4 decomposition levels, real IntegrationRunner
emitting checkpoints at BabyStep/Step/Phase/Concept levels, and real
ResearchCache/ResearchBudgetManager/PackageRef/SourceQualityTable
infrastructure. Production wiring of the L1→L2 path is verified at
task_runner.rs:3083 (research_decompose), task_runner.rs:3143
(PipelineBuilder::build), task_runner.rs:3176-3179
(NavigatorTree+DepthFirstNavigator), and task_runner.rs:3197 (baby step
loop driving nav.next_baby_step).

The dashboard sees ONE projection of this, and it is the SHALLOWEST
possible one: a research-step rolling list (5 coarse loop tags) and a
phase-completion timeline mislabeled as the plan tree. Every other
artifact — ConceptDecomposition tree, ExecutablePipeline DAG,
DagRepairLoop's repair_log, NavigatorTree state, FailureRecord cascade,
BubbleUpEvent enum, IntegrationCheckpoint pass/fail per level,
PackageRef.verified, ResearchCache hit-rate, FactExtraction quotes,
Designer's concept/phase/step/baby_step alternatives, SourceTier hit
distribution, ResearchBudgetManager admit/prune progression — is
either INVISIBLE (no command, no panel) or INVISIBLE-VIA-EMPTY-STUB
(command exists but always returns zero / NotFound).

Concretely:
1. The DecompositionTree is NOT visualized. The PlanWindow.svelte
   wrapper that would render Concept→Phase→Step→BabyStep IS WRITTEN,
   but it is NOT mounted (WindowRegistry maps PLAN→Workspace, NOT
   PlanWindow). The backend feeder it would call always returns an
   empty snapshot anyway (ssb_viewer.rs:1383-1391 hard-codes
   `Ok(DecompositionTreeSnapshot::default())` because no SSB row
   carries the tree).
2. The per-research-node FORMULATE/SEARCH/EXTRACT/REFLECT phases are
   NOT separately observable. Only 5 coarse action tokens are emitted
   to SSB ("iterate", "no_results", "converged", "pruned",
   "max_iterations" — see web_research_orchestrator.rs:201/210/331/349/
   360/378). The fine-grained FORMULATE/SEARCH/EXTRACT/REFLECT stages
   are documented as comments in the loop and emitted to `tracing`
   only, never to SSB.
3. DAG repair events are essentially INVISIBLE. There is no SSB
   action_type for repair_action, the dashboard has no panel for
   ExecutablePipeline.repair_log, and the only visible signal is a
   single tracing::info!() line at task_runner.rs:3149 (`pipeline
   built: ... repair_rounds=N`) which goes to stderr, not the SSB,
   not the dashboard.
4. BubbleUpEvent cascade DOES surface, BUT only as undifferentiated
   `Finding` rows in the activity feed. There is no panel showing the
   cascade tree (3 baby fails → ResearchAlternative → 2 step fails →
   RedecomposeStep → 1 phase fail → ReplanPhase → 1 concept fail →
   Escalate). The user sees a flat amber "bubble_up · ..." row plus an
   "integration_bubble_up" row but cannot follow which baby step rolled
   up into which step into which phase.

The phase-4D and phase-4E implementation audits both grade the engines
as substantively complete. This audit closes the dashboard gap and
finds the user-facing surface area is roughly 5% of the implemented
backend.

================================================================
PHASE 4D — PIPELINE FACTORY L1 (FlashResearch + Recursive
                                Decomposition + Web Research)
================================================================

----------------------------------------------------------------
Backend signals available in principle (per phase-4D audit)
----------------------------------------------------------------
- ResearchTree per-node loop with FORMULATE / SEARCH / QUALITY GATE /
  FETCH+EXTRACT / INDEX / RETRIEVE / REFLECT / CONVERGE stages
  (web_research_orchestrator.rs:166-396).
- NodeStatus (Pending / Active / Converged / Pruned),
  ConvergenceStatus (Converged / Partial / Failed), low_gain_streak,
  phi/psi per-iteration, info_gain trend
  (research_tree.rs:553/533/996/1098).
- 5-tier SourceTier table (T1Authoritative .. T5Phishing) with
  per-URL deterministic lookup
  (web_search/source_quality.rs:66-93, 24 tests).
- ResearchTree::find_contradictions, ConvergenceStatus aggregation
  (research_tree.rs:971/996).
- ConceptDecomposition tree: ConceptNode → PhaseNode → StepNode →
  BabyStepSpec with grounding/tdd/acceptance/integration/
  exit_criteria fields, plus TechStackDecision and PackageRef with
  `verified: bool` (research_tree.rs:1245-1390).
- ResearchCache (TTL + LRU) with hits/misses per level
  (research_tree.rs:1448-1480).
- DecompositionTree wrapper at L1508 — full Concept/Phase/Step/
  BabyStep persistable graph with id newtypes.
- WebResearchOrchestrator with per-node iteration stats:
  `node_id, iterations, phi, psi, sources_fetched, sources_used`
  (web_research_orchestrator.rs:525).
- ResearchBudgetManager: admit/prune accounting with
  current_tokens / can_admit / compress_evidence
  (research_budget.rs:83-152).
- WebSearch crate: SearxClient health probe, SourceQualityTable,
  RobotsCache, RateLimiter (per spec defaults).
- ResearchAgent isolation as Tier C process with no reqwest/no
  provider deps (agent/Cargo.toml + agent/src/main.rs INVARIANT).
- VRAM-tiered ResearchTierConfig (8/12/16/24/48GB + Cloud,
  research_tier.rs:50-128).
- 6 FactExtraction-related fields exist on ResearchNode (citations
  with quality + evidence_level + excerpt) but no fact-quote
  emission to SSB.

----------------------------------------------------------------
Backend signal production reality (what reaches the SSB)
----------------------------------------------------------------
- WebResearchStepPayload (ssb.rs:772-781): agent_id, node_id, action
  (String), phi, psi, iteration, sources_fetched, sources_used.
  Emitted at web_research_orchestrator.rs:201/210/331/349/360/378
  via emit_step. The `action` field carries one of FIVE coarse
  tokens: "iterate", "no_results", "converged", "pruned",
  "max_iterations". The substep stage (FORMULATE / SEARCH /
  FETCH+EXTRACT / INDEX / RETRIEVE / REFLECT / CONVERGE) is COMMENTED
  in the loop body but only logged via `debug!()` / `info!()` to
  tracing, never to SSB. Therefore the per-stage progress is NOT
  observable in the dashboard.
- ResearchStepPayload (ssb.rs:760-769): agent_id, node_id, action,
  phi, psi, iteration. Used by the local research path. Same coarse
  action vocabulary.
- SsbActionType::ResearchDecomposeLevel is registered (ssb.rs:413,
  registered as "research_decompose_level"). Emitted by
  research_tree.rs:1194 record_research_decompose_level — one row
  per concept/phase/step/baby_step level walked. Carries the level
  label only; no concept/phase/step IDs flow to SSB.
- ZERO SSB action_type carries: ConceptDecomposition tree, PhaseNode/
  StepNode/BabyStepSpec content, PackageRef list with verified flag,
  TechStackDecision, ResearchCache hits, SourceTier per-URL hit
  counts, FactExtraction quotes, decomposition Alternatives,
  Contradictions, EvidenceLevel upgrades, ResearchReport.

----------------------------------------------------------------
Dashboard wiring — what reaches the UI
----------------------------------------------------------------

(A) ResearchDrawer.svelte + ResearchWindow.svelte
    File: src-frontend/src/drawers/ResearchDrawer.svelte (102 LOC)
          src-frontend/src/windows/ResearchWindow.svelte (40 LOC)
    Backend: get_research_tree (commands.rs:3737-3746),
             get_research_budget (commands.rs:3748-3753),
             get_source_tier_distribution (commands.rs:3755-3760),
             get_searxng_status (commands.rs:3768-3823).
    Layout: ResearchWindow auto-mounts in PRIMARY slot when FSM is
            in RESEARCHING state (layoutState.svelte.ts:131-134) and
            is reachable as a tab pick otherwise. CHAT goes to
            SECONDARY in that layout.
    On expand:
    - Promise.all of the four invokes (lines 37-42).
    - get_research_tree returns up to STEP_LIMIT=25 ResearchStepEntry
      rows from ssb_viewer.rs:2435 (UNION of WebResearchStep +
      ResearchStep). Each row renders as
      `[kind] n{node_id}·i{iteration} {action} φ{phi} ψ{psi}`. The
      `action` field carries the same 5-token coarse vocabulary
      ("iterate", "no_results", "converged", "pruned", "max_iterations")
      so the user CANNOT see FORMULATE / SEARCH / FETCH+EXTRACT /
      INDEX / RETRIEVE / REFLECT individually.
    - get_research_budget reads
      ssb_viewer.rs:2508-2518 which IS A KNOWN STUB. It returns
      `{admitted: 0, pruned: 0, tokens_used: 0, tokens_max: 0,
      known: false}` and the drawer body shows literal "no data
      yet". DEF-T3-RESEARCH-1 of the dashboard-wiring plan is
      cited as the candidate writer-side row type — never landed.
    - get_source_tier_distribution reads
      ssb_viewer.rs:2526-2537, ALSO A KNOWN STUB returning all
      zeros + `known: false`. Drawer renders "no data yet". The
      5-tier table itself IS real (web_search/source_quality.rs:66
      with 24 tests + >=200 trusted domains in elai_cc.toml) but
      the per-research-fetch counters are not persisted, so the
      distribution per task is invisible.
    - get_searxng_status is a 2-second probe of the sidecar
      `/search?q=ping` endpoint. Real network call, real elapsed_ms,
      real error string. Header shows `searxng ok|down`.
    Expanded behaviour:
    - One-shot load on toggle. NO polling. If a 30-iteration node is
      mid-flight when the user opens the drawer, the drawer shows the
      first snapshot and never updates until manual collapse+re-expand.
    VERDICT: PARTIAL.
    - SearXNG reachability: SURFACED (real probe).
    - Research step trace: PARTIAL (kind/node_id/iteration/phi/psi/
      sources are real, action vocabulary is too coarse to see the
      sub-stages spec calls out).
    - Budget progression: STUB (`known: false`).
    - Source tier hit distribution: STUB (`known: false`).

(B) ActivityFeed / ActivityEntry — research-step recognition
    File: src-frontend/src/ActivityEntry.svelte
    Lines: 229-231 colour for `web_research_step`, `research_step`,
           `research_decompose_level`.
    typeLabel: lines 1115-1132 render
           `research · step | research · L{N} | research · iterate |
            research · no_results | research · converged | research ·
            pruned | research · max_iterations | research · web`.
    VERDICT: SURFACED-COLOUR-ONLY. Each research event is tagged with
    its own colour and label. Clicking the row opens the generic
    DiffViewer / detail pane on `seq` — there is no research-tree
    tooltip showing the parent/child relationship of nodes. The user
    sees a flat sequence of research events, not the BFS/DFS shape of
    the tree.

(C) WebQualityDrawer.svelte — adjacent but unrelated to 4D
    File: src-frontend/src/drawers/WebQualityDrawer.svelte (140 LOC)
    Backend: get_build_slot_state, get_dev_server_state,
             get_web_validation_results.
    Reads the 4K WebValidator surface (build slots, dev server, 7-tier
    web-app validation). NOTHING here surfaces 4D source-quality
    fetching. Despite the name, it has no relationship to 4D's
    SourceQualityTable / SourceTier usage.
    VERDICT: NOT-RELATED. Listed in the input set; reviewed only to
    confirm it does not back-stop 4D source-tier visibility.

(D) ProjectIntelligenceDrawer.svelte — orthogonal
    File: src-frontend/src/drawers/ProjectIntelligenceDrawer.svelte (135 LOC)
    Backend: get_project_profile, get_override_file, write_override_file.
    Reads ProjectProfile (language, build_system, framework, test
    runner, linter, constraints_count) and the .elai-override.toml
    file. NO surface for ConceptDecomposition / TechStackDecision /
    PackageRef / Designer alternatives.
    VERDICT: NOT-RELATED. The drawer's `constraints_count` is an
    integer with no relationship to 4D's per-baby-step grounding /
    integration constraints.

(E) Designer alternatives, Contradictions, EvidenceLevel upgrades,
    PackageRef.verified, ResearchCache hit-rate, FactExtraction
    quotes — INVISIBLE
    There is no Tauri command, no backend reader, no Svelte panel,
    no SSB row carrying any of these. The Designer agent's per-level
    "alternatives presented" path (research_tree.rs:1541
    ExecutionMode::Interactive) is logged as `tracing::info!` and
    was already noted in phase-4D.md as not blocking the runtime
    (the UI gate is intentionally delegated to the dashboard, but
    the dashboard never sees it). PackageRef.verified is real on the
    decomposition object but never serialized to SSB.
    VERDICT: ALL DEAD-INVISIBLE.

================================================================
PHASE 4E — PIPELINE FACTORY L2 (Builder + DAG repair +
                                DecompositionTree + TreeNavigator
                                + BubbleUpHandler)
================================================================

----------------------------------------------------------------
Backend signals available in principle (per phase-4E audit)
----------------------------------------------------------------
- ExecutablePipeline (pipeline_builder.rs:256) with: pipeline_id
  (blake3-derived), DependencyGraph, fsm_states,
  novel_step_ids, agent_configs, rule_configs, resource_estimate,
  test_harness, repair_log (Vec<RepairAction>), output_dir.
- DependencyGraph with structural_hash() (blake3) for fixpoint
  detection (pipeline_builder.rs:97-104).
- StructuralValidator with 8 ValidationErrorKind variants
  (MissingTool, ResourceOverflow, Cycle, UnmeasurableMetric,
  InvalidPredicate, MissingAdapter, InvalidVisualCheck,
  MissingDependency).
- DagRepairLoop (pipeline_builder.rs:563-710) with REORDER (cycle
  back-edge removal) + SUBSTITUTE (tools.find_substitute) strategies,
  T_DAG=10 budget, fixpoint detection.
- PhaseSizingSplitter (phase_sizing_splitter.rs, runtime split when
  AutonomyDecision::SplitTask).
- DecompositionLevel enum (BabyStep / Step / Phase / Concept) with
  Ord (pipeline_tree_navigator.rs:73-79).
- BubbleUpAction enum: ResearchAlternative, RedecomposeStep,
  ReplanPhase, ReplanConcept, Escalate, plus DiagnoseAndRetry
  (pipeline_tree_navigator.rs:83-96).
- BubbleUpPolicy with thresholds 3/2/1/1 (DEFAULT_MAX_RETRIES_*).
- BubbleUpEvent (level, action, failure_count, threshold).
- FailureRecord (baby_step_id, step_id, phase_id, concept_id,
  reason, attempts).
- IntegrationCheckpoint (level, success, failure_reason, duration_ms).
- TreeNavigatorState — full snapshot (completed/failed/blocked/
  skipped sets, execution_log, bubble_up_log,
  integration_checkpoints, failure_records, cascade & integration
  step/phase/concept counters), serializable for crash recovery.
- TreeNavigator trait (object-safe) + DepthFirstNavigator impl.
- IntegrationRunner (NoOp + CommandIntegrationRunner) with 4-level
  registration at task_runner.rs:5846.

----------------------------------------------------------------
Backend signal production reality (what reaches the SSB)
----------------------------------------------------------------
- ZERO SsbActionType variants exist for: PipelineBuilt, RepairAction,
  RepairOutcome, BubbleUpEvent (typed), IntegrationCheckpointResult
  (typed), NavigatorState, FailureRecord, PhaseSizingSplit,
  StructuralValidatorError. The SsbActionType enum (ssb.rs:98-298,
  ~70 variants) does NOT contain any of these.
- BubbleUpAction is emitted as a Finding row (severity=warning,
  category="bubble_up", message=`{:?}` of the BubbleUpAction enum)
  at task_runner.rs:3505-3513. Single line of text. No structured
  fields.
- IntegrationCheckpoint failures emitted as Finding rows
  (category=INTEGRATION_CHECKPOINT_FAILED_CATEGORY,
  task_runner.rs:6003-6009).
- Integration-triggered bubble-ups emitted as Finding rows
  (category=INTEGRATION_BUBBLE_UP_CATEGORY,
  task_runner.rs:5926/5935/5944/5966/5974/5982 + emit helper at 6015).
- DAG repair: tracing::info!("pipeline built: {} fsm_states, {}
  dag_nodes, repair_rounds={}", ...) at task_runner.rs:3149. Goes
  to stderr only. No SSB row, no Finding, no metric persisted.
- ExecutablePipeline output_dir (.elai_cc/pipelines/{id}) is written
  to disk but the dashboard does not read it.

----------------------------------------------------------------
Dashboard wiring — what reaches the UI
----------------------------------------------------------------

(A) PlanWindow.svelte (DEAD on shipping path) — would-be
    DecompositionTree visualizer
    File: src-frontend/src/windows/PlanWindow.svelte (272 LOC)
    Backend: get_decomposition_tree (commands.rs:2620-2641),
             get_phase_detail (commands.rs:2644-2656),
             get_baby_step_detail (commands.rs:2659-2673).
    The component is fully WIRED — fetches a Concept→Phase→Step→
    BabyStep snapshot every 1500ms, renders the tree with collapsible
    `<details>` per concept, passes a `babyStepsByStep` map to the
    legacy PlanTree for per-step BabyStep extension, and provides a
    detail pane that calls `get_phase_detail` / `get_baby_step_detail`
    on click. This is exactly the missing DecompositionTree view.
    BUT:
    1. Header comment (line 7) says "Registration in WindowRegistry.ts
       is intentionally deferred to a follow-up patch (parallel-track
       non-overlap with Track A)". WindowRegistry.ts:27 maps
       `WindowType.PLAN → Workspace`, NOT PlanWindow. PlanWindow is
       not imported anywhere; ZERO production code paths mount it.
    2. Even if mounted, the backend feeder is a STUB. ssb_viewer.rs:
       1383-1391 returns `Ok(DecompositionTreeSnapshot::default())`
       (empty concepts list) with the comment "Returns an empty
       snapshot today. The orchestrator does NOT yet persist a
       concept_decomposition SSB row carrying the full
       ConceptDecomposition tree". The user would see "no
       decomposition yet" forever.
    3. get_phase_detail (ssb_viewer.rs:1412-1447) only confirms the
       phase has emitted a `phase_complete` SSB row, then returns
       `PhaseDetail::empty(phase_id)` — the description /
       expected_outcome / dependencies are always (unknown) /
       (unknown) / (none) regardless of the real PhaseNode content.
    4. get_baby_step_detail (ssb_viewer.rs:1456-1478) ALWAYS returns
       SsbViewerError::NotFound. The comment explicitly states "No
       BabyStepSpec persistence yet — every lookup is a NotFound."
    VERDICT: TRIPLE-DEAD. Component is not mounted; if mounted the
    feeder returns empty; if the feeder were populated the leaf
    fetches return NotFound. Three independent gaps stacked.

(B) Workspace.svelte → PlanTree.svelte (the SHIPPING "plan" surface)
    File: src-frontend/src/Workspace.svelte (line 110 mounts PlanTree)
          src-frontend/src/PlanTree.svelte (162 LOC)
    Backend: get_phase_progress (commands.rs:73-78) +
             step_progress (commands.rs:329-336).
    PlanTree polls `get_phase_progress` every 1500ms. This reads
    PhaseComplete SSB rows aggregated by phase_id (ssb_viewer.rs:790
    range, per the rules-plan-wiring.md sister audit). Each phase row
    expands to step-level via `step_progress`. There is NO concept
    layer above the phase, NO baby-step layer below the step. The
    optional `babyStepsByStep` prop exists but Workspace never
    populates it (it would need PlanWindow's snapshot).
    VERDICT: WRONG-LEVEL. The "plan" surface the user actually sees
    is a phase-completion timeline, not the L2 DecompositionTree.
    Concepts and BabyStepSpecs cannot be reached from here.

(C) ActivityFeed / ActivityEntry — bubble_up + integration recognition
    File: src-frontend/src/ActivityEntry.svelte
    The `bubble_up` signal flows as Finding rows. The activity entry's
    finding-category branch lists `integration_bubble_up` and
    `integration_checkpoint_failed` (lines 47-48,
    260-263 colour=warn). The "bubble_up" string emitted from
    task_runner.rs:3509 is NOT in the FINDING_CATEGORIES list (lines
    44-88), so cascade bubble_ups land via the generic finding default
    (warn, label="finding · warning"). The user can see the row but
    cannot tell baby-step-cascade vs integration-checkpoint apart
    without reading the truncated msgpack tail.
    `research_decompose_level` IS handled (label "research · L{N}"
    at line 1115-1119). But the level number is the only structured
    field — concept/phase/step IDs are not surfaced.
    VERDICT: PARTIAL-FLAT. Cascade is renderable as individual rows
    but never aggregated into a hierarchy. The user cannot follow:
    "this BabyStep failed 3 times → triggered ResearchAlternative on
    its parent Step → which then exhausted its 2 retries → triggered
    RedecomposeStep on its parent Phase". They see four warn rows
    spaced apart in time.

(D) ExecutablePipeline DAG, DagRepairLoop, RepairAction — INVISIBLE
    No Tauri command exposes ExecutablePipeline. No backend reader
    queries `.elai_cc/pipelines/{id}`. No Svelte panel visualizes a
    DAG. The only signal that DAG repair fired is a single
    tracing::info!() at task_runner.rs:3149 of the form
    `pipeline built: 5 fsm_states, 5 dag_nodes, repair_rounds=2` which
    goes to stderr; the user never sees it. There is no surface for:
    - The 8 ValidationErrorKind variants (MissingTool, Cycle,
      ResourceOverflow, ...).
    - REORDER vs SUBSTITUTE strategy distribution.
    - T_DAG=10 budget consumption.
    - Fixpoint hash comparison.
    - Per-repair RepairAction enum (Reorder { cycle_edge: (a, b) },
      Substitute { from: tool_id, to: tool_id }).
    - Pipeline structural_hash (blake3) for change detection.
    VERDICT: ALL DEAD-INVISIBLE. The user cannot tell whether their
    submitted task triggered ANY structural-validator failure, let
    alone which repair was attempted. If RepairOutcome::Escalated
    fires, task_runner returns PipelineBuilderError::RepairEscalated
    which surfaces as a generic agent failure in the activity feed
    with NO indication that the cause was DAG repair exhaustion.

(E) NavigatorTree state (TreeNavigatorState) — INVISIBLE
    The full state object — completed/failed/blocked/skipped sets,
    execution_log, bubble_up_log, integration_checkpoints,
    failure_records, cascade counters per level — is held in-memory
    by task_runner during a baby-step loop (line 3197+) and serialized
    only for crash recovery. There is no Tauri command, no SSB row,
    no panel, no drawer. The user has no way to see "we are 5 of 12
    baby_steps into Step S3 of Phase P2 of Concept C1, 2 baby_steps
    failed and were retried, 1 was blocked".
    VERDICT: DEAD-INVISIBLE. The Workspace's PlanTree shows phase
    progress (X/Y steps complete) which is a coarse projection of
    nav state but loses the baby-step granularity AND the
    cascade/failure dimension.

(F) Integration-checkpoint per-level pass/fail — PARTIAL
    Backend emits `integration_checkpoint_failed` and
    `integration_bubble_up` Finding rows at the 4 levels (BabyStep at
    task_runner.rs:3534, Step/Phase/Concept at 5926/5935/5944,
    auxiliary fall-through at 5966/5974/5982). The activity entry
    handles `integration_checkpoint_failed` as warn-coloured Finding
    (ActivityEntry.svelte line 48 + 261). The user sees the failures
    in the feed.
    BUT: there is no panel showing the 4-level checkpoint matrix
    (BabyStep ✓ Step ✓ Phase ✗ Concept ?), no drilldown into the
    failed criterion, no per-level success rate. CommandIntegrationRunner
    runs with per-criterion timeout INTEGRATION_CRITERION_TIMEOUT_SECS=30
    (pipeline_tree_navigator.rs:34) and aggregates `success` —
    only the aggregate boolean reaches SSB.
    VERDICT: PARTIAL. Feed-level visibility of failures only;
    successes are invisible (no row emitted on PASS); structural
    overview missing.

(G) PhaseSizingSplit events — PARTIAL
    The `task_split` SsbActionType IS registered (ssb.rs:437) and
    rendered in ActivityEntry (lines 939-963: surfaces sub_task_count,
    complexity_estimate, depth). This is emitted from the autonomy
    gate's SplitTask path (Phase 4G), not from PipelineBuilder's
    inline Phase Sizing stage. PhaseSizingSplitter from
    phase_sizing_splitter.rs is consulted at runtime; per the 4E
    implementation audit "PipelineBuilder::build does NOT call
    phase_sizing_splitter inline". The split that fires from the
    gate IS visible; a build-time Phase Sizing split would not be.
    VERDICT: SURFACED-INDIRECT. Visible only via the autonomy gate
    path, not as a PipelineBuilder stage signal.

================================================================
KEY UNANSWERED QUESTIONS
================================================================

(1) Is the DecompositionTree visualized in the dashboard
    (tree view of phases → steps → baby_steps)?
    NO. The intended visualizer (PlanWindow.svelte) is written but
    NOT mounted (WindowRegistry maps PLAN → Workspace.svelte). The
    backend feeder it would call (get_decomposition_tree) hard-codes
    an empty snapshot at ssb_viewer.rs:1383-1391 because no SSB row
    carries ConceptDecomposition. The actually-mounted "plan" surface
    is Workspace → PlanTree, which polls PhaseComplete rows and
    renders a flat phase→step list with no concept layer above and
    no baby_step layer below (the babyStepsByStep prop is unused).

(2) Are research tree FORMULATE / SEARCH / EXTRACT / REFLECT events
    visible per node?
    NO. The 7-stage substep loop (FORMULATE / SEARCH / QUALITY GATE /
    FETCH+EXTRACT / INDEX / RETRIEVE / REFLECT / CONVERGE in
    web_research_orchestrator.rs:166-396) only emits FIVE coarse
    end-of-iteration tokens to SSB: "iterate", "no_results",
    "converged", "pruned", "max_iterations". The fine-grained
    substages are documented as code comments and emitted to
    `tracing::debug!()` / `info!()`, never to SSB. The
    ResearchDrawer renders the coarse tokens as
    `[web|local] n{node}·i{iter} {action} φ{phi} ψ{psi}`, so the
    user sees iteration progress and phi/psi convergence but cannot
    distinguish a search-failed iteration from an extract-failed
    iteration.

(3) Does the user see DAG repair events when validation fails?
    NO. There is no SsbActionType for repair_action / repair_outcome
    / pipeline_built. The only repair signal is a single
    tracing::info!() line at task_runner.rs:3149 going to stderr.
    No Tauri command exposes ExecutablePipeline.repair_log. No
    panel visualizes the DAG, the 8 ValidationErrorKind variants,
    REORDER/SUBSTITUTE strategy distribution, or T_DAG budget
    consumption. RepairOutcome::Escalated surfaces only as an opaque
    PipelineBuilderError::RepairEscalated bubbling into the activity
    feed as a generic agent failure with no attribution to DAG
    repair exhaustion.

(4) What pipeline-factory signals are INVISIBLE?
    INVISIBLE-NO-COMMAND, INVISIBLE-NO-PANEL:
    - ConceptDecomposition tree (concepts, phases, steps,
      baby_steps with grounding/tdd/acceptance/integration/
      exit_criteria).
    - PhaseNode / StepNode / BabyStepSpec content (description,
      expected_outcome, dependencies are always returned as
      "(unknown)").
    - PackageRef list with verified flag (anti-hallucination gate
      result is invisible to user).
    - TechStackDecision per concept/phase.
    - DecompositionAlternative (Designer's alternatives presented
      in INTERACTIVE mode are tracing-only).
    - Contradiction set per ResearchTree (find_contradictions output).
    - EvidenceLevel upgrades (SingleSource → MultiSource).
    - SourceCitation list per node (URL, title, quality, evidence
      level, excerpt) — only count flows via sources_used counter.
    - FactExtraction quotes.
    - ResearchCache hit-rate and TTL state.
    - SourceTier per-URL hit distribution per task (table is real,
      stub viewer returns all zeros).
    - ResearchBudgetManager admit/prune progression (stub viewer
      returns all zeros).
    - ResearchReport (build_report output of ResearchTree).
    - ConvergenceStatus aggregation (Converged/Partial/Failed).
    - ExecutablePipeline (entire object — id, dag, fsm_states,
      novel_step_ids, agent_configs, rule_configs,
      resource_estimate, test_harness, repair_log).
    - DependencyGraph + structural_hash.
    - StructuralValidator's 8 ValidationErrorKind variants.
    - DagRepairLoop's REORDER/SUBSTITUTE strategy distribution,
      T_DAG budget consumption, fixpoint detection.
    - PhaseSizingSplitter at build time (only runtime/gate splits
      surface).
    - TreeNavigatorState (completed/failed/blocked/skipped sets,
      execution_log, bubble_up_log, integration_checkpoints,
      failure_records, per-level cascade counters).
    - FailureRecord per baby step (baby_step_id, step_id, phase_id,
      concept_id, reason, attempts).
    - BubbleUpEvent typed structure (level, action, failure_count,
      threshold) — only the BubbleUpAction enum debug-string flows
      via Finding.
    - IntegrationCheckpoint successes (only failures emit Finding
      rows; PASSes are invisible).
    - Per-criterion timeout breach (aggregate boolean only).
    - Cascade hierarchy (the user sees flat "bubble_up" rows with
      no parent/child link).
    - RepairOutcome::Escalated attribution.
    - Spec_from_decomposition flattening — the linear PipelineSpec
      created at task_runner.rs:6044 is invisible.

    INVISIBLE-VIA-EMPTY-STUB (command exists, returns empty/zero):
    - get_decomposition_tree (always returns empty concepts list).
    - get_phase_detail (always returns PhaseDetail::empty —
      description/expected_outcome/dependencies always None).
    - get_baby_step_detail (always returns NotFound).
    - get_research_budget (always returns zeros + known=false).
    - get_source_tier_distribution (always returns zeros + known=false).

================================================================
EVIDENCE SUMMARY (FILES + LINES)
================================================================

DASHBOARD BACKEND (Tauri commands):
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\commands.rs:2620-2641
  (get_decomposition_tree)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\commands.rs:2644-2656
  (get_phase_detail)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\commands.rs:2659-2673
  (get_baby_step_detail)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\commands.rs:3737-3746
  (get_research_tree)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\commands.rs:3748-3753
  (get_research_budget)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\commands.rs:3755-3760
  (get_source_tier_distribution)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\commands.rs:3768-3823
  (get_searxng_status)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\lib.rs:1457-1459 +
  1518-1521 (Tauri handler registration)

VIEWER STUBS (return empty / zeros / NotFound):
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs:1383-1391
  (decomposition_tree always empty)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs:1412-1447
  (phase_detail returns ::empty shell)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs:1456-1478
  (baby_step_detail always NotFound)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs:2508-2518
  (research_budget_snapshot stub)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs:2526-2537
  (source_tier_distribution stub)

VIEWER REAL READS:
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs:2435-2499
  (research_steps — UNION of WebResearchStep + ResearchStep)

SSB EMITTERS (research / bubble_up / integration / repair):
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\web_research_orchestrator.rs:201/210/331/349/360/378
  (emit_step — 5 coarse action tokens, NOT FORMULATE/SEARCH/EXTRACT/
  REFLECT)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\research_tree.rs:1194
  (record_research_decompose_level)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs:3149
  (DAG repair → tracing::info! ONLY, not SSB)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs:3505-3513
  (BubbleUpAction → Finding row, debug-string only)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs:5926/5935/5944/5966/5974/5982
  + 6015 (integration bubble-ups → Finding rows)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs:6003-6009
  (integration checkpoint failures → Finding rows)

DASHBOARD FRONTEND:
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\drawers\ResearchDrawer.svelte
  (102 LOC — research-step list + budget/tier stubs)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\windows\ResearchWindow.svelte
  (40 LOC — host wrapper)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\windows\PlanWindow.svelte
  (272 LOC — DecompositionTree-aware, NOT mounted in WindowRegistry)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\PlanTree.svelte
  (162 LOC — phase/step polling, mounted via Workspace)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\Workspace.svelte:110
  (mounts PlanTree, NOT PlanWindow)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\shell\WindowRegistry.ts:27
  (WindowType.PLAN → Workspace; PlanWindow unimported)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\shell\layoutState.svelte.ts:131-134
  (RESEARCHING → ResearchWindow primary)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\ActivityEntry.svelte:229-231
  (research_step / web_research_step / research_decompose_level
  colour mapping)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\ActivityEntry.svelte:1115-1132
  (research typeLabel — surfaces level / 5-token action vocabulary)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\ActivityEntry.svelte:47-48,260-263
  (integration_bubble_up + integration_checkpoint_failed Finding
  recognition; the bare "bubble_up" category from cascade is NOT in
  FINDING_CATEGORIES and falls through to the generic finding default)

================================================================
SUMMARY VERDICT
================================================================

The pipeline factory is RICHLY IMPLEMENTED in the engine and POORLY
PROJECTED to the user. Phase 4D's research tree does flow research
events to the dashboard, but at FIVE coarse tokens of granularity
instead of the seven-stage FORMULATE / SEARCH / QUALITY GATE /
FETCH+EXTRACT / INDEX / RETRIEVE / REFLECT / CONVERGE the spec calls
out. Phase 4D's ConceptDecomposition + PackageRef + Designer
alternatives + Contradictions + EvidenceLevel + Citations + Cache
state + per-task SourceTier distribution + ResearchBudget progression
are ALL invisible.

Phase 4E's ExecutablePipeline + DAG + Repair are entirely invisible
(no command, no panel, no SSB row — only a stderr tracing line).
Phase 4E's TreeNavigatorState (the full execution graph at runtime,
including FailureRecords and per-level cascade counters) is entirely
invisible. Phase 4E's BubbleUpEvent cascade IS in the activity feed
but as flat undifferentiated Finding rows — the user cannot follow a
3-fail-baby-step → ResearchAlternative → 2-fail-step → RedecomposeStep
→ 1-fail-phase → ReplanPhase → 1-fail-concept → Escalate trajectory.
Phase 4E's IntegrationCheckpoint surfaces as feed rows on FAILURE
only; PASS events emit nothing.

The single fully-wired-but-dead piece is PlanWindow.svelte: the
component CAN render Concept→Phase→Step→BabyStep, the Tauri commands
ARE registered (get_decomposition_tree / get_phase_detail /
get_baby_step_detail), but (a) WindowRegistry routes PLAN to
Workspace.svelte instead, (b) the backend reader hard-codes an empty
snapshot, and (c) the leaf detail commands always return NotFound /
PhaseDetail::empty. Three independent gaps stacked.

Production wiring of L1 → L2 (research_decompose →
spec_from_decomposition → PipelineBuilder::build → NavigatorTree →
DepthFirstNavigator → baby_step loop) is REAL in task_runner. The
user sees the side effects (a research-step list, individual
bubble_up Finding rows, phase-completion progress) but never the
structure that ties them together. The DecompositionTree visualizer
remains TO-BE-WRITTEN on the writer side (ConceptDecomposition →
SSB row) before any frontend tree view will show non-empty data.
