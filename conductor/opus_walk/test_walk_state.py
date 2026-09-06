"""Unit tests for walk_state.py.

All tests operate on temp-directory copies of the state file; the real
walk-state.json checked into workflow/orchestrator-walk/ is never touched.
"""
import json
import os
import shutil
import tempfile
import unittest
from datetime import datetime, timedelta, timezone
from unittest import mock

import walk_state as ws

SEED = {
    "schema_version": 1,
    "walk_id": "opus-walk-2026-07-06",
    "lock": None,
    "budget": {
        "dispatches_this_session": 0,
        "dispatch_cap": 30,
        "plans_this_session": 0,
        "plans_per_session_cap": 3,
        "sessions_completed": 0,
        "concurrency": 3,
        "degrade_on_429_to": 2,
    },
    "cursor": {"plan_id": None, "stage": None, "pickup": None},
    "queue": [],
    "escalation_queue": [],
    "decisions": [],
    "integration_smoke": {"last_after_plan": None, "cadence_plans": 3},
    "notes": "test seed",
}


class TempStateCase(unittest.TestCase):
    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="walkstate_test_")
        self.path = os.path.join(self.tmpdir, "walk-state.json")
        self._write_seed()

    def tearDown(self):
        shutil.rmtree(self.tmpdir, ignore_errors=True)

    def _write_seed(self, overrides=None):
        state = json.loads(json.dumps(SEED))
        if overrides:
            state.update(overrides)
        with open(self.path, "w", encoding="utf-8") as f:
            json.dump(state, f)
        return state

    def _tmp_residue(self):
        return [n for n in os.listdir(self.tmpdir) if n.endswith(".tmp")]


class TestConcurrentClaim(TempStateCase):
    """Regression for the 2026-07-06 audit BLOCK: two concurrent claim_lock
    callers must never BOTH win (the read-decide-write section is serialized
    by the _StateMutex sidecar, not just the atomic os.replace)."""

    def test_two_concurrent_claims_exactly_one_wins(self):
        import threading

        TRIALS = 20
        for _ in range(TRIALS):
            self._write_seed()
            barrier = threading.Barrier(2)
            results = {}

            def _claim(sid):
                barrier.wait()
                try:
                    ws.claim_lock(sid, self.path)
                    results[sid] = "won"
                except ws.LockRefused:
                    results[sid] = "refused"

            threads = [
                threading.Thread(target=_claim, args=("session-a",)),
                threading.Thread(target=_claim, args=("session-b",)),
            ]
            for t in threads:
                t.start()
            for t in threads:
                t.join(timeout=30)

            outcomes = sorted(results.values())
            self.assertEqual(
                outcomes, ["refused", "won"],
                f"expected exactly one winner, got {results}",
            )
            holder = ws.load(self.path)["lock"]["session_id"]
            winner = next(k for k, v in results.items() if v == "won")
            self.assertEqual(holder, winner)
            self.assertEqual(self._tmp_residue(), [])


class TestSessionRollover(TempStateCase):
    def test_rollover_by_lock_holder_resets_window(self):
        ws.claim_lock("s-a", self.path)
        for _ in range(3):
            ws.record_dispatch(self.path)
        budget = ws.session_rollover("s-a", self.path)
        self.assertEqual(budget["sessions_completed"], 1)
        self.assertEqual(budget["dispatches_this_session"], 0)
        self.assertEqual(budget["plans_this_session"], 0)
        on_disk = ws.load(self.path)["budget"]
        self.assertEqual(on_disk["dispatches_this_session"], 0)
        self.assertEqual(on_disk["sessions_completed"], 1)

    def test_rollover_without_lock_refused(self):
        with self.assertRaises(ws.LockRefused):
            ws.session_rollover("s-a", self.path)

    def test_rollover_by_non_holder_refused(self):
        ws.claim_lock("s-a", self.path)
        with self.assertRaises(ws.LockRefused):
            ws.session_rollover("s-b", self.path)


