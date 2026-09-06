#!/usr/bin/env python3
"""Official Grok Build transport for the Sol-fast/max ELAI boss.

The boss uses the project GROK_HOME and keeps the private leader socket.
"""
from __future__ import annotations

import os
import signal
import subprocess
import sys
import time
import uuid
from pathlib import Path

HERE = Path(__file__).resolve().parent
CONDUCTOR = HERE.parent
sys.path.insert(0, str(CONDUCTOR))

import grok_provider  # noqa: E402
import provider_usage  # noqa: E402  (Z-V2: record grok per-turn token usage into the ledger)
from conductor_db import ControlDB  # noqa: E402  (the W8 ledger handle)

MODEL = "cline-glm-53-flash"
DEFAULT_TIMEOUT_S = 3 * 60 * 60
ELAI = CONDUCTOR.parent / "elai"
GROK_LEDGER_LANE = "grok"  # the provider_usage lane key (matches worker_telemetry.lane_for_slot shape)


def _record_usage(db_path: str | Path | None, stdout: str) -> None:
    """Z-V2: parse this grok turn's token/cost from its streaming-JSON end event and accumulate
    it into the per-lane provider_usage ledger (the W8 ledger GET /usage -> provider_costs reads).
    FAIL-SOFT, mirrors devin_model_assist._record_usage: a missing db, an unparseable stream, or
    a degraded write is swallowed -- telemetry must never fail a dispatch that produced a real
    result. Records nothing on a spawn-only / non-model turn (parse_worker_usage returns None,
    matching base-cc's parse_cc_json_usage semantics)."""
    if not db_path:
        return
    try:
        usage = grok_provider.parse_worker_usage(stdout or "")
    except Exception:  # noqa: BLE001 - parse is best-effort; never fail the turn
        return
    if not usage:
        return
    try:
        cost_usd = usage.get("cost_usd")
        cost_cents = int(round((float(cost_usd) if isinstance(cost_usd, (int, float)) else 0.0) * 100))
        provider_usage.record_turn(
            ControlDB(db_path), GROK_LEDGER_LANE,
            tokens_in=int(usage.get("tokens_in", 0) or 0),
            tokens_out=int(usage.get("tokens_out", 0) or 0),
            cache_read=int(usage.get("cache_read", 0) or 0),
            cost_cents=cost_cents,
        )
    except (OSError, ValueError, TypeError):
        # Accounting is observable when available, but cannot turn a real grok result into a
        # failed turn when the local ledger is temporarily unavailable.
        pass


