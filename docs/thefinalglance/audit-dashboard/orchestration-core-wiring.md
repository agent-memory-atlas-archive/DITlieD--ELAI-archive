# Orchestration Core Dashboard Wiring Audit
Date: 2026-04-25
Auditor stance: HOSTILE — every wiring claim verified against source.
Spec: F:\.ELAI_workflow\docs\thefinalglance\summary\phase-3-summary.md (3A–3E) + phase-4-summary.md §4K.
Backend: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\{lib.rs, commands.rs}
Frontend: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\{StatusBar.svelte, ActivityFeed.svelte, ActivityEntry.svelte, drawers\OrchestrationDrawer.svelte, windows\MonitorWindow.svelte, windows\PlanWindow.svelte}
SSB writers/readers: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\{ssb.rs, ssb_viewer.rs, fsm.rs, task_runner.rs, context_budget.rs, checkpoint.rs, anti_spiral.rs, supervision.rs, bsp\engine.rs, agent_pool.rs}
Inventory files NOT trusted: dashboard-wiring-inventory.txt, dashboard-wiring-plan.txt (per audit instructions and 4K audit §J).

Overall verdict: BROKEN. The orchestration core is the most under-wired
surface in the dashboard. Of the 5 phases:
- 3A FSM        : INVISIBLE in real time (StatusBar reads a row that no
                  production writer ever emits — see §3A below).
- 3B SSB        : the substrate IS wired (ActivityFeed and StatusBar both
                  poll get_recent_actions every 500ms / 1000ms). This is
                  the ONE surface that genuinely streams from the bus.
- 3C Agents     : DEAD. No spawn / lifecycle / death / heartbeat /
                  supervision surface anywhere. Anti-spiral hits exist as
                  ActivityFeed rows only (typed renderer present in
                  ActivityEntry). The Orchestration drawer's
                  "checkpoints" panel reads an SSB row that production
                  only emits from BSP-routed parallel phases — every
                  sequential phase produces zero rows there.
- 3D BSP        : INVISIBLE. No superstep counter, no channel state, no
                  parallel-agent run progress anywhere on screen.
- 3E Budget     : DOUBLE-DEAD. ContextReduction SSB rows have NO
                  production writer (only contract tests fire
                  adaptive_reduce). The drawer panel will read "no
                  context reductions recorded" forever. Per-slot token
                  allocation, ModelCapabilityProfile in use, and 3E.5
                  file_overlap_score are all absent from the dashboard.

The activity feed IS real — it pulls SSB tail every 500ms via
get_recent_actions (ActivityFeed.svelte:66, SsbViewer::recent_actions at
ssb_viewer.rs:819). It is NOT mocked. Whether useful orchestration rows
land in that tail depends on whether each phase actually writes its own
SSB rows in production — and the answer for 3A/3D/3E is mostly NO.

================================================================================

## Phase 3A — Full FSM (Interactive Mode)

Backend signals available (per phase-3A.md audit + source verification):
- Current FsmState (10 variants incl. Researching) — fsm.rs:24-38.
- TransitionRecord {source, target, phase_label, timestamp_ms, evidence:
  Vec<EvidenceArtifact>} — fsm.rs:161-167. Persisted to msgpack on
  disk only (fsm.rs:628-637 persist_state) — not to SSB.
- 7 EvidenceGate verdicts via gates_for_transition + check_transition —
  evidence.rs:483, 502-513.
- run_interactive_gate decisions: Approve/Reject/Modify/Redirect —
  gate.rs:227-293.
- RiskLevel default per transition (Low/Medium/High) — gate.rs:169-195.
- ReplanRequest enforcement (max-1 per agent per phase, severity) —
  fsm.rs:170-209, 642-667.

SSB writers (or lack thereof):
- SsbActionType::PhaseTransition — exists in the enum at ssb.rs:107 with
  PhaseTransitionPayload at ssb.rs:603. ZERO production writers. The
  only constructions are: (a) ssb.rs:2608 inside the `example_payload`
  test fixture, (b) ssb.rs:3823 inside another test fixture, (c)
  ssb.rs:4187 inside a proptest, (d) dst.rs:415 inside the Deterministic
  Simulation Testing reference model (test-only). Verified by
  `Grep PhaseTransitionPayload\\b|SsbActionType::PhaseTransition\\b`
  across the workspace.
