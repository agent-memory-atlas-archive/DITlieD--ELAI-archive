Reasoning + design: propagating the ELAI DETERMINISM DOCTRINE to EVERY ELAI system

PURPOSE: the user named this "the biggest thing" -- bigger than the ContextController/
  CarryState design, which is a downstream application of it. The determinism doctrine
  (docs/strategy/ELAI-DETERMINISM-DOCTRINE.md, "the model is a cog, not a brain") today
  exists as a PASSIVE LENS: a checklist + worked examples you hand to a reviewing model.
  A passive lens is a soft gate, and a soft gate that an agent applies to its own work
  rubber-stamps -- which is the precise failure the doctrine itself warns about (D1,
  Berkeley RDI Apr-2026 in-boundary artifacts gamed ~100%). This file designs HOW the
  doctrine becomes structurally enforced at every point where an ELAI feature is
  conceived, planned, built, audited, and dispatched -- WITHOUT the propagation
  mechanism itself violating the doctrine. It is reasoning + a design spec, not a plan
  file. A later session (gated on the user + a cross-family diversity vote) turns the
  accepted parts into plan files.

GROUNDING POSTURE: every ELAI internal below is grounded in files read this session --
  docs/strategy/ELAI-DETERMINISM-DOCTRINE.md (D1-D5, the boundary, the 8-question
  reviewer checklist), .claude/rules/{wiring-enforcement,production-caller-gate,
  maintainability-review,tracing-test-assert}.md, and the five skill files the user
  pointed at (spec-eval, oracle, idea-intake, planning-workflow, orchestrate,
  auto-orchestrate). No claim here is headlined on an unmeasured number; the whole
  propagation ships observe-only and is judged on a measured doctrine-debt delta
  (section 6), which is the doctrine (D4) turned on its own rollout.

================================================================
0. THE ONE-SENTENCE THESIS
================================================================
The determinism doctrine is the GENERAL LAW of which wiring-enforcement,
  production-caller-gate, maintainability-review, tracing-test-assert, the F-4 cant-lie
  gate, and freeze-the-gate are already SPECIFIC INSTANCES -- so "propagate it to every
  system" means doing for the whole doctrine what wiring-enforcement already did for one
  sub-doctrine, and the doctrine's own D5/D3/D4/D2 tell you exactly how to enforce it
  without the enforcement itself becoming an ungated proxy.

================================================================
1. THE KEY STRUCTURAL FACT (the doctrine is already partially propagated)
================================================================
The doctrine has five sub-doctrines. Each already has at least one concrete,
  off-agent, fail-closed enforcement instance shipped or specced in this repo:

  D1 GENERATOR PROPOSES / GATE DISPOSES
     -> rules engine PROPOSED->CANDIDATE->SHADOW->ACTIVE (finding-source auditor can
        never approve its own rule); DGM archive (proposer never self-scores);
        production-caller-gate (executor not the authority on its own wiring).
  D2 EVIDENCE IS OFF-AGENT AND FROZEN
     -> F-4 cant-lie gate (blake3 stdout_hash captured by orchestrator, not auditor);
        wiring R8 + gate_freeze_check.py (gate frozen in .claude/gate-manifest.txt);
        "main model never runs tests, auditor is the sole runner".
  D3 MISSING EVIDENCE IS A HARD FAIL
     -> wiring R1/R2 (zero contracts = FAIL, dead evidence path screams);
        grounding-verification phase-2B (any ungrounded claim BLOCKS PLANNING->EXECUTING);
        tracing-test T-1/T-2 (captured-but-unasserted log = BLOCK);
        production_caller_check exit 2 fail-closed on scan-config error.
  D4 MEASURE THE OBJECTIVE NOT A PROXY
     -> wiring R4 (off-agent trace, not stdout); SAFEDELTA G-6 (gate-ON-vs-OFF
        ablation, not a leaderboard rank); DGM VD-1 (replace blake3(bytes) fitness with
        execution-grounded pass-rate).
  D5 LLM IN A GATE IS ADVISORY AND CROSS-FAMILY
     -> TD-3 (LLM is T3-advisory, mechanical legs are the gate); D-13 adversary
        (different pretraining family, same-model self-critique BANNED); D-10
        select_grounding_mode (LLM has zero input into mode selection); HD-3 (never
        constrained-decode the reasoning/coder/planner roles).

  Implication: we are NOT inventing enforcement from zero. We are GENERALIZING a pattern
  that already works in four places and making it the default for the OTHER ~200 ELAI
  features that were never reviewed through this lens. This is "activation not
  invention", the same posture the carry-state design takes -- and it lowers risk.

