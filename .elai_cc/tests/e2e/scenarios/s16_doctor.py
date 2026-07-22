"""S-16 doctor scenario: drives `elai doctor --output-format json` and asserts the
binary actually ran its real local diagnostic probes (off-agent functional
content, not exit-0-only).

Model-free and deterministic: doctor probes Auth/Config/Workspace/Sandbox/System/
Runtime assets/Sidecar health/VRAM tier/Model staging on a fresh isolated home.
Two runs produce byte-identical STRUCTURED output (the 9 section statuses + the
OK/Warnings/Failures summary counts + has_failures are stable across runs; only
the tempdir path embedded inside one summary line varies, which this scenario
does not assert on). Asserting the full section->status map + the summary counts
proves doctor really probed every section rather than dumping a stub.
"""

from __future__ import annotations

import json
import re

from runner import RunResult, register


# The exact status of each diagnostic section on a fresh isolated home with no
# API key and no install-state (verified off-agent on the warm binary, two
# byte-identical runs). Asserting the FULL map proves doctor probed every
# section: a section dropped, re-ordered, or status-flipped all fail this.
_EXPECTED_SECTION_STATUS = {
    "Auth": "warn",
    "Config": "ok",
    "Workspace": "warn",
    "Sandbox": "ok",
    "System": "ok",
    "Runtime assets": "warn",
    "Sidecar health": "warn",
    "VRAM tier": "warn",
    "Model staging": "warn",
}

# Summary counts: 3 ok sections, 6 warn, 0 failures (verified stable across runs).
_EXPECTED_OK = 3
_EXPECTED_WARNINGS = 6
_EXPECTED_FAILURES = 0


@register("s16_doctor", ["doctor", "--output-format", "json"])
def s16_doctor(result: RunResult) -> None:
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"

    snap = json.loads(result.stdout)
    assert isinstance(snap, dict), \
        f"FUNCTIONAL FAIL: expected a JSON object, got {type(snap).__name__}; " \
        f"stdout={result.stdout[:200]!r}"

    assert snap.get("kind") == "doctor", \
        f"FUNCTIONAL FAIL: expected kind=doctor, got {snap.get('kind')!r}"

    # has_failures reflects that at least one section FAILED; a fresh home has 0.
    assert snap.get("has_failures") is False, \
        f"FUNCTIONAL FAIL: expected has_failures=False, got {snap.get('has_failures')!r}"

    report = snap.get("report", "")
    assert isinstance(report, str) and report, \
        f"FUNCTIONAL FAIL: expected a non-empty report string, got {type(report).__name__}"

    # Parse every "<Section>\n  Status  <status>\n  Summary <summary>" block out
    # of the report. This proves doctor actually emitted each section's result.
    section_status = dict(re.findall(
        r"^([A-Za-z][A-Za-z /]*?)\n  Status\s+(\w+)\n  Summary\s+.+$",
        report,
        re.MULTILINE,
    ))
    assert section_status == _EXPECTED_SECTION_STATUS, \
        f"FUNCTIONAL FAIL: section->status map mismatch.\n" \
        f"  expected={_EXPECTED_SECTION_STATUS}\n  got={section_status}"

    # The summary block counts (OK/Warnings/Failures) are a second, independent
    # invariant: they must equal the per-section tally.
    m = re.search(r"OK\s+(\d+)\s+Warnings\s+(\d+)\s+Failures\s+(\d+)", report)
    assert m is not None, \
        f"FUNCTIONAL FAIL: summary counts block not found in report"
    ok, warnings, failures = int(m.group(1)), int(m.group(2)), int(m.group(3))
    assert ok == _EXPECTED_OK, \
        f"FUNCTIONAL FAIL: expected OK count {_EXPECTED_OK}, got {ok}"
    assert warnings == _EXPECTED_WARNINGS, \
        f"FUNCTIONAL FAIL: expected Warnings count {_EXPECTED_WARNINGS}, got {warnings}"
    assert failures == _EXPECTED_FAILURES, \
        f"FUNCTIONAL FAIL: expected Failures count {_EXPECTED_FAILURES}, got {failures}"
