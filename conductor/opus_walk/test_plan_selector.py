#!/usr/bin/env python3
"""Unit tests for plan_selector - the free-gated auto-seed. Framework: stdlib unittest. Every test
injects fakes (status_reader / dirty_reader / post_fn) so nothing touches the network, git, or the
F: tracker. The contract under test: fail CLOSED to a halt (return None) on stale/unreadable status,
no eligible gap, a dirty-file overlap, an already-classified gap, or a doubtful quorum; only return a
seed when a clean eligible gap wins a free-model plurality."""
import hashlib
import json
import os
import tempfile
import unittest
from pathlib import Path

import plan_selector as ps

# Point the SUMMARY/synthesis sources at nonexistent paths so the pre-existing select_next tests keep
# their PUNCH-ONLY semantics (pool == punch) deterministically without reading the live /mnt/f tree.
_NO_SUMMARY = "/nonexistent-summary-dir"
_NO_SYNTH = "/nonexistent-synthesis.md"


def _status(punch, age_min=10.0):
    return {"snapshot_age_min": age_min, "punch": punch}


def _voter(text, ok=True):
    return {"ok": ok, "text": text, "status": 200 if ok else None, "error": None, "latency_ms": 5}


def _post_all(text):
    """A post_fn where every voter returns the same text."""
    return lambda model, prompt: _voter(text)


class TestExtractFiles(unittest.TestCase):
    def test_basenames_from_desc(self):
        desc = "leak at task_runner.rs:15303 and production_agent_rpc.rs:86; mount PagerankPanel.svelte"
        self.assertEqual(ps._extract_files(desc),
                         {"task_runner.rs", "production_agent_rpc.rs", "PagerankPanel.svelte"})

    def test_empty(self):
        self.assertEqual(ps._extract_files(""), set())
        self.assertEqual(ps._extract_files("no files here"), set())


class TestParsePick(unittest.TestCase):
    def test_longest_id_wins(self):
        valid = {"H-28", "H-28-WIRE-1"}
        self.assertEqual(ps._parse_pick("i pick H-28-WIRE-1 clearly", valid), "H-28-WIRE-1")

    def test_none_vote(self):
        self.assertIsNone(ps._parse_pick("NONE of these are safe", {"H-10", "C-15"}))
        self.assertIsNone(ps._parse_pick("", {"H-10"}))

    def test_word_boundary(self):
        # 'B-4' must not match inside 'B-40'
        self.assertIsNone(ps._parse_pick("the fix is B-40 related", {"B-4"}))
        self.assertEqual(ps._parse_pick("do B-4 next", {"B-4"}), "B-4")


class TestEligibleGaps(unittest.TestCase):
    def _run(self, punch, state=None, dirty=None):
        state = state or {"queue": [], "escalation_queue": []}
        return ps.eligible_gaps(_status(punch), state, dirty or set(), "/nonexistent-plan-status-dir")

    def test_skips_escalated_and_drained(self):
        punch = [
            {"id": "I-33", "desc": "x", "disposition": {"verdict": "escalated"}},
            {"id": "H-28", "desc": "y", "disposition": {"verdict": "drained"}},
            {"id": "C-15", "desc": "regressed task_runner.rs:1"},
        ]
        self.assertEqual([g["id"] for g in self._run(punch)], ["C-15"])

    def test_skips_queued_and_escalated_ids(self):
        punch = [{"id": "A-1", "desc": "a"}, {"id": "A-2", "desc": "b"}, {"id": "A-3", "desc": "c"}]
        state = {"queue": ["A-1"], "escalation_queue": [{"plan_id": "A-2"}]}
        self.assertEqual([g["id"] for g in self._run(punch, state=state)], ["A-3"])

    def test_skips_dirty_file_overlap(self):
        punch = [
            {"id": "I-23", "desc": "bypass in production_agent_rpc.rs:86"},
            {"id": "H-10", "desc": "mount PagerankPanel.svelte"},
        ]
        dirty = {"production_agent_rpc.rs", "Cargo.lock"}
        self.assertEqual([g["id"] for g in self._run(punch, dirty=dirty)], ["H-10"])

    def test_skips_non_plan_id_gaps(self):
        punch = [
            {"id": "glm-orchestrator dirt-baseline", "desc": "bug in orchestrator.py"},
            {"id": "H Track-3", "desc": "capstone gates"},
            {"id": "I-23", "desc": "bypass"},
            {"id": "H-28-WIRE-1", "desc": "ok"},
        ]
        self.assertEqual([g["id"] for g in self._run(punch)], ["I-23", "H-28-WIRE-1"])

    def test_skips_plan_done(self):
        punch = [{"id": "DONE-1", "desc": "a"}, {"id": "OPEN-1", "desc": "b"}]
        orig = ps.quorum_triage._plan_done
        ps.quorum_triage._plan_done = lambda pid, d: pid == "DONE-1"
        try:
            self.assertEqual([g["id"] for g in self._run(punch)], ["OPEN-1"])
        finally:
            ps.quorum_triage._plan_done = orig


