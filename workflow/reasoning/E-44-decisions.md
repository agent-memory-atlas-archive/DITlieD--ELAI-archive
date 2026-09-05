E-44 Checkpointing Durability — Decisions Log
Session: s-25120 session 2/4 (2026-05-25)
Plan: docs/thefinalglance/E-plan/E-44_checkpointing_durability.md
SCHEMA_VERSION bump: 135 -> 136 (atomic +5 variants)

What was shipped:
  - 6 durability sub-modules under
    .elai_cc/crates/orchestrator/src/durability/
      * mod.rs (public re-exports)
      * checkpoint_migrator.rs (lazy schema migrator + identity
        transforms for every LEGACY..CURRENT gap)
      * idempotent_dispatcher.rs (Arc<dyn ToolDispatcher> wrapper with
        BTreeMap<u64, CachedToolResult> dedup cache, native async-fn-
        in-trait impl mirroring tool_dispatcher.rs:439-503)
      * compact_lock.rs (fd-lock-backed cross-process advisory lock,
        same Box::into_raw self-reference pattern as
        orchestrator/src/lock.rs:368-414)
      * continue_as_new.rs (NEW wrapper around create_handoff, NOT a
        modification of the existing signature)
      * resume_validator.rs (checkpoint signature + schema + integrity
        validation)
  - AgentCheckpointData schema_version field added with
    serde(default = "default_checkpoint_schema_version") so existing
    on-disk msgpack blobs deserialise at version 40 (the legacy floor)
    and the lazy migrator brings them up to CURRENT on read.
    Constant: LEGACY_CHECKPOINT_SCHEMA_VERSION = 40 at
    checkpoint.rs:67-77; struct field at checkpoint.rs:86-92.
  - 5 SSB action types: CheckpointWritten, CheckpointResumed,
    CheckpointMigrated, IdempotentDispatchSkipped,
    DurabilityLockBlocked. Each is wired across all 9-tuple atomic
    sites: enum variant + as_sql_str arm + from_str arm + SsbPayload
    variant + payload struct + action_type() discriminant arm +
    test_payload helper arm + all_test_payloads list entry + SQL
    CHECK constraint string list. SCHEMA_VERSION 135 -> 136.
    all_test_payloads().len() 320 -> 325 (assertion updated).
  - CLI subverb `elai durability <status|validate-checkpoint|migrate|
    continue-as-new-preview|acquire-lock>` at
    .elai_cc/crates/cli/src/durability_cmd.rs + DurabilityAction enum
    + parser at .elai_cc/crates/cli/src/args.rs + mod declaration in
    main.rs + CliAction::Durability dispatch arm.
  - Contract test
    .elai_cc/crates/contracts/tests/e44_checkpointing_durability_contract.rs
    (14 assertions covering: SCHEMA floor, schema_version field with
    serde(default) back-compat, migrator chain LEGACY->CURRENT,
    downgrade rejection, migrate_checkpoint helper, IdempotentDispatcher
    impls ToolDispatcher (compile-time), call_shape_hash method
    separation, CompactLock try_acquire returns Result,
    continue_as_new is NEW wrapper, validate_checkpoint variants,
    5 SSB variant round-trip, action_type round-trip, WIRE-DARK
    avoidance grep of durability_cmd, determinism).

Decisions:

DECISION-1: SSB SCOPE CUT from 10 variants to 5 essential variants.
  Rationale: the original E-44 plan proposed 10 atomic SsbActionType
  variants (ToolIdempotencyHit, ToolIdempotencyReplayedIrreversible,
  CompensationLogged, CompensationReplayed, CompensationFailed,
  SessionDagContinueAsNew, CheckpointLazyMigrated,
  CompactLockContended, AgentReliabilityMetrics, ToolErrorClassified).
  10 variants would be the largest cluster atomic-bump in cluster
  history — E-37 previously held the record at x9 (audit posture
  0.78 PASSED_WITH_WARNS, 4 WARN). Cluster-cascade risk on a 10x
  bump exceeds the audit budget. The 5 ESSENTIAL variants for E-44's
  cross-check missing-list closure are: CheckpointWritten,
  CheckpointResumed, CheckpointMigrated, IdempotentDispatchSkipped,
  DurabilityLockBlocked. The other 5 (SAGA compensation log,
  reliability metrics, error taxonomy) are SCOPE-CUT to a follow-on
  patch (or full E-N) once paper cites verify per mistakes.md
  cluster #8.

