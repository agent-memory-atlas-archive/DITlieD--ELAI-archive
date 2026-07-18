"""S-7 safedelta-live scenario: drives `elai bench safedelta --live
--protocol safety-preamble-off-on` against the REAL local llama.cpp lane
on :8181 and asserts on the SafetyDeltaMeasured SSB row (off-agent
evidence, not stdout).

The functional claim under test (PS-1 live arm): the elai binary's
safedelta live arm, wired to the local llama.cpp lane, runs the
SafetyPreambleOffOn before/after protocol over the embedded SeCodePLT
corpus and leaves a SafetyDeltaMeasured SSB row carrying live=true plus
lane provenance fields. This is the integration smoke for the PS-1 live
arm (wiring-enforcement R10): it is authored by a role that did NOT
write the feature, and it drives the REAL entrypoint end-to-end.

Lane tag (matches safedelta_runner.rs LiveLaneTag populated by
run_safedelta_live_preamble_off_on):
  model = qwen3.5-4b-q4_k_m
  quant = Q4_K_M
  host  = localhost:8181 llama.cpp Vulkan RX 5700 XT

REDTEAM guards folded into the assertions below:
  ATTACK 2 (lane-down silent SKIP): zero live=true rows = FAIL, never
    SKIP. Same fail-closed discipline as s6 (empty evidence = FAIL).
  ATTACK 3 (static relabeled live): model_id must NOT equal the static
    CORPUS_MODEL_ID `seccodeplt_curated_corpus`, and lane_model_tag must
    be present/non-null. A real live arm reports the model string from
    CompletionResponse.model; a static-relabel would carry the static
    corpus id.
  ATTACK 1 fallback guard (hardcoded-stub): exactly ONE live=true
    SafetyDeltaMeasured row per run (not zero, not duplicated). The
    payload has NO usage_tokens field (owner deferred it), so the
    usage>0 guard is NOT available; the model_id!=static + row-count==1
    fallback still catches the stub attack per spec line 138.
"""

from __future__ import annotations

import math

from runner import RunResult, register

_LANE_MODEL_TAG = "qwen3.5-4b-q4_k_m"
_LANE_HOST_CONTAINS = "localhost:8181"
_PROTOCOL = "SafetyPreambleOffOn"
_STATIC_CORPUS_MODEL_ID = "seccodeplt_curated_corpus"
_ACTION_TYPE = "safety_delta_measured"


@register(
    "s7_safedelta_live",
    ["bench", "safedelta", "--live", "--protocol", "safety-preamble-off-on", "--json"],
    timeout=300,
)
def s7_safedelta_live(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0 from elai bench safedelta --live against " \
        f"local:8181, got {result.exit_code}; stderr={result.stderr[:200]}"

    rows = result.ssb_rows(action_type=_ACTION_TYPE)
    assert len(rows) >= 1, \
        f"LIVE-LANE-UP GUARD FAIL (ATTACK 2): expected >=1 " \
        f"{_ACTION_TYPE} SSB row from the safedelta live arm driving " \
        f"local:8181, got {len(rows)}; the local lane must be UP and " \
        f"leave off-agent SSB evidence (empty evidence = FAIL, never " \
        f"SKIP). stderr={result.stderr[:200]}"

    live_rows = [
        r for r in rows
        if r["payload"].get("SafetyDeltaMeasured", {}).get("live") is True
    ]
    assert len(live_rows) == 1, \
        f"STUB GUARD FAIL (ATTACK 1 fallback): expected exactly ONE " \
        f"live=true {_ACTION_TYPE} row per run, got {len(live_rows)} " \
        f"(total {len(rows)}); a hardcoded stub would emit zero or " \
        f"duplicate rows"

    inner = live_rows[0]["payload"]["SafetyDeltaMeasured"]

    assert inner["live"] is True, \
        f"expected live=true, got {inner['live']!r}"

    model_id = inner["model_id"]
    assert model_id != _STATIC_CORPUS_MODEL_ID, \
        f"STATIC-RELABEL GUARD FAIL (ATTACK 3): model_id=" \
        f"{model_id!r} matches the static CORPUS_MODEL_ID " \
        f"{_STATIC_CORPUS_MODEL_ID!r}; a real live arm reports the " \
        f"model string from CompletionResponse.model, not the static " \
        f"corpus id"

    lane_model_tag = inner.get("lane_model_tag")
    assert lane_model_tag is not None and lane_model_tag != "", \
        f"STATIC-RELABEL GUARD FAIL (ATTACK 3): lane_model_tag is " \
        f"missing/null; a real live arm populates lane provenance"
    assert lane_model_tag == _LANE_MODEL_TAG, \
        f"expected lane_model_tag={_LANE_MODEL_TAG!r}, got " \
        f"{lane_model_tag!r}"

    lane_host = inner.get("lane_host")
    assert lane_host is not None and _LANE_HOST_CONTAINS in lane_host, \
        f"expected lane_host containing {_LANE_HOST_CONTAINS!r}, got " \
        f"{lane_host!r}"

    protocol = inner.get("protocol")
    assert protocol == _PROTOCOL, \
        f"expected protocol={_PROTOCOL!r} (the exact string " \
        f"LiveBeforeAfter::SafetyPreambleOffOn.as_str() serializes in " \
        f"safedelta_runner.rs:106), got {protocol!r}"

    safety_delta = inner["safety_delta"]
    assert isinstance(safety_delta, float), \
        f"expected safety_delta float, got {type(safety_delta).__name__}"
    assert math.isfinite(safety_delta), \
        f"expected safety_delta finite f64, got {safety_delta!r} " \
        f"(NaN/inf is a live-arm numeric failure)"
