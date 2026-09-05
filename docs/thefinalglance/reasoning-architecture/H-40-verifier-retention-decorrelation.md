# Verifier Retention by Decorrelation (H-40)

- **ELAI ids:** H-40 / RG-40 / RD-2 / RS-9 / RS-13 / H-002/H40-001..H40-019
- **Spec status (as designed):** PARTIAL (feature-map 1.2); original plan PLANNED Track-6 extension of E-6
- **Crate / module (spec):** `crates/orchestrator/src/iso_budget_harness.rs` (`unique_catch_rate`, `IsoBudgetReport.insufficient_contribution`); consume `verification_result.rs::VerifierKind`; reuse shipped `iso_budget_measured` SSB row (no new variant)
- **Purpose of this file:** enough architecture to rebuild the both-low prune predicate — unique-catch + decorrelation axis on the existing iso-budget harness. Not a status ledger.

## 1. What it is

**Decorrelation-keyed verifier/role retention** extends the shipped E-6 iso-budget harness so a role is **prunable only when its marginal lift AND its unique-catch are both low**. Per-role `unique_catch_rate(R)` is the fraction of ensemble **failures** (items the consensus got wrong) that **R alone got right**. `decorrelation_score(R) = 1 - mean pairwise error-correlation` of R with the other roles, **conditional on ground truth**. `insufficient_lift` stays for compatibility; only `insufficient_contribution = low_lift AND low_unique_catch` recommends prune (`H-plan/H-40_decorrelation_keyed_verifier_retention.md:11-13`, `:53-60`, `H-REASONING-GROUNDING-MAP.md:99-102`).

## 2. Why it exists

`iso_budget_harness.rs` decides whether an agent role earns its budget by **marginal lift**. It emits `insufficient_lift` (`iso_budget_harness.rs:93`); the E-6/E-10 deletion gate prunes a role whose standalone accuracy lift is `< ~1 sigma` (`H-plan/H-40_decorrelation_keyed_verifier_retention.md:5-13`, `:34-37`). Weaver 2506.18203: a verifier **below** the generator’s accuracy still raises ensemble accuracy if errors are **decorrelated** (conditionally independent given Y). A lift-only gate therefore **wrongly prunes** the decorrelated low-accuracy verifier/auditor that is catching failures the rest miss (RD-2). If this organ is missing, the harness deletes the role Weaver says is most valuable. If it is faked (a diversity slogan without unique-catch math), lift-only prune still fires. Honest ceiling (GD-5): an ensemble **approaches, never exceeds**, the Pass@K oracle verifier (`H-plan/H-40_decorrelation_keyed_verifier_retention.md:25-32`, `:69-71`, `H-REASONING-GROUNDING-MAP.md:99-102`).

## 3. How it works (runtime)

1. **Who calls it.** The live iso-budget / `elai measure`-style pass over a role set — not a `#[cfg(test)]` caller (`H-plan/H-40_decorrelation_keyed_verifier_retention.md:81-83`, `:111`).
2. **Input.** Per-role outcome vectors the harness **already collects**: role R’s pass/fail per eval item **alongside ground truth** (`:53-56`).
3. **`unique_catch_rate(R)` (deterministic integer/ratio math).** Let F = items the **ensemble consensus got wrong**. Unique catch = items in F that R got **right** and **no other role** got right, divided by `|F|` (`:53-56`). Seed-stable; no wall-clock.
4. **`decorrelation_score(R)` (deterministic).** `1 - mean pairwise error-correlation` of R with the other roles, **conditional on ground truth** (`:55-56`).
5. **No-label degradation.** On tasks with **no ground truth**, the metric degrades to “disagreement diversity” (weaker proxy). **Flag it as such**; do not assert decorrelation without labels (`:153-154`).
6. **Extend `IsoBudgetReport`.** Add per-role `unique_catch_rate` + `decorrelation_score`. Keep `insufficient_lift`. Add `insufficient_contribution = low_lift AND low_unique_catch` (both below LOCKED floors). **Only** `insufficient_contribution` recommends pruning (`:58-60`).
7. **Deletion gate.** E-6/E-10 role-deletion gate reads `insufficient_contribution`, **not** `insufficient_lift` alone (`:83`). A lift-only prune is a FAIL (RD-2) (`:85`).
8. **Measured row (no new `action_type`).** Emit the extended fields on the **existing** `iso_budget_measured` payload (GD-1). Numbers recomputable from per-role outcome vectors under `ELAI_SEED` (GD-2) (`:62-64`, `H-REASONING-GROUNDING-MAP.md:145-146`).
9. **Dashboard.** Unique-catch / decorrelation columns next to the lift flag in the iso-budget panel (`commands_e6.rs` + Svelte). A low-lift-but-decorrelated role is shown **RETAINED** with its unique-catch number (`H-plan/H-40_decorrelation_keyed_verifier_retention.md:66-67`, `:89-91`).
10. **R2 broken arm.** `--prune-on-lift-only` **must** prune the decorrelated role (and flag both R and C). If the axis does not change the decision, it is vacuous (`:87`, `:97-98`).
11. **Fail-closed / P0.** No second harness. No re-implementation of `iso_budget_harness` (GD-1 P0). blake3; no `.unwrap()`/`.expect()` in prod (`:86-88`).

