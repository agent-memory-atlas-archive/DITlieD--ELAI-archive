#!/usr/bin/env python3
"""Unit tests for opus_walk_chain.py - the T6-entry10 auto-restart-with-backoff wrapper.

Framework: stdlib unittest. Every test injects run_one / wait / sync_token so nothing spawns a real
supervisor subprocess or sleeps for real, and the STOP sentinel + chain-result paths are redirected to
a temp dir so the live walk is never touched. The contract under test: restart on a clean no-work halt
with an escalating 15/30/60min backoff, reset the backoff on progress, and NEVER restart on the STOP
sentinel, a DISK_LOW-twice, a non-restart halt, env-off, or the consecutive-no-progress cap."""
import json
import os
import shutil
import tempfile
import types
import unittest
from pathlib import Path

import opus_walk_chain as chain


class ChainLoopCase(unittest.TestCase):
    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="opuschain_test_")
        self._orig_sentinel = chain.STOP_SENTINEL
        self._orig_result = chain.CHAIN_RESULT_PATH
        self._orig_cap = chain.MAX_NO_PROGRESS_RESTARTS
        self._orig_emit = chain._emit
        chain.STOP_SENTINEL = Path(self.tmpdir) / "OPUS_WALK_STOP"
        chain.CHAIN_RESULT_PATH = Path(self.tmpdir) / "last-chain-result.json"
        chain._emit = lambda *a, **k: None  # no control.db writes in unit tests
        self._orig_autorestart = os.environ.get(chain.AUTORESTART_ENV)
        os.environ.pop(chain.AUTORESTART_ENV, None)  # default = ON

    def tearDown(self):
        chain.STOP_SENTINEL = self._orig_sentinel
        chain.CHAIN_RESULT_PATH = self._orig_result
        chain.MAX_NO_PROGRESS_RESTARTS = self._orig_cap
        chain._emit = self._orig_emit
        if self._orig_autorestart is None:
            os.environ.pop(chain.AUTORESTART_ENV, None)
        else:
            os.environ[chain.AUTORESTART_ENV] = self._orig_autorestart
        shutil.rmtree(self.tmpdir, ignore_errors=True)

    def _make_sentinel(self):
        chain.STOP_SENTINEL.write_text("stop", encoding="utf-8")

    @staticmethod
    def _result(halt_class, made_progress=False, rc=0):
        return {"halt_class": halt_class, "made_progress": made_progress, "rc": rc, "detail": {}}

    def _scripted_run_one(self, script):
        it = iter(script)
        calls = []

        def run_one():
            item = next(it)
            calls.append(item["halt_class"])
            return item

        run_one.calls = calls
        return run_one


class TestRestartDecision(ChainLoopCase):
    def test_clean_no_work_restarts(self):
        for hc in ("CHAIN_NO_WORK", "NO_SPAWN", "QUORUM_QUIET"):
            do, reason = chain._restart_decision(hc, None)
            self.assertTrue(do)
            self.assertEqual(reason, hc)

    def test_disk_low_once_restarts_twice_stops(self):
        self.assertEqual(chain._restart_decision("DISK_LOW", None), (True, "disk_low_once"))
        self.assertEqual(chain._restart_decision("DISK_LOW", "DISK_LOW"),
                         (False, "disk_low_twice_consecutive"))

    def test_error_and_stop_and_unknown_never_restart(self):
        # FAST_FAIL_STORM may rescue once per 6h (P0 token-auth); tested separately.
        for hc in ("STOPPED", "ERROR_WALKSTATE", "UNKNOWN"):
            do, _ = chain._restart_decision(hc, None)
            self.assertFalse(do, hc)

    def test_fast_fail_storm_rescues_once(self):
        import time
        stamp = Path("/tmp/opus-token-rescue-stamp")
        try:
            if stamp.is_file():
                stamp.unlink()
        except OSError:
            pass
        do, reason = chain._restart_decision("FAST_FAIL_STORM", None)
        self.assertTrue(do)
        self.assertEqual(reason, "fast_fail_token_rescue_once")
        stamp.write_text(str(int(time.time())), encoding="utf-8")
        do2, _ = chain._restart_decision("FAST_FAIL_STORM", None)
        self.assertFalse(do2)

    def test_fast_fail_rescue_uses_60s_backoff(self):
        import time
        stamp = Path("/tmp/opus-token-rescue-stamp")
        try:
            if stamp.is_file():
                stamp.unlink()
        except OSError:
            pass
        script = [
            self._result("FAST_FAIL_STORM", rc=1),
            self._result("STOPPED"),
        ]
        run_one = self._scripted_run_one(script)
        waits = []
        syncs = []

        def sync_token(force=False):
            syncs.append(bool(force))

        rc = chain.run_chain_loop(
            run_one=run_one,
            wait=lambda s: (waits.append(s), True)[1],
            sync_token=sync_token,
        )
        self.assertEqual(waits, [60])
        self.assertEqual(syncs, [True])  # force sync on rescue restart
        self.assertEqual(run_one.calls, ["FAST_FAIL_STORM", "STOPPED"])
        self.assertEqual(rc, 0)


