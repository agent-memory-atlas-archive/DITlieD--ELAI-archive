# C-1: SSB Emission — Knowledge + Reasoning Producers

Objective: wire every defined-but-never-emitted knowledge and reasoning SSB
action type so a live cloud-Sonnet task writes real rows with non-null payloads
for all seven types: grc_step_emitted, grounding_sweep_result, csq_evaluated,
auditor_rubber_stamp_rejected, hyde_query, vector_cache_hit_rate, query_trace.
Estimated effort: 1 session (~6-8 h).
Pre-requisites: B-4 (knowledge crate + DegradationCascade shipped), B-5
(ReasoningDrawer layout shipped; SSB variants missing — this session adds them),
B-8 (PipelineGrc production callers wired; csq_evaluated TODO deferred here).
Parallel-safe with: C-2 (drawer Svelte wiring), C-3 (Tauri command liveness),
C-4 (recall metrics live read) — all dashboard-side, none touch ssb.rs or
task_runner.rs emit logic.

## Context

B-5.report:48-53 verified MISSING: none of the five reasoning SSB variants
(GrcStepEmitted, GroundingSweepResult, CsqEvaluated, AuditorRubberStampRejected,
FeasibilityCheck) exist in SsbActionType. Enum ends at CliSubprocessKilled
(ssb.rs:960). emit_reasoning at task_runner.rs:6228 hardcodes agent_id=
"task_runner", budget_remaining=0, elapsed_ms=0 (lines 6230-6240); the TODO at
lines 6218-6227 documents the exact refactor but it is unimplemented.

B-4.report:69-78 verified MISSING: hyde_query and vector_cache_hit_rate exist
nowhere in the codebase. pipeline.rs:644 calls tracing::debug("query_trace") —
no SSB write follows (B-4.report:17). NoopSsbNotifier is the production default
in CrossEncoderReranker::new at reranker.rs:123 (B-4.report:52-53).

B-8.report:37-39: csq_evaluated is deferred by TODO(B-future) at
task_runner.rs:5119-5120. AuditorRubberStampRejected: engine.rs:329-344 returns
GrcError::InsufficientFindings but never calls the SSB writer (B-5.report:58).
get_grounding_sweep at commands.rs:1566-1574 returns hardcoded zeros;
drawers.svelte.ts:230 marks unwired.groundingSweep=true (B-5.report:37-38).

All seven emit sites span only three crates (orchestrator, knowledge, reasoning).
The schema bump is atomic; dashboard consumers (C-2/C-3/C-4) read these rows
without requiring changes on this branch.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-4_knowledge_producer_wiring.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-5_reasoning_producer_wiring.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-8_pipelinegrc_activation.report.md
- .elai_cc/crates/orchestrator/src/ssb.rs (SsbActionType enum line 961; as_sql_str/from_str arms; SsbPayload enum; SCHEMA_VERSION)
- .elai_cc/crates/orchestrator/src/task_runner.rs (emit_reasoning line 6228; TODO 6218-6227; csq_evaluated TODO line 5119; ground_plan call line 4438)
- .elai_cc/crates/reasoning/src/engine.rs (anti-rubber-stamp block line 329)
- .elai_cc/crates/knowledge/src/hyde.rs (HydeClient trait; HydeStage cache field line 80)
- .elai_cc/crates/knowledge/src/vector/cache.rs (VectorCache struct line 51)
- .elai_cc/crates/knowledge/src/pipeline.rs (tracing::debug lines 639-645; run_query signature)
- .elai_cc/crates/knowledge/src/reranker.rs (NoopSsbNotifier default line 123)
- .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs (get_grounding_sweep line 1566; get_query_trace_recent line 824; get_csq_pending line 1578)
- .elai_cc/crates/dashboard-v2/src/lib/state/drawers.svelte.ts (unwired flags lines 149, 150, 230, 231)

## Tasks

1. **ssb.rs — add 7 variants**: after CliSubprocessKilled (line 960) add
   `GrcStepEmitted`, `GroundingSweepResult`, `CsqEvaluated`,
   `AuditorRubberStampRejected`, `HydeQuery`, `VectorCacheHitRate`, `QueryTrace`.
   Add as_sql_str + from_str arms for each. Bump SCHEMA_VERSION by 1.

