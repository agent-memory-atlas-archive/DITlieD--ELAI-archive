# PipelineGrc Activation in Production Hot Paths (B-8)

- **ELAI ids:** B-8 / E-B8-001..E-B8-020 / SUM-0056..SUM-0061
- **Spec status (as designed):** WIRED (`ELAI-FEATURE-MAP.md:639-641`)
- **Crate / module (spec):** `crates/orchestrator/src/grc.rs` (`PipelineGrc`); call sites `task_runner.rs`, `dispatcher.rs`, `file_ops.rs` (pre-write), `evidence.rs` (`GroundingSweepGate`), `agent_pool.rs` (auditor `think`); `crates/reasoning` `InlineFeasibilityState::reset`
- **Purpose of this file:** enough architecture to rebuild the organ that **calls** Phase-2 GRC methods from production FSM/tool paths so CSQ, plan grounding, claim verify, inline feasibility, and anti-rubber-stamp actually fire. Not a status ledger.

## 1. What it is

**PipelineGrc activation** is the production-caller organ around an existing wrapper: `PipelineGrc::{verify_claim, ground_plan, generate_csq, check_inline_feasibility, think}` plus `InlineFeasibilityState::reset` (`B-8_pipelinegrc_activation.md:1-8`, `:82-90`, E-B8-010). The wrapper already exists at `orchestrator/src/grc.rs`; the failure mode is **zero production callers**, so `GroundingSweepGate` (registered at `evidence.rs:471`) vacuously passes because `ctx.ungrounded_claims` stays empty (`B-8_pipelinegrc_activation.md:13-22`). B-5 surfaces the resulting events; B-8 is what makes them true (`B-8_pipelinegrc_activation.md:7-8`, `:101`). Feature-map kill switch: `ELAI_DISABLE_PIPELINE_GRC` (`ELAI-FEATURE-MAP.md:639-641`).

## 2. Why it exists

A clean `PipelineGrc` that nobody calls is theater: the model can emit ungrounded plans, unguarded `delete_file`, novel crates mid-EXECUTING, and auditor `Converge` with zero `FINDING:` (`B-8_pipelinegrc_activation.md:13-18`, `ELAI-SPEC-OVERVIEW.md:66-72`). If activation is faked (tests-only `rg` hits under `*test*`), the gate list still “passes”. If `ground_plan` is skipped, B-17/B-38 have nothing to score (`B-38_ambiguity_score_gate.md:8-10`, `B-PLAN-SUMMARY.md:207-215`).

## 3. How it works (runtime)

Six production wires. The model may generate plans, claims, function signatures, FINDING lines. It may **not** skip a wire or mark a Critical CSQ passed.

### Wire 1 — `ground_plan` at PLANNING → TESTING_FIRST

1. **Who.** `task_runner` / evidence path, **immediately before** `GroundingSweepGate` evaluates (`B-8_pipelinegrc_activation.md:36-39`, E-B8-002).
2. **Call.** `pipeline_grc.ground_plan(plan)` (2B `PlanGroundingResult`).
3. **Deterministic write.** Result populates `ctx.ungrounded_claims` so the registered gate is non-vacuous (`:38-39`, `:20-22`).
4. **Fail-closed.** Any UNGROUNDED → gate BLOCK (2B `all_grounded == false` blocks PLANNING→EXECUTING; this transition is the B-8 insertion named PLANNING→TESTING_FIRST) (`02B-grounding-verification.md` §3 step 8, E-B8-015).
5. **Journal.** B-5 `grounding_sweep_result` (neighbor).

Whether the same call also runs on **every replan** is an open decision (`B-8_pipelinegrc_activation.md:148-149`, E-B8-020) — see §16.

### Wire 2 — `verify_claim` in ContextCompiler pre-fetch

1. **Who.** ContextCompiler, before retrieving context for a phase (`B-8_pipelinegrc_activation.md:41-44`, E-B8-003).
2. **Per claim** in the phase plan: `pipeline_grc.verify_claim(...)`.
3. **Deterministic skip.** Claims that fail `FILE_CHECK` / `SOURCE_CHECK` are **not loaded** (token save). They are not silently marked grounded.

### Wire 3 — `generate_csq` pre-tool (Write / Edit / Bash)

1. **Who.** `dispatcher.rs` (or the pre-write hook in `file_ops.rs`) **before** the call reaches the tool layer (`B-8_pipelinegrc_activation.md:46-48`, `:136-137`, E-B8-004).
2. **Call.** `pipeline_grc.generate_csq(function_signature, function_purpose)` (`:49`). 2D’s engine signature is `generate_csq(function_id, sensitive_ops)` (`02D-csq-inline-feasibility.md` §3) — B-8 names the **call-site arguments**; mapping onto 2D is a rebuild obligation (see §16).
3. **Outcomes (deterministic)** (`:50-55`, E-B8-005):
   - `severity == CRITICAL && !guard_added` → **BLOCK** the tool call; emit SSB `csq_evaluated`; surface B-5 intervention card.
   - WARNING → log to SSB, **proceed**.
   - passed → proceed silently.
