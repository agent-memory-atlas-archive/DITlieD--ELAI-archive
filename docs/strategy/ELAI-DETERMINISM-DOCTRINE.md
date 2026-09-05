ELAI DETERMINISM DOCTRINE — "the model is a cog, not a brain"
Purpose: a portable lens another LLM can apply to ANY ELAI feature to judge
  whether it respects the core design principle. Hand this file to a reviewing
  model along with the feature under review and the ELAI-FEATURE-MAP. It is a
  checklist + worked examples, not a spec. Plain text, ELAI house style.
Companion: ELAI-DETERMINISM-DOCTRINE-AUX.md is the METHOD (how to reason about an
  ELAI improvement on ANY axis -- research, planning, reasoning, memory, security,
  routing -- the same way this lens was derived). This file is the WHAT; the AUX is
  the HOW. Read both: a reviewer applies this lens; an improver runs the AUX method.
Written 2026-06-16. Grounded against docs/specs/guide/steps/phase-index-MASTER.txt,
  .claude/rules/{wiring-enforcement,production-caller-gate,maintainability-review,
  tracing-test-assert}.md, and docs/thefinalglance/ELAI-FEATURE-MAP.md.

================================================================================
THE PRINCIPLE (one sentence)
================================================================================
The model decides as little as possible, and nothing the model decides is
  trusted until a deterministic, model-independent mechanism has checked it.

Corollary: every place ELAI relies on the model's judgment for something that
  COULD be made mechanical (sizing, admission, a pass/fail verdict, a count, a
  reachability claim, a "this is grounded" assertion) is a latent defect, even
  if it currently works. "It works because the model happened to get it right"
  is not a guarantee; it is an ungated proxy.

================================================================================
THE BOUNDARY — what the model is ALLOWED to decide vs what the harness owns
================================================================================
The model is legitimately the engine for acts that are irreducibly generative
  or selective and have no deterministic oracle:
  - GENERATION: write the code, the prose, the candidate plan, the hypothesis.
  - PROPOSAL: propose a decomposition, a rule, a config change, a fix.
  - SELECTION / RANKING: pick which of N retrieved pages/candidates is relevant,
    rank search results, choose a template.
  - SEMANTIC JUDGMENT with no mechanical test: "is this design tasteful", "does
    this answer the user's intent" — but see TD-3 below: even here the model is
    ADVISORY, never the load-bearing gate.

The harness (deterministic machinery: rules engine, gates, checkers, the SSB
  journal, the FSM, the context-budget allocator, the auditor) owns everything
  that CAN be made mechanical:
  - SIZING / ADMISSION: is this unit the right size, does it fit the budget.
  - VERDICTS: pass/fail, grounded/ungrounded, wired/dark, reachable/orphan.
  - COUNTS: how many rows, files, callers, tokens.
  - PROVENANCE / REACHABILITY: did a real entrypoint run this, is this citation
    in the source, did this fact come from a trusted ingest site.
  - STATE: what phase we are in, what the budget is, what the trial count is.

Rule of thumb for a reviewer: read the feature, list every decision it makes,
  and sort each decision into "model" or "harness" by the lists above. Any
  decision in the harness column that is actually being made by the model is a
  finding.

================================================================================
THE FIVE SUB-DOCTRINES (each: statement / why / ELAI examples / anti-pattern)
================================================================================

------------------------------------------------------------
D1. GENERATOR PROPOSES, GATE DISPOSES
------------------------------------------------------------
Statement: the component that produces an artifact is never the component that
  admits it. Proposal and disposal are separated, and the disposer is
  mechanical wherever possible.
Why: a producer judging its own output is inside the boundary the judgment was
  meant to police; it rubber-stamps. (Berkeley RDI Apr-2026: in-boundary
  artifacts gamed ~100%.)
