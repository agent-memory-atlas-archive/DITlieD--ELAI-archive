#!/usr/bin/env python3
"""Bounded replay of parked opus-land rows."""
from __future__ import annotations

import datetime as dt
import fcntl
import hashlib
import json
import os
import signal
import stat
import subprocess
import sys
import uuid
from pathlib import Path
from typing import Any, Callable

try:
    import pending_lands
except ModuleNotFoundError:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import pending_lands

CONDUCTOR = Path(__file__).resolve().parents[1]
OPUS_LAND = Path(__file__).resolve().with_name("opus_land.py")
LEDGER = Path(__file__).resolve().with_name("pending-lands.json")
DEADLINE_SECONDS = 10800
MAX_ATTEMPTS = 2
BACKOFF_BASE_SECONDS = 900
BACKOFF_MAX_SECONDS = 3600
WAITING_FOR_VERIFIER = "WAITING_FOR_VERIFIER"


class ReplayError(RuntimeError):
    """A replay state transition failed."""


def _now() -> dt.datetime:
    return dt.datetime.now(dt.timezone.utc)


def _iso(value: dt.datetime) -> str:
    return value.astimezone(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def _parse_iso(value: Any) -> dt.datetime | None:
    if not isinstance(value, str) or not value.strip():
        return None
    try:
        parsed = dt.datetime.fromisoformat(value.strip().replace("Z", "+00:00"))
    except ValueError:
        return None
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=dt.timezone.utc)
    return parsed.astimezone(dt.timezone.utc)


def _attempt_number(value: Any) -> int:
    try:
        return max(0, int(value or 0))
    except (TypeError, ValueError, OverflowError):
        return 0


def backoff_seconds(attempt: int) -> int:
    if attempt < 1:
        raise ValueError("attempt must be positive")
    return min(900 * 2 ** (attempt - 1), 3600)


def _worker_id() -> str:
    return f"u3b-replay-{os.getpid()}-{uuid.uuid4().hex}"


def _canonical_args_sha256(args: dict[str, Any]) -> str:
    return pending_lands._canonical_args_sha256(args)


def _read_data(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, TypeError):
        return {"schema_version": pending_lands.RECORD_VERSION, "lands": []}
    if not isinstance(data, dict) or not isinstance(data.get("lands"), list):
        return {"schema_version": pending_lands.RECORD_VERSION, "lands": []}
    return data


