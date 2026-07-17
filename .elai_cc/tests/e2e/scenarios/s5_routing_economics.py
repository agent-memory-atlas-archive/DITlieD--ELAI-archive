"""S-5 routing-economics scenario: drives `elai bench routing-economics --json`
and asserts on the RoutingEconomicsMeasured SSB row (off-agent evidence, not
stdout).

The functional claim under test (PS-4 routing-economics): selective offload
lowers the cost per verified task. The hybrid arm offloads selectively and
verifies the same 6 tasks for less than the force-cloud arm that offloads
nothing. The per-verified cost is also internally consistent: per-verified
* n_verified == total_cost, integer-exact.
"""

from __future__ import annotations

from runner import RunResult, register

_FLOAT_TOL = 1e-9

_LIVE_COST_NOT_FUNDED = "cost unavailable \u2014 live lane not funded"


@register("s5_routing_economics", ["bench", "routing-economics", "--json"])
def s5_routing_economics(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"

    rows = result.ssb_rows(action_type="routing_economics_measured")
    assert len(rows) == 1, \
        f"expected exactly 1 routing_economics_measured row, got {len(rows)}"

    payload = rows[0]["payload"]
    assert "RoutingEconomicsMeasured" in payload, \
        f"payload envelope key mismatch: {list(payload.keys())}"
    inner = payload["RoutingEconomicsMeasured"]

    assert inner["seed"] == 0, \
        f"expected seed=0, got {inner['seed']!r}"
    assert inner["runner_status"] == "runner_present", \
        f"expected runner_status=runner_present, got {inner['runner_status']!r}"
    assert inner["fixture_status"] == "measured", \
        f"expected fixture_status=measured, got {inner['fixture_status']!r}"
    assert inner["selected_arm"] == "hybrid", \
        f"expected selected_arm=hybrid, got {inner['selected_arm']!r}"
    assert inner["suite"] == "routing_economics_fixture_v1", \
        f"expected suite=routing_economics_fixture_v1, got {inner['suite']!r}"

    fixture_blake3 = inner["fixture_blake3"]
    assert fixture_blake3 == \
        "f227a7ecd0c296060e23a2dd0210a157d7036850d59fc408b582f115fc3e028e", \
        f"expected fixture_blake3 to match the deterministic INPUT fixture " \
        f"hash, got {fixture_blake3!r}"

    report_blake3 = inner["report_blake3"]
    assert report_blake3 == \
        "cb6bd4b446e038c89943c046ebabed7a889ec67e2806cf0fada427006ff6f33d", \
        f"expected report_blake3 to match the byte-stable recon value, " \
        f"got {report_blake3!r}"

    hybrid_n_tasks = inner["hybrid_n_tasks"]
    hybrid_n_offloaded = inner["hybrid_n_offloaded"]
    hybrid_percent_offloaded = inner["hybrid_percent_offloaded"]
    hybrid_n_verified = inner["hybrid_n_verified"]
    hybrid_total_cost_micros = inner["hybrid_total_cost_micros"]
    hybrid_cost_per_verified_task_micros = inner["hybrid_cost_per_verified_task_micros"]

    assert hybrid_n_tasks == 6, \
        f"expected hybrid_n_tasks=6, got {hybrid_n_tasks}"
    assert hybrid_n_offloaded == 3, \
        f"expected hybrid_n_offloaded=3, got {hybrid_n_offloaded}"
    assert abs(hybrid_percent_offloaded - 50.0) < _FLOAT_TOL, \
        f"expected hybrid_percent_offloaded=50.0, got {hybrid_percent_offloaded!r}"
    assert hybrid_n_verified == 6, \
        f"expected hybrid_n_verified=6, got {hybrid_n_verified}"
    assert hybrid_total_cost_micros == 68400, \
        f"expected hybrid_total_cost_micros=68400, got {hybrid_total_cost_micros}"
    assert hybrid_cost_per_verified_task_micros == 11400, \
        f"expected hybrid_cost_per_verified_task_micros=11400 (Some), " \
        f"got {hybrid_cost_per_verified_task_micros!r}"

    force_cloud_n_tasks = inner["force_cloud_n_tasks"]
    force_cloud_n_offloaded = inner["force_cloud_n_offloaded"]
    force_cloud_percent_offloaded = inner["force_cloud_percent_offloaded"]
    force_cloud_n_verified = inner["force_cloud_n_verified"]
    force_cloud_total_cost_micros = inner["force_cloud_total_cost_micros"]
    force_cloud_cost_per_verified_task_micros = \
        inner["force_cloud_cost_per_verified_task_micros"]

    assert force_cloud_n_tasks == 6, \
        f"expected force_cloud_n_tasks=6, got {force_cloud_n_tasks}"
    assert force_cloud_n_offloaded == 0, \
        f"expected force_cloud_n_offloaded=0, got {force_cloud_n_offloaded}"
    assert abs(force_cloud_percent_offloaded - 0.0) < _FLOAT_TOL, \
        f"expected force_cloud_percent_offloaded=0.0, " \
        f"got {force_cloud_percent_offloaded!r}"
    assert force_cloud_n_verified == 6, \
        f"expected force_cloud_n_verified=6, got {force_cloud_n_verified}"
    assert force_cloud_total_cost_micros == 86700, \
        f"expected force_cloud_total_cost_micros=86700, " \
        f"got {force_cloud_total_cost_micros}"
    assert force_cloud_cost_per_verified_task_micros == 14450, \
        f"expected force_cloud_cost_per_verified_task_micros=14450 (Some), " \
        f"got {force_cloud_cost_per_verified_task_micros!r}"

    live_cost = inner["live_cost"]
    live_lane_funded = inner["live_lane_funded"]
    assert live_lane_funded is False, \
        f"OFFLINE-FIXTURE GUARD FAIL: live_lane_funded={live_lane_funded!r}; " \
        f"expected False (the fixture path; a True means the harness home " \
        f"hit a funded live lane and the cost numbers are non-reproducible)"
    assert live_cost == _LIVE_COST_NOT_FUNDED, \
        f"OFFLINE-FIXTURE GUARD FAIL: live_cost={live_cost!r}; expected the " \
        f"exact 'not funded' Rust source literal"

    assert hybrid_cost_per_verified_task_micros < \
        force_cloud_cost_per_verified_task_micros, \
        f"FUNCTIONAL FAIL (a): hybrid $/verified-task " \
        f"{hybrid_cost_per_verified_task_micros} is not below force-cloud " \
        f"{force_cloud_cost_per_verified_task_micros}; selective offload " \
        f"should lower per-verified cost (PS-4 routing-economics claim)"

    hybrid_composed = \
        hybrid_cost_per_verified_task_micros * hybrid_n_verified
    assert hybrid_composed == hybrid_total_cost_micros, \
        f"FUNCTIONAL FAIL (b) hybrid: per_verified*n_verified " \
        f"= {hybrid_cost_per_verified_task_micros}*{hybrid_n_verified} = " \
        f"{hybrid_composed} != total_cost {hybrid_total_cost_micros}"

    force_cloud_composed = \
        force_cloud_cost_per_verified_task_micros * force_cloud_n_verified
    assert force_cloud_composed == force_cloud_total_cost_micros, \
        f"FUNCTIONAL FAIL (b) force_cloud: per_verified*n_verified " \
        f"= {force_cloud_cost_per_verified_task_micros}*" \
        f"{force_cloud_n_verified} = {force_cloud_composed} != total_cost " \
        f"{force_cloud_total_cost_micros}"
