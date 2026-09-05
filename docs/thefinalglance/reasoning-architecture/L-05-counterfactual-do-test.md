# Counterfactual do-test on model claims (L-5)

- **ELAI ids:** L-5 / L-T1 / NN-4 / HG-1 / WAVE 2 KEYSTONE / CANT-LIE
- **Spec status (as designed):** PLANNED (`L-plan/L-5_counterfactual_do_test_on_model_claims.md:3`). Live `claim/counterfactual.rs` is a **cousin fill**: planner + hash-compare + unfired wired predictor exist; **live intervention execution and F-4 hash injection are out of scope for the initial landing** (`:256`) and W-1 is unmet until they land.
- **Crate / module (spec):** `crates/orchestrator/src/claim/counterfactual.rs`; production seam is `claim/roundtrip.rs` `score_claim_risk_wired_with_samples` predictor slice (`L-plan/L-5_counterfactual_do_test_on_model_claims.md:87-104, 98-104`).
- **Purpose of this file:** enough architecture to rebuild the interventional do-test that alone may label a claim causal. Not a status ledger.

## 1. What it is

The **counterfactual do-test** is the CGG’s causal core. When a model emits a Causal/Behavior claim (“A causes/fixes B”, “this patch fixes the timeout”), the harness **runs an intervention** — revert only A / disable the flag / patch only the claimed function / add a trace to the claimed path — then re-runs an **un-forgeable** outcome (F-4 blake3 stdout-hash, C-28/I-20 mutation, or C-29 metamorphic relation) and compares. A claim is `causal_supported` **only** when the interventional outcome **moves as claimed** (HG-1). Static / temporal / import-edge evidence is scored at its weaker tier and **never** called causal (LS-2 fake-precision). L-5 **drives** F-4 / mutation / metamorphic; it does not re-implement them (`L-plan/L-5_counterfactual_do_test_on_model_claims.md:5`, `L-plan/L-SUMMARY_INDEX.md:39-44`, `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:37-45,87-88,99-102`).

## 2. Why it exists

P1 and P3 are uncertainty signals. The do-test is the only **causal** one, and the only signal the model cannot game: F-4’s stdout-hash is captured by the orchestrator, not the worker (R8) (`L-plan/L-5_counterfactual_do_test_on_model_claims.md:13`). The paper’s action-marginalization test: perturb the action, see if output responds; action-shuffle ratio ≤ 1.1 means the model **ignored** the action = hallucination (§4.1.ii / §5) — **[EXTERNAL/DESIGN-RATIONALE], never an ELAI threshold** (`:13,260`, `L-plan/L-0_INDEX.md:168`).

If this organ is missing, L-2 can set `causal_supported` from a fired flag or an import graph. If it is faked (LLM “I reverted A”, or FILE-exists as causal, or worker-supplied `passed=true` without hash), HG-1 and F-4 both die.

**Honest residue (GD-5):** not every claim is cheaply interventional (production traffic, non-snapshottable effects). Those score at the **runtime-trace** tier (strong but not counterfactual); the limit is named (`:13,225`).

## 3. How it works (runtime)

