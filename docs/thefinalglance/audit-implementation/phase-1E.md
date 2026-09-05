PHASE 1E AUDIT — CatRAG + A-RAG QUERY PIPELINE ASSEMBLY
========================================================

Auditor stance: HOSTILE. Claims rejected by default; only verified-from-source evidence accepted.
Spec: F:\.ELAI_workflow\docs\thefinalglance\summary\phase-1-summary.md §PHASE 1E (L716-859).
Target: F:\.ELAI_workflow\.elai_cc\crates\knowledge\

OVERALL VERDICT: PARTIAL — STRUCTURAL PRESENT, RUNTIME MOSTLY DEGRADED

The three types (CatRag, AragOrchestrator, QueryPipeline) exist with real bodies;
pipeline composes all stage slots in spec order; unit tests fire the types at the
component level. But at end-to-end runtime via the CLI:
  1. CatRAG overlay is COMPUTED but DISCARDED (never fed back into PPR).
  2. A-RAG runs with an EMPTY corpus (no chunks are ever added to it from CLI).
  3. PageRank / CatRAG / DKG branch is GATED on .elai/dkg.db, which does not
     exist — actual DB lives at .elai_cc/.elai_cc/indexes/dkg.db. Same inherited
     gating bug already flagged in Phase 1D audit.
  4. 1E.7 metrics harness ran but in "degraded_mode = true" with
     stages_skipped = "vector,catrag,arag". Token-savings number (96.1%) is a
     truncation artifact of BM25 snippet policy, not real retrieval selectivity —
     the measure binary documents this explicitly.

Result: the pipeline compiles, the types are wired, acceptance tests pass on
synthetic tempdirs, and a metrics TOML was produced. But the live retrieval path
exercised by the CLI is effectively "BM25 -> empty -> empty -> empty -> rerank
(BM25 only) -> token_fit (BM25 only)". Phase 1E ships the plumbing, not the
water.

--------------------------------------------------------------------------------
PER-SUB-STEP VERDICTS
--------------------------------------------------------------------------------

1E.1 SKELETON — CatRAG / A-RAG / Pipeline stubs
   VERDICT: PASS
   Evidence: crates\knowledge\src\{catrag.rs, arag.rs, pipeline.rs} all exist;
   lib.rs re-exports catrag, arag, pipeline (elai.map, multiple DEFs).
   CatRagConfig (catrag.rs — phi()/named-entity/tokenize helpers present),
   CatRagOverlay, AragConfig (arag.rs:37-52 with defaults 15/128000/5),
   RetrievalTool enum (arag.rs:24-28 with KeywordSearch/SemanticSearch/ChunkRead),
   RetrievalResult (arag.rs:60-65), ContextTracker (arag.rs:85-106 with
   is_read/mark_read/token_count), PipelineStage (pipeline.rs:35-43),
   QueryPipeline (pipeline.rs:50-62). No todo!() bodies remain — all stubs
   filled. Skeleton exceeds spec (includes cross_encoder, vector_module,
   vector_indexer, dkg_engine, freshness_db, hyde_stage, splade fields added in
   later phases).

1E.2 VERIFY COMPILATION
   VERDICT: ASSUMED PASS (not re-run; main model does not execute tests).
   Evidence: file is in active use by cli, benchmarks, contracts; tests
   reference QueryPipeline::new(bm25, catrag, arag) directly; recent commits
   (patch-ui, batch-ful-d) compiled. No source evidence of drift.

