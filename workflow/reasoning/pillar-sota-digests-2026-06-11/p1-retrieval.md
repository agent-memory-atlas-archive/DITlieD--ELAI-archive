DIGEST: p1-retrieval.md
SOURCE: eff4eb34-2d38-4976-b223-c01e9966a659.txt

TOPIC: Pillar P1 deep-dive — code-aware retrieval for coding agents. Audits the stack: tree-sitter symbol graph + BM25 (Tantivy) + dual ONNX embeddings (GraphCodeBERT-era 768d + text 384d) + HNSW + PageRank repo map + RRF + CPU cross-encoder reranker + HyDE. Overall verdict HARDEN, not replace.

REPORT VERDICTS (per sub-question):
- Q1 Compiler-grade indexes (SCIP/LSIF/LSP) vs tree-sitter = HARDEN with an LSP sidecar. Add Serena (or rust-analyzer/gopls LSP client) as fourth retrieval channel for go-to-definition / find-references / cross-file type resolution. Keep tree-sitter as incremental always-available fallback (LSP init can fail). Do NOT replace tree-sitter.
- Q2 Graph-RAG for code = Real gains for multi-hop; marginal for single-file. Upgrade is the QUERY INTERFACE (let agent issue multi-hop traversal queries, RANGER/LocAgent style), an agent-orchestration enhancement not necessarily a new index. Tree-sitter graph already captures structure.
- Q3 Newer code embedding models = REPLACE code-specialized model with CodeRankEmbed INT8 ONNX. Measurable CoIR gains, smaller footprint, 2x faster CPU, 8192-token context.
- Q4 Agentic grep vs pre-built index = KEEP pre-built index stack; HARDEN by adding agentic repair/verification layer (agent greps to confirm/extend index results, CORTEXA pattern). "Not OR but AND."
- Q5 Repo-map algorithms newer than PageRank = HARDEN PageRank with Louvain community priors (surface "the module cluster this symbol belongs to" not isolated symbols). Direct replacement (MCTS traversal) is a larger architectural change.
- Overall: HARDEN, three high-confidence additions. No single published system cleanly beats the complete pipeline on all query types as of June 2026.

TOP 3 RANKED CHANGES: 1. Replace code-embedding model -> CodeRankEmbed INT8 ONNX (HIGH, three independent benchmarks). 2. Add LSP/Serena as fourth channel for symbol-resolution queries (MEDIUM-HIGH, production+practitioner, no peer-reviewed ablation). 3. Add agentic grep as post-retrieval verification/repair action (MEDIUM, strong DCI paper but conflicting coding-agent ablation).