def run_official_grok_turn(
    prompt: str,
    session_id: str,
    *,
    dispatch_cap: int,
    plan_cap: int,
    timeout_s: float = DEFAULT_TIMEOUT_S,
    extra_env: dict[str, str] | None = None,
    cwd: Path | None = None,
    db_path: str | Path | None = None,
    wire_model: str | None = None,
    effort: str = grok_provider.GROK_WORKER_EFFORT,
) -> dict:
    """Run one fresh controller turn on the Grok CLI; supervisor result dict.

    The controller route is strict Sol-fast/max. The private leader socket and
    project GROK_HOME remain unchanged by the caller's authentication setup.
    """
    try:
        binary = grok_provider._resolve_binary()  # noqa: SLF001
    except FileNotFoundError as exc:
        return {
            "rc": 78,
            "timed_out": False,
            "cancelled": False,
            "stdout_tail": "",
            "stderr_tail": str(exc),
        }

    # CLI --session-id must be a UUID; keep walk session_id in env for lock/claim.
    cli_session = str(uuid.uuid4())
    work_cwd = str(cwd or ELAI)

    os.environ["TEIKOKU_FLEET_GROK_HOME"] = str(grok_provider.PROJECT_FLEET_GROK_HOME)
    env = grok_provider.build_grok_env(os.environ, fleet=True)
    env["HOME"] = os.environ.get("HOME", str(Path.home()))
    env["PATH"] = os.environ.get("PATH", env.get("PATH", ""))
    # Private leader socket: concurrent nest sessions on the default leader cancel
    # headless orch turns (stopReason=Cancelled, num_turns=1, zero tools).
    fleet_home = Path(env.get("GROK_HOME") or grok_provider.PROJECT_FLEET_GROK_HOME)
    fleet_home.mkdir(parents=True, exist_ok=True)
    leader_sock = str(fleet_home / "fallback-walk-leader.sock")
    env["GROK_LEADER_SOCKET"] = leader_sock
    env["TEIKOKU_AUTONOMOUS"] = "1"
    env["ELAI_WALK_DISPATCH_CAP"] = str(dispatch_cap)
    env["ELAI_WALK_PLAN_CAP"] = str(plan_cap)
    env["ELAI_WALK_CONTROLLER_PID"] = str(os.getpid())
    env["CLAUDE_CODE_SESSION_ID"] = session_id
    env.setdefault("ELAI_WALK_WORKER_MODEL", grok_provider.GROK_WORKER_MODEL)
    env.setdefault("ELAI_WALK_WORKER_LANE", "posthog")
    env.setdefault("OPUS_PRELAND_VERIFIER_MODEL", grok_provider.GROK_WORKER_MODEL)
    if extra_env:
        env.update(extra_env)

    # -p is one user prompt for multi-agent orchestration. bypassPermissions +
    # always-approve matches the live free-lane headless pattern; private leader
    # socket must also be on argv (env alone is not enough for nest2).
    command = [
        binary,
        "--leader-socket",
        leader_sock,
        "-p",
        prompt,
        "--model",
        wire_model or MODEL,
        "--reasoning-effort",
        effort,
        "--output-format",
        "streaming-json",
        # LIVE-U0: workspace sandbox denied conductor/ writes (claim_lock mutex
        # PermissionError). "disabled" is a custom profile that still bwraps
        # with --unshare-net, so grok cannot reach host zenov :9020.
        # Built-in "off" skips bwrap; that is the actual disable.
        "--sandbox",
        "off",
        "--permission-mode",
        "bypassPermissions",
        "--always-approve",
        "--cwd",
        work_cwd,
        "-s",
        cli_session,
    ]
    started = time.monotonic()
    try:
        proc = subprocess.Popen(
            command,
            cwd=work_cwd,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            start_new_session=True,
        )
    except OSError as exc:
        return {
            "rc": 78,
            "timed_out": False,
            "cancelled": False,
            "stdout_tail": "",
            "stderr_tail": f"{type(exc).__name__}: {exc}",
        }

    try:
        stdout, stderr = proc.communicate(timeout=timeout_s)
        rc = int(proc.returncode if proc.returncode is not None else 1)
        timed_out = False
    except subprocess.TimeoutExpired:
        try:
            os.killpg(proc.pid, signal.SIGTERM)
        except (ProcessLookupError, PermissionError):
            pass
        try:
            stdout, stderr = proc.communicate(timeout=15)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(proc.pid, signal.SIGKILL)
            except (ProcessLookupError, PermissionError):
                pass
            stdout, stderr = "", "timeout kill"
        rc = -1
        timed_out = True

    elapsed = time.monotonic() - started
    notes = ""
    try:
        notes = grok_provider.parse_worker_notes(stdout or "", stderr or "") or ""
    except Exception:  # noqa: BLE001
        notes = (stdout or "")[-4000:]
    # Z-V2: record this turn's token/cost into the per-lane ledger. The grok CLI's streaming-JSON
    # end event already carries input/output/cache_read tokens + cost; parse_worker_usage extracts
    # them. Fail-soft (a bad stream or dead db records nothing, never fails the turn).
    _record_usage(db_path, stdout or "")

    return {
        "rc": rc,
        "timed_out": timed_out,
        "cancelled": False,
        "stdout_tail": (stdout or "")[-12_000:],
        "stderr_tail": (stderr or "")[-12_000:],
        "notes_tail": (notes or "")[-4000:],
        "duration_s": round(elapsed, 1),
        "model": wire_model or MODEL,
        "effort": effort,
        "provider": "sol-fast-orchestrator",
        "session_id": session_id,
        "cli_session_id": cli_session,
        "backend": grok_provider.GROK_LANE_SENTINEL,
    }
