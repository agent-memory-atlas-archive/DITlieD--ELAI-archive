PHASE 1C — VECTOR SEARCH (dual-model ONNX) — HOSTILE AUDIT
================================================================================

Scope: Verify implementation of Phase 1C (Vector Search — GraphCodeBERT 768d + Granite 384d via ONNX Runtime, vector cache, circuit breakers, HYBRID->SPARSE_ONLY->BM25_ONLY cascade, canary readiness, RRF fusion, SSB-driven cache invalidation, SPLADE deferred).

Target: F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\vector\ and integration in crates\cli\src\main.rs, crates\knowledge\src\pipeline.rs, degradation.rs, startup.rs.

--------------------------------------------------------------------------------
SUB-STEP VERDICTS
--------------------------------------------------------------------------------

1C STEP 1 — DEPENDENCY VERIFICATION / CARGO
VERDICT: PARTIAL PASS
Evidence:
- Cargo.toml:27 — `ort = "=2.0.0-rc.12"` (pinned, but RELEASE CANDIDATE, NOT STABLE).
- Cargo.toml:34 — `usearch = { version = "2", default-features = false, optional = true }` gated behind `usearch-hnsw` feature.
- Cargo.toml:28 — `tokenizers = "0.22"` present with `onig`.
- Cargo.toml:21 — `blake3 = "1.8"` present (spec/CLAUDE.md mandate blake3 not blake2s — OK).
- No `fastembed` entry (correct — spec forbids it since it wraps ort).
- Decision doc: workflow/reasoning/phase-1C-decisions.md exists.
Issues:
- ort pin is still RC (2.0.0-rc.12). Spec STEP 1 said "pinned"; technically satisfied, but CLAUDE.md Anti-Stale-Training rule says "ort stable (never -rc)". This is a direct violation of a HARD RULE, even if the phase spec was weaker.
- usearch is *optional / feature-gated* (`usearch-hnsw`). Default build uses BruteForceIndex (pure-Rust). Spec STEP 2 says "HNSW via usearch v2.24.0" as the target; brute-force is a fallback, not the mandated path.
Confidence: HIGH (Cargo.toml read in full).

1C STEP 2 — SKELETON (DualVector, OrtSession, CircuitBreaker, VectorCache, VectorIndex)
VERDICT: PASS with CAVEATS
Evidence:
- vector/mod.rs:48-49 — `CODE_VEC_DIM=768`, `TEXT_VEC_DIM=384`.
- vector/mod.rs:52-72 — `DualVector` with dim-validating `new()`.
- vector/ort_session.rs:35-40 — `OrtSession { session: Arc<Mutex<Session>>, tokenizer, max_seq_len, output_dim }`. Arc<Mutex<Session>> present (spec req). max_seq_len field present (spec req).
- vector/ort_session.rs:55-121 — `OrtSession::new` loads model + tokenizer, configures truncation at max_seq_len and padding Fixed(max_seq_len). Missing model returns `OrtError::ModelNotFound` (no panic). Uses `std::panic::catch_unwind` around Session::builder commit — a defensive wrapper that converts ort panics to ModelNotFound.
- vector/circuit_breaker.rs:98-117 — `EmbeddingCircuitBreaker` with `BreakerState { Closed, Open, HalfOpen }`, threshold 3/60s, half-open base 30s, backoff cap 120s, deterministic jitter from model_id hash, ClockProvider-backed (NO std::time::Instant — compliant with spec STEP 4 check). Independent instance per model_id.
- vector/cache.rs:12-20 — schema `vector_cache (content_hash BLOB PK, code_vec BLOB, text_vec BLOB, cached_at INTEGER)`.
- vector/cache.rs:61 — blake3 hash used.
- vector/hnsw_index.rs:34-45 — `VectorIndex` trait with `add/search/filtered_search/save/load`.
- vector/hnsw_index.rs:51-139 — `UsearchIndex` impl (feature-gated) with MetricKind::Cos.
- vector/hnsw_index.rs:146-291 — `BruteForceIndex` default backend.