1. **Who calls it.** Production: `score_claim_risk_wired_with_samples` predictor slice (4th element `CounterfactualPredictor`). Callers named: `research_profile.rs:75`, `task_runner.rs:1998` / `:2025` (pins; re-grep W-7) (`:98-104`). Contracts-only does not satisfy R7.
2. **Input.** An L-1 Causal/Behavior claim, J-16 blast-radius **cone**, and two orchestrator-captured F-4 hashes (baseline vs intervened). The predictor **does not** capture hashes; the caller does (`:62`).
3. **Plan intervention (deterministic, TD-3).** `plan_intervention(claim, cone) -> Intervention`. Causal/Behavior with a **resolved ref in the cone** → one of `{RevertOnlyA, DisableFlag, PatchOnlyFn, AddTrace}` from the ref kind. No ref in cone, or non-Causal/Behavior → `NotInterventional` (`:50-51`). **No model call.**
4. **Tier guard (HG-1).** `tier_guard(claim)`: only-static/temporal/import evidence → weaker tier, **never** causal. Non-snapshottable → runtime-trace cap, limit recorded on `TierGuard` (`:52`).
5. **Isolate then intervene (safety).** Revert/patch to test a claim **mutates** a tree. Run inside I-2 COW worktree / Tier-C. **Never** the live tree. Never ship the change (observe-the-outcome) (`:266`). Initial landing **does not** execute live interventions (`:256`); a rebuild of **this organ** still owes the isolation seam before W-1.
6. **Run outcomes (GD-1, compose).** Baseline and intervened:
   - F-4 `AntiFabricationGate.check` + blake3 `stdout_hash` (un-forgeable).
   - and/or C-28/I-20 mutants on the J-16 cone (`compute_cone` / `select_regression_set`).
   - and/or C-29 `MetamorphicRelation`.
   Auditor cannot emit `passed=true` without the orchestrator-captured hash (`:16`).
7. **Compare (deterministic).** `run_do_test(claim, intervention, baseline_hash, intervened_hash) -> DoTestVerdict { moved, delta }`. Hashes in, hashes compared; no F-4 reimplementation (`:62-63`). `moved` must mean the outcome changed **as claimed** (fix-claim: fail→pass; cause-claim: B changes when A is reverted). Live cousin uses **string inequality only** (`counterfactual.rs:106-107`) — that is not “as claimed”.
8. **Emit.** `PredictorSignal { source: Counterfactual, fired: moved && interventional kind, value: 1.0|0.0 }`. `NotInterventional` → signal **present but unfired**; `causal_supported` stays false (HG-1) (`:64`). L-2 already derives `causal_supported` from a **fired** Counterfactual signal (`risk.rs:93-94`); do **not** add a `PredictorKind` arm (`:39,120`).
9. **Commit.** Prefer-reuse `ClaimRiskScored` (carries `causal_supported` via `ClaimRisk`). **No** `CounterfactualProbed` unless a later unit does the 9-tuple lockstep (`:40,124`).
10. **Wired path without hashes.** `CounterfactualPredictor::new()` returns unfired signals (L-4 n<2 pattern). Present, not stub (`:91,146`). W-1 is **not** satisfied by that degrade.
11. **The model may generate** the claim text. It may **not** design the intervention, judge the outcome, supply `passed=true`, or label causal.

F-4 pin in L-CGG-GROUNDING-MAP `task_runner.rs:9565` is **stale**; recon amendment 1: `task_runner.rs:12486`, struct `evidence.rs:820`, check `evidence.rs:283`, hash `evidence.rs:90` (`:35`). Rebuild the **symbols**, re-grep lines at edit.

Paper ratio ≤ 1.1 is **not** copied into `DoTestVerdict.delta` (ID-5).

## 4. Types, enums, objects

Copy from `L-plan/L-5_counterfactual_do_test_on_model_claims.md:49-64`. Do not invent fields.

```
Intervention {
  target_ref: ClaimRef,
  kind: InterventionKind,
  cone: Vec<String>,            // J-16 blast-radius members
}

enum InterventionKind {
  RevertOnlyA,
  DisableFlag,
  PatchOnlyFn,
  AddTrace,
  NotInterventional,
}

TierGuard { /* weaker tier + recorded limit for non-snapshottable */ }

DoTestVerdict { moved: bool, delta: f64 }

fn plan_intervention(claim: &ExtractedClaim, cone: &[String]) -> Intervention
fn tier_guard(claim: &ExtractedClaim) -> TierGuard
fn run_do_test(claim, intervention, baseline_hash, intervened_hash) -> DoTestVerdict

struct CounterfactualPredictor          // PredictorSource
  // new() → no verdict → unfired
  // with hashes/verdict → fired iff moved && Causal|Behavior
```

**Consumed, not redefined:**

