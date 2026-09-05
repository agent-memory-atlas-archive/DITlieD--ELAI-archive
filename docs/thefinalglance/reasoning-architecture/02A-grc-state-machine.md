# GRC State Machine (Phase 2A)

- **ELAI ids:** phase-2A / P2A-E001..P2A-E033 / IDX-E016
- **Spec status (as designed):** WIRED
- **Crate / module (spec):** `crates/reasoning` (`reasoning/src/types.rs`, `reasoning/src/engine.rs`); orchestrator later wraps as `PipelineGrc`
- **Purpose of this file:** enough architecture to rebuild the Grounded Reasoning Chain engine core — states, per-topic budget, format-only grounding, typed rejections. Not a status ledger.

## 1. What it is

The **Grounded Reasoning Chain (GRC) state machine** is a code-driven, per-topic reasoning loop. A `ThinkStep` must walk `Observe → Hypothesize → PlanAction → Act → Verify`, with `Converge` and `Status` reachable from any state. Every step that the spec marks as requiring grounding carries a `type:reference` citation string. A `GrcBudget` caps calls and characters per topic and forces a `Converge` when `remaining_calls <= 1`. Phase 2A validates **format and transitions only**; it does not check files, tests, or the web (`phase-2A.txt:11-13`, `phase-2A.txt:46-50`).

## 2. Why it exists

ELAI’s trust spine refuses uncited reasoning. If this organ is missing, an agent can skip Observe, invent a conclusion, or run until the model window is full. If it is faked (prompt instructions instead of `GrcEngine::think` rejections), invalid chains pass silently (`phase-2A.txt:5-9`, `phase-2A.txt:21-22`, `ELAI-SPEC-OVERVIEW.md:66-68`). Budget-forced `Converge` is the bound that later isolation (4A) still counts, only from the other side of IPC (`phase-2A.txt:39-44`, `phase-4A.txt:26-27`).

## 3. How it works (runtime)

1. **Who calls it.** The orchestrator (in-process through Phase 3; via `ThinkStep` IPC after 4A) calls `GrcEngine::think(&mut self, step: ThinkStep)` (`phase-2A.txt:101-106`, `phase-4A.txt:159-166`).
2. **Input.** One `ThinkStep` with `step_type`, `content`, `topic`, optional `grounding` / `conclusion` / `scope` (`phase-2A.txt:77-79`).
3. **Topic lookup.** `BasicGrcEngine` holds `HashMap<String, TopicState>`. First `think` on an unseen topic applies default budget `max_calls=5`, `max_chars=800` unless `set_budget` already ran (`phase-2A.txt:117-120`, `phase-2A.txt:211-215`).
4. **Status short-circuit (deterministic).** `Status` is allowed from any state, consumes **no** budget, requires **no** grounding, and **does not** update `last_state` (`phase-2A.txt:207-209`, `phase-2A.txt:270-271`, `phase-2A.txt:285-285`).
5. **Transition check (deterministic).** If `last_state` is `None`, the first non-Status/non-Converge step must be `Observe`; else `step.step_type` must be in `last_state.allowed_next()` (plus 2C whitelist when config is set). Else `Err(InvalidTransition)` (`phase-2A.txt:265-271`, `phase-2A.txt:217-220`).
6. **Grounding parse (deterministic).** For `Observe`, `Hypothesize`, `Verify`: `grounding=None` → `UngroundedStep`. `Some(s)` is comma-separated `type:reference`. Valid types: `file`, `grep`, `test`, `search`, `tool`, `training`. Else `MalformedGrounding`. Classify: `test`/`search` → `Verified`; `file`/`grep`/`tool` → `High`; `training` → `Uncertain` (warn log, do **not** reject in 2A) (`phase-2A.txt:273-283`).
7. **Scope (deterministic, config-gated).** When `AgentGrcConfig.require_scope_enumeration=true`, Observe must populate `scope`. Orchestrator later checks every scope item appears in a subsequent `PlanAction`; unaddressed items → `WARN` in `ThinkResult.warnings` (`phase-2A.txt:81-84`, `phase-2A.txt:222-237`). Config itself is Phase 2C (`phase-2A.txt:11-13`).
8. **Budget check-then-increment (deterministic, all non-Status).** Before increment: if `calls_used >= max_calls` → `BudgetExhausted`; if `chars_used + content.len() > max_chars` → `BudgetExhausted`; if `must_converge()` (`remaining_calls <= 1`) and `step_type != Converge` → `BudgetExhausted`. Then `saturating_add` on `calls_used` and `chars_used` (`phase-2A.txt:287-298`).
9. **Commit.** Return `ThinkResult { step, budget_remaining, grounding_level, warnings }`. Persist the step on `TopicState.steps` and `grounding_log` (`phase-2A.txt:305-307`, `phase-2A.txt:117-119`).
10. **Abstain / fail-closed.** There is no “soft pass” for missing grounding on Observe/Hypothesize/Verify, malformed citations, illegal transitions, or budget overflow. Uncertain citations warn; they do not pass as Verified (`phase-2A.txt:54-56`, `phase-2A.txt:357-364`).
11. **Artifact.** In 2A the artifact is in-memory `TopicState` plus the `ThinkResult`. SSB `reasoning_trace` is written by the orchestrator after a GRC step, not by `reasoning/` (`phase-index-MASTER.txt:456-458`, `phase-index-MASTER.txt:436-438`).