================================================================
2. THE SELF-REFERENTIAL TRAP (why the obvious heavyweight approach is wrong)
================================================================
The naive propagation is "build a sixth deterministic checker,
  doctrine_conformance_check.py, that scores any feature against the 8-question
  checklist, fail-closed, frozen, auditor-run -- exactly like wiring." That is APPROACH
  A, and the doctrine itself forbids most of it. Here is the trap, stated plainly:

  To enforce "the model decides as little as possible," you must decide, for every
  decision a feature makes, whether it is GENERATIVE/SELECTIVE (legitimately the
  model's) or SIZING/ADMISSION/VERDICT/COUNT/PROVENANCE/STATE (the harness's). But
  CLASSIFYING a decision into those buckets is itself a semantic judgement with no
  clean mechanical oracle. So:
   - if a deterministic checker CLAIMS to render that classification, it will have low
     recall and silently miss real violations -> it advertises coverage it does not
     have, which is the exact D3 anti-pattern the doctrine bans;
   - if an LLM renders the doctrine-conformance VERDICT on a gating path, that is an
     LLM-as-judge on a release-gating path -> a D5 violation, a fail-open gate wearing
     a gate's clothes.

  So a propagation built as "one big checker" either lies about its coverage (violates
  D3) or smuggles an LLM onto the critical path (violates D5). The doctrine cannot be
  fully mechanically enforced, and pretending otherwise is itself non-doctrinal.

  THE RESOLUTION IS THE DOCTRINE APPLIED TO ITSELF. D5 already tells you what to do
  when a judgement has no mechanical oracle: split it. A small DETERMINISTIC SUBSET
  (the structurally-detectable anti-patterns) becomes the frozen, fail-closed teeth;
  the SEMANTIC remainder (the boundary classification, "is this a proxy") becomes a
  cross-family ADVISORY review that produces findings and a registered DEBT row, never
  a sole release verdict. This is EXACTLY the shape maintainability-review.md already
  took: "THE DETERMINISTIC SUBSET (MR-1..MR-4, off-agent, frozen, fail-closed)" +
  "the qualitative catalog (the rest)" + "unknown = FAIL" + "auditor-applied, never
  executor-self-applied". maintainability-review is therefore the PRECEDENT TEMPLATE,
  and the determinism doctrine propagates as a maintainability-review-shaped rule ONE
  ALTITUDE UP -- it is the meta-rule that wiring, production-caller, maintainability,
  and tracing-assert are all instances of.

================================================================
3. THE PROPAGATION (five lifecycle injection points + a registry + a freeze)
================================================================
Map the doctrine onto the points where an ELAI feature is touched, cheapest-first.
  Prevention at design/plan time is cheaper than detection at audit time (the
  wiring-enforcement lesson). Each injection reuses an EXISTING seam in the skills the
  user pointed at; none is green-field.

  INJECTION 1 -- DESIGN TIME (spec-eval, oracle, grill, cook). Cheapest.
    spec-eval already runs an adversarial "attack-the-design" habit. ADD a mandatory
    output dimension: the DECISION LEDGER pass. For the design under evaluation,
    enumerate every decision it makes and sort each into model-class vs harness-class
    per the doctrine boundary; any harness-class decision the design hands to the model
    is a named DESIGN WEAKNESS (a legitimate mark-down in spec-eval's own grammar,
    distinct from the not-yet-built readiness note). This is the doctrine's reviewer
    checklist (questions 1-2) fired at the conceptual level, before a line is planned.
    oracle stays out of the gate (it grounds external facts); but a doctrine question
    ("is this design letting the model decide X") is a legitimate oracle/gpt-bridge
    diversity prompt and the cross-family vote (D5) lives here.

  INJECTION 2 -- PLAN TIME (idea-intake per-level auditors; planning-workflow
    plan-improvement step 4 + step 7). A DOCTRINE PRE_GATE, exactly parallel to the
    WALKING-SKELETON PRE_GATE already in planning-workflow step 4 and the OFF-AGENT
    PROOF acceptance already in step 7. Every deliverable/baby-step must declare, in
    the plan file:
      (a) DECISIONS: the list of decisions this unit makes;
      (b) for each, MODEL or HARNESS, by the boundary;
      (c) for each HARNESS-class decision, the off-agent DISPOSER that checks it (a
          checker, an SSB row, a coverage hit, a span) -- the thing the producing agent
          did not author;
      (d) for each decision that is LEGITIMATELY the model's but currently UNGATED
          because no mechanical oracle exists, a DOCTRINE-DEBT row (section 5).
    A deliverable that puts a sizing/admission/verdict/count/provenance decision on the
    model with no disposer is REFUSED at decomposition -- the same way a producer-only
    step is refused today. idea-intake's per-level plan-auditor gets a doctrine check
    added to its existing rubric (it already checks "hard D-rule violations").

  INJECTION 3 -- AUDIT TIME (auditor, AUDITING state). The auditor already runs
    production_caller_check.py, maintainability_check.py, traced_test_assert_check.py
    off-agent over the changeset. ADD the thin deterministic doctrine floor (section 4)
    as a fourth frozen checker, plus a cross-family ADVISORY doctrine review that emits
    findings tagged D-DETERMINISM-* (parallel to the C-MAINTAINABILITY-* tags). The
    deterministic floor is the teeth (fail-closed); the advisory review is the
    qualitative catalog (unknown = FAIL, re-read don't wave through). The auditor, not
    the executor, renders it -- never a self-pass.

  INJECTION 4 -- DISPATCH TIME (orchestrate, auto-orchestrate, agent-dispatch-preamble).
    The preamble already binds wiring + production-caller on every worker. ADD the
    doctrine pointer so every worker is doctrine-bound, and extend the R10 integration
    gate: a cross-seam smoke must confirm that a harness-class decision a worker
    implemented is actually disposed by the harness, not silently by the model at the
    seam (the carry-state ADMISSION check is a concrete example of a seam that must be
    gated, not assumed).

  INJECTION 5 -- FREEZE (gate-manifest.txt). The doctrine floor checker AND the
    doctrine rule file are added to .claude/gate-manifest.txt so a run cannot weaken
    them (D2 / wiring R8). A run that edits its own doctrine gate is BLOCK.

================================================================
4. THE THIN DETERMINISTIC FLOOR (high-confidence structural anti-patterns only)
================================================================
The floor is deliberately LOW-RECALL / ZERO-FALSE-POSITIVE: it only flags the few
  doctrine violations that have an unambiguous structural signature, so it never lies
  about coverage (D3-honest) and never blocks a legitimate pattern. Everything it
  cannot prove is left to the advisory review, which is honest about being advisory.
  Candidate checks (regex/AST, necessary-not-sufficient, the load-bearing leg is the
  advisory review + the existing instance-checkers):
    DD-1  GATE-READS-OWN-OUTPUT: a gate/checker/verdict function whose evidence source
          is the same agent's stdout/log/claim rather than an off-agent artifact (the
          F-4 inverse). Signature: a pass/fail verdict computed from a string the
          producing path emitted. -> BLOCK candidate.
    DD-2  SKIP-ON-MISSING-EVIDENCE: a gate that returns SKIP/UNVERIFIED/WARN (instead
          of FAIL) when its evidence path is absent (the D3 inverse). Signature: a
          missing-file / empty-result branch that does not fail-closed. -> BLOCK
          candidate.
    DD-3  SAME-FAMILY-CRITIC: an adversary/verifier/judge dispatched on the SAME
          pretraining family as the producer (the D5 inverse). Signature: a critic
          dispatch whose model arg equals the producer's. -> BLOCK candidate.
    DD-4  LLM-VERDICT-ON-GATING-PATH: an LLM call whose return is consumed as a
          release-gating boolean with no mechanical leg behind it (the D5 inverse).
          Signature: an FSM/CI gate predicate that is purely an LLM judgement. -> WARN
          (hard to prove mechanically; advisory review confirms).
    DD-5  CONSTRAINED-REASONING: a grammar/force-JSON/constrained-decode applied to a
          reasoning/coder/planner role rather than pure op-emission (the HD-3 inverse).
          -> WARN.
  The floor is frozen (section 3 injection 5), auditor-run (never executor-self-run),
  exit-2 fail-closed on a scan-config error, and ships observe-only (WARN-only) until
  its false-positive rate is measured (section 6). Thresholds + the rule file are
  manifest-frozen so a plan run cannot weaken them.

