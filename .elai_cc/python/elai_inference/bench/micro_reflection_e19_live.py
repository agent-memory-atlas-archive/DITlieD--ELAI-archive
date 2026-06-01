#!/usr/bin/env python3
"""E-19 micro-reflection emitter LIVE test against running sidecar.

Mechanism (per arxiv:2502.18581 self-certainty + E-19 plan):
- L1 confidence proxy = (mean_logprob, entropy_mean) from gg_signals
- L1 gate: skip L2 if mean_logprob > -0.3 AND entropy_mean < 0.6 (high self-certainty)
- L2 micro-reflection fires only when L1 self-certainty is BELOW the gate

This bench dispatches 6 prompts (3 easy/3 hard), captures gg_signals, applies the
gate, and for L2-fired cases issues a critique prompt asking the model to assess
its first answer.

Output: JSON list of per-task records.
"""
from __future__ import annotations
import json
import sys
import time
import urllib.request
import urllib.error

SIDECAR = "http://127.0.0.1:8100/v1/messages"
MODEL = "qwen"
TIMEOUT_S = 60

EASY = [
    "What is 2 + 2? Answer with just the number.",
    "What color is the sky on a clear sunny day? One word.",
    "Is water wet? Reply yes or no.",
]
HARD = [
    "Prove that the sum of the angles in a non-Euclidean spherical triangle exceeds 180 degrees, citing the Gauss-Bonnet theorem in your derivation.",
    "Explain why the Riemann hypothesis remains unproven and identify the deepest analytic obstruction encountered by recent attempts using Levinson's method.",
    "Derive the dispersion relation for a tachyonic Klein-Gordon field and explain why this violates microcausality at the level of the Wightman two-point function.",
]

# L1 gate (per self-certainty proxy, conservative): high confidence skips L2.
GATE_LOGPROB = -0.3
GATE_ENTROPY = 0.6


def post(payload: dict, timeout_s: int = TIMEOUT_S) -> dict:
    body = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        SIDECAR,
        data=body,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=timeout_s) as resp:
        return json.loads(resp.read().decode("utf-8"))


def extract_text(resp: dict) -> str:
    out = []
    for blk in resp.get("content", []):
        if blk.get("type") == "text":
            out.append(blk.get("text", ""))
    return "".join(out).strip()


def l1_signals(resp: dict) -> dict:
    meta = resp.get("meta", {}) or {}
    sigs = meta.get("gg_signals", {}) or {}
    return {
        "mean_logprob": sigs.get("mean_logprob"),
        "min_logprob": sigs.get("min_logprob"),
        "entropy_mean": sigs.get("entropy_mean"),
        "entropy_max": sigs.get("entropy_max"),
    }


def l1_gate_skips_l2(sigs: dict) -> bool:
    """High self-certainty short-circuits L2."""
    mlp = sigs.get("mean_logprob")
    ent = sigs.get("entropy_mean")
    if mlp is None or ent is None:
        return False
    return (mlp > GATE_LOGPROB) and (ent < GATE_ENTROPY)


def run_task(prompt: str, prompt_class: str) -> dict:
    rec = {
        "prompt_class": prompt_class,
        "prompt": prompt,
        "timestamp": time.time(),
    }
    t0 = time.time()
    try:
        r1 = post({
            "model": MODEL,
            "max_tokens": 220,
            "messages": [{"role": "user", "content": prompt}],
        })
        rec["answer"] = extract_text(r1)
        rec["l1_signals"] = l1_signals(r1)
        rec["t_answer_ms"] = int((time.time() - t0) * 1000)
    except Exception as exc:
        rec["error"] = f"l1_call_failed: {type(exc).__name__}: {exc}"
        rec["l1_signals"] = {}
        rec["l2_fired"] = False
        return rec

    skip_l2 = l1_gate_skips_l2(rec["l1_signals"])
    rec["l1_gate_skipped_l2"] = skip_l2
    rec["l2_fired"] = not skip_l2

    if rec["l2_fired"]:
        critique_prompt = (
            "Below is your previous answer to a question. Critique it for errors, "
            "missing reasoning steps, or unsupported claims. Be brief (3 sentences max). "
            "If the answer is fully correct, say so.\n\n"
            f"Original question: {prompt}\n"
            f"Your previous answer: {rec['answer']}\n\n"
            "Critique:"
        )
        t1 = time.time()
        try:
            r2 = post({
                "model": MODEL,
                "max_tokens": 200,
                "messages": [{"role": "user", "content": critique_prompt}],
            })
            rec["l2_output"] = extract_text(r2)
            rec["l2_signals"] = l1_signals(r2)
            rec["t_l2_ms"] = int((time.time() - t1) * 1000)
        except Exception as exc:
            rec["l2_error"] = f"l2_call_failed: {type(exc).__name__}: {exc}"
            rec["l2_output"] = None
    else:
        rec["l2_output"] = None
    return rec


def main() -> int:
    tasks = []
    for p in EASY:
        tasks.append(run_task(p, "easy"))
    for p in HARD:
        tasks.append(run_task(p, "hard"))

    fired_easy = sum(1 for t in tasks if t["prompt_class"] == "easy" and t.get("l2_fired"))
    fired_hard = sum(1 for t in tasks if t["prompt_class"] == "hard" and t.get("l2_fired"))

    summary = {
        "schema": "micro_reflection_e19_live.v1",
        "model": MODEL,
        "sidecar": SIDECAR,
        "gate": {"logprob_threshold": GATE_LOGPROB, "entropy_threshold": GATE_ENTROPY},
        "counts": {
            "easy_total": len(EASY),
            "hard_total": len(HARD),
            "easy_l2_fired": fired_easy,
            "hard_l2_fired": fired_hard,
            "expected_easy_l2_fired": 0,
            "expected_hard_l2_fired": 3,
        },
        "status": (
            "PASS" if (fired_easy == 0 and fired_hard >= 2)
            else ("PARTIAL" if fired_hard >= 1 else "FAIL")
        ),
        "tasks": tasks,
    }
    print(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
