"""E-9 -- clawsbench-split runner.

arXiv:2604.05172 ClawsBench (Apr 2026). 5 mock services, 44 tasks, 39-64% task success with 7-33% unsafe action rate across 6 models. Scaffold effect +39-63pp vs model effect 10pp. MANDATORY companion to harness changes. RUNPOD-PENDING: cross product {model A, B} x {harness X, Y}; 2-way ANOVA decomposes variance into model_effect_pp + harness_effect_pp + interaction.

This file ships as host-side scaffolding ONLY (workflow/reasoning/E-9-decisions.md
DECISION-5). Live runs RUNPOD-PENDING.
"""
from __future__ import annotations

import sys

import _common as common

SUITE_NAME = 'clawsbench-split'
SUITE_VERSION = '2604.05172'
SOTA_REFERENCE = 0.55
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
