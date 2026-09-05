# H-39: Abstention-aware promotion + eval scoring — stop the self-improvement loop selecting for confident hallucination [TRACK 6 KEYSTONE]

STATUS: PLANNED (TRACK 6 GROUNDED-REASONING HARDENING; WAVE 6; W-10 tiered: in-process scorer + measured SSB row Tier-1; the bench live-run is a separately-gated checkbox)

Objective: ELAI ships the ABSTENTION capability (conformal_calibration.rs RefusalDecision::Abstain, D-16) and the
honest-refusal SLA (refusal_sla.rs, D-17), but the reward the AUTONOMOUS loop OPTIMIZES is abstention-blind:
promotion_gate.rs::two_gate_promote ranks an ArchiveEntry by score+tau, bai.rs::sample ranks ArmPosteriors by
pass/fail, and coding_metrics pass_at_k is pure pass/total. None credits a correct abstention or penalizes a
confident-wrong answer MORE than an abstention. Per OpenAI 2509.04664 (Observation 1) a binary-graded reward makes
abstention NEVER optimal, so the self-mod loop (darwin_admit_loop) + BAI promotion + GEPA will SELECT FOR confident
guessing — actively breeding hallucination into the archive while D-16/D-17 try to suppress it at inference. H-39
builds one AbstentionAwareScore (credit IDK=0, correct=+1, confident-wrong = -t/(1-t), report the abstention rate),
reads the abstain bit from the shipped RefusalDecision (GD-1), and feeds it into the promotion reward + the bench
scorers so the optimized objective matches the refusal layer. This is the single cross-subsystem MISALIGNMENT the
Hallucination-Stations research surfaced (RD-1).

Estimated effort: ~1 session (M, ~6-8 h). W-10 SPLIT: Tier-1 = the in-process AbstentionAwareScore + the
  AbstentionScored SSB row + the promotion-reward wire (fail-closed done-gate); Tier-2 = the live bench run over the
  local Qwen lane (ships a deterministic fixture harness now; the live RunPod/sidecar run is a separately-gated checkbox).
Pre-requisites: NONE hard (all substrate SHIPPED). Composes D-16 (conformal RefusalDecision), D-17 (refusal_sla),
  the promotion loop (promotion_gate/config_archive/bai/darwin_admit_loop), coding_metrics, the seccodeplt/SAFEDELTA
  measured-row pattern. Reads RS-4/5/6/7/8 in H-REASONING-GROUNDING-MAP.md.
Parallel-safe with: H-40, H-41, H-42 (disjoint files). Sequence H-39 FIRST in Track 6 (the keystone the others assume).
Source: the Hallucination-Stations research thread (2026-06-04) + OpenAI 2509.04664; _grounding/H-REASONING-GROUNDING-MAP.md
  RG-39 + RD-1 + the CITATIONS block. Doctrine: H-0_INDEX.md (W-1..W-10 + GD-1/2/4/5 + HD-1..HD-5 + RD-1).

## Context

OpenAI's "Why Language Models Hallucinate" (2509.04664) proves the COMMON hallucinations are low-complexity factual
ones sustained NOT by a computational ceiling but by EVALUATION INCENTIVES: under any binary grader, IDK earns 0
expected credit while any guess with p_correct>0 has strictly positive expected value (Observation 1), so a model —
and any loop that optimizes that grade — is rewarded for guessing over abstaining (Table 2: GPQA/MMLU-Pro/BBH/
SWE-bench/HLE all binary-grade with zero IDK credit). The paper's concrete fix is a confidence-target grading
protocol: "answer only if >t confident, penalty = t/(1-t)"; t in {0.5 -> 1x, 0.75 -> 2x, 0.9 -> 9x}. ELAI already
HAS the abstain primitive (RS-4) and the refusal SLA (RS-5); the gap (RG-39) is that the OPTIMIZED reward path
(RS-6/RS-7/RS-8) ignores it. H-39 closes the loop: the reward the optimizer climbs is the same abstention-credited
objective the refusal layer enforces, so D-16/D-17 stop fighting D-5/E-8/E-49.

## Existing state (B/C/D/E/F/G coverage)
- conformal_calibration.rs RefusalDecision {Answer, Abstain} + decide/claim_score + tau (D-16) — SHIPPED. The abstain
  bit H-39 consumes (GD-1, do NOT re-derive an abstain signal).
