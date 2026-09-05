# Serial-Depth Reasoning Budget (H-42)

- **ELAI ids:** H-42 / RG-42 / RD-4 / RS-10 / RS-11
- **Spec status (as designed):** PARTIAL (feature-map 1.2); original plan PLANNED Track-6 extension of Phase-2C GRC + E-19
- **Crate / module (spec):** `crates/orchestrator/src/reasoning_budget.rs` (`estimate_serial_depth`, `scale_grc_budget`, `classify_failure`); consume `grc.rs::AgentGrcConfig`, `core/reasoning/self_certainty.rs::compute_score`; `ssb.rs::SsbActionType::ReasoningBudgetScaled`
- **Purpose of this file:** enough architecture to rebuild depth-proportional GRC/reflection budgets and the depth-not-width failure classifier. Not a status ledger.

## 1. What it is

The **serial-depth-scaled reasoning budget** takes the shipped **per-role static** `AgentGrcConfig` (call+char, `required_steps`) and multiplies it by an estimated **serial depth** `d` of the current sub-task, **clamped** to a LOCKED ceiling (`mutable=0`). The same factor scales the E-19 micro-reflection budget. On GRC exhaustion or low `self_certainty` at high `d`, `classify_failure` labels `DepthBound` (needs more CoT **steps** / decomposition) vs `AccessBound` (needs more/better **context**). A context-window lift is **never** the fix for `DepthBound` (RD-4) (`H-plan/H-42_serial_depth_reasoning_budget.md:5-15`, `:59-72`, `H-REASONING-GROUNDING-MAP.md:107-110`, `ELAI-FEATURE-MAP.md:632-634`).

## 2. Why it exists

GRC budget is per-**role** and **static** (`grc.rs:223` `AgentGrcConfig.required_steps`; `reasoning/config.rs:430-580` `agent_grc_config_for_role`) (`H-plan/H-42_serial_depth_reasoning_budget.md:5-6`, `:38-39`). Formal CoT-expressivity: a single forward pass is ~TC0; the autoregressive CoT loop is the escape to higher classes — polynomial CoT reaches **exactly P** (2310.07923); compositional reasoning needs ~O(n) CoT tokens with tight step lower bounds (2503.01544 / 2502.02393). A flat per-role budget **under-budgets** a deep serial sub-task and **forces** the truncation the budget was meant to prevent. The video/target paper 2507.07505 claims “more context / more steps buys nothing”; the formal literature says the **opposite** on both counts: steps buy class-reach; a bigger window buys **information access**, not compute-depth (`:27-35`, `H-REASONING-GROUNDING-MAP.md:9-16`, `:85-89`). If this organ is missing, deep baby-steps starve. If it is faked (window-size as the tier), a depth-class failure gets the wrong remedy (RD-4). This formally grounds “smarter orchestration, not a smarter model” (H-7) (`H-plan/H-42_serial_depth_reasoning_budget.md:34-35`). Honest residue: depth-scaling helps a **serial-decomposable** task; a task above the model’s reachable class even with CoT must route to a tool/solver (H-41) or refuse (D-16) — not “more steps always wins” (`:80-81`).

## 3. How it works (runtime)

1. **Who calls it.** Per-baby-step dispatch, where `agent_grc_config_for_role` is resolved: `scale_grc_budget(base, d)` then a `ReasoningBudgetScaled` row. GRC exhaustion → `classify_failure` (`H-plan/H-42_serial_depth_reasoning_budget.md:93-95`, `:154`).
2. **`estimate_serial_depth(sub_task)` (deterministic).** Read the **cheapest shipped** signal: decomposition-tree depth of the current node (`concept > phase > step > babystep` nesting) **or**, for a code task, DKG reachability-chain length (longest must-traverse call/dep chain from C-30 `query_reachability` / B-29 impact radius). Returns bounded integer `d`. No wall-clock; seed-stable (`:59-62`).
3. **`scale_grc_budget(base: AgentGrcConfig, d)` (deterministic).** Multiply per-role call+char **and** E-19 reflection budget by a **monotone factor of `d`**, CLAMPED to a LOCKED ceiling. Base per-role config is the **floor**; scaling only ever **raises** toward the ceiling for genuinely deep sub-tasks. Depth 0/1 unchanged (`:64-67`, `:148`). Ceiling `mutable=0` so a loop cannot inflate its own budget (`:66-67`, `:98`).
4. **`classify_failure` (labeled diagnostic, not a hard gate).** On GRC budget-exhaustion **or** low `self_certainty.rs::compute_score` at high `d`:
   - `DepthBound`: high `d` + budget exhausted → fix is **more CoT steps / decomposition**, **never** more context.
   - `AccessBound`: low `d` + retrieval miss → fix is more/better context.
   (`:69-72`)
