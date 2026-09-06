"""Official Cursor Agent and official OpenAI Codex controller transports."""
from __future__ import annotations

import json
import os
import signal
import subprocess
import tempfile
import time
from pathlib import Path

TEIKOKU_ROOT = Path("/home/researcher/teikoku")
CURSOR_AGENT = Path("/home/researcher/.local/bin/cursor-agent")
CURSOR_MODEL = "cursor-grok-4.5-high"
CURSOR_MODEL_NAME = "Cursor Grok 4.5 High"
CODEX_BIN = Path("/usr/local/bin/codex")
CODEX_MODEL = "gpt-5.6-sol"
DEFAULT_TIMEOUT_S = 3 * 60 * 60

_STRIP_PREFIXES = ("BMAPI", "ANTHROPIC", "ZCODE")
_STRIP_EXACT = {
    "CURSOR_API_KEY",
    "CURSOR_AUTH_TOKEN",
    "OPENAI_API_KEY",
    "OPENAI_BASE_URL",
    "CODEX_API_KEY",
    "CODEX_ACCESS_TOKEN",
}


def official_auth_env(extra_env: dict[str, str] | None = None) -> dict[str, str]:
    """Return an environment that cannot silently select a proxy/API-key lane."""
    env = {}
    for key, value in os.environ.items():
        if key in _STRIP_EXACT or key.startswith(_STRIP_PREFIXES):
            continue
        env[key] = value
    if extra_env:
        env.update(extra_env)
    return env


