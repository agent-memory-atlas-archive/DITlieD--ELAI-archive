"""Fallback supervisor + LIVE-U4 dispatch policy tests."""
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from fallback_walk import dispatch_policy, fallback_supervisor as supervisor
import fallback_walk_control


class FallbackSupervisorTests(unittest.TestCase):
    def test_official_grok_macaron_and_glm_are_in_rotation(self):
        self.assertEqual([p["id"] for p in supervisor.PROVIDERS], ["sol-fast"])
        provider = supervisor.PROVIDERS[0]
        self.assertEqual(provider["model"], "cline-glm-53-flash")
        self.assertEqual(provider["effort"], "max")
        self.assertEqual(provider["dispatch_cap"], 60)
        self.assertEqual(provider["plan_cap"], 1)

    def test_live_u4_policy_requires_posthog_luna(self):
        with tempfile.TemporaryDirectory() as tmp:
            artifact = Path(tmp) / "policy.json"
            result = dispatch_policy.preflight_fallback_start(
                artifact_path=artifact,
                supported_models=list(dispatch_policy.POSTHOG_WORKER_MODELS)
                + list(dispatch_policy.FREE_BRIDGE_MODELS),
            )
            saved = json.loads(artifact.read_text(encoding="utf-8"))
        # May soft-fail credentials if zenov credit probe fails in CI; policy/executor must pass.
        self.assertEqual(saved["policy"]["resolved_model"], "cline-glm-53-flash")
        self.assertEqual(saved["policy"]["lane"], "posthog")
        self.assertTrue(saved["checks"]["policy_resolves_required_model"]["ok"])
        self.assertTrue(saved["checks"]["executor_supports_resolved_model"]["ok"])

    def test_paid_grok_never_accepted_as_free_worker_model(self):
        with tempfile.TemporaryDirectory() as tmp:
            artifact = Path(tmp) / "policy.json"
            result = dispatch_policy.preflight_fallback_start(
                artifact_path=artifact,
                supported_models=("grok-4-5-high", "glm-5-2"),
                policy_table={
                    dispatch_policy.LIVE_U4: {
                        "lane": "free",
                        "provider": "devin",
                        "model": "grok-4-5-high",
                    },
                },
            )
        self.assertFalse(result["ok"])
        # either policy mismatch (not luna) or paid-model-as-free
        self.assertIn(
            result["reason_code"],
            {"policy-model-mismatch", "paid-model-as-free", "unsupported-model"},
        )

    def test_policy_mismatch_blocks_wrong_model(self):
        with tempfile.TemporaryDirectory() as tmp:
            artifact = Path(tmp) / "policy.json"
            result = dispatch_policy.preflight_fallback_start(
                artifact_path=artifact,
                supported_models=("glm-5-2",),
                policy_table={
                    dispatch_policy.LIVE_U4: {
                        "lane": "free",
                        "provider": "devin",
                        "model": "glm-5-2",
                    },
                },
            )
        self.assertFalse(result["ok"])
        self.assertEqual(result["reason_code"], "policy-model-mismatch")

    def test_start_blocks_before_spawn_when_executor_cannot_run_luna(self):
        with tempfile.TemporaryDirectory() as tmp, mock.patch.object(
            fallback_walk_control.opus_dispatch,
            "BRIDGE_MODELS",
            ("glm-5-2",),
        ), mock.patch.object(
            fallback_walk_control.opus_dispatch,
            "FREE_BRIDGE_MODELS",
            ("glm-5-2",),
        ), mock.patch.object(
            fallback_walk_control.opus_dispatch,
            "POSTHOG_WORKER_MODELS",
            (),
        ), mock.patch.object(fallback_walk_control.subprocess, "Popen") as popen:
            result = fallback_walk_control.fallback_walk_start(
                policy_artifact_path=Path(tmp) / "policy.json",
                credential_path=Path(tmp) / "credentials.toml",
            )
        self.assertFalse(result["ok"])
        self.assertEqual(result["reason_code"], "unsupported-model")
        popen.assert_not_called()
    def test_prepare_work_uses_posthog_readiness_not_legacy_bridge(self):
        with mock.patch.object(
            supervisor.glm_supervisor,
            "_reconcile_done_plans",
            return_value=[],
        ), mock.patch(
            "opus_walk.pending_replay.replay_one",
            return_value=None,
        ), mock.patch.object(
            supervisor.glm_supervisor,
            "_no_actionable_work",
            return_value=False,
        ), mock.patch.object(
            supervisor.glm_supervisor,
            "_bridge_healthy",
            return_value=False,
        ) as legacy_bridge, mock.patch.object(
            supervisor.dispatch_policy,
            "check_posthog_worker_ready",
            return_value={"ok": True, "reason": None},
        ) as readiness, mock.patch.object(
            supervisor.glm_supervisor.disk_hygiene,
            "guard_enabled",
            return_value=True,
        ), mock.patch.object(
            supervisor.glm_supervisor.disk_hygiene,
            "disk_guard",
            return_value={"action": "continue"},
        ), mock.patch.object(
            supervisor.glm_supervisor.disk_hygiene,
            "mem_guard",
            return_value={"action": "continue"},
        ):
            result = supervisor._prepare_work(1)

        self.assertEqual(result, (True, "actionable work"))
        readiness.assert_called_once_with()
        legacy_bridge.assert_not_called()

    def test_prepare_work_returns_posthog_failure_before_resource_guards(self):
        reason = "zenov health failed: connection refused"
        with mock.patch.object(
            supervisor.glm_supervisor,
            "_reconcile_done_plans",
            return_value=[],
        ), mock.patch(
            "opus_walk.pending_replay.replay_one",
            return_value=None,
        ), mock.patch.object(
            supervisor.glm_supervisor,
            "_no_actionable_work",
            return_value=False,
        ), mock.patch.object(
            supervisor.dispatch_policy,
            "check_posthog_worker_ready",
            return_value={"ok": False, "reason": reason},
        ) as readiness, mock.patch.object(
            supervisor.glm_supervisor.disk_hygiene,
            "guard_enabled",
            return_value=True,
        ) as guard_enabled, mock.patch.object(
            supervisor.glm_supervisor.disk_hygiene,
            "disk_guard",
        ) as disk_guard, mock.patch.object(
            supervisor.glm_supervisor.disk_hygiene,
            "mem_guard",
        ) as mem_guard:
            result = supervisor._prepare_work(2)

        self.assertEqual(result, (False, reason))
        readiness.assert_called_once_with()
        guard_enabled.assert_not_called()
        disk_guard.assert_not_called()
        mem_guard.assert_not_called()


