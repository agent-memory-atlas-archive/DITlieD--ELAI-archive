# I-15: semantic cache + blake3 fast path

- Track 14 — CACHE-INVARIANT ECONOMICS MIDDLEWARE | lever CHEAPER-SPEED | grounding STRONG | nature EXTENSION | effort S | WAVE 1
- Premise: the OracleAnswerCache caches final answers (H-8), but intermediate grounding artefacts — DKG chunk
  summaries, research extracts — are recomputed on every semantically-similar query, paying embedding and
  ONNX inference cost repeatedly.
  An exact blake3 fast path runs before any vector math on repeated queries, and intermediate summaries
  are cached under the same confinement rule inherited verbatim from H-8.

## Steal/build
Extends C-30/H-8 OracleAnswerCache (the same store; tier TTLs + persistence — compose, never restate).
Caches INTERMEDIATE summaries (DKG chunk summaries, research chunk extracts), not just final answers, so
semantically-similar follow-on queries skip re-summarisation.
Exact blake3 fast path: O(1) lookup keyed on blake3(canonical_query_bytes) runs BEFORE any vector cosine
search or ONNX embedding call; cache miss proceeds to the normal vector path.
Confinement: the durable cache stores only IDEMPOTENT grounding content (same rule as H-8); code-gen
outputs, context-dependent tool results, and session-specific reasoning chains must NEVER enter the durable
cache (the H-8 confinement test is inherited verbatim and must still pass).
SSB reuse decision: prefer reuse of the existing OracleCacheHit/Miss family (GD-1); add a
semantic_cache source discriminator field to the existing payload if the field is absent; only if the
existing payload cannot carry it, add +SemanticCacheEvent (decide at edit; the I-0 table specifies this
preference and the decision is recorded in the SSB row at edit time per W-9).

## Existing ground truth
- IS-13: compiled_pipeline.rs CompiledPipelineStore:669 (C-25) — the storage precedent; C-30/H-8 OracleAnswerCache is the direct composition target (PLAN-asserted; executor re-greps per W-7)
- Phase 1E A-RAG (the summarise site where intermediate summaries are generated): the insertion point (PLAN-asserted; executor re-greps per W-7)
- Phase 1G reranker: the vector path this blake3 fast path short-circuits (PLAN-asserted; executor re-greps per W-7)
- H-8 confinement rule: code-gen queries must NEVER hit the durable cache (inherited test must still pass)
- GD-1 (G-0_INDEX.md): compose, never restate — OracleCacheHit/Miss rows exist; reuse before adding a new event type

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- knowledge/src/arag.rs interceptor: blake3 exact-match fast path before the ONNX embedding call + intermediate-summary cache insertion/lookup at the DKG-chunk summarise site
- cache TTL: inherit H-8's tier TTL logic; intermediate summaries use the 5m volatile tier by default (I-10's planner can override per call class)
- SSB reuse: add semantic_cache source discriminator to OracleCacheHit/Miss payload if field absent (decide at edit, record decision); emit hit source = "blake3_exact" or "semantic_vector" in the row
- inherits and re-runs the H-8 confinement test (code-gen query never caches)
- cache-source badge on research answers in the dashboard (frontend_pair)

