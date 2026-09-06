#!/usr/bin/env python3
"""Ledger of codex worktrees awaiting off-agent land."""
from __future__ import annotations

import json
import os
import time
from pathlib import Path

LEDGER = Path(__file__).resolve().parent / "pending-lands.json"

RECORD_VERSION = 2
LEASE_SECONDS = 900
REPLAY_ARG_FIELDS = (lambda _ast, _source: tuple(sorted({
    next(
        (
            keyword.value.value
            for keyword in node.keywords
            if keyword.arg == "dest"
            and isinstance(keyword.value, _ast.Constant)
            and isinstance(keyword.value.value, str)
        ),
        None,
    )
    or next(
        (
            argument.value.lstrip("-").replace("-", "_")
            for argument in node.args
            if isinstance(argument, _ast.Constant)
            and isinstance(argument.value, str)
            and argument.value.startswith("--")
        ),
        "",
    )
    for node in _ast.walk(_ast.parse(_source))
    if isinstance(node, _ast.Call)
    and isinstance(node.func, _ast.Attribute)
    and node.func.attr == "add_argument"
}))) (
    __import__("ast"),
    Path(__file__).with_name("opus_land.py").read_text(encoding="utf-8"),
)
def _load(path=LEDGER):
    p = Path(path)
    try:
        data = json.loads(p.read_text(encoding="utf-8"))
        if isinstance(data, dict) and isinstance(data.get("lands"), list):
            return data
    except (OSError, json.JSONDecodeError, TypeError):
        pass
    return {"schema_version": 1, "lands": []}


def _save(data, path=LEDGER):
    p = Path(path)
    p.parent.mkdir(parents=True, exist_ok=True)
    tmp = p.with_name(p.name + f".{os.getpid()}.tmp")
    tmp.write_text(json.dumps(data, indent=2, sort_keys=True), encoding="utf-8")
    os.replace(tmp, p)