class TestQuorumSelect(unittest.TestCase):
    ELIG = [{"id": "H-10", "desc": "mount"}, {"id": "C-15", "desc": "regressed"}]

    def test_plurality_winner(self):
        winner, detail = ps.quorum_select(self.ELIG, post_fn=_post_all("H-10"))
        self.assertEqual(winner, "H-10")
        self.assertEqual(detail["reason"], "PICK")

    def test_plurality_2_1_wins_outright(self):
        # two voters for H-10, one for C-15 -> clear plurality, no tie-break needed.
        calls = iter(["H-10", "H-10", "C-15"])
        post = lambda m, p: _voter(next(calls))
        winner, detail = ps.quorum_select(self.ELIG, post_fn=post)
        self.assertEqual(winner, "H-10")
        self.assertEqual(detail["reason"], "PICK")

    def test_tie_breaks_to_pool_order(self):
        # 1-1 split with >=2 parseable votes: every voter affirmed an actionable plan, so this is
        # preference not doubt -> break to the pool-order leader (H-10 is first in ELIG), not a halt.
        calls = iter(["H-10", "C-15", "garbage"])
        post = lambda m, p: _voter(next(calls))
        winner, detail = ps.quorum_select(self.ELIG, post_fn=post)
        self.assertEqual(winner, "H-10")
        self.assertEqual(detail["reason"], "TIE_BROKEN")
        self.assertEqual(detail["tally"], {"H-10": 1, "C-15": 1})

    def test_tie_breaks_to_pool_order_reversed(self):
        # same split but C-15 leads the pool this time -> tie-break follows pool order to C-15.
        elig = [{"id": "C-15", "desc": "regressed"}, {"id": "H-10", "desc": "mount"}]
        calls = iter(["H-10", "C-15", "garbage"])
        post = lambda m, p: _voter(next(calls))
        winner, detail = ps.quorum_select(elig, post_fn=post)
        self.assertEqual(winner, "C-15")
        self.assertEqual(detail["reason"], "TIE_BROKEN")

    def test_single_parseable_vote_is_thin_quorum(self):
        # only ONE voter parses a valid id (the live defect: a lone voter must not decide a paid boot).
        calls = iter(["H-10", "garbage", "also garbage"])
        post = lambda m, p: _voter(next(calls))
        winner, detail = ps.quorum_select(self.ELIG, post_fn=post)
        self.assertIsNone(winner)
        self.assertEqual(detail["reason"], "QUORUM_THIN")
        self.assertEqual(detail["tally"], {"H-10": 1})

    def test_all_none_at_floor_halts(self):
        # every voter engages and abstains: floor met (3 parseable) but zero id-picks -> ALL_NONE,
        # the genuine "nothing actionable" fail-closed halt (distinct from NO_PICK = nobody engaged).
        winner, detail = ps.quorum_select(self.ELIG, post_fn=_post_all("NONE"))
        self.assertIsNone(winner)
        self.assertEqual(detail["reason"], "ALL_NONE")
        self.assertEqual(detail["parseable"], 3)
        self.assertEqual(detail["tally"], {})

    def test_none_plus_id_meets_floor_and_seeds(self):
        # an explicit NONE is a PARSEABLE abstention: NONE + one id-pick = 2 parseable (floor met),
        # plurality over id-picks alone -> the id wins. This is the live swe+glm shape that must seed.
        calls = iter(["NONE", "H-10", "let me look around the repo first"])
        post = lambda m, p: _voter(next(calls))
        winner, detail = ps.quorum_select(self.ELIG, post_fn=post)
        self.assertEqual(winner, "H-10")
        self.assertEqual(detail["reason"], "PICK")
        self.assertEqual(detail["parseable"], 2)
        self.assertEqual(detail["tally"], {"H-10": 1})

    def test_agentic_derail_is_unparseable(self):
        # kimi-class agentic replies (tool-chatter, no id, no NONE) are unparseable, never abstentions.
        ramble = "I'll investigate the repository structure. Let me look around and read the plans."
        self.assertEqual(ps._classify_vote(ramble, {"H-10", "C-15"}), ("unparseable", None))
        winner, detail = ps.quorum_select(self.ELIG, post_fn=_post_all(ramble))
        self.assertIsNone(winner)
        self.assertEqual(detail["reason"], "NO_PICK")
        self.assertEqual(detail["parseable"], 0)

    def test_first_line_decides_over_body_mentions(self):
        # the hardened prompt demands the pick be the FIRST LINE: a first-line NONE stays an
        # abstention even if the explanation quotes ballot ids; a first-line id wins likewise.
        valid = {"H-10", "C-15"}
        self.assertEqual(ps._classify_vote("NONE\nthough H-10 came close", valid), ("none", None))
        self.assertEqual(ps._classify_vote("H-10\nC-15 was my second choice", valid), ("id", "H-10"))

    def test_prompt_forbids_tools_and_pins_first_line(self):
        # the anti-derail directives must reach every voter verbatim.
        prompt = ps._select_prompt(self.ELIG)
        self.assertIn("Do NOT use tools", prompt)
        self.assertIn("do NOT investigate", prompt)
        self.assertIn("FIRST LINE", prompt)

    def test_ballot_capped_at_select_ballot_max(self):
        # a wide pool: voters must only ever see the top SELECT_BALLOT_MAX candidates (#K+1 hidden).
        n = ps.SELECT_BALLOT_MAX
        pool = [{"id": f"Q-{i}", "desc": f"unit {i}"} for i in range(1, n + 3)]
        seen = []
        post = lambda m, p: (seen.append(p), _voter("Q-1"))[1]
        winner, _ = ps.quorum_select(pool, post_fn=post)
        self.assertEqual(winner, "Q-1")
        prompt = seen[0]
        self.assertIn(f"Q-{n}", prompt)          # last on-ballot candidate is shown
        self.assertNotIn(f"Q-{n + 1}", prompt)   # candidate #K+1 never reaches the voters
        self.assertNotIn(f"Q-{n + 2}", prompt)


class TestSelectNext(unittest.TestCase):
    STATE = {"queue": [], "escalation_queue": []}
    PUNCH = [{"id": "H-10", "desc": "orphan; mount PagerankPanel.svelte"},
             {"id": "C-15", "desc": "regressed task_runner.rs:1"}]

    # disp_reader=lambda: set() keeps these tests hermetic: the live plan-dispositions.json holds
    # H-10/C-15/I-23/I-33, the exact PUNCH ids here, so a live ledger read would wrongly exclude them.
    def test_happy_path_returns_seed(self):
        pick = ps.select_next(
            self.STATE, "/nonexistent", status_reader=lambda: _status(self.PUNCH),
            dirty_reader=lambda: set(), post_fn=_post_all("H-10"),
            summary_dir=_NO_SUMMARY, synthesis_path=_NO_SYNTH, disp_reader=lambda: set())
        self.assertIsNotNone(pick)
        self.assertEqual(pick["plan_id"], "H-10")
        self.assertIn("AUTO-SELECTED", pick["pickup"])

    def test_stale_snapshot_returns_none(self):
        pick = ps.select_next(
            self.STATE, "/nonexistent",
            status_reader=lambda: _status(self.PUNCH, age_min=ps.AUTOSEED_MAX_SNAPSHOT_AGE_MIN + 1),
            dirty_reader=lambda: set(), post_fn=_post_all("H-10"),
            summary_dir=_NO_SUMMARY, synthesis_path=_NO_SYNTH, disp_reader=lambda: set())
        self.assertIsNone(pick)

    def test_unreadable_status_returns_none(self):
        pick = ps.select_next(self.STATE, "/nonexistent", status_reader=lambda: None,
                              dirty_reader=lambda: set(), post_fn=_post_all("H-10"),
                              summary_dir=_NO_SUMMARY, synthesis_path=_NO_SYNTH,
                              disp_reader=lambda: set())
        self.assertIsNone(pick)

    def test_no_eligible_returns_none(self):
        # PUNCH-only, escalated -> empty punch AND no summary source -> empty pool -> None.
        punch = [{"id": "I-33", "desc": "x", "disposition": {"verdict": "escalated"}}]
        pick = ps.select_next(self.STATE, "/nonexistent", status_reader=lambda: _status(punch),
                              dirty_reader=lambda: set(), post_fn=_post_all("I-33"),
                              summary_dir=_NO_SUMMARY, synthesis_path=_NO_SYNTH,
                              disp_reader=lambda: set())
        self.assertIsNone(pick)

    def test_doubtful_quorum_returns_none(self):
        pick = ps.select_next(
            self.STATE, "/nonexistent", status_reader=lambda: _status(self.PUNCH),
            dirty_reader=lambda: set(), post_fn=_post_all("NONE"),
            summary_dir=_NO_SUMMARY, synthesis_path=_NO_SYNTH, disp_reader=lambda: set())
        self.assertIsNone(pick)


