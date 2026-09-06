# Code symbol graph + impact analysis tools (May 2026)

## TL;DR
- ELAI's B-29 plan (tree-sitter SQLite, 27 name shards + 256 SHA1 reverse-call shards, 500-file payload LRU) sits in the right design zone, but the dominant production pattern in 2024-2026 is SCIP precomputed indices fed from compiler-grade indexers (rust-analyzer scip, scip-typescript, scip-python on Pyright). Add a SCIP import path so Sourcegraph-style cross-repo navigation falls into ELAI for free.
- Stack Graphs (GitHub) was archived 2025-09-09 — do not adopt as a dependency, but the underlying paper (arXiv:2211.01224) is still the canonical reference for incremental, zero-config name resolution on tree-sitter CSTs, and its scope/push-pop semantics are the right model for B-30 multi-module resolvers.
- Pure tree-sitter call graphs cannot resolve dynamic dispatch, first-class functions, or callbacks (Chen et al. ACER 2023, still cited 2025). For Python/JS/Ruby ELAI must layer an LSP fallback (Pyright/tsserver) OR import SCIP from the dynamic-language indexer; do not pretend tree-sitter alone is sufficient.
- The 2025 SOTA for "what's affected by my change" in agentic settings is graph-database backed (CodexGraph NAACL 2025, RANGER arXiv:2509.25257, GraphCoder ASE 2024). All three beat embedding/BM25 baselines on repo-level tasks. ELAI's compute_impact_radius is well-aligned but should expose a Datalog/Cypher-style query surface, not just hardcoded BFS, so the auditor can ask richer questions ("callers of X that also touch trait Y").
- An exploratory 2025 study (Preprints.org 202510.0924) found that despite LSP availability, agents still do most retrieval via grep+tree-sitter and that LSP adds tokens without measurable recall gain in the agent setting — so ELAI should treat LSP as a precision layer behind tree-sitter, NOT replace tree-sitter with LSP. The cost is real.

## Key sources
| URL | what | takeaway |
|---|---|---|
| https://github.com/sourcegraph/scip | SCIP protocol repo | Protobuf, human-readable symbol IDs, replaces LSIF; 8x smaller, 3x faster to process than LSIF |
| https://sourcegraph.com/blog/announcing-scip | SCIP intro | Format spec, symbol-string grammar, document/occurrence model |
| https://sourcegraph.com/blog/announcing-scip-typescript | scip-typescript perf | 1k-5k LOC/sec indexing, 3-10x faster than lsif-node, compiler-precision |
| https://sourcegraph.com/blog/scip-python | scip-python | Built on Pyright, 960-2595 LOC/sec measured, type-inference precision |
| https://rust-lang.github.io/rust-analyzer/rust_analyzer/cli/scip/index.html | rust-analyzer scip CLI | Salsa-incremental analysis, batch SCIP emit; PR #19062 fixed module names Feb 2025 |
| https://github.blog/open-source/introducing-stack-graphs/ | Stack Graphs design | Tree-sitter-based, incremental, zero-config name resolution, works for dynamic AND static languages |
| arXiv:2211.01224 | Stack Graphs paper (Creager, GitHub) | Push/pop scope semantics, partial paths per file, composable across repos |
| https://github.com/github/stack-graphs | Status | ARCHIVED 2025-09-09; no official successor; don't depend on it as a library |
| https://github.com/ast-grep/ast-grep | ast-grep | Rust-native tree-sitter wrapper, pattern matching with metavars, complementary to symbol graphs (rewriting, not navigation) |
| https://aider.chat/docs/repomap.html | Aider repo map | Tree-sitter + personalized PageRank; 50x multiplier for chat-file references; token-budgeted |
| https://engineering.fb.com/2024/12/19/developer-tools/glean-open-source-code-indexing/ | Glean (Meta) open-sourced Dec 2024 | Angle query language (Datalog-flavored), facts-as-predicates schema, multi-language |
| https://kythe.io/docs/schema/ | Kythe (Google) | VName-based node identity (lang, sig, corpus, root, path, digest); Rust extractor added 2024; team relocated April 2024 |
| arXiv:2406.07003 | GraphCoder (ASE 2024) | Code Context Graph w/ control+data+control-dep edges; coarse-to-fine retrieval beats sequential RAG |
| arXiv:2408.03910 | CodexGraph (NAACL 2025) | Property graph DB (MODULE/CLASS/FUNCTION nodes, CONTAINS/INHERITS/USES edges); 27.9% EM on CrossCodeEval-Lite, 22.96% Pass@1 SWE-bench-Lite |
| arXiv:2509.25257 | RANGER (Sep 2025) | Repo-level retrieval agent over a comprehensive knowledge graph; handles both entity queries AND NL queries; multi-hop |
| arXiv:2406.14497 | CodeRAG-Bench (NAACL 2025) | Five-source retrieval benchmark; finding: current retrievers still struggle when lexical overlap is low |
| arXiv:2511.00262 | LLM-Driven Cost-Effective Requirements Change Impact Analysis (Nov 2025) | Refinement+Filtering post-processing on LLM-proposed impact sets; satellite-system case study |
| https://www.preprints.org/manuscript/202510.0924 | Exploratory Study of Code Retrieval in Coding Agents (Oct 2025) | 3-layer model (grep/tree-sitter/LSP); LSP added tokens without measurable recall gain in agent loop; Cline 17.5% utilization, LSP-inspired 4.3-6.5% |
| https://github.blog/engineering/architecture-optimization/the-technology-behind-githubs-new-code-search/ | Blackbird (GitHub) | Rust, sharded by Git blob OID (dedup + uniform load); 640 QPS, 120k docs/sec ingest; 45M repos, 115TB |
| https://sourcegraph.com/blog/how-cody-understands-your-codebase | Cody RAG pipeline | Hybrid: keyword + vector + "code graph lookup utilizing precise code intelligence to gather hard links" + rerank |
| https://buck2.build/docs/about/why/ | Buck2 (Meta) | Single incremental dependency graph, no build phases; queryable target graph for impact |
| https://yage.ai/share/why-coding-agents-still-use-grep-en-20260327.html | March 2026 analysis | Empirical: agents converge on grep+tree-sitter as default; LSP is precision-only |

