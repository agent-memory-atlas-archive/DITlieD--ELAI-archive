"""E-9 — SWE-Bench Pro live runner (arXiv:2509.16941, primary suite).

Paper grounding (verified + RAG-indexed at project:paper-swe-bench-pro per
workflow/sessions/s-25120/completion-drive/grounding/
E-9_agent_eval_benchmarks-grounding.md entry 12):

  - 1,865 problems across 41 repos (public / held-out / commercial split).
  - Top models ~23% on the public set vs 70%+ on SWE-bench Verified.
  - Long-horizon multi-file patches (hours-to-days for human engineers).
  - Contamination-resistant via proprietary / held-out splits.

This file ships as host-side scaffolding ONLY. Live dataset pull + agent
patch evaluation against the 1,865 fixture issues happens on the RUNPOD
sidecar (~/swe-bench-pro/ on pythondev:6465).
"""
from __future__ import annotations

import sys

import _common as common  # noqa: E402  -- sibling module at runtime

SUITE_NAME = "swe-bench-pro"
SUITE_VERSION = "2509.16941"
SOTA_REFERENCE = 0.23  # top public-set score per paper §6.1


def main() -> int:
    args = common.parse_args(SUITE_NAME)

    # Loaded beacon: registry entry confirmed.
    common.BenchmarkRunObservedRow(
        phase="loaded",
        suite_name=SUITE_NAME,
        suite_version=SUITE_VERSION,
        sota_reference=SOTA_REFERENCE,
    ).emit()

    common.ensure_env_gate(SUITE_NAME)

    # Started beacon.
    common.BenchmarkRunObservedRow(
        phase="started",
        suite_name=SUITE_NAME,
        suite_version=SUITE_VERSION,
        sota_reference=SOTA_REFERENCE,
        runtime_lane="cloud_api",
    ).emit()

    # RUNPOD-PENDING: ssh maintainer@192.0.2.10 -p 6465 -- pull the SWE-Bench
    # Pro dataset via `datasets.load_dataset('princeton-nlp/SWE-Bench-Pro')`
    # (gated; held-out + commercial splits require auth). Run agent against
    # fixture issues; collect per-issue patch verdicts; aggregate to a
    # pass-rate; emit one ItemEvaluated row per fixture and one Completed
    # row at end with paired-bootstrap p vs prior month's score.
    sys.stderr.write(
        f"{SUITE_NAME}: live eval RUNPOD-PENDING -- scaffolding only.\n"
    )
    common.BenchmarkRunObservedRow(
        phase="aborted",
        suite_name=SUITE_NAME,
        suite_version=SUITE_VERSION,
        sota_reference=SOTA_REFERENCE,
        runtime_lane="cloud_api",
    ).emit()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
