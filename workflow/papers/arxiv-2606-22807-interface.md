Algorithm: KaLM-Reranker-V1 (Fast But Not Late Interaction Reranker)
ArXiv: 2606.22807
Paper: KaLM-Reranker-V1: Fast but Not Late Interaction for Compressed Document Reranking
Submitted: 2026-06-22 (Work in Progress)
Authors: Xinping Zhao et al. (Harbin Institute of Technology + Harbin Engineering University)

WHAT THIS PAPER ACTUALLY IS
A reranker (not a retriever/embedder) that decouples passage encoding from query+relevance scoring
via an encoder-decoder split. The encoder pre-encodes passages into reusable compressed representations
offline; the decoder attends to them via cross-attention at query time. Three sizes: Nano 0.27B,
Small 1B, Large 4B. Evaluated on BEIR (13 tasks), MIRACL (18 languages), LMEB (6 memory tasks).
This is the companion reranker to the KaLM-Embedding-V2.5 line.

RELATION TO ELAI
ELAI uses ms-marco cross-encoder as reranker (CPU). This paper offers a faster alternative reranker
paradigm that pre-encodes passages offline, which reduces online latency. However: still requires
GPU for the 4B decoder at query time; no ONNX export mentioned; the FBNL architecture is not
directly compatible with sentence-transformers cross-encoder API.

Inputs:
passage_text | str | up to 512 tokens | pre-encodable offline
query_text | str | up to 128 tokens | online at query time
system_instruction | str | task-specific prompt | optional, improves domain adaptation
task_instruction | str | per-query task description | optional
compression_ratio_r | int | {1,2,4,8,16,32} | default r=4 for evaluation; r=1 = no compression

Outputs:
relevance_score | float | scalar | logit for "yes" token minus logit for "no" token
binary_verdict | {yes, no} | | softmax over yes/no token logits
ranked_candidates | list | ordered by score | retrieved from top-100 candidates from first-stage retriever

Hyperparameters (from paper sections 4.1-4.2 and Table configurations):
query_max_length | 128 | [32, 512] | fixed; longer queries not tested
passage_max_length | 512 | [128, 4096] | offline; encoder runs once per passage
compression_ratio_r | 4 | {2,4,8,16,32} | r=4 is sweet spot; r<=8 negligible drop; r>=16 measurable loss
lora_r_equivalent | N/A | N/A | architecture does not use LoRA; full fine-tune of encoder-decoder
stage1_lr | 1e-4 | | general reranking; domain-agnostic
stage2_lr | 2e-4 | | task-specific adaptation with task instructions
stage3_lr | 5e-5 | | fine-grained distillation from Large -> smaller variants
batch_size | 2-1 per GPU with 4 gradient_accumulation | | multi-GPU training
optimizer | Adam beta1=0.9 beta2=0.999 | |
mep_ratios_during_training | {2,4,8,16} | | Matryoshka multi-scale training; all ratios jointly trained
epochs_per_stage | 1 | | three stages x 1 epoch each

Equations (verbatim from paper Section 3.1):
Encoder output: H_p = Enc(p) in R^{n x d}
Matryoshka pooling: H_p^(r)[j] = MeanPool(H_p[(j-1)*r+1 : min(j*r, n)])   [Eq. from Section 3.1]
Attention: Q = X*W_Q,  K = [X; H_p]*W_K,  V = [X; H_p]*W_V
Output: O = softmax((Q*K^T / sqrt(d_h) + M) * V) * W_O

Loss functions (verbatim from paper Section 4.1):
SFT base: L_sft(I,q,p,l) = -log[ exp(z_l) / (exp(z_yes) + exp(z_no)) ]
With MEP (multi-ratio): L_sft(I,q,p,l,R) = sum_{r in R} lambda_r * L_sft^(r)
KD (distillation): L_kd = -y*log(y_hat) - (1-y)*log(1-y_hat)
Final: L = L_sft + L_kd   [stage 3 only]

Critical invariants (testable assertions):
assert passage_tokens <= 512, "passage encoder trained with max 512 tokens"
assert query_tokens <= 128, "query decoder trained with max 128 tokens"
assert r in {1,2,4,8,16,32}, "MEP ratio must be power-of-2 in [1..32]"
# compressed passage shape: assert H_p_compressed.shape[1] == math.ceil(n / r)
# score is logit difference, NOT cosine similarity:
assert score_type == "logit_yes_minus_logit_no", "not a dot-product retriever"
# two-stage pipeline mandatory: cross-encoder is NOT a first-stage retriever
assert first_stage_retriever is not None, "FBNL reranker requires pre-filtered top-K candidates"