The model may **generate** `content`, `grounding` strings, `conclusion`, and `scope` names. It may **not** decide whether a transition is legal, whether budget remains, or whether a string is well-formed.

## 4. Types, enums, objects

Copy from `phase-2A.txt:71-100`. Do not invent fields.

```
GrcState { Observe, Hypothesize, PlanAction, Act, Verify, Converge, Status }
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  fn allowed_next(&self) -> &[GrcState]
  Clone required because InvalidTransition.allowed: Vec<GrcState>

ThinkStep {
  step_type: GrcState,
  content: String,
  topic: String,
  grounding: Option<String>,
  conclusion: Option<String>,
  scope: Option<Vec<String>>,
}

ThinkResult {
  step: ThinkStep,
  budget_remaining: GrcBudget,
  grounding_level: Option<GroundingLevel>,
  warnings: Vec<String>,
}

GrcBudget {
  max_calls: u32,
  max_chars: u32,
  calls_used: u32,
  chars_used: u32,
}
  remaining_calls() -> u32
  remaining_chars() -> u32
  is_exhausted() -> bool
  must_converge() -> bool   // remaining_calls <= 1
  Default: max_calls=5, max_chars=800   // reasoning.txt §3 "moderate"

GroundingLevel { ... }   // see ordering invariant below
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]

GrcError {
  UngroundedStep { step_type, topic },
  BudgetExhausted { topic, calls_used, max_calls },
  InvalidTransition { from: GrcState, to: GrcState, allowed: Vec<GrcState> },
  MalformedGrounding { raw, reason },
}
```

**2A forward stubs** (replaced in 2B; keep names): `VerificationResult { claim, passes, evidence, method_used: String }`, `GroundingResult { claim, method: String, level, source }` (`phase-2A.txt:113-116`).

**`TopicState`:** `{ budget, last_state: Option<GrcState>, steps: Vec<ThinkStep>, grounding_log: Vec<(String, GroundingLevel)> }` (`phase-2A.txt:117-119`).

**`GrcEngine` trait — 2A surface (6 methods):** `think`, `set_budget`, `get_budget`, `reset_topic` implemented; `verify_claim`, `ground_plan` as `unimplemented!()` for trait stability (`phase-2A.txt:101-112`). Later phases add methods; that is additive, not a 2A cousin.

