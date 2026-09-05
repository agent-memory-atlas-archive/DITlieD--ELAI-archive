# B-2: FSM PhaseTransition Wiring (the 30-min fix)
Objective: write a `PhaseTransition` SSB row inside `Fsm::transition_with_metrics`
so the dashboard's StatusBar badge, EXECUTING pulse animation,
MonitorWindow.currently text, and ActivityEntry phase_transition renderer
all become live for the first time.
Estimated effort: 30 minutes (~10-line patch + tests).
Pre-requisites: none.
Parallel-safe with: B-1, B-3 onwards.

## V2 REBASELINE (2026-05-07)

This file's V1 dashboard references are historical. Re-resolve against V2:
  V1 path                                          V2 path
  ----------------------------------------------------------------------------
  crates/dashboard/src/ssb_viewer.rs:733-765    -> (no V2 equivalent — V2
                                                    consumer reads SSB via
                                                    AppState.ssb_writer in
                                                    a B-36 Tauri command)
  crates/dashboard/src-frontend/src/StatusBar
    .svelte:308-363                             -> crates/dashboard-v2/src/
                                                    lib/layout/StatusBar.svelte
  crates/dashboard/src-frontend/src/
    MonitorWindow.svelte                        -> crates/dashboard-v2/src/
                                                    lib/windows/MonitorWindow
                                                    .svelte
  crates/dashboard/src-frontend/src/
    ActivityEntry.svelte (phase_transition row) -> NO DIRECT EQUIVALENT — V2
                                                    ActivityWindow is the
                                                    orbit visualizer; FSM
                                                    transitions surface as
                                                    halo on centre node +
                                                    bubble-up event in the
                                                    strip beneath the tree
                                                    (see DOMAIN-CONTEXT §2)
  crates/dashboard/src/commands.rs::submit_task -> crates/dashboard-v2/src-
                                                    tauri/src/lib.rs
                                                    (currently stubbed; B-36
                                                    Session 3 wires it)

The producer side of B-2 (SSB write inside `Fsm::transition_with_metrics`)
is V2-agnostic — it lives in orchestrator, not dashboard. Land that change
exactly as written. The consumer side (StatusBar + MonitorWindow) becomes a
read against SSB through the V2 Tauri command surface (B-36 wires
`get_fsm_state` and emits `phase_transition` events).

## Context

Per the dashboard wiring audit
(audit-dashboard/orchestration-core-wiring.md §5):

> **Single highest-impact fix: ~30 min, ~10-line patch — append
> `writer.write_payload("fsm", 0, SsbPayload::PhaseTransition(...), Vec::new())`
> inside `Fsm::transition_with_metrics` at fsm.rs:639. That one change
> activates StatusBar badge, MonitorWindow.currently, ActivityEntry's
> phase_transition renderer, and the EXECUTING-pulse animation in one shot.**

Today: `StatusBar.svelte` polls `get_fsm_state` every 1000ms; backend
queries `WHERE action_type = 'phase_transition' ORDER BY seq DESC LIMIT 1`.
The ONLY production writers of `PhaseTransitionPayload` are test fixtures
(`ssb.rs:2608, 3823, 4187`) and the DST reference model (`dst.rs:415`).
`Fsm::transition_with_metrics` (`fsm.rs:528-640`) only persists FSM state
to a per-session msgpack file — never writes SSB.

Result:
- StatusBar badge defaults to "IDLE" forever in production
- The elaborate 10-state stateColour/shouldPulse switch
  (StatusBar.svelte:308-357) is dead code
- EXECUTING pulse animation never fires
- MonitorWindow.currently never updates
- ActivityEntry's phase_transition row renderer never rendered

The single A-22 `emit_fsm_transition` writer (`task_runner.rs:2346`) only
fires for web-research entry/exit and emits `fsm_transition` (a different
action_type the StatusBar badge does not read). Worse: it suggests someone
intended this and stopped halfway.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\docs\thefinalglance\audit-dashboard\orchestration-core-wiring.md`
  §3 (StatusBar invisible) and §5 (the fix)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\fsm.rs:528-640`
  (Fsm::transition_with_metrics)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs` search for
  `PhaseTransitionPayload` definition + existing SsbPayload::PhaseTransition
  constructor
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard\src\ssb_viewer.rs:733-765`
  (the consumer query)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard\src-frontend\src\StatusBar.svelte:308-363`
  (the dashboard renderer)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs:2346`
  (the partial A-22 emit_fsm_transition — note action_type used)

## Tasks

1. Read `SsbPayload::PhaseTransition` variant signature and confirm the
   payload schema (`from`, `to`, `ts_ms`, optional `reason`).

2. Locate the `SsbWriter` accessor inside `Fsm`. The Fsm struct should
   either own one or have access via dependency injection. If not present,
   add `ssb_writer: Option<Arc<dyn SsbWriter>>` to the Fsm struct.
   (Make it Optional to avoid breaking tests that construct Fsm without SSB.)

3. Inside `Fsm::transition_with_metrics` at fsm.rs:639, AFTER the existing
   state-update logic and BEFORE the function returns success, append:

   ```rust
   if let Some(writer) = self.ssb_writer.as_ref() {
       let now = self.clock.now_ms();
       let payload = PhaseTransitionPayload {
           from: from_state,
           to: to_state,
           ts_ms: now,
           reason: reason.clone().unwrap_or_default(),
       };
       // Use action_type that the dashboard reads: "phase_transition"
       if let Err(e) = writer.write_payload(
           "phase_transition",
           0,  // sequence assigned by writer
           SsbPayload::PhaseTransition(payload),
           Vec::new(),
       ) {
           tracing::warn!(target = "fsm", "phase_transition SSB write failed: {e}");
       }
   }
   ```

   IMPORTANT: do not block FSM transition on SSB write failure. SSB write
   is OBSERVATION, not a decision input. Use a non-blocking try and log
   on failure.