def run_preland_judge_turn(
    plan_spec_text,
    diff_text: str,
    gates_json: str,
    *,
    model: str = MODEL,
    popen_factory=None,
    timeout_s: float = 600,
) -> dict:
    import json
    import shutil
    import tempfile

    schema = {
        "type": "object",
        "additionalProperties": False,
        "required": ["verdict", "grounds"],
        "properties": {
            "verdict": {"type": "string", "enum": ["APPROVE", "REJECT"]},
            "grounds": {"type": "string", "minLength": 1},
        },
    }
    definition = {
        "name": "u2-preland-judge",
        "description": "U2 direct non-leader pre-land judge",
        "promptMode": "full",
        "toolConfig": {"tools": []},
        "injectDefaultTools": False,
        "agentsMd": False,
        "discoverSkills": False,
        "inheritSkills": False,
        "mcpInheritance": "none",
        "permissionMode": "dontAsk",
        "tools": [],
    }
    prompt = (
        "=== PLAN SPEC ===\n" + (str(plan_spec_text).strip() if plan_spec_text else "(spec:absent)")
        + "\n\n=== FINAL DIFF ===\n" + (diff_text or "(empty diff)").strip()
        + "\n\n=== GATE OUTPUTS ===\n" + (gates_json or "(empty gates)").strip()
    )
    rubric = (
        "You are an independent pre-land judge. Judge only the plan spec, final diff, and "
        "deterministic gate evidence in the user prompt. Output exactly one JSON object with "
        "exactly these two keys: verdict and grounds. verdict must be APPROVE or REJECT. "
        "grounds must be a non-empty string. Do not output approved, summary, issues, or any "
        "other key."
    )
    with tempfile.TemporaryDirectory(prefix="preland-judge-") as cwd_name:
        cwd = Path(cwd_name)
        with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False, encoding="utf-8") as agent_file:
            agent_path = Path(agent_file.name)
            agent_file.write("---\n")
            for key, value in definition.items():
                if key == "toolConfig":
                    agent_file.write("toolConfig:\n  tools: []\n")
                elif isinstance(value, bool):
                    agent_file.write(f"{key}: {str(value).lower()}\n")
                else:
                    agent_file.write(f"{key}: {value}\n")
            agent_file.write("---\n")
        binary = "/home/researcher/.local/bin/grok"
        if not Path(binary).is_file():
            binary = shutil.which("grok") or "grok"
        command = [
            binary,
            "--agent", str(agent_path),
            "-p", prompt,
            "--model", model,
            "--no-memory",
            "--no-subagents",
            "--disable-web-search",
            "--sandbox", "off",
            "--cwd", str(cwd),
            "--json-schema", json.dumps(schema, separators=(",", ":")),
            "--system-prompt-override", rubric,
        ]
        env = grok_provider.build_grok_env(os.environ, fleet=True)
        env.pop("__GROK_INSIDE_BWRAP", None)
        if os.environ.get("__GROK_INSIDE_BWRAP") == "1":
            # Grok uses this marker to reuse the verified outer Bubblewrap.
            # Nested namespaces are blocked there.
            env["__GROK_INSIDE_BWRAP"] = "1"
        env.pop("GROK_LEADER_SOCKET", None)
        env["PATH"] = os.environ.get("PATH", env.get("PATH", ""))
        popen = popen_factory or subprocess.Popen
        started = time.monotonic()
        try:
            proc = popen(
                command,
                cwd=str(cwd),
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            stdout, stderr = proc.communicate(timeout=timeout_s)
            rc = int(proc.returncode if proc.returncode is not None else 1)
        except subprocess.TimeoutExpired:
            return {"ok": False, "rc": None, "text": "", "error": "judge timeout", "argv": command,
                    "agent_definition": definition, "duration_s": round(time.monotonic() - started, 1)}
        except OSError as exc:
            return {"ok": False, "rc": None, "text": "", "error": f"{type(exc).__name__}: {exc}",
                    "argv": command, "agent_definition": definition}
        finally:
            try:
                agent_path.unlink()
            except OSError:
                pass
    raw = (stdout or "").strip()
    envelope = None
    decoder = json.JSONDecoder()
    for index, character in enumerate(raw):
        if character != "{":
            continue
        try:
            candidate, end = decoder.raw_decode(raw[index:])
        except json.JSONDecodeError:
            continue
        if isinstance(candidate, dict):
            envelope = candidate
            if raw[index + end:].strip() == "":
                break
    if rc != 0:
        return {"ok": False, "rc": rc, "text": raw, "error": (stderr or raw)[-1000:],
                "argv": command, "agent_definition": definition}
    structured = envelope.get("structuredOutput") if isinstance(envelope, dict) else None
    if not isinstance(structured, dict):
        nested = envelope.get("text") if isinstance(envelope, dict) else None
        if isinstance(nested, str):
            try:
                nested_value = json.loads(nested)
            except json.JSONDecodeError:
                nested_value = None
            if isinstance(nested_value, dict):
                if nested_value.get("approved") is False:
                    structured = {
                        "verdict": "REJECT",
                        "grounds": str(nested_value.get("summary") or "explicit reject")[:200],
                    }
                elif set(nested_value) == {"verdict", "grounds"}:
                    structured = nested_value
    valid = (
        isinstance(structured, dict)
        and set(structured) == {"verdict", "grounds"}
        and structured.get("verdict") in {"APPROVE", "REJECT"}
        and isinstance(structured.get("grounds"), str)
        and bool(structured["grounds"].strip())
    )
    if not valid:
        return {"ok": False, "rc": rc, "text": raw, "error": "schema-invalid verdict",
                "argv": command, "agent_definition": definition}
    return {"ok": True, "rc": rc, "text": json.dumps(structured, separators=(",", ":")),
            "structured_output": structured, "raw_output": raw, "error": None,
            "argv": command, "agent_definition": definition,
            "wire_model": model, "duration_s": round(time.monotonic() - started, 1)}
