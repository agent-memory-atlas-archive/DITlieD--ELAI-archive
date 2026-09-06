"""Unit tests for opus_dispatch.py codex-objective composition (opus-backlog-walk ENTRY 10 F-2 + F-4).

Pure composition tests -- no worker is dispatched, no db/canvas/network is touched, no supervisor or
process is started. The tests read the REAL teikoku-side binding files (agent-dispatch-preamble.md,
the agent roster, ELAI-SPEC-OVERVIEW.md) which the CONDUCTOR can see under /mnt/f; only codex's OWN
scoped mount cannot, which is exactly the bug these fixes address.
"""
import unittest

import opus_dispatch as od

# A pointer with a UNIQUE unreachable path: asserting THIS path is absent proves the INCOMING
# objective's pointer was stripped, without colliding with the binding preamble's own documented
# example of the pointer convention (which legitimately survives as inlined reference content).
_MARKER_PATH = "/UNREACHABLE/codex-cannot-open/agent-dispatch-preamble.md"
_INCOMING_POINTER = f"Read {_MARKER_PATH} FIRST and obey it as binding. handoff_file=/x; task_id=t1"
_TASK_BODY = "Wire fn foo into seam bar and add the unit test. This is the real task."


class StripUnreachablePointers(unittest.TestCase):
    def test_strips_obey_form(self):
        obj = _INCOMING_POINTER + "\n\n" + _TASK_BODY
        out = od._strip_unreachable_pointers(obj)
        self.assertNotIn(_MARKER_PATH, out)
        self.assertNotIn("FIRST and obey it as binding", out)
        self.assertIn(_TASK_BODY, out)

    def test_strips_follow_form(self):
        obj = "Read /mnt/f/.claude/agents/x.md FIRST and follow it strictly.\n\n" + _TASK_BODY
        out = od._strip_unreachable_pointers(obj)
        self.assertNotIn("follow it strictly", out)
        self.assertIn(_TASK_BODY, out)

    def test_no_pointer_is_noop_body_preserved(self):
        out = od._strip_unreachable_pointers(_TASK_BODY)
        self.assertEqual(out, _TASK_BODY)


class CapOverview(unittest.TestCase):
    def test_short_text_unchanged(self):
        self.assertEqual(od._cap_overview("short", max_chars=100), "short")

    def test_long_text_truncated_with_marker(self):
        capped = od._cap_overview("x" * 500, max_chars=50)
        self.assertEqual(len(capped[:50]), 50)
        self.assertTrue(capped.startswith("x" * 50))
        self.assertIn("truncated at 50 chars", capped)
        self.assertLess(len(capped.replace(od._CODEX_OVERVIEW_TRUNC_MARK.format(n=50), "")), 500)

    def test_load_overview_reads_and_caps(self):
        # The real overview is reachable to the conductor and non-empty; the load helper caps it.
        text = od._load_codex_overview()
        self.assertTrue(text, "L1 overview should be readable teikoku-side")
        self.assertLessEqual(len(text), od.CODEX_OVERVIEW_MAX_CHARS + len(
            od._CODEX_OVERVIEW_TRUNC_MARK.format(n=od.CODEX_OVERVIEW_MAX_CHARS)))


class ComposeCodexObjectiveDispatch1(unittest.TestCase):
    """dispatch#1 shape: agent=None (codex dispatches pass no --agent), overview inlined."""

    def _compose(self, agent=None, evidence_file=None):
        overview = od._load_codex_overview()
        incoming = _INCOMING_POINTER + "\n\n" + _TASK_BODY
        return od._compose_codex_objective(agent, incoming, evidence_file=evidence_file,
                                           overview_text=overview)

    def test_preamble_always_inlined_even_without_agent(self):
        out = self._compose(agent=None)
        # F-4: the binding block is present with agent=None (was previously gated behind `if agent:`).
        self.assertIn("BINDING RULES (INLINED)", out)
        self.assertIn("agent-dispatch-preamble (binding)", out)
        # verbatim content from the REAL preamble file proves it was inlined, not just referenced.
        self.assertIn("GIT IS READ-ONLY FOR YOU", out)

    def test_l1_overview_header_and_body_inlined(self):
        out = self._compose(agent=None)
        self.assertIn("ELAI design overview (inlined", out)
        # a verbatim thesis line from the real overview file.
        self.assertIn("local-first, deterministic coding-and-reasoning agent", out)

    def test_incoming_unreachable_pointer_stripped(self):
        out = self._compose(agent=None)
        # the INCOMING objective's unreachable pointer path is gone from what codex sees.
        self.assertNotIn(_MARKER_PATH, out)
        # the real task body survives.
        self.assertIn(_TASK_BODY, out)

    def test_agent_block_conditional_absent_when_agent_none(self):
        out = self._compose(agent=None)
        self.assertNotIn("----- your agent role:", out)

    def test_agent_block_present_when_agent_named(self):
        out = self._compose(agent="integration-explorer-elai")
        self.assertIn("----- your agent role: integration-explorer-elai -----", out)
        self.assertIn("BINDING RULES (INLINED)", out)  # preamble still inlined too

    def test_unknown_agent_fail_closed(self):
        with self.assertRaises(ValueError):
            od._compose_codex_objective("no-such-agent-xyz", _TASK_BODY, overview_text="")

    def test_no_overview_still_binds(self):
        out = od._compose_codex_objective(None, _TASK_BODY, overview_text="")
        self.assertIn("BINDING RULES (INLINED)", out)
        self.assertNotIn("ELAI design overview", out)


class ComposeAgentObjectiveCodexPrep(unittest.TestCase):
    """F-2 L2/L3: the FREE recon/explorer evidence-file handoff instructs a codex-prep pre-digest."""

    def test_evidence_handoff_carries_predigest_instruction(self):
        out = od._compose_agent_objective(
            "integration-explorer-elai", _TASK_BODY, evidence_file="/mnt/f/elai-codex-worktrees/t/ev.md")
        self.assertIn("PRE-DIGEST", out)
        self.assertIn("SUMMARY_INDEX", out)
        self.assertIn("*SUMMARY*.md", out)
        self.assertIn("/mnt/f/elai-codex-worktrees/t/ev.md", out)

    def test_no_evidence_no_predigest(self):
        out = od._compose_agent_objective("integration-explorer-elai", _TASK_BODY)
        self.assertNotIn("PRE-DIGEST", out)


if __name__ == "__main__":
    unittest.main()
