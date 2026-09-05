# Verification Report: B-19_hook_system_unification
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-19_hook_system_unification.md
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary

B-19 is substantially implemented: the 15-marker DAIRA vocabulary is defined, Stop hook
enforcement is FSM-state-aware (StopHookGate), T1/T2/T3 latency budgets exist with
emit_hook_latency_violation_to_ssb, ruff PostToolUse runner is implemented in
plugins/post_tool_use.rs, PreCompact dispatches real SSB breadcrumbs, and independent
auditor approval is gated via CircularTrust check in rules/lifecycle.rs. However,
three acceptance criteria remain unmet: InProcessHookDispatcher was NOT deleted (still
the live in-process dispatcher), IpcHookDispatcher has zero production callers (test-only),
and ruff is not installed into the production dispatcher path (with_ruff_runner is never
called outside tests).

## Claims verified DONE

- DAIRA 15-marker vocabulary defined — evidence: .elai_cc/crates/orchestrator/src/ssb.rs:2859 (DAIRA_MARKERS const, 15 entries)
- emit_daira_trace validates marker at every call site — evidence: .elai_cc/crates/orchestrator/src/task_runner.rs:6685 (debug_assert with DAIRA_MARKERS)
- Stop hook enforces non-IDLE block via dispatch_stop — evidence: .elai_cc/crates/orchestrator/src/hook_dispatcher.rs:822-838 (fsm_state check, Block verdict on non-IDLE)
- StopHookGate wired to production Ctrl+C handler — evidence: .elai_cc/crates/cli/src/live_cli.rs:581-610 (gate.observe_signal called on ctrl_c)
- dispatch_stop threads EXECUTING fsm_state in task_runner — evidence: .elai_cc/crates/orchestrator/src/task_runner.rs:6059-6065
- Stop hook tests: FSM-aware double-Ctrl+C logic — evidence: .elai_cc/crates/orchestrator/tests/stop_hook_double_ctrl_c.rs:1-113
- T1/T2/T3 latency budgets defined — evidence: .elai_cc/crates/orchestrator/src/hook_dispatcher.rs:72,78,84 (HOOK_T1_BUDGET_MS=1, T2=10, T3=5)
- emit_hook_latency_violation_to_ssb implemented — evidence: .elai_cc/crates/orchestrator/src/hook_dispatcher.rs:1015; test at .elai_cc/crates/orchestrator/tests/hook_latency_budgets.rs:53
- HookLatencyViolation SSB action type — evidence: .elai_cc/crates/orchestrator/src/ssb.rs:1219
- PreCompact dispatches via dispatch_pre_compact (replaces no-op) — evidence: .elai_cc/crates/orchestrator/src/task_runner.rs:6096-6140
- PreCompact emits HookPre/HookPost DairaTrace breadcrumbs — evidence: .elai_cc/crates/orchestrator/src/task_runner.rs:6086-6095, 6117-6126
- SessionDag::create_handoff wired (85% token threshold path) — evidence: .elai_cc/crates/cli/src/live_cli.rs:1404 (called with fsm_state)
- RuffPostToolUseRunner implemented with 5s timeout, PATH detection, content-hash cache — evidence: .elai_cc/crates/plugins/src/post_tool_use.rs:1-460 (full implementation)
- ruff PostToolUse wired into InProcessHookDispatcher via run_ruff_post_tool_use — evidence: .elai_cc/crates/orchestrator/src/hook_dispatcher.rs:595-660
- Independent auditor approval gate (CircularTrust) — evidence: .elai_cc/crates/rules/src/lifecycle.rs:432-442 (FindingSourceAuditor rejected, SecondAuditor allowed)
- IpcHookDispatcher implemented in plugins/src/ipc_hooks.rs — evidence: .elai_cc/crates/plugins/src/ipc_hooks.rs:206 (struct def)
- PostToolUse dispatches per-changed-file with Block verdict propagation — evidence: .elai_cc/crates/orchestrator/src/task_runner.rs:5694-5753

