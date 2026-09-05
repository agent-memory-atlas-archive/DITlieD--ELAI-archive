PAPER-NOT-FOUND: core-bench (repository-conditioned issue-to-edit retrieval benchmark)

SEARCH STATUS: UNVERIFIED / CANNOT GROUND
Date searched: 2026-06-23
Searches conducted: 25+ queries across multiple search engines and direct arXiv fetches
Conclusion: No arXiv paper was found matching the described characteristics.

The description given:
  Benchmark name: CORE-Bench
  Task: repository-conditioned issue-to-edit-localization retrieval
  3 levels: L1 code-understanding, L2 issue-to-edit localization, L3 broader-context
  Baseline: Qwen3-Embedding-8B 20.3 nDCG@10 / 48.0 Recall@100 on issue-to-edit
  After in-domain SFT: 32.8 nDCG@10 / 66.4 Recall@100
  Described slug: core-bench

PAPERS CHECKED AND RULED OUT:
- arXiv:2409.11363 CORE-Bench (Computational Reproducibility Agent Benchmark) -- completely different domain (science paper reproducibility, not code retrieval)
- arXiv:2507.05281 CoreCodeBench -- repository LLM benchmark, no L1/L2/L3 issue-to-edit retrieval, no nDCG/Recall@100
- arXiv:2602.05892 ContextBench -- context retrieval for coding agents, uses file/block/line granularity, not nDCG/Recall@100
- arXiv:2505.07849 SweRank -- issue localization via ranking, uses Acc@k not nDCG@10/Recall@100
- arXiv:2605.04615 CoREB -- text-to-code/code-to-text/code-to-code tasks, different metric structure
- arXiv:2506.05176 Qwen3 Embedding technical report -- evaluates on MTEB Code (nDCG@10), no L1/L2/L3 issue-to-edit benchmark
- arXiv:2605.16352 LARGER -- file/function localization, Acc@5/Recall@5, no Qwen3-Embedding-8B
- arXiv:2509.25242 MULocBench -- issue localization, different metrics (Acc@5, F1)
- arXiv:2606.14066 FastContext -- repository exploration agents, F1/Precision/Recall not nDCG
- arXiv:2603.29067 Beyond Localization -- SWE-bench RAG-APR, no embedding benchmark
- arXiv:2411.12644 CodeXEmbed -- multi-task code retrieval, different metric structure
- arXiv:2505.12697 CodeR (generalist code embedding) -- 16 diverse tasks, different evaluation structure

POSSIBLE EXPLANATIONS:
1. The paper may be a very recent preprint (post June 2026) not yet indexed
2. The paper name or slug in the task description may differ from the actual arxiv title
3. The described numbers (20.3/48.0/32.8/66.4) may be from a paper under a different name
4. The description may conflate elements from multiple papers

CLOSEST MATCHES (for reference, not ground truth):
- SweRank (2505.07849): issue-to-file localization with a retrieve-and-rerank approach, Qwen3 models used
- ContextBench (2602.05892): 3-level granularity (file/block/line) for issue-resolution context retrieval
- Qwen3-Embedding paper (2506.05176): reports nDCG@10 on code tasks including Code-Edit-Search

ANTI-SLOPSQUATTING DECLARATION:
Per project rules (CLAUDE.md anti-slopsquatting + grounded-by-default): this file DOES NOT fabricate
paper content, interface maps, or numbers. All content above is from verified search results.
No interface map, no contract tests, and no SFT recipe are written here because the source paper
cannot be verified. Any implementation claiming to reproduce CORE-Bench results without citing a
verified arXiv ID is ungrounded.

ACTION REQUIRED:
Provide the verified arXiv ID for this paper so it can be properly ingested. Once the ID is
confirmed via arXiv search, run:
  python F:/.ELAI_workflow/.claude/tools/fetch_arxiv.py <confirmed-id>
Then re-run this ingestor with mode=paper and the confirmed ID.
