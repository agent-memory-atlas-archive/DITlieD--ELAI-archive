"""S-15 sandbox scenario: drives `elai sandbox --output-format json` and asserts
the binary emits the REAL sandbox isolation snapshot (off-agent functional
content, not exit-0-only).

Model-free and fully deterministic: sandbox reports the live isolation state
(whether the sandbox is active/enabled/supported, the namespace vs network
isolation split, the filesystem mode). Two runs produce byte-identical stdout
(sha256-stable), so the full snapshot is asserted as an exact-dict invariant.
"""

from __future__ import annotations

import json

from runner import RunResult, register


# The exact sandbox isolation snapshot a fresh isolated home produces (verified
# off-agent on the warm binary, two byte-identical runs). Asserting the FULL dict
# proves the sandbox command actually probed the real isolation state rather than
# dumping a stub or partial report.
_EXPECTED_SANDBOX = {
    "active": True,
    "active_namespace": True,
    "active_network": False,
    "allowed_mounts": [],
    "enabled": True,
    "fallback_reason": None,
    "filesystem_active": True,
    "filesystem_mode": "workspace-only",
    "in_container": False,
    "kind": "sandbox",
    "markers": [],
    "requested_namespace": True,
    "requested_network": False,
    "supported": True,
}


@register("s15_sandbox", ["sandbox", "--output-format", "json"])
def s15_sandbox(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"

    snap = json.loads(result.stdout)
    assert isinstance(snap, dict), \
        f"FUNCTIONAL FAIL: expected a JSON object, got {type(snap).__name__}; " \
        f"stdout={result.stdout[:200]!r}"

    assert snap.get("kind") == "sandbox", \
        f"FUNCTIONAL FAIL: expected kind=sandbox, got {snap.get('kind')!r}"

    # The ENTIRE snapshot must match the real isolation state (exact-dict match
    # catches any sandbox-state regression -- a field added/removed/flipped all
    # fail this, not just a missing kind).
    assert snap == _EXPECTED_SANDBOX, \
        f"FUNCTIONAL FAIL: sandbox snapshot mismatch. " \
        f"expected={_EXPECTED_SANDBOX}, got={snap}"
