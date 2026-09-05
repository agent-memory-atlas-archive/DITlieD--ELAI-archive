Algorithm: Persona Vectors (Monitoring and Controlling Character Traits in LLMs)
ArXiv: 2507.21509
Paper: Persona Vectors: Monitoring and Controlling Character Traits in Language Models
Source: Anthropic (Jul 2025)
Reliability: HIGH (peer-reviewed, Anthropic internal research)

Inputs:
model_activations | tensor | shape [seq_len, hidden_dim] | residual stream activations from a specific layer
persona_pairs | list[tuple[str,str]] | list of (in-persona-example, out-of-persona-example) | min 10 pairs for stable extraction
target_trait | str | one of: sycophancy, evil, hallucination_propensity, or custom | well-defined trait with contrastive examples
layer_index | int | typically middle-to-late layers | model-specific; requires sweep to find best layer

Outputs:
persona_vector | tensor | shape [hidden_dim] | unit vector in activation space encoding trait direction
trait_score | float | scalar in [-1, 1] | projection of current activations onto persona_vector; monitors trait expression
steered_activations | tensor | shape [seq_len, hidden_dim] | activations after adding scaled persona_vector * coefficient

Hyperparameters:
steering_coefficient | alpha=1.0 | range [-5.0, 5.0] | high sensitivity; large alpha can override coherent outputs
layer_sweep_range | default: layers [0.4*L, 0.8*L] | middle-to-late layers | early/late layers less effective
contrastive_pair_count | default: 10-50 | minimum 10 | more pairs = more stable vector; diminishing returns above 100

Critical invariants (testable assertions):
assert abs(np.linalg.norm(persona_vector) - 1.0) < 1e-5  # persona vector must be unit norm
assert persona_vector.shape == (hidden_dim,)  # must match model hidden dimension exactly
assert -1.0 <= trait_score <= 1.0  # score is cosine similarity; bounded
# vectors are model-specific: vectors from model A must NOT be applied to model B
assert extracted_model_id == inference_model_id  # fail hard on cross-model application

Complexity:
Vector extraction: O(N_pairs * seq_len * hidden_dim) per forward pass; one-time at boot
Monitoring: O(seq_len * hidden_dim) per token; negligible overhead
Steering: O(seq_len * hidden_dim) per token; hook into decode loop

Failure modes:
Vectors extracted from one model do not transfer to different architectures or training conditions (model-specific)
Traits aligned with safety training (e.g., helpful, honest) show ceiling effects; cannot be steered away easily
Abliterated model variants may have different safety-alignment resistance than base instruct variants
Large steering coefficients (|alpha| > 3) can produce incoherent outputs
Complex or overlapping traits are harder to decompose; single vector may encode multiple traits
Long-term persistent steering has uncertain effects on multi-turn coherence

Implementation pitfalls (do NOT do this):
Do NOT reuse persona vectors across model checkpoints without re-extracting
Do NOT apply vectors extracted from larger model to smaller model of same family (dimensionality matches but direction may not)
Do NOT skip layer sweep; default middle layers are heuristic -- validate empirically per model
Do NOT use alpha > 2.0 for production steering without A/B coherence test
Do NOT claim Anthropic's quantitative effect sizes apply to Qwen2.5-Coder abliterated variants without validation
Do NOT apply steering to safety-critical outputs without safety override check