# --- F-1 pool-expansion fixtures + tests -------------------------------------------------

_B_SUMMARY = """# B-Plan Summary
## 3. Per-Session Index
### B-1: Done thing  [DONE s-1]
- Files: b1.rs
### B-2: Partial thing  [PARTIAL — decomposition landed]
- Files: b2.rs
### B-3: Planned thing  [PLANNED]
- Files: b3.rs
### B-9: Deferred thing  [DEFERRED — conditional]
- Files: b9.rs
"""

_E_SUMMARY = """# E-plan Summary Index
### E-1 — Open unit alpha
- Closes: 01-alpha
- Files: crates/memory/e1.rs; +some_ssb SSB
- Smoke: x
### E-2 — Open unit beta
- Files: e2.svelte
### E-5 — Dirty overlap unit
- Files: dirtyfile.rs
"""

_D_SUMMARY = """# D-plan Summary
### Wave 1 — no per-unit rows here
- just a wave header, not a seedable unit
"""

_SYNTHESIS = """MASTER SYNTHESIS
PUNCH LIST
 1. E-1 open alpha needs its consumer wired (audit-confirmed OPEN).
 E PARTIAL itemized: E-1.
 H Track-3 capstone (free text, not an id).
"""


def _write_summary_tree():
    """Build a temp docs/thefinalglance-shaped tree + a synthesis file. Returns (root, synth_path)."""
    root = Path(tempfile.mkdtemp(prefix="plansel_summary_"))
    for letter, body in (("B", _B_SUMMARY), ("E", _E_SUMMARY), ("D", _D_SUMMARY)):
        d = root / f"{letter}-plan"
        d.mkdir(parents=True)
        name = "B-PLAN-SUMMARY.md" if letter == "B" else f"{letter}-SUMMARY_INDEX.md"
        (d / name).write_text(body, encoding="utf-8")
    synth = root / "_MASTER-SYNTHESIS.md"
    synth.write_text(_SYNTHESIS, encoding="utf-8")
    return root, synth


class TestSummaryParse(unittest.TestCase):
    def setUp(self):
        self.root, self.synth = _write_summary_tree()

    def tearDown(self):
        import shutil
        shutil.rmtree(self.root, ignore_errors=True)

    def test_classify_status(self):
        self.assertEqual(ps._classify_summary_status("DONE s-1"), "DONE")
        self.assertEqual(ps._classify_summary_status("PARTIAL — x DONE"), "PARTIAL")  # PARTIAL wins
        self.assertEqual(ps._classify_summary_status("PLANNED"), "PLANNED")
        self.assertEqual(ps._classify_summary_status("DEFERRED — later"), "DEFERRED")
        self.assertEqual(ps._classify_summary_status("KEYSTONE"), "OPEN")
        self.assertEqual(ps._classify_summary_status(""), "OPEN")

    def test_parse_ids_and_status(self):
        units, ok = ps._parse_summary_file(self.root / "B-plan" / "B-PLAN-SUMMARY.md")
        self.assertTrue(ok)
        by_id = {u["id"]: u for u in units}
        self.assertEqual(set(by_id), {"B-1", "B-2", "B-3", "B-9"})
        self.assertEqual(by_id["B-1"]["status"], "DONE")
        self.assertEqual(by_id["B-2"]["status"], "PARTIAL")
        self.assertEqual(by_id["B-3"]["status"], "PLANNED")
        self.assertEqual(by_id["B-9"]["status"], "DEFERRED")
        self.assertEqual(by_id["B-2"]["files"], ["b2.rs"])
        # title strips the trailing [..] status bracket
        self.assertEqual(by_id["B-2"]["desc"], "Partial thing")

    def test_wave_grain_yields_no_units(self):
        units, ok = ps._parse_summary_file(self.root / "D-plan" / "D-SUMMARY_INDEX.md")
        self.assertTrue(ok)  # read succeeded
        self.assertEqual(units, [])  # but no ### D-N rows -> nothing seedable

    def test_read_error_returns_not_ok(self):
        units, ok = ps._parse_summary_file(self.root / "does-not-exist.md")
        self.assertFalse(ok)
        self.assertEqual(units, [])

    def test_summary_candidates_excludes_empty_letters(self):
        units, excluded = ps.summary_candidates(self.root)
        got = {u["id"] for u in units}
        self.assertEqual(got, {"B-1", "B-2", "B-3", "B-9", "E-1", "E-2", "E-5"})
        # D parsed (wave-grain -> 0 units) and every letter with no dir -> excluded
        self.assertIn("D", excluded)
        self.assertNotIn("B", excluded)
        self.assertNotIn("E", excluded)
        for missing in ("C", "F", "J", "Q"):
            self.assertIn(missing, excluded)

    def test_synthesis_named_ids(self):
        named = ps.synthesis_named_ids(self.synth)
        self.assertIn("E-1", named)
        self.assertNotIn("E-2", named)

    def test_synthesis_missing_returns_empty(self):
        self.assertEqual(ps.synthesis_named_ids("/nonexistent-synth.md"), set())


class TestBuildPool(unittest.TestCase):
    def setUp(self):
        self.root, self.synth = _write_summary_tree()
        self.state = {"queue": [], "escalation_queue": []}
        self.punch = [{"id": "P-1", "desc": "punch head fix task_runner.rs:1"}]

    def tearDown(self):
        import shutil
        shutil.rmtree(self.root, ignore_errors=True)

    def _pool(self, dirty=None, state=None):
        # dispositioned=set() keeps these tests hermetic (no live plan-dispositions.json read).
        return ps.build_pool(_status(self.punch), state or self.state, dirty or set(),
                             "/nonexistent-plan-status", summary_dir=self.root,
                             synthesis_path=self.synth, dispositioned=set())

    def test_ordering_punch_then_audited_then_recon(self):
        pool, stats = self._pool()
        ids = [g["id"] for g in pool]
        # punch head first
        self.assertEqual(ids[0], "P-1")
        # DONE (B-1) + DEFERRED (B-9) dropped
        self.assertNotIn("B-1", ids)
        self.assertNotIn("B-9", ids)
        # audited-open (B-2 PARTIAL, E-1 synthesis-named) rank ABOVE recon-first (B-3, E-2)
        self.assertLess(ids.index("B-2"), ids.index("B-3"))
        self.assertLess(ids.index("E-1"), ids.index("E-2"))
        self.assertLess(ids.index("E-1"), ids.index("B-3"))
        self.assertEqual(stats["excluded_letters"].count("D"), 1)

    def test_recon_first_flag(self):
        pool, _ = self._pool()
        flag = {g["id"]: g["recon_first"] for g in pool}
        self.assertFalse(flag["P-1"])
        self.assertFalse(flag["B-2"])   # PARTIAL = audited
        self.assertFalse(flag["E-1"])   # synthesis-named = audited
        self.assertTrue(flag["B-3"])    # PLANNED, unnamed = recon-first
        self.assertTrue(flag["E-2"])    # OPEN, unnamed = recon-first

    def test_dirty_overlap_dropped(self):
        pool, _ = self._pool(dirty={"dirtyfile.rs"})
        self.assertNotIn("E-5", [g["id"] for g in pool])

    def test_queued_and_escalated_dropped(self):
        state = {"queue": ["B-2"], "escalation_queue": [{"plan_id": "E-1"}]}
        ids = [g["id"] for g in self._pool(state=state)[0]]
        self.assertNotIn("B-2", ids)
        self.assertNotIn("E-1", ids)

    def test_plan_done_dropped(self):
        orig = ps.quorum_triage._plan_done
        ps.quorum_triage._plan_done = lambda pid, d: pid == "E-2"
        try:
            ids = [g["id"] for g in self._pool()[0]]
            self.assertNotIn("E-2", ids)
        finally:
            ps.quorum_triage._plan_done = orig

    def test_stats_counts(self):
        _, stats = self._pool()
        self.assertEqual(stats["punch"], 1)
        # audited: B-2, E-1 ; recon: B-3, E-2 ; E-5 clean here (no dirty) -> recon too
        self.assertEqual(stats["audited_open"], 2)
        self.assertEqual(stats["recon_first"], 3)
        self.assertEqual(stats["summary"], 5)

    def test_summary_pool_cap(self):
        orig = ps.SUMMARY_POOL_MAX
        ps.SUMMARY_POOL_MAX = 2
        try:
            _, stats = self._pool()
            self.assertEqual(stats["summary"], 2)          # capped
            self.assertEqual(stats["summary_total_eligible"], 5)  # but true eligible count preserved
        finally:
            ps.SUMMARY_POOL_MAX = orig


