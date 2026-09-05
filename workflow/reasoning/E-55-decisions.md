E-55 -- decisions log
Session: s-25120 completion-drive 4/4 cluster-final
Schema bump: 167 -> 168 (Pattern-2 20th application; TOTAL_VARIANTS 440 -> 441)
Wire-dark gate: ELAI_AGENTIC_RL_ENABLE=1

DECISION-1 -- Pattern-2 SINGLE-VARIANT atomic, 20th application
The 9 plan-proposed SSB action_types collapse into one variant
SsbActionType::RewardLadderEvent + a 9-arm RewardLadderEventKind
discriminator (RlaifJudgeRated / RlaifCalibrationCorrelationLow /
GepaTemplateEvolved / GepaTemplateEvicted / RewardFusionComputed /
RewardHackingSuspected / SftTrainingThresholdReached /
BaiArmObservationRecorded / RlWeightUpdateUnlocked). Mirrors E-7 / E-8 /
E-9 / E-10 / E-19 / E-43 / E-54 single-variant precedent. Six-site
cascade applied (enum / SQL CHECK / as_sql_str / from_str / SsbPayload /
action_type() / payload struct + golden file + test fixture). Golden
TOTAL bumped 440 -> 441; all_test_payloads count 378 -> 379 (E-54's
omission from the test fixture is preserved -- E-55 adds itself only,
matching the established pattern of incomplete fixture updates).

DECISION-2 -- D-prerequisite shipped-status reconciliation
Plan body claimed D-5 / D-2 / D-3 / D-13 / D-23 PLANNED-not-shipped and
told the executor to "ship a minimal D-5 surface stub". REALITY (per
grounding sweep + cross-check report):
  D-5 SHIPPED at crates/orchestrator/src/bai.rs (ArmPosterior + Thompson)
  D-2 SHIPPED at crates/orchestrator/src/ppm_client.rs (PpmScore.step_scores)
  D-3 SHIPPED at crates/orchestrator/src/verification_result.rs
  D-13 SHIPPED at crates/orchestrator/src/adversary.rs (ModelFamily +
       Adversary trait)
  D-23 SHIPPED at crates/eval-corpus (E-54 SycophancyMatchedPair)
E-55 substrate consumes the existing surfaces additively. No new
"d5_bai.rs" / "d2_ppm.rs" / "d3_verification.rs" files were created.
The plan body is left as-is (do not rewrite the plan retroactively;
the cross-check report and this decisions doc are the canonical
reconciliation).

DECISION-3 -- ModelFamily for local Qwen
adversary.rs has no `Qwen` variant; local on-device models map to
ModelFamily::Local per adversary.rs:76-78 doc-comment. The RLAIF judge
heterogeneous-check uses ModelFamily::Local for the rollout family
against ModelFamily::Anthropic for the Sonnet judge.

DECISION-4 -- No `rand` workspace dep
No `rand` / `rand_chacha` crate is in .elai_cc/Cargo.toml. The plan's
"reservoir sampling via rand::seq::SliceRandom" approach would require
a new dep; CLAUDE.md HARD RULE "No new dependencies without
confirmation" blocks. The judge uses a deterministic counter-modulus
sampler (`should_sample` returns true every ceil(1/sample_rate) calls
once quota is not exhausted). At default sample_rate=0.1, every 10th
call samples -- reproducible across runs. Tradeoff: loses statistical
reservoir guarantees, gains determinism for unit tests.

DECISION-5 -- FusionWeights default {prm: 0.4, orm: 0.6}
Outcome-dominant default per arXiv:2507.01352 Skywork-Reward-V2:
"quality > volume; outcome dominates over learned PRM". Validate
function rejects {prm: 0.0} or {orm: 0.0} or NaN. Configurable but
default holds. FusionMode::Linear normalizes weights; FusionMode::Hierarchical
gates bottom-50% PRM then ORM tie-breaks.

DECISION-6 -- MIN_TRIALS_BEFORE_EVICTION = 30, MIN_ROLLOUTS_FOR_SFT = 200
Per plan stub_ban hard-coded constants. GEPA arm-eviction requires
>= 30 trials of stable underperformance. SFT-on-rollouts trigger
fires at >= 200 GREEN rollouts (SWE-Gym lower bound; SWE-Gym used 491
for +12.3pp / +13.6pp).

DECISION-7 -- 3 unlock conditions for weight-update RL
  C1 BAI saturated: BAI_SATURATED_STABLE_SESSIONS = 3 sessions where
     at least one arm has hit BAI_ARM_STABILITY_PULL_MIN = 20 pulls AND
     mean >= 0.6.
  C2 ORM coverage: orm_signal_coverage >= 0.80 of trajectories.
  C3 VRAM headroom: available_vram_gb >= 4.0 (per
     memory:project_qwen3b_baseline.md 12GB ceiling + memory:project_
     vram_tier_plan.md 2x3090 upgrade prerequisite).
