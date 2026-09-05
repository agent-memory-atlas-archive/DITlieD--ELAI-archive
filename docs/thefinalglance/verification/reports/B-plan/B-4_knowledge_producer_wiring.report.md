# Verification Report: B-4_knowledge_producer_wiring
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-4_knowledge_producer_wiring.md
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary

The knowledge engine backend (degradation cascade, DegradationCascade struct, SsbNotifier trait, error types, HyDE stage, 3-ranker RRF, circuit breaker infrastructure) is well-built. The stage_models scripts exist and the three ONNX model files are physically present on disk. However, the frontend surface is almost entirely unwired: `get_query_trace_recent` returns `[]`, `get_search_mode` returns hardcoded "HYBRID", `get_recall_metrics` returns static constants, `get_canary_drift` and `get_pagerank_top` have no Tauri command implementations at all, `vector_cache_hit_rate` SSB action type does not exist, `hyde_query` SSB action type does not exist, and all per-stage `latency_ms` fields remain hardcoded to 0 in pipeline.rs. The WIRING_CONTRACT stub_ban on NoopSsbNotifier is violated: it is still the production default in reranker.rs constructors (not wired via cli).

## Claims verified DONE

- **DegradationCascade struct + SearchMode enum defined** — evidence: [degradation.rs:39,79](.elai_cc/crates/knowledge/src/degradation.rs)
- **SsbNotifier trait defined** — evidence: [degradation.rs:18](.elai_cc/crates/knowledge/src/degradation.rs)
- **knowledge_degradation SSB action type defined and emitted by DegradationCascade** — evidence: [degradation.rs:144](.elai_cc/crates/knowledge/src/degradation.rs), [reranker.rs:390](.elai_cc/crates/knowledge/src/reranker.rs)
- **circuit_breaker_state_change SSB action type defined in schema** — evidence: [ssb.rs:246,1046,1223](.elai_cc/crates/orchestrator/src/ssb.rs)
- **query_trace SSB action type emitted (tracing::debug only, not SSB row write)** — evidence: [pipeline.rs:644](.elai_cc/crates/knowledge/src/pipeline.rs)
- **.expect("semaphore closed") removed from reranker.rs:465** — evidence: [reranker.rs:465-474](.elai_cc/crates/knowledge/src/reranker.rs), replaced with graceful match/warn
- **Box<dyn Error> removed from knowledge crate public APIs** — evidence: [error.rs](.elai_cc/crates/knowledge/src/error.rs) defines KnowledgeError; [vector/mod.rs](.elai_cc/crates/knowledge/src/vector/mod.rs) uses VectorError; no Box<dyn Error> found in bm25.rs or pagerank/mod.rs
- **stage_models scripts created (.sh + .bat + .py)** — evidence: [scripts/stage_onnx_models.sh](.elai_cc/scripts/stage_onnx_models.sh), [scripts/stage_onnx_models.bat](.elai_cc/scripts/stage_onnx_models.bat), [scripts/stage_onnx_models.py](.elai_cc/scripts/stage_onnx_models.py)
- **ONNX model files physically present on disk** — evidence: `test -f` confirmed all three: cross-encoder-l12.onnx, graphcodebert.onnx, granite-embed.onnx in `.elai_cc/models/onnx/`
- **3-ranker RRF (code_vec stage) implemented in pipeline.rs** — evidence: [pipeline.rs:375-421](.elai_cc/crates/knowledge/src/pipeline.rs), code_vector stage wired in STANDARD+DEEP tiers
- **HyDE stage integrated in pipeline.rs (DEEP tier)** — evidence: [pipeline.rs:227](.elai_cc/crates/knowledge/src/pipeline.rs), SidecarHydeClient wired in live_cli.rs:1056-1081
- **get_query_trace_recent Tauri command registered** — evidence: [lib.rs:91](.elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs), [commands.rs:824](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs)
- **get_search_mode Tauri command registered** — evidence: [lib.rs:92](.elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs), [commands.rs:831](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs)
- **get_embedding_breakers Tauri command registered and partially live** — evidence: [lib.rs:93](.elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs); reads SsbViewer.embedding_breakers() with hardcoded fallback; [commands.rs:840-863](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs)
- **get_recall_metrics Tauri command registered** — evidence: [lib.rs:94](.elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs), [commands.rs:867](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs)
- **get_dkg_stats Tauri command registered and live** — evidence: [lib.rs:95](.elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs); reads SsbViewer.dkg_stats()
- **KnowledgeDrawer.svelte renders breakers grid (section 4)** — evidence: [KnowledgeDrawer.svelte:229-240](.elai_cc/crates/dashboard-v2/src/lib/drawers/KnowledgeDrawer.svelte)
- **KnowledgeDrawer.svelte renders RECALL @ TIER section** — evidence: [KnowledgeDrawer.svelte:255-274](.elai_cc/crates/dashboard-v2/src/lib/drawers/KnowledgeDrawer.svelte)
- **DegradationCascade wired at startup in live_cli.rs** — evidence: [live_cli.rs:1086-1103](.elai_cc/crates/cli/src/live_cli.rs), cascade evaluated; startup_readiness_check called