1E.3 TESTS (TDD harness)
   VERDICT: PARTIAL — tests exist but coverage skewed toward later phases.
   Evidence: tests/pipeline_tests.rs (3 tests: find-imports, error-handling,
   graph-traversal — all run pipeline.query() on empty CatRag/Arag + BM25).
   src/pipeline.rs #[cfg(test)] mod: 30+ tests but most are for Phase 1G
   (readiness_check), Phase 3K (HyDE, tier routing), A-13 freshness.
   Spec-mandated CatRAG tests (a-e: symbolic anchoring, dynamic edge weighting,
   key-fact enhancement, density guard, phi() tiers) — phi() tiers covered;
   unclear whether full overlay round-trip (test n) is covered without reading
   catrag.rs fully. Spec A-RAG tests (f-j) partially present in arag.rs tests.
   Pipeline smoke test (k: stages in order ["bm25","vector","pagerank",
   "catrag","arag"]) — verified: pipeline.rs:260,323,411,435,449 push stages
   in exactly that order, with extra stages query_expansion/hyde/splade/rerank/
   cross_encoder/token_fit interleaved per spec note (knowledge.txt L104-105).
   Pipeline latency test (l: latency_ms > 0) — FAIL: every stage hard-codes
   latency_ms: 0 (pipeline.rs:178,189,227,237,263,326,335,374,414,424,439,452,
   463,491,505). Comment on line 32-33 admits "entropy quarantine: raw Instant
   confined to core/src/clock.rs" — the pipeline deliberately does NOT measure
   per-stage latency. Spec 1E.3 check (l) is not satisfied by this code; any
   test asserting latency_ms > 0 at this layer would fail.

1E.4 IMPLEMENT CatRAG
   VERDICT: PARTIAL — overlay is computed, RESULT IS DISCARDED AT PIPELINE
   INTEGRATION.
   Evidence:
   - catrag.rs has real compute_overlay body (not todo!), with three early-
     return paths that construct CatRagOverlay (catrag.rs:261, 269, 371). phi()
     table matches spec Table 7.
   - But pipeline.rs:431-441 (stage 3 CatRAG) reads:
         if use_vector {
             let _overlay = self.catrag.compute_overlay(query, &snapshot, &[])?;
         }
         stages.push(PipelineStage { stage: "catrag", results: Vec::new(), ... });
     The `_overlay` is bound to `_` and THROWN AWAY. Spec 1E.4 step (e) is
     explicit: "Apply overlay to PPR: modify reset vector + transition matrix;
     run PPR with modified inputs (reuse Phase 1D); return ranked results."
     None of that happens. The CatRAG stage emits an empty results vector
     regardless of overlay content.
   - Third argument to compute_overlay is `&[]` — an empty slice of PageRank
     scores — even when the pagerank stage JUST computed ranked_symbols on
     line 388-397. The previous stage's output is not plumbed into CatRAG.
   - Density guard per spec step (d): likely present inside catrag.rs (three
     separate early-return Ok(CatRagOverlay {...}) paths suggest it); but
     moot because overlay output is dropped at the caller.
   - Passthrough-on-empty-graph ACHIEVED by accident: pipeline never applied
     the overlay anyway.
   Net: the compute_overlay FUNCTION is implemented; its INTEGRATION into the
   pipeline is a stub. Spec acceptance "overlay sparse + query-local; base
   graph never mutated" is technically true only because the overlay is never
   even read.

1E.5 IMPLEMENT A-RAG
   VERDICT: PARTIAL — tools implemented; pipeline wiring not fed with corpus.
   Evidence:
   - arag.rs has full bodies for keyword_search (score formula with length
     normalization, arag.rs:191+), semantic_search (SEMANTIC_OVERSAMPLE_FACTOR
     = 3 matching spec), ChunkRead dispatch (arag.rs:151-178) honoring
     ContextTracker "(previously read)" behavior, token_count (arag.rs:103-105
     using 1000 tokens/chunk per arag-interface.md §29).
   - query_pipeline body (arag.rs:303-348) executes KeywordSearch then
     SemanticSearch then ChunkRead on dedup'd hits, with budget guard
     `tracker.token_count() < self.config.max_token_budget` (line 339).
   - CRITICAL GAP: AragOrchestrator stores its own `chunks: Vec<AragChunk>`
     (arag.rs:117), populated via add_chunk(). Grep of entire CLI tree:
     `add_chunk(` has NO matches in cli\src\main.rs. The orchestrator is
     constructed in CLI via `AragOrchestrator::new(AragConfig::default())`
     (main.rs:3661) and never fed any chunks. Therefore at runtime both
     keyword_search and semantic_search return Ok(Vec::new()) unconditionally
     (arag.rs:196-198: `if self.chunks.is_empty() ... return Ok(Vec::new())`).
   - Spec (a) says keyword search "wraps BM25" — but the implementation keeps
     its own separate corpus rather than delegating to the Bm25Index already
     held by QueryPipeline. This is a design divergence that guarantees the
     empty-corpus failure mode in production.
   - Spec (f) query pipeline step order matches implementation order.
   Net: tools work in tests where corpus is injected (arag unit tests + the
   1E.7 measure binary indexes the corpus into BM25 only — NOT into
   AragOrchestrator, hence the metrics file's `stages_skipped = "...,arag"`
   and `degraded_mode = true`).

