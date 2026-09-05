# C-16: Cosmos Pipeline Integration — CosmosStage wiring + tick scheduler + SSB emission + test suite
Objective: close the five structural gaps left open by B-42 — wire CosmosStage into
QueryPipeline, drive CosmosWorker::tick() from task_runner.rs on an hourly cadence,
implement trigger_cosmos_refresh (currently NotImplemented), replace tracing-only cosmos
events with real SSB row emission, and deliver the five missing standalone test files.
Estimated effort: 2 sessions.
Pre-requisites: C-2 (cosmos_* SSB action types defined in ssb.rs — DONE per B-42);
C-8 (CosmosStage wiring stub created in pipeline.rs — must exist before this plan executes).
Parallel-safe with: C-17 (knowledge resilience), C-18..C-20 (test infra).

## Context

B-42 shipped CosmosWorker (~1232 LOC), briefing compiler, 4 SSB action types
(CosmosRefreshStarted/CosmosArtifactUpdated/CosmosArtifactDecayed/CosmosGroundingUsed),
and CosmosDrawer.svelte. Three wiring gaps remain confirmed by
B-42_cosmos_artifact_tier.report.md (MOSTLY_DONE, dated 2026-05-17):

1. CosmosStage NOT in QueryPipeline — pipeline.rs has zero cosmos references; plan §5
   requires insertion between ArtifactStage and ARagStage (pipeline.rs:532-573 region).
2. CosmosWorker::tick() NOT called from task_runner.rs — cosmos_worker.rs documents
   DEFAULT_TICK_INTERVAL_SECS=3600 but no caller wires the hourly scheduler.
3. trigger_cosmos_refresh Tauri command returns explicit NotImplemented error
   (commands.rs:6615); production ResearchDispatcher not injected.
4. SSB rows NOT emitted from cosmos_worker.rs — events go to tracing channel only;
   4 SSB variants in ssb.rs unreachable from CosmosWorker::tick().
5. Five standalone test files absent: cosmos_worker_tick.rs, cosmos_lock_contention.rs,
   cosmos_pipeline_stage.rs (knowledge/tests/); cosmos_decay_contract.rs (contracts/tests/);
   cosmos_briefing_budget.rs (orchestrator/tests/).

Decay threshold discrepancy: plan §8 says "age > 2x cadence = DECAYED"; code uses
3x (cosmos_worker.rs:102-106). This is a deliberate divergence — the 3x factor provides
a grace buffer before SHADOW-serving. This plan documents the divergence in a code
comment and in workflow/reasoning/C-16-decisions.md rather than changing the live value
(changing decay threshold = behavioral regression risk requiring its own audit cycle).

## Reads

- .elai_cc/crates/knowledge/src/pipeline.rs (full — stage insertion point at lines 532-573)
- .elai_cc/crates/knowledge/src/cosmos_worker.rs (tick/SSB integration points)
- .elai_cc/crates/orchestrator/src/task_runner.rs (scheduler pattern; lines 1-130 for imports)
- .elai_cc/crates/orchestrator/src/briefing.rs (compile_cosmos_briefing; prompt injection point)
- .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs (trigger_cosmos_refresh stub at 6615)
- .elai_cc/crates/dashboard-v2/src/lib/drawers/CosmosDrawer.svelte (SSB event handling)
- .elai_cc/crates/orchestrator/src/ssb.rs (CosmosRefreshStartedPayload at 4892-4935)
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\B-plan\B-42_cosmos_artifact_tier.report.md

## Tasks

1. Decision doc: write workflow/reasoning/C-16-decisions.md documenting (a) CosmosStage
   insertion point in pipeline.rs between ArtifactStage and ARagStage (lines 532-573),
   (b) 3x vs 2x decay threshold deliberate divergence rationale, (c) SSB injection
   strategy via Ssb handle passed into tick(), (d) trigger_cosmos_refresh implementation
   approach (in-process tokio::spawn of one-shot tick; no separate thread).

2. Add CosmosStage struct to pipeline.rs (field: cosmos_stage: Option<CosmosStage>)
   and with_cosmos_stage() builder method. CosmosStage wraps CosmosWorker (Arc) and
   runs cosmos_get() for relevant artifacts given the query; returns Vec<RetrievalResult>
   with tool_used="cosmos". Stage label: "cosmos". Insert between ArtifactStage (line ~558)
   and ARagStage (line ~562) in query_with_tier(); skip for FAST tier; active for
   STANDARD and DEEP.