## Claims PARTIAL

- **get_query_trace_recent returns live data**
  - exists: [commands.rs:824-828](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs) — command registered
  - gap: returns `json!([])` unconditionally; no SSB row of type `query_trace` is written; KnowledgeDrawer marks `unwired.recentTraces = true` at [drawers.svelte.ts:149](.elai_cc/crates/dashboard-v2/src/lib/state/drawers.svelte.ts)

- **get_search_mode returns live SearchMode from cascade**
  - exists: [commands.rs:831-837](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs) — command registered
  - gap: hardcoded `"HYBRID"` response; cascade result not propagated to Tauri state; KnowledgeDrawer marks `unwired.searchMode = true` at [drawers.svelte.ts:150](.elai_cc/crates/dashboard-v2/src/lib/state/drawers.svelte.ts)

- **get_recall_metrics returns live retrieval quality measurements**
  - exists: [commands.rs:867-875](.elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs) — returns static toml constants
  - gap: static hardcoded JSON (DEEP MRR=0.2060); not read dynamically from toml file; KnowledgeDrawer note at line 262 confirms "static toml constants; live benchmark not wired"

- **DEEP MRR >= STANDARD MRR monotonicity restored**
  - exists: DEEP MRR=0.2060, STANDARD MRR=0.5750 are hardcoded in commands.rs:872-873
  - gap: violation persists in the hardcoded fallback response (0.206 < 0.575); no fix applied

- **NoopSsbNotifier removed from production (cli/main.rs:3721)**
  - exists: NoopSsbNotifier still the default in [reranker.rs:123](.elai_cc/crates/knowledge/src/reranker.rs) `Self::with_ssb(config, clock, Arc::new(NoopSsbNotifier))`; no SsbNotifier reference found in cli/main.rs or cli/live_cli.rs
  - gap: real SsbNotifier was not substituted in cli layer; cli has no SsbNotifier wiring at all for the query path

- **per-stage latency_ms != 0**
  - exists: PipelineStage struct has latency_ms field; [pipeline.rs:32-34](.elai_cc/crates/knowledge/src/pipeline.rs) comment: "latency_ms is always 0 here"
  - gap: 17+ locations in pipeline.rs set latency_ms to 0 literally; Clock threading into stage constructors not done

- **elai_cc.toml [models.cross_encoder] documentation**
  - exists: stage_models scripts enumerate the models
  - gap: no [models] section found in elai_cc.toml; the toml documentation task not completed

- **Canary embed("fn main()") per spec (currently embed("test"))**
  - exists: [pipeline.rs:694](.elai_cc/crates/knowledge/src/pipeline.rs) documents `embed("fn main()")` + BM25("main")
  - gap: canary probe uses BM25("main") but `embed_code` path in readiness_check delegates to `vm.readiness()`, not an explicit `embed_code("fn main()")` call; partial match

## Claims MISSING

