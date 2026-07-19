"""J-13 contract: SeededGumbelProcessor deterministic Gumbel-Max sampling.

Verifies the per-(seed, step) Generator contract:
1. SAME-SEED: identical returned scores / argmax for seed=42 twice on a
   fixed fixture logits tensor. Locks: same (logits, seed) -> same token.
2. DIFF-SEED: seed=42 vs seed=43 produce unequal returned tensors on the
   same fixture (tensor-inequality, not argmax-inequality, so the contract
   holds even when noise does not flip the argmax).
3. STEP-ADVANCE: two consecutive __call__ invocations on the same processor
   produce different perturbations (the per-step generator advances).

These tests run WITHOUT loading a model and WITHOUT GPU. They exercise the
host-side processor shape only. Live model inference with end-to-end
seeded-token determinism is GPU-PAUSED-BY-OWNER per W-10 (J-13 plan).
"""

from __future__ import annotations

import os
import sys

import pytest

# server.py is sibling to this test file; add to path for direct import
# without packaging (mirrors the existing python/elai_inference test files
# in the codebase, e.g. test_xgrammar_logits_processor.py).
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

# The import may fail if torch/transformers are absent on the dev host;
# skip the entire test module in that case so the file still passes
# `python -m pytest --collect-only` discovery.
try:
    import torch  # noqa: F401
    from elai_inference.server import SeededGumbelProcessor  # type: ignore
    _SERVER_IMPORTABLE = True
except ImportError as e:
    _SERVER_IMPORTABLE = False
    _IMPORT_ERROR = e


pytestmark = pytest.mark.skipif(
    not _SERVER_IMPORTABLE,
    reason="server.py not importable (torch/transformers missing on host)",
)


# Fixed fixture logits: top-2 are close (2.0 vs 1.9) so Gumbel noise from
# different seeds can flip the argmax, but the contract is verified via
# tensor-inequality rather than argmax-inequality (gather prescription).
_FIXTURE_LOGITS = torch.tensor([[1.0, 2.0, 0.5, 1.9, 3.0, 1.5]])
_FIXTURE_INPUT_IDS = torch.zeros((1, 1), dtype=torch.long)


def _call_processor(proc: SeededGumbelProcessor) -> torch.Tensor:
    """Invoke __call__ with the shared fixture and return the result."""
    return proc(_FIXTURE_INPUT_IDS, _FIXTURE_LOGITS.clone())


def test_same_seed_same_scores():
    """Same seed on a fresh processor -> identical returned tensor."""
    out_a = _call_processor(SeededGumbelProcessor(seed=42))
    out_b = _call_processor(SeededGumbelProcessor(seed=42))
    assert torch.equal(out_a, out_b), (
        "same-seed contract broken: SeededGumbelProcessor(seed=42) produced "
        "different scores across two independent constructions on the same "
        "fixture logits"
    )


def test_diff_seed_diff_scores():
    """Different seeds -> different perturbations (tensor-inequality)."""
    out_42 = _call_processor(SeededGumbelProcessor(seed=42))
    out_43 = _call_processor(SeededGumbelProcessor(seed=43))
    assert not torch.equal(out_42, out_43), (
        "diff-seed contract broken: seed=42 and seed=43 produced identical "
        "scores on the same fixture logits; the per-seed generator is not "
        "diverging as expected"
    )


def test_step_advancement_changes_perturbation():
    """Two consecutive __call__ invocations on the same processor produce
    different perturbations because the per-step generator advances."""
    proc = SeededGumbelProcessor(seed=42)
    out_step0 = _call_processor(proc)
    out_step1 = _call_processor(proc)
    assert not torch.equal(out_step0, out_step1), (
        "step-advance contract broken: two consecutive __call__ invocations "
        "on the same SeededGumbelProcessor produced identical scores; the "
        "per-step generator is not advancing"
    )
