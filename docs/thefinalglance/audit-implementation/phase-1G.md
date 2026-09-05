PHASE 1G AUDIT — Cross-Encoder Reranker (ms-marco-MiniLM-L12-v2)
================================================================================

Scope: Verify implementation of 1G.1-1G.6 deliverables. Target crate: .elai_cc/crates/knowledge/. Hostile posture — code must exist, wire in, and show measured outcomes. Claims without evidence are DEAD.

Environment state at audit time:
- .elai_cc/models/ contains only Qwen2.5-Coder LLM dirs. NO .elai_cc/models/onnx/ directory.
- cross-encoder-l12.onnx, tokenizer.json, cross-encoder-int8.onnx, and the L-6 ONNX NOT staged.
- .elai_cc/metrics/phase_1G.toml does NOT exist. Every other phase (1A/1B/1D/1E/2*, 3*, 4*) has its metrics TOML. Phase 1G does not. The optimization sweep has never produced output.
- scripts/quantize_cross_encoder.py does NOT exist in .elai_cc/scripts/ (directory contains only start-searxng.sh).

--------------------------------------------------------------------------------
1G.1 — RERANKER MODULE SKELETON + DEPENDENCIES: IMPLEMENTED
--------------------------------------------------------------------------------

Evidence:
- Dependencies pinned in .elai_cc/crates/knowledge/Cargo.toml:27-28:
    ort = "=2.0.0-rc.12"
    tokenizers = { version = "0.22", default-features = false, features = ["progressbar", "onig"] }
- Struct definition: crates/knowledge/src/reranker.rs:105-117. Fields: config, session (Option<Arc<Mutex<Session>>>), tokenizer (Option<Arc<Tokenizer>>), fallback_session, semaphore (Arc<Semaphore>), request_tx (mpsc::Sender), circuit_breaker (Mutex<EmbeddingCircuitBreaker>), fallback_breaker, clock, ssb_notifier.
- RerankerConfig: reranker.rs:50-71 with all spec-required fields (model_path, tokenizer_path, fallback_model_path, quantized_model_path, skip_threshold=10, max_sequence_length=512, max_queue_depth=4, intra_op_threads=1, graph_optimization_level=ORT_ENABLE_ALL, enable_deterministic_compute=true). Default impl reranker.rs:73-88.
- Pipeline wiring: crates/knowledge/src/pipeline.rs:17 imports CrossEncoderReranker; pipeline.rs:54 holds Option<CrossEncoderReranker>; pipeline.rs:79-81 with_cross_encoder() builder.
- SPARSE_ONLY/BM25_ONLY gating: pipeline.rs:152 (use_reranker = QueryTier::Standard|Deep); pipeline.rs:469-487 calls reranker only when use_reranker true, else returns ce_input unchanged. FAST tier skips reranker.

Verdict: IMPLEMENTED. Skeleton matches spec exactly.

--------------------------------------------------------------------------------
1G.2 — ONNX MODEL LOADING, TOKENIZER, GRAPH OPTIMIZATION: IMPLEMENTED (code)
--------------------------------------------------------------------------------

Evidence:
- reranker.rs:156-177 load(): resolves quantized_model_path first (if exists), falls back to model_path; loads tokenizer with truncation.
- build_session (reranker.rs:181-223): Session::builder -> with_optimization_level(graph_optimization_level) -> with_intra_threads(intra_op_threads) -> with_deterministic_compute(enable_deterministic_compute) -> commit_from_file. Wrapped in std::panic::catch_unwind converting panics to RerankerError::ModelLoad. CPU-only (no CUDA/TensorRT EP registered).
- load_tokenizer (reranker.rs:226-251): tokenizers::Tokenizer::from_file + TruncationParams { max_length: max_sequence_length, strategy: LongestFirst }.
- load_fallback (reranker.rs:256-276): lazy L-6 load; errors if fallback_model_path is None; cold-start comment documents 5-15s acceptable.
- Unit tests for invalid/zero-byte/corrupt model returning Err (not panic): reranker.rs:851-924.
- Determinism test reranker.rs:1086-1116 asserts 5 runs produce identical scores with 4 threads + enable_deterministic_compute=true. ORT_ENABLE_ALL vs ORT_DISABLE_ALL score identity: reranker.rs:1119-1148.

