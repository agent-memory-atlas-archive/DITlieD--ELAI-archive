#!/usr/bin/env python3
"""E-21 SBFL (Spectrum-Based Fault Localization) LIVE test against running sidecar.

Mechanism (per crates/orchestrator/src/peer_systems/sbfl.rs and AutoCodeRover arXiv:2404.05427):
- Ochiai score per file = e_fail(file) / sqrt(F_total * e_total(file))
- e_fail = failing tests touching the file
- e_total = passing+failing tests touching the file
- F_total = total failing tests in run

This bench:
- Constructs a synthetic 10-file repo trace
- Computes Ochiai per file
- Asks the sidecar to "pick the file most likely to contain the bug"
- Validates the model selects one of the top-2 SBFL-ranked files
"""
from __future__ import annotations
import json
import math
import re
import sys
import time
import urllib.request

SIDECAR = "http://127.0.0.1:8100/v1/messages"
MODEL = "qwen"

# Synthetic 10-file trace.
# Total tests: 10 passed + 5 failed.
# Files A,B = touched by both (and heavy on failing) -> high Ochiai
# Files C,D,E,F = touched by passing tests only -> Ochiai = 0
# Files G,H,I,J = touched by failing tests with varying density
F_TOTAL = 5  # total failing tests

# Per-file (e_total, e_fail)
TRACE = {
    "src/auth/login.rs":       (12, 5),  # touched by all 5 failing + 7 passing (suspect)
    "src/auth/token.rs":       (10, 4),  # 4 failing + 6 passing
    "src/util/log.rs":         (8, 0),   # passing only -> 0.0
    "src/util/fmt.rs":         (6, 0),   # passing only
    "src/db/conn.rs":          (4, 0),
    "src/db/migrate.rs":       (3, 0),
    "src/auth/hash.rs":        (7, 3),   # 3 fail + 4 pass
    "src/http/router.rs":      (5, 2),   # 2 fail + 3 pass
    "src/http/middleware.rs":  (5, 1),
    "src/cli/main.rs":         (2, 0),
}


def ochiai(e_total: int, e_fail: int, f_total: int) -> float:
    denom = f_total * e_total
    if denom == 0:
        return 0.0
    return e_fail / math.sqrt(denom)


def post(payload: dict, timeout_s: int = 60) -> dict:
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


def main() -> int:
    ranking = []
    for path, (e_total, e_fail) in TRACE.items():
        score = ochiai(e_total, e_fail, F_TOTAL)
        ranking.append({
            "path": path, "e_total": e_total, "e_fail": e_fail, "ochiai": score,
        })
    ranking.sort(key=lambda r: (-r["ochiai"], r["path"]))

    top2 = [r["path"] for r in ranking[:2]]

    # Build the prompt: present the ranking, ask for top-1 file selection.
    lines = ["Suspiciousness ranking (Ochiai SBFL, higher = more suspicious):"]
    for i, r in enumerate(ranking, 1):
        lines.append(
            f"  {i}. {r['path']}  ochiai={r['ochiai']:.4f}  "
            f"(fails={r['e_fail']}/{F_TOTAL}, total_tests_touching={r['e_total']})"
        )
    lines.append("")
    lines.append("Question: based on this SBFL ranking alone, which single file is "
                 "most likely to contain the bug? Reply with ONLY the file path "
                 "from the list, nothing else.")

    prompt = "\n".join(lines)

    t0 = time.time()
    try:
        resp = post({
            "model": MODEL,
            "max_tokens": 60,
            "messages": [{"role": "user", "content": prompt}],
        })
    except Exception as exc:
        print(json.dumps({
            "schema": "sbfl_e21_live.v1",
            "status": "FAIL",
            "error": f"{type(exc).__name__}: {exc}",
        }, indent=2))
        return 1

    answer = extract_text(resp)
    elapsed_ms = int((time.time() - t0) * 1000)

    # Parse: did the model name one of the top-2 paths?
    chosen = None
    for path in TRACE.keys():
        if path in answer:
            chosen = path
            break

    in_top1 = chosen == ranking[0]["path"]
    in_top2 = chosen in top2

    result = {
        "schema": "sbfl_e21_live.v1",
        "model": MODEL,
        "sidecar": SIDECAR,
        "f_total": F_TOTAL,
        "ranking": ranking,
        "top2": top2,
        "prompt": prompt,
        "model_answer": answer,
        "model_chose": chosen,
        "in_top1": in_top1,
        "in_top2": in_top2,
        "elapsed_ms": elapsed_ms,
        "gg_signals": (resp.get("meta") or {}).get("gg_signals", {}),
        "status": "PASS" if in_top2 else ("PARTIAL" if chosen else "FAIL"),
    }
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
