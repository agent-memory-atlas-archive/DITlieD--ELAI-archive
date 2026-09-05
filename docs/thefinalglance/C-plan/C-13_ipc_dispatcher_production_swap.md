# C-13: IPC Dispatcher Production Swap — IpcToolRegistry + IpcHookDispatcher + AdaptiveTool<T> + WIP Squash
Objective: complete B-19 and B-20 production wiring — replace InProcessToolDispatcher with
IpcToolRegistry in cli/src/run_subcommand.rs, replace the absent hook_dispatcher field with
IpcHookDispatcher in runtime_setup.rs, install RuffPostToolUseRunner into the production
dispatcher path, implement AdaptiveTool<T> to emit the 10 missing DAIRA markers, and implement
WIP commit squash on AUDITING->MEMORIZING transition.
Estimated effort: 2 sessions.
Pre-requisites: C-11 (sandbox completeness; agent process boundary must be solid before IPC swap).
Parallel-safe with: C-12, C-14, C-15.

## Context

B-19 and B-20 verification reports (2026-05-17, HIGH confidence) confirm three production gaps:

1. InProcessToolDispatcher is the live production dispatcher (run_subcommand.rs:149,221).
   IpcToolRegistry is test-only (crates/tools/tests/tool_dispatch.rs only).

2. IpcHookDispatcher has zero production callers. apply_task_runner_wiring in runtime_setup.rs
   does not set a hook_dispatcher field at all (stays None). with_ruff_runner is only called in
   crates/plugins/tests/ruff_post_tool_use.rs (test-only).

3. AdaptiveTool<T> wrapper is absent from elai.map. Only 5 of 15 DAIRA markers are emitted in
   task_runner.rs (Call, Return, Error, HookPre, HookPost). The remaining 10 (Mutation, Branch,
   Finding, Read, Write, ToolCallStart, ToolCallEnd, Timeout, Escalate, Converge) are defined in
   DAIRA_MARKERS const at ssb.rs:2859 but never emitted.

4. WIP commit squash is entirely absent (no wip_commit / squash_commit / WipSquash anywhere).

RuntimeWiring struct (runtime_setup.rs:55-63) has no hook_dispatcher field; apply_task_runner_wiring
(runtime_setup.rs:172-183) sets only feasibility_prober, layer1a_processor, baby_step_executor.

## Reads

- .elai_cc/crates/cli/src/run_subcommand.rs (InProcessToolDispatcher call sites lines 149,221)
- .elai_cc/crates/orchestrator/src/runtime_setup.rs (RuntimeWiring struct, apply_task_runner_wiring)
- .elai_cc/crates/tools/src/ipc_registry.rs (IpcToolRegistry, FSM gate at lines 112-146)
- .elai_cc/crates/orchestrator/src/hook_dispatcher.rs (InProcessHookDispatcher, with_ruff_runner)
- .elai_cc/crates/orchestrator/src/task_runner.rs (emit_daira_trace call sites, FSM transition)
- .elai_cc/crates/plugins/src/ipc_hooks.rs (IpcHookDispatcher struct at line 206)
- .elai_cc/crates/plugins/src/post_tool_use.rs (RuffPostToolUseRunner full impl)
- .elai_cc/crates/orchestrator/src/ssb.rs (DAIRA_MARKERS const line 2859)
- docs/thefinalglance/verification/reports/B-plan/B-19_hook_system_unification.report.md
- docs/thefinalglance/verification/reports/B-plan/B-20_tool_registry_unification.report.md

## Tasks

1. **Add hook_dispatcher field to RuntimeWiring + apply_task_runner_wiring**
   File: .elai_cc/crates/orchestrator/src/runtime_setup.rs
   - Add field `hook_dispatcher: Option<Arc<dyn HookDispatcher + Send + Sync>>` to RuntimeWiring.
   - Default it to None in build_task_runner_wiring.
   - In apply_task_runner_wiring: set cfg.hook_dispatcher = wiring.hook_dispatcher.clone().
   - Confirm TaskRunnerConfig has a hook_dispatcher field before writing (grep it).
   - Zero .unwrap()/.expect()/.panic!() on this path.