4. **2D composition.** `CsqResult.passed()` is `all(severity != Critical || guard_added)` (`02D-csq-inline-feasibility.md` §3). Critical-without-guard also blocks AUDITING in 2D; B-8 additionally blocks the **tool dispatch**.

### Wire 4 — `check_inline_feasibility` in EXECUTING

1. **Who.** EXECUTING handler when the executor produces a **novel** claim not in the plan (examples: `"uses crate X"`, `"calls API Y"`) (`B-8_pipelinegrc_activation.md:57-59`, E-B8-006).
2. **Outcomes:** Pass → proceed; Fail → **use original plan approach**; Blocked → emit `ReplanRequest(MAJOR)` (`:61-63`).
3. **Per-baby-step conversion.** `FeasibilityOutcome::Blocked` is already returned by 2D; B-8 **must convert it** to `ReplanRequest` at the EXECUTING-state handler — the return value alone is not enough (`:65-67`, E-B8-007).
4. **2D counter.** Max **2** inline checks per EXECUTING phase, separate from per-topic GRC budget; third call `BudgetExhausted` / treat-as-Fail (`02D-csq-inline-feasibility.md` §3, `:40-46`).

### Wire 5 — Auditor `think` + anti-rubber-stamp

1. **Who.** Auditor flow via `agent_pool.rs` (`B-8_pipelinegrc_activation.md:69-77`, `:139`, E-B8-008).
2. Spawn auditor → auditor calls `pipeline_grc.think()` with `FINDING:`-prefixed **Verify** steps → engine anti-rubber-stamp (`engine.rs:297-315` as cited; 2C `FINDING_PREFIX`, `InsufficientFindings`) → rejected Converge emits B-5 `auditor_rubber_stamp_rejected`.
3. **Fail-closed.** Auditor must **invoke** `think` in production; config-only anti-rubber-stamp is not this organ (`:70-71`).

### Wire 6 — `InlineFeasibilityState.reset()` at phase boundary

1. **Who.** FSM phase transition (`:79-80`, E-B8-009).
2. **Deterministic.** `reset()` zeros `checks_used` so one phase cannot exhaust the next (`02D-csq-inline-feasibility.md` T10). Currently never called between phases — that is the bug this wire fixes (`B-8_pipelinegrc_activation.md:79-80`).

**Reachability:** all activation reachable from `task_runner.rs::drive_task` (`:93-95`, E-B8-011). **Kill switch:** if `ELAI_DISABLE_PIPELINE_GRC` is truthy, seed/bypass short-circuits (default-OFF = GRC **runs**) (`ELAI-FEATURE-MAP.md:639-641`, `wire-dark-allowlist.toml:475-487`). Original B-8 plan does not name the env var — see §16.

## 4. Types, enums, objects

B-8 introduces **no new reasoning types**. It binds existing ones:

```
PipelineGrc {
  ground_plan(...)
  verify_claim(...)
  generate_csq(...)
  check_inline_feasibility(...)
  think(...)                    // GRC step + anti-rubber-stamp
}

// construction named by later B-steps as prerequisite
PipelineGrc::new_from_default()           // B-39 / B-PLAN-SUMMARY.md:207-208
TaskRunnerConfig::with_pipeline_grc()     // B-PLAN-SUMMARY.md:207-208

InlineFeasibilityState::reset()           // reasoning/src/types.rs

ctx.ungrounded_claims                     // evidence context; gate input
GroundingSweepGate                        // evidence.rs:191-218, :471

ReplanRequest(MAJOR)                      // 2D / 3C channel
FeasibilityOutcome { Pass, Fail, Blocked }
CsqSeverity { Critical, Warning }
```

2B `PlanGroundingResult`, 2D `CsqResult` / `InlineFeasibilityResult` are the return shapes (`02B-grounding-verification.md` §4, `02D-csq-inline-feasibility.md` §4).

## 5. Control flow / state machine

**Owner of the loop:** orchestrator FSM + dispatcher, **not** the model.

| FSM / site | Method | On fail |
|---|---|---|
| PLANNING → TESTING_FIRST (pre-gate) | `ground_plan` | UNGROUNDED → BLOCK transition |
| ContextCompiler pre-fetch | `verify_claim` | skip bogus deps (do not load) |
| pre-Write/Edit/Bash | `generate_csq` | Critical∧¬guard → BLOCK tool |
| EXECUTING novel claim | `check_inline_feasibility` | Fail→original plan; Blocked→MAJOR replan |
| AUDITING auditor spawn | `think` + FINDING | Converge without FINDING rejected |
| any FSM phase boundary | `reset()` | (no fail; clears counter) |

