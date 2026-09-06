#!/usr/bin/env python3
"""Spawn one headless macaron-v1-venti controller turn (Novita, via the retry shim).

Owner ruling 2026-07-25: macaron-v1-venti is the orchestrator. It is a Mixture-of-LoRA
model on top of GLM-5.2 with a 1,048,576-token window, free on Novita, so it replaces the
quota-drained GLM-5.2 ZCode lane in that seat.

Shape mirrors opus_supervisor._spawn_opus_turn (the proven base-cc spawn): the same
free_api_worker command builder, the same TEIKOKU_AUTONOMOUS gate-drop a headless
--print turn needs, the same prompt-over-stdin. Only the ENV differs, which is exactly
the split worker_providers documents -- a base-cc backend is chosen by which
ANTHROPIC_BASE_URL and auth it points at, nothing else.

THE RPM SAFEGUARD LIVES BELOW THIS FILE. worker_providers.novita_env points base-cc at
127.0.0.1:8813 (novita_retry_proxy.py), which absorbs Novita's 30-rpm 429s and re-issues
after the window rolls. This adapter therefore never has to classify a rate limit, and a
throttled turn cannot end the walk. Verified 2026-07-25: 45 requests fired at 10-way
concurrency through the shim returned 45x 200 with 10 budget rejections absorbed.
"""
from __future__ import annotations

import os
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
CONDUCTOR = HERE.parent
TEIKOKU_ROOT = CONDUCTOR.parent
sys.path.insert(0, str(CONDUCTOR))

import free_api_worker  # noqa: E402
import worker_providers  # noqa: E402

MODEL = worker_providers.NOVITA_MODEL
DEFAULT_TIMEOUT_S = 3 * 60 * 60
PROXY_HEALTH_TIMEOUT_S = 10.0
MCP_CONFIG = CONDUCTOR / "opus_walk" / "opus-mcp.json"


def proxy_healthy(port: int = worker_providers.NOVITA_PROXY_PORT) -> bool:
    """True when the retry shim answers on loopback.

    A dead shim is not a soft condition: base-cc would get connection-refused on
    every call and the turn would fail in a way that looks like a model fault. The
    supervisor checks this BEFORE spending a turn, so a missing shim reports itself
    instead of being misread (wiring-enforcement R2: a dead evidence path screams).
    """
    import socket

    try:
        with socket.create_connection(("127.0.0.1", port), timeout=PROXY_HEALTH_TIMEOUT_S):
            return True
    except OSError:
        return False


def run_macaron_turn(
    prompt: str,
    session_id: str,
    *,
    dispatch_cap: int,
    plan_cap: int,
    timeout_s: float = DEFAULT_TIMEOUT_S,
    extra_env: dict[str, str] | None = None,
) -> dict:
    """Run one fresh macaron controller turn; returns the supervisor result dict."""
    if not proxy_healthy():
        return {
            "rc": 78, "timed_out": False, "cancelled": False, "stdout_tail": "",
            "stderr_tail": (
                f"novita retry shim is not listening on 127.0.0.1:"
                f"{worker_providers.NOVITA_PROXY_PORT}; refusing to spend a turn "
                f"(start it with: NOVITA_API_KEY=... python3 conductor/novita_retry_proxy.py)"
            ),
        }

    with tempfile.NamedTemporaryFile("w", suffix=".prompt.txt", delete=False,
                                     encoding="utf-8") as handle:
        handle.write(prompt)
        prompt_file = Path(handle.name)
    try:
        command, input_text = free_api_worker.build_free_api_command(MODEL, prompt_file)
        if MCP_CONFIG.is_file():
            command = list(command) + ["--mcp-config", str(MCP_CONFIG)]

        env = worker_providers.novita_env(dict(os.environ))
        # A headless --print turn cannot satisfy the interactive meta-cognitive gates
        # (scope_ack / read_full / project_memory), which would DENY its Read/Bash and
        # leave the turn doing nothing. This is the same drop the conductor's own free
        # workers carry; the safety gates that ignore it stay on.
        env["TEIKOKU_AUTONOMOUS"] = "1"
        env["CLAUDE_CODE_SESSION_ID"] = session_id
        env["ELAI_WALK_DISPATCH_CAP"] = str(dispatch_cap)
        env["ELAI_WALK_PLAN_CAP"] = str(plan_cap)
        env["ELAI_WALK_CONTROLLER_PID"] = str(os.getpid())
        if extra_env:
            env.update(extra_env)

        try:
            proc = subprocess.run(
                command,
                input=input_text,
                cwd=str(TEIKOKU_ROOT),  # loads the merged ~/teikoku/.claude harness
                env=env,
                capture_output=True,
                text=True,
                timeout=timeout_s,
            )
            return {
                "rc": proc.returncode,
                "timed_out": False,
                "cancelled": False,
                "stdout_tail": (proc.stdout or "")[-2000:],
                "stderr_tail": (proc.stderr or "")[-2000:],
                "model": MODEL,
            }
        except subprocess.TimeoutExpired as exc:
            return {
                "rc": None, "timed_out": True, "cancelled": False,
                "stdout_tail": (exc.stdout or "")[-2000:] if isinstance(exc.stdout, str) else "",
                "stderr_tail": f"TIMEOUT after {timeout_s}s",
            }
        except OSError as exc:
            return {
                "rc": None, "timed_out": False, "cancelled": False, "stdout_tail": "",
                "stderr_tail": f"spawn failed: {type(exc).__name__}: {exc}",
            }
    finally:
        try:
            prompt_file.unlink()
        except OSError:
            pass
