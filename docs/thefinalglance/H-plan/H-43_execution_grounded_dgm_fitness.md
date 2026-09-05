# H-43: Execution-grounded DGM fitness — replace the byte-hash derived_alpha with a measured pass-rate signal [TRACK 7 KEYSTONE]

STATUS: PLANNED (TRACK 7 DGM SELF-IMPROVEMENT VERIFICATION HARDENING; WAVE 7; W-10 tiered: in-process scorer + measured SSB row Tier-1; the live multi-generation bench run is a separately-gated checkbox)

Objective: ELAI's self-improvement loop optimizes a fitness that is a function of the candidate's BYTES, not its
PERFORMANCE. darwin_admit_loop.rs:186 `derived_alpha` returns `blake3(config_blob)` first-8-bytes normalized to
[0,1] (read this session, DG-1) — a stable scalar OF the candidate, but with ZERO causal connection to whether the
candidate produces working code. A config that ships failing code can score 0.99; one that ships correct code can
score 0.01. The dossier's META-FINDING (Self-Verification-Prevents-Collapse, NeurIPS X5Hk8aMs6w; Skalse 2209.13085)
makes this the DIVERGENT branch: a recursive loop whose verification filter is a no-op proxy PROVABLY collapses.
H-43 replaces derived_alpha with an EXECUTION-GROUNDED fitness — CodeRM-style pass-rate over generated unit tests
(2501.01054, the TOP PICK) combined with CodeT dual-execution agreement (2207.10397) — so the signal the loop
climbs is the candidate's measured behavior. This is VD-1 (the keystone the whole track assumes) and the off-agent
fitness E-19 Task 11 (SE-Agent recombination) and H-39 (the abstention reward's "correct" bit) both wait on.

Estimated effort: ~1.5 sessions (M-L, ~8-10 h). W-10 SPLIT: Tier-1 = the in-process execution-grounded scorer over
  a FROZEN deterministic fixture (compiled candidate -> generated tests -> pass-rate) + the FitnessEvaluated SSB
  row + the derived_alpha replacement (fail-closed done-gate); Tier-2 = the live multi-generation bench eval over
  the local Qwen / RunPod lane (the dossier's deferred Tier-2; ships a fixture harness now, live run a gated checkbox).
Pre-requisites: NONE hard (the loop + the safedelta frozen-runner pattern are SHIPPED). Composes D-2 (PPM sidecar —
  the cross-family verifier UPGRADE path, sequenced after), E-19 PRM-lite (the auditor-deterministic step signal),
  E-53 (the narrow proof ceiling, VD-5). Reads DG-1/DG-7, PC-1/PC-6/PC-7/PC-8, VD-1/VD-4 in H-VERIFICATION-GROUNDING-MAP.md.
Parallel-safe with: H-44 (disjoint file: trap_tasks). BLOCKS H-45 (aggregates this signal), H-46 (clade reads this
  signal's subtree deltas), H-48 (bandit on this signal); enables H-39's "correct" bit + E-19 Task 11. Sequence H-43 FIRST in Track 7.
Source: ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md (GAP-1 P0; DOSSIER 4 CodeRM/CodeT/Sol-Ver; PART C/F);
  _grounding/H-VERIFICATION-GROUNDING-MAP.md DG-1 + VD-1 + the CITATIONS block. Doctrine: H-0_INDEX.md (W + GD + VD).

## Context

The dossier's six agents converge: gain is bounded by the verifier-minus-generator gap (Solver-Verifier Gap
2507.00075) and a self-judging loop collapses (Co-rewarding 2508.00410 — 44.81% vs 85.14% ground-truth). The
cheapest un-gameable fitness is EXECUTION: run the candidate's code against tests it did not write and score by
what actually passes. CodeRM (2501.01054, ACL 2025, RUCKBReasoning/CodeRM, WebSearch-confirmed this session) shows
pass-rate over many GENERATED tests is a strong reward signal — +18.43% HumanEval Plus for Llama3-8B via best-of-N
with 100 generated tests, and #tests correlates with reward quality. CodeT (2207.10397) adds dual-execution
agreement (RANSAC-style: score by tests-passed AND output-consensus across solutions) — the agreement MASS is the
fitness. Sol-Ver (2502.14948) co-evolves solutions and verifying tests. H-43 wires the simplest sound version:
derived_alpha becomes the candidate's pass-rate over a FROZEN generated-test set (Tier-1 fixture), upgraded to the
live bench in Tier-2. The verifier runs on a DIFFERENT family than the generator where a cross-family signal is
used (VD-4; the abliterated 7B does not grade its own family). D-2 (PPM sidecar) later upgrades the step-level signal.

## Existing state (B/C/D/E/F/G coverage)
- darwin_admit_loop.rs:186 derived_alpha = blake3(config_blob) -> [0,1] — SHIPPED PLACEHOLDER (DG-1, GAP-1). The REPLACE target.
- darwin_admit_loop.rs:424 run_one_generation + :447 alpha bind + score_candidate guard chain — SHIPPED. The reachable_from path.
- benchmarks/safedelta_runner.rs (SecCodePltRunner, InsecureRateMeasured SSB, bootstrap_ci) — SHIPPED. The FROZEN-runner + measured-row pattern H-43 MIRRORS (GD-1; do NOT build a new runner shape).
- E-19 PRM-lite (auditor-deterministic node score) — PLANNED. A per-step signal H-43 can blend into the fitness once shipped.
- D-2 PPM sidecar (Qwen2.5-Math-PRM-7B @ 8101) — PLANNED. The cross-family step-verifier UPGRADE path (PC-1).
- Execution-grounded fitness / a generated-test pass-rate scorer — ABSENT (GAP-1). The pure-BUILD surface.

## Reads (mandatory before coding)
- _grounding/H-VERIFICATION-GROUNDING-MAP.md: DG-1, DG-7, PC-1/6/7/8, VD-1, VD-4, the SCHEMA + CITATIONS blocks.
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\darwin_admit_loop.rs:176-215 + :424-460 (derived_alpha + run_one_generation)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\benchmarks\safedelta_runner.rs (the frozen-runner + measured-row pattern to mirror)
- workflow/reasoning/ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md (GAP-1 + DOSSIER 4 + PART C/F)
- CodeRM 2501.01054 + CodeT 2207.10397 PDFs (cite in tests + decision doc; WebSearch-confirm before locking numbers)
- CLAUDE.md HARD RULES: blake3 (for the test-set/fixture HASH, not the fitness), no .unwrap()/.expect() in prod.

## Tasks (WAVE 7 — Tier-1 scorer + the derived_alpha replacement + measured row)

1. **ExecutionGroundedFitness scorer (~180 LOC, crates/orchestrator/src/execution_fitness.rs).** A fn over a
   candidate that COMPILES the candidate's produced code and runs it against a FROZEN generated-test set, returning
   `{pass_rate: f64, tests_total, tests_passed, dual_agreement: f64}`. pass_rate is the CodeRM signal; dual_agreement
   is the CodeT cross-solution output-consensus mass. fitness = `w_pass*pass_rate + w_agree*dual_agreement` with
   LOCKED weights (default w_pass=0.7, w_agree=0.3 — a self-mod loop cannot tune the weights toward a gameable
   signal; VD-1/TD-2-style freeze). Deterministic under ELAI_SEED; no wall-clock.

2. **Replace derived_alpha (~30 LOC, darwin_admit_loop.rs).** derived_alpha calls ExecutionGroundedFitness over the
   candidate (Tier-1: the frozen fixture; Tier-2: the live bench). KEEP the `force_alpha` fixture override (tests
   drive a specific margin) and KEEP determinism. DELETE the blake3-of-bytes path as the production fitness (VD-1:
   a byte-hash fitness defaulted-ON is a FAIL). The blake3 import stays ONLY if used for the test-set fixture hash.

3. **Cross-family guard (~30 LOC).** Where a model-graded sub-signal is used (not pure execution), assert the
   verifier model family != the generator family (VD-4; Co-rewarding collapse). Pure execution (compile+run) is
   family-free and is the default; the cross-family assert fires only on the optional model-graded blend.

4. **Off-agent measured row (~30 LOC).** Emit `{pass_rate, dual_agreement, tests_total, fitness, seed, test_set_hash}`
   as a FitnessEvaluated SSB row (Pattern-2; Kind ExecutionPassRate / DualAgreement / FitnessRegression; baseline+1
   from LIVE 224/501, re-read ssb.rs:3010 + action_type_lock.rs:645 at edit; the FIRST Track-7 bump also fixes the
   stale e45 pin at e45_cross_language_drift_contract.rs:294). Reuse the InsecureRateMeasured measured-row pattern (GD-1).

5. **FitnessRegression guard (~20 LOC).** If a candidate's execution-fitness is BELOW its parent's, emit the
   FitnessRegression Kind and the two-gate margin (DG-3) sees a negative margin -> blocked. This is the off-agent
   artifact proving the signal is causal (a failing candidate scores low, R2/W-1).

6. **Decision doc workflow/reasoning/H-43-decisions.md (~90 LOC):** why bytes->execution (the convergence theorem +
   Skalse unhackable-proxy); the LOCKED weights; the D-2/E-19 PRM upgrade path (when D-2 ships, blend the PPM step
   score); the honest residue (execution-grounding is necessary-not-sufficient — a test the candidate also wrote is
   still gameable, so the test set is FROZEN and out-of-band; VD-5 / E-53 proof ceiling for the CSQ-Critical slice).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/execution_fitness.rs::execution_grounded_fitness (the scorer)
  - crates/orchestrator/src/execution_fitness.rs::ExecutionFitness (the {pass_rate, dual_agreement, ...} struct)
  - crates/orchestrator/src/ssb.rs::SsbActionType::FitnessEvaluated (off-agent fitness artifact; baseline+1, Pattern-2)
consumes_symbols:
  - crates/orchestrator/src/darwin_admit_loop.rs::derived_alpha (the REPLACE target — extend the body, keep the signature)
  - crates/orchestrator/src/darwin_admit_loop.rs::run_one_generation (the reachable_from generation entry)
  - crates/orchestrator/src/benchmarks/safedelta_runner.rs (the frozen-runner + measured-row pattern — mirror, GD-1)
  - crates/orchestrator/src/promotion_gate.rs::two_gate_promote (the margin gate now sees a CAUSAL score — compose, do not rewrite)
reachable_from:
  - the self-mod loop entrypoint (darwin_admit_loop::run_one_generation -> derived_alpha -> execution_grounded_fitness -> FitnessEvaluated SSB) [PRODUCTION, behind the cfg(feature="darwinian-evolve")+ELAI_DARWINIAN_LOOP_ENABLE kill-switch — W-2 caveat: the kill-switch is the registered wire-dark-allowlist entry, not a terminal default-off]
stub_ban:
  - NO byte-hash / config-bytes fitness as the production signal (VD-1; that is the divergent branch) — FAIL
  - NO self-graded fitness where the verifier family == the generator family on a model-graded blend (VD-4) — FAIL
  - NO agent-tunable fitness weights (LOCKED; a loop tuning w toward a gameable signal restores the placeholder)
  - the broken `--fitness-from-bytes` arm MUST make a failing candidate score HIGH (R2: a metric that does not change when broken is a FAIL)
  - blake3 only for the test-set fixture hash; no blake2/sha2; no .unwrap()/.expect() in prod
frontend_pair:
  the per-candidate pass-rate + fitness in the DGM/archive drawer (composes H-9 cost cockpit / the archive panel; cross_lang_diff PASS).
  W-6 E2E: a failing-code candidate scoring LOW (and losing promotion) is visible in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && ELAI_DARWINIAN_LOOP_ENABLE=1 cargo run -p elai-cli -- <the self-mod generation subcommand>
    over a FROZEN fixture containing candidate C_good (passes the generated tests) and C_bad (fails them), NO other env override"
  expect_ssb: "two FitnessEvaluated rows land; fitness(C_good) > fitness(C_bad); each recomputable from the journal
    under ELAI_SEED (GD-2); C_bad carries the FitnessRegression Kind"
  expect_artifact: "the promotion path ranks C_good ABOVE C_bad (execution-grounded margin); the broken
    `--fitness-from-bytes` arm inverts it (C_bad's hash wins), proving the metric is causal not vacuous (R2)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-43_execution_grounded_dgm_fitness.md --run-trigger
rg "execution_grounded_fitness|FitnessEvaluated" .elai_cc/crates/orchestrator/src --glob '!*test*'   # produced + wired
rg "blake3::hash\(&?blob" .elai_cc/crates/orchestrator/src/darwin_admit_loop.rs                       # the byte-hash fitness MUST be gone from the prod path
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/execution_fitness.rs                             # MUST be 0
test -f workflow/reasoning/H-43-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py --run-trigger PASS (C_good > C_bad; FitnessEvaluated rows landed). cargo-green != DONE.
- [ ] W-2: execution-grounded fitness is the default signal under the existing darwinian kill-switch; NO new default-off gate; the kill-switch has its wire-dark-allowlist row.
- [ ] W-3 production entrypoint: the fitness fires from run_one_generation, not a #[cfg(test)] caller.
- [ ] VD-1: a config producing FAILING code scores LOW; the byte-hash fitness is removed from the prod path.
- [ ] VD-4: any model-graded blend asserts verifier-family != generator-family; pure execution is the default.
- [ ] GD-1: derived_alpha / two_gate_promote / the safedelta measured-row pattern REUSED (no re-implementation; P0 if violated).
- [ ] GD-2 reproducible: the {pass_rate, dual_agreement, fitness} aggregate re-computes byte-identically under ELAI_SEED.
- [ ] R2: the --fitness-from-bytes broken arm inverts the ranking (the metric is falsifiable).
- [ ] W-10: Tier-1 in-process scorer over the frozen fixture is the fail-closed done-gate; the live bench run is a separately-gated checkbox.
- [ ] cargo check --workspace GREEN; auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; the scorer (produced) feeds derived_alpha (W-8) reached by run_one_generation (W-3).
- W-2: default signal under the existing kill-switch; the kill-switch is allowlisted, not a terminal default-off.
- W-3: live self-mod generation path; not test-only.
- W-4: frontend_pair = the DGM/archive pass-rate tile; cross_lang_diff PASS.
- W-5: observable_effect is a runnable generation over a frozen fixture -> real journal rows; Tier-1 in-process.
- W-6: a failing candidate scoring low + losing promotion is visible in the dashboard (dashboard-smoker).
- W-7: derived_alpha / run_one_generation / safedelta_runner / two_gate_promote grep-confirmed (DG-1/DG-7/DG-3); execution-fitness ABSENT (GAP-1).
- W-8: execution_grounded_fitness (produced) consumed by derived_alpha + (transitively) the promotion margin.
- W-9: +FitnessEvaluated baseline+1 (Pattern-2); the first Track-7 bump fixes the stale e45 pin.
- W-10: SPLIT — Tier-1 scorer fail-closed; Tier-2 live bench fixture + checkbox.
- VD-1/VD-4 + GD-1/2: execution-grounded; cross-family-guarded; compose; reproducible.

## Tests to add
- crates/contracts/tests/h43_execution_fitness_contract.rs (~170 LOC): (a) C_good (passes) > C_bad (fails);
  (b) fitness is monotone in pass_rate; (c) derived_alpha now reflects execution, not bytes (two configs with the
  same pass-rate but different bytes score EQUAL — the inverse of today); (d) the --fitness-from-bytes arm inverts
  it (R2); (e) weights are LOCKED (cannot be set at runtime); (f) ELAI_SEED twice -> identical aggregate;
  (g) FitnessRegression Kind fires when a child scores below its parent.
- crates/contracts/tests/fixtures/execution_fitness/ (frozen candidates C_good/C_bad + the frozen generated-test set + its hash).

## Files touched
- CREATE: crates/orchestrator/src/execution_fitness.rs (~180 LOC)
- MODIFY: crates/orchestrator/src/darwin_admit_loop.rs (derived_alpha body -> execution-grounded, ~30 LOC; keep force_alpha + signature)
- MODIFY: crates/orchestrator/src/ssb.rs (+FitnessEvaluated) + action_type_lock.rs:645 + e45:294 (the stale pin)
- MODIFY: the DGM/archive dashboard drawer (+pass-rate/fitness field, ~40 LOC) + its types.ts pair
- CREATE: crates/contracts/tests/h43_execution_fitness_contract.rs (~170 LOC)
- CREATE: workflow/reasoning/H-43-decisions.md (~90 LOC)

## Citations / Grounding
- CodeRM 2501.01054 "Dynamic Scaling of Unit Tests for Code Reward Modeling" (ACL 2025; RUCKBReasoning/CodeRM;
  pass-rate over generated tests as reward; +18.43% HumanEval Plus Llama3-8B; #tests correlates with reward quality).
  WebSearch-confirmed this session (title + repo match).
- CodeT 2207.10397 (dual-execution agreement, RANSAC-style; 65.8% pass@1 HumanEval, +18.8 abs). Sol-Ver 2502.14948 (co-evolve solutions+tests).
- Self-Verification-Prevents-Collapse (NeurIPS 2025, OpenReview X5Hk8aMs6w — NO arXiv id, do not invent one): a real
  verification filter on the synthetic feedback is the convergence precondition; a no-op/gameable filter is the divergent branch.
- Skalse 2209.13085 (unhackable proxy: raising proxy-return never lowers true-return — a byte-hash fails this trivially).
- Solver-Verifier Gap 2507.00075 (gain bounded by verifier-minus-generator strength). Co-rewarding 2508.00410 (self-judge collapse).
- ELAI ground truth (DG-1/DG-3/DG-7, grep-confirmed 2026-06-05): derived_alpha; two_gate_promote; safedelta_runner.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); GD-5/VD-5 (execution-grounding is necessary-not-sufficient).

## Open questions / risks
- A candidate that writes its OWN passing tests games execution-fitness. Mitigation: the generated-test set is
  FROZEN + out-of-band (CodeRM's generator is separate from the candidate; the fixture is gate-frozen, R8/TD-2). The
  live Tier-2 must generate tests from a SEPARATE model/family (VD-4), never the candidate.
- Tier-1 fixture pass-rate is a proxy for the deferred live bench; the dossier's PART F notes 1+2+5 only become
  MEANINGFUL once the Tier-2 real bench run exists (RunPod). The Tier-1 gate proves the WIRING is causal; the live
  number is the separately-gated checkbox. Do not advertise the Tier-1 fixture pass-rate as the production capability number (HD-1/GD-5).
- LOCKED weights (0.7/0.3) are initial; if dual_agreement proves noisy on ELAI's lane, tune via a frozen calibration
  set (mirror E-19 task-10), never at runtime. Document the calibration date.
- Execution-grounding does not cover semantically-correct-but-spec-violating code; that is H-44's behavioral detector
  (ImpossibleBench conflict canary) + the E-53 proof ceiling for the narrow slice (VD-5).

## Next
- H-45 aggregates this signal worst-case + lineage-diverse; H-46's clade reads this signal's subtree deltas; H-48
  wires the bandit on it; H-39's abstention reward reads the "correct" bit this signal supplies. E-19 Task 11
  (SE-Agent recombination) becomes runnable once this off-agent fitness exists (PC-7). D-2 (PPM sidecar) upgrades the
  step-level signal after H-43; E-53 is the un-gameable proof ceiling for the CSQ-Critical slice (VD-5).

## J-11 AMENDMENT — S-2 GD-1 PRIOR-ART POINTER (source-cited)
Grounding: Arbor `src/coordinator/tools/git_ops.py:402-446` (GitMergeBranchTool.execute), read directly per W-3.
GD-1 prior-art pointer: Arbor's merge gate is a working existence proof of the execution-grounded fitness
pattern H-43 needs. The gate re-runs the held-out eval (B_test) in a detached worktree
(`git_ops.py:390` _run_eval_in_worktree, `:130` separate pool) and uses the VERIFIED score
(`:412` verified_score from the eval output), NOT the LLM-reported score (`:407` logged for comparison only).
No improvement vs trunk/baseline => reject (`:437` is_improvement). This is H-43's core doctrine: fitness =
execution-grounded (VD-1), not LLM-reported. The Arbor gate confirms the pattern informally; H-43 formalizes
it with the dual_agreement + fixture_pass_rate + convergence_delta scoring (LOCKED 0.7/0.3). Compose, do NOT
restate — H-43 owns the fitness function; the Arbor gate is the prior-art citation for "verified eval score
beats LLM-reported score."

End of H-43.
