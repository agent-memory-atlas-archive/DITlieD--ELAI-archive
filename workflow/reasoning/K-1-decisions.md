K-1 decisions (problem-quantification dossier + claims policy)

What: authored docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md,
docs/strategy/ELAI-CLAIMS-POLICY.md, .claude/tools/claims_policy_lint.py,
docs/thefinalglance/K-plan/_fixtures/claims_negative.md; added one
conditional-routing.md line; per K-1's Tasks 1-6.

Key decisions:
- Plan-gate check on K-1 itself FAILS (legacy plan, authored before
  plan-standard.md's wargame blocks: no EXPECTED_OBSERVATION/FORK/
  RECON_NEEDED/ABORT/VERIFY/REDTEAM markers). Per dispatch instruction,
  proceeded on the plan's stated intent rather than rewriting the plan
  file (out of this task's scope).
- Lint adjacency is per-paragraph-block (blank-line-delimited), not
  same-line: same-line-only failed on the dossier's own multi-line prose
  (a number on one line, its [EXTERNAL] KW-N tag two lines later in the
  same paragraph). Block-level adjacency passes real prose while the
  negative fixture (each seeded violation in its own block) still BLOCKs.
  Residual risk documented in the tool's docstring: a dense multi-claim
  block with no blank line between distinct claims can leak clearance
  from one claim's tag to a neighboring untagged claim in the SAME block
  -- authors of table-shaped copy must keep one claim per block.
- KW-5 (SpecBench, arXiv:2605.21384) was NOT left as "PLAN-asserted,
  unquoted" -- WebFetch + WebSearch against the live arXiv abstract during
  this session confirmed the exact quote ("the gap...grows by 28
  percentage points for every tenfold increase in code size") and the
  paper's real title/authors, so KW-5 ships [EXTERNAL] (verified) rather
  than [EXTERNAL, unverified].
- ELAI's own measured rows (1-4) are ALL PENDING: no hallucination-rate
  bench exists, SeCodePLT/SAFEDELTA substrate is present but no live run
  has produced a number on this tree (grep-verified: no
  SafeDeltaMeasured SSB row, no `hallucinat` hit under crates/*/src/
  benchmarks/), and K-4's gate-tax bench is not yet authored. This session
  did NOT invoke `elai bench seccodeplt` / `elai bench safedelta` live
  (out of scope for a doc-authoring dispatch; recompute commands are
  named in the dossier for a future session to run).
- Did not author docs/strategy/ELAI-CLAIMS-POLICY.md's "Compliant vs
  non-compliant" examples as literally lint-clean; the two demonstration
  lines are DELIBERATELY uncleared and the doc says so explicitly, so a
  future direct lint-scan of this specific file is expected to BLOCK on
  exactly those two lines (self-consistent, not a defect).
- Did not add crates/contracts/tests/k1_claims_policy_contract.rs or a
  Python unit-test file: the plan's "Tests to add" section offers a
  Python test as an OR-alternative and the plan's own "Files touched"
  list (the DETERMINISTIC_CHECK-gated deliverable set) does not include
  one. Verification instead ran the lint directly against 5 cases (a-e)
  ad hoc during this session (see VERIFIED below); a follow-up session
  can promote these into .claude/tools/tests/test_claims_policy_lint.py
  matching the test_python_wire_check.py convention if the auditor wants
  a persisted regression test.

Expected test outcome: `claims_policy_lint.py --files
docs/thefinalglance/K-plan/_fixtures/claims_negative.md` exits 1 (BLOCK);
`--files docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md` exits 0
(PASS); a missing-file input exits 2 (SCAN_CONFIG_ERROR, fail-closed per
R1/R2); `rg -n "\[EXTERNAL\]|\[DESIGN\]"` over the dossier returns 6 hits.

Unknowns / open for a follow-up session:
- Row 1 (hallucination) and Row 4 (orchestration overhead) have no ELAI
  bench at all yet -- scoping those benches is out of K-1's stated scope
  (K-1 is doc + lint, near-zero code) and is a separate future unit.
- The lint's per-block adjacency has the documented table-row leak risk;
  it has not been exercised against a real dense markdown table in this
  session (the dossier avoids that shape by design).
- K-1's own plan file failing plan_gate_check.py was not remediated (out
  of this dispatch's scope per instruction); K-7 or a later K-session
  should run the wargame-refinement pass on it if it is walked again.
