#!/usr/bin/env python3
"""Strict Sol-fast controller for the shared ELAI backlog walk.

The stopped shared harness uses one fresh Sol-fast/max boss per plan. Quota and
rate-limit failures cool the same provider. They never select another boss.
"""
from __future__ import annotations

import argparse
import json
import os
import sys
import time
from contextlib import contextmanager
from datetime import datetime, timedelta, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
CONDUCTOR = HERE.parent
TEIKOKU_ROOT = CONDUCTOR.parent
SHARED_WALK = CONDUCTOR / "opus_walk"
sys.path.insert(0, str(CONDUCTOR))
sys.path.insert(0, str(SHARED_WALK))
sys.path.insert(0, str(HERE))

import events  # noqa: E402
import walk_state  # noqa: E402
from fallback_walk import dispatch_policy  # noqa: E402
from fallback_walk import grok_adapter  # noqa: E402
from fallback_walk import macaron_adapter  # noqa: E402
from fallback_walk import provider_adapters  # noqa: E402
from glm_walk import glm_supervisor  # noqa: E402
from opus_walk import opus_supervisor  # noqa: E402

STOP_SENTINEL = CONDUCTOR / "FALLBACK_WALK_STOP"
OPUS_STOP = CONDUCTOR / "OPUS_WALK_STOP"
GLM_STOP = CONDUCTOR / "GLM_WALK_STOP"
WALK_STATE = SHARED_WALK / "walk-state.json"
RUNTIME_STATE = CONDUCTOR / "state" / "fallback-walk-state.json"
DB_PATH = CONDUCTOR / "state" / "control.db"
SKILL_PATH = HERE / "skills" / "sequential-fallback-orchestrator" / "SKILL.md"
OWNER_DIRECTIVE_PATH = CONDUCTOR / "OWNER_DIRECTIVE.md"
CANVAS_PLAN_ID = "teikoku-opus-walk"
NO_WORK_RECHECK_S = 15 * 60
WAIT_TICK_S = 5
INTER_TURN_SLEEP_S = 4
SMALL_DISPATCH_CAP = 10
SMALL_PLAN_CAP = 1

# Owner-approved ELAI power mode: one fresh Sol-fast/max boss, one plan, sixty
# dispatches, and ordinary Luna-fast/max workers. The worker doctrine degrades
# concurrency from six to two on the first observed 429/RATE_LIMITED wave.
BOSS_MODEL = "cline-glm-53-flash"
BOSS_EFFORT = "max"
WORKER_MODEL = "cline-glm-53-flash"
WORKER_EFFORT = "max"
MAX_DISPATCHES = 60
MAX_PLANS = 1
MAX_WORKER_CONCURRENCY = 6
RATE_LIMITED_WORKER_CONCURRENCY = 2
PROVIDERS = (
    {
        "id": "sol-fast",
        "provider": "sol-fast-orchestrator",
        "model": BOSS_MODEL,
        "effort": BOSS_EFFORT,
        "context_window": 1_048_576,
        "context_verified": True,
        "dispatch_cap": MAX_DISPATCHES,
        "plan_cap": MAX_PLANS,
    },
)
PROVIDER_BY_ID = {provider["id"]: provider for provider in PROVIDERS}

