"""Unit tests for standing_goals.py (P1.7).

Temp-dir idiom; NO real subprocess and NO real walk-state are touched. The predicate runner is
injected, walk_state.seed_plan is mocked, and every path points into a temp dir. Framework is
unittest (pytest is broken repo-wide)."""
import json
import os
import shutil
import tempfile
import unittest
from unittest import mock

import standing_goals as sg
import walk_state as ws

WALK_SEED = {
    "schema_version": 1,
    "lock": None,
    "budget": {"dispatches_this_session": 0, "dispatch_cap": 30, "plans_this_session": 0,
               "plans_per_session_cap": 3, "sessions_completed": 0},
    "cursor": {"plan_id": None, "stage": None, "pickup": None},
    "queue": [],
    "escalation_queue": [],
    "decisions": [],
    "integration_smoke": {"last_after_plan": None, "cadence_plans": 3},
}


def pass_runner(predicate, timeout_s):
    return 0


def fail_runner(predicate, timeout_s):
    return 1


class CountingRunner:
    def __init__(self, rc=1):
        self.rc = rc
        self.calls = []

    def __call__(self, predicate, timeout_s):
        self.calls.append(predicate)
        return self.rc


class TempGoalsCase(unittest.TestCase):
    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="standinggoals_test_")
        self.goals_dir = os.path.join(self.tmpdir, "goals")
        self.walk_path = os.path.join(self.tmpdir, "walk-state.json")
        self.ledger_path = os.path.join(self.tmpdir, "goal-ledger.json")
        self._write_walk(WALK_SEED)

    def tearDown(self):
        shutil.rmtree(self.tmpdir, ignore_errors=True)

    def _write_walk(self, state):
        with open(self.walk_path, "w", encoding="utf-8") as fh:
            json.dump(state, fh)

    def _read_goal(self, plan_id):
        return sg._read_goal(sg._goal_path(self.goals_dir, plan_id))

    def _ledger_rows(self):
        if not os.path.isfile(self.ledger_path):
            return []
        with open(self.ledger_path, encoding="utf-8") as fh:
            return json.load(fh)["rows"]

    def _verify(self, runner):
        return sg.verify_all(goals_dir=self.goals_dir, runner=runner,
                             walk_path=self.walk_path, ledger_path=self.ledger_path)


class TestEnroll(TempGoalsCase):
    def test_enroll_writes_goal_file_with_predicates_and_timeout(self):
        preds = ["cargo test -p elai-orchestrator", "cargo check -p elai-orchestrator"]
        rec = sg.enroll("H-28", preds, meta={"commit_sha": "abc123"},
                        goals_dir=self.goals_dir, timeout_s=777)
        self.assertTrue(os.path.isfile(sg._goal_path(self.goals_dir, "H-28")))
        on_disk = self._read_goal("H-28")
        self.assertEqual(on_disk["predicates"], preds)
        self.assertEqual(on_disk["timeout_s"], 777)
        self.assertEqual(on_disk["status"], sg.STATUS_ACTIVE)
        self.assertEqual(on_disk["consecutive_failures"], 0)
        self.assertEqual(on_disk["commit_sha"], "abc123")
        self.assertEqual(rec["plan_id"], "H-28")

    def test_enroll_default_timeout(self):
        sg.enroll("H-1", ["cargo test"], goals_dir=self.goals_dir)
        self.assertEqual(self._read_goal("H-1")["timeout_s"], sg.DEFAULT_TIMEOUT_S)

    def test_enroll_empty_predicates_raises(self):
        with self.assertRaises(ValueError):
            sg.enroll("H-2", [], goals_dir=self.goals_dir)
        with self.assertRaises(ValueError):
            sg.enroll("H-2", ["  "], goals_dir=self.goals_dir)

    def test_reenroll_refreshes_and_resets(self):
        sg.enroll("H-3", ["old"], goals_dir=self.goals_dir)
        # simulate a prior violation on disk, then re-land
        rec = self._read_goal("H-3")
        rec["status"] = sg.STATUS_VIOLATED
        rec["consecutive_failures"] = 2
        sg._write_goal(sg._goal_path(self.goals_dir, "H-3"), rec)
        sg.enroll("H-3", ["new1", "new2"], goals_dir=self.goals_dir)
        fresh = self._read_goal("H-3")
        self.assertEqual(fresh["predicates"], ["new1", "new2"])
        self.assertEqual(fresh["status"], sg.STATUS_ACTIVE)
        self.assertEqual(fresh["consecutive_failures"], 0)


