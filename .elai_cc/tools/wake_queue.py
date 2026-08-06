#!/usr/bin/env python3
"""Crash-safe, coalescing wake queue for supervision handoffs."""
from __future__ import annotations

import argparse
import json
import os
import time
from contextlib import contextmanager
from pathlib import Path
from typing import Iterator

DEFAULT_PATH = ".<REDACTED_TOKEN>"
FM_SIGNAL_GRACE = 30.0

@contextmanager
def _lock(path: Path) -> Iterator[None]:
    lock = path.with_name("wake_queue.lock")
    lock.parent.mkdir(parents=True, exist_ok=True)
    fd = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
    try:
        os.write(fd, f"{os.getpid()} {time.time()}\n".encode())
        os.close(fd)
        yield
    finally:
        try:
            lock.unlink()
        except FileNotFoundError:
            pass

def _path(path: str | None) -> Path:
    return Path(path or DEFAULT_PATH)

def _load(path: Path) -> list[dict]:
    try:
        value = json.loads(path.read_text())
    except (FileNotFoundError, json.JSONDecodeError):
        return []
    return value if isinstance(value, list) else []

def _save(path: Path, items: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(path.name + f".{os.getpid()}.tmp")
    tmp.write_text(json.dumps(items, sort_keys=True, separators=(",", ":")) + "\n")
    os.replace(tmp, path)

def enqueue(item: dict, path: str | None = None) -> None:
    """Add a wake item, coalescing a recent item with the same key."""
    target = _path(path)
    now = time.time()
    record = dict(item)
    record.setdefault("enqueued_ts", now)
    if not record.get("handoff_topic") or not record.get("plan_id") or not record.get("reason"):
        raise ValueError("wake item requires handoff_topic, plan_id, and reason")
    record.setdefault("key", f"{record['plan_id']}:{record['handoff_topic']}")
    with _lock(target):
        items = _load(target)
        for old in items:
            if old.get("key") == record["key"] and now - float(old.get("enqueued_ts", 0)) <= FM_SIGNAL_GRACE:
                # Coalesce the signal: refresh its age but retain the original payload.
                old["enqueued_ts"] = now
                _save(target, items)
                return
        items.append(record)
        _save(target, items)

def dequeue_ready(now: float | None = None, grace_s: float = 30.0) -> list[dict]:
    """Remove and return items old enough for delivery."""
    target = _path(None)
    moment = time.time() if now is None else now
    with _lock(target):
        items = _load(target)
        ready = [x for x in items if moment - float(x.get("enqueued_ts", 0)) >= grace_s]
        _save(target, [x for x in items if x not in ready])
    return ready

def list_pending(path: str | None = None) -> list[dict]:
    """Return pending items without changing the queue."""
    return _load(_path(path))

def clear(item_id: str, path: str | None = None) -> bool:
    """Delete an item by its id, key, or handoff topic."""
    target = _path(path)
    with _lock(target):
        items = _load(target)
        remaining = [x for x in items if str(x.get("id", x.get("key", ""))) != item_id and x.get("key") != item_id]
        changed = len(remaining) != len(items)
        if changed:
            _save(target, remaining)
        return changed

def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    put = sub.add_parser("enqueue"); put.add_argument("item", help="JSON object"); put.add_argument("--path")
    show = sub.add_parser("list"); show.add_argument("--path")
    take = sub.add_parser("dequeue"); take.add_argument("--grace", type=float, default=30.0)
    remove = sub.add_parser("clear"); remove.add_argument("item_id"); remove.add_argument("--path")
    args = parser.parse_args(argv)
    if args.command == "enqueue": enqueue(json.loads(args.item), args.path); print(json.dumps({"status":"enqueued"}))
    elif args.command == "list": print(json.dumps(list_pending(args.path), sort_keys=True))
    elif args.command == "dequeue": print(json.dumps(dequeue_ready(grace_s=args.grace), sort_keys=True))
    else: print(json.dumps({"cleared": clear(args.item_id, args.path)}))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
