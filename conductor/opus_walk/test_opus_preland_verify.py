import json
import os
import subprocess
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

import opus_land as ol
import opus_preland_verify as pv
from fallback_walk import grok_adapter


def _structured(verdict="APPROVE", grounds="ok"):
    structured = {"verdict": verdict, "grounds": grounds}
    return {
        "ok": True,
        "text": json.dumps(structured),
        "structured_output": structured,
        "rc": 0,
        "error": None,
    }


def _spawn(result=None, exc=None, record=None):
    def spawn_fn(prompt, model):
        if record is not None:
            record["model"] = model
            record["prompt"] = prompt
        if exc is not None:
            raise exc
        return result
    return spawn_fn


def run_sandbox_probe(root: Path, outside_marker: Path) -> dict:
    """Derive and run the test-only one-read-tool strict-sandbox probe."""
    captured = {}
    probe_prompt = (
        "Read the exact outside path in the user request with the single read_file tool. "
        f"The outside path is {outside_marker}. Return a JSON object with the exact tool outcome "
        "and do not repeat file contents."
    )
    probe_schema = {
        "type": "object",
        "additionalProperties": False,
        "required": ["outcome"],
        "properties": {"outcome": {"type": "string", "minLength": 1}},
    }
    probe_rubric = (
        "Use only the single GrokBuild:read_file tool. Read the exact requested path. "
        "Return exactly one JSON object with exactly one key, outcome. "
        "The outcome must report the actual tool result, including an explicit denial."
    )

    def _replace_arg(command, flag, value):
        derived = list(command)
        index = derived.index(flag)
        derived[index + 1] = value
        return derived

    def _production_popen(command, **kwargs):
        base_command = list(command)
        base_cwd = str(kwargs["cwd"])
        base_env = dict(kwargs["env"])
        base_profile_path = Path(base_command[base_command.index("--agent") + 1])
        base_profile = base_profile_path.read_text(encoding="utf-8")
        captured.update({
            "production_base_argv": base_command,
            "production_base_cwd": base_cwd,
            "production_base_environment": base_env,
            "production_base_profile_path": str(base_profile_path),
            "production_base_profile": base_profile,
        })
        one_tool_config = (
            "toolConfig:\n"
            "  tools:\n"
            "    - id: GrokBuild:read_file\n"
            "      params: null\n"
            "      name_override: read_file\n"
            "      params_name_overrides: null\n"
            "      description_override: null\n"
            "      behavior_version: null\n"
            "      kind: read"
        )
        derived_profile = base_profile.replace(
            "toolConfig:\n  tools: []", one_tool_config, 1
        ).replace("\ntools: []\n---", "\ntools:\n  - read_file\n---", 1)
        if derived_profile == base_profile:
            raise AssertionError("production profile did not expose the pinned empty tool fields")
        probe_profile_path = base_profile_path.with_name(
            f"{base_profile_path.stem}-u2-probe.md"
        )
        probe_profile_path.write_text(derived_profile, encoding="utf-8")
        derived_command = _replace_arg(base_command, "--agent", str(probe_profile_path))
        derived_command = _replace_arg(derived_command, "-p", probe_prompt)
        derived_command = _replace_arg(
            derived_command,
            "--json-schema",
            json.dumps(probe_schema, separators=(",", ":")),
        )
        derived_command = _replace_arg(
            derived_command, "--system-prompt-override", probe_rubric
        )
        captured.update({
            "derived_probe_argv": list(derived_command),
            "derived_probe_cwd": base_cwd,
            "derived_probe_environment": dict(base_env),
            "derived_probe_profile_path": str(probe_profile_path),
            "derived_probe_profile": derived_profile,
        })
        expected_replacements = {"--agent", "-p", "--json-schema", "--system-prompt-override"}
        if len(base_command) != len(derived_command):
            raise AssertionError("derived probe changed production argv length")
        for index, (base_value, derived_value) in enumerate(zip(base_command, derived_command)):
            if base_value == derived_value:
                continue
            if index == 0 or base_command[index - 1] not in expected_replacements:
                raise AssertionError("derived probe changed an unexpected argv field")
        for flag in ("--no-memory", "--no-subagents", "--disable-web-search", "--sandbox", "--cwd"):
            if flag not in derived_command:
                raise AssertionError(f"derived probe lost pinned flag: {flag}")
        turn_cap_fields = ("max" + "Turns", "--" + "max-" + "turns", "MAX" + "_TURNS")
        if any(value in derived_command for value in turn_cap_fields):
            raise AssertionError("derived probe retained a turn-count cap")
        if derived_command.count("--no-memory") != 1 or derived_command.count("--no-subagents") != 1:
            raise AssertionError("derived probe changed zero-tool isolation controls")
        if derived_command[derived_command.index("--sandbox") + 1] != "strict":
            raise AssertionError("derived probe changed sandbox profile")
        if derived_command[derived_command.index("--cwd") + 1] != base_cwd:
            raise AssertionError("derived probe changed production cwd")
        if any(flag in derived_command for flag in ("--leader-socket", "--resume", "--continue")):
            raise AssertionError("derived probe gained a leader or resume flag")
        if derived_profile.count("id: GrokBuild:read_file") != 1:
            raise AssertionError("derived probe does not define exactly one read_file tool")
        if "toolConfig:\n  tools: []" in derived_profile or "\ntools: []\n---" in derived_profile:
            raise AssertionError("derived probe retained an empty tool field")
        if "\ntools:\n  - read_file\n---" not in derived_profile:
            raise AssertionError("derived probe top-level tool list is not exact")
        try:
            completed = subprocess.run(
                derived_command,
                cwd=base_cwd,
                env=base_env,
                capture_output=True,
                text=True,
                check=False,
                timeout=180,
            )
            captured["derived_probe_stdout"] = completed.stdout
            captured["derived_probe_stderr"] = completed.stderr
            captured["derived_probe_returncode"] = completed.returncode
        finally:
            try:
                probe_profile_path.unlink()
            except OSError:
                pass
        output = captured["derived_probe_stdout"] + captured["derived_probe_stderr"]
        from urllib.parse import quote

        binary_text = Path(captured["production_base_argv"][0]).read_text(encoding="utf-8")
        grok_home = Path(
            binary_text.split('export GROK_HOME="', 1)[1].split('"', 1)[0]
        ) if 'export GROK_HOME="' in binary_text else Path(
            base_env.get("GROK_HOME", "/home/researcher/teikoku/Ougi/grok-build-main/.grok")
        )
        session_root = grok_home / "sessions" / quote(base_cwd, safe="")
        session_events = []
        selected_events = None
        selected_updates = None
        for _ in range(200):
            session_events = sorted(
                session_root.glob("*/events.jsonl"),
                key=lambda path: path.stat().st_mtime_ns,
                reverse=True,
            )
            for candidate in session_events:
                candidate_events = candidate.read_text(encoding="utf-8")
                candidate_updates_path = candidate.parent / "updates.jsonl"
                candidate_updates = (
                    candidate_updates_path.read_text(encoding="utf-8")
                    if candidate_updates_path.is_file()
                    else ""
                )
                if (
                    '"type":"turn_ended"' in candidate_events
                    and '"type":"tool_started","tool_name":"read_file"' in candidate_events
                    and "Permission denied:" in candidate_updates
                ):
                    events_path = candidate
                    selected_events = candidate_events
                    selected_updates = candidate_updates
                    break
            if selected_events is not None:
                break
            time.sleep(0.05)
        if selected_events is None:
            raise AssertionError("derived probe session trace is missing or incomplete")
        session_dir = events_path.parent
        trace_parts = [selected_events, selected_updates]
        trace = "\n".join(trace_parts)
        captured["derived_probe_session_dir"] = str(session_dir)
        captured["derived_probe_events"] = trace_parts[0]
        captured["derived_probe_updates"] = trace_parts[1] if len(trace_parts) > 1 else ""
        captured["derived_probe_trace"] = trace
        read_starts = trace.count('"type":"tool_started","tool_name":"read_file"')
        if read_starts != 1:
            raise AssertionError(f"derived probe read_file tool count was {read_starts}")
        if "Permission denied:" not in trace:
            raise AssertionError("derived probe produced no explicit sandbox denial")
        if outside_marker.read_text(encoding="utf-8") in trace:
            raise AssertionError("sandbox probe leaked the canary content")

        class _ProductionProcess:
            returncode = 0

            def communicate(self, timeout=None):
                return (
                    json.dumps({
                        "structuredOutput": {
                            "verdict": "REJECT",
                            "grounds": "sandbox probe completed",
                        },
                        "text": "{\"verdict\":\"REJECT\",\"grounds\":\"sandbox probe completed\"}",
                    }),
                    "",
                )

        return _ProductionProcess()

    cwd = root / "cwd"
    cwd.mkdir(parents=True, exist_ok=True)
    outside_marker.parent.mkdir(parents=True, exist_ok=True)
    if not outside_marker.exists():
        outside_marker.write_text("CANARY_CONTENT", encoding="utf-8")
    production_result = grok_adapter.run_preland_judge_turn(
        "probe production spec", "probe production diff", "probe production gates",
        model="posthog-gpt56-terra",
        popen_factory=_production_popen,
    )
    if not production_result.get("ok"):
        raise AssertionError(f"production judge capture failed: {production_result}")
    captured["production_result"] = production_result
    captured["command"] = captured["derived_probe_argv"]
    captured["cwd"] = captured["derived_probe_cwd"]
    captured["profile"] = captured["derived_probe_profile"]
    captured["stdout"] = captured["derived_probe_stdout"]
    captured["stderr"] = captured["derived_probe_stderr"]
    captured["returncode"] = captured["derived_probe_returncode"]
    return captured


