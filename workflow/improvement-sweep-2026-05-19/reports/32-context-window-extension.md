# Context window extension for ELAI

## TL;DR
- Qwen2.5-Coder-7B is natively trained to 32,768 tokens; ELAI's 25K ceiling is already inside the native window — no extension active. Going to 32K is free (no config changes, no quality loss).
- Pushing past 32K requires YaRN (or DCA). YaRN-static (the only mode vLLM ships) degrades short prompts and costs ~15-20% quality at factor=4 (32K to 128K) on Qwen3-32B; F1 drops catastrophically at ~40-50% of the max length (Qwen2.5-7B hits the cliff near 50-65K of a claimed 128K window).
- The Qwen2.5-7B-Instruct-1M variant uses Dual Chunk Attention (DCA), a length-extrapolation method that does NOT alter behavior on short sequences and achieves near-perfect passkey at 1M. DCA is a drop-in 7B replacement and the strongest "more context" lever available on consumer hardware. Tradeoff: 120GB VRAM total claimed for 1M; 256K is feasible on 24GB with chunked prefill.
- For ELAI's actual use cases (code refactor, multi-file context, deep research streams) RAG + good chunking continues to win on cost; long-context only wins on global summarization and when retrieval would shred semantic boundaries. EMNLP 2024 Self-Route cuts cost 39-65% vs pure long-context with comparable quality. ELAI is already on the right side of this tradeoff.
- Concrete recommendation: (a) raise hard ceiling from 25K to 32K (no YaRN, no risk); (b) add an opt-in "deep mode" that swaps the sidecar model to Qwen2.5-7B-Instruct-1M with 64K cap on a 24GB box, gated by SessionDag handoff signal; (c) ship Found-in-the-Middle attention calibration as a free win (up to +15pp RAG recall, training-free); (d) do NOT enable YaRN on the default path.

## Key sources
| URL | what | takeaway |
|---|---|---|
| arxiv.org/abs/2309.00071 | YaRN paper | Combines NTK-by-parts + attention temperature; ~64x extension with <400 fine-tune steps; standard for Qwen |
| arxiv.org/abs/2402.13753 | LongRoPE (Microsoft, ICML 2024) | 2048K context via non-uniform PI search + progressive extension; 8x extension non-fine-tuned; integrated into Phi-3 |
| arxiv.org/abs/2309.17453 | StreamingLLM | Attention sinks keep generation stable to 4M tokens; but context window NOT actually extended — middle is discarded |
| arxiv.org/abs/2404.07143 | Infini-attention (Google) | Compressive memory + local attention; 114x less memory than Transformer-XL; 1M passkey works; 8B-scale only |
| arxiv.org/abs/2307.03172 | Lost in the Middle (TACL 2024) | 30%+ accuracy drop when answer moves from position 1 to 10 in 20-doc context; U-shaped attention |
| arxiv.org/abs/2406.16008 | Found in the Middle | Training-free calibration subtracts positional prior; +15pp RAG accuracy; Recall@3 20.5% to 68.3% |
| arxiv.org/abs/2407.01100 | PINE (ICLR 2025) | Mechanistic position-bias removal; eliminates "lost in the middle" for retrieval |
| arxiv.org/abs/2407.16833 | RAG-vs-LC + Self-Route (EMNLP 2024) | 63% of queries agree between RAG and LC; Self-Route cuts cost 39-65% with LC-comparable quality |
| arxiv.org/abs/2501.15383 | Qwen2.5-1M Technical Report | DCA extrapolation; 7B-1M variant; near-perfect 1M passkey; 120GB VRAM for 1M, chunked prefill saves 96.7% activation memory |
| arxiv.org/abs/2404.06654 | RULER (NVIDIA) | Real effective context << claimed; only ~half of 32K-claimed models actually pass 32K threshold |
| arxiv.org/abs/2407.11963 | NeedleBench | Bilingual long-context eval at 4K/32K/128K/200K/1M; scaling window benefits retrieval but not multi-hop reasoning |
| arxiv.org/abs/2412.15204 | LongBench v2 | 8K-2M contexts, multiple choice; Falcon3-Mamba-Inst-7B SOTA 30.8; Qwen3-4B-Instruct-2507 45.7 beats Qwen3-8B 44.3 |
| arxiv.org/abs/2310.01889 | Ring Attention (ICLR 2024) | Blockwise + ring communication; near-infinite context across multi-device; useless on single 12GB GPU |
| arxiv.org/abs/2502.12110 | A-MEM (NeurIPS 2025) | Zettelkasten-style agentic memory; atomic notes + dynamic linking; alternative to long context |
| huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct | Official model card | Native 32K, YaRN to 131K; warning: vLLM static YaRN hurts short-text perf; only enable when needed |
| qwenlm.github.io/blog/qwen2.5-1m/ | Qwen 1M blog | 32K-trained model + DCA reaches 1M passkey near-perfectly without retraining |
| arxiv.org/pdf/2601.15300 | Intelligence Degradation in Long-Context LLMs | Qwen2.5-7B critical threshold at 40-50% of 128K; F1 drops 0.55 to 0.30 (45.5% degradation) |
| amaarora.github.io/posts/2025-09-21-rope-context-extension.html | RoPE timeline deep-dive | Comparative table PI/NTK/Dynamic-NTK/YaRN/LongRoPE on extension ratio + fine-tune cost + short-context loss |
| arxiv.org/abs/2406.06025 | RepoQA | Code-specific long-context eval (function localization across repos); 7B class struggles past 32K |
| letta.com/blog/agent-memory | Letta/MemGPT | Three-tier memory (core/recall/archival); maintains task context 500+ interactions where RAG fragments at 50 |

