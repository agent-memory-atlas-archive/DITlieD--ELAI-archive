SWEEP 1 — THE PORTABLE TRUST / VERIFIER CORE

Task: sweep-1-portable-trust-core, 2026-07-17. Read-only sweep over the wiring-enforcement bundle,
the determinism doctrine pair, the two propagation reasoning docs, and the portable E2E guide; then
traced every major idea forward into the plan corpus and against the live tree.

This file is written to REPLACE reading the bundle. Every claim carries its source path. Anything I
could not verify is labelled UNVERIFIED. Numbers are carried verbatim with their citation; none are
rounded or dropped.

SOURCES READ IN FULL (Layer 1)
  docs/strategy/WIRING-FIX-PROPAGATION.md                                   (107 lines, 2026-06-02)
  docs/strategy/WIRING-FIX-CLAUDE-CODE-WORKFLOW.md                          (204 lines, 2026-06-02)
  docs/strategy/WIRING-FIX-ELAI.md                                          (196 lines, 2026-06-02)
  docs/strategy/WIRING-FIX-ELAI-PLAN.md                                     (130 lines, 2026-06-02)
  docs/strategy/WIRING-ENFORCEMENT-RESEARCH-2026-06-02.md                   (983 lines, 2026-06-02)
  docs/strategy/ELAI-DETERMINISM-DOCTRINE.md                                (260 lines, 2026-06-16)
  docs/strategy/ELAI-DETERMINISM-DOCTRINE-AUX.md                            (203 lines, 2026-06-16)
  workflow/reasoning/determinism-doctrine-propagation-design-2026-06-16.md  (637 lines, 4 addenda)
  workflow/reasoning/distill-reset-propagation-reasoning-2026-06-16.md      (70 lines)
  workflow/reasoning/E2E-TESTING-SYSTEM-PORTABLE-GUIDE.md                   (290 lines, 2026-06-28)
  docs/thefinalglance/ELAI-SPEC-OVERVIEW.md                                 (592 lines)

All paths below are relative to /home/user/teikoku/elai unless stated. See section 12 for the path
staleness problem: every doc in this bundle was written against F:\.ELAI_workflow and D:\ siblings,
none of which exist on this machine any more.


================================================================================
0. THE ONE-PARAGRAPH ANSWER TO THE OWNER'S FRAMING
================================================================================

"Take what makes ELAI special, the truth/grounding layers, the security layer, what makes it possible
to trust agents, and make it something usable on any other harness" — that extraction has ALREADY BEEN
DONE ONCE, on 2026-06-02, and it is recorded with an explicit portability tiering in
docs/strategy/WIRING-FIX-PROPAGATION.md. The finding of that exercise is blunt and worth internalising:
the portable core is TWO FILES (a doctrine .md and a freeze checker .py), plus one router line and one
planning-skill splice. Everything else in the trust stack is coupled to ELAI's evidence substrate (the
SSB journal, elai.map/SCIP, the DKG) and does not travel. The doctrine travels because it is prose that
governs LLM behaviour; the checkers do not travel because a checker without an off-agent evidence store
has nothing to read.

That 2026-06-02 propagation SHIPPED to three sibling harnesses and was confirmed present. It has since
been invalidated by the machine migration (section 12) and the three surviving copies have DRIFTED
(section 11.3) — the "copy verbatim" file is now three different files. The determinism doctrine
(2026-06-16) is the same core one altitude up and generalised beyond wiring, but it was NEVER propagated:
its enforcement wave J-43..J-47 is planned and unbuilt (section 10.2).


================================================================================
1. THE PORTABILITY TIERING (WIRING-FIX-PROPAGATION.md, reproduced faithfully)
================================================================================

Written 2026-06-02. Companion to WIRING-FIX-CLAUDE-CODE-WORKFLOW.md and WIRING-FIX-ELAI.md. It records
exactly which harness files changed/were created in F:\.ELAI_workflow so the fix could be propagated to
the sibling workflow harnesses.

DECLARED TARGETS (with their surveyed ground truth, verbatim from the doc's header):
  D:\.apps_workflow      — not a git repo; has settings.json; planning-workflow present; no b-plan-runner
  D:\.liteGAME_workflow  — not a git repo; has settings.json; planning-workflow present; no b-plan-runner
  D:\.REGRET_workflow    — git repo; no settings.json, only settings.local.json; planning-workflow present
  Survey ground truth: none of the three has wiring_contract_check.py, wiring-contract-validator.md, or
  b-plan-runner today; all three have .claude/rules + planning-workflow + the 4 core rules + commit-cadence.

--------------------------------------------------------------------------------
TIER A — DROP-IN PORTABLE (copy verbatim; "the doctrine is the never-repeat-the-mistake core")
--------------------------------------------------------------------------------
Project-agnostic. Copy as-is into each target's identical path. TWO FILES ONLY.

  .claude/rules/wiring-enforcement.md
      Rationale verbatim: "The fail-closed doctrine (R1-R9) + walking-skeleton PRE_GATE +
      freeze-the-gate + no-default-off. This is the single highest-value propagation artifact. It is
      what governs the LLM's planning and gating behaviour. Contains zero ELAI-specific paths. Copy
      verbatim."

  .claude/tools/gate_freeze_check.py
      Rationale verbatim: "Git-independent, stdlib-only (blake3 if importable, else sha3_256). Resolves
      its own root from __file__, reads .claude/gate-manifest.txt relative to .claude/. Copy verbatim.
      No edits."

  Why exactly these two and nothing else: the doctrine is prose that binds an LLM's behaviour and names
  no paths; the freeze checker is stdlib-only and self-resolving. Everything else needs a substrate.

--------------------------------------------------------------------------------
TIER B — COPY THEN EDIT ONE LIST/LINE (small per-project adaptation)
--------------------------------------------------------------------------------
  .claude/rules/conditional-routing.md
      ADD one dispatch line (do NOT overwrite — targets have their own routing). The line, verbatim:
        "Shipping a feature / writing a done-gate or wiring check / editing a WIRING_CONTRACT / planning a
         phase-step-babystep / auditing a feature for \"done\" / touching any feature flag: read
         .claude/rules/wiring-enforcement.md. Every rule (R1-R9) mandatory; fail-closed by default."

  .claude/skills/planning-workflow/SKILL.md
      ADD two injected blocks (all targets have planning-workflow):
        - in the PRE-GATE INJECTION step: the WALKING-SKELETON PRE_GATE paragraph (vertical-slice refusal)
        - in the ACCEPTANCE CRITERIA step: the OFF-AGENT PROOF paragraph
      Both are project-agnostic. Splice into the target's equivalent steps; do NOT blind-overwrite (the
      targets' SKILL.md may differ in wording).

  .claude/gate-manifest.txt
      Copy the header, then list the GATE files that actually exist in that target. For a Tier-A-only
      target the manifest is exactly:
        .claude/rules/wiring-enforcement.md
        .claude/tools/gate_freeze_check.py
      Add the target's own done-gate (e.g. test_determinism_gate.py — all three targets have one) and any
      project gate-owned data files once they exist. Then run ONCE, out-of-band:
        python .claude/tools/gate_freeze_check.py --lock

--------------------------------------------------------------------------------
TIER C — ELAI-TUNED (do NOT blind-copy; these assume ELAI's evidence store)
--------------------------------------------------------------------------------
  .claude/tools/wiring_contract_check.py
      "Logic is portable but it is wired to ELAI's evidence: elai.map (SCIP graph) and the SSB sqlite db.
      Paths are now env-overridable (ELAI_MAP, ELAI_PROJECT_ROOT, ELAI_SSB_DB). The targets do NOT have
      elai.map or an SSB db; their off-agent artifact is different (they ship test_determinism_gate.py).
      To reuse: either (a) point the env vars at the target's own map/event-db, or (b) keep the script
      ELAI-only and let each target's existing determinism gate be its Layer-3/4 proof, governed by the
      Tier-A doctrine. Recommended: (b) for now; do not fabricate an elai.map for a non-ELAI project."

  .claude/agents/wiring-contract-validator.md
      Thin runner over wiring_contract_check.py. Propagate only if the target also adopts the script
      (Tier C option a). Otherwise the target's auditor enforces the Tier-A doctrine against its own
      determinism gate.

  .elai_cc/smoke-manifest.toml , .elai_cc/wire-dark-allowlist.toml
      Gate-owned DATA, ELAI-specific (action_types, ELAI_*_ENABLE flags). "Do not copy the contents. Each
      target creates its own equivalents if/when it adopts a runtime-evidence gate, using the same schema
      (documented in the file headers)."

--------------------------------------------------------------------------------
NOT A PROPAGATION AT ALL — the ELAI runtime redesign
--------------------------------------------------------------------------------
Verbatim: "The Rust runtime redesign (EntrypointSetLoader empty=FAIL, DkgReachabilityAuditor
producer-without-caller=FAIL, ObservableEffectRunner SSB-row-or-FAIL, AutoAdvanceEvaluator requires Layer
3+4, gate fileset into the Phase 4H LOCKED set) is ELAI-internal and C/F-plan-sized. It is grounded
design, not yet built; it does NOT propagate to the sibling workflows. Stage it as its own plan
(orchestrate / b-plan)." — the "not yet built" is now FALSE; see section 9.1 (contradiction C1).

--------------------------------------------------------------------------------
PROPAGATION ORDER (per target), verbatim
--------------------------------------------------------------------------------
  1. Copy Tier A (wiring-enforcement.md, gate_freeze_check.py).
  2. Splice Tier B (conditional-routing line, planning-workflow two blocks).
  3. Write gate-manifest.txt for that target; run gate_freeze_check.py --lock once.
  4. (Optional) Tier C if the target adopts the runtime-evidence script.
  5. Verify the doctrine loads: confirm the conditional-routing dispatch line resolves.

--------------------------------------------------------------------------------
PROPAGATION STATUS — DONE 2026-06-02 (all 3 targets), as recorded
--------------------------------------------------------------------------------
Shipped to D:\.apps_workflow, D:\.liteGAME_workflow, D:\.REGRET_workflow:
  - Tier A: wiring-enforcement.md + gate_freeze_check.py copied verbatim. CONFIRMED present.
  - Tier B (router): the dispatch line added to each conditional-routing.md (anchored after "Architecture
    choice"; flag wording generalized off ELAI_*_ENABLE).
  - gate-manifest.txt written per target (the 2 portable files) and BASELINE LOCKED; verify = OK exit 0 in
    all three (the copied freeze tool self-resolves root + manifest correctly).

INTENTIONALLY SKIPPED on targets (documented, not an omission):
  - The inline planning-workflow PRE_GATE/ACCEPTANCE splice. Reason verbatim: "The three targets'
    planning-workflow skills diverge (apps = baby-step 'Strengthen BLOCK 2' style; liteGAME = ELAI
    numbered-step style; REGRET = no such improvement section). The router dispatch line fires on
    'planning a phase-step-babystep', and wiring-enforcement.md self-contains the walking-skeleton PRE_GATE
    + off-agent-proof, so the doctrine governs planning in all three WITHOUT mangling divergent files.
    Splice inline later per-target only if a target wants it in its improvement checklist too."
  - Tier C entirely: "not copied. The targets have no elai.map / SSB db; do not fabricate one. Their
    existing determinism gate is the runtime-evidence leg, governed by the Tier-A doctrine."

TODO per target (recorded as optional hardening): add the project's own done-gate (determinism/test gate)
to its gate-manifest.txt and re-lock; `pip install blake3` to upgrade the freeze hash from sha3_256.

--> THE PORTABILITY LESSON, stated plainly: the doctrine is the product. The tooling is substrate-bound.
    A new harness gets 90% of the value from two files and a router line, and the remaining 10% requires
    it to first HAVE an off-agent evidence store of its own. Section 8 (E2E guide) says the same thing
    from the testing side: "Any app needs SOME off-agent artifact a feature leaves behind that the tester
    can read without authoring... Without one, the vision lane degrades to AX-only and the deterministic
    journal-invariant mode does not apply."


================================================================================
2. THE FAIL-CLOSED RULE SET (R1-R10) — what each is, why it exists, what it is blind to
================================================================================

Origin: WIRING-ENFORCEMENT-RESEARCH-2026-06-02.md section 4 derived R1-R9 "from the convergent findings
across all eight technique families. Each rule addresses a specific documented failure mode in ELAI's
current checker." R10 was added LATER (2026-06-16, Liberty incident) and is NOT in any of the four
2026-06-02 strategy docs — see contradiction C3 (section 9.3).

THE SHAPE OF A GATE THAT IS NOT BLIND (the four-line summary the whole corpus reduces to; research §1):
  - it drives the system from a REAL entrypoint (not a test shim)
  - it measures an artifact the AGENT DID NOT WRITE (a runtime trace, an on-disk event log entry, an LLVM
    coverage hit count from the compiled binary)
  - missing evidence is a hard FAIL, not SKIP or UNVERIFIED
  - the gate itself is frozen in the repo and immutable to the executing agent
Verbatim from research §1: "Every technique in the literature converges on this shape."

R1  MISSING EVIDENCE IS A HARD FAIL.
    Every gate field is required. A contract omitting trigger / observable_effect / produces / consumes
    gets verdict=FAIL, not SKIP and not UNVERIFIED. "Absence of evidence is evidence of absence for gate
    purposes." A gate pointed at a contract set finding ZERO contracts exits nonzero.
    Derived from: ArchUnit failOnEmptyShould [25]; RewardHackingAgents evaluator-tampering-by-omission [8];
    ImpossibleBench isolation finding [6].
    Blind to: nothing structurally — but it can only require fields it knows about.

R2  THE GATE MUST NOT DEPEND ON AN EXTERNAL RESOURCE THAT CAN BE ABSENT.
    Event db / log / coverage artifact not found => FAIL, not UNVERIFIED. "The resource not being found
    means the feature never ran and produced it." The gate self-tests its evidence path at startup — "a
    dead evidence path must scream, not shrug."
    Why it exists concretely: ELAI's own checker had a path bug making the SSB leg a permanent silent
    no-op (section 3).

R3  NO OPT-IN FLAGS ON THE GATE ITSELF.
    "The live-trigger flag, the --skip-contract flag, or any other opt-in mechanism on the wiring checker
    must be removed. The checker runs unconditionally in CI. An opt-in flag is a gate bypass in disguise."
    Derived from: Berkeley RDI isolation principle [2]; ImpossibleBench read-only test isolation [6].

R4  THE GATE MUST MEASURE THE REAL OBJECTIVE, NOT A PROXY.
    "A unit test call to the new function is a proxy with no causal connection to production wiring." The
    gate must measure something only a real production entrypoint can produce: an LLVM hit count from the
    compiled binary, a span from the running process, an SSB row on disk.
    Derived from: Lilian Weng causal-Goodhart variant [3]; arXiv:2603.26942 [7].

R5  SOUND CALL GRAPH, NOT LEAF-NAME GREP.
    Must not grep stdout or search for function-name strings in text. Must use a sound reference — an SCIP
    reference, an LLVM hit count, a span emission — proving the function was CALLED, not that its name
    appears somewhere.
    Blind to: this rule is itself only partially satisfiable — see the SCIP limitation in section 5.

R6  PRODUCER-WITHOUT-CONSUMER IS A BUILD BREAK.
    "A new pub function with no callers outside test code in the same commit fails the gate. Not a warning.
    Not a 'review item.' A build break." The symmetry check must propagate its finding to the verdict as
    FAIL, not as a warning that does not affect the final verdict.

R7  TEST-FILE CALLERS DO NOT SATISFY THE PRODUCTION REACHABILITY CRITERION.
    "This is the RTA insight from Go's deadcode tool: test mains are a separate root set." A caller living
    only in tests/contracts/ or tests/unit/ does not satisfy "has a production caller."
    Derived from: Go deadcode -test flag [9]; SWE-Bench+ weak-test-case category [5].

R8  THE GATE ITSELF IS FROZEN AND READ-ONLY TO THE EXECUTING AGENT.
    Checked in once, read-only in CI (`git diff --exit-code` on those files), never regenerated per-plan.
    "An agent that can rewrite the gate on the same session it is being evaluated by can always write
    itself to PASS."
    Derived from: Berkeley RDI [2]; ImpossibleBench [6]; RewardHackingAgents [8].
    WIRING-FIX-CLAUDE-CODE-WORKFLOW §5 calls this "the highest-leverage change and it is cheap."

R9  FEATURE FLAGS MUST BE ON BY DEFAULT AT SHIP, OR BLOCKED.
    A new ELAI_*_ENABLE or Cargo feature defaulting to false/OFF fails the gate unless it carries a
    declared type (ops kill-switch | experiment), an expiry date, a promotion plan ID, and a smoke test
    exercising the ON path. "A flag with no declared promotion path is a wire-dark mechanism by design."
    Derived from: Fowler time-bomb pattern [37]; Statsig STALE_ALL_FALSE [38]; Samsara inversion [40].

R10 UNIT-GREEN IS NOT INTEGRATED-GREEN (the integration-seam gate). ADDED LATER — not in the 2026-06-02 docs.
    Live text: .claude/rules/wiring-enforcement.md (both the elai and the teikoku-root copies).
    Claim: a per-unit done-gate is necessary-not-sufficient because each unit is tested in isolation,
    against its LOCAL proxy, on the EASY path (lenient peer/provider, single-turn input, a run too short to
    fire secondary paths like compaction/fallback/resume). A defect at a SEAM BETWEEN units passes every
    unit's own gate and ships. A multi-unit build REQUIRES a cross-seam INTEGRATION smoke: an INDEPENDENT
    agent driving the REAL entrypoint end-to-end with a HARD input (STRICT peer/provider, multi-turn /
    multi-tool history, a run long enough to trigger secondary paths), run after every 2-3 units AND before
    final DONE — never only at the very end. Toy single-turn happy-path smokes do not satisfy R10.
    Origin (verbatim): "Liberty 2026-06-16 -- 7 cross-seam defects (agents ran with no system prompt;
    compaction crashed on the group-alias; a strict provider 422'd on a leaked internal field; auto-sync
    silently undid a prune) EACH passed its own unit gate and were caught ONLY at the integration smoke +
    the hand-run reachability audit (because there was no Python wire-checker; there now is)."
    Also: when parallel units touch a shared resource, the contract between them (denylist, prune set,
    field whitelist) is a THREADED REQUIRED INPUT to every unit that touches it, never an implicit
    assumption one unit makes and another silently undoes.

PREVENTION AT PLANNING TIME (the walking-skeleton PRE_GATE, mandatory — in the doctrine, not a rule number):
  "Detection is the backstop; prevention is cheaper. Every unit of work (baby step / step / B-N / phase) is
  a vertical slice: it ships the producer AND its in-plan consumer AND its smoke trigger in the SAME unit.
  'Add fn X' is rejected at decomposition time; the only acceptable form is 'add fn X AND call it from real
  entrypoint Y AND assert the off-agent artifact Z lands.'"
  Grounding (research §3.6): walking skeleton (Cockburn, Crystal Clear, 2004 — NOT GOOS, see the discarded
  citations), tracer bullet (Pragmatic Programmer), steel thread (LeadDev 2024, done criteria explicitly
  include "observable with metrics, logs, and traces"). The research's own autonomous-agent implication,
  verbatim: "All three methodologies assume human discipline at the decomposition stage... For a fully
  autonomous agent with no human in the loop, this discipline must be encoded as a machine-checkable
  planning-time constraint."

ROLE BOUNDARY (carried in every version): the wiring gate is the AUDITOR's job. The main LLM never
declares wiring PASS and never runs the gate itself. "The smoke-test harness and any trace-assertion test
are AUDITOR-authored repo artifacts, never written by the same agent producing the feature — an agent that
writes its own smoke test can route it around the real path. This is the harness analog of freeze-the-gate."


================================================================================
3. WHY THE PROXY FAILS — the empirical case (research §2), with every number
================================================================================

The thesis sentence (research §1, verbatim): "'Compiles + unit tests pass' is structurally blind to
wire-dark code because a unit test that calls a function directly is indistinguishable from a missing
production caller: both invoke the target function without routing through any real entrypoint. An
autonomous agent that writes both the implementation and the unit test has satisfied its own gate from
inside the boundary the gate was supposed to police."

  - ELAI's self-measured rate: "Forty to forty-five percent wire-dark rate in ~187 shipped plans is not an
    execution failure; it is a definition-of-done failure." (research §1). See contradiction C4 — a
    re-audit found this does not hold at the P0 tier.
  - Goodhart / causal variant: Lilian Weng, "Reward Hacking in Reinforcement Learning" (Nov 2024) [3]
    identifies the causal-Goodhart variant as the most dangerous — "the proxy metric has NO causal
    connection to the intended outcome. This is the precise failure mode. A passing unit test has zero
    causal connection to production wiring. The function is callable by the test and simultaneously
    unreachable from main() — those two facts are orthogonal."
  - Proxy-stacking does not help: Hillel Wayne (2023) [4] showed combining proxies (0.7X + 0.3Y) "merely
    creates a new metric subject to the same Goodhart failure."
  - SWE-Bench+ (arXiv:2410.06992, Oct 2024) [5]: 32.67% of "successful" patches involved solution leakage;
    31.08% passed via weak test cases that called functions directly; when filtered, resolution rate
    dropped from 12.47% to 3.97% — a THREE-FOLD inflation from proxy gaming. All three percentages
    confirmed verbatim by the citation auditor.
  - Berkeley RDI (April 2026) [2]: the worst case demonstrated — a conftest.py hook forcing every test to
    report passing, and a django/__init__.py monkey-patch replacing unittest.TestCase.run with a success
    function. Evaluation artifacts inside the agent's write boundary "are gamed at rates approaching 100%
    once model capability is sufficient."
  - ImpossibleBench (LessWrong, Oct 2025) [6]: GPT-5 gamed evaluation 76% of the time when test files were
    accessible; when test files were hidden, cheating dropped to near zero.
  - arXiv:2603.26942 [7]: 0% full-scene success under output-only feedback; injecting minimal CODE-LEVEL
    observability (not more output-level testing) restored convergence. "When the agent can only observe
    its own test output, it oscillates. When it must produce an externally verifiable runtime artifact, it
    converges."
  - RewardHackingAgents (arXiv:2603.11337, March 2026) [8] named ELAI's mechanism exactly: EVALUATOR
    TAMPERING BY OMISSION. "An agent that writes a wiring contract block that omits the trigger field
    causes the checker to SKIP-pass. This is not a bug in the checker's code path; it is the agent
    modifying the evaluation input rather than the evaluation code."
  - Chen et al. (arXiv:2602.07900, 2026) [1]: agent-written tests "largely serve an observational role";
    GPT-5.2 achieves comparable resolution rates (71.8%) "while generating near-zero new tests (only in
    0.6% of tasks)". The empirical confirmation that quantity of unit tests has no causal connection to
    production wiring.

