# ELAI Conductor Implementation Plan

Date: 2026-06-18
Scope: `/workspace/ELAI/conductor`

This plan implements the autonomous-finish Conductor as workflow tooling outside
ELAI product code. All Conductor writes stay in this directory. ELAI is read
directly only for maps, plans, status fragments, frozen gates, and source
verification; future mutation of ELAI work happens only through isolated git
worktrees owned by the Conductor, never by editing the developer checkout in
place.

## Source Designs Read

- `/workspace/ELAI/workflow/reasoning/AUTONOMOUS-FINISH-SYSTEM-2026-06-18.md`
- `/workspace/ELAI/workflow/reasoning/AUTONOMOUS-FINISH-SYSTEM-DEVIN-2026-06-18.md`
- `AGENTS.md` in this conductor directory

The active implementation target is the Devin CLI/free-model variant: native
Windows Python daemon, `devin -p` workers, free-model routing, ATIF export
parsing, SQLite WAL state, isolated worktrees, off-agent verification, serialized
integration, and no commits or pushes by this implementation pass.

## Inspected Integration Evidence

ELAI gates and maps were inspected before naming integrations. The important
surfaces are:

- Frozen verifier tools: `/workspace/ELAI/.claude/tools/production_caller_check.py`,
  `/workspace/ELAI/.claude/tools/python_wire_check.py`,
  `/workspace/ELAI/.claude/tools/maintainability_check.py`,
  `/workspace/ELAI/.claude/tools/traced_test_assert_check.py`,
  `/workspace/ELAI/.claude/tools/gate_freeze_check.py`,
  `/workspace/ELAI/.claude/tools/wiring_contract_check.py`,
  `/workspace/ELAI/.claude/tools/spec_link_check.py`,
  `/workspace/ELAI/.claude/tools/cross_lang_diff.py`,
  `/workspace/ELAI/.claude/tools/bn_status_report.py`,
  `/workspace/ELAI/.claude/tools/failure_taxonomy.py`.
- Frozen manifest: `/workspace/ELAI/.claude/gate-manifest.txt` and
  `/workspace/ELAI/.claude/gate-manifest.lock`.
- Maps and retrieval artifacts: `/workspace/ELAI/.claude/tools/elai.map`,
  `/workspace/ELAI/.claude/tools/elai_graph.json`,
  `/workspace/ELAI/elai.map`, `/workspace/ELAI/elai_graph.json`,
  `/workspace/ELAI/.elai_cc/.codegraph/codegraph.db`,
  `/workspace/ELAI/.elai_cc/indexes/dkg.db`,
  `/workspace/ELAI/.elai_cc/sessions/ssb.db`,
  `/workspace/ELAI/.elai_cc/sessions/dag.db`.
- Live Rust/CLI wiring surfaces:
  `/workspace/ELAI/.elai_cc/crates/cli/src/wiring_audit.rs`,
  `/workspace/ELAI/.elai_cc/crates/cli/src/wiring_coverage_cmd.rs`,
  `/workspace/ELAI/.elai_cc/crates/cli/src/plan_status_cmd.rs`,
  `/workspace/ELAI/.elai_cc/crates/cli/src/freeze_cmd.rs`,
  `/workspace/ELAI/.elai_cc/crates/orchestrator/src/wiring_contract.rs`.
- ELAI config and frozen-policy inputs:
  `/workspace/ELAI/.elai_cc/elai_cc.toml` with `[wiring.entrypoints]`,
  `/workspace/ELAI/.elai_cc/scope-freeze.toml`,
  `/workspace/ELAI/.elai_cc/wire-dark-allowlist.toml`,
  `/workspace/ELAI/.elai_cc/smoke-manifest.toml`.
- Plan corpus and status sources:
  `/workspace/ELAI/docs/thefinalglance`,
  `/workspace/ELAI/workflow/sessions/s-plan-audit-2026-06-04/FINISH`.