## Extension technique comparison
| Technique | Tested max | Quality at 2x trained | Works on Qwen2.5-Coder-7B | KV cost | Citation |
|---|---|---|---|---|---|
| Position Interpolation (Su/Meta) | 32K from 4K | Mild PPL increase | Yes (predecessor of YaRN) | Linear in seq len | arxiv 2306.15595 / amaarora 2025 |
| NTK-Aware (bloc97) | 8x | Minimal PPL loss; no fine-tune | Yes; legacy mode | Linear | EleutherAI YaRN blog |
| Dynamic NTK | 8x+ | Near-baseline; no short-ctx loss | Yes; not in vLLM static path | Linear | EleutherAI |
| YaRN | 64x; Qwen ships 4x to 131K | 15-20% drop at factor=4 on Qwen3-32B | Yes (official Qwen config) | Linear; ~4-8GB KV at 32K NF4 | arxiv 2309.00071; vllm issue 18728 |
| LongRoPE | 2048K on LLaMA2/Mistral | Recovered at 8K w/ readjust | Not officially tested on Qwen-Coder; would require ~1K fine-tune steps | Linear | arxiv 2402.13753 |
| Dual Chunk Attention (Qwen 1M) | 1M from 32K training | No short-seq impact (extrapolation only) | Yes via Qwen2.5-7B-Instruct-1M | Sparse attention + chunked prefill; 96.7% activation reduction | arxiv 2501.15383 |
| StreamingLLM | 4M generation | Context window NOT extended; middle discarded | Yes (any RoPE LLM); wrong tool for ELAI | O(window) constant | arxiv 2309.17453 |
| Infini-attention | 1M passkey; 500K book summary | Compressive memory bounded | Not ported to Qwen2.5; Google internal | Bounded constant | arxiv 2404.07143 |
| Ring Attention | Near-infinite, multi-device | Same quality, sharded | Requires multi-GPU; ELAI is single 12GB | Sharded across N devices | arxiv 2310.01889 |