class TestVerifyPass(TempGoalsCase):
    def test_pass_stamps_last_pass_and_ledger_row(self):
        sg.enroll("H-4", ["cargo test"], goals_dir=self.goals_dir)
        with mock.patch.object(ws, "seed_plan") as seed:
            summary = self._verify(pass_runner)
        rec = self._read_goal("H-4")
        self.assertEqual(rec["status"], sg.STATUS_ACTIVE)
        self.assertIsNotNone(rec["last_pass"])
        self.assertEqual(rec["consecutive_failures"], 0)
        self.assertIn("H-4", summary["passed"])
        seed.assert_not_called()
        rows = self._ledger_rows()
        self.assertEqual(rows[-1]["result"], "pass")


class TestVerifyViolation(TempGoalsCase):
    def test_fail_flips_violated_ledger_row_and_reseeds(self):
        sg.enroll("H-5", ["cargo test"], goals_dir=self.goals_dir)
        with mock.patch.object(ws, "seed_plan") as seed:
            summary = self._verify(fail_runner)
        rec = self._read_goal("H-5")
        self.assertEqual(rec["status"], sg.STATUS_VIOLATED)
        self.assertEqual(rec["consecutive_failures"], 1)
        self.assertIn("H-5", summary["violated"])
        self.assertIn("H-5", summary["reseeded"])
        seed.assert_called_once()
        self.assertEqual(seed.call_args.args[0], "H-5")
        row = self._ledger_rows()[-1]
        self.assertEqual(row["result"], "violated")
        self.assertEqual(row["failed_predicate"], "cargo test")

    def test_reseed_skipped_when_already_queued(self):
        sg.enroll("H-6", ["cargo test"], goals_dir=self.goals_dir)
        self._write_walk({**WALK_SEED, "queue": ["H-6"]})
        with mock.patch.object(ws, "seed_plan") as seed:
            summary = self._verify(fail_runner)
        seed.assert_not_called()
        self.assertNotIn("H-6", summary["reseeded"])
        self.assertIn("H-6", summary["violated"])

    def test_reseed_skipped_when_escalated(self):
        sg.enroll("H-7", ["cargo test"], goals_dir=self.goals_dir)
        self._write_walk({**WALK_SEED, "escalation_queue": [{"plan_id": "H-7", "reason": "x"}]})
        with mock.patch.object(ws, "seed_plan") as seed:
            self._verify(fail_runner)
        seed.assert_not_called()

    def test_reseed_skipped_when_nondone_plan_status_row(self):
        sg.enroll("H-8", ["cargo test"], goals_dir=self.goals_dir)
        plans_dir = os.path.join(self.tmpdir, "plan-status")
        os.makedirs(plans_dir, exist_ok=True)
        with open(os.path.join(plans_dir, "H-8.json"), "w", encoding="utf-8") as fh:
            json.dump({"plan_id": "H-8", "status": "IN_PROGRESS"}, fh)
        with mock.patch.object(ws, "seed_plan") as seed:
            self._verify(fail_runner)
        seed.assert_not_called()

    def test_reseed_happens_when_done_plan_status_row(self):
        # the regression case: plan landed (DONE row), then its smoke broke -> re-enter the pipeline.
        sg.enroll("H-9", ["cargo test"], goals_dir=self.goals_dir)
        plans_dir = os.path.join(self.tmpdir, "plan-status")
        os.makedirs(plans_dir, exist_ok=True)
        with open(os.path.join(plans_dir, "H-9.json"), "w", encoding="utf-8") as fh:
            json.dump({"plan_id": "H-9", "status": "DONE"}, fh)
        with mock.patch.object(ws, "seed_plan") as seed:
            self._verify(fail_runner)
        seed.assert_called_once()

    def test_timeout_rc_counts_as_violation(self):
        sg.enroll("H-10", ["cargo test"], goals_dir=self.goals_dir)
        timeout_runner = lambda pred, t: sg.TIMEOUT_RC
        with mock.patch.object(ws, "seed_plan"):
            summary = self._verify(timeout_runner)
        self.assertIn("H-10", summary["violated"])
        self.assertEqual(self._read_goal("H-10")["status"], sg.STATUS_VIOLATED)


