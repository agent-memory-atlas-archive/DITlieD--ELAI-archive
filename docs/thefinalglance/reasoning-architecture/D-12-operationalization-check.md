# Operationalization check (D-12)

- **ELAI ids:** D-12 / RPS.III / D0-016 / D0-047 / D12-001..D12-038 / SUP-001
- **Spec status (as designed):** DARK(`ELAI_OPERATIONALIZATION_ENABLE`) (`ELAI-FEATURE-MAP.md:590-591`). D-SUMMARY labels the plan PLANNED (`D-plan/D-SUMMARY_INDEX.md:92`). Live hook is default-OFF (`task_runner.rs:3747-3765`, `runtime_config.rs:1379-1408`).
- **Crate / module (spec):** `crates/orchestrator/src/operationalization.rs`; block 9 on `BabyStepSpec` in `baby_step.rs`; auditor + babystep-author prompts
- **Purpose of this file:** enough architecture to rebuild the claim→measurable-procedure gate. Not a status ledger.

## 1. What it is

**OperationalizationCheck** is the baby-step predicate that every **claim-emitting** baby step must co-emit a typed `MEASURABLE_PROCEDURE` — a runnable checker invocation that would **fail if the claim is wrong**. Without the procedure the step **cannot enter `MEMORIZING`**; the auditor BLOCKs at acceptance (`D-plan/D-12_operationalization_check_predicate.md:1-8,32-33`). This is Voyager’s skill-with-measurable-effect applied to claims (real-problem §4.1).

## 2. Why it exists

Unfalsifiable retained claims rot the archive. If this organ is missing, a baby step can say “O(n log n)” and memorize it. If it is faked (a shell line in `DETERMINISTIC_CHECK` that never runs a checker), the auditor cannot diff expected vs actual. The ninth block is typed and parseable; `DETERMINISTIC_CHECK` stays unstructured shell (`D-plan/D-12_operationalization_check_predicate.md:140-143`).

## 3. How it works (runtime)

1. **Who calls it.**
   - **Authoring:** babystep-author must emit block 9 on every claim-emitting step (`D-plan/D-12_operationalization_check_predicate.md:79-81`).
   - **AUDITING exit:** auditor scans for claims; missing procedure → BLOCK `"missing MEASURABLE_PROCEDURE"`; present → orchestrator `run_procedure`; actual ≠ expected → BLOCK `"measurable procedure result diverges from expected"` (`:87-93,167-169`).
2. **Input.** `BabyStepSpec` (8 existing blocks + optional `measurable_procedure`). Detection also reads `tasks` + `observable_effect` prose (`:108-113`).
3. **Parse (deterministic).** Token `MEASURABLE_PROCEDURE`. TOML-like body. Parse error → `ParseError` diagnostic (`:67-77`).
4. **`detect_claim_in_baby_step` (deterministic, conservative).** Keywords include “this is correct”, “complexity is”, “produces \<new symbol\>", “guarantees”, “ensures”, plus `produces_symbols` / `produces_artifact` with novelty hints (`:95-116`). **False-positive OK** (forces a procedure). **False-negative forbidden**.
5. **`require_procedure_for_claim` (fail-closed).** Any detected claim + `measurable_procedure == None` → `Err(OperationalizationError)`. No opt-out (`:170-176`).
6. **`run_procedure`.** Dispatch through D-11 `CheckerRegistry` with `OrchContext`. Must honor `checker.timeout_secs` (`:102-104,173`).
7. **`diff_expected`.** Equality over the **triad** Pass/Fail/Inapplicable, not a bool. `Pass==Pass` → `None`; mismatch → `Some(ProcedureDivergence)` (`:105-106,128-130,174`).
8. **Commit.** Missing procedure → `ClaimWithoutProcedure` SSB + AUDITING blocked + human-review prompt. Executed procedure → `ProcedureExecuted` (with optional divergence) (`:118-123,177-183`).
9. **The model may generate** the procedure’s claim/candidate payloads. It may **not** skip the block, mark a claim as “not a claim”, or accept a Pass when the checker returned Fail.

**Transition field:** `measurable_procedure: Option<…>` during rollout; enforcement makes it **required** for claim-emitting steps (`:56-60`).

## 4. Types, enums, objects

Copy from `D-plan/D-12_operationalization_check_predicate.md:54-63,95-116`. Do not invent fields.

