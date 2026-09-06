#!/usr/bin/env python3
"""Expand allowed_writes for SSB action-type registration cascade."""
from __future__ import annotations

_BASES = ("crates/orchestrator/", ".elai_cc/crates/orchestrator/")
_CORE = (
    "src/ssb/action_type.rs",
    "src/ssb/payloads.rs",
    "src/ssb/schema.rs",
    "src/ssb.rs",
    "src/ssb/tests.rs",
    "tests/action_type_lock.rs",
    "tests/action_type_lock.golden",
)
_TRIGGER = (
    "ssb/action_type",
    "ssb/payloads",
    "ssb/schema",
    "ssb.rs",
    "ssb/tests",
    "action_type_lock",
    "idea_framing",
)


def _both_forms(rel: str) -> list:
    # Do NOT lstrip("./") — that charset form strips the leading '.' from
    # ".elai_cc/..." and collapses it to "elai_cc/...", which then fails the
    # free-lane scope check against real worktree paths.
    rel = rel.replace("\\", "/").strip()
    if rel.startswith("./"):
        rel = rel[2:]
    if rel.startswith(".elai_cc/"):
        return [rel, rel[len(".elai_cc/") :]]
    if rel.startswith("crates/"):
        return [rel, f".elai_cc/{rel}"]
    return [rel]


def expand_allowed_writes(paths):
    raw = [p for p in (paths or []) if p and str(p).strip()]
    out, seen = [], set()

    def add(p: str) -> None:
        p = p.replace("\\", "/").strip()
        if p and p not in seen:
            seen.add(p)
            out.append(p)

    for p in raw:
        for f in _both_forms(p):
            add(f)
    if any(any(t in p for t in _TRIGGER) for p in list(out)):
        for base in _BASES:
            for core in _CORE:
                add(base + core)
            add(base + "src/idea_framing.rs")
            add(base + "tests/idea_framing_unit.rs")
    return out


def elai_cc_path_preamble() -> str:
    # Codex lane only: worktree root is monorepo wrapper; cargo/product live under .elai_cc/.
    return (
        "PATH LAYOUT (binding, codex): product sources live under `.elai_cc/` in the worktree; "
        "run cargo from that root (cd .elai_cc). Prefer `.elai_cc/crates/...` write paths "
        "(bare `crates/` is also rewritten by expand_allowed_writes)."
    )
