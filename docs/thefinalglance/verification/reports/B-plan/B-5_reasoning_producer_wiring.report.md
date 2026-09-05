# Verification Report: B-5_reasoning_producer_wiring.md
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-5_reasoning_producer_wiring.md
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary
B-5 targets replacement of synthetic emit_reasoning constants with real PipelineGrc values, plus 4-5 new SSB action types and 5 Tauri commands. The four named SSB variants (grc_step_emitted, grounding_sweep_result, csq_evaluated, feasibility_check, auditor_rubber_stamp_rejected) are entirely absent from SsbActionType. emit_reasoning still hardcodes the synthetic constants with a TODO comment referencing B-5 T3. Partial credit: 4 Tauri commands exist (get_grc_budgets, get_grounding_sweep, get_csq_pending, get_gg_signals) and are registered; the ReasoningDrawer is redesigned with budget gauges, sweep panel, and CSQ section. However, get_grounding_sweep and get_csq_pending return empty/hardcoded data, the intervention card CsqBlockCard.svelte is absent, anti-rubber-stamp SSB emit is absent, and B-5-decisions.md was never written.

## Claims verified DONE
- BasicGrcEngine exists in production code — evidence: crates/reasoning/src/engine.rs:73 (SCIP_DEF agent_config)
- PipelineGrc wrapper exists — evidence: crates/orchestrator/src/grc.rs:195 (engine() method)
- emit_reasoning function exists in task_runner.rs — evidence: crates/orchestrator/src/task_runner.rs:6228
- Tauri command get_grc_budgets defined and registered — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:1515; lib.rs:99
- Tauri command get_grounding_sweep defined and registered — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:1566; lib.rs:100
- Tauri command get_csq_pending defined and registered — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:1578; lib.rs:101
- Tauri command get_gg_signals defined and registered — evidence: crates/dashboard-v2/src-tauri/src/commands.rs:1583; lib.rs:102
- Stub functions get_grc_budgets, get_grounding_sweep, get_csq_pending, get_gg_signals present in stub.ts — evidence: crates/dashboard-v2/src/lib/api/stub.ts:549,557,565,573
- ReasoningDrawer.svelte exists in V2 location — evidence: crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte:1
- ReasoningDrawer has GRC budget gauge rows rendered via d.grcBudgets loop — evidence: ReasoningDrawer.svelte:164-183
- ReasoningDrawer has grounding sweep panel — evidence: ReasoningDrawer.svelte:186-215
- ReasoningDrawer has CSQ critical pending section — evidence: ReasoningDrawer.svelte:217-250
- ReasoningDrawer has feasibility panel — evidence: ReasoningDrawer.svelte:252-291
- Anti-rubber-stamp enforcement exists in engine.rs (line 329 check) — evidence: crates/reasoning/src/engine.rs:329
- AgentGrcConfig table exists in agent_pool.rs — evidence: crates/orchestrator/src/agent_pool.rs:63,707

## Claims PARTIAL
- Task 1: Real per-agent GRC emit in task_runner.rs
  - exists: crates/orchestrator/src/task_runner.rs:6228-6250 (emit_reasoning function)
  - gap: Still uses synthetic constants. Lines 6230-6240 hardcode agent_id=RUNNER_AGENT_ID ("task_runner"), topic="task_runner", grounding_level="medium", confidence=0.9, elapsed_ms=0, budget_remaining=0. The TODO at lines 6218-6227 describes the required refactor but it is not implemented. stub_ban condition (no hardcoded agent_id="task_runner") is NOT met.

- Task 3: Tauri command get_grc_budgets returns partially real data
  - exists: crates/dashboard-v2/src-tauri/src/commands.rs:1515-1562
  - gap: When SSB is empty it returns hardcoded spec-default JSON constants (lines 1522-1532). When SSB has data it reads ReasoningTrace rows via v.grc_budgets() — these rows use the synthetic "task_runner" agent_id, so real per-agent breakdowns remain synthetic until T1 is done.

- Task 3: get_grounding_sweep returns real Tauri command but stub data
  - exists: crates/dashboard-v2/src-tauri/src/commands.rs:1566-1575
  - gap: Returns hardcoded zeros ({grounded:0, verify_needed:0, ungrounded:0, claims:[]}). Comment at line 1567 explicitly acknowledges GroundingSweepGate producer is unwired. drawers.svelte.ts:230 marks unwired.groundingSweep=true.

- Task 3: get_csq_pending returns Tauri command but empty
  - exists: crates/dashboard-v2/src-tauri/src/commands.rs:1578-1579
  - gap: Returns Ok(json!([])) unconditionally. drawers.svelte.ts:231 marks unwired.csqPending=true.

- Task 4: ReasoningDrawer per-agent budget gauges rendered
  - exists: ReasoningDrawer.svelte:164-183 iterates d.grcBudgets
  - gap: Data source is still synthetic (see Task 1/3 gaps). The plan specifies 8 named rows (Executor, Auditor, ContextCompiler, PromptEngineer, MemoryWriter, Designer, Researcher, MediaValidator) with red bar at <20%; actual render shows a generic loop with no red bar threshold logic and no named-role guarantee.

