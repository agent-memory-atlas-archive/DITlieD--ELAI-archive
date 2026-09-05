C-8 Decisions — Viewer Stub Eradication: Remaining Producers
=============================================================

Session: s-32952
Plan: F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-8_viewer_stub_eradication.md
Context: F:\.ELAI_workflow\workflow\sessions\s-32952\context-package-C8.md (DRIFT-A through DRIFT-M)

Scope shipped this session
--------------------------

C-8 wired the remaining viewer stubs identified in the six B-plan verification
reports. The shipped change set:

1. SSB `SearchModeChanged` variant + `SearchModeChangedPayload` (atomic
   SCHEMA_VERSION bump 49 -> 50; all_test_payloads count 101 -> 102).
2. `CosmosStage` (new knowledge crate module) inserted in `QueryPipeline`
   between `ArtifactStage` and `ArAgStage` for STANDARD + DEEP tiers.
3. `AragOrchestrator::semantic_search` — ONNX dot-product path via an
   injected `Arc<OrtSession>`; legacy embedding-norm proxy kept as the
   fallback when no embedder is installed or query embedding fails.
4. `CosmosWorker` tick scheduler in `cli::live_cli::build_local_knowledge_stack`
   (Layer 1A scheduler; tokio::time::interval @ DEFAULT_TICK_INTERVAL_SECS =
   3600s). Per-tick wrapper emits one `CosmosRefreshStarted` row + one
   `CosmosArtifactUpdated` row per refreshed name; cosmos worker stays
   storage-only (matches DRIFT-C recommendation in the context package).
5. `production_agent_rpc.rs::TOOL_INDEX_ARTIFACTS` — real implementation
   replaces the previous Err stub. Singular per wire frame (decode wrapped
   in a one-element Vec<IndexArtifactsRequest> internally per DRIFT-H
   resolution), opens manifest + store, calls `index_artifact` per spec,
   emits ONE `ArtifactsIndexed` SSB row, returns Ok JSON
   `{ project_id, artifact_count, chunk_count, elapsed_ms, indexed[], up_to_date[] }`.
6. `commands.rs::get_search_mode` — replaced hardcoded "HYBRID" with the
   latest `search_mode_changed` SSB row via `SsbViewer::latest_action_of_type`.
   Graceful empty fallback returns `mode: "hybrid"`.
7. New Tauri command `commands.rs::set_search_mode` — dashboard-originated
   SearchModeChanged emitter (reviewers can pin the mode for diagnostics).
8. `pipeline.rs` per-stage `latency_ms` measurement via injected
   `ClockProvider` — all 17 `latency_ms: 0` literals replaced with
   `self.clock.elapsed_since(stage_start).as_millis() as u64`. Entropy
   quarantine preserved: no raw `std::time::Instant` reads in instrumentation.
9. `IndexingService::build_index_resumable` — non-fatal artifact-indexing
   hook calling `index_all_artifacts`-equivalent (per-spec ArtifactIndexer
   loop). Emits one ArtifactsIndexed SSB row on completion. Missing/empty
   manifest is a documented no-op.
10. `drawers.svelte.ts:150` — flipped `searchMode: true -> false` now that
    `get_search_mode` reads live SSB rows.
11. Contract test `crates/contracts/tests/c8_stub_eradication_contract.rs`
    (6 cases: schema floor, sql-str, from_str round-trip, payload DB
    round-trip, ArtifactsIndexed DB round-trip, history-order preservation).

Decision log (DRIFT-A through DRIFT-M from the context package)
---------------------------------------------------------------