5. **Mixed failure.** A mixed case (deep **and** retrieval-starved) should surface **BOTH**, not force one (`:175-176`).
6. **Commit.** `ReasoningBudgetScaled` Pattern-2 `{role, base_calls, serial_depth, scaled_calls, ceiling, failure_class}`. Recomputable from the depth signal + LOCKED ceiling under `ELAI_SEED` (`:74-76`).
7. **R2.** `--flat-budget` **must** give a deep and a shallow sub-task the **same** budget. If equal anyway, depth-scaling is vacuous (`:100`, `:110-111`).
8. **Fail-closed.** A `DepthBound` row whose remedy is “raise context” is a FAIL (RD-4) (`:97`). Unbounded budget banned (`:98`). No re-implementation of GRC config or E-19 reflection (GD-1 P0) (`:99`).
9. **Cost guardrail.** The ceiling bounds total cost; HD-2/HD-3 (cache + output) still apply to the **scaled** budget (`:173-174`).

The model may generate step **content**. It may **not** pick `d`, raise the ceiling, or treat window size as the GRC budget (that is 3E / 4A / I-26 neighbors; RD-4). J-84 later routes trivial=ZERO / easy-medium=serial DEPTH / hard=parallel WIDTH **over** this organ + I-26; **WINDOW is never a tier** (`ELAI-FEATURE-MAP.md:656-667`).

## 4. Types, enums, objects

Copy from `H-plan/H-42_serial_depth_reasoning_budget.md:38-45`, `:59-76`. Do not invent fields.

```
AgentGrcConfig { required_steps, ... }   // Phase-2C; EXTEND via scaler, do not replace

fn estimate_serial_depth(sub_task) -> u32
  // cheapest of: tree depth | DKG reachability-chain length
  // bounded integer d

fn scale_grc_budget(base: AgentGrcConfig, d: u32) -> AgentGrcConfig
  // monotone in d; floor = base; clamp to LOCKED ceiling (mutable=0)
  // also scales E-19 reflection budget

FailureClass { DepthBound, AccessBound }

fn classify_failure(...) -> FailureClass
  // diagnostic, not a hard gate

ReasoningBudgetScaled {               // SsbActionType; Pattern-2
  role,
  base_calls,
  serial_depth,
  scaled_calls,
  ceiling,
  failure_class,
}
```

Exact monotone formula (linear in `d`? `O(n)`?) and numeric ceiling: **UNCERTAIN** (§16).

## 5. Control flow / state machine

**Owner:** orchestrator dispatch at baby-step budget resolution. GRC `think` loop remains 2A/2C/4A. H-42 **sets** the budget those engines enforce.

```
resolve agent_grc_config_for_role(role) -> base
d = estimate_serial_depth(sub_task)
scaled = scale_grc_budget(base, d)          // floor=base, clamp=ceiling
emit ReasoningBudgetScaled { ..., failure_class maybe empty }
run GRC / E-19 reflection under scaled budget
on exhaustion or (low self_certainty ∧ high d):
  classify_failure -> DepthBound | AccessBound (or BOTH if mixed)
  write failure_class on the row
```

**Forced:** `scaled_calls(deep) > scaled_calls(shallow)` for same role, both `<= ceiling` (`:106-109`). `DepthBound` remedy = steps, not context (`:97`).

**Not this organ:** J-85 overthinking guard (chain **DRIFT** off a correct answer) — name only (`ELAI-FEATURE-MAP.md:668-677`). Anti-spiral C-10 catches **REPETITION**. H-42 catches **under-budgeted serial depth**.

