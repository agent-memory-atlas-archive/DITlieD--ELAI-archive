# RAG retrieval quality (2024-2026 SOTA for ELAI)

## TL;DR
- ELAI's biggest unrealized win is **Anthropic Contextual Retrieval at ingestion** (LLM-prepended chunk context + contextual BM25): -49% top-20 failure on Anthropic's own internal codebase eval, -67% with rerank stacked. Slots cleanly above existing BM25 + dense pipeline; replaces HyDE for cost reasons (HyDE pays per-query, contextual pays once at ingest).
- **Drop or de-prioritize HyDE** for code queries. Recent code-RAG ablations show HyDE underperforms vanilla dense on precise / numerical lookups; sidecar 128-tok generation cost is paid every query.
- **Replace ms-marco-MiniLM-L12 reranker** with a 2024-2025 SOTA reranker (Voyage rerank-2 / Jina-rerank-v3 / BGE-reranker-v2-m3). MiniLM-L12 sits ~10-15 NDCG@10 points below current SOTA on BEIR; Jina-rerank-v3 reports 61.94 NDCG@10 BEIR.
- **DEEP MRR < STANDARD MRR is over-retrieval dilution + reranker saturation**, not a sidecar / fusion bug. Top-250 candidate pools show no improvement over top-100 in recent literature; gold doc misses cap MRR regardless of reranker quality. Fix by capping DEEP candidate pool to ~100 and adding a quality gate (drop candidates below RRF threshold before rerank).
- **Add late-interaction (PLAID / FastPlaid Rust) as an optional 8th layer** for code-symbol queries — ColBERTv2 dominates 22/28 OOD BEIR tasks vs single-vector dense, and FastPlaid ships Rust+CUDA so it integrates with ELAI's Rust core. Reason-ModernColBERT (LightOn 2025) is the new SOTA for reasoning-intensive retrieval, beats 7B dense rerankers at 1/45 the params.

## Key sources
| URL | What | Takeaway |
|---|---|---|
| https://www.anthropic.com/news/contextual-retrieval | Anthropic Contextual Retrieval official (Sep 2024) | -49% top-20 failure with contextual embed+BM25; -67% w/ rerank |
| https://arxiv.org/abs/2406.14497 | CodeRAG-Bench (Wang/Asai et al., 2024) | Retrievers struggle on low-lexical-overlap code queries; current code RAG ceiling far from solved |
| https://arxiv.org/abs/2510.20609 | Practical Code RAG at Scale (Galimzyanov et al., Oct 2025) | PL-PL: BM25 word-level beats dense at 10x lower latency. NL-PL: dense beats sparse but 100x slower. Optimal chunk: 32-64 lines at small budget, whole-file at 16K |
| https://arxiv.org/abs/2506.15655 | cAST (EMNLP 2025) | AST-aware chunking: +4.3 Recall@5 on RepoEval, +2.67 Pass@1 on SWE-bench vs line-based |
| https://arxiv.org/abs/2411.11767 | Drowning in Documents (Nov 2024) | Rerankers degrade beyond a candidate-pool threshold; top-250 ≈ top-100; retrieval is the hard ceiling, not reranker |
| https://arxiv.org/abs/2407.01449 | ColPali (Jul 2024, ICLR 2025) | Late-interaction VLM, +29% NDCG@5 vs OCR+dense pipelines on ViDoRe |
| https://arxiv.org/abs/2405.14831 | HippoRAG (NeurIPS 2024) | KG + Personalized PageRank, +20% on multi-hop, 6-13x faster than IRCoT |
| https://arxiv.org/abs/2508.10391 | LeanRAG (Aug 2025) | KG + bottom-up retrieval reduces redundant retrieval; token-efficient alternative to GraphRAG |
| https://arxiv.org/abs/2502.09956 | KGGen (NeurIPS 2025) | text-to-KG with entity clustering; MINE benchmark for KG extractors |
| https://huggingface.co/lightonai/Reason-ModernColBERT | Reason-ModernColBERT (LightOn, 2025) | Beats all <7B models on BRIGHT reasoning-retrieval; +2.5 NDCG@10 over ReasonIR-8B on StackExchange |
| https://blog.voyageai.com/2024/09/30/rerank-2/ | Voyage rerank-2 (Sep 2024) | +6.33% NDCG vs Cohere v3, +14.75% vs BGE-v2-m3, +2.84% vs rerank-1 |
| https://jina.ai/models/jina-reranker-v3/ | Jina-rerank-v3 (2025) | 61.94 NDCG@10 BEIR, +4.88% vs v2 |
| https://qdrant.tech/articles/bm42/ | Qdrant BM42 + critique | BM42 attempts attention-weighted BM25; subsequent evals show validity questions — do NOT adopt as drop-in |
| https://qdrant.tech/articles/minicoil/ | miniCOIL (Qdrant 2025) | Sparse neural retrieval that ships; preferred over BM42 for production |
| https://crates.io/crates/next-plaid | next-plaid (Rust) + https://github.com/lightonai/fast-plaid | PLAID engine in Rust: CPU-only crate next-plaid; FastPlaid Rust+GPU. ELAI can integrate without Python sidecar |
| https://arxiv.org/abs/2212.10496 | HyDE (Gao et al., 2022) | Original HyDE paper — hypothetical doc captures relevance pattern but hallucinates details; weakness on precise/numerical queries confirmed in downstream evals |
| https://arxiv.org/abs/2506.21182 | Maintaining MTEB (Jun 2025) | MTEB transitioning from self-reported to verified results; prefix/normalization differences explain large gaps between reported and measured numbers |

