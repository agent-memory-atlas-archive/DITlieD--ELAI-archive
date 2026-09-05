# Dynamic thinking budgets (I-26)

- **ELAI ids:** I-26 / landmine #6 / HD-3 / ID-5 / RD-1..RD-4 / H-42 extension (no new SSB type)
- **Spec status (as designed):** DARK (`ELAI_DYNAMIC_BUDGET_ENABLE`; `ELAI-FEATURE-MAP.md:2355-2359`). Original plan WAVE 4 EXTENSION of H-42 onto I-4 search nodes (`I-plan/I-26_dynamic_thinking_budgets.md:3-8`).
- **Crate / module (spec):** `crates/orchestrator/src/routing/budget_controller.rs` (`BudgetController`, `node_budget`, `cloud_effort_for_node`); cloud builder `providers/src/anthropic_api.rs` (live: `providers/src/runtime/anthropic_api.rs::build_request`); H-42 row `ReasoningBudgetScaled` **extended in place**.
- **Purpose of this file:** enough architecture to rebuild depth-scaled **search-node** compute, 85% / plateau collapse, and Opus **effort** mapping. Not a status ledger.

## 1. What it is

**Dynamic thinking budgets** apply H-42’s serial-depth discipline to **I-4 search-node exploration**. Each node’s compute envelope scales with DAG depth. When accumulated sibling spend hits **85%** of the search envelope, exploration **halts**, the frontier is L2-scored, and the best result is synthesized. If top-L2 is **flat across 5 supersteps**, collapse **early**. Cloud reasoning maps to the Anthropic **`effort`** parameter (`auto` / `expert`). **`budget_tokens` is never sent on Opus 4.7+** (landmine #6). Reasoning roles are **never force-JSON’d** (HD-3) (`I-plan/I-26_dynamic_thinking_budgets.md:4-25`, `I-plan/I-0_INDEX.md:560-561`, `H-plan/H-0_INDEX.md:291-297`).

## 2. Why it exists

H-42 budgets the **GRC chain**. Unbounded **search width** still burns tokens after the answer has plateaued. If this organ is missing, I-4 expands until E-39 or the window. If it is faked (`budget_tokens` on Opus 4.7+, or grammar-wrapping a reasoner), the API rejects or reasoning degrades (`I-plan/I-26_dynamic_thinking_budgets.md:4-8,23-25`, `H-plan/H-0_INDEX.md:294-297`). Paper depth-factors as gates violate ID-5 (`:73,88`).

## 3. How it works (runtime)

1. **Who calls it.** I-4 `SearchLadder` at **each node expansion** and at the **collapse point** (`:49,66-68`). Cloud `build_request` for reasoning roles (`:50-51`). The generator does **not** pick depth, ceiling, or effort.
2. **Input.** Node **depth** in the search DAG; accumulated sibling spend; top-L2 score this superstep; whether the role is a reasoning role; E-39 hard ceiling as supreme cap (`:14-18,61-64`).
3. **`node_budget(depth)` (deterministic).** `base * depth_factor^depth`, capped at E-39 / locked ceiling (`:14`). Live integer form: `BASE_BUDGET_UNITS * (125/100)^depth`, clamp `BASE * MAX_DEPTH_MULT` (`budget_controller.rs:111-134`). Depth 0 = base.
4. **Record expansion (deterministic).** After `should_collapse` is false, add `node_budget(depth)` to `accumulated_spend` (`budget_controller.rs:173-181`).
5. **85% collapse (deterministic).** If `accumulated_spend / total_envelope >= 0.85`: halt **new** expansions; score frontier with I-17 L2; synthesize best; emit H-42 `ReasoningBudgetScaled` (collapse metadata) and I-27 `ConvergenceCurveMeasured` (`:15-17,81-82`). **No** `SearchNodeEvent(Expanded)` after the collapse marker (`:82`).
6. **Plateau collapse (deterministic).** If top-L2 of the surviving branch does not improve by `delta_min` across **5 consecutive supersteps**, early collapse (`:17-18`). Composes E-19 confidence as the **prior**, does not re-implement E-19 (`:63,95`).
7. **Cloud mapping (deterministic).** Reasoning-role node → `effort` only. Live: depth ≤1 → `"auto"`; depth ≥2 → `"expert"`; non-reasoning → `None` (`budget_controller.rs:261-291`). **`budget_tokens` structurally absent** from the Opus 4.7+ builder (`:19-21,70-71`).
8. **HD-3.** Caller must **not** set `force_json` / `tool_choice` JSON schema on a reasoning / extended-thinking role (`:21,72`, `H-plan/H-0_INDEX.md:294-297`). Constrained decode is for **op-emission** roles (I-34 / H-13), not this path.
9. **Commit.** Reuse **H-42** `ReasoningBudgetScaled`. **No new action_type.** If payload lacks `depth_level` / `plateau_fired`, **extend in place** (`:22-23,61,90`).
10. **Fail-closed / abstain.** Collapse is host-enforced; generator cannot reach around it (IS-19 / R8: collapse not generator-reachable) (`:32`). Post-collapse expand is structurally impossible (`:82`).
11. **The model may generate** node content inside the envelope. It may **not** raise the ceiling, keep expanding after collapse, emit `budget_tokens`, or JSON-schema a reasoner.

## 4. Types, enums, objects

Copy from `I-plan/I-26_dynamic_thinking_budgets.md:13-23,56-58` and live controller. Do not invent.

```
BudgetController {
  total_envelope: u32,          // session search cap; never above E-39
  accumulated_spend: u32,
  score_history: VecDeque<f32>, // last PLATEAU_WINDOW(+1) top-L2 scores
}

fn node_budget(depth: u32) -> u32
  // plan: base * depth_factor^depth, cap E-39
  // live: BASE_BUDGET_UNITS * (DEPTH_FACTOR_NUMERATOR/DENOMINATOR)^depth
  //       clamp BASE_BUDGET_UNITS * MAX_DEPTH_MULT

fn BudgetController::new(total_envelope: u32)
fn record_expansion(&mut self, depth: u32)
fn budget_fraction(&self) -> f32          // 0 if envelope==0
fn should_collapse_budget(&self) -> bool  // fraction >= 0.85
fn record_score_and_check_plateau(&mut self, top_score: f32) -> bool
fn should_collapse(&mut self, top_score: f32) -> bool  // plateau OR 85%

fn cloud_effort_for_node(depth: u32, is_reasoning_role: bool) -> Option<String>
  // Some("auto") | Some("expert") | None
  // NEVER "budget_tokens"

fn dynamic_budget_enabled() -> bool   // env 1|true; gates 85%/plateau ONLY

// Live starter constants — ID-5: NOT gates until I-27 measures them
BASE_BUDGET_UNITS: u32 = 100
DEPTH_FACTOR_NUMERATOR / DENOMINATOR = 125 / 100    // 1.25x per level
MAX_DEPTH_MULT: u32 = 3
COLLAPSE_THRESHOLD: f32 = 0.85
PLATEAU_WINDOW: usize = 5
PLATEAU_DELTA_MIN: f32 = 0.01
DEPTH_FACTOR_CALIBRATION_PENDING: bool = true
ELAI_DYNAMIC_BUDGET_ENABLE_ENV = "ELAI_DYNAMIC_BUDGET_ENABLE"

// H-42 payload EXTENDED in place (I-26 fields Option + skip_serializing_if)
ReasoningBudgetScaledPayload {
  kind, role, base_calls, serial_depth, scaled_calls, ceiling, seed,  // H-42
  depth_level: Option<u32>,     // I-26 search-DAG depth; None on GRC rows
  budget_pct: Option<f32>,      // sibling spend fraction
  plateau_fired: Option<bool>,
}
```

**Do not confuse:** H-42 `serial_depth` (GRC concept/phase/step/babystep) vs I-26 `depth_level` (search DAG). Same row family, different fields (`ssb/payloads.rs:19476-19501`).

**Do not confuse:** I-4 `SearchNodeEvent::BudgetCollapsed` (all candidates **vetoed**, no winner) vs I-26 collapse (envelope/plateau). Different events (`I-04` §16).

## 5. Control flow / state machine

**Owner of the loop:** orchestrator `BudgetController` inside I-4’s expansion. Generator proposes nodes; controller admits or collapses (`:96`).

| Check | Fire | Effect |
|---|---|---|
| `budget_fraction >= 0.85` | 85% collapse | halt new expansions; L2-score frontier; synthesize best |
| L2 flat `delta_min` for 5 supersteps | plateau | early collapse (saves tokens) |
| either | `should_collapse` | no further `Expanded` |
| flag OFF | skip 85%/plateau | effort mapping still live (live fill) |

**Forced:**

- E-39 ceiling is **supreme**; controller never exceeds it (`:14,62`).
- Opus 4.7+ request: `effort` present-or-absent; **`budget_tokens` field does not exist** (`:70-71`).
- HD-2: prefix must **not** encode a mutable budget token; `None` effort / I-26 Options **omit from JSON** (`:33`, contract b/c).
- Adaptive `depth_factor` from I-27 = Tier-2 (`:44-46`).
- Per-role effort beyond auto/expert = Tier-2 (`:46`).

**Live wire-dark:** `BranchInput` has **no depth**; search-node methods wait on `I-26-promotion` (`budget_controller.rs:25-32,145-147`). Effort **field plumbing** is already in `build_request`. Rebuild the **plan** loop (ladder calls controller per expansion); do not treat missing depth on v1 `BranchInput` as a license to skip the organ.

## 6. Grounding, receipts, gates

This organ **allocates compute**. It does not verify claims (I-17) or pick winners (I-4 receipts).

**Never allowed as a budget / quality signal:** paper depth-factor as a gate (ID-5); window size as the thinking tier (RD-4 / J-84: WINDOW is never a tier); LLM “I need more tokens”; `budget_tokens` on Opus 4.7+; force-JSON of a reasoner as a substitute for effort (`:70-73`, `ELAI-FEATURE-MAP.md:661-664`).

Collapse **scores the frontier with I-17 L2** (`:16`) — that is a **receipt**, not a self-eval. Synthesize-best still cannot resurrect HardVeto (ID-4; I-4/I-17).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| 85% collapse | `COLLAPSE_THRESHOLD = 0.85` | per search envelope |
| plateau window | **5** supersteps | per controller |
| `delta_min` | live starter **0.01** (`PLATEAU_DELTA_MIN`) | until I-27 |
| depth factor | live 1.25x/level, cap **3×** base | until I-27 |
| E-39 | supreme USD/token ceiling | session |
| effort | `auto` / `expert` only in v1 | cloud reasoning |
| SSB | **+0** types; extend H-42 (`:90`) | organ |
| module size (plan) | ~100 LOC (`:37`) | advisory |

**Overflow:** hit 85% or plateau → collapse, **no** further expansions. Envelope 0 → `budget_fraction` 0 (no divide-by-zero) (`budget_controller.rs:187-190`). Integer path uses saturating mul (`:128-133`).

**ID-5:** `DEPTH_FACTOR_*` and `PLATEAU_DELTA_MIN` stay tagged calibration-pending until ≥3 I-27 measurement runs (`:42-45,72-73`, `wire-dark-allowlist.toml:1923-1928`).

## 8. Roles and neighbors

Search-node budgets are **role-agnostic** except cloud **reasoning** roles get `effort`. Coder/planner/reasoner: **no** constrained JSON (HD-3). Op-emission roles: I-34 grammars, not this controller.

**Consumes:** H-42 row family; E-39 ceiling (live stand-in `LOCKED_BUDGET_CEILING` in `reasoning_budget.rs` — `budget_controller.rs:60-61`); E-19 confidence prior; I-4 expansion loop (`:60-64`).

**Produces:** depth-scaled envelopes; collapse; `effort` on `CompletionConfig` / `MessageRequest`; extended `ReasoningBudgetScaled` fields. I-27 curve is the **measurement** of this bet (`:17,44`). J-84 routes trivial=ZERO / easy-medium=serial DEPTH / hard=parallel WIDTH **over** H-42 + this controller (`ELAI-FEATURE-MAP.md:656-664`) — cite, do not author J-84.

**Must not fork:** a second GRC scaler (that is H-42); a second I-4 expansion loop (`:96`); `budget_tokens` “compat” for Opus 4.7+.

**H-42 architecture file:** `reasoning-architecture/H-42-serial-depth-reasoning-budget.md` — GRC `scale_grc_budget`. This organ does **search nodes**.

## 9. Config / flags / defaults

- Live: **`ELAI_DYNAMIC_BUDGET_ENABLE`** default-OFF (1/`true`). Gates **85%/plateau only**. Effort plumbing is **not** gated (`budget_controller.rs:100-105,294-307`, `ELAI-FEATURE-MAP.md:2355`).
- Promotion: `I-26-promotion` after I-4 API has depth **and** ≥3 measured runs (`wire-dark-allowlist.toml:1927-1930`).
- Decision doc: `workflow/reasoning/I-26-decisions.md` (`:42`).
- Frontend: budget bar on I-4/H-15 branch board from `depth` + `budget_remaining` / `budget_pct` (`:53,76-77`).
- HD-2: `effort=None` and I-26 `Option` fields **absent from JSON** (contract b/c).
- WebSearch-confirm current Anthropic effort API **before** editing the builder (`:86`).

Rebuild: **default-OFF collapse** until measured; **always-on structural absence of `budget_tokens`**.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `routing/budget_controller.rs::BudgetController` | `I-plan/I-26_dynamic_thinking_budgets.md:57` |
| cloud builder `effort` replaces `budget_tokens` (Opus 4.7+) | `:58,50` |
| I-4 `search/ladder.rs` expansion hook | `:66-67` |
| H-42 `ReasoningBudgetScaled` extend | `:61` |
| E-39 ceiling | `:62` |
| `i26_dynamic_thinking_budgets_contract.rs` | `:40-41` |
| `workflow/reasoning/I-26-decisions.md` | `:42` |
| live `providers/src/runtime/anthropic_api.rs::build_request` | module docs `:34-38` |
| live `MessageRequest.effort` / `CompletionConfig.effort` | contract (a)(b) |
| `ELAI_DYNAMIC_BUDGET_ENABLE` allowlist | `wire-dark-allowlist.toml:1918-1937` |

## 11. Journal / SSB events

**No new action_type** (`:22-23,90`). Reuse H-42 `ReasoningBudgetScaled`. I-26 fields: `depth_level`, `budget_pct`, `plateau_fired` as `Option` with `skip_serializing_if` so old H-42 rows stay byte-stable (HD-2) (`ssb/payloads.rs:19488-19501`).

On 85% collapse the plan also names **I-27** `ConvergenceCurveMeasured` (`:17`) — that event is **I-27’s** instrument; I-26 **triggers** it, does not define it.

Observable: per-node rows with `depth_level`; over-budget fixture → collapse row (`plateau_fired=true` **or** `budget_pct>=85`) then **no** further expansions; Opus 4.7+ journaled request bytes contain `effort`, **zero** `budget_tokens` (`:80-81`).

## 12. Tests and observables

Named (`:70-82`, contract a–g):

| Id | Assert |
|---|---|
| (a) | `MessageRequest` serde JSON has **no** `budget_tokens` when `effort=Some("auto")` on `claude-opus-4-7` |
| (b) | `effort` round-trips; `None` omitted (HD-2) |
| (c) | H-42 row with I-26 Options None still decodes; Some round-trips |
| (d) | `node_budget(0)==BASE`; monotone up to ceiling; collapse/plateau deterministic |
| (e) | `cloud_effort_for_node` never returns `"budget_tokens"`; HD-3 no force-JSON |
| (f) | real sqlite `ReasoningBudgetScaled` with I-26 fields |
| deep vs shallow | same role, deeper fixture **larger** node budget (`I-plan/I-SUMMARY_INDEX.md:493-494`) |
| 85% | seeded over-budget: frontier scored, best synthesized, **no** post-collapse Expanded |
| plateau | early collapse, measured token save, GD-2 recomputable |
| rg | `budget_tokens` count **0** on Opus 4.7+ builder branches (`:82`) |

Trigger: `elai run --task <deep-fixture>` (`:80`). Smoke: `i26_f_real_db_reasoning_budget_scaled_with_i26_fields_lands`.

Adversarial: `budget_tokens` sneaked onto Opus; force-JSON reasoner; post-collapse expand; paper 1.25x promoted to a gate before I-27; encoding budget in the cache prefix (HD-2).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Search-node budget = **serial-depth scaled**, capped by a hard cost ceiling (E-39-class).
- **85%** envelope → halt expansions, score frontier, synthesize best; **no** further Expanded.
- Plateau: L2 flat across **5** supersteps → early collapse.
- Cloud reasoning uses **`effort` (`auto`/`expert`)**. **`budget_tokens` does not exist** on Opus 4.7+ request types.
- Reasoning / extended-thinking roles are **never force-JSON / grammar-wrapped** (HD-3).
- SSB is **H-42 `ReasoningBudgetScaled` extended**, not a new type.
- Depth-factor / delta_min are **not** paper gates (ID-5 / I-27).
- Collapse is **not** generator-reachable.
- Window size is **not** the thinking budget (RD-4).

A “thinking_tokens=400” Claude knob, an MCTS sim-limit, or JSON-schema’d CoT is a **cousin**.

## 14. Non-goals / banned designs

- `budget_tokens` on Opus 4.7+ (`:24,70-71`; `I-plan/I-0_INDEX.md:560-561`).
- Force-JSON / constrained decode on reasoning roles (`:25,72`; `H-plan/H-0_INDEX.md:294-297`).
- New SSB action_type (`:22-23`).
- Reimplementing H-42 GRC scaler, E-39 ceiling, E-19 estimator, or I-4 ladder (`:92-96`).
- Paper constants as thresholds (`:73`).
- Adaptive depth_factor / extra effort levels in v1 (`:44-46`).
- Treating I-4 `BudgetCollapsed` as this collapse.
- Encoding mutable budget in the cache prefix (`:33`).

## 15. Source citations

Primary: `docs/thefinalglance/I-plan/I-26_dynamic_thinking_budgets.md` (full file). Index: `I-plan/I-SUMMARY_INDEX.md:483-497`. Landmine #6: `I-plan/I-0_INDEX.md:560-561`. HD-3: `H-plan/H-0_INDEX.md:291-297`. H-42 organ: `reasoning-architecture/H-42-serial-depth-reasoning-budget.md`. Feature map: `ELAI-FEATURE-MAP.md:2355-2359`, `:634`. Live: `routing/budget_controller.rs`, `ssb/payloads.rs:19467-19501`, `crates/contracts/tests/i26_dynamic_thinking_budgets_contract.rs`, `wire-dark-allowlist.toml:1918-1937`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Numeric `base` / `depth_factor` in the **plan** (only the formula); live 100 / 1.25 / 3× are starters (`:14` vs `budget_controller.rs:73-86`).
- Plan `effort (auto/expert)` does not pin depth≤1 vs ≥2 (`:19` vs `cloud_effort_for_node`).
- Whether 85% is of **sum of node_budgets** vs wall-clock vs $ (plan: “accumulated node-budget spend”) (`:15-16`).
- `total_envelope` derivation from E-39 (callers “should base this on task difficulty”) (`budget_controller.rs:163-164`).
- Plateau `delta_min` not named in the plan body (only “does not improve”); live 0.01 (`:17-18`).
- I-27 `ConvergenceCurveMeasured` on collapse vs H-42 row only (`:17` vs `:22-23` no new type).
- Live `should_collapse` **records a score even when checking budget** (side-effect) (`:240-243`) — plan does not.
- Search-node hook vs wire-dark `BranchInput` without depth (`:66` vs `budget_controller.rs:25-32`).
- `elai run <task>` as reachability vs explicit ladder call (`:67-68`).
- Whether non-Anthropic cloud lanes must omit `budget_tokens` too (plan says Opus 4.7+; stub_ban says “Opus 4.7+ cloud requests”).
- E-39 USD cap “no importable symbol at HEAD” (`budget_controller.rs:60-61`) vs plan “call E-39”.
