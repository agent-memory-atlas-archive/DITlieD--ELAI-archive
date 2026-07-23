"""S-20 help scenario: drives `elai help --output-format json` and asserts the binary
actually emitted its real CLI usage banner against the isolated run home (off-agent
functional content, not exit-0-only).

Model-free, deterministic, no credentials, no network: the CLI help banner is a static
usage string the binary prints, so two runs across fresh isolated homes produce
BYTE-IDENTICAL output (verified sha256-stable off-author; cwd-independent, unlike mcp).
Asserting the structure (kind=help) plus the load-bearing banner phrases proves the
binary really ran its banner assembly, rather than dumping a stub or an empty object.
A regression that drops a subcommand, the usage header, or the slash-commands section
fails these assertions, not just a missing kind.
"""

from __future__ import annotations

import json

from runner import RunResult, register


@register("s20_help", ["help", "--output-format", "json"])
def s20_help(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"

    snap = json.loads(result.stdout)
    assert isinstance(snap, dict), \
        f"FUNCTIONAL FAIL: expected a JSON object, got {type(snap).__name__}; " \
        f"stdout={result.stdout[:200]!r}"

    assert snap.get("kind") == "help", \
        f"FUNCTIONAL FAIL: expected kind=help, got {snap.get('kind')!r}"

    banner = snap.get("message")
    assert isinstance(banner, str) and len(banner) > 1000, \
        f"FUNCTIONAL FAIL: expected a non-trivial (>1000 char) usage banner, " \
        f"got {type(banner).__name__} len={len(banner) if isinstance(banner, str) else 'n/a'}"

    # load-bearing invariant phrases the real CLI banner stamps (verified off-author this
    # turn, sha256-stable across two fresh homes + cwd-independent). A regression that drops
    # the usage header, the REPL entry, an output-format flag, a documented subcommand, the
    # slash-commands section, or the flags block fails here, not just a missing kind.
    for phrase in (
        "Usage:",            # the usage header
        "elai v0.1.0",       # the version stamp
        "Start the interactive REPL",   # the REPL entry description
        "--output-format",   # the json/text output flag
        "elai doctor",       # a documented subcommand
        "elai mcp",          # a documented subcommand
        "elai skills",       # a documented subcommand
        "elai init",         # a documented subcommand
        "Slash commands",    # the slash-commands section header
        "Flags:",            # the flags block header
    ):
        assert phrase in banner, \
            f"FUNCTIONAL FAIL: expected banner to carry invariant phrase {phrase!r}, " \
            f"but it was absent (banner_len={len(banner)})"
