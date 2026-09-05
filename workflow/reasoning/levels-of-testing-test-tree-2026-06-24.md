LEVELS-OF-TESTING / PLAN-BOUND TEST TREE — reasoning doc
Author: main LLM, 2026-06-24. Origin: user idea raised mid-teach session.
Grounding source: full read of docs/thefinalglance/ELAI-FEATURE-MAP.md (2026-06-24)
+ the house wiring/trust doctrine (.claude/rules/wiring-enforcement.md R10,
production-caller-gate, visual-ui-loop.md). Every ELAI feature cited by its
plan-id is from that read; nothing here is invented.

This is a REASONING doc, not a plan file. It decides whether the idea is worth a
plan and in what shape. It does not author plan units, does not transition the FSM.


1. THE IDEA, RESTATED PRECISELY
The user proposed, in their words: tests live at every decomposition level (concept
> phase > step > baby-step); authored alongside the decomposition; executed in the
OPPOSITE order (bottom-up) as implementation lands — a baby-step test proves that
step works, then as steps assemble the higher level is re-tested, bubbling up to the
concept. On every new implementation the relevant tests run again. On top of that a
full end-to-end "fake user" suite (real clicks, real app interaction) that GROWS as
the plan is implemented, so the system continuously proves CONSISTENCY across the
whole plan, not only that things are wired.

Decomposed into three distinct claims:
  IDEA-A  a test exists per node of the decomposition tree, authored top-down with
          the tree.
  IDEA-B  execution rolls UP bottom-up: pass a node, then re-prove its parent after
          assembly; ancestors re-tested as descendants land (continuous regression).
  IDEA-C  a living fake-user e2e suite that accretes a scenario per shipped slice and
          re-runs cumulatively, proving cross-level + whole-plan consistency.


2. HONEST OVERLAP — MOST OF THIS IS ALREADY SPECIFIED IN ELAI (scattered)
The user independently re-derived a large fraction of ELAI's existing test/trust
design. Anti-sycophancy: this must be said plainly before claiming any novelty.

  - Per-level testing/judging already exists: PER-LEVEL AUDITOR MODES (B-41) ships 6
    modes Concepts/Phases/Steps/BabySteps/BabyStepDetail/Plan with auditor_mode +
    per_level_audit_dispatch. The "different test at each level" instinct = B-41.
    (Status PARTIAL: navigator audit-insertion points not fully wired.)
  - Per-module interface tests already exist: contract tests in crates/contracts/
    tests/, doctrine "run at phase entry (health check) and during audit; context
    agent generates the interface map, main model writes contract tests, auditor
    validates coverage." That is IDEA-A at module granularity.
  - The bubble-up structure already exists: the Idea->Concepts->Phases->Steps->
    BabySteps tree with DecompositionTree + TreeNavigator + BubbleUpHandler + 4-level
    integration checkpoints (phase-4E, B-7). IDEA-B's roll-up has a home.
  - "Tests before code" is structural: the FSM TESTING_FIRST state precedes EXECUTING;
    the walking-skeleton PRE_GATE forces every unit to ship producer + in-plan
    consumer + smoke in the SAME unit.
  - "Prove consistency, not only wiring" is the R10 integration-seam doctrine almost
    verbatim: wiring-enforcement.md R10 — UNIT-GREEN IS NOT INTEGRATED-GREEN; a
    multi-unit build REQUIRES an independent cross-seam integration smoke driving the
    REAL entrypoint on a HARD input, run after every 2-3 units AND before DONE. This
    is exactly IDEA-C's purpose, already a house rule.
  - The fake-user e2e already exists in spec: BROWSER E2E (E-14) = AXTree-first
    Playwright + axe-core (a synthetic user). (Status PARTIAL: DeployPreviewWorker /
    HybridDiff / VlmConstrainedClient absent; not tied to plan progression.)
  - Semantic consistency (not just wiring) already exists: INTENT-PAIR + METAMORPHIC
    (C-29) — intent_pairs<3 blocks PLANNING->TESTING_FIRST; metamorphic relations
    check behavior, not symbols.
  - Requirement<->test binding already exists: EARS REQUIREMENT OBJECTS (I-37) — typed
    requirements bound to tests + mutation + MR, per-hunk VERIFIED/UNVERIFIED.
  - Regression-as-you-go already exists: BLAST-RADIUS / ANTI-REGRESSION GATE (J-16,
    KEYSTONE) — reverse-dep cone + regression-test SELECTION, pass->fail = BLOCK;
    MUTATION TESTING GATE (C-28).
  - "Grows over time" already exists at the failure layer: LIVING EVAL CORPUS (J-56) —
    a caught failure becomes a categorized corpus row; subset-on-change, full-on-merge.

