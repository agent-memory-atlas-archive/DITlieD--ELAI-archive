"""standing_goals.py - post-land smoke-regression watchdog for the opus walk (P1.7).

WHY THIS EXISTS (2026-07-10, opus-orchestrator improvement roadmap): a plan lands green, then a LATER
plan silently regresses its smoke. The walk never noticed because a gate only runs at land time. A
STANDING GOAL is the drained plan's OWN smoke, re-run on a daily timer: enroll(plan_id, predicates)
right after a successful land records the exact cargo gate commands that just passed, and --verify
re-runs them. A predicate that flips non-zero is a VIOLATION: the goal file is stamped VIOLATED, a
goal-ledger row is appended, and the plan is RE-ENTERED into the walk (walk_state.seed_plan, guarded)
so a fresh Opus turn re-lands the fix. A goal that keeps flapping is QUARANTINED after
QUARANTINE_AFTER consecutive violations (stop bothering the pipeline); retirement stays a human call.

Storage: goals/<plan_id>.md (a small ```json frontmatter block) beside walk-state.json; the run ledger
is goal-ledger.json beside the walk-state file. Depends only on walk_state + stdlib (no coupling to
the conductor events/DB: violations are logged to stderr + the goal-ledger, and the re-seed re-enters
the pipeline whose downstream already emits events - a direct events import would need the control.db
and would write real state from a --verify run; the ledger row + re-seed are the durable record).
"""
from __future__ import annotations

import argparse
import glob
import json
import os
import subprocess
import sys
from pathlib import Path

import walk_state  # sibling: seed_plan (re-entry), load (guard reads), _StateMutex/_atomic_write

# --- statuses + tuning (named; nothing a magic number) ---------------------------------------
STATUS_ACTIVE = "ACTIVE"
STATUS_VIOLATED = "VIOLATED"
STATUS_QUARANTINED = "QUARANTINED"

DEFAULT_TIMEOUT_S = 900  # per-goal predicate wall ceiling. steal-doc says 60s; measured cargo smokes
#                          here run ~671s, so 900s is a deliberate deviation with headroom (D-9).
QUARANTINE_AFTER = 3     # consecutive violations before a flapping goal is set aside (never deleted)
TIMEOUT_RC = 124        # a predicate timeout counts as a violation (shell `timeout`'s own code)

_HERE = Path(__file__).resolve().parent
DEFAULT_GOALS_DIR = str(_HERE / "goals")
CARGO_BIN = Path.home() / ".cargo" / "bin"
# the daily verifier runs the real cargo smokes against the canonical repo checkout.
ELAI_ELAI_CC = "/workspace/ELAI/.elai_cc"

_GOAL_FENCE_OPEN = "```json"
_GOAL_FENCE_CLOSE = "```"


class StandingGoalError(Exception):
    """A goal file exists but its frontmatter block is missing/unparsable."""


# ---- goal-file (de)serialization (kept in helpers, never inline) -----------------------------

def _goal_path(goals_dir: str, plan_id: str) -> str:
    return os.path.join(goals_dir, f"{plan_id}.md")


def _serialize_goal(rec: dict) -> str:
    body = json.dumps(rec, indent=2, sort_keys=True)
    preds = "\n".join(f"- {p}" for p in rec.get("predicates", []))
    return (f"# Standing goal: {rec.get('plan_id')}\n\n"
            f"{_GOAL_FENCE_OPEN}\n{body}\n{_GOAL_FENCE_CLOSE}\n\n"
            f"Predicates (human copy; the json block above is authoritative):\n{preds}\n")


def _parse_goal(text: str) -> dict:
    start = text.find(_GOAL_FENCE_OPEN)
    if start == -1:
        raise StandingGoalError("goal file has no json frontmatter block")
    start += len(_GOAL_FENCE_OPEN)
    end = text.find(_GOAL_FENCE_CLOSE, start)
    if end == -1:
        raise StandingGoalError("goal file json frontmatter block is unterminated")
    try:
        rec = json.loads(text[start:end])
    except json.JSONDecodeError as exc:
        raise StandingGoalError(f"goal file json frontmatter invalid: {exc}") from exc
    if not isinstance(rec, dict) or not rec.get("plan_id"):
        raise StandingGoalError("goal file json frontmatter is not a plan record")
    return rec


def _read_goal(path: str) -> dict:
    with open(path, "r", encoding="utf-8") as fh:
        return _parse_goal(fh.read())


def _write_goal(path: str, rec: dict) -> None:
    # atomic write of TEXT: mirror walk_state's tmp+os.replace idiom for a str payload.
    _atomic_write_text(path, _serialize_goal(rec))


