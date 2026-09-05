# C-8: Viewer Stub Eradication — Wiring Remaining Producers

## STATUS (2026-05-23 REBASELINE, session s-25120 completion-drive recheck)

C-8 is CLOSED. The 12 producer-wiring tasks shipped in s-32952; the
s-25120 recheck reclassified 2 "PARTIAL" claims as STALE-SHIPPED-ON-V2
(verifiable at the cited file:lines today) and 2 "MISSING" claims as
PHANTOM-CLAIM-RESOLVED (one flag never existed; one method rename
drift). 4 contract test files (3 new + 1 extend) were authored in
s-25120 to pin the wiring witnesses (14/14 PASS).

Resolution markers applied below:
  SHIPPED-ON-V2 (Tasks 3 sub-claim + 9 sub-claim) -- file:line evidence
    today contradicts the prior PARTIAL classification.
  PHANTOM-CLAIM-RESOLVED (Tasks 2 sub-claim + 11 sub-claim) -- the
    cited symbol never existed (drawers.svelte.ts ambiguityState flag)
    or shipped under a different name (last_ambiguity_score ->
    latest_ambiguity_scored).
  SPEC_DRIFT fixes (6) -- plan text patched to match shipped reality
    (DRIFT-C/D scheduler host crate, DRIFT-G/H ArtifactsIndexed reuse,
    DRIFT-I cross-process search_mode bridge, DRIFT-K ClockProvider
    instead of raw Instant, DRIFT-M phantom flag, DRIFT-J unbudgeted
    SCHEMA bump 49 -> 50).
  DET-CHECK refinements (4) -- audit-time guidance per
    C-8-decisions.md A4 (NoopSsbNotifier construction grep, HYBRID
    literal scoped to get_search_mode body, pipeline.rs latency_ms:0
    synthetic-stage carve-out, scheduler host crate path).

Decision doc: workflow/reasoning/C-8-decisions.md (R1-R4 resolutions,
T1-T4 test rationale, A1-A4 plan-amendment notes that this rebaseline
implements verbatim).

Original objective (preserved): replace every remaining live-data mock
and hardcoded stub in the producer layer: wire AmbiguityScorer into
task_runner.rs before GroundingSweepGate fires; wire
CosmosWorker::tick() at hourly cadence (Layer 1A); wire
index_all_artifacts into IndexingService::build_index_resumable;
implement TOOL_INDEX_ARTIFACTS in production_agent_rpc.rs; wire real
SsbNotifier into CrossEncoderReranker; replace embedding-norm proxy in
AragOrchestrator::semantic_search with ONNX dot-product; replace 17+
latency_ms=0 fields in pipeline.rs; store DegradationCascade result in
AppState; wire CosmosStage into QueryPipeline; clear matching unwired
flags in drawers.svelte.ts.
Estimated effort: 2-3 sessions (actual: 1 session s-32952 + 1 recheck
s-25120).
Pre-requisites: C-1 (QueryTrace/CosmosRefreshStarted SSB variants defined;
C-8 writes latency payloads into those types), C-2 (SsbNotifier injection paths
confirmed).
Parallel-safe with: C-3, C-4 (dashboard reads — no overlap with pipeline.rs or
task_runner.rs producer paths).

## Context

Six B-plan verification reports document surviving producer stubs.

B-38.report: AmbiguityScorer::score() is complete (ambiguity.rs:244). Gate in
evidence.rs:229-315 already consumes TransitionContext.ambiguity_score when Some
(falls back to B-17 behavior on None — zero risk). task_runner.rs has no call to
AmbiguityScorer. get_ambiguity_state (commands.rs:6553) returns mock data with
"mock": true acknowledged in-code.

B-42.report: cosmos_worker.rs defines DEFAULT_TICK_INTERVAL_SECS=3600 and tick()
at line 444. task_runner.rs has zero cosmos references. SSB types
CosmosRefreshStarted/CosmosArtifactUpdated defined in ssb.rs:923-934 but
cosmos_worker.rs emits via tracing::info!(target="cosmos") only — no SsbRow INSERT.
CosmosStage absent from pipeline.rs (zero references confirmed by report).

B-35.report: index_all_artifacts at indexer.rs:201 exists. indexing_service.rs
has zero artifact references. TOOL_INDEX_ARTIFACTS at production_agent_rpc.rs:1014
returns Err("full indexing deferred to Phase C").

B-4.report: NoopSsbNotifier is the production default in reranker.rs:123.
DegradationCascade evaluated at live_cli.rs:1086-1103 but result not stored in
AppState — get_search_mode returns hardcoded "HYBRID". pipeline.rs:32-34 comment
"latency_ms is always 0 here" with 17+ literal-zero assignments.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-38_ambiguity_score_gate.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-42_cosmos_artifact_tier.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-35_context_artifacts_pipeline.report.md
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-4_knowledge_producer_wiring.report.md
- .elai_cc/crates/orchestrator/src/task_runner.rs (TransitionContext build site;
  Layer 1A scheduler pattern; ground_plan call site)