## 6. Grounding, receipts, gates

**Receipt for `d`:** shipped tree depth or DKG reachability length — a **proxy**, not the true CoT-step lower bound (generally uncomputable) (`H-plan/H-42_serial_depth_reasoning_budget.md:170-172`).

**Never allowed as the depth / budget signal:** context-window size; remaining tokens; LLM “this is hard”; treating 3E slot fill as GRC `max_calls` (`H-REASONING-GROUNDING-MAP.md:107-110`, `ELAI-FEATURE-MAP.md:663`).

**Self-certainty** is an **input to the classifier**, reused from E-19; it is not permission to raise the ceiling (`H-plan/H-42_serial_depth_reasoning_budget.md:69-70`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| floor | base `AgentGrcConfig` for the role | per role |
| `d` | bounded integer from tree or DKG chain | per sub-task |
| scale | monotone factor of `d`, **UNCERTAIN** closed form | per sub-task |
| ceiling | LOCKED, `mutable=0` | global cost guardrail |
| depth 0/1 | unchanged from base (`:148`) | per sub-task |
| Status-free GRC steps | still 2A rules | not redefined here |
| schema | `+ReasoningBudgetScaled` baseline+1 off `220/497` | SSB; feature-map SCHEMA 220 |

Start conservative (small factor); raise only on evidence of depth-bound truncation (GD-4, ≥3 real runs) (`:171-172`).

## 8. Roles and neighbors

**Consumes:** `AgentGrcConfig` (RS-10), `compute_score` + `truncate_to_budget` / micro-reflection (RS-11 / E-19), B-7 tree depth or C-30/B-29 reachability (`:43-44`). **Produces:** `estimate_serial_depth`, `scale_grc_budget`, `classify_failure`, `ReasoningBudgetScaled`. **Feeds:** DYNAMIC THINKING BUDGETS (I-26); J-84 effort classifier (`ELAI-FEATURE-MAP.md:632-634`, `:656-667`).

**Must not fork:** a second GRC engine; window-as-tier; unbounded self-inflating budget.

**Parallel-safe with:** H-39, H-40, H-41 (`H-plan/H-42_serial_depth_reasoning_budget.md:21`). Out-of-class tasks → H-41 tool/solver or D-16 refuse (`:80-81`).

**Dashboard:** per-sub-task serial-depth + scaled-budget + failure-class in the reasoning drawer (`:102-104`).

**Neighbor, do not author:** J-85 overthinking guard (DRIFT). C-10 anti-spiral (REPETITION).

## 9. Config / flags / defaults

- Default-ON: depth-scaling on every sub-task; no env override; kill-switch only; floor = base per-role config (`:124`).
- Ceiling LOCKED `mutable=0` (reuse rules-engine / `LockedKeysAuditor` pattern named in H-0 HD, not a new key specified here).
- Kill-switch identifier: **UNCERTAIN**.
- Decision doc: `workflow/reasoning/H-42-decisions.md` (`:78-81`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `reasoning_budget.rs::estimate_serial_depth` | `H-plan/H-42_serial_depth_reasoning_budget.md:85` |
| `scale_grc_budget` | `:86` |
| `classify_failure` | `:87` |
| `SsbActionType::ReasoningBudgetScaled` | `:88` |
| consume `grc.rs::AgentGrcConfig` | `:90` |
| consume `self_certainty.rs::compute_score` | `:91` |
| consume tree depth **or** `query_reachability` | `:92` |
| per-baby-step dispatch site | `:154` |
| reasoning drawer Svelte | `:157` |
| `h42_serial_depth_budget_contract.rs` | `:147` |
| fixtures `fixtures/reasoning_budget/` | `:151` |

Which depth source is “cheapest” at ship time: **pick one shipped**; do not build a new depth oracle (`:44-45`, `:59-62`).

## 11. Journal / SSB events

| Event | Payload | Notes |
|---|---|---|
| `ReasoningBudgetScaled` | `{role, base_calls, serial_depth, scaled_calls, ceiling, failure_class}` | Pattern-2; feature-map SCHEMA 220 (`ELAI-FEATURE-MAP.md:634`) |

J-84 **prefers reuse** of this row (`ELAI-FEATURE-MAP.md:666`). Do not invent J-84 here.

## 12. Tests and observables

Contract (`H-plan/H-42_serial_depth_reasoning_budget.md:147-151`):

| Id | Assert |
|---|---|
| (a) | `estimate_serial_depth` on a fixture tree |
| (b) | `scale_grc_budget` raises with depth, clamps at ceiling |
| (c) | base is floor (depth 0/1 unchanged) |
| (d) | `DepthBound` at high `d` + exhaustion; `AccessBound` at low `d` + retrieval miss |
| (e) | `--flat-budget` equalizes deep and shallow (R2) |
| (f) | ceiling `mutable=0` (cannot raise at runtime) |
| (g) | `ELAI_SEED` twice → identical |

**Observable effect** (`:105-111`): same role, shallow `d~1` vs deep serial sub-task → two rows, `scaled_calls(b) > scaled_calls(a)`, both `<= ceiling`; forced exhaustion at high depth → `failure_class=DepthBound`.

W-1: `--run-trigger` PASS. `cargo-green != DONE` (`:123`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Budget scales with estimated **serial depth**, **never** with context-window size (RD-4).
- `d` from shipped tree depth or DKG reachability length (proxy, named as proxy).
- Floor = base per-role `AgentGrcConfig`; ceiling LOCKED `mutable=0`; monotone raise only.
- E-19 reflection budget scaled with the same factor.
- `DepthBound` remedy = steps/decomposition, not window; `AccessBound` = context; mixed may surface both.
- `ReasoningBudgetScaled` carries `{role, base_calls, serial_depth, scaled_calls, ceiling, failure_class}`.
- `--flat-budget` equalizes deep and shallow (R2).
- Out-of-class tasks documented as tool/refuse, not infinite CoT.

A larger context window, “think harder” prompt, or treating 3E slots as GRC `max_calls` is a cousin.

## 14. Non-goals / banned designs

- Context-window lift as the fix for `DepthBound` (RD-4 FAIL) (`:97`).
- Unbounded / self-inflating budget (`:98`).
- Replacing GRC or E-19 (`:99`).
- Window-as-tier (J-84 forbids it; H-42 is the substrate) (`ELAI-FEATURE-MAP.md:663`).
- Authoring J-85 overthinking guard (DRIFT neighbor).
- Claiming more steps always wins (`H-plan/H-42_serial_depth_reasoning_budget.md:80-81`).
- Transferring paper 6K/7K token knees as the ceiling (K-16 / J-85 doctrine; ID-5) (`ELAI-FEATURE-MAP.md:673-675`) — H-42’s ceiling is LOCKED local, not a paper bin.

## 15. Source citations

Primary: `docs/thefinalglance/H-plan/H-42_serial_depth_reasoning_budget.md` (full file). Grounding: `H-REASONING-GROUNDING-MAP.md:13-16`, `:57-61`, `:85-89`, `:107-110`, `:117-125`, `:150`. Doctrine: `H-plan/H-0_INDEX.md:348-351`. Index: `H-plan/H-SUMMARY_INDEX.md:686-696`. Overview: `ELAI-SPEC-OVERVIEW.md:248-249`. Feature map: `ELAI-FEATURE-MAP.md:632-634`, `:656-677` (J-84/J-85 neighbors). Papers: 2310.07923, 2402.12875, 2503.01544, 2502.02393 (confirm ids before headlines).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Closed-form scale factor (linear in `d` vs other monotone) (`H-plan/H-42_serial_depth_reasoning_budget.md:64-67`).
- Numeric ceiling (`:66`, `:173-174`).
- Which shipped depth source is selected when **both** tree depth and DKG chain exist (`:59-62`).
- `classify_failure` output type when mixed (two flags vs a third variant) (`:175-176`).
- Kill-switch name.
- Whether `failure_class` is omitted on non-exhaustion rows (`:74` lists the field always).
- SCHEMA live integer vs feature-map 220.