Caveat: model-dependent tests (lines 989+, 1086+, 1119+) early-return with "SKIP" when .elai_cc/models/onnx/ absent. Determinism and ORT_ENABLE_ALL identity assertions have never executed in this tree because the fixtures are not staged.

Verdict: IMPLEMENTED (code path). Tests that actually exercise ort are GATED by missing fixtures that have never been checked in. Code compiles and handles all error branches; end-to-end ONNX load has no proof-of-life in the repo.

--------------------------------------------------------------------------------
1G.3 — PAIR SCORING, RERANKING, CONCURRENCY: PARTIAL
--------------------------------------------------------------------------------

Evidence:
- Skip threshold enforcement: reranker.rs:293-295 returns candidates unchanged when len <= skip_threshold. Boundary tests (5, 10, 11) at reranker.rs:804-837, 1265-1322.
- Semaphore(1) guard: reranker.rs:142, 671 (new and new_with_worker both) `Arc::new(Semaphore::new(1))`. Sync rerank uses try_acquire (reranker.rs:300-306); on Err returns candidates unranked. Concurrency test reranker.rs:1438-1495 (5 concurrent calls — only 1 executes; all return input-length).
- Bounded mpsc(4): reranker.rs:133 `mpsc::channel::<RerankerRequest>(queue_depth)` where queue_depth=config.max_queue_depth (default 4). Overflow path: rerank_via_channel at 435-438 uses try_send and returns candidates unchanged on Full.
- Request batching worker: run_batch_worker (reranker.rs:451-503) drains channel via try_recv loop, merges with tracked offsets `(start, len)`, runs single score_pairs_raw, splits back via per-caller oneshot. Cross-contamination test reranker.rs:1708-1792 verifies isolation.
- score_pairs_raw (reranker.rs:521-622): tokenizer.encode_batch with Dual InputSequence (query, snippet), pads to batch-max, builds input_ids/attention_mask/token_type_ids i64 tensors, ort::inputs! with those three named inputs, session.run, extracts f32 output — handles both [batch,1] and [batch] output shapes, rejects mismatched length.
- apply_scores (reranker.rs:626-642): sorts DESC; NaN/Inf -> NEG_INFINITY (sink to bottom).

UNWIRED DEAD CODE: run_batch_worker + rerank_via_channel only called from reranker.rs tests (1589, 1660, 1678, 1690, 1736, 1752, 1764). grep across entire .elai_cc confirms no production caller. Pipeline (pipeline.rs:474) uses the sync `rerank()` method, which does its own try_acquire semaphore but never inserts anything into request_tx. The mpsc channel is therefore a struct field that is written to nowhere and drained by nobody in production — the "bounded mpsc depth 4 + batching" deliverable ships as test-only infrastructure. The sync rerank fallback still handles semaphore overflow correctly, but there is no batching in production.

Verdict: PARTIAL. Logic is there and individually tested. The request-batching channel path is DEAD in the shipping pipeline.

--------------------------------------------------------------------------------
1G.4 — CIRCUIT BREAKER, DEGRADATION, SSB EVENTS, MODEL TIERING: PARTIAL
--------------------------------------------------------------------------------

Evidence:
- Independent breakers: reranker.rs:134-135 EmbeddingCircuitBreaker::with_defaults("cross_encoder", clock) and ..._fallback. Both wrapped in std::sync::Mutex in struct fields (reranker.rs:112-113).
- ClockProvider injected (no std::time::Instant in reranker.rs).
- Fallback cascade: rerank() at reranker.rs:288-348 tries L-12 breaker, on Err emits degradation + tries L-6 if fallback_session.is_some(), emits degradation on L-6 failure, finally returns candidates unchanged (spec L149). Tests: circuit_breaker_opens_after_3 (1841), l12_breaker_open_no_l6_returns_unchanged (1859), l12_breaker_half_open_probe_closes_on_success (1883), l12_open_l6_fallback_used (2017), both_breakers_open_skips_reranking (2079), l6_nonexistent_path_graceful_skip (2134), breaker_instances_are_independent (2163), ssb_event_on_recovery (2184).
- SSB notifier abstraction: reranker.rs:29 re-exports DegradationEvent/NoopSsbNotifier/SsbNotifier from crate::degradation. emit_degradation at reranker.rs:388-394 emits action_type="knowledge_degradation", stage="rerank", detail with "l12_open"/"l6_open" keys. SSB write failure non-fatal by contract (FailingSsbNotifier test at 1822-1987).
- CircuitBreaker exists: crates/knowledge/src/vector/circuit_breaker.rs (EmbeddingCircuitBreaker). Same used for vector module.

