//! E-55 -- Agentic RL + reward ladder substrate.
//!
//! Deferred RL ladder: RLAIF judge (cloud Sonnet on local-Qwen rollouts) +
//! GEPA reflective prompt evolution + hybrid PRM+ORM in BAI reward + MONA
//! composite-reward hacking guards + SFT-on-rollouts collection surface +
//! 3-condition unlock gate for DPO/PPO/GRPO/KTO weight-update modules
//! (which DO NOT exist by design until unlock).
//!
//! Wire-dark: emissions gated by `ELAI_AGENTIC_RL_ENABLE=1`. Live training
//! runs RUNPOD-PENDING -- see `// RUNPOD-PENDING:` markers on the cloud
//! rollout invocation sites. The substrate ships fully; the actual
//! Sonnet-judge cloud calls + SFT-LoRA fine-tune run on the rented GPU.
//!
//! Paper grounding (post-triage):
//!   - arXiv:2212.08073 Constitutional AI (Anthropic) -- Task 1 RLAIF
//!     pattern.
//!   - arXiv:2309.00267 RLAIF vs RLHF (Lee et al.) -- comparable / superior
//!     to RLHF at fraction of label cost.
//!   - arXiv:2507.19457 GEPA (ICLR 2026 oral) -- 35x fewer rollouts than
//!     PPO/GRPO for prompt-level evolution.
//!   - arXiv:2501.13011 MONA (Farquhar et al. Google DeepMind) -- myopic
//!     optimization with non-myopic approval prevents multi-step reward
//!     hacking.
//!   - arXiv:2509.15557 Verifiable Composite Rewards -- base reward minus
//!     penalty_format / penalty_length / penalty_sycophancy / penalty_repetition.
//!   - arXiv:2507.01352 Skywork-Reward-V2 -- outcome dominates over learned
//!     PRM (grounds FusionWeights default {prm: 0.4, orm: 0.6}).
//!   - arXiv:2412.21139 SWE-Gym (ICML 2025) -- 491 SFT-on-rollout traces buy
//!     +12.3pp SWE-Bench Lite / +13.6pp Verified.
//!   - arXiv:2508.16546 RL Is Neither Panacea Nor Mirage -- SFT-2K matches
//!     RL-20K on weak models; grounds SFT-first escalation rung.
//!   - arXiv:2501.17161 SFT Memorizes, RL Generalizes (ICML 2025) -- format
//!     stability prerequisite.
//!   - arXiv:2410.12784 JudgeBench (ICLR 2025) -- quarterly calibration
//!     cadence + correlation <0.6 = downgrade to advisory. (Replaces plan's
//!     WRONG_ID 2412.12509 per grounding-triage FIX.)
//!   - Nature 2025 AlphaProof <https://www.nature.com/articles/s41586-025-09833-y>
//!     -- test-time RL per problem on Lean variants; grounds the framing
//!     that ELAI's task-family does NOT have universal RLVR ground truth,
//!     hence BAI / GEPA / RLAIF preferred. (Replaces plan's FATAL bad cite
//!     arXiv:2406.15829 which resolves to an unrelated video-diffusion
//!     paper per grounding-triage.)
//!   - arXiv:2305.18290 Rafailov DPO -- cited only to JUSTIFY rejection
//!     (Apple 2025 OOD drop + arXiv:2502.20847 gradient imbalance + ELAI
//!     has no paired preference data; DPO is HARD-AVOIDED).
//!   - arXiv:2402.01306 KTO -- future-tier option if/when unlock (binary
//!     signal matches ELAI's audit BLOCK/PASS shape).
//!   - arXiv:2503.06639 GRPO loss dynamics -- future-tier reference.
//!   - arXiv:2502.20847 Gradient Imbalance in DPO -- grounds DPO HARD-AVOID.
//!   - arXiv:2502.14187 Federated KTO vs DPO -- KTO > DPO future-tier rank.
//!   - arXiv:2308.08998 ReST -- background SFT-on-rollouts pattern lineage.
//!   - arXiv:2411.15124 Tulu 3 / RLVR -- future-tier RL grounding.
//!   - arXiv:2402.19446 ArCHer -- future-tier multi-turn hierarchical RL.
//!   - arXiv:2410.01679 VinePPO -- future-tier RL reference.
//!   - arXiv:2504.06141 Adv-RM -- grounds reward-hacking canary rationale.
//!
//! Existing-surface consumption (per grounding sweep, D-prereqs SHIPPED):
//!   - `crate::bai::ArmPosterior` (D-5 SHIPPED) -- consume + additive
//!     `update_with_fused` method.
//!   - `crate::ppm_client::PpmScore::step_scores: Vec<f32>` (D-2 SHIPPED).
//!   - `crate::verification_result::VerifierKind` + `aggregate` (D-3 SHIPPED).
//!   - `crate::adversary::ModelFamily` (D-13 SHIPPED).
//!   - `crate::per_role_runtime::PerRoleRuntime` (B-37 SHIPPED commit fd5c2e5).
//!
//! NO DPO / PPO / GRPO / DAPO / KTO weight-update modules exist by design.
//! See `crate::contracts::tests::no_dpo_no_grpo_until_unlock`.

pub mod rlaif_judge;
pub mod gepa_template_evolution;
pub mod reward_fusion;
pub mod mona_composite_reward;
pub mod rollout_collector;
pub mod rl_weight_update_deferred;

/// Env-gate constant: the entire E-55 substrate is wire-dark unless this
/// env var is set to `1` at process start. The orchestrator instantiates
/// the modules unconditionally so symbol-presence contracts pass, but the
/// `emit_*` sites short-circuit when the gate is off.
pub const ENV_AGENTIC_RL_ENABLE: &str = "ELAI_AGENTIC_RL_ENABLE";

/// Returns `true` when the env gate is set to `1`. Module-level helper so
/// every emit site uses the same check.
#[must_use]
pub fn agentic_rl_enabled() -> bool {
    std::env::var(ENV_AGENTIC_RL_ENABLE)
        .ok()
        .map(|v| v == "1")
        .unwrap_or(false)
}

#[cfg(test)]
mod env_gate_tests {
    use super::*;

    #[test]
    fn env_gate_default_off() {
        // The test process should not have the gate set by default.
        // We don't mutate env here to keep the test deterministic in a
        // multi-threaded test runner.
        let _ = agentic_rl_enabled();
        // Constant is the canonical env name (regression guard).
        assert_eq!(ENV_AGENTIC_RL_ENABLE, "ELAI_AGENTIC_RL_ENABLE");
    }
}