**Forced:** `rg` of production (non-test) `pipeline_grc.(ground_plan|verify_claim|generate_csq|check_inline_feasibility|think)` ≥ **4** hits, one per required activation method (`B-8_pipelinegrc_activation.md:110-114`, E-B8-014). `think` is the fifth method in the produces list; the grep expects ≥4 — rebuild **all five** plus `reset`.

## 6. Grounding, receipts, gates

Activation **feeds** `GroundingSweepGate`; it does not replace 2B binary verify.

**Never allowed as a confidence signal:** empty `ungrounded_claims` as proof the plan is grounded; test-module callers as proof of production wiring; LLM saying a file exists.

CSQ WARNING is **not** a pass token for Critical. `guard_added` is set by the **caller** after adding guard code, never default-true (`02D-csq-inline-feasibility.md` §3).

B-38 later **extends** this same gate with a continuous score; B-8 must still BLOCK on `ungrounded_count > 0` first (`B-38_ambiguity_score_gate.md:136-137`). Ambiguity readiness does not substitute for this wire (`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` B-001/X-006).

## 7. Budgets, limits, ceilings

| Knob | Value | Owner |
|---|---|---|
| inline feasibility max checks | **2** per EXECUTING phase | 2D, reset by B-8 |
| ReplanRequest | max 1 per agent per phase (3C) | neighbor |
| production `rg` hits | ≥4 (one per method in the grep) | B-8 done-gate |
| kill switch | `ELAI_DISABLE_PIPELINE_GRC` truthy disables | feature-map / allowlist |
| 2A–2D suites | no regressions | acceptance |

GRC numeric budgets stay per-role 2C/4D; B-8 does not change 5/800 defaults.

## 8. Roles and neighbors

**Consumes:** `GrcEngine` 2A–2D methods; `GroundingSweepGate`; FSM transitions; dispatcher tool routing.

**Produces:** populated `ctx.ungrounded_claims`; blocked tools; `ReplanRequest(MAJOR)`; live auditor `think`; phase-local feasibility counter.

**Must not fork:** a second in-process GRC that the orchestrator does not count after 4A (`04A-orchestrator-authoritative-grc-budget.md` §1); prompt-only “please call CSQ”.

**Frontend pair:** B-5 (`B-8_pipelinegrc_activation.md:101`). **Downstream:** B-17 gate liveness, B-38 ambiguity, B-39 `PipelineGrc::new_from_default()` prerequisite (`B-39_chat_reflect_pipeline.md:8-9`).

## 9. Config / flags / defaults

- **`ELAI_DISABLE_PIPELINE_GRC`:** kill_switch; default-OFF means GRC **runs**; truthy (`1` / `true`) short-circuits seed/bypass (`ELAI-FEATURE-MAP.md:640-641`, `wire-dark-allowlist.toml:475-487`). Smoke: `safety_gate_integration.rs::kill_switch_env_var_disables_pipeline_grc` / `pipeline_grc_disabled_via_env`.
- Original B-8 has **no** named flag; the disable env is a later lock (`wire-dark-allowlist.toml:475-479`). Rebuild: default is **on**.
- Decision doc: `workflow/reasoning/B-8-decisions.md` (`B-8_pipelinegrc_activation.md:141`). Open questions in that doc are **not** resolved by this organ’s body (§16).

## 10. Implementation map

| Symbol | Spec |
|---|---|
| `orchestrator/src/grc.rs` `PipelineGrc::{ground_plan,verify_claim,generate_csq,check_inline_feasibility,think}` | `:82-90` |
| `reasoning/src/types.rs` `InlineFeasibilityState::reset` | `:89` |
| `evidence.rs` `GroundingSweepGate` | `:92`, `:29` |
| `task_runner.rs` `drive_task` (4 wiring points) | `:93-95`, `:135` |
| `dispatcher.rs` CSQ pre-tool | `:136` |
| `file_ops.rs` pre-write (or wherever pre-write lives) | `:137` |
| `agent_pool.rs` auditor invokes engine | `:139` |
| `tests/contracts/safety_gate_integration.rs` | `:141` |
| `PipelineGrc::new_from_default` / `TaskRunnerConfig::with_pipeline_grc` | `B-PLAN-SUMMARY.md:207-208` |

stub_ban: those four methods `!= TODO`; `think != TODO` (`B-8_pipelinegrc_activation.md:96-100`).

## 11. Journal / SSB events

B-8 does not add new `action_type`s. It **causes** B-5 rows:

- `csq_evaluated` on CSQ eval (required smoke) (`:102-104`)
- `grounding_sweep_result` once `ground_plan` fills claims
- `feasibility_check` / `FeasibilityReport` on inline check
- `auditor_rubber_stamp_rejected` on rejected Converge

