#!/usr/bin/env python3
"""disk_hygiene.py - keep the opus-orchestrator walk from filling the disk.

WHY THIS EXISTS (incident 2026-07-12, the SECOND hand-reap, on the old WSL box): the opus walk builds
cargo in a shared target and in a throwaway git worktree per dispatch under conductor/elai-worktrees
(free swe lane) and the codex worktree base (codex lane). NOTHING reaped them. They grew to ~109 GB
(+ a ~37 GB shared target), the disk hit its ceiling, and builds started throwing EIO -- the whole walk
wedged until a human freed space. This module makes the reclaim automatic: a pre-spawn free-space GUARD
that refuses to mint a paid Opus turn when free space is dangerously low, plus a between-plan RECLAIM of
the regenerable build scratch. Since the 2026-07-17 Kubuntu migration the binding number is the root
ext4 fs free space (the old vhdx-grows-into-F: indirection is gone).

WHY REAPING, NOT `cargo clean`: the cargo target is a pure regenerable cache (safe to delete whole).
The worktrees are throwaway per-dispatch trees (a re-dispatch always makes a fresh one), so reaping a
worktree that is not for an in-flight plan is safe. Reclaim runs BETWEEN Opus turns (no turn is active
at the call sites -- one turn runs at a time), which makes reaping structurally safe; the in-flight
plan-id skip is defensive belt-and-braces on top of that.

Called from opus_supervisor.run_supervisor at two seams: (a) the pre-spawn guard (below), and (b) an
unconditional reclaim after a drained plan is reconciled off the queue. Escape hatch:
OPUS_WALK_DISK_GUARD=off disables both (mirrors OPUS_WALK_AUTOSEED / OPUS_WALK_QUORUM). Stdlib only.
"""
from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path

# --- fixed wiring (named constants; no magic numbers downstream) --------------
# Native Kubuntu (2026-07-17 migration): the binding constraint is the root ext4 fs itself; the old
# WSL vhdx-on-F: architecture is gone, so HOST_MOUNT and DISTRO_ROOT are the same filesystem now.
HOST_MOUNT = "/"
# Kept as a separate constant so the guard's two-number report stays stable for log consumers.
DISTRO_ROOT = "/"
# Below this many GB free, a paid Opus turn is REFUSED and the chain halts (emergency floor).
HARD_FLOOR_GB = 12.0
# Below this many GB free (but >= the hard floor), run the reclaim first, then only spawn if the
# reclaim brought free space back above this threshold -- otherwise halt (a paid build needs real headroom).
SOFT_THRESHOLD_GB = 30.0
# The env escape hatch: OPUS_WALK_DISK_GUARD=off disables the guard AND the reclaim (default on).
GUARD_ENV = "OPUS_WALK_DISK_GUARD"
_HOME = Path.home()  # /home/researcher for the supervisor process
# The shared cargo target the free + codex + opus_land lanes all setdefault CARGO_TARGET_DIR to
# (standing_goals.py / opus_land.py / opus_dispatch.py). Pure regenerable cache: reaped whole.
CARGO_TARGET_DIR = _HOME / ".cache" / "cargo-target-elai"
# The two per-dispatch throwaway worktree parents: the free swe lane (elai_walk_driver / diskguard
# base) and the codex lane (CODEX_WORKTREE_BASE in opus_dispatch).
WORKTREE_DIRS = (
    _HOME / "teikoku" / "conductor" / "elai-worktrees",
    _HOME / "teikoku" / "conductor" / "elai-codex-worktrees",
)
# The canonical git repo the worktrees belong to; `git worktree prune` clears stale registrations after
# the dirs are reaped.
ELAI_REPO = Path("/workspace/ELAI")
FSTRIM_BIN = "/usr/sbin/fstrim"
MEMINFO_PATH = Path("/proc/meminfo")
# Integrator-owned in-flight merge workspaces (mirrors diskguard.py) -- never reaped.
INTEGRATOR_PREFIX = "integrate_"
# Wall ceiling for the git-prune / fstrim subprocesses (fstrim on a large fs can take a while).
RUN_TIMEOUT_S = 180


