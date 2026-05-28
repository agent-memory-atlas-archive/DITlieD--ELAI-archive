"""E-9 -- swe-bench-live runner.

arXiv:2505.23419 SWE-bench-Live. 1,319 monthly-refresh tasks from real GitHub issues post-2024; 93 repos; automated curation pipeline; contamination-resistant via live updates. RUNPOD-PENDING: pull last 50 fresh issues from https://swe-bench-live.github.io/ leaderboard (HTTP cache TTL=24h via httpx); verify content-hash + temporal cutoff per E-7 contam filter.

This file ships as host-side scaffolding ONLY (workflow/reasoning/E-9-decisions.md
DECISION-5). Live runs RUNPOD-PENDING.
"""
from __future__ import annotations

import sys

import _common as common

SUITE_NAME = 'swe-bench-live'
SUITE_VERSION = '2505.23419'
SOTA_REFERENCE = 0.25
RUNTIME_LANE = 'cloud_api'

# E-7 contam filter: reject any issue whose `released_at` predates the
# `temporal_cutoff` (the cutoff is the model's training-data freeze date;
# `released_at` is the GitHub issue creation timestamp from SWE-bench-Live's
# monthly feed). RUNPOD-PENDING: populate via dataset metadata.
TEMPORAL_CUTOFF_ISO = "2024-12-31T00:00:00Z"  # Qwen2.5-Coder freeze (per HF card)


def _is_contaminated(released_at: str, temporal_cutoff: str = TEMPORAL_CUTOFF_ISO) -> bool:
    """Return True if `released_at` predates `temporal_cutoff` (i.e. the issue
    could have been in training data). Caller filters out contaminated items.
    """
    return released_at < temporal_cutoff


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
