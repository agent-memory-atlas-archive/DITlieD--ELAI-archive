#!/usr/bin/env python3
"""Official Grok Build CLI transport shared by interactive and worker lanes.

Authentication, endpoint selection, credential refresh, and provider protocol details stay
inside the official ``grok`` executable.  Teikoku supplies a bounded one-shot prompt, parses
the documented streaming-JSON envelope, and never opens the CLI's credential store.
"""
from __future__ import annotations

import json
import logging
import os
import re
import shutil
import subprocess
import uuid
from collections.abc import Callable, Mapping, Sequence
from pathlib import Path
from typing import Any

from redaction import redact_text


LOGGER = logging.getLogger("grok_provider")

GROK_BINARY = "grok"
GROK_ROSTER_MODEL = "grok-official-4-5"
GROK_WIRE_MODEL = "grok-4.5"
GROK_WORKER_MODEL = "cline-glm-53-flash"
GROK_WORKER_EFFORT = "max"
PROJECT_FLEET_GROK_HOME = Path("/home/researcher/teikoku/Ougi/grok-build-main/.grok")
GROK_HARNESS = "grok-cli"
GROK_LANE_SENTINEL = "grok-cli://official"
GROK_SUPPORTED_EFFORTS = frozenset({"low", "medium", "high", "max"})
GROK_TIMEOUT_SECONDS = 900
FLEET_GROK_HOME_ENV = "TEIKOKU_FLEET_GROK_HOME"
DEFAULT_FLEET_GROK_HOME_NAME = ".ougi-grok-fleet"
FLEET_AUTO_GC_VALUE = "0"

_SAFE_ENV_KEYS = frozenset({
    "PATH", "PATHEXT", "SYSTEMROOT", "WINDIR", "COMSPEC", "USERPROFILE",
    "HOMEDRIVE", "HOMEPATH", "APPDATA", "LOCALAPPDATA", "TEMP", "TMP",
    "HOME", "LANG", "LC_ALL", "SSL_CERT_FILE", "SSL_CERT_DIR",
    "REQUESTS_CA_BUNDLE", "CURL_CA_BUNDLE", "TEIKOKU_AUTONOMOUS",
    # Cargo/gate offload to Depot (not a secret; values are remote|prefer|local).
    "TEIKOKU_DEPOT_OFFLOAD", "TEIKOKU_DEPOT_OFFLOAD_CONFIG", "DEPOT_ORG",
    "DEPOT_PROJECT_ID", "DEPOT_BIN", "DEPOT_INSTALL_DIR",
})
_SECRET_KEY_MARKERS = ("KEY", "TOKEN", "SECRET", "PASSWORD", "CREDENTIAL", "AUTH")
_QUOTA_MARKERS = (
    "free usage limit", "usage limit reached", "quota exhausted", "quota exceeded",
    "upgrade to continue", "purchase more", "payment required", "paywall",
)
_AUTH_MARKERS = (
    "401", "403", "unauthorized", "forbidden", "not signed in", "login required",
    "authentication required", "invalid credential", "credential expired",
)
_RATE_MARKERS = ("429", "rate limit", "rate-limit", "too many requests", "throttled")


class _GrokProtocolError(RuntimeError):
    pass


class _GrokReportedError(RuntimeError):
    pass


def _wire_model(model: str | None) -> str:
    value = (model or "").strip()
    if not value or value == GROK_ROSTER_MODEL:
        return GROK_WIRE_MODEL
    return value


def _resolve_binary() -> str:
    binary = shutil.which(GROK_BINARY)
    if not binary:
        raise FileNotFoundError("official Grok Build CLI was not found on PATH")
    return binary


def _base_command(
    prompt: str,
    *,
    model: str | None,
    sandbox: str,
    effort: str = "",
) -> list[str]:
    command = [
        _resolve_binary(),
        "-p", prompt,
        "--model", _wire_model(model),
        "--output-format", "streaming-json",
        "--sandbox", sandbox,
        "--permission-mode", "dontAsk",
        "--disable-web-search",
    ]
    normalized_effort = (effort or "").strip().lower()
    if normalized_effort in GROK_SUPPORTED_EFFORTS:
        command.extend(("--reasoning-effort", normalized_effort))
    return command


def build_chat_command(
    prompt: str,
    *,
    model: str = GROK_ROSTER_MODEL,
    effort: str = "",
) -> list[str]:
    command = _base_command(
        prompt,
        model=model,
        sandbox="read-only",
        effort=effort,
    )
    # Chat is deliberately one-shot. Use Grok Build's explicit supported clamp so a user-level
    # default cannot silently enable subagent fan-out for an interactive turn.
    command.append("--no-subagents")
    return command