CANDIDATES:
Serena (oraios/serena) | https://github.com/oraios/serena (docs https://oraios.github.io/serena/01-about/000_intro.html) Apache 2.0 (+$5/mo JetBrains backend opt) GA ~April 2026 | MCP server wrapping native LSP 30-40+ languages for symbol-level navigation/refactoring inside any agent | adds Q1 LSP fourth-channel; compiler-grade go-to-def/find-refs/type resolution
LocAgent | arXiv:2503.09089 (https://arxiv.org/abs/2503.09089 ; ACL https://aclanthology.org/2025.acl-long.426/ ; alphaXiv https://www.alphaxiv.org/resources/2503.09089v1) MIT ACL 2025 | Heterogeneous AST graph + multi-hop MCTS traversal; 92.7% file-level accuracy SWE-Bench-Lite Qwen2.5-32B, 86% cost reduction vs proprietary | grounds Q2 multi-hop query interface
CodeRankEmbed | https://huggingface.co/nomic-ai/CodeRankEmbed (INT8 ONNX https://huggingface.co/mrsladoje/CodeRankEmbed-onnx-int8) Apache 2.0 ICLR 2025 | 137M bi-encoder, 8192 context, CoIR NDCG@10=60.1, outperforms CodeSage-Large-1.3B (59.4); INT8 ONNX 132MB 2x CPU speedup, 0.961 cosine parity | REPLACES P1 code-specialized 768d embedding slot
Codebase-Memory / codebase-memory-mcp | arXiv:2603.27277 (https://arxiv.org/abs/2603.27277 ; https://deusdata.github.io/codebase-memory-mcp/) MIT March 2026 | Tree-sitter + SQLite persistent knowledge graph via MCP; 83% answer quality at 10x fewer tokens, 2.1x fewer tool calls than file-explorer agent across 31 repos; Louvain community detection | grounds Q4 index token-efficiency + Q5 Louvain priors
RANGER | arXiv:2509.25257 (https://arxiv.org/abs/2509.25257 ; https://arxiv.org/html/2509.25257v1) academic preprint, no public release confirmed | Dual-stage: Cypher lookups for entity queries + MCTS-guided graph exploration for NL queries over per-repo KG w/ embeddings on nodes | Q5 replacement candidate (not drop-in deployable)
DCI / GrepSeek | arXiv:2605.05242 (https://arxiv.org/abs/2605.05242 ; https://stackfutures.com/blog/...) MIT (DCI-Agent-Lite) May 2026 | Zero-index agentic grep (agent issues grep/find/bash); +11% BrowseComp-Plus, +30.7% multi-hop QA over best semantic retrievers, ~30% cost reduction; beats sparse/dense/reranking on BRIGHT+BEIR | grounds Q4 agentic-grep-repair add
RepoGraph | no-url (ICLR 2025 https://proceedings.iclr.cc/paper_files/paper/2025/file/4a4a3c197deac042461c677219efd36c-Paper-Conference.pdf) | Graph-based ego-subgraph retrieval; +2.66 and +2.34 absolute resolve-rate on SWE-bench across two base frameworks | Q2 graph-RAG evidence
CGM | arXiv:2505.16901 (https://arxiv.org/html/2505.16901v3) | Graph-based context retrieval more effective for code completion than text-flattening (RepoFuse) | Q2 graph-over-flattening
CodexGraph | arXiv:2408.03910 (https://arxiv.org/abs/2408.03910) | Graph DB + Cypher queries; 27.9% EM CrossCodeEval Python GPT-4o; 22.96% Pass@1 SWE-bench Lite | Q2 graph-RAG evidence
SWE Context Bench | no-url (Emergent Mind https://www.emergentmind.com/topics/swe-context-bench) Feb 2026, 1,136 tasks, 66 repos | Advanced agents w/ embedding/graph/file tools do NOT consistently outperform basic mini-SWE-agent baseline on aggregate; oracle summary reuse +34% beats complex scaffolding +0% | attacks graph-RAG aggregate claims; Q5 PPR overhead
SFR-Embedding-Code-400M_R | no-url (CoIR leaderboard https://archersama.github.io/coir/) Salesforce | 400M, CoIR NDCG@10=61.89, CoIR rank 3 | Q3 alternative embedding
CodeXEmbed-7B | arXiv:2411.12644 (https://arxiv.org/abs/2411.12644) Salesforce/NV | 7B, CoIR SOTA 2024 (>60), outperforms Voyage-Code by 20%+; too heavy for CPU | Q3 (rejected for CPU)
nomic-embed-code | no-url (https://www.nomic.ai/news/introducing-state-of-the-art-nomic-embed-code) Nomic March 2025 | 7B, outperforms Voyage-Code-3 on CSN; too large for CPU | Q3 (rejected for CPU)
voyage-code-3 | no-url (AWS https://aws.amazon.com/marketplace/pp/prodview-d5nri3kbddsrw) API-only Dec 2024 | MTEB avg 85.51, 13.8% over OpenAI-v3-large, 32K context; not self-hostable | Q3 (cloud-only, rejected)
jina-code-embeddings | arXiv:2508.21290 (https://arxiv.org/abs/2508.21290) Jina Aug 2025 | Small autoregressive, SotA CoIR at model size, 8192 context, ONNX export supported | Q3 secondary candidate once ONNX matures
NV-EmbedCode-7B | no-url (https://wowelec.wordpress.com/2026/05/18/...) NVIDIA 7B Mistral | 71.95% recall SWE-bench file retrieval; too heavy for CPU | Q4 index-recall benchmark anchor
Tantivy (BM25) | no-url | Standard tokenizers split snake_case/CamelCase on word boundaries (validate_jwt_token -> validate/jwt/token), inflates BM25 for unrelated shared-subtoken code | the audited BM25 leg (failure mode)
HyDE | arXiv:2212.10496 (https://arxiv.org/pdf/2212.10496.pdf) | Hypothetical-document embedding; one LLM forward pass per query; no benefit for exact-identifier lookups, hallucinated API surface risk | audited HyDE leg (questioned cost at local inference)
CORTEXA / Nemotron-Cortexa | no-url (OpenReview https://openreview.net/pdf?id=k6p8UKRdH7) | Index retrieval + localization agent repair stage; resolves 68.2% SWE-bench Verified at $3.28/problem | grounds Q4 index-first-grep-to-repair pattern
Coding Agents are Effective Long-Context Processors | arXiv:2603.20432 (https://arxiv.org/html/2603.20432v1) March 2026 | Equipping coding agents w/ retrieval tools does NOT consistently improve and can degrade it (agents reduce native grep, brittle index dependence) | conflicting evidence on agentic retrieval
rust-analyzer | https://github.com/rust-lang/rust-analyzer | Rust compiler front-end for IDEs; first-class for Rust orchestrator LSP channel; resolves macros/trait impls/associated types | Q1 LSP channel for Rust
Claude Code (RAG abandonment) | no-url (LinkedIn https://www.linkedin.com/posts/cole-medin-...) | Abandoned local vector DB after Boris Cherny confirmed agentic search "outperformed everything by a lot" | Q4 agentic-grep evidence
Noqta semantic indexing report | no-url (https://noqta.tn/en/blog/...) May 2026 | Semantic indexing: 84% fewer agent steps, 62x fewer tokens on VS Code codebase (2.45M LOC) vs grep | Q4 pre-built-index evidence

ATTACK-THE-DESIGN (strongest criticisms):
- B1 tree-sitter: CST parser not type resolver; can't distinguish overloaded methods, resolve dynamically-bound interfaces (Rust trait objects, Python duck typing), follow macro-expanded symbols; false edges (name collision) + false negatives (macro-generated invisible); per-language .scm walker maintenance debt at 18 grammars; build-time generated sources (protobuf/sqlx/proc_macro) absent at parse time -> zero incoming edges breaking PageRank.
- B2 BM25: identifier tokenization split inflates scores; synonym blindness (buf/buffer/BytesMut); stale-window artifact (function name in comment in one 50-line window, body in next).
- B3 embeddings: GraphCodeBERT-era underperforms; likely leaving 3-8 CoIR NDCG@10 points on table while running larger model; 512-token windows truncate long functions; circuit-breaker degrades to BM25-only (40.67% file-level vs 71.95% NV-EmbedCode).
- B4 PageRank: stale context on cross-service monorepo edits (interface drift not captured, old call signature ranked high); seeding sensitivity (degrades to unpersonalized PageRank over most-cited utils when entry point unknown); token-budget compression elides bodies to signatures (right names wrong implementations = silent retrieval failure for debugging).
- B5 HyDE: ~3-5s per retrieval at consumer latency, paid unconditionally; no benefit for exact-identifier majority; hallucinated API surface retrieves adjacent-but-wrong code, invisible until patch generation.
- B6 RRF: normalizes by rank not score, a retriever returning nothing relevant still contributes top-k noise; no query-type routing (identifier vs semantic have opposite retriever profiles, RRF averages, underperforms oracle routing).
- B7 cross-encoder reranker: latency blow-up under load (MS-MARCO-MiniLM 100 pairs ~2.3s CPU; <50ms on 50 candidates tight on CPU); runs full reranking unconditionally, no selective skip when BM25+vector agree.

NUMBERS (claimed source, unverified):
- LocAgent: 92.7% file-level accuracy SWE-Bench-Lite Qwen2.5-32B vs 86.13% Claude-3.5; +12% Pass@10 over Agentless; -18% if TraverseGraph/SearchEntity disabled; 86% cost reduction [arXiv:2503.09089].
- CodeRankEmbed: 60.1 CoIR NDCG@10 vs 59.4 CodeSage-Large-1.3B; INT8 132MB 2x CPU 0.961 min cosine vs FP32; CoRNStack 21M triples [HF/ICLR2025].
- SFR-Embedding-Code: CoIR NDCG@10 61.89 [CoIR leaderboard].
- voyage-code-3: MTEB avg 85.51, 13.8% over OpenAI-v3-large [AWS].
- Codebase-Memory: 83% answer quality, 10x fewer tokens, 2.1x fewer tool calls, 31 repos [arXiv:2603.27277].
- RepoGraph: +2.66 / +2.34 absolute resolve-rate SWE-bench [ICLR 2025].
- CodexGraph: 27.9% EM CrossCodeEval Python GPT-4o; 22.96% Pass@1 SWE-bench Lite [arXiv:2408.03910].
- SWE Context Bench: 1,136 tasks, 66 repos; NV-EmbedCode 71.95% file-level recall vs BM25 40.67% vs GPT-4o-retriever 65.55%; CORTEXA 68.2% SWE-bench Verified at $3.28/problem; oracle summary reuse +34% vs complex scaffolding +0% [emergentmind/openreview].
- DCI: +11% BrowseComp-Plus, +30.7% multi-hop QA, ~30% cost reduction [arXiv:2605.05242].
- Noqta: 84% fewer agent steps, 62x fewer tokens, VS Code 2.45M LOC [noqta.tn].
- Crossover threshold ~500K+ LOC where grep returns unmanageable hits [report synthesis].

REPORT'S OWN FLAGS: ~120x token reduction for codebase-memory-mcp is a specific "what calls X?" query not the mean (arXiv reports 10x mean at 83% quality); Sourcegraph SCIP-precision claim is vendor/internal, no neutral benchmark; DCI advantage shown on general IR corpora (Wikipedia/BEIR) not code-specific monorepo-scale (>1M LOC, >18 langs); RANGER (Sept 2025) no confirmed public release.
