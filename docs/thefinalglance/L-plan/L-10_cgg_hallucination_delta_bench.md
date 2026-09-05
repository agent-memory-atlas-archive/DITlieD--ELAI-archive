# L-10: CGG hallucination-delta bench — the ELAI-measured gate-ON-vs-OFF reduction number [the whole-plan done-gate]

STATUS: PLANNED (WAVE 4; M; BUILD-instrument; BENCH-PENDING — the live-model arm is RunPod-gated, W-10)

Objective: the CGG is only "done" when its effect is MEASURED, not asserted (ID-5/KD-1; the same discipline as G-6 SAFEDELTA and K-18). L-10 builds `elai bench hallucination-delta`: a gate-ON-vs-OFF ablation over a held-out claim corpus measuring the unsupported-claim rate, the contradicted-claim rate, and the reviewer-reaching rate with the CGG ON vs OFF, under ELAI_SEED, recomputable (GD-2), receipts on the y-axis (GD-3 — never pass@k). The deterministic FIXTURE tier ships now; the live-model arm (real claims from a real backend) is RunPod/free-pool-gated (W-10). The number is ELAI-MEASURED on an ELAI gold set, NEVER a paper AUROC/rho (ID-5); reported as "N fewer unsupported claims per 100 model outputs", never a leaderboard rank. This bench is the L-plan's whole-plan done-gate: the CGG ships default-ON (L-7) only after this delta is positive AND the over-block/FP rate is bounded (GD-4). HARD BOUNDARY: deterministic scoring, NO LLM-judge (TD-3 — the corpus labels are ground-truth, the gate verdicts are the measured signal). GD-1: do NOT fork the SAFEDELTA/QDELTA runner — compose its shape.

Estimated effort: ~1 session (~M) for the fixture tier; the live-model arm is BENCH-PENDING (RunPod, W-10).
Pre-requisites (re-grep live per W-7): G-6 SAFEDELTA runner (benchmarks/safedelta_runner.rs, args_bench.rs:75); I-38 QDELTA (the quality sibling + statistical floor >=5 seeds paired); I-27 ConvergenceCurveMeasured + bench_convergence (the runner shape); L-1..L-7 (the gate under measurement); E-54 eval corpus (the held-out claim corpus substrate).
Parallel-safe with: L-9. Composes L-1..L-7.
Source: cgg-hallucination-causal-system-2026-06-27.md S12 + L-CGG-GROUNDING-MAP NN-7/TS-19. Doctrine: L-0_INDEX.md (ID-5, GD-2, GD-3, GD-4).

## Context
Why this matters: every gate generation that shipped on faith and friction died (the K-T3 research); the CGG must prove it reduces hallucination MORE than it costs in over-block. The SAFEDELTA framing (G-6) is the right shape: ablate the gate ON vs OFF on a fixed corpus, report the delta as a count ("N fewer unsupported diffs per 100 tasks"), never a rank. L-10 builds the hallucination analogue: a held-out claim corpus (real model outputs labelled supported/unsupported/contradicted by a ground-truth oracle), run with the CGG ON vs OFF, measure the reduction in unsupported/contradicted claims that reach the user AND the over-block rate (a benign claim wrongly blocked). The done-gate is honest by ID-5: no number ships until it is ELAI-measured + journal-recomputable; the paper's rho~=0.80 and AUROC are [EXTERNAL] design-rationale, never claimed as ELAI's. This bench also feeds back the THRESHOLDS L-2/L-4/L-7 use (the tier boundaries, the entropy cutoff, the BLOCK-by-default promotion) — they are all calibrated here, never guessed.

## Existing state (E/G/I coverage)
- G-6 SAFEDELTA [BENCH-PENDING] — gate-ON-vs-OFF ablation; "N fewer exploitable diffs per 100"; safedelta_runner.rs + args_bench.rs:75. The runner shape. [MAP]
- I-38 QDELTA + statistical floor [PLANNED] — quality sibling; >=5 seeds, paired, pass^k. [MAP]
- I-27 ConvergenceCurveMeasured [DARK] + bench_convergence — bootstrap-CI runner; ConvergencePanel. [MAP]
- E-54 eval corpus design [DARK] — 13-gram contamination filter, per-family shards. The held-out corpus substrate. [MAP]
- L-1..L-7 — the gate under measurement.

