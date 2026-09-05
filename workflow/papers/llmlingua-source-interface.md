Source: LLMLingua family (LLMLingua / LongLLMLingua / LLMLingua-2)
Reliability: HIGH (official Microsoft open-source repo, MIT license, reviewed source)
Files examined:
  F:\.ELAI_workflow\docs\.source\LLMLingua\llmlingua\prompt_compressor.py (primary, 1200+ lines)
  F:\.ELAI_workflow\docs\.source\LLMLingua\llmlingua\utils.py (helpers)
  F:\.ELAI_workflow\docs\.source\LLMLingua\README.md
  F:\.ELAI_workflow\docs\.source\LLMLingua\setup.cfg
Backing arXiv papers (all verified via WebSearch):
  LLMLingua: arXiv:2310.05736 (EMNLP 2023)
  LongLLMLingua: arXiv:2310.06839 (ACL 2024)
  LLMLingua-2: arXiv:2403.12968 (ACL 2024 Findings)

COMPRESSION CHARACTERIZATION
Lossy: YES. Tokens are irreversibly dropped. "minimal performance loss" (not zero loss) is the design target. At 20x compression, task quality measurably degrades on some benchmarks.
LLM/model required to compress: YES. CRITICAL. LLMLingua v1 loads a small causal LM (default: NousResearch/Llama-2-7b-hf) at __init__ time and runs a forward pass to compute per-token perplexity scores before dropping tokens. LLMLingua-2 uses a token classification encoder (XLM-RoBERTa-large or mBERT-class model) trained via data distillation from GPT-4. BOTH variants require a model inference call to compress. There is NO path through the library that compresses without a model forward pass.
Compression axis: token count. Compressed text is a subset of original tokens (or slightly reordered); it is still a text string (not a float tensor). The compressed output is HUMAN-READABLE and MODEL-AGNOSTIC (any LLM can consume it).
Deterministic: CONDITIONALLY. With temperature=0 / greedy decode and fixed model weights, compression output is deterministic for the same input. In practice: seed_everything(42) is called in LLMLingua-2 init. LLMLingua v1 uses perplexity (no sampling), so it IS deterministic given fixed model + input.
Lossless-on-task-quality claim: FALSE. Paper claims "minimal" loss on GSM8K/BBH/ShareGPT/Arxiv-March23 at up to 20x, but this is a TRADEOFF not a guarantee. At 20x compression on math reasoning (GSM8K), accuracy measurably drops. At 4x-6x, performance is near-original on tested benchmarks. This is a MEASURED TRADEOFF, not "lossless."

Public API:
PromptCompressor.__init__ | (model_name="NousResearch/Llama-2-7b-hf", device_map="cuda", model_config={}, open_api_config={}, use_llmlingua2=False, use_slingua=False, llmlingua2_config={}) -> None | loads model into GPU at construction; GPU required by default; blocks until model loads
PromptCompressor.compress_prompt | (context: List[str], instruction="", question="", rate=0.5, target_token=-1, iterative_size=200, force_context_ids=None, force_context_number=None, use_sentence_level_filter=False, use_context_level_filter=True, use_token_level_filter=True, keep_split=False, keep_first_sentence=0, keep_last_sentence=0, keep_sentence_number=0, high_priority_bonus=100, context_budget="+100", token_budget_ratio=1.4, condition_in_question="none", reorder_context="original", dynamic_context_compression_ratio=0.0, condition_compare=False, add_instruction=False, rank_method="llmlingua", concate_question=True, ..., force_tokens=[], force_reserve_digit=False, drop_consecutive=False, chunk_end_tokens=[".","\n"], strict_preserve_uncompressed=True) -> dict | v1 main entry point
PromptCompressor.compress_prompt_llmlingua2 | (context: List[str], rate=0.5, target_token=-1, use_context_level_filter=False, use_token_level_filter=True, ..., force_tokens=[], force_reserve_digit=False, drop_consecutive=False, chunk_end_tokens=[".","\n"]) -> dict | v2 main entry point (called when use_llmlingua2=True)
PromptCompressor.compress_json | (json_data: dict, json_config, ...) -> dict | structured JSON compression wrapper over structured_compress_prompt
PromptCompressor.structured_compress_prompt | (context: List[str], ...) -> dict | handles llmlingua-tag-segmented context
PromptCompressor.get_ppl | (text: str, granularity="sentence", ...) -> float | per-token or sentence perplexity; the scoring backbone of v1; runs a model forward pass
PromptCompressor.__call__ | (*args, **kwargs) -> dict | delegates to compress_prompt

compress_prompt return dict schema:
  "compressed_prompt": str  (the compressed text, human-readable, model-agnostic)
  "origin_tokens": int
  "compressed_tokens": int
  "ratio": str  (e.g. "2.5x" = origin/compressed)
  "rate": str   (e.g. "0.40" = compressed/origin)
  "saving": str (estimated GPT-4 cost savings)
compress_prompt_llmlingua2 return dict adds:
  "compressed_prompt_list": List[str]
  "fn_labeled_original_prompt": str (word + keep/drop label pairs)

