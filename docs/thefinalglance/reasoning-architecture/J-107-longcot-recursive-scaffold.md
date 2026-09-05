# LongCoT recursive-scaffold bench (`elai bench longcot`)

- **ELAI ids:** J-107 / RG-2 / RG-3 / RG-6 / RS-5 / WAVE RLM-1
- **Spec status (as designed):** PLANNED (`ELAI-FEATURE-MAP.md:2846-2862`). J-SUMMARY labels it the one clear RLM build that GATES J-108 (`J-plan/J-SUMMARY_INDEX.md:2216-2229`). Fixture/cloud arm ships now; live local-lane run is RunPod-gated (W-10).
- **Crate / module (spec):** `.elai_cc/benchmarks/longcot_runner.rs` (new); CLI subcommand on `.elai_cc/crates/cli/src/args_bench.rs`; curve emission reuses I-27 `ConvergenceCurveMeasured` / `bench_convergence` / `ConvergencePanel`. Does **not** live in `crates/reasoning`.
- **Purpose of this file:** enough architecture to rebuild the LongCoT two-arm measurement instrument — flat vs constrained-scaffold, receipt y-axis, per-(model,lane) knee, J-108 gate. Not a status ledger.

## 1. What it is

The **LongCoT recursive-scaffold bench** is the CLI instrument `elai bench longcot`. It adapts the open MIT LongCoT harness (GitHub `LongHorizonReasoning/longcot` + HF `LongHorizonReasoning/LongCoT`, arXiv:2604.14140) as an ELAI-measured reasoning-bench arm over the **reused** K-15 / I-27 runner. Two arms: **LB-1 FLAT (OFF)** single-shot vs **LB-2 CONSTRAINED-SCAFFOLD (ON)** the J-108 within-step recursive-context primitive. Y-axis is verified-outcome rate (receipts) **and** abstention-rate **and** tokens-per-correct. X-axis is imposed attention/recursion budget bands. The extracted per-(model,lane) **knee** plus the lift plus the local-lane collapse-check are the recorded artifact that GATES J-108’s local-lane default-ON (`J-plan/J-107_longcot_recursive_scaffold_bench.md:1-7,15-20`, `J-plan/_grounding/J-RLM-GROUNDING-MAP.md:23-26,72-75`).

## 2. Why it exists

The RLM corpus’s empirical finding is ELAI’s harness>model thesis **measured**: a scaffold with no fine-tuning lifts small open models on long-horizon reasoning (`J-plan/_grounding/J-RLM-GROUNDING-MAP.md:13-22`, RG-1). That lands on the local Qwen2.5-Coder-7B lane. The paper RLM scaffold works by the model writing Python over a context variable; J-55 forbids general CodeAct on the sub-14B lane. Two ELAI experiments already measured that a **constrained** named-op/slice path sits below that collapse threshold (RG-6) while free-code fabricates (RG-4) (`J-plan/J-107_longcot_recursive_scaffold_bench.md:12-13`). If this organ is missing, J-108’s local-lane default is a vibe and the paper leaderboard (Qwen3-4B 0→100, Codex 72.5%) becomes the fake done-gate (`J-plan/_grounding/J-RLM-GROUNDING-MAP.md:105-110`). If it is faked (LLM-judge scorer, paper 6K/7K as the knee, a verified-outcome number with zero curve rows), the gate is theater (`J-plan/J-107_longcot_recursive_scaffold_bench.md:56-59,65`). This organ ships **no** new reasoning architecture and **no** model-chosen orchestration (RG-1 G6 anti-goal) (`J-plan/J-107_longcot_recursive_scaffold_bench.md:5`).

## 3. How it works (runtime)