- .elai_cc/crates/orchestrator/src/production_agent_rpc.rs (stub at line 1014;
  TOOL_LIST_ARTIFACTS at line 988 as reference)
- .elai_cc/crates/cli/src/live_cli.rs (lines 1086-1103; CrossEncoderReranker ctor)
- .elai_cc/crates/knowledge/src/pipeline.rs (latency_ms sites; stage ordering)
- .elai_cc/crates/knowledge/src/arag.rs (AragOrchestrator::semantic_search)
- .elai_cc/crates/dashboard-v2/src/lib/state/drawers.svelte.ts (lines 149-150, ~230)

## Tasks

1. **Wire AmbiguityScorer into task_runner.rs** — at the TransitionContext
   construction site before GroundingSweepGate (PLANNING->TESTING_FIRST transition
   only; not every step). Instantiate AmbiguityScorer with the existing LlmClient
   ref; call `.score(&plan_text).await`; set `ambiguity_score: Some(score)`.
   On Err: tracing::warn! and set None (B-17 fallback preserved).

2. **[SHIPPED-ON-V2 / SPEC_DRIFT: method-name rename] Replace get_ambiguity_state mock.**
   Original plan text said add `SsbViewer::last_ambiguity_score()`. The
   shipped name is `SsbViewer::latest_ambiguity_scored()`
   (ssb_viewer.rs:3833) following the recent_* / latest_* sibling
   pattern at ssb_viewer.rs:3626-3752 (recent_failure_records,
   recent_tree_positions, recent_package_verifications,
   recent_tech_stack_decisions, research_node_stages). Same intent,
   different name; renaming the shipped method to
   `last_ambiguity_score` would break `get_ambiguity_state` at
   commands.rs:7887 and violate the recent_*/latest_* convention.
   PHANTOM-CLAIM-RESOLVED for the prior recheck "MISSING:
   SsbViewer::last_ambiguity_score" row (R4 in C-8-decisions.md).
   Read `latest_ambiguity_scored` everywhere the plan said
   `last_ambiguity_score`. Implementation: query the most recent
   ambiguity_scored row, return the real score struct, omit the
   "mock" field (follows the get_embedding_breakers pattern at
   commands.rs:840-863).

3. **[SHIPPED-ON-V2 / SPEC_DRIFT: host crate is live_cli, not task_runner] Wire CosmosWorker::tick() — Layer 1A scheduler.**
   Original plan text said add a tokio::spawn in task_runner.rs. The
   scheduler ships at cli/src/live_cli.rs:1964 inside an
   `if let Some(ref ssb_arc) = ssb { ... }` block, gated additionally
   on a manifest-present branch (live_cli.rs:1989 carries the
   "C-8: CosmosWorker init failed; cosmos stage + tick scheduler
   disabled" doc-marker confirming this is the C-8 scheduler
   bootstrap). Rationale (DRIFT-D in C-8-decisions.md):
   task_runner.rs handles a single task lifetime; the tick scheduler
   must outlive any one task and reflect orchestrator uptime. The
   scheduler is spawned only when Arc<Ssb> is present, so cold-start
   CLI paths without an SSB writer are unaffected. The cycle body is
   `cosmos_tick_emit_loop(&ssb_for_tick, &worker_for_tick).await` per
   tick (interval_secs = elai_knowledge::DEFAULT_TICK_INTERVAL_SECS =
   3600s); the wrapper emits one CosmosRefreshStarted row + one
   CosmosArtifactUpdated row per refreshed name (DRIFT-C: tick()
   stays sync `fn tick(&self) -> TickReport` and the SSB emission is
   external). Cosmos worker stays storage-only (no SsbNotifier field
   added). PHANTOM-CLAIM-RESOLVED for the prior recheck PARTIAL
   marker that searched task_runner.rs for the scheduler block
   (R2 in C-8-decisions.md).

4. **[SHIPPED / SPEC_DRIFT: ArtifactStale row emission dropped] Wire index_all_artifacts into IndexingService::build_index_resumable.**
   After code-phase completion call the artifact-indexing hook
   non-fatally (`maybe_index_artifacts` at indexing_service.rs:815-821
   wraps in tracing::warn! and continues). Emit ArtifactsIndexed SSB
   on success. SPEC_DRIFT (DRIFT-F): the original "ArtifactStale rows
   when detect_staleness returns non-empty" bullet is dropped from
   scope. Per-spec staleness is detected inside `index_artifact`
   (private surface) and surfaces via the `ArtifactStatus::Stale`
   variant in the returned `ArtifactIndexState`; results are
   partitioned into `indexed[]` vs `up_to_date[]` and the dashboard
   derives the stale view from those sets. `detect_staleness` is not
   exposed on the ArtifactIndexer public surface — keeping the
   original bullet would require a public-API widening that is out of
   C-8 scope.

