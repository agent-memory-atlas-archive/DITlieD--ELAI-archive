"""Walk-state spine for the sequential-Opus backlog walk.

Stdlib-only (json, os, tempfile, datetime). Every mutating call re-reads the
state file, applies the change, and writes it back atomically (tmp file in
the same directory + os.replace(), which is atomic on the same volume
including Windows). Loads are validated: a missing file, invalid JSON, wrong
schema_version, or a missing required top-level key all raise
MalformedWalkState rather than returning a silent default.

Production consumer: .claude/skills/sequential-opus-orchestrator/SKILL.md
(authored the phase immediately after this one) calls load/claim_lock/
heartbeat/release_lock/update_cursor/append_decision/record_dispatch/
mark_plan_done from its resume/stop protocol so a fresh Opus session can pick
up the walk from durable state instead of conversation memory.
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import tempfile
import time
from datetime import datetime, timedelta, timezone

HEARTBEAT_STALE_MIN = 30
SCHEMA_VERSION = 1
# Cross-process state mutex (serializes every read-decide-write critical
# section; the atomic os.replace alone does NOT serialize concurrent claims —
# audit 2026-07-06 proved 157/200 double-claims without it).
MUTEX_TIMEOUT_S = 10
MUTEX_RETRY_S = 0.05
MUTEX_STALE_S = 60
REPLACE_RETRIES = 5
REPLACE_RETRY_S = 0.05

_REQUIRED_KEYS = (
    "schema_version",
    "lock",
    "budget",
    "cursor",
    "queue",
    "escalation_queue",
    "decisions",
    "integration_smoke",
)

STATE_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "walk-state.json")
DISPATCH_CAP_ENV = "ELAI_WALK_DISPATCH_CAP"
PLAN_CAP_ENV = "ELAI_WALK_PLAN_CAP"
LOCK_OWNER_PID_ENV = "ELAI_WALK_CONTROLLER_PID"

# Teikoku-local disposition ledger (schema-matched to the F: canonical ledger
# workflow/status/plan-dispositions.json). This walk cannot write /mnt/f
# (scope_write_gate denies it), so it writes its verdict HERE and the F: sync
# workflow/status/sync_teikoku_dispositions.py pulls it over UNC into the
# canonical ledger, which the feeder projects into elai-status.json so
# elai_status_query --punch shows the verdict instead of re-discovering the gap.
DISPOSITIONS_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "plan-dispositions.json")
SOURCE_WALK = "teikoku-opus-walk"


class WalkStateError(Exception):
    """Base class for all walk-state errors."""


class MalformedWalkState(WalkStateError):
    """The state file is missing, unparsable, or fails schema validation."""


class LockRefused(WalkStateError):
    """A lock claim/heartbeat/release was refused because another live
    session holds the lock, or there is no lock to act on."""


class DoneRowRejected(WalkStateError):
    """mark_plan_done was called without a commit_sha or gate_evidence_ref."""


class StateMutexBusy(WalkStateError):
    """The cross-process state mutex could not be acquired within
    MUTEX_TIMEOUT_S — another process is mutating the state file."""


class _StateMutex:
    """Cross-process mutex serializing the read-decide-write critical section
    on the state file. O_CREAT|O_EXCL sidecar file (atomic on Windows + POSIX);
    a mutex file older than MUTEX_STALE_S is treated as a crashed holder and
    broken."""

    def __init__(self, state_path: str):
        self.mutex_path = state_path + ".mutex"
        self._fd = None

    def __enter__(self) -> "_StateMutex":
        deadline = time.monotonic() + MUTEX_TIMEOUT_S
        while True:
            try:
                self._fd = os.open(self.mutex_path, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
                os.write(self._fd, str(os.getpid()).encode("ascii"))
                return self
            except FileExistsError:
                try:
                    if time.time() - os.path.getmtime(self.mutex_path) > MUTEX_STALE_S:
                        os.remove(self.mutex_path)
                        continue
                except OSError:
                    pass  # holder released/broke it between our checks; retry loop
                if time.monotonic() > deadline:
                    raise StateMutexBusy(f"state mutex busy for >{MUTEX_TIMEOUT_S}s: {self.mutex_path}")
                time.sleep(MUTEX_RETRY_S)

    def __exit__(self, exc_type, exc, tb) -> None:
        try:
            if self._fd is not None:
                os.close(self._fd)
        finally:
            try:
                os.remove(self.mutex_path)
            except OSError:
                pass  # best-effort cleanup; a leftover file is broken as stale


def _utc_now() -> datetime:
    return datetime.now(timezone.utc)


def _utc_now_iso() -> str:
    return _utc_now().isoformat()


def _atomic_write(path: str, data: dict) -> None:
    directory = os.path.dirname(path) or "."
    os.makedirs(directory, exist_ok=True)
    fd, tmp_path = tempfile.mkstemp(prefix=".walkstate-", suffix=".tmp", dir=directory)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2)
        for attempt in range(REPLACE_RETRIES):
            try:
                os.replace(tmp_path, path)
                break
            except PermissionError:
                # Windows: a concurrent reader/replacer can transiently hold
                # the destination; retry briefly, then propagate.
                if attempt == REPLACE_RETRIES - 1:
                    raise
                time.sleep(REPLACE_RETRY_S)
    except BaseException:
        try:
            os.remove(tmp_path)
        except OSError:
            pass
        raise


def load(path: str = STATE_PATH) -> dict:
    """Load and validate the walk-state file. Never returns a silent default:
    any structural problem raises MalformedWalkState."""
    if not os.path.isfile(path):
        raise MalformedWalkState(f"walk-state file not found: {path}")
    try:
        with open(path, "r", encoding="utf-8") as f:
            state = json.load(f)
    except (OSError, json.JSONDecodeError) as exc:
        raise MalformedWalkState(f"walk-state file unreadable/invalid JSON: {path} ({exc})") from exc

    if not isinstance(state, dict):
        raise MalformedWalkState(f"walk-state root is not an object: {path}")

    missing = [k for k in _REQUIRED_KEYS if k not in state]
    if missing:
        raise MalformedWalkState(f"walk-state missing required keys {missing}: {path}")

    if state.get("schema_version") != SCHEMA_VERSION:
        raise MalformedWalkState(
            f"walk-state schema_version {state.get('schema_version')!r} != expected {SCHEMA_VERSION}: {path}"
        )

    return state


def _pid_dead(pid) -> bool:
    """True only when `pid` is PROVABLY not a live process on this host, so its lock is safe to
    steal. Conservative fail-safe: a non-int/None pid, a pid owned by another user (PermissionError),
    or any pid we cannot probe is treated as ALIVE (returns False) -> we never steal a lock we cannot
    prove is dead. Same-host only (the opus walk's holder is always a local teikoku process)."""
    if not isinstance(pid, int):
        return False
    try:
        os.kill(pid, 0)
        return False   # signal delivered -> alive
    except ProcessLookupError:
        return True    # no such process -> dead, safe to steal
    except (PermissionError, OSError):
        return False   # alive-but-not-ours, or unknowable -> conservative


def _requested_session_budget() -> tuple[int, int] | None:
    """Return a controller-requested ``(dispatch_cap, plan_cap)``.

    The shared state is used by controllers with different context windows.
    Their child process supplies both values at spawn time; persisting them
    while that child owns the lock makes ``record_dispatch`` and
    ``mark_plan_done`` enforce the active controller's real budget instead of
    relying on prompt compliance.  Both variables are required together so a
    partially configured process can never leave a mixed budget behind.
    """
    dispatch_raw = os.environ.get(DISPATCH_CAP_ENV)
    plan_raw = os.environ.get(PLAN_CAP_ENV)
    if dispatch_raw is None and plan_raw is None:
        return None
    if dispatch_raw is None or plan_raw is None:
        raise ValueError(
            f"{DISPATCH_CAP_ENV} and {PLAN_CAP_ENV} must be supplied together"
        )
    try:
        dispatch_cap = int(dispatch_raw)
        plan_cap = int(plan_raw)
    except ValueError as exc:
        raise ValueError("walk budget overrides must be positive integers") from exc
    if not 1 <= dispatch_cap <= 1_000 or not 1 <= plan_cap <= 100:
        raise ValueError("walk budget overrides are outside the safe range")
    return dispatch_cap, plan_cap


def _lock_owner_pid() -> int:
    """PID representing the long-lived controller, not a transient tool shell."""
    raw = os.environ.get(LOCK_OWNER_PID_ENV)
    if raw is None:
        return os.getpid()
    try:
        pid = int(raw)
    except ValueError as exc:
        raise ValueError(f"{LOCK_OWNER_PID_ENV} must be a positive integer") from exc
    if pid <= 0:
        raise ValueError(f"{LOCK_OWNER_PID_ENV} must be a positive integer")
    # Refuse a typo/stale injected owner. The controller must be alive at the
    # moment a short-lived Python tool records it.
    if _pid_dead(pid):
        raise ValueError(f"{LOCK_OWNER_PID_ENV} points to a dead process")
    return pid


def claim_lock(session_id: str, path: str = STATE_PATH) -> dict:
    """Claim the walk lock for session_id.

    - unlocked -> claim.
    - locked by the SAME session_id -> re-claim (heartbeat refresh).
    - locked by a DIFFERENT session_id with a fresh heartbeat (< HEARTBEAT_STALE_MIN old) AND a live
      holder pid -> LockRefused.
    - locked by a DIFFERENT session_id with a stale heartbeat (>= HEARTBEAT_STALE_MIN old) OR a DEAD
      holder pid -> steal the lock. The dead-pid steal (K-27, 2026-07-11) closes the respawn-burn
      window: an rc=1/crashed turn leaves the lock held, and without the pid check every fresh turn
      LockRefused-and-exited for the FULL 30m heartbeat threshold, burning ~1 paid Opus boot/turn.
    """
    with _StateMutex(path):
        state = load(path)
        existing = state.get("lock")
        abandoned_dead_session = False

        if existing is not None and existing.get("session_id") != session_id:
            held_since = datetime.fromisoformat(existing["heartbeat_iso"])
            age = _utc_now() - held_since
            holder_dead = _pid_dead(existing.get("pid"))
            if age < timedelta(minutes=HEARTBEAT_STALE_MIN) and not holder_dead:
                raise LockRefused(
                    f"lock held by {existing.get('session_id')!r}, heartbeat {age} old (< {HEARTBEAT_STALE_MIN}m stale threshold), holder pid {existing.get('pid')} alive"
                )
            abandoned_dead_session = holder_dead

        requested_budget = _requested_session_budget()
        if requested_budget is not None:
            dispatch_cap, plan_cap = requested_budget
            state["budget"]["dispatch_cap"] = dispatch_cap
            state["budget"]["plans_per_session_cap"] = plan_cap
        if abandoned_dead_session:
            # A crashed/quota-killed controller cannot run its STOP protocol.
            # Recover its unfinished budget window while stealing its provably
            # dead-PID lock, so the fresh controller receives its own complete
            # bounded window without anyone hand-editing walk-state.json.
            state["budget"]["sessions_completed"] += 1
            state["budget"]["dispatches_this_session"] = 0
            state["budget"]["plans_this_session"] = 0

        lock = {"session_id": session_id, "pid": _lock_owner_pid(), "heartbeat_iso": _utc_now_iso()}
        state["lock"] = lock
        _atomic_write(path, state)
        return lock


def heartbeat(session_id: str, path: str = STATE_PATH) -> dict:
    """Refresh the heartbeat on a lock already held by session_id."""
    with _StateMutex(path):
        state = load(path)
        existing = state.get("lock")
        if existing is None or existing.get("session_id") != session_id:
            raise LockRefused(f"no lock held by {session_id!r} to refresh")

        lock = {"session_id": session_id, "pid": _lock_owner_pid(), "heartbeat_iso": _utc_now_iso()}
        state["lock"] = lock
        _atomic_write(path, state)
        return lock


def release_lock(session_id: str, path: str = STATE_PATH) -> None:
    """Release the lock if held by session_id. A no-op if the lock is
    already None; raises LockRefused if held by a different session_id."""
    with _StateMutex(path):
        state = load(path)
        existing = state.get("lock")
        if existing is None:
            return
        if existing.get("session_id") != session_id:
            raise LockRefused(f"lock held by {existing.get('session_id')!r}, not {session_id!r}; refusing release")

        state["lock"] = None
        _atomic_write(path, state)


def update_cursor(plan_id, stage, pickup, path: str = STATE_PATH) -> dict:
    """Set the walk cursor. pickup must be a non-empty string (the exact next
    action a resuming session should take)."""
    if not pickup or not isinstance(pickup, str):
        raise ValueError("pickup must be a non-empty string")

    with _StateMutex(path):
        state = load(path)
        state["cursor"] = {"plan_id": plan_id, "stage": stage, "pickup": pickup}
        _atomic_write(path, state)
        return state["cursor"]


def seed_plan(plan_id, pickup, stage="precheck", path: str = STATE_PATH) -> dict:
    """Seed a plan into the walk: append it to the queue (idempotent) and point the
    cursor at it so the empty-queue guard re-arms. This is the legitimate way to hand
    the walk a new plan (the SKILL bans hand-editing walk-state.json). pickup is the
    exact first action for the resuming Opus. The cursor plan_id must NOT be in
    escalation_queue or the SET_ASIDE guard treats it as no-work."""
    if not plan_id or not isinstance(plan_id, str):
        raise ValueError("plan_id must be a non-empty string")
    if not pickup or not isinstance(pickup, str):
        raise ValueError("pickup must be a non-empty string")
    with _StateMutex(path):
        state = load(path)
        queue = state.setdefault("queue", [])
        if plan_id not in queue:
            queue.append(plan_id)
        state["cursor"] = {"plan_id": plan_id, "stage": stage, "pickup": pickup}
        _atomic_write(path, state)
        return {"queue": queue, "cursor": state["cursor"]}


def complete_plan(plan_id: str, path: str = STATE_PATH) -> dict:
    """Retire a drained plan from the live walk (the inverse of seed_plan):
    idempotently drop plan_id from the queue and, if the cursor points at it,
    clear the cursor to the idle terminal shape (plan_id None) that the
    empty-queue / no-work guards read as 'nothing to run'. Idempotent: a plan_id
    absent from both the queue and the cursor is a no-op. Used by
    opus_supervisor._reconcile_done_plans to self-heal when an Opus turn commits a
    plan but exits before its own mark_plan_done bookkeeping runs."""
    if not plan_id or not isinstance(plan_id, str):
        raise ValueError("plan_id must be a non-empty string")
    with _StateMutex(path):
        state = load(path)
        state["queue"] = [p for p in (state.get("queue") or []) if p != plan_id]
        cursor = state.get("cursor") or {}
        if cursor.get("plan_id") == plan_id:
            state["cursor"] = {"plan_id": None, "stage": None, "pickup": None}
        _atomic_write(path, state)
        return {"queue": state["queue"], "cursor": state.get("cursor")}


def append_decision(what: str, why: str, evidence: str, path: str = STATE_PATH) -> dict:
    """Append a decision row with an auto-incrementing D-<n> id."""
    with _StateMutex(path):
        state = load(path)
        decisions = state["decisions"]
        entry = {
            "id": f"D-{len(decisions) + 1}",
            "date": _utc_now().date().isoformat(),
            "what": what,
            "why": why,
            "evidence": evidence,
        }
        decisions.append(entry)
        _atomic_write(path, state)
        return entry


def _disposition_path_for(state_path: str) -> str:
    """The ledger sits beside the state file, so a test passing a temp state
    path writes its dispositions to that temp dir - never the real ledger
    (mirrors how mark_plan_done derives plan-status/ from the state path)."""
    return os.path.join(os.path.dirname(state_path) or ".", "plan-dispositions.json")


def _plan_index_path_for(ledger_path: str) -> str:
    """The shared corpus plan-index sits beside the disposition ledger / state file, so a test passing
    a temp path upserts into that temp dir -- never the real plan-index.json (mirrors
    _disposition_path_for + how mark_plan_done derives plan-status/ from the state path)."""
    return os.path.join(os.path.dirname(ledger_path) or ".", "plan-index.json")


def _upsert_plan_index(plan_id: str, state_value: str, ledger_path: str) -> None:
    """Best-effort: reflect a terminal verdict into the shared corpus plan-index so corpus_drain's
    ordered drain sees LIVE delivery (drained->wired, escalated/blocked/analyzed->partial). NEVER
    breaks the walk/land: a missing index (nothing to upsert into -- corpus build_index OWNS creation),
    an uncarded unit (no matching row -- the SUMMARY fallback tier covers it), or any read/write failure
    is logged and swallowed. Only keeps an EXISTING carded row's state live."""
    try:
        idx_path = _plan_index_path_for(ledger_path)
        if not os.path.isfile(idx_path):
            return
        with open(idx_path, "r", encoding="utf-8") as fh:
            data = json.load(fh)
        units = data.get("units") if isinstance(data, dict) else None
        if not isinstance(units, list):
            return
        for u in units:
            if isinstance(u, dict) and u.get("id") == plan_id:
                if u.get("state") == state_value:
                    return  # already reflects this verdict -- no rewrite
                u["state"] = state_value
                _atomic_write(idx_path, data)
                return
        # uncarded unit: no row to update -- corpus fallback (SUMMARY tier) covers it, nothing to do.
    except Exception as exc:  # noqa: BLE001 - projection only, never break the walk/land
        import sys as _sys
        print(f"[walk_state] plan-index upsert failed for {plan_id!r}: "
              f"{type(exc).__name__}: {exc}", file=_sys.stderr)


def _emit_disposition(plan_id, verdict, reason, evidence_ref="", commit_sha="", path: str = DISPOSITIONS_PATH) -> None:
    """Best-effort upsert of a terminal disposition into the teikoku-local
    ledger (same row shape as the F: canonical ledger). NEVER breaks the walk:
    a ledger-write failure is logged and swallowed, because the authoritative
    verdict already landed in walk-state.json / plan-status/{id}.json - the
    ledger is only the cross-harness projection the F: sync harvests."""
    try:
        now = _utc_now_iso()
        # The disposition ledger has independent writers; atomic replace alone does not serialize
        # their read-modify-write cycles.  Reuse the cross-process mutex for this ledger path.
        with _StateMutex(path):
            if os.path.exists(path):
                with open(path, encoding="utf-8") as fh:
                    data = json.load(fh)
            else:
                data = {"schema_version": SCHEMA_VERSION, "dispositions": {}}
            data.setdefault("dispositions", {})
            prior = data["dispositions"].get(plan_id)
            data["dispositions"][plan_id] = {
                "plan_id": plan_id,
                "verdict": verdict,
                "review_state": "settled" if verdict in ("drained", "delivered", "claimed_delivered") else verdict,
                "implementation_state": (
                    "claimed_delivered" if verdict in ("drained", "delivered", "claimed_delivered")
                    else "blocked" if verdict in ("escalated", "set_aside", "blocked")
                    else "unknown"
                ),
                "proof_state": "candidate" if commit_sha and evidence_ref else "missing",
                "reason": reason,
                "evidence_ref": evidence_ref or "",
                "source_walk": SOURCE_WALK,
                "commit_sha": commit_sha or "",
                "first_seen": prior.get("first_seen", now) if prior else now,
                "updated": now,
            }
            _atomic_write(path, data)
        # Deliberately do not mutate plan-index here.  It is an analyst declaration/index, not a
        # proof ledger; the old drained->wired upsert inflated both the walk and the dashboard.
    except Exception as exc:  # noqa: BLE001 - projection only, never break the walk
        import sys as _sys
        print(f"[walk_state] disposition emit failed for {plan_id!r}: "
              f"{type(exc).__name__}: {exc}", file=_sys.stderr)


def append_escalation(plan_id: str, reason: str, attempts: int, gate_json_ref: str, path: str = STATE_PATH) -> dict:
    """Append an escalation_queue row for a SET_ASIDE plan (S3 greenfield route or
    the escalation-ladder second-fail). Idempotent per plan_id: a re-escalation
    of the same plan overwrites its prior row rather than duplicating it."""
    with _StateMutex(path):
        state = load(path)
        queue = state.setdefault("escalation_queue", [])
        entry = {
            "plan_id": plan_id,
            "reason": reason,
            "attempts": attempts,
            "gate_json_ref": gate_json_ref,
            "ts": _utc_now_iso(),
        }
        for i, row in enumerate(queue):
            if row.get("plan_id") == plan_id:
                queue[i] = entry
                break
        else:
            queue.append(entry)
        _atomic_write(path, state)
        _emit_disposition(plan_id, "escalated", reason, evidence_ref=gate_json_ref,
                          path=_disposition_path_for(path))
        return entry


def resolve_escalation(plan_id: str, reason: str = "", path: str = STATE_PATH) -> dict:
    """Remove plan_id from escalation_queue so seed_plan can re-enter it.

    Owner-ratified / un-gated units must leave escalation_queue via this helper
    (never by hand-editing walk-state.json). Idempotent: absent plan_id is a
    no-op. Does NOT seed the queue or move the cursor — call seed_plan after.
    """
    if not plan_id or not isinstance(plan_id, str):
        raise ValueError("plan_id must be a non-empty string")
    with _StateMutex(path):
        state = load(path)
        before = state.get("escalation_queue") or []
        removed = [row for row in before if row.get("plan_id") == plan_id]
        state["escalation_queue"] = [row for row in before if row.get("plan_id") != plan_id]
        _atomic_write(path, state)
        if removed and reason:
            _emit_disposition(
                plan_id,
                "resolved",
                reason=reason,
                evidence_ref=(removed[0].get("gate_json_ref") or ""),
                path=_disposition_path_for(path),
            )
        return {
            "plan_id": plan_id,
            "removed": bool(removed),
            "prior": removed[0] if removed else None,
            "escalation_queue_len": len(state["escalation_queue"]),
        }


def record_dispatch(path: str = STATE_PATH) -> tuple[int, bool]:
    """Increment budget.dispatches_this_session; return (count, tripped)."""
    with _StateMutex(path):
        state = load(path)
        budget = state["budget"]
        budget["dispatches_this_session"] += 1
        count = budget["dispatches_this_session"]
        tripped = count >= budget["dispatch_cap"]
        _atomic_write(path, state)
        return count, tripped


def session_rollover(session_id: str, path: str = STATE_PATH) -> dict:
    """Close out the current session's budget window (stop-protocol step 2):
    increments budget.sessions_completed and zeroes the per-session counters
    (dispatches_this_session, plans_this_session) so the NEXT session starts
    with a fresh window. Refuses (LockRefused) unless session_id holds the
    lock — only the stopping session may roll the window."""
    with _StateMutex(path):
        state = load(path)
        existing = state.get("lock")
        if existing is None or existing.get("session_id") != session_id:
            raise LockRefused(f"session_rollover requires the lock; not held by {session_id!r}")

        budget = state["budget"]
        budget["sessions_completed"] += 1
        budget["dispatches_this_session"] = 0
        budget["plans_this_session"] = 0
        _atomic_write(path, state)
        return budget


def mark_plan_done(plan_id: str, commit_sha, gate_evidence_ref, path: str = STATE_PATH) -> tuple[int, bool]:
    """Write/update the plan-status row for plan_id and bump
    budget.plans_this_session. A DONE row with no commit_sha or
    gate_evidence_ref is impossible to write: raises DoneRowRejected and
    leaves the state file and the plan-status directory untouched."""
    if not commit_sha:
        raise DoneRowRejected(f"mark_plan_done({plan_id!r}): commit_sha is required and must be non-empty")
    if not gate_evidence_ref:
        raise DoneRowRejected(f"mark_plan_done({plan_id!r}): gate_evidence_ref is required and must be non-empty")

    with _StateMutex(path):
        state = load(path)
        state["budget"]["plans_this_session"] += 1
        count = state["budget"]["plans_this_session"]
        tripped = count >= state["budget"]["plans_per_session_cap"]

        plans_dir = os.path.join(os.path.dirname(path) or ".", "plan-status")
        row = {
            "plan_id": plan_id,
            "status": "DONE",
            "commit_sha": commit_sha,
            "gate_evidence_ref": gate_evidence_ref,
            "last_run_iso": _utc_now_iso(),
        }
        row_path = os.path.join(plans_dir, f"{plan_id}.json")
        _atomic_write(row_path, row)
        _atomic_write(path, state)
        _emit_disposition(plan_id, "drained",
                          reason=f"drained: gated + committed {commit_sha}",
                          evidence_ref=gate_evidence_ref, commit_sha=commit_sha,
                          path=_disposition_path_for(path))
        return count, tripped


def dispositioned_ids(path: str = DISPOSITIONS_PATH) -> set:
    """The set of plan_ids that already carry a terminal disposition row in the
    teikoku-local ledger (drained/escalated/...). A plan in this set has already
    been ruled on by the walk, so it is NOT fresh re-seed fodder. Read-only and
    fail-safe: a missing or unreadable ledger returns an empty set and never
    raises, so a caller consulting it to skip re-seeding a settled plan can never
    break the walk."""
    try:
        with open(path, "r", encoding="utf-8") as fh:
            data = json.load(fh)
    except (OSError, json.JSONDecodeError):
        return set()
    if not isinstance(data, dict):
        return set()
    dispositions = data.get("dispositions")
    return set(dispositions.keys()) if isinstance(dispositions, dict) else set()


def is_plan_done(plan_id: str, path: str = STATE_PATH) -> bool:
    """True when plan-status/{plan_id}.json (beside the state file, the same
    directory mark_plan_done writes) exists with status == "DONE". Read-only and
    fail-safe: a missing, unreadable, or non-DONE row returns False and never
    raises, so a reconcile pass that consults it can never break the walk."""
    plans_dir = os.path.join(os.path.dirname(path) or ".", "plan-status")
    row_path = os.path.join(plans_dir, f"{plan_id}.json")
    if not os.path.isfile(row_path):
        return False
    try:
        with open(row_path, "r", encoding="utf-8") as fh:
            row = json.load(fh)
    except (OSError, json.JSONDecodeError):
        return False
    if not isinstance(row, dict) or row.get("status") != "DONE":
        return False
    # Temp-path callers are unit fixtures.  Production reads fail closed on the complete proof
    # contract: reachable landed SHA plus durable repo-local evidence (never /tmp or prose only).
    if os.path.realpath(path) != os.path.realpath(STATE_PATH):
        return True
    sha = str(row.get("commit_sha") or "").strip()
    evidence = str(row.get("gate_evidence_ref") or "").strip()
    if not re.fullmatch(r"[0-9a-fA-F]{7,40}", sha) or not evidence or "/tmp" in evidence:
        return False
    elai_root = os.environ.get(
        "ELAI_WORKFLOW_ROOT",
        os.path.realpath(os.path.join(os.path.dirname(__file__), "..", "..", "elai")),
    )
    landed = subprocess.run(
        ["git", "-C", elai_root, "merge-base", "--is-ancestor", sha, "HEAD"],
        capture_output=True,
        timeout=10,
    ).returncode == 0
    if not landed:
        return False
    candidates = re.findall(
        r"(?:/home/researcher/teikoku/[^\s,;)]+|conductor/[^\s,;)]+|opus_walk/[^\s,;)]+|"
        r"\.elai_cc/[^\s,;)]+|workflow/[^\s,;)]+)", evidence,
    )
    teikoku_root = os.path.realpath(os.path.join(os.path.dirname(__file__), "..", ".."))
    allowed_roots = (
        os.path.realpath(os.path.join(os.path.dirname(__file__), "evidence")),
        os.path.realpath(os.path.join(os.path.dirname(__file__), "dispatch-logs")),
    )
    artifact_re = re.compile(r"(?:gate|land|audit|verdict|result|proof)", re.I)
    for token in candidates:
        token = token.rstrip(".:+]")
        if token.startswith("/home/researcher/teikoku/"):
            evidence_path = token
        elif token.startswith("conductor/"):
            evidence_path = os.path.join(teikoku_root, token)
        elif token.startswith("opus_walk/"):
            evidence_path = os.path.join(teikoku_root, "conductor", token)
        else:
            evidence_path = os.path.join(elai_root, token)
        resolved = os.path.realpath(evidence_path)
        if not any(resolved == root or resolved.startswith(root + os.sep) for root in allowed_roots):
            continue
        if os.path.isfile(resolved) and artifact_re.search(os.path.basename(resolved)) and os.path.getsize(resolved) > 0:
            return True
        if os.path.isdir(resolved):
            for dirpath, _dirs, files in os.walk(resolved):
                for name in files:
                    candidate = os.path.join(dirpath, name)
                    if artifact_re.search(name) and os.path.getsize(candidate) > 0:
                        return True
    return False
