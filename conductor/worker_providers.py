#!/usr/bin/env python3
"""TEIKOKU P4-B worker-plane unify (TEIKOKU-P4-REVISION-2026-06-20 D1/D2).

The unified base-cc worker backends. A worker_slot runs ONE base-cc CLI; the "pool" is
just which ANTHROPIC_BASE_URL (and auth) the spawned turn points at. Three base-cc backends
share build_free_api_command (the command shape is identical -- only the env differs):

  claude     -> OAuth, NO base_url override (the high-trust plan lane).
  devin-free -> the devin BRIDGE (devin_proxy.py) on the Windows host :8810. The bridge
                presents Anthropic /v1/messages and routes to free Devin models
                (kimi-k2-7 default / glm-5-2 / swe-1-6 / kimi-k2-6, no credit cost). It
                IGNORES the incoming bearer (it uses its own Devin token from the host's
                credentials.toml), so any non-empty ANTHROPIC_AUTH_TOKEN works.
  free-api   -> the litellm proxy :4000 (handled by free_api_worker, kept there).

Why a SEPARATE module from worker_adapter.py: worker_adapter.py is at 994 lines; adding the
backend env logic inline would cross the 1000-line maintainability boundary (MR-1 BLOCK).
worker_adapter dispatches into this module for the base-cc backends (precedent: free_api_worker,
gate_packs).

NETWORKING (the devin-free path): this machine's WSL is DEFAULT-NAT, so the distro does NOT
reach a Windows-host :8810 on localhost. It must reach WIN_HOST:8810 where WIN_HOST is the
default-route gateway IP, discovered AT RUNTIME (it is NAT-assigned and changes per WSL
restart). resolve_win_host() reads it from the routing table. The bridge must bind 0.0.0.0
on the Windows side (devin_proxy.py __main__ already does); the launch script enforces it.

Stdlib only (no new dependency).
"""
from __future__ import annotations

import os
import socket
import subprocess
from pathlib import Path

import free_api_worker

# Harness names that route to a base-cc turn through the devin bridge (the devin-free pool).
DEVIN_FREE_HARNESSES = ("devin-free", "devin-bridge")
# Harness names that route to a base-cc turn on the plan-OAuth lane (no base_url override).
CLAUDE_HARNESSES = ("claude", "claude-oauth")

# The devin bridge port on the Windows host (devin_proxy.py default; launch script binds 0.0.0.0).
DEFAULT_BRIDGE_PORT = 8810
# A non-empty dummy bearer for the devin-free lane: the bridge ignores it (it uses its own
# Devin token), but base-cc must send SOME auth or it falls back to the plan-OAuth path. Value
# is irrelevant to the bridge; NEVER a real key (a global key would mis-route, the 0.3 hazard).
DEVIN_FREE_DUMMY_TOKEN = "teikoku-devin-bridge"


# ---------------------------------------------------------------------------
# TEIKOKU P-BE2 (interactive-orchestration ENGINE): the difficulty/role -> POOL routing rule.
# Owner decision: the orchestrator + auditor + a genuinely HARD executor leaf run on the PAID
# high-trust lane (claude); every trivial / leaf / context-compiler / free-able sub-task runs
# FREE (devin-free preferred, free-api the fallback). The rule is DETERMINISTIC + simple (no LLM
# in the decision) so it is auditable -- it returns a pool id (model_pool.POOLS), which the caller
# resolves to a slot via model_pool.resolve_slot. The FREE preference order is data here, so a
# caller can fall through devin-free -> free-api when a pool is unavailable.
# ---------------------------------------------------------------------------
# Pool ids (kept as literals to avoid importing model_pool, which imports THIS module -- a cycle).
ROUTE_POOL_CLAUDE = "claude"
ROUTE_POOL_DEVIN_FREE = "devin-free"
ROUTE_POOL_FREE_API = "free-api"

# The roles that route to the PAID lane (the high-trust work the owner pays for). An EXECUTE
# leaf is FREE by default; only a HARD executor (difficulty="hard") escalates to PAID.
PAID_ROLES = ("ORCHESTRATE", "AUDIT")
# The FREE pool preference order (devin-free first, free-api fallback) -- owner decision.
FREE_POOL_PREFERENCE = (ROUTE_POOL_DEVIN_FREE, ROUTE_POOL_FREE_API)
# The difficulty tokens a caller may pass (free-text-tolerant; only "hard" escalates EXECUTE).
DIFFICULTY_HARD = "hard"
DIFFICULTY_TRIVIAL = "trivial"


