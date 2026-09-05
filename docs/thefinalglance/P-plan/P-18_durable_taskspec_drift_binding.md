# P-18: durable TaskSpec execution + approval drift-binding (anti-TOCTOU)

STATUS: PLANNED (WAVE OC-2; EXTENSION of O-2 + O-4 + P-11 + 3A/4G; PROOF_SPINE: none (product), owner-acked 2026-07-11; execute after the proof spine per A-7)

- Track OPENCLAW-STEAL | lever durable-runs-anti-toctou | grounding STRONG for the verified half (Task-Flow revision-counter + approval drift-binding source-verified V-grade); WEAK-until-RECON for the resume-token step-skip (C1 unverified-external) | nature EXTENSION | effort M-L | WAVE OC-2 | depends P-11 | PROOF_SPINE none (product)
- Premise: O-2/P-11 have the trust semantics but not the boring durable-execution correctness openclaw bled for. Add the VERIFIED mechanics (revision-counter concurrency, sticky cancel, retry ladders, due-time clamp) + approval drift-binding (anti-TOCTOU); carry the UNVERIFIED lobster resume-token as a RECON item re-derived as a real subprocess-checkpoint model.

## Context — why this matters
Grounded in workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md 2.4 + 2.14 + steals S1/S6/S19 + deep-read C1/S6 amendment. openclaw's Task Flow uses REVISION-COUNTER optimistic concurrency (a stale write = conflict, never clobber; sticky persisted cancel surviving restart) — VERIFIED-IN-SOURCE (ENTRY 13 amendment, task-flow-registry.ts:515-521). Its exec-approval BINDS a canonical systemRunPlan (cwd/argv/env/pinned-executable + one file operand), frozen between prepare and exec, drift = DENIED (anti-TOCTOU) — the invariant ELAI's corpus has gates for but never STATES. CORRECTION C1: the lobster "approve+resume never re-runs prior steps" claim is UNVERIFIED-external (the engine is an un-vendored npm type-stub; the in-repo wrapper force-errors needs_input) — so re-derive it as a REAL subprocess-checkpoint model (SIGKILL + pipe backpressure + a step-checkpoint store), stronger than their cooperative enforcement. The commitment due-time clamp (S19): an inferred follow-up may never fire in the tick it was inferred.

## Existing ground truth (B..Q coverage; PLAN-asserted; executor re-greps W-7)
- O-2 scheduled TaskSpec substrate (O-SUMMARY_INDEX.md:32; task_spec_fired): the run store this hardens.
- O-4 canvas WORKFLOW mode (O-SUMMARY_INDEX.md:64): the run visualization.
- P-11 workflow-from-goal confirm gate (P-0_INDEX.md SECOND WAVE): drift-binding hardens its confirm-before-run MOVE 3.
- 3A per-transition human gates + 4G AutonomousGate: every approval (human or autonomous) binds a plan; G blast-radius gates compose.
- Q-4 swap-restart: iteration state is model-conditioned — a substitution restarts the unit; sticky-cancel + revision-counter keep that clean.

## Steal/build (REVISED scope, W-10 tiered)
- SHIP NOW (Tier-1): (a) revision-counter optimistic concurrency on the TaskSpec run store (a stale expectedRevision -> conflict, never clobber) + sticky persisted cancel intent surviving restart; (b) approval drift-binding — every human/autonomous approval binds a canonical hashed run-plan (cwd/argv/env/pinned-executable) + concrete operand fingerprints; execution re-validates; ANY drift = deny.
- DEFER (Tier-2): retry-ladders-per-failure-class + watchdog/preflight phases; the commitment due-time clamp (S19); the subprocess-checkpoint resume model (the C1 re-derivation — RECON first).
- EXCLUDED as vaporware: the lobster "approve+resume never re-runs prior steps" claim AS openclaw's (C1 unverified — cite as an idea to re-derive, never "openclaw shipped this").

## Reads (mandatory before coding)
- workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md (2.4, 2.14; steals S1/S6/S19; ENTRY 13 C1 + S6 amendment)
- docs/thefinalglance/O-plan/O-2_*.md + O-4_*.md (the run store + canvas), P-11 (the confirm gate)
- crates/orchestrator/src/ (the TaskSpec run store + the 3A/4G approval seams)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/taskspec/run_store.rs (RunStore: revision-counter optimistic concurrency + sticky-cancel)
  - crates/orchestrator/src/approval/drift_bind.rs (ApprovalPlan: canonical hashed run-plan + operand fingerprints; re-validate -> deny on drift)
consumes_symbols:
  - crates/orchestrator/src/taskspec/mod.rs::TaskSpec (O-2; the run store persists these)
  - crates/orchestrator/src/fsm/gates.rs::ApprovalGate (3A/4G; the approval seam drift-binding hardens)
