# GRC-v3 2-axis reasoning-effort classifier (J-84)

- **ELAI ids:** J-84 / GA-2 / CC-1..CC-4 / WAVE GRC-2 / IMPL 3/5/24/25/26 / RD-4 / TD-3 / ID-5 / GD-4
- **Spec status (as designed):** PLANNED (EXTENSION of H-42 + I-26; depends J-83). Classifier module is absent; the budget substrate is PARTIAL (H-42) + DARK (I-26 `ELAI_DYNAMIC_BUDGET_ENABLE`) (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:1-5`, `J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:67-69`, `ELAI-FEATURE-MAP.md:656-667`).
- **Crate / module (spec):** `crates/orchestrator/src/reasoning_classifier.rs` (new); wires H-42 `reasoning_budget.rs` + I-26 `routing/budget_controller.rs`; TYPE axis is `select_grounding_mode`.
- **Purpose of this file:** enough architecture to rebuild the TYPE × COST router that sizes reasoning over the J-83 carrier. Not a status ledger.

## 1. What it is

The **2-axis reasoning-effort classifier** is a deterministic orchestrator predicate that maps `(task TYPE, difficulty COST)` onto a budget **tier** `{Zero, Depth, Width, EscalateOrAbstain}` over the shipped H-42 serial-depth budget and I-26 `BudgetController`. The TYPE axis **reuses** D-10 `select_grounding_mode` (reason vs **RETRIEVE** vs validate). The COST axis is a proxy over signals ELAI already has (H-39, C-45/C-47, J-38, C-29). A manual **lambda** effort toggle (I-27 / H-9) may override the auto-tier inside a deterministic floor/ceiling. **The LLM never picks the tier. WINDOW is never a tier.** The organ is **OBSERVE-ONLY first**; gating waits on the K-15-measured per-(model, task-type) knee, never a paper bin (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:5,13-18`, `ELAI-FEATURE-MAP.md:656-667`).

GRC-v3 is UNIFICATION + ACTIVATION of existing pieces plus **one** net-new organ (J-85). J-84 is an **extension** of H-42 + I-26, not that new organ (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:8-11,67-69`).

## 2. Why it exists

More reasoning is **non-monotonic**. It helps math/symbolic and medium multi-step work; it **hurts** trivial tasks and factual recall (ROUND-3 ADD-1 table; zero-shot-CoT CommonsenseQA −4.2pp). Routing a recall query to “more chain” is the confabulation case the user flagged (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:5,11`). Test-time-compute (2408.03314) says easy/medium want revision **depth**, hard want parallel **width**, very-hard make little progress (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:105`).

H-42 already scales GRC budget by serial depth and forbids treating window size as the depth lever (RD-4) (`reasoning_budget.rs:1-19`, `H-42-serial-depth-reasoning-budget.md:10-14`). I-26 already has a `BudgetController` + cloud `effort` field, wire-dark behind `ELAI_DYNAMIC_BUDGET_ENABLE` (`budget_controller.rs:1-52`). What is **absent** is the 2-axis classifier that **chooses** Zero / Depth / Width / EscalateOrAbstain (GA-2) (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:67-69`).

If this organ is missing, every task gets the same chain (or the model is asked how hard it is). If it is faked (LLM picks the tier, or window size is a tier), the boxed-myopic grader and the RULER usable-not-available failure both return. Honest residue (R-2): the difficulty proxy is imperfect (a true pass@K oracle is expensive); observe-only then **measure** (K-15), never assume (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:156-158`).

DC-5 (must be stated, not hidden): “depth-not-width” is right that **WINDOW is never the lever**, but hard problems want parallel WIDTH, not more revision-depth. Reconciliation: window→never; easy/medium→serial DEPTH (revise the J-83 carrier); hard→parallel WIDTH (J-86) (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:144-147`, `J-plan/J-84_grcv3_reasoning_effort_classifier.md:110`).

## 3. How it works (runtime)