The model may be **a** verifier under test. It may **not** decide keep/cut, invent unique-catch, or self-score decorrelation.

## 4. Types, enums, objects

Copy from `H-plan/H-40_decorrelation_keyed_verifier_retention.md:35-41`, `:53-60`. Do not invent fields.

```
IsoBudgetReport {                    // E-6 SHIPPED; EXTEND
  // existing:
  insufficient_lift: bool,           // iso_budget_harness.rs:93; KEEP
  // H-40 adds, per role:
  unique_catch_rate: f64,            // |F ∩ {R-only-right}| / |F|
  decorrelation_score: f64,          // 1 - mean pairwise error-corr | Y
  insufficient_contribution: bool,   // low_lift AND low_unique_catch
}

fn unique_catch_rate(R, outcomes, y) -> f64
fn decorrelation_score(R, outcomes, y) -> f64

VerifierKind { Deterministic, Intrinsic, Ppm, ... }  // D-3; REUSE
OrchContext.available_verifiers
has_cross_family_verifier

LOCKED floors { low_lift, low_unique_catch }         // numeric values UNCERTAIN
```

`iso_budget_measured` SSB row: **same variant**, extended payload. If a payload field forces a SCHEMA touch: baseline+0 variants, lockstep the const (`H-plan/H-40_decorrelation_keyed_verifier_retention.md:128`).

## 5. Control flow / state machine

**Owner of the loop:** the shipped iso-budget harness. H-40 is an **axis + predicate**, not a new FSM.

**Keep/cut table:**

| Role | lift | unique-catch | `insufficient_lift` | `insufficient_contribution` | action |
|---|---|---|---|---|---|
| R (Weaver-class) | low | high | may be true | **false** | **retain** |
| C (control) | low | low | true | **true** | **prunable** |
| both under `--prune-on-lift-only` | low | * | true | (ignored) | **both flagged** |

**Forced:** prune recommendation **only** on both-low (`:59-60`, `:112`).

**No auto-prune at first ship** until floors are measured (GD-4) — see §9 / §16.

## 6. Grounding, receipts, gates

**Receipt:** per-role outcome vectors + ground-truth labels in the journal. Unique-catch / decorrelation **recompute** from those vectors (GD-2). Without labels, do not emit a decorrelation claim (`:153-154`).

**Never allowed as a keep/cut signal:** standalone accuracy alone; LLM self-eval of “I catch different bugs”; random seed diversity as a substitute for conditional error correlation; Pass@K advertised as exceeded (`H-REASONING-GROUNDING-MAP.md:99-102`).