4. Decide on `action_type` string: dashboard reads `"phase_transition"`
   (per ssb_viewer.rs:733-765). Honor that.

5. Verify the existing partial writer at task_runner.rs:2346 — the one
   that uses `fsm_transition` (different action_type). Either:
   (a) Migrate it to the same `phase_transition` action_type so all
       transitions flow through the same SSB stream, OR
   (b) Add a second StatusBar query that includes both action_types.
   Recommendation: (a). Single source of truth.

6. Update or delete the now-redundant `emit_fsm_transition` if it becomes
   superseded by the in-Fsm emit. If it adds value (e.g., FSM-external
   web research transitions), keep it but use the same action_type.

7. Add SSB writer wiring at the call sites that construct Fsm. Search
   for `Fsm::new` callers and pass through the SSB writer Arc.

8. Repeat for any FSM transitions that bypass `transition_with_metrics`
   (search for direct state mutations like `self.state = ...`). These
   should be rare; if found, refactor to go through the metrics method.

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/fsm.rs::transition_with_metrics
  - crates/orchestrator/src/ssb/payloads.rs::PhaseTransitionPayload
consumes_symbols:
  - crates/orchestrator/src/ssb_viewer.rs::latest_fsm_state
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_context_state
  - crates/dashboard-v2/src-tauri/src/commands.rs::get_fsm_state
  - crates/dashboard-v2/src/lib/components/ContextStatePanel.svelte::get_context_state
reachable_from:
  - crates/cli/src/live_cli.rs::run_async → Fsm::set_ssb_writer → Fsm::transition_with_metrics
  - crates/orchestrator/src/task_runner.rs::build_orchestrator → Fsm::with_ssb_writer
  - crates/orchestrator/src/runtime_setup.rs::build_runtime → Fsm::with_ssb_writer
  - crates/dashboard-v2/src-tauri/src/commands.rs::submit_task → task_runner → Fsm transitions
stub_ban:
  - "phase_transition" action_type must be written from Fsm
    (not just test fixtures)
  - StatusBar badge never permanently "IDLE" when task is running
frontend_pair: B-2 itself (this step wires both producer and consumer
                already exists in Svelte)
observable_effect:
  trigger: cd .elai_cc && cargo test -p elai_cc_contracts --test fsm_phase_transition_smoke_ssb -- --nocapture
  expect_ssb: phase_transition
  expect_artifact: StatusBar badge changes from "IDLE" to "PLANNING" then
                   "EXECUTING" etc. EXECUTING pulse animation visible.

## DETERMINISTIC_CHECK

```bash
# 1. Source check: emit point exists
rg "phase_transition" .elai_cc/crates/orchestrator/src/fsm.rs
# Expected: at least one write_payload call.

# 2. Production writers count >0 outside tests
rg "SsbPayload::PhaseTransition" .elai_cc/crates --type rust \
  --glob '!**/tests/**' --glob '!**/dst.rs' --glob '!**/ssb.rs'
# Expected: at least one hit (the new fsm.rs write).

# 3. Smoke test
elai.bat repl
> /task "create file foo.txt with content bar"
# Observe: StatusBar transitions visible in dashboard within 2s of /task.

# 4. SQL verification
sqlite3 .elai_cc/.elai_cc/sessions/<session>/ssb.db \
  "SELECT seq, action_type, payload FROM ssb \
   WHERE action_type='phase_transition' ORDER BY seq DESC LIMIT 5;"
# Expected: rows with from/to FSM states.
```

## ACCEPTANCE

- [ ] `SsbPayload::PhaseTransition` written on every successful Fsm transition
- [ ] No regression in fsm.rs unit tests (94 tests should still pass)
- [ ] StatusBar badge changes during a real task (visual verification)
- [ ] EXECUTING pulse animation visible during EXECUTING state
- [ ] MonitorWindow.currently text updates (visual verification)
- [ ] ActivityEntry feed shows phase_transition rows for each transition
- [ ] No blocking failure if SSB writer is unavailable (log + continue)

## Tests to add

- `crates/orchestrator/src/fsm.rs` (within #[cfg(test)] mod):
  test_transition_emits_phase_transition_to_ssb — construct Fsm with mock
  SsbWriter, perform transition, assert the writer received the right payload
- `crates/orchestrator/src/fsm.rs`:
  test_transition_succeeds_when_ssb_writer_unavailable — Fsm without
  SSB writer, transition succeeds, no panic
- Update existing fsm tests to pass mock SsbWriter where Fsm constructor
  changed
- Optional: end-to-end dashboard test (Tauri CommandsTest harness)
  asserting get_fsm_state returns the latest transition

## Files touched

- MODIFY: `crates/orchestrator/src/fsm.rs` (~10 lines added, plus optional
  Fsm struct field for ssb_writer)
- MODIFY: `crates/orchestrator/src/task_runner.rs:2346` (consolidate
  emit_fsm_transition to use the same action_type, OR delete if superseded)
- MODIFY: any Fsm::new callers (pass through SSB writer)
- CREATE: `workflow/reasoning/B-2-decisions.md`

## Decision doc

Write workflow/reasoning/B-2-decisions.md noting:
- Decision: Fsm owns Option<Arc<dyn SsbWriter>> vs caller emits per transition
- Decision: action_type singular `"phase_transition"` vs maintaining the
  existing `"fsm_transition"` for web research
- Mitigation: SSB write failure does NOT block FSM transition

## Next

B-3 (Stub Command Eradication). Can also progress B-4..B-7 in parallel.
