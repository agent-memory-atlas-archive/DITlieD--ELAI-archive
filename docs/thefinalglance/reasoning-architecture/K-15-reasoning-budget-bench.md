# Reasoning-budget bench + receipt-grounded-reasoning differentiator

- **ELAI ids:** K-15 / WAVE KW-3 / GS-10 / GA-5 / KD-1
- **Spec status (as designed):** PLANNED (`ELAI-FEATURE-MAP.md:2828-2845`). PROVE+PACKAGE. FIXTURE tier ships now; GRC-v3-ON arm is BENCH-PENDING on J-83/J-85; live local-lane run RunPod-gated (W-10).
- **Crate / module (spec):** `.elai_cc/benchmarks/reasoning_budget_runner.rs`; CLI `elai bench reasoning-budget` on `args_bench.rs:75`; knee written into the J-34 calibration artifact; K-1 dossier + `claims_policy_lint.py` extension. Does **not** author J-83..J-86.
- **Purpose of this file:** enough architecture to rebuild the GRC-v3 done-gate instrument — verified-outcome vs reasoning-compute, measured peak/knee, tagged differentiator. Not a status ledger.

## 1. What it is

The **reasoning-budget bench** is `elai bench reasoning-budget`. It is the J-34 empirical-context-calibration **sibling** with the x-axis swapped to **reason-budget** (not context-fill). Over a fixed corpus it plots per-(model, task-type) **verified-outcome-rate vs reasoning-compute**, two arms (GRC-v3 router **ON** = receipt-gated + budget-routed + guard vs flat-budget **OFF**), under `ELAI_SEED`, recomputable (GD-2). Y-axis is **receipts**, never self-reported accuracy (GD-3, SAFEDELTA-correct). It extracts the per-(model,task-type) **PEAK/KNEE** that J-84’s classifier and J-85’s overthinking-guard ceiling consume. Second leg **packages** the differentiator: “receipt-grounded reasoning: every step’s confidence is from an external receipt, never the chain self-grading” as a K-1 dossier `[DESIGN]`/`[BENCH-PENDING]` row + claims-policy + K-7 brief line (`K-plan/K-15_reasoning_budget_bench_receipt_grounded_differentiator.md:1-5,12`, `K-plan/K-SUMMARY_INDEX.md:388-411`).

## 2. Why it exists

GRC-v3 capability (J-83..J-86) is grounded, but the headline proof — does the engine’s reasoning-state management lift verified-outcome rate, and at what compute — does not exist as an ELAI number (`K-plan/K-15_reasoning_budget_bench_receipt_grounded_differentiator.md:12`). The overthinking paper shows accuracy is **non-monotonic** in reasoning budget (rises, peaks, **falls**) and that the knee is **model-specific** and **lower** for small models, and **forbids transferring** its threshold. The local Qwen-7B lane overthinks earliest/lowest, so its knee is its own — never the paper’s 6K/7K (ID-5) (`:5,109`, `K-plan/K-SUMMARY_INDEX.md:390-395`). If this organ is missing, J-84/J-85 ceilings are folklore and positioning copy can claim “GRC-v3 +20%.” If it is faked (LLM-judge y-axis, paper knee, untagged “reasoning self-verifies”), KD-1 is broken (`:56-59,66`). K’s premise: the number is the product; the J track waits behind this proof (`:12`).

## 3. How it works (runtime)

