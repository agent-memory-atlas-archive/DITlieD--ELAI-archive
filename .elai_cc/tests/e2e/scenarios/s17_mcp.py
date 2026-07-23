"""S-17 mcp scenario: drives `elai mcp --output-format json` and asserts the binary
actually ran its MCP-config discovery against the isolated run home (off-agent
functional content, not exit-0-only).

Model-free, deterministic, no credentials: on a fresh isolated home MCP discovery
finds no configured servers. Two runs produce byte-identical STRUCTURED output once
the variable working_directory path is normalized. Asserting the full structure
(kind/action/configured_servers/servers) AND that working_directory equals the
run's home proves mcp really resolved + reported the directory it discovered
against, rather than dumping a stub or a fixed path.
"""

from __future__ import annotations

import json

from runner import RunResult, register


@register("s17_mcp", ["mcp", "--output-format", "json"])
def s17_mcp(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"

    snap = json.loads(result.stdout)
    assert isinstance(snap, dict), \
        f"FUNCTIONAL FAIL: expected a JSON object, got {type(snap).__name__}; " \
        f"stdout={result.stdout[:200]!r}"

    assert snap.get("kind") == "mcp", \
        f"FUNCTIONAL FAIL: expected kind=mcp, got {snap.get('kind')!r}"
    assert snap.get("action") == "list", \
        f"FUNCTIONAL FAIL: expected action=list, got {snap.get('action')!r}"
    assert snap.get("configured_servers") == 0, \
        f"FUNCTIONAL FAIL: expected configured_servers=0 on a fresh home, " \
        f"got {snap.get('configured_servers')!r}"
    assert snap.get("servers") == [], \
        f"FUNCTIONAL FAIL: expected an empty servers list on a fresh home, " \
        f"got {snap.get('servers')!r}"

    wd = snap.get("working_directory")
    assert isinstance(wd, str) and wd, \
        f"FUNCTIONAL FAIL: expected a non-empty working_directory, got {wd!r}"
    assert wd == str(result.home), \
        f"FUNCTIONAL FAIL: working_directory {wd!r} != run home {str(result.home)!r}"
