#!/usr/bin/env python3
# D-11 registry_check checker -- anti-slopsquatting verifier.
#
# CRITICAL: this script is the canonical defense for the CLAUDE.md
# anti-slopsquatting HARD RULE. When an agent emits a claim that says
# "use package <X>" or imports <X>, this checker queries the upstream
# registry (PyPI / npm) and reports PASS iff the package exists with a
# recent publish window.
#
# Grounding:
#   * CLAUDE.md "Anti-slopsquatting (HARD RULE)" -- NEVER assume a
#     dependency exists; verify name + existence + version + last-published
#     date before use.
#   * PoT arxiv:2211.12588 §3.1 (deterministic external check as ground
#     truth) -- the registry's own API IS the ground truth on package
#     existence, the model does not get a vote.
#
# Protocol:
#   stdin  : {"claim": "<pkg-name>", "candidate": {"ecosystem": "pypi"|"npm",
#            "max_age_days": 730 (optional, default 730)}}
#       OR shorthand: {"claim": "<pkg-name>", "candidate": "pypi"}
#   stdout : {"result": "pass"|"fail"|"inapplicable", "reason": "..."}
#   exit   : 0 in all three result cases; non-zero only on script-level error.
#
# PASS conditions: registry returns 200 with a `latest` version AND the
# latest publish timestamp is within `max_age_days` (default 2 years).
# FAIL: registry returns 404 OR the latest publish is older than
# `max_age_days` (the package may exist but is stale/abandoned --
# slopsquatting is also catching agents that hallucinate a stale fork).
# INAPPLICABLE: network error, missing ecosystem, empty claim.
#
# Runtime <30s enforced by the Rust caller. Network dep: `requests` (and
# stdlib `json`). Stdlib `urllib` fallback used when `requests` is absent.

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from typing import Any


def _isoparse(s: str) -> datetime | None:
    """Parse an ISO-8601 timestamp into an aware UTC datetime, or None."""
    # PyPI publishes "YYYY-MM-DDTHH:MM:SS"; npm uses RFC 3339 with offsets.
    # `fromisoformat` (Python 3.11+) accepts both; fall back to a manual
    # trim for older runtimes.
    s = s.strip().replace("Z", "+00:00")
    try:
        dt = datetime.fromisoformat(s)
    except ValueError:
        return None
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    return dt.astimezone(timezone.utc)


def _http_get_json(url: str, timeout: float = 10.0) -> tuple[int, dict[str, Any] | None, str]:
    """Return (status_code, json_body, error_str). status_code=0 on transport error."""
    try:
        import requests  # noqa: WPS433
    except ImportError:
        # Stdlib fallback so the checker still works in minimal envs.
        try:
            from urllib import error as urlerror
            from urllib import request as urlrequest

            req = urlrequest.Request(url, headers={"User-Agent": "elai-registry-check/1.0"})
            with urlrequest.urlopen(req, timeout=timeout) as resp:  # noqa: S310
                body = resp.read().decode("utf-8", errors="replace")
                try:
                    return resp.status, json.loads(body), ""
                except json.JSONDecodeError as e:
                    return resp.status, None, f"json decode: {e}"
        except urlerror.HTTPError as e:
            return e.code, None, f"http {e.code}: {e.reason}"
        except (urlerror.URLError, TimeoutError, OSError) as e:
            return 0, None, f"transport: {e}"

    try:
        resp = requests.get(url, timeout=timeout, headers={"User-Agent": "elai-registry-check/1.0"})
        try:
            return resp.status_code, resp.json(), ""
        except ValueError as e:
            return resp.status_code, None, f"json decode: {e}"
    except requests.RequestException as e:
        return 0, None, f"transport: {e}"


def _check_pypi(pkg: str, max_age_days: int) -> tuple[str, str]:
    """Return (result, reason) for a PyPI registry query."""
    url = f"https://pypi.org/pypi/{pkg}/json"
    status, body, err = _http_get_json(url)
    if status == 404:
        return "fail", f"pypi 404: package '{pkg}' does not exist (likely slopsquat)"
    if status != 200 or body is None:
        return "inapplicable", f"pypi query failed (status={status}): {err}"

    info = body.get("info") or {}
    latest = info.get("version", "")
    if not latest:
        return "fail", f"pypi: '{pkg}' has no 'latest' version"

    # Pull the upload time of the latest release.
    releases = body.get("releases") or {}
    latest_files = releases.get(latest) or []
    if not latest_files:
        return "fail", f"pypi: '{pkg}' has version {latest} but no release files"
    upload_str = latest_files[0].get("upload_time_iso_8601") or latest_files[0].get("upload_time", "")
    upload_dt = _isoparse(upload_str)
    if upload_dt is None:
        # Existence is established; age check is best-effort.
        return "pass", f"pypi: '{pkg}'=={latest} exists (publish date unparseable)"
    age_days = (datetime.now(timezone.utc) - upload_dt).days
    if age_days > max_age_days:
        return "fail", f"pypi: '{pkg}'=={latest} last published {age_days}d ago (>{max_age_days}d stale)"
    return "pass", f"pypi: '{pkg}'=={latest} published {age_days}d ago"