1. **Who calls it.** Production CLI: `elai bench reasoning-budget --suite default --seed N --json` (runnable verbatim). Auditor runs `claims_policy_lint.py` over differentiator copy. J-84/J-85 **read** the measured-knee artifact as the ceiling. A `#[cfg(test)]` caller never satisfies R7 (`K-plan/K-15_reasoning_budget_bench_receipt_grounded_differentiator.md:50-54,63-64`).
2. **Input.** `--suite`, `--seed`, `--json`; corpus of tasks keyed by `(model, task-type)`; arm set ON vs OFF. `ELAI_SEED` binds recomputation (`:31,63`).
3. **Runner (compose, do not fork).** Mirror I-27 `bench_convergence` (bootstrap-CI, same family as `bench_qdelta` / `safedelta_runner.rs`). Register the `reasoning-budget` sub at `args_bench.rs:75` (KG-9) (`:31,45-47`).
4. **OFF arm (always runnable).** Flat budget, no GRC-v3 guard. FIXTURE tier ships now (`:31,113`).
5. **ON arm (BENCH-PENDING).** GRC-v3 router ON = receipt-gated + budget-routed + overthinking guard. Cannot produce a real delta until J-83 (carrier) + J-85 (guard) land. Dossier row **says** `[BENCH-PENDING]`; the brief must **not** imply a measured lift before the J track ships (KD-1; K-13 injection-row discipline) (`:31,113`).
6. **Y-axis (deterministic receipts).** Verified-outcome / gate verdicts. **Not** self-reported accuracy. **Not** LLM-judge (TD-3) (`:5,57-58`).
7. **X-axis.** Reasoning-compute (the budget the ON router / OFF flat spend). Peak/knee extracted per `(model, task-type)` (`:31`).
8. **Knee artifact (deterministic write).** Write the measured knee into the **J-34** calibration artifact (reuse, GD-1), keyed by **model identity + task-type**, G-10-drift-invalidated. One store, never a parallel knee file (`:32,115`).
9. **PACKAGE differentiator.** (a) K-1 dossier `[DESIGN]` row with F-4/D-13/I-4 code basis + `[BENCH-PENDING]` for the lift; (b) one-line claims-policy so “grounds its own reasoning / self-verifying reasoning” carries `[DESIGN]` + receipt-not-self-eval correction; (c) K-7 brief line. Do **not** amend K-1/K-7 **plan files** — a dossier row + brief line, cross-linked (`:33`).
10. **Lint extension (~12–15 LOC).** Extend `claims_policy_lint.py` (not a fork): untagged “reasoning grounds itself / self-verifying reasoning” **FAILS**; a reasoning-budget number without an ELAI-measured journal pointer **FAILS** (`:34`).
11. **Fail-closed / negative arm.** Fixture `reasoning_grounding_negative.md` must fail all three: untagged “ELAI’s reasoning self-verifies”; bare “GRC-v3 lifts accuracy 20%”; paper 7K knee as ELAI’s (`:35,66`). Empty input fails-loud (R1) (`:94`).

The model may generate reasoning **inside** a measured task. It may **not** score the curve, pick the knee, or grade its own confidence as the y-axis.

## 4. Types, enums, objects

Copy from `K-plan/K-15_reasoning_budget_bench_receipt_grounded_differentiator.md:31-33,63-66` and `ELAI-FEATURE-MAP.md:2828-2844`. Do not invent I-27 payload fields.

```
ReasoningBudgetArm {
  Off,                               // flat budget, no guard
  GrcV3On,                           // receipt-gated + budget-routed + guard; BENCH-PENDING on J-83/J-85
}

YChannel {
  verified_outcome_rate,             // receipts / gate verdicts; NEVER self-reported accuracy
}

XChannel {
  reasoning_compute,                 // NOT context-fill (that is J-34)
}

ReasoningBudgetKnee {                // per-(model, task-type)
  model,
  task_type,
  peak,                              // PEAK/KNEE the plan names together
  knee,
}

// written into J-34 calibration artifact (ONE store):
CalibrationArtifact {
  // existing J-34 context-fill fields (owned by J-34)
  reasoning_budget_knee,             // added; keyed by model identity + task-type
  // G-10 drift invalidates
}

ConvergenceCurveMeasured             // I-27; REUSE
  // optional +Kind ReasoningBudgetArm (decide at edit, lockstep, W-9)

DifferentiatorClaim {
  tags: [DESIGN] | [BENCH-PENDING],  // both until lift is measured
  text: "receipt-grounded reasoning: every reasoning step's confidence comes from an external receipt, never the chain self-grading (the self-correction blind spot)",
  code_basis: F-4 / D-13 / I-4,
}
```

**U_λ = $/verified-task** is IMPLICATION 30; H-9 CloudPanel displays cost (`:108,47`). Exact coupling of U_λ into the runner output is UNCERTAIN (§16).

**Paper 7K / 6K / R1.32B** must not appear as results (`rg -ci "7000|7K|paper.*knee|R1.32B"` MUST be 0) (`:73`).

## 5. Control flow / state machine

**Owner of the loop:** bench runner + auditor lint. J-84/J-85 are **consumers** of the knee, not owners of this organ. K-16 is the doctrine that **governs** the differentiator claim; K-15 **measures**.

**Forced order:**

| Step | Rule |
|---|---|
| FIXTURE / OFF first | ships without J-83/J-85 |
| ON arm | BENCH-PENDING; no fabricated delta |
| Curve rows before scalar | same R1 as J-107: a number with zero rows is a cousin fail |
| Knee write | into J-34 artifact, model+task-type key, G-10-invalidated |
| Differentiator copy | `[DESIGN]`/`[BENCH-PENDING]` until ON arm runs |
| Lint | auditor-applied; main-LLM must not self-run (`:59`) |

**No token-level budget FSM.** Window size is **not** the x-axis (RD-4 / K-16 RR-3).

## 6. Grounding, receipts, gates

**Y-axis = external receipts** (gate verdicts). That **is** the organ. Trace/CoT length is monitoring, not the score (K-16 RR-1).