================================================================
5. THE DOCTRINE-DEBT LEDGER (the wire-dark-allowlist analog -- "no graveyard")
================================================================
Not every doctrine gap can be closed today: sometimes the model legitimately makes a
  decision that SHOULD be mechanical but has no oracle yet (e.g. DGM fitness was
  blake3(bytes) until H-43; carry-state quality has no deterministic oracle so an LLM
  judge is advisory by necessity). The doctrine handles this the way wiring R9 handles
  a default-OFF flag: register it, don't bury it. A DOCTRINE-DEBT row carries:
    decision_id, feature, sub_doctrine (D1..D5), why_unmechanized (the missing oracle),
    current_disposer (advisory-cross-family | none | manual), owner, expiry_date,
    promotion_plan_id (the plan that will close it).
  A debt row with no promotion plan is itself a finding (the "no graveyard" rule).
  This makes every KNOWN violation visible and time-boxed, and it is the source of the
  measured metric. The ledger lives beside wire-dark-allowlist.toml, same posture.

================================================================
6. MEASURE, DON'T ASSERT (the doctrine, D4, turned on its own propagation)
================================================================
The propagation MUST NOT be headlined as "the doctrine is now enforced" on faith --
  that would be the exact proxy-for-objective failure D4 names. Instead:
  - Instrument a DOCTRINE-DEBT RATE = (open debt rows / total reviewed decisions),
    tracked over time, per subsystem. The propagation is judged on the measured DELTA,
    not on the existence of a checker.
  - Ship the floor + ledger + advisory review OBSERVE-ONLY (WARN, never BLOCK) first.
    Run it across the existing ~213 B..K plans + live code to establish the baseline
    debt rate and the floor's false-positive rate. ONLY THEN promote each DD-* finding
    class to fail-closed BLOCK, per-class, once its FP rate is known. This is the same
    observe-only-then-promote rollout the carry-state falsifier uses and every ELAI
    gate uses -- the doctrine applied to its own thesis (the carry-state doc's P5).
  - A subsystem's doctrine-conformance stamp goes into ELAI-FEATURE-MAP.md (the living
    source of truth, which already carries the Section 3 doctrine blocks W/GD/HD/TD/VD/
    RV/CL/ID/JD). Each feature gets clean | debt(row-id) | unreviewed, upserted on the
    same FEATURE-MAP UPSERT seam planning-workflow/orchestrate already run. unreviewed
    is treated as debt (D3: absence of a review is not a pass).

================================================================
7. RELATIONSHIP TO THE CONTEXTCONTROLLER / CARRYSTATE DESIGN (two altitudes, one program)
================================================================
The carry-state design is the FLAGSHIP FIRST APPLICATION of the propagated doctrine,
  not a separate thing. Its section 5 already maps cleanly onto the D-boundary:
  D-CONTROLLER-1 (WHEN to checkpoint) = harness SIZING; D-CONTROLLER-2 (which operator /
  reset-vs-curate) = harness STATE; D-CONTROLLER-3 (ADMISSION of the proposed carry) =
  harness ADMISSION via span-containment + provenance, fail-closed (D3); D-CONTROLLER-4
  (emit CarryStateCheckpoint SSB row) = off-agent frozen EVIDENCE (D2); and the ONLY
  model-owned act is GENERATION of the proposed carry text (D-boundary GENERATION, HD-3
  clean -- never constrained-decode it). The carry-state ADMISSION gate is literally a
  D1/D2/D3 instance: generator proposes the carry, a mechanical span/provenance check
  disposes it, missing provenance = reject. So:
   - the DOCTRINE PROPAGATION (this file) is the program that makes EVERY feature pass
     that bar;
   - the CARRYSTATE CONTROLLER (the sibling design) is the first feature built to pass
     it from day one, and its observe-only falsifier IS the doctrine's measure-don't-
     assert rollout (section 6) instantiated for one feature.
  Building carry-state through the doctrine PRE_GATE (injection 2) is the proof that
  the PRE_GATE works on a real, hard, novel feature. They ship together: the doctrine
  PRE_GATE first (observe-only), then carry-state as its first graduate.

  And the framing the user gave -- "almost as important as SSB" -- is exact and
  doctrine-grounded: SSB makes what the model CLAIMS un-fabricable (the EVIDENCE
  substrate, D2); CarryState makes what the model REMEMBERS un-lose-able (the STATE
  substrate); the DETERMINISM DOCTRINE is the law that says BOTH must exist because the
  model is a cog. SSB and CarryState are two pillars; the doctrine is the foundation
  they both stand on. Propagating the doctrine is therefore strictly more fundamental
  than either pillar -- which is why the user ranked it first.

