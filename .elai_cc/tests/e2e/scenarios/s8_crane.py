"""S-8 crane scenario: drives `elai bench crane` and asserts on the STDOUT
grammar-gate verdict summary (off-agent artifact, not ssb.db — crane writes
its GrammarGateEvent rows to a HERMETIC tempdir torn down at exit, so the
project ssb.db reads empty; the bench's OWN printed verdict is the artifact)."""

from __future__ import annotations

from runner import RunResult, register


@register("s8_crane", ["bench", "crane"])
def s8_crane(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"
    # crane writes 3 GrammarGateEvent rows (CranePassed/PinViolation/PostParseRejected)
    # to a HERMETIC tempdir it tears down at exit -> UNQUERYABLE from ssb.db.
    # The off-agent artifact is the bench's OWN STDOUT verdict line.
    assert "CRANE bench PASS" in result.stdout, \
        f"FUNCTIONAL FAIL: crane verdict summary missing from stdout; " \
        f"got stdout={result.stdout[:300]!r}"
