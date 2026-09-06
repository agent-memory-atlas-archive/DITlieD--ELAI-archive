from __future__ import annotations

import os
import signal
import threading
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Any


@dataclass(frozen=True)
class CancelResult:
    in_process_cancelled: bool = False
    pid_signalled: bool = False
    pid: int | None = None
    error: str | None = None

    def to_json_dict(self) -> dict[str, Any]:
        return {
            "in_process_cancelled": self.in_process_cancelled,
            "pid_signalled": self.pid_signalled,
            "pid": self.pid,
            "error": self.error,
        }


@dataclass
class _ActiveWorker:
    db_path: str
    task_id: str
    lease_id: str | None
    cancel: Callable[[], None]


_LOCK = threading.Lock()
_ACTIVE: dict[tuple[str, str], _ActiveWorker] = {}


def register_active_worker(
    db_path: str | Path,
    task_id: str,
    lease_id: str | None,
    cancel: Callable[[], None],
) -> None:
    key = (_db_key(db_path), task_id)
    with _LOCK:
        _ACTIVE[key] = _ActiveWorker(key[0], task_id, lease_id, cancel)


def unregister_active_worker(db_path: str | Path, task_id: str) -> None:
    key = (_db_key(db_path), task_id)
    with _LOCK:
        _ACTIVE.pop(key, None)


def cancel_active_worker(
    db_path: str | Path,
    task_id: str,
    *,
    lease_id: str | None = None,
    pid: int | None = None,
) -> CancelResult:
    in_process_cancelled = False
    error = None
    key = (_db_key(db_path), task_id)
    with _LOCK:
        worker = _ACTIVE.get(key)
    if worker is not None and (lease_id is None or worker.lease_id == lease_id):
        try:
            worker.cancel()
            in_process_cancelled = True
        except Exception as exc:  # pragma: no cover - defensive cancellation path
            error = str(exc)

    pid_signalled = False
    if pid is not None:
        try:
            _signal_pid(pid)
            pid_signalled = True
        except ProcessLookupError:
            pid_signalled = False
        except Exception as exc:  # pragma: no cover - platform-specific fallback
            error = error or str(exc)

    return CancelResult(
        in_process_cancelled=in_process_cancelled,
        pid_signalled=pid_signalled,
        pid=pid,
        error=error,
    )


def _signal_pid(pid: int) -> None:
    if os.name == "nt":
        ctrl_break = getattr(signal, "CTRL_BREAK_EVENT", None)
        if ctrl_break is not None:
            os.kill(pid, ctrl_break)
            return
        os.kill(pid, signal.SIGTERM)
        return
    try:
        os.killpg(pid, signal.SIGTERM)
    except ProcessLookupError:
        raise
    except Exception:
        os.kill(pid, signal.SIGTERM)


def _db_key(db_path: str | Path) -> str:
    text = str(db_path)
    if text == ":memory:":
        return text
    return str(Path(text).resolve())
