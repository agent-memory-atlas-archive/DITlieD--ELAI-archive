C-16 — Cosmos Pipeline Integration: Decisions

Session: s-32952
Plan: docs/thefinalglance/C-plan/C-16_cosmos_pipeline_integration.md
Context: workflow/sessions/s-32952/context-package-C16.md

Orchestrator decisions (A-H) reconciled against the C-16 plan + context-package
DRIFT items A-H. Recorded BEFORE EXECUTING per CLAUDE.md decision-doc rule.

----------------------------------------------------------------------
DECISION-A — Tick scheduler placement.

Plan §4 asks the scheduler to live in task_runner.rs. C-8 shipped it in
cli::live_cli::build_local_knowledge_stack and DECISION-D justified the
divergence: "task_runner handles a single task lifetime; the tick scheduler
must outlive any one task and reflect orchestrator uptime."

C-16 keeps the hourly outer loop in live_cli.rs (C-8 placement) and wires the
orphan task_runner.rs:7240 `trigger_cosmos_refresh` helper as the
**explicit refresh on demand** path. Two distinct callers, two distinct
lifetimes:

  1. Hourly cadence — live_cli.rs spawned task (lives for orchestrator uptime).
  2. On-demand refresh — task_runner.rs:7240 helper invoked from the Tauri
     `trigger_cosmos_refresh` command. One-shot per user click.

This satisfies the plan's deterministic check `rg cosmos task_runner.rs` AND
keeps the long-running scheduler outside task_runner's lifetime. Resolves
DRIFT-A and DRIFT-G.

----------------------------------------------------------------------
DECISION-B — Replace NoopResearchDispatcher in production.

The plan requires (§4) "ResearchDispatcher wired from production context".
C-8 shipped NoopResearchDispatcher (cosmos_worker.rs:705) as a placeholder
that returns Err on every call — every tick fails, TickReport.refreshed
stays empty, no SSB rows emit. C-16 adds:

  `HttpCosmosResearchDispatcher` in orchestrator::grounding_client

