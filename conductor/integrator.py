from __future__ import annotations

import re
import shutil
import subprocess
import time
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable

from conductor_config import normalize_path
from conductor_db import ControlDB
from gate_runner import run_gates
from redaction import redact_text
from time_utils import iso_utc


GateRunner = Callable[..., dict[str, Any]]

# Failed-land integrate_* trees are kept this long for forensics, then reaped by the
# next integration on the same repo. DONE trees are removed immediately (merged).
INTEGRATION_SWEEP_HOURS = 24.0


class IntegrationBusy(RuntimeError):
    pass


class IntegrationRejected(RuntimeError):
    pass


@dataclass(frozen=True)
class IntegrationResult:
    integration_id: str
    task_id: str
    status: str
    task_status: str
    integration_worktree_path: str
    patch_artifact: str
    post_gate_verdict: str | None
    reason: str

    def to_json_dict(self) -> dict[str, Any]:
        return {
            "integration_id": self.integration_id,
            "task_id": self.task_id,
            "status": self.status,
            "task_status": self.task_status,
            "integration_worktree_path": self.integration_worktree_path,
            "patch_artifact": self.patch_artifact,
            "post_gate_verdict": self.post_gate_verdict,
            "reason": self.reason,
        }


class SerializedIntegrator:
    def __init__(
        self,
        db_path: str | Path,
        *,
        repo_root: str | Path,
        workflow_root: str | Path,
        artifact_root: str | Path,
        worktree_root: str | Path,
        base_ref: str = "HEAD",
        allow_local_commits: bool = False,
        allow_push: bool = False,
        block_dirty_checkout: bool = True,
        gate_runner: GateRunner = run_gates,
        packs: tuple[str, ...] = ("rust",),
    ) -> None:
        self.db = ControlDB(db_path)
        self.repo_root = normalize_path(repo_root)
        self.workflow_root = normalize_path(workflow_root)
        self.artifact_root = normalize_path(artifact_root)
        self.worktree_root = normalize_path(worktree_root)
        self.base_ref = base_ref
        self.allow_local_commits = allow_local_commits
        self.allow_push = allow_push
        self.block_dirty_checkout = block_dirty_checkout
        self.gate_runner = gate_runner
        # TEIKOKU P3 BLOCK-1: the integrator must run the SAME pack-keyed gate set the
        # loop's verify gate used (conductor_loop.py threads packs=active_packs(project)).
        # Defaults to the rust pack so the ELAI/Rust path is unchanged; a generic/unity
        # project now gets its generic gate set at the candidate AND post-merge gate.
        self.packs = packs

    def integrate_verified_task(
        self,
        *,
        task_id: str,
        source_worktree_path: str | Path,
        source_base_commit: str,
        commit_message: str | None = None,
    ) -> IntegrationResult:
        """Integrate one verified task, then clean up after ourselves.

        Every attempt creates a throwaway `integrate_*` worktree plus a
        `conductor/integrate/*` branch. Nothing ever consumed them afterwards
        (the patch artifact + DB rows are the audit record), so they leaked:
        72 dead trees / 458 stale registrations found on 2026-08-07. Policy:
        a DONE integration is merged into base_ref, so its tree+branch are
        removed at once; a failed one keeps its tree for forensics but is
        swept once it is older than INTEGRATION_SWEEP_HOURS.
        """
        result = self._integrate_verified_task_inner(
            task_id=task_id,
            source_worktree_path=source_worktree_path,
            source_base_commit=source_base_commit,
            commit_message=commit_message,
        )
        try:
            if result.status == "DONE":
                self._remove_integration_tree(Path(result.integration_worktree_path))
            self._sweep_stale_integrations()
        except Exception:  # noqa: BLE001 - cleanup must never mask the verdict
            pass
        return result

    def _remove_integration_tree(self, integration_path: Path) -> None:
        """Remove one integrate_* worktree and its conductor/integrate/* branch."""
        name = integration_path.name
        if not name.startswith("integrate_"):
            return  # refuse to touch anything we did not create
        if integration_path.exists():
            rc = self._git_result(
                self.repo_root, ["worktree", "remove", "--force", str(integration_path)]
            )
            if rc.returncode != 0 and integration_path.exists():
                shutil.rmtree(integration_path, ignore_errors=True)
        self._git_result(self.repo_root, ["worktree", "prune"])
        # integrate_{safe_task_id}_{id8} -> conductor/integrate/{safe_task_id}-{id8}
        stem = name[len("integrate_"):]
        if "_" in stem:
            safe_task_id, id8 = stem.rsplit("_", 1)
            self._git_result(
                self.repo_root, ["branch", "-D", f"conductor/integrate/{safe_task_id}-{id8}"]
            )

    def _sweep_stale_integrations(self, max_age_hours: float = INTEGRATION_SWEEP_HOURS) -> None:
        """Reap integrate_* trees older than the window (failed-land forensics expired)."""
        cutoff = time.time() - max_age_hours * 3600.0
        try:
            candidates = list(self.worktree_root.glob("integrate_*"))
        except OSError:
            return
        for tree in candidates:
            try:
                if tree.is_dir() and tree.stat().st_mtime < cutoff:
                    self._remove_integration_tree(tree)
            except OSError:
                continue

    def _integrate_verified_task_inner(
        self,
        *,
        task_id: str,
        source_worktree_path: str | Path,
        source_base_commit: str,
        commit_message: str | None = None,
    ) -> IntegrationResult:
        integration_id = str(uuid.uuid4())
        safe_task_id = re.sub(r"[^A-Za-z0-9_.-]+", "_", task_id)
        integration_path = self.worktree_root / f"integrate_{safe_task_id}_{integration_id[:8]}"
        patch_artifact = self.artifact_root / task_id / "integrations" / f"{integration_id}.patch"
        source_worktree = normalize_path(source_worktree_path)

        self._start_integration(
            integration_id,
            task_id,
            source_worktree,
            source_base_commit,
            integration_path,
            patch_artifact,
        )
        try:
            if self.block_dirty_checkout and self._repo_is_dirty():
                return self._finish(
                    integration_id,
                    task_id,
                    "NEEDS_REVIEW",
                    "NEEDS_REVIEW",
                    integration_path,
                    patch_artifact,
                    None,
                    "dirty_checkout_blocks_integration",
                )

            current_head = self._git_stdout(self.repo_root, ["rev-parse", self.base_ref]).strip()
            self._make_untracked_visible_to_diff(source_worktree)
            patch_text = self._git_stdout(source_worktree, ["diff", "--binary", source_base_commit])
            patch_artifact.parent.mkdir(parents=True, exist_ok=True)
            # Write bytes, not text: Path.write_text on Windows translates \n to
            # \r\n, which corrupts git diff hunk content and makes `git apply`
            # fail with "patch does not apply". The patch must keep LF endings.
            # NEVER redact the apply payload: redact_text turns path tokens into
            # <REDACTED_TOKEN> and git apply then fails with rebase_apply_conflict
            # (measured 2026-08-04 J-54 UNIT8). Redacted copy is sidecar only.
            patch_artifact.write_bytes(patch_text.encode("utf-8"))
            redacted_side = patch_artifact.with_suffix(patch_artifact.suffix + ".redacted")
            try:
                redacted_side.write_bytes(redact_text(patch_text).encode("utf-8"))
            except OSError:
                pass
            self._record_artifact(task_id, "integration_patch", patch_artifact)

            # ELAI-walk scope guard: the .ELAI_workflow repo tracks ONLY .elai_cc/.
            # A worker diff that touches docs/ or workflow/ (the glm-orchestrator
            # committer bug) is refused here so it can never land. Only enforced on
            # the elai pack; other projects integrate their whole diff unchanged.
            if "elai" in self.packs and patch_text.strip():
                _touched = set()
                for _ln in patch_text.splitlines():
                    if _ln.startswith("diff --git "):
                        _m = re.match(r"diff --git a/(.+?) b/(.+)$", _ln)
                        if _m:
                            _touched.add(_m.group(1))
                            _touched.add(_m.group(2))
                _bad = sorted(p for p in _touched if not p.startswith(".elai_cc/"))
                if _bad:
                    return self._finish(
                        integration_id,
                        task_id,
                        "NEEDS_REVIEW",
                        "NEEDS_REVIEW",
                        integration_path,
                        patch_artifact,
                        None,
                        "diff_touches_non_elai_cc_path",
                        summary="refused non-.elai_cc paths: " + ", ".join(_bad[:10]),
                    )

            branch = f"conductor/integrate/{safe_task_id}-{integration_id[:8]}"
            self._git(self.repo_root, ["worktree", "add", "-b", branch, str(integration_path), current_head])
            if patch_text.strip():
                apply_result = self._git_result(
                    integration_path,
                    ["apply", "--3way", "--index", str(patch_artifact)],
                )
                if apply_result.returncode != 0:
                    return self._finish(
                        integration_id,
                        task_id,
                        "CONFLICT",
                        "NEEDS_REVIEW",
                        integration_path,
                        patch_artifact,
                        None,
                        "rebase_apply_conflict",
                        summary=apply_result.stderr or apply_result.stdout,
                    )
            self._mark_applied(integration_id, task_id)

            candidate_gate = self.gate_runner(
                task_id=task_id,
                worktree_path=str(integration_path),
                base_commit=current_head,
                db_path=str(self.db.db_path),
                workflow_root=str(self.workflow_root),
                repo_root=str(self.repo_root),
                artifact_root=str(self.artifact_root),
                packs=self.packs,
            )
            candidate_gate_verdict = str(candidate_gate.get("overall_verdict", "ERROR"))
            candidate_gate_run_id = self._latest_gate_run_id(task_id)
            if candidate_gate_verdict != "PASS":
                return self._finish(
                    integration_id,
                    task_id,
                    "POST_GATE_FAILED",
                    "NEEDS_REVIEW",
                    integration_path,
                    patch_artifact,
                    candidate_gate_verdict,
                    "candidate_gate_failed",
                    post_gate_run_id=candidate_gate_run_id,
                    summary=str(candidate_gate.get("structured_feedback", "")),
                )

            if not self.allow_local_commits:
                return self._finish(
                    integration_id,
                    task_id,
                    "NEEDS_REVIEW",
                    "NEEDS_REVIEW",
                    integration_path,
                    patch_artifact,
                    candidate_gate_verdict,
                    "local_commits_disabled_after_candidate_gate_pass",
                    post_gate_run_id=candidate_gate_run_id,
                )

            if patch_text.strip():
                self._git(integration_path, ["commit", "-m", commit_message or f"integrate {task_id}"])
                self._git(self.repo_root, ["merge", "--ff-only", branch])
            post_merge_head = self._git_stdout(self.repo_root, ["rev-parse", self.base_ref]).strip()
            post_merge_gate = self.gate_runner(
                task_id=task_id,
                worktree_path=str(self.repo_root),
                base_commit=current_head,
                db_path=str(self.db.db_path),
                workflow_root=str(self.workflow_root),
                repo_root=str(self.repo_root),
                artifact_root=str(self.artifact_root),
                packs=self.packs,
            )
            post_gate_verdict = str(post_merge_gate.get("overall_verdict", "ERROR"))
            post_gate_run_id = self._latest_gate_run_id(task_id)
            if post_gate_verdict != "PASS":
                # Atomic failure: undo the ff-only merge on the shared checkout so
                # the rejected commit cannot ride out on the NEXT land's push
                # (push moves the whole branch, not one commit). The commit stays
                # reachable on the integration branch for forensics; the lock
                # serializes lands, so HEAD is exactly the merge we just made.
                rollback = "rollback_skipped_head_moved"
                if self._git_stdout(self.repo_root, ["rev-parse", "HEAD"]).strip() == post_merge_head:
                    self._git(self.repo_root, ["reset", "--hard", current_head])
                    rollback = f"rolled_back_to={current_head}"
                # Durable front-door marker for audits/tests (A-CTRL-OPUS-LAND).
                self.db.append_event(
                    "INTEGRATION_POST_MERGE_REVERTED",
                    task_id,
                    {
                        "integration_id": integration_id,
                        "post_gate_verdict": post_gate_verdict,
                        "rollback": rollback,
                        "reason": "post_merge_gate_failed",
                    },
                )
                return self._finish(
                    integration_id,
                    task_id,
                    "POST_GATE_FAILED",
                    "NEEDS_REVIEW",
                    integration_path,
                    patch_artifact,
                    post_gate_verdict,
                    "post_merge_gate_failed",
                    post_gate_run_id=post_gate_run_id,
                    summary=f"{rollback}\n" + str(post_merge_gate.get("structured_feedback", "")),
                )
            # Push ONLY after the post-merge gate passes. 2026-08-08 00:34Z and
            # 02:03Z: two lands pushed first, then post-merge gate_freeze_check
            # BLOCKed, so a land REPORTED as failed was already public on origin
            # (9d22dbe, dab4ceb). A POST_GATE_FAILED merge must stay local-only
            # (recoverable); a push failure after PASS is transport, not verdict,
            # and still returns DONE (opus_land maps it to LANDED_PUSH_FAILED).
            if patch_text.strip() and self.allow_push:
                self._push_after_merge(task_id)
            return self._finish(
                integration_id,
                task_id,
                "DONE",
                "DONE",
                integration_path,
                patch_artifact,
                post_gate_verdict,
                "post_merge_gate_passed",
                post_gate_run_id=post_gate_run_id,
                summary=f"merged_head={post_merge_head}",
            )
        except Exception as exc:
            return self._finish(
                integration_id,
                task_id,
                "NEEDS_REVIEW",
                "NEEDS_REVIEW",
                integration_path,
                patch_artifact,
                None,
                "ambiguous_integration_failure",
                summary=str(exc),
            )

    def _push_after_merge(self, task_id: str) -> None:
        """Best-effort push of the integrated commit to origin. A push failure
        never fails the integration -- the commit is already merged locally and
        recoverable. Only the elai walk enables this (config.safety.allow_push)."""
        branch = self._git_stdout(self.repo_root, ["rev-parse", "--abbrev-ref", "HEAD"]).strip()
        if not branch or branch == "HEAD":
            branch = "main"
        result = self._git_result(self.repo_root, ["push", "origin", branch])
        if result.returncode != 0:
            self.db.append_event(
                "INTEGRATION_PUSH_FAILED",
                task_id,
                {"branch": branch, "stderr": (result.stderr or result.stdout or "")[:500]},
            )
        else:
            self.db.append_event("INTEGRATION_PUSHED", task_id, {"branch": branch})

    def _start_integration(
        self,
        integration_id: str,
        task_id: str,
        source_worktree: Path,
        source_base_commit: str,
        integration_path: Path,
        patch_artifact: Path,
    ) -> None:
        now = iso_utc()
        with self.db.transaction() as conn:
            active = conn.execute(
                """
                SELECT integration_id, task_id
                FROM integrations
                WHERE finished_at IS NULL
                  AND status IN ('PENDING','APPLIED')
                ORDER BY started_at
                LIMIT 1
                """
            ).fetchone()
            if active is not None:
                raise IntegrationBusy(f"integration already active: {active['integration_id']}")
            task = conn.execute(
                "SELECT status FROM tasks WHERE task_id = ?",
                (task_id,),
            ).fetchone()
            if task is None:
                raise KeyError(task_id)
            if task["status"] != "VERIFIED":
                raise IntegrationRejected(f"task must be VERIFIED before integration: {task['status']}")
            conn.execute(
                """
                INSERT INTO integrations (
                  integration_id, task_id, source_worktree_path, source_base_commit,
                  integration_worktree_path, status, started_at, patch_artifact
                )
                VALUES (?, ?, ?, ?, ?, 'PENDING', ?, ?)
                """,
                (
                    integration_id,
                    task_id,
                    str(source_worktree),
                    source_base_commit,
                    str(integration_path),
                    now,
                    str(patch_artifact),
                ),
            )
            conn.execute(
                "UPDATE tasks SET status = 'INTEGRATING', updated_at = ? WHERE task_id = ?",
                (now, task_id),
            )
        self.db.append_event(
            "INTEGRATION_STARTED",
            task_id,
            {
                "integration_id": integration_id,
                "source_worktree_path": str(source_worktree),
                "integration_worktree_path": str(integration_path),
            },
            created_at=now,
        )
        self.db.append_event(
            "TASK_STATUS_CHANGED",
            task_id,
            {"old_status": "VERIFIED", "new_status": "INTEGRATING", "reason": "integration_started"},
            created_at=now,
        )

    def _mark_applied(self, integration_id: str, task_id: str) -> None:
        now = iso_utc()
        with self.db.transaction() as conn:
            conn.execute(
                "UPDATE integrations SET status = 'APPLIED' WHERE integration_id = ?",
                (integration_id,),
            )
        self.db.append_event("INTEGRATION_APPLIED", task_id, {"integration_id": integration_id}, created_at=now)

    def _finish(
        self,
        integration_id: str,
        task_id: str,
        integration_status: str,
        task_status: str,
        integration_path: Path,
        patch_artifact: Path,
        post_gate_verdict: str | None,
        reason: str,
        *,
        post_gate_run_id: str | None = None,
        summary: str | None = None,
    ) -> IntegrationResult:
        now = iso_utc()
        with self.db.transaction() as conn:
            old_status_row = conn.execute(
                "SELECT status FROM tasks WHERE task_id = ?",
                (task_id,),
            ).fetchone()
            old_status = old_status_row["status"] if old_status_row else None
            conn.execute(
                """
                UPDATE integrations
                SET status = ?, finished_at = ?, post_gate_run_id = ?
                WHERE integration_id = ?
                """,
                (integration_status, now, post_gate_run_id, integration_id),
            )
            conn.execute(
                "UPDATE tasks SET status = ?, updated_at = ? WHERE task_id = ?",
                (task_status, now, task_id),
            )
        payload = {
            "integration_id": integration_id,
            "status": integration_status,
            "task_status": task_status,
            "post_gate_verdict": post_gate_verdict,
            "reason": reason,
        }
        if summary:
            payload["summary"] = redact_text(summary)[:2000]
        self.db.append_event("INTEGRATION_FINISHED", task_id, payload, created_at=now)
        self.db.append_event(
            "TASK_STATUS_CHANGED",
            task_id,
            {"old_status": old_status, "new_status": task_status, "reason": reason},
            created_at=now,
        )
        return IntegrationResult(
            integration_id=integration_id,
            task_id=task_id,
            status=integration_status,
            task_status=task_status,
            integration_worktree_path=str(integration_path),
            patch_artifact=str(patch_artifact),
            post_gate_verdict=post_gate_verdict,
            reason=reason,
        )

    def _record_artifact(self, task_id: str, kind: str, path: Path) -> None:
        with self.db.transaction() as conn:
            conn.execute(
                """
                INSERT INTO artifacts (artifact_id, task_id, kind, path, created_at, redacted)
                VALUES (?, ?, ?, ?, ?, 1)
                """,
                (str(uuid.uuid4()), task_id, kind, str(path), iso_utc()),
            )

    def _repo_is_dirty(self) -> bool:
        result = self._git_result(self.repo_root, ["status", "--porcelain"])
        return bool(result.stdout.strip())

    def _latest_gate_run_id(self, task_id: str) -> str | None:
        conn = self.db.connect()
        try:
            row = conn.execute(
                """
                SELECT gate_run_id
                FROM gate_runs
                WHERE task_id = ?
                ORDER BY finished_at DESC, started_at DESC
                LIMIT 1
                """,
                (task_id,),
            ).fetchone()
            return str(row["gate_run_id"]) if row else None
        finally:
            conn.close()

    def _git_stdout(self, cwd: Path, args: list[str]) -> str:
        result = self._git_result(cwd, args)
        if result.returncode != 0:
            raise RuntimeError(redact_text(result.stderr or result.stdout))
        return result.stdout

    def _git(self, cwd: Path, args: list[str]) -> None:
        result = self._git_result(cwd, args)
        if result.returncode != 0:
            raise RuntimeError(redact_text(result.stderr or result.stdout))

    def _make_untracked_visible_to_diff(self, cwd: Path) -> None:
        result = self._git_result(cwd, ["ls-files", "--others", "--exclude-standard", "-z"])
        if result.returncode != 0 or not result.stdout:
            return
        paths = [path for path in result.stdout.split("\0") if path]
        if paths:
            self._git(cwd, ["add", "-N", "--", *paths])

    def _git_result(self, cwd: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["git", "-C", str(cwd), *args],
            capture_output=True,
            text=True,
            errors="replace",
            timeout=180,
        )
