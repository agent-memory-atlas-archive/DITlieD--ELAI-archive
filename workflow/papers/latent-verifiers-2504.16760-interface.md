Algorithm: LiLaVe (Lightweight Latent Verifier)
ArXiv: 2504.16760
Paper: Lightweight Latent Verifiers for Efficient Meta-Generation Strategies
Authors: Piotrowski, Drzewakowski, Staniszewski, Milos (IDEAS NCBR / U. Warsaw / IMPAN / NVIDIA)
Scope-note: grounding for ELAI D-2 verifier-only runtime tier (PpmAdapter is_verifier_only=true)

Inputs:
hidden_state h_t^l | float32 tensor | shape [hidden_dim] (e.g. 4096 for Llama-3.1-8B) | extracted from layer l at token position t during base LLM forward pass; NO second LLM call needed
layer_index l | int | one of L_LiLaVe = {-1, -2, -4, -8, -16} (negative = from output layer) | mandatory
token_index t | int | one of T_LiLaVe = {-16, -15, ..., -1} (last 16 generated tokens) | mandatory; signal strongest at suffix positions
correctness_label c | int | {0, 1} | training only; obtained by comparing final answer to ground truth string match
training_questions Q | list[str] | size >= 1000 per benchmark | used to build 5K training pairs per dataset (1000 q * 5 temperatures)
temperature_set | list[float] | {0.0, 0.25, 0.5, 0.75, 1.0} | training distribution; mix ensures robustness across inference temps

Outputs:
score | float | scalar in [0.0, 1.0] | interpreted as P(response is correct); aggregated by arithmetic mean over all |L|*|T| = 5*16 = 80 location scores; invariant: 0.0 <= score <= 1.0
per_location_score s_h | float | scalar in [0.0, 1.0] | one XGBoost classifier output per (l, t) pair; intermediate value before aggregation

Verifier classifier (XGBoost head):
classifier type | XGBoost (gradient boosted decision trees) | selected over LogReg, MLP, SWIGLU after ablation (Appendix A.2, Figure 9)
input features | h_t^l concatenated with encoded (l, t) location indices | dim = hidden_dim of base LLM
max_depth | 10 | range 1..30 | not flagged as high-sensitivity
eta (learning rate) | 0.1 | range 0.01..0.3 | authors note XGBoost defaults used for all other params
nrounds | 30 | range 10..100 | training takes ~3 min on CPU per location
training_size | 5000 instances per dataset | 1000 questions * 5 temperatures | minimum viable; authors used this exact count
num_classifiers | 80 | one per (l, t) pair; 5 layers * 16 token positions | fixed by L_LiLaVe and T_LiLaVe sets

Hyperparameters (meta-generation strategies):
threshold s | 0.5 (typical) | [0.0, 1.0] | controls trigger rate for conditional strategies; s=0 = always trigger, s=1 = never trigger; must be tuned per dataset
n_samples (best-of-n / majority voting) | 8 (paper experiments) | [2, 32] | more samples = better oracle ceiling but more compute
probe_temperature t0 (conditional MV) | 0.0 | [0.0, 1.0] | temperature for the initial probe sample; greedy decode for probe is typical
voting_temperature t_mv (conditional MV) | 0.5..1.0 | [0.0, 1.0] | temperature for fallback majority-vote samples; higher = more diverse candidates
aggregation_fn | mean | {min, max, mean} | mean outperformed min/max in ablation; do not substitute

Critical invariants (testable assertions):
assert 0.0 <= score <= 1.0, "LiLaVe score must be in unit interval"
assert len(hidden_states) == len(L_LiLaVe) * len(T_LiLaVe), f"Expected 80 hidden states, got {len(hidden_states)}"
assert all(abs(s) <= 1.0 for s in per_location_scores), "All per-location XGBoost probs must be in [0,1]"
assert aggregated_score == sum(per_location_scores) / len(per_location_scores), "Aggregation must be arithmetic mean, not weighted or geometric"
assert verifier_trained_on_same_base_llm, "Verifier classifiers are NOT transferable across base LLMs without retraining (Sec 1.p4, Appendix A.7 only 'promising' preliminary)"
assert hidden_states_extracted_during_generation_pass, "Hidden states come from the single generative forward pass; no second LLM call is required or correct"

