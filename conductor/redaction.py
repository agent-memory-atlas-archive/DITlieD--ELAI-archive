from __future__ import annotations

import re
from collections.abc import Mapping, Sequence
from typing import Any


SECRET_ASSIGNMENT_RE = re.compile(
    r"(?i)\b(api[_-]?key|access[_-]?token|auth[_-]?token|bearer|credential|"
    r"password|secret)\b\s*[:=]\s*([^\s,;\"']+)"
)
BEARER_RE = re.compile(r"(?i)\bbearer\s+[A-Za-z0-9._~+/=-]{12,}")
LONG_TOKEN_RE = re.compile(r"\b[A-Za-z0-9_./+=-]{40,}\b")


def redact_text(value: str) -> str:
    redacted = SECRET_ASSIGNMENT_RE.sub(lambda m: f"{m.group(1)}=<REDACTED>", value)
    redacted = BEARER_RE.sub("Bearer <REDACTED>", redacted)
    return LONG_TOKEN_RE.sub("<REDACTED_TOKEN>", redacted)


def redact_json(value: Any) -> Any:
    if isinstance(value, str):
        return redact_text(value)
    if isinstance(value, Mapping):
        result: dict[str, Any] = {}
        for key, item in value.items():
            if re.search(r"(?i)(token|secret|password|credential|api[_-]?key)", str(key)):
                result[str(key)] = "<REDACTED>"
            else:
                result[str(key)] = redact_json(item)
        return result
    if isinstance(value, Sequence) and not isinstance(value, (bytes, bytearray)):
        return [redact_json(item) for item in value]
    return value
