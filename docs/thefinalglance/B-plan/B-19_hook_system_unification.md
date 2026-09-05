# B-19: Hook System Unification
Objective: pick ONE hook dispatcher (InProcessHookDispatcher OR
IpcHookDispatcher) — delete the other. Today both exist; one is dead.
Expand DAIRA from 2 to 15 markers. Implement ruff PostToolUse subprocess.
Stop hook actually enforces non-IDLE block (currently telemetry-only).
Estimated effort: 1-2 sessions.
Pre-requisites: B-18 (Tier C IPC accept loop) — required if choosing
IpcHookDispatcher. If not, can run earlier.
Parallel-safe with: B-20.

## STATUS (s-25120, 2026-05-23)

REBASELINE — plan body line numbers cited below are STALE by ~2289 lines per
elai.map SCIP_REF; current live line numbers are 5627 / 7032 / 7415 / 7459
in `crates/orchestrator/src/task_runner.rs`. SHIPPED markers tracked inline.

PREREQ-OK — both B-18 (`crates/orchestrator/src/ipc_accept.rs::run_agent_session`
with hook_dispatcher threading at :391-684) AND B-9
(`crates/orchestrator/src/session_dag.rs:1627::SessionDag::create_handoff`)
have shipped, so plan tasks #1 / #3 / #5 are unblocked.

SHIPPED-ON-V2 (7 items, see recheck report):
- HookDispatcher trait + 4 hook methods @ hook_dispatcher.rs
- Stop non-IDLE Block enforcement @ hook_dispatcher.rs:889-910
- DAIRA 15-marker expansion @ ssb.rs:5335 const DAIRA_MARKERS
- AdaptiveTool<T> wrapper @ crates/tools/src/adaptive_tool.rs:112
- Ruff PostToolUse subprocess @ crates/plugins/src/post_tool_use.rs:155
- T1/T2/T3 latency budgets + HookLatencyViolation @ hook_dispatcher.rs:78/84/90
- Circular-trust gate @ crates/rules/src/lifecycle.rs:439-441

SHIPPED-THIS-CYCLE (s-25120):
- MarkerReport SsbActionType (SCHEMA_VERSION 100->101, golden lock 288->298
  including 9 pre-existing missing variants) — task #9 CLOSED
- IpcHookAsyncAdapter (sync IpcHookTrait -> async HookDispatcher bridge) — GAP-1 closed
- PreCompactHandoffSaver trait + dispatch_pre_compact 0.85 token-budget gate
  honouring spec phase-4C.txt:28-29 — task #5 STUB closed
- build_production_ipc_hook_dispatcher() in runtime_setup.rs — NOWIRE-2 closed
  (IpcHookDispatcher non-test constructor available)
- RuntimeSsbFindingWriter production adapter wrapping Arc<Ssb>
- SsbFindingWriter / CrossSessionLockChecker / IpcHookTrait gained Send+Sync
  supertraits so the adapter chain stays object-safe across tasks

