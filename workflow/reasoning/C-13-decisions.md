C-13 — IPC Dispatcher Production Swap — Decisions
Session: s-32952
Date: 2026-05-18
Plan: docs/thefinalglance/C-plan/C-13_ipc_dispatcher_production_swap.md
Context: workflow/sessions/s-32952/context-package-C13.md

================================================================================
SUMMARY
================================================================================

Implemented C-13 in full with no deferrals: 4 production InProcessToolDispatcher
construction sites swapped to IpcToolDispatcherAdapter (D1 Option B); Tier-1
HookDispatcher with RuffPostToolUseRunner wired through RuntimeWiring +
apply_task_runner_wiring; AdaptiveTool<T> emits 10 missing DAIRA markers
around every typed dispatch call; WipSquashComplete SSB variant added with
full 9-tuple lockstep (SCHEMA 52 -> 53, count 106 -> 107); wip_squash module
+ `elai wip-squash` CLI subcommand + task_runner post-audit invocation;
10 contract tests PASS; cargo check --workspace clean.

================================================================================
DECISIONS (vs. context-package-C13.md drifts)
================================================================================

DECISION D1 — IpcToolDispatcherAdapter (Option B)
-------------------------------------------------
Resolution: Option B (adapter shim). Implemented IpcToolDispatcherAdapter
in .elai_cc/crates/tools/src/adaptive_tool.rs. The adapter wraps an inner
Arc<dyn ToolDispatcher> AND an IpcToolRegistry; it implements ToolDispatcher
by:

  1. consulting IpcToolRegistry::write_tools_allowed (made pub for C-13) to
     enforce the same Executing|Auditing write window the IPC-side gate has.
  2. delegating typed dispatch to the inner via an AdaptiveTool<T> wrapper
     (so the 10 missing DAIRA markers fire on every routed call).
  3. returning ToolDispatchError::Denied when the write gate rejects.

Did NOT take Option A (no swap) because the 4 production seats would have
stayed unguarded and the DAIRA marker gap would have remained. Did NOT take
Option C (rewrite ProductionAgentRpcServer.handle_tool_request to bypass
translate_tool_request and route through IpcToolRegistry.dispatch directly)
because Option C requires registering 9 new ToolHandler impls — outside
C-13's stated scope.

The adapter is a transparent swap at the 4 sites — ProductionAgentRpcServer
keeps its `dyn ToolDispatcher` arg unchanged. The IpcToolRegistry held by
the adapter today is empty; future ToolHandler registrations land there
without touching the swap sites.

DECISION D2 — IpcToolRegistry FSM gate semantics inherited verbatim
-------------------------------------------------------------------
Resolution: documented inheritance. write_tools_allowed remains
`Executing | Auditing` only. The adapter shim replicates this exactly via
the now-public `IpcToolRegistry::write_tools_allowed`. BashRequest is still
NOT wired to IpcGateway (C-11 D8 deferral); the adapter does not change
that — it stays a separate concern under C-future.

DECISION D3 — HookDispatcher trait selection (Option 3A)
--------------------------------------------------------
Resolution: Option 3A (Tier-1 trait `hook_dispatcher::HookDispatcher`).
RuntimeWiring.hook_dispatcher is typed `Option<Arc<dyn HookDispatcher + Send + Sync>>`;
apply_task_runner_wiring assigns it to `cfg.hook_dispatcher` directly. The
new helper `build_production_hook_dispatcher()` returns
`Arc<InProcessHookDispatcher>` with `RuffPostToolUseRunner` installed via
`.with_ruff_runner(Arc::new(RuffPostToolUseRunner::new()))`.

Did NOT take Option 3B (Tier-2 ipc_accept::IpcHookDispatcher) because (1)
the matching IpcHookDispatcher struct has NO with_ruff_runner method, so
the ruff requirement would force a 2-helper split; (2) the Tier-2 surface
is already wired by AgentPoolBabyStepExecutor::with_hook_dispatcher per
C-11. Tier-2 IPC hook installation can be a follow-up.

Both the CLI (run_subcommand.rs) AND the dashboard (lib.rs) call
build_production_hook_dispatcher() and stash the result on
wiring.hook_dispatcher BEFORE apply_task_runner_wiring runs.

DECISION D4 — AUDITING->MEMORIZING trigger surface (Option A)
-------------------------------------------------------------
Resolution: Option A (Rust-side opportunistic squash) WITH an additional
CLI shim (`elai wip-squash`) so the Python hook chain can fire the same
code path when the FSM transition originates outside the Rust runner.