3. Emit SSB rows from CosmosWorker::tick(). Extend CosmosWorker::new() to accept
   Option<Arc<Ssb>> (or a SsbEmitter trait). In tick(): emit CosmosRefreshStarted before
   the manifest loop (artifacts_queued = stale+decayed count, manifest_size). In
   refresh_one(): emit CosmosArtifactUpdated on Ok(true); emit CosmosArtifactDecayed
   on cosmos_get() returning Decayed status. SSB emitter is Option — absent in tests
   (None), present in production. No tracing-only path removed (keep both; tracing is
   still the observability channel for non-SSB consumers).

4. Wire CosmosWorker::tick() into task_runner.rs. At TaskRunner startup (after
   TaskRunner::new() / start_task_runner()), spawn a tokio task that loops:
   tick() -> TickReport logged at info level -> tokio::time::sleep(Duration::from_secs(
   DEFAULT_TICK_INTERVAL_SECS)) -> repeat. CancelHandle::cancel() must terminate the
   loop (check cancel flag before sleep, use tokio::select! over cancel_notify and
   sleep). Worker requires CosmosManifest + cosmos_root + clock + ResearchDispatcher
   wired from production context (cloud grounding client or sidecar grounding client
   depending on ExecutionMode).

5. Implement trigger_cosmos_refresh Tauri command (commands.rs:6615). Replace the
   NotImplemented error with a tokio::spawn of a one-shot CosmosWorker::tick() using
   the same production dispatcher injected at startup. Result is a TriggerCosmosRefreshResponse
   {queued: bool, artifacts_due: usize} returned immediately; the actual refresh runs
   in the background. Command remains best-effort — if dispatcher is not yet initialized
   (cold start), return queued=false with a reason field.

6. Verify CosmosDrawer.svelte handles CosmosRefreshStarted / CosmosArtifactUpdated /
   CosmosArtifactDecayed SSB events (not just the status poll). If the drawer currently
   only uses the 30s poll path from MonitorWindow.svelte, add SSB subscription for
   these three action types so the card updates live without waiting for the next poll.
   CosmosGroundingUsed is informational only — log to browser console, no UI state.

7. Document decay threshold divergence in cosmos_worker.rs at line ~102: add a comment
   block explaining the 3x factor choice (grace buffer) and referencing C-16-decisions.md.
   No code change to decay_secs(). The B-35 report and this plan are the audit trail.

## WIRING_CONTRACT

produces_symbols:
  - CosmosStage struct in knowledge::pipeline (field cosmos_stage in QueryPipeline,
    with_cosmos_stage() builder, run() in stage execution block with label "cosmos")
  - SSB emission from CosmosWorker::tick() (CosmosRefreshStarted, CosmosArtifactUpdated,
    CosmosArtifactDecayed via Ssb handle passed as Option<Arc<Ssb>>)
  - cosmos tick loop in task_runner.rs startup path (tokio task, 3600s cadence,
    cancel-safe via CancelHandle)
  - trigger_cosmos_refresh Tauri command returns TriggerCosmosRefreshResponse
    {queued, artifacts_due} instead of NotImplemented
  - CosmosDrawer.svelte SSB subscription for CosmosRefreshStarted/Updated/Decayed
consumes_symbols:
  - CosmosWorker::tick() (.elai_cc/crates/knowledge/src/cosmos_worker.rs:444)
  - DEFAULT_TICK_INTERVAL_SECS (.elai_cc/crates/knowledge/src/cosmos_worker.rs:71)
  - SsbActionType::{CosmosRefreshStarted,CosmosArtifactUpdated,CosmosArtifactDecayed}
    (.elai_cc/crates/orchestrator/src/ssb.rs:923-931)
  - ArtifactStage insertion pattern (.elai_cc/crates/knowledge/src/artifacts/stage.rs)
  - CancelHandle from task_runner.rs (existing cancel mechanism)
  - CloudGroundingClient / SidecarGroundingClient as ResearchDispatcher impl
reachable_from:
  - QueryPipeline::query_with_tier() (cosmos stage runs after artifact stage, before arag)
  - task_runner::start_task_runner() (hourly tick loop)
  - Tauri command trigger_cosmos_refresh (one-shot tick)
  - CosmosDrawer.svelte (live SSB subscription)
