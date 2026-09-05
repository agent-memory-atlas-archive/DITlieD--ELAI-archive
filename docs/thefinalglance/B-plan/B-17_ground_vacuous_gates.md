# B-17: Ground the Vacuous Gates
Objective: gates that exist but pass vacuously (no real signal flows in)
get the producers wired so they actually fire. Covers: GroundingSweepGate,
SearchMode cascade, SupervisionManager (covered partially by B-6).
Estimated effort: 1 session.
Pre-requisites: B-8 (PipelineGrc activation — covers GroundingSweepGate
already), B-4 (SearchMode cascade wired), B-6 (SupervisionManager wired).
Parallel-safe with: B-15, B-16, B-18, B-19, B-20.

## V2 REBASELINE NOTE (2026-05-07)

Producer-side gate-grounding work is V2-agnostic. Dashboard surface targets:
  V1 src-frontend/src/orchestration/
     GatesPanel.svelte                     -> Render inside V2's
                                               OrchestrationDrawer.svelte
                                               (already scaffolded; consumes
                                               stub.get_evidence_gates).
                                               If a dedicated panel file is
                                               desired, place it under
                                               crates/dashboard-v2/src/lib/
                                               orchestration/GatesPanel.svelte
  V1 crates/dashboard/src/commands.rs (gate
     verdict aggregation queries)          -> crates/dashboard-v2/src-tauri/
                                               src/lib.rs (B-36 wires
                                               get_evidence_gates real path)

V2 ActivityWindow surfaces gate verdicts as 7 pips between phase ring and
step ring (DOMAIN-CONTEXT §2 ACTIVITY signals layer); the drawer is the
detailed view.

## Context

Many gates exist but are dead in practice:
- GroundingSweepGate: registered, vacuously passes (B-8 fixes via
  ground_plan call)
- SearchMode cascade: implemented, only one startup-time call site (B-4
  fixes via QueryPipeline integration)
- SupervisionManager: implemented, NEVER wired into task_runner (B-6 fixes)
- CSQ enforcement in pre-tool dispatch (B-8)
- Inline feasibility blocks in EXECUTING (B-8)
- Auditor anti-rubber-stamp activation (B-8)
- CliInteractiveGate instantiation (B-2)

This B-step is a META-step: VERIFY all the above gates fire in production,
add regression tests that fail if they go vacuous again, and add dashboard
visibility to gate verdict frequency.

## Reads

- audit-implementation/_AGGREGATE.md PATTERN 2 (WRAPPER LAYER WITHOUT CALLERS)
- audit-implementation/_AGGREGATE.md "ACTIVATE THE GATES" recommendation
- B-8 / B-6 / B-4 / B-2 outputs

## Tasks

1. **Regression test suite for gate liveness.** Add tests/contracts/
   gate_liveness.rs that, for each gate, runs a synthetic flow and
   asserts the gate fires at least once with non-vacuous input.
   Examples:
   - Submit task with intentionally ungrounded claim → assert
     GroundingSweepGate BLOCKS (not vacuously passes)
   - Disable graphcodebert mid-session → assert SearchMode cascades to
     SparseOnly + emits SSB knowledge_degradation
   - Spawn agent that crashes 3x → assert SupervisionManager triggers
     escalation
   - Submit destructive call without guard → assert CSQ Critical blocks
   - Inject infeasible inline claim → assert ReplanRequest emitted
   - Auditor returns Converge without FINDING → assert
     anti_rubber_stamp_rejected fires

2. **Dashboard gate-verdict frequency panel.** Add new section to
   OrchestrationDrawer (or create GatesPanel.svelte) showing per-gate
   verdict counts over the last N sessions:
   - GroundingSweepGate: 47 PASS / 3 BLOCK
   - SearchMode cascade: 0 transitions today
   - SupervisionManager: 12 spawns / 0 escalations
   - CSQ guards: 156 evaluated / 4 blocked
   - Feasibility checks: 89 PASS / 2 FAIL / 1 BLOCKED
   - Anti-rubber-stamp: 23 auditor converges / 1 rejected

3. **CI gate-liveness check.** Run the regression suite on every PR.
   Fail if any gate becomes vacuous.