class TestBackoffSequence(ChainLoopCase):
    def test_escalates_then_resets_on_progress(self):
        script = [
            self._result("CHAIN_NO_WORK"),                       # -> 900
            self._result("NO_SPAWN"),                            # -> 1800
            self._result("QUORUM_QUIET"),                        # -> 3600 (cap)
            self._result("CHAIN_NO_WORK"),                       # -> 3600 (held at cap)
            self._result("CHAIN_NO_WORK", made_progress=True),   # progress -> reset -> 900
            self._result("CHAIN_NO_WORK"),                       # -> 1800
            self._result("STOPPED"),                             # non-restart -> loop returns
        ]
        run_one = self._scripted_run_one(script)
        waits = []
        syncs = []
        rc = chain.run_chain_loop(
            run_one=run_one,
            wait=lambda s: (waits.append(s), True)[1],
            sync_token=lambda: syncs.append(1))
        self.assertEqual(waits, [900, 1800, 3600, 3600, 900, 1800])
        self.assertEqual(len(syncs), 6)          # token sync once per restart
        self.assertEqual(run_one.calls[-1], "STOPPED")
        self.assertEqual(rc, 0)

    def test_progress_chain_resets_no_progress_counter(self):
        # a progress chain in the middle must clear the consecutive-no-progress count so the cap is not
        # tripped by non-consecutive no-work halts.
        chain.MAX_NO_PROGRESS_RESTARTS = 2
        script = [
            self._result("CHAIN_NO_WORK"),                     # consec 1
            self._result("CHAIN_NO_WORK", made_progress=True), # reset consec -> 0
            self._result("CHAIN_NO_WORK"),                     # consec 1
            self._result("CHAIN_NO_WORK"),                     # consec 2
            self._result("STOPPED"),
        ]
        run_one = self._scripted_run_one(script)
        rc = chain.run_chain_loop(run_one=run_one, wait=lambda s: True, sync_token=lambda: None)
        self.assertEqual(run_one.calls[-1], "STOPPED")  # reached the stop, cap never tripped
        self.assertEqual(rc, 0)


class TestSentinelKillsLoop(ChainLoopCase):
    def test_sentinel_present_before_any_chain(self):
        self._make_sentinel()

        def run_one():
            raise AssertionError("run_one must not be called when the sentinel is already present")

        rc = chain.run_chain_loop(run_one=run_one, wait=lambda s: True, sync_token=lambda: None)
        self.assertEqual(rc, 0)

    def test_sentinel_after_chain_exits(self):
        # a clean no-work halt would normally restart, but the sentinel appearing after the chain wins.
        def run_one():
            self._make_sentinel()
            return self._result("CHAIN_NO_WORK")

        waits = []
        rc = chain.run_chain_loop(run_one=run_one, wait=lambda s: (waits.append(s), True)[1],
                                  sync_token=lambda: None)
        self.assertEqual(rc, 0)
        self.assertEqual(waits, [])  # never reached the backoff

    def test_sentinel_during_backoff(self):
        # wait() returns False to simulate the sentinel appearing mid-backoff -> the loop exits.
        run_one = self._scripted_run_one([self._result("CHAIN_NO_WORK")])
        rc = chain.run_chain_loop(run_one=run_one, wait=lambda s: False, sync_token=lambda: None)
        self.assertEqual(rc, 0)
        self.assertEqual(run_one.calls, ["CHAIN_NO_WORK"])  # ran exactly one chain


