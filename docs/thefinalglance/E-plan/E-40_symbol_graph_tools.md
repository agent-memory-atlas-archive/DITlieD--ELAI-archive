# E-40: Symbol graph augmentations — SCIP import path, Stack-Graph scope semantics, Datalog query DSL, data-flow edges, LSP-as-precision-oracle (NOT primary), FocalAD mis-citation flag
Objective: Augment ELAI's B-29 Symbol-Level Impact Analysis API (PLANNED, ★ highest leverage per B-PLAN-SUMMARY) with the 2024-2026 SOTA patterns the report ranks 1-8: SCIP import for compiler-grade Rust precision, port Stack-Graphs push/pop scope semantics for B-30 multi-module resolution (Stack-Graphs ARCHIVED Sep 2025 — adopt design only), expose a Datalog/Cypher-lite query DSL on SymbolGraphCache, add data-flow edges to impact_radius, treat LSP as on-demand precision (NOT primary), hybrid symbol+embedding RAG signal, personalized PageRank for auditor PostToolUse. Explicitly surface the FocalAD arXiv:2506.11419 mis-citation in the report's record.
Estimated effort: 3 sessions, ~10-14 h (largest E-N in this assigned set; SCIP import + DSL are heaviest)
Pre-requisites: B-29 PLANNED (Symbol-Level Impact Analysis API — `compute_impact_radius`, `compute_symbol_context`, hop-banded BFS, depth-clamp 10, 27 name shards + 256 SHA1 reverse-call shards, 500-file payload LRU; per B-PLAN-SUMMARY line 571), B-29.5 PLANNED (Symbol Graph Scale Benchmark — p99 ≤ 500ms target; per B-PLAN-SUMMARY line 592), B-30 PLANNED (Multi-Language Symbol Extractors; per B-PLAN-SUMMARY), C-24 PLANNED (Symbol Impact Dashboard Surface; per C-SUMMARY_INDEX line 314 — 4 ToolDefinition entries + KnowledgeDrawer IMPACT ANALYSIS panel)
Parallel-safe with: E-36 (different concern), E-37 (different concern), E-38 (different concern — but E-38 audit-trail liveness gate composes with E-40's query DSL audit logs), E-39 (different concern)
Source report: workflow/improvement-sweep-2026-05-19/reports/40-symbol-graph-tools.md

## Context

ELAI's existing symbol graph surface (per elai.map lines 4527/4634/4637/5167/6138/26747 for `SymbolGraphCache::new`, `compute_impact_radius`, `compute_symbol_context`) sits at line 149 of `crates/knowledge/src/symbol_graph/traversal.rs` per elai.map line 26747 and is already SCIP-enriched (the codebase map's SCIP_REF tags come from rust-analyzer scip + scip-python + scip-typescript per `.claude/tools/scip_indices`). The report's verdict is precise: ELAI's B-29 plan (tree-sitter SQLite, 27 name shards + 256 SHA1 reverse-call shards, 500-file payload LRU, hop-banded BFS) sits in the right design zone, but the dominant 2024-2026 production pattern is SCIP precomputed indices from compiler-grade indexers — and there are 7 ranked augmentations beyond that floor.

The 8 ranked best additions from the report:
1. SCIP import path for rust-analyzer's `scip` emit — compiler-grade Rust precision; ~1k LOC new crate; converts SCIP Documents to ELAI's existing 27-shard layout.
2. Stack-Graph scope/push-pop semantics ported into B-30 multi-module resolvers — archived 2025-09-09 but the design (arXiv:2211.01224) is still the canonical reference for incremental, zero-config name resolution on tree-sitter CSTs. DO NOT add the crate as a runtime dependency.
3. Property-graph query DSL on top of SymbolGraphCache (Angle/Cypher-lite) — CodexGraph NAACL 2025 + Glean's Angle + RANGER arxiv:2509.25257. Replaces 4 hardcoded BFS shapes with a flexible read interface.
4. Data-flow edges, not just call edges, in impact_radius — GraphCoder ASE 2024 (arxiv:2406.07003) evidence.
5. LSP as on-demand precision oracle, NOT primary index — Preprints 202510.0924 + Claude Code LSP 2.0.74 Dec 2025; agents in real loops do most retrieval via grep+tree-sitter; LSP adds tokens without measurable recall gain in agent settings.
6. Symbol-graph as RAG signal alongside embeddings, not instead of — CodeRAG-Bench NAACL 2025 + Sourcegraph Cody three-layer pipeline.
7. PageRank-style file ranking for the auditor's PostToolUse summary — Aider repo map; 50x chat-file multiplier.
8. Buck2/Bazel-style queryable target dependency graph mental model — Buck2 single incremental graph.

The FocalAD mis-citation (report Open Question 5; report explicitly flags this — "the previously-cited arXiv:2506.11419 ('FocalAD') is autonomous-driving (Ego-Local-Agents Interactor); ELAI's prior citation of FocalAD as an impact-analysis paper appears to be a mis-cite"): E-40 surfaces this in workflow/reasoning/E-40-decisions.md AND in any place B-29's design docs cite FocalAD. The actual relevant code-change impact-analysis references are GraphCoder ASE 2024 (arxiv:2406.07003) and CodexGraph NAACL 2025 (arxiv:2408.03910), not FocalAD. Per CLAUDE.md "Verify paper citations" rule (memory:feedback_paper_citation_verify): "WebSearch arXiv IDs before citing; effect-size numbers need quote+page-anchor."

