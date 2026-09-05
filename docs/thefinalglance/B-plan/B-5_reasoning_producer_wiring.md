# B-5: Producer Wiring — Reasoning Engine
Objective: surface real per-agent GRC budgets, grounding sweep results,
CSQ guards, and feasibility outcomes in the dashboard. Replace the
synthetic ReasoningDrawer with real engine telemetry. Couples tightly
with B-8 (PipelineGrc activation in production hot paths).
Estimated effort: 1-2 sessions.
Pre-requisites: none for producer code; B-8 must land for the producers
to actually fire on every reasoning event in production.
Parallel-safe with: B-4, B-6.

## V2 REBASELINE NOTE (2026-05-07)

Producer-side work (orchestrator/PipelineGrc/GroundingSweep) is V2-agnostic.
Dashboard surface targets:
  V1 crates/dashboard/src/commands.rs       -> V2 crates/dashboard-v2/
                                                src-tauri/src/lib.rs (B-36)
  V1 src-frontend/src/drawers/Reasoning
     Drawer.svelte                          -> crates/dashboard-v2/src/lib/
                                                drawers/ReasoningDrawer
                                                .svelte (already scaffolded;
                                                stub-backed via stub.ts:
                                                get_grc_budgets,
                                                get_grounding_sweep,
                                                get_csq_pending,
                                                get_gg_signals)
  V1 src-frontend/src/intervention/
     CsqBlockCard.svelte                    -> NEW component at
                                                crates/dashboard-v2/src/lib/
                                                components/CsqBlockCard
                                                .svelte (V2 has no
                                                intervention/ subdirectory
                                                yet)
Keep stub.ts names 1:1 when wiring live invokes.

## Context