- SsbActionType::FsmTransition (a separate, A-22 Researching variant) —
  ssb.rs:1376 FsmTransitionPayload. ONE production writer:
  task_runner.rs:2346 `emit_fsm_transition()`, called ONLY at web-research
  task entry/exit (task_runner.rs:2335 + 4 other call sites in
  drive_web_research_task). It does NOT cover the standard
  IDLE→PLANNING→…→COMPLETE chain.
- Fsm::transition (fsm.rs:517-640) ONLY persists to msgpack
  (`.fsm-state.msgpack` per-session file, fsm.rs:628-637). It does NOT
  call writer.write_payload(). I read the function in full.
- The CLI fsm_transition_sync helper (cli/main.rs:4245-4259) drives
  Fsm::transition; it also does not write SSB. The fsm_transition tool
  in tools/lib.rs:1506-1527 mutates only an in-memory fsm_state()
  Mutex<FsmStateGlobal> and never touches SSB either.

Dashboard wiring (consumers):
- StatusBar.svelte:111-129 polls `get_fsm_state` every 1000ms.
  get_fsm_state (commands.rs:67-71) → SsbViewer::latest_fsm_state
  (ssb_viewer.rs:733-765) — and that function reads
  `WHERE action_type = 'phase_transition' ORDER BY seq DESC LIMIT 1`,
  with the `Ok(None)` branch returning {state: "IDLE", phase: ""}.
  Because no production writer ever emits a PhaseTransition row, the
  query returns NoRows on every real session, so StatusBar renders the
  IDLE badge forever. The carefully-crafted stateColour() / stateLabel()
  / shouldPulse() switch (StatusBar.svelte:308-357) is dead in production.
- ActivityEntry.svelte does carry typed renderers for `phase_transition`
  (line 244, 1063) and `fsm_transition` (line 458, 1171). The
  fsm_transition row WILL render when the user enters/exits a Research
  task — that is the only FSM event surfaced via the activity feed today.
- Evidence-gate verdicts: NO Tauri command, NO drawer, NO ActivityEntry
  variant. EvidenceError / GateResult / EvidenceSummary are entirely
  invisible to the dashboard. There is no surface that says "gate
  KnowledgeReadinessGate failed: missing dkg_status; suggestion=run
  build-knowledge first" — the user sees only that the FSM did not
  advance. Fully INVISIBLE.
- InteractiveGate ceremony: same as above PLUS the audit at
  phase-3A.md:137-146 records that CliInteractiveGate is never
  instantiated in production. The "human pause / Approve / Reject /
  Modify / Redirect" 4-decision flow is library-only. The dashboard has
  no surface to display it either.
- Risk-level (Low/Medium/High) per transition: not surfaced anywhere.
- Replan-log: not surfaced.

Verdict: INVISIBLE for the standard FSM chain; PARTIAL only for the
RESEARCHING corridor. The single most prominent UI element on screen —
the StatusBar FSM badge that drives the EXECUTING-pulse animation — is
wired to a column the orchestrator never writes. Highest-impact dead
wire in the entire dashboard.

================================================================================

## Phase 3B — SSB (Shared State Board)

Backend signals available:
- 74 SsbActionType variants (ssb.rs:98-295), ~17 of them used directly
  by the dashboard renderer.
- Append-only WAL ssb table; read pool deadpool-sqlite; broadcast
  channel capacity 1024 (ssb.rs:2034). All real per phase-3B.md audit.
- Conflict detection via ssb_files JOIN — ssb.rs query_conflict (per
  phase-3B audit §3B.3). Not surfaced in the dashboard.
- Action-type stream: every variant typed+timestamped+ordered by seq.

Dashboard wiring:
- ActivityFeed.svelte:66 `invoke<SsbActionSummary[]>("get_recent_actions",
  { limit: 50, offset: 0 })`, polled every 500ms (POLL_MS=500). This is
  the genuine "live SSB tail" surface and it works. The query path:
  commands.rs:80-91 → SsbViewer::recent_actions
  (ssb_viewer.rs:819-856) → real SQL `SELECT seq, created_at,
  action_type, agent_id, payload FROM ssb ORDER BY seq DESC LIMIT ?1
  OFFSET ?2`. Not mocked.
- ActivityEntry.svelte renders typed branches per action_type — the
  contract test
  contracts/tests/activity_entry_coverage_contract.rs covers ~50 of the
  74 variants (line 39 lists 17 explicit + 5 added in Batch 4C, with
  more added per Tier 2/3). PASS for the variants that production
  actually emits.