_QUOTA_PATTERNS = (
    "you've hit your weekly limit",
    "weekly usage limit",
    "usage limit reached",
    "rate_limit_exceeded",
    "resource_exhausted",
    "insufficient_balance",
    "quota exceeded",
    "quota limit",
    '"code":1308',
    "too many requests",
    # cursor-agent's real exhaustion string, 2026-07-22: it emits
    #   "ActionRequiredError: Increase limits for faster responses You're out of
    #    usage. Switch to Auto..."
    # which matched NOTHING above, so a plain quota hit was classified
    # "unexpected" and fail-stopped the whole ladder instead of falling through
    # to GLM. Both spellings: the apostrophe is a UTF-8 right-single-quote in
    # some emissions and an ASCII ' in others.
    "you're out of usage",
    "you’re out of usage",
    # official Grok Build's real exhaustion string, 2026-08-07 (live 402 during the night
    # drain): '"message": "API error (status 402 Payment Required): Grok Build usage balance
    # exhausted"'. Matched NOTHING above, so the paid-boss quota hit classified "unexpected",
    # the supervisor halted instead of falling through the ladder, and the driver burned its
    # 3 relaunches on 2-4s failures. Same bug class as the cursor-agent case above.
    "payment required",
    "usage balance exhausted",
    "(status 402",
    "out of usage",
    # Novita macaron's real exhaustion string, 2026-08-07 12:13Z (same night as the 402
    # above, next rung down): claude-cli stdout carries
    #   "Failed to authenticate. API Error: 403 not enough balance"
    # with api_error_status 403 — an AUTH status used for a BILLING condition. It matched
    # nothing here, classified "unexpected", halted the ladder, and burned driver
    # relaunch 1/3. Third instance of this bug class; grep the provider's real emission
    # into this table the day a rung is added, not the night it dies.
    "not enough balance",
    "not enough credit",
    "insufficient balance",
    "insufficient credit",
    "balance is insufficient",
    # official Grok Build's OTHER exhaustion string, 2026-08-08 13:23Z (first turn after a
    # PC restart): '"API error (status 403 Forbidden): personal-team-blocked:spending-limit:
    # You have run out of credits or need a Grok subscription."' — a 403 wearing a billing
    # message, so neither "(status 402" nor the 402-era strings matched. FOURTH instance of
    # the unmatched-exhaustion class; it halted the ladder and burned driver relaunches on
    # 6s failures again. The api rotates between 402 and 403 forms for the same empty
    # wallet, so both wordings live here.
    "spending-limit",
    "run out of credits",
    "personal-team-blocked",
    # Novita emits these two. Classifying them as "quota" (fall through / short cooldown)
    # rather than "unexpected" (hard-halt the whole ladder) is the SECOND layer of the RPM
    # safeguard: the retry shim should absorb every 429 below the controller, but if one
    # ever leaks into a turn's stderr, throttling must never be what stops the walk.
    # server_overload is capacity noise that retries away, so it is emphatically not a halt.
    "server_overload",
    "server overload",
    # H-U3A free-lane probe (2026-08-03 full ramp + reclassify): observed on the
    # devin bridge under concurrency drain. Landed so classify_failure maps them
    # to quota (fall-through / cooldown) instead of unexpected (hard-halt).
    "permission_denied",
    "reset in",
    "rate limit",
    "rate-limit",
    "ratelimit",
    "429",
    "http 429",
    "throttl",
    "capacity",
    "overloaded",
    "try again later",
    "temporarily unavailable",
    "message quota",
    "free tier",
    "usage cap",
    "requests per minute",
    "rpm limit",
    "rps limit",
    "content_filter",
    "content filter",
    "blocked this request",
    "http 500",
    "internal server error",
    "devin-proxy gateway error",
    "gateway error",
    "an internal error occurred",
    "message rate limit",
    "overall message rate limit",
    "limit will reset",
    "for your free plan",
    "http 500: internal server error",
    # Zenov/posthog gateway shapes (BURN-U2). Bare credit_exhausted / virtual-key
    # quota denials and pool-empty messages matched NOTHING above historically and
    # would classify unexpected (hard-halt) instead of short-cooldown fall-through.
    # Sources: zenov free-lane admission, model_scrub public errors, posthog_accounts
    # is_creditish_deny, routes_chat virtual-key admit, otel_export classify_error.
    "credit_exhausted",
    "credit remaining",
    "virtual-key quota denied",
    "virtual_key_quota_denied",
    "quota_exhausted",
    "no_ready_key",
    "all keys in pool are cooled",
    "posthog-credits-empty",
    "posthog aggregate remaining credits",
    "v2 virtual-key quota denied",
    "lane demoted",
    "credit floor",
    # Grok talking to zenov (Cline walk 2026-08-26): four identical ~1344s deaths
    # of `reqwest error stream: error sending request for url (http://127.0.0.1:9020/...)`.
    # That is a dropped stream, not a Cline 429. Unmatched, it halted the ladder.
    # Treat as retryable so the walk continues while zenov rotates siblings.
    "reqwest error stream",
    "error sending request",
)
_AUTH_PATTERNS = (
    "not logged in",
    "authentication failed",
    "authentication required",
    "unauthorized",
    "invalid access token",
    "invalid api key",
    "coding plan key",
    "login required",
    "http 401",
    "http 403",
    # Zenov/posthog gateway auth shapes (BURN-U2).
    "missing or invalid bearer",
    "invalid bearer",
    "auth_failed",
    "bearer required",
    "zenov auth",
    "posthog oauth",
    "oauth token",
    "token refresh failed",
)


