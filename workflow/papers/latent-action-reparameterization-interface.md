Algorithm: Latent Action Reparameterization (LAR)
ArXiv: 2605.18597
Paper: Latent Action Reparameterization for Efficient Agent Inference
Authors: Wenhao Huang, Qingwen Zeng, Qiyue Chen, Zijie Guo, Yu Sun, Cheng Yang, Siru Ouyang, Jiri Gesi, Fang Wu, Jiayi Zhang, Huaming Chen, Bang Liu, Xiangru Tang, Chenglin Wu
Date: May 18, 2026
Code: https://github.com/EZ-hwh/LAR
License: CC BY 4.0

Inputs:
trajectory | sequence of (observation, action) pairs | tau = [(o1,a1), ..., (oT,aT)] | actions are token sequences; observations are arbitrary text
n_gram_range | tuple[int,int] | (min_n, max_n) | e.g. (2,6) or (3,5); defines segment length for candidate extraction
f_min | int | scalar >= 1 | frequency threshold; per-task values: TriviaQA=2000, KodCode=10, Mind2Web=1000
H_max | float | scalar >= 0 | entropy ceiling; all tasks use 10.0; prevents abstracting high-entropy parameter-binding content
top_K | int | scalar >= 1 | max latent action vocabulary size; range 100-1000 depending on task
overlap_threshold_rho | float | [0.0, 1.0] | deduplication overlap threshold; all tasks use 0.7
teacher_model | frozen LLM | same architecture as student | produces logits z^T over shared positions M
student_model | LoRA-adapted LLM | same base as teacher | trainable: LoRA weights + new token embeddings (~0.1% of parameters)
distillation_temperature | float | scalar > 0 | paper uses 2.0 for all experiments
lora_rank | int | 8 (8B models), 16 (32B models) | controls LoRA capacity; alpha = 2x rank
learning_rate | float | 1e-4 (distillation), 1e-6 (GRPO) | cosine schedule with 100 warmup steps

Outputs:
reparameterized_trajectory | sequence | tau_hat; original tokens with latent symbols substituted | latent symbols are contiguous with normal vocabulary; no special decoding needed
latent_vocabulary | dict[str -> token_id] | segment_text -> new_token_id | K entries; segments pass freq + entropy filters
task_success_rate | float | [0.0, 1.0] | maintained or improved vs vanilla baseline (see Table 1)
token_reduction | float | [0.0, 1.0] | fraction of tokens eliminated; TriviaQA up to -27.1%, KodCode -9.8%, Mind2Web -20.8%
throughput_gain | float | tokens/s | +17.5% on TriviaQA (Qwen3-8B); task-dependent

Hyperparameters:
n_gram_range | (3,5) for QA; (2,6) for code/web | valid: any (min>=1, max>=min) | low sensitivity within reasonable bounds; wider range finds more candidates
f_min | 2000 (QA), 10 (code), 1000 (web) | valid: >= 1 | HIGH sensitivity; too low includes high-entropy segments causing Phase III collapse
H_max | 10.0 | valid: > 0 | HIGH sensitivity; primary guard against semantic collapse; do not raise above 10.0 without validation
top_K | 200-1000 (QA), ~100 (code/web) | valid: >= 1 | moderate sensitivity; more symbols increase coverage but may conflict in unified training
overlap_threshold_rho | 0.7 | valid: [0,1] | deduplication aggressiveness; lower = more symbols, higher = fewer
distillation_temperature | 2.0 | valid: > 0 | standard KD value; authors do not report ablation
lora_rank | 8 (8B), 16 (32B) | valid: >= 1 | standard LoRA values; alpha = 2x rank in all experiments
batch_size | 16 (distillation) | 16 | standard
learning_rate | 1e-4 (distillation), 1e-6 (GRPO) | cosine + 100 warmup | standard; not ablated
epochs | 3 (8B distillation), 2 (32B) | | more epochs not reported
precision | fp16 | | training precision; not ablated

Critical invariants (testable assertions):
assert all(entropy(segment) <= H_max for segment in latent_vocabulary)  # Eq. for H(s); entropy filter is mandatory, not optional
assert all(frequency(segment, corpus) >= f_min for segment in latent_vocabulary)  # frequency filter prerequisite
assert abs(sum(p for p in kl_positions) / len(M) - loss) < 1e-6  # KL loss averaged only over shared positions M, not latent symbol positions
assert all(latent_symbol in model.tokenizer.vocab for latent_symbol in latent_vocabulary)  # latent symbols are first-class vocab entries; no special decoding path
assert lora_alpha == 2 * lora_rank  # all reported configs use this ratio
assert distillation_temperature == 2.0  # paper fixes this; deviation is untested
assert not any(is_parameter_binding(seg) for seg in latent_vocabulary)  # executability constraint: search queries, entity names MUST NOT be abstracted

