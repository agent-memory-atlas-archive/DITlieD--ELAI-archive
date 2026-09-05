Phase 1I Audit — Resilience Layer (circuit breaker, degradation, startup, tracing)

Scope: audit 1I.1-1I.7 against the spec in F:\.ELAI_workflow\docs\thefinalglance\summary\phase-1-summary.md (## PHASE 1I, lines 1144-1260).

Source files inspected:
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\vector\circuit_breaker.rs (654 LOC, incl. tests)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\degradation.rs (524 LOC)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\query_validator.rs (256 LOC)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\startup.rs (751 LOC)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\query_trace.rs (435 LOC)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\index_version.rs (~540 LOC)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\reranker.rs (semaphore wiring at lines 110/142/297/454/465/671/697)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\pipeline.rs (lines 16/140/510-527 QueryTrace emit)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\lib.rs (ReadinessStatus 3 variants at 120-125)
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\main.rs (lines 77-78, 3800-3819 startup wiring)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\Cargo.toml (tracing, uuid, tokio sync all present)

------------------------------------------------------------------------
1I.1 CIRCUIT BREAKER — VERDICT: IMPLEMENTED (complete + spec-faithful)
------------------------------------------------------------------------
Evidence:
- vector/circuit_breaker.rs:36-41 — BreakerState enum (Closed/Open/HalfOpen).
- :25-30 — FailureKind enum (Transient/Permanent) — classification via enum, NOT string match, per spec L1161.
- :47-66 — CircuitBreakerConfig defaults: failure_threshold=3, window=60s, half_open_base=30s, backoff_cap=120s, half_open_success_threshold=2. Matches spec exactly (L1156, L1167).
- :155-162 current_timeout() — 30s->60s->120s exponential backoff (1u64 << consecutive_opens.min(2)), capped at 120s. Matches L1163.
- :307-315 compute_jitter — DefaultHasher(model_id) % base_ms. Deterministic per model, satisfies L1164 "hash of model_id, NOT random".
- :98-117 EmbeddingCircuitBreaker uses `Arc<dyn ClockProvider>` — no std::time::Instant/SystemTime::now (grep for `std::time::Instant|SystemTime::now|Instant::now` in circuit_breaker.rs: NO MATCHES). Entropy-quarantined per spec L1157.
- :231-261 record_failure — Permanent => trip_open immediately after 1 failure; Transient => count within window; >= threshold => trip_open. Matches L1160-1162.
- :208-228 record_success — HALF_OPEN requires 2 consecutive successes (half_open_success_threshold=2) before returning to CLOSED. Matches L1167 Azure success-counter pattern.
- :284-297 trip_open — on re-open from HALF_OPEN, consecutive_opens.saturating_add(1) escalates backoff. On close (record_success) at :219, consecutive_opens reset to 0. Matches L1163-1164.
- State NOT checkpointed: no file/DB persistence code in the module — fresh CLOSED on process restart. Matches L1165.
- 20 unit tests in the module cover CLOSED-OPEN-HALFOPEN transitions, exponential backoff 30/60/120/cap, deterministic jitter across 4 models, independent-per-model, permanent-opens-immediately, half_open requires 2 successes, window-reset, recovery-resets-backoff.

Partial gap: "Fresh ONNX InferenceSession for HALF_OPEN probes" (spec L1168) is NOT implemented inside EmbeddingCircuitBreaker. The breaker is model-agnostic — it takes FnOnce closures. The burden of producing a fresh Session for a HALF_OPEN probe falls on the call site (reranker.rs, vector/ort_session.rs). No grep hit for `fresh.*session` or "new InferenceSession" guarded by HalfOpen. This is an ARCHITECTURAL gap: the breaker supports fresh-session-on-probe but nothing wires it in. Caller is responsible, but no evidence any caller does so.

Verdict: CORE IMPLEMENTED. Fresh-session-on-probe guidance from ONNX issues #20043/#20354 (spec L1168, L1171) is NOT enforced anywhere in the tree.

------------------------------------------------------------------------
1I.2 SEARCHMODE CASCADE & DEGRADATION — VERDICT: PARTIAL (data-plane not wired)
------------------------------------------------------------------------
Evidence of struct/logic:
- degradation.rs:39-50 — SearchMode enum. Spec (L1175) asks for 3 variants {HYBRID, SPARSE_ONLY, BM25_ONLY}. Implementation has 5: Hybrid, CodeOnly, TextOnly, SparseOnly, Bm25Only. This is a SUPERSET (spec L1177-1179 actually describes the finer-grained fallback chain; the 5-variant enum more faithfully matches L1177-1180). Acceptable enrichment.
- :79-163 DegradationCascade with evaluate() reading breaker states and emitting `knowledge_degradation` SSB events on mode transitions. Matches L1182-1184.
- :171-182 breaker_is_available — HALF_OPEN counts as available; poisoned Mutex treated unavailable. Reasonable.
- :10-27 DegradationEvent struct + SsbNotifier trait + NoopSsbNotifier. Wire contract complete.
- degradation.rs tests 228-523 cover: all-healthy=Hybrid, code-fail->TextOnly, text-fail->CodeOnly, both-dense-fail->Bm25Only or SparseOnly-if-SPLADE, both-dense+splade-fail->Bm25Only, auto-restore via sync_state, SSB event-per-transition, flapping model, per-model independence, HalfOpen=available.

Evidence of wiring gaps (THIS IS THE AUDIT-CRITICAL FINDING):
- pipeline.rs:133 docstring acknowledges "Distinct from Phase 1I `SearchMode` cascade". Only one reference to "cascade" in the entire pipeline.rs.
- pipeline.rs:510-527 BUILDS a QueryTrace at the end of every query_with_tier() — but passes `SearchMode::Hybrid` HARDCODED (line 520: `let trace = QueryTrace::new(stage_traces, 0, SearchMode::Hybrid, result_count);`). The cascade is NOT consulted per query. The QueryTrace will always say `mode=HYBRID` even if the pipeline actually ran degraded.
- Grep for `cascade.evaluate|cascade.current_mode` in crates/: only hits are tests in degradation.rs and startup.rs + ONE production use in cli/src/main.rs:3800-3808 during startup_readiness_check. There is NO runtime code that re-evaluates the cascade on component failure during a live query.
- Per-query cascade trigger: DOES NOT EXIST. If GraphCodeBERT ONNX segfaults mid-query, the breaker opens (local to vector module), but no code path exists that propagates that open-state into a SearchMode switch. The cascade lives in startup only.
- SSB emission path for `knowledge_degradation`: only two writers — (a) degradation.rs:143-147 (only invoked inside DegradationCascade::evaluate, only called from startup), and (b) reranker.rs:390 (invoked on cross-encoder breaker open, which only affects the reranker-internal skip logic). The production CLI path wires startup with its cascade, but does NOT hold onto the cascade for use during subsequent queries.

ACCEPTANCE criteria from spec (L1185):
- "kill embedding -> BM25 within 100ms of failure detection" — UNPROVEN. No test injects a live failure and asserts the next query runs in BM25_ONLY. The existing tests trip the breaker manually then re-evaluate; there is no production control-flow that does the re-evaluation during query.
- "auto-restores when health passes" — LOGIC EXISTS in cascade (half-open=available), but since cascade is never re-evaluated at query time, auto-restoration is effectively dead code outside of the startup call.
- "SSB has event for every change" — only during startup. Mid-session failures will NOT fire `knowledge_degradation` events through DegradationCascade. Cross-encoder failures DO fire via reranker.rs:390, but the main dense-vector cascade does not.
- "never total outage (BM25_ONLY always reachable)" — structurally OK: BM25 (tantivy) has no ONNX dep and is invoked in pipeline.rs:243-266 unconditionally.

Verdict: CASCADE LOGIC IMPLEMENTED; NOT WIRED TO QUERY PATH. This is the same "SSB degradation events noop in 1G CLI path" issue flagged by prior audits, now reconfirmed at the cascade level.

------------------------------------------------------------------------
1I.3 QUERY VALIDATOR — VERDICT: IMPLEMENTED (production-wired, well-tested)
------------------------------------------------------------------------
Evidence:
- query_validator.rs:10 MAX_QUERY_LEN = 10_000 (spec L1195 requires constant "10_000 or MAX_QUERY").
- :58-83 QueryValidator::validate — rejects (a) empty/whitespace-only via split_whitespace().next().is_none(), (b) >10_000 chars (char count, not bytes, at :60-63), (c) null byte / C0 control chars (byte < 0x20 excluding \t\n\r).
- :44 struct is a zero-sized unit type — no model references possible; validation structurally cannot invoke ONNX.
- 15 tests cover: empty, whitespace-only, 10_001 rejected, exactly-10_000 accepted, null byte, BEL, all C0 control chars, valid, newline, tab+CR, UTF-8 char-count not byte-count, length-check-before-empty, DEL 0x7F accepted, unicode accepted.
- Production wiring: pipeline.rs:140 — `QueryValidator::validate(query)?;` is the FIRST statement of query_with_tier(). Spec L1190 "runs BEFORE any embedding call" SATISFIED.
- <1ms constraint (L1194, L1196): structural (unit struct, single-pass byte scan, no allocation). Not benchmarked but defensible.

ACCEPTANCE (L1196): empty, 10_001, null byte, control char all rejected; valid + newline-embedded accepted. All covered.

Verdict: FULLY IMPLEMENTED AND WIRED.

------------------------------------------------------------------------
1I.4 STARTUP READINESS — VERDICT: IMPLEMENTED (with one BM25-failure test gap)
------------------------------------------------------------------------
Evidence:
- lib.rs:120-125 ReadinessStatus {Ready, Degraded, Unavailable} — exactly 3 variants (L1208).
- startup.rs:14-17 constants CANARY_TIMEOUT=5s, MAX_RETRIES=3, DRIFT_THRESHOLD=0.85, matching spec L1206.
- :126-207 startup_readiness_check — retries up to MAX_RETRIES, enforces canary_timeout via ClockProvider elapsed_since (deterministic), preserves best_status, Unavailable short-circuits retries (BM25 fail = no point retrying), Degraded retries, Ready breaks.
- :214-251 check_canary_drift — cosine_similarity vs reference vec loaded from project_data_dir. Drift < 0.85 => Degraded (downgrade). First startup with no reference => persists reference, returns ReferenceGenerated (Ready preserved). Spec L1206 bullet 5 satisfied.
- :66-85 cosine_similarity — f64-accumulated dot+norms, returns 0 for length-mismatch/empty/zero-norm. Correct.
- :95-109 load_reference_vec / save_reference_vec — JSON file at project_data_dir/canary_reference_vec.json. Spec says TOML `canary_reference_vec` field, implementation uses separate JSON file — DEVIATION from spec L1206 "persisted in index_version.toml as canary_reference_vec". Functionally equivalent but structurally divergent. Minor issue.
- :188-199 DEGRADED => cascade.set_mode(SearchMode::Bm25Only). Invariant "DEGRADED implies BM25_ONLY" enforced (L1212).
- Tests cover: healthy=>Ready, unloaded reranker=>Degraded with Bm25Only, drift-fail=>Degraded, first-startup persists reference, canary-timeout=>Degraded via TimeoutClock that jumps 6s per now(), retries-all-3-attempts, cosine identical/orthogonal/opposite/empty.
- CLI wiring: cli/src/main.rs:3800-3819 — cascade created with splade gate, startup_readiness_check called, Degraded/Unavailable emit eprintln warnings.

GAPS:
- bm25_fail_returns_unavailable test (startup.rs:397-430) documents that actually producing Unavailable requires "corrupt tantivy index" but the test only verifies structural assertion (assert_ne!(result.status, Unavailable)). No end-to-end test proves that a broken BM25 returns Unavailable — this is an UNTESTED path.
- FSM gating (L1215): knowledge crate returns Unavailable, but there is NO orchestrator code reading it to block IDLE->PLANNING. The contract exists in the type system but the gate is not implemented.
- Reference-vector location: spec says inside index_version.toml; impl uses separate JSON. Deviation.

Verdict: IMPLEMENTED, with FSM-gate integration and true-BM25-failure path UNPROVEN.

------------------------------------------------------------------------
1I.5 CROSS-ENCODER SEMAPHORE — VERDICT: IMPLEMENTED (with known CLAUDE.md violation)
------------------------------------------------------------------------
Evidence:
- reranker.rs:7 `use tokio::sync::{mpsc, oneshot, Semaphore};`
- :110 `semaphore: Arc<Semaphore>` field on CrossEncoderReranker.
- :142 / :671 Semaphore::new(1) — spec L1222 "Semaphore(1)" satisfied.
- :297-300 synchronous `try_acquire` path for non-async callers — busy semaphore returns Ok(unranked) (spec L1222 overflow path).
- :451-503 run_batch_worker — drains mpsc, merges candidate sets, runs one Self::score_pairs_raw inference, splits results per caller via oneshot reply channels. Implements L1222 "Batching: merge candidate sets, rerank once".
- Config `max_queue_depth` used for `mpsc::channel(queue_depth)` at :662 — spec L1222 says capacity 4; the default is spec-configurable.
- Tests (lines 1476-1531 and beyond): overflow returns unranked candidates not empty, 2nd request serializes behind 1st, batching merges 2 requests 10-each into 1 inference call of 20 pairs.

BLOCKING FINDING (prior audits already flagged):
- reranker.rs:465 `let _permit = semaphore.acquire().await.expect("semaphore closed");` inside run_batch_worker. This is a production-hot-path `.expect()` — PANICKING on semaphore closure. CLAUDE.md HARD RULE: "No .unwrap() in production/library Rust code ... .expect('reason') same restriction". Semaphore::acquire().await returns Err only if the Semaphore was closed via Semaphore::close(); we never call close, but the panic path still exists. This is the same violation flagged in the 1G audit.

Verdict: SEMAPHORE BEHAVIOR CORRECT; `.expect("semaphore closed")` at line 465 VIOLATES CLAUDE.md and MUST be replaced with `?` via a worker-level error or graceful task exit.

------------------------------------------------------------------------
1I.6 QUERY TRACE — VERDICT: PARTIAL (structs right, mode hardcoded in wiring)
------------------------------------------------------------------------
Evidence:
- query_trace.rs:14-21 StageTrace { stage, latency_ms, input_count, output_count, error, circuit_breaker_state } — matches spec L1231.
- :33-39 QueryTrace { query_id (Uuid), stages, total_latency_ms, mode (String), result_count } — matches L1230. Uuid via `uuid::Uuid::new_v4()` at :53.
- :62-73 emit() — serde_json serialize + tracing::debug!("query_trace", ...). DEBUG level satisfies L1233.
- :84-175 SessionTraceStats — per-stage sorted latencies, p50/p99 via nearest-rank (:185-191), thread-safe via Mutex. Matches L1234 "Aggregated per-session statistics (p50/p99 per stage)".
- :177-181 Default impl; SessionSummary serializable for SSB flush.
- 9 tests: three_queries_emit_three_traces (subscriber-captured count==3), hybrid-trace-has-all-stages, bm25_only-reflects-mode, circuit_breaker_state in StageTrace, p50/p99 with known samples, single-sample, empty-session, multi-stage.
- Production wiring: pipeline.rs:15 imports, :511-518 builds StageTrace per pipeline stage (preserves error + circuit_breaker_state from each PipelineStage), :520 constructs QueryTrace, :521-527 emits tracing::debug "query_trace".

CRITICAL DEFECT:
- pipeline.rs:520 hardcodes `SearchMode::Hybrid`. Spec L1237 ACCEPTANCE: "BM25_ONLY trace has mode==BM25_ONLY". This is false in production — every real query reports HYBRID regardless of actual cascade state. The only reason the existing test `bm25_only_trace_reflects_mode` passes is that the test manually constructs the QueryTrace with Bm25Only; no end-to-end test runs the pipeline under degraded conditions.
- Also at :520, `total_latency_ms: 0` — the sum/wall-clock is NOT computed. QueryTrace::new accepts it, but pipeline hardcodes zero. Observability is compromised.
- SessionTraceStats has no production call site — grep returns no references in pipeline.rs or cli/main.rs. The aggregation story (flush to SSB at session end, L1234) is NOT wired anywhere.

Verdict: STRUCTS + PERCENTILES CORRECT; PIPELINE WIRING INCOMPLETE (mode hardcoded, latency=0, session stats orphaned).

------------------------------------------------------------------------
1I.7 INDEX VERSION SCHEMA — VERDICT: IMPLEMENTED (well-tested, one wiring check)
------------------------------------------------------------------------
Evidence:
- index_version.rs:14-19 CURRENT_SCHEMA_VERSION=1, CURRENT_INDEX_VERSION=2 (bumped to 2 in 3K.1 for AST-aware chunking).
- :32-45 IndexVersionSchema{version, dkg_schema_version, vector_dimensions{code,text}, bm25_format, created_at} + IndexVersionFile wrapper. TOML structure matches spec L1243.
- :54-61 UpgradeAction {Reuse, RebuildBm25Only, FullRebuild}. Spec L1244 "full rebuild OR BM25-only rebuild OR reuse" satisfied.
- :195-217 compare() — schema_version or vector_dimensions change => FullRebuild; bm25_format change => RebuildBm25Only; version bump => FullRebuild (conservative catch-all); else Reuse.
- :225-270 archive_index — enumerates entries, skips `index_backup_*` and `index_version.toml`, creates `index_backup_{old_version}/`, std::fs::rename for each entry. ArchiveFailed error captures source + context. Spec L1245 "Archive old index before rebuild" satisfied.
- :277-311 check_and_upgrade — reads existing version, compares, early-returns Reuse, calls notifier.notify BEFORE destructive op (L1246), archives, writes new. Order correct.
- :126-142 RebuildNotifier trait + LogNotifier (tracing::warn). User notification pluggable.
- Error handling: explicit enum IndexVersionError with std::error::Error + source() chain. Zero unwrap/expect in non-test code. From impls for std::io::Error, toml::de::Error, toml::ser::Error.
- 14 tests cover: round-trip, missing=>default created, corrupt TOML=>Err, schema-mismatch=>FullRebuild, vector-dim-change=>FullRebuild, bm25-format-change=>RebuildBm25Only, no-change=>Reuse, archive-created-before-rebuild, archive-failure-returns-error, toml-not-archived, previous-archives-not-re-archived, full-upgrade-flow, corrupted-file-in-upgrade-flow, version-bump.
- Caller: orchestrator/src/indexing_service.rs:163 passes `index_version_path.as_path()` into whatever consumes it — so the orchestrator indexing service does integrate with this module.

ACCEPTANCE (L1248): round-trip pass, missing=>default no panic, mismatch=>notify+archive+rebuild, archive_created — ALL PASS.

Verdict: FULLY IMPLEMENTED.

------------------------------------------------------------------------
CROSS-CUTTING: .unwrap() / .expect() AUDIT IN RESILIENCE HOT PATH
------------------------------------------------------------------------
Production (non-test) offenses inside resilience files:
- reranker.rs:465 `.expect("semaphore closed")` — CONFIRMED VIOLATION (see 1I.5).
- startup.rs: all .unwrap()/.expect() are inside `#[cfg(test)] mod tests` (tempdir, save_reference_vec in tests, BM25 fixture init). ACCEPTABLE.
- degradation.rs: all .unwrap() in #[cfg(test)] mod tests. ACCEPTABLE.
- query_trace.rs: one `.unwrap_or_else` for serde_json fallback on emit() — SAFE. Tests have .unwrap() in helper (CounterLayer). ACCEPTABLE.
- circuit_breaker.rs: no .unwrap/.expect in non-test code.
- query_validator.rs: no .unwrap/.expect at all.
- index_version.rs: all .unwrap() are inside #[cfg(test)] mod tests. ACCEPTABLE.

Box<dyn> audit in knowledge/src/: grep for `Box<dyn` returns NO matches. No Box<dyn Error> antipattern present. Spec uses `Arc<dyn ClockProvider>` / `Arc<dyn SsbNotifier>` correctly. Clean.

------------------------------------------------------------------------
KNOWN-ISSUE STATUS (from prompt)
------------------------------------------------------------------------
(a) Dep graph missing 1G->1I edge (flagged in 1A audit): this is a spec/doc concern, not code. Phase 1I source clearly consumes reranker from 1G (reranker.rs:29 `pub use crate::degradation::{...}` + EmbeddingCircuitBreaker usage at :663-664). Dep is REAL in code. Doc update is a separate todo.

(b) Cross-encoder semaphore `expect("semaphore closed")` from 1G audit: CONFIRMED STILL PRESENT at reranker.rs:465. Not fixed.

(c) SSB degradation events "noop" in 1G CLI path: PARTIALLY TRUE. Reranker-level `knowledge_degradation` SSB emission (reranker.rs:390) exists and fires on cross-encoder breaker open, but the MAIN DegradationCascade (dense-vector model-failure path) is never re-evaluated during live queries — so embedding failures produce breaker state changes but NOT SSB `knowledge_degradation` events. The startup-time single-shot is the only live path.

------------------------------------------------------------------------
SUMMARY TABLE
------------------------------------------------------------------------
1I.1 Circuit breaker                    IMPLEMENTED (fresh-session-on-probe not wired)
1I.2 SearchMode cascade                 PARTIAL (logic complete, NOT wired to query path)
1I.3 Query validator                    IMPLEMENTED (production-wired)
1I.4 Startup readiness                  IMPLEMENTED (FSM gate missing, BM25-fail unproven)
1I.5 Cross-encoder semaphore            IMPLEMENTED (CLAUDE.md violation at :465)
1I.6 Query trace                        PARTIAL (mode hardcoded, latency=0, stats orphan)
1I.7 Index version schema               IMPLEMENTED

OVERALL VERDICT: PARTIALLY IMPLEMENTED, NOT PRODUCTION-READY.

The resilience crate has all the structural parts — enums, breakers, cascades, traces, validators — and each individually passes its own unit tests. But the integration layer that glues them into the live query path is missing in three critical spots:
  (1) the dense-vector DegradationCascade is evaluated once at startup and never again,
  (2) QueryTrace hardcodes HYBRID instead of consulting the cascade,
  (3) the cross-encoder semaphore worker panics via .expect() in production.
Result: under a real ONNX failure mid-session, the breaker will open correctly, but the pipeline will NOT fall back to BM25_ONLY, the SSB event will NOT fire, and the trace will LIE about the mode. The resilience is observable only at startup, not during operation.

Priority fixes (in order):
  P0: reranker.rs:465 — replace .expect("semaphore closed") with graceful worker exit.
  P0: pipeline.rs:520 — take a &DegradationCascade (or &AtomicU8 mode indicator) and call cascade.current_mode() (or evaluate) per query, feed into QueryTrace::new. Also thread total_latency_ms via an Instant::now() measured across the full function (or via ClockProvider for determinism).
  P1: wire DegradationCascade into QueryPipeline as a field, call evaluate() after stages that consulted a breaker (or at top of query after checking breaker states). Ensure SSB notifier used in CLI is the real SSB writer, not NoopSsbNotifier.
  P1: orchestrator FSM gate on ReadinessStatus (block IDLE->PLANNING on Unavailable per spec L1215).
  P2: add end-to-end integration test: inject ONNX permanent failure -> next query reports mode=BM25_ONLY in its QueryTrace AND emits knowledge_degradation SSB event AND BM25 still returns results.
  P2: move canary_reference_vec into index_version.toml per spec L1206 (currently separate JSON file).
  P2: wire SessionTraceStats into a session-end flush (currently orphaned).
  P3: enforce fresh ONNX InferenceSession on HALF_OPEN probe at each caller (ort_session.rs / reranker.rs).
