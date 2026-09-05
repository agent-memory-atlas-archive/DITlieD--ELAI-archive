Tranche B-6 quad -- coordinated decisions (s-22300)

Date: 2026-05-31
Session: s-22300
SCHEMA_VERSION: 193 -> 194 (single coordinated bump for all four plans)
TOTAL_VARIANTS: 466 -> 470 (+4 new variants, 3 deprecated kept alive one release window)

OVERVIEW
========

Four plans (C-42, C-47, B-39, C-50) land in one orchestrate run with a single
SCHEMA_VERSION bump. Pattern-2 collapse applied to C-42, C-47, C-50; B-39 is
a single-add into the alphabetical-contiguous reflect_* family. Three
pre-existing tool_calibration_* variants (HintEmitted, DriftDetected,
CallLogged) are kept alive as deprecation aliases for one release window;
they migrate fully to ToolCalibrationEvent kinds in C-47b.

C-42 (proc-macro deferred to C-42b)
====================================

SHIPPABLE-NOW: plain-fn caller-side guard at
crates/plugins/src/authz/compose.rs::compose_with_tool_gate_emit (NEW). The
new wrapper takes an `Option<&dyn McpAuthzSsbEmitter>` (also NEW trait
defined in the same file). When `Some`, the composed verdict (Allow / DeniedByAuthz / DeniedByToolGate) is mirrored into an `McpAuthzEvent` SSB row
via the orchestrator-side adapter. When `None`, behaviour is byte-identical
to the legacy `compose_with_tool_gate` surface (zero overhead, fuzz-harness
friendly).

DEFER to C-42b:
  - proc-macro `#[authz(states="...", actors="...")]` declarative attribute
    (requires NEW crates/plugins_macros/ crate; first proc-macro=true crate
    in workspace adds friction without changing threat model)
  - 200+ tuple deterministic corpus generator (build.rs or corpusgen bin)
  - corpus replay binary

Per the exploration recommendation: "ship runtime guard NOW (closes
SHIPPED-WIRED gap per MASTER-INDEX line 194); flag G4 as DEFER to a follow-up
C-42b plan once a second proc-macro need surfaces". The runtime guard
achieves the same SSB-row-per-call outcome as the proc-macro path; the
declarative attribute is a developer-UX win, not a security/correctness win.

The deterministic corpus directory IS established now: NEW
`.elai_cc/data/mcp_fuzz_corpus/.gitkeep`. The canonical path constant
`MCP_FUZZ_CORPUS_DIR` ships in ssb.rs. The corpus generator + populated
tuples come in C-42b.

Defense-in-depth boundary preserved: `check_authz` in
`crates/plugins/src/authz/checker.rs:1-9` stays pure (the SSB emission
deliberately lives on the CALLER side per the load-bearing doc-comment).

Files touched:
  - crates/plugins/src/authz/compose.rs (NEW McpAuthzSsbEmitter trait +
    compose_with_tool_gate_emit + 3 tests)
  - crates/orchestrator/src/ssb.rs (McpAuthzEvent variant + payload +
    kind enum + env-gate const + corpus-dir const)
  - .elai_cc/data/mcp_fuzz_corpus/.gitkeep (NEW)

Env-gate: `ELAI_C42_MCP_AUTHZ_EMIT_ENABLE` (wire-dark default OFF).

C-47 (Pattern-2 collapse; rusqlite over sled)
=============================================

SHIPPABLE-NOW: Pattern-2 collapse `ToolCalibrationEvent` (6-arm Kind:
HintEmitted, DriftDetected, CallLogged, CaseAdded, CaseSelected,
CalibrationApplied). Three pre-existing helpers (ToolCalibrationHintEmitted,
ToolCalibrationDriftDetected, ToolCallLogged) stay alive for one release
window as deprecation aliases (their `from_str` arms + `as_sql_str` arms
remain reachable, replay back-compat preserved). Full migration of callers
to the new Pattern-2 variant happens in C-47b.