**Selector hierarchy for the done-gate number:** journal-recomputable verified-outcome curve > nothing. Vote-over-samples is not this bench’s y-axis. LLM-judge is banned.

**Never allowed:** paper overthinking knee as ELAI’s ceiling; self-reported accuracy; untagged “self-verifying reasoning”; “GRC-v3 +20%” without a journal pointer; a parallel knee store beside J-34 (`:56-59,115`).

The differentiator’s **architecture** half is already shipped (F-4/D-13/I-4). The **lift** is unmeasured — `[DESIGN]` is honest about architecture; `[BENCH-PENDING]` is the number K-15 will fill (`:114`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| CLI | `elai bench reasoning-budget --suite default --seed N --json` | production |
| arms | ON vs OFF | per cell |
| key | `(model, task-type)` | knee + curve |
| seeds | `ELAI_SEED`; FIXTURE determinism = byte-identical under fixed seed (`:94`) | run |
| SSB | prefer-reuse `ConvergenceCurveMeasured` (+Kind); net 0..+1 from LIVE 268/577 | W-9 |
| lint extension | ~12–15 LOC | `claims_policy_lint.py` |
| runner | mirrors `bench_convergence` / `safedelta_runner` | new file |
| CLI add | ~5 LOC at `args_bench.rs:75` | seam |
| knee writer | ~15 LOC on J-34 artifact | reuse |
| live local | RunPod-gated W-10 | not FIXTURE |

**Overflow:** ON arm without J-83/J-85 must not emit a lift. Paper constants in the runner = FAIL `rg`. Ceiling J-84/J-85 read is **this** knee, never a literature figure.

Do not confuse with: J-107 (LongCoT / recursion-band x-axis; third calibration axis beside this one), J-34 (context-fill), I-27 `elai bench convergence` (the composed harness).

## 8. Roles and neighbors

K-native PROVE+PACKAGE. **Out-of-K:** J-83 BeliefState carrier, J-84 effort classifier, J-85 overthinking guard, J-86 receipt-gated ladder, I-14 preserve-receipts/fold-prose rider (`:117`). K-15 does **not** author them.

**Consumes:** I-27 curve harness; J-34 artifact; `safedelta_runner` + `args_bench.rs:75`; H-9 cockpit; J-83..J-86 as the ON arm; K-1 dossier + lint; K-7 brief (`:44-49`).

**Produces:** curve + knee; dossier row; lint recogniser; K-7 line; `workflow/reasoning/K-15-decisions.md`.

**Must not fork:** I-27, J-34, K-1 lint, a second knee store, a J-track capability file.

**Parallel-safe with** K-13, K-14, K-16 (disjoint files). Soft-depends K-1, K-7. Bench-blocks on the J GRC-v3 track (`:9`).

**Governs / is governed:** K-16 RR-3 **states** measure-your-own-knee; this organ **is** that measurement (`K-plan/K-SUMMARY_INDEX.md:419-427`).

## 9. Config / flags / defaults

- No runtime feature flag (W-2 N/A — a bench + a doctrine claim) (`:87`).
- FIXTURE suite runs now; ON arm gated on J-83/J-85; local live W-10 (`:88`).
- Do not modify K-1/K-7 **plan files** (`:33,82`).
- Decision doc `workflow/reasoning/K-15-decisions.md` (`:36`).
- Frontend: I-27 ConvergencePanel + H-9 CloudPanel as a reasoning-budget arm, or justified N/A (W-4) (`:61-62`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `.elai_cc/benchmarks/reasoning_budget_runner.rs` | curve + knee (`:40,97`) |
| `args_bench.rs:75` `reasoning-budget` sub | production CLI (`:41,50,98`) |
| J-34 calibration artifact writer | knee store (`:41,99`) |
| `docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md` | +1 `[DESIGN]`/`[BENCH-PENDING]` row (`:100`) |
| `.claude/tools/claims_policy_lint.py` | +~14 LOC (`:43,101`) |
| `K-plan/_fixtures/reasoning_grounding_negative.md` | negative arm (`:102`) |
| `workflow/reasoning/K-15-decisions.md` | (`:103`) |
| `ssb.rs` + lock + `.golden` | only if Kind added (`:104`) |
| `crates/contracts/tests/k15_reasoning_grounding_claims_contract.rs` | or a Python lint test (`:94`) |

## 11. Journal / SSB events

Prefer-reuse (`:65`, `ELAI-FEATURE-MAP.md:2843`):

```
ConvergenceCurveMeasured (+Kind: ReasoningBudgetArm)  // net 0..+1; decide at edit, W-9
```

No new action_type family required. Rebuild: emit I-27-shaped rows; do not rewind SCHEMA. Differentiator is **prose + lint**, not an SSB row (E-23-class packaging).

## 12. Tests and observables

Lint / contract (`:94`):

| Id | Assert |
|---|---|
| (a) | `[DESIGN]`/`[BENCH-PENDING]` differentiator **passes** |
| (b) | “reasoning self-verifies” untagged **fails** |
| (c) | bare “GRC-v3 +20%” with no measured pointer **fails** |
| (d) | paper-7K-knee-as-ELAI’s **fails** |
| (e) | empty input fails-loud (R1) |
| FIXTURE det | byte-identical under fixed seed |

CLI: `elai bench reasoning-budget --suite default --seed 1234 --json` produces a recomputable curve; ON arm lift vs OFF is BENCH-PENDING (`:63-66`).

`rg` (`:71-73`): `reasoning-budget` wired in `args_bench.rs`; `verified.outcome|receipt|knee|peak` in the runner; paper-knee strings = 0.

W-1: FIXTURE arm **runs** + curve exists **and** lint negative fixture **FAILS** — not “the runner compiles” (`:86`).

Adversarial: LLM-judge y-axis; transferring 7K; implying a measured lift in the K-7 brief before J-83/J-85; a second knee file.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Production CLI `elai bench reasoning-budget` (or equivalent **production** verb).
- Two arms: router-ON vs flat-OFF. ON may be stubbed **only** if tagged BENCH-PENDING and no lift is claimed.
- Y-axis is verified-outcome **receipts**. No LLM-judge. No self-report.
- X-axis is reasoning-compute, not context-fill and not window size.
- Peak/knee is per-(model, task-type), written to the **same** calibration store the overthinking guard reads, invalidated on model drift.
- Runner **composes** the convergence-curve harness, does not fork it.
- Differentiator copy is tagged `[DESIGN]`/`[BENCH-PENDING]` until the ON arm exists; a claims lint fails untagged self-verify, unmeasured %, and imported paper knees.
- This organ does **not** implement BeliefState, the effort classifier, or the overthinking guard.

A leaderboard rank, a prompt “think up to 7K tokens,” or quoting the overthinking paper’s knee as the product number is a **cousin**.

## 14. Non-goals / banned designs

- Paper number as done-gate or ceiling (ID-5; the overthinking paper forbids transfer) (`:56`).
- Self-reported accuracy on the y-axis (GD-3) (`:57`).
- Fabricated differentiator number (KD-1) (`:58`).
- LLM-judge scoring; fork of I-27/J-34/K-1 lint; main-LLM self-run of the lint (`:59`).
- Authoring J-83..J-86 here (out-of-K) (`:60`).
- Amending K-1/K-7 plan files (`:33`).
- Parallel knee store (GD-2) (`:115`).
- Window-as-the-lever (RD-4; K-16 RR-3).

## 15. Source citations

Primary: `docs/thefinalglance/K-plan/K-15_reasoning_budget_bench_receipt_grounded_differentiator.md` (full file). Index: `K-plan/K-SUMMARY_INDEX.md:388-411`. Feature map: `ELAI-FEATURE-MAP.md:2828-2845`. Reasoning doc: `workflow/reasoning/reasoning-engine-grc-v3-2026-06-25.md` DONE-GATE + FINAL SYNTHESIS §5/§6 + IMPLICATION 20/29/30 + T1..T5. Grounding map: `_grounding/J-GRC-V3-GROUNDING-MAP.md` GS-10, GA-5, GW, R-2/R-3/R-4. ArXiv EXTERNAL (why, not an ELAI number): overthinking 2604.10739; Reflexion 2303.11366 + faithfulness. Doctrine: K-0 KD-1/HD-1/GD-3; J-34/ID-5. Neighbors: I-27, J-34, J-84, J-85, K-1, K-7, K-16, K-13 (BENCH-PENDING pattern).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- `ConvergenceCurveMeasured` field list (I-27 owns it).
- Whether `+Kind ReasoningBudgetArm` is added (`:65,104`).
- Exact `reasoning_compute` unit (tokens, GRC calls, wall-clock, $) — IMPLICATION 30 names U_λ = $/verified-task; H-9 is the display; the runner’s x-axis unit is not a typed field (`:108`).
- `task-type` enum (what keys the knee besides model).
- Peak vs knee: named together; whether they are two numbers or one (`:31`).
- FIXTURE corpus contents / `--suite default` membership.
- How G-10 drift invalidation is wired on this new field (`:32`).
- Python lint test vs `k15_reasoning_grounding_claims_contract.rs` (`:94`).
- Dashboard tile vs reuse (W-4) (`:61-62`).
- LIVE 268/577 derived; re-derive W-9.
- Honest residue T1..T5 (depth-vs-width-vs-window, difficulty proxy, lagging flip-ratio, local-lane-overthinks-earliest, seam-composition) belongs to J authoring + this calibration — not specified as runner fields (`:117`).
