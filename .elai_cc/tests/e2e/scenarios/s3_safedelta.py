"""S-3 safedelta scenario: drives `elai bench safedelta --json` and asserts
on the SafetyDeltaMeasured SSB row (off-agent evidence, not stdout).

The functional claim under test: the measured safety_delta lies within its
own reported confidence interval (ci_low <= safety_delta <= ci_high).
"""

from __future__ import annotations

from runner import RunResult, register

_FLOAT_TOL = 1e-9


@register("s3_safedelta", ["bench", "safedelta", "--json"])
def s3_safedelta(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"

    rows = result.ssb_rows(action_type="safety_delta_measured")
    assert len(rows) == 1, \
        f"expected exactly 1 safety_delta_measured row, got {len(rows)}"

    payload = rows[0]["payload"]
    assert "SafetyDeltaMeasured" in payload, \
        f"payload envelope key mismatch: {list(payload.keys())}"
    inner = payload["SafetyDeltaMeasured"]

    assert inner["suite"] == "seccodeplt_static_gate", \
        f"expected suite=seccodeplt_static_gate, got {inner['suite']!r}"
    assert inner["model_id"] == "seccodeplt_curated_corpus", \
        f"expected model_id=seccodeplt_curated_corpus, got {inner['model_id']!r}"
    assert inner["n_tasks"] == 96, \
        f"expected n_tasks=96, got {inner['n_tasks']}"
    assert inner["seed"] == 0, \
        f"expected seed=0, got {inner['seed']}"

    safety_delta = inner["safety_delta"]
    ci_low = inner["ci_low"]
    ci_high = inner["ci_high"]
    gate_recall = inner["gate_recall"]

    assert abs(safety_delta - 0.34375) < _FLOAT_TOL, \
        f"expected safety_delta=0.34375, got {safety_delta!r}"
    assert abs(ci_low - 0.28125) < _FLOAT_TOL, \
        f"expected ci_low=0.28125, got {ci_low!r}"
    assert abs(ci_high - 0.40625) < _FLOAT_TOL, \
        f"expected ci_high=0.40625, got {ci_high!r}"
    assert abs(gate_recall - 0.6875) < _FLOAT_TOL, \
        f"expected gate_recall=0.6875, got {gate_recall!r}"

    corpus_blake3 = inner["corpus_blake3"]
    assert isinstance(corpus_blake3, str) and len(corpus_blake3) == 64, \
        f"expected corpus_blake3 64-char hex, got len={len(corpus_blake3)}"

    assert inner["runpod_pending"] is True, \
        f"expected runpod_pending=True, got {inner['runpod_pending']!r}"

    assert ci_low <= safety_delta <= ci_high, \
        f"FUNCTIONAL FAIL: measured safety_delta={safety_delta} outside its own " \
        f"confidence interval [{ci_low}, {ci_high}]; " \
        f"expected ci_low <= safety_delta <= ci_high"
