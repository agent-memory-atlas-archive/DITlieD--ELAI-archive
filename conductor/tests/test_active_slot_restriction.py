from __future__ import annotations

import shutil
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path

from conductor_db import ControlDB
from leases import LeaseManager

from tests.test_models import sample_task


def _task(task_id: str, scope: list[str], tier: str):
    return replace(
        sample_task(task_id), plan_id=task_id, allowed_write_set=scope,
        context_refs=scope, production_hook=f"{scope[0]}::hook", model_tier=tier,
    )


class ActiveSlotRestrictionTests(unittest.TestCase):
    """SLOT-FIX (paid-leg-entry.md): acquire_ready must restrict its slot candidates to the
    ACTIVE config's slot_id set, so a stale worker_slots row left by a PRIOR config (seeding
    only upserts, never removes) can never win a lease the current config has no dispatch
    path for (worker_adapter.select_slot resolves against that same in-memory pool)."""

    def setUp(self) -> None:
        self.temp = Path(tempfile.mkdtemp(prefix="slot-restrict-"))
        self.addCleanup(lambda: shutil.rmtree(self.temp, ignore_errors=True))
        self.db_path = self.temp / "control.db"
        self.db = ControlDB(self.db_path)
        self.db.init(reset=True)
        self.db.seed_worker_slot("devin1", "glm-5.2", model_tier="FREE", account_id="account-a")
        self.db.seed_worker_slot("opus-exec", "opus", model_tier="PAID", account_id="account-b")
        self.leases = LeaseManager(self.db_path)

    def test_lone_free_task_gets_no_lease_when_slot_excluded_from_active_set(self) -> None:
        self.db.upsert_task(_task("FREE-TASK", ["src/free.rs"], "FREE"), status="READY")
        lease = self.leases.acquire_ready(active_slot_ids={"opus-exec"})
        self.assertIsNone(lease)
        self.assertEqual(self.db.get_task_row("FREE-TASK")["status"], "READY")

    def test_paid_task_leases_in_same_call_despite_stale_free_slot(self) -> None:
        self.db.upsert_task(_task("FREE-TASK", ["src/free.rs"], "FREE"), status="READY")
        self.db.upsert_task(_task("PAID-TASK", ["src/paid.rs"], "PAID"), status="READY")
        lease = self.leases.acquire_ready(active_slot_ids={"opus-exec"})
        self.assertIsNotNone(lease)
        self.assertEqual(lease.task_id, "PAID-TASK")
        self.assertEqual(lease.slot_id, "opus-exec")
        self.assertEqual(self.db.get_task_row("FREE-TASK")["status"], "READY")

    def test_active_slot_ids_none_preserves_today_behavior(self) -> None:
        self.db.upsert_task(_task("FREE-TASK", ["src/free.rs"], "FREE"), status="READY")
        self.db.upsert_task(_task("PAID-TASK", ["src/paid.rs"], "PAID"), status="READY")
        lease = self.leases.acquire_ready()
        self.assertIsNotNone(lease)
        self.assertEqual(lease.task_id, "FREE-TASK")
        self.assertEqual(lease.slot_id, "devin1")



class WorkerAdapterTierPartitionTests(unittest.TestCase):
    """SLOT-FIX leg 2 (paid-leg-entry.md DECISION a): WorkerAdapter pools partition by each
    slot's OWN model_tier, not by which TOML section ([[workers]] vs [[escalation_workers]])
    declared it. leases.acquire_ready already matches per-slot model_tier; the section-based
    split was the inconsistent leg (KeyError 'worker slot not found for PAID: opus-exec')."""

    def _adapter(self):
        from worker_adapter import WorkerAdapter
        return WorkerAdapter({
            "workers": [
                {"slot_id": "opus-exec", "model": "claude-opus-4-8[1m]",
                 "model_tier": "PAID", "harness": "claude", "pool": "claude"},
                {"slot_id": "devin1", "model": "swe-1.6",
                 "model_tier": "FREE", "harness": "devin-cli", "pool": "devin"},
            ],
            "escalation_workers": [
                {"slot_id": "esc", "model": "gpt-6.1-codex",
                 "model_tier": "PAID", "harness": "codex"},
            ],
        })

    def test_paid_slot_under_workers_is_dispatchable(self):
        adapter = self._adapter()
        slot = adapter.select_slot("PAID", "opus-exec")
        self.assertEqual(slot.slot_id, "opus-exec")
        self.assertEqual(slot.model_tier, "PAID")

    def test_free_slot_under_workers_stays_in_free_pool(self):
        adapter = self._adapter()
        self.assertEqual([s.slot_id for s in adapter.free_slots], ["devin1"])
        self.assertEqual(adapter.select_slot("FREE", "devin1").slot_id, "devin1")

    def test_escalation_slot_keeps_current_paid_behavior(self):
        adapter = self._adapter()
        self.assertIn("esc", [s.slot_id for s in adapter.paid_slots])
        self.assertEqual(adapter.select_slot("PAID", "esc").slot_id, "esc")

    def test_default_pools_without_config_unchanged(self):
        from worker_adapter import WorkerAdapter, FREE_MODEL_POOL, PAID_MODEL_POOL
        adapter = WorkerAdapter()
        self.assertEqual([s.model for s in adapter.free_slots], FREE_MODEL_POOL)
        self.assertEqual([s.model for s in adapter.paid_slots], PAID_MODEL_POOL)

    def test_empty_tier_pool_raises_keyerror_not_zerodivision(self):
        from worker_adapter import WorkerAdapter
        adapter = WorkerAdapter({
            "workers": [
                {"slot_id": "opus-exec", "model": "claude-opus-4-8[1m]",
                 "model_tier": "PAID", "harness": "claude"},
            ],
            "escalation_workers": [
                {"slot_id": "esc", "model": "gpt-6.1-codex",
                 "model_tier": "PAID", "harness": "codex"},
            ],
        })
        with self.assertRaises(KeyError):
            adapter.select_slot("FREE")


if __name__ == "__main__":
    unittest.main()