Current ELAI checkout state is dirty and unrelated to this plan:
`AGENTS.md` is deleted, `crates/dashboard-v2/src/lib/layout/shell/TabBar.svelte`
and `crates/dashboard-v2/src/lib/screens/ProjectSelector.svelte` are modified,
and `crates/dashboard-v2/src/lib/layout/shell/BrandCluster.svelte` is untracked.
The Conductor must not touch those changes. Future integration must use a clean
integration worktree or stop with `NEEDS_REVIEW`.

## Current Conductor State

Existing local prototypes are present and must be audited rather than blindly
replaced:

- `backlog_import.py`: prototype importer and schema creator.
- `worker_adapter.py`: prototype Devin adapter.
- `state/control.db`: existing SQLite database.
- `.build-context/*`, `test_wt/*`, `worktrees/*`, `test_*.json`, `test_*.yaml`,
  `test_taskspec.txt`: existing untracked artifacts.

Notable prototype issues to resolve in early phases:

- `worker_adapter.py` uses `--agent-config`; the Devin design says verified CLI
  flag is `--config`. Phase 0 must confirm the actual installed CLI and update
  the adapter before unattended execution.
- `worker_adapter.py` marks dry-run results as `DONE`; worker output must never
  create Conductor `DONE`. Only off-agent verifier acceptance can do that.
- `worker_adapter.py` removes worktrees in `finally`; artifact retention is
  required until verifier and integrator have recorded result evidence.
- `backlog_import.py` has Windows hard-coded paths and duplicate code in
  `compute_idem_key`; path handling must be normalized and idempotence tested.
- `sqlite3` CLI is not available in the current shell; validation should use a
  Python DB check command or documented Windows tooling.

## Non-Negotiable Invariants

- No credentials in the repo, logs, browser responses, TaskSpec, TaskResult,
  or event rows.
- No `gate_freeze_check.py --lock` in any worker or Conductor automatic path.
- No weakening, editing, regenerating, or allowlisting frozen gates as part of
  task execution.
- No direct writes to `/workspace/ELAI/.elai_cc`; use git worktrees under
  the Conductor workspace.
- No push. Local commits are disabled by default and require an explicit later
  operator decision; this plan is based on patch artifacts and serialized local
  integration worktrees.
- A worker can return a patch and notes, but cannot certify completion.
- Missing verifier evidence is `FAIL`, not `SKIP`.
- A disconnected mutating worker run becomes `NEEDS_REVIEW`, never automatic
  replay.

## Bounded Phases

### Phase 0: Baseline And Compatibility

Goal: prove the external surfaces before building on them.

Implement:

- `config/conductor.toml.example` with non-secret paths, port, worker cap, model
  names, timeouts, and gate command templates.
- `conductor doctor` or an equivalent `doctor.py` that checks:
  - Python version and Windows path normalization.
  - Devin executable exists.
  - Exact free `--model` identifiers.
  - Whether `devin -p` accepts `--config`, `--prompt-file`, `--export`, and
    `--permission-mode`.
  - Outer `.claude/tools` gate files and map files exist.
  - ELAI checkout dirty state is reported and integration is blocked when dirty.
  - `gate_freeze_check.py` is callable without `--lock`.

Acceptance commands:

```powershell
py -3 conductor.py doctor --config config\conductor.toml.example --json
py -3 worker_adapter.py --dry-run --task-id DOCTOR
```

The second command remains dry-run only; no real Devin task is launched in
Phase 0 acceptance.

### Phase 1: Durable State And Schema

Goal: define the authoritative task store and append-only event log.

Files:

- `conductor_db.py`
- `migrations/001_initial.sql`
- `tests/test_db_schema.py`

Required schema:

```sql
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE schema_meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE tasks (
  task_id TEXT PRIMARY KEY,
  plan_id TEXT NOT NULL,
  wave TEXT NOT NULL,
  role TEXT NOT NULL CHECK (role IN (
    'PLAN','EXECUTE','WIRE','INTEGRATE','AUDIT','POLISH','FIX'
  )),
  objective TEXT NOT NULL,
  base_commit TEXT NOT NULL,
  allowed_write_set_json TEXT NOT NULL,
  context_refs_json TEXT NOT NULL,
  production_hook TEXT,
  acceptance_text TEXT NOT NULL,
  deterministic_check_text TEXT,
  forbidden_text TEXT NOT NULL,
  verifier_json TEXT NOT NULL,
  lease_ttl_seconds INTEGER NOT NULL,
  idem_key TEXT NOT NULL UNIQUE,
  model_tier TEXT NOT NULL CHECK (model_tier IN ('FREE','PAID')),
  priority INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL CHECK (status IN (
    'PLANNED','READY','LEASED','RUNNING','VERIFYING','VERIFIED',
    'INTEGRATING','DONE','NEEDS_REVIEW','BLOCKED','FAILED','CANCELLED'
  )),
  attempts INTEGER NOT NULL DEFAULT 0,
  max_attempts INTEGER NOT NULL DEFAULT 3,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE task_deps (
  task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
  depends_on_task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
  PRIMARY KEY (task_id, depends_on_task_id),
  CHECK (task_id <> depends_on_task_id)
);

CREATE TABLE worker_slots (
  slot_id TEXT PRIMARY KEY,
  harness TEXT NOT NULL CHECK (harness IN ('devin-cli')),
  model TEXT NOT NULL,
  model_tier TEXT NOT NULL CHECK (model_tier IN ('FREE','PAID')),
  max_concurrency INTEGER NOT NULL DEFAULT 1,
  state TEXT NOT NULL CHECK (state IN ('IDLE','LEASED','COOLDOWN','DISABLED')),
  cooldown_until TEXT,
  last_error_category TEXT
);

CREATE TABLE leases (
  lease_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL UNIQUE REFERENCES tasks(task_id) ON DELETE CASCADE,
  slot_id TEXT NOT NULL REFERENCES worker_slots(slot_id),
  worktree_path TEXT NOT NULL,
  branch_name TEXT NOT NULL,
  session_id TEXT,
  lease_started_at TEXT NOT NULL,
  lease_expires_at TEXT NOT NULL,
  heartbeat_at TEXT NOT NULL,
  disconnected_after_mutation INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE task_attempts (
  attempt_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
  attempt_no INTEGER NOT NULL,
  slot_id TEXT,
  model TEXT,
  started_at TEXT NOT NULL,
  finished_at TEXT,
  exit_code INTEGER,
  status_hint TEXT,
  error_category TEXT,
  atif_path TEXT,
  diff_path TEXT,
  sanitized_notes TEXT,
  UNIQUE (task_id, attempt_no)
);

CREATE TABLE gate_runs (
  gate_run_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
  attempt_id TEXT REFERENCES task_attempts(attempt_id) ON DELETE SET NULL,
  gate_name TEXT NOT NULL,
  command_json TEXT NOT NULL,
  cwd TEXT NOT NULL,
  started_at TEXT NOT NULL,
  finished_at TEXT,
  exit_code INTEGER,
  verdict TEXT NOT NULL CHECK (verdict IN ('PASS','FAIL','ERROR','TIMEOUT')),
  stdout_artifact TEXT,
  stderr_artifact TEXT,
  summary TEXT,
  UNIQUE (task_id, gate_name, started_at)
);

CREATE TABLE integrations (
  integration_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
  source_worktree_path TEXT NOT NULL,
  source_base_commit TEXT NOT NULL,
  integration_worktree_path TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN (
    'PENDING','APPLIED','CONFLICT','POST_GATE_FAILED','DONE','NEEDS_REVIEW'
  )),
  started_at TEXT,
  finished_at TEXT,
  patch_artifact TEXT,
  post_gate_run_id TEXT REFERENCES gate_runs(gate_run_id)
);

CREATE TABLE artifacts (
  artifact_id TEXT PRIMARY KEY,
  task_id TEXT REFERENCES tasks(task_id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  path TEXT NOT NULL,
  blake3 TEXT,
  created_at TEXT NOT NULL,
  redacted INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE model_cooldowns (
  model TEXT PRIMARY KEY,
  cooldown_until TEXT NOT NULL,
  reason TEXT NOT NULL,
  observed_at TEXT NOT NULL
);

CREATE TABLE events (
  event_seq INTEGER PRIMARY KEY AUTOINCREMENT,
  event_id TEXT NOT NULL UNIQUE,
  task_id TEXT,
  event_type TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE INDEX idx_tasks_status_priority ON tasks(status, priority DESC, created_at);
CREATE INDEX idx_task_deps_dep ON task_deps(depends_on_task_id);
CREATE INDEX idx_gate_runs_task ON gate_runs(task_id, gate_name);
CREATE INDEX idx_events_task ON events(task_id, event_seq);
```

