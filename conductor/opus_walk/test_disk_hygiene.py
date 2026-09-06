"""Unit tests for disk_hygiene - the opus-walk F: free-space guard + between-plan reclaim.

Framework: stdlib unittest. Every test injects fakes (free_fn / reclaim_fn / rmtree_fn / run_fn /
state_loader) so nothing touches the real filesystem, git, subprocess, or the /mnt/f drive. The
worktree-skip test uses a throwaway temp dir (real dirs so is_dir()/resolve()/parent all work) but
injects an rmtree_fn recorder so no directory is actually deleted.

Contract under test:
  * guard OFF via env                     -> spawn, no reclaim.
  * F: >= soft                            -> spawn, reclaim NOT called.
  * hard-floor (reclaim cannot recover)   -> halt, below_hard True.
  * soft band, reclaim recovers >= soft   -> reclaim called, spawn.
  * recovered above hard but below soft   -> halt, below_hard False.
  * an in-flight worktree is SKIPPED, a non-in-flight one is reaped.
  * reclaim is fail-soft on an rmtree error (returns a dict, never raises).
  * path guard refuses a dir not directly under the expected parent.
"""
import os
import tempfile
import unittest
from pathlib import Path

import disk_hygiene as dh


def _quiet(_msg):
    """Swallow log output in tests."""
    return None


def _const_free(value):
    """A free_fn that always returns the same GB number."""
    return lambda _path: value


def _seq_free(values):
    """A free_fn that returns successive values (before-reclaim, after-reclaim, ...)."""
    it = iter(values)
    return lambda _path: next(it)


class TestGuardEnabled(unittest.TestCase):
    def setUp(self):
        self._orig = os.environ.get(dh.GUARD_ENV)

    def tearDown(self):
        if self._orig is None:
            os.environ.pop(dh.GUARD_ENV, None)
        else:
            os.environ[dh.GUARD_ENV] = self._orig

    def test_on_by_default(self):
        os.environ.pop(dh.GUARD_ENV, None)
        self.assertTrue(dh.guard_enabled())

    def test_off_disables(self):
        os.environ[dh.GUARD_ENV] = "off"
        self.assertFalse(dh.guard_enabled())

    def test_off_disables_disk_guard(self):
        os.environ[dh.GUARD_ENV] = "off"
        called = []
        decision = dh.disk_guard(free_fn=_const_free(1.0),
                                 reclaim_fn=lambda *a, **k: called.append(1) or {},
                                 log=_quiet)
        self.assertEqual(decision["action"], "spawn")
        self.assertIsNone(decision["reclaim"])
        self.assertEqual(called, [])  # reclaim never runs when the guard is off


class TestInFlightPlanIds(unittest.TestCase):
    def _empty_pending(self, td: str):
        # Isolate from the live pending-lands ledger (parked plans pollute state-only asserts).
        pl = Path(td) / "pending-lands.json"
        pl.write_text('{"schema_version": 1, "lands": []}\n', encoding="utf-8")
        return pl

    def test_queue_and_cursor(self):
        state = {"queue": ["H-10", "C-15"], "cursor": {"plan_id": "I-23"}}
        with tempfile.TemporaryDirectory() as td:
            self.assertEqual(
                dh.in_flight_plan_ids(state, pending_path=self._empty_pending(td)),
                {"H-10", "C-15", "I-23"},
            )

    def test_empty(self):
        with tempfile.TemporaryDirectory() as td:
            pl = self._empty_pending(td)
            self.assertEqual(
                dh.in_flight_plan_ids(
                    {"queue": [], "cursor": {"plan_id": None}}, pending_path=pl
                ),
                set(),
            )
            self.assertEqual(dh.in_flight_plan_ids({}, pending_path=pl), set())