2. **ssb.rs — add payload structs** for each variant: `GrcStepEmittedPayload
   { agent_id, topic, step_type, grounding_level, confidence:f64,
   budget_remaining:u32, elapsed_ms:u64 }`, `GroundingSweepResultPayload
   { ungrounded:u32, verify_needed:u32, grounded:u32, top_claim:Option<String> }`,
   `CsqEvaluatedPayload { severity:String, guard_present:bool, blocked:bool,
   check_count:u32 }`, `AuditorRubberStampRejectedPayload { role:String,
   finding_count:u32, expected_min:u32 }`, `HydeQueryPayload { query_hash:String,
   hypothesis_len:u32, cache_hit:bool, latency_ms:u64 }`,
   `VectorCacheHitRatePayload { session_id:String, hits:u32, misses:u32,
   hit_rate_pct:f64 }`, `QueryTracePayload { query_id:String, tier:String,
   stage_count:u32, result_count:u32, total_latency_ms:u64 }`. Add SsbPayload
   arms and SQL serialization for each.

3. **task_runner.rs — refactor emit_reasoning** (B-5 T3 TODO lines 6218-6227):
   add `grc: Option<&Arc<AsyncMutex<PipelineGrc>>>` parameter; when Some, extract
   real budget_remaining/agent_id/topic/elapsed_ms; emit `SsbPayload::GrcStepEmitted`
   in addition to existing ReasoningTrace row. Update all 11 call sites to pass
   `resolved.pipeline_grc.as_ref()`.

4. **task_runner.rs:5119 — emit CsqEvaluated** (replace TODO(B-future) comment):
   write `SsbPayload::CsqEvaluated(CsqEvaluatedPayload { severity, guard_present,
   blocked: hook_pre_blocked, check_count })` via writer.write_payload().

5. **task_runner.rs:4438-4440 — emit GroundingSweepResult** immediately after
   ground_plan returns: populate from grounding struct already in scope
   (ungrounded.len(), grounded.len(), verify.len(), ungrounded.first().cloned()).

6. **reasoning/src/engine.rs:340 — emit AuditorRubberStampRejected** inside
   GrcError::InsufficientFindings branch. Inject `ssb_writer: Option<Arc<Ssb>>`
   into BasicGrcEngine::advance (check existing method signature before adding;
   prefer a channel or existing side-channel if one is present).

7. **knowledge/src/hyde.rs — emit HydeQuery**: in HydeStage::run, record
   cache_hit and latency; emit `SsbPayload::HydeQuery(HydeQueryPayload {
   query_hash: blake3_hex(query), hypothesis_len, cache_hit, latency_ms })` via
   injected `Arc<dyn SsbNotifier>` (same trait as degradation.rs).

8. **knowledge/src/vector/cache.rs — emit VectorCacheHitRate**: add `hits:u32,
   misses:u32` counters to VectorCache; add `flush_hit_rate_ssb(session_id,
   writer)` method that emits `VectorCacheHitRatePayload` and resets counters.
   Call from QueryPipeline::run_query after query completes.

9. **knowledge/src/pipeline.rs:639-645 — replace tracing::debug with dual emit**:
   keep tracing::debug, then immediately write `SsbPayload::QueryTrace(
   QueryTracePayload { query_id, tier, stage_count, result_count, total_latency_ms
   })`. Add `ssb: Option<Arc<dyn SsbNotifier>>` field to QueryPipeline; thread
   the concrete Ssb arc from live_cli.rs:1056-1081 (same site where HyDE is wired).

10. **reranker.rs:123 — replace NoopSsbNotifier**: thread the real Ssb arc from
    live_cli.rs into CrossEncoderReranker::new.

11. **commands.rs:1566-1574 — wire get_grounding_sweep** to read real
    GroundingSweepResult SSB rows via SsbViewer. Remove hardcoded zeros.
    Set unwired.groundingSweep=false in drawers.svelte.ts:230.

12. **commands.rs:824-828 — wire get_query_trace_recent** to return real
    query_trace rows from SsbViewer (remove json!([])). Set unwired.recentTraces=false.

13. **commands.rs:1578-1579 — wire get_csq_pending** to return csq_evaluated rows
    where blocked=true. Set unwired.csqPending=false in drawers.svelte.ts:231.

14. **Write workflow/reasoning/C-1-decisions.md**: document SsbNotifier injection
    approach chosen for HydeStage and VectorCache, writer path into engine.rs:340,
    final SCHEMA_VERSION after bump.

## WIRING_CONTRACT

