E-55 -- agentic RL + reward ladder rollout collection
======================================================

Layout

  rollouts/
    {YYYY-MM-DD}/
      {session_id}.jsonl
    _index.jsonl

Schema (one JSONL row per GREEN-session trajectory)

  {
    "session_id": "s-25120",
    "trajectory_id": "traj-1234",
    "prompt": "<user prompt>",
    "full_trajectory": "<flattened reasoning trace>",
    "ppm_scores": [<f32 per step>],
    "fused_reward": {
      "prm_step_scores": [...],
      "orm_signal": { "kind": "build_green"|"tests_pass"|"probe_semantic"|"verifier_consensus", ... },
      "fused_reward": <f32 in [0, 1]>,
      "weights": { "prm": 0.4, "orm": 0.6 },
      "mode": "linear"|"hierarchical"
    },
    "composite_reward": {
      "base": <f32>,
      "penalties": {
        "penalty_format": <f32>,
        "penalty_length": <f32>,
        "penalty_sycophancy": <f32>,
        "penalty_repetition": <f32>
      },
      "final_reward": <f32 in [0, 1]>,
      "myopic_reward": <f32>,
      "step_validity": <f32 | null>,
      "hacking_verdict": {
        "suspected": false,
        "myopic_reward": <f32>,
        "step_validity": <f32>,
        "divergence": <f32>
      }
    },
    "rlaif_rating": null | { "correctness": <f32>, "step_validity": <f32>, ... },
    "final_diff": "<unified diff>",
    "audit_verdict": "pass",
    "collected_at_ms": <i64>
  }

Eligibility

A trajectory enters the rollout pool only when ALL hold:
  - audit_verdict == Pass
  - composite_reward.hacking_verdict.suspected == false
  - if rlaif_rating is present, sycophancy_absence >= 0.5

SFT fine-tune threshold

  MIN_ROLLOUTS_FOR_SFT = 200 (per SWE-Gym arXiv:2412.21139 lower bound;
  SWE-Gym used 491 traces for +12.3pp / +13.6pp on SWE-Bench Lite /
  Verified). The 200th GREEN rollout fires
  SsbActionType::RewardLadderEvent with kind=SftTrainingThresholdReached
  -- the SFT-LoRA fine-tune patch (E-55.1) is unblocked at that point.

Live training is RUNPOD-PENDING

The collection surface ships fully. The actual SFT-LoRA fine-tune run
requires:
  - 2x3090 OR 24GB single-GPU rent (memory:project_vram_tier_plan)
  - PEFT + accelerate + HF transformers in
    python/elai_inference/pyproject.toml (NEW deps; require user
    confirmation per CLAUDE.md "No new dependencies without confirmation")
  - The unlock SSB row from the 3-condition gate
    (crates/orchestrator/src/rl/rl_weight_update_deferred.rs)

DO NOT add DPO / PPO / GRPO / DAPO / KTO modules pre-unlock. Contract
test crates/contracts/tests/no_dpo_no_grpo_until_unlock.rs HARD-BLOCKS
any such module. See workflow/reasoning/E-55-decisions.md.
