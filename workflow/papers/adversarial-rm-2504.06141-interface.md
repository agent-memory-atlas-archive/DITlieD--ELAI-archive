Algorithm: Adv-RM (Adversarial Training of Reward Models)
ArXiv: 2504.06141
Paper: Adversarial Training of Reward Models
Authors: (preprint, under review, submitted April 8 2025, revised April 11 2025)
Verified: WebFetch of https://arxiv.org/html/2504.06141v2 on 2026-05-25

D-28 SCOPE NOTES:
  "PPM" does not appear in this paper. Internal D-28 use of "PPM" is a project term, not paper terminology.
  "+5-7pp accuracy gains" is NOT grounded in this paper. Paper shows +0.7pp overall RewardBench, +2.86pp on Chat-Hard subcategory.
  The "+5pp acceptance gate" in D-28 is ungrounded relative to this paper's results and must be re-justified or recalibrated.
  The farming loop the paper calls "Adv-RM" maps to what D-28 calls "adversarial example farming methodology."


Inputs:
prompt x | text | single prompt string | drawn from held-out evaluation set; 128 prompts used in experiments (mandatory)
SFT policy pi_SFT | LLM | generates baseline responses y_SFT | must be instruction-tuned; Llama-3.1-8B-Instruct used (mandatory)
ensemble of K reward models R_theta_1..R_theta_K | RM | K>=2; K=2 in all experiments | must share same input/output interface (mandatory)
gold reward model R_gold | RM | used only for evaluation, not training | Llama-3.1-Nemotron-70B-Reward used in synthetic setup (optional at train time, mandatory for evaluation)
original RLHF preference dataset | preference pairs | arbitrary size | Helpsteer-2-Preferences used; ~1000 adversarial pairs added per round (mandatory)
lambda (disagreement weight) | float | scalar | tuned over {8, 10, 12}; controls trade-off between high target-RM reward and ensemble disagreement (mandatory hyperparameter)
threshold T(x) | float | per-prompt scalar | mean reward of pi_SFT on prompt x under R_theta_1; computed dynamically per prompt (mandatory, computed)
constant C | float | scalar | -25 in experiments; penalty reward when adversarial response does not exceed T(x) (mandatory hyperparameter)


Outputs:
adversarial response y_adv | text | single response per (prompt, policy-step) pair | satisfies R_theta_1(x, y_adv) > T(x) AND Z(U(x, y_adv)) > 1.96 (filtered)
adversarial preference pair (x, y_SFT, y_adv) | triple | one pair per accepted adversarial example | y_SFT is preferred, y_adv is rejected; y_SFT must itself exceed mean pi_SFT reward for x
adversarial dataset D_adv | set of preference pairs | ~1000 pairs per round | quality-filtered; more than ~1000 pairs hurts performance (saturation)
hardened reward model R_theta_hardened | RM | same architecture as base RM | trained from scratch on D_original union D_adv; RewardBench overall: 83.99 vs baseline 83.29


Hyperparameters:
lambda | 10 (best in ablations, tuned over {8,10,12}) | [8,12] tested | controls disagreement signal weight; high sensitivity reported implicitly (critical)
C | -25 | not ablated | penalty for responses that do not exceed T(x); drives policy away from low-reward regions
Z-score threshold | 1.96 | fixed | 95th percentile of SFT uncertainty distribution; samples below discarded; not ablated in paper
adversarial dataset size per round | ~1000 | tested: {500, 1000, 2000, 5000} implicitly | CRITICAL: >1000 hurts performance due to diversity saturation; 1000 is the sweet spot
max training rounds | 2 | fixed | after round 2 attack success drops to near zero; additional rounds yield diminishing returns
adversarial policy LR | 5e-7 | range [1e-7, 1e-6] typical for RL fine-tune | Adam optimizer; not ablated
RM retraining LR | tuned from {1e-6, 5e-6, 7e-6, 1e-5} | selected via RewardBench checkpoint selection | single epoch
RM retraining batch size | 128 | not ablated |
adversarial policy RLOO batch size | 64 | not ablated | RLOO = Reinforcement Learning Leave-One-Out (Ahmadian et al. 2024)
RLOO samples per prompt | 4 | not ablated |


Critical invariants (testable assertions):

# Eq. 3 / Section 3.2: adversarial reward function shape
# R_adv(x,y) = R_theta1(x,y) - lambda*R_theta2(x,y)  if R_theta1(x,y) > T(x)
# R_adv(x,y) = R_theta1(x,y) - C                      otherwise
# Invariant: C must be sufficiently negative to prevent policy from gaming the else branch
assert C <= -10, "C must be strongly negative (paper uses -25) to penalize sub-threshold responses"