1. **Who calls it.** The GRC reasoning loop / `task_runner`, per step, **after** a claim exists and **before** the budget is spent. The effort toggle is a CLI/dashboard lambda dial (I-27 / H-9). A `#[cfg(test)]` caller does not satisfy reachable_from (R7) (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:50-53`).
2. **Input.** Typed signals only: D-10 mode for the claim; H-39 confidence/abstention score; C-45/C-47 case-base hit; J-38 retrieval coverage; C-29 `IntentPair` count; optional `U_lambda` override. **No LLM call inside the classifier** (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:14-18,55`).
3. **TYPE axis (CC-1, deterministic, reuse D-10).** Read `select_grounding_mode(claim, ctx)`. Split:
   - **RETRIEVE** — factual/recall → DKG / A-RAG, **never a chain** (it confabulates).
   - **reason** — constructive / operational / cross-family work that needs a chain over the J-83 carrier.
   - **validate** — checker-grounded Mode A/D style confirmation, not more generation.
   J-84 **does not rebuild** D-10 (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:15,45`, `grounding_mode.rs:110`). Live enum is the D-10 cousin (`ConservativeFallback` etc.); rebuild consumes the **plan** A–E meanings for this split. Exact mapping of each `GroundingMode` variant onto `{reason, RETRIEVE, validate}` is **UNCERTAIN** (§16).
4. **COST axis (CC-2, deterministic proxy).** `difficulty ← f(H-39 confidence, C-45 case-base hit, J-38 coverage, C-29 IntentPair count)`. Named constants. No LLM. Plan: start transparent linear/threshold; **calibrate weights against K-15**, do not tune by vibes (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:16,35,109`). Proxy inputs live status: H-39 WIRED (`abstention_score.rs`); C-29 WIRED (`intent_pair.rs:86-97`); C-45/C-47 DARK; J-38 PLANNED (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:24`). Degenerate/empty proxy → fail-closed to the **safe (lower-cost or abstain)** tier (R1) (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:93`).
5. **Route (CC-3, DC-5):**

| TYPE | COST | Tier | What it does |
|---|---|---|---|
| RETRIEVE (recall/factual) | any | retrieve, **not a chain** | DKG/A-RAG; journal shows no chain budget |
| reason/validate | trivial | `Zero` | skip the chain (CoT-hurts-trivial) |
| reason/validate | easy/medium | `Depth` | serial revision of the J-83 carrier via H-42 |
| reason/validate | hard | `Width` | parallel candidates (J-86 best-of-N / BSP) |
| reason/validate | very-hard (`proxy<~0.05`) | `EscalateOrAbstain` | I-7 escalate **or** I-8 abstain |

