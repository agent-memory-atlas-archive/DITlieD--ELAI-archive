# D-12: OperationalizationCheck baby-step predicate

Objective: every claim-emitting baby step must co-emit a MEASURABLE_PROCEDURE
field that operationalizes the claim into a runnable test (per Voyager
arxiv:2305.16291 T1 skill-with-measurable-effect pattern). Without the
procedure, the baby step CANNOT enter MEMORIZING; auditor BLOCKs at
acceptance gate. Real-problem §4.1 (lines 185-196): "every retained claim
produces a measurable procedure."
Estimated effort: 1-2 sessions.
Pre-requisites: D-11 (checker registry; MEASURABLE_PROCEDURE references a
  checker). spec-grounder verifies DONE.
Parallel-safe with: D-13 (Adversary; orthogonal AUDITING-state mechanism),
  D-14 (sycophancy probe; orthogonal). NOT parallel-safe with D-15 (NoveltyClass
  labels checker results — same hook surface).

## Context

Real-problem §4.1 (lines 185-196): unfalsifiability filter. Voyager
(arxiv:2305.16291, T1) retains only executable skills that produce measurable
environment effect. ELAI's analog: every retained CLAIM produces a measurable
procedure that would FAIL if claim is wrong.

Currently ELAI's BabyStepSpec has 8 blocks (per C-31): TITLE, OBJECTIVE,
PRE_GATE, FILES_PLAN (C-31), TASKS, DETERMINISTIC_CHECK, TESTS, ACCEPTANCE.
D-12 adds block 9: MEASURABLE_PROCEDURE.

The procedure references a CheckerEntry by name + provides any data the checker
needs (claim, candidate, optionally domain-specific args). At AUDITING state
exit, auditor invokes orchestrator runtime to execute the procedure; the
checker's CheckerResult contributes to ACCEPTANCE gate decision.

The L3-locked rule: a claim WITHOUT MEASURABLE_PROCEDURE cannot enter MEMORIZING;
babystep-author agent emits one alongside claim; auditor BLOCKs if missing.

Partial foundation present and usable:
- C-31 FILE_PLAN block 8 pattern (extension model for new BabyStepSpec block).
- Auditor agent (.claude/agents/auditor.md) acceptance-gate hook.
- D-11 CheckerEntry types + CheckerRegistry.
- D-1 ReasoningTemplate.verifier_required field (overlapping concept; template
  is the system-level mutation surface, MEASURABLE_PROCEDURE is the per-claim
  hook).

## Reads

- F:\.ELAI_workflow\docs\specs\guide\real-problem-solving-grounded-2026-05-19.md
  (§4.1 OperationalizationCheck — lines 185-196)
- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-31_code_completeness_file_plan.md
  (FILE_PLAN extension pattern)
- .elai_cc/crates/orchestrator/src/baby_step.rs or wherever BabyStepSpec lives
  (grep first; current 8-block layout)
- .elai_cc/crates/orchestrator/src/checker_registry.rs (D-11)
- .claude/agents/auditor.md (auditor acceptance-gate behavior)

## Tasks

1. **MEASURABLE_PROCEDURE schema** in baby_step.rs (extending existing
   BabyStepSpec):
   - Add field: `measurable_procedure: Option<MeasurableProcedure>` (None
     during transition; D-12 enforcement makes it required for claim-emitting
     baby steps).
   - `struct MeasurableProcedure { checker_name: String, claim_payload:
     BoundedString<2048>, candidate_payload: Option<BoundedString<8192>>,
     domain_args: Option<BoundedString<1024>>, expected_result: CheckerResult }`
   - `expected_result` is what the babystep-author claims will happen;
     orchestrator runs the procedure + diffs.

2. **Baby-step parser extension** in parser logic (BabyStepSpec deserialization):
   - Block 9 token: `MEASURABLE_PROCEDURE`.
   - Parsing format (TOML-compatible, easy to author):
     ```
     MEASURABLE_PROCEDURE:
       checker: sympy
       claim: "x^2 = (x+1)*(x-1) + 1"
       expected_result: Pass
     ```
   - Parser populates measurable_procedure field; on parse error, emit
     ParseError diagnostic.

