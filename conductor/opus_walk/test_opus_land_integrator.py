"""A-CTRL-OPUS-LAND: opus_land production land uses SerializedIntegrator single-flight.

No direct git merge --ff-only in opus_land. IntegrationBusy surfaces. Post-merge red
gate reverts via INTEGRATION_POST_MERGE_REVERTED when land goes through the front door.
"""
from __future__ import annotations

import ast
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import opus_land as ol

CONDUCTOR = Path(__file__).resolve().parents[1]
if str(CONDUCTOR) not in sys.path:
    sys.path.insert(0, str(CONDUCTOR))

from conductor_db import ControlDB
from integrator import IntegrationBusy, IntegrationResult


def _git(repo: Path, args: list[str]) -> str:
    r = subprocess.run(
        ["git", "-C", str(repo), *args],
        check=True,
        capture_output=True,
        text=True,
    )
    return (r.stdout or "").strip()


def _make_elai_repo(path: Path) -> Path:
    path.mkdir(parents=True)
    elai = path / ".elai_cc"
    elai.mkdir(parents=True)
    (elai / "marker.txt").write_text("seed\n", encoding="utf-8")
    subprocess.run(["git", "init"], cwd=path, check=True, capture_output=True, text=True)
    subprocess.run(
        ["git", "config", "user.email", "opus-land-test@example.invalid"],
        cwd=path, check=True, capture_output=True,
    )
    subprocess.run(
        ["git", "config", "user.name", "Opus Land Test"],
        cwd=path, check=True, capture_output=True,
    )
    subprocess.run(["git", "add", "."], cwd=path, check=True, capture_output=True)
    subprocess.run(
        ["git", "commit", "-m", "seed"],
        cwd=path, check=True, capture_output=True, text=True,
    )
    return path


def _passing_gate(**kwargs):
    return {
        "overall_verdict": "PASS",
        "structured_feedback": "test pass",
        "task_id": kwargs.get("task_id"),
    }


