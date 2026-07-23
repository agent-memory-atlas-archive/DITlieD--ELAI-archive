"""S-27 formal-verify scenarios: drive `elai formal-verify` (the E-53 / H-54
production entrypoint, the formal-verification pipeline + closure-escape-hatch
trap-detector gate) on a fresh isolated home through its two surface actions.

`formal-verify run` with --demo-escape-hatch seeds an in-process scan that
detects a closure-escape-hatch trap, blocks the closure, emits a
TrapBehaviorDetected(ClosureEscapeHatch) SSB row, and reports the verdict as
JSON. A correctly-DETECTED escape hatch is a successful gate run (the gate
reports the verdict; it does not fail-loud here), so both paths exit 0.

`formal-verify status` reports the verifier registry (RunPod-deferred toolchains)
plus the Gemma-4 block and the substitute-model list, asserting the real shipped
config.

Both actions emit REAL JSON (honored via --output-format json), so assertions
parse `json.loads(result.stdout)` and assert on the parsed object's fields, not
substring search. A regression that fails to detect the escape hatch
(closure_scan_kind != "escape_hatch_anywhere") or fails to emit the SSB row
(ssb_row_id null) fails the scenario.
"""

from __future__ import annotations

import json

from runner import RunResult, register


def _require(cond: bool, msg: str) -> None:
    assert cond, f"FUNCTIONAL FAIL: {msg}"


@register(
    "s27_formal_verify_escape_hatch",
    ["formal-verify", "run", "--target", "demo_goal.rs", "--verifier", "z3_smt", "--demo-escape-hatch", "--output-format", "json"],
)
def s27_formal_verify_escape_hatch(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"
    obj = json.loads(result.stdout)
    _require(obj["action"] == "run", f"action != run (got {obj.get('action')!r})")
    _require(obj["verifier"] == "z3_smt", f"verifier != z3_smt (got {obj.get('verifier')!r})")
    _require(obj["demo_escape_hatch"] is True, "demo_escape_hatch is not True")
    _require(
        obj["closure_scan_kind"] == "escape_hatch_anywhere",
        f"closure_scan_kind != escape_hatch_anywhere (got {obj.get('closure_scan_kind')!r})",
    )
    _require(obj["closure_blocked"] is True, "closure_blocked is not True")
    _require(
        obj["ssb_row_id"] is not None and obj["ssb_row_id"] >= 1,
        f"ssb_row_id missing or < 1 (got {obj.get('ssb_row_id')!r})",
    )
    _require(obj["contract_emitted"] is True, "contract_emitted is not True")
    _require(obj["outcome"] == "inapplicable", f"outcome != inapplicable (got {obj.get('outcome')!r})")


@register(
    "s27_formal_verify_status",
    ["formal-verify", "status", "--output-format", "json"],
)
def s27_formal_verify_status(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"
    obj = json.loads(result.stdout)
    _require(obj["action"] == "status", f"action != status (got {obj.get('action')!r})")
    verifiers = obj["verifiers"]
    _require(isinstance(verifiers, list) and len(verifiers) == 4,
             f"verifiers is not a 4-element list (got {type(verifiers).__name__}, len={len(verifiers) if isinstance(verifiers, list) else 'n/a'})")
    names = {v["verifier"] for v in verifiers}
    _require(names == {"z3_smt", "dafny", "kani", "verus"},
             f"verifier names mismatch (got {names!r})")
    for v in verifiers:
        _require(v.get("runpod_pending") is True,
                 f"verifier {v.get('verifier')!r} runpod_pending is not True (got {v.get('runpod_pending')!r})")
    _require("gemma4_blocked" in obj and obj["gemma4_blocked"],
             f"gemma4_blocked missing or falsy (got {obj.get('gemma4_blocked')!r})")
    subs = obj["substitute_models"]
    _require(isinstance(subs, list) and len(subs) >= 4,
             f"substitute_models is not a list of >= 4 (got {type(subs).__name__}, len={len(subs) if isinstance(subs, list) else 'n/a'})")