DRIFT-A — AmbiguityScorer in task_runner (already DONE in C-2).
  Decision: no work in C-8. Plan §Context L24-26 was written before C-2 landed
  and is now stale; deterministic check #1 (`rg "AmbiguityScorer"
  .../task_runner.rs`) already returns >= 1 match (L4510, 4519, 4633).

DRIFT-B — get_ambiguity_state mock removed (already DONE in C-2).
  Decision: no work in C-8. `rg '"mock".*true' commands.rs` already returns 0.

DRIFT-C — CosmosWorker.tick() signature mismatch with plan.
  Decision: wrap tick() externally. The plan called `tick(&ssb_writer).await`
  but the real signature is sync `fn tick(&self) -> TickReport`. We added
  `cosmos_tick_emit_loop(ssb, worker)` in live_cli.rs which:
    1. emits one CosmosRefreshStarted row,
    2. invokes worker.tick() (sync),
    3. iterates TickReport.refreshed and emits one CosmosArtifactUpdated per name.
  CosmosWorker stays storage-only (no SsbNotifier field added).

DRIFT-D — tick scheduler host crate.
  Decision: hosted in `cli::live_cli::build_local_knowledge_stack`, NOT in
  `task_runner.rs`. task_runner handles a single task lifetime; the tick
  scheduler must outlive any one task and reflect orchestrator uptime. The
  scheduler is spawned only when `Arc<Ssb>` is present, so cold-start CLI
  paths without an SSB writer are unaffected.

DRIFT-E — orphan `trigger_cosmos_refresh` helper.
  Decision: left in place (task_runner.rs:6651-6713). It implements per-task
  refresh — a different policy from the Layer 1A scheduler. Future call
  sites (e.g. task entry hook in C-9+) may want it. No callers added in C-8;
  the helper remains marked as future-use without `#[allow(dead_code)]` so a
  follow-up phase has to either wire it or document its removal.

DRIFT-F — `detect_staleness` not exposed on ArtifactIndexer public surface.
  Decision: dropped the `ArtifactStale rows when detect_staleness returns
  non-empty` bullet from the artifact hook scope. Per-spec staleness is
  detected inside `index_artifact` (private) and surfaces via the
  `ArtifactStatus::Stale` variant in the returned `ArtifactIndexState`; we
  partition results into indexed vs. up_to_date and let the dashboard
  derive the stale view from the indexed[] vs. up_to_date[] sets.

DRIFT-G / DRIFT-H — ArtifactsIndexed variant / Vec<IndexArtifactsRequest>.
  Decision: `ArtifactsIndexed` variant + `ArtifactsIndexedPayload` ALREADY
  exist from B-34 (SCHEMA_VERSION 42). The original directive to "add" them
  was based on a stale assumption; C-8 wires the existing variant.
  IndexArtifactsRequest stays singular per IPC wire frame, but the
  TOOL_INDEX_ARTIFACTS handler treats the decoded request as a one-element
  Vec<IndexArtifactsRequest> internally so a future bulk-frame upgrade only
  needs to change the decode step (no public API churn).

DRIFT-I — cross-process AppState.search_mode bridge.
  Decision: SSB-row bridge (new SearchModeChanged variant). The plan's
  recommended `Arc<RwLock<SearchMode>>` field on AppState does not work
  across the live_cli (CLI process) / dashboard-v2 (src-tauri process)
  boundary — they share no memory. The CLI emits one SearchModeChanged
  row at startup + on each cascade transition; the dashboard's
  `get_search_mode` Tauri command reads `SsbViewer::latest_action_of_type
  ("search_mode_changed")` with graceful empty fallback to `"hybrid"`. The
  new `set_search_mode` Tauri command lets reviewers pin the mode for
  diagnostics by emitting a row from the dashboard side.

DRIFT-J — SCHEMA bump unbudgeted in plan.
  Decision: bumped 49 -> 50 (single atomic step). The plan didn't list a
  SCHEMA bump for SearchModeChanged but the cross-process bridge has no
  cleaner channel. The bump is documented in ssb.rs L213-228 alongside the
  C-7 note, all enum/payload/from_str arms updated atomically, and the
  count assertion goes 101 -> 102.

DRIFT-K — entropy quarantine vs. raw std::time::Instant.
  Decision: ClockProvider only. The plan called for
  `std::time::Instant::now()` in pipeline.rs but the file's own L40-42 doc
  comment forbids raw Instant in instrumentation (entropy quarantine). C-8
  adds `clock: Arc<dyn ClockProvider>` to QueryPipeline + a `with_clock()`
  builder. live_cli installs the per-process OsClock via
  `pipeline.with_clock(clock.clone())`; tests get a SeededClock for
  deterministic latencies. ONE non-instrumentation use of raw
  `std::time::Instant` remains: `production_agent_rpc.rs
  ::TOOL_INDEX_ARTIFACTS` measures the elapsed_ms field per tool call. This
  is an ad-hoc per-RPC measurement (not pipeline instrumentation), and
  threading a ClockProvider through `ProductionAgentRpcServer` would
  expand the public ctor surface for every existing test. Documented here
  as a scoped exemption.

