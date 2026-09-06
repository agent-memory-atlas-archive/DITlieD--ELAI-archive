#!/usr/bin/env python3
"""opus_preland_verify.py - P1.5 fresh pre-land verifier for the opus walk (orchestrator D-8).

Before opus_land commits a gated changeset, spin up ONE FRESH headless base-cc session on the
plan-OAuth (claude) lane that has NEVER seen the build conversation. It is handed ONLY the plan
spec, the final diff, and the deterministic gate outputs, and must answer APPROVE or REJECT. This
is the loop-01 four-party split: the model that built the change cannot also be the authority that
approves landing it (in-boundary self-grading is the rubber-stamp failure the whole gate stack
exists to prevent).

FAIL-CLOSED (D-8): the gate is ON by default in opus_land (--no-verifier opts out). A REJECT
refuses the land (VERIFIER_REJECTED); a spawn/transport failure OR an unparseable answer refuses
the land too (VERIFIER_UNAVAILABLE) -- an absent verifier is never a silent pass.

The spawn machinery mirrors opus_supervisor._spawn_opus_turn (free_api_worker.build_free_api_command
+ worker_providers.claude_env); the seat model reuses opus_supervisor.OPUS_MODEL (the walk's model
id) unless env OPUS_PRELAND_VERIFIER_MODEL overrides. Stdlib only.
"""
from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path

_HERE = Path(__file__).resolve().parent
CONDUCTOR = _HERE.parent
for _p in (str(CONDUCTOR), str(_HERE)):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import free_api_worker            # noqa: E402  (the base-cc command builder, shared with the supervisor)
import worker_providers           # noqa: E402  (the plan-OAuth lane env)
import opus_supervisor            # noqa: E402  (reuse OPUS_MODEL / OPUS_ACCOUNT / TEIKOKU_ROOT + token sync)

# --- fixed wiring (named constants) ------------------------------------------
# Env override for the verifier seat; default is the walk's own model id (reused, not re-hardcoded).
VERIFIER_MODEL_ENV = "OPUS_PRELAND_VERIFIER_MODEL"
# The verifier runs on the same paid plan-OAuth account the supervisor uses.
VERIFIER_ACCOUNT = opus_supervisor.OPUS_ACCOUNT
# cwd for the spawned turn: the merged ~/teikoku/.claude harness, exactly as the supervisor uses.
VERIFIER_CWD = opus_supervisor.TEIKOKU_ROOT
# Wall ceiling for one verification turn (a spec+diff read-and-judge is short; this is the backstop).
VERIFIER_TIMEOUT_S = 10 * 60
# Cap the diff fed into the prompt so a huge changeset does not blow the verifier's window.
DIFF_MAX_CHARS = 60000
# Retry a TRANSIENT verifier transport failure (empty completion / 401 / 429 / timeout) this many
# times, re-syncing the plan-OAuth token before each attempt, before declaring UNAVAILABLE. One
# transient base-cc blip must not fail-close a land that already passed every deterministic gate.
VERIFIER_MAX_ATTEMPTS = 3
# Linear backoff seconds between verifier retries (attempt-index * this): 0s, 8s, 16s.
VERIFIER_RETRY_BACKOFF_S = 8
# The strict decision instruction (ONE module-level constant, not inline f-string soup).
VERIFIER_INSTRUCTION = (
    "You are a FRESH, independent pre-land verifier. You did NOT build this change and have NOT "
    "seen the build conversation. Judge ONLY the plan spec, the final diff, and the deterministic "
    "gate outputs below; decide whether this change should LAND on main.\n"
    "Answer on the FIRST line with exactly one word: APPROVE or REJECT. On the SECOND line give one "
    "short sentence of grounds. Do not hedge. If the diff does not match the plan's intent, is "
    "incomplete, or the gate evidence is unconvincing, answer REJECT.")


def _default_model() -> str:
    """The verifier seat: env OPUS_PRELAND_VERIFIER_MODEL, else the walk's model id (OPUS_MODEL)."""
    return (os.environ.get(VERIFIER_MODEL_ENV) or "").strip() or opus_supervisor.OPUS_MODEL


