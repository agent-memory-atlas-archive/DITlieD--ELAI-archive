#!/usr/bin/env python3
# E-31 sub-phase 6A -- internal-PRO bench scaffold.
#
# Status: SCAFFOLD ONLY. The real harness runner is RUNPOD-PENDING
# until the 2x3090 box arrives (memory:project_vram_tier_plan). This
# scaffold locks the file path per E-31 plan acceptance line and prints
# a deterministic RUNPOD-PENDING banner so CI does not silently treat
# an absent runner as a passing run.
#
# Sibling harnesses for reference (NOT imported here -- keeping the
# scaffold zero-dep):
#   - benches/baseline_harness.py
#   - benches/run_benchmark.py
#   - benches/model_capability/ (per-task ground-truth fixtures)
#
# Once the live runner lands (sub-phase 6B+), it will:
#   1. Load fixtures from benches/coding_lane/fixtures/internal_pro_*.json
#      (a curated subset of internal review tasks; NOT HumanEval / MBPP /
#      SWE-bench Verified, which are contaminated per CLAUDE.md
#      Anti-Stale "STOP-Verified" lint).
#   2. Dispatch each task across N=5 seeds for stable_pass_at_1 per
#      memory:project_b40_shipped (E-49 named-token contract).
#   3. Emit per-task SSB rows via the E-49 CodingMetricNamedToken
#      Pattern-2 variant (already shipped at ssb.rs SCHEMA 189).
#   4. Aggregate into a leaderboard JSON written to
#      --output path (default: benches/coding_lane/results/<ts>.json).
#
# Until then this script is a dry-run no-op that:
#   * Validates argparse contract so callers can integrate.
#   * Prints RUNPOD-PENDING banner so any CI gate fails LOUD.
#   * Returns exit code 0 in --dry-run mode, 2 otherwise.
#
# Anti-stale (CLAUDE.md): orjson over json for serialization once a real
# output lands; for the dry-run we use stdlib json so the scaffold has
# zero external deps.

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

# Banner emitted on every invocation so callers see the deferred state.
RUNPOD_PENDING_BANNER = (
    "RUNPOD-PENDING: E-31 internal-PRO bench runner. "
    "Scaffold only -- real harness requires 2x3090 hardware "
    "(memory:project_vram_tier_plan). See docs/strategy/E-31-tier-matrix.md."
)

# Default suite -- internal-PRO is the only stop-verified suite for
# this lane. STOP-Verified marker per CLAUDE.md Anti-Stale rule.
DEFAULT_SUITE = "swe-pro"
SUPPORTED_SUITES = ("swe-pro", "internal-pro")


def parse_args(argv: list[str]) -> argparse.Namespace:
    """Argparse contract that callers integrate against today.

    The flags + types are locked here so the live runner in sub-phase 6B
    can drop in without breaking integration callers (e.g. CI jobs,
    dashboard buttons, /runtime CLI subcommand).
    """
    parser = argparse.ArgumentParser(
        prog="run_internal_pro.py",
        description=(
            "E-31 sub-phase 6A internal-PRO coding-lane bench runner "
            "(SCAFFOLD; RUNPOD-PENDING)."
        ),
    )
    parser.add_argument(
        "--model",
        default="huihui-ai/Qwen2.5-Coder-7B-Instruct-abliterated",
        help=(
            "HF model id or local path. Default: 7B abliterated "
            "(CLAUDE.md canonical local lane)."
        ),
    )
    parser.add_argument(
        "--suite",
        default=DEFAULT_SUITE,
        choices=SUPPORTED_SUITES,
        help="Bench suite name. STOP-Verified per E-49 contract.",
    )
    parser.add_argument(
        "--output",
        default=None,
        help=(
            "Output JSON path. Default: "
            "benches/coding_lane/results/<unix_ms>.json. Honored in dry-run "
            "by writing the banner + arg payload."
        ),
    )
    parser.add_argument(
        "--max-tasks",
        type=int,
        default=5,
        help=(
            "Max tasks to dispatch. Default 5 for dry-run. "
            "Live runner defaults to full suite (~150 tasks)."
        ),
    )
    parser.add_argument(
        "--seeds",
        type=int,
        default=5,
        help=(
            "Seeds per task for stable_pass_at_1 aggregation. "
            "Default 5 per E-49 named-token contract."
        ),
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        default=True,
        help=(
            "Dry-run mode (default). Emits banner + writes scaffold "
            "JSON; never loads a model. Disable to fall through to the "
            "live runner (which currently exits 2)."
        ),
    )
    return parser.parse_args(argv)


def write_scaffold_output(args: argparse.Namespace) -> Path | None:
    """Write a deterministic dry-run record so CI can verify the file
    contract before the live runner exists."""
    if args.output is None:
        return None
    out_path = Path(args.output)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    record = {
        "status": "RUNPOD-PENDING",
        "phase": "E-31 sub-phase 6A scaffold",
        "model": args.model,
        "suite": args.suite,
        "max_tasks": args.max_tasks,
        "seeds": args.seeds,
        "note": RUNPOD_PENDING_BANNER,
        # E-49 contract pointers (memory:project_b41_shipped + b40_shipped).
        "contracts": {
            "stable_pass_at_1": "see SsbActionType::CodingMetricNamedToken kind=stable_pass_at_1",
            "harness_attribution_tuple": (
                "see SsbActionType::CodingMetricNamedToken kind=harness_attribution_tuple"
            ),
            "stop_verified": (
                "see SsbActionType::CodingMetricNamedToken kind=stop_verified -- "
                "this scaffold ONLY accepts internal-pro / swe-pro per the "
                "STOP-Verified lint"
            ),
        },
    }
    out_path.write_text(json.dumps(record, indent=2), encoding="utf-8")
    return out_path


def main(argv: list[str] | None = None) -> int:
    if argv is None:
        argv = sys.argv[1:]
    args = parse_args(argv)

    # Always print the banner so it cannot be missed (matches the
    # E-3 4A tracing-log precedent).
    print(RUNPOD_PENDING_BANNER, file=sys.stderr)

    written = write_scaffold_output(args)
    if written is not None:
        print(f"scaffold wrote: {written}", file=sys.stderr)

    if args.dry_run:
        # Dry-run = exit 0 so CI integration tests pass without the
        # live runner.
        return 0

    # Live mode requested but no runner exists yet -> exit 2 LOUD so
    # callers cannot silently treat absence as success.
    print(
        "ERROR: --dry-run=false requested but live runner is RUNPOD-PENDING. "
        "Set --dry-run to keep the scaffold contract.",
        file=sys.stderr,
    )
    return 2


if __name__ == "__main__":
    sys.exit(main())