def add(plan_id, worktree, artifact_dir="", path=LEDGER, reason="", replay_args=None,
        replay_argv=None, args=None, plan_spec_snapshot=None, plan_spec_sha256=None):
    if not plan_id or not worktree:
        return None
    import hashlib
    import fcntl
    import inspect
    import uuid

    p = Path(path)
    p.parent.mkdir(parents=True, exist_ok=True)
    lock_path = p.with_name(p.name + ".lock")
    with lock_path.open("a+", encoding="utf-8") as lock:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
        try:
            try:
                current = json.loads(p.read_text(encoding="utf-8")) if p.is_file() else {}
            except (OSError, json.JSONDecodeError, TypeError):
                current = {}
            if not isinstance(current, dict) or not isinstance(current.get("lands"), list):
                current = {"schema_version": RECORD_VERSION, "lands": []}
            lands = list(current["lands"])
            if args is not None and replay_args is None:
                replay_args = vars(args) if hasattr(args, "__dict__") else args
            replay_args = dict(replay_args or {})
            serializable = True

            def safe_value(value):
                nonlocal serializable
                if value is None or isinstance(value, (bool, int, float, str)):
                    return value
                if isinstance(value, (list, tuple)):
                    return [safe_value(item) for item in value]
                if isinstance(value, dict):
                    return {str(key): safe_value(item) for key, item in value.items()}
                serializable = False
                return {"__u3a_non_serializable__": type(value).__name__}

            replay_args = {str(key): safe_value(value) for key, value in replay_args.items()}
            if replay_argv is None:
                frame = inspect.currentframe()
                candidate = frame.f_back if frame is not None else None
                while candidate is not None:
                    possible_argv = candidate.f_locals.get("argv")
                    if isinstance(possible_argv, list):
                        replay_argv = list(possible_argv)
                        break
                    candidate = candidate.f_back
                del candidate, frame
            if replay_argv is None:
                replay_argv = list(__import__("sys").argv[1:])
            safe_argv = []
            for item in replay_argv:
                if isinstance(item, str):
                    safe_argv.append(item)
                else:
                    serializable = False
                    safe_argv.append(f"<non-serializable:{type(item).__name__}>")
            replay_argv = safe_argv
            snapshot = plan_spec_snapshot
            snapshot_hash = plan_spec_sha256
            spec_path = replay_args.get("plan_spec")
            if snapshot is None and isinstance(spec_path, str) and spec_path:
                try:
                    snapshot = Path(spec_path).read_text(encoding="utf-8")
                except (OSError, UnicodeDecodeError):
                    snapshot = None
            if snapshot is not None and not isinstance(snapshot, str):
                serializable = False
                snapshot = f"<non-serializable:{type(snapshot).__name__}>"
                snapshot_hash = None
            if snapshot is not None and snapshot_hash is None:
                snapshot_hash = hashlib.sha256(snapshot.encode("utf-8")).hexdigest()
            entry = {
                "record_version": RECORD_VERSION,
                "plan_id": plan_id,
                "worktree": str(worktree),
                "artifact_dir": artifact_dir or "",
                "added_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                "reason": reason or "",
                "replay_args": replay_args,
                "replay_argv": replay_argv,
                "plan_spec_snapshot": snapshot,
                "plan_spec_sha256": snapshot_hash,
                "args_sha256": None,
                "claim_token": None,
                "replayable": False,
                "refusal_reason": "pending_validation",
            }
            for field in REPLAY_ARG_FIELDS:
                if field in replay_args:
                    entry[field] = replay_args[field]
            try:
                entry["args_sha256"] = _canonical_args_sha256(replay_args)
                json.dumps(replay_args, sort_keys=True, separators=(",", ":"))
            except (TypeError, ValueError, OverflowError):
                entry["refusal_reason"] = "non_serializable_argument"
            else:
                if not serializable:
                    entry["refusal_reason"] = "non_serializable_argument"
                else:
                    unknown = sorted(set(replay_args) - set(REPLAY_ARG_FIELDS))
                    missing = sorted(set(REPLAY_ARG_FIELDS) - set(replay_args))
                    if unknown:
                        entry["refusal_reason"] = "unknown_replay_argument"
                        entry["unknown_fields"] = unknown
                    elif missing:
                        entry["refusal_reason"] = "missing_replay_argument"
                        entry["missing_fields"] = missing
                    elif replay_args.get("gates_only") is True or replay_args.get("no_verifier") is True:
                        entry["refusal_reason"] = "disallowed_replay_flag"
                    elif replay_args.get("gates_only") is not False or replay_args.get("no_verifier") is not False:
                        entry["refusal_reason"] = "invalid_replay_flag"
                    else:
                        entry["replayable"] = True
                        entry["refusal_reason"] = ""
            matching = [
                index for index, row in enumerate(lands)
                if isinstance(row, dict)
                and row.get("plan_id") == plan_id
                and row.get("worktree") == str(worktree)
            ]
            if matching:
                first = matching[0]
                lands[first] = entry
                lands = [row for index, row in enumerate(lands) if index == first or index not in matching[1:]]
            else:
                lands.append(entry)
            current["schema_version"] = RECORD_VERSION
            current["lands"] = lands
            tmp = p.with_name(p.name + f".{os.getpid()}.{uuid.uuid4().hex}.tmp")
            tmp.write_text(json.dumps(current, indent=2, sort_keys=True), encoding="utf-8")
            os.replace(tmp, p)
        finally:
            fcntl.flock(lock.fileno(), fcntl.LOCK_UN)
    return entry


