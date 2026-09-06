ACP IMPORT NOTES

Compared against `coleam00/agent-control-plane` at GitHub HEAD `7fd7b55` and the local Conductor working tree on 2026-06-25.

Decision: import ACP's product surface, run-history shape, and deterministic test harness ideas. Do not import its authority model.

KEEP THESE CONDUCTOR INVARIANTS

- `state/control.db` remains the source of truth for tasks, leases, attempts, gates, integrations, artifacts, events, budgets, and control flags.
- `scheduler.py`, `leases.py`, `gate_runner.py`, and `integrator.py` remain the authority path for work selection and completion.
- Worker output can create a patch, notes, and artifacts, but never `DONE`.
- `DONE` remains owned by off-agent gates plus serialized integration plus post-integration verification.
- `elai-repo/` remains the isolated integration target. ACP has no equivalent safety boundary.
- Concurrency stays lease-driven. No prompt-directed fanout may bypass disjoint write-set checks, model cooldowns, cargo gate caps, or integration locking.
- The dashboard may expose controls, but the backend must keep fail-closed semantics.

IMPORT 1: RUN LINEAGE AND HISTORY FIELDS

ACP has a clean `loops` / `runs` / `run_events` shape with `role`, `parent_run_id`, `reasoning`, `input_tokens`, `output_tokens`, `cost_usd`, and session-style run history. Conductor already has stronger task truth, but the operator history is harder to read because task attempts, events, gate runs, integrations, and artifacts are split across tables.

Import the lineage idea, not the `loops` table.

Suggested Conductor target:

- Add migration `005_attempt_lineage.sql`.
- Extend `task_attempts` with:
  - `role TEXT NOT NULL DEFAULT 'worker'`
  - `parent_attempt_id TEXT`
  - `caused_by_event_seq INTEGER`
  - `provider_session_id TEXT`
  - `input_tokens INTEGER`
  - `output_tokens INTEGER`
  - `cost_usd REAL`
  - `decision_summary TEXT`
- Add a narrow `attempt_events` table for provider stream events that should not pollute the canonical task event log:
  - `attempt_event_id TEXT PRIMARY KEY`
  - `attempt_id TEXT NOT NULL`
  - `task_id TEXT NOT NULL`
  - `event_type TEXT NOT NULL`
  - `payload_json TEXT NOT NULL`
  - `created_at TEXT NOT NULL`

Implementation notes:

- `worker_adapter.py` should populate token, cost, session, and event fields when a provider exposes them.
- `conductor_db.py` should expose one query that returns a full task timeline: task status changes, attempts, attempt events, gate runs, integrations, and artifacts in timestamp order.
- `events.py` should stay append-only for canonical conductor state. `attempt_events` is for provider telemetry and can be retained or pruned separately.

Acceptance:

- Given a task id, one API call can explain who ran, what model ran, what it cost, what the worker said, what gate failed, and what artifact proves it.
- Missing token or cost data never fails a task. Store nulls.
- Retrying or escalating a task links the new attempt to the prior failed attempt.

IMPORT 2: FAKE WORKER HARNESS

ACP's `ACP_FAKE_PI=1` mode is valuable because it tests loop control without spending model calls or relying on provider behavior. Conductor has provider abstractions already, but it should have a first-class deterministic fake worker path for runtime control tests.

Suggested Conductor target:

- Add `FakeWorkerProvider` beside `SubprocessWorkerProvider` in `worker_adapter.py`, or a new `tests/fake_worker_provider.py` if keeping production code clean matters more.
- Activate it only through explicit test config or a `--fake-worker` CLI flag used by tests.
- Script fake outcomes by task objective or task metadata:
  - `fake:applied` returns a tiny valid patch.
  - `fake:no-change` returns no diff.
  - `fake:rate-limit` returns a rate-limit classification.
  - `fake:timeout` simulates timeout without mutation.
  - `fake:mutated-timeout` simulates ambiguous mutation and must land in `NEEDS_REVIEW`.
  - `fake:slow` lets pause, drain, cancel, and emergency stop tests observe in-flight behavior.

Acceptance:

- Tests can prove pause, drain, cancel, emergency stop, late worker return, retry, and recovery without invoking Devin.
- A fake mutating timeout never becomes `DONE`.
- A fake clean rate-limit returns to `READY` when retry budget allows.

