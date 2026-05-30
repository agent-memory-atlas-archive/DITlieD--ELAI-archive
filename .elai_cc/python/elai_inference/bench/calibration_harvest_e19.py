"""E-19 -- 200-trace L2 calibration harvest (subset 30) via live sidecar.

Purpose: drive the sidecar with 30 short reasoning prompts that span easy /
medium / hard difficulty, capture response text + latency + sidecar metadata
(when present), and emit a single JSON file of harvested traces conforming
to the schema in .elai_cc/eval/reasoning/traces.toml. This is the live
counterpart to the 5 seed rows that ship in traces.toml.

The full 200-trace replay loop lives on the remote pod under
~/elai_inference/bench/ in a follow-up; this subset proves the harvest path.

Run:
  ELAI_LIVE_DISPATCH=1 ANTHROPIC_BASE_URL=http://localhost:8100 \\
    python calibration_harvest_e19.py --n 30 --out e19-calibration-30trace.json

Output schema (per trace):
  {
    "id": "trace-NNN",
    "prompt_class": "arithmetic" | "lookup" | "logic" | "code",
    "difficulty": "easy" | "medium" | "hard",
    "response_chars": int,
    "latency_s": float,
    "stop_reason": str,
    "input_tokens": int (when sidecar returns usage),
    "output_tokens": int (when sidecar returns usage),
    "preview": first 160 chars of response,
  }

Aggregate stats per difficulty bucket: mean latency, mean output_tokens,
mean response_chars. These feed the L2-threshold sweep in calibration.toml
on a future pass.
"""
from __future__ import annotations

import argparse
import json
import os
import statistics
import sys
import time

import httpx


SIDECAR_URL = os.environ.get("ANTHROPIC_BASE_URL", "http://localhost:8100") + "/v1/messages"
API_KEY = os.environ.get("ANTHROPIC_API_KEY", "local-dummy")
MODEL = "qwen"


# 30 prompts: 10 easy / 10 medium / 10 hard. Selected to vary the entropy
# of the next-token distribution at L1 (the gate signal the calibration
# harvest is supposed to characterize).
PROMPTS = [
    # easy (low entropy expected) -- single deterministic answer
    ("p-001", "arithmetic", "easy", "What is 2 + 2?"),
    ("p-002", "arithmetic", "easy", "What is 10 - 7?"),
    ("p-003", "lookup", "easy", "What is the capital of France?"),
    ("p-004", "lookup", "easy", "What language is Rust compiled to (LLVM IR target)?"),
    ("p-005", "arithmetic", "easy", "What is 5 * 6?"),
    ("p-006", "lookup", "easy", "Name the largest planet in our solar system."),
    ("p-007", "arithmetic", "easy", "Round 3.7 to the nearest integer."),
    ("p-008", "lookup", "easy", "What does HTTP stand for?"),
    ("p-009", "arithmetic", "easy", "What is 100 / 4?"),
    ("p-010", "lookup", "easy", "What is the SI unit of force?"),
    # medium (moderate entropy expected)
    ("p-011", "logic", "medium", "If all bloops are razzies and all razzies are lazzies, are all bloops definitely lazzies?"),
    ("p-012", "code", "medium", "Write a one-liner Python list comprehension that yields squares of 0..9."),
    ("p-013", "arithmetic", "medium", "Solve for x: 3x + 7 = 22."),
    ("p-014", "logic", "medium", "If today is Wednesday, what day will it be 100 days from now?"),
    ("p-015", "code", "medium", "Write the shortest valid Python that prints 'hello'."),
    ("p-016", "arithmetic", "medium", "What is 7 factorial?"),
    ("p-017", "logic", "medium", "A train leaves at 9 AM going 60 mph. Another at 10 AM going 80 mph. When does the second catch the first?"),
    ("p-018", "code", "medium", "What does `[*range(3)]` evaluate to in Python?"),
    ("p-019", "arithmetic", "medium", "What is the prime factorization of 84?"),
    ("p-020", "logic", "medium", "If A implies B and B implies C, does A imply C?"),
    # hard (high entropy expected -- multi-step, open-ended)
    ("p-021", "code", "hard", "Implement a Python function `lis(xs)` returning the length of the longest increasing subsequence. O(n log n)."),
    ("p-022", "logic", "hard", "Three switches control three bulbs in another room. You can flip switches as you like but only enter the bulb room once. How do you identify each switch?"),
    ("p-023", "arithmetic", "hard", "Prove that the sum of the first n odd numbers equals n^2."),
    ("p-024", "code", "hard", "Write a Rust function that returns the median of a Vec<f64> without using external crates."),
    ("p-025", "logic", "hard", "If two trains 200km apart approach each other at 50 km/h and 30 km/h while a fly bounces between them at 100 km/h, how far does the fly travel?"),
    ("p-026", "code", "hard", "Explain the difference between Send and Sync in Rust with a one-line example for each."),
    ("p-027", "arithmetic", "hard", "Find all integer solutions to x^2 + y^2 = 25."),
    ("p-028", "logic", "hard", "A father is three times as old as his son. In 12 years he will be twice as old. How old are they now?"),
    ("p-029", "code", "hard", "Write a Python decorator that memoizes a unary function using only the standard library."),
    ("p-030", "arithmetic", "hard", "Compute the integral of x^2 from 0 to 3 by hand and verify with the power rule."),
]


