Algorithm: ARLCP — Adaptive Reflection and Length Coordinated Penalty
ArXiv: 2602.12113
Paper: Stop Unnecessary Reflection: Training LRMs for Efficient Reasoning with Adaptive Reflection and Length Coordinated Penalty

Inputs:
base_model | LRM | DeepSeek-R1-Distill-Qwen-1.5B or 7B | finetuned with RLVR; ARLCP adds additional RL training pass
training_rollouts | RLVR trajectories | sequence of (prompt, completion, outcome) | must have outcome signal for penalty computation
reflection_penalty_coeff | float | dynamic | calibrated per-instance by estimated problem complexity
length_penalty_coeff | float | dynamic | calibrated per-instance by estimated problem complexity

Outputs:
policy | trained model | weights | model that generates shorter but accurate reasoning chains
length_reduction | float | 35.0% for 7B, 53.1% for 1.5B | measured on 5 math benchmarks
accuracy_delta | float | +2.7% for 7B, +5.8% for 1.5B | measured on 5 math benchmarks

Hyperparameters:
reflection_penalty | dynamic per instance | [0, ...] | RLOO advantage estimator; not a fixed cap
length_penalty | dynamic per instance | calibrated to problem complexity | not proportional to confidence score

Critical invariants (testable assertions):
assert method == "training_time_rl"  # ARLCP is NOT an inference-time method
assert inference_token_cap_not_proposed == True  # paper proposes no per-inference confidence-proportional cap
assert length_reduction_7B == 0.35  # verified number
assert accuracy_gain_7B == 0.027  # verified number

Complexity:
time | additional RL training pass on top of base RLVR training | inference cost unchanged after training
space | standard RL training memory | no inference overhead

Failure modes:
ARLCP requires model retraining — gains do NOT transfer to inference-only deployments of existing weights.
The 35% length reduction is a trained behavior, not achievable via prompt engineering or token caps alone.
Applying a hard inference-time token cap to achieve the 35% reduction without retraining will degrade accuracy differently than reported.

Implementation pitfalls (do NOT do this):
Do NOT use the 35.0% / 2.7% numbers to justify an inference-time L2_MAX_TOKENS = 240 cap — those numbers come from a training-time RL method, not an inference cap.
Do NOT use ARLCP to claim that token caps proportional to L1 confidence are supported by the paper — the paper has no confidence-proportional cap mechanism.
Do NOT apply to non-math tasks without re-evaluating — validated on 5 math benchmarks only.
The 35% reduction applies to the full reasoning chain, not just L2 reflection segment specifically.
