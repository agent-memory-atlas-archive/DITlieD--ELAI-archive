"""Unit tests for VERIFIER_UNAVAILABLE park (A-EV-QUEUE-BLOCK / ARCH-EV-QUEUE-BLOCK).

Park is typed WAITING_FOR_VERIFIER: durable plan-status + pending_lands reason, never DONE
or LANDED. Other READY tasks remain leaseable. REJECT does not park.
"""
from __future__ import annotations

import inspect
import json
import sys
import tempfile
import unittest
from pathlib import Path

import opus_land as ol
import pending_lands as pl

CONDUCTOR = Path(__file__).resolve().parents[1]
if str(CONDUCTOR) not in sys.path:
    sys.path.insert(0, str(CONDUCTOR))


class TestParkVerifierUnavailable(unittest.TestCase):
    def test_park_writes_plan_status_waiting_for_verifier(self):
        with tempfile.TemporaryDirectory() as td:
            td_path = Path(td)
            status_dir = td_path / "plan-status"
            pending = td_path / "pending-lands.json"
            events = []

            park = ol.park_verifier_unavailable(
                "PLAN-PARK-1",
                "/tmp/wt-park-1",
                detail={"error": "spawn timeout"},
                attempt=3,
                pending_path=pending,
                plan_status_dir=status_dir,
                emit_fn=lambda et, payload: events.append((et, payload)),
            )

            self.assertEqual(park["waiting_status"], "WAITING_FOR_VERIFIER")
            self.assertTrue(park["park"])
            self.assertEqual(park["plan_id"], "PLAN-PARK-1")
            self.assertNotEqual(park.get("status"), "DONE")
            row_path = status_dir / "PLAN-PARK-1.json"
            self.assertTrue(row_path.is_file())
            row = json.loads(row_path.read_text(encoding="utf-8"))
            self.assertEqual(row["status"], "WAITING_FOR_VERIFIER")
            self.assertEqual(row["waiting_reason"], "verifier_unavailable")
            self.assertNotEqual(row["status"], "DONE")
            self.assertIsNone(row.get("commit_sha"))

            lands = pl.list_lands(pending)
            self.assertEqual(len(lands), 1)
            self.assertEqual(lands[0]["reason"], "WAITING_FOR_VERIFIER")
            self.assertEqual(lands[0]["plan_id"], "PLAN-PARK-1")
            waiting = pl.list_waiting_for_verifier(pending)
            self.assertEqual(len(waiting), 1)
            self.assertEqual(park["resume_queue_size"], 1)

            self.assertEqual(len(events), 1)
            self.assertEqual(events[0][0], "OPUS_LAND_VERIFIER_PARKED")
            self.assertEqual(events[0][1]["waiting_status"], "WAITING_FOR_VERIFIER")

    def test_park_not_landed_verdict_contract(self):
        """Park artifact must never look like a successful land."""
        with tempfile.TemporaryDirectory() as td:
            td_path = Path(td)
            park = ol.park_verifier_unavailable(
                "PLAN-X",
                "/tmp/wt-x",
                detail={"error": "oauth flake"},
                pending_path=td_path / "pl.json",
                plan_status_dir=td_path / "ps",
                emit_fn=lambda *_: None,
            )
            self.assertTrue(park["park"])
            self.assertEqual(park["waiting_status"], "WAITING_FOR_VERIFIER")
            self.assertNotIn(park.get("verdict"), ("LANDED", "DONE", "GATES_PASS"))
            self.assertFalse(park.get("landed", False))

    def test_park_with_task_id_sets_controldb_status(self):
        from conductor_db import ControlDB
        from models import TaskSpec, stable_idem_key
        from leases import LeaseManager

        with tempfile.TemporaryDirectory() as td:
            td_path = Path(td)
            db_path = td_path / "control.db"
            db = ControlDB(db_path)
            db.init(reset=True)
            db.seed_worker_slot("w1", "swe-1.6", account_id="account-a")

            def _task(tid: str, plan: str) -> TaskSpec:
                return TaskSpec(
                    task_id=tid,
                    plan_id=plan,
                    wave="IMPLEMENT",
                    role="EXECUTE",
                    objective=f"objective for {tid}",
                    base_commit="HEAD",
                    allowed_write_set=[f"src/{tid}.rs"],
                    context_refs=["plan.md"],
                    production_hook="hook",
                    acceptance="ok",
                    deterministic_check="true",
                    forbidden=[],
                    verifier=[],
                    lease_ttl_seconds=60,
                    idem_key=stable_idem_key([tid, "HEAD"]),
                    token_budget=100,
                )

            db.upsert_task(_task("TASK-PARKED", "PLAN-PARK"), status="READY")
            db.upsert_task(_task("TASK-OTHER", "PLAN-OTHER"), status="READY")

            park = ol.park_verifier_unavailable(
                "PLAN-PARK",
                "/tmp/wt-park",
                detail={"error": "unavailable"},
                task_id="TASK-PARKED",
                pending_path=td_path / "pl.json",
                plan_status_dir=td_path / "ps",
                db_path=db_path,
                emit_fn=lambda *_: None,
            )
            self.assertTrue(park["task_parked"])
            self.assertEqual(
                db.get_task_row("TASK-PARKED")["status"], "WAITING_FOR_VERIFIER"
            )
            self.assertEqual(db.get_task_row("TASK-OTHER")["status"], "READY")

            lm = LeaseManager(db_path, worktree_root=td_path / "wt")
            lease = lm.acquire_ready()
            self.assertIsNotNone(lease)
            self.assertEqual(lease.task_id, "TASK-OTHER")
            self.assertEqual(
                db.get_task_row("TASK-PARKED")["status"], "WAITING_FOR_VERIFIER"
            )

    def test_resume_queue_lists_only_waiting_reason(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "pl.json"
            pl.add("A", "/wt/a", path=p, reason="WAITING_FOR_VERIFIER")
            pl.add("B", "/wt/b", path=p, reason="")
            pl.add("C", "/wt/c", path=p, reason="WAITING_FOR_PROVIDER")
            waiting = pl.list_waiting_for_verifier(p)
            self.assertEqual([x["plan_id"] for x in waiting], ["A"])


class TestMainWiresPark(unittest.TestCase):
    def test_main_calls_park_on_unavailable(self):
        src = inspect.getsource(ol.main)
        self.assertIn("park_verifier_unavailable", src)
        self.assertIn("VERIFIER_UNAVAILABLE_PARKED", src)
        self.assertIn("WAITING_FOR_VERIFIER", src)
        self.assertIn("OPUS_LAND_VERIFIER_PARKED", src)
        self.assertIn('verdict == "unavailable"', src)
        self.assertIn('verdict == "reject"', src)
        reject_idx = src.find('verdict == "reject"')
        unavail_idx = src.find('verdict == "unavailable"')
        self.assertGreater(unavail_idx, reject_idx)
        reject_block = src[reject_idx:unavail_idx]
        self.assertNotIn("park_verifier_unavailable", reject_block)

    def test_park_helper_is_public_and_main_is_consumer(self):
        self.assertTrue(callable(ol.park_verifier_unavailable))
        self.assertIn("park_verifier_unavailable", inspect.getsource(ol.main))


if __name__ == "__main__":
    unittest.main()
