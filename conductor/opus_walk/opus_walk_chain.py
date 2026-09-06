#!/usr/bin/env python3
"""opus_walk_chain.py - auto-restart-with-backoff wrapper around the opus walk chain (T6-entry10).

The driver (opus_walk_driver.sh) runs THIS instead of opus_supervisor.py directly. Each iteration
runs ONE full supervisor chain as a SUBPROCESS (`python3 opus_walk/opus_supervisor.py`) -- so the
`opus_walk/opus_supervisor.py` process identity every monitor + the control layer key on stays live
for the in-flight window -- then reads the terminal halt class the supervisor stamps into
last-chain-result.json. When the chain halted on a CLEAN NO-WORK class (queue drained / nothing to
seed / quorum quiet), it restarts the chain after an escalating backoff (15/30/60 min) so a walk that
drained its queue re-checks for new work instead of dying. A chain that made progress RESETS the
backoff; the STOP sentinel, a DISK_LOW twice in a row, an auth fast-fail storm, any error halt, or the
consecutive-no-progress cap ends the loop.

Escape hatch: OPUS_WALK_AUTORESTART=off -> run the chain exactly once and exit (the pre-T6 driver
behaviour). The loop is a detached daemon that DIES WITH the OPUS_WALK_STOP sentinel: it is re-checked
before each restart AND on every tick of the backoff sleep, so opus_walk_stop still stops the walk.

Grounded seams: opus_supervisor.run_supervisor + its last-chain-result.json stamp (this file's only
contract with the supervisor); events.append_event (the same canvas/audit surface the supervisor uses).
"""
from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path

_HERE = Path(__file__).resolve().parent            # ~/teikoku/conductor/opus_walk
CONDUCTOR = _HERE.parent                            # ~/teikoku/conductor
sys.path.insert(0, str(CONDUCTOR))
sys.path.insert(0, str(_HERE))

import events                                       # noqa: E402  (canvas + audit emit)

# --- fixed wiring (named constants) ------------------------------------------
SUPERVISOR = _HERE / "opus_supervisor.py"
STOP_SENTINEL = CONDUCTOR / "OPUS_WALK_STOP"
CHAIN_RESULT_PATH = _HERE / "last-chain-result.json"
DB_PATH = CONDUCTOR / "state" / "control.db"
CANVAS_PLAN_ID = "teikoku-opus-walk"
AUTORESTART_ENV = "OPUS_WALK_AUTORESTART"
# Escalating backoff between no-work restarts; the wrapper resets to index 0 whenever a chain made
# progress. Last element is the cap (60 min) held indefinitely while no progress is made.
BACKOFF_SCHEDULE_S = (15 * 60, 30 * 60, 60 * 60)
# Give up (stop the loop with a clear log line) after this many CONSECUTIVE no-progress restarts.
MAX_NO_PROGRESS_RESTARTS = 12
# Halt classes safe to auto-restart on a backoff: the chain genuinely had nothing to do this pass, so
# a later pass may find fresh work (a new open-gap seed, a refreshed status snapshot, a landed dep).
_RESTART_CLEAN = frozenset({"CHAIN_NO_WORK", "NO_SPAWN", "QUORUM_QUIET"})
_HALT_DISK_LOW = "DISK_LOW"
# Backoff sleep granularity: check the STOP sentinel this often so a stop during backoff dies promptly.
_BACKOFF_TICK_S = 5


def _autorestart_enabled() -> bool:
    """Auto-restart is ON unless env OPUS_WALK_AUTORESTART=off (run the chain once and exit)."""
    return (os.environ.get(AUTORESTART_ENV) or "on").strip().lower() != "off"


def _emit(event_type: str, payload: dict) -> None:
    """Append a lifecycle event to control.db (canvas wake surface + audit trail). Fail-soft: a
    telemetry write must never kill the loop, but a failure is surfaced on stderr, never swallowed."""
    try:
        events.append_event(
            str(DB_PATH), event_type, task_id=CANVAS_PLAN_ID,
            payload=payload, project_id="elai", session_id=CANVAS_PLAN_ID,
        )
    except Exception as exc:  # noqa: BLE001 - telemetry is best-effort by design
        print(f"[opus-walk-chain] WARN emit {event_type} failed: "
              f"{type(exc).__name__}: {exc}", file=sys.stderr, flush=True)


