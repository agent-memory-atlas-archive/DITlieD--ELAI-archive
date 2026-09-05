# Verification Report: phase-1G.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-1G.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Phase 1G implements a cross-encoder reranker (ms-marco-MiniLM-L12-v2) after A-RAG output. The core
implementation is complete: CrossEncoderReranker struct, RerankerConfig, ONNX session loading,
tokenizer, concurrency via Semaphore + mpsc channel, circuit breaker with L-6 fallback, SSB events,
startup canary integration in readiness_check(), and the benchmark fixture. The optimization sweep
test harness (reranker_sweep.rs) exists and is complete. Two deliverables remain ungenerated at
runtime: metrics/phase_1G.toml (requires model files to be staged and the sweep to run) and
scripts/quantize_cross_encoder.py (INT8 quantization script). These are runtime/conditional
artifacts, not compiled code, but the spec mandates they be committed.

## Deliverables verified DONE

- CrossEncoderReranker struct with all required fields (session, tokenizer, fallback_session, semaphore, request_tx, circuit_breaker, fallback_breaker, clock, ssb_notifier) — evidence: .elai_cc/crates/knowledge/src/reranker.rs:105-117

- RerankerConfig struct with all 10 required fields (model_path, tokenizer_path, fallback_model_path, quantized_model_path, skip_threshold, max_sequence_length, max_queue_depth, intra_op_threads, graph_optimization_level, enable_deterministic_compute) — evidence: .elai_cc/crates/knowledge/src/reranker.rs:50-88

- ort = "=2.0.0-rc.12" pinned and tokenizers = "0.22" added to knowledge Cargo.toml — evidence: .elai_cc/crates/knowledge/Cargo.toml:46-47

- GraphOptimizationLevel::All (ORT_ENABLE_ALL) and enable_deterministic_compute=true applied in build_session() — evidence: .elai_cc/crates/knowledge/src/reranker.rs:181-222

- ONNX model loading with quantized_model_path preference, fallback to model_path — evidence: .elai_cc/crates/knowledge/src/reranker.rs:156-177

- Tokenizer loaded from tokenizer.json via tokenizers::Tokenizer::from_file() with truncation at max_sequence_length — evidence: .elai_cc/crates/knowledge/src/reranker.rs:226-251

- Lazy-load fallback L-6 model on first L-12 circuit breaker open — evidence: .elai_cc/crates/knowledge/src/reranker.rs:253-276

- Pair scoring via encode_batch() with input_ids, attention_mask, token_type_ids tensors and single ort session.run() — evidence: .elai_cc/crates/knowledge/src/reranker.rs:529-633

- NaN/Inf scores assigned f32::NEG_INFINITY (sorted to bottom) via apply_scores() — evidence: .elai_cc/crates/knowledge/src/reranker.rs:636-653

- Skip reranking when candidate count <= skip_threshold (configurable, default 10) — evidence: .elai_cc/crates/knowledge/src/reranker.rs:293-295

- tokio::Semaphore(1) guards session access; try_acquire for non-async path — evidence: .elai_cc/crates/knowledge/src/reranker.rs:297-306

- Bounded mpsc channel (capacity 4); try_send overflow returns candidates unranked — evidence: .elai_cc/crates/knowledge/src/reranker.rs:418-443

- run_batch_worker: drains channel via try_recv, merges into single batch, splits by offset — evidence: .elai_cc/crates/knowledge/src/reranker.rs:451-512

- Independent circuit breakers for L-12 and L-6 (both use injectable ClockProvider via EmbeddingCircuitBreaker::with_defaults) — evidence: .elai_cc/crates/knowledge/src/reranker.rs:134-135

- Fallback cascade: L-12 fail -> L-6 (if configured) -> skip reranking (return in A-RAG order) — evidence: .elai_cc/crates/knowledge/src/reranker.rs:308-347

- SSB event emitted on degradation with action_type='knowledge_degradation', stage='rerank'; non-fatal — evidence: .elai_cc/crates/knowledge/src/reranker.rs:387-394

- ClockProvider injected for all time-dependent circuit breaker logic (no direct Instant::now) — evidence: .elai_cc/crates/knowledge/src/reranker.rs:115, 122, 134-135

- Pipeline wired: cross_encoder stage between A-RAG output and token_fit; skipped for FAST tier via use_reranker flag — evidence: .elai_cc/crates/knowledge/src/pipeline.rs:183, 585-612

- SPARSE_ONLY/FAST mode skips reranker entirely (QueryTier::Fast excluded from use_reranker) — evidence: .elai_cc/crates/knowledge/src/pipeline.rs:183, 603

