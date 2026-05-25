"""D-28: Adversarial example farming -- live-PPM Python loop.

Companion to the Rust host-side scaffolding at
``crates/orchestrator/src/farm/adv_rm.rs``. Walks a seed dataset, scores
each candidate-step chain via the LIVE PPM sidecar at
``http://127.0.0.1:8101/v1/ppm/score`` (port-forwarded from the remote
GPU host), applies the paper-grounded Z>1.96 dual-filter, and writes
``corpus.jsonl`` rows with ``gold_verdict: "pending"`` for downstream
gold-label verification.

Paper: Liu et al, "Adversarial Examples for Reward Models" (arXiv:2504.06141).

Recalibration (D-28-decisions.md DECISION-3):
  The plan-doc UNGROUNDED "+5pp" PRMBench acceptance gate is REJECTED.
  This module operates against a +1pp recalibrated floor derived from
  the paper's actually-reported +0.7pp overall RewardBench delta
  rounded UP to integer-pp to stay conservative without overclaiming.
  +1pp is the floor used by the downstream Rust `bench_prmbench` v1/v2
  delta comparator -- NOT the +5pp in the plan text.

K>=2 ensemble fallback (DECISION-5):
  Paper requires K>=2 RM ensemble for the inter-model std driving the
  Z>1.96 filter. ELAI has K=1 (Qwen2.5-Math-PRM-7B singleton). Fallback
  uses the PPM response's ``score_var`` (per-step variance) as the K=1
  stand-in for K>=2 inter-model std -- a grounded hypothesis from D-4's
  rm-overoptimization proxy>>gold gap pattern, not a paper-direct claim.

# RUNPOD-PENDING: bash workflow/remote-setup/start-ppm-sidecar.sh

The live PPM sidecar boot + port-forward + corpus upload pipeline is
operator-driven via the workflow/remote-setup/ scripts. This script
defaults to ``--mock`` (no network calls) for CI smoke; ``--live``
mode requires the remote PPM sidecar reachable on 127.0.0.1:8101.

Deps: stdlib only (urllib + json) + orjson for the JSONL write path
(orjson is already in pyproject.toml). NO ``requests`` import -- the
sidecar uses FastAPI on a localhost port; urllib.request is sufficient
and avoids adding a runtime dep that's not in pyproject.

Caps (HARD, mirrors Rust runner):
  --max-samples N : default 1000, hard cap 1500 (paper §5 saturation)
  --max-rounds N  : default 2,    hard cap 2    (paper §5 round-3 plateau)
  --z-threshold F : default 1.96  (paper §3.2 95th-pct uncertainty cutoff)
"""

from __future__ import annotations

import argparse
import json
import math
import os
import pathlib
import sys
import time
import urllib.error
import urllib.request
from typing import Any

import orjson

# ---------------------------------------------------------------------------
# Constants (paper-grounded; mirror Rust runner adv_rm.rs)
# ---------------------------------------------------------------------------

DEFAULT_ENDPOINT: str = "http://127.0.0.1:8101/v1/ppm/score"
"""Default PPM sidecar URL. Port 8101 is reserved for the live PPM
sidecar on the remote GPU host (see CLAUDE.md 'Remote sidecar (canonical)')."""

DEFAULT_MAX_SAMPLES: int = 1000
"""Paper §5 sweet spot. >~1000 hurts performance."""

MAX_SAMPLES_CAP: int = 1500
"""Hard cap -- runner refuses values above this."""

DEFAULT_MAX_ROUNDS: int = 2
"""Paper §5 round-3 plateau. Hard ceiling at 2."""

MAX_ROUNDS_CAP: int = 2
"""Hard cap -- runner refuses values above this."""

DEFAULT_Z_THRESHOLD: float = 1.96
"""Paper §3.2: 95th-pct two-sided normal CDF. Z>1.96 is the high-uncertainty
cutoff for ensemble-std filtering. ELAI K=1 fallback: PPM `score_var` stands
in for ensemble std (D-4 proxy>>gold gap pattern)."""

DEFAULT_VERIFIER_ID: str = "ppm_qwen25_math_prm_7b"

# ---------------------------------------------------------------------------
# Wire types (mirror PpmScoreRequest / PpmScoreResponse in Rust)
# ---------------------------------------------------------------------------