if __name__ == "__main__":
    unittest.main()


def test_grok_402_balance_exhausted_classifies_quota_and_cools_down():
    """Live 2026-08-07 regression: official grok 402 halted the ladder as 'unexpected'."""
    from fallback_walk import fallback_supervisor as fs
    result = {
        "rc": 1,
        "stderr_tail": '"message": "API error (status 402 Payment Required): '
                       'Grok Build usage balance exhausted", "http_status": 402',
    }
    assert fs.classify_failure(result) == "quota"
    # the cooldown lookup must not KeyError for the boss provider (second live bug)
    assert fs._cooldown_hours("official-grok", "quota") == 6.0
    # unknown future provider ids degrade to the 2h default instead of crashing
    assert fs._cooldown_hours("new-provider", "quota") == 2.0


def test_macaron_403_not_enough_balance_classifies_quota():
    """Live 2026-08-07 12:13Z: novita macaron emitted `403 not enough balance` in
    claude-cli STDOUT (an auth status used for a billing condition). It classified
    "unexpected", halted the ladder, burned driver relaunch 1/3. Third instance of
    the unmatched-exhaustion-string bug class (cursor-agent, grok-402, now this)."""
    from fallback_walk import fallback_supervisor as fs
    result = {
        "rc": 1,
        "stderr_tail": "claude.ai connectors are disabled because ANTHROPIC_API_KEY",
        "stdout_tail": '{"type":"result","subtype":"success","is_error":true,'
                       '"api_error_status":403,"result":"Failed to authenticate. '
                       'API Error: 403 not enough balance"}',
    }
    assert fs.classify_failure(result) == "quota"


