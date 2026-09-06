#!/usr/bin/env python3
"""TEIKOKU P3-D (master-plan 3.5 / D-07): event-driven session wake.

A waiter SLEEPS and is WOKEN by a dep-done EVENT, never poll-spams. The wake source
is the existing append-only events table (D-08): a TASK_STATUS_CHANGED row with
new_status=DONE is the "dep done" signal (conductor_db.update_task_status). The
cross-process wake channel is the slice_api ThreadingHTTPServer (127.0.0.1:8787),
extended with a blocking GET /events/wait long-poll and a POST /events/notify publish.

This module has two faces:
  HUB SIDE (imported in-process by slice_api): an event registry guarded by a
    threading.Condition + a monotonic published watermark. register_event() bumps the
    watermark and notifies; wait_for_new() blocks the long-poll until a relevant event
    arrives or the server timeout elapses.
  CLIENT SIDE: publish() signals the hub (in-process if shared, else a localhost POST,
    best-effort); wait_for_dep() blocks on the hub long-poll, re-checking the dep SQL
    on each wake, until all of a task's deps are DONE.

LOST-WAKEUP SAFETY (correctness, not optional): the waiter records the latest
events.event_seq (monotonic AUTOINCREMENT, conductor_db.append_event) as a watermark
BEFORE blocking, and RE-CHECKS the dep state AFTER registering interest. A DONE event
that fires between the dep-check and the block is NOT lost: the next long-poll passes
since=last_seen and the hub returns the already-recorded event, and the post-wake dep
re-check catches it regardless. A long-interval safety re-check + a hard max-wait
self-heal a truly-lost notify or a dead hub (BACKSTOP, not the wake mechanism).

Stdlib only (threading, sqlite3, json, time, urllib).
"""
from __future__ import annotations

import json
import sqlite3
import threading
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any, Callable

# The shared hub transport (the existing slice_api server). A client with no explicit
# url targets this; named constant, no magic string.
DEFAULT_HUB_URL = "http://127.0.0.1:8787"

# BACKSTOP cadence (FORK F): the hub long-poll returns on its own server-side timeout to
# trigger a waiter re-check. >=30s is deliberately NOT poll-spam (D-07 kills the 60s/500ms
# tight loop); it is the lost-wakeup / dead-hub self-heal interval. The hard max-wait maps
# to the orchestrate TRIPWIRE T3 (dep-wait > 4h surfaces to the owner).
SAFETY_INTERVAL_SECONDS = 30
MAX_WAIT_SECONDS = 14400  # 4h, the T3 ceiling


# --------------------------------------------------------------------------- hub side
class _Hub:
    """In-process event registry shared by every thread of one server process.

    register_event() is called by the publish path (in-process or via POST /events/notify);
    wait_for_new() is called by the GET /events/wait long-poll handler thread. The Condition
    lets a long-poll BLOCK (releasing the GIL) until a new event is published, so the route
    is a real sleep, not a busy-poll.
    """

    def __init__(self) -> None:
        self._cond = threading.Condition()
        self._latest_seq = 0
        # ring of the most-recent published events so a long-poll arriving slightly late
        # (since < latest) still gets the events it missed (watermark replay).
        self._recent: list[dict[str, Any]] = []
        self._recent_cap = 256

    def register_event(self, event: dict[str, Any]) -> None:
        seq = int(event.get("event_seq") or 0)
        with self._cond:
            if seq > self._latest_seq:
                self._latest_seq = seq
            self._recent.append(event)
            if len(self._recent) > self._recent_cap:
                self._recent = self._recent[-self._recent_cap:]
            self._cond.notify_all()

    def latest_seq(self) -> int:
        with self._cond:
            return self._latest_seq

    def wait_for_new(
        self,
        since: int,
        project: str | None,
        timeout: float,
        recipient: str | None = None,
    ) -> list[dict[str, Any]]:
        """Block until an event with event_seq > since (matching project, and addressed to
        ``recipient`` when given) is published, or until ``timeout`` seconds elapse. Returns
        the matching events (possibly empty on timeout). project=None matches every project;
        recipient=None matches every event (no addressing filter).

        TEIKOKU P4-A (D-07 mailbox): the recipient filter lets a long-poll wake ONLY for
        events addressed to it (an addressed session_message carries recipient==<to session>),
        so a blocked inbox-wait is not woken by unrelated traffic. The recipient is matched
        against the published event dict's top-level "recipient" key (wake.publish carries it)."""
        deadline = time.monotonic() + max(0.0, timeout)
        with self._cond:
            while True:
                hits = [
                    e
                    for e in self._recent
                    if int(e.get("event_seq") or 0) > since
                    and (project is None or e.get("project_id") == project)
                    and (recipient is None or e.get("recipient") == recipient)
                ]
                if hits:
                    return hits
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    return []
                self._cond.wait(timeout=remaining)


