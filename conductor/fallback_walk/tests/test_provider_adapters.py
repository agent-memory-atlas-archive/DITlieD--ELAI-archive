import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from fallback_walk import provider_adapters as adapters


class ProviderAdapterTests(unittest.TestCase):
    def test_official_auth_env_strips_proxy_and_api_key_lanes(self):
        unsafe = {
            "PATH": os.environ.get("PATH", ""),
            "OPENAI_API_KEY": "secret",
            "OPENAI_BASE_URL": "https://proxy.invalid/v1",
            "CODEX_API_KEY": "secret",
            "CODEX_ACCESS_TOKEN": "secret",
            "BMAPI_API_KEY": "secret",
            "ANTHROPIC_BASE_URL": "https://proxy.invalid",
            "ZCODE_API_KEY": "secret",
            "CURSOR_API_KEY": "secret",
        }
        with mock.patch.dict(os.environ, unsafe, clear=True):
            env = adapters.official_auth_env({"ELAI_WALK_DISPATCH_CAP": "10"})
        self.assertIn("PATH", env)
        self.assertEqual(env["ELAI_WALK_DISPATCH_CAP"], "10")
        for key in unsafe:
            if key != "PATH":
                self.assertNotIn(key, env)

    @mock.patch.object(adapters, "_run")
    @mock.patch.object(adapters, "_preflight")
    def test_cursor_uses_exact_official_model_and_fresh_session(self, preflight, run):
        preflight.return_value = (True, "ok")
        run.return_value = {
            "rc": 0,
            "timed_out": False,
            "cancelled": False,
            "stdout_head": (
                '{"type":"system","subtype":"init",'
                '"model":"Cursor Grok 4.5 High"}\n'
            ),
            "stdout_tail": '{"type":"result","subtype":"success"}\n',
            "stderr_tail": "",
        }
        result = adapters.run_cursor_turn(
            "prompt",
            stop_path=Path("/tmp/not-present-fallback-test"),
            extra_env={"ELAI_WALK_DISPATCH_CAP": "10"},
        )
        self.assertEqual(result["rc"], 0)
        command = run.call_args.args[0]
        self.assertIn(adapters.CURSOR_MODEL, command)
        self.assertNotIn("--resume", command)
        self.assertNotIn("--continue", command)
        self.assertIn("--force", command)
        self.assertIn("--sandbox", command)

    @mock.patch.object(adapters, "_run")
    @mock.patch.object(adapters, "_preflight")
    def test_cursor_fails_closed_when_stream_reports_no_model(self, preflight, run):
        preflight.return_value = (True, "ok")
        run.return_value = {
            "rc": 0,
            "timed_out": False,
            "cancelled": False,
            "stdout_head": '{"type":"system","subtype":"init","model":"Other"}\n',
            "stdout_tail": "",
            "stderr_tail": "",
        }
        result = adapters.run_cursor_turn(
            "prompt", stop_path=Path("/tmp/not-present-fallback-test")
        )
        self.assertEqual(result["rc"], 78)

    @mock.patch.object(adapters, "_run")
    @mock.patch.object(adapters, "_preflight")
    def test_codex_uses_chatgpt_auth_and_ignores_proxy_config(self, preflight, run):
        preflight.return_value = (True, "Logged in using ChatGPT")
        run.return_value = {
            "rc": 0,
            "timed_out": False,
            "cancelled": False,
            "stdout_head": "",
            "stdout_tail": "",
            "stderr_tail": "",
        }
        with mock.patch.dict(
            os.environ,
            {"OPENAI_BASE_URL": "https://proxy.invalid", "OPENAI_API_KEY": "secret"},
            clear=False,
        ):
            adapters.run_codex_turn(
                "prompt", stop_path=Path("/tmp/not-present-fallback-test")
            )
        command = run.call_args.args[0]
        env = run.call_args.kwargs["env"]
        self.assertIn("--ignore-user-config", command)
        self.assertIn("--ephemeral", command)
        self.assertIn(adapters.CODEX_MODEL, command)
        self.assertNotIn("OPENAI_BASE_URL", env)
        self.assertNotIn("OPENAI_API_KEY", env)
        self.assertNotIn("bmapi", " ".join(command).lower())


if __name__ == "__main__":
    unittest.main()
