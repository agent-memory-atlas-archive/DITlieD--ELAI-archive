"""Unit tests for ALLOW_NO_VERIFIER body parse (A-EV-ESCAPE-BODY / ARCH-EV-ESCAPE-BODY).

Pure parse coverage + land-path event wiring smoke. No live land, no live control.db write
(event emit is captured). No env-based escape path is accepted.
"""
from __future__ import annotations

import json
import os
import tempfile
import unittest
from pathlib import Path

import opus_land as ol


def _write(path: Path, body) -> None:
    if isinstance(body, str):
        path.write_text(body, encoding="utf-8")
    else:
        path.write_text(json.dumps(body), encoding="utf-8")


class TestParseAllowNoVerifier(unittest.TestCase):
    def test_empty_file_refused(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "ALLOW_NO_VERIFIER"
            p.write_text("", encoding="utf-8")
            ok, reason, detail = ol.parse_allow_no_verifier(p, "PLAN-X")
            self.assertFalse(ok)
            self.assertEqual(reason, "empty_body")
            self.assertEqual(detail["plan_id"], "PLAN-X")

    def test_whitespace_only_refused(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "ALLOW_NO_VERIFIER"
            p.write_text("  \n\t  \n", encoding="utf-8")
            ok, reason, _ = ol.parse_allow_no_verifier(p, "PLAN-X")
            self.assertFalse(ok)
            self.assertEqual(reason, "empty_body")

    def test_missing_path_refused(self):
        p = Path("/tmp/no-such-allow-no-verifier-sentinel-a-ev-escape")
        if p.exists():
            p.unlink()
        ok, reason, _ = ol.parse_allow_no_verifier(p, "PLAN-X")
        self.assertFalse(ok)
        self.assertEqual(reason, "missing")

    def test_valid_body_accepts_matching_plan(self):
        body = {
            "timestamp": "2026-07-29T12:00:00Z",
            "scope": {"plan_id": "PLAN-X"},
            "signer": "contributor",
            "reason": "flake after 3 retries",
        }
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "ALLOW_NO_VERIFIER"
            _write(p, body)
            ok, reason, detail = ol.parse_allow_no_verifier(p, "PLAN-X")
            self.assertTrue(ok)
            self.assertEqual(reason, "granted")
            self.assertEqual(detail["timestamp"], "2026-07-29T12:00:00Z")
            self.assertEqual(detail["scope"]["plan_id"], "PLAN-X")
            self.assertEqual(detail["signer"], "contributor")

    def test_unix_timestamp_int_accepted(self):
        body = {"timestamp": 1722268800, "scope": {"plan_id": "PLAN-X"}}
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "ALLOW_NO_VERIFIER"
            _write(p, body)
            ok, reason, detail = ol.parse_allow_no_verifier(p, "PLAN-X")
            self.assertTrue(ok)
            self.assertEqual(reason, "granted")
            self.assertEqual(detail["timestamp"], 1722268800)

    def test_scope_mismatch_refused(self):
        body = {
            "timestamp": "2026-07-29T12:00:00Z",
            "scope": {"plan_id": "OTHER-PLAN"},
        }
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "ALLOW_NO_VERIFIER"
            _write(p, body)
            ok, reason, detail = ol.parse_allow_no_verifier(p, "PLAN-X")
            self.assertFalse(ok)
            self.assertEqual(reason, "scope_mismatch")
            self.assertEqual(detail["scope_plan_id"], "OTHER-PLAN")

    def test_wildcard_scope_refused(self):
        body = {"timestamp": "2026-07-29T12:00:00Z", "scope": {"plan_id": "*"}}
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "ALLOW_NO_VERIFIER"
            _write(p, body)
            ok, reason, _ = ol.parse_allow_no_verifier(p, "PLAN-X")
            self.assertFalse(ok)
            self.assertEqual(reason, "scope_mismatch")

    def test_list_scope_plan_id_refused(self):
        body = {
            "timestamp": "2026-07-29T12:00:00Z",
            "scope": {"plan_id": ["PLAN-X", "PLAN-Y"]},
        }
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "ALLOW_NO_VERIFIER"
            _write(p, body)
            ok, reason, _ = ol.parse_allow_no_verifier(p, "PLAN-X")
            self.assertFalse(ok)
            self.assertEqual(reason, "scope_mismatch")

    def test_bad_json_refused(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "ALLOW_NO_VERIFIER"
            p.write_text("{not-json", encoding="utf-8")
            ok, reason, detail = ol.parse_allow_no_verifier(p, "PLAN-X")
            self.assertFalse(ok)
            self.assertEqual(reason, "bad_json")
            self.assertIn("error", detail)

    def test_missing_timestamp_refused(self):
        body = {"scope": {"plan_id": "PLAN-X"}}
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "ALLOW_NO_VERIFIER"
            _write(p, body)
            ok, reason, detail = ol.parse_allow_no_verifier(p, "PLAN-X")
            self.assertFalse(ok)
            self.assertEqual(reason, "missing_fields")
            self.assertEqual(detail["missing"], "timestamp")

    def test_missing_scope_refused(self):
        body = {"timestamp": "2026-07-29T12:00:00Z"}
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "ALLOW_NO_VERIFIER"
            _write(p, body)
            ok, reason, detail = ol.parse_allow_no_verifier(p, "PLAN-X")
            self.assertFalse(ok)
            self.assertEqual(reason, "missing_fields")
            self.assertEqual(detail["missing"], "scope")

    def test_env_var_does_not_grant(self):
        """ABORT: env-based escape must not exist; parse is Path-only."""
        env_keys = (
            "ALLOW_NO_VERIFIER",
            "ELAI_ALLOW_NO_VERIFIER",
            "OPUS_ALLOW_NO_VERIFIER",
            "TEIKOKU_ALLOW_NO_VERIFIER",
        )
        saved = {k: os.environ.get(k) for k in env_keys}
        try:
            for k in env_keys:
                os.environ[k] = "1"
            p = Path("/tmp/no-such-allow-no-verifier-env-check")
            if p.exists():
                p.unlink()
            ok, reason, _ = ol.parse_allow_no_verifier(p, "PLAN-X")
            self.assertFalse(ok)
            self.assertEqual(reason, "missing")
            with tempfile.TemporaryDirectory() as td:
                empty = Path(td) / "ALLOW_NO_VERIFIER"
                empty.write_text("", encoding="utf-8")
                ok2, reason2, _ = ol.parse_allow_no_verifier(empty, "PLAN-X")
                self.assertFalse(ok2)
                self.assertEqual(reason2, "empty_body")
        finally:
            for k, v in saved.items():
                if v is None:
                    os.environ.pop(k, None)
                else:
                    os.environ[k] = v


class TestOptOutLandPathEvents(unittest.TestCase):
    """Dual-method B: GRANTED/REFUSED payload contract matching main's opt-out branch."""

    def _run_opt_out_branch(self, sentinel: Path, plan_id: str = "PLAN-X"):
        events = []

        def capture_emit(event_type, payload):
            events.append((event_type, payload))

        granted, reason, parse_detail = ol.parse_allow_no_verifier(sentinel, plan_id)
        opt_payload = {
            "plan_id": plan_id,
            "sentinel": str(sentinel),
            "reason": reason,
        }
        for key in ("timestamp", "scope", "error", "missing", "scope_plan_id",
                    "signer", "owner_reason"):
            if key in parse_detail:
                opt_payload[key] = parse_detail[key]
        if granted:
            capture_emit("OPUS_LAND_VERIFIER_OPTOUT_GRANTED", opt_payload)
        else:
            capture_emit("OPUS_LAND_VERIFIER_OPTOUT_REFUSED", opt_payload)
        return granted, events

    def test_empty_emits_refused_empty_body(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "ALLOW_NO_VERIFIER"
            p.write_text("", encoding="utf-8")
            granted, events = self._run_opt_out_branch(p)
            self.assertFalse(granted)
            self.assertEqual(len(events), 1)
            etype, payload = events[0]
            self.assertEqual(etype, "OPUS_LAND_VERIFIER_OPTOUT_REFUSED")
            self.assertEqual(payload["reason"], "empty_body")

    def test_valid_emits_granted(self):
        body = {
            "timestamp": "2026-07-29T12:00:00Z",
            "scope": {"plan_id": "PLAN-X"},
        }
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "ALLOW_NO_VERIFIER"
            _write(p, body)
            granted, events = self._run_opt_out_branch(p)
            self.assertTrue(granted)
            self.assertEqual(len(events), 1)
            etype, payload = events[0]
            self.assertEqual(etype, "OPUS_LAND_VERIFIER_OPTOUT_GRANTED")
            self.assertEqual(payload["reason"], "granted")
            self.assertEqual(payload["scope"]["plan_id"], "PLAN-X")
            self.assertEqual(payload["timestamp"], "2026-07-29T12:00:00Z")

    def test_mismatch_emits_refused_scope(self):
        body = {
            "timestamp": "2026-07-29T12:00:00Z",
            "scope": {"plan_id": "OTHER"},
        }
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "ALLOW_NO_VERIFIER"
            _write(p, body)
            granted, events = self._run_opt_out_branch(p)
            self.assertFalse(granted)
            etype, payload = events[0]
            self.assertEqual(etype, "OPUS_LAND_VERIFIER_OPTOUT_REFUSED")
            self.assertEqual(payload["reason"], "scope_mismatch")

    def test_main_wires_parse_and_granted_event(self):
        """Production wire: main source calls parse + emits GRANTED/REFUSED."""
        import inspect
        src = inspect.getsource(ol.main)
        self.assertIn("parse_allow_no_verifier", src)
        self.assertIn("OPUS_LAND_VERIFIER_OPTOUT_GRANTED", src)
        self.assertIn("OPUS_LAND_VERIFIER_OPTOUT_REFUSED", src)
        self.assertIn("args.no_verifier", src)


if __name__ == "__main__":
    unittest.main()
