"""opus_walk_control.py - start/stop/status for the opus-orchestrator walk mode.

Isolated sibling of walk_control.py (which drives the DETERMINISTIC elai walk via
elai_walk_driver.sh). This one drives opus_walk_driver.sh -> opus_supervisor.py: the
auto-opus-context-management resume chain where a paid Opus IS the orchestration loop
(see opus_walk/opus_supervisor.py). Kept as its OWN module so the shipped, reviewed
walk_control internals (single elai driver/sentinel/pgrep assumptions) are untouched;
slice_api routes provider=="opus-orchestrator" here.

Same lifecycle contract as walk_control: idempotent start (a second start while the
chain runs is a no-op), GRACEFUL stop (the STOP sentinel is checked between Opus turns
so a running turn finishes), status via pgrep of the supervisor process.
"""
from __future__ import annotations

import os
import subprocess
from pathlib import Path

import walk_controller_registry as controller_registry

CONDUCTOR_DIR = Path(__file__).resolve().parent
DRIVER = CONDUCTOR_DIR / "opus_walk_driver.sh"
STOP_SENTINEL = CONDUCTOR_DIR / "OPUS_WALK_STOP"
WALK_LOG = "/tmp/opus-walk.log"
# pgrep pattern matching the running opus walk: the T6-entry10 auto-restart wrapper
# (opus_walk_chain.py, alive for the WHOLE lifecycle including between-chain backoff) OR a supervisor
# chain (opus_supervisor.py, alive during each in-flight chain, and for a manual/legacy direct run).
# The wrapper leg is what makes the single-instance guard correct across a backoff window (no
# supervisor is alive then, but the wrapper is, so a second start still no-ops). ERE alternation (pgrep
# uses extended regex); never matches the elai driver nor this query process. procps-ng verified.
_SUPERVISOR_PATTERN = r"opus_walk/opus_(walk_chain|supervisor)\.py"
_GLM_PATTERN = r"glm_walk/glm_(walk_chain|supervisor)\.py"
_FALLBACK_PATTERN = r"fallback_walk/fallback_supervisor\.py"
PROVIDER_ID = "opus-orchestrator"

# Canvas plan_id + the DB the dispatch primitive upserts worker rows under (mirrors opus_dispatch).
_DB_PATH = CONDUCTOR_DIR / "state" / "control.db"
_CANVAS_PLAN_ID = "teikoku-opus-walk"
# In-flight worker statuses: a canvas task in one of these with NO supervisor alive is a GHOST (its
# opus_dispatch process died before update_task_status ran -- the orphaned-worker class). See the
# 2026-07-06T17:20 INTEGRATE row that showed "1 task active" on the canvas with the walk stopped.
_ACTIVE_STATUSES = ("RUNNING", "LEASED", "VERIFYING", "INTEGRATING")


def _reap_stale_canvas_tasks(reason: str) -> list[str]:
    """Cancel any non-terminal teikoku-opus-walk worker task. Callers MUST ensure no supervisor is
    running first (no live supervisor -> no live worker -> every active canvas row is an orphan).
    Best-effort: a DB error never blocks start/stop. Returns the reaped task_ids."""
    reaped: list[str] = []
    try:
        from conductor_db import ControlDB  # local import: keep module import cheap + cycle-free
        db = ControlDB(_DB_PATH)
        placeholders = ",".join("?" for _ in _ACTIVE_STATUSES)
        with db.transaction() as conn:
            rows = conn.execute(
                f"SELECT task_id FROM tasks WHERE plan_id = ? AND status IN ({placeholders})",
                (_CANVAS_PLAN_ID, *_ACTIVE_STATUSES),
            ).fetchall()
        for row in rows:
            tid = row["task_id"] if not isinstance(row, tuple) else row[0]
            try:
                db.update_task_status(tid, "CANCELLED", reason=reason)
                reaped.append(tid)
            except (KeyError, ValueError):
                pass
    except Exception:  # noqa: BLE001 - reaping is best-effort; never break start/stop
        pass
    return reaped


def _pids(pattern: str) -> list[int]:
    try:
        out = subprocess.run(
            ["pgrep", "-f", pattern], capture_output=True, text=True, timeout=5
        )
    except (OSError, subprocess.SubprocessError):
        return []
    return [int(tok) for tok in out.stdout.split() if tok.strip().isdigit()]


