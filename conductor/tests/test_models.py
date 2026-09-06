from __future__ import annotations

import unittest
from dataclasses import replace
from pathlib import Path

from models import TaskResult, TaskSpec, stable_idem_key


def sample_task(task_id: str = "T-1-EXECUTE-1") -> TaskSpec:
    return TaskSpec(
        task_id=task_id,
        plan_id="T-1",
        wave="IMPLEMENT",
        role="EXECUTE",
        objective="Exercise the typed TaskSpec contract.",
        base_commit="HEAD",
        allowed_write_set=["crates/core/src/lib.rs"],
        context_refs=["plan.md"],
        production_hook="crates/core/src/lib.rs::hook",
        acceptance="Acceptance text",
        deterministic_check="echo deterministic",
        forbidden=["no commit", "no push"],
        verifier=[{"name": "unit", "command": ["true"]}],
        lease_ttl_seconds=60,
        idem_key=stable_idem_key([task_id, "HEAD"]),
        token_budget=100,
    )


class ModelTests(unittest.TestCase):
    def test_taskspec_prompt_contains_no_done_certification(self) -> None:
        text = sample_task().to_prompt_text()
        self.assertIn("Only the off-agent verifier", text)
        self.assertIn('"task_id": "T-1-EXECUTE-1"', text)

    def test_taskspec_prompt_contains_constraint_block(self) -> None:
        # M9 (teikoku-merge-addon-M8-M9-2026-07-05.md): the imperative CONSTRAINT BLOCK
        # (allowed files / production_hook reachability / DARK-is-a-BLOCK) must appear in
        # taskspec.txt (to_prompt_text), the no-char-budget dispatch surface.
        text = sample_task().to_prompt_text()
        self.assertIn("CONSTRAINT BLOCK", text)
        self.assertIn("ALLOWED FILES: crates/core/src/lib.rs", text)
        self.assertIn("crates/core/src/lib.rs::hook", text)
        self.assertIn("production_caller_check and python_wire_check", text)
        self.assertIn("DARK", text)
        self.assertIn("BLOCK", text)

    def test_constraint_lines_omit_production_hook_sentence_when_absent(self) -> None:
        task = replace(sample_task(), production_hook=None)
        lines = task.constraint_lines()
        self.assertFalse(any("MUST be reachable" in line for line in lines))
        self.assertTrue(any("ALLOWED FILES" in line for line in lines))

    def test_constraint_lines_omit_allowed_files_sentence_when_no_write_set(self) -> None:
        task = replace(sample_task(), allowed_write_set=[])
        lines = task.constraint_lines()
        self.assertFalse(any("ALLOWED FILES" in line for line in lines))

    def test_taskresult_rejects_done_hint(self) -> None:
        task = sample_task()
        with self.assertRaises(ValueError):
            TaskResult(
                task_id=task.task_id,
                base_commit=task.base_commit,
                worktree_path=str(Path("/tmp/wt")),
                branch_name="conductor/T-1",
                model_used="swe-1.6",
                session_id=None,
                exit_code=0,
                atif_artifact=None,
                diff_artifact=None,
                touched_files=[],
                worker_status_hint="DONE",
                error_category=None,
                rate_limited=False,
                disconnected_after_mutation=False,
                sanitized_notes="bad",
            )

    def test_dry_run_result_uses_no_change_hint(self) -> None:
        task = sample_task()
        result = TaskResult.dry_run(task, Path("/tmp/wt"), "swe-1.6")
        self.assertEqual(result.worker_status_hint, "NO_CHANGE")
        self.assertNotEqual(result.worker_status_hint, "DONE")


if __name__ == "__main__":
    unittest.main()