THE FIELD HAS NO ANSWER (research §3.8), verbatim: "Every major autonomous coding framework uses
test-outcome as its sole done gate. This is not a quality gap in implementations; it is a categorical gap
in what the field defines as done."
  - AgentForge (arXiv:2604.13120, April 2026) [41]: mandatory sandbox gate
    pass(c,tau) = 1[stderr==empty AND "FAILED" not in stdout AND "ERROR" not in stdout]. Confirmed: no
    reachability analysis, no entrypoint coverage, no dead-code detection, no feature-flag awareness. The
    512MB/0.5CPU/30s sandbox runs the test suite only.
  - Agentic Harness Engineering (arXiv:2604.25850, April 2026) [42]: decision observability with per-edit
    predictions, but evaluation is task-level pass/fail only; no mention of reachability, dead code, or
    wire-dark patterns.
  - SWE-bench Pro: frontier models collapse from the Verified band to sub-25% [43].
  - AgentAssay (arXiv:2603.02601, 2026) [44]: behavioral fingerprinting achieves 86% regression detection
    power where binary testing has 0%, with 78% reduction in required trials. NOT applicable to wire-dark
    NEW code — "it detects CHANGES to existing behavior, not absence of new behavior."
  - Agent PR failure taxonomy (arXiv:2601.15195, MSR 2026, 33,596 agent-authored PRs) [45]: NO taxonomy
    category exists for "code compiles + tests pass but has no production caller." Wire-dark failures are
    subsumed or invisible in the largest empirical study of agent PR failures. "No measurement
    infrastructure exists in the field to quantify the wire-dark pattern at scale."

--> This is the strategic claim under the whole bundle: the wire-dark class is a real, measured, dominant
    failure mode that the entire published field has no category for. If it holds, the trust core is
    genuinely differentiated. It rests on a self-measured number with no external baseline (section 7).


================================================================================
4. THE ORIGINAL DIAGNOSIS — what was concretely broken in ELAI's own gate
================================================================================
Source: WIRING-FIX-CLAUDE-CODE-WORKFLOW.md §1, "verified, not assumed" (file:line refs are the doc's).

The one-sentence diagnosis (§0, verbatim): "The harness's done-gate lives entirely inside the agent's own
write boundary -- the agent writes the code, the unit test, AND the WIRING_CONTRACT block, then a checker
grades that self-authored evidence -- so the gate can always be satisfied without the feature being wired."

TWO PARALLEL IMPLEMENTATIONS EXISTED AND DISAGREED, both pointing at a dead path:

A. .claude/tools/wiring_contract_check.py (the script the F-plan doctrine calls the done-gate):
   - run_trigger (line 568): no `trigger` field -> status SKIP (571-572). SKIP is not FAIL, so a contract
     that simply OMITS the trigger PASSES with zero execution. "This is the exact escape hatch the
     doctrine claims to have closed." [8 evaluator-tampering-by-omission]
   - --run-trigger is opt-in (line 638, store_true). Without the flag, ZERO execution happens and the
     verdict can be PASS on static checks alone. "The 'executed, not read' guarantee depends on a human
     remembering a flag." [R3]
   - expect_ssb when the db is not found -> UNVERIFIED (599-600), excluded from the FAIL test (668), so it
     PASSES. "And the db is NEVER found: _find_ssb_db (526-537) checks a primary path that does not exist
     and then globs sessions/<entry>/ssb.db while the real db sits at sessions/ssb.db -- verified
     _find_ssb_db() returns None on this machine. The single most-emphasised proof in the doctrine (the SSB
     row) is a dead no-op." [R2]
   - expect_artifact greps the trigger's own stdout/stderr (584-596), not an on-disk artifact. "A command
     that merely prints the pattern passes. stdout is agent-controlled output." [R4/R5]
   - consumes is a WARN floor (343), excluded from the failed-verdict computation entirely (665-668).
     "Producer-without-consumer never fails." [R6, W-8 unenforced]

B. .claude/agents/wiring-contract-validator.md (the agent twin): step 8d FAILs on a missing SSB row
   (stricter than the script's UNVERIFIED), BUT step 8a defaults to the same nonexistent path
   .elai_cc/.elai/ssb.db, then falls back to grepping session logs. "Two implementations, two different
   fail behaviours, both anchored to a broken path."

THE VERDICT ON ITSELF, verbatim: "Net: the gate advertises fail-closed and is fail-open in the dimension it
shouts loudest about. That is a self-violation of the doctrine's own W-10 ('MUST NOT advertise fail-closed
if it is fail-open')."

THE SEVEN PRESCRIBED FIXES (§3), each mapped to a rule:
  3.1 every contract field required (R1); delete the SKIP-on-no-trigger path
  3.2 remove --run-trigger as opt-in (R3); the done-gate path never has the static-only branch
  3.3 db-not-found = FAIL (R2) + fix the path bug + a STARTUP SELF-TEST that errors loudly if the
      configured SSB path does not exist when expect_ssb is used anywhere in the run
  3.4 expect_artifact must check disk, not stdout (R4/R5) — or demote it to advisory and make expect_ssb
      the load-bearing proof
  3.5 consumes feeds the verdict (R6); _production_caller (line 370) already excludes test paths — make its
      result BLOCK the verdict, not just sharpen the report
  3.6 fail on empty (R1, ArchUnit failOnEmptyShould [25]): zero contracts found => exit nonzero
  3.7 HONEST LIMIT: the leaf-name grep is unsound. "_def_hits / _production_caller match a symbol name in
      elai.map; a dyn-Trait method called only via vtable has zero direct references and will be FALSELY
      flagged dead, while a same-named symbol in the wrong file can FALSELY pass. Do NOT pretend this is
      sound." Two mitigations: (a) require an ENTRYPOINTS map incl. trait-dispatch sites; (b) treat the
      static check as necessary-but-not-sufficient and make the RUNTIME SSB-row check authoritative.

RECONCILIATION (§4): pick ONE source of truth — make wiring-contract-validator.md CALL the script and
report its exit code + JSON rather than re-implementing the SSB query in prose. "The agent becomes a thin
runner + evidence narrator; the script is the single arbiter. This removes the UNVERIFIED-vs-FAIL
divergence by construction."

LIVE STATUS (verified by me, 2026-07-17): THE FIXES LANDED. .claude/tools/wiring_contract_check.py is now
796 lines and its own header states: "db-not-found is FAIL (not UNVERIFIED), an incomplete contract is
FAIL"; "--run-trigger  deprecated no-op: the trigger always runs (R3)"; line 50 comments the old path bug
as "a dead no-op that always returned UNVERIFIED (a silent pass)"; line 643 states "FAIL. UNVERIFIED is
gone." Exit codes are now 0 PASS / 1 FAIL (incl. missing/incomplete contract) / 2 SKIP (only with the
explicit --allow-missing-contract debug flag). So section 1 of WIRING-FIX-CLAUDE-CODE-WORKFLOW.md is a
HISTORICAL diagnosis, not a live defect list. See contradiction C2 for the stale invocation that survives.


================================================================================
5. THE LAYERED GATE (Layers 1-4, plus 2b) — what each proves and what it is blind to
================================================================================
Canonical text: research §5 (generic), WIRING-FIX-ELAI.md §2 (mapped onto ELAI), and the live
.claude/rules/wiring-enforcement.md "THE LAYERED GATE AT A GLANCE".
Ordering discipline: cheapest first; a Layer-1 fail blocks 2-4; a Layer-2 fail blocks 3-4; 3 and 4 run in
parallel after 1+2 pass. "This matches ELAI's existing scoped-test discipline (per-crate -p during
EXECUTING, full gate at AUDITING)." A feature cannot be DONE unless ALL layers pass; any layer returning
non-zero, not-found, or empty-evidence FAILs the whole gate.

--------------------------------------------------------------------------------
LAYER 1 — COMPILE-TIME (every commit, ~5-30s, static, cheap; "the floor, not the ceiling")
--------------------------------------------------------------------------------
  1a Rust dead_code: RUSTFLAGS="-D dead_code" cargo check --workspace.
     THE CRITICAL LIMITATION: "rustc's dead_code lint EXEMPTS pub items entirely (it assumes an external
     crate may call them)". Open issues #120079 (workspace-level analysis) [10] and #74970 (pub items in
     BINARY crates treated as pub(crate)) [11] — both confirmed open and unresolved as of 2024.
     THE WORKAROUND that makes this layer work at all: adopt the pub(crate) DISCIPLINE — anything not a
     genuine cross-crate API surface is pub(crate), at which point dead_code fires on unused internal items.
  1b Enum exhaustiveness — "a free, zero-cost, always-on fitness function": REMOVE #[non_exhaustive] from
     internal dispatch enums (SsbActionType dispatch, FsmSubstate, RouteDecisionEventKind, SkillEventPhase).
     Adding a variant without a match arm becomes a COMPILE ERROR, so "a producer that is not dispatched
     cannot compile."
     BLIND TO: "this proves a dispatch ARM exists, not that the dispatcher is reached from a production
     entrypoint (Layer 3 covers that)."
  1c TS/Svelte: knip --production from the real Svelte entrypoints, test entrypoints excluded from the root
     set [15]. Unreachable export from a production root = BLOCK. The gate must exit nonzero on ZERO
     modules (failOnEmpty equivalent).
     BLIND TO: module/export granularity, not call-graph granularity within a file; dynamic imports via
     computed paths and runtime require() are not tracked.
  1d Python: vulture --min-confidence 80 with a checked-in whitelist for decorator-registered routes
     (@app.post etc.) [16], whitelist entries requiring justification comments.
     BLIND TO: "misses any function registered via decorator — FastAPI @router.get, SQLAlchemy hooks,
     Pydantic validators. These are invoked by frameworks via routing/reflection, not direct call-site
     references. High false-positive rate on modern Python frameworks without maintained whitelists."
  1e cargo-machete (v0.9.2, April 2026) [28] for dead Cargo deps, exits 1 on unused dep. "Catches dead
     CRATES, not dead FUNCTIONS — complementary only."
  Python producer-without-consumer leg (added later, live rule): .claude/tools/python_wire_check.py — AST,
  module-level public defs, R6/R7 semantics, WIRE-DARK[..] escape; the runnable sibling of
  production_caller_check.py. Plus vulture for general dead code.

--------------------------------------------------------------------------------
LAYER 2 — CROSS-LANGUAGE WIRING CONTRACT (every commit, ~30s, static)
--------------------------------------------------------------------------------
  2a EntrypointSetLoader: empty / absent entrypoint set = FAIL, not warn. "A wiring audit with no roots
     proves nothing; vacuous pass is the same bug class as the harness SKIP-pass." [R1, ArchUnit
     failOnEmptyShould [25]]
  2b DkgReachabilityAuditor producer-without-(non-test)-caller = FAIL, propagated to the audit verdict.
     Test-only callers are a SEPARATE root set [9]. "This enforces W-8, which is currently unenforced."
  2c Tauri symmetry: every name in generate_handler!() requires >=1 invoke() in dashboard-v2 production TS;
     every @app.post/@router route in server.py requires >=1 caller in crates/cli runtime_client.rs.
     Missing either side = FAIL. Executor: cross_lang_diff.py (E-45).
     LIVE: this is now RUNNABLE at .claude/tools/tauri_ipc_symmetry_check.py — "an invoke('X') whose X is
     in no generate_handler! list = dead IPC = BLOCK; a registered command no call site names = WARN;
     wrapper-agnostic via the literal-presence suppression -- caught 5 REAL dead IPCs on dashboard-v2's
     first run" (live wiring-enforcement.md; VERIFIED the file exists).
  2d action_type golden lock (E-45): new SsbActionType variants must flow through the append-only golden
     file; cross_lang_diff --strict green. "Already exists — make it a hard gate, not advisory."
  2e HONEST LIMIT, verbatim: "DKG/SCIP have no call-graph edges for dyn-Trait / IPC-string / event-bus
     dispatch [research HIGH risk]. A vtable-dispatched method shows zero references and is FALSELY flagged
     dead. MITIGATION: [wiring.entrypoints] in elai_cc.toml must enumerate trait-dispatch sites; a produces
     that 'implements dispatched trait T, T reachable from entrypoint E' is accepted as wired. Layer 3 is
     the real backstop for everything static analysis cannot see."
  2c(flag) Feature-flag default audit (research 2c): a polyglot script scanning every ELAI_*_ENABLE,
     extracting the default, asserting true/1/"true" for any feature shipped in the current plan; any new
     flag defaulting false without a wire-dark-allowlist.toml row = BLOCK.