1. **Who calls it.** A real CLI: `elai bench longcot --model <m> --lane <local|cloud> --arms flat,scaffold` dispatched from `args_bench.rs`. A `#[cfg(test)]` / contracts caller **never** satisfies production reachability (R7) (`J-plan/J-107_longcot_recursive_scaffold_bench.md:52-54,64`).
2. **Input.** Model id, lane (`local|cloud`), arm set (`flat`, `scaffold`), seed count (≥5 paired, I-38), and the cached HF LongCoT task set. Imposed-budget bands mirror exp-1/exp-2’s `W / 0.5W / 2W / 8W` shape (`J-plan/J-107_longcot_recursive_scaffold_bench.md:37,63`).
3. **Task adapter (deterministic fetch + score).** Fetch/cache HF `LongHorizonReasoning/LongCoT`. Score exact-match / receipt-checkable **on the ELAI side**. No LLM-judge (TD-3). Do **not** vendor the MIT harness wholesale — adapt the task spec + scoring only (RG-2, license-clean) (`J-plan/J-107_longcot_recursive_scaffold_bench.md:37,51,57`).
4. **LB-1 FLAT (OFF).** Single-shot reasoning at the imposed attention/recursion budget W. Baseline. No J-108 primitive (`J-plan/J-107_longcot_recursive_scaffold_bench.md:16`).
5. **LB-2 CONSTRAINED-SCAFFOLD (ON).** Calls J-108’s primitive (named-op slice + one capped child-spawn + HippoRAG hop). **Not** free-Python RLM (RG-4, REJECTED). **Not** model-chosen orchestration (RG-1 G6). If J-108 has not landed, the arm is a **registered STUB** wired to J-108’s entrypoint; the gate runs flat-only and **says so** — no silent half-measure (`J-plan/J-107_longcot_recursive_scaffold_bench.md:17,38,78,111-112`).
6. **Y-axis (deterministic, receipts).** Verified-outcome rate from receipts (SAFEDELTA-correct, GD-3), abstention-rate (abstention is **GOOD** when the answer is unreachable — the RG-4 trust property), and tokens-per-correct. Abstention is a **distinct channel**, not counted as a wrong answer (`J-plan/J-107_longcot_recursive_scaffold_bench.md:18,58,79`, `J-plan/J-SUMMARY_INDEX.md:2220-2221`).
7. **X-axis + knee (deterministic extract).** Budget bands and/or recursion depth. Knee = the per-(model,lane) point past which more recursion/steps stop converting abstentions to correct answers. This is the K-15-class calibration source J-108’s `max_steps` and J-84/J-85’s overthinking ceiling consume. ELAI-MEASURED. **Never** the paper 6K/7K (ID-5) (`J-plan/J-107_longcot_recursive_scaffold_bench.md:19-20,39`).
8. **Seeds.** ≥5 seeds paired (I-38). `<5` is rejected. Knee extraction must be monotone-stable across a re-run with the same `ELAI_SEED` (GD-2) (`J-plan/J-107_longcot_recursive_scaffold_bench.md:39,81,93`).
9. **Collapse-check (deterministic, local lane).** LB-2 code-error / fabrication rate on the local lane **must not** exceed the flat arm (the RG-6 J-55 gate). A scaffold arm whose local-lane fabrication exceeds flat **fails** the gate (`J-plan/J-107_longcot_recursive_scaffold_bench.md:41,65,80`).
10. **Commit.** Emit `ConvergenceCurveMeasured` rows (reuse I-27; `+Kind LongCotArm` only if a discriminator is needed, decide at edit, lockstep W-9), one per `(model, lane, band, arm)`. Extract the knee. Write the lift (`scaffold − flat` verified-outcome rate) + collapse-check as the artifact J-108’s authoring reads (`J-plan/J-107_longcot_recursive_scaffold_bench.md:39,41,64-65`).
11. **Abstain / fail-closed.** A verified-outcome number with **zero** `ConvergenceCurveMeasured` rows behind it is rejected (fabricated-curve negative arm). Single-seed claims rejected. LLM-judge scorer banned. Routing LB-2 to free-Python CodeAct on the local lane banned (`J-plan/J-107_longcot_recursive_scaffold_bench.md:56-59,65,93`).

The model may **generate** answers (and, on LB-2, propose one named op / read one entity — that is J-108). It may **not** score the item, choose the arm, set the knee, or judge whether abstention was “good.”