def build_score_request(
    seed_id: str,
    problem_statement: str,
    reasoning_steps: list[str],
    mode: str,
    round_idx: int,
) -> dict[str, Any]:
    """Construct the wire body for one /v1/ppm/score POST.

    Matches ``PpmScoreRequest`` in
    ``crates/orchestrator/src/sidecar_ppm_hook.rs``.
    """
    return {
        "candidate_id": f"seed:{seed_id}|round:{round_idx}",
        "verifier_id": DEFAULT_VERIFIER_ID,
        "problem_statement": problem_statement,
        "reasoning_steps": reasoning_steps,
        "mode": mode,
    }


def score_one(endpoint: str, body: dict[str, Any], timeout_s: float) -> dict[str, Any] | None:
    """POST one request to the PPM sidecar. Returns parsed JSON on 200,
    None on any transport / status error (the caller skips the seed --
    anti-mock-data: no synthetic fallback scores)."""
    raw = orjson.dumps(body)
    req = urllib.request.Request(
        endpoint,
        data=raw,
        method="POST",
        headers={"Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout_s) as resp:
            if resp.status != 200:
                return None
            payload = resp.read()
            return orjson.loads(payload)
    except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError, OSError) as exc:
        sys.stderr.write(
            f"ppm_adversarial: score_one transport error for {body.get('candidate_id')}: {exc}\n"
        )
        return None


def z_score_from_var(mean: float, var: float) -> float:
    """K=1 fallback: Z = (mean - 0.5) / sqrt(var); returns 0.0 when
    std == 0 to avoid div-by-zero. Mirrors Rust ``z_score_from_var``."""
    if var <= 0.0:
        return 0.0
    std = math.sqrt(var)
    if std <= 0.0:
        return 0.0
    return (mean - 0.5) / std


# ---------------------------------------------------------------------------
# Mock hook (CI default; no network)
# ---------------------------------------------------------------------------


def mock_score(body: dict[str, Any]) -> dict[str, Any]:
    """Deterministic mock used in --mock mode. Forced mean=0.9 + small
    var so the Z>1.96 filter consistently passes; lets CI exercise the
    write path without the live sidecar.

    Anti-mock-data note: this is NOT used in --live mode. The output
    rows still carry ``gold_verdict: "pending"`` so the downstream
    verifier knows not to trust them as gold labels."""
    steps = body.get("reasoning_steps") or []
    n = max(1, len(steps))
    step_scores = [0.9] * n
    return {
        "candidate_id": body.get("candidate_id", ""),
        "verifier_id": body.get("verifier_id", DEFAULT_VERIFIER_ID),
        "mode": body.get("mode", "math"),
        "step_scores": step_scores,
        "score_mean": 0.9,
        "score_var": 0.04,
        "num_steps": n,
        "latency_ms": 0,
    }


# ---------------------------------------------------------------------------
# Main farm loop
# ---------------------------------------------------------------------------


def load_seeds(dataset_path: pathlib.Path) -> list[dict[str, Any]]:
    """Load JSONL seed rows. Empty lines skipped. Malformed lines raise."""
    raw = dataset_path.read_bytes()
    out: list[dict[str, Any]] = []
    for i, line in enumerate(raw.splitlines()):
        stripped = line.strip()
        if not stripped:
            continue
        try:
            obj = orjson.loads(stripped)
        except orjson.JSONDecodeError as exc:
            raise ValueError(
                f"malformed seed at line {i} of {dataset_path}: {exc}"
            ) from exc
        out.append(obj)
    return out


def farm(
    *,
    endpoint: str,
    dataset_path: pathlib.Path,
    corpus_dir: pathlib.Path,
    max_samples: int,
    max_rounds: int,
    z_threshold: float,
    timeout_s: float,
    live: bool,
) -> dict[str, Any]:
    """Walk the seed dataset, apply the Z>1.96 filter, write
    corpus.jsonl. Returns a summary dict mirroring the Rust report
    shape (samples_farmed, rounds, corpus_path)."""
    if max_samples <= 0 or max_samples > MAX_SAMPLES_CAP:
        raise ValueError(f"max_samples {max_samples} out of [1, {MAX_SAMPLES_CAP}]")
    if max_rounds <= 0 or max_rounds > MAX_ROUNDS_CAP:
        raise ValueError(f"max_rounds {max_rounds} out of [1, {MAX_ROUNDS_CAP}]")

    seeds = load_seeds(dataset_path)
    if not seeds:
        raise ValueError(f"empty dataset at {dataset_path}")

    selected: list[dict[str, Any]] = []
    rounds_executed = 0
    ts_ms = int(time.time() * 1000)
    out_dir = corpus_dir / str(ts_ms)
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / "corpus.jsonl"

    with out_path.open("wb") as fh:
        for round_idx in range(1, max_rounds + 1):
            rounds_executed = round_idx
            for seed in seeds:
                if len(selected) >= max_samples:
                    break
                seed_id = str(seed.get("seed_id", ""))
                problem = str(seed.get("problem_statement", ""))
                steps = list(seed.get("reasoning_steps") or [])
                mode = str(seed.get("mode", "math"))
                provenance = seed.get("provenance") or {"type": "canary_corpus"}

                body = build_score_request(seed_id, problem, steps, mode, round_idx)
                if live:
                    resp = score_one(endpoint, body, timeout_s)
                    if resp is None:
                        continue
                else:
                    resp = mock_score(body)

                mean = float(resp.get("score_mean", 0.0))
                var = float(resp.get("score_var", 0.0))
                z = z_score_from_var(mean, var)
                if z <= z_threshold:
                    continue

                # Provenance tag matches CorpusProvenance::tag() in Rust.
                if isinstance(provenance, dict):
                    prov_tag = str(provenance.get("type", "canary_corpus"))
                else:
                    prov_tag = "canary_corpus"

                row = {
                    "candidate_id": str(resp.get("candidate_id", body["candidate_id"])),
                    "seed_id": seed_id,
                    "problem_statement": problem,
                    "reasoning_steps": steps,
                    "ppm_step_scores": list(resp.get("step_scores") or []),
                    "ppm_score_mean": mean,
                    "ppm_score_var": var,
                    "z_score": z,
                    # HARD: anti-mock-data. Downstream gold verifier
                    # flips this to "pass" / "fail".
                    "gold_verdict": "pending",
                    "provenance_tag": prov_tag,
                    "round": round_idx,
                    "ts_ms": ts_ms,
                }
                fh.write(orjson.dumps(row))
                fh.write(b"\n")
                selected.append(row)
            if len(selected) >= max_samples:
                break

    return {
        "samples_farmed": len(selected),
        "rounds": rounds_executed,
        "corpus_path": str(out_path),
        "z_threshold": z_threshold,
        "live": live,
    }


def main() -> int:
    p = argparse.ArgumentParser(description="D-28 adversarial example farming loop")
    p.add_argument("--ppm-endpoint", default=DEFAULT_ENDPOINT)
    p.add_argument("--dataset", required=True)
    p.add_argument("--corpus-dir", default=".elai_cc/benchmarks/adv_rm/corpus")
    p.add_argument("--max-samples", type=int, default=DEFAULT_MAX_SAMPLES)
    p.add_argument("--max-rounds", type=int, default=DEFAULT_MAX_ROUNDS)
    p.add_argument("--z-threshold", type=float, default=DEFAULT_Z_THRESHOLD)
    p.add_argument("--timeout-s", type=float, default=30.0)
    mode = p.add_mutually_exclusive_group()
    mode.add_argument(
        "--live",
        action="store_true",
        help="Score against the live PPM sidecar. RUNPOD-PENDING -- needs port 8101 reachable.",
    )
    mode.add_argument(
        "--mock",
        action="store_true",
        help="Deterministic mock (no network). Default if neither flag given.",
    )
    args = p.parse_args()

    # Default to mock when neither was passed -- mirrors the Rust CLI's
    # default (CI smoke must not require the live sidecar).
    live = bool(args.live) and not args.mock
    if not live and not args.mock:
        sys.stderr.write(
            "ppm_adversarial: neither --live nor --mock passed; defaulting to --mock\n"
        )

    dataset_path = pathlib.Path(args.dataset)
    corpus_dir = pathlib.Path(args.corpus_dir)
    try:
        report = farm(
            endpoint=args.ppm_endpoint,
            dataset_path=dataset_path,
            corpus_dir=corpus_dir,
            max_samples=args.max_samples,
            max_rounds=args.max_rounds,
            z_threshold=args.z_threshold,
            timeout_s=args.timeout_s,
            live=live,
        )
    except (FileNotFoundError, ValueError) as exc:
        sys.stderr.write(f"ppm_adversarial: {exc}\n")
        return 1

    sys.stdout.write(json.dumps(report, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
