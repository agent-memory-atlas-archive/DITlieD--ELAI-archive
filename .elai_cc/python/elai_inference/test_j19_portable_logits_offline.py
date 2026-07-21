"""J-19 Slice-3 S5: ETCLogitsProcessor + SeededGumbelProcessor portable offline contract.

Verifies the portable logits-processor subset of J-19 on CPU without loading
a model and without GPU. Exercises ETCLogitsProcessor (entropy-trend constraint
observer) and documents that both processors co-exist in the same module
(the J-19 portable subset). Does NOT re-assert SeededGumbel same-seed/diff-seed/
step-advance (those are in test_seeded_gumbel_processor.py).
"""

from __future__ import annotations

import os
import re
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

try:
    import torch
    from elai_inference.server import (
        ETCLogitsProcessor,
        SeededGumbelProcessor,
    )
    _SERVER_IMPORTABLE = True
except ImportError:
    _SERVER_IMPORTABLE = False


pytestmark = pytest.mark.skipif(
    not _SERVER_IMPORTABLE,
    reason="server.py not importable (torch/transformers missing on host)",
)


_FIXTURE_LOGITS = torch.tensor([[1.0, 2.0, 0.5, 1.9, 3.0, 1.5]])
_FIXTURE_INPUT_IDS = torch.zeros((1, 5), dtype=torch.long)


class _StubTokenizer:
    """Minimal tokenizer returning a controlled decode string."""

    def __init__(self, text: str = "hello world"):
        self._text = text

    def decode(self, ids, skip_special_tokens=True):
        return self._text


def test_etc_importable_from_server():
    """ETCLogitsProcessor is importable from elai_inference.server."""
    assert ETCLogitsProcessor is not None


def test_etc_never_mutates_scores():
    """ETCLogitsProcessor.__call__ returns the scores tensor unchanged."""
    tokenizer = _StubTokenizer("hello world")
    proc = ETCLogitsProcessor(
        threshold=0.0,
        import_pattern=re.compile(r"import\s+\w+"),
        tokenizer=tokenizer,
    )
    original = _FIXTURE_LOGITS.clone()
    returned = proc(_FIXTURE_INPUT_IDS, original.clone())
    assert torch.equal(returned, original), (
        "ETCLogitsProcessor mutated scores; it must be an observer only"
    )


def test_etc_threshold_inf_disables_trigger():
    """threshold=inf never triggers across several steps."""
    tokenizer = _StubTokenizer("import foo")
    proc = ETCLogitsProcessor(
        threshold=float("inf"),
        import_pattern=re.compile(r"import\s+\w+"),
        tokenizer=tokenizer,
    )
    logits = _FIXTURE_LOGITS.clone()
    for _ in range(5):
        proc(_FIXTURE_INPUT_IDS, logits.clone())
    assert proc.triggered is False, (
        "threshold=inf should disable triggering but triggered is True"
    )
    assert proc.trigger_token_idx == -1


def test_etc_threshold_zero_calibration_log():
    """threshold=0.0 populates d2h_log and leaves scores unmodified."""
    tokenizer = _StubTokenizer("hello world")
    proc = ETCLogitsProcessor(
        threshold=0.0,
        import_pattern=re.compile(r"import\s+\w+"),
        tokenizer=tokenizer,
    )
    original = _FIXTURE_LOGITS.clone()
    returned = proc(_FIXTURE_INPUT_IDS, original.clone())
    assert len(proc.d2h_log) > 0, (
        "threshold=0.0 should populate d2h_log for calibration"
    )
    assert torch.equal(returned, original), (
        "ETCLogitsProcessor mutated scores during calibration logging"
    )


def test_etc_trigger_path():
    """Real __call__ with import context and low threshold sets triggered=True.

    Uses threshold=-1e9 so the first step's d2h (entropy, positive) exceeds it,
    and the stub tokenizer decode returns 'import foo' which matches the
    import pattern. No monkeypatch of triggered -- the real __call__ path fires.
    """
    tokenizer = _StubTokenizer("import foo")
    proc = ETCLogitsProcessor(
        threshold=-1e9,
        import_pattern=re.compile(r"import\s+\w+"),
        tokenizer=tokenizer,
    )
    logits = _FIXTURE_LOGITS.clone()
    proc(_FIXTURE_INPUT_IDS, logits.clone())
    assert proc.triggered is True, (
        "trigger path did not fire with low threshold and import context"
    )
    assert proc.trigger_token_idx >= 0, (
        "trigger_token_idx should be non-negative after trigger fires"
    )


def test_j19_portable_subset_both_processors_importable():
    """Both ETCLogitsProcessor and SeededGumbelProcessor are importable from
    the same module -- documents the J-19 portable logits-processor subset."""
    assert ETCLogitsProcessor is not None
    assert SeededGumbelProcessor is not None
    assert ETCLogitsProcessor is not SeededGumbelProcessor
