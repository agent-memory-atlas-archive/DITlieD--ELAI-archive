"""E-9 -- refinebench runner.

arXiv:2511.22173 RefineBench (RAG-indexed at project:paper-refinebench). 1,000 problems across 11 domains, checklist-based eval. Self-refinement: Gemini 2.5 Pro 31.3%, GPT-5 29.1% baseline. Guided refinement near-perfect with targeted feedback. RUNPOD-PENDING: pull fixture; multi-iteration refinement loop; baseline-vs-guided paired comparison.

This file ships as host-side scaffolding ONLY (workflow/reasoning/E-9-decisions.md
DECISION-5). Live runs RUNPOD-PENDING.
"""
from __future__ import annotations

import sys

import _common as common

SUITE_NAME = 'refinebench'
SUITE_VERSION = '2511.22173'
SOTA_REFERENCE = 0.313
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
