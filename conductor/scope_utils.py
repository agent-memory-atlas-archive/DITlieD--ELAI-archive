from __future__ import annotations

from fnmatch import fnmatch
from pathlib import PurePosixPath


FROZEN_GATE_PATTERNS = (
    ".claude/**",
    "**/.claude/**",
    "gate-manifest.txt",
    "gate-manifest.lock",
    "**/gate-manifest.txt",
    "**/gate-manifest.lock",
    ".git/**",
    "**/.git/**",
)


def normalize_repo_path(path: str) -> str:
    text = path.replace("\\", "/").strip()
    while text.startswith("./"):
        text = text[2:]
    if len(text) >= 3 and text[1] == ":" and text[2] == "/":
        text = text[3:]
    return PurePosixPath(text).as_posix().strip("/")


def path_matches_scope(path: str, scope: str) -> bool:
    candidate = normalize_repo_path(path)
    allowed = normalize_repo_path(scope)
    if not allowed:
        return False
    if any(ch in allowed for ch in "*?[]"):
        return fnmatch(candidate, allowed)
    if allowed.endswith("/"):
        allowed = allowed.rstrip("/")
    return candidate == allowed or candidate.startswith(allowed + "/")


def is_frozen_gate_path(path: str) -> bool:
    candidate = normalize_repo_path(path)
    return any(fnmatch(candidate, pattern) for pattern in FROZEN_GATE_PATTERNS)


def path_is_allowed(path: str, allowed_write_set: list[str]) -> bool:
    if is_frozen_gate_path(path):
        return False
    return any(path_matches_scope(path, scope) for scope in allowed_write_set)


def scope_violations(touched_files: list[str], allowed_write_set: list[str]) -> list[str]:
    return sorted(
        normalize_repo_path(path)
        for path in touched_files
        if not path_is_allowed(path, allowed_write_set)
    )