def _build_prompt(spec_text, diff_text: str, gates_json: str) -> str:
    """Assemble the verifier's ONLY view: instruction + plan spec (or spec:absent) + final diff
    (truncated) + the deterministic gate outputs. It never sees the build conversation."""
    spec_block = spec_text.strip() if spec_text and spec_text.strip() \
        else "(spec:absent -- judge on the diff + gate outputs only)"
    diff_block = (diff_text or "").strip()
    if len(diff_block) > DIFF_MAX_CHARS:
        diff_block = diff_block[:DIFF_MAX_CHARS] + "\n... [diff truncated]"
    return (f"{VERIFIER_INSTRUCTION}\n\n"
            f"=== PLAN SPEC ===\n{spec_block}\n\n"
            f"=== FINAL DIFF ===\n{diff_block or '(empty diff)'}\n\n"
            f"=== GATE OUTPUTS (deterministic, off-agent) ===\n{gates_json}\n")


def _cc_result_text(stdout: str) -> str:
    """Extract the assistant answer from base-cc's `--print --output-format json` envelope: the
    last JSON line's .result field, with a few fallbacks. Never raises."""
    import json
    for line in reversed([ln for ln in (stdout or "").splitlines() if ln.strip()]):
        try:
            obj = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(obj, dict):
            for key in ("result", "final_message", "message", "text"):
                value = obj.get(key)
                if isinstance(value, str) and value.strip():
                    return value
            return ""
    return ""


def _parse_verdict(text: str):
    """Strict parse: the first standalone approve|reject token, case-insensitive. Unparseable
    (hedge, empty, neither word) returns None -> fail-closed 'unavailable'."""
    import re
    match = re.search(r"\b(approve|reject)\b", text or "", re.IGNORECASE)
    return match.group(1).lower() if match else None


def _grounds_line(text: str) -> str:
    """The one-line grounds: the second non-empty line (after the verdict word), else the first."""
    lines = [ln.strip() for ln in (text or "").splitlines() if ln.strip()]
    if len(lines) >= 2:
        return lines[1][:200]
    return lines[0][:200] if lines else ""


def _sync_token_best_effort() -> None:
    """Keep the plan-OAuth token fresh before the fresh spawn so a stale token does not 401 and
    wrongly REFUSE a good land (fail-closed makes a token 401 pessimistic). Reuses the supervisor's
    sync; best-effort, never raises."""
    try:
        opus_supervisor._sync_elai_token_from_max()
    except Exception as exc:  # noqa: BLE001 - token freshening is best-effort
        print(f"[preland-verify] token sync skipped: {exc}", file=sys.stderr, flush=True)


def default_spawn(prompt: str, model: str) -> dict:
    """Delegate one verifier request to the single retry owner."""
    import json

    verdict, detail = run_verifier_with_retry(None, prompt, "", model=model)
    structured = None
    if verdict in {"approve", "reject"}:
        structured = {"verdict": verdict.upper(), "grounds": detail.get("grounds") or ""}
    return {
        "ok": verdict in {"approve", "reject"},
        "text": json.dumps(structured, separators=(",", ":")) if structured else "",
        "structured_output": structured,
        "rc": detail.get("rc"),
        "error": detail.get("error"),
        "attempts": detail.get("attempts", []),
    }


def fresh_preland_verify(plan_spec_text, diff_text, gates_json, model=None, spawn_fn=None) -> tuple:
    """Return approve, reject, or unavailable from strict structured judge output."""
    import json

    seat = model or (os.environ.get(VERIFIER_MODEL_ENV) or "").strip() or "cline-glm-53-flash"
    detail_base = {
        "model": seat,
        "spec_present": bool(plan_spec_text and plan_spec_text.strip()),
        "verdict_raw": None,
        "grounds": None,
        "rc": None,
        "error": None,
    }
    if spawn_fn is not None:
        prompt = _build_prompt(plan_spec_text, diff_text, gates_json)
        try:
            result = spawn_fn(prompt, seat)
        except Exception as exc:  # noqa: BLE001 - injected transport is fail-closed
            detail_base["error"] = f"spawn_exception: {type(exc).__name__}: {exc}"
            return "unavailable", detail_base
        detail_base["rc"] = result.get("rc")
        detail_base["error"] = result.get("error")
        if not result.get("ok"):
            return "unavailable", detail_base
        structured = result.get("structured_output")
        if not isinstance(structured, dict):
            try:
                parsed = json.loads(result.get("text") or "")
            except (TypeError, ValueError, json.JSONDecodeError):
                parsed = None
            structured = parsed if isinstance(parsed, dict) else None
        valid = (
            isinstance(structured, dict)
            and set(structured) == {"verdict", "grounds"}
            and structured.get("verdict") in {"APPROVE", "REJECT"}
            and isinstance(structured.get("grounds"), str)
            and bool(structured["grounds"].strip())
        )
        if not valid:
            detail_base["verdict_raw"] = str(result.get("text") or "")[:300]
            detail_base["error"] = "schema-invalid verdict"
            return "unavailable", detail_base
        detail_base["verdict_raw"] = str(result.get("text") or structured)[:300]
        detail_base["grounds"] = structured["grounds"][:200]
        return structured["verdict"].lower(), detail_base
    verdict, detail = run_verifier_with_retry(
        plan_spec_text, diff_text, gates_json, model=seat
    )
    detail["spec_present"] = detail_base["spec_present"]
    return verdict, detail
