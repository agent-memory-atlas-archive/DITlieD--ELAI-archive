"""Unit tests for opus_supervisor.py — the no-work / reconcile / pre-spawn gates.

All tests operate on temp-directory copies of the state file and a temp STOP
sentinel; the live walk-state.json and the real OPUS_WALK_STOP path are never
touched. The bridge health probe is injected as a stub so no test hits the
network, and no test spawns a paid Opus turn.
"""
import json
import os
import shutil
import tempfile
import unittest
from pathlib import Path

import walk_state as ws
import opus_supervisor as sup

SEED = {
    "schema_version": 1,
    "walk_id": "opus-walk-test",
    "lock": None,
    "budget": {
        "dispatches_this_session": 0,
        "dispatch_cap": 30,
        "plans_this_session": 0,
        "plans_per_session_cap": 3,
        "sessions_completed": 0,
        "concurrency": 2,
        "degrade_on_429_to": 1,
    },
    "cursor": {"plan_id": None, "stage": None, "pickup": None},
    "queue": [],
    "escalation_queue": [],
    "decisions": [],
    "integration_smoke": {"last_after_plan": None, "cadence_plans": 3},
    "notes": "test seed",
}


class TempStateCase(unittest.TestCase):
    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="opussup_test_")
        self.path = os.path.join(self.tmpdir, "walk-state.json")
        # Point the STOP sentinel at the temp dir so tests never see the real one.
        self._orig_sentinel = sup.STOP_SENTINEL
        sup.STOP_SENTINEL = Path(self.tmpdir) / "OPUS_WALK_STOP"

    def tearDown(self):
        sup.STOP_SENTINEL = self._orig_sentinel
        shutil.rmtree(self.tmpdir, ignore_errors=True)

    def _write_state(self, queue=None, cursor=None, escalation=None):
        state = json.loads(json.dumps(SEED))
        if queue is not None:
            state["queue"] = queue
        if cursor is not None:
            state["cursor"] = cursor
        if escalation is not None:
            state["escalation_queue"] = escalation
        with open(self.path, "w", encoding="utf-8") as f:
            json.dump(state, f)
        return state

    def _write_done_row(self, plan_id):
        plans_dir = os.path.join(self.tmpdir, "plan-status")
        os.makedirs(plans_dir, exist_ok=True)
        with open(os.path.join(plans_dir, f"{plan_id}.json"), "w", encoding="utf-8") as f:
            json.dump({"plan_id": plan_id, "status": "DONE", "commit_sha": "abc",
                       "gate_evidence_ref": "ref", "last_run_iso": "2026-07-10T00:00:00+00:00"}, f)

    def _make_sentinel(self):
        sup.STOP_SENTINEL.write_text("stop", encoding="utf-8")


class TestNoActionableWork(TempStateCase):
    def test_true_on_h28_burn_queue(self):
        # Exact H-28 burn: a DONE plan left sitting in the queue must NOT read as work.
        self._write_state(queue=["H-28"])
        self._write_done_row("H-28")
        self.assertTrue(sup._no_actionable_work(self.path))

    def test_true_on_h28_burn_cursor(self):
        # A DONE plan left on the cursor (queue empty) must NOT read as work.
        self._write_state(queue=[], cursor={"plan_id": "H-28", "stage": "done", "pickup": "x"})
        self._write_done_row("H-28")
        self.assertTrue(sup._no_actionable_work(self.path))

    def test_false_on_genuinely_actionable(self):
        # A queued plan with no DONE row is real work.
        self._write_state(queue=["Y"])
        self.assertFalse(sup._no_actionable_work(self.path))

    def test_true_on_empty_drained(self):
        self._write_state(queue=[], cursor={"plan_id": None, "stage": None, "pickup": None})
        self.assertTrue(sup._no_actionable_work(self.path))

    def test_true_on_escalated_cursor(self):
        self._write_state(queue=[], cursor={"plan_id": "I-33", "stage": "precheck", "pickup": "x"},
                          escalation=[{"plan_id": "I-33", "reason": "greenfield", "attempts": 2,
                                       "gate_json_ref": "", "ts": ""}])
        self.assertTrue(sup._no_actionable_work(self.path))

    def test_malformed_state_reraises(self):
        with open(self.path, "w", encoding="utf-8") as f:
            f.write("{ not json")
        with self.assertRaises(ws.WalkStateError):
            sup._no_actionable_work(self.path)