2. **Build production IpcHookDispatcher + install RuffPostToolUseRunner**
   File: .elai_cc/crates/orchestrator/src/runtime_setup.rs (new helper fn)
   Add fn `build_production_hook_dispatcher(ssb: Arc<Ssb>) -> Arc<IpcHookDispatcher>`:
   - Instantiate RuffPostToolUseRunner (from elai_plugins::post_tool_use).
   - Instantiate IpcHookDispatcher via its constructor, call .with_ruff_runner(runner).
   - Return Arc<IpcHookDispatcher>.
   Callers: run_subcommand.rs wiring paths (both at lines ~109 and ~189) call this and set
   wiring.hook_dispatcher = Some(build_production_hook_dispatcher(wiring.ssb.clone())).

3. **Wire IpcToolRegistry in run_subcommand.rs (replace InProcessToolDispatcher)**
   File: .elai_cc/crates/cli/src/run_subcommand.rs
   At lines 149 and 221 (both InProcessToolDispatcher instantiation sites):
   - Replace `elai_tools::dispatcher::InProcessToolDispatcher::new(Arc::new(validator))` with
     `elai_tools::ipc_registry::IpcToolRegistry::new(Arc::new(validator))`.
   - Verify IpcToolRegistry::new signature matches (check ipc_registry.rs before editing).
   - The FSM gate at ipc_registry.rs:112-146 applies automatically; no extra wiring needed.
   Stub-ban: InProcessToolDispatcher must NOT appear in production call sites after this change.
   Feature-gate it for tests: add `#[cfg(test)]` to InProcessToolDispatcher in dispatcher.rs OR
   add a stub-ban compile guard (cfg-check that errors on non-test use).

4. **Implement AdaptiveTool<T> wrapper**
   File: .elai_cc/crates/tools/src/adaptive_tool.rs (new file)
   Struct `AdaptiveTool<T: ToolHandler>` wraps an inner T and intercepts each dispatch call to
   emit the 10 remaining DAIRA markers via emit_daira_trace before/after delegating to inner.
   Marker mapping (deterministic, no guessing):
   - Before dispatch: ToolCallStart
   - On file read path: Read
   - On file write path: Write, Mutation (if content changed)
   - On test assertion path: Branch (always), Finding (on assertion result)
   - On timeout: Timeout
   - On escalate: Escalate
   - On converge/done: Converge
   - After dispatch: ToolCallEnd
   The 5 already-emitted markers (Call, Return, Error, HookPre, HookPost) remain in task_runner;
   AdaptiveTool<T> does NOT duplicate them.
   Expose AdaptiveTool<T> from elai_tools::adaptive_tool; pub use from crate root.
   Register AdaptiveTool in IpcToolRegistry (wrap each registered handler with AdaptiveTool<T>
   at registration time in ipc_registry.rs register() method).
   Verify DAIRA_MARKERS const at ssb.rs:2859 contains all 15 strings before writing emit calls.
   No .unwrap(); all emit paths use if-let or map_err.

5. **Feature-gate or stub-ban InProcessToolDispatcher**
   File: .elai_cc/crates/tools/src/dispatcher.rs
   After IpcToolRegistry is wired as the production dispatcher:
   Add at top of InProcessToolDispatcher impl block:
   `#[cfg(any(test, feature = "in_process_dispatcher"))]`
   This retains it for existing contract tests without polluting production builds.
   Update Cargo.toml for elai_tools to add optional feature `in_process_dispatcher = []`.

