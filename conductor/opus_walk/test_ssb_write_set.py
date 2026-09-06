#!/usr/bin/env python3
import unittest
from ssb_write_set import expand_allowed_writes


class TestExpand(unittest.TestCase):
    def test_ssb_trigger_expands(self):
        out = expand_allowed_writes(["crates/orchestrator/src/ssb/action_type.rs"])
        joined = " ".join(out)
        self.assertIn("ssb.rs", joined)
        self.assertIn("action_type_lock", joined)
        self.assertIn(".elai_cc/crates/orchestrator/src/ssb.rs", out)
        self.assertIn(
            ".elai_cc/crates/orchestrator/tests/action_type_lock.golden", out
        )

    def test_elai_cc_prefix_preserves_dot(self):
        out = expand_allowed_writes(
            [".elai_cc/crates/orchestrator/src/ssb/action_type.rs"]
        )
        self.assertIn(".elai_cc/crates/orchestrator/src/ssb/action_type.rs", out)
        self.assertIn("crates/orchestrator/src/ssb/action_type.rs", out)
        self.assertNotIn("elai_cc/crates/orchestrator/src/ssb/action_type.rs", out)

    def test_unrelated(self):
        out = expand_allowed_writes(["crates/cli/src/main.rs"])
        self.assertTrue(any("main.rs" in p for p in out))
        self.assertFalse(any(p.endswith("src/ssb.rs") for p in out))

    def test_empty(self):
        self.assertEqual(expand_allowed_writes([]), [])


if __name__ == "__main__":
    unittest.main()
