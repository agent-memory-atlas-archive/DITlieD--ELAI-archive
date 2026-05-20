#!/usr/bin/env python3
# D-11 sympy checker -- thin stdin/stdout shim called from
# .elai_cc/crates/orchestrator/src/checker_registry.rs (SympyChecker).
#
# Protocol:
#   stdin  : {"claim": "<expr_with_=>", "candidate": null}
#   stdout : {"result": "pass"|"fail"|"inapplicable", "reason": "..."}
#   exit   : 0 in all three result cases; 1 only on script-level error.
#
# The Rust side enforces a 30s wall-clock timeout via tokio::time::timeout.
# Document any added imports in pyproject.toml (sympy is the only non-stdlib
# requirement). Anti-stale-training: pin via pyproject.toml at install time;
# do not import from a fixed version inline.

from __future__ import annotations

import json
import sys


def main() -> int:
    try:
        raw = sys.stdin.read()
    except Exception as e:
        print(json.dumps({"result": "inapplicable", "reason": f"stdin read failed: {e}"}))
        return 0

    if not raw.strip():
        print(json.dumps({"result": "inapplicable", "reason": "empty stdin"}))
        return 0

    try:
        payload = json.loads(raw)
    except Exception as e:
        print(json.dumps({"result": "inapplicable", "reason": f"stdin not json: {e}"}))
        return 0

    claim = payload.get("claim", "")
    if not isinstance(claim, str) or "=" not in claim:
        print(json.dumps({"result": "inapplicable", "reason": "claim has no '=' to grade"}))
        return 0

    try:
        import sympy  # noqa: WPS433 -- intentional optional import
    except ImportError as e:
        print(json.dumps({"result": "inapplicable", "reason": f"sympy not installed: {e}"}))
        return 0

    # Split once on the first '=' so claims like "x^2 = (x+1)*(x-1) + 1" work.
    lhs_str, rhs_str = claim.split("=", 1)
    lhs_str = lhs_str.strip()
    rhs_str = rhs_str.strip()
    if not lhs_str or not rhs_str:
        print(json.dumps({"result": "inapplicable", "reason": "empty side of '='"}))
        return 0

    try:
        # sympify accepts '^' for exponent when convert_xor=True. Keep a
        # bounded namespace so the helper cannot evaluate arbitrary code.
        from sympy import simplify, sympify

        lhs = sympify(lhs_str, convert_xor=True)
        rhs = sympify(rhs_str, convert_xor=True)
        diff = simplify(lhs - rhs)
    except Exception as e:
        print(json.dumps({"result": "inapplicable", "reason": f"parse error: {e}"}))
        return 0

    if diff == 0:
        print(json.dumps({"result": "pass"}))
        return 0
    print(json.dumps({"result": "fail", "reason": f"lhs - rhs simplifies to {diff!s}"}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
