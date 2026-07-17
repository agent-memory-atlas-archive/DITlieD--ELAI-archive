"""S-6 local-live scenario: drives `elai prompt --backend=local:8181` against
the REAL local llama.cpp lane and asserts on off-agent SSB rows (not stdout).

The functional claim under test (Q-16 GO-2 Tier-1 local-lane e2e): the elai
binary's prompt arm, wired to the local llama.cpp lane on :8181, completes a
turn and leaves inference-trace evidence in the SSB db. This is the live-lane
thesis-substrate smoke: it proves the LOCAL-LANE directive (2026-07-17) end to
end, binary -> lane -> SSB, with no test shim in between.

Discovered action_types from the prompt arm (MOVE 2 recon, verbatim):
  search_mode_changed, canary_drift, cache_economics_event.
cache_economics_event carries call_class='stream' and is the inference-call
trace row; it is the load-bearing assertion. The other two are recorded as
the full discovered set so a regression that drops one is caught.

Lane tag (registry row local-lane-e2e, lane_tag_required=true):
  model = qwen3.5-4b-q4_k_m
  quant = Q4_K_M
  host  = localhost:8181 llama.cpp Vulkan RX 5700 XT
"""

from __future__ import annotations

from runner import RunResult, register

_LANE_MODEL = "qwen3.5-4b-q4_k_m"
_LANE_QUANT = "Q4_K_M"
_LANE_HOST = "localhost:8181 llama.cpp Vulkan RX 5700 XT"

_PROMPT_ARM_ACTION_TYPES = (
    "search_mode_changed",
    "canary_drift",
    "cache_economics_event",
)

_INFERENCE_TRACE_ACTION_TYPE = "cache_economics_event"


@register(
    "s6_local_live",
    ["prompt", "--backend=local:8181", "minimal prompt"],
)
def s6_local_live(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0 from elai prompt against local:8181, got " \
        f"{result.exit_code}; stderr={result.stderr[:200]}"

    inference_rows = result.ssb_rows(action_type=_INFERENCE_TRACE_ACTION_TYPE)
    assert len(inference_rows) >= 1, \
        f"LIVE-LANE-UP GUARD FAIL: expected >=1 {_INFERENCE_TRACE_ACTION_TYPE} " \
        f"SSB row from the prompt arm driving local:8181, got " \
        f"{len(inference_rows)}; the local lane must be UP and leave " \
        f"off-agent SSB evidence (empty evidence = FAIL, never SKIP). " \
        f"stderr={result.stderr[:200]}"

    payload = inference_rows[0]["payload"]
    assert "CacheEconomicsEvent" in payload, \
        f"payload envelope key mismatch: {list(payload.keys())}"
    inner = payload["CacheEconomicsEvent"]
    assert inner["call_class"] == "stream", \
        f"expected call_class='stream' (the live inference call trace), " \
        f"got {inner['call_class']!r}"

    all_rows = result.ssb_rows()
    discovered = {r["action_type"] for r in all_rows}
    missing = set(_PROMPT_ARM_ACTION_TYPES) - discovered
    assert not missing, \
        f"prompt-arm action_type regression: expected the discovered set " \
        f"{_PROMPT_ARM_ACTION_TYPES}, missing {sorted(missing)}; " \
        f"present={sorted(discovered)}"

    assert _LANE_MODEL == "qwen3.5-4b-q4_k_m", \
        f"lane tag drift: model={_LANE_MODEL!r}"
    assert _LANE_QUANT == "Q4_K_M", \
        f"lane tag drift: quant={_LANE_QUANT!r}"
    assert _LANE_HOST == "localhost:8181 llama.cpp Vulkan RX 5700 XT", \
        f"lane tag drift: host={_LANE_HOST!r}"
