# CSQ and Inline Feasibility (Phase 2D)

- **ELAI ids:** phase-2D / P2D-E001..P2D-E023
- **Spec status (as designed):** WIRED
- **Crate / module (spec):** `crates/reasoning` (`types.rs` CSQ/feasibility types, `engine.rs` `generate_csq` / `check_inline_feasibility`); orchestrator emits `ReplanRequest` (3A+)
- **Purpose of this file:** enough architecture to rebuild counterfactual safety questions on sensitive ops and the max-2 inline feasibility check during EXECUTING. Not a status ledger.

## 1. What it is

**Counterfactual Safety Questions (CSQ)** are structured checks generated for functions that touch files, network, money, or auth: “if [assumption] is wrong, [consequence] — was a guard added?” (`phase-2D.txt:6-8`, `phase-2D.txt:413-416`). **Inline feasibility** is a separate, cross-topic counter (max 2 checks per EXECUTING phase) that tests a **new** claim against PipelineSpec / Research Report / DKG sources and returns Pass / Fail / Blocked (`phase-2D.txt:52-75`, `phase-2D.txt:417-420`). After 2D the reasoning crate’s Phase-2 surface is complete (`phase-2D.txt:9`, `phase-2D.txt:409-410`).

## 2. Why it exists

A grounded plan can still emit a write/delete/auth call with no guard, or invent a new algorithm mid-EXECUTING. CSQ without severity would treat a wrong comment like data loss. Inline checks charged to the **topic** GRC budget would starve unrelated topics and would not bound the EXECUTING phase (`phase-2D.txt:40-57`, `phase-2D.txt:425`). Blocked without a MAJOR replan path lets the executor silently change the plan (`phase-2D.txt:67-75`).

## 3. How it works (runtime)

### CSQ (`generate_csq`)

1. **Who calls it.** Orchestrator at call time, injecting a `sensitive_ops` list (DKG/call-graph **optional**; reasoning must work **without** a `knowledge/` dependency) (`phase-2D.txt:24-38`). Intended production hook: pre-Write/Edit/Bash (`audit-implementation/phase-2D.md:193-226` — wiring is 3A+, not 2D).
2. **Input:** `function_id: &str`, `sensitive_ops: &[String]`. Receiver `&self` (`phase-2D.txt:136-139`).
3. **Empty ops:** `CsqResult { function_id, checks: vec![] }` — `passed()` is true (vacuous) (`phase-2D.txt:287-289`, T5).
4. **Per op (deterministic, case-insensitive: lowercase both sides)** (`phase-2D.txt:290-304`, `phase-2D.txt:342-344`):
   - **CRITICAL patterns:** `"fs::"`, `"file"`, `"write"`, `"delete"`, `"remove"`, `"net::"`, `"tcpstream"`, `"money"`, `"payment"`, `"auth"`, `"token"`, `"credential"`, `"password"`.
   - Else **WARNING**.
   - `assumption`: `"The call to {sensitive_op} in {function_id} behaves as expected"`
   - `consequence` Critical: `"data loss/security breach/financial error possible if {sensitive_op} fails"`
   - `consequence` Warning: `"incorrect output possible if {sensitive_op} fails"`
   - `guard_added: false` — caller must set true after adding guard code. No `Default` that sets true (`phase-2D.txt:160-162`).
5. **`CsqResult.passed()` (computed, not stored):** `checks.iter().all(|c| c.severity != Critical || c.guard_added)` (`phase-2D.txt:83-84`, `phase-2D.txt:325-329`).
6. **Fail-closed:** Critical + no guard → CSQ failure **blocks AUDITING** (`phase-2D.txt:45-46`, `phase-2D.txt:401-403`). Warning + no guard → log only, `passed()` still true (T4).
7. **Per-role mandate (reasoning.txt §7, cited):** executor = mandatory, auditor = mandatory, compiler = none (`phase-2D.txt:421`, `P2D-E021`). Enforced via 2C `csq_required` (validate: csq_required ⇒ ≥1 Verify) (`phase-2C.txt:46-47`).

The model may propose guards. It may not mark `guard_added=true` without the caller doing so.

### Inline feasibility (`check_inline_feasibility`)

