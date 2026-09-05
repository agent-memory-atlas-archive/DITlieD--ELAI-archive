Algorithm: RecursiveMAS (Recursive Multi-Agent Systems)
ArXiv: 2604.25917
Paper: Recursive Multi-Agent Systems
Authors: Xiyuan Yang, Jiaru Zou et al., April 2026
Reliability: HIGH (fetched from arxiv.org HTML, cross-checked via two WebFetch passes)

CORE MECHANISM SUMMARY
RecursiveMAS casts a multi-agent system as a unified latent-space recursive computation. Instead of agents exchanging text messages, intermediate agents exchange hidden-state vectors (latent thoughts). Only the FINAL agent in the final recursion round produces text output. The key component is the RecursiveLink module, a small trained adapter (two linear layers + GELU) inserted between agents. All LLM backbone parameters are frozen; only RecursiveLink parameters are trained.

WHITE-BOX ACCESS REQUIREMENT (MOST IMPORTANT PORTABILITY FACT)
RecursiveMAS REQUIRES white-box access to the last-layer hidden states of each LLM transformer. It reads h_t (last-layer hidden representation at each generated token position) from inside the model's forward pass. This is INCOMPATIBLE with API-only models (Anthropic/OpenAI closed APIs that return only text). It requires either: open-weight models with direct tensor access, or a custom inference server exposing hidden states. This is not an inference-time wrapper; it requires modifying the generation loop to extract and inject embeddings.

RECURSIVELINK MODULE
Two variants:

Inner Link (within a single agent, Dense-to-Shallow Transition):
  Formula: R_in(h) = h + W2 * sigma(W1 * h)
  W1, W2 are linear layers; sigma is GELU activation
  Purpose: maps last-layer embeddings back into the input embedding space of the SAME agent for the next recursion round
  This allows an agent to "think again" on its own latent output without decoding to text

Outer Link (between different agents, Cross-Model Transition):
  Formula: R_out(h) = W3*h + W2*sigma(W1*h)
  W3 is an additional linear layer to handle different embedding dimensions between heterogeneous models
  Purpose: maps source agent A_i's last-layer hidden state into the input embedding space of target agent A_j
  Required when agents have different model families/hidden dimensions (e.g., Qwen3-1.7B -> Llama3.2-1B)

Layer extraction: always the LAST transformer layer hidden representation h_t at each generated token position t.

RECURSION ROUND MECHANISM
Round r: all N agents run sequentially (or in parallel for Mixture style) entirely in latent space.
  1. Agent A1 receives input context embeddings E = [e1, e2, ..., e_T]
  2. A1 runs transformer forward pass, extracting last-layer hidden states H_A1 = [h_t, h_{t+1}, ..., h_{t+m}] (m = latent thought length)
  3. Inner link maps H_A1 back to input embedding space for A1's next-round input
  4. Outer link maps H_A1 into A2's embedding space; A2 runs forward pass producing H_A2
  5. Continue agent chain: A2 -> A3 -> ... -> A_N all in latent space
  6. After A_N finishes, its latent output is passed back to A1, closing the loop
  7. Repeat for r rounds total
  8. ONLY in the FINAL round does A_N produce text output via standard autoregressive decoding

Latent thought length m: performance improves up to approximately m=80 steps, then diminishes (paper Section 6).
Recursion rounds r: r=1, 2, 3 tested experimentally. Paper primarily reports r=3 for headline results.

FOUR COLLABORATION PATTERNS

1. Sequential (Planner -> Critic -> Solver):
   Three agents with complementary roles process problem iteratively.
   Light version: Qwen3-1.7B (Planner), Llama3.2-1B (Critic), Qwen2.5-Math-1.5B (Solver)
   Scaled version: Gemma3-4B, Llama3.2-3B, Qwen3.5-4B
   Outer links bridge all three agent pairs.

2. Mixture (domain specialists + Summarizer):
   Math, Code, Science specialist agents run IN PARALLEL on the same input.
   Each specialist's H is transformed via outer RecursiveLink into Summarizer's embedding space.
   Summarizer aggregates parallel latent representations and produces final answer.
   Parallel agents do not communicate with each other; all converge to Summarizer.