# the single process-wide hub instance (slice_api imports this module, so its server
# threads and any in-process emitter share ONE hub).
HUB = _Hub()

# TEIKOKU teikoku-polishA (D10 token-streaming): the event_type a live assistant-text increment
# rides. A chat_text_delta is a TRANSIENT in-process hub event (NOT a durable DB row): it carries
# the CUMULATIVE assistant text so far so a dropped delta self-heals (the next one replaces it).
# The frontend renders it into the in-flight placeholder and MUST NOT advance its real-event
# watermark off it (the synthetic seq lives above the DB watermark; treating it as a real seq
# would filter out genuine DB events). Named constant, no magic string.
EVENT_CHAT_TEXT_DELTA = "chat_text_delta"

# Synthetic-seq guard for delta events: a process-wide monotonic counter so rapid deltas are
# STRICTLY increasing (each new delta out-ranks the prior). Seeded above the hub watermark at
# emit time so a delta's seq is always > the frontend's `since` (which only ever reflects real
# DB events <= the watermark). Guarded by its own lock; never reused across the process.
_delta_seq_lock = threading.Lock()
_delta_seq_counter = 0


def _next_delta_seq() -> int:
    """Allocate a synthetic, strictly-increasing event_seq for a transient delta event, kept
    ABOVE the current hub watermark so the hub's `event_seq > since` filter always delivers it
    to a frontend whose `since` tracks only real DB events. Process-wide monotonic; never reused.
    """
    global _delta_seq_counter
    with _delta_seq_lock:
        floor = HUB.latest_seq() + 1
        if _delta_seq_counter < floor:
            _delta_seq_counter = floor
        else:
            _delta_seq_counter += 1
        return _delta_seq_counter


def publish_delta(
    event_type: str,
    recipient: str | None,
    payload: dict[str, Any],
    *,
    project_id: str | None = None,
) -> int:
    """Emit a TRANSIENT in-process hub event (TEIKOKU token-streaming) addressed to ``recipient``
    and return the synthetic seq used. In-process only: register_event wakes any same-process
    /events/wait long-poll (the chat_turn handler thread and the long-poll handler thread share
    THIS hub object, ThreadingHTTPServer). It is NOT written to the DB events table (a streamed
    increment is ephemeral; only the final POST /chat/turn response is authoritative). NEVER
    raises -- a hub-register failure must not break the streaming turn (best-effort, like publish).
    """
    seq = _next_delta_seq()
    event = {
        "event_seq": seq,
        "event_type": event_type,
        "task_id": None,
        "project_id": project_id,
        "recipient": recipient,
        "payload": payload,
    }
    try:
        register_event(event)
    except Exception:
        pass
    return seq


def register_event(event: dict[str, Any]) -> None:
    """Hub-side entry: record a published event and wake any blocked long-poll."""
    HUB.register_event(event)


def wait_for_new(
    since: int,
    project: str | None,
    timeout: float,
    recipient: str | None = None,
) -> list[dict[str, Any]]:
    """Hub-side entry: the GET /events/wait long-poll body. recipient (TEIKOKU P4-A) filters
    to events addressed to it (the mailbox-wake leg); None preserves the P3-D behaviour."""
    return HUB.wait_for_new(since, project, timeout, recipient)


# ------------------------------------------------------------------------ client side
def publish(
    event_seq: int | None,
    event_type: str,
    task_id: str | None,
    project_id: str | None,
    *,
    recipient: str | None = None,
    hub_url: str = DEFAULT_HUB_URL,
) -> bool:
    """Signal the hub that a new event landed. BEST-EFFORT: a down/absent hub MUST NOT
    raise (the caller is on the DB-write hot path; a wake-notify failure is never allowed
    to break a status update). Returns True if the notify was delivered.

    In-process the registry is the same object slice_api serves, so register_event() alone
    wakes a same-process long-poll. The POST is the cross-process leg (the emitter and the
    hub may be different OS processes); it is wrapped so any transport error is swallowed.

    TEIKOKU P4-A (D-07 mailbox): recipient is the addressed session id for a session_message
    publish; it rides the event dict's top-level "recipient" key so wait_for_new's recipient
    filter can wake ONLY the target. None (the P3-D dep-done path) means a broadcast wake.
    """
    event = {
        "event_seq": event_seq,
        "event_type": event_type,
        "task_id": task_id,
        "project_id": project_id,
        "recipient": recipient,
    }
    # in-process: always record locally (covers the same-process waiter).
    try:
        register_event(event)
    except Exception:
        pass
    # cross-process: POST to the hub, best-effort.
    try:
        data = json.dumps(event).encode("utf-8")
        req = urllib.request.Request(
            hub_url.rstrip("/") + "/events/notify",
            data=data,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=2) as resp:
            return 200 <= resp.status < 300
    except Exception:
        return False