- StatusBar.svelte:134-148 also polls get_recent_actions to drive the
  `indexing…` indicator and the active-profile badge. Same SSB tail.
- get_action_detail (commands.rs:338) returns a single row's full
  payload, used by DiffViewer for InterventionRequest rows. Real
  end-to-end via SsbViewer::get_action_detail (per 4K audit).
- Conflict detection (3B.3 invariant): NOT surfaced. No drawer reads
  query_conflict; no ActivityEntry variant flags conflicts. The
  invariant runs at write time and any violation manifests as an
  SsbError, never as a dashboard event.
- 50K inserts/sec performance ceiling: NEVER measured per phase-3B.md
  TL;DR. No dashboard widget shows actual SSB throughput either.

Verdict: SURFACED (substrate-level). The single bright spot in this
audit. Activity feed and StatusBar BOTH genuinely stream from SSB
every 500–1000 ms. What the user sees there depends entirely on what
upstream phases choose to write — and §§ 3A/3D/3E below show that the
orchestration-core writes are largely missing.

================================================================================

## Phase 3C — Agent System

Backend signals available:
- AgentRole (8 variants: Executor, Auditor, Researcher, Designer,
  PromptEngineer, ContextCompiler, MemoryWriter, MediaValidator) —
  ipc-types/lib.rs:17-28.
- AgentStatus (Spawning/Ready/Running/Checkpointing/Finished/Failed) —
  agent_pool.rs:30-39.
- AgentDeathClassification (Crash/CleanExit/Timeout/BudgetExceeded) —
  recovery.rs:18-27.
- Heartbeat: 5s interval, 15s timeout — ipc-types/lib.rs:45-49.
- AgentCheckpoint state-hash + file-sizes per spawn — ssb.rs:585
  CheckpointPayload. Producer: checkpoint.rs:243 (production caller
  list verified — see writers below) AND bsp/engine.rs:559.
- SpiralDetected payload — ssb.rs:707 SpiralDetectedPayload, written
  from task_runner.rs:4124 (REAL production writer, anti-spiral path).
- SupervisionExhausted payload — ssb.rs:697 SupervisionExhaustedPayload.
  ZERO production writers (only proptest fixture at ssb.rs:2653 and
  test fixture at ssb.rs:3895).

SSB writers verified (Grep for `*Payload \\{` and `SsbPayload::*`):
- CheckpointManager::save_checkpoint (checkpoint.rs:243) — the
  CheckpointManager is constructed ONLY in contract tests and unit
  tests (verified by `grep -rn "CheckpointManager::new"` — every hit
  is under `tests/` or under `#[cfg(test)] mod tests`). Zero
  production callers. So this code path is dead at runtime.
- bsp/engine.rs:559 save_superstep_checkpoint — fires inside BspEngine
  loop. BspEngine IS instantiated in production at task_runner.rs:2484,
  but ONLY for phases marked is_parallel = true. Sequential phases (the
  default path) never go through BSP, so no Checkpoint row is emitted
  for them.
- SupervisionManager (supervision.rs) is fully implemented but never
  wired into task_runner — verified at phase-3C.md §3C.5 FAIL and
  re-verified by `grep SupervisionManager` returning only the lib.rs
  re-export at line 467. No SupervisionExhausted SSB row will ever be
  written from the standard sequential agent path.
- SpiralDetected: writes from task_runner.rs:4124. REAL.

Dashboard wiring:
- OrchestrationDrawer.svelte:53 `invoke<AgentCheckpointEntry[]>(
  "get_agent_checkpoints", { limit: 10 })`. Backend:
  commands.rs:3688-3696 → SsbViewer::agent_checkpoints
  (ssb_viewer.rs:2284-2329) — real SQL `WHERE action_type =
  'checkpoint' ORDER BY seq DESC LIMIT ?2`. Will return rows ONLY for
  parallel BSP phases. For the sequential default flow this returns
  []. The drawer renders nothing in that case (the
  `{#if checkpoints && checkpoints.length > 0}` guard at line 103 hides
  the panel) — the user does not even see the empty header.
- Spawned agents / lifecycle status / death classification: NO Tauri
  command, NO Svelte component anywhere. agent_pool / recovery /
  supervision events are entirely INVISIBLE. The
  OrchestrationDrawer was originally specced to include
  get_bsp_superstep_log and get_agent_heartbeats, but the lib.rs
  comment at line 1505 explicitly notes "get_bsp_superstep_log +
  get_agent_heartbeats dropped as ghosts". Confirmed by absence from
  the invoke_handler list.