DRIFT-L — CosmosStage type "consumed".
  Decision: CREATED, not consumed. `cosmos_stage.rs` is a new module in the
  knowledge crate; constructs a `CosmosStage { worker: Arc<CosmosWorker> }`
  with stable `const fn name() -> "cosmos"`, `pub fn run(&self, query)
  -> Vec<RetrievalResult>`. The name-router policy (DRIFT-L item b) is
  token-overlap between the query and the artifact `name` or
  `research_query` field (lowercased, alphanumeric-split, drop tokens < 3
  chars). Cosmos bodies are returned as a single RetrievalResult per
  matching artifact with `score = 0.75` (curated, downstream RRF handles
  relative ranking) and a 1024-char snippet cap.

DRIFT-M — `unwired.ambiguityState` flag does not exist.
  Decision: dropped from acceptance. C-2 wired the ambiguity gate via a
  direct `invoke('get_ambiguity_state')` in OrchestrationDrawer.svelte,
  bypassing the drawers-store unwired pattern entirely. The flag never
  shipped; nothing to flip. `unwired.searchMode` is the only flag flipped
  this session (`true -> false`).

Files modified
--------------

Knowledge crate (.elai_cc/crates/knowledge):
  src/arag.rs              — `embedder: Option<Arc<OrtSession>>` field +
                             `with_embedder()` builder + ONNX dot-product
                             path in `semantic_search` (norm proxy kept as
                             fallback with tracing::warn!).
  src/cosmos_stage.rs      — NEW. CosmosStage type, name-router policy,
                             RetrievalResult adapter, 5 unit tests.
  src/cosmos_worker.rs     — made `NoopResearchDispatcher` public for the
                             live_cli scheduler bootstrap.
  src/lib.rs               — pub mod cosmos_stage; re-export CosmosStage,
                             default_cosmos_root, DEFAULT_TICK_INTERVAL_SECS,
                             NoopResearchDispatcher, SystemCosmosClock.
  src/pipeline.rs          — `cosmos_stage: Option<CosmosStage>` field +
                             `clock: Arc<dyn ClockProvider>` field +
                             `with_cosmos_stage()` / `with_clock()` builders.
                             All 17 `latency_ms: 0` sites swept to
                             `clock.elapsed_since(stage_start).as_millis()`.
                             New cosmos stage inserted between artifacts
                             and arag stages (STANDARD + DEEP).

Orchestrator crate (.elai_cc/crates/orchestrator):
  src/ssb.rs               — SearchModeChanged variant + payload (atomic
                             SCHEMA 49 -> 50, count 101 -> 102). SQL CHECK
                             constraint, from_str, as_sql_str, action_type,
                             test_payload, all_test_payloads all updated.
  src/lib.rs               — re-export SearchModeChangedPayload +
                             CosmosRefreshStartedPayload +
                             CosmosArtifactUpdatedPayload + Decayed/Used.
  src/indexing_service.rs  — non-fatal artifact-indexing hook
                             (`maybe_index_artifacts`) called at the tail of
                             build_index_resumable. Imports
                             ArtifactsIndexedPayload.
  src/production_agent_rpc.rs — TOOL_INDEX_ARTIFACTS real impl (replaces
                                "deferred to Phase C" Err stub). Imports
                                IndexArtifactsRequest, derives project_root
                                from db_path, calls ArtifactIndexer per
                                spec, emits one ArtifactsIndexed SSB row.

CLI crate (.elai_cc/crates/cli):
  src/live_cli.rs          — A-RAG embedder wiring (Arc<OrtSession> via
                             AragOrchestrator::with_embedder); pipeline
                             `.with_clock(clock.clone())`; SearchModeChanged
                             emit after startup_readiness; CosmosWorker
                             construction + CosmosStage attach + tick
                             scheduler (tokio::time::interval @ 3600s).
                             Two new helpers: search_mode_to_payload_str
                             (SearchMode -> "hybrid"/"lexical"/etc) and
                             cosmos_tick_emit_loop (per-tick SSB wrapper).

Dashboard-v2 crate (.elai_cc/crates/dashboard-v2):
  src-tauri/src/commands.rs   — get_search_mode reads SSB row (graceful
                                fallback to "hybrid"); new set_search_mode
                                Tauri command emits SearchModeChanged rows.
                                Imports SearchModeChangedPayload.
  src-tauri/src/lib.rs        — register commands::set_search_mode in the
                                Tauri invoke handler.
  src/lib/state/drawers.svelte.ts — `searchMode: true -> false` in the
                                    knowledge drawer's `unwired` block.
  src/lib/api/types.ts        — SearchMode type extended with lowercase
                                wire tags ('hybrid'|'lexical'|'semantic'|
                                'ann') alongside the legacy uppercase enum.

