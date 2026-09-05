# D-28: Adversarial example farming for PPM (DEFERRED — research-grade)

Objective: iteratively generate OOD high-proxy-reward low-quality examples
per arxiv:2504.06141 (T1); retrain PPM (D-2) on adversarial corpus to
immunize against gaming. Companion §3 Phase 4H.X (deferred): "hard to
bootstrap until BAI is running."
Estimated effort: 4-6 sessions wall-clock + RunPod hours.
Status: DEFERRED.
Pre-requisites: D-5 (BAI promotion has ≥20 stable cycles), D-19 (PRMBench
  baseline measured). RunPod (4090 or A100 for PPM fine-tune).
Parallel-safe with: D-29, D-30 (all deferred; independent).

## Context

Companion §3 deferred block (lines 405-408): adversarial example farming for
RM/PPM. Iteratively:
1. Generate candidate reasoning steps that score HIGH on current PPM but
   are LOW quality per deterministic check (compile fail, test fail, formal
   contradiction).
2. Mine the gap between proxy reward (PPM score) and gold (deterministic).
3. Retrain PPM on this adversarial corpus with negative labels.
4. Re-measure PRMBench Deception Resistance + Circular Logic; goal +5pp
   improvement.

Hard to bootstrap because:
- Requires stable BAI selection signal (D-5 needs ≥20 promotion cycles).
- Requires PRMBench baseline (D-19) to measure delta.
- Requires GPU for PPM fine-tune (RunPod 4090 / A100; cost-throttled).

## Reads

- F:\.ELAI_workflow\docs\specs\guide\reasoning-improvement-grounded-2026-05-19.md
  (§3 Phase 4H.X — lines 405-408)
- F:\.ELAI_workflow\workflow\papers\prmbench-2501.03124-interface.md (D-2)
- F:\.ELAI_workflow\workflow\papers\rstar-math-2501.04519-interface.md (D-2)
- workflow/papers/rm-overoptimization-2210.10760-interface.md (D-4)
- D-19 PRMBench report (baseline measurements)

## Paper-ingestor pre-step

Spawn paper-ingestor with mode=paper for:
- arxiv:2504.06141 Adversarial Training Reward Models

Verify workflow/papers/adversarial-rm-2504.06141-interface.md.

## Tasks

1. **Adversarial example generator** at .elai_cc/python/elai_inference/
   ppm_adversarial.py:
   - Per arxiv:2504.06141 §3: search step space for HIGH-PPM low-deterministic
     candidates.
   - Procedure: sample candidate; compute (proxy_reward, gold_reward); if
     proxy >> gold AND gold = Fail → adversarial example.
   - Store at .elai_cc/calibration/ppm_adversarial/{ts}/{step_id}.jsonl.

2. **PPM fine-tune script** at workflow/remote-setup/ppm_finetune.sh:
   - Run on RunPod 4090 or A100.
   - Loads Qwen2.5-Math-PRM-7B; adds adversarial corpus with negative labels;
     LoRA fine-tune (rank=16, epochs=3, lr=1e-5; conservative).
   - Output: ppm_v2.safetensors; deploy to remote sidecar.

3. **Pre/post-immunization PRMBench measurement**:
   - Run D-19 PRMBenchRunner on v1 (baseline); save report.
   - Deploy v2; run PRMBenchRunner; save report.
   - Compare deltas on Deception Resistance + Circular Logic.

4. **Re-validation gate**:
   - Standard PRMBench categories must NOT regress >2pp (defense against
     fine-tune overfitting on adversarial).
   - Deception + Circular Logic must improve ≥+5pp (gate).

5. **Rollback procedure**:
   - If immunization regresses standard categories or fails to improve
     adversarial → revert to v1; log adversarial corpus + decision; do not
     deploy v2.

6. **SSB action types** (atomic with appropriate SCHEMA bump if D-28 runs):
   - `AdversarialCorpusGenerated { count: u32, source_session: String }`
   - `PpmFinetuneCompleted { v1_score: f64, v2_score: f64, delta: f64,
     verdict: String }`