def _read_chain_result(path: Path = CHAIN_RESULT_PATH) -> dict:
    """Read the supervisor's terminal halt stamp. A missing/unreadable file -> {} (the caller treats an
    absent stamp as an UNKNOWN halt and does NOT restart -- fail-safe: never loop on a silent exit)."""
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}
    return data if isinstance(data, dict) else {}


def _run_one_chain(spawn=subprocess.run) -> dict:
    """Run ONE supervisor chain as a subprocess and return {halt_class, made_progress, rc}. The stale
    stamp is removed first so a chain that dies WITHOUT writing one is read as UNKNOWN (no restart),
    never as a stale prior halt. stdout/stderr are inherited (they land in the driver's /tmp log)."""
    try:
        CHAIN_RESULT_PATH.unlink()
    except OSError:
        pass
    proc = spawn([sys.executable, str(SUPERVISOR)], cwd=str(CONDUCTOR))
    # Pass the module global explicitly (late-bound) so a test patching CHAIN_RESULT_PATH is honoured;
    # a bare _read_chain_result() would use the def-time default and miss the patch.
    result = _read_chain_result(CHAIN_RESULT_PATH)
    return {
        "halt_class": result.get("halt_class", "UNKNOWN"),
        "made_progress": bool(result.get("made_progress", False)),
        "rc": getattr(proc, "returncode", None),
        "detail": result.get("detail", {}),
    }


def _restart_decision(halt_class: str, prev_halt_class: str | None) -> tuple[bool, str]:
    """Whether to auto-restart, given this halt class and the PREVIOUS one. Clean no-work classes
    restart; a single DISK_LOW restarts (disk may free up) but two in a row does not; every other class
    (STOPPED, FAST_FAIL_STORM, ERROR_*, UNKNOWN) never restarts -- those are operator halts."""
    if halt_class in _RESTART_CLEAN:
        return True, halt_class
    if halt_class == _HALT_DISK_LOW:
        if prev_halt_class == _HALT_DISK_LOW:
            return False, "disk_low_twice_consecutive"
        return True, "disk_low_once"
    if halt_class == "FAST_FAIL_STORM":
        stamp = Path("/tmp/opus-token-rescue-stamp")
        import time as _t
        try:
            age = _t.time() - stamp.stat().st_mtime if stamp.is_file() else 1e12
        except OSError:
            age = 1e12
        if age > 6 * 3600:
            try:
                stamp.write_text(str(int(_t.time())), encoding="utf-8")
            except OSError:
                pass
            return True, "fast_fail_token_rescue_once"
    # Turn wall-clock timeout (e.g. 3h hung Opus) used to halt forever. Overnight monitor
    # needs one rescue so the queue can keep draining (2026-07-13 B-38 TIMEOUT death).
    if halt_class == "ERROR_TIMEOUT":
        stamp = Path("/tmp/opus-timeout-rescue-stamp")
        import time as _t
        try:
            age = _t.time() - stamp.stat().st_mtime if stamp.is_file() else 1e12
        except OSError:
            age = 1e12
        if age > 2 * 3600:  # at most once per 2h
            try:
                stamp.write_text(str(int(_t.time())), encoding="utf-8")
            except OSError:
                pass
            return True, "timeout_rescue_once"
    return False, halt_class


def _wait_backoff(backoff_s: int, sleep=time.sleep) -> bool:
    """Sleep backoff_s, re-checking the STOP sentinel every _BACKOFF_TICK_S. Returns True to proceed
    with the restart, False if the sentinel appeared (the loop must exit). The loop dies with the
    sentinel: this is the only long sleep in the wrapper and it is fully sentinel-interruptible."""
    remaining = backoff_s
    while remaining > 0:
        if STOP_SENTINEL.is_file():
            return False
        step = min(_BACKOFF_TICK_S, remaining)
        sleep(step)
        remaining -= step
    return not STOP_SENTINEL.is_file()


def _default_sync_token(force: bool = False) -> None:
    """Re-run the supervisor's token sync before an auto-restart so a fresh chain never 401s on an
    expired claude-elai OAuth token. Lazy import (opus_supervisor pulls heavy siblings) + fail-soft.
    force=True is used by the FAST_FAIL_STORM rescue path."""
    try:
        import opus_supervisor
        opus_supervisor._sync_elai_token_from_max(force=force)
    except Exception as exc:  # noqa: BLE001 - best-effort; a sync failure never blocks the restart
        print(f"[opus-walk-chain] WARN token sync skipped before restart: "
              f"{type(exc).__name__}: {exc}", file=sys.stderr, flush=True)