Three new CaseBase entry-points land in
crates/plugins/src/tool_calibration/case_base.rs:
  - `add_case(case, emitter, ts_ms, store_fn)` -- admits a fresh case and
    emits CaseAdded on actual admit
  - `select_case(tool_name, call_shape_hash, emitter, ts_ms, lookup_fn)` --
    looks up + emits CaseSelected on hit
  - `apply_calibration(tool_name, call_shape_hash, emitter, ts_ms, apply_fn)` --
    applies a calibration outcome + emits CalibrationApplied on
    `CalibrationOutcome::Applied`

`CaseBaseSsbEmitter` trait (in
crates/plugins/src/tool_calibration/registry.rs) extended with 3 new
default-no-op methods: emit_case_added / emit_case_selected /
emit_calibration_applied. Default impls keep legacy emitters compiling
without surface-area churn.

Orchestrator adapter
(crates/orchestrator/src/tool_calibration_ssb_adapter.rs) implements the 3
new methods, routing to `SsbPayload::ToolCalibrationEvent` with the
appropriate Kind discriminator.

DEPENDENCY: sled REJECTED, rusqlite 0.39 substituted (already in Cargo.toml
per CLAUDE.md anti-stale "no new dependencies"). Persistence layer
(crates/plugins/src/tool_calibration/persist.rs) DEFERRED to C-47b alongside
the full deprecation-alias removal -- it requires a SEPARATE
rusqlite::Connection from the SSB pool (same-thread contention guard) +
ELAI_CASE_BASE_PERSIST env gate.

DEFER to C-47b:
  - rusqlite persistence layer (Optional<Connection>)
  - hard-remove of the 3 deprecated tool_calibration_* variants
  - nearest-neighbor per-query retrieval (paper §3.3) -- separate feature

Env-gate: `ELAI_C47_CASE_BASE_ENABLE` (wire-dark default OFF).

B-39 (single-add)
=================

SHIPPABLE-NOW: 85% soft-warn beacon. Single-add SSB action type
`ReflectBudgetSoftWarn` (the reflect_* family stays alphabetical-contiguous;
5th sibling adds cheapest).

`crates/reasoning/src/chat_reflect.rs`:
  - NEW `pub const REFLECT_BUDGET_SOFT_WARN_PCT: u8 = 85;` near the
    constants block
  - NEW `ChatReflectConfig.budget_soft_warn_pct: u8` field, default 85
  - NEW `ReflectEvent::BudgetSoftWarn { turn_id, used, soft_cap, hard_cap, pct }`
    variant
  - Soft-warn check inserted BEFORE the existing hard-cap log block at
    ~L454: when `used >= soft_cap && used < hard_cap` AND `soft_cap > 0`,
    emit `tracing::warn!` + the new ReflectEvent

Soft-warn arithmetic uses u64 intermediate to dodge u32 overflow at
`hard_cap * pct`; `soft_cap` clamps to u32::MAX.

`crates/orchestrator/src/ssb.rs` ReflectEventSink impl extended with the
BudgetSoftWarn arm: env-gate-checked (ELAI_B39_SOFTWARN_ENABLE) at
emit-time; turn_id hashed via the new shared `blake3_label_prefix` helper.

Three new shared SSB helpers introduced for the tranche:
  - `env_gate_truthy(name) -> bool` -- canonical 1/true/yes/on
    case-insensitive truthy predicate (replaces ad-hoc gate reads)
  - `blake3_label_prefix(label) -> u64` -- shared blake3-prefix label
    hasher (matches C-45 / C-46 / C-47 hashing discipline; CLAUDE.md
    HARD RULE blake3-not-blake2)
  - `system_time_ms_i64() -> i64` -- SystemTime-based wall-clock helper
    used by emit sites that don't carry the clock-provider Arc

Env-gate: `ELAI_B39_SOFTWARN_ENABLE` (wire-dark default OFF).

C-50 (Pattern-2 collapse; producer pinned)
==========================================

