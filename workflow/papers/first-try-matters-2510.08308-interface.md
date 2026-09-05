Algorithm: First Try Matters — Reflection Role Analysis in Reasoning Models
ArXiv: 2510.08308
Paper: First Try Matters: Revisiting the Role of Reflection in Reasoning Models

Inputs:
rollout | reasoning trace | sequence of tokens | must contain candidate answer followed by reflection segment
model | LRM | 8 models studied | DeepSeek-R1 family, QwQ, others
dataset | math benchmark | 5 datasets | mathematical reasoning tasks only

Outputs:
reflection_type | enum | confirmatory | flip_wrong_to_right | flip_right_to_wrong
first_answer_correctness | float | [0,1] | accuracy of model before any reflection segment
post_reflection_correctness | float | [0,1] | accuracy after reflection; rarely higher for wrong-to-right

Hyperparameters:
early_stop_threshold | N/A | N/A | "a few plausible candidate answers" — not quantified; dynamic truncation achieves 24.5% token reduction / 2.9% accuracy drop

Critical invariants (testable assertions):
assert reflections_confirmatory_rate >> reflections_flip_wrong_to_right_rate  # paper's central finding
assert training_on_more_reflection_steps_improves_first_answer_not_correction  # SFT finding
assert early_stop_after_candidate_tokens_reduction >= 0.245  # 24.5% measured on 5 math datasets

Complexity:
time | O(reflection_segment_length) overhead per token | analysis is post-hoc on existing rollouts
space | constant | per-rollout classification

Failure modes:
Reflection is predominantly confirmatory — it does not reliably flip wrong answers to right on math tasks.
Training on longer reflection rollouts trains first-answer quality, not the ability to self-correct.
Dynamic truncation after candidate answer: 24.5% token reduction with 2.9% accuracy drop — acceptable trade for 7B-class.

Implementation pitfalls (do NOT do this):
Do NOT assume reflection tokens improve accuracy on a 7B — the paper shows they are mostly confirmatory.
Do NOT use this paper to justify inference-time token caps proportional to confidence — the paper does not propose that mechanism.
Do NOT apply these findings outside math reasoning datasets — scope is 5 math datasets only.
Do NOT treat flip-wrong-to-right rate as zero — it is "rare" not zero; early stopping has a 2.9% cost.
The paper is analysis-only. It does NOT provide a training algorithm for reducing reflection (that is ARLCP 2602.12113).