def build_worker_command(
    prompt: str,
    worktree_path: Path,
    *,
    model: str = GROK_WORKER_MODEL,
    effort: str = GROK_WORKER_EFFORT,
    session_id: str | None = None,
    leader_socket: str | Path | None = None,
) -> tuple[list[str], None]:
    # The subprocess cwd, not a CLI argument, owns the leased-worktree boundary.  Accepting the
    # path here makes that contract explicit to WorkerAdapter and prevents accidental chat reuse.
    if not isinstance(worktree_path, Path):
        worktree_path = Path(worktree_path)
    command = _base_command(
        prompt,
        model=GROK_WORKER_MODEL,
        sandbox="off",
        effort=GROK_WORKER_EFFORT,
    )
    # Headless WORKER turns follow the proven walk pattern (grok_adapter / free-lane headless):
    # bypassPermissions + --always-approve. With "dontAsk" the first tool call that would ask
    # dies stopReason=Cancelled (proven live 2026-08-07 on the posthog lane probe: private
    # leader socket on argv, still Cancelled before any write). Chat keeps dontAsk; write
    # safety for workers is enforced OFF-agent (allowed-writes scope check + frozen gates).
    # sandbox=off is required: workspace still bwraps --unshare-net, so the worker cannot
    # reach host zenov :9020 (H-17 recon D-617..D-621, reqwest "error sending request").
    pm = command.index("--permission-mode")
    command[pm + 1] = "bypassPermissions"
    command.append("--always-approve")
    # Private per-dispatch leader socket ON ARGV (grok_adapter lesson, proven again live
    # 2026-08-07: env GROK_LEADER_SOCKET alone is not enough for nest2 — a worker sharing the
    # default leader dies stopReason=Cancelled after ~1 turn with zero tools).
    if leader_socket:
        command.extend(("--leader-socket", str(leader_socket)))
    minted = (session_id or "").strip() or str(uuid.uuid4())
    command.extend(("-s", minted))
    return command, None


def _resolve_fleet_home(source: Mapping[str, str] | None = None) -> Path:
    values = source if source is not None else os.environ
    override = (values.get(FLEET_GROK_HOME_ENV) or "").strip()
    if override:
        return Path(override).expanduser().resolve()
    return PROJECT_FLEET_GROK_HOME.resolve()


def _ensure_fleet_home(fleet_home: Path) -> Path:
    try:
        fleet_home.mkdir(parents=True, exist_ok=True)
    except OSError as exc:
        raise RuntimeError(
            f"fleet GROK_HOME {fleet_home} cannot be created; refusing human home fallback"
        ) from exc
    return fleet_home


def build_grok_env(
    source: Mapping[str, str] | None = None,
    *,
    fleet: bool = False,
) -> dict[str, str]:
    """Return only execution context needed by the CLI; credential-looking keys stay stripped.

    When fleet=True (worker/fleet spawn), force isolation after the allowlist strip so a
    parent GROK_HOME cannot leak through and the harness never defaults to human ~/.grok.
    Chat/interactive callers keep fleet=False.
    """
    base = source if source is not None else os.environ
    env = {
        str(key): str(value)
        for key, value in base.items()
        if str(key).upper() in _SAFE_ENV_KEYS
        and not any(marker in str(key).upper() for marker in _SECRET_KEY_MARKERS)
    }
    if fleet:
        fleet_home = _ensure_fleet_home(_resolve_fleet_home(base))
        env["GROK_HOME"] = str(fleet_home)
        # ~/.local/bin/grok overwrites GROK_HOME from pwd unless this pin is set.
        # Without it, cwd=/tmp (or any path outside /home/researcher/teikoku) points
        # GROK_HOME at ~/.grok, which has no [profiles.disabled], and the walk
        # dies before it can reach zenov.
        env["GROK_HOME_PIN"] = "1"
        env["GROK_LEADER_SOCKET"] = str(fleet_home / "leader.sock")
        env["GROK_WORKTREE_AUTO_GC"] = FLEET_AUTO_GC_VALUE
        # Heavy cargo/gates must not thrash the owner PC (Depot offload default).
        env.setdefault("TEIKOKU_DEPOT_OFFLOAD", "remote")
        env.setdefault(
            "TEIKOKU_DEPOT_OFFLOAD_CONFIG",
            "/home/researcher/teikoku/conductor/state/depot_offload_config.json",
        )
        env.setdefault("DEPOT_ORG", "example-org")
        # Ensure depot CLI is reachable for gate_runner / depot_offload.py
        depot_bin_dir = str(Path.home() / ".depot" / "bin")
        path_now = env.get("PATH") or ""
        if depot_bin_dir not in path_now.split(os.pathsep):
            env["PATH"] = depot_bin_dir + os.pathsep + path_now
    return env


