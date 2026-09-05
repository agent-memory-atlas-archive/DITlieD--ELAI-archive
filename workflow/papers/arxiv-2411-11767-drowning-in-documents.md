Algorithm: Scaling Reranker Inference (empirical study, not a standalone algorithm)
ArXiv: 2411.11767
Paper: Drowning in Documents: Consequences of Scaling Reranker Inference
Authors: Mathew Jacob, Erik Lindgren, Matei Zaharia, Michael Carbin, Omar Khattab, Andrew Drozdov (Databricks / UIUC)
Venue: ReNeuIR 2025 at SIGIR 2025 (4th Workshop on Reaching Efficiency in Neural Information Retrieval)
Version: v2 (generated 2025-07-11)

Abstract:
Rerankers (typically cross-encoders) are assumed to consistently outperform first-stage retrievers
and to improve with more reranked documents. The authors challenge both assumptions empirically.
Using strong dense embedding retrievers (not BM25) and diverse academic + enterprise datasets,
they find that reranker recall initially improves but then degrades substantially as K (number of
documents reranked) increases, often dropping below retriever-only recall. They identify "phantom
hits" as the primary failure mode and show listwise LLM reranking is more robust but impractical.


KEY EMPIRICAL FINDINGS (verbatim quotes + section anchors)
==========================================================

Finding 1: Scaling K leads to degraded recall (the "saturation and decline" finding)

Section 4 / S4.p1 (footnote 1):
  "For cross-encoder evaluations, reranking the top-100 [Gao et al. 2021] or top-1000 [refs] documents
  is typical."
  Context: paper varies K from small up to 5000+ to characterize behavior beyond the typical range.

Section 4.1 / S4.SS1.p2 (Table 1 caption, key finding):
  "rerankers are much less consistently helpful when scaling the number of documents for reranking"

Section 4.1 / S4.SS1.p4 (bold emphasis in paper):
  "One of our key observations is that continuing to include more documents for the reranker is an
  ineffective way to scale computation."
  "In very few cases did using the largest value of K lead to the best average Recall@10 (see 'Helps
  & Scaling is Best' in Table 1)."
  "53.3% and 44.4% of the time across academic and enterprise datasets, respectively, we saw that
  even when reranking a small K was effective, that scaling K lead to Recall@10 that was worse than
  retrieval alone (see 'Helps & Scaling Hurts' in Table 1)."

Table 1 data (S4.T1) — percentage of experiments meeting each condition:
  Helps (any K):                academic=85.0%, enterprise=88.9%
  Helps & Never Hurts:          academic=23.3%, enterprise=22.2%
  Helps & Distracted:           academic=60.0%, enterprise=66.7%
  Helps & Scaling is Best:      academic=3.3%,  enterprise=14.8%
  Helps & Scaling Hurts:        academic=53.3%, enterprise=44.4%

Figure 1 caption (S1.F1):
  "While rerankers are thought to outperform retrievers, we find instances where scaling the number
  of documents K used in reranking often leads to a substantial decrease in recall."

Figures 2 and 3 (S3.F2, S3.F3):
  "Recall@10 when reranking, averaged across academic [and enterprise] datasets. The dashed line
  shows the first stage recall, and the solid line is the rerankers' recall. The rerankers' recall
  often degrades as the reranked K increases."

NOTE ON "TOP-250 vs TOP-100 SATURATION" CLAIM:
  The paper does NOT contain an explicit "top-250 vs top-100 saturation" finding or that exact
  phrasing. The paper compares K values from small (e.g. 10, 50, 100) up to 5000+. The phantom-hit
  analysis specifically filters for queries where "Recall@10 decreased when reranking K=5000 instead
  of K=100 documents" (S4.SS2.p2). The text in S4.p1 fn1 notes that top-100 is "typical" for
  cross-encoders. The saturation / peak-then-decline shape is visible in Figures 2 and 3 (images,
  not quoted text), occurring somewhere in the 50-500 range depending on model and dataset.
  No specific cap value (100, 200, 250) is textually recommended anywhere in the paper.


Finding 2: "improved candidate list sometimes causes inferior reranking" (phantom hits)

Section 1 / S1.p3:
  "reranking with large K decreases recall precipitously (Figure 1), often dropping beneath the
  quality of standalone retrievers."
  "rerankers often score completely irrelevant documents, lacking semantic or lexical similarity to
  the query, very highly. These same documents, which we call phantom hits, are often given very low
  scores by the retrievers, revealing a counterintuitive way in which retriever models can outperform
  reranker models in practice."

