# H-45: Worst-case + lineage-diverse promotion scoring + PRMBench-calibrated tau — stop promoting on the candidate's own mean [TRACK 7]

STATUS: PLANNED (TRACK 7 DGM SELF-IMPROVEMENT VERIFICATION HARDENING; WAVE 7; W-10 tiered: in-process worst-case/variance scorer + measured SSB row Tier-1; the PRMBench tau-calibration sweep is a separately-gated checkbox)

Objective: promotion_gate.rs:109 `two_gate_promote` admits a candidate when `margin >= tau` where margin is the
candidate's OWN alpha minus the incumbent (often 0.0), tau is a constant the doc self-labels "UNCONFIRMED; carries
forward bai.rs" (DG-3, GAP-3). Two failures: (1) scoring on the candidate's own MEAN signal is exactly the
overoptimization regime — a noisy reward maximised by a single judge drifts off the true objective (Gao 2210.10760;
Coste 2310.02743 shows a conservative WORST-CASE / variance-penalized ensemble "practically eliminates
overoptimization", +70% BoN); (2) ensembling by seed alone does not help — correlated error survives (Eisenstein
2312.09244), and a single self-judge collapses 40 points (Co-rewarding 2508.00410). H-45 EXTENDS the score
two_gate_promote consumes (not the tau/k_m gate itself — GD-1): score on the WORST-CASE / variance-penalized verdict
of judges diversified by PRETRAINING lineage, BLOCK on cross-view disagreement, and CALIBRATE tau on PRMBench
(2501.03124) instead of carrying forward an unconfirmed constant at default-ON. This is VD-3.

Estimated effort: ~1 session (M, ~6-8 h). W-10 SPLIT: Tier-1 = the in-process worst-case/variance aggregator over
  the lineage-diverse verdicts + the PromotionScoreAggregated SSB row + the tau gate wired to the aggregated score
  (fail-closed done-gate); Tier-2 = the PRMBench tau-calibration sweep (fixture now, the live PRMBench run a gated checkbox).
Pre-requisites: H-43 (the execution-grounded SIGNAL the worst-case aggregates — a worst-case over byte-hashes is
  meaningless). Composes D-5 (BAI promotion), D-13 (cross-family lineage views), promotion_gate (extend, do not rewrite).
  Reads DG-3/DG-5, PC-2/PC-5, VD-3/VD-4 in H-VERIFICATION-GROUNDING-MAP.md.
Parallel-safe with: H-46, H-47 (disjoint files). Sequence AFTER H-43 (needs the real signal); compose H-39 (abstention dimension on the same reward).
Source: ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md (GAP-3 P1; DOSSIER 3 Coste/Eisenstein/Co-rewarding/Gao;
  PART C/F); _grounding/H-VERIFICATION-GROUNDING-MAP.md DG-3 + VD-3.

## Context

Gao et al. 2210.10760 is the gold-vs-proxy KL law: "scored better != actually better" past a KL budget. The fix is
conservative ensembling: Coste 2310.02743 (ICLR 2024) shows WCO (worst-case-optimization) / UWO (variance-penalized)
"practically eliminate overoptimization" with +70% BoN; Eisenstein 2312.09244 shows pretrain-seed diversity
generalizes better than finetune-seed and that ensembling does NOT eliminate hacking when errors are correlated — so
the judges must differ by LINEAGE. Co-rewarding 2508.00410 is the empirical floor: a single self-judge collapses
44.81% vs 85.14% ground-truth; two disagreeing views stay stable. PRMBench 2501.03124 (ssmisya/PRMBench) is the
calibration measuring-stick — the best PRM scores only 68.0 and PRMs OVER-ACCEPT, so tau cannot be a guessed
constant; it is calibrated against PRMBench's labeled accept/reject. H-45 wires the aggregator on top of H-43's
execution-grounded per-candidate signal; the verdict feeds the SAME two_gate_promote margin (compose, GD-1).

## Existing state (B/C/D/E/F/G coverage)
- promotion_gate.rs:109 two_gate_promote + tau (:50/60 DEFAULT_TAU, "UNCONFIRMED" :39) + k_m (:54/61) + GateError — SHIPPED. The score-consumer to EXTEND (DG-3, GAP-3).
- config_archive.rs select_parent + sigmoid_perf_score — SHIPPED. H-46's target, not H-45's (do not collide).
- bai.rs sample / ArmPosterior + bai_hierarchical.rs (D-29) — SHIPPED. D-5's bandit; H-48's target. H-45 only changes the SCORE the gate reads.
- H-43 execution_grounded_fitness — PLANNED (Track 7). The per-candidate signal H-45 aggregates (hard pre-req).
- D-13 cross-family adversary — PLANNED. The lineage-diverse views (different pretraining family) H-45 consumes (PC-5).
- H-39 abstention-aware score (Track 6) — PLANNED. ORTHOGONAL: the abstention dimension on the SAME reward; H-45 aggregates, H-39 credits abstention. Compose, do not collide.
- Worst-case / variance-penalized / lineage-diverse aggregator + PRMBench tau calibration — ABSENT (GAP-3). The BUILD surface.

