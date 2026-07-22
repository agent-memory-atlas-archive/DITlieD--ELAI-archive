"""S-13 version scenario: drives `elai version --output-format json` and asserts
the binary emits the REAL version report (off-agent functional content, not
exit-0-only).

Model-free and deterministic: version dumps the compiled build identity (version
string, git sha, target, build date). The FUNCTIONAL claim under test is that the
version report carries the real ELAI build identity, not an empty or stub dump.
Two runs produce byte-identical stdout (sha256-stable), so the asserted fields are
genuine invariants.
"""

from __future__ import annotations

import json

from runner import RunResult, register


@register("s13_version", ["version", "--output-format", "json"])
def s13_version(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"

    report = json.loads(result.stdout)
    assert isinstance(report, dict), \
        f"FUNCTIONAL FAIL: expected a JSON object, got {type(report).__name__}; " \
        f"stdout={result.stdout[:200]!r}"

    assert report.get("kind") == "version", \
        f"FUNCTIONAL FAIL: expected kind=version, got {report.get('kind')!r}"

    # version is a real semver string, never null/empty (the compiled identity).
    assert report.get("version") == "0.1.0", \
        f"FUNCTIONAL FAIL: expected version=0.1.0, got {report.get('version')!r}"

    # git_sha/target are null in this debug build (unknown at build time) -- a
    # real invariant, asserted not hidden.
    assert report.get("git_sha") is None, \
        f"FUNCTIONAL FAIL: expected git_sha=null in this build, got {report.get('git_sha')!r}"
    assert report.get("target") is None, \
        f"FUNCTIONAL FAIL: expected target=null in this build, got {report.get('target')!r}"

    # message is the real human-readable report. Asserting it starts with the
    # ELAI banner and contains the version proves the report is genuine, not a
    # stub dump. (Exact full message is sha256-stable but the banner prefix +
    # version presence is the load-bearing functional check.)
    msg = report.get("message", "")
    assert isinstance(msg, str) and msg.startswith("ELAI\n  Version"), \
        f"FUNCTIONAL FAIL: message is not the real ELAI version banner: {msg[:80]!r}"
    assert "0.1.0" in msg, \
        f"FUNCTIONAL FAIL: version string 0.1.0 missing from message: {msg!r}"