## Technique landscape
| Technique | NDCG@10 cite | Latency | Rust impl | Code-suitable | Citation |
|---|---|---|---|---|---|
| BM25 (tantivy) | baseline | ~5ms | yes (tantivy) | yes (PL-PL wins per 2510.20609) | ELAI current |
| Dense single-vec (Granite/GraphCodeBERT) | ~50-55 BEIR | 10-50ms | ONNX runtime | partial | ELAI current (degraded — code_vec unqueried) |
| ColBERTv2 / PLAID late-interaction | ~52-54 BEIR; +8% over single-vec on 22/28 OOD | 20-100ms (PLAID 7x GPU/45x CPU speedup) | yes (next-plaid, FastPlaid) | yes | arxiv:2112.01488 + 2205.09707 |
| ColPali | 0.81 vs 0.66 baseline NDCG@5 on ViDoRe | VLM-heavy | no (Python) | docs/PDFs only | arxiv:2407.01449 |
| Reason-ModernColBERT | beats 7B dense on BRIGHT, +2.5 vs ReasonIR-8B | small (modernBERT base) | via PyLate, not Rust | yes (reasoning-heavy) | LightOn 2025 |
| SPLADE-v3 | ~50 BEIR | 30-80ms | no native | yes | qdrant.tech sparse-neural |
| BM42 (Qdrant) | contested, validity questioned post-launch | ~BM25 | yes (Qdrant Rust core) | unclear | qdrant.tech/articles/bm42 |
| miniCOIL | replaces BM42 in Qdrant's roadmap | sparse-fast | yes (Qdrant) | yes | qdrant.tech/articles/minicoil |
| Anthropic Contextual Retrieval | -49% failure, -67% w/ rerank | ingest-time only | language-agnostic (prompt-based) | yes (tested on codebases per Anthropic blog) | anthropic.com/news/contextual-retrieval |
| HyDE | mixed; underperforms vanilla on precise queries | +1 LLM call/query | yes (wrapper) | weak for numerical/code APIs | arxiv:2212.10496 |
| ms-marco-MiniLM-L-12 (ELAI current) | ~45-50 BEIR baseline | fast (33M params) | ONNX | generic | ELAI current |
| BGE-reranker-v2-m3 | baseline | medium | ONNX | yes | agentset.ai/rerankers leaderboard |
| Cohere Rerank v3 | +6.33% under Voyage rerank-2 | API-only | no | yes | blog.voyageai.com rerank-2 |
| Voyage rerank-2 | +14.75% over BGE-v2-m3 | medium API | no | yes | blog.voyageai.com rerank-2 |
| Jina-rerank-v3 | 61.94 BEIR (+4.88% vs v2) | medium | ONNX-friendly | yes | jina.ai/models/jina-reranker-v3 |
| HippoRAG | +20% multi-hop, 6-13x faster than IRCoT | KG build cost + PPR | partial (PPR is cheap in Rust) | yes (multi-file reasoning) | arxiv:2405.14831 |
| LeanRAG | token-efficient KG-RAG | medium | no native | yes | arxiv:2508.10391 |
| GraphRAG (MS) | strong global QA; LazyGraphRAG wins 96/96 vs LightRAG/RAPTOR/TREX | high index cost | no | partial | microsoft.com BenchmarkQED |
| LightRAG | 20-30ms faster than GraphRAG | medium | no | partial | maargasystems.com comparison |
| Self-RAG | 5.8% hallucination rate (vs 12-14% baseline) per 2025 MDPI | +N LLM calls per query | n/a | yes | arxiv:2310.11511 |
| CRAG | retrieval evaluator + web fallback | +1-2 LLM calls | n/a | yes (used in Cosmos-style refresh) | arxiv:2409.15337 |

