"""E-9/E-49 -- minimal LIVE-DISPATCH runner for SWE-Bench Pro + RefineBench fixtures.

This is the minimum viable path that PROVES the dispatch chain works:
fixture -> prompt-wrap -> POST /v1/messages -> capture response -> score
(score is patch-emitted-y/n, not full SWE-Bench harness verdict).

The legacy runners in this same dir (swe_bench_pro.py, refinebench.py) are
intentionally scaffolding-only with a hard `ensure_env_gate` abort. This
file is the companion that ACTUALLY DISPATCHES against the live sidecar
when ELAI_LIVE_DISPATCH=1.

Run with:
  ELAI_LIVE_DISPATCH=1 python swe_bench_pro_live_dispatch.py --suite swe-bench-pro --n-tasks 2
  ELAI_LIVE_DISPATCH=1 python swe_bench_pro_live_dispatch.py --suite refinebench --n-tasks 1
"""
from __future__ import annotations

import argparse
import json
import os
import sys
import time

import httpx


SIDECAR_URL = os.environ.get("ANTHROPIC_BASE_URL", "http://localhost:8100") + "/v1/messages"
API_KEY = os.environ.get("ANTHROPIC_API_KEY", "local-dummy")
MODEL = "qwen"


# Tiny in-file fixture set: SWE-Bench Pro-shaped issues (1 trivial + 1 medium)
# and 1 RefineBench-shaped multi-iteration refinement task. Real fixtures live
# on the remote pod under ~/swe-bench-pro/ -- these stand-ins prove the wire.
FIXTURES_SWE_BENCH_PRO = [
    {
        "instance_id": "demo-001-add-fn",
        "repo": "synthetic/util-math",
        "problem_statement": (
            "Add a function `def median(xs: list[float]) -> float` to math_util.py. "
            "Empty list should raise ValueError. Return the middle element for odd "
            "lengths, average of the two middle for even lengths. Only edit math_util.py."
        ),
        "gold_patch_marker": "def median",
    },
    {
        "instance_id": "demo-002-fix-bug",
        "repo": "synthetic/cache",
        "problem_statement": (
            "Function `lru_get(key, store, max_size)` in cache.py has an off-by-one "
            "error: it evicts AFTER inserting, so the store can briefly hold "
            "max_size+1 entries. Fix the eviction to happen BEFORE insert. Only edit cache.py."
        ),
        "gold_patch_marker": "lru_get",
    },
]

FIXTURES_REFINEBENCH = [
    {
        "task_id": "rb-demo-001",
        "domain": "code",
        "initial_prompt": (
            "Write a Python function `is_palindrome(s: str) -> bool` that returns True "
            "if s reads the same forwards and backwards, ignoring case and non-alphanumerics."
        ),
        "refinement_feedback": (
            "Your previous solution missed unicode normalization. Add NFKC normalization "
            "via unicodedata before comparison."
        ),
    },
]


def call_sidecar(prompt: str, max_tokens: int = 800, timeout_s: float = 120.0) -> tuple[str, float, dict]:
    """Returns (response_text, latency_s, raw_json_meta)."""
    body = {
        "model": MODEL,
        "max_tokens": max_tokens,
        "messages": [{"role": "user", "content": prompt}],
    }
    headers = {"Content-Type": "application/json", "x-api-key": API_KEY}
    t0 = time.time()
    with httpx.Client(timeout=timeout_s) as c:
        r = c.post(SIDECAR_URL, json=body, headers=headers)
    latency = time.time() - t0
    r.raise_for_status()
    data = r.json()
    text = ""
    for block in data.get("content", []):
        if block.get("type") == "text":
            text += block.get("text", "")
    return text, latency, data


def wrap_swe_bench_pro(item: dict) -> str:
    return (
        "You are an expert software engineer. Below is a GitHub issue. "
        "Produce a unified diff patch (git format) that fixes the issue. "
        "Wrap the patch in ```diff ... ``` fences.\n\n"
        f"Repo: {item['repo']}\n"
        f"Issue (instance {item['instance_id']}):\n{item['problem_statement']}\n\n"
        "Output ONLY the unified diff."
    )


def wrap_refinebench(item: dict) -> str:
    return (
        f"Domain: {item['domain']}. Initial task:\n{item['initial_prompt']}\n\n"
        f"Refinement feedback from prior turn:\n{item['refinement_feedback']}\n\n"
        "Produce the refined solution as a single self-contained code block."
    )


def score_patch_emitted(response: str, item: dict) -> int:
    """1 if response contains a diff fence AND the gold marker, else 0."""
    has_diff_fence = "```diff" in response or "diff --git" in response or "@@" in response
    has_marker = item.get("gold_patch_marker", "") in response
    return 1 if (has_diff_fence and has_marker) else 0


def score_refined_emitted(response: str) -> int:
    """1 if response contains a fenced code block, else 0."""
    return 1 if "```" in response else 0


def run_suite(suite: str, n_tasks: int) -> dict:
    if suite == "swe-bench-pro":
        fixtures = FIXTURES_SWE_BENCH_PRO[:n_tasks]
        wrap = wrap_swe_bench_pro
        scorer = lambda r, i: score_patch_emitted(r, i)
    elif suite == "refinebench":
        fixtures = FIXTURES_REFINEBENCH[:n_tasks]
        wrap = wrap_refinebench
        scorer = lambda r, _i: score_refined_emitted(r)
    else:
        raise SystemExit(f"unknown suite: {suite}")

    results = []
    for item in fixtures:
        prompt = wrap(item)
        try:
            text, latency, meta = call_sidecar(prompt, max_tokens=800)
            score = scorer(text, item)
            results.append({
                "id": item.get("instance_id") or item.get("task_id"),
                "score": score,
                "latency_s": round(latency, 2),
                "response_chars": len(text),
                "preview": text[:200].replace("\n", " "),
            })
        except Exception as exc:
            results.append({
                "id": item.get("instance_id") or item.get("task_id"),
                "score": 0,
                "error": str(exc),
            })

    n = len(results)
    score_sum = sum(r.get("score", 0) for r in results)
    return {
        "suite": suite,
        "n": n,
        "passed": score_sum,
        "pass_rate": (score_sum / n) if n else 0.0,
        "items": results,
        "model": MODEL,
        "sidecar_url": SIDECAR_URL,
        "ts_unix": int(time.time()),
    }


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--suite", default="swe-bench-pro", choices=["swe-bench-pro", "refinebench"])
    p.add_argument("--n-tasks", type=int, default=2)
    p.add_argument("--out", default=None, help="path to write JSON result")
    args = p.parse_args()

    if os.environ.get("ELAI_LIVE_DISPATCH", "") != "1":
        sys.stderr.write("ELAI_LIVE_DISPATCH=1 required.\n")
        return 2

    result = run_suite(args.suite, args.n_tasks)
    out = json.dumps(result, indent=2)
    print(out)
    if args.out:
        with open(args.out, "w", encoding="utf-8") as f:
            f.write(out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
