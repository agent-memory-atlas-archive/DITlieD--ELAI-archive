# E-44: Idempotency-keyed tool wrappers + SAGA compensation log + Continue-As-New rollover + DST harness for AgentPool
Objective: close the three concrete gaps in ELAI's SessionDag durability layer per ACRFence (arXiv:2603.20625) and the Temporal/DBOS/Inngest 2025-2026 convergence — (a) add an idempotency-key layer between AgentCheckpoint and every tool call to defeat semantic-rollback attacks, (b) promote SSB into a SAGA log with compensating actions logged BEFORE each irreversible side effect, (c) enforce a hard event-count ceiling on SessionDag with Continue-As-New rollover, (d) add a madsim-based deterministic simulation harness for AgentPool supervisor tests, and (e) ship the lazy checkpoint schema migrator that unblocks the SCHEMA_VERSION 41/44/45 trio already in flight.
Estimated effort: 3 sessions, ~10-12 h
Pre-requisites: B-9 SHIPPED (SessionDag Activation, s-59964), B-40 SHIPPED (memory project_b40_shipped.md, SCHEMA_VERSION 41), B-42 SHIPPED (Cosmos Artifact Tier, SCHEMA_VERSION 44), B-41 SHIPPED (Per-Level Auditor Modes, SCHEMA_VERSION 45)
Parallel-safe with: E-41 (PR CI — different surface), E-42 (grammars), E-43 (skills), E-45 (drift — feeds the SAGA log validator)
Source report: workflow/improvement-sweep-2026-05-19/reports/44-checkpointing-durability.md

## Context

ELAI's SessionDag (B-9 SHIPPED, s-59964) and AgentCheckpoint (per-agent 5-msgpack files) are CMV-derived (arXiv:2602.22402), blake3-integrity-checked, branch-bounded (max 3 active branches), and disk-backed. The May 2026 durability report identifies three concrete gaps against 2025-2026 SOTA:

**Gap A — idempotency at the tool boundary.** ACRFence (arXiv:2603.20625, Mar 2026, UCSC+UConn) demonstrates semantic-rollback attacks against LangGraph, Cursor, and Claude Code: retried tool calls produce subtly different requests even at temperature=0 due to GPU floating-point order across runs, and servers treat these as new requests. Without idempotency keys, a crash-then-retry on an irreversible side effect (file delete, git push, pod terminate) double-executes. Temporal/DBOS/Inngest converged on this in 2025; ELAI has no equivalent. Per the report: "ELAI should auto-generate `{session_id}:{phase_id}:{step_id}:{tool_name}:{call_seq}` and require every tool wrapper to accept it."

**Gap B — SAGA log for compensating actions.** Temporal (temporal.io/blog/compensating-actions-part-of-a-complete-breakfast-with-sagas — "AI agent workflows are sagas in disguise"), DBOS, Inngest, Restate, and Azure (learn.microsoft.com/en-us/azure/architecture/patterns/compensating-transaction) all enforce: log the reverse action BEFORE the side effect executes. ELAI's SSB has atomic durability markers but no compensation-table — a crash-after-effect cannot unwind. The fix is a new SSB table `compensation_actions` written before any irreversible op (delete, push, terminate); on crash-recovery, if SSB shows committed action without phase_complete marker, replay compensations in reverse order.

**Gap C — Continue-As-New rollover.** Temporal's event-history cap is the canonical answer to unbounded DAG growth (zylos.ai/research/2026-03-04). The CMV trim primitive already in SessionDag (B-9) is the right substrate; what's missing is a hard event-count ceiling that triggers rollover (snapshot minimal carry-over state, archive old DAG under blake3 hash, start new root). The report suggests 500 nodes or 100 phases; the actual threshold needs measurement (report Open Question 1: instrument 4-hour runs, plot DAG depth + SQLite write latency, pick where p99 exceeds 50ms).