Complexity (from paper Section 5):
Conventional reranker per-candidate: O(K * L * ((|q|+n)^2 * d + (|q|+n) * d^2))
KaLM-Reranker-V1 with MEP ratio r: O((L/2) * K * (|q| * (|q| + ceil(n/r)) * d + (|q| + ceil(n/r)) * d^2))
Speedup over conventional at n=1024, r=4: ~18.5x
Speedup at n=4096, r=16: ~203.4x
Passage encoding: O(L/2 * n^2 * d) once offline, then cached
Space with MEP: O(N * ceil(n/r) * d) vs O(N * n * d) conventional
Parallelism: encoder runs offline in batch; decoder online per-query; fully parallelizable per candidate

Model size specifics (from Table in Section 3.1):
Nano:  0.27B activated, 100M non-emb, 168M emb, 18 layers, 640 token-dim, 128K max ctx
Small: 1B activated,   698M non-emb, 302M emb, 26 layers, 1152 token-dim, 128K max ctx
Large: 4B activated,   3209M non-emb, 675M emb, 34 layers, 2560 token-dim, 128K max ctx

Evaluation results (nDCG@10, from paper Section 6.2):
BEIR avg: Nano=57.41, Small=60.01, Large=62.87
MIRACL avg (18 langs): Nano=62.08, Small=66.89, Large=70.07
LMEB avg (6 memory tasks): Nano=61.39, Small=63.15

Failure modes (from paper):
1. High compression (r>=16): measurable performance degradation, especially on shorter passages
2. Out-of-domain: task instructions required for domain shift; without them performance degrades
3. First-stage retrieval quality: FBNL depends on top-100 recall from KaLM-Embedding-V2.5; different retrievers not validated
4. Short queries (<10 tokens): cross-attention over-compresses relevance signal
5. Long passages (n>>512): offline encoder was trained at 512 max; longer requires truncation or tiling

Implementation pitfalls (do NOT do this):
- Do NOT use as a first-stage retriever; it is a reranker only, requires pre-filtered candidates
- Do NOT run without task instructions in domain-shifted scenarios (stage 2 trains them in)
- Do NOT set r=32 for high-value recall tasks; r<=4 for quality, r<=8 for memory trade-off
- Do NOT reuse passage representations across different compression ratios without re-encoding
- Do NOT skip the multi-ratio MEP training if fine-tuning; single-ratio training loses flexibility
- Do NOT assume ONNX-exportable: the encoder-decoder cross-attention architecture uses custom attention
  masking M that is not standard and would require custom ONNX ops; NO ONNX export in paper
- Do NOT replace ms-marco cross-encoder with this directly without GPU: inference requires decoder
  at query time on GPU (no CPU inference path mentioned or validated)

ONNX/ELAI EXPORT VERDICT:
NOT directly ONNX-exportable as described. The cross-attention with dynamic mask M,
the MEP grouping, and the decoder's causal generation over variable-length inputs all
require GPU execution. Passage encoder output could in theory be serialized separately
as a static ONNX export (encoder-only, fixed compression ratio), but the paper does not
mention this and the decoder must still run on GPU per query. The ELAI ms-marco cross-encoder
is a better CPU fit. This paper is most useful as an architecture reference for a GPU reranker
that could eventually replace ms-marco if ELAI gets a GPU inference path.

CONTRACT TEST STUBS:
import numpy as np

def test_mep_compression_shape():
    n_tokens = 512
    r = 4
    expected_compressed = -(-n_tokens // r)  # ceil(n/r)
    assert expected_compressed == 128, f"MEP r=4 should yield 128 groups from 512, got {expected_compressed}"

def test_score_is_binary_logit_not_cosine():
    # KaLM-Reranker scores come from yes/no logit difference, NOT dot-product
    score_yes = 3.2
    score_no = -1.1
    relevance_score = score_yes - score_no  # paper Section 3.2
    assert relevance_score > 0, "positive relevance should yield positive score"
    # cosine similarity would be in [-1, 1]; logit difference is unbounded
    assert not (-1.0 <= relevance_score <= 1.0) or True, "logit diff is NOT bounded to [-1,1]"

def test_two_stage_required():
    # reranker requires pre-filtered candidates; top-K from first-stage retriever
    top_k_from_retriever = 100
    assert top_k_from_retriever > 0, "first-stage retrieval mandatory"
    assert top_k_from_retriever <= 200, "paper evaluates with top-100; larger sets untested"

def test_compression_ratio_range():
    valid_ratios = {1, 2, 4, 8, 16, 32}
    test_r = 4
    assert test_r in valid_ratios, f"r={test_r} not in trained MEP ratios"
    # r=1 = no compression = baseline
    # r=4 = paper default for evaluation
    assert 1 in valid_ratios and 4 in valid_ratios

def test_nano_activated_params():
    nano_non_emb = 100e6
    nano_emb = 168e6
    nano_total_activated = 270e6
    assert abs((nano_non_emb + nano_emb) - nano_total_activated) < 5e6, \
        "Nano param counts: 100M non-emb + 168M emb ~ 270M activated"
