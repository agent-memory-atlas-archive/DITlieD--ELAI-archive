"""S-2 seccodeplt scenario: drives `elai bench seccodeplt --json` and asserts
on the InsecureRateMeasured SSB row (off-agent evidence, not stdout).

The functional claim under test: the static gate demonstrably lowers the
insecure rate (insecure_rate_after < insecure_rate_before).
"""

from __future__ import annotations

from runner import RunResult, register

_FLOAT_TOL = 1e-9


@register("s2_seccodeplt", ["bench", "seccodeplt", "--json"])
def s2_seccodeplt(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"

    rows = result.ssb_rows(action_type="insecure_rate_measured")
    assert len(rows) == 1, \
        f"expected exactly 1 insecure_rate_measured row, got {len(rows)}"

    payload = rows[0]["payload"]
    assert "InsecureRateMeasured" in payload, \
        f"payload envelope key mismatch: {list(payload.keys())}"
    inner = payload["InsecureRateMeasured"]

    assert inner["suite"] == "seccodeplt_static_gate", \
        f"expected suite=seccodeplt_static_gate, got {inner['suite']!r}"
    assert inner["n_samples"] == 96, \
        f"expected n_samples=96, got {inner['n_samples']}"

    before = inner["insecure_rate_before"]
    after = inner["insecure_rate_after"]
    recall = inner["gate_recall"]

    assert abs(before - 0.5) < _FLOAT_TOL, \
        f"expected insecure_rate_before=0.5, got {before!r}"
    assert abs(after - 0.15625) < _FLOAT_TOL, \
        f"expected insecure_rate_after=0.15625, got {after!r}"
    assert abs(recall - 0.6875) < _FLOAT_TOL, \
        f"expected gate_recall=0.6875, got {recall!r}"

    corpus_blake3 = inner["corpus_blake3"]
    assert isinstance(corpus_blake3, str) and len(corpus_blake3) == 64, \
        f"expected corpus_blake3 64-char hex, got len={len(corpus_blake3)}"

    assert after < before, \
        f"FUNCTIONAL FAIL: static gate did not lower insecure rate " \
        f"(before={before}, after={after}); expected after < before"
