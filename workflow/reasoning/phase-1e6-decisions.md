Phase 1E.6 Decision Doc

What: Implement QueryPipeline::query() returning 7 PipelineStages, plus A-RAG stubs (1E.5 prerequisite).

Key decisions:
1. Empty DkgSnapshot (nodes:vec![], edges:vec![]) for PageRank/CatRAG stages — no DkgEngine field on QueryPipeline. compute_overlay density guard returns empty overlay when n<2.
2. Latency_ms set to 0 in pipeline.rs; callers measure wall time externally (entropy quarantine: raw Instant confined to core/src/clock.rs).
3. AragOrchestrator has no bm25 field — KeywordSearch/SemanticSearch gracefully degrade to empty Vec. ChunkRead uses tracker for dedup.
4. RRF fusion for rerank: score = 1/(60+rank), rank starts at 1. Merge bm25+vector results by chunk_id, sum scores.
5. token_fit stage: passthrough (no actual budget enforcement at pipeline level — budget is A-RAG internal).
6. No new struct fields added to AragOrchestrator or QueryPipeline.

Expected test outcome: 7 stages returned, names match ["bm25","vector","pagerank","catrag","arag","rerank","token_fit"], latency_ms = 0 (caller-measured externally).

Unknowns: none after source reads.