- F-4 `AntiFabricationGate` / `TransitionContext` / `IntegrationTestRowSummary.stdout_hash` (`:110`).
- C-28/I-20 `MutationBudgetedRun`.
- C-29 `MetamorphicRelation` (`metamorphic_testing.rs:49` pin).
- CSQ (phase-2D) — **safety questions on fs/net/money/auth**, not the do-test itself (`02D-csq-inline-feasibility.md` §1). Compose; do not merge.
- J-16 `compute_cone` / `select_regression_set`.
- G-1 taint on untrusted evidence used as “A”.
- L-2 `PredictorKind::Counterfactual` (already exists).

Live cousin: `Intervention` / `InterventionKind` / `tier_guard` / `run_do_test` are **private**; only `CounterfactualPredictor` is `pub` (`counterfactual.rs:6-75`). `plan_intervention` picks kind by substring `flag` / `trace` / `::` (`:40-48`) — not specified as the plan’s “depending on the ref kind” table. `tier_guard`: Dependency/Reachability → Medium; Causal/Behavior with resolved ref → High; else High + `limit: Some("runtime-trace")` (`:58-65`). File/import-only is **not** a named static-tier in that match — test (d) uses Dependency.

## 5. Control flow / state machine

**Owner:** orchestrator do-test. Model proposes A/B; engine plans, isolates, hashes, compares.

```
ClaimKind Causal|Behavior?
  no  → NotInterventional; tier_guard weaker; causal_supported false
  yes → J-16 cone
        ref ∩ cone?
          no  → NotInterventional
          yes → plan_intervention (RevertOnlyA|DisableFlag|PatchOnlyFn|AddTrace)
                I-2/Tier-C isolate
                F-4 (and/or mutation/metamorphic) baseline hash
                apply intervention (observe-only)
                F-4 intervened hash
                AntiFabricationGate.check (forged passed=true → Err)
                run_do_test → moved?
                  yes + as-claimed → Counterfactual fired → L-2 causal_supported true
                  no  → NOT-causal
```

**Forced:**

- HG-1: no causal label without interventional/executable **pass**.
- Cost-gate: most expensive predictor; plan says it runs **last**, only when cheap+P1+P3 already flag **high**, J-84 budgeted (`:267`). L-2 text said L-5 on **medium-or-above**. Rebuild: **last + J-84**; High-only vs medium+ is a named gap (§16).
- L-15 R2: land this **before** L-4 in the wave (`L-plan/L-15_repair_controller_and_evidence_first_reorder.md:112-117`).
- Parallel-safe files with L-3/L-4 (`:9`).

**Do not confuse with CSQ.** CSQ asks “if this fs/auth call is wrong, was a guard added?” (2D). L-5 asks “if I revert A, does B change?”

## 6. Grounding, receipts, gates

The receipt **is** the pair of F-4 hashes (or mutation/metamorphic outcomes) plus `DoTestVerdict`. GD-2: `moved` recomputes from those two hashes under `ELAI_SEED` (`:239`).

**Selector hierarchy (HG-1):**

1. Interventional / executable pass (this organ) → may set `causal_supported`.
2. Runtime trace (non-snapshottable GD-5 cap) → **not** causal.
3. Static / temporal / import-edge / FILE exists / “A imports B” → weaker tier, **never** causal.
4. Model narrative / GRC trace (RR-1) → never.

**Never allowed:** worker `passed=true` without matching orchestrator hash; LLM-designed intervention; calling P1 reconstruction “causal”; paper shuffle-ratio as `delta` threshold.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| planner | ~60 LOC | Slice 1A (`:46`) |
| runner | ~100 LOC | Slice 1B (`:59`) |
| file | ~160 LOC | (`:246`) |
| contract | ~90 LOC / 7 tests | (`:72,242`) |
| seam | ~3 LOC | roundtrip slice (`:248`) |
| cost | last in L-2 gate; J-84 | per claim (`:267`) |
| isolation | I-2 COW / Tier-C | never live tree (`:266`) |
| live model repro | RunPod-gated | W-10 (`:7,236`) |
| SSB | reuse ClaimRiskScored; net 0 | W-9 (`:40`) |
| paper shuffle ratio | ≤ 1.1 [EXTERNAL] | **do not** code (`:13,260`) |