7. **Decision doc** workflow/reasoning/D-28-decisions.md (DEFERRED — drafted
   at D-28 unblock time):
   - (a) Why DEFERRED: needs stable BAI + baseline + RunPod; not in pre-
     production scope.
   - (b) Why LoRA fine-tune (not full): smaller rank conservative; risk-
     bounded; matches PPM domain-narrow scope.
   - (c) Why ≥+5pp gate: arxiv:2504.06141 reports +5-7pp typical; +5pp is
     conservative.

## WIRING_CONTRACT

produces_symbols:
  - AdvRmFarmer (farm/adv_rm.rs:335) + AdvRmFarmer::run (farm/adv_rm.rs:440)
  - AdvRmFarmRunReport (farm/adv_rm.rs:226) + AdvRmSeed + AdvRmCorpusEntry
  - verdict (farm/adv_rm.rs:650) -> GatingVerdict
  - SsbActionType::AdversarialFarmRunCompleted + AdversarialFarmRunCompletedPayload (ssb.rs)
  # NOTE: plan-doc named variants AdversarialCorpusGenerated + PpmFinetuneCompleted;
  # both REJECTED in shipping pass; actual variant is AdversarialFarmRunCompleted.
  # ppm_adversarial.py (python/elai_inference/ppm_adversarial.py) is the Python producer;
  # fine-tune script (workflow/remote-setup/ppm_finetune.sh) is RUNPOD-PENDING.
consumes_symbols:
  - D-5 BAI promotion cycles (prerequisite: >=20 stable cycles before farming)
  - D-19 PRMBenchRunner (baseline delta measurement)
  - SidecarPpmHook (runtime sidecar PPM scoring hook)
  - darwinian_bai (D-5; flat BAI baseline for comparison)
reachable_from:
  - CLI `elai farm-adv-rm run` (cli/src/farm_adv_rm.rs:124)
  - CLI `elai farm-adv-rm verdict` (cli/src/farm_adv_rm.rs:213)
  # RUNPOD-PENDING: python ppm_adversarial.py --live requires sidecar on RunPod 4090/A100
stub_ban:
  - AdvRmFarmer MUST emit AdversarialFarmRunCompleted SSB row per run (audit trail)
  - verdict MUST require report.passed() to be computed; not mocked
  - ppm_adversarial.py MUST NOT run in --mock mode for production corpus generation
observable_effect:
  trigger: elai farm-adv-rm run --dataset <prmbench_seed_path> --corpus-dir .elai_cc/benchmarks/adv_rm/corpus/
  expect_ssb: 1 AdversarialFarmRunCompleted row with samples_farmed > 0
  expect_artifact: .elai_cc/benchmarks/adv_rm/corpus/ populated with AdvRmCorpusEntry JSONL
  # RUNPOD-PENDING: PPM fine-tune + post-immunization PRMBench delta legs require
  # RunPod 4090/A100; D-5 >=20 BAI cycles; D-19 baseline published.
  # The CLI run + SSB emit legs ARE wired (farm_adv_rm.rs + ssb.rs:6710 confirmed live).

## DETERMINISTIC_CHECK

DEFERRED — populated at D-28 unblock time.

## ACCEPTANCE

- [ ] DEFERRED. Sessions unblock when: D-5 has ≥20 BAI cycles AND D-19
      baseline is published.
- [ ] RunPod 4090 / A100 access confirmed.
- [ ] arxiv:2504.06141 paper-ingestor output present.
- [ ] PPM v2 deployment + post-immunization PRMBench report.
- [ ] Adversarial gain ≥+5pp on Deception + Circular Logic.
- [ ] Standard category regression ≤2pp.
- [ ] Decision doc workflow/reasoning/D-28-decisions.md.

## Files touched (anticipated)

- ADD: .elai_cc/python/elai_inference/ppm_adversarial.py
- ADD: workflow/remote-setup/ppm_finetune.sh
- ADD: .elai_cc/calibration/ppm_adversarial/{ts}/ (corpus dir)
- MODIFY: workflow/remote-setup/start-ppm-sidecar.sh (point to v2 model
    when ready)
- ADD: workflow/reasoning/D-28-decisions.md
- ADD: workflow/papers/adversarial-rm-2504.06141-interface.md

## Out-of-scope

- Full PPM rebuild (DEFERRED beyond D-28; LoRA suffices)
- Multi-model adversarial corpus (DEFERRED)

## Next

When unblocked: D-29 (hierarchical BAI) is independent; can run in parallel
on different RunPod session.
