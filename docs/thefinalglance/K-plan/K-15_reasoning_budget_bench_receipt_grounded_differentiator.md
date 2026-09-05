# K-15: reasoning-budget bench + receipt-grounded-reasoning differentiator — PROVE the GRC-v3 win with an ELAI-measured budget→verified-outcome knee, and bank the "the engine grounds the model" differentiator as a tagged claim

STATUS: PLANNED (WAVE KW-3; PROVABLE+PACKAGE; BUILD-instrument (a bench composing I-27/J-34) + dossier/brief differentiator rows; the GRC-v3 done-gate; BENCH-PENDING on the J GRC-v3 track landing; +0..1 SSB, prefer-reuse first)

Objective: the GRC-v3 reasoning run (workflow/reasoning/reasoning-engine-grc-v3-2026-06-25.md) is mostly CAPABILITY work (the typed BeliefState carrier, the 2-axis classifier, the overthinking guard, the receipt-gated ladder = the J-83..J-86 technical track). But its non-negotiable DONE-GATE (FINAL SYNTHESIS, "DONE-GATE non-negotiable, ID-5/GD-3") is a PROVE artifact that is pure K: the win must be shown by an ELAI-MEASURED curve, never a paper number. K-15 builds `elai bench reasoning-budget` — the per-(model,task-type) verified-outcome-rate vs reasoning-compute curve (a J-34 EMPIRICAL CONTEXT CALIBRATION sibling, same harness shape, x-axis = reason-budget not context-fill) under the SAFEDELTA gate-ON-vs-OFF framing (GRC-v3 router ON vs flat-budget OFF), composing the SHIPPED I-27 ConvergenceCurveMeasured harness + H-9 cost cockpit; receipts (not self-reported accuracy) on the y-axis. It finds the per-(model,task-type) PEAK/KNEE that J-84's classifier and J-85's overthinking-guard ceiling track (the local Qwen-7B lane overthinks EARLIEST/LOWEST, so its knee is its own — never the paper's 6K/7K, which the overthinking paper itself forbids transferring). SECOND leg [PACKAGE/PROVE]: bank the on-wedge DIFFERENTIATOR (the deepest grounded finding of the run, IMPLICATION 20) as a K-1 dossier [DESIGN] row + a claims-policy entry + a K-7 brief line — "ELAI's reasoning engine grounds every reasoning step on an EXTERNAL receipt; the model never grades its own confidence, because a model structurally cannot reliably ground itself (the self-correction blind spot, three independent 2024-2026 papers)". HARD BOUNDARY it must NOT cross: it does NOT author the capability (J-83..J-86 = J, recorded below NOT here); the bench is BENCH-PENDING until the J track lands (the harness ships now, the GRC-v3-ON arm is gated on J-83/J-85, like K-13's injection row is [BENCH-PENDING] until A-2/A-3); it claims NO measured number for the differentiator until the bench runs (it is a [DESIGN] architectural claim, KD-1); the done-gate is ELAI-measured on an ELAI gold set, NEVER a paper constant (ID-5, GD-3); it does NOT re-implement I-27/J-34/H-9/G-6 (composes the runner shape, GD-1).

Estimated effort: ~1 session (~M). A bench runner composing I-27's bench_convergence shape + a `reasoning-budget` bench sub + the dossier/brief differentiator rows + the K-1 lint recogniser extension. The deterministic FIXTURE tier ships now; the GRC-v3-ON arm is BENCH-PENDING on J-83/J-85; the live local-lane run is RunPod/remote-sidecar-gated (W-10).
Pre-requisites (re-grep at authoring, W-7): I-27 ConvergenceCurveMeasured + bench_convergence (the verified-outcome-vs-compute harness — COMPOSE the runner shape, do NOT fork); J-34 `elai bench context-fill` (the sibling calibration bench, the per-(model,lane) curve pattern); the SAFEDELTA/QDELTA runner shape (benchmarks/safedelta_runner.rs, KG-5/KG-9 — the args_bench.rs:75 bench-sub seam); H-9 CloudPanel (the cost display); the K-1 dossier + claims-policy lint (the differentiator row + KD-1 enforcement K-15 reuses, never forks). The J GRC-v3 track (J-83 carrier, J-84 classifier, J-85 guard, J-86 ladder — the thing the bench measures; BENCH-PENDING on them).
Parallel-safe with: K-13, K-14, K-16 (disjoint files). Soft-depends K-1 (lint + dossier), K-7 (brief). Bench-blocks on the J GRC-v3 track.
Source: reasoning-engine-grc-v3-2026-06-25.md DONE-GATE + FINAL SYNTHESIS §5/§6 + IMPLICATION 20/29/30 + T1..T5 (the honest residue); _grounding/J-GRC-V3-GROUNDING-MAP.md GS-10 (I-27/J-34) + GA-5 + R-2/R-3/R-4 + GW. Doctrine: K-0_INDEX.md (KD-1 number-before-claim, HD-1 measured-per-config, GD-3 ablation-not-leaderboard); J-34/ID-5 (measure-your-own-knee).

## Context
Why this matters: the GRC-v3 design is grounded, but the HEADLINE PROOF — "does the engine's reasoning-state management actually lift the verified-outcome rate, and at what compute" — does not exist as an ELAI number. The overthinking paper proves accuracy is NON-MONOTONIC in reasoning budget (rises, peaks, FALLS) and that the knee is MODEL-SPECIFIC and LOWER for small models, explicitly forbidding transferring its threshold. So ELAI must MEASURE its own per-(model,task-type) reasoning-budget→verified-outcome curve and find the peak — exactly the J-34 pattern applied to reasoning budget instead of context fill. K-15 is that measurement (the done-gate the J track ships against) plus the banking of the run's deepest finding as an honest, tagged differentiator. The number is the product (K's premise); everything in the J track waits behind this proof. Mechanism: a bench composing I-27's verified-outcome-vs-compute harness, two arms (GRC-v3 router ON = receipt-gated + budget-routed + guard; OFF = flat budget, no guard), under ELAI_SEED, recomputable from the journal (GD-2); the differentiator is a [DESIGN] claim until the bench produces the delta.

