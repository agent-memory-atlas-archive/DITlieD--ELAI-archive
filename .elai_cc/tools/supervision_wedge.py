#!/usr/bin/env python3
"""Deterministic detector for an external worker whose output has stopped changing."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable
from glob import glob

@dataclass(frozen=True)
class OutputSignature:
    path: str
    size: int
    mtime_ns: int
    content_hash: str

def signature_of(path: str, tail_bytes: int = 65536) -> OutputSignature:
    """Return metadata and a digest of the output tail (or the whole small file)."""
    p = Path(path)
    st = p.stat()
    with p.open("rb") as stream:
        if st.st_size > tail_bytes:
            stream.seek(-tail_bytes, os.SEEK_END)
        data = stream.read()
    digest = hashlib.blake2b(data, digest_size=32).hexdigest()
    return OutputSignature(str(p), st.st_size, st.st_mtime_ns, digest)

def is_wedge(history: list[OutputSignature], n_cycles: int = 3,
             busy_globs: list[str] | None = None) -> bool:
    """True if the last *n_cycles* observations match and no busy marker exists."""
    if n_cycles < 1 or len(history) < n_cycles:
        return False
    if busy_globs and any(os.path.exists(match) for pattern in busy_globs for match in glob(pattern)):
        return False
    recent = history[-n_cycles:]
    return len({(s.size, s.mtime_ns, s.content_hash) for s in recent}) == 1

def _read_state(path: str) -> list[OutputSignature]:
    try:
        raw = json.loads(Path(path).read_text())
    except (FileNotFoundError, json.JSONDecodeError):
        return []
    return [OutputSignature(**entry) for entry in raw if isinstance(entry, dict)]

def _write_state(path: str, history: Iterable[OutputSignature]) -> None:
    target = Path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    tmp = target.with_name(target.name + ".tmp")
    tmp.write_text(json.dumps([asdict(s) for s in history], sort_keys=True) + "\n")
    os.replace(tmp, target)

def _record(path: str, state: str) -> list[OutputSignature]:
    history = _read_state(state)
    history.append(signature_of(path))
    history = history[-32:]
    _write_state(state, history)
    return history

def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    for name in ("record", "check"):
        cmd = sub.add_parser(name)
        cmd.add_argument("--path", required=True)
        cmd.add_argument("--state", required=True)
        if name == "check": cmd.add_argument("--n", type=int, default=3)
    args = parser.parse_args(argv)
    history = _record(args.path, args.state)
    if args.command == "check" and is_wedge(history, args.n):
        print(json.dumps({"verdict": "WEDGE", "path": args.path, "cycles": args.n}, sort_keys=True))
        return 2
    if args.command == "check":
        print(json.dumps({"verdict": "OK", "path": args.path, "cycles": args.n}, sort_keys=True))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
