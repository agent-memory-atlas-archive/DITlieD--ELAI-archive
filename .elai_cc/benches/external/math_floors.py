"""E-9 -- math-floors runner.

GSM8K-Platinum (gradientscience.org/gsm8k-platinum; madrylab/gsm8k-platinum HF; 110/219 broken items corrected) + MATH-500. Anti-stale-training: RAW GSM8K NEVER used; only the platinum-cleaned subset. Methodology paper arXiv:2502.03461. RUNPOD-PENDING: pull madrylab/gsm8k-platinum + MATH-500; threshold GSM8K-Platinum >= 90% local, MATH-500 >= 50% local.

This file ships as host-side scaffolding ONLY (workflow/reasoning/E-9-decisions.md
DECISION-5). Live runs RUNPOD-PENDING.
"""
from __future__ import annotations

import sys

import _common as common

SUITE_NAME = 'math-floors'
SUITE_VERSION = 'platinum+math500'
SOTA_REFERENCE = 0.9
RUNTIME_LANE = 'local_sidecar'


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
