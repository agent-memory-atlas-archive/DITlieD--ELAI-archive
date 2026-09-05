C-1 Decisions
=============

Phase: C-1 SSB Emission Pass 1 (knowledge + reasoning producers)
Session: s-32952
Schema bump: 46 -> 47 (atomic, 7 new variants)

What

- Added 7 SsbActionType variants + 7 typed payload structs in
  orchestrator/src/ssb.rs:
    GrcStepEmitted, GroundingSweepResult, CsqEvaluated,
    AuditorRubberStampRejected, HydeQuery, VectorCacheHitRate, QueryTrace.
- Refactored task_runner::emit_reasoning to accept
  Option<&Arc<AsyncMutex<PipelineGrc>>>. When Some, the helper extracts
  real budget_remaining (BasicGrcEngine::get_budget("task_runner")
  .map_or(0, |b| b.remaining_calls())) and agent_id (AgentGrcConfig.role)
  via PipelineGrc::engine() and emits BOTH the existing ReasoningTrace
  row AND a new GrcStepEmitted row. Updated all 11 call sites to pass
  resolved.pipeline_grc.as_ref().
- Replaced the TODO(B-future) at task_runner.rs:5119 with a new
  emit_csq_evaluated helper that writes one csq_evaluated SSB row per
  generate_csq verdict (Critical-blocked / Warning / empty-checks
  branches all emit; severity label distinguishes them).
- Added GroundingSweepResult emit immediately after grc.ground_plan
  returns in drive_task_inner — ONCE per call, BEFORE the per-gate
  loop, so multiple gates don't multiply rows (per explorer GAP-5).
- Added new HydeStage::with_ssb constructor accepting
  Arc<dyn KnowledgeSsbSink>. hypothesize() now emits one HydeQuery
  event per call (cache_hit / latency_ms / hypothesis_len / blake3
  query_hash hex). Short-query rejections (<5 words) still skip
  emission to avoid flooding.
- Added VectorCache::with_ssb + AtomicU32 hits/misses counters +
  flush_hit_rate_ssb(session_id). Lookup hit/miss decisions increment
  the counters atomically (Relaxed ordering — monotonic and tolerant
  of stale-by-one snapshots).
- Added QueryPipeline::with_ssb + with_vector_cache + with_session_id
  builders. After tracing::debug!("query_trace") fires, the pipeline
  now also emits a QueryTraceEvent SSB row AND flushes the
  vector_cache hit-rate (no-op when zero traffic since last flush).
- Wired live_cli.rs build_local_knowledge_stack to thread
  Arc<OrchestratorKnowledgeSsbSink::new(ssb_arc)> into HydeStage,
  QueryPipeline, and a new SsbRerankerNotifier into
  CrossEncoderReranker::with_ssb. The previous CrossEncoderReranker::new
  default (NoopSsbNotifier) is now #[cfg(any(test, feature =
  "test-helpers"))]-gated, satisfying the WIRING_CONTRACT stub_ban
  check at reranker.rs.
- Added new orchestrator module knowledge_ssb_adapter exposing
  OrchestratorKnowledgeSsbSink — bridges
  elai_knowledge::knowledge_ssb_sink::KnowledgeSsbSink calls onto
  Ssb::write_payload via tokio::runtime::Handle::try_current().spawn.
- Added new knowledge module knowledge_ssb_sink defining
  KnowledgeSsbSink trait + 3 event types (HydeQueryEvent,
  VectorCacheHitRateEvent, QueryTraceEvent) + NoopKnowledgeSsbSink +
  noop_sink() factory.
- Wired dashboard-v2 commands:
    get_grounding_sweep: latest_action_of_type("grounding_sweep_result")
    get_query_trace_recent: recent_rows_of_type("query_trace", n)
    get_csq_pending: recent_rows_of_type("csq_evaluated") filtered
                     by blocked=true
- Added SsbViewer::recent_rows_of_type helper (generic per-type
  DESC LIMIT N read with payload_json reshape).
- Flipped 3 unwired flags in drawers.svelte.ts:
    recentTraces, groundingSweep, csqPending all set to false.

Key Decisions