Complexity:
training | O(|L|*|T|*|Q|*k * n_trees) on CPU | 80 classifiers * ~3 min each = ~4h total for full 80-head verifier; each classifier independent (parallelizable)
inference | O(|L|*|T|) XGBoost calls per generated response | negligible vs LLM forward pass; no additional LLM invocations
memory (verifier) | XGBoost model files << 1MB each; 80 models ~ tens of MB | vs LLM verifier at 7B-34B params (14-70+ GB); LiLaVe VRAM overhead is effectively 0
hidden_state extraction overhead | one hook per selected (l,t) position during generation | adds <1% to forward-pass wall-clock time; no recomputation
parallelism | 80 classifiers are fully independent; can score in parallel | single response scoring is serial unless explicitly parallelized

Failure modes:
verifier-oracle gap: LiLaVe AUC 0.86-0.93 but oracle (always picks correct if present) achieves far higher accuracy; verifier cannot match perfect selection (Section 5, Fig 10)
cross-model transfer unverified: verifiers trained on LLM-A degrade on responses from LLM-B; Appendix A.7 is preliminary with no published AUC numbers; do NOT assume transferability
unconditional self-correction degrades accuracy: applying self-correction without LiLaVe gate turns 15-30% of correct responses incorrect; the conditional trigger (score < s) is mandatory, not optional
temperature distribution mismatch: verifier trained on {0, 0.25, 0.5, 0.75, 1.0} shows reduced AUC if inference temperature is outside this range; always include target inference temp in training set
false-positive accumulation in best-of-n: P(at least one false positive) grows with n; at high n the verifier may select a confidently-wrong response; combine with majority voting for robustness
domain/task mismatch: all experiments on mathematical reasoning (GSM8K, MATH, algebra); transfer to code, open-ended QA, factual retrieval is unvalidated by authors
probe score underestimates entropy: Section 5 notes verifier score inversely correlated with answer entropy in downstream samples, but adaptive voting not yet implemented; treating score as a direct diversity proxy is ungrounded

Implementation pitfalls (do NOT do this):
do NOT run a second LLM forward pass to compute hidden states; states must come from the generation pass via registered hooks
do NOT use a single shared XGBoost classifier for all 80 (l, t) locations; each location has its own trained model
do NOT substitute geometric mean or min/max for arithmetic mean in aggregation; ablation showed mean is best
do NOT train on a single temperature; training data must cover multiple temperatures to generalize at inference time
do NOT assume LiLaVe scores are calibrated probabilities for ensemble fusion with external rerankers; they are ordinal within a single base LLM
do NOT re-use a verifier trained on one base LLM for a different base LLM without retraining on that model's hidden states
do NOT use MLP or SWIGLU head as drop-in replacement for XGBoost without ablation; they underperformed in paper experiments and may be sensitive to hyperparameter tuning

ELAI integration notes (scope: D-2 PpmAdapter is_verifier_only=true):
score_function signature: fn score(hidden_states: &[Vec<f32>], layer_indices: &[i32], token_indices: &[i32]) -> f32
hidden_states must be passed from the generation loop, not computed post-hoc
PpmAdapter in verifier-only mode does NOT need to run its own generation; it only classifies hidden states provided by the policy model
threshold s is a runtime config parameter, not a compile-time constant; expose via RuntimeConfig
the 80 classifier files (one per location) are the serialized model artifact; load on init, keep in memory
VRAM impact of LiLaVe itself on the RTX 3090 sidecar: negligible (XGBoost runs on CPU alongside GPU inference)
