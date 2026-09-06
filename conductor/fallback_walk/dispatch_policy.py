"""Dispatch policy resolution and fallback-start preflight checks.

Owner-approved ELAI walk uses cline-glm-53-flash (Cline free GLM-5.3 Flash,
1M context, vision) at max effort for ordinary workers. Legacy PostHog names
remain compatibility inputs only.
"""
from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Iterable, Mapping

LIVE_U4 = "LIVE-U4"
ORDINARY_WORKER_MODEL = "cline-glm-53-flash"
ORDINARY_WORKER_EFFORT = "max"
WORKER_CONCURRENCY_CAP = 6
RATE_LIMITED_WORKER_CONCURRENCY_CAP = 2
REQUIRED_MODEL = ORDINARY_WORKER_MODEL
REQUIRED_LANE = "posthog"
DEFAULT_DEVIN_CREDENTIALS_PATH = Path("/home/researcher/.local/share/devin/credentials.toml")
DEFAULT_POLICY_ARTIFACT_PATH = (
    Path(__file__).resolve().parents[1] / "state" / "fallback-dispatch-policy.json"
)
POLICY_SCHEMA_VERSION = 1

# Free Devin bridge (from live free_roster.json)
FREE_BRIDGE_MODELS = ("glm-5-2", "swe-1-7", "swe-1-7-medium", "swe-check")
# PostHog-credit workers (zenov posthog-bridge / grok CLI slugs)
POSTHOG_WORKER_MODELS = (
    ORDINARY_WORKER_MODEL,
    "posthog-gpt56-luna",
    "posthog-gpt56-terra",
    "posthog-gpt56-sol",
)
# Seam-design / escalation-decision authors (owner 2026-08-06): doc-only write sets,
# enforced fail-closed in opus_dispatch.dispatch_posthog_worker. Never EXECUTE workers.
POSTHOG_DESIGN_MODELS = (
    "posthog-fable-5",
    "posthog-opus-5",
)
# Never free
PAID_NOT_FREE = ("grok-4-5-high", "grok-4.5", "grok-official-4-5")

DISPATCH_POLICY: dict[str, dict[str, str]] = {
    LIVE_U4: {
        "lane": REQUIRED_LANE,
        "provider": "posthog",
        "model": REQUIRED_MODEL,
    },
}


def worker_concurrency_cap(rate_limited: bool = False) -> int:
    return RATE_LIMITED_WORKER_CONCURRENCY_CAP if rate_limited else WORKER_CONCURRENCY_CAP


def resolve_dispatch_policy(
    workload: str = LIVE_U4,
    *,
    policy_table: Mapping[str, Mapping[str, str]] | None = None,
) -> dict[str, str | None]:
    table = DISPATCH_POLICY if policy_table is None else policy_table
    row = table.get(workload)
    if not isinstance(row, Mapping):
        row = {}
    resolved = row.get("resolved_model", row.get("model"))
    return {
        "workload": workload,
        "lane": row.get("lane") or REQUIRED_LANE,
        "provider": row.get("provider") or "posthog",
        "required_model": REQUIRED_MODEL,
        "resolved_model": str(resolved) if resolved else None,
    }


def configured_devin_credentials_path(
    path: str | Path | None = None,
    *,
    env: Mapping[str, str] | None = None,
) -> Path:
    if path is not None:
        return Path(path).expanduser()
    environ = os.environ if env is None else env
    return Path(environ.get("DEVIN_CREDS") or DEFAULT_DEVIN_CREDENTIALS_PATH).expanduser()


def check_devin_credentials(
    path: str | Path | None = None,
    *,
    env: Mapping[str, str] | None = None,
) -> dict[str, object]:
    # PostHog path does not require Devin credentials; still report path status for free fallback.
    resolved_path = configured_devin_credentials_path(path, env=env)
    try:
        with resolved_path.open("rb"):
            pass
    except OSError:
        return {
            "ok": False,
            "path": str(resolved_path),
            "reason_code": "credentials-unreadable",
            "reason": "configured Devin credential file is not readable",
            "required_for_lane": "free",
        }
    return {"ok": True, "path": str(resolved_path), "required_for_lane": "free"}