reachable_from:
  - crates/cli/src/main.rs (`elai workflow resume|cancel`) [PRODUCTION CLI]
  - crates/orchestrator/src/fsm/gates.rs (the 3A/4G approval path re-validating the bound plan) [PRODUCTION FSM gate]
  - a #[cfg(test)] or contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO last-writer-wins on a concurrent run write (a stale revision = conflict, never clobber)
  - NO approval that executes a DRIFTED plan (re-validate the hashed plan + operand fingerprints; drift = deny)
  - NO commitment firing in the same tick it was inferred (due-time clamp >= 1 tick after creation)
  - NO cooperative-only timeout dressed as a hard kill (the resume model is a real subprocess checkpoint or it is DEFERRED); NO .unwrap on prod; NO blake2/sha2
frontend_pair:
  run-revision + drift-reject rows in the O-4 workflow view (W-4).
observable_effect:
  trigger: "elai workflow run --fixture tests/fixtures/workflow/concurrent_write_and_operand_swap.json"
  expect_ssb: "approval_drift_reject"
  expect_artifact: "one taskspec_run_revision conflict (the stale write refused, no clobber) + one approval_drift_reject on a post-grant operand swap; a cancel survives a simulated restart; NEGATIVE: a last-writer-wins clobber, an approval that executes a drifted plan, or a commitment firing in its own inference tick, FAILS"

## Wargame (plan-standard unit shape)
EXPECTED_OBSERVATION: two concurrent writes to one TaskSpec run (one on a stale revision) produce one taskspec_run_revision conflict (the stale write refused, no clobber); an approval whose operand is swapped after grant produces one approval_drift_reject; a cancel survives a simulated restart.
ON_FAIL: a stale write clobbers, or a drifted approval executes (CAUSE: the run store lacks the revision check, or the approval binds nothing). COUNTER-MOVE: gate every run mutation on expectedRevision; hash the run-plan + operands at grant and re-validate at exec; the negative tests (clobber / drift-exec) must FAIL.
FORK: if the resume-token step-skip cannot be built as a REAL checkpoint (structural), DO NOT ship a cooperative-only imitation — defer the resume leg to Tier-2 and ship the revision-counter + drift-binding (the verified half) alone; record the deferral. This is the C1 fork.
RECON_NEEDED: the lobster resume-token step-skip is UNVERIFIED-external (C1). Settling check: source-read the openclaw lobster wrapper (lobster-runner.ts) + task-flow-registry.ts to confirm the revision-counter shape, and prototype a subprocess step-checkpoint store before committing the resume leg.
ABORT: if an approval could ever bind a plan LOOSELY (hashing only part of the argv/env/operand set), STOP and flag — a partial bind is a TOCTOU hole (the exact thing S1 closes).
VERIFY: the load-bearing proof is the off-agent approval_drift_reject + taskspec_run_revision-conflict rows, driven by the real `elai workflow`/FSM approval entrypoints. Run:
```bash
python .claude/tools/plan_gate_check.py --file docs/thefinalglance/P-plan/P-18_durable_taskspec_drift_binding.md   # authoring gate (exit 0)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-18_durable_taskspec_drift_binding.md   # execution done-gate
```
REDTEAM: surviving-attack seed — a TOCTOU attacker who swaps the file operand between approval and exec is defeated by the operand fingerprint re-validation (drift = deny). Attack that would succeed if unpatched: a caller that edits argv after approval — PATCH: the canonical plan hash covers cwd/argv/env/pinned-executable; any post-grant edit mismatches. Deep red-team DEFERRED to execution pre-flight.

## INTEGRATION_SMOKE
An independent agent drives a real multi-step workflow through P-11's confirm gate on a HARD input: concurrent writers, a mid-run restart (asserting sticky-cancel survives), and a post-approval operand swap (asserting drift-reject), all under 4G autonomous mode. Toy single-write test does NOT satisfy R10.

## Grounding / citations
- Source: workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md 2.4/2.14, steals S1/S6/S19, ENTRY 13 C1 + S6 amendment (revision-counter VERIFIED; resume-token UNVERIFIED-external).
- ELAI ground truth (PLAN-asserted, executor re-greps W-7): O-2, O-4, P-11, 3A/4G, Q-4 per the indexes cited.
- CLAUDE.md hard rules: no last-writer-wins (local-first determinism); blake3 for the plan hash; no .unwrap on prod; no cooperative timeout dressed as a hard kill.

## PRODUCTION_HOOK
crates/orchestrator/src/fsm/gates.rs — the 3A/4G ApprovalGate re-validates the bound ApprovalPlan before execution; `elai workflow resume|cancel` drives the RunStore revision-counter + sticky-cancel. Both are wired into their production seams in this changeset.

## Next
Closes O2 (approval drift-binding — a stated invariant the corpus lacked) + the durable-execution correctness gap in O-2. Pairs with P-11 (hardens its confirm gate) and Q-4 (swap-restart attribution).

End of P-18.