def _history_text(history: Sequence[Mapping[str, Any]] | None) -> str:
    rows: list[str] = []
    for item in history or ():
        role = str(item.get("role") or "message").strip().upper()
        text = str(item.get("text") or item.get("content") or "").strip()
        if text:
            rows.append(f"[{role}]\n{text}")
    return "\n\n".join(rows)


def _build_seed(
    prompt: str,
    *,
    history: Sequence[Mapping[str, Any]] | None,
    system_note: str | None,
) -> str:
    parts: list[str] = []
    if system_note and system_note.strip():
        parts.append("[SYSTEM]\n" + system_note.strip())
    transcript = _history_text(history)
    if transcript:
        parts.append("[CONVERSATION]\n" + transcript)
    parts.append("[CURRENT USER]\n" + (prompt or "").strip())
    return "\n\n".join(parts)


def _usage_from_end(end: Mapping[str, Any]) -> dict[str, int]:
    raw = end.get("usage")
    if raw is None:
        raw = {}
    if not isinstance(raw, Mapping):
        raise _GrokProtocolError("terminal end usage must be an object")

    def integer(name: str) -> int:
        value = raw.get(name, 0)
        if isinstance(value, bool):
            raise _GrokProtocolError(f"terminal usage {name} must be an integer")
        try:
            parsed = int(value or 0)
        except (TypeError, ValueError) as exc:
            raise _GrokProtocolError(f"terminal usage {name} must be an integer") from exc
        if parsed < 0:
            raise _GrokProtocolError(f"terminal usage {name} cannot be negative")
        return parsed

    try:
        num_turns = int(end.get("num_turns", 0) or 0)
    except (TypeError, ValueError) as exc:
        raise _GrokProtocolError("terminal num_turns must be an integer") from exc
    if num_turns < 0:
        raise _GrokProtocolError("terminal num_turns cannot be negative")
    return {
        "input_tokens": integer("input_tokens"),
        "cache_read_input_tokens": integer("cache_read_input_tokens"),
        "output_tokens": integer("output_tokens"),
        "reasoning_tokens": integer("reasoning_tokens"),
        "total_tokens": integer("total_tokens"),
        "num_turns": num_turns,
    }


def _trusted_cost(end: Mapping[str, Any]) -> float | None:
    if end.get("usage_is_incomplete") or end.get("cost_is_partial"):
        return None
    value = end.get("total_cost_usd")
    if value is None or isinstance(value, bool):
        return None
    try:
        parsed = float(value)
    except (TypeError, ValueError):
        return None
    return parsed if parsed >= 0 else None


def parse_streaming_json(
    stdout: str,
    *,
    on_text_delta: Callable[[str], None] | None = None,
) -> dict[str, Any]:
    """Parse the documented NDJSON stream and require exactly one terminal end event."""
    if not (stdout or "").strip():
        raise _GrokProtocolError("Grok CLI returned an empty response")
    chunks: list[str] = []
    terminal: Mapping[str, Any] | None = None
    for line_number, raw_line in enumerate(stdout.splitlines(), start=1):
        line = raw_line.strip()
        if not line:
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError as exc:
            raise _GrokProtocolError(f"malformed streaming JSON on line {line_number}") from exc
        if not isinstance(event, dict):
            raise _GrokProtocolError(f"stream event on line {line_number} must be an object")
        event_type = event.get("type")
        if event_type == "text":
            if terminal is not None or not isinstance(event.get("data"), str):
                raise _GrokProtocolError("invalid text event ordering or payload")
            chunk = event["data"]
            chunks.append(chunk)
            if on_text_delta is not None:
                on_text_delta(chunk)
        elif event_type == "thought":
            if terminal is not None or not isinstance(event.get("data"), str):
                raise _GrokProtocolError("invalid thought event ordering or payload")
        elif event_type == "error":
            message = event.get("message")
            raise _GrokReportedError(str(message or "Grok CLI reported an unspecified error"))
        elif event_type == "end":
            if terminal is not None:
                raise _GrokProtocolError("stream contained multiple terminal end events")
            terminal = event
        else:
            raise _GrokProtocolError(f"unsupported stream event type: {event_type!r}")
    if terminal is None:
        raise _GrokProtocolError("stream ended without a terminal end event")
    return {
        "text": "".join(chunks),
        "usage": _usage_from_end(terminal),
        "cost_usd": _trusted_cost(terminal),
    }


