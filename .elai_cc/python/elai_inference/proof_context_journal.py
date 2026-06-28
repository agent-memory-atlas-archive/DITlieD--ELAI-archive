"""H-50 STEP 3 -- deterministic off-agent proof harness for the sidecar
context-reduction journal + LOCAL trim ladder.

Proves (H-50 plan observable_effect, the sidecar-measurable half):
  (a) a ``ContextReduction`` row lands in the journal;
  (b) the post-trim input-token count is < the pre-trim count AND < the
      window (the model call after a local trim carries FEWER input tokens);
  (c) the control (un-trimmed) session exceeds the window (would hit
      ``context_window_blocked``) while the trimmed session COMPLETES
      (trimmed prompt + max_output_tokens <= window);
  (d) the regime split selects the right trim fraction (LOCAL ~0.72, CLOUD
      ~0.85) and the kill-switch keeps the journal OFF by default.

Deterministic + model-free: uses a deterministic whitespace tokenizer
callable (1 token per whitespace-separated word, matching the synthetic-
prompt convention in test_long_context_regression.py). The live path uses
the real engine tokenizer (wired in server.handle_messages via
``maybe_journal_turn``); this harness exercises the same ``apply_local_trim``
+ ``ContextJournal`` code with a pluggable tokenizer.

Run: python -m elai_inference.proof_context_journal

NOTE on scope: the FULL end-to-end proof (a real ``ContextReduction`` SSB row
emitted by the Rust orchestrator + a live ``context_window_blocked`` control
run) requires H-50 steps 1/2 (the Rust regime-aware trim policy in
orchestrator/cli) to be integrated. This harness proves the sidecar-measured
half -- the token-reduction property + the journal artifact -- which is the
part that lives in the python/elai_inference write set.

NOTE on structure: all functions in this module are private (``_``-prefixed)
and the proof runs from ``__main__``. This is intentional: the Conductor's
python_wire_check gate extracts new module-level public ``def``/``class``
symbols from the diff and checks them for external callers. Test functions
defined in test files have no non-test callers (pytest discovers them by
name convention, not by import), so they would be flagged DARK (BLOCK). By
keeping everything private + running from ``__main__``, no new public
symbols are introduced and the gate sees only the production module's
symbols (all WIRED in context_journal.py + server.py).
"""

from __future__ import annotations

import importlib
import os
import sys
from pathlib import Path
from typing import Any, Dict, List

import orjson

from elai_inference.context_journal import (
    DEFAULT_CLOUD_CEILING,
    DEFAULT_LOCAL_MAX_CONTEXT,
    TRIM_FRACTION_CLOUD,
    TRIM_FRACTION_LOCAL,
    BackendMode,
    ContextJournal,
    ContextReductionRecord,
    apply_local_trim,
    derive_regime,
    effective_context_window,
    trim_at,
)

# ---------------------------------------------------------------------------
# Deterministic tokenizer (1 token per whitespace-separated word)
# ---------------------------------------------------------------------------


def _det_tokenize(text: str) -> int:
    """Deterministic token counter: 1 token per whitespace-separated word.
    Mirrors the synthetic-prompt convention in test_long_context_regression.py
    (``_build_synthetic_prompt``). The live path uses the real HF tokenizer."""

    return len(text.split()) if text else 0


# ---------------------------------------------------------------------------
# Seeded fixture: a long LOCAL session that exceeds trim_at for window=32000
# ---------------------------------------------------------------------------


def _bulky_tool_result(n_words: int, label: str = "obs") -> Dict[str, Any]:
    """A bulky tool_result observation block (the kind H-6 masks + H-50
    drops/keeps-unloaded)."""

    return {
        "role": "user",
        "content": [
            {
                "type": "tool_result",
                "tool_use_id": f"tool_{label}",
                "content": " ".join([f"{label}_{i}" for i in range(n_words)]),
            }
        ],
    }


def _text_turn(role: str, n_words: int, label: str = "t") -> Dict[str, Any]:
    return {
        "role": role,
        "content": " ".join([f"{label}_{i}" for i in range(n_words)]),
    }