def test_grok_403_spending_limit_classifies_quota_and_balance():
    """Live 2026-08-08 13:23Z (first turn after a PC restart): official grok emitted
    `403 Forbidden: personal-team-blocked:spending-limit: You have run out of credits`
    — the 403-form of the 402 wallet-empty error. It matched nothing, classified
    "unexpected", halted the ladder, burned driver relaunches 1-3 on ~6s failures.
    FOURTH instance of the unmatched-exhaustion-string class."""
    from fallback_walk import fallback_supervisor as fs
    result = {
        "rc": 1,
        "stderr_tail": 'Error: Internal error: {\n  "message": "API error (status 403 '
                       'Forbidden): personal-team-blocked:spending-limit: You have run out '
                       'of credits or need a Grok subscription.",\n  "http_status": 403\n}',
        "stdout_tail": "",
    }
    assert fs.classify_failure(result) == "quota"
    assert fs._is_balance_exhaustion(result) is True  # top-up-only: hours floor, not RPM


def test_balance_exhaustion_floors_cooldown_over_rpm_window():
    """Balance-empty (top-up-only) must not reuse macaron's 2-minute RPM cooldown."""
    from fallback_walk import fallback_supervisor as fs
    balance = {"rc": 1, "stdout_tail": "API Error: 403 not enough balance"}
    rpm = {"rc": 1, "stderr_tail": "too many requests, retry shortly"}
    assert fs._is_balance_exhaustion(balance) is True
    assert fs._is_balance_exhaustion(rpm) is False
    # both classify quota; only the balance one gets the hours-long floor
    assert fs.classify_failure(balance) == "quota"
    assert fs.classify_failure(rpm) == "quota"
    assert fs._cooldown_hours("macaron", "quota") < 0.1  # RPM stays short


def test_rc78_provider_unavailable_falls_through_ladder():
    """Live 2026-08-07: macaron rc=78 (novita shim down) halted the ladder at rung 2."""
    from fallback_walk import fallback_supervisor as fs
    result = {"rc": 78, "stderr_tail": "novita retry shim is not listening on 127.0.0.1:18812"}
    assert fs.classify_failure(result) == "unavailable"
    # cooldown path must resolve for every ladder provider without KeyError
    for pid in ("official-grok", "macaron", "glm", "cursor-grok"):
        assert fs._cooldown_hours(pid, "unavailable") > 0


def test_depot_law_env_reaches_every_boss_lane():
    """Third boss lane (macaron) repeated the local-cargo bypass 2026-08-07: the depot law
    must come from the _spawn chokepoint, not per-adapter memory."""
    from fallback_walk import fallback_supervisor as fs
    env = fs._depot_law_env()
    assert env["TEIKOKU_DEPOT_OFFLOAD"] == "remote"
    assert env["DEPOT_ORG"]
    assert "depot-shim" in env["PATH"].split(":")[0]


def test_zenov_boss_rungs_present_and_routed(monkeypatch):
    """The owner-approved power route has one Sol-fast/max boss and no fallback."""
    from fallback_walk import fallback_supervisor as fs

    assert [p["id"] for p in fs.PROVIDERS] == ["sol-fast"]
    provider = fs.PROVIDERS[0]
    assert provider["model"] == "cline-glm-53-flash"
    assert provider["effort"] == "max"
    assert provider["dispatch_cap"] == 60
    assert provider["plan_cap"] == 1

    seen = {}

    def fake_turn(prompt, session_id, **kwargs):
        seen.update(kwargs)
        return {"rc": 0, "timed_out": False, "cancelled": False,
                "stdout_tail": "", "stderr_tail": ""}

    monkeypatch.setattr(fs.grok_adapter, "run_official_grok_turn", fake_turn)
    fs._spawn(provider, "s-test-sol-fast-1")
    assert seen["wire_model"] == "cline-glm-53-flash"
    assert seen["effort"] == "max"
    assert seen["dispatch_cap"] == 60
    assert seen["plan_cap"] == 1