1. **Who calls it.** EXECUTING-phase handler when GRC verify detects a **NEW** claim not in PipelineSpec / Research Report / DKG (`phase-2D.txt:52-54`, `phase-2D.txt:418`). 2D returns the outcome; orchestrator emits `ReplanRequest` (`phase-2D.txt:250-252`).
2. **Receiver `&mut self`** (mutates the dedicated counter). 2B `verify_claim` stays `&self` (`phase-2D.txt:62-65`).
3. **Counter:** `InlineFeasibilityState` **separate from per-topic GRC budget**, cross-topic (`phase-2D.txt:52-57`). `max_checks = 2` **hardcoded** in `Default`; not settable via public constructor (`phase-2D.txt:92-94`, `phase-2D.txt:163-165`).
4. **Logic** (`phase-2D.txt:306-323`):
   - If `is_exhausted()` → `Err(GrcError::BudgetExhausted { topic: "inline_feasibility", … })`. Exhausted new claims are also specified as treat-as-Fail (`phase-2D.txt:95`) — T8 asserts **Err** on the third call.
   - If claim contains `CONTRADICTS_PLAN:` prefix → `FeasibilityOutcome::Blocked` (2D heuristic; full semantic contradiction is 4F) (`phase-2D.txt:315-321`, T9).
   - Else if claim appears in any `sources` (substring or exact; adversarial: meaningful multi-word / >3 words, not `"the"`) → `Pass` (`phase-2D.txt:312-314`, `phase-2D.txt:345-347`).
   - Else → `Fail` (executor **must use the plan’s original approach**) (`phase-2D.txt:69`, `:314`).
   - Then `record_check()` (`saturating_add`).
   - Return `{ claim, method: FeasibilityMethod::SourceMatch, outcome }`.
5. **Blocked → replan:** executor emits `ReplanRequest(MAJOR)` → EXECUTING→PLANNING, human approval in **both** AUTONOMOUS and INTERACTIVE (`phase-2D.txt:70-73`). Emission is orchestrator 3A+ (`phase-2D.txt:250-252`, `P2D-E022`).
6. **`reset()`:** orchestrator calls between phases so one phase cannot exhaust the next (`phase-2D.txt:94`, `phase-2D.txt:351-353`, T10).

## 4. Types, enums, objects

Seven new types (`phase-2D.txt:111-135`, `phase-2D.txt:168-169`):

```
CsqSeverity { Critical, Warning }                 // Debug, Clone, Copy, PartialEq, Eq

CsqCheck {
  assumption: String,
  consequence: String,
  severity: CsqSeverity,
  guard_added: bool,
}

CsqResult { function_id: String, checks: Vec<CsqCheck> }
  fn passed(&self) -> bool   // NO stored passed field

InlineFeasibilityState { checks_used: u32, max_checks: u32 }
  Default: max_checks = 2
  is_exhausted() -> checks_used >= max_checks
  record_check() -> saturating_add(1)
  reset() -> checks_used = 0

FeasibilityOutcome { Pass, Fail, Blocked }

FeasibilityMethod { SourceMatch }
  // future (4F): WebSearch, Grep, ApiRegistryCheck  — NOT in 2D enum

InlineFeasibilityResult { claim: String, method: FeasibilityMethod, outcome: FeasibilityOutcome }
```

**GrcEngine +2 methods** (`phase-2D.txt:136-141`):

```
fn generate_csq(&self, function_id: &str, sensitive_ops: &[String]) -> Result<CsqResult, GrcError>
fn check_inline_feasibility(&mut self, claim: &str, sources: &[String]) -> Result<InlineFeasibilityResult, GrcError>
```

`BasicGrcEngine` holds `InlineFeasibilityState` (`phase-2D.txt:143`).

**D2 severity mapping** (names in prose, not extra enum variants) (`phase-2D.txt:40-46`):
- CRITICAL / guard MANDATORY: `data_loss`, `security_breach`, `financial_error`, `auth_bypass`
- WARNING / guard RECOMMENDED: `incorrect_output`, `performance_degradation`, `stale_cache`

Detection targets in D1: params/returns `Path`/`File`/`TcpStream`/`Money`/`AuthToken` or calls `std::fs`/`std::net` (`phase-2D.txt:26-28`). 2D implements this as **caller-supplied `sensitive_ops` strings** plus the pattern list in I1 — not a tree-sitter pass inside reasoning.

## 5. Control flow / state machine

CSQ is a function, not a GRC state. Inline feasibility is a **bounded side counter** during FSM state EXECUTING.

```
EXECUTING
  new claim not in sources?
    check_inline_feasibility
      Pass    → continue EXECUTING
      Fail    → stick to original plan approach (no deviation)
      Blocked → ReplanRequest(MAJOR) → EXECUTING → PLANNING (human gate)
      3rd check → BudgetExhausted topic="inline_feasibility"
CSQ Critical !guard → block AUDITING
CSQ Warning  !guard → log, do not block
```