================================================================
8. TWO APPROACHES CONSIDERED (house rule: two minimum before a recommendation)
================================================================
  APPROACH A -- HEAVYWEIGHT SIXTH RULE. A full doctrine_conformance_check.py claiming
    broad coverage of the 8-question checklist, fail-closed BLOCK, parallel to wiring in
    weight. REJECTED as the primary: its semantic core (boundary classification, proxy
    detection) is only weakly mechanizable, so it would either advertise coverage it
    lacks (violates D3) or route an LLM verdict onto the gating path (violates D5). It
    also invites doctrine-theater: a checklist filled to pass, not to think (Goodhart on
    the ledger itself, the D4 failure). High authoring cost for low honest coverage.
  APPROACH B -- LENS + THIN FLOOR + DEBT LEDGER (RECOMMENDED, hybrid leaning B). The
    doctrine stays primarily the DESIGN+AUDIT LENS (cross-family advisory, semantic),
    backed by a thin frozen deterministic FLOOR (DD-1..DD-5, high-confidence structural
    anti-patterns only, zero-FP target), a fail-closed DOCTRINE-DEBT LEDGER that makes
    every known gap visible with a promotion plan, a measured doctrine-debt-rate metric,
    a feature-map conformance stamp, and a freeze. Honest about residual risk (no
    pretense of full mechanical coverage), doctrine-consistent (it is the doctrine
    applied to itself), and low marginal cost (reuses existing seams in five skills +
    the auditor + the preamble + the manifest + the feature-map upsert).
  RECOMMENDATION: APPROACH B. The decisive argument is self-consistency: a propagation
    mechanism that violated D3/D5 to enforce D1-D5 would be self-refuting. B is the only
    form that propagates the doctrine without breaking it.

================================================================
9. RESIDUAL RISK (do not oversell -- mirrors wiring-enforcement's own honesty section)
================================================================
  - The semantic core is unmechanized by design; the floor catches only the structural
    tells. A subtle boundary violation (a proxy dressed as an objective, a model
    judgement laundered through a mechanical-looking wrapper) can pass the floor and a
    rushed advisory review. Mitigation: cross-family advisory (D5), measured debt rate
    (catch drift in aggregate), and the freeze (stop a run weakening the gate).
  - Doctrine-theater: the DECISION LEDGER could be filled mechanically to pass. The
    debt-rate metric is the backstop -- a subsystem whose ledger says "all clean" but
    whose debt rate is implausibly zero is itself suspect (the floor's self-test, D3:
    a dead evidence path must scream, not shrug).
  - Cross-family availability: gpt-bridge / a second family is rate-limit-fragile and
    occasional; the advisory review degrades to single-family when the second family is
    unreachable, which weakens D5. Record that as a debt row on the review itself, do
    not silently treat single-family advice as cross-family.
  - The ~213 existing plans were mostly authored BEFORE the doctrine was written
    (2026-06-16). The baseline debt sweep (section 6) will likely surface a large
    backlog. That backlog is a SEQUENCING fact (it gets a promotion plan), not a reason
    to gate retroactively and stall everything -- observe-only first, exactly so the
    baseline can be taken without blocking.

================================================================
10. CANDIDATE PLAN WAVE FOR A LATER SESSION (gated on user + cross-family vote)
================================================================
  K-? DOCTRINE-PROPAGATION KEYSTONE -- author .claude/rules/determinism-doctrine.md as
    a HARD RULE (the meta-rule, maintainability-review-shaped); the thin frozen
    deterministic floor .claude/tools/doctrine_floor_check.py (DD-1..DD-5, observe-only);
    the DOCTRINE-DEBT LEDGER (beside wire-dark-allowlist.toml); add both to
    .claude/gate-manifest.txt. Grounding: ELAI-DETERMINISM-DOCTRINE.md, maintainability-
    review.md (template), wiring-enforcement R8/R9.
  K-? DESIGN-TIME INJECTION -- add the DECISION LEDGER pass to spec-eval as a required
    output dimension; add the doctrine question to the oracle/gpt-bridge diversity-vote
    path.
  K-? PLAN-TIME INJECTION -- add the DOCTRINE PRE_GATE to planning-workflow step 4/7 and
    to idea-intake's per-level plan-auditor rubric (parallel to the WALKING-SKELETON
    PRE_GATE + OFF-AGENT PROOF acceptance).
  K-? AUDIT-TIME INJECTION -- wire doctrine_floor_check.py into the auditor's off-agent
    checker set; add the D-DETERMINISM-* finding tags + the cross-family advisory review.
  K-? DISPATCH-TIME INJECTION -- add the doctrine pointer to agent-dispatch-preamble.md;
    extend the R10 integration gate with the seam-disposal check.
  K-? MEASURE -- the doctrine-debt-rate metric + the baseline sweep across B..K + the
    ELAI-FEATURE-MAP conformance stamp; promote DD-* classes to BLOCK per-class only
    after the FP rate is measured.
  K-? (sibling, already designed) CONTEXTCONTROLLER + CARRYSTATE as the flagship first
    graduate of the doctrine PRE_GATE (context-controller-carry-state-design-2026-06-16.md
    section 8).