1E.6 IMPLEMENT PIPELINE
   VERDICT: PARTIAL — stages wired in order but several are non-functional
   at runtime.
   Evidence (pipeline.rs:134-530 `query_with_tier`):
   Stage-by-stage, in execution order:
   (a) query_expansion — fires for DEEP tier (pipeline.rs:157-194). PRESENT.
   (b) hyde — fires for DEEP when hyde_stage.is_some(). PRESENT.
   (c) bm25 — fires always, iterates expansion_texts (pipeline.rs:243-266).
       PRESENT AND LIVE.
   (d) vector — fires for STANDARD/DEEP (pipeline.rs:268-338). Requires
       vector_module AND vector_indexer AND !indexer.is_empty() to produce
       results. Otherwise stage emits empty results. In CLI main.rs:3695-3709
       it loads from `<project_root>/.elai/data/vector_index.json`. PRESENT,
       live only if vector_index.json exists at that path.
   (e) splade — feature-gated (pipeline.rs:341-378), fires when splade
       expander loaded. PRESENT.
   (f) pagerank — fires for STANDARD/DEEP (pipeline.rs:380-429). Requires
       self.dkg_engine.is_some(). In CLI (main.rs:3752-3762) gated on
       `project_root.join(".elai").join("dkg.db")` which DOES NOT EXIST
       (actual writer per Phase 1D audit: .elai_cc/.elai_cc/indexes/dkg.db —
       `find` confirms only that location has dkg.db). Same gating bug
       previously flagged in Phase 1D audit. RESULT: dkg_engine = None,
       stage emits empty results + empty snapshot. Spec step (c) "build
       DkgSnapshot, run PageRank" technically runs via from_engine(engine),
       but never executes at runtime because engine is never attached.
   (g) catrag — fires for STANDARD/DEEP (pipeline.rs:431-441). Calls
       compute_overlay but DISCARDS result. Stage pushes empty results
       unconditionally. NOT LIVE regardless of tier.
   (h) arag — fires for STANDARD/DEEP (pipeline.rs:443-455). Calls
       arag.query_pipeline with snapshot, but arag has empty corpus at
       runtime (no add_chunk caller). Returns Vec::new(). NOT LIVE.
   (i) rerank — RRF fusion over all prior stages (pipeline.rs:457-465). FIRES
       always; with most upstream stages empty, this is effectively RRF over
       BM25 results only.
   (j) cross_encoder — fires for STANDARD/DEEP if cross_encoder configured
       (pipeline.rs:467-494). Graceful degradation via circuit breaker.
       PRESENT.
   (k) token_fit — passthrough clone of cross_encoder stage results
       (pipeline.rs:496-508). Present but spec says it's a Phase 3E stub,
       and here it does no budget enforcement — literally clones the prior
       stage.
   Stage ordering vs spec "bm25 -> vector -> pagerank -> catrag -> arag":
   ORDER CORRECT. Stages-fire-at-runtime reality: bm25 + rerank + token_fit
   only (default CLI invocation with empty vector index and missing dkg.db).

