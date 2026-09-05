verify-p1-retrieval
task_id: verify-p1-retrieval
date: 2026-06-11
scope: web research only, no code edits

---

CLAIM 1: CodeRankEmbed license = MIT
STATUS: VERIFIED
URL: https://huggingface.co/nomic-ai/CodeRankEmbed
NOTES: MIT license confirmed on HF model card. Official nomic-ai release.

CLAIM 2: CodeRankEmbed params = 137M
STATUS: VERIFIED
URL: https://huggingface.co/nomic-ai/CodeRankEmbed
NOTES: 137M confirmed. Initialized from Arctic-Embed-M-Long.

CLAIM 3: CodeRankEmbed embedding dimension = 768
STATUS: VERIFIED (CRITICAL)
URL: https://huggingface.co/nomic-ai/CodeRankEmbed/resolve/main/config.json (config.json n_embd=768); corroborated https://simonwillison.net/2025/Mar/27/nomic-embed-code/
NOTES: hidden_size=768, n_head=12, n_inner=3072, n_layer=12, vocab=30528. EXACT MATCH to current HNSW slot. No matryoshka variants listed.

CLAIM 4: CodeRankEmbed context length = 8192
STATUS: VERIFIED
URL: https://huggingface.co/nomic-ai/CodeRankEmbed (README), config.json n_positions=8192
NOTES: max_position_embeddings=8192. Note max_trained_positions=2048; the 8192 window uses positional extension (common with NTK/YaRN on Arctic lineage).

CLAIM 5: CodeRankEmbed CoIR NDCG@10 = 60.1
STATUS: VERIFIED
URL: https://huggingface.co/nomic-ai/CodeRankEmbed
NOTES: 60.1 NDCG@10 on CoIR confirmed from HF card. CoIR leaderboard context: zembed-1 (proprietary API, ZeroEntropy) scores 0.6452, CodeRankEmbed at 0.601 is a strong open/lightweight competitor. No open <500M model verified above it on CoIR as of mid-2026.

CLAIM 6: CodeRankEmbed query prefix required = "Represent this query for searching relevant code"
STATUS: VERIFIED
URL: https://huggingface.co/nomic-ai/CodeRankEmbed
NOTES: Asymmetric: query MUST use this prefix; document/code passages encoded WITHOUT prefix. This is a hard API contract — omitting prefix on queries degrades recall. Current retrieval pipeline must inject prefix at query-encode time.

CLAIM 7: mrsladoje/CodeRankEmbed-onnx-int8 — third-party INT8 ONNX quant
STATUS: VERIFIED (with CAVEATS)
URL: https://huggingface.co/mrsladoje/CodeRankEmbed-onnx-int8
NOTES: Repo exists. 162 downloads/month (very low). Community quant by "mrsladoje", NOT official nomic-ai. Derived from jalipalo/CodeRankEmbed-onnx (also community). Dynamic INT8 with reduce_range=True for VNNI-less CPU compat. Includes SHA256 for verification. 139 MB ONNX file. Trustworthiness: transparent methodology + MIT inherit, but no nomic-ai affiliation and very low download count. RECOMMENDATION: export ONNX ourselves from the official card using optimum CLI (one-command, deterministic) rather than trusting a 162-dl community quant. The base model is public and optimum export is well-established.

CLAIM 8: jina-code-embeddings arXiv:2508.21290 — ONNX availability
STATUS: PARTIALLY VERIFIED / UNFAVORABLE FOR DROP-IN
URL: https://arxiv.org/abs/2508.21290 ; https://huggingface.co/jinaai/jina-code-embeddings-0.5b
NOTES: Paper real, Aug 2025. Two sizes: 0.5B (896d, CC-BY-NC-4.0) and 1.5B. License is CC-BY-NC — non-commercial, incompatible with product use. Dimension 896 != 768, so NOT a drop-in for current HNSW slot. No confirmed ONNX INT8 export on HF card at time of check. Not a viable alternative under current constraints.

