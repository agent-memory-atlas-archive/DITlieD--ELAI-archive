# Verification Report: phase-1I.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-1I.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
Phase 1I delivers 7 resilience deliverables for the knowledge pipeline. All 7 primary
structures exist and are fully implemented. The circuit breaker, degradation cascade,
query validator, startup readiness, cross-encoder semaphore, query trace, and index
version schema are all present with tests. Three gaps remain: the canary reference
vector is stored in a separate JSON file rather than in index_version.toml as spec
requires; UNAVAILABLE blocks only IDLE->PLANNING rather than all FSM transitions per
spec; and the fresh-ONNX-session-on-HALF_OPEN-probe acceptance test is absent.

## Deliverables verified DONE

- EmbeddingCircuitBreaker struct with CLOSED/OPEN/HALF_OPEN states — evidence:
  crates/knowledge/src/vector/circuit_breaker.rs:98 (DEF_STRUCT), :120 (fn new)

- FailureKind enum (Transient/Permanent) for failure classification — evidence:
  crates/knowledge/src/vector/circuit_breaker.rs (FailureKind::Transient, FailureKind::Permanent used in tests)

- Exponential backoff (30s/60s/120s cap) with deterministic jitter per model_id — evidence:
  crates/knowledge/src/vector/circuit_breaker.rs:154 (current_timeout), :469 (test_breaker_exponential_backoff)

- Independent breaker per ONNX model (4 models) — evidence:
  crates/knowledge/src/vector/circuit_breaker.rs:550 (circuit_breaker_independent_per_model test)

- HALF_OPEN requires 2 consecutive successes; 1 success then 1 failure re-opens — evidence:
  crates/knowledge/src/vector/circuit_breaker.rs:417 (test_breaker_half_open_requires_2_consecutive_successes), :454

- ClockProvider trait used for injectable time (entropy quarantine) — evidence:
  crates/core/src/clock.rs:33 (trait ClockProvider); circuit_breaker.rs uses Arc<dyn ClockProvider>