class TestFreshPrelandVerify(unittest.TestCase):
    def test_approve_proceeds(self):
        verdict, detail = pv.fresh_preland_verify(
            "spec", "diff", "[]", spawn_fn=_spawn(_structured("APPROVE", "matches"))
        )
        self.assertEqual(verdict, "approve")
        self.assertEqual(detail["grounds"], "matches")

    def test_reject_maps_to_rejected(self):
        verdict, detail = pv.fresh_preland_verify(
            "spec", "diff", "[]", spawn_fn=_spawn(_structured("REJECT", "incomplete"))
        )
        self.assertEqual(verdict, "reject")
        self.assertEqual(detail["grounds"], "incomplete")

    def test_spawn_exception_is_unavailable_failclosed(self):
        verdict, detail = pv.fresh_preland_verify(
            "spec", "diff", "[]", spawn_fn=_spawn(exc=RuntimeError("boom"))
        )
        self.assertEqual(verdict, "unavailable")
        self.assertIn("spawn_exception", detail["error"])

    def test_transport_failure_is_unavailable(self):
        result = {"ok": False, "text": "", "rc": None, "error": "timeout after 600s"}
        verdict, detail = pv.fresh_preland_verify("spec", "diff", "[]", spawn_fn=_spawn(result))
        self.assertEqual(verdict, "unavailable")
        self.assertEqual(detail["error"], "timeout after 600s")

    def test_unparseable_answer_is_unavailable(self):
        result = {"ok": True, "text": "I am not sure", "rc": 0, "error": None}
        verdict, _ = pv.fresh_preland_verify("spec", "diff", "[]", spawn_fn=_spawn(result))
        self.assertEqual(verdict, "unavailable")

    def test_env_model_override_respected(self):
        record = {}
        previous = os.environ.get(pv.VERIFIER_MODEL_ENV)
        os.environ[pv.VERIFIER_MODEL_ENV] = "override-model"
        try:
            pv.fresh_preland_verify(
                "spec", "diff", "[]", spawn_fn=_spawn(_structured(), record=record)
            )
        finally:
            if previous is None:
                os.environ.pop(pv.VERIFIER_MODEL_ENV, None)
            else:
                os.environ[pv.VERIFIER_MODEL_ENV] = previous
        self.assertEqual(record["model"], "override-model")

    def test_explicit_model_arg_beats_env(self):
        record = {}
        previous = os.environ.get(pv.VERIFIER_MODEL_ENV)
        os.environ[pv.VERIFIER_MODEL_ENV] = "env-model"
        try:
            pv.fresh_preland_verify(
                "spec", "diff", "[]", model="explicit-model",
                spawn_fn=_spawn(_structured(), record=record),
            )
        finally:
            if previous is None:
                os.environ.pop(pv.VERIFIER_MODEL_ENV, None)
            else:
                os.environ[pv.VERIFIER_MODEL_ENV] = previous
        self.assertEqual(record["model"], "explicit-model")

    def test_spec_absent_noted_in_prompt(self):
        record = {}
        verdict, detail = pv.fresh_preland_verify(
            None, "diff", "[]", spawn_fn=_spawn(_structured(), record=record)
        )
        self.assertEqual(verdict, "approve")
        self.assertFalse(detail["spec_present"])
        self.assertIn("spec:absent", record["prompt"])


