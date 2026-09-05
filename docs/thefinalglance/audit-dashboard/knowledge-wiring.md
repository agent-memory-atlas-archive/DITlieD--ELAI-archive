# Knowledge Engine Dashboard Wiring Audit
Date: 2026-04-25
Auditor stance: HOSTILE — every wiring claim verified against source.
Spec: F:\.ELAI_workflow\docs\thefinalglance\summary\phase-{1,3,4}-summary.md
Backend: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\{lib.rs, commands.rs, projects.rs}
Frontend: F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\
SSB writers: F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\{ssb.rs, ssb_viewer.rs, task_runner.rs}

Overall verdict: PARTIAL with major INVISIBLE surface and three DEAD claims.

The dashboard surfaces only the most superficial knowledge-engine signals
(DKG node/edge counts, BM25 file counts, indexing stage). Every richer
signal that Phases 1A–1I and 3K actually compute — query traces, query
tier decisions, RRF stage breakdowns, search-mode degradation cascade
state, cross-encoder breaker state, freshness/PageRank top symbols,
canary results, ONNX model staging — is either silently dropped at the
producer (NoopSsbNotifier in CLI), stubbed at the SsbViewer (returns
zeros), absent from any Tauri command, or absent from any Svelte
component. The KnowledgeDrawer and Settings Knowledge section reuse the
SAME three queries (readiness + dkg + bm25). There is no “query inspector”
anywhere — the dashboard cannot show what the engine actually did for any
given query.

================================================================================

## Phase 1A — DKG (Dependency Knowledge Graph)

Backend signals available (from indexing_service.rs, dkg/mod.rs,
task_runner.rs:2682–2697):
- DKG node_count (DkgEngine::nodes count) — emitted via SSB DkgReady payload
- DKG edge_count
- languages (Vec<String>) — currently emitted as empty Vec at task_runner.rs:2818
- files_parsed / files_skipped / failed_files (FailedFileInfo)
- per-edge type breakdown (EDGE_IMPORTS / EDGE_CONTAINS / EDGE_IMPLEMENTS)
- signatures_populated count (in metrics/phase_1A.toml)
- DkgReadiness {Ready, Empty, Unavailable}

Dashboard wiring:
- Tauri command get_dkg_stats — commands.rs:3538 → SsbViewer::dkg_stats
  ssb_viewer.rs:2124. Returns DkgStatsSnapshot{project_id, node_count,
  edge_count, languages, ts_ms, known}. REAL — reads latest SSB DkgReady
  row.
- Consumed by:
  - KnowledgeDrawer.svelte:55 (renders nodes, edges, languages)
  - Settings.svelte:274 (same three fields)
- get_knowledge_readiness — commands.rs:3531 → ssb_viewer.rs:2085. Returns
  overall + dkg_ready + indexing_stage + files_processed/total.
- get_indexing_status — commands.rs:146. Reads `.elai_cc/indexes/dkg.db`
  mtime as proxy for "indexed". Consumed by ProjectSelector.svelte:80.
- StatusBar.svelte:141 listens for action_type == "indexing_progress".

Verdict: PARTIAL.
Gap: only 3 of ~10 producible signals surfaced (nodes, edges, languages).
INVISIBLE: failed_files, signatures_populated, edge-type breakdown
(imports/contains/implements), per-language file counts,
data-flow-edge count (DataFlowEdge from A-48), build_time_ms.
Languages field always renders blank because task_runner.rs:2818 hard-codes
`Vec::new()` — DEAD claim: KnowledgeDrawer.svelte:108 prints
`{dkg.languages.join(", ") || "–"}` and will always show "–" until the
producer is fixed.

================================================================================

## Phase 1B — BM25 (Tantivy full-text index)

Backend signals available:
- files_indexed / files_total / files_skipped_binary / chunks_indexed
  (IndexStats, bm25.rs:107)
