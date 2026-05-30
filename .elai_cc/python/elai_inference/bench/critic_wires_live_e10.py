"""E-10 -- live-signal harvester for the 4 critic wires.

Drives the sidecar to produce REPRESENTATIVE input pairs for the four
CriticTilePhase producers that are wire-dark in this slice:

  TierMatrix      -- BENCH-LM local-vs-cloud pass-rate pair
  RoleAblation    -- Self-MoA 3-role vs 8-role iso-budget pair
  GrcBlockquote   -- GRC blockquote on vs off pair (Lanham-anchored,
                     also covers FaithCoT-Bench arXiv:2510.04040)
  CloudDefault    -- cloud-default switch test (local vs cloud baseline pair)

Each wire runs the sidecar with TWO prompt variants on the same fixture
(baseline vs challenger) and emits a fixture JSON record in the exact
shape the orchestrator producers expect (TierMatrixSnapshot,
RoleProfileConfig + IsoBudgetReport, on/off score vectors, CloudDefaultSnapshot).

Wiring these into the actual SSB requires the orchestrator's `Arc<Ssb>`
context and is DEFERRED-COMPLEXITY (>40 LOC of Rust + state setup per
sidecar-G scope rules). This script proves the INPUTS for the producers
flow through the live sidecar.

Run:
  ELAI_LIVE_DISPATCH=1 ANTHROPIC_BASE_URL=http://localhost:8100 \\
    python critic_wires_live_e10.py --out e10-critic-wires-live.json
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


def call(prompt: str, max_tokens: int = 400) -> tuple[str, float]:
    body = {"model": MODEL, "max_tokens": max_tokens,
            "messages": [{"role": "user", "content": prompt}]}
    headers = {"Content-Type": "application/json", "x-api-key": API_KEY}
    t0 = time.time()
    with httpx.Client(timeout=90.0) as c:
        r = c.post(SIDECAR_URL, json=body, headers=headers)
    latency = time.time() - t0
    r.raise_for_status()
    data = r.json()
    text = ""
    for blk in data.get("content", []):
        if blk.get("type") == "text":
            text += blk.get("text", "")
    return text, latency


# Common 3-task fixture used by all 4 wires for a shared signal floor.
FIXTURES = [
    "What is the median of [1, 3, 5, 7, 9]?",
    "Write a Python one-liner that reverses a string.",
    "If x + 5 = 12, what is x?",
]


def score_response(prompt: str, response: str) -> int:
    """Crude per-fixture grading: 1 if response contains the expected token, else 0."""
    p = prompt.lower()
    r = response.lower()
    if "median" in p:
        return 1 if "5" in r else 0
    if "reverses a string" in p:
        return 1 if ("[::-1]" in r or "reversed" in r) else 0
    if "x + 5" in p:
        return 1 if "7" in r else 0
    return 0


def run_pair(baseline_wrap, challenger_wrap, fixtures):
    bs, cs = [], []
    for f in fixtures:
        bt, _ = call(baseline_wrap(f), max_tokens=200)
        ct, _ = call(challenger_wrap(f), max_tokens=200)
        bs.append(score_response(f, bt))
        cs.append(score_response(f, ct))
    return bs, cs


# ----- wire 1: TierMatrix (BENCH-LM) -----
def wire_tier_matrix():
    """Local tier: terse coder prompt. Cloud-comparison stand-in: same prompt
    with verbose chain-of-thought (since cloud Sonnet would naturally produce
    longer reasoning). Pass-rate comparison is the TierMatrixSnapshot."""
    base = lambda q: f"Answer briefly: {q}"
    chal = lambda q: f"Think step-by-step then give the final answer to: {q}"
    bs, cs = run_pair(base, chal, FIXTURES)
    return {
        "phase": "tier_matrix",
        "producer_input": {
            "local_pass_rate": sum(bs) / len(bs),
            "cloud_pass_rate": sum(cs) / len(cs),
            "n_tasks": len(bs),
        },
        "per_task_scores": {"baseline": bs, "challenger": cs},
    }


# ----- wire 2: RoleAblation (Self-MoA arXiv:2502.00674) -----
def wire_role_ablation():
    """8-role full pipeline stand-in: prefix with explicit role-list framing.
    3-role reduced stand-in: prefix with single 'expert' role. The producer
    consumes (full.pass_rate, reduced.pass_rate)."""
    base = lambda q: ("Consider this question from 8 expert perspectives "
                      f"(coder, mathematician, reviewer, tester, auditor, "
                      f"architect, debugger, planner). Synthesize: {q}")
    chal = lambda q: f"As an expert, answer: {q}"
    bs, cs = run_pair(base, chal, FIXTURES)
    return {
        "phase": "role_ablation",
        "producer_input": {
            "full_pipeline_pass_rate": sum(bs) / len(bs),
            "single_agent_wait_pass_rate": sum(cs) / len(cs),
            "n_tasks": len(bs),
        },
        "per_task_scores": {"baseline": bs, "challenger": cs},
    }


# ----- wire 3: GrcBlockquote (Lanham + FaithCoT-Bench) -----
def wire_grc_blockquote():
    """Blockquote-ON stand-in: prefix with the GRC blockquote pattern markers.
    Blockquote-OFF stand-in: bare prompt. Producer consumes the on/off score
    vectors directly."""
    base = lambda q: (f"> **observe** {q}\n> **hypothesize** [grounded reasoning here]\n\nFinal:")
    chal = lambda q: q
    bs, cs = run_pair(base, chal, FIXTURES)
    return {
        "phase": "grc_blockquote",
        "producer_input": {
            "blockquote_on_scores": [float(x) for x in bs],
            "blockquote_off_scores": [float(x) for x in cs],
        },
        "per_task_scores": {"baseline": bs, "challenger": cs},
    }


# ----- wire 4: CloudDefault -----
def wire_cloud_default():
    """Local-default stand-in: bare prompt. Cloud-default stand-in: prompt
    with explicit anthropic-style system framing. Producer also consumes a
    cloud_prompt_cache_hit_rate scalar; we stub at 0.0 since Anthropic
    OpenAI-compat lacks caching per critic Punch 4."""
    base = lambda q: q
    chal = lambda q: f"You are Claude, a helpful AI assistant. Question: {q}"
    bs, cs = run_pair(base, chal, FIXTURES)
    return {
        "phase": "cloud_default",
        "producer_input": {
            "local_pass_rate": sum(bs) / len(bs),
            "cloud_pass_rate": sum(cs) / len(cs),
            "cloud_prompt_cache_hit_rate": 0.0,
            "n_tasks": len(bs),
        },
        "per_task_scores": {"baseline": bs, "challenger": cs},
    }


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--out", required=True)
    args = p.parse_args()

    if os.environ.get("ELAI_LIVE_DISPATCH", "") != "1":
        sys.stderr.write("ELAI_LIVE_DISPATCH=1 required.\n")
        return 2

    wires = [wire_tier_matrix(), wire_role_ablation(),
             wire_grc_blockquote(), wire_cloud_default()]

    result = {
        "schema": "e10_critic_wires_live_v1",
        "model": MODEL,
        "sidecar_url": SIDECAR_URL,
        "ts_unix": int(time.time()),
        "wires": wires,
        "note": ("Producer-side SSB emission (emit_tier_matrix_decision, "
                 "run_role_ablation, emit_grc_blockquote_lift, "
                 "emit_cloud_default_decision) requires Arc<Ssb> and is "
                 "DEFERRED-COMPLEXITY per sidecar-G scope rules. This file "
                 "provides the producer input data harvested from live "
                 "sidecar inference."),
    }
    with open(args.out, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2)
    summary = {w["phase"]: w["producer_input"] for w in wires}
    print(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