class TestU2AdversarialContracts(unittest.TestCase):
    def test_garbled_verdict_never_approves(self):
        result = {"ok": True, "text": "APPROVE, probably", "rc": 0, "error": None}
        verdict, _ = pv.fresh_preland_verify("spec", "diff", "gates", spawn_fn=_spawn(result))
        self.assertNotEqual(verdict, "approve")
        self.assertEqual(verdict, "unavailable")

    def test_transport_failure_parks_fail_closed(self):
        calls = []

        def judge(*args, **kwargs):
            calls.append((args, kwargs))
            return {"ok": False, "rc": 1, "text": "", "error": "transport down"}

        verdict, detail = pv.run_verifier_with_retry(
            "spec", "diff", "gates", model="posthog-gpt56-terra", judge_fn=judge
        )
        self.assertEqual(verdict, "unavailable")
        self.assertEqual(len(calls), 2)
        with tempfile.TemporaryDirectory() as td:
            park = ol.park_verifier_unavailable(
                "U2-TRANSPORT", "/tmp/u2-worktree", detail=detail,
                attempt=detail["attempt_count"], pending_path=Path(td) / "pending.json",
                plan_status_dir=Path(td) / "status", emit_fn=lambda *_: None,
            )
        self.assertTrue(park["park"])
        self.assertEqual(park["waiting_status"], "WAITING_FOR_VERIFIER")

    def test_run_verifier_with_retry_bound(self):
        calls = []

        class Process:
            returncode = 1

            def communicate(self, timeout=None):
                return "", "persistent transport failure"

        def popen(command, **kwargs):
            calls.append((command, kwargs))
            return Process()

        verdict, detail = pv.run_verifier_with_retry(
            "spec", "diff", "gates", model="posthog-gpt56-terra", popen_factory=popen
        )
        self.assertEqual(verdict, "unavailable")
        self.assertEqual(detail["attempt_count"], 2)
        self.assertEqual(len(calls), 2)

    def test_judge_prompt_excludes_builder_history(self):
        record = {}
        marker = "BUILDER_HISTORY_MARKER"
        verdict, _ = pv.fresh_preland_verify(
            "plan only", "diff only", "gate only",
            spawn_fn=_spawn(_structured("REJECT", "review"), record=record),
        )
        self.assertEqual(verdict, "reject")
        self.assertNotIn(marker, record["prompt"])
        self.assertNotIn("builder", record["prompt"].lower())

    def _fake_judge_process(self, captured, output=None):
        output = output or {
            "structuredOutput": {"verdict": "REJECT", "grounds": "fixture"},
            "text": '{"verdict":"REJECT","grounds":"fixture"}',
        }

        class Process:
            returncode = 0

            def communicate(self, timeout=None):
                return json.dumps(output), ""

        def popen(command, **kwargs):
            captured["command"] = list(command)
            captured["cwd"] = kwargs["cwd"]
            captured["env"] = dict(kwargs["env"])
            agent_path = Path(command[command.index("--agent") + 1])
            captured["definition"] = agent_path.read_text(encoding="utf-8")
            return Process()

        return popen

    def test_judge_agent_definition_contract(self):
        captured = {}
        result = grok_adapter.run_preland_judge_turn(
            "spec", "diff", "gates", popen_factory=self._fake_judge_process(captured)
        )
        self.assertTrue(result["ok"])
        definition = captured["definition"]
        for line in (
            "promptMode: full", "injectDefaultTools: false", "agentsMd: false",
            "discoverSkills: false", "inheritSkills: false", "mcpInheritance: none",
            "permissionMode: dontAsk", "tools: []",
        ):
            self.assertIn(line, definition)
        turn_cap_field = "max" + "Turns"
        self.assertNotIn(turn_cap_field, definition)
        self.assertIn("toolConfig:\n  tools: []", definition)
        self.assertNotIn("read_file", definition)

    def test_judge_argv_isolation_contract(self):
        captured = {}
        grok_adapter.run_preland_judge_turn(
            "spec", "diff", "gates", popen_factory=self._fake_judge_process(captured)
        )
        command = captured["command"]
        required = (
            ("--no-memory", None), ("--no-subagents", None),
            ("--disable-web-search", None), ("--sandbox", "strict"),
            ("--cwd", captured["cwd"]), ("--json-schema", None),
            ("--system-prompt-override", None),
        )
        for flag, value in required:
            self.assertIn(flag, command)
            if value is not None:
                self.assertEqual(command[command.index(flag) + 1], value)
        turn_cap_flag = "--" + "max-" + "turns"
        turn_cap_field = "max" + "Turns"
        self.assertNotIn(turn_cap_flag, command)
        self.assertNotIn(turn_cap_field, command)
        self.assertNotIn("--resume", command)
        self.assertNotIn("--continue", command)

    def test_judge_preserves_outer_bwrap_marker_and_strict_argv(self):
        captured = {}
        with patch.dict(os.environ, {"__GROK_INSIDE_BWRAP": "1"}, clear=False):
            grok_adapter.run_preland_judge_turn(
                "spec", "diff", "gates", popen_factory=self._fake_judge_process(captured)
            )
        self.assertEqual(captured["env"].get("__GROK_INSIDE_BWRAP"), "1")
        command = captured["command"]
        self.assertEqual(command[command.index("--sandbox") + 1], "strict")

    def test_judge_does_not_synthesize_outer_bwrap_marker(self):
        captured = {}
        with patch.dict(os.environ, {}, clear=False):
            os.environ.pop("__GROK_INSIDE_BWRAP", None)
            grok_adapter.run_preland_judge_turn(
                "spec", "diff", "gates", popen_factory=self._fake_judge_process(captured)
            )
        self.assertNotIn("__GROK_INSIDE_BWRAP", captured["env"])

    def test_judge_strips_noncanonical_allowlisted_marker(self):
        for parent_marker in (None, "0", "01"):
            with self.subTest(parent_marker=parent_marker):
                captured = {}
                with patch.dict(os.environ, {}, clear=False), patch.object(
                    grok_adapter.grok_provider,
                    "build_grok_env",
                    return_value={"__GROK_INSIDE_BWRAP": "0"},
                ):
                    if parent_marker is None:
                        os.environ.pop("__GROK_INSIDE_BWRAP", None)
                    else:
                        os.environ["__GROK_INSIDE_BWRAP"] = parent_marker
                    grok_adapter.run_preland_judge_turn(
                        "spec", "diff", "gates",
                        popen_factory=self._fake_judge_process(captured),
                    )
                self.assertNotIn("__GROK_INSIDE_BWRAP", captured["env"])

    def test_judge_zero_internal_retries(self):
        captured = []

        class Process:
            returncode = 1

            def communicate(self, timeout=None):
                return "", "one failure"

        def popen(command, **kwargs):
            captured.append(command)
            return Process()

        result = grok_adapter.run_preland_judge_turn(
            "spec", "diff", "gates", popen_factory=popen
        )
        self.assertFalse(result["ok"])
        self.assertEqual(len(captured), 1)

    def test_sandbox_probe_denies_external_read(self):
        with tempfile.TemporaryDirectory(dir="/home/researcher/teikoku/Ougi") as td:
            root = Path(td)
            outside = root / "outside" / "marker.txt"
            outside.parent.mkdir()
            outside.write_text("CANARY_CONTENT", encoding="utf-8")
            result = run_sandbox_probe(root, outside)
        output = result["stdout"] + result["stderr"] + result["derived_probe_trace"]
        self.assertIn("read_file", result["profile"])
        turn_cap_flag = "--" + "max-" + "turns"
        turn_cap_field = "max" + "Turns"
        self.assertNotIn(turn_cap_flag, result["derived_probe_argv"])
        self.assertNotIn(turn_cap_field, result["derived_probe_argv"])
        self.assertIn('"tool_started","tool_name":"read_file"', result["derived_probe_trace"])
        self.assertIn("Permission denied:", result["derived_probe_trace"])
        self.assertNotIn("CANARY_CONTENT", output)


if __name__ == "__main__":
    unittest.main()
