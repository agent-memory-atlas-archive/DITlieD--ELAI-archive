K-trancheK-decisions.md

Session: s-22600 (Tranche K)
Date: 2026-06-01
Scope: K.1 Svelte client-sort drop investigation (E.3 follow-up) + K.2 C-50 non-web retry Option-B wire
Outcome: 1 SHIP (K.2) + 1 INVESTIGATION-NULL (K.1)
SCHEMA_VERSION: 195 (unchanged; Tranche K adds ZERO new SsbActionType variants / ZERO Kind arms / ZERO payload shape changes)

==============================================================================
ITEM K.1 -- Svelte client-sort drop investigation
==============================================================================

DECISION: ZERO Svelte source edits. Full investigation report at workflow/reasoning/K1-svelte-sort-investigation.md.

Summary:
- HeldOutPanel.sourceList @L106: VALUE-sort (count DESC). KEEP -- BTreeMap key-order does NOT replace count-rank.
- CanaryPanel, LockedKeysPanel: per-key dictionary lookup only; no sort to drop.
- PlanningLedgerPanel: firings_by_canary never iterated; reasonHistogram sort is on an unrelated array.

Stub-tracking: B15-dashboard-BTreeMap row in s-22300 stubs.md updated to SVELTE-CLIENT-SORT-INVESTIGATED-NO-EDIT.

==============================================================================
ITEM K.2 -- C-50 non-web retry Option-B wire
==============================================================================

DECISION-1 (W3 resolution): Option (ii) per-mode `c50_emit_gate_truthy(non_web: bool) -> bool` helper, NOT option (i) shared OR-gate.
  Rationale: option (i) would silently widen the existing ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE gate so flipping ONLY the new non-web gate would ALSO start firing web-path emits. Operators must be able to flip each mode independently for staged rollout. The per-mode dispatcher preserves true independence.
  Implementation: new fn `c50_emit_gate_truthy(non_web: bool)` in `orchestrator::contract_coding::retry_emit` selects ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV when non_web=true, else ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE_ENV. The 3 emit fns (emit_retry_attempted / emit_retry_resolved / emit_retry_abandoned) gained a `non_web: bool` arg routed through the dispatcher.

DECISION-2: New env constant
  File: .elai_cc/crates/orchestrator/src/ssb.rs (after L24453)
  Const: ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV = "ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE"
  Wire-dark default; mirrors the canonical Pattern-2 naming convention.

DECISION-3: Second fix_and_rerun_until_green call site
  File: .elai_cc/crates/orchestrator/src/task_runner.rs
  Insert: AFTER the web is_web_project branch close brace (post-L7897), BEFORE the B-8 PipelineGrc block.
  Guard: `!is_web_project && !outcome_passed && env_gate_truthy(ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV)`.
  Validator: re-execute baby_step_executor.execute(exec_ctx_snapshot); Green when BabyStepOutcome::Passed, Failed when Failed/Skipped/Blocked (with reason / blocked_by surfaced as the validator summary).
  step_id namespace: format!("baby_step_{}_non_web", bs_id.0) -- blake3 prefix differs from web row so dashboards can distinguish two rows per baby step when both gates are on (intentional, no collision risk).

DECISION-4: fix_and_rerun_until_green signature change
  Added `non_web: bool` as the 8th argument (after `step_id: &str`). All 3 inner emit calls thread it verbatim. The 2 existing callers (1 prod web site at task_runner.rs:7864; 2 contract tests at fix_and_rerun_contract.rs:217 + 264) pass `false` to preserve web-mode parity. The 1 NEW non-web caller (task_runner.rs new branch) passes `true`.

DECISION-5: BabyStepExecutionCtx clone-into-closure
  Verified: BabyStepExecutionCtx is `#[derive(Debug, Clone)]` at task_runner.rs:756 -- W6 caveat resolved without needing a snapshot helper. The closure captures a fresh ctx snapshot built at the call site (mirrors the original drive_task exec_ctx @L7191 with `escalation_message: None` + `mode_selection: None` because retries reuse STRUCTURED mode unconditionally, per the inner fix_and_rerun_until_green exec_ctx @L1135).

DECISION-6: details_json `non_web: bool` marker
  All 3 emit fns now include a `"non_web": non_web` field in their `details_json` payload. Dashboards can filter / group by mode without re-deriving from the step_id_label suffix. SCHEMA_VERSION is unchanged because `details_json` is `serde_json::Value` (free-form per the B-6 quad payload spec @ssb.rs:24443).

==============================================================================
DELIVERABLES (K.2)
==============================================================================

Files modified:
  1. .elai_cc/crates/orchestrator/src/ssb.rs -- new ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE_ENV const
  2. .elai_cc/crates/orchestrator/src/contract_coding/retry_emit.rs -- new c50_emit_gate_truthy helper + non_web arg on 3 emit fns
  3. .elai_cc/crates/orchestrator/src/contract_coding/mod.rs -- re-export c50_emit_gate_truthy
  4. .elai_cc/crates/orchestrator/src/task_runner.rs -- fix_and_rerun_until_green signature +1 arg; web caller passes false; new non-web branch after L7897 passes true
  5. .elai_cc/crates/contracts/tests/fix_and_rerun_contract.rs -- both existing callers pass non_web=false for web parity

Files created:
  1. .elai_cc/crates/contracts/tests/c50_non_web_retry_contract.rs -- 8 contract tests (env-gate string stability, default-off, truthy synonyms, independence, c50_emit_gate_truthy routing per mode, wire-dark default, web back-compat, schema version pin)

==============================================================================
BUILD VERDICT
==============================================================================

cargo check --workspace --tests : GREEN (Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 32s)
Only pre-existing warnings; ZERO errors; zero warnings introduced by Tranche K files.

==============================================================================
STUB-TRACKING UPDATES (s-22300/stubs.md)
==============================================================================

  - C50-NON-WEB-RETRY-EMIT row: DEFERRED -> WIRED s-22600 trancheK. Resolution: Option-B per W3 finding (option ii per-mode dispatcher chosen over option i shared OR-gate). New env gate ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE; second fix_and_rerun_until_green call site at task_runner.rs (after L7897). retry_emit fns gained non_web arg + dispatcher. 8-test contract file landed.
  - B15-dashboard-BTreeMap row: SVELTE-PENDING -> SVELTE-CLIENT-SORT-INVESTIGATED-NO-EDIT s-22600 trancheK. Conclusion: HeldOutPanel.sourceList is value-sort (count desc) NOT key-sort; other 3 panels don't sort migrated maps. See workflow/reasoning/K1-svelte-sort-investigation.md.

==============================================================================
FOLLOW-UPS (deferred)
==============================================================================

  - DASHBOARD-OWNER-REVIEW (K.1): HeldOutPanel.sourceList UX choice (alphabetical-by-source vs count-DESC ranking) is a UX call; defer to dashboard governance forum (workflow/audits/). Not blocking.
  - W7 (K.2): dashboard ContractCodingEvent consumers (CanaryDrawer / GatesPanel) not enumerated this tranche; verify that the new `non_web: true` rows render correctly in a later tranche.
  - P-2 (K.2): document the two-rows-per-baby-step pattern (when both gates are on) for downstream dashboard consumers so they do NOT assume one row per bs_id.