class TestReconcile(TempStateCase):
    def test_dequeues_done_plan(self):
        self._write_state(queue=["X", "Y"], cursor={"plan_id": "X", "stage": "done", "pickup": "x"})
        self._write_done_row("X")
        reconciled = sup._reconcile_done_plans(self.path)
        self.assertEqual(reconciled, ["X"])
        state = ws.load(self.path)
        self.assertEqual(state["queue"], ["Y"])
        self.assertEqual(state["cursor"], {"plan_id": None, "stage": None, "pickup": None})

    def test_noop_when_nothing_done(self):
        self._write_state(queue=["Y"], cursor={"plan_id": "Y", "stage": "EXECUTING", "pickup": "run"})
        reconciled = sup._reconcile_done_plans(self.path)
        self.assertEqual(reconciled, [])
        self.assertEqual(ws.load(self.path)["queue"], ["Y"])

    def test_reconcile_then_no_work_true(self):
        # The end-to-end H-28 self-heal: reconcile dequeues the DONE plan, then no-work is True.
        self._write_state(queue=["H-28"], cursor={"plan_id": "H-28", "stage": "done", "pickup": "x"})
        self._write_done_row("H-28")
        sup._reconcile_done_plans(self.path)
        self.assertTrue(sup._no_actionable_work(self.path))


class TestShouldSpawn(TempStateCase):
    def test_refuses_empty_work(self):
        state = self._write_state(queue=[], cursor={"plan_id": None, "stage": None, "pickup": None})
        ok, reason = sup._should_spawn(state, probe=lambda: True)
        self.assertFalse(ok)
        self.assertIn("no actionable work", reason)

    def test_refuses_dead_bridge(self):
        state = self._write_state(queue=["Y"])
        ok, reason = sup._should_spawn(state, probe=lambda: False)
        self.assertFalse(ok)
        self.assertIn("bridge", reason)

    def test_refuses_stop_sentinel(self):
        self._make_sentinel()
        state = self._write_state(queue=["Y"])
        ok, reason = sup._should_spawn(state, probe=lambda: True)
        self.assertFalse(ok)
        self.assertIn("sentinel", reason)

    def test_passes_live_state(self):
        state = self._write_state(queue=["Y"])
        ok, reason = sup._should_spawn(state, probe=lambda: True)
        self.assertTrue(ok)

    def test_passes_actionable_cursor(self):
        state = self._write_state(queue=[], cursor={"plan_id": "Y", "stage": "EXECUTING",
                                                    "pickup": "run"})
        ok, reason = sup._should_spawn(state, probe=lambda: True)
        self.assertTrue(ok)

    def test_refuses_done_cursor_no_queue(self):
        # cursor stage 'done' is not actionable even though plan_id is set
        state = self._write_state(queue=[], cursor={"plan_id": "Y", "stage": "done", "pickup": "x"})
        ok, reason = sup._should_spawn(state, probe=lambda: True)
        self.assertFalse(ok)

    def test_f6_reconcile_before_work_refuses_landed_plan(self):
        # F-6 race: a plan that LANDED (plan-status DONE) but is still sitting in the queue must NOT
        # wake a paid turn. With the reconcile injected, _should_spawn dequeues it and re-checks on the
        # reconciled truth -> refuses. This is the gate being independently DONE-aware.
        self._write_state(queue=["H-28"], cursor={"plan_id": "H-28", "stage": "done", "pickup": "x"})
        self._write_done_row("H-28")
        stale = ws.load(self.path)  # loaded BEFORE reconcile: still shows H-28 queued (the race window)
        self.assertEqual(stale["queue"], ["H-28"])
        ok, reason = sup._should_spawn(stale, probe=lambda: True,
                                       reconcile=sup._reconcile_done_plans, path=self.path)
        self.assertFalse(ok)
        self.assertIn("no actionable work", reason)
        # the landed plan was dequeued as part of the gate
        self.assertEqual(ws.load(self.path)["queue"], [])

    def test_f6_reconcile_keeps_real_work(self):
        # A genuinely-undone queued plan survives the reconcile-in-gate and still permits the spawn.
        self._write_state(queue=["H-28", "Z"], cursor={"plan_id": "H-28", "stage": "done"})
        self._write_done_row("H-28")  # H-28 landed; Z is real work
        stale = ws.load(self.path)
        ok, reason = sup._should_spawn(stale, probe=lambda: True,
                                       reconcile=sup._reconcile_done_plans, path=self.path)
        self.assertTrue(ok)
        self.assertEqual(ws.load(self.path)["queue"], ["Z"])  # only the DONE one was dequeued


