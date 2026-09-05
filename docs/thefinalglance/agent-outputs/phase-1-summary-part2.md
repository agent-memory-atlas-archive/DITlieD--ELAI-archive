# ELAI Phase 1 — Knowledge Engine — Summary Part 2 (1D, 1E, 1F)

(See phase-1-summary-part1.md for index and parts 1A, 1B, 1C.)

=====================================================================
PHASE 1D — PAGERANK REPO MAP
=====================================================================

Prerequisites: Phase 1C complete (cargo check -p knowledge 0 with ort).
Goal: rank code symbols by importance via PageRank on DKG; produce token-budget-fitted repo maps.

Risks:
- R20 (risk-register L88) density degradation — guard at 0.3. ID collision with phase-0 R20 (entropy scope).
- R-NEW dangling nodes zero rank — pass dangling=personalization per aider.
- R-NEW orphan symbols vanish — self-edges weight=0.1 per aider.

STEPS

1D.1 SKELETON + DEPENDENCY VERIFICATION
PRE_GATE: 1C check 0; DKG symbol types exist; providers::token_count_estimate exists;
DKG edge/adjacency API exists (query dkg_edges). If missing, 1D.1 must add one.
TASK:
- Verify proptest; add to dev-deps (proptest = "1").
- In-memory projection:
    `DkgSnapshot { nodes: Vec<DkgNodeInfo>, edges: Vec<(usize, usize, f64)> }`.
    `DkgNodeInfo { node_id: i64, symbol_name: String, file_path: PathBuf, signature: String,
                   kind: SymbolKind, ref_count: usize }`.
    `pub fn DkgSnapshot::from_engine(engine: &DkgEngine) -> Result<DkgSnapshot>`.
    Adds self-edge weight=0.1 for nodes with zero incoming edges (aider pattern).
- Pagerank module:
    `PageRankConfig { damping_factor: f64=0.85, max_iterations: usize=100, tolerance: f64=1e-6,
                      density_threshold: f64=0.3, max_map_tokens: usize=2048 }`
    (defaults UNGROUNDED — standard Brin/Page 1998 — stored/overridable).
    `RepoMap { ranked_symbols: Vec<RankedSymbol>, token_count: usize, density: f64,
               used_pagerank: bool, fit_iterations: usize }`.
    `RankedSymbol { node_id: i64, symbol_name: String, file_path: PathBuf, rank: f64, signature: String }`.
- Public API:
    `pub fn compute_repo_map(snapshot, task, active_files, config, token_estimator: &dyn Fn(&str)->usize) -> RepoMap`
  Bodies todo!().

DETERMINISTIC_CHECK: cargo check 0; 1 hit each for compute_repo_map, PageRankConfig, RepoMap, DkgSnapshot;
proptest hit.

ADVERSARIAL: schema drift; empty edges; max_map_tokens=0 handled.

ACCEPTANCE: cargo check 0; proptest listed.

1D.2 VERIFY
TASK: cargo check workspace 0; existing knowledge tests pass.
ACCEPTANCE: zero regressions.

1D.3 TEST (TDD RED)
Unit tests (>=21):
T1 basic linear 5-node + task mention -> node 3 ranked top.
T2 edge weight boost *=50 for active_files.
T3 personalization 10x for mentioned nodes.
T4 density>0.3 -> used_pagerank==false; fallback = insertion order (NOT BM25-ranked;
  compute_repo_map has no BM25 dep; BM25 rerank is upstream per knowledge.txt L88).
T5 density<0.3 -> used_pagerank==true.
T6 token count <= max*1.15 (+15% tolerance).
T7 undershoot guard: 200 nodes @ 10 tokens, budget=1000 -> token_count>=850 (~85-115 nodes).
T8 special files always included (Cargo.toml, README).
T9 personalization sum==1.0 (f64 epsilon).
T10 1000-node convergence: fit_iterations<=10.
T11 empty graph: empty RepoMap, tokens=0, fits=0.
T12 single-node returns that node.
T13 max_map_tokens=0: empty, no panic.
T14 all nodes mentioned uniform: still valid (no NaN/Inf).
T15 dangling redistribution: A->B->C chain, C rank>0, sum~=1.0.
T16 orphan self-edges: node-4 orphan with self-edge weight=0.1 -> rank>0.
T17 sqrt(ref_count) scaling: 100-ref vs 4-ref -> rank ratio <8x (not 25x linear).
T18 line truncation 100 chars: 500-char sig -> token count uses 100-char.
Property tests: P1 ranks finite >=0; P2 ranks sum~=1.0 under used_pagerank; P3 token budget bound.
Adversarial: A1 near-complete graph density ~0.95 -> guard activates;
A2 star graph hub #1; A3 disconnected components handled.
Test helper: `fn make_snapshot(nodes: usize, edges: Vec<(usize,usize)>) -> DkgSnapshot`.
DETERMINISTIC_CHECK: >=21 #[test]; proptest! >=1; runtime FAILED (todo!).
ACCEPTANCE: compiles + fails.