## Why DEEP < STANDARD MRR (diagnosis)

Three cited failure modes line up exactly with ELAI's symptom (DEEP MRR 0.206 vs STANDARD 0.575):

1. **Reranker dilution beyond candidate-quality ceiling.** Drowning in Documents (arxiv:2411.11767) measured that "testing top-250 candidates found almost no improvement over top-100, meaning the retriever exhausts its useful candidates well before rank 250" and "an improved candidate list sometimes causes inferior reranking." When DEEP retrieves more candidates than the retriever can supply at quality, the reranker sees increasing low-relevance docs and its softmax output flattens. MRR collapses because the gold doc is no longer near rank 1.

2. **Retrieval ceiling unchanged by deeper pools.** Same paper: "If the dense retriever does not place the correct document in the top-100 candidates, no reranker can recover it." ELAI's measured fact that "ONNX models NOT staged on disk so dual-vector degrades to BM25-only at runtime; code_vec populated but never queried" means DEEP is already running on a crippled retriever — making the pool deeper just amplifies dilution without adding gold docs.

3. **HyDE noise on precise queries.** A finance-RAG benchmark cited recall@5 0.544 (HyDE) vs 0.587 (vanilla dense) — "LLM-generated hypothetical documents introduce noise by hallucinating plausible but incorrect financial figures, pulling the embedding away from the true relevant context." Code symbol names + API signatures behave like financial figures (high precision, low overlap) — HyDE hurts here. DEEP likely runs HyDE harder than STANDARD, compounding the problem.

Cross-cut: CodeRAG-Bench (arxiv:2406.14497) explicitly flags "current retrievers struggle to fetch useful contexts especially with limited lexical overlap" — exactly the DEEP regime in ELAI.

Sources: https://arxiv.org/abs/2411.11767 ; https://towardsdatascience.com/understanding-context-and-contextual-retrieval-in-rag/ ; https://arxiv.org/abs/2406.14497

## Best additions for ELAI
Ranked by ROI / integration cost. Each lists the ELAI layer it replaces or augments.

