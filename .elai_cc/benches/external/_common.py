"""E-9 shared utilities for external benchmark runners.

Provides:
  - Argparse front-end with the canonical flag set (--suite, --p-target,
    --harness-vary, --json).
  - Paired-bootstrap p-value implementation mirroring the Rust
    `orchestrator::bench_significance::paired_bootstrap_p` shape for
    cross-language determinism (SplitMix64-equivalent xorshift).
  - SSB-row emission helper that writes ONE JSON line to stdout matching
    `BenchmarkRunObservedPayload` per
    .elai_cc/crates/orchestrator/src/ssb.rs:17750 (the Rust caller parses
    stdout to extract the row).

All RUNPOD-PENDING markers are co-located with the live-eval call sites.

This file is pure scaffolding. No dataset downloads; no live model calls.
Per workflow/reasoning/E-9-decisions.md DECISION-5, live runs only fire
when ELAI_EVAL_BENCHMARK_ENABLE=1 AND the live invocation is gated by
the orchestrator-side dispatcher (the Rust CLI sets this env var).

References (per E-9-decisions.md DECISION-3 — verified arXiv IDs only):
  - arXiv:2509.16941 SWE-Bench Pro
  - arXiv:2505.23419 SWE-bench-Live
  - arXiv:2511.22173 RefineBench
  - arXiv:2604.05172 ClawsBench
  - arXiv:2311.12983 GAIA
  - arXiv:2601.18137 DeepPlanning
  - arXiv:2506.07982 tau2
  - arXiv:2406.07791 LLM-as-judge position bias (judge rotation source)
  - arXiv:2306.05685 MT-Bench foundational
  - gradientscience.org/gsm8k-platinum (110/219 cleanup)
"""
from __future__ import annotations

import argparse
import os
import sys
import time
from dataclasses import dataclass, field, asdict
from typing import Iterable, Sequence

try:
    import orjson as _json_lib  # noqa: F401

    def _dumps(obj):
        return _json_lib.dumps(obj).decode("utf-8")

except ImportError:  # graceful fallback when deps aren't installed (host-side)
    import json as _json_lib

    def _dumps(obj):
        return _json_lib.dumps(obj)


ENV_GATE = "ELAI_EVAL_BENCHMARK_ENABLE"


@dataclass
class BenchmarkRunObservedRow:
    """Mirror of Rust crates/orchestrator/src/ssb.rs BenchmarkRunObservedPayload.

    The Rust dispatcher parses one JSON line per emitted row.
    """

    phase: str  # "loaded" | "started" | "item_evaluated" | "completed" | "aborted"
    suite_name: str
    suite_version: str
    score: float | None = None
    n: int | None = None
    sota_reference: float | None = None
    p_value: float | None = None
    low_power: bool = False
    required_n: int | None = None
    model_effect_pp: float | None = None
    harness_effect_pp: float | None = None
    coverage_gap_pp: float | None = None
    runtime_lane: str = ""
    harness_blake3_prefix: str = ""
    ts_ms: int = field(default_factory=lambda: int(time.time() * 1000))

    def emit(self) -> None:
        """Write one JSON line to stdout (the orchestrator parses it)."""
        sys.stdout.write(_dumps(asdict(self)) + "\n")
        sys.stdout.flush()


def parse_args(suite_default: str) -> argparse.Namespace:
    """Standard CLI front-end for all 11 suite runners."""
    parser = argparse.ArgumentParser(
        description=f"E-9 external benchmark runner: {suite_default}"
    )
    parser.add_argument("--suite", default=suite_default, help="benchmark suite wire name")
    parser.add_argument(
        "--p-target",
        type=float,
        default=0.05,
        help="paired-bootstrap p threshold (default 0.05)",
    )
    parser.add_argument(
        "--harness-vary",
        action="store_true",
        help="ClawsBench-style 2x2 harness-vs-model cross-product (only honoured by clawsbench-split)",
    )
    parser.add_argument("--json", action="store_true", help="emit JSON (default)")
    return parser.parse_args()


