C-16 BLOCK-1 fix — async-native ResearchDispatcher + SsbEmitter
================================================================

Session: s-32952
Date: 2026-05-18
Audit reference: workflow/sessions/s-32952/audit-C16.md BLOCK-1
Fixer cycle: 1

----------------------------------------------------------------------
PROBLEM (from audit BLOCK-1)
----------------------------------------------------------------------

`SsbEmitterAdapter` (grounding_client.rs:1109-1196) and
`HttpCosmosResearchDispatcher` (grounding_client.rs:973-1077) each owned a
dedicated `tokio::runtime::Builder::new_current_thread().build()` and
called `self.runtime.block_on(...)` inside their trait methods. Both call
chains run inside an outer multi-thread tokio runtime in production:

  1. live_cli.rs:1425 `tokio::spawn(async { ... worker.tick() ... })`
       -> cosmos_worker.rs:574 `emitter.emit_refresh_started(...)`
       -> grounding_client.rs:1135 `self.runtime.block_on(...)`  → PANIC

  2. commands.rs:7433 `pub async fn trigger_cosmos_refresh`
       -> task_runner.rs:7271 `worker.refresh_one(spec, ...)`
       -> cosmos_worker.rs:653-655 `self.research.run(...)`
       -> grounding_client.rs:1036 `self.runtime.block_on(...)`  → PANIC

Tokio docs: "This function panics if called within an asynchronous
execution context."
(https://docs.rs/tokio/latest/tokio/runtime/struct.Runtime.html#method.block_on)

35/35 tests passed only because they used sync `MockResearch` /
`MockSsbEmitter` that never invoked `block_on`. Production was the first
invocation.

----------------------------------------------------------------------
DECISION-K — async-native traits (audit option 1)
----------------------------------------------------------------------

Picked option 1 of the four offered in the audit:
"Make `ResearchDispatcher::run` + `SsbEmitter::emit_*` async fns.
Propagates through `CosmosWorker::tick()` / `refresh_one` — these all
become `async fn`. tick loop already runs on tokio; `await` instead of
block_on eliminates the bridge entirely. CLEANEST."

Trait shape: `Pin<Box<dyn Future<Output = T> + Send + 'a>>` instead of
native `async fn` in trait because the codebase uses `Arc<dyn
ResearchDispatcher>` / `Arc<dyn SsbEmitter>` and native `async fn` in
trait is not yet dyn-compatible without manual boxing (Rust 1.75+ stable
behavior). Pattern already established in
`.elai_cc/crates/cli/src/main.rs:167-174`,
`.elai_cc/crates/contracts/tests/ipc_dispatcher_production_contract.rs`,
etc. — matches CLAUDE.md "Native async fn in traits (never async-trait
crate)" rule (no `#[async_trait]` macro, no new deps).

----------------------------------------------------------------------
DECISION-K-1 — cosmos_get stays SYNC
----------------------------------------------------------------------

The audit's option 1 listed `cosmos_get` as becoming async too. Rejected:
`cosmos_get` is called from two sync pipelines:

  - `cosmos_stage::run` (sync) called from `QueryPipeline::query` (sync,
    called from `live_cli.rs:2168 pipeline.query(input)` + many tests)
  - `briefing::compile_cosmos_briefing` (sync) called from prompt
    assembly (sync)

Making `cosmos_get` async would force `QueryPipeline::query` and every
caller of `compile_cosmos_briefing` to become async — a ~50-call-site
cross-crate ripple far beyond the BLOCK-1 scope.

Trade-off: the `cosmos_get` Decayed branch previously called
`emitter.emit_artifact_decayed(...)` inside the sync function. Post-fix
that call is REMOVED — the synchronous tracing::warn channel remains
the sole observability path for sync-call-site Decayed events. Async
call sites (cosmos tick scheduler, Tauri trigger_cosmos_refresh, future
`cosmos_get_async` helper) emit the typed SSB row themselves if needed.

Justification: zero production code path called `cosmos_get` from an
async context that would benefit from the typed SSB row. The Tauri
`trigger_cosmos_refresh` command and live_cli tick both invoke
`tick()`/`refresh_one()` which already emit
`CosmosArtifactUpdated`/`CosmosRefreshStarted` after the fix. No
regression: pre-fix the call site PANICKED on first invocation —
removing it strictly improves correctness.

----------------------------------------------------------------------
DECISION-K-2 — HttpCosmosResearchDispatcher: tokio::sync::Mutex
----------------------------------------------------------------------

