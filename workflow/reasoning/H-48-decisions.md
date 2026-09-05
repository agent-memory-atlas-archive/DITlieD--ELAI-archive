H-48 decisions — Wire BAI bandit on the real reward
Session: s-10612 (executor run 2026-06-11)

WHY H-48 IS LAST IN TRACK 7

A regret-bounded selector over a placeholder reward is theatre (VD-1). H-48
is meaningless before H-43 (real execution fitness) + H-45 (worst-case
aggregate). Both are now SHIPPED (confirmed at execute time). H-45's
aggregate_promotion_score exists at crates/orchestrator/src/promotion_score.rs
and is already wired into darwin_admit_loop.rs::score_candidate_with_aggregate.

WHAT IS COMPOSED (GD-1), NOT RE-AUTHORED

- bai::sample_beta — REUSED as the Thompson draw inside select_next_arm.
  No new Beta sampler written. darwinian_bai imports crate::bai::sample_beta.
- DiscountedArmPosterior::discounted_update — SHIPPED E-7 scaffold, now
  CALLED from update_arms_from_reward (the activation gap H-48 closes).
- SuccessiveRejects::log_bar — SHIPPED E-7 scaffold, called from run_sr_phase.
  The run_sr_phase fixed-budget best-arm fn is new (~40 LOC).

DISCOUNTED-TS CHOICE (arXiv:2305.10718)

The darwin archive's reward distribution shifts as the population evolves —
the exact non-stationary regime Discounted-TS targets. Vanilla Thompson
retains stale counts; the exp(-1/lambda) discount decays them. The production
DEFAULT_DISCOUNT_FACTOR = 0.0357 (exp(-1/0.3)) is the shipped E-7 constant
(a design hypothesis from the MSI lambda=0.3 timescale, not paper-quoted).

DISCOUNT FACTOR ON THE FROZEN FIXTURE

The production constant 0.0357 causes over-forgetting on the compact fixture
(60 alternating rounds). arXiv:2305.10718 §5 experiments use a less aggressive
gamma (~0.9) on short horizons. The fixture uses FIXTURE_GAMMA = 0.90. This is
documented in fixtures/bandit_real_reward/README and in the contract test. The
production default is unchanged (config-knob, per E-7-decisions.md DECISION-13).

SUCCESSIVE REJECTS IMPLEMENTATION

The run_sr_phase function implements Audibert COLT 2010's fixed-budget
best-arm identifier. For H-48's Tier-1 done-gate, it operates on the frozen
aggregate mean rewards (not per-round observation streams). This is the
"Tier-1 in-process" variant — the live multi-generation regret measurement
with observation streams is RUNPOD-PENDING Tier-2 (unchanged from plan).

ExIT CURRICULUM SEEDING (arXiv:2509.04575)

ExIt (Expert Iteration) generates an autocurriculum from partial intermediate
episode states. H-48 adapts this: seed_arms_from_partial_history applies
discounted updates from past partial scores before the main loop. The production
injection channel is ELAI_DARWINIAN_PARTIAL_HISTORY (comma-separated
"idx:reward" pairs). An absent/empty env var is a no-op — the arms start
from the Beta(1,1) uniform prior. The ExIt seeding is complete, not a stub;
the caller decides whether to supply history.

SSB DECISION: REUSE BaiArmUpdated (GD-1, no new variant)

BaiArmUpdated payload fields:
  template_id: String  — darwinian arm label ("darwinian-arm-N")
  alpha: f64           — discounted alpha count
  beta: f64            — discounted beta count
  n_pulls: u32         — cumulative pulls
  mean_estimate: f64   — posterior mean
  was_promoted: bool   — was this candidate admitted this generation

The template_id field carries a synthetic arm label rather than a Blake3
template hash; this is intentional and the mapping is documented in the
emit call (darwin_admit_loop.rs). The was_promoted field maps cleanly to
"was this arm's candidate admitted this generation".

A new BanditArmSampled variant was evaluated and REJECTED because:
  (a) the existing fields are sufficient for the darwinian arm telemetry;
  (b) minimising SSB bumps (GD-1) is the preferred posture when fields fit;
  (c) the darwinian context differs only in the template_id semantics,
      which is documented in the emit call.

SCHEMA stays 225. TOTAL_VARIANTS stays 536.

PRODUCTION WIRING (W-3, R6, R7)

select_next_arm and update_arms_from_reward are called from run_one_generation
(the PRODUCTION entrypoint), not from test-only code. Both functions have
non-test callers in darwin_admit_loop.rs::run_one_generation. The bandit
selection drives GenerationSummary::bandit_selected_arm, which surfaces on the
CLI output (darwin_loop_cmd.rs).

VD-1 COMPLIANCE (no bytes-stub reward)

The reward fed to update_arms_from_reward is promotion_score.worst_case —
the H-45 execution-grounded aggregate (W_PASS*pass_rate + W_AGREE*dual_agreement
from H-43, then min-kappa*var from H-45). A grep confirms no blake3(config)
or derived_alpha call feeds the bandit arm update. The R2 contract test
(broken_arm_bytes_reward_converges_to_arm_b) proves the real reward is
load-bearing: flipping to a high bytes-stub reward reverses convergence.

RESIDUAL RISK / HONEST OPEN ITEMS

- Regret bounds (arXiv:2305.10718) assume the reward distribution is
  non-stationary but not adversarial. A self-mod loop that shapes its own
  fitness signal could exploit the discount window. E-47's eval firewall
  (date-gate + template-regen) is the primary defense; H-48 provides the
  regret-bounded selection on top of H-47's cleaned signal.
- Live multi-generation regret measurement (RUNPOD-PENDING Tier-2): the
  bandit's regret on a real multi-generation run is not measured on the
  frozen fixture. The fixture proves convergence exists; the live number
  is a separately-gated checkbox per W-10.
- DEFAULT_DISCOUNT_FACTOR (0.0357) is a design hypothesis, not
  paper-calibrated. A calibrated value requires a live multi-generation run
  (Tier-2). The config-knob posture (E-7-decisions.md DECISION-13) lets this
  be tuned without a code deploy.

FILES TOUCHED

- MODIFY crates/orchestrator/src/darwinian_bai.rs (activate update_arms_from_reward +
  select_next_arm + run_sr_phase + seed_arms_from_partial_history)
- MODIFY crates/orchestrator/src/darwin_admit_loop.rs (wire bandit into
  run_one_generation; add bandit_selected_arm to GenerationSummary)
- MODIFY crates/cli/src/darwin_loop_cmd.rs (print bandit_selected_arm in summary)
- CREATE crates/contracts/tests/h48_bandit_real_reward_contract.rs (12 tests)
- CREATE crates/contracts/tests/fixtures/bandit_real_reward/README
- CREATE workflow/reasoning/H-48-decisions.md (this file)