IMPORT 3: STOP WINS OVER LATE RESULTS

ACP explicitly tests that a user stop is not clobbered by a late completed or failed run. Conductor has stronger control states, but this exact invariant should be locked with regression tests because late provider output is a common source of corrupt status.

Suggested Conductor target:

- Add tests in `tests/test_control_plane.py`, `tests/test_single_worker_loop.py`, and `tests/test_scheduler_integrator.py`.
- Add or confirm a monotonically increasing control epoch in `control_state`.
- On stop, drain, emergency stop, or cancel, record the epoch and reason.
- When a worker result returns, compare the lease/control epoch before applying disposition.

Required behavior:

- Emergency stop wins over every late result.
- Cancelled task plus late mutating worker output becomes `NEEDS_REVIEW`, not `READY` and not `DONE`.
- Clean late rate-limit output after a cancel stays cancelled unless a human retries it.
- Stop/drain prevents new leases even if old workers finish cleanly.

Acceptance:

- A test starts a fake slow worker, issues stop, then lets the worker return success. Final state remains stopped or drained, no new lease starts, and the task is not promoted to `DONE` by the stale return.

IMPORT 4: HUMAN APPROVAL GATE FOR RUN BUDGETS

ACP parks a loop at `awaiting_approval` when it hits its iteration cap. Conductor should not copy ACP's loop model, but it should copy the operator contract: when a bounded autonomous run reaches a configured ceiling, park in a clearly resumable state rather than silently continuing or hard failing.

Suggested Conductor target:

- Extend `control_state` with:
  - `approval_required INTEGER NOT NULL DEFAULT 0`
  - `approval_reason TEXT`
  - `approval_requested_at TEXT`
  - `resume_budget_json TEXT`
- Add CLI:
  - `conductor.py control approve --extra-attempts N --reason TEXT`
  - `conductor.py control approval-status`
- Add dashboard actions for approve and reject.

Use approval for:

- daily free attempt cap exhausted
- paid attempt cap requested
- repeated integration conflicts above threshold
- global gate health degraded
- too many `NEEDS_REVIEW` items created in one pass
- disk pressure that cleanup cannot fix

Do not use approval for:

- task-level deterministic gate failure
- ordinary retry budget exhaustion
- expected `NEEDS_REVIEW`

Acceptance:

- Hitting the cap parks the driver without starting more leases.
- Resume requires an explicit approved budget.
- The approval event records the actor-provided reason and budget.

IMPORT 5: DASHBOARD HISTORY SHAPE

ACP's React dashboard is a better operator product than Conductor's current simple local dashboard. Import the layout and history ideas while keeping Conductor's backend semantics.

Suggested Conductor target:

- Keep `dashboard_server.py` as the control API until a replacement is proven.
- Add a React app under `dashboard/` only if it reads from Conductor APIs, never directly from SQLite.
- Add these API shapes first:
  - `GET /api/status`
  - `GET /api/tasks?status=...&wave=...&limit=...`
  - `GET /api/tasks/:task_id/timeline`
  - `GET /api/attempts/:attempt_id/events`
  - `GET /api/gates?task_id=...`
  - `GET /api/integrations?task_id=...`
  - `GET /api/escalations`
  - `POST /api/control/pause`
  - `POST /api/control/resume`
  - `POST /api/control/drain`
  - `POST /api/control/emergency-stop`
  - `POST /api/control/cancel`
  - `POST /api/control/approve`

Dashboard panels to import:

- live run summary: active workers, current wave, drain/paused/emergency flags, disk guard, cargo gate pressure
- history table: attempts, gates, integrations, token/cost metrics, final disposition
- task detail: TaskSpec, allowed write set, context refs, patch, ATIF, gate output, integration result
- escalation queue: retry hint, failing gate, previous worker notes, recommended next action
- control panel: pause, resume, drain, cancel, emergency stop, approve budget

Acceptance:

- The dashboard can answer "why is this task not DONE ?" without opening SQLite manually.
- The dashboard can answer "what changed since the last hour ?" from events and integration records.
- All mutating actions call backend control endpoints, not direct DB writes.

IMPORT 6: PROVIDER-AGNOSTIC WORKER RESULT NORMALIZATION