def route_pool_for(role: str, difficulty: str | None = None) -> str:
    """Pick the POOL a sub-task should run on from its role + difficulty (P-BE2 routing rule).

    PAID (claude): the orchestrator (role ORCHESTRATE), the auditor (role AUDIT), or a HARD
    executor leaf (role EXECUTE + difficulty "hard"). FREE (devin-free, the first of
    FREE_POOL_PREFERENCE): everything else -- trivial leaves, context-compiler, ordinary
    executors. Deterministic, no LLM. Returns a pool id; the caller resolves it via
    model_pool.resolve_slot and may fall through FREE_POOL_PREFERENCE on pool unavailability."""
    r = (role or "").strip().upper()
    d = (difficulty or "").strip().lower()
    if r in PAID_ROLES:
        return ROUTE_POOL_CLAUDE
    if r in ("EXECUTE", "EXECUTOR") and d == DIFFICULTY_HARD:
        return ROUTE_POOL_CLAUDE
    return FREE_POOL_PREFERENCE[0]


def is_devin_free_harness(harness: str) -> bool:
    """True when a slot's harness routes a base-cc turn through the devin bridge."""
    return harness in DEVIN_FREE_HARNESSES


def is_claude_harness(harness: str) -> bool:
    """True when a slot's harness routes a base-cc turn on the plan-OAuth lane."""
    return harness in CLAUDE_HARNESSES


def is_base_cc_backend(harness: str) -> bool:
    """True for ANY base-cc backend (claude / devin-free / free-api). worker_adapter uses
    build_free_api_command for the command for all of these; only the env differs by backend."""
    return (
        is_claude_harness(harness)
        or is_devin_free_harness(harness)
        or free_api_worker.is_free_api_harness(harness)
    )


def _bridge_reachable(host: str, port: int) -> bool:
    """Return whether the bridge accepts a short TCP connection."""
    try:
        with socket.create_connection((host, port), timeout=0.25):
            return True
    except OSError:
        return False


def resolve_win_host() -> str | None:
    """Return the reachable devin bridge host.

    An explicit TEIKOKU_WIN_HOST override wins without discovery or probing. Otherwise use the
    default-route gateway when its bridge is reachable, then localhost when reachable. Return
    None when neither endpoint is reachable so callers retain the safe localhost fallback."""
    override = os.environ.get("TEIKOKU_WIN_HOST")
    if override:
        return override
    gateway = None
    try:
        result = subprocess.run(
            ["ip", "route", "show", "default"],
            capture_output=True,
            text=True,
            timeout=5,
        )
    except (OSError, subprocess.SubprocessError):
        result = None
    if result is not None and result.returncode == 0:
        # "default via 172.x.x.1 dev eth0 ..." -> the token after "via".
        for line in result.stdout.splitlines():
            parts = line.split()
            if "via" in parts:
                idx = parts.index("via")
                if idx + 1 < len(parts):
                    gateway = parts[idx + 1]
                    break
    if gateway and _bridge_reachable(gateway, DEFAULT_BRIDGE_PORT):
        return gateway
    if _bridge_reachable("127.0.0.1", DEFAULT_BRIDGE_PORT):
        return "127.0.0.1"
    return None


def bridge_base_url(port: int = DEFAULT_BRIDGE_PORT, win_host: str | None = None) -> str:
    """The ANTHROPIC_BASE_URL a devin-free base-cc turn points at: the bridge on the Windows
    host. Falls back to 127.0.0.1 when no reachable bridge is discovered (mirrored-net distro)."""
    host = win_host if win_host is not None else (resolve_win_host() or "127.0.0.1")
    return f"http://{host}:{port}"