3. Deliberation (Reflector + Tool-Caller):
   Reflector generates latent thoughts; outer link maps them into Tool-Caller's space.
   Tool-Caller invokes external tools (Python interpreter, search APIs) and its results loop back.
   Iterative refinement continues until convergence.
   NOTE: Tool-Caller must still produce text to call tools; only internal deliberation is latent.

4. Distillation (Expert large model + Learner small model):
   Expert (e.g., Qwen3.5-9B) generates latent thoughts transferred to Learner (Qwen3.5-4B).
   Learner generates response conditioned on Expert's latent knowledge via outer link.
   Enables knowledge distillation during inference without text-based summarization.

TRAINING REQUIREMENTS
Stage 1 - Inner loop training (per-agent alignment):
  Loss: L_in = 1 - cos(R_in(H), Emb_theta(y))
  Trains inner RecursiveLink to align latent thoughts H with the semantic distribution of ground-truth answer y in input embedding space.
  H is the latent thought sequence; Emb_theta(y) passes ground-truth text through the standard input embedding layer.
  Requires ground-truth labels (supervised).

Stage 2 - Outer loop training (end-to-end system):
  Loss: L_out = CE(S^(n)(S^(n-1)(...S^(1)(x))), y)
  CE = cross-entropy on final text prediction.
  Gradient backpropagation assigns each outer link a credit signal based on global contribution.
  Requires backpropagation THROUGH the agent chain (not through LLM parameters, which are frozen, but through the RecursiveLink adapters between agents).

What is trained: ONLY inner RecursiveLink (W1, W2 per agent) and outer RecursiveLink (W1, W2, W3 per agent pair).
What is frozen: ALL LLM backbone parameters across all agents.
Training data: curated from s1K (math), m1k (medical/scientific), OpenCodeReasoning (code), ARPO-SFT (agentic tool use).
Optimizer: AdamW, lr=5e-4, cosine scheduler, batch size=4.
This IS a train-time architectural change requiring custom checkpoints for the RecursiveLink weights. Off-the-shelf model weights are used as-is; the trained artifact is the set of RecursiveLink adapter weights.

Inputs:
x | text / token embeddings | [T] tokens | mandatory; the problem/query to solve
y | text (ground truth) | [T_y] tokens | mandatory during training; not needed at inference
r | int | scalar | recursion rounds; default 3; range [1, inf) but tested up to 3
m | int | scalar | latent thought length per agent per round; tested up to ~80; performance plateaus after m~80
N | int | scalar | number of agents; 3 in sequential/deliberation, 4 in mixture (3 specialists + summarizer), 2 in distillation

Outputs:
y_hat | text | decoded tokens | produced only by the FINAL agent in the FINAL recursion round
H_i | float tensor | [m, d_i] | latent thoughts from agent i (intermediate only; not exposed externally)
Final accuracy improvement: +8.3% average over strongest baseline (TextGrad) at r=3 (Table 3)

Hyperparameters:
r (recursion rounds) | default 3 | range 1-inf | strong sensitivity; each round adds ~+3% accuracy and 0.7x speedup gain; diminishing returns beyond r=3
m (latent thought length) | not fixed | tested up to 80 | performance peaks around m=80, then saturates
lr (learning rate) | 5e-4 | typical range 1e-5 to 1e-3 | standard for adapter training
batch_size | 4 | typical 4-32 | constrained by GPU memory (full latent chain forward pass)
RecursiveLink architecture | Residual+2-Layer (proposed) | ablated: 1-layer, residual+1-layer, 2-layer | ablation shows residual+2-layer best: 88.0% vs 85.6% (2-layer no residual) on MATH500

Critical invariants (testable assertions):
assert outer_link_output.shape[-1] == target_agent_embedding_dim, "Outer link must project to target agent hidden dim"
assert inner_link_output.shape == inner_link_input.shape, "Inner link is residual; output shape must match input"
assert text_output_count_per_round == 1 or (is_final_round and text_output_count_per_round == 1), "Only final agent in final round produces text"
assert all(param.requires_grad == False for param in llm_backbone.parameters()), "LLM backbones must be frozen during training"
assert latent_thought_length <= 80 or performance_gain_verified_empirically, "Paper reports diminishing returns beyond m~80"
assert len(recursive_link_params) << len(llm_params), "RecursiveLink is lightweight; should be orders of magnitude fewer params than LLMs"