- elapsed_ms (always 0 — caller-populated only; per Phase 1B audit)
- ChunkKind distribution (Function/Struct/Enum/Impl/Class/Module/Trait/
  LineBlock/File)
- INDEX_FORMAT_VERSION, tantivy_version (in phase_1B.toml)
- per-query: query latency p50/p99 (measure_bm25.rs)
- BM25Hit list (path, line_start, line_end, snippet, score) per query
- index size in bytes (phase_1B.toml: 2456531)

Dashboard wiring:
- Tauri command get_bm25_stats — commands.rs:3545 → ssb_viewer.rs:2148.
  Returns Bm25StatsSnapshot{project_id, files_indexed, files_total,
  last_stage, ts_ms, known}.
- Consumed by KnowledgeDrawer.svelte:56 (renders files + last_stage),
  Settings.svelte:279 (same).

Verdict: PARTIAL.
Gap: chunk_count INVISIBLE (1544 chunks per phase_1B.toml, never shown).
ChunkKind distribution INVISIBLE. Index size INVISIBLE. Per-query
latency p50/p99 INVISIBLE. BM25Hit results never streamed back to dashboard
(ExploreWindow uses sidecar /search, NOT the Rust pipeline.bm25.query).
INVISIBLE: tantivy_version, INDEX_FORMAT_VERSION (would warn the user
when an upgrade is needed; index_version.rs has full UpgradeAction
machinery that never reaches the UI).

================================================================================

## Phase 1C — Vector Search (GraphCodeBERT 768d + Granite 384d via ONNX)

Backend signals available:
- DualVector (code_vec dim=768, text_vec dim=384)
- VectorIndexer chunk count, vector_dim, persistence path
- OrtSession.is_loaded() per model
- VectorCache hit/miss rate (cache.rs)
- HNSW vs BruteForce backend in use
- EmbeddingCircuitBreaker state per model (CLOSED/HALF_OPEN/OPEN +
  consecutive_opens + current_timeout)
- Vector RRF stage results in PipelineStage{stage="vector", results, error,
  circuit_breaker_state} (pipeline.rs:317–326)
- Canary embed("test") result (vector/mod.rs:124)
- ONNX model staging status (file present? loaded? failed?)

Dashboard wiring:
- Tauri command get_vector_stats — commands.rs:3552 → ssb_viewer.rs:2174.
  Returns VectorStatsSnapshot{project_id, docs_estimated, ts_ms, known}.
  docs_estimated is just `files_processed` from indexing_progress (NOT
  a real vector-index doc count).
- NO frontend consumer of get_vector_stats (grep:
  `get_vector_stats` returns zero hits in src-frontend/).
- KnowledgeDrawer.svelte does NOT call get_vector_stats. Settings.svelte
  does NOT call it.

Verdict: INVISIBLE / DEAD.
DEAD: `get_vector_stats` Tauri command exists but has zero consumers.
DEAD: `docs_estimated` is a synonym for files_processed, not a real
vector count — would mislead even if surfaced.
INVISIBLE (high impact): code-model vs text-model load state, ONNX
file staging warnings (per Phase 1C audit, model files are absent on
default checkout — pipeline silently degrades to BM25-only and the user
is never told). Per-model EmbeddingCircuitBreaker state INVISIBLE.
VectorCache hit-rate INVISIBLE (spec target was >=90%).

================================================================================

## Phase 1D — PageRank repo map

Backend signals available:
- RankedSymbol{node_id, symbol_name, file_path, rank, signature} top-N
- RepoMap{ranked_symbols, token_count, density, used_pagerank,
  fit_iterations}
- density value vs threshold (0.3) — used_pagerank true/false
- binary search fit_iterations
- per-snapshot node_count, edge_count

Dashboard wiring:
- Tauri command get_pagerank_top — commands.rs:3559 → ssb_viewer.rs:2196.
- ssb_viewer.rs:2196 IMPLEMENTATION: `Ok(Vec::new())` — STUB returning
  empty list unconditionally.