- Anti-spiral (SpiralDetected): exists as an ActivityEntry variant at
  ActivityEntry.svelte:217 (`case "spiral_detected":`) and at
  line 1094 (`case "spiral_detected":`). Real wire — SpiralDetected
  rows from task_runner.rs:4124 will appear in the activity feed
  with a typed renderer.
- AgentDeathClassification + heartbeat-based Timeout vs Crash: per
  phase-3C.md §3C.2 F3 the Timeout path is collapsed into Crash and
  the IPC event loop that converts heartbeat gaps into Timeout is
  absent. So even if a death classification surface existed, it would
  only ever observe Crash/CleanExit. Today it observes nothing.
- Invisible-walls leakage (phase-3C.md §3C.2 F1): the
  visibility_filtered error_code distinguishes hidden from
  nonexistent. The dashboard has no surface to render this either way
  — it is a pure tools-layer concern. Not relevant for dashboard
  wiring per se but it means the agent layer is broadcasting an R31
  violation to LLMs invisibly.

Verdict: PARTIAL → mostly DEAD. The Orchestration drawer's checkpoints
list is plumbed end-to-end but production rarely writes the rows. The
only Phase 3C signal that genuinely flows into the dashboard at runtime
is SpiralDetected via the activity feed. AgentRole / status / death
class / heartbeat / supervision are all INVISIBLE.

================================================================================

## Phase 3D — BSP Supersteps + Channels

Backend signals available:
- BspEngine superstep counter (u64, monotonic) — bsp/engine.rs:138-145
  inside BspCheckpoint.
- 4 channel types (LastValue, Topic, EphemeralValue, UntrackedValue) —
  bsp/channels.rs:205-225. ChannelRegistry trait at lines 324-396.
- ChannelState (3 variants: LastValue, Topic, Ephemeral —
  UntrackedValue intentionally excluded for determinism) —
  bsp/channels.rs:192-197.
- save_superstep_checkpoint (bsp/engine.rs:606) — msgpack atomic
  write to {checkpoint_dir}/superstep_{N}.msgpack.
- Memory monitoring via MemoryMonitoringPayload — ssb.rs:670
  MemoryMonitoringPayload, written from bsp/engine.rs:314.
- Channel-write conflict / accumulate-flag conflict / TIMEOUT
  classification per superstep.

SSB writers (production):
- bsp/engine.rs:314 emits MemoryMonitoring rows.
- bsp/engine.rs:559 emits Checkpoint rows.
- BspEngine constructed in production at task_runner.rs:2484 —
  ONLY for is_parallel=true phases. Sequential phases bypass BSP entirely.

Dashboard wiring:
- Superstep boundaries / counter: NO Tauri command, NO Svelte
  consumer. The dashboard cannot tell you which superstep is running,
  how many have completed, or whether the engine is in an attempt-retry
  loop. INVISIBLE.
- Channel state: NO command, NO consumer. Channel writes are a
  documented stub at bsp/engine.rs:245-247 (apply_writes is a placeholder
  per phase-3D.md KNOWN GAP) so even if the dashboard surfaced channel
  state, it would render the same empty-state for every superstep.
- Parallel agent execution / JoinSet straggler timeout / TIMEOUT
  classification: NO surface.
- Memory monitoring: ActivityEntry.svelte:383 has `case
  "memory_monitoring":` — typed renderer present. Will render in the
  activity feed when bsp/engine.rs:314 fires. PARTIAL surface only —
  no aggregate "channel X has Y bytes" panel.
- Checkpoint via OrchestrationDrawer (see §3C above) — fires for BSP
  phases only.
- ContextReduction (memory-monitoring uses untracked-channel deep_size
  to fire context reductions — see 3E below): NEVER fires in production
  because adaptive_reduce has no production caller (verified — see §3E
  below).

Verdict: INVISIBLE. The BSP engine runs (for parallel phases only),
emits Checkpoint and MemoryMonitoring rows, and the dashboard renders
those generically via ActivityEntry. There is no superstep panel, no
channel inspector, no JoinSet status, no parallel-execution progress
indicator. The dashboard cannot tell that BSP is doing anything beyond
"some checkpoint rows showed up".

================================================================================

## Phase 3E — Context Budget Allocator

