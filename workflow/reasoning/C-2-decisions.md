C-2 Decisions
=============

Phase: C-2 SSB Emission Pass 2 (orchestration + cosmos + ambiguity + stagnation + anti-spiral)
Session: s-32952
Schema bump: 47 -> 48 (atomic, 10 new variants)

What

- Added 10 SsbActionType variants + 10 typed payload structs in
  orchestrator/src/ssb.rs:
    PhaseNodeReady, StepNodeReady, BabyStepReady, DagRepairEscalated,
    AgentStatusChanged, StagnationSpinningDetected,
    StagnationOscillationDetected, LateralThinkingActivated,
    GoalDriftWarning, WatchdogTimeout.
- Full 9-tuple lockstep registration touched in one atomic patch:
    (1) SQL CHECK list, (2) enum decl, (3) as_sql_str arms,
    (4) ssb_action_type_from_str arms, (5) SsbPayload enum,
    (6) SsbPayload::action_type arms, (7) test_payload arms,
    (8) all_test_payloads literals, (9) count assert 90 -> 100.
- SCHEMA_VERSION bumped 47 -> 48 with full descriptive comment block
  identifying each new variant + its production wiring site (where
  applicable) or C-N deferred wiring (5 anti-spiral variants are
  definition-only here per plan §Out-of-scope; producer logic lands
  in C-10).
- pipeline_builder.rs: extended DagRepairEmitter trait with
  `emit_escalated(rounds, remaining_issues, last_error)` default-no-op
  method (additive — keeps existing emitters source-compatible).
  Implemented on SsbDagRepairEmitter. Called at the
  RepairOutcome::Escalated construction site in PipelineBuilder::build
  (L411-418) immediately before returning PipelineBuilderError::
  RepairEscalated. Fire-and-forget via tokio::spawn; repair loop is
  never blocked on SSB I/O.
- bsp/engine.rs: co-emit AgentStatusChanged at every existing
  AgentLifecycle write site (7 sites: timeout-exit, clean-exit,
  deferred clean-exit, deferred timeout, spawned, JoinSet crash x2).
  Added module-level `emit_agent_status_changed` helper that sources
  `timestamp_ms` from the engine's ClockProvider (never SystemTime::now)
  per ENTROPY_QUARANTINE. Imported AgentStatusChangedPayload.
- task_runner.rs: added PhaseNodeReady / StepNodeReady / BabyStepReady
  emission inside the baby-step loop (L4736 area). The navigator has
  no on_node_entered callback (DRIFT-6 in context-package-C2.md), so
  boundary detection compares `tree.location_index[bs_id]` against
  `last_location` for phase/step boundary changes. BabyStepReady fires
  every iteration; PhaseNodeReady/StepNodeReady fire on boundary cross.
  Label fields populated via tree.find_step_for_baby_step / tree.find_baby_step.
- task_runner.rs: added new pub helper `trigger_cosmos_refresh(writer,
  worker)` that fires once per task entry (when the caller opts in),
  iterates the manifest, calls CosmosWorker::refresh_one, and emits
  one CosmosRefreshStarted row + one CosmosArtifactUpdated row per
  successful refresh. Unblocks the ACCEPTANCE smoke item (a) (>=1
  cosmos row on a live run) without waiting for C-16's spawn_loop.
- task_runner.rs: wired AmbiguityScorer.score() before
  gates_for_transition (L4509 area). When `resolved.llm_provider` is
  Some, the runner constructs a scorer with the active backend model,
  calls .score() with serialized plan_text + grounding summary +
  brownfield=false + empty deferrals, populates TransitionContext.
  ambiguity_score with the result, and emits one of three SSB rows:
  AmbiguityScored (always), AmbiguityFloorFailed (when failures
  present), AmbiguityStreakAdvanced (when overall_score <= threshold
  and no floor breaches). Score errors degrade to B-17 behaviour (gate
  treats ambiguity_score=None as the previous zero-ungrounded->ALLOW
  path per evidence.rs:256).
- knowledge/src/cosmos_worker.rs: promoted `refresh_one` from `fn`
  to `pub fn`; added `pub fn now_secs_clock()` accessor so external
  callers (orchestrator trigger_cosmos_refresh helper) can pass a
  consistent `now` value without reaching into the private clock field.