class TestQuorumGateInSupervisor(TempStateCase):
    """P1.4 production wiring: run_supervisor calls the quorum gate AFTER _should_spawn passes and
    BEFORE the paid boot. A QUIET quorum skips the spawn; a WAKE quorum lets it through. The gate
    helpers are monkeypatched (the injected vote fn) so no test hits the bridge or spawns Opus."""

    def _neutralize(self, quorum_return, stop_after_spawn=False):
        """Patch the pre-quorum gates open + record spawns; return (spawns, restore_fn)."""
        self._orig = {
            "WALK_STATE": sup.WALK_STATE,
            "reconcile": sup._reconcile_done_plans,
            "nowork": sup._no_actionable_work,
            "should": sup._should_spawn,
            "quorum": sup._quorum_gate,
            "spawn": sup._spawn_opus_turn,
            "emit": sup._emit,
            "sleep": sup.INTER_TURN_SLEEP_S,
        }
        spawns = []
        sup.WALK_STATE = Path(self.path)
        sup._reconcile_done_plans = lambda *a, **k: []
        sup._no_actionable_work = lambda *a, **k: False
        sup._should_spawn = lambda *a, **k: (True, "ok")
        sup._quorum_gate = lambda state: quorum_return

        def fake_spawn(sid, budget):
            spawns.append(sid)
            if stop_after_spawn:
                self._make_sentinel()
            return {"rc": 0, "timed_out": False, "stdout_tail": "", "stderr_tail": ""}

        sup._spawn_opus_turn = fake_spawn
        sup._emit = lambda *a, **k: None
        sup.INTER_TURN_SLEEP_S = 0
        return spawns

    def _restore(self):
        sup.WALK_STATE = self._orig["WALK_STATE"]
        sup._reconcile_done_plans = self._orig["reconcile"]
        sup._no_actionable_work = self._orig["nowork"]
        sup._should_spawn = self._orig["should"]
        sup._quorum_gate = self._orig["quorum"]
        sup._spawn_opus_turn = self._orig["spawn"]
        sup._emit = self._orig["emit"]
        sup.INTER_TURN_SLEEP_S = self._orig["sleep"]

    def test_quiet_quorum_skips_paid_spawn(self):
        self._write_state(queue=["Y"])
        spawns = self._neutralize((False, {"reason": "QUORUM_QUIET", "yes": 0, "no": 3}))
        try:
            rc = sup.run_supervisor()
        finally:
            self._restore()
        self.assertEqual(rc, 0)
        self.assertEqual(spawns, [])  # the paid Opus boot was skipped

    def test_wake_quorum_permits_spawn(self):
        self._write_state(queue=["Y"])
        spawns = self._neutralize(
            (True, {"reason": "QUORUM_WAKE", "yes": 3, "no": 0}),
            stop_after_spawn=True,
        )
        try:
            sup.run_supervisor()  # the mocked spawn writes STOP_SENTINEL after one call
        finally:
            self._restore()
        self.assertEqual(len(spawns), 1)


class TestAutoseedIncrementsLedger(TempStateCase):
    """T6-entry10: _try_autoseed bumps the plan_selector seed-attempt ledger on every successful seed so
    the reseed filter can bound re-selection. select_next + record_seed_attempt are patched so no test
    touches the network/git or the real ledger file."""

    def test_records_seed_attempt_on_successful_seed(self):
        self._write_state(queue=[], cursor={"plan_id": None, "stage": None, "pickup": None})
        orig = {"WALK_STATE": sup.WALK_STATE, "select": sup.plan_selector.select_next,
                "record": sup.plan_selector.record_seed_attempt, "emit": sup._emit,
                "drain": sup._drain_superseded}
        recorded = []
        sup.WALK_STATE = Path(self.path)
        # the free-skip pass is neutralized so this test never reads the real cards or writes the real
        # disposition ledger; select_next now takes the injected corpus_reader kwarg (tolerate it).
        sup._drain_superseded = lambda *a, **k: 0
        sup.plan_selector.select_next = lambda state, psd, **kw: {
            "plan_id": "C-47", "pickup": "run S1/S2/S3 recon on C-47",
            "detail": {"eligible": ["C-47"]}}
        sup.plan_selector.record_seed_attempt = lambda pid: (recorded.append(pid), 1)[1]
        sup._emit = lambda *a, **k: None
        try:
            ok = sup._try_autoseed(turn=0)
        finally:
            sup.WALK_STATE = orig["WALK_STATE"]
            sup.plan_selector.select_next = orig["select"]
            sup.plan_selector.record_seed_attempt = orig["record"]
            sup._emit = orig["emit"]
            sup._drain_superseded = orig["drain"]
        self.assertTrue(ok)
        self.assertEqual(recorded, ["C-47"])            # ledger bumped exactly once for the seeded plan
        # the seed really landed in the temp walk-state (real seed_plan ran)
        self.assertIn("C-47", ws.load(self.path)["queue"])



