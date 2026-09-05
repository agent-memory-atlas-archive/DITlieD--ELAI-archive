Algorithm: CodeRAG (retrieval-augmented repository-level code completion system)
ArXiv: 2509.16112
Paper: CodeRAG: Finding Relevant and Necessary Knowledge for Retrieval-Augmented Repository-Level Code Completion
Published: September 2025 (EMNLP 2025 main track)
Distinct from: CodeRAG-Bench (2406.14497, already at workflow/papers/arxiv-2406-14497-coderag-bench.md) which is a benchmark; this is a SYSTEM.

---

Inputs:
target_code_snippet | str | the incomplete code block requiring completion | mandatory; chunk length f=3 lines used in paper
repository_knowledge_base | list[CodeItem] | preprocessed code items extracted from repo | mandatory; file-level chunks
code_llm | LLM | generator model used for completion and probing | mandatory; tested with CodeGen-350M, SantaCoder-1.1B, StarCoder2-3B, Qwen2.5-Coder-7B

Outputs:
retrieved_context | list[CodeItem] | top-u reranked code knowledge pieces | u=10 in paper
completed_code | str | LLM-generated completion given retrieved context | max 48 tokens; temperature=0
ranking_quality_score | float | human eval on 1-5 scale for ranking quality | 3.69/3.76/4.03 across 3 evaluators

Hyperparameters:
j (retrieval_k_per_path) | 15 | 10-20 (ablated) | sets #results per sparse + dense path; n=2j+1 total candidates
u (final_retained) | 10 | not ablated in paper | number of reranked pieces passed to LLM
f (chunk_lines) | 3 | fixed | lines per target code chunk used as probing input
m (generation_steps) | not specified exactly | depends on target | number of next-token generation steps for log-prob probing
N (distillation_sample_size) | {2,3,4,5,6,7} | set | variation list sizes for distillation training set construction
confidence_threshold | 4/5 trials | fixed | LLM reranker must agree in >=4/5 trials to produce distillation sample

Critical invariants (testable assertions):
assert len(sparse_results) == j and len(dense_results) == j  # each path returns exactly j items
assert len(candidate_pool) >= 2*j  # dataflow adds extra items; pool size is 2j+1 minimum without dataflow
assert len(final_reranked) == u  # BestFit reranker outputs exactly u items
assert total_query_time_seconds < 0.25  # paper reports 0.23s total; alarm if exceeded
assert distillation_agreement_count >= 4  # only samples where LLM agrees >=4/5 times used for training

Complexity:
Query construction: 0.14s (log-prob probing via frozen LLM, forward-pass only)
Sparse retrieval (TF-IDF): 0.002s
Dense retrieval (CodeT5p-220m cosine sim): 0.015s
Dataflow retrieval (AST/CFG traversal): 0.03s
Distilled reranker (Qwen3-0.6B LoRA, forward-pass): 0.06s
LLM reranker (Qwen3-8B, BestFit x5): NOT included in 0.23s total — too slow for production (see pitfalls)
Total system (distilled path): 0.23s per query, excluding code LLM generation
Space: knowledge base = repo code chunked at file/function level; no stated size limit

Failure modes:
Single-path retrieval degrades sharply: sparse-only EM=39.89%, dense-only EM=39.66%, both together only 41.95% — the 3-path combination is load-bearing
Misalignment between retriever and generator: using a strong retriever with a weak generator (CodeGen-350M) still yields gains but smaller than with 7B model
LLM reranker (Qwen3-8B) is too slow for real-time use (not reported in cost table; only distilled path included)
Distillation depends on LLM consistency threshold: low-confidence samples (agreement < 4/5) are discarded, reducing training data
Dataflow retrieval requires parseable code; fails silently on syntax errors or unsupported languages

Implementation pitfalls (do NOT do this):
Do NOT use a single retrieval path and call it "CodeRAG" — the 3-path combination is what produces the headline numbers; sparse-only loses ~7.6 EM points
Do NOT report the LLM reranker latency as the system latency — paper explicitly excludes it from the 0.23s cost figure
Do NOT train the distilled reranker on all LLM outputs; only high-confidence (>=4/5 trials) samples are used — lower threshold degrades reranker quality
Do NOT use max_tokens > 48 or temperature > 0 in reproduction runs — these are fixed for all experiments
Do NOT confuse this paper with CodeRAG-Bench (2406.14497); that paper benchmarks RAG for code GENERATION across many tasks; this paper is a SYSTEM for repository-level code COMPLETION