The structural delta of E-40 vs B-29 is COMPOSITIONAL — every task here is an augmentation of B-29's planned surface, not a replacement. The 4 hardcoded BFS tools (impact_radius / symbol_context / symbol_flow / symbol_list per B-29 + C-24 lines 232 + 314) stay; the DSL is layered ON TOP as a read interface for agent flexibility. The hop-banded BFS stays as the DEFAULT for cheap deterministic queries; data-flow edges are an OPTIONAL second pass with the B-29.5 p99 ≤ 500ms budget. SCIP is the source of truth for Rust; tree-sitter is the fallback + non-Rust path.

## Existing state (B/C/D coverage)

- B-29 PLANNED (Symbol-Level Impact Analysis API — ★ highest leverage): per B-PLAN-SUMMARY line 571. EXTENSION: E-40 augments the planned design with SCIP import (Task 1), data-flow edges (Task 4), DSL surface (Task 3). The hop-banded BFS + 27 name shards + 256 reverse-call shards + 500-file payload LRU stay AS-IS — additive only.
- B-29.5 PLANNED (Symbol Graph Scale Benchmark — p99 ≤ 500ms): per B-PLAN-SUMMARY line 592. EXTENSION: E-40 adds 4 corpora to the bench harness (Linux v6.7, Chromium, VS Code, Postgres per report Open Question 1) measuring SCIP-import vs tree-sitter-only.
- B-30 PLANNED (Multi-Language Symbol Extractors): per B-PLAN-SUMMARY line 1040 (B-30 makes B-29 multi-language). REPLACEMENT of design intent: E-40 Task 2 replaces the ad-hoc "JVM suffix map / C# namespaces / Go modules / TS path aliases" resolvers with a unified Stack-Graph-style push/pop scope-graph layer. ONE model, 13+ language adapters. Lower long-term maintenance.
- C-24 PLANNED (Symbol Impact Dashboard Surface): per C-SUMMARY_INDEX line 314. EXTENSION: E-40 adds 2 new ToolDefinition entries to C-24's planned 4 (`symbol_dsl_query` + `symbol_dataflow_impact`); KnowledgeDrawer surfaces DSL query results.
- C-22 PLANNED (FeasibilityProber completeness): per C-SUMMARY_INDEX. RELATED: C-22 consumes B-29 symbol context; if the DSL ships, C-22's feasibility checks can express richer queries.
- C-30 PLANNED (Surgical-mode DKG tooling): per D-SUMMARY_INDEX line 301. RELATED: D-11 consumes C-30 surgical DKG; the DSL surface here composes with surgical-mode queries.
- D-11 PLANNED (Constructive-checker registry): per D-SUMMARY_INDEX line 86. RELATED: api/ + code/ checkers dispatch via symbol_impact (C-24) + symbol_dsl_query (E-40 new).
- LSP integration: NOT currently in B-29 / B-30 plans per report finding. E-40 Task 5 introduces LSP-as-on-demand-precision, NOT primary. Caches in the existing 500-file payload LRU per B-29 plan.
- elai.map already SCIP-enriched (per CLAUDE.md "Codebase map" block: SCIP_REF/SCIP_DEF rows carry fully-resolved canonical paths from rust-analyzer / scip-python / scip-typescript): CONFIRMS Task 1 SCIP-import is grounded — ELAI is already running these indexers offline; Task 1 wires the runtime path.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\40-symbol-graph-tools.md` (full — 108 lines)
- `F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\symbol_graph\traversal.rs` (existing `compute_impact_radius` at line 149 per elai.map line 26747; `compute_symbol_context` at the same file)
- `F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\symbol_graph\cache.rs` (existing `SymbolGraphCache` — pattern reference; per elai.map lines 4527 / 5167)
- `F:\.ELAI_workflow\.elai_cc\crates\knowledge\tests\symbol_graph_traversal_tests.rs` (existing test patterns per elai.map lines 1692/2702/5422)
- `F:\.ELAI_workflow\.elai_cc\crates\knowledge\tests\scale_determinism.rs` (existing perf bench per elai.map lines 4527/4637/6138)
- `F:\.ELAI_workflow\.elai_cc\crates\knowledge\benches\workload.rs` (existing workload bench per elai.map lines 6493/6494)
- `F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\symbol_graph_b29_phase3_contract.rs` (existing B-29 phase 3 contract per elai.map line 27150)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 571-700 (B-29 + B-29.5 + B-30 detail)
- `F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-SUMMARY_INDEX.md` lines 314-330 (C-24 dashboard surface)
- `F:\.ELAI_workflow\.claude\tools\elai.map` (codebase map; SCIP-enriched; reference for the SCIP import shape)
- github.com/sourcegraph/scip — WebFetch (SCIP protocol spec + Rust crate)
- sourcegraph.com/blog/announcing-scip — WebFetch (format spec, symbol-string grammar)
- rust-lang.github.io/rust-analyzer/rust_analyzer/cli/scip — WebFetch (rust-analyzer scip CLI)
- arxiv:2211.01224 Stack Graphs paper — WebFetch via paper-ingestor mode=paper
- arxiv:2406.07003 GraphCoder ASE 2024 — WebFetch via paper-ingestor mode=paper
- arxiv:2408.03910 CodexGraph NAACL 2025 — WebFetch via paper-ingestor mode=paper
- arxiv:2509.25257 RANGER Sep 2025 — WebFetch via paper-ingestor mode=paper
- arxiv:2511.00262 LLM-Driven Cost-Effective Requirements Change Impact Analysis Nov 2025 — WebFetch via paper-ingestor
- preprints.org/manuscript/202510.0924 Exploratory Study of Code Retrieval in Coding Agents — WebFetch
- aider.chat/docs/repomap.html (PageRank pattern, 50x chat-file multiplier) — WebFetch
- github.blog/open-source/introducing-stack-graphs — WebFetch (Stack Graphs design)
- yage.ai/share/why-coding-agents-still-use-grep-en-20260327.html — WebFetch
- engineering.fb.com/2024/12/19/developer-tools/glean-open-source-code-indexing — WebFetch (Glean Angle DSL)

## Tasks

1. SCIP import path for rust-analyzer's `scip` emit. New crate `crates/scip-import/` (~700 LOC across 4 modules: `lib.rs`, `document.rs`, `symbol_table.rs`, `to_shards.rs`). Reads a `.scip` protobuf file produced by `rust-analyzer scip` (CLI exists today; PR #19062 fixed module names Feb 2025). Converts SCIP Documents to ELAI's existing 27-shard SymbolGraphCache layout (`document.rs` parses; `symbol_table.rs` maps SCIP symbol-strings to ELAI's 27-shard hash; `to_shards.rs` writes shards). Add `elai index --scip <path>` CLI. SCIP becomes the SOURCE OF TRUTH for Rust; tree-sitter stays as fallback for files not in the SCIP corpus + all non-Rust files. Anti-slopsquatting: VERIFY `scip` crate on crates.io (the report cites "scip crate on crates.io") + version + last-publish date before adding to deps. Reference: github.com/sourcegraph/scip + sourcegraph.com/blog/announcing-scip — 8x smaller / 3x faster than LSIF; symbol-string grammar standardized.

2. Stack-Graph push/pop scope semantics ported into B-30 multi-module resolvers. New module `crates/knowledge/src/symbol_graph/scope_graph.rs` (~400 LOC). Implements the push/pop scope-graph model from arxiv:2211.01224 in Rust on top of ELAI's existing tree-sitter queries. The GitHub `github/stack-graphs` Rust crate is ARCHIVED Sep 2025 — DO NOT add as a runtime dependency. Re-implement the design: file-local partial paths + cross-file scope-stack join + language-specific push/pop rules in TOML at `.elai_cc/scope_graph_rules/{language}.toml`. 13+ language adapters (per B-30 PLANNED) collapse to one model. Reference: arxiv:2211.01224 + github.com/github/stack-graphs (archive notice as design reference).

3. Property-graph query DSL on top of SymbolGraphCache. New module `crates/knowledge/src/symbol_graph/dsl.rs` (~500 LOC). Angle/Cypher-lite syntax: `MATCH (caller)-[:CALLS]->(target {name: "X"}) WHERE caller.file MATCHES "*/auth/*" RETURN caller`. Implemented as a thin Datalog evaluator over the existing 27+256 shards. Compiles a DSL query to a sequence of shard lookups; uses hop-banded BFS as the BFS primitive. New CLI: `elai symbol query '<dsl>'`. Agent ToolDefinition: `symbol_dsl_query` joins C-24's existing 4 BFS tools. The "callers of X that also touch trait Y" question from the report is expressible. Reference: CodexGraph (arxiv:2408.03910) "agent emits a property-graph query DSL" pattern; Glean's Angle (engineering.fb.com/glean-open-source-code-indexing); RANGER (arxiv:2509.25257) multi-hop knowledge-graph agent.

4. Data-flow edges in impact_radius. New module `crates/knowledge/src/symbol_graph/dataflow.rs` (~280 LOC). For Rust: cheap via SCIP occurrences (def/ref pairs are already in the SCIP file from Task 1). For non-Rust (tree-sitter heuristic): tree-sitter query patterns matching common def-use templates per language. New tool: `compute_impact_radius_dataflow(cache, target, depth)` — separate from the existing `compute_impact_radius` (which is call-edges only). Default depth=2 for auditor PostToolUse summary to stay within B-29.5 p99 ≤ 500ms budget. Reference: GraphCoder ASE 2024 (arxiv:2406.07003) — Code Context Graph with control-flow + data-dep + control-dep edges; coarse-to-fine retrieval beats sequential RAG.

5. LSP-as-on-demand-precision oracle. New module `crates/knowledge/src/symbol_graph/lsp_oracle.rs` (~200 LOC). Triggers ONLY when `compute_impact_radius` or `compute_symbol_context` returns a result marked `confidence: Unresolved` AND that result affects an audit verdict. Spawns rust-analyzer / Pyright / tsserver as subprocess via the existing PerRoleRuntime (B-37 — new role `lsp_oracle`). Caches resolved positions in the existing 500-file payload LRU (B-29). The LSP is NEVER the primary index; this is the report's recommendation #5 ("Verdict: keep LSP as an on-demand precision tool, not the primary index"). Per Preprints 202510.0924: agents in real loops did not extract recall gains commensurate with LSP token cost. Reference: preprints.org/manuscript/202510.0924 + Claude Code LSP 2.0.74 Dec 2025 (900x faster than text grep for "find all 23 real call sites").

6. Refinement+Filter pattern on agent-proposed impact sets. New module `crates/knowledge/src/symbol_graph/refine_filter.rs` (~120 LOC). When an agent proposes "I will change X and the impact is limited to files [Y, Z]", the deterministic symbol graph FILTERS the proposal: walk impact_radius(X, depth=∞ bounded by depth-clamp 10) and reject if any file downstream of X is NOT in the agent's claimed list. Emit `agent_impact_claim_refuted` SSB row with the missing files. Reference: arxiv:2511.00262 LLM-Driven Cost-Effective Requirements Change Impact Analysis Nov 2025 — pattern: don't trust the LLM's first impact-set; deterministic filter against the symbol graph.

7. Symbol-graph as RAG signal alongside embeddings. Extend the E-17 RRF pipeline (existing 3-list / 4-list with vision per E-36) by ADDING a `symbol_graph_neighbors` retriever: for any file mentioned in the agent's context query, return all files within hop-2 of that file's defined symbols. Fuse into the RRF as a 5th list (or 4th if vision disabled per E-36 policy). Configurable weight `[knowledge.rrf] weights = {bm25 = 1.0, code_vec = 1.0, text_vec = 0.8, vision_vec = 0.6, symbol_graph_vec = 0.9}`. Reference: Sourcegraph Cody three-layer pipeline (keyword + embedding + code-graph + rerank) per sourcegraph.com/blog/how-cody-understands-your-codebase; CodeRAG-Bench NAACL 2025 (arxiv:2406.14497) low-lexical-overlap finding.

8. Personalized PageRank for auditor PostToolUse file ranking. New module `crates/knowledge/src/symbol_graph/pagerank.rs` (~180 LOC). Precompute personalized PageRank on the call graph with the changed file as the personalization vector. Surface top-k (default k=10) files in the auditor's PostToolUse summary. Determinism: stable tiebreaker via symbol-string lexicographic ordering (per B-29.5 hop-band determinism requirement; per report Open Question 4). Reference: Aider repo map (aider.chat/docs/repomap.html) — 50x chat-file multiplier; battle-tested 2024-2026 heuristic.

9. Dashboard surface — C-24 extension. Add 2 new ToolDefinition entries to `crates/orchestrator/src/task_runner.rs::build_tool_catalog` (per C-24 design): `symbol_dsl_query` + `symbol_dataflow_impact`. Wire DSL query input form to the KnowledgeDrawer IMPACT ANALYSIS panel (C-24 PLANNED). Show: data-flow impact_radius results; DSL query results table; PageRank top-k for the most-recent change; LSP oracle invocation count (should be LOW — anti-pattern to spam LSP). Tauri commands `get_symbol_dsl_recent`, `get_dataflow_impact_recent`, `get_pagerank_top_k`, `get_lsp_oracle_stats`.

10. FocalAD mis-citation flag. Author `workflow/reasoning/E-40-decisions.md` with explicit "Citation Errata" section: arXiv:2506.11419 is autonomous-driving (Ego-Local-Agents Interactor for ego-vehicle planning), NOT code-change impact analysis. ELAI's prior citation of FocalAD-as-impact-analysis is INCORRECT. The actual references for code-change impact analysis are GraphCoder ASE 2024 (arxiv:2406.07003) + CodexGraph NAACL 2025 (arxiv:2408.03910) + RANGER (arxiv:2509.25257) + the Nov 2025 RCIA paper (arxiv:2511.00262). Update any existing B-29 plan documents that cite FocalAD-as-impact-analysis. Per memory:feedback_paper_citation_verify — "WebSearch arXiv IDs before citing; effect-size numbers need quote+page-anchor; 2 bad cites in B-40/B-42 plans." This is the third bad cite flag.

## WIRING_CONTRACT

produces_symbols:
  - scip_import::ScipImporter (new crate)
  - scip_import::Document, scip_import::SymbolEntry (new types)
  - scip_import::import_to_shards(scip_path: &Path, cache: &mut SymbolGraphCache) -> Result<ImportReport> (new fn)
  - knowledge::symbol_graph::scope_graph::ScopeGraph (new struct)
  - knowledge::symbol_graph::scope_graph::resolve_name(query: &NameQuery) -> Vec<Resolution> (new fn)
  - knowledge::symbol_graph::dsl::DslQuery (new struct; parser)
  - knowledge::symbol_graph::dsl::execute(query: &DslQuery, cache: &mut SymbolGraphCache) -> Vec<DslResult> (new fn)
  - knowledge::symbol_graph::dataflow::compute_impact_radius_dataflow(cache: &mut SymbolGraphCache, target: &str, depth: u8) -> Result<DataflowImpact> (new fn)
  - knowledge::symbol_graph::lsp_oracle::resolve_unresolved(cache: &mut SymbolGraphCache, unresolved: &[UnresolvedRef]) -> Result<Vec<Resolution>> (new fn)
  - knowledge::symbol_graph::refine_filter::refute_claim(cache: &mut SymbolGraphCache, claim: &AgentImpactClaim) -> RefutationVerdict (new fn)
  - knowledge::symbol_graph::pagerank::personalized_top_k(cache: &mut SymbolGraphCache, changed_file: &Path, k: usize) -> Vec<(PathBuf, f64)> (new fn)
  - cli::index::scip_subcommand (`elai index --scip <path>`)
  - cli::symbol::query_subcommand (`elai symbol query '<dsl>'`)
  - SsbActionType::ScipImported (new)
  - SsbActionType::ScopeGraphResolved (new)
  - SsbActionType::SymbolDslExecuted (new)
  - SsbActionType::DataflowImpactComputed (new)
  - SsbActionType::LspOracleInvoked (new — count surfaced in dashboard for anti-spam discipline)
  - SsbActionType::AgentImpactClaimRefuted (new)
  - SsbActionType::PagerankComputed (new)
consumes_symbols:
  - knowledge::symbol_graph::cache::SymbolGraphCache (B-29 PLANNED; per elai.map line 4527)
  - knowledge::symbol_graph::traversal::compute_impact_radius (B-29 PLANNED; per elai.map line 26747)
  - knowledge::symbol_graph::traversal::compute_symbol_context (B-29 PLANNED; per elai.map line 18734)
  - knowledge::pipeline::run_query (E-17 RRF pipeline; symbol_graph_vec slot adds via Task 7)
  - orchestrator::runtime_orchestrator::PerRoleRuntime (B-37 SHIPPED; `lsp_oracle` role registers here)
  - orchestrator::task_runner::build_tool_catalog (C-24 PLANNED; 2 new ToolDefinition entries register here)
  - scip crate on crates.io (anti-slopsquatting verification mandatory before adding)
reachable_from:
  - crates/cli/src/main.rs (`elai index --scip <path>`, `elai symbol query '<dsl>'`)
  - crates/orchestrator/src/task_runner.rs::knowledge_query (Task 7 RRF 5th-list integration)
  - crates/orchestrator/src/task_runner.rs::auditor_post_tool_use (Task 8 PageRank top-k surfacing)
  - crates/dashboard-v2/src-tauri/src/commands.rs (4 new Tauri commands per Task 9)
  - crates/contracts/tests/symbol_graph_b29_phase3_contract.rs (existing; extend with E-40 contracts — DO NOT replace per CLAUDE.md "MODIFY existing" pattern)
stub_ban:
  - no homegrown SCIP symbol-string scheme (use SCIP's standardized grammar — interop cost otherwise; report anti-pattern flag)
  - no Stack-Graphs runtime dependency (archive notice — design adoption only)
  - no LSP as primary index (anti-pattern per Preprints 202510.0924; on-demand precision only)
  - no tree-sitter-only claim for dynamic-language call-graph parity (ACER 2023 flag; mark Unresolved + escalate to LSP/SCIP-python)
  - no embedding-only RAG for repo-level code tasks (CodeRAG-Bench finding)
  - no hardcoded query surface beyond the existing 4 BFS tools + 2 new (Task 9) — DSL is the flexibility surface
  - no FocalAD-as-impact-analysis citation anywhere in new code or docs (report Open Question 5 — Task 10 flag)
  - no .unwrap() in production symbol-graph paths (CLAUDE.md HARD RULE)
frontend_pair: extends existing C-24 KnowledgeDrawer IMPACT ANALYSIS panel + adds Symbol DSL Query input form
observable_effect:
  trigger: "elai /task --mode hybrid 'rename the SsbActionType enum and update all callers'" (existing C-24 smoke per C-SUMMARY_INDEX line 326)
  expect_ssb: "ScopeGraphResolved (B-30 unified resolver via Task 2); DataflowImpactComputed (Task 4, data-flow edges); SymbolDslExecuted (Task 3, agent self-selects DSL tool); PagerankComputed (Task 8, top-k file ranking); AgentImpactClaimRefuted if agent claims a too-narrow impact set"
  expect_artifact: "scip_import index built on Rust crates; query 'MATCH (c)-[:CALLS]->(t {name: \"SsbActionType\"}) RETURN c' returns all caller files; PageRank top-10 files visible in dashboard PostToolUse summary"

## DETERMINISTIC_CHECK

```bash
# 1. scip-import crate exists with 4 modules
test -f .elai_cc/crates/scip-import/Cargo.toml
test -f .elai_cc/crates/scip-import/src/lib.rs
test -f .elai_cc/crates/scip-import/src/document.rs
test -f .elai_cc/crates/scip-import/src/symbol_table.rs
test -f .elai_cc/crates/scip-import/src/to_shards.rs
# Expected: all 5 exist

