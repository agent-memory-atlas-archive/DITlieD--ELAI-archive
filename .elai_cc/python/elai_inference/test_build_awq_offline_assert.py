"""J-19 S5 slice-4: AWQ build fail-closed assert (offline, no GPU, no autoawq).

Verifies the pre-flight calibration guard inserted at the top of
build_awq_checkpoint raises RuntimeError for invalid calibration_size
BEFORE the autoawq import is attempted. Lets the assert run on CPU-only
hosts (no autoawq install, no model download).

Run: python -m pytest elai_inference/test_build_awq_offline_assert.py
"""

from __future__ import annotations

import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from elai_inference.scripts.build_awq import build_awq_checkpoint  # noqa: E402


def test_build_awq_refuses_zero_calibration_size():
    with pytest.raises(RuntimeError, match="calibration_size must be >= 1"):
        build_awq_checkpoint(
            model_path="/unused",
            output_path="/unused",
            calibration_size=0,
        )


def test_build_awq_refuses_negative_calibration_size():
    with pytest.raises(RuntimeError, match="calibration_size must be >= 1"):
        build_awq_checkpoint(
            model_path="/unused",
            output_path="/unused",
            calibration_size=-1,
        )