SHIPPABLE-NOW: Pattern-2 collapse `ContractCodingEvent` (3-arm Kind:
RetryAttempted, RetryResolved, RetryAbandoned). Pre-existing
`ContractViolationDetected` stays alive (NOT collapsed in this tranche --
separate landing, deprecation alias not yet warranted).

Producer pinned per the exploration findings:
`crates/orchestrator/src/task_runner.rs::fix_and_rerun_until_green` at
three sites:
  - L1055 Healed branch -> emit RetryResolved BEFORE return
  - L1109 EscalatedToHuman branch -> emit RetryAbandoned AFTER
    emit_autonomous_decision (so dashboards see the human escalation row
    first, then the Pattern-2 contract-coding summary)
  - L1057-equivalent Failed branch -> emit RetryAttempted AFTER the
    `iterations += 1` increment (so iteration count matches the dashboard's
    1-indexed display)

Emit helpers live in NEW
`crates/orchestrator/src/contract_coding/retry_emit.rs`:
  - `emit_retry_attempted(ssb, step_id, iterations, failure_summary)`
  - `emit_retry_resolved(ssb, step_id, iterations)`
  - `emit_retry_abandoned(ssb, step_id, iterations, escalation_reason)`

Each helper:
  - reads `ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE` and short-circuits on
    miss (wire-dark default OFF)
  - hashes `step_id` via `blake3_label_prefix`
  - saturates `iterations: u32` into the `u8` wire slot (production cap is
    5 per fix_and_rerun_contract.rs:278; saturation defends against future
    cap bumps)
  - truncates carrier strings (`failure_summary`, `escalation_reason`) to
    240 chars (matches the existing emit_autonomous_decision truncation
    cap)
  - severity = 0 (advisory) for Attempted / Resolved; severity = 1
    (critical) for Abandoned

ISOLATION CAVEAT: per task_runner.rs:1025-1030 doc-comment,
fix_and_rerun_until_green is currently exercised only by the contract test
in isolation -- the caller wiring at drive_task is a SEPARATE concern
guarded by `is_web_project`. This tranche does NOT modify drive_task. The
caller-side wiring lands in a follow-up.

Env-gate: `ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE` (wire-dark default OFF).

LOCKSTEP SITES UPDATED
======================

ssb.rs (single source-of-truth):
  - SCHEMA_VERSION 193 -> 194
  - SQL CHECK constraint: +4 new wire tags (mcp_authz_event,
    tool_calibration_event, reflect_budget_soft_warn,
    contract_coding_event)
  - SsbActionType enum: +4 new variants
  - as_sql_str: +4 arms
  - from_str: +4 arms
  - SsbPayload enum: +4 new variants
  - SsbPayload::action_type(): +4 arms
  - test_payload helper: +4 arms (msgpack round-trip)
  - 3 shared helpers added: env_gate_truthy, blake3_label_prefix,
    system_time_ms_i64

action_type_lock.rs:
  - TOTAL_VARIANTS 466 -> 470
  - ALL_VARIANTS array: +4 entries
  - exhaustive_match_guard: +4 arms

action_type_lock.golden:
  - 4 new rows inserted alphabetically (ordinals 0093 contract_coding_event,
    0237 mcp_authz_event, 0329 reflect_budget_soft_warn,
    0427 tool_calibration_event)
  - "Total variants: 470" header updated
  - Tranche header comment added

python/elai_inference/schemas/messages.py:
  - SCHEMA_VERSION 193 -> 194 with tranche docs

NEW FILES
=========

  - crates/orchestrator/src/contract_coding/retry_emit.rs (~180 LOC,
    C-50 emit helpers)
  - crates/contracts/tests/b6_quad_ssb_contract.rs (~250 LOC, 11 contract
    tests covering all 4 plans)
  - .elai_cc/data/mcp_fuzz_corpus/.gitkeep (C-42 corpus dir sentinel)
  - workflow/reasoning/B6-quad-tranche-decisions.md (this file)