def claude_env(base_env: dict[str, str]) -> dict[str, str]:
    """Env for the claude (plan-OAuth) base-cc backend: NO base_url override. We POP any
    ANTHROPIC_BASE_URL / ANTHROPIC_AUTH_TOKEN / ANTHROPIC_API_KEY that leaked in so the turn
    uses the plan-OAuth credentials base-cc resolves itself (the 0.3 degenerate case)."""
    env = dict(base_env)
    for key in ("ANTHROPIC_BASE_URL", "ANTHROPIC_AUTH_TOKEN", "ANTHROPIC_API_KEY"):
        env.pop(key, None)
    return env


def devin_free_env(
    base_env: dict[str, str],
    *,
    bridge_url: str | None = None,
    port: int = DEFAULT_BRIDGE_PORT,
    win_host: str | None = None,
) -> dict[str, str]:
    """Env for the devin-free base-cc backend: point ANTHROPIC_BASE_URL at the bridge + send a
    non-empty dummy bearer (the bridge ignores it but base-cc needs one) + POP ANTHROPIC_API_KEY.

    DEVIN_TOOL_DESC / DEVIN_SYSTEM_FILE / DEVIN_PROXY_MODEL are consumed by the PROXY process
    (which runs on the Windows host), NOT by base-cc, so they are set on the proxy launch env,
    never here. base-cc only needs the base_url + a bearer."""
    env = dict(base_env)
    env["ANTHROPIC_BASE_URL"] = bridge_url or bridge_base_url(port=port, win_host=win_host)
    env["ANTHROPIC_AUTH_TOKEN"] = DEVIN_FREE_DUMMY_TOKEN
    env["TEIKOKU_WORKER_BARE"] = "1"
    env.pop("ANTHROPIC_API_KEY", None)
    return env


# TEIKOKU-IR TR-1: base-cc / provider context-window-overflow signal tokens. When a prompt exceeds
# the model's window the provider rejects the turn with one of these; worker_adapter maps the hit
# to the "context_window_blocked" error_category, and scheduler.record_worker_result reads that as
# the overflow signal that force-escalates a FREE lane onto a bigger window (the runtime backstop
# for a wrong size estimate). Lives here (not worker_adapter) to keep that file under the 1000-line
# MR-1 boundary -- the same extraction precedent as guard_blocked_result / pre_guard_block.
_CONTEXT_OVERFLOW_TOKENS = (
    "context_length_exceeded",
    "context length exceeded",
    "maximum context length",
    "prompt is too long",
    "input is too long",
    "too many tokens",
    "exceeds the maximum",
    "reduce the length",
    "context window",
    "context_window_exceeded",
    "max_tokens_to_sample",
)


def classify_context_overflow(output: str) -> bool:
    """True when the worker output carries a provider context-window-overflow signal (TR-1). This
    is orthogonal to a transient network error: the prompt was rejected for SIZE, so a retry on the
    SAME small window will just fail again -- the fix is a bigger window, which the scheduler drives
    via the FREE->bigger escalation. Substring match (unsound but safe: a false positive escalates
    to a bigger window, which is fail-closed-safe, never a lost turn)."""
    lower = (output or "").lower()
    return any(token in lower for token in _CONTEXT_OVERFLOW_TOKENS)


def context_cap_env(env: dict, model: str, pool: str | None = None) -> dict:
    """TEIKOKU P4-D (D-09 reconciliation): stamp the REAL per-model context window into base-cc's
    env so base-cc's OWN auto-compact gate (getContextWindowForModel, now reading the ungated
    CLAUDE_CODE_MAX_CONTEXT_TOKENS) uses the right window -- NOT its flat-200k default, which is
    wrong for every non-claude lane. The window is sourced from model_pool.resolve_window (the
    MIN-across-reachable-fallback value when a pool degrades), the SAME source the cross-session
    migration trigger uses, so base-cc's in-session compaction and the conductor's migration fire
    against ONE consistent window. A model with no resolvable window leaves the env untouched
    (base-cc keeps its own default). Mutates a copy; never raises (env-stamp is best-effort)."""
    try:
        import model_pool
        window, _migrate_at = model_pool.resolve_window(model or "", pool or None)
        out = dict(env)
        if window and window > 0:
            out["CLAUDE_CODE_MAX_CONTEXT_TOKENS"] = str(window)
        return out
    except Exception:
        return dict(env)


