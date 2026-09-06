"""Clear-bug unit tests for conductor_db event atomicity + events.iter_events soft parse."""
from __future__ import annotations

import unittest
from unittest import mock

from conductor_db import ControlDB
from events import iter_events, replay_check
from tests.helpers import TempDirMixin
from tests.test_models import sample_task


class EventAtomicityTests(TempDirMixin, unittest.TestCase):
    def test_update_task_status_rolls_back_when_event_insert_fails(self) -> None:
        db_path = self.make_tempdir() / "control.db"
        db = ControlDB(db_path)
        db.init(reset=True)
        task = sample_task("ATOMIC-1")
        db.upsert_task(task, status="READY")

        with mock.patch.object(
            ControlDB, "_insert_event", side_effect=RuntimeError("boom")
        ):
            with self.assertRaises(RuntimeError):
                db.update_task_status(task.task_id, "LEASED", "unit")

        # Status must remain READY — not silently LEASED without TASK_STATUS_CHANGED.
        self.assertEqual(dict(db.get_task_row(task.task_id))["status"], "READY")
        # No TASK_STATUS_CHANGED trail either.
        rows = db.query_events(task_id=task.task_id, event_type="TASK_STATUS_CHANGED")
        self.assertEqual(rows, [])

    def test_update_task_status_atomic_success_emits_event(self) -> None:
        db_path = self.make_tempdir() / "control.db"
        db = ControlDB(db_path)
        db.init(reset=True)
        task = sample_task("ATOMIC-2")
        db.upsert_task(task, status="READY")
        db.update_task_status(task.task_id, "LEASED", "unit")
        self.assertEqual(dict(db.get_task_row(task.task_id))["status"], "LEASED")
        rows = db.query_events(task_id=task.task_id, event_type="TASK_STATUS_CHANGED")
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["payload"]["new_status"], "LEASED")

    def test_iter_events_soft_parses_corrupt_payload(self) -> None:
        db_path = self.make_tempdir() / "control.db"
        db = ControlDB(db_path)
        db.init(reset=True)
        db.append_event("GOOD", "T-1", {"k": 1})
        with db.transaction() as conn:
            conn.execute(
                """
                INSERT INTO events
                    (event_id, task_id, event_type, payload_json, created_at, project_id, session_id)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                """,
                ("bad-row", "T-1", "BAD", "not-json", "t", "conductor", None),
            )
        events = iter_events(db_path)
        by_type = {e["event_type"]: e for e in events}
        self.assertIn("GOOD", by_type)
        self.assertEqual(by_type["GOOD"]["payload"], {"k": 1})
        self.assertIn("BAD", by_type)
        self.assertEqual(by_type["BAD"]["payload"], {})

    def test_replay_check_survives_corrupt_payload_and_matches_status(self) -> None:
        db_path = self.make_tempdir() / "control.db"
        db = ControlDB(db_path)
        db.init(reset=True)
        task = sample_task("REPLAY-1")
        db.upsert_task(task, status="READY")
        db.update_task_status(task.task_id, "LEASED", "unit")
        with db.transaction() as conn:
            conn.execute(
                """
                INSERT INTO events
                    (event_id, task_id, event_type, payload_json, created_at, project_id, session_id)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                """,
                ("bad-row", None, "NOISE", "not-json", "t", "conductor", None),
            )
        result = replay_check(db_path)
        self.assertTrue(result["ok"], result)


if __name__ == "__main__":
    unittest.main()