## Reads (mandatory before coding)
- _grounding/H-VERIFICATION-GROUNDING-MAP.md: DG-3, DG-5, PC-2, PC-5, PC-8, VD-3, VD-4, the SCHEMA + CITATIONS blocks.
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\promotion_gate.rs:1-160 (two_gate_promote + tau/k_m + GateError — the extend target)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\execution_fitness.rs (H-43's per-candidate signal — the aggregator input)
- workflow/reasoning/ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md (GAP-3 + DOSSIER 3 + PART C/F)
- Coste 2310.02743 + Eisenstein 2312.09244 + Co-rewarding 2508.00410 + Gao 2210.10760 + PRMBench 2501.03124 PDFs.
- CLAUDE.md HARD RULES: no .unwrap()/.expect(); LOCKED-constant discipline (TD-2 style) for the calibrated tau.

## Tasks (WAVE 7 — Tier-1 aggregator + tau wire + measured row)

1. **Worst-case / variance-penalized aggregator (~120 LOC, crates/orchestrator/src/promotion_score.rs).** Given the
   per-candidate signal from >=2 lineage-diverse verdicts, return `aggregate = min(verdicts) - kappa*var(verdicts)`
   (WCO + UWO; Coste 2310.02743) with a LOCKED kappa. Expose {worst_case, variance, mean, disagreement}.

2. **Lineage-diverse verdicts (~50 LOC).** Collect verdicts from views that differ by PRETRAINING family (execution-
   grounded leg from H-43 + a cross-family adversary verdict via D-13). Assert family-distinctness (VD-4; Eisenstein —
   seed-diversity is insufficient). disagreement = spread across families.

3. **Disagreement BLOCK (~30 LOC).** If cross-view disagreement exceeds a LOCKED band, BLOCK promotion (Co-rewarding:
   two views must agree) — LineageDiverseDisagree. A single-view-only candidate (no second family available) is
   penalized, not auto-passed (VD-4).

4. **Wire the aggregated score into two_gate_promote (~30 LOC).** The margin two_gate_promote compares to tau now uses
   the worst-case aggregate, NOT the candidate's own mean alpha. Do NOT rewrite the tau/k_m gate logic (GD-1; extend
   the score it consumes, the same discipline H-39 uses).

5. **PRMBench-calibrated tau (~40 LOC).** Replace the carried-forward "UNCONFIRMED" DEFAULT_TAU with a value
   CALIBRATED on PRMBench-style labeled accept/reject (target a chosen precision/recall point; Harvey-Liu t>3.0
   analog). tau is a LOCKED constant surfaced with a calibrated_at date (a self-mod loop cannot tune tau toward
   easier promotion — TD-2 freeze). The calibration sweep is Tier-2; the Tier-1 gate ships the locked calibrated value.

6. **Off-agent measured row (~30 LOC).** Emit `{worst_case, variance, mean, disagreement, tau, calibrated_at, seed}`
   as a PromotionScoreAggregated SSB row (Pattern-2; Kind WorstCaseScored / VariancePenalized / LineageDiverseDisagree;
   baseline+1 from LIVE 224/501, re-read at edit; reconcile e45 pin if first Track-7 bump). Reuse the InsecureRateMeasured pattern (GD-1).

7. **Decision doc workflow/reasoning/H-45-decisions.md (~80 LOC):** the overoptimization KL law (Gao); WCO/UWO over
   mean (Coste); lineage > seed (Eisenstein); the disagreement block (Co-rewarding); the calibrated-vs-guessed tau;
   the honest residue (worst-case over a finite ensemble approaches but never reaches the oracle verifier — Solver-Verifier Gap; VD-4/VD-5).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/promotion_score.rs::aggregate_promotion_score (the worst-case/variance aggregator)
  - crates/orchestrator/src/promotion_score.rs::PromotionScore (the {worst_case, variance, disagreement} struct)
  - crates/orchestrator/src/ssb.rs::SsbActionType::PromotionScoreAggregated (off-agent scoring artifact; baseline+1, Pattern-2)
