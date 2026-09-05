# C-3: Tauri Commands — Knowledge / Reasoning Dashboard Surface
Objective: replace every hardcoded zero, empty array, and stub return in the
knowledge and reasoning Tauri command layer with real SSB reads or live DB
queries; add six missing commands that have no implementation at all.
Estimated effort: 1-2 sessions.
Pre-requisites: C-1 ships first (writes query_trace and csq_evaluated SSB rows
that these viewer methods consume).
Parallel-safe with: C-4 (different Tauri commands, no file overlap).

## Context

Seven viewer methods in ssb_viewer.rs return hardcoded zeros or empty vecs with
stub comments (verified 2026-05-17 from source):
- pagerank_top (line 2699): Ok(Vec::new()), comment "No SSB row type persists
  per-symbol PageRank scores"
- query_tier_stats (line 2708): all-zero QueryTierStats, comment "Reserved until
  a dedicated SSB row records per-query tier decisions"
- freshness_state (line 2722): known:false FreshnessState, comment "freshness
  meta DB does not expose a read endpoint yet"
- research_budget_snapshot (line 3055): known:false ResearchBudgetSnapshot,
  comment "ResearchBudgetManager is in-memory only; no SSB row type carries its
  progression yet"
- source_tier_distribution (line 3073): known:false SourceTierDistribution,
  comment "5-tier domain lookup is deterministic but counts not persisted"

Six Tauri commands are missing entirely from commands.rs (B-4 / B-5 reports):
- get_canary_drift — no implementation, no lib.rs registration
- get_csq_log — no implementation, no lib.rs registration
- get_feasibility_log — no implementation (get_feasibility_report exists but has
  different name and shape per B-5 report)
- get_agent_grc_config — no implementation, no lib.rs registration
- research_budget_snapshot Tauri surface — ssb_viewer method exists, no command
- source_tier_distribution Tauri surface — ssb_viewer method exists, no command

Additionally, get_query_trace_recent (commands.rs:824) returns json!([]) with a
comment "No first-class QueryTrace SSB row today"; after C-1 lands the SSB row
exists and this command must read it. The drawers.svelte.ts unwired flag at
line 149 (recentTraces:true) must be cleared.

AppState.writer is typed Option<Arc<Ssb>> (state.rs:220); no SsbWriter trait
abstraction is needed — the viewer path (state.viewer: Option<Arc<SsbViewer>>)
is sufficient for all read commands in this plan.

The frontend QueryTrace interface (types.ts:448) already matches the shape C-1
will emit: id, query, mode, stages[], total_ms, ts, hit_count.

## Reads

- .elai_cc/crates/orchestrator/src/ssb_viewer.rs lines 620-660, 2695-2730,
  3050-3084 (stub bodies + struct definitions)
- .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs lines 820-828
  (get_query_trace_recent stub body)
- .elai_cc/crates/dashboard-v2/src-tauri/src/state.rs lines 212-230
  (AppState fields: viewer, writer)
- .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs lines 88-110
  (command registration block for knowledge/reasoning section)
- .elai_cc/crates/dashboard-v2/src/lib/state/drawers.svelte.ts lines 106, 149
  (recentTraces state + unwired flag)
- .elai_cc/crates/dashboard-v2/src/lib/api/types.ts lines 448-456
  (QueryTrace interface shape)
- docs/thefinalglance/verification/reports/B-plan/B-3_stub_command_eradication.report.md
  (pagerank_top, query_tier_stats, freshness_state action items)
- docs/thefinalglance/verification/reports/B-plan/B-4_knowledge_producer_wiring.report.md
  (get_canary_drift, get_pagerank_top, get_freshness_state missing commands)
- docs/thefinalglance/verification/reports/B-plan/B-5_reasoning_producer_wiring.report.md
  (get_csq_log, get_feasibility_log, get_agent_grc_config missing commands)

## Tasks

1. Implement query_trace SSB read in ssb_viewer.rs.
   Add pub fn recent_query_traces(&self, limit: u32) -> Result<Vec<QueryTraceRow>,
   SsbViewerError> that SELECTs the most recent `limit` rows WHERE action_type =
   'query_trace' ORDER BY seq DESC. Deserialize each payload via rmp_serde into
   the QueryTrace SSB payload type C-1 defines.

2. Wire get_query_trace_recent to the live viewer.
   Replace the json!([]) body in commands.rs:824 with:
   state.viewer.as_ref().and_then(|v| v.recent_query_traces(n as usize).ok())
   .unwrap_or_default() serialized to Value::Array.
   Set drawers.svelte.ts line 149 unwired.recentTraces to false.

3. Implement pagerank_top SQL in ssb_viewer.rs:2699.
   Replace Ok(Vec::new()) with:
   SELECT symbol, path, score FROM dkg_nodes_pagerank
   ORDER BY score DESC LIMIT ?1
   Map rows into Vec<PageRankEntry>. Return Ok(Vec::new()) without error when
   table does not exist (catch SqliteFailure with code=1 or use IF EXISTS probe).

