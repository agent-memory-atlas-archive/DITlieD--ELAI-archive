#!/usr/bin/env python3
"""Unit tests for corpus_drain -- focused on the load-bearing fail-closed SKIP_SUPERSEDED guard.

No network, no git, no filesystem side effects: fixtures are in-memory index dicts + an injected
is_shipped_fn. Run: `python3 test_corpus_drain.py` (exit 0 all-pass) or `pytest test_corpus_drain.py`.
"""
from __future__ import annotations

import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import corpus_drain as cd


def _unit(uid, state, superseded_by=None, supersedes=None, residual="", does="x"):
    return {
        "id": uid,
        "does": does,
        "state": state,
        "supersedes": supersedes or [],
        "superseded_by": superseded_by or [],
        "residual": residual,
        "key_files": [],
        "respect": [],
    }


def _index(units):
    return {"schema_version": 1, "units": units, "edges": [], "parse_warnings": []}


class SkipSupersededGuard(unittest.TestCase):
    """The verify-then-skip guard is fail-closed: skip a dark unit ONLY if its superseder is proven
    shipped; otherwise it stays a BUILD candidate (never lose real work behind a phantom superseder)."""

    def test_a_dark_with_shipped_superseder_is_skipped(self):
        idx = _index([_unit("C-54", "dark", superseded_by=["E-9"]),
                      _unit("E-9", "wired")])
        shipped = lambda uid: uid == "E-9"  # E-9 proven shipped
        self.assertEqual(cd.classify(cd._units_by_id(idx)["C-54"], idx, shipped),
                         cd.SKIP_SUPERSEDED)

    def test_b_same_unit_dark_with_unshipped_superseder_is_build(self):
        # SAME dark unit, but the superseder is NOT proven shipped AND its card state is not wired.
        idx = _index([_unit("C-54", "dark", superseded_by=["E-9"]),
                      _unit("E-9", "dark")])  # superseder itself dark -> unconfirmed
        shipped = lambda uid: False  # nothing shipped
        self.assertEqual(cd.classify(cd._units_by_id(idx)["C-54"], idx, shipped),
                         cd.BUILD)

    def test_b2_dark_superseder_confirmed_via_card_wired_still_skips(self):
        # Fail-closed guard accepts card-state==wired as CONFIRMED-SHIPPED even without a live signal.
        idx = _index([_unit("C-54", "dark", superseded_by=["E-9"]),
                      _unit("E-9", "wired")])
        shipped = lambda uid: False  # no live signal, but E-9's card is wired
        self.assertEqual(cd.classify(cd._units_by_id(idx)["C-54"], idx, shipped),
                         cd.SKIP_SUPERSEDED)

    def test_b3_partial_superseder_is_not_confirmed(self):
        # A partial (not wired, not shipped) superseder does NOT confirm -> fail-closed BUILD.
        idx = _index([_unit("C-54", "dark", superseded_by=["E-9"]),
                      _unit("E-9", "partial")])
        self.assertEqual(cd.classify(cd._units_by_id(idx)["C-54"], idx, lambda u: False),
                         cd.BUILD)

    def test_b4_multi_superseder_one_unconfirmed_blocks_skip(self):
        # ALL superseders must be confirmed; one unconfirmed keeps the unit as BUILD.
        idx = _index([_unit("X-1", "dark", superseded_by=["Y-1", "Y-2"]),
                      _unit("Y-1", "wired"),
                      _unit("Y-2", "dark")])
        self.assertEqual(cd.classify(cd._units_by_id(idx)["X-1"], idx, lambda u: False),
                         cd.BUILD)

    def test_c_partial_unit_is_build_residual(self):
        idx = _index([_unit("E-17", "partial", superseded_by=["J-38"], residual="stage ONNX"),
                      _unit("J-38", "wired")])
        self.assertEqual(cd.classify(cd._units_by_id(idx)["E-17"], idx, lambda u: False),
                         cd.BUILD_RESIDUAL)

    def test_wired_no_residual_is_done(self):
        idx = _index([_unit("B-1", "wired", residual="none")])
        self.assertEqual(cd.classify(cd._units_by_id(idx)["B-1"], idx, lambda u: False), cd.DONE)

    def test_wired_with_material_residual_is_build_residual(self):
        idx = _index([_unit("B-9", "wired", residual="dashboard dag-write; verify shipped")])
        self.assertEqual(cd.classify(cd._units_by_id(idx)["B-9"], idx, lambda u: False),
                         cd.BUILD_RESIDUAL)

    def test_uncertain_is_recon(self):
        idx = _index([_unit("E-10", "uncertain")])
        self.assertEqual(cd.classify(cd._units_by_id(idx)["E-10"], idx, lambda u: False), cd.RECON)

    def test_dark_no_superseder_is_build(self):
        idx = _index([_unit("I-25", "dark")])
        self.assertEqual(cd.classify(cd._units_by_id(idx)["I-25"], idx, lambda u: False), cd.BUILD)

    def test_live_shipped_overrides_to_done(self):
        # A unit the walk already landed is DONE regardless of its (stale) card state.
        idx = _index([_unit("I-6", "dark")])
        self.assertEqual(cd.classify(cd._units_by_id(idx)["I-6"], idx, lambda uid: uid == "I-6"),
                         cd.DONE)