DEFER (Tier-2 / gated):
- cross-session cache warming (pre-populating the semantic cache from a prior session's DKG chunks before the first query; depends on H-51 cross-session topology being live)
- similarity threshold auto-tuning (requires a measured recall/precision baseline on ELAI's own research corpus per ID-5 before any threshold becomes a gate)

## Integration
Interception point: knowledge/src/arag.rs chunk-summarise path (IS-13 store precedent; PLAN-asserted; executor re-greps per W-7)
Store extension: oracle_cache extension (C-30/H-8 — extend the same store, never replace)
Reranker bypass: Phase 1G reranker call is gated by the blake3 fast-path miss; hit = skip ONNX call entirely
frontend_pair: cache-source badge (blake3_exact / semantic_vector / miss) on research answers in the relevant drawer (PLAN-asserted; executor re-greps per W-7)

## WIRING_CONTRACT
produces_symbols:
  - knowledge/src/arag.rs::blake3_exact_lookup (O(1) fast path before ONNX embedding)
  - knowledge/src/arag.rs::intermediate_summary_cache_insert / _lookup (DKG-chunk summary cache)
  - semantic_cache discriminator field in OracleCacheHit/Miss payload (or +SemanticCacheEvent if payload incompatible — decide at edit per I-0 table preference for reuse)
consumes_symbols:
  - C-30/H-8 OracleAnswerCache (IS-13 store — extend the same store; never replace or restate) — GD-1
  - Phase 1G reranker (the ONNX vector path that the blake3 fast path short-circuits) — GD-1
  - I-10 CacheEconomicsPlanner (the TTL tier for intermediate summaries comes from the planner; I-15 reads the plan, never hard-codes TTL) — GD-1
reachable_from:
  - knowledge::arag summarise call -> blake3_exact_lookup()  [PRODUCTION research pipeline]
  - knowledge::arag DKG-chunk summarise -> intermediate_summary_cache_insert()  [PRODUCTION DKG pipeline]
  - (test callers in contracts/ or #[cfg(test)] do NOT satisfy R7 production reachability)
stub_ban:
  - NO durable caching of context-dependent code-gen outputs (confinement rule inherited from H-8; the confinement test must still pass)
  - NO durable caching of session-specific reasoning chains or tool results
  - NO hardcoded similarity threshold advertised as a target (ID-5: measure ELAI's own recall/precision first; the "50-60% token reduction" number from the research dossier is [DESIGN] until measured)
  - NO re-implementation of the OracleAnswerCache store (extend only)
  - NO ONNX embedding call before the blake3 exact-match check (ordering invariant)
  - NO blake2/sha2 for the fast-path key (blake3 only)
  - NO .unwrap()/.expect() on production cache lookup/insert paths
frontend_pair:
  Cache-source badge (blake3_exact / semantic_vector / miss) on research answers — required; cache efficiency is a user-visible economics signal (W-4/W-6).
observable_effect:
  trigger: "cargo run -p elai-cli -- research --query 'what is the Rust ownership model' --emit-ssb  (run twice with the same query)"
  expect_ssb: "first run: OracleCacheHit or SemanticCacheEvent with source=miss (computes, caches); second run: OracleCacheHit with source=blake3_exact (zero ONNX calls — call-count verifiable from the trace); RAG input-token delta measured between the two runs (ID-5: measure, do not assert 50-60%)"
  expect_artifact: "second run's ONNX call count = 0 (trace-verified); a code-gen query (e.g., 'write a function that does X') NEVER produces a durable cache entry (the inherited H-8 confinement test checks this and must still pass; a broken arm that caches code-gen output FAILS)"

## Grounding
- IS-13 (CompiledPipelineStore:293/669 store precedent), H-8 OracleAnswerCache (PLAN-asserted), Phase 1E A-RAG + 1G reranker (PLAN-asserted)
- GD-1 (compose not restate; OracleCacheHit/Miss rows reused per the I-0 SSB reuse preference)
- ID-5 (I-0_INDEX.md: paper-derived percentages are [DESIGN]; measure ELAI's own before any gate)
- H-8 confinement rule (inherited; code-gen query must not cache — the test is a mandatory inherited gate)
- SSB: prefer reuse of OracleCacheHit/Miss family (GD-1); add semantic_cache discriminator or +SemanticCacheEvent — decide at edit; baseline+0 or baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)

## COMPOSES-WITH
C-30/H-8 OracleAnswerCache (direct extension — the store, the TTL logic, the confinement test; never replaces any of it)
Phase 1G reranker (the vector path whose ONNX call is short-circuited by the blake3 fast path)
Phase 1E A-RAG summarise site (the intermediate-summary insertion point)
I-10 CacheEconomicsPlanner (provides TTL tier for intermediate summaries; I-15 reads, never hard-codes)
IS-13 CompiledPipelineStore (storage format precedent)

End of I-15.