class TestOpusLandIntegratorAbsorb(unittest.TestCase):
    def test_static_opus_land_has_no_ff_only_merge_call(self):
        src = Path(ol.__file__).read_text(encoding="utf-8")
        tree = ast.parse(src)
        for node in ast.walk(tree):
            if not isinstance(node, (ast.List, ast.Tuple)):
                continue
            elts = []
            for e in node.elts:
                if isinstance(e, ast.Constant) and isinstance(e.value, str):
                    elts.append(e.value)
            if "merge" in elts and "--ff-only" in elts:
                self.fail("opus_land still contains a git merge --ff-only argv list")
        self.assertNotIn('["git", "merge", "--ff-only"', src.replace(" ", ""))
        self.assertIn("land_via_serialized_integrator", src)
        self.assertIn("ensure_verified_task_for_land", src)

    def test_ensure_verified_task_for_land_upserts_verified(self):
        with tempfile.TemporaryDirectory(prefix="opus-land-task-") as td:
            td_path = Path(td)
            db_path = td_path / "control.db"
            ControlDB(db_path).init(reset=True)
            tid = ol.ensure_verified_task_for_land(
                plan_id="PLAN-X",
                task_id=None,
                base_commit="abc123",
                allowed_write_set=[".elai_cc/a.rs"],
                db_path=db_path,
            )
            self.assertEqual(tid, "opus-land-PLAN-X")
            row = ControlDB(db_path).get_task_row(tid)
            self.assertIsNotNone(row)
            self.assertEqual(row["status"], "VERIFIED")
            # re-call after DONE-like status still yields VERIFIED
            ControlDB(db_path).update_task_status(tid, "DONE", "test")
            tid2 = ol.ensure_verified_task_for_land(
                plan_id="PLAN-X",
                task_id=None,
                base_commit="abc123",
                allowed_write_set=[".elai_cc/a.rs"],
                db_path=db_path,
            )
            self.assertEqual(tid2, tid)
            self.assertEqual(ControlDB(db_path).get_task_row(tid)["status"], "VERIFIED")

    def test_land_via_serialized_integrator_promotes_on_dual_pass(self):
        with tempfile.TemporaryDirectory(prefix="opus-land-int-") as td:
            td_path = Path(td)
            db_path = td_path / "control.db"
            ControlDB(db_path).init(reset=True)
            repo = _make_elai_repo(td_path / "repo")
            base = _git(repo, ["rev-parse", "HEAD"])
            source = td_path / "source"
            subprocess.run(
                ["git", "-C", str(repo), "worktree", "add", "-b", "src-branch", str(source), base],
                check=True, capture_output=True, text=True,
            )
            (source / ".elai_cc" / "marker.txt").write_text("landed\n", encoding="utf-8")
            tid = ol.ensure_verified_task_for_land(
                plan_id="PLAN-LAND",
                task_id="task-land-1",
                base_commit=base,
                allowed_write_set=[".elai_cc/marker.txt"],
                db_path=db_path,
            )
            result = ol.land_via_serialized_integrator(
                task_id=tid,
                source_worktree=source,
                source_base_commit=base,
                commit_message="land marker via integrator",
                allow_push=False,
                db_path=db_path,
                repo_root=repo,
                workflow_root=td_path / "workflow",
                artifact_root=td_path / "artifacts",
                worktree_root=td_path / "worktrees",
                gate_runner=_passing_gate,
                packs=("elai",),
                block_dirty_checkout=False,
            )
            self.assertEqual(result.status, "DONE")
            self.assertEqual(ControlDB(db_path).get_task_row(tid)["status"], "DONE")
            self.assertEqual(
                (repo / ".elai_cc" / "marker.txt").read_text(encoding="utf-8"),
                "landed\n",
            )
            active = ControlDB(db_path).connect().execute(
                "SELECT COUNT(*) AS n FROM integrations WHERE finished_at IS NULL "
                "AND status IN ('PENDING','APPLIED')"
            ).fetchone()["n"]
            self.assertEqual(active, 0)

    def test_land_via_integration_busy_raises(self):
        with tempfile.TemporaryDirectory(prefix="opus-land-busy-") as td:
            td_path = Path(td)
            db_path = td_path / "control.db"
            db = ControlDB(db_path)
            db.init(reset=True)
            tid = ol.ensure_verified_task_for_land(
                plan_id="PLAN-BUSY",
                task_id="task-busy",
                base_commit="HEAD",
                allowed_write_set=[".elai_cc/x"],
                db_path=db_path,
            )
            with db.transaction() as conn:
                conn.execute(
                    """
                    INSERT INTO integrations (
                      integration_id, task_id, source_worktree_path, source_base_commit,
                      integration_worktree_path, status, started_at
                    )
                    VALUES ('busy-1', ?, 'source', 'HEAD', 'integration', 'PENDING', 'now')
                    """,
                    (tid,),
                )
            with self.assertRaises(IntegrationBusy):
                ol.land_via_serialized_integrator(
                    task_id=tid,
                    source_worktree=td_path,
                    source_base_commit="HEAD",
                    commit_message="should not land",
                    db_path=db_path,
                    repo_root=td_path,
                    workflow_root=td_path,
                    artifact_root=td_path / "artifacts",
                    worktree_root=td_path / "worktrees",
                    gate_runner=_passing_gate,
                    packs=("generic",),
                    block_dirty_checkout=False,
                )

    def test_post_merge_fail_reverts_via_integrator_front_door(self):
        with tempfile.TemporaryDirectory(prefix="opus-land-revert-") as td:
            td_path = Path(td)
            db_path = td_path / "control.db"
            ControlDB(db_path).init(reset=True)
            repo = _make_elai_repo(td_path / "repo")
            base = _git(repo, ["rev-parse", "HEAD"])
            source = td_path / "source-rev"
            subprocess.run(
                ["git", "-C", str(repo), "worktree", "add", "-b", "src-rev", str(source), base],
                check=True, capture_output=True, text=True,
            )
            (source / ".elai_cc" / "marker.txt").write_text("bad-land\n", encoding="utf-8")
            tid = ol.ensure_verified_task_for_land(
                plan_id="PLAN-REV",
                task_id="task-rev",
                base_commit=base,
                allowed_write_set=[".elai_cc/marker.txt"],
                db_path=db_path,
            )
            calls = {"n": 0}

            def gate_runner(**kwargs):
                calls["n"] += 1
                # candidate (worktree != repo) PASS; post-merge (worktree == repo) FAIL
                if Path(kwargs["worktree_path"]).resolve() == repo.resolve():
                    return {
                        "overall_verdict": "FAIL",
                        "structured_feedback": "post-merge red",
                        "task_id": kwargs.get("task_id"),
                    }
                return _passing_gate(**kwargs)

            result = ol.land_via_serialized_integrator(
                task_id=tid,
                source_worktree=source,
                source_base_commit=base,
                commit_message="should revert",
                allow_push=False,
                db_path=db_path,
                repo_root=repo,
                workflow_root=td_path / "workflow",
                artifact_root=td_path / "artifacts",
                worktree_root=td_path / "worktrees",
                gate_runner=gate_runner,
                packs=("elai",),
                block_dirty_checkout=False,
            )
            self.assertEqual(result.status, "POST_GATE_FAILED")
            self.assertEqual(result.reason, "post_merge_gate_failed")
            self.assertEqual(_git(repo, ["rev-parse", "HEAD"]), base)
            self.assertEqual(
                (repo / ".elai_cc" / "marker.txt").read_text(encoding="utf-8"),
                "seed\n",
            )
            conn = ControlDB(db_path).connect()
            try:
                n = conn.execute(
                    "SELECT COUNT(*) AS n FROM events "
                    "WHERE task_id=? AND event_type='INTEGRATION_POST_MERGE_REVERTED'",
                    (tid,),
                ).fetchone()["n"]
            finally:
                conn.close()
            self.assertGreaterEqual(n, 1)
            self.assertGreaterEqual(calls["n"], 2)

    def test_main_land_path_calls_serialized_integrator_not_ff_merge(self):
        """Production hook opus_land.main wires land_via_serialized_integrator (R7/R8)."""
        fake = IntegrationResult(
            integration_id="int-1",
            task_id="task-main",
            status="DONE",
            task_status="DONE",
            integration_worktree_path="/tmp/i",
            patch_artifact="/tmp/p.patch",
            post_gate_verdict="PASS",
            reason="post_merge_gate_passed",
        )
        with tempfile.TemporaryDirectory(prefix="opus-land-main-") as td:
            td_path = Path(td)
            repo = _make_elai_repo(td_path / "repo")
            base = _git(repo, ["rev-parse", "HEAD"])
            wt = td_path / "wt"
            subprocess.run(
                ["git", "-C", str(repo), "worktree", "add", "-b", "wt-main", str(wt), base],
                check=True, capture_output=True, text=True,
            )
            (wt / ".elai_cc" / "marker.txt").write_text("via-main\n", encoding="utf-8")
            files = ".elai_cc/marker.txt"
            calls = []

            def _fake_land(**kwargs):
                calls.append(kwargs)
                return fake

            ControlDB(td_path / "control.db").init(reset=True)
            tools = td_path / "tools"
            tools.mkdir(parents=True)
            for name in ol.CHECKERS:
                (tools / name).write_text("# stub\n", encoding="utf-8")

            real_run_fn = ol._run
            merge_hits = []

            def _run_guard(cmd, cwd, timeout=None, env=None):
                if isinstance(cmd, (list, tuple)) and len(cmd) >= 2 and cmd[0] == "git":
                    if "merge" in cmd and "--ff-only" in cmd:
                        merge_hits.append(list(cmd))
                        raise AssertionError(f"direct ff-merge: {cmd}")
                if isinstance(cmd, (list, tuple)) and cmd[:1] == ["python3"]:
                    class R:
                        returncode = 0
                        stdout = ""
                        stderr = ""
                    return R()
                return real_run_fn(cmd, cwd, timeout=timeout or 60, env=env)

            import standing_goals as sg
            import trust_ledger as tl
            with mock.patch.object(ol, "ELAI_ROOT", repo), \
                 mock.patch.object(ol, "DB_PATH", td_path / "control.db"), \
                 mock.patch.object(ol, "CANON_TOOLS", tools), \
                 mock.patch.object(ol, "LAND_TRIGGER", td_path / "land-trigger"), \
                 mock.patch.object(ol, "land_via_serialized_integrator", side_effect=_fake_land), \
                 mock.patch.object(ol, "ensure_verified_task_for_land", return_value="task-main"), \
                 mock.patch.object(ol, "integration_push_failed", return_value=False), \
                 mock.patch.object(ol, "_emit", lambda *a, **k: None), \
                 mock.patch.object(ol, "_run", side_effect=_run_guard), \
                 mock.patch.object(
                     ol,
                     "parse_allow_no_verifier",
                     return_value=(True, "granted", {"timestamp": "t", "scope": {"plan_id": "P1"}}),
                 ), \
                 mock.patch.object(sg, "enroll", lambda *a, **k: None), \
                 mock.patch.object(tl, "record", return_value="T0"):
                code = ol.main([
                    "--worktree", str(wt),
                    "--plan-id", "P1",
                    "--files", files,
                    "--message", "land via integrator hook",
                    "--no-verifier",
                    "--json",
                ])
            self.assertEqual(code, 0, msg=f"merge_hits={merge_hits} calls={calls}")
            self.assertEqual(len(calls), 1)
            self.assertEqual(calls[0]["task_id"], "task-main")
            self.assertEqual(calls[0]["source_base_commit"], base)
            self.assertEqual(Path(calls[0]["source_worktree"]), wt.resolve())
            self.assertEqual(merge_hits, [])

if __name__ == "__main__":
    unittest.main()