DEFERRED:
- InProcessHookDispatcher deletion (plan task #3) — KEEP per
  B-19-decisions.md DECISION-A (it owns ruff + ShortCoder + Tier1/2/3 +
  non-IDLE Block; deleting it would regress 4 SHIPPED surfaces). The
  unification surface is the trait `HookDispatcher`, not the impl.
- task_runner.rs migration to IpcHookAsyncAdapter (plan task #2) — opt-in
  swap available at `RuntimeWiring.hook_dispatcher` callsites; default
  remains InProcessHookDispatcher per DECISION-A.

SPEC DRIFT (documented, not a blocker):
- phase-4C.txt:283 "blake2s" vs code blake3 in ipc_hooks.rs:228 — spec is
  STALE per CLAUDE.md anti-stale rule; code stays on blake3 (canonical).

## Context

Per audit-implementation/phase-4C.md and audit-dashboard/tools-hooks-isolation-wiring.md:
- IpcHookDispatcher (plugins/src/ipc_hooks.rs): ZERO non-test callers.
  Doc-comment admits "Production wiring scheduled in Session wire-4A".
  (B-19 s-25120 update: production constructor now lives at
  `runtime_setup::build_production_ipc_hook_dispatcher`.)
- InProcessHookDispatcher (orchestrator/src/hook_dispatcher.rs): live, fires
  from task_runner.rs at baby-step boundaries (NOT IPC frames).
- PreToolUse runs against natural-language task_summary
  (task_runner.rs:5627 — was :3338; REBASELINE)
- PostToolUse runs per-changed-file (task_runner.rs:7032 — was :3589;
  REBASELINE) WITH ruff (B-19A SHIPPED)
- Stop fires (task_runner.rs:7415-7423 — was :3877; REBASELINE) ENFORCES
  non-IDLE block (B-19 W1 SHIPPED at hook_dispatcher.rs:889-910)
- PreCompact (task_runner.rs:7459 — was :3898; REBASELINE) — DAIRA
  HookPre/HookPost shipped (B-19 W2); 0.85 token-budget gate +
  handoff-saver callback shipped this cycle
- DAIRA EXPANDED to 15 markers per spec (B-19 W2 SHIPPED)
- AdaptiveTool<T> wrapper SHIPPED at crates/tools/src/adaptive_tool.rs:112
- MarkerReport SHIPPED this cycle (SCHEMA_VERSION 101)
- Failure taxonomy 13 categories vs spec 9 (covered by B-20)

## Reads

- audit-implementation/phase-4C.md
- audit-dashboard/tools-hooks-isolation-wiring.md
- F:\.ELAI_workflow\docs\specs\guide\steps\phase-4C.txt
- crates/plugins/src/ipc_hooks.rs (the dead implementation)
- crates/orchestrator/src/hook_dispatcher.rs (the live implementation)
- crates/tools/src/dispatcher.rs (DAIRA marker emission site)

## Tasks

### Decision: pick one hook dispatcher

1. **Decision: keep IpcHookDispatcher (spec-compliant) IF B-18 lands.**
   Rationale: spec required Tier C interception; IPC path is the future.
   With B-18's accept loop, IpcHookDispatcher becomes live. Delete
   InProcessHookDispatcher.
   
   **Alternative: keep InProcessHookDispatcher** if Tier C is not landing
   soon. Then update spec to reflect in-process model. Deprecate IPC
   variant.
   
   **Recommended**: keep IpcHookDispatcher (after B-18 lands). The spec
   intent is correct.

2. **Migrate task_runner.rs hook call sites.** Replace direct
   InProcessHookDispatcher calls at lines 3338 / 3589 / 3877 / 3898
   with IpcHookDispatcher calls (which now route through IPC frame
   interception per B-18).

3. **Delete InProcessHookDispatcher.** Remove the parallel implementation.
   Update tests.

### Stop hook enforcement

4. **Stop hook actually enforces non-IDLE block.** Per spec, Stop hook
   should block termination of running tasks (force user to confirm).
   Currently it's telemetry-only. Refactor to:
   - On Ctrl+C / SIGTERM: Stop hook fires
   - If FSM != IDLE: prompt user "Task running, abort? [y/N]"
   - If yes: orchestrator.abort() + clean-up
   - If no: ignore signal

### PreCompact

5. **PreCompact actual implementation.** Currently no-op stub. Per spec:
   fires when token_used > 0.85. Saves SESSION_HANDOFF before truncation.
   Wire to SessionDag::create_handoff (couples with B-9).

### Ruff PostToolUse

6. **Ruff PostToolUse subprocess.** When PostToolUse runs after a Python
   file write, spawn ruff check on the file. If errors: emit Finding
   + add to retry feedback. Subprocess timeout 5s; cache results by
   file content hash.

### DAIRA markers

7. **Expand DAIRA to 15 markers.** Current 2 ("enter"/"exit"). Add per spec:
   - call (function call)
   - return (function return)
   - mutation (variable assignment)
   - branch (if/else taken)
   - error (panic/unwrap/ok_or)
   - finding (auditor finding emitted)
   - read (file/SSB read)
   - write (file/SSB write)
   - tool_call_start
   - tool_call_end
   - hook_pre
   - hook_post
   - timeout
   - escalate
   - converge

8. **AdaptiveTool<T> wrapper.** Wraps any tool to emit DAIRA markers
   automatically. Reduces per-tool boilerplate.

9. **MarkerReport struct.** Aggregates DAIRA markers per agent execution
   for post-mortem debugging. Stored in SSB.

### Hook latency budgets

10. **T1/T2/T3 hook latency budgets enforced.** Per spec: T1 <1ms, T2
    <10ms, T3 <5ms. Add timing assertions in hook dispatcher; emit
    `hook_latency_violation` SSB row on overrun.

### Circular trust prevention

11. **AUDITOR rules need independent approval.** Per spec C12: when
    self-discovery proposes a rule from auditor findings, require
    independent verification (second auditor instance OR human approval)
    before promotion to ACTIVE. Currently rules.txt §6 says this; impl
    needs verification.

## WIRING_CONTRACT

produces_symbols:
  - IpcHookDispatcher
  - DAIRA_MARKERS
  - MarkerReport
  - RuffPostToolUseRunner
  - dispatch_stop
  - create_handoff
  - HookLatencyViolation
consumes_symbols:
  - task_runner.rs hook call sites (migrated to IPC variant)
  - dashboard ToolsHooksDrawer (richer per-event filter)
  - dashboard latency histogram (per-tier)
reachable_from:
  - dashboard task submit → agent → IPC frames
stub_ban:
  - InProcessHookDispatcher absent (after deletion)
  - DAIRA 2-marker mode disallowed
  - PreCompact no-op disallowed
  - Stop hook telemetry-only disallowed
frontend_pair: B-3 dead UI clean-up + B-6 hook events panel
observable_effect:
  trigger: cd /d F:\.ELAI_workflow\.elai_cc && cargo test -p elai_cc_contracts --test hook_smoke_ssb -- --nocapture
  expect_ssb: hook_event
  expect_artifact: ToolsHooksDrawer per-event filter shows real
                   distribution of 4 hook types

## DETERMINISTIC_CHECK

```bash
# 1. InProcessHookDispatcher absent
rg "InProcessHookDispatcher" .elai_cc/crates/orchestrator/src
# Expected: zero hits.

# 2. IpcHookDispatcher has production callers
rg "IpcHookDispatcher" .elai_cc/crates --type rust --glob '!*test*'
# Expected: ≥1 hit in non-test code.

# 3. DAIRA marker count
rg "DairaMarker::" .elai_cc/crates --type rust
# Expected: 15 distinct variants used.

# 4. Ruff invocation
rg "\"ruff\"" .elai_cc/crates/plugins/src/post_tool_use.rs
# Expected: ≥1 hit.
```

## ACCEPTANCE

- [ ] InProcessHookDispatcher deleted
- [ ] IpcHookDispatcher live in production
- [ ] All 4 hook types fire via IPC
- [ ] Stop hook enforces non-IDLE block (verified end-to-end)
- [ ] PreCompact wired to handoff
- [ ] Ruff PostToolUse subprocess works on Python writes
- [ ] DAIRA 15 markers via AdaptiveTool<T>
- [ ] T1/T2/T3 latency budgets emit violation rows on overrun
- [ ] Auditor-discovered rules require independent approval

## Files touched

- DELETE: crates/orchestrator/src/hook_dispatcher.rs (after migration)
- MODIFY: crates/orchestrator/src/task_runner.rs (4 call sites)
- MODIFY: crates/plugins/src/ipc_hooks.rs (production-ready polish)
- MODIFY: crates/plugins/src/post_tool_use.rs (ruff)
- MODIFY: crates/orchestrator/src/fsm.rs (Stop hook FSM-state-aware)
- MODIFY: crates/orchestrator/src/session_dag.rs (PreCompact integration)
- MODIFY: crates/tools/src/dispatcher.rs (15 DAIRA markers + AdaptiveTool)
- MODIFY: crates/rules/src/* (independent approval gate for auditor rules)
- CREATE: workflow/reasoning/B-19-decisions.md

## Decision doc

- Definitive choice: IpcHookDispatcher vs InProcessHookDispatcher
- Whether ruff is required dep or detected at runtime
- AdaptiveTool<T> design pattern

## Next

B-20 (Tool registry unification).
