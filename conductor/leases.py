from __future__ import annotations

import json
import re
import subprocess
import uuid
from dataclasses import dataclass
from datetime import timedelta
from pathlib import Path

from conductor_config import normalize_path
from conductor_db import ControlDB
from models import TaskSpec
from time_utils import iso_utc, utc_now


@dataclass(frozen=True)
class LeaseRecord:
    lease_id: str
    task_id: str
    slot_id: str
    worktree_path: str
    branch_name: str
    lease_expires_at: str
    account_id: str | None = None
    model: str | None = None


@dataclass(frozen=True)
class LeaseMutationEvidence:
    mutated: bool | None
    reason: str
    touched_files: tuple[str, ...] = ()
    head_commit: str | None = None
    base_commit: str | None = None


def normalize_scope(path: str) -> str:
    return path.replace("\\", "/").strip("/").lower()


def scopes_conflict(left: list[str], right: list[str], *, empty_conflicts: bool = True) -> bool:
    """Do two write-sets conflict? An EMPTY write-set means "undeclared". With empty_conflicts
    (the safe default) an undeclared scope conflicts with everything, serializing dispatch. With
    empty_conflicts=False (the opt-in 'parallel where it can' model) an undeclared scope conflicts
    with nothing, so only DECLARED overlaps serialize -- relying on worktree isolation + serialized
    integration to catch a real collision as a rejected ff-merge, never corruption."""
    if not left or not right:
        return empty_conflicts
    left_norm = [normalize_scope(item) for item in left]
    right_norm = [normalize_scope(item) for item in right]
    for a in left_norm:
        for b in right_norm:
            if a == b or a.startswith(b + "/") or b.startswith(a + "/"):
                return True
    return False


def scopes_overlap(left: list[str], right: list[str]) -> bool:
    """Back-compat wrapper: the conservative default (an empty scope conflicts with everything)."""
    return scopes_conflict(left, right, empty_conflicts=True)