def _supervisor_pids() -> list[int]:
    return controller_registry.controller_pids(PROVIDER_ID)


def _glm_pids() -> list[int]:
    return controller_registry.controller_pids("glm-orchestrator")


def _fallback_pids() -> list[int]:
    return controller_registry.controller_pids("fallback-orchestrator")


def opus_walk_status() -> dict:
    """{ok, running, pids, stop_pending, provider}. running=the supervisor chain is
    alive; stop_pending=a STOP sentinel is set (a graceful stop is in flight, waiting
    for the current Opus turn to finish)."""
    pids = _supervisor_pids()
    return {
        "ok": True,
        "running": bool(pids),
        "pids": pids,
        "stop_pending": STOP_SENTINEL.is_file(),
        "provider": PROVIDER_ID,
    }


def _opus_walk_start_locked() -> dict:
    """Start the opus resume chain if not already running. Idempotent: a second start
    while the chain runs is a no-op. Clears the STOP sentinel and spawns the driver
    DETACHED (start_new_session) so it outlives this request and survives a conductor
    restart. The Opus lane needs no retier: the opus supervisor drains via its own
    walk-state queue, not the conductor's PAID/FREE tier flip."""
    if _supervisor_pids():
        return {"ok": True, "running": True, "already": True, "message": "opus walk already running"}
    if _glm_pids():
        return {
            "ok": False, "running": False, "provider": PROVIDER_ID,
            "conflict": "glm-orchestrator",
            "error": "refusing Opus start: GLM controller is active on the shared walk lock",
        }
    if _fallback_pids():
        return {
            "ok": False, "running": False, "provider": PROVIDER_ID,
            "conflict": "fallback-orchestrator",
            "error": "refusing Opus start: fallback controller is active on the shared walk lock",
        }
    if not DRIVER.is_file():
        return {"ok": False, "running": False, "error": f"driver not found: {DRIVER}"}
    # No supervisor is running (guard above) -> any active canvas task is an orphan from a crashed
    # prior chain; reap it so the fresh chain starts on a clean board (no phantom "task active").
    reaped = _reap_stale_canvas_tasks("opus walk start: reaped orphan canvas task")
    try:
        STOP_SENTINEL.unlink()
    except OSError:
        pass
    try:
        with open(WALK_LOG, "ab") as log:
            subprocess.Popen(
                ["bash", str(DRIVER)],
                cwd=str(CONDUCTOR_DIR),
                stdout=log,
                stderr=log,
                stdin=subprocess.DEVNULL,
                start_new_session=True,
                env=dict(os.environ),
            )
    except OSError as exc:
        return {"ok": False, "running": False, "error": f"{type(exc).__name__}: {exc}"}
    return {"ok": True, "running": True, "provider": PROVIDER_ID, "message": "opus walk started"}


def opus_walk_start() -> dict:
    with controller_registry.launch_guard():
        return _opus_walk_start_locked()


def opus_walk_stop() -> dict:
    """Request a GRACEFUL stop: write the STOP sentinel. The supervisor checks it
    BETWEEN Opus turns, so the in-flight turn finishes (no Opus killed mid-orchestration)
    and then the chain exits."""
    try:
        STOP_SENTINEL.write_text("stop requested via dashboard\n", encoding="utf-8")
    except OSError as exc:
        return {"ok": False, "error": f"{type(exc).__name__}: {exc}"}
    running = bool(_supervisor_pids())
    reaped: list[str] = []
    if not running and not _glm_pids() and not _fallback_pids():
        # The walk is already dead but a stop was requested (e.g. clearing a stuck state). With no
        # supervisor alive, any active canvas task is an orphan -> reap it. A GRACEFUL stop of a LIVE
        # chain does NOT reap (the in-flight worker is real and finishes on its own).
        reaped = _reap_stale_canvas_tasks("opus walk stop: reaped orphan canvas task")
    return {
        "ok": True,
        "running": running,
        "stop_pending": True,
        "reaped_tasks": reaped,
        "message": "stop requested; the opus chain exits after the current turn finishes",
    }