## Claims PARTIAL

- DAIRA 15 markers via AdaptiveTool<T>
  - exists: DAIRA_MARKERS array at ssb.rs:2859 (all 15 defined); emit_daira_trace uses Call/Return/Error/HookPre/HookPost in task_runner
  - gap: Only 5 of 15 markers are actually emitted in task_runner.rs (Call, Return, Error, HookPre, HookPost). AdaptiveTool<T> wrapper is absent (not found in elai.map or codebase). Mutation, Branch, Finding, Read, Write, ToolCallStart, ToolCallEnd, Timeout, Escalate, Converge markers are defined but never emitted in production code.

- PreCompact wired to SessionDag::create_handoff (per spec: "saves SESSION_HANDOFF before truncation")
  - exists: live_cli.rs:1404 calls create_handoff when token budget > 85%
  - gap: The create_handoff call is triggered by estimated token count in prepare_turn_runtime, not by the dispatch_pre_compact return value. The plan specifies coupling them (PreCompact hook fires -> create_handoff is called). The actual wiring is a separate threshold check, not a direct response to PreCompact findings.

## Claims MISSING

- InProcessHookDispatcher deleted — searched: grep for InProcessHookDispatcher in orchestrator/src; file hook_dispatcher.rs (46 KB, 1186 lines) confirmed present at crates/orchestrator/src/hook_dispatcher.rs:404
- IpcHookDispatcher live in production (non-test callers) — searched: IpcHookDispatcher across all .rs files; only in crates/plugins/src/ipc_hooks.rs, crates/plugins/src/lib.rs, crates/contracts/tests/plugins_hooks.rs, crates/contracts/tests/plugins_adversarial.rs — zero orchestrator/src callers
- ruff runner installed in production dispatcher — searched: with_ruff_runner callers; only in crates/plugins/tests/ruff_post_tool_use.rs (test-only); apply_task_runner_wiring in runtime_setup.rs does not set hook_dispatcher field at all (stays None in production); no production path instantiates InProcessHookDispatcher with a ruff runner
- MarkerReport struct in SSB — searched: MarkerReport in elai.map; no matches
- AdaptiveTool<T> wrapper — searched: AdaptiveTool in elai.map; no matches

## Claims UNVERIFIABLE

- None

## What must be done (action list)

- [ ] Delete crates/orchestrator/src/hook_dispatcher.rs (InProcessHookDispatcher) after migration per acceptance criteria; currently still present
- [ ] Wire IpcHookDispatcher to production: update apply_task_runner_wiring in crates/orchestrator/src/runtime_setup.rs to set hook_dispatcher field using IpcHookDispatcher (requires B-18 IPC accept loop)
- [ ] Install ruff runner in production: instantiate RuffPostToolUseRunner and pass via with_ruff_runner when constructing the production hook dispatcher
- [ ] Implement AdaptiveTool<T> wrapper in crates/tools/src/dispatcher.rs to auto-emit remaining 10 DAIRA markers (Mutation, Branch, Finding, Read, Write, ToolCallStart, ToolCallEnd, Timeout, Escalate, Converge)
- [ ] Implement MarkerReport struct and SSB storage per spec
- [ ] Couple PreCompact hook findings to create_handoff trigger (currently independent threshold check)

## Notes

The plan's DETERMINISTIC_CHECKs confirm the unmet state: check 1 expects zero InProcessHookDispatcher hits (still present), check 2 expects IpcHookDispatcher non-test callers (none found), check 3 expects 15 distinct DairaMarker variants used (not applicable — DAIRA uses string constants not an enum), check 4 expects ruff invocation in post_tool_use.rs (present in plugins but not wired to production). The B-19 stub_ban list (InProcessHookDispatcher absent, PreCompact no-op disallowed, Stop hook telemetry-only disallowed) is 1-of-3: PreCompact and Stop are fixed; InProcessHookDispatcher deletion is blocked on B-18.