## 8. Roles and neighbors

Role-agnostic on **any** backend’s claim (HG-3), but the **outcome** is orchestrator-captured (R8). Auditor cannot forge F-4 (`:16`).

**Consumes:** F-4, C-28/I-20, C-29, CSQ, J-16, G-1, L-1 ClaimSet, L-2 Counterfactual kind + `causal_supported` (`:109-114`).

**Produces:** Counterfactual `PredictorSignal`; `causal_supported` via L-2 fuse; reused ClaimRiskScored.

**Must not fork:** a second test runner; worker-side hash; new `PredictorKind`; new SSB variant; `workflow/reasoning/L-5-decisions.md` (gitignored — record in `conductor/opus_walk/evidence/L-5/`) (`:41,125`).

**Frontend:** N/A — feeds L-2; F-4/mutation rows already surface (`:127-128`).

## 9. Config / flags / defaults

- No predictor flag; gate flag is L-7 (`:233`).
- Initial landing: unfired without hashes (`:256`). Rebuild still needs a hash-injection unit for W-1.
- Isolation is mandatory before live mutate (`:266`).
- Do **not** modify `claim/risk.rs` (`causal_supported` already derived) (`:254`).
- blake3; no `.unwrap()` on prod (`:126`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `claim/counterfactual.rs` | planner + runner + `CounterfactualPredictor` (`:107`) |
| `claim.rs` `pub mod counterfactual` | after `semantic_entropy` (`:47,247`) |
| `roundtrip.rs` predictor slice | 4th element (`:87-89,115-116`) |
| F-4 | `evidence.rs` struct/check/hash; `task_runner.rs` check call (`:110`) |
| mutation emit | `deterministic_audit_gate.rs:505` pin |
| `MetamorphicRelation` | `metamorphic_testing.rs:49` pin |
| J-16 | `blast_radius.rs` `compute_cone` / `select_regression_set` (`:111`) |
| `l5_counterfactual_contract.rs` | tests (a)–(g) (`:249`) |

**Live neighbors:** `CounterfactualPredictor::new` unfired (`counterfactual.rs:75-80`); `signals` refuses causal fire on non-Causal/Behavior even if `moved` (`:86-91`); contract `fabricated_pass_is_rejected` calls `AntiFabricationGate.check` with `stdout_hash: None` (`l5_counterfactual_contract.rs:41-48`). Re-grep all numeric pins at edit (W-7); several L-5 pins already replaced MAP `:9565`.

## 11. Journal / SSB events

**No new family.** Reuse `ClaimRiskScored` (`ssb` ClaimRiskScored / payload). `causal_supported=true` only when the do-test outcome moves (`:131`). F-4 `TestIntegrityEvent` / `MutationBudgetedRun` / `MetamorphicRelation` rows remain the underlying receipts (`L-plan/L-0_INDEX.md:134` still allowed prefer-reuse of those; L-5 recon **forbids adding** `CounterfactualProbed`).

GD-2: `DoTestVerdict` recomputable from the two hashes (`:63`).

## 12. Tests and observables

Named (`:72-79,242`):

| Id | Assert |
|---|---|
| (a) | true fix-claim: intervention in cone; different hashes → `moved`; signal fires; `causal_supported` |
| (b) | false fix-claim: same hashes → unfired; not causal |
| (c) | “A causes B”, revert A, B unchanged → NOT-causal |
| (d) | “A imports B” / non-Causal-Behavior → static-tier, never causal |
| (e) | `plan_intervention` deterministic (no model) |
| (f) | forged `passed=true` with `stdout_hash=None` rejected by F-4 |
| (g) | `NotInterventional` runtime-trace cap recorded on `TierGuard` |

Wiring (`:213-218`): `rg` intervention/counterfactual/moved; `rg -ci causal` shows the guard; `pub mod counterfactual`; wired in `roundtrip.rs`.

**observable_effect** (`:129-132`, `L-plan/L-SUMMARY_INDEX.md:44`): “this patch fixes the timeout” is causal **only** if the F-4-bound failing repro flips under patch-only intervention. “A causes B” where revert A leaves B unchanged → NOT-causal. Negative: import-edge never causal.

**W-1:** live do-test flips a fix-claim’s causal label by re-running F-4 under the intervention — **not** cargo-green, **not** the unfired wired degrade (`:232`).

Adversarial: causal from File kind; hash inequality without isolation; live-tree revert; paper 1.1 ratio; LLM-chosen intervention; new SSB variant.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- `InterventionKind` is exactly those five variants.
- `plan_intervention` is deterministic from claim refs + J-16 cone; no LLM.
- `run_do_test` compares **caller-captured** F-4 (or mutation/metamorphic) hashes; does not reimplement those engines.
- `causal_supported` / Counterfactual `fired` only on interventional **pass** for Causal/Behavior; static/import never causal (HG-1).
- Forged `passed=true` without stdout-hash is rejected (F-4).
- Interventions run isolated (I-2/Tier-C), never on the live tree; observe-only.
- Reuses `PredictorKind::Counterfactual` and `ClaimRiskScored`; no new kind/family unless a later unit locksteps schema.
- Wired on the production claim-risk slice (not contracts-only).
- Tests (a)–(g). W-1: a real repro hash-flips the causal bit.
- Paper shuffle-ratio is not a coded constant.

A hash-inequality helper with unfired `new()`, or “the model said reverting A would work”, is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Re-implementing F-4 / mutation / metamorphic (GD-1 P0) (`:122`).
- New `PredictorKind` arm (`:123`).
- New SSB `CounterfactualProbed` in this unit (`:124`).
- `workflow/reasoning/L-5-decisions.md` (`:125`).
- Labelling File/Dependency/Reachability/import-edge causal (HG-1).
- Model-designed intervention or model-judged outcome (TD-3).
- Applying the intervention to the live workspace (`:266`).
- Coding paper ratio ≤ 1.1 (ID-5).
- Merging CSQ into the do-test.
- Treating the unfired wired predictor as W-1 done (`:232` vs `:256`).

## 15. Source citations

Primary: `docs/thefinalglance/L-plan/L-5_counterfactual_do_test_on_model_claims.md` (full file). Index: `L-plan/L-SUMMARY_INDEX.md:39-44`. Doctrine: `L-plan/L-0_INDEX.md:50,95-96,134`. Grounding map: `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:37-45,87-88,99-102`. Recon: `conductor/opus_walk/evidence/L-5/L-5-recon.md` (named `:33,262`; 8 amendments in `:35-42`). Neighbors: `02D-csq-inline-feasibility.md`, `L-02-claimrisk-convergence-fuse.md`. Live: `claim/counterfactual.rs`, `l5_counterfactual_contract.rs`, `claim.rs:309-310`. Paper: arXiv:2606.27326 §4.1.ii / §5 [EXTERNAL/DESIGN-RATIONALE].

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Map from ref kind to `{RevertOnlyA, DisableFlag, PatchOnlyFn, AddTrace}` (live uses substring `flag`/`trace`/`::`).
- `DoTestVerdict.delta` semantics beyond 1.0/0.0; “as claimed” direction vs mere inequality.
- When to use F-4 vs mutation vs metamorphic vs all three.
- Cost-gate High-only (`:267`) vs L-2 medium-or-above.
- `TierGuard` fields (live `{tier: RiskTier, limit: Option<&'static str>}` not in the plan struct sketch).
- How G-1 taint on A blocks an intervention.
- Hash-injection production API (explicitly a future design unit, `:256,266`).
- Whether CSQ must pass before a do-test may mutate even in COW.
- Stale line pins vs current `task_runner.rs` / `evidence.rs` / `ssb.rs` (always re-grep).
- Public vs private `Intervention` API (plan lists them in produces_symbols; live keeps them private).
- Test (d) File vs Dependency as the import-edge stand-in.