================================================================
11. OPEN QUESTIONS FOR THE PLAN-AUTHORING SESSION
================================================================
  - Floor scope: are DD-1..DD-5 the right five, or is there a sixth high-confidence
    structural tell worth adding (e.g. an executor self-running its own gate -- the
    production-caller-gate already has a leg for this; avoid duplicating)?
  - Where does the DECISION LEDGER physically live for a plan deliverable -- inline in
    the phase/baby-step file (more friction, more visible) or a sibling doctrine-ledger
    file (less friction, easier to ignore)? Lean: inline, like PRE_GATE blocks, so it
    cannot drift from the deliverable it judges.
  - Retroactive sweep depth: full B..K (expensive, complete baseline) vs only-WIRED
    features (cheaper, judges what actually runs)? Lean: only-WIRED first for the
    baseline, full sweep as a background debt-burn-down.
  - Cross-family vote BEFORE plan-authoring: per D5 and the carry-state doc's open
    questions, run a GPT/Grok diversity pass on THIS propagation design (is the
    self-referential resolution sound, is the floor zero-FP claim realistic) before
    committing any plan file. The user wanted to review first; this vote is the next
    gate after the user's go.

================================================================
12. BOTTOM LINE
================================================================
The doctrine is already enforced in four places; propagating it to every system means
  generalizing that pattern, and the doctrine's own D3/D5/D4 dictate the shape: a thin
  frozen deterministic FLOOR for the few structural anti-patterns, a cross-family
  ADVISORY semantic review for the boundary-classification core that has no mechanical
  oracle, a fail-closed DOCTRINE-DEBT LEDGER (no graveyard) for known gaps, a measured
  doctrine-debt-rate metric (judge the propagation on a delta, not on faith), a
  feature-map conformance stamp, and a freeze. It injects at five existing lifecycle
  seams (design via spec-eval, plan via planning-workflow + idea-intake, audit via the
  auditor, dispatch via the preamble, freeze via the manifest), each reusing machinery
  that already exists. The heavyweight "one big checker" approach is rejected because it
  would have to violate D3 or D5 to enforce D1-D5 -- self-refuting. The ContextController
  / CarryState design is the same program one altitude down: the doctrine is the law,
  SSB and CarryState are the two substrates it mandates (evidence + state), and
  carry-state is the flagship first feature built to pass the doctrine PRE_GATE from day
  one. Ships observe-only, baseline-measured, promoted per-finding-class -- the
  determinism doctrine applied to its own propagation.

================================================================
ADDENDUM 2026-06-16b -- THE AUX REFRAME (method vs lens; what sections 1-12 missed)
================================================================
The user pointed at docs/strategy/ELAI-DETERMINISM-DOCTRINE-AUX.md after the above was
  written. The AUX is explicit: the determinism doctrine (DOCTRINE.md) is the WHAT (a
  lens); the AUX is the HOW (the reasoning METHOD, M1-M12 + the stance question, that
  PRODUCED the lens). "The determinism doctrine is ONE output of this method applied to
  ONE axis (decomposition + grounding)." And the warning, verbatim: "If you only
  pattern-match 'determinism' you have missed the point -- the point is a way of
  reasoning about any ELAI improvement." Sections 1-12 above propagated the LENS and
  under-weighted the METHOD. The corrected design has TWO propagation tracks at two
  altitudes, plus a generative third.

A1. TWO THINGS TO PROPAGATE (they are different KINDS of thing, enforced differently).
  TRACK 1 -- THE LENS (DOCTRINE.md, "model is a cog"). This is a checkable PROPERTY OF A
    FEATURE: a feature either does/doesn't hand a harness-class decision to the model.
    Propagated by GATES -- exactly the sections 1-12 design (thin frozen floor DD-1..DD-5
    + cross-family advisory + fail-closed DOCTRINE-DEBT LEDGER + 5 lifecycle injections +
    freeze + measured debt-rate). That design stands; it is the WHAT-the-feature-must-
    satisfy track.
  TRACK 2 -- THE METHOD (AUX.md, M1-M12 + the stance). This is a property of the
    REASONING that PRODUCES a feature/plan, not of the feature. You CANNOT regex the
    QUALITY of reasoning. But the ARTIFACT SHAPE the method mandates IS deterministically
    presence-checkable, and that is the lever:
      M1 grounding-before-claim   -> every external claim carries a source or a tag.
      M2 confidence calibration   -> every claim tagged VERIFIED / high / training-unsearched.
      the stance question         -> a DECISION CLASSIFICATION (what the model decides
                                     here, what the harness could decide instead) present.
      M4/M6 net-new-vs-existing   -> a "reinvention | extension | net-new" + named
                                     file:line seam present.
      M7 attack-your-own-design   -> an ATTACK section present (strongest case against).
      M9 falsifier-as-step-1      -> the cheapest falsifying experiment is literally
                                     step 1; nothing default-ON before its FP/over-block
                                     rate is bounded on >=3 runs.
      M10 honest residue          -> a RESIDUE section (unverified / provisional / deferred).
      M12 write-it-down           -> the reasoning chain + durable lens + handoff exist.
    So TRACK 2 propagates as (a) a required PROTOCOL baked into the reasoning/design
    surfaces -- spec-eval, oracle, deep-research, idea-intake's framer+critic loop,
    planning-workflow plan-improvement, grill, the GRC ledger -- and (b) a deterministic
    ARTIFACT-SHAPE GATE that checks the mandated sections EXIST and the claims are TAGGED
    (missing attack section = FAIL, untagged load-bearing claim = FAIL: this is D3 applied
    to the reasoning artifact), with the QUALITY of those sections left to cross-family
    advisory. Same split as the lens, one altitude up: mechanize the structure, advise
    the substance, fail-closed on missing structure, falsify before trust.
  Note the self-consistency again: TRACK 2 is propagated USING the method's own M9
    (falsifier/observe-only) and the lens's own D3 (missing = FAIL). The method is
    enforced by the method. planning-workflow's plan-improvement protocol (PRE_GATE +
    ADVERSARIAL injection + GROUNDING section + ACCEPTANCE) is ALREADY a partial TRACK-2
    instance for plan files -- the work is generalizing it to ALL reasoning artifacts and
    adding the explicit M2/stance/M7/M10 presence checks it lacks.