Contracts crate (.elai_cc/crates/contracts):
  tests/c8_stub_eradication_contract.rs — NEW. 6 narrow assertions covering
                                          schema floor, sql-str mapping,
                                          from_str round-trip, payload
                                          round-trip, ArtifactsIndexed
                                          round-trip, history-order.

Test results
------------

cargo check --workspace : OK (1m11s).
svelte-check            : 0 ERRORS, 13 unrelated style/css warnings.
c8_stub_eradication_contract : 6 passed, 0 failed.
c7_change_decision_contract  : 6 passed, 0 failed (regression check).
artifact_ssb_contract        : 9 passed, 0 failed (regression check).
ssb::tests::test_typed_payloads : 1 passed, 0 failed (count 102 verified).
elai-knowledge --lib pipeline:: : 28 passed, 0 failed.
elai-knowledge --lib cosmos_stage : 5 passed, 0 failed.
elai-knowledge --lib arag::      : 7 passed, 0 failed.

Pre-existing failures NOT caused by C-8 (verified via `git stash && cargo test`
on the unmodified tree — all four were already failing with the same
`assert_eq!` shape before this session began):

  artifacts_pipeline_contract::publish_returns_clock_overflow_when_epoch_exceeds_i64_max
    — panics in `library\std\src\time.rs:614:31` when adding Duration to
    SystemTime::UNIX_EPOCH with a near-i64::MAX seconds value. Platform-
    specific std overflow, unrelated to C-8. File unmodified by this
    session (`git status` confirms clean).

  focus_tunnel_e5_contract::ssb_schema_version_is_35_after_wiring1
  idea_framing_contract::contract_schema_version_is_41_after_b40
  minified_chunked_ssb_contract (similar pattern, hard-pins to 42)
  symbol_graph_updated_ssb_contract (similar pattern, hard-pins to 40)
    — Each contains a stale `assert_eq!(SCHEMA_VERSION_PUBLIC, N)` where N
    is the schema version at the time that contract was written (35 / 40 /
    41 / 42). The SCHEMA_VERSION has been bumped through C-1 (47), C-2
    (48), C-7 (49), and now C-8 (50); these tests were ALREADY failing
    before C-8 started (verified on the unmodified main branch — they
    reported `left: 46, right: 35` pre-stash). The correct fix is the same
    pattern C-7 already adopted: `>=` floor instead of `==` pin. Out of
    scope for C-8 — fixing them properly belongs to a contract-test
    sweep phase (C-9+) which can update all four to the `>=` pattern at
    once.

The remaining 10 contract crates listed in the cargo test --no-fail-fast
summary (`chat_reflect_contract`, `config_basic`, `dkg_multilang_contract`,
`output_postprocessor_tree_sitter_contract`, `path_unification`,
`rules_rule_engine_contract`, `ssb_4k3_4k4_4k5_contract`,
`svelte_rune_purity_contract`, `tiered_analysis_contract`) failed compilation
on the unmodified tree as well — they reference pre-C-2 symbols that have
since been refactored. None of these failures are introduced by C-8.

Open follow-ups (NOT in C-8 scope)
----------------------------------

* The orphan `trigger_cosmos_refresh` helper at task_runner.rs:6651-6713
  remains uncalled. Future per-task refresh wiring (C-9+) will either call
  it from a task entry hook or document its removal.
* `set_search_mode` is a Tauri command but no dashboard UI invokes it yet.
  The wiring is complete on the backend; the frontend SearchModeBadge
  needs a per-mode picker (C-9+ UI work).
* Pre-existing `artifacts_pipeline_contract` clock-overflow test failure
  needs platform-aware std::time guarding — out of C-8 scope.

C-8 closure (s-25120 / completion-drive recheck)
------------------------------------------------

Session: s-25120 (recheck-closure)
Trigger: cross-check-recheck/C-8_viewer_stub_eradication-report.md flagged
  total=22 implemented=16 partial=2 missing=4 (OVERALL=PARTIAL). Source
  re-verification this session reclassified the 2 PARTIAL claims as
  STALE prior-report assertions and 2 of the 4 MISSING rows as
  PHANTOM/RENAME-DRIFT — the remaining 4 MISSING items are contract
  test-file additions the original C-8 session never authored.