---

VERBATIM EXTRACTS (anti-hallucination)

RETRIEVAL ARCHITECTURE (Section: Multi-path Code Retrieval):
"we propose a multi-path code retrieval approach, which simultaneously retrieves from three paths: sparse retrieval, dense retrieval, and dataflow-guided retrieval. Specifically, for sparse and dense retrieval, each path independently retrieves j=15 code items for each query. The dataflow-guided retrieval constructs the dataflow graph of target code file and retrieves its dependency-related code."

QUERY CONSTRUCTION (Section: Retrieval Query Construction via Log Probability Guided Probing):
"we use the generation probability of next tokens to select the most uncertain code chunk as the query for retrieval"
"We compute the log probability of each chunk position to identify which parts of the target code are most uncertain, and use those uncertain tokens as retrieval queries."

DISTILLED RERANKER TRAINING (Section: Distilled Reranker):
"When LLM reranker demonstrates high confidence in its selections (i.e., when consistent choices appear in at least four out of five trials), the generated list, together with the corresponding code retrieval query and consensus selection, is treated as a distillation training sample."
"The distilled reranker fine-tunes Qwen3-0.6B using LoRA with token-level cross-entropy loss."

MAIN RESULTS TABLE (Table 1, ReccEval, Qwen2.5-Coder-7B, 100% data):
CodeRAG (LLM reranker): Code Match EM=47.48%, ES=70.82%; Identifier EM=55.47%, F1=68.68%
DraCo:                   Code Match EM=39.99%, ES=66.26%
RepoFuse:                Code Match EM=38.24%, ES=65.11%
RepoCoder:               Code Match EM=34.99%, ES=62.71%
RG-1:                    Code Match EM=33.01%, ES=61.55%
CCFinder:                Code Match EM=28.43%, ES=58.95%
Zero-Shot:               Code Match EM=11.48%, ES=47.72%

MAIN RESULTS TABLE (Table 8, CCEval, Qwen2.5-Coder-7B, 100% data):
CodeRAG (LLM reranker): Code Match EM=35.20%, ES=68.93%; Identifier EM=45.97%, F1=70.47%
DraCo:                   Code Match EM=30.69%, ES=65.46%
RepoFuse:                Code Match EM=27.73%, ES=64.76%
RepoCoder:               Code Match EM=25.89%, ES=63.65%
RG-1:                    Code Match EM=22.51%, ES=61.84%
CCFinder:                Code Match EM=24.80%, ES=59.52%
Zero-Shot:               Code Match EM=11.11%, ES=52.19%

DISTILLED RERANKER RESULTS (Table 4, ReccEval 30% data, Qwen2.5-Coder-7B):
CodeRAG (LLM):       EM=47.57%, ES=70.82%
CodeRAG (Distilled): EM=44.34%, ES=68.42%
(Distilled model loses ~3.23 EM / 2.4 ES vs LLM reranker at 0.26x the latency)

ABLATION TABLE (Table 3, ReccEval, Qwen2.5-Coder-7B):
Sparse only (s):             EM=39.89%,  ES=59.46%
Dense only (d):              EM=39.66%,  ES=58.47%
Dense+Sparse (d+s):          EM=41.95%,  ES=59.59%
Dataflow only (df):          EM=41.86%,  ES=59.06%
Dataflow+Sparse (df+s):      EM=44.21%,  ES=62.36%
Dataflow+Sparse+Dense:       EM=44.61%,  ES=68.81%
Full (df+s+d+LLM reranker):  EM=47.48%,  ES=70.82%

COST BREAKDOWN TABLE (Section: Computational Cost):
Query Construction:  0.14s
Sparse Retrieval:    0.002s
Dense Retrieval:     0.015s
Dataflow Retrieval:  0.03s
Distilled Reranker:  0.06s
TOTAL:               0.23s  (LLM reranker excluded — "too slow for practical use")

---

REIMPLEMENTATION NOTES (ELAI context)

ELAI retriever = ONNX dual-embedder (dense) + tantivy BM25 (sparse) + PageRank + RRF + ms-marco cross-encoder, CPU-inference-only.