class TestLoad(TempStateCase):
    def test_load_seed_passes(self):
        state = ws.load(self.path)
        self.assertEqual(state["schema_version"], 1)
        self.assertIn("lock", state)
        self.assertIn("budget", state)
        self.assertIn("cursor", state)
        self.assertIn("queue", state)
        self.assertIn("escalation_queue", state)
        self.assertIn("decisions", state)
        self.assertIn("integration_smoke", state)

    def test_load_missing_file_raises(self):
        missing = os.path.join(self.tmpdir, "does-not-exist.json")
        with self.assertRaises(ws.MalformedWalkState):
            ws.load(missing)

    def test_load_invalid_json_raises(self):
        with open(self.path, "w", encoding="utf-8") as f:
            f.write("{not valid json")
        with self.assertRaises(ws.MalformedWalkState):
            ws.load(self.path)

    def test_load_wrong_schema_version_raises(self):
        self._write_seed(overrides={"schema_version": 2})
        with self.assertRaises(ws.MalformedWalkState):
            ws.load(self.path)

    def test_load_missing_required_key_raises(self):
        state = json.loads(json.dumps(SEED))
        del state["budget"]
        with open(self.path, "w", encoding="utf-8") as f:
            json.dump(state, f)
        with self.assertRaises(ws.MalformedWalkState):
            ws.load(self.path)


class TestLock(TempStateCase):
    def test_claim_on_unlocked_succeeds(self):
        lock = ws.claim_lock("session-a", path=self.path)
        self.assertEqual(lock["session_id"], "session-a")
        self.assertIn("pid", lock)
        self.assertIn("heartbeat_iso", lock)
        state = ws.load(self.path)
        self.assertEqual(state["lock"]["session_id"], "session-a")

    def test_reclaim_same_session_refreshes_heartbeat(self):
        first = ws.claim_lock("session-a", path=self.path)
        second = ws.claim_lock("session-a", path=self.path)
        self.assertEqual(second["session_id"], "session-a")
        self.assertGreaterEqual(second["heartbeat_iso"], first["heartbeat_iso"])

    def test_claim_persists_controller_budget_overrides(self):
        env = {
            ws.DISPATCH_CAP_ENV: "10",
            ws.PLAN_CAP_ENV: "1",
        }
        with mock.patch.dict(os.environ, env, clear=False):
            ws.claim_lock("session-a", path=self.path)
        budget = ws.load(self.path)["budget"]
        self.assertEqual(budget["dispatch_cap"], 10)
        self.assertEqual(budget["plans_per_session_cap"], 1)

    def test_claim_records_injected_long_lived_controller_pid(self):
        with mock.patch.dict(
            os.environ,
            {ws.LOCK_OWNER_PID_ENV: str(os.getpid())},
            clear=False,
        ):
            lock = ws.claim_lock("session-a", path=self.path)
            heartbeat = ws.heartbeat("session-a", path=self.path)
        self.assertEqual(lock["pid"], os.getpid())
        self.assertEqual(heartbeat["pid"], os.getpid())

    def test_dead_injected_controller_pid_is_rejected(self):
        with mock.patch.dict(
            os.environ,
            {ws.LOCK_OWNER_PID_ENV: "2000000000"},
            clear=False,
        ):
            with self.assertRaises(ValueError):
                ws.claim_lock("session-a", path=self.path)
        self.assertIsNone(ws.load(self.path)["lock"])

    def test_larger_controller_restores_its_budget(self):
        with mock.patch.dict(
            os.environ,
            {ws.DISPATCH_CAP_ENV: "10", ws.PLAN_CAP_ENV: "1"},
            clear=False,
        ):
            ws.claim_lock("session-a", path=self.path)
        ws.release_lock("session-a", path=self.path)
        with mock.patch.dict(
            os.environ,
            {ws.DISPATCH_CAP_ENV: "30", ws.PLAN_CAP_ENV: "3"},
            clear=False,
        ):
            ws.claim_lock("session-b", path=self.path)
        budget = ws.load(self.path)["budget"]
        self.assertEqual(budget["dispatch_cap"], 30)
        self.assertEqual(budget["plans_per_session_cap"], 3)

    def test_partial_budget_override_is_rejected_without_mutation(self):
        with mock.patch.dict(
            os.environ,
            {ws.DISPATCH_CAP_ENV: "10"},
            clear=True,
        ):
            with self.assertRaises(ValueError):
                ws.claim_lock("session-a", path=self.path)
        state = ws.load(self.path)
        self.assertIsNone(state["lock"])
        self.assertEqual(state["budget"]["dispatch_cap"], 30)
        self.assertEqual(state["budget"]["plans_per_session_cap"], 3)

    def test_claim_by_other_session_fresh_heartbeat_refused(self):
        ws.claim_lock("session-a", path=self.path)
        with self.assertRaises(ws.LockRefused):
            ws.claim_lock("session-b", path=self.path)

    def test_claim_by_other_session_stale_heartbeat_steals(self):
        ws.claim_lock("session-a", path=self.path)
        state = ws.load(self.path)
        stale = datetime.now(timezone.utc) - timedelta(minutes=ws.HEARTBEAT_STALE_MIN + 1)
        state["lock"]["heartbeat_iso"] = stale.isoformat()
        with open(self.path, "w", encoding="utf-8") as f:
            json.dump(state, f)
        lock = ws.claim_lock("session-b", path=self.path)
        self.assertEqual(lock["session_id"], "session-b")

    def test_dead_holder_recovery_starts_fresh_budget_window(self):
        state = ws.load(self.path)
        state["lock"] = {
            "session_id": "dead-session",
            "pid": 2_000_000_000,
            "heartbeat_iso": datetime.now(timezone.utc).isoformat(),
        }
        state["budget"]["dispatches_this_session"] = 7
        state["budget"]["plans_this_session"] = 1
        with open(self.path, "w", encoding="utf-8") as f:
            json.dump(state, f)
        with mock.patch.dict(
            os.environ,
            {ws.DISPATCH_CAP_ENV: "10", ws.PLAN_CAP_ENV: "1"},
            clear=False,
        ):
            ws.claim_lock("replacement", path=self.path)
        budget = ws.load(self.path)["budget"]
        self.assertEqual(budget["dispatches_this_session"], 0)
        self.assertEqual(budget["plans_this_session"], 0)
        self.assertEqual(budget["dispatch_cap"], 10)
        self.assertEqual(budget["plans_per_session_cap"], 1)
        self.assertEqual(budget["sessions_completed"], 1)

    def test_heartbeat_refuses_when_not_holder(self):
        ws.claim_lock("session-a", path=self.path)
        with self.assertRaises(ws.LockRefused):
            ws.heartbeat("session-b", path=self.path)

    def test_heartbeat_refuses_when_no_lock(self):
        with self.assertRaises(ws.LockRefused):
            ws.heartbeat("session-a", path=self.path)

    def test_release_by_holder_works(self):
        ws.claim_lock("session-a", path=self.path)
        ws.release_lock("session-a", path=self.path)
        state = ws.load(self.path)
        self.assertIsNone(state["lock"])

    def test_release_by_non_holder_raises(self):
        ws.claim_lock("session-a", path=self.path)
        with self.assertRaises(ws.LockRefused):
            ws.release_lock("session-b", path=self.path)

    def test_release_on_none_lock_is_noop(self):
        ws.release_lock("session-a", path=self.path)
        state = ws.load(self.path)
        self.assertIsNone(state["lock"])