## Reads (mandatory before coding)
1. _grounding/L-CGG-GROUNDING-MAP.md (NN-7, TS-19); cgg-hallucination-causal-system-2026-06-27.md S12.
2. safedelta_runner.rs + args_bench.rs:75 (the bench-sub seam); I-38 QDELTA statistical floor; bench_convergence; E-54 corpus shards; L-2/L-7 (the gate the bench drives).
3. F-0 W-1/W-10; G-0 GD-2/GD-3/GD-4; H-0 HD-1 (every efficiency/effect number measured per config); CLAUDE.md ID-5, no-.unwrap.

## Tasks (WAVE 4 — measure the delta, ELAI-grounded)
1. the held-out claim corpus (~50 LOC + data): build/seed a claim gold set (real model outputs labelled supported/unsupported/contradicted by a ground-truth oracle), per-family shards via E-54, contamination-filtered. The labels are ground-truth (NOT an LLM-judge, TD-3).
2. the hallucination-delta runner (~80 LOC): mirror safedelta_runner.rs — two arms (CGG ON vs OFF), report per-arm unsupported-claim-reaching-user rate + contradicted-claim rate + over-block rate (benign claim wrongly blocked), under ELAI_SEED, >=5 seeds paired (I-38 floor), bootstrap-CI (bench_convergence). receipts on the y-axis (GD-3). NO LLM-judge.
3. the `hallucination-delta` bench sub (~25 LOC): args_bench.rs:75 arm; the deterministic FIXTURE tier ships now; the live-model arm is RunPod/free-pool-gated (W-10) and refuses to emit a live number until the arm runs (no fabricated delta, ID-5).
4. the calibration write-back (~25 LOC): the measured thresholds (tier boundaries, entropy cutoff, BLOCK-by-default promotion) write to the L-2/L-4/L-7 config (ID-5 — measured, never guessed).
5. workflow/reasoning/L-10-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - benchmarks/hallucination_delta_runner.rs — the ON/OFF ablation runner + the held-out corpus loader + the calibration write-back
  - the `hallucination-delta` bench sub (args_bench.rs:75)
consumes_symbols:
  - G-6 safedelta_runner.rs (the runner shape — COMPOSE, never fork); I-38 QDELTA statistical floor; bench_convergence (bootstrap-CI); E-54 corpus shards; L-2 ClaimRisk + L-7 policy gate (the gate under measurement)
reachable_from:
  - `elai bench hallucination-delta [--scope fixture|live] [--seed] [--json]` [PRODUCTION bench]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO LLM-judge scoring (TD-3 — ground-truth corpus labels + deterministic gate verdicts)
  - NO paper AUROC/rho as the done-gate number (ID-5 — ELAI-measured on the gold set)
  - NO fabricated live delta before the live arm runs (the live number refuses to emit until measured)
  - NO fork of the SAFEDELTA/QDELTA runner (GD-1 P0); NO .unwrap on prod; blake3 not sha2
frontend_pair:
  a hallucination-delta tile on the audit/cost surface (decide at edit, W-4) OR "N/A — the number surfaces via the existing bench/convergence panels. Justified."
observable_effect:
  trigger: "`elai bench hallucination-delta --scope fixture --seed 0 --json` over the held-out claim corpus"
  expect_ssb: "prefer-reuse SAFEDELTA / QDELTA / ConvergenceCurveMeasured (+Kind HallucinationDeltaArm); baseline+0..1 from LIVE ~268/577 (re-grep, W-9)"
  expect_artifact: "one artifact per arm with per-arm unsupported-claim rate + over-block rate, recomputable from the journal (GD-2); NEGATIVE: a `--gate-off-but-claim-savings` broken arm shows the delta COLLAPSE (the gate is the cause); the live arm refuses to emit a number until the RunPod run lands (ID-5)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-10_cgg_hallucination_delta_bench.md --run-trigger