Verdict on overlap: the user reinvented ~80% of ELAI's own test/trust spine. That is
a GOOD signal (their intuition matches the design), and it means the answer is NOT
"build a new test system from scratch."


3. THE GENUINE DELTA (what is actually new or underdeveloped)
Strip out everything in section 2 and two things remain that ELAI does NOT yet have
as first-class objects:

  DELTA-1  A PLAN-BOUND TEST TREE that MIRRORS the decomposition tree. Today the
    pieces are scattered: B-41 gives per-level AUDITORS, contract-tests give per-MODULE
    interface tests, C-29/I-37 give semantic/requirement checks — but there is no single
    artifact where each decomposition NODE (this concept, this phase, this step, this
    baby-step) carries its own frozen ACCEPTANCE CONTRACT, authored top-down at
    decomposition time, and where node verdicts compose upward. The Test Tree is the
    unification: one tree, isomorphic to the plan tree, test-per-node.

  DELTA-2  A MONOTONICALLY-GROWING fake-user ACCEPTANCE SUITE keyed by plan node. R10's
    integration smoke is PERIODIC (every 2-3 units) and FINAL; E-14 is a capability not
    bound to plan progression. The user wants a cumulative suite that gains one fake-user
    scenario per shipped slice and re-runs the accumulated set, so "the whole plan still
    behaves" is proven CONTINUOUSLY and concept-level "done" is re-proven AFTER assembly,
    not inferred from children being individually green. This closes the specific gap
    "every sub-phase audited green but the assembled concept is broken" — which is the
    exact failure class R10 exists to catch but currently catches only at checkpoints.

  (IDEA-B's "re-run on every implementation" is NOT new and must NOT be taken literally —
   see constraint C-2.)


4. HOW IT HELPS ELAI (grounded payoff)
  - Attacks the integration-seam class earlier. R10 names the seam-defect problem;
    DELTA-2 turns the periodic check into a continuously-growing regression artifact, so
    a seam defect surfaces at the slice that introduced it, not at the next checkpoint.
  - Makes CONCEPT-level "done" meaningful. Today a concept is effectively "done" when its
    children audit green individually; DELTA-1's frozen concept-acceptance contract must
    re-pass after the children assemble, which is a strictly stronger done-gate and is
    the bottom-up bubble the user described.
  - Gives FRONTEND wiring a real off-agent artifact — the user's dashboard pain. A
    code-side reachability gate (production-caller-gate) proves a Rust/TS symbol has a
    non-test caller, but a UI feature's true proof of "a user can trigger it" is a
    synthetic-click trace through the real app. DELTA-2's AXTree fake-user run is that
    off-agent proof for the dashboard, complementing the code-side gate.
  - Composes cleanly (GD-1): every node verdict is just another off-agent SSB row gated
    by the F-4 cant-lie stdout hash; the Test Tree is a view over those rows, the same
    way the dashboard is a view over the journal. No parallel state.