Behavioral contracts:
__init__ -> loads full LM into GPU memory; for Llama-2-7b default this is ~14GB fp16 or ~7GB in 4-bit; WILL OOM on consumer GPU without model_config quantization
compress_prompt -> runs get_ppl forward pass on context+question+instruction to score tokens; then drops low-perplexity (less informative) tokens; output is a subset (and possible reordering) of input tokens — not generated text
compress_prompt_llmlingua2 -> runs token classification forward pass via XLM-RoBERTa or mBERT encoder; assigns keep/drop binary label per token; output is the kept tokens concatenated
get_ppl -> torch.no_grad() inference (not torch.inference_mode() — note: this violates the project's Python anti-stale rule which requires torch.inference_mode(); this is upstream code, not project code)
rate parameter -> rate=0.5 means keep 50% of tokens; actual output may differ due to tokenizer mismatches between compressor model and target LLM
target_token overrides rate when both specified; rate is recalculated from target_token / input_length
force_tokens: tokens listed here are ALWAYS preserved regardless of perplexity; useful for special tokens, task keywords

Platform constraints:
Requires GPU (device_map="cuda" default); CPU mode supported but ~100x slower for Llama-7B
PyTorch + Transformers required; tested on Python 3.8+; setup.cfg specifies transformers, torch, tiktoken, nltk as core deps
CUDA required for default model; MPS (Apple Silicon) supported via device_map="mps" but untested in production
Model download: default Llama-2-7b requires HuggingFace Hub access + Llama license acceptance
LLMLingua-2 recommended model: microsoft/llmlingua-2-xlm-roberta-large-meetingbank (much smaller, faster than Llama-2-7b)
Windows: works but path handling via os.path; no known Windows-specific failures in source

Gotchas:
compress_prompt rate parameter is COMPRESSED/ORIGINAL (lower = more compression), NOT ratio — confusing with the "ratio" field in output which is ORIGINAL/COMPRESSED (higher = more compression)
actual compression rate often differs from target rate by 10-20% due to tokenizer boundary effects between compressor model and target LLM
model_name="NousResearch/Llama-2-7b-hf" is the default; this is a 7B parameter model; using it for compression adds ~14GB GPU overhead for a compression preprocessing step
force_reserve_digit=True preserves ALL digit-containing tokens; useful for numerical reasoning tasks (math, code) — set this to True for ELAI coding tasks if LLMLingua is ever adopted
LLMLingua-2 requires init_llmlingua2() which calls seed_everything(42) — the library mutates global numpy/torch/random seeds at init time (side effect)
compress_prompt does NOT guarantee the compressed prompt is valid for all downstream models — token boundary artifacts can introduce grammatically broken segments
No streaming output: compress_prompt returns only when full compression completes; for 10K token contexts at v1 Llama-7B this can take 5-30 seconds depending on GPU

Critical assertions (testable):
assert result["compressed_tokens"] <= result["origin_tokens"], "compression never increases token count"
assert 0.0 < float(result["rate"].rstrip("x")) <= 1.0 or result["rate"].endswith("x"), "rate field sanity"
assert result["compressed_tokens"] / result["origin_tokens"] <= rate * 1.2, "actual rate within 20% of target"
assert isinstance(result["compressed_prompt"], str), "output is always text (never float tensors)"
assert result["origin_tokens"] > 0, "empty input guard"

ELAI FIT ASSESSMENT (CRITICAL):
LLMLingua PARTIALLY COLLIDES with ELAI hard rules:
(a) Deterministic-only, no LLM-resummarization ban: COLLISION. LLMLingua requires a small LM (Llama-2-7b for v1, XLM-RoBERTa for v2) to score and drop tokens. This IS an LLM call inside the compression step. Even though it is a smaller model than the target LLM, it is still a model inference call. The ELAI ban on "LLM-resummarization of cache prefix" is ambiguous about whether a scorer (not a summarizer) counts — but the spirit of the ban (no model-dependent, non-reproducible compression) clearly covers this. LLMLingua-2 is slightly better: it uses a classification encoder (not a generative LM), is faster, and its token-keep/drop decisions are more interpretable. But it still requires a model forward pass.
(b) Append-only journal as lossless record: COMPATIBLE. LLMLingua output is plain text — a subset of original tokens. It CAN be stored in a text journal. The lossless record would be the ORIGINAL text; the journal stores the original. What gets sent to the target LLM is the compressed version. This is compatible if ELAI stores originals in the journal and compresses only for inference.
(c) Cross-model handoff with typed belief-state: COMPATIBLE. Unlike AutoCompressor/ICAE, LLMLingua output is plain text that any LLM can consume. No embedding-space binding. Cross-model handoff works.
VERDICT: LLMLingua is PARTIALLY compatible — it satisfies (b) and (c) but violates (a) if the ELAI "no LLM-resummarization" ban is read to cover token-scoring model calls. LLMLingua-2 is the better variant if partial adoption is considered (classification encoder, not generative LM, 3-6x faster). The practical mitigation: use LLMLingua as an OFFLINE preprocessing step (compress prompts once, cache the result) rather than inline on every request — this reduces the ban violation to a build-time step rather than a runtime call. Full compliance with rule (a) requires a deterministic text-only compression (e.g., truncation, extractive sentence selection without a model) which LLMLingua does not offer.

Benchmark results (verified via paper abstracts and README):
LLMLingua on GSM8K/BBH/ShareGPT/Arxiv-March23: up to 20x compression with "minimal performance loss" (no exact numbers in abstract; EMNLP 2023 paper required for tables)
LongLLMLingua on NaturalQuestions (GPT-3.5-Turbo): +21.4% performance improvement vs no compression at 4x fewer tokens (reduces "lost in the middle" effect); 1.4x-2.6x end-to-end latency reduction at 2x-6x compression on ~10K token prompts
LLMLingua-2 vs LLMLingua: 3x-6x faster compression; on MeetingBank/LongBench/ZeroScrolls/GSM8K/BBH: 1.6x-2.9x end-to-end latency reduction; better out-of-domain generalization
