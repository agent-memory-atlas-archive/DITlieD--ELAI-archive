# H-48: Wire the BAI bandit on the real reward — activate Discounted-TS + Successive-Rejects regret-bounded selection on H-43's signal [TRACK 7]

STATUS: PLANNED (TRACK 7 DGM SELF-IMPROVEMENT VERIFICATION HARDENING; WAVE 7; W-10 tiered: in-process bandit update/sample on the real reward + measured row Tier-1; the live multi-arm regret measurement is a separately-gated checkbox)

Objective: ELAI ships the BAI structs (bai.rs:286 sample(&[ArmPosterior]), bai_hierarchical.rs D-29) but per the
dossier the darwinian_bai sample/update/run loop is SCAFFOLD-ONLY — promotion does not yet use regret-bounded
selection on a REAL reward (GAP-4). It could not, because the reward WAS a byte-hash (DG-1). Once H-43 supplies an
execution-grounded reward and H-45 aggregates it worst-case, the bandit can finally run on a signal that means
something. H-48 ACTIVATES the Discounted-Thompson-Sampling (2305.10718) + Successive-Rejects (Audibert COLT 2010)
selection on H-43/H-45's reward, and seeds the arm/curriculum from ExIt (2509.04575) partial-history guard tasks.
This is COMPOSITION/ACTIVATION of D-5/D-29 (GD-1; H-48 is the wiring on the real signal, NOT a new bandit), and it
is the lowest-priority Track-7 step (P2) because it is meaningless until H-43 lands.

Estimated effort: ~1 session (M, ~6-8 h). W-10 SPLIT: Tier-1 = the in-process bandit update/sample on H-45's
  aggregated reward over a FROZEN multi-arm fixture + the measured row (fail-closed done-gate); Tier-2 = the live
  multi-generation regret measurement (RUNPOD-PENDING per the dossier; fixture now, live a gated checkbox).
Pre-requisites: H-43 (the real reward — HARD), H-45 (the worst-case aggregate the arm-reward reads). COMPOSES D-5
  (BAI Thompson promotion), D-29 (hierarchical BAI). Reads DG-5, PC-2, VD-1/VD-4.
Parallel-safe with: none in Track 7 (depends H-43 + H-45). Sequence LAST in Track 7.
Source: ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md (GAP-4 P2; DOSSIER 1 ExIt; PART C/F step 6; "Bandit
  already interface-mapped: Discounted-TS 2305.10718; Successive Rejects Audibert COLT 2010");
  _grounding/H-VERIFICATION-GROUNDING-MAP.md DG-5 + PC-2.

## Context

The dossier's GAP-4: darwinian_bai.rs Discounted-TS + Successive-Rejects are scaffold (sample/update/run RUNPOD-
PENDING); promotion does not yet use regret-bounded selection. The fix is not a new bandit — D-5/D-29 already plan
it and the structs exist (ArmPosterior Beta(a,b); bai.rs:286 sample; bai_hierarchical sample_theta/best-arm). It is
wiring the update/sample loop on a REAL reward. Discounted-TS (2305.10718) handles non-stationarity (the loop's
reward distribution shifts as the archive evolves); Successive-Rejects (Audibert COLT 2010) is the fixed-budget
best-arm identifier. ExIt (2509.04575, Meta) generates an autocurriculum from PARTIAL intermediate episode states —
ELAI's guard/trap tasks (H-44) can be seeded from partial states of past generations, not only terminal benchmarks.
H-48 is strictly downstream: a regret-bounded selector over a byte-hash reward is theatre (VD-1); over H-45's
worst-case execution-grounded aggregate it is the convergent selection the dossier's PART F step 6 names.

## Existing state (B/C/D/E/F/G coverage)
- bai.rs:286 sample(&[ArmPosterior]) + bai.rs:83 ArmPosterior + update/insert helpers — SHIPPED structs; the sample/update/run LOOP is scaffold per the dossier (DG-5, GAP-4).
- bai_hierarchical.rs (D-29) sample_theta:392 + best-arm:512/607/661 — SHIPPED. The hierarchical variant H-48 can drive.
- D-5 BAI Thompson promotion — PLANNED. H-48 ACTIVATES it on the real reward (compose, do not re-author).
- H-43 execution_grounded_fitness + H-45 aggregate_promotion_score — PLANNED (Track 7). The real reward the bandit reads (hard pre-reqs).
- ExIt 2509.04575 partial-history curriculum — research-only (no repo); seeds the arm/guard-task set.
- A live bandit update/sample loop on a real reward + ExIt curriculum seeding — ABSENT/scaffold (GAP-4). The activation surface.

