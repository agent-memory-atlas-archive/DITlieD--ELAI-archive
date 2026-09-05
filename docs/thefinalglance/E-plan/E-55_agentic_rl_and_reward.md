# E-55: Deferred RL ladder — RLAIF judge + GEPA reflective prompt evolution + hybrid PRM+ORM in BAI reward + MONA/composite-reward hacking guards; SFT-on-rollouts as the practical next rung; DPO/PPO/GRPO weight-update HARD-DEFERRED until BAI saturates
Objective: D-5's BAI Thompson sampling on typed-ReasoningTemplate is ALREADY soft contextual RL (logarithmic regret per Agrawal-Goyal; Wikipedia: Thompson sampling is "a reinforcement learning policy"). E-55 ships the right rungs of the RL ladder for ELAI's actual state: (i) RLAIF judge using cloud Sonnet 4.6 as the rater on local-Qwen rollouts (highest-leverage, zero weight-update, per Anthropic CAI arXiv:2212.08073 + RLAIF Lee 2023 arXiv:2309.00267); (ii) GEPA-style reflective prompt evolution on D-5's template arms (ICLR 2026 oral, arXiv:2507.19457 — 35x fewer rollouts than PPO/GRPO); (iii) hybrid PRM (Qwen2.5-Math-PRM-7B already wired per D-2) + verifiable ORM (build/test/probe outcome) in BAI reward fusion (+5-12pp BoN per VisualPRM / OR-PRM); (iv) MONA + composite-reward guards against reward hacking (arXiv:2501.13011 + arXiv:2509.15557). DEFERRED: weight-level RL fine-tune (DPO/PPO/GRPO/KTO) until BAI provably saturates AND a dense verifiable reward exists. SFT-on-rollouts (SWE-Gym pattern, 491 traces → +12.3/+13.6pp on SWE-Bench Lite/Verified per ICML 2025 arXiv:2412.21139) is the FIRST escalation rung BEFORE any RL fine-tune — likely beats the entire RL ladder for ELAI's current state. HARD-AVOID DPO entirely (Apple 2025: 3-7pp OOD drop; gradient imbalance arXiv:2502.20847; ELAI has no paired data anyway).
Estimated effort: 3 sessions, ~10-12 h (RLAIF + GEPA + reward fusion + MONA guards); SFT-on-rollouts is a separate follow-up (deferred until rollout pool ≥200 GREEN sessions)
Pre-requisites: D-5 PLANNED (BAI promotion: Beta-posterior Thompson sampling — provides the contextual-RL backbone E-55 augments; per D-SUMMARY_INDEX line 46). D-2 PLANNED (PPM client — provides the Qwen2.5-Math-PRM-7B step scores E-55 fuses with ORM). D-3 PLANNED (VerificationResult aggregator — provides the verifier surface that becomes the ORM signal in Task 3). D-13 PLANNED (Adversary cloud-call — provides the cloud-LLM surface RLAIF judge reuses; per D-SUMMARY_INDEX line 100). B-37 SHIPPED (commit fd5c2e5 — provides PerRoleRuntime so cloud Sonnet 4.6 routing is already wired). D-28 DEFERRED (Adversarial PPM farming — coordinates with weight-update deferral; per D-SUMMARY_INDEX line 196).
Parallel-safe with: E-51 (edge inference — disjoint), E-52 (LSP integration — coordinates on verifier signal that feeds ORM), E-53 (formal verification — verifier signal feeds ORM), E-54 (eval corpus — held-out provides reward gate). EXTENDS D-5 BAI + D-2 PPM + D-3 verifier consensus + D-13 Adversary. HARD-DEFERS D-28 Adversarial farming (DEFERRED in D-plan stays DEFERRED until BAI saturates + RL fine-tune greenlight).
Source report: workflow/improvement-sweep-2026-05-19/reports/55-agentic-rl-and-reward.md

## Context