class TestSelectNextExpansion(unittest.TestCase):
    """The F-1 headline: a DRAINED punch head no longer halts -- select_next refills from the summary
    pool. Injected summary_dir/synthesis_path so no /mnt/f read."""

    def setUp(self):
        self.root, self.synth = _write_summary_tree()
        # punch head is escalated-only -> zero eligible punch, forcing the summary refill path.
        self.punch = [{"id": "P-9", "desc": "x", "disposition": {"verdict": "escalated"}}]

    def tearDown(self):
        import shutil
        shutil.rmtree(self.root, ignore_errors=True)

    def _select(self, vote):
        return ps.select_next(
            {"queue": [], "escalation_queue": []}, "/nonexistent",
            status_reader=lambda: _status(self.punch), dirty_reader=lambda: set(),
            post_fn=_post_all(vote), summary_dir=self.root, synthesis_path=self.synth,
            disp_reader=lambda: set())

    def test_drained_punch_refills_from_summary(self):
        pick = self._select("E-1")
        self.assertIsNotNone(pick)                      # did NOT halt
        self.assertEqual(pick["plan_id"], "E-1")
        self.assertFalse(pick["detail"]["recon_first"])  # E-1 is synthesis-named = audited
        self.assertIn("AUTO-SELECTED", pick["pickup"])
        self.assertNotIn("RECON-FIRST", pick["pickup"])

    def test_recon_first_unit_carries_flag_in_pickup(self):
        pick = self._select("E-2")
        self.assertEqual(pick["plan_id"], "E-2")
        self.assertTrue(pick["detail"]["recon_first"])
        self.assertIn("RECON-FIRST", pick["pickup"])

    def test_pool_stats_in_detail(self):
        pick = self._select("E-1")
        self.assertIn("pool", pick["detail"])
        self.assertEqual(pick["detail"]["pool"]["punch"], 0)  # escalated punch -> 0 eligible


class TestBallotRotation(unittest.TestCase):
    """Head-of-line fix (2026-07-17): an ALL_NONE verdict advances the persisted ballot window over
    the non-punch tail, so a stable-rejected head cannot starve the rest of the pool. PICK does not
    move the window; punch-tier units stay on every ballot."""

    def setUp(self):
        self.root, self.synth = _write_summary_tree()
        self._orig_rot = ps.ROTATION_PATH
        fd, self.rot = tempfile.mkstemp(suffix=".json")
        os.close(fd)
        os.unlink(self.rot)  # start absent -> offset 0
        ps.ROTATION_PATH = Path(self.rot)
        self.punch = [{"id": "P-1", "desc": "punch head fix task_runner.rs:1"}]

    def tearDown(self):
        import shutil
        shutil.rmtree(self.root, ignore_errors=True)
        ps.ROTATION_PATH = self._orig_rot
        if os.path.exists(self.rot):
            os.unlink(self.rot)

    def _select(self, post_fn):
        return ps.select_next(
            {"queue": [], "escalation_queue": []}, "/nonexistent",
            status_reader=lambda: _status(self.punch), dirty_reader=lambda: set(),
            post_fn=post_fn, summary_dir=self.root, synthesis_path=self.synth,
            disp_reader=lambda: set())

    def test_all_none_advances_rotation(self):
        # pool = P-1 + 5-unit tail; ALL_NONE must persist offset = SELECT_BALLOT_MAX - punch_n = 4
        self.assertIsNone(self._select(_post_all("NONE")))
        self.assertEqual(ps._rotation_offset(), (ps.SELECT_BALLOT_MAX - 1) % 5)

    def test_rotated_window_changes_ballot(self):
        prompts = []

        def post(model, prompt):
            prompts.append(prompt)
            return _voter("NONE")

        self._select(post)          # cycle 1: offset 0
        first_window = prompts[0]
        prompts.clear()
        self._select(post)          # cycle 2: rotated
        second_window = prompts[0]
        self.assertNotEqual(first_window, second_window)   # the ballot actually moved
        self.assertIn("P-1", second_window)                # punch tier never rotates out

    def test_pick_does_not_advance_rotation(self):
        pick = self._select(_post_all("E-1"))
        self.assertEqual(pick["plan_id"], "E-1")
        self.assertEqual(ps._rotation_offset(), 0)

    def test_rotation_wraps_over_tail(self):
        # 5-unit tail, step 4: offsets walk 4 -> 3 -> 2 (mod 5), never out of range.
        self._select(_post_all("NONE"))
        self._select(_post_all("NONE"))
        self.assertEqual(ps._rotation_offset(), (2 * (ps.SELECT_BALLOT_MAX - 1)) % 5)

    def test_unreadable_rotation_file_is_offset_zero(self):
        Path(self.rot).write_text("not json", encoding="utf-8")
        self.assertEqual(ps._rotation_offset(), 0)


# --- WIP-guard independent-legs + fail-closed floor (T3b-entry10) ------------------------

