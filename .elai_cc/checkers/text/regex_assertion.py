#!/usr/bin/env python3
# D-11 regex_assertion checker -- thin stdin/stdout shim called from
# .elai_cc/crates/orchestrator/src/checker_registry.rs (TOML-registered).
#
# Grounding: PoT arxiv:2211.12588 §3.1 -- "the Python interpreter is the
# ground truth"; for textual claims, Python `re` is the canonical
# regex authority. The pure-Rust RegexMatchChecker in
# checker_registry.rs handles the in-process fast path; this script
# exists for parity with the plan + lets the registry TOML manifest
# express regex assertions declaratively without recompiling the
# orchestrator.
#
# Protocol:
#   stdin  : {"claim": "<text under test>",
#             "candidate": {"pattern": "<regex>",
#                           "mode": "contains"|"fullmatch" (default "contains"),
#                           "flags": "i,m,s" (optional)}}
#   stdout : {"result": "pass"|"fail"|"inapplicable", "reason": "..."}
#   exit   : 0 in all three result cases; non-zero only on script-level error.
#
# The Rust side enforces a 30s wall-clock timeout via tokio::time::timeout.
# Deterministic on (claim, candidate) -- no time / no random access.

from __future__ import annotations

import json
import re
import sys


def _parse_flags(raw: str) -> int:
    """Parse a comma-separated 'i,m,s,x' flag string into re module flags."""
    out = 0
    if not raw:
        return out
    mapping = {
        "i": re.IGNORECASE,
        "ignorecase": re.IGNORECASE,
        "m": re.MULTILINE,
        "multiline": re.MULTILINE,
        "s": re.DOTALL,
        "dotall": re.DOTALL,
        "x": re.VERBOSE,
        "verbose": re.VERBOSE,
    }
    for tok in raw.split(","):
        flag = mapping.get(tok.strip().lower())
        if flag is not None:
            out |= flag
    return out


def main() -> int:
    try:
        raw = sys.stdin.read()
    except (OSError, ValueError) as e:
        print(json.dumps({"result": "inapplicable", "reason": f"stdin read failed: {e}"}))
        return 0

    if not raw.strip():
        print(json.dumps({"result": "inapplicable", "reason": "empty stdin"}))
        return 0

    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as e:
        print(json.dumps({"result": "inapplicable", "reason": f"stdin not json: {e}"}))
        return 0

    claim = payload.get("claim", "")
    if not isinstance(claim, str):
        print(json.dumps({"result": "inapplicable", "reason": "claim is not a string"}))
        return 0

    candidate = payload.get("candidate")
    if not isinstance(candidate, dict):
        print(json.dumps({"result": "inapplicable", "reason": "candidate must be an object with 'pattern'"}))
        return 0

    pattern = candidate.get("pattern", "")
    if not isinstance(pattern, str) or not pattern:
        print(json.dumps({"result": "inapplicable", "reason": "candidate.pattern is empty"}))
        return 0

    mode = candidate.get("mode", "contains")
    if mode not in ("contains", "fullmatch"):
        print(json.dumps({"result": "inapplicable", "reason": f"unknown mode '{mode}' (want contains|fullmatch)"}))
        return 0

    flags = _parse_flags(candidate.get("flags", ""))

    try:
        compiled = re.compile(pattern, flags)
    except re.error as e:
        print(json.dumps({"result": "inapplicable", "reason": f"regex compile error: {e}"}))
        return 0

    if mode == "contains":
        hit = compiled.search(claim) is not None
    else:
        hit = compiled.fullmatch(claim) is not None

    if hit:
        print(json.dumps({"result": "pass", "reason": f"regex /{pattern}/ matched ({mode})"}))
    else:
        print(json.dumps({"result": "fail", "reason": f"regex /{pattern}/ did not match ({mode})"}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