**Honest ceiling:** Pass@K oracle. Decorrelation **raises the floor**; it does **not** make verification sound. Final trust still routes to a sound external checker (H-41 / D-11) or refusal (D-16) (`H-plan/H-40_decorrelation_keyed_verifier_retention.md:157-158`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| existing lift flag | `insufficient_lift` (~1 sigma; E-6) | per role |
| unique-catch | fraction of consensus failures R-only-right | per role |
| decorrelation | `1 - mean pairwise error-corr \| Y` | per role |
| prune predicate | both-low, LOCKED floors | per role |
| new `action_type` | **none** | GD-1 |
| hash | blake3; `blake2\|sha2` count 0 in harness | crate |
| auto-prune | not until FP/over-retain measured on ≥3 real runs (GD-4) | deletion gate |

Numeric `low_lift` / `low_unique_catch` floors: **UNCERTAIN** (tuning knobs; set with measured FP/over-retain) (`:155-156`).

## 8. Roles and neighbors

**Consumes:** E-6 `IsoBudgetReport` + `insufficient_lift` (RS-9); D-3 `VerifierKind` / `available_verifiers` (RS-13); B-41/C-12 per-level auditor roles (`:38-40`). **Produces:** `unique_catch_rate`, `insufficient_contribution` (and `decorrelation_score` on the report). **Feeds:** VERIFICATION AGGREGATOR; E-6/E-10 deletion gate (`ELAI-FEATURE-MAP.md:627-628`).

**Must not fork:** a second iso-budget harness; pruning on lift alone; advertising “catches everything.”

**Parallel-safe with:** H-39, H-41, H-42 (disjoint files) (`:19`). Downstream: H-41 measures how often a claim can reach a **sound** checker (stronger than any decorrelated LLM ensemble); H-39 fixes the reward retained verdicts feed (`:160-162`).

## 9. Config / flags / defaults

- Default-ON: decorrelation axis computed on **every** iso-budget run; kill-switch only (`:110`).
- Kill-switch identifier: **UNCERTAIN** (same class of gap as H-39).
- Floors LOCKED after GD-4 calibration on ≥3 real iso-budget runs **before** the deletion gate trusts `insufficient_contribution` to auto-prune (`:155-156`).
- Observe-then-enforce implied by GD-4; the plan does not name an observe-only flag — **UNCERTAIN** whether auto-prune is off until calibration.
- Decision doc: `workflow/reasoning/H-40-decisions.md` (`:69-71`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `iso_budget_harness.rs::unique_catch_rate` | `H-plan/H-40_decorrelation_keyed_verifier_retention.md:75` |
| `IsoBudgetReport.insufficient_contribution` | `:76` |
| consume `IsoBudgetReport` (extend) | `:78` |
| consume `VerifierKind` | `:79` |
| reuse `iso_budget_measured` | `:80` |
| `commands_e6.rs` + iso-budget Svelte columns | `:141` |
| `crates/contracts/tests/h40_decorrelation_retention_contract.rs` | `:133` |
| fixtures `crates/contracts/tests/fixtures/iso_budget/` | `:136` |
| `workflow/reasoning/H-40-decisions.md` | `:143` |

## 11. Journal / SSB events

**No new variant.** Extend `iso_budget_measured` payload with unique-catch / decorrelation fields (`H-REASONING-GROUNDING-MAP.md:145-146`, `H-plan/H-40_decorrelation_keyed_verifier_retention.md:62-64`).

If SCHEMA lockstep is required: baseline+0 variants (`:128`).

## 12. Tests and observables

Contract (`:132-136`):

| Id | Assert |
|---|---|
| (a) | `unique_catch_rate` correct on fixture |
| (b) | low-lift / high-unique-catch **not** `insufficient_contribution` |
| (c) | low-lift / low-unique-catch **is** `insufficient_contribution` |
| (d) | `--prune-on-lift-only` flags **both** (R2) |
| (e) | `ELAI_SEED` twice → identical metrics |

**Observable effect** (`:92-98`): frozen ensemble, role R low-lift/high-unique-catch, control C low/low, no env override → row carries high unique-catch(R) / low unique-catch(C); R retained; C prunable; broken arm flags both.

W-1: `--run-trigger` PASS. `cargo-green != DONE` (`:109`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Same iso-budget harness (extended), not a cousin harness.
- Per-role `unique_catch_rate` = fraction of consensus failures R-alone-right.
- `decorrelation_score` = `1 - mean pairwise error-corr` given Y, or **explicitly flagged** disagreement-diversity when Y is absent.
- `insufficient_lift` preserved; prune recommendation is `insufficient_contribution` = both-low.
- Decorrelated low-lift role **retained**; low/low **prunable**.
- No new SSB `action_type`; payload extension of `iso_budget_measured`.
- `--prune-on-lift-only` prunes the Weaver-class role (R2).
- Pass@K named as the honest ceiling; no “soundness” claim.

A “keep diverse judges” prompt, or pruning on accuracy sigma alone, is a cousin, not this organ.

## 14. Non-goals / banned designs

- New harness / re-implementation of `iso_budget_harness` (GD-1 P0) (`:86`).
- Prune on marginal lift alone (RD-2 FAIL) (`:85`).
- Asserting decorrelation without labels (`:153-154`).
- Claiming the ensemble beats Pass@K or is sound (`:69-71`, `:157-158`).
- Replacing H-41 sound-checker coverage or D-16 refusal with weak-ensemble vote.
- blake2 / sha2 (`:104`).

## 15. Source citations

Primary: `docs/thefinalglance/H-plan/H-40_decorrelation_keyed_verifier_retention.md` (full file). Grounding: `H-REASONING-GROUNDING-MAP.md:54-56`, `:64-65`, `:75-78`, `:99-102`, `:126-128`, `:145-146`. Doctrine: `H-plan/H-0_INDEX.md:340-343`. Index: `H-plan/H-SUMMARY_INDEX.md:662-670`. Overview: `ELAI-SPEC-OVERVIEW.md:248-249`. Feature map: `ELAI-FEATURE-MAP.md:627-628`. Exhaustive: H-40 section ~25094+. Paper: Weaver 2506.18203 (confirm 87.7% headline before citing).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Numeric LOCKED floors for `low_lift` and `low_unique_catch` (`H-plan/H-40_decorrelation_keyed_verifier_retention.md:60`, `:155-156`).
- Exact pairwise correlation estimator (Pearson vs other); spec says “mean pairwise error-correlation … conditional on ground truth” (`:55-56`).
- Tie-break when two roles uniquely catch the same consensus failure (definition says “R alone”) (`:53-56`).
- Kill-switch name / polarity.
- Whether deletion-gate auto-prune is off until GD-4 calibration, or default-ON immediately (`:110` vs `:155-156`).
- `VerifierKind` full variant list beyond `{Deterministic, Intrinsic, Ppm, ...}` (`:38`).