def _reported_stream_error(stdout: str) -> str | None:
    for raw_line in (stdout or "").splitlines():
        try:
            event = json.loads(raw_line)
        except (json.JSONDecodeError, TypeError):
            continue
        if isinstance(event, dict) and event.get("type") == "error":
            return str(event.get("message") or "")
    return None


def _classify_error(message: str, *, protocol: bool = False) -> tuple[str, int | None, int | None]:
    lower = (message or "").lower()
    status = next((code for code in (401, 403, 429) if str(code) in lower), None)
    retry_match = re.search(r"retry(?: after|_after)?\D{0,8}(\d+)", lower)
    retry_after = int(retry_match.group(1)) if retry_match else None
    if any(marker in lower for marker in _QUOTA_MARKERS):
        return "terminal_quota", status, retry_after
    if any(marker in lower for marker in _AUTH_MARKERS):
        return "terminal_auth", status, retry_after
    if any(marker in lower for marker in _RATE_MARKERS):
        return "transient_rate_limit", status, retry_after
    return ("protocol_failure" if protocol else "transport_failure"), status, retry_after


def _failure(message: str, *, protocol: bool = False) -> dict[str, Any]:
    safe = redact_text(message or "Grok CLI transport failed")
    error_type, status, retry_after = _classify_error(safe, protocol=protocol)
    return {
        "ok": False,
        "text": "",
        "model": GROK_WIRE_MODEL,
        "usage": {},
        "tool_steps": [],
        "cost_usd": None,
        "error": safe,
        "status": status,
        "retry_after": retry_after,
        "error_type": error_type,
    }


def run_grok_turn(
    prompt: str,
    *,
    history: Sequence[Mapping[str, Any]] | None = None,
    system_note: str | None = None,
    on_text_delta: Callable[[str], None] | None = None,
    model: str = GROK_ROSTER_MODEL,
    effort: str = "",
    timeout_seconds: int = GROK_TIMEOUT_SECONDS,
) -> dict[str, Any]:
    seed = _build_seed(prompt, history=history, system_note=system_note)
    try:
        command = build_chat_command(seed, model=model, effort=effort)
        completed = subprocess.run(
            command,
            capture_output=True,
            text=True,
            timeout=max(1, int(timeout_seconds)),
            env=build_grok_env(),
        )
    except subprocess.TimeoutExpired:
        return _failure("Grok CLI timed out")
    except OSError as exc:
        return _failure(str(exc))

    if completed.returncode != 0:
        message = _reported_stream_error(completed.stdout) or completed.stderr or completed.stdout
        return _failure(message or f"Grok CLI exited with status {completed.returncode}")
    try:
        parsed = parse_streaming_json(completed.stdout, on_text_delta=on_text_delta)
    except _GrokReportedError as exc:
        return _failure(str(exc))
    except _GrokProtocolError as exc:
        return _failure(str(exc), protocol=True)
    return {
        "ok": True,
        "text": parsed["text"],
        "model": _wire_model(model),
        "usage": parsed["usage"],
        "tool_steps": [],
        "cost_usd": parsed["cost_usd"],
        "error": None,
        "status": None,
        "retry_after": None,
        "error_type": None,
    }


def parse_worker_notes(stdout: str, stderr: str = "") -> str | None:
    try:
        return redact_text(parse_streaming_json(stdout)["text"])
    except (_GrokProtocolError, _GrokReportedError):
        fallback = _reported_stream_error(stdout) or stderr
        return redact_text(fallback) if fallback else None


def parse_worker_usage(stdout: str) -> dict[str, Any] | None:
    try:
        parsed = parse_streaming_json(stdout)
    except (_GrokProtocolError, _GrokReportedError):
        return None
    usage = parsed["usage"]
    return {
        "tokens_in": usage["input_tokens"],
        "tokens_out": usage["output_tokens"],
        "cache_read": usage["cache_read_input_tokens"],
        "reasoning_tokens": usage["reasoning_tokens"],
        "num_turns": usage["num_turns"],
        "cost_usd": parsed["cost_usd"],
    }