5. HARD CONSTRAINTS (without these the idea self-games or blows the budget)
  C-1  TESTS ARE FROZEN AND AUTHORED OFF-AGENT. If the agent that implements a baby-step
       also writes that step's acceptance test, it is the C0-THESIS in-boundary failure
       (impl + test + done-claim from one party). The Test Tree's node contracts must be
       authored by the PLANNER/auditor side at decomposition time and FROZEN before the
       implementing agent starts (TESTING_FIRST + freeze-the-gate, applied per node). This
       is the single load-bearing constraint.
  C-2  SELECT, DO NOT RE-RUN-ALL. "Re-test on every implementation" taken literally fights
       J-16 (which exists precisely to SELECT the affected regression set) and the HD /
       token-economy doctrine. Rule: on a node change, re-run the blast-radius-SELECTED
       tests PLUS the ancestor-chain acceptance contracts of the touched node — never the
       whole tree. Bubble-up is scoped to the lineage, not the forest.
  C-3  DETERMINISTIC VERDICTS, NOT LLM-JUDGE. "Proves consistency" must be a deterministic
       gate verdict (test pass/fail, hash match, metamorphic relation holds), never an LLM
       saying "looks consistent" (TD-3, the LLM is advisory not the gate).
  C-4  E2E IS AXTREE-FIRST AND FLAKE-QUARANTINED. Fake-user UI tests are flaky and pixels
       are noise; per visual-ui-loop.md use AXTree assertions for facts, pixel-diff only
       against golden renders, and quarantine flakes (I-17 flake quarantine) so a flaky
       e2e never fail-opens or fail-closes the gate falsely.
  C-5  OFF-AGENT + CANT-LIE. Each Test Tree node verdict is an SSB row produced by the
       orchestrator/auditor side and carries the F-4 blake3 stdout hash; a node cannot be
       marked passed without the matching captured hash (no fabricated green nodes).


6. CONCRETE SHAPE (a sketch, not a plan)
  - TestTree: a structure isomorphic to the existing DecompositionTree. Each TestNode
    references its DecompositionNode and holds a frozen AcceptanceContract (the EARS/I-37
    requirement objects + the C-29 intent-pairs/metamorphic relations for that node's
    scope) authored top-down during PLANNING, frozen at TESTING_FIRST.
  - Execution: bottom-up. A baby-step lands -> its TestNode runs -> on green, its parent
    step's acceptance contract runs against the assembled steps -> bubble to phase ->
    concept. Each result is an off-agent SSB row (C-5). A red ancestor after green
    children = the seam-defect catch (DELTA-2 payoff).
  - AcceptanceSuite (the fake-user leg): a growing, plan-node-keyed set of E-14 AXTree
    scenarios. Shipping a UI-touching slice appends its scenario; the accumulated suite
    re-runs under C-2 selection. This is the frontend off-agent artifact.
  - Composition, not invention: TestTree drives B-41 per-level auditors; node contracts
    ARE I-37 requirement objects + C-29 pairs; regression selection IS J-16; the smoke IS
    R10 made continuous + tree-keyed; verdicts ARE SSB rows under F-4. The new code is the
    TREE OBJECT + the GROWING SUITE INDEX + the bubble-up scheduler, nothing below.


7. IS IT A NEW PLAN?
Yes, but a COMPOSING one (per GD-1 compose-not-restate, and the "new plan files only"
house rule — never amend old plan files). It would be a new track that wires together
B-41 + contract-tests + C-29 + I-37 + J-16 + E-14 + R10 into the Test-Tree artifact
(DELTA-1) and the growing fake-user AcceptanceSuite (DELTA-2). It must NOT re-author any
of those mechanisms; it consumes their symbols and verdicts. SSB-wise it likely needs at
most one new Pattern-2 action_type (a TestTreeNodeVerdict with a Kind discriminator), or
it reuses existing audit/test rows — prefer-reuse first, decide at authoring against the
LIVE schema stamp (grep ssb.rs, never a doc number).


8. OPEN QUESTIONS FOR THE USER (decide before any plan authoring)
  Q-1  Granularity of the AcceptanceContract: a contract at EVERY tree node (heavy,
       thorough) or only at concept + phase nodes (lighter, baby-steps keep the existing
       unit smoke)? Recommend concept + phase get full acceptance contracts; steps/baby-
       steps keep the existing vertical-slice smoke. Cheaper, and it puts the new weight
       exactly where the seam-defect risk lives.
  Q-2  Scope of the fake-user suite: dashboard-v2 only (Tauri/Svelte) to start, or also
       ELAI-GENERATED apps (the products ELAI builds for the user)? They need different
       harnesses; recommend dashboard-v2 first (it exists, E-14 substrate is there).
  Q-3  Re-run policy confirmation: accept C-2 (blast-radius-selected + ancestor chain) over
       literal re-run-all? This is a cost vs thoroughness call and is the main place the
       idea could become too expensive to run.
  Q-4  Does this become its own plan letter/track, or fold into the existing J-16
       blast-radius KEYSTONE + B-41 as an extension? (Recommend its own track: the Test
       Tree is a structural artifact, not a gate tweak.)