- knowledge/src/lib.rs: re-exported CosmosWorker, CosmosArtifact,
  CosmosClock, CosmosError, CosmosManifest, ResearchDispatcher so the
  orchestrator can construct a worker without a knowledge-internal
  path.
- reasoning/src/verification.rs: added new optional callback
  `cosmos_grounding_callback: Option<Arc<dyn Fn(&str, &str, &str, f32,
  bool) + Send + Sync>>` + builder `with_cosmos_grounding_callback`.
  Fires inside VerificationMethod::CosmosCheck dispatch (L386-394
  area) after the lookup result is known with (artifact_name,
  claim_category, query, score, matched). Orchestrator wires this to
  CosmosWorker::record_grounding_used so the cosmos_grounding_used
  SSB row fires on a live cloud run even before C-16's tick loop
  lands (resolves DRIFT-3).
- orchestrator/src/ssb_viewer.rs: added 4 new read methods —
  `recent_agent_status_changes(limit)` (latest-per-agent dedup),
  `latest_ambiguity_scored()`, `latest_ambiguity_floor_failed()`,
  `latest_ambiguity_streak()`.
- dashboard-v2/src-tauri/src/commands.rs:
    `get_agents` (L1776) — replaced hardcoded 8-IDLE block with a live
    SSB read via viewer.recent_agent_status_changes(). Cold-start
    fallback returns 8 canonical roles in IDLE so the AGENTS section
    never empty-renders (per stub_ban interpretation: empty-state
    fallback INSIDE the live read path is acceptable).
    `get_ambiguity_state` (L6682) — replaced mock body with live
    SsbViewer reads (latest_ambiguity_scored + latest_ambiguity_
    floor_failed + latest_ambiguity_streak). Cold-start returns the
    same shape with zero score + empty floor_failures + 0/required
    streak so GatesPanel can render an "awaiting data" layout.
    Threshold + weights + floors sourced from elai_reasoning re-exports.
    Removed `mock: true` field from the response body.
- dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte: inserted
  new AMBIGUITY section between EVIDENCE GATES (L160-174) and AGENTS
  (L176-191). Layout mirrors GatesPanel.svelte:91-148 (overall score +
  component table + floor warning + streak). Live-refreshes every 5s
  via invoke('get_ambiguity_state'). Path was corrected per DRIFT-4
  (GatesPanel.svelte lives in lib/components/, not lib/panels/).
- dashboard-v2/src/lib/api/types.ts: marked `mock` on AmbiguityState
  as optional (legacy field; backend stops emitting it in C-2+).
- contracts/tests/c2_ssb_new_variants_contract.rs: 14 contract tests
  covering (a) SCHEMA_VERSION_PUBLIC >= 48, (b) all 10 action-type
  string serializations, (c) round-trip through write_payload +
  SsbViewer::get_action_detail for each of the 10 new payloads, (d)
  recent_agent_status_changes dedup-per-agent invariant, (e) all 3
  ambiguity latest-* viewer helpers round-trip. All 14 pass.

Key Decisions

1. DRIFT-1 (line-number drift in plan): used the REAL line numbers
   from context-package-C2.md (`get_agents` at L1776, `get_ambiguity_state`
   at L6682, etc.) rather than the stale plan line numbers (1659 /
   6561).

2. DRIFT-2 (bubble_up_event already shipped): documented as a no-op.
   The plan implies a missing `bubble_up_event` SsbActionType but
   BubbleUpTyped already exists at ssb.rs and is wired via
   SsbBubbleUpEmitter at pipeline_tree_navigator.rs:1602-1622. The
   real C-2 gap is the FOUR NEW node-entry variants (PhaseNodeReady /
   StepNodeReady / BabyStepReady / DagRepairEscalated), which are
   what we added.

3. DRIFT-3 (cosmos producer wiring): chose the "additive callback"
   pattern for verification.rs rather than widening the cosmos_lookup
   signature. Rationale: (a) the existing cosmos_lookup is referenced
   from `cosmos_grounding_contract.rs` and other contracts — widening
   the closure breaks those tests; (b) a separate callback is
   strictly additive and keeps the lookup surface focused on its
   boolean role; (c) the orchestrator owns the CosmosWorker and can
   wire the callback when constructing the LocalVerifier without
   touching the reasoning crate's API surface. Also added the
   trigger_cosmos_refresh helper so the cosmos_refresh_started /
   cosmos_artifact_updated rows fire on a live run even without
   C-16's spawn_loop landing.