def _atomic_write_text(path: str, text: str) -> None:
    import tempfile
    directory = os.path.dirname(path) or "."
    os.makedirs(directory, exist_ok=True)
    fd, tmp = tempfile.mkstemp(prefix=".goal-", suffix=".tmp", dir=directory)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as f:
            f.write(text)
        os.replace(tmp, path)
    except BaseException:
        try:
            os.remove(tmp)
        except OSError:
            pass
        raise


# ---- enrollment (P1.7 production hook: called by opus_land after a successful land) ----------

def enroll(plan_id: str, predicates, meta: dict | None = None,
           goals_dir: str = DEFAULT_GOALS_DIR, timeout_s: int = DEFAULT_TIMEOUT_S) -> dict:
    """Enroll (or re-arm) plan_id's standing goal from the SAME gate commands opus_land just ran.
    Re-enrollment (a re-land) refreshes predicates and resets the goal to ACTIVE with a clean
    failure count. Raises on an empty predicate set (a goal with nothing to check is meaningless -
    opus_land skips the call in that case rather than enrolling a vacuous goal)."""
    if not plan_id or not isinstance(plan_id, str):
        raise ValueError("plan_id must be a non-empty string")
    predicates = [p for p in (predicates or []) if isinstance(p, str) and p.strip()]
    if not predicates:
        raise ValueError("enroll requires >=1 predicate (the plan's real gate commands)")
    meta = meta or {}
    path = _goal_path(goals_dir, plan_id)
    prior_enrolled = None
    if os.path.isfile(path):
        try:
            prior_enrolled = _read_goal(path).get("enrolled_ts")
        except StandingGoalError:
            prior_enrolled = None
    now = walk_state._utc_now_iso()
    rec = {
        "plan_id": plan_id,
        "predicates": predicates,
        "timeout_s": int(timeout_s),
        "status": STATUS_ACTIVE,
        "last_pass": None,
        "consecutive_failures": 0,
        "enrolled_ts": prior_enrolled or now,
        "reenrolled_ts": now if prior_enrolled else None,
        "commit_sha": meta.get("commit_sha", ""),
    }
    _write_goal(path, rec)
    return rec


# ---- verification (P1.7 production hook: the --verify systemd-timer entrypoint) --------------

def _default_runner(predicate: str, timeout_s: int) -> int:
    """Run one predicate command and return its exit code (TIMEOUT_RC on timeout). cwd is the
    canonical .elai_cc checkout with cargo on PATH; injectable so tests never spawn a real process."""
    env = dict(os.environ)
    env["PATH"] = f"{CARGO_BIN}:{env.get('PATH', '')}"
    env.setdefault("CARGO_TARGET_DIR", str(Path.home() / ".cache" / "cargo-target-elai"))
    cwd = ELAI_ELAI_CC if os.path.isdir(ELAI_ELAI_CC) else None
    try:
        proc = subprocess.run(predicate, shell=True, cwd=cwd, env=env,
                              capture_output=True, text=True, timeout=timeout_s)
        return proc.returncode
    except subprocess.TimeoutExpired:
        return TIMEOUT_RC


def _run_predicates(rec: dict, runner) -> tuple[bool, str | None]:
    """Run every predicate; short-circuit on the first non-zero. Returns (all_passed, failed_cmd)."""
    timeout_s = int(rec.get("timeout_s", DEFAULT_TIMEOUT_S))
    for pred in rec.get("predicates", []):
        if runner(pred, timeout_s) != 0:
            return False, pred
    return True, None


def _plan_status(plan_id: str, walk_path: str) -> str | None:
    """Status string from plan-status/<plan_id>.json beside the walk-state file, or None if no row."""
    row_path = os.path.join(os.path.dirname(walk_path) or ".", "plan-status", f"{plan_id}.json")
    if not os.path.isfile(row_path):
        return None
    try:
        with open(row_path, "r", encoding="utf-8") as fh:
            return json.load(fh).get("status")
    except (OSError, json.JSONDecodeError):
        return None


def _should_reseed(plan_id: str, walk_path: str) -> bool:
    """Guard the re-entry: skip if the plan is already queued, escalated, or has a non-DONE (in-flight)
    plan-status row. A missing row or a DONE row (the regression case: it landed, then broke) re-seeds."""
    try:
        state = walk_state.load(walk_path)
    except walk_state.WalkStateError:
        return False
    if plan_id in (state.get("queue") or []):
        return False
    if plan_id in {e.get("plan_id") for e in (state.get("escalation_queue") or [])}:
        return False
    status = _plan_status(plan_id, walk_path)
    return status is None or status == "DONE"