## Claims MISSING
- SSB action_type grc_step_emitted — searched: GrcStepEmitted, grc_step_emitted in ssb.rs; no evidence. SsbActionType enum ends at CliSubprocessKilled (line 960); none of the 4-5 new types from Task 2 are present.
- SSB action_type grounding_sweep_result — searched: GroundingSweepResult, grounding_sweep_result in ssb.rs; no evidence.
- SSB action_type csq_evaluated — searched: CsqEvaluated, csq_evaluated in ssb.rs; only a TODO comment at task_runner.rs:5020.
- SSB action_type feasibility_check — searched: FeasibilityCheck in ssb.rs; no evidence. (FeasibilityReport exists but is a different variant from Batch 3A.)
- SSB action_type auditor_rubber_stamp_rejected — searched: AuditorRubberStamp, rubber_stamp in ssb.rs; no evidence. Anti-rubber-stamp only exists as test logic (reasoning/tests/agent_config.rs:353, gate_liveness.rs:299), never emits to SSB.
- Task 3: get_feasibility_log command — searched: get_feasibility_log in dashboard-v2; no evidence. (get_feasibility_report exists, different name/shape.)
- Task 3: get_csq_log command — searched: get_csq_log; no evidence.
- Task 3: get_agent_grc_config command — searched: get_agent_grc_config in dashboard-v2; no evidence.
- Task 3: get_grc_budgets_per_agent command (B-5 plan name) — searched: get_grc_budgets_per_agent; no evidence. Only get_grc_budgets exists.
- Task 5: auditor_rubber_stamp_rejected SSB emit — searched: rubber_stamp in task_runner.rs and ssb.rs; no evidence.
- Task 6: CsqBlockCard.svelte intervention card — searched: CsqBlockCard in entire .elai_cc; no evidence. V2 has no intervention/ subdirectory.
- Task 6: CSQ Critical-without-guard surfaces as red intervention card in ChatWindow — searched: CsqBlock, csq.*intervention; no evidence in dashboard-v2 components.
- workflow/reasoning/B-5-decisions.md — searched via ls workflow/reasoning; file not present.

## Claims UNVERIFIABLE
- DETERMINISTIC_CHECK 3: sqlite3 SSB query for csq_evaluated at runtime — requires a live session; no SSB db available for inspection.
- V1 crates/dashboard/src/commands.rs (B-5 "Files touched" list) — path does not match reality; the live dashboard is crates/dashboard-v2, not crates/dashboard. V1 commands.rs appears to be a backup/bkp path. Whether V1 commands got any B-5 work cannot be determined without running V1.

## What must be done (action list)
- [ ] crates/orchestrator/src/ssb.rs: Add SsbActionType variants GrcStepEmitted, GroundingSweepResult, CsqEvaluated, FeasibilityCheck, AuditorRubberStampRejected with payloads and as_sql_str/from_str arms.
- [ ] crates/orchestrator/src/task_runner.rs:6218-6227: Implement the TODO — refactor emit_reasoning to accept Option<&Arc<AsyncMutex<PipelineGrc>>>, extract real budget/agent_id/topic values when Some; remove hardcoded constants.
- [ ] crates/orchestrator/src/task_runner.rs: Add real grc_step_emitted SSB row emit at each GRC step site (all 11 emit_reasoning call sites after refactor).
- [ ] crates/orchestrator/src/task_runner.rs:5020: Implement the csq_evaluated SSB emit (currently only a TODO comment).
- [ ] crates/reasoning/src/engine.rs: Add AuditorRubberStampRejected SSB emit in the anti-rubber-stamp enforcement path (line 329).
- [ ] crates/dashboard-v2/src-tauri/src/commands.rs: Add get_csq_log, get_feasibility_log, get_agent_grc_config commands; rename or supplement get_grc_budgets to get_grc_budgets_per_agent per B-5 spec.
- [ ] crates/dashboard-v2/src-tauri/src/commands.rs:1566-1574: Wire get_grounding_sweep to real SSB GroundingSweepResult rows (B-8/B-17 dependency).
- [ ] crates/dashboard-v2/src-tauri/src/commands.rs:1578-1579: Wire get_csq_pending to real csq_evaluated SSB rows.
- [ ] crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte: Add red bar at <20% calls/chars remaining; ensure 8 named roles always rendered even with empty SSB.
- [ ] crates/dashboard-v2/src/lib/components/: Create CsqBlockCard.svelte intervention card; wire to ChatWindow on CSQ Critical-without-guard.
- [ ] workflow/reasoning/B-5-decisions.md: Create the required pre-execution decision doc.

## Notes
- The plan references crates/dashboard/src/commands.rs (V1 path) as a file to modify. Actual live dashboard is crates/dashboard-v2. V1 code under crates/.bkp_old/ is archived; changes must target dashboard-v2.
- get_gg_signals is present and registered but the plan does not list it among B-5 deliverables; it pre-dates B-5.
- FeasibilityReport (Batch 3A) already exists in SSB and is different from the B-5 feasibility_check variant; naming must not collide.
- B-8 is a hard dependency for get_grounding_sweep and CSQ emit to fire in production hot paths. B-5 SSB type registration can proceed independently but live data requires B-8.
