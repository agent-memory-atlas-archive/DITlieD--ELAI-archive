# One counterfactual / ablation engine (N-9)

- **ELAI ids:** N-9 / TIER 3 / W-20+W-27 / ND-5 / WAVE 6 / depends N-1
- **Spec status (as designed):** PLANNED (BUILD). Unifies four attribution engines + four ablation harnesses (`N-plan/N-9_counterfactual_ablation_engine.md:1-12`, `ELAI-FEATURE-MAP.md:3293-3298`, `N-plan/N-0_INDEX.md:187-191`).
- **Crate / module (spec):** `crates/orchestrator` counterfactual engine core + eight thin adapters at the existing call sites
- **Purpose of this file:** enough architecture to rebuild one drop-one/inject-one journal engine with target as parameter. Not a status ledger.

## 1. What it is

The **counterfactual / ablation engine** is **one** drop-one / inject-one replay over the SSB. **Target is a parameter** (`role` / `verifier` / `ranker` / `rule` / `memory-entry` / `case`). Eight **thin** adapters (ND-5) point existing units at that core instead of each re-implementing replay (`N-plan/N-9_counterfactual_ablation_engine.md:8-12`, `N-plan/N-SUMMARY_INDEX.md:180-193`):

**Attribution side:** C-45 CANTANTE per-role credit (drop-a-role); I-28 DAG credit (winner vs sibling); J-105 per-memory-entry causal ROI (counterfactual replay); J-109 AD-4 pre-write counterfactual replay.

**Ablation side:** E-6 iso-budget role audit (delete-a-role unless >1-sigma lift); E-10 critic ablations; H-40 verifier retention by decorrelation; J-75 RRF ablation (drop-a-ranker).

The delta is scored against **N-1 outcome labels** (W-11), the same ground truth every other measurement uses (`:11`, `N-plan/N-0_INDEX.md:110-113`).

This is **not** L-5 (interventional do-test on a **model claim** — HG-1). L-5 asks “does intervening on A change B?” N-9 asks “if we drop this **harness factor**, how do **journal labels** move?” Do not merge them (`L-plan/L-SUMMARY_INDEX.md:34-44` vs `:8`).

## 2. Why it exists

The same subtle measurement — re-run / re-decide with one factor removed or injected, score the delta off journal outcomes — is specified **eight** times (`:5-6`, interlace W-20 + W-27). If this organ is missing, eight hard-to-get-right implementations drift (I-28 already forbids Shapley while a cousin might reintroduce it; H-40 already forbids lift-only prune). If it is faked (eight replay loops behind one facade, or a fat adapter that re-scores locally), that is an **ND-1 fork in disguise** (`N-plan/N-0_INDEX.md:99-101,114-116`).

K-18 reuse-ON/OFF arms are named as the **same shape**, not a ninth adapter (`:16`). A ninth attribution target is **excluded** (`:12`).

## 3. How it works (runtime)

1. **Who calls it.** Production ablation seams: a live **role / verifier / ranker** ablation through the **one** engine (Tier-1). A `#[cfg(test)]` caller never satisfies R7 (`:33-35`). The model does not pick the target or compute the delta.
2. **Input.** A **target** (which factor to drop or inject) + the SSB event stream + **N-1 outcome labels** (`:30-32`).
3. **Core (deterministic) — the only place replay+scoring live:**
   1. Bind target (adapter **names** it; core **does** the work — ND-5).
   2. Replay / re-decide the journal **with that factor removed or injected**.
   3. Score **delta** against N-1 labels / the journal (GD-2). **Not** against LLM judgment, adapter-local metrics, or a private store (`:39`, ND-4).
   4. If the ablation **feeds a promotion**, N-3 deflation applies (trial-count); N-9 does not mint a promotion threshold (`:17`, `N-plan/N-0_INDEX.md:110-113`).