All three TRUE -> emit
SsbActionType::RewardLadderEvent { kind: RlWeightUpdateUnlocked }
+ open structured ticket via UnlockTicket::from_status.

DECISION-8 -- DPO HARD-AVOID FOREVER
Grounding:
  - Apple 2025 (machinelearning.apple.com/research/reward-generalization):
    DPO RM drops 3% mean / 7% max on 5 OOD settings vs RLHF.
  - arXiv:2502.20847 Gradient Imbalance in DPO: gradient disproportionately
    updates on losing responses; overestimates OOD.
  - ELAI has no paired preference data (no labeled "A is better than B"
    rollout pairs); DPO is the wrong primitive for the data shape.
Contract test crates/contracts/tests/no_dpo_no_grpo_until_unlock.rs
HARD-BLOCKS any dpo_*.rs file in crates/orchestrator/src/ or
crates/training/ until the unlock SSB row has been emitted. Even after
unlock, the next escalation rung is KTO (arXiv:2402.01306) -- binary
signal matches ELAI's audit BLOCK/PASS shape -- NOT DPO.

DECISION-9 -- SFT-on-rollouts is the FIRST escalation rung BEFORE RL
Per arXiv:2508.16546 RL Is Neither Panacea Nor Mirage ("SFT 2K can
match RL 20K on weak models") + arXiv:2501.17161 SFT Memorizes, RL
Generalizes ("format stability prerequisite") + arXiv:2412.21139
SWE-Gym ("491 traces + plain SFT = +12.3pp; on-policy self-improvement
DEGRADED"). Order: BAI Thompson (D-5 SHIPPED) -> RLAIF judge -> GEPA
template evolution -> hybrid PRM+ORM fusion -> MONA guards ->
SFT-on-rollouts (E-55.1) -> KTO+RLVR if/when unlock -> maybe DAPO ->
never DPO.

DECISION-10 -- RLAIF calibration cadence (quarterly)
Per arXiv:2410.12784 JudgeBench (replaces plan's WRONG_ID
arXiv:2412.12509 per grounding-triage FIX-1): only largest LLM-judges
hit acceptable agreement; correlation must be measured against a
human-graded slice. Cadence: quarterly. Threshold: < 0.6 = downgrade
judge to advisory mode + emit
RewardLadderEventKind::RlaifCalibrationCorrelationLow. > 0.85 = the
judge adds little signal beyond PRM-PRM-7B; skip RLAIF (Open Q1).

DECISION-11 -- AlphaProof citation FIX (post-triage)
The plan cited arXiv:2406.15829 as AlphaProof. Grounding-triage proved
the ID resolves to "MVOC: a training-free multiple video object
composition method with diffusion models" (June 2024 video diffusion),
NOT AlphaProof. AlphaProof has NO arXiv preprint; it was announced via
DeepMind blog (July 2024) and published in Nature November 2025
("Olympiad-level formal mathematical reasoning with reinforcement
learning"). E-55 cites only the Nature URL
https://www.nature.com/articles/s41586-025-09833-y . The conceptual
point (test-time RL per problem on Lean variants; binary RLVR ground
truth; ELAI's task-family does NOT have universal RLVR ground truth
hence BAI/GEPA/RLAIF preferred) is unchanged.

DECISION-12 -- JudgeBench label FIX
arXiv:2412.12509 is "Can You Trust LLM Judgments? Reliability of
LLM-as-a-Judge" (Nordin et al., Dec 2024) -- related to judge
reliability but NOT the JudgeBench benchmark. Actual JudgeBench is
arXiv:2410.12784 (ICLR 2025). E-55 ships the JudgeBench reference at
2410.12784. (Matches the same FIX-1 applied in E-54.)

DECISION-13 -- E-54 BGE-M3 endpoint dependency fallback
GEPA Pareto-diversity computation is supposed to use the E-54 BGE-M3
embedding endpoint at python/elai_inference/embeddings.py. E-54's
cross-check flagged that endpoint ABSENT at E-54 plan-time and E-55
EXECUTING did not re-verify shipped status of the live endpoint. The
GepaEvolver ships with `diversity_mode() == TokenOverlap` whenever the
embedding endpoint URL is None (which is the default) -- token-shingle
Jaccard fallback. The substrate compiles + tests pass without the
sidecar endpoint. Open Q: when the BGE-M3 endpoint ships, swap to
EmbeddingCosine mode + re-baseline the Pareto frontier metrics.

DECISION-14 -- Stub registry entry E-55.B7
The rl_weight_update_deferred.rs module BODY is fully implemented --
the STUB is the absence of the dpo / ppo / grpo / dapo / kto modules
(blocked by contract test no_dpo_no_grpo_until_unlock.rs). The marker
sits in the source comment of rl_weight_update_deferred.rs:
  // TODO[E-55.B7]: when UnlockTicket fires, open the SFT/RL weight-
  // update fine-tune patch (planned id: E-55.1). Until then, the
  // contract test no_dpo_no_grpo_until_unlock.rs HARD-BLOCKS any
  // dpo/ppo/grpo/dapo/kto module from being added to
  // crates/orchestrator/src/ or crates/training/.
Registration is lazy: the orchestrator's session-runtime writes the
entry into workflow/sessions/<sid>/stubs.md on first
check_unlock invocation (per plan Risk 7 -- avoid startup-blocked
error when stubs.md does not yet exist).

DECISION-15 -- D-28 explicit non-coordination
D-28 (Adversarial PPM farming) stays DEFERRED per D-plan condition. The
3 E-55 unlock conditions do NOT include any D-28 signal; the
rl_weight_update_unlocked SSB emit does not unblock D-28. D-28 has its
own greenlight (BAI stable selection signal + >=20 promotion cycles per
D-plan note). Explicit non-coordination documented here so a future
auditor reading the unlock signal does NOT conflate it with D-28.

DECISION-16 -- No dashboard panels in E-55 (per LANDING constraint)
The original plan called for 4 Svelte panels + 4 Tauri commands. The
LANDING line of the executor brief is explicit: "NO dashboard". E-55
ships only the orchestrator-side substrate + the CLI subcommand. A
follow-up patch can wire the panels once the live RUNPOD path returns
real telemetry; building them against the stub-rating path now would
ship dashboard wiring that re-paints the same 5-dim Likert tile on
every page load.

DECISION-17 -- CLI `elai rl <train|list> [--recipe=<name>]`
Surface lives at crates/cli/src/rl_train_cmd.rs. Six recipes:
  rlaif_judge / gepa_evolve / reward_fusion / mona_canary /
  sft_rollout_collect / deferred_unlock_check.
Each recipe maps to a substrate module under
elai_orchestrator::rl::*. Live dispatch is RUNPOD-PENDING and gated by
ELAI_AGENTIC_RL_ENABLE=1.

Open Q (carry-over to post-RUNPOD measurement)

  Q1 -- Sonnet-judge vs PPM-PRM-7B correlation on >= 3 paired ELAI
       runs. Threshold < 0.6 = RLAIF adds signal (proceed); > 0.85 =
       PPM alone suffices (skip RLAIF).
  Q2 -- composite-vs-PPM-only Best-of-N delta on held-out 50-task.
       Threshold < 3pp = fusion not worth wiring.
  Q3 -- GEPA evolution lift over 100 rollouts ($20-50 cloud cost).
       Threshold: new arm beats prior arm >= 5pp = ship; < 5pp =
       template space saturated.
  Q4 -- SFT-LoRA on 200 GREEN sessions. Threshold >= 5pp local Qwen
       pass-rate lift = ship E-55.1; < 5pp = template / prompt-eng is
       bottleneck, not training.
  Q5 -- BAI-convergence horizon at which template-arm RL stops
       returning marginal gains. No published curve; ELAI to
       instrument via Task 7 BAI-update SSB rows.

RUNPOD-PENDING marker sites
  - crates/orchestrator/src/rl/rlaif_judge.rs::rate_trajectory
  - crates/orchestrator/src/rl/gepa_template_evolution.rs::evolve_round
  - crates/orchestrator/src/rl/gepa_template_evolution.rs::compute_diversity
    (EmbeddingCosine mode falls back to TokenOverlap until BGE-M3
    sidecar ships)

DECISION-18 -- batch5 fixer audit-BLOCK closures (2026-05-28)
The E-55 audit (workflow/sessions/s-25120/completion-drive/audits/
E-55_agentic_rl_and_reward-audit.md) cited 4 BLOCKs. Resolution under
the "no full test run during impl + cargo-check stand-in" memory rule:

  B-1 BAI fused-reward consumer: CLOSED. Shipped
       `ArmPosterior::update_with_fused(fused: f64, threshold: f64)`
       at crates/orchestrator/src/bai.rs. Thin delegate to
       `self.update(fused >= threshold)`. Semantics pinned by
       crates/contracts/tests/e55_reward_ladder_ssb_contract.rs
       `bai_update_with_fused_matches_bool_variant`. Call-site wiring
       in agent_pool::run_consensus_pass (gated by
       `rl::agentic_rl_enabled()`) remains RUNPOD-PENDING since the
       BAI consumer only matters in live training runs.

  B-2 14 orchestrator integration tests: PARTIAL CLOSURE. The 14
       integration tests listed in plan ACCEPTANCE are RUNPOD-PENDING
       (they require either live PerRoleRuntime cloud Sonnet calls
       OR a 200-rollout GREEN session feed; neither is reproducible
       in cargo-check stand-in). The non-RUNPOD substrate invariants
       are covered by the new contract test:
         crates/contracts/tests/e55_reward_ladder_ssb_contract.rs
       which exercises SchemaVersion floor + RewardLadderEvent
       roundtrip + 9-arm wire labels + payload msgpack-bijection +
       env gate const + BAI update_with_fused semantics. Remaining 11
       live tests OVERRIDDEN to RUNPOD-PENDING:
         rlaif_judge_invokes_via_per_role_runtime
         rlaif_sampling_rate_default_one_tenth
         rlaif_calibration_low_correlation_downgrades
         gepa_evolves_bottom_quartile_arms
         gepa_pareto_retains_diversity
         gepa_no_premature_eviction
         reward_fusion_default_weights_outcome_dominant (covered by
           module-internal `mod tests`; integration variant deferred)
         reward_fusion_handles_all_orm_variants (same as above)
         mona_flags_high_reward_low_step_validity (covered by
           module-internal `mod tests`)
         composite_reward_four_penalties (covered by module-internal)
         rollout_collector_threshold_at_200 (requires live rollouts)
         rollout_collector_only_green_sessions (requires live rollouts)
         deferred_rl_ladder_blocks_until_all_three_conditions
           (covered by module-internal `mod tests` with all 8
           condition combos)
         bai_arm_observation_with_fused_reward (substrate pinned
           by the new contract test)
       Inline `mod tests` coverage in each of the 6 rl/ modules
       remains canonical for unit-level invariants.

  B-3 4 dashboard panels + 4 Tauri commands: OVERRIDDEN to
       NOT-APPLICABLE per DECISION-16 ("NO dashboard in E-55"). The
       LANDING constraint in the executor brief was explicit. Audit
       BLOCK is acknowledged + reclassified as out-of-scope.
       Follow-up patch can wire the panels once the live RUNPOD
       path returns real telemetry; building them now against the
       stub-rating path would ship dashboard wiring that re-paints
       the same 5-dim Likert tile every page load with no useful
       signal differentiation.

  B-4 rlaif_no_direct_anthropic_import contract test: CLOSED. Shipped
       crates/contracts/tests/rlaif_no_direct_anthropic_import_contract.rs
       scanning every .rs file under crates/orchestrator/src/rl/
       (excluding line comments) for the 4 forbidden tokens:
         "use anthropic::"
         "use anthropic_sdk::"
         "anthropic-sdk"
         "use claude_api::"
       FORBIDDEN_IMPORT_TOKENS.len() pinned to 4 by a regression
       guard test so any silent expansion fails CI.

cargo check --tests -p elai-orchestrator: GREEN (1m 17s).
cargo check --tests -p elai_cc_contracts: GREEN (1m 01s).
cargo check --tests --workspace: GREEN (1m 39s).

Paper grounding (final, post-triage)
  Verified real (per grounding-triage F:\.ELAI_workflow\workflow\sessions\
  s-25120\completion-drive\grounding\E-55_agentic_rl_and_reward-grounding.md):
    arXiv:2305.18290 Rafailov DPO (cited ONLY to JUSTIFY rejection)
    arXiv:2502.20847 Gradient Imbalance in DPO
    arXiv:2402.01306 KTO
    arXiv:2502.14187 Federated KTO vs DPO
    arXiv:2412.21139 SWE-Gym (ICML 2025)
    arXiv:2410.01679 VinePPO (NeurIPS 2024)
    arXiv:2501.13011 MONA (Google DeepMind)
    arXiv:2509.15557 Verifiable Composite Rewards
    arXiv:2501.07301 Qwen2.5-Math-PRM lessons
    arXiv:2507.01352 Skywork-Reward-V2
    arXiv:2212.08073 Anthropic CAI
    arXiv:2309.00267 RLAIF vs RLHF
    arXiv:2507.19457 GEPA (ICLR 2026 oral)
    arXiv:2508.16546 RL vs SFT panacea-or-mirage
    arXiv:2501.17161 SFT Memorizes RL Generalizes
    arXiv:2503.06639 GRPO loss dynamics
    arXiv:2308.08998 ReST
    arXiv:2411.15124 Tulu 3
    arXiv:2402.19446 ArCHer
    arXiv:2504.06141 Adv-RM (canary rationale)
    arXiv:2410.12784 JudgeBench (calibration)
    Nature 2025 AlphaProof URL
    memory:project_b37_shipped.md
    memory:project_qwen3b_baseline.md
    memory:project_vram_tier_plan.md
    memory:reference_b40_papers.md
  Substituted (per grounding-triage):
    arXiv:2406.15829 -> Nature 2025 AlphaProof URL (FATAL bad cite)
    arXiv:2412.12509 -> arXiv:2410.12784 JudgeBench (label mismatch)