WINDOW is **never** a row in this table (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:17,79`).

6. **Wire the tier into shipped budgets (deterministic).** `Depth` extends H-42 `scale_grc_budget` / `ReasoningBudgetScaled`. Width/search envelopes extend I-26 `BudgetController`. Do **not** re-author those modules (GD-1) (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:36-37,47`). Live H-42: `estimate_serial_depth` maps Concept=0 … BabyStep=3; `scale_grc_budget` multiplies by `1 + 0.25*d`, clamp `LOCKED_BUDGET_CEILING` (`reasoning_budget.rs:21-26,131-137`). Live I-26: `node_budget` / `should_collapse` / `record_plateau` are WIRE-DARK; `cloud_effort_for_node` is production-wired (`depth <= 1 → "auto"`, `>= 2 → "expert"`) (`budget_controller.rs:15-38`). J-84 **sets** which envelope those organs apply; it does not replace `FailureClass` or the 85% collapse rule.
7. **Lambda overlay (CC-4).** Manual `U_lambda` from I-27 `ConvergenceCurveMeasured` + H-9 cost cockpit. User dials light↔heavy. **Floor/ceiling stay deterministic** regardless of the dial (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:18,38`). An LLM-proposed tier is **ignored** (TD-3) (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:64,93`).
8. **OBSERVE-ONLY first (GD-4).** Emit chosen-tier vs realized verified-outcome. **Do not gate** until K-15 calibrates the per-(model, task-type) knee. Then default-ON behind a `*_DISABLE` kill-switch + allowlist row. I-26 keeps its existing `ELAI_DYNAMIC_BUDGET_ENABLE` (default-OFF until I-4 depth lands) (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:39,86-87`, `budget_controller.rs:51-52`).
9. **Commit.** Prefer-reuse `ReasoningBudgetScaled` (+Kind `TierRouted` / `EffortOverridden`), net 0..+1, decide at edit (W-9) (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:63`). Live payload: `ReasoningBudgetScaledPayload { kind, role, base_calls, serial_depth, scaled_calls, ceiling, seed, …I-26 extras }` (`ssb/payloads.rs:19467-19491`).
10. **On the local Qwen-7B lane.** The engine still **imposes** structured decomposition (small model needs the steps, Cobbe, R-4) **and** the classifier still routes trivial→zero and recall→retrieve (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:11`, `J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:162-165`).

The model may generate claim text and (irrelevant) “I want high effort” chatter. It may **not** select the tier, raise the ceiling, or treat context-window size as the budget.

## 4. Types, enums, objects

Copy from `J-plan/J-84_grcv3_reasoning_effort_classifier.md:13-18,41-48`. Do not invent fields.

```
ReasoningType { Reason, Retrieve, Validate }   // derived from D-10 mode; names from CC-1 prose

DifficultyProxy  // f(H-39, C-45 hit, J-38 coverage, C-29 IntentPair count)
                 // very-hard named threshold: proxy < ~0.05
                 // exact combination UNCERTAIN (§16)

ReasoningTier { Zero, Depth, Width, EscalateOrAbstain }

fn classify_reasoning_effort(...) -> ReasoningTier
  // TYPE from select_grounding_mode; COST from DifficultyProxy
  // WINDOW is not an input and not an output

fn apply_lambda_override(auto: ReasoningTier, u_lambda) -> ReasoningTier
  // stays inside deterministic floor/ceiling

U_lambda   // I-27 ConvergenceCurveMeasured + H-9 cockpit surface
```

**Consumed, not redefined:**

```
select_grounding_mode(claim, ctx) -> GroundingMode     // D-10; GS-8
OutcomeTriple / abstention_aware_score                 // H-39; GS-4
IntentPair { natural_language_intent, concrete_test_assertion, baby_step_ref }  // C-29; intent_pair.rs:86-97
fn estimate_serial_depth(level: DecompositionLevel) -> u32   // H-42; reasoning_budget.rs:131
fn scale_grc_budget(base: AgentGrcConfig, d: u32) -> AgentGrcConfig
FailureClass { depth_bound, access_bound }             // two bools live, not a strict enum (reasoning_budget.rs:28-30)
BudgetController { node_budget, should_collapse, record_plateau, cloud_effort_for_node }  // I-26
ReasoningBudgetScaledPayload { kind, role, base_calls, serial_depth, scaled_calls, ceiling, seed, ... }
```

**Live cousin (do not rebuild as this organ):** I-26 `cloud_effort_for_node` returning Anthropic `"auto"|"expert"` from **search-node depth** (`budget_controller.rs:19-24`). That is a **provider effort string**, not `{Zero, Depth, Width, EscalateOrAbstain}`. J-84 may **feed** I-26 after choosing a tier; it must not collapse the four tiers into `"auto"/"expert"`.

**WINDOW is not a type.** `rg` of `window|context.*size.*tier|32k|32000` as a **tier output** in `reasoning_classifier.rs` MUST be 0 (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:71`).

## 5. Control flow / state machine

**Owner:** orchestrator classifier. H-42 still **enforces** the scaled GRC budget. I-26 still **enforces** search-node envelopes when enabled. J-83 still **carries** committed entries. J-86 still **runs** Width. J-85 still **stops** overthinking past the budget this organ set.

```
claim exists
     │
     ▼
TYPE = map(select_grounding_mode(claim, ctx))     // CC-1; no LLM
COST = difficulty_proxy(H-39, C-45, J-38, C-29)   // CC-2; empty → fail-closed safe
     │
     ▼
auto_tier = route(TYPE, COST)                     // CC-3; WINDOW not in domain
     │
     ├─ RETRIEVE → retrieval path; no chain budget
     ├─ Zero     → skip chain
     ├─ Depth    → H-42 scale_grc_budget; revise J-83 carrier
     ├─ Width    → J-86 ladder (after J-83)
     └─ EscalateOrAbstain → I-7 or I-8
     │
     ▼
tier = apply_lambda_override(auto_tier, U_lambda) // CC-4; floor/ceiling hold
                                                  // LLM-proposed tier discarded
     │
     ▼
OBSERVE-ONLY: emit chosen-tier vs verified-outcome
  until K-15 knee exists, do NOT gate
  then default-ON + *_DISABLE
```