ELAI examples (grounded):
  - ID-1 (I-0 doctrine): "generator proposes / orchestrator owns state." The
    generator emits Intent->Directive; a process-separated Supervisor is the
    only side-effect executor and holds credentials (I-1 DIRECTIVE SUPERVISOR
    PLANE).
  - Rules engine (phase-3F): a proposed rule walks PROPOSED->CANDIDATE->SHADOW
    ->ACTIVE; an AUDITOR-sourced rule is stuck at PROPOSED until an INDEPENDENT
    second auditor approves; the finding-source auditor can NEVER approve its
    own rule.
  - Self-modification (phase-4H): "the modifying agent NEVER evaluates its own
    modifications"; DGM archive selects a parent variant, it does not let the
    proposer self-score.
  - Production-caller gate (D-030): the executor that writes a new pub fn is not
    the authority on whether it wired it; the auditor re-runs the checker
    off-agent.
Anti-pattern to flag: a single agent that writes the implementation AND the
  test AND the wiring-contract that judges it; an LLM that scores its own
  candidate; a decomposer that declares its own nodes correctly-sized.

------------------------------------------------------------
D2. EVIDENCE IS OFF-AGENT AND FROZEN
------------------------------------------------------------
Statement: a gate measures an artifact the agent did NOT author (a journal row,
  a coverage hit from the compiled binary, an in-process span, a captured
  stdout hash), and the gate itself is frozen and read-only to the run being
  evaluated.
Why: a run that can write its own evaluator always writes itself to PASS. The
  load-bearing proof must be something the producer cannot fabricate.
ELAI examples (grounded):
  - F-4 cant-lie gate: AntiFabricationGate compares the auditor's claimed
    test result against a blake3 stdout_hash captured by the ORCHESTRATOR; the
    auditor cannot emit passed=true without the matching orchestrator-captured
    hash.
  - Wiring R8 + gate_freeze_check.py: the checker, smoke harness, entrypoints
    map, and allowlist are frozen in .claude/gate-manifest.txt; a run that
    modified its own gate is BLOCK.
  - "Main model never runs tests; the auditor is the sole test runner." The
    auditor sees a clean context; the producing model is biased toward
    declaring its own code correct.
  - production_caller_check.py / maintainability_check.py / traced_test_assert
    _check.py: all auditor-invoked over the changeset, never run by the
    executor as a self-pass.
Anti-pattern to flag: a gate that reads the agent's own stdout/log/claim; a
  "smoke test" written by the same agent that wrote the feature; a checker the
  executing plan can regenerate or edit.

------------------------------------------------------------
D3. MISSING EVIDENCE IS A HARD FAIL (fail-closed)
------------------------------------------------------------
Statement: absence of proof is proof of absence for gate purposes. A required
  field that is missing, an evidence DB that is not found, a gate pointed at a
  contract set that finds ZERO contracts -> FAIL, never SKIP / UNVERIFIED /
  WARN.
Why: "not found" usually means the feature never ran and produced it. A gate
  that shrugs on missing evidence is fail-open and is worse than no gate
  because it advertises coverage it does not have.
ELAI examples (grounded):
  - Wiring R1/R2: every required gate field is required; a gate whose evidence
    path is absent returns FAIL and self-tests its evidence path at startup
    (the dead evidence path must scream, not shrug). ArchUnit failOnEmptyShould.
  - GROUNDING VERIFICATION (phase-2B): plan sweep BLOCKS PLANNING->EXECUTING on
    ANY ungrounded claim; binary pass/fail, no soft "probably fine."
  - tracing-test rule (T-1/T-2): a #[traced_test] with zero logs_contain inside
    an assert! is BLOCK; the macro does not auto-fail, so a captured-but-
    unasserted log is wire-dark.
  - production_caller_check.py: exit 2 fail-closed on a scan-config error
    (git unreachable, zero new symbols found) — it never silently passes.
Anti-pattern to flag: any verdict of SKIP/UNKNOWN/UNVERIFIED that is treated as
  not-a-failure; a gate with an opt-in --skip flag; "no findings" reported
  without proof the checker actually reached the files.