class TestDiskGuardThresholds(unittest.TestCase):
    def setUp(self):
        os.environ.pop(dh.GUARD_ENV, None)

    def test_above_soft_spawns_without_reclaim(self):
        called = []
        decision = dh.disk_guard(
            free_fn=_const_free(dh.SOFT_THRESHOLD_GB + 5),
            reclaim_fn=lambda *a, **k: called.append(1) or {}, log=_quiet)
        self.assertEqual(decision["action"], "spawn")
        self.assertIsNone(decision["reclaim"])
        self.assertEqual(called, [])

    def test_unreadable_free_fails_soft_to_spawn(self):
        decision = dh.disk_guard(free_fn=_const_free(-1.0),
                                 reclaim_fn=lambda *a, **k: {}, log=_quiet)
        self.assertEqual(decision["action"], "spawn")
        self.assertIn("unreadable", decision["reason"])

    def test_soft_band_reclaim_recovers_spawns(self):
        # 20GB (below soft, above hard) before; reclaim brings it to 40GB -> spawn.
        called = []
        decision = dh.disk_guard(
            free_fn=_seq_free([20.0, 40.0]),
            reclaim_fn=lambda *a, **k: called.append(1) or {"freed_gb": 20.0,
                                                            "f_free_before": 20.0,
                                                            "f_free_after": 40.0, "reaped": []},
            log=_quiet)
        self.assertEqual(decision["action"], "spawn")
        self.assertEqual(called, [1])  # reclaim ran once
        self.assertIsNotNone(decision["reclaim"])

    def test_hard_floor_halts_below_hard(self):
        # 5GB before, reclaim cannot recover (still 6GB) -> halt, below_hard True.
        decision = dh.disk_guard(
            free_fn=_seq_free([5.0, 6.0]),
            reclaim_fn=lambda *a, **k: {"freed_gb": 1.0, "f_free_before": 5.0,
                                        "f_free_after": 6.0, "reaped": []},
            log=_quiet)
        self.assertEqual(decision["action"], "halt")
        self.assertTrue(decision["below_hard"])

    def test_recovered_above_hard_below_soft_halts(self):
        # 15GB before, reclaim brings it to 20GB: above hard(12) but below soft(30) -> halt, not below_hard.
        decision = dh.disk_guard(
            free_fn=_seq_free([15.0, 20.0]),
            reclaim_fn=lambda *a, **k: {"freed_gb": 5.0, "f_free_before": 15.0,
                                        "f_free_after": 20.0, "reaped": []},
            log=_quiet)
        self.assertEqual(decision["action"], "halt")
        self.assertFalse(decision["below_hard"])


class TestSafeRmtree(unittest.TestCase):
    def test_refuses_path_not_under_parent(self):
        removed = []
        ok = dh._safe_rmtree(Path("/tmp/some-target"), Path("/other/parent"),
                             _quiet, rmtree_fn=lambda p: removed.append(p))
        self.assertFalse(ok)
        self.assertEqual(removed, [])  # nothing removed when the parent guard fails

    def test_removes_direct_child(self):
        removed = []
        base = Path(tempfile.mkdtemp(prefix="dh_rm_"))
        try:
            child = base / "child"
            child.mkdir()
            ok = dh._safe_rmtree(child, base, _quiet, rmtree_fn=lambda p: removed.append(p))
            self.assertTrue(ok)
            self.assertEqual(removed, [str(child)])
        finally:
            import shutil
            shutil.rmtree(base, ignore_errors=True)


