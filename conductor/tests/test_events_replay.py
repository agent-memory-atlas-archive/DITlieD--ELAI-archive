from __future__ import annotations

import unittest

from conductor_db import ControlDB
from events import replay_check
from tests.helpers import TempDirMixin
from tests.test_models import sample_task


class EventsReplayTests(TempDirMixin, unittest.TestCase):
    def test_replay_matches_task_statuses(self) -> None:
        db_path = self.make_tempdir() / "control.db"
        db = ControlDB(db_path)
        db.init(reset=True)
        task = sample_task("B-4-EXECUTE-1")
        db.upsert_task(task, status="READY")
        db.update_task_status(task.task_id, "LEASED", "unit")
        db.update_task_status(task.task_id, "VERIFYING", "unit")
        result = replay_check(db_path)
        self.assertTrue(result["ok"], result)
        self.assertEqual(result["tasks_checked"], 1)


if __name__ == "__main__":
    unittest.main()