def latest_event_seq(db_path: str | Path) -> int:
    """The current monotonic watermark (MAX(events.event_seq)). 0 on an empty/absent log.
    Used so a waiter records last_seen BEFORE blocking (lost-wakeup safety)."""
    try:
        conn = sqlite3.connect(str(db_path))
        try:
            row = conn.execute("SELECT MAX(event_seq) FROM events").fetchone()
            return int(row[0]) if row and row[0] is not None else 0
        finally:
            conn.close()
    except sqlite3.Error:
        return 0


def deps_satisfied(db_path: str | Path, task_id: str) -> bool:
    """True iff EVERY dep of ``task_id`` is DONE (the per-task form of the ready_task_rows
    NOT-EXISTS clause, conductor_db.ready_task_rows). A task with no deps is satisfied.
    Read-only; does not consider the task's own status (the loop owns that)."""
    conn = sqlite3.connect(str(db_path))
    try:
        row = conn.execute(
            """
            SELECT COUNT(*) FROM task_deps d
            JOIN tasks dep ON dep.task_id = d.depends_on_task_id
            WHERE d.task_id = ? AND dep.status != 'DONE'
            """,
            (task_id,),
        ).fetchone()
        return (row[0] if row else 0) == 0
    finally:
        conn.close()


def wait_for_dep(
    db_path: str | Path,
    task_id: str,
    hub_url: str = DEFAULT_HUB_URL,
    *,
    max_wait_seconds: float = MAX_WAIT_SECONDS,
    safety_interval_seconds: float = SAFETY_INTERVAL_SECONDS,
    predicate: Callable[[], bool] | None = None,
) -> bool:
    """BLOCK until every dep of ``task_id`` is DONE (D-07 dep-done wake), then return True.
    Return False only on the hard max-wait timeout (TRIPWIRE T3).

    No busy-poll: the wait blocks on the hub long-poll (GET /events/wait), which itself
    sleeps on a threading.Condition until a real event is published. The ONLY timed wakeups
    are the long-interval safety re-checks (>= safety_interval_seconds), the lost-wakeup /
    dead-hub backstop, NOT the wake mechanism.

    Lost-wakeup safety: last_seen is recorded from the watermark, the dep SQL is re-checked
    AFTER recording interest and again on every wake; a DONE between the check and the block
    is returned by the next long-poll (since=last_seen) and caught by the re-check.
    """
    check = predicate or (lambda: deps_satisfied(db_path, task_id))
    deadline = time.monotonic() + max(0.0, max_wait_seconds)
    # record the watermark BEFORE the first dep-check so a DONE racing in after the check
    # still has event_seq > last_seen and is returned by the first long-poll.
    last_seen = latest_event_seq(db_path)
    if check():
        return True
    while True:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return False
        poll_timeout = min(safety_interval_seconds, remaining)
        events = _hub_wait(hub_url, last_seen, poll_timeout)
        for e in events:
            seq = int(e.get("event_seq") or 0)
            if seq > last_seen:
                last_seen = seq
        # re-check on every wake (event-driven) AND on every safety tick (backstop).
        if check():
            return True


def _hub_wait(
    hub_url: str,
    since: int,
    timeout: float,
    *,
    project: str | None = None,
    recipient: str | None = None,
) -> list[dict[str, Any]]:
    """Client long-poll against GET /events/wait. On any transport error (dead hub) return
    [] so the caller falls back to its safety re-check (the dead-hub self-heal).

    TEIKOKU P4-A: project + recipient narrow the server-side filter so a mailbox-waiter's
    long-poll only returns (and only wakes on) events addressed to it (the wait_for_message
    leg). Both default None, preserving the P3-D dep-wait long-poll behaviour."""
    from urllib.parse import urlencode

    params: dict[str, str] = {"since": str(int(since))}
    if project is not None:
        params["project"] = project
    if recipient is not None:
        params["recipient"] = recipient
    url = hub_url.rstrip("/") + "/events/wait?" + urlencode(params)
    # client read timeout slightly above the server-side long-poll timeout so the server
    # returns its empty payload first; a hard +5s guards a hung connection.
    try:
        req = urllib.request.Request(url, method="GET")
        with urllib.request.urlopen(req, timeout=timeout + 5) as resp:
            body = resp.read().decode("utf-8")
        parsed = json.loads(body)
        return parsed.get("events", []) if isinstance(parsed, dict) else []
    except (urllib.error.URLError, OSError, ValueError):
        # dead hub / timeout / bad body: sleep out the interval, then the caller re-checks.
        time.sleep(min(timeout, 1.0))
        return []


