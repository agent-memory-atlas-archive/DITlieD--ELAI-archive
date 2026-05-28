"""E-9 -- tau2-bench runner.

arXiv:2506.07982 / sierra-research/tau2-bench. 4 domains (Mock, Airline, Retail, Telecom); dual-control evaluation. Anthropic SOTA 0.862 retail / 0.993 telecom. Policy-adherence rubric (booking a flight while violating change-fee policy fails). Cloud thresholds: retail >= 0.70 / telecom >= 0.85. RUNPOD-PENDING: clone sierra-research/tau2-bench; run telecom + retail subsets; orchestrator policy-check pass.

This file ships as host-side scaffolding ONLY (workflow/reasoning/E-9-decisions.md
DECISION-5). Live runs RUNPOD-PENDING.
"""
from __future__ import annotations

import sys

import _common as common

SUITE_NAME = 'tau2-bench'
SUITE_VERSION = '2506.07982'
SOTA_REFERENCE = 0.862
RUNTIME_LANE = 'cloud_api'


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
