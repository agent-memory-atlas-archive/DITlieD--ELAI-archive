"""S-10 production-checklist scenario: drives `elai production-checklist
--output-format json` on a fresh isolated home and asserts the binary actually
ran the production security checklist and emitted a real per-check result set
(off-agent functional content, not exit-0-only — R4).

Model-free: production-checklist runs real security-scan tools
(gitleaks/trufflehog/semgrep/cargo-audit/etc) against the project root. On a
fresh empty home most tools are absent -> toolchain_missing, and checks like
npm_audit report ok-empty (no package.json to audit). The FUNCTIONAL claim under
test is that the checklist enumerated its real check set with a genuine result
per check, not that it passed.
"""

from __future__ import annotations

import json

from runner import RunResult, register


# The real production-checklist check set (grounded off-agent on the warm binary
# this session). Asserting the exact set proves the checklist actually ran its
# checks rather than exiting 0 on an empty/no-op path.
_EXPECTED_CHECK_IDS = {
    "secrets_scan",
    "trufflehog_scan",
    "owasp_static",
    "npm_audit",
    "pip_audit",
    "cargo_audit",
    "lighthouse",
    "biome",
    "ruff",
    "clippy",
    "sqlite3_rls",
    "cargo_machete",
    "otel_check",
    "sentry_check",
    "lsof_port",
}
_VALID_KINDS = {"toolchain_missing", "ok", "error"}


@register("s10_production_checklist", ["production-checklist", "--output-format", "json"])
def s10_production_checklist(result: RunResult) -> None:
    # project-root defaults to cwd (the fresh isolated home the runner created),
    # so the checklist scans an empty dir -> the documented toolchain_missing /
    # ok-empty results, exit 0.
    assert result.exit_code == 0, \
        f"expected exit 0, got {result.exit_code}; stderr={result.stderr[:200]}"

    # The off-agent functional content: the real enumerated per-check result set.
    checks = json.loads(result.stdout)
    assert isinstance(checks, list) and len(checks) > 0, \
        f"FUNCTIONAL FAIL: expected a non-empty JSON array of checks, " \
        f"got {type(checks).__name__}; stdout={result.stdout[:200]!r}"

    check_ids = set()
    for entry in checks:
        assert isinstance(entry, list) and len(entry) == 2, \
            f"expected [check_id, result] pair, got {entry!r}"
        check_id, res = entry
        assert isinstance(check_id, str), f"check_id not a str: {check_id!r}"
        check_ids.add(check_id)
        assert isinstance(res, dict) and "kind" in res, \
            f"FUNCTIONAL FAIL: result for {check_id!r} has no 'kind' field: {res!r}"
        assert res["kind"] in _VALID_KINDS, \
            f"FUNCTIONAL FAIL: {check_id!r} kind {res['kind']!r} not in {_VALID_KINDS}"

    # Every expected check was actually enumerated (proves the checklist ran its
    # real check set, not an empty stub). Exact-set match catches both a
    # truncated checklist and a silent no-op.
    assert check_ids == _EXPECTED_CHECK_IDS, \
        f"FUNCTIONAL FAIL: production checklist check set mismatch. " \
        f"missing={_EXPECTED_CHECK_IDS - check_ids}, " \
        f"unexpected={check_ids - _EXPECTED_CHECK_IDS}"

    # npm_audit on a home with no package.json is a documented ok-empty (a real
    # result, not a skip). Asserting it is present with kind 'ok' proves the
    # tool ran against the real (empty) project state.
    npm = dict(checks)["npm_audit"]
    assert npm["kind"] == "ok" and npm.get("findings") == [], \
        f"FUNCTIONAL FAIL: npm_audit expected ok/empty on a no-package home, got {npm!r}"