**Gap D — DST harness.** madsim (github.com/madsim-rs/madsim) and Antithesis ($105M Series A Dec 2025) bring deterministic simulation testing to async Rust; RisingWave uses madsim in production. ELAI's AgentPool supervisor has no DST coverage — a race-condition crash in a 4-hour run is invisible to current tests. madsim swaps tokio in `#[cfg(test)]` with deterministic scheduling and lets the test inject crashes at every await point.

**Gap E — lazy schema migrator.** ELAI's three recent B-Ns bumped SCHEMA_VERSION 41 → 44 → 45 (memory project_b40/42/41_shipped.md). LangGraph hit this exact problem and the answer is forward-compat reader + lazy state transformer at load (langgraphjs#536). E-44 ships the AgentCheckpoint migrator so old checkpoints don't break new code.

The report also recommends postcard over msgpack for AgentCheckpoint files (30% smaller, no_std, deterministic byte order, blake3-friendly) keeping msgpack only at the dashboard IPC boundary; this is included as a feature-gated path, not the default — switching the on-disk format is a separate measurement-driven decision.

## Existing state (B/C/D coverage)

- B-9 SHIPPED (SessionDag Activation, s-59964; dag-write-in-dashboard deferred), per B-PLAN-SUMMARY:217-228: SessionDag::create_handoff at 85% token threshold, msgpack BLOB columns (rmp_serde), per-GRC-step / per-tool-call / per-phase-transition checkpoints, branch UI (/session fork|merge|close|list), SPEC AMENDMENT #2 SSB action_type='checkpoint' on merge_branch. EXTENSION: E-44 adds the idempotency-key layer + SAGA compensation log + Continue-As-New rollover on top of the existing SessionDag; no schema replacement, only additions.
- B-40 / B-42 / B-41 SHIPPED (memory project_b40/42/41_shipped.md): SCHEMA_VERSION 41/44/45 trio. EXTENSION: E-44 ships the lazy migrator at load that handles forward-compat across these and future bumps.
- B-Plan AgentCheckpoint at 5 msgpack files per agent: per elai.map hits at session_dag.rs:1956 / fsm.rs:1187 / feasibility_probe.rs:918, the msgpack files are the disk-of-truth. EXTENSION: E-44 adds an optional postcard format behind a feature flag; default remains msgpack until measurements support the swap.
- PreCompact hook auto-saves FSM state to SESSION_HANDOFF (CLAUDE.md). EXTENSION: E-44 adds a lock/barrier so the 85% token compact does not race with the PreCompact hook's SESSION_HANDOFF write — report Open Question 4.
- `crates/orchestrator/src/session_dag.rs` already implements trim/branch/merge per the elai.map hits at session_dag.rs:436, 491, 685, 2914, 3229, 3250, 3310, 3322, 3337. EXTENSION: E-44 adds the event-count ceiling + Continue-As-New trigger; the trim primitive already handles the data movement.
- No current B-N or C-N covers ACRFence-style idempotency or SAGA compensation. E-44 is the first phase to ship it.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\44-checkpointing-durability.md` (full)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 217-228 (B-9 SessionDag scope) + lines 968-1010 (SCHEMA_VERSION bump log)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\session_dag.rs` lines 430-700 (trim/branch/merge primitives — confirm extension points)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\session_dag.rs` lines 1900-3350 (test fixtures + handoff path)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs` (compensation table addition target)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\fsm.rs` line 1187 area (fsm-state.msgpack location — touched by lazy migrator)
- WebFetch: arXiv:2603.20625 (ACRFence — VERIFY ID per memory feedback_paper_citation_verify before citing)
- WebFetch: github.com/madsim-rs/madsim (DST runtime)
- WebFetch: temporal.io/blog/compensating-actions-part-of-a-complete-breakfast-with-sagas
- WebFetch: learn.microsoft.com/en-us/azure/architecture/patterns/compensating-transaction
- WebFetch: github.com/jamesmunns/postcard (deterministic byte order, no_std)
- WebFetch: github.com/langchain-ai/langgraphjs/issues/536 (lazy schema transform pattern)

## Tasks

1. Idempotency-key layer. New module `crates/orchestrator/src/idempotency.rs` with `IdempotencyKey::compose(session_id, phase_id, step_id, tool_name, call_seq) -> String` (deterministic, sortable). Every `mcp_*` call wrapper and every Bash/Edit/Write tool wrapper accepts an `idem: IdempotencyKey` parameter and consults AgentCheckpoint's `tool_results` map before issuing the real call. Replay-or-fork semantic per ACRFence: if a prior call existed with the same idem AND had irreversible effect (per the SAGA log of Task 2), replay returns the cached result; if no irreversible effect, fork allowed (re-execute). SSB row: `tool_idempotency_hit` (cache hit) or `tool_idempotency_replayed_irreversible` (cached because of saga entry). Cited grounding: arXiv:2603.20625 ACRFence + Temporal Activity model.

2. SAGA compensation log in SSB. Add a new SSB table `compensation_actions` with columns `{session_id, phase_id, action_id, action_name, inverse_action_name, inverse_args_msgpack, written_at, executed_at, status}`. Every irreversible op declares its compensation: file delete writes `{action: delete, inverse: write_back, args: <prev_bytes>}` BEFORE the delete fires; git push writes `{action: push, inverse: revert_or_reset_to_<prev_sha>}`; pod terminate writes `{action: terminate, inverse: respawn_with_<prev_config>}`. On crash-recovery: scan compensation_actions where `status=committed AND phase_status != complete`; replay inverse_action_name in reverse order. New SSB action types `compensation_logged`, `compensation_replayed`, `compensation_failed`. Cited grounding: temporal.io/blog/compensating-actions + Azure compensating-transaction pattern.

3. Continue-As-New rollover for SessionDag. Add a tunable threshold `session_dag.event_count_ceiling` (default 500) and `session_dag.phase_count_ceiling` (default 100); when SessionDag exceeds either, atomically (a) snapshot minimal carry-over state to `session_dag_archive/<old_session_id>.archive` keyed by blake3 hash of the archived content, (b) start a new root with the carried-over state, (c) emit SSB row `session_dag_continue_as_new` referencing both old + new session_id. The trim primitive already lives at session_dag.rs:436-685 — this task adds the cap-and-rollover policy on top. Cited grounding: zylos.ai/research/2026-03-04-ai-agent-workflow-checkpointing-resumability + Temporal Continue-As-New.

4. DST harness for AgentPool. Add a `#[cfg(test)] use madsim::runtime::Runtime;` shim in `crates/orchestrator/tests/agent_pool_dst.rs` that swaps tokio for madsim only in test builds. Add `madsim = { version = "X", optional = true }` to `crates/orchestrator/Cargo.toml` under `[dev-dependencies]` — verify version via `cargo search madsim` before pinning. The DST tests inject crashes at every await point in `AgentPool::supervise()` and assert the supervisor honors the 3-restart-per-N-seconds Erlang limit per the report's "anti-spiral skill formalized as a test." Cited grounding: github.com/madsim-rs/madsim + medium.com/let-it-crash-7-erlang-lessons + supertrees crate.