3. **Babystep-author agent prompt update** (.claude/agents/babystep-author.md):
   - Document: every CLAIM-emitting baby step MUST include MEASURABLE_PROCEDURE
     block.
   - Refusal example: a baby step claiming "this algorithm has O(n log n)
     complexity" but emitting no procedure → auditor BLOCK.
   - Compliance example: same claim with `MEASURABLE_PROCEDURE: checker: perf,
     claim: "asymptotic curve fit on n=10..10000", expected_result: Pass`.

4. **Auditor acceptance-gate enforcement** (.claude/agents/auditor.md):
   - Document: auditor scans current baby step for claim-emit pattern;
     if claim present + measurable_procedure absent → BLOCK with reason
     "missing MEASURABLE_PROCEDURE".
   - Document: auditor invokes orchestrator to run measurable_procedure;
     if actual CheckerResult ≠ expected_result → BLOCK with reason
     "measurable procedure result diverges from expected".

5. **Orchestrator runtime check** in
   .elai_cc/crates/orchestrator/src/operationalization.rs (new file):
   - `fn require_procedure_for_claim(baby_step: &BabyStepSpec) ->
     Result<(), OperationalizationError>`:
     scans baby_step for claim-emit pattern (e.g. "produces_symbols" claims,
     "produces_artifact" claims with novelty hints); requires
     measurable_procedure if any found.
   - `fn run_procedure(procedure: &MeasurableProcedure, registry:
     &CheckerRegistry, ctx: &OrchContext) -> CheckerResult`:
     dispatches per D-11 CheckerRegistry.
   - `fn diff_expected(expected: &CheckerResult, actual: &CheckerResult) ->
     Option<ProcedureDivergence>`: equality check; populate divergence.

6. **Claim-detection helper** in operationalization.rs:
   - `fn detect_claim_in_baby_step(baby_step: &BabyStepSpec) ->
     Vec<DetectedClaim>`:
     - Look for keywords like "this is correct", "complexity is", "produces
       <new symbol>", "guarantees", "ensures", in baby_step.tasks +
       observable_effect blocks.
     - Per-detection rules; conservative — false-positive on detection is
       acceptable (forces babystep-author to add procedure), false-negative is
       not (lets a claim slip through).

7. **SSB action types** in ssb.rs (SCHEMA_VERSION 57→58):
   - `ClaimWithoutProcedure { baby_step_id: String, claim_snippet: String,
     audit_id: Uuid }`
   - `ProcedureExecuted { baby_step_id: String, checker_name: String,
     result: String, expected: String, divergence: Option<String> }`
   - Round-trip tests.

8. **Unit tests** in operationalization.rs:
   - detect_claim_in_baby_step on synthetic baby steps; classify correctly
     (3 with claims, 3 without).
   - require_procedure_for_claim returns error when claim present + procedure
     absent.
   - diff_expected: Pass==Pass → None; Pass≠Fail → Some(divergence).

9. **Integration test** in
   .elai_cc/crates/orchestrator/tests/operationalization_integration.rs:
   - Synthetic claim-emitting baby step WITHOUT procedure → auditor invocation
     → BLOCK + ClaimWithoutProcedure SSB row.
   - Synthetic claim-emitting baby step WITH procedure → auditor invocation
     → procedure runs → if Pass: allow; if Fail: BLOCK + ProcedureExecuted SSB
     row with divergence.