rg "hallucination.?delta|gate_on|gate_off|over.?block" .elai_cc/benchmarks/hallucination_delta_runner.rs   # the ablation present
rg -ci "llm.?judge|gpt.?score|model.*grade" .elai_cc/benchmarks/hallucination_delta_runner.rs              # MUST be 0 (no LLM-judge)
test -f workflow/reasoning/L-10-decisions.md
```

## ACCEPTANCE
- [ ] ID-5: the delta is ELAI-measured on an ELAI gold set; no paper AUROC/rho claimed as ELAI's.
- [ ] TD-3: scoring is deterministic (ground-truth labels + gate verdicts), no LLM-judge.
- [ ] GD-3: receipts on the y-axis (verified-outcome rate), never pass@k.
- [ ] I-38 floor: >=5 seeds, paired, bootstrap-CI.
- [ ] the live arm refuses to emit a number until the RunPod run lands (no fabricated delta).
- [ ] GD-1: the SAFEDELTA/QDELTA runner shape composed, not forked.
- [ ] cargo check --workspace GREEN; auditor PASS on -p the bench crate. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = `elai bench hallucination-delta --scope fixture` emits a recomputable ON/OFF delta (not cargo-green).
- W-2: no behavior gate (a bench); the gate flag is L-7.
- W-3: composes the BUILT SAFEDELTA runner, not a summary-only port.
- W-4: frontend_pair = a delta tile (decide at edit) or justified N/A.
- W-5/W-10: the fixture tier ships now; the live-model arm is RunPod/free-pool-gated (BENCH-PENDING).
- W-7: safedelta_runner.rs + args_bench.rs:75 + E-54 shards re-grepped at edit.
- W-9: SSB prefer-reuse SAFEDELTA/QDELTA/ConvergenceCurveMeasured (+Kind) or +1.
- GD-2: every reported number is recomputable from the journal under ELAI_SEED.
- GD-4: the L-7 BLOCK-by-default promotion is gated on this bench's positive delta + bounded over-block rate.

## Tests to add
- crates/contracts/tests/l10_hallucination_delta_contract.rs (~75 LOC): (a) the fixture ON/OFF run emits a recomputable delta; (b) scoring uses no LLM-judge (assert no model call); (c) a `--gate-off-but-claim-savings` arm collapses the delta (the gate is the cause); (d) the live arm refuses to emit until the RunPod run lands; (e) >=5 seeds paired (I-38 floor); (f) the calibration write-back updates the L-2/L-4/L-7 thresholds from the measured knee (ID-5).

## Files touched
- CREATE: .elai_cc/benchmarks/hallucination_delta_runner.rs (~130 LOC)
- MODIFY: .elai_cc/crates/cli/src/args_bench.rs (~25 LOC: the `hallucination-delta` bench sub)
- CREATE: data/claim_gold/ (the held-out claim corpus shards; per-family, E-54-filtered)
- CREATE: crates/contracts/tests/l10_hallucination_delta_contract.rs (~75 LOC)
- CREATE: workflow/reasoning/L-10-decisions.md

## Citations / Grounding
- arXiv:2606.27326 §5.1 (rho~=0.80 predictor-vs-error, AUROC) [EXTERNAL/DESIGN-RATIONALE — the CGG's own number is measured here, never the paper's]; cgg-hallucination-causal-system-2026-06-27.md S12.
- ELAI ground truth (L-CGG-GROUNDING-MAP NN-7/TS-19): safedelta_runner.rs, args_bench.rs:75, bench_convergence, E-54 (re-grepped at edit, W-7).
- CLAUDE.md: ID-5 (ELAI-measured, never a paper number), TD-3 (no LLM-judge), HD-1 (every effect number measured per config), no .unwrap on prod, blake3-only.

## Open questions / risks
- The gold-set label source: the supported/unsupported/contradicted labels must be ground-truth (a human-curated or executable oracle), NOT an LLM-judge (TD-3). For coding claims the oracle is largely deterministic (file/symbol/dep/test exists); for behavior/causal claims the L-5 do-test outcome is the oracle. Decide the label provenance per ClaimKind at authoring.
- Over-block weighting: the delta must net unsupported-claim-reduction AGAINST over-block friction (a benign claim wrongly blocked); report BOTH, never just the reduction (the gate-killer the K-4 ceiling guards). The BLOCK-by-default promotion (L-7) is gated on both being acceptable.

## Next
L-10 closes the L-plan: the CGG ships default-ON (L-7) only when this measured delta is positive and the over-block rate is bounded. It feeds the K-1 dossier (a real hallucination-reduction number for the claims policy, KD-1) and pairs with G-6 SAFEDELTA (the security sibling) — the trust spine now has a measured hallucination axis alongside its security axis.

End of L-10.