6. **Implement WIP commit squash on AUDITING->MEMORIZING transition**
   File: .elai_cc/crates/orchestrator/src/task_runner.rs
   Find the FSM state transition site where AUDITING transitions to MEMORIZING (grep
   "AUDITING\|Auditing\|auditing" and "MEMORIZING\|Memorizing" in task_runner.rs first).
   Add fn `squash_wip_commits(project_root: &Path) -> Result<(), anyhow::Error>`:
   - Run `git log --oneline --no-decorate` via std::process::Command, capture stdout.
   - Count consecutive leading lines matching /^[0-9a-f]+ [Ww][Ii][Pp]/i.
   - If count > 5: run `git reset --soft HEAD~{count}` then
     `git commit --no-verify -m "phase: squash {count} WIP commits"`.
   - If count <= 5: no-op (return Ok(())).
   - Emit wip_squash_complete SSB row with {wip_count, squashed: bool} on success.
   - All git subprocess calls: capture stderr, map non-zero exit to anyhow error.
   - No .unwrap(); no panic.
   Call squash_wip_commits at the AUDITING->MEMORIZING transition site before handing off.

7. **SSB action type: wip_squash_complete**
   File: .elai_cc/crates/orchestrator/src/ssb.rs
   Confirm SsbActionType enum location (grep "SsbActionType" in ssb.rs).
   Add variant `WipSquashComplete` and string constant `WIP_SQUASH_COMPLETE = "wip_squash_complete"`.
   Follow existing pattern for new action types (same derive, same CATEGORY constant).

8. **Contract tests: ipc_dispatcher_production_contract.rs** (10 functions)
   File: .elai_cc/crates/contracts/tests/ipc_dispatcher_production_contract.rs
   - test_ipc_tool_registry_is_production_dispatcher: grep run_subcommand.rs; assert
     IpcToolRegistry present, InProcessToolDispatcher absent in production context
   - test_in_process_tool_dispatcher_is_cfg_test_only: confirm no non-test import
   - test_runtime_wiring_has_hook_dispatcher_field: construct RuntimeWiring; assert
     hook_dispatcher field exists and is type Option<Arc<dyn HookDispatcher>>
   - test_apply_task_runner_wiring_sets_hook_dispatcher: call apply_task_runner_wiring
     with hook_dispatcher=Some(...); assert TaskRunnerConfig.hook_dispatcher is Some
   - test_build_production_hook_dispatcher_returns_ipc_hook_dispatcher: call fn;
     assert returned Arc downcasts to IpcHookDispatcher
   - test_ruff_runner_installed_in_production_hook_dispatcher: assert ruff runner is
     wired (check has_ruff_runner() method or equivalent on IpcHookDispatcher)
   - test_adaptive_tool_emits_tool_call_start_and_end: wrap a no-op handler in
     AdaptiveTool; dispatch once; assert ToolCallStart + ToolCallEnd SSB rows present
   - test_adaptive_tool_emits_read_write_mutation: wrap a file-write handler;
     dispatch; assert Write + Mutation rows emitted
   - test_wip_squash_triggers_at_6_wip_commits: create temp git repo with 6 WIP
     commits; call squash_wip_commits; assert git log has exactly 1 squash commit
   - test_wip_squash_noop_at_5_or_fewer: create temp git repo with 5 WIP commits;
     call squash_wip_commits; assert 5 commits unchanged

9. **Contract tests: adaptive_tool_daira_coverage_contract.rs** (5 functions)
   File: .elai_cc/crates/contracts/tests/adaptive_tool_daira_coverage_contract.rs
   - test_all_15_daira_markers_emitted_across_tool_lifecycle: run a full AdaptiveTool
     dispatch sequence covering read, write, mutation, branch, finding, converge;
     collect SSB rows; assert all 15 DAIRA marker action_type strings appear at least once
   - test_timeout_marker_emitted_on_deadline_breach: inject slow handler; assert Timeout row
   - test_escalate_marker_emitted_on_error: inject error handler; assert Escalate row
   - test_converge_marker_emitted_on_success: assert Converge row on clean return
   - test_adaptive_tool_does_not_duplicate_call_return_error: assert Call/Return/Error
     each appear exactly once (emitted by task_runner, not double-emitted by AdaptiveTool)

## WIRING_CONTRACT

