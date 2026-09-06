from __future__ import annotations

import json
import os
import subprocess
import time
import unittest
from dataclasses import replace
from pathlib import Path

from tests.helpers import TempDirMixin
from tests.test_models import sample_task
from conductor_db import ControlDB
from leases import LeaseManager
from time_utils import utc_now
from worker_adapter import (
    ProviderResult,
    WorkerAdapter,
    classify_rate_limit,
    classify_transient_provider_error,
    classify_error_category,
    _head_tail,
    _wsl_path,
)


class HeadTailTests(unittest.TestCase):
    """N-2/N-14/O-1 regression (2026-08-12): head-only truncation ([:4000] then [:1200])
    amputated the typed recon VERDICT line, which lives at the TAIL of the worker's notes."""

    def test_short_text_unchanged(self):
        self.assertEqual(_head_tail("VERDICT: AMEND", 100), "VERDICT: AMEND")

    def test_long_text_keeps_tail_verdict(self):
        text = ("orientation chatter. " * 400) + "\nSTATUS: DONE-CANDIDATE\nVERDICT: AMEND"
        bounded = _head_tail(text, 1200)
        self.assertLessEqual(len(bounded), 1300)
        self.assertIn("VERDICT: AMEND", bounded)
        self.assertTrue(bounded.startswith("orientation chatter."))


class FakeProvider:
    def __init__(
        self,
        writes: dict[str, str] | None = None,
        exit_code: int = 0,
        stdout: str = "session_id: fake-session",
        stderr: str = "",
        hold_seconds: float = 0.0,
    ):
        self.writes = writes or {}
        self.exit_code = exit_code
        self.stdout = stdout
        self.stderr = stderr
        self.hold_seconds = hold_seconds
        self.commands: list[list[str]] = []
        self.envs: list[dict[str, str]] = []
        self.cancelled = False

    def run(
        self,
        command: list[str],
        cwd: Path,
        timeout_seconds: int,
        env: dict[str, str],
        input_text: str | None = None,
    ) -> ProviderResult:
        if self.hold_seconds:
            time.sleep(self.hold_seconds)
        self.commands.append(command)
        self.envs.append(env)
        for rel_path, content in self.writes.items():
            path = cwd / rel_path
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        if "--export" in command:
            atif = Path(command[command.index("--export") + 1])
            atif.parent.mkdir(parents=True, exist_ok=True)
            atif.write_text(
                json.dumps(
                    {
                        "final_message": "session_id: fake-session api_key=SHOULD_NOT_LEAK",
                        "credential": "super-secret-value",
                    }
                ),
                encoding="utf-8",
            )
        return ProviderResult(self.exit_code, self.stdout, self.stderr)

    def cancel(self) -> None:
        self.cancelled = True