```
struct MeasurableProcedure {
  checker_name: String,
  claim_payload: BoundedString<2048>,
  candidate_payload: Option<BoundedString<8192>>,
  domain_args: Option<BoundedString<1024>>,
  expected_result: CheckerResult,   // D-11 triad; prediction, not "must Pass"
}

BabyStepSpec.measurable_procedure: Option<MeasurableProcedure>  // block 9

fn require_procedure_for_claim(baby_step: &BabyStepSpec)
  -> Result<(), OperationalizationError>

fn run_procedure(procedure, registry, ctx) -> CheckerResult

fn diff_expected(expected, actual) -> Option<ProcedureDivergence>

fn detect_claim_in_baby_step(baby_step) -> Vec<DetectedClaim>
```

**Authoring example** (`:69-75`):

```
MEASURABLE_PROCEDURE:
  checker: sympy
  claim: "x^2 = (x+1)*(x-1) + 1"
  expected_result: Pass
```

**Refusal example:** “this algorithm has O(n log n) complexity” with no procedure → auditor BLOCK. **Compliance example:** same claim with `checker: perf`, `claim: "asymptotic curve fit on n=10..10000"`, `expected_result: Pass` (`:82-85`). (perf checker itself is D-11-deferred; the example is the **pattern**.)

Eight existing blocks: `TITLE`, `OBJECTIVE`, `PRE_GATE`, `FILES_PLAN`, `TASKS`, `DETERMINISTIC_CHECK`, `TESTS`, `ACCEPTANCE` (`:23-25`). D-12 adds **block 9**. D-SUMMARY’s “block 7” is the **loser** (SUP-001, `ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md:2657`). Rebuild block **9**.

## 5. Control flow / state machine

**Owner:** auditor at AUDITING→MEMORIZING. Orchestrator runs the procedure; babystep-author only authors.

```
PLAN authoring → BabyStepSpec (+ block 9 if claim)
        ↓
AUDITING
  detect_claim?
    no  → existing gates
    yes → procedure present?
            no  → BLOCK + ClaimWithoutProcedure  (cannot MEMORIZING)
            yes → run_procedure → diff_expected
                    match     → procedure contributes to ACCEPTANCE
                    mismatch  → BLOCK + ProcedureExecuted{divergence}
        ↓
MEMORIZING   // only if not blocked
```

**Forced:** claim present ⇒ procedure required. Fresh run every AUDITING — no cache (`:250-251`).

**Not parallel-safe with D-15** (same checker-result hook). Parallel-safe with D-13/D-14 (`:12-14`).

## 6. Grounding, receipts, gates

The receipt is `CheckerResult` from D-11, diffed against `expected_result`. Negative tests are first-class: expected Fail on malformed input is a valid prediction (`:144-147`).

**Never allowed as a confidence signal:** LLM “the procedure would pass”; shell-only DETERMINISTIC_CHECK standing in for block 9; skipping detection because the author said “no claims”.