## Lost-in-the-middle reality
- Liu et al. 2024 (TACL): 30%+ accuracy drop when the answer document moves from position 1 to position 10 in a 20-document context — for ALL tested LLMs including GPT-3.5 and Claude-1. U-shaped attention curve is the mechanism.
- RoPE's frequency decay makes the bias structural: attention falls off with distance, so middle tokens lose to both ends. (Morph LLM 2026 explainer; Found-in-the-Middle arxiv 2406.16008.)
- Confirmed at 7B scale: Qwen2.5-7B F1 drops 0.55 to 0.30 at 40-50% of 128K window (arxiv 2601.15300). For ELAI: even if we extended to 64K via YaRN, the productive middle zone shrinks to ~25K of high-quality attention plus ~10K each end.
- Fix exists and is essentially free: Found-in-the-Middle calibration (Hsieh et al. ACL 2024) — pass a dummy doc, subtract positional prior, training-free, +15pp RAG accuracy. PINE (ICLR 2025, arxiv 2407.01100) goes further with mechanistic removal but requires modifying inference.

## Long-context-vs-RAG decision
For ELAI's three primary load profiles (code refactor, multi-file context, deep research streams):

| Workload | Winner | Citation | Why |
|---|---|---|---|
| Multi-file refactor (5-15 files, ~20K tokens total) | Long context (32K native) | RepoQA arxiv 2406.06025 | Cross-file reasoning benefits from co-attention; below YaRN cliff |
| Deep research / FlashResearch stream | Hybrid (RAG + summarize tail) | Self-Route EMNLP 2024 | 63% query overlap with LC; routing saves 39-65% cost |
| Whole-repo understanding | RAG with semantic chunking + symbol graph | Liner review of arxiv 2407.16833 | Long-context fails on global summarization at >100K for 7B; A-MEM/Letta beat fragmenting RAG past 50 interactions |
| Single-function deep edit | Native 32K is overkill; current 25K is enough | RULER arxiv 2404.06654 | 7B-class effective context < claimed; only half of "32K" models pass at 32K |

Bottom line: ELAI's stateless-per-call + knowledge engine + baby-step decomposition design is empirically aligned with what 2024-2025 research recommends. Long context wins only on (a) global summarization and (b) tasks where retrieval would chop semantic units. Both are addressable via knowledge engine improvements, not context window extension.

## Best techniques to adopt
Ranked by ROI for ELAI:

1. **Raise hard ceiling 25K to 32K** (native, no YaRN). Grounding: huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct ("native context length 32,768"). Integration hook: `ELAI_LOCAL_MAX_CONTEXT=32000` env in workflow/remote-setup/REMOTE-SIDECAR.md; SessionDag handoff threshold (85%) becomes 27,200 tokens. KV cache impact: +~700MB at NF4 (linear-in-length formula, KV Cache Memory Calculation guide).
2. **Add Found-in-the-Middle calibration to the sidecar inference path**. Grounding: arxiv 2406.16008, training-free, +15pp RAG recall. Integration hook: monkey-patch logits processor in `python/elai_inference/server.py`; pass-through dummy doc on every >8K request. Cost: 1 extra forward pass per long request.
3. **Ship "deep mode" Qwen2.5-7B-Instruct-1M variant with DCA, 64K cap**. Grounding: arxiv 2501.15383; DCA does not alter short-seq behavior; near-perfect 1M passkey. Integration hook: a second sidecar model entry, gated by orchestrator policy (`ELAI_DEEP_MODE=1` triggered by SessionDag at 85%-of-32K). 24GB friend's-box can hold 64K with chunked prefill; do not enable on 12GB local.
4. **Add Self-Route hybrid policy to SessionDag handoff**. Grounding: EMNLP 2024 arxiv 2407.16833. Integration hook: before triggering handoff, ask the local model "do you have enough context to answer?" — if yes, skip deep-mode escalation. Saves cloud calls.
5. **Adopt A-MEM-style Zettelkasten note graph for cross-session memory**, parallel to existing RAG. Grounding: arxiv 2502.12110 (NeurIPS 2025). Integration hook: knowledge engine adds `mcp_memorize` write that creates atomic note + auto-links to related concepts; reading is via graph traversal not vector search. Beats RAG on multi-hop cross-session continuity per the Letta benchmark.
6. **Do NOT enable YaRN on default path**. Grounding: vllm issue 18728 (15-20% drop on Qwen3-32B at factor=4); Qwen2.5-7B critical degradation 0.55 to 0.30 F1 past 50K (arxiv 2601.15300). Only enable if user explicitly opts into legacy-128K mode for one-shot tasks.