- StageTrace emitted for "cross_encoder" stage in QueryTrace pipeline — evidence: .elai_cc/crates/knowledge/src/pipeline.rs:607, 629-636

- readiness_check() updated to include rerank canary; DEGRADED returned when reranker unloaded or breaker open — evidence: .elai_cc/crates/knowledge/src/pipeline.rs:692-785

- Startup canary tests: readiness_check_healthy_reranker_returns_ready, readiness_check_broken_reranker_returns_degraded, readiness_check_empty_bm25_canary_loaded_reranker_gets_empty_input — evidence: .elai_cc/crates/knowledge/src/pipeline.rs:892-975

- Benchmark fixture committed with 50 candidates, binary relevance, actual Rust code snippets — evidence: .elai_cc/tests/fixtures/reranker_benchmark.json (confirmed code chunks: fn, struct, impl, Arc, RwLock)

- reranker_sweep.rs: full optimization sweep harness (A baseline, B thread sweep, C INT8 conditional, D threshold sweep, E final measurement) with metrics write to phase_1G.toml — evidence: .elai_cc/crates/knowledge/tests/reranker_sweep.rs:1-637

- All 1G.1/1G.2/1G.3/1G.4 unit tests present including: config defaults, boundary threshold tests, empty input, NaN/Inf handling, corrupt/zero-byte model errors, semaphore busy returns unranked, channel overflow, batch cross-contamination, circuit breaker tests, SSB event tests, mock clock half-open probe — evidence: .elai_cc/crates/knowledge/src/reranker.rs:750-2200+

- No .unwrap() in production paths (only in test code and test helpers) — evidence: reranker.rs production code uses ? and .map_err() throughout

## Deliverables PARTIAL

- metrics/phase_1G.toml
  - exists: NOT PRESENT — .elai_cc/metrics/phase_1G.toml does not exist (confirmed via find)
  - gap: The sweep harness generates this file at test runtime (reranker_sweep.rs:591-592 writes it). Requires ONNX model files staged at models/onnx/. The sweep test skips gracefully when models absent. File must be generated and committed per spec DETERMINISTIC_CHECK (grep "baseline_latency_p50" .elai_cc/metrics/phase_1G.toml).

## Deliverables MISSING

- scripts/quantize_cross_encoder.py — searched: find /d/.ELAI_workflow/.elai_cc/scripts -name "quantize*", find .elai_cc -name "quantize_cross_encoder*"; no evidence. Spec 1G.6.C requires this script be committed and runnable (python3 scripts/quantize_cross_encoder.py --input ... --output ...). Referenced in reranker_sweep.rs as the INT8 quantization path but the script itself is absent.

## Deliverables UNVERIFIABLE

- SSB event on circuit breaker CLOSE (recovery) — the emit_recovery_if_needed() method exists but the implementation comment notes it does NOT emit on Closed->Closed to avoid noise (reranker.rs:396-411). Spec says "SSB event emitted on circuit breaker close (recovery)" but the actual behavior is intentionally suppressed. Whether this constitutes a spec deviation or spec clarification is not determinable from code alone.

- Actual latency <50ms for 50 candidates on target hardware — spec marks this CONDITIONAL; requires real model staged and the sweep to run. reranker_sweep.rs has the measurement harness. No measured data exists yet (metrics file absent).

- INT8 NDCG@10 delta <0.5 acceptance gate — conditional on baseline p50 >30ms; not runnable without model files and quantize_cross_encoder.py.

## What must be done (action list)

- [ ] Create scripts/quantize_cross_encoder.py with --input/--output args that run onnxruntime dynamic INT8 quantization on the cross-encoder ONNX model (target: .elai_cc/scripts/quantize_cross_encoder.py)
- [ ] Stage ONNX model files at models/onnx/cross-encoder-l12.onnx and models/onnx/tokenizer.json, then run cargo test --package elai-knowledge optimization_sweep to generate .elai_cc/metrics/phase_1G.toml
- [ ] Commit metrics/phase_1G.toml after sweep run (verify grep "baseline_latency_p50" passes)
- [ ] Verify or add SSB recovery event on circuit breaker close per spec 1G.4 DETERMINISTIC_CHECK ("Unit test: on circuit breaker close (recovery), SSB event emitted") — current emit_recovery_if_needed() is a no-op stub

## Notes
The reranker_sweep.rs test writes the metrics file only when models are staged. The spec's DETERMINISTIC_CHECK for 1G.6 is a mix of code-level checks (fixture file, NDCG computation) that pass unconditionally and model-dependent metrics that require runtime execution. The code architecture is sound and complete; the missing items are runtime-generated artifacts (metrics file) and an unimplemented Python script.