4. **Adapters (thin, dispatch only).** Each of the eight units becomes a parameter/dispatch wrapper: which target id, which drop vs inject, which existing call site. **No** business logic in the adapter (`:12,38`, `N-plan/N-0_INDEX.md:114-116`).
5. **Tier-1 (ship now):** engine core + **3–4 adapters** proving parameterization: **role, verifier, ranker** (`:19-20`).
6. **Tier-2 (defer):** memory / case / **DAG-credit** adapters, after those subsystems stabilize (`:21`). I-28’s `DagCreditAssigner` therefore stays the live credit walk until its adapter lands — cite, do not merge I-28 into this organ early (`I-28-dag-credit-assignment.md` §8).
7. **Commit.** `+counterfactual_scored` Pattern-2 Kind `Role | Verifier | Ranker | Memory | Case` (`:44`, `N-plan/N-0_INDEX.md:277`).
8. **ND-2.** Unifying eight paths **requires** a retirement map: each superseded replay → adapter in the unifier; old path behind `ELAI_*_LEGACY` one release; user-confirmed removal. Two live cores is the split-brain this letter kills (`N-plan/N-0_INDEX.md:103-107`). N-9’s source file does **not** enumerate that map — **UNCERTAIN** (§16); rebuild must still **ship** one (ND-2 is letter-wide).
9. **Fail-closed.** Adapter that carries scoring/replay **fails ND-5**. Delta scored off anything but N-1 labels / journal **fails** (`:37-39`).
10. **The model may be a role/verifier under test.** It may **not** write the delta, choose drop-vs-inject, or keep a private ablation loop.

## 4. Types, enums, objects

Copy from `N-plan/N-9_counterfactual_ablation_engine.md:8-12,44`. Do not invent targets.

```
// TARGET as parameter (plan list):
enum AblationTarget {
  Role,
  Verifier,
  Ranker,
  Rule,            // named in TARGET list; NO matching Kind — see §16
  MemoryEntry,     // Kind: Memory
  Case,
}

// drop-one XOR inject-one — both named; API enum UNCERTAIN (§16)
fn replay_with_factor_removed_or_injected(target, journal) -> ...
fn score_delta(baseline_labels, ablated_labels) -> ...   // N-1 labels ONLY

// eight THIN adapters (name the target, delegate):
C-45  drop-a-role credit          -> Role
I-28  DAG credit                  -> (DAG-credit adapter DEFERRED Tier-2)
J-105 memory-entry causal ROI     -> Memory   (Tier-2)
J-109 AD-4 pre-write replay       -> Case     (Tier-2)
E-6   iso-budget delete-a-role    -> Role
E-10  critic ablations            -> (critic → Role or Verifier? UNCERTAIN §16)
H-40  verifier retention          -> Verifier
J-75  RRF drop-a-ranker           -> Ranker

counterfactual_scored.Kind { Role, Verifier, Ranker, Memory, Case }
```

Do **not** add Shapley/SHARP (I-28 declined) (`I-28-dag-credit-assignment.md:10`). Do **not** add an L-5 claim-intervention target (wrong mechanism).

**H-40 metrics** (`unique_catch_rate`, `decorrelation_score`, `insufficient_contribution`) are specified on the iso-budget harness (`H-40-verifier-retention-decorrelation.md` §3). Under ND-5 those formulas belong in the **core** as scoring modes, or H-40 remains a fat adapter — **UNCERTAIN** (§16). Rebuild: **one** scoring core; do not copy unique-catch into a second file.

## 5. Control flow / state machine

**Owner of the loop:** orchestrator core. Adapters dispatch; core replays; N-1 labels score.

```
call site (role | verifier | ranker | …)
  → thin adapter names AblationTarget
  → core: snapshot journal baseline (N-1 labels)
  → core: drop-one OR inject-one the named factor
  → core: re-decide / replay
  → core: delta vs N-1 labels
  → if feeds promotion: N-3 deflation
  → emit counterfactual_scored(Kind)
```

