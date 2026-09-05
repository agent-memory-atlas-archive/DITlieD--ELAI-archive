C-50-trancheB-decisions.md

Session: s-22600 Tranche B ITEM B.2 (C-50 drive_task call-site wiring)
Date: 2026-05-31
Status: DEFERRED with rationale (Option-C chosen; preserves existing semantics)

Question

Where to fire ContractCodingEvent::Retry{Attempted,Resolved,Abandoned} for the NON-web baby-step boundary when outcome_passed=false. Today the 3 retry_emit helpers fire ONLY inside fix_and_rerun_until_green (task_runner.rs:1058 / 1123 / 1159) which has a SINGLE production caller at task_runner.rs:7834-7843 guarded by `is_web_project && !outcome_passed`. Non-web baby steps emit ZERO retry rows.

Decision: Option-C (defer + document)

Keep the existing is_web_project guard. Non-web retry-event emission stays at zero rows. Rationale:

1. C-50 PLAN SCOPE MISMATCH.
The C-50 plan in docs/thefinalglance/C-plan/C-50_contract_coding_test_scaffolding.md is about Contract-Coding (arXiv:2604.13100) -- precondition / postcondition emission for code-completeness, NOT generic baby-step retry observability. The SSB action types the plan calls for (per §6) are:
  - contract_emitted
  - contract_violation_detected
  - contract_retry_resolved
These are DIFFERENT from the 3-arm ContractCodingEventKind shipped in Tranche B-6 (RetryAttempted / RetryResolved / RetryAbandoned). The B-6 wire-substrate landed under the ContractCodingEvent SsbActionType umbrella but the call-site semantics (validator-loop retries inside fix_and_rerun_until_green) do not match the C-50 plan's intent (per-function contract violation retries inside code-completeness).

2. OPTION-A REQUIRES STRUCTURAL CHANGE BEYOND WIRING.
Option-A (drive_task baby-step retry-boundary emit unconditional) requires ADDING a retry loop at drive_task level. Drive_task today dispatches a baby step ONCE -- there is no iteration counter at that scope (the only retry semantics live inside fix_and_rerun_until_green). Threading "retry attempted" at drive_task would require introducing a NEW retry loop, which:
  - changes ContractCodingEvent semantics from "validator-loop retries" to "baby-step retries"
  - touches a hot-path control-flow boundary
  - has no plan authorization (C-50 plan does not call for drive_task retry loops)
This is scope widening, not wiring.

3. OPTION-B (LIFT GUARD) BREAKS THE VALIDATOR CONTRACT.
Option-B (lift is_web_project guard so fix_and_rerun_until_green runs unconditionally) would call the Lighthouse-CWV validator on non-web projects, which fails immediately on absent build output -- producing spurious RetryAbandoned rows on EVERY non-web baby-step failure. The is_web_project guard is load-bearing: fix_and_rerun_until_green's validator closure (task_runner.rs:7741-7833) hard-codes `npm run build` + Chrome probe + Lighthouse + dev-server -- a web-validator pipeline. Lifting the guard without supplying a parallel non-web validator (e.g. CargoCheckValidator, NoOpValidator) would inject noise into the ContractCoding telemetry, not signal.

4. CONTRACT TEST PROTECTION.
crates/contracts/tests/b6_quad_ssb_contract.rs asserts:
  - SCHEMA_VERSION_PUBLIC >= 194
  - SsbActionType::ContractCodingEvent round-trips via from_str("contract_coding_event")
  - ContractCodingEventKind label stability (retry_attempted / retry_resolved / retry_abandoned)
  - ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE env-gate name stability
  - env_gate_truthy default-off posture
The contract does NOT pin a call-site literal. So either Option-A or Option-B would be contract-test-clean, but neither is semantically correct without C-50 plan-author authorization.

5. ZERO-EMIT IS OBSERVABLE.
Today the C-50 dashboard panel (ContractCodingPanel, wire-dark unless ELAI_C50_CONTRACT_RETRY_EMIT_ENABLE=1) shows zero rows for non-web projects. This IS the correct observable state given that ContractCodingEvent semantics today = "web-validator-loop retries". A user running a non-web project who sees zero rows is correctly informed that no retry telemetry is being captured for their workload. Spurious row injection (Option-B) would be worse than zero rows.

Resolution

C-50 drive_task call-site wiring stays as-is. The is_web_project guard at task_runner.rs:7737 remains. fix_and_rerun_until_green stays the canonical emit site. Non-web retry-event coverage is a separate follow-up that needs:

(a) explicit C-50 plan author authorization to widen ContractCodingEvent semantics OR introduce a parallel non-web validator, OR
(b) a NEW plan that ships a non-web validator (CargoCheckValidator / pytest validator / etc) + threads it into a parallel fix_and_rerun_until_green call site outside the is_web_project guard, with its own env-gate (e.g. ELAI_C50_NON_WEB_RETRY_EMIT_ENABLE) so the rollout is wire-dark and reversible.

Follow-up registration

Adding stubs.md row C50-NON-WEB-RETRY-EMIT to track the deferral. RUNPOD-PENDING / awaiting-plan-author is the will_resolve_in marker.

Grounding

- workflow/sessions/s-22600/exploration/trancheB-integration.md SUB-2 (load-bearing semantic-drift question, contract-test re-check recommendation).
- workflow/sessions/s-22600/context-package-trancheB.md ITEM B.2 (Option-A/B/C decision matrix).
- .elai_cc/crates/contracts/tests/b6_quad_ssb_contract.rs (read in full at decision time; no call-site literal pin).
- .elai_cc/crates/orchestrator/src/contract_coding/retry_emit.rs:1-12 doc-comment (substrate intent: wire-dark by default; callers MUST NOT re-gate).
- docs/thefinalglance/C-plan/C-50_contract_coding_test_scaffolding.md §6 (intended SSB action types: contract_emitted / contract_violation_detected / contract_retry_resolved -- DIFFERENT from the 3-arm B-6 shipped variants).
- CLAUDE.md: "No unsolicited refactoring. No cleanup while working on something else." + "No removal or simplification without explicit user confirmation" (applies to introducing new validator surface).