class LeaseManager:
    def __init__(self, db_path: str | Path, worktree_root: str | Path | None = None):
        self.db = ControlDB(db_path)
        self.worktree_root = Path(worktree_root) if worktree_root is not None else Path("worktrees")

    def acquire_ready(
        self,
        task_id: str | None = None,
        wave: str | None = None,
        slot_id: str | None = None,
        worktree_path: str | None = None,
        branch_name: str | None = None,
        now=None,
        excluded_pools: set[str] | None = None,
        exclude_paid: bool = False,
        empty_scope_conflicts: bool = True,
        active_slot_ids: set[str] | None = None,
    ) -> LeaseRecord | None:
        now_dt = now or utc_now()
        now_s = iso_utc(now_dt)

        with self.db.transaction() as conn:
            candidates = conn.execute(
                """
                SELECT t.*
                FROM tasks t
                WHERE t.status = 'READY'
                  AND (? IS NULL OR t.task_id = ?)
                  AND (? IS NULL OR t.wave = ?)
                  AND t.attempts < t.max_attempts
                  AND t.token_budget >= 0
                  AND NOT EXISTS (
                    SELECT 1
                    FROM task_deps d
                    JOIN tasks dep ON dep.task_id = d.depends_on_task_id
                    WHERE d.task_id = t.task_id
                      AND dep.status != 'DONE'
                  )
                ORDER BY t.priority DESC, t.created_at, t.task_id
                """,
                (task_id, task_id, wave, wave),
            ).fetchall()
            if not candidates:
                return None

            slots = conn.execute(
                """
                SELECT s.*
                FROM worker_slots s
                WHERE s.state = 'IDLE'
                  AND (? IS NULL OR s.slot_id = ?)
                  AND NOT EXISTS (
                    SELECT 1
                    FROM model_cooldowns c
                    WHERE c.model = s.model
                      AND c.cooldown_until > ?
                  )
                ORDER BY s.slot_id
                """,
                (slot_id, slot_id, now_s),
            ).fetchall()
            # SLOT-FIX (paid-leg-entry.md root cause): worker_slots is a superset across
            # lane configs -- seeding only upserts, never removes a slot no longer in the
            # active config. Restrict candidates to the slot pool the ACTIVE config actually
            # defines, so a stale slot from a prior config can never win a lease it cannot
            # dispatch to (worker_adapter.select_slot resolves against that same pool).
            if active_slot_ids is not None:
                slots = [row for row in slots if row["slot_id"] in active_slot_ids]
            if not slots:
                return None

            active_rows = conn.execute(
                """
                SELECT
                  t.allowed_write_set_json,
                  COALESCE(s.account_id, s.slot_id) AS account_key
                FROM leases l
                JOIN tasks t ON t.task_id = l.task_id
                JOIN worker_slots s ON s.slot_id = l.slot_id
                """
            ).fetchall()
            active_scopes = [json.loads(row["allowed_write_set_json"]) for row in active_rows]
            active_accounts = {row["account_key"] for row in active_rows}

            selected = None
            selected_slot = None
            for row in candidates:
                # TEIKOKU W-U3 usage throttle: when the PAID account is at/over its weekly
                # ceiling, the scheduler refuses NEW PAID leases while FREE keeps flowing. This
                # skips PAID-tier candidates only; a supervisor-plane liveness gate, never an
                # FSM/replay input (plan R-3).
                if exclude_paid and str(row["model_tier"] or "").upper() == "PAID":
                    continue
                candidate_scopes = json.loads(row["allowed_write_set_json"])
                if any(
                    scopes_conflict(candidate_scopes, scope, empty_conflicts=empty_scope_conflicts)
                    for scope in active_scopes
                ):
                    continue
                for slot in slots:
                    if slot["model_tier"] != row["model_tier"]:
                        continue
                    # TEIKOKU role->slot affinity: a slot with a roles set only serves
                    # tasks whose role is in it (NULL roles = serves all).
                    slot_roles = slot["roles"] if "roles" in slot.keys() else None
                    if slot_roles:
                        allowed = {x.strip().upper() for x in slot_roles.split(",") if x.strip()}
                        if (row["role"] or "").strip().upper() not in allowed:
                            continue
                    account_key = slot["account_id"] or slot["slot_id"]
                    if account_key in active_accounts:
                        continue
                    # TEIKOKU P3-A (master-plan 3.4): skip slots whose rate-limit pool is at
                    # its workers.toml max_concurrency. pool_key mirrors pool_concurrency's
                    # grouping (the slot's pool, or its slot_id when ungrouped).
                    if excluded_pools:
                        pool_key = slot["pool"] if slot["pool"] else slot["slot_id"]
                        if pool_key in excluded_pools:
                            continue
                    # TEIKOKU-IR B1-fix: a task carrying a pinned model or a required window is
                    # leased ONLY to a slot that can actually SERVE it (the slot's lane runs the
                    # pinned model, and its effective served window holds the required context).
                    # An unpinned task (no pin, no required window) is UNCHANGED (tier-only match),
                    # so the walk/self-test infra is untouched. If no slot serves a pinned/large
                    # task, it is left READY (not forced onto a too-small slot) and the reason is
                    # recorded as a LEASE_PIN_UNSERVED event below.
                    if not _slot_can_serve(row, slot):
                        continue
                    selected = row
                    selected_slot = slot
                    break
                if selected is not None:
                    break
            pin_unserved = [] if selected is not None else _collect_unserved_pins(candidates, slots)

            if selected is None:
                # No slot could serve the frontier this pass. Fall out of the txn and, if any
                # candidate was blocked specifically by the pin/window contract, record it.
                pass
            else:
                task = TaskSpec.from_db_row(selected)
                slot = selected_slot
                lease_id = str(uuid.uuid4())
                expires = iso_utc(now_dt + timedelta(seconds=task.lease_ttl_seconds))
                safe_task_id = _safe_task_id(task.task_id)
                wt = worktree_path or str(self.worktree_root / f"{safe_task_id}_{lease_id[:8]}")
                branch = branch_name or f"conductor/{safe_task_id}-{lease_id[:8]}"
                conn.execute(
                    """
                    INSERT INTO leases (
                      lease_id, task_id, slot_id, worktree_path, branch_name,
                      lease_started_at, lease_expires_at, heartbeat_at
                    )
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (lease_id, task.task_id, slot["slot_id"], wt, branch, now_s, expires, now_s),
                )
                conn.execute(
                    "UPDATE tasks SET status = 'LEASED', updated_at = ? WHERE task_id = ?",
                    (now_s, task.task_id),
                )
                conn.execute(
                    "UPDATE worker_slots SET state = 'LEASED' WHERE slot_id = ?",
                    (slot["slot_id"],),
                )

        # TEIKOKU-IR B1-fix: emit outside the transaction (append_event opens its own). A pinned/
        # large task that no idle slot could serve is left READY and its reason recorded, so it is
        # VISIBLE (not silently dropped). Emitted only when the whole acquire produced no lease.
        if selected is None:
            for ev in pin_unserved:
                self.db.append_event("LEASE_PIN_UNSERVED", ev.get("task_id"), ev, created_at=now_s)
            return None

        self.db.append_event(
            "LEASE_ACQUIRED",
            task.task_id,
            {
                "lease_id": lease_id,
                "slot_id": slot["slot_id"],
                "account_id": slot["account_id"],
                "expires_at": expires,
            },
            created_at=now_s,
        )
        self.db.append_event(
            "TASK_STATUS_CHANGED",
            task.task_id,
            {"old_status": selected["status"], "new_status": "LEASED", "reason": "lease_acquired"},
            created_at=now_s,
        )
        return LeaseRecord(
            lease_id,
            task.task_id,
            slot["slot_id"],
            wt,
            branch,
            expires,
            account_id=slot["account_id"],
            model=slot["model"],
        )

    def update_execution_refs(
        self,
        lease_id: str,
        *,
        worktree_path: str | None = None,
        branch_name: str | None = None,
        session_id: str | None = None,
        worker_pid: int | None = None,
    ) -> None:
        updates: list[str] = []
        values: list[object] = []
        if worktree_path:
            updates.append("worktree_path = ?")
            values.append(worktree_path)
        if branch_name:
            updates.append("branch_name = ?")
            values.append(branch_name)
        if session_id:
            updates.append("session_id = ?")
            values.append(session_id)
        if worker_pid is not None:
            updates.append("worker_pid = ?")
            values.append(worker_pid)
        if not updates:
            return
        with self.db.transaction() as conn:
            row = conn.execute("SELECT task_id FROM leases WHERE lease_id = ?", (lease_id,)).fetchone()
            if row is None:
                raise KeyError(lease_id)
            conn.execute(
                f"UPDATE leases SET {', '.join(updates)} WHERE lease_id = ?",
                (*values, lease_id),
            )
        self.db.append_event(
            "LEASE_EXECUTION_REFS_UPDATED",
            row["task_id"],
            {
                "lease_id": lease_id,
                "worktree_path": worktree_path,
                "branch_name": branch_name,
                "session_id": session_id,
                "worker_pid": worker_pid,
            },
        )

    def inspect_mutation_evidence(self, lease_id: str) -> LeaseMutationEvidence:
        conn = self.db.connect()
        try:
            row = conn.execute(
                """
                SELECT l.worktree_path, t.base_commit
                FROM leases l
                JOIN tasks t ON t.task_id = l.task_id
                WHERE l.lease_id = ?
                """,
                (lease_id,),
            ).fetchone()
            if row is None:
                raise KeyError(lease_id)
            return inspect_worktree_mutation(row["worktree_path"], row["base_commit"])
        finally:
            conn.close()

    def heartbeat(self, lease_id: str, now=None) -> None:
        now_s = iso_utc(now or utc_now())
        with self.db.transaction() as conn:
            row = conn.execute("SELECT task_id FROM leases WHERE lease_id = ?", (lease_id,)).fetchone()
            if row is None:
                raise KeyError(lease_id)
            conn.execute(
                "UPDATE leases SET heartbeat_at = ? WHERE lease_id = ?",
                (now_s, lease_id),
            )
        self.db.append_event("LEASE_HEARTBEAT", row["task_id"], {"lease_id": lease_id}, created_at=now_s)

    def mark_disconnected_after_mutation(self, lease_id: str) -> None:
        with self.db.transaction() as conn:
            row = conn.execute("SELECT task_id FROM leases WHERE lease_id = ?", (lease_id,)).fetchone()
            if row is None:
                raise KeyError(lease_id)
            conn.execute(
                "UPDATE leases SET disconnected_after_mutation = 1 WHERE lease_id = ?",
                (lease_id,),
            )
        self.db.append_event(
            "LEASE_MUTATION_DISCONNECT_RECORDED",
            row["task_id"],
            {"lease_id": lease_id},
        )

    def release(self, lease_id: str, next_status: str, reason: str) -> None:
        now_s = iso_utc()
        with self.db.transaction() as conn:
            row = conn.execute("SELECT * FROM leases WHERE lease_id = ?", (lease_id,)).fetchone()
            if row is None:
                raise KeyError(lease_id)
            old_status = conn.execute(
                "SELECT status FROM tasks WHERE task_id = ?",
                (row["task_id"],),
            ).fetchone()["status"]
            conn.execute("DELETE FROM leases WHERE lease_id = ?", (lease_id,))
            conn.execute(
                "UPDATE worker_slots SET state = 'IDLE' WHERE slot_id = ?",
                (row["slot_id"],),
            )
            conn.execute(
                "UPDATE tasks SET status = ?, updated_at = ? WHERE task_id = ?",
                (next_status, now_s, row["task_id"]),
            )
        self.db.append_event(
            "LEASE_RELEASED",
            row["task_id"],
            {"lease_id": lease_id, "next_status": next_status, "reason": reason},
            created_at=now_s,
        )
        self.db.append_event(
            "TASK_STATUS_CHANGED",
            row["task_id"],
            {"old_status": old_status, "new_status": next_status, "reason": reason},
            created_at=now_s,
        )

    def reclaim_expired(self, now=None) -> list[dict[str, object]]:
        now_dt = now or utc_now()
        now_s = iso_utc(now_dt)
        reclaimed = []
        with self.db.transaction() as conn:
            rows = conn.execute(
                """
                SELECT l.*, t.attempts, t.max_attempts, t.status, t.base_commit
                FROM leases l
                JOIN tasks t ON t.task_id = l.task_id
                WHERE l.lease_expires_at <= ?
                ORDER BY l.lease_expires_at, l.lease_id
                """,
                (now_s,),
            ).fetchall()
            for row in rows:
                evidence = inspect_worktree_mutation(row["worktree_path"], row["base_commit"])
                if row["disconnected_after_mutation"]:
                    next_status = "NEEDS_REVIEW"
                    reason = "expired_after_mutation"
                elif evidence.mutated is True:
                    next_status = "NEEDS_REVIEW"
                    reason = "expired_with_mutation_evidence"
                elif evidence.mutated is None:
                    next_status = "NEEDS_REVIEW"
                    reason = f"expired_mutation_unknown_{evidence.reason}"
                elif int(row["attempts"]) >= int(row["max_attempts"]):
                    next_status = "FAILED"
                    reason = "expired_retry_budget_exhausted"
                else:
                    next_status = "READY"
                    reason = "expired_requeued"
                conn.execute("DELETE FROM leases WHERE lease_id = ?", (row["lease_id"],))
                conn.execute(
                    "UPDATE worker_slots SET state = 'IDLE' WHERE slot_id = ?",
                    (row["slot_id"],),
                )
                conn.execute(
                    "UPDATE tasks SET status = ?, updated_at = ? WHERE task_id = ?",
                    (next_status, now_s, row["task_id"]),
                )
                reclaimed.append(
                    {
                        "lease_id": row["lease_id"],
                        "task_id": row["task_id"],
                        "next_status": next_status,
                        "reason": reason,
                        "mutation_evidence": evidence.reason,
                        "touched_files": list(evidence.touched_files),
                    }
                )
        for item in reclaimed:
            self.db.append_event("LEASE_RECLAIMED", item["task_id"], item, created_at=now_s)
            self.db.append_event(
                "TASK_STATUS_CHANGED",
                item["task_id"],
                {"old_status": "LEASED", "new_status": item["next_status"], "reason": item["reason"]},
                created_at=now_s,
            )
        return reclaimed

    def reclaim_stale_heartbeats(self, stale_after_seconds: int, now=None) -> list[dict[str, object]]:
        now_dt = now or utc_now()
        threshold = now_dt - timedelta(seconds=stale_after_seconds)
        with self.db.transaction() as conn:
            rows = conn.execute(
                """
                SELECT lease_id, lease_expires_at
                FROM leases
                WHERE heartbeat_at <= ?
                  AND lease_expires_at > ?
                """,
                (iso_utc(threshold), iso_utc(now_dt)),
            ).fetchall()
            for row in rows:
                conn.execute(
                    "UPDATE leases SET lease_expires_at = ? WHERE lease_id = ?",
                    (iso_utc(now_dt), row["lease_id"]),
                )
        return self.reclaim_expired(now_dt)

    def active_leases(self) -> list[dict[str, str]]:
        conn = self.db.connect()
        try:
            return [dict(row) for row in conn.execute("SELECT * FROM leases ORDER BY lease_id").fetchall()]
        finally:
            conn.close()


# ---------------------------------------------------------------------------
# TEIKOKU-IR B1-fix: pinned-model / required-window lease enforcement helpers. A task carrying a
# pinned model (the concrete model the RouteDecider chose) and/or a required_context_window is
# leased ONLY to a slot whose LANE can run the pinned model AND whose effective served window
# holds the required context. A task with neither is UNCHANGED (tier-only match). Kept as pure
# helpers off model_pool so acquire_ready enforces the SAME fit rule the decider picks against.
# ---------------------------------------------------------------------------
def _opt_row(row, key: str):
    try:
        return row[key] if key in row.keys() else None
    except (IndexError, KeyError, TypeError):
        return None


def _pin_and_window(row) -> tuple[str | None, int | None]:
    pinned = _opt_row(row, "pinned_model")
    req = _opt_row(row, "required_context_window")
    pinned = str(pinned).strip() if pinned else None
    req = int(req) if req not in (None, "") else None
    return (pinned or None), req


def _slot_can_serve(row, slot) -> bool:
    """True when ``slot`` can serve ``row``. An unpinned task with no required window always
    serves (tier-only, backward-compatible). A pinned task requires the slot's lane to run the
    pinned model; a required window requires the slot's effective served window (with the 0.8
    safety headroom) to hold it."""
    pinned, req = _pin_and_window(row)
    if not pinned and not req:
        return True
    import model_pool
    eff_model, eff_win = model_pool.effective_served_window(slot["model"], slot["harness"], pinned)
    if pinned and eff_model != pinned:
        return False
    if req and eff_win * model_pool.WINDOW_SAFETY_FRACTION < req:
        return False
    return True


def _collect_unserved_pins(candidates, slots) -> list[dict]:
    """Diagnostic rows for pinned/large candidates that NO idle same-tier slot can serve (so they
    are legitimately left READY, not silently dropped). A candidate that some slot COULD serve but
    lost a race is NOT reported (it will lease on a later pass)."""
    import model_pool
    out: list[dict] = []
    for row in candidates:
        pinned, req = _pin_and_window(row)
        if not pinned and not req:
            continue
        tier = str(row["model_tier"] or "").upper()
        tier_slots = [s for s in slots if str(s["model_tier"] or "").upper() == tier]
        if not tier_slots or any(_slot_can_serve(row, s) for s in tier_slots):
            continue
        wins = []
        for s in tier_slots:
            m, w = model_pool.effective_served_window(s["model"], s["harness"], pinned)
            wins.append({"slot": s["slot_id"], "seeded_model": s["model"], "served_model": m, "window": w})
        out.append({
            "task_id": row["task_id"],
            "pinned_model": pinned,
            "required_context_window": req,
            "model_tier": tier,
            "idle_tier_slots": wins,
            "reason": "no_idle_slot_serves_pinned_or_required_window",
        })
    return out


def inspect_worktree_mutation(worktree_path: str, base_commit: str) -> LeaseMutationEvidence:
    worktree = normalize_path(worktree_path)
    if not worktree.exists():
        return LeaseMutationEvidence(None, "worktree_missing", base_commit=base_commit)
    inside = _git_result(worktree, ["rev-parse", "--is-inside-work-tree"])
    if inside.returncode != 0 or inside.stdout.strip() != "true":
        return LeaseMutationEvidence(None, "not_a_git_worktree", base_commit=base_commit)
    head = _git_result(worktree, ["rev-parse", "HEAD"])
    if head.returncode != 0:
        return LeaseMutationEvidence(None, "head_unresolved", base_commit=base_commit)
    head_commit = head.stdout.strip()
    if not base_commit or base_commit == "HEAD":
        return LeaseMutationEvidence(
            None,
            "base_commit_not_pinned",
            head_commit=head_commit,
            base_commit=base_commit,
        )
    base = _git_result(worktree, ["rev-parse", base_commit])
    if base.returncode != 0:
        return LeaseMutationEvidence(
            None,
            "base_commit_unresolved",
            head_commit=head_commit,
            base_commit=base_commit,
        )
    status = _git_result(worktree, ["status", "--porcelain=v1", "--untracked-files=all"])
    if status.returncode != 0:
        return LeaseMutationEvidence(
            None,
            "status_unreadable",
            head_commit=head_commit,
            base_commit=base.stdout.strip(),
        )
    touched = tuple(_status_paths(status.stdout))
    base_resolved = base.stdout.strip()
    head_changed = head_commit != base_resolved
    if touched and head_changed:
        reason = "head_changed_and_worktree_dirty"
    elif touched:
        reason = "worktree_dirty"
    elif head_changed:
        reason = "head_changed"
    else:
        reason = "clean"
    return LeaseMutationEvidence(
        bool(touched or head_changed),
        reason,
        touched_files=touched,
        head_commit=head_commit,
        base_commit=base_resolved,
    )


def _status_paths(status_text: str) -> list[str]:
    paths: set[str] = set()
    for line in status_text.splitlines():
        if not line.strip():
            continue
        payload = line[3:] if len(line) > 3 else line.strip()
        if " -> " in payload:
            old_path, new_path = payload.split(" -> ", 1)
            for path in (old_path.strip('"'), new_path.strip('"')):
                if not _is_conductor_metadata_path(path):
                    paths.add(normalize_scope(path))
        else:
            path = payload.strip('"')
            if not _is_conductor_metadata_path(path):
                paths.add(normalize_scope(path))
    return sorted(path for path in paths if path)


def _git_result(cwd: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", "-C", str(cwd), *args],
        capture_output=True,
        text=True,
        errors="replace",
        timeout=60,
    )


def _safe_task_id(task_id: str) -> str:
    return re.sub(r"[^A-Za-z0-9_.-]+", "_", task_id)


def _is_conductor_metadata_path(path: str) -> bool:
    normalized = normalize_scope(path)
    return normalized == ".task" or normalized.startswith(".task/")