**Forced:**
- Tier-1 production path must exercise **role, verifier, and ranker** through the **same** core (`:19,42-45`).
- I-28 / J-105 / J-109 adapters **do not** ship in Tier-1 (`:21`).
- No LLM inside score_delta (`:39`).
- Unifier defaults ON behind the retire switch of the old paths (N wiring W-2 caveat), old path `ELAI_*_LEGACY` one release (`N-plan/N-0_INDEX.md:67-69`).

N-0_INDEX says “**five** thin adapters” (`:188`) while N-9 and the feature map say **eight** (`:8-11`, `ELAI-FEATURE-MAP.md:3293-3296`). Rebuild the **eight named consumers**; Tier-1 ships **3–4**. Do not invent a nine-adapter table to split the difference.

## 6. Grounding, receipts, gates

The **receipt** is the journal + N-1 labels. The delta is GD-2 recomputable. LLM narrative of “this role helped” is never the score.

**Selector hierarchy:** N-1 outcome labels > journal events > **not** adapter-local heuristics > **not** LLM.

**Never allowed as a confidence / delta signal:** Shapley over a missing tree (I-28); lift-only prune without unique-catch when the target is H-40’s verifier question (`H-40-verifier-retention-decorrelation.md` §2); paper AUROC; a second calibration store (ND-4).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| cores | **1** replay+score engine | organ |
| adapters Tier-1 | 3–4 (role, verifier, ranker) | ship-now (`:19`) |
| adapters named | 8 | full unifier (`:8-11`) |
| ninth target | **0** | excluded (`:12`) |
| Kind | Role / Verifier / Ranker / Memory / Case | SSB (`N-plan/N-0_INDEX.md:277`) |
| schema | `+counterfactual_scored` baseline+1 | (`:277`) |
| promotion | N-3 deflation when ablation feeds promotion | (`:17`) |
| labels | N-1 spine only (ND-4) | (`:11,39`) |
| E-6 prune threshold (legacy) | >1-sigma lift (E-6); H-40 replaces with both-low unique-catch | compose, don’t restated-fork (`:10`) |

Replay cost / max journal window / iso-budget item count are owned by E-6/H-40/I-28 — not restated here (GD-1).

## 8. Roles and neighbors

**Consumes as adapters (do not re-implement replay):** C-45, I-28, J-105, J-109, E-6, E-10, H-40, J-75 (`:31`). **Consumes as labels:** N-1. **Consumes as stream:** SSB (`:32`).

**Produces:** `counterfactual_scored` rows; retirement map (ND-2, required at letter grain).

**Must not fork:** eight replay implementations; fat adapters; a ninth attribution target; merging L-5 do-tests into this engine; I-28 Shapley; a private calibration store.

**I-28 pack file:** “N-9 later unifies attribution engines including I-28 — cite, do not merge” (`I-28-dag-credit-assignment.md` §8). This file **is** that unifier; I-28 adapter is **Tier-2**.

**Frontend:** reuse existing attribution drawers, or N/A justified (`:25,41`).

## 9. Config / flags / defaults

