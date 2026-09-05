Algorithm: AutoCompressor (Adapting Language Models to Compress Contexts)
ArXiv: 2305.14788
Paper: Adapting Language Models to Compress Contexts
Authors: Alexis Chevalier, Alexander Wettig, Anirudh Ajith, Danqi Chen (Princeton NLP)
Venue: EMNLP 2023 / NeurIPS 2023 context-compression track
Note: Task description called this "RECOMP" but 2305.14788 is AutoCompressor, NOT RECOMP. RECOMP is arXiv:2310.04408.

COMPRESSION CHARACTERIZATION
Lossy: YES. Summary vectors lose information; "summary vectors ignore some useful information that is accessible via full attention" (paper §4.3). Full attention outperforms on longer sequences.
LLM call required to compress: YES. Encoding requires a forward pass through the adapted LM to generate summary vectors. Compression is NOT a deterministic text transform.
Compression axis: token count (40:1 ratio achieved on 8K tokens with k=50 summary vectors per 2048-token segment); information loss is a measured degradation, not zero.
Deterministic: NO. Soft prompt generation is a neural network forward pass; results depend on model weights.

Inputs:
context_segments | List[str] | variable length | mandatory; documents split into segments of 1024-2048 tokens each
summary_tokens_from_prior | FloatTensor | [k, d_model] | optional; accumulated summary vectors from previous segments (empty for first segment)
k | int | scalar | mandatory; number of summary tokens to generate per segment (default 50)
special_summary_tokens | token embeddings | [k, d_model] | mandatory; learned <Sum>_i tokens appended to each segment

Outputs:
summary_vectors | FloatTensor | [k, d_model] | soft prompt vectors; fed as prefix to decoder for downstream tasks
accumulated_summaries | FloatTensor | [n_segments * k, d_model] | stacked summary vectors from all previous segments
perplexity_reduction | float | scalar | measured gain vs baseline; not invariant

Hyperparameters:
k (summary_tokens) | 50 | tested 20, 50, 70, 100 | more k helps up to ~70; authors note diminishing returns beyond 100; OPT uses 50
segment_length | 1024-2048 tokens | randomized per paper | randomization improves robustness; fixed segments degrade OOD
stop_gradient_interval | every 2 compression steps | required | prevents gradient explosion across accumulation; skipping breaks training
learning_rate | 2e-5 (OPT), 4e-4 (Llama-2 LoRA) | not tuned further by authors | LoRA rank not reported for Llama-2 variant
training_tokens | 2B (OPT-2.7B), 15B (Llama-2-7B) | authors note "extensive" pretraining required | undertraining causes zero-shot regression

Critical invariants (testable assertions):
assert summary_vectors.shape == (k, model.d_model), "summary vector count must equal k"
assert len(accumulated_summaries) == n_segments * k, "one summary block per segment"
assert compression_ratio == original_tokens / k, f"ratio must equal segment_len/k"
# from paper Table 3: Llama-2 with full attention beats AutoCompressor beyond 8K tokens
assert full_attention_perplexity <= autocompressor_perplexity + epsilon, "full attention dominates on long seqs"
# stop-gradient is structural: each segment encoded independently except via summary prefix
assert grad_flows_blocked_across_segment_boundary, "stop-gradient required between accumulation steps"

Complexity:
Time: O(n_segments * segment_length^2) for encoding (quadratic self-attention per segment); O(k^2) for decoding prefix (k << segment_length, so fast)
Space: O(n_segments * k * d_model) for accumulated summaries; quadratic in segments if k grows
Parallelism: segments processed sequentially (causal dependency via accumulated summaries); CANNOT parallelize encoding across segments

Failure modes:
Summary accumulation still O(n^2) in number of segments — does not achieve true O(n) scaling
Extended full attention outperforms on longer sequences (Table 3): AutoCompressor is NOT universally better
Llama-2 fine-tuned model shows "substantially worse zero-shot accuracy on some tasks" — instruction-following regresses
Out-of-domain generalization slightly weaker than in-domain (measured on BookCorpus vs training corpus)
Increasing k beyond ~100 does not reliably improve results; paper notes non-monotonic behavior
With LoRA on Llama-2: only tested up to 7B; scaling behavior to 13B+ unknown

Implementation pitfalls (do NOT do this):
Do NOT skip stop-gradient between segment accumulation steps — training diverges
Do NOT use fixed segment lengths without randomization — reduces out-of-domain robustness
Do NOT treat summary vectors as lossless — they cannot reconstruct original context verbatim
Do NOT use as a drop-in replacement for full attention in accuracy-critical paths without measuring degradation
Do NOT confuse with RECOMP (2310.04408) which is extractive/abstractive retrieval compression, not soft-prompt compression
Do NOT assume cross-model portability — summary vectors are tied to the specific model that generated them
Do NOT compress via a different model than the one consuming the summary vectors

Benchmark results (verified):
Language modeling OPT-2.7B, in-domain (8K context): perplexity 5.93 (AutoCompressor) vs 6.28 (no compression baseline)
Language modeling OPT-2.7B, out-of-domain: 8.10 vs 8.53 baseline
In-context learning Llama-2 (11 tasks): summary vectors outperform 150-token plain text on 8/11 tasks; outperform 750-token text on 8/11 tasks
Retrieval-augmented LM: 1.5x perplexity gain vs plain-text passages at equal sequence lengths; top-10 retrieval NQ: 3.74% gain

ELAI fit:
AutoCompressor COLLIDES with ELAI hard rules. Compression requires a full LM forward pass (rule (a): deterministic reversible transforms only, no LLM-resummarization banned). The summary vectors are model-specific soft prompts that cannot be passed across models or decoded back to text (rule (c): cross-model handoff must carry a typed belief-state, not raw history — summary vectors are neither). The append-only journal (rule (b)) would need to store either the raw text (no compression benefit) or the opaque float tensors (not a readable belief-state). AutoCompressor is architecturally incompatible with all three ELAI constraints unless ELAI adopts a single fixed encoder-decoder model and abandons cross-model handoffs entirely.