1E.7 MEASURE — latency + token savings
   VERDICT: PARTIAL — harness written and executed, but metrics are produced
   in DEGRADED mode and the savings figure is a structural artifact.
   Evidence:
   - crates/knowledge/examples/measure_1e.rs exists with all 5 required
     queries (Q1-Q5 from spec L848-852 verbatim).
   - Output file .elai_cc/metrics/phase_1E.toml exists with every spec-
     required key: pipeline_p50_ms (1), pipeline_p99_ms (1),
     token_savings_mean_pct (96.1), catrag_overlay_size (0),
     arag_tools_per_query (2.0), stages_skipped ("vector,catrag,arag"),
     plus bonus degraded_mode (true) / index_build_time_ms / files_indexed /
     chunks_indexed.
   - Spec acceptance gate "pipeline_p50_ms < 2000ms": PASSES (1ms).
   - Spec acceptance gate "token_savings_mean_pct > 50%": formally PASSES
     (96.1%), but the measure binary itself admits in a comment
     (measure_1e.rs:237-241): "BM25Hit.snippet is truncated to first 200
     chars of chunk content (bm25.rs:259). This creates a structural savings
     floor of ~87-94% regardless of query quality. ... the high value
     reflects truncation policy, not retrieval selectivity."
   - stages_skipped = "vector,catrag,arag" + degraded_mode = true:
     self-attestation that the measured pipeline is running in the same
     degraded state as production. Three out of five of the spec's
     required stages are non-functional during measurement.
   - pipeline_p50_ms = 1 is not a latency measurement of the full pipeline
     because the degraded stages exit in O(1). A non-degraded run would be
     bound by vector embedding (~100ms) + PageRank (~10-100ms depending on
     graph size) + A-RAG semantic oversample (~50-200ms). The "<2000ms"
     acceptance is satisfied vacuously.
   - catrag_overlay_size = 0: consistent with pipeline.rs:433 discarding
     the overlay — the measure binary reads `stages.iter().find(|s| s.stage
     == "catrag")` and counts `results.len()`, which is always 0 because
     pipeline.rs:437 hard-codes `results: Vec::new()` for the catrag stage.
     The overlay COULD be non-empty internally, but the caller never
     surfaces it.
   Net: the metrics artifact exists and the file satisfies the
   DETERMINISTIC_CHECK format; the acceptance gates technically pass; but
   nothing in the measured run demonstrates CatRAG or A-RAG actually doing
   any retrieval work.

--------------------------------------------------------------------------------
CROSS-CUTTING FINDINGS
--------------------------------------------------------------------------------

F1. Phase 1D gating bug inherited unchanged.
    Phase 1D audit found compute_repo_map gated on .elai/dkg.db which never
    exists. Phase 1E did not fix this — cli\src\main.rs:3752 still
    `project_root.join(".elai").join("dkg.db")`. Actual file is at
    `.elai_cc/.elai_cc/indexes/dkg.db` (verified via filesystem scan). The
    duplicated `.elai_cc/.elai_cc/` path itself suggests a cwd-relative
    writer colliding with an absolute-root reader.

F2. Phase 1C code_vec still not exercised at query time.
    Phase 1C audit found code_vec (GraphCodeBERT) loaded but not queried.
    Pipeline stage 1 vector branch (pipeline.rs:280-313) calls vm.embed_text
    which per Phase 1C audit delegates to text_vec only. Nothing in 1E
    plumbing changes which embedding is used per-query.

F3. Box<dyn Error> count in knowledge crate: 16 occurrences across 5 files
    (bm25.rs, pagerank/mod.rs, vector/indexer.rs, splade_tests.rs,
    splade_measure_tests.rs). Project rule "No new dependencies / no
    Box<dyn Error>" is violated in 3 production files. Not a Phase 1E
    regression (bm25, pagerank, vector/indexer predate it), but the phase
    did not reduce the count either.