PRODUCTION WIRING GAPS:
1. CLI (crates/cli/src/main.rs:3721) constructs reranker with CrossEncoderReranker::new (NoopSsbNotifier). It NEVER calls with_ssb. Therefore every degradation event in production is silently dropped. SSB emission is proven in tests only.
2. CLI reads ce_model_path = models_dir.join("cross-encoder.onnx") and ce_tok_path = models_dir.join("cross-encoder-tokenizer.json") — but the sweep tests and module defaults use models/onnx/cross-encoder-l12.onnx + tokenizer.json. Path names diverge. Whichever is correct, staging one will break the other.
3. fallback_model_path is NEVER set in production (CLI uses `..RerankerConfig::default()` at main.rs:3719 which leaves it None). The entire L-6 intermediate tier is only reachable in tests.
4. emit_recovery_if_needed (reranker.rs:397-411) is a no-op: explicitly does not emit on Closed->Closed "to avoid noise" and has no previous-state tracking. The "SSB event on recovery (Closed)" deliverable is effectively unimplemented. ssb_event_on_recovery test (2184) only asserts breaker closes; never asserts an SSB recovery event was emitted.
5. StageTrace for rerank emitted via QueryTrace at pipeline.rs:459-465 (stage="rerank" RRF fusion) and pipeline.rs:488-494 (stage="cross_encoder"). circuit_breaker_state is populated with `ce.breaker_state()` (pipeline.rs:471-473). Tracing DEBUG line present (pipeline.rs:521).

Verdict: PARTIAL. Unit-test surface is complete. Production: SSB events go nowhere, recovery events never emit, L-6 fallback tier is unreachable. The circuit breaker itself is wired; the degradation observability on top of it is not.

--------------------------------------------------------------------------------
1G.5 — STARTUP CANARY UPDATE: IMPLEMENTED
--------------------------------------------------------------------------------

Evidence:
- crates/knowledge/src/startup.rs:118 doc comment names "canary query (BM25 + rerank) via QueryPipeline::readiness_check".
- readiness_check() in pipeline.rs:583-674: Stage 1 = BM25 query("main", 3), Stage 1.5 = vector readiness, Stage 2 = rerank canary at pipeline.rs:634-665. Checks is_loaded() then runs `ce.rerank("fn main()", candidates)` probe and inspects breaker_state. Status: Ready if all pass; Degraded if reranker fails; Unavailable if BM25 fails.
- Degraded branch reached when is_loaded()=false OR breaker not Closed. CanaryResult (pipeline.rs:683-693) captures rerank_ok/rerank_error/canary_candidates.
- Tests: startup.rs:264-697 uses CrossEncoderReranker::new with default config (not loaded) -> expects Degraded (startup.rs:644-646). Confirms "broken reranker -> DEGRADED (not UNAVAILABLE)" rule.

Caveat: Spec calls for "all within 5s" per L160. Code does not impose a 5s timeout or record per-stage latency in CanaryResult (no latency_ms fields). The 5s budget is unenforced — if the reranker hangs on session.run, readiness_check hangs with it.

Verdict: IMPLEMENTED (logic). The 5s time budget is not enforced.

--------------------------------------------------------------------------------
1G.6 — OPTIMIZATION SWEEP AND PRECISION VALIDATION: DEAD
--------------------------------------------------------------------------------

Evidence — what exists:
- crates/knowledge/tests/reranker_sweep.rs (full 550+ line harness). Steps A-E present:
  A (baseline fp32 ORT_ENABLE_ALL deterministic 1 thread): sweep.rs:294-332. Asserts ndcg_improvement > 0.
  B (thread scaling 1/2/4/8 with determinism check): sweep.rs:333-373. Picks optimal_threads = lowest p50 passing 5-run order identity.
  C (INT8 conditional on baseline p50 > 30ms): sweep.rs:375-432. Loads from models/onnx/cross-encoder-int8.onnx, accepts if NDCG delta < 0.5. Falls through with int8_decision="skipped" if model missing.
  D (threshold sweep 5/10/15/20/30): sweep.rs:434-496. Optimal = lowest threshold where boundary_delta > 0.02.
  E (final measurement): sweep.rs:498-550. Records final_latency_p50/p99, final_ndcg_at_10, model_variant, cold_start_ms. Writes spec_amendment_required=true if final_p50 > 50ms.
