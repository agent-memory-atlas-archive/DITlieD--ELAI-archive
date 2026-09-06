"""glm_walk_control.py - start/stop/status for the glm-orchestrator walk mode.

Isolated sibling of walk_control.py (which drives the DETERMINISTIC elai walk via
elai_walk_driver.sh). This one drives glm_walk_driver.sh -> glm_supervisor.py, where
the installed ZCode GLM-5.2 agent is the orchestration loop. Kept as its own module so the reviewed
walk_control internals (single elai driver/sentinel/pgrep assumptions) are untouched;
slice_api routes provider=="glm-orchestrator" here.

Same lifecycle contract as walk_control: idempotent start (a second start while the
chain runs is a no-op), controller-local cancellation, and status via pgrep.
"""
from __future__ import annotations

import os
import subprocess
from pathlib import Path

import walk_controller_registry as controller_registry

CONDUCTOR_DIR = Path(__file__).resolve().parent
DRIVER = CONDUCTOR_DIR / "glm_walk_driver.sh"
STOP_SENTINEL = CONDUCTOR_DIR / "GLM_WALK_STOP"
WALK_LOG = "/tmp/glm-walk.log"
# pgrep pattern matching the running glm walk: the T6-entry10 auto-restart wrapper
# (glm_walk_chain.py, alive for the WHOLE lifecycle including between-chain backoff) OR a supervisor
# chain (glm_supervisor.py, alive during each in-flight chain, and for a manual direct run).
# The wrapper leg is what makes the single-instance guard correct across a backoff window (no
# supervisor is alive then, but the wrapper is, so a second start still no-ops). ERE alternation (pgrep
# uses extended regex); never matches the elai driver nor this query process. procps-ng verified.
_SUPERVISOR_PATTERN = r"glm_walk/glm_(walk_chain|supervisor)\.py"
_OPUS_PATTERN = r"opus_walk/opus_(walk_chain|supervisor)\.py"
_FALLBACK_PATTERN = r"fallback_walk/fallback_supervisor\.py"
PROVIDER_ID = "glm-orchestrator"
SHARED_WALK_STATE = CONDUCTOR_DIR / "opus_walk" / "walk-state.json"

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


def _opus_pids() -> list[int]:
    return controller_registry.controller_pids("opus-orchestrator")


def _fallback_pids() -> list[int]:
    return controller_registry.controller_pids("fallback-orchestrator")


def glm_walk_status() -> dict:
    """{ok, running, pids, stop_pending, provider}. running=the supervisor chain is
    alive; stop_pending=a STOP sentinel is set (a graceful stop is in flight, waiting
    for the current GLM turn to finish)."""
    pids = _supervisor_pids()
    return {
        "ok": True,
        "running": bool(pids),
        "pids": pids,
        "stop_pending": STOP_SENTINEL.is_file(),
        "provider": PROVIDER_ID,
        "model": "GLM-5.2",
        "context_window": 1_000_000,
        "walk_state": str(SHARED_WALK_STATE),
    }


def _glm_walk_start_locked() -> dict:
    """Start the GLM resume chain if neither controller already owns the walk.

    The GLM and Opus controllers also claim the same walk_state lock, so this
    pre-spawn process check is a fast refusal rather than the sole exclusion.
    Idempotent: a second GLM start
    while the chain runs is a no-op. Clears the STOP sentinel and spawns the driver
    DETACHED (start_new_session) so it outlives this request and survives a conductor
    restart. The GLM lane needs no retier: the GLM supervisor drains via the shared
    walk-state queue, not the conductor's PAID/FREE tier flip."""
    if _supervisor_pids():
        return {"ok": True, "running": True, "already": True, "message": "glm walk already running"}
    opus_pids = _opus_pids()
    if opus_pids:
        return {
            "ok": False,
            "running": False,
            "provider": PROVIDER_ID,
            "conflict": "opus-orchestrator",
            "error": "refusing GLM start: Opus controller is active on the shared walk lock",
        }
    fallback_pids = _fallback_pids()
    if fallback_pids:
        return {
            "ok": False,
            "running": False,
            "provider": PROVIDER_ID,
            "conflict": "fallback-orchestrator",
            "error": "refusing GLM start: fallback controller is active on the shared walk lock",
        }
    if not DRIVER.is_file():
        return {"ok": False, "running": False, "error": f"driver not found: {DRIVER}"}
    # No supervisor is running (guard above) -> any active canvas task is an orphan from a crashed
    # prior chain; reap it so the fresh chain starts on a clean board (no phantom "task active").
    reaped = _reap_stale_canvas_tasks("glm walk start: reaped orphan canvas task")
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
    return {"ok": True, "running": True, "provider": PROVIDER_ID, "message": "glm walk started"}


def glm_walk_start() -> dict:
    with controller_registry.launch_guard():
        return _glm_walk_start_locked()


def glm_walk_stop() -> dict:
    """Cancel only this lane through GLM_WALK_STOP; never signal or reap Opus."""
    try:
        STOP_SENTINEL.write_text("stop requested via dashboard\n", encoding="utf-8")
    except OSError as exc:
        return {"ok": False, "error": f"{type(exc).__name__}: {exc}"}
    running = bool(_supervisor_pids())
    reaped: list[str] = []
    if not running and not _opus_pids() and not _fallback_pids():
        # The walk is already dead but a stop was requested (e.g. clearing a stuck state). With no
        # supervisor alive, any active canvas task is an orphan -> reap it. A GRACEFUL stop of a LIVE
        # chain does NOT reap (the in-flight worker is real and finishes on its own).
        reaped = _reap_stale_canvas_tasks("glm walk stop: reaped orphan canvas task")
    return {
        "ok": True,
        "running": running,
        "stop_pending": True,
        "reaped_tasks": reaped,
        "provider": PROVIDER_ID,
        "message": "stop requested; the active ZCode GLM turn is cancelled and its chain exits",
    }