class TestReclaim(unittest.TestCase):
    def setUp(self):
        # Redirect the module's reap targets at throwaway temp dirs so no real path is ever touched.
        self._orig_cargo = dh.CARGO_TARGET_DIR
        self._orig_worktrees = dh.WORKTREE_DIRS
        self.tmp = Path(tempfile.mkdtemp(prefix="dh_reclaim_"))
        # cargo target -> a NONEXISTENT temp path so the cargo reap is a no-op in these tests.
        dh.CARGO_TARGET_DIR = self.tmp / "cargo-target-elai"
        self.base = self.tmp / "worktrees"
        self.base.mkdir()
        dh.WORKTREE_DIRS = (self.base,)

    def tearDown(self):
        dh.CARGO_TARGET_DIR = self._orig_cargo
        dh.WORKTREE_DIRS = self._orig_worktrees
        import shutil
        shutil.rmtree(self.tmp, ignore_errors=True)

    def _mk_worktree(self, name):
        (self.base / name).mkdir()

    def test_in_flight_worktree_is_skipped(self):
        self._mk_worktree("H-10_ab12cd")   # in-flight -> keep
        self._mk_worktree("C-15_ff0099")   # not in-flight -> reap
        removed = []
        rc = dh.reclaim({"H-10"},
                        free_fn=_const_free(40.0),
                        rmtree_fn=lambda p: removed.append(p),
                        run_fn=lambda *a, **k: type("R", (), {"returncode": 0, "stderr": ""})(),
                        log=_quiet)
        self.assertEqual(removed, [str(self.base / "C-15_ff0099")])
        self.assertIn(str(self.base / "C-15_ff0099"), rc["reaped"])
        self.assertNotIn(str(self.base / "H-10_ab12cd"), rc["reaped"])

    def test_none_protected_skips_worktree_reap(self):
        self._mk_worktree("C-15_ff0099")
        removed = []
        rc = dh.reclaim(None,
                        free_fn=_const_free(40.0),
                        rmtree_fn=lambda p: removed.append(p),
                        run_fn=lambda *a, **k: type("R", (), {"returncode": 0, "stderr": ""})(),
                        log=_quiet)
        self.assertEqual(removed, [])  # unknown in-flight set -> no worktree reap
        self.assertEqual(rc["reaped"], [])

    def test_integrator_worktree_is_skipped(self):
        self._mk_worktree("integrate_merge_1")
        self._mk_worktree("D-4_a1b2c3")
        removed = []
        dh.reclaim(set(),
                   free_fn=_const_free(40.0),
                   rmtree_fn=lambda p: removed.append(p),
                   run_fn=lambda *a, **k: type("R", (), {"returncode": 0, "stderr": ""})(),
                   log=_quiet)
        self.assertEqual(removed, [str(self.base / "D-4_a1b2c3")])  # integrator dir untouched

    def test_keep_for_land_marker_skips_reap(self):
        """Worktrees carrying .KEEP_FOR_LAND must never be reaped (P0 land handoff)."""
        keep_name = "opuswalk-E-11-EXECUTE-landcand"
        reap_name = "opuswalk-E-99-EXECUTE-ephemeral"
        self._mk_worktree(keep_name)
        self._mk_worktree(reap_name)
        (self.base / keep_name / dh.KEEP_LAND_MARKER).write_text(
            "plan=E-11\ntask=landcand\n", encoding="utf-8")
        removed = []
        rc = dh.reclaim(
            set(),
            free_fn=_const_free(40.0),
            rmtree_fn=lambda p: removed.append(p),
            run_fn=lambda *a, **k: type("R", (), {"returncode": 0, "stderr": ""})(),
            log=_quiet,
        )
        self.assertEqual(removed, [str(self.base / reap_name)])
        self.assertNotIn(str(self.base / keep_name), rc["reaped"])
        self.assertTrue((self.base / keep_name).is_dir())
        self.assertTrue(dh._has_keep_for_land(self.base / keep_name))

    def test_fail_soft_on_rmtree_error(self):
        self._mk_worktree("C-15_ff0099")

        def _boom(_p):
            raise OSError("permission denied")

        # reclaim must NOT raise; it returns a dict with the failed dir NOT in reaped.
        rc = dh.reclaim(set(),
                        free_fn=_const_free(40.0),
                        rmtree_fn=_boom,
                        run_fn=lambda *a, **k: type("R", (), {"returncode": 0, "stderr": ""})(),
                        log=_quiet)
        self.assertIsInstance(rc, dict)
        self.assertEqual(rc["reaped"], [])
        self.assertIn("freed_gb", rc)

    def test_fail_soft_on_subprocess_error(self):
        # git-prune / fstrim raising must not crash reclaim.
        def _boom_run(*_a, **_k):
            raise OSError("no such binary")

        rc = dh.reclaim(set(),
                        free_fn=_const_free(40.0),
                        rmtree_fn=lambda p: None,
                        run_fn=_boom_run,
                        log=_quiet)
        self.assertIsInstance(rc, dict)


if __name__ == "__main__":
    unittest.main()