class TestTokenSyncFromMax(unittest.TestCase):
    """P0 token-auth: _sync_elai_token_from_max writes both dests, never copies refreshToken, and
    syncs on fresher/near-expiry/force."""

    def setUp(self):
        self.tmpdir = tempfile.mkdtemp(prefix="opussup_tok_")
        self.max_path = Path(self.tmpdir) / "max.credentials.json"
        self.elai_path = Path(self.tmpdir) / "elai.credentials.json"
        self.home_claude = Path(self.tmpdir) / "home" / ".claude"
        self.home_claude.mkdir(parents=True)
        self.home_creds = self.home_claude / ".credentials.json"
        self._orig_accounts = dict(sup.token_refresh.ACCOUNTS)
        self._orig_home = Path.home
        self._orig_emit = sup._emit
        self.emitted = []
        sup.token_refresh.ACCOUNTS = {"max": str(self.max_path), "elai": str(self.elai_path)}
        Path.home = lambda: Path(self.tmpdir) / "home"  # type: ignore[assignment]
        sup._emit = lambda et, payload: self.emitted.append((et, payload))

    def tearDown(self):
        sup.token_refresh.ACCOUNTS = self._orig_accounts
        Path.home = self._orig_home  # type: ignore[assignment]
        sup._emit = self._orig_emit
        shutil.rmtree(self.tmpdir, ignore_errors=True)

    def _write(self, path: Path, access: str, exp: int, refresh: str | None = None):
        oauth = {"accessToken": access, "expiresAt": exp}
        if refresh is not None:
            oauth["refreshToken"] = refresh
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps({"claudeAiOauth": oauth}, indent=2), encoding="utf-8")

    def _read(self, path: Path) -> dict:
        return json.loads(path.read_text(encoding="utf-8"))["claudeAiOauth"]

    def test_syncs_both_dests_when_max_fresher(self):
        import time
        now = int(time.time() * 1000)
        self._write(self.max_path, "MAXTOK", now + 8 * 3600 * 1000, refresh="MAXREF")
        self._write(self.elai_path, "OLD", now - 1000, refresh="ELAIAREF")
        self._write(self.home_creds, "OLD2", now - 1000, refresh="HOMEREF")
        sup._sync_elai_token_from_max(force=False)
        for p, ref in ((self.elai_path, "ELAIAREF"), (self.home_creds, "HOMEREF")):
            o = self._read(p)
            self.assertEqual(o["accessToken"], "MAXTOK")
            self.assertEqual(o["expiresAt"], now + 8 * 3600 * 1000)
            self.assertEqual(o.get("refreshToken"), ref)  # never copied from max
        self.assertTrue(any(et == "OPUS_TOKEN_SYNC" for et, _ in self.emitted))

    def test_force_sync_when_local_future_but_stale_token(self):
        import time
        now = int(time.time() * 1000)
        # Local still has future expiresAt (> now+30min) equal-ish to max -> without force no write.
        future = now + 4 * 3600 * 1000
        self._write(self.max_path, "NEWTOK", future, refresh="MAXREF")
        self._write(self.elai_path, "STALE", future, refresh="ELAIAREF")
        self._write(self.home_creds, "STALE", future, refresh="HOMEREF")
        sup._sync_elai_token_from_max(force=False)
        self.assertEqual(self._read(self.elai_path)["accessToken"], "STALE")
        sup._sync_elai_token_from_max(force=True)
        self.assertEqual(self._read(self.elai_path)["accessToken"], "NEWTOK")
        self.assertEqual(self._read(self.home_creds)["accessToken"], "NEWTOK")
        self.assertEqual(self._read(self.elai_path).get("refreshToken"), "ELAIAREF")

    def test_near_expiry_triggers_sync(self):
        import time
        now = int(time.time() * 1000)
        # Local expires in 10 min (< 30 min near-expiry window); max is fresher later.
        self._write(self.max_path, "FRESH", now + 8 * 3600 * 1000, refresh="MAXREF")
        self._write(self.elai_path, "SOON", now + 10 * 60 * 1000, refresh="ELAIAREF")
        self._write(self.home_creds, "SOON", now + 10 * 60 * 1000, refresh="HOMEREF")
        sup._sync_elai_token_from_max(force=False)
        self.assertEqual(self._read(self.elai_path)["accessToken"], "FRESH")
        self.assertEqual(self._read(self.home_creds)["accessToken"], "FRESH")