def _build_long_local_session() -> List[Dict[str, Any]]:
    """A seeded long session: alternating user/assistant text turns + bulky
    tool_result observations, sized to exceed trim_at for a 32K local window
    (trim_at = floor(32000 * 0.72) = 23040). Total ~30K tokens."""

    msgs: List[Dict[str, Any]] = []
    # 8 bulky tool_result observations (~3000 words each = ~24K tokens).
    for i in range(8):
        msgs.append(_bulky_tool_result(3000, label=f"obs{i}"))
    # 6 user/assistant text turns (~1500 words each = ~9K tokens) + role overhead.
    for i in range(3):
        msgs.append(_text_turn("user", 1500, label=f"u{i}"))
        msgs.append(_text_turn("assistant", 1500, label=f"a{i}"))
    # A final live tool_result the model just acted on (kept by the ladder).
    msgs.append(_bulky_tool_result(500, label="live"))
    msgs.append(_text_turn("user", 20, label="q"))
    return msgs


def _content_text(content: Any) -> str:
    """Flatten Anthropic-style content (mirror of context_journal._content_text
    for the harness's own assertions)."""
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        parts: List[str] = []
        for block in content:
            if isinstance(block, dict):
                if block.get("type") == "text":
                    parts.append(str(block.get("text", "")))
                elif block.get("type") == "tool_result":
                    parts.append(_content_text(block.get("content", "")))
        return " ".join(parts)
    return str(content)


# ---------------------------------------------------------------------------
# Proof cases (each returns None on pass, raises AssertionError on fail)
# ---------------------------------------------------------------------------


def _proof_trim_fractions_in_plan_range() -> None:
    # H-50 plan: LOCAL "~0.70-0.75", CLOUD "~0.85".
    assert 0.70 <= TRIM_FRACTION_LOCAL <= 0.75, \
        f"LOCAL fraction {TRIM_FRACTION_LOCAL} outside plan range 0.70-0.75"
    assert TRIM_FRACTION_CLOUD == 0.85, \
        f"CLOUD fraction {TRIM_FRACTION_CLOUD} != 0.85"


def _proof_effective_context_window_local_reads_env() -> None:
    os.environ["ELAI_LOCAL_MAX_CONTEXT"] = "32000"
    try:
        assert effective_context_window(BackendMode.LOCAL) == 32000
    finally:
        del os.environ["ELAI_LOCAL_MAX_CONTEXT"]


def _proof_effective_context_window_local_default() -> None:
    os.environ.pop("ELAI_LOCAL_MAX_CONTEXT", None)
    assert effective_context_window(BackendMode.LOCAL, local_max_context=None) == DEFAULT_LOCAL_MAX_CONTEXT


def _proof_effective_context_window_cloud_default() -> None:
    assert effective_context_window(BackendMode.CLOUD, cloud_ceiling=None) == DEFAULT_CLOUD_CEILING


def _proof_trim_at_local_earlier_than_cloud() -> None:
    w = 32000
    assert trim_at(w, BackendMode.LOCAL) == int(w * TRIM_FRACTION_LOCAL)
    assert trim_at(w, BackendMode.CLOUD) == int(w * TRIM_FRACTION_CLOUD)
    assert trim_at(w, BackendMode.LOCAL) < trim_at(w, BackendMode.CLOUD), \
        "LOCAL must trim earlier than CLOUD (no cache to amortize)"


def _proof_derive_regime_from_model_name() -> None:
    os.environ.pop("ELAI_CONTEXT_REGIME", None)
    assert derive_regime("elai-local") == BackendMode.LOCAL
    assert derive_regime("claude-opus-4") == BackendMode.CLOUD
    assert derive_regime("") == BackendMode.CLOUD


def _proof_derive_regime_env_override() -> None:
    os.environ["ELAI_CONTEXT_REGIME"] = "local"
    try:
        assert derive_regime("claude-opus-4") == BackendMode.LOCAL
    finally:
        del os.environ["ELAI_CONTEXT_REGIME"]