Acceptance commands:

```powershell
py -3 -m pytest -q tests\test_db_schema.py
py -3 conductor.py db init --db state\control.test.db --reset
py -3 conductor.py db check --db state\control.test.db
```

### Phase 2: Backlog Import

Goal: import plan files, dependencies, scopes, checks, and status seeds
deterministically.

Inputs:

- `/workspace/ELAI/docs/thefinalglance/{B..K}-plan`
- `/workspace/ELAI/workflow/sessions/s-plan-audit-2026-06-04/FINISH`
- `/workspace/ELAI/.claude/tools/bn_status_report.py`
- `/workspace/ELAI/.elai_cc/crates/cli/src/plan_status_cmd.rs` behavior
  as a secondary status reference.

Implement:

- Normalize Windows and WSL paths.
- Parse every non-index plan markdown file into one or more small tasks.
- Extract `WIRING_CONTRACT`, `DETERMINISTIC_CHECK`, `ACCEPTANCE`,
  pre-requisites, files touched, production hook, and plan objective.
- Seed status from FINISH fragments and `bn_status_report.py`.
- Create `WIRE` tasks for wire-dark or unreachable plans, not only `EXECUTE`
  tasks.
- Store all parsed source references as artifact rows.
- Reject imports with zero tasks or zero non-empty contracts.

Acceptance commands:

```powershell
py -3 backlog_import.py --dry-run --json --corpus-root /workspace/ELAI\docs --finish-dir /workspace/ELAI\workflow\sessions\s-plan-audit-2026-06-04\FINISH
py -3 backlog_import.py --db state\control.test.db --force --json --corpus-root /workspace/ELAI\docs --finish-dir /workspace/ELAI\workflow\sessions\s-plan-audit-2026-06-04\FINISH
py -3 backlog_import.py --db state\control.test.db --check --json --corpus-root /workspace/ELAI\docs --finish-dir /workspace/ELAI\workflow\sessions\s-plan-audit-2026-06-04\FINISH
```

### Phase 3: Context Packet Builder

Goal: build bounded, deterministic context per task without LLM orchestration.

Inputs:

- Task row and parsed plan source.
- `/workspace/ELAI/.claude/tools/elai.map`
- `/workspace/ELAI/.claude/tools/elai_graph.json`
- `/workspace/ELAI/.elai_cc/.codegraph/codegraph.db`
- `/workspace/ELAI/.elai_cc/indexes/dkg.db`
- Specific files in `allowed_write_set`.

Implement:

- `context_builder.py` with `build_context_packet(task_id)`.
- Size caps by section: objective, contract, acceptance, graph neighborhood,
  touched-file excerpts, gate command list.
- Map freshness check using `/workspace/ELAI/.claude/tools/elai_map_freshness.py`.
- Deterministic ordering and stable hash for each context packet.
- No secrets or credentials copied into context.

Acceptance commands:

```powershell
py -3 conductor.py context build --db state\control.test.db --task-id <known-task> --out .build-context\task.md
py -3 conductor.py context check --db state\control.test.db --task-id <known-task>
```

### Phase 4: Gate Runner

Goal: make off-agent verification the only owner of accepted truth.

Implement:

- `gate_runner.py` with `run_gate_stack(task_id, worktree_path)`.
- Per-task scoped command expansion from `verifier_json`.
- Artifact capture for stdout, stderr, exit code, wall time, and summary.
- Fail-closed behavior when a command is missing, times out, or returns
  inconclusive evidence.
- Redaction before storing gate output.

Authoritative gates:

- Always run:
  - `python /workspace/ELAI\.claude\tools\gate_freeze_check.py`
  - `cargo check --workspace`