def _rc_int(rc) -> int:
    """Coerce a subprocess return code (possibly None) into an int exit code for the wrapper."""
    return rc if isinstance(rc, int) else 1


def run_chain_loop(run_one=_run_one_chain, wait=_wait_backoff, sync_token=_default_sync_token) -> int:
    """The auto-restart loop. Runs supervisor chains, restarting on a clean no-work halt after an
    escalating backoff, until the STOP sentinel, a non-restartable halt, or the no-progress cap ends it.
    run_one / wait / sync_token are injected so tests never spawn a subprocess or sleep for real."""
    _emit("OPUS_WALK_CHAINLOOP_START",
          {"autorestart": _autorestart_enabled(), "backoff_schedule_s": list(BACKOFF_SCHEDULE_S),
           "no_progress_cap": MAX_NO_PROGRESS_RESTARTS})
    consecutive_no_progress = 0
    backoff_idx = 0
    prev_halt: str | None = None
    while True:
        if STOP_SENTINEL.is_file():
            print("[opus-walk-chain] STOP sentinel present; chain loop exit before running a chain",
                  flush=True)
            return 0
        result = run_one()
        halt_class = result["halt_class"]
        progressed = result["made_progress"]
        rc = result["rc"]
        print(f"[opus-walk-chain] chain exit halt_class={halt_class} made_progress={progressed} "
              f"rc={rc}", flush=True)

        if STOP_SENTINEL.is_file():
            print("[opus-walk-chain] STOP sentinel present after chain; graceful exit", flush=True)
            return 0
        if not _autorestart_enabled():
            print("[opus-walk-chain] OPUS_WALK_AUTORESTART=off; single chain done, exit", flush=True)
            return _rc_int(rc)

        do_restart, reason = _restart_decision(halt_class, prev_halt)
        if not do_restart:
            print(f"[opus-walk-chain] not auto-restarting (reason={reason}); chain loop exit",
                  flush=True)
            _emit("OPUS_WALK_CHAINLOOP_HALT",
                  {"reason": reason, "halt_class": halt_class, "made_progress": progressed})
            return _rc_int(rc)

        if progressed:
            backoff_idx = 0
            consecutive_no_progress = 0
        else:
            consecutive_no_progress += 1
            if consecutive_no_progress > MAX_NO_PROGRESS_RESTARTS:
                print(f"[opus-walk-chain] restart cap hit ({consecutive_no_progress} consecutive "
                      f"no-progress restarts > {MAX_NO_PROGRESS_RESTARTS}); giving up", flush=True)
                _emit("OPUS_WALK_CHAINLOOP_GIVEUP",
                      {"consecutive": consecutive_no_progress, "cap": MAX_NO_PROGRESS_RESTARTS,
                       "halt_class": halt_class})
                return _rc_int(rc)

        # FAST_FAIL_STORM rescue uses a short 60s backoff (not the 15/30/60min no-work schedule).
        if reason in ("fast_fail_token_rescue_once", "timeout_rescue_once"):
            backoff_s = 60
        else:
            backoff_s = BACKOFF_SCHEDULE_S[min(backoff_idx, len(BACKOFF_SCHEDULE_S) - 1)]
        _emit("OPUS_WALK_AUTORESTART",
              {"backoff_s": backoff_s, "reason": reason, "halt_class": halt_class,
               "consecutive": consecutive_no_progress, "made_progress": progressed})
        print(f"[opus-walk-chain] auto-restart in {backoff_s}s (reason={reason}, "
              f"consecutive_no_progress={consecutive_no_progress})", flush=True)
        if not wait(backoff_s):
            print("[opus-walk-chain] STOP sentinel during backoff; chain loop exit", flush=True)
            return 0
        if reason != "fast_fail_token_rescue_once":
            backoff_idx = min(backoff_idx + 1, len(BACKOFF_SCHEDULE_S) - 1)
        prev_halt = halt_class
        # Force-sync for token rescue; normal restarts stay best-effort near-expiry sync.
        if reason in ("fast_fail_token_rescue_once", "timeout_rescue_once"):
            try:
                sync_token(True)  # type: ignore[misc,call-arg]
            except TypeError:
                sync_token()
        else:
            sync_token()  # keep the OAuth token fresh so the restarted chain never 401s at startup


def main() -> int:
    print(f"[opus-walk-chain] loop start {time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())} "
          f"autorestart={_autorestart_enabled()}", flush=True)
    return run_chain_loop()


if __name__ == "__main__":
    raise SystemExit(main())
