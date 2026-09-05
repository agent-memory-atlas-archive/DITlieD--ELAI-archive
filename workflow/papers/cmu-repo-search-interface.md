Source: Repository-level Code Search with Neural Retrieval Methods
ArXiv: 2502.07067
URL: https://arxiv.org/abs/2502.07067
Authors: Siddharth Gandhi, Luyu Gao, Jamie Callan
Institution: Language Technologies Institute, Carnegie Mellon University (confirmed via ResearchGate + cs.cmu.edu affiliations)
Submitted: 10 February 2025
Reliability: HIGH (peer-reviewed arXiv, CMU LTI; numbers extracted from HTML full-text at arxiv.org/html/2502.07067v1)

CORE FINDING
Multi-stage BM25+CommitReranker+CodeReranker pipeline does NOT significantly outperform a single CodeReranker applied at depth 100,
while costing 7x more runtime. The CodeReranker alone matches the full pipeline on MAP/MRR/P@1 at depth 100.
The commit-message reranking stage is the expensive bottleneck that adds minimal incremental gain.

SETUP
Task: repository-level code search — given a natural-language bug report, retrieve the set of files from the current
repo state most relevant to fixing it.
Dataset: 7 popular open-source repos (includes Facebook/React), queries derived from real issue reports.
Evaluation: MAP, MRR, P@1, P@5, R@10, R@100. Two settings: normal (no oracle) and oracle.
Baseline: BM25 over commit messages.

PIPELINE STAGES (sequential)
Stage 1 BM25 Initial Ranker: full-text BM25 over commit messages. Retrieves ~1,000 candidate files.
Stage 2 CommitReranker: BERT/CodeBERT reranker over commit messages. Reduces to ~100s of files. Depth=100 or 1000.
Stage 3 CodeReranker: BERT/CodeBERT reranker over source code. Final output ~10s of files. Depth=100.
Full Pipeline: Stages 1+2+3 chained.

VERBATIM EXTRACTS (from arxiv.org/html/2502.07067v1)

Verbatim key conclusion:
"CodeReranker demonstrates significantly better performance compared to the CommitReranker, matching the full
pipeline's performance at a reranking depth of 100."

Verbatim over-staging finding:
"The full pipeline performs well but does not significantly outperform the CodeReranker at a depth of 100,
despite taking 7 times longer to execute."

Verbatim abstract:
"Experiments on a new dataset created from 7 popular open-source repositories demonstrate substantial improvements
of up to 80% in MAP, MRR and P@1 over the BM25 baseline, across a diverse set of queries, demonstrating the
effectiveness this approach."

Table 7 — Averaged Performance Across 7 Repositories (non-oracle):
Component        | Depth | MAP   | MRR   | P@1   | P@5   | R@10  | R@100
BM25             | -     | 0.163 | 0.227 | 0.139 | 0.090 | 0.265 | 0.607
CommitReranker   | 100   | 0.252 | 0.361 | 0.250 | 0.143 | 0.390 | 0.607
CodeReranker     | 100   | 0.288 | 0.408 | 0.289 | 0.163 | 0.421 | 0.607
Full Pipeline    | -     | 0.307 | 0.434 | 0.299 | 0.175 | 0.470 | 0.709

Table 8a — Facebook/React Detailed Results (non-oracle):
Component        | Depth | MAP   | MRR   | P@1   | P@5   | R@100
BM25             | -     | 0.119 | 0.180 | 0.080 | 0.094 | 0.557
CommitReranker   | 100   | 0.187 | 0.330 | 0.200 | 0.168 | 0.557
CodeReranker     | 100   | 0.267 | 0.420 | 0.300 | 0.202 | 0.557
Full Pipeline    | -     | 0.270 | 0.434 | 0.280 | 0.202 | 0.590

Note: On Facebook/React, Full Pipeline MAP=0.270 vs CodeReranker MAP=0.267 — negligible delta, confirms the claim.

Table 6 — Runtime (Facebook/React, 100 test queries):
Component                | Total Time | Per-Query Time
BM25                     | 7.5 mins   | 4.5 s
CommitReranker@1000      | 5.5 hrs    | 3.5 mins
CodeReranker@100         | 1.5 hrs    | 1 min
Full Pipeline            | 7.5 hrs    | 4.5 mins

Runtime ratio: Full Pipeline 7.5 hrs vs CodeReranker@100 1.5 hrs = 5x (paper rounds to "7x" likely including
commit indexing overhead not in this table, or referencing a different query count / depth configuration).

ABLATION INTERPRETATION
The critical ablation: CodeReranker@100 achieves MAP=0.288 vs Full Pipeline MAP=0.307 averaged (6.6% relative gap).
On React specifically the gap is 0.267 vs 0.270 (1.1% relative). The 7x runtime cost is not justified by the
marginal MAP gain in these conditions.
CommitReranker@100 is substantially weaker than CodeReranker@100 (MAP 0.252 vs 0.288) — commit history reranking
is less effective than direct code reranking, confirming that code signals dominate.

REIMPLEMENTATION NOTES FOR ELAI
ELAI's retrieval stack: BM25 + dual ONNX dense encoders + PageRank + RRF fan-in + cross-encoder reranker.
This paper is ANTI-OVER-ENGINEERING evidence.
The question this paper answers: does adding commit-history stages on top of a good code reranker pay off?
Answer: No, at depth 100. A single strong code reranker captures most of the gain.

Concrete ELAI implications:
1. ELAI's cross-encoder reranker is the high-value stage. The BM25+dense+RRF fan-in feeding into the
   reranker is the analog of "BM25+CommitReranker feeding CodeReranker." If the reranker is strong, adding
   more upstream stages (e.g. PageRank, a second dense encoder) buys marginal MAP at high latency cost.
2. The paper does NOT say BM25 alone is sufficient — it scores MAP=0.163 vs reranker MAP=0.288-0.307.
   So the BM25 first-pass + single reranker is justified. ELAI's pattern is sound.
3. Where ELAI diverges: ELAI uses RRF to combine BM25 + 2 dense encoders + PageRank BEFORE reranking.
   This paper's analog would be the CommitReranker stage — it's an intermediate fan-in that costs latency.
   The paper suggests the incremental value of that fan-in over a single good retriever is small.
4. Recommendation: if ELAI latency is acceptable, keep the full stack. If latency is the pain point,
   ablate by removing the PageRank signal from the RRF blend and measure MAP delta. The paper predicts
   the delta will be small (~6% relative on averaged repos, <2% on popular repos like React).
5. The paper does NOT cover symbol-graph / call-graph navigation signals — those are orthogonal and
   not comparable to this ablation. See codescalebench-interface.md for that evidence.

GROUNDED CHECKS (testable assertions against real retrieval runs)
1. assert single_reranker_map >= 0.85 * full_pipeline_map, "single reranker should capture >=85% of full pipeline MAP per Table 7 (0.288/0.307=0.938)"
2. assert commit_reranker_map < code_reranker_map, "commit-message reranker weaker than code reranker per Table 7 (0.252 < 0.288)"
3. assert full_pipeline_runtime >= 4.5 * code_reranker_runtime, "full pipeline at least 4.5x slower per Table 6 (7.5h vs 1.5h)"
4. assert bm25_map < 0.20, "BM25 alone under 0.20 MAP per Table 7 (0.163)"
5. assert code_reranker_p1 >= 2.0 * bm25_p1, "CodeReranker P@1 at least 2x BM25 per Table 7 (0.289 vs 0.139)"