**GroundingLevel ordering (load-bearing).** Semantic hierarchy is `VERIFIED > HIGH > UNCERTAIN` (`phase-2A.txt:54-55`). Phase 2A’s example declaration `{ Verified, High, Uncertain }` **inverts** derived `PartialOrd`. Phase 2C requires declaration `{ Uncertain, High, Verified }` (or an explicit `PartialOrd`) so `Uncertain < High < Verified` (`phase-2C.txt:337-347`, `phase-index-MASTER.txt:283-284`). A rebuild that derives `PartialOrd` from the 2A example order is wrong.

**`InvalidTransition.from` vs T9.** The enum sketch types `from: GrcState`, but T9 requires a `from=None` indication when the first step is not Observe (`phase-2A.txt:96-98`, `phase-2A.txt:217-220`). **UNCERTAIN in the 2A sketch; T9 wins:** `from: Option<GrcState>` is the shape that satisfies the named test (`audit-implementation/phase-2A.md:24`, `P2A-A003`).

**Numeric range (not the default):** spec grounding summary allows `max_calls` 1–12 and `max_chars` 200–1500 (`phase-2A.txt:369-370`). Default remains 5/800.

## 5. Control flow / state machine

**Owner of the loop:** `BasicGrcEngine` per topic. The model proposes a step; the engine accepts or returns `GrcError`. No token-level FSM (`phase-2A.txt:26-32`).

**Default chain (must reject skips):** `Observe → Hypothesize → PlanAction → Act → Verify` (`phase-2A.txt:27-28`, `phase-2A.txt:295-296`).

**Forced transitions:**
- First non-Status/non-Converge step in a topic = `Observe` (`phase-2A.txt:76`, `phase-2A.txt:269`).
- `remaining_calls <= 1` → only `Converge` is accepted (`phase-2A.txt:41-42`, `phase-2A.txt:292-293`).
- `Converge` allowed from any state (`phase-2A.txt:75`).
- `Status` allowed from any state; does not update `last_state` (`phase-2A.txt:75`, `phase-2A.txt:270`).

**`allowed_next()` table — what the spec actually pins:**

| From | Must include | Must not include | Spec source |
|---|---|---|---|
| Observe | Hypothesize | Verify | T4 `allowed:[Hypothesize]`; adversarial Observe↛Verify (`phase-2A.txt:136-138`, `phase-2A.txt:193-197`) |
| Hypothesize | PlanAction | Verify (skip) | D1 example hypothesize↛verify (`phase-2A.txt:27-28`) |
| PlanAction | Act | — | default chain (`phase-2A.txt:27`, `phase-2A.txt:295`) |
| Act | Verify | — | default chain |
| Verify | at least Observe **or** documented terminal | empty-and-undocumented | adversarial (`phase-2A.txt:317-319`) |
| any | Converge, Status (as *allowed-from*, not necessarily listed in `allowed_next` return) | — | `phase-2A.txt:75-76` |

**UNCERTAIN:** the full `allowed_next()` vector for every variant, including whether `Converge`/`Status` appear inside `allowed_next()` or are special-cased in `think()`; whether `Verify` may go to `Hypothesize`; whether T4’s `allowed:[Hypothesize]` omits always-allowed Status/Converge from the error payload (`P2A-A001`). One audited tree filled Verify → `[Observe, Hypothesize, Converge, Status]` (`audit-implementation/phase-2A.md:19`, `:180`) — that fill is **not** in the 2A phase file.

**`reset_topic`:** remove the map entry entirely; do not zero fields (`phase-2A.txt:300-303`, `phase-2A.txt:322-323`).

## 6. Grounding, receipts, gates

Phase 2A is **format-only**. A well-formed `training:…` citation is accepted at `Uncertain` with a warning (`phase-2A.txt:50`, `phase-2A.txt:279-283`). There is no receipt, no file read, no test run.

**Selector hierarchy (2A classification, not 2B verification):**

