Algorithm: Rewarding Doubt (RL-trained calibrated confidence expression)
ArXiv: 2503.02623
Paper: Rewarding Doubt: A Reinforcement Learning Approach to Calibrated Confidence Expression of Large Language Models
Authors: Bani-Harouni et al., TU Munich, v6 Feb 2026

Inputs:
question | str | natural language question | mandatory
answer | str | pre-generated model answer (frozen during confidence training) | mandatory; generated in separate step, fixed before confidence optimization
confidence_integer | int | integer in [0,10] | mandatory at inference; 0 = certain incorrect, 10 = certain correct
correctness_signal | binary | j(a) in {0,1} | mandatory during training only; binary exact-match or F1 > 0.5 threshold

Output format at generation:
"Answer: <answer>, Confidence: <confidence_integer>"
Out-of-format outputs receive reward -3 (hard penalty, not part of log-scoring).

Outputs:
p_hat | float | confidence probability in [epsilon, 1-epsilon] | derived as p_hat = c / 10 where c is confidence_integer (implicit; normalization not stated explicitly in paper)
reward | float | scalar in [-1, 1] after normalization | R(a, p_hat, j) = log(p_hat) if correct, log(1 - p_hat) if incorrect; clipped at epsilon=0.001
calibrated_confidence | int | [0,10] | trained output; invariant: E[R] maximized iff p_hat = p* (Proposition 1)

Hyperparameters:
epsilon (clip) | 0.001 | (0, 0.1) | clips p_hat away from 0/1 to avoid log(0); small enough to preserve Proposition 1 optimality near-exactly
reward_multiplier | 1x (single-answer), 5x (multi-answer) | {1, 5} | applied to reward before PPO update; 5x used for QAMPARI because reward spread too small otherwise
out_of_format_penalty | -3 | fixed | applied when model fails to output "Answer: ..., Confidence: ..." format; note: -3 is outside [-1,1] normalized range, a hard formatting gate
learning_rate | 1e-5 | paper does not vary | single value used across all experiments; no sensitivity analysis
epochs | 2 (single-answer) | single value | for QAMPARI: 24,000 steps with batchsize 8 instead
LoRA | 4-bit quantized via Unsloth + LoRA | not varied in main paper | used for memory efficiency on single A40

Training setup:
RL algorithm: PPO (Schulman et al. 2017) — NOT GRPO. Confirmed from Section 4 "Implementation Details": "We optimize the reward function using the Proximal Policy Optimization (PPO) algorithm."
Base model (primary): Meta-Llama-3-8B-Instruct (4-bit Unsloth quantized)
Ablation models: LLaMA-3.1-8B, Qwen-2.5-3B, Qwen-2.5-7B, Gemma-2-9B
Hardware: single Nvidia A40; 7 days per training run
Answers frozen: YES. Answers pre-generated, treated as fixed inputs. Only confidence generation is the optimization target.
Dataset (single-answer training): TriviaQA (~28K questions, questions+answers only, no evidence)
Dataset (multi-answer training): QAMPARI (questions+answers only, no evidence)

MDP formulation (Section 3):
State s_t = (q, a, c_{1:t-1}): question, frozen answer, partial confidence tokens so far
Action a_t: next token from LLM vocabulary (numerical token or EOS)
Transition: deterministic autoregressive; s_{t+1} = (q, a, c_{1:t})
Reward: terminal only (on complete confidence expression): R(a, p_hat, j)