Per audit-dashboard/reasoning-wiring.md: reasoning is "MOSTLY INVISIBLE".
Only 2A has any drawer (ReasoningDrawer 121 lines), and it renders
SYNTHETIC data: emit_reasoning at task_runner.rs:3985-4004 hardcodes
agent_id="task_runner", topic="task_runner", budget_remaining=0,
grounding_level="medium" literal, confidence=0.9 literal, elapsed_ms=0
literal. BasicGrcEngine::think() is never invoked in production
(only in reasoning/tests/* and orchestrator/src/grc.rs:217 test mod).

Phases 2B / 2C / 2D have ZERO Tauri commands, ZERO frontend components,
and (for 2B/2D) ZERO SSB action_types. CSQ Critical blocks and Blocked
feasibility outcomes are silent — a safety-UX failure.

## Reads

- audit-dashboard/reasoning-wiring.md
- audit-implementation/phase-2A.md, 2B.md, 2C.md, 2D.md
- summary/phase-2-summary.md
- crates/reasoning/src/engine.rs (BasicGrcEngine)
- crates/orchestrator/src/grc.rs (PipelineGrc wrapper)
- crates/orchestrator/src/task_runner.rs:3985-4004 (synthetic emit_reasoning)
- crates/orchestrator/src/agent_pool.rs:467 (divergent table to FIX in B-15)

## Tasks

1. **Real per-agent GRC emit.** Refactor task_runner.rs:3985-4004 to
   accept a real PipelineGrc handle and pull live values
   (agent_id, topic, budget_remaining_calls, budget_remaining_chars,
   grounding_level enum, confidence f32, elapsed_ms u64). Stop hardcoding.

2. **New SSB action_types** for reasoning visibility:
   - `grc_step_emitted`: payload {agent_id, topic, step_type, grounding_level, char_count}
   - `grounding_sweep_result`: payload {plan_id, claims_total, grounded, verify_needed, ungrounded[]}
   - `csq_evaluated`: payload {function_name, severity (CRITICAL|WARNING), guard_present, blocked}
   - `feasibility_check`: payload {claim, outcome (Pass|Fail|Blocked), source}

3. **Tauri commands for ReasoningDrawer**:
   - `get_grc_budgets_per_agent()`: returns Vec<{agent_id, budgets_per_topic[]}>
   - `get_grounding_sweep(plan_id)`: returns last sweep result
   - `get_csq_log(limit)`: returns recent CSQ events
   - `get_feasibility_log(limit)`: returns recent feasibility events
   - `get_agent_grc_config()`: exposes the AgentGrcConfig table (helps user
     SEE the divergent-table bug B-15 will fix)

4. **ReasoningDrawer surface** — full redesign per dashboard-design.txt
   reasoning section:
   - Per-agent budget gauges (8 rows: Executor, Auditor, ContextCompiler,
     PromptEngineer, MemoryWriter, Designer, Researcher, MediaValidator)
   - Each gauge shows: budget_remaining_calls / budget_max_calls,
     budget_remaining_chars / budget_max_chars, with red bar if <20%
   - Last grounding sweep panel: pie chart (GROUNDED green, VERIFY_NEEDED amber,
     UNGROUNDED red), click for ungrounded claim list
   - CSQ guards section: list of recent (function, severity, blocked?) with
     red badge if Critical-unguarded blocked
   - Feasibility checks: Pass/Fail/Blocked counts + last 5 events
   - Per-role config viewer (collapsible): shows AgentGrcConfig fields
     as table with cell highlighted red where divergent table differs from spec

5. **Anti-rubber-stamp counter.** Auditor anti-rubber-stamp enforcement
   (engine.rs:297-315) currently rejects converge silently. Add SSB emit
   `auditor_rubber_stamp_rejected` so dashboard can show "Auditor 12 rubber
   stamps rejected this session" in ReasoningDrawer.

6. **Wire safety blocks.** When CSQ Critical-without-guard fires (already
   blocks progression in BasicGrcEngine), surface in dashboard as a red
   intervention card: "Tool call blocked: function `delete_file` requires
   guard for data_loss CSQ. [View details] [Override] [Decline]". Decline
   bubbles back to FSM as ReplanRequest.

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/ssb/action_type.rs::SsbActionType::GrcStepEmitted
  - crates/orchestrator/src/ssb/action_type.rs::SsbActionType::GroundingSweepResult
  - crates/orchestrator/src/ssb/action_type.rs::SsbActionType::CsqEvaluated
  - crates/orchestrator/src/ssb/action_type.rs::SsbActionType::FeasibilityReport
  - crates/orchestrator/src/ssb/action_type.rs::SsbActionType::AuditorRubberStampRejected
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_grc_budgets
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_grounding_sweep
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_csq_log
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_feasibility_log
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_agent_grc_config
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_auditor_rubber_stamp_rejections
consumes_symbols:
  - crates/dashboard-v2/src/lib/drawers/ReasoningDrawer.svelte
  - crates/dashboard-v2/src/lib/components/CsqBlockCard.svelte
reachable_from:
  - crates/orchestrator/src/task_runner.rs::start_task_runner
stub_ban:
  - emit_reasoning != agent_id="task_runner"
  - emit_reasoning != grounding_level="medium"
  - emit_reasoning != confidence=0.9
  - emit_reasoning != elapsed_ms=0
frontend_pair: same B-step
observable_effect:
  trigger: cd /d F:\.ELAI_workflow\.elai_cc && cargo run -p elai-cli -- run --project-root F:\.ELAI_workflow\.elai_cc --task "delete_file test.txt"
  expect_ssb: csq_evaluated
  expect_artifact: ReasoningDrawer shows real per-agent budgets and last
                   CSQ event

## DETERMINISTIC_CHECK

```bash
# 1. No hardcoded synthetic emit_reasoning constants
rg "agent_id.*=.*\"task_runner\"" .elai_cc/crates/orchestrator/src/task_runner.rs
# Expected: zero (replaced with real per-agent values).

# 2. SSB types added
rg "SsbActionType::GrcStepEmitted|GroundingSweepResult|CsqEvaluated|FeasibilityReport" \
   .elai_cc/crates/orchestrator/src/ssb/action_type.rs
# Expected: 4 hits.

# 3. Smoke: run a task with file delete, verify CSQ event in SSB
sqlite3 .elai_cc/.elai_cc/sessions/<sid>/ssb.db \
  "SELECT * FROM ssb WHERE action_type='csq_evaluated' LIMIT 5;"
```

## ACCEPTANCE

- [ ] emit_reasoning pulls real values; no hardcoded constants
- [ ] 4-5 new SSB action_types added
- [ ] 5 new Tauri commands return real data
- [ ] ReasoningDrawer shows 8 per-agent budget gauges
- [ ] Grounding sweep pie chart renders
- [ ] CSQ + feasibility panels render real events
- [ ] Anti-rubber-stamp counter visible
- [ ] CSQ Critical-without-guard surfaces as intervention card

## Files touched

- MODIFY: crates/orchestrator/src/task_runner.rs (real emit_reasoning)
- MODIFY: crates/orchestrator/src/ssb.rs (new action_types + payloads)
- MODIFY: crates/dashboard/src/commands.rs + ssb_viewer.rs (new commands)
- MODIFY: crates/dashboard/src-frontend/src/drawers/ReasoningDrawer.svelte
- ADD: crates/dashboard/src-frontend/src/intervention/CsqBlockCard.svelte
- CREATE: workflow/reasoning/B-5-decisions.md

## Next

B-6 (Orchestration), B-7 (Pipeline Factory), B-8 (PipelineGrc activation)
fire the producers added here.