consumes_symbols:
  - crates/orchestrator/src/promotion_gate.rs::two_gate_promote (the margin now reads the aggregate — EXTEND, do not rewrite the gate, GD-1)
  - crates/orchestrator/src/execution_fitness.rs::execution_grounded_fitness (H-43 — one lineage verdict)
  - crates/orchestrator/src/promotion_gate.rs::tau (the DEFAULT_TAU now CALIBRATED + LOCKED)
  - crates/orchestrator/src/benchmarks/safedelta_runner.rs (the measured-row pattern — mirror, GD-1)
reachable_from:
  - the promotion path (darwin_admit_loop::run_one_generation -> two_gate_promote reads aggregate_promotion_score -> PromotionScoreAggregated SSB) [PRODUCTION, darwinian kill-switch, allowlisted]
stub_ban:
  - NO promotion on the candidate's own MEAN signal (VD-3; worst-case/variance over lineage-diverse views) — FAIL
  - NO seed-only ensemble where a cross-family view is available (VD-4; Eisenstein correlated-error) — FAIL
  - NO carried-forward UNCONFIRMED tau at default-ON (VD-3; tau is calibrated + LOCKED with a date) — FAIL
  - NO agent-tunable tau or kappa (LOCKED; a loop loosening tau restores easy promotion)
  - the broken `--score-own-mean` arm MUST promote a candidate the worst-case aggregate blocks (R2)
  - no .unwrap()/.expect() in prod
frontend_pair:
  the worst-case score + cross-view disagreement + the calibrated tau in the DGM/trust drawer (composes the archive panel; cross_lang_diff PASS).
  W-6 E2E: a candidate that wins on its own mean but LOSES on the worst-case aggregate is visibly blocked in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && ELAI_DARWINIAN_LOOP_ENABLE=1 cargo run -p elai-cli -- <the self-mod generation subcommand>
    over a FROZEN fixture with candidate M (high mean, high variance across families) and S (lower mean, low variance), NO other env override"
  expect_ssb: "a PromotionScoreAggregated row per candidate; worst_case(S) > worst_case(M) despite mean(M) > mean(S);
    M carries a high disagreement; recomputable under ELAI_SEED (GD-2)"
  expect_artifact: "two_gate_promote admits S and BLOCKS M on the worst-case margin; the broken `--score-own-mean`
    arm flips it (M wins), proving the aggregation is load-bearing (R2)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-45_worstcase_lineage_promotion_scoring.md --run-trigger
rg "aggregate_promotion_score|PromotionScoreAggregated" .elai_cc/crates/orchestrator/src --glob '!*test*'
rg "calibrated_at|DEFAULT_TAU" .elai_cc/crates/orchestrator/src/promotion_gate.rs   # tau calibrated + dated, not "UNCONFIRMED"
test -f workflow/reasoning/H-45-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (S admitted, M blocked on worst-case; rows landed). cargo-green != DONE.
- [ ] W-2: the aggregated score is the default promotion input under the darwinian kill-switch; no new default-off gate.
- [ ] W-3 production entrypoint: the aggregate fires from two_gate_promote on the live promotion path, not a #[cfg(test)] caller.
- [ ] VD-3: promotion scores on worst-case/variance over lineage-diverse views; tau is calibrated + LOCKED with a date.
- [ ] VD-4: verdicts differ by pretraining family; cross-view disagreement BLOCKS.
- [ ] GD-1: two_gate_promote + tau + execution_fitness + the measured-row pattern REUSED (no gate rewrite; P0 if violated).
- [ ] GD-2 reproducible: the {worst_case, variance, disagreement} aggregate re-computes byte-identically under ELAI_SEED.
- [ ] R2: the --score-own-mean broken arm promotes a worst-case-blocked candidate (falsifiable).
- [ ] W-10: Tier-1 aggregator + locked calibrated tau is the fail-closed done-gate; the PRMBench calibration sweep is a separately-gated checkbox.
- [ ] cargo check --workspace GREEN; auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; the aggregator (produced) feeds two_gate_promote (W-8) reached by run_one_generation (W-3).
- W-2: default promotion input under kill-switch; allowlisted.
- W-3: live promotion path; not test-only.
- W-4: frontend_pair = the worst-case/disagreement/tau tile; cross_lang_diff PASS.
- W-5: observable_effect is a runnable generation over a frozen fixture -> real journal rows; Tier-1 in-process.
- W-6: a high-mean candidate blocked on worst-case is visible in the dashboard (dashboard-smoker).
- W-7: two_gate_promote / tau / execution_fitness / safedelta_runner grep-confirmed (DG-3/DG-5/DG-7); the aggregator ABSENT (GAP-3).
- W-8: aggregate_promotion_score (produced) consumed by two_gate_promote's margin.
- W-9: +PromotionScoreAggregated baseline+1 (Pattern-2); reconcile the e45 pin if first Track-7 bump.
- W-10: SPLIT — Tier-1 aggregator + locked tau fail-closed; Tier-2 PRMBench calibration + checkbox.
- VD-3/VD-4 + GD-1/2: worst-case; lineage-diverse; calibrated tau; compose; reproducible.

