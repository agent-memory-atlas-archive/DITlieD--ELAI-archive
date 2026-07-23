"""S-21 leakage-check scenario: drives `elai leakage-check --output-format json` (the
H-20 W-1 production entrypoint, a fail-closed financial-leakage security gate) on a
fresh isolated home and asserts the binary actually ran its real fail-closed
DKG-availability check (functional content, not exit-0-only).

Model-free, deterministic, no credentials, no network: the gate fail-closed BLOCKs
when no DKG is present, so two runs across fresh isolated homes produce byte-identical
output (verified sha256-stable off-author this turn). The --output-format json flag is
ignored by this subcommand; output is structured text, not JSON.

A regression that exits non-zero, drops the rule id, flips the verdict away from
fail-closed BLOCK, or loses the R1/R2 missing-evidence clause fails these assertions,
not just a missing banner.
"""

from __future__ import annotations

from runner import RunResult, register


@register("s21_leakage_check", ["leakage-check", "--output-format", "json"])
def s21_leakage_check(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"

    out = result.stdout

    for phrase in (
        "elai leakage-check",
        "H-20",
        "W-1 production entrypoint",
        "pipeline.py",
        "dkg_mode",
        "FIN-LEAK-001",
        "seed:",
        "VERDICT:",
        "BLOCK",
        "dkg_unavailable",
        "fail-closed",
        "missing-evidence=FAIL",
        "leakage_blocked",
    ):
        assert phrase in out, \
            f"FUNCTIONAL FAIL: expected stdout to carry {phrase!r}, but it was absent (stdout_len={len(out)})"