- refusal_sla.rs RefusalSlaConfig + refusal_rate + band_lo/hi (D-17) — SHIPPED. The session-level rate H-39 reports against.
- promotion_gate.rs:109 two_gate_promote + config_archive.rs (insert_variant/select_parent) + bai.rs:286 sample(&[ArmPosterior])
  + darwin_admit_loop.rs:298 run_one_generation — SHIPPED. The promotion REWARD path; abstention-blind today (RG-39).
- coding_metrics/{collector,aggregator,types}.rs pass_at_k / record_pass_sample / pass_at_1 — SHIPPED. Pure pass/total;
  the abstention-aware accumulator is the sibling H-39 adds.
- benchmarks/safedelta_runner.rs + bench_seccodeplt.rs (SecCodePltRunner) + InsecureRateMeasured SSB — SHIPPED. The
  frozen-runner + measured-row pattern H-39's AbstentionScored row MIRRORS (GD-1; do NOT build a new runner shape).
- AbstentionAwareScore / abstention-credited reward — ABSENT (RG-39). The pure-BUILD surface.

## Reads (mandatory before coding)
- _grounding/H-REASONING-GROUNDING-MAP.md: RG-39, RD-1, RS-4/5/6/7/8, the CITATIONS block, the SCHEMA block.
- workflow/papers/why-llms-hallucinate-2509.04664-interface.md (Observation 1; Theorems 1-3 floors; the t/(1-t) protocol).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\conformal_calibration.rs (RefusalDecision — the abstain source)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\promotion_gate.rs:109-230 (two_gate_promote — the reward to extend)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\coding_metrics\collector.rs:100-180 (record_pass_sample — the accumulator)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\benchmarks\safedelta_runner.rs:230-490 (the frozen-runner + measured-row pattern)
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod; TD-style determinism; RD-1.

## Tasks (WAVE 6 — Tier-1 scorer + reward wire + measured row)

1. **AbstentionAwareScore (~70 LOC, crates/orchestrator/src/abstention_score.rs).** A pure fn over a per-item
   outcome `{answered: bool, correct: bool, refusal: RefusalDecision}` returning a scalar: `correct & answered ->
   +1.0`; `refusal == Abstain (answered=false) -> 0.0`; `answered & !correct -> -penalty` where `penalty = t/(1-t)`
   for a LOCKED confidence target `t` (default 0.75 -> 2x, per 2509.04664). Aggregate to {mean_score,
   abstention_rate, confident_wrong_rate, n}. Deterministic; no wall-clock; the abstain bit READ from RefusalDecision
   (GD-1). `t` is a LOCKED constant (a self-mod loop cannot tune the penalty toward guessing — TD-2-style freeze).

2. **Abstention-aware accumulator in coding_metrics (~40 LOC).** A sibling to record_pass_sample that records the
   outcome triple and exposes `abstention_aware_score()` alongside pass_at_k (no breaking change; pass_at_k stays).

3. **Feed the promotion reward (~30 LOC).** Where two_gate_promote / the ArchiveEntry score bai::sample ranks is
   computed, use the abstention-aware score as the (or a LOCKED-weighted component of the) reward, so darwin_admit_loop
   ranks an abstaining variant ABOVE a confident-wrong one. Do NOT rewrite two_gate_promote's tau gate; extend the
   score it consumes (GD-1).

4. **Off-agent measured row (~30 LOC).** Emit `{mean_score, abstention_rate, confident_wrong_rate, t, n, seed}` as
   an AbstentionScored SSB row (Pattern-2; baseline+1 from LIVE 220/497, re-read ssb.rs:2990 + action_type_lock.rs:624
   at edit; the FIRST Track-6 bump also fixes the stale e45 pin). Reuse the InsecureRateMeasured measured-row pattern.

5. **Bench-scorer pass (~40 LOC).** Add an abstention-aware scoring pass to the bench path (safedelta/seccodeplt
   pattern) so `elai bench <suite>` reports the abstention-adjusted score next to pass@1, and append it to the phase TOML.

6. **Decision doc workflow/reasoning/H-39-decisions.md (~80 LOC):** the t/(1-t) protocol + the LOCKED default t;
   Observation-1 grounding; why the abstain bit is READ from RefusalDecision not re-derived (GD-1); the honest residue
   (this aligns the reward with the refusal layer — it does NOT eliminate hallucination, only stops REWARDING it; GD-5).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/abstention_score.rs::abstention_aware_score (the scorer)
  - crates/orchestrator/src/abstention_score.rs::AbstentionAggregate (the {mean_score, abstention_rate, ...} roll-up)
  - crates/orchestrator/src/ssb.rs::SsbActionType::AbstentionScored (off-agent reward artifact; baseline+1, Pattern-2)