## 4. Types, enums, objects

Copy from `J-plan/J-107_longcot_recursive_scaffold_bench.md:15-20,37-41,64-65` and `J-plan/_grounding/J-RLM-GROUNDING-MAP.md:118-119`. Do not invent fields I-27 owns.

```
LongCotArm {                         // two closed arms
  Flat,                              // LB-1 OFF: single-shot at imposed budget W
  ConstrainedScaffold,               // LB-2 ON: J-108 primitive; NOT free-Python RLM
}

Lane { Local, Cloud }

BudgetBand                           // shape: W, 0.5W, 2W, 8W (mirror exp-1/exp-2)
                                     // exp-1 used W=6000 as OWN DATA (RG-3).
                                     // Production numeric W: UNCERTAIN (§16).

YChannels {
  verified_outcome_rate,             // receipts, GD-3; NEVER self-reported accuracy
  abstention_rate,                   // GOOD on unreachable (RG-4); distinct channel
  tokens_per_correct,
}

LongCotKnee {                        // per-(model, lane)
  model, lane,
  // point past which more recursion/steps stop converting abstentions → correct
}

LongCotGateArtifact {                // what J-108's authoring reads
  knee: LongCotKnee,
  lift,                              // LB-2 minus LB-1 verified-outcome rate
  collapse_check,                    // local-lane LB-2 fabrication ≰ flat
}

ConvergenceCurveMeasured             // I-27 organ; REUSE, do not fork
  // one row per (model, lane, band, arm)
  // optional +Kind LongCotArm (decide at edit, lockstep, W-9)

fn longcot_scorer(task, answer) -> {Correct, Incorrect, Abstain}
  // exact-match / receipt-checkable; same input → same verdict; no model call
```

**I-27 payload fields are not declared in J-107.** Rebuild consumes I-27’s row shape. Do not invent `ConvergenceCurveMeasured` members here.

**Stub contract when J-108 is absent:** LB-2 is a registered stub at J-108’s entrypoint; the CLI **says** flat-only; no lift number may be claimed (`J-plan/J-107_longcot_recursive_scaffold_bench.md:38,111-112`).

## 5. Control flow / state machine

**Owner of the loop:** the bench runner (`longcot_runner.rs`) over the reused I-27 / K-15 harness. Not the model. Not `GrcEngine::think`. Not J-108 (J-108 is the LB-2 **primitive under test**).

**Forced order:**

| Step | Rule |
|---|---|
| Adapter first | HF dataset cached; deterministic scorer wired before any arm runs |
| Flat always runnable | LB-1 does not wait on J-108 |
| Scaffold gated | LB-2 requires J-108 primitive **or** a registered stub that refuses a lift claim |
| Seeds | `<5` → reject the run (I-38) |
| Curve before number | a scalar verified-outcome with zero rows → reject (R1) |
| Knee last | extract after all `(band, arm, seed)` cells exist |
| J-108 local default | authored ON **only if** lift is real **and** collapse-check passes |

**No token-level FSM.** This is a bench arm. G6 (who chooses decomposition) is **not** opened here — LB-2’s op set is J-108’s FIXED `NavOp`, not model-authored control flow (`J-plan/_grounding/J-RLM-GROUNDING-MAP.md:13-22`).

## 6. Grounding, receipts, gates

This organ **measures** whether a constrained recursive scaffold lifts verified outcomes. It does not itself verify files or run product GRC.

**Selector hierarchy (scoring, not candidate selection):** external receipt / exact-match **only**. LLM-judge of answers is banned (TD-3). Chain-of-thought as a “looks correct” signal is banned (that is K-16 RR-1; this bench must not grow a cousin).

**Never allowed as a confidence / done-gate signal:** paper LongCoT leaderboard numbers, paper 6K/7K overthinking knee, RLM marketing 0→100 / 72.5% / 75.4%, self-reported accuracy, a scalar with no `ConvergenceCurveMeasured` rows, treating abstention as failure (`J-plan/J-107_longcot_recursive_scaffold_bench.md:56-58`, `J-plan/_grounding/J-RLM-GROUNDING-MAP.md:105-110`).