Section 4.1 (Table 1 narrative) / S4.SS1.p4:
  "53.3% and 44.4% of the time across academic and enterprise datasets, respectively, we saw that
  even when reranking a small K was effective, that scaling K lead to Recall@10 that was worse than
  retrieval alone"

Section 4.2 / S4.SS2.p1:
  "reranking with a large K often leads to worse performance: Recall@10 is lower than that of
  retrieval alone in 53.3% and 44.4% of experiments on academic and enterprise datasets, respectively."
  "we attribute many reranker errors to what we call 'phantom hits'. These are cases where the reranker
  prefers an irrelevant negative document to the positive ground truth when there is no apparent
  connection between the query and irrelevant document, lexical, semantic, or otherwise."

Section 4.2 / S4.SS2.p2 (analysis methodology):
  "we filtered for queries where the Recall@10 decreased when reranking K=5000 instead of K=100
  documents."
  "we see multiple cases where the reranker prefers irrelevant phantom hits over positive documents
  even when they have little or no text overlap with the query."
  "this behavior is not isolated to a single reranker"

Figure 6 example (S4.F6) — Cohere rerank-english-v3.0 with text-embedding-3-large:
  Positive document (retriever rank 5, reranker rank 33) replaced by a negative about
  dishwashing apparatus (retriever rank 4615, reranker rank 3).


RERANKER MODELS TESTED
======================

Open models (S3.SS2.SSS0.Px1):
  - jina-reranker-v2-base-multilingual (accessed via API; not evaluated on enterprise datasets
    due to legal constraints)
  - bge-reranker-v2-m3 (run locally)

Closed models (S3.SS2.SSS0.Px2):
  - cohere rerank-english-v3.0 (accessed via Cohere API)
  - voyage-rerank-1 (accessed via Voyage AI API)

LLM listwise (S4.SS3):
  - gpt-4o-mini with sliding window (window=20, stride=10) from Sun et al. 2024

Table 2 (A3.T2) — complete model list:
  voyage-2               (E = Embedding retriever)
  text-embedding-3-large (E = Embedding retriever)
  voyage-rerank-1        (R = Reranker)
  cohere-rerank-v3       (R = Reranker, same as rerank-english-v3.0)
  bge-reranker-v2-m3     (R = Reranker)
  jina-reranker-v2-base-multilingual (R = Reranker)

ms-marco-MiniLM-L12 STATUS: NOT tested in this paper. The paper tests jina-reranker-v2-base-multilingual,
bge-reranker-v2-m3, cohere rerank-english-v3.0, and voyage-rerank-1. ms-marco-MiniLM-L12 (a
sentence-transformers cross-encoder) is not mentioned anywhere in the paper. Findings apply to it
by analogy (same pointwise cross-encoder architecture and exposure-bias hypothesis) but are not
directly measured on it.


RECOMMENDED CAP VALUE
=====================

The paper does NOT recommend a specific cap value. Key quotes:

S4.p1 footnote 1: "reranking the top-100 or top-1000 documents is typical" (descriptive, not
  prescriptive — this is what prior work does, not what this paper recommends).

S4.SS2.p2: The paper's own analysis uses K=5000 vs K=100 as a contrast point for identifying
  phantom-hit failures. K=100 is used as the "small K" comparison baseline, not as a
  recommended cap.

S4.SS3.SSS0.Px2.p1: Listwise LLM experiments run at K in {50, 100, 500, 1000}.

The paper's prescription is qualitative: "reranking a small set of documents is an effective way
to improve recall" (S4.SS1.p1). It identifies that "small K" works and "large K" hurts, but never
specifies a numeric threshold. The choice of cap=100 in ELAI's plan is consistent with the paper's
findings (K=100 is the prior-work-typical value cited in the paper) but is NOT a paper-endorsed
specific number.


RRF FLOOR RECOMMENDATION
=========================

The paper contains NO mention of RRF (Reciprocal Rank Fusion), RRF floor values, or any threshold
constant resembling 0.005. The paper's scope is purely empirical characterization of scaling
behavior; it does not propose hybrid retrieval algorithms, fusion formulas, or score thresholds.

The 0.005 RRF floor used in ELAI E-17 is NOT supported by this paper. It must be sourced
elsewhere (e.g., the original RRF paper by Cormack et al. 2009, or empirical tuning).