| type | GroundingLevel |
|---|---|
| `test`, `search` | Verified |
| `file`, `grep`, `tool` | High |
| `training` | Uncertain |

Rationale pinned: `"search" = web-verified`; `"tool" = tool output this session ≡ file/grep = HIGH` (`phase-2A.txt:280-282`).

**Never allowed as a confidence signal:** LLM self-rating, scalar 0–1 scores, “the model said it looked it up”, window fill, token-budget remaining as a quality score. 2A has no confidence field at all (`phase-2A.txt:13`, MiniCheck binary deferred to 2B).

**Header vs body conflict:** the grounding summary line “Grounding required on every step” (`phase-2A.txt:371`) is **false against I2**. Body: required on Observe/Hypothesize/Verify; optional on PlanAction/Act; none on Status (`phase-2A.txt:273-285`, `P2A-C001`). Rebuild the body.

**Converge grounding: UNCERTAIN.** I2 does not name Converge. Do not invent a requirement.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| default `max_calls` | 5 | per topic |
| default `max_chars` | 800 | per topic |
| documented range | calls 1–12, chars 200–1500 | reasoning.txt §3 (`phase-2A.txt:369-370`) |
| `must_converge` | `remaining_calls <= 1` | per topic |
| Status | 0 calls, 0 chars | per step |
| validation overhead target | `< 5000` µs | `.elai_cc/metrics/phase_2A.toml` |
| memory per topic | `< 1KB` base + history | same metrics file |
| integers | `u32` + `saturating_add` / remaining clamps to 0 | no signed underflow (`phase-2A.txt:138-139`) |

**Overflow:** `BudgetExhausted` **before** increment so `get_budget` never shows `calls_used = max_calls+1` (`phase-2A.txt:296-298`). T2 uses `max_calls=3` (not 2) so forced-converge does not mask overflow (`phase-2A.txt:179-189`).

**Authority through Phase 3:** this in-process engine **is** the authority (`phase-2C.txt:373-380`). After 4A the same `GrcBudget` struct is **advisory** on the agent; the orchestrator counts `ThinkStep` messages (`phase-4A.txt:26-27`). Do not rebuild 2A as if 4A already demoted it.

**Do not confuse with:** context-window slots (3E), prompt-transform “moderate=5/1000” in other docs (audit: engine Default is 800 following reasoning.txt §3) (`audit-implementation/phase-2A.md:147-156`).

## 8. Roles and neighbors

2A is role-agnostic. `AgentGrcConfig` is named only for `require_scope_enumeration` (`phase-2A.txt:81-84`). Roles, method allowlists, and anti-rubber-stamp are 2C. Verification methods are 2B. CSQ is 2D. Isolation budget counting is 4A. GrcHistory context slot is 3E.

**Must not fork:** a second GRC engine inside the agent after 4A that can overrule orchestrator counts; a prompt-only “please follow Observe then Hypothesize” cousin; MCTS as a replacement for `allowed_next`.

**Consumes:** `ThinkStep` from the caller. **Produces:** `ThinkResult` or `GrcError`. **Does not produce:** SSB rows (orchestrator), IPC frames (4A), verification evidence (2B).

## 9. Config / flags / defaults