CLAIM 9: Alternatives — anything better than CodeRankEmbed at <500M params CPU ONNX mid-2026
STATUS: UNCLEAR (no confirmed open alternative found above 60.1 CoIR at <500M with ONNX + commercial license)
URL: https://www.zeroentropy.dev/articles/05-best-embedding-for-code/ (redirected to homepage, no table); https://github.com/coir-team/coir
NOTES: zembed-1 tops CoIR at 64.5 but is API-only (ZeroEntropy SaaS), not local ONNX. Next open-weight contenders (voyage-4-nano 64.1, OpenAI 61.5) are API-only. CodeRankEmbed at 60.1 NDCG@10 remains the best confirmed open, local, <500M, commercial-licensed ONNX-exportable code embedding as of mid-2026. Claim to replace it stands.

CLAIM 10: Serena github.com/oraios/serena — LSP MCP server, headless, MIT
STATUS: VERIFIED
URL: https://github.com/oraios/serena
NOTES: 25.2k stars, MIT license, 40+ language LSP support including Rust. Works as MCP server. Can run headless (HTTP mode or stdio) without an editor. rust-analyzer not named explicitly but Rust is listed among supported languages (implying rust-analyzer backend). Active project (2,944 commits).

CLAIM 11: Direct LSP stdio from Rust (rust-analyzer/pyright/typescript-language-server) as lighter path
STATUS: PLAUSIBLE / UNVERIFIED from primary source
URL: (no dedicated primary source fetched; based on LSP spec and rust-analyzer docs in training)
NOTES: All three servers support stdio transport per LSP spec. Rust binding: tower-lsp crate (client-side). Lighter than Serena (no MCP layer) but requires per-language client wiring. This is a real option but adds per-LSP integration maintenance burden vs Serena's unified abstraction.

CLAIM 12: LocAgent arXiv:2503.09089 — real, ACL 2025, 92.7% file-level localization
STATUS: VERIFIED (benchmark dataset = UNCLEAR)
URL: https://arxiv.org/abs/2503.09089 ; https://github.com/gersteinlab/LocAgent
NOTES: Real paper, ACL 2025 accepted. GitHub repo at gersteinlab/LocAgent, 613 stars, functional codebase. Model: Qwen2.5-Coder-32B-CL fine-tuned. 92.7% is file-level localization accuracy. Benchmark dataset: repo supports SWE-Bench Lite, SWE-Bench Verified, and a custom Loc-Bench_V1. The abstract does not specify which dataset the 92.7% figure is from — treat as UNCLEAR on exact dataset provenance. The 92.7% claim itself is consistent with published abstract language ("up to 92.7%"), not a fabricated number.

CLAIM 13: DCI / agentic-grep arXiv:2605.05242 — real
STATUS: VERIFIED
URL: https://arxiv.org/abs/2605.05242 ; https://github.com/DCI-Agent/DCI-Agent-Lite
NOTES: Real paper (May 2026). Title: "Beyond Semantic Similarity: Rethinking Retrieval for Agentic Search via Direct Corpus Interaction." Argues for terminal-tool-based corpus search (grep, file reads, shell) with NO embedding model / vector index. Substantially outperforms strong sparse+dense+reranking baselines on BRIGHT, BEIR, BrowseComp-Plus. GitHub repo DCI-Agent/DCI-Agent-Lite exists. ONE-LINE TAKEAWAY: embedding retrieval is a bottleneck for agents; direct grep-based corpus interaction wins on complex multi-hop tasks.

CLAIM 14: Counter-paper arXiv:2603.20432 — "retrieval tools can degrade coding agents"
STATUS: REFUTED AS DESCRIBED
URL: https://arxiv.org/abs/2603.20432
NOTES: The paper's actual title is "Coding Agents are Effective Long-Context Processors." It argues IN FAVOR of agents using executable tools and file systems for retrieval — it does NOT argue retrieval degrades agents. It outperforms SOTA by 17.3% average on long-context benchmarks. The "degrades" framing in the deep-research report does not match this paper's actual position. ONE-LINE TAKEAWAY: long-context file-system + tool-using agents outperform semantic retrieval by 17.3% across benchmarks. This paper and DCI (2605.05242) both favor agentic/direct approaches, but neither argues AGAINST adding retrieval channels to a hybrid stack — they argue against retrieval-ONLY architectures. The "opposite directions" framing in the original task prompt appears to be an over-reading; both papers are critical of pure semantic retrieval, not of hybrid retrieval+grep.

