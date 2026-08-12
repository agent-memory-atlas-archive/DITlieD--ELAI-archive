from __future__ import annotations

import contextlib
import copy
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
ELAI_CC = REPO_ROOT / ".elai_cc"
if str(ELAI_CC) not in sys.path:
    sys.path.insert(0, str(ELAI_CC))

from scripts.trustbench import (  # noqa: E402
    DEFAULT_MANIFEST,
    TrustbenchError,
    build_parser,
    command_run,
    load_manifest,
    normalize_artifact,
    parse_adapter_commands,
    path_inside,
    redact_command,
    relative_lexical_project_path,
)


class TrustbenchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.manifest = load_manifest(DEFAULT_MANIFEST)

    def test_manifest_contains_requested_suite_states(self):
        suites = {suite["id"]: suite for suite in self.manifest["suites"]}
        self.assertEqual(
            set(suites),
            {"harnessaudit", "control-arena", "agent-safetybench", "openbench", "underspecbench"},
        )
        self.assertEqual(suites["underspecbench"]["status"], "paper_only")
        for suite_id in set(suites) - {"underspecbench"}:
            self.assertEqual(len(suites[suite_id]["ref"]), 40)

    def test_path_inside_rejects_escape(self):
        with self.assertRaises(TrustbenchError):
            path_inside(ELAI_CC.parent / "outside", ELAI_CC)

    def test_lexical_path_reports_virtualenv_symlink_location(self):
        path = ELAI_CC / "benchmarks" / "harness_trust" / "venvs" / "suite" / "bin" / "python"
        self.assertEqual(
            relative_lexical_project_path(path),
            "benchmarks/harness_trust/venvs/suite/bin/python",
        )

    def test_adapter_command_parsing_and_redaction(self):
        commands = parse_adapter_commands(
            ["openbench=python -m adapter --model {model} --output {output}"]
        )
        self.assertEqual(
            commands["openbench"],
            ["python", "-m", "adapter", "--model", "{model}", "--output", "{output}"],
        )
        self.assertEqual(
            redact_command(["tool", "--api-key", "sensitive", "OPENAI_TOKEN=secret"]),
            ["tool", "--api-key", "<redacted>", "OPENAI_TOKEN=<redacted>"],
        )
        with self.assertRaises(TrustbenchError):
            parse_adapter_commands(["missing-separator"])

    def test_normalize_artifact_requires_json_objects(self):
        with tempfile.TemporaryDirectory(dir=ELAI_CC) as temp_dir:
            artifact = Path(temp_dir) / "result.jsonl"
            artifact.write_text('{"status":"completed","score":0.5}\n', encoding="utf-8")
            rows = normalize_artifact(artifact, "openbench", {"seed": 7})
            self.assertEqual(rows[0]["suite"], "openbench")
            self.assertEqual(rows[0]["score"], 0.5)
            artifact.write_text("[]\n", encoding="utf-8")
            with self.assertRaises(TrustbenchError):
                normalize_artifact(artifact, "openbench", {"seed": 7})

    def test_dry_run_report_is_complete_and_non_scoring(self):
        with tempfile.TemporaryDirectory(dir=ELAI_CC) as temp_dir:
            temp_root = Path(temp_dir)
            manifest = copy.deepcopy(self.manifest)
            relative_root = temp_root.relative_to(ELAI_CC).as_posix()
            manifest["checkout_root"] = f"{relative_root}/upstreams"
            manifest["venv_root"] = f"{relative_root}/venvs"
            manifest["result_root"] = f"{relative_root}/results"
            manifest_path = temp_root / "manifest.json"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            args = build_parser().parse_args(
                [
                    "run",
                    "--manifest",
                    str(manifest_path),
                    "--harness",
                    "/usr/local/bin/future-harness",
                    "--model",
                    "fixed-model",
                    "--seed",
                    "7",
                    "--all",
                    "--adapter-command",
                    "harnessaudit=python -m adapter --model {model} --output {output}",
                    "--output-dir",
                    f"{relative_root}/results",
                    "--dry-run",
                ]
            )
            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                self.assertEqual(command_run(args, manifest), 0)
            report = json.loads(output.getvalue())
            self.assertEqual(len(report["suites"]), 5)
            self.assertEqual(report["suites"][0]["status"], "dry_run_ready")
            self.assertEqual(report["suites"][-1]["status"], "paper_only")
            self.assertFalse(report["aggregate"]["enabled"])
            self.assertIsNone(report["aggregate"]["score"])


if __name__ == "__main__":
    unittest.main()