## Tool landscape
| tool | language coverage | dynamic-dispatch resolution | indexing speed | query latency | rust-portable | citation |
|---|---|---|---|---|---|---|
| SCIP (Sourcegraph) | 9+ via per-lang indexers | depends on indexer (Pyright/tsserver = good) | scip-ts 1-5k LOC/s, scip-py 1-2.6k LOC/s | precomputed -> ms | yes (scip crate on crates.io) | sourcegraph.com/blog/announcing-scip |
| rust-analyzer scip | Rust only | static, compiler-precision | Salsa-incremental | ms (Salsa cache) | yes (native Rust) | rust-lang.github.io/rust-analyzer scip |
| Stack Graphs | 5+ tree-sitter grammars | language-defined push/pop rules — dynamic OK | seconds per commit incremental | ms per query | yes (Rust crate, but archived) | github.blog introducing-stack-graphs |
| ast-grep | 12+ tree-sitter | NO — pattern matching only | fast (no semantics) | grep-class | yes (native Rust) | github.com/ast-grep/ast-grep |
| Aider repo map | 12+ tree-sitter | NO — defs/refs only | seconds per repo | per-conversation rebuild | partial (Python) | aider.chat/docs/repomap |
| LSP (tsserver/Pyright/rust-analyzer) | per-language | YES — full type inference | seconds-minutes warm-up | ms steady-state | via subprocess only | preprints 202510.0924 |
| Kythe | C++, Java, Go, Rust (extractor added 2024) | static, compiler-grade | batch, slow | depends on storage backend | partial (Go server) | kythe.io |
| Glean | C++, Hack, Python, others | static + custom rules | batch via "fact production" | Angle (Datalog) ms | Haskell server | engineering.fb.com glean-open-source |
| ELAI B-29 (planned) | 13+ via B-30 | NO (current plan) | unmeasured | p99 <=500ms target (B-29.5) | yes | (this codebase) |

## LSP vs tree-sitter vs SCIP vs Stack-Graphs

When **tree-sitter** wins: multi-language coverage, no per-project config, parsing of broken/incomplete code (mid-edit), pure speed. ELAI's current path. Penalty: zero semantic understanding — cannot distinguish two methods named `process` on different classes, cannot follow `obj.method()` to the right class, cannot resolve `import * from` re-exports. (preprints 202510.0924; ACER 2023 noted in 2025 callgraph survey).

When **SCIP** wins: precomputed cross-repo navigation, stable symbol IDs across versions, deterministic facts the auditor can replay, 8x smaller / 3x faster than LSIF. Specifically beats tree-sitter when the codebase has a working build/type-check (otherwise SCIP indexers degrade or fail). Cross-language: SCIP is one format, multiple indexers — ELAI's B-30 13-grammar approach can be re-cast as "13 SCIP importers" without reinventing the schema. (Sourcegraph announcing-scip; rust-analyzer scip docs).

When **Stack Graphs** wins: incremental name resolution across files without a build system, works on tree-sitter CSTs ELAI already has, handles dynamic languages via language-specific scope rules. Penalty: project archived 2025-09-09 — adopt the design, not the dependency. Re-implementing the push/pop scope-graph model in Rust on top of existing tree-sitter queries is the right B-30 north star. (arXiv:2211.01224; github.com/github/stack-graphs archive notice).