# ---------------------------------------------------------------------------
# Paired-bootstrap (xorshift PRNG; cross-language reproducible with Rust)
# ---------------------------------------------------------------------------


def _splitmix64(state: int) -> tuple[int, int]:
    """Single-step SplitMix64; mirrors crates/orchestrator/src/bench_significance.rs.

    The Rust implementation uses identical magic numbers + shifts; given the
    same seed both languages produce the same sequence.
    """
    state = (state + 0x9E37_79B9_7F4A_7C15) & 0xFFFF_FFFF_FFFF_FFFF
    z = state
    z = ((z ^ (z >> 30)) * 0xBF58_476D_1CE4_E5B9) & 0xFFFF_FFFF_FFFF_FFFF
    z = ((z ^ (z >> 27)) * 0x94D0_49BB_1331_11EB) & 0xFFFF_FFFF_FFFF_FFFF
    return state, z ^ (z >> 31)


def paired_bootstrap_p(a: Sequence[float], b: Sequence[float], n_iter: int, seed: int) -> float:
    """Two-sided paired-bootstrap p with add-1 smoothing.

    Output identical to crates/orchestrator/src/bench_significance.rs under
    the same `seed`. Returns 0.5 on empty input; 1.0 on length mismatch.
    """
    if not a or not b:
        return 0.5
    if len(a) != len(b):
        return 1.0
    n = len(a)
    diffs = [bi - ai for ai, bi in zip(a, b)]
    observed = sum(diffs) / n
    abs_observed = abs(observed)
    state = seed
    extreme = 0
    for _ in range(n_iter):
        resample_sum = 0.0
        for _ in range(n):
            state, rnd = _splitmix64(state)
            idx = rnd % n
            resample_sum += diffs[idx] - observed
        resample_mean = resample_sum / n
        if abs(resample_mean) >= abs_observed:
            extreme += 1
    return (extreme + 1.0) / (n_iter + 1.0)


# ---------------------------------------------------------------------------
# Env-gate guard
# ---------------------------------------------------------------------------


def ensure_env_gate(suite_name: str) -> None:
    """Refuse live dispatch when ELAI_EVAL_BENCHMARK_ENABLE != "1"."""
    if os.environ.get(ENV_GATE, "") != "1":
        BenchmarkRunObservedRow(
            phase="aborted",
            suite_name=suite_name,
            suite_version="env-gate",
        ).emit()
        sys.stderr.write(
            f"E-9 abort: {ENV_GATE} must be '1' to dispatch the live runner. "
            f"Host-side scaffolding only -- live eval RUNPOD-PENDING.\n"
        )
        sys.exit(2)


# ---------------------------------------------------------------------------
# Judge rotation helper (anti-bias contract; arXiv:2406.07791 + 2306.05685)
# ---------------------------------------------------------------------------


def rotated_options(options: Sequence[str], rotation_seed: int) -> list[str]:
    """Deterministic cyclic rotation of LLM-judge options.

    Per arXiv:2406.07791 §3 position-bias and arXiv:2306.05685 §6 verbosity-
    bias findings: every LLM-as-judge call MUST permute option positions
    between turns. This helper produces a deterministic rotation from a
    seed so the same trace is reproducible.
    """
    if not options:
        return list(options)
    state, _ = _splitmix64(rotation_seed)
    shift = state % len(options)
    return list(options[shift:]) + list(options[:shift])


# ---------------------------------------------------------------------------
# E-31 coding-model-leaderboard scraper substrate
# ---------------------------------------------------------------------------
#
# Mirror of crates/orchestrator/src/ssb.rs::CodingModelLeaderboardEventPayload
# (SCHEMA_VERSION 174). The Rust dispatcher parses one JSON line per emitted
# row. Live scrape RUNPOD-PENDING; default-off ENV gate matches the SSB
# variant's wire-dark behaviour so existing tests keep their row counts.
#
# Score is reported in basis points (0-10000) so a source reporting "46%"
# lands as 4600 on the wire -- avoids float drift on the round-trip and
# keeps the contract test deterministic. The `entry_hash` field is a
# blake3 8-byte hex prefix (blake3 NOT blake2/sha2 per CLAUDE.md
# Anti-Stale-Training) of the canonical row tuple.
#
# Citation-verified May 2026:
#   - bigcode-bench.github.io (BigCodeBench leaderboard, ICLR'25)
#   - evalplus.github.io/leaderboard.html (HumanEval+ / MBPP+)
#   - labs.scale.com/leaderboard/swe_bench_pro_public (SWE-Bench Pro)
#   - livecodebench.github.io/leaderboard.html (contamination-free)
#   - morphllm.com/swe-bench-pro (Pro vs Verified gap)
#   - platform.claude.com/docs/en/about-claude/pricing (Anthropic 2026)