## When 25K is enough
- RULER 2024 (arxiv 2404.06654): only ~half of models claiming 32K actually clear the 85% quality bar at 32K. ELAI staying at 25-32K is in the empirically reliable zone.
- RepoQA (arxiv 2406.06025): 7B-class models struggle past 32K on cross-function reasoning; a baby-step + knowledge-engine architecture already keeps per-call context well under that.
- LongBench v2 (arxiv 2412.15204): Falcon3-Mamba-Inst-7B SOTA at 30.8 (out of 100); Qwen3-4B at 45.7 — these are absolute scores, not percentages — i.e. 7B-class long-context understanding is genuinely poor regardless of window size. More tokens does not equal more comprehension.
- Liner review of arxiv 2407.16833: 63% of queries get identical answers from RAG and long-context. The other 37% split mostly on global-summarization tasks, which ELAI handles via decomposition.
- Letta benchmarking (letta.com/blog/benchmarking-ai-agent-memory 2024): "long-context wins on sequential learning and summarization; RAG wins on retrieval; no single paradigm covers all competencies" — directly endorses ELAI's hybrid stance.

## Anti-patterns
- **Cranking YaRN factor=4 on the default sidecar path**. 15-20% quality drop on short prompts (vllm issue 18728), F1 cliff at 40-50% of advertised window (arxiv 2601.15300). The "free" 128K is not free.
- **Treating StreamingLLM as a context-window extension.** It is NOT — it discards middle tokens to keep generation stable indefinitely. Wrong abstraction for ELAI's use case (arxiv 2309.17453 + GitHub README).
- **Pursuing Ring Attention on a single GPU.** Requires inter-device communication; collapses to vanilla attention on one GPU (arxiv 2310.01889).
- **Trusting "1M context" marketing claims at face value.** Llama 4 Scout claims 10M but the practical performance gap is "more nuanced" (composio.dev 2026 review); Mistral-7B-Instruct-v0.2 claims 32K but RULER score at 128K is 13.8% (RULER github).
- **Adding long context to fix multi-hop reasoning.** NeedleBench arxiv 2407.11963: scaling the window benefits retrieval; multi-hop / compositional reasoning is unchanged. Reasoning bottleneck is model size, not context.
- **Q4 KV cache quantization at >32K**. KV-cache quantization to Q4 saves VRAM but "degrades quality, especially on long contexts where attention precision matters most" (insiderllm.com KV cache optimization guide). Keep KV at FP8 minimum past 16K.

## Open questions
1. Measure: does enabling DCA via Qwen2.5-7B-Instruct-1M improve RepoQA scores at ELAI's actual repo sizes (~25-60K tokens) by >5pp vs current 25K cap? If no, skip deep-mode entirely.
2. Measure: what is Found-in-the-Middle's measured uplift on ELAI's knowledge-engine retrieval at 16K and 24K context? Need an internal eval — papers report +15pp on NaturalQuestions, ELAI workload is code+plans not QA.
3. Measure: on the friend's RTX 3090 24GB, what is the actual KV-cache footprint of Qwen2.5-7B-Instruct-1M-NF4 at 64K with chunked prefill — must fit under 18GB (leaving room for activations + SearXNG sidecar)?
4. Decide: should SessionDag handoff (currently 85% of 25K) escalate to a cloud Sonnet call OR to a local deep-mode 64K Qwen-1M call? Need cost-vs-quality A/B.
5. Open: A-MEM atomic-note graph vs ELAI's existing knowledge engine + symbol graph — is there overlap or are they complementary? Worth a small spike before committing.
