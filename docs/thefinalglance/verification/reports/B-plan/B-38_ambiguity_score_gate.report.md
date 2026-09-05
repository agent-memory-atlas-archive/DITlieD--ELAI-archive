# Verification Report: B-38_ambiguity_score_gate
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-38_ambiguity_score_gate.md
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The core Rust implementation is fully shipped: all types, constants, methods, gate logic, SSB payloads, and contract tests (20 tests) are present in the codebase. The dashboard GatesPanel renders milestone label, per-dimension breakdown bars, floor-failure flags, and streak progress. Two gaps remain: (1) AmbiguityScorer is NOT wired into task_runner.rs or gate.rs as a live caller — the Tauri command `get_ambiguity_state` returns acknowledged mock data with `"mock": true`; (2) the SSB action types exist in ssb.rs but are not surfaced in OrchestrationDrawer.svelte.

## Claims verified DONE

- AmbiguityScore struct with overall_score + breakdown fields — evidence: crates/reasoning/src/ambiguity.rs:109
- ScoreBreakdown struct with goal_clarity / constraint_clarity / success_criteria_clarity / context_clarity fields — evidence: crates/reasoning/src/ambiguity.rs:100
- ComponentScore struct with name / clarity_score / weight / justification — evidence: crates/reasoning/src/ambiguity.rs:92
- is_ready() returns true iff overall_score <= 0.2 — evidence: crates/reasoning/src/ambiguity.rs:128-130
- floor_failures() returns per-dimension failure names — evidence: crates/reasoning/src/ambiguity.rs:135-156
- qualifies_for_advance() requires BOTH overall <= 0.2 AND no floor failures — evidence: crates/reasoning/src/ambiguity.rs:160-162
- AMBIGUITY_THRESHOLD = 0.2 — evidence: crates/reasoning/src/ambiguity.rs:28
- AUTO_COMPLETE_STREAK_REQUIRED = 2 — evidence: crates/reasoning/src/ambiguity.rs:31
- GOAL_CLARITY_FLOOR = 0.75 — evidence: crates/reasoning/src/ambiguity.rs:34
- CONSTRAINT_CLARITY_FLOOR = 0.65 — evidence: crates/reasoning/src/ambiguity.rs:36
- SUCCESS_CRITERIA_CLARITY_FLOOR = 0.70 — evidence: crates/reasoning/src/ambiguity.rs:38
- BROWNFIELD_CONTEXT_CLARITY_FLOOR = 0.60 — evidence: crates/reasoning/src/ambiguity.rs:40
- SCORING_TEMPERATURE = 0.1 — evidence: crates/reasoning/src/ambiguity.rs:43
- GOAL_CLARITY_WEIGHT = 0.40, CONSTRAINT_CLARITY_WEIGHT = 0.30, SUCCESS_CRITERIA_CLARITY_WEIGHT = 0.30 — evidence: crates/reasoning/src/ambiguity.rs:46-48
- BROWNFIELD_GOAL_CLARITY_WEIGHT = 0.35, BROWNFIELD_CONSTRAINT_CLARITY_WEIGHT = 0.25, BROWNFIELD_SUCCESS_CRITERIA_CLARITY_WEIGHT = 0.25, BROWNFIELD_CONTEXT_CLARITY_WEIGHT = 0.15 — evidence: crates/reasoning/src/ambiguity.rs:51-54
- AmbiguityScorer struct with async score() method — evidence: crates/reasoning/src/ambiguity.rs:213, 244
- Adaptive token allocation: initial_max_tokens=2048, doubles on truncation up to max_truncation_doublings=5, max_format_error_retries=5 — evidence: crates/reasoning/src/ambiguity.rs:57-61, 257-273
- JSON parse: handles fenced markdown code-block and raw JSON — evidence: crates/reasoning/src/ambiguity.rs:500-519
- Scores clamped to [0.0, 1.0]; missing justifications get fallback string; missing scores are hard error — evidence: crates/reasoning/src/ambiguity.rs:431-434, 486-493, 480-485
- Deferral instruction in system prompt: "Items in DEFERRALS are decide-later and MUST NOT penalize clarity." — evidence: crates/reasoning/src/ambiguity.rs:326-327
- GroundingSweepGate extended: ungrounded_count > 0 -> BLOCK (priority 1); ambiguity > 0.2 -> BLOCK; floor failures -> BLOCK; qualifies AND streak < 2 -> BLOCK(streak); qualifies AND streak >= 2 -> ALLOW — evidence: crates/orchestrator/src/evidence.rs:229-315
- TransitionContext carries ambiguity_score: Option<AmbiguityScore> and streak_count — evidence: crates/orchestrator/src/evidence.rs:98-143
- Backward-compat: None ambiguity_score -> B-17 behavior (ALLOW) — evidence: crates/orchestrator/src/evidence.rs:255-258
- Streak counter logic uses is_plan_revision flag; caller resets streak_count to 0 before invoking gate on plan revision — evidence: crates/orchestrator/src/evidence.rs:295-314, contract test:494-519
- SSB action types AmbiguityScored / AmbiguityFloorFailed / AmbiguityStreakAdvanced — evidence: crates/orchestrator/src/ssb.rs:914,918,921
- SSB payloads AmbiguityScoredPayload / AmbiguityFloorFailedPayload / AmbiguityStreakAdvancedPayload — evidence: crates/orchestrator/src/ssb.rs:4851,4869,4882
- "ambiguity_scored" / "ambiguity_floor_failed" / "ambiguity_streak_advanced" string serialization — evidence: crates/orchestrator/src/ssb.rs:1114-1116, 1291-1293
- lib.rs re-exports all B-38 public symbols — evidence: crates/reasoning/src/lib.rs:10-12
- Contract test file with 20 tests — evidence: crates/contracts/tests/ambiguity_scorer_contract.rs:1-519
- ouroboros-ambiguity-interface.md paper produced — evidence: workflow/papers/ouroboros-ambiguity-interface.md (file exists)
- B-38-decisions.md produced — evidence: workflow/reasoning/B-38-decisions.md (file exists)
- GatesPanel shows overall ambiguity score with milestone label and colors — evidence: crates/dashboard-v2/src/lib/components/GatesPanel.svelte:96-101
- GatesPanel shows per-dimension breakdown bars with score/weight/floor columns — evidence: crates/dashboard-v2/src/lib/components/GatesPanel.svelte:105-134
- GatesPanel shows floor-failure flags — evidence: crates/dashboard-v2/src/lib/components/GatesPanel.svelte:136-139
- GatesPanel shows streak progress (current/required) — evidence: crates/dashboard-v2/src/lib/components/GatesPanel.svelte:142-144
- get_ambiguity_state Tauri command registered — evidence: crates/dashboard-v2/src-tauri/src/lib.rs:256, crates/dashboard-v2/src-tauri/src/commands.rs:6561

