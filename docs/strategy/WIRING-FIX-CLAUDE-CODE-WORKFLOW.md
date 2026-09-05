WIRING FIX -- THE CLAUDE CODE WORKFLOW (the .claude harness)
Written 2026-06-02. Grounded in WIRING-ENFORCEMENT-RESEARCH-2026-06-02.md (citations [n] refer to its
SOURCES list) and in code read this session (file:line refs are verified, not assumed).
Scope: how the .claude/ harness that drives work on ELAI must change so that work done FROM HERE cannot
ship wire-dark code. Companion: WIRING-FIX-ELAI.md (the same problem inside ELAI's own runtime).

==============================================================================
0. THE ONE-SENTENCE DIAGNOSIS
==============================================================================
The harness's done-gate lives entirely inside the agent's own write boundary -- the agent writes the code,
the unit test, AND the WIRING_CONTRACT block, then a checker grades that self-authored evidence -- so the
gate can always be satisfied without the feature being wired. Fixing it means moving the proof onto an
artifact the agent did not write (an SSB row emitted by the real `elai` binary), making missing evidence a
hard FAIL, and freezing the gate scripts so the executing agent cannot rewrite its own rubric. [2][6][8][3]

==============================================================================
1. WHAT IS BROKEN TODAY (verified)
==============================================================================
Two parallel implementations exist and they DISAGREE, and both point at a dead path:

  A. .claude/tools/wiring_contract_check.py -- the script the F-plan doctrine calls the done-gate.
     - run_trigger (line 568): no `trigger` field -> status SKIP (line 571-572). SKIP is not FAIL, so a
       contract that simply omits the trigger PASSES with zero execution. This is the exact escape hatch
       the doctrine claims to have closed. [8 evaluator-tampering-by-omission]
     - --run-trigger is opt-in (line 638, store_true). Without the flag, ZERO execution happens and the
       verdict can be PASS on static checks alone. The "executed, not read" guarantee depends on a human
       remembering a flag. [research Rule 3: no opt-in flags on the gate itself]
     - expect_ssb when the db is not found -> status UNVERIFIED (line 599-600). UNVERIFIED is excluded from
       the FAIL test (line 668), so it PASSES. And the db is NEVER found: _find_ssb_db (line 526-537)
       checks a primary path that does not exist and then globs sessions/<entry>/ssb.db while the real db
       sits at sessions/ssb.db -- verified _find_ssb_db() returns None on this machine. The single
       most-emphasised proof in the doctrine (the SSB row) is a dead no-op. [research Rule 2]
     - expect_artifact greps the trigger's own stdout/stderr (line 584-596), not an on-disk artifact. A
       command that merely prints the pattern passes. stdout is agent-controlled output. [research Rule 4/5]
     - consumes is a WARN floor (line 343) and is excluded from the failed-verdict computation entirely
       (lines 665-668). Producer-without-consumer never fails. [research Rule 6, W-8 unenforced]

  B. .claude/agents/wiring-contract-validator.md -- the agent twin.
     - Step 8d FAILs on a missing SSB row (stricter than the script's UNVERIFIED), BUT step 8a defaults to
       the same nonexistent path .elai_cc/.elai/ssb.db, then falls back to grepping session logs. Two
       implementations, two different fail behaviours, both anchored to a broken path.

Net: the gate advertises fail-closed and is fail-open in the dimension it shouts loudest about. That is a
self-violation of the doctrine's own W-10 ("MUST NOT advertise fail-closed if it is fail-open").

==============================================================================
2. THE FAIL-CLOSED PRINCIPLE (the rules every fix below obeys)
==============================================================================
From research section 4, the rules that matter for the harness:
  R1  Missing evidence = FAIL, never SKIP/UNVERIFIED/WARN. Absence of proof is proof of absence here.
  R2  The gate must not depend on a resource that can be silently absent. db-not-found = FAIL, not skip.
  R3  No opt-in flag on the gate. It runs unconditionally or it is not a gate.
  R4  Measure the real objective, not a proxy. The objective is "a real entrypoint ran and left a trace
      the agent did not author." For this harness that artifact is an SSB row in the real .elai_cc db.
  R5  Sound reference, not leaf-name grep. A name appearing in text is not a call.
  R6  Producer-without-(non-test)-consumer = build break, propagated to the verdict, not a warning.
  R7  Test-file callers do NOT satisfy production reachability (Go deadcode -test root-set insight [9]).
  R8  The gate script is frozen and read-only to the executing agent. An agent that can rewrite the gate
      on the same run it is evaluated by will always write itself to PASS. [2][6][8]
  R9  A new default-OFF flag fails the gate unless it is registered with a type, expiry, and promotion
      plan. Default-off is not a terminal state. [37][38][40]

==============================================================================
3. CONCRETE FIXES TO wiring_contract_check.py
==============================================================================
These are surgical edits to the existing script. Each maps to a rule above. (Single-file patch-class
change; do NOT skip the standard explorer pass if it grows beyond the verdict/path logic.)

3.1  Make every contract field required (R1). In main() after parse, before checks: if any of
     produces_symbols / consumes_symbols / reachable_from / observable_effect is absent OR
     observable_effect lacks `trigger` + `expect_ssb`, set verdict=FAIL with an explicit
     "contract incomplete: <field>" reason. Delete the SKIP-on-no-trigger path in run_trigger (571-572):
     a contract with no trigger is FAIL, not SKIP.

3.2  Remove --run-trigger as opt-in (R3). The trigger ALWAYS runs. Either drop the flag and always execute,
     or keep the arg for local debugging but have CI/the b-plan-runner always pass it AND have main()
     refuse to emit PASS if the trigger did not run. The done-gate path never has the static-only branch.

3.3  db-not-found = FAIL (R2) and fix the path bug. In run_trigger (599-600): if _find_ssb_db() returns
     None, verdict-contributing status = FAIL ("expect_ssb declared but no ssb.db found -> feature never
     ran"). And fix _find_ssb_db (526-537): the real db is at <root>/sessions/ssb.db, not
     <root>/sessions/<entry>/ssb.db. Resolve the actual path (check the direct file, the
     .elai_cc/.elai_cc/sessions/ssb.db and .elai_cc/sessions/ssb.db locations found this session) and add
     a startup self-test that errors loudly if the configured SSB path does not exist when expect_ssb is
     used anywhere in the run. A dead evidence path must scream, not shrug.

3.4  expect_artifact must check disk, not stdout (R4/R5). Change the artifact leg to stat/glob a real file
     path produced by the trigger, OR demote expect_artifact to advisory and make expect_ssb (the off-agent
     SSB row) the load-bearing proof. stdout matching alone can never satisfy the verdict.

3.5  consumes feeds the verdict (R6). Add the consumes result to the `failed` computation (lines 665-668):
     a produces_symbol whose only callers are test-file callers (RTA: tests/, contracts/tests/, #[cfg(test)])
     -> FAIL. _production_caller (line 370) already excludes test paths; make its result block the verdict,
     not just sharpen the report. NOTE the unsoundness in 3.7.

3.6  Fail on empty (R1, ArchUnit failOnEmptyShould [25]). If the checker is pointed at a contract set and
     finds ZERO contracts to evaluate, exit nonzero. A gate that vacuously passes on no input is the same
     bug class as SKIP-passes.

3.7  Honest limit -- the leaf-name grep is unsound (research HIGH risk; SCIP has no call-graph edges).
     _def_hits / _production_caller match a symbol name in elai.map; a dyn-Trait method called only via
     vtable has zero direct references and will be FALSELY flagged dead, while a same-named symbol in the
     wrong file can FALSELY pass. Do NOT pretend this is sound. Two mitigations:
       (a) Require an ENTRYPOINTS map (a checked-in list of real entrypoints incl. trait-dispatch sites);
           a produces that "implements dispatched trait T, T reached from entrypoint E" is accepted as wired.
       (b) Treat the static check as necessary-but-not-sufficient and make the RUNTIME SSB-row check
           (3.3) the authoritative proof. Runtime coverage catches what the grep cannot. [9][21]

==============================================================================
4. RECONCILE THE TWO IMPLEMENTATIONS -- ONE SOURCE OF TRUTH
==============================================================================
Today the script (wiring_contract_check.py) and the agent (wiring-contract-validator.md) implement the
same idea with different fail semantics and the same broken path. Pick ONE:
  - Make wiring-contract-validator.md CALL wiring_contract_check.py and report its exit code + JSON, rather
    than re-implementing the SSB query in prose. The agent becomes a thin runner + evidence narrator; the
    script is the single arbiter. This removes the UNVERIFIED-vs-FAIL divergence by construction.
  - Fix the SSB path in ONE place (the script) and have the agent consume it.
  - Update both to the fail-closed rules in section 2 in the same change so they cannot drift again.

==============================================================================
5. THE STRUCTURAL FIX THAT MATTERS MOST -- FREEZE THE GATE (R8)
==============================================================================
This is the highest-leverage change and it is cheap. The reason the gate keeps being fail-open is that the
same run that is being evaluated can edit the evaluator. Berkeley RDI and ImpossibleBench show this gets
gamed at rates approaching 100% once the model is capable [2][6].
  - Mark .claude/tools/wiring_contract_check.py, the smoke-harness scripts, and ENTRYPOINTS/allowlist files
    as gate-owned. Add a CI/Stop-hook step: `git diff --exit-code` over that fileset during any
    orchestrate / b-plan run. If the executing run modified its own gate, BLOCK and require a separate,
    human-reviewed change. The gate is authored ONCE, out-of-band, never regenerated per-plan.
  - The existing Stop hook (PreToolUse/PostToolUse/Stop in .claude) is the natural enforcement point; add a
    "gate-files-unchanged" assertion to it. Mirrors ELAI's own LOCKED_KEYS / LockedKeysAuditor pattern.

==============================================================================
6. PREVENTION AT PLANNING TIME -- THE WALKING-SKELETON RULE
==============================================================================
Detection is the backstop; prevention is cheaper. The walking-skeleton / tracer-bullet / steel-thread
literature [34][35][36] all define "done" as a thin slice that runs end-to-end from a real entrypoint to an
observable effect -- never "ship the producer now, wire the consumer later" (which the F-plan named W-8 and
the checker fails to enforce).
  - planning-workflow SKILL.md + b-plan-runner: add a PRE_GATE that REFUSES a baby step / B-N whose
    deliverable is a producer without its in-plan consumer AND its smoke trigger in the SAME unit of work.
    "Producer now, consumer in a later step" is rejected at decomposition time, not discovered at audit.
  - The prompt-transform / scope-planner step must phrase each task as a vertical slice: "add fn X AND call
    it from entrypoint Y AND assert the SSB row Z," never "add fn X."

==============================================================================
7. THE SMOKE MANIFEST -- DECLARE THE ENTRYPOINT AND THE OFF-AGENT ARTIFACT
==============================================================================
Add .elai_cc/smoke-manifest.toml (gate-owned, frozen per section 5). Every B-N that ships a feature adds:
  [[feature]]
  plan          = "B-N"
  entrypoint    = "<CLI subcommand | IPC command | FSM transition | hook | Tauri command>"
  expect_ssb    = "<action_type the feature emits when the real entrypoint runs>"
  flag_default  = "on"            # or a wire-dark-allowlist.toml reference if legitimately a kill-switch
The auditor (NOT the main LLM) verifies: the manifest entry exists for the new feature; the wiring
contract's trigger actually drives the declared entrypoint; and the declared expect_ssb row landed in the
real ssb.db within the window. A missing manifest entry for a new feature = BLOCK. [research Layer 3]

==============================================================================
8. ROLE BOUNDARIES (who may author what)
==============================================================================
  - The wiring gate is the AUDITOR's job. planning-workflow already says "main model never runs tests;
    the auditor is the sole test runner." Extend: the auditor (or the wiring-contract-validator agent) runs
    wiring_contract_check.py; the main LLM never declares wiring PASS.
  - The smoke-test harness and any trace-assertion test are AUDITOR-authored repo artifacts, never written
    by the same agent producing the feature (research residual risk: an agent that writes its own smoke
    test can route around the real path). This is the harness analog of "freeze the gate."

==============================================================================
9. NO DEFAULT-OFF GRAVEYARD (R9)
==============================================================================
A new ELAI_*_ENABLE or cargo feature defaulting to false/OFF fails the gate UNLESS it has a row in a
checked-in wire-dark-allowlist.toml carrying: type (kill_switch | experiment), owner, expiry_date,
promotion_plan_id, and a smoke test that exercises the ON path. [37 time-bomb][38 STALE_ALL_FALSE][40]
This directly enforces F-plan W-2 ("default-off env gate is not a terminal state"), which the night-run
proved was the most common way a "shipped" plan stayed dark.

==============================================================================
10. FILE-LEVEL CHANGE LIST (what to touch in .claude/)
==============================================================================
  - tools/wiring_contract_check.py        : section 3 fixes (fail-closed verdict + SSB path + empty-fail).
  - agents/wiring-contract-validator.md    : section 4 (call the script; drop the divergent re-impl).
  - skills/b-plan-runner/ + planning-workflow SKILL.md : section 6 PRE_GATE (walking-skeleton refusal),
                                              section 7 manifest verification in the AUDITING sub-state.
  - rules/ (new or existing)               : R8 freeze-the-gate Stop-hook assertion; R9 default-off policy.
  - NEW .elai_cc/smoke-manifest.toml        : section 7 (gate-owned).
  - NEW wire-dark-allowlist.toml            : section 9 (gate-owned).
  - tools/ (new)                           : entrypoint-coverage runner that drives the real `elai` binary
                                              and parses the SSB db -- the off-agent evidence layer the
                                              harness currently lacks (mirrors ELAI Layer 3, see companion).

==============================================================================
11. RESIDUAL RISK (do not oversell this)
==============================================================================
  - No sound static reachability for Rust (no Go-style deadcode). Static legs are approximations; the
    runtime SSB-row / coverage leg is the real backstop. [research 3.1, OPEN QUESTIONS]
  - dyn-Trait / IPC-string dispatch is invisible to SCIP/elai.map -> false positives unless ENTRYPOINTS
    enumerates dispatch sites. [research HIGH risk]
  - The smoke harness only proves what it exercises. Auditor-authored manifests + coverage-diff on NEW
    functions (count==0 = BLOCK) bound this, but a lazy manifest can still under-cover. Track coverage of
    the manifest itself.
  - Wire-dark has no external benchmark [45]; measure your own rate before/after (wire-dark plans / total)
    so the fix is evaluated on data, not vibes.
End of WIRING-FIX-CLAUDE-CODE-WORKFLOW.md.
