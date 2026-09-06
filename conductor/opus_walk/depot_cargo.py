"""Cargo gate execution under the conductor Depot policy."""
from __future__ import annotations

import os
import re
import subprocess
import sys
from collections.abc import Mapping, Sequence
from dataclasses import replace
from pathlib import Path
from typing import Any

TOOLS_DIR = Path(__file__).resolve().parents[1] / "tools"
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

import depot_offload

DEPOT_OFFLOAD_ENV = "TEIKOKU_DEPOT_OFFLOAD"
VALID_MODES = frozenset({"remote", "prefer", "local"})
_REQUEST_ID_RE = re.compile(r"\b(?:build|run|request)_id=([A-Za-z0-9_.:-]+)")


class DepotCompletedProcess(subprocess.CompletedProcess):
    """CompletedProcess with Depot evidence attached when a remote run occurred."""

    def __init__(
        self,
        args: Sequence[str],
        returncode: int,
        stdout: str = "",
        stderr: str = "",
        *,
        depot_id: str | None = None,
        request_id: str | None = None,
        provider: str | None = None,
        offload_mode: str | None = None,
    ) -> None:
        super().__init__(list(args), returncode, stdout, stderr)
        self.depot_id = depot_id
        self.request_id = request_id
        self.provider = provider
        self.offload_mode = offload_mode


def offload_mode() -> str:
    """Return the effective policy, defaulting to remote for walk execution."""
    raw = (os.environ.get(DEPOT_OFFLOAD_ENV) or "remote").strip().lower()
    return raw if raw in VALID_MODES else "remote"


def _annotate_local(proc: subprocess.CompletedProcess) -> subprocess.CompletedProcess:
    proc.depot_id = None
    proc.request_id = None
    proc.provider = "local"
    proc.offload_mode = "local"
    return proc


def _run_local(
    command: Sequence[str],
    cwd: Path,
    timeout_s: int,
    env: Mapping[str, str] | None,
) -> subprocess.CompletedProcess:
    return _annotate_local(
        subprocess.run(
            list(command),
            cwd=str(cwd),
            env=dict(env) if env is not None else None,
            capture_output=True,
            text=True,
            timeout=timeout_s,
        )
    )


def _remote_id(raw: Mapping[str, Any]) -> str | None:
    for key in ("depot_id", "run_id", "build_id", "request_id"):
        value = raw.get(key)
        if value:
            return str(value)
    match = _REQUEST_ID_RE.search(str(raw.get("output") or ""))
    return match.group(1) if match else None


def _remote_result(
    command: Sequence[str],
    raw: Mapping[str, Any],
    mode: str,
) -> DepotCompletedProcess:
    output = str(raw.get("output") or "")
    stderr = str(raw.get("stderr") or "")
    request_id = _remote_id(raw)
    return DepotCompletedProcess(
        command,
        int(raw.get("exit_code") if raw.get("exit_code") is not None else 1),
        output,
        stderr,
        depot_id=request_id,
        request_id=str(raw.get("request_id")) if raw.get("request_id") else request_id,
        provider=str(raw.get("provider") or "depot"),
        offload_mode=str(raw.get("offload_mode") or mode),
    )


def _fail_closed(command: Sequence[str], mode: str, error: BaseException) -> DepotCompletedProcess:
    message = f"[depot-offload] FAIL-CLOSED: {type(error).__name__}: {error}"
    return DepotCompletedProcess(
        command,
        2,
        message,
        "",
        provider="depot",
        offload_mode=mode,
    )


def run_gate_command(
    cmd: list[str],
    cwd: Path,
    timeout_s: int,
    *,
    env: Mapping[str, str] | None = None,
) -> subprocess.CompletedProcess:
    """Run one gate command locally or through the existing Depot offload module."""
    command = list(cmd)
    mode = offload_mode()
    if mode == "local" or not depot_offload.is_heavy_cargo_command(command):
        return _run_local(command, cwd, timeout_s, env)
    try:
        cfg = replace(depot_offload.OffloadConfig.load(), mode=mode)
        raw = depot_offload.run_offloaded(
            command,
            cwd,
            gate_name="",
            timeout_sec=timeout_s,
            env=env,
            cfg=cfg,
        )
    except depot_offload.DepotOffloadError as exc:
        if mode == "prefer":
            return _run_local(command, cwd, timeout_s, env)
        return _fail_closed(command, mode, exc)
    if not isinstance(raw, Mapping):
        if mode == "prefer":
            return _run_local(command, cwd, timeout_s, env)
        return _fail_closed(command, mode, ValueError("Depot returned a non-mapping result"))
    return _remote_result(command, raw, mode)