DECISION-2: IdempotentDispatcher composes AFTER ToolReliabilityGate.
  Rationale: per tool_reliability/mod.rs:35-44 the existing chain is
  Authz -> ToolGate -> Calibration -> Reliability -> dispatch.
  Idempotency is a CACHE not a GATE — it returns a previously
  observed result rather than re-deciding whether to allow. The
  natural composition point is BETWEEN Reliability and dispatch:
  Reliability has already settled allow/spill/retry, then
  IdempotentDispatcher looks up the cached result, returning early on
  hit or forwarding to the inner dispatcher on miss.

DECISION-3: Continue-As-New is a NEW pure decision wrapper, not a
  modification of SessionDag::create_handoff.
  Rationale: B-9 (SessionDag Activation, s-59964) contract requires
  create_handoff signature stability for all downstream callers
  (hook_dispatcher::dispatch_pre_compact at line 410, task_runner
  drive_task, cli/src/live_cli.rs:1429). Modifying create_handoff
  would force a cascading signature update across these callers.
  Instead, continue_as_new_if_needed is a pure function over event
  count + phase count + ceilings that returns a decision +
  optional NewSessionHandle. The caller (any of the existing
  create_handoff callers) is responsible for performing the actual
  snapshot write via the existing create_handoff API. This preserves
  the B-9 contract exactly.

DECISION-4: postcard feature gate is REFRAMED — postcard.workspace
  is already a non-optional dep at orchestrator/Cargo.toml:72. The
  E-44 plan §7 proposed adding postcard "behind a Cargo feature
  postcard_checkpoints" but the dep is already present. The
  reframe: postcard_checkpoints (when added in a future patch)
  controls the runtime serialization PATH inside checkpoint.rs, not
  dep inclusion. NOT shipped in this E-44 ship — bench (Task 7) +
  feature gate are SCOPE-CUT to follow-on per DECISION-1.

DECISION-5: fd-lock 4 reused (anti-slopsquatting confirmed).
  Verification: orchestrator/Cargo.toml:104 already declares
  fd-lock = "4". The crate is also a transitive dep of rustyline
  (Cargo.lock L1733-1741, per the B-31 Phase 3 comment block). No
  new external dep is introduced. The alternative `fs2` was REJECTED
  per anti-stale-training (fs2 last published 2020).

DECISION-6: schema_version field uses serde(default) with the
  LEGACY default = 40.
  Rationale: existing on-disk msgpack blobs were written by
  CheckpointManager::write before E-44 — they lack the
  schema_version field. serde(default) is the standard idiom for
  backwards-compatible field addition. The default helper
  `default_checkpoint_schema_version` returns
  LEGACY_CHECKPOINT_SCHEMA_VERSION (40), the assumed earliest
  shipped version. The lazy migrator then chains identity transforms
  up to CURRENT (136) on read. Validated by the contract test
  e44_f2_agent_checkpoint_data_has_schema_version_with_default.