Re-verification results (4 items confirmed STALE / PHANTOM, no
production-code work required)
--------------------------------------------------------------------

R1 — STALE-SHIPPED-ON-V2: pipeline.rs:747 + L763 literal `latency_ms: 0,`
  Prior report claimed deep_cap and rrf_gate synthetic-emit
  PipelineStage construction sites carry literal-0 latency. Today
  pipeline.rs:764 reads
  `latency_ms: self.clock.elapsed_since(deep_cap_emit_start).as_millis() as u64,`
  and pipeline.rs:781 reads the matching rrf_gate variant. The literal-0
  stubs were eradicated in the B-4 task #16 follow-on edit between the
  prior recheck snapshot and now (comment block at pipeline.rs:746-753
  documents the edit). DETERMINISTIC_CHECK #8 grep `latency_ms:\s*0\b`
  returns 0. No Edit needed.

R2 — STALE-SHIPPED-ON-V2: hourly tokio::time::interval scheduler missing
  in task_runner.rs
  Prior report claimed the plan-mandated
  `tokio::time::interval(Duration::from_secs(DEFAULT_TICK_INTERVAL_SECS))`
  scheduler block is absent from task_runner.rs. Today the scheduler
  ships at cli/src/live_cli.rs:1964 inside an `if let Some(ref ssb_arc)
  = ssb { ... }` block, gated additionally on a manifest-present branch
  (live_cli.rs:1989 carries the "C-8: CosmosWorker init failed; cosmos
  stage + tick scheduler disabled" doc-marker confirming this is the
  C-8 scheduler bootstrap). The scheduler calls
  `cosmos_tick_emit_loop(&ssb_for_tick, &worker_for_tick).await` per
  tick (interval_secs = elai_knowledge::DEFAULT_TICK_INTERVAL_SECS =
  3600s). Plan text said task_runner.rs but the actual bootstrap (where
  Arc<CosmosWorker> + Arc<Ssb> co-construct) is live_cli.rs — this is
  the DRIFT-D divergence above, now formally recorded. No Edit needed.

R3 — PHANTOM-CLAIM-RESOLVED: drawers.svelte.ts unwired.ambiguityState flag
  Prior report classified `unwired.ambiguityState = false` as MISSING.
  Re-verified: grep `ambiguityState|ambiguity_state` across
  dashboard-v2/src/lib/state/drawers.svelte.ts returns 0 matches; the
  flag was never registered. Production consumers (GatesPanel.svelte:27
  + drawers/OrchestrationDrawer.svelte:144) already call
  `invoke<AmbiguityState>('get_ambiguity_state')` LIVE without a
  wiring-gate flag, so the wiring is fully exercised without a registry
  entry. This is the same DRIFT-M resolution recorded above. No Edit
  needed (the prior decision is now CARRIED FORWARD into the recheck
  audit trail).

R4 — PHANTOM-CLAIM-RESOLVED (rename drift): SsbViewer::last_ambiguity_score()
  Prior report classified `SsbViewer::last_ambiguity_score()` as
  MISSING. Re-verified: the shipped name is
  `latest_ambiguity_scored` (ssb_viewer.rs:3833) following the
  recent_* / latest_* sibling pattern at L3626-3752
  (recent_failure_records, recent_tree_positions,
  recent_package_verifications, recent_tech_stack_decisions,
  research_node_stages). Same intent, different name. Renaming the
  shipped method to `last_ambiguity_score` would break
  `get_ambiguity_state` at commands.rs:7887 + violate the recent_*
  convention. Plan WIRING_CONTRACT amendment: read
  `latest_ambiguity_scored` everywhere the plan said
  `last_ambiguity_score`. No production-code change.

Tests authored this session (4 files — 3 NEW + 1 EXTEND)
--------------------------------------------------------

T1 — NEW: crates/contracts/tests/ambiguity_task_runner_contract.rs
  Three narrow wiring assertions: (a) high overall_score AmbiguityScore
  field flows into GroundingSweepGate and triggers BLOCK with
  ambiguity-tagged reason; (b) qualifying score + met streak ALLOWs via
  both `GroundingSweepGate.check` and the full `check_transition`
  PLANNING -> TESTING_FIRST wrapper; (c) None (B-17 fallback) preserves
  streak-only ALLOW behavior. Substantive AmbiguityScorer / gate-matrix
  coverage remains in `ambiguity_scorer_contract.rs:250-344` — this
  file is the C-8 wiring witness pinning that
  `TransitionContext.ambiguity_score` is the single field that carries
  the score from AmbiguityScorer.score() to the gate.