class TestDiskLowTwice(ChainLoopCase):
    def test_two_consecutive_disk_low_stops(self):
        script = [self._result("DISK_LOW", rc=0), self._result("DISK_LOW", rc=0)]
        run_one = self._scripted_run_one(script)
        waits = []
        rc = chain.run_chain_loop(run_one=run_one, wait=lambda s: (waits.append(s), True)[1],
                                  sync_token=lambda: None)
        self.assertEqual(waits, [900])            # first DISK_LOW restarted once, second stopped
        self.assertEqual(run_one.calls, ["DISK_LOW", "DISK_LOW"])
        self.assertEqual(rc, 0)

    def test_disk_low_broken_by_progress_restarts_again(self):
        # DISK_LOW, then a progress chain, then DISK_LOW again -> the second DISK_LOW is NOT consecutive
        # (prev halt was CHAIN_NO_WORK) so it restarts.
        script = [
            self._result("DISK_LOW"),
            self._result("CHAIN_NO_WORK", made_progress=True),
            self._result("DISK_LOW"),
            self._result("STOPPED"),
        ]
        run_one = self._scripted_run_one(script)
        waits = []
        chain.run_chain_loop(run_one=run_one, wait=lambda s: (waits.append(s), True)[1],
                             sync_token=lambda: None)
        self.assertEqual(len(waits), 3)  # disk_low_once, progress-reset(900), disk_low_once again


class TestEnvOff(ChainLoopCase):
    def test_autorestart_off_runs_once(self):
        os.environ[chain.AUTORESTART_ENV] = "off"
        run_one = self._scripted_run_one([self._result("CHAIN_NO_WORK", rc=0)])
        waits = []
        rc = chain.run_chain_loop(run_one=run_one, wait=lambda s: (waits.append(s), True)[1],
                                  sync_token=lambda: None)
        self.assertEqual(run_one.calls, ["CHAIN_NO_WORK"])  # exactly one chain
        self.assertEqual(waits, [])                          # no restart
        self.assertEqual(rc, 0)


class TestRestartCap(ChainLoopCase):
    def test_gives_up_after_cap(self):
        chain.MAX_NO_PROGRESS_RESTARTS = 3
        # an endless no-work stream; the loop must give up after the cap rather than loop forever.
        def run_one():
            run_one.n = getattr(run_one, "n", 0) + 1
            return self._result("CHAIN_NO_WORK")

        waits = []
        rc = chain.run_chain_loop(run_one=run_one, wait=lambda s: (waits.append(s), True)[1],
                                  sync_token=lambda: None)
        # 3 restarts performed (consec 1..3), the 4th no-progress chain (consec 4 > 3) gives up.
        self.assertEqual(len(waits), 3)
        self.assertEqual(run_one.n, 4)
        self.assertEqual(rc, 0)


class TestRunOneChain(ChainLoopCase):
    def test_reads_supervisor_stamp(self):
        def fake_spawn(cmd, cwd=None):
            chain.CHAIN_RESULT_PATH.write_text(
                json.dumps({"halt_class": "CHAIN_NO_WORK", "made_progress": True, "detail": {"a": 1}}),
                encoding="utf-8")
            return types.SimpleNamespace(returncode=0)

        res = chain._run_one_chain(spawn=fake_spawn)
        self.assertEqual(res["halt_class"], "CHAIN_NO_WORK")
        self.assertTrue(res["made_progress"])
        self.assertEqual(res["rc"], 0)

    def test_missing_stamp_is_unknown(self):
        # a chain that dies WITHOUT stamping a result -> UNKNOWN (fail-safe: no restart on a silent die).
        def fake_spawn(cmd, cwd=None):
            return types.SimpleNamespace(returncode=1)  # writes no stamp

        res = chain._run_one_chain(spawn=fake_spawn)
        self.assertEqual(res["halt_class"], "UNKNOWN")
        self.assertFalse(res["made_progress"])
        self.assertEqual(res["rc"], 1)

    def test_stale_stamp_removed_before_run(self):
        # a pre-existing stamp from a prior chain must not be read as this chain's result.
        chain.CHAIN_RESULT_PATH.write_text(
            json.dumps({"halt_class": "QUORUM_QUIET", "made_progress": True}), encoding="utf-8")

        def fake_spawn(cmd, cwd=None):
            return types.SimpleNamespace(returncode=2)  # dies, writes nothing new

        res = chain._run_one_chain(spawn=fake_spawn)
        self.assertEqual(res["halt_class"], "UNKNOWN")  # stale QUORUM_QUIET was cleared, not reused


if __name__ == "__main__":
    unittest.main()


# (appended by fix_timeout_restart.py)