------------------------------------------------------------
D4. MEASURE THE OBJECTIVE, NOT A PROXY
------------------------------------------------------------
Statement: the gate measures the real thing you care about, not a stand-in that
  correlates with it until the model learns to satisfy the stand-in.
Why: any proxy becomes a target and then stops measuring the objective
  (causal-Goodhart). "Compiles + unit tests pass" is a proxy for "the feature
  is wired and works"; a name-grep is a proxy for "a real caller exists"; bytes
  are a proxy for performance.
ELAI examples (grounded):
  - Wiring R4: the objective is "a real entrypoint ran it and left a trace the
    agent did not author," not the agent's stdout or its own unit test.
  - SAFEDELTA (G-6): gate-ON-vs-OFF ablation ("N fewer exploitable diffs per
    100 tasks"), explicitly NOT a leaderboard rank (SWE-bench Verified dropped
    as headline, E-9 / GD-3).
  - DGM fitness (Track 7 / VD-1): the LIVE placeholder derived_alpha =
    blake3(config_bytes) is "fitness of BYTES not performance" and is a KNOWN
    GAP; H-43 replaces it with execution-grounded pass-rate over frozen tests.
  - R5 (skill eval extension): test the skill CHANGED behavior (baseline-
    without-skill RED vs with-skill GREEN), not just that the with-skill run
    passes.
Anti-pattern to flag: a metric that the producer can satisfy without doing the
  real work; a hash/byte/length proxy standing in for a behavioral outcome;
  green tests treated as sufficient (necessary, never sufficient).

------------------------------------------------------------
D5. WHEN AN LLM IS IN A GATE, IT IS ADVISORY AND CROSS-FAMILY
------------------------------------------------------------
Statement: for judgments with no mechanical oracle, an LLM may contribute but
  is never the load-bearing decision: it is one advisory vote, it never grounds
  a claim or sits on a critical path, and a critic LLM is always a DIFFERENT
  pretraining family than the producer.
Why: same-model self-critique shares the producer's blind spots; an LLM verdict
  on a critical path is a fail-open gate wearing a gate's clothes.
ELAI examples (grounded):
  - TD-3: "the LLM is T3-advisory, never the load-bearing gate." The mechanical
    legs (METRIC_GATE/GREP/AST predicates) are the gate; the LLM lens is the
    last, weakest tier.
  - D-13 ADVERSARY ROLE: the adversary must be a different pretraining family
    from the executor; same-model self-critique is BANNED. VD-4: verifier must
    be stronger-than-generator and cross-family.
  - Mode-routing predicate (D-10): select_grounding_mode is deterministic — the
    LLM has NO input into which grounding mode is selected.
  - HD-3: never force-JSON / constrained-decode the reasoning/coder/planner
    roles; grammars only on pure op-emission (arXiv:2408.02442). Constraining
    the model's reasoning to satisfy a schema degrades the reasoning.
  - AutoB2G DAG repair (phase-4E): the 2 repair strategies (REORDER for cycles,
    SUBSTITUTE for missing tools) are DETERMINISTIC; the LLM-driven SOCIA repair
    was NOT adopted.
Anti-pattern to flag: an LLM-as-judge on a release-gating path; same-family
  self-critique counted as verification; a single model vote treated as
  grounding.