```
WIRING_CONTRACT:
  produces_symbols:
    - SsbActionType::GrcStepEmitted
    - SsbActionType::GroundingSweepResult
    - SsbActionType::CsqEvaluated
    - SsbActionType::AuditorRubberStampRejected
    - SsbActionType::HydeQuery
    - SsbActionType::VectorCacheHitRate
    - SsbActionType::QueryTrace
    - GrcStepEmittedPayload / GroundingSweepResultPayload / CsqEvaluatedPayload
    - AuditorRubberStampRejectedPayload / HydeQueryPayload
    - VectorCacheHitRatePayload / QueryTracePayload
    - VectorCache::flush_hit_rate_ssb
  consumes_symbols:
    - PipelineGrc (real budget/agent_id extraction in emit_reasoning)
    - SsbNotifier trait (injection into HydeStage + VectorCache + QueryPipeline)
    - BasicGrcEngine::advance (AuditorRubberStampRejected emit path)
    - ground_plan return value (GroundingSweepResult populate source)
    - generate_csq result (CsqEvaluated populate source)
  reachable_from:
    - elai /task --mode cloud (exercises GRC, CSQ, HyDE, query pipeline)
  stub_ban:
    - NoopSsbNotifier MUST NOT remain default in CrossEncoderReranker::new
    - emit_reasoning MUST NOT use hardcoded agent_id="task_runner" when PipelineGrc is Some
    - get_grounding_sweep MUST NOT return hardcoded zeros after this session
    - get_query_trace_recent MUST NOT return json!([]) after this session
  observable_effect:
    - sqlite3 .elai/sessions/{id}/ssb.db
      "SELECT action_type, COUNT(*) FROM ssb_actions
       WHERE action_type IN ('grc_step_emitted','grounding_sweep_result',
       'csq_evaluated','auditor_rubber_stamp_rejected','hyde_query',
       'vector_cache_hit_rate','query_trace') GROUP BY action_type"
      -- must return >= 5 distinct types, count >= 1 each, payload non-null
```

## DETERMINISTIC_CHECK

```bash
# 1. All 7 variants present (expect >= 3 hits each: enum + as_sql_str + from_str)
rg "GrcStepEmitted|GroundingSweepResult|CsqEvaluated|AuditorRubberStampRejected|HydeQuery|VectorCacheHitRate|QueryTrace" \
  .elai_cc/crates/orchestrator/src/ssb.rs | wc -l
# Expected: >= 21

# 2. emit_reasoning accepts PipelineGrc parameter
rg "fn emit_reasoning.*PipelineGrc\|fn emit_reasoning.*Option.*Grc" \
  .elai_cc/crates/orchestrator/src/task_runner.rs
# Expected: 1 match

# 3. CsqEvaluated emit present at former TODO site (task_runner.rs ~line 5119)
rg "CsqEvaluated" .elai_cc/crates/orchestrator/src/task_runner.rs
# Expected: >= 2 hits

# 4. GroundingSweepResult emit after ground_plan
rg -A 10 "ground_plan" .elai_cc/crates/orchestrator/src/task_runner.rs | grep GroundingSweepResult
# Expected: 1 match

# 5. AuditorRubberStampRejected emitted in engine.rs
rg "AuditorRubberStampRejected" .elai_cc/crates/reasoning/src/engine.rs
# Expected: >= 1 match

# 6. HydeQuery emit in hyde.rs
rg "HydeQuery\|HydeQueryPayload" .elai_cc/crates/knowledge/src/hyde.rs
# Expected: >= 1 match

# 7. QueryTrace SSB write in pipeline.rs (not just tracing::debug)
rg "SsbPayload::QueryTrace\|QueryTracePayload" .elai_cc/crates/knowledge/src/pipeline.rs
# Expected: >= 1 match

# 8. NoopSsbNotifier gone from reranker production path
rg "NoopSsbNotifier" .elai_cc/crates/knowledge/src/reranker.rs | grep -v "#\[cfg(test\)\]\|//\|tests/"
# Expected: 0 matches

# 9. get_grounding_sweep no longer hardcoded zeros
rg "grounded.*:.*0\b" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs \
  | grep -A3 -B3 "grounding_sweep"
# Expected: 0 matches inside get_grounding_sweep body

# 10. cargo check passes
cargo check --workspace 2>&1 | grep "^error" | head -5
# Expected: 0 lines

# 11. SCHEMA_VERSION >= 47
rg "SCHEMA_VERSION\s*=\s*[0-9]+" .elai_cc/crates/orchestrator/src/ssb.rs | head -1
# Expected: value >= 47
```

## ACCEPTANCE

- [ ] `cargo check --workspace` passes — zero errors
- [ ] `cargo test -p elai-orchestrator` passes (includes new variant contract tests)
- [ ] `cargo test -p elai-knowledge` passes (pipeline, hyde, vector cache tests)
- [ ] `cargo test -p elai-reasoning` passes (anti-rubber-stamp test extended)
- [ ] DETERMINISTIC_CHECK items 1-11 all pass
- [ ] SCHEMA_VERSION incremented by exactly 1 (one atomic bump for all 7 variants)
- [ ] ReasoningDrawer budget gauges (`ReasoningDrawer.svelte` lines 164-183)
      show at least one agent_id that is NOT "task_runner" in a live GRC session