def call_sidecar(prompt: str, max_tokens: int = 256, timeout_s: float = 90.0) -> dict:
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
    usage = data.get("usage", {}) or {}
    return {
        "text": text,
        "latency_s": latency,
        "stop_reason": data.get("stop_reason", ""),
        "input_tokens": usage.get("input_tokens", 0),
        "output_tokens": usage.get("output_tokens", 0),
    }


def harvest(n: int) -> dict:
    selected = PROMPTS[:n]
    traces = []
    for trace_id, klass, diff, prompt in selected:
        try:
            out = call_sidecar(prompt, max_tokens=256)
            traces.append({
                "id": trace_id,
                "prompt_class": klass,
                "difficulty": diff,
                "response_chars": len(out["text"]),
                "latency_s": round(out["latency_s"], 3),
                "stop_reason": out["stop_reason"],
                "input_tokens": out["input_tokens"],
                "output_tokens": out["output_tokens"],
                "preview": out["text"][:160].replace("\n", " "),
            })
        except Exception as exc:
            traces.append({
                "id": trace_id,
                "prompt_class": klass,
                "difficulty": diff,
                "error": str(exc),
            })

    # Aggregate per difficulty
    agg = {}
    for diff in ("easy", "medium", "hard"):
        items = [t for t in traces if t.get("difficulty") == diff and "error" not in t]
        if not items:
            continue
        agg[diff] = {
            "n": len(items),
            "mean_latency_s": round(statistics.mean(t["latency_s"] for t in items), 3),
            "mean_output_tokens": round(statistics.mean(t["output_tokens"] for t in items), 1),
            "mean_response_chars": round(statistics.mean(t["response_chars"] for t in items), 1),
        }

    return {
        "schema_version": 1,
        "subset_of_total": 200,
        "harvested": len(traces),
        "model": MODEL,
        "sidecar_url": SIDECAR_URL,
        "aggregate_by_difficulty": agg,
        "traces": traces,
        "ts_unix": int(time.time()),
    }


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--n", type=int, default=30)
    p.add_argument("--out", required=True)
    args = p.parse_args()

    if os.environ.get("ELAI_LIVE_DISPATCH", "") != "1":
        sys.stderr.write("ELAI_LIVE_DISPATCH=1 required.\n")
        return 2

    result = harvest(args.n)
    with open(args.out, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2)
    # Echo summary to stdout
    summary = {
        "harvested": result["harvested"],
        "aggregate_by_difficulty": result["aggregate_by_difficulty"],
    }
    print(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
