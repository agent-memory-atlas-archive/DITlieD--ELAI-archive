Phase 1E.1 decisions: CatRAG + A-RAG + Pipeline skeleton

What: Three new skeleton modules (catrag.rs, arag.rs, pipeline.rs) added to knowledge crate. Structs, enums, configs with paper-verified defaults, todo!() method bodies.

Key decisions:
- phi() uses const array lookup with debug_assert on score range (auditor WARN fix)
- compute_overlay density guard returns empty overlay for n<2 and low-density (indistinguishable — defer differentiation to 1E.4 implementation)
- ContextTracker uses derive(Default) instead of manual impl (clippy preference)
- QueryPipeline gets explicit new() constructor (auditor WARN fix)
- DkgSnapshot and Bm25Index imported, not redefined — cross-module contracts preserved

Expected test outcome: 65/65 existing tests pass unchanged, 0 new tests (skeleton step).

Unknowns:
- ContextTracker::token_count() will need tokenizer access not yet plumbed
- PipelineStage.latency_ms must use ClockProvider (not Instant::now) per entropy quarantine