Backend signals available:
- ContextBudget {model_max_context, slots: [SlotAllocation; 8],
  cache_breakpoints: [CacheBreakpoint; 2], total_allocated, total_used,
  adaptive_level} — context_budget.rs:563.
- 8 ContextSlot variants (SystemPrompt, Rules, PhaseSpec, RepoMap,
  CodeContext, SsbEntries, GrcHistory, OutputReserve) —
  context_budget.rs:509-518.
- AdaptiveLevel (Normal, FilteredByRelevance, ReducedGranularity,
  ReducedScope, Terminal) — context_budget.rs:522-528.
- ContextReduction SSB row {level, before_tokens, after_tokens,
  slots_affected} — ssb.rs:714 ContextReductionPayload.
- ModelCapabilityProfile.fixed_slot_targets / slot_weights /
  attention_ceiling — A-10 plumbing per phase-3E.md §A-10.
- file_overlap_score (3E.5): MISSING ENTIRELY — phase-3E.md §1
  confirms no function named file_overlap_score / compute_overlap /
  filter_ssb anywhere in the orchestrator.

SSB writers (production):
- adaptive_reduce (context_budget.rs:961) writes ContextReduction at
  line 985. Verified callers via `grep -rn "adaptive_reduce"`: every
  hit is in `contracts/tests/context_budget_contract.rs`. ZERO
  production callers. The reduction loop is library-only. So
  ContextReduction SSB rows are NEVER written in production.
- ContextBudget::compute is called from dispatcher.rs:221-222. Per
  phase-3E.md §3J.WIRE.BUDGET, the one production call site in
  task_runner.rs:2992-2999 passes `None` for the
  ModelCapabilityProfile parameter, so dispatch always falls through
  to compute(raw_max, [0,0,0], 4096). The A-10 profile machinery is
  plumbed but receives None at runtime. Dead at runtime.

Dashboard wiring:
- OrchestrationDrawer.svelte:52 `invoke<ContextBudgetSnapshot>(
  "get_context_budget_snapshot")`. Backend: commands.rs:3680-3685 →
  SsbViewer::context_budget_snapshot (ssb_viewer.rs:2230-2279) — real
  SQL `WHERE action_type = 'context_reduction' ORDER BY seq DESC LIMIT 1`.
  Returns {known: false} when no row exists. Because no production
  writer ever fires, the drawer always renders the
  `<p class="ghost">no context reductions recorded</p>` branch
  (OrchestrationDrawer.svelte:100). DEAD wire end-to-end.
- Per-slot token allocation (the 8 ContextSlot allocations): NO
  command, NO consumer. The 8-slot budget split is invisible.
- ModelCapabilityProfile in use (which profile drives the current
  dispatch): NO surface. Even if a profile WERE in use, the dashboard
  has no widget that shows it.
- Adaptive reduction events live: would render via ActivityEntry's
  `case "context_reduction":` (line 381) IF any row were ever
  written. None ever is. The typed renderer is dormant.
- file_overlap_score (3E.5): the dashboard cannot surface what does
  not exist. The drawer's `slots_affected` field would be the natural
  display — but slots_affected is only populated by
  apply_reduction_level (context_budget.rs:975), which is called only
  from the dead adaptive_reduce path.

Verdict: DEAD. Two layers of breakage stacked: (1) the producer
(adaptive_reduce) has no production caller, and (2) the orchestration
drawer panel that consumes its output is live and waiting. The user
sees "no context reductions recorded" no matter how big the prompt
gets. The 8-slot allocation, A-10 profile, and 3E.5 overlap scoring
are all invisible by separate failures.

================================================================================

## Cross-cutting wiring gaps

1. EvidenceGate verdict surface — MISSING. All 7 gates
   (KnowledgeReadiness, GroundingSweep, TestFailReasons, TestFlipped,
   ReplanFastTrack, ZeroBlocks, DkgChangelog) write Result<(),
   EvidenceError> objects with structured `missing_artifacts` and
   `suggestion` fields (evidence.rs:21-25). NONE of this reaches the
   dashboard. When a transition is blocked, the user sees the FSM
   "stuck" but cannot see which gate failed or what artifact is
   missing. This is the highest-actionability missing surface in the
   product.

