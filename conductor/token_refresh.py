"""token_refresh.py - keep a claude.ai OAuth credentials file fresh FOREVER (no re-login).

Headless base-cc (`--print`, the opus-orchestrator worker) does NOT auto-refresh its OAuth token, so
once .claude-elai's accessToken expires the walk 401s and the owner has to re-login by hand. This
refreshes the token from the stored refreshToken against the SAME endpoint + client_id base-cc itself
uses -- grounded in base-cc/dist/cli.js (TOKEN_URL https://platform.claude.com/v1/oauth/token,
CLIENT_ID 9d1c250a-..., body {grant_type:"refresh_token", refresh_token, client_id}; the response
maps expiresAt = Date.now() + expires_in*1000 and ROTATES the refresh token). We mirror that exactly.

Rotation-aware (writes the NEW refresh_token back or the next refresh fails), atomic write (tmp+rename,
never a torn creds file), near-expiry-gated (only calls the network when the token is actually close to
expiry, so a 20-min timer is cheap), preserves every other field (mcpOAuth / organizationUuid / scopes /
rateLimitTier). NEVER raises -- a failure returns {ok:false, reason} so the caller degrades honestly.

Owned lane: .claude-elai (the walk's dedicated MAX account; nothing interactive competes for it). The
main .claude account is refreshed by the interactive base-cc that holds it -- do NOT fight that here.
Stdlib only.
"""
from __future__ import annotations

import json
import os
import time
import urllib.error
import urllib.request
from pathlib import Path

TOKEN_URL = "https://platform.claude.com/v1/oauth/token"
CLIENT_ID = "9d1c250a-e61b-44d9-88ed-5944d1962f5e"  # Claude Code OAuth client (base-cc/dist/cli.js)
GRANT_TYPE_REFRESH = "refresh_token"
DEFAULT_NEAR_EXPIRY_SECS = 900   # refresh when under 15 min of life remains
HTTP_TIMEOUT_SECS = 20

# named accounts -> credentials path. `elai` is the walk lane this module owns.
ACCOUNTS = {
    "elai": "/home/researcher/.claude-elai/.credentials.json",
    "max": "/home/researcher/.claude/.credentials.json",
}


def _now_ms() -> int:
    return int(time.time() * 1000)


def _atomic_write(path: Path, data: dict) -> None:
    """Write JSON via tmp+rename so a crash never leaves a half-written creds file."""
    tmp = path.with_name(path.name + ".tmp")
    tmp.write_text(json.dumps(data, indent=2), encoding="utf-8")
    os.replace(tmp, path)


def _post_refresh(refresh_token: str) -> dict:
    """POST the refresh grant, return the parsed token response. Raises on transport/HTTP error."""
    body = json.dumps({
        "grant_type": GRANT_TYPE_REFRESH,
        "refresh_token": refresh_token,
        "client_id": CLIENT_ID,
    }).encode("utf-8")
    req = urllib.request.Request(
        TOKEN_URL, data=body, method="POST",
        headers={"Content-Type": "application/json", "Accept": "application/json",
                 "User-Agent": "teikoku-token-refresh/1.0"},
    )
    with urllib.request.urlopen(req, timeout=HTTP_TIMEOUT_SECS) as resp:  # noqa: S310 - pinned https endpoint
        return json.loads(resp.read().decode("utf-8", errors="replace"))


def refresh_if_needed(creds_path: str | Path, *, near_expiry_secs: int = DEFAULT_NEAR_EXPIRY_SECS,
                      force: bool = False) -> dict:
    """Refresh the claude.ai OAuth token in creds_path when it is within near_expiry_secs of expiry
    (or force=True). Returns a status dict; NEVER raises. On success writes the new access + rotated
    refresh token + recomputed expiresAt back atomically, leaving all other fields untouched."""
    path = Path(creds_path)
    try:
        creds = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        return {"ok": False, "reason": f"read: {type(exc).__name__}: {exc}"}
    oauth = creds.get("claudeAiOauth")
    if not isinstance(oauth, dict):
        return {"ok": False, "reason": "no claudeAiOauth block"}
    refresh_token = oauth.get("refreshToken")
    if not refresh_token:
        return {"ok": False, "reason": "no refreshToken (interactive re-login required once)"}
    try:
        expires_at = int(oauth.get("expiresAt") or 0)
    except (TypeError, ValueError):
        expires_at = 0
    remaining = (expires_at - _now_ms()) / 1000.0
    if not force and remaining > near_expiry_secs:
        return {"ok": True, "refreshed": False, "remaining_secs": int(remaining)}

    try:
        data = _post_refresh(refresh_token)
    except urllib.error.HTTPError as exc:
        detail = ""
        try:
            detail = exc.read().decode("utf-8", "replace")[:200]
        except Exception:  # noqa: BLE001
            pass
        return {"ok": False, "refreshed": False, "reason": f"http {exc.code}: {detail}"}
    except (urllib.error.URLError, TimeoutError, OSError, ValueError) as exc:
        return {"ok": False, "refreshed": False, "reason": f"net: {type(exc).__name__}: {exc}"}

    access = data.get("access_token")
    expires_in = data.get("expires_in")
    # Accept int/float or numeric string; API normally sends a number but string has been observed.
    try:
        expires_in_secs = float(expires_in)
    except (TypeError, ValueError):
        expires_in_secs = None
    if not access or expires_in_secs is None:
        return {"ok": False, "refreshed": False, "reason": f"unexpected response keys: {list(data.keys())}"}
    oauth["accessToken"] = access
    if data.get("refresh_token"):  # rotation: persist the NEW refresh token or the next refresh 401s
        oauth["refreshToken"] = data["refresh_token"]
    oauth["expiresAt"] = _now_ms() + int(expires_in_secs * 1000)
    creds["claudeAiOauth"] = oauth
    try:
        _atomic_write(path, creds)
    except OSError as exc:
        return {"ok": False, "refreshed": False, "reason": f"write: {type(exc).__name__}: {exc}"}
    return {"ok": True, "refreshed": True, "new_expiresAt": oauth["expiresAt"],
            "valid_for_secs": int(expires_in_secs)}


def main(argv: list[str] | None = None) -> int:
    import argparse
    ap = argparse.ArgumentParser(description="refresh a claude.ai OAuth token from its refreshToken")
    ap.add_argument("--account", default="elai", help="elai | max | <path to .credentials.json>")
    ap.add_argument("--force", action="store_true", help="refresh even if not near expiry")
    ap.add_argument("--near-expiry-secs", type=int, default=DEFAULT_NEAR_EXPIRY_SECS)
    args = ap.parse_args(argv)
    path = ACCOUNTS.get(args.account, args.account)
    result = refresh_if_needed(path, near_expiry_secs=args.near_expiry_secs, force=args.force)
    print(json.dumps(result))
    return 0 if result.get("ok") else 1


if __name__ == "__main__":
    import sys
    sys.exit(main())