**J-108 gate (this organ’s product):** local-lane default-ON for the DEPTH operator is authored **only if** this arm shows a real lift with no J-55 collapse. Cloud/capable lane may ship without waiting (`J-plan/J-107_longcot_recursive_scaffold_bench.md:3,13`, `J-plan/J-108_retrieval_navigator_depth_operator.md:3,8`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| arms | exactly `{Flat, ConstrainedScaffold}` | per run |
| seeds | ≥5 paired (I-38) | per run; `<5` rejected |
| budget-band shape | `W / 0.5W / 2W / 8W` | per cell; numeric W UNCERTAIN (§16) |
| scorer | exact-match / receipt-checkable | per item |
| SSB | reuse `ConvergenceCurveMeasured`; net 0..+1 Kind | per `(model,lane,band,arm)` |
| adapter LOC (plan estimate) | ~80 | task adapter |
| two-arm driver | ~70 | over K-15/I-27 runner, not a fork |
| curve + knee | ~40 | emission |
| CLI | ~25 | `args_bench.rs` |
| contract tests | ~70 | `j107_longcot_bench_contract.rs` |
| runner total (plan) | ~190 | `longcot_runner.rs` |
| live local model | RunPod-gated W-10 | fixture/cloud ships now |

**Overflow / refuse:** no lift claim before J-108’s primitive is real. No paper constant as a result (`rg` of `6000|6144|7000|paper.*leaderboard|72\\.5|0->100` in `longcot_runner.rs` **MUST be 0**) (`J-plan/J-107_longcot_recursive_scaffold_bench.md:72`). Note the tension with exp-1’s measured `W=6000` (RG-3): that figure is OWN DATA for the **shape**, not a hardcoded result string — see §16.

No per-topic `GrcBudget` lives here (that is 2A/4A). No token window as the lever (RD-4 / K-16 RR-3).

## 8. Roles and neighbors

J-107 is **role-agnostic**. It is a CLI bench, not an agent role.

**Consumes:** K-15 runner shape (`elai bench reasoning-budget`, RS-5); I-27 `ConvergenceCurveMeasured` + `bench_convergence` + `ConvergencePanel`; J-34 calibration artifact (sibling; x-axis = context-fill; this organ’s x-axis = imposed recursion/attention budget); I-38 QDELTA ≥5-seed paired floor; J-108 RetrievalNavigator / HippoRAG primitive (LB-2 seam); HF `LongHorizonReasoning/LongCoT` (`J-plan/J-107_longcot_recursive_scaffold_bench.md:47-51`).

**Produces:** two-arm curve rows, per-(model,lane) knee, lift, abstention channel, collapse-check artifact, `elai bench longcot` CLI, `workflow/reasoning/J-107-decisions.md`.

**Must not fork:** a second convergence runner; wholesale MIT harness vendoring; a free-Python CodeAct “scaffold”; an LLM-judge cousin; treating this as a runtime GRC engine.

**Parallel-safe with J-108 on files** (this is `benchmarks/` + CLI; J-108 is `orchestrator/search` + memory). **Sequence:** J-107 flat + harness first; scaffold-ON reads J-108 (`J-plan/J-107_longcot_recursive_scaffold_bench.md:8-9`).

**Third calibration axis:** J-34 context-fill · K-15 reason-budget · J-107 imposed recursion/attention budget (`J-plan/J-107_longcot_recursive_scaffold_bench.md:116`, `ELAI-FEATURE-MAP.md:2858-2860`).

## 9. Config / flags / defaults

- **No** new default-OFF runtime gate. It is a CLI bench arm, invoked on demand (`J-plan/J-107_longcot_recursive_scaffold_bench.md:86`).
- `--arms flat,scaffold`. `--lane local|cloud`. `--seeds 5` (floor, not a folklore constant to lower).
- LB-2 may be stubbed; stub must **announce** flat-only.
- Decision doc required: `workflow/reasoning/J-107-decisions.md` (two arms, abstention-as-good, knee-is-the-gate-for-J-108, ID-5 never-cite-the-paper-number) (`J-plan/J-107_longcot_recursive_scaffold_bench.md:42`).
- Hardware-heavy: flag sidecar boot before any live local run (`J-plan/J-107_longcot_recursive_scaffold_bench.md:113`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `.elai_cc/benchmarks/longcot_runner.rs` | adapter + two-arm driver + curve/knee (`J-plan/J-107_longcot_recursive_scaffold_bench.md:46,96`) |
| `.elai_cc/crates/cli/src/args_bench.rs` `elai bench longcot` | production CLI (`:53,97`) |
| I-27 `bench_convergence` / `ConvergenceCurveMeasured` / `ConvergencePanel` | REUSE, do not fork (`:48`) |
| K-15 reasoning-budget runner shape | REUSE (`:23,48`; `ELAI-FEATURE-MAP.md:2266-2287` cited as RS-5 in the grounding map — feature-map 1.13 live block is `:2828-2845`) |
| J-34 calibration artifact | sibling store, not a second context bench (`:25,49`) |
| J-108 primitive | LB-2 arm-under-test (`:50`) |
| `crates/contracts/tests/j107_longcot_bench_contract.rs` | named contract (`:93,100`) |
| `workflow/reasoning/J-107-decisions.md` | four decisions (`:42,101`) |
| dashboard ConvergencePanel / LongCot tile | W-4 decide at edit (`:61,98`) |
| `ssb.rs` + `.golden` | **only if** `+Kind LongCotArm` (`:99`) |

A contracts-only caller is **not** this organ (R7).

## 11. Journal / SSB events

Prefer-reuse I-27. **No** new action_type family (`J-plan/_grounding/J-RLM-GROUNDING-MAP.md:118-119`):

```
ConvergenceCurveMeasured
  // one row per (model, lane, band, arm)
  // optional Kind LongCotArm — decide at edit, lockstep, W-9
  // net 0..+1 from LIVE ~268/577 (derived; re-derive at edit)
```

Rebuild: **emit I-27-shaped curve rows**. Do not rewind a live schema number. Do not invent a `LongCotMeasured` cousin action_type.

## 12. Tests and observables

Named contract `j107_longcot_bench_contract.rs` (`J-plan/J-107_longcot_recursive_scaffold_bench.md:92-93`):

| Id | Assert |
|---|---|
| (a) two-arm rows | driver emits a `ConvergenceCurveMeasured` row per `(model,lane,band,arm)` |
| (b) deterministic scorer | same input → same verdict; no model call |
| (c) abstention channel | recorded distinct; **not** counted as a wrong answer |
| (d) knee GD-2 | monotone-stable under same `ELAI_SEED` |
| (e) fabricated-curve | result with zero curve rows is rejected |
| (f) J-55 collapse | flags when local-lane scaffold fabrication exceeds flat |
| (g) seed floor | `<5` seeds rejected (I-38) |

CLI smoke: `elai bench longcot --arms flat,scaffold --seeds 5` (fixture now; live local W-10).

`rg` hygiene (`:70-72`): `llm.judge|judge_model|model.*scores.*answer` = 0; paper-constant result strings = 0.

Done-gates (`:77-82,85`): two arms over reused runner; y-axis receipts + abstention + tokens/correct; knee + lift + collapse-check as J-108’s artifact; ≥5 seeds; no wholesale MIT vendoring; production_caller 0-DARK (the subcommand **is** the production caller); W-1 = a real CLI run produced rows + a knee, not file-exists.

Adversarial: treating abstention as failure; silent stub that claims a lift; LB-2 routed to CodeAct.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- CLI `elai bench longcot` (or the harness’s equivalent **production** bench verb) with arms `{flat, scaffold}`.
- LB-1 is single-shot at an imposed budget; LB-2 is a **constrained** named-op/slice primitive, not free-Python RLM and not model-chosen orchestration.
- Scoring is deterministic exact-match / receipt-checkable. **No** LLM-judge.
- Y-axis includes verified-outcome **and** abstention-rate **and** tokens/correct. Abstention on unreachable is **good**, not a miss.
- ≥5 paired seeds. A scalar with zero curve rows is rejected.
- Per-(model,lane) knee is extracted from the curve, never imported from a paper.
- Local-lane scaffold fabrication ≰ flat is a **failing** gate, not a footnote.
- If the scaffold primitive is absent, the run **says** flat-only and claims no lift.
- Curve rows reuse the convergence-instrument family; the MIT LongCoT harness is adapted, not vendored.
- This organ does **not** implement HippoRAG, `NavOp`, or GRC transitions.

A prompt “think longer,” an MCTS search, or quoting LongCoT leaderboard numbers as the result is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- New reasoning architecture / model-chosen orchestration (RG-1 G6) (`J-plan/J-107_longcot_recursive_scaffold_bench.md:5`).
- Forking the K-15 / I-27 runner (GD-1) (`:56`).
- Wholesale copy of the MIT harness (`:57`).
- LLM-judge scorer (TD-3) (`:56`).
- Paper 6K/7K / leaderboard as a result (ID-5 / HD-1) (`:56,72`).
- Single-seed claims (I-38) (`:58`).
- Treating abstention as failure (RG-4) (`:58`).
- Routing LB-2 to free-Python CodeAct on the local lane (RG-4 / RG-6 / J-55) (`:59`).
- Claiming a lift before J-108’s primitive is real (`:112`).
- A second context-fill bench (that is J-34) (`:25`).
- `.unwrap` on prod (`:59`).

## 15. Source citations

Primary: `docs/thefinalglance/J-plan/J-107_longcot_recursive_scaffold_bench.md` (full file). Index: `J-plan/J-SUMMARY_INDEX.md:2216-2229`. Feature map: `ELAI-FEATURE-MAP.md:2846-2862`. Grounding map: `J-plan/_grounding/J-RLM-GROUNDING-MAP.md` RG-2/RG-3/RG-4/RG-5/RG-6, RS-5, RC-3. Own measured data: `workflow/experiments/rlm-vs-elai-retrieval-2026-06-26.md`, `workflow/experiments/RLM-VS-ELAI-RESULTS-2026-06-26.md` (the experiment this bench productionizes). Interface: `workflow/papers/longcot-bench-2604.14140-interface.md`. Steal analysis: `workflow/reasoning/rlm-recursive-agents-grc-2026-06-26.md` PART 5 (F2/F4) + PART 6 (LongCoT = NEW) + PART 7/8/9. Neighbor organs: K-15, I-27, J-34, J-108, J-55, I-38.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Production numeric `W` for budget bands. Exp-1 used `W=6000` (RG-3). The plan says **mirror the shape** `W/0.5W/2W/8W`, and also `rg` of `6000` in the runner **MUST be 0** as a paper-constant result (`J-plan/J-107_longcot_recursive_scaffold_bench.md:37,72`). Whether the runner takes `W` as a CLI/config input (honest) or hardcodes 6000 (conflicts with the rg gate) is unspecified.
- Whether x-axis is budget-band **or** recursion depth **or** both (`:19`).
- `ConvergenceCurveMeasured` field list (owned by I-27; J-107 does not restate it).
- Whether `+Kind LongCotArm` is added (`:39,64,99`).
- HF LongCoT task schema; which subset is exact-match / receipt-checkable vs excluded as subjective (`:110`).
- Stub API for LB-2 before J-108 lands (entrypoint name, error string).
- Knee extraction algorithm (how “stop converting abstentions to correct” is computed; monotone-stable is required, formula is not).
- Collapse-check statistic: code-error **or** fabrication **or** both; rate vs count (`:41,65`).
- `tokens_per_correct` definition (which token counter; J-28 dependency UNCERTAIN).
- Dashboard: dedicated LongCot tile vs reuse ConvergencePanel (W-4, decide at edit) (`:61`).
- Live SCHEMA_VERSION at implement time (plan cites ~268/577 derived; re-derive, W-9).