- SearchMode enum: Hybrid, TextOnly, CodeOnly, SparseOnly, Bm25Only (superset of spec's 3) — evidence:
  crates/knowledge/src/degradation.rs:39 (DEF_ENUM SearchMode with 5 variants)

- Degradation cascade (GraphCodeBERT fail->TextOnly, Granite fail->CodeOnly, both->SparseOnly, SPLADE fail->Bm25Only) — evidence:
  crates/knowledge/src/degradation.rs:106-125 (evaluate() logic), tests at :236-328

- SSB logging with action_type='knowledge_degradation' on every mode change — evidence:
  crates/knowledge/src/degradation.rs:144 (DegradationEvent action_type), :253 and :386 (test asserts)

- Mode auto-restore when breaker recovers (HALF_OPEN->CLOSED re-evaluates cascade) — evidence:
  crates/knowledge/src/degradation.rs:351-360 (mode_auto_restores_when_breaker_recovers test)

- QueryValidator struct; rejects empty, >10,000 chars, null bytes, control chars; accepts \n — evidence:
  crates/knowledge/src/query_validator.rs:44 (DEF_STRUCT), :97 (empty test), :134 (null byte), :144 (control char), :193 (valid unicode)

- QueryValidator wired before any embedding call in pipeline — evidence:
  crates/knowledge/src/pipeline.rs:171 (CALL_RESOLVED QueryValidator::validate)

- ReadinessStatus enum: Ready, Degraded, Unavailable — evidence:
  crates/knowledge/src/lib.rs:138 (DEF_ENUM ReadinessStatus with 3 variants)

- startup readiness sequence with canary query, 3-retry logic, 5s timeout — evidence:
  crates/knowledge/src/startup.rs:14-16 (CANARY_TIMEOUT=5s, MAX_RETRIES=3, DRIFT_THRESHOLD=0.85)

- Canary drift check (cosine_similarity vs reference, <0.85->DEGRADED) — evidence:
  crates/knowledge/src/startup.rs:214 (check_canary_drift fn), :230 (cosine_similarity call)

- First-startup reference generation and persistence (canary_reference_vec.json) — evidence:
  crates/knowledge/src/startup.rs:17 (CANARY_REF_FILENAME const), :245 (save_reference_vec call)

- ReadinessStatus reported to orchestrator; UNAVAILABLE blocks IDLE->PLANNING — evidence:
  crates/orchestrator/src/evidence.rs:179 (KnowledgeReadinessGate), :198 (Unavailable -> Err), :573 (gate registered)

- readiness_check fn on QueryPipeline — evidence:
  crates/knowledge/src/pipeline.rs:701 (DEF_FN readiness_check)

- Semaphore(1) around cross-encoder ONNX session access — evidence:
  crates/knowledge/src/reranker.rs:110 (semaphore field Arc<Semaphore>), :142 (Semaphore::new(1))

- Bounded mpsc channel with capacity 4 (max_queue_depth) — evidence:
  crates/knowledge/src/reranker.rs:64 (max_queue_depth field), :82 (default=4), :133 (channel built with queue_depth)

- 6th request overflow returns candidates unranked (not empty) via try_send — evidence:
  crates/knowledge/src/reranker.rs:415-437 (rerank_via_channel, try_send path with return Ok(candidates))

- Batching: drain all pending, merge candidates, one rerank, split results back — evidence:
  crates/knowledge/src/reranker.rs:445-497 (run_batch_worker with offsets tracking per-caller results)

- QueryTrace struct with query_id (UUID), stages, total_latency_ms, mode, result_count — evidence:
  crates/knowledge/src/query_trace.rs:35 (DEF_STRUCT QueryTrace)

- StageTrace struct with stage, latency_ms, input_count, output_count, error, circuit_breaker_state — evidence:
  crates/knowledge/src/query_trace.rs:16 (DEF_STRUCT StageTrace)

- QueryTrace emitted via tracing::debug — evidence:
  crates/knowledge/src/query_trace.rs:72 (tracing::debug! call in emit())

- QueryTrace constructed in pipeline for every query — evidence:
  crates/knowledge/src/pipeline.rs:638 (QueryTrace::new call in query_with_tier)

- SessionTraceStats: p50/p99 per stage, accumulated per session, summarized on flush — evidence:
  crates/knowledge/src/query_trace.rs:92 (SessionTraceStats struct), :145 (summarize fn with percentile_index)

- IndexVersionSchema with version, dkg_schema_version, vector_dimensions (code=768, text=384), bm25_format, created_at — evidence:
  crates/knowledge/src/index_version.rs:34 (IndexVersionSchema struct), :157-167 (current_version fn)

- UpgradeAction enum: Reuse, RebuildBm25Only, FullRebuild — evidence:
  crates/knowledge/src/index_version.rs:55

- archive_index fn creates index_backup_{version}/ before rebuild — evidence:
  crates/knowledge/src/index_version.rs:221-270 (archive_index fn), :410-420 (test asserts archive exists)

- Archive failure returns error; caller must not proceed with rebuild — evidence:
  crates/knowledge/src/index_version.rs:223-225 (doc comment), :429-432 (test archive failure returns Err)

- User notified before rebuild via RebuildNotifier trait (LogNotifier via tracing::warn) — evidence:
  crates/knowledge/src/index_version.rs:127 (RebuildNotifier trait), :134-141 (LogNotifier impl)

- Missing version file creates default (no panic) — evidence:
  crates/knowledge/src/index_version.rs:348 (missing_file_creates_default test)

- Round-trip read/write correctness — evidence:
  crates/knowledge/src/index_version.rs:337 (round_trip test)

## Deliverables PARTIAL

- UNAVAILABLE blocks FSM transitions (spec: "orchestrator blocks all transitions")
  - exists: crates/orchestrator/src/evidence.rs:173-209 — KnowledgeReadinessGate gates IDLE->PLANNING only
  - gap: Spec says UNAVAILABLE should block ALL FSM transitions, not just IDLE->PLANNING.
    Only one gate covering one edge is implemented. No evidence of gates on PLANNING->TESTING_FIRST
    or other transitions that also check KnowledgeReadiness==Unavailable.

- canary_reference_vec stored in index_version.toml (spec: "canary_reference_vec" as field in index_version.toml)
  - exists: crates/knowledge/src/startup.rs:17 — stored as canary_reference_vec.json (separate file)
  - gap: Spec says reference vector is "persisted in index_version.toml as canary_reference_vec".
    Implementation stores it in a separate JSON file (CANARY_REF_FILENAME = "canary_reference_vec.json")
    not as a field inside index_version.toml. IndexVersionSchema struct has no canary_reference_vec field.

## Deliverables MISSING

- Fresh ONNX InferenceSession on HALF_OPEN probe test (acceptance: "mock verifies InferenceSession::new called during HALF_OPEN, not reused")
  - searched: fresh_session, new_session_on_probe, half_open.*new.*session, InferenceSession::new; no test found
  - The EmbeddingCircuitBreaker struct does not manage session creation — correct by design (caller responsibility).
    However the acceptance criterion requires a mock/integration test verifying fresh session construction,
    and no such test exists in the codebase.

## Deliverables UNVERIFIABLE

None.

## What must be done (action list)

- [ ] Decide canonical storage for canary reference vector: either add canary_reference_vec: Vec<f32> to
  IndexVersionSchema in crates/knowledge/src/index_version.rs and move persistence there, OR update the
  spec to reflect the current separate-file approach (canary_reference_vec.json). Both are valid; the
  current impl deviates from spec text.

- [ ] Add FSM gates for UNAVAILABLE on transitions beyond IDLE->PLANNING if spec intent ("all transitions")
  is literal. Alternatively confirm with spec owner that only IDLE->PLANNING is required and update
  crates/knowledge/src/startup.rs or orchestrator spec comment accordingly.

- [ ] Add acceptance test for fresh ONNX session on HALF_OPEN probe: in an integration or mock test,
  verify that after a circuit breaker enters HALF_OPEN, the embedding caller constructs a new Session
  rather than reusing a cached one. Target file: crates/knowledge/src/vector/circuit_breaker.rs or
  crates/knowledge/tests/.

## Notes

SearchMode has 5 variants (Hybrid, TextOnly, CodeOnly, SparseOnly, Bm25Only) versus the spec's
minimum of 3 (HYBRID, SPARSE_ONLY, BM25_ONLY). This is a compliant superset, not a gap.
The cross-encoder semaphore tests (rerank_queue, semaphore) in reranker.rs:802 and :1559 confirm
the queue depth=4 and Semaphore(1) are tested. The KnowledgeReadinessGate in evidence.rs is fully
connected to the orchestrator gate runner (evidence.rs:573).