**Forced:**

- Recall/factual TYPE cannot emit a reasoning-chain budget (negative arm of the smoke) (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:64`).
- `proxy<~0.05` → `EscalateOrAbstain`, not more Width (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:17`).
- Observe-only until K-15; a classifier that gates on paper quintiles is a FAIL (ID-5) (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:57,81`).

## 6. Grounding, receipts, gates

This organ **routes compute**. It does not verify claims. Confidence still comes from **external receipts** on the J-83 carrier (H-39 is a **proxy input**, not a substitute commit signal).

**Never allowed as a tier chooser:** LLM self-rating of difficulty; “the model asked for high effort”; window size / RULER fill; paper quintile bins (6K/7K overthinking knee is J-85’s cousin ban; here the ban is paper **difficulty bins**, ID-5); pass@K oracle at routing time (too expensive; IMPLICATION 26).

Selector hierarchy for **answers** remains receipt > vote-over-independent-samples (J-86, receipt-absent only) > LLM self-eval banned. J-84 must not introduce a fourth “the classifier is sure” grade.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| very-hard proxy | `< ~0.05` | COST axis (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:17,93`) |
| H-42 depth map | Concept=0, Phase=1, Step=2, BabyStep=3 | per node (`reasoning_budget.rs:131-137`) |
| H-42 scale | `1 + 0.25*d`, clamp `LOCKED_BUDGET_CEILING × base` | per role (`reasoning_budget.rs:24-26`) |
| I-26 cloud effort | depth≤1 `"auto"`, ≥2 `"expert"` | per cloud node (`budget_controller.rs:22-24`) |
| I-26 collapse | 85% sibling spend; plateau window | search envelope (`budget_controller.rs:8-14`) |
| I-26 flag | `ELAI_DYNAMIC_BUDGET_ENABLE` default-OFF (search methods dark) | process (`budget_controller.rs:51-52,294`) |
| H-42 kill-switch | `ELAI_REASONING_BUDGET_SCALE_DISABLE` | default-ON scaling (`reasoning_budget.rs:35-37`) |
| SSB | reuse `ReasoningBudgetScaled` (+Kind); net 0..+1 from snapshot 268/577 | W-9 |
| Classifier size | ~160 LOC new file | plan |
| Observe-only | mandatory before gating | GD-4 |

**Do not confuse with:** 2A `GrcBudget` 5/800 (engine cap); 3E window slots (RD-4: never a tier); J-85 overthinking **ceiling** (stop a run that already has a budget); K-15 **measurement** (this organ’s calibration source, not a runtime tier).

I-26 starter constants (`DEPTH_FACTOR`, `PLATEAU_DELTA_MIN`) are tagged `DEPTH_FACTOR_CALIBRATION_PENDING` — not paper, not yet ELAI-measured (`budget_controller.rs:42-71`). J-84 must not freeze those as the K-15 knee.

## 8. Roles and neighbors

Role-agnostic classifier. H-42 payload still records `role` (`ssb/payloads.rs:19472-19473`).

**Consumes:** J-83 carrier (the object whose reasoning is sized); D-10 TYPE; H-39 / C-45 / C-47 / J-38 / C-29 COST inputs; H-42 + I-26 budget targets; I-27 + H-9 lambda surface; K-15 knee (when it exists).

**Produces:** `ReasoningTier`; budget applied through H-42/I-26; observe-only tier-vs-outcome rows.

**Must not fork:** a second D-10; an LLM “pick your effort”; window-as-tier; collapsing Width into more Depth.

