"""trust_ledger.py - per-task-class autonomy-tier ledger for the opus walk (P1.6).

WHY THIS EXISTS (2026-07-10, opus-orchestrator improvement roadmap): the walk lands plans across
task classes of very different risk. A per-class trust record lets the orchestrator ROUTE autonomy
(watch vs queue vs auto) off measured outcomes instead of a flat policy. This module owns only the
MECHANICAL record + tier read; the actual routing decision stays Opus judgment (SKILL prose, D-10).

The stored truth is COUNTS, never a tier string: every write persists (runs, passes, last_result,
last_ts) and the tier is RECOMPUTED on read from those counts (Loop-04). Promotion ladder:
  new/unknown class      -> watch
  watch  -> queue  at  QUEUE_MIN_RUNS (10)  runs AND >= QUEUE_MIN_RATE (90%) pass
  queue  -> auto   at  AUTO_MIN_RUNS  (20)  runs AND >= AUTO_MIN_RATE  (95%) pass
Any FAIL forces IMMEDIATE demotion one tier below the count-derived base (last_result=fail is a
stored fact, not a stored tier), and emits a stderr ALERT when that fail actually drops the tier.

Storage: trust.json beside walk-state.json - {"schema_version":1,"classes":{<class>:{runs,passes,
last_ts,last_result}}}. Mutex + atomic write reuse walk_state's canonical helpers (same package, same
concurrency contract) rather than a near-duplicate of the read-decide-write critical section.
"""
from __future__ import annotations

import json
import os
import sys

import walk_state  # sibling: reuse _StateMutex + _atomic_write (canonical concurrency helpers)

SCHEMA_VERSION = 1
DEFAULT_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "trust.json")

# --- tier ladder (Loop-04 thresholds, verbatim; named so nothing is a magic number) ----------
WATCH = "watch"
QUEUE = "queue"
AUTO = "auto"
TIER_ORDER = (WATCH, QUEUE, AUTO)  # index = rank; demotion moves one step toward WATCH
QUEUE_MIN_RUNS = 10
QUEUE_MIN_RATE = 0.90
AUTO_MIN_RUNS = 20
AUTO_MIN_RATE = 0.95

RESULT_PASS = "pass"
RESULT_FAIL = "fail"


class TrustLedgerError(Exception):
    """The trust ledger file exists but is unparsable/malformed."""


def _load(path: str) -> dict:
    """Missing file -> a fresh empty ledger; a present-but-corrupt file raises (callers wrap
    record()/tier_map() best-effort, so a bad ledger never breaks the land, only loses history)."""
    if not os.path.isfile(path):
        return {"schema_version": SCHEMA_VERSION, "classes": {}}
    try:
        with open(path, "r", encoding="utf-8") as fh:
            data = json.load(fh)
    except (OSError, json.JSONDecodeError) as exc:
        raise TrustLedgerError(f"trust ledger unreadable/invalid JSON: {path} ({exc})") from exc
    if not isinstance(data, dict) or not isinstance(data.get("classes"), dict):
        raise TrustLedgerError(f"trust ledger malformed (no classes map): {path}")
    return data


def _base_tier(runs: int, passes: int) -> str:
    """The count-derived tier before any fail-demotion. Pure function of (runs, passes)."""
    if runs <= 0:
        return WATCH
    rate = passes / runs
    if runs >= AUTO_MIN_RUNS and rate >= AUTO_MIN_RATE:
        return AUTO
    if runs >= QUEUE_MIN_RUNS and rate >= QUEUE_MIN_RATE:
        return QUEUE
    return WATCH


def _tier_from(rec: dict) -> str:
    """Recompute the tier for one class record: the count-derived base, demoted one step when the
    last recorded result was a fail (immediate-demotion-on-fail, Loop-04)."""
    base = _base_tier(int(rec.get("runs", 0)), int(rec.get("passes", 0)))
    if rec.get("last_result") == RESULT_FAIL:
        return TIER_ORDER[max(0, TIER_ORDER.index(base) - 1)]
    return base


def record(task_class: str, passed: bool, path: str = DEFAULT_PATH) -> str:
    """Record one outcome for task_class and return the resulting tier. Serialized by the walk-state
    mutex + atomic write. On a fail that drops the tier below what it was before this outcome, emit a
    one-line stderr ALERT (the demotion signal the orchestrator reads)."""
    if not task_class or not isinstance(task_class, str):
        raise ValueError("task_class must be a non-empty string")
    with walk_state._StateMutex(path):
        data = _load(path)
        classes = data.setdefault("classes", {})
        rec = classes.setdefault(
            task_class, {"runs": 0, "passes": 0, "last_ts": None, "last_result": None})
        pre_tier = _tier_from(rec)
        rec["runs"] = int(rec.get("runs", 0)) + 1
        if passed:
            rec["passes"] = int(rec.get("passes", 0)) + 1
        rec["last_result"] = RESULT_PASS if passed else RESULT_FAIL
        rec["last_ts"] = walk_state._utc_now_iso()
        post_tier = _tier_from(rec)
        runs, passes = rec["runs"], rec["passes"]
        walk_state._atomic_write(path, data)
    if not passed and TIER_ORDER.index(post_tier) < TIER_ORDER.index(pre_tier):
        print(f"[trust-ledger] ALERT demotion {task_class!r}: {pre_tier} -> {post_tier} on fail "
              f"(runs={runs} passes={passes})", file=sys.stderr, flush=True)
    return post_tier


def tier(task_class: str, path: str = DEFAULT_PATH) -> str:
    """The current autonomy tier for task_class. An unknown/never-recorded class is WATCH."""
    rec = _load(path).get("classes", {}).get(task_class)
    return _tier_from(rec) if rec else WATCH


def tier_map(path: str = DEFAULT_PATH) -> dict:
    """{task_class: tier} for every recorded class (the compact read the supervisor surfaces into
    the Opus boot context). Composed from tier() so there is a single tier-derivation read path.
    Empty when the ledger does not yet exist."""
    classes = _load(path).get("classes", {})
    return {name: tier(name, path) for name in classes}


if __name__ == "__main__":
    # Convenience read-only dump (not a production entrypoint; the wired callers are opus_land.record
    # + opus_supervisor.tier_map).
    print(json.dumps(tier_map(), indent=2, sort_keys=True))