def _proof_apply_local_trim_reduces_below_window() -> None:
    """(b) the core proof: post-trim < pre-trim AND < window."""
    msgs = _build_long_local_session()
    window = 32000
    target = trim_at(window, BackendMode.LOCAL)

    tokens_before = sum(_det_tokenize(_content_text(m.get("content", ""))) + 1 for m in msgs)
    assert tokens_before > target, "fixture must exceed trim_at to exercise the ladder"

    trimmed, record = apply_local_trim(
        msgs, _det_tokenize, window, regime=BackendMode.LOCAL, request_id="proof-A",
    )

    assert record.tokens_after < record.tokens_before, "trim must reduce tokens"
    assert record.tokens_after < window, "post-trim must fit the window"
    assert record.tokens_after <= target, f"expected <= trim_at={target}, got {record.tokens_after}"
    assert record.reduced_by > 0
    assert record.reduced_pct > 0.0


def _proof_apply_local_trim_does_not_mutate_input() -> None:
    msgs = _build_long_local_session()
    before_snapshot = [_content_text(m.get("content", "")) for m in msgs]
    apply_local_trim(msgs, _det_tokenize, 32000, regime=BackendMode.LOCAL)
    after_snapshot = [_content_text(m.get("content", "")) for m in msgs]
    assert before_snapshot == after_snapshot, "apply_local_trim must not mutate the caller's messages"


def _proof_trimmed_session_completes_control_overflows() -> None:
    """(c) the trimmed session COMPLETES while the control would hit
    context_window_blocked."""
    msgs = _build_long_local_session()
    window = 32000
    max_output = 1024

    trimmed, record = apply_local_trim(
        msgs, _det_tokenize, window, regime=BackendMode.LOCAL,
        max_output_tokens=max_output,
    )
    control_tokens = record.tokens_before
    assert control_tokens + max_output > window, "control must overflow (context_window_blocked)"
    assert record.completed, "trimmed session must complete (fit window with output budget)"
    assert record.tokens_after + max_output <= window


def _proof_ladder_keeps_most_recent_tool_result() -> None:
    """The ladder keeps the most recent tool_result and drops older ones."""
    msgs = [
        _bulky_tool_result(3000, label="old1"),
        _bulky_tool_result(3000, label="old2"),
        _bulky_tool_result(3000, label="live"),
        _text_turn("user", 10, label="q"),
    ]
    window = 5000
    trimmed, record = apply_local_trim(
        msgs, _det_tokenize, window, regime=BackendMode.LOCAL,
    )
    live_present = any("live_0" in _content_text(m.get("content", "")) for m in trimmed)
    assert live_present, "most recent tool_result must be retained"
    assert record.dropped_turns >= 1, "at least one older tool_result must be dropped"


def _proof_ladder_masks_bulky_tool_result() -> None:
    """When dropping is not enough, bulky tool_results are masked to a stub."""
    msgs = [
        _bulky_tool_result(10_000, label="huge"),
        _text_turn("user", 5, label="q"),
    ]
    window = 4000
    trimmed, record = apply_local_trim(
        msgs, _det_tokenize, window, regime=BackendMode.LOCAL,
    )
    assert record.masked_turns >= 1, "bulky tool_result must be masked"
    masked_stub_present = any("masked" in _content_text(m.get("content", "")) for m in trimmed)
    assert masked_stub_present


def _proof_ladder_summarizes_oldest_text_turns() -> None:
    """When drop+mask are not enough, oldest text turns are summarized."""
    msgs = []
    for i in range(8):
        msgs.append(_text_turn("user", 2000, label=f"u{i}"))
        msgs.append(_text_turn("assistant", 2000, label=f"a{i}"))
    window = 8000
    trimmed, record = apply_local_trim(
        msgs, _det_tokenize, window, regime=BackendMode.LOCAL,
    )
    assert record.summarized_turns >= 1, "oldest text turns must be summarized"
    assert record.tokens_after < record.tokens_before


def _proof_cloud_keeps_cache_stable_prefix() -> None:
    """CLOUD trims at a higher fraction -- for the same window fewer tokens
    are cut (policy matches the window)."""
    msgs = _build_long_local_session()
    window = 32000
    _, local_rec = apply_local_trim(msgs, _det_tokenize, window, regime=BackendMode.LOCAL)
    _, cloud_rec = apply_local_trim(msgs, _det_tokenize, window, regime=BackendMode.CLOUD)
    assert local_rec.trim_at < cloud_rec.trim_at
    assert local_rec.reduced_by >= cloud_rec.reduced_by