4. **Decision doc for INTERACTIVE blocks.** When a gate BLOCKS in
   INTERACTIVE mode, the user sees the intervention card. Decision:
   what's the default action if user dismisses without choice? Log +
   re-prompt, or treat as Reject? Document in B-17-decisions.md.

5. **Performance check.** With gates active, measure latency added per
   FSM transition. If >100ms, optimize (e.g., async ground_plan in
   parallel with other PLANNING work).

6. **Spec amendment if needed.** If any gate's verdict semantics change
   during wiring (e.g., GroundingSweepGate now blocks where it used to
   pass), update spec text accordingly.

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/tests/gate_liveness.rs (regression suite, 6 active / 2 ignored)
  - crates/dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte VERDICT COUNTS block (L418-421, inline per-gate counts; no separate GatesPanel.svelte for B-17 — GatesPanel.svelte belongs to B-38 AMBIGUITY)
  - .github/workflows/gate-liveness.yml + scripts/check_gate_liveness.sh (CI gate-liveness runner)
consumes_symbols:
  - all gates from B-2/B-4/B-6/B-8
reachable_from:
  - CI runner (.github/workflows/gate-liveness.yml on push + pull_request)
  - dashboard (OrchestrationDrawer.svelte via stub.get_gate_verdict_counts -> commands::get_gate_verdict_counts -> ssb_viewer::aggregate_gate_verdict_counts)
stub_ban:
  - No gate may regress to vacuous-pass-only state
frontend_pair: B-17 itself
observable_effect:
  trigger: bash .elai_cc/scripts/check_gate_liveness.sh
  expect_artifact: each gate fires at least once with non-trivial input
  expect_ssb: SsbPayload::EvidenceGateEvaluated emitted at PLANNING->TESTING_FIRST (task_runner.rs:7103) and EXECUTING->AUDITING (task_runner.rs:10746) gate-fire sites; aggregated by ssb_viewer::aggregate_gate_verdict_counts (GROUP BY gate_id, verdict)

## DETERMINISTIC_CHECK

```bash
# 1. Regression suite passes (6 active / 2 ignored)
cd .elai_cc && cargo test --manifest-path crates/orchestrator/Cargo.toml --test gate_liveness -- --nocapture

# 2. CI runner exists
test -f .github/workflows/gate-liveness.yml && \
  test -f scripts/check_gate_liveness.sh && \
  bash scripts/check_gate_liveness.sh

# 3. Dashboard surface (inline VERDICT COUNTS block in OrchestrationDrawer, NOT a separate GatesPanel)
rg "gateVerdictCounts" .elai_cc/crates/dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte
# Expected: ≥1 hit at L418-421.
```

## ACCEPTANCE

- [ ] Regression suite has one test per gate (≥6 tests)
- [ ] All tests pass
- [ ] Dashboard OrchestrationDrawer VERDICT COUNTS block renders per-gate counts
- [ ] CI gate-liveness check runs on PR
- [ ] Performance overhead per transition <100ms

## Files touched

- ADD: crates/orchestrator/tests/gate_liveness.rs (regression suite, 6 active / 2 ignored)
- ADD: scripts/check_gate_liveness.sh + scripts/check_gate_liveness.bat (CI runner)
- ADD: .github/workflows/gate-liveness.yml (CI workflow)
- MODIFY: crates/dashboard-v2/src-tauri/src/commands.rs (get_evidence_gates + get_gate_verdict_counts aggregation commands)
- MODIFY: crates/dashboard-v2/src-tauri/src/lib.rs (invoke_handler registration)
- MODIFY: crates/orchestrator/src/ssb_viewer.rs (recent_evidence_gates + aggregate_gate_verdict_counts)
- MODIFY: crates/dashboard-v2/src/lib/api/{types,stub}.ts (EvidenceGate + GateVerdictCount interfaces + tryInvoke wrappers)
- MODIFY: crates/dashboard-v2/src/lib/state/drawers.svelte.ts (gateVerdictCounts $state + Promise.all refresh)
- MODIFY: crates/dashboard-v2/src/lib/drawers/OrchestrationDrawer.svelte (EVIDENCE GATES section + inline VERDICT COUNTS block at L418-421; NOT a separate GatesPanel.svelte)
- CREATE: workflow/reasoning/B-17-decisions.md

## Next

B-18 (Tier C sandbox activation).