4. Add get_pagerank_top Tauri command in commands.rs.
   Accept n: u32 (default 20). Call state.viewer pagerank_top(n as usize).
   Register in lib.rs alongside get_query_trace_recent.

5. Implement query_tier_stats aggregate in ssb_viewer.rs:2708.
   Replace all-zero body with a SELECT aggregating query_trace SSB rows:
   count distinct tier values (fast/standard/deep from QueryTrace.mode) and
   AVG(total_ms) per tier from the most recent 500 rows. Populate sample_size.
   Return zeroed struct with known:false sentinel via sample_size=0 when no
   rows exist (no behavioral change until C-1 lands).

6. Implement freshness_state real DB read in ssb_viewer.rs:2722.
   Read the most recent freshness_meta SSB row (action_type='freshness_computed'
   if C-1 defines it) or fall back to reading the config decay_rate and
   access_boost from elai_cc.toml via the existing ConfigManager in AppState.
   Set known:true only when a real source is found.

7. Add get_freshness_state Tauri command in commands.rs.
   Call state.viewer freshness_state(). Register in lib.rs.

8. Add get_canary_drift Tauri command.
   Read the most recent canary_drift SSB row via SsbViewer (action_type=
   'canary_drift'; C-1 defines this row from the HyDE readiness check path).
   Return structured JSON: {delta_ms, baseline_ms, current_ms, drift_pct, ts}.
   Fall back to {known:false} when no row exists.
   Register in lib.rs.

9. Add get_csq_log Tauri command (B-5).
   SELECT the most recent 50 csq_evaluated SSB rows (C-1 writes these). Return
   array of {csq_id, conclusion, guard_present, ts_ms}. Empty array when no
   rows exist (not an error). Register in lib.rs.

10. Add get_feasibility_log Tauri command (B-5).
    Distinct from existing get_feasibility_report (Batch 3A shape). SELECT the
    most recent 20 feasibility_check SSB rows. Return array of
    {task_id, verdict, score, ts_ms}. Register in lib.rs.

11. Add get_agent_grc_config Tauri command (B-5).
    Read AgentGrcConfig table from agent_pool.rs:63 via the SSB db or the
    config store. Return array of {role, max_calls, max_chars, topic_prefix}.
    Static read from compiled defaults if no runtime override row exists.
    Register in lib.rs.

12. Wire research_budget_snapshot Tauri command.
    The ssb_viewer method already exists (line 3055). Add a #[tauri::command]
    pub fn get_research_budget(state) wrapper that calls it and serializes to
    Value. When C-1 lands a research_budget_updated SSB row, update the viewer
    method to read it; until then the known:false fallback is correct.
    Register in lib.rs.

13. Wire source_tier_distribution Tauri command.
    Same pattern as Task 12 — wrap the existing viewer method (line 3073).
    Register in lib.rs.

## WIRING_CONTRACT

produces_symbols:
  - SsbViewer::recent_query_traces (new method, ssb_viewer.rs)
  - SsbViewer::pagerank_top (real SQL, replaces Ok(Vec::new()))
  - SsbViewer::query_tier_stats (real aggregate, replaces zeros)
  - SsbViewer::freshness_state (real read, replaces known:false stub)
  - Tauri commands: get_pagerank_top, get_freshness_state, get_canary_drift,
    get_csq_log, get_feasibility_log, get_agent_grc_config,
    get_research_budget, get_source_tier_distribution
  - drawers.svelte.ts: unwired.recentTraces = false
consumes_symbols:
  - AppState.viewer: Option<Arc<SsbViewer>> (state.rs:216)
  - SsbViewer::lock() (existing DB access pattern)
  - SsbActionType::QueryTrace (C-1 produces this variant)
  - SsbActionType::CsqEvaluated (C-1 produces this variant)
  - dkg_nodes_pagerank table (existing schema, knowledge crate)
  - QueryTraceRow / QueryTrace payload shape (C-1 defines)
reachable_from:
  - KnowledgeDrawer.svelte polling via drawers.svelte.ts
  - ReasoningDrawer.svelte polling via drawers.svelte.ts
stub_ban:
  - pagerank_top MUST NOT return Ok(Vec::new()) after this C-3 ships
  - get_query_trace_recent MUST NOT return json!([]) after C-1 is active
  - get_canary_drift MUST return {known:false} (not panic) when no SSB row exists
  - All six new commands MUST be registered in lib.rs
frontend_pair: KnowledgeDrawer.svelte + ReasoningDrawer.svelte via drawers.svelte.ts
observable_effect:
  trigger: elai /task --mode cloud "search the codebase for foo and explain"
  expect_ssb: query_trace rows written by C-1 pipeline path
  expect_tauri: get_query_trace_recent returns >= 1 row; unwired.recentTraces = false

## DETERMINISTIC_CHECK