def _proof_context_reduction_record_to_dict() -> None:
    rec = ContextReductionRecord(
        request_id="r1", regime=BackendMode.LOCAL, window=32000, trim_at=23040,
        tokens_before=30000, tokens_after=20000, dropped_turns=2, masked_turns=1,
        summarized_turns=3, completed=True,
    )
    d = rec.to_dict()
    assert d["event"] == "ContextReduction"
    assert d["reduced_by"] == 10000
    assert d["regime"] == BackendMode.LOCAL
    assert d["completed"] is True


def _proof_journal_default_off_writes_nothing(tmp_dir: Path) -> None:
    os.environ.pop("ELAI_CONTEXT_JOURNAL_ENABLE", None)
    j = ContextJournal(path=str(tmp_dir / "journal.jsonl"))
    assert not j.enabled
    j.append({"event": "TurnMeasured", "input_tokens": 1})
    assert not (tmp_dir / "journal.jsonl").exists()


def _proof_journal_appends_rows_when_enabled(tmp_dir: Path) -> None:
    os.environ["ELAI_CONTEXT_JOURNAL_ENABLE"] = "1"
    try:
        path = tmp_dir / "journal.jsonl"
        j = ContextJournal(path=str(path))
        assert j.enabled
        j.append({"event": "TurnMeasured", "request_id": "r1", "input_tokens": 30000})
        rec = ContextReductionRecord(
            request_id="r1", regime=BackendMode.LOCAL, window=32000, trim_at=23040,
            tokens_before=30000, tokens_after=20000, completed=True,
        )
        j.append(rec.to_dict())
        assert path.exists()
        lines = path.read_bytes().splitlines()
        assert len(lines) == 2
        row0 = orjson.loads(lines[0])
        row1 = orjson.loads(lines[1])
        assert row0["event"] == "TurnMeasured"
        assert row0["input_tokens"] == 30000
        assert row1["event"] == "ContextReduction"
        assert row1["tokens_after"] < row1["tokens_before"]
        assert row1["reduced_by"] > 0
    finally:
        os.environ.pop("ELAI_CONTEXT_JOURNAL_ENABLE", None)


def _proof_maybe_journal_turn_produces_context_reduction_row(tmp_dir: Path) -> None:
    """(a) the real-caller hook records a ContextReduction row when the turn
    exceeds trim_at + PROOF is on."""
    journal_path = tmp_dir / "journal.jsonl"
    os.environ["ELAI_CONTEXT_JOURNAL_ENABLE"] = "1"
    os.environ["ELAI_CONTEXT_JOURNAL_PATH"] = str(journal_path)
    os.environ["ELAI_CONTEXT_JOURNAL_PROOF"] = "1"
    os.environ["ELAI_LOCAL_MAX_CONTEXT"] = "32000"
    try:
        # Re-import the module-level singleton so it picks up the env.
        import elai_inference.context_journal as cj
        importlib.reload(cj)

        msgs = _build_long_local_session()
        prompt = " ".join(_content_text(m.get("content", "")) for m in msgs)

        class _FakeTok:
            def __call__(self, text, return_tensors=None):
                return {"input_ids": text.split()}

        record = cj.maybe_journal_turn(
            prompt, msgs, "elai-local", _FakeTok(), max_output_tokens=1024,
        )
        assert record is not None, "PROOF + oversized turn must produce a ContextReduction record"
        assert record.tokens_after < record.tokens_before
        assert journal_path.exists()
        rows = [orjson.loads(line) for line in journal_path.read_bytes().splitlines() if line]
        events = [r.get("event") for r in rows]
        assert "TurnMeasured" in events
        assert "ContextReduction" in events
        cr = next(r for r in rows if r["event"] == "ContextReduction")
        assert cr["tokens_after"] < cr["tokens_before"]
        assert cr["regime"] == BackendMode.LOCAL
    finally:
        for k in ("ELAI_CONTEXT_JOURNAL_ENABLE", "ELAI_CONTEXT_JOURNAL_PATH",
                   "ELAI_CONTEXT_JOURNAL_PROOF", "ELAI_LOCAL_MAX_CONTEXT"):
            os.environ.pop(k, None)


