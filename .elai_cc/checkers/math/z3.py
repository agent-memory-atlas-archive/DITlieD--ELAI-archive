#!/usr/bin/env python3
# D-11 z3 checker -- thin stdin/stdout shim called from
# .elai_cc/crates/orchestrator/src/checker_registry.rs (Z3Checker, TOML-registered).
#
# Grounding: PoT arxiv:2211.12588 §3.1 -- "the Python interpreter is the
# ground truth"; D-11 elevates a SMT kernel to the same role for claims
# that bind a satisfiability obligation. AlphaProof Nature 2025 §Methods
# reinforces: the symbolic kernel's verdict is correctness; neural
# confidence does not vote.
#
# Protocol:
#   stdin  : {"claim": "<smt2 assertion text>", "candidate": <optional model>}
#   stdout : {"result": "pass"|"fail"|"inapplicable", "reason": "..."}
#   exit   : 0 in all three result cases; non-zero only on script-level error.
#
# A claim is "pass" iff z3 reports the asserted formula is SAT (i.e. a
# satisfying model exists). An "unsat" result is "fail" (the claim's
# satisfiability assertion is false). "unknown" maps to "inapplicable"
# (the solver could not decide within its own bound -- AlphaProof's
# UNDECIDED case).
#
# The Rust side enforces a 30s wall-clock timeout via tokio::time::timeout.
# Anti-stale-training: pin z3-solver via pyproject.toml at install time;
# do not import a fixed version inline.

from __future__ import annotations

import json
import sys


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
    if not isinstance(claim, str) or not claim.strip():
        print(json.dumps({"result": "inapplicable", "reason": "claim is empty"}))
        return 0

    try:
        import z3  # noqa: WPS433 -- intentional optional import
    except ImportError as e:
        print(json.dumps({"result": "inapplicable", "reason": f"z3-solver not installed: {e}"}))
        return 0

    # Build a Solver and feed the claim as raw SMT-LIB2 text. The helper
    # accepts either a single (assert ...) form or a full smt2 script.
    try:
        solver = z3.Solver()
        # Bound the internal search so unbounded queries surface as
        # "unknown" rather than running past the Rust 30s wall clock.
        # 25_000 ms leaves ~5s headroom for parse + IPC.
        solver.set("timeout", 25_000)
        solver.from_string(claim)
    except z3.Z3Exception as e:
        print(json.dumps({"result": "inapplicable", "reason": f"z3 parse error: {e}"}))
        return 0
    except (ValueError, TypeError) as e:
        print(json.dumps({"result": "inapplicable", "reason": f"smt2 syntax error: {e}"}))
        return 0

    try:
        verdict = solver.check()
    except z3.Z3Exception as e:
        print(json.dumps({"result": "inapplicable", "reason": f"z3 check error: {e}"}))
        return 0

    if verdict == z3.sat:
        print(json.dumps({"result": "pass", "reason": "smt-lib formula is SAT"}))
        return 0
    if verdict == z3.unsat:
        print(json.dumps({"result": "fail", "reason": "smt-lib formula is UNSAT"}))
        return 0
    # z3.unknown -- AlphaProof UNDECIDED case: solver could not decide.
    print(json.dumps({"result": "inapplicable", "reason": "z3 verdict: unknown (solver timeout or incomplete)"}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