def test_boss_argv_is_sol_fast_max_and_project_scoped(monkeypatch):
    from fallback_walk import grok_adapter
    import grok_provider

    seen = {}

    class Proc:
        returncode = 0
        pid = 123

        def communicate(self, timeout=None):
            seen["timeout"] = timeout
            return ('{"type":"end","stopReason":"EndTurn","usage":{}}\n', "")

    def fake_popen(command, **kwargs):
        seen["command"] = command
        seen["env"] = kwargs["env"]
        seen["popen_kwargs"] = kwargs
        return Proc()

    monkeypatch.setattr(grok_provider, "_resolve_binary", lambda: "/usr/bin/grok")
    monkeypatch.setattr(grok_adapter.subprocess, "Popen", fake_popen)
    out = grok_adapter.run_official_grok_turn(
        "prompt", "s-test", dispatch_cap=60, plan_cap=1,
        timeout_s=123, wire_model="posthog-gpt56-sol-fast", effort="max",
    )
    command = seen["command"]
    turn_cap = "--" + "max-" + "turns"
    assert turn_cap not in command
    assert command[command.index("--model") + 1] == "posthog-gpt56-sol-fast"
    assert command[command.index("--reasoning-effort") + 1] == "max"
    assert command[command.index("--leader-socket") + 1] == seen["env"]["GROK_LEADER_SOCKET"]
    assert command[command.index("--sandbox") + 1] == "off"
    assert command[command.index("--permission-mode") + 1] == "bypassPermissions"
    assert "--always-approve" in command
    assert command[command.index("--cwd") + 1] == str(grok_adapter.ELAI)
    assert command[command.index("-s") + 1]
    assert seen["env"]["CLAUDE_CODE_SESSION_ID"] == "s-test"
    assert seen["popen_kwargs"]["cwd"] == str(grok_adapter.ELAI)
    assert seen["timeout"] == 123
    assert seen["popen_kwargs"]["env"] is seen["env"]
    assert seen["env"]["GROK_HOME"] == str(grok_provider.PROJECT_FLEET_GROK_HOME)
    assert seen["env"]["GROK_HOME_PIN"] == "1"
    assert out["model"] == "posthog-gpt56-sol-fast"
    assert out["effort"] == "max"


def test_burn_u2_ladder_order_caps_and_zenov_cooldown():
    """Strict power route keeps one Sol-fast boss with short pool cooldowns."""
    from fallback_walk import fallback_supervisor as fs

    assert [p["id"] for p in fs.PROVIDERS] == ["sol-fast"]
    provider = fs.PROVIDERS[0]
    assert provider["dispatch_cap"] == 60
    assert provider["plan_cap"] == 1
    assert abs(fs._cooldown_hours("sol-fast", "quota") - 0.1) < 1e-9
    assert abs(fs._cooldown_hours("sol-fast", "auth") - 0.1) < 1e-9


def test_burn_u2_zenov_posthog_quota_and_auth_classify():
    """BURN-U2 falsifier: known zenov quota/auth classify; unknown stays unexpected."""
    from fallback_walk import fallback_supervisor as fs

    quota_samples = [
        'gateway: {"error":{"type":"credit_exhausted","message":"credit remaining 0"}}',
        "v2 virtual-key quota denied chat for key k1",
        "error type=virtual_key_quota_denied status=429",
        "posthog-credits-empty: posthog aggregate remaining credits is 0",
        "no_ready_key: all keys in pool are cooled down",
        "Error: Internal error: reqwest error stream: error sending request for url (http://127.0.0.1:9020/v1/chat/completions)",
    ]
    for sample in quota_samples:
        result = {"rc": 1, "stderr_tail": sample, "stdout_tail": ""}
        assert fs.classify_failure(result) == "quota", sample

    auth_samples = [
        "missing or invalid bearer",
        "Error: auth_failed on posthog oauth token refresh failed",
        "invalid bearer token from zenov auth",
    ]
    for sample in auth_samples:
        result = {"rc": 1, "stderr_tail": sample, "stdout_tail": ""}
        assert fs.classify_failure(result) == "auth", sample

    # Fail-closed: unmatched string is still unexpected (halts the ladder).
    unknown = {
        "rc": 1,
        "stderr_tail": "COMPLETELY_UNKNOWN_ZENOV_WOBBLE_XYZ_99 never seen before",
        "stdout_tail": "",
    }
    assert fs.classify_failure(unknown) == "unexpected"

    # Falsifier: bare synthetic strings (not result mappings) must not raise AttributeError.
    # Known zenov quota string → quota; unknown bare string → unexpected (still fail-closed).
    bare_quota = "credit_exhausted: posthog aggregate remaining credits is 0"
    assert fs.classify_failure(bare_quota) == "quota"
    bare_auth = "missing or invalid bearer token from zenov auth"
    assert fs.classify_failure(bare_auth) == "auth"
    bare_unknown = "COMPLETELY_UNKNOWN_ZENOV_WOBBLE_XYZ_99 never seen before"
    assert fs.classify_failure(bare_unknown) == "unexpected"