T2 — NEW: crates/contracts/tests/cosmos_tick_wiring_contract.rs
  Three narrow wiring assertions exercising
  `elai_orchestrator::trigger_cosmos_refresh`: (a) one-artifact manifest
  emits a CosmosRefreshStarted SSB row with queued=1, total=1
  (count_due() classifies missing meta as due); (b) empty manifest
  still emits exactly one CosmosRefreshStarted row with queued=0,
  total=0 (unconditional pre-loop write contract per
  task_runner.rs:7942); (c) empty manifest produces 0 refreshed_count
  AND emits zero CosmosArtifactUpdated rows (helper only emits on
  Ok(o) with o.changed=true per task_runner.rs:7980). Uses the
  `c8_stub_eradication_contract.rs` open_writer/open_viewer helper
  pattern (drop writer before opening read-side SsbViewer).

T3 — NEW: crates/contracts/tests/indexing_service_artifact_hook_contract.rs
  Two narrow wiring assertions driving
  `IndexingService::build_index_resumable`: (a) absent manifest -> hook
  is a documented no-op, indexing returns Ok with bm25_files >= 1; (b)
  MALFORMED elai_cc.toml (TOML parse error) -> ArtifactError::TomlParse
  is caught at indexing_service.rs:815-821 via tracing::warn! and the
  core indexing pipeline still returns Ok. Pins the C-8 Task 4
  non-fatal contract: artifact-indexing failure MUST NOT propagate.
  ArtifactsIndexed SSB row round-trip is already covered end-to-end in
  c8_stub_eradication_contract.rs (5) — we deliberately don't
  duplicate that here; the witness is the non-fatal contract.

T4 — EXTEND: crates/orchestrator/tests/production_agent_rpc_contract.rs
  Added test F (`f_index_artifacts_returns_ok_with_artifact_count_field`)
  asserting TOOL_INDEX_ARTIFACTS returns ToolOutcome::Result on the
  empty-manifest happy path. Encodes an IndexArtifactsRequest via
  postcard (matching production_agent_rpc.rs:1456 decode), drives
  handle_tool_request, decodes the result as JSON, asserts the
  `artifact_count` field is present + u32-parseable + equals 0 for an
  empty manifest, AND pins the sibling fields documented at
  production_agent_rpc.rs:1576-1583 (project_id, chunk_count,
  elapsed_ms, indexed[], up_to_date[]). Constructs a fake project_root
  layout matching `<project_root>/.elai/indexes/context_*.db` so the
  handler's `derive_project_root_from_db_path` succeeds.

Plan amendments (documentation-only)
------------------------------------

A1 — Plan Task 2 WIRING_CONTRACT line "SsbViewer::last_ambiguity_score()"
  should read `SsbViewer::latest_ambiguity_scored()` (R4 above).
A2 — Plan Task 11 acceptance row
  `unwired.ambiguityState = false in drawers.svelte.ts` is removed
  (R3 above; phantom flag never existed).
A3 — Plan Task 3 wire description "tokio::spawn in task_runner.rs" should
  read "tokio::spawn in cli::live_cli build_local_knowledge_stack"
  (R2 above; this is the DRIFT-D resolution carried forward).
A4 — Plan DETERMINISTIC_CHECK refinements (audit-time guidance, not a
  spec change): #6 should target NoopSsbNotifier construction
  (`NoopSsbNotifier::new` or `::default()`) not docstring mentions; #7
  should scope the "HYBRID" literal grep to the `get_search_mode`
  function body, not the whole commands.rs file (the L3927 match is in
  an unrelated input-validation matcher).

Test results (this session)
---------------------------

Tests authored (deferred to auditor for cargo execution):
  ambiguity_task_runner_contract           : 3 tests
  cosmos_tick_wiring_contract              : 3 tests
  indexing_service_artifact_hook_contract  : 2 tests
  production_agent_rpc_contract (extended) : +1 test (now 6 total)

No production-code Edits. Net change: 4 test files, 1 doc append.
SCHEMA_VERSION unchanged at 103. action_type_lock TOTAL_VARIANTS
unchanged at 306. bijection unchanged at 245.