DISCUSSION / HYPOTHESES FOR FAILURE (Section 5)
================================================

Negatives in Training (S5.SS0.SSS0.Px1):
  "pointwise rerankers may see less negatives in training than their embedding model counterparts
  because large batch reranker training is computationally expensive and does not easily share
  negatives across the batch"
  "rerankers are trained on negatives selected from a subset of the corpus filtered by retrievers,
  which may explain the peak performance of rerankers occurring at a lower number of retrieved
  documents"

Reranking as Ensembling (S5.SS0.SSS0.Px2):
  "reranker is providing alternative features (compared to embeddings) that are helpful to
  discriminate relevant from irrelevant documents only from the top of the list"

Robustness of Deep Learning (S5.SS0.SSS0.Px3):
  "as one scales the number of documents, each additional document included creates a risk that the
  model may assign an inappropriately high score to an irrelevant document"

Favorable conditions for rerankers (S5.SS0.SSS0.Px6 — important caveat):
  "Prior studies are often more limited to settings that are particularly well suited for reranking:
  using BM25 as the first stage retrieval and evaluated against MSMarco or similar datasets"
  This means results are STRONGER against dense embeddings (as in ELAI) than BM25 pipelines.


DATASETS USED
=============

Academic: Scifact (BEIR), RELIC (BIRCO), DORIS-MAE (BIRCO), Biology (BRIGHT), Pony (BRIGHT)
Enterprise: FinanceBench, ManufacturingQA, DocumentationQA
Corpus size: downsampled to max N=10,000 documents per dataset
Query truncation: 512 tokens (both queries and documents)


KEY FIGURES
===========

Figure 1 (S1.F1): Illustration of recall degradation as K increases; phantom hit definition diagram.
Figure 2 (S3.F2): Recall@10 vs K, averaged across academic datasets. Shows peak then decline.
Figure 3 (S3.F3): Recall@10 vs K, averaged across enterprise datasets. Shows peak then decline.
Figure 4 (S4.F4): Recall@K for full retrieval (reranker applied to entire corpus). Retrievers win.
Figures 5-6 (S4.F5, S4.F6): Phantom hit examples (Scifact). Cohere ranks dishwashing document
  at position 3 over true positive at position 33.
Table 1 (S4.T1): Percentage statistics for Helps / Never Hurts / Distracted / Scaling is Best /
  Scaling Hurts across all (dataset, retriever, reranker) experiment tuples.


LIMITATIONS (paper's own statement, section "Limitations")
===========================================================

"several of the models used are closed source, where we do not have access to the information like
the training data, precise model architecture, and model size"
"experimenting with different training strategies, training data distributions, and model sizes
may confirm some of our hypotheses"


IMPLEMENTATION IMPLICATIONS FOR ELAI E-17
==========================================

1. cap=100 is consistent with the paper (it is the "typical" small-K value cited as the prior-work
   norm) and sits safely before the observed saturation/decline inflection in the graphs. The paper
   does not explicitly endorse cap=100 as optimal; it could be 50 or 150 and remain within the
   paper's spirit. The key claim is "small K good, large K bad" without a precise boundary.

2. cap=250 would NOT be directly contradicted by the paper but the paper's graphs show degradation
   beginning around that range for some model/dataset combinations (graphical observation, not a
   quoted number). cap=100 is therefore more conservative and better supported.

3. RRF floor=0.005 is completely unsupported by this paper. Cite the Cormack et al. 2009 RRF
   paper or label it as an empirically tuned constant.

4. The paper's findings are specifically about pointwise cross-encoders with STRONG dense embedding
   retrievers as first stage. This matches ELAI's likely architecture. The failure modes are
   STRONGER in this setting than BM25+reranker.

5. The "phantom hits" phenomenon means that any candidate beyond the saturation point can actively
   HURT ranking quality by introducing high-scoring irrelevant documents. A cap is therefore not
   merely a cost optimization but a correctness safeguard.


Critical invariants for E-17 (testable assertions):
assert deep_cap <= 200, "cap above 200 enters documented degradation territory for cross-encoders"
assert deep_cap >= 20, "cap below 20 loses most reranking benefit (paper: 'small K helps')"
assert rrf_floor_is_not_from_this_paper, "2411.11767 contains zero mention of RRF or 0.005"
# No specific cap value can be derived from this paper; 100 is a reasonable prior-work-typical value
# not a paper-endorsed optimum