def run_verifier_with_retry(
    plan_spec_text,
    diff_text: str,
    gates_json: str,
    *,
    model: str | None = None,
    popen_factory=None,
    judge_fn=None,
) -> tuple:
    """Own exactly two transport attempts and return a strict verdict."""
    import json

    seat = model or (os.environ.get(VERIFIER_MODEL_ENV) or "").strip() or "cline-glm-53-flash"
    attempts = []
    for attempt in (1, 2):
        try:
            if judge_fn is not None:
                result = judge_fn(plan_spec_text, diff_text, gates_json, model=seat)
            else:
                from fallback_walk import grok_adapter
                result = grok_adapter.run_preland_judge_turn(
                    plan_spec_text,
                    diff_text,
                    gates_json,
                    model=seat,
                    popen_factory=popen_factory,
                )
        except Exception as exc:  # noqa: BLE001 - verifier transport is fail-closed
            result = {
                "ok": False,
                "rc": None,
                "text": "",
                "error": f"judge_exception: {type(exc).__name__}: {exc}",
            }
        attempt_record = {
            "attempt": attempt,
            "ok": bool(result.get("ok")),
            "rc": result.get("rc"),
            "error": result.get("error"),
        }
        attempts.append(attempt_record)
        print(
            f"[preland-verify] judge transport attempt {attempt}/2 "
            f"ok={str(attempt_record['ok']).lower()} rc={attempt_record['rc']}",
            file=sys.stderr,
            flush=True,
        )
        if not result.get("ok"):
            if attempt == 1:
                continue
            return "unavailable", {
                "model": seat,
                "verdict_raw": None,
                "grounds": None,
                "rc": result.get("rc"),
                "error": result.get("error") or "transport failure",
                "attempts": attempts,
                "attempt_count": len(attempts),
                "argv": result.get("argv"),
                "agent_definition": result.get("agent_definition"),
                "raw_output": result.get("raw_output"),
                "wire_model": result.get("wire_model", seat),
            }
        structured = result.get("structured_output")
        if not isinstance(structured, dict):
            try:
                parsed = json.loads(result.get("text") or "")
            except (TypeError, ValueError, json.JSONDecodeError):
                parsed = None
            structured = parsed if isinstance(parsed, dict) else None
        valid = (
            isinstance(structured, dict)
            and set(structured) == {"verdict", "grounds"}
            and structured.get("verdict") in {"APPROVE", "REJECT"}
            and isinstance(structured.get("grounds"), str)
            and bool(structured["grounds"].strip())
        )
        if not valid:
            return "unavailable", {
                "model": seat,
                "verdict_raw": str(result.get("text") or "")[:300],
                "grounds": None,
                "rc": result.get("rc"),
                "error": "schema-invalid verdict",
                "attempts": attempts,
                "attempt_count": len(attempts),
            }
        return structured["verdict"].lower(), {
            "model": seat,
            "verdict_raw": str(result.get("text") or structured)[:300],
            "grounds": structured["grounds"][:200],
            "rc": result.get("rc"),
            "error": result.get("error"),
            "attempts": attempts,
            "attempt_count": len(attempts),
            "argv": result.get("argv"),
            "agent_definition": result.get("agent_definition"),
            "raw_output": result.get("raw_output"),
            "wire_model": result.get("wire_model", seat),
        }
    raise AssertionError("unreachable verifier retry state")  # unparseable answer -> fail-closed