produces_symbols:
  - AdaptiveTool<T> wrapper in elai_tools::adaptive_tool
  - RuntimeWiring.hook_dispatcher field (Option<Arc<dyn HookDispatcher>>)
  - build_production_hook_dispatcher() in runtime_setup.rs
  - squash_wip_commits() in task_runner.rs
  - SsbActionType::WipSquashComplete + WIP_SQUASH_COMPLETE constant
  - IpcToolRegistry as production dispatcher at run_subcommand.rs:149,221
  - #[cfg(any(test, feature="in_process_dispatcher"))] gate on InProcessToolDispatcher
  - 15 contract tests (2 files)

consumes_symbols:
  - IpcToolRegistry (crates/tools/src/ipc_registry.rs; struct + FSM gate)
  - IpcHookDispatcher (crates/plugins/src/ipc_hooks.rs:206)
  - RuffPostToolUseRunner (crates/plugins/src/post_tool_use.rs)
  - with_ruff_runner method on IpcHookDispatcher
  - emit_daira_trace (task_runner.rs, verified present)
  - DAIRA_MARKERS const (ssb.rs:2859; 15 entries verified)
  - C-11: agent process boundary (sandbox must be solid before IPC swap)

reachable_from:
  - elai run / elai /task (run_subcommand.rs both wiring sites)
  - apply_task_runner_wiring (production chokepoint)
  - AUDITING->MEMORIZING FSM transition (wip squash trigger)

stub_ban:
  - InProcessToolDispatcher MUST NOT appear in non-test production code after Task 3
  - hook_dispatcher field MUST be set in apply_task_runner_wiring (not left as None in production)
  - squash_wip_commits MUST use git subprocess (not libgit2 / gitoxide); capture stderr
  - AdaptiveTool<T> MUST NOT duplicate Call/Return/Error markers (task_runner owns those)
  - No .unwrap()/.expect()/panic! in any new production path

observable_effect:
  trigger: elai /task --mode cloud "implement and test a sum function in src/sum.rs"
  expect_ssb:
    - all tool calls routed through IpcToolRegistry (no InProcessToolDispatcher in stack trace)
    - all 15 DAIRA marker action_type strings present in SSB rows
    - hook_post_tool_use row emitted when any .py file is written (RuffPostToolUse fired)
    - after AUDITING->MEMORIZING transition: exactly 1 squashed commit if >5 WIP commits existed

## DETERMINISTIC_CHECK

```bash
# 1. IpcToolRegistry is the production dispatcher; InProcessToolDispatcher absent from production
rg "IpcToolRegistry" .elai_cc/crates/cli/src/run_subcommand.rs | wc -l
# Expected: >= 2

rg "InProcessToolDispatcher" .elai_cc/crates/cli/src/run_subcommand.rs | wc -l
# Expected: 0

# 2. InProcessToolDispatcher feature-gated in dispatcher.rs
rg "cfg.*test.*in_process_dispatcher|in_process_dispatcher.*test" \
  .elai_cc/crates/tools/src/dispatcher.rs | wc -l
# Expected: >= 1

# 3. RuntimeWiring has hook_dispatcher field
rg "hook_dispatcher" .elai_cc/crates/orchestrator/src/runtime_setup.rs | wc -l
# Expected: >= 3 (field def, default, apply site)

# 4. build_production_hook_dispatcher present
rg "fn build_production_hook_dispatcher" \
  .elai_cc/crates/orchestrator/src/runtime_setup.rs | wc -l
# Expected: 1

# 5. AdaptiveTool<T> defined
rg "struct AdaptiveTool" .elai_cc/crates/tools/src/adaptive_tool.rs | wc -l
# Expected: 1

# 6. All 15 DAIRA markers covered in AdaptiveTool + task_runner combined
rg "daira_tool_call_start|daira_tool_call_end|daira_read|daira_write|\
daira_mutation|daira_branch|daira_finding|daira_timeout|daira_escalate|\
daira_converge" .elai_cc/crates/tools/src/adaptive_tool.rs | wc -l
# Expected: >= 10

# 7. squash_wip_commits present in task_runner
rg "fn squash_wip_commits" .elai_cc/crates/orchestrator/src/task_runner.rs | wc -l
# Expected: 1

# 8. WipSquashComplete in ssb.rs
rg "WipSquashComplete|wip_squash_complete" \
  .elai_cc/crates/orchestrator/src/ssb.rs | wc -l
# Expected: >= 2

# 9. Contract tests pass
cargo test -p elai_contracts -- ipc_dispatcher_production adaptive_tool_daira \
  2>&1 | grep "test result"
# Expected: all 15 tests passed

# 10. cargo check clean across affected crates
cargo check -p elai_tools -p elai_orchestrator -p elai_cli -p elai_contracts \
  2>&1 | tail -3
# Expected: 0 errors
```