class TestQuarantine(TempGoalsCase):
    def test_three_consecutive_violations_quarantines_then_skips(self):
        sg.enroll("H-11", ["cargo test"], goals_dir=self.goals_dir)
        runner = CountingRunner(rc=1)
        with mock.patch.object(ws, "seed_plan"):
            self._verify(runner)   # cf=1 VIOLATED
            self._verify(runner)   # cf=2 VIOLATED
            summary3 = self._verify(runner)   # cf=3 -> QUARANTINED
        self.assertEqual(self._read_goal("H-11")["status"], sg.STATUS_QUARANTINED)
        self.assertIn("H-11", summary3["quarantined"])
        self.assertEqual(len(runner.calls), 3)   # ran on each of the first three sweeps
        # a QUARANTINED goal is SKIPPED, not run, on the next sweep
        with mock.patch.object(ws, "seed_plan") as seed:
            summary4 = self._verify(runner)
        self.assertEqual(len(runner.calls), 3)   # runner NOT invoked a fourth time
        self.assertIn("H-11", summary4["skipped"])
        self.assertNotIn("H-11", summary4["checked"])
        seed.assert_not_called()

    def test_quarantine_is_not_reseeded(self):
        sg.enroll("H-12", ["cargo test"], goals_dir=self.goals_dir)
        with mock.patch.object(ws, "seed_plan") as seed:
            self._verify(fail_runner)   # cf=1
            self._verify(fail_runner)   # cf=2
            seed.reset_mock()
            self._verify(fail_runner)   # cf=3 -> QUARANTINED, must NOT re-seed
        seed.assert_not_called()


class TestDefaultRunnerTimeout(TempGoalsCase):
    def test_default_runner_timeout_returns_timeout_rc(self):
        import subprocess
        with mock.patch("subprocess.run",
                        side_effect=subprocess.TimeoutExpired(cmd="x", timeout=1)):
            rc = sg._default_runner("sleep 999", timeout_s=1)
        self.assertEqual(rc, sg.TIMEOUT_RC)


class TestGoalSerialization(TempGoalsCase):
    def test_roundtrip(self):
        sg.enroll("H-13", ["a", "b"], meta={"commit_sha": "z"}, goals_dir=self.goals_dir)
        rec = self._read_goal("H-13")
        self.assertEqual(rec["predicates"], ["a", "b"])

    def test_unparsable_goal_is_skipped_not_fatal(self):
        os.makedirs(self.goals_dir, exist_ok=True)
        with open(sg._goal_path(self.goals_dir, "BADGOAL"), "w", encoding="utf-8") as fh:
            fh.write("# no json block here\n")
        sg.enroll("H-14", ["cargo test"], goals_dir=self.goals_dir)
        with mock.patch.object(ws, "seed_plan"):
            summary = self._verify(pass_runner)
        self.assertIn("H-14", summary["passed"])
        self.assertNotIn("BADGOAL", summary["checked"])


if __name__ == "__main__":
    unittest.main()