- [ ] KnowledgeDrawer recentTraces section renders >= 1 query_trace row after query
- [ ] drawers.svelte.ts `unwired.groundingSweep`, `unwired.recentTraces`,
      `unwired.csqPending` all set to `false`
- [ ] Cloud-Sonnet smoke test (MANDATORY — OAuth path PROVEN end-to-end):

```bash
# Run a task that exercises GRC, CSQ, HyDE, and query pipeline
elai /task --mode cloud "implement a pure-Rust add(a: i32, b: i32) -> i32 function in math.rs with doc-comment and unit test"

SESSION_ID=$(ls -t .elai/sessions/ | head -1)

# Query all seven new action types
sqlite3 ".elai/sessions/${SESSION_ID}/ssb.db" \
  "SELECT action_type, COUNT(*) AS n, MAX(LENGTH(payload)) AS max_payload_bytes
   FROM ssb_actions
   WHERE action_type IN (
     'grc_step_emitted','grounding_sweep_result','csq_evaluated',
     'auditor_rubber_stamp_rejected','hyde_query',
     'vector_cache_hit_rate','query_trace'
   )
   GROUP BY action_type ORDER BY action_type;"
# PASS: >= 5 distinct rows, n >= 1, max_payload_bytes > 0 for each row.
# grc_step_emitted and query_trace MUST appear on every /task run.
# auditor_rubber_stamp_rejected MAY be absent (only fires on auditor shortcut).

# Spot-check that grc_step_emitted carries real agent_id, not synthetic constant
sqlite3 ".elai/sessions/${SESSION_ID}/ssb.db" \
  "SELECT json_extract(payload,'$.agent_id'), json_extract(payload,'$.budget_remaining')
   FROM ssb_actions WHERE action_type='grc_step_emitted' LIMIT 3;"
# PASS: at least one row where budget_remaining > 0 (real PipelineGrc value)
```

## TESTS

- ADD: `.elai_cc/crates/orchestrator/tests/ssb_new_variants_contract.rs` —
  round-trip all 7 variants through as_sql_str/from_str; serialize each payload
  struct; assert non-empty JSON output.
- ADD: `.elai_cc/crates/orchestrator/tests/emit_reasoning_grc_contract.rs` —
  mock PipelineGrc with known budget; call emit_reasoning with Some(&grc_arc);
  assert GrcStepEmittedPayload.budget_remaining matches mock value, not 0.
- ADD: `.elai_cc/crates/knowledge/tests/hyde_ssb_emit_contract.rs` — drive
  HydeStage::run with MockHydeClient + recording SsbNotifier; assert one
  HydeQuery row per call; assert cache_hit=true on second identical query.
- ADD: `.elai_cc/crates/knowledge/tests/vector_cache_hit_rate_contract.rs` —
  N cache get+put cycles; call flush_hit_rate_ssb; assert hit_rate_pct within
  0.01 of hits/(hits+misses)*100.
- ADD: `.elai_cc/crates/knowledge/tests/query_trace_ssb_contract.rs` — run
  minimal QueryPipeline against in-memory index; assert exactly one QueryTrace
  row with result_count >= 0 and total_latency_ms >= 0.
- EXTEND: `.elai_cc/crates/reasoning/tests/agent_config.rs` (line 353 region) —
  assert BasicGrcEngine::advance into Converge without prior FINDING emits
  AuditorRubberStampRejected row via mock writer.
- EXTEND: `.elai_cc/crates/orchestrator/tests/production_agent_rpc_contract.rs` —
  assert >= 1 grc_step_emitted row in in-memory SSB after a pipeline GRC step.

## Out-of-scope

- C-2: KnowledgeDrawer HyDE hypothesis preview panel (Svelte component)
- C-2: CsqBlockCard.svelte intervention card in ChatWindow
- C-3: get_pagerank_top, get_freshness_state, get_canary_drift Tauri commands
- C-4: live recall metrics; DEEP MRR >= STANDARD MRR fix; per-stage latency_ms
- ReplanRequest::Major on FeasibilityOutcome::Blocked (task_runner.rs:5583 TODO)
- verify_claim wiring into ContextCompiler pre-fetch (agent_pool.rs:559)
- elai_cc.toml [models.cross_encoder] documentation