5. Lazy AgentCheckpoint schema migrator. New `crates/orchestrator/src/checkpoint_migrator.rs` with a registry of transform functions keyed by `(from_version, to_version)`. On AgentCheckpoint load: read `schema_version` from the msgpack header; if `version < CURRENT_SCHEMA_VERSION`, apply registered transforms in order; emit `checkpoint_lazy_migrated` SSB row with `{from, to, file_path}`. Never big-bang. Register transforms for 40→41 (B-40), 41→42 (B-41 pin per memory), 42→44 (B-42), 44→45 (B-41 — note: the SCHEMA_VERSION sequence may not be monotonic by B-N number per the memory — read each project_bN_shipped.md to confirm). Cited grounding: langgraphjs#536 + LangGraph's "no built-in detection of schema-incompatible state."

6. PreCompact + 85% token compact race fix. Per report Open Question 4: reproduce by injecting a crash 100ms into compaction; if SSB ends inconsistent, add a file-lock or a sentinel SSB row that PreCompact reads. New `crates/orchestrator/src/compact_lock.rs` (~50 LOC) with `acquire_compact_lock()`/`release_compact_lock()` using a flock-on-Linux / LockFileEx-on-Windows sentinel at `workflow/sessions/{id}/.compact-lock`. Both PreCompact hook and the 85% trigger acquire before mutating SESSION_HANDOFF or SSB. SSB row: `compact_lock_contended` if waiter blocks >100ms.

