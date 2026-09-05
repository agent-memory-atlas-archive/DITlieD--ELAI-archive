Q-14 DECISIONS — conjecture-engine instrument hardening (E-1 first)

DATE 2026-07-18
PLAN docs/thefinalglance/Q-plan/Q-14_conjecture_engine_instrument_hardening.md (plan_gate_check PASS)
PRECONDITION B-SYN-0 = SUPPORTED (workflow/reasoning/B-SYN-0-decisions.md); E-1's verdicts leg cleared
  to proceed to its next gated implementation step, BOUNDED to in-distribution ELAI artifacts.

SCOPE THIS WAVE: E-1 only (SHIP FIRST). E-2/E-3/E-4 are later units in this same file, not this step.

RECON_NEEDED-1 RESULT (mandatory pre-E-1 re-read, settled 2026-07-18)
- Consumption point: conjecture_triage.py:835-836 loads --verdicts JSON then calls
  score_novelty(cards, archive, verdicts=verdicts) in conjecture_archive.py.
- score_novelty (conjecture_archive.py:174-209): per-candidate, supplied verdict ->
  _validated_verdict (conjecture_archive.py:152-171) -> instrument_strength="NLI-JUDGED";
  no supplied verdict -> bin-coordinate WEAK fallback (equivalence=UNCERTAIN,
  instrument_version="bin-coordinate-v1", instrument_strength="WEAK"). This is the always-UNCERTAIN
  baseline B-SYN-0's A2 arm shadowed.
- Verdict-file row schema (validated by _validated_verdict): {equivalence in {SAME,DISTINCT,UNCERTAIN},
  nearest_known_mechanism in archive ids, instrument_version non-empty, judge_votes matching \d+/\d+}.
- FINDING: NO functional-equivalent canary exists. instrument_strength is only ever NLI-JUDGED or WEAK;
  there is NO FAILED state, NO known-answer canary pair, NO fail-closed suppression on that leg.
  => RECON-1's "live equivalent collapses E-1 to wiring-only" branch does NOT fire. E-1 is a real build.

E-1 DELIVERABLE (from the plan, unchanged)
  (a) documented cheap MINT path for the NLI verdicts file (spawned judge, model=sonnet, fresh context) —
      OPTIONAL network leg; absent judge -> existing honest WEAK degrade (FORK route B, unchanged).
  (b) a known-SAME + known-DISTINCT canary PAIR injected into EVERY --novelty invocation; a canary MISS
      (known-SAME judged DISTINCT or vice-versa, OR a scrambled/identity-mismatched verdicts file)
      fail-closes to instrument_strength=FAILED and SUPPRESSES all novelty verdict rows for that run
      (writes NO verdict rows, not advisory), mirroring Q-11 C6.
  (c) --self-test-novelty-canary CLI arm that FAILS if no canary pair is wired (dead-evidence-path
      screams, wiring R2). The canary oracle is DETERMINISTIC known-answers, never an LLM judge (ABORT-b).

INTEGRITY BACKSTOP (this family): the free-executor Agent lane (glm-5.2) was down during B-SYN-0, forcing
  a solo build there. For E-1 the executor lane is attempted first (proper delegation + builder-is-not-
  main-agent independence). If the lane is down again, fall back to the owner-approved pattern for this
  family: main-agent solo build under RULE ZERO (tests verify machinery, never outcome), an INDEPENDENT
  checker agent (0 producer imports), and the adversarial-auditor. Either path keeps
  execution/evaluation separation (D-6): the builder never authors the check that judges its own output.