class OrderRespectsDependency(unittest.TestCase):
    def test_d_foundation_before_extension(self):
        # Z-9 supersedes (extends) Z-2, so the foundation Z-2 must precede Z-9 in the drain order.
        idx = _index([_unit("Z-9", "dark", supersedes=["Z-2"]),
                      _unit("Z-2", "dark")])
        # Both are BUILD; use a lambda that ships nothing.
        ids = cd.order(idx, lambda uid: False)
        self.assertIn("Z-2", ids)
        self.assertIn("Z-9", ids)
        self.assertLess(ids.index("Z-2"), ids.index("Z-9"),
                        "foundation Z-2 must be built before extension Z-9")

    def test_done_and_skip_excluded_from_order(self):
        idx = _index([_unit("A-9", "wired", residual="none"),          # DONE
                      _unit("C-54", "dark", superseded_by=["E-9"]),    # SKIP
                      _unit("E-9", "wired"),                            # DONE (wired)
                      _unit("I-25", "dark")])                           # BUILD
        ids = cd.order(idx, lambda uid: False)
        self.assertIn("I-25", ids)
        self.assertNotIn("A-9", ids)
        self.assertNotIn("C-54", ids)   # skipped-superseded excluded
        self.assertNotIn("E-9", ids)    # wired excluded

    def test_gated_M_after_non_M_same_tier(self):
        idx = _index([_unit("M-1", "dark"), _unit("K-99", "dark")])
        ids = cd.order(idx, lambda uid: False)
        self.assertLess(ids.index("K-99"), ids.index("M-1"))

    def test_n1_keystone_first(self):
        idx = _index([_unit("N-5", "dark"), _unit("N-1", "dark"), _unit("B-99", "dark")])
        ids = cd.order(idx, lambda uid: False)
        self.assertEqual(ids[0], "N-1", "N-1 measurement spine must lead the order")


class BuildBriefSmoke(unittest.TestCase):
    def test_brief_lists_shipped_superseder_as_dont_revert(self):
        idx = _index([_unit("C-54", "dark", superseded_by=["E-9"], residual="whole build"),
                      _unit("E-9", "wired", does="SWE-Live/Pro/tau2 benchmarks")])
        brief = cd.build_brief("C-54", idx, lambda uid: False)
        self.assertIn("DRAIN CARD C-54", brief)
        self.assertIn("RESPECT", brief)
        self.assertIn("E-9", brief)  # shipped superseder surfaced as a don't-revert anchor

    def test_brief_missing_unit(self):
        idx = _index([_unit("B-1", "wired")])
        self.assertIn("not found", cd.build_brief("Z-99", idx, lambda uid: False))