- Run when applicable:
  - `python /workspace/ELAI\.claude\tools\wiring_contract_check.py --bn-file <plan_md> --json`
  - `python /workspace/ELAI\.claude\tools\production_caller_check.py --files <files>`
  - `python /workspace/ELAI\.claude\tools\python_wire_check.py --files <py_files>`
  - `python /workspace/ELAI\.claude\tools\maintainability_check.py --files <files> --base <base_commit>`
  - `python /workspace/ELAI\.claude\tools\traced_test_assert_check.py --files <test_files>`
  - `python /workspace/ELAI\.claude\tools\spec_link_check.py --files <plan_or_doc_files>`
  - `python /workspace/ELAI\.claude\tools\cross_lang_diff.py --json`
  - `cargo test -p <crate> <selector>`
  - `cargo run -p elai-cli -- wiring audit --scope all --contracts /workspace/ELAI\docs\thefinalglance`
  - `cargo run -p elai-cli -- wiring-coverage --diff-base <base_commit>`
  - `cargo run -p elai-cli -- freeze verify`

Ownership rules:

- Gate runner writes `gate_runs`, `artifacts`, and `events`.
- Gate runner may mark `VERIFIED`.
- Only the serialized integrator plus post-merge gate may mark `DONE`.
- Workers can run tests for feedback, but their output is advisory only.
- `gate_freeze_check.py --lock` is forbidden everywhere.

Acceptance commands:

```powershell
py -3 -m pytest -q tests\test_gate_runner.py
py -3 conductor.py gate dry-run --db state\control.test.db --task-id <known-task>
py -3 conductor.py gate run --db state\control.test.db --task-id <known-task> --worktree <read-only-fixture-worktree>
```

### Phase 5: Devin Worker Adapter

Goal: dispatch one bounded TaskSpec into one `devin -p` process and parse one
TaskResult without allowing worker self-certification.

TaskSpec contract:

```json
{
  "schema_version": 1,
  "task_id": "I-34-EXECUTE-1",
  "plan_id": "I-34",
  "wave": "IMPLEMENT",
  "role": "EXECUTE",
  "objective": "One paragraph from the plan.",
  "base_commit": "<git-sha>",
  "allowed_write_set": ["repo/relative/path.rs"],
  "context_refs": ["plan file", "map slice", "source excerpts"],
  "production_hook": "repo/relative/file.rs::symbol",
  "acceptance": "verbatim ACCEPTANCE text",
  "deterministic_check": "verbatim DETERMINISTIC_CHECK text",
  "forbidden": [
    "no gate edits",
    "no gate relock",
    "no destructive git",
    "no commit",
    "no push",
    "no credential reads"
  ],
  "verifier": [{"name": "gate_freeze", "command": ["python", "..."]}],
  "lease_ttl_seconds": 3600,
  "idem_key": "stable-hash",
  "token_budget": 100000,
  "model_tier": "FREE"
}
```

TaskResult contract:

```json
{
  "schema_version": 1,
  "task_id": "I-34-EXECUTE-1",
  "base_commit": "<git-sha>",
  "worktree_path": "/workspace/ELAI\\conductor\\worktrees\\...",
  "branch_name": "conductor/I-34-EXECUTE-1",
  "model_used": "swe-1.6",
  "session_id": "<devin-session-or-null>",
  "exit_code": 0,
  "atif_artifact": "artifacts\\...\\result.atif.json",
  "diff_artifact": "artifacts\\...\\patch.diff",
  "touched_files": ["repo/relative/path.rs"],
  "worker_status_hint": "APPLIED",
  "error_category": null,
  "rate_limited": false,
  "disconnected_after_mutation": false,
  "sanitized_notes": "No secrets, no raw credentials."
}
```

Allowed `worker_status_hint` values are `APPLIED`, `NO_CHANGE`, `BLOCKED`,
`FAILED`, `RATE_LIMITED`, and `NEEDS_REVIEW`. There is no `DONE` worker hint.

Implement:

- Replace any unverified Devin flag names with Phase 0 confirmed names.
- Render TaskSpec to a prompt file.
- Render role config from `/workspace/ELAI/.claude/agents/*` or the
  nearest available role prompt.
- Apply write scopes from `allowed_write_set`.
- Capture ATIF, git diff, touched files, and sanitized notes.
- Preserve worktree and artifacts until verifier/integrator are done.
- Classify quota/rate errors as model cooldown events, not task failures.