def _check_npm(pkg: str, max_age_days: int) -> tuple[str, str]:
    """Return (result, reason) for an npm registry query."""
    url = f"https://registry.npmjs.org/{pkg}"
    status, body, err = _http_get_json(url)
    if status == 404:
        return "fail", f"npm 404: package '{pkg}' does not exist (likely slopsquat)"
    if status != 200 or body is None:
        return "inapplicable", f"npm query failed (status={status}): {err}"

    dist_tags = body.get("dist-tags") or {}
    latest = dist_tags.get("latest", "")
    if not latest:
        return "fail", f"npm: '{pkg}' has no 'latest' dist-tag"

    time_map = body.get("time") or {}
    upload_str = time_map.get(latest, "")
    upload_dt = _isoparse(upload_str)
    if upload_dt is None:
        return "pass", f"npm: '{pkg}'@{latest} exists (publish date unparseable)"
    age_days = (datetime.now(timezone.utc) - upload_dt).days
    if age_days > max_age_days:
        return "fail", f"npm: '{pkg}'@{latest} last published {age_days}d ago (>{max_age_days}d stale)"
    return "pass", f"npm: '{pkg}'@{latest} published {age_days}d ago"


def _check_cargo(pkg: str, max_age_days: int) -> tuple[str, str]:
    """Return (result, reason) for a crates.io registry query."""
    url = f"https://crates.io/api/v1/crates/{pkg}"
    status, body, err = _http_get_json(url)
    if status == 404:
        return "fail", f"cargo 404: crate '{pkg}' does not exist (likely slopsquat)"
    if status != 200 or body is None:
        return "inapplicable", f"crates.io query failed (status={status}): {err}"

    crate_info = body.get("crate") or {}
    updated_str = crate_info.get("updated_at", "")
    if not updated_str:
        # Existence confirmed even if timestamp absent.
        return "pass", f"cargo: crate '{pkg}' exists (updated_at unparseable)"
    updated_dt = _isoparse(updated_str)
    if updated_dt is None:
        return "pass", f"cargo: crate '{pkg}' exists (updated_at unparseable)"
    age_days = (datetime.now(timezone.utc) - updated_dt).days
    if age_days > max_age_days:
        return "fail", f"cargo: '{pkg}' last updated {age_days}d ago (>{max_age_days}d stale)"
    return "pass", f"cargo: '{pkg}' last updated {age_days}d ago"


# Per-ecosystem default maximum-age thresholds (days).
# pypi/npm: 2 years (any longer and the package is effectively abandoned).
# cargo: 5 years (Rust crates age more gracefully; tokio/serde haven't
#   had a major release in years but are manifestly alive).
_DEFAULT_MAX_AGE_DAYS: dict[str, int] = {
    "pypi": 730,
    "npm": 730,
    "cargo": 1825,
}


def main() -> int:
    try:
        raw = sys.stdin.read()
    except (OSError, ValueError) as e:
        print(json.dumps({"result": "inapplicable", "reason": f"stdin read failed: {e}"}))
        return 0

    if not raw.strip():
        print(json.dumps({"result": "inapplicable", "reason": "empty stdin"}))
        return 0

    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as e:
        print(json.dumps({"result": "inapplicable", "reason": f"stdin not json: {e}"}))
        return 0

    pkg = payload.get("claim", "")
    if not isinstance(pkg, str) or not pkg.strip():
        print(json.dumps({"result": "inapplicable", "reason": "claim (package name) is empty"}))
        return 0
    pkg = pkg.strip()

    candidate = payload.get("candidate", {})
    if isinstance(candidate, str):
        ecosystem = candidate
        max_age_days = _DEFAULT_MAX_AGE_DAYS.get(ecosystem, 730)
    elif isinstance(candidate, dict):
        ecosystem = candidate.get("ecosystem", "pypi")
        default_age = _DEFAULT_MAX_AGE_DAYS.get(ecosystem, 730)
        try:
            max_age_days = int(candidate.get("max_age_days", default_age))
        except (TypeError, ValueError):
            max_age_days = default_age
    else:
        ecosystem = "pypi"
        max_age_days = _DEFAULT_MAX_AGE_DAYS["pypi"]

    if ecosystem == "pypi":
        result, reason = _check_pypi(pkg, max_age_days)
    elif ecosystem == "npm":
        result, reason = _check_npm(pkg, max_age_days)
    elif ecosystem == "cargo":
        result, reason = _check_cargo(pkg, max_age_days)
    else:
        result = "inapplicable"
        reason = f"unsupported ecosystem: '{ecosystem}' (want pypi|npm|cargo)"

    print(json.dumps({"result": result, "reason": reason}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