ACP is useful as a provider-independent wrapper around Pi. Conductor should keep Devin as the active worker harness, but `worker_adapter.py` should normalize results so a future Pi, Codex, GPT bridge, or paid-model lane can plug in without changing scheduler or gate logic.

Suggested Conductor target:

- Keep `TaskSpec` as the input contract.
- Keep `TaskResult` as the output contract.
- Add a provider capability record:
  - supports token usage
  - supports cost usage
  - supports stream events
  - supports session id
  - supports hard cancel
  - supports sandbox mode
- Ensure every provider maps to the same disposition fields:
  - `status_hint`
  - `error_category`
  - `mutated`
  - `diff_path`
  - `atif_path`
  - `sanitized_notes`
  - `provider_session_id`
  - `input_tokens`
  - `output_tokens`
  - `cost_usd`

Acceptance:

- Adding a Pi provider does not touch `scheduler.py`, `leases.py`, `gate_runner.py`, or `integrator.py`.
- Provider-specific failures are classified before scheduler disposition.
- Unknown provider output fails closed into `NEEDS_REVIEW` when mutation cannot be proven clean.

IMPORT 7: INTEGRATION TEST SCRIPT AS A PRODUCT CONTRACT

ACP has an end-to-end `itest.ts` that treats the API like the product boundary. Conductor needs the same style for its dashboard/control surface, separate from lower-level pytest coverage.

Suggested Conductor target:

- Add `tests/test_dashboard_contract.py` for Python-level API tests.
- Add `dashboard/itest` later if a React app is introduced.
- Use fake workers and a temp DB.

Contract cases:

- validation rejects missing task ids and invalid control payloads
- pause prevents new leases
- drain lets in-flight workers finish but starts no new workers
- emergency stop blocks scheduling immediately
- cancel handles clean not-yet-mutating lease
- late mutating result after cancel lands in `NEEDS_REVIEW`
- approval gate parks and resumes with a new budget
- task timeline returns attempts, gate runs, integration record, artifacts, and canonical events

Acceptance:

- One test command proves the operator API can be trusted before running a real autonomous pass.

IMPORT 8: READ-ONLY RETOOL OPTION, LATER

ACP documents Retool reading Neon directly and calling backend endpoints for actions. Conductor should not move truth to Neon, but a read-only Retool or external dashboard could be useful after local APIs are stable.

Suggested Conductor target:

- First expose sanitized JSON endpoints from `dashboard_server.py`.
- Later, optionally export a read-only SQLite snapshot or JSON status bundle for Retool.
- Mutations still go through Conductor control endpoints.

Acceptance:

- No external tool gets write access to `state/control.db`.
- Secrets, ATIF raw content, and provider transcripts are redacted before leaving localhost.

DO NOT IMPORT

- Do not import ACP's LLM orchestrator as the planner of record. ELAI already has a plan corpus and deterministic scheduler.
- Do not let an LLM decide `DONE`.
- Do not use `PROGRESS.md` as authoritative state.
- Do not replace SQLite and local artifacts with Neon for the conductor source of truth.
- Do not copy broad worker tool access without lease/write-set enforcement.
- Do not use prompt-only "stay within your task" as a concurrency control.
- Do not copy `Promise.all` fanout semantics into the scheduler path.
- Do not expose a public backend with wide-open CORS until auth and redaction are explicit.
- Do not add a React dashboard that writes directly to DB.
- Do not introduce provider switching that bypasses configured model caps, cooldowns, budget caps, or escalation policy.

RECOMMENDED ORDER

P0

- Add fake worker harness.
- Add stop-wins and late-result tests.
- Add approval gate state if current caps need operator-controlled continuation.

P1

- Add attempt lineage fields and provider telemetry.
- Add task timeline query.
- Add dashboard contract tests around existing `dashboard_server.py`.

P2

- Build the React dashboard against Conductor APIs.
- Add timeline, history, escalation queue, and operator controls.

P3

- Add optional Pi provider adapter.
- Add read-only external dashboard export if local operations still need it.

FIRST PATCH SCOPE

The smallest useful patch is not the React UI. It is:

- `migrations/005_attempt_lineage.sql`
- `conductor_db.py` timeline query
- `worker_adapter.py` fake provider path
- `tests/test_single_worker_loop.py` stop-wins regression
- `tests/test_dashboard_server.py` timeline endpoint coverage

That patch gives better evidence and safer control before adding more UI surface.
