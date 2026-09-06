"""Unit tests for trust_ledger.py (P1.6).

Temp-dir idiom from test_walk_state.py; the real trust.json is never touched. Framework is unittest
(pytest is broken repo-wide)."""
import io
import json
import os
import shutil
import tempfile
import unittest
from contextlib import redirect_stderr

import trust_ledger as tl


class TempLedgerCase(unittest.TestCase):
    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="trustledger_test_")
        self.path = os.path.join(self.tmpdir, "trust.json")

    def tearDown(self):
        shutil.rmtree(self.tmpdir, ignore_errors=True)

    def _record_many(self, task_class, results):
        for passed in results:
            tl.record(task_class, passed, path=self.path)

    def _residue(self):
        return [n for n in os.listdir(self.tmpdir) if n.endswith(".tmp") or n.endswith(".mutex")]


class TestPromotionLadder(TempLedgerCase):
    def test_unknown_class_is_watch(self):
        self.assertEqual(tl.tier("never-seen", path=self.path), tl.WATCH)

    def test_no_premature_promotion_at_9_runs(self):
        # 9 perfect runs is still WATCH -- QUEUE needs >= 10 runs regardless of rate.
        self._record_many("c", [True] * 9)
        self.assertEqual(tl.tier("c", path=self.path), tl.WATCH)

    def test_promotion_to_queue_at_exactly_10_and_90pct(self):
        # 1 fail + 9 pass = 10 runs, 9 passes (90%), last result pass -> QUEUE (not AUTO: <20 runs).
        self._record_many("c", [False] + [True] * 9)
        self.assertEqual(tl.tier("c", path=self.path), tl.QUEUE)

    def test_ten_runs_below_90pct_stays_watch(self):
        # 10 runs, 8 passes (80%) -> below the QUEUE rate floor -> WATCH.
        self._record_many("c", [False, False] + [True] * 8)
        self.assertEqual(tl.tier("c", path=self.path), tl.WATCH)

    def test_promotion_to_auto_at_exactly_20_and_95pct(self):
        # 1 fail + 19 pass = 20 runs, 19 passes (95%), last result pass -> AUTO.
        self._record_many("c", [False] + [True] * 19)
        self.assertEqual(tl.tier("c", path=self.path), tl.AUTO)

    def test_twenty_runs_below_95pct_is_queue_not_auto(self):
        # 20 runs, 18 passes (90%) -> qualifies for QUEUE but not AUTO.
        self._record_many("c", [False, False] + [True] * 18)
        self.assertEqual(tl.tier("c", path=self.path), tl.QUEUE)


class TestImmediateDemotion(TempLedgerCase):
    def test_demotion_from_auto_on_fail(self):
        self._record_many("c", [True] * 20)               # 20/20 = AUTO
        self.assertEqual(tl.tier("c", path=self.path), tl.AUTO)
        tl.record("c", False, path=self.path)             # 21 runs 20 pass = 95.2%, last fail
        self.assertEqual(tl.tier("c", path=self.path), tl.QUEUE)

    def test_demotion_from_queue_on_fail(self):
        self._record_many("c", [True] * 10)               # 10/10 = QUEUE (100% but <20 runs)
        self.assertEqual(tl.tier("c", path=self.path), tl.QUEUE)
        tl.record("c", False, path=self.path)             # 11 runs 10 pass = 90.9%, last fail
        self.assertEqual(tl.tier("c", path=self.path), tl.WATCH)

    def test_recovery_after_a_pass_restores_tier(self):
        self._record_many("c", [True] * 20)               # AUTO
        tl.record("c", False, path=self.path)             # demoted to QUEUE (last fail)
        self.assertEqual(tl.tier("c", path=self.path), tl.QUEUE)
        tl.record("c", True, path=self.path)              # 22 runs 21 pass = 95.5%, last pass -> AUTO
        self.assertEqual(tl.tier("c", path=self.path), tl.AUTO)

    def test_demotion_emits_stderr_alert(self):
        self._record_many("c", [True] * 20)
        buf = io.StringIO()
        with redirect_stderr(buf):
            post = tl.record("c", False, path=self.path)
        self.assertEqual(post, tl.QUEUE)
        self.assertIn("ALERT", buf.getvalue())
        self.assertIn("auto -> queue", buf.getvalue())

    def test_fail_from_watch_emits_no_alert(self):
        buf = io.StringIO()
        with redirect_stderr(buf):
            tl.record("c", False, path=self.path)          # WATCH stays WATCH, no demotion
        self.assertNotIn("ALERT", buf.getvalue())


class TestTierMapAndStorage(TempLedgerCase):
    def test_tier_map_reflects_all_classes(self):
        self._record_many("hi", [True] * 20)               # AUTO
        self._record_many("mid", [True] * 10)              # QUEUE
        self._record_many("lo", [True] * 3)                # WATCH
        self.assertEqual(tl.tier_map(path=self.path),
                         {"hi": tl.AUTO, "mid": tl.QUEUE, "lo": tl.WATCH})

    def test_stored_truth_is_counts_not_tier(self):
        self._record_many("c", [True, False, True])
        with open(self.path, encoding="utf-8") as fh:
            data = json.load(fh)
        rec = data["classes"]["c"]
        self.assertEqual(rec["runs"], 3)
        self.assertEqual(rec["passes"], 2)
        self.assertEqual(rec["last_result"], tl.RESULT_PASS)
        self.assertNotIn("tier", rec)

    def test_empty_ledger_tier_map_is_empty(self):
        self.assertEqual(tl.tier_map(path=self.path), {})


class TestAtomicity(TempLedgerCase):
    def test_no_tmp_or_mutex_residue(self):
        self._record_many("c", [True, False, True, True])
        self.assertEqual(self._residue(), [])

    def test_malformed_ledger_raises(self):
        with open(self.path, "w", encoding="utf-8") as fh:
            fh.write("{ not json")
        with self.assertRaises(tl.TrustLedgerError):
            tl.tier_map(path=self.path)


if __name__ == "__main__":
    unittest.main()