def check_posthog_worker_ready(
    *,
    env: Mapping[str, str] | None = None,
) -> dict[str, object]:
    """PostHog worker readiness: zenov up + pool has remaining credit (best-effort)."""
    import urllib.request

    environ = os.environ if env is None else env
    base = (environ.get("ZENOV_BASE_URL") or "http://127.0.0.1:9020").rstrip("/")
    master = (environ.get("ZENOV_MASTER_KEY") or "").strip()
    if not master:
        # try .env beside zenov
        env_path = Path("/home/researcher/teikoku/Ougi/zenov/.env")
        try:
            for line in env_path.read_text(encoding="utf-8").splitlines():
                if line.startswith("ZENOV_MASTER_KEY="):
                    master = line.split("=", 1)[1].strip()
                    break
        except OSError:
            pass
    health_url = f"{base}/health"
    try:
        with urllib.request.urlopen(health_url, timeout=5) as resp:
            body = resp.read().decode("utf-8", errors="replace")
        if "ok" not in body.lower() and '"status"' not in body:
            return {
                "ok": False,
                "reason_code": "zenov-unhealthy",
                "reason": f"zenov health unexpected: {body[:120]}",
                "base": base,
            }
    except Exception as exc:  # noqa: BLE001
        return {
            "ok": False,
            "reason_code": "zenov-unreachable",
            "reason": f"zenov health failed: {exc}",
            "base": base,
        }
    # credit pool optional soft check
    remaining = None
    try:
        req = urllib.request.Request(
            f"{base}/api/mngr/providers/usage",
            headers={"Authorization": f"Bearer {master}"} if master else {},
        )
        with urllib.request.urlopen(req, timeout=10) as resp:
            doc = json.loads(resp.read().decode("utf-8", errors="replace"))
        usage = (doc.get("posthog-bridge") or {}).get("usage") or {}
        pool = usage.get("accounts") or {}
        remaining = pool.get("aggregate_remaining_credits")
        if remaining is not None and float(remaining) <= 0 and not str(REQUIRED_MODEL).startswith("cline"):
            return {
                "ok": False,
                "reason_code": "posthog-credits-empty",
                "reason": "posthog aggregate remaining credits is 0",
                "remaining": remaining,
            }
    except Exception as exc:  # noqa: BLE001
        # soft: health ok is enough to try Luna; credit board may be auth-gated
        return {
            "ok": True,
            "reason_code": None,
            "reason": f"zenov ok; credit probe soft-fail: {exc}",
            "remaining": remaining,
            "base": base,
        }
    return {
        "ok": True,
        "reason_code": None,
        "reason": None,
        "remaining": remaining,
        "base": base,
    }


def _model_names(models: Iterable[str]) -> list[str]:
    return list(dict.fromkeys(str(model) for model in models if str(model).strip()))