Caveats:
- Cache SCHEMA drift vs spec: spec STEP 2 says column is `content_blake2s`. Actual column is `content_hash` (cache.rs:15). Cosmetic rename; blake3 used instead of blake2s, consistent with CLAUDE.md.
- Cache SCHEMA drift vs spec: spec says `DELETE FROM vector_cache WHERE file_path=?`. There is NO `file_path` column. `invalidate_file()` (cache.rs:118-124) returns `CacheError::NotSupported` unconditionally. The spec's file-based invalidation pathway is NOT implemented. `invalidate_keys(&[&[u8]])` exists as a caller-driven alternative, but no SSB handler wires it.
- `VectorIndex` trait requires `Sized` (trait method `load(path, dim) -> Result<Self>`), so it is not object-safe. Callers must be generic. Non-fatal; spec did not require dyn.
Confidence: HIGH (every file in src/vector read in full).

1C STEP 3 — TESTS (TDD failing tests)
VERDICT: PASS
Evidence (vector/mod.rs lines 285-546 contain named Step 3 tests):
- test_dual_vector_dimensions (line 288)
- test_graceful_degradation_code_model_fail (line 300) — drives SearchMode::TextOnly
- test_graceful_degradation_both_models_fail (line 318) — drives SearchMode::SparseOnly
- test_query_validation_empty / _oversized / _control_chars (lines 333-357) via production `QueryValidator`
- test_startup_canary_success / _timeout (lines 360, 367)
- test_onnx_canonical_path (line 375) — asserts "models/onnx/" substring
- test_missing_model_degraded (line 397)
- test_semaphore_queue_overflow (line 414)
- test_cache_hit_skips_embed (line 428)
- test_truncation_warning_on_long_input (line 464)
- test_concurrent_embed_mutex_session (line 493)
- test_rrf_fusion_formula (line 515) — k=60, 3 rankers (BM25+code_vec+text_vec)
Plus cache tests (cache.rs): test_vector_cache_insert_lookup, test_vector_cache_invalidation (via invalidate_keys), vector_cache_roundtrip, vector_cache_invalidate_file_returns_not_supported, etc.
Plus 13 circuit_breaker tests (circuit_breaker.rs:336-653).
Plus hnsw_index tests incl. test_hnsw_index_persistence_roundtrip and test_hnsw_filtered_search_project_isolation (feature-gated).
Contract tests: .elai_cc/crates/contracts/tests/knowledge_vector_contract.rs exists with signature checks.
Count: well above the spec's ">=20 unit tests" minimum (~30+ in vector/ alone).
Gaps:
- test_semaphore_queue_overflow uses a hand-rolled Mutex<u32> shim, not the production tokio::Semaphore(1). Phase 1C Step 4 expected real semaphore; see Step 4 verdict.
- test_startup_canary_success/timeout uses a stub `readiness_check(mock)` helper, not QueryPipeline::readiness_check. That real path IS implemented separately in pipeline.rs (see 1C Step 4 verdict).
- test_truncation_warning_on_long_input asserts truncation fired via a mock closure, not tracing::warn!. No tracing-test capture.
Confidence: HIGH.

1C STEP 3.5 — TENSOR FORMAT VERIFICATION
VERDICT: PASS
Evidence: workflow/reasoning/phase-1C-decisions.md lines 26-61 state:
  tensor_format_verified: true
  GraphCodeBERT: input_ids (batch, seq_len) int64, attention_mask (batch, seq_len) int64
  Granite: same
ort_session.rs:150-154 feeds input_ids + attention_mask + token_type_ids — matching the documented format. tokenizers crate Encoding API (ids as u32 cast to i64) is respected.
Confidence: HIGH.

