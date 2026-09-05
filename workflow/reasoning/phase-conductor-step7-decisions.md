# Conductor Step 7 Decision Doc

Date: 2026-06-19
Context: graduate the durable backbone from in-process SQLite loops to Temporal workflows/activities.

## What
Add an opt-in `backbone = "sqlite" | "temporal"` config flag. When `sqlite`, the existing
`run_task_loop` / `run_wave_loop` / `run_backlog_waves` / `loop_until_dry` code runs unchanged.
When `temporal`, the conductor starts a Temporal client and executes durable workflows that wrap
idempotent activities backed by the same SQLite schema and state.

## Key decisions
1. **Temporal server target**: local dev server (`temporal server start-dev`). The CLI is installed at
   `F:\.ELAI_workflow\tools\temporal-cli\temporal.exe` and the Python SDK is `temporalio==1.29.0`.
2. **Activity mapping** (thin wrappers around existing code):
   - `schedule_frontier` -> `DependencyFrontierScheduler.schedule_frontier`
   - `lease_task` -> `LeaseManager.acquire_ready`
   - `dispatch_worker` -> `WorkerAdapter.dispatch` + `record_worker_result` via scheduler
   - `run_gates` -> existing `gate_runner.run_gates`
   - `integrate_verified_task` -> `SerializedIntegrator.integrate_verified_task`
   - `record_event` -> `ControlDB.append_event`
   - `compact_epoch` -> `conductor_loop._compact_epoch`
   - `set_active_wave` -> `ControlManager.set_active_wave`
   - `update_task_status` -> `ControlDB.update_task_status` (needed for gate-fail transitions)
3. **Workflow mapping**:
   - `RunWaveWorkflow` schedules a frontier, executes a child `RunTaskWorkflow` per lease, and compacts an epoch.
   - `RunTaskWorkflow` runs dispatch -> gates -> integrate -> status updates.
   - `LoopUntilDryWorkflow` runs waves in `LOOP_UNTIL_DRY_ORDER` until two consecutive dry passes.
   - `BacklogWorkflow` runs `WAVE_ORDER` waves sequentially.
4. **Concurrency**: child workflows per lease are started asynchronously and awaited; the actual SQLite
   operations still serialize on the DB, mirroring the existing in-process behavior.
5. **Testing**: use `temporalio.testing.WorkflowEnvironment.start_time_skipping()` for fast unit tests,
   and `temporal server start-dev` for operational smoke tests.

## Actual test outcome
- `python -m unittest discover -s tests` passes 68 tests (65 SQLite + 3 Temporal).
- `test_temporal_backbone.py` passes `RunWaveWorkflow`, `LoopUntilDryWorkflow`, and
  `BacklogWorkflow` against the `temporalio.testing` time-skipping server.
- `python_wire_check.py` PASS (DARK-TYPE warnings on internal dataclasses only; no DARK public
  functions after renaming the test-only provider setter to `_set_worker_provider`).
- `maintainability_check.py` 0 BLOCK / 0 WARN.

## Unknowns
- Operational smoke against a real `temporal server start-dev` instance is not yet automated.