- NDCG@10 computation: sweep.rs:94-124 (correct DCG/IDCG formulation, tested for identity/ordering at 236-263).
- Fixture: tests/fixtures/reranker_benchmark.json exists (256 lines, 1 query: "How to implement a thread-safe cache with expiration in Rust?"). Cannot verify >=10 relevant labels without reading further, but file is present.

Evidence — what is missing:
- .elai_cc/models/onnx/ directory does not exist. cross-encoder-l12.onnx, tokenizer.json, cross-encoder-int8.onnx not staged. models_staged() at sweep.rs:57-59 returns false. The optimization_sweep test early-returns at sweep.rs:272 every run.
- .elai_cc/metrics/phase_1G.toml does not exist. Every sibling phase (1A/1B/1D/1E, 2A-D, 3I/3J/3K, 4A/4C/4D/4E) has a metrics TOML; phase 1G does not. The harness has never committed a single measurement.
- scripts/quantize_cross_encoder.py does not exist — the INT8 path has no way to produce inputs even if triggered.
- No evidence of NDCG@10 improvement anywhere (no TOML, no commit, no doc). The assertion `ndcg_improvement > 0.0` at sweep.rs:318-321 has never been evaluated with a real model.

Verdict: DEAD. Harness wired but never runs because fixtures and scripts don't exist.

--------------------------------------------------------------------------------
ANCILLARY CHECKS
--------------------------------------------------------------------------------

Box<dyn Error> count in reranker.rs: 0 (verified via grep). Error handling uses typed enum RerankerError + CircuitBreaker's BreakerError. No training-era anti-patterns.

.unwrap() in production library code (reranker.rs): one at reranker.rs:465 inside run_batch_worker: `semaphore.acquire().await.expect("semaphore closed")`. Per CLAUDE.md hard rule this is prohibited in library code. Additional .unwrap() inside #[cfg(test)] is fine. reranker.rs:547 `encodings.iter().map(|enc| enc.get_ids().len()).max().unwrap_or(0)` is safe (unwrap_or).

tokio::sync::Semaphore used (not std::sync::Semaphore) — correct for async.

MiniLM-L-12-v2 identity never verified in source — the code path is model-agnostic (any BERT-style cross-encoder producing 1 logit per pair would work). ms-marco-MiniLM-L12-v2 appears only in spec prose and the model filename convention; no hash check, no vocab-size assertion, no 384-dim check. Plugging L-6-v2 or distilbert as the L-12 path would compile and run.

--------------------------------------------------------------------------------
SUMMARY
--------------------------------------------------------------------------------

| Sub-step | Verdict     | Core Issue                                                        |
|----------|-------------|-------------------------------------------------------------------|
| 1G.1     | IMPLEMENTED | Skeleton, config, deps, pipeline wiring all present               |
| 1G.2     | IMPLEMENTED | Loader code complete; model-dependent tests skip (no fixture)     |
| 1G.3     | PARTIAL     | Batching channel is DEAD in production (sync path used instead)   |
| 1G.4     | PARTIAL     | SSB=Noop in CLI; L-6 fallback unreachable; recovery emit no-op    |
| 1G.5     | IMPLEMENTED | Canary integrated; 5s budget not enforced                         |
| 1G.6     | DEAD        | Sweep harness exists; fixtures + quantize script missing; no TOML |

Report answers:
(1) Path written: F:\.ELAI_workflow\docs\thefinalglance\audit-implementation\phase-1G.md
(2) Overall verdict: PARTIAL. ~60% of Phase 1G implemented in code; ~40% non-functional in production (batching dead, L-6 dead, SSB noop, quant script missing, metrics never emitted).
(3) WIRED/UNWIRED: WIRED at call-site — pipeline.rs:474 calls ce.rerank(...) on STANDARD/DEEP tiers and cli/main.rs:3715-3730 attaches a reranker when cross-encoder.onnx + cross-encoder-tokenizer.json exist. Observability and batching infrastructure UNWIRED.
(4) NDCG@10 delta measured: UNMEASURED. No phase_1G.toml exists, no model fixtures staged, no sweep has ever executed against real data. ndcg_improvement assertion in the harness has never been evaluated.