def _log(msg: str) -> None:
    """Default fail-soft log sink: prefixed line to stderr, flushed."""
    print(f"[disk-hygiene] {msg}", file=sys.stderr, flush=True)


def free_gb(path: str) -> float:
    """Free space (GB) on the filesystem holding `path`; -1.0 on any error (an unreadable stat is
    reported as -1.0, never raised, so the caller fails soft rather than crashing the walk)."""
    try:
        return shutil.disk_usage(path).free / (2 ** 30)
    except OSError:
        return -1.0


def free_space() -> dict:
    """Free GB report. Since the Kubuntu migration HOST_MOUNT and DISTRO_ROOT are the same root fs;
    both keys are kept so log consumers keep their shape."""
    return {"root_gb": free_gb(DISTRO_ROOT), "f_gb": free_gb(HOST_MOUNT)}


def guard_enabled() -> bool:
    """The disk guard + reclaim are ON unless env OPUS_WALK_DISK_GUARD=off (the skippable escape)."""
    return (os.environ.get(GUARD_ENV) or "on").strip().lower() != "off"


def in_flight_plan_ids(state: dict, *, pending_path=None) -> set:
    """The plan-ids the walk considers live: every queued plan + the cursor plan + pending_lands.

    A worktree dir whose name contains any of these is defensively skipped by the reap
    (belt-and-braces on top of the between-turns structural safety).

    pending_path: optional override for the pending_lands ledger (tests inject an empty
    ledger so live park rows cannot pollute unit assertions). Default is the live ledger.
    """
    ids = set(p for p in (state.get("queue") or []) if isinstance(p, str) and p)
    cursor_plan = (state.get("cursor") or {}).get("plan_id")
    if isinstance(cursor_plan, str) and cursor_plan:
        ids.add(cursor_plan)
    try:
        from pending_lands import protected_plan_ids
        kwargs = {} if pending_path is None else {"path": pending_path}
        ids |= protected_plan_ids(**kwargs)
    except Exception:
        try:
            from opus_walk.pending_lands import protected_plan_ids
            kwargs = {} if pending_path is None else {"path": pending_path}
            ids |= protected_plan_ids(**kwargs)
        except Exception:
            pass
    return ids


def _is_protected(name: str, protected_ids: set) -> bool:
    """True when a worktree dir `name` contains any in-flight plan-id as a substring (I-23 in
    'I-23_ab12cd', H-28 in 'H-28-WIRE-1_ff00'). Conservative: a substring match skips a maybe-safe
    worktree, which is the safe direction."""
    return any(pid in name for pid in protected_ids if pid)


KEEP_LAND_MARKER = ".KEEP_FOR_LAND"


def _has_keep_for_land(dirpath: Path) -> bool:
    try:
        return (dirpath / KEEP_LAND_MARKER).is_file()
    except OSError:
        return False


def _safe_rmtree(target: Path, must_parent: Path, log, rmtree_fn=shutil.rmtree) -> bool:
    """Path-guarded rmtree. Removes `target` ONLY when it is a directly-nested child of `must_parent`
    and its basename is non-empty (never rm from a relative/empty/root-ish var). Returns True iff a
    directory was removed. Fail-soft: any resolve/remove error logs + returns False, never raises."""
    try:
        rt = target.resolve()
        mp = must_parent.resolve()
    except OSError as exc:
        log(f"resolve failed for {target}: {exc}")
        return False
    if not rt.name or rt.name in (".", ".."):
        log(f"REFUSE rmtree of empty/root-ish path {rt}")
        return False
    if rt.parent != mp:
        log(f"REFUSE rmtree {rt}: not directly under expected parent {mp}")
        return False
    if not target.is_dir():
        return False  # already gone / not a directory -> nothing to reap
    try:
        rmtree_fn(str(target))
    except OSError as exc:
        log(f"could not reap {rt}: {exc}")
        return False
    log(f"reaped {rt}")
    return True


