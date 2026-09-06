from __future__ import annotations

import json
import unittest
from pathlib import Path

import free_api_worker
import provider_usage
from conductor_db import ControlDB
from worker_adapter import ProviderResult, WorkerAdapter

from tests.helpers import TempDirMixin
from tests.test_worker_adapter import FakeProvider, make_git_repo, scoped_task


_ENVELOPE = json.dumps({
    "type": "result", "subtype": "success", "result": "worker finished the edit",
    "usage": {"input_tokens": 120, "output_tokens": 45, "cache_read_input_tokens": 10},
    "total_cost_usd": 0.0,
})


class ParseCcJsonUsageTests(unittest.TestCase):
    """W-U3: extract per-turn token/cost from base-cc's --output-format json envelope."""

    def test_extracts_tokens_and_cost(self) -> None:
        got = free_api_worker.parse_cc_json_usage("noise\n" + _ENVELOPE)
        self.assertEqual(got, {"tokens_in": 120, "tokens_out": 45, "cache_read": 10, "cost_cents": 0})

    def test_cost_usd_rounds_to_cents(self) -> None:
        env = json.dumps({"result": "x", "usage": {"input_tokens": 1, "output_tokens": 1},
                          "total_cost_usd": 0.037})
        self.assertEqual(free_api_worker.parse_cc_json_usage(env)["cost_cents"], 4)

    def test_no_usage_block_returns_none(self) -> None:
        self.assertIsNone(free_api_worker.parse_cc_json_usage(json.dumps({"result": "x"})))
        self.assertIsNone(free_api_worker.parse_cc_json_usage("not json at all"))

    def test_all_zero_is_none(self) -> None:
        env = json.dumps({"result": "x", "usage": {"input_tokens": 0, "output_tokens": 0}})
        self.assertIsNone(free_api_worker.parse_cc_json_usage(env))


class TimeoutProvider:
    """A provider that always reports a wall-clock timeout (the watchdog-fire condition)."""

    def run(self, command, cwd, timeout_seconds, env, input_text=None) -> ProviderResult:
        return ProviderResult(-1, "partial stdout", "partial stderr", timed_out=True)

    def cancel(self) -> None:
        pass


class WorkerSeamTests(TempDirMixin, unittest.TestCase):
    """W-U3 + W-U4d: the AUTONOMOUS worker seam records per-lane usage and surfaces the watchdog."""

    def _adapter(self, temp: Path, provider) -> tuple[WorkerAdapter, Path]:
        repo = make_git_repo(temp / "repo")
        adapter = WorkerAdapter(
            {
                "worktree_base": str(temp / "worktrees"),
                "artifact_root": str(temp / "artifacts"),
                "repo_root": str(repo),
                "agent_root": str(temp / "agents"),
                "workers": [{"slot_id": "fa1", "model": "free-fast", "model_tier": "FREE",
                             "harness": "free-api", "account_id": "account-a"}],
            },
            provider=provider,
        )
        return adapter, repo

    def test_autonomous_worker_turn_lands_a_provider_row(self) -> None:
        temp = self.make_tempdir()
        db_path = temp / "control.db"
        ControlDB(db_path).init(reset=True)
        provider = FakeProvider(stdout=_ENVELOPE)
        adapter, _ = self._adapter(temp, provider)
        adapter.dispatch(scoped_task("W-U3-CAP-1"), db_path=db_path)
        rows = {r["lane"]: r for r in provider_usage.provider_rows(ControlDB(db_path))}
        self.assertIn("free-api", rows)
        self.assertEqual(rows["free-api"]["tokens_in"], 120)
        self.assertEqual(rows["free-api"]["tokens_out"], 45)
        self.assertEqual(rows["free-api"]["cache_read"], 10)

    def test_env_marks_worker_autonomous(self) -> None:
        # W2: every spawned worker session carries TEIKOKU_AUTONOMOUS=1 so usage/agent-time fire.
        adapter, _ = self._adapter(self.make_tempdir(), FakeProvider(stdout=_ENVELOPE))
        env = adapter._worker_env(adapter.select_slot("FREE"))
        self.assertEqual(env.get("TEIKOKU_AUTONOMOUS"), "1")

    def test_watchdog_fire_emitted_only_when_wall_clock_kill(self) -> None:
        temp = self.make_tempdir()
        db_path = temp / "control.db"
        ControlDB(db_path).init(reset=True)
        adapter, _ = self._adapter(temp, TimeoutProvider())
        # autonomous (default wall_clock_kill=True): the watchdog fire is surfaced.
        adapter.dispatch(scoped_task("W-U4-WD-1"), db_path=db_path)
        events = ControlDB(db_path).query_events(event_type="WATCHDOG_FIRE", limit=5)
        self.assertEqual(len(events), 1)
        self.assertTrue(events[0]["payload"]["killed"])
        self.assertIn("exceeded", events[0]["payload"]["observation"])
        # interactive (wall_clock_kill=False): no forced-kill observation.
        adapter.dispatch(scoped_task("W-U4-WD-2"), db_path=db_path, wall_clock_kill=False)
        events2 = ControlDB(db_path).query_events(event_type="WATCHDOG_FIRE", limit=5)
        self.assertEqual(len(events2), 1)


if __name__ == "__main__":
    unittest.main()