A2. THE GENERATIVE THIRD TRACK -- A FAMILY OF PER-AXIS DOCTRINES (the AUX's actual ask).
  Because the LENS is one output of the METHOD applied to ONE axis, the highest-leverage
  propagation is not just "enforce the existing doctrine everywhere" -- it is "RUN THE
  METHOD on every axis to PRODUCE the axis-specific doctrine," deliberately, the way the
  determinism doctrine was produced for decomposition+grounding. The AUX already sketches
  the stance answer per axis (research / planning / reasoning-GRC / memory / security /
  routing / eval / UX). Each sketch, run through M1->M3/M4->M5->M7->M8->M9->M10->M12,
  yields a durable per-axis doctrine doc in docs/strategy/ + its falsifier-first plan.
  The determinism doctrine becomes the first member of a doctrine FAMILY, not a singleton.
  This is what "propagate to EVERY ELAI system" means at full strength: not one lens
  stamped on 200 features, but the METHOD applied per axis to yield the right lens for
  each, all sharing the model-as-cog spine but specialised (research's "what is enough"
  gate, memory's staleness verdict + objectivity firewall, routing's tier decision, etc).

A3. CONSEQUENCE FOR THE PLAN WAVE (section 10 amended).
  The keystone is now TWO rules, not one: .claude/rules/determinism-doctrine.md (TRACK 1,
  the feature-property meta-rule) AND .claude/rules/reasoning-method.md (TRACK 2, the
  artifact-shape + protocol meta-rule, generalising plan-improvement). The deterministic
  floor splits accordingly: doctrine_floor_check.py (DD-1..DD-5 over CODE) + a
  reasoning_artifact_shape_check.py (sections-present + claims-tagged over REASONING/PLAN
  DOCS). The generative third track is a sequence of per-axis doctrine-authoring sessions
  (each its own small plan), starting from the AUX's existing axis sketches. The
  cross-family diversity vote (D5 / M8) on this whole reframe is the immediate next gate,
  before any plan file -- which is why the next artifact is a research prompt, not a plan.

A4. RESIDUE ON THIS ADDENDUM (M10, honestly).
  - reasoning_artifact_shape_check.py is a presence/structure check; a section can be
    PRESENT and EMPTY-of-substance (attack-theater). The debt-rate backstop + cross-family
    advisory are the only guards; honest about this being weaker than the code floor.
  - The per-axis doctrine family (A2) is open-ended scope; it must be sequenced by
    leverage (M-style), not authored all at once. Which axis first is itself an open
    question routed below.
  - Whether enforcing artifact SHAPE measurably improves reasoning QUALITY (vs producing
    compliance-theater) is training-uncertain -- routed to the research prompt (M8), not
    asserted here.
  - I have NOT read workflow/reasoning/decomposition-grounding-reasoning-2026-06-16.md
    (the AUX's named canonical worked example) or the J-39/40/41/42 plan files this
    session; the AUX's summary of them is my source. Flagging that as a grounding gap to
    close before plan-authoring.

================================================================
ADDENDUM 2026-06-16c -- POST-RESEARCH RE-REASONING (the deep-research vote, folded in)
================================================================
SOURCE: a deep-research report (anonymized prompt, 2021-2026 tiered sources) answering
  the six open questions section A3/A4 routed to grounding (M8). It settled the forks the
  design rested on. Below: the verdict, the corrections the evidence FORCED, and the
  honest residue (M10). The architecture survives; three specific claims were overstated
  and are corrected; one scope risk is now the dominant concern.

V0. HEADLINE VERDICT (confidence upgraded reasoned -> VERIFIED).
  Approach B (thin deterministic floor + cross-family ADVISORY review + fail-closed
  EXCEPTION REGISTRY + measured violation-rate + observe-only rollout) is a RECOGNIZED
  meta-governance pattern, NOT a novel bet at the component level. It is the documented
  shape of architecture fitness functions (evolutionary architecture), policy-as-code
  with suppression/baseline files (OPA/Rego/Conftest), supply-chain attestation
  (SLSA/in-toto), tuned static analysis with baselines (Semgrep/CodeQL), and ISO/PCAOB/
  DO-178C exception-registry + measured-non-conformance governance. The self-referential
  resolution (section 2) is sound and has precedent ("who audits the auditor" =
  separation of measurer from measured). The report's own words: this is "evidence-
  aligned and pragmatic -- stronger than pure automation for inherently semantic
  doctrine." The rejected Approach A (one comprehensive scorer) is confirmed brittle
  precisely because the scorer's own classification is subjective -> high false-negatives
  or an LLM-on-the-gate. So the central architectural choice STANDS, now grounded.

V1. CORRECTION C1 -- DROP "ZERO-FP FLOOR"; IT IS LOW-FP, AND RUST IS THE WEAK LEG.
  Section 4's "zero-FP target" was overstated. Evidence: structural-anti-pattern / test-
  smell static detection runs 17-53% FP on auto-generated code, ~10-20% on manual after
  tuning, and tuned Semgrep SECURITY rules reach <5% FP -- low, never zero. The four DD-*
  patterns are pattern-matchable at single-digit FP with tuning + baselines, but the
  report flags a specific gap: "Rust full dataflow" detection is weaker than for JS/
  Python/Java/Go/C#. Since this codebase is Rust-heavy, the floor must lean on AST +
  syntactic pattern + baseline-files for Rust, NOT full dataflow, and accept that its
  Rust recall/precision is the weakest leg. CONSEQUENCE: the floor stays WARN / observe-
  only (never silently authoritative), DD-4 (LLM-verdict-on-gate) stays WARN as already
  specced (the report found "no LLM-specific benchmark" for it), and the observe-only-
  first rollout is now MANDATORY-by-evidence, not a nicety.

V2. CORRECTION C2 -- THE METHOD-TRACK AND THE LEDGER ARE BOTH THEATER WITHOUT A RANDOM-
  SAMPLE DEEP AUDIT OF SUBSTANCE. This is the single most important fold-in.
  The artifact-shape gate (track 2) checks SECTIONS EXIST. The evidence (Q3): mandated
  shapes (ADRs, pre-mortems, WHO/Pronovost checklists) improve outcomes ONLY when
  audited / random-sampled, and -- critically -- partial compliance gives a NULL effect
  (WHO checklist meta-analyses: full compliance 18-47% complication drop, partial = no
  effect). Boilerplate-filled sections ("empty attack paragraph, generic falsifier") are
  the DOMINANT outcome of shape mandates without deep audits.
  The debt ledger (section 5): Goodhart is ubiquitous and an exception ledger + tracked
  rate AMPLIFIES surrogation (people fill rows minimally and proliferate registered
  exceptions without remediation).
  THE FIX (evidence-backed, Q3+Q4): a RANDOM-SAMPLE DEEP AUDIT of artifact SUBSTANCE and
  of ledger rows, performed by an auditor that is NOT the author (separate measurer from
  measured), tracking REMEDIATION VELOCITY not just exception count, with hold-out
  un-gamed subsets. Without this leg, both track 2 and the ledger are decoration. This
  promotes "random-sample substance audit" from an implicit hope to a NAMED, REQUIRED
  component of the design. And "partial = null" means the method must be all-or-nothing
  WITHIN each sampled subset -- you cannot half-enforce it and expect half the benefit.

V3. CORRECTION C3 -- THE CROSS-FAMILY ADVISORY IS DECORATION UNLESS CALIBRATED + GIVEN
  ESCALATION TEETH. Evidence (Q5): same-family LLM judges show 10-25% self-preference
  inflation (cross-family confirmed necessary -> D5 vindicated), BUT advisory-only review
  is routinely IGNORED unless (a) calibrated against 100-300 human-labeled traces to
  Cohen kappa > 0.6 (uncalibrated = "decoration," kappa < 0.5 = unusable) and (b)
  integrated with HUMAN ESCALATION for flagged cases + explicit weighting/visibility, not
  just a dashboard. CONSEQUENCE: the advisory leg carries a real, previously-unbudgeted
  cost -- a calibration corpus of 100-300 human-labeled doctrine-conformance examples --
  and until that corpus exists the advisory leg is ADMITTEDLY decorative and must be
  labelled so (it does not count as the D5 cross-family check until calibrated). Flagged
  findings route to the orchestrator/user as a decision, never die in a dashboard.

V4. SCOPE DISCIPLINE -- THE PER-AXIS DOCTRINE FAMILY (A2) IS THE DOMINANT RISK NOW.
  The report's strongest attack lands here: proliferating per-domain doctrines spawn
  "process debt" that slows velocity more than it helps, and heavy process governance
  EMPIRICALLY HURTS in high-velocity/uncertain domains when culture is absent (CMMI-vs-
  agile studies). So A2 must be disciplined HARD: sequence axes by MEASURED pain (an
  observed, recorded failure on that axis), CAP the live doctrine count, and author a new
  axis-doctrine ONLY when a real failure justifies it -- never speculatively "for
  completeness." The determinism doctrine earned its place via a real decomposition/
  grounding failure; every sibling must clear the same bar. Better to have two
  battle-tested doctrines than eight ceremonial ones.

V5. FALSIFIER-AS-STEP-1 (M9) -- ADOPT THE REPORT'S A/B/C EXPERIMENT VERBATIM.
  The cheapest experiment that falsifies the CENTRAL claim (this hybrid beats both doing
  nothing and a comprehensive scorer, without devolving into theater): 20-50 parallel
  small-scope agent runs split A (hybrid: shape gates + advisory + registry + observe-
  only baseline) / B (nothing: pure model discretion) / C (simulated comprehensive
  scorer: LLM classification on the gating path). Run an observe-only baseline window,
  then flip to blocking where possible; measure % of shipped features with post-hoc
  HUMAN-AUDITED "unmechanized-discretion leaks" + bug density + engineer time-to-ship.
  FALSIFIED if hybrid does NOT show statistically lower leaks than B AND lower bureaucracy
  overhead than C (or only-equal quality). This becomes step 1 of the whole propagation
  program; nothing goes blocking-default-ON before it passes. Cost: one harness fork + a
  human audit panel, days of effort.

V6. HONEST RESIDUE (M10).
  - The central claim remains an UNPROVEN BET: no direct precedent exists in the
    autonomous-agent-harness domain (the components are proven elsewhere; their
    composition HERE is not). Success "hinges on execution: audits, culture, tight
    integration" -- the report is explicit. We are not buying a guaranteed win; we are
    buying a well-precedented architecture whose domain-transfer must be measured (V5).
  - Three claims the report could NOT verify directly: exact FP for LLM-verdict-gate
    detection (DD-4, analogous patterns only); a large RCT on artifact-shape-alone vs
    downstream code quality; a direct A/B of hybrid-vs-scorer in agent harnesses. So
    DD-4's FP, track-2's quality lift, and the hybrid's edge are all instrument-and-
    measure items, not assertions.
  - The calibration corpus (V3, 100-300 labeled traces) and the random-sample deep-audit
    panel (V2) are real recurring HUMAN costs the design now explicitly carries; they are
    the price of the advisory + ledger legs not being theater.
  - Rust static detection (V1) is the weakest technical leg; budget for higher FP there.
  - Still un-read this session: the AUX's named worked example + J-39/40/41/42 (A4 gap,
    unchanged).

V7. NET EFFECT ON THE PLAN WAVE (sections 10 + A3 amended).
  The two keystone rules + two deterministic checkers stand. ADD, as first-class
  components (not afterthoughts): (i) the random-sample SUBSTANCE-AUDIT panel for track 2
  + the ledger (V2); (ii) the advisory CALIBRATION-CORPUS build + kappa>0.6 gate +
  escalation routing (V3); (iii) the A/B/C falsifier as program step 1 (V5). CONSTRAIN
  the per-axis doctrine family to pain-justified, capped sequencing (V4). Re-label the
  floor LOW-FP-observe-only, Rust-weak (V1). Everything else from sections 1-12 + the
  addenda holds. The design is now grounded enough to author plan files -- gated only on
  the user's go and closing the A4 read-gap (the worked example + J-3x plans).

================================================================
ADDENDUM 2026-06-16d -- A4 READ-GAP CLOSED (grounding corrections from the DD wave)
================================================================
Read this session: workflow/reasoning/decomposition-grounding-reasoning-2026-06-16.md
  (the AUX's canonical worked example, the method run end-to-end) + the four J-plan files
  J-39/40/41/42. Five corrections/confirmations that set the plan-authoring baseline:

G1. THE PER-AXIS DOCTRINE FAMILY (A2) IS PARTLY ALREADY BUILT -- and that narrows the
  net-new. J-39 (decomposition), J-40 (research), J-41 (grounding), J-42 (reasoning) ARE
  the "DD wave": each is the determinism doctrine applied to one feature-axis, and each
  ends with the line "ELAI-DETERMINISM-DOCTRINE: the canonical <X> application." So four
  axes are already planned. CONSEQUENCE: the propagation wave's generative third track
  (A2) must NOT re-author these. It targets (i) the META axis the DD wave does NOT cover
  -- the doctrine-conformance review of EVERY feature (track 1) + the reasoning-METHOD
  enforcement (track 2) -- which is the actual net-new of this whole program; and (ii)
  the REMAINING feature-axes (memory, routing, eval, UX, security), authored ONLY on
  measured pain per V4, never speculatively. The DD wave is the precedent that the
  doctrine propagates cleanly as J-format plan files; the propagation wave sits one
  altitude above it (it governs how features like the DD wave get reviewed and how the
  reasoning that produces them is disciplined).

G2. THE J-FORMAT IS THE TEMPLATE (use J-39 as the exemplar). Every new file: a header
  line (Track | lever | grounding | nature | effort | WAVE | depends); a one-paragraph
  problem/thesis stating the cog invariant; ## Steal/build with the FALSIFIER EXP as
  numbered OBSERVE-ONLY step 1 + an EXCLUDED list; ## Existing ground truth with
  grep-confirmed file:line seams (re-grep at edit, W-7); ## Integration with an honest
  residue line; ## WIRING_CONTRACT (produces_symbols / consumes_symbols / reachable_from
  / stub_ban / observable_effect = an OFF-AGENT artifact / SSB); ## Grounding (each cite
  VERIFIED or PROVISIONAL, arXiv ids WebSearch-verified before load-bearing); ##
  COMPOSES-WITH; ## Open questions / risks; ## Next. The wave also gets a _grounding map
  with live grep evidence (cf. J-plan/_grounding/J-HARNESS-GROUNDING-MAP.md).