Acceptance commands:

```powershell
py -3 -m pytest -q tests\test_worker_adapter.py
py -3 worker_adapter.py --dry-run --task-id WORKER-DRYRUN
py -3 conductor.py worker validate-models --config config\conductor.toml
```

### Phase 6: Scheduler, Leases, And Single-Worker Loop

Goal: run SELECT -> CONTEXT -> EXECUTE -> VERIFY -> RECORD for one worker and
one task type.

Implement:

- Dependency-aware ready query.
- Lease TTL and heartbeat.
- Crash-safe reclaim.
- Retry cap and state transitions.
- Model cooldown routing.
- Event rows for every transition.
- No integration in this phase; verified patches remain artifacts.

Acceptance commands:

```powershell
py -3 -m pytest -q tests\test_scheduler.py tests\test_leases.py tests\test_single_worker_loop.py
py -3 conductor.py run-once --db state\control.test.db --dry-run-worker --task-id <known-task>
py -3 conductor.py events replay --db state\control.test.db --check
```

### Phase 7: Worktree Integration

Goal: serialize patch application and post-merge verification without touching
the dirty developer checkout.

Rules:

- Source repo: `/workspace/ELAI/.elai_cc`.
- Worktree root: `/workspace/ELAI/conductor/worktrees`.
- One task worktree per leased task.
- One integration worktree, locked by `integration.lock`.
- Concurrent tasks must have disjoint `allowed_write_set` paths.
- The developer checkout is never used as the integration target.
- No push.
- Local commit creation remains disabled unless a later operator explicitly
  enables it; default integration artifact is a patch plus post-gate evidence.
- If rebase/apply conflicts, create an `INTEGRATE` task and mark the original
  `NEEDS_REVIEW`.
- Post-integration gate stack must pass before `DONE`.

Acceptance commands:

```powershell
py -3 -m pytest -q tests\test_worktrees.py tests\test_integrator.py
py -3 conductor.py integrate dry-run --db state\control.test.db --task-id <verified-task>
py -3 conductor.py gate run-post --db state\control.test.db --task-id <verified-task>
```

### Phase 8: Two Workers, Waves, And Completion Policy

Goal: widen from one worker to the full deterministic wave loop.

Waves:

- Wave 1 `IMPLEMENT`: PLANNED/PARTIAL -> EXECUTE -> build, wire, scoped tests.
- Wave 2 `WIRE-SWEEP`: repo-wide wire checks -> WIRE tasks for dark symbols.
- Wave 3 `INTEGRATION`: real entrypoint smoke -> INTEGRATE tasks for seam defects.
- Wave 4 `AUDIT`: adversarial auditor -> FIX tasks.
- Wave 5 `POLISH`: maintainability, visual dashboard loop, docs, dead-code.
- Wave N `RE-PASS`: audit and polish until K dry passes.

Completion is system-level:

- All milestone tasks terminal.
- Global gate stack green on the integration head.
- K consecutive dry audit passes.
- No `NEEDS_REVIEW` tasks.
- No unresolved model cooldown, quota, or gate-missing blocker.

Acceptance commands:

```powershell
py -3 -m pytest -q tests\test_disjoint_scheduler.py tests\test_waves.py tests\test_completion_policy.py
py -3 conductor.py wave plan --db state\control.test.db --wave IMPLEMENT --json
py -3 conductor.py wave run --db state\control.test.db --wave IMPLEMENT --limit 1 --dry-run-worker
```

### Phase 9: Browser Control Surface

Goal: provide a local operations dashboard, not a chat UI.

Server:

- Bind only to `127.0.0.1`.
- Default port `8766` to avoid bootstrap dashboard `8765`; configurable.
- Read-only by default until an operator clicks start/resume.
- All API output sanitized.
- No credentials, raw prompts, raw ATIF secrets, or environment dumps.

Views:

- System state: paused/running/draining/emergency-stopped.
- Worker slots: model, state, active lease, cooldown, last sanitized error.
- Queue: ready, leased, verifying, integrating, blocked, needs review.
- Task detail: TaskSpec, context refs, artifacts, gate runs, events.
- Integration queue: lock holder, pending patches, conflicts, post-gate result.
- Gate health: frozen manifest status, map freshness, last global gate result.
- Event log: append-only recent transitions.

Controls:

- Seed/import backlog.
- Start, pause, resume, drain.
- Cancel a not-yet-mutating lease.
- Mark `NEEDS_REVIEW` after human inspection.
- Retry a failed task within retry budget.
- Emergency stop.
- Export sanitized support bundle.

HTTP surface:

- `GET /api/health`
- `GET /api/state`
- `GET /api/tasks?status=...`
- `GET /api/tasks/{task_id}`
- `GET /api/workers`
- `GET /api/events?after=...`
- `GET /api/gates`
- `POST /api/control/start`
- `POST /api/control/pause`
- `POST /api/control/resume`
- `POST /api/control/drain`
- `POST /api/control/emergency-stop`
- `POST /api/tasks/{task_id}/retry`
- `POST /api/tasks/{task_id}/needs-review`

Acceptance commands:

```powershell
py -3 -m pytest -q tests\test_browser_api.py
py -3 conductor.py serve --db state\control.test.db --host 127.0.0.1 --port 8766 --read-only
curl http://127.0.0.1:8766/api/health
```

### Phase 10: Audit, Recovery, And Handoff

Goal: prove the Conductor can be stopped, restarted, audited, and handed off.

Implement:

- Event replay check: current task state can be reconstructed from events.
- Lease reclaim after process kill.
- Mutating disconnect detection.
- Artifact retention and redaction checks.
- Epoch summary every K completions.
- Sanitized handoff report.

Acceptance commands:

```powershell
py -3 -m pytest -q tests\test_recovery.py tests\test_redaction.py tests\test_event_replay.py
py -3 conductor.py recovery simulate --db state\control.test.db
py -3 conductor.py handoff write --db state\control.test.db --out .build-context\handoff.md
```

## Verifier Ownership Model

The verifier is an independent Conductor subsystem. It owns:

- Running frozen gates.
- Recording `gate_runs`.
- Deciding `PASS`, `FAIL`, `ERROR`, or `TIMEOUT`.
- Promoting `VERIFYING` to `VERIFIED`.

The integrator owns:

- Applying verified patches in serialized order.
- Running post-integration gates.
- Promoting `VERIFIED` to `DONE` only after post-gate success.

The worker owns only:

- Editing files in its worktree within `allowed_write_set`.
- Producing ATIF, diff, touched files, and notes.

The browser owns only:

- Operator controls and views.
- It cannot mark a task `DONE`.

## Deterministic Acceptance For This Plan File

Documentation-only acceptance for this task:

```bash
test -f IMPLEMENTATION_PLAN.md
rg -n "Bounded Phases|Database|TaskSpec|TaskResult|Verifier Ownership|Worktree Integration|Browser Control|Deterministic Acceptance" IMPLEMENTATION_PLAN.md
python -m py_compile backlog_import.py worker_adapter.py
git status --short
```

Expected result: only `IMPLEMENTATION_PLAN.md` is newly created by this task.
Existing untracked conductor prototypes and unrelated dirty ELAI dashboard
changes remain untouched.

## Remaining Blockers

- Confirm exact Devin free model identifiers and current CLI flags on Windows.
- Confirm ATIF export schema against one real `devin -p` dry fixture.
- Confirm free-tier concurrency and cooldown behavior empirically.
- Decide whether local commit creation is allowed for future serialized
  integration; this plan keeps it disabled by default and forbids push.
- Resolve or isolate the dirty ELAI checkout before any integration run.
- Install or replace `sqlite3` CLI validation with a Python DB checker.
- Run outer `.claude/tools/elai_map_freshness.py` before relying on map slices.

## Next Executable Step

Implement Phase 0 `doctor` checks and fix the Devin adapter flag mismatch
(`--config` vs `--agent-config`) in a scoped patch, then run:

```powershell
py -3 conductor.py doctor --config config\conductor.toml.example --json
py -3 worker_adapter.py --dry-run --task-id DOCTOR
```