def _terminate_group(proc: subprocess.Popen) -> None:
    if proc.poll() is not None:
        return
    try:
        os.killpg(proc.pid, signal.SIGTERM)
        proc.wait(timeout=5)
    except (ProcessLookupError, subprocess.TimeoutExpired):
        if proc.poll() is None:
            try:
                os.killpg(proc.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                pass


def _tail(handle, limit: int = 12_000) -> str:
    handle.flush()
    handle.seek(0, os.SEEK_END)
    size = handle.tell()
    handle.seek(max(0, size - limit))
    return handle.read()


def _head(handle, limit: int = 12_000) -> str:
    handle.flush()
    handle.seek(0)
    return handle.read(limit)


def _run(
    command: list[str],
    *,
    prompt_stdin: str | None,
    env: dict[str, str],
    stop_path: Path,
    timeout_s: float,
) -> dict:
    started = time.monotonic()
    cancelled = False
    timed_out = False
    with tempfile.TemporaryFile(mode="w+t", encoding="utf-8") as stdout_file, \
            tempfile.TemporaryFile(mode="w+t", encoding="utf-8") as stderr_file:
        try:
            proc = subprocess.Popen(
                command,
                stdin=subprocess.PIPE if prompt_stdin is not None else subprocess.DEVNULL,
                stdout=stdout_file,
                stderr=stderr_file,
                text=True,
                cwd=str(TEIKOKU_ROOT),
                env=env,
                start_new_session=True,
            )
        except OSError as exc:
            return {
                "rc": None,
                "timed_out": False,
                "cancelled": False,
                "stdout_tail": "",
                "stderr_tail": f"{type(exc).__name__}: {exc}",
            }
        if prompt_stdin is not None and proc.stdin is not None:
            try:
                proc.stdin.write(prompt_stdin)
                proc.stdin.close()
            except (BrokenPipeError, OSError):
                pass
        while proc.poll() is None:
            if stop_path.is_file():
                cancelled = True
                _terminate_group(proc)
                break
            if time.monotonic() - started >= timeout_s:
                timed_out = True
                _terminate_group(proc)
                break
            time.sleep(0.25)
        return {
            "rc": proc.returncode,
            "timed_out": timed_out,
            "cancelled": cancelled,
            "stdout_head": _head(stdout_file),
            "stdout_tail": _tail(stdout_file),
            "stderr_tail": _tail(stderr_file),
        }


def _preflight(command: list[str], env: dict[str, str], expected: str) -> tuple[bool, str]:
    try:
        check = subprocess.run(
            command,
            cwd=str(TEIKOKU_ROOT),
            env=env,
            capture_output=True,
            text=True,
            timeout=30,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        return False, f"{type(exc).__name__}: {exc}"
    text = f"{check.stdout}\n{check.stderr}".strip()
    return check.returncode == 0 and expected in text, text[-2_000:]


def run_cursor_turn(
    prompt: str,
    *,
    stop_path: Path,
    timeout_s: float = DEFAULT_TIMEOUT_S,
    extra_env: dict[str, str] | None = None,
) -> dict:
    """Run one fresh official Cursor Agent session on Cursor Grok 4.5 High."""
    env = official_auth_env(extra_env)
    if not CURSOR_AGENT.is_file():
        return {
            "rc": None, "timed_out": False, "cancelled": False,
            "stdout_tail": "", "stderr_tail": f"Cursor Agent not found: {CURSOR_AGENT}",
        }
    ok, detail = _preflight([str(CURSOR_AGENT), "status"], env, "Logged in")
    if not ok:
        return {
            "rc": 78, "timed_out": False, "cancelled": False,
            "stdout_tail": "", "stderr_tail": f"Cursor official auth preflight failed: {detail}",
        }
    ok, detail = _preflight([str(CURSOR_AGENT), "models"], env, CURSOR_MODEL)
    if not ok:
        return {
            "rc": 78, "timed_out": False, "cancelled": False,
            "stdout_tail": "", "stderr_tail": f"Cursor model preflight failed: {detail}",
        }
    result = _run(
        [
            str(CURSOR_AGENT),
            "-p",
            "--output-format", "stream-json",
            "--model", CURSOR_MODEL,
            "--workspace", str(TEIKOKU_ROOT),
            "--trust",
            "--force",
            "--sandbox", "disabled",
            "--approve-mcps",
            prompt,
        ],
        prompt_stdin=None,
        env=env,
        stop_path=stop_path,
        timeout_s=timeout_s,
    )
    if result["rc"] == 0:
        # A supported slug is necessary but not sufficient: fail closed if the
        # official stream initializes another model.
        model_seen = False
        for line in result["stdout_head"].splitlines():
            try:
                frame = json.loads(line)
            except ValueError:
                continue
            if frame.get("type") == "system" and frame.get("subtype") == "init":
                reported = str(frame.get("model") or "")
                model_seen = CURSOR_MODEL_NAME.lower() in reported.lower()
        if not model_seen:
            result["rc"] = 78
            result["stderr_tail"] = (
                result["stderr_tail"] + "\nCursor stream did not prove the requested Grok 4.5 model"
            ).strip()
    return result


def run_codex_turn(
    prompt: str,
    *,
    stop_path: Path,
    timeout_s: float = DEFAULT_TIMEOUT_S,
    extra_env: dict[str, str] | None = None,
) -> dict:
    """Run one fresh official Codex session using ChatGPT subscription auth."""
    env = official_auth_env(extra_env)
    if not CODEX_BIN.is_file():
        return {
            "rc": None, "timed_out": False, "cancelled": False,
            "stdout_tail": "", "stderr_tail": f"Codex CLI not found: {CODEX_BIN}",
        }
    ok, detail = _preflight([str(CODEX_BIN), "login", "status"], env, "Logged in using ChatGPT")
    if not ok:
        return {
            "rc": 78, "timed_out": False, "cancelled": False,
            "stdout_tail": "", "stderr_tail": f"official ChatGPT auth preflight failed: {detail}",
        }
    return _run(
        [
            str(CODEX_BIN), "exec",
            "--json",
            "--ephemeral",
            "--ignore-user-config",
            "-C", str(TEIKOKU_ROOT),
            "-m", CODEX_MODEL,
            "-c", 'approval_policy="never"',
            "-s", "danger-full-access",
            "-",
        ],
        prompt_stdin=prompt,
        env=env,
        stop_path=stop_path,
        timeout_s=timeout_s,
    )
