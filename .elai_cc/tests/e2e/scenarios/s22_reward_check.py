"""S-22 reward-check scenarios: drive `elai reward-check` (the H-28 W-1 production
entrypoint, a reward-tampering guard: frozen-evaluator-hash + locked-reward-band
security gate) on a fresh isolated home through its three distinct verdict paths.

ALLOW on a clean in-band reward delta, BLOCK on a frozen-evaluator hash mismatch
proving the agent edited its own evaluator, FLAG on a hash-intact but out-of-band
reward jump routed to human Layer-3. Each path is model-free + deterministic
(byte-identical sha256 across fresh homes). The --output-format json flag is
ignored; output is structured text.

The gate's exit code encodes the verdict: ALLOW exits 0 (the reward is admitted);
BLOCK and FLAG exit non-zero (the gate fires / fails loud). Each scenario asserts
the exit code matching its verdict path, then the functional substrings.

A regression that flips a verdict, drops the delta/band math, or loses the
hash-mismatch / out-of-band clause fails the assertions, not just exit-0.
Three registrations in one file = three distinct gate branches proven wired +
functional.
"""

from __future__ import annotations

from runner import RunResult, register


def _assert_phrases(result: RunResult, phrases: tuple[str, ...]) -> None:
    out = result.stdout
    for phrase in phrases:
        assert phrase in out, \
            f"FUNCTIONAL FAIL: expected stdout to carry {phrase!r}, but it was absent (stdout_len={len(out)})"


@register("s22_reward_check_clean", ["reward-check", "--demo-clean", "--output-format", "json"])
def s22_reward_check_clean(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"
    _assert_phrases(result, (
        "elai reward-check",
        "H-28",
        "W-1 production entrypoint",
        "demo-clean",
        "seed:",
        "VERDICT:",
        "ALLOW",
        "evaluator hash intact",
        "reward delta within band",
        "reward:",
        "delta:",
        "band:",
        "reward_tampering",
    ))


@register("s22_reward_check_tampered", ["reward-check", "--demo-tampered", "--output-format", "json"])
def s22_reward_check_tampered(result: RunResult) -> None:
    assert result.exit_code != 0, \
        f"FUNCTIONAL FAIL: expected non-zero exit (BLOCK verdict fires) for demo-tampered, got exit 0; stdout={result.stdout[:200]!r}"
    _assert_phrases(result, (
        "elai reward-check",
        "H-28",
        "demo-tampered",
        "VERDICT:",
        "BLOCK",
        "frozen-evaluator hash MISMATCH",
        "edited its own evaluator",
        "reward:",
        "delta:",
        "band:",
        "reward_tampering",
    ))


@register("s22_reward_check_jump", ["reward-check", "--demo-score-jump", "--output-format", "json"])
def s22_reward_check_jump(result: RunResult) -> None:
    assert result.exit_code != 0, \
        f"FUNCTIONAL FAIL: expected non-zero exit (FLAG verdict fires) for demo-score-jump, got exit 0; stdout={result.stdout[:200]!r}"
    _assert_phrases(result, (
        "elai reward-check",
        "H-28",
        "demo-score-jump",
        "VERDICT:",
        "FLAG",
        "hash intact but reward jumped beyond the locked band",
        "reward:",
        "delta:",
        "band:",
        "reward_tampering",
    ))