================================================================================
HOW THIS APPLIES TO DECOMPOSITION AND RESEARCH (the live debate, 2026-06-16)
================================================================================
The instinct "let the model decide babystep size" violates the boundary: SIZING
  is a harness decision (D-doctrine boundary), not a generative one. ELAI
  already has the deterministic scaffold and should lean on it harder:
  - Phase 4E Stage 3.5 PHASE SIZING: phase_complexity = files_touched *
    avg_file_tokens + test_count * avg_test_tokens + contract_count * 200; split
    if > available_for_work * 0.6; max split depth 2. (Deterministic TODAY, but
    coarse: avg_file_tokens is a heuristic, not the measured per-(model,lane)
    context ceiling.)
  - Phase 4H AllowedConfigKey BabyStepMaxLoc{vram_tier}: babystep max LOC is
    already a per-VRAM-tier, self-mod-tunable knob.
  - Phase 3E CONTEXT BUDGET ALLOCATOR: deterministic token estimator
    (token_count_estimate for allocation, exact token_count for validation),
    8 fixed slots, never exceeds the model window.
  - Phase 4G AutoAdvanceEvaluator: every FSM transition maps to a deterministic
    EvidenceCheck with NO LLM.
  - Measured ceilings (metrics/baseline_findings_and_actions.md): 3B NF4+FA2
    ~40K, 7B ~25K, both 100% pass on a 12GB card. J-34 (PLANNED) replaces the
    folklore thresholds with measured per-(model,lane) fill->degradation curves.
The correct extension (model is a cog): the model PROPOSES a decomposition; a
  deterministic NODE-ADMISSION GATE measures each proposed node (files<=2,
  impact-cone<=K from the B-29 DKG impact API, pre-computed context-package size
  <= measured-lane-ceiling * safety_factor, intent_pairs>=3, single
  observable_effect) and REJECTS oversized nodes, forcing a re-split or a
  cloud-lane escalation (I-7 cascading escalation). The model never answers "is
  this the right size."

For RESEARCH: the model SELECTS (formulates queries, ranks pages); the harness
  FETCHES (already true via the Phase 4D Tier-C IPC web-search proxy + the
  sidecar /search endpoint + E-48 CitationVerifier) and FINGERPRINTS the fetched
  source; grounding is then a deterministic check against the harness-fetched,
  hashed corpus (span/n-gram containment as a cheap first pass, claim-level
  entailment for the rest, closed-set citation binding per H-53). Token-level
  hash containment is the WRONG granularity (function/common tokens are in every
  source -> vacuous; strict enforcement forbids legitimate synthesis); span +
  claim + closed-set citation is the sound form.

================================================================================
THE REVIEWER CHECKLIST (apply to any ELAI feature)
================================================================================
For the feature under review, answer each. Any "no" on 2-7 is a finding.
  1. What does the model decide here? (enumerate every decision)
  2. For each decision: is it generative/selective (legitimately the model's,
     D-boundary) or sizing/admission/verdict/count/provenance/state (should be
     the harness's)? Flag any harness-class decision the model is making.
  3. Is the model's output checked by a mechanism it did NOT author? (D1/D2)
  4. Is the checking artifact frozen and read-only to the run? (D2)
  5. Is missing/absent evidence treated as FAIL, not SKIP/UNVERIFIED? (D3)
  6. Does the gate measure the real objective, or a proxy the producer can
     satisfy without doing the work? (D4)
  7. If an LLM is anywhere in the gate, is it advisory-only AND cross-family,
     off the critical path, never grounding a claim? (D5)
  8. If the feature ships behind a default-OFF flag, is the flag registered with
     a promotion plan (not a graveyard)? (wiring R9)
A feature that passes 3-8 is "model-as-cog clean." A feature that works only
  because the model happened to decide well is debt, regardless of green tests.

================================================================================
SOURCES (for the reviewing model to verify against, do not trust this file alone)
================================================================================
- docs/specs/guide/steps/phase-index-MASTER.txt (phase 2B/3E/3F/4E/4F/4G/4H).
- .claude/rules/wiring-enforcement.md (R1-R9, the layered gate, residual risk).
- .claude/rules/production-caller-gate.md (D-030, off-agent changeset gate).
- .claude/rules/maintainability-review.md (auditor-applied, frozen, fail-closed).
- .claude/rules/tracing-test-assert.md (T-1/T-2/T-3 fail-closed assert rule).
- docs/thefinalglance/ELAI-FEATURE-MAP.md (status of every named feature; the
  Section 3 doctrine blocks W/GD/HD/TD/VD/RV/CL/ID/JD; Section 4 rejected ledger).
- metrics/baseline_findings_and_actions.md (measured context ceilings).