This adapter implements `elai_knowledge::ResearchDispatcher` and uses the
existing `SearxClient` (elai-web-search crate) to query the sidecar's
SearXNG-compatible `/search` endpoint, then concatenates the top T1-tier
results into a Markdown body with provenance footnotes. Pattern mirrors
`SearxSearchBackend` (orchestrator/src/searx_search_backend.rs) which already
bridges the sync `SearchBackend::search` trait to async `SearxClient` via a
dedicated current-thread tokio runtime + Mutex.

  - No new external dependencies (reuses elai-web-search + reqwest already
    pulled in by orchestrator).
  - No `blocking` feature added to reqwest — the bridge is the existing
    tokio current-thread runtime pattern.
  - Sidecar URL defaults to DEFAULT_SIDECAR_SEARCH_URL
    (`http://localhost:8100`). live_cli.rs reads `ELAI_SIDECAR_URL` /
    `ANTHROPIC_BASE_URL` env var and passes it to the adapter.
  - The Noop dispatcher remains in the crate (still used by read-only
    `cosmos_status_snapshot_standalone` — dashboard `get_cosmos_status` path
    never invokes the dispatcher, so a real one isn't needed there).

Resolves DRIFT-B.

----------------------------------------------------------------------
DECISION-C — SSB-listen transport: poll-based via subscribe_cosmos_events.

Plan §6 asks for SSB subscription. The dashboard has no `listen('ssb_row')`
infrastructure today (DRIFT-C). C-16 adds a Tauri command
`subscribe_cosmos_events(last_seq: i64) -> Result<CosmosEventBatch, String>`
that polls the SsbViewer for cosmos_* action types since `last_seq` and
returns a delta. The frontend polls this at 5s cadence (down from 30s) and
updates the drawer reactively.

This is the lighter end of the spectrum from the C-16 context-package DRIFT-C
recommendation. A real `app.emit()` push channel is a separate UI
infrastructure phase — out of scope for C-16.

Resolves DRIFT-C.

----------------------------------------------------------------------
DECISION-D — Tauri `trigger_cosmos_refresh` signature + return shape.

Plan ACCEPTANCE asks for `TriggerCosmosRefreshResponse`. The orchestrator
brief specifies:

```rust
pub struct TriggerCosmosRefreshResponse {
    pub refreshed_count: u32,
    pub started_at_ms: i64,
    pub completed_at_ms: i64,
}
```

The Tauri command builds a CosmosWorker on the fly (using the
HttpCosmosResearchDispatcher from DECISION-B + default_cosmos_root), then
calls `task_runner::trigger_cosmos_refresh(writer, &worker)` — the previously
orphan helper at task_runner.rs:7240 — and times the call. SsbViewer
last_seq before/after delta yields the refreshed_count.

If the writer is absent (cold-start dashboard with no SSB DB), the command
returns Err(String) — honest "wiring unavailable" failure, NOT mock data.

Resolves DRIFT-D + DRIFT-G.

----------------------------------------------------------------------
DECISION-E — CosmosArtifactDecayed SSB emit at cosmos_worker.rs:579.

The Decayed branch in `cosmos_get` is tracing-only today. C-16 adds an
optional `SsbEmitter` callback to CosmosWorker via a new
`with_ssb_emitter(...)` builder. When set, the Decayed branch invokes the
emitter with the artifact name + age + cadence. Tracing path stays in place
(both channels — SSB is additive, not a replacement).

  - SsbEmitter is a sync trait (`fn emit_*(...)`). Production implementation
    in orchestrator wraps an `Arc<Ssb>` + dedicated current-thread tokio
    runtime to bridge sync->async (same pattern as DECISION-B).
  - When None (tests / standalone read-only path), the worker is a pure
    no-op on the emission side — backward-compatible.

Resolves DRIFT-E.

----------------------------------------------------------------------
DECISION-F — CosmosRefreshStarted.queued = stale+decayed count.

Plan §3 mandates "artifacts_queued = stale+decayed count". Today both
emitters (live_cli, task_runner) use manifest_size as `queued` (renders
"N/M refreshing" with N==M always).

C-16 adds `pub fn count_due(&self) -> usize` to CosmosWorker. It iterates
the manifest, classifies each artifact via `status_for(&meta, now)`, and
counts non-Fresh entries (Stale + Decayed + Missing). Called by both
emitters before the SSB write so `queued` reflects actual work-to-do.

Resolves DRIFT-F.

----------------------------------------------------------------------
DECISION-G — Wire orphan task_runner.rs:7240 helper.

Per DECISION-A and DECISION-D, the Tauri `trigger_cosmos_refresh` command is
the new caller. The helper is no longer orphan: dashboard click ->
Tauri command -> `task_runner::trigger_cosmos_refresh(writer, &worker)`.

Resolves DRIFT-G.

----------------------------------------------------------------------
DECISION-H — Real blake3/size/latency in CosmosArtifactUpdated payload.

Today both emitters use `blake3: String::new(), body_bytes: 0, latency_ms: 0`
(DRIFT-H). C-16 changes the signature of `CosmosWorker::refresh_one` from

  `Result<bool, CosmosError>`

to

  `Result<RefreshOutcome, CosmosError>` where

```rust
pub struct RefreshOutcome {
    pub changed: bool,
    pub blake3: String,
    pub body_bytes: u32,
    pub latency_ms: u32,
}
```

Backward-compat for any call sites that only check `Ok(true)` for changed
status: `RefreshOutcome::changed` carries the same bool, and the unit-test
`status_for` path is unaffected (it doesn't call refresh_one).

Migration: live_cli.rs `cosmos_tick_emit_loop` rebuilt to call refresh_one
itself (rather than relying on `worker.tick()` which only returns a
`TickReport` of names). Simpler: extend TickReport to carry RefreshOutcome
records for each refreshed artifact.

Chose: extend TickReport with `outcomes: Vec<RefreshOutcome>` so existing
`refreshed: Vec<String>` stays for backward compat. Emitter reads
`outcomes` to populate the SSB payload with real data.

Resolves DRIFT-H.

----------------------------------------------------------------------
DECISION-I — 3x decay threshold rationale (Task 7).

cosmos_worker.rs uses `decay_secs = 3 * interval`. The C-16 plan §8 mentions
"age > 2x cadence = DECAYED" in its prose, but the code uses 3x. This is a
deliberate divergence introduced by B-42: the 3x factor provides a grace
buffer before SHADOW-serving stops.

Lowering to 2x would shrink the safe-stale window and risk falsely marking
artifacts DECAYED during a single missed refresh cycle. The B-42 audit
accepted 3x; C-16 documents the divergence in cosmos_worker.rs and does
NOT change the value (behavioral regression risk requires its own audit
cycle per CLAUDE.md "No removal or simplification without explicit user
confirmation").

  - decay_secs() formula: unchanged
  - cosmos_worker.rs doc-comment updated to cite C-16-decisions.md
  - cosmos_decay_contract.rs test file pins both the 3x ratio and the
    Fresh/Stale/Decayed banding contract

----------------------------------------------------------------------
DECISION-J — Tracing channels retained alongside SSB.

The four tracing sites in cosmos_worker.rs (lines 472, 543, 579, 516) are
left intact. SSB emission is additive — tracing is still the observability
channel for non-SSB consumers (CLI logs, audit replays without DB). The
context-package §"Tracing-only event sites" listed the four; C-16 adds SSB
to three (472 logical equivalent at the pre-loop point, 543, 579) and
leaves the body-overflow warn at line 516 as tracing-only (no SSB variant
for size truncation events, out of scope).

----------------------------------------------------------------------
Files modified (estimated):

  - .elai_cc/crates/knowledge/src/cosmos_worker.rs
       RefreshOutcome, count_due, SsbEmitter trait, with_ssb_emitter,
       tick() pre-loop CosmosRefreshStarted, refresh_one return type,
       cosmos_get Decayed SSB emit, doc-comment update.
  - .elai_cc/crates/orchestrator/src/grounding_client.rs
       HttpCosmosResearchDispatcher + SsbEmitterAdapter.
  - .elai_cc/crates/orchestrator/src/lib.rs
       Re-export HttpCosmosResearchDispatcher + SsbEmitterAdapter.
  - .elai_cc/crates/orchestrator/src/task_runner.rs
       Update trigger_cosmos_refresh helper to use count_due + RefreshOutcome.
  - .elai_cc/crates/cli/src/live_cli.rs
       Replace NoopResearchDispatcher with HttpCosmosResearchDispatcher;
       attach SsbEmitter to CosmosWorker; cosmos_tick_emit_loop refactor
       to use outcomes/count_due.
  - .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs
       Replace trigger_cosmos_refresh stub with one-shot worker call;
       new subscribe_cosmos_events command; TriggerCosmosRefreshResponse;
       CosmosEventBatch types.
  - .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs
       Register subscribe_cosmos_events.
  - .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts
       Update triggerCosmosRefresh return type; add subscribeCosmosEvents.
  - .elai_cc/crates/dashboard-v2/src/lib/drawers/CosmosDrawer.svelte
       Wire subscribeCosmosEvents poll at 5s; consume new response shape.

Files created:

  - .elai_cc/crates/knowledge/tests/cosmos_worker_tick.rs
  - .elai_cc/crates/knowledge/tests/cosmos_lock_contention.rs
  - .elai_cc/crates/knowledge/tests/cosmos_pipeline_stage.rs
  - .elai_cc/crates/contracts/tests/cosmos_decay_contract.rs
  - .elai_cc/crates/orchestrator/tests/cosmos_briefing_budget.rs