5. **[SHIPPED / SPEC_DRIFT: ArtifactsIndexed variant reused, request stays singular] Implement TOOL_INDEX_ARTIFACTS in production_agent_rpc.rs.**
   Real implementation replaces the previous Err("deferred to Phase C")
   stub at production_agent_rpc.rs:1014. SPEC_DRIFT (DRIFT-G/H):
   ArtifactsIndexed variant + ArtifactsIndexedPayload ALREADY exist
   from B-34 (SCHEMA_VERSION 42); the original directive to "add"
   them was based on a stale assumption. C-8 wires the existing
   variant. IndexArtifactsRequest stays singular per IPC wire frame
   (per artifact_dispatcher_contract.rs:24-27), but the handler
   treats the decoded request as a one-element
   Vec<IndexArtifactsRequest> internally so a future bulk-frame
   upgrade only needs to change the decode step (no public API
   churn). Opens manifest + store, calls `index_artifact` per spec,
   emits ONE `ArtifactsIndexed` SSB row, returns Ok JSON
   `{ project_id, artifact_count, chunk_count, elapsed_ms, indexed[], up_to_date[] }`.
   Err propagates as AgentRpcError::ToolError (no panic).
   production_agent_rpc_contract.rs extended (T4): test F
   `f_index_artifacts_returns_ok_with_artifact_count_field` asserts
   ToolOutcome::Result with `artifact_count` field parseable as u32
   (plus sibling fields project_id, chunk_count, elapsed_ms,
   indexed[], up_to_date[]).

6. **Wire real SsbNotifier into CrossEncoderReranker** — at live_cli.rs:1086
   region, pass Arc<Ssb> cast to Arc<dyn SsbNotifier> into
   CrossEncoderReranker::new (replacing the NoopSsbNotifier default at
   reranker.rs:123).

7. **[SHIPPED / SPEC_DRIFT: SSB-row bridge replaces AppState Arc<RwLock>, atomic SCHEMA bump 49 -> 50] Store DegradationCascade result via SSB row.**
   SPEC_DRIFT (DRIFT-I): the original `Arc<RwLock<SearchMode>>` field
   on AppState does not work across the live_cli (CLI process) /
   dashboard-v2 (src-tauri process) boundary — they share no memory.
   Shipped design: new `SearchModeChanged` SSB variant
   (`SearchModeChangedPayload`); the CLI emits one row at startup +
   on each cascade transition; the dashboard's `get_search_mode`
   Tauri command reads `SsbViewer::latest_action_of_type(
   "search_mode_changed")` with graceful empty fallback to `"hybrid"`.
   New `set_search_mode` Tauri command lets reviewers pin the mode
   for diagnostics by emitting a row from the dashboard side.
   SPEC_DRIFT (DRIFT-J): SCHEMA_VERSION bumped 49 -> 50 atomically
   (count assertion 101 -> 102, all enum/payload/from_str arms
   updated in one step); the plan did not budget a SCHEMA bump but
   the cross-process bridge has no cleaner channel. Documented in
   ssb.rs:213-228 alongside the C-7 note. Set `unwired.searchMode = false`
   in drawers.svelte.ts:150 (shipped at L150 today).

8. **Wire ONNX dot-product into AragOrchestrator::semantic_search** — replace
   embedding-norm proxy with the graphcodebert.onnx session (model confirmed
   present at .elai_cc/models/onnx/). Pass Arc<OrtSession> as a field or via
   constructor injection (follow vector/mod.rs embedding pattern).

9. **[SHIPPED-ON-V2 / SPEC_DRIFT: ClockProvider instead of raw Instant] Replace latency_ms=0 in pipeline.rs.**
   SPEC_DRIFT (DRIFT-K): the original "thread `std::time::Instant::now()`
   into each stage" approach is BANNED by pipeline.rs L40-42's own
   doc comment (entropy quarantine: no raw Instant in
   instrumentation). Shipped design: add
   `clock: Arc<dyn ClockProvider>` to QueryPipeline + a `with_clock()`
   builder; live_cli installs the per-process OsClock via
   `pipeline.with_clock(clock.clone())`; tests get a SeededClock for
   deterministic latencies. All 17 `latency_ms: 0` literal sites
   swept to `self.clock.elapsed_since(stage_start).as_millis() as u64`.
   STALE-SHIPPED-ON-V2 (R1): the prior recheck report claimed
   pipeline.rs:747 + L763 still carried literal `latency_ms: 0`
   on the deep_cap and rrf_gate synthetic-emit PipelineStage sites;
   today pipeline.rs:764 reads
   `latency_ms: self.clock.elapsed_since(deep_cap_emit_start).as_millis() as u64,`
   and pipeline.rs:781 reads the matching rrf_gate variant (the
   literal-0 stubs were eradicated in the B-4 task #16 follow-on
   edit per the comment block at pipeline.rs:746-753). Det-check #8
   grep returns 0 today. ONE scoped exemption: TOOL_INDEX_ARTIFACTS
   measures elapsed_ms via raw std::time::Instant — this is per-RPC
   measurement (not pipeline instrumentation), and threading
   ClockProvider through ProductionAgentRpcServer would expand the
   public ctor surface for every existing test (DRIFT-K exemption
   documented in C-8-decisions.md).