class WorkerAdapterTests(TempDirMixin, unittest.TestCase):
    def test_dry_run_preserves_no_done_and_renders_config_flag_path(self) -> None:
        temp = self.make_tempdir()
        adapter = WorkerAdapter(
            {
                "worktree_base": str(temp / "worktrees"),
                "artifact_root": str(temp / "artifacts"),
                "repo_root": str(temp / "repo"),
                "devin_bin_path": str(temp / "devin"),
                "agent_root": str(temp / "agents"),
            }
        )
        task = sample_task("B-1-EXECUTE-1")
        result = adapter.dispatch(task, dry_run=True)
        self.assertEqual(result.worker_status_hint, "NO_CHANGE")
        task_dir = temp / "worktrees" / "dryrun_B-1-EXECUTE-1" / ".task"
        self.assertTrue((task_dir / "taskspec.json").exists())
        self.assertTrue((task_dir / "role.config.yaml").exists())
        # P4-B: devin.exe retired; an unspecified slot is the devin-free base-cc lane.
        # The base-cc command shape (build_free_api_command) replaces the old devin.exe path.
        slot = adapter.select_slot("FREE")
        command, _ = adapter._build_worker_command(
            slot,
            task,
            task_dir / "taskspec.txt",
            task_dir / "role.config.yaml",
            task_dir / "result.atif.json",
            task_dir.parent,
        )
        self.assertIn("--print", command)
        self.assertFalse(any("devin.exe" in str(c) for c in command))

    def test_codex_command_uses_wsl_and_codex_home(self) -> None:
        temp = self.make_tempdir()
        repo = make_git_repo(temp / "repo")
        adapter = WorkerAdapter(
            {
                "worktree_base": str(temp / "worktrees"),
                "artifact_root": str(temp / "artifacts"),
                "repo_root": str(repo),
                "devin_bin_path": str(temp / "devin"),
                "agent_root": str(temp / "agents"),
                "workers": [
                    {
                        "slot_id": "w1",
                        "model": "gpt-5.5",
                        "model_tier": "FREE",
                        "harness": "codex",
                        "account_id": "account-a",
                        "codex_home": "/root/.codex-accounts/account-a",
                    }
                ],
            }
        )
        task = scoped_task("B-1-CODEX-1")
        result = adapter.dispatch(task, dry_run=True)
        task_dir = Path(result.worktree_path) / ".task"
        command, input_text = adapter._build_worker_command(
            adapter.select_slot("FREE"),
            task,
            task_dir / "taskspec.txt",
            task_dir / "role.config.yaml",
            task_dir / "result.atif.json",
            Path(result.worktree_path),
        )
        self.assertEqual(command[0], "wsl")
        self.assertEqual(command[2], "codex-vm")
        self.assertIn("codex", command)
        self.assertIn("exec", command)
        self.assertIn("--config", command)
        self.assertIn("/root/.codex-accounts/account-a/config.toml", command)
        self.assertEqual(command[-1], "-")
        self.assertIn("You are executing a bounded ELAI Conductor TaskSpec.", input_text)

    def test_codex_event_stream_parsing(self) -> None:
        temp = self.make_tempdir()
        repo = make_git_repo(temp / "repo")
        events = [
            json.dumps({"type": "turn.started"}),
            json.dumps({"type": "item.completed", "item": {"type": "agent_message", "text": "first"}}),
            json.dumps({"type": "item.completed", "item": {"type": "agent_message", "text": "final message"}}),
            json.dumps({"type": "turn.completed"}),
        ]
        provider = FakeProvider(writes={"src/lib.rs": "pub fn codex_changed() {}\n"}, stdout="\n".join(events))
        adapter = WorkerAdapter(
            {
                "worktree_base": str(temp / "worktrees"),
                "artifact_root": str(temp / "artifacts"),
                "repo_root": str(repo),
                "devin_bin_path": str(temp / "devin"),
                "agent_root": str(temp / "agents"),
                "workers": [
                    {
                        "slot_id": "w1",
                        "model": "gpt-5.5",
                        "model_tier": "FREE",
                        "harness": "codex",
                        "account_id": "account-a",
                        "codex_home": "/root/.codex-accounts/account-a",
                    }
                ],
            },
            provider=provider,
        )
        result = adapter.dispatch(scoped_task("B-1-CODEX-PARSE-1"))
        self.assertEqual(result.worker_status_hint, "APPLIED")
        self.assertEqual(result.model_used, "gpt-5.5")
        self.assertIn("final message", result.sanitized_notes)
        self.assertTrue(Path(result.atif_artifact).exists())
        atif_data = json.loads(Path(result.atif_artifact).read_text(encoding="utf-8"))
        self.assertTrue(atif_data["turn_completed"])
        self.assertIn("final message", atif_data["final_message"])

    def test_rate_limit_classification(self) -> None:
        self.assertTrue(classify_rate_limit("429 quota exhausted", 1, False))
        self.assertTrue(classify_rate_limit("stdout says rate limit exceeded", 1, False))
        self.assertTrue(classify_rate_limit("Concurrency limit exceeded for account", 1, False))
        self.assertFalse(classify_rate_limit("compiler failed", 1, False))
        self.assertFalse(classify_rate_limit("", 0, False))

    def test_transient_provider_error_classification(self) -> None:
        self.assertTrue(
            classify_transient_provider_error(
                'unexpected status 524: error code: 524, cf-ray: abc123', 1, False
            )
        )
        self.assertTrue(
            classify_transient_provider_error(
                "stream disconnected before completion: idle timeout waiting for SSE", 1, False
            )
        )
        self.assertTrue(
            classify_transient_provider_error(
                "Reconnecting... 1/2 (unexpected status 524)", 1, False
            )
        )
        self.assertTrue(
            classify_transient_provider_error("connection reset by peer", 1, False)
        )
        self.assertTrue(
            classify_transient_provider_error("ETIMEDOUT", 1, True)
        )
        self.assertFalse(
            classify_transient_provider_error("compiler error", 1, False)
        )
        self.assertFalse(
            classify_transient_provider_error("", 0, False)
        )

    def test_error_category_routes_524_as_transient(self) -> None:
        category = classify_error_category(
            rate_limited=False,
            timed_out=False,
            cancelled=False,
            violations=[],
            exit_code=1,
            combined_output="unexpected status 524: error code: 524, cf-ray: xyz",
        )
        self.assertEqual(category, "transient_provider_error")

    def test_error_category_routes_concurrency_as_rate_limited(self) -> None:
        category = classify_error_category(
            rate_limited=True,
            timed_out=False,
            cancelled=False,
            violations=[],
            exit_code=1,
            combined_output="concurrency limit exceeded",
        )
        self.assertEqual(category, "rate_limited")

    def test_fake_provider_runs_in_isolated_worktree_and_routes_both_accounts(self) -> None:
        temp = self.make_tempdir()
        repo = make_git_repo(temp / "repo")
        provider = FakeProvider({"src/lib.rs": "pub fn changed() {}\n"})
        adapter = WorkerAdapter(
            {
                "worktree_base": str(temp / "worktrees"),
                "artifact_root": str(temp / "artifacts"),
                "repo_root": str(repo),
                "devin_bin_path": str(temp / "devin"),
                "agent_root": str(temp / "agents"),
                "workers": [
                    {"slot_id": "w1", "model": "swe-1.6", "model_tier": "FREE", "account_id": "account-a"},
                    {"slot_id": "w2", "model": "glm-5.2", "model_tier": "FREE", "account_id": "account-b"},
                ],
            },
            provider=provider,
        )

        first = adapter.dispatch(scoped_task("B-10-EXECUTE-1"))
        second = adapter.dispatch(scoped_task("B-11-EXECUTE-1"))

        self.assertEqual(first.model_used, "swe-1.6")
        self.assertEqual(second.model_used, "glm-5.2")
        self.assertEqual(provider.envs[0]["CONDUCTOR_WORKER_ACCOUNT"], "account-a")
        self.assertEqual(provider.envs[1]["CONDUCTOR_WORKER_ACCOUNT"], "account-b")
        self.assertEqual(first.worker_status_hint, "APPLIED")
        self.assertIn("src/lib.rs", first.touched_files)
        self.assertTrue(first.branch_name.startswith("conductor/B-10-EXECUTE-1-"))
        self.assertNotIn("SHOULD_NOT_LEAK", first.sanitized_notes)
        self.assertNotIn("super-secret-value", Path(first.atif_artifact).read_text(encoding="utf-8"))

    def test_worker_env_does_not_inherit_secret_keys(self) -> None:
        temp = self.make_tempdir()
        repo = make_git_repo(temp / "repo")
        provider = FakeProvider()
        adapter = WorkerAdapter(
            {
                "worktree_base": str(temp / "worktrees"),
                "artifact_root": str(temp / "artifacts"),
                "repo_root": str(repo),
                "devin_bin_path": str(temp / "devin"),
                "agent_root": str(temp / "agents"),
                "workers": [{"slot_id": "w1", "model": "swe-1.6", "model_tier": "FREE"}],
            },
            provider=provider,
        )
        old = os.environ.get("OPENAI_API_KEY")
        os.environ["OPENAI_API_KEY"] = "should-not-reach-worker"
        try:
            adapter.dispatch(scoped_task("B-10-ENV-1"))
        finally:
            if old is None:
                os.environ.pop("OPENAI_API_KEY", None)
            else:
                os.environ["OPENAI_API_KEY"] = old
        self.assertNotIn("OPENAI_API_KEY", provider.envs[0])

    def test_dispatch_uses_lease_owned_worktree_and_branch(self) -> None:
        temp = self.make_tempdir()
        repo = make_git_repo(temp / "repo")
        lease_path = temp / "leased-worktree"
        provider = FakeProvider({"src/lib.rs": "pub fn leased() {}\n"})
        adapter = WorkerAdapter(
            {
                "worktree_base": str(temp / "worktrees"),
                "artifact_root": str(temp / "artifacts"),
                "repo_root": str(repo),
                "devin_bin_path": str(temp / "devin"),
                "agent_root": str(temp / "agents"),
                "workers": [{"slot_id": "w1", "model": "swe-1.6", "model_tier": "FREE"}],
            },
            provider=provider,
        )

        result = adapter.dispatch(
            scoped_task("B-10-LEASE-1"),
            slot_id="w1",
            worktree_path=lease_path,
            branch_name="conductor/leased-branch",
        )

        self.assertEqual(Path(result.worktree_path), lease_path)
        self.assertEqual(result.branch_name, "conductor/leased-branch")
        self.assertTrue((lease_path / ".git").exists())

    def test_dispatch_threads_lease_heartbeat_interval_to_live_path(self) -> None:
        temp = self.make_tempdir()
        repo = make_git_repo(temp / "repo")
        db_path = temp / "control.db"
        db = ControlDB(db_path)
        db.init(reset=True)
        db.seed_worker_slot("w1", "swe-1.6", model_tier="FREE", max_concurrency=1)
        task = scoped_task("LIVE-U8-HEARTBEAT-1")
        db.upsert_task(task, status="READY")
        lease = LeaseManager(db_path, worktree_root=temp / "worktrees").acquire_ready(
            task_id=task.task_id,
            slot_id="w1",
            worktree_path=str(temp / "leased-worktree"),
            now=utc_now(),
        )
        self.assertIsNotNone(lease)

        provider = FakeProvider(hold_seconds=0.12)
        adapter = WorkerAdapter(
            {
                "worktree_base": str(temp / "worktrees"),
                "artifact_root": str(temp / "artifacts"),
                "repo_root": str(repo),
                "devin_bin_path": str(temp / "devin"),
                "agent_root": str(temp / "agents"),
                "workers": [{"slot_id": "w1", "model": "swe-1.6", "model_tier": "FREE"}],
            },
            provider=provider,
        )

        result = adapter.dispatch(
            task,
            slot_id="w1",
            worktree_path=lease.worktree_path,
            branch_name=lease.branch_name,
            lease_id=lease.lease_id,
            db_path=db_path,
            lease_heartbeat_interval_seconds=0.01,
        )

        self.assertEqual(result.task_id, task.task_id)
        conn = db.connect()
        try:
            heartbeat_count = conn.execute(
                """
                SELECT COUNT(*) AS n
                FROM events
                WHERE event_type = 'LEASE_HEARTBEAT' AND task_id = ?
                """,
                (task.task_id,),
            ).fetchone()["n"]
        finally:
            conn.close()
        self.assertGreaterEqual(heartbeat_count, 1)

    def test_fake_provider_blocks_allowed_write_violation(self) -> None:
        temp = self.make_tempdir()
        repo = make_git_repo(temp / "repo")
        provider = FakeProvider({"outside.txt": "changed outside scope\n"})
        adapter = WorkerAdapter(
            {
                "worktree_base": str(temp / "worktrees"),
                "artifact_root": str(temp / "artifacts"),
                "repo_root": str(repo),
                "devin_bin_path": str(temp / "devin"),
                "agent_root": str(temp / "agents"),
                "workers": [{"slot_id": "w1", "model": "swe-1.6", "model_tier": "FREE"}],
            },
            provider=provider,
        )
        result = adapter.dispatch(scoped_task("B-12-EXECUTE-1"))
        self.assertEqual(result.worker_status_hint, "BLOCKED")
        self.assertEqual(result.error_category, "allowed_write_violation")
        self.assertIn("outside.txt", result.sanitized_notes)

    def test_cancel_reaches_provider(self) -> None:
        temp = self.make_tempdir()
        provider = FakeProvider()
        adapter = WorkerAdapter(
            {
                "worktree_base": str(temp / "worktrees"),
                "artifact_root": str(temp / "artifacts"),
                "repo_root": str(temp / "repo"),
                "devin_bin_path": str(temp / "devin"),
                "agent_root": str(temp / "agents"),
            },
            provider=provider,
        )
        adapter.cancel()
        self.assertTrue(provider.cancelled)


def make_git_repo(path: Path) -> Path:
    path.mkdir(parents=True)
    (path / "src").mkdir()
    (path / "src" / "lib.rs").write_text("pub fn original() {}\n", encoding="utf-8")
    (path / "outside.txt").write_text("original outside\n", encoding="utf-8")
    subprocess.run(["git", "init"], cwd=path, check=True, capture_output=True, text=True)
    subprocess.run(["git", "config", "user.email", "conductor-test@example.invalid"], cwd=path, check=True)
    subprocess.run(["git", "config", "user.name", "Conductor Test"], cwd=path, check=True)
    subprocess.run(["git", "add", "."], cwd=path, check=True)
    subprocess.run(["git", "commit", "-m", "initial"], cwd=path, check=True, capture_output=True, text=True)
    return path


def scoped_task(task_id: str):
    return replace(
        sample_task(task_id),
        allowed_write_set=["src/lib.rs"],
        context_refs=["src/lib.rs"],
        production_hook="src/lib.rs::changed",
    )


if __name__ == "__main__":
    unittest.main()