def _reap_worktree_base(base: Path, protected_ids: set, log, rmtree_fn=shutil.rmtree) -> list:
    """Reap every non-protected, non-integrator worktree directly under `base`. Returns the list of
    reaped paths. A missing base or an unreadable listing is a no-op (fail-soft)."""
    reaped: list = []
    try:
        if not base.is_dir():
            return reaped
        children = sorted(base.iterdir())
    except OSError as exc:
        log(f"cannot list worktree base {base}: {exc}")
        return reaped
    for d in children:
        try:
            if not d.is_dir():
                continue
        except OSError:
            continue
        if d.name.startswith(INTEGRATOR_PREFIX):
            continue  # integrator-owned in-flight merge workspace
        if _is_protected(d.name, protected_ids):
            log(f"skip in-flight worktree {d.name}")
            continue
        if _has_keep_for_land(d):
            log(f"skip KEEP_FOR_LAND worktree {d.name}")
            continue
        if _safe_rmtree(d, base, log, rmtree_fn):
            reaped.append(str(d))
    return reaped


def _run_soft(cmd: list, run_fn, log) -> None:
    """Run a reclaim subprocess fail-soft: any spawn error or non-zero exit logs + continues, never
    raises (a git-prune / fstrim failure must not crash the supervisor)."""
    try:
        out = run_fn(cmd, capture_output=True, text=True, timeout=RUN_TIMEOUT_S, check=False)
    except (OSError, subprocess.SubprocessError) as exc:
        log(f"{cmd[0]} failed: {type(exc).__name__}: {exc}")
        return
    if getattr(out, "returncode", 0) != 0:
        log(f"{' '.join(cmd)} rc={out.returncode}: {(getattr(out, 'stderr', '') or '').strip()[:200]}")


def reclaim(protected_ids, *, free_fn=free_gb, rmtree_fn=shutil.rmtree,
            run_fn=subprocess.run, log=_log) -> dict:
    """Reap the regenerable build scratch. FAIL-SOFT throughout: every step is guarded and a failure
    logs + continues, so reclaim never raises into the supervisor. Steps:
      1. delete the whole shared cargo target (pure cache; always safe),
      2. reap non-protected worktrees under each WORKTREE_DIRS base (protected = an in-flight plan-id
         appears in the dir name; None means the in-flight set is unknown -> skip worktree reaping,
         only cargo-target + fstrim run),
      3. `git worktree prune` on the ELAI repo (clear stale worktree registrations),
      4. `sudo -n fstrim -av` (issue discards for the freed blocks; harmless no-op if unneeded).
    Returns {freed_gb, f_free_before, f_free_after, reaped:[paths]} measured on HOST_MOUNT across
    the whole pass (after fstrim)."""
    f_before = free_fn(HOST_MOUNT)
    reaped: list = []

    # 1. shared cargo target -- whole dir, pure regenerable cache.
    if _safe_rmtree(CARGO_TARGET_DIR, CARGO_TARGET_DIR.parent, log, rmtree_fn):
        reaped.append(str(CARGO_TARGET_DIR))

    # 2. per-dispatch worktrees (skip when the in-flight set is unknown).
    if protected_ids is None:
        log("in-flight plan set unknown; skipping worktree reap (cargo-target + fstrim only)")
    else:
        for base in WORKTREE_DIRS:
            reaped.extend(_reap_worktree_base(base, protected_ids, log, rmtree_fn))

    # 3. clear stale git worktree registrations for the dirs just reaped.
    _run_soft(["git", "-C", str(ELAI_REPO), "worktree", "prune"], run_fn, log)

    # 4. return freed ext4 blocks to the host (the vhdx does not auto-shrink without this).
    _run_soft(["sudo", "-n", FSTRIM_BIN, "-av"], run_fn, log)

    f_after = free_fn(HOST_MOUNT)
    freed = max(0.0, f_after - f_before) if (f_before >= 0 and f_after >= 0) else 0.0
    return {
        "freed_gb": round(freed, 2),
        "f_free_before": round(f_before, 2) if f_before >= 0 else None,
        "f_free_after": round(f_after, 2) if f_after >= 0 else None,
        "reaped": reaped,
    }