**Owner:** `GrcEngine` for computation; orchestrator for hooks, AUDITING block, `reset()`, and ReplanRequest (`phase-2D.txt:70-73`, `:250-252`).

**Forced:** `max_checks` not caller-configurable (`phase-2D.txt:163-165`). `guard_added` starts false (`phase-2D.txt:302`).

## 6. Grounding, receipts, gates

Inline Pass is **source membership**, not 2B FILE_CHECK. Method in 2D is only `SourceMatch`. Spec §8 names methods `web_search`, `grep`, `api_registry_check` as the **eventual** set (`phase-2D.txt:58`); 2D enum has SourceMatch only and defers those variants to 4F (`phase-2D.txt:85-87`, `phase-2D.txt:132`, `P2D-E009`).

**Never allowed as a confidence signal:** LLM “this looks feasible”; Blocked via loose token-overlap without `CONTRADICTS_PLAN:` in 2D (`phase-2D.txt:315-321`); treating Warning like Critical; DKG required to even run CSQ (`phase-2D.txt:28-38`).

Adversarial: `"uses std::fs::write"` vs `"uses std::fs::read"` must **not** Blocked (`phase-2D.txt:348-350`).

## 7. Budgets, limits, ceilings

| Knob | Value |
|---|---|
| inline `max_checks` | **2** hardcoded |
| third check | `Err(BudgetExhausted)` topic `"inline_feasibility"` (T8) |
| CSQ overhead | `< 1000` µs |
| inline check overhead | `< 2000` µs |
| false_positive_rate | `0.0` on 20 synthetic claims (10 present, 10 absent); claims **>5 words** (`phase-2D.txt:366-391`) |
| deps | only `core`, `providers`, `tools`, `thiserror`, `serde` (`phase-2D.txt:177-190`) |

Do **not** debit the per-topic `GrcBudget` for inline checks (`phase-2D.txt:52-57`).

`BUDGET_EXCEEDED` as an **agent death cause that does not restart** is 3C, not 2D (`ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:256`).

## 8. Roles and neighbors

| Role | CSQ |
|---|---|
| executor | mandatory (`phase-2D.txt:421`) |
| auditor | mandatory |
| context-compiler | none |

**Consumes:** `sensitive_ops` from orchestrator; `sources` (PipelineSpec lines, DKG entries, research report). **Produces:** `CsqResult`, `InlineFeasibilityResult`. **Does not emit:** `ReplanRequest` (3A+). **Does not implement:** 4F `FEASIBILITY_PROBE` semantics.

**Must not fork:** a second feasibility engine inside the agent that ignores the max-2 counter; a DKG import from `reasoning/` (`phase-2D.txt:36-38`).

## 9. Config / flags / defaults

- `AgentGrcConfig.csq_required` (2C) gates whether the role must CSQ; 2D implements generation.
- `max_checks=2` not a TOML key.
- `CONTRADICTS_PLAN:` is a phase-2D **placeholder protocol**, UNGROUNDED as a semantic detector (`phase-2D.txt:428`).
- No Default on `CsqCheck` that sets `guard_added=true` (`phase-2D.txt:160-162`).

## 10. Implementation map

| Symbol | Spec |
|---|---|
| `generate_csq` / `check_inline_feasibility` | `engine.rs` |
| 7 types | `types.rs` |
| tests | `reasoning/tests/csq_tests.rs` ≥10 |
| metrics | `.elai_cc/metrics/phase_2D.toml` (`csq_overhead_us`, `inline_check_overhead_us`, `false_positive_rate`) |
| architecture-overview.txt §4 trait listing | must include the two methods (`phase-2D.txt:372-373`) |
| `PipelineGrc` wrapper | orchestrator neighbor (`audit-implementation/phase-2D.md:193-198`) |

## 11. Journal / SSB events

2D specifies no SSB row. Blocked **signals** MAJOR replan; 3A/3B journal `replan_request` (`phase-index-MASTER.txt:423`). Warning-without-guard is “logged” (`phase-2D.txt:46`) — **UNCERTAIN** whether that log is stderr, `ThinkResult.warnings`, or SSB.

## 12. Tests and observables

`reasoning/tests/csq_tests.rs` (`phase-2D.txt:199-257`):