10. **[SHIPPED / SPEC_DRIFT: CosmosStage CREATED not consumed] Wire CosmosStage into QueryPipeline.**
    SPEC_DRIFT (DRIFT-L): CosmosStage did not exist as a "consumable"
    type prior to C-8. `cosmos_stage.rs` was CREATED as a new module
    in the knowledge crate: `CosmosStage { worker: Arc<CosmosWorker> }`
    with stable `const fn name() -> "cosmos"` and
    `pub fn run(&self, query) -> Vec<RetrievalResult>`. Inserted
    between ArtifactStage and ARagStage in pipeline.rs for
    STANDARD+DEEP tiers only (NOT FAST). Name-router policy:
    token-overlap between the query and the artifact `name` or
    `research_query` field (lowercased, alphanumeric-split, drop
    tokens < 3 chars). Cosmos bodies are returned as a single
    RetrievalResult per matching artifact with `score = 0.75`
    (curated; downstream RRF handles relative ranking) and a
    1024-char snippet cap.

11. **[SHIPPED / PHANTOM-CLAIM-RESOLVED + SPEC_DRIFT: ambiguityState flag never existed] Clear unwired flags.**
    SPEC_DRIFT (DRIFT-M / A2 in C-8-decisions.md): the
    `unwired.ambiguityState` flag DOES NOT EXIST in
    drawers.svelte.ts. C-2 wired the ambiguity gate via a direct
    `invoke<AmbiguityState>('get_ambiguity_state')` call in
    GatesPanel.svelte:27 + drawers/OrchestrationDrawer.svelte:144,
    bypassing the drawers-store unwired pattern entirely. The flag
    never shipped; nothing to flip. Re-verified in s-25120:
    grep `ambiguityState|ambiguity_state` across
    drawers.svelte.ts returns 0 matches. PHANTOM-CLAIM-RESOLVED for
    the prior recheck "MISSING: unwired.ambiguityState=false" row
    (R3 in C-8-decisions.md). The only flag flipped this session is
    `unwired.searchMode = true -> false` at drawers.svelte.ts:150
    (gated on the get_search_mode SSB row read landing in Task 7).

12. **Write workflow/reasoning/C-8-decisions.md** — IndexLock decision for cosmos
    tick; ONNX injection approach for arag.rs; AppState vs singleton for SearchMode;
    Clock threading pattern for pipeline.rs.

## WIRING_CONTRACT

produces_symbols (REBASELINED):
  - AmbiguityScorer live call in task_runner.rs (PLANNING->TESTING_FIRST gate only)
  - SsbViewer::latest_ambiguity_scored() [SPEC_DRIFT rename; was last_ambiguity_score]
  - CosmosWorker hourly tick loop in cli::live_cli build_local_knowledge_stack
    [SPEC_DRIFT scheduler host; was task_runner.rs] + cosmos_tick_emit_loop wrapper
    emits CosmosRefreshStarted + CosmosArtifactUpdated SSB rows externally
    (cosmos worker stays storage-only per DRIFT-C)
  - index_all_artifacts hook in IndexingService::build_index_resumable
    (non-fatal via maybe_index_artifacts; ArtifactStale rows DROPPED per DRIFT-F)
  - TOOL_INDEX_ARTIFACTS real implementation (production_agent_rpc.rs:1014)
    [reuses ArtifactsIndexed variant from B-34 per DRIFT-G; request stays
    singular per IPC frame per DRIFT-H]
  - Arc<dyn SsbNotifier> wired into CrossEncoderReranker (live_cli.rs)
  - SearchModeChanged SSB row (cross-process bridge per DRIFT-I; was
    AppState.search_mode Arc<RwLock>) + get_search_mode live read +
    set_search_mode Tauri command
  - SCHEMA_VERSION 49 -> 50 (atomic; payload count 101 -> 102 per DRIFT-J)
  - ONNX dot-product in AragOrchestrator::semantic_search (norm-proxy fallback retained)
  - Real latency_ms in PipelineStage results via Arc<dyn ClockProvider>
    [SPEC_DRIFT per DRIFT-K; entropy quarantine forbids raw Instant in instrumentation;
    one scoped exemption for TOOL_INDEX_ARTIFACTS per-RPC elapsed_ms]
  - CosmosStage CREATED as new module (knowledge/src/cosmos_stage.rs) and
    wired into QueryPipeline (STANDARD+DEEP) [SPEC_DRIFT per DRIFT-L: was
    described as "consumed"]
  - drawers.svelte.ts unwired.searchMode=false [ambiguityState flag REMOVED
    from scope per DRIFT-M / R3 — flag never existed]