1C STEP 4 — IMPLEMENTATION (embed, cache, HNSW, RRF, canary, SSB)
VERDICT: PARTIAL PASS (several SPEC DEVIATIONS)
Evidence (positive):
- ort_session.rs:128-212 — `embed()` IS fully implemented: tokenize, build input_ids/attention_mask/token_type_ids tensors, `session.run(inputs)`, extract f32 tensor, apply mean-pooling masked by attention_mask. Handles both 3D [1,S,H] and pre-pooled 2D [1,H] output shapes. Dim mismatch returns `OrtError::InferenceFailed`. No unwrap on the hot path.
- Truncation happens at tokenizer level via TruncationParams(LongestFirst, max_length=max_seq_len). Padding Fixed(max_seq_len) also applied. Spec said "emit tracing::warn! on truncation" — actual code does NOT emit a per-call warn; truncation is silent at the tokenizer layer.
- CircuitBreaker uses ClockProvider/LogicalInstant, no std::time (spec compliant; grep for std::time::Instant in vector/ returns zero matches).
- pipeline.rs:538-572 — `rrf_rerank` with `1.0 / (60.0 + (rank_idx + 1) as f64)` across stages — k=60 per Cormack et al. CORRECT formula.
- pipeline.rs:574-670 — `QueryPipeline::readiness_check()` performs BM25 + vector_module.readiness() + cross-encoder probe. Called from startup.rs with `CANARY_TIMEOUT = Duration::from_secs(5)` (startup.rs:14).
- degradation.rs:39-58 — `SearchMode { Hybrid, CodeOnly, TextOnly, SparseOnly, Bm25Only }` enum present; Display impl returns the required string identifiers.
- degradation.rs:86-144 — cascade logic computes mode from `{code_ok, text_ok, splade_ok}` and emits SSB event `knowledge_degradation` on mode change.
- cli/main.rs:3666-3688 — runtime loads `graphcodebert.onnx` + `granite-embed.onnx` via OrtSession::new and constructs OrtVectorModule; on failure it silently degrades to BM25-only (log only, no crash). Pipeline.vector_module wired at 3677.
- pipeline.rs:278-329 — vector stage in QueryPipeline::query: on `vector_module + vector_indexer` both present, embed query, run indexer.search, push RetrievalResult with `tool_used="vector"`.
- pipeline.rs:615 — vector_module consulted in readiness_check.
- Independent circuit breakers per model_id supported (jitter test asserts distinct jitter for graphcodebert/granite/cross_encoder/splade — circuit_breaker.rs:530-546).

Evidence (negative / spec deviations):
- VECTOR SEARCH USES text_vec ONLY, NOT DUAL. pipeline.rs:281 and :316 both call `vm.embed_text(query)` only. There is no call to `embed_code()` on the query path, and `VectorIndexer` builds a single-dim index (TEXT_VEC_DIM / 384) per cli/main.rs:3696,3703,3707. The code embedding is populated into the cache inside `VectorIndexer::index_file` (indexer.rs:128-135) but never queried at retrieval time. Spec STEP 4 RRF explicitly says "BM25 + code_vector + text_vector" (3 rankers). Implementation fuses BM25 + vector-stage + other stages; the `vector` stage alone carries only text embedding. CODE RANKER MISSING FROM RRF INPUT.
- SSB `file_write` -> vector_cache invalidation is NOT wired. `VectorCache::invalidate_file` returns `CacheError::NotSupported` because schema has no `file_path` column. No startup-registered handler exists. Grep for `file_write` + cache in src/ shows zero matches. Re-indexing happens via `VectorIndexer::remove_file` + re-embed at ingest time (indexer.rs:99, 180), but that is NOT the spec's "immediate cache invalidation on file edit" via SSB.
- No SPLADE code path active in 1C (feature-gated `splade` flag). This is CORRECT per spec (deferred). Verified at Cargo.toml:39 and pipeline.rs:341 (`#[cfg(feature = "splade")]`).
- QueryTrace / StageTrace: query_trace.rs exists; pipeline.rs uses PipelineStage with latency_ms field (spec required). stage latency_ms is populated in most stages but `latency_ms: 0` on lines 326,334 for vector stage — suggests placeholder / not actually timed.
- Cross-encoder tokio::Semaphore(1) + queue depth 4 — Spec Step 4 requirement. Verified NOT in vector/ (as expected, reranker.rs handles that). Out of Phase 1C scope.
- Rerank Semaphore contention test is a hand-rolled Mutex<u32> counter (mod.rs:209-216), not real Semaphore. Light coverage.
- HNSW default index is BruteForceIndex (pure-Rust linear scan), not usearch. Spec's EXIT CRITERIA #9 says "HNSW via usearch v2.24.0; persistence via save/load". Usearch is only built with `--features usearch-hnsw`. Default cargo build violates that criterion.
- Canary `embed("fn main()")` probe: startup.rs:14 declares CANARY_TIMEOUT=5s; QueryPipeline::readiness_check at pipeline.rs:583 does NOT call embed("fn main()"). It calls vm.readiness() which internally calls `embed("test")` in OrtVectorModule (vector/mod.rs:124-125). The literal canary string per spec and knowledge.txt L159 is "fn main()". Semantically equivalent probe, but the spec string is not present. Cross-encoder probe DOES use "fn main()" (pipeline.rs:647).
Confidence: HIGH.