1D.4 IMPLEMENT
TASK:
- DkgSnapshot::from_engine: query tables -> in-memory.
- compute_repo_map:
  1. Build adjacency matrix: self-edges from snapshot + sqrt(ref_count) scaling (aider).
  2. Apply edge-weight *=50 for edges touching active_files BEFORE PageRank.
  3. Personalization 10x for task-mentioned, normalize sum=1.0.
  4. Density = |edges|/(|nodes|*(|nodes|-1)); >0.3 -> skip + insertion order, used_pagerank=false;
     nodes<=1 -> density=0 skip.
  5. Power iteration: damping=0.85, max_iter=100, tol=1e-6; dangling redistributes via personalization.
  6. Special files (13 Rust items hardcoded initial):
     Cargo.toml, Cargo.lock, README.md, README, LICENSE, .gitignore, Dockerfile,
     docker-compose.yml, build.rs, rust-toolchain.toml, clippy.toml, rustfmt.toml, .cargo/config.toml.
     (Config-driven extensibility deferred.)
  7. Binary search fit: max 10 iter; initial guess = min(max_tokens/25, num_ranked_nodes)
     (aider); tolerance +-15%.
     If max_map_tokens==0: empty ranked, no special files, fit_iterations=0.
  7b. Truncate each signature to 100 chars BEFORE estimating tokens.
  8. Return RepoMap populated.
- Internal helpers:
  `build_adjacency_matrix(snapshot, active_files) -> Vec<Vec<f64>>` (sqrt scaling + self-edges).
  `build_personalization(node_count, task_mentioned_indices) -> Vec<f64>` sum=1.0.
  `build_dangling_vector(matrix, personalization) -> Vec<f64>`.
  `power_iteration(matrix, personalization, dangling, damping, max_iter, tolerance) -> Vec<f64>`
    with NaN/Inf clamp-to-0 + warn.
  `truncate_signature(sig, max_chars=100) -> String`.
  `is_special_file(path) -> bool`.
  `binary_search_fit(ranked, special, max_tokens, tolerance, estimator) -> (Vec<RankedSymbol>, usize)`.

DETERMINISTIC_CHECK: cargo test pagerank 0 failures; T1-T18 + P1-P3 + A1-A3 all pass;
no todo!/unimplemented!/panic("not implemented").

ADVERSARIAL: NaN on bipartite; personalization drift; off-by-one in bsearch; special files incomplete;
zero-budget edge.

ACCEPTANCE: 0 failures; clippy 0; no todo markers.