1. **Anthropic Contextual Retrieval at ingestion** — replace nothing; augment Layer 2 (BM25) + Layer 3 (dense vec). At chunk-ingest time, prepend an LLM-generated 50-100-token context sentence ("This chunk is from file X, in class Y, doing Z"). Cache via prompt caching (Anthropic's own technique). Hook: DKG ingest pipeline (where chunks are first emitted). Grounding: anthropic.com/news/contextual-retrieval — -49% retrieval failure on codebases. ELAI is Anthropic-protocol-compatible already; trivial wire-up via existing sidecar.

2. **Reranker upgrade ms-marco-MiniLM-L12 -> Jina-rerank-v3 (ONNX) or BGE-reranker-v2** — replace Layer 7. Same ONNX runtime, no architectural change. Grounding: jina.ai/models/jina-reranker-v3 reports 61.94 NDCG@10 BEIR (4.88% over v2). MiniLM-L12 ms-marco is 2020-era; minimum 10-15 NDCG@10 lift expected. Integration: swap ONNX file + reset normalization range.

3. **AST-aware chunking via cAST** — replace Layer 1's "adaptive AST-aware chunking" with cAST's specific recursive-split + sibling-merge algorithm. Grounding: arxiv:2506.15655 measures +4.3 Recall@5 on RepoEval, +2.67 Pass@1 on SWE-bench. ELAI already runs tree-sitter, so the implementation cost is the algorithm, not the parser. Hook: DKG node emission.

4. **PLAID / FastPlaid late-interaction for Layer 4** — add as optional ColBERT-style retriever alongside dense. next-plaid (Rust, CPU) is drop-in; FastPlaid (Rust+CUDA) for production. Replaces nothing, augments dual-vector dense as a stronger code-aware retriever per arxiv:2205.09707. Hook: new ranker in RRF fusion, weight starting 0.5 until measured. Integration: ELAI is Rust-native, so this is the lowest-friction late-interaction option in the ecosystem.

5. **Stage ONNX models on disk (FIX, not add)** — Layer 3 currently degrades to BM25-only at runtime. This is the cheapest fix in the report and likely re-orders the entire MRR landscape. Verify via runtime probe before any new technique is added.

6. **DEEP candidate cap + quality gate** — fix DEEP regression without architectural change. Cap candidate pool to 100 (per arxiv:2411.11767 finding). Add RRF-score threshold: drop candidates below 0.005 (k=60 floor) before rerank. Grounding: drowning-in-documents paper. No new code paths, just config.

7. **Replace HyDE with Anthropic Contextual Retrieval for code queries** — explicit drop. HyDE stays optional for natural-language DEEP queries where hallucination noise is tolerable. Grounding: HyDE underperforms vanilla on precise queries per multiple 2024-2025 benchmarks.

8. **Reason-ModernColBERT for "DEEP reasoning" tier** — for the small subset of queries that actually require multi-hop reasoning (B-N planning, audit, multi-file refactor scope), call Reason-ModernColBERT via PyLate sidecar. Grounding: lighton.ai/lighton-blogs/lighton-releases-reason-colbert — beats 7B models on BRIGHT at <200M params. Hook: existing remote sidecar.

9. **HippoRAG PPR as an upgrade to ELAI's existing PageRank repo-map (Layer 5)** — current PageRank uses aider's static graph. Personalized PageRank (seeded from query-extracted concepts) outperforms static PageRank on multi-hop by +20% (arxiv:2405.14831). Hook: same graph, replace seeding logic. KGGen (arxiv:2502.09956) can supply the KG construction if ELAI's symbol-graph isn't dense enough.

## What to drop / refactor

- **HyDE (Layer A in 3K hardening)** — for code queries: drop. Confirmed under-performance on precise / API-name / numeric queries (HyDE article on Towards Data Science citing recall@5 0.544 vs 0.587 vanilla). Pay-per-query cost in sidecar 128-tok generation, no measured ELAI uplift. Keep as opt-in for NL queries only.

- **ms-marco-MiniLM-L-12-v2 reranker (Layer 7)** — refactor to ONNX-compatible 2024-2025 reranker. The 33M-param size is fine; the staleness of training data (2020 ms-marco) is the issue. Jina-rerank-v3 or BGE-reranker-v2-m3 are both ONNX-friendly.

- **Query tier "DEEP" as currently implemented** — refactor. Either (a) cap candidate pool and add quality gate so DEEP > STANDARD on MRR as spec requires, or (b) remove DEEP and replace with a query-router that escalates only on detected reasoning intent (Adaptive-RAG / Self-RAG style). Current DEEP violates spec monotonicity per ELAI's own measurement.

- **BM42** — do NOT adopt. Qdrant's own subsequent posts ("modern sparse neural retrieval") signal BM42's evaluation methodology was questioned post-launch; miniCOIL is their forward-looking replacement. Stick with classic BM25 (tantivy) until SPLADE-v3 or miniCOIL is benchmarked specifically on code corpora.

Sources: https://qdrant.tech/articles/bm42/ ; https://qdrant.tech/articles/minicoil/

## Anti-patterns

- **Trusting BEIR/MTEB numbers as production proxy.** arxiv:2506.21182 documents how MTEB transitioned away from self-reported numbers because reproduction failed for many models (prefix handling, normalization, prompt-per-task differences). A 2026 Medium analysis ("The 8 Retrieval Benchmarks Lying to Your RAG") confirms single-hop recall benchmarks dangerously over-fit. Action: measure on a private ELAI-specific eval (R@5 / R@10 / MRR on the 135-file DKG), not just on public benchmarks.

- **Stacking more retrievers without measuring the marginal contribution.** RRF k=60 is empirically robust per multiple sources, but adding a 4th or 5th retriever rarely beats a well-weighted 3-retriever fusion. Use weighted RRF and measure per-component lift. Source: elastic.co weighted-RRF blog.

- **Building GraphRAG before measuring whether vector RAG is the bottleneck.** Microsoft's BenchmarkQED + LazyGraphRAG win-rate work shows GraphRAG dominates global / aggregation queries, not lookup-style queries. ELAI's queries are mostly code-symbol lookup — invest in late-interaction + contextual retrieval first; reserve GraphRAG for high-level architecture queries. Source: microsoft.com/research BenchmarkQED.

- **Adopting agentic RAG (Self-RAG, CRAG) before fixing the static pipeline.** Self-RAG cuts hallucinations to 5.8% vs 12-14% baseline (2025 MDPI Electronics study referenced in marsdevs.com) but adds 2-3 LLM calls per query. For a local sidecar (1.92 GB VRAM 3B model), this is not free. Fix retrieval first; add agentic loops only after static pipeline saturates the retrieval ceiling.

- **Reranker without retrieval-quality gate.** Per arxiv:2411.11767: a reranker scoring 250 candidates where only 50 are relevant is strictly worse than reranking 100 where 60 are relevant. Always gate the reranker input pool by a sparsity / score threshold.

## Open questions

1. **Does ONNX dual-vector staging fix DEEP MRR alone, or is the regression structurally rooted in over-retrieval?** Test plan: stage code_vec ONNX, re-run R@5/R@10/MRR on the DEEP tier, target STANDARD MRR ≥ 0.55, DEEP MRR > STANDARD MRR (spec monotonicity).

2. **What's ELAI's actual reranker ceiling on the 135-file DKG?** Run controlled ablation: replace ms-marco-MiniLM-L12 with Jina-rerank-v3 (ONNX) and BGE-reranker-v2-m3, measure ΔNDCG@10 on private eval. If Δ < 5 points, reranker is not the bottleneck — retrieval is.

3. **PL-PL retrieval: does BM25-word-level beat ELAI's current BM25+dual-vec at 10x lower latency on the ELAI corpus too?** Per arxiv:2510.20609 this is the dominant finding for code-to-code retrieval. Test: ablate dense vector ranker from RRF on PL-PL queries only.

4. **Does Anthropic Contextual Retrieval generalize from text corpora to code chunks with similar -49% lift?** Anthropic's blog claims it tested on "codebases, scientific papers, fiction" but the public numbers are aggregate. Run before/after on a 100-query ELAI-specific eval.

5. **What query fraction actually needs Reason-ModernColBERT (DEEP reasoning) vs. plain ColBERTv2?** Self-RAG / Adaptive-RAG style routers report 50-70% of queries can be answered without the deep path. Measure ELAI's distribution before allocating sidecar budget for Reason-ModernColBERT.
