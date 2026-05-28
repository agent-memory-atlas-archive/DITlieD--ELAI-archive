"""E-9 — host-side tests for _common.py.

These tests run dependency-free (no httpx / scipy / datasets needed) so the
auditor can exercise the scaffolding without RUNPOD reachability.

Coverage:
  - paired_bootstrap_p returns a float in [0.0, 1.0]
  - paired_bootstrap_p is deterministic under fixed seed
  - paired_bootstrap_p handles empty / length-mismatch defensively
  - rotated_options is deterministic + cyclic permutation
  - BenchmarkRunObservedRow JSON shape matches the Rust payload schema
"""
from __future__ import annotations

import json
import os
import sys

# Path: tests/ is a sibling of _common.py
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import _common as common  # noqa: E402


def test_paired_bootstrap_p_in_unit_interval():
    a = [0.5, 0.6, 0.55, 0.7, 0.65, 0.58, 0.62, 0.61, 0.59, 0.6]
    b = [0.55, 0.65, 0.6, 0.72, 0.68, 0.61, 0.64, 0.63, 0.62, 0.6]
    p = common.paired_bootstrap_p(a, b, 200, 42)
    assert 0.0 <= p <= 1.0


def test_paired_bootstrap_p_deterministic_under_fixed_seed():
    a = [0.5, 0.6, 0.55, 0.7, 0.65]
    b = [0.55, 0.65, 0.6, 0.72, 0.68]
    p1 = common.paired_bootstrap_p(a, b, 100, 1234)
    p2 = common.paired_bootstrap_p(a, b, 100, 1234)
    assert abs(p1 - p2) < 1e-12


def test_paired_bootstrap_p_empty_returns_half():
    assert abs(common.paired_bootstrap_p([], [], 50, 1) - 0.5) < 1e-12


def test_paired_bootstrap_p_length_mismatch_returns_one():
    p = common.paired_bootstrap_p([0.5, 0.6], [0.7], 50, 1)
    assert abs(p - 1.0) < 1e-12


def test_rotated_options_cyclic():
    opts = ["A", "B", "C", "D"]
    rot = common.rotated_options(opts, seed=1)
    # Cyclic permutation means original set is preserved
    assert sorted(rot) == sorted(opts)
    assert len(rot) == len(opts)


def test_rotated_options_deterministic():
    opts = ["yes", "no", "abstain"]
    assert common.rotated_options(opts, seed=99) == common.rotated_options(opts, seed=99)


def test_benchmark_row_serialises_to_expected_shape():
    row = common.BenchmarkRunObservedRow(
        phase="completed",
        suite_name="swe-bench-pro",
        suite_version="2509.16941",
        score=0.23,
        n=200,
        sota_reference=0.23,
        p_value=0.04,
        low_power=False,
        required_n=196,
        runtime_lane="cloud_api",
        harness_blake3_prefix="e90000000000a9b1",
    )
    import dataclasses

    body = dataclasses.asdict(row)
    # Field names must mirror BenchmarkRunObservedPayload in Rust.
    expected_keys = {
        "phase",
        "suite_name",
        "suite_version",
        "score",
        "n",
        "sota_reference",
        "p_value",
        "low_power",
        "required_n",
        "model_effect_pp",
        "harness_effect_pp",
        "coverage_gap_pp",
        "runtime_lane",
        "harness_blake3_prefix",
        "ts_ms",
    }
    assert set(body.keys()) == expected_keys
    # JSON round-trip
    j = json.dumps(body)
    back = json.loads(j)
    assert back["suite_name"] == "swe-bench-pro"
    assert back["phase"] == "completed"
    assert back["score"] == 0.23


def test_env_gate_constant():
    assert common.ENV_GATE == "ELAI_EVAL_BENCHMARK_ENABLE"