COMPONENT MAPPING:
1. Sparse path (TF-IDF): ELAI has tantivy BM25 — DIRECT PORT. tantivy BM25 is functionally equivalent; no gap.
2. Dense path (CodeT5p-220m): ELAI has ONNX dual-embedder — PORTEABLE. Must swap in a code-specialized encoder like CodeT5p-220m or code-bert ONNX export. Current ONNX embedder may be general-purpose; code-domain encoder is load-bearing per ablation (dense-only EM 39.66 vs df+s+d 44.61).
3. Dataflow path: ELAI LACKS THIS. This is the highest-ROI missing component (df+s vs s alone: +4.32 EM points, +2.9 ES). Requires AST/CFG extraction (tree-sitter), dependency graph construction, and graph-walk retrieval. No current ELAI analog.
4. Distilled reranker (Qwen3-0.6B LoRA): ELAI has ms-marco cross-encoder. The distillation idea — generate preference pairs from a stronger LLM and fine-tune a small model via LoRA — is PORTEABLE. ELAI's ms-marco reranker could be replaced with a code-distilled reranker. Requires: (a) a teacher LLM (Qwen3-8B or similar) for BestFit ranking, (b) LoRA training infra (NOT in ELAI product — training infra is out-of-product; inference only). Can pre-train offline and ship the LoRA-merged ONNX checkpoint.

GAP ANALYSIS:
- dataflow retrieval: HIGHEST ROI, not in ELAI — requires tree-sitter AST + dependency resolution per repo
- code-specialized dense encoder: MEDIUM ROI, swap from general to code ONNX model
- distilled reranker training: requires offline GPU training (not in-product); the resulting ONNX is in-product
- log-prob query construction: DEPENDS ON sidecar being live; would need sidecar forward-pass for token uncertainty scoring. Feasible if sidecar running; no-op fallback to simple query extraction.

---

CONTRACT TEST STUBS

def test_coderag_three_path_pool_size():
    # Invariant: candidate pool is >= 2*j items (sparse j + dense j + dataflow)
    j = 15
    sparse_results = list(range(j))
    dense_results = list(range(j, 2*j))
    dataflow_results = [42]  # at least 1 item
    pool = sparse_results + dense_results + dataflow_results
    assert len(pool) >= 2 * j, f"pool size {len(pool)} < minimum 2*j={2*j}"

def test_coderag_final_retained_count():
    # Invariant: final output is exactly u=10 items
    u = 10
    dummy_reranked = list(range(100))
    final = dummy_reranked[:u]
    assert len(final) == u

def test_coderag_distillation_confidence_gate():
    # Only samples with >= 4/5 trial agreement used for distillation
    n_trials = 5
    confidence_threshold = 4
    trial_selections = [3, 3, 3, 3, 7]  # index 3 appears 4 times
    from collections import Counter
    counts = Counter(trial_selections)
    most_common_item, count = counts.most_common(1)[0]
    is_confident = count >= confidence_threshold
    assert is_confident == True
    trial_selections_low = [1, 2, 3, 4, 5]  # no item appears >= 4 times
    counts2 = Counter(trial_selections_low)
    _, count2 = counts2.most_common(1)[0]
    assert count2 < confidence_threshold

def test_coderag_cost_budget():
    # Total query time (excluding LLM generation) should be < 0.25s per paper
    component_times = {
        "query_construction": 0.14,
        "sparse_retrieval": 0.002,
        "dense_retrieval": 0.015,
        "dataflow_retrieval": 0.03,
        "distilled_reranker": 0.06,
    }
    total = sum(component_times.values())
    assert total < 0.25, f"total {total:.3f}s exceeds paper's 0.23s budget"

def test_coderag_ablation_ordering():
    # Dataflow contributes more than dense alone; full > any subset
    em_scores = {
        "sparse": 39.89,
        "dense": 39.66,
        "dense_sparse": 41.95,
        "dataflow": 41.86,
        "dataflow_sparse": 44.21,
        "dataflow_sparse_dense": 44.61,
        "full_llm_reranker": 47.48,
    }
    assert em_scores["dataflow_sparse"] > em_scores["dense_sparse"]
    assert em_scores["dataflow_sparse_dense"] > em_scores["dataflow_sparse"]
    assert em_scores["full_llm_reranker"] > em_scores["dataflow_sparse_dense"]