LANE PATH TAKEN: SOLO. Executor Agent (subagent_type=executor, glm-5.2) dispatched 2026-07-18 and
  terminated with the same API error as B-SYN-0 ("issue with the selected model (glm-5.2), it may not
  exist or you may not have access"). The free-executor lane is down. Fallback engaged per the family
  pattern: main-agent solo build under RULE ZERO (machinery-not-outcome tests) -> INDEPENDENT checker
  agent (separate context, 0 producer imports) -> adversarial-auditor. Execution/evaluation separation
  (D-6) preserved: the checker + auditor are authored by agents that did not write the E-1 code.

E-1 BUILD RESULT (main-agent solo, 2026-07-18)
FILES:
  .claude/tools/conjecture_triage.py — canary constants, novelty_canary_probes/_expectations,
    check_novelty_canary (known-answer + identity binding), self_test_novelty_canary, the fail-closed
    suppression wired into the `if args.novelty:` arm, the --self-test-novelty-canary CLI arm, and a
    _human FAILED branch. Documented (comment) minting path (sonnet judge, optional network leg).
  .claude/tools/tests/test_conjecture_novelty_canary.py — 16 RULE-ZERO tests (gate logic both sides;
    no outcome assertions).
DESIGN: the canary is a deterministic known-answer PAIR (reserved ids __novelty_canary_same__ /
  __novelty_canary_distinct__, fixed SAME/DISTINCT the canary owns, never an LLM oracle). A supplied
  --verdicts file is judged against it PLUS identity binding (no verdict keyed outside {run cards} ∪
  {canary ids}; reserved id used by a real candidate = miss). A miss => instrument_strength=FAILED and
  ZERO verdict rows (fail-closed, mirrors Q-11 C6). No --verdicts file => unchanged honest WEAK degrade
  (FORK route B; nothing to fail-close). The minting judge (sonnet) is documented + OPTIONAL, never a
  live call on the gate path.
SELF-VERIFY (all green):
  - python_wire_check conjecture_triage.py,conjecture_archive.py -> PASS, 0 DARK (4 new defs INTERNAL,
    consumed by main()'s CLI arms in-file, same class as the existing self_test).
  - 16/16 unittest green (pytest not installed on this host; tool tests are unittest-based).
  - --self-test-novelty-canary --json -> passed true (canary wired; scrambled/missing/foreign fail-closed).
  - regression: test_conjecture_v11 / _novelty_v11_ab / _multicycle_ab all OK.
  - M1 on the REAL AIDE2 fixture via the production --novelty entrypoint (arch A1, minted good file):
    top instrument_strength=NLI-JUDGED, canary intact, C1 equivalence=DISTINCT (non-UNCERTAIN). The
    inert leg is now live.
  - M2 same fixture, scrambled canary (known-SAME flipped to DISTINCT): instrument_strength=FAILED,
    0 candidate rows, profile suppressed, miss reported. Fail-closed.
SCOPE: E-1 only. novelty.rs, gate-manifest.txt, SKILL.md, ELAI-FEATURE-MAP.md, conjecture_archive.py
  NOT touched (E-2/E-3/E-4 are later units). .claude/tools + workflow are gitignored; no commit involved.
NEXT: adversarial-auditor validates off-agent (D-6 / PCG-3 / R10 integration smoke) before E-1 is DONE.

E-1 AUDIT ROUND 1 -> FIX (2026-07-18)
Auditor (adversarial-auditor, own model; glm-5.2 lane down does not affect it) returned NOT-READY:
  BLOCK MR-1 file-size crossing: conjecture_triage.py 890 -> 1073, crossed the 1000-line HARD boundary
    (maintainability-review MR-1). The auditor's own maintainability_check.py FALSE-CLEANED it (gitignored
    path -> empty `git diff HEAD` -> silent pass); it caught the crossing by hand.
  WARN reserved-id collision guard dead on the real path (parse_cards upper-cases ids; constants lowercase;
    test bypassed parse_cards). WARN _human FAILED branch untested. WARN shuffle-preserving-canary not
    documented. WARN maintainability_check.py false-cleans any gitignored .claude/tools|workflow path.
FIXES APPLIED:
  - Extracted the canary block (4 fns + 3 constants) to .claude/tools/conjecture_novelty_canary.py (155
    lines), mirroring the conjecture_archive.py / conjecture_profiles.py sibling-module precedent;
    conjecture_triage.py imports the 3 CLI-consumed symbols. Now 933 lines (< 1000). check_novelty_canary
    + self_test_novelty_canary now classify WIRED (external caller), better than the prior INTERNAL.
  - Collision guard made case-insensitive (reserved.upper() vs {cid.upper()}); its test now routes through
    parse_cards on a literal markdown card and confirms the guard fires.
  - Added a CanaryHumanOutput test exercising the non-json _human FAILED branch.
  - Added a SCOPE note to check_novelty_canary's docstring: it is a known-answer + identity-binding canary,
    NOT a permutation detector; a real-candidate shuffle that preserves the canary answers is out of scope
    (B-SYN-0 A3's job).
  - maintainability_check.py false-clean: NOT fixed here (frozen gate, R8 forbids editing the evaluator
    inside the run); escalated to the owner as a separate gate-reliability item.
RE-VERIFY (all green): 17/17 tests, wire PASS 0 dark (canary fns WIRED), self-test passes, M1 NLI-JUDGED
  live verdict / M2 FAILED zero-row unchanged on the real AIDE2 fixture, collision guard fires via
  parse_cards, regression (v11 / novelty_v11_ab / multicycle_ab) OK. Re-audit dispatched.

E-1 AUDIT ROUND 2 -> READY (2026-07-18, FINAL)
Adversarial-auditor re-verified off-agent: FINAL VERDICT READY, 0 BLOCKS, score 0.90. MR-1 crossing
resolved (933 / 155 lines, both < 1000; import mirrors the archive/profiles precedent; no circular import,
no duplicate defs). Collision guard re-attacked through the real parse_cards -> fires (intact False).
Human-mode FAILED output now covered (17th test). Shuffle-limit documented. M1/M2 re-run on the real
AIDE2 fixture BYTE-DIFFED identical to the pre-extraction outputs (NLI-JUDGED non-UNCERTAIN [C6,C10] /
FAILED 0 rows). Carried-forward WARNs are informational: maintainability_check.py gitignore false-clean
(frozen gate, escalated separately) and the full-plan wiring_contract_check FAIL (expected until
E-2/E-3/E-4 land; not an E-1 defect).

E-1 STATUS: DONE. PROOF-SPINE: E-1 built the harness novelty-measurement INTEGRITY leg (the canary +
fail-closed suppression). It does NOT by itself produce the PS-3 NUMBER (hallucination-delta) — that is
a separate measurement; E-1 is the integrity substrate PS-3 will be measured on. No commit (.claude +
workflow are gitignored; the tracked tree .elai_cc is untouched; no owner commit authorization anyway).

REGISTRY JUDGMENT (2026-07-18, owner-asked): .elai_cc/benchmarks/registry.json read in full — NO CHANGE
NEEDED. The registry is the measurement CONTRACT (6 PS-N spine numbers + `elai bench <sub>` feature-benches
under .elai_cc/benchmarks/ + the e2e lane). E-1 is a harness-tool INTEGRITY leg (novelty canary +
--self-test-novelty-canary on conjecture_triage.py, kept ADVISORY per Q-14: not gate-manifest, not an
elai bench subcommand), so it has no runner row to add. PS-3 (hallucination-delta, runner PLANNED,
provenance L-10) is what E-1 FEEDS but E-1 did not build its runner, so PS-3's row is unchanged; provenance
names the DEFINING unit (L-10), not downstream integrity legs. B-SYN-0 (SUPPORTED) is a preregistered
validation experiment under gitignored workflow/benchmarks/ on a different track, correctly absent from
this elai-bench registry. registry.json left byte-untouched; .elai_cc tree stays clean, no commit surfaces.