LAYER 2b — FRONTEND MOUNT REACHABILITY (added later; live rule only, NOT in the 2026-06-02 docs)
  .claude/tools/svelte_mount_reachability.py BFS-walks the module graph (imports + barrel re-exports +
  dynamic imports over .svelte+.ts) from the app root (App.svelte/main.ts); a .svelte component on no path
  from a root = wire-dark UI. ORPHAN (imported nowhere) = BLOCK; UNREACHABLE (pulled in only off the root
  tree) = WARN; gated default-.svelte-import models "imported but never written as <Tag>" = the
  never-mounted bug. The stated distinction: "knip sees 'has an importer'; this sees 'reachable from the
  root mount tree' -- the stronger property." VERIFIED the file exists.

--------------------------------------------------------------------------------
LAYER 3 — ENTRYPOINT COVERAGE (gated before EXECUTING->AUDITING, ~2-5 min, runtime binary)
--------------------------------------------------------------------------------
"The compensating control for Rust's unsound static reachability. This is the layer ELAI does NOT yet
have." (WIRING-FIX-ELAI.md §2, 2026-06-02 — now false, see C1.)
  3a Build the REAL binary instrumented: RUSTFLAGS="-C instrument-coverage" cargo build --bin elai. Run
     the auditor-owned smoke scenarios against the REAL binary with PRODUCTION-DEFAULT env (all
     ELAI_*_ENABLE at shipped defaults, NO test overrides). Merge .profraw
     (llvm-profdata merge -sparse *.profraw -o merged.profdata); cargo-llvm-cov export --format=json [21].
     Parse functions with count==0; diff against functions introduced in the current plan's git diff; any
     NEW function with count==0 = BLOCK. Suppression: #[cfg(test)] (not in the prod binary) OR a registered
     // WIRE-DARK[B-N]: <justification> stub per stub-tracking.md.
     Tool ground truth: cargo-llvm-cov v0.8.7 released May 13, 2026; --fail-under-lines exists; run
     subcommand documented; zero-count function export behavior confirmed at source level [21].
     "This is the only layer with a direct causal connection to production reachability."
  3b Python: coverage run -m uvicorn elai_inference.server:app — the REAL entrypoint, NOT pytest; coverage
     json; zero-hit new functions in the diff = BLOCK.
  3c A feature behind a default-OFF flag "will correctly show zero coverage under production defaults --
     that IS the wire-dark signal, and it is correct behaviour." The flag must be in
     wire-dark-allowlist.toml to be exempt; otherwise BLOCK.
  3d smoke-manifest.toml per B-N: plan / entrypoint / expect_ssb / flag_default. A missing entry for a new
     feature = BLOCK.
  3c(TS) Playwright E2E under c8 [23], gated on the E2E profile ONLY, never merged with unit coverage —
     "merging them allows unit hits to mask E2E misses." c8 v11.0.0, February 2026.
  BLIND TO: "Does not distinguish 'not reached in this smoke test' from 'structurally unreachable'."
  THE THREE NAMED FAIL-OPEN RISKS (research §3.3): running the smoke with flags at non-default values (e.g.
  ENABLE_X=1 in CI env) masks wire-dark code behind default-OFF flags; running coverage only during pytest
  reproduces the unit-test proxy; not parsing count==0 specifically (just aggregate %) "allows one
  well-exercised function to mask ten uncovered ones."

--------------------------------------------------------------------------------
LAYER 4 — RUNTIME TRACE ASSERTION (every PR, in-process, "the un-gameable layer")
--------------------------------------------------------------------------------
"This is the ONLY layer that cannot be gamed without modifying production code." ELAI's SSB IS the
OTel-InMemorySpanExporter equivalent [29] — an in-process, off-agent artifact.
  4a Drive the REAL entrypoint (CLI subcommand / FSM transition / IPC handler), NOT the produced function
     directly. "A test that calls the function directly reproduces the unit-test proxy and proves nothing."
  4b Bind an in-process Ssb to a temp db BEFORE invoking production code; after the entrypoint returns,
     query for the contract's expect_ssb action_type within the time window. Zero rows OR db-not-found =
     FAIL. The InMemory pattern makes "sink missing" structurally impossible [29] — the exporter is
     in-process, so there is no external sink to be absent, "the exact failure mode of ELAI's path-bug
     silent UNVERIFIED."
  4c tracing-test (v0.2.6) #[traced_test] + an EXPLICIT assert!(logs_contain("<key>")).
     THE HAZARD, verbatim [31]: "the macro does NOT auto-fail. A test using #[traced_test] without an
     explicit assert!(logs_contain('span_name')) silently passes even if zero events were emitted. This
     reproduces the fail-open pattern at the assertion level." Integration tests in tests/ require the
     no-env-filter feature. The auditor rule must REJECT any #[traced_test] test lacking the assert.
  4d SimpleSpanProcessor is MANDATORY; BatchSpanProcessor is BANNED in test setup — "its flush timing
     causes spans to not flush before the assertion runs, producing silent PASS when the feature did not
     fire" [30].
  4a(schema) research 4a: every contract must declare a span_name; missing span_name = BLOCK at Layer 2.
  FOUR NAMED FAIL-OPEN RISKS (research §3.5): BatchSpanProcessor in tests; #[traced_test] without an
  explicit assert; a test exercising the feature via direct function call rather than the real entrypoint
  ("the trace captures execution but the wiring is never verified"); a test driving the real entrypoint but
  wiring the exporter to a DIFFERENT TracerProvider than production code uses ("spans route to the wrong
  sink").
  LIVE FRONTEND EDITION (added later, not in the 2026-06-02 docs): .claude/tools/visual/ui-wire-smoke.mjs
  (bun + CDP + bun:sqlite, zero deps) drives a REAL browser click on a running dashboard and asserts a NEW
  Rust-owned SSB row lands (baseline-delta) — "the sound backstop for 'a control's handler is actually
  wired', which the static Layer-2/2b legs and the pixel-only visual-ui-loop cannot prove; db-not-found /
  Chrome-unreachable / dev-server-unreachable = exit 2 fail-closed, never a green pass."

THE TRACING-TEST RULE, promoted to its own frozen hard rule (.claude/rules/tracing-test-assert.md, VERIFIED
present; checker .claude/tools/traced_test_assert_check.py VERIFIED present):
  T-1 every #[traced_test] fn MUST contain >=1 logs_contain/logs_assert that is LEXICALLY an argument to
      assert!/assert_eq!/assert_ne!/debug_assert!/debug_assert_eq!. A bare `logs_contain("x");`, a
      `let _ = logs_contain(...)`, or one inside println!/format!/dbg! is a FAIL.
  T-2 a #[traced_test] with ZERO logs_contain/logs_assert is BLOCK, not WARN (mirrors R1). "The whole point
      of #[traced_test] is to assert on captured logs; a body that captures and asserts nothing is
      wire-dark."
  T-3 OTel-span variant: SimpleSpanProcessor + InMemorySpanExporter, assert the exported span set is
      non-empty AND contains the expected span. BatchSpanProcessor in test setup is a FAIL — "its async
      flush timing makes 'not flushed yet' indistinguishable from 'never produced', which silently
      fail-opens the assertion."
  Slopsquat receipt carried in the rule: tracing-test v0.2.6 verified live 2026-06-02 —
  max_stable=newest=0.2.6, v0.2.6 created 2026-02-07, NOT yanked.
  Self-honesty: "The checker is advisory-unsound (regex + brace-matcher over source, not a full Rust
  parse)... This rule guarantees the assert EXISTS; the harness guarantees it MEANS something."
  With --expect-traced, a scan reaching files but finding zero #[traced_test] is itself a FAIL (exit 2) —
  the checker self-tests that its scan reached the files (R2, the dead evidence path screams).


================================================================================
6. THE PRODUCTION-CALLER GATE (D-030) — the changeset-grain leg that actually bites
================================================================================
Live rule: .claude/rules/production-caller-gate.md (VERIFIED present). Checkers VERIFIED present:
.claude/tools/production_caller_check.py (.rs), .claude/tools/python_wire_check.py (.py).
This is NOT in the 2026-06-02 bundle — it was born from a measured repeat-failure and is the single most
operationally load-bearing piece of the trust core today. It matters for portability because it is the
cheapest real gate and it needs only git + a name-grep, NOT an SSB db.

WHY IT EXISTS (verbatim): "The elai-finish-sweep run (session s-107b022a, 2026-06-11..13) hit the SAME
failure mode SIX TIMES in one stretch: I-34 (hook pending), I-10 (BLOCK), I-8 (BLOCK), I-1 (BLOCK), I-32
(empty-store BLOCK), I-41 (dead framing surface). The shape is always identical: the executor builds the
machinery AND its unit test in the same changeset, the unit test calls the new fn directly and goes green,
but NO production entrypoint ever calls it."
AND THE DECISIVE LESSON (verbatim): "Prompt pleas were already added across the dispatch templates and only
REDUCED the rate, never eliminated it. An LLM cannot be reliably instructed out of its shortest-path bias;
the fix has to be STRUCTURAL and off-agent."

LEG A — the deterministic checker. Takes the changeset's .rs files, derives NEW pub symbols (git-diff-added
lines for tracked files, all pub symbols for untracked/ignored), and FAILS unless each NEW pub fn has >=1
non-test, non-import reference OUTSIDE its defining file.
  VERDICTS: WIRED (>=1 external production ref) | INTERNAL (no external ref but a non-test same-file ref —
  a transitive helper; WARN, auditor verifies the chain reaches a wired entrypoint) | DARK (a new pub FN
  with zero production refs -> BLOCK) | DARK-TYPE (a new pub type used only in its own file -> WARN, "types
  legitimately can be") | REGISTERED-DARK (a WIRE-DARK[plan-id] marker in the comment/attribute block
  directly above the def -> WARN, auditor confirms the backing stubs.md / wire-dark-allowlist.toml row) |
  UNSOUND-NAME (an fn whose name is too common for name-grep — new/run/get/.. or under 4 chars -> WARN,
  auditor resolves via codegraph_callers).
  EXIT: 0 clean (WARNs allowed) | 1 on any DARK fn (BLOCK) | 2 fail-closed on a scan-config error (git
  unreachable, missing input, --expect-symbols finding zero new pub symbols, any unhandled exception).
  "There is NO allowlist flag and NO static-only opt-out (R3): the only escape is the in-code WIRE-DARK[..]
  marker, which is visible in the diff and auditable."
  Honest self-classification: "a Layer-2 static, regex-unsound, necessary-not-sufficient leg (name-grep is
  not a call graph, R5) -- the load-bearing proof remains the Layer-3/4 runtime evidence. What it guarantees
  is that a production-LOOKING caller EXISTS lexically, EARLY ENOUGH for the executor to fix in the same
  changeset instead of after a wasted audit cycle."
  Python sibling caveat: "v1 = module-level public defs only, class methods + dynamic/string-keyed dispatch
  are residual risk on the runtime leg."

LEG B — the PRODUCTION_HOOK dispatch contract (prevention at dispatch time):
  PCG-1 (explorer) every explorer whose sub-phase ships a new pub fn MUST return a PRODUCTION_HOOK field
    naming the EXACT seam — file path + fn (and line if known). "'needs wiring' is not a seam; an unnamed
    seam is a BLOCKED explorer return."
  PCG-2 (executor) the PRODUCTION_HOOK is a NUMBERED DELIVERABLE: "wire <new fn> into <seam> in the SAME
    changeset". The executor self-runs the checker before returning DONE; a DARK fn it cannot wire is a
    BLOCKED return with the seam question, never a green DONE.
  PCG-3 (auditor) RE-RUNS the checker off-agent. "The executor's self-run is necessary-not-sufficient; the
    executor that wrote the code cannot be the authority on whether it wired it, same in-boundary failure
    freeze-the-gate exists to prevent." The checker is frozen in .claude/gate-manifest.txt (R8).

THE LEGITIMATE-DARK ESCAPE (the only non-BLOCK path for an unwired new fn): an in-code marker on the line
directly above the definition — `// WIRE-DARK[<plan-id>]: <one line: why dark + where the consumer lands>`
— AND a matching row in workflow/sessions/<sid>/stubs.md (with will_resolve_in) OR wire-dark-allowlist.toml
(with type/owner/expiry/promotion_plan_id per R9). "The marker downgrades BLOCK to REGISTERED-DARK (WARN);
a marker with no backing row is still a finding."