class ParserRobustness(unittest.TestCase):
    def test_parse_all_four_formats(self):
        # One card in each of the four analyst formats + a shared SUPERSESSION_EDGES block.
        text_bcd = (
            "SUPERSESSION_EDGES\n"
            "- E-9 SUPERSEDES C-54 (drop SWE-bench Verified)\n"
            "=================================================================\n"
            "## B-1 path unification\n"
            "does: promote paths.\n"
            "state: wired (SHIPPED s-59964).\n"
            "supersedes[]: — | superseded_by[]: —\n"
            "residual: none.\n"
            "key_files: crates/core/src/paths.rs\n"
            "respect[]: never reintroduce .elai/ literals.\n"
        )
        text_efgh = (
            "--- E-9 agent eval benchmarks | does: SWE-Live/Pro/tau2. | state: partial (SCAFFOLDING). "
            "| superseded_by[]: — | residual: live bench run. | key_files: benchmark.rs. | respect[]: E-49.\n"
        )
        text_ijkl = (
            "id: I-30 | does: verifier-ranked SFT exhaust | state: dark (SftExhaust absent) "
            "| superseded_by: J-50 (recovery-trajectory extension) | residual: whole build "
            "| key_files: sft_exhaust.rs | respect: J-50\n"
        )
        text_mnopq = (
            "id: M-1  seat entitlement + purchase rail  [KEYSTONE]\n"
            "does: Offline-verifiable per-seat entitlement.\n"
            "state: dark — EntitlementEvent absent from fresh map (0 refs); GATE M-0.\n"
            "supersedes[]: none  superseded_by[]: none\n"
            "residual: entire unit + +EntitlementEvent SSB row.\n"
            "key_files[]: (unbuilt) — sequences K-24. respect[]: K-24, J-49, MD-1 fail-open floor.\n"
        )
        with tempfile.TemporaryDirectory() as d:
            for name, txt in (("cards-BCD.md", text_bcd), ("cards-EFGH.md", text_efgh),
                              ("cards-IJKL.md", text_ijkl), ("cards-MNOPQ.md", text_mnopq)):
                with open(os.path.join(d, name), "w", encoding="utf-8") as fh:
                    fh.write(txt)
            idx = cd.build_index(os.path.join(d, "cards-*.md"))
        by_id = cd._units_by_id(idx)
        self.assertEqual(set(by_id), {"B-1", "E-9", "I-30", "M-1"})
        self.assertEqual(by_id["B-1"]["state"], "wired")
        self.assertEqual(by_id["E-9"]["state"], "partial")
        self.assertEqual(by_id["I-30"]["state"], "dark")
        self.assertEqual(by_id["I-30"]["superseded_by"], ["J-50"])
        self.assertEqual(by_id["M-1"]["state"], "dark")
        # SUPERSESSION_EDGES union: E-9 SUPERSEDES C-54 -> C-54 unknown here, but the edge is captured.
        self.assertTrue(any(e["newer"] == "E-9" and e["older"] == "C-54" for e in idx["edges"]))
        # key_files parsed for each style
        self.assertIn("crates/core/src/paths.rs", by_id["B-1"]["key_files"])
        self.assertTrue(by_id["I-30"]["key_files"])

    def test_none_tokens_yield_empty(self):
        self.assertEqual(cd._extract_ids("—"), [])
        self.assertEqual(cd._extract_ids("none"), [])
        self.assertEqual(cd._extract_ids("-"), [])
        self.assertEqual(cd._extract_ids("SUPERSEDED by E-34 (was wrong); CLOSED by C-21"),
                         ["E-34", "C-21"])

    def test_state_normalization(self):
        self.assertEqual(cd._normalize_state("WIRED (SHIPPED)"), "wired")
        self.assertEqual(cd._normalize_state("partial->wired (SHIPPED)"), "partial")
        self.assertEqual(cd._normalize_state("dark (FINISH SHIPPED_WIRE_DARK)"), "dark")
        self.assertEqual(cd._normalize_state("REFUTED — RAN"), "refuted")
        self.assertEqual(cd._normalize_state("uncertain (DOC)"), "uncertain")
        self.assertEqual(cd._normalize_state("NOT_STARTED (blocked)"), "dark")


if __name__ == "__main__":
    unittest.main(verbosity=2)
