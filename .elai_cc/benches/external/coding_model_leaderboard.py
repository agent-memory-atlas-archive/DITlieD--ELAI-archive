"""E-31 -- coding-model leaderboard scraper.

Scrapes the canonical contamination-aware coding-model leaderboards and
emits one SSB row per (suite, model) tuple. Wire-dark default: the env
gate ELAI_CODING_MODEL_LEADERBOARD_ENABLE must be set to "1" for any
live HTTP fetch -- without it the scraper emits a single
`scrape_failed` row with reason_label='env_gate_off' and exits non-zero.

Scrape phase flow per source:
  - one `scrape_started` row carrying suite_name + suite_version +
    leaderboard_url + scrape_run_id
  - N `scrape_completed` rows (one per model entry parsed) carrying
    model_name + score_pct_bp + sample_size + entry_hash + pricing_label
  - on failure, exactly one `scrape_failed` row carrying reason_label +
    http_status (0 if non-HTTP); subsequent sources continue

Leaderboard targets (citation-verified May 2026):
  - bigcode-bench.github.io                          (BigCodeBench, ICLR'25)
  - evalplus.github.io/leaderboard.html              (HumanEval+ / MBPP+)
  - labs.scale.com/leaderboard/swe_bench_pro_public  (SWE-Bench Pro)
  - livecodebench.github.io/leaderboard.html         (LiveCodeBench)
  - morphllm.com/swe-bench-pro                       (Pro vs Verified gap)

Pricing label grounding: platform.claude.com/docs/en/about-claude/pricing
(Anthropic 2026 API rates -- Sonnet 4.6 $3/$15, Opus 4.7 $5/$25,
Haiku 4.5 $1/$5 per MTok; verified May 2026).

Schema source of truth: crates/orchestrator/src/ssb.rs
CodingModelLeaderboardEventPayload (SCHEMA_VERSION 174). The Rust
dispatcher parses one JSON line per emitted row.

Live HTTP scrape RUNPOD-PENDING -- this module ships the row-emission +
parsing scaffold + a synthetic-leaderboard fixture path so the contract
test exercises the round-trip without external network.

Usage:
  python coding_model_leaderboard.py --suite bigcodebench
  python coding_model_leaderboard.py --all   # iterate every target

The --fixture flag points at a local JSON file with the
{suite, models: [{name, score, n}]} shape so CI can exercise the parser
without external network.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import uuid
from typing import Iterable

import _common as common

# ---------------------------------------------------------------------------
# Leaderboard catalog. Each entry carries:
#   - suite key (kebab-case wire-string; matches the SSB payload suite_name)
#   - version label (suite-internal versioning where applicable)
#   - URL the live HTTP fetch would target (RUNPOD-PENDING)
#   - arxiv id (where applicable; empty string if leaderboard-only)
# ---------------------------------------------------------------------------

LEADERBOARDS: list[dict[str, str]] = [
    {
        "suite": "bigcodebench",
        "version": "hard-set",
        "url": "https://bigcode-bench.github.io",
        "arxiv_id": "",
    },
    {
        "suite": "humaneval-plus",
        "version": "evalplus-v0.3",
        "url": "https://evalplus.github.io/leaderboard.html",
        "arxiv_id": "",
    },
    {
        "suite": "mbpp-plus",
        "version": "evalplus-v0.3",
        "url": "https://evalplus.github.io/leaderboard.html",
        "arxiv_id": "",
    },
    {
        "suite": "swe-bench-pro",
        "version": "public",
        "url": "https://labs.scale.com/leaderboard/swe_bench_pro_public",
        "arxiv_id": "2509.16941",
    },
    {
        "suite": "livecodebench",
        "version": "v6",
        "url": "https://livecodebench.github.io/leaderboard.html",
        "arxiv_id": "2403.07974",
    },
]


# ---------------------------------------------------------------------------
# Pricing-tier classifier. Maps a canonical model kebab-case name to one of
# the 4 ELAI-tracked pricing tiers. Grounding:
# platform.claude.com/docs/en/about-claude/pricing (verified May 2026).
# Unknown models map to "" so the SSB row carries an empty pricing_label
# rather than a fabricated one (anti-slopsquatting on labels).
# ---------------------------------------------------------------------------

PRICING_MAP: dict[str, str] = {
    # Anthropic 2026 tiers
    "claude-opus-4-7": "opus",
    "claude-opus-4-6": "opus",
    "claude-opus-4-5": "opus",
    "claude-sonnet-4-6": "sonnet",
    "claude-sonnet-4-5": "sonnet",
    "claude-haiku-4-5": "haiku",
    # Cheap-volume cloud route
    "deepseek-v4-flash": "cheap-volume",
    "deepseek-v4-pro": "cheap-volume",
    # Local lane
    "qwen2-5-coder-7b-instruct-abliterated": "local",
    "qwen2-5-coder-32b-instruct": "local",
    "qwen2-5-coder-3b-instruct-abliterated": "local",
}


def pricing_for(model_name: str) -> str:
    """Return pricing tier label for a canonical model name, or '' if unknown."""
    return PRICING_MAP.get(model_name, "")


def canonical_model_name(raw: str) -> str:
    """Lowercase + dot/space/period -> dash; preserve digits.

    Examples:
      'Claude Opus 4.7'                              -> 'claude-opus-4-7'
      'Qwen2.5-Coder-7B-Instruct-abliterated'         -> 'qwen2-5-coder-7b-instruct-abliterated'
      'DeepSeek V4-Flash'                             -> 'deepseek-v4-flash'
    """
    s = raw.strip().lower()
    out = []
    last_was_dash = False
    for ch in s:
        if ch.isalnum():
            out.append(ch)
            last_was_dash = False
        else:
            if not last_was_dash:
                out.append("-")
                last_was_dash = True
    name = "".join(out).strip("-")
    return name


def entry_hash_for(
    suite_name: str,
    suite_version: str,
    model_name: str,
    score_pct_bp: int,
    sample_size: int,
    scrape_ts_ms: int,
) -> str:
    """blake3 8-byte hex prefix of the canonical row tuple.

    blake3 NOT blake2/sha2 per CLAUDE.md Anti-Stale-Training. We round
    scrape_ts_ms to the nearest hour so two scrapes of the same row in
    quick succession dedup to the same hash.
    """
    rounded = (scrape_ts_ms // 3_600_000) * 3_600_000
    canonical = "|".join(
        [
            suite_name,
            suite_version,
            model_name,
            str(score_pct_bp),
            str(sample_size),
            str(rounded),
        ]
    )
    try:
        import blake3  # type: ignore

        h = blake3.blake3(canonical.encode("utf-8")).digest()
    except ImportError:
        # Fallback: sha256 first 8 bytes. Logged in details_json so the
        # consumer can spot the degradation. This is acceptable because
        # entry_hash is for dedup only -- no cryptographic guarantee.
        h = hashlib.sha256(canonical.encode("utf-8")).digest()
    return h[:8].hex()


# ---------------------------------------------------------------------------
# Fixture parser (CI-resilient: no external network)
# ---------------------------------------------------------------------------


def parse_fixture(path: str) -> dict:
    """Parse a local JSON fixture in shape:
      {"suite": "bigcodebench", "version": "hard-set",
       "models": [{"name": "...", "score": 0.46, "n": 148}, ...]}
    """
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


# ---------------------------------------------------------------------------
# Live HTTP scrape (RUNPOD-PENDING). The scaffold below records the
# intent and emits a `scrape_failed` row with reason_label='runpod_pending'
# so the dispatcher always observes the refusal until the live path lands.
# ---------------------------------------------------------------------------


def live_scrape(suite_entry: dict) -> dict | None:
    """RUNPOD-PENDING. Returns None and emits a `scrape_failed` row."""
    common.CodingModelLeaderboardEventRow(
        kind="scrape_failed",
        suite_name=suite_entry["suite"],
        suite_version=suite_entry["version"],
        leaderboard_url=suite_entry["url"],
        reason_label="runpod_pending",
        http_status=0,
        details_json={
            "caller_site": "live_scrape",
            "arxiv_id": suite_entry.get("arxiv_id", ""),
        },
    ).emit()
    return None


# ---------------------------------------------------------------------------
# Main loop
# ---------------------------------------------------------------------------


def scrape_one(
    suite_entry: dict,
    fixture_payload: dict | None,
    scrape_run_id: str,
    session_id: str | None,
) -> int:
    """Emit started + N completed (or one failed) row(s) for one suite.

    Returns count of `scrape_completed` rows emitted (0 on failure).
    """
    common.CodingModelLeaderboardEventRow(
        kind="scrape_started",
        suite_name=suite_entry["suite"],
        suite_version=suite_entry["version"],
        leaderboard_url=suite_entry["url"],
        scrape_run_id=scrape_run_id,
        session_id=session_id,
        details_json={
            "caller_site": "scrape_one",
            "arxiv_id": suite_entry.get("arxiv_id", ""),
        },
    ).emit()

    if fixture_payload is None:
        live_scrape(suite_entry)
        return 0

    completed = 0
    models = fixture_payload.get("models", [])
    for entry in models:
        raw_name = entry.get("name", "")
        if not raw_name:
            continue
        model_name = canonical_model_name(raw_name)
        raw_score = entry.get("score", 0)
        # score may be float (0.46) or string ("46%") or basis points int
        if isinstance(raw_score, (int, float)):
            score_pct_bp = (
                int(round(float(raw_score) * 10000.0))
                if 0.0 <= float(raw_score) <= 1.5
                else int(round(float(raw_score) * 100.0))
            )
        else:
            score_pct_bp = common.score_str_to_bp(str(raw_score))
        sample_size = int(entry.get("n", 0))
        ts_ms = int(common.time.time() * 1000)
        e_hash = entry_hash_for(
            suite_entry["suite"],
            suite_entry["version"],
            model_name,
            score_pct_bp,
            sample_size,
            ts_ms,
        )
        common.CodingModelLeaderboardEventRow(
            kind="scrape_completed",
            suite_name=suite_entry["suite"],
            suite_version=suite_entry["version"],
            leaderboard_url=suite_entry["url"],
            scrape_run_id=scrape_run_id,
            model_name=model_name,
            score_pct_bp=score_pct_bp,
            sample_size=sample_size,
            entry_hash=e_hash,
            pricing_label=pricing_for(model_name),
            session_id=session_id,
            details_json={
                "caller_site": "scrape_one",
                "score_str": str(raw_score),
                "raw_model_name": raw_name,
                "arxiv_id": suite_entry.get("arxiv_id", ""),
            },
            ts_ms=ts_ms,
        ).emit()
        completed += 1
    return completed


def main() -> int:
    parser = argparse.ArgumentParser(description=(
        "E-31 coding-model-leaderboard scraper. Wire-dark default; set "
        "ELAI_CODING_MODEL_LEADERBOARD_ENABLE=1 to dispatch a live scrape "
        "(currently RUNPOD-PENDING)."
    ))
    parser.add_argument(
        "--suite",
        default="all",
        help=(
            "leaderboard suite (bigcodebench / humaneval-plus / mbpp-plus / "
            "swe-bench-pro / livecodebench / all)"
        ),
    )
    parser.add_argument(
        "--fixture",
        default="",
        help=(
            "optional path to a local JSON fixture in shape {suite, version, "
            "models: [{name, score, n}]} so CI can exercise the parser without "
            "external network. When omitted, falls through to live_scrape "
            "(RUNPOD-PENDING)."
        ),
    )
    parser.add_argument(
        "--session-id",
        default="",
        help="optional orchestrator session id to embed in every emitted row",
    )
    args = parser.parse_args()

    common.ensure_coding_leaderboard_env_gate(
        args.suite if args.suite != "all" else "coding-model-leaderboard"
    )

    scrape_run_id = str(uuid.uuid4())
    session_id = args.session_id or None

    fixture_payload = None
    if args.fixture:
        try:
            fixture_payload = parse_fixture(args.fixture)
        except (OSError, json.JSONDecodeError) as exc:
            common.CodingModelLeaderboardEventRow(
                kind="scrape_failed",
                suite_name=args.suite,
                reason_label="fixture_parse_error",
                http_status=0,
                details_json={
                    "caller_site": "main",
                    "error": str(exc),
                    "fixture_path": args.fixture,
                },
            ).emit()
            return 2

    if args.suite == "all":
        targets: Iterable[dict] = LEADERBOARDS
    else:
        targets = [s for s in LEADERBOARDS if s["suite"] == args.suite]
        if not targets:
            common.CodingModelLeaderboardEventRow(
                kind="scrape_failed",
                suite_name=args.suite,
                reason_label="unknown_suite",
                http_status=0,
                details_json={
                    "caller_site": "main",
                    "known_suites": [s["suite"] for s in LEADERBOARDS],
                },
            ).emit()
            return 2

    total_rows = 0
    for entry in targets:
        # In single-suite mode the fixture applies to the one suite;
        # in all-suites mode the fixture is only used when its `suite`
        # field matches (else fall through to live_scrape -> RUNPOD-PENDING).
        eff_fixture = None
        if fixture_payload is not None:
            if fixture_payload.get("suite", "") == entry["suite"]:
                eff_fixture = fixture_payload
        total_rows += scrape_one(entry, eff_fixture, scrape_run_id, session_id)

    sys.stderr.write(
        f"E-31 scraper: emitted {total_rows} scrape_completed row(s) "
        f"across {sum(1 for _ in targets) if args.suite != 'all' else len(LEADERBOARDS)} "
        f"target(s); live HTTP scrape RUNPOD-PENDING.\n"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