## Reads (mandatory before coding)
- _grounding/H-VERIFICATION-GROUNDING-MAP.md: DG-5, PC-2, VD-1, VD-4, the SCHEMA + CITATIONS blocks.
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\bai.rs:69-300 (ArmPosterior + sample + update) + bai_hierarchical.rs (D-29 hierarchical)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\promotion_score.rs (H-45 — the aggregated reward the arm reads)
- docs/thefinalglance/D-plan/D-5_bai_thompson_promotion.md + D-29_hierarchical_bai_research.md (the bandit substrate to activate — compose)
- workflow/reasoning/ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md (GAP-4 + PART C/F step 6)
- Discounted-TS 2305.10718 + ExIt 2509.04575 PDFs; Audibert Successive-Rejects (COLT 2010).
- CLAUDE.md HARD RULES: no .unwrap()/.expect(); deterministic SeededRng (the bandit already uses SeededRng).

## Tasks (WAVE 7 — Tier-1 bandit update/sample on the real reward + curriculum seed + measured row)

1. **Update the arm posteriors from the real reward (~70 LOC, in/near darwinian_bai).** On each generation, update
   the candidate's ArmPosterior with H-45's aggregated worst-case reward (a success/failure or a Beta update derived
   from the aggregate), NOT a byte-hash or a placeholder (VD-1). Use Discounted-TS discounting for non-stationarity (2305.10718).

2. **Drive selection by sample() on the live arms (~40 LOC).** The generation's next-arm choice runs through
   bai.rs:286 sample (or the D-29 hierarchical best-arm) on the updated posteriors — the regret-bounded selection the
   scaffold deferred. Compose D-5; do not re-author the bandit (GD-1).

3. **Successive-Rejects fixed-budget best-arm (~40 LOC).** Under a fixed generation budget, run Successive-Rejects
   (Audibert COLT 2010) to identify the best arm — the dossier's named fixed-budget identifier.

4. **ExIt curriculum seeding (~50 LOC).** Seed the arm/guard-task set from PARTIAL intermediate states of past
   generations (ExIt 2509.04575), not only terminal benchmarks — compose H-44's trap/guard tasks (a guard task from a
   partial past state). This is the autocurriculum leg.

5. **Off-agent measured row (~30 LOC) — conditional variant.** Emit the arm update/selection as a measured row. PREFER
   REUSING D-5's existing BAI promotion telemetry (GD-1; minimise bumps). If darwinian_bai has NO emitter, add
   +BanditArmSampled (Pattern-2; baseline+1 from LIVE 224/501, re-read at edit; reconcile e45 pin if first Track-7
   bump). The author decides at edit; default = reuse.

6. **Decision doc workflow/reasoning/H-48-decisions.md (~70 LOC):** why this is LAST (meaningless before H-43);
   Discounted-TS for non-stationarity; Successive-Rejects fixed budget; ExIt curriculum; what is COMPOSED (D-5/D-29)
   vs activated; the honest residue (regret bounds assume a stationary-enough reward; the live multi-arm regret number is RUNPOD-PENDING Tier-2; VD-4/GD-5).

## WIRING_CONTRACT
produces_symbols:
  - the bandit update/select wire in darwinian_bai (update_arms_from_reward + select_next_arm — the activated loop)
  - crates/orchestrator/src/ssb.rs::SsbActionType::BanditArmSampled (CONDITIONAL: only if D-5 has no reusable emitter; baseline+1, Pattern-2)
consumes_symbols:
  - crates/orchestrator/src/bai.rs::sample (the SHIPPED Thompson sampler — ACTIVATE on the real reward, GD-1)
  - crates/orchestrator/src/bai.rs::ArmPosterior (the SHIPPED posterior to update)
  - crates/orchestrator/src/bai_hierarchical.rs (D-29 — the hierarchical best-arm option)
  - crates/orchestrator/src/promotion_score.rs::aggregate_promotion_score (H-45 — the real reward the arm reads)
  - crates/orchestrator/src/execution_fitness.rs::execution_grounded_fitness (H-43 — the underlying signal)
