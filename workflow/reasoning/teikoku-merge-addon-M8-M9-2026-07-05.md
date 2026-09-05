TEIKOKU MERGE PLAN ADDON — M8 + M9 (the guide-up-front leg of the 0-commits fix)
Date: 2026-07-05
Composes with (does NOT restate): workflow/handoff/teikoku-glm-orchestrator-merge.md (M1-M7). Read that first; this adds two items it is missing.
Grounding: the teikoku handoffs (glm-walk, glm-orchestrator-merge, general-orchestrator, build); ELAI mechanisms I-5 / I-20 (docs/thefinalglance/ELAI-SYSTEMS-OVERVIEW.md §13) and D-030 (.claude/rules/production-caller-gate.md + wiring-enforcement.md). Every named TEIKOKU seam is taken FROM the handoffs, not from live source — confirm against ~/teikoku/teikoku/conductor/*.py before editing.
Boundary: this file is the ELAI-BACKLOG-WALK-THROUGH-TEIKOKU fix (the money metric: INTEGRATION_PUSHED > 0). The harness-itself fix is a SEPARATE file (teikoku-harness-fix-modes-2026-07-05.md).


0. THE THROUGH-LINE (why M1-M9 cohere)

The merge plan found seven fixes (M1-M7) by diagnosis. They cohere under ONE principle: the 0-commits failure is a REMEDIATION-vs-GUIDANCE problem. TEIKOKU hands a free/weak model (glm-5.2) a hard greenfield task with a prose objective, lets it run, then REJECTS the output at a frozen gate — so the model burns context and lands NEEDS_REVIEW. The standalone glm-orchestrator committed ~70 plans because its dominant operation was RECOGNITION (precheck -> certify already-shipped), which is guidance-shaped, not blind generation.

M1 (precheck routing) and M6 (lane split) already move the DOMINANT operation from generate-then-reject to recognize-then-certify. M3 (populate write-sets from WIRING_CONTRACT) already starts compiling the task's constraints up front. M8 and M9 complete the guide-up-front leg for the tasks that DO need genuine execution.

Priority: M1 + M9 + M6 are the land-the-first-commit path (do these first). M8 is the free-lane yield multiplier once tasks actually run. M9 SUBSUMES M3 (same import seam, more fields) — implement M9 and M3 is done.


M8 — GATE-FINDING -> FORWARD-CONSTRAINT, ONE BOUNDED RETRY BEFORE NEEDS_REVIEW

PROBLEM. Today a gate BLOCK sends the task straight to NEEDS_REVIEW (terminal; dependents starve). The gate's structured finding — which is an off-agent artifact already in the SSB (the exact wire-dark symbol from production_caller_check, the allowed_write_violation path, the failed acceptance id) — is discarded. The merge plan's own failure-split shows GATE_PASS=16 as PARTIAL passes that never fully cleared: these are precisely the marginal tasks that ONE targeted, guided retry could flip, with no paid lane. Sending them to NEEDS_REVIEW throws that away.

FIX. Insert a GUIDED_RETRY disposition between gate-fail and NEEDS_REVIEW. On a BLOCK whose finding is machine-actionable AND attempts < 1: extract the finding into a compact forward constraint (<= ~30 tokens, the I-5 discipline: keep the LESSON, do NOT re-feed the whole failed transcript — that anchors and pollutes the retry) and re-dispatch the SAME task once with the constraint injected as an explicit directive. This is I-20's "survivors become structured counterexample directives to the generator," at task grain.

  - Exactly ONE guided retry (anti-spiral; matches the existing 1-cycle-audit policy). A second failure -> NEEDS_REVIEW or M6 paid lane.
  - The constraint MUST be built from the gate's OWN output (the off-agent artifact), never the worker's self-report (wiring-enforcement R4/R5 — measure the objective, not a proxy).
  - Ordering vs M6: M8 is the CHEAP free-lane attempt BEFORE M6's paid escalation. gate fail -> finding machine-actionable & attempts<1 ? -> guided retry (free) -> still fail -> M6 paid / NEEDS_REVIEW. M8 reduces paid-lane load.

RETRY-WORTHY vs NOT (deterministic classifier, NOT an LLM judge):
  RETRY-WORTHY: production_caller_check DARK symbol ("call new fn F from seam S"); allowed_write_violation ("you wrote outside scope; allowed set is X" — but M9 should stop this recurring by populating the set up front); a single failed acceptance/test with a named assertion; a maintainability MR-1 file-size crossing ("split file X").
  NOT RETRY-WORTHY (a retry here is a BLIND retry = anti-spiral): a compile error the worker already saw its own compiler emit (no new information); a semantic gate fail that needs capability the model lacks (route to M6 paid); anything where the finding is not machine-extractable.

SEAMS (confirm against source): gate_runner.py emits the verdict + finding; conductor_loop.py owns the disposition to NEEDS_REVIEW (add the GUIDED_RETRY branch there, gated on attempts + a finding-is-actionable predicate); the SSB already journals gate verdicts. worker_adapter / the dispatch prompt builder injects the constraint directive on the retry.

GROUNDING: ELAI I-20 budgeted-mutation "counterexample directives" + I-5 pristine-context injector (overview §13); the failure-split (GATE_PASS=16 partial) from teikoku-glm-orchestrator-merge.md GROUNDING section.


M9 — COMPILE THE WIRING_CONTRACT PRODUCTION_HOOK + ACCEPTANCE INTO THE TASK SPEC (subsumes M3)

PROBLEM. The merge plan's failure-split: 37 of 40 stuck tasks failed on real gates, and the dominant class on this corpus is wire-dark (a new pub Rust fn with no production caller = production_caller_check DARK = BLOCK, per D-030). The ELAI plan files ALREADY name the fix: every WIRING_CONTRACT carries a PRODUCTION_HOOK seam (the exact file+fn where the new symbol must be called from) plus produces/consumes/observable_effect. But the backlog import drops all of it — it leaves allowed_write_set EMPTY (P2) and never hands the worker the seam. So the worker discovers the wiring requirement ONLY via gate rejection, on a free model that then can't recover.

FIX. At backlog-import time, parse each task's source plan WIRING_CONTRACT and extract THREE fields (M3 was only extracting the first):
  (a) allowed_write_set  <- "Reads (mandatory)" + produced files   [this is M3]
  (b) production_hook     <- the PRODUCTION_HOOK seam (file+fn)      [NEW]
  (c) acceptance          <- observable_effect / the gate's check   [NEW]
Then the worker dispatch injects an up-front CONSTRAINT BLOCK:
  "ALLOWED FILES: <write-set>. WIRE your new public symbol into <production_hook> in THIS changeset. The audit will run production_caller_check + <acceptance>; a DARK symbol is a BLOCK."
This turns wire-dark greenfield into ACTIVATION-shaped work — and the owner's own measured note is that free workers succeed on activation, struggle on greenfield. You cannot make glm smarter; you can make the task narrower and pre-wired so glm's pass rate rises.

SEAMS (confirm against source): the backlog-import path that populates tasks from ELAI plan files (the one leaving write-set empty today — teikoku-glm-orchestrator-merge.md P2/M3 names it); the WIRING_CONTRACT format is known from wiring-enforcement.md + production-caller-gate.md (produces/consumes/reachable_from/observable_effect/PRODUCTION_HOOK). worker_adapter._worker_env / the dispatch prompt builder injects the block.

WHY M9 SUBSUMES M3: identical import seam, one extra parse of two more fields + one dispatch-prompt block. Implement M9 and M3 is delivered. It also closes P2 (empty write-set deny-all) as a byproduct — a task whose scope is populated from its own contract is never any([])=deny-all.

GROUNDING: D-030 production-caller gate (PRODUCTION_HOOK is a NUMBERED deliverable in the executor dispatch) + wiring-enforcement PRE_GATE (vertical slice: producer + consumer + smoke in one unit). The corpus HAS these contracts; the walk just isn't reading them into the task.


HOW M8 + M9 COMPOSE WITH M1-M7

  - M9 makes the executed tasks carry their landable bar up front (scope + seam + acceptance) -> higher first-pass rate, no empty-set BLOCK.
  - M8 catches the ones that still fail on a machine-actionable finding and gives ONE guided free retry -> flips the marginal tasks, before M6's paid escalation.
  - M1 (precheck) still routes the DOMINANT already-shipped mass to certify-not-execute; M8/M9 only touch the genuine-remainder tasks M1 leaves for execution.
  - M6 lane split unchanged: M8 is the free-lane guided retry that runs BEFORE paid escalation, lowering paid load.

MEETS THE HARNESS FILE: M9 defines WHAT to hand the worker (scope+seam+acceptance). The harness-fix file (teikoku-harness-fix-modes-2026-07-05.md, PART B) makes the harness DELIVER that block up front (pre-read the declared files, hand the scope) instead of gating on proof the worker read it. File 1 compiles the contract into the task; File 2 makes the harness deliver it. Same dispatch seam.


SEQUENCING

1. M9 (import extracts write-set + seam + acceptance; dispatch injects the constraint block). Closes P2, attacks the wire-dark failure class. Highest single leverage on the execute path.
2. M1 (precheck routing) in parallel — the recognition mass that lands the bulk of commits.
3. M8 (guided one-retry) once tasks actually run and produce findings.
4. M6 lane split as the backstop for genuine-capability failures.
M8/M9 do not change the frozen gates (R8) — they change the IMPORT and the DISPATCH, which are not in the gate manifest. Confirm that before implementing.


HONEST LIMITS

  - M8's win is bounded to machine-actionable findings; a genuine capability gap (glm can't write the adversarial test at all) is not retry-able and correctly routes to M6. Do not oversell M8 as "makes free-glm pass hard tasks."
  - M9 depends on the ELAI plans actually carrying a well-formed PRODUCTION_HOOK; some older plan units may not. Where the seam is absent, the task ships without the wiring directive and falls back to today's behavior (a WIRE-DARK marker or M6). Measure the fraction of tasks whose contract yields a seam before trusting M9's coverage.
  - Neither item is verified against live TEIKOKU source; both name seams from handoffs. Confirm the import path + WIRING_CONTRACT parse + dispatch-prompt builder exist as described before writing code.
