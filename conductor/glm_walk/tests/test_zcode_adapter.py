#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import signal
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(HERE))
import zcode_adapter as zc  # noqa: E402

FAKE = Path(__file__).with_name("fake_zcode_child.py")


class AdapterCase(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="zcode-adapter-test-")
        self.log = Path(self.tmp.name) / "protocol.jsonl"
        self.config = Path(self.tmp.name) / "desktop-config.json"
        self.secret = "offline-zcode-secret-must-never-escape"
        self.write_config()

    def tearDown(self):
        self.tmp.cleanup()

    def provider_config(self):
        return {
            "provider": {
                "builtin:zai-coding-plan": {
                    "enabled": True,
                    "options": {
                        "apiKey": self.secret,
                        "baseURL": "https://api.z.ai/api/anthropic",
                    },
                    "models": {"GLM-5.2": {"limit": {"context": 1_000_000}}},
                }
            }
        }

    def write_config(self, config=None):
        self.config.write_text(
            json.dumps(config if config is not None else self.provider_config()),
            encoding="utf-8",
        )

    def run_fake(self, scenario="success", timeout=3, stop=None, policy="allow"):
        return zc.ZCodeAppServer(
            command=(sys.executable, str(FAKE)), timeout_s=timeout,
            permission_policy=policy,
            extra_env={"FAKE_ZCODE_SCENARIO": scenario, "FAKE_ZCODE_LOG": str(self.log)},
            desktop_config_path=self.config,
        ).run_turn("controller prompt", "controller-turn-1", stop)

    def frames(self):
        return [json.loads(line) for line in self.log.read_text(encoding="utf-8").splitlines()]

    def assert_dead(self, pid):
        with self.assertRaises(ProcessLookupError):
            os.kill(pid, 0)

    def test_exact_production_command_environment_model_and_workspace(self):
        self.assertEqual(zc.ZCODE_COMMAND, (
            "/opt/ZCode/zcode", "/opt/ZCode/resources/glm/zcode.cjs", "app-server",
            "--cwd", "/home/researcher/teikoku"))
        self.assertEqual(zc.ZCODE_ENV, {
            "ELECTRON_RUN_AS_NODE": "1", "ZCODE_APP_VERSION": "3.3.6", "ZCODE_ENV": "production"})
        self.assertEqual(zc.EXPECTED_BASE_URL, "https://api.z.ai/api/anthropic")
        self.assertEqual(zc.MODEL_REF, {"providerId": "builtin:zai-coding-plan", "modelId": "GLM-5.2"})
        self.assertEqual(zc.EXPECTED_CONTEXT_WINDOW, 1_000_000)
        result = self.run_fake()
        create = next(f for f in self.frames() if f.get("method") == "session/create")
        self.assertEqual(create["params"]["workspace"], {
            "workspacePath": "/home/researcher/teikoku", "workspaceKey": "/home/researcher/teikoku"})
        self.assertEqual(create["params"]["model"], zc.MODEL_REF)
        self.assertEqual(result["stdout_tail"], "fake controller complete")
        self.assertEqual(result["stderr_tail"], "")

    def test_controller_env_reaches_tools_without_claude_secrets(self):
        with patch.dict(os.environ, {
            "ANTHROPIC_API_KEY": "must-not-leak",
            "CLAUDE_CODE_OAUTH_TOKEN": "must-not-leak",
        }):
            adapter = zc.ZCodeAppServer(
                command=(sys.executable, str(FAKE)), timeout_s=3,
                extra_env={
                    "FAKE_ZCODE_LOG": str(self.log),
                    "CLAUDE_CODE_SESSION_ID": "controller-session",
                    "TEIKOKU_AUTONOMOUS": "1",
                    "ELAI_OPUS_CODEX_EXECUTOR": "1",
                },
                desktop_config_path=self.config,
            )
            adapter.run_turn("prompt", "turn")
        startup = next(f["startup_env"] for f in self.frames() if "startup_env" in f)
        self.assertEqual(startup["CLAUDE_CODE_SESSION_ID"], "controller-session")
        self.assertEqual(startup["TEIKOKU_AUTONOMOUS"], "1")
        self.assertEqual(startup["ELAI_OPUS_CODEX_EXECUTOR"], "1")
        self.assertFalse(startup["claude_secret_present"])
        self.assertTrue(startup["zcode_api_key_present"])

    def test_desktop_provider_preflight_fails_closed_on_every_invariant(self):
        cases = {}

        missing_provider = self.provider_config()
        missing_provider["provider"] = {}
        cases["provider"] = missing_provider

        disabled = self.provider_config()
        disabled["provider"]["builtin:zai-coding-plan"]["enabled"] = False
        cases["disabled"] = disabled

        wrong_base = self.provider_config()
        wrong_base["provider"]["builtin:zai-coding-plan"]["options"]["baseURL"] = "https://invalid.example"
        cases["base URL mismatch"] = wrong_base

        missing_model = self.provider_config()
        missing_model["provider"]["builtin:zai-coding-plan"]["models"] = {}
        cases["model"] = missing_model

        wrong_context = self.provider_config()
        wrong_context["provider"]["builtin:zai-coding-plan"]["models"]["GLM-5.2"]["limit"]["context"] = 200_000
        cases["context mismatch"] = wrong_context

        missing_key = self.provider_config()
        missing_key["provider"]["builtin:zai-coding-plan"]["options"]["apiKey"] = "  "
        cases["API key is absent"] = missing_key

        for message, config in cases.items():
            with self.subTest(message=message):
                self.write_config(config)
                if self.log.exists():
                    self.log.unlink()
                result = self.run_fake()
                self.assertEqual(result["rc"], 1)
                self.assertIn(message, result["stderr_tail"])
                self.assertNotIn(self.secret, json.dumps(result, default=str))
                self.assertFalse(self.log.exists(), "preflight failure must not spawn the child")

    def test_runtime_secret_is_never_captured_and_source_is_unchanged(self):
        before = self.config.read_bytes()
        for scenario in ("secret-output", "secret-error"):
            with self.subTest(scenario=scenario):
                if self.log.exists():
                    self.log.unlink()
                result = self.run_fake(scenario)
                captured = json.dumps(result, sort_keys=True, default=str)
                captured += self.log.read_text(encoding="utf-8")
                self.assertNotIn(self.secret, captured)
                self.assertIn("<redacted>", captured)
                startup = next(f["startup_env"] for f in self.frames() if "startup_env" in f)
                self.assertTrue(startup["zcode_api_key_present"])
                self.assertEqual(self.config.read_bytes(), before)

    def test_caller_cannot_override_runtime_api_key(self):
        with self.assertRaisesRegex(ValueError, "must not provide ZCODE_API_KEY"):
            zc.ZCodeAppServer(
                command=(sys.executable, str(FAKE)),
                extra_env={"ZCODE_API_KEY": "caller-secret"},
                desktop_config_path=self.config,
            )

    def test_fresh_create_and_set_model_each_turn_never_resume(self):
        self.run_fake()
        self.run_fake()
        methods = [f.get("method") for f in self.frames()]
        self.assertEqual(methods.count("session/create"), 2)
        self.assertEqual(methods.count("session/setModel"), 2)
        self.assertNotIn("session/resume", methods)

    def test_permission_policy_and_event_result(self):
        result = self.run_fake("permission")
        response = next(f for f in self.frames() if f.get("id") == "permission-1" and "result" in f)
        self.assertEqual(response["result"]["decision"], "allow")
        self.assertEqual(result["rc"], 0)
        self.assertTrue(any("decision=allow" in d for d in result["diagnostics"]))

    def test_wrong_model_and_context_fail_closed(self):
        for scenario in ("wrong-model", "wrong-context"):
            with self.subTest(scenario=scenario):
                result = self.run_fake(scenario)
                self.assertEqual(result["rc"], 1)
                self.assertIn("mismatch", result["stderr_tail"])
                self.assert_dead(result["child_pid"])

    def test_timeout_eof_and_malformed_cleanup(self):
        for scenario, timeout in (("timeout", .4), ("eof", 2), ("malformed", 2)):
            with self.subTest(scenario=scenario):
                result = self.run_fake(scenario, timeout=timeout)
                self.assertNotEqual(result["rc"], 0)
                self.assert_dead(result["child_pid"])

    def test_stop_sentinel_cancels_session_and_child(self):
        stop = Path(self.tmp.name) / "GLM_WALK_STOP"
        threading.Timer(.3, lambda: stop.write_text("stop", encoding="utf-8")).start()
        result = self.run_fake("wait-cancel", timeout=3, stop=stop)
        self.assertTrue(result["cancelled"])
        self.assertEqual(result["rc"], 130)
        self.assertIn("session/stop", [f.get("method") for f in self.frames()])
        self.assert_dead(result["child_pid"])


if __name__ == "__main__":
    unittest.main()
