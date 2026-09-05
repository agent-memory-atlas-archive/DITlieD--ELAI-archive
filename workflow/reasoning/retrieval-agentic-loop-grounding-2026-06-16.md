# Retrieval: agentic-loop vs one-shot pipeline — grounding for J-38 (+ J-18 reinforcement)

Session 2026-06-16. Trigger: the user handed a Turbopuffer talk transcript ("RAG is dead, right? — how
hybrid tool-rich retrieval is becoming the default for serious agentic search", Kuba Rogut) and asked
whether ELAI already does it or missed something. Validated via spec-eval + 2 sonnet oracle-mode research
agents (web-grounded, ~181K subagent tokens) + main-LLM arXiv verification.

## The three methods on the table
- METHOD A (Cursor): persistent embedding index — parse/chunk/embed up front, Merkle-tree incremental
  re-embed on changed files only, semantic search at runtime; cross-team simhash dedup.
- METHOD B (Claude Code / Codex): NO vector DB — iterative agentic grep/glob/read loop, re-discovers
  context every session.
- METHOD ELAI: hybrid pipeline (BM25 tantivy + dual-ONNX vector + PageRank repo map + cross-encoder
  rerank + SCIP/tree-sitter graph + RRF k=60 + HyDE + tier routing), invoked largely as a PRE-TASK
  pipeline filling fixed context-budget slots (RepoMap/CodeContext). Narrow agent tools exist
  (impact_radius/symbol_context/call_flow); the FULL hybrid retriever is NOT a mid-task callable tool.

## Verdict (design level, as-specified)
ELAI's design is CLOSER TO SOTA than either pole. "RAG is dead" is refuted at the retrieval layer; the
real shift is "put the hybrid index behind an agent-callable interface and loop on it" (Turbopuffer's own
actual thesis, not "kill vectors"). ELAI has the superset index; the one genuine gap is that retrieval is
pre-task slot-fill, not a mid-task loop.

## VERIFIED arXiv IDs (main-LLM WebFetch on arxiv.org/abs, 2026-06-16)
- 2606.07297 = "SWE-Explore: Benchmarking How Coding Agents Explore Repositories" (Zhang, Wang, Liang,
  Shi, Zeng, Wang, He, Xu, Ye, Cai, Gu). 848 issues / 10 langs / 203 repos. Abstract VERIFIED:
  "agentic explorers form a clear tier above classical retrieval"; "file-level localization is already
  strong... line-level coverage and efficient ranking remain the key axes". The 51-68% file-hit /
  15-19% line-recall / 48-50% resolve numbers are from the full-paper HTML the agent fetched (full-paper,
  not abstract) — treat as design-rationale, not a target (HD-1/ID-5).
- 2512.16956 = "SpIDER: Spatially Informed Dense Embedding Retrieval for Software Issue Localization"
  (Chaudhari, Jacob, Goswami, Cao, Rashid, Bock). Abstract VERIFIED: "improves dense retrieval
  performance by at least 13% across programming languages and benchmarks" — the +13pp is in the abstract.
  The Recall@3 0.44-vs-LocAgent-0.18 and 48h-LocAgent figures are full-paper, design-rationale.
- 2601.23254 = "GrepRAG: An Empirical Study and Optimization of Grep-Like Retrieval for Code Completion"
  (Wang, Wang, Li, Zhi, Han, Zhao, Wang, Deng, Yin). Abstract VERIFIED: "7.04-15.58% relative improvement
  in code exact match over the best baseline on CrossCodeEval". The 42.29% EM / 35x-faster figures are
  full-paper. Honest counter-evidence: lexical grep wins on identifier-heavy completion.
- 2505.07849 = "SweRank: Software Issue Localization with Code Ranking". Abstract VERIFIED: "outperforming
  both prior ranking models and costly agent-based systems using closed-source LLMs like Claude-3.5". The
  83.21% Acc@1 / 44x-cheaper figures are full-paper.
- 2503.09089 = LocAgent (already VERIFIED in J-18: ACL 2025, gersteinlab/LocAgent, 92.7% file-level —
  dataset-ambiguous, design-rationale not target).

## The two grounded gaps competitor methods win
GAP 1 (Method A) — stale code embedder. GraphCodeBERT (2020). voyage-code-3 ~MRR 0.973 vs GraphCodeBERT
~0.509 on Voyage's 32-dataset code suite (~91% relative); NVIDIA code-tuned embedder 71.95% file recall vs
BM25 40.67%. ALREADY ELAI's plan J-18 (CodeRankEmbed INT8 swap — chosen over voyage-code-3 deliberately for
CPU-ONNX/MIT/exact-768-dim reasons; voyage is cloud-only/>500M). J-18's decision stands; the agents
independently re-derived it and grounded the staleness delta.

GAP 2 (Method B / the talk's actual thesis) — retrieval is pre-task slot-fill, not a mid-task loop.
SWE-Explore: line-level recall 15-19% even at 64-68% file coverage = "missing context is the dominant
failure mode". LocAgent: 92.7% file-level precisely via mid-task graph-traversal TOOLS the agent loops on.
ELAI exposes narrow traversal tools but not the full hybrid retriever as a callable tool. -> J-38.

## What ELAI's design still wins (do not over-correct)
- SweRank: non-agentic retrieve+rerank beats the SWE-Agent grep loop on localization AND cost (44x). Pure
  agentic grep (Method B) is NOT strictly better.
- SpIDER: graph-augmented one-shot beats the iterative LLM graph agent 2.4x on recall at far less compute —
  "iterative always wins" is false. The pre-task pipeline amortizes (Turbopuffer's own 6,314->424 token
  discovery drop is cached-index-as-tool, not no-index).
- Hybrid+rerank is validated (Anthropic contextual-retrieval -67% top-20 failures; T2-RAGBench +12.1pp
  Recall@5 from the reranker) — general-domain, code-direction confirmed by SpIDER/SweRank.

## The three grounded improvements -> J-38 (#2-#4); J-18 already owns #1
1. (J-18, promote) embedder swap — already planned, grounded by GAP 1.
2. (J-38 leg 1) expose the J-18-hardened HYBRID cascade as a mid-task agent-callable tool the executor
   loops on (FAST tier, reranker optional for latency) — grounded by SWE-Explore + LocAgent. ADDITIVE to
   pre-task fill (index-first, loop-to-target), NOT a replacement.
3. (J-38 leg 2) SpIDER-style retrieval-time graph-neighbor expansion + rerank over the SHIPPED SCIP/DKG
   graph — grounded by SpIDER +13% (2512.16956 VERIFIED). ELAI has both the graph and the retriever but
   does not fuse them at retrieval time today.
4. (J-38 leg 3) deterministic context-sufficiency signal gating when a mid-task search fires — grounded by
   SWE-Explore r=0.92 (ranked context quality is measurable). Bounds over-retrieval; LLM never adjudicates
   (TD-3).

Honest residue: GrepRAG (2601.23254) — for identifier-heavy queries pure grep beats the pipeline 35x
faster; J-38's sufficiency gate + FAST-tier routing should send exact-identifier queries to BM25+symbol
graph, reserving the DEEP pipeline / mid-task semantic loop for intent queries.

Source: 2 sonnet oracle-mode agents this session (agentId a5a9d61ec59d13253 = Method-A/embedder leg;
aba55a58bea2b8514 = Method-B/agentic-loop leg). Cursor's 12.5%/24% numbers are INTERNAL, never
third-party reproduced (UNVERIFIED, design-rationale only).