- N-9 names **no** new ENABLE flag in its source. Letter-wide W-2: unifier that **replaces** live paths defaults ON behind the retire switch; old path `ELAI_*_LEGACY` one release (`N-plan/N-0_INDEX.md:67-69`).
- I-28 live `ELAI_DAG_CREDIT_ENABLE` default-OFF stays until the DAG-credit adapter lands (`I-28-dag-credit-assignment.md` §3). Do not “fix” that by silently forcing I-28 through N-9 in Tier-1.
- No `.unwrap()` / `.expect()` on prod (`:40`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/orchestrator` counterfactual engine core | drop-one/inject-one + delta (`:24,28`) |
| eight call sites as adapters | C-45 / I-28 / J-105 / J-109 / E-6 / E-10 / H-40 / J-75 (`:24,29`) |
| N-1 outcome labels | score sink (`:32`) |
| `counterfactual_scored` | SSB (`:44`) |

Exact rust module path is **not** named beyond “counterfactual engine” (`:24`). Do not invent `counterfactual.rs` as if the spec pinned it — **UNCERTAIN** filename (§16).

## 11. Journal / SSB events

From `:44` and `N-plan/N-0_INDEX.md:277`:

```
counterfactual_scored {  // Pattern-2; baseline+1
  Kind: Role | Verifier | Ranker | Memory | Case
}
```

Payload fields (target id, drop vs inject, baseline, ablated, delta, label source) are unspecified (§16). Kind `Rule` is **absent** even though TARGET lists `rule`.

## 12. Tests and observables

From `:42-45,37-39`:

| Assert | Notes |
|---|---|
| live role **or** verifier **or** ranker ablation through the **one** engine | production seam, not unit-test-only |
| `+counterfactual_scored` with matching Kind | Role / Verifier / Ranker (Tier-1) |
| delta computed by the **core** from journal / N-1 labels | GD-2 |
| adapter carrying business logic | **FAILS ND-5** |
| eight separate replay implementations | **FAILS** stub_ban |
| `#[cfg(test)]` as sole caller | **FAILS** R7 |

N-9’s source does **not** name a `n9_*_contract.rs`. Rebuild from observable + stub_ban.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- **One** drop-one/inject-one replay+score core over the journal. Target is a parameter.
- Eight named consumers are **thin adapters** (ND-5). Core owns replay and scoring.
- Deltas use **N-1 labels** / journal only. Promotion uses N-3 deflation, not a private threshold.
- `counterfactual_scored` Kind `Role | Verifier | Ranker | Memory | Case`.
- Tier-1: core + role/verifier/ranker adapters on a **live** seam. I-28/J-105/J-109 adapters deferred.
- ND-2 retirement map for superseded replay paths (letter-wide, even if N-9 text omits the table).
- No ninth target. No L-5 merge. No Shapley. No fat adapter.

Eight independent `replay()` functions behind a match statement is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Re-implementing replay eight times (`:12,37`).
- Fat adapters (ND-5) (`:12,38`).
- Delta off anything but N-1 / journal (`:39`).
- A ninth attribution target (`:12`).
- L-5 interventional claim do-test as an adapter (wrong organ).
- Shapley/SHARP (`I-28-dag-credit-assignment.md:10`).
- Private calibration store (ND-4).
- Shipping memory/case/DAG adapters in Tier-1 (`:21`).
- Two live cores without ND-2 (`N-plan/N-0_INDEX.md:103-107`).

## 15. Source citations

Primary: `docs/thefinalglance/N-plan/N-9_counterfactual_ablation_engine.md` (full file). Index: `N-plan/N-SUMMARY_INDEX.md:180-193`. Doctrine: `N-plan/N-0_INDEX.md:99-116,187-191,277`. Feature map: `ELAI-FEATURE-MAP.md:3293-3298`. Overview: `ELAI-SPEC-OVERVIEW.md:419-428`. Neighbors: `I-28-dag-credit-assignment.md`, `H-40-verifier-retention-decorrelation.md`. Interlace: W-20 (four counterfactual engines), W-27 (four ablation harnesses) (`:48`).

`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` has **no** N-9 atom block. Do not mint N9-E### ids here.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Exact rust filename/module for the core (`:24` “counterfactual engine”).
- Drop-one vs inject-one as one enum vs two entry points (`:8` names both).
- TARGET includes `rule` but Kind does not (`:8` vs `:44`).
- N-0 “five thin adapters” vs N-9/feature-map **eight** (`N-plan/N-0_INDEX.md:188` vs `:8-11`).
- E-10 critic → Role vs Verifier Kind.
- Where H-40 unique-catch / decorrelation math lives under ND-5 (core scoring mode vs adapter).
- ND-2 per-path retirement table (letter-required; N-9 file silent).
- `counterfactual_scored` payload fields; sign convention of delta.
- Whether K-18 reuse-ON/OFF is a future adapter or only a cited shape (`:16`).
- Replay window (full journal vs last task vs N-1 corpus slice).
- Named contract-test file (none in source).