class TestFastFailTokenRescue(TempStateCase):
    """P0: first FAST_FAIL force-syncs once (token_rescue_used) and allows one extra spawn before
    permanent FAST_FAIL_STORM."""

    def _wire(self, spawn_results):
        self._orig = {
            "WALK_STATE": sup.WALK_STATE,
            "reconcile": sup._reconcile_done_plans,
            "nowork": sup._no_actionable_work,
            "should": sup._should_spawn,
            "quorum_enabled": sup._quorum_enabled,
            "spawn": sup._spawn_opus_turn,
            "emit": sup._emit,
            "sleep": sup.INTER_TURN_SLEEP_S,
            "sync": sup._sync_elai_token_from_max,
            "guard": sup.disk_hygiene.guard_enabled,
        }
        self.syncs = []
        self.spawns = []
        it = iter(spawn_results)

        def fake_spawn(sid, budget):
            self.spawns.append(sid)
            return next(it)

        def fake_sync(force=False):
            self.syncs.append(bool(force))

        sup.WALK_STATE = Path(self.path)
        sup._reconcile_done_plans = lambda *a, **k: []
        sup._no_actionable_work = lambda *a, **k: False
        sup._should_spawn = lambda *a, **k: (True, "ok")
        sup._quorum_enabled = lambda: False
        sup.disk_hygiene.guard_enabled = lambda: False
        sup._spawn_opus_turn = fake_spawn
        sup._sync_elai_token_from_max = fake_sync
        sup._emit = lambda *a, **k: None
        sup.INTER_TURN_SLEEP_S = 0

    def _restore(self):
        for k, v in self._orig.items():
            if k == "guard":
                sup.disk_hygiene.guard_enabled = v
            elif k == "sync":
                sup._sync_elai_token_from_max = v
            elif k == "spawn":
                sup._spawn_opus_turn = v
            elif k == "emit":
                sup._emit = v
            elif k == "sleep":
                sup.INTER_TURN_SLEEP_S = v
            elif k == "WALK_STATE":
                sup.WALK_STATE = v
            elif k == "reconcile":
                sup._reconcile_done_plans = v
            elif k == "nowork":
                sup._no_actionable_work = v
            elif k == "should":
                sup._should_spawn = v
            elif k == "quorum_enabled":
                sup._quorum_enabled = v

    def test_rescues_once_then_storms(self):
        # FAST_FAIL_SECONDS=45; rc!=0 with short duration. 3 fails would storm, but rescue
        # rolls the limit counter once, so 4 fails are needed before permanent halt.
        fail = {"rc": 1, "timed_out": False, "stdout_tail": "", "stderr_tail": "auth"}
        self._write_state(queue=["Y"])
        self._wire([fail, fail, fail, fail])
        try:
            # monkeypatch time.time so each spawn appears as a ~5s fast-fail
            import time as _time
            real_time = _time.time
            t = {"v": 1_000_000.0}

            def fake_time():
                # alternate so end-start = 5s within _spawn path in run_supervisor
                t["v"] += 5.0
                return t["v"]

            _time.time = fake_time
            try:
                rc = sup.run_supervisor()
            finally:
                _time.time = real_time
        finally:
            self._restore()
        self.assertEqual(rc, 1)
        # First fast-fail triggers force sync; permanent halt path does not force again after used.
        self.assertIn(True, self.syncs)
        self.assertEqual(self.syncs.count(True), 1)
        # Extra attempt after rescue: more than FAST_FAIL_LIMIT spawns before halt.
        self.assertGreaterEqual(len(self.spawns), sup.FAST_FAIL_LIMIT + 1)

if __name__ == "__main__":
    unittest.main()