class TestCursor(TempStateCase):
    def test_update_cursor_sets_fields(self):
        ws.update_cursor("PLAN-1", "EXECUTING", "run step 3", path=self.path)
        state = ws.load(self.path)
        self.assertEqual(state["cursor"]["plan_id"], "PLAN-1")
        self.assertEqual(state["cursor"]["stage"], "EXECUTING")
        self.assertEqual(state["cursor"]["pickup"], "run step 3")

    def test_update_cursor_empty_pickup_raises(self):
        with self.assertRaises(ValueError):
            ws.update_cursor("PLAN-1", "EXECUTING", "", path=self.path)

    def test_update_cursor_none_pickup_raises(self):
        with self.assertRaises(ValueError):
            ws.update_cursor("PLAN-1", "EXECUTING", None, path=self.path)


class TestDecisions(TempStateCase):
    def test_append_decision_ids_increment(self):
        ws.append_decision("did X", "because Y", "evidence Z", path=self.path)
        ws.append_decision("did A", "because B", "evidence C", path=self.path)
        state = ws.load(self.path)
        self.assertEqual(len(state["decisions"]), 2)
        self.assertEqual(state["decisions"][0]["id"], "D-1")
        self.assertEqual(state["decisions"][1]["id"], "D-2")
        self.assertEqual(state["decisions"][1]["what"], "did A")


class TestDispatchBudget(TempStateCase):
    def test_record_dispatch_increments(self):
        count, tripped = ws.record_dispatch(path=self.path)
        self.assertEqual(count, 1)
        self.assertFalse(tripped)

    def test_record_dispatch_trips_at_cap(self):
        self._write_seed(overrides=None)
        state = ws.load(self.path)
        state["budget"]["dispatches_this_session"] = state["budget"]["dispatch_cap"] - 1
        with open(self.path, "w", encoding="utf-8") as f:
            json.dump(state, f)
        count, tripped = ws.record_dispatch(path=self.path)
        self.assertEqual(count, state["budget"]["dispatch_cap"])
        self.assertTrue(tripped)


