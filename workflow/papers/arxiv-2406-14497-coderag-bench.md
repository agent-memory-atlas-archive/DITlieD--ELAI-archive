Algorithm: CodeRAG-Bench retrieval-augmented code generation (RACG) benchmark findings
ArXiv: 2406.14497
Paper: CodeRAG-Bench: Can Retrieval Augment Code Generation?
Authors: Wang et al. (CMU, UW, USC), 2024

---

SCOPE OF PAPER

This is a benchmark paper, not an algorithm paper. It evaluates 10 retrievers x 10 generators
on six code generation tasks. It does NOT evaluate HyDE, query expansion, or pseudo-document
generation strategies. All retrieval experiments use vanilla retrieval only (authors' own
statement, Limitations section: "for benchmarking purposes, we mostly experimented with vanilla
retrieval, reranking, and generation methods").

---

CRITICAL FINDING FOR E-17 HyDE GATE

HyDE is NOT evaluated, NOT discussed, and NOT mentioned anywhere in this paper.
This paper CANNOT directly justify "disable HyDE for code queries" through empirical comparison.

What the paper DOES provide as indirect support:

1. BM25 (exact/lexical match) achieves near-perfect NDCG@10 on problem-solution tasks
   (HumanEval: 100.0, MBPP: 98.6, CSN: 89.1) but catastrophically fails on library-doc
   retrieval tasks (DS-1000: 5.2, ODEX: 6.7). Table 3, Section 3.2.

2. The NL-to-code task structure: queries are NL problem descriptions; documents are
   code/documentation. The CSN task (CodeSearchNet) is the canonical NL-query-to-code-function
   retrieval task with NDCG@10 as primary metric.

3. Dense code-specific models outperform general-purpose dense models and BM25 for code
   retrieval: Jina-v2-code beats GIST-base by +7.4 NDCG@10 average. Section 3.2.

---

TABLE 3: RETRIEVAL NDCG@10 BY RETRIEVER AND TASK CATEGORY (verbatim from paper)

Columns: Method | HumanEval | MBPP | CSN | DS-1000 | ODEX | RepoEval | SWE-bench | Avg

BM25              | 100.0 | 98.6 | 89.1 | 5.2  | 6.7  | 93.2 | 43.0 | 57.7
GIST-base (768)   |  98.0 | 98.0 | 89.9 | 12.0 | 12.1 | 81.2 | 46.8 | 58.0
GIST-large (1024) | 100.0 | 98.9 | 89.6 | 13.6 | 28.0 | 82.9 | 47.8 | 61.7
BGE-base (768)    |  99.7 | 98.0 | 90.0 | 10.8 | 22.0 | 77.5 | 44.9 | 58.8
BGE-large (1024)  |  98.0 | 99.0 | 90.6 |  8.9 | 11.5 | 80.4 | 40.1 | 56.3
SFR-Mistral (4096)| 100.0 | 99.0 |  N/A | 19.3 | 37.1 | 83.8 | 62.7 | 67.0
Codesage-small    | 100.0 | 96.3 | 90.7 |  8.9 | 14.3 | 94.1 | 47.1 | 60.1
Jina-v2-code (768)| 100.0 | 97.7 |  N/A | 26.2 | 19.9 | 90.5 | 58.3 | 65.4
OpenAI-03 (1536)  | 100.0 | 98.9 |  N/A | 18.2 | 16.5 | 93.0 | 43.3 | 61.7
Voyage-code (1536)| 100.0 | 99.0 |  N/A | 33.1 | 26.6 | 94.3 | 29.1 | 63.7

Key pattern: DS-1000 and ODEX are library-documentation retrieval tasks where the query is an
NL programming problem and the document is API documentation. ALL retrievers fail here relative
to problem-solution tasks. Best performer (Voyage-code) achieves only 33.1 on DS-1000 vs 100.0
on HumanEval. BM25 achieves only 5.2 on DS-1000 (19x worse than HumanEval).

---

QUERY/DOCUMENT TYPE DISTINCTIONS (what the paper actually distinguishes)

The paper distinguishes task categories, not query types per se:

1. Basic programming (HumanEval, MBPP): NL problem -> solution code. Retrieval is easy (near
   perfect NDCG) because problem phrasing overlaps with solution code naming conventions.

2. Open-domain (DS-1000, ODEX): NL + library usage intent -> API documentation or SO posts.
   Retrieval is hard. Best NDCG@10: 33.1 (Voyage-code, DS-1000). BM25 collapses to 5.2.
   Reason implied (not stated explicitly): library doc language differs significantly from
   user problem description.

3. Repository-level (RepoEval, SWE-bench): code-context query -> in-repo code snippets.
   Moderate retrieval quality. BM25 strong here (93.2 on RepoEval) because code tokens are
   shared between query and document.

4. CodeSearchNet (CSN): NL docstring -> Python function body. All retrievers score ~89-91.
   This is the code-symbol/NL-to-function task most similar to ELAI's use case.

NOTE: The paper does NOT explicitly frame this as "NL query vs PL/code-symbol query".
That framing is ELAI's interpretation applied to this data. The paper frames it as
task category difficulty.

---

RECOMMENDED RETRIEVAL ARCHITECTURES (from empirical results)

1. Code-specific dense retrievers outperform general dense and BM25 on average:
   Voyage-code (avg 63.7), Jina-v2-code (avg 65.4), SFR-Mistral (avg 67.0) are top performers.

2. BM25 remains competitive for exact-match code tasks (problem-solution, repo-level):
   BM25 is best or near-best on HumanEval, MBPP, RepoEval (93.2 > Voyage 94.3 close).

3. Hybrid is NOT evaluated in this paper. RRF is NOT evaluated. Only vanilla retrieval +
   optional reranking (BGE-reranker-base on top-100 OpenAI results) was tested.

4. BM25 params used: k1=1.2, b=0.75 (Section 3.1). Implementation: pyserini.

5. Chunking recommendation: pre-retrieval chunking at 200-800 tokens outperforms post-retrieval.
   First 800 tokens best for most sources except SO posts. Section 4 / Figure 2.

6. Number of documents: top-5 retrieved documents is optimal across models (Appendix E).

7. Reranking with BGE-reranker-base hurts performance within the 200-800 token range:
   "we find reranking within this optimal range of 200-800 tokens greatly degrades the results,
   showing limited utility of current rerankers." Section 4.

---

GENERATION FAILURE MODES FROM RETRIEVAL (Section 3.4 / Appendix)

1. Context distraction: "most models can be easily distracted or disturbed by additional
   contexts (Wang et al., 2023a), and fail to conduct the designated code generation task"
   Source: Section 3.4 body, RepoEval analysis.

2. Function copying: "most models tend to copy functions in the context, sometimes even
   overwriting the function being queried, thus failing all the test cases specific to the
   queried function." Source: Section 4, ODEX open retrieval analysis.

3. Over-complication: "models tend to generate over-complicated programs which, however,
   do not often pass the test cases" Source: Section 4.

4. Context window limit: "generation models with limited context windows exhibit smaller
   improvements" Source: Introduction. "generation models have limited context capacity and
   RAG abilities, both leading to suboptimal RACG results." Source: Conclusion.

---

RETRIEVAL FAILURE MODES

1. Low NDCG on library documentation retrieval: all retrievers fail severely on DS-1000 and
   ODEX. The gap between basic-programming NDCG (~99) and library-doc NDCG (5-33) is the
   central finding of Section 3.2. Paper implies this is a semantic mismatch problem but
   does NOT explicitly use that term.

2. "retrievers often struggle to fetch useful contexts, particularly for open-domain and
   repository-level tasks" Source: Introduction.

3. "current retrieval models struggle to find useful documents" Source: Conclusion.

4. Despite low retrieval NDCG, RACG can still improve over no-retrieval: "despite relatively
   low NDCG@10 scores, the best-performing RACG combinations match their canonical results
   on ODEX-hard" Section 3.4. This means retrieval quality and generation quality are
   partially decoupled.

---

QUERY-CLASS CLASSIFIER

The paper does NOT recommend or evaluate any query-class classifier upstream of retrieval.
No such system is proposed or studied.

---

HYPERPARAMETERS CONFIRMED IN PAPER

BM25: k1=1.2, b=0.75 (Section 3.1, pyserini)
Dense retrieval: top-k=5 documents passed to generation (optimal, Appendix E)
Chunking: 200-800 tokens pre-retrieval (Section 4, Figure 2)
Temperature for generation: t=0.2, top_p=0.95 (Section 3.1)
Context window used: varied by model (StarCoder2: 16k, DeepSeekCoder: 4k)

---

COMPLEXITY

10 retrievers x 10 generators x 6 tasks evaluated.
BM25 encoding: 0.15ms/doc, search: 0.02ms/query, index: 141MB.
SFR-Mistral (7B): encoding 316ms/doc, search 113ms/query, index 14220MB.
GIST-base: encoding 3.7ms/doc, search 9.7ms/query, model 440MB, index 307MB.

---

WHAT THIS PAPER CANNOT SUPPORT

1. Any claim about HyDE performance on code queries (not evaluated).
2. Any claim about query expansion degrading recall on code tasks (not evaluated).
3. NL-vs-PL query classification framework (not studied).
4. Hybrid BM25+dense or RRF (not studied).
5. Non-Python languages (only Python studied, per Limitations).

---

CONTRACT ASSERTIONS (testable from Table 3 data)

assert bm25_ndcg_humaneval >= 99.0  # Table 3
assert bm25_ndcg_ds1000 <= 10.0     # Table 3, BM25=5.2
assert voyage_code_ndcg_ds1000 > bm25_ndcg_ds1000 * 5  # 33.1 vs 5.2
assert jina_v2_code_avg > gist_base_avg  # 65.4 > 58.0, +7.4 as stated
assert optimal_chunk_size_tokens in range(200, 801)  # Section 4
assert optimal_docs_count == 5  # Appendix E