1C STEP 5 — MEASUREMENTS
VERDICT: FAIL
Evidence:
- .elai_cc/metrics/phase_1C* -> does not exist (ls returned "No such file or directory").
- workflow/reasoning/phase-1C-measurements.md -> not found via ls glob.
Spec STEP 5 requires: measurements file with throughput_code_model, throughput_text_model, cache_hit_rate >=90%, dual_vector_vs_bm25. None captured.
Note: spec labels Step 5 "operational, requires real ONNX models, does not block phase exit." That softens the failure, but EXIT CRITERIA #8 still says "Measurements file with cache_hit_rate >=90%". That exit criterion is UNMET.
Confidence: HIGH (negative evidence from filesystem).

--------------------------------------------------------------------------------
KNOWN DEBT CHECK — Box<dyn Error> count
--------------------------------------------------------------------------------
Grep of `F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\vector\indexer.rs`:
  line 57:  pub fn load(path: &Path, dim: usize) -> Result<Self, Box<dyn std::error::Error>>
  line 71:  pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>>
  line 89:  ) -> Result<IndexStats, Box<dyn std::error::Error>>        (index_file)
  line 194: pub fn search(...) -> Result<Vec<(ChunkMeta, f32)>, Box<dyn std::error::Error>>
  line 218: fn chunk_file(path: &Path) -> Result<Vec<BM25Chunk>, Box<dyn std::error::Error>>
COUNT: 5 (exactly matches Phase 1A audit claim).
Four are on public API (load, save, index_file, search); fifth (chunk_file) is crate-private helper but still violates anti-erased-error rule.
Spec has no explicit Phase 1C requirement to fix these; debt persists unfixed.

--------------------------------------------------------------------------------
MODEL STAGING / PATHS
--------------------------------------------------------------------------------
Audit input claimed ".elai_cc/models/onnx/". Evidence:
- F:\.ELAI_workflow\.elai_cc\models\ contains only Qwen2.5-Coder dirs (generative LLM weights, unrelated).
- No `models/onnx/`, no `graphcodebert.onnx`, no `granite-embed.onnx` anywhere under .elai_cc (find .onnx returned empty).
- Runtime path used by cli/main.rs:3666-3670 is `{project_root}/.elai/models/` — NOT `.elai_cc/models/onnx/` (spec STEP 2: "Loads from canonical .elai_cc/models/onnx/"). Spec STEP 5 PRE_GATE also said `.elai_cc/models/graphcodebert/` and `.elai_cc/models/granite-embedding-small/`. Three different paths across spec and code. Inconsistent.
- Consequence: every user's first boot degrades to BM25-only because ONNX files are missing. No hash verification, no index_version.toml model check. `index_version.toml` exists (index_version.rs:282) but tracks BM25/vector_dim only, not model-file hashes.
Confidence: HIGH.