10. **Decision doc** workflow/reasoning/D-12-decisions.md (3 decisions):
    - (a) Why a 9th BabyStepSpec block (not embedded in DETERMINISTIC_CHECK):
      DETERMINISTIC_CHECK is unstructured shell; MEASURABLE_PROCEDURE is
      typed + parseable; separation simplifies tooling.
    - (b) Why expected_result + actual diff (not "must always Pass"):
      auditor can verify Fail-as-expected for negative tests (e.g. "this
      malformed input must fail compile"); claim-as-prediction is structurally
      richer.
    - (c) Why conservative claim detection: false-positive (forcing extra
      procedure) is cheap; false-negative (silent untested claim) is the bug
      D-12 exists to prevent.

## WIRING_CONTRACT

produces_symbols:
  - MeasurableProcedure
  - require_procedure_for_claim + run_procedure + diff_expected
  - detect_claim_in_baby_step + DetectedClaim
  - SsbActionType::ClaimWithoutProcedure + ::ProcedureExecuted + payloads
  - SCHEMA_VERSION bump 57→58
  - BabyStepSpec block 9 (MEASURABLE_PROCEDURE)
consumes_symbols:
  - CheckerRegistry (D-11)
  - CheckerResult (D-11)
  - BabyStepSpec parser (existing)
  - Auditor agent (.claude/agents/auditor.md)
  - babystep-author agent (.claude/agents/babystep-author.md)
reachable_from:
  - AUDITING state → run_procedure dispatch
  - babystep-author at PLAN authoring time
stub_ban:
  - detect_claim_in_baby_step MUST be conservative (false-positive ok,
    false-negative NOT ok)
  - run_procedure MUST timeout per checker.timeout_secs
  - diff_expected MUST handle Pass/Fail/Inapplicable triad (not just bool)
  - require_procedure_for_claim MUST fail-closed (claim present → procedure
    required, no opt-out)
observable_effect:
  trigger: babystep-author emits claim-baby-step without procedure; auditor
           invoked
  expect_ssb: 1 ClaimWithoutProcedure row; AUDITING transition BLOCKED;
              human-review prompt issued
  expect_artifact: .claude/agents/babystep-author.md updated; .claude/agents/
                   auditor.md updated; BabyStepSpec parser accepts block 9

## DETERMINISTIC_CHECK

```bash
# 1. SCHEMA_VERSION 58
rg "SCHEMA_VERSION: i64 = 58" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. SSB variants
rg "ClaimWithoutProcedure|ProcedureExecuted" .elai_cc/crates/orchestrator/src/ssb.rs

# 3. MeasurableProcedure + types
rg "struct MeasurableProcedure|fn require_procedure_for_claim|fn run_procedure" \
  .elai_cc/crates/orchestrator/src/operationalization.rs

# 4. BabyStepSpec block 9
rg "MEASURABLE_PROCEDURE" .elai_cc/crates/orchestrator/src/baby_step.rs

# 5. Agent doc updates
rg "MEASURABLE_PROCEDURE" .claude/agents/babystep-author.md \
  .claude/agents/auditor.md

# 6. Build + tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- operationalization 2>&1 | grep "test result"
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 58
- [ ] 2 SSB variants round-trip
- [ ] MeasurableProcedure struct + helpers compile; unit tests pass
- [ ] BabyStepSpec parser accepts block 9
- [ ] detect_claim_in_baby_step correctly classifies ≥6 synthetic cases
- [ ] require_procedure_for_claim returns error when expected
- [ ] diff_expected handles 3-state CheckerResult
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- operationalization: all pass, ≥8 tests
- [ ] babystep-author agent doc updated with examples + refusal pattern
- [ ] auditor agent doc updated with enforcement pattern
- [ ] Cloud smoke test A: cloud /task that produces a CLAIM baby step;
      verify MEASURABLE_PROCEDURE block present; auditor runs procedure;
      ACCEPTANCE gate passes on Pass result
- [ ] Cloud smoke test B: synthetic baby step missing procedure → AUDITING
      BLOCKED + ClaimWithoutProcedure SSB row
- [ ] Cloud smoke test C: baby step with expected=Pass but actual=Fail →
      ProcedureExecuted row with divergence; AUDITING BLOCKED
- [ ] Decision doc workflow/reasoning/D-12-decisions.md

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/operationalization.rs
- MODIFY: .elai_cc/crates/orchestrator/src/baby_step.rs (block 9 field +
    parser extension)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION 57→58;
    2 new variants)
- MODIFY: .elai_cc/crates/orchestrator/src/lib.rs (pub mod operationalization)
- MODIFY: .claude/agents/babystep-author.md
- MODIFY: .claude/agents/auditor.md
- ADD: .elai_cc/crates/orchestrator/tests/operationalization_integration.rs
- ADD: workflow/reasoning/D-12-decisions.md

## Out-of-scope

- Per-checker procedure templates (DEFERRED; ad-hoc authoring for now)
- Procedure auto-generation by babystep-author from claim text (DEFERRED;
  human-authored procedures more accurate)
- Procedure caching (DEFERRED; each AUDITING runs fresh)

## Next

D-13 (Adversary role) is next; it extends AUDITING with cross-family
adversarial probing for claims that DON'T have a Mode A checker.