reachable_from:
  - the self-mod loop selection (darwin_admit_loop::run_one_generation -> update_arms_from_reward -> select_next_arm via bai::sample) [PRODUCTION, darwinian kill-switch, allowlisted]
stub_ban:
  - NO bandit update on a byte-hash / placeholder reward (VD-1; the arm reads H-45's execution-grounded aggregate) — FAIL
  - NO scaffold sample/update left unwired after H-43+H-45 land (the regret-bounded selection must actually run) — FAIL
  - NO re-implementation of the bandit (compose D-5/D-29 — GD-1 P0)
  - the broken `--bandit-reward-from-bytes` arm MUST converge selection to a failing arm (R2)
  - no .unwrap()/.expect() in prod; SeededRng not wall-clock RNG
frontend_pair:
  the per-arm posterior + the selected arm in the DGM/archive drawer (composes the archive panel; cross_lang_diff PASS). [N/A acceptable if D-5 already ships the panel — justify]
  W-6 E2E: the bandit converging selection to the genuinely-best arm (high execution-fitness) is visible in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && ELAI_DARWINIAN_LOOP_ENABLE=1 cargo run -p elai-cli -- <the self-mod generation subcommand>
    over a FROZEN multi-arm fixture where arm G has the high execution-grounded reward and arm B has low, NO other env override"
  expect_ssb: "the bandit telemetry (reused D-5 row or BanditArmSampled) shows selection converging to G over
    generations; recomputable under ELAI_SEED (GD-2)"
  expect_artifact: "select_next_arm converges to G; the broken `--bandit-reward-from-bytes` arm converges to B
    (the byte-hash favourite), proving the reward wiring is load-bearing (R2)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-48_wire_bandit_on_real_reward.md --run-trigger
rg "update_arms_from_reward|select_next_arm" .elai_cc/crates/orchestrator/src --glob '!*test*'
rg "aggregate_promotion_score|execution_grounded_fitness" .elai_cc/crates/orchestrator/src/bai*.rs   # the bandit reads the real reward (H-45/H-43)
test -f workflow/reasoning/H-48-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (selection converges to G; telemetry landed). cargo-green != DONE.
- [ ] W-2: the bandit selection is the default under the darwinian kill-switch; no new default-off gate.
- [ ] W-3 production entrypoint: update/sample fire from run_one_generation, not a #[cfg(test)] caller.
- [ ] VD-1: the arm reward is H-43/H-45's execution-grounded aggregate, never a byte-hash.
- [ ] GAP-4: the scaffold sample/update/run is ACTIVATED (regret-bounded selection actually runs).
- [ ] GD-1: bai::sample + ArmPosterior + D-5/D-29 REUSED (no new bandit; P0 if violated); SSB row reused where possible.
- [ ] GD-2 reproducible: the arm posteriors + selection re-compute byte-identically under ELAI_SEED.
- [ ] R2: the --bandit-reward-from-bytes broken arm converges to the failing arm (falsifiable).
- [ ] W-10: Tier-1 update/sample on the frozen fixture is the fail-closed done-gate; the live multi-arm regret measurement is a separately-gated checkbox (RUNPOD-PENDING).
- [ ] cargo check --workspace GREEN; auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; the activated update/select (produced) reached by run_one_generation (W-3), reading the real reward (W-8).
- W-2: default selection under kill-switch; allowlisted.
- W-3: live selection step; not test-only.
- W-4: frontend_pair = the arm-posterior tile (or justified N/A if D-5 ships it); cross_lang_diff PASS.
- W-5: observable_effect is a runnable generation over a frozen multi-arm fixture -> real telemetry; Tier-1 in-process.
- W-6: the bandit converging to the best arm is visible in the dashboard (dashboard-smoker).
- W-7: bai::sample / ArmPosterior / bai_hierarchical / promotion_score / execution_fitness grep-confirmed (DG-5/H-45/H-43); the activated loop is scaffold (GAP-4).
- W-8: update_arms_from_reward + select_next_arm (produced) consume H-45's aggregate; selection drives the generation.
- W-9: SSB row REUSED from D-5 where possible; +BanditArmSampled only if no reusable emitter (conditional baseline+1); reconcile e45 pin if first Track-7 bump.
- W-10: SPLIT — Tier-1 fixture loop fail-closed; Tier-2 live regret measurement + checkbox (RUNPOD-PENDING).
- VD-1/VD-4 + GD-1/2: real-reward; compose the bandit; reproducible.

## Tests to add
- crates/contracts/tests/h48_bandit_real_reward_contract.rs (~150 LOC): (a) arm posteriors update from H-45's
  aggregate, not a hash; (b) sample converges to the high-execution-fitness arm over a fixed budget; (c)
  Successive-Rejects identifies the best arm under the budget; (d) the --bandit-reward-from-bytes arm converges to the
  failing arm (R2); (e) ExIt seeds a guard task from a partial past state; (f) ELAI_SEED twice -> identical posteriors/selection.
- crates/contracts/tests/fixtures/bandit_real_reward/ (a frozen multi-arm fixture G/B with execution-grounded rewards + a partial-history curriculum seed).

## Files touched
- MODIFY: darwinian_bai (activate update_arms_from_reward + select_next_arm on the real reward, ~110 LOC; compose D-5/D-29, do not re-author)
- MODIFY: the selection step in darwin_admit_loop (route select_next_arm, ~30 LOC)
- MODIFY (conditional): crates/orchestrator/src/ssb.rs (+BanditArmSampled) + action_type_lock.rs:645 (+ e45:294 if first Track-7 bump) — ONLY if D-5 has no reusable emitter
- MODIFY (or N/A): the DGM/archive dashboard drawer (+arm posterior, ~30 LOC) + its types.ts pair — N/A if D-5 ships it
- CREATE: crates/contracts/tests/h48_bandit_real_reward_contract.rs (~150 LOC)
- CREATE: workflow/reasoning/H-48-decisions.md (~70 LOC)

## Citations / Grounding
- Discounted-Thompson-Sampling 2305.10718 (non-stationary bandit — the loop's reward shifts as the archive evolves). Successive-Rejects (Audibert/Bubeck COLT 2010; fixed-budget best-arm identification — the dossier's named identifier).
- ExIt 2509.04575 (Jiang/Lupu/Bachrach, Meta; autocurriculum from partial intermediate episode histories — seed guard tasks from partial past states). Self-Verification-Prevents-Collapse (the bandit-on-a-real-filter is the convergent branch).
- ELAI ground truth (DG-5, grep-confirmed 2026-06-05): bai.rs:286 sample + ArmPosterior; bai_hierarchical (D-29); composes D-5 (PLANNED). Reads H-43/H-45 (Track 7).
- CLAUDE.md HARD RULES: no .unwrap()/.expect(); deterministic SeededRng.

## Open questions / risks
- Regret bounds assume a stationary-enough reward; the archive's evolution makes the reward non-stationary, which is
  why Discounted-TS (not vanilla TS) is used. If the discount is mis-tuned the bandit chases noise; tune on the frozen
  fixture, never at runtime; the live regret number is RUNPOD-PENDING Tier-2 (same constraint as the dossier's PART F note).
- This is the LOWEST-priority Track-7 step (P2): a regret-bounded selector over a placeholder reward is theatre. It MUST
  NOT ship before H-43 (real reward) + H-45 (aggregate) — the stub_ban enforces "no bandit update on a byte-hash reward".
- Prefer reusing D-5's telemetry over a new SSB variant; a new variant is conditional and only if no emitter exists (minimise bumps, GD-1).

## Next
- With H-43..H-48 the DGM loop has: execution-grounded fitness (H-43), a behavioral cheat-detector with measured
  recall (H-44), worst-case lineage-diverse promotion scoring (H-45), clade-metaproductivity selection (H-46), an
  eval firewall (H-47), and regret-bounded bandit selection on the real reward (H-48) — the real, un-gameable,
  stronger-than-generator verification filter the META-FINDING requires for the loop to PROVABLY CONVERGE rather than
  collapse. E-53's Verus/Kani proof gate is the narrow un-gameable ceiling on top (VD-5, never routine).

End of H-48.