# 2. Scope-graph module + per-language TOML rules
rg "ScopeGraph::resolve_name|PushScope|PopScope" .elai_cc/crates/knowledge/src/symbol_graph/scope_graph.rs
# Expected: ≥3 hits
ls .elai_cc/scope_graph_rules/ | wc -l
# Expected: ≥4 languages (e.g., rust.toml, python.toml, typescript.toml, go.toml)

# 3. DSL parser + executor
rg "DslQuery|execute.*DslQuery|MATCH.*WHERE.*RETURN" .elai_cc/crates/knowledge/src/symbol_graph/dsl.rs
# Expected: ≥3 hits

# 4. Data-flow tool exists separately from call-edge tool
rg "compute_impact_radius_dataflow|DataflowImpact" .elai_cc/crates/knowledge --type rust
# Expected: ≥3 hits (impl + struct + usage)

# 5. LSP oracle is on-demand only, not primary
rg "lsp_oracle::resolve_unresolved" .elai_cc/crates/knowledge --type rust
# Expected: ≥1 hit; called only from Unresolved-confidence path
rg "Lsp.*primary|primary_index.*lsp|LSP.*default" .elai_cc/crates/knowledge --type rust
# Expected: zero hits (anti-pattern guard)

# 6. PageRank stable tiebreaker
rg "personalized_top_k|PageRank.*tiebreaker" .elai_cc/crates/knowledge/src/symbol_graph/pagerank.rs
# Expected: ≥2 hits