Critical invariants (testable assertions):
# Proposition 1: expected reward maximized iff p_hat = p*
# f(p_hat) = p* * log(p_hat) + (1 - p*) * log(1 - p_hat)
# df/dp_hat = p*/p_hat - (1-p*)/(1-p_hat) = 0 => p_hat = p*
# f''(p_hat) = -p*/p_hat^2 - (1-p*)/(1-p_hat)^2 < 0 everywhere on (0,1) => global max
assert epsilon > 0 and epsilon < 0.01  # clipping value used: 0.001
assert p_hat >= epsilon and p_hat <= (1 - epsilon)  # confidence always clipped
assert confidence_integer >= 0 and confidence_integer <= 10  # output range
# p_hat mapping: p_hat = confidence_integer / 10.0  (implicit in paper)
assert abs(p_hat - confidence_integer / 10.0) < 1e-9
# reward range after normalization: [-1, 1]
# raw log score at epsilon: log(0.001) ~ -6.9; normalization maps this to -1
# raw log score at 1-epsilon correct: log(0.999) ~ -0.001; normalization maps to ~1
# reward for correct answer with confidence 5 (p_hat=0.5): log(0.5) ~ -0.693
# reward for incorrect answer with confidence 5: log(0.5) ~ -0.693 (symmetric at 0.5)
assert reward >= -1.0 and reward <= 1.0  # after normalization

Complexity:
Training: PPO over autoregressive confidence token generation; O(L) per forward pass where L = length of confidence expression (short: 1-2 tokens for integer + EOS)
Space: 4-bit quantized LLaMA-3-8B fits on single A40 (48 GB VRAM) with LoRA
Inference: single forward pass; confidence generation is separate from answer generation
Parallelism: standard data-parallel; no constraint stated beyond single-GPU training

Calibration results (verified from paper):
Single-answer TriviaQA: ECE 0.3459 (Verbalize baseline) -> 0.0226 (Rewarding Doubt)
Multi-answer QAMPARI: ECE 0.5319 (Verbalize) -> 0.0816 (Rewarding Doubt)
OOD MedQA (zero-shot transfer): ECE 0.4480 -> 0.1145
OOD CommonsenseQA (zero-shot transfer): AUROC 0.5425 -> 0.6385
Accuracy: stable across training (no notable degradation)

Failure modes (from paper):
1. Tested only on 3B-9B models; authors note no empirical validation on larger models (>9B)
2. Correctness signal is binary only; cannot handle partial correctness or graded quality
3. CommonsenseQA ECE comparable to baselines despite AUROC improvement; calibration gains not uniform across domains
4. Model trained on single-answer setting underperforms on multi-answer tasks (no cross-task transfer for format mismatch)
5. Reward hacking not studied; paper assumes log-scoring's theoretical properties prevent gaming without empirical validation
6. User over-reliance risk: authors flag that calibrated confidence may create false trust

D-16 relevance (refusal-cost / coverage-loss tradeoff):
The paper does NOT implement abstention/refusal. There is no coverage-abstention tradeoff curve.
Low confidence (confidence=0, p_hat~0) is the closest analog to "implicit abstention" but the model always produces an answer.
Selective prediction / conformal prediction: NOT connected in this paper.
The reward function is a PROPER SCORING RULE: it is theoretically optimal to express true epistemic uncertainty, not to game thresholds.
D-16 MUST NOT assume this paper provides a refusal reward signal. It provides a confidence calibration signal only.
D-16 can adopt the log-scoring reward as the calibration sub-component of a coverage-loss objective but must add its own abstention/threshold mechanism separately.
The refusal-as-reward concept (RLKF by Xu et al. 2024) is cited in related work but not implemented here.

Implementation pitfalls (do NOT do this):
Do NOT use GRPO - the paper uses PPO. If using GRPO in a re-implementation, that is a deliberate deviation.
Do NOT jointly optimize answer and confidence - answers must be frozen/pre-generated before confidence training.
Do NOT skip the epsilon clipping - log(0) is undefined and will produce NaN gradients.
Do NOT assume the integer->probability mapping is anything other than p_hat = c/10 (0-10 scale).
Do NOT use the raw unnormalized log reward for PPO updates without normalizing to [-1,1]; training instability results.
Do NOT treat the out-of-format penalty (-3) as part of the calibration reward; it is a separate formatting gate.
Do NOT use this paper to justify abstention/refusal mechanisms; it provides no coverage guarantee or refusal reward.
Do NOT apply to tasks without binary correctness judgments; the reward requires a clear j(a) in {0,1}.
Do NOT expect calibration gains to transfer across domain types; CommonsenseQA shows mixed results.
Do NOT conflate AUROC improvement with ECE improvement; they measure different things (discrimination vs calibration).