consumes_symbols:
  - crates/orchestrator/src/conformal_calibration.rs::RefusalDecision (the abstain bit — REUSE, GD-1)
  - crates/orchestrator/src/refusal_sla.rs::refusal_rate (the session SLA — compose, GD-1)
  - crates/orchestrator/src/promotion_gate.rs::two_gate_promote (the reward path the score feeds — extend, not rewrite)
  - crates/orchestrator/src/coding_metrics/collector.rs::record_pass_sample (the accumulator — add a sibling)
  - crates/orchestrator/src/benchmarks/safedelta_runner.rs (the frozen-runner + measured-row pattern — mirror, GD-1)
reachable_from:
  - the eval/bench entrypoint (`elai bench <suite>`) -> abstention_aware_score -> AbstentionScored SSB row [PRODUCTION]
  - the promotion path (darwin_admit_loop::run_one_generation -> two_gate_promote) reads the abstention-aware reward
stub_ban:
  - NO reward an autonomous loop optimizes may score confident-wrong == abstain (RD-1; that is the breeding signal) — FAIL
  - NO re-derived abstain signal; the bit comes from the shipped RefusalDecision (GD-1 P0)
  - NO agent-tunable penalty t (LOCKED; a loop tuning t->0 restores guessing — freeze it)
  - the broken `--reward-confident-guessing` arm MUST flip the ranking (R2: a metric that does not change when broken is a FAIL)
  - blake3 / no .unwrap()/.expect() in prod
frontend_pair:
  the abstention rate + abstention-adjusted score in the cost/trust cockpit (composes H-9 / G-5; cross_lang_diff PASS).
  W-6 E2E: a promotion that an abstaining variant WINS over a confident-wrong one is visible in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- bench <suite-with-a-frozen-fixture-containing-a-confident-wrong
    variant V1 and a correct-abstaining variant V2>, NO env override"
  expect_ssb: "an AbstentionScored row lands carrying {mean_score, abstention_rate, confident_wrong_rate, t, n},
    recomputable from the journal under ELAI_SEED (GD-2), with score(V2) > score(V1)"
  expect_artifact: "the promotion path ranks V2 (abstains correctly) ABOVE V1 (confident-wrong); the broken
    `--reward-confident-guessing` arm flips it (V1 wins), proving the metric is not vacuous (R2)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-39_abstention_aware_promotion_eval.md --run-trigger
rg "abstention_aware_score|AbstentionScored" .elai_cc/crates/orchestrator/src --glob '!*test*'   # produced + wired
rg "RefusalDecision" .elai_cc/crates/orchestrator/src/abstention_score.rs                          # abstain bit reused (GD-1)
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/abstention_score.rs                           # MUST be 0
test -f workflow/reasoning/H-39-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py --run-trigger PASS (V2 ranked above V1; AbstentionScored row landed). cargo-green != DONE.
- [ ] W-2 default-ON: the abstention-aware score is the optimized reward with no env override; kill-switch only.
- [ ] W-3 production entrypoint: the reward fires from the live bench + promotion paths, not a #[cfg(test)] caller.
- [ ] RD-1: confident-wrong is penalized strictly more than abstain (penalty=t/(1-t)>0); the abstain bit is from RefusalDecision.
- [ ] GD-1: RefusalDecision + two_gate_promote + the measured-row pattern REUSED (no re-implementation; P0 if violated).
- [ ] GD-2 reproducible: the {mean_score, abstention_rate, ...} aggregate re-computes byte-identically under ELAI_SEED.
- [ ] R2: the --reward-confident-guessing broken arm flips the ranking (the metric is falsifiable, not vacuously stable).
- [ ] W-10: Tier-1 in-process scorer is the fail-closed done-gate; the live bench run is a separately-gated checkbox.
- [ ] cargo check --workspace GREEN; auditor PASS.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; the scorer (produced) feeds the promotion reward (W-8) reached by the bench/promotion entrypoint (W-3).
- W-2: optimized reward with no override; kill-switch only.
- W-3: live bench + promotion paths; not test-only.
- W-4: frontend_pair = the abstention-rate/score cockpit tile; cross_lang_diff PASS.
- W-5: observable_effect is a runnable bench -> a real journal row; Tier-1 in-process.
- W-6: an abstaining variant winning a promotion is visible in the dashboard (dashboard-smoker).
- W-7: conformal RefusalDecision / two_gate_promote / coding_metrics / safedelta_runner grep-confirmed (RS-4/6/7/8); AbstentionAwareScore absent (RG-39).
- W-8: abstention_aware_score (produced) consumed by the promotion reward + the bench scorer.
- W-9: +AbstentionScored baseline+1 (Pattern-2); the first Track-6 bump fixes the stale e45 pin.
- W-10: SPLIT — Tier-1 scorer fail-closed; Tier-2 live bench fixture + checkbox.
- GD-1/2 + RD-1: compose; reproducible; abstention-credited, penalty-LOCKED.

