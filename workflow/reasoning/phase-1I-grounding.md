Phase 1I Grounding Log
Session: s-978020
Date: 2026-04-06

CLAIMS AND GROUNDING STATUS

1. Circuit breaker: 3 failures in 60s window, HALF_OPEN after 30s timeout
   -> GROUNDED: knowledge.txt:L139-142

2. Independent circuit breaker per ONNX model
   -> GROUNDED: knowledge.txt:L135

3. Circuit breaker state not checkpointed
   -> GROUNDED: knowledge.txt:L151

4. SearchMode cascade: HYBRID -> SPARSE_ONLY -> BM25_ONLY
   -> GROUNDED: knowledge.txt:L117-120

5. Per-model fallback chain (GraphCodeBERT -> Granite -> SPARSE -> BM25)
   -> GROUNDED: knowledge.txt:L144-149

6. Mode change logged to SSB as knowledge_degradation
   -> GROUNDED: knowledge.txt:L124

7. Query validation: max 10,000 chars, reject null/control/empty
   -> GROUNDED: knowledge.txt:L126-130

8. Validation latency <1ms
   -> GROUNDED: knowledge.txt:L130

9. Startup canary: embed("fn main()") + BM25 + rerank within 5s
   -> GROUNDED: knowledge.txt:L159-160

10. ReadinessStatus: READY | DEGRADED | UNAVAILABLE
    -> GROUNDED: architecture-overview.txt:L99

11. 3 retries then BM25_ONLY degraded mode
    -> GROUNDED: knowledge.txt:L164

12. Cross-encoder Semaphore(1), max queue 4, overflow returns unranked
    -> GROUNDED: knowledge.txt:L219-220

13. Batching: merge candidate sets if multiple queued
    -> GROUNDED: knowledge.txt:L221

14. QueryTrace struct with StageTrace
    -> GROUNDED: knowledge.txt:L229-244

15. Traces at DEBUG level, not in SSB, aggregated stats at session end
    -> GROUNDED: knowledge.txt:L246-247

16. index_version.toml schema and migration strategy
    -> GROUNDED: knowledge.txt:L195-211

17. Tree-sitter fallback: unparseable files in BM25
    -> GROUNDED: knowledge.txt:L168-177

ISSUES FOUND

1. DEPENDENCY CONFLICT: implementation-guide.txt dep graph (L140-141) shows 1E -> 1I.
   Phase file originally said "Phase 1G complete". Both are partially correct:
   1I needs 1E (query pipeline) AND 1G (cross-encoder for semaphore work).
   Resolution: prerequisite updated to "Phase 1E AND 1G complete".
   The dep graph in implementation-guide.txt should be updated to reflect 1G -> 1I.

2. MISSING TESTS in original: HALF_OPEN probe behavior, queue overflow,
   index_version.toml round-trip, corrupted version file handling.
   Resolution: added to sub-step ADVERSARIAL and ACCEPTANCE blocks.

3. SPLADE CONDITIONAL: Phase 1H is conditional. Degradation cascade references SPLADE.
   If 1H not implemented, SPARSE_ONLY == BM25_ONLY effectively.
   Resolution: added adversarial test for SPLADE-disabled scenario.

UNGROUNDED CLAIMS:
  - "Index version check latency: <10ms" in MEASURE section -> UNGROUNDED (no spec basis).
    Kept as measurement target, not acceptance criterion. Low risk.