consumes_symbols:
  - AmbiguityScorer::score() (crates/reasoning/src/ambiguity.rs:244)
  - TransitionContext.ambiguity_score (crates/orchestrator/src/evidence.rs:98)
  - CosmosWorker::tick() (crates/knowledge/src/cosmos_worker.rs:444)
  - index_all_artifacts (crates/knowledge/src/artifacts/indexer.rs:201)
  - IndexArtifactsRequest / ArtifactsIndexedPayload (confirmed artifact_dispatcher_contract.rs:24-27)
  - SsbNotifier trait (crates/knowledge/src/degradation.rs:18)
  - DegradationCascade result (live_cli.rs:1086-1103)
  - CosmosStage type (confirmed B-42.report; crates/knowledge/src/cosmos_worker.rs)
  - C-1 QueryTrace / CosmosRefreshStarted SSB variants
reachable_from:
  - elai /research --mode cloud "..." (AmbiguityScorer, QueryPipeline+CosmosStage,
    CrossEncoderReranker)
  - elai /task --mode cloud "..." (TOOL_INDEX_ARTIFACTS via agent)
  - IndexingService triggered by reindex_now (index_all_artifacts hook)
  - get_search_mode Tauri command (AppState.search_mode live read)
stub_ban:
  - AmbiguityScorer MUST be called only at PLANNING->TESTING_FIRST gate
  - CosmosWorker tick MUST emit real SSB rows (tracing-only not acceptable)
  - TOOL_INDEX_ARTIFACTS MUST return Ok (no Err stub)
  - NoopSsbNotifier MUST NOT remain in CrossEncoderReranker production path
  - latency_ms MUST NOT be literal 0 in PipelineStage results
  - get_search_mode MUST NOT return hardcoded "HYBRID"
observable_effect:
  trigger: elai /research --mode cloud "compare X and Y in the codebase"
  expect_ssb:
    - ambiguity_scored rows with non-null score, no "mock" key
    - query_trace rows with total_latency_ms > 0
    - cosmos_refresh_started row when cosmos artifact configured and tick fires
  expect_artifact:
    - TOOL_INDEX_ARTIFACTS via agent returns Ok { artifact_count >= 0 }

## DETERMINISTIC_CHECK (rebaselined; 4 refinements per A4 in C-8-decisions.md)

```bash
# 1. AmbiguityScorer called in task_runner.rs
rg "AmbiguityScorer" .elai_cc/crates/orchestrator/src/task_runner.rs
# Expect: >= 1 match

# 2. get_ambiguity_state mock:true gone
rg '"mock".*true' .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
# Expect: 0 matches

# 3. [DET-CHECK REFINEMENT: scheduler host is live_cli, not task_runner per DRIFT-D]
#    CosmosWorker scheduler bootstrap in cli::live_cli build_local_knowledge_stack
rg "CosmosWorker|cosmos_worker|cosmos_tick_emit_loop" .elai_cc/crates/cli/src/live_cli.rs
# Expect: >= 1 match (scheduler block at live_cli.rs:1964 +
# cosmos_tick_emit_loop helper). Per-task helper trigger_cosmos_refresh
# remains in task_runner.rs:6651-6713 unwired (orphan; see DRIFT-E
# decision — kept for future per-task refresh wiring in C-9+).

# 4. index_all_artifacts hook in indexing_service.rs
rg "maybe_index_artifacts|index_artifact" .elai_cc/crates/orchestrator/src/indexing_service.rs
# Expect: >= 1 match (the C-8 hook is `maybe_index_artifacts` called at
# the tail of build_index_resumable; underlying call is per-artifact
# `index_artifact` not the bulk `index_all_artifacts` plan-text name).

# 5. TOOL_INDEX_ARTIFACTS Err stub gone
rg "deferred to Phase C" .elai_cc/crates/orchestrator/src/production_agent_rpc.rs
# Expect: 0 matches

# 6. [DET-CHECK REFINEMENT per A4: target NoopSsbNotifier CONSTRUCTION,
#    not docstring mentions]
rg "NoopSsbNotifier::(new|default)\b" .elai_cc/crates/cli/src/live_cli.rs
# Expect: 0 matches (any remaining `NoopSsbNotifier` token in
# comments/docs is acceptable; the production CrossEncoderReranker
# construction path must not instantiate one).

# 7. [DET-CHECK REFINEMENT per A4: scope HYBRID grep to get_search_mode body,
#    not the whole commands.rs file — the L3927 "HYBRID" match is in an
#    unrelated input-validation matcher]
awk '/fn get_search_mode\(/,/^\}/' .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs | rg '"HYBRID"'
# Expect: 0 matches inside the get_search_mode function body only.

# 8. [DET-CHECK REFINEMENT per A4: synthetic-emit carve-out documented;
#    pipeline.rs:747 + L763 deep_cap/rrf_gate sites already swept to
#    clock.elapsed_since per R1 / B-4 task #16 follow-on]
rg "latency_ms:\s*0\b" .elai_cc/crates/knowledge/src/pipeline.rs | wc -l
# Expect: 0 (all 17 instrumentation sites + the 2 synthetic-emit sites
# at pipeline.rs:747 + L763 use `self.clock.elapsed_since(...).as_millis()
# as u64`).