When **LSP** wins: ground-truth precision for dynamic-dispatch / virtual / generic / first-class-function call sites in Python/TS/JS/Ruby. Measured 900x faster than text grep for "find all 23 real call sites" (claude code LSP 2.0.74 docs, Dec 2025). Penalty: per-language daemon, cold-start cost, and Preprints 202510.0924 found agents in a real loop did not extract recall gains commensurate with token cost. **Verdict: keep LSP as an on-demand precision tool, not the primary index.**

## Impact-radius for agentic edits

SOTA 2024-2026:

- **GraphCoder (ASE 2024, arXiv:2406.07003)**: builds a Code Context Graph with control-flow + data-dep + control-dep edges per statement, retrieves coarse-to-fine. Beats sequential RAG. ELAI's hop-banded BFS captures part of this; adding data-flow edges (not just call edges) is a clear extension.
- **CodexGraph (NAACL 2025, arXiv:2408.03910)**: nodes = MODULE/CLASS/FUNCTION, edges = CONTAINS/INHERITS/USES; LLM emits property-graph queries. 22.96% Pass@1 on SWE-bench-Lite. The "agent emits a query DSL" pattern is more flexible than ELAI's 4 hardcoded tools.
- **RANGER (arXiv:2509.25257, Sep 2025)**: explicit knowledge-graph agent for repo-level retrieval, multi-hop, hierarchical edges preserved. Handles NL queries in addition to entity queries. Closest published analog to ELAI's compute_symbol_context + impact_radius combo.
- **LLM-Driven Cost-Effective Requirements Change Impact Analysis (arXiv:2511.00262, Nov 2025)**: post-processing of LLM-proposed impact sets via Refinement+Filtering steps. Pattern: don't trust the LLM's first impact-set; deterministic filter against the symbol graph.
- **FocalAD (arXiv:2506.11419, Jun 2025)**: the cited title is autonomous-driving (ego-local-agents interactor); ELAI's prior citation of FocalAD as an impact-analysis paper appears to be a mis-cite. Note for record: the actual relevant arXiv tag for "focal" impact in code-change literature is GraphCoder/CodexGraph, not 2506.11419. **Verify before further citation.**

Recommendations for ELAI B-29:
1. Keep the hop-banded BFS as the **default** impact_radius tool (cheap, deterministic).
2. Add a second tool `compute_impact_radius_dataflow` that follows def-use edges (B-29.5 has the budget per the p99<=500ms target). GraphCoder's evidence is that data-dep edges meaningfully change retrieval quality.
3. Expose a query DSL surface (Angle-style or Datalog-style; Glean is the reference) instead of locking in 4 hardcoded BFS shapes. Auditor's "callers of X that also touch Y" cannot be expressed in the current plan.
4. Apply the arXiv:2511.00262 Refinement+Filter pattern: when the agent proposes "I will change X", let the deterministic symbol graph FILTER the proposal — reject if claimed-unaffected files are downstream of X.

## Best additions for ELAI

Ranked:

1. **SCIP import path for rust-analyzer's scip emit** | grounded in rust-analyzer scip CLI shipping today + B-29 SymbolGraphCache already shards by name | hook: convert SCIP Documents to the existing 27-shard layout via a one-time importer; treat SCIP as the source of truth for Rust, keep tree-sitter as fallback/non-Rust path. Cost: one new crate, ~1k LOC. Wins compiler-grade Rust call-graph precision essentially for free, and gives ELAI a path to cross-repo navigation later.

2. **Stack-Graph scope/push-pop semantics ported into B-30 multi-module resolvers** | grounded in arXiv:2211.01224 + the archived but still-readable github/stack-graphs Rust crate | hook: replace the planned "JVM suffix map / C# namespaces / Go modules / TS path aliases" ad-hoc resolvers with a unified scope-graph layer. One model, 13+ language adapters. Lower long-term maintenance than per-language switches.

3. **Property-graph query DSL on top of SymbolGraphCache** | grounded in CodexGraph (NAACL 2025), Glean's Angle, RANGER (Sep 2025) | hook: expose to the auditor an Angle/Cypher-lite read interface so agents can ask multi-hop questions. Current 4-tool surface is too narrow. Implementation: thin Datalog evaluator over the existing 27+256 shards.

4. **Data-flow edges, not just call edges, in impact_radius** | grounded in GraphCoder (ASE 2024) | hook: extend B-29's edge model to include def-use chains for Rust (cheap via SCIP occurrences) and tree-sitter heuristic edges for other languages. Keep depth=2 default for the Auditor PostToolUse summary so cost stays bounded.