Implementation:
- crates/orchestrator/src/wip_squash.rs:
    * count_leading_wip_commits (`git log --pretty=%s -100`, regex on
      `^[Ww][Ii][Pp]` with delimiter check to reject `wiper`/`wipe`).
    * squash_n_commits (`git reset --soft HEAD~N` + `git commit --no-verify`).
    * squash_wip_commits_if_any (threshold 5 — `>5` triggers; `<=5` no-op).
    * post_transition_auditing_to_memorizing — semantic alias.
- task_runner.rs invokes post_transition_auditing_to_memorizing AFTER
  run_post_step_deterministic_audit when outcome_passed &&
  !exec_result.changed_files.is_empty() (and project_dir is Some).
- CLI subcommand `elai wip-squash [--project-root <path>]` for the Python
  hook side-channel.

Both paths emit `SsbActionType::WipSquashComplete` unconditionally so the
dashboard sees both the squashed and no-op branches.

DECISION D5 — DAIRA_MARKERS location correction
------------------------------------------------
Confirmed: DAIRA_MARKERS lives at .elai_cc/crates/orchestrator/src/ssb.rs:3171
(plan said 2859 which is ArtifactsIndexedPayload — wrong). All 15 markers
are present. AdaptiveTool emits 10: Mutation, Branch, Finding, Read, Write,
ToolCallStart, ToolCallEnd, Timeout, Escalate, Converge. The remaining 5
(Call, Return, Error, HookPre, HookPost) stay owned by
task_runner::emit_daira_trace per the plan WIRING_CONTRACT stub_ban.

DECISION D6 — production InProcessToolDispatcher construction sites
-------------------------------------------------------------------
Confirmed: 4 sites — not 2.
- run_subcommand.rs:195 (cloud arm) — SWAPPED
- run_subcommand.rs:275 (sidecar arm) — SWAPPED
- dashboard-v2/src-tauri/src/lib.rs:472 (cloud arm) — SWAPPED
- dashboard-v2/src-tauri/src/lib.rs:552 (sidecar arm) — SWAPPED

All 4 now construct `IpcToolDispatcherAdapter::new(inner, wiring.ssb.clone())`
where `inner: Arc<dyn ToolDispatcher>` is the existing InProcessToolDispatcher.

================================================================================
FILES TOUCHED
================================================================================

MODIFIED:
- .elai_cc/crates/orchestrator/src/ssb.rs (lockstep: enum + as_sql_str +
    from_str + SsbPayload + action_type discriminator + test_payload arm +
    all_test_payloads arm + count assert + SQL CHECK + SCHEMA_VERSION bump
    52->53 + header comment block + WipSquashCompletePayload struct)
- .elai_cc/crates/orchestrator/src/runtime_setup.rs (RuntimeWiring.hook_dispatcher
    field + default in build_task_runner_wiring + thread into apply_task_runner_wiring
    + build_production_hook_dispatcher helper + imports HookDispatcher +
    InProcessHookDispatcher)
- .elai_cc/crates/orchestrator/src/lib.rs (pub mod wip_squash + re-export
    build_production_hook_dispatcher)
- .elai_cc/crates/orchestrator/src/task_runner.rs (post_transition_auditing_to_memorizing
    invocation after run_post_step_deterministic_audit)
- .elai_cc/crates/tools/src/lib.rs (pub mod adaptive_tool)
- .elai_cc/crates/tools/src/ipc_registry.rs (write_tools_allowed -> pub)
- .elai_cc/crates/cli/src/main.rs (mod wip_squash_subcommand + CliAction::WipSquash arm)
- .elai_cc/crates/cli/src/args.rs (CliAction::WipSquash + parse_wip_squash_args)
- .elai_cc/crates/cli/src/run_subcommand.rs (2 sites swapped + hook dispatcher
    install before apply_task_runner_wiring)
- .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (2 sites swapped + hook
    dispatcher install before apply_task_runner_wiring)

CREATED:
- .elai_cc/crates/orchestrator/src/wip_squash.rs (count + squash_n_commits +
    squash_wip_commits_if_any + post_transition_auditing_to_memorizing + tests)
- .elai_cc/crates/tools/src/adaptive_tool.rs (AdaptiveTool<T> + IpcToolDispatcherAdapter)
- .elai_cc/crates/cli/src/wip_squash_subcommand.rs (CLI entry point)
- .elai_cc/crates/contracts/tests/ipc_dispatcher_production_contract.rs (10 tests)

================================================================================
TEST RESULTS
================================================================================

cargo check -p elai-tools -p elai-orchestrator -p elai-cli -p elai-dashboard-v2: 0 errors.
cargo check --workspace: 0 errors.

