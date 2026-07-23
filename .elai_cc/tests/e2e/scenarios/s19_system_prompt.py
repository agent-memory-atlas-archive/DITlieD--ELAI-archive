"""S-19 scenario: drives the top-level ELAI prompt-assembly subcommand with --output-format json
and asserts the binary actually ran its real prompt assembly against the isolated run home
(off-agent functional content, not exit-0-only).

Model-free and deterministic: the prompt assembly needs no credentials and no inference. Two
runs on fresh isolated homes produce BYTE-IDENTICAL structured output (verified sha256-stable
across two fresh homes). Asserting kind, that `message` is a real non-empty assembled prompt
carrying the load-bearing invariant phrases the ELAI generator stamps (the opening identity +
the permission-governance section), and that `sections` is a real non-empty structure, proves
the generator really ran rather than dumped a stub or an empty object.
"""

from __future__ import annotations

import json

from runner import RunResult, register


@register("s19_system_prompt", ["system-prompt", "--output-format", "json"])
def s19_system_prompt(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"

    snap = json.loads(result.stdout)
    assert isinstance(snap, dict), \
        f"FUNCTIONAL FAIL: expected a JSON object, got {type(snap).__name__}; " \
        f"stdout={result.stdout[:200]!r}"

    assert snap.get("kind") == "system-prompt", \
        f"FUNCTIONAL FAIL: expected kind=system-prompt, got {snap.get('kind')!r}"

    message = snap.get("message")
    assert isinstance(message, str) and len(message) > 1000, \
        f"FUNCTIONAL FAIL: expected a real (>1000 char) assembled prompt message, " \
        f"got len={len(message) if isinstance(message, str) else 'non-str'}"

    # load-bearing invariant phrases the real ELAI generator stamps: the opening agent identity
    # + the permission-governance section. A stub dump or an empty/template regression drops one
    # or both. These are the generator's OWN verbatim phrases (verified off-agent this turn).
    assert "software engineering tasks" in message, \
        f"FUNCTIONAL FAIL: opening identity phrase missing from assembled prompt; " \
        f"message[:200]={message[:200]!r}"
    assert "permission mode" in message, \
        f"FUNCTIONAL FAIL: permission-governance phrase missing from assembled prompt; " \
        f"message[:200]={message[:200]!r}"

    sections = snap.get("sections")
    assert sections is not None and sections != [], \
        f"FUNCTIONAL FAIL: expected a non-empty sections structure, got {sections!r}"