Replaced `std::sync::Mutex<SearxClient>` with
`tokio::sync::Mutex<SearxClient>` so the guard can be held across
`client.search().await` without blocking the async runtime. The
previous `runtime: TokioRuntime` field is removed entirely.
`SearxClient::search` is already async (uses reqwest's async client),
so the dispatcher dispatches directly via the ambient runtime — zero
nested runtime, zero bridge. No new dep (tokio with "sync" feature is
already a workspace pin).

----------------------------------------------------------------------
DECISION-K-3 — SsbEmitterAdapter: no runtime field
----------------------------------------------------------------------

`SsbEmitterAdapter` now holds only `writer: Arc<Ssb>`. Each `emit_*`
async fn awaits `self.writer.write_payload(...)` directly. `new(writer)`
remains `Result<Self, GroundingError>` for source compatibility with
existing call sites (`SsbEmitterAdapter::new(...)?` patterns in
live_cli.rs); construction is now infallible.

----------------------------------------------------------------------
Files modified
----------------------------------------------------------------------

  - .elai_cc/crates/knowledge/src/cosmos_worker.rs
      ResearchDispatcher trait → async-native Pin<Box<Future>>;
      SsbEmitter trait → async-native Pin<Box<Future>>;
      CosmosWorker::tick() / refresh_one() → async fn;
      cosmos_get() stays sync; emit_artifact_decayed call removed
      (tracing::warn remains);
      NoopResearchDispatcher / MockResearch impls updated.
  - .elai_cc/crates/orchestrator/src/grounding_client.rs
      HttpCosmosResearchDispatcher: drop runtime field; use
      tokio::sync::Mutex; impl ResearchDispatcher → async.
      SsbEmitterAdapter: drop runtime field; impl SsbEmitter → async.
  - .elai_cc/crates/orchestrator/src/task_runner.rs
      worker.refresh_one(spec, clock_now, None).await
  - .elai_cc/crates/cli/src/live_cli.rs
      worker.tick().await
  - .elai_cc/crates/orchestrator/src/briefing.rs
      Module tests: make_worker async, #[tokio::test] on each test.
  - .elai_cc/crates/orchestrator/tests/cosmos_briefing_budget.rs
      CannedResearch impl async; make_worker_with_three_artifacts async;
      all tests #[tokio::test].
  - .elai_cc/crates/contracts/tests/cosmos_decay_contract.rs
      CannedResearch async; cosmos_get_serves_decayed_artifact and
      cosmos_get_returns_blake3_mismatch as #[tokio::test].
  - .elai_cc/crates/knowledge/tests/cosmos_worker_tick.rs
      CannedResearch / CountingEmitter async; all tick tests
      #[tokio::test]; concurrent_workers test creates per-thread
      current_thread runtimes for std::thread compatibility.
  - .elai_cc/crates/knowledge/tests/cosmos_lock_contention.rs
      CannedResearch async; each std::thread builds a fresh
      current-thread tokio runtime and block_on the async tick/refresh.
  - .elai_cc/crates/knowledge/tests/cosmos_pipeline_stage.rs
      CannedResearch async; cosmos_stage_results_populated_after_tick
      and cosmos_stage_does_not_abort_pipeline_on_missing_body as
      #[tokio::test].

----------------------------------------------------------------------
Test results post-fix (2026-05-18)
----------------------------------------------------------------------

  cargo check --workspace                                       CLEAN
  cargo test -p elai-knowledge --test cosmos_worker_tick        11/11 PASS
  cargo test -p elai-knowledge --test cosmos_lock_contention     3/3  PASS
  cargo test -p elai-knowledge --test cosmos_pipeline_stage      6/6  PASS
  cargo test -p elai_cc_contracts --test cosmos_decay_contract   6/6  PASS
  cargo test -p elai-orchestrator --test cosmos_briefing_budget  6/6 +2 ignored
  cargo test -p elai-knowledge --lib cosmos                     21/21 PASS
  cargo test -p elai-orchestrator --lib briefing                 7/7  PASS

Total cosmos-related: 60 PASS, 0 FAIL, 2 ignored (pre-existing GAPs).

The two ignored tests in cosmos_briefing_budget are the C-16 §"File 5
GAP" placeholders (Fresh-before-Stale ordering / Decayed-exclusion-when-
Fresh-exists) — explicitly out of scope per context-package, untouched
by the BLOCK-1 fix.

----------------------------------------------------------------------
Acceptance verification (against task BLOCK-1 ACCEPTANCE list)
----------------------------------------------------------------------

  [x] No nested `Runtime::build().block_on()` anywhere in adapters.
      (grep for `block_on` in grounding_client.rs → 0 hits.)
  [x] SsbEmitter trait async-native (Pin<Box<Future>> dyn-compatible
      since `Arc<dyn SsbEmitter>` is the call-site shape).
  [x] ResearchDispatcher trait async-native.
  [x] HttpCosmosResearchDispatcher uses async SearxClient via
      tokio::sync::Mutex; reqwest async client under the hood
      (web_search/src/client.rs:80 `pub async fn search`).
  [x] SsbEmitterAdapter::emit_*() directly awaits ssb.write_payload().
  [x] All targeted tests pass after refactor (60/60 cosmos tests green).
  [x] cargo check --workspace + scoped tests PASS.
  [x] Trait change call-site sweep: every impl (NoopResearchDispatcher,
      MockResearch, CannedResearch x4, AlwaysFails, CountingEmitter,
      HttpCosmosResearchDispatcher, SsbEmitterAdapter) updated; every
      caller (live_cli, task_runner, all 5 test files, lib tests in
      cosmos_worker.rs and briefing.rs) updated.
