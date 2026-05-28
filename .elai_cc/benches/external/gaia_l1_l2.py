"""E-9 -- gaia-l1-l2 runner.

arXiv:2311.12983 GAIA (ICLR 2024). 466 real-world questions; humans 92% vs GPT-4+plugins 15%. Multi-modal, web-browsing, tool-use. Claude Sonnet 4.5 leads 74.6%. RUNPOD-PENDING: HAL framework (hal.cs.princeton.edu/gaia); L1 >= 50%, L2 >= 30% required for shipped tag.

This file ships as host-side scaffolding ONLY (workflow/reasoning/E-9-decisions.md
DECISION-5). Live runs RUNPOD-PENDING.
"""
from __future__ import annotations

import sys

import _common as common

SUITE_NAME = 'gaia-l1-l2'
SUITE_VERSION = '2311.12983'
SOTA_REFERENCE = 0.746
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