- **SSB action type `hyde_query`** — searched: `hyde_query`, `action_type.*hyde`, `ssb.*hyde`; no evidence in any file; SidecarHydeClient emits no SSB row
- **SSB action type `vector_cache_hit_rate`** — searched: `vector_cache_hit_rate`; no evidence anywhere in codebase
- **Tauri command `get_pagerank_top` implemented** — searched: `get_pagerank_top` in dashboard-v2/src-tauri; only referenced in contract test import, no implementation in commands.rs
- **Tauri command `get_freshness_state` implemented** — searched: `get_freshness_state` in dashboard-v2/src-tauri; only referenced in contract test import, no implementation in commands.rs
- **Tauri command `get_canary_drift` implemented** — searched: `get_canary_drift`, `canary_drift`; no evidence in commands.rs or lib.rs
- **HyDE preview visible in KnowledgeDrawer ("HyDE hypothesis preview")** — searched: `hyde.*panel`, `hypothesis.*preview`, `hyde_query`; no UI panel for HyDE hypothesis in KnowledgeDrawer.svelte
- **DKG failed_files count visible in KnowledgeDrawer** — searched: `failed_files`, `failed.*count` in KnowledgeDrawer.svelte and get_dkg_stats command; get_dkg_stats does not return failed_files field
- **Index format version + UpgradeAction status in Settings panel** — searched: `schema_version`, `index_version`, `upgrade_action` in dashboard-v2; no evidence
- **VectorCache hit rate gauge (SSB metric + UI)** — searched: `vector_cache_hit_rate`; no evidence anywhere

## Claims UNVERIFIABLE

- **DEEP MRR >= STANDARD MRR in metrics/phase_3K_rag.toml** — the toml file is referenced by the plan but reading memory-bank/metrics files was not performed; commands.rs hardcodes DEEP=0.206 which violates the spec, so this check would fail on current hardcoded data

## What must be done (action list)

- [ ] Write `query_trace` SSB row in pipeline.rs after each query (replace tracing::debug with ssb.emit); wire SsbNotifier into QueryPipeline or pass notifier to `run_query`
- [ ] Wire real SsbNotifier into cli/live_cli.rs query path (replace NoopSsbNotifier default in CrossEncoderReranker::new)
- [ ] Implement `get_query_trace_recent` to read `query_trace` SSB rows via SsbViewer; set `unwired.recentTraces = false`
- [ ] Wire cascade result into AppState so `get_search_mode` returns the real current mode; set `unwired.searchMode = false`
- [ ] Add `hyde_query` SSB action type to ssb.rs and emit it from SidecarHydeClient (hypothesis text + cache hit/miss + latency)
- [ ] Add `vector_cache_hit_rate` SSB action type and emit from vector indexer per-session
- [ ] Implement `get_pagerank_top` Tauri command in commands.rs + register in lib.rs
- [ ] Implement `get_freshness_state` Tauri command in commands.rs + register in lib.rs
- [ ] Implement `get_canary_drift` Tauri command in commands.rs + register in lib.rs
- [ ] Fix per-stage latency_ms: thread a Clock ref into run_query, capture Instant before/after each stage, populate latency_ms
- [ ] Add HyDE hypothesis preview panel to KnowledgeDrawer (tied to hyde_query SSB row)
- [ ] Add DKG failed_files count to get_dkg_stats command and render in KnowledgeDrawer
- [ ] Fix DEEP MRR >= STANDARD MRR monotonicity (either gate query expansion or ensure cross-encoder runs; then read from toml dynamically)
- [ ] Add [models.cross_encoder] section to elai_cc.toml documenting model paths and blake3 hashes
- [ ] Add index format version + UpgradeAction status Tauri command and Settings panel surface

## Notes

get_embedding_breakers is the most complete of the new commands: it attempts live SsbViewer.embedding_breakers() and falls back to hardcoded defaults. The backend infrastructure (DegradationCascade, VectorError, KnowledgeError, CrossEncoderReranker circuit breaker, 3-ranker RRF, HyDE pipeline stage) is genuinely built and tested. The gap is entirely in the signal propagation layer: SSB row writes are missing for query_trace and hyde_query, and the Tauri command layer does not yet consume live data from AppState for search_mode or query_trace.