## Tests to add
- crates/contracts/tests/h39_abstention_score_contract.rs (~150 LOC): (a) correct-answer=+1, abstain=0,
  confident-wrong=-t/(1-t); (b) score(correct-abstain) > score(confident-wrong) at default t; (c) the promotion path
  ranks the abstaining variant above the confident-wrong one; (d) the --reward-confident-guessing arm flips it (R2);
  (e) t is LOCKED (cannot be set at runtime); (f) ELAI_SEED twice -> identical aggregate.
- crates/contracts/tests/fixtures/abstention/ (a frozen eval set with both variants + their RefusalDecision rows).

## Files touched
- CREATE: crates/orchestrator/src/abstention_score.rs (~70 LOC)
- MODIFY: crates/orchestrator/src/coding_metrics/collector.rs (+abstention-aware accumulator, ~40 LOC)
- MODIFY: the promotion reward site (promotion_gate.rs / darwin_admit_loop.rs, ~30 LOC: feed the score)
- MODIFY: crates/orchestrator/src/ssb.rs (+AbstentionScored) + action_type_lock.rs:624 + e45:294 (the stale pin)
- MODIFY: the bench path (safedelta/seccodeplt scorer, ~40 LOC: emit the abstention-adjusted score + TOML field)
- CREATE: crates/contracts/tests/h39_abstention_score_contract.rs (~150 LOC)
- CREATE: workflow/reasoning/H-39-decisions.md (~80 LOC)

## Citations / Grounding
- OpenAI 2509.04664 "Why Language Models Hallucinate" (Observation 1: binary grading makes abstention never optimal;
  Theorems 1-3 error floors from singleton rate; the confidence-target protocol penalty=t/(1-t), t in {0.5,0.75,0.9}).
  WebSearch-confirm the arxiv id + the t/(1-t) formula before locking the default (feedback_paper_citation_verify).
- ELAI ground truth (RS-4/5/6/7/8, grep-confirmed 2026-06-04): conformal_calibration RefusalDecision; promotion_gate.rs:109
  two_gate_promote; coding_metrics pass_at_k; safedelta_runner.rs the measured-row pattern.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); GD-5 (this aligns the reward; it does not eliminate hallucination).

## Open questions / risks
- The "correct" bit for a free-form coding outcome is itself the eval's verdict (pass/fail from the harness). H-39 does
  NOT redefine correctness; it ADDS the abstain/confident-wrong dimension on top of the existing pass/fail. For tasks
  with no abstain channel (the model always answers), abstention_rate is 0 and the score reduces to a penalty-weighted
  pass rate — still strictly better-aligned than raw pass@k.
- Default t: 0.75 (penalty 2x) is the paper's middle recommendation; the right t for ELAI's lanes is a VERIFY before
  locking (cite quote+page). Too-high t (0.9, penalty 9x) can over-suppress answers (the D-17 overcautious band catches this).
- This is necessary-not-sufficient: it stops the loop REWARDING confident hallucination; the inference-time suppression
  is still D-16/D-17's job. Do not advertise it as a hallucination cure (GD-5 / RD-2 honest-ceiling).

## Next
H-40 (decorrelation-keyed verifier retention) and H-41 (live checker-coverage instrument) are the other two
verification-quality refinements; H-42 (serial-depth reasoning budget) is the reasoning-budget one. H-39 is the
keystone because the promotion reward it fixes is what the whole self-improvement loop (D-5/E-7/E-8) climbs.

End of H-39.
