#!/usr/bin/env python3
"""quorum_triage.py - P1.4 blind free-model quorum cost-gate for the opus walk.

Before the supervisor pays for an Opus boot, poll a small roster of FREE devin-bridge models
(swe-1-6 / kimi-k2-7 / glm-5-2) with the compact walk signals and one blind question: "is there
actionable work?". Each voter is an INDEPENDENT raw urllib POST to the bridge's /v1/messages and
is blind to the others (same prompt, no cross-talk). Majority-yes wakes the paid orchestrator.

This is a COST gate, not a correctness gate (orchestrator decision D-5): the deterministic
_should_spawn gate already proved there is real work + a live bridge; the quorum only adds a
cheap second opinion so a paid boot is skipped when the free models agree there is nothing to do.
Therefore it FAILS OPEN: if fewer than QUORUM_MIN_VOTES voters return a parseable YES/NO, the
verdict is wake=True ("QUORUM_DEGRADED") -- a degraded quorum must never suppress real work.

Stdlib only (urllib). No paid turn is ever spawned here.
"""
from __future__ import annotations

import json
import re
import time

# --- fixed wiring (named constants, no magic numbers downstream) -------------
# The blind voter roster. Each answers on the FREE devin bridge; kimi-k2-7 + glm-5-2 answer on
# POST /v1/messages even though they are absent from the bridge's /v1/models listing (grounded
# 2026-07-10 recon). swe-1-6 is listed. All three cost nothing.
QUORUM_ROSTER = ("swe-1-6", "kimi-k2-7", "glm-5-2")
# The free devin bridge's Anthropic-shaped message endpoint (same host the supervisor health-probes).
QUORUM_ENDPOINT = "http://127.0.0.1:8810/v1/messages"
# Per-voter HTTP timeout. A voter that does not answer in time simply does not vote.
VOTER_TIMEOUT_S = 30
# The vote is one token of signal; keep the completion tiny so it is cheap + fast.
VOTER_MAX_TOKENS = 64
# Fewer than this many parseable votes -> fail-open (wake). Quorum is cost, not correctness (D-5).
QUORUM_MIN_VOTES = 2
# The single blind question every voter answers (identical prompt = blind + independent).
QUORUM_QUESTION = ("Is there actionable work for the orchestrator? Answer strictly YES or NO "
                   "then one line naming the plan id.")


def _plan_done(plan_id: str, plan_status_dir: str) -> bool:
    """True when plan-status/{plan_id}.json exists with status == DONE (mirrors
    walk_state.is_plan_done; read-only, never raises)."""
    import os
    row_path = os.path.join(plan_status_dir, f"{plan_id}.json")
    if not os.path.isfile(row_path):
        return False
    try:
        with open(row_path, "r", encoding="utf-8") as fh:
            row = json.load(fh)
    except (OSError, json.JSONDecodeError):
        return False
    return isinstance(row, dict) and row.get("status") == "DONE"


def build_signals(state: dict, plan_status_dir: str) -> dict:
    """Compact the walk state into the signal bundle the voters see: each queued plan id + whether
    its plan-status row is already DONE, the cursor plan/stage, and the escalated (set-aside) ids.
    Small on purpose -- the voters decide on 'is there work', not on the full state."""
    queue = list(state.get("queue") or [])
    queue_states = [{"plan_id": pid, "done": _plan_done(pid, plan_status_dir)} for pid in queue]
    cursor = state.get("cursor") or {}
    escalated = [e.get("plan_id") for e in (state.get("escalation_queue") or []) if e.get("plan_id")]
    return {
        "queue": queue_states,
        "cursor": {"plan_id": cursor.get("plan_id"), "stage": cursor.get("stage")},
        "escalation_ids": escalated,
    }


def _voter_prompt(signals: dict) -> str:
    """The single prompt handed IDENTICALLY to every voter (blind + independent)."""
    return ("You are one blind voter deciding whether a paid orchestrator should wake for a "
            "backlog walk. You cannot see the other voters.\n"
            "Walk signals (JSON):\n"
            f"{json.dumps(signals, indent=2)}\n\n"
            f"{QUORUM_QUESTION}")