```
# 1. recent_query_traces method added to ssb_viewer.rs
rg "fn recent_query_traces" .elai_cc/crates/orchestrator/src/ssb_viewer.rs

# 2. pagerank_top no longer returns Vec::new() unconditionally
rg "Ok\(Vec::new\(\)\)" .elai_cc/crates/orchestrator/src/ssb_viewer.rs
# expected: 0 hits in the pagerank_top fn body (grep for surrounding context)

# 3. get_query_trace_recent no longer returns json!([]) unconditionally
rg "json!\(\[\]\)" .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
# expected: 0 hits inside get_query_trace_recent fn body

# 4. All 8 new/updated commands registered in lib.rs
rg "get_pagerank_top|get_freshness_state|get_canary_drift|get_csq_log|get_feasibility_log|get_agent_grc_config|get_research_budget|get_source_tier_distribution" .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs
# expected: 8 hits

# 5. unwired.recentTraces cleared
rg "recentTraces: true" .elai_cc/crates/dashboard-v2/src/lib/state/drawers.svelte.ts
# expected: 0 hits

# 6. Contract tests compile
cargo test -p elai-contracts -- knowledge_surface 2>&1 | grep "test result"
```

## ACCEPTANCE

- [ ] get_query_trace_recent returns >= 1 row after `elai /task --mode cloud
      "search the codebase for foo and explain"` populates query_trace SSB rows
      (C-1 required); each row has non-null stage_latencies matching QueryTrace
      interface (types.ts:448)
- [ ] get_pagerank_top returns >= 1 entry with non-zero score when dkg_nodes_pagerank
      table has rows; returns empty array (not error) when table is empty
- [ ] get_freshness_state returns a structured FreshnessState with known != false
      when at least one freshness source is readable; known:false acceptable when
      no source is present
- [ ] get_canary_drift returns {known:false} when no canary_drift SSB row exists;
      returns structured drift object when C-1 emits one
- [ ] get_csq_log returns >= 1 entry after a /task run with GRC enabled (C-1
      writes csq_evaluated rows)
- [ ] get_feasibility_log registered and returns typed array (empty when no rows)
- [ ] get_agent_grc_config returns at least 8 role entries (one per named agent role)
- [ ] get_research_budget and get_source_tier_distribution registered and return
      known:false stubs (acceptable until C-1 research-budget SSB row lands)
- [ ] drawers.svelte.ts unwired.recentTraces = false (compile-time check)
- [ ] cargo check --workspace passes with 0 errors
- [ ] All 8 new commands appear in lib.rs registration block
- [ ] No .unwrap() calls in any new command handler or ssb_viewer method body
      (CLAUDE.md hard rule: use ? / .ok_or() / .map())

Cloud-Sonnet smoke test (run after C-1 is active):
  Terminal A: elai /task --mode cloud "search the codebase for foo and explain"
  Terminal B (dashboard session):
    assert get_query_trace_recent(10) returns >= 1 row with non-null stage_latencies
    assert get_pagerank_top(20) returns >= 1 entry with score > 0.0
    assert get_freshness_state().known != false
    assert get_csq_log() returns >= 1 entry

## Tests

Location: .elai_cc/crates/contracts/tests/

1. knowledge_surface_contract.rs
   - test_get_query_trace_recent_empty: MockRuntime, no SSB rows -> returns []
   - test_get_query_trace_recent_live: insert synthetic query_trace row into
     temp SSB db, assert command returns 1 row with expected fields
   - test_get_pagerank_top_empty_table: dkg_nodes_pagerank absent -> returns [],
     no panic
   - test_get_pagerank_top_with_rows: insert 3 rows, assert ORDER BY score DESC
     order preserved in response
   - test_get_freshness_state_stub: no freshness source -> known:false returned
   - test_get_canary_drift_no_rows: command returns {known:false} JSON object
   - test_get_query_tier_stats_no_rows: command returns sample_size:0 JSON

2. reasoning_surface_contract.rs
   - test_get_csq_log_empty: no csq_evaluated SSB rows -> returns []
   - test_get_csq_log_with_rows: insert synthetic csq_evaluated row, assert
     {csq_id, conclusion, guard_present, ts_ms} shape returned
   - test_get_feasibility_log_empty: returns [] without error
   - test_get_agent_grc_config_returns_eight_roles: assert len >= 8

## Out of scope

- Writing query_trace SSB rows in the pipeline — that is C-1 work
- Writing csq_evaluated SSB rows in task_runner.rs — that is C-1 work
- Fixing per-stage latency_ms hardcoded zeros in pipeline.rs — B-4 deferred
- NoopSsbNotifier replacement in cli/live_cli.rs — B-4 deferred
- Canary embed("fn main()") correction in pipeline.rs — B-4 deferred
- ts-rs type generation — B-3 deferred
- MonitorWindow feature-gate for AutoAdvanceStats/ProposalsPanel — B-3 deferred
- CsqBlockCard.svelte intervention card — B-5 deferred
- get_grounding_sweep real data (requires B-8 GroundingSweepGate producer)
- research_budget SSB row type definition — C-1 scope
- source_tier_distribution SSB row type definition — C-1 scope
- ELAI_DASHBOARD_OFFLINE flag gating in stub.ts — B-36 deferred