## Tests to add
- crates/contracts/tests/h45_promotion_score_contract.rs (~170 LOC): (a) worst_case = min - kappa*var; (b)
  worst_case(S) > worst_case(M) when mean(M)>mean(S) but var(M)>>var(S); (c) cross-view disagreement above the band
  BLOCKS (LineageDiverseDisagree); (d) two_gate_promote reads the aggregate not the mean; (e) tau is LOCKED + carries
  calibrated_at; (f) the --score-own-mean arm flips the verdict (R2); (g) ELAI_SEED twice -> identical aggregate.
- crates/contracts/tests/fixtures/promotion_score/ (frozen M/S verdict sets across >=2 families + the locked tau + the PRMBench-style calibration slice).

## Files touched
- CREATE: crates/orchestrator/src/promotion_score.rs (~170 LOC)
- MODIFY: crates/orchestrator/src/promotion_gate.rs (wire the aggregate into the margin; calibrate + lock tau with a date, ~50 LOC; do NOT rewrite the gate)
- MODIFY: crates/orchestrator/src/ssb.rs (+PromotionScoreAggregated) + action_type_lock.rs:645 (+ e45:294 if first Track-7 bump)
- MODIFY: the DGM/trust dashboard drawer (+worst-case/disagreement/tau, ~40 LOC) + its types.ts pair
- CREATE: crates/contracts/tests/h45_promotion_score_contract.rs (~170 LOC)
- CREATE: workflow/reasoning/H-45-decisions.md (~80 LOC)

## Citations / Grounding
- Coste 2310.02743 (ICLR 2024; WCO/UWO conservative ensemble "practically eliminates overoptimization", +70% BoN; UWO penalizes intra-ensemble variance).
- Eisenstein 2312.09244 "Helping or Herding?" (pretrain-seed ensembles generalize better than finetune-seed; ensembling does not eliminate correlated-error hacking — diversify by LINEAGE).
- Co-rewarding 2508.00410 (single self-judge collapses 44.81% vs 85.14%; two disagreeing views stay stable — the disagreement block). Gao 2210.10760 (gold-vs-proxy KL overoptimization law).
- PRMBench 2501.03124 (ssmisya/PRMBench; best PRM 68.0, PRMs over-accept — the tau calibration measuring-stick). Solver-Verifier Gap 2507.00075 (worst-case over a finite ensemble approaches but never reaches the oracle).
- ELAI ground truth (DG-3/DG-5/DG-7, grep-confirmed 2026-06-05): two_gate_promote + tau "UNCONFIRMED"; bai sample; safedelta_runner.
- CLAUDE.md HARD RULES: no .unwrap()/.expect(); LOCKED-constant (TD-2) for tau/kappa.

## Open questions / risks
- A worst-case over only 2 views is fragile if both views are correlated despite different families (Eisenstein warns
  even lineage-diverse error can correlate). Mitigation: report disagreement honestly; where a 3rd family is cheap
  (D-13 + the execution leg + a PPM via D-2), add it. The aggregate is necessary-not-sufficient (VD-4/VD-5).
- PRMBench-calibrated tau is only as good as PRMBench's labels (best PRM 68.0). The calibrated tau is a defensible
  default, not a proof of correct admission; re-calibrate when the lane's model changes. Document calibrated_at.
- kappa (variance penalty) trades false-rejects for safety; tune on the frozen calibration slice, never at runtime;
  GD-4 — bound the over-block rate on >=3 runs before default-on.

## Next
- H-46 changes WHICH parent the loop expands (clade-metaproductivity) — orthogonal to H-45's WHETHER-to-promote;
  H-48 wires the bandit on H-43's signal that H-45 aggregates; D-2's PPM adds a third lineage verdict; E-53's proof
  gate is the un-gameable verdict for the CSQ-Critical slice (VD-5). H-39's abstention dimension composes on the same reward.

End of H-45.