class TestDirtyGuardIndependentLegs(unittest.TestCase):
    """The two git legs of the WIP guard run independently: a failure of one NEVER drops the other,
    and only BOTH failing blinds the guard (dirty_basenames -> None -> caller halts). Patches the leg
    helpers so nothing touches git."""

    def setUp(self):
        self._orig_status = ps._git_status_basenames
        self._orig_stash = ps._stashed_basenames

    def tearDown(self):
        ps._git_status_basenames = self._orig_status
        ps._stashed_basenames = self._orig_stash

    def test_status_leg_fails_stash_leg_survives(self):
        # status times out (None), stash still names the parked file -> union keeps the stash file.
        ps._git_status_basenames = lambda repo=None: None
        ps._stashed_basenames = lambda repo=None: {"production_agent_rpc.rs"}
        self.assertEqual(ps.dirty_basenames(), {"production_agent_rpc.rs"})

    def test_stash_leg_fails_status_leg_survives(self):
        ps._git_status_basenames = lambda repo=None: {"foo.rs"}
        ps._stashed_basenames = lambda repo=None: None
        self.assertEqual(ps.dirty_basenames(), {"foo.rs"})

    def test_both_legs_survive_union(self):
        ps._git_status_basenames = lambda repo=None: {"foo.rs"}
        ps._stashed_basenames = lambda repo=None: {"production_agent_rpc.rs"}
        self.assertEqual(ps.dirty_basenames(), {"foo.rs", "production_agent_rpc.rs"})

    def test_both_legs_fail_returns_none(self):
        ps._git_status_basenames = lambda repo=None: None
        ps._stashed_basenames = lambda repo=None: None
        self.assertIsNone(ps.dirty_basenames())

    def test_no_stash_is_not_a_leg_failure(self):
        # stash rc!=0 (no stash) -> empty set, NOT None; a clean working tree stays a real empty guard.
        ps._git_status_basenames = lambda repo=None: set()
        ps._stashed_basenames = lambda repo=None: set()
        self.assertEqual(ps.dirty_basenames(), set())  # empty guard, not blind


class TestDirtyGuardEndToEnd(unittest.TestCase):
    """select_next behaviour when the guard legs misbehave, driving the REAL dirty_basenames (legs
    patched) through select_next -> build_pool -> eligible_gaps."""

    STATE = {"queue": [], "escalation_queue": []}

    def setUp(self):
        self._orig_status = ps._git_status_basenames
        self._orig_stash = ps._stashed_basenames

    def tearDown(self):
        ps._git_status_basenames = self._orig_status
        ps._stashed_basenames = self._orig_stash

    def test_status_timeout_stash_still_skips_stashed_candidate(self):
        # Live-defect regression: status leg times out but the stashed file must STILL be guarded, so a
        # candidate naming it is skipped while a clean candidate survives and can be seeded.
        ps._git_status_basenames = lambda repo=None: None  # timeout / rc!=0
        ps._stashed_basenames = lambda repo=None: {"production_agent_rpc.rs"}
        punch = [{"id": "I-23", "desc": "bypass in production_agent_rpc.rs:86"},
                 {"id": "H-10", "desc": "orphan; mount PagerankPanel.svelte"}]
        pick = ps.select_next(
            self.STATE, "/nonexistent", status_reader=lambda: _status(punch),
            dirty_reader=ps.dirty_basenames, post_fn=_post_all("H-10"),
            summary_dir=_NO_SUMMARY, synthesis_path=_NO_SYNTH, disp_reader=lambda: set())
        self.assertIsNotNone(pick)
        self.assertEqual(pick["plan_id"], "H-10")
        self.assertNotIn("I-23", pick["detail"]["eligible"])  # stashed file guarded despite status fail

    def test_both_legs_blind_halts(self):
        ps._git_status_basenames = lambda repo=None: None
        ps._stashed_basenames = lambda repo=None: None
        punch = [{"id": "H-10", "desc": "orphan; mount PagerankPanel.svelte"}]
        pick = ps.select_next(
            self.STATE, "/nonexistent", status_reader=lambda: _status(punch),
            dirty_reader=ps.dirty_basenames, post_fn=_post_all("H-10"),
            summary_dir=_NO_SUMMARY, synthesis_path=_NO_SYNTH, disp_reader=lambda: set())
        self.assertIsNone(pick)  # fail-closed: blind guard halts rather than seed


# --- T6-entry10 reseed filter + seed-attempt ledger --------------------------------------

class TestSeedAttemptLedger(unittest.TestCase):
    def setUp(self):
        self.d = tempfile.mkdtemp(prefix="seedledger_")
        self.p = os.path.join(self.d, "seed-attempts.json")

    def tearDown(self):
        import shutil
        shutil.rmtree(self.d, ignore_errors=True)

    def test_record_increments_and_persists(self):
        self.assertEqual(ps.record_seed_attempt("C-47", self.p), 1)
        self.assertEqual(ps.record_seed_attempt("C-47", self.p), 2)
        self.assertEqual(ps.record_seed_attempt("H-10", self.p), 1)
        self.assertEqual(ps.load_seed_attempts(self.p), {"C-47": 2, "H-10": 1})

    def test_load_missing_is_empty(self):
        self.assertEqual(ps.load_seed_attempts("/nonexistent-seed-ledger.json"), {})


class TestReseedFilter(unittest.TestCase):
    """build_pool excludes a candidate seeded >= RESEED_LIMIT times that STILL has no disposition and is
    not plan-DONE (the paid-turn-burning re-seed loop), logging one SKIP_RESEED line per exclusion."""

    STATE = {"queue": [], "escalation_queue": []}

    def _pool(self, punch, seed_attempts, dispositioned):
        # punch-only pool (no summary source) so the filter behaviour is isolated.
        return ps.build_pool(_status(punch), self.STATE, set(), "/nonexistent-plan-status",
                             summary_dir=_NO_SUMMARY, synthesis_path=_NO_SYNTH,
                             seed_attempts=seed_attempts, dispositioned=dispositioned)

    def test_excludes_2x_seeded_undispositioned_and_logs(self):
        import contextlib
        import io
        punch = [{"id": "C-47", "desc": "already-delivered seam"}, {"id": "H-10", "desc": "mount x"}]
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            pool, stats = self._pool(punch, {"C-47": 2}, set())
        ids = [g["id"] for g in pool]
        self.assertNotIn("C-47", ids)            # seeded 2x, no disposition -> excluded
        self.assertIn("H-10", ids)               # 0 seeds -> kept
        self.assertEqual(stats["skipped_reseed"], ["C-47"])
        self.assertIn("SKIP_RESEED C-47 (seeded 2x, no disposition)", buf.getvalue())

    def test_one_seed_remains_eligible(self):
        pool, stats = self._pool([{"id": "C-47", "desc": "x"}], {"C-47": 1}, set())
        self.assertEqual([g["id"] for g in pool], ["C-47"])   # 1 < RESEED_LIMIT -> kept
        self.assertEqual(stats["skipped_reseed"], [])

    def test_dispositioned_id_excluded_before_reseed_filter(self):
        # T6b contract: a plan that HAS a disposition is now dropped UNCONDITIONALLY by the disposition
        # filter (regardless of seed count), BEFORE _drop_reseeded runs -- so it leaves the pool, and
        # skipped_reseed stays empty because the candidate was removed before the reseed filter saw it.
        pool, stats = self._pool([{"id": "C-47", "desc": "x"}], {"C-47": 5}, {"C-47"})
        self.assertEqual([g["id"] for g in pool], [])          # dropped by the disposition filter
        self.assertEqual(stats["dispositioned"], 1)
        self.assertEqual(stats["skipped_reseed"], [])          # never reached the reseed filter

    def test_existing_filter_drops_drained_disposition_regardless_of_seeds(self):
        # the pre-existing eligible_gaps filter drops an escalated/drained punch verdict regardless of
        # the seed count -- the 'a dispositioned id stays excluded by the existing filter' contract.
        punch = [{"id": "C-47", "desc": "x", "disposition": {"verdict": "drained"}},
                 {"id": "H-10", "desc": "mount"}]
        pool, stats = self._pool(punch, {"C-47": 5}, set())
        self.assertEqual([g["id"] for g in pool], ["H-10"])   # C-47 gone via eligible_gaps, not reseed
        self.assertEqual(stats["skipped_reseed"], [])         # nothing reached the reseed filter to skip


