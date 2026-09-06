"""Unit tests for quorum_triage.py (P1.4 blind free-model cost gate).

Every voter transport is a mock post_fn; no test touches the network. Signals are built over a
temp plan-status directory so the real walk is never read.
"""
import json
import os
import shutil
import tempfile
import unittest

import quorum_triage as qt


def _post(mapping):
    """A mock post_fn: model -> {text|error}. Records the prompt each voter was handed so a test
    can assert the voters are blind (identical prompt) and independent."""
    calls = []

    def post_fn(model, prompt):
        calls.append((model, prompt))
        spec = mapping.get(model, {})
        if "text" in spec:
            return {"ok": True, "text": spec["text"], "status": 200, "error": None, "latency_ms": 5}
        return {"ok": False, "text": "", "status": spec.get("status"), "error": spec.get("error", "dead"),
                "latency_ms": 5}

    post_fn.calls = calls
    return post_fn


SIGNALS = {"queue": [{"plan_id": "Y", "done": False}],
           "cursor": {"plan_id": None, "stage": None}, "escalation_ids": []}


class TestVote(unittest.TestCase):
    def test_majority_yes_wakes(self):
        pf = _post({"swe-1-6": {"text": "YES, plan Y"}, "kimi-k2-7": {"text": "yes Y"},
                    "glm-5-2": {"text": "no"}})
        wake, detail = qt.vote(SIGNALS, post_fn=pf)
        self.assertTrue(wake)
        self.assertEqual(detail["reason"], "QUORUM_WAKE")
        self.assertEqual((detail["yes"], detail["no"]), (2, 1))

    def test_majority_no_quiets(self):
        pf = _post({"swe-1-6": {"text": "NO"}, "kimi-k2-7": {"text": "no work"},
                    "glm-5-2": {"text": "YES"}})
        wake, detail = qt.vote(SIGNALS, post_fn=pf)
        self.assertFalse(wake)
        self.assertEqual(detail["reason"], "QUORUM_QUIET")
        self.assertEqual((detail["yes"], detail["no"]), (1, 2))

    def test_one_of_three_returned_fails_open_degraded(self):
        # two voters error out, one returns NO -> only 1 parseable vote -> fail-open wake.
        pf = _post({"swe-1-6": {"error": "conn refused"}, "kimi-k2-7": {"error": "timeout"},
                    "glm-5-2": {"text": "NO nothing to do"}})
        wake, detail = qt.vote(SIGNALS, post_fn=pf)
        self.assertTrue(wake)  # cost gate fails OPEN
        self.assertEqual(detail["reason"], "QUORUM_DEGRADED")
        self.assertEqual(detail["returned"], 1)

    def test_unparseable_votes_dropped(self):
        # two hedge (no yes/no token) -> unparseable -> dropped -> only 1 real vote -> degraded.
        pf = _post({"swe-1-6": {"text": "maybe, unsure"}, "kimi-k2-7": {"text": "it depends"},
                    "glm-5-2": {"text": "YES Y"}})
        wake, detail = qt.vote(SIGNALS, post_fn=pf)
        self.assertEqual(detail["returned"], 1)
        self.assertEqual(detail["reason"], "QUORUM_DEGRADED")
        verdicts = {v["model"]: v["verdict"] for v in detail["voters"]}
        self.assertIsNone(verdicts["swe-1-6"])
        self.assertIsNone(verdicts["kimi-k2-7"])
        self.assertEqual(verdicts["glm-5-2"], "yes")

    def test_all_no_quiets(self):
        pf = _post({m: {"text": "NO"} for m in qt.QUORUM_ROSTER})
        wake, detail = qt.vote(SIGNALS, post_fn=pf)
        self.assertFalse(wake)
        self.assertEqual((detail["yes"], detail["no"]), (0, 3))

    def test_voters_are_blind_identical_prompt(self):
        pf = _post({m: {"text": "YES"} for m in qt.QUORUM_ROSTER})
        qt.vote(SIGNALS, post_fn=pf)
        # every voter in the roster was polled exactly once, with the SAME prompt (blind).
        self.assertEqual([m for m, _ in pf.calls], list(qt.QUORUM_ROSTER))
        prompts = {p for _, p in pf.calls}
        self.assertEqual(len(prompts), 1)


class TestBuildSignals(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp(prefix="quorum_sig_")
        self.status = os.path.join(self.tmp, "plan-status")
        os.makedirs(self.status, exist_ok=True)

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def _done_row(self, pid):
        with open(os.path.join(self.status, f"{pid}.json"), "w", encoding="utf-8") as f:
            json.dump({"plan_id": pid, "status": "DONE"}, f)

    def test_marks_done_and_escalated(self):
        self._done_row("A")
        state = {"queue": ["A", "B"], "cursor": {"plan_id": "B", "stage": "EXECUTING"},
                 "escalation_queue": [{"plan_id": "I-33", "reason": "greenfield"}]}
        sig = qt.build_signals(state, self.status)
        by_id = {q["plan_id"]: q["done"] for q in sig["queue"]}
        self.assertTrue(by_id["A"])
        self.assertFalse(by_id["B"])
        self.assertEqual(sig["cursor"], {"plan_id": "B", "stage": "EXECUTING"})
        self.assertEqual(sig["escalation_ids"], ["I-33"])


if __name__ == "__main__":
    unittest.main()