L3-locked rule: claim without procedure cannot enter MEMORIZING (`:32-33`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| claim_payload | `BoundedString<2048>` | per procedure |
| candidate_payload | `BoundedString<8192>` optional | per procedure |
| domain_args | `BoundedString<1024>` optional | per procedure |
| checker timeout | D-11 `timeout_secs` | per run |
| claim-detection unit set | 3 with claims, 3 without | tests |
| SCHEMA_VERSION (plan) | 57 → 58 | SSB |
| unit tests | ≥8 | acceptance |

No per-session procedure budget. Overflow of bounded payloads: **UNCERTAIN**.

## 8. Roles and neighbors

**babystep-author** emits. **auditor** enforces. Orchestrator executes. Consumes D-11 `CheckerRegistry` / `CheckerResult` and existing BabyStepSpec parser. Overlaps D-1 `ReasoningTemplate.verifier_required` (system-level mutation surface vs per-claim hook) — do not collapse them (`:39-41`).

**Produces:** block 9, `MeasurableProcedure`, two SSB variants, AUDITING BLOCK.

**Must not fork:** embedding the procedure only in DETERMINISTIC_CHECK; auto-generating procedures from claim text (deferred).

## 9. Config / flags / defaults

- **`ELAI_OPERATIONALIZATION_ENABLE`** — opt-in, same truthy set as D-11 (`runtime_config.rs:1379-1408`, `wire-dark-allowlist.toml:841-846`). Copy this flag. Original plan does not name it; FEATURE-MAP is the flag authority.
- Live: `operationalization_runtime: Option<…>` default None (`task_runner.rs:3747-3765`).
- Agent docs must be updated: `.claude/agents/babystep-author.md`, `.claude/agents/auditor.md` (`:79-93,240-241`).
- No `elai_cc.toml` section in the original D-12 plan.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `operationalization.rs` | require/run/diff/detect |
| `baby_step.rs` | block 9 field + parser |
| `ssb.rs` 57→58 | ClaimWithoutProcedure, ProcedureExecuted |
| `lib.rs` | `pub mod operationalization` |
| `.claude/agents/babystep-author.md` | must-include + examples |
| `.claude/agents/auditor.md` | missing / diverge BLOCK |
| `orchestrator/tests/operationalization_integration.rs` | two integration cases |
| `workflow/reasoning/D-12-decisions.md` | three decisions |
| live `agent_pool::run_operational_check` | `task_runner.rs:3752` |
| live `BabyStepDispatchedEmitter` / `baby_step_dispatched` | later fill (`runtime_config.rs:1411-1418`) |

C-31 FILE_PLAN is the **extension pattern** for adding a block, not the procedure itself (`:35-36,46-48`).

## 11. Journal / SSB events

Plan SCHEMA_VERSION **57 → 58**:

```
ClaimWithoutProcedure { baby_step_id: String, claim_snippet: String, audit_id: Uuid }
ProcedureExecuted { baby_step_id: String, checker_name: String,
                    result: String, expected: String, divergence: Option<String> }
```

(`D-plan/D-12_operationalization_check_predicate.md:118-123`). D-SUMMARY/`D-0` also name `CLAIM_WITHOUT_PROCEDURE` (`D-plan/D-SUMMARY_INDEX.md:94`, exhaustive D0-047). Live later fill: `baby_step_dispatched` (`task_runner.rs:3753`). Rebuild the **plan** pair; do not rewind live schema.

## 12. Tests and observables

| Case | Assert |
|---|---|
| detect 3+3 synthetics | claims vs non-claims |
| require_procedure | claim + none → error |
| diff Pass==Pass | None |
| diff Pass≠Fail | Some(divergence) |
| triad | Inapplicable handled, not bool-only |
| integration missing procedure | BLOCK + ClaimWithoutProcedure |
| integration with procedure Pass | allow |
| integration expected Pass actual Fail | BLOCK + ProcedureExecuted divergence |
| cloud A | claim baby step has block 9; auditor runs; ACCEPTANCE on Pass |
| cloud B | missing procedure → AUDITING BLOCK + row |
| cloud C | expected Pass / actual Fail → divergence + BLOCK |

Parser accepts block 9. ≥8 tests. Decision doc required.

## 13. Rebuild contract

Another harness has **this** organ iff:

- BabyStepSpec has **block 9** `MEASURABLE_PROCEDURE` as a typed `MeasurableProcedure` (not shell).
- Claim-emitting steps cannot enter MEMORIZING without it (fail-closed).
- Auditor BLOCK reasons are the two strings in §3.
- `run_procedure` goes through D-11; timeout honored.
- `diff_expected` is triad equality; expected Fail is legal.
- Detection is conservative (FP ok, FN not).
- Both SSB variants exist and round-trip.
- Author + auditor prompts document the rule.
- `ELAI_OPERATIONALIZATION_ENABLE` may dark the live hook, but a claim-emitting step in AUDITING must still fail closed when the gate is conceptually on.

A comment “we tested it” or a DETERMINISTIC_CHECK cousin is not this organ.

## 14. Non-goals / banned designs

- Per-checker procedure templates (deferred).
- Auto-generation from claim text (deferred).
- Procedure caching (deferred; every AUDITING is fresh) (`:245-251`).
- Treating D-SUMMARY “block 7” as the target (SUP-001 loser).
- Always-Pass (destroys negative tests).

## 15. Source citations

Primary: `D-plan/D-12_operationalization_check_predicate.md` (full file). Index: `D-plan/D-SUMMARY_INDEX.md:92-96` (block-7 slip — ignore). Feature map: `ELAI-FEATURE-MAP.md:590-591`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:235-236`. Exhaustive: D0-016, D0-047, D12-001..038, SUP-001. Live: `runtime_config.rs:1379-1408`, `task_runner.rs:3747-3765`, `wire-dark-allowlist.toml:841-846`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Exact `DetectedClaim` / `OperationalizationError` / `ProcedureDivergence` field lists (named, not specified).
- Keyword list completeness vs `produces_symbols` structured scan.
- How `Option<MeasurableProcedure>` becomes required (parser vs auditor vs both).
- Live `baby_step_dispatched` vs plan `ClaimWithoutProcedure`/`ProcedureExecuted` mapping.
- perf-checker example while D-11 defers perf scripts.
- BoundedString overflow policy.
- Interaction when D-11 returns Inapplicable and expected was Pass (BLOCK yes; human-review? unspecified beyond divergence).
- Parallel-unsafe hook sharing with D-15: merge order unspecified.