7. Postcard feature-flag for AgentCheckpoint on-disk format. Behind a Cargo feature `postcard_checkpoints`, switch the 5 msgpack files to postcard serialization. Default remains msgpack — switching requires measurement first (the dashboard IPC boundary must keep msgpack per report Open Question 5). Add a benchmark target `crates/orchestrator/benches/checkpoint_format.rs` comparing msgpack vs postcard on AgentCheckpoint size + serialize/deserialize latency; document result in `workflow/audits/E-44-checkpoint-format-bench-2026-05-19.md`. Cited grounding: github.com/jamesmunns/postcard + david.kolo.ski/blog/rkyv-is-faster-than (postcard ~70% bincode size).

8. Reliability metrics (RDC / VAF / GDS / MOP) as observability. Per arXiv:2603.29231 (Reliability Decay Curve / Variance Amplification): instrument AgentPool to emit `{rdc, vaf, graceful_degradation_score, meltdown_onset_point}` over rolling windows of 10/50/100 tool calls. New SSB action type `agent_reliability_metrics`. Auditor reads these during phase-end audit; "meltdown onset detected" emits a phase-level recommendation to escalate or restart. Cited grounding: arXiv:2603.29231 — VERIFY ID per memory feedback_paper_citation_verify before citing.

9. Tool-error 3-bucket recovery split. Per arXiv:2603.22744 (cited in report's recommendation 9): classify errors at the tool boundary into {transient, semantic, spec-level} — transient (network/429) → tool-level retry with backoff; semantic (wrong arg type, validation fail) → step-level replan; spec-level (contract violation) → phase-level escalation. New module `crates/orchestrator/src/error_taxonomy.rs` with deterministic regex classifier + a `RecoveryDirective` enum. Already partially exists at `.claude/tools/failure_taxonomy.py` for auditor builds — port the categories into orchestrator runtime. SSB row: `tool_error_classified` with `{bucket, retry_path}`. Cited grounding: arXiv:2603.22744 — VERIFY ID.

10. SAGA + idempotency end-to-end smoke. New integration test `crates/orchestrator/tests/saga_idempotency_e2e.rs` that simulates: (a) tool call begins, (b) compensation written, (c) crash injected before tool call's effect commits, (d) recovery sees committed compensation_actions, (e) replay re-issues with same idempotency key → cache hit → no double-effect. Asserts no double-write to a fixture file, no double-emission of SSB rows, and SessionDag root unchanged.

## WIRING_CONTRACT

produces_symbols:
  - orchestrator::idempotency::IdempotencyKey::compose(session_id: &str, phase_id: &str, step_id: &str, tool_name: &str, call_seq: u32) -> String
  - orchestrator::idempotency::IdempotencyKey::lookup(checkpoint: &AgentCheckpoint, key: &str) -> Option<CachedResult>
  - orchestrator::saga::SagaLogger::log_compensation(action: SagaAction) -> Result<()>
  - orchestrator::saga::SagaLogger::replay_pending(session_id: &str) -> Result<Vec<SagaResult>>
  - orchestrator::session_dag::continue_as_new(threshold: usize) -> Result<NewRoot>
  - orchestrator::checkpoint_migrator::migrate(loaded: RawCheckpoint, from: u32, to: u32) -> Result<AgentCheckpoint>
  - orchestrator::compact_lock::CompactLock::acquire(session_id: &str) -> Result<CompactGuard>
  - orchestrator::error_taxonomy::classify(err: &ToolError) -> RecoveryDirective
  - orchestrator::reliability::compute_metrics(window: &[ToolCallOutcome]) -> ReliabilityMetrics
  - orchestrator::ssb::SsbActionType::ToolIdempotencyHit
  - orchestrator::ssb::SsbActionType::ToolIdempotencyReplayedIrreversible
  - orchestrator::ssb::SsbActionType::CompensationLogged
  - orchestrator::ssb::SsbActionType::CompensationReplayed
  - orchestrator::ssb::SsbActionType::CompensationFailed
  - orchestrator::ssb::SsbActionType::SessionDagContinueAsNew
  - orchestrator::ssb::SsbActionType::CheckpointLazyMigrated
  - orchestrator::ssb::SsbActionType::CompactLockContended
  - orchestrator::ssb::SsbActionType::AgentReliabilityMetrics
  - orchestrator::ssb::SsbActionType::ToolErrorClassified
consumes_symbols:
  - crates/orchestrator/src/session_dag.rs (trim + branch + handoff already shipped per B-9)
  - crates/orchestrator/src/ssb.rs (new compensation_actions table)
  - crates/orchestrator/src/fsm.rs:1187 area (fsm-state.msgpack location)
  - .claude/tools/failure_taxonomy.py (categories source — port to Rust)
  - blake3::Hash (NEVER blake2/sha2 — HARD RULE)
reachable_from:
  - crates/orchestrator/src/task_runner.rs (every tool dispatch wraps idem + saga)
  - crates/orchestrator/src/agent_pool.rs (DST tests + supervision)
  - crates/cli/src/live_cli.rs:1429 area (create_handoff path — already wired)
stub_ban:
  - no .unwrap() in saga.rs, idempotency.rs, checkpoint_migrator.rs (production code)
  - no blake2 or sha2 anywhere — HARD RULE Anti-Stale-Training requires blake3
  - no synchronous checkpoint write on hot path (async writer + bounded queue per report anti-pattern)
  - no big-bang schema migration (lazy transform-at-load only)
  - no compensation written AFTER the side effect (logged BEFORE per Azure pattern — verify in test)
  - no random UUID thread_id per retry (stable session_id only)
  - no removing of SCHEMA_VERSION values from the migrator registry — append-only
frontend_pair: crates/dashboard-v2/src/lib/windows/DurabilityPanel.svelte (NEW, optional — shows compensation log status, idempotency cache hit-rate, reliability metrics, last continue_as_new event; gated behind B-26)
observable_effect:
  trigger: simulated crash mid-tool-call via the DST harness in saga_idempotency_e2e
  expect_ssb: "compensation_logged" BEFORE the tool call, then "compensation_replayed" on recovery, then "tool_idempotency_hit" on the retry
  expect_artifact: no double-write on the fixture file; recovery test passes

## DETERMINISTIC_CHECK

```bash
# 1. idempotency module exists with compose function
rg "fn compose.*IdempotencyKey|impl IdempotencyKey" .elai_cc/crates/orchestrator/src/idempotency.rs
# Expected: ≥1 hit

# 2. SAGA logger present, log_compensation requires BEFORE-action ordering (enforced via type/state)
rg "fn log_compensation|fn replay_pending|SagaLogger" .elai_cc/crates/orchestrator/src/saga.rs
# Expected: ≥3 hits

# 3. continue_as_new threshold + invocation
rg "continue_as_new|event_count_ceiling|phase_count_ceiling" .elai_cc/crates/orchestrator/src/session_dag.rs
# Expected: ≥3 hits

# 4. Lazy migrator with registered transforms
rg "fn migrate|register_transform|from_version|to_version" .elai_cc/crates/orchestrator/src/checkpoint_migrator.rs
# Expected: ≥4 hits

# 5. CompactLock present
rg "fn acquire.*CompactLock|CompactGuard|compact_lock" .elai_cc/crates/orchestrator/src/compact_lock.rs
# Expected: ≥2 hits

# 6. All new SSB action types registered
rg "ToolIdempotencyHit|ToolIdempotencyReplayedIrreversible|CompensationLogged|CompensationReplayed|CompensationFailed|SessionDagContinueAsNew|CheckpointLazyMigrated|CompactLockContended|AgentReliabilityMetrics|ToolErrorClassified" .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: ≥10 hits

# 7. No blake2 or sha2 in new files
! rg "blake2|Sha256|sha2::" .elai_cc/crates/orchestrator/src/idempotency.rs .elai_cc/crates/orchestrator/src/saga.rs .elai_cc/crates/orchestrator/src/checkpoint_migrator.rs 2>/dev/null
# Expected: zero hits

# 8. No .unwrap() in new production modules
! rg "\.unwrap\(\)" .elai_cc/crates/orchestrator/src/idempotency.rs .elai_cc/crates/orchestrator/src/saga.rs .elai_cc/crates/orchestrator/src/checkpoint_migrator.rs 2>/dev/null
# Expected: zero hits in non-test contexts

# 9. madsim under dev-dependencies (DST harness)
rg "madsim" .elai_cc/crates/orchestrator/Cargo.toml
# Expected: ≥1 hit under dev-dependencies

# 10. Bench artifact present (Task 7)
test -f workflow/audits/E-44-checkpoint-format-bench-2026-05-19.md
# Expected: file exists with measured size + latency for msgpack vs postcard
```

## ACCEPTANCE

- [ ] Every `mcp_*` and Bash/Edit/Write tool dispatch goes through the idempotency-keyed wrapper (assert via integration test that no direct dispatch exists outside the wrapper)
- [ ] SAGA logger logs compensation BEFORE the irreversible op; type-state pattern OR explicit assertion in test confirms ordering
- [ ] SessionDag triggers continue_as_new when event_count > 500 (default); old DAG archived under blake3 hash
- [ ] AgentCheckpoint loads cleanly across SCHEMA_VERSION 40 → 45 via lazy migrator; no big-bang
- [ ] PreCompact hook + 85% token compact do not race — concurrent invocation test passes; one acquires the CompactLock, the other waits and emits `compact_lock_contended`
- [ ] DST harness (`agent_pool_dst.rs`) injects N crashes at random await points across N runs and the supervisor respects the 3-restart Erlang ceiling; failing run produces deterministic seed-replayable output
- [ ] Postcard feature-flag bench artifact exists with measured size delta + latency delta; recommendation logged but NOT applied as default
- [ ] Reliability metrics emit `agent_reliability_metrics` SSB rows on rolling windows
- [ ] Tool-error classifier emits `tool_error_classified` with one of {transient, semantic, spec-level}; the orchestrator routes recovery accordingly
- [ ] End-to-end test `saga_idempotency_e2e.rs` passes: crash-then-retry produces zero double-effects
- [ ] No new dependencies added without user confirmation (madsim is dev-only; postcard is feature-gated — both flagged in commit)
- [ ] blake3 used for all new hash operations; zero `blake2`/`sha2::` references in new files

## Tests to add

- `crates/orchestrator/tests/idempotency_replay.rs`: same key replays cached result; different key forks
- `crates/orchestrator/tests/saga_before_action.rs`: type-state enforces compensation logged BEFORE side effect
- `crates/orchestrator/tests/saga_recovery_replay.rs`: crash injected after compensation logged but before action; recovery replays inverse
- `crates/orchestrator/tests/continue_as_new_threshold.rs`: 501 events trigger rollover; archive file exists with blake3 hash; new root is fresh
- `crates/orchestrator/tests/checkpoint_migrator_40_to_45.rs`: synthetic v40 checkpoint loads as v45 via lazy chain
- `crates/orchestrator/tests/compact_lock_contention.rs`: two threads contend; one wins, other gets `compact_lock_contended` SSB
- `crates/orchestrator/tests/agent_pool_dst.rs`: madsim DST run with crash injection; 3-restart limit honored
- `crates/orchestrator/tests/error_taxonomy.rs`: each of the 3 buckets correctly classified on fixture errors
- `crates/orchestrator/tests/saga_idempotency_e2e.rs`: full end-to-end smoke per Task 10
- `crates/orchestrator/benches/checkpoint_format.rs`: msgpack vs postcard benchmark
- `tests/contracts/durability_contract.rs`: every new SSB action type is registered; every new module exists; no banned tokens

## Files touched

- CREATE: `.elai_cc/crates/orchestrator/src/idempotency.rs` (~120 LOC)
- CREATE: `.elai_cc/crates/orchestrator/src/saga.rs` (~180 LOC — SagaLogger + SagaAction + type-state)
- CREATE: `.elai_cc/crates/orchestrator/src/checkpoint_migrator.rs` (~150 LOC — registry + transforms for 40→45)
- CREATE: `.elai_cc/crates/orchestrator/src/compact_lock.rs` (~60 LOC — Win/Linux file-lock wrapper)
- CREATE: `.elai_cc/crates/orchestrator/src/error_taxonomy.rs` (~120 LOC — port of failure_taxonomy.py categories)
- CREATE: `.elai_cc/crates/orchestrator/src/reliability.rs` (~100 LOC — RDC/VAF/GDS/MOP rolling window)
- MODIFY: `.elai_cc/crates/orchestrator/src/session_dag.rs` (+~80 LOC — continue_as_new + threshold config)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (+~80 LOC for 10 new action types + `compensation_actions` table — bump SCHEMA_VERSION to next ordinal per memory trajectory)
- MODIFY: `.elai_cc/crates/orchestrator/src/task_runner.rs` (every tool dispatch routed through idempotency wrapper; estimated +~40 LOC delta)
- MODIFY: `.elai_cc/crates/orchestrator/Cargo.toml` (madsim under dev-dependencies; postcard under optional feature `postcard_checkpoints`)
- CREATE: `.elai_cc/crates/orchestrator/benches/checkpoint_format.rs`
- CREATE: `workflow/audits/E-44-checkpoint-format-bench-2026-05-19.md`
- CREATE: `workflow/reasoning/E-44-decisions.md`
- OPTIONAL: `.elai_cc/crates/dashboard-v2/src/lib/windows/DurabilityPanel.svelte` (~150 LOC — gated behind B-26)

## Citations / Grounding

- arXiv:2603.20625 (ACRFence, Mar 2026 UCSC+UConn — semantic-rollback attack against LangGraph/Cursor/Claude Code) — grounds Task 1 — VERIFY ID per memory feedback_paper_citation_verify
- arXiv:2603.29231 (Reliability Decay Curve — RDC/VAF/GDS/MOP) — grounds Task 8 — VERIFY ID
- arXiv:2603.22744 (cited in report's recommendation 9 — 70% recovery rate with structured feedback) — grounds Task 9 — VERIFY ID
- arXiv:2602.21806 (998 LangChain/CrewAI bugs study — top root causes: API misuse, API incompatibility, Documentation Desync) — grounds Task 9 taxonomy
- arXiv:2603.04257 (Memex(RL) — indexed external memory, dereference by index) — grounds the trim+archive pattern in Task 3
- temporal.io/blog/durable-execution-meets-ai-why-temporal-is-the-perfect-foundation-for-ai — grounds Activity model + Continue-As-New
- temporal.io/blog/compensating-actions-part-of-a-complete-breakfast-with-sagas ("AI agent workflows are sagas in disguise") — grounds Task 2
- learn.microsoft.com/en-us/azure/architecture/patterns/compensating-transaction (record reverse for every step, replay in reverse) — grounds Task 2 ordering rule
- dbos.dev/blog/durable-execution-crashproof-ai-agents (auto-generated idempotency keys, Postgres-backed) — grounds Task 1 key composition
- inngest.com/blog/durable-execution-key-to-harnessing-ai-agents (15-30% retry rate, no framework enforces exactly-once at tool boundary) — grounds Task 1 motivation
- github.com/langchain-ai/langgraphjs/issues/536 (lazy schema transform at load) — grounds Task 5
- github.com/madsim-rs/madsim (DST runtime, RisingWave production user) — grounds Task 4
- antithesis.com Series A Dec 2025 ($105M, DST mainstream) — grounds Task 4 motivation
- github.com/jamesmunns/postcard + david.kolo.ski/blog/rkyv-is-faster-than (postcard size + speed) — grounds Task 7
- sqlite.org/wal.html (1000-page WAL checkpoint default) — grounds Task 6 lock substrate
- B-9 SHIPPED (B-PLAN-SUMMARY:217-228 + memory) — grounds the extension surface

## Open questions / risks

- ACRFence arXiv ID needs WebSearch verification per memory feedback_paper_citation_verify (2 bad cites already in B-40/B-42 plans). If 2603.20625 is invalid, replace with the verified Tempo report from Inngest blog as the grounding for Task 1; ACRFence motivation degrades to "GPU floating-point non-determinism is documented in vendor docs" with citation TBD.
- Continue-As-New threshold (default 500 events / 100 phases) is a guess per report Open Question 1. Instrument first — pick the threshold where p99 SQLite write latency exceeds 50ms, not the report's nominal number.
- WAL append granularity (report Open Question 3): every mcp_think? every tool call? every state mutation? Default to every state mutation initially; tune after measuring write rate during a HEAVY-tier task.
- madsim Rust version compatibility: ELAI's MSRV (per Cargo.toml) may not match madsim's; verify before adding dev-dep. If mismatch, postpone DST harness to a follow-up E-N rather than gating E-44 on it.
- Postcard at dashboard IPC boundary (report Open Question 5): the dashboard reads msgpack today. Switching at the IPC layer breaks Tauri command serialization. Keep postcard FILE-only, msgpack IPC-only, document the boundary in `crates/orchestrator/src/serialization.rs` header comment.
- Schema version monotonic vs non-monotonic: memory project_b40/42/41_shipped.md shows the sequence 41 → 44 → 45 (with a 42 pin reservation). The migrator must handle gaps; register `None`-transitions for missing version numbers and never claim "all versions register a transform."
- GPU non-determinism scope (report Open Question 2): if only relevant when attacker controls a tool server (not ELAI's threat model today), idempotency keys are sufficient. If a future C-N adds untrusted external tool servers, the replay-or-fork forking branch becomes load-bearing.

## Next

E-44 unblocks any plan that runs for >1 hour on the remote sidecar without bricking on a tool retry, KV-cache OOM, or schema drift on resume. The Continue-As-New rollover should be followed up with an archive-search CLI (`elai dag list-archives`) so users can re-derive trimmed branches from blake3 hashes — out of scope for E-44 but a natural follow-on. The reliability metrics (Task 8) feed E-43's skill telemetry: a skill that triggers RDC ≥ threshold is a candidate for the dead-weight audit. E-41 (PR review) consumes the SAGA log as the only safety net for the auto-merge guard — if E-41 ever pushes irreversibly, E-44's compensation_actions table is the rollback path.
