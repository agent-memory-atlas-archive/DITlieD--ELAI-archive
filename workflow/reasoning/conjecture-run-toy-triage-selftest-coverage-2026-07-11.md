# conjecture-run: toy-triage-selftest-coverage (2026-07-11)

This is the UNIT-1 MOVE-1.3 TOY smoke for the improvement-conjecture engine — a minimal
end-to-end run that proves the run-artifact contract (seven sections + KILL-EDGE grammar +
--validate exit 0). It is NOT the UNIT-5 hard integration smoke (that is an independent
auditor-class agent on a hard trigger).

## TRIGGER

Verbatim: "improve the conjecture_triage.py self-test coverage".
Routing fork taken (FORK-1): standalone /conjecture (a named weakness in a harness tool).
Anchor oracle (FORK-2): elai.map is STALE (elai_map_freshness.py reports 107h) AND the new
conjecture_triage.py symbols are not yet in it, so this toy run substitutes a small fixture
anchor map (scratchpad/toy_fixture.map) as the anchor oracle and records the substitution
here. The UNIT-5 hard smoke regenerates elai.map or uses codegraph-elai as the real oracle.

## BIN MATRIX

Coordinates attempted (mechanism_family x target_area), target_area = "conjecture-triage-tool":
- measurement/bench x conjecture-triage-tool   FILLED (C2)
- gate/checker x conjecture-triage-tool         FILLED (C3)
- doctrine/process x conjecture-triage-tool     FILLED (C1)
- integration/seam x conjecture-triage-tool     EMPTY-ATTEMPTED (no distinct candidate this toy round)

## CANDIDATES

### CANDIDATE C1
- id: C1
- mechanism_family: doctrine/process
- target_area: conjecture-triage-tool
- target_units: conjecture_triage, self_test
- expected_effect: more self-test fixtures is generally good
- cost_class: S
- falsifier:
- bin: doctrine/process x conjecture-triage-tool
- stance: Hacker
- typicality: 0.8
- proof_spine: none (discipline)
- external_claims: none

### CANDIDATE C2
- id: C2
- mechanism_family: measurement/bench
- target_area: conjecture-triage-tool
- target_units: self_test, conjecture_triage
- expected_effect: add a fixture card exercising the (f) external-claim RECON flag so that path is covered
- cost_class: S
- falsifier: run `python .claude/tools/conjecture_triage.py --self-test`; pass-if a new external-claim fixture reports the recon_external flag
- bin: measurement/bench x conjecture-triage-tool
- stance: Simplifier
- typicality: 0.3
- proof_spine: feeds: PS-6 (a wire-dark-rate-adjacent tool gains coverage; discipline-class otherwise)
- external_claims: none

### CANDIDATE C3
- id: C3
- mechanism_family: gate/checker
- target_area: conjecture-triage-tool
- target_units: check_external, conjecture_triage
- expected_effect: studies show property-based tests find more edge cases, so add a property test
- cost_class: M
- falsifier: run a small hypothesis-style script over check_external; pass-if it flags a mutated card
- bin: gate/checker x conjecture-triage-tool
- stance: Researcher
- typicality: 0.5
- proof_spine: none (discipline)
- external_claims: "studies show property-based tests find more edge cases" (no source cited)

## STAGE-2 KILLS

(populated by `python .claude/tools/conjecture_triage.py --run <this file> --elai-map scratchpad/toy_fixture.map --json`)

KILL-EDGE: C1 <- triage:demarcation

C2: pending (survives stage 2 — valid falsifier, real anchors, no ledger dup).
C3: pending (survives kills; FLAG recon_external — external prior-art claim without a verified source
    pins it at pending until the claim is grounded or dropped).

## STAGE-3 RESULTS

- C2 MICRO-EXPERIMENT (SF-1 executed leg): executed the falsifier — `python
  .claude/tools/conjecture_triage.py --self-test` runs green (exit 0). Receipt: the self-test PASS is
  the executed check that C2's "add a coverage fixture" is on a working test harness. blake3 receipt of
  the self-test JSON recorded at smoke time (toy: receipt-present marker, hash elided in the toy run).
- C3: judge-only + recon-pending. The external claim ("studies show property-based tests...") is
  unresolved (no source), so C3 stays PENDING and cannot win (anti-laundering, ATTACK-1).
- C1: refuted at stage 2 (no stage-3 budget spent — a refuted candidate is not tested).

## PICK

Top candidates by SF-1 preference order (executed micro-experiment > cited estimate > judge-only):
1. C2 (measurement/bench) — executed leg (self-test green), real anchors, cheap (S), directly extends
   the tool's own coverage. Falsifier is runnable and was run. WINNER.
2. C3 (gate/checker) — plausible (property test) but pinned PENDING on an ungrounded external claim;
   cannot win until grounded.
3. C1 (doctrine/process) — REFUTED (forbids nothing; no falsifier).

PICK: C2. Status winner (has an executed leg). NOT PICKED-PENDING — an executed candidate exists.
The winner flows into the invoking decomposition unchanged; this toy stops here (no plan authored).

## RECHECK HOOKS

Mechanical loser re-check: `grep -n "KILL-EDGE:" workflow/reasoning/conjecture-run-*.md` then reinstate
any candidate whose killer changed. C1 is refuted by triage:demarcation — it is reinstated to pending
the moment it acquires a runnable falsifier (re-run the triage). C3 leaves pending the moment its
external claim is grounded (a verified source) or dropped. No premise-killed candidate this run, so no
suspended entries.