def _atomic_save(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temp = path.with_name(f".{path.name}.{os.getpid()}.{uuid.uuid4().hex}.tmp")
    try:
        temp.write_text(json.dumps(data, indent=2, sort_keys=True), encoding="utf-8")
        os.replace(temp, path)
    finally:
        if temp.exists():
            temp.unlink()


def _candidate_rows(path: Path) -> list[dict[str, Any]]:
    waiting = pending_lands.list_waiting_for_verifier(path)
    waiting_keys = {
        (row.get("plan_id"), row.get("worktree"))
        for row in waiting
        if isinstance(row, dict)
    }
    rows = pending_lands.load_rows(path)
    candidates = []
    for row in rows:
        if not isinstance(row, dict) or not row.get("replayable"):
            continue
        if row.get("record_version") != pending_lands.RECORD_VERSION:
            continue
        key = (row.get("plan_id"), row.get("worktree"))
        if key not in waiting_keys:
            continue
        candidates.append(row)
    return candidates


def _eligible(row: dict[str, Any], now: dt.datetime) -> bool:
    if row.get("stale") or row.get("owner_attention"):
        return False
    if _attempt_number(row.get("attempt")) >= MAX_ATTEMPTS:
        return False
    next_at = _parse_iso(row.get("next_eligible_iso"))
    return next_at is None or next_at <= now


def _mutate_claimed(
    path: Path,
    row: dict[str, Any],
    mutator: Callable[[dict[str, Any]], Any],
    *,
    allow_unclaimed: bool = False,
) -> tuple[bool, Any]:
    plan_id = row.get("plan_id")
    worktree = row.get("worktree")
    claim_token = row.get("claim_token")
    lock_path = path.with_name(path.name + ".lock")
    path.parent.mkdir(parents=True, exist_ok=True)
    with lock_path.open("a+", encoding="utf-8") as lock:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
        try:
            data = _read_data(path)
            lands = list(data.get("lands") or [])
            for index, original in enumerate(lands):
                if not isinstance(original, dict):
                    continue
                if original.get("plan_id") != plan_id or original.get("worktree") != worktree:
                    continue
                current_token = original.get("claim_token")
                if current_token != claim_token and not (allow_unclaimed and not current_token):
                    return False, None
                updated = dict(original)
                value = mutator(updated)
                lands[index] = updated
                data["schema_version"] = pending_lands.RECORD_VERSION
                data["lands"] = lands
                _atomic_save(path, data)
                return True, value
            return False, None
        finally:
            fcntl.flock(lock.fileno(), fcntl.LOCK_UN)


def _remove_claimed(path: Path, row: dict[str, Any]) -> tuple[bool, bool]:
    plan_id = row.get("plan_id")
    worktree = row.get("worktree")
    claim_token = row.get("claim_token")
    lock_path = path.with_name(path.name + ".lock")
    path.parent.mkdir(parents=True, exist_ok=True)
    with lock_path.open("a+", encoding="utf-8") as lock:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
        try:
            data = _read_data(path)
            lands = list(data.get("lands") or [])
            found = False
            removed = False
            kept = []
            for original in lands:
                if not isinstance(original, dict):
                    kept.append(original)
                    continue
                same = original.get("plan_id") == plan_id and original.get("worktree") == worktree
                if not same:
                    kept.append(original)
                    continue
                found = True
                if original.get("claim_token") == claim_token:
                    removed = True
                    continue
                kept.append(original)
            if removed:
                data["schema_version"] = pending_lands.RECORD_VERSION
                data["lands"] = kept
                _atomic_save(path, data)
            return found, removed
        finally:
            fcntl.flock(lock.fileno(), fcntl.LOCK_UN)


def _release(row: dict[str, Any], path: Path) -> bool:
    return pending_lands.release_claim(
        row.get("plan_id"),
        claim_token=row.get("claim_token"),
        path=path,
        worktree=row.get("worktree", ""),
    ) is not None


def _release_checked(row: dict[str, Any], path: Path) -> None:
    if not _release(row, path):
        raise ReplayError("replay claim release failed")


def _mark_stale(row: dict[str, Any], path: Path, now: dt.datetime) -> dict[str, Any]:
    def update(target: dict[str, Any]) -> None:
        target.update(
            {
                "stale": True,
                "stale_reason": "missing_worktree",
                "missing_worktree": target.get("worktree", ""),
                "stale_at_iso": _iso(now),
                "next_eligible_iso": None,
            }
        )
        for key in ("claim_token", "claimed_by", "claimed_at_iso", "lease_seconds"):
            target.pop(key, None)

    changed, _ = _mutate_claimed(path, row, update)
    if not changed:
        raise ReplayError("stale transition lost its claim")
    return {
        "status": "stale",
        "plan_id": row.get("plan_id"),
        "worktree": row.get("worktree"),
        "missing_worktree": row.get("worktree"),
        "retained": True,
    }


def _record_failure(
    row: dict[str, Any],
    path: Path,
    now: dt.datetime,
    failure: str,
    detail: str = "",
) -> dict[str, Any]:
    attempt = _attempt_number(row.get("attempt")) + 1
    delay = backoff_seconds(attempt)
    next_at = now + dt.timedelta(seconds=delay)

    def update(target: dict[str, Any]) -> None:
        target.update(
            {
                "attempt": attempt,
                "next_eligible_iso": _iso(next_at),
                "last_replay_at_iso": _iso(now),
                "last_replay_status": failure,
                "last_replay_detail": detail[-1000:],
                "backoff_seconds": delay,
                "owner_attention": attempt >= MAX_ATTEMPTS,
            }
        )
        for key in ("claim_token", "claimed_by", "claimed_at_iso", "lease_seconds"):
            target.pop(key, None)

    changed, _ = _mutate_claimed(path, row, update)
    if not changed:
        released = _release(row, path)
        if not released:
            raise ReplayError("failure transition could not release its claim")
        raise ReplayError("failure transition lost its ledger row")
    return {
        "status": "failed",
        "failure": failure,
        "detail": detail[-1000:],
        "plan_id": row.get("plan_id"),
        "attempt": attempt,
        "backoff_seconds": delay,
        "next_eligible_iso": _iso(next_at),
        "owner_attention": attempt >= MAX_ATTEMPTS,
        "retained": True,
    }


def _kill_process_group(process: Any) -> None:
    pid = getattr(process, "pid", None)
    if pid is None:
        return
    try:
        os.killpg(pid, signal.SIGKILL)
    except ProcessLookupError:
        return
    except OSError:
        kill = getattr(process, "kill", None)
        if callable(kill):
            kill()


def _text(value: Any) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    return str(value)


def _land_succeeded(returncode: Any, stdout: str, stderr: str) -> bool:
    combined = f"{stdout}\n{stderr}"
    decoder = json.JSONDecoder()
    verdicts: list[dict[str, Any]] = []
    for index, character in enumerate(combined):
        if character != "{":
            continue
        try:
            value, _ = decoder.raw_decode(combined[index:])
        except json.JSONDecodeError:
            continue
        if isinstance(value, dict):
            verdicts.append(value)
    if returncode != 0:
        return any(
            value.get("verdict") == "LANDED_PUSH_FAILED" and value.get("landed") is True
            for value in verdicts
        )
    if any(value.get("verdict") == "LANDED" and value.get("landed") is True for value in verdicts):
        return True
    return any(line.strip() == "STATUS: LANDED" for line in combined.splitlines())


def _argv_is_safe(argv: Any) -> bool:
    if not isinstance(argv, list) or not all(isinstance(item, str) for item in argv):
        return False
    forbidden = {"--gates-only", "--no-verifier"}
    return not any(item in forbidden for item in argv)


# WIRE-DARK[U3B-REPLAY]: called by fallback_supervisor through its bounded dynamic import.
def replay_one(
    *,
    path: Path | str = LEDGER,
    deadline_seconds: float = DEADLINE_SECONDS,
    now: dt.datetime | None = None,
    claimed_by: str | None = None,
    opus_land_path: Path | str = OPUS_LAND,
    popen_factory: Callable[..., Any] | None = None,
) -> dict[str, Any] | None:
    ledger = Path(path)
    current = now or _now()
    worker = claimed_by or _worker_id()
    claimed = None
    for candidate in _candidate_rows(ledger):
        if not _eligible(candidate, current):
            continue
        claimed = pending_lands.claim_row(
            candidate.get("plan_id"),
            worker,
            path=ledger,
            worktree=candidate.get("worktree", ""),
        )
        if claimed is not None:
            break
    if claimed is None:
        return None

    try:
        args = claimed.get("replay_args")
        if not isinstance(args, dict):
            _release_checked(claimed, ledger)
            return {
                "status": "refused",
                "reason": "missing_replay_args",
                "plan_id": claimed.get("plan_id"),
            }
        if claimed.get("args_sha256") != _canonical_args_sha256(args):
            _release_checked(claimed, ledger)
            return {
                "status": "refused",
                "reason": "args_hash_mismatch",
                "plan_id": claimed.get("plan_id"),
            }
        if args.get("gates_only") is not False or args.get("no_verifier") is not False:
            _release_checked(claimed, ledger)
            return {
                "status": "refused",
                "reason": "disallowed_replay_flag",
                "plan_id": claimed.get("plan_id"),
            }
        argv = claimed.get("replay_argv")
        if not _argv_is_safe(argv):
            _release_checked(claimed, ledger)
            return {
                "status": "refused",
                "reason": "unsafe_replay_argv",
                "plan_id": claimed.get("plan_id"),
            }

        worktree = Path(str(claimed.get("worktree", "")))
        try:
            worktree_stat = worktree.stat()
        except OSError:
            worktree_stat = None
        if worktree_stat is None or not stat.S_ISDIR(worktree_stat.st_mode):
            return _mark_stale(claimed, ledger, current)

        command = [sys.executable, str(opus_land_path), *argv]
        env = dict(os.environ)
        python_path = [
            str(CONDUCTOR),
            str(Path(__file__).resolve().parent),
            str(Path(opus_land_path).resolve().parent),
        ]
        if env.get("PYTHONPATH"):
            python_path.append(env["PYTHONPATH"])
        env["PYTHONPATH"] = os.pathsep.join(python_path)
        factory = popen_factory or subprocess.Popen
        process = factory(
            command,
            cwd=str(CONDUCTOR),
            env=env,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            start_new_session=True,
        )
        try:
            stdout, stderr = process.communicate(timeout=float(deadline_seconds))
        except subprocess.TimeoutExpired:
            _kill_process_group(process)
            try:
                stdout, stderr = process.communicate(timeout=5)
            except Exception:
                stdout, stderr = "", ""
            return _record_failure(
                claimed,
                ledger,
                current,
                "deadline",
                f"deadline_seconds={deadline_seconds}; stdout={_text(stdout)[-300:]}; stderr={_text(stderr)[-300:]}",
            )
        stdout = _text(stdout)
        stderr = _text(stderr)
        returncode = getattr(process, "returncode", None)
        if _land_succeeded(returncode, stdout, stderr):
            found, removed = _remove_claimed(ledger, claimed)
            if found and not removed:
                raise ReplayError("successful land row is claimed by another replay")
            return {
                "status": "landed",
                "plan_id": claimed.get("plan_id"),
                "returncode": returncode,
                "removed": True,
                "stdout_tail": stdout[-1000:],
                "stderr_tail": stderr[-1000:],
            }
        return _record_failure(
            claimed,
            ledger,
            current,
            "land_failed",
            f"returncode={returncode}; stdout={stdout[-500:]}; stderr={stderr[-500:]}",
        )
    except Exception as exc:
        if claimed is not None:
            try:
                _record_failure(claimed, ledger, current, "exception", f"{type(exc).__name__}: {exc}")
            except Exception as transition_exc:
                raise ReplayError(
                    f"{type(exc).__name__}: {exc}; claim transition failed: "
                    f"{type(transition_exc).__name__}: {transition_exc}"
                ) from exc
        return {
            "status": "error",
            "plan_id": claimed.get("plan_id") if claimed else None,
            "error": f"{type(exc).__name__}: {exc}",
            "retained": True,
        }


__all__ = [
    "BACKOFF_MAX_SECONDS",
    "DEADLINE_SECONDS",
    "MAX_ATTEMPTS",
    "ReplayError",
    "backoff_seconds",
    "replay_one",
]