Complexity:
Stage 1 (segment extraction) | O(N * max_n) time over trajectory corpus of N tokens | O(V) space for candidate dict
Stage 2 (vocab construction) | O(K * L) for longest-first matching over L trajectory tokens | O(K) space
Stage 3 (training data prep) | O(L) per trajectory | O(L) space
Stage 4 (distillation training) | O(B * L * d^2) per step, standard transformer; LoRA reduces trainable param count to ~0.1% | 8x H200 140GB for 8B models
Inference overhead | zero; latent symbols processed as ordinary tokens via standard embedding lookup | no additional memory beyond K new embedding vectors

Failure modes:
Phase III semantic collapse: compressing high-entropy parameter-binding segments (search queries, entity names, numeric arguments) breaks environment-facing transitions; onset is abrupt not gradual (Figure 3)
Unified cross-domain training reduces per-domain compression efficiency; domain-specific tokens compete for shared vocabulary slots (Table 7)
Reasoning-heavy tasks with diverse free-form content admit small compressible subsets; LAR gains are smaller on open-ended generation
f_min too low includes rare high-entropy segments that should be excluded; causes silent degradation without explicit failure
H_max set too high admits parameter-binding content; same failure mode as above
32B models require higher LoRA rank (16) and lower LR (5e-5); using 8B defaults on 32B degrades training stability

Implementation pitfalls (do NOT do this):
Do NOT apply longest-first matching without the entropy filter first; order matters (filter then match)
Do NOT compute KL loss over latent symbol positions; loss is restricted to shared content positions M only
Do NOT use a separate decoding head for latent symbols; they must go through standard embedding + transformer path
Do NOT abstract segments that contain slot-filling variables, entity references, or numeric parameters even if they appear frequently
Do NOT train on unified multi-domain data expecting per-domain compression parity; domain-specific models compress more
Do NOT skip the overlap deduplication step (rho threshold); redundant symbols waste vocabulary slots and destabilize embeddings
Do NOT set f_min below task-appropriate floor; KodCode uses 10 (small corpus) while TriviaQA uses 2000 (large corpus); these are not interchangeable
Do NOT treat token reduction as the primary optimization target; it is a byproduct of semantic abstraction, not the objective

Benchmarks and reported numbers:
Table 1 (main results): Qwen3-8B TriviaQA LAR 80.09% vs Vanilla 67.40% (-27.1% tokens); KodCode LAR 54.30% vs Vanilla 34.44% (-9.2% tokens); Mind2Web LAR 39.84% vs Vanilla 36.73% (-2.9% tokens)
Table 1 Llama-3.1-8B: TriviaQA LAR 72.46% vs Vanilla 73.63% (-23.3% tokens); KodCode LAR 35.10% vs Vanilla 31.13% (-9.8%); Mind2Web LAR 28.30% vs Vanilla 24.40% (-20.8%)
Table 2 (held-out generalization): latent actions transfer to Musique, HumanEval, MBPP without retraining
Table 8 (throughput): Qwen3-8B TriviaQA +17.5% tokens/s (150.2 vs 127.8); peak GPU memory -2.9% to -6.8%
Table 10 (OpenClaw industrial): Short compression +27.0% EM; AllStatic compression +2.1% EM (Phase III boundary evident)
Table 5 (32B): Qwen3-32B LAR 75.26% vs ReAct 73.20% on TriviaQA
Table 9 (Mind2Web subtests): Cross-Task Qwen3 45.05% vs 39.56%; Cross-Website 26.37% vs 17.58%; Cross-Domain 35.64% vs 30.69%

Baselines LAR outperforms:
TokenSkip (token-level pruning): severe performance degradation
ACON (context compression): performance collapse on structured tasks
ConciseHint (prompt-based compression): inconsistent results

Reproducibility:
Code: https://github.com/EZ-hwh/LAR (public, stated in paper)
Datasets: TriviaQA, KodCode, Mind2Web are all public benchmarks
Hardware: 8x H200 140GB GPUs for training; inference numbers from same hardware
Training hyperparameters fully reported in Table 4
GRPO hyperparameters fully reported in Table 5
All backbone models are public (Meta-Llama-3.1-8B-Instruct, Qwen3-8B, Qwen3-32B)

Limitations (author-acknowledged):
Reasoning-intensive tasks with diverse free-form content have smaller compressible subsets; LAR gains are bounded by structural scaffolding fraction
Method is conservative by design: identifies largest safe abstraction subset rather than maximizing compression
Domain specificity required: unified multi-domain training shows reduced compression efficiency per-domain
Per-task threshold tuning required (f_min, H_max); no single universal config reported