--------------------------------------------------------------------------------
IS VECTOR SEARCH WIRED INTO QUERY PIPELINE?
--------------------------------------------------------------------------------
YES — wired, but DEGRADED BY DEFAULT:
- QueryPipeline struct holds `vector_module: Option<Box<dyn VectorModule+Send+Sync>>` and `vector_indexer: Option<VectorIndexer>` (pipeline.rs:55-56).
- CLI startup path (crates/cli/src/main.rs:3666-3710) instantiates OrtSession for both models, wraps in OrtVectorModule, attaches to pipeline, and loads VectorIndexer (with persistence).
- pipeline.rs:278-329 runs the vector stage per-query when both module and non-empty indexer are present.
- On missing ONNX files (the default state of the repo), the code silently logs "vector code/text model not loaded" and falls back to BM25-only. No canary drift error, no fatal exit. This is spec-compliant "graceful degradation" behavior, but practically means vector search is OFF on every clean checkout until models are staged.
- ONLY text_vec is used at query time. Code-vec ranker is absent from RRF (see Step 4 deviations).

--------------------------------------------------------------------------------
ort VERSION
--------------------------------------------------------------------------------
Cargo.toml line 27: `ort = "=2.0.0-rc.12"`.
Verdict: pinned to a release candidate, not a stable release. Violates CLAUDE.md Anti-Stale-Training rule ("ort stable (never -rc)"). Phase 1C spec requirement "ort version-pinned" is literally satisfied; architectural rule is violated.

--------------------------------------------------------------------------------
SUMMARY
--------------------------------------------------------------------------------
Overall verdict: PARTIAL PASS with SIGNIFICANT GAPS.

What works:
- Dual-model ONNX skeleton present, compiles, and tokenizer+tensor+mean-pool path is real (not stubbed).
- OrtSession wraps Arc<Mutex<Session>>, max_seq_len field, missing-model = ModelNotFound (not panic).
- EmbeddingCircuitBreaker fully implemented with FailureKind split (Transient/Permanent), exponential backoff, deterministic jitter, HALF_OPEN 2-success threshold, independent per model_id, ClockProvider-backed.
- SearchMode enum + cascade + SSB `knowledge_degradation` events present (degradation.rs).
- RRF fusion formula k=60 correct in pipeline.rs:538.
- Canary path exists with 5s budget (startup.rs:14, pipeline.rs:583).
- VectorCache schema + roundtrip + blake3 hashing + invalidate_keys present.
- Tests >> 20 in vector/ plus contract test in crates/contracts/.
- Tensor format verified (phase-1C-decisions.md).
- SPLADE correctly deferred (feature-gated).
- TurboQuant correctly NOT implemented (spec says defer; raw f32 stored — cache.rs uses `f32.to_le_bytes`).

What does NOT work or deviates from spec:
1. ort still on `=2.0.0-rc.12` (RC, not stable) — violates CLAUDE.md hard rule.
2. Vector query path uses ONLY embed_text — the spec's dual-vector + 3-ranker RRF (BM25+code_vec+text_vec) is NOT realized; code embeddings are populated into the cache but never queried.
3. Canary does not literally embed("fn main()") — uses embed("test") in OrtVectorModule.readiness(); cross-encoder probe uses "fn main()" but that's a different model.
4. SSB file_write -> vector_cache invalidation NOT wired. `invalidate_file()` unconditionally returns NotSupported; schema lacks file_path column. Violates "Immediate cache invalidation on file edit".
5. Default HNSW backend is BruteForceIndex; usearch is feature-gated — fails EXIT CRITERIA #9.
6. Model staging: no ONNX files present in repo; paths in code (`.elai/models/`) disagree with spec (`.elai_cc/models/onnx/`). No model-file hash verification.
7. Phase 1C measurements file absent (.elai_cc/metrics/phase_1C*.* + workflow/reasoning/phase-1C-measurements.md). Fails EXIT CRITERIA #8.
8. Box<dyn Error> count in vector/indexer.rs public surface: 5 (debt from Phase 1A NOT paid down).
9. test_semaphore_queue_overflow / test_startup_canary are stub-shaped (hand-rolled mutex, stub helper), not driving the production tokio::Semaphore or pipeline.readiness_check. Tests pass but coverage of production concurrency path is thin.
10. vector-stage latency_ms is hardcoded 0 in pipeline.rs:326,334 — spec required per-stage latency. Other stages have real timing.

Confidence (overall): HIGH. All conclusions grounded in direct reads of source files and Cargo.toml.

Ungrounded conjecture: none — every claim above cites a concrete file:line.
