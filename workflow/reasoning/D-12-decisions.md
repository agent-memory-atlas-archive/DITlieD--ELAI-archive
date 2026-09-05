D-12 decisions — OperationalizationCheck baby-step predicate

What

Add `is_operationalized(step)` predicate + `dispatch_check_for_baby_step` routing helper + `run_operational_check` gating function. SCHEMA 80 -> 81 (one new SsbActionType variant: `baby_step_dispatched`). NEW orchestrator/src/baby_step.rs + ADDITIVE modifies to ssb.rs + agent_pool.rs + lib.rs. NEW Tauri command_d12.rs + Svelte BabyStepPanel.svelte + 3 dashboard registry rows.

Grounding

arxiv:2305.16291 Voyager (T1) skill-with-measurable-effect: only retain executable skills with environment effect.
arxiv:2407.01489 AGENTLESS (T1) single-step verification: one checker invocation per claim, no agent re-planning inside the verifier.
docs/specs/guide/real-problem-solving-grounded-2026-05-19.md §4.1 (lines 185-196).

Decisions

(a) field-name rename `expected_result` (not `expected_result_payload`): mirror the markdown surface the babystep-author writes; one struct per markdown block keeps the parser deterministic. Plan called for `expected_result` already; we did not introduce a new name.

(b) `DispatchOutcome` re-exported at the crate root as `BabyStepDispatchOutcome` (not bare `DispatchOutcome`): the orchestrator already had a `structural_retry::DispatchOutcome` re-export. Renaming at re-export site is cheaper than touching D-12 internals and preserves both APIs.

(c) Three-state `DispatchOutcome` (NotOperationalized | Inapplicable | Dispatched) — distinguishes "missing fields" from "fields present but no checker fired". The auditor BLOCKs in both cases but the dashboard renders them differently (the second is a registry-coverage problem, not an author problem).

(d) `BabyStepDispatchedEmitter` + `VecBabyStepDispatchedEmitter` trait pattern mirrors D-11's `CheckerResultEmitter` + `VecCheckerResultEmitter` — production callers wire one Ssb writer adapter under both traits.

(e) Pure stdlib parser (no serde_yaml dep). The MEASURABLE_PROCEDURE block is whitespace-indented blocks of `key: value` lines. The parser is forgiving on blank lines + case-insensitive on the field names. Adding a YAML dep would violate the "no new external dep" policy.

(f) `regex_match` + `json_schema_conforms` ExpectedKind variants are reserved as closed-enum slots — the diff implementations are deferred to D-15. The variants exist in the enum from D-12 day one so the closed surface is complete and downstream callers do not need a schema bump to start emitting the labels.

(g) `run_operational_check` returns `Result<DispatchOutcome, OperationalError>` — the auditor matches on the Err variant to populate the BLOCK reason. `Ok(outcome)` only when every checker returned Pass; Inapplicable + Fail both produce Err.

Test results

cargo check -p elai-orchestrator: green (1 cycle, 19m 46s incl D-11 deps).
Unit tests in baby_step.rs: 9 (predicate / dispatch hint / fallback / not-op / inapplicable / parser-full / parser-empty / parser-missing-title / parser-unknown-kind / outcome-label / kinds-used / serde-roundtrip).
Unit tests in agent_pool.rs d12_dispatch_tests: 3 (pass / not-op-emits-row / fail-non-pass).
Integration tests in operationalization_integration.rs: 4 (end-to-end pass / missing-claim / fail propagation / pure dispatch).

Not-yet-run (deferred to auditor per ORCHESTRATE policy): cargo test --workspace, svelte-check, full SSB round-trip for the new variant (the variant IS included in `all_test_payloads()` so `test_typed_payloads` covers it).

Out of scope

- per-checker procedure templates (deferred to D-15)
- procedure auto-generation from claim text (deferred)
- regex/json-schema expected-result diff implementations (slots reserved, deferred to D-15)
- updates to .claude/agents/babystep-author.md and .claude/agents/auditor.md (workflow tooling territory, not landed in this session per "ADDITIVE in elai_cc only" scope)

Schema bump

SCHEMA_VERSION: 80 -> 81. test_typed_payloads assertion: 170 -> 171.

Closure-pass decisions (s-25120, post-substrate landing)

DECISION-1 (no schema bump for closure): D-12 substrate landed at SCHEMA_VERSION 81; current head is 121 (C-46 UnoOrchestra). The plan blurb's "57 -> 58" bump is plan-stale (eighth+ confirmation of the [SCHEMA-actual vs plan-stale] rule). Closure adds production wiring + agent-doc updates only; no SSB row family added; no SCHEMA bump emitted. Evidence: ssb.rs:1343 SCHEMA_VERSION = 121, ssb.rs:747 historic D-12 row comment.

