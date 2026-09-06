from __future__ import annotations

import sqlite3
import unittest

from conductor_db import ControlDB
from tests.helpers import TempDirMixin


class DBSchemaTests(TempDirMixin, unittest.TestCase):
    def test_init_enables_wal_foreign_keys_and_required_tables(self) -> None:
        db_path = self.make_tempdir() / "control.db"
        db = ControlDB(db_path)
        db.init(reset=True)
        check = db.check()
        self.assertTrue(check["ok"], check)
        self.assertEqual(check["journal_mode"], "wal")
        self.assertEqual(check["foreign_keys"], 1)
        self.assertEqual(check["schema_version"], "2")

    def test_events_are_append_only(self) -> None:
        db_path = self.make_tempdir() / "control.db"
        db = ControlDB(db_path)
        db.init(reset=True)
        event_id = db.append_event("UNIT_EVENT", "T-1", {"secret_token": "abc123"})
        conn = db.connect()
        try:
            with self.assertRaises(sqlite3.IntegrityError):
                conn.execute(
                    "UPDATE events SET event_type = 'MUTATED' WHERE event_id = ?",
                    (event_id,),
                )
        finally:
            conn.close()

    def test_events_redact_secret_keys(self) -> None:
        db_path = self.make_tempdir() / "control.db"
        db = ControlDB(db_path)
        db.init(reset=True)
        db.append_event("UNIT_EVENT", None, {"api_key": "not-for-logs"})
        conn = db.connect()
        try:
            row = conn.execute("SELECT payload_json FROM events").fetchone()
            self.assertIn("<REDACTED>", row["payload_json"])
            self.assertNotIn("not-for-logs", row["payload_json"])
        finally:
            conn.close()


if __name__ == "__main__":
    unittest.main()