stub_ban:
  - trigger_cosmos_refresh MUST NOT return NotImplemented after this plan
  - cosmos tick MUST acquire IndexLock(LockScope::Index) before calling tick()
    (cosmos_worker.rs:328 documents this as caller's responsibility)
  - SSB CosmosRefreshStarted MUST be emitted before the manifest loop, not after
frontend_pair: CosmosDrawer.svelte SSB subscription for three action types
observable_effect:
  trigger: configure [[knowledge.artifacts]] in elai_cc.toml with one artifact entry
           whose refresh_cadence = "daily"; run `elai /research --mode cloud
           "how do I use the X endpoint?"` via cloud Sonnet 4.6
  expect_ssb: CosmosRefreshStarted row with artifacts_queued >= 1; CosmosArtifactUpdated
              row per refreshed artifact; query_trace SSB stage="cosmos" present
  expect_artifact: CosmosWorker::status_snapshot() returns [{status: "fresh", ...}]
                   within one tick cycle; CosmosDrawer.svelte renders updated timestamp
                   without full page reload; briefing compiler log includes AVAILABLE_COSMOS
                   block in injected prompt

## DETERMINISTIC_CHECK

```bash
# 1. CosmosStage field in QueryPipeline
rg "cosmos_stage" .elai_cc/crates/knowledge/src/pipeline.rs

# 2. cosmos stage label in query_with_tier
rg "stage.*cosmos|cosmos.*stage" .elai_cc/crates/knowledge/src/pipeline.rs

# 3. SSB emission in cosmos_worker tick path
rg "CosmosRefreshStarted|CosmosArtifactUpdated|CosmosArtifactDecayed" \
   .elai_cc/crates/knowledge/src/cosmos_worker.rs

# 4. Tick loop in task_runner
rg "cosmos|tick.*interval|DEFAULT_TICK_INTERVAL" \
   .elai_cc/crates/orchestrator/src/task_runner.rs

# 5. trigger_cosmos_refresh no longer returns NotImplemented
rg "NotImplemented|not_implemented" \
   .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs | grep -v "^Binary"

# 6. Five test files present
for f in \
  .elai_cc/crates/knowledge/tests/cosmos_worker_tick.rs \
  .elai_cc/crates/knowledge/tests/cosmos_lock_contention.rs \
  .elai_cc/crates/knowledge/tests/cosmos_pipeline_stage.rs \
  .elai_cc/crates/contracts/tests/cosmos_decay_contract.rs \
  .elai_cc/crates/orchestrator/tests/cosmos_briefing_budget.rs; do
  test -f "$f" && echo "OK: $f" || echo "MISSING: $f"
done

# 7. Decay threshold divergence documented
rg "3x|3 \* interval|deliberate|C-16" \
   .elai_cc/crates/knowledge/src/cosmos_worker.rs

# 8. Crate-level tests compile and pass (no live inference required)
cargo test -p elai_knowledge -- cosmos 2>&1 | tail -5
cargo test -p elai_orchestrator -- cosmos_briefing 2>&1 | tail -5
cargo test -p elai_contracts -- cosmos_decay 2>&1 | tail -5
```

## ACCEPTANCE

- [ ] CosmosStage field in QueryPipeline; with_cosmos_stage() builder compiles; stage
      label "cosmos" appears in query_with_tier result for STANDARD/DEEP tiers
- [ ] FAST tier skips cosmos stage (no cosmos entry in PipelineStage list for FAST queries)
- [ ] CosmosWorker::tick() emits CosmosRefreshStarted SSB row before manifest loop
      when Ssb handle is Some; SSB row has artifacts_queued >= 0 field
- [ ] CosmosArtifactUpdated SSB row emitted on Ok(true) from refresh_one()
- [ ] CosmosArtifactDecayed SSB row emitted when cosmos_get() returns Decayed
- [ ] CosmosWorker accepts None Ssb handle without panic; tracing path unchanged
- [ ] task_runner.rs spawns tokio tick loop at 3600s cadence; loop terminates on
      CancelHandle::cancel() without tokio runtime error
- [ ] IndexLock(LockScope::Index) acquired inside tick loop before calling tick()
- [ ] trigger_cosmos_refresh Tauri command returns TriggerCosmosRefreshResponse
      {queued: true, artifacts_due: N} for N >= 0; no NotImplemented error
- [ ] CosmosDrawer.svelte subscribes to CosmosRefreshStarted/Updated/Decayed and
      re-renders card without waiting for next 30s poll
- [ ] Decay threshold divergence (3x vs 2x) documented in cosmos_worker.rs and
      workflow/reasoning/C-16-decisions.md
- [ ] All five standalone test files exist and pass (no live inference):
      cosmos_worker_tick.rs (~200 LOC), cosmos_lock_contention.rs (~150 LOC),
      cosmos_pipeline_stage.rs (~180 LOC), cosmos_decay_contract.rs (~100 LOC),
      cosmos_briefing_budget.rs (~120 LOC)
- [ ] cargo check --workspace passes after all changes
- [ ] Cloud-Sonnet smoke test (live, manual): configure [[knowledge.artifacts]] in
      elai_cc.toml with one OpenAPI spec URL fixture; run
      `elai /research --mode cloud "how do I use the repos endpoint?"` via cloud
      Sonnet 4.6; assert: CosmosRefreshStarted SSB row appears in query_trace;
      artifact status_snapshot() returns Fresh within tick cadence; CosmosStage
      visible in query_trace with stage="cosmos"; prompt log contains AVAILABLE_COSMOS
      block (briefing compiler injected); CosmosDrawer renders CosmosRefreshStarted /
      CosmosArtifactUpdated / CosmosArtifactDecayed events live

## Test files

- CREATE: .elai_cc/crates/knowledge/tests/cosmos_worker_tick.rs (~200 LOC)
  Tests: tick writes body+meta on first run; tick skips Fresh artifacts; tick refreshes
  Stale artifacts; tick refreshes Decayed artifacts; tick is idempotent when body unchanged;
  tick records failed artifacts without panic; TickReport counts match manifest size;
  concurrent tick calls on separate workers do not corrupt shared cosmos_root (tempdir
  isolation); SSB handle Some emits CosmosRefreshStarted; SSB handle None is a no-op.

- CREATE: .elai_cc/crates/knowledge/tests/cosmos_lock_contention.rs (~150 LOC)
  Tests: two CosmosWorker instances targeting the same tempdir do not produce torn writes
  (write_atomic temp+rename is the only write path); blake3 hash read after concurrent
  write matches the written body; stress test: 4 threads each calling tick() 5 times
  against the same root with distinct manifest slices (non-overlapping artifact names)
  produces zero Blake3Mismatch errors.

- CREATE: .elai_cc/crates/knowledge/tests/cosmos_pipeline_stage.rs (~180 LOC)
  Requires CosmosStage wiring (Task 2) to be complete.
  Tests: QueryPipeline with cosmos_stage set returns a "cosmos" PipelineStage for
  STANDARD tier; result list is empty when worker has no refreshed artifacts;
  result list is non-empty after tick() writes a body; FAST tier query omits "cosmos"
  stage; stage error from cosmos_get() (e.g. MissingBody) does not abort pipeline
  (error field set, results empty, pipeline continues).

- CREATE: .elai_cc/crates/contracts/tests/cosmos_decay_contract.rs (~100 LOC)
  Tests: CosmosStatus::Fresh at age < interval; CosmosStatus::Stale at age between
  1x and 3x interval; CosmosStatus::Decayed at age >= 3x interval (not 2x — document
  deliberate 3x choice in test comment); decay_secs() == 3 * interval_secs() for all
  four cadences; CosmosWorker::cosmos_get() emits WARN and returns Decayed (not Err)
  when artifact exceeds 3x threshold; Blake3 mismatch returns Err not Decayed.

- CREATE: .elai_cc/crates/orchestrator/tests/cosmos_briefing_budget.rs (~120 LOC)
  Tests: compile_cosmos_briefing with 3 artifacts, budget = 200 tokens produces output
  <= 800 chars; lower-tier_relevance artifacts dropped when budget exceeded; artifact
  with tier_relevance < DEFAULT_TIER_RELEVANCE_THRESHOLD excluded entirely; Fresh
  artifact appears before Stale in output; Decayed artifact excluded when a Fresh
  alternative exists for the same project_class; empty manifest produces empty string.

## Out of scope

- CosmosStage embedding (vector search over cosmos bodies) — cosmos artifacts are Markdown
  briefs injected whole; chunked vector search over them is a future enhancement.
- Auto-seeding the cosmos_default_manifest.toml from elai_cc.toml [[knowledge.artifacts]]
  config — C-16 wires the existing manifest path; config-driven manifest merging is C-17+.
- CosmosCheck verifier changes — LocalVerifier::with_cosmos_lookup is complete (B-42);
  C-16 does not modify reasoning/src/verification.rs.
- Dashboard cosmos settings UI (artifact CRUD, cadence editor) — deferred.
- spawn_loop() async variant in cosmos_worker.rs — the synchronous tick() driven by the
  task_runner loop is sufficient for the 3600s cadence; async loop is over-engineered.
- Full workspace test suite — auditor's job; C-16 runs only -p elai_knowledge,
  -p elai_orchestrator, -p elai_contracts plus cargo check --workspace.
