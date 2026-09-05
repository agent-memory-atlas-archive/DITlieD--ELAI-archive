# ELAI Dashboard Wiring Audit — Aggregate Synthesis
Date: 2026-04-25
Scope: 6 wiring audit reports covering knowledge, reasoning, orchestration core, rules+plan, session+DST, inference, tools+hooks+isolation, pipeline factory
Method: HOSTILE — every wiring claim verified against frontend Svelte components, Tauri commands, and SSB writer call sites. Inventory files (dashboard-wiring-plan.txt + dashboard-wiring-inventory.txt) treated as untrusted.


================================================================================
EXECUTIVE VERDICT
================================================================================

The Tauri dashboard SHIPPED but is OVERWHELMINGLY UNWIRED to its own backend.

  Phase area         Backend signals  Surfaced  Partial  Invisible  Dead UI
  -----------------  ---------------  --------  -------  ---------  -------
  Knowledge engine   ~67              ~6 (9%)   2        54 (81%)   6
  Reasoning engine   ~25              0         1        24         0
  Orchestration core ~40              ~5        4        31         many
  Rules + Plan       ~30              ~3        1        26         8
  Session + DST      ~12              0         0        12         5
  Inference (3J)     ~25              ~6 (24%)  3        16         3
  Tools/Hooks/Isol.  ~35              ~7        4        24         3
  Pipeline factory   ~50              ~3 (6%)   2        45         5
  -----------------  ---------------  --------  -------  ---------  -------
  TOTAL              ~284             ~30 (~11%) ~17     ~232 (~82%) ~30

This is the single root cause behind the user's complaint that the dashboard
"doesn't feel or look like I wanted." The dashboard renders ~11% of what the
backend produces. ~82% of the engine's intelligence is INVISIBLE to the user.
~30 UI elements display hardcoded zeros / stubs / "no data yet" because the
backend producer was never wired up.

The dashboard frontend is structurally DENSE (98 Tauri commands, 9 windows,
10 drawers). The problem is not lack of UI — the problem is that the UI is
not connected to the running engine.


================================================================================
THE SINGLE HIGHEST-IMPACT FIX
================================================================================