class TestDispositionExclusion(unittest.TestCase):
    """T6b: build_pool UNCONDITIONALLY drops any candidate whose id carries a terminal disposition
    (drained/escalated) in the teikoku-local ledger, regardless of seed count -- the SUMMARY-path hole
    that let a drained gap (B-28) get re-seeded a full paid Opus turn before the reseed cap caught it."""

    STATE = {"queue": [], "escalation_queue": []}

    def _pool(self, punch, dispositioned, summary_dir=_NO_SUMMARY, synth=_NO_SYNTH):
        return ps.build_pool(_status(punch), self.STATE, set(), "/nonexistent-plan-status",
                             summary_dir=summary_dir, synthesis_path=synth,
                             seed_attempts={}, dispositioned=dispositioned)

    def test_summary_drained_candidate_excluded(self):
        # E-2 is a summary-enumerated (recon-first) unit with ZERO seeds; a drained disposition must
        # drop it from the pool while a sibling summary unit (E-1) survives. This is the exact hole.
        root, synth = _write_summary_tree()
        try:
            pool, stats = self._pool([{"id": "P-1", "desc": "punch head"}], {"E-2"},
                                     summary_dir=root, synth=synth)
            ids = [g["id"] for g in pool]
            self.assertNotIn("E-2", ids)          # drained summary unit dropped despite 0 seeds
            self.assertIn("E-1", ids)             # a sibling summary unit stays eligible
            self.assertIn("P-1", ids)             # punch head unaffected
            self.assertEqual(stats["dispositioned"], 1)
        finally:
            import shutil
            shutil.rmtree(root, ignore_errors=True)

    def test_escalated_disposition_excluded(self):
        # an escalated ledger row excludes just like a drained one, and with zero seeds.
        pool, stats = self._pool([{"id": "C-47", "desc": "x"}, {"id": "H-10", "desc": "mount"}],
                                 {"C-47"})
        self.assertEqual([g["id"] for g in pool], ["H-10"])
        self.assertEqual(stats["dispositioned"], 1)

    def test_absent_from_ledger_stays_eligible(self):
        # a candidate not in the ledger is untouched; dispositioned count is 0.
        pool, stats = self._pool([{"id": "H-10", "desc": "mount"}], set())
        self.assertEqual([g["id"] for g in pool], ["H-10"])
        self.assertEqual(stats["dispositioned"], 0)

    def test_disposition_filter_before_pool_cap(self):
        """Disposition exclusion must run BEFORE SUMMARY_POOL_MAX: if the highest-ranked
        SUMMARY_POOL_MAX units are all dispositioned, the pool must refill with lower-ranked
        never-dispositioned units rather than going empty (CHAIN_NO_WORK regression 2026-07-13)."""
        import tempfile
        from pathlib import Path as P
        root = tempfile.mkdtemp(prefix="poolcap-")
        try:
            letter_dir = P(root) / "E-plan"
            letter_dir.mkdir()
            lines = ["# E summary\n"]
            n = ps.SUMMARY_POOL_MAX + 5
            for i in range(1, n + 1):
                lines.append(f"### E-{i} unit {i}  [OPEN]\n- files: e{i}.rs\n")
            (letter_dir / "E-SUMMARY.md").write_text("".join(lines), encoding="utf-8")
            synth = P(root) / "synth.md"
            synth.write_text("# no named ids\n", encoding="utf-8")
            disp = {f"E-{i}" for i in range(1, ps.SUMMARY_POOL_MAX + 1)}
            pool, stats = ps.build_pool(
                _status([]), self.STATE, set(), "/nonexistent-plan-status",
                summary_dir=P(root), synthesis_path=synth,
                seed_attempts={}, dispositioned=disp,
            )
            ids = [g["id"] for g in pool]
            self.assertTrue(ids, "pool must be non-empty after filtering dispositioned top-N")
            for d in disp:
                self.assertNotIn(d, ids)
            self.assertIn(f"E-{ps.SUMMARY_POOL_MAX + 1}", ids)
            self.assertEqual(stats["dispositioned"], ps.SUMMARY_POOL_MAX)
            self.assertGreaterEqual(stats["summary"], 1)
        finally:
            import shutil
            shutil.rmtree(root, ignore_errors=True)

    def test_unreadable_ledger_excludes_nothing_no_crash(self):
        # dispositioned=None triggers build_pool's OWN read; a missing/unreadable ledger must yield the
        # empty set (walk_state.dispositioned_ids fail-safe) -> nothing excluded, no crash, no halt.
        orig = ps.walk_state.dispositioned_ids
        ps.walk_state.dispositioned_ids = lambda: orig("/nonexistent-dispositions-ledger.json")
        try:
            pool, stats = ps.build_pool(
                _status([{"id": "H-10", "desc": "mount"}]), self.STATE, set(),
                "/nonexistent-plan-status", summary_dir=_NO_SUMMARY, synthesis_path=_NO_SYNTH,
                seed_attempts={})  # dispositioned=None -> exercises the default read
            self.assertEqual([g["id"] for g in pool], ["H-10"])
            self.assertEqual(stats["dispositioned"], 0)
        finally:
            ps.walk_state.dispositioned_ids = orig

    def test_select_next_excludes_dispositioned_from_eligible(self):
        # end-to-end through the production seam: a drained id must be absent from BOTH the pool and the
        # eligible ids the quorum votes over. C-15 (undispositioned here) wins; H-10 (drained) is gone.
        punch = [{"id": "H-10", "desc": "mount"}, {"id": "C-15", "desc": "regressed x"}]
        pick = ps.select_next(
            self.STATE, "/nonexistent", status_reader=lambda: _status(punch),
            dirty_reader=lambda: set(), post_fn=_post_all("C-15"),
            summary_dir=_NO_SUMMARY, synthesis_path=_NO_SYNTH, disp_reader=lambda: {"H-10"})
        self.assertIsNotNone(pick)
        self.assertEqual(pick["plan_id"], "C-15")
        self.assertNotIn("H-10", pick["detail"]["eligible"])
        self.assertEqual(pick["detail"]["pool"]["dispositioned"], 1)


