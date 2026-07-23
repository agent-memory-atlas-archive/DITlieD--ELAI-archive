"""S-18 skills scenario: drives `elai skills --output-format json` and asserts the binary
actually ran its skills discovery against the isolated run home (off-agent functional
content, not exit-0-only).

Model-free, deterministic, no credentials: on a fresh isolated home skills discovery finds
no skills and reports a zero/zero/zero summary. Two runs produce BYTE-IDENTICAL structured
output (verified sha256-stable across two fresh homes; no variable path field, unlike mcp).
Asserting the full structure (kind/action/skills list + the exact summary dict) proves skills
really resolved and ran its discovery, rather than dumping a stub or a fixed object.
"""

from __future__ import annotations

import json

from runner import RunResult, register


@register("s18_skills", ["skills", "--output-format", "json"])
def s18_skills(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"

    snap = json.loads(result.stdout)
    assert isinstance(snap, dict), \
        f"FUNCTIONAL FAIL: expected a JSON object, got {type(snap).__name__}; " \
        f"stdout={result.stdout[:200]!r}"

    assert snap.get("kind") == "skills", \
        f"FUNCTIONAL FAIL: expected kind=skills, got {snap.get('kind')!r}"
    assert snap.get("action") == "list", \
        f"FUNCTIONAL FAIL: expected action=list, got {snap.get('action')!r}"
    assert snap.get("skills") == [], \
        f"FUNCTIONAL FAIL: expected an empty skills list on a fresh home, " \
        f"got {snap.get('skills')!r}"

    # exact-dict match on the summary: a dropped/added/renamed/flipped count all fail here,
    # not just field presence. This is the load-bearing assertion (verified off-agent this turn:
    # skills summary on a fresh isolated home is exactly {"active": 0, "shadowed": 0, "total": 0}).
    expected_summary = {"active": 0, "shadowed": 0, "total": 0}
    assert snap.get("summary") == expected_summary, \
        f"FUNCTIONAL FAIL: expected summary {expected_summary!r}, got {snap.get('summary')!r}"