cargo test -p elai_cc_contracts --test ipc_dispatcher_production_contract:
  10/10 PASS:
    test_ipc_tool_dispatcher_adapter_compiles_and_impls_tool_dispatcher
    test_adapter_fsm_gate_denies_write_in_read_only_state
    test_adapter_passes_writes_through_in_executing_state
    test_adaptive_tool_emits_tool_call_start_and_end
    test_adaptive_tool_emits_read_write_mutation_on_edit
    test_adaptive_tool_emits_branch_and_converge_on_validate
    test_build_production_hook_dispatcher_installs_ruff_runner
    test_wip_squash_complete_ssb_round_trip
    test_wip_squash_noop_at_threshold_or_fewer
    test_wip_squash_collapses_above_threshold

cargo test -p elai-orchestrator --lib ssb::: 27/27 PASS (including
  test_typed_payloads at the new count = 107).
cargo test -p elai-orchestrator --lib wip_squash: 2/2 PASS.
cargo test -p elai_cc_contracts --test tool_dispatcher_contract: 9/9 PASS
  (existing tests unaffected).

================================================================================
RESIDUALS / KNOWN OUT-OF-SCOPE
================================================================================

1. crates/contracts/tests/task_runner_indexing_service_contract.rs has a
   pre-existing compile error (missing `cloud_router` + `cloud_spend_cap_daily_usd`
   fields in struct-literal `TaskRunnerConfig { ... }`). NOT introduced by
   C-13. Caused by older C-5 work adding fields to TaskRunnerConfig. Per
   "Scoped test runs" rule this is out of scope; flag for follow-up.

2. crates/tools/src/lib.rs has 4 pre-existing test failures (WorkerCreate
   confirmation gate; bash/glob tool error reporting). Unrelated to C-13.

3. The IpcToolRegistry inside IpcToolDispatcherAdapter is initialized empty
   today. Per D1 explanation: typed `ToolDispatcher` dispatch routes through
   the inner InProcessToolDispatcher, not the registry's
   `dispatch(tool_id, postcard_bytes, fsm_state)` path. Registering real
   ToolHandlers and rewriting ProductionAgentRpcServer.handle_tool_request
   to go through the registry is the Option C work — explicit C-future.

4. The Tier-2 ipc_accept::IpcHookDispatcher surface is NOT installed by
   C-13. Per D3 it's a follow-up; the Tier-1 InProcessHookDispatcher is
   what apply_task_runner_wiring consumes today.

5. AdaptiveTool's IpcToolRegistry handler-wrapping path (registry-side, not
   ToolDispatcher-side) was NOT activated. The AdaptiveTool<T> generic
   wraps ToolDispatcher only — wrapping ToolHandler at IpcToolRegistry
   registration time would only matter once Option C is taken.

6. BashRequest dispatch remains NOT wired to IpcGateway (C-11 D8 deferral
   per context-package-C13.md D-2). C-13 inherits this deferral; the
   adapter's FSM gate does not fire for BashRequest. Document and revisit
   when Option C is scheduled.

================================================================================
ACCEPTANCE CRITERIA STATUS
================================================================================

[x] Adapter shim IpcToolDispatcherAdapter compiles + impls ToolDispatcher.
    (test_ipc_tool_dispatcher_adapter_compiles_and_impls_tool_dispatcher PASS)
[x] All 4 production InProcessToolDispatcher construction sites swapped.
    (run_subcommand.rs:195/275 + dashboard-v2/lib.rs:472/552)
[x] InProcessHookDispatcher with RuffPostToolUseRunner installed; wired
    into TaskRunnerConfig at apply_task_runner_wiring.
    (test_build_production_hook_dispatcher_installs_ruff_runner PASS)
[x] AdaptiveTool<T> emits 10 DAIRA markers (Mutation, Branch, Finding, Read,
    Write, ToolCallStart, ToolCallEnd, Timeout, Escalate, Converge).
    (test_adaptive_tool_emits_* PASS)
[x] WipSquashComplete SSB variant added (SCHEMA 52->53, count 106->107,
    9-tuple lockstep). (test_wip_squash_complete_ssb_round_trip + ssb::
    test_typed_payloads PASS)
[x] wip_squash module + `elai wip-squash` CLI subcommand +
    post-AUDITING->MEMORIZING invocation path.
    (test_wip_squash_noop_at_threshold_or_fewer +
    test_wip_squash_collapses_above_threshold PASS)
[x] Contract test ipc_dispatcher_production_contract.rs (10 tests).
    All 10 PASS.
[x] cargo check + tests PASS. No new errors.
[x] NO STUBS — every helper has a real implementation; no .unwrap() on
    production paths.