def unread_for(db_path: str | Path, recipient: str, *, project_id: str | None = None) -> bool:
    """True iff at least one session_message addressed to ``recipient`` sits past that
    session's consumed-watermark (TEIKOKU P4-A mailbox; the wait_for_message predicate).

    Read-only, stdlib sqlite. Addressing rides the event payload's "to" field (FORK F1 OPTION
    A: zero schema migration); the consumed-watermark is the MAX(up_to_seq) over that
    session's appended ``session_inbox_consumed`` events (append-only read cursor, no row
    mutation). A message with event_seq > watermark AND payload.to == recipient is unread.
    The mailbox module reuses this so the hub-wake predicate and the read path agree exactly.
    """
    try:
        conn = sqlite3.connect(str(db_path))
        conn.row_factory = sqlite3.Row
        try:
            watermark = 0
            wm_rows = conn.execute(
                "SELECT payload_json FROM events WHERE event_type = ?",
                ("session_inbox_consumed",),
            ).fetchall()
            for r in wm_rows:
                try:
                    pj = json.loads(r["payload_json"] or "{}")
                except (ValueError, TypeError):
                    continue
                if pj.get("session") == recipient:
                    up_to = int(pj.get("up_to_seq") or 0)
                    if up_to > watermark:
                        watermark = up_to
            sql = (
                "SELECT event_seq, payload_json FROM events "
                "WHERE event_type = ? AND event_seq > ?"
            )
            args: list[Any] = ["session_message", watermark]
            if project_id is not None:
                sql += " AND project_id = ?"
                args.append(project_id)
            rows = conn.execute(sql, tuple(args)).fetchall()
            for r in rows:
                try:
                    pj = json.loads(r["payload_json"] or "{}")
                except (ValueError, TypeError):
                    continue
                if pj.get("to") == recipient:
                    return True
            return False
        finally:
            conn.close()
    except sqlite3.Error:
        return False


def wait_for_message(
    db_path: str | Path,
    recipient: str,
    hub_url: str = DEFAULT_HUB_URL,
    *,
    project_id: str | None = None,
    max_wait_seconds: float = MAX_WAIT_SECONDS,
    safety_interval_seconds: float = SAFETY_INTERVAL_SECONDS,
) -> bool:
    """BLOCK until an UNREAD session_message addressed to ``recipient`` exists (TEIKOKU P4-A,
    D-07 mailbox-wake), then return True. Return False only on the hard max-wait (TRIPWIRE T3).

    The mailbox analog of wait_for_dep: the SAME lost-wakeup-safe loop, with the dep-check
    replaced by unread_for(recipient). The waiter SLEEPS on the hub long-poll (GET /events/wait
    filtered to recipient) and re-checks the unread predicate on every wake AND every safety
    tick (the dead-hub / lost-notify backstop) -- never a busy-poll (D-07 kills the 500ms tick).

    Lost-wakeup safety (correctness): last_seen is recorded from the watermark BEFORE the first
    unread-check, so a message racing in between the check and the block still has
    event_seq > last_seen and is returned by the first long-poll; the post-wake re-check catches
    it regardless. The recipient filter on the long-poll means unrelated traffic does not even
    wake this waiter (a tighter wake than dep-wait's project-only filter).
    """

    def check() -> bool:
        return unread_for(db_path, recipient, project_id=project_id)

    deadline = time.monotonic() + max(0.0, max_wait_seconds)
    last_seen = latest_event_seq(db_path)
    if check():
        return True
    while True:
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return False
        poll_timeout = min(safety_interval_seconds, remaining)
        events = _hub_wait(
            hub_url, last_seen, poll_timeout, project=project_id, recipient=recipient
        )
        for e in events:
            seq = int(e.get("event_seq") or 0)
            if seq > last_seen:
                last_seen = seq
        if check():
            return True
