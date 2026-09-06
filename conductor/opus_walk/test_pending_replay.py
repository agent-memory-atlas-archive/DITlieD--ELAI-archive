from __future__ import annotations

import datetime as dt
import json
import multiprocessing
import os
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

import pending_lands as pl
import pending_replay as replay


NOW = dt.datetime(2026, 8, 13, 12, 0, tzinfo=dt.timezone.utc)


def _args(plan_id: str, worktree: Path) -> dict:
    values = {}
    for field in pl.REPLAY_ARG_FIELDS:
        if field == "worktree":
            values[field] = str(worktree)
        elif field == "plan_id":
            values[field] = plan_id
        elif field == "files":
            values[field] = [".elai_cc/marker.txt"]
        elif field == "gate_cmd":
            values[field] = ["true"]
        elif field in {"gates_only", "no_verifier", "push", "json"}:
            values[field] = False
        elif field == "message":
            values[field] = "replay test land"
        elif field == "task_class":
            values[field] = "u3b-test"
        else:
            values[field] = None
    return values


def _argv(plan_id: str, worktree: Path) -> list[str]:
    return [
        "--worktree", str(worktree),
        "--plan-id", plan_id,
        "--files", ".elai_cc/marker.txt",
        "--gate-cmd", "true",
        "--message", "replay test land",
        "--task-class", "u3b-test",
        "--json",
    ]


def _add(path: Path, plan_id: str, worktree: Path, *, attempt: int = 0) -> None:
    pl.add(
        plan_id,
        str(worktree),
        path=path,
        reason="WAITING_FOR_VERIFIER",
        replay_args=_args(plan_id, worktree),
        replay_argv=_argv(plan_id, worktree),
    )
    if attempt:
        raw = json.loads(path.read_text(encoding="utf-8"))
        raw["lands"][0]["attempt"] = attempt
        raw["lands"][0]["next_eligible_iso"] = "2026-08-13T00:00:00Z"
        path.write_text(json.dumps(raw), encoding="utf-8")


class _FakeProcess:
    def __init__(self, returncode: int, stdout: str, stderr: str = "") -> None:
        self.returncode = returncode
        self.stdout = stdout
        self.stderr = stderr
        self.pid = 99999999

    def communicate(self, timeout=None):
        return self.stdout, self.stderr


def _fake_factory(calls, *, returncode=0, stdout="STATUS: LANDED", stderr=""):
    def factory(command, **kwargs):
        calls.append((command, kwargs))
        return _FakeProcess(returncode, stdout, stderr)
    return factory


def _timeout_worker(path: str, script: str, queue) -> None:
    result = replay.replay_one(
        path=Path(path),
        opus_land_path=Path(script),
        deadline_seconds=0.2,
        now=NOW,
    )
    queue.put(result)


def _concurrent_worker(path: str, script: str, queue) -> None:
    queue.put(
        replay.replay_one(
            path=Path(path),
            opus_land_path=Path(script),
            now=NOW,
        )
    )


