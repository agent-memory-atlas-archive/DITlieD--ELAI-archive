# B-8: PipelineGrc Activation in Production Hot Paths
Objective: call verify_claim, ground_plan, generate_csq, and
check_inline_feasibility from real production code paths so the safety
gates fire during actual task execution. Currently they exist as a
clean wrapper layer with zero callers.
Estimated effort: 1-2 sessions.
Pre-requisites: B-5 (Reasoning producer wiring — so dashboard can render
the activation events).
Parallel-safe with: B-6, B-7, B-9, B-10.

## Context

Per audit-implementation/_AGGREGATE.md PATTERN 2:
PipelineGrc::verify_claim, ground_plan, generate_csq, check_inline_feasibility
all exist at orchestrator/src/grc.rs as clean wrappers around the reasoning
engine. ZERO production code paths call them. Result: every reasoning
capability from Phase 2 is invisible at runtime — model can produce code
with no CSQ guard, no feasibility check, no plan grounding sweep.

GroundingSweepGate is REGISTERED in the gate list at evidence.rs:471 and
PASSES tests, but vacuously passes in production because nothing populates
ctx.ungrounded_claims.

## Reads

- audit-implementation/phase-2B.md, 2C.md, 2D.md
- audit-implementation/_AGGREGATE.md PATTERN 2
- crates/orchestrator/src/grc.rs (PipelineGrc wrapper — find it)
- crates/orchestrator/src/evidence.rs:191-218, 471 (GroundingSweepGate)
- crates/orchestrator/src/task_runner.rs (the hot paths that need to invoke)
- crates/orchestrator/src/dispatcher.rs (pre-tool dispatch — CSQ insertion point)
- crates/orchestrator/src/file_ops.rs (pre-write hook — CSQ insertion point)

## Tasks

1. **Wire ground_plan into PLANNING → TESTING_FIRST transition.**
   Just before evidence_gate evaluates GroundingSweepGate, call
   `pipeline_grc.ground_plan(plan)`. Result populates
   `ctx.ungrounded_claims`. The gate then has real data to evaluate.

2. **Wire verify_claim into context-compiler pre-fetch.** When
   ContextCompiler is about to retrieve context for a phase, verify
   each claim in the phase plan. Skip claims that fail FILE_CHECK /
   SOURCE_CHECK at retrieval (saves tokens by not loading bogus deps).

3. **Wire generate_csq into pre-write dispatch.** In dispatcher.rs (or
   wherever Write/Edit/Bash tool calls are routed), BEFORE the call
   reaches the tool layer:
   - Call `pipeline_grc.generate_csq(function_signature, function_purpose)`
   - If CsqResult.severity == CRITICAL && !guard_added:
     - BLOCK the tool call
     - Emit SSB `csq_evaluated` (B-5 producer wires this)
     - Surface intervention card via dashboard
   - If WARNING: log to SSB but proceed
   - If passed: proceed silently

4. **Wire check_inline_feasibility into EXECUTING.** When executor
   produces code that contains a novel claim (e.g., "uses crate X",
   "calls API Y") not in the plan, call check_inline_feasibility.
   Outcomes:
   - Pass → proceed
   - Fail → use original plan approach
   - Blocked → emit ReplanRequest(MAJOR) — orchestrator handles

5. **Per-baby-step ReplanRequest emission.** Currently 2D's
   FeasibilityOutcome::Blocked is RETURNED but nothing converts it to
   ReplanRequest. Wire that conversion at the EXECUTING-state handler.

6. **Anti-rubber-stamp activation.** Auditor's AgentGrcConfig already
   requires >=1 FINDING: before Converge. Currently engine enforces but
   auditor agent never invokes engine.think() in production. Refactor
   auditor flow to:
   - Spawn auditor agent
   - Auditor calls pipeline_grc.think() with FINDING: prefixed verify steps
   - Anti-rubber-stamp engine check fires
   - Rejected converges emit auditor_rubber_stamp_rejected SSB row
     (B-5 producer)

7. **InlineFeasibilityState.reset() per phase boundary.** Currently
   never called between phases. Wire reset() at FSM phase transition.

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/grc.rs::PipelineGrc::ground_plan
  - crates/orchestrator/src/grc.rs::PipelineGrc::verify_claim
  - crates/orchestrator/src/grc.rs::PipelineGrc::generate_csq
  - crates/orchestrator/src/grc.rs::PipelineGrc::check_inline_feasibility
  - crates/reasoning/src/types.rs::InlineFeasibilityState::reset
  - crates/orchestrator/src/grc.rs::PipelineGrc::think
consumes_symbols:
  - crates/orchestrator/src/evidence.rs::GroundingSweepGate
  - crates/orchestrator/src/task_runner.rs::drive_task
reachable_from:
  - crates/orchestrator/src/task_runner.rs::drive_task
stub_ban:
  - PipelineGrc::ground_plan != TODO
  - PipelineGrc::generate_csq != TODO
  - PipelineGrc::check_inline_feasibility != TODO
  - PipelineGrc::think != TODO
frontend_pair: B-5 surfaces the events
observable_effect:
  trigger: cd /d F:\.ELAI_workflow\.elai_cc && cargo test -p elai-orchestrator --test safety_gate_integration -- generate_csq_critical_without_guard_blocks_when_kill_switch_off
  expect_ssb: csq_evaluated
  expect_artifact: dashboard shows red intervention card requiring user
                   to confirm or cancel

## DETERMINISTIC_CHECK

```bash
# 1. PipelineGrc methods have production callers
rg "pipeline_grc\\.(ground_plan|verify_claim|generate_csq|check_inline_feasibility|think)" \
   .elai_cc/crates/orchestrator/src --glob '!*test*'
# Expected: ≥4 hits (one per method).

# 2. GroundingSweepGate non-vacuous
# Run a task with intentionally ungrounded claim, verify the gate BLOCKS

# 3. CSQ blocks destructive writes
# Run a task that tries to delete files, verify CSQ guard required
```

## ACCEPTANCE

- [ ] All 4 PipelineGrc methods have production callers
- [ ] GroundingSweepGate sees real ungrounded_claims (not always empty)
- [ ] CSQ Critical-without-guard blocks production (verified end-to-end)
- [ ] Inline feasibility Blocked surfaces ReplanRequest
- [ ] Auditor anti-rubber-stamp triggers in production runs
- [ ] InlineFeasibilityState resets per phase boundary
- [ ] No regressions in 2A-2D test suites

## Files touched

- MODIFY: crates/orchestrator/src/task_runner.rs (4 wiring points)
- MODIFY: crates/orchestrator/src/dispatcher.rs (CSQ pre-tool gate)
- MODIFY: crates/orchestrator/src/file_ops.rs (or wherever pre-write lives)
- MODIFY: crates/orchestrator/src/evidence.rs (ground_plan input wire)
- MODIFY: crates/orchestrator/src/agent_pool.rs (auditor invokes engine)
- MODIFY: crates/orchestrator/src/grc.rs (if API needs adjustment)
- ADD: tests in tests/contracts/safety_gate_integration.rs
- CREATE: workflow/reasoning/B-8-decisions.md

## Decision doc

- How to handle CSQ block when user is not present (autonomous mode):
  block + alert via webhook? Or fall back to safer approach?
- Whether ground_plan fires only at PLANNING→TESTING_FIRST, or also on
  every replan cycle.

## Next

B-9 (SessionDag activation), B-10 (ContextBudget profile activation).
