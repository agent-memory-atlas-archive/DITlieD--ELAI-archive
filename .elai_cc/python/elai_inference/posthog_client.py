"""Shared PostHog analytics client for the ELAI inference sidecar.

Initialised once at import time using POSTHOG_PROJECT_TOKEN and POSTHOG_HOST
environment variables.  All callers import `posthog_client` and `server_id`
from this module.

If POSTHOG_PROJECT_TOKEN is absent the module still loads; every public
function becomes a no-op so the sidecar continues working without analytics.
In non-production environments the absence is surfaced as a loud warning so
operators notice the misconfiguration before it silently drops events.
"""

from __future__ import annotations

import atexit
import logging
import os
import uuid

logger = logging.getLogger("elai.posthog")

# Stable identifier for this sidecar process instance.  Used as distinct_id
# on every event so PostHog can correlate activity across a single run.
server_id: str = f"elai-sidecar-{uuid.uuid4().hex[:12]}"

_token = os.getenv("POSTHOG_PROJECT_TOKEN", "")
_host = os.getenv("POSTHOG_HOST", "https://us.i.posthog.com")

posthog_client = None

if _token:
    try:
        from posthog import Posthog as _Posthog

        posthog_client = _Posthog(
            _token,
            host=_host,
            enable_exception_autocapture=True,
        )
        atexit.register(posthog_client.shutdown)
    except Exception as _init_err:  # pragma: no cover
        logger.warning("posthog_client_init_failed: %s", _init_err)
        posthog_client = None
else:
    _env = os.getenv("ELAI_ENV", "development").lower()
    if _env not in ("production", "prod"):
        logger.warning(
            "POSTHOG_PROJECT_TOKEN variable required by PostHog is missing or "
            "un-configured, this causes events to be silently missed. "
            "This error stops appearing once POSTHOG_PROJECT_TOKEN is configured"
        )


def capture(event: str, properties: dict | None = None) -> None:
    """Capture a PostHog event using the sidecar server_id as distinct_id."""
    if posthog_client is None:
        return
    try:
        posthog_client.capture(
            distinct_id=server_id,
            event=event,
            properties=properties or {},
        )
    except Exception:
        pass