4. DRIFT-5 (AgentStatusChangedPayload field shape): dropped
   old_lifecycle from the payload (single `state` field). Rationale:
   (a) mirrors the existing AgentLifecyclePayload shape so the two
   streams are consistent; (b) avoids the per-agent last-state map
   that would need threading through BspEngine; (c) consumers derive
   transitions by reading the ordered stream per agent_id, which is
   what the dashboard already does for AgentLifecycle. The contract
   test `agent_status_changed_payload_round_trips_single_state_field`
   asserts old_lifecycle is NULL in the serialized JSON.

5. DRIFT-6 (PhaseNodeReady/StepNodeReady/BabyStepReady emit
   location): emits live in task_runner.rs baby-step loop (L4736
   area), NOT inside pipeline_tree_navigator.rs. The navigator's
   TreeNavigator trait has no on_node_entered method; adding one
   would touch 4+ test files. Instead we compare
   `tree.location_index[bs_id]` against `last_location` (which the
   loop already maintains) and emit on boundary changes. The emit
   fires BEFORE the existing emit_reasoning call so the dashboard
   renders the tree in depth-first order.

6. DRIFT-7 (DagRepairEscalated emit site): extended the
   DagRepairEmitter trait with a default-no-op `emit_escalated`
   method (chose option a from context package — "widen the emitter
   trait"). Rationale: (a) keeps the dependency direction intact
   (PipelineBuilder still talks only to the emitter trait); (b)
   matches the existing fire-and-forget pattern; (c) default impl
   means downstream emitter implementations (e.g. in tests) do not
   need to change unless they want to observe escalations.
   Implemented on SsbDagRepairEmitter; called at L411-418 (caller
   path) which is the ONLY production exit from the repair loop
   that surfaces RepairOutcome::Escalated.

7. DRIFT-8/9/10: skipped per orchestrator instruction; the cosmos
   and ambiguity payloads already existed (no new payload structs
   added for those types). C-2 added zero stagnation logic — only
   variant definitions per plan §Out-of-scope.

8. Cosmos viewer constants for the dashboard: sourced AMBIGUITY_THRESHOLD,
   AUTO_COMPLETE_STREAK_REQUIRED, GOAL_CLARITY_FLOOR, etc. directly
   from elai_reasoning (the dashboard-v2 src-tauri Cargo.toml
   already depended on elai-reasoning). Avoids duplicating the
   constants in ssb.rs or commands.rs.

Schema Version Bump
- SCHEMA_VERSION: 47 -> 48 (atomic for all 10 new variants).
- SCHEMA_VERSION_PUBLIC re-exports auto-tracks.
- B-34 contract floor (>=42) preserved (additive only).
- all_test_payloads count bumped 90 -> 100 (+ 10 C-2 variants).

Test Results
- Contract test: 14/14 PASS (c2_ssb_new_variants_contract).
- Orchestrator unit tests: 27/27 PASS (ssb::tests), including
  test_typed_payloads (count assert) and test_all_action_types_insert
  (every SsbActionType arm exercised end-to-end).
- cargo check -p elai-orchestrator -p elai-knowledge -p elai-reasoning
  -p elai_cc_contracts -p elai-cli -p elai-dashboard-v2 -> 0 errors.
- npx svelte-check (dashboard-v2) -> 0 errors, 13 warnings (all
  pre-existing in unrelated files).

Out-of-Scope (deferred per plan)
- C-8: AmbiguityScorer interrogation-round integration (the scorer
  is wired into task_runner.rs but only fires the score-and-emit
  pass; interrogation rounds remain deferred).
- C-10: StagnationDetection / ThinkingPersona / GenerationWatchdog
  / GoalDriftCanary logic modules — variant DEFINITIONS shipped in
  C-2; producer logic ships in C-10.
- C-16: CosmosWorker spawn_loop tick wiring + CosmosStage in
  QueryPipeline. The trigger_cosmos_refresh helper is a stop-gap
  that lets the orchestrator fire one refresh per task entry; the
  long-running tick loop still lives behind C-16.
- C-3: get_phase_detail / get_baby_step_detail V2 Tauri commands;
  BubbleUp cascade arrows in PlanWindow.svelte.
- C-5: research_node_stage / package_ref_verified / tech_stack_decided
  SSB types.