def _canonical_args_sha256(args):
    import hashlib
    canonical = json.dumps(args, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def load_rows(path=LEDGER):
    p = Path(path)
    try:
        data = json.loads(p.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, TypeError):
        return []
    if not isinstance(data, dict) or not isinstance(data.get("lands"), list):
        return []
    import hashlib
    import importlib
    import sys

    def live_surface():
        argparse = importlib.import_module("argparse")
        module = sys.modules.get("opus_land")
        if module is None:
            module = importlib.import_module("opus_land")
        import types
        original_module = module.argparse
        original_parser = argparse.ArgumentParser
        captured = []

        class ParserProbe:
            def __init__(self, parser):
                self._parser = parser

            def add_argument(self, *argv, **kwargs):
                return self._parser.add_argument(*argv, **kwargs)

            @property
            def _actions(self):
                return self._parser._actions

            def parse_args(self, *argv, **kwargs):
                raise RuntimeError("parser_surface_capture")

        def capture(*argv, **kwargs):
            parser = ParserProbe(original_parser(*argv, **kwargs))
            captured.append(parser)
            return parser

        module.argparse = types.SimpleNamespace(ArgumentParser=capture)
        try:
            try:
                module.main([])
            except RuntimeError as exc:
                if str(exc) != "parser_surface_capture":
                    raise
        finally:
            module.argparse = original_module
        if not captured:
            return ()
        return tuple(sorted(action.dest for action in captured[-1]._actions if action.dest != "help"))

    def refusal(row, reason):
        copy = dict(row)
        copy["replayable"] = False
        copy["refusal_reason"] = reason
        return copy

    surface = live_surface()
    output = []
    for original in data["lands"]:
        if not isinstance(original, dict):
            output.append({"replayable": False, "refusal_reason": "invalid_row"})
            continue
        row = dict(original)
        if row.get("record_version") != RECORD_VERSION:
            output.append(refusal(row, "legacy_v1_schema"))
            continue
        if surface != tuple(sorted(REPLAY_ARG_FIELDS)):
            output.append(refusal(row, "parser_surface_drift"))
            continue
        replay_args = row.get("replay_args")
        if replay_args is None:
            replay_args = {field: row[field] for field in REPLAY_ARG_FIELDS if field in row}
        if not isinstance(replay_args, dict):
            output.append(refusal(row, "non_serializable_argument"))
            continue
        unknown = sorted(set(replay_args) - set(REPLAY_ARG_FIELDS))
        if unknown:
            copy = refusal(row, "unknown_replay_argument")
            copy["unknown_fields"] = unknown
            output.append(copy)
            continue
        missing = sorted(set(REPLAY_ARG_FIELDS) - set(replay_args))
        if missing:
            copy = refusal(row, "missing_replay_argument")
            copy["missing_fields"] = missing
            output.append(copy)
            continue
        try:
            encoded = json.dumps(replay_args, sort_keys=True, separators=(",", ":"))
        except (TypeError, ValueError, OverflowError):
            output.append(refusal(row, "non_serializable_argument"))
            continue
        def contains_marker(value):
            if isinstance(value, dict):
                return "__u3a_non_serializable__" in value or any(contains_marker(item) for item in value.values())
            if isinstance(value, list):
                return any(contains_marker(item) for item in value)
            return False

        if contains_marker(replay_args):
            output.append(refusal(row, "non_serializable_argument"))
            continue
        if any(field in row and row[field] != replay_args[field] for field in REPLAY_ARG_FIELDS):
            output.append(refusal(row, "args_hash_mismatch"))
            continue
        replay_argv = row.get("replay_argv")
        if not isinstance(replay_argv, list) or not all(isinstance(item, str) for item in replay_argv):
            output.append(refusal(row, "non_serializable_argument"))
            continue
        if replay_args.get("gates_only") is True or replay_args.get("no_verifier") is True:
            output.append(refusal(row, "disallowed_replay_flag"))
            continue
        if replay_args.get("gates_only") is not False or replay_args.get("no_verifier") is not False:
            output.append(refusal(row, "invalid_replay_flag"))
            continue
        actual_hash = _canonical_args_sha256(replay_args)
        if row.get("args_sha256") != actual_hash:
            output.append(refusal(row, "args_hash_mismatch"))
            continue
        snapshot = row.get("plan_spec_snapshot")
        snapshot_hash = row.get("plan_spec_sha256")
        if replay_args.get("plan_spec") is not None and not isinstance(snapshot, str):
            output.append(refusal(row, "missing_plan_spec_snapshot"))
            continue
        if snapshot is not None:
            try:
                expected_snapshot_hash = hashlib.sha256(snapshot.encode("utf-8")).hexdigest()
            except (AttributeError, TypeError, UnicodeError):
                output.append(refusal(row, "non_serializable_argument"))
                continue
            if snapshot_hash != expected_snapshot_hash:
                output.append(refusal(row, "plan_spec_hash_mismatch"))
                continue
        row["replay_args"] = dict(replay_args)
        row["replayable"] = True
        row["refusal_reason"] = ""
        output.append(row)
    return output


def claim_row(row_or_plan_id, claimed_by=None, path=LEDGER, worktree="", now=None,
              claim_token=None):
    import datetime
    import fcntl
    import uuid

    p = Path(path)
    p.parent.mkdir(parents=True, exist_ok=True)
    requested = row_or_plan_id if isinstance(row_or_plan_id, dict) else None
    plan_id = requested.get("plan_id") if requested is not None else row_or_plan_id
    if requested is not None:
        worktree = requested.get("worktree", worktree)
        claim_token = requested.get("claim_token", claim_token)
    worker = claimed_by or (requested or {}).get("claimed_by")
    if not plan_id or not worker:
        return None

    def epoch(value):
        if value is None:
            return time.time()
        if isinstance(value, (int, float)) and not isinstance(value, bool):
            return float(value)
        if isinstance(value, str):
            text = value.replace("Z", "+00:00")
            return datetime.datetime.fromisoformat(text).timestamp()
        return float(value.timestamp())

    def iso(value):
        return datetime.datetime.fromtimestamp(value, datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    current_time = epoch(now)
    lock_path = p.with_name(p.name + ".lock")
    with lock_path.open("a+", encoding="utf-8") as lock:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
        try:
            rows = load_rows(p)
            target_index = None
            for index, row in enumerate(rows):
                if row.get("plan_id") == plan_id and (not worktree or row.get("worktree") == worktree):
                    target_index = index
                    break
            if target_index is None:
                return None
            target = rows[target_index]
            if not target.get("replayable"):
                return None
            existing = target.get("claim_token")
            if existing:
                claimed_at = target.get("claimed_at_iso")
                try:
                    expired = current_time > epoch(claimed_at) + LEASE_SECONDS
                except (TypeError, ValueError, OverflowError):
                    expired = False
                if not expired and claim_token != existing:
                    return None
            token = uuid.uuid4().hex
            target = dict(target)
            target.update({
                "claim_token": token,
                "claimed_by": str(worker),
                "claimed_at_iso": iso(current_time),
                "lease_seconds": LEASE_SECONDS,
            })
            rows[target_index] = target
            data = {"schema_version": RECORD_VERSION, "lands": rows}
            tmp = p.with_name(p.name + f".{os.getpid()}.{token}.tmp")
            tmp.write_text(json.dumps(data, indent=2, sort_keys=True), encoding="utf-8")
            os.replace(tmp, p)
            return target
        finally:
            fcntl.flock(lock.fileno(), fcntl.LOCK_UN)


def release_claim(row_or_plan_id, claim_token=None, path=LEDGER, worktree="", claimed_by=None):
    import fcntl

    p = Path(path)
    requested = row_or_plan_id if isinstance(row_or_plan_id, dict) else None
    plan_id = requested.get("plan_id") if requested is not None else row_or_plan_id
    if requested is not None:
        worktree = requested.get("worktree", worktree)
        claim_token = requested.get("claim_token", claim_token)
        claimed_by = requested.get("claimed_by", claimed_by)
    if not plan_id:
        return None
    lock_path = p.with_name(p.name + ".lock")
    with lock_path.open("a+", encoding="utf-8") as lock:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
        try:
            rows = load_rows(p)
            for index, original in enumerate(rows):
                if original.get("plan_id") != plan_id or (worktree and original.get("worktree") != worktree):
                    continue
                current = original.get("claim_token")
                if current and claim_token != current:
                    return None
                if claimed_by is not None and original.get("claimed_by") != claimed_by:
                    return None
                released = dict(original)
                for key in ("claim_token", "claimed_by", "claimed_at_iso", "lease_seconds"):
                    released.pop(key, None)
                rows[index] = released
                data = {"schema_version": RECORD_VERSION, "lands": rows}
                tmp = p.with_name(p.name + f".{os.getpid()}.release.tmp")
                tmp.write_text(json.dumps(data, indent=2, sort_keys=True), encoding="utf-8")
                os.replace(tmp, p)
                return released
            return None
        finally:
            fcntl.flock(lock.fileno(), fcntl.LOCK_UN)


def list_waiting_for_verifier(path=LEDGER):
    """Resume set: lands parked because the preland verifier was unavailable.

    Re-run opus_land on each worktree when the verifier seat is back (no re-execute).
    """
    return [
        x
        for x in list_lands(path)
        if (x.get("reason") or "") == "WAITING_FOR_VERIFIER"
    ]


def remove(plan_id="", worktree="", path=LEDGER):
    data = _load(path)
    before = len(data["lands"])
    lands = []
    for x in data["lands"]:
        if plan_id and x.get("plan_id") == plan_id:
            continue
        if worktree and x.get("worktree") == worktree:
            continue
        lands.append(x)
    data["lands"] = lands
    _save(data, path)
    return before - len(lands)


def list_lands(path=LEDGER):
    return list(_load(path).get("lands") or [])


def protected_plan_ids(path=LEDGER):
    return {x.get("plan_id") for x in list_lands(path) if x.get("plan_id")}
