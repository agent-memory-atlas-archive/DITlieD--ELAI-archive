"""E-9 -- bfcl-v3 runner.

gorilla.cs.berkeley.edu BFCL v3. ICML 2025 paper proceedings.mlr.press/v267/patil25a.html. 2k question-function-answer pairs, multi-language (Python/JS/Java/REST), AST-based eval. GLM 4.5 SOTA 76.7, Qwen3-32B 75.7. RUNPOD-PENDING: pull leaderboard fixture; per-role function-calling correctness; threshold >= 70%.

This file ships as host-side scaffolding ONLY (workflow/reasoning/E-9-decisions.md
DECISION-5). Live runs RUNPOD-PENDING.
"""
from __future__ import annotations

import sys

import _common as common

SUITE_NAME = 'bfcl-v3'
SUITE_VERSION = 'gorilla-icml-2025'
SOTA_REFERENCE = 0.767
RUNTIME_LANE = 'mixed'


def main() -> int:
    args = common.parse_args(SUITE_NAME)

    common.BenchmarkRunObservedRow(
        phase="loaded",
        suite_name=SUITE_NAME,
        suite_version=SUITE_VERSION,
        sota_reference=SOTA_REFERENCE,
    ).emit()

    common.ensure_env_gate(SUITE_NAME)

    common.BenchmarkRunObservedRow(
        phase="started",
        suite_name=SUITE_NAME,
        suite_version=SUITE_VERSION,
        sota_reference=SOTA_REFERENCE,
        runtime_lane=RUNTIME_LANE,
    ).emit()

    # RUNPOD-PENDING: live eval dispatch (see module docstring for command).
    sys.stderr.write(f"{SUITE_NAME}: live eval RUNPOD-PENDING -- scaffolding only.\n")
    common.BenchmarkRunObservedRow(
        phase="aborted",
        suite_name=SUITE_NAME,
        suite_version=SUITE_VERSION,
        sota_reference=SOTA_REFERENCE,
        runtime_lane=RUNTIME_LANE,
    ).emit()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