DECISION-2 (single BabyStepDispatched SSB variant preserved): plan asked for a 2-variant split (`ClaimWithoutProcedure` + `ProcedureExecuted`); shipped consolidates into a single `SsbActionType::BabyStepDispatched` with an `outcome` discriminator (`"pass" | "fail" | "inapplicable"`). Mirrors D-10 DECISION-2 (single `ClaimRouted` over plan's `ModeSelected + ClaimDispatched`) and D-11 DECISION-1 (single `CheckerResultEmitted`). Closure preserves the consolidated variant. Rationale: adding 2 variants would inflate TOTAL_VARIANTS in .elai_cc/crates/orchestrator/tests/action_type_lock.rs from 340 to 342, force regen of action_type_lock.golden, and contradict the trio precedent without adding any signal the discriminator does not already carry.

DECISION-3 (MeasurableProcedure 4-field shape preserved over plan's 5-field): plan asked for `{checker_name, claim_payload, candidate_payload, domain_args, expected_result}`; shipped uses `{steps, success_criterion, checker_kind_hint, max_runtime_ms}` at .elai_cc/crates/orchestrator/src/baby_step.rs:113, with `ExpectedResult` + `Claim` carried as separate `Option<...>` fields on `BabyStep` rather than bounded-string blobs inside `MeasurableProcedure`. Rationale: typed `Claim` (struct with claim_id + text + claim_type + evidence + ts_ms) is structurally richer than `BoundedString<2048>`; separation lets the parser stay deterministic per markdown block; renaming would shatter the integration tests at tests/operationalization_integration.rs + the parser at baby_step.rs:457. Per PLAN-VOCAB-vs-SHIPPED-NAMING (D-9 DECISION-2 precedent).

DECISION-4 (code placement preserved in baby_step.rs + agent_pool.rs over plan's new operationalization.rs module): plan's Files-touched list called for `.elai_cc/crates/orchestrator/src/operationalization.rs` as a new module + `pub mod operationalization` in lib.rs. Shipped consolidates: predicate + dispatch + parser in `baby_step.rs` (single owner of the typed value object), production gate + emitter trait + read surface in `agent_pool.rs` (sibling pattern with D-10 `route_claim` + D-11 `dispatch_constructive_checkers`). Closure does NOT create operationalization.rs as a re-export shim — would add module noise without functional gain. lib.rs re-exports the public surface at lines 760-773 + 1009-1012.

DECISION-5 (Voyager INV-5 = state delta is the canonical operationalization criterion): per workflow/papers/voyager-2305.16291-interface.md, Voyager retains skills iff they produce a detectable environment effect (INV-5). ELAI's analog: a baby-step claim is operationalized iff `is_operationalized(step)` returns true (procedure + expected + claim all present) AND the dispatch through `run_operational_check` produces a typed `DispatchOutcome` whose `overall_pass()` resolves the success criterion. The shipped predicate is conservative (every field MUST be present; false-positive forces author rework, false-negative is the bug D-12 exists to prevent) — matches Voyager INV-5 strict-retention semantics.

DECISION-6 (production caller wired via ELAI_OPERATIONALIZATION_ENABLE env gate): closes GAP-1 from the cross-check-recheck report (run_operational_check had zero non-test callers). Closure adds `OperationalizationRuntimeConfig` + `OperationalizationRuntimeState` at runtime_config.rs and `run_operational_check_hook` at task_runner.rs, invoked from `drive_task` after `dispatch_constructive_checkers_hook`. Default behaviour: no-op (both runtime plumbing + env gate required to fire). Pattern mirrors D-5 BAI (ELAI_BAI_ENABLE), D-10 claim-router (ELAI_CLAIM_ROUTER_ENABLE), D-11 constructive-checker (ELAI_CHECKER_ENABLE). The synthesised per-task BabyStep envelope intentionally carries None-fields so the predicate returns false + the SSB row carries outcome="inapplicable" — durable coverage-gap signal for the dashboard. Real per-claim envelopes land in D-15 (NoveltyClass labels) and downstream babystep-author emits.

DECISION-7 (auditor + babystep-author agent prompts updated): closes GAP-2 from the cross-check-recheck report. babystep-author.md now documents the D-12 operationalization contract with refusal + compliance examples and the canonical `NOT_CLAIM_EMITTING` marker for structurally-non-claim baby steps. auditor.md adds a D-12 OPERATIONALIZATION ENFORCEMENT section that cites shipped APIs by path (is_operationalized @ baby_step.rs:188, dispatch_check_for_baby_step @ baby_step.rs:284, run_operational_check @ agent_pool.rs:2438) and references the single-variant SSB consolidation per DECISION-2. Edits are surgical — surrounding agent definitions preserved verbatim.