## Existing state (A/B..J coverage — what the bench/claim compose, do NOT re-author, GD-1)
- I-27 BUDGETED CONVERGENCE INSTRUMENTATION [DARK] ConvergenceCurveMeasured + bench_convergence (bootstrap-CI, mirrors bench_qdelta); `elai bench convergence` — the verified-outcome-vs-compute harness K-15 composes (GS-10).
- J-34 EMPIRICAL CONTEXT CALIBRATION [PLANNED] `elai bench context-fill` per-(model,lane) fill→degradation curve, gate verdicts never LLM-judge (TD-3) — the SIBLING bench; K-15's reasoning-budget curve is the same shape, x-axis swapped.
- G-6 SAFEDELTA [BENCH-PENDING] gate-ON-vs-OFF ablation framing (the structure K-15 mirrors) + I-38 QDELTA runner (KG-5) + args_bench.rs:75 (the bench-sub seam, KG-9).
- H-9 COST COCKPIT [PARTIAL] $/resolved-task + CloudPanel — the cost (lambda) display the curve plots against.
- the J GRC-v3 track J-83..J-86 (this same run's technical half) — the GRC-v3-ON arm; BENCH-PENDING on it.
- K-1 dossier + claims-policy lint (the differentiator row + KD-1 enforcement K-15 reuses); K-7 brief (the buyer line it feeds); J-85 overthinking guard (whose ceiling reads K-15's measured knee — the bench is its calibration source).

## Reads (mandatory before coding)
1. reasoning-engine-grc-v3-2026-06-25.md DONE-GATE block + FINAL SYNTHESIS §5/§6 + IMPLICATION 20 (the differentiator)/29 (model-specific knee)/30 (U_lambda) + the HONEST RESIDUE T1..T5.
2. _grounding/J-GRC-V3-GROUNDING-MAP.md (GS-10, GA-5, GW, R-2/R-3/R-4) + the K-1 dossier + the K-1 claims-policy lint (reuse, GD-1).
3. I-27 bench_convergence + benchmarks/safedelta_runner.rs + args_bench.rs:75 + J-34's plan (the sibling bench shape) — re-grep live (W-7).
4. K-13's plan (the [BENCH-PENDING] number-binding pattern K-15 mirrors) + K-1/K-7 (the dossier/brief surfaces) + J-85 (the guard whose ceiling consumes the measured knee).
5. CLAUDE.md: KD-1 (number-before-claim), HD-1 (measured-per-config, never a literature figure), ID-5 (measure-your-own knee, never the paper's), GD-3 (ablation-not-leaderboard), TD-3 (gate verdicts never LLM-judge).

## Tasks (WAVE KW-3 — the bench + the differentiator banking)
1. [PROVE] `elai bench reasoning-budget` (~M, BUILD): a bench runner mirroring I-27 bench_convergence + a `reasoning-budget` bench sub at args_bench.rs:75; two arms (GRC-v3 router ON vs flat-budget OFF) over a fixed task corpus per (model, task-type), reporting the verified-outcome-rate vs reasoning-compute curve + the measured PEAK/KNEE, all under ELAI_SEED, recomputable (GD-2). Receipts on the y-axis (NOT self-reported accuracy; the SAFEDELTA-correct version). Deterministic FIXTURE tier ships now; the ON arm is BENCH-PENDING on J-83/J-85; the live local-lane run is RunPod-gated (W-10).
2. [PROVE] expose the measured knee as the artifact J-84/J-85 consume (the per-(model,task-type) ceiling the overthinking guard tracks) — write it to the ModelCapabilityProfile-adjacent calibration artifact J-34 already defines (reuse, GD-1), keyed by model identity, G-10-drift-invalidated.
3. [PACKAGE/PROVE] bank the differentiator: (a) a K-1 dossier [DESIGN] row — "receipt-grounded reasoning: every reasoning step's confidence comes from an external receipt, never the chain self-grading (the self-correction blind spot)" with the F-4/D-13/I-4 code basis + the [BENCH-PENDING] tag for the measured lift (no fabricated number, KD-1); (b) a one-line claims-policy entry so any positioning string asserting "grounds its own reasoning / self-verifying reasoning" carries the [DESIGN] tag + the receipt-not-self-eval correction; (c) a K-7 brief line. Do NOT modify K-1/K-7 plan files (a dossier row + brief line, cross-linked, new-plan-files-only).
4. EXTEND the K-1 claims-policy lint recogniser (NOT a fork, GD-1) so a "reasoning grounds itself / self-verifying reasoning" claim without the [DESIGN] tag + receipt-not-self-eval note FAILS, and a reasoning-budget number without an ELAI-measured journal pointer FAILS (KD-1/HD-1). ~12-15 LOC.
5. NEGATIVE arm: a fixtures file with (a) a "ELAI's reasoning self-verifies" untagged over-claim, (b) a bare "GRC-v3 lifts accuracy 20%" number with no measured pointer, (c) a row citing the paper's 7K knee as ELAI's — the lint fails all three.
6. workflow/reasoning/K-15-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - benchmarks/reasoning_budget_runner.rs + a `reasoning-budget` bench sub (args_bench.rs:75) — the per-(model,task-type) verified-outcome-vs-reasoning-compute curve + knee (the GRC-v3 done-gate)
  - the measured-knee calibration artifact (the per-(model,task-type) ceiling J-84/J-85 consume; written to the J-34 calibration artifact, reused not forked)
  - the receipt-grounded-reasoning differentiator: a K-1 dossier [DESIGN] row + a claims-policy entry + a K-7 brief line
  - the differentiator + reasoning-budget-number recogniser extension in .claude/tools/claims_policy_lint.py (K-1's lint, extended)
consumes_symbols:
  - I-27 ConvergenceCurveMeasured + bench_convergence (COMPOSE the runner shape; GS-10; GD-1) — NOT forked
  - J-34 `elai bench context-fill` + its calibration artifact (the sibling shape + the knee store; reuse)
  - benchmarks/safedelta_runner.rs + args_bench.rs:75 (the ablation framing + bench-sub seam; KG-5/KG-9) + H-9 cockpit (cost display)
  - the J GRC-v3 track J-83..J-86 (the GRC-v3-ON arm under measure; BENCH-PENDING on it)
  - the K-1 dossier + claims-policy lint (EXTEND, never fork; KD-1; GD-1)
reachable_from:
  - `elai bench reasoning-budget --suite default --seed N --json` (a REAL CLI entrypoint, runnable verbatim) [PRODUCTION bench]
  - the auditor runs claims_policy_lint.py over any positioning copy carrying the differentiator [PRODUCTION audit gate]
  - J-84/J-85 read the measured-knee artifact as the per-(model,task-type) ceiling [PRODUCTION reasoning loop, the calibration consumer]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO paper number as the done-gate or the ceiling (ELAI-measured per-(model,task-type) knee; ID-5 — the overthinking paper forbids transferring its threshold)
  - NO self-reported accuracy on the y-axis (verified-OUTCOME / receipts; the SAFEDELTA-correct version, GD-3)
  - NO fabricated differentiator number ([DESIGN]/[BENCH-PENDING] until the bench runs; KD-1)
  - NO LLM-judge scoring in the bench (deterministic gate verdicts; TD-3); NO fork of I-27/J-34/the K-1 lint (compose/extend; GD-1); NO main-LLM self-run of the lint (auditor-applied)
  - NO authoring of the J capability (J-83..J-86) here (out-of-K)
frontend_pair:
  the curve + the measured knee surface on the I-27 ConvergencePanel + the H-9 CloudPanel ($/verified-task) — a reasoning-budget arm on the existing convergence tile (decide at edit, W-4); justified N/A if it rides those existing panels.
observable_effect:
  trigger: "elai bench reasoning-budget --suite default --seed 1234 --json (the deterministic FIXTURE tier; the GRC-v3-ON arm is BENCH-PENDING on J-83/J-85, the live local-lane run RunPod-gated W-10) + the auditor runs claims_policy_lint.py over the differentiator copy"
  expect_ssb: "prefer-reuse I-27 ConvergenceCurveMeasured (+Kind: ReasoningBudgetArm) — net 0..+1 from LIVE 268/577, decide at edit (W-9)"
  expect_artifact: "one curve artifact per (model, task-type) with the measured knee, recomputable from the journal (GD-2); the GRC-v3-ON arm shows a verified-outcome lift at/near the knee vs the flat-budget OFF arm (BENCH-PENDING until the J track lands); the K-1 lint PASSES the [DESIGN]/[BENCH-PENDING]-tagged differentiator and FAILS (the NEGATIVE arm) a 'reasoning self-verifies' untagged over-claim + a bare unmeasured number + a paper-knee-as-ELAI's row"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-15_reasoning_budget_bench_receipt_grounded_differentiator.md --run-trigger
rg -n "reasoning-budget|reasoning_budget" .elai_cc/crates/cli/src/args_bench.rs   # the bench sub is wired
rg -n "verified.outcome|receipt|knee|peak" .elai_cc/benchmarks/reasoning_budget_runner.rs   # receipts on the y-axis, knee reported
rg -ci "7000|7K|paper.*knee|R1.32B" .elai_cc/benchmarks/reasoning_budget_runner.rs   # MUST be 0 (no transferred paper knee; ID-5)
python .claude/tools/claims_policy_lint.py --files docs/thefinalglance/K-plan/_fixtures/reasoning_grounding_negative.md ; test $? -ne 0   # the over-claims MUST fail
test -f workflow/reasoning/K-15-decisions.md
```

## ACCEPTANCE
- [ ] `elai bench reasoning-budget` produces the per-(model,task-type) verified-outcome-vs-compute curve + the measured knee, ELAI_SEED-recomputable; receipts on the y-axis (GD-3).
- [ ] the measured knee is written to the calibration artifact J-84/J-85 consume (the ceiling, per model+task-type; ID-5, never the paper's).
- [ ] KD-1: the receipt-grounded-reasoning differentiator is [DESIGN]/[BENCH-PENDING]-tagged; the K-1 lint passes it and FAILS the untagged/unmeasured/paper-knee over-claims.
- [ ] the K-1 lint is EXTENDED not forked (GD-1); K-1/K-7 plan files NOT amended (a dossier row + brief line, cross-linked).
- [ ] the J capability (J-83..J-86) is NOT authored here; the GRC-v3-ON arm is BENCH-PENDING on it; the live local run is RunPod-gated (W-10).

## WIRING DONE-GATE (clause by clause)
- W-1: done = the bench actually runs the FIXTURE arm + produces a recomputable curve, AND the lint negative fixture FAILS — not "the runner compiles".
- W-2: N/A (a bench + a doctrine claim, no feature flag). W-3: composes I-27/J-34, not a summary-port.
- W-4: rides the I-27/H-9 panels (reasoning-budget arm); justified N/A if no new tile. W-5/W-10: FIXTURE tier now; ON arm BENCH-PENDING on J-83/J-85; live local-lane run RunPod-gated. W-6: existing convergence/cost panel or justified N/A.
- W-7: I-27 bench_convergence / safedelta_runner / args_bench.rs:75 / J-34 calibration artifact re-grepped at edit.
- W-9: prefer-reuse ConvergenceCurveMeasured (+Kind); net 0..+1 from LIVE 268/577 derived.
- KD-1: the lint IS the enforcement (no differentiator claim without [DESIGN]/[BENCH-PENDING]; no reasoning-budget number without a measured pointer). HD-1/ID-5: the knee is measured-per-(model,task-type), never a literature figure. GD-2: the curve is journal-recomputable. GD-3: ablation framing (GRC-v3-ON vs OFF), never a leaderboard rank.

## Tests to add
- a Python test for the lint extension (or crates/contracts/tests/k15_reasoning_grounding_claims_contract.rs ~50 LOC): (a) the [DESIGN]/[BENCH-PENDING] differentiator passes; (b) "reasoning self-verifies" untagged fails; (c) a bare "GRC-v3 +20%" with no measured pointer fails; (d) a paper-7K-knee-as-ELAI's row fails; (e) empty input fails-loud (R1). PLUS a bench determinism test: the FIXTURE arm is byte-identical under a fixed seed (the curve is reproducible).

## Files touched
- CREATE: .elai_cc/benchmarks/reasoning_budget_runner.rs (~M LOC, mirrors bench_convergence/safedelta_runner)
- MODIFY: .elai_cc/crates/cli/src/args_bench.rs (~5 LOC: the `reasoning-budget` bench sub at :75)
- MODIFY: the J-34 calibration artifact writer (~15 LOC: add the reasoning-budget knee, keyed by model+task-type; reuse, GD-1)
- MODIFY: docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md (+1 [DESIGN]/[BENCH-PENDING] differentiator row; K-1's file — a row add)
- MODIFY: .claude/tools/claims_policy_lint.py (+~14 LOC differentiator + reasoning-budget-number recogniser; K-1's lint, extended)
- CREATE: docs/thefinalglance/K-plan/_fixtures/reasoning_grounding_negative.md (~12 LOC, the negative arm)
- CREATE: workflow/reasoning/K-15-decisions.md (~30 LOC)
- (SSB, if a Kind is added) MODIFY: ssb.rs + action_type_lock.rs + .golden (9-tuple lockstep; decide at edit)

## Citations / Grounding
- reasoning-engine-grc-v3-2026-06-25.md DONE-GATE ("the win is proven by an ELAI-MEASURED curve, never a paper number ... `elai bench reasoning-budget` + I-27 ... receipts on the y-axis ... SAFEDELTA-style gate-ON-vs-OFF") + IMPLICATION 20 (the self-correction blind spot = the differentiator), 29 (the per-model knee, the J-34 sibling), 30 (U_lambda = $/verified-task).
- ELAI ground truth (GS-10): I-27 ConvergenceCurveMeasured + bench_convergence [DARK], J-34 `elai bench context-fill` [PLANNED] + its calibration artifact, safedelta_runner.rs + args_bench.rs:75 (KG-5/KG-9), H-9 CloudPanel — re-grepped at edit (W-7).
- arXiv [EXTERNAL, grounding for WHY not an ELAI number]: overthinking 2604.10739 (non-monotonic budget→accuracy; the knee is model-specific and forbidden to transfer — the load-bearing reason K-15 measures ELAI's own), Reflexion 2303.11366 + faithfulness (the self-correction blind spot = the differentiator's basis).
- CLAUDE.md: KD-1 (number-before-claim), HD-1 (measured-per-config), ID-5 (measure-your-own knee), GD-3 (ablation framing), TD-3 (deterministic bench verdicts).

## Open questions / risks
- BENCH-PENDING sequencing: the harness + FIXTURE arm ship now, but the GRC-v3-ON arm cannot produce a real delta until J-83 (carrier) + J-85 (guard) land. State this honestly in the dossier row ([BENCH-PENDING]); do NOT let the brief imply a measured lift before the J track ships (KD-1) — exactly the K-13 injection-row discipline.
- the differentiator is partly proven by ARCHITECTURE (receipts-not-self-eval is shipped: F-4/D-13/I-4) but the LIFT is unmeasured — the [DESIGN] claim is honest about that; the [BENCH-PENDING] number is what K-15's bench will fill.
- the knee artifact is the calibration source J-84/J-85 read; keep it ONE artifact (reuse J-34's), never a parallel store (GD-2), keyed by model identity + invalidated on G-10 drift.
- TECHNICAL COMPANION WORK, OUT-OF-K (recorded so nothing is lost; the J GRC-v3 track, authored alongside this wave as J-83..J-86 per the user's 2026-06-26 ruling): J-83 the typed BeliefState reasoning carrier [KEYSTONE]; J-84 the 2-axis reasoning-effort classifier; J-85 the deterministic overthinking guard (the one net-new mechanism); J-86 the receipt-gated candidate ladder activation; the I-14 deterministic-compression "preserve-receipts/fold-prose" RIDER on the existing I-14 unit (not re-authored). All capability (= J), all measured by THIS bench. The honest residue T1..T5 (depth-vs-width-vs-window reconciliation, the imperfect difficulty proxy, the lagging flip-ratio, the local-lane-overthinks-earliest, the seam-composition risk) belongs to the J authoring + K-15 calibration.

## Next
K-15 PROVES the GRC-v3 win with an ELAI-measured budget→verified-outcome knee (the done-gate the J track ships against, and the calibration source J-84/J-85 consume), and banks the receipt-grounded-reasoning differentiator honestly. It pairs with K-1 (the dossier + lint it extends), K-7 (the brief it feeds), K-16 (the doctrine that binds the receipts-not-traces + measure-your-own-knee rules the bench measures), and the J GRC-v3 track (the capability it measures, BENCH-PENDING on it).

End of K-15.