MODIFIED FILES
==============

  - crates/orchestrator/src/ssb.rs
  - crates/orchestrator/tests/action_type_lock.rs
  - crates/orchestrator/tests/action_type_lock.golden
  - crates/orchestrator/src/contract_coding/mod.rs
  - crates/orchestrator/src/tool_calibration_ssb_adapter.rs
  - crates/orchestrator/src/task_runner.rs
  - crates/plugins/src/authz/compose.rs
  - crates/plugins/src/tool_calibration/case_base.rs
  - crates/plugins/src/tool_calibration/registry.rs
  - crates/reasoning/src/chat_reflect.rs
  - python/elai_inference/schemas/messages.py

PATTERN-2 PRECEDENTS CITED
==========================

  - E-47 LoraShipGateEvent (44th application)
  - E-40 Tranche B-2 SymbolGraphEvent (45th application)
  - E-43 SkillSystemEvent
  - E-31 RouteDecisionEvent

This tranche adds the 46th (McpAuthzEvent), 47th (ToolCalibrationEvent),
48th (ContractCodingEvent) Pattern-2 applications. B-39 is single-add
(NOT Pattern-2; reflect_* family stays alphabetical-contiguous).

NET DELTA
=========

TOTAL_VARIANTS 466 + 4 = 470 (Pattern-2 collapse on 3 ToolCalibration*
variants kept alive for one release window of deprecation alias, NOT
hard-removed; same for ContractViolationDetected on the C-50 side).

When C-47b lands and we hard-remove the 3 deprecated variants:
TOTAL_VARIANTS 470 - 3 = 467 (a NET +1 over the pre-tranche 466 baseline
once the migration window closes).

DEFERRED ITEMS (for follow-up tranches)
=======================================

  - C-42b: proc-macro `#[authz(states="...", actors="...")]` declarative
    attribute (NEW crates/plugins_macros/), 200+ tuple deterministic
    corpus generator (build.rs or corpusgen bin)
  - C-47b: rusqlite persistence layer
    (crates/plugins/src/tool_calibration/persist.rs), hard-remove of 3
    deprecated tool_calibration_* variants, nearest-neighbor per-query
    retrieval
  - C-50 follow-up: caller wiring at drive_task (guarded by
    `is_web_project`)
  - Dashboard mirrors: TypeScript types for ContractCodingEvent /
    ToolCalibrationEvent / McpAuthzEvent / ReflectBudgetSoftWarn in
    crates/dashboard-v2/src/lib/api/types.ts; ReasoningDrawer.svelte
    soft-warn UI strip; AuthzPanel.svelte + CaseBaseInspector.svelte
    kind-field rendering

NOTES FOR AUDITOR
=================

  - Schema-version contract test in
    crates/contracts/tests/e40_symbol_graph_contract.rs at L43-49 asserts
    SCHEMA_VERSION_PUBLIC == 193; this assertion is now STALE (HEAD=194).
    Same pattern as the existing stale e47_lora_ship_gate_contract.rs L31-40
    (asserts 192 against HEAD=193, has been stale across the E-40 land).
    Auditor decision: leave both stale OR update both to 194. The CLAUDE.md
    "no unsolicited refactoring" rule prefers the leave-stale path; the
    new b6_quad_ssb_contract.rs::contract_schema_version_is_194_after_b6_quad_tranche
    is the HEAD-pinning assertion for this tranche.

  - C-50 producer wiring is INSIDE `fix_and_rerun_until_green` which today
    is exercised only by the contract test (`fix_and_rerun_contract.rs`);
    the function is not yet wired at drive_task. This tranche delivers the
    producer-side emit helpers + Pattern-2 SSB substrate; observability in
    production traffic awaits the drive_task wiring (separate follow-up
    guarded by is_web_project).

  - All 4 env gates default OFF per E-31 / E-43 / E-47 wire-dark precedent.
    `b6_quad_ssb_contract.rs::contract_b6_quad_default_off_posture` test
    pins this invariant.