2. InteractiveGate prompts (Approve/Reject/Modify/Redirect) — MISSING.
   Per phase-3A.md §3A.4 CONDITIONAL PASS, CliInteractiveGate is
   defined but never instantiated. There is no GUI equivalent either.
   The 4-decision ceremony has no dashboard surface — Reject and
   Modify decisions cannot be communicated through the UI.

3. ReplanRequest log — INVISIBLE. fsm.rs tracks max-1-per-phase
   ReplanRequests with severity (ReplanSeverity::Minor / Major). No
   command, no surface.

4. RiskLevel per transition — INVISIBLE. The MEDIUM/HIGH/LOW
   classification (gate.rs:169-195) drives the LOW auto-skip path but
   is never shown to the user.

5. AgentRole + AgentStatus — INVISIBLE. The user cannot see which
   agents are alive, what role they have, or what step_index they are
   on. The dashboard does have a `monitor` window with `currently:`
   text (MonitorWindow.svelte:112) but that field is populated from a
   string field on MonitorSnapshot — verified in commands.rs but the
   value originates from generic SSB summaries, not from a real
   AgentHandle list.

6. Heartbeat / death classification — INVISIBLE.

7. SupervisionManager events — INVISIBLE because the producer is
   never wired (phase-3C.md §3C.5 FAIL).

8. BSP superstep counter / channel state / JoinSet status —
   INVISIBLE.

9. ContextBudget per-slot allocation + AdaptiveLevel + active
   ModelCapabilityProfile — INVISIBLE.

10. file_overlap_score — DOES NOT EXIST in producer (3E.5). Cannot
    be surfaced.

11. PhaseTransition SSB writer — MISSING ENTIRELY. This is the single
    most consequential gap, because it disables the FSM badge that
    StatusBar prominently renders. Suggested fix: at the end of every
    Fsm::transition_with_metrics (fsm.rs:639) emit a PhaseTransition
    SSB row with from/to/evidence_summary. One ~10-line patch would
    light up StatusBar's stateColour/shouldPulse for the entire
    standard FSM chain.

12. Inventory file lies. dashboard-wiring-inventory.txt:147 lists
    file_overlap_score as "outstanding". The 4K audit §J already
    tagged the inventory as untrustworthy; this audit confirms it via
    independent grep.

================================================================================

## Recommended surfaces (priority order)

P0 — fix PhaseTransition writer.
- Producer patch: append `writer.write_payload("fsm", 0,
  SsbPayload::PhaseTransition(PhaseTransitionPayload { from_state,
  to_state, evidence_summary }), Vec::new())` inside
  Fsm::transition_with_metrics at fsm.rs:639. Wire the writer Arc<Ssb>
  into the Fsm builder. This single change activates StatusBar,
  MonitorWindow.currently, ActivityEntry's phase_transition renderer,
  PlanWindow phase progress, and the "EXECUTING -> teal pulse" badge
  in one shot.
- ~30 min patch. Highest visibility-per-LOC fix in the entire dashboard.

P1 — surface EvidenceGate verdicts.
- Add commands::get_recent_gate_verdicts returning the last N
  EvidenceError records. Producer: write a new SsbActionType::
  GateVerdict row inside Fsm::transition when a gate Err returns.
  Surface in OrchestrationDrawer or a new GatesDrawer with
  {gate_name, missing_artifacts, suggestion} per row. Without this
  the user cannot tell why a transition is blocked.

P2 — wire SupervisionManager into task_runner (closes 3C.5 FAIL).
- Once wired, SupervisionExhausted rows will fire from
  supervision.rs and render via the activity feed. No new dashboard
  command needed; ActivityEntry can carry a typed renderer.

P3 — wire CheckpointManager into the standard sequential agent path.
- At present, OrchestrationDrawer's checkpoints panel reads rows
  that only BSP-routed parallel phases produce. Hook
  CheckpointManager::save_checkpoint into the baby-step loop in
  task_runner so every sequential agent execution also produces
  Checkpoint rows.

P4 — surface ContextBudget per-slot allocation.
- New SsbActionType::BudgetAllocated emitted from
  ContextBudget::compute (or from dispatcher.rs:221-222 wrapper)
  with {slots: [SlotAllocation; 8], total_allocated,
  cache_breakpoints}. Drawer renders an 8-row table with
  allocated/floor/actual per slot. Independently, fix
  task_runner.rs:2992-2999 to pass a non-None
  ModelCapabilityProfile so A-10 actually drives dispatch.