DECISION-7: WIRE-DARK avoidance via `elai durability` CLI subverb.
  Rationale: per E-41 precedent (E40_WIRE_DARK_AVOIDANCE_VIA_CLI),
  every new library module must have a production caller anchored
  at edit time. The natural anchors for durability/* are:
    * task_runner (idempotent_dispatcher wrap at TaskRunnerConfig
      construction) — DEFERRED to follow-on; wiring at task_runner
      sites is out of scope for the cross-check missing-list
      closure
    * hook_dispatcher (compact_lock acquire before create_handoff)
      — DEFERRED to follow-on
    * checkpoint::read_latest (migrator interception) — DEFERRED
  None of those wiring sites land in this E-44 ship. Instead the
  CLI subverb `elai durability` calls every primitive directly:
    * status -> CURRENT_CHECKPOINT_SCHEMA_VERSION + defaults
    * validate-checkpoint -> validate_checkpoint
    * migrate -> migrate_checkpoint
    * continue-as-new-preview -> continue_as_new_if_needed
    * acquire-lock -> CompactLock::try_acquire
  Contract test e44_f13 greps durability_cmd.rs for each primitive
  to assert WIRE-DARK avoidance.

DECISION-8: 3 paper cites DEFERRED to mistakes.md cluster #8.
  Rationale: arXiv:2603.20625, arXiv:2603.29231, arXiv:2603.22744
  all need a WebSearch pass + extracted quote with page-anchor per
  memory feedback_paper_citation_verify. The IDs are structurally
  suspect (arXiv 2603.* numbering shape moved to YYMM.NNNNN after
  the 2024 reformat). Until paper-ingestor verifies the IDs, the
  cites are DEFERRED — modules ship as DESIGN-INTENT only without
  inline paper anchors. The implementation surface remains
  grounded against vendor docs (Temporal / Azure / langgraphjs#536
  + fd-lock README + sibling code patterns).

DECISION-9: 4 msgpack + 1 ASCII-digit timestamp file count.
  Drift fix: the E-44 plan body §5 claims "5 msgpack files per
  agent" but checkpoint.rs:23-29 shows 4 msgpack + 1
  CHECKPOINT_TS_FILE (ASCII u64 seconds). The lazy migrator runs on
  state.msgpack only; CHECKPOINT_TS_FILE is timestamp metadata not
  schema-versioned. The 5 file count is correct; the "5 msgpack"
  claim in the plan is the drift. CheckpointWrittenPayload uses
  file_count: u32 = 5 to reflect the on-disk reality.

DECISION-10: BTreeMap (not HashMap) for the IdempotencyCache.
  Rationale: CLAUDE.md HARD RULE bans HashMap in production for
  determinism. BTreeMap<u64, CachedToolResult> trades O(1) -> O(log
  N) lookup for deterministic key ordering. Session lifetimes
  bound N to ~10^3 calls; the log-factor cost is negligible.

What was NOT shipped (deferred to follow-on):
  - SAGA compensation log (plan Task 2) + Logged<T> type-state
  - madsim DST harness (plan Task 4) + crates/orchestrator/tests/
    agent_pool_dst.rs — madsim MSRV verification not run; deferred
    to follow-on with pre-allocated SSB variant slots per
    gotchas.md MULTI_LAYER_PAPER_DEFERRED pattern
  - Postcard feature-gate bench (plan Task 7) + workflow/audits/
    E-44-checkpoint-format-bench-2026-05-19.md — requires measurement
    pass; deferred
  - Reliability metrics module (plan Task 8) — paper-grounded;
    awaiting cite verification per mistakes.md cluster #8
  - Tool-error 3-bucket taxonomy (plan Task 9) — paper-grounded;
    awaiting cite verification
  - End-to-end saga_idempotency_e2e smoke (plan Task 10) — depends
    on SAGA + idempotency wiring at task_runner site; deferred
  - DurabilityPanel.svelte (plan §WIRING_CONTRACT.frontend_pair) —
    explicitly flagged OPTIONAL in plan, gated behind B-26

Run-pod pending:
  - 1 RUNPOD-PENDING item: resume-from-crash end-to-end test on a
    multi-session corpus with diverse schema versions
    (workflow/audits/E-44-resume-from-crash-2026-05-25.md when run).

Mechanical follow-on (next E-N or patch-N):
  - Wire IdempotentDispatcher into TaskRunnerConfig::tool_dispatcher
    construction at task_runner.rs:2440 + 2750.
  - Wire CompactLock::acquire before SessionDag::create_handoff at
    hook_dispatcher::dispatch_pre_compact (line 410 area).
  - Wire CheckpointMigrator::migrate at CheckpointManager::read_latest
    (line 265) so on-disk blobs auto-migrate on read.
  - Resolve 3 paper cites via paper-ingestor; revive
    reliability.rs + error_taxonomy.rs + saga.rs in the next E-N if
    cites verify, OR record REJECTED in this decisions log if they
    do not.

References:
  - E-44 plan:
    docs/thefinalglance/E-plan/E-44_checkpointing_durability.md
  - Cross-check report:
    workflow/sessions/s-25120/cross-check-E/
    E-44_checkpointing_durability-report.md
  - Context package:
    workflow/sessions/s-25120/completion-drive/contexts/
    E-44_checkpointing_durability-context.md
  - Exploration:
    workflow/sessions/s-25120/completion-drive/exploration/
    E-44_checkpointing_durability-exploration.md
  - Mistakes log cluster #8:
    workflow/sessions/s-25120/mistakes.md (E-44 section)
  - Sibling fd-lock pattern:
    .elai_cc/crates/orchestrator/src/lock.rs (IndexLock)
  - Sibling tool dispatcher trait:
    .elai_cc/crates/core/src/tool_dispatcher.rs:439-503
  - B-9 SessionDag (preserved):
    .elai_cc/crates/orchestrator/src/session_dag.rs:1627 (create_handoff)

---

E-44 Tranche 7A + 7C — saga + reliability + error_taxonomy (2026-05-31)
Session: s-21636 tranche-7 (E-44 follow-on)
SCHEMA_VERSION bump: NONE — 5 reserved DurabilityEventKind arms
already shipped at SCHEMA 186 (prior night-run session).

What was shipped this tranche:
  - NEW crates/orchestrator/src/durability/saga.rs (~370 LOC)
    + phantom-typestate SagaTicket<Pending|Committed|Failed>
    + SagaLogger + SagaAction + SagaError (thiserror)
    + 5 unit tests inside #[cfg(test)] mod tests
    + compile_fail doctest pinning the negative-space ordering
      invariant (cannot turn Pending into Committed without
      log_compensation)
  - NEW crates/orchestrator/src/durability/reliability.rs (~180 LOC)
    + ReliabilityWindow (VecDeque, window_ms eviction)
    + compute_metrics + emit_rollup
    + ReliabilityMetrics { success_rate, p50_latency_ms,
      p95_latency_ms, sample_window }
    + 2 unit tests
  - NEW crates/orchestrator/src/durability/error_taxonomy.rs
    (~230 LOC)
    + ErrorClass (6 arms: TransientNetwork / RateLimited /
      AuthFailure / InvalidInput / ToolBug / Unknown)
    + ToolErrorClassifier (case-insensitive substring match)
    + emit_classification + truncate_message helper
    + 4 unit tests
  - MODIFIED crates/orchestrator/src/durability/observability.rs
    (APPEND-only: 5 new notify_* helpers identical-shape to the 5
    shipped last session)
  - MODIFIED crates/orchestrator/src/durability/mod.rs
    (pub mod saga / reliability / error_taxonomy + re-exports)
  - MODIFIED crates/orchestrator/src/task_runner.rs
    (run_classify_dispatch now calls
    e44_emit_tool_error_classified when ELAI_E44_SAGA_ENABLE is on;
    e44_record_reliability_outcome + e44_run_saga_step helpers
    available behind the env gate, wired-dark by default)
  - NEW crates/orchestrator/tests/e44_saga_contract.rs (~150 LOC)
    + 5 contract tests covering all 5 producer wires

Decisions:

DECISION-7A-1: phantom typestate over runtime SagaState tag.
  Rationale: VALIDATION WARNING HIGH-2 in the tranche-7 context
  package flagged the ordering invariant as the highest-risk part
  of the saga design. A runtime tag relies on a contract test to
  catch ordering violations; a phantom typestate lifts the check to
  compile time. The exchange consumes the Pending ticket so there is
  NO public path from SagaTicket<Pending> to SagaTicket<Committed>
  except through SagaLogger::log_compensation. The negative-space
  compile_fail doctest pins this. Alternative considered (rejected):
  runtime tag + contract test — same correctness story but
  rationalised by "the test will catch it"; phantom typestate is
  always-on enforcement with zero runtime cost.

DECISION-7A-2: drive_task integration over a separate saga executor.
  Rationale: agent_pool.rs already owns process lifecycle; a
  separate saga executor would duplicate the dispatch-tracking state
  already held by drive_task and break the "single owner per tool
  call" rule observable in idempotent_dispatcher's BTreeMap cache.
  The integration in this tranche is the env-gated
  e44_emit_tool_error_classified call from run_classify_dispatch +
  the e44_run_saga_step + e44_record_reliability_outcome helpers
  staged for Tranche 7D wiring. Full drive_task pre/post tool
  dispatch wiring is gated on Tranche 7D (E2E) because the
  drive_task body is 6_000+ lines and a wider edit there is an
  anti-spiral signal per CLAUDE.md.

DECISION-7A-3: NO SCHEMA bump — reuse the 5 reserved Kind arms.
  Rationale: prior session shipped the 10-arm DurabilityEventKind at
  SCHEMA 186 with 5 arms reserved for these producers. The wire
  format does not change; only the in-source emit-call-site count
  changes. The Pattern-2 single-variant invariant holds (38th
  application).

DECISION-7A-4: Pattern-2 strict carry — no per-Kind payload structs.
  Rationale: 5 new producers would have introduced 5 new payload
  structs in a naive design. Pattern-2 mandates a single payload
  carrier (DurabilityEventPayload) with details_json: serde_json::Value
  carrying the per-Kind scalars. All 5 new notify_* helpers honour
  this by building serde_json::json!({...}) inline. Confirmed
  matches the 5 helpers shipped last session.

DECISION-7A-5: ELAI_E44_SAGA_ENABLE — wire-dark default OFF gate
  at the producer call site.
  Rationale: the observability layer already has its own gate
  (ELAI_DURABILITY_OBSERVABILITY_ENABLE) that short-circuits the
  notify_* helpers. The task_runner.rs integration adds a second
  gate (ELAI_E44_SAGA_ENABLE) so the EXTRA work of running the
  ToolErrorClassifier + reliability window updates does not run on
  every classify dispatch in production. Both gates must be ON for
  rows to land; either OFF makes the integration inert.

DECISION-7A-6: blake3 only for saga_id / action_id derivation.
  Rationale: CLAUDE.md HARD RULE anti-stale-training. Used in
  SagaAction::derive_action_id via blake3::Hasher (NEVER blake2 /
  sha2). Mirrors idempotent_dispatcher::call_shape_hash.

DECISION-7A-7: BTreeMap for the pending-actions map.
  Rationale: HashMap is banned per CLAUDE.md. BTreeMap<(SagaId,
  String), SagaAction> is deterministic and the cardinality is
  bounded by the in-flight irreversible op count per session
  (typically <100).

Still deferred (Tranche 7B + 7D):
  - madsim DST harness (Tranche 7B) — separate sub-phase
  - saga_idempotency_e2e (Tranche 7D) — depends on 7A + 7B
  - full drive_task pre/post tool dispatch saga integration —
    requires careful review of the 6_000-line drive_task body to
    pick the right dispatch sites without touching the
    IdempotentDispatcher-cached path

Build status:
  - cargo check -p elai-orchestrator: PASS (1m 04s)
  - cargo check -p elai-orchestrator --tests: PASS (3m 32s)
  - cargo check --workspace: PASS (3m 46s)

Audit notes for follow-on:
  - 1 dead_code warning on e44_record_reliability_outcome — by
    design; the helper is staged for Tranche 7D drive_task
    integration. Suppressed via #[allow(dead_code)].
  - The compile_fail doctest in saga.rs validates the typestate at
    `cargo test --doc`; the contract test file
    e44_saga_contract.rs validates the runtime behaviour.