## Claims PARTIAL

- AmbiguityScorer wired into GroundingSweepGate (live call path)
  - exists: gate logic in crates/orchestrator/src/evidence.rs consumes ambiguity_score when present in TransitionContext; SSB payloads and action types are defined
  - gap: AmbiguityScorer is NOT called anywhere in task_runner.rs or gate.rs to populate ambiguity_score; get_ambiguity_state command returns hardcoded mock (mock: true) explicitly deferred to follow-up B-N; searched: task_runner.rs grep "AmbiguityScorer" = no matches; gate.rs grep "AmbiguityScorer" = no matches

- SSB action types surfaced in OrchestrationDrawer
  - exists: SsbActionType::AmbiguityScored / AmbiguityFloorFailed / AmbiguityStreakAdvanced in ssb.rs:914-921; OrchestrationDrawer.svelte exists
  - gap: OrchestrationDrawer.svelte contains no "ambiguity" references; the 3 action types are defined and serialized in ssb.rs but not rendered in the drawer

## Claims MISSING

None — all plan claims have at least partial implementation evidence.

## Claims UNVERIFIABLE

- >= 12 tests pass (cargo test result) — reason: auditor role does not run tests; 20 test functions counted in contract file (tests 1-20) which exceeds the >= 12 requirement, but actual pass/fail result is unverified without a test run

## What must be done (action list)

- [ ] Wire AmbiguityScorer into the live prompt-flow: task_runner.rs (or equivalent orchestrator call site) must call AmbiguityScorer.score() and populate TransitionContext.ambiguity_score before GroundingSweepGate fires
- [ ] Replace get_ambiguity_state mock data with real SSB row reads (ambiguity_scored / ambiguity_floor_failed / ambiguity_streak_advanced) from state.viewer once live producer lands
- [ ] Add ambiguity_scored / ambiguity_floor_failed / ambiguity_streak_advanced rendering to OrchestrationDrawer.svelte
- [ ] Run cargo test -p elai_reasoning -- ambiguity + cargo test -p elai_contracts -- ambiguity to confirm >= 12 tests pass

## Notes

The plan itself explicitly scopes the live AmbiguityScorer-into-task_runner wiring as a "follow-up B-N" (GatesPanel comment line 9: "real producer wires AmbiguityScorer into the gate in a follow-up B-N"). The commands.rs comment at line 6553 confirms this is an acknowledged intentional deferral, not an oversight. The gate gracefully degrades to B-17 behavior when ambiguity_score is None, so the deferral is safe. The 19 constants (all 11 weights + 4 floors + threshold + streak + 3 retry defaults) are ported verbatim. The OrchestrationDrawer gap is a minor surface omission; the SSB wiring in ssb.rs is complete.
