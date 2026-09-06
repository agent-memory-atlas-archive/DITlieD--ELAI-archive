"""Start, stop, and inspect the ordered hosted-controller fallback walk."""
from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path

import walk_controller_registry as registry
from fallback_walk import dispatch_policy
from opus_walk import opus_dispatch

CONDUCTOR_DIR = Path(__file__).resolve().parent
DRIVER = CONDUCTOR_DIR / "fallback_walk_driver.sh"
STOP_SENTINEL = CONDUCTOR_DIR / "FALLBACK_WALK_STOP"
OPUS_STOP = CONDUCTOR_DIR / "OPUS_WALK_STOP"
GLM_STOP = CONDUCTOR_DIR / "GLM_WALK_STOP"
WALK_LOG = "/tmp/fallback-walk.log"
RUNTIME_STATE = CONDUCTOR_DIR / "state" / "fallback-walk-state.json"
SHARED_WALK_STATE = CONDUCTOR_DIR / "opus_walk" / "walk-state.json"
PROVIDER_ID = "fallback-orchestrator"


def _runtime_state() -> dict:
    try:
        data = json.loads(RUNTIME_STATE.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}
    return data if isinstance(data, dict) else {}


def _walk_budget() -> dict:
    try:
        data = json.loads(SHARED_WALK_STATE.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}
    budget = data.get("budget")
    return budget if isinstance(budget, dict) else {}


def fallback_walk_status() -> dict:
    pids = registry.controller_pids(PROVIDER_ID)
    runtime = _runtime_state()
    active = runtime.get("active_provider")
    identities = {
        "sol-fast": {
            "provider": "sol-fast-orchestrator",
            "model": "cline-glm-53-flash",
            "effort": "max",
            "context_window": 1_048_576,
            "context_verified": True,
            "dispatch_cap": 60,
            "plan_cap": 1,
        },
    }
    return {
        "ok": True,
        "running": bool(pids),
        "pids": pids,
        "stop_pending": STOP_SENTINEL.is_file(),
        "provider": PROVIDER_ID,
        "priority": ["sol-fast-orchestrator"],
        "active": identities.get(active),
        "active_provider_key": active,
        "last_provider_key": runtime.get("last_provider"),
        "cooldowns": runtime.get("cooldowns", {}),
        "budget": _walk_budget(),
        "walk_state": str(SHARED_WALK_STATE),
        "note": (
            "Cline free GLM-5.3 Flash boss and workers at max, 1M context, vision. "
            "Concurrency is 6 and degrades to 2 on the first 429/RATE_LIMITED wave."
        ),
    }


def fallback_walk_start(
    *,
    policy_artifact_path: str | Path | None = None,
    credential_path: str | Path | None = None,
) -> dict:
    # Free Devin bridge + PostHog-credit workers (never treat Grok 4.5 as free).
    supported = list(getattr(opus_dispatch, "FREE_BRIDGE_MODELS", opus_dispatch.BRIDGE_MODELS))
    supported += list(getattr(opus_dispatch, "POSTHOG_WORKER_MODELS", ()))
    preflight = dispatch_policy.preflight_fallback_start(
        artifact_path=policy_artifact_path,
        credential_path=credential_path,
        supported_models=supported,
    )
    if not preflight["ok"]:
        return {
            "ok": False,
            "running": False,
            "provider": PROVIDER_ID,
            "error": f"fallback start preflight failed: {preflight['reason']}",
            "reason_code": preflight["reason_code"],
            "policy_artifact": preflight["artifact_path"],
        }
    with registry.launch_guard():
        own = registry.controller_pids(PROVIDER_ID)
        if own:
            return {
                "ok": True,
                "running": True,
                "already": True,
                "provider": PROVIDER_ID,
                "pids": own,
                "message": "fallback walk already running",
            }
        conflicts = registry.conflicts(PROVIDER_ID)
        if conflicts:
            return {
                "ok": False,
                "running": False,
                "provider": PROVIDER_ID,
                "conflicts": conflicts,
                "error": "refusing fallback start: another controller is active",
            }
        if not DRIVER.is_file():
            return {
                "ok": False,
                "running": False,
                "provider": PROVIDER_ID,
                "error": f"driver not found: {DRIVER}",
            }
        for sentinel in (STOP_SENTINEL, OPUS_STOP, GLM_STOP):
            try:
                sentinel.unlink()
            except OSError:
                pass
        try:
            with open(WALK_LOG, "ab") as log:
                subprocess.Popen(
                    ["bash", str(DRIVER)],
                    cwd=str(CONDUCTOR_DIR),
                    stdout=log,
                    stderr=log,
                    stdin=subprocess.DEVNULL,
                    start_new_session=True,
                    env=dict(os.environ),
                )
        except OSError as exc:
            return {
                "ok": False,
                "running": False,
                "provider": PROVIDER_ID,
                "error": f"{type(exc).__name__}: {exc}",
            }
    return {
        "ok": True,
        "running": True,
        "provider": PROVIDER_ID,
        "policy_artifact": preflight["artifact_path"],
        "message": "ordered fallback walk started",
    }


def fallback_walk_stop() -> dict:
    try:
        STOP_SENTINEL.write_text("stop requested via dashboard\n", encoding="utf-8")
    except OSError as exc:
        return {"ok": False, "error": f"{type(exc).__name__}: {exc}"}
    active = _runtime_state().get("active_provider")
    # Existing Opus stops at the end of its turn; ZCode observes its own
    # sentinel and cancels promptly. Cursor/Codex observe FALLBACK_WALK_STOP.
    target = OPUS_STOP if active == "opus" else GLM_STOP if active == "glm" else None
    if target is not None:
        try:
            target.write_text("stop requested via fallback controller\n", encoding="utf-8")
        except OSError:
            pass
    pids = registry.controller_pids(PROVIDER_ID)
    return {
        "ok": True,
        "running": bool(pids),
        "pids": pids,
        "stop_pending": True,
        "provider": PROVIDER_ID,
        "active_provider_key": active,
        "message": "fallback stop requested; active controller is exiting safely",
    }