# Section 3.2 / D_adv definition: dual filtering gate
# A sample enters D_adv IFF both conditions hold
assert lambda_z_threshold == 1.96, "Z-score gate must be 1.96 (95th pct of pi_SFT uncertainty distribution)"
# assert R_theta1(x, y_adv) > T(x)  -- enforced by reward function structure
# assert Z(std(R_theta1(x,y_adv), R_theta2(x,y_adv))) > 1.96  -- enforced by post-hoc filter

# Section 3.4: preference pair polarity
# y_SFT = preferred (positive), y_adv = rejected (negative)
# AND y_SFT itself must exceed mean pi_SFT reward for x
assert reward_ysft > mean_reward_pisft_on_x, "preferred response must be above average SFT reward (not just better than y_adv)"

# Section 5: dataset size saturation
assert adversarial_dataset_size <= 1500, "paper finds >~1000 samples hurts performance; cap at 1000-1500"

# Section 5: round termination
# Stop retraining rounds when attack success rate of Adv-RM policy against hardened RM drops near zero
# Paper terminates after round 2 in all experiments
assert max_rounds <= 3, "empirically, 2 rounds sufficient; beyond 2 diminishing returns observed"

# Eq. 1 / Section 3.1: standard Bradley-Terry RM loss (used for RM retraining)
# L(theta) = -E[log(sigma(R_theta(x, y_w) - R_theta(x, y_l)))]
# Invariant: y_w is the preferred response, y_l is rejected; loss is cross-entropy over reward gap


Complexity:
Adversarial policy training | O(N_prompts * N_RLOO_samples * RM_forward_pass * 2) per step | K=2 RM forward passes per response | embarrassingly parallel across prompts
RM retraining | O(|D_original| + |D_adv|) * 1 epoch | single epoch; from scratch each round | parallelizable
Total cost vs baseline | ~3x compute cost of standard RLHF training | 672 A100-hours (Adv-RM) vs 224 A100-hours (baseline RLHF) | Appendix F


Failure modes:
Ensemble blind spot: if all K RMs agree a low-quality response is high-quality, adversarial examples are missed; diversity of RM ensemble is critical
Diversity saturation: >~1000 adversarial examples per round causes performance degradation; the farming loop produces similar failure modes that do not generalize
Uncertainty as quality signal is noisy: Pearson correlation between uncertainty and gold reward is -0.05 when sampling from pi_SFT (Section 3.3); uncertainty is useful only when high, not for fine-grained ranking
3x compute cost: substantially limits applicability in resource-constrained settings
Round 2 plateau: the hardened RM after round 2 resists Adv-RM attacks; further rounds yield near-zero new adversarial examples
Proxy RM quality: adversarial policy is trained against proxy RM (e.g. Llama-3.1-8B-Instruct based RM); quality of proxy RM bounds the quality of farmed examples


Implementation pitfalls (do NOT do this):
Do NOT use adversarial examples as preferred side of preference pairs; y_adv is ALWAYS the rejected response
Do NOT skip the Z-score filter; training on low-uncertainty adversarial examples adds noise without improving robustness
Do NOT train RM for more than 1 epoch on augmented dataset; paper uses single epoch (overfitting risk on small adversarial set)
Do NOT use more than ~1000 adversarial samples per round; performance degrades with more due to diversity collapse
Do NOT claim +5-7pp accuracy gains from this paper; verified gains are +0.7pp overall RewardBench and +2.86pp on Chat-Hard; larger gains are not reported
Do NOT use K=1 (single RM) for uncertainty estimation; uncertainty metric requires ensemble STD (K>=2)
Do NOT add adversarial data incrementally to an existing RM checkpoint; paper trains RM from scratch each round
Do NOT conflate "PPM" (project-internal term) with any algorithm in this paper; the paper has no PPM concept
Do NOT use RM over-optimization (naive reward maximization) as a substitute; it achieves only 8.31% attack success vs Adv-RM 99.31%
Do NOT apply Adv-RM to the gold RM directly; only apply to proxy/target RMs; the gold RM is evaluation-only


Quantitative results (verified from paper):
RewardBench overall: baseline 83.29, Adv-RM 83.99 (+0.7pp)
RewardBench Chat-Hard: baseline 61.18, Adv-RM 64.04 (+2.86pp) -- largest subcategory gain
RewardBench Chat: baseline 97.21, Adv-RM 96.09 (-1.12pp) -- slight regression
RewardBench Safety: baseline 85.41, Adv-RM 85.14 (-0.27pp) -- slight regression
RewardBench Reasoning: baseline 89.38, Adv-RM 90.70 (+1.32pp)
Attack success rate (Adv-RM vs synthetic RM, standard criterion): 99.31% train, 100% test
Attack success rate (Adv-RM vs Skyworks Gemma 27B, human judge): 100%
Attack success rate (Adv-RM vs Nemotron 340B, human judge): 78.85%
RLHF longevity: Adv-RM trained RM allows 3x more RLHF steps before reward hacking manifests