class TestU4AuditedAdmission(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="u4_selector_tests_"))
        self.status_dir = self.root / "plan-status"
        self.status_dir.mkdir()
        self.index = self.root / "audit-index.json"
        self._old_index = ps.AUDIT_INDEX_PATH
        ps.AUDIT_INDEX_PATH = self.index

    def tearDown(self):
        ps.AUDIT_INDEX_PATH = self._old_index
        import shutil
        shutil.rmtree(self.root, ignore_errors=True)

    def _source(self, plan_id, text=None):
        path = self.root / f"{plan_id}-source.txt"
        path.write_text(text or f"{plan_id} cited source\n", encoding="utf-8")
        return path

    def _record(self, plan_id, *, evidence_id=None, verdict="EXECUTE-AS-IS",
                citation=None, rationale="fixture rationale", recon_ts="2026-08-13T00:00:00Z"):
        source = self._source(plan_id)
        if citation is None:
            citation = {"file": str(source), "line": 1,
                        "text": source.read_text(encoding="utf-8").splitlines()[0],
                        "why": "fixture citation"}
        record = {
            "id": evidence_id if evidence_id is not None else plan_id,
            "verdict": verdict,
            "citations": [citation],
            "rationale": rationale,
            "recon_ts": recon_ts,
        }
        path = self.root / f"{plan_id}.json"
        path.write_text(json.dumps(record), encoding="utf-8")
        return path

    def _row(self, plan_id, verdict="EXECUTE-AS-IS", **kwargs):
        return {"id": plan_id, "verdict": verdict,
                "evidence_path": str(self._record(plan_id, verdict=verdict, **kwargs))}

    def _load(self, rows):
        self.index.write_text(json.dumps({"rows": rows}), encoding="utf-8")
        return ps.load_audited_index(self.index, self.status_dir)

    def test_strict_admission_pins_sources_and_pickup_contract(self):
        entries = self._load([self._row("A-VALID")])
        self.assertEqual([entry["id"] for entry in entries], ["A-VALID"])
        entry = entries[0]
        self.assertEqual(entry["tier"], "audited")
        self.assertEqual(entry["verdict"], "EXECUTE-AS-IS")
        self.assertEqual(len(entry["audit_citations"]), 1)
        pin = entry["audit_citations"][0]
        self.assertEqual(pin["line"], 1)
        self.assertEqual(pin["text"], "A-VALID cited source")
        self.assertEqual(pin["sha256"], hashlib.sha256(Path(pin["file"]).read_bytes()).hexdigest())
        pickup = ps._build_pickup(entry, ["A-VALID"])
        self.assertIsNotNone(pickup)
        for token in ("audit_evidence", "production_caller", "adversarial_tests",
                      "wire_check", "plan_spec", "land_evidence", "fresh_recon"):
            self.assertIn(token, pickup)
        self.assertIn(entry["audit_evidence_path"], pickup)
        self.assertIn(entry["audit_evidence_sha256"], pickup)

    def test_non_execute_verdicts_are_excluded_and_counted(self):
        rows = [self._row("A-AMEND", "AMEND"),
                self._row("A-OWNER", "OWNER-GATED"),
                self._row("A-DONE", "ALREADY-DONE"),
                self._row("A-SUPERSEDED", "SUPERSEDED-BY A-NEW")]
        self.assertEqual(self._load(rows), [])
        exclusions = ps.load_audited_index.last_report["exclusions"]
        self.assertEqual(exclusions["AMEND"], 1)
        self.assertEqual(exclusions["OWNER-GATED"], 1)
        self.assertEqual(exclusions["ALREADY-DONE"], 1)
        self.assertEqual(exclusions["SUPERSEDED-BY"], 1)

    def test_schema_failures_are_excluded_and_counted(self):
        dead_source = self.root / "missing-source.txt"
        dead = self.root / "A-DEAD.json"
        dead.write_text(json.dumps({
            "id": "A-DEAD", "verdict": "EXECUTE-AS-IS",
            "citations": [{"file": str(dead_source), "line": 1, "text": "x",
                           "why": "dead source"}],
            "rationale": "fixture", "recon_ts": "2026-08-13T00:00:00Z",
        }), encoding="utf-8")
        nonpositive_source = self._source("A-NONPOSITIVE")
        nonpositive = self.root / "A-NONPOSITIVE.json"
        nonpositive.write_text(json.dumps({
            "id": "A-NONPOSITIVE", "verdict": "EXECUTE-AS-IS",
            "citations": [{"file": str(nonpositive_source), "line": 0, "text": "x",
                           "why": "bad line"}],
            "rationale": "fixture", "recon_ts": "2026-08-13T00:00:00Z",
        }), encoding="utf-8")
        missing_why = self._record("A-MISSING-WHY")
        missing_why.write_text(json.dumps({
            "id": "A-MISSING-WHY", "verdict": "EXECUTE-AS-IS",
            "citations": [{"file": str(self._source("A-MISSING-WHY-SOURCE")),
                           "line": 1, "text": "A-MISSING-WHY-SOURCE cited source"}],
            "rationale": "fixture", "recon_ts": "2026-08-13T00:00:00Z",
        }), encoding="utf-8")
        rows = [
            {"id": "A-MISSING", "verdict": "EXECUTE-AS-IS",
             "evidence_path": str(self.root / "does-not-exist.json")},
            {"id": "A-ID", "verdict": "EXECUTE-AS-IS",
             "evidence_path": str(self._record("A-ID", evidence_id="OTHER"))},
            {"id": "A-TEXT", "verdict": "EXECUTE-AS-IS",
             "evidence_path": str(self._record("A-TEXT", citation={
                 "file": str(self._source("A-TEXT")), "line": 1,
                 "text": "not the cited line", "why": "text mismatch"}))},
            {"id": "A-DEAD", "verdict": "EXECUTE-AS-IS", "evidence_path": str(dead)},
            {"id": "A-NONPOSITIVE", "verdict": "EXECUTE-AS-IS",
             "evidence_path": str(nonpositive)},
            {"id": "A-BAD-TS", "verdict": "EXECUTE-AS-IS",
             "evidence_path": str(self._record("A-BAD-TS", recon_ts="not-a-timestamp"))},
            {"id": "A-MISSING-WHY", "verdict": "EXECUTE-AS-IS", "evidence_path": str(missing_why)},
        ]
        self.assertEqual(self._load(rows), [])
        report = ps.load_audited_index.last_report
        self.assertEqual(report["exclusions"]["MISSING-EVIDENCE"], 1)
        self.assertEqual(report["exclusions"]["SCHEMA"], 6)
        reasons = {entry["reason"] for entry in report["excluded"] if entry["class"] == "SCHEMA"}
        self.assertIn("evidence_id_mismatch", reasons)
        self.assertIn("citation_text_not_byte_exact", reasons)
        self.assertIn("citation_source_invalid", reasons)
        self.assertIn("recon_ts_unparseable", reasons)
        self.assertIn("citation_shape_invalid", reasons)

    def test_pickup_rejects_audit_and_citation_source_drift(self):
        evidence_one = self._record("A-EVIDENCE-DRIFT")
        entries = self._load([{"id": "A-EVIDENCE-DRIFT", "verdict": "EXECUTE-AS-IS",
                               "evidence_path": str(evidence_one)},
                              self._row("A-SOURCE-DRIFT")])
        by_id = {entry["id"]: entry for entry in entries}
        evidence_one.write_text(evidence_one.read_text(encoding="utf-8") + "\n", encoding="utf-8")
        self.assertIsNone(ps._build_pickup(by_id["A-EVIDENCE-DRIFT"], ["A-EVIDENCE-DRIFT"]))
        source = Path(by_id["A-SOURCE-DRIFT"]["audit_citations"][0]["file"])
        source.write_text("A-SOURCE-DRIFT mutated source\n", encoding="utf-8")
        self.assertIsNone(ps._build_pickup(by_id["A-SOURCE-DRIFT"], ["A-SOURCE-DRIFT"]))
        exclusions = ps.load_audited_index.last_report["exclusions"]
        self.assertEqual(exclusions["audit_evidence_drift"], 1)
        self.assertEqual(exclusions["citation_source_drift"], 1)

    def test_all_settled_rejects_non_frozen_verdict_and_bad_plan_binding(self):
        proof = self.root / "proof.txt"
        proof.write_text("proof\n", encoding="utf-8")
        row = self._row("A-SETTLED")
        ledger = self.status_dir.parent / "plan-dispositions.json"
        ledger.write_text(json.dumps({"dispositions": {
            "A-SETTLED": {"plan_id": "OTHER", "verdict": "resolved",
                           "evidence_ref": str(proof)},
        }}), encoding="utf-8")
        entries = self._load([row])
        self.assertEqual(len(entries), 1)
        unsettled = ps.load_audited_index.last_report["all_settled"]["unsettled"]
        self.assertEqual(unsettled, [{"id": "A-SETTLED", "reason": "disposition_plan_id_mismatch"}])

    def test_all_settled_accepts_done_commit_proof(self):
        proof = self.root / "done-proof.txt"
        proof.write_text("proof\n", encoding="utf-8")
        status = self.status_dir / "A-DONE.json"
        status.write_text(json.dumps({"plan_id": "A-DONE", "status": "DONE",
                                      "commit_sha": "abc123"}), encoding="utf-8")
        ledger = self.status_dir.parent / "plan-dispositions.json"
        ledger.write_text(json.dumps({"dispositions": {
            "A-DONE": {"plan_id": "A-DONE", "verdict": "already_done",
                        "evidence_ref": str(proof)},
        }}), encoding="utf-8")
        self.assertEqual(self._load([self._row("A-DONE")]), [])
        report = ps.load_audited_index.last_report
        self.assertTrue(report["all_settled"]["complete"])
        self.assertEqual(report["all_settled"]["unsettled"], [])

    def test_all_settled_accepts_superseded_target_proof(self):
        proof = self.root / "superseded-proof.txt"
        proof.write_text("proof\n", encoding="utf-8")
        ledger = self.status_dir.parent / "plan-dispositions.json"
        ledger.write_text(json.dumps({"dispositions": {
            "A-OLD": {"plan_id": "A-OLD", "verdict": "superseded",
                       "evidence_ref": str(proof), "superseding_id": "A-NEW"},
        }}), encoding="utf-8")
        self.assertEqual(self._load([self._row("A-OLD")]), [])
        self.assertTrue(ps.load_audited_index.last_report["all_settled"]["complete"])

    def test_tier_order_is_punch_then_audited_then_corpus(self):
        self._load([self._row("A-AUDITED")])
        pool, stats = ps.build_pool(
            _status([{"id": "P-1", "desc": "punch"}]),
            {"queue": [], "escalation_queue": []}, set(), str(self.status_dir),
            summary_dir=_NO_SUMMARY, synthesis_path=_NO_SYNTH, dispositioned=set(),
            corpus_ordered=["A-CORPUS"],
            corpus_index={"units": [{"id": "A-CORPUS", "does": "corpus", "key_files": [],
                                      "state": "certain"}]},
        )
        self.assertEqual([gap["id"] for gap in pool[:3]], ["P-1", "A-AUDITED", "A-CORPUS"])
        self.assertEqual(stats["punch"], 1)
        self.assertEqual(stats["audited_open"], 1)
        self.assertEqual(stats["corpus"], 1)

    def test_real_try_autoseed_forwards_pickup_byte_identical(self):
        import importlib
        supervisor = importlib.import_module("glm_walk.glm_supervisor")
        expected = {"plan_id": "A-VALID", "pickup": "fresh_recon=required", "detail": {}}
        captured = {}
        original = {
            "enabled": supervisor._autoseed_enabled,
            "drain": supervisor._drain_superseded,
            "load": supervisor.walk_state.load,
            "select": supervisor.plan_selector.select_next,
            "seed": supervisor.walk_state.seed_plan,
            "record": supervisor.plan_selector.record_seed_attempt,
            "emit": supervisor._emit,
        }
        try:
            supervisor._autoseed_enabled = lambda: True
            supervisor._drain_superseded = lambda turn: 0
            supervisor.walk_state.load = lambda path: {"queue": [], "escalation_queue": []}
            supervisor.plan_selector.select_next = lambda *args, **kwargs: expected
            supervisor.walk_state.seed_plan = lambda plan_id, pickup, path: captured.update(
                {"plan_id": plan_id, "pickup": pickup, "path": path}) or {}
            supervisor.plan_selector.record_seed_attempt = lambda plan_id: 1
            supervisor._emit = lambda *args, **kwargs: None
            self.assertTrue(supervisor._try_autoseed(0))
        finally:
            supervisor._autoseed_enabled = original["enabled"]
            supervisor._drain_superseded = original["drain"]
            supervisor.walk_state.load = original["load"]
            supervisor.plan_selector.select_next = original["select"]
            supervisor.walk_state.seed_plan = original["seed"]
            supervisor.plan_selector.record_seed_attempt = original["record"]
            supervisor._emit = original["emit"]
        self.assertEqual(captured["pickup"], expected["pickup"])
        self.assertEqual(captured["plan_id"], expected["plan_id"])
        self.assertIn("fresh_recon", captured["pickup"])


if __name__ == "__main__":
    unittest.main()
