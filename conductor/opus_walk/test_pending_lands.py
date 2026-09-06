#!/usr/bin/env python3
import json
import multiprocessing
import tempfile
import unittest
from pathlib import Path
from unittest import mock
import pending_lands as pl


def _claim_worker(path, worker_id, queue):
    queue.put(pl.claim_row("P", worker_id, path=Path(path), worktree="/wt-P"))


def _live_parser_destinations():
    import argparse
    import types
    import opus_land

    original_module = opus_land.argparse
    original_parser = argparse.ArgumentParser
    captured = []

    class ParserProbe:
        def __init__(self, parser):
            self._parser = parser

        def add_argument(self, *argv, **kwargs):
            return self._parser.add_argument(*argv, **kwargs)

        @property
        def _actions(self):
            return self._parser._actions

        def parse_args(self, *argv, **kwargs):
            raise RuntimeError("parser_surface_capture")

    def capture(*argv, **kwargs):
        parser = ParserProbe(original_parser(*argv, **kwargs))
        captured.append(parser)
        return parser

    opus_land.argparse = types.SimpleNamespace(ArgumentParser=capture)
    try:
        try:
            opus_land.main([])
        except RuntimeError as exc:
            if str(exc) != "parser_surface_capture":
                raise
    finally:
        opus_land.argparse = original_module
    return tuple(sorted(action.dest for action in captured[-1]._actions if action.dest != "help"))


