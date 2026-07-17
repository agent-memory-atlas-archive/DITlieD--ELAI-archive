"""S-4 gate-tax scenario: drives `elai bench gate-tax --json` and asserts
on the GateTaxMeasured SSB row (off-agent evidence, not stdout).

The functional claim under test (PS-2 composed gate-tax): the composed
gate-tax number sits above its own ranked top-3 cost contributors --
(a) the top-3 are ranked descending, and
(b) their sum cannot exceed the composed total (they are a subset of the
n_gates contributors composed into coding_task_gate_tax_ms).
"""

from __future__ import annotations

from runner import RunResult, register

_FLOAT_TOL = 1e-9


@register("s4_gate_tax", ["bench", "gate-tax", "--json"])
def s4_gate_tax(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"

    rows = result.ssb_rows(action_type="gate_tax_measured")
    assert len(rows) == 1, \
        f"expected exactly 1 gate_tax_measured row, got {len(rows)}"

    payload = rows[0]["payload"]
    assert "GateTaxMeasured" in payload, \
        f"payload envelope key mismatch: {list(payload.keys())}"
    inner = payload["GateTaxMeasured"]

    assert inner["seed"] == 0, \
        f"expected seed=0, got {inner['seed']!r}"
    assert inner["runner_status"] == "runner_present", \
        f"expected runner_status=runner_present, got {inner['runner_status']!r}"
    assert inner["task_class_label"] == "fixture", \
        f"expected task_class_label=fixture, got {inner['task_class_label']!r}"
    assert inner["audit_unbounded"] is True, \
        f"expected audit_unbounded=True, got {inner['audit_unbounded']!r}"

    coding_task_gate_tax_ms = inner["coding_task_gate_tax_ms"]
    audit_gate_tax_ms = inner["audit_gate_tax_ms"]
    over_block_rate = inner["over_block_rate"]

    assert abs(coding_task_gate_tax_ms - 0.26) < _FLOAT_TOL, \
        f"expected coding_task_gate_tax_ms=0.26, got {coding_task_gate_tax_ms!r}"
    assert abs(audit_gate_tax_ms - 0.0) < _FLOAT_TOL, \
        f"expected audit_gate_tax_ms=0.0, got {audit_gate_tax_ms!r}"
    assert abs(over_block_rate - 0.0) < _FLOAT_TOL, \
        f"expected over_block_rate=0.0, got {over_block_rate!r}"

    assert inner["over_block_blocked"] == 0, \
        f"expected over_block_blocked=0, got {inner['over_block_blocked']}"
    assert inner["over_block_total"] == 55, \
        f"expected over_block_total=55, got {inner['over_block_total']}"
    assert inner["n_in_scope_coding"] == 11, \
        f"expected n_in_scope_coding=11, got {inner['n_in_scope_coding']}"
    assert inner["n_trading_surface"] == 12, \
        f"expected n_trading_surface=12, got {inner['n_trading_surface']}"
    assert inner["n_offline_audit"] == 1, \
        f"expected n_offline_audit=1, got {inner['n_offline_audit']}"
    assert inner["n_dormant"] == 1, \
        f"expected n_dormant=1, got {inner['n_dormant']}"
    assert inner["n_untagged"] == 0, \
        f"expected n_untagged=0, got {inner['n_untagged']}"

    top1_gate_id = inner["top1_gate_id"]
    top1_cost_ms = inner["top1_cost_ms"]
    top2_gate_id = inner["top2_gate_id"]
    top2_cost_ms = inner["top2_cost_ms"]
    top3_gate_id = inner["top3_gate_id"]
    top3_cost_ms = inner["top3_cost_ms"]

    assert top1_gate_id == "grounding_sweep", \
        f"expected top1_gate_id=grounding_sweep, got {top1_gate_id!r}"
    assert abs(top1_cost_ms - 0.05) < _FLOAT_TOL, \
        f"expected top1_cost_ms=0.05, got {top1_cost_ms!r}"
    assert top2_gate_id == "diagnostic_probe", \
        f"expected top2_gate_id=diagnostic_probe, got {top2_gate_id!r}"
    assert abs(top2_cost_ms - 0.04) < _FLOAT_TOL, \
        f"expected top2_cost_ms=0.04, got {top2_cost_ms!r}"
    assert top3_gate_id == "test_fail_reasons", \
        f"expected top3_gate_id=test_fail_reasons, got {top3_gate_id!r}"
    assert abs(top3_cost_ms - 0.03) < _FLOAT_TOL, \
        f"expected top3_cost_ms=0.03, got {top3_cost_ms!r}"

    assert inner["n_gates"] == 11, \
        f"expected n_gates=11, got {inner['n_gates']}"
    assert inner["n_fires"] == 11, \
        f"expected n_fires=11, got {inner['n_fires']}"

    ts_ms = inner["ts_ms"]
    assert isinstance(ts_ms, int) and ts_ms > 0, \
        f"expected ts_ms positive int, got {ts_ms!r}"

    assert top1_cost_ms >= top2_cost_ms >= top3_cost_ms, \
        f"FUNCTIONAL FAIL (a): top-3 costs not ranked descending: " \
        f"top1={top1_cost_ms}, top2={top2_cost_ms}, top3={top3_cost_ms}; " \
        f"expected top1 >= top2 >= top3"

    top3_sum = top1_cost_ms + top2_cost_ms + top3_cost_ms
    assert top3_sum <= coding_task_gate_tax_ms + _FLOAT_TOL, \
        f"FUNCTIONAL FAIL (b): top-3 sum {top3_sum} exceeds composed " \
        f"coding_task_gate_tax_ms {coding_task_gate_tax_ms} (tolerance " \
        f"{_FLOAT_TOL}); expected top1+top2+top3 <= coding_task_gate_tax_ms"