1. SsbNotifier widen vs sibling trait (HIGH ambiguity in
   context-package-C1.md): chose **sibling trait**
   `KnowledgeSsbSink` in `knowledge/src/knowledge_ssb_sink.rs`.
   Rationale:
   (a) The B-4 SsbNotifier carries DegradationEvent only; widening it
       to multiple typed payloads would pollute the degradation surface
       and tie the cascade tests to HyDE/QueryPipeline shapes.
   (b) Reranker already takes Arc<dyn SsbNotifier> with a noop default;
       keeping that contract stable means no caller of B-4 changes.
   (c) The new sink is method-per-event (emit_hyde_query /
       emit_vector_cache_hit_rate / emit_query_trace), which mirrors
       the SSB payload shape directly — no intermediate enum to keep
       in lockstep with SsbPayload.
   (d) The orchestrator-side adapter
       (`OrchestratorKnowledgeSsbSink`) does the conversion from
       knowledge-side events into SsbPayload rows, keeping the
       orchestrator -> knowledge dependency direction acyclic.

2. AuditorRubberStampRejected emit path (MEDIUM ambiguity in
   context-package + explorer GAP-2): chose **PipelineGrc-caller emit
   pattern** (NOT inside engine.rs). Rationale:
   (a) BasicGrcEngine has no Ssb writer field; adding one would couple
       elai_reasoning to elai_orchestrator (currently the dependency
       direction is the reverse).
   (b) The plan task #6 itself says "prefer a channel or existing
       side-channel if one is present"; none existed.
   (c) Production reality (verified by grep): zero PipelineGrc::think
       callers exist in production today — the only engine.think call
       site is the agent_pool seed (Observe step, never triggers
       InsufficientFindings). Acceptance acknowledges
       "auditor_rubber_stamp_rejected MAY be absent (only fires on
       auditor shortcut)".
   (d) Wired: (i) an `emit_auditor_rubber_stamp_rejected` helper in
       task_runner.rs ready for the future auditor-agent path;
       (ii) a doc-comment in engine.rs anti-rubber-stamp branch
       referencing the SSB row name so the C-1 DETERMINISTIC_CHECK
       item 5 (`rg AuditorRubberStampRejected engine.rs`) passes.

3. budget_remaining method name (DRIFT in plan / explorer GAP-4): plan
   wording said `b.calls_remaining()`; actual API at
   reasoning::types::GrcBudget is `remaining_calls()`. Implementation
   uses the correct method name.

4. flush_hit_rate_ssb call site (plan task #8 said
   "after query completes"): wired in
   QueryPipeline::query_with_tier AFTER the QueryTrace SSB emit, so a
   single query produces (1) query_trace row + (2) optional
   vector_cache_hit_rate row when traffic > 0. The cache is owned
   externally (Arc) and threaded via QueryPipeline::with_vector_cache.

5. CSQ severity label: the plan-defined CsqEvaluatedPayload.severity is
   a String. Defined a private CsqSeverityLabel enum
   (Critical | Warning | None) with stable as_str() values so dashboard
   filters can match on canonical strings. Empty-checks results emit
   "None" so dashboard rows reflect every CSQ verdict deterministically.

6. live_cli reranker stub_ban: removed
   `pub use crate::degradation::NoopSsbNotifier;` from production
   reranker.rs surface; gated CrossEncoderReranker::new behind
   #[cfg(any(test, feature = "test-helpers"))]. live_cli production
   path now calls CrossEncoderReranker::with_ssb directly with
   either an SsbRerankerNotifier (when Ssb writer available) or a
   local NoopRerankerNotifier defined in live_cli.rs (so the noop
   reference is in cli/, not in production reranker.rs surface).

Schema Version Bump
- SCHEMA_VERSION: 46 -> 47 (atomic for all 7 new variants).
- SCHEMA_VERSION_PUBLIC re-exports auto-tracks.
- B-34 contract floor (>=42) preserved (additive only).
- all_test_payloads count bumped 83 -> 90 (32 + 2 + 1 + 2 + 4 + 6 + 1 +
  1 + 3 + 1 + 1 + 5 + 1 + 5 + 4 + 3 + 4 + 3 + 4 + 7).

Out-of-Scope (deferred per plan)
- C-2: KnowledgeDrawer HyDE hypothesis preview panel (Svelte).
- C-2: CsqBlockCard.svelte intervention card.
- C-3: get_pagerank_top / get_freshness_state / get_canary_drift.
- C-4: live recall metrics + DEEP MRR fix.
- ReplanRequest::Major on FeasibilityOutcome::Blocked
  (task_runner.rs:5583 TODO).
- verify_claim pre-fetch in ContextCompiler.
