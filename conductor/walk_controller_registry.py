"""Process registry and atomic launch guard for the shared backlog walk.

The durable ``walk_state`` lock is the final authority.  This module closes the
smaller pre-spawn race where two dashboard requests could both observe "not
running" and launch controllers before either model had time to claim that
lock.
"""
from __future__ import annotations

import fcntl
import os
import re
from contextlib import contextmanager
from pathlib import Path

LAUNCH_LOCK = Path("/tmp/teikoku-walk-controller-launch.lock")

CONTROLLER_PATTERNS = {
    "opus-orchestrator": re.compile(r"opus_walk/opus_(?:walk_chain|supervisor)\.py"),
    "glm-orchestrator": re.compile(r"glm_walk/glm_(?:walk_chain|supervisor)\.py"),
    # Match driver OR supervisor so a fault-backoff sleep (driver alive, supervisor
    # gone) still counts as running — otherwise fallback_walk_start races a second orch.
    "fallback-orchestrator": re.compile(
        r"(?:fallback_walk_driver\.sh|fallback_walk/fallback_supervisor\.py)"
    ),
    # LIVE-U2: elai walk is a drain family (exclusive with opus/glm/fallback).
    # Continuous soak is NOT exclusive — it is heartbeat/organism, not product drain.
    "elai-walk": re.compile(r"elai_walk_driver\.sh"),
}


def _process_args(pid: int) -> list[str]:
    try:
        raw = Path(f"/proc/{pid}/cmdline").read_bytes()
    except OSError:
        return []
    return [
        part.decode("utf-8", errors="replace")
        for part in raw.split(b"\0")
        if part
    ]


def controller_pids(provider: str) -> list[int]:
    pattern = CONTROLLER_PATTERNS.get(provider)
    if pattern is None:
        raise ValueError(f"unknown walk controller: {provider}")
    own_pid = os.getpid()
    found: list[int] = []
    for entry in Path("/proc").iterdir():
        if not entry.name.isdigit():
            continue
        pid = int(entry.name)
        if pid == own_pid:
            continue
        # Match direct argv path elements only.  Searching the flattened
        # command line makes ``bash -lc 'rg ... glm_supervisor.py ...'`` look
        # like every controller at once, and a model prompt can mention these
        # paths too. Script paths never contain whitespace.
        args = _process_args(pid)
        if any(
            not any(char.isspace() for char in arg) and pattern.search(arg)
            for arg in args[1:]
        ):
            found.append(pid)
    return sorted(found)


def active_controllers() -> dict[str, list[int]]:
    return {
        provider: pids
        for provider in CONTROLLER_PATTERNS
        if (pids := controller_pids(provider))
    }


def conflicts(requested_provider: str) -> dict[str, list[int]]:
    return {
        provider: pids
        for provider, pids in active_controllers().items()
        if provider != requested_provider
    }


def refuse_second_family(requested_provider: str) -> dict:
    """LIVE-U2 single-boss: typed refuse when another drain family is already live.

    Returns {"ok": True} when start is allowed, else
    {"ok": False, "reason": "second_family_refused", "conflicts": {...}}.
    """
    if requested_provider not in CONTROLLER_PATTERNS:
        return {
            "ok": False,
            "reason": "unknown_provider",
            "requested": requested_provider,
        }
    c = conflicts(requested_provider)
    if not c:
        return {"ok": True, "requested": requested_provider, "conflicts": {}}
    return {
        "ok": False,
        "reason": "second_family_refused",
        "requested": requested_provider,
        "conflicts": {k: list(v) for k, v in c.items()},
    }


@contextmanager
def launch_guard():
    """Serialize the process-check + spawn boundary across all controllers."""
    fd = os.open(LAUNCH_LOCK, os.O_CREAT | os.O_RDWR, 0o600)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX)
        yield
    finally:
        fcntl.flock(fd, fcntl.LOCK_UN)
        os.close(fd)