def _utc_now() -> datetime:
    return datetime.now(timezone.utc)


def _iso(value: datetime) -> str:
    return value.isoformat()


def _atomic_write(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temp = path.with_name(f"{path.name}.{os.getpid()}.tmp")
    temp.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    os.replace(temp, path)


def _initial_state(now: datetime | None = None) -> dict:
    now = now or _utc_now()
    # Owner-reported reset estimates for the first activation only.  They are
    # persisted, so a restart does not extend the cooldown.
    return {
        "schema_version": 1,
        "active_provider": None,
        "last_provider": None,
        "cooldowns": {
            "sol-fast": {
                "until": _iso(now),
                "reason": "owner-approved Sol-fast/max power mode",
            },
        },
        "history": [],
        "updated_at": _iso(now),
    }


def load_runtime_state(path: Path = RUNTIME_STATE) -> dict:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        data = _initial_state()
        _atomic_write(path, data)
    except (OSError, ValueError) as exc:
        raise RuntimeError(f"fallback runtime state is unreadable: {exc}") from exc
    if not isinstance(data, dict) or data.get("schema_version") != 1:
        raise RuntimeError("fallback runtime state has an unsupported schema")
    data.setdefault("cooldowns", {})
    data.setdefault("history", [])
    return data


def _save_runtime_state(state: dict, path: Path = RUNTIME_STATE) -> None:
    state["updated_at"] = _iso(_utc_now())
    state["history"] = list(state.get("history") or [])[-100:]
    _atomic_write(path, state)


def _record(state: dict, event: str, **detail) -> None:
    state.setdefault("history", []).append(
        {"ts": _iso(_utc_now()), "event": event, **detail}
    )


def _cooldown_until(state: dict, provider_id: str) -> datetime | None:
    row = (state.get("cooldowns") or {}).get(provider_id)
    if not isinstance(row, dict):
        return None
    try:
        return datetime.fromisoformat(str(row["until"]))
    except (KeyError, TypeError, ValueError):
        return None


def select_provider(state: dict, now: datetime | None = None) -> dict | None:
    """Return the only ready boss, or wait while its short cooldown is active."""
    now = now or _utc_now()
    provider = PROVIDERS[0]
    until = _cooldown_until(state, provider["id"])
    return None if until is not None and until > now else provider


def _next_cooldown_delay(state: dict, now: datetime | None = None) -> int:
    now = now or _utc_now()
    values = [
        until for provider in PROVIDERS
        if (until := _cooldown_until(state, provider["id"])) is not None and until > now
    ]
    if not values:
        return NO_WORK_RECHECK_S
    return max(1, int((min(values) - now).total_seconds()))


def classify_failure(result) -> str:
    """Classify a turn failure.

    Accepts the real adapter result mapping. Also accepts a synthetic bare string
    (BURN-U2 falsifier / diagnostic probes). Non-mapping inputs are normalized to
    an error-shaped mapping; unmatched text stays fail-closed as ``unexpected``.
    """
    if not isinstance(result, dict):
        text = str(result if result is not None else "")
        result = {
            "rc": 1,
            "error": text,
            "stderr_tail": text,
            "stdout_tail": "",
            "notes_tail": "",
        }
    if result.get("cancelled"):
        return "cancelled"
    if result.get("timed_out"):
        return "unexpected"
    # rc=78 is the adapters' deliberate PROVIDER-UNAVAILABLE refusal (macaron: novita shim not
    # listening; grok: binary missing). Live 2026-08-07: it classified "unexpected" and halted
    # the whole ladder at rung 2 instead of falling through to GLM. Unavailable = cool down and
    # try the next rung, exactly like quota.
    if result.get("rc") == 78:
        return "unavailable"
    text = "\n".join(
        str(result.get(key) or "") for key in ("error", "stderr_tail", "stdout_tail", "notes_tail")
    ).lower()
    compact = text.replace(" ", "")
    if '"stopreason":"cancelled"' in compact:
        return "unexpected"
    if result.get("rc") == 0:
        return "ok"
    if any(pattern in text for pattern in _QUOTA_PATTERNS):
        return "quota"
    if any(pattern in text for pattern in _AUTH_PATTERNS):
        return "auth"
    return "unexpected"


# Money-empty (vs rate-window) exhaustion strings: these clear on owner top-up, never by
# waiting out a window, so the cooldown floor is hours even on short-cooldown providers.
_BALANCE_PATTERNS = (
    "not enough balance",
    "not enough credit",
    "insufficient balance",
    "insufficient credit",
    "balance is insufficient",
    "insufficient_balance",
    "payment required",
    "usage balance exhausted",
    "(status 402",
    # grok's 403-form of the same empty wallet (2026-08-08 13:23Z): clears on top-up only.
    "spending-limit",
    "run out of credits",
    "personal-team-blocked",
)


def _is_balance_exhaustion(result) -> bool:
    if not isinstance(result, dict):
        text = str(result if result is not None else "").lower()
    else:
        text = " ".join(
            str(result.get(key) or "") for key in ("error", "stderr_tail", "stdout_tail", "notes_tail")
        ).lower()
    return any(pattern in text for pattern in _BALANCE_PATTERNS)


def _cooldown_hours(provider_id: str, failure_class: str) -> float:
    if failure_class == "auth" and provider_id != "sol-fast":
        return 1.0
    return {
        "opus": 6.0,
        "glm": 2.0,
        "cursor-grok": 1.0,
        "chatgpt": 1.0,
        # official Grok Build balance exhaustion (402) clears only when the owner tops up,
        # so probe rarely; each re-probe is a ~3s failed CLI call. Missing key found live
        # 2026-08-07: _cooldown_hours("official-grok", "quota") raised KeyError, so even a
        # correctly classified quota hit would have crashed the supervisor.
        "official-grok": 6.0,
        # macaron's only rate limit is Novita's 30 rpm, which clears in SIXTY SECONDS.
        # The retry shim should absorb it before the controller ever sees it, so a quota
        # classification here means one leaked past. Cooling for hours over a one-minute
        # window would idle the walk for nothing, so this is deliberately ~2 minutes:
        # long enough for the window to roll, short enough that the walk just continues.
        "macaron": 0.034,
        # Sol-fast/PostHog pool quota and rate windows are short. Other provider ids
        # are retained only for historical runtime-state compatibility.
        "sol-fast": 0.1,
    }.get(provider_id, 2.0)


# Short cooldown for the Zenov/PostHog pool. Balance-shaped errors keep this
# short because the pool can recover by key rotation.
UNLIMITED_SHORT_COOLDOWN_IDS = frozenset({"sol-fast"})


def cooldown_hours_for_failure(provider_id: str, failure_class: str, result) -> float:
    """Return the short Sol-fast cooldown used after quota or auth failures.

    The single-boss route never falls through to another boss model.
    """
    hours = _cooldown_hours(provider_id, failure_class)
    if failure_class == "quota" and _is_balance_exhaustion(result):
        if provider_id not in UNLIMITED_SHORT_COOLDOWN_IDS:
            # RPM-style quota clears in minutes (macaron cools 0.034h for its 60s
            # window), but an empty BALANCE on a paid/wallet rung clears only when
            # the owner tops up. Live 2026-08-07: macaron `403 not enough balance`
            # on a 2-minute cooldown would have re-probed a dead paid rung ~30x/hour.
            hours = max(hours, 6.0)
    return hours


def _emit(event_type: str, payload: dict) -> None:
    try:
        events.append_event(
            str(DB_PATH),
            event_type,
            task_id=CANVAS_PLAN_ID,
            payload=payload,
            project_id="elai",
            session_id=CANVAS_PLAN_ID,
        )
    except Exception as exc:  # noqa: BLE001 - telemetry is fail-soft
        print(f"[fallback-supervisor] WARN event emit failed: {exc}", file=sys.stderr, flush=True)


def _owner_directive() -> str:
    try:
        return OWNER_DIRECTIVE_PATH.read_text(encoding="utf-8")[:60_000].strip()
    except OSError:
        return ""


def _boot_prompt(provider: dict, session_id: str) -> str:
    unverified = (
        f"The {provider['context_window']:,}-token window is the provider's declared "
        f"figure and is NOT numerically verified end to end. "
        if not provider["context_verified"]
        else ""
    )
    directive = _owner_directive()
    directive_block = f"\n\nOWNER DIRECTIVE:\n{directive}" if directive else ""
    power_override = (
        "\n\nLATEST OWNER POWER-MODE OVERRIDE (binding; supersedes conflicting older lane text above):\n"
        f"- Exactly one fresh {BOSS_MODEL} boss at reasoning effort {BOSS_EFFORT} owns one plan.\n"
        f"- Every boss-dispatched worker shape executes {WORKER_MODEL} at effort {WORKER_EFFORT}.\n"
        f"- Run up to {MAX_WORKER_CONCURRENCY} disjoint workers; after the first 429/RATE_LIMITED "
        f"wave, cap the rest of that plan at {RATE_LIMITED_WORKER_CONCURRENCY}.\n"
        "- Terra remains the independent pre-land verifier. Gates and SerializedIntegrator remain unchanged."
    )
    return (
        f"You are the {provider['provider']} controller using model {provider['model']}. "
        f"Session id: {session_id}. This is one fresh provider session in the ordered "
        f"fallback chain.\n\n"
        f"FIRST read {SKILL_PATH} in full and follow it as binding. It points to the "
        f"canonical sequential backlog-walk skill, which you must also read in full. "
        f"Shared state: {WALK_STATE}. Canonical handoff: "
        f"/workspace/ELAI/workflow/handoff/opus-backlog-walk.md.\n\n"
        f"Claim the shared lock as {session_id}. LockRefused means STOP immediately. "
        f"Continue the exact cursor plan only. This turn has a HARD budget of "
        f"{provider['dispatch_cap']} worker dispatches and exactly {provider['plan_cap']} plan. "
        f"{unverified}Do not begin a second plan. At a clean unit boundary perform the "
        f"canonical stop protocol, release the lock, and exit. Power-mode contract:\n"
        f"  - Boss: {BOSS_MODEL} at explicit reasoning effort {BOSS_EFFORT}. "
        f"This is the only boss route. Never silently select another boss model.\n"
        f"  - All boss-dispatched worker shapes, including legacy free, posthog, prime-rlm, "
        f"design, and codex-shaped calls, execute {WORKER_MODEL} at effort {WORKER_EFFORT}. "
        f"Design calls keep doc-only write safety. Codex-shaped calls keep write-set safety.\n"
        f"  - Dispatch up to {MAX_WORKER_CONCURRENCY} concurrent disjoint workers. "
        f"On the first observed 429 or RATE_LIMITED wave, degrade concurrency to "
        f"{RATE_LIMITED_WORKER_CONCURRENCY} for the rest of the plan.\n"
        f"  - The independent Terra pre-land verifier remains unchanged and independent. "
        f"Never route it through opus_dispatch.py.\n"
        f"Depot remote offload, isolated worktrees, fresh verification, and "
        f"SerializedIntegrator remain unchanged.\n"
        f"Keep one-plan law. Never author product code yourself. Never use BMAPI, "
        f"codex-vm pools, cursor-grok bridge, ChatGPT bridges, or any OpenAI-compatible proxy."
        f"{directive_block}{power_override}"
    )


@contextmanager
def _controller_budget(dispatch_cap: int, plan_cap: int):
    keys = {
        "ELAI_WALK_DISPATCH_CAP": str(dispatch_cap),
        "ELAI_WALK_PLAN_CAP": str(plan_cap),
        "ELAI_WALK_WORKER_CONCURRENCY": str(MAX_WORKER_CONCURRENCY),
        "ELAI_WALK_WORKER_CONCURRENCY_DEGRADED": str(RATE_LIMITED_WORKER_CONCURRENCY),
    }
    previous = {key: os.environ.get(key) for key in keys}
    os.environ.update(keys)
    try:
        yield
    finally:
        for key, value in previous.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value


def _depot_law_env() -> dict[str, str]:
    """Depot law for EVERY boss shell (2026-08-07): offload stays remote and a plain `cargo`
    resolves to the depot shim. Injected at the _spawn chokepoint because each boss lane
    kept re-growing the same hole one adapter at a time: official-grok got it via
    build_grok_env, glm got it after its live absolute-path `cargo test` catch, and then the
    MACARON boss did the identical thing on its first revived turn (uncaged `cargo check
    -p elai-orchestrator`, breaker v6, 02:25Z). Absolute-path bypass stays tripwire-enforced;
    this closes the plain-invocation path for all current and future rungs."""
    shim = str(Path(__file__).resolve().parents[1] / "opus_walk" / "depot-shim")
    path_now = os.environ.get("PATH", "")
    return {
        "TEIKOKU_DEPOT_OFFLOAD": os.environ.get("TEIKOKU_DEPOT_OFFLOAD", "remote"),
        "TEIKOKU_DEPOT_OFFLOAD_CONFIG": os.environ.get(
            "TEIKOKU_DEPOT_OFFLOAD_CONFIG",
            "/home/researcher/teikoku/conductor/state/depot_offload_config.json",
        ),
        "DEPOT_ORG": os.environ.get("DEPOT_ORG", "example-org"),
        "PATH": shim + os.pathsep + path_now if shim not in path_now.split(os.pathsep) else path_now,
    }


def _spawn(provider: dict, session_id: str) -> dict:
    if provider["id"] != "sol-fast":
        raise RuntimeError(f"unsupported fallback provider: {provider['id']}")
    with _controller_budget(provider["dispatch_cap"], provider["plan_cap"]):
        return grok_adapter.run_official_grok_turn(
            _boot_prompt(provider, session_id),
            session_id,
            dispatch_cap=provider["dispatch_cap"],
            plan_cap=provider["plan_cap"],
            db_path=str(DB_PATH),
            wire_model=provider["model"],
            effort=provider["effort"],
        )


def _wait(seconds: int) -> bool:
    remaining = max(0, seconds)
    while remaining:
        if STOP_SENTINEL.is_file():
            return False
        step = min(WAIT_TICK_S, remaining)
        time.sleep(step)
        remaining -= step
    return not STOP_SENTINEL.is_file()


def _prepare_work(turn: int) -> tuple[bool, str]:
    try:
        reconciled = glm_supervisor._reconcile_done_plans(str(WALK_STATE))
        if reconciled:
            _emit("FALLBACK_WALK_RECONCILED", {"turn": turn, "plans": reconciled})
        try:
            from opus_walk import pending_replay
            replay = pending_replay.replay_one()
        except Exception as exc:  # noqa: BLE001 - replay failure must be visible and fail safe
            message = f"replay preflight failed: {type(exc).__name__}: {exc}"
            print(f"[fallback-supervisor] {message}", file=sys.stderr, flush=True)
            _emit("FALLBACK_WALK_REPLAY_ERROR", {"turn": turn, "error": message})
            replay = None
        if replay is not None:
            _emit("FALLBACK_WALK_REPLAY", {"turn": turn, **replay})
        if glm_supervisor._no_actionable_work(str(WALK_STATE)):
            if glm_supervisor._try_autoseed(turn):
                return True, "autoseeded"
            return False, "no actionable work and no safe autoseed"
        readiness = dispatch_policy.check_posthog_worker_ready()
        if not readiness.get("ok"):
            return False, str(readiness.get("reason") or "posthog worker readiness check failed")
        if glm_supervisor.disk_hygiene.guard_enabled():
            decision = glm_supervisor.disk_hygiene.disk_guard(
                state_loader=lambda: walk_state.load(str(WALK_STATE))
            )
            if decision["action"] == "halt":
                return False, f"disk guard: {decision['reason']}"
        memory = glm_supervisor.disk_hygiene.mem_guard()
        if memory["action"] == "halt":
            return False, f"memory guard: {memory['reason']}"
    except walk_state.WalkStateError as exc:
        raise RuntimeError(f"walk-state preflight failed: {exc}") from exc
    return True, "actionable work"


def run_supervisor() -> int:
    state = load_runtime_state()
    _emit("FALLBACK_WALK_START", {"priority": [p["provider"] for p in PROVIDERS]})
    print("[fallback-supervisor] priority: " +
          " > ".join(f"{p['provider']}/{p['model']}" for p in PROVIDERS), flush=True)
    turn = 0
    while not STOP_SENTINEL.is_file():
        ready, reason = _prepare_work(turn)
        if not ready:
            print(f"[fallback-supervisor] {reason}; recheck in {NO_WORK_RECHECK_S}s", flush=True)
            _record(state, "wait", reason=reason)
            state["active_provider"] = None
            _save_runtime_state(state)
            if not _wait(NO_WORK_RECHECK_S):
                break
            continue

        provider = select_provider(state)
        if provider is None:
            delay = _next_cooldown_delay(state)
            print(f"[fallback-supervisor] all providers cooling down; retry in {delay}s", flush=True)
            _record(state, "all_unavailable", retry_in_s=delay)
            state["active_provider"] = None
            _save_runtime_state(state)
            if not _wait(delay):
                break
            continue

        turn += 1
        session_id = (
            f"s-fallback-{provider['id']}-{int(time.time())}-{os.getpid()}-{turn}"
        )
        state["active_provider"] = provider["id"]
        _record(state, "turn_start", provider=provider["id"], session_id=session_id)
        _save_runtime_state(state)
        _emit("FALLBACK_TURN_START", {
            "turn": turn,
            "provider": provider["provider"],
            "model": provider["model"],
            "session_id": session_id,
            "dispatch_cap": provider["dispatch_cap"],
            "plan_cap": provider["plan_cap"],
        })
        print(f"[fallback-supervisor] turn {turn}: {provider['provider']} / "
              f"{provider['model']} session={session_id}", flush=True)
        started = time.monotonic()
        result = _spawn(provider, session_id)
        elapsed = time.monotonic() - started
        result = {**result, "duration_s": round(elapsed, 1)}
        failure_class = classify_failure(result)
        state["active_provider"] = None
        state["last_provider"] = provider["id"]
        _record(
            state,
            "turn_done",
            provider=provider["id"],
            rc=result.get("rc"),
            classification=failure_class,
            duration_s=round(elapsed, 1),
        )
        try:
            glm_supervisor.disk_hygiene.reap_after_turn(
                state_loader=lambda: walk_state.load(str(WALK_STATE))
            )
        except Exception as exc:
            print(f"[fallback-supervisor] WARN per-turn reap failed: {type(exc).__name__}: {exc}",
                  flush=True)
        _emit("FALLBACK_TURN_DONE", {
            "turn": turn,
            "provider": provider["provider"],
            "rc": result.get("rc"),
            "classification": failure_class,
            "duration_s": round(elapsed, 1),
            "stderr_tail": str(result.get("stderr_tail") or "")[-1_000:],
            "stdout_tail": str(result.get("stdout_tail") or "")[-1_500:],
            "notes_tail": str(result.get("notes_tail") or "")[-800:],
        })
        print(
            f"[fallback-supervisor] turn_done turn={turn} rc={result.get('rc')} "
            f"class={failure_class} duration_s={round(elapsed, 1)}",
            flush=True,
        )

        if failure_class == "cancelled" or STOP_SENTINEL.is_file():
            _save_runtime_state(state)
            break
        if failure_class in ("quota", "auth", "unavailable"):
            hours = cooldown_hours_for_failure(provider["id"], failure_class, result)
            until = _utc_now() + timedelta(hours=hours)
            state.setdefault("cooldowns", {})[provider["id"]] = {
                "until": _iso(until),
                "reason": failure_class,
            }
            _record(
                state,
                "provider_cooldown",
                provider=provider["id"],
                failure_class=failure_class,
                until=_iso(until),
            )
            _save_runtime_state(state)
            print(f"[fallback-supervisor] {provider['id']} {failure_class}; "
                  f"falling through until {until.isoformat()}", flush=True)
            continue
        if failure_class == "unexpected":
            _record(
                state,
                "halt_unexpected",
                provider=provider["id"],
                stderr=str(result.get("stderr_tail") or "")[-1_000:],
            )
            _save_runtime_state(state)
            print(f"[fallback-supervisor] unexpected {provider['id']} failure; "
                  "halting instead of masking it", flush=True)
            return 1

        state.get("cooldowns", {}).pop(provider["id"], None)
        _save_runtime_state(state)
        if not _wait(INTER_TURN_SLEEP_S):
            break

    state["active_provider"] = None
    _record(state, "stopped")
    _save_runtime_state(state)
    _emit("FALLBACK_WALK_STOPPED", {"turns": turn})
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="print resolved controller wiring without starting a provider turn",
    )
    args = parser.parse_args()
    if args.check:
        state = load_runtime_state()
        selected = select_provider(state)
        print(f"WALK_STATE={WALK_STATE} exists={WALK_STATE.is_file()}")
        print(f"SKILL_PATH={SKILL_PATH} exists={SKILL_PATH.is_file()}")
        print(f"STOP_SENTINEL={STOP_SENTINEL} present={STOP_SENTINEL.is_file()}")
        print("PRIORITY=" + " -> ".join(provider["provider"] for provider in PROVIDERS))
        print(f"NEXT_PROVIDER={selected['provider'] if selected else 'cooldown-wait'}")
        return 0
    return run_supervisor()


if __name__ == "__main__":
    raise SystemExit(main())