| Id | Name | Assert |
|---|---|---|
| T1 | `csq_generates_check_for_file_function` | `generate_csq("write_config", &["std::fs::write"])` → `checks.len()>=1`, ≥1 Critical |
| T2 | `csq_critical_without_guard_fails` | Critical `guard_added=false` → `passed()==false` |
| T3 | `csq_critical_with_guard_passes` | Critical `guard_added=true` → `passed()==true` |
| T4 | `csq_warning_without_guard_still_passes` | Warning no guard → `passed == true` (T4 writes `passed` not `passed()`; method is `passed()`) |
| T5 | `csq_no_sensitive_ops_returns_empty` | empty ops → empty checks, `passed()==true` |
| T6 | `inline_feasibility_pass_on_known_claim` | multi-word claim present in sources → Pass |
| T7 | `inline_feasibility_fail_on_unknown_claim` | empty sources → Fail |
| T8 | `inline_feasibility_max_two_calls` | start 0; two checks → `checks_used==2`; third → `BudgetExhausted` |
| T9 | `inline_feasibility_blocked_signals_replan` | `"CONTRADICTS_PLAN: …"` → Blocked (no ReplanRequest assertion in 2D) |
| T10 | `inline_feasibility_state_reset` | after exhaust, `reset` → `checks_used==0`, not exhausted |

Each test owns its own `BasicGrcEngine` (`phase-2D.txt:269-270`). Zero timing tests. Exit: 2A–2D tests pass; zero `todo!`/`unwrap` in CSQ/inline methods (`phase-2D.txt:400-407`).

## 13. Rebuild contract

- Seven types with computed `passed()`, private/hardcoded `max_checks=2`, `FeasibilityMethod::{SourceMatch}` only.
- `generate_csq` + `check_inline_feasibility` on `GrcEngine`; engine holds `InlineFeasibilityState`.
- Critical pattern list **exact** (case-insensitive contains); `guard_added` starts false.
- Critical without guard fails CSQ and is specified to block AUDITING.
- Inline counter ≠ topic budget; third call `BudgetExhausted` with topic `inline_feasibility`; `reset` between phases.
- Pass / Fail / Blocked semantics as §3. Blocked **only** via `CONTRADICTS_PLAN:` in this phase.
- No `knowledge/` dependency. Sensitive signatures injected by caller.
- T1–T10 pass. Metrics file with the three named fields.
- ReplanRequest emission **out of this crate**.

A “model, please think if this is dangerous” prompt, a configurable max-checks knob, or semantic NLI Blocked in 2D is a cousin.

## 14. Non-goals / banned designs

- DKG as a compile-time dependency of `reasoning/` (`phase-2D.txt:28-38`).
- Public constructor that sets `max_checks = u32::MAX` (`phase-2D.txt:163-165`).
- `Default` `guard_added=true` (`phase-2D.txt:160-162`).
- Stored `passed` field that can desync (`phase-2D.txt:83-84`).
- Charging inline checks to per-topic `GrcBudget` (`phase-2D.txt:52-57`).
- Full semantic contradiction in 2D (deferred 4F) (`phase-2D.txt:318-321`).
- Adding WebSearch/Grep/ApiRegistryCheck to the 2D enum (`phase-2D.txt:132`).
- Silent truncate / ignoring the 3rd check (`phase-2D.txt:240`).
- Emitting ReplanRequest from `reasoning/` (`phase-2D.txt:250-252`).
- Case-sensitive-only pattern match (`phase-2D.txt:342-344`).
- Single-word substring Pass (`phase-2D.txt:345-347`).

## 15. Source citations

Primary: `docs/specs/guide/steps/phase-2D.txt`. Index: `phase-index-MASTER.txt:364-385`. Overview: `ELAI-SPEC-OVERVIEW.md:71-72`. Feature map: `ELAI-FEATURE-MAP.md:578-580`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:79-81`. Exhaustive: P2D-E001..E023. Audit: `audit-implementation/phase-2D.md` (production callers **none** — in-scope for 3A+).

## 16. Gaps

- T8 Err vs D5 “treat new claims as Fail” when exhausted (`phase-2D.txt:95` vs T8 `phase-2D.txt:240`) — tests pin **Err**.
- Warning “logged only” destination UNCERTAIN (`phase-2D.txt:46`).
- Who flips `guard_added` to true (caller outside reasoning; no API specified).
- How AUDITING is blocked from a `passed()==false` result (2D computes; 3A+ must honour — `audit-implementation/phase-2D.md:176-179`).
- `sources: &[String]` vs typed PipelineSpec/DKG objects — strings only in 2D.
- Whether `check_inline_feasibility` should prefer `CONTRADICTS_PLAN:` before source match when both could apply (I2 lists exhaust → match → fail → Blocked-by-prefix; implement order in I2 checks exhaust, then membership, then describes Blocked as prefix — T9 uses a claim that also wouldn’t match sources. **UNCERTAIN** if a claim both matches a source **and** has the prefix).