The May 2026 agentic-RL survey establishes that ELAI's current loop (BAI Thompson sampling on typed-ReasoningTemplate + Qwen2.5-Math-PRM-7B scoring + tripwires per D-5/D-2 planned surface) already sits in the soft contextual-RL regime — Wikipedia entry on Thompson sampling explicitly calls it "a reinforcement learning policy"; Agrawal-Goyal proved logarithmic regret for stochastic MAB. So D-5 already does an RL update at the template-arm level. The 2025-2026 evidence on weight-level escalation is mixed: "SFT memorizes, RL generalizes" (arXiv:2501.17161) holds for math/code with verifiable rewards (GRPO/DAPO + RLVR), but on agent traces SWE-Gym (arXiv:2412.21139, ICML 2025) reports plain SFT on 491 trajectories already buys +12.3/+13.6pp on SWE-Bench Lite/Verified — no RL needed. Apple 2025 (machinelearning.apple.com/research/reward-generalization) reports DPO RM drops 3% mean / 7% max on 5 OOD settings; arXiv:2502.20847 documents gradient imbalance in DPO. KTO (arXiv:2402.01306) works on unpaired binary signals (matches ELAI's audit BLOCK/PASS shape) and beats DPO on MT-Bench-1 / Vicuna / AdvBench per Feb 2025 federated study (arXiv:2502.14187). The RLAIF pattern (Lee et al arXiv:2309.00267; Anthropic CAI arXiv:2212.08073) replaces the human preference-data bottleneck with cloud-LLM judges — comparable or superior to RLHF on summarization + dialogue at fraction of label cost; ELAI's cloud Sonnet 4.6 is already wired per memory:project_b37_shipped.md. GEPA (arXiv:2507.19457, ICLR 2026 oral) reports 35x fewer rollouts than PPO/GRPO on prompt-optimization frontier — explicitly outperforms gradient methods on template-level evolution. Hybrid PRM+ORM beats either alone (VisualPRM-8B +5.9pp BoN over ORM-only; OR-PRM +12.5pp BoN absolute). MONA (arXiv:2501.13011) and Verifiable Composite Rewards (arXiv:2509.15557) are reward-hacking guards: MONA = myopic optimization with non-myopic approval prevents multi-step reward hacking; composite reward = base - penalty_format - penalty_length - penalty_sycophancy. JitRL evidence: code generation BoN-64 = 90% pass vs test-time training = 30.6% pass — over-sharpening collapses diversity; gradient updates kill diversity that BoN exploits. AlphaProof (arXiv:2406.15829 / Nature 2025) does test-time RL per problem on Lean variants — but Lean has perfect verifier (RLVR binary ground truth); ELAI doesn't have that universally. The right primitive ranking for ELAI today: RLAIF judge > GEPA prompt evolution > PRM+ORM fusion > MONA guards > SFT-on-rollouts (if/when 200+ GREEN sessions collected) > KTO+RLVR (only after BAI provably saturates) >> DPO (never — wrong primitive for ELAI's data shape).

## Existing state (B/C/D coverage)

- D-5 PLANNED (BAI promotion: Beta-posterior Thompson sampling; per D-SUMMARY_INDEX line 46): provides the contextual-RL backbone. EXTENSION: E-55 feeds RLAIF judge scores + GEPA-evolved template arms + PRM+ORM fused reward into the Beta-posterior update. Existing template-arm selection unchanged additively.
- D-2 PLANNED (PPM client — Qwen2.5-Math-PRM-7B; per D-SUMMARY_INDEX line 140): provides the PRM step-scores E-55 fuses. EXTENSION: E-55 adds the ORM signal source + the fusion (Task 3); D-2 step-score surface unchanged.
- D-3 PLANNED (VerificationResult aggregator; per D-SUMMARY_INDEX line 34): the verifier consensus from D-3 (deterministic / ppm / intrinsic / cross_context + E-53's Z3SmtCheck) becomes the ORM signal in Task 3. CONSUMPTION: D-3's consensus_rule output → fed into hybrid reward.
- D-13 PLANNED (Adversary cloud-call; per D-SUMMARY_INDEX line 100): provides the cloud Sonnet 4.6 surface RLAIF judge reuses. EXTENSION: E-55's RLAIF judge is a NEW SPECIALIZATION of D-13's cloud-call pathway (different prompt, different rating schema); D-13's existing Mode B routing unchanged.
- D-23 PLANNED (Sycophancy detection benchmark; per D-SUMMARY_INDEX line 162): the matched-pair sycophancy probe from E-54 Task 6 feeds the `penalty_sycophancy` term in MONA composite reward (Task 4).
- D-28 DEFERRED (Adversarial PPM farming; per D-SUMMARY_INDEX line 196): coordinates with the RL fine-tune deferral. EXTENSION: E-55 explicitly KEEPS D-28 DEFERRED — its bootstrap requires BAI to have stable selection signal AND ≥20 promotion cycles per D-plan note.
- B-37 SHIPPED (Multi-Runtime Backend Abstraction): PerRoleRuntime + cloud Sonnet 4.6 routing already wired. CONSUMPTION: E-55's RLAIF judge invokes via PerRoleRuntime::cloud_sonnet without new transport code.
- E-8 SHIPPED if exists (Self-evolving prompts): GEPA-style evolution may overlap; verify via Read on `E-8_self_evolving_prompts.md`. COORDINATION: E-55's GEPA tasks consume E-8's prompt-evolution surface if present.
- No existing RLAIF code, no GEPA implementation, no MONA reward guards, no SFT-on-rollouts pipeline: all NEW additions.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\55-agentic-rl-and-reward.md` (full)
- `F:\.ELAI_workflow\docs\thefinalglance\D-plan\D-SUMMARY_INDEX.md` lines 30-110 (D-2, D-3, D-5, D-13, D-23) + lines 195-210 (D-28 deferred)
- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\E-plan\E-8_self_evolving_prompts.md` (coordinate on prompt-evolution surface if shipped — verify via Read first)
- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\E-plan\E-54_eval_corpus_design.md` (held-out + matched-pair surface for ORM + penalty_sycophancy)
- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\E-plan\E-53_formal_verification_codegen.md` (verifier consensus → ORM signal)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\per_role_runtime.rs` lines 50-130 (cloud Sonnet routing surface; per elai.map line 6155)
- `F:\.ELAI_workflow\CLAUDE.md` "Anti-Stale-Training" + "Local LLM" + "No new dependencies without confirmation" rules
- https://arxiv.org/abs/2305.18290 (WebFetch — DPO original; cited per memory:reference_b40_papers.md as verified-paper, used here to JUSTIFY rejection)
- https://machinelearning.apple.com/research/reward-generalization (WebFetch — Apple DPO OOD drop)
- https://arxiv.org/pdf/2502.20847 (WebFetch — Gradient Imbalance in DPO)
- https://arxiv.org/pdf/2402.01306 (WebFetch — KTO prospect-theory loss)
- https://arxiv.org/abs/2502.14187 (WebFetch — Federated KTO vs DPO Feb 2025)
- https://arxiv.org/abs/2412.21139 (WebFetch — SWE-Gym ICML 2025; 491 trajectories +12-14pp)
- https://arxiv.org/abs/2410.01679 (WebFetch — VinePPO NeurIPS 2024)
- https://arxiv.org/pdf/2501.13011 (WebFetch — MONA myopic optimization)
- https://arxiv.org/pdf/2509.15557 (WebFetch — Verifiable Composite Rewards medical reasoning)
- https://arxiv.org/pdf/2501.07301 (WebFetch — Qwen2.5-Math-PRM lessons; MC + LLM-judge consensus filter)
- https://qwenlm.github.io/blog/qwen2.5-math-prm/ (WebFetch — PRM blog; BoN methodology)
- https://arxiv.org/pdf/2507.01352 (WebFetch — Skywork-Reward-V2 8B beats 70B)
- https://arxiv.org/abs/2212.08073 (WebFetch — Anthropic CAI / RLAIF original)
- https://arxiv.org/pdf/2309.00267 (WebFetch — RLAIF vs RLHF Lee et al.)
- https://arxiv.org/pdf/2507.19457 (WebFetch — GEPA ICLR 2026 oral; 35x fewer rollouts than PPO/GRPO)
- https://arxiv.org/html/2508.16546v1 (WebFetch — RL vs SFT panacea-or-mirage)
- https://arxiv.org/pdf/2501.17161 (WebFetch — SFT Memorizes, RL Generalizes)
- https://github.com/SWE-Gym/SWE-Gym (WebFetch — SWE-Gym repo)

## Tasks

1. RLAIF judge using cloud Sonnet 4.6 on local-Qwen rollouts. CREATE `.elai_cc/crates/orchestrator/src/rlaif_judge.rs` (~280 LOC). Surface: `RlaifJudge::rate_trajectory(trajectory: &Trajectory, criteria: &RatingCriteria) -> RlaifRating`. The judge invokes cloud Sonnet 4.6 via PerRoleRuntime::cloud_sonnet (existing B-37 surface) with a structured rating prompt — 5-point Likert across {correctness, step_validity, sycophancy_absence, format_compliance, conciseness}. The judge runs ONLY on trajectory samples (default 1/10 of D-5 BAI rollouts via reservoir sampling — keeps cloud cost bounded per memory:feedback_audit_cycles "honor explicit caps"). Returns `RlaifRating { correctness, step_validity, sycophancy_absence, format_compliance, conciseness, rationale, latency_ms }`. Calibration: per arXiv:2412.12509 JudgeBench / report's Open Q "calibrate cloud-LLM grader against held-out human grades quarterly", ship a quarterly calibration harness that compares Sonnet judge ratings to a 20-task human-graded slice. If correlation <0.6, abort and downgrade RLAIF to advisory-only. Per arXiv:2309.00267 RLAIF matches RLHF at fraction of label cost.

2. GEPA-style reflective prompt evolution on D-5 template arms. CREATE `.elai_cc/crates/orchestrator/src/gepa_template_evolution.rs` (~260 LOC). GEPA pattern (per arXiv:2507.19457): (i) collect bottom-quartile trajectories by D-2 PPM step-score over the last K rollouts (default K=100); (ii) Sonnet reflects on each: "Why did this trajectory underperform? Propose a refined template arm that addresses the failure mode."; (iii) candidate templates retained on Pareto frontier of (estimated mean reward, semantic-diversity vs existing arms) — diversity computed via embedding cosine (reuse E-54 BGE-M3 endpoint). New templates enter D-5's BAI candidate set; old arms with persistently-low posterior expected value are evicted (default at posterior 95th-percentile reward < 50th-percentile of leading arm after 30 trials). Per arXiv:2507.19457 35x rollout efficiency vs PPO/GRPO. Emit `gepa_template_evolved` / `gepa_template_evicted` SSB rows.

3. Hybrid PRM+ORM in BAI reward fusion. CREATE `.elai_cc/crates/orchestrator/src/reward_fusion.rs` (~220 LOC). Surface: `FusedReward { prm_step_scores: Vec<f32>, orm_signal: OrmSignal, fused_reward: f32, weights: FusionWeights }`. `OrmSignal::{BuildGreen, TestsPass, ProbeSemantic, VerifierConsensus(Vec<VerifierKind>)}` — each variant maps to a [0,1] reward (BuildGreen=1.0 on success / 0.0 on fail; TestsPass = fraction of tests passing; ProbeSemantic = E-54 test_executor result; VerifierConsensus = D-3 consensus_rule outcome). FusionWeights default {prm: 0.4, orm: 0.6} — outcome dominates per arXiv:2507.01352 Skywork lesson "quality > volume; outcome > learned PRM" while PRM provides step-level guidance. Or HRM-style hierarchical: PRM filters bottom-50% before ORM tie-breaks. Per VisualPRM +5.9pp BoN + OR-PRM +12.5pp BoN. Wires into D-5 BAI Beta-posterior as the reward observation.

4. MONA + composite-reward hacking guards. CREATE `.elai_cc/crates/orchestrator/src/mona_composite_reward.rs` (~240 LOC). Composite reward = base_reward (from Task 3) - penalty_format - penalty_length - penalty_sycophancy - penalty_repetition. Each penalty is a deterministic measurement:
   - `penalty_format`: 0.1 per detected format violation (regex on expected schema; e.g. JSON parse failure, missing required fields)
   - `penalty_length`: 0.05 if response >2x median session length OR <0.3x median (catches both rambling and refusal-shaped sycophancy)
   - `penalty_sycophancy`: 0.2 if E-54 sycophancy_matched_pair flags `sycophancy_framing_gap_detected` on this trajectory
   - `penalty_repetition`: 0.05 if any consecutive 50-token sequence repeats within the same response (catches loop-stuck failures)
   MONA pattern (arXiv:2501.13011): myopic optimization (single-step reward) MUST be approved by non-myopic critic (step_validity from RLAIF rating in Task 1). If step_validity <0.5 BUT myopic reward >0.8, FLAG potential reward-hacking and emit `reward_hacking_suspected` SSB row. Per arXiv:2509.15557 medical-reasoning composite-reward pattern.

5. SFT-on-rollouts pipeline (DEFERRED-EXECUTION, but ship the data-collection surface). CREATE `.elai_cc/crates/orchestrator/src/rollout_collector.rs` (~200 LOC). Collects GREEN-session trajectories (audit verdict = PASS + matched-pair sycophancy clean + no reward-hacking flag) into `.elai_cc/training/rollouts/{date}/{session_id}.jsonl`. Schema: `{prompt, full_trajectory, ppm_scores, orm_signal, rlaif_rating, fused_reward, final_diff, audit_verdict}`. GATE: SFT fine-tune NOT TRIGGERED until ≥200 rollouts collected per SWE-Gym lower bound (their full corpus was 491; +12.3pp at that scale). At threshold, emit `sft_training_threshold_reached` SSB row + open ticket; ACTUAL SFT-LoRA fine-tune is a SEPARATE patch (E-55.1) — E-55 ships only the collection surface and the threshold trigger. Per arXiv:2412.21139 SWE-Gym.

6. Reward-hacking nightly canary. CREATE `.elai_cc/crates/contracts/tests/reward_hacking_canary.rs` (~160 LOC). Synthetic dataset: 20 "high reward, low quality" trajectories (LLM-generated; human-verified to game the reward signal without actually solving the task — e.g. format-compliant but logically wrong; passes ProbeSemantic test but with degenerate solution like `return None` that satisfies test signature). Test: each trajectory passes through Task 4 composite-reward; MUST trigger `reward_hacking_suspected` for ≥80% (16/20). Runs as nightly canary, NOT per-audit. Per arXiv:2504.06141 Adversarial RM Training (cited in D-plan for D-28); ship the CANARY surface NOW even though D-28 stays DEFERRED.

7. BAI reward observation update + posterior wiring. MODIFY `.elai_cc/crates/orchestrator/src/d5_bai.rs` (verify path via Grep `BAI|ThompsonSampling|Beta` — if D-5 is PLANNED-not-shipped, ship a minimal D-5 surface stub alongside; the Beta-posterior update slot is the integration point). The composite reward from Task 4 becomes the Bernoulli observation for the Beta posterior (success if fused_reward >0.5 after penalties; failure otherwise; report Task 4 caveat: composite reward is a CONTINUOUS observation — for Beta posterior, threshold to binary OR use Beta-Bernoulli with Jeffreys prior on fractional success; ship threshold-to-binary as default, Jeffreys as opt-in). Each arm's Beta(α,β) updates per arm pull; tracking includes the GEPA evolution events from Task 2 (new arms enter with non-informative Beta(1,1) prior). Emit `bai_arm_observation_recorded` row per update.

8. Weight-update fine-tune is HARD-DEFERRED — ship the DEFERRAL marker + escalation triggers. CREATE `.elai_cc/crates/orchestrator/src/rl_weight_update_deferred.rs` (~120 LOC). The module is intentionally a STUB-WITH-CONDITIONS (per CLAUDE.md stub registry rule, registered with babystep id `E-55.B7` in `workflow/sessions/{id}/stubs.md`). It enumerates the THREE escalation triggers required before weight-level RL begins:
   (i) BAI provably saturated: ≥3 sessions where BAI converged to a stable arm AND tripwire stack stopped firing BUT performance still misses target (per report condition 1);
   (ii) Dense verifiable reward available: ORM signal from Task 3 covers ≥80% of trajectories (per report condition 2 — Goodhart guaranteed without this);
   (iii) Local model VRAM headroom: per memory:project_qwen3b_baseline.md, current 12GB ceiling leaves <2GB headroom; SFT-LoRA needs 4-6GB additional. Trigger requires either 2x3090 upgrade (per memory:project_vram_tier_plan) OR 24GB single-GPU rent.
   When all three are TRUE, the module emits `rl_weight_update_unlocked` SSB row + opens a structured ticket. UNTIL ALL THREE TRUE: DPO/PPO/GRPO/KTO weight updates remain BLOCKED. Per report's "Concrete trigger to escalate to gradient" framing.

9. DPO/PPO/GRPO/KTO module-presence guard. CREATE `.elai_cc/crates/contracts/tests/no_dpo_no_grpo_until_unlock.rs` (~80 LOC). Asserts NO `dpo_*` / `ppo_*` / `grpo_*` / `dapo_*` / `kto_*` modules exist in `crates/orchestrator/src/` or `crates/training/` until the unlock SSB row from Task 8 has been emitted in a recorded session. ELAI specifically DOES NOT ship a DPO module per report: "DPO is the WRONG primitive" (Apple 2025 3-7pp OOD drop; gradient imbalance arXiv:2502.20847; ELAI has no paired data). If someone tries to add one, this contract test BLOCKS. Per the report's anti-pattern list + memory:reference_b40_papers.md "always WebSearch arXiv ID before citing" — both apply.

10. Dashboard panels + Tauri commands. CREATE `crates/dashboard-v2/src/lib/components/RewardFusionPanel.svelte` (~180 LOC): live PRM+ORM fusion weights + current trajectory's fused_reward breakdown + the 4 penalty components from Task 4. CREATE `crates/dashboard-v2/src/lib/components/RlaifJudgePanel.svelte` (~160 LOC): recent Sonnet judge ratings + latency + calibration-vs-human-grade correlation when available. CREATE `crates/dashboard-v2/src/lib/components/GepaTemplateEvolutionPanel.svelte` (~160 LOC): Pareto-frontier visualization of template arms with mean-reward x diversity axes; evolved arms highlighted. CREATE `crates/dashboard-v2/src/lib/components/DeferredRlLadderPanel.svelte` (~140 LOC): visual status of the 3 unlock triggers from Task 8 + estimated time-to-unlock. CREATE 4 Tauri commands in `crates/dashboard-v2/src-tauri/src/commands.rs` (~120 LOC).

11. Decision doc + open questions tracking. CREATE `workflow/reasoning/E-55-decisions.md` (~150 LOC) recording: (i) why DPO is HARD-AVOIDED for ELAI's data shape; (ii) why SFT-on-rollouts is the first escalation rung BEFORE any RL fine-tune; (iii) the 3 unlock conditions for weight-update; (iv) RLAIF calibration cadence (quarterly per JudgeBench); (v) FusionWeights default rationale (outcome 0.6 / PRM 0.4 per Skywork); (vi) Open Questions 1-5 from the report carried as measurable TODOs:
   - Open Q1: Sonnet-judge ranking vs PPM-PRM-7B correlation on ≥3 paired runs; <0.6 = RLAIF adds signal, >0.85 = skip RLAIF
   - Open Q2: composite-reward vs PPM-only Best-of-N delta on held-out 50-task; <3pp = fusion not worth wiring
   - Open Q3: GEPA evolution for 100 rollouts ($20-50 cloud cost); converged arm change AND new arm beats prior arm ≥5pp = ship; <5pp = template space saturated
   - Open Q4: SFT-LoRA on 200 GREEN sessions; ≥5pp local Qwen pass rate lift = ship E-55.1; <5pp = template/prompt-eng is the bottleneck
   - Open Q5: BAI-convergence horizon at which template-arm RL stops returning marginal gains — no published curve; ELAI to instrument

## WIRING_CONTRACT

produces_symbols:
  - orchestrator::rlaif_judge::RlaifJudge::rate_trajectory(trajectory, criteria) -> RlaifRating
  - orchestrator::rlaif_judge::RlaifRating { correctness, step_validity, sycophancy_absence, format_compliance, conciseness, rationale, latency_ms }
  - orchestrator::gepa_template_evolution::GepaEvolver::evolve_round(bottom_quartile: &[Trajectory]) -> Vec<NewTemplate>
  - orchestrator::reward_fusion::FusedReward + FusionWeights + OrmSignal::{BuildGreen, TestsPass, ProbeSemantic, VerifierConsensus}
  - orchestrator::mona_composite_reward::CompositeReward { base, penalty_format, penalty_length, penalty_sycophancy, penalty_repetition }
  - orchestrator::mona_composite_reward::detect_reward_hacking(traj: &Trajectory) -> RewardHackingVerdict
  - orchestrator::rollout_collector::RolloutCollector::collect(trajectory: GreenSessionTrajectory) -> ()
  - orchestrator::rl_weight_update_deferred::UnlockStatus { bai_saturated, orm_coverage, vram_headroom } + check_unlock() -> bool
  - SSB action_types: rlaif_judge_rated, rlaif_calibration_correlation_low, gepa_template_evolved, gepa_template_evicted, reward_fusion_computed, reward_hacking_suspected, sft_training_threshold_reached, bai_arm_observation_recorded, rl_weight_update_unlocked
  - Tauri commands: get_reward_fusion_breakdown, get_rlaif_judge_history, get_gepa_template_pareto, get_deferred_rl_ladder_status
consumes_symbols:
  - D-5 PLANNED BAI Beta-posterior update slot (extension surface)
  - D-2 PLANNED PPM step-score surface (per D-SUMMARY_INDEX line 140)
  - D-3 PLANNED VerificationResult / consensus_rule (per line 34; ORM signal source)
  - D-13 PLANNED Adversary cloud-call (per line 100; RLAIF judge reuses)
  - D-23 PLANNED sycophancy benchmark (matched-pair from E-54 Task 6 → penalty_sycophancy)
  - crates/orchestrator/src/per_role_runtime.rs::PerRoleRuntime (B-37 SHIPPED; cloud Sonnet 4.6 routing; per elai.map line 6155)
  - E-54 BGE-M3 embedding endpoint (for GEPA Pareto-diversity computation)
  - E-53 D-3 Z3SmtCheck verifier (feeds VerifierConsensus ORM variant)
  - blake3 (rollout-trajectory content hash for dedup; NOT blake2/sha2)
  - serde + serde_json (rollout JSONL serialization; existing)
reachable_from:
  - crates/orchestrator/src/d5_bai.rs (BAI update with fused reward — verify path via Grep first)
  - crates/orchestrator/src/auditor.rs (RLAIF judge invoked on sampled trajectories during AUDITING)
  - .elai_cc/training/rollouts/{date}/{session_id}.jsonl (rollout collection storage)
  - .elai_cc/training/rollouts/_index.jsonl (summary)
  - workflow/sessions/{id}/stubs.md (E-55.B7 stub registered per CLAUDE.md stub_tracking rule)
stub_ban:
  - no `.unwrap()` outside `#[cfg(test)]` in any new Rust module (CLAUDE.md HARD RULE)
  - no `unimplemented!()` / `todo!()` in any new module (Task 8 `rl_weight_update_deferred.rs` IS a registered stub per CLAUDE.md stub registry; the unlock-conditions module BODY is implemented, it's the gated dpo/ppo/grpo/kto modules that don't exist by design)
  - NO DPO module — `crates/orchestrator/src/dpo*.rs` / `crates/training/dpo*.rs` PROHIBITED (contract test `no_dpo_no_grpo_until_unlock`); DPO is wrong primitive per Apple 2025 + gradient imbalance arXiv:2502.20847; ELAI has no paired data
  - NO PPO / GRPO / DAPO / KTO weight-update modules until Task 8 emits `rl_weight_update_unlocked` (contract test enforces)
  - SFT-on-rollouts trigger gate is HARD: ≥200 GREEN rollouts before `sft_training_threshold_reached` emits; never below
  - RLAIF judge MUST sample (default 1/10) to bound cloud cost; never invoke on every trajectory
  - RLAIF calibration correlation <0.6 = downgrade to advisory; correlation logged quarterly
  - GEPA Pareto-frontier MUST retain semantic-diversity dimension (NOT just mean reward) — per report's "Pareto-frontier retention keeps complementary strategies alive"
  - GEPA arm-eviction MUST require ≥30 trials of stable underperformance (no premature eviction)
  - composite reward weights MUST be configurable but DEFAULT MUST be {prm: 0.4, orm: 0.6} per Skywork lesson (outcome dominates)
  - MONA myopic-vs-non-myopic check is MANDATORY — no composite-reward computation without it (reward-hacking-suspected emit)
  - Reward-hacking canary MUST flag ≥80% of synthetic high-reward-low-quality trajectories (contract test `reward_hacking_canary`)
  - blake3 only for rollout content-hash; no blake2/sha2 (CLAUDE.md anti-stale-training)
  - cloud Sonnet calls via PerRoleRuntime — NO direct anthropic SDK import in new modules
frontend_pair: 4 Svelte panels (RewardFusion, RlaifJudge, GepaTemplateEvolution, DeferredRlLadder)
observable_effect:
  trigger: synthetic D-5 BAI round with 100 rollouts on a held-out fixture from E-54 corpus; 10% sampled for RLAIF; bottom-quartile fed to GEPA
  expect_ssb: ≥10 `rlaif_judge_rated` rows; ≥1 `gepa_template_evolved` row with new arm in BAI candidate set; 100 `reward_fusion_computed` rows; ≥1 `reward_hacking_suspected` if any trajectory triggers MONA divergence; 100 `bai_arm_observation_recorded` rows after Beta update
  expect_artifact: D-5 BAI posterior reflects fused reward (not just PRM); 1 new template arm added; 0 evicted (insufficient trials); zero `rl_weight_update_unlocked` rows (3 conditions all still FALSE)

## DETERMINISTIC_CHECK

```bash
# 1. RLAIF judge module exists with 5-dim rating
rg "correctness|step_validity|sycophancy_absence|format_compliance|conciseness" .elai_cc/crates/orchestrator/src/rlaif_judge.rs
# Expected: ≥5 hits

# 2. RLAIF sampling cap (default 1/10)
rg "1.*10|0\\.1|reservoir|sample_rate" .elai_cc/crates/orchestrator/src/rlaif_judge.rs
# Expected: ≥1 hit

# 3. GEPA template evolution with Pareto frontier + 30-trial eviction
rg "pareto|Pareto|bottom_quartile|gepa_template_evolved|gepa_template_evicted" .elai_cc/crates/orchestrator/src/gepa_template_evolution.rs
# Expected: ≥3 hits
rg "30|MIN_TRIALS_BEFORE_EVICTION" .elai_cc/crates/orchestrator/src/gepa_template_evolution.rs
# Expected: ≥1 hit

# 4. Reward fusion with OrmSignal variants + weights default
rg "OrmSignal::BuildGreen|OrmSignal::TestsPass|OrmSignal::ProbeSemantic|OrmSignal::VerifierConsensus" .elai_cc/crates/orchestrator/src/reward_fusion.rs
# Expected: ≥4 hits
rg "prm.*0\\.4|orm.*0\\.6|FusionWeights" .elai_cc/crates/orchestrator/src/reward_fusion.rs
# Expected: ≥2 hits

# 5. MONA + composite reward + 4 penalty terms
rg "penalty_format|penalty_length|penalty_sycophancy|penalty_repetition" .elai_cc/crates/orchestrator/src/mona_composite_reward.rs
# Expected: ≥4 hits
rg "reward_hacking_suspected|myopic|step_validity" .elai_cc/crates/orchestrator/src/mona_composite_reward.rs
# Expected: ≥2 hits

# 6. Rollout collector + 200 threshold
rg "200|MIN_ROLLOUTS_FOR_SFT|sft_training_threshold_reached" .elai_cc/crates/orchestrator/src/rollout_collector.rs
# Expected: ≥2 hits

# 7. Deferred RL ladder with 3 conditions
rg "bai_saturated|orm_coverage|vram_headroom|UnlockStatus" .elai_cc/crates/orchestrator/src/rl_weight_update_deferred.rs
# Expected: ≥4 hits

# 8. NO DPO / PPO / GRPO / DAPO / KTO weight-update modules
rg --files .elai_cc/crates/orchestrator/src/ .elai_cc/crates/training/ 2>/dev/null | grep -iE "dpo|^ppo|grpo|dapo|^kto" | grep -v "no_dpo_no_grpo_until_unlock"
# Expected: zero hits (the contract test file is the only file with these strings in its name)

# 9. No anthropic SDK direct import in new modules — must route via PerRoleRuntime
rg "anthropic\\:\\:|anthropic-sdk|claude_api" .elai_cc/crates/orchestrator/src/rlaif_judge.rs
# Expected: zero hits (must use PerRoleRuntime::cloud_sonnet)
rg "PerRoleRuntime|cloud_sonnet" .elai_cc/crates/orchestrator/src/rlaif_judge.rs
# Expected: ≥1 hit

# 10. blake3 only
rg "blake3" .elai_cc/crates/orchestrator/src/rollout_collector.rs
# Expected: ≥1 hit
rg "blake2|sha2" .elai_cc/crates/orchestrator/src/rlaif_judge.rs .elai_cc/crates/orchestrator/src/gepa_template_evolution.rs .elai_cc/crates/orchestrator/src/reward_fusion.rs .elai_cc/crates/orchestrator/src/mona_composite_reward.rs .elai_cc/crates/orchestrator/src/rollout_collector.rs
# Expected: zero hits

# 11. No .unwrap() outside #[cfg(test)]
rg "\\.unwrap\\(\\)" .elai_cc/crates/orchestrator/src/rlaif_judge.rs .elai_cc/crates/orchestrator/src/gepa_template_evolution.rs .elai_cc/crates/orchestrator/src/reward_fusion.rs .elai_cc/crates/orchestrator/src/mona_composite_reward.rs .elai_cc/crates/orchestrator/src/rollout_collector.rs .elai_cc/crates/orchestrator/src/rl_weight_update_deferred.rs | rg -v "cfg\\(test\\)|#\\[test\\]|tests/"
# Expected: zero hits

# 12. Stub registry entry exists for Task 8 deferred surface
ls workflow/sessions/*/stubs.md 2>/dev/null | head -1 | xargs -I{} rg "E-55.B7|rl_weight_update_deferred" {} 2>/dev/null
# Expected: at session-time, ≥1 hit (E-55-decisions.md notes the registration step)

# 13. Dashboard panels exist
ls .elai_cc/crates/dashboard-v2/src/lib/components/RewardFusionPanel.svelte .elai_cc/crates/dashboard-v2/src/lib/components/RlaifJudgePanel.svelte .elai_cc/crates/dashboard-v2/src/lib/components/GepaTemplateEvolutionPanel.svelte .elai_cc/crates/dashboard-v2/src/lib/components/DeferredRlLadderPanel.svelte
# Expected: 4 files exist

# 14. Smoke tests
cd .elai_cc && cargo test -p orchestrator rlaif_judge_invokes_via_per_role_runtime
cargo test -p orchestrator gepa_evolves_bottom_quartile_arms
cargo test -p orchestrator reward_fusion_default_weights_outcome_dominant
cargo test -p orchestrator mona_flags_high_reward_low_step_validity
cargo test -p orchestrator rollout_collector_threshold_at_200
cargo test -p orchestrator deferred_rl_ladder_blocks_until_all_three_conditions
# Expected: 6 PASS
cargo test -p contracts no_dpo_no_grpo_until_unlock reward_hacking_canary
# Expected: 2 PASS
```

## ACCEPTANCE

- [ ] RlaifJudge invokes cloud Sonnet 4.6 via PerRoleRuntime (no direct anthropic SDK import); samples 1/10 of trajectories by default; 5-dim Likert rating
- [ ] Quarterly calibration harness vs human-graded slice; correlation <0.6 = downgrade to advisory; `rlaif_calibration_correlation_low` SSB row
- [ ] GepaEvolver reflects on bottom-quartile trajectories, proposes new template arms, retains on Pareto frontier (mean reward × semantic diversity), evicts only after ≥30 trials of underperformance
- [ ] RewardFusion default weights {prm: 0.4, orm: 0.6}; 4 OrmSignal variants; configurable but never both weights = 0
- [ ] MONA composite reward = base - penalty_{format,length,sycophancy,repetition}; myopic vs non-myopic check fires `reward_hacking_suspected`
- [ ] RolloutCollector writes GREEN sessions to `training/rollouts/{date}/{session_id}.jsonl`; emits `sft_training_threshold_reached` at 200 rollouts
- [ ] D-5 BAI Beta-posterior consumes fused-reward observations; GEPA-evolved arms enter with Beta(1,1)
- [ ] DeferredRL ladder module tracks 3 unlock conditions; emits `rl_weight_update_unlocked` ONLY when all 3 TRUE; stub registered in workflow/sessions/{id}/stubs.md
- [ ] NO DPO module exists anywhere — contract test `no_dpo_no_grpo_until_unlock` enforces; PPO/GRPO/DAPO/KTO also blocked until unlock
- [ ] Reward-hacking nightly canary flags ≥80% of synthetic high-reward-low-quality trajectories
- [ ] 4 dashboard panels + 4 Tauri commands wired
- [ ] blake3 only for content-hash; no blake2/sha2
- [ ] No `.unwrap()` outside `#[cfg(test)]` in new modules
- [ ] `cargo check --workspace` is green; D-2/D-3/D-5/D-13/D-23/D-28 PLANNED-or-DEFERRED surfaces unchanged (additive)
- [ ] memory:reference_b40_papers.md "always WebSearch arXiv ID before citing" — all 19+ arXiv IDs in this E-N verified

## Tests to add

- `crates/orchestrator/tests/rlaif_judge_invokes_via_per_role_runtime.rs`: judge routes through PerRoleRuntime::cloud_sonnet, no direct anthropic dep
- `crates/orchestrator/tests/rlaif_sampling_rate_default_one_tenth.rs`: 100 trajectories → ~10 rated, ~90 skipped
- `crates/orchestrator/tests/rlaif_calibration_low_correlation_downgrades.rs`: simulated correlation 0.4 → advisory mode + SSB row
- `crates/orchestrator/tests/gepa_evolves_bottom_quartile_arms.rs`: 100 trajectories, bottom 25 → reflection → ≥1 new arm
- `crates/orchestrator/tests/gepa_pareto_retains_diversity.rs`: 2 arms with equal mean reward but different semantic diversity → both retained
- `crates/orchestrator/tests/gepa_no_premature_eviction.rs`: arm underperforms for 20 trials → NOT evicted (<30 threshold)
- `crates/orchestrator/tests/reward_fusion_default_weights_outcome_dominant.rs`: default FusionWeights = {0.4, 0.6}
- `crates/orchestrator/tests/reward_fusion_handles_all_orm_variants.rs`: 4 variants → 4 distinct reward values
- `crates/orchestrator/tests/mona_flags_high_reward_low_step_validity.rs`: myopic=0.9, step_validity=0.3 → reward_hacking_suspected
- `crates/orchestrator/tests/composite_reward_four_penalties.rs`: 4 penalty terms each independently subtracted
- `crates/orchestrator/tests/rollout_collector_threshold_at_200.rs`: 199 rollouts → no trigger; 200 → sft_training_threshold_reached
- `crates/orchestrator/tests/rollout_collector_only_green_sessions.rs`: BLOCKed session → NOT collected
- `crates/orchestrator/tests/deferred_rl_ladder_blocks_until_all_three_conditions.rs`: 2/3 conditions TRUE → NO unlock; 3/3 → unlock emitted
- `crates/orchestrator/tests/bai_arm_observation_with_fused_reward.rs`: Beta update uses fused reward, not PPM-only
- `crates/contracts/tests/no_dpo_no_grpo_until_unlock.rs`: assert no module file paths match dpo/ppo/grpo/dapo/kto regex
- `crates/contracts/tests/reward_hacking_canary.rs`: 20 synthetic high-reward-low-quality trajectories → ≥16 flagged
- `crates/contracts/tests/rlaif_no_direct_anthropic_import.rs`: assert rlaif_judge.rs imports only PerRoleRuntime, not anthropic SDK

## Files touched

- CREATE: `crates/orchestrator/src/rlaif_judge.rs` (~280 LOC)
- CREATE: `crates/orchestrator/src/gepa_template_evolution.rs` (~260 LOC)
- CREATE: `crates/orchestrator/src/reward_fusion.rs` (~220 LOC)
- CREATE: `crates/orchestrator/src/mona_composite_reward.rs` (~240 LOC)
- CREATE: `crates/orchestrator/src/rollout_collector.rs` (~200 LOC)
- CREATE: `crates/orchestrator/src/rl_weight_update_deferred.rs` (~120 LOC)
- CREATE: 17 test files listed above (~40-160 LOC each)
- CREATE: `crates/dashboard-v2/src/lib/components/RewardFusionPanel.svelte` (~180 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/RlaifJudgePanel.svelte` (~160 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/GepaTemplateEvolutionPanel.svelte` (~160 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/DeferredRlLadderPanel.svelte` (~140 LOC)
- CREATE: `workflow/reasoning/E-55-decisions.md` (~150 LOC)
- CREATE: `.elai_cc/training/_README.md` (~40 LOC documenting rollout JSONL schema + 200-rollout SFT threshold)
- MODIFY: `crates/orchestrator/src/d5_bai.rs` (~40 LOC delta — Beta-posterior accepts fused-reward observation; verify path via Grep — if D-5 PLANNED-not-shipped, ship a minimal surface stub)
- MODIFY: `crates/orchestrator/src/d2_ppm.rs` (~20 LOC delta — expose step-score Vec<f32> to reward_fusion; verify path via Grep)
- MODIFY: `crates/orchestrator/src/d3_verification.rs` (~20 LOC delta — VerificationResult exposes consensus_outcome to OrmSignal::VerifierConsensus)
- MODIFY: `crates/orchestrator/src/auditor.rs` (~30 LOC delta — RLAIF judge invoked on sampled trajectories during AUDITING)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (~120 LOC delta — 4 new Tauri commands)
- MODIFY: `workflow/sessions/{id}/stubs.md` (~10 LOC delta — register E-55.B7 stub per CLAUDE.md stub registry)

## Citations / Grounding

- https://arxiv.org/abs/2305.18290 — Rafailov DPO original; CITED to JUSTIFY rejection (Apple 2025 OOD drop + gradient imbalance documented separately)
- https://machinelearning.apple.com/research/reward-generalization — Apple 2025; DPO RM matches RLHF in-distribution, drops 3% mean / 7% max on 5 OOD settings; grounds DPO HARD AVOID
- https://arxiv.org/pdf/2502.20847 — Gradient Imbalance in DPO Feb 2025; grounds DPO HARD AVOID
- https://arxiv.org/pdf/2402.01306 — KTO prospect-theory loss on binary signals; grounds future-tier preference (if/when unlock)
- https://arxiv.org/abs/2502.14187 — Federated KTO vs DPO Feb 2025; KTO beats DPO on MT-Bench-1/Vicuna/AdvBench; grounds future-tier rank
- https://arxiv.org/abs/2412.21139 — SWE-Gym ICML 2025; 491 trajectories +12.3pp Lite / +13.6pp Verified; grounds Task 5 SFT threshold + Open Q4
- https://arxiv.org/abs/2410.01679 — VinePPO NeurIPS 2024 / ICML 2025; MC value estimates; grounds future-tier RL (if/when unlock)
- https://arxiv.org/pdf/2501.13011 — MONA myopic-with-non-myopic-approval; grounds Task 4 reward-hacking guard
- https://arxiv.org/pdf/2509.15557 — Verifiable Composite Rewards medical reasoning; grounds Task 4 4-penalty structure
- https://arxiv.org/pdf/2501.07301 — Qwen2.5-Math-PRM lessons (MC + LLM-judge consensus filter); grounds D-2 PPM context
- https://qwenlm.github.io/blog/qwen2.5-math-prm/ — PRM blog BoN methodology; grounds Task 3 BoN framing
- https://arxiv.org/pdf/2507.01352 — Skywork-Reward-V2 8B 290K curated > 70B raw; grounds FusionWeights {0.4, 0.6} outcome-dominant default
- https://arxiv.org/abs/2212.08073 — Anthropic CAI / RLAIF original; grounds Task 1 RLAIF pattern
- https://arxiv.org/pdf/2309.00267 — RLAIF vs RLHF Lee et al.; comparable/superior to RLHF on summ + dialogue at fraction of label cost; grounds Task 1
- https://arxiv.org/pdf/2507.19457 — GEPA ICLR 2026 oral; 35x fewer rollouts than PPO/GRPO; grounds Task 2 prompt-evolution pattern
- https://arxiv.org/html/2508.16546v1 — RL vs SFT panacea-or-mirage Aug 2025; SFT 2K can match RL 20K on weak models; grounds SFT-on-rollouts as first rung
- https://arxiv.org/pdf/2501.17161 — SFT Memorizes, RL Generalizes Jan 2025; grounds future-tier RL conditions
- https://github.com/SWE-Gym/SWE-Gym — SWE-Gym repo; 491 traces from GPT-4o+Sonnet → 32B Qwen-Coder, +14% absolute; grounds Task 5 schema
- https://arxiv.org/html/2503.06639 — GRPO loss dynamics analysis Mar 2025; grounds future-tier
- memory:project_b37_shipped.md — PerRoleRuntime cloud Sonnet 4.6 surface; grounds Task 1 cloud routing
- memory:project_qwen3b_baseline.md — 12GB VRAM ceiling; grounds Task 8 vram_headroom condition
- memory:project_vram_tier_plan.md — 2x3090 upgrade coming; grounds Task 8 unlock prerequisite
- memory:reference_b40_papers.md — "always WebSearch arXiv IDs before citing"; grounds Citations integrity rule
- memory:feedback_audit_cycles.md — "honor explicit caps"; grounds Task 1 RLAIF sampling rate
- CLAUDE.md anti-stale-training — blake3 only; no .unwrap() in lib code; grounds stub_ban
- CLAUDE.md "No new dependencies without confirmation" — grounds future-tier RL fine-tune as user-greenlight required

## Open questions / risks

- Open Q1 (from report, measurable): Sonnet-judge ranking vs PPM-PRM-7B correlation on ≥3 paired ELAI runs. Threshold: correlation <0.6 = RLAIF adds signal (proceed); >0.85 = PPM alone suffices (skip RLAIF). Logged to E-55-decisions.md after first calibration pass.
- Open Q2 (from report, measurable): composite-reward (PPM step-score + verifiable ORM) lift over PPM-only on held-out 50-task set. Threshold: <3pp = fusion not worth wiring. Measured by Task 6 reward-hacking canary + Task 3 ablation harness on E-54 held-out.
- Open Q3 (from report, measurable): GEPA evolution for 100 rollouts (cloud cost ~$20-50). Threshold: BAI-converged arm changes AND new arm beats prior arm ≥5pp on held-out task set = ship; <5pp = template space saturated, spend budget elsewhere. Pilot in E-55-decisions.md.
- Open Q4 (from report, measurable): SFT-LoRA on 200 GREEN session trajectories. Threshold: ≥5pp local Qwen pass-rate lift on held-out 50-task ELAI bench = ship E-55.1; <5pp = template/prompt-eng is bottleneck, not training. Triggered automatically once Task 5 reaches threshold.
- Open Q5 (from report): at what BAI-convergence horizon does template-arm RL stop returning marginal gains and weight-level KTO+RLVR become preferred? No published curve. Mitigation: Task 7 instruments arm-stability metrics + arm-saturation-detection per session; logs trend to `workflow/reasoning/E-55-bai-convergence-trace.jsonl`.
- Risk 1: RLAIF cloud Sonnet cost. Mitigation: 1/10 sampling cap (Task 1) + per-session quota (configurable; default 100 RLAIF calls/session); over-quota → emit `rlaif_quota_exceeded` + skip remaining. Cost-trace logged.
- Risk 2: GEPA-generated templates may drift toward gaming the PRM (reward hacking via prompt). Mitigation: Task 4 MONA reward-hacking guard fires on the GEPA-evolved arm before its Beta posterior updates; flagged arms enter quarantine for human review before BAI candidate set.
- Risk 3: Reward-fusion weight tuning may need per-family-task customization (math-proof vs code-fix). Mitigation: FusionWeights is `HashMap<TaskFamily, FusionWeights>` (per-family override); default applies to all unless override; per-family tuning is a follow-up patch E-55.2 after Open Q2 measurement.
- Risk 4: SFT-on-rollouts (Task 5 → E-55.1) requires LoRA training infrastructure (PEFT, accelerate, HF transformers) which is NOT currently in `python/elai_inference/pyproject.toml`. Mitigation: E-55 ships ONLY the collection surface + threshold trigger; E-55.1 (separate patch with user confirmation per CLAUDE.md "No new dependencies without confirmation") adds training pipeline. Per memory:project_vram_tier_plan, 2x3090 upgrade is also a prerequisite for the actual training run.
- Risk 5: Reward-hacking canary (Task 6) needs 20 synthetic-but-human-verified gaming trajectories. Mitigation: ship the canary with 5 seed trajectories first; expand to 20 in a separate iteration as the team curates examples; canary still runs nightly even with 5 seeds (proportional 80% threshold = 4/5 flag).
- Risk 6: cloud Sonnet 4.6 may change behavior across versions (Anthropic ships new versions). Mitigation: pin model version in RLAIF judge config (e.g. `claude-sonnet-4-7-1m` exact ID per CLAUDE.md "model ID is claude-opus-4-7[1m]"); calibration harness re-runs on every pinned-version-change.
- Risk 7: Stub registry requires `workflow/sessions/{id}/stubs.md` to exist; if session hasn't been started yet, the registration emits a startup-blocked error. Mitigation: Task 8 registers the stub LAZILY at first invocation of `rl_weight_update_deferred::check_unlock()`, not at module load.
- Risk 8: Citation integrity per memory:reference_b40_papers.md — every arXiv ID in this E-N MUST be WebSearch-verified. Mitigation: the prompt-engineer / paper-ingestor agent runs a one-time pass over E-55 Citations block before EXECUTING begins; any unverifiable ID is flagged and substituted per memory:feedback_paper_citation_verify.md protocol.

## Next

E-55 ships the right RL rungs for ELAI's actual state: RLAIF judge (highest leverage, zero weight update), GEPA reflective prompt evolution (35x cheaper than PPO/GRPO), hybrid PRM+ORM fusion (+5-12pp BoN), and MONA composite-reward guards against reward hacking. SFT-on-rollouts collection surface is wired; actual SFT-LoRA fine-tune is E-55.1 (deferred, gated on 200 GREEN rollouts + 2x3090 + user confirmation). DPO is HARD-AVOIDED forever for ELAI's data shape; PPO/GRPO/DAPO/KTO weight updates are HARD-DEFERRED until 3 unlock conditions all TRUE (BAI saturation, ORM coverage ≥80%, VRAM headroom). The deferred-RL-ladder is observable in the dashboard so the user sees concrete unlock progress, not vague "RL coming soon". Downstream: E-53's Z3SmtCheck feeds OrmSignal::VerifierConsensus; E-54's matched-pair sycophancy feeds penalty_sycophancy; E-52's diagnostic probe could feed an additional Probe variant in a follow-up patch. Open follow-ups: (i) Open Q1-Q5 measurements logged after first sessions; (ii) E-55.1 SFT-LoRA fine-tune once Task 5 hits 200 rollouts AND 2x3090 lands AND user opts in; (iii) E-55.2 per-family FusionWeights tuning after Open Q2 measurement; (iv) D-28 Adversarial PPM farming UN-deferral once BAI saturates (per D-plan condition); (v) future RL fine-tune patch ladder: KTO+RLVR → maybe DAPO → never DPO. The point of E-55 is to ship the EVIDENCE-GROUNDED rungs while making the DEFERRED ones first-class observable artifacts — not silent TODOs.