1D.5 MEASURE
.elai_cc/metrics/phase_1D.toml keys:
  test_count, compile_time_seconds (cargo build --timings), pagerank_loc (wc -l; tokei optional),
  ranking_quality (star-hub=#1 bool, mentioned top-3 bool),
  token_budget_accuracy (max % deviation), density_guard_correct (bool),
  binary_search_max_iterations.
ACCEPTANCE: file with all 7 metrics.

AIDER SOURCE ANALYSIS
ADOPTED (5 improvements each with test proving benefit):
1. dangling=personalization (T15).
2. self-edges weight=0.1 for orphans (T16).
3. sqrt(num_refs) edge scaling (T17).
4. binary search initial guess max_tokens/25 (T10).
5. line truncation 100 chars (T18).
DIVERGENCES (3 deliberate):
1. Symbol-level nodes (vs aider file-level) — finer granularity; larger graph.
2. Density guard 0.3 (aider has none / ZeroDivisionError) — proof A1.
3. Insertion-order fallback (aider returns empty).
NOT ADOPTED:
- Identifier specificity boost (10x long names, 100x cumulative) — symbol-level graph already granular.
- Private-symbol 0.1x — Rust uses visibility keywords, not underscore convention.
- Generic-name 0.1x penalty — sqrt already dampens; risk over-penalizing shared types.

GROUNDING summary: density guard, binary search, edge boost, personalization, special files,
convergence, ±15% tolerance, sampling token estimate all GROUNDED to knowledge.txt / providers.txt;
damping=0.85, max_iter=100, tol=1e-6 UNGROUNDED (standard, config-overridable);
aider patterns GROUNDED to aider/repomap.py (dangling, self-edge, sqrt, initial guess, truncation).

SPEC CONFLICTS:
1. Header "Section 2 Layer 4" — PageRank actually Layer 3.5 (FIXED in file).
2. R20 collision between risk-register.txt L88 and phase-0.txt L22 (cross-file fix noted).

EXIT CRITERIA (13 items): >=21 tests pass; density guard activates; ±15% budget; special files present;
no todo/unimplemented; clippy 0; metrics toml 7 metrics; zero-budget T13 no panic; proptest 100;
T15/T16/T17/T18 all pass.

=====================================================================
PHASE 1E — CatRAG + A-RAG (query pipeline assembly)
=====================================================================

Prerequisites: 1D complete; depends on 1B, 1C, 1D.
Goal: assemble BM25 -> vector -> PageRank -> CatRAG -> A-RAG; target 55-87% token savings;
sub-second latency.

GROUNDING SOURCES:
- CatRAG arXiv 2602.01965 (Context-Aware Traversal for Robust RAG).
  Interface map: workflow/papers/catrag-interface.md.
  Algorithms: Symbolic Anchoring, Query-Aware Dynamic Edge Weighting, Key-Fact Enhancement.
  Applied as query-time overlays on PPR (Phase 1D) — modifies reset vector + transition matrix.
- A-RAG arXiv 2602.03442 (Scaling Agentic RAG via Hierarchical Retrieval Interfaces).
  Interface map: workflow/papers/arag-interface.md.
  Three-tool hierarchy (keyword/semantic/chunk_read); ReAct agent loop;
  snippet progressive disclosure; context tracker deduplication.
- Spec: knowledge.txt Layers 4-5, Section 3.

ADAPTATION (paper -> ELAI):
CatRAG entity nodes V_E -> DkgNode (fn/struct/trait/mod).
CatRAG passage nodes V_P -> BM25 chunks.
CatRAG relation edges E_rel -> DKG edges (calls/refs/imports/implements).
CatRAG synonym edges E_syn -> high embedding similarity (>0.8 cos).
CatRAG context edges E_ctx -> DKG node-to-file mapping.
OpenIE triple extraction -> NOT NEEDED (tree-sitter already structured).
NER -> regex/fuzzy symbol name extraction against DKG.
A-RAG keyword_search -> BM25 + snippets.
A-RAG semantic_search -> dual-vector + snippets.
A-RAG chunk_read -> file read or line range.
ReAct loop -> local LLM (ollama) or external.
No parallel tool calling.

Pipeline order (knowledge.txt L104-105):
query -> BM25+SPLADE(RRF) -> dual-vector rerank -> PageRank importance filter
  -> CatRAG expand -> A-RAG filter -> cross-encoder rerank -> token-budget fit -> context.
Cross-encoder=1G, token-fit=3E. 1E implements CatRAG + A-RAG + wiring + stubs for 1G/3E.

Risks:
- R30 CatRAG LLM query-time latency — limit N_seed=5, K_edge=15; cache LLM scores.
- R31 sparse code graph — synonym edges at >0.8 cos; skip CatRAG if density<0.01.
- R32 ReAct infinite loops — max_loops=15, context tracker.
- R33 snippet loses context — ±2 line window for code.
- R34 2s total latency budget — measure per-layer.

STEPS

1E.1 SKELETON
TASK:
- catrag.rs with:
    `CatRagConfig { damping: f64=0.5, epsilon: f64=0.2, beta: f64=2.5, n_seed: usize=5,
                    k_edge: usize=15, min_graph_density: f64=0.01 }` (paper defaults).
    `CatRagOverlay { reset_distribution: Vec<(usize, f64)>,
                     edge_weight_deltas: Vec<(usize, usize, f64)> }` (sparse).
    `CatRag::new(config)`;
    `compute_overlay(&self, query, snapshot: &DkgSnapshot, pagerank_scores: &[(usize,f64)]) -> Result<CatRagOverlay>`.
- arag.rs with:
    `AragConfig { max_loops: usize=15, max_token_budget: usize=128000, top_k: usize=5 }`.
    `enum RetrievalTool { KeywordSearch { keywords, top_k }, SemanticSearch { query, top_k },
                          ChunkRead { chunk_ids } }`.
    `RetrievalResult { chunk_id: String, snippet: String, score: f64, tool_used: String }`.
    `AragOrchestrator` (holds BM25 + vector index + file reader refs).
    `ContextTracker { read_chunks: HashSet<String> }` (per-query instance).
    `execute_tool(tool, tracker) -> Result<Vec<RetrievalResult>>`.
    `query_pipeline(query, snapshot) -> Result<Vec<RetrievalResult>>`.
- pipeline.rs with:
    `QueryPipeline { bm25: Bm25Index, catrag: CatRag, arag: AragOrchestrator, ... }`.
    `PipelineStage { stage: String, results: Vec<RetrievalResult>, latency_ms: u64 }`.
    `query(&self, q: &str) -> Result<Vec<PipelineStage>>`.
- Wire mods in lib.rs. All todo!() stubs.

DETERMINISTIC_CHECK: cargo check 0; pub mod catrag/arag/pipeline all present.
ADVERSARIAL: overlay sparse not matrix-copy; tracker per-query.
ACCEPTANCE: compiles.

1E.2 VERIFY — cargo check package + workspace 0.

1E.3 TEST (TDD RED)
CatRAG tests (a-e):
a. Symbolic anchoring: reset_distribution weight=epsilon*|files_containing|^-1 for matched nodes.
b. Dynamic edge weighting: overlay has modified weights for seed outgoing; non-seeds unchanged.
c. Key-fact enhancement: context edges to referenced files get weight *=(1+beta).
d. Density guard: density<min -> empty overlay (passthrough).
e. phi() tiers: 0-3 -> 0.0, 4 -> 0.2, 5 -> 0.25, 6 -> 0.3, 7 -> 2.0, 8 -> 2.5, 9 -> 3.0, 10 -> 5.0.
A-RAG tests (f-j):
f. KeywordSearch returns snippets only, score=sum(count*|keyword|).
g. SemanticSearch oversample 3x sentence, aggregate by max score per file.
h. ChunkRead first returns full; second returns "(previously read)".
i. ContextTracker monotonic read_chunks; is_read() correct.
j. Token budget enforcement returns partial.
Pipeline tests (k-m):
k. query_pipeline returns stages in order ["bm25","vector","pagerank","catrag","arag"].
l. each stage latency_ms>0.
m. empty query -> empty results no panic.
Contract (n-o):
n. Overlay round-trip: base graph unchanged after apply.
o. RetrievalResult all fields populated.
ACCEPTANCE: tests fail with todo!().

1E.4 IMPLEMENT CatRAG
PRE_GATE: catrag-interface.md ingested.
TASK:
a. Symbolic anchoring: split query (whitespace, camelCase, snake_case); fuzzy match DKG names
   (case-insensitive, substring); reset_weight=epsilon*|files_containing|^-1;
   combine with PageRank seeds (higher base); normalize sum=1.0. Deterministic (no LLM).
b. Dynamic edge weighting (code-graph simplified):
   top-N_seed by reset prob; for each seed outgoing edges; if |neighbors|>K_edge coarse prune by
   embedding similarity; heuristic scoring:
     edge type weight (calls=8, implements=7, references=5, imports=3),
     target node type relevance,
     file proximity to query-mentioned files.
   Map combined score to phi() tiers. Store as edge_weight_deltas (sparse).
   LLM scoring optional upgrade via circuit breaker.
c. Key-Fact: for each DKG edge symbol->file, if symbol in T_seed (mentioned + direct callees),
   boost context edge *= (1+beta). Zero LLM cost.
d. Density guard: density<0.01 -> empty overlay + warn.
e. Apply overlay: modify PPR reset + transition, re-run Phase 1D power iteration, return ranked.
ADVERSARIAL: modify overlay not base graph; phi boundary tests explicit; cap matched nodes at 2*N_seed;
warn if >50% DKG nodes matched.
ACCEPTANCE: overlays sparse + local; base never mutated.

1E.5 IMPLEMENT A-RAG
TASK:
a. KeywordSearch wraps BM25: merge keyword results; score=sum(count*|keyword|);
   snippets=sentences with keyword ±2 lines code context; top_k with snippets.
b. SemanticSearch wraps vector index: 3x oversample sentence; aggregate by max per file;
   snippets with surrounding context.
c. ChunkRead: if tracker.is_read -> "(previously read)"; else read full + mark read.
d. ContextTracker: HashSet<String>; per-query; is_read, mark_read, token_count.
e. execute_tool dispatch by enum; track tokens; enforce max_token_budget (partial on overflow).
f. query_pipeline deterministic (not ReAct — full agent loop = Phase 2+): 
   keyword_search -> semantic_search -> chunk_read top hits.
ADVERSARIAL: snippet non-empty for .rs; budget enforcement triggers; per-query tracker isolation.

1E.6 IMPLEMENT PIPELINE
Stages:
1. BM25: extract keywords (dedup, stopwords); Bm25Index::query(query, top_k=20); record stage.
2. Vector: query embedding; RRF fuse with BM25 (k=60); skip if CB open.
3. PageRank: build DkgSnapshot, run PageRank, filter/rerank by PR score.
4. CatRAG: compute overlay; apply to PageRank; re-run PPR; expand via high-weight edges; passthrough
   if density guard.
5. A-RAG: orchestrator filter + refine; snippet disclosure.
6-7. Stubs: "rerank" (1G), "token_fit" (3E).
Return Vec<PipelineStage>.
ADVERSARIAL: each stage Result-wrapped (failure logs, falls through); RRF k=60 prevents div-by-zero
(rank starts at 1); verify stage order vs knowledge.txt L104-105.

1E.7 MEASURE
Binary: knowledge/src/bin/measure_1e.rs.
Index .elai_cc; run 5 queries:
  Q1 FSM state transitions (multi-hop cross-crate)
  Q2 BM25 query latency (keyword-heavy)
  Q3 What implements LlmProvider (symbol, PageRank)
  Q4 Error handling in orchestrator (broad semantic)
  Q5 Tools registered and dispatched (relationship)
Per query: total latency (<2000ms), per-stage, tokens_retrieved, token_savings_pct,
stages_active.
Output: .elai_cc/metrics/phase_1E.toml:
  pipeline_p50_ms, pipeline_p99_ms, token_savings_mean_pct (>50% target),
  catrag_overlay_size, arag_tools_per_query, stages_skipped.
ACCEPTANCE: all 5 produce results; p50<2000ms; savings>50%; 21+ tests green.

EXIT CRITERIA: pipeline operational; token savings >50%; latency <2s; graceful degradation;
no regressions.

=====================================================================
PHASE 1F — SPECULATIVE DECODING EVALUATION
=====================================================================

Prerequisites: Phase 1C complete (ModelPool VRAM accounting needs providers/ operational).
Correction: original "Phase 0 independent" conflicted with dep graph (implementation-guide L141 shows 1C->1F).

Status markers:
- RESOLVED (2026-04-07, commit eeb9a63): Ollama v0.20.2 confirmed NO speculative decoding.
  PR #8134 closed, never merged. Probe verified draft_model silently ignored.
  DECISION: DROP for Phase 1. Steps 1F.3-1F.5 SKIPPED.
  Revisit in Phase 2+ (vLLM provider). 1F.1 skeleton + 1F.2 probe committed.
- SPEC CORRECTIONS APPLIED (session s-937656, 2026-04-06):
  providers.txt L46-47 -> 2x-6.5x + ~4.8GB headroom;
  external-analysis.txt L312/456 -> 2x-6.5x;
  implementation-plan.txt L184 still stale "1.8-2.3x" (INFO not blocking).

Goal: EVALUATION phase — ADOPT if >1.3x throughput, DROP else. Output = measured decision.

Risks:
- R4 VRAM on RTX 3060 12GB: main ~7.2GB (Gemma 4 E2B Q4_K_M); headroom ~4.8GB (not ~10GB);
  scenario A (~2GB draft): 2.8GB remaining; scenario B (~3GB max): 1.8GB tight.
- R-NEW-1 Ollama spec decode NOT merged (PR #8134). Fallback: llama.cpp direct (only viable P1 fallback;
  vLLM not until Phase 2+). Verify before benchmarking.
- R-NEW-2 EAGLE-3 2x-6.5x on 70B+; gains on 2.3B effective model much lower. 1.3x threshold accounts.
- R-NEW-3 Gemma 4 EAGLE-3 draft may not exist (vLLM issue #38893). Fall back to standard same-family
  smaller model. 1F.2 verifies.

GROUNDING:
- EAGLE-3 corrected 2x-6.5x (arxiv 2503.01840, RedHat 2025-07-01, LMSYS 2025-12-01).
- "~10GB headroom" WRONG — 4.8GB actual.
- TIDE 1.15x over static spec decode (arxiv 2602.05145).
- Ollama PR #8134 UNVERIFIED at plan time.
- 1.3x UNGROUNDED (project threshold).
- Same-family smaller INCOMPLETE: llama.cpp #10466 — 3B draft max 1.08x@k=1; 1.5B 1.17x@k=4;
  0.5B 1.4x@k=4. Smaller drafts outperform larger on consumer GPUs.
- k=5 default SUBOPTIMAL — sweep per 1F.4.
- Acceptance rate <0.5 HURTS throughput (vLLM blog 2024-10-17, Spheron) — early kill.
- Prompt lookup decoding ZERO-VRAM, up to 2.8x on summarization (vLLM blog, prompt-lookup-decoding repo).
- Prereq corrected from Phase 0 to Phase 1C.

STEPS

1F.1 SPECULATIVE DECODING CONFIG SKELETON
PRE_GATE: Phase 1C tests pass; providers compiles; OllamaConfig exists.
TASK: add to providers/ config (OllamaConfig or nested SpeculativeConfig):
  speculative_decoding: bool (default false),
  draft_model: Option<String>,
  num_draft_tokens: u8 (default 5).
Serde round-trip.
DETERMINISTIC_CHECK: cargo check providers 0; grep 3+ hits; prior phase tests still pass.
ADVERSARIAL: serde mismatch silent drop; num_draft_tokens=0 invalid.
ACCEPTANCE: cargo check 0; default (false, None, 5) test; serde round-trip test.

1F.2 OLLAMA SPECULATIVE DECODING API AVAILABILITY CHECK
PRE_GATE: ollama running; main model loaded.
TASK: probe /api/generate or /api/chat for draft_model parameter.
If NOT supported: evaluate fallback (llama.cpp direct only; if neither viable -> DROP + defer to Phase 2).
If supported: identify API params.
Draft model selection: prefer 0.5B-1.5B (llama.cpp #10466 evidence). Record: name, params, quant, VRAM.
DETERMINISTIC_CHECK: probe script output "SUPPORTED: [api params]" or "NOT_SUPPORTED: [reason]";
workflow/reasoning/phase-1F-decisions.md documents fallback.
ADVERSARIAL: ollama silently ignoring unknown params (false positive) — check timing/log differ;
record ollama --version vs minimum version.
ACCEPTANCE: AVAILABLE or NOT_AVAILABLE binary; decisions file on NOT_AVAILABLE; API params on AVAILABLE.

ACTUAL OUTCOME (per NOTE): NOT_AVAILABLE. Probe verified silent draft_model ignore. DROP.

1F.3 CORRECTNESS VERIFICATION (SKIPPED per outcome)
PRE_GATE: 1F.2 AVAILABLE (else skip to 1F.5 with DROP).
TASK: 10 prompts with and without spec decode at seed+temperature=0; compare decoded text
(soft match: identical content; token ID divergence logged WARN not fail — ollama float
non-determinism per issues #586, #5321).
DETERMINISTIC_CHECK: .elai_cc/metrics/phase_1F_correctness.toml with 10/10.
ADVERSARIAL: temperature=0 only; tokenizer differences; cache between runs.
ACCEPTANCE: 10/10 identical text.

1F.3b PROMPT LOOKUP DECODING BASELINE (zero-VRAM) (SKIPPED per DROP)
PRE_GATE: 1F.3 passes OR 1F.2 NOT_AVAILABLE.
TASK: benchmark n-gram speculation from input context; zero VRAM; up to 2.8x on
repetitive/summarization (vLLM blog 2024-10-17).
5 prompts x 3 runs = 15 measurements. Record ratio.
If neither ollama nor llama.cpp --lookup-ngram-min supports: skip, log UNAVAILABLE.
WHY: if prompt lookup alone >1.3x, draft model spec decode unnecessary.
Output: .elai_cc/metrics/phase_1F_prompt_lookup.toml, 15 measurements, throughput ratio.
ADVERSARIAL: include 2+ low-overlap prompts (creative, code gen).

1F.4 THROUGHPUT BENCHMARK (SKIPPED)
PRE_GATE: 1F.3 10/10; nvidia-smi accessible; system idle (<500MB non-ollama).
TASK: 5 diverse prompts (short/medium/long; code/prose/structured); parameter sweep
k in {1,2,3,4,5,7}; 3 runs per prompt per k per mode; record prompt_len, output_len, wall time,
tokens/sec, GPU util, ACCEPTANCE RATE (accepted/total drafts).
EARLY KILL: if mean acceptance <0.5 across all k for 3+ prompts, STOP and DROP.
Compute mean/stddev per mode at best-k; ratio spec/baseline.
DETERMINISTIC_CHECK: .elai_cc/metrics/phase_1F_throughput.toml all measurements (5x6x3 + baseline);
best k per prompt + acceptance rate; stddev <20% mean PER mode independently.
ADVERSARIAL: thermal throttle (>85C discard + 60s cooldown); prompt too short bias (>=2 prompts >200 out);
model loading bias (warmup + exclude first); acceptance rate proxy via timing if API not exposed.
ACCEPTANCE: all measurements + mean/stddev/CI; best k per prompt; GPU<85C; early kill documented.

1F.5 VRAM MEASUREMENT AND DECISION GATE (SKIPPED per DROP)
PRE_GATE: 1F.4 done OR 1F.2 NOT_AVAILABLE.
TASK: baseline vs speculative VRAM (nvidia-smi polling during generation, PEAK not idle).
DECISION GATE (top-down, first match):
  - NOT_AVAILABLE (from 1F.2) -> DROP.
  - acceptance rate <0.5 -> DROP (hurts throughput).
  - prompt_lookup >1.3x AND spec<=prompt_lookup -> DROP spec, ADOPT prompt lookup (zero VRAM).
  - spec >1.3x AND delta_vram <3000MB (~1.8GB margin on 3060) -> ADOPT spec + record best-k.
    If prompt_lookup also >1.3x: prefer spec only if measurably better.
  - spec <=1.3x OR delta >=3000MB -> DROP.
  - boundary (==1.3x or ==3000MB) -> DROP (conservative tie-break).
Write workflow/reasoning/phase-1F-decisions.md with all measurements.
DETERMINISTIC_CHECK: .elai_cc/metrics/phase_1F_vram.toml peak baseline + peak speculative;
decisions file has ADOPT/DROP with justification.
ADVERSARIAL: idle-vs-generation differ significantly; production RTX 3090 vs Phase 0 hardware
(re-evaluate in Phase 4); marginal cases (ratio 1.28-1.32) -> 5 additional prompts.
ACCEPTANCE: VRAM delta in MB with nvidia-smi evidence; binary ADOPT/DROP; decisions file with
ratio + delta + hardware + date + rationale; if ADOPT config updated and compiles.

EXIT CRITERIA: decision + documented; method (spec vs prompt lookup); config (best-k, draft name/size);
all metrics in .elai_cc/metrics/phase_1F_*.toml; decisions in workflow/reasoning/phase-1F-decisions.md;
no regression.

MEASUREMENTS:
  Throughput ratio (spec at best-k): target >1.3x to ADOPT (UNGROUNDED threshold).
  Throughput ratio (prompt lookup): zero-VRAM baseline.
  Acceptance rate: must be >0.5.
  Optimal k: swept {1,2,3,4,5,7}.
  VRAM overhead: target <3000MB.
  Draft model size: record (expect 0.5B-1.5B optimal).
  EAGLE-3 reference 2x-6.5x on large models.
  TIDE 1.15x over static.
