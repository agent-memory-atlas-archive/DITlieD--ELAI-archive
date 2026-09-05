SP-1C ops/integration exploration - teikoku conductor (units 1.3/1.4/1.5)
session_id=s-finish-0705 task_id=SP-1C-explore subsystem=teikoku-conductor-ops
All paths relative to /home/user/teikoku/conductor/ (WSL distro "teikoku"). Read-only; no files touched.

PRODUCTION_HOOK 1.4a
Seam: walk_control.py::walk_start() lines 49-73. Insert the reclaim call right after
STOP_SENTINEL.unlink() (line 56-58) and before subprocess.Popen(...) spawns the driver (line 61-72).
Call: recovery.recover_after_restart(db_path, heartbeat_stale_seconds) - recovery.py:9-22. This
wrapper already composes both primitives: leases.py::LeaseManager.reclaim_expired (leases.py:380) and
leases.py::LeaseManager.reclaim_stale_heartbeats (leases.py:441, the stale-heartbeat primitive asked
for). recovery.recover_after_restart currently has ZERO callers anywhere in conductor/*.py (the only
other "recovery" hit is an unrelated argparse subparsers var conductor.py:718) - it is wire-dark
machinery, not a missing primitive. walk_control.py has no db_path/heartbeat_stale_seconds in scope
today; source both from ConductorConfig.from_file(config/conductor.elai-walk.toml) (config.state_db,
config.limits.heartbeat_stale_seconds=600, conductor_config.py:146).

PRODUCTION_HOOK 1.4b
Seam: conductor_loop.py::run_task_loop, the `except Exception as exc:` block, lines 395-421.
Two widenings needed, both proven against LIVE data (see Q2/Q4 below):
 (a) infra_error test (line 407) is `stage == "EXECUTE" and (isinstance(exc, OSError) or dur<60)`.
     All 11 live matches actually land at stage=="CONTEXT" (stage set at conductor_loop.py:196,
     before EXECUTE at :223) so infra_error is False for every one of them today (payload literally
     carries "infra":false). Add stage=="CONTEXT" to the guard.
 (b) `_prepare_worktree` (worker_adapter.py:428-461) raises bare RuntimeError(redact_text(stderr))
     on a `git worktree add` failure (line 461), never OSError, so even an EXECUTE-stage worktree-prep
     Errno5 slips the `isinstance(exc, OSError)` leg and depends only on the dur<60 fallback.
Existing refund primitive: budgets.refund_attempt_budget(task_id, "infra_execute_exception")
(conductor_loop.py:413, imported from budgets.py) ALREADY EXISTS and already fires when infra_error
is True - it decrements the attempt/daily-cap counter only. The missing half is status: line 398
`leases.release(lease_id, "NEEDS_REVIEW", "loop_error")` runs UNCONDITIONALLY before infra_error is
even evaluated, so even a correctly-classified infra error still parks the task in NEEDS_REVIEW, not
READY. Fix = move the release() call after the infra_error test and branch next_status to "READY"
(reason "infra_context_or_execute_error") when infra_error is True, "NEEDS_REVIEW" otherwise.

1. WALK_START RECLAIM SEAM - see PRODUCTION_HOOK 1.4a above. Both ends: walk_control.py:49
walk_start() (consumer) <- leases.py:441 reclaim_stale_heartbeats (producer), reached via
recovery.py:9 recover_after_restart (the existing but uncalled composition wrapper). Root-cause why
this matters: scheduler.py::schedule_frontier (scheduler.py:93-101) is the actual capacity gate run
every pass and it calls ONLY leases.reclaim_expired (line 101), never reclaim_stale_heartbeats - so a
lease that is heartbeat-dead but not yet TTL-expired keeps its worker_slot BUSY and schedule_frontier
returns scheduled_count:0 forever (until the raw lease TTL elapses), meaning conductor_loop.py:170's
own reclaim_stale_heartbeats call (inside run_task_loop's SELECT branch) never gets a chance to run
because no task_id ever reaches that branch. Confirmed no live instance of this exists right now (see
Q4: 0 leases currently) but the deadlock shape is real and unrebutted by any other code path.

2. WORKER-FAILURE CLASSIFICATION SEAM - see PRODUCTION_HOOK 1.4b above. classify_error_category
(worker_adapter.py:804-825) classifies a COMPLETED worker result (rate_limited/timed_out/cancelled/
violations/exit_code/output text) into rate_limited|transient_provider_error|context_window_blocked|
timeout|cancelled|allowed_write_violation|worker_failed|None - it has NO branch for Errno5/IO/
worktree-prep because those are raised Python exceptions during CONTEXT/EXECUTE setup, not worker
output. That path is entirely conductor_loop.py's outer except-block (1.4b above). Live proof this is
a real, currently-manifesting gap, not a hypothetical: 11 of 37 NEEDS_REVIEW tasks (30%) carry
event_type=LOOP_ERROR payload `{"error":"[Errno 5] Input/output error: '<path>'","infra":false,
"stage":"CONTEXT"}` (events table, created_at 2026-07-04T16:21:13Z..16:21:26Z, a single ~13s cluster -
task_ids C-50-WIRE-1, D-11-WIRE-1, E-18-EXECUTE-1, E-22-WIRE-1, E-25-WIRE-1, E-31-EXECUTE-1,
E-42-WIRE-1, E-45-EXECUTE-1, G-5-WIRE-1, G-8-EXECUTE-1, G-10-EXECUTE-1). All 11 sit at attempts=1/
max_attempts=3 (2 budget attempts unused, verified by direct query) yet are permanently parked in
NEEDS_REVIEW with no auto-promotion path back to READY - this IS the P7 false-negative class, sized
today at 11/37 = 30% of the current NEEDS_REVIEW backlog.

3. INTEGRATOR ATTEMPT LIFECYCLE / DIRT-BASELINE - PROVEN FALSE (no inheritance) on the default retry
path, two independent lines of evidence. (a) code: SELECT-stage call conductor_loop.py:172
`leases.acquire_ready(task_id=task_id, now=utc_now())` passes NO worktree_path override ->
leases.py::acquire_ready (leases.py:69, path-compute at line 201) `wt = worktree_path or
str(self.worktree_root / f"{safe_task_id}_{lease_id[:8]}")` - lease_id is freshly minted per acquire,
so every new attempt gets a brand-new directory name, then worker_adapter.py::_prepare_worktree
(worker_adapter.py:428-461) does a fresh `git worktree add -b <branch> <path> <task.base_commit>`
(no reuse of any prior worktree's file state). (b) diskguard.py's own header (diskguard.py:16-18,
authored 2026-07-04/05 same incident) states it directly: "A worktree is dead the instant its task is
no longer LEASED: a re-lease always creates a FRESH worktree ... old ones are never reused." The ONE
legitimate reuse path is the IntegrationBusy requeue (conductor_loop.py ~684-690,
`leases.release(lease_id, "VERIFIED", "integration_busy_requeue")`) which intentionally keeps the same
worktree/branch because the code in it IS already verified and only integration needs a retry - not a
bug. A FAILED/NEEDS_REVIEW task's old worktree is simply orphaned on ext4 until diskguard's between-
pass reap (diskguard.py::reap_stale_worktrees, lines 65-92, keyed on tasks.status != 'LEASED') deletes
it; it is never read from again.

4. LIVE COUNTS (control.db, read via sqlite3 file:...?mode=ro, verified 2026-07-05).
LEASED: 0 rows in `leases` table, 0 tasks with status='LEASED' - no orphans right now. Consistent with
ELAI_WALK_STOP sentinel present (conductor/ELAI_WALK_STOP, 0 bytes, mtime Jul 5 17:41 - the walk is
currently stopped, so nothing holds a lease). NEEDS_REVIEW: 37 total tasks. Of those, 11 (30%) match
the Errno5/IO/worktree-prep pattern per item 2 above (all at stage=CONTEXT, all infra:false, all
attempts=1/max_attempts=3). This 11/37 is the concrete, present-day size of the P7 gap; P8 (whatever
the plan's second infra-misclassification category is) was not separately identifiable in the events
sample without the plan file's own P8 definition - flagging as unresolved rather than guessing.

5. DISKGUARD / WORKTREE ROOT / WORKER_CAP.
diskguard.py: PRESENT-AND-WIRED, but only through one seam: elai_walk_driver.sh (repo root, no path
prefix needed - lives beside conductor.py) calls `python3 diskguard.py --reap --db "$DB"
--worktree-root "$WORKTREE_ROOT"` BETWEEN every pass, unconditionally (elai_walk_driver.sh lines
~15-22). No cron entry, no call from conductor.py/conductor_loop.py itself - a manual
`python3 conductor.py run-backlog` invoked outside the driver script never reaps. Verdict:
present-but-only-wired-through-the-dashboard-walk-path.
Worktree root: conductor/elai-worktrees (native WSL distro ext4, NOT a /mnt/f DrvFs/9p path -
diskguard.py:7-14 docstring). The distro's ext4 lives inside a single vhdx file that is physically
backed by host F: (HOST_MOUNT="/mnt/f" is only the disk-usage-check target, diskguard.py:41); the
2026-07-05 Errno5 storm was the vhdx starving host F:, not 9p cross-fs flakiness - correcting an
initial hypothesis against diskguard.py's own documented ground truth.
worker_cap: per-profile config file, NOT a single global value - config/conductor.elai-walk.toml:33
worker_cap=2 (the live dashboard-walk profile) vs config/conductor.elai-opus.toml:31 worker_cap=4.
elai_walk_driver.sh additionally hardcodes --limit-per-wave 2 to match the walk profile's cap 1:1 (see
its inline comment). Verdict: present-and-wired, split across two profile tomls by design.

GAPS / RISKS
- schedule_frontier (scheduler.py:101) never calls reclaim_stale_heartbeats - only run_task_loop's
  per-task SELECT branch does (conductor_loop.py:170). If 1.4a's fix lands ONLY at walk_start and not
  also in schedule_frontier, a heartbeat-stale lease that survives a live (non-restart) crash inside an
  otherwise-running walk still deadlocks capacity until raw TTL expiry - restart-time reclaim does not
  cover the in-place-crash case.
- The 11 stuck NEEDS_REVIEW tasks are real backlog inventory today; whatever remediation 1.4b ships
  will need a one-time backfill pass (flip these 11 to READY, they still have attempts<max_attempts)
  separate from the code fix itself, or they stay stuck forever under the new code too.
- P8 category could not be identified from live event data alone; needs the plan text's own P8
  definition to size against events.

R-5 STRUCTURED RETURN
STATUS: DONE
FILES TOUCHED: NONE (read-only exploration; wrote only this report)
VERIFIED: file:line evidence for every claim above read directly from conductor/*.py via wsl -d
teikoku; live control.db queried read-only (file:...?mode=ro) via python3/sqlite3 - lease count,
NEEDS_REVIEW count, LOOP_ERROR payload pattern match, attempts/max_attempts all directly queried, not
inferred. UNVERIFIED: P8 category identity (no plan-text definition in scope); whether 1.4a's exact
config wiring (db_path/heartbeat_stale_seconds sourcing into walk_control.py) needs a new import or
can reuse an existing config singleton - not traced, left for the executor.
QUESTION: none (not blocked)
NOTES: PRODUCTION_HOOK 1.4a and 1.4b both cite real, currently-dark or currently-mis-scoped code, not
hypothetical seams. The 11/37 NEEDS_REVIEW Errno5-at-CONTEXT finding was verified against live event
rows, not guessed from static reading - it changed my read of 1.4b mid-exploration (from "OSError type
mismatch is the whole story" to "stage-scoping to EXECUTE-only is the dominant miss").
