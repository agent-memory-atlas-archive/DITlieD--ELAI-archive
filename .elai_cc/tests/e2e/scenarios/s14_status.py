"""S-14 status scenario: drives `elai status --output-format json` on a fresh
isolated home and asserts the binary emits the REAL workspace status snapshot
(off-agent functional content, not exit-0-only).

Model-free and deterministic for the asserted fields: status reports the active
model, permission mode, the full sandbox isolation snapshot, cumulative usage
counters, and the workspace git state. On a fresh empty tempdir cwd the workspace
is clean (no git project), so the asserted fields are genuine invariants of the
status command's real report, not exit-0-only.
"""

from __future__ import annotations

import json

from runner import RunResult, register


# The exact sandbox snapshot a fresh isolated home produces (verified off-agent
# on the warm binary, two identical runs). Asserting the full dict proves the
# status command actually probed the sandbox isolation state rather than dumping
# a stub.
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
    "markers": [],
    "requested_namespace": True,
    "requested_network": False,
    "supported": True,
}


@register("s14_status", ["status", "--output-format", "json"])
def s14_status(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"

    snap = json.loads(result.stdout)
    assert isinstance(snap, dict), \
        f"FUNCTIONAL FAIL: expected a JSON object, got {type(snap).__name__}; " \
        f"stdout={result.stdout[:200]!r}"

    assert snap.get("kind") == "status", \
        f"FUNCTIONAL FAIL: expected kind=status, got {snap.get('kind')!r}"

    # Active model + permission mode are real configured invariants.
    assert snap.get("model") == "claude-opus-4-6", \
        f"FUNCTIONAL FAIL: expected model=claude-opus-4-6, got {snap.get('model')!r}"
    assert snap.get("permission_mode") == "danger-full-access", \
        f"FUNCTIONAL FAIL: expected permission_mode=danger-full-access, " \
        f"got {snap.get('permission_mode')!r}"

    # The ENTIRE sandbox sub-object must match the real isolation snapshot
    # (exact-dict match catches any sandbox-state regression, not just presence).
    assert snap.get("sandbox") == _EXPECTED_SANDBOX, \
        f"FUNCTIONAL FAIL: sandbox snapshot mismatch. " \
        f"expected={_EXPECTED_SANDBOX}, got={snap.get('sandbox')!r}"

    # usage counters are all zero on a fresh session (a real invariant, asserted
    # not vacuously passed).
    usage = snap.get("usage", {})
    for key in ("cumulative_input", "cumulative_output", "cumulative_total",
                "estimated_tokens", "latest_total", "messages", "turns"):
        assert usage.get(key) == 0, \
            f"FUNCTIONAL FAIL: usage.{key} expected 0 on a fresh home, got {usage.get(key)!r}"

    # workspace: on a fresh empty tempdir cwd the workspace is clean with no git
    # project. cwd is the run's tempdir (assert present non-empty str, not a
    # fixed value); project_root is None (no git project found); git_state clean.
    ws = snap.get("workspace", {})
    assert isinstance(ws.get("cwd"), str) and len(ws["cwd"]) > 0, \
        f"FUNCTIONAL FAIL: workspace.cwd not a non-empty path: {ws.get('cwd')!r}"
    assert ws.get("project_root") is None, \
        f"FUNCTIONAL FAIL: expected project_root=null on a fresh tempdir, " \
        f"got {ws.get('project_root')!r}"
    assert ws.get("git_state") == "clean", \
        f"FUNCTIONAL FAIL: expected git_state=clean on a fresh tempdir, " \
        f"got {ws.get('git_state')!r}"
    assert ws.get("session") == "live-repl", \
        f"FUNCTIONAL FAIL: expected session=live-repl, got {ws.get('session')!r}"
    for key in ("changed_files", "staged_files", "unstaged_files", "untracked_files"):
        assert ws.get(key) == 0, \
            f"FUNCTIONAL FAIL: workspace.{key} expected 0, got {ws.get(key)!r}"