G3. LIVE SSB BASELINE IS SCHEMA 268 / TV 577 (per J-39/40/42 authoring), NOT the 234/543
  in stale memory. Any new action type bumps baseline+1 COMPUTED AT IMPLEMENTATION, never
  hard-coded, sequential after any sibling that ships first (W-9). BUT the propagation
  wave likely needs FEW/NO new SSB types: prefer reusing existing telemetry (J-40 reused
  QueryTrace, J-41 reused CitationAnchorsBound, net 0 new types each). A DoctrineViolation
  / DoctrineDebtRegistered row is justified only if violation telemetry is wanted as a
  first-class queryable stream (Pattern-2, grain-discriminated) -- decide at authoring.

G4. TWO FALSIFIER ALTITUDES. The DD wave's EXP-1/2/3 are PER-FILE observe-only falsifiers
  (does THIS gate have signal). The program-level A/B/C experiment (V5) sits ABOVE them:
  does the PROPAGATION MECHANISM (shape gates + advisory + registry) beat both nothing
  and a comprehensive scorer. Both altitudes are observe-only-step-1; neither is skipped
  to default-ON. The worked example confirms this is the house pattern (EXP-1/EXP-2 are
  literally step 1 of J-39/J-41).

G5. CONFIRMED CONSISTENCY: the worked example's unifying principle ("push every
  content-bearing and sizing decision out of model discretion into deterministic harness
  machinery; leave the model only the irreducibly generative/selection acts, and gate
  even those against frozen, measured criteria") is verbatim the doctrine spine, and the
  AUX method (M1-M12) is the reasoning that produced it. The two-track + family design is
  the same principle turned reflexively on the act of producing features -- no
  contradiction surfaced on the read. Net: baseline is sound; proceed to author when the
  user greenlights. Next free J-number: J-43 (highest existing is J-42); wave name
  DOCTRINE-PROPAGATION (DP).