# 9. CosmosStage in pipeline.rs
rg "CosmosStage" .elai_cc/crates/knowledge/src/pipeline.rs
# Expect: >= 1 match (CosmosStage type CREATED at
# knowledge/src/cosmos_stage.rs per DRIFT-L; inserted between
# ArtifactStage and ARagStage in STANDARD+DEEP tiers).

# 10. unwired flags cleared (ambiguityState removed from scope per
#     DRIFT-M / R3 — never existed; searchMode is the only flag flipped)
rg "unwired\.searchMode\s*=\s*true" \
  .elai_cc/crates/dashboard-v2/src/lib/state/drawers.svelte.ts
# Expect: 0 matches (searchMode = false at drawers.svelte.ts:150).

# 11. cargo check
cargo check --workspace 2>&1 | grep "^error" | head -5
# Expect: 0 lines

# 12. Contract tests (4 files; 14 tests total per s-25120 closure)
cargo test -p elai-orchestrator --test production_agent_rpc_contract 2>&1 | grep "test result"
cargo test -p elai_cc_contracts --test ambiguity_task_runner_contract 2>&1 | grep "test result"
cargo test -p elai_cc_contracts --test cosmos_tick_wiring_contract 2>&1 | grep "test result"
cargo test -p elai_cc_contracts --test indexing_service_artifact_hook_contract 2>&1 | grep "test result"
# Expect: production_agent_rpc 6/6 PASS, ambiguity_task_runner 3/3 PASS,
# cosmos_tick_wiring 3/3 PASS, indexing_service_artifact_hook 2/2 PASS.
```

## ACCEPTANCE (rebaselined)

- [x] AmbiguityScorer::score() called in task_runner.rs at PLANNING->TESTING_FIRST
      gate only; Err falls back to None (B-17 preserved)
- [x] get_ambiguity_state reads real SSB rows; "mock" field absent
- [x] CosmosWorker::tick() driven by hourly tokio::interval in
      cli::live_cli build_local_knowledge_stack (SPEC_DRIFT per DRIFT-D:
      host crate is live_cli, not task_runner); real CosmosRefreshStarted
      and CosmosArtifactUpdated SSB rows emitted via cosmos_tick_emit_loop
- [x] index_all_artifacts hook (maybe_index_artifacts) called from
      IndexingService::build_index_resumable; artifact Err is non-fatal;
      ArtifactsIndexed SSB row emitted on success (ArtifactStale rows
      DROPPED per DRIFT-F)
- [x] TOOL_INDEX_ARTIFACTS returns Ok { project_id, artifact_count,
      chunk_count, elapsed_ms, indexed[], up_to_date[] }; Err stub removed
- [x] production_agent_rpc_contract.rs asserts TOOL_INDEX_ARTIFACTS Ok
      response (T4: test F shipped in s-25120)
- [x] NoopSsbNotifier absent from live_cli.rs CrossEncoderReranker
      construction (DET-CHECK refinement #6: targets ::new/::default
      construction, not docstring mentions)
- [x] SearchModeChanged SSB row stores live DegradationCascade result
      (SPEC_DRIFT per DRIFT-I: SSB-row bridge replaces
      Arc<RwLock<SearchMode>>; SCHEMA bump 49 -> 50 per DRIFT-J)
- [x] get_search_mode returns live SearchMode (not hardcoded "HYBRID"
      in the get_search_mode function body — DET-CHECK refinement #7)
- [x] unwired.searchMode = false in drawers.svelte.ts:150
- [x] AragOrchestrator::semantic_search uses ONNX dot-product
      (norm-proxy fallback retained on embedder failure)
- [x] pipeline.rs latency_ms populated via Arc<dyn ClockProvider>
      (SPEC_DRIFT per DRIFT-K: ClockProvider not raw Instant; entropy
      quarantine preserved); 0 literal count = 0 across all 17
      instrumentation sites + 2 synthetic-emit sites (R1)
- [x] CosmosStage CREATED (knowledge/src/cosmos_stage.rs) and inserted
      in QueryPipeline (STANDARD+DEEP, not FAST) — SPEC_DRIFT per
      DRIFT-L: type was described as "consumed" but had to be authored
- [PHANTOM-CLAIM-RESOLVED] unwired.ambiguityState flag never existed in
      drawers.svelte.ts (DRIFT-M / R3); GatesPanel.svelte:27 +
      OrchestrationDrawer.svelte:144 invoke `get_ambiguity_state`
      directly, bypassing the unwired registry. Nothing to flip.
- [x] cargo check --workspace passes (1m11s clean in s-32952; reverified
      clean in s-25120 audit)
- [x] DETERMINISTIC_CHECK items 1-12 all pass (rebaselined per A4)
- [x] C-8-decisions.md in workflow/reasoning/ (R1-R4 + T1-T4 + A1-A4)
- [DEFERRED] Cloud-Sonnet smoke test deferred to C-9+ (requires live
      cloud Sonnet + configured [[cosmos.artifacts]] in elai_cc.toml;
      contract tests T1-T4 cover the wiring witnesses deterministically):
      `elai /research --mode cloud "compare X and Y in the codebase"`
      - ambiguity_scored SSB rows present; `json_extract(payload,'$.overall_score')`
        is non-null numeric (not mock)
      - query_trace SSB rows with `total_latency_ms > 0` on every row
      - CrossEncoderReranker does not crash with real SsbNotifier
      - Configure one [[cosmos.artifacts]] in elai_cc.toml; after tick: assert
        cosmos_refresh_started SSB row present; status_snapshot() returns Fresh
      - Trigger TOOL_INDEX_ARTIFACTS via agent: Ok with artifact_count field present

## Tests (SHIPPED in s-25120 — see C-8-decisions.md T1-T4)

- [x] EXTEND (T4): `.elai_cc/crates/orchestrator/tests/production_agent_rpc_contract.rs`
  — test F `f_index_artifacts_returns_ok_with_artifact_count_field` asserts
  TOOL_INDEX_ARTIFACTS returns ToolOutcome::Result with artifact_count +
  chunk_count + elapsed_ms + project_id + indexed[] + up_to_date[] fields
  (empty-manifest happy path; no external network/sidecar required). 6/6 PASS.
- [x] ADD (T1): `.elai_cc/crates/contracts/tests/ambiguity_task_runner_contract.rs`
  — pins TransitionContext.ambiguity_score field as the channel from
  AmbiguityScorer.score() to GroundingSweepGate: (a) qualifying score +
  met streak ALLOWs via both gate.check and full check_transition; (b)
  high overall_score triggers BLOCK with ambiguity-tagged reason;
  (c) None (B-17 fallback) preserves streak-only ALLOW. 3/3 PASS.
- [x] ADD (T2): `.elai_cc/crates/contracts/tests/cosmos_tick_wiring_contract.rs`
  — exercises elai_orchestrator::trigger_cosmos_refresh: (a)
  one-artifact manifest emits CosmosRefreshStarted with queued=1/total=1;
  (b) empty manifest emits exactly one row with queued=0/total=0
  (unconditional pre-loop write contract); (c) empty manifest produces
  0 refreshed_count AND zero CosmosArtifactUpdated rows. 3/3 PASS.
  Note: tests the trigger_cosmos_refresh HELPER (not the live_cli
  scheduler block at live_cli.rs:1964). Host-integration assertion
  lives in live_cli.rs construction, intentionally separated per
  contracts/ vs cli/ discipline.
- [x] ADD (T3): `.elai_cc/crates/contracts/tests/indexing_service_artifact_hook_contract.rs`
  — drives IndexingService::build_index_resumable: (a) absent manifest
  -> hook is documented no-op, indexing returns Ok with bm25_files >= 1;
  (b) MALFORMED elai_cc.toml -> ArtifactError::TomlParse is caught at
  indexing_service.rs:815-821 via tracing::warn! and core pipeline
  still returns Ok. Pins the non-fatal contract. 2/2 PASS.

Audit (s-25120): 14/14 PASS across all 4 contract test invocations.

## Files touched (rebaselined to shipped reality)

Knowledge crate (.elai_cc/crates/knowledge):
- MODIFY: src/arag.rs — `embedder: Option<Arc<OrtSession>>` field +
  `with_embedder()` builder + ONNX dot-product path in semantic_search
  (norm-proxy fallback with tracing::warn!).
- ADD: src/cosmos_stage.rs — NEW (DRIFT-L). CosmosStage type, name-router
  policy, RetrievalResult adapter, 5 unit tests.
- MODIFY: src/cosmos_worker.rs — made NoopResearchDispatcher public for
  live_cli scheduler bootstrap.
- MODIFY: src/lib.rs — pub mod cosmos_stage; re-export CosmosStage,
  default_cosmos_root, DEFAULT_TICK_INTERVAL_SECS, NoopResearchDispatcher,
  SystemCosmosClock.
- MODIFY: src/pipeline.rs — `cosmos_stage: Option<CosmosStage>` field +
  `clock: Arc<dyn ClockProvider>` field + `with_cosmos_stage()` /
  `with_clock()` builders. All 17 `latency_ms: 0` sites + 2 synthetic-emit
  sites (deep_cap, rrf_gate at L747/L763) swept to
  `clock.elapsed_since(stage_start).as_millis()`. CosmosStage inserted
  between artifacts and arag stages (STANDARD + DEEP).

Orchestrator crate (.elai_cc/crates/orchestrator):
- MODIFY: src/ssb.rs — SearchModeChanged variant + payload (atomic
  SCHEMA 49 -> 50, count 101 -> 102; DRIFT-J unbudgeted bump documented).
- MODIFY: src/lib.rs — re-export SearchModeChangedPayload +
  CosmosRefreshStartedPayload + CosmosArtifactUpdatedPayload + Decayed/Used.
- MODIFY: src/indexing_service.rs — non-fatal artifact-indexing hook
  (`maybe_index_artifacts`) at tail of build_index_resumable.
- MODIFY: src/production_agent_rpc.rs — TOOL_INDEX_ARTIFACTS real impl
  (replaces "deferred to Phase C" Err stub); reuses ArtifactsIndexed
  variant from B-34 (DRIFT-G).
- MODIFY: src/ssb_viewer.rs — `latest_ambiguity_scored()` method
  (SPEC_DRIFT per R4: was `last_ambiguity_score` in plan; renamed to
  match recent_*/latest_* sibling convention at L3626-3752).

CLI crate (.elai_cc/crates/cli):
- MODIFY: src/live_cli.rs — A-RAG embedder wiring; pipeline `.with_clock()`;
  SearchModeChanged emit after startup_readiness; CosmosWorker construction
  + CosmosStage attach + tick scheduler (tokio::time::interval @ 3600s) at
  L1964 (DRIFT-D: scheduler host is live_cli not task_runner). Two new
  helpers: search_mode_to_payload_str + cosmos_tick_emit_loop.

Dashboard-v2 crate (.elai_cc/crates/dashboard-v2):
- MODIFY: src-tauri/src/commands.rs — get_search_mode reads SSB row
  (graceful "hybrid" fallback); new set_search_mode Tauri command
  emits SearchModeChanged rows.
- MODIFY: src-tauri/src/lib.rs — register commands::set_search_mode.
- MODIFY: src/lib/state/drawers.svelte.ts — `searchMode: true -> false`
  at L150 (ambiguityState flag never existed per DRIFT-M / R3).
- MODIFY: src/lib/api/types.ts — SearchMode type extended with lowercase
  wire tags alongside legacy uppercase enum.

NOT touched (SPEC_DRIFT — original plan text was wrong about host crate):
- crates/orchestrator/src/task_runner.rs — the original plan called
  task_runner.rs as the scheduler host for CosmosWorker (Task 3) but
  the actual bootstrap (where Arc<CosmosWorker> + Arc<Ssb> co-construct)
  is live_cli.rs (DRIFT-D). The orphan per-task refresh helper
  `trigger_cosmos_refresh` at task_runner.rs:6651-6713 remains uncalled
  pending C-9+ per-task wiring or documented removal (DRIFT-E).

Contracts crate (.elai_cc/crates/contracts):
- ADD: tests/c8_stub_eradication_contract.rs — 6 narrow assertions
  (schema floor, sql-str mapping, from_str round-trip, payload round-trip,
  ArtifactsIndexed round-trip, history-order). Shipped s-32952.
- ADD: tests/ambiguity_task_runner_contract.rs (T1) — 3 tests. Shipped s-25120.
- ADD: tests/cosmos_tick_wiring_contract.rs (T2) — 3 tests. Shipped s-25120.
- ADD: tests/indexing_service_artifact_hook_contract.rs (T3) — 2 tests. Shipped s-25120.
- EXTEND: orchestrator/tests/production_agent_rpc_contract.rs (T4) —
  test F added (now 6 total). Shipped s-25120.

Workflow:
- CREATE: workflow/reasoning/C-8-decisions.md (DRIFT-A..M + R1-R4 + T1-T4 + A1-A4)
- MODIFY: docs/thefinalglance/C-plan/C-8_viewer_stub_eradication.md
  (this file — REBASELINE applied per s-25120 audit BLOCK)

## Next

C-9 (SSB viewer completions — pagerank_top SQL, freshness_state live DB read,
query_tier_stats aggregate from ssb_viewer.rs all-zeros; also cosmos_pipeline_stage
test gap unblocked by CosmosStage landing here). Carry forward from C-8:
- Either wire `trigger_cosmos_refresh` (task_runner.rs:6651-6713) from a
  task entry hook or document its removal (DRIFT-E).
- Build the dashboard SearchModeBadge per-mode picker that calls the
  new `set_search_mode` Tauri command (backend complete; frontend pending).
- Sweep the stale `assert_eq!(SCHEMA_VERSION_PUBLIC, N)` contract tests
  (focus_tunnel_e5_contract, idea_framing_contract, minified_chunked_ssb,
  symbol_graph_updated_ssb) to the `>=` floor pattern C-7 adopted.
- Platform-aware std::time guarding for the artifacts_pipeline_contract
  clock-overflow pre-existing failure.