From orchestration-core-wiring.md §5:

  "Append `writer.write_payload('fsm', 0, SsbPayload::PhaseTransition(...),
   Vec::new())` inside `Fsm::transition_with_metrics` at fsm.rs:639.
   That one change activates StatusBar badge, MonitorWindow.currently,
   ActivityEntry's phase_transition renderer, and the EXECUTING-pulse
   animation in one shot."

~30-minute, ~10-line patch. Fixes 4 visible UI elements simultaneously.
The current state: StatusBar's elaborate 10-state stateColour/shouldPulse
switch (StatusBar.svelte:308-357) is dead code because the only production
writers of PhaseTransitionPayload are test fixtures. The badge defaults to
"IDLE" forever in production.

This is a microcosm of the entire wiring problem.


================================================================================
WIRING FAILURE TAXONOMY
================================================================================

The audits surfaced 7 distinct wiring failure shapes. B-plan must address each:

SHAPE A — STUB COMMAND
  Tauri command registered, returns hardcoded empty/zero. Examples:
    get_pagerank_top → Vec::new() (ssb_viewer.rs:2196)
    get_query_tier_stats → all zeros (ssb_viewer.rs:2205)
    get_freshness_state → known:false/zeros (ssb_viewer.rs:2219)
    get_decomposition_tree → DecompositionTreeSnapshot::default()
    get_phase_detail → PhaseDetail::empty
    get_baby_step_detail → NotFound always
    get_research_budget → empty
    get_source_tier_distribution → empty
  Fix: implement the SQL/state read.

SHAPE B — RENDERER WITHOUT PRODUCER
  Frontend has switch-case for an SSB action_type the backend never writes.
  Examples:
    knowledge_degradation render at ActivityEntry.svelte:382, 1133 — no
      production writer (CrossEncoderReranker uses NoopSsbNotifier)
    branch_merge render — only test fixtures write this action_type
    isolation · metric render at ActivityEntry.svelte:1135-1136 — no
      production caller of compute_isolation_metric
    chat session_handoff entries — schema mismatch (Settings expects
      {id, branch, ts}, backend returns {seq, ts_ms, winner, abandoned, ...})
    PlanWindow.svelte (272 LOC) fully implemented but unregistered in
      WindowRegistry.ts:27 (PLAN routes to Workspace, not PlanWindow)
    IpcToolRegistry handlers in tools/ — orchestrator wires
      InProcessToolDispatcher instead
    IpcHookDispatcher in plugins/ — orchestrator wires
      InProcessHookDispatcher instead
  Fix: either implement the producer OR delete the consumer to avoid
  user-facing confusion.

SHAPE C — INVISIBLE PRODUCER
  Backend writes rich SSB rows / has rich state — no Tauri command exposes
  it; no Svelte component renders it. Examples:
    QueryTrace per query (pipeline stages, latency per stage, breaker state)
    SearchMode current state + cascade transitions
    Per-model EmbeddingCircuitBreaker state (5 models)
    HyDE hypothesis text + cache hits
    Per-tier Recall@5/Recall@10/MRR (real values in metrics/phase_3K_rag.toml)
    Token-savings -94.3% headline
    All AgentGrcConfig per agent
    All CSQ Critical/guard_added/passed events
    All InlineFeasibility Pass/Fail/Blocked outcomes
    7 EvidenceGate verdicts + suggestions
    InteractiveGate Approve/Reject/Modify/Redirect prompts
    SupervisionManager events
    BSP superstep counter / channel state
    ContextBudget per-slot allocation in use
    file_overlap_score (3E.5)
    Per-rule deprecated_patterns matches
    PlanSkeleton tree (DAG visualization)
    DAG repair Reorder/Substitute distribution + T_DAG counter
    DecompositionTree concept layer
    BabyStep typed result + bubble_up typed event
    IntegrationCheckpoint successes (only failures emit)
    Sandbox tier (A/B/C) badge
    Per-event hook axis (4 events collapse to 1 render line)
    T1/T2/T3 hook latency budgets/measurements
    Per-step inference latency / tokens-per-sec
    Live token streaming for dashboard chat
  Fix: add SsbActionType + emit + Tauri command + render.

SHAPE D — POLLING-ONLY (no live updates)
  Most drawers poll on first expand, no re-poll. Examples:
    RulesDrawer: only on expand (no live update of lifecycle counts)
    ToolsHooksDrawer: only on first expand
    AntiStalenessDrawer: never polls (permanent "no data yet")
    KnowledgeDrawer: stubs anyway
    Settings panels: mostly mount-only
  Fix: 1s polling for state-snapshot data, WebSocket / SSB broadcast
  for event-stream data.

SHAPE E — SCHEMA MISMATCH
  Frontend types disagree with backend response shape. Examples:
    Settings.svelte:112-117: SessionHandoffEntry declares {id, branch, ts}
      — backend returns {seq, ts_ms, winner, abandoned, trunk_snap_id,
      from_snap_count}. Renders ? rows even on non-empty data.
    Settings.svelte:419-421: passes {projectId: ""} to a command whose
      param is Option<usize>. Silently ignored.
    SlashPalette /continue: labelled "resume a previous handoff" but
      ChatWindow.svelte:344-376 has no CONTINUE case. Misleading affordance.
  Fix: shared TypeScript bindings generated from Rust types (e.g. via ts-rs).

SHAPE F — BACKEND HARDCODES MOCK DATA
  Backend writes real SSB rows but with hardcoded constant fields. Examples:
    task_runner.rs:3985-4004: emit_reasoning hardcodes
      agent_id="task_runner", topic="task_runner", budget_remaining=0,
      grounding_level="medium" literal, confidence=0.9 literal,
      elapsed_ms=0 literal
    task_runner.rs:4784: model_swap event hardcodes gpu_id=0,
      swap_duration_ms=0
    KnowledgeDrawer "languages" line: task_runner.rs:2818 hard-codes
      Vec::new() for languages on DkgReady — drawer always shows "–"
  Fix: pass real values, not constants.

SHAPE G — DUAL-PATH CONSUMERS
  CLI and dashboard read different paths. Examples:
    cli/main.rs:3752 + 5437 reads .elai/dkg.db
    dashboard reads .elai_cc/indexes/dkg.db (correct path via
      ELAI_DATA_DIR_NAME constant)
    Net: CLI sees empty pagerank/catrag/arag; dashboard sees real
      data only when IndexingService has run
  Fix: unify on .elai_cc/indexes/ everywhere; delete .elai/ literal fallbacks.


================================================================================
DEAD UI INVENTORY (display X but X has no producer)
================================================================================

These render boxes / commands exist in the dashboard but have nothing real
to show. The user sees empty/zero/stub data and concludes the engine is broken.
Each one needs either a producer wire OR removal.

  1. KnowledgeDrawer "languages" line — always "–"
  2. KnowledgeDrawer pagerank section — always empty (stub command)
  3. KnowledgeDrawer query tier stats — always zero
  4. KnowledgeDrawer freshness panel — always known:false
  5. ActivityEntry knowledge_degradation case — no producer
  6. AntiStalenessDrawer — three permanent "no data yet" lines
  7. AntiStalenessDrawer dep_audit / baseline_harness / deterministic_audit
  8. ReasoningDrawer GRC budgets — synthetic data from emit_reasoning hardcoded
  9. ReasoningDrawer label "reasoning · N topics" — misleading (shows task_runner only)
  10. RulesDrawer — no live update; no rule_transitions echo
  11. PlanWindow.svelte — fully built, unregistered (PLAN routes elsewhere)
  12. get_decomposition_tree — stub
  13. get_phase_detail — stub
  14. get_baby_step_detail — stub
  15. get_research_budget — stub
  16. get_source_tier_distribution — stub
  17. Workspace PlanTree — only PhaseComplete, no concept/step/babystep layers
  18. Settings SessionHandoffEntry — schema mismatch, renders ? rows
  19. ActivityEntry branch_merge — no production writer
  20. ActivityEntry isolation · metric — no production emit
  21. StatusBar FSM badge — defaults to "IDLE" forever (no production writer)
  22. StatusBar 10-state stateColour/shouldPulse switch — dead code
  23. EXECUTING pulse animation — never fires
  24. MonitorWindow.currently — never fires
  25. /continue slash command label — "resume a previous handoff" with no handler
  26. Branch operations (merge/close) — no caller anywhere
  27. /session fork — works in CLI, NOT in dashboard slash palette
  28. Settings reindex button — emits Finding "reindex queued" but doesn't reindex
  29. AutoAdvanceStats panel — built before Phase 4G done
  30. ProposalsPanel (Phase 4H) — built before Phase 4H done


================================================================================
INVISIBLE PRODUCER INVENTORY (rich backend, no UI consumer)
================================================================================

These backend signals are real but the user has no way to see them. They are
what the user is most likely "missing" when complaining the dashboard doesn't
"feel like" the design.

KNOWLEDGE (high-priority surfaces):
  - Live QueryTrace inspector (per-query stage timeline + latency)
  - SearchMode cascade indicator + transitions log
  - Per-model EmbeddingCircuitBreaker state grid
  - HyDE hypothesis preview + cache hit rate
  - Per-tier Recall/MRR badges (real measured numbers exist)
  - VectorCache hit rate
  - DKG: failed_files list, edge-type breakdown, signatures_populated %
  - Index format version + UpgradeAction status
  - Canary drift cosine

REASONING (high-priority surfaces):
  - Per-agent GRC budget remaining gauge (real per role, not synthetic)
  - GroundingSweep result panel (claims classified GROUNDED/VERIFY_NEEDED/UNGROUNDED)
  - CSQ guard prompts (Critical pending guard)
  - Inline feasibility Pass/Fail/Blocked badges
  - Anti-rubber-stamp count (auditor findings per session)
  - Per-agent role config viewer (shows the divergent table bug visually)

ORCHESTRATION:
  - FSM state badge (LIVE — once the 30-min fix lands)
  - 7 EvidenceGate verdict panel (which gates passed/failed per transition)
  - InteractiveGate prompt UI (Approve/Reject/Modify/Redirect)
  - ReplanRequest log + severity
  - Per-agent panel (status, lifecycle, death classification)
  - Heartbeat liveness indicator
  - SupervisionManager event log
  - BSP superstep counter + parallel-agent run progress
  - ContextBudget per-slot allocation visualizer (token bar chart)
  - AdaptiveLevel events log

RULES + PLAN:
  - Per-rule drilldown (FPR/TPR/CCR, predicate type, evaluations)
  - Rule transitions log (PROPOSED → CANDIDATE → SHADOW → ACTIVE)
  - Predicate type filter
  - deprecated_patterns aggregator panel
  - PlanSkeleton tree visualizer (DAG with phase nodes + contracts + deps)
  - PlanSkeleton stub_reference markers
  - Coherence check violations panel

SESSION:
  - Session DAG branch tree visualizer
  - Snapshot history per branch
  - Handoff trigger threshold gauge (current % of token budget)
  - Stub recovery success rate
  - /session fork in dashboard slash palette
  - Branch merge/close UI

INFERENCE (3J):
  - Live token streaming in dashboard chat
  - vLLM /v1/models advertised list
  - DoLa per-role toggle UI + alpha knob
  - Per-step inference latency + tokens-per-sec gauge
  - Prompt cache hit rate
  - KV cache stats
  - Real-time VRAM gauge (not one-shot snapshot)
  - WorkloadProfile per agent per phase
  - MILP scheduling decisions log
  - AppContainer isolation runtime metrics
  - call_mcp_tool button per discovered tool

TOOLS / HOOKS / ISOLATION:
  - Sandbox tier (A/B/C) badge per agent
  - IPC frame stats (Tier C)
  - SO_PEERCRED + AgentHeartbeat liveness
  - ToolFilter rejections log
  - IntegrationTestRunner per-level pass results (currently failures only)
  - Edit-matching tier discriminant (exact/whitespace/ellipsis hit type)
  - Failure taxonomy histogram
  - Per-event hook axis (split PreToolUse/PostToolUse/Stop/PreCompact)
  - T1/T2/T3 hook latency histograms
  - ShortCoder reduction percentage gauge
  - File outline trigger reason

PIPELINE FACTORY:
  - DecompositionTree visualizer (Concept → Phase → Step → BabyStep)
  - PackageRef.verified anti-hallucination gate badges
  - TechStackDecision (RESEARCH/EXISTING_PROJECT/USER_SPECIFIED/FALLBACK)
  - Designer alternatives panel
  - Contradiction set log
  - SourceCitation list per fact
  - FactExtraction quotes panel
  - ResearchCache hit rate
  - ConvergenceStatus aggregation
  - ExecutablePipeline DAG visualizer
  - DAG repair events log (Reorder/Substitute distribution + T_DAG counter)
  - Phase Sizing splits indicator
  - TreeNavigatorState position cursor
  - FailureRecord typed log per baby step
  - BubbleUpEvent typed structure (currently debug-string only)
  - IntegrationCheckpoint per-level cascade counters
  - Cascade hierarchy visualization (BabyStep → Step → Phase → Concept)
  - Web research per-node FORMULATE/SEARCH/EXTRACT/REFLECT/CONVERGE log
  - Source quality tier per URL inline


================================================================================
DASHBOARD VISUAL DRIFT (root cause of "doesn't feel like wanted")
================================================================================

Even where wiring works, the visual identity drifted from dashboard-design.txt:

  - Body font: Inter (via --font-ui), spec mandates JetBrains Mono throughout
  - Type scale: 11/13/14/16/20px vs spec 10/12/13/15/18px (UI ~25% too large)
  - bg-void: #0b0d12 vs spec #0a0e12; text colors lighter than spec
  - Line height looser than spec's 1.6 text / 1.4 code
  - Two competing shells:
      Tier-0 Shell (sidebar + dual-PanelHost) is what the user lands on
      Spec MVP Workspace exists but reachable only as the "PLAN window"
        inside Shell
  - Activity feed buried under autonomy strip + feasibility panel +
    violations strip + proposals panel + intervention cards before reaching it
  - Activity feed missing: auto-scroll, Jump-to-Latest pill, inline [preview]
    entry with teal link
  - Status bar lives in Shell, not in Workspace as spec required
  - DiffViewer per-hunk hover controls and multi-file vertical nav thin
  - ChatWindow does NOT stream LLM output; only writes chat_message intent
    rows. Real conversation runs only in CLI/REPL.

Restoring spec visual identity is BLOCKED until the Tier-0 Shell decision is
revisited — Group-C drawers/windows were built before the underlying wiring,
so the Tier-0 Shell now has 10 drawers and 9 windows that displaced the
spec's tighter design.


================================================================================
WHAT THIS MEANS FOR THE B-PLAN
================================================================================

The wiring deficit dwarfs the implementation deficit. The B-plan must
prioritize:

  1. THE 30-MIN FSM FIX — first patch out (B-1.0). Activates 4 UI elements.

  2. KILL THE STUB COMMANDS — implement or remove. Each stub command is a
     lie to the user. Better to delete than render zero.

  3. MAKE THE PRODUCERS EMIT — for every "INVISIBLE PRODUCER" in the list
     above, add SsbActionType + emit point + Tauri command + Svelte
     consumer. This is mechanical work but each row delivers visible
     progress.

  4. UNIFY THE DUAL PATHS — single .elai_cc/ everywhere; one Hook
     dispatcher; one Tool dispatcher; one Session persistor. Each
     dual-path is a future drift event.

  5. SHARED TYPE BINDINGS — generate TypeScript types from Rust via ts-rs
     to prevent SHAPE E schema mismatches.

  6. RESTORE THE DESIGN SYSTEM — JetBrains Mono, 10/12/13/15/18, #0a0e12,
     decision on Tier-0 Shell vs spec Workspace. This is one B-track.

  7. WIRING_CONTRACT IN EVERY B-STEP — per Layer 0 of testing-trust, every
     B-step must declare producer/consumer file paths. No new types
     unless the consumer call site is also added in the same B-step.

  8. ACTIVATE THE GATES — many gates exist (GroundingSweepGate,
     anti-rubber-stamp, CSQ, feasibility, SupervisionManager,
     CliInteractiveGate, SearchMode cascade) but vacuously pass / never
     fire. B-plan must include "producer wiring" steps that feed the
     gates real signals.

The system is much more nearly complete than it appears, but the dashboard
makes it look ~10% done. Every B-N step that adds a producer/consumer pair
will visibly close that gap.