P5 — wire adaptive_reduce.
- Find the dispatch entrypoints that build prompts and call
  ContextBudget::compute, then call .adaptive_reduce when
  total_used > model_max_context. This will start producing
  ContextReduction rows and the OrchestrationDrawer's currently-dead
  context-budget panel will light up.

P6 — surface BSP supersteps.
- New get_bsp_superstep_log (the inventory listed it as a "ghost
  command" but that is exactly the surface the Orchestration drawer
  needs). Backed by a new SsbActionType::SupersteoBoundary row
  emitted from bsp/engine.rs:354 (after gate passes, before
  checkpoint). Render counter + per-attempt status.

P7 — surface AgentRole/AgentStatus pool live state.
- In-memory snapshot from agent_pool.rs (no SSB needed). New
  command get_agent_pool_snapshot returning Vec<AgentHandle>.
  Render in MonitorWindow under a new "agents" section.

================================================================================

## Summary

(1) Path written:
    F:\.ELAI_workflow\docs\thefinalglance\audit-dashboard\
    orchestration-core-wiring.md

(2) Overall verdict: BROKEN. Of the 5 orchestration-core phases, only
    3B (SSB substrate) is genuinely surfaced. 3A is INVISIBLE because
    no production code writes the row StatusBar reads. 3C is mostly
    DEAD (only SpiralDetected reaches the activity feed). 3D is
    INVISIBLE (no superstep / channel / parallel-execution surface).
    3E is DOUBLE-DEAD (the producer adaptive_reduce has no production
    caller AND the consumer drawer panel is wired to an empty SSB
    query).

(3) Orchestration signals INVISIBLE in dashboard:
    - FSM state for the standard IDLE→PLANNING→…→COMPLETE chain
      (StatusBar.svelte:113 reads a row no one writes)
    - All 7 EvidenceGate verdicts (no command, no drawer)
    - InteractiveGate Approve/Reject/Modify/Redirect prompts
    - ReplanRequest log + severity
    - RiskLevel per transition
    - AgentRole / AgentStatus / death classification / heartbeat gaps
    - SupervisionManager events (producer never wired into task_runner)
    - BSP superstep counter, channel state, JoinSet status,
      parallel-agent run progress
    - ContextBudget per-slot allocation, AdaptiveLevel,
      ModelCapabilityProfile in use
    - Adaptive context reduction events (producer has no production
      caller)
    - file_overlap_score (3E.5 missing in source per phase-3E.md §1)
    - SsbViewer conflict-detection events

(4) Does the activity feed actually pull from SSB or is it mocked?
    REAL — not mocked. ActivityFeed.svelte:66 invokes
    `get_recent_actions` every 500ms via the Tauri bridge.
    commands.rs:80-91 calls SsbViewer::recent_actions
    (ssb_viewer.rs:819-856) which executes
    `SELECT seq, created_at, action_type, agent_id, payload FROM ssb
    ORDER BY seq DESC LIMIT ?1 OFFSET ?2` against the real WAL
    sqlite db opened read-only at ssb_viewer.rs:721. The same
    query backs StatusBar's indexing/profile detection
    (StatusBar.svelte:134) and the "Recent commits" strip via
    get_recent_commits (ActivityFeed.svelte:80). The substrate is
    sound. The reason the feed often looks sparse for orchestration
    events is that 3A/3D/3E producers fail to write the rows in the
    first place, NOT that the feed mocks them out.

(5) Is FSM state visible in real-time in status bar?
    NO. The StatusBar polls every 1000ms (StatusBar.svelte:363
    setInterval(refresh, 1000)), invokes get_fsm_state, which queries
    `WHERE action_type = 'phase_transition' ORDER BY seq DESC LIMIT 1`
    (ssb_viewer.rs:733-765). Because the only production writers of
    PhaseTransitionPayload are test fixtures (ssb.rs:2608, 3823, 4187)
    and the DST reference model (dst.rs:415), the query always returns
    NoRows in production and the badge defaults to "IDLE". The
    elaborate state-color/pulse switch (StatusBar.svelte:308-357)
    that handles all 10 FsmState variants is dead code at runtime.
    The ONLY FSM-related signal that does reach StatusBar is when a
    web-research task fires emit_fsm_transition (task_runner.rs:2346)
    — but that emits an `fsm_transition` row, which the StatusBar
    badge does not read (it only reads `phase_transition`); the
    activity feed shows the row via ActivityEntry's typed renderer
    instead. Net: the FSM state badge is functionally inert in
    production today.