def guard_blocked_result(
    task,
    worktree_path,
    model: str,
    diff_artifact,
    branch_name: str | None,
    direction: str,
    scan,
    *,
    touched_files=None,
    redact=None,
):
    """Build a BLOCKED TaskResult when the safety guard (guard.py) fires on a worker prompt or
    output. Lives here (not worker_adapter.py) to keep that file under the 1000-line MR-1
    boundary. redact is worker_adapter.redact_text, passed in to avoid a circular import."""
    from models import TaskResult
    reasons = "; ".join(scan.reasons)
    raw = f"Guard blocked the {direction} (score {scan.score}): {reasons}"
    notes = redact(raw) if redact is not None else raw
    if not diff_artifact.exists():
        diff_artifact.write_text("", encoding="utf-8")
    return TaskResult(
        task_id=task.task_id,
        base_commit=task.base_commit,
        worktree_path=str(worktree_path),
        branch_name=branch_name or f"conductor/{task.task_id}",
        model_used=model,
        session_id=None,
        exit_code=1,
        atif_artifact=None,
        diff_artifact=str(diff_artifact),
        touched_files=list(touched_files or []),
        worker_status_hint="BLOCKED",
        error_category="guard_blocked",
        rate_limited=False,
        disconnected_after_mutation=False,
        sanitized_notes=notes[:4000],
        artifact_refs=[str(diff_artifact)] if diff_artifact.exists() else [],
    )


def pre_guard_block(prompt_guard, text, task, worktree_path, model, diff_artifact, branch_name, redact):
    """Run the PromptGuard over the worker prompt. Returns a BLOCKED TaskResult on a hit, else
    None (the caller proceeds to spawn the provider). Extracted from worker_adapter for MR-1."""
    scan = prompt_guard.check(text)
    if scan.blocked:
        return guard_blocked_result(
            task, worktree_path, model, diff_artifact, branch_name, "prompt", scan, redact=redact
        )
    return None


def post_guard_block(
    output_guard, stdout, stderr, task, worktree_path, model, diff_artifact, branch_name,
    touched_files, redact,
):
    """Run the OutputGuard over the worker stdout+stderr. Returns a BLOCKED TaskResult on a hit
    (the diff is already captured by the caller), else None. Extracted for MR-1."""
    scan = output_guard.check((stdout or "") + "\n" + (stderr or ""))
    if scan.blocked:
        return guard_blocked_result(
            task, worktree_path, model, diff_artifact, branch_name, "output", scan,
            touched_files=touched_files, redact=redact,
        )
    return None


# ---------------------------------------------------------------------------
# Novita / macaron orchestrator lane (consumed by fallback_walk/macaron_adapter).
# Restored 2026-08-03 for H-U7B drain resume: macaron_adapter imports these at
# module load; without them the ordered fallback supervisor cannot start.
# ---------------------------------------------------------------------------
NOVITA_MODEL = "mindai/macaron-v1-venti"
NOVITA_PROXY_PORT = 8813


def novita_token() -> str | None:
    """The Novita bearer the retry shim (novita_retry_proxy.py) starts with. Resolution order
    matches every other lane bearer: env NOVITA_API_KEY first, then the gitignored key file
    ~/.teikoku/novita_key. Returns None when neither exists (the shim refuses to start loudly).
    RESTORED 2026-08-07: the provider-routing rework (5cc015ef) dropped this helper while the
    shim still imported it, so the macaron boss rung died at startup with ImportError — found
    live when macaron was the only warm ladder rung."""
    key = (os.environ.get("NOVITA_API_KEY") or "").strip()
    if key:
        return key
    key_file = Path.home() / ".teikoku" / "novita_key"
    try:
        key = key_file.read_text(encoding="utf-8").strip()
    except OSError:
        return None
    return key or None


def novita_env(base: dict | None = None) -> dict:
    """Env for a base-cc turn pointed at the local Novita retry shim (:8813)."""
    env = dict(base or os.environ)
    env["ANTHROPIC_BASE_URL"] = f"http://127.0.0.1:{NOVITA_PROXY_PORT}"
    # Shim injects the real key; child must still send some bearer.
    if not env.get("ANTHROPIC_AUTH_TOKEN"):
        env["ANTHROPIC_AUTH_TOKEN"] = "teikoku-novita-shim"
    env.pop("ANTHROPIC_API_KEY", None)
    return env