## ACCEPTANCE

- [ ] IpcToolRegistry is the sole production dispatcher at run_subcommand.rs lines 149,221;
      InProcessToolDispatcher has zero non-test callers (rg confirms)
- [ ] InProcessToolDispatcher feature-gated (#[cfg(any(test, feature="in_process_dispatcher"))]);
      existing contract tests still compile and pass
- [ ] RuntimeWiring.hook_dispatcher field exists; apply_task_runner_wiring sets it from wiring
- [ ] build_production_hook_dispatcher() returns IpcHookDispatcher with RuffPostToolUseRunner wired
- [ ] with_ruff_runner called in production path (not test-only); confirmed by grep
- [ ] AdaptiveTool<T> in crates/tools/src/adaptive_tool.rs; registers as wrapper in IpcToolRegistry
- [ ] All 10 previously-missing DAIRA markers emitted by AdaptiveTool<T>
- [ ] squash_wip_commits triggers on AUDITING->MEMORIZING; >5 WIP commits squashed to 1
- [ ] WipSquashComplete SSB action type added; emitted on squash
- [ ] 15 contract tests pass (10 in ipc_dispatcher_production + 5 in adaptive_tool_daira_coverage)
- [ ] cargo check -p elai_tools -p elai_orchestrator -p elai_cli -p elai_contracts: 0 errors
- [ ] Cloud-Sonnet smoke test PROVEN:
      Run: elai /task --mode cloud "implement and test a sum function in src/sum.rs"
      (a) All tool calls flow through IpcToolRegistry; InProcessToolDispatcher absent in stack trace
      (b) SSB contains all 15 DAIRA marker action_type strings:
          daira_call, daira_return, daira_mutation, daira_branch, daira_finding,
          daira_read, daira_write, daira_tool_call_start, daira_tool_call_end,
          daira_hook_pre, daira_hook_post, daira_timeout, daira_error,
          daira_escalate, daira_converge
      (c) hook_post_tool_use SSB row emitted when any .py file written (RuffPostToolUse fired)
      (d) After AUDITING->MEMORIZING transition: git log shows exactly 1 squashed commit
          (not 5+ WIP commits) when >5 WIP commits preceded the transition

## Files touched

- MODIFY: .elai_cc/crates/cli/src/run_subcommand.rs (lines 149,221: IpcToolRegistry swap)
- MODIFY: .elai_cc/crates/orchestrator/src/runtime_setup.rs (hook_dispatcher field + helper fn)
- MODIFY: .elai_cc/crates/tools/src/ipc_registry.rs (wrap handlers with AdaptiveTool<T>)
- MODIFY: .elai_cc/crates/tools/src/dispatcher.rs (feature-gate InProcessToolDispatcher)
- ADD: .elai_cc/crates/tools/src/adaptive_tool.rs (AdaptiveTool<T>, 10 DAIRA markers)
- MODIFY: .elai_cc/crates/orchestrator/src/task_runner.rs (squash_wip_commits, squash call site)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (WipSquashComplete variant + constant)
- ADD: .elai_cc/crates/contracts/tests/ipc_dispatcher_production_contract.rs (10 tests)
- ADD: .elai_cc/crates/contracts/tests/adaptive_tool_daira_coverage_contract.rs (5 tests)
- CREATE: workflow/reasoning/C-13-decisions.md

## Next

C-14 or C-15 (parallel-safe). Confirm C-11 shipped before opening this session.