def _protected_from_loader(state_loader, log):
    """Resolve the in-flight plan-id set from an injected walk-state loader. Fail-soft: if the loader
    is None or raises, return None (reclaim then skips worktree reaping and does the always-safe
    cargo-target + fstrim), never crash the guard."""
    if state_loader is None:
        return None
    try:
        state = state_loader()
    except Exception as exc:  # noqa: BLE001 - a walk-state read failure must not crash the guard
        log(f"could not load walk-state for in-flight guard ({exc}); skipping worktree reap")
        return None
    return in_flight_plan_ids(state)


def reap_after_turn(state_loader) -> dict:
    """Reap non-protected dispatch worktrees after every completed controller turn."""
    try:
        protected_ids = _protected_from_loader(state_loader, _log)
    except Exception as exc:
        _log(f"per-turn state protection lookup failed: {type(exc).__name__}: {exc}")
        return {"reaped": [], "protected_ids": None, "skipped": True}
    if protected_ids is None:
        return {"reaped": [], "protected_ids": None, "skipped": True}
    reaped: list[str] = []
    for base in WORKTREE_DIRS:
        try:
            reaped.extend(_reap_worktree_base(base, protected_ids, _log))
        except Exception as exc:
            _log(f"per-turn worktree reap failed for {base}: {type(exc).__name__}: {exc}")
    try:
        _run_soft(["git", "-C", str(ELAI_REPO), "worktree", "prune"], subprocess.run, _log)
    except Exception as exc:
        _log(f"per-turn git worktree prune failed: {type(exc).__name__}: {exc}")
    return {"reaped": reaped, "protected_ids": sorted(protected_ids), "skipped": False}


def mem_guard(*, meminfo_path: str | Path | None = None) -> dict:
    meminfo_path = Path(meminfo_path or MEMINFO_PATH)
    try:
        floor_gb = float(os.environ.get("TEIKOKU_WALK_MEM_FLOOR_GB") or "16")
    except ValueError:
        return {
            "action": "halt",
            "available_gb": None,
            "reason": "memory floor is invalid; halting fail-closed",
        }
    if floor_gb < 0:
        return {
            "action": "halt",
            "available_gb": None,
            "reason": "memory floor is negative; halting fail-closed",
        }
    try:
        text = Path(meminfo_path).read_text(encoding="ascii")
        available_kib = next(
            int(line.split()[1])
            for line in text.splitlines()
            if line.startswith("MemAvailable:") and len(line.split()) >= 2
        )
    except (OSError, StopIteration, ValueError, IndexError):
        return {
            "action": "halt",
            "available_gb": None,
            "reason": "MemAvailable unreadable; halting fail-closed",
        }
    available_gb = available_kib / (1024 * 1024)
    if available_gb < floor_gb:
        return {
            "action": "halt",
            "available_gb": available_gb,
            "reason": f"available memory {available_gb:.1f}GB below floor {floor_gb:.1f}GB",
        }
    return {
        "action": "spawn",
        "available_gb": available_gb,
        "reason": f"available memory {available_gb:.1f}GB meets floor {floor_gb:.1f}GB",
    }