class PendingReplayTests(unittest.TestCase):
    def test_backoff_values_and_attempt_cap(self):
        self.assertEqual(replay.backoff_seconds(1), 900)
        self.assertEqual(replay.backoff_seconds(2), 1800)
        self.assertEqual(replay.backoff_seconds(3), 3600)
        self.assertEqual(replay.backoff_seconds(8), 3600)

        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            ledger = root / "pending-lands.json"
            worktree = root / "worktree"
            worktree.mkdir()
            _add(ledger, "CAP", worktree)
            calls = []
            factory = _fake_factory(calls, returncode=1, stdout="GATE_FAIL")
            first = replay.replay_one(
                path=ledger, now=NOW, popen_factory=factory,
            )
            self.assertEqual(first["attempt"], 1)
            second = replay.replay_one(
                path=ledger,
                now=NOW + dt.timedelta(seconds=901),
                popen_factory=factory,
            )
            self.assertEqual(second["attempt"], 2)
            self.assertTrue(second["owner_attention"])
            self.assertIsNone(
                replay.replay_one(
                    path=ledger,
                    now=NOW + dt.timedelta(seconds=2701),
                    popen_factory=factory,
                )
            )
            self.assertEqual(len(calls), 2)
            row = pl.list_lands(ledger)[0]
            self.assertEqual(row["attempt"], 2)
            self.assertTrue(row["owner_attention"])
            self.assertNotIn("claim_token", row)

    def test_success_removes_only_after_landed_subprocess(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            ledger = root / "pending-lands.json"
            worktree = root / "worktree"
            worktree.mkdir()
            _add(ledger, "SUCCESS", worktree)
            calls = []
            result = replay.replay_one(
                path=ledger,
                now=NOW,
                popen_factory=_fake_factory(calls),
            )
            self.assertEqual(result["status"], "landed")
            self.assertTrue(result["removed"])
            self.assertEqual(pl.list_lands(ledger), [])
            self.assertEqual(len(calls), 1)
            self.assertTrue(calls[0][1]["start_new_session"])
            self.assertIn("--plan-id", calls[0][0])

    def test_gate_failure_retains_row_and_records_backoff(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            ledger = root / "pending-lands.json"
            worktree = root / "worktree"
            worktree.mkdir()
            _add(ledger, "GATE", worktree)
            result = replay.replay_one(
                path=ledger,
                now=NOW,
                popen_factory=_fake_factory([], returncode=1, stdout="STATUS: GATE_FAIL"),
            )
            self.assertEqual(result["status"], "failed")
            self.assertEqual(result["failure"], "land_failed")
            row = pl.list_lands(ledger)[0]
            self.assertEqual(row["attempt"], 1)
            self.assertEqual(row["backoff_seconds"], 900) if "backoff_seconds" in row else None
            self.assertEqual(row["last_replay_status"], "land_failed")
            self.assertIn("next_eligible_iso", row)
            self.assertTrue(row["replayable"])

    def test_hash_tamper_releases_claim_without_subprocess(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            ledger = root / "pending-lands.json"
            worktree = root / "worktree"
            worktree.mkdir()
            _add(ledger, "HASH", worktree)
            raw = json.loads(ledger.read_text(encoding="utf-8"))
            raw["lands"][0]["replay_args"]["gate_cmd"] = ["false"]
            ledger.write_text(json.dumps(raw), encoding="utf-8")
            calls = []
            result = replay.replay_one(
                path=ledger,
                now=NOW,
                popen_factory=_fake_factory(calls),
            )
            self.assertIsNone(result)
            self.assertEqual(calls, [])
            self.assertFalse(pl.list_lands(ledger)[0].get("claim_token"))

    def test_missing_worktree_is_stale_and_retained(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            ledger = root / "pending-lands.json"
            missing = root / "gone"
            _add(ledger, "STALE", missing)
            result = replay.replay_one(path=ledger, now=NOW)
            self.assertEqual(result["status"], "stale")
            self.assertTrue(result["retained"])
            row = pl.list_lands(ledger)[0]
            self.assertTrue(row["stale"])
            self.assertEqual(row["stale_reason"], "missing_worktree")
            self.assertNotIn("claim_token", row)

    def test_timeout_kills_child_and_grandchild_and_records_backoff(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            ledger = root / "pending-lands.json"
            worktree = root / "worktree"
            worktree.mkdir()
            pid_file = root / "grandchild.pid"
            script = root / "timeout_land.py"
            script.write_text(
                "import pathlib, subprocess, sys, time\n"
                f"p = subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(30)'])\n"
                f"pathlib.Path({str(pid_file)!r}).write_text(str(p.pid))\n"
                "time.sleep(30)\n",
                encoding="utf-8",
            )
            _add(ledger, "TIMEOUT", worktree)
            result = replay.replay_one(
                path=ledger,
                opus_land_path=script,
                deadline_seconds=0.2,
                now=NOW,
            )
            self.assertEqual(result["status"], "failed")
            self.assertEqual(result["failure"], "deadline")
            self.assertEqual(result["backoff_seconds"], 900)
            self.assertTrue(pl.list_lands(ledger)[0]["replayable"])
            pid = int(pid_file.read_text(encoding="utf-8"))
            for _ in range(50):
                if not Path(f"/proc/{pid}").exists():
                    break
                time.sleep(0.05)
            self.assertFalse(Path(f"/proc/{pid}").exists())

    def test_concurrent_replay_has_one_winner(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            ledger = root / "pending-lands.json"
            worktree = root / "worktree"
            worktree.mkdir()
            script = root / "success_land.py"
            script.write_text("print('STATUS: LANDED')\n", encoding="utf-8")
            _add(ledger, "WINNER", worktree)
            context = multiprocessing.get_context("fork")
            queue = context.Queue()
            workers = [
                context.Process(target=_concurrent_worker, args=(str(ledger), str(script), queue))
                for _ in range(2)
            ]
            for worker in workers:
                worker.start()
            results = [queue.get(timeout=10) for _ in workers]
            for worker in workers:
                worker.join(timeout=10)
            self.assertEqual(sum(result is not None for result in results), 1, results)
            self.assertEqual(pl.list_lands(ledger), [])


if __name__ == "__main__":
    unittest.main()