CODING_LEADERBOARD_ENV_GATE = "ELAI_CODING_MODEL_LEADERBOARD_ENABLE"


@dataclass
class CodingModelLeaderboardEventRow:
    """Mirror of Rust CodingModelLeaderboardEventPayload.

    Emit one JSON line per scrape phase. The Rust dispatcher parses the line
    and writes an SSB row of action_type=coding_model_leaderboard_event.
    """

    kind: str  # "scrape_started" | "scrape_completed" | "scrape_failed"
    suite_name: str
    suite_version: str = ""
    leaderboard_url: str = ""
    scrape_run_id: str = ""
    model_name: str = ""
    score_pct_bp: int = 0  # basis points 0-10000
    sample_size: int = 0
    entry_hash: str = ""  # blake3 8-byte hex prefix
    pricing_label: str = ""
    reason_label: str = ""
    http_status: int = 0
    session_id: str | None = None
    details_json: dict | None = None
    ts_ms: int = field(default_factory=lambda: int(time.time() * 1000))

    def emit(self) -> None:
        """Write one JSON line to stdout (the orchestrator parses it)."""
        payload = asdict(self)
        if payload.get("details_json") is None:
            payload["details_json"] = {}
        sys.stdout.write(_dumps(payload) + "\n")
        sys.stdout.flush()


def ensure_coding_leaderboard_env_gate(suite_name: str) -> None:
    """Refuse live scrape when ELAI_CODING_MODEL_LEADERBOARD_ENABLE != '1'.

    Mirrors the BenchmarkRunObservedRow ensure_env_gate pattern. Emits a
    `scrape_failed` row with reason_label='env_gate_off' so the dispatcher
    still observes the refusal and exits 2 so the calling CLI surfaces a
    non-zero status to the user.
    """
    if os.environ.get(CODING_LEADERBOARD_ENV_GATE, "") != "1":
        CodingModelLeaderboardEventRow(
            kind="scrape_failed",
            suite_name=suite_name,
            reason_label="env_gate_off",
        ).emit()
        sys.stderr.write(
            f"E-31 abort: {CODING_LEADERBOARD_ENV_GATE} must be '1' to "
            f"dispatch the live scraper. Host-side scaffolding only -- "
            f"live scrape RUNPOD-PENDING.\n"
        )
        sys.exit(2)


def score_str_to_bp(score_str: str) -> int:
    """Parse a score string like '46', '46%', '0.46', '46.3', '46.3%' to basis points.

    Returns 0 on parse failure. 46% / 0.46 / 46.0 / 46 all return 4600. Caller
    must NOT trust the return value for parse-error detection -- pair this
    with a try/except OR an explicit format-known guard at the caller.
    """
    raw = score_str.strip().rstrip("%").strip()
    if not raw:
        return 0
    try:
        v = float(raw)
    except ValueError:
        return 0
    # If value is between 0 and 1.5 we treat it as a fraction; else as %
    if 0.0 <= v <= 1.5:
        return int(round(v * 10000.0))
    return int(round(v * 100.0))


__all__ = [
    "BenchmarkRunObservedRow",
    "CODING_LEADERBOARD_ENV_GATE",
    "CodingModelLeaderboardEventRow",
    "ENV_GATE",
    "ensure_coding_leaderboard_env_gate",
    "ensure_env_gate",
    "paired_bootstrap_p",
    "parse_args",
    "rotated_options",
    "score_str_to_bp",
]