def _extract_text(body: dict) -> str:
    """Pull the assistant text out of a bridge /v1/messages Anthropic-shaped envelope. Handles the
    content-block list ([{type:text,text:..}]) plus a few fallbacks. Never raises."""
    if not isinstance(body, dict):
        return ""
    content = body.get("content")
    if isinstance(content, list):
        parts = [b.get("text", "") for b in content
                 if isinstance(b, dict) and b.get("type") == "text"]
        joined = "".join(parts).strip()
        if joined:
            return joined
    for key in ("text", "result", "message", "completion"):
        value = body.get(key)
        if isinstance(value, str) and value.strip():
            return value
    return ""


def default_post(model: str, prompt: str) -> dict:
    """One blind independent HTTP vote against ONE free model on the devin bridge. Returns
    {ok, text, status, error, latency_ms}. Never raises (honest-degrade at the boundary): any
    transport/HTTP error is ok=False, which simply means this voter did not vote."""
    import urllib.request
    import urllib.error
    payload = {"model": model, "max_tokens": VOTER_MAX_TOKENS,
               "messages": [{"role": "user", "content": prompt}]}
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(QUORUM_ENDPOINT, data=data,
                                 headers={"Content-Type": "application/json"})
    t0 = time.time()
    try:
        with urllib.request.urlopen(req, timeout=VOTER_TIMEOUT_S) as resp:
            body = json.loads(resp.read().decode("utf-8", errors="replace"))
    except urllib.error.HTTPError as exc:
        return {"ok": False, "text": "", "status": exc.code, "error": f"http {exc.code}",
                "latency_ms": int((time.time() - t0) * 1000)}
    except Exception as exc:  # noqa: BLE001 - any transport failure = this voter abstains
        return {"ok": False, "text": "", "status": None, "error": str(exc),
                "latency_ms": int((time.time() - t0) * 1000)}
    return {"ok": True, "text": _extract_text(body), "status": 200, "error": None,
            "latency_ms": int((time.time() - t0) * 1000)}


def _parse_vote(text: str) -> str | None:
    """Robust YES/NO parse: the first standalone yes|no token, case-insensitive. Anything else
    (hedge, empty, unparseable) returns None = no vote."""
    match = re.search(r"\b(yes|no)\b", text or "", re.IGNORECASE)
    return match.group(1).lower() if match else None


def vote(signals: dict, post_fn=default_post) -> tuple:
    """Poll the roster blind and return (wake: bool, detail: dict). Majority of RETURNED votes
    decides (wake on a tie -- fail-open, cost gate). Fewer than QUORUM_MIN_VOTES parseable votes
    -> fail-open wake with reason QUORUM_DEGRADED. post_fn is injectable so tests never touch the
    network; every voter is called with the SAME prompt (blind + independent)."""
    prompt = _voter_prompt(signals)
    voters = []
    yes = no = 0
    for model in QUORUM_ROSTER:
        res = post_fn(model, prompt)
        verdict = _parse_vote(res.get("text", "")) if res.get("ok") else None
        voters.append({"model": model, "verdict": verdict, "latency_ms": res.get("latency_ms"),
                       "status": res.get("status"), "error": res.get("error"),
                       "raw": (res.get("text") or "")[:200]})
        if verdict == "yes":
            yes += 1
        elif verdict == "no":
            no += 1
    returned = yes + no
    if returned < QUORUM_MIN_VOTES:
        return True, {"reason": "QUORUM_DEGRADED", "wake": True, "returned": returned,
                      "yes": yes, "no": no, "voters": voters}
    wake = yes >= no  # majority-yes wakes; a tie wakes (fail-open cost gate)
    return wake, {"reason": "QUORUM_WAKE" if wake else "QUORUM_QUIET", "wake": wake,
                  "returned": returned, "yes": yes, "no": no, "voters": voters}