**Parallel-safe with:** J-85 (guard reads the budget this sets; disjoint seam). **Depends:** J-83. **Pairs with:** J-86 (Width consumer), K-15 (calibration) (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:8,114`).

## 9. Config / flags / defaults

- Classifier ships **OBSERVE-ONLY** (no gate) first. After K-15: default-ON behind `*_DISABLE` + allowlist (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:86-87`).
- I-26 keeps `ELAI_DYNAMIC_BUDGET_ENABLE` (experiment, default-OFF for search-node methods). Do not silently turn I-26 on by shipping J-84 (`budget_controller.rs:25-32,51-52`).
- Exact `ELAI_*` name for the classifier disable: **UNCERTAIN** (§16); plan says `*_DISABLE` pattern.
- Frontend: H-9 CloudPanel / cost-effort tile shows chosen tier + lambda dial; telemetry rides I-27 ConvergencePanel. Decide tile at edit (W-4); justified N/A if it rides existing panels (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:59-60`).
- Decisions doc: `workflow/reasoning/J-84-decisions.md` **must** state DC-5 explicitly (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:40,110`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `.elai_cc/crates/orchestrator/src/reasoning_classifier.rs` | CREATE ~160 LOC |
| GRC loop / `task_runner` caller | MODIFY ~30 LOC invoke + set budget |
| `reasoning_budget.rs` `estimate_serial_depth` / `scale_grc_budget` | EXTEND, do not rewrite |
| `routing/budget_controller.rs` `BudgetController` | EXTEND I-26 target |
| `grounding_mode.rs::select_grounding_mode` | TYPE axis |
| `abstention_score.rs` | COST input |
| `intent_pair.rs::IntentPair` | COST input (count) |
| C-45/C-47 case-base, J-38 coverage | COST inputs (DARK/PLANNED) |
| `ssb.rs` + `action_type_lock.rs` + golden | only if Kind added |
| `crates/contracts/tests/j84_reasoning_classifier_contract.rs` | CREATE ~70 LOC |
| `workflow/reasoning/J-84-decisions.md` | CREATE ~25 LOC |
| `h42_serial_depth_budget_contract.rs` / `i26_dynamic_thinking_budgets_contract.rs` | existing substrates |

## 11. Journal / SSB events

Prefer-reuse **`ReasoningBudgetScaled`** (SCHEMA 220 snapshot). Plan Kind add (decide at edit): `TierRouted` / `EffortOverridden` (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:63`).

Live payload fields listed in §4 (`ssb/payloads.rs:19467-19491`). Live `kind` strings today are `depth_bound` / `access_bound` / `neither` (`reasoning_budget.rs:107-112`) — those are **failure-class** labels, not J-84 tiers. Rebuild: either add Kind arms or encode the tier in an existing field **without** overloading `FailureClass`. Which: **UNCERTAIN** (§16).

GD-2: tier decisions recomputable from the journal (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:90`). Do not rewind schema to 268/577.

I-27 `ConvergenceCurveMeasured` is the observe-only outcome curve J-84 compares against; K-15 composes that harness (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:53-56`).

## 12. Tests and observables

Named contract `j84_reasoning_classifier_contract.rs` (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:92-93`):

| Id | Assert |
|---|---|
| (a) | trivial → `Zero` |
| (b) | recall → Retrieve, **never** a reasoning chain |
| (c) | medium → `Depth` |
| (d) | hard → `Width` |
| (e) | very-hard (`proxy<0.05`) → `EscalateOrAbstain` |
| (f) | lambda override shifts tier **within** floor/ceiling |
| (g) | LLM-proposed tier is **ignored** |
| (h) | empty/degenerate proxy → fail-closed **safe** (lower-cost or abstain) |

**W-1 smoke (R10):** drive the real reasoning entrypoint on (a) trivial (b) factual-recall (c) medium multi-step (d) hard branching. Each routes to the right tier. Negative: recall→chain MUST NOT happen; LLM-proposed tier ignored (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:61-64,85`).

**DETERMINISTIC_CHECK:** `Zero|Depth|Width|EscalateOrAbstain|difficulty|select_grounding_mode` present; `llm.*tier|model.*picks` count 0; window-as-tier-output count 0; `production_caller_check` 0 DARK; `J-84-decisions.md` exists (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:66-74`).

Adversarial: `--flat-budget` cousin is H-42’s R2, not this organ; a classifier that emits Width for recall is a FAIL.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Two axes: TYPE from a locked D-10-style predicate (reuse, not rebuild); COST from a deterministic proxy over abstention/case-base/coverage/intent-count. **No LLM in the function.**
- Closed tiers `{Zero, Depth, Width, EscalateOrAbstain}` plus a retrieve-not-chain TYPE outcome. **WINDOW is not a tier.**
- Routing table in §3, including recall→retrieve and trivial→zero.
- Depth wires a serial-depth budget (H-42 shape). Width is parallel candidates (J-86), not more revision. Very-hard escalates or abstains.
- Manual lambda may override **inside** a deterministic floor/ceiling.
- OBSERVE-ONLY emit tier vs verified outcome **before** any gating. Knee is **measured per (model, task-type)** (K-15), never a paper quintile.
- LLM-proposed effort is ignored.
- H-42 / I-26 / D-10 composed, not rewritten.

A prompt “think longer if needed”, a window-size heuristic, or I-26’s `"auto"|"expert"` string **alone** is a cousin, not this organ.

## 14. Non-goals / banned designs

- Re-author H-42, I-26, D-10 (GD-1) (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:58`).
- LLM-picked tier (TD-3).
- Window-size as tier (RD-4, RULER 2404.06654) (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:144-147`).
- Paper quintile / 6K / 7K / test-time-compute bins hard-coded as ELAI thresholds (ID-5) (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:57`).
- Gating before observe-only calibration (GD-4).
- Routing recall to a reasoning chain (confabulation) (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:64`).
- Hiding DC-5 (document depth-vs-width-vs-window explicitly) (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:110`).
- Treating I-26 `"auto"|"expert"` as the four-tier enum.
- Shipping J-84 before J-83 (it sizes that carrier) (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:111`).

## 15. Source citations

Primary: `docs/thefinalglance/J-plan/J-84_grcv3_reasoning_effort_classifier.md` (full file). Grounding map: `J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md` GA-2, GS-7/GS-8/GS-4, DC-5, R-2. Index: `J-plan/J-SUMMARY_INDEX.md:1871-1883`. Feature map: `ELAI-FEATURE-MAP.md:656-667`. Context: `ELAI-CONTEXT-MANAGEMENT-DEEP-DIVE.md:262-264`. Neighbor: `reasoning-architecture/H-42-serial-depth-reasoning-budget.md`. Live: `reasoning_budget.rs`, `routing/budget_controller.rs`, `grounding_mode.rs:110`, `abstention_score.rs`, `intent_pair.rs:86-97`, `ssb/payloads.rs:19467-19491`. Design: `workflow/reasoning/reasoning-engine-grc-v3-2026-06-25.md` §5 + IMPLICATION 3/5/24/25/26 + ROUND-3 ADD-1 (as cited by the J-84 plan).

arXiv **[EXTERNAL, WHY]:** test-time-compute 2408.03314; zero-shot-CoT 2205.11916; RULER 2404.06654 (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:105`, `J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:85-96`).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- COST proxy formula: linear vs threshold; weights of H-39 vs C-45 vs J-38 vs C-29 (`J-plan/J-84_grcv3_reasoning_effort_classifier.md:109`).
- Numeric bins for trivial / easy / medium / hard besides very-hard `~0.05`. Those bins are K-15-measured; until then observe-only.
- Exact map D-10 `GroundingMode` / live cousin enum → `{Reason, Retrieve, Validate}`.
- When C-45/C-47 are DARK and J-38 is PLANNED, which subset of the proxy is legal at first ship (fail-closed safe vs wait).
- `U_lambda` type, range, and how it “shifts the tier” (adjacent-tier only vs continuous mix).
- Whether Retrieve is a fifth `ReasoningTier` or a TYPE short-circuit before the enum (`J-plan` tests say `recall→Retrieve` **and** the enum is `{Zero, Depth, Width, EscalateOrAbstain}`).
- Classifier kill-switch env name; how it composes with `ELAI_DYNAMIC_BUDGET_ENABLE` and `ELAI_REASONING_BUDGET_SCALE_DISABLE`.
- Whether `ReasoningBudgetScaled.kind` grows `TierRouted` or a new field is required vs overloading `FailureClass` strings.
- Function name `classify_reasoning_effort` is inferred from produces_symbols; the plan names the **file** `reasoning_classifier.rs`, not a single fn.
- How local-lane “impose structured decomposition” (R-4) interacts with trivial→Zero (must the engine still emit Observe, or skip the chain entirely?).