def disk_guard(state_loader=None, *, free_fn=free_gb, reclaim_fn=reclaim, log=_log) -> dict:
    """Pre-spawn free-space decision on F:. Returns a decision dict the supervisor acts on:
        {action: 'spawn'|'halt', f_free_gb, reclaim: <reclaim result|None>, below_hard: bool, reason}
    Policy (binding number = free space on HOST_MOUNT):
      * guard disabled                     -> spawn (no read, no reclaim).
      * free space unreadable (-1)            -> spawn, fail-soft (never block the walk on a bad stat).
      * free >= SOFT                    -> spawn (plenty of headroom; no reclaim).
      * free <  SOFT                       -> RECLAIM, re-read, then:
            re-read >= SOFT                -> spawn (reclaim recovered enough headroom).
            re-read <  SOFT                -> halt (below_hard = re-read < HARD floor); a paid build
                                             needs real headroom, so below-soft-after-reclaim halts.
    state_loader/free_fn/reclaim_fn are injectable so tests never touch the fs, git, or subprocess."""
    decision = {"action": "spawn", "f_free_gb": None, "reclaim": None,
                "below_hard": False, "reason": ""}
    if not guard_enabled():
        decision["reason"] = f"guard disabled ({GUARD_ENV}=off)"
        return decision

    f0 = free_fn(HOST_MOUNT)
    decision["f_free_gb"] = round(f0, 2) if f0 >= 0 else None
    if f0 < 0:
        decision["reason"] = f"disk free unreadable at {HOST_MOUNT}; allowing spawn (fail-soft)"
        return decision
    if f0 >= SOFT_THRESHOLD_GB:
        decision["reason"] = f"disk free {f0:.1f}GB >= soft {SOFT_THRESHOLD_GB:.0f}GB"
        return decision

    # Below the soft threshold: reclaim first (this covers the critical band too -- the owner directive
    # is to clean even during low-space, not only between plans), then re-check F:.
    protected = _protected_from_loader(state_loader, log)
    rc = reclaim_fn(protected, log=log)
    decision["reclaim"] = rc
    f1 = free_fn(HOST_MOUNT)
    decision["f_free_gb"] = round(f1, 2) if f1 >= 0 else None
    if f1 >= SOFT_THRESHOLD_GB:
        decision["reason"] = (f"reclaim recovered disk {f0:.1f}->{f1:.1f}GB "
                              f"(>= soft {SOFT_THRESHOLD_GB:.0f}GB)")
        return decision

    decision["action"] = "halt"
    decision["below_hard"] = f1 < HARD_FLOOR_GB
    band = (f"hard floor {HARD_FLOOR_GB:.0f}GB" if decision["below_hard"]
            else f"soft {SOFT_THRESHOLD_GB:.0f}GB")
    decision["reason"] = (f"disk free {f1:.1f}GB below {band} after reclaim "
                          f"(freed {rc.get('freed_gb')}GB); halting -- a paid build needs headroom")
    return decision


def main() -> int:
    ap = argparse.ArgumentParser(description="opus-walk disk hygiene (free-space guard + reaper)")
    ap.add_argument("--status", action="store_true", help="report free space + reap targets, do nothing")
    ap.add_argument("--reclaim", action="store_true", help="run the between-plan reclaim now")
    args = ap.parse_args()
    if args.reclaim:
        rc = reclaim(set())  # CLI reap: no walk loaded, protect nothing extra (operator-invoked)
        print(f"reclaim: freed~{rc['freed_gb']}GB "
              f"({rc['f_free_before']}->{rc['f_free_after']}GB); reaped {len(rc['reaped'])} dir(s)")
        for p in rc["reaped"]:
            print(f"  - {p}")
        return 0
    fs = free_space()
    print(f"disk free = {fs['f_gb']:.1f}GB (binding)")
    print(f"hard floor = {HARD_FLOOR_GB:.0f}GB   soft threshold = {SOFT_THRESHOLD_GB:.0f}GB   "
          f"guard={'on' if guard_enabled() else 'off'}")
    print(f"cargo target: {CARGO_TARGET_DIR} (exists={CARGO_TARGET_DIR.is_dir()})")
    for base in WORKTREE_DIRS:
        n = sum(1 for d in base.iterdir() if d.is_dir()) if base.is_dir() else 0
        print(f"worktree base: {base} (exists={base.is_dir()}, dirs={n})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