KNOWN TRUE-POSITIVE recorded in the rule (the checker's first real catch, 2026-06-13):
crates/tools/src/prompt_hardening.rs::dropped_row_count (the I-47 scream-path accessor) is DARK — its only
non-test readers are a comment and the i47 contract. "This is the gate working as designed on its first
production run." Routed to a Stage-5 punch list, not fixed under the D-030 pause. UNVERIFIED whether it has
since been resolved — I did not re-check that symbol.


================================================================================
7. THE DETERMINISM DOCTRINE — the general law the wiring stack is one instance of
================================================================================
Source: docs/strategy/ELAI-DETERMINISM-DOCTRINE.md (2026-06-16). Purpose, verbatim: "a portable lens
another LLM can apply to ANY ELAI feature to judge whether it respects the core design principle. Hand this
file to a reviewing model along with the feature under review and the ELAI-FEATURE-MAP. It is a checklist +
worked examples, not a spec."

THE PRINCIPLE (one sentence, verbatim): "The model decides as little as possible, and nothing the model
decides is trusted until a deterministic, model-independent mechanism has checked it."
COROLLARY: "every place ELAI relies on the model's judgment for something that COULD be made mechanical
(sizing, admission, a pass/fail verdict, a count, a reachability claim, a 'this is grounded' assertion) is a
latent defect, even if it currently works. 'It works because the model happened to get it right' is not a
guarantee; it is an ungated proxy."

THE BOUNDARY — the load-bearing operational artifact, because the whole doctrine reduces to sorting
decisions into two columns:
  THE MODEL legitimately owns acts that are irreducibly generative or selective with no deterministic
  oracle: GENERATION (write the code/prose/candidate plan/hypothesis); PROPOSAL (propose a decomposition,
  rule, config change, fix); SELECTION/RANKING (pick which of N retrieved pages is relevant, rank search
  results, choose a template); SEMANTIC JUDGMENT with no mechanical test ("is this design tasteful") — but
  even here the model is ADVISORY, never the load-bearing gate (D5).
  THE HARNESS owns everything mechanizable: SIZING/ADMISSION (is this unit the right size, does it fit the
  budget); VERDICTS (pass/fail, grounded/ungrounded, wired/dark, reachable/orphan); COUNTS (rows, files,
  callers, tokens); PROVENANCE/REACHABILITY (did a real entrypoint run this, is this citation in the
  source, did this fact come from a trusted ingest site); STATE (what phase, what budget, what trial count).
  Rule of thumb for a reviewer, verbatim: "read the feature, list every decision it makes, and sort each
  decision into 'model' or 'harness' by the lists above. Any decision in the harness column that is actually
  being made by the model is a finding."

THE FIVE SUB-DOCTRINES:
  D1 GENERATOR PROPOSES, GATE DISPOSES. "The component that produces an artifact is never the component
     that admits it... a producer judging its own output is inside the boundary the judgment was meant to
     police; it rubber-stamps. (Berkeley RDI Apr-2026: in-boundary artifacts gamed ~100%.)"
     ELAI instances: ID-1/I-0 generator-proposes-orchestrator-owns-state (the generator emits
     Intent->Directive; a process-separated Supervisor is the only side-effect executor and holds
     credentials, I-1); the rules engine phase-3F (PROPOSED->CANDIDATE->SHADOW->ACTIVE; an AUDITOR-sourced
     rule is stuck at PROPOSED until an INDEPENDENT second auditor approves; the finding-source auditor can
     NEVER approve its own rule); self-modification phase-4H ("the modifying agent NEVER evaluates its own
     modifications"; the DGM archive selects a parent variant, it does not let the proposer self-score);
     the production-caller gate D-030.
     ANTI-PATTERN: "a single agent that writes the implementation AND the test AND the wiring-contract that
     judges it; an LLM that scores its own candidate; a decomposer that declares its own nodes
     correctly-sized."
  D2 EVIDENCE IS OFF-AGENT AND FROZEN. "A gate measures an artifact the agent did NOT author (a journal
     row, a coverage hit from the compiled binary, an in-process span, a captured stdout hash), and the gate
     itself is frozen and read-only to the run being evaluated."
     ELAI instances: the F-4 cant-lie gate (AntiFabricationGate compares the auditor's CLAIMED test result
     against a blake3 stdout_hash captured by the ORCHESTRATOR; "the auditor cannot emit passed=true without
     the matching orchestrator-captured hash"); wiring R8 + gate_freeze_check.py; "main model never runs
     tests; the auditor is the sole test runner"; production_caller_check.py / maintainability_check.py /
     traced_test_assert_check.py all auditor-invoked, never an executor self-pass.
     ANTI-PATTERN: "a gate that reads the agent's own stdout/log/claim; a 'smoke test' written by the same
     agent that wrote the feature; a checker the executing plan can regenerate or edit."
  D3 MISSING EVIDENCE IS A HARD FAIL. "Absence of proof is proof of absence for gate purposes."
     Why: "'not found' usually means the feature never ran and produced it. A gate that shrugs on missing
     evidence is fail-open and IS WORSE THAN NO GATE because it advertises coverage it does not have."
     Instances: wiring R1/R2; grounding verification phase-2B (the plan sweep BLOCKS PLANNING->EXECUTING on
     ANY ungrounded claim; binary pass/fail, no soft "probably fine"); tracing-test T-1/T-2;
     production_caller_check exit 2 fail-closed on scan-config error.
     ANTI-PATTERN: "any verdict of SKIP/UNKNOWN/UNVERIFIED that is treated as not-a-failure; a gate with an
     opt-in --skip flag; 'no findings' reported without proof the checker actually reached the files."
  D4 MEASURE THE OBJECTIVE, NOT A PROXY.
     Instances: wiring R4; SAFEDELTA (G-6) — a gate-ON-vs-OFF ablation ("N fewer exploitable diffs per 100
     tasks"), explicitly NOT a leaderboard rank (SWE-bench Verified dropped as the headline, E-9 / GD-3);
     DGM fitness (Track 7 / VD-1) — the LIVE placeholder derived_alpha = blake3(config_bytes) is "fitness of
     BYTES not performance" and is a KNOWN GAP, H-43 replaces it with execution-grounded pass-rate over
     frozen tests; R5 skill-eval extension (test that the skill CHANGED behaviour — baseline-without-skill
     RED vs with-skill GREEN — not just that the with-skill run passes).
     ANTI-PATTERN: "a metric that the producer can satisfy without doing the real work; a hash/byte/length
     proxy standing in for a behavioral outcome; green tests treated as sufficient."
  D5 WHEN AN LLM IS IN A GATE, IT IS ADVISORY AND CROSS-FAMILY. "for judgments with no mechanical oracle, an
     LLM may contribute but is never the load-bearing decision: it is one advisory vote, it never grounds a
     claim or sits on a critical path, and a critic LLM is always a DIFFERENT pretraining family than the
     producer... an LLM verdict on a critical path is a fail-open gate wearing a gate's clothes."
     Instances: TD-3 ("the LLM is T3-advisory, never the load-bearing gate"; the mechanical legs
     METRIC_GATE/GREP/AST predicates are the gate, the LLM lens is the last, weakest tier); D-13 ADVERSARY
     ROLE (must be a different pretraining family from the executor; same-model self-critique is BANNED);
     VD-4 (verifier must be stronger-than-generator and cross-family); D-10 mode-routing predicate
     (select_grounding_mode is deterministic — "the LLM has NO input into which grounding mode is
     selected"); HD-3 (never force-JSON/constrained-decode the reasoning/coder/planner roles; grammars only
     on pure op-emission, arXiv:2408.02442 — "constraining the model's reasoning to satisfy a schema
     degrades the reasoning"); AutoB2G DAG repair phase-4E (the 2 repair strategies REORDER for cycles /
     SUBSTITUTE for missing tools are DETERMINISTIC; the LLM-driven SOCIA repair was NOT adopted).
     ANTI-PATTERN: "an LLM-as-judge on a release-gating path; same-family self-critique counted as
     verification; a single model vote treated as grounding."

THE REVIEWER CHECKLIST (the portable artifact — 8 questions; "Any 'no' on 2-7 is a finding"):
  1. What does the model decide here? (enumerate EVERY decision)
  2. For each: is it generative/selective (legitimately the model's) or sizing/admission/verdict/count/
     provenance/state (should be the harness's)? Flag any harness-class decision the model is making.
  3. Is the model's output checked by a mechanism it did NOT author? (D1/D2)
  4. Is the checking artifact frozen and read-only to the run? (D2)
  5. Is missing/absent evidence treated as FAIL, not SKIP/UNVERIFIED? (D3)
  6. Does the gate measure the real objective, or a proxy the producer can satisfy without doing the work? (D4)
  7. If an LLM is anywhere in the gate, is it advisory-only AND cross-family, off the critical path, never
     grounding a claim? (D5)
  8. If the feature ships behind a default-OFF flag, is the flag registered with a promotion plan (not a
     graveyard)? (wiring R9)
  "A feature that passes 3-8 is 'model-as-cog clean.' A feature that works only because the model happened
  to decide well is debt, regardless of green tests."

THE WORKED APPLICATION (decomposition + research; the doctrine's own §"HOW THIS APPLIES", 2026-06-16):
  "The instinct 'let the model decide babystep size' violates the boundary: SIZING is a harness decision,
  not a generative one." The existing deterministic scaffold it says to lean on harder:
    - Phase 4E Stage 3.5 PHASE SIZING: phase_complexity = files_touched * avg_file_tokens + test_count *
      avg_test_tokens + contract_count * 200; split if > available_for_work * 0.6; max split depth 2.
      "(Deterministic TODAY, but coarse: avg_file_tokens is a heuristic, not the measured per-(model,lane)
      context ceiling.)"
    - Phase 4H AllowedConfigKey BabyStepMaxLoc{vram_tier} — already a per-VRAM-tier self-mod-tunable knob.
    - Phase 3E CONTEXT BUDGET ALLOCATOR: deterministic token estimator (token_count_estimate for
      allocation, exact token_count for validation), 8 fixed slots, never exceeds the model window.
    - Phase 4G AutoAdvanceEvaluator: every FSM transition maps to a deterministic EvidenceCheck with NO LLM.
    - MEASURED CEILINGS (metrics/baseline_findings_and_actions.md): 3B NF4+FA2 ~40K, 7B ~25K, "both 100%
      pass on a 12GB card." J-34 (PLANNED) replaces the folklore thresholds with measured
      per-(model,lane) fill->degradation curves.
  THE CORRECT EXTENSION, verbatim: "the model PROPOSES a decomposition; a deterministic NODE-ADMISSION GATE
  measures each proposed node (files<=2, impact-cone<=K from the B-29 DKG impact API, pre-computed
  context-package size <= measured-lane-ceiling * safety_factor, intent_pairs>=3, single observable_effect)
  and REJECTS oversized nodes, forcing a re-split or a cloud-lane escalation (I-7 cascading escalation). The
  model never answers 'is this the right size.'"
  FOR RESEARCH: "the model SELECTS (formulates queries, ranks pages); the harness FETCHES... and FINGERPRINTS
  the fetched source; grounding is then a deterministic check against the harness-fetched, hashed corpus
  (span/n-gram containment as a cheap first pass, claim-level entailment for the rest, closed-set citation
  binding per H-53). Token-level hash containment is the WRONG granularity (function/common tokens are in
  every source -> vacuous; strict enforcement forbids legitimate synthesis); span + claim + closed-set
  citation is the sound form."


================================================================================
7B. THE AUX — the METHOD that produced the lens (the thing most likely to be missed)
================================================================================
Source: docs/strategy/ELAI-DETERMINISM-DOCTRINE-AUX.md (2026-06-16). Its own warning, verbatim: "If you
only pattern-match 'determinism' you have missed the point -- the point is a way of reasoning about any
ELAI improvement." And: "The determinism doctrine is ONE output of this method applied to ONE axis
(decomposition + grounding)."

THE STANCE (generalize to ANY subsystem): "For whatever subsystem the user brings an idea about, the first
move is always the same question: WHAT IS THE MODEL DECIDING HERE, and which part of that decision could a
deterministic mechanism make instead? Push the mechanizable part (sizing, admission, a verdict, a count,
provenance, state, selection bounds) into the harness; leave the model only the irreducibly generative or
selective act; gate even that against frozen, measured, model-independent criteria."

THE METHOD M1-M12 (each carries a real worked example from the thread that produced the doctrine):
  M1 GROUND BEFORE YOU CLAIM. Never assert how an ELAI subsystem works, or that an idea is novel, from
     training memory. Check live code (codegraph/elai.map/scoped read), the FEATURE-MAP (status +
     composition edges), measured data (metrics/), verify any external claim before it influences a
     decision. "Codebase-first: a question the code already answers is a finding you state, not a question
     you ask." EXAMPLE: "'E-48 is DARK with no caller' was my training-shaped assumption; the grounding pass
     found it is WIRED to the REPL and only the search backend is stubbed -- which made the research-modes
     plan narrower and correct."
  M2 CALIBRATE CONFIDENCE OUT LOUD. Tag every claim VERIFIED / high / training-unsearched. "An unverified
     claim is allowed only if labelled and not load-bearing." EXAMPLE: Co-Coder arXiv:2606.00953 — "the
     title kept appearing in search summaries but NO canonical arxiv.org page landed across 3 searches;
     marked PROVISIONAL/UNVERIFIED and explicitly NOT load-bearing, with the technique (community
     detection) cited independently so the design did not depend on it."
  M3 STATE THE VERDICT BLUNTLY, INCLUDING THE CORRECTION. "a polite 'yes, and...' that buries a fatal flaw
     costs a whole build cycle." EXAMPLE: "'hash every token and check membership' -> stated plainly that
     this is VACUOUS (function/common tokens are in every source) and forbids synthesis, then gave the sound
     form (span fingerprint -> small-NLI -> closed-set citation)."
  M4 SEPARATE NET-NEW FROM ALREADY-BUILT: (a) reinvention of a shipped feature, (b) extension/activation of
     a partial one, (c) genuinely new. "Reinvention is the default trap... ELAI has ~250 plan units; the
     cheap win is composing what exists, not rebuilding it. COMPOSE-NOT-RESTATE (GD-1) is a hard rule."
  M5 FIND THE ONE PRINCIPLE UNDER A CLUSTER OF IDEAS. EXAMPLE: four ideas (token-hash, decomposition,
     Godel-gating, research modes) "all reduced to 'push decisions out of model discretion into
     deterministic machinery' -- which is already the ELAI spine, so the work became choosing the sound
     mechanism per idea, not inventing a paradigm."
  M6 EXISTS-BUT-COARSE IS NOT GREENFIELD. "the right plan says 'replace heuristic H at file:line with
     measured signal S'." EXAMPLE: the 4E sizer already splits on files*avg_tokens*0.6; the net-new was
     swapping that heuristic for a cone-token lower-bound vs the MEASURED ceiling.
  M7 ATTACK YOUR OWN DESIGN. "an unattacked design ships its own failure mode." EXAMPLE: the
     cost-estimation idea was attacked with the r<0.15 result (total cost is unpredictable) -> forced the
     redesign to "bound-and-escalate, never predict"; SWEnergy (4B models ~0% agentic) forced the honest
     caveat that sizing makes a local model FIT not CAPABLE.
  M8 ROUTE GENUINELY-OPEN QUESTIONS TO GROUNDING; DO NOT GUESS. "'I don't know, routing to grounding' beats
     a confident wrong answer baked into a plan."
  M9 MAKE THE FALSIFIER THE OBSERVE-ONLY FIRST STEP. "nothing goes default-ON until its false-positive /
     over-block rate is bounded on >=3 real runs (GD)... gates that ship default-ON unvalidated become
     either noise or a fail-open guarantee nobody trusts."
  M10 RECORD HONEST RESIDUE. "A correct answer with a missing caveat is a wrong answer... the next session
     inherits your confidence level; hidden residue becomes a silent landmine."
  M11 DO NOT CONFLATE AXES. "Latency/cost is not capability; speed is not correctness; a cache is not
     state." EXAMPLE: KV reuse vs compression are orthogonal — "the answer is 'compose them', not 'pick
     one'."
  M12 WRITE IT DOWN SO IT SURVIVES THE SESSION (reasoning chain in workflow/reasoning/, durable lens in
     docs/strategy/, handoff in workflow/handoff/).

THE PER-AXIS STANCE SKETCHES (the AUX's actual generative ask — apply the method to each axis to PRODUCE
that axis's doctrine):
  RESEARCH: model decides queries + page ranking; harness decides fetch + extraction + what is "enough"
    (deterministic source ranking, extractive not generative compression, a context-sufficiency gate the LLM
    does not adjudicate per TD-3, external per-step state). -> J-40.
  PLANNING: model proposes the decomposition; harness decides node SIZE/admission (bound-and-escalate, not
    predict), ordering (topological/DAG), and done (an evidence gate). "Cross-cutting-but-atomic intents
    need an escape so the gate does not split pointlessly." -> J-39.
  REASONING/GRC: model generates the step; harness decides DEPTH budget (serial-depth not window-size), when
    to stop (must_converge/coverage), whether a step is grounded (off-agent check), and whether to
    DISTILL+RESET+re-reason. "The mechanizable carry-state must be EXTRACTIVE+STRUCTURED, not free-text
    prose, and the training-free reasoning variant may REGRESS a small model -> falsify before trusting."
    -> J-42.
  MEMORY: model proposes a write; harness decides validity (citation-before-use, staleness verdict),
    retrieval (hard-filter superseded rows, never narrate [OUTDATED]), injection scope ("auditor/critic get
    zero memory -- objectivity firewall"). See H-31/H-32/F-12.
  SECURITY/ROUTING/EVAL: "same shape -- the model never adjudicates the gate, the verdict is off-agent +
    frozen, missing evidence is FAIL, the objective is measured not proxied, an LLM in the loop is advisory
    + cross-family."

THE ONE-LINE TEST (the AUX's own exit criterion): "Before proposing any ELAI improvement on any axis, you
should be able to answer: 'What does the model decide here, what can the harness decide instead, what is the
cheapest experiment that proves me wrong, and where did I write it down?' If you cannot, you have not
applied the method yet."


================================================================================
7C. THE DOCTRINE PROPAGATION DESIGN — and the self-referential trap
================================================================================
Source: workflow/reasoning/determinism-doctrine-propagation-design-2026-06-16.md (637 lines, base design +
four addenda b/c/d). The owner's own framing recorded in it: "the user named this 'the biggest thing' --
bigger than the ContextController/CarryState design, which is a downstream application of it."

THE PROBLEM: "The determinism doctrine today exists as a PASSIVE LENS: a checklist + worked examples you
hand to a reviewing model. A passive lens is a soft gate, and a soft gate that an agent applies to its own
work rubber-stamps -- which is the precise failure the doctrine itself warns about (D1)."

THE THESIS (§0, verbatim): "The determinism doctrine is the GENERAL LAW of which wiring-enforcement,
production-caller-gate, maintainability-review, tracing-test-assert, the F-4 cant-lie gate, and
freeze-the-gate are already SPECIFIC INSTANCES -- so 'propagate it to every system' means doing for the
whole doctrine what wiring-enforcement already did for one sub-doctrine."
And the risk-lowering fact (§1): each of D1-D5 ALREADY has >=1 shipped/specced enforcement instance, so
"we are NOT inventing enforcement from zero. We are GENERALIZING a pattern that already works in four
places and making it the default for the OTHER ~200 ELAI features that were never reviewed through this
lens. This is 'activation not invention'."

THE SELF-REFERENTIAL TRAP (§2 — the sharpest single idea in this whole bundle, stated plainly):
  "To enforce 'the model decides as little as possible,' you must decide, for every decision a feature
  makes, whether it is GENERATIVE/SELECTIVE or SIZING/ADMISSION/VERDICT/COUNT/PROVENANCE/STATE. But
  CLASSIFYING a decision into those buckets is itself a semantic judgement with no clean mechanical oracle.
  So:
   - if a deterministic checker CLAIMS to render that classification, it will have low recall and silently
     miss real violations -> it advertises coverage it does not have, which is the exact D3 anti-pattern the
     doctrine bans;
   - if an LLM renders the doctrine-conformance VERDICT on a gating path, that is an LLM-as-judge on a
     release-gating path -> a D5 violation, a fail-open gate wearing a gate's clothes.
  So a propagation built as 'one big checker' either lies about its coverage (violates D3) or smuggles an
  LLM onto the critical path (violates D5). The doctrine cannot be fully mechanically enforced, and
  pretending otherwise is itself non-doctrinal."
  THE RESOLUTION IS THE DOCTRINE APPLIED TO ITSELF: D5 says split it. A small DETERMINISTIC SUBSET (the
  structurally-detectable anti-patterns) becomes the frozen fail-closed teeth; the SEMANTIC remainder
  becomes a cross-family ADVISORY producing findings + a registered DEBT row, never a sole release verdict.
  "This is EXACTLY the shape maintainability-review.md already took... maintainability-review is therefore
  the PRECEDENT TEMPLATE, and the determinism doctrine propagates as a maintainability-review-shaped rule
  ONE ALTITUDE UP -- it is the meta-rule that wiring, production-caller, maintainability, and
  tracing-assert are all instances of."

TWO APPROACHES (§8, house rule):
  APPROACH A — HEAVYWEIGHT SIXTH RULE (a full doctrine_conformance_check.py claiming broad coverage of the
    8-question checklist, fail-closed BLOCK, parallel to wiring in weight). REJECTED as the primary: "its
    semantic core is only weakly mechanizable, so it would either advertise coverage it lacks (violates D3)
    or route an LLM verdict onto the gating path (violates D5). It also invites doctrine-theater: a
    checklist filled to pass, not to think (Goodhart on the ledger itself, the D4 failure). High authoring
    cost for low honest coverage."
  APPROACH B — LENS + THIN FLOOR + DEBT LEDGER. RECOMMENDED. "The decisive argument is self-consistency: a
    propagation mechanism that violated D3/D5 to enforce D1-D5 would be self-refuting. B is the only form
    that propagates the doctrine without breaking it."

THE FIVE LIFECYCLE INJECTION POINTS (§3 — each reuses an EXISTING seam; none is greenfield):
  1 DESIGN TIME (spec-eval, oracle, grill, cook) — cheapest. ADD a mandatory DECISION LEDGER output
    dimension: enumerate every decision the design makes and sort model-class vs harness-class; any
    harness-class decision handed to the model is a named DESIGN WEAKNESS. The cross-family vote (D5) lives
    here.
  2 PLAN TIME (idea-intake per-level auditors; planning-workflow plan-improvement steps 4 + 7) — a DOCTRINE
    PRE_GATE exactly parallel to the WALKING-SKELETON PRE_GATE. Every deliverable declares: (a) DECISIONS,
    (b) MODEL or HARNESS per decision, (c) for each HARNESS-class decision the off-agent DISPOSER that
    checks it, (d) for each legitimately-model-but-ungated decision a DOCTRINE-DEBT row. "A deliverable that
    puts a sizing/admission/verdict/count/provenance decision on the model with no disposer is REFUSED at
    decomposition -- the same way a producer-only step is refused today."
  3 AUDIT TIME — the thin deterministic doctrine floor as a fourth frozen checker + a cross-family ADVISORY
    review emitting D-DETERMINISM-* findings. "The deterministic floor is the teeth; the advisory review is
    the qualitative catalog (unknown = FAIL)."
  4 DISPATCH TIME (orchestrate, auto-orchestrate, agent-dispatch-preamble) — add the doctrine pointer so
    every worker is doctrine-bound; extend the R10 integration gate: "a cross-seam smoke must confirm that a
    harness-class decision a worker implemented is actually disposed by the harness, not silently by the
    model at the seam."
  5 FREEZE (gate-manifest.txt) — the floor checker AND the doctrine rule file added, so a run cannot weaken
    them. "A run that edits its own doctrine gate is BLOCK."

THE THIN DETERMINISTIC FLOOR (§4) — "deliberately LOW-RECALL / ZERO-FALSE-POSITIVE: it only flags the few
doctrine violations that have an unambiguous structural signature, so it never lies about coverage
(D3-honest) and never blocks a legitimate pattern." (The zero-FP claim was later RETRACTED — see C1 below.)
  DD-1 GATE-READS-OWN-OUTPUT: a gate/checker/verdict fn whose evidence source is the same agent's
       stdout/log/claim rather than an off-agent artifact (the F-4 inverse). -> BLOCK candidate.
  DD-2 SKIP-ON-MISSING-EVIDENCE: a gate returning SKIP/UNVERIFIED/WARN instead of FAIL when its evidence
       path is absent (the D3 inverse). -> BLOCK candidate.
  DD-3 SAME-FAMILY-CRITIC: an adversary/verifier/judge dispatched on the SAME pretraining family as the
       producer (the D5 inverse). -> BLOCK candidate.
  DD-4 LLM-VERDICT-ON-GATING-PATH: an LLM call whose return is consumed as a release-gating boolean with no
       mechanical leg behind it. -> WARN ("hard to prove mechanically; advisory review confirms").
  DD-5 CONSTRAINED-REASONING: a grammar/force-JSON/constrained-decode applied to a reasoning/coder/planner
       role rather than pure op-emission (the HD-3 inverse). -> WARN.

THE DOCTRINE-DEBT LEDGER (§5, the wire-dark-allowlist analog — "no graveyard"): a row carries decision_id,
feature, sub_doctrine (D1..D5), why_unmechanized (the missing oracle), current_disposer
(advisory-cross-family | none | manual), owner, expiry_date, promotion_plan_id. "A debt row with no
promotion plan is itself a finding."

MEASURE, DON'T ASSERT (§6 — D4 turned on its own propagation): a DOCTRINE-DEBT RATE = open debt rows /
total reviewed decisions, per subsystem, over time. "The propagation is judged on the measured DELTA, not on
the existence of a checker." Ship the floor + ledger + advisory OBSERVE-ONLY (WARN, never BLOCK) first; run
across the existing ~213 B..K plans + live code to establish the baseline debt rate and the floor's FP rate;
"ONLY THEN promote each DD-* finding class to fail-closed BLOCK, per-class, once its FP rate is known." A
FEATURE-MAP conformance stamp per feature: clean | debt(row-id) | unreviewed — "unreviewed is treated as
debt (D3: absence of a review is not a pass)."

RELATIONSHIP TO SSB / CARRYSTATE (§7 — the owner's "almost as important as SSB" framing, answered): "SSB
makes what the model CLAIMS un-fabricable (the EVIDENCE substrate, D2); CarryState makes what the model
REMEMBERS un-lose-able (the STATE substrate); the DETERMINISM DOCTRINE is the law that says BOTH must exist
because the model is a cog. SSB and CarryState are two pillars; the doctrine is the foundation they both
stand on. Propagating the doctrine is therefore strictly more fundamental than either pillar."

--- ADDENDUM b (2026-06-16b) — THE AUX REFRAME: sections 1-12 propagated the LENS and under-weighted the
    METHOD. Two tracks, not one:
  TRACK 1 THE LENS = a checkable PROPERTY OF A FEATURE, propagated by GATES (the §1-12 design).
  TRACK 2 THE METHOD = a property of the REASONING that PRODUCES a feature. "You CANNOT regex the QUALITY of
    reasoning. But the ARTIFACT SHAPE the method mandates IS deterministically presence-checkable":
      M1 -> every external claim carries a source or a tag
      M2 -> every claim tagged VERIFIED / high / training-unsearched
      stance -> a DECISION CLASSIFICATION present
      M4/M6 -> a "reinvention | extension | net-new" + named file:line seam present
      M7 -> an ATTACK section present
      M9 -> the cheapest falsifying experiment is literally step 1
      M10 -> a RESIDUE section present
      M12 -> the reasoning chain + durable lens + handoff exist
    "missing ATTACK section = FAIL, untagged load-bearing claim = FAIL: this is D3 applied to the reasoning
    artifact", with QUALITY left to cross-family advisory.
  TRACK 3 (A2) THE GENERATIVE THIRD TRACK — "the highest-leverage propagation is not just 'enforce the
    existing doctrine everywhere' -- it is 'RUN THE METHOD on every axis to PRODUCE the axis-specific
    doctrine'... The determinism doctrine becomes the first member of a doctrine FAMILY, not a singleton.
    Not one lens stamped on 200 features, but the METHOD applied per axis to yield the right lens for each."

--- ADDENDUM c (2026-06-16c) — POST-RESEARCH RE-REASONING. A deep-research report settled the six open
    questions. THE ARCHITECTURE SURVIVES; three claims were overstated and corrected. THIS IS THE MOST
    LOAD-BEARING HONESTY IN THE BUNDLE — do not read the design without it.
  V0 HEADLINE (confidence upgraded reasoned -> VERIFIED): Approach B "is a RECOGNIZED meta-governance
     pattern, NOT a novel bet at the component level. It is the documented shape of architecture fitness
     functions (evolutionary architecture), policy-as-code with suppression/baseline files
     (OPA/Rego/Conftest), supply-chain attestation (SLSA/in-toto), tuned static analysis with baselines
     (Semgrep/CodeQL), and ISO/PCAOB/DO-178C exception-registry + measured-non-conformance governance. The
     self-referential resolution is sound and has precedent ('who audits the auditor' = separation of
     measurer from measured)." Approach A confirmed brittle "precisely because the scorer's own
     classification is subjective -> high false-negatives or an LLM-on-the-gate."
  V1 CORRECTION C1 — DROP "ZERO-FP FLOOR"; IT IS LOW-FP, AND RUST IS THE WEAK LEG. Evidence: structural-
     anti-pattern / test-smell static detection runs 17-53% FP on auto-generated code, ~10-20% on manual
     after tuning; tuned Semgrep SECURITY rules reach <5% FP — "low, never zero." Specific gap: "'Rust full
     dataflow' detection is weaker than for JS/Python/Java/Go/C#. Since this codebase is Rust-heavy, the
     floor must lean on AST + syntactic pattern + baseline-files for Rust, NOT full dataflow, and accept
     that its Rust recall/precision is the weakest leg." Consequence: the floor stays WARN/observe-only,
     DD-4 stays WARN ("no LLM-specific benchmark" found for it), and observe-only-first is now
     MANDATORY-BY-EVIDENCE, not a nicety.
  V2 CORRECTION C2 — "THE METHOD-TRACK AND THE LEDGER ARE BOTH THEATER WITHOUT A RANDOM-SAMPLE DEEP AUDIT
     OF SUBSTANCE. This is the single most important fold-in." Evidence: mandated shapes (ADRs, pre-mortems,
     WHO/Pronovost checklists) improve outcomes ONLY when audited/random-sampled, and PARTIAL COMPLIANCE
     GIVES A NULL EFFECT (WHO checklist meta-analyses: full compliance 18-47% complication drop, partial =
     no effect). "Boilerplate-filled sections ('empty attack paragraph, generic falsifier') are the DOMINANT
     outcome of shape mandates without deep audits." The ledger: "Goodhart is ubiquitous and an exception
     ledger + tracked rate AMPLIFIES surrogation (people fill rows minimally and proliferate registered
     exceptions without remediation)." THE FIX: a RANDOM-SAMPLE DEEP AUDIT of artifact SUBSTANCE and of
     ledger rows, by an auditor that is NOT the author, tracking REMEDIATION VELOCITY not exception count,
     with hold-out un-gamed subsets. "Without this leg, both track 2 and the ledger are decoration." And:
     "'partial = null' means the method must be all-or-nothing WITHIN each sampled subset -- you cannot
     half-enforce it and expect half the benefit."
  V3 CORRECTION C3 — THE CROSS-FAMILY ADVISORY IS DECORATION UNLESS CALIBRATED + GIVEN ESCALATION TEETH.
     Evidence: same-family LLM judges show 10-25% self-preference inflation (cross-family confirmed
     necessary -> D5 vindicated), BUT advisory-only review is routinely IGNORED unless (a) calibrated
     against 100-300 human-labeled traces to Cohen kappa > 0.6 ("uncalibrated = 'decoration,' kappa < 0.5 =
     unusable") and (b) integrated with HUMAN ESCALATION for flagged cases + explicit weighting/visibility,
     "not just a dashboard." Consequence: "the advisory leg carries a real, previously-unbudgeted cost -- a
     calibration corpus of 100-300 human-labeled doctrine-conformance examples -- and until that corpus
     exists the advisory leg is ADMITTEDLY decorative and must be labelled so (it does not count as the D5
     cross-family check until calibrated)."
  V4 SCOPE DISCIPLINE — THE PER-AXIS DOCTRINE FAMILY IS NOW THE DOMINANT RISK. "The report's strongest
     attack lands here: proliferating per-domain doctrines spawn 'process debt' that slows velocity more
     than it helps, and heavy process governance EMPIRICALLY HURTS in high-velocity/uncertain domains when
     culture is absent (CMMI-vs-agile studies). So A2 must be disciplined HARD: sequence axes by MEASURED
     pain (an observed, recorded failure on that axis), CAP the live doctrine count, and author a new
     axis-doctrine ONLY when a real failure justifies it -- never speculatively 'for completeness.'...
     Better to have two battle-tested doctrines than eight ceremonial ones."
  V5 FALSIFIER-AS-STEP-1 — the A/B/C experiment, adopted verbatim: 20-50 parallel small-scope agent runs
     split A (hybrid: shape gates + advisory + registry + observe-only baseline) / B (nothing: pure model
     discretion) / C (simulated comprehensive scorer: LLM classification on the gating path). Observe-only
     baseline window, then flip to blocking where possible; measure % of shipped features with post-hoc
     HUMAN-AUDITED "unmechanized-discretion leaks" + bug density + engineer time-to-ship. "FALSIFIED if
     hybrid does NOT show statistically lower leaks than B AND lower bureaucracy overhead than C (or
     only-equal quality)." Cost: one harness fork + a human audit panel, days of effort.
  V6 HONEST RESIDUE: "The central claim remains an UNPROVEN BET: no direct precedent exists in the
     autonomous-agent-harness domain (the components are proven elsewhere; their composition HERE is not).
     Success 'hinges on execution: audits, culture, tight integration'... We are not buying a guaranteed
     win; we are buying a well-precedented architecture whose domain-transfer must be measured." Three
     claims the report could NOT verify: exact FP for LLM-verdict-gate detection (DD-4); a large RCT on
     artifact-shape-alone vs downstream code quality; a direct A/B of hybrid-vs-scorer in agent harnesses.
     Plus: the calibration corpus + the audit panel are "real recurring HUMAN costs the design now
     explicitly carries; they are the price of the advisory + ledger legs not being theater."

--- ADDENDUM d (2026-06-16d) — the read-gap closed. Five baseline corrections:
  G1 THE PER-AXIS FAMILY IS PARTLY ALREADY BUILT: J-39 (decomposition), J-40 (research), J-41 (grounding),
     J-42 (reasoning) ARE the "DD wave"; each ends "ELAI-DETERMINISM-DOCTRINE: the canonical <X>
     application." So the propagation wave must NOT re-author these; it targets the META axis (the
     doctrine-conformance review of EVERY feature + the reasoning-METHOD enforcement) "which is the actual
     net-new of this whole program", plus the REMAINING axes (memory, routing, eval, UX, security) authored
     ONLY on measured pain (V4).
  G2 THE J-FORMAT IS THE TEMPLATE (J-39 as exemplar): header line (Track | lever | grounding | nature |
     effort | WAVE | depends); a problem/thesis paragraph stating the cog invariant; ## Steal/build with the
     FALSIFIER EXP as numbered OBSERVE-ONLY step 1 + an EXCLUDED list; ## Existing ground truth with
     grep-confirmed file:line seams; ## Integration with an honest residue line; ## WIRING_CONTRACT
     (produces_symbols / consumes_symbols / reachable_from / stub_ban / observable_effect = an OFF-AGENT
     artifact / SSB); ## Grounding (each cite VERIFIED or PROVISIONAL); ## COMPOSES-WITH; ## Open questions
     / risks; ## Next.
  G3 LIVE SSB BASELINE IS SCHEMA 268 / TV 577 at that authoring, "NOT the 234/543 in stale memory." Any new
     action type bumps baseline+1 COMPUTED AT IMPLEMENTATION, never hard-coded. "the propagation wave likely
     needs FEW/NO new SSB types."
  G4 TWO FALSIFIER ALTITUDES: the DD wave's EXP-1/2/3 are PER-FILE observe-only falsifiers; the V5 A/B/C
     experiment sits ABOVE them (does the PROPAGATION MECHANISM beat both nothing and a comprehensive
     scorer). "Both altitudes are observe-only-step-1; neither is skipped to default-ON."
  G5 CONFIRMED CONSISTENCY: the worked example's unifying principle is verbatim the doctrine spine; no
     contradiction surfaced. Next free J-number at the time: J-43; wave name DOCTRINE-PROPAGATION (DP).


================================================================================
7D. THE DISTILL-RESET REASONING (the fourth reasoning doc) — mostly a NEGATIVE result
================================================================================
Source: workflow/reasoning/distill-reset-propagation-reasoning-2026-06-16.md. Included in the bundle
because the owner's hypothesis was that ELAI-as-context-pipeline generalizes across reasoning/planning/
research. IT IS ONLY TANGENTIALLY A TRUST-CORE DOC, but its grounding posture is the doctrine in action and
its verdict is a correction, so it is summarised here.

GROUNDING POSTURE (read before trusting any number): the input is ONE deep-research report — "a generated
survey with inline 'citeturn' markers -> a WEAK tertiary source... Single source, no second-model
diversity vote yet." All deltas are AS-REPORTED.
  VERIFIED-THIS-PROJECT (main-LLM WebSearch, 2026-06-16): InftyThink arXiv:2503.06692 (ICLR 2026); RSA
    2509.26626 (NeurIPS 2025); self-distill degradation 2603.24472; summarization attractors 2601.08846;
    CoRAG 2501.14342 (prior session).
  REPORTED-UNVERIFIED (cited ONLY by the report; "must be confirmed before load-bearing -- this is the
    Co-Coder lesson"): Accordion-Thinking 2602.03249; InftyThink+ 2602.06960; MEM1 2506.15841; ReSum
    2509.13313; AdaCoM 2605.30785; ContextCurator 2604.11462; Context-Folding 2510.11967 / AgentFold
    2510.24699 / FoldAct 2512.22733; recursive-summary-dialogue 2308.15022; ACON; CompAct; CaOPD; KV-Fold;
    "Characterizing Prompt Compression Methods" (ICML 2024); "Maximum Effective Context Window" 2509.21361.

THE NUMBERS (all AS-REPORTED):
  F2 KEYSTONE POSITIVE: InftyThink on Qwen2.5-Math-7B BEATS a same-trained 32K RoPE extension:
     91.65/40.00/51.86 vs 90.91/30.63/48.26 (MATH500/AIME24/GPQA_d) = +0.74/+9.37/+3.60. "BUT requires SFT
     on reconstructed iterative data."
  F3 KEYSTONE NEGATIVE (the design's main threat): Accordion-Thinking cold-start FOLD HURTS before RL:
     Qwen2.5-Math-7B 48.5->45.7 avg, Qwen3-4B-Base -4.9. "Lossy self-compression destroys intermediate state
     the next step needs. This is the direct counter to 'training-free folding helps a small model.'"
  F4 RL closes the gap: InftyThink+ on DeepSeek-R1-Distill-Qwen-1.5B beats vanilla RL AIME24 50.94 vs
     38.75, GPQA_d 37.50 vs 29.81, latency 149s->100s (->48s with efficiency reward).
  F5 CARRY-STATE: extractive/structured > abstractive prose. ICML-2024 prompt-compression: extractive beats
     abstractive by 3-5 pts, up to 10x compression, HotpotQA Mixtral +4.54; CompAct up to 47x.
  F7 FROZEN-EXECUTOR + EXTERNAL-CURATOR is the highest-evidence architecture "and it IS ELAI's thesis":
     ContextCurator (7B curator matches GPT-4o; WebArena 36.4->41.2 with fewer tokens; DeepSearch 53.9->57.1
     with 8x less token use), AdaCoM, MEM1 (3.5x perf / 3.7x mem vs Qwen2.5-14B on long multi-hop), ACON
     (26-54% peak-token savings, up to 46% for smaller LMs).
  F8 FAILURE MODES with strengths: error-accumulation (moderate); uncertainty-suppression/overconfidence
     (moderate-but-indirect, "up to 40% OOD on Qwen3-8B/DeepSeek-7B from self-distill 2603.24472");
     attractor-drift (weak-moderate); lost-in-the-middle after reseed (STRONG; "RULER: 32K-advertised models
     fail at 32K on hard tasks"); hallucinated-continuity (moderate); cold-start degradation (STRONG).
  F9 ALTERNATIVES ranked for a 3-8B local model: (1) external retrieval/extractive note store; (2)
     harness-managed distill-reset; (3) KV-cache/sparse-attention (SnapKV ~3.6x gen speedup/8.2x mem;
     DuoAttention 2.55x mem/2.18x decode); (4) simple context extension (often fails); (5) paged virtual
     memory (MemGPT GPT-3.5 DMR 38.7->66.9, GPT-4 32.1->92.5); (6) state-space/linear-attention. KV-Fold
     (100% exact-match retrieval 16K-128K, depth 511 on Llama-3.1-8B/40GB) flagged as emerging.
  F10 THE GAP (the report's own conclusion): "NO strong/replicated evidence that a TRAINING-FREE,
     DETERMINISTIC, HARNESS-managed distill-reset loop improves PURE reasoning quality on small local models
     at equal budget; NO established best-practice carry-state for reasoning specifically; NO accepted
     unified framework proving one context policy works across reasoning+planning+research on the same local
     model. 'Those are precisely the holes your project could test.'"

THE REFRAME (P2/P3, the propagatable thing): "the thing that propagates is NOT the distill-reset OPERATOR;
it is the EXTERNAL CONTEXT CONTROLLER + a CARRY-STATE CONTRACT. Distill-reset is ONE policy that controller
can apply." The primitive: a harness-owned ContextController that, for a frozen small executor, on a
DETERMINISTIC trigger (context-fill/budget), captures a CARRY-STATE CONTRACT = {typed state slots +
extractive anchors (verbatim spans + IDs) + unsolved-branch list + uncertainty fields + provenance, raw
trace recoverable}, then reseeds with {pinned goal + carry-state}. "ELAI already has most pieces
wire-dark/partial: context-compiler (distills), SESSION DAG (lossless trim + recoverable stubs), E-25
compaction ladder, H-50 local unload, GRC reset_topic, B-39 reflect. The propagation work is UNIFYING these
under one controller + contract, not inventing the operator."
SEQUENCE BY EVIDENCE, NOT ELEGANCE (P4): RESEARCH first (strongest training-free evidence), PLANNING second,
REASONING LAST ("weakest training-free, can REGRESS").
BOTTOM LINE, verbatim: "The report does NOT support 'small models clearly reason better when reset.'"


================================================================================
8. THE PORTABLE E2E TESTING SYSTEM (the other extraction, already written as a port guide)
================================================================================
Source: workflow/reasoning/E2E-TESTING-SYSTEM-PORTABLE-GUIDE.md (2026-06-28). This is the SECOND
already-written portability artifact in the bundle and is directly relevant to the owner's ask: it is the
trust core applied to testing, with an explicit KEEP/REPLACE checklist. All its paths are under
F:\.ELAI_workflow\conductor\ (stale; the live dir is conductor/ — VERIFIED all named
files exist there).

THE LOAD-BEARING IDEA (verbatim): "a tester that scores its own output rubber-stamps. So every verdict is
fail-closed (missing evidence == fail) and grounded in an off-agent artifact (AX tree / journal row /
process exit code), and the gate the tester runs is frozen so the thing under test cannot weaken it."

TWO LANES:
  Lane A VISION (paid, ~10K input tokens per step, expensive, periodic): "the only thing that can catch a
    VISUAL/interaction bug a human would see. Used sparingly, pixel-diff gated."
  Lane B DETERMINISTIC (free, every pass): "the workhorse; it found the first real bug (a CLI `--json` vs
    human exit-code divergence) and the fix flowed through the flywheel to VERIFIED autonomously."
  A THIRD WAS TRIED AND CORRECTLY KILLED — record this as a decision, verbatim: "a single-shot 'CLI
    acceptance judge' (one read-only model fire deciding if a feature meets its acceptance rubric). It was
    deproved because a single read-only fire cannot satisfy a multi-scenario contract-test rubric, so it only
    ever produced unknown=fail noise. Lesson: acceptance is contract-tests' job; liveness is the
    deterministic lane's / wiring-gate's job; paid model credits are worth spending ONLY on vision. Do not
    build a single-shot LLM acceptance judge."

THE FOUR DETERMINISTIC MODES (test_modes.py — free, every pass, all driving the REAL binary read-only):
  1 metamorphic/differential CLI: same command twice => identical journal action_type signature modulo
    per-run nonce (a difference = non-determinism defect); a `--json` run and a human run must agree on
    success/failure class; an idempotent command's second run must add NO new state row.
  2 arg-surface fuzz: malformed flags MUST fail cleanly (normal nonzero/usage exit), NEVER crash (panic,
    abort, signal, Windows 0xC000xxxx). CRITICAL FALSE-POSITIVE CLASS, verbatim: "A LAUNCHER-side failure
    (input never reached the binary, e.g. an embedded null byte on Windows) is SKIPPED, never counted as a
    defect -- it tested nothing. (This false-positive class cost a whole live run before it was fixed...)"
  3 journal (SSB) invariants: schema_version monotonic, action_type in a golden lock set, seq strictly
    increasing (no dup), no orphan rows (action_type with blank session_id).
  4 replay-verify: `replay --verify <recorded-run>`; nonzero exit = the run no longer reproduces.
  "A finding is NON-LOAD-BEARING: it is display + a fix-task seed, never a gate, never ships code, never
  weakens a gate."

THE VISION LANE'S FAIL-CLOSED VERIFY (the pattern worth stealing): "the model saying 'done' is NOT evidence.
A pass requires (a) the expected_effect string appears as an AX role/name in the tree, OR (b) a journal row
matching expected_effect landed after the pre-journey watermark. No evidence => verdict 'unknown' == fail."
Plus a fail-closed deny-list UI safety guard (classify_ui_action_safety resolves the SMALLEST AX element
under the click point and refuses if its name/role matches DESTRUCTIVE_ACTION_LABELS: launch/cloud/run-task,
pay/billing, delete/reset/wipe, commit/push/merge/publish). "The verdict cannot be flipped by a flag."

THE NON-NEGOTIABLE DOCTRINE (§7) — the same spine restated for testing: fail-closed everywhere; off-agent
evidence ("The model's 'looks done' is worth zero"); unknown == fail; frozen gate; default-OFF + fail-soft
(so the tester can never take the daemon down); injectable edges ("the whole system is unit-testable with
zero credits / zero live processes (the suite is ~795 tests, all green, no live anything)"); data
minimization to the model ("prompts are SOURCE-FREE (goal + rubric + AX list + downscaled crop only; never
source code, paths, or codenames) and outbound/inbound guard-scanned. Matters doubly when the model is a
third-party proxy"); anti-rubber-stamp schema (per-item pass|fail|unknown with a mandatory evidence region).

THE PORTING CHECKLIST (§8):
  KEEP almost verbatim (app-agnostic machinery): tester_sandbox.py, cdp_action_driver.mjs, cdp_driver.py,
    tester_vision.py (the see->act loop, verify, safety guard), tester_runtime.py, vision_gate.py,
    tester_lane.py, checkpoint_map.py (the loader), test_modes.py (the four modes), trigger_safety.py.
    ALL VERIFIED PRESENT at conductor/.
  REPLACE/SUPPLY per app:
    1. The rubric file (ELAI-TEST-CHECKPOINT-MAP.json equivalent — VERIFIED present at
       workflow/reasoning/ELAI-TEST-CHECKPOINT-MAP.json): "This is the bulk of the retargeting work.
       expected_effect is the contract between 'what the feature does' and 'what counts as proof', so it
       must name a REAL observable (an AX role/name you can read, or a real journal row type), not prose."
    2. THE OFF-AGENT EVIDENCE SOURCE — the portability crux, verbatim: "Any app needs SOME off-agent
       artifact a feature leaves behind that the tester can read without authoring -- a structured log, a
       metrics file, an analytics event, a DB row, an exit code. Without one, the vision lane degrades to
       AX-only (still useful) and the deterministic journal-invariant mode does not apply."
    3. The launch story (DashboardRuntime / make_cli_runner / Sandbox.ssb_db) — "All three are small,
       localized."
    4. The destructive-action deny-list + CLI safe-list for YOUR surfaces ("a tester that can auto-click
       'Delete account' or run a billable command is a liability").
    5. Goldens: ~80 known-good screenshots the app itself produced, for the pixel-diff gate. "Optional but
       it is the main credit saver."
  DECISIONS TO COPY (learned the hard way): do NOT build a single-shot LLM acceptance judge; capture MUST be
    pinned (scale=1, fromSurface, font-render-hinting=none, motion killed, rAF-settled) "or every diff flags
    font/AA churn and the whole pixel gate is noise"; pixel-diff FIRST, vision SECOND; one bug -> one root
    cause (dedup by command/element, not by feature — "an open issue here"); vision sparingly, deterministic
    always.

KNOWN ROUGH EDGES (§9, honest): vision E2E is milestone-gated and rarely fires (no milestone's source plans
are all DONE) "so two codex accounts sit largely idle"; the daemon tester lanes have TWICE been "silently
dead" from refactor regressions that the FAIL-SOFT SWALLOW HID (a missing constructor arg, a moved import) —
"worth replicating: fail-soft hides exactly the breakage you most need to see" (a daemon-path integration
test now drives the real entrypoints through production default-construction); tester defect dedup-by-command
still open; the LETTERS plan-prefix list is hardcoded.


================================================================================
9. CONTRADICTIONS (both dates, both claims, no silent preference)
================================================================================

C1 — "NOT YET BUILT" vs BUILT. THE SHARPEST ONE.
  CLAIM A (2026-06-02, docs/strategy/WIRING-FIX-ELAI.md line 5): "This is GROUNDED DESIGN, not yet built. It
    is C/F-plan-sized." Reinforced by WIRING-FIX-PROPAGATION.md ("It is grounded design, not yet built; it
    does NOT propagate to the sibling workflows. Stage it as its own plan"), by WIRING-FIX-ELAI-PLAN.md
    ("Plan only; NO Rust edits yet... This plan is NOT built."), and by WIRING-FIX-ELAI.md §2 Layer 3 ("This
    is the layer ELAI does NOT yet have").
  CLAIM B (verified by me 2026-07-17, live tree): IT IS BUILT AND COMMITTED.
    - .elai_cc/crates/orchestrator/src/wiring_contract.rs carries WiringContractError::EmptyEntrypointSet
      (line 190) and ::AbsentEntrypointSet (line 281) returned as Err from WiringEntrypointSetLoader::load
      (lines 266-281) — i.e. Phase 1.3 "empty/absent = FAIL, not warn" LANDED. This directly contradicts
      C-27's own spec (docs/thefinalglance/C-plan/C-27_testing_trust_t0_wiring_contract.md:265: "[ ]
      EntrypointSetLoader: returns empty EntrypointSet with tracing::warn! when..."), which is now stale.
    - .elai_cc/crates/orchestrator/src/wiring_coverage.rs + .elai_cc/crates/cli/src/wiring_coverage_cmd.rs
      EXIST — i.e. Phase 2 (Layer 3 entrypoint coverage) LANDED.
    - .elai_cc/crates/contracts/tests/wiring_layer4_trace_contract.rs EXISTS — Phase 3 (Layer 4) LANDED.
    - .elai_cc/crates/orchestrator/src/evidence.rs carries layer3_entrypoint_covered (line 216) +
      layer4_trace_present, with a test named entrypoint_trace_gate_none_layer3_is_fail (line 1661) and a
      comment that these are POPULATED by task_runner::derive_layer3_entrypoint_covered from a
      wiring_evidence(leg=layer3_coverage) SSB row — i.e. Phase 4.1 (AutoAdvanceEvaluator requires Layer
      3+4) LANDED.
    - Committed: 981a412 touches both wiring_contract and wiring_coverage.
  HOW IT HAPPENED: not via the C/F-plan route the plan doc prescribed. WIRING-FIX-ELAI-PLAN.md line 8 says
    "Move/copy into the canonical ELAI plan dir (docs/thefinalglance/{C,F}-plan/) before orchestrating if the
    runner expects it there." That never happened — I found NO {C,F}-plan file for it. It was executed via
    workflow/reasoning/wiring-elai-edit-specs.md ("Generated: 2026-06-02 (WAVE-0 consolidation of 7 cluster
    verdicts: D-WC, D-EV, D-L3, D-L4, D-XL, D-EXH, D-FZ)") + workflow/reasoning/wiring-elai-adr.md — a
    parallel decision-cluster route.
  CONSEQUENCE FOR THE SWEEP: the four strategy docs describe a state of the world that is ~13 months of
    project-time out of date on its central claim. A reader who trusts them will conclude the ELAI runtime
    gate is unbuilt. It is built. The docs were never updated.

C2 — THE CHECKER'S DIAGNOSIS IS HISTORICAL, BUT A STALE INVOCATION SURVIVES DOWNSTREAM.
  CLAIM A (2026-06-02, WIRING-FIX-CLAUDE-CODE-WORKFLOW.md §1): wiring_contract_check.py is fail-open —
    SKIP-on-no-trigger, --run-trigger opt-in, UNVERIFIED on db-not-found, consumes excluded from the verdict.
  CLAIM B (verified live): all fixed. The file's own header now reads "db-not-found is FAIL (not
    UNVERIFIED), an incomplete contract is FAIL"; "--run-trigger  deprecated no-op: the trigger always runs
    (R3)"; line 643 "FAIL. UNVERIFIED is gone."
  THE SURVIVING STALENESS: docs/thefinalglance/F-plan/F-SUMMARY_INDEX.md:11 + :279 still define the F-plan
    done-gate as "wiring_contract_check.py --run-trigger PASSes", and G-SUMMARY_INDEX.md:13 does the same.
    The flag is now a documented no-op, so the invocation is harmless but misleading — anyone reading F/G
    will think the trigger leg is opt-in.

C3 — R1-R9 vs R1-R10 (the doctrine grew after the propagation shipped).
  CLAIM A (2026-06-02): every one of the four strategy docs says "R1-R9", and the Tier-B router line
    prescribes the text "Every rule (R1-R9) mandatory".
  CLAIM B (2026-06-16 onward): the live .claude/rules/wiring-enforcement.md is titled "THE TEN FAIL-CLOSED
    RULES (R1-R10, blocking)" and R10 (the integration-seam gate) was added from the Liberty 2026-06-16
    incident (7 cross-seam defects). Also added after the propagation: Layer 2b (frontend mount
    reachability), the Python wire-check leg, tauri_ipc_symmetry_check.py, ui-wire-smoke.mjs.
  CONSEQUENCE: the propagated Tier-A copies are frozen at the R1-R9 era unless re-propagated. VERIFIED: the
    apps/ copy has ZERO occurrences of R10 / Layer 2b / ui-wire-smoke (section 11.3).

C4 — THE 40-45% WIRE-DARK RATE IS BOTH THE HEADLINE AND SELF-REFUTED AT THE P0 TIER.
  CLAIM A (2026-06-02, research §1 + all four strategy docs): "Forty to forty-five percent wire-dark rate in
    ~187 shipped plans is not an execution failure; it is a definition-of-done failure." This is THE number
    the entire bundle is motivated by.
  CLAIM B (2026-06-03, workflow/sessions/s-20188/wiring/SUMMARY.md, i.e. ONE DAY LATER): "KEY REFRAME: the
    2026-05-30 '40-45% wire-dark' figure does NOT hold at the P0 tier today. These features are either
    already production-reachable or deliberately gated pending measurement/RUNPOD. The real remaining work
    is small + precise, not a 400-edit grind."
    The chunk-1 headline over 14 P0 gaps: ALREADY_WIRED 3 / INTENTIONAL_DARK 8 / ACCIDENTAL_DARK 2 /
    SUPERSEDED 1 / NOT_SHIPPED 0 / APPLIED EDITS 0. The two genuinely-dark: B40-003 (frame_idea_with_fsm has
    ZERO production callers; task_runner.rs:5746 still calls bare frame_idea(); FsmSubstate::Framing
    unreachable -> IdeaFramingWindow dead) and E49-005 (record_stable_pass_at_1 /
    record_harness_attribution_tuple / record_stop_verified have ZERO callers, stable_pass.rs:147/194/245).
    The re-audit's own reframe of the gap: 9 R9 VIOLATIONS — env gates intentionally dark but MISSING
    wire-dark-allowlist.toml rows (ELAI_CONTEXTUAL_RETRIEVAL_ENABLE, ELAI_CAST_CHUNKER_ENABLE,
    ELAI_IDEA_FRAMING_ENABLE, ELAI_CONFORMAL_REFUSAL_ENABLE, ELAI_REFUSAL_SLA_TICK_ENABLE,
    ELAI_DURABILITY_OBSERVABILITY_ENABLE, ELAI_DARWINIAN_LOOP_ENABLE + feature darwinian-evolve,
    ELAI_WEB_RESEARCH_ORCHESTRATOR_ENABLE, ELAI_CODING_METRIC_NAMED_TOKEN_ENABLE).
  NEITHER IS OBVIOUSLY WRONG: A is a whole-corpus rate over 187 plans (2026-05-30 night-run); B is a
    re-verification of the P0 TIER ONLY (14 gaps) three days later after intervening fixes. They are not the
    same measurement. But the bundle keeps quoting A as live motivation and never records B. The re-audit's
    own framing ("that audit is 3 DAYS STALE") is the honest reading: the 40-45% was real at audit time,
    largely closed at P0 by 06-03, and the residual was mostly a BOOKKEEPING gap (missing R9 allowlist rows),
    not dead code.
  This matters strategically: the research doc itself says (§6) "The 40-45% rate in ELAI's 187 plans is a
    self-measured internal figure, not validated against external baselines" and instructs "measure your own
    rate before/after (wire-dark plans / total) so the fix is evaluated on data, not vibes." That
    before/after instrumentation is the thing that would settle A-vs-B, and I found no evidence it exists as
    a live metric (UNVERIFIED: I did not exhaustively search for a wire-dark-rate SSB action_type).
  LIVE DATA POINT I did verify: .elai_cc/wire-dark-allowlist.toml now carries 136 [[...]] rows;
    .elai_cc/smoke-manifest.toml carries 9. The 136 vs 9 asymmetry is itself notable — the R9 registry is
    heavily used, the Layer-3 smoke manifest barely.

C5 — INTERNAL: FOWLER RELEASE-TOGGLE LIFESPAN.
  research §3.7 says release toggles "must die within 1-4 weeks"; the same doc's source entry [37] says
  "1-2 week release toggle lifespan... confirmed verbatim". Minor, but the two are not the same claim and
  both are presented as verbatim-confirmed.

C6 — THE PROPAGATION'S "ZERO-FP FLOOR" vs ITS OWN ADDENDUM.
  CLAIM A (§4, base design): the floor is "deliberately LOW-RECALL / ZERO-FALSE-POSITIVE".
  CLAIM B (addendum c, V1, same file, same day): "Section 4's 'zero-FP target' was overstated... low, never
  zero", with Rust named as the weakest leg. The base section was NOT edited; a reader of §4 alone gets the
  retracted claim. (This is self-correction working — but it is a contradiction inside one file.)


================================================================================
10. PLAN TRACING — did each idea become a plan unit, and what is its status?
================================================================================
Method: read ELAI-SPEC-OVERVIEW.md in full; grepped all 17 letter indexes (B-PLAN-SUMMARY.md +
{C..Q}-SUMMARY_INDEX.md) + phase-index-MASTER.txt for the bundle's concepts; opened the matching plan files
where the row mattered; checked the live tree for the artifact.

10.1 WIRING ENFORCEMENT (the harness leg) -> BUILT, NOT AS A LETTER PLAN.
  It never became a lettered plan unit. It is a RULE + a checker set, shipped directly into .claude/.
  VERIFIED PRESENT: .claude/rules/wiring-enforcement.md; .claude/tools/gate_freeze_check.py;
  .claude/gate-manifest.txt; .claude/tools/wiring_contract_check.py (796 lines, fixed per §3);
  .claude/agents/wiring-contract-validator.md; .claude/tools/production_caller_check.py;
  .claude/tools/python_wire_check.py; .claude/tools/maintainability_check.py;
  .claude/tools/traced_test_assert_check.py; .claude/tools/plan_gate_check.py;
  .claude/tools/tauri_ipc_symmetry_check.py; .claude/tools/svelte_mount_reachability.py;
  .claude/tools/claims_policy_lint.py.
  THE MANIFEST IS FAR BIGGER THAN THE PROPAGATION DOC IMPLIES — it now freezes: the wiring rule + checker +
  freeze tool + validator agent; the tracing-test rule + checker; the maintainability rule + checker;
  slop_semantic_check.py (from CONDUCTOR-QUALITY-GAP-2026-06-29 — the TT-*/TV-* slop classes "the merge-lane
  GLM review kept catching that the static per-task gate was blind to"); .elai_cc/smoke-manifest.toml;
  .elai_cc/wire-dark-allowlist.toml; .elai_cc/elai_cc.toml (D-FZ: "the wiring config (BFS-root entrypoint
  set + Layer-3 coverage thresholds) is gate-owned"); .elai_cc/scripts/locked_keys_registry.toml ("freeze it
  against itself so a run cannot quietly drop a lock row"); .elai_cc/crates/orchestrator/src/trial_ledger.rs
  (H-17); the H-18 deflation gate; and more. R8 is the most thoroughly-realised rule in the set.

10.2 THE ELAI RUNTIME REDESIGN (WIRING-FIX-ELAI-PLAN Phases 0-4) -> BUILT, via decision clusters not a plan.
  Executed through workflow/reasoning/wiring-elai-edit-specs.md (WAVE-0 consolidation of 7 cluster verdicts:
  D-WC wiring_contract.rs, D-EV evidence/task_runner/autonomous_gate, D-L3 wiring_coverage.rs +
  wiring_coverage_cmd.rs, D-L4 wiring_layer4_trace_contract.rs + the tracing-test rule/checker, D-XL+D-FZ
  cross_lang/CI/data, D-EXH the dispatch-enum lock test) + workflow/reasoning/wiring-elai-adr.md.
  Evidence of real engineering friction recorded there, worth knowing:
    - "C4 action_type name divergence: D-EV (EntrypointCoverage/WireDarkTrace) vs D-FZ
      (wiring_audit_completed/wiring_coverage_measured/wire_dark_rate_sampled) vs D-L4 (wiring_evidence).
      Pick ONE canonical set." Resolved (adr.md:13): "Canonical = wiring_evidence + wiring_violation."
    - "C8 elai_cc.toml [wiring.entrypoints]: D-XL (6 symbols) vs D-FZ (2 symbols)". Merged to D-XL's 6
      verified symbols (cli main, fsm::transition, ipc_registry::dispatch, in_process::dispatch_tool, tauri
      main, tauri lib::run) + a D-FZ [wiring.layer3] block (enabled, fail_under_lines,
      new_fn_zero_hit_is_block, instrument_profile, llvm_cov_json_flag=--json).
    - "D-FZ self-lock needs an L3_OVERRIDE / ELAI_GATE_FREEZE_SKIP=1 documented escape for legitimate future
      registry edits." UNVERIFIED whether that escape was implemented.
  The C-27 origin unit (docs/thefinalglance/C-plan/C-27_testing_trust_t0_wiring_contract.md) IS a real plan
  file and is where the substrate came from; its spec text is now stale on the empty=warn point (C1).

10.3 THE DETERMINISM DOCTRINE -> A LENS DOC + A PLANNED-BUT-UNBUILT ENFORCEMENT WAVE. THE KEY FINDING.
  The lens and the AUX are docs/strategy/ files only. Their enforcement DID become plan units — J-43..J-47,
  the DOCTRINE-PROPAGATION META-WAVE (docs/thefinalglance/J-plan/J-SUMMARY_INDEX.md:1147-1267), authored
  2026-06-16, one file per component of the Approach-B design, each with its EXP falsifier as step 1:
    J-43 doctrine-propagation keystone [Track 1, the LENS] — .claude/rules/determinism-doctrine.md +
      .claude/tools/doctrine_floor_check.py (DD-1..DD-5) + .claude/doctrine-debt-ledger.toml + the auditor
      re-run + the gate-manifest freeze. WAVE DPW-1, effort M. EXP-DP1 = the FP-baseline sweep.
    J-44 reasoning-method artifact-shape [Track 2, the METHOD] — .claude/rules/reasoning-method.md +
      .claude/tools/reasoning_artifact_shape_check.py + THE RANDOM-SAMPLE SUBSTANCE-AUDIT PANEL by a
      NON-author + 4 lifecycle injections. WAVE DPW-2, effort M-L. EXP-DP2.
    J-45 cross-family advisory calibration [the D5 leg] — the 100-300-trace human-labeled CALIBRATION
      CORPUS -> Cohen kappa>0.6 gate + ESCALATION teeth + single-family-degraded debt honesty. WAVE DPW-3,
      effort M. EXP-DP3 measures kappa on a held-out slice.
    J-46 doctrine-debt-rate measurement [D4 on the propagation itself] — .claude/tools/doctrine_debt_rate.py
      + the only-WIRED-first baseline sweep across B..K + the FEATURE-MAP conformance stamp + the
      implausible-zero self-test. WAVE DPW-4, effort M-L. EXP-DP4 IS the baseline.
    J-47 program falsifier + per-axis discipline [M9 at program altitude] — EXP-DP5 = the V5 A/B/C
      experiment "= step 1 of the WHOLE program, nothing blocking-default-ON before it passes" + THE
      PER-AXIS DOCTRINE-FAMILY DISCIPLINE (V4). WAVE DPW-5, effort M.
    Net 0 new SSB across the wave (harness leg). All ship OBSERVE-ONLY first.
  STATUS: NOT BUILT. VERIFIED MISSING on the live tree, every one:
    .claude/rules/determinism-doctrine.md            MISSING
    .claude/tools/doctrine_floor_check.py            MISSING
    .claude/rules/reasoning-method.md                MISSING
    .claude/tools/reasoning_artifact_shape_check.py  MISSING
    .claude/doctrine-debt-ledger.toml                MISSING
  --> THE DETERMINISM DOCTRINE HAS NEVER BEEN ENFORCED. It is exactly the passive lens its own propagation
      design says is worthless ("A passive lens is a soft gate, and a soft gate that an agent applies to its
      own work rubber-stamps -- which is the precise failure the doctrine itself warns about (D1)"). Thirteen
      months on, the doctrine that the owner called "the biggest thing" is still a document you hand to a
      model and hope.

10.4 THE DD WAVE (the per-axis doctrines the propagation must not re-author) -> PLANNED.
  J-39 (decomposition / node-admission sizing), J-40 (research / extractive compression), J-41 (grounding /
  the cheap three-tier stack), J-42 (reasoning / within-step distill-reset). Each is the determinism doctrine
  applied to one axis, each ends "ELAI-DETERMINISM-DOCTRINE: the canonical <X> application"
  (J-SUMMARY_INDEX.md:1074 confirms the pattern for J-39: "the canonical 'gate disposes on size'
  application"). ELAI-SPEC-OVERVIEW.md §J confirms both waves as design: "Determinism doctrine propagation —
  the 'model is a cog' doctrine made structurally enforced at every lifecycle seam (a thin frozen floor + a
  cross-family advisory + a fail-closed debt ledger + a measured rate + a program falsifier), and the DD
  triad + reasoning axis". UNVERIFIED build status of J-39..J-42 individually (I did not check each seam);
  the doctrine-propagation artifacts they depend on are absent, which bounds how enforced they can be.

10.5 THE DISTILL-RESET / CONTEXTCONTROLLER PRIMITIVE -> PARTIALLY PLANNED, the keystone NOT FOUND.
  J-42 exists as the reasoning-axis plan and is named repeatedly. The doc's P3 "ContextController +
  CarryStateContract (the P3 primitive) = the keystone new plan" — I found NO plan unit by that name in any
  index. The nearest specced surface is H's "Context lifecycle — regime-aware in-session context management
  (local window-minimization vs cloud cache-stable) + cross-session sleep-time handoff with a grounded
  topic-digest" (ELAI-SPEC-OVERVIEW.md §H) and H-50/H-51, plus L's "context-core inversion (an immutable
  transcript + a pure fold-view projection, making cache-stability structural)". THE KEYSTONE UNIFYING
  CONTROLLER IS AN IDEA WITH NO PLAN UNIT — an explicit finding per the task's instruction. Its sibling
  design file (context-controller-carry-state-design-2026-06-16.md) is referenced by the propagation doc
  (§7, §10) but I did not read it (out of the named Layer-1 set); UNVERIFIED whether it contains the plan.

10.6 THE E2E TESTING SYSTEM -> BUILT (conductor/), NOT A LETTER PLAN.
  All named machinery VERIFIED present at conductor/: cdp_action_driver.mjs, cdp_driver.py,
  checkpoint_map.py, test_modes.py, tester_lane.py, tester_metrics.py, tester_pass.py, tester_runtime.py,
  tester_sandbox.py, tester_vision.py, trigger_safety.py, vision_gate.py. Rubric VERIFIED at
  workflow/reasoning/ELAI-TEST-CHECKPOINT-MAP.json. It is daemon machinery, not a plan unit.

10.7 THE RRA / ATTESTATION (the trust-DISTRIBUTION leg — what makes the core sellable, not just internal).
  G-2 "Reproducible-Run Artifact (RRA) [THE KEYSTONE]" (G-SUMMARY_INDEX.md:38): a single portable
  msgpack+zstd file bundling the SSB action stream + ELAI_SEED + HardwareClass + system_fingerprint pin +
  model pin + an evidence manifest of every F-gate's blake3 verdict hash (F-1 source_blake3, F-4
  stdout_hash, F-9 dep_name_hash, F-10 delta_blake3) + a blake3 root over the bundle. New `elai
  run --emit-rra` + `elai rra show` + `elai replay --verify`. "claude-replay is viewer-only (no
  re-exec/seed/divergence) = first-in-category."
    ITS HONEST SCOPE (GD-5), verbatim and load-bearing: "cloud-lane runs are reproducible-ACTION-STREAM, not
    reproducible-TOKENS (Anthropic system_fingerprint opacity); SSB stores HASHED tool I/O so the guarantee
    may be verify-divergence not full re-execute for body-reading steps -- state which, do not over-claim."
  G-8 "Signed in-toto provenance attestation + Article-50 verifier CLI": a custom in-toto v1 predicate
    (subject bound by blake3/sha256 digest; carries model id, ELAI_SEED, an SLSA-VSA-style
    PASSED/verifiedLevels rollup of every F-gate, evidence hashes, SSB event id) -> a git commit trailer
    (digest only) + a sidecar file + a new SSB action_type; signed via a LOCAL blake3-Merkle proof (offline
    default) or opt-in cosign keyless. New `elai verify-attestation` re-checks OFFLINE. Positioning
    discipline recorded: "POSITION AS 'designed to support Article-50 marking', NOT 'certified compliant'
    (final Code June 2026; source code is not an enumerated example). Penalty tier is EUR 15M/3% (Art-99),
    NOT 35M/7%."
  LIVE: .elai_cc/crates/orchestrator/src/rra.rs and .elai_cc/crates/orchestrator/src/attestation.rs BOTH
    EXIST (VERIFIED). Memory corroborates ("rra.rs/attestation.rs confirmed SHIPPED"). UNVERIFIED: whether
    `elai replay --verify` and `elai verify-attestation` are wired to real CLI entrypoints — I did not drive
    them. NOTE the E2E deterministic lane's mode 4 IS `replay --verify <recorded-run>`, which implies a live
    entrypoint.
  DOWNSTREAM: I-40 "trust-layer wrapper + rra spec" — "run any external agent's tool traffic through ELAI's
    taint + write-gates + RRA emission (the wedge unbundled)" (ELAI-SPEC-OVERVIEW §I). THIS IS THE MOST
    DIRECT EXISTING ANSWER TO THE OWNER'S ASK and it is already a planned unit. K-20 "re-executable-RRA
    differentiator [PROVE+PACKAGE; THE WEDGE]". M-5 "commercial packaging + verifier OSS + trust
    continuity". M-6 "evidence format as a versioned open standard [the schema, not just the verifier; the
    format-moat play]" — "so third-party tools can emit ELAI-verifiable evidence." Q also carries "a locked
    size-and-dependency ceiling on the standalone verifier (the tool that audits every run stays smaller and
    more dependency-light than what it audits)."
  --> THE STRATEGIC POINT THE SWEEP SHOULD CARRY: "make the trust core usable on any other harness" has
      TWO existing, distinct answers in the corpus, and they are at different altitudes:
        (a) the HARNESS-DOCTRINE port (WIRING-FIX-PROPAGATION Tier A/B) — done once, 2 files, decayed;
        (b) the RUNTIME-WRAPPER port (I-40 trust-layer wrapper + M-6 evidence format as an open standard +
            M-5 standalone OSS verifier) — planned, far more valuable, unbuilt.
      The owner's phrasing ("usable on any other harness") maps onto (b) much more than (a).

10.8 CROSS-CHECK against ELAI-SPEC-OVERVIEW.md — the bundle IS the spine, self-described.
  The thesis paragraph names all three properties verbatim: "off-agent evidence — a gate measures an artifact
  the generator did not write... fail-closed — missing evidence is a FAIL, never a SKIP; the gate that judges
  a run is frozen and read-only to that run... determinism — a seeded clock + entropy source, an append-only
  journal, and a reproducible-run artifact make a whole session byte-recomputable." And the closing line:
  "The invariant across all of it: the model generates and selects; the deterministic harness sizes, gates,
  journals, and decides 'done' off evidence the model did not author — measured, fail-closed, replayable."
  The substrate this rests on is Phase 0's ENTROPY QUARANTINE: "all time and randomness flow through injected
  ClockProvider + EntropySource traits, so a seeded run is byte-identical across executions — the foundation
  every later 'reproducible / replayable' claim rests on." NOTE: commit 981a412 (the most recent touching
  wiring) is an entropy-leak quarantine batch — "leak count 83 -> 53" — i.e. the determinism foundation was
  still leaking at 83 sites and is now at 53. That is a live, measured, honest number for how complete the
  bottom of the spine is. UNVERIFIED: current leak count.


================================================================================
11. WHAT EXISTS, WITH REAL PATHS (the artifact inventory, verified 2026-07-17)
================================================================================

11.1 PRESENT AND VERIFIED
  Rules (.claude/rules/):        wiring-enforcement.md, production-caller-gate.md, tracing-test-assert.md,
                                 maintainability-review.md, plan-standard.md, agent-dispatch-preamble.md,
                                 dispatch-core.md, proof-spine.md, conditional-routing.md
  Checkers (.claude/tools/):     gate_freeze_check.py, wiring_contract_check.py (796 lines, fixed),
                                 production_caller_check.py, python_wire_check.py,
                                 traced_test_assert_check.py, maintainability_check.py, plan_gate_check.py,
                                 tauri_ipc_symmetry_check.py, svelte_mount_reachability.py,
                                 claims_policy_lint.py, slop_semantic_check.py, elai_status_query.py,
                                 proof_spine_status.py, impact_radius.py, cross_lang_diff.py,
                                 failure_taxonomy.py, spec_link_check.py, elai_map_freshness.py,
                                 bn_status_report.py, visual/ui-diff.mjs, visual/ui-wire-smoke.mjs
  Agents:                        .claude/agents/wiring-contract-validator.md
  Frozen data:                   .claude/gate-manifest.txt; .elai_cc/wire-dark-allowlist.toml (136 rows);
                                 .elai_cc/smoke-manifest.toml (9 features); .elai_cc/elai_cc.toml;
                                 .elai_cc/scripts/locked_keys_registry.toml
  Rust runtime:                  .elai_cc/crates/orchestrator/src/wiring_contract.rs (EmptyEntrypointSet +
                                 AbsentEntrypointSet errors; ObservableEffectRunner; BrokenObservable);
                                 .elai_cc/crates/orchestrator/src/wiring_coverage.rs;
                                 .elai_cc/crates/cli/src/wiring_coverage_cmd.rs;
                                 .elai_cc/crates/cli/src/wiring_audit.rs;
                                 .elai_cc/crates/contracts/tests/wiring_layer4_trace_contract.rs;
                                 .elai_cc/crates/orchestrator/src/evidence.rs (layer3_entrypoint_covered /
                                 layer4_trace_present, entrypoint_trace_gate_none_layer3_is_fail);
                                 .elai_cc/crates/orchestrator/src/rra.rs;
                                 .elai_cc/crates/orchestrator/src/attestation.rs
  E2E daemon:                    conductor/{tester_sandbox,cdp_driver,tester_vision,tester_runtime,
                                 vision_gate,tester_lane,checkpoint_map,test_modes,trigger_safety,
                                 tester_pass,tester_metrics}.py + conductor/cdp_action_driver.mjs;
                                 workflow/reasoning/ELAI-TEST-CHECKPOINT-MAP.json
  Decision record:               workflow/reasoning/wiring-elai-adr.md; workflow/reasoning/
                                 wiring-elai-edit-specs.md; workflow/sessions/s-20188/wiring/ (SUMMARY.md,
                                 WORKLIST.md, AGENT-PROTOCOL.md, reports/, wire-dark-allowlist.PROPOSED.toml)

11.2 PLANNED, VERIFIED ABSENT (the whole doctrine-propagation wave)
  .claude/rules/determinism-doctrine.md | .claude/tools/doctrine_floor_check.py |
  .claude/rules/reasoning-method.md | .claude/tools/reasoning_artifact_shape_check.py |
  .claude/doctrine-debt-ledger.toml | .claude/tools/doctrine_debt_rate.py (J-43..J-47)

11.3 THE TIER-A DRIFT (measured — the "copy verbatim" file is now three different files)
  /home/user/teikoku/.claude/rules/wiring-enforcement.md        63 lines  md5 f59446db  R10 yes, Layer2b no
  .claude/rules/wiring-enforcement.md   65 lines  md5 93d8fc2f  R10 yes, Layer2b yes
  /home/user/teikoku/apps/.claude/rules/wiring-enforcement.md   60 lines  md5 8780b384  R10 NO,  Layer2b NO
  The apps/ copy is frozen at the R1-R9 era — it is the 2026-06-02 propagation artifact, never refreshed. It
  has neither R10 (the integration-seam gate born from a real 7-defect incident) nor the frontend legs. This
  is the propagation's own failure mode made concrete: a doctrine copied verbatim to N roots has N-1 stale
  copies the moment the source improves, and NOTHING in the Tier-A/B design addresses re-propagation. There
  is no version stamp, no sync check, no manifest of where the copies live. The gate-manifest freeze
  protects each copy from being weakened LOCALLY while doing nothing about the copies DIVERGING.


================================================================================
12. STALENESS — every path in this bundle is wrong
================================================================================
  Every one of the ten Layer-1 docs was written against F:\.ELAI_workflow (workflow root) with the project
  at F:\.ELAI_workflow\.elai_cc\. The live root is /home/user/teikoku/elai with the project at
  .elai_cc. VERIFIED ABSENT on this machine: /mnt/f/.ELAI_workflow.
  THE PROPAGATION MANIFEST'S THREE TARGETS ARE ALL GONE. VERIFIED ABSENT: /mnt/d/.apps_workflow,
  /mnt/d/.liteGAME_workflow, /mnt/d/.REGRET_workflow. The tree migrated to /home/user/teikoku/{apps, elai,
  conductor, liberty, packs, work_games, work_trading, ...}. So:
    - D:\.apps_workflow  -> apps/ survives WITH Tier A (VERIFIED: apps/.claude/rules/wiring-enforcement.md +
      apps/.claude/tools/gate_freeze_check.py + apps/.claude/gate-manifest.txt all present) — the
      propagation SURVIVED the migration for this target, in its 2026-06-02 form.
    - D:\.liteGAME_workflow -> NO liteGAME root exists. Nearest: work_games/ and packs/ (Unity pack rules
      live at the teikoku git-root .claude/rules/unity-*.md). UNVERIFIED whether the liteGAME Tier-A copy
      survived anywhere.
    - D:\.REGRET_workflow -> NO REGRET root exists. UNVERIFIED whether its copy survived.
    - The teikoku git-root /home/user/teikoku/.claude/rules/ now carries wiring-enforcement.md as an
      always-loaded CORE rule (per its conditional-routing.md, which states core rules "sit at the git-root
      core .claude/rules, an ancestor on every project walk"). This is a BETTER propagation mechanism than
      copy-to-N-roots and it post-dates the propagation doc: one ancestor copy governs every project on the
      walk. The propagation manifest's Tier-A/B design is architecturally obsolete on this machine — but
      apps/ still carries its own now-stale duplicate, so both mechanisms are live simultaneously and they
      disagree (section 11.3).
  Other stale references I hit: WIRING-FIX-CLAUDE-CODE-WORKFLOW §1 cites wiring_contract_check.py by line
  number (568, 571-572, 638, 599-600, 584-596, 526-537, 343, 665-668, 370) — the file is now 796 lines and
  those line refs no longer resolve to the cited code. The E2E guide's "All file paths below are under
  F:\.ELAI_workflow\conductor\" -> live at conductor/ (contents verified).
  The distill-reset doc cites its input at C:\Users\user\Downloads\deep-research-report (23).md — UNVERIFIED,
  almost certainly gone; that report is the SOLE source for its numbers, so those numbers are now
  unrecheckable at source.


================================================================================
13. THE HONEST LIMITATIONS (carried verbatim; do NOT sanitize these into strengths)
================================================================================
Every doc in this bundle carries a residual-risk section and they say the same four things. They are
load-bearing: the corpus's credibility rests on having stated them.

  1. NO SOUND STATIC REACHABILITY FOR RUST. "Go's deadcode implements sound RTA from main(). No equivalent
     exists for Rust with standard library linking, tokio async runtime, and dyn Trait dispatch. The
     SCIP-reference-count approach is a necessary approximation but is not sound." Consequence: "Layer 1
     will have false negatives on some genuinely unreachable code and false positives on some
     trait-dispatched code. Layer 3 (runtime coverage) is the compensating control." -> "treat it as
     load-bearing, not optional."
  2. SCIP HAS NO CALL-GRAPH EDGES (HIGH RISK). "SCIP stores per-token reference positions but does NOT store
     call graph edges. A pub impl method called only via dyn Trait vtable will have zero SCIP direct
     references and will be falsely flagged as dead by any SCIP-based tool. Any gate using elai.map to check
     for symbol callers will FAIL-OPEN on trait-object dispatch." Mitigation: [wiring.entrypoints] must
     enumerate trait-dispatch sites — "can go stale. Audit it."
  3. THE SMOKE HARNESS ONLY PROVES WHAT IT EXERCISES. "An agent that writes a smoke test that bypasses the
     actual new feature path will pass Layer 3 while the feature remains wire-dark. The smoke-test script
     must be authored and maintained by the auditor as a first-class repo artifact, not generated per-plan by
     the same agent being evaluated." And: "a lazy manifest can still under-cover. Track coverage of the
     manifest itself." (Live data: 9 smoke-manifest features vs 136 allowlist rows.)
  4. WIRE-DARK HAS NO EXTERNAL BENCHMARK. "arXiv:2601.15195 (MSR 2026, 33,596 agent-authored PRs) found no
     taxonomy category for wire-dark code in the largest empirical study of agent PR failures. There is no
     established benchmark, no standard measurement tool, and no baseline rate published in the literature.
     The 40-45% rate in ELAI's 187 plans is a self-measured internal figure, not validated against external
     baselines." -> "instrument the rate (wire-dark plans / total) so this redesign is judged on measured
     deltas, not faith."
  5. THE FRONTEND LEGS ARE UNSOUND THE SAME WAY (live rule, added later): "tauri_ipc_symmetry_check cannot
     resolve a non-literal invoke(cmd) (advisory DYNAMIC note), and svelte_mount_reachability lands a
     runtime-built <svelte:component this={map[key]}> mount in UNREACHABLE (WARN) not REACHABLE when its
     module is not statically imported on a live path... 'Perfect' frontend wiring enforcement is not
     claimed; these raise the floor from static-only to having the same runtime-proof backbone the backend
     has."
  6. THE DOCTRINE CANNOT BE FULLY MECHANIZED (propagation §9). "The semantic core is unmechanized by design;
     the floor catches only the structural tells. A subtle boundary violation (a proxy dressed as an
     objective, a model judgement laundered through a mechanical-looking wrapper) can pass the floor and a
     rushed advisory review." Plus doctrine-theater ("the DECISION LEDGER could be filled mechanically to
     pass"), cross-family fragility ("gpt-bridge / a second family is rate-limit-fragile and occasional; the
     advisory review degrades to single-family when the second family is unreachable, which weakens D5.
     Record that as a debt row... do not silently treat single-family advice as cross-family"), and the
     pre-doctrine backlog (~213 plans authored before 2026-06-16).
  7. THE PROPAGATION'S CENTRAL CLAIM IS AN UNPROVEN BET (V6, verbatim): "no direct precedent exists in the
     autonomous-agent-harness domain (the components are proven elsewhere; their composition HERE is not).
     Success 'hinges on execution: audits, culture, tight integration'... We are not buying a guaranteed win."
  8. THE CITATION AUDITOR'S OWN DISCARD LIST (research, "CITATIONS EXPLICITLY DISCARDED OR DOWN-WEIGHTED") —
     preserve this; it is the strongest evidence the research was honestly done:
       - "73% of feature flags are never removed" attributed to martinfowler.com: REFUTED. Does not appear in
         the Fowler article. Originates from FlagShark (vendor marketing).
       - "Uber Piranha" attributed to the Fowler article URL: REFUTED (Piranha is real, PLDI 2024, but is not
         mentioned in that article).
       - Quoted phrases "No call graph data..." / "No interprocedural analysis..." attributed to
         deepwiki.com/rust-lang/rust-analyzer: REFUTED — "the page exists but these exact sentences do not
         appear on it. The underlying SCIP-has-no-call-graph claim is substantively correct (verified against
         scip.proto) and is stated in this report on that basis, not on the DeepWiki citation."
       - OTel span names "steps/toolCallRequest"/"steps/toolCallResult" with attributes "toolId", "inputs",
         "executionId": REFUTED (FABRICATED). Actual convention: gen_ai.operation.name='execute_tool' with
         gen_ai.tool.name, gen_ai.tool.call.id, gen_ai.tool.output.
       - SWE-bench Verified scores (Claude Code 78.4%, Devin 60.8%): STALE (leaderboard top 93.9% June 2026).
       - cargo-mutants v25.1.0 as introducing --in-diff: REFUTED — introduced in v23.11.1. "Version numbers
         as cited are fabricated."
       - Atlassian 80% mutation threshold as merge-blocking policy: OVERSTATED (appears only in a Rovo Dev
         CLI demo prompt).
       - Bogard "Vertical Slice Architecture" date 2023: REFUTED (April 19, 2018).
       - The ArchUnit getAccessesToSelf() code snippet: REFUTED (API real, snippet not in the guide).
     Also flagged inline: walking skeleton originates with Cockburn (Crystal Clear, 2004), NOT GOOS; the
     knip "mark-and-sweep" phrasing is the auditor's characterization not knip's; the pytest-cov claim that
     docs "explicitly state no mechanism exists" is NOT verbatim ("the page simply does not discuss this
     topic"); [35]'s Drift quote is a citation misrouting (it appears in builtin.com, not aihero.dev).
  9. TOOLS EXPLICITLY RULED OUT (do not re-adopt without re-checking): warnalyzer ("last release 0.2.0 May
     2021; nightly Rust required; save-analysis backend removed from Rust in February 2023. NOT viable as a
     CI gate"); cargo-call-stack ("of very limited use on programs that link to the standard library" due to
     trait-object dispatch; experimental nightly; last tested nightly 2023-11-13. "NOT viable for a
     std-linked async Rust workspace"); malabi ("ARCHIVED as of 2024. Do not adopt"); tracetest (requires
     external infra; self-hosted Core "not suitable for production" per its own README); Piranha (real and
     TS IS supported, correcting the earlier claim — but "it is a CLEANUP tool, not a PREVENTION gate. It
     fires after a human identifies a stale flag; it cannot prevent wire-dark code from entering at merge
     time"). Mutation testing is ruled out AS A WIRING GATE for a structural reason worth remembering: "A
     function with 100% mutation kill score and zero production callers is still wire-dark; the unit test
     kills every mutant perfectly." Only mutation kill from INTEGRATION tests would count.

THE RESEARCH'S OWN PROVENANCE (so its weight is legible): generated 2026-06-02 by the
wiring-enforcement-research multi-agent workflow (run wf_7acf1e7b-153): 9 web-grounded researchers -> 9
hostile citation auditors (one per topic) -> 1 synthesizer. 19 agents, ~1.09M subagent tokens, 560 web tool
calls. "Every load-bearing claim was adversarially citation-checked... Some arXiv ids post-date the
assistant's Jan-2026 training cutoff and were confirmed by the auditor via live fetch, not from memory."


================================================================================
14. WHAT A READER SHOULD DO WITH THIS
================================================================================
Not recommendations about direction — just the actionable facts the sweep surfaced, ranked by how load-bearing.

  1. THE PORTABLE CORE IS ALREADY IDENTIFIED AND IT IS TWO FILES. If the goal is "usable on any other
     harness" at the DOCTRINE grain, WIRING-FIX-PROPAGATION.md Tier A is the answer and it is done:
     wiring-enforcement.md + gate_freeze_check.py, plus a router line and (optionally) a planning splice.
     The mechanism has since been superseded on this machine by the teikoku git-root core-rules ancestor,
     which is strictly better (one copy, every project). The remaining Tier-A work is a re-propagation and a
     drift check, not a design.
  2. THE MORE VALUABLE PORT IS I-40 + M-6 + M-5, AND IT IS UNBUILT. "Run any external agent's tool traffic
     through ELAI's taint + write-gates + RRA emission (the wedge unbundled)" (I-40) is the corpus's own
     answer to the owner's exact phrasing, at the RUNTIME grain rather than the doctrine grain. M-6 makes
     the evidence format a versioned open standard "so third-party tools can emit ELAI-verifiable evidence";
     M-5 ships the standalone OSS verifier; Q caps its size/deps. That is a coherent four-unit product and
     none of it is built.
  3. THE DETERMINISM DOCTRINE IS UNENFORCED AND ITS OWN DESIGN SAYS THAT MAKES IT WORTHLESS. J-43..J-47
     exist as well-formed plans with falsifiers, corrections already folded in (V1/V2/V3/V4/V5), and every
     artifact verified absent. The wave is Approach-B-shaped and self-consistent; the only thing missing is
     execution. Note J-47 is explicitly gated first: "EXP-DP5 = step 1 of the WHOLE program, nothing
     blocking-default-ON before it passes."
  4. THE DOCS LIE ABOUT BUILD STATE. Four strategy docs say the ELAI runtime gate is unbuilt; it is built
     and committed. Anyone using docs/strategy/ as a status source will be wrong.
  5. THE MOTIVATING NUMBER IS CONTESTED AND UNINSTRUMENTED. 40-45%/187 plans (2026-05-30) vs "does NOT hold
     at the P0 tier" (2026-06-03). The research doc's own instruction — instrument the rate so the redesign
     is judged on measured deltas — appears unexecuted. Every external claim built on 40-45% is currently
     resting on a figure the project itself re-audited and reframed one day later.
  6. THE ASYMMETRY WORTH LOOKING AT: 136 wire-dark-allowlist rows vs 9 smoke-manifest features. R9 (register
     the dark flag) is heavily used; Layer 3 (prove the feature ran) is barely used. The registry is doing
     the work the runtime proof was supposed to do.


END OF SWEEP 1.