- NO frontend consumer (grep `get_pagerank_top` returns zero hits in
  src-frontend/).

Verdict: DEAD.
Triple-DEAD: (1) producer is empty stub, (2) no Tauri command consumer in
frontend, (3) per Phase 1D audit, the PageRank stage in pipeline.rs:387 is
WIRED-BUT-DEAD at runtime due to the dual-DB-path bug (cli/main.rs:3752
reads `.elai/dkg.db` while IndexingService writes `.elai_cc/indexes/dkg.db`).
Even if a UI existed, no data would flow.
INVISIBLE: density / used_pagerank / fit_iterations would let the user
see when the binary-search budget fitter is over-shrinking (a documented
gotcha).

================================================================================

## Phase 1E — CatRAG + A-RAG QueryPipeline

Backend signals available:
- CatRagOverlay (sparse anchored-symbol set, key-fact-boosted edge weights)
- AragOrchestrator: tools_used per query (KeywordSearch / SemanticSearch /
  ChunkRead), token_count, ContextTracker read history
- pipeline.rs PipelineStage list per query: bm25, vector, splade,
  pagerank, catrag, arag, rerank, cross_encoder, token_fit, hyde,
  query_expansion, with results count + latency_ms (always 0) +
  circuit_breaker_state per stage
- QueryPipeline.readiness_check CanaryResult{bm25_ok, vector_ok,
  rerank_ok, errors, candidates}
- token_savings vs full-chunk baseline (measured externally in
  measure_1e.rs)

Dashboard wiring:
- ZERO. Grep for `catrag`, `CatRag`, `arag`, `Arag`, `query_pipeline`,
  `PipelineStage`, `query_with_tier` in
  F:\.ELAI_workflow\.elai_cc\crates\dashboard\ returns NO matches.
