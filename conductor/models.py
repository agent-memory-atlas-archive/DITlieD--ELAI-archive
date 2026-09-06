from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


ROLES = {"PLAN", "EXECUTE", "WIRE", "INTEGRATE", "AUDIT", "POLISH", "FIX"}
MODEL_TIERS = {"FREE", "PAID"}
TASK_STATUSES = {
    "PLANNED",
    "DISCOVERED",  # program_o signal intake / work-unit spine
    "ACCEPTED",
    "READY",
    "LEASED",
    "RUNNING",
    "VERIFYING",
    "VERIFIED",
    "INTEGRATING",
    "DONE",
    "NEEDS_REVIEW",
    "BLOCKED",
    "FAILED",
    "CANCELLED",
    "SUPERSEDED",
    # Opus-land park when preland verifier seat is unavailable (A-EV-QUEUE-BLOCK).
    # Not terminal: resume re-runs land when the verifier returns.
    "WAITING_FOR_VERIFIER",
}
WORKER_STATUS_HINTS = {
    "APPLIED",
    "NO_CHANGE",
    "BLOCKED",
    "FAILED",
    "RATE_LIMITED",
    "NEEDS_REVIEW",
    "TRANSIENT_PROVIDER_ERROR",
}
TERMINAL_STATUSES = {"DONE", "NEEDS_REVIEW", "BLOCKED", "FAILED", "CANCELLED"}

# Architecture §4.4 / Program O work-unit TaskSpec schema. V1 = pre-organism walk.
# V2 = typed work-unit fields used by program_o signal intake / idea merge / soak.
TASKSPEC_SCHEMA_V1 = 1
TASKSPEC_SCHEMA_V2 = 2


def stable_idem_key(parts: list[str]) -> str:
    h = hashlib.sha256()
    for part in parts:
        h.update(part.encode("utf-8"))
        h.update(b"\0")
    return h.hexdigest()[:32]