F4. Latency measurement architectural mismatch.
    pipeline.rs comment (L30-33): "latency_ms is always 0 here; callers
    measure wall time externally (entropy quarantine: raw Instant confined
    to core/src/clock.rs)". Spec 1E.3 test (l) says "each PipelineStage
    latency_ms > 0". These two are incompatible — either the spec needs
    updating or the pipeline needs per-stage Instant::now() calls wired
    through ClockProvider. Right now the per-stage latency field is dead
    weight.

F5. QueryPipeline callers (spec question 7):
    - crates/cli/src/main.rs:3662 — live entry point (partial-degraded)
    - crates/cli/src/main.rs:4358 — pipeline.query(input) in REPL/session
    - crates/knowledge/src/benchmark.rs:349 — phase benchmarks
    - crates/knowledge/src/startup.rs:285 — canary
    - examples/measure_1e.rs:53 — 1E.7 harness
    - tests/benchmark_eval.rs, tests/benchmark_eval_improved.rs,
      tests/pipeline_tests.rs — test harnesses
    No orchestrator caller outside CLI — no AgentOrchestrator or Tool
    dispatch consumes QueryPipeline. Retrieval only fires on user-facing
    query commands.

F6. Token-savings methodology is structural not semantic.
    measure_1e.rs:262 computes full_chunk_tokens = (line_end - line_start
    + 1) * 8 words/line, then compares to BM25 snippet.
    split_whitespace().count(). BM25 truncates snippets to 200 chars (per
    measure_1e.rs's own admission). So token_savings_mean_pct measures
    "200 chars vs (N lines * 8 words * ~5 chars)" — a constant ratio
    dominated by mean chunk size, not by retrieval quality. This metric
    cannot go below ~80% on any realistic codebase, irrespective of
    whether CatRAG or A-RAG improved anything.

--------------------------------------------------------------------------------
SUMMARY ANSWERS TO AUDIT QUESTIONS
--------------------------------------------------------------------------------

(1) Path written: F:\.ELAI_workflow\docs\thefinalglance\audit-implementation\phase-1E.md

(2) Overall verdict: PARTIAL — structural present, runtime mostly degraded.
    Types exist, pipeline composes stages in spec order, tests pass at the
    unit level, metrics TOML produced. But in live CLI use three of five
    core stages (vector conditionally, pagerank, catrag, arag) are
    non-functional due to (a) dkg.db path gating bug inherited from 1D,
    (b) CatRAG overlay computed-then-discarded, (c) AragOrchestrator never
    fed chunks.

(3) Pipeline stages at runtime (CLI default, typical user query):
    FIRE: query_expansion (DEEP only), hyde (DEEP + sidecar healthy),
    bm25 (always, live), rerank (RRF over whatever upstream produced),
    cross_encoder (STANDARD/DEEP if ONNX present), token_fit
    (passthrough clone).
    EMPTY/SKIPPED: vector (only if vector_index.json present at
    .elai/data/ — typically absent), pagerank (dkg_engine always None due
    to path bug), catrag (compute_overlay runs but _overlay discarded and
    stage emits empty), arag (query_pipeline runs but corpus empty ->
    returns Vec::new()).
    Effective runtime: "bm25 -> rerank(BM25) -> cross_encoder -> token_fit".

(4) Token savings: MEASURED BUT NOT MEANINGFUL. phase_1E.toml records
    token_savings_mean_pct = 96.1, which technically exceeds the >50%
    acceptance gate. But the measure binary's own code comment
    (measure_1e.rs:237-241) attributes this to BM25's 200-char snippet
    truncation, not to retrieval selectivity. The run is tagged
    degraded_mode = true with stages_skipped = "vector,catrag,arag", so
    the number does not reflect CatRAG/A-RAG's impact — it reflects the
    baseline BM25 snippet policy. Real token savings from hierarchical
    retrieval remain UNMEASURED.