ADVERSARIAL AUDIT (post-improvement, session s-978020)
  4 BLOCKs found, all resolved:
  B1. Dep graph divergence (1E->1I vs 1E+1G->1I) -> strengthened NOTE with ACTION REQUIRED
  B2. UNAVAILABLE ReadinessStatus undefined -> added trigger (BM25 also fails), DEGRADED/BM25_ONLY invariant, acceptance tests
  B3. Queue depth ambiguity (Semaphore(1) vs queue-of-4 confusion) -> clarified: Semaphore(1)+channel(4)=5 in-flight, 6th overflows
  B4. Project isolation invariants missing -> added VERIFY checks for isolation under degradation, cited prior phase ownership

  Key WARNs resolved:
  W1. ClockProvider pre-gate missing -> added to 1I.1 PRE_GATE
  W2. HALF_OPEN concurrent test untestable -> specified tokio::sync::Barrier mechanism
  W3. DEGRADED/BM25_ONLY conflation -> added explicit invariant: DEGRADED implies BM25_ONLY
  W4. Health check polling unspecified -> documented: driven by circuit breaker HALF_OPEN probe, no separate timer
  W5. Batching segregation missing from acceptance -> added segregation test

  WARNs acknowledged but deferred:
  W6. Concept annotation batching (knowledge.txt section 9) has no phase owner -> not in 1I scope, flagged for phase map review
  W7. Tree-sitter fallback grounded but no 1I step -> owned by Phase 1A/1B, 1I verifies invariant holds under degradation in VERIFY


SUBSTANTIVE IMPROVEMENTS (evidence-backed, session s-978020)

  S1. HALF_OPEN success threshold = 2 consecutive (was: 1 probe)
      Evidence: Azure Architecture Center circuit breaker diagram explicitly shows
      "increment success counter" and "success count threshold is reached" before CLOSED.
      Why it helps: prevents premature recovery on flaky ONNX models. Deterministic test:
      1 success stays HALF_OPEN, 2 consecutive -> CLOSED, failure after 1st resets counter.
      Source: https://learn.microsoft.com/en-us/azure/architecture/patterns/circuit-breaker

  S2. Exponential backoff with deterministic jitter for successive OPEN timeouts
      Config: 30s -> 60s -> 120s (cap). Jitter = hash(model_id) mod window, not random.
      Evidence: AWS Builders Library confirms correlated retries without jitter cause
      re-overload. Azure recommends "increasing time-out timer" for successive opens.
      Why it helps: 4 independent circuit breakers recovering simultaneously would
      reload 4 ONNX models at once, causing GPU OOM. Jitter spreads recovery.
      Sources: https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/
               Azure Architecture Center circuit breaker "Recoverability" section

  S3. Failure classification: transient vs permanent
      Transient (timeout, temp OOM): count to threshold (3). Permanent (segfault, missing
      model, invalid ONNX): open immediately after 1 failure.
      Evidence: Azure Architecture Center "Types of exceptions" section recommends adjusting
      strategy based on exception nature. "Retry logic should be sensitive to exceptions
      that circuit breaker returns and stop if fault isn't transient."
      Why it helps: avoids 2 wasted retries on permanent failures (model file deleted).
      Source: https://learn.microsoft.com/en-us/azure/architecture/patterns/circuit-breaker

  S4. Canary embedding drift check via cosine similarity
      On startup, compare embed("fn main()") against stored reference vector. If cosine
      similarity < 0.85, model weights are corrupted/wrong version. Enter DEGRADED.
      Evidence: embedding audit best practices — models that load without error but produce
      garbage embeddings silently corrupt all downstream retrieval with no error signal.
      Why it helps: catches a failure mode the original canary (non-error check only) misses.
      Reference vector persisted in index_version.toml, generated on first successful startup.
      Source: https://dev.to/aairom/decoding-ais-inner-language-how-to-test-your-embedding-models-126

  S5. Fresh ONNX InferenceSession for HALF_OPEN probes
      On HALF_OPEN, create a new InferenceSession rather than reusing the potentially-
      corrupted session that caused the failures.
      Evidence: ONNX Runtime issue #20354 (session hangs during load), #20043 (session
      crashes intermittently). Sessions can enter unrecoverable states.
      Why it helps: without fresh session, HALF_OPEN probe always fails on corrupted session,
      circuit breaker never recovers (permanent OPEN until process restart).
      Sources: https://github.com/microsoft/onnxruntime/issues/20354
               https://github.com/microsoft/onnxruntime/issues/20043