def json_dumps(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def parse_json_list(value: str | list[str] | None) -> list[str]:
    if value is None:
        return []
    if isinstance(value, list):
        return [str(item) for item in value]
    loaded = json.loads(value)
    if not isinstance(loaded, list):
        raise ValueError("expected JSON list")
    return [str(item) for item in loaded]


@dataclass(frozen=True)
class TaskSpec:
    task_id: str
    plan_id: str
    wave: str
    role: str
    objective: str
    base_commit: str
    allowed_write_set: list[str]
    context_refs: list[str]
    production_hook: str | None
    acceptance: str
    deterministic_check: str | None
    forbidden: list[str]
    verifier: list[dict[str, Any]]
    lease_ttl_seconds: int
    idem_key: str
    token_budget: int
    model_tier: str = "FREE"
    priority: int = 0
    max_attempts: int = 3
    # TEIKOKU-IR B1-fix (concrete-model enforcement): the specific model the RouteDecider chose
    # (pinned so the worker RUNS it, not the seeded slot's model) and the estimated context this
    # task needs (so acquire_ready refuses a too-small slot). BOTH nullable/None: a task with
    # neither behaves EXACTLY as before (tier-only lease + seeded slot model) -- only decide()-
    # pinned tasks get the stricter routing (backward-compatible; the walk/self-test infra is
    # untouched because those tasks are never decide()-pinned).
    pinned_model: str | None = None
    required_context_window: int | None = None
    schema_version: int = TASKSPEC_SCHEMA_V1
    # §4.4 self-containment fields (required when schema_version == V2)
    expected_observation: str = ""
    on_fail: str = ""
    abort_conditions: list[str] = field(default_factory=list)
    composes_with: list[Any] = field(default_factory=list)
    supersedes: list[Any] = field(default_factory=list)
    superseded_by: list[Any] = field(default_factory=list)
    supersession_reason: str | None = None

    def __post_init__(self) -> None:
        if self.role not in ROLES:
            raise ValueError(f"invalid role: {self.role}")
        if self.model_tier not in MODEL_TIERS:
            raise ValueError(f"invalid model_tier: {self.model_tier}")
        if not self.task_id:
            raise ValueError("task_id is required")
        if not self.plan_id:
            raise ValueError("plan_id is required")
        if not self.objective.strip():
            raise ValueError("objective is required")
        if self.lease_ttl_seconds <= 0:
            raise ValueError("lease_ttl_seconds must be positive")
        if self.token_budget < 0:
            raise ValueError("token_budget cannot be negative")
        if self.max_attempts <= 0:
            raise ValueError("max_attempts must be positive")
        if self.schema_version not in (TASKSPEC_SCHEMA_V1, TASKSPEC_SCHEMA_V2):
            raise ValueError(f"invalid schema_version: {self.schema_version}")
        if self.schema_version == TASKSPEC_SCHEMA_V2:
            if not str(self.expected_observation or "").strip():
                raise ValueError("expected_observation is required for schema_version V2")
            if not str(self.on_fail or "").strip():
                raise ValueError("on_fail is required for schema_version V2")
            if self.abort_conditions is None:
                raise ValueError("abort_conditions is required for schema_version V2")
            if self.composes_with is None:
                raise ValueError("composes_with is required for schema_version V2")
            if self.supersedes is None:
                raise ValueError("supersedes is required for schema_version V2")

    def to_json_dict(self) -> dict[str, Any]:
        out = {
            "schema_version": self.schema_version,
            "task_id": self.task_id,
            "plan_id": self.plan_id,
            "wave": self.wave,
            "role": self.role,
            "objective": self.objective,
            "base_commit": self.base_commit,
            "allowed_write_set": list(self.allowed_write_set),
            "context_refs": list(self.context_refs),
            "production_hook": self.production_hook,
            "acceptance": self.acceptance,
            "deterministic_check": self.deterministic_check,
            "forbidden": list(self.forbidden),
            "verifier": self.verifier,
            "lease_ttl_seconds": self.lease_ttl_seconds,
            "idem_key": self.idem_key,
            "token_budget": self.token_budget,
            "model_tier": self.model_tier,
            "priority": self.priority,
            "max_attempts": self.max_attempts,
            "pinned_model": self.pinned_model,
            "required_context_window": self.required_context_window,
            "expected_observation": self.expected_observation,
            "on_fail": self.on_fail,
            "abort_conditions": list(self.abort_conditions),
            "composes_with": list(self.composes_with),
            "supersedes": list(self.supersedes),
            "superseded_by": list(self.superseded_by),
            "supersession_reason": self.supersession_reason,
        }
        return out

    def constraint_lines(self) -> list[str]:
        """M9 (teikoku-merge-addon-M8-M9-2026-07-05.md, subsumes M3): the imperative
        CONSTRAINT BLOCK over the SAME fields build_task already populates from the plan's
        WIRING_CONTRACT (production_hook/acceptance/allowed_write_set). Shared by BOTH
        dispatch surfaces -- this method's own to_prompt_text (taskspec.txt, no char budget)
        and context_builder.py's _constraint_block_text (context.md) -- so the worker sees
        the same directive text in both artifacts. M89 correction: in context.md the block is
        no longer embedded in the first ("Task") section; it is rendered separately and
        appended AFTER _cap_sections/redact_text run (reserved out of the 12000-char budget
        up front) so a long allowed_write_set path is never mistaken for a secret by
        redaction.py's LONG_TOKEN_RE -- at the cost of no longer being the physically-first
        content in context.md. production_hook is a REACHABILITY-SCENARIO prose string on
        this corpus (M89-scout.md corpus sample, n=3/3), not a file+fn seam, so this is
        worded as a reachability requirement, never "wire into X" (which reads grammatically
        wrong against prose)."""
        lines = ["CONSTRAINT BLOCK (read before writing any code):"]
        if self.allowed_write_set:
            lines.append(
                "ALLOWED FILES: " + ", ".join(sorted(self.allowed_write_set))
                + ". Write ONLY within these."
            )
        if self.production_hook:
            lines.append(
                "Your new public symbol MUST be reachable in production per: "
                + self.production_hook
                + " -- and wired in THIS changeset, not a later one."
            )
        lines.append(
            "The audit runs production_caller_check and python_wire_check plus this "
            "task's Acceptance; a DARK (unreferenced) new public symbol is a BLOCK."
        )
        return lines

    def to_prompt_text(self) -> str:
        payload = self.to_json_dict()
        lines = [
            "You are executing a bounded ELAI Conductor TaskSpec.",
            "The worker may propose edits, notes, and blockers, but must not claim DONE.",
            "Only the off-agent verifier and serialized integrator can certify DONE.",
            "",
            *self.constraint_lines(),
            "",
            json.dumps(payload, indent=2, sort_keys=True),
        ]
        return "\n".join(lines)

    def db_values(self, now: str, status: str) -> dict[str, Any]:
        if status not in TASK_STATUSES:
            raise ValueError(f"invalid status: {status}")
        return {
            "task_id": self.task_id,
            "plan_id": self.plan_id,
            "wave": self.wave,
            "role": self.role,
            "objective": self.objective,
            "base_commit": self.base_commit,
            "allowed_write_set_json": json_dumps(self.allowed_write_set),
            "context_refs_json": json_dumps(self.context_refs),
            "production_hook": self.production_hook,
            "acceptance_text": self.acceptance,
            "deterministic_check_text": self.deterministic_check,
            "forbidden_text": "\n".join(self.forbidden),
            "verifier_json": json_dumps(self.verifier),
            "lease_ttl_seconds": self.lease_ttl_seconds,
            "idem_key": self.idem_key,
            "token_budget": self.token_budget,
            "model_tier": self.model_tier,
            "priority": self.priority,
            "status": status,
            "max_attempts": self.max_attempts,
            "pinned_model": self.pinned_model,
            "required_context_window": self.required_context_window,
            "created_at": now,
            "updated_at": now,
        }

    @classmethod
    def from_db_row(cls, row: Any) -> "TaskSpec":
        return cls(
            task_id=row["task_id"],
            plan_id=row["plan_id"],
            wave=row["wave"],
            role=row["role"],
            objective=row["objective"],
            base_commit=row["base_commit"],
            allowed_write_set=json.loads(row["allowed_write_set_json"]),
            context_refs=json.loads(row["context_refs_json"]),
            production_hook=row["production_hook"],
            acceptance=row["acceptance_text"],
            deterministic_check=row["deterministic_check_text"],
            forbidden=[line for line in row["forbidden_text"].splitlines() if line],
            verifier=json.loads(row["verifier_json"]),
            lease_ttl_seconds=int(row["lease_ttl_seconds"]),
            idem_key=row["idem_key"],
            token_budget=int(row["token_budget"]),
            model_tier=row["model_tier"],
            priority=int(row["priority"]),
            max_attempts=int(row["max_attempts"]),
            pinned_model=(row["pinned_model"] if "pinned_model" in row.keys() else None),
            required_context_window=(
                int(row["required_context_window"])
                if "required_context_window" in row.keys() and row["required_context_window"] is not None
                else None
            ),
        )


@dataclass(frozen=True)
class TaskResult:
    task_id: str
    base_commit: str
    worktree_path: str
    branch_name: str
    model_used: str
    session_id: str | None
    exit_code: int
    atif_artifact: str | None
    diff_artifact: str | None
    touched_files: list[str]
    worker_status_hint: str
    error_category: str | None
    rate_limited: bool
    disconnected_after_mutation: bool
    sanitized_notes: str
    schema_version: int = 1
    artifact_refs: list[str] = field(default_factory=list)

    def __post_init__(self) -> None:
        if self.worker_status_hint not in WORKER_STATUS_HINTS:
            raise ValueError(f"invalid worker_status_hint: {self.worker_status_hint}")
        if self.worker_status_hint == "DONE":
            raise ValueError("workers cannot self-certify DONE")

    def to_json_dict(self) -> dict[str, Any]:
        return {
            "schema_version": self.schema_version,
            "task_id": self.task_id,
            "base_commit": self.base_commit,
            "worktree_path": self.worktree_path,
            "branch_name": self.branch_name,
            "model_used": self.model_used,
            "session_id": self.session_id,
            "exit_code": self.exit_code,
            "atif_artifact": self.atif_artifact,
            "diff_artifact": self.diff_artifact,
            "touched_files": list(self.touched_files),
            "worker_status_hint": self.worker_status_hint,
            "error_category": self.error_category,
            "rate_limited": self.rate_limited,
            "disconnected_after_mutation": self.disconnected_after_mutation,
            "sanitized_notes": self.sanitized_notes,
            "artifact_refs": list(self.artifact_refs),
        }

    @classmethod
    def dry_run(
        cls,
        task: TaskSpec,
        worktree_path: Path,
        model_used: str,
        diff_artifact: Path | None = None,
        branch_name: str | None = None,
    ) -> "TaskResult":
        return cls(
            task_id=task.task_id,
            base_commit=task.base_commit,
            worktree_path=str(worktree_path),
            branch_name=branch_name or f"conductor/{task.task_id}",
            model_used=model_used,
            session_id=None,
            exit_code=0,
            atif_artifact=None,
            diff_artifact=str(diff_artifact) if diff_artifact else None,
            touched_files=[],
            worker_status_hint="NO_CHANGE",
            error_category=None,
            rate_limited=False,
            disconnected_after_mutation=False,
            sanitized_notes="Dry run rendered TaskSpec and Devin config only.",
            artifact_refs=[str(diff_artifact)] if diff_artifact else [],
        )