5. **LSP as on-demand precision oracle, NOT primary index** | grounded in Preprints 202510.0924 (Oct 2025), Claude Code LSP 2.0.74 (Dec 2025) | hook: spawn rust-analyzer/Pyright/tsserver only when impact_radius hits an "Unresolved" symbol that affects audit verdict. Cache results in the 500-file payload LRU. Keeps the steady-state cost off the hot path.

6. **Symbol-graph as RAG signal alongside embeddings, not instead of** | grounded in Sourcegraph Cody architecture + CodeRAG-Bench (NAACL 2025) finding that pure embedding retrieval has low recall when lexical overlap is low | hook: when the executor asks for context, return (embedding top-k) UNION (symbol-graph hop-2 neighbors of files mentioned). Sourcegraph's three-layer (keyword + embedding + code-graph + rerank) is the production-tested template.

7. **PageRank-style file ranking for the Auditor's PostToolUse summary** | grounded in Aider repo map (still cited as the reference 2024-2026) | hook: precompute personalized PageRank on the call graph with the changed file as the personalization vector; surface top-k files in the audit summary. Aider's 50x chat-file multiplier is a battle-tested heuristic.

8. **Buck2/Bazel-style "queryable target dependency graph" mental model for the workspace** | grounded in Buck2 docs + Bazel docs | hook: even without adopting Bazel, mirror the "single incremental dependency graph, no phases" architecture in B-29 — the SymbolGraphCache should be the one graph all auditor queries touch. Avoid parallel-but-different graphs in the dashboard vs. CLI.

## Anti-patterns
- **Tree-sitter alone as the full call-graph source for dynamic languages** — ACER 2023 (cited in 2025) and the Volito 2024 tree-sitter call-graph guide both note that without alias analysis, dynamic dispatch / first-class functions / function pointers are unresolvable. Don't ship B-30 claiming Python/JS call-graph parity with Rust. Mark these as `Unresolved` per ELAI's confidence tier and escalate to LSP/SCIP-python.
- **Treating Stack Graphs as a live dependency** — archived 2025-09-09. Adopt the design (push/pop scope rules) from arXiv:2211.01224, but do NOT add the crate as a runtime dependency.
- **LSP as the default symbol index in an agent loop** — Preprints 202510.0924 measured token-overhead without measurable recall gain; Cline's grep+tree-sitter+fzf approach hit the same task completion at lower utilization. LSP is a precision tool, not a primary index.
- **Embedding-only RAG for repo-level code tasks** — CodeRAG-Bench (NAACL 2025) finding: low recall when lexical overlap is low. Hybrid (symbol + embedding) wins. Sourcegraph Cody's three-layer pipeline is the production proof.
- **Hardcoding the agent's query surface to N traversal kernels** — CodexGraph and RANGER both show the agent benefits from a query DSL. Lock-in to "4 tools" forces every new audit question into a new tool implementation.
- **Re-mapping SCIP to a homegrown symbol-string scheme** — SCIP's symbol-string grammar (local + global, package-qualified) is standardized; reusing it gives ELAI cross-tool interop (Sourcegraph, scip-clang, scip-ruby, vscode-scip-navigation). Inventing a new scheme costs interop.

## Open questions
1. **Measured break-even**: at what repo size does SCIP-import outperform tree-sitter-only for Rust on ELAI's auditor PostToolUse latency budget (depth=2 impact summary, p99<=500ms target from B-29.5)? Run B-29.5 against both backends on the same 4 corpora (Linux v6.7, Chromium, VS Code, Postgres).
2. **Dynamic-dispatch recall**: on the ELAI Python sidecar (python/elai_inference/), what % of `model.method()` call sites does tree-sitter resolve correctly vs. Pyright/scip-python on the same source? Benchmark before deciding LSP-on-demand vs. SCIP-batch.
3. **Query DSL adoption cost**: if we ship an Angle-lite / Datalog evaluator over SymbolGraphCache, what % of current auditor questions are expressible (target >=90% from prior auditor logs)?
4. **PageRank stability**: does Aider's personalized PageRank produce deterministic top-k across 5 runs on the same change-set when ranks are tied? B-29.5 already requires hop-band determinism — PageRank ties may need a stable tiebreaker (e.g., symbol-string lexicographic).
5. **FocalAD citation**: the previously-cited arXiv:2506.11419 ("FocalAD") is autonomous-driving (Ego-Local-Agents Interactor). The ELAI plan's citation of FocalAD-as-impact-analysis appears mis-attributed. What is the actually-intended reference? Verify against the original B-29 design notes before re-citing.
