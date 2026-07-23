"""S-23 gate-selfcheck scenarios: drive `elai gate-selfcheck` (the H-27 W-1
production entrypoint, a gate-self-validation guard that mutation-tests ELAI's
own safety gates) on a fresh isolated home through its two demo modes.

demo-all-valid runs the 8-gate registered set; demo-gate-broken runs the 9-gate
set (one extra broken gate). Both modes BLOCK on the deflation gate's
evaluate_errored fixture with mutation_score 0.0% and exit non-zero (fail-loud,
same contract as reward-check's BLOCK/FLAG). The --output-format json flag is
ignored; output is structured text.

Model-free + deterministic (byte-identical sha256 across fresh homes). The two
modes are distinguished by their registered gate count (8 vs 9), asserted as a
load-bearing line each. A regression that drops the verdict, the
mutation_score, the offending-gate clause, or collapses the two modes' gate
counts fails the assertions, not just exit.
"""

from __future__ import annotations

from runner import RunResult, register


def _assert_phrases(result: RunResult, phrases: tuple[str, ...]) -> None:
    out = result.stdout
    for phrase in phrases:
        assert phrase in out, \
            f"FUNCTIONAL FAIL: expected stdout to carry {phrase!r}, but it was absent (stdout_len={len(out)})"


@register("s23_gate_selfcheck_all_valid", ["gate-selfcheck", "--demo-all-valid", "--output-format", "json"])
def s23_gate_selfcheck_all_valid(result: RunResult) -> None:
    assert result.exit_code != 0, \
        f"FUNCTIONAL FAIL: expected non-zero exit (BLOCK verdict fires) for demo-all-valid, got exit 0; stdout={result.stdout[:200]!r}"
    out = result.stdout
    assert "0/8 killed" in out, \
        f"FUNCTIONAL FAIL: expected gates 0/8 killed for demo-all-valid, got stdout_len={len(out)}"
    _assert_phrases(result, (
        "elai gate-selfcheck",
        "H-27",
        "W-1 production entrypoint",
        "demo-all-valid",
        "seed:",
        "VERDICT:",
        "BLOCK",
        "a gate was un-runnable",
        "mutation_score:",
        "0.0%",
        "offending gate:",
        "deflation",
        "gate_self_validation",
    ))


@register("s23_gate_selfcheck_broken", ["gate-selfcheck", "--demo-gate-broken", "--output-format", "json"])
def s23_gate_selfcheck_broken(result: RunResult) -> None:
    assert result.exit_code != 0, \
        f"FUNCTIONAL FAIL: expected non-zero exit (BLOCK verdict fires) for demo-gate-broken, got exit 0; stdout={result.stdout[:200]!r}"
    out = result.stdout
    assert "0/9 killed" in out, \
        f"FUNCTIONAL FAIL: expected gates 0/9 killed for demo-gate-broken, got stdout_len={len(out)}"
    _assert_phrases(result, (
        "elai gate-selfcheck",
        "H-27",
        "W-1 production entrypoint",
        "demo-gate-broken",
        "seed:",
        "VERDICT:",
        "BLOCK",
        "a gate was un-runnable",
        "mutation_score:",
        "0.0%",
        "offending gate:",
        "deflation",
        "gate_self_validation",
    ))