class TestMarkPlanDone(TempStateCase):
    def test_mark_plan_done_writes_row_and_bumps_budget(self):
        plans_dir = os.path.join(self.tmpdir, "plan-status")
        count, tripped = ws.mark_plan_done(
            "PLAN-1", "abc123", "workflow/orchestrator-walk/evidence/PLAN-1.json",
            path=self.path,
        )
        self.assertEqual(count, 1)
        self.assertFalse(tripped)
        row_path = os.path.join(plans_dir, "PLAN-1.json")
        self.assertTrue(os.path.exists(row_path))
        with open(row_path, encoding="utf-8") as f:
            row = json.load(f)
        self.assertEqual(row["plan_id"], "PLAN-1")
        self.assertEqual(row["status"], "DONE")
        self.assertEqual(row["commit_sha"], "abc123")
        self.assertEqual(row["gate_evidence_ref"], "workflow/orchestrator-walk/evidence/PLAN-1.json")
        self.assertIn("last_run_iso", row)

    def test_mark_plan_done_trips_at_cap(self):
        state = ws.load(self.path)
        state["budget"]["plans_this_session"] = state["budget"]["plans_per_session_cap"] - 1
        with open(self.path, "w", encoding="utf-8") as f:
            json.dump(state, f)
        count, tripped = ws.mark_plan_done("PLAN-2", "sha2", "evidence-ref", path=self.path)
        self.assertEqual(count, state["budget"]["plans_per_session_cap"])
        self.assertTrue(tripped)

    def test_mark_plan_done_empty_commit_sha_raises(self):
        with self.assertRaises(ws.DoneRowRejected):
            ws.mark_plan_done("PLAN-3", "", "evidence-ref", path=self.path)
        plans_dir = os.path.join(self.tmpdir, "plan-status")
        self.assertFalse(os.path.exists(os.path.join(plans_dir, "PLAN-3.json")))
        state = ws.load(self.path)
        self.assertEqual(state["budget"]["plans_this_session"], 0)

    def test_mark_plan_done_none_commit_sha_raises(self):
        with self.assertRaises(ws.DoneRowRejected):
            ws.mark_plan_done("PLAN-4", None, "evidence-ref", path=self.path)

    def test_mark_plan_done_empty_evidence_ref_raises(self):
        with self.assertRaises(ws.DoneRowRejected):
            ws.mark_plan_done("PLAN-5", "sha5", "", path=self.path)
        plans_dir = os.path.join(self.tmpdir, "plan-status")
        self.assertFalse(os.path.exists(os.path.join(plans_dir, "PLAN-5.json")))

    def test_mark_plan_done_none_evidence_ref_raises(self):
        with self.assertRaises(ws.DoneRowRejected):
            ws.mark_plan_done("PLAN-6", "sha6", None, path=self.path)


class TestAtomicity(TempStateCase):
    def test_no_tmp_residue_after_write(self):
        ws.claim_lock("session-a", path=self.path)
        ws.append_decision("x", "y", "z", path=self.path)
        ws.update_cursor("PLAN-1", "EXECUTING", "go", path=self.path)
        self.assertEqual(self._tmp_residue(), [])

    def test_garbage_tmp_file_does_not_break_load(self):
        garbage = self.path + ".garbage.tmp"
        with open(garbage, "w", encoding="utf-8") as f:
            f.write("not json at all {{{")
        state = ws.load(self.path)
        self.assertEqual(state["schema_version"], 1)


