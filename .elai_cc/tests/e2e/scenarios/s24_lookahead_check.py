"""S-24 lookahead-check scenarios: drive `elai lookahead-check` (the H-21 W-1
production entrypoint, a look-ahead / future-leak differential guard over a
trading strategy) on a fresh isolated home through its two distinct verdict
paths.

BLOCK on a demo-future-leak pipeline that read the future (next_close_leak
changed at row 4 when the future was truncated: full=99, truncated=14,
abs=85, rel=0.8585...). ALLOW on a demo-causal pipeline where all 2 columns
diff to zero within tolerance when the future was truncated, no future used.

Unlike reward-check / gate-selfcheck, BOTH paths exit 0: a correctly-DETECTED
leak is a successful gate run, not a failure. The gate reports the verdict;
the exit code does not encode it. Each path is model-free + deterministic
(byte-identical sha256 across 3 fresh homes, no state leak, verified
off-author this turn). The --output-format json flag is ignored; output is
structured text.

A regression that flips a verdict, drops the rule_id / tolerance, loses the
future-leak evidence clause (next_close_leak, Fail-closed Block) or the
causal no-future-used clause fails the assertions, not just exit-0. Two
registrations in one file = two distinct gate branches proven wired +
functional.
"""

from __future__ import annotations

from runner import RunResult, register


def _assert_phrases(result: RunResult, phrases: tuple[str, ...]) -> None:
    out = result.stdout
    for phrase in phrases:
        assert phrase in out, \
            f"FUNCTIONAL FAIL: expected stdout to carry {phrase!r}, but it was absent (stdout_len={len(out)})"


@register("s24_lookahead_check_future_leak", ["lookahead-check", "--demo-future-leak", "--output-format", "json"])
def s24_lookahead_check_future_leak(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"
    _assert_phrases(result, (
        "elai lookahead-check",
        "H-21",
        "W-1 production entrypoint",
        "demo-future-leak",
        "FIN-LOOKAHEAD-001",
        "demo_strategy.py",
        "tolerance:",
        "seed:",
        "VERDICT:",
        "BLOCK",
        "future leak",
        "future_leak",
        "next_close_leak",
        "Fail-closed Block",
        "lookahead_differential",
    ))


@register("s24_lookahead_check_causal", ["lookahead-check", "--demo-causal", "--output-format", "json"])
def s24_lookahead_check_causal(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"
    _assert_phrases(result, (
        "elai lookahead-check",
        "H-21",
        "demo-causal",
        "FIN-LOOKAHEAD-001",
        "VERDICT:",
        "ALLOW",
        "causal",
        "Causal: all 2 columns diff to zero within tolerance",
        "No future used",
        "lookahead_differential",
    ))