- ExploreWindow.svelte:104 calls `submit_explore_query` which routes to
  sidecar `/search` HTTP endpoint (commands.rs:2794–2848) — explicitly
  documented as bypassing the Rust QueryPipeline ("the only knowledge-
  engine endpoint available today is GET /search").
- NO Tauri command exposes pipeline stages, tools-used, CanaryResult, or
  CatRAG overlay.

Verdict: INVISIBLE.
The entire CatRAG + A-RAG + QueryPipeline assembly — Phase 1E's headline
deliverable — has zero dashboard surface. The user has no way to know
whether a query went through expansion, HyDE, vector, PageRank, CatRAG,
A-RAG, or fell back to BM25-only. Per Phase 1E audit, in production today
the pipeline is effectively bm25 + rerank + token_fit because (a)
dkg_engine never attaches due to path bug, (b) CatRAG overlay is computed
then discarded at pipeline.rs:432, (c) AragOrchestrator corpus is empty
because nothing calls add_chunk(). The dashboard cannot show this — it
has no inspector.

================================================================================

## Phase 1G — Cross-Encoder Reranker (ms-marco-MiniLM-L12-v2)

Backend signals available:
- CrossEncoderReranker.is_loaded() — bool
- breaker_state for cross_encoder + cross_encoder_fallback
- DegradationEvent emissions ("l12_open", "l6_open" detail keys) from
  reranker.rs:388 — emitted via SsbNotifier
- Rerank pair count + skip_threshold decisions
- Quantization variant in use (fp32 / int8)
- ONNX model staging (cross-encoder.onnx + tokenizer.json present?)
- circuit-breaker recovery (Closed → Closed transition; reranker.rs:397
  emit_recovery_if_needed exists but is a no-op)
- run_batch_worker queue depth + batched-pairs count

Dashboard wiring:
- ZERO Tauri commands exposing reranker state. Grep for `reranker`,
  `cross_encoder`, `breaker_state` in dashboard/src/ returns no command
  definitions.
- ActivityEntry.svelte:382 + 1133 has a `case "knowledge_degradation":`
  branch that returns the label "knowledge · degradation" with color
  "info" (dim gray). The branch only ever fires if an SSB row of action
  type knowledge_degradation actually exists.
- WIRING BREAK: cli/main.rs:3721 constructs the reranker with
  `CrossEncoderReranker::new(ce_config, clock.clone())` — uses NoopSsbNotifier
  (per Phase 1I audit). `with_ssb` is never called in production. So
  reranker degradations NEVER produce SSB rows. The ActivityEntry case
  is unreachable in real runs.

Verdict: DEAD.
DEAD claim: ActivityEntry has `knowledge_degradation` rendering, but the
producer (CrossEncoderReranker) is wired with NoopSsbNotifier — nothing
ever writes the row in production. Test code is the only writer.
INVISIBLE: every reranker signal — load state, model variant (fp32/int8/
fallback), breaker state, recovery transitions, queue depth, pairs/sec,
NDCG@10 metric (would let the user see whether the ONNX is even helping).
Per Phase 1G audit, model fixtures are NOT staged (no
.elai_cc/models/onnx/), so the reranker degrades to RRF-only — and the
dashboard cannot show that this is happening.

================================================================================

## Phase 1I — Resilience layer (cascade, validator, trace, version)

Backend signals available:
- SearchMode enum (Hybrid/CodeOnly/TextOnly/SparseOnly/Bm25Only) — current
  mode of DegradationCascade
- DegradationCascade.evaluate() result, transitions emitted as
  knowledge_degradation SSB rows
- ReadinessStatus {Ready, Degraded, Unavailable} — startup canary result
- CanaryResult with rerank_ok / rerank_error / canary_candidates
- canary_drift cosine similarity vs reference vec (drift < 0.85 → Degraded)
- QueryTrace{query_id (Uuid), stages: Vec<StageTrace>, total_latency_ms,
  mode (String), result_count} — emitted via tracing::debug per query
- StageTrace per stage{stage, latency_ms, input_count, output_count, error,
  circuit_breaker_state}
- SessionTraceStats per-stage p50/p99 (orphaned — no caller)
- IndexVersionSchema + UpgradeAction{Reuse, RebuildBm25Only, FullRebuild}
- BreakerState per (model_id) — independent breakers
- QueryValidator rejection reason (empty/oversized/control-char)

Dashboard wiring:
- ZERO Tauri commands for any of these. No get_query_trace, no
  get_search_mode, no get_canary_result, no get_index_version,
  no get_breaker_states.
- DegradationCascade is constructed once in cli/main.rs:3800 and used
  only for the startup readiness check; never re-evaluated mid-session
  (per Phase 1I audit). knowledge_degradation SSB events fire only at
  startup.
- KnowledgeReadinessSnapshot.overall → readiness.overall is rendered in
  KnowledgeDrawer.svelte:85,95 ("ready"/"degraded"/"unavailable" string).
  This is the sole production-visible ReadinessStatus surface.
- ActivityEntry has the dim "knowledge · degradation" label but no detail
  payload renderer — the user sees a one-line dim entry with no mode,
  no component, no recovery info.

Verdict: PARTIAL — overall ReadinessStatus surfaced; everything else INVISIBLE.
DEAD: QueryTrace's mode field is hardcoded `SearchMode::Hybrid` at
pipeline.rs:520 (per Phase 1I audit); even if the dashboard fetched
traces, the mode would always lie.
INVISIBLE (high impact): per-query trace, per-stage latency, current
SearchMode, drift cosine, canary candidates, individual breaker states,
index_version upgrade decisions. The user cannot see WHY the engine is
degraded, only that it is.

================================================================================

## Phase 3K — RAG production hardening

Backend signals available:
- AST chunking decision per file (try_ast_chunk vs line_block_chunks
  fallback, ChunkKind per chunk)
- QueryTier enum (Fast/Standard/Deep/Auto) — auto-detection rules + final
  resolved tier per query (lib.rs:65–85)
- Per-tier latency p50/p95 (phase_3K_rag.toml)
- Per-tier Recall@5/Recall@10/MRR (phase_3K_rag.toml)
- Per-category metrics (simple/code/complex)
- HyDE hypothesis text per query (when DEEP tier + sidecar healthy)
- HyDE cache hit rate (5-min TTL) and circuit-breaker state
- HyDE sidecar latency
- Query expansion: variants generated per query (Keywords / CodeSymbol /
  Synonym strategies)
- A-RAG keyword-search vs semantic-search vs chunk-read tool selection
  per query
- A-RAG token-budget consumption
- progressive-disclosure "(previously read)" replays
- token_savings vs full-chunk baseline (improved.toml: -94.3% measured)
- CatRAG symbolic-anchor matches per query (DKG entity lookups)
- CatRAG key-fact-enhanced edges per query (beta=2.5 boost)
- benchmark regression-gate status (PASS/FAIL vs 5% threshold)

Dashboard wiring:
- get_query_tier_stats — commands.rs:3570 → ssb_viewer.rs:2205. Returns
  QueryTierStats{fast_pct, standard_pct, deep_pct, avg_latency_ms_*,
  sample_size}. STUB — body is `Ok(QueryTierStats{ ... 0.0 ... 0 })`.
- get_freshness_state — commands.rs:3577 → ssb_viewer.rs:2219. STUB —
  returns FreshnessState{0.0, 0.0, 0, known: false}.
- NO Tauri command exposes per-query trace, HyDE hypothesis, expansion
  variants, A-RAG tool choice, CatRAG overlay, benchmark metrics, or
  token-savings.
- KnowledgeDrawer.svelte own comment line 5 admits: "pagerank_top,
  query_tier_stats, freshness_state are exposed but currently stubbed
  server-side; the drawer renders '–' for them" — and then doesn't even
  render them.

Verdict: DEAD.
Triple-DEAD per signal: producer absent (no SSB row type for query trace
or tier decisions), viewer is a hardcoded zero stub, no Svelte consumer.
INVISIBLE: Recall@10 / MRR (measured in metrics/phase_3K_rag.toml,
read by no Tauri command), per-tier latency distribution,
token-savings -94.3% improvement (a headline number with ZERO surface),
HyDE quality, CatRAG anchor coverage, A-RAG tool mix.

================================================================================

## Cross-cutting wiring gaps (knowledge)

C1. ONE knowledge query inspector for ALL phases. The dashboard has 9
windows + 10 drawers + 98 Tauri commands but NO surface that shows
"what the engine did for this query" — the union of QueryTier, expansion
variants, HyDE hypothesis, BM25 hits, vector hits, PageRank ranking,
CatRAG anchors, A-RAG tool choice, reranker scores, final fused
results. Phase 3J defined QueryTrace exactly for this; the surface is
zero.

C2. SsbViewer has THREE knowledge stubs that every drawer ignores:
pagerank_top → empty Vec, query_tier_stats → all zeros, freshness_state
→ all zeros (ssb_viewer.rs:2196–2226). Five Tauri commands ride on top
(get_pagerank_top, get_query_tier_stats, get_freshness_state plus the
ones that just echo files_processed). They are dashboard-visible
function names that the audit tooling counts as "wired" but they ship
hard-coded zero output. This is the largest category of DEAD wiring in
the engine.

C3. NoopSsbNotifier is the production reality. cli/main.rs:3721 attaches
the reranker without ssb, and DegradationCascade is constructed once and
discarded. ActivityEntry has a `knowledge_degradation` case that
practically never fires. To fix the dashboard, the wiring path
backend → SSB → viewer must be established before any new UI is built.

C4. Dual DB path bug (per Phase 1A, 1D, 1E audits) — IndexingService
writes `.elai_cc/indexes/dkg.db` but cli/main.rs:3752,5437 reads
`.elai/dkg.db`. PageRank, telescope, CatRAG enrichment all see empty
data. The dashboard correctly uses `.elai_cc/indexes/dkg.db`
(commands.rs:185–188) — meaning the DASHBOARD-side reader works but the
CLI-side reader does not. Any "PageRank top" surface added to the
dashboard would still see empty results until the CLI bug is fixed.

C5. Languages list always blank. task_runner.rs:2818 emits DkgReady with
`Vec::new()` for languages. KnowledgeDrawer.svelte:108 prints
"languages: –" forever. Fix is one line in task_runner.rs but it touches
the visible UI immediately.

C6. ONNX model staging warnings absent. Phase 1C and 1G audits both
flag that the default checkout has no .elai_cc/models/onnx/ directory,
so vector + reranker silently degrade. The dashboard has zero surface
for "model files missing" — no warning banner, no readiness gate, no
Settings panel. Users are left wondering why search quality is poor.

C7. trigger_reindex is a stub (commands.rs:4054) — emits a Finding SSB
row with text "reindex queued ... sidecar restart required". No actual
reindex happens. Settings.svelte:289 calls it and reports success
silently — DEAD claim that the user can re-trigger indexing from the UI.

C8. Anti-Staleness drawer is a documented stub (AntiStalenessDrawer.svelte
lines 8–13: "drawer reads nothing and renders 'no data yet' for each of
the three sections"). Fine as documentation but contributes to the
"feels empty" perception.

C9. ExploreWindow bypasses the Rust pipeline entirely. The user-facing
"explore" experience hits sidecar /search HTTP — none of CatRAG, A-RAG,
PageRank, reranker, freshness steering, query expansion, HyDE participate.
The user is searching with sidecar's basic HTTP endpoint and never sees
the rich pipeline.

================================================================================

## Recommended dashboard surfaces (what is missing the user likely wants)

R1. Query Inspector window. Per-query trace: tier decided, stages fired
(BM25/vector/PageRank/CatRAG/A-RAG/rerank), per-stage latency, per-stage
result count, per-stage error/breaker state, final fused top-N with
score breakdown. Source: pipeline.rs PipelineStage already records this;
needs an SSB action_type and a viewer query.

R2. Engine Health panel (in Settings or new window):
  - Per-model load state (graphcodebert / granite / cross_encoder /
    cross_encoder_fallback / splade) — loaded? path? size? ONNX missing?
  - Per-model EmbeddingCircuitBreaker state (Closed/HalfOpen/Open) +
    consecutive_opens + current_timeout
  - Current SearchMode from DegradationCascade (Hybrid/CodeOnly/...)
  - Drift cosine vs canary reference + last canary time
  - VectorCache hit rate

R3. Reranker panel (or merge into R2):
  - Variant in use (fp32 / int8 / fallback L-6 / RRF-only)
  - Pairs scored / sec
  - NDCG@10 latest measurement (read from metrics/phase_1G.toml when present)
  - Skip-threshold decisions (queries skipped because <10 candidates)

R4. Knowledge Index panel (replace KnowledgeDrawer):
  - DKG: nodes / edges / per-edge-type (imports/contains/implements) /
    per-language counts / failed_files list
  - BM25: chunks / per-ChunkKind distribution / index size / format version
  - Vector: code-vec count / text-vec count / cache hit rate /
    HNSW-or-bruteforce
  - Index version + last UpgradeAction (Reuse / RebuildBm25Only / FullRebuild)
  - Real "Reindex" button that invokes IndexingService::run_full_index,
    not the stub Finding.

R5. RAG metrics panel (read from .elai_cc/metrics/phase_3K_rag*.toml):
  - Per-tier Recall@5/10, MRR, p50/p95 latency
  - Per-category (simple/code/complex)
  - Token-savings -94.3% headline number
  - Regression-gate PASS/FAIL since baseline

R6. PageRank top window/drawer: top-N RankedSymbol{symbol_name, file_path,
rank, signature} so the user can SEE the ranked symbols (after C4 is fixed).

R7. Freshness panel: A-13 freshness steering vector is visible in
Settings.svelte (steering vectors row) BUT the actual freshness decay
state (FreshnessState{decay_rate, access_boost, last_recalc_ms}) is a
stub. Activate the producer + show real decay state.

R8. Inline knowledge_degradation cards in ActivityFeed: when a degradation
fires, render a yellow card with mode_before → mode_after, component name,
recovery hint — instead of a one-line dim "info" entry.

R9. Move ExploreWindow off /search and onto QueryPipeline so the
inspector (R1) actually has something to show. Today the user-facing
search UI bypasses every interesting Phase 1B–3K signal.

================================================================================

## Summary

Backend signals identified across 8 phases: ~60 distinct producible signals.

| Phase | Signals exposed | SURFACED | PARTIAL | INVISIBLE | DEAD |
|------:|---------------:|---------:|--------:|----------:|-----:|
| 1A    | 10             | 3        | 1       | 6         | 0    |
| 1B    | 8              | 2        | 0       | 6         | 0    |
| 1C    | 9              | 0        | 0       | 8         | 1    |
| 1D    | 5              | 0        | 0       | 4         | 1    |
| 1E    | 6              | 0        | 0       | 6         | 0    |
| 1G    | 8              | 0        | 0       | 7         | 1    |
| 1I    | 9              | 1        | 0       | 7         | 1    |
| 3K    | 12             | 0        | 0       | 10        | 2    |
| **Total** | **67**     | **6**    | **1**   | **54**    | **6** |

Surfaced: 6 (~9%) — only DKG node/edge/languages, BM25 files/stage,
ReadinessStatus.overall, indexing_progress.
Partial: 1 (~2%) — DKG languages list field exists but is always empty.
Invisible: 54 (~81%) — every other knowledge signal has no Tauri command
or no consumer.
Dead: 6 (~9%) — get_pagerank_top stub + get_query_tier_stats stub +
get_freshness_state stub + ActivityEntry knowledge_degradation case
(producer is Noop) + KnowledgeDrawer languages field (producer hard-codes
empty Vec) + trigger_reindex (stub Finding).

Top INVISIBLE signals (highest user-impact):
- Per-query trace (pipeline stages, tier, mode, latency)
- Current SearchMode + cascade transitions
- Per-model ONNX load state + circuit breaker states
- Reranker variant + NDCG@10
- HyDE hypothesis + cache hit rate
- CatRAG overlay + A-RAG tool choice
- Recall@5/10 + MRR per tier
- Token-savings -94.3% measurement
- VectorCache hit rate (spec target >=90%)
- Index format upgrade decisions

Top DEAD claims (UI element renders, backend is empty):
- get_pagerank_top → ssb_viewer.rs:2196 returns Vec::new()
- get_query_tier_stats → ssb_viewer.rs:2205 returns all zeros
- get_freshness_state → ssb_viewer.rs:2219 returns known:false
- ActivityEntry case "knowledge_degradation" → CrossEncoderReranker uses
  NoopSsbNotifier in CLI (cli/main.rs:3721); rows are never written
- KnowledgeDrawer "languages" line → task_runner.rs:2818 hard-codes
  Vec::new()
- Settings "Reindex" button → trigger_reindex emits a Finding, doesn't
  actually reindex

Verdict: PARTIAL with major INVISIBLE surface. The dashboard shows that
the engine RAN but never WHAT the engine did. To match the user's
"feels and looks like I wanted", the priority is not new features — it
is wiring the producer→SSB→viewer→UI chain for QueryTrace, SearchMode,
per-model load state, and the metrics already sitting in
.elai_cc/metrics/phase_*.toml unread by any Tauri command.