class TestPending(unittest.TestCase):
    def test_roundtrip(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "pl.json"
            pl.add("E-11", "/mnt/f/x", "/a", path=p)
            pl.add("E-11", "/mnt/f/x", "/a", path=p)
            self.assertEqual(len(pl.list_lands(p)), 1)
            self.assertIn("E-11", pl.protected_plan_ids(p))
            self.assertEqual(pl.remove(plan_id="E-11", path=p), 1)

    def test_reason_waiting_for_verifier(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "pl.json"
            pl.add("E-12", "/wt/e12", path=p, reason="WAITING_FOR_VERIFIER")
            lands = pl.list_lands(p)
            self.assertEqual(lands[0]["reason"], "WAITING_FOR_VERIFIER")
            waiting = pl.list_waiting_for_verifier(p)
            self.assertEqual(len(waiting), 1)
            self.assertEqual(waiting[0]["plan_id"], "E-12")
            pl.add("E-13", "/wt/e13", path=p)
            self.assertEqual(pl.list_lands(p)[1].get("reason"), "")
            self.assertEqual(len(pl.list_waiting_for_verifier(p)), 1)


class TestLandSuccessClearsPending(unittest.TestCase):
    def test_land_success_removes_pending_row(self):
        """BURN-U2: successful land must call pending_lands.remove so the ledger shrinks."""
        import opus_land as ol

        with tempfile.TemporaryDirectory() as td:
            ledger = Path(td) / "pending-lands.json"
            pl.add("BURN-U2-PLAN", "/wt/burn-u2", path=ledger, reason="WAITING_FOR_VERIFIER")
            pl.add("OTHER-PLAN", "/wt/other", path=ledger)
            self.assertEqual(len(pl.list_lands(ledger)), 2)
            before = len(pl.list_lands(ledger))
            removed = ol.remove_pending_land_on_success(
                "BURN-U2-PLAN", worktree="/wt/burn-u2", pending_path=ledger
            )
            after = len(pl.list_lands(ledger))
            self.assertEqual(removed, 1)
            self.assertEqual(after, before - 1)
            remaining = pl.list_lands(ledger)
            self.assertEqual(len(remaining), 1)
            self.assertEqual(remaining[0]["plan_id"], "OTHER-PLAN")
            # second remove is a no-op (idempotent)
            self.assertEqual(
                ol.remove_pending_land_on_success("BURN-U2-PLAN", pending_path=ledger),
                0,
            )

    def test_finish_landed_path_wires_remove(self):
        """Structural: the shipped land-success finish path must call remove helper."""
        import inspect
        import opus_land as ol

        src = inspect.getsource(ol.main)
        self.assertIn("remove_pending_land_on_success", src)
        self.assertIn('startswith("LANDED")', src)
        helper_src = inspect.getsource(ol.remove_pending_land_on_success)
        self.assertIn("pending_lands.remove", helper_src)


class TestU3ASchemaAndClaims(unittest.TestCase):
    def _args(self, *, gates_only=False, no_verifier=False):
        values = {}
        for field in pl.REPLAY_ARG_FIELDS:
            if field == "files":
                values[field] = [".elai_cc/src.py"]
            elif field == "gate_cmd":
                values[field] = ["true"]
            elif field in {"gates_only", "no_verifier", "push", "json"}:
                values[field] = field == "gates_only" and gates_only or field == "no_verifier" and no_verifier
            elif field in {"worktree", "plan_id", "task_class"}:
                values[field] = {"worktree": "/wt", "plan_id": "P", "task_class": "class"}[field]
            else:
                values[field] = None
        return values

    def _write_valid(self, path, plan_id="P", argv=None, **kwargs):
        values = self._args(**kwargs)
        values["plan_id"] = plan_id
        values["worktree"] = "/wt-" + plan_id
        return pl.add(plan_id, "/wt-" + plan_id, path=path, replay_args=values,
                      replay_argv=argv or ["--plan-id", plan_id])

    def test_live_parser_surface_and_both_drift_directions(self):
        live = _live_parser_destinations()
        self.assertEqual(live, tuple(sorted(pl.REPLAY_ARG_FIELDS)))
        self.assertEqual(
            set(pl.REPLAY_ARG_FIELDS),
            {
                "worktree", "plan_id", "files", "gate_cmd", "gates_only", "no_verifier",
                "plan_spec", "message", "push", "task_class", "task_id", "codex_reap_task", "json",
            },
        )
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "rows.json"
            self._write_valid(p)
            self.assertTrue(pl.load_rows(p)[0]["replayable"])
            with mock.patch.object(pl, "REPLAY_ARG_FIELDS", tuple(sorted((*pl.REPLAY_ARG_FIELDS, "fake_extra")))):
                self.assertEqual(pl.load_rows(p)[0]["refusal_reason"], "parser_surface_drift")
            with mock.patch.object(pl, "REPLAY_ARG_FIELDS", tuple(x for x in pl.REPLAY_ARG_FIELDS if x != "json")):
                self.assertEqual(pl.load_rows(p)[0]["refusal_reason"], "parser_surface_drift")

    def test_round_trip_hash_and_argv(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "rows.json"
            argv = ["--worktree", "/wt-P", "--files", ".elai_cc/src.py", "--gate-cmd", "true"]
            row = self._write_valid(p, argv=argv)
            loaded = pl.load_rows(p)[0]
            self.assertTrue(loaded["replayable"])
            self.assertEqual(loaded["replay_argv"], argv)
            self.assertEqual(loaded["args_sha256"], pl._canonical_args_sha256(row["replay_args"]))
            self.assertEqual(loaded["gates_only"], False)
            self.assertEqual(loaded["no_verifier"], False)

    def test_refusals_are_distinct_and_preserved(self):
        cases = {
            "unknown": lambda values: values.update(extra=True),
            "missing": lambda values: values.pop("files"),
            "nonserial": lambda values: values.update(files=object()),
            "gates": lambda values: values.update(gates_only=True),
            "noverify": lambda values: values.update(no_verifier=True),
        }
        with tempfile.TemporaryDirectory() as td:
            for name, mutate in cases.items():
                p = Path(td) / f"{name}.json"
                values = self._args()
                mutate(values)
                pl.add(name, "/wt-" + name, path=p, replay_args=values, replay_argv=[name])
                loaded = pl.load_rows(p)
                self.assertEqual(len(loaded), 1)
                self.assertFalse(loaded[0]["replayable"])
                self.assertEqual(loaded[0]["refusal_reason"], {
                    "unknown": "unknown_replay_argument",
                    "missing": "missing_replay_argument",
                    "nonserial": "non_serializable_argument",
                    "gates": "disallowed_replay_flag",
                    "noverify": "disallowed_replay_flag",
                }[name])
                self.assertEqual(len(json.loads(p.read_text(encoding="utf-8"))["lands"]), 1)
            p = Path(td) / "legacy.json"
            row = self._write_valid(p, plan_id="legacy")
            raw = json.loads(p.read_text(encoding="utf-8"))
            raw["lands"][0]["record_version"] = 1
            p.write_text(json.dumps(raw), encoding="utf-8")
            legacy = pl.load_rows(p)[0]
            self.assertFalse(legacy["replayable"])
            self.assertEqual(legacy["refusal_reason"], "legacy_v1_schema")
            self.assertEqual(len(json.loads(p.read_text(encoding="utf-8"))["lands"]), 1)
            p = Path(td) / "tamper.json"
            self._write_valid(p, plan_id="tamper")
            raw = json.loads(p.read_text(encoding="utf-8"))
            raw["lands"][0]["replay_args"]["gate_cmd"] = ["false"]
            p.write_text(json.dumps(raw), encoding="utf-8")
            tampered = pl.load_rows(p)[0]
            self.assertFalse(tampered["replayable"])
            self.assertEqual(tampered["refusal_reason"], "args_hash_mismatch")

    def test_claim_lease_boundaries_and_release(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "rows.json"
            self._write_valid(p)
            first = pl.claim_row("P", "worker-a", path=p, worktree="/wt-P", now=1000)
            self.assertIsNotNone(first)
            self.assertEqual(first["lease_seconds"], 900)
            self.assertIsNone(pl.claim_row("P", "worker-b", path=p, worktree="/wt-P", now=1899))
            reclaimed = pl.claim_row("P", "worker-b", path=p, worktree="/wt-P", now=1901)
            self.assertIsNotNone(reclaimed)
            released = pl.release_claim("P", claim_token=reclaimed["claim_token"], path=p, worktree="/wt-P")
            self.assertIsNotNone(released)
            self.assertNotIn("claim_token", released)
            self.assertNotIn("claimed_by", released)
            self.assertTrue(json.loads(p.read_text(encoding="utf-8"))["lands"][0]["replayable"])

    def test_concurrent_claim_has_one_winner(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "rows.json"
            self._write_valid(p)
            ctx = multiprocessing.get_context("fork")
            queue = ctx.Queue()
            workers = [ctx.Process(target=_claim_worker, args=(str(p), f"worker-{i}", queue)) for i in range(2)]
            for worker in workers:
                worker.start()
            results = [queue.get(timeout=10) for _ in workers]
            for worker in workers:
                worker.join(timeout=10)
            self.assertEqual(sum(result is not None for result in results), 1)


if __name__ == "__main__":
    unittest.main()