def _atomic_write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.{os.getpid()}.tmp")
    try:
        temporary.write_text(
            json.dumps(payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        os.replace(temporary, path)
    except BaseException:
        try:
            temporary.unlink(missing_ok=True)
        except OSError:
            pass
        raise


def preflight_fallback_dispatch(
    *,
    artifact_path: str | Path | None = None,
    credential_path: str | Path | None = None,
    supported_models: Iterable[str] = (),
    policy_table: Mapping[str, Mapping[str, str]] | None = None,
    workload: str = LIVE_U4,
    env: Mapping[str, str] | None = None,
) -> dict[str, object]:
    policy = resolve_dispatch_policy(workload, policy_table=policy_table)
    resolved_model = policy["resolved_model"]
    lane = str(policy.get("lane") or "")
    models = _model_names(supported_models)
    # Default supported set if caller passes empty: free + posthog (+ design authors)
    if not models:
        models = (
            list(FREE_BRIDGE_MODELS)
            + list(POSTHOG_WORKER_MODELS)
            + list(POSTHOG_DESIGN_MODELS)
        )

    policy_check = {
        "ok": resolved_model == REQUIRED_MODEL and lane == REQUIRED_LANE,
        "required_model": REQUIRED_MODEL,
        "required_lane": REQUIRED_LANE,
        "resolved_model": resolved_model,
        "resolved_lane": lane,
        "reason_code": None,
        "reason": None,
    }
    if not policy_check["ok"]:
        policy_check.update({
            "reason_code": "policy-model-mismatch",
            "reason": (
                f"policy must resolve model={REQUIRED_MODEL} lane={REQUIRED_LANE}, "
                f"got model={resolved_model} lane={lane}"
            ),
        })

    if resolved_model in PAID_NOT_FREE:
        executor_check = {
            "ok": False,
            "resolved_model": resolved_model,
            "supported_models": models,
            "reason_code": "paid-model-as-free",
            "reason": (
                f"{resolved_model} is PAID (Grok 4.5 is not free on Devin or official). "
                f"Use free bridge {FREE_BRIDGE_MODELS} or PostHog {POSTHOG_WORKER_MODELS}."
            ),
        }
    else:
        executor_check = {
            "ok": bool(policy_check["ok"] and resolved_model in models),
            "resolved_model": resolved_model,
            "supported_models": models,
            "reason_code": None,
            "reason": None,
        }
        if not executor_check["ok"]:
            executor_check.update({
                "reason_code": (
                    "policy-model-mismatch" if not policy_check["ok"] else "unsupported-model"
                ),
                "reason": (
                    "dispatch policy did not resolve the required model"
                    if not policy_check["ok"]
                    else f"executor does not support resolved model {resolved_model}"
                ),
            })

    # Credentials: free lane needs Devin creds; posthog lane needs zenov+credits.
    if lane == "posthog" or resolved_model in POSTHOG_WORKER_MODELS:
        credentials_check = check_posthog_worker_ready(env=env)
        # map shape
        credentials_check = {
            "ok": bool(credentials_check.get("ok")),
            "path": credentials_check.get("base"),
            "reason_code": credentials_check.get("reason_code"),
            "reason": credentials_check.get("reason"),
            "remaining": credentials_check.get("remaining"),
            "lane": "posthog",
        }
    else:
        credentials_check = check_devin_credentials(credential_path, env=env)

    checks = {
        "policy_resolves_required_model": policy_check,
        "executor_supports_resolved_model": executor_check,
        "lane_credentials_ready": credentials_check,
    }
    failures = [
        check
        for check in (policy_check, executor_check, credentials_check)
        if not check["ok"]
    ]
    first_failure = failures[0] if failures else None
    artifact = (
        Path(artifact_path).expanduser()
        if artifact_path is not None
        else DEFAULT_POLICY_ARTIFACT_PATH
    )
    result: dict[str, object] = {
        "schema_version": POLICY_SCHEMA_VERSION,
        "workload": workload,
        "policy": policy,
        "checks": checks,
        "ok": not failures,
        "reason_code": first_failure.get("reason_code") if first_failure else None,
        "reason": first_failure.get("reason") if first_failure else None,
        "artifact_path": str(artifact),
        "free_bridge_models": list(FREE_BRIDGE_MODELS),
        "posthog_worker_models": list(POSTHOG_WORKER_MODELS),
        "paid_never_free": list(PAID_NOT_FREE),
    }
    try:
        _atomic_write_json(artifact, result)
    except OSError:
        result["ok"] = False
        result["reason_code"] = "policy-artifact-unwritable"
        result["reason"] = "dispatch policy artifact is not writable"
    return result


preflight_fallback_start = preflight_fallback_dispatch