# 7. No homegrown SCIP symbol-string scheme
rg "fn.*encode_symbol_string|HomeGrown.*Symbol" .elai_cc/crates/scip-import/ -g '*.rs'
# Expected: zero hits

# 8. 7 new SSB action variants
rg "ScipImported|ScopeGraphResolved|SymbolDslExecuted|DataflowImpactComputed|LspOracleInvoked|AgentImpactClaimRefuted|PagerankComputed" .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: 7 hits

# 9. No FocalAD citation anywhere
rg "FocalAD|2506.11419" .elai_cc/crates .elai_cc/docs workflow/reasoning/E-40-decisions.md
# Expected: ONLY hits in E-40-decisions.md "Citation Errata" section flagging the mis-cite

# 10. SCIP crate anti-slopsquatting verified in decision doc
rg "scip crate.*verified|scip.*crates.io.*version" workflow/reasoning/E-40-decisions.md
# Expected: ≥1 hit (verification record)
```

## ACCEPTANCE

- [ ] `elai index --scip <path>` imports a rust-analyzer `scip` file into the existing 27-shard SymbolGraphCache; Rust precision improves vs tree-sitter-only on the B-29.5 4-corpus benchmark
- [ ] ScopeGraph push/pop scope-graph layer resolves names across files for 4+ languages via TOML rules in `.elai_cc/scope_graph_rules/`; no `github/stack-graphs` runtime dep
- [ ] DSL parser executes Angle-lite queries (MATCH / WHERE / RETURN) over SymbolGraphCache; ≥90% of historical auditor questions expressible (per report Open Question 3)
- [ ] `compute_impact_radius_dataflow` separately surfaces data-flow def-use edges (depth=2 default); p99 ≤ 500ms per B-29.5
- [ ] LSP oracle invoked ONLY from `confidence: Unresolved` path; `LspOracleInvoked` SSB count visible in dashboard; anti-spam guard
- [ ] Refinement+Filter rejects too-narrow agent impact claims; `AgentImpactClaimRefuted` SSB row emitted with missing-files list
- [ ] symbol_graph_vec is the 4th-or-5th RRF list (depending on E-36 vision policy); deterministic across 3 runs
- [ ] Personalized PageRank top-k surfaced in auditor PostToolUse summary; stable tiebreaker via symbol-string lexicographic
- [ ] 2 new ToolDefinition entries (symbol_dsl_query + symbol_dataflow_impact) wired in build_tool_catalog
- [ ] FocalAD mis-cite flagged in E-40-decisions.md "Citation Errata"; no FocalAD references in new code or docs
- [ ] anti-slopsquatting: `scip` crate verified (name + version + last-publish date) in decision doc before any deps add
- [ ] 7 new SSB action variants emit on the relevant fixture triggers
- [ ] No homegrown SCIP symbol-string scheme; report anti-pattern guard intact

## Tests to add

- `crates/contracts/tests/scip_import_roundtrip_contract.rs`: import a fixture `.scip` file; assert symbol_strings match SCIP grammar; assert call-edge resolution precision ≥95% on a 100-symbol Rust fixture (vs ≥85% tree-sitter-only target)
- `crates/contracts/tests/scope_graph_resolver_4lang_contract.rs`: synthetic 4-language fixture (Rust / Python / TypeScript / Go); assert push/pop resolves at least 1 cross-file name per language correctly
- `crates/contracts/tests/symbol_dsl_query_executor_contract.rs`: 10 sample DSL queries × expected result set; assert deterministic across 3 runs
- `crates/contracts/tests/dataflow_impact_depth2_p99_contract.rs`: 100-query benchmark on the elai_self fixture (per existing scale_determinism.rs); assert p99 ≤ 500ms (B-29.5 target)
- `crates/contracts/tests/lsp_oracle_on_demand_only_contract.rs`: synthetic scenario × 50 queries; assert LSP invoked ONLY when confidence=Unresolved AND the result affects audit verdict; assert invocation count ≤5 per session
- `crates/contracts/tests/agent_impact_claim_refute_contract.rs`: agent claims [Y, Z]; symbol-graph walk shows [Y, Z, W] downstream; assert refute + AgentImpactClaimRefuted SSB row
- `crates/contracts/tests/pagerank_top_k_stable_tiebreaker_contract.rs`: 5 runs on the same change-set with ties present; assert identical top-k order via lexicographic tiebreaker
- `crates/contracts/tests/symbol_graph_vec_rrf_5list_contract.rs`: 5-list RRF with symbol_graph_vec; assert stable ordering; assert weight respected
- `crates/contracts/tests/focalad_no_citation_anywhere_contract.rs`: grep entire codebase + docs (excluding E-40-decisions.md Errata section); assert zero hits

## Files touched

- CREATE: `crates/scip-import/Cargo.toml` (new crate)
- CREATE: `crates/scip-import/src/lib.rs` (~80 LOC mod declarations)
- CREATE: `crates/scip-import/src/document.rs` (~200 LOC SCIP protobuf parser)
- CREATE: `crates/scip-import/src/symbol_table.rs` (~180 LOC symbol-string → 27-shard mapping)
- CREATE: `crates/scip-import/src/to_shards.rs` (~240 LOC writer to existing SymbolGraphCache shards)
- CREATE: `crates/knowledge/src/symbol_graph/scope_graph.rs` (~400 LOC)
- CREATE: `crates/knowledge/src/symbol_graph/dsl.rs` (~500 LOC parser + executor)
- CREATE: `crates/knowledge/src/symbol_graph/dataflow.rs` (~280 LOC)
- CREATE: `crates/knowledge/src/symbol_graph/lsp_oracle.rs` (~200 LOC)
- CREATE: `crates/knowledge/src/symbol_graph/refine_filter.rs` (~120 LOC)
- CREATE: `crates/knowledge/src/symbol_graph/pagerank.rs` (~180 LOC)
- MODIFY: `crates/knowledge/src/symbol_graph/mod.rs` (+6 mod declarations)
- MODIFY: `crates/knowledge/src/pipeline.rs` (+~40 LOC for symbol_graph_vec RRF integration)
- MODIFY: `crates/orchestrator/src/runtime_orchestrator.rs` (+~25 LOC for `lsp_oracle` role)
- MODIFY: `crates/orchestrator/src/task_runner.rs::build_tool_catalog` (+~40 LOC for 2 new ToolDefinitions + auditor PostToolUse PageRank surfacing)
- MODIFY: `crates/orchestrator/src/ssb.rs` (+7 SsbActionType variants)
- MODIFY: `crates/cli/src/main.rs` (+~50 LOC for `elai index --scip` + `elai symbol query` subcommands)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (+~80 LOC for 4 new Tauri commands)
- MODIFY: existing KnowledgeDrawer IMPACT ANALYSIS panel (C-24 PLANNED — coordinate so C-24 surface receives the new fields)
- CREATE: `.elai_cc/scope_graph_rules/rust.toml` (~50 LOC push/pop rules)
- CREATE: `.elai_cc/scope_graph_rules/python.toml` (~50 LOC)
- CREATE: `.elai_cc/scope_graph_rules/typescript.toml` (~50 LOC)
- CREATE: `.elai_cc/scope_graph_rules/go.toml` (~40 LOC)
- MODIFY: `elai_cc.toml` schema docs (`[knowledge.scip_import]`, `[knowledge.dsl]`, `[knowledge.dataflow]`, `[knowledge.lsp_oracle]`, `[knowledge.pagerank]`)
- CREATE: 9 contract tests under `crates/contracts/tests/` per "Tests to add"
- CREATE: `workflow/reasoning/E-40-decisions.md` (with "Citation Errata" section per Task 10)

## Citations / Grounding

- github.com/sourcegraph/scip (cited in source) — SCIP protocol; protobuf; human-readable symbol IDs; 8x smaller / 3x faster than LSIF
- sourcegraph.com/blog/announcing-scip (cited in source) — format spec, symbol-string grammar, document/occurrence model
- sourcegraph.com/blog/announcing-scip-typescript (cited in source) — scip-typescript perf 1k-5k LOC/sec
- sourcegraph.com/blog/scip-python (cited in source) — scip-python on Pyright; 960-2595 LOC/sec
- rust-lang.github.io/rust-analyzer/rust_analyzer/cli/scip (cited in source) — rust-analyzer scip CLI; Salsa-incremental
- arxiv:2211.01224 Stack Graphs (cited in source) — push/pop scope semantics; canonical reference (project archived 2025-09-09 — design only, no dep)
- github.com/github/stack-graphs (cited in source) — ARCHIVED 2025-09-09; do NOT depend
- arxiv:2406.07003 GraphCoder ASE 2024 (cited in source) — Code Context Graph w/ control+data+control-dep edges; beats sequential RAG
- arxiv:2408.03910 CodexGraph NAACL 2025 (cited in source) — property graph DB; 27.9% EM CrossCodeEval-Lite, 22.96% Pass@1 SWE-bench-Lite
- arxiv:2509.25257 RANGER (cited in source) — repo-level retrieval agent over knowledge graph; multi-hop; NL + entity queries
- arxiv:2406.14497 CodeRAG-Bench NAACL 2025 (cited in source) — low lexical-overlap retrieval finding
- arxiv:2511.00262 LLM-Driven Cost-Effective Requirements Change Impact Analysis Nov 2025 (cited in source) — Refinement+Filter post-processing pattern
- preprints.org/manuscript/202510.0924 Exploratory Study of Code Retrieval in Coding Agents Oct 2025 (cited in source) — LSP adds tokens without recall gain in agent setting
- engineering.fb.com/2024/12/19/developer-tools/glean-open-source-code-indexing (cited in source) — Glean Dec 2024; Angle DSL
- github.com/ast-grep/ast-grep (cited in source) — Rust-native tree-sitter wrapper (complementary, not replacement)
- aider.chat/docs/repomap.html (cited in source) — personalized PageRank; 50x chat-file multiplier
- buck2.build/docs/about/why (cited in source) — Buck2 single incremental dependency graph
- sourcegraph.com/blog/how-cody-understands-your-codebase (cited in source) — Cody 3-layer pipeline (keyword + embedding + code-graph + rerank)
- github.blog/engineering/architecture-optimization/the-technology-behind-githubs-new-code-search (cited in source) — Blackbird; sharding by Git blob OID

## Open questions / risks

- **FocalAD mis-citation (report Open Question 5; THIS E-N's flag)**: arxiv:2506.11419 is autonomous-driving (Ego-Local-Agents Interactor for ego-vehicle planning), NOT code-change impact analysis. ELAI's prior citation of FocalAD-as-impact-analysis is INCORRECT. Per CLAUDE.md "Verify paper citations" rule + memory:feedback_paper_citation_verify ("WebSearch arXiv IDs before citing; effect-size numbers need quote+page-anchor; 2 bad cites in B-40/B-42 plans"), this is the third bad-cite pattern. E-40 Task 10 surfaces the errata in workflow/reasoning/E-40-decisions.md. Any B-29 design notes that reference FocalAD MUST be corrected. The actual references for code-change impact analysis are GraphCoder ASE 2024 (arxiv:2406.07003), CodexGraph NAACL 2025 (arxiv:2408.03910), RANGER (arxiv:2509.25257), and the Nov 2025 RCIA paper (arxiv:2511.00262).
- SCIP-import break-even (report Open Question 1): at what repo size does SCIP-import outperform tree-sitter-only for Rust on the auditor PostToolUse latency budget (depth=2 impact summary, p99 ≤ 500ms B-29.5 target)? Run B-29.5 benchmark against both backends on Linux v6.7 / Chromium / VS Code / Postgres. If SCIP-import doesn't beat tree-sitter on small repos (< 100K LOC), gate it behind a `[knowledge.scip_import] enabled = false` for small projects and document the break-even.
- Dynamic-dispatch recall on Python sidecar (report Open Question 2): on `python/elai_inference/`, what % of `model.method()` call sites does tree-sitter resolve correctly vs Pyright/scip-python? Run benchmark before deciding LSP-on-demand vs SCIP-batch for Python.
- DSL adoption coverage (report Open Question 3): if E-40 ships the Angle-lite DSL, what % of historical auditor questions are expressible (target ≥90%)? Sample 100 historical auditor logs; classify each as DSL-expressible or not.
- PageRank stability (report Open Question 4): does Aider's personalized PageRank produce deterministic top-k across 5 runs when ranks are tied? B-29.5 requires determinism. PageRank ties need a stable tiebreaker (symbol-string lexicographic per Task 8); verify on 100-symbol fixture.
- Anti-slopsquatting on `scip` crate (Task 1): MANDATORY before adding. Check crates.io for current version + last-publish date + repo owner. Document in decision doc.
- Stack-Graphs runtime dep: report explicitly flags `github/stack-graphs` ARCHIVED 2025-09-09. DO NOT add as runtime dep. Task 2 re-implements the design in fresh Rust on top of ELAI's existing tree-sitter queries.
- LSP token cost (Task 5): Preprints 202510.0924 measured token-overhead without measurable recall gain in agent settings. The on-demand-only constraint is HARD — Task 5 must guard against agent loops that over-spam the LSP. The `LspOracleInvoked` SSB count is the discipline signal; >5 invocations/session = bug-pattern in the agent code.
- ScopeGraph language coverage: the 13+ languages B-30 targets may not all have public push/pop rule references. For under-served languages, fall back to tree-sitter heuristic edges + mark `confidence: Heuristic`. The 4 starter languages (Rust, Python, TypeScript, Go) cover ~80% of ELAI's user code; the remaining 9 are follow-ups.

## Next

E-40 + B-29 + B-30 + C-24 together complete the symbol-graph picture. E-40 unblocks: (a) better auditor PostToolUse summaries (PageRank top-k); (b) DSL query surface for the agent's flexible "callers of X that touch Y" questions; (c) cross-repo navigation via SCIP for future multi-project setups; (d) Refinement+Filter discipline closes the "agent claims a too-narrow impact" failure mode. Future E-Ns: (i) ScopeGraph language extensions beyond the 4 starter — fill the remaining 9 from B-30's target list; (ii) Buck2-style "queryable target dependency graph" view across the workspace (report rec #8 — design-level only here; implementation is a future E-N); (iii) full Glean Angle parity if the DSL proves load-bearing; (iv) HippoRAG PPR (per E-17.D follow-up) seeded from query-extracted concepts.
