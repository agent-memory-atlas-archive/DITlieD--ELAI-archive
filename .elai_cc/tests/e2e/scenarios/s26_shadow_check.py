"""S-26 shadow-check scenarios: drive `elai shadow-check` (the H-26 W-1 production
entrypoint, a shadow-exec + financial-invariant metamorphic-relation gate) on a
fresh isolated home through its two distinct verdict paths.

ALLOW when the demo invariant holds (in-band divergence, exit 0), BLOCK when the
demo invariant is violated (financial invariant broken, exit 1, the gate fires
loud). Each path is model-free + deterministic. The --output-format json flag is
ignored; output is structured text.

The gate's exit code encodes the verdict: ALLOW exits 0, BLOCK exits non-zero.
Each scenario asserts the exit code matching its verdict path, then the
functional substrings. A regression that flips a verdict or drops the
metamorphic-relation math fails the assertions, not just exit-0.
"""

from __future__ import annotations

from runner import RunResult, register


def _assert_phrases(result: RunResult, phrases: tuple[str, ...]) -> None:
    out = result.stdout
    for phrase in phrases:
        assert phrase in out, \
            f"FUNCTIONAL FAIL: expected stdout to carry {phrase!r}, but it was absent (stdout_len={len(out)})"


@register("s26_shadow_check_held", ["shadow-check", "--demo-invariant-held", "--output-format", "json"])
def s26_shadow_check_held(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"
    _assert_phrases(result, (
        "elai shadow-check",
        "H-26",
        "W-1 production entrypoint",
        "demo-invariant-held",
        "rule_id:",
        "FIN-INVARIANT-001",
        "seed:",
        "VERDICT:",
        "ALLOW",
        "invariants held within band",
        "differential_divergence",
    ))


@register("s26_shadow_check_violated", ["shadow-check", "--demo-invariant-violated", "--output-format", "json"])
def s26_shadow_check_violated(result: RunResult) -> None:
    assert result.exit_code != 0, \
        f"FUNCTIONAL FAIL: expected non-zero exit (BLOCK verdict fires) for demo-invariant-violated, got exit 0; stdout={result.stdout[:200]!r}"
    _assert_phrases(result, (
        "elai shadow-check",
        "H-26",
        "demo-invariant-violated",
        "rule_id:",
        "FIN-INVARIANT-001",
        "VERDICT:",
        "BLOCK",
        "financial invariant violated",
        "pnl_additivity",
        "differential_divergence",
    ))