Complexity (Appendix A.1):
Time per round: O(N * T * d^2) where N=agents, T=sequence length, d=embedding dim; latent pass is faster than text pass because no tokenization/detokenization overhead
Space: O(N * m * d_max) for storing latent thought tensors across the agent chain per round
Parallelism: Sequential/Distillation patterns are inherently sequential (agent chain); Mixture pattern supports parallel execution of domain specialists before Summarizer
Token savings source: intermediate rounds produce NO text tokens; only final agent in final round decodes; all intermediate agent outputs are latent tensors only
Speedup source: skipping text generation for intermediate agents eliminates vocabulary projection + sampling overhead per intermediate step

Benchmark results (r=3, Table 3, vs TextGrad best baseline):
MATH500: RecursiveMAS 88.0% vs TextGrad 85.8%
AIME2025: RecursiveMAS 86.7% vs TextGrad/Recursive-TextMAS 73.3%
AIME2026: RecursiveMAS 86.7% vs TextGrad 76.7%
GPQA-Diamond: RecursiveMAS 66.2% vs TextGrad 62.5%
LiveCodeBench: RecursiveMAS 42.9% vs TextGrad 39.8%
MedQA: RecursiveMAS 79.3% vs TextGrad 77.2%
Average improvement over strongest baseline: +8.3%

Efficiency results (Section 5.4, vs Recursive-TextMAS token baseline):
r=1: 1.2x speedup, 34.6% token reduction, +3.4% accuracy
r=2: 1.9x speedup, 65.5% token reduction, +6.0% accuracy
r=3: 2.4x speedup, 75.6% token reduction, +7.2% accuracy (these are averages across benchmarks)
Token reduction measured as: total system tokens vs Recursive-TextMAS (text-based version of same recursive structure)

Baselines compared (Appendix B.2):
Single Agent w/ LoRA: final agent only, LoRA-tuned
Single Agent w/ Full-SFT: final agent only, full fine-tune
Mixture-of-Agents (MoA): layered aggregation via text
TextGrad: natural-language feedback propagation (text-based optimization)
LoopLM: single-model latent recurrence (not multi-agent)
Recursive-TextMAS: same recursive MAS structure but text communication (direct ablation baseline)

Failure modes:
Performance gain diminishes after m~80 latent thought steps; longer latent sequences do not help and may hurt
No dedicated limitations section; theoretical analysis (Appendix A.2) assumes specific initialization and entropy thresholds that may not hold for all model families
Deliberation pattern requires tool-calling which forces some text generation; latent-only claim is partial for this pattern
Knowledge distillation direction is fixed (Expert->Learner); reverse direction not studied
Training data curation is pattern-specific; generalizing to a new collaboration pattern requires new training data

Implementation pitfalls (do NOT do this):
Do NOT attempt to use RecursiveMAS with API-only closed models (GPT-4o, Claude, Gemini via API); it requires hidden state tensors, not text
Do NOT decode intermediate agent outputs to text and re-encode; this defeats the efficiency and breaks the training objective
Do NOT skip Stage 1 (inner loop) training and go straight to Stage 2; inner links must first align latent distribution to input embedding space
Do NOT share RecursiveLink weights across all agent pairs if using heterogeneous models (different dims); outer link W3 is pair-specific
Do NOT measure token savings against single-agent text baseline; the paper measures against Recursive-TextMAS (same structure, text comms)
Do NOT use BatchSpanProcessor or any async decode in the agent loop; each agent's forward pass must complete before latent output is passed to the next
Do NOT treat the m latent thought tokens as equivalent to m output tokens; they are hidden state vectors at each GENERATION STEP, not final tokens
Do NOT expect RecursiveMAS to work zero-shot; RecursiveLink modules require training (2-stage supervised) on task-relevant data before any benefit appears
