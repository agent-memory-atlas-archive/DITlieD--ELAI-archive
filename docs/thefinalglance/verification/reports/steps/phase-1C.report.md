# Verification Report: phase-1C.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-1C.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Phase 1C (dual-model ONNX vector search) is substantially implemented. All five module files exist under crates/knowledge/src/vector/: mod.rs, cache.rs, ort_session.rs, circuit_breaker.rs, hnsw_index.rs. Dependencies (ort, usearch, blake3, tokenizers) are present and version-pinned in Cargo.toml. The tensor format decisions file exists with tensor_format_verified:true. Two gaps prevent DONE status: (1) vector_cache schema uses blake3/content_hash instead of spec-required blake2s/content_blake2s, and invalidate_by_file returns NotSupported (no file_path column), meaning the SSB file_write handler for per-file cache invalidation is not wired. (2) The measurements file phase-1C-measurements.md is absent — Step 5 was never executed.

## Deliverables verified DONE

- VectorModule trait + DualVector struct (code_vec 768d, text_vec 384d) — evidence: crates/knowledge/src/vector/mod.rs:79-83, :48-55
- DualVector::new validates dimensions (returns Err on mismatch) — evidence: crates/knowledge/src/vector/mod.rs:58-72
- CODE_VEC_DIM=768 / TEXT_VEC_DIM=384 constants — evidence: crates/knowledge/src/vector/mod.rs:48-49
- VectorCache CRUD: insert, lookup by hash, invalidate_keys — evidence: crates/knowledge/src/vector/cache.rs:65-139
- OrtSession struct with Arc<Mutex<Session>>, max_seq_len, output_dim fields — evidence: crates/knowledge/src/vector/ort_session.rs:35-43
- OrtSession::new returns Err(ModelNotFound) on missing model (not panic) — evidence: crates/knowledge/src/vector/ort_session.rs:65-97
- OrtSession tokenizer truncation at max_seq_len — evidence: crates/knowledge/src/vector/ort_session.rs:102-109
- CircuitBreaker struct (EmbeddingCircuitBreaker): states CLOSED/OPEN/HALF_OPEN, threshold 3/60s window, half-open after 30s — evidence: crates/knowledge/src/vector/circuit_breaker.rs:10-14, 36-41
- CircuitBreaker uses ClockProvider from core (not std::time::Instant) — evidence: crates/knowledge/src/vector/circuit_breaker.rs:1 (uses elai_core::ClockProvider)
- VectorIndex trait with add, search, filtered_search, save, load — evidence: crates/knowledge/src/vector/hnsw_index.rs:34-45
- UsearchIndex impl (feature-gated usearch-hnsw): cosine metric, dim config, save/load/filtered_search — evidence: crates/knowledge/src/vector/hnsw_index.rs:51-138
- BruteForceIndex as fallback (no native dep) — evidence: crates/knowledge/src/vector/hnsw_index.rs:146-292
- ort = "=2.0.0-rc.12" version-pinned in Cargo.toml — evidence: crates/knowledge/Cargo.toml:46
- usearch = { version = "2", optional = true } in Cargo.toml — evidence: crates/knowledge/Cargo.toml:53
- blake3 = "1.8" in Cargo.toml — evidence: crates/knowledge/Cargo.toml:39
- tokenizers = "0.22" in Cargo.toml — evidence: crates/knowledge/Cargo.toml:47
- HNSW decision recorded in workflow/reasoning/phase-1C-decisions.md — evidence: workflow/reasoning/phase-1C-decisions.md:5-8
- tensor_format_verified: true in decisions file, shapes for GraphCodeBERT and Granite — evidence: workflow/reasoning/phase-1C-decisions.md:26-57
- OrtVectorModule production struct wrapping two OrtSessions — evidence: crates/knowledge/src/vector/mod.rs:96-132
- RRF k=60 fusion formula implemented in pipeline — evidence: crates/knowledge/src/pipeline.rs:660
- RRF unit test present (test_rrf_fusion_formula) — evidence: crates/knowledge/src/vector/mod.rs:515
- QueryTrace and StageTrace structs defined and emitted in query path — evidence: crates/knowledge/src/query_trace.rs:16,35; crates/knowledge/src/pipeline.rs:628-638
- knowledge_degradation SSB event emitted on SearchMode change — evidence: crates/knowledge/src/degradation.rs:144; crates/knowledge/src/reranker.rs:390
- tokio::Semaphore(1) + max_queue_depth=4 in cross-encoder reranker — evidence: crates/knowledge/src/reranker.rs:142, :82
- startup_readiness_check with 5s canary timeout — evidence: crates/knowledge/src/startup.rs:25,124-176
- Canary warmup via readiness_check before Ready status — evidence: crates/knowledge/src/startup.rs:141-159
- test_dual_vector_dimensions present — evidence: crates/knowledge/src/vector/mod.rs:288
- test_vector_cache_insert_lookup present — evidence: crates/knowledge/src/vector/cache.rs:260
- test_vector_cache_invalidation present — evidence: crates/knowledge/src/vector/cache.rs:278
- test_circuit_breaker_opens_after_3_failures (circuit_breaker_opens_after_3_transient_failures) — evidence: crates/knowledge/src/vector/circuit_breaker.rs:344
- test_graceful_degradation_code_model_fail — evidence: crates/knowledge/src/vector/mod.rs:300
- test_graceful_degradation_both_models_fail — evidence: crates/knowledge/src/vector/mod.rs:318
- test_query_validation_empty, _oversized, _control_chars — evidence: crates/knowledge/src/vector/mod.rs:332-357
- test_startup_canary_success, test_startup_canary_timeout — evidence: crates/knowledge/src/vector/mod.rs:360-372
- test_onnx_canonical_path, test_missing_model_degraded — evidence: crates/knowledge/src/vector/mod.rs:375-411
- test_semaphore_queue_overflow — evidence: crates/knowledge/src/vector/mod.rs:413
- test_cache_hit_skips_embed — evidence: crates/knowledge/src/vector/mod.rs:428
- test_truncation_warning_on_long_input — evidence: crates/knowledge/src/vector/mod.rs:464
- test_concurrent_embed_mutex_session — evidence: crates/knowledge/src/vector/mod.rs:493
- test_hnsw_index_persistence_roundtrip — evidence: crates/knowledge/src/vector/hnsw_index.rs:410
- test_hnsw_filtered_search_project_isolation — evidence: crates/knowledge/src/vector/hnsw_index.rs:447
- Contract tests (knowledge_vector_contract.rs): test_query_returns_dual_vector_results, test_query_project_isolation — evidence: crates/contracts/tests/knowledge_vector_contract.rs:92,116
- TurboQuant NOT implemented in Phase 1C (deferred) — evidence: no TurboQuant code found in vector/; decisions file confirms deferred
- std::time::Instant not used in vector/ (only std::time::Duration for config constants) — evidence: grep confirms Duration only, no Instant
- No .unwrap() in production ort_session.rs code (only in #[cfg(test)] block) — evidence: crates/knowledge/src/vector/ort_session.rs:240-286 (test block only)

## Deliverables PARTIAL

- SSB file_write event handler: DELETE FROM vector_cache WHERE file_path=?
  - exists: crates/knowledge/src/vector/cache.rs:118 — invalidate_file() exists but returns Err(NotSupported)
  - gap: vector_cache schema has no file_path column (column is content_hash only). invalidate_file() raises NotSupported; per-file invalidation requires either a schema extension adding file_path or a caller-managed hash-to-file mapping. The spec requires DELETE FROM vector_cache WHERE file_path=? on SSB file_write event; this is not implemented. No SSB subscriber registering this handler at KnowledgeEngine startup was found.

- test_circuit_breaker_half_open_probe (spec-named test)
  - exists: crates/knowledge/src/vector/circuit_breaker.rs:409 — functionally equivalent test circuit_breaker_half_open_requires_2_consecutive_successes covers HALF_OPEN state after 30s timeout
  - gap: the exact test name from the spec (test_circuit_breaker_half_open_probe) is absent; a behavioral equivalent exists under a different name

- vector_cache schema column naming: spec says content_blake2s BLOB PK
  - exists: crates/knowledge/src/vector/cache.rs:15 — column is named content_hash BLOB PRIMARY KEY
  - gap: spec specifies column name content_blake2s and blake2 crate; implementation uses blake3 hash and column named content_hash. Functional behavior is equivalent but column name and hash algorithm deviate from spec. Decisions file selected blake2s crate but implementation used blake3.

## Deliverables MISSING

- workflow/reasoning/phase-1C-measurements.md — searched: Glob for phase-1C-measurements.md, ls workflow/reasoning; file not found. Step 5 (MEASURE) was never executed. Exit criterion 8 (cache_hit_rate >= 90% in measurements file) is unmet.

## Deliverables UNVERIFIABLE

- None. All deliverables were verifiable via grep/read.

## What must be done (action list)

- [ ] Extend vector_cache schema: add file_path TEXT column to allow DELETE FROM vector_cache WHERE file_path=? (target: crates/knowledge/src/vector/cache.rs schema + migration M3)
- [ ] Implement invalidate_file(&self, file_path: &str) using file_path column instead of returning NotSupported (target: crates/knowledge/src/vector/cache.rs:118)
- [ ] Register SSB file_write event handler at KnowledgeEngine startup that calls cache.invalidate_file() when action_type='file_write' fires (target: crates/knowledge/src/startup.rs or knowledge engine init path)
- [ ] Rename or alias test to test_circuit_breaker_half_open_probe as spec requires, or document deviation (target: crates/knowledge/src/vector/circuit_breaker.rs)
- [ ] Execute Step 5 MEASURE (requires ONNX model files locally): embed 100 chunks, record throughput/latency/cache-hit-rate, write workflow/reasoning/phase-1C-measurements.md with all 4 required entries (throughput_code_model, throughput_text_model, cache_hit_rate, dual_vector_vs_bm25)
- [ ] Confirm cache_hit_rate >= 90% in measurements file (exit criterion 8)
- [ ] Decide whether to rename content_hash column to content_blake2s and switch from blake3 to blake2 (or document intentional deviation from spec)

## Notes

The ort version pinned is "=2.0.0-rc.12" (an rc, not a stable release). The spec said to pin exact version from crates.io at execution time — acceptable per spec, but the rc status should be tracked. The usearch feature gate (usearch-hnsw) means UsearchIndex is not compiled in the default build; BruteForceIndex serves as the default implementation. The spec's primary HNSW deliverable using usearch is feature-gated and not active by default, which may be intentional for CI builds without native usearch support.