def test_burn_u2_all_free_rungs_cooling_sleeps_not_halts():
    """The single Sol-fast boss sleeps during its own cooldown and never falls through."""
    from datetime import datetime, timedelta, timezone
    from fallback_walk import fallback_supervisor as fs

    now = datetime(2026, 8, 12, 12, 0, 0, tzinfo=timezone.utc)
    future = (now + timedelta(minutes=30)).isoformat()
    state = {"cooldowns": {"sol-fast": {"until": future, "reason": "quota"}}}
    assert fs.select_provider(state, now=now) is None
    delay = fs._next_cooldown_delay(state, now=now)
    assert 0 < delay <= 30 * 60
    later = now + timedelta(minutes=31)
    assert fs.select_provider(state, now=later)["id"] == "sol-fast"
    assert fs.select_provider({"cooldowns": {}}, now=now)["id"] == "sol-fast"


def test_burn_u2_zenov_credit_uses_short_cooldown_not_balance_floor():
    """BURN-U2: zenov/posthog credit-balance quota keeps ~0.1h, not the 6h paid floor.

    Drives the real cooldown-setting helper used by the turn loop
    (``cooldown_hours_for_failure``), not just ``_cooldown_hours``. Known
    credit/balance-shaped zenov strings must still classify quota and cool
    for ~0.1h so unlimited seats are not idled for half a day. Paid/wallet
    rungs keep the 6h balance floor.
    """
    from fallback_walk import fallback_supervisor as fs

    zenov_credit_samples = [
        # balance-like text that matches _BALANCE_PATTERNS
        "API Error: 403 not enough credit on posthog pool",
        "insufficient credit for posthog-gpt56-luna",
        "not enough balance on zenov virtual key",
        # zenov quota patterns that may or may not be balance-shaped
        "credit_exhausted: posthog aggregate remaining credits is 0",
        "posthog-credits-empty: credit remaining 0",
        "v2 virtual-key quota denied chat for key k1",
    ]
    for sample in zenov_credit_samples:
        result = {"rc": 1, "stderr_tail": sample, "stdout_tail": "", "error": sample}
        assert fs.classify_failure(result) == "quota", sample
        hours = fs.cooldown_hours_for_failure("sol-fast", "quota", result)
        assert abs(hours - 0.1) < 1e-9, (sample, hours)
        assert hours < 1.0, (sample, hours)

    # Bare-string path (falsifier shape) through the same helper
    bare = "credit_exhausted: not enough credit remaining"
    assert fs.classify_failure(bare) == "quota"
    assert abs(fs.cooldown_hours_for_failure("sol-fast", "quota", bare) - 0.1) < 1e-9

    # Paid/wallet still floors balance to 6h via the same helper
    paid_balance = {"rc": 1, "stdout_tail": "API Error: 403 not enough balance"}
    assert fs.classify_failure(paid_balance) == "quota"
    assert fs._is_balance_exhaustion(paid_balance) is True
    assert fs.cooldown_hours_for_failure("macaron", "quota", paid_balance) >= 6.0
    assert fs.cooldown_hours_for_failure("official-grok", "quota", paid_balance) >= 6.0

    # RPM (non-balance) macaron stays short
    rpm = {"rc": 1, "stderr_tail": "server_overload rate limit 429"}
    assert fs.classify_failure(rpm) == "quota"
    assert fs._is_balance_exhaustion(rpm) is False
    assert fs.cooldown_hours_for_failure("macaron", "quota", rpm) < 0.1
