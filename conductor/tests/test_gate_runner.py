from __future__ import annotations

import json
import subprocess
import sys
import unittest
from dataclasses import replace
from pathlib import Path

from conductor_db import ControlDB
from gate_runner import build_gate_commands, run_gates
from tests.helpers import TempDirMixin
from tests.test_worker_adapter import make_git_repo, scoped_task


class GateRunnerTests(TempDirMixin, unittest.TestCase):
    def test_build_gate_commands_never_uses_gate_lock_refresh(self) -> None:
        temp = self.make_tempdir()
        repo = make_git_repo(temp / "repo")
        (repo / "src" / "lib.rs").write_text("pub fn changed() {}\n", encoding="utf-8")
        workflow = make_fake_workflow(temp / "workflow")
        db_path = temp / "control.db"
        db = ControlDB(db_path)
        db.init(reset=True)
        task = scoped_task("GATE-1")
        db.upsert_task(task, status="READY")
        result = run_gates(
            task.task_id,
            str(repo),
            "HEAD",
            str(db_path),
            workflow_root=str(workflow),
            repo_root=str(repo),
            artifact_root=str(temp / "artifacts"),
        )
        conn = db.connect()
        try:
            rows = conn.execute("SELECT command_json FROM gate_runs").fetchall()
        finally:
            conn.close()
        commands = [json.loads(row["command_json"]) for row in rows]
        flattened = [part for command in commands for part in command]
        self.assertNotIn("--lock", flattened)
        self.assertEqual(result["overall_verdict"], "PASS")

    def test_allowed_write_violation_fails_closed_and_records_gate_run(self) -> None:
        temp = self.make_tempdir()
        repo = make_git_repo(temp / "repo")
        (repo / "outside.txt").write_text("changed outside\n", encoding="utf-8")
        workflow = make_fake_workflow(temp / "workflow")
        db_path = temp / "control.db"
        db = ControlDB(db_path)
        db.init(reset=True)
        task = scoped_task("GATE-2")
        db.upsert_task(task, status="READY")
        result = run_gates(
            task.task_id,
            str(repo),
            "HEAD",
            str(db_path),
            workflow_root=str(workflow),
            repo_root=str(repo),
            artifact_root=str(temp / "artifacts"),
        )
        self.assertEqual(result["overall_verdict"], "FAIL")
        self.assertEqual(result["failure_category"], "allowed_write_violation")
        self.assertIn("outside.txt", result["structured_feedback"])
        conn = db.connect()
        try:
            row = conn.execute(
                "SELECT verdict FROM gate_runs WHERE gate_name = 'allowed_write_enforcement'"
            ).fetchone()
        finally:
            conn.close()
        self.assertEqual(row["verdict"], "FAIL")

    def test_gate_rerun_upserts_verdict_and_records_stdout_artifact(self) -> None:
        temp = self.make_tempdir()
        repo = make_git_repo(temp / "repo")
        (repo / "src" / "lib.rs").write_text("pub fn changed() {}\n", encoding="utf-8")
        workflow = make_fake_workflow(temp / "workflow")
        db_path = temp / "control.db"
        db = ControlDB(db_path)
        db.init(reset=True)
        task = scoped_task("GATE-RERUN")
        db.upsert_task(task, status="READY")

        for _ in range(2):
            result = run_gates(
                task.task_id,
                str(repo),
                "HEAD",
                str(db_path),
                workflow_root=str(workflow),
                repo_root=str(repo),
                artifact_root=str(temp / "artifacts"),
            )
            self.assertEqual(result["overall_verdict"], "PASS")

        conn = db.connect()
        try:
            verdict_rows = conn.execute(
                "SELECT COUNT(*) AS n FROM verdicts WHERE task_id = ? AND gate_name = 'gate_freeze_check'",
                (task.task_id,),
            ).fetchone()
            gate_rows = conn.execute(
                "SELECT stdout_artifact FROM gate_runs WHERE task_id = ? ORDER BY started_at",
                (task.task_id,),
            ).fetchall()
        finally:
            conn.close()
        self.assertEqual(verdict_rows["n"], 1)
        self.assertTrue(gate_rows)
        self.assertTrue(all(row["stdout_artifact"] for row in gate_rows))


def make_fake_workflow(path: Path) -> Path:
    tools = path / ".claude" / "tools"
    tools.mkdir(parents=True)
    (path / ".claude" / "gate-manifest.txt").write_text("fake\n", encoding="utf-8")
    (path / ".claude" / "gate-manifest.lock").write_text("fake\tsha3_256\tabc\n", encoding="utf-8")
    (tools / "elai.map").write_text("", encoding="utf-8")
    scripts = {
        "gate_freeze_check.py": "import json; print(json.dumps({'status':'OK'})); raise SystemExit(0)\n",
        "production_caller_check.py": "import json; print(json.dumps({'verdict':'PASS'})); raise SystemExit(0)\n",
        "python_wire_check.py": "import json; print(json.dumps({'verdict':'PASS'})); raise SystemExit(0)\n",
        "maintainability_check.py": "import json; print(json.dumps({'clean': True})); raise SystemExit(0)\n",
        "traced_test_assert_check.py": "import json; print(json.dumps({'violations': []})); raise SystemExit(0)\n",
        "wiring_contract_check.py": "import json; print(json.dumps({'verdict':'PASS'})); raise SystemExit(0)\n",
        "spec_link_check.py": "import json; print(json.dumps({'results': []})); raise SystemExit(0)\n",
    }
    for name, body in scripts.items():
        script = tools / name
        script.write_text(body, encoding="utf-8")
        script.chmod(0o755)
    docs = path / "docs"
    docs.mkdir()
    (docs / "T-1.md").write_text("## WIRING_CONTRACT\nproducer: x\n", encoding="utf-8")
    return path


if __name__ == "__main__":
    unittest.main()