CLAIM 15: codebase-memory-mcp arXiv:2603.27277 + Louvain community priors — real
STATUS: VERIFIED
URL: https://arxiv.org/abs/2603.27277 ; https://github.com/DeusData/codebase-memory-mcp
NOTES: Real paper (March 2026). Tree-Sitter + SQLite + Louvain community detection for repo maps. 10x lower token cost, 2.1x fewer tool calls vs raw retrieval on 31 languages. GitHub repo DeusData/codebase-memory-mcp, 3.1k stars. Louvain is a confirmed component (not Leiden). Architecture aligns with our existing tree-sitter symbol graph in SQLite — this is a prior art confirmation, not a replacement.

CLAIM 16: Louvain Rust crate — anti-slopsquat check
STATUS: VERIFIED (concrete crate identified)
URL: https://crates.io/crates/ocg (via search); https://crates.io/crates/single-clustering
NOTES: "ocg" (last published Feb 26 2026, includes Louvain function in graph algorithm suite) and "single-clustering" (Sep 11 2025, Louvain + Leiden + Rayon parallel, marked NOT production-ready). "graphify-cluster" (Apr 2026, Leiden via lib.rs). "louvain-rs" (graphext/louvain-rs on GitHub, not confirmed on crates.io as primary). "fast-louvain" (Splines, work-in-progress, no crates.io release). RECOMMENDATION: ocg is the most recently published (Feb 2026) confirmed crates.io entry with Louvain; single-clustering has it but is self-described not-production-ready. Neither has high download counts. For production use, implementing Louvain directly over petgraph (which IS production-grade and widely used in ELAI) is the safer path than trusting low-traffic community crates.

---

DIMENSION-COMPATIBILITY NOTE

CodeRankEmbed outputs 768-dimensional vectors — exact match to the current usearch HNSW slot (GraphCodeBERT-768d code embedding). A drop-in swap requires:
  - Re-encoding all existing code documents with the new model (index rebuild, not schema migration)
  - Changing the model-load path in the embedder crate
  - Injecting the mandatory query prefix "Represent this query for searching relevant code" at query-encode time
  - No usearch index_version bump needed (same dimension)
  - No RRF weight change needed (same retrieval channel)
The swap is index-rebuild-only, not an architecture change. The Granite-384d text embedding in the second HNSW slot is UNAFFECTED (different channel, different dimension).

---

FEASIBILITY HINTS

Change 1 — REPLACE GraphCodeBERT with CodeRankEmbed INT8 ONNX:
  FEASIBLE. Dimension matches (768). License is MIT. ONNX export from official model via Hugging Face optimum is one command; do not use the 162-dl community quant. Only cost: full index rebuild (offline) + query prefix injection at encode time. No architecture changes. Risk: "up to 8192 ctx" with max_trained_positions=2048 means long-file chunking behavior needs testing at >2K tokens.

Change 2 — ADD Serena MCP as 4th retrieval channel (LSP):
  FEASIBLE with scope caveat. Serena is MIT, 25k stars, headless-capable, Rust-supported. The integration point is a new retrieval channel in the RRF merge (k=60 already handles variable channel counts). Main risk: Serena adds a Python/Node process dependency and an MCP round-trip per query; latency budget must be measured. If Serena adds too much overhead, direct rust-analyzer stdio (tower-lsp client) is the lighter path — same LSP semantics, pure Rust, no MCP layer.

Change 3 — ADD agentic-grep post-retrieval repair:
  FEASIBLE and LOW-RISK as a post-processing step. DCI (2605.05242) and 2603.20432 both support the direction. The framing in the original report that these two papers "pull opposite directions" is inaccurate — both favor direct/agentic corpus interaction over pure semantic retrieval. Agentic-grep as a repair pass after primary retrieval is additive and reversible. The risk is latency (an LLM-driven grep loop adds 1-3 round trips on cache-miss queries). Wire behind a feature flag.

---

UNVERIFIED
  - ocg crate exact download count and API surface (crates.io page did not render; search-confirmed existence and publish date)
  - leiden-rs exact version/downloads (page did not render)
  - Which specific benchmark dataset (SWE-Bench Lite vs Verified vs Loc-Bench_V1) the LocAgent 92.7% figure refers to
  - Serena's exact rust-analyzer integration path (Rust listed as supported but rust-analyzer not named; likely via the generic LSP client abstraction)
  - max_trained_positions=2048 impact on CodeRankEmbed quality at 2049-8192 token inputs (positional extension not benchmarked in available sources)