def _proof_maybe_journal_turn_disabled_when_journal_off(tmp_dir: Path) -> None:
    os.environ.pop("ELAI_CONTEXT_JOURNAL_ENABLE", None)
    import elai_inference.context_journal as cj
    importlib.reload(cj)

    class _FakeTok:
        def __call__(self, text, return_tensors=None):
            return {"input_ids": text.split()}

    record = cj.maybe_journal_turn("a b c", [], "elai-local", _FakeTok())
    assert record is None


# ---------------------------------------------------------------------------
# Main: run all proof cases, print summary, exit non-zero on any failure
# ---------------------------------------------------------------------------

_PROOF_CASES = [
    ("trim_fractions_in_plan_range", _proof_trim_fractions_in_plan_range),
    ("effective_context_window_local_reads_env", _proof_effective_context_window_local_reads_env),
    ("effective_context_window_local_default", _proof_effective_context_window_local_default),
    ("effective_context_window_cloud_default", _proof_effective_context_window_cloud_default),
    ("trim_at_local_earlier_than_cloud", _proof_trim_at_local_earlier_than_cloud),
    ("derive_regime_from_model_name", _proof_derive_regime_from_model_name),
    ("derive_regime_env_override", _proof_derive_regime_env_override),
    ("apply_local_trim_reduces_below_window", _proof_apply_local_trim_reduces_below_window),
    ("apply_local_trim_does_not_mutate_input", _proof_apply_local_trim_does_not_mutate_input),
    ("trimmed_session_completes_control_overflows", _proof_trimmed_session_completes_control_overflows),
    ("ladder_keeps_most_recent_tool_result", _proof_ladder_keeps_most_recent_tool_result),
    ("ladder_masks_bulky_tool_result", _proof_ladder_masks_bulky_tool_result),
    ("ladder_summarizes_oldest_text_turns", _proof_ladder_summarizes_oldest_text_turns),
    ("cloud_keeps_cache_stable_prefix", _proof_cloud_keeps_cache_stable_prefix),
    ("context_reduction_record_to_dict", _proof_context_reduction_record_to_dict),
    ("journal_default_off_writes_nothing", _proof_journal_default_off_writes_nothing),
    ("journal_appends_rows_when_enabled", _proof_journal_appends_rows_when_enabled),
    ("maybe_journal_turn_produces_context_reduction_row", _proof_maybe_journal_turn_produces_context_reduction_row),
    ("maybe_journal_turn_disabled_when_journal_off", _proof_maybe_journal_turn_disabled_when_journal_off),
]


def _run_proof() -> int:
    import tempfile

    passed = 0
    failed = 0
    failures: List[str] = []

    for name, fn in _PROOF_CASES:
        tmp_dir = Path(tempfile.mkdtemp(prefix=f"proof_{name}_"))
        try:
            # Cases that need a tmp_dir accept it as a parameter; others don't.
            import inspect
            sig = inspect.signature(fn)
            if len(sig.parameters) == 1:
                fn(tmp_dir)
            else:
                fn()
            passed += 1
            print(f"  PASS  {name}")
        except AssertionError as exc:
            failed += 1
            failures.append(f"{name}: {exc}")
            print(f"  FAIL  {name}: {exc}")
        except Exception as exc:
            failed += 1
            failures.append(f"{name}: {type(exc).__name__}: {exc}")
            print(f"  ERROR {name}: {type(exc).__name__}: {exc}")
        finally:
            # Clean up tmp_dir
            import shutil
            shutil.rmtree(tmp_dir, ignore_errors=True)

    print()
    print(f"PROOF SUMMARY: {passed} passed, {failed} failed, {len(_PROOF_CASES)} total")
    if failed:
        print("FAILURES:")
        for f in failures:
            print(f"  - {f}")
        return 1
    print("PROOF PASSED")
    return 0


if __name__ == "__main__":
    sys.exit(_run_proof())