def _append_ledger(ledger_path: str, row: dict) -> None:
    with walk_state._StateMutex(ledger_path):
        if os.path.isfile(ledger_path):
            try:
                with open(ledger_path, "r", encoding="utf-8") as fh:
                    data = json.load(fh)
            except (OSError, json.JSONDecodeError):
                data = {}
        else:
            data = {}
        if not isinstance(data, dict) or not isinstance(data.get("rows"), list):
            data = {"schema_version": 1, "rows": []}
        row = dict(row, ts=walk_state._utc_now_iso())
        data["rows"].append(row)
        walk_state._atomic_write(ledger_path, data)


def verify_all(goals_dir: str = DEFAULT_GOALS_DIR, runner=_default_runner,
               walk_path: str = walk_state.STATE_PATH, ledger_path: str | None = None) -> dict:
    """Re-run every ACTIVE/VIOLATED goal's predicates. Pass -> stamp last_pass + reset failures.
    Fail/timeout -> increment failures, flip VIOLATED (or QUARANTINED at QUARANTINE_AFTER), append a
    ledger row, log, and guarded-re-seed the plan into the walk. QUARANTINED goals are skipped, not
    run. Returns a summary dict."""
    if ledger_path is None:
        ledger_path = os.path.join(os.path.dirname(walk_path) or ".", "goal-ledger.json")
    summary: dict = {"checked": [], "passed": [], "violated": [], "quarantined": [],
                     "skipped": [], "reseeded": []}
    for path in sorted(glob.glob(os.path.join(goals_dir, "*.md"))):
        try:
            rec = _read_goal(path)
        except StandingGoalError as exc:
            print(f"[standing-goals] skipping unparsable goal {path}: {exc}", file=sys.stderr)
            continue
        plan_id = rec["plan_id"]
        if rec.get("status") == STATUS_QUARANTINED:
            summary["skipped"].append(plan_id)
            continue
        summary["checked"].append(plan_id)
        ok, failed = _run_predicates(rec, runner)
        now = walk_state._utc_now_iso()
        if ok:
            rec["status"] = STATUS_ACTIVE
            rec["consecutive_failures"] = 0
            rec["last_pass"] = now
            _write_goal(path, rec)
            _append_ledger(ledger_path, {"plan_id": plan_id, "result": "pass",
                                         "status": STATUS_ACTIVE, "consecutive_failures": 0})
            summary["passed"].append(plan_id)
            continue
        fails = int(rec.get("consecutive_failures", 0)) + 1
        rec["consecutive_failures"] = fails
        if fails >= QUARANTINE_AFTER:
            rec["status"] = STATUS_QUARANTINED
            summary["quarantined"].append(plan_id)
        else:
            rec["status"] = STATUS_VIOLATED
            summary["violated"].append(plan_id)
        _write_goal(path, rec)
        _append_ledger(ledger_path, {"plan_id": plan_id, "result": "violated",
                                     "status": rec["status"], "failed_predicate": failed,
                                     "consecutive_failures": fails})
        print(f"[standing-goals] VIOLATION {plan_id}: predicate failed ({failed!r}); "
              f"status={rec['status']} consecutive={fails}", file=sys.stderr, flush=True)
        # QUARANTINED goals are being set aside -> do NOT re-enter the pipeline. Only a VIOLATED
        # (below the quarantine threshold) goal is re-seeded, and only when the guard permits.
        if rec["status"] == STATUS_VIOLATED and _should_reseed(plan_id, walk_path):
            try:
                walk_state.seed_plan(
                    plan_id,
                    pickup=f"standing-goal regression: re-verify + re-land {plan_id} "
                           f"(predicate failed: {failed})",
                    path=walk_path)
                summary["reseeded"].append(plan_id)
            except Exception as exc:  # noqa: BLE001 - re-seed is best-effort, never crash the sweep
                print(f"[standing-goals] re-seed failed for {plan_id}: {exc}", file=sys.stderr)
    return summary


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="opus-walk standing-goal smoke-regression verifier")
    ap.add_argument("--verify", action="store_true", help="re-run every enrolled goal's predicates")
    ap.add_argument("--goals-dir", default=DEFAULT_GOALS_DIR)
    args = ap.parse_args(argv)
    if not args.verify:
        ap.error("nothing to do: pass --verify")
    summary = verify_all(goals_dir=args.goals_dir)
    for plan_id in summary["checked"]:
        state = ("pass" if plan_id in summary["passed"]
                 else "quarantined" if plan_id in summary["quarantined"] else "violated")
        print(f"{plan_id}: {state}")
    for plan_id in summary["skipped"]:
        print(f"{plan_id}: skipped (quarantined)")
    failed_this_run = summary["violated"] + summary["quarantined"]
    return 1 if failed_this_run else 0


if __name__ == "__main__":
    sys.exit(main())