class TestCompletePlan(TempStateCase):
    def _seed_queue_cursor(self, queue, cursor):
        state = json.loads(json.dumps(SEED))
        state["queue"] = queue
        state["cursor"] = cursor
        with open(self.path, "w", encoding="utf-8") as f:
            json.dump(state, f)

    def test_removes_queued_id(self):
        self._seed_queue_cursor(["X", "Y"], {"plan_id": None, "stage": None, "pickup": None})
        ws.complete_plan("X", self.path)
        self.assertEqual(ws.load(self.path)["queue"], ["Y"])

    def test_idempotent_when_absent(self):
        self._seed_queue_cursor(["Y"], {"plan_id": None, "stage": None, "pickup": None})
        ws.complete_plan("X", self.path)  # X not present -> no-op
        self.assertEqual(ws.load(self.path)["queue"], ["Y"])

    def test_clears_matching_cursor(self):
        self._seed_queue_cursor(["X"], {"plan_id": "X", "stage": "done", "pickup": "commit X"})
        ws.complete_plan("X", self.path)
        state = ws.load(self.path)
        self.assertEqual(state["queue"], [])
        self.assertEqual(state["cursor"], {"plan_id": None, "stage": None, "pickup": None})

    def test_leaves_non_matching_cursor(self):
        self._seed_queue_cursor(["X"], {"plan_id": "Y", "stage": "EXECUTING", "pickup": "run Y"})
        ws.complete_plan("X", self.path)
        state = ws.load(self.path)
        self.assertEqual(state["queue"], [])
        self.assertEqual(state["cursor"], {"plan_id": "Y", "stage": "EXECUTING", "pickup": "run Y"})

    def test_empty_plan_id_raises(self):
        with self.assertRaises(ValueError):
            ws.complete_plan("", self.path)

    def test_no_tmp_residue(self):
        self._seed_queue_cursor(["X"], {"plan_id": "X", "stage": "done", "pickup": "commit X"})
        ws.complete_plan("X", self.path)
        self.assertEqual(self._tmp_residue(), [])


class TestIsPlanDone(TempStateCase):
    def _write_status_row(self, plan_id, status="DONE"):
        plans_dir = os.path.join(self.tmpdir, "plan-status")
        os.makedirs(plans_dir, exist_ok=True)
        with open(os.path.join(plans_dir, f"{plan_id}.json"), "w", encoding="utf-8") as f:
            json.dump({"plan_id": plan_id, "status": status, "commit_sha": "abc",
                       "gate_evidence_ref": "ref", "last_run_iso": "2026-07-10T00:00:00+00:00"}, f)

    def test_missing_row_is_false(self):
        self.assertFalse(ws.is_plan_done("NOPE", self.path))

    def test_done_row_is_true(self):
        self._write_status_row("X", "DONE")
        self.assertTrue(ws.is_plan_done("X", self.path))

    def test_non_done_row_is_false(self):
        self._write_status_row("X", "IN_PROGRESS")
        self.assertFalse(ws.is_plan_done("X", self.path))

    def test_unreadable_row_is_false(self):
        plans_dir = os.path.join(self.tmpdir, "plan-status")
        os.makedirs(plans_dir, exist_ok=True)
        with open(os.path.join(plans_dir, "X.json"), "w", encoding="utf-8") as f:
            f.write("{ not json")
        self.assertFalse(ws.is_plan_done("X", self.path))

    def test_mark_plan_done_makes_is_plan_done_true(self):
        ws.mark_plan_done("X", "sha", "ref", path=self.path)
        self.assertTrue(ws.is_plan_done("X", self.path))


class TestResolveEscalation(TempStateCase):
    def test_removes_matching_row(self):
        ws.append_escalation("J-34", "k27", 1, "ref-a", path=self.path)
        ws.append_escalation("J-24", "k27", 1, "ref-b", path=self.path)
        out = ws.resolve_escalation("J-34", reason="owner ratified A+A", path=self.path)
        state = ws.load(self.path)
        ids = [r.get("plan_id") for r in state["escalation_queue"]]
        self.assertTrue(out["removed"])
        self.assertEqual(ids, ["J-24"])
        self.assertEqual(out["prior"]["gate_json_ref"], "ref-a")

    def test_idempotent_when_absent(self):
        ws.append_escalation("J-24", "k27", 1, "ref-b", path=self.path)
        out = ws.resolve_escalation("J-34", path=self.path)
        self.assertFalse(out["removed"])
        self.assertIsNone(out["prior"])
        self.assertEqual(
            [r.get("plan_id") for r in ws.load(self.path)["escalation_queue"]],
            ["J-24"],
        )

    def test_empty_plan_id_raises(self):
        with self.assertRaises(ValueError):
            ws.resolve_escalation("", path=self.path)

    def test_no_tmp_residue(self):
        ws.append_escalation("J-34", "k27", 1, "ref-a", path=self.path)
        ws.resolve_escalation("J-34", reason="cleared", path=self.path)
        self.assertEqual(self._tmp_residue(), [])


if __name__ == "__main__":
    unittest.main()