- Default budget applied on first `think` if `set_budget` was never called (`phase-2A.txt:120`, T8).
- `set_budget(topic, max_calls, max_chars)` overrides per topic (`phase-2A.txt:104`).
- No dashboard panel in 2A (`phase-2A.txt:373`).
- No feature flag named in 2A. Code-driven, not prompt-driven (`phase-2A.txt:21`).
- Crate deps 2A-only: `core`, `thiserror`, `serde`. No `providers/`, `tools/`, `knowledge/` (`phase-2A.txt:152-164`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/reasoning` workspace member | `phase-2A.txt:69-70` |
| `reasoning/src/types.rs` | all types above |
| `reasoning/src/engine.rs` `trait GrcEngine` + `BasicGrcEngine` | `phase-2A.txt:101-120` |
| `think` / `set_budget` / `get_budget` / `reset_topic` | 2A implemented |
| `verify_claim` / `ground_plan` | 2A stubs; 2B real |
| tests: `reasoning/tests/` (audited file `grc_engine.rs`) | `phase-2A.txt:172` |
| metrics: `.elai_cc/metrics/phase_2A.toml` field `validation_overhead_us` | `phase-2A.txt:336-342` |
| orchestrator `PipelineGrc` / `AgentHandle.grc_engine` | implementation neighbor, not 2A skeleton (`audit-implementation/phase-2A.md:163-171`) |

IPC message also named `ThinkStep` appears in the 15-message 4A enum (`phase-4A.txt:55-59`). **UNCERTAIN** whether that wire type is the same Rust struct as `reasoning::ThinkStep` or a distinct postcard type. Rebuild: orchestrator **counts messages named ThinkStep**; do not assume byte identity with `types.rs`.

## 11. Journal / SSB events

2A specifies **none**. Orchestrator later writes `reasoning_trace` per GRC step (`phase-index-MASTER.txt:422`, `:456`). `reasoning/` must not depend on SSB (`phase-index-MASTER.txt:436-438`).

## 12. Tests and observables

Named tests, all TDD-red against stubs (`phase-2A.txt:174-237`):

| Id | Name | Assert |
|---|---|---|
| T1 | `observe_requires_grounding` | `think(Observe, grounding=None)` → `UngroundedStep` |
| T2 | `budget_overflow_forces_convergence` | `max_calls=3`; two non-Converge; third non-Converge → `BudgetExhausted`; Converge ok; fourth → `BudgetExhausted` |
| T3 | `per_topic_budget_independence` | exhaust `"a"`; `"b"` still `remaining_calls==3` |
| T4 | `invalid_transition_rejected` | Observe then Verify → `InvalidTransition{from:Observe, to:Verify, allowed:[Hypothesize]}` |
| T5 | `must_converge_at_budget_limit` | after 2 of 3, `must_converge()==true`; next must be Converge |
| T6 | `malformed_grounding_rejected` | `"no_colon_no_type"` → `MalformedGrounding` |
| T7 | `status_step_free` | `think(Status)` → `calls_used==0` |
| T8 | `default_budget_applied_on_first_think` | no `set_budget`; Observe ok; `max_calls==5` |
| T9 | `first_step_must_be_observe` | first Hypothesize → `InvalidTransition` with from=None indication; Status/Converge may be first |
| T10 | `scope_enumeration_present_when_required` | scope `[ui,api,tests]`, PlanAction omits tests → warnings contain WARN for `"tests"` |
| T11 | `scope_enumeration_missing_when_required` | Observe `scope=None` with flag true → **Err or ThinkResult with warning** (`P2A-A002` — hard vs soft **UNCERTAIN**) |
| T12 | `scope_enumeration_optional_when_not_required` | flag false, `scope=None` → ok, no scope warnings |

Done-gates (`phase-2A.txt:357-365`): grounding on required steps; overflow → converge or `BudgetExhausted`; `InvalidTransition`; `MalformedGrounding`; free Status; independent topics; scope WARN when required; **≥12 tests**; zero `todo!`/`unimplemented!`/`unwrap()` in `engine.rs`.

Adversarial: Observe.allowed_next must not contain Verify; remaining_calls never wraps to `u32::MAX`; no `sleep`/`Duration`/`Instant` in tests; stubs must be `todo!()` not `Ok` (`phase-2A.txt:136-139`, `:247-254`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- A `GrcState` enum with exactly those seven variants and a default Observe→Hypothesize→PlanAction→Act→Verify chain that rejects Observe→Verify.
- `Converge` and `Status` allowed from any state; Status is free and does not move `last_state`; first real step is Observe.
- `ThinkStep` has the six fields including `scope: Option<Vec<String>>`.
- Grounding strings parse as comma-separated `type:reference` with the six types above; unknown type / missing colon → `MalformedGrounding`; missing grounding on Observe/Hypothesize/Verify → `UngroundedStep`.
- Level map test/search=Verified, file/grep/tool=High, training=Uncertain (warn, not reject in this organ).
- Per-topic `GrcBudget` default 5/800, `u32`, check-then-increment, `must_converge` at `remaining_calls<=1`, Status free, `reset_topic` deletes the entry.
- Typed `GrcError` variants named above (plus `from: Option<GrcState>` to satisfy T9).
- `GrcEngine::{think,set_budget,get_budget,reset_topic}` with **no LLM call** inside `reasoning/`.
- T1–T12 exist and pass. T2 uses `max_calls=3`.
- `GroundingLevel` comparable as Uncertain < High < Verified (2C-safe).

A prompt wrapper, an MCTS tree, or a “reasoning tokens until window full” cap is a cousin, not this organ.

## 14. Non-goals / banned designs

- Grounding **verification** (FILE_CHECK etc.) — Phase 2B (`phase-2A.txt:12`).
- Per-agent budgets / double-verify / FINDING: — Phase 2C.
- CSQ / inline feasibility — Phase 2D.
- Prompt-driven transition rules (`phase-2A.txt:21`).
- Token-level FSM masking as a substitute for step-level `allowed_next` (`phase-2A.txt:30-32`).
- Increment-before-check budget (`phase-2A.txt:296-298`).
- Signed budget counters (`phase-2A.txt:138-139`).
- LLM self-eval of grounding quality; scalar confidence (`phase-2B.txt:13-14` already bans scores for 2B; 2A has none).
- Treating context-window size as the GRC call budget (that is 3E / 4A IPC).
- `Ok` placeholders for 2B stubs; they must be `unimplemented!()` until 2B (`phase-2A.txt:247-248`).
- Criterion dep without approval (`phase-2A.txt:347`).
- Declaring `GroundingLevel { Verified, High, Uncertain }` and deriving `PartialOrd` (`phase-2C.txt:342-347`).

## 15. Source citations

Primary: `docs/specs/guide/steps/phase-2A.txt` (full file). Index: `phase-index-MASTER.txt:277-299`. Overview: `ELAI-SPEC-OVERVIEW.md:64-72`. Feature map: `ELAI-FEATURE-MAP.md:563-569`. Systems: `ELAI-SYSTEMS-OVERVIEW.md:67-72`. Context: `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:240-244` (§7.1). Exhaustive ids: `ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` P2A-E001..E033, P2A-A001..A003, P2A-C001. Audit (implementation truth, not spec): `audit-implementation/phase-2A.md`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Full `allowed_next()` table beyond the chain + T4 + Verify-not-dead (`P2A-A001`, `phase-2A.txt:72-76`).
- Whether `InvalidTransition.allowed` includes Status/Converge (`phase-2A.txt:193-197` vs `:75`).
- T11 hard `Err` vs warning (`P2A-A002`, `phase-2A.txt:229-232`).
- Whether Converge requires grounding (`phase-2A.txt:273-285` silent).
- Whether IPC `ThinkStep` ≡ `reasoning::ThinkStep` (`phase-4A.txt:55-59`).
- How `ThinkResult.grounding_level` is chosen when a step has mixed citation levels (spec classifies “each citation”; result field is a single `Option<GroundingLevel>`) (`phase-2A.txt:279`, `:87-88`).
- Exact warn-log destination (engine log vs `warnings` vec). Spec says “warn log” for Uncertain and WARN strings for scope (`phase-2A.txt:55`, `:84`).
- `<5ms` / `<1KB` targets were UNGROUNDED in the phase file itself and only empirically gated in 2A.5 (`phase-2A.txt:374`, `:380`).