Observable test name: `generate_csq_critical_without_guard_blocks_when_kill_switch_off` (`:103`). Expect dashboard red intervention confirm/cancel (`:105-106`).

## 12. Tests and observables

| Id | Assert |
|---|---|
| D1 | production `rg pipeline_grc.(ground_plan\|verify_claim\|generate_csq\|check_inline_feasibility\|think)` `--glob '!*test*'` ≥ 4 |
| D2 | intentionally ungrounded claim → `GroundingSweepGate` **BLOCKS** |
| D3 | delete-files task → CSQ guard required |
| Named | `safety_gate_integration` CSQ Critical-without-guard + `csq_evaluated` + red card |
| Regression | 2A–2D test suites green |

Acceptance (`:123-131`, E-B8-017): all **4** PipelineGrc methods have production callers (produces list has **5** including `think` — rebuild all); gate sees real ungrounded_claims; CSQ Critical-without-guard blocks e2e; Blocked → ReplanRequest; anti-rubber-stamp live; `reset` per phase; no 2A–2D regressions.

## 13. Rebuild contract

Another harness has **this** organ iff:

- `ground_plan` runs immediately before GroundingSweepGate on PLANNING→TESTING_FIRST and **writes** `ctx.ungrounded_claims`.
- ContextCompiler `verify_claim`s phase-plan claims and skips FILE_CHECK/SOURCE_CHECK failures at retrieval.
- Write/Edit/Bash cannot reach tools until `generate_csq`; Critical∧¬guard blocks + journals + intervention.
- EXECUTING novel claims hit `check_inline_feasibility`; Blocked becomes `ReplanRequest(MAJOR)` per baby-step.
- Auditor production path calls `think` with FINDING-prefixed Verify; rubber-stamp reject is visible.
- `InlineFeasibilityState.reset()` on every FSM phase transition.
- Methods are real function calls, not TODOs, reachable from `drive_task`.
- Default is GRC **on**; a disable kill switch must not be the implicit production path.

A unit-tested wrapper with zero `drive_task` callers is a cousin.

## 14. Non-goals / banned designs

- Implementing 2A–2D algorithms (`B-8_pipelinegrc_activation.md:13-16` — wrappers already exist).
- Vacuous gate: registered `GroundingSweepGate` + empty `ungrounded_claims` as “pass”.
- Prompt-only CSQ / “please add a guard”.
- Charging inline feasibility to per-topic GRC budget (2D forbids; B-8 must keep `reset` on the **dedicated** counter).
- Skipping auditor `think` because `AgentGrcConfig` already lists FINDING (`:70-71`).
- Treating B-5 dashboard as a substitute for production callers.

## 15. Source citations

Primary: `docs/thefinalglance/B-plan/B-8_pipelinegrc_activation.md` (full file). Index: `B-PLAN-SUMMARY.md:207-215`, `:973`. Feature map: `ELAI-FEATURE-MAP.md:639-641`. Exhaustive: `ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` E-B8-001..E-B8-020, SUM-0056..SUM-0061. Engine: `02A-grc-state-machine.md`, `02B-grounding-verification.md`, `02D-csq-inline-feasibility.md`, `02C-per-agent-grc-config.md` (FINDING). Flag lock: `.elai_cc/wire-dark-allowlist.toml:475-487`. Overview: `ELAI-SPEC-OVERVIEW.md:66-72`. Producer pair: `B-5_reasoning_producer_wiring.md`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- CSQ when user is absent (autonomous): block+webhook vs safer fallback (`B-8_pipelinegrc_activation.md:144-147`, E-B8-019).
- `ground_plan` only at PLANNING→TESTING_FIRST vs every replan (`:148-149`, E-B8-020).
- `generate_csq(function_signature, function_purpose)` vs 2D `generate_csq(function_id, sensitive_ops)` — argument mapping unspecified in B-8.
- PLANNING→TESTING_FIRST (B-8) vs PLANNING→EXECUTING (2B header). Rebuild **both** intents: sweep before leaving PLANNING; do not invent a third transition.
- Grep “≥4 hits” vs five methods including `think` (`:114` vs `:82-90`).
- `ELAI_DISABLE_PIPELINE_GRC` not in the original B-8 task list; named in feature-map + allowlist only.
- Exact `PipelineGrc` struct fields / whether it owns one `BasicGrcEngine` or a map per agent (`grc.rs` “find it” — `:28`).
- `new_from_default()` / `with_pipeline_grc()` are B-PLAN-SUMMARY / B-39 names, not B-8 body.
- Pre-write hook file: `dispatcher.rs` vs `file_ops.rs` “or wherever” (`:46-47`, `:136-137`).
