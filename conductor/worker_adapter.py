#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import logging
import os
import re
import signal
import subprocess
import sys
import uuid
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any, Protocol

from active_workers import register_active_worker, unregister_active_worker
from conductor_config import normalize_path
from context_builder import build_context_packet_for_task, write_context_packet
from leases import LeaseManager
import free_api_worker
import worker_providers
import worker_telemetry  # A2-fix: relocated telemetry helpers (usage/watchdog/stream-artifact)
import codex_provider
import grok_provider  # 2026-08-07: grok-cli worker lane (posthog models via zenov)
import guard
from models import TaskResult, TaskSpec, parse_json_list, stable_idem_key
from redaction import redact_text
from scope_utils import normalize_repo_path, scope_violations
# Re-exported for existing importers (worker_telemetry, tests): WorkerSlot and the
# fallback pools moved to slot_pools.py (SLOT-FIX leg 2 module split).
from slot_pools import FREE_MODEL_POOL, PAID_MODEL_POOL, WorkerSlot, build_slot_pools


# P-BE1: the conductor lives in-distro at ~/teikoku; defaults resolve UNDER it so a spawned
# worker CWD walks up to ~/teikoku/.claude and loads the merged harness. An explicit Windows
# path in config is still honoured (normalize_path maps it to /mnt/<drive>); these are only
# the fallbacks used when no config / CLI override supplies a worktree.
_TEIKOKU_ROOT = Path.home() / "teikoku"
DEFAULT_WORKTREE_BASE = str(_TEIKOKU_ROOT / "conductor" / "worktrees")
DEFAULT_ARTIFACT_ROOT = str(_TEIKOKU_ROOT / "conductor" / "artifacts")
DEFAULT_REPO_ROOT = str(_TEIKOKU_ROOT / "projects" / "elai_cc")
DEFAULT_AGENT_ROOT = str(_TEIKOKU_ROOT / ".claude" / "agents")

# P-BE1 (interactive-orchestrator foundation): a spawned ORCHESTRATOR worker is a base-cc
# session whose CWD is inside ~/teikoku (so .claude discovery walks up to the merged harness).
# Its prompt is built here so the preamble-injection contract is owned by the worker plane:
#   line 1  the agent-dispatch-preamble pointer (binding on every spawned worker), and
#   then    the run directive -- /auto-orchestrate <objective> for an orchestration spawn, or
#           the plain objective for a leaf task.
# The pointer path is the IN-DISTRO merged-harness rule (not the F: source), so a worker that
# only sees the WSL tree can still resolve it.
PREAMBLE_POINTER_PATH = "/home/researcher/teikoku/.claude/rules/agent-dispatch-preamble.md"
PREAMBLE_POINTER_LINE = (
    f"Read {PREAMBLE_POINTER_PATH} FIRST and obey it as binding."
)
AUTO_ORCHESTRATE_DIRECTIVE = "/auto-orchestrate"

# TEIKOKU W2: every spawned worker is a HEADLESS/autonomous base-cc session, so its
# UserPromptSubmit/PostToolUse hooks (usage_stamp, clock_stamp agent-time) run on-by-default.
# The hooks read this env var to know the session is autonomous; the launcher (this seam) is
# the single place that exports it, so the on-by-default usage line fires on the J-plan walk.
AUTONOMOUS_ENV_VAR = "TEIKOKU_AUTONOMOUS"

# A sentinel "no wall-clock kill" timeout for the INTERACTIVE plane (U7 IMP-7): the 30-min
# subprocess kill is AUTONOMOUS-only; an interactive dispatch keeps the human pace. ~1 year.
_NO_WALL_CLOCK_TIMEOUT_SECONDS = 365 * 24 * 3600


def preamble_prefixed_objective(objective: str, *, orchestrate: bool) -> str:
    """Compose a spawned-worker objective per the P-BE1 frozen contract.

    Every spawned worker gets the agent-dispatch-preamble pointer as line 1 (binding). When
    orchestrate is True the worker is built as an ORCHESTRATOR: line 2 is the owner's
    /auto-orchestrate skill directive over the resolved objective, so the spawned base-cc
    session runs the orchestration skill. When False it is a plain leaf task: the objective
    follows the pointer unchanged. GENERAL: objective is pure data (any project), never an
    ELAI literal -- an empty objective stays empty (the caller validates non-emptiness).
    """
    resolved = (objective or "").strip()
    if orchestrate:
        directive = f"{AUTO_ORCHESTRATE_DIRECTIVE} {resolved}".rstrip()
        return PREAMBLE_POINTER_LINE + chr(10) + directive
    return PREAMBLE_POINTER_LINE + chr(10) + resolved


def _grok_guard_projection(text: str) -> str:
    fragments: list[str] = []

    def add(value: Any) -> None:
        if value is None:
            return
        if isinstance(value, str):
            if value:
                fragments.append(value)
            return
        try:
            fragments.append(json.dumps(value, ensure_ascii=False, sort_keys=True))
        except (TypeError, ValueError):
            fragments.append(str(value))

    def add_tool_result(value: Any) -> None:
        metadata_keys = {
            "toolCallId", "tool_call_id", "toolName", "tool_name", "status", "success",
            "isError", "is_error", "interrupted", "durationMs", "duration_ms", "timestamp",
            "sessionId", "session_id", "requestId", "request_id", "responseId", "response_id",
            "type", "subtype", "usage", "modelUsage", "model_usage", "cost", "cost_usd",
            "total_cost_usd", "num_turns", "stopReason", "stop_reason", "model", "version",
            "api", "provider", "permission_denials", "uuid", "parentUuid",
            "parent_uuid", "cwd", "details", "metadata", "metrics", "telemetry", "lifecycle",
            "thinking", "reasoning", "reasoning_content", "thinkingSignature", "signature",
        }
        if isinstance(value, str):
            add(value)
        elif isinstance(value, list):
            for item in value:
                add_tool_result(item)
        elif isinstance(value, dict):
            for key, item in value.items():
                if key not in metadata_keys:
                    add_tool_result(item)

    def add_content(content: Any, *, tool_only: bool = False) -> None:
        if isinstance(content, str):
            if not tool_only:
                add(content)
            return
        if isinstance(content, dict):
            if content.get("type") == "tool_result":
                add_tool_result(content.get("content"))
            elif not tool_only and content.get("type") not in {"thinking", "redacted_thinking"}:
                if content.get("type") == "text":
                    add(content.get("text"))
                elif content.get("type") in {"tool_use", "server_tool_use", "toolCall", "toolUse"}:
                    add(content.get("name", content.get("tool_name")))
                    add(content.get("input", content.get("arguments")))
            return
        if not isinstance(content, list):
            return
        for block in content:
            if not isinstance(block, dict):
                continue
            block_type = block.get("type")
            if block_type in {"thinking", "redacted_thinking"} or (
                tool_only and block_type != "tool_result"
            ):
                continue
            if block_type == "text":
                add(block.get("text"))
            elif block_type in {"tool_use", "server_tool_use", "toolCall", "toolUse"}:
                add(block.get("name", block.get("tool_name")))
                add(block.get("input", block.get("arguments")))
            elif block_type in {"tool_result", "web_search_tool_result"}:
                add_tool_result(block.get("content", block))

    def add_message(message: Any) -> None:
        if not isinstance(message, dict):
            return
        role = message.get("role")
        if role == "assistant":
            add_content(message.get("content"))
        elif role in {"tool", "tool_result"}:
            add_content(message.get("content"))
        elif role == "user":
            add_content(message.get("content"), tool_only=True)

    for line in (text or "").splitlines():
        if not line.strip():
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            fragments.append(line)
            continue
        if not isinstance(event, dict):
            continue
        event_type = event.get("type")
        if event_type in {"message_start", "message_end", "assistant", "user"}:
            add_message(event.get("message") or event)
        elif event_type == "message_update":
            update = event.get("assistantMessageEvent")
            if isinstance(update, dict):
                add_message(update.get("partial"))
                add_message(update.get("message"))
        elif event_type == "agent_end":
            for message in event.get("messages", []):
                add_message(message)
        elif event_type in {
            "tool_execution_start", "tool_execution_update", "tool_execution_end", "tool_result",
        }:
            for key in (
                "toolName", "tool_name", "args", "input", "arguments", "rawInput", "raw_input",
                "partialResult", "result", "output", "rawOutput", "content",
            ):
                if key in event:
                    if key in {"toolName", "tool_name", "args", "input", "arguments", "rawInput", "raw_input"}:
                        add(event[key])
                    else:
                        add_tool_result(event[key])
        elif event_type == "text":
            add(event.get("data"))
        elif event_type == "result":
            add_tool_result(event.get("result"))
            add_tool_result(event.get("structured_output"))
        elif event_type == "content_block_delta":
            delta = event.get("delta")
            if isinstance(delta, dict):
                if delta.get("type") == "text_delta":
                    add(delta.get("text"))
                elif delta.get("type") == "input_json_delta":
                    add(delta.get("partial_json"))
    return "\n".join(fragments)


@dataclass(frozen=True)
class ProviderResult:
    exit_code: int
    stdout: str
    stderr: str
    timed_out: bool = False
    cancelled: bool = False


class WorkerProvider(Protocol):
    def run(
        self,
        command: list[str],
        cwd: Path,
        timeout_seconds: int,
        env: dict[str, str],
        input_text: str | None = None,
    ) -> ProviderResult:
        ...

    def cancel(self) -> None:
        ...


class SubprocessWorkerProvider:
    def __init__(self) -> None:
        self._process: subprocess.Popen[str] | None = None
        self.on_start = None

    def run(
        self,
        command: list[str],
        cwd: Path,
        timeout_seconds: int,
        env: dict[str, str],
        input_text: str | None = None,
    ) -> ProviderResult:
        creationflags = 0
        popen_kwargs: dict[str, Any] = {}
        if os.name == "nt":
            creationflags = getattr(subprocess, "CREATE_NEW_PROCESS_GROUP", 0)
        else:
            popen_kwargs["start_new_session"] = True
        try:
            self._process = subprocess.Popen(
                command,
                cwd=str(cwd),
                stdin=subprocess.PIPE if input_text is not None else None,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                env=env,
                creationflags=creationflags,
                **popen_kwargs,
            )
            if self.on_start is not None:
                self.on_start(self._process.pid)
            stdout, stderr = self._process.communicate(
                input=input_text, timeout=timeout_seconds
            )
            return ProviderResult(self._process.returncode, stdout or "", stderr or "")
        except subprocess.TimeoutExpired as timeout_error:
            self.cancel()

            def output_to_text(output: str | bytes | None) -> str:
                if isinstance(output, bytes):
                    return output.decode("utf-8", errors="replace")
                return output or ""

            stdout = output_to_text(timeout_error.stdout)
            stderr = output_to_text(timeout_error.stderr)
            if self._process:
                try:
                    cleanup_stdout, cleanup_stderr = self._process.communicate(timeout=15)
                except subprocess.TimeoutExpired as cleanup_error:
                    cleanup_stdout = cleanup_error.stdout
                    cleanup_stderr = cleanup_error.stderr
                stdout += output_to_text(cleanup_stdout)
                stderr += output_to_text(cleanup_stderr)
            return ProviderResult(-1, stdout, stderr, timed_out=True)
        finally:
            self._process = None

    def cancel(self) -> None:
        process = self._process
        if process is None or process.poll() is not None:
            return
        try:
            if os.name == "nt":
                process.send_signal(signal.CTRL_BREAK_EVENT)  # type: ignore[attr-defined]
            else:
                os.killpg(process.pid, signal.SIGTERM)
        except Exception:
            process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()


class ProcessHandle:
    def __init__(self, provider: WorkerProvider) -> None:
        self._provider = provider
        self.cancelled = False

    def cancel(self) -> None:
        self.cancelled = True
        self._provider.cancel()


def dispatch_task(taskspec: dict[str, Any] | TaskSpec, dry_run: bool = False) -> dict[str, Any]:
    adapter = WorkerAdapter()
    return adapter.dispatch(taskspec, dry_run=dry_run).to_json_dict()


class WorkerAdapter:
    def __init__(
        self,
        config: dict[str, Any] | None = None,
        provider: WorkerProvider | None = None,
    ):
        data = config or {}
        self.worktree_base = normalize_path(data.get("worktree_base", data.get("worktree_root", DEFAULT_WORKTREE_BASE)))
        self.artifact_root = normalize_path(data.get("artifact_root", DEFAULT_ARTIFACT_ROOT))
        self.repo_root = normalize_path(data.get("repo_root", DEFAULT_REPO_ROOT))
        self.agent_root = normalize_path(data.get("agent_root", DEFAULT_AGENT_ROOT))
        self.subprocess_timeout = int(data.get("subprocess_timeout", data.get("subprocess_timeout_seconds", 1800)))
        self.provider = provider or SubprocessWorkerProvider()
        self.process_handle = ProcessHandle(self.provider)
        # P4-B: harvested safety guard (guard.py, donor codex conductor). Scans the
        # prompt sent to the worker (pre) + the output it returns (post) across ALL lanes.
        self._prompt_guard = guard.PromptGuard()
        self._output_guard = guard.OutputGuard()
        # SLOT-FIX leg 2 (paid-leg-entry.md, DECISION a): pools partition by each slot's
        # OWN model_tier, never by TOML section; see slot_pools.build_slot_pools.
        self.free_slots, self.paid_slots = build_slot_pools(data)
        self._slot_indexes: dict[str, int] = {"FREE": 0, "PAID": 0}

    @classmethod
    def from_conductor_config(
        cls,
        config: Any,
        provider: WorkerProvider | None = None,
    ) -> "WorkerAdapter":
        return cls(
            {
                "worktree_root": str(config.worktree_root),
                "artifact_root": str(config.artifact_root),
                "repo_root": str(config.repo_root),
                "agent_root": str(config.workflow_root / ".claude" / "agents"),
                "subprocess_timeout_seconds": config.limits.subprocess_timeout_seconds,
                "workers": list(config.workers),
                "escalation_workers": list(config.escalation_workers),
            },
            provider=provider,
        )

    def dispatch(
        self,
        taskspec: dict[str, Any] | TaskSpec,
        dry_run: bool = False,
        slot_id: str | None = None,
        worktree_path: str | Path | None = None,
        branch_name: str | None = None,
        lease_id: str | None = None,
        db_path: str | Path | None = None,
        wall_clock_kill: bool = True,
    ) -> TaskResult:
        task = coerce_taskspec(taskspec)
        slot = self.select_slot(task.model_tier, slot_id=slot_id)
        # TEIKOKU-IR B1-fix (concrete-model enforcement): when the task carries a pinned model (the
        # one the RouteDecider chose) and this slot's LANE can serve it, the worker RUNS the pinned
        # model, not the seeded slot model -- base-cc --model + the context-cap env below both read
        # slot.model, so overriding it here makes the effective model == the pinned model. Only a
        # same-lane override is safe (the proxy/env is the slot's); acquire_ready already guaranteed
        # the leased slot serves the pin, and this guard is the belt-and-braces for any other caller.
        pinned = getattr(task, "pinned_model", None)
        if pinned:
            import model_pool
            compatible = [
                candidate
                for candidate in (self.free_slots if task.model_tier.upper() == "FREE" else self.paid_slots)
                if model_pool.lane_serves_model(candidate.harness, pinned)
                or (pinned == "gpt-5-6-sol" and candidate.harness == "codex")
            ]
            if compatible and not (
                model_pool.lane_serves_model(slot.harness, pinned)
                or (pinned == "gpt-5-6-sol" and slot.harness == "codex")
            ):
                slot = compatible[0]
            if pinned != slot.model and model_pool.lane_serves_model(slot.harness, pinned):
                slot = replace(slot, model=pinned)
        if slot.harness == "prime-rlm" and pinned:
            slot = replace(slot, model=pinned)
        if slot.harness == grok_provider.GROK_HARNESS:
            slot = replace(slot, model=grok_provider.GROK_WORKER_MODEL)
            if getattr(task, "pinned_model", None) != grok_provider.GROK_WORKER_MODEL:
                task = replace(task, pinned_model=grok_provider.GROK_WORKER_MODEL)
        worktree_path, branch_name = self._prepare_worktree(
            task,
            dry_run=dry_run,
            worktree_path=worktree_path,
            branch_name=branch_name,
        )
        task_dir = worktree_path / ".task"
        task_dir.mkdir(parents=True, exist_ok=True)
        artifact_dir = self.artifact_root / task.task_id
        artifact_dir.mkdir(parents=True, exist_ok=True)

        self.render_taskspec(task, task_dir)
        self.render_context_packet(task, task_dir, worktree_path)
        role_config = self.render_role_config(task, task_dir)
        diff_artifact = artifact_dir / "patch.diff"

        if dry_run:
            diff_artifact.write_text("", encoding="utf-8")
            return TaskResult.dry_run(task, worktree_path, slot.model, diff_artifact, branch_name=branch_name)

        atif_path = task_dir / "result.atif.json"
        taskspec_file = task_dir / "taskspec.txt"
        command, input_text = self._build_worker_command(slot, task, taskspec_file, role_config, atif_path, worktree_path)
        env = self._worker_env(slot)
        # P4-B PRE-GUARD (all lanes): scan the prompt for injection BEFORE the provider runs.
        guard_text = input_text or taskspec_file.read_text(encoding="utf-8", errors="replace")
        blocked = worker_providers.pre_guard_block(
            self._prompt_guard, guard_text, task, worktree_path, slot.model,
            diff_artifact, branch_name, redact_text,
        )
        if blocked is not None:
            return blocked
        self._attach_pid_recorder(db_path, lease_id)
        if db_path is not None:
            register_active_worker(db_path, task.task_id, lease_id, self.cancel)
        # W-U4 Plane-2 (U7 IMP-7): the 30-min wall-clock kill is AUTONOMOUS-only. Every worker
        # dispatch is autonomous (wall_clock_kill defaults True); an interactive dispatch passes
        # wall_clock_kill=False and keeps the human pace (no forced subprocess kill).
        run_timeout = self.subprocess_timeout if wall_clock_kill else _NO_WALL_CLOCK_TIMEOUT_SECONDS
        try:
            provider_result = self.provider.run(command, worktree_path, run_timeout, env, input_text)
        finally:
            if hasattr(self.provider, "on_start"):
                setattr(self.provider, "on_start", None)
            if db_path is not None:
                unregister_active_worker(db_path, task.task_id)
        # W5 CRASH VISIBILITY (glm-orchestrator steal): persist the raw worker streams
        # as artifacts. sanitized_notes truncates to 4000 chars, so a crash traceback at
        # the end of a long stream is otherwise lost (the glm silent-crash-loop class:
        # a hidden window swallowed tracebacks and the loop looked idle, not crashing).
        stdout_artifact = artifact_dir / "worker_stdout.log"
        stderr_artifact = artifact_dir / "worker_stderr.log"
        worker_telemetry.write_stream_artifact(stdout_artifact, provider_result.stdout)
        worker_telemetry.write_stream_artifact(stderr_artifact, provider_result.stderr)
        # W-U3: capture THIS autonomous worker turn's per-lane token/cost into the W8 ledger
        # (provider_usage rows). Only the interactive /chat/turn seam recorded before; this wires
        # the autonomous walk so its lanes populate provider_costs / GET /orchestration. FAIL-SOFT.
        worker_telemetry.record_worker_usage(db_path, slot, provider_result.stdout)
        # W-U4 Plane-2: when the autonomous wall-clock watchdog FIRES (subprocess timeout), surface
        # it as a structured observation event instead of a silent reclaim (partial output kept).
        if provider_result.timed_out and wall_clock_kill:
            worker_telemetry.record_watchdog_fire(
                db_path, task.task_id, slot, self.subprocess_timeout,
                stdout_artifact, stderr_artifact,
                provider_result.stdout, provider_result.stderr,
            )
        # Capture the diff + touched set ONCE (used by the post-guard and the result below).
        touched_files = self._git_touched_files(worktree_path)
        self._write_diff(worktree_path, diff_artifact)
        # P4-B POST-GUARD (all lanes): scan the worker output for dangerous / exfil patterns.
        guard_stdout = (
            _grok_guard_projection(provider_result.stdout)
            if slot.harness == grok_provider.GROK_HARNESS
            else provider_result.stdout
        )
        blocked = worker_providers.post_guard_block(
            self._output_guard, guard_stdout, provider_result.stderr,
            task, worktree_path, slot.model, diff_artifact, branch_name, touched_files, redact_text,
        )
        if blocked is not None:
            return blocked
        notes = (
            self._extract_worker_notes(slot.harness, atif_path, provider_result.stdout, provider_result.stderr)
            or provider_result.stdout
            or provider_result.stderr
            or ""
        )
        rate_limited = classify_rate_limit(
            provider_result.stdout + "\n" + provider_result.stderr,
            provider_result.exit_code,
            provider_result.timed_out,
        )
        violations = scope_violations(touched_files, task.allowed_write_set)
        disconnected_after_mutation = bool(
            touched_files
            and (
                provider_result.timed_out
                or provider_result.cancelled
                or self.process_handle.cancelled
                or provider_result.exit_code < 0
            )
        )
        combined_output = provider_result.stdout + "\n" + provider_result.stderr
        error_category = classify_error_category(
            rate_limited=rate_limited,
            timed_out=provider_result.timed_out,
            cancelled=provider_result.cancelled or self.process_handle.cancelled,
            violations=violations,
            exit_code=provider_result.exit_code,
            combined_output=combined_output,
        )
        status_hint = self._status_hint(
            provider_result.exit_code,
            rate_limited,
            disconnected_after_mutation,
            touched_files,
            violations,
            error_category=error_category,
        )
        sanitized_notes = redact_text(notes)
        if violations:
            sanitized_notes = (
                sanitized_notes
                + "\n\nAllowed-write violation: "
                + ", ".join(violations)
            )
        return TaskResult(
            task_id=task.task_id,
            base_commit=task.base_commit,
            worktree_path=str(worktree_path),
            branch_name=branch_name,
            model_used=slot.model,
            session_id=self._parse_session_id(provider_result.stdout + "\n" + provider_result.stderr),
            exit_code=provider_result.exit_code,
            atif_artifact=str(atif_path) if atif_path.exists() else None,
            diff_artifact=str(diff_artifact),
            touched_files=touched_files,
            worker_status_hint=status_hint,
            error_category=error_category,
            rate_limited=rate_limited,
            disconnected_after_mutation=disconnected_after_mutation,
            sanitized_notes=_head_tail(sanitized_notes, 4000),
            artifact_refs=[
                str(path)
                for path in (atif_path, diff_artifact, stdout_artifact, stderr_artifact)
                if path.exists()
            ],
        )

    def cancel(self) -> None:
        self.process_handle.cancel()

    def select_slot(self, tier: str, slot_id: str | None = None) -> WorkerSlot:
        normalized_tier = tier.upper()
        pool = self.free_slots if normalized_tier == "FREE" else self.paid_slots
        # SLOT-FIX leg 2: with tier-partitioned pools a pool can now legitimately be EMPTY
        # (e.g. a PAID-only role-split config has zero FREE slots). Fail with the same
        # KeyError contract callers already handle instead of ZeroDivisionError on the
        # round-robin modulo below.
        if not pool:
            raise KeyError(f"no worker slots configured for tier {normalized_tier}")
        if slot_id:
            for slot in pool:
                if slot.slot_id == slot_id:
                    return slot
            raise KeyError(f"worker slot not found for {normalized_tier}: {slot_id}")
        index = self._slot_indexes[normalized_tier] % len(pool)
        self._slot_indexes[normalized_tier] += 1
        return pool[index]

    def select_model(self, tier: str) -> str:
        return self.select_slot(tier).model

    def _prepare_worktree(
        self,
        task: TaskSpec,
        dry_run: bool,
        worktree_path: str | Path | None = None,
        branch_name: str | None = None,
    ) -> tuple[Path, str]:
        safe_task_id = re.sub(r"[^A-Za-z0-9_.-]+", "_", task.task_id)
        logical_branch = branch_name or f"conductor/{safe_task_id}"
        if dry_run:
            path = normalize_path(worktree_path) if worktree_path is not None else self.worktree_base / f"dryrun_{safe_task_id}"
            path.mkdir(parents=True, exist_ok=True)
            return path, logical_branch

        self.worktree_base.mkdir(parents=True, exist_ok=True)
        path = normalize_path(worktree_path) if worktree_path is not None else self.worktree_base / f"{safe_task_id}_{uuid.uuid4().hex[:8]}"
        branch = branch_name or f"conductor/{safe_task_id}-{uuid.uuid4().hex[:8]}"
        # --reuse-worktree: path already exists as a git worktree (partial continuation).
        # Do not re-run `git worktree add` (fails with "already exists").
        if worktree_path is not None and path.exists() and (path / ".git").exists():
            br = subprocess.run(
                ["git", "-C", str(path), "rev-parse", "--abbrev-ref", "HEAD"],
                capture_output=True,
                text=True,
                timeout=30,
            )
            if br.returncode == 0 and br.stdout.strip():
                branch = br.stdout.strip()
            return path, branch
        path.parent.mkdir(parents=True, exist_ok=True)
        result = subprocess.run(
            [
                "git",
                "-C",
                str(self.repo_root),
                "worktree",
                "add",
                "-b",
                branch,
                str(path),
                task.base_commit,
            ],
            capture_output=True,
            text=True,
            timeout=120,
        )
        if result.returncode != 0:
            raise RuntimeError(redact_text(result.stderr or result.stdout))
        return path, branch

    def render_taskspec(self, task: TaskSpec, task_dir: Path) -> tuple[Path, Path]:
        json_path = task_dir / "taskspec.json"
        text_path = task_dir / "taskspec.txt"
        json_path.write_text(json.dumps(task.to_json_dict(), indent=2, sort_keys=True), encoding="utf-8")
        text_path.write_text(task.to_prompt_text(), encoding="utf-8")
        return json_path, text_path

    def render_context_packet(self, task: TaskSpec, task_dir: Path, worktree_path: Path) -> tuple[Path, Path]:
        packet = build_context_packet_for_task(task, worktree_path)
        return write_context_packet(packet, task_dir)

    def render_role_config(self, task: TaskSpec, task_dir: Path) -> Path:
        role_prompt = self._load_role_prompt(task.role)
        config_path = task_dir / "role.config.yaml"
        allow = []
        for path in sorted(task.allowed_write_set):
            normalized = normalize_repo_path(path)
            allow.append(f"Read({normalized})")
            allow.append(f"Write({normalized})")
        deny = [
            "Exec(git reset)",
            "Exec(git checkout)",
            "Exec(git clean)",
            "Exec(git push)",
            "Exec(git commit)",
            "Exec(* --lock *)",
            "Write(.claude/**)",
            "Write(**/.claude/**)",
            "Write(**/.claude/tools/**)",
            "Write(gate-manifest*)",
            "Write(**/gate-manifest*)",
            "Read(**/*credential*)",
            "Read(**/*secret*)",
            "Read(**/*.env)",
        ]
        lines = ["system_instructions: |"]
        for line in role_prompt.splitlines() or [""]:
            lines.append(f"  {line}")
        lines.extend(["permissions:"])
        if allow:
            lines.append("  allow:")
            lines.extend(f"    - {json.dumps(item)}" for item in allow)
        else:
            lines.append("  allow: []")
        lines.append("  deny:")
        lines.extend(f"    - {json.dumps(item)}" for item in deny)
        lines.extend(["  ask: []", "allowed_tools: []", "mcp_servers: {}", "extensions: {}"])
        config_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
        return config_path

    def _build_worker_command(
        self,
        slot: WorkerSlot,
        task: TaskSpec,
        taskspec_file: Path,
        role_config: Path,
        atif_file: Path,
        worktree_path: Path,
    ) -> tuple[list[str], str | None]:
        if slot.harness == "codex":
            return codex_provider.build_codex_command(slot, task, taskspec_file, worktree_path)
        if slot.harness == grok_provider.GROK_HARNESS:  # "grok-cli": official Grok Build CLI
            # 2026-08-07 fix: grok-cli slots (the posthog zenov lane) previously fell through to
            # the base-cc branch below, so the claude binary ran with NO base-url override and hit
            # Anthropic plan-OAuth -> every posthog dispatch failed 401 while the record claimed
            # the posthog model. grok_provider.build_worker_command was authored for exactly this
            # harness but was never wired here. Prompt rides argv (-p), not stdin.
            prompt = taskspec_file.read_text(encoding="utf-8", errors="replace")
            return grok_provider.build_worker_command(
                prompt,
                worktree_path,
                model=grok_provider.GROK_WORKER_MODEL,
                effort=grok_provider.GROK_WORKER_EFFORT,
                # Per-dispatch PRIVATE leader socket on argv (env alone is not enough for
                # nest2; without it the worker joins the shared leader and gets Cancelled).
                leader_socket=Path(worktree_path) / ".task" / "grok-leader.sock",
            )
        # P4-B unified worker plane: every non-codex slot is a base-cc backend
        # (claude OAuth / devin-free bridge / free-api proxy). They share the base-cc
        # command shape; only the env (worker_env) differs by backend. devin.exe is retired.
        # Free/devin workers run --bare to skip hooks that can block headless startup. The
        # plan-OAuth Claude lane must keep normal mode so it can use its OAuth credentials.
        bare = (
            worker_providers.is_devin_free_harness(slot.harness)
            or free_api_worker.is_free_api_harness(slot.harness)
        )
        return free_api_worker.build_free_api_command(slot.model, taskspec_file, bare=bare)

    def _extract_worker_notes(
        self,
        harness: str,
        atif_path: Path,
        stdout: str,
        stderr: str,
    ) -> str | None:
        if harness == "codex":
            return codex_provider.parse_codex_notes(stdout, stderr, atif_path)
        if harness == grok_provider.GROK_HARNESS:  # grok-cli streaming-json envelope
            return grok_provider.parse_worker_notes(stdout, stderr)
        if worker_providers.is_base_cc_backend(harness):  # P4-B: base-cc JSON envelope
            return free_api_worker.parse_cc_json_notes(stdout, stderr, atif_path)
        return self._parse_atif_notes(atif_path)

    def _load_role_prompt(self, role: str) -> str:
        candidates = [
            self.agent_root / f"{role}.md",
            self.agent_root / f"{role.lower()}.md",
            self.agent_root / "executor.md",
        ]
        for candidate in candidates:
            if candidate.exists():
                return redact_text(candidate.read_text(encoding="utf-8"))
        return (
            f"You are the {role} worker for the ELAI Conductor. "
            "Operate only within the TaskSpec write set. Never commit, push, "
            "refresh frozen gate locks, or read credentials. Do not claim DONE; "
            "the off-agent verifier is the only completion authority."
        )

    def _worker_env(self, slot: WorkerSlot) -> dict[str, str]:
        keep = {
            "PATH",
            "PATHEXT",
            "SYSTEMROOT",
            "WINDIR",
            "COMSPEC",
            "USERPROFILE",
            "HOMEDRIVE",
            "HOMEPATH",
            "APPDATA",
            "LOCALAPPDATA",
            "TEMP",
            "TMP",
            "HOME",
            "LANG",
            "LC_ALL",
            "SSL_CERT_FILE",
            "SSL_CERT_DIR",
            "REQUESTS_CA_BUNDLE",
            "CURL_CA_BUNDLE",
            # systemd-run --user (opus_dispatch._scope_wrap) needs the session bus;
            # headless orchestrator shells often omit these — inject below if absent.
            "XDG_RUNTIME_DIR",
            "DBUS_SESSION_BUS_ADDRESS",
            "USER",
            "LOGNAME",
            "UID",
        }
        # Ensure session bus vars exist before the whitelist copy (headless fix 2026-08-06).
        if not os.environ.get("XDG_RUNTIME_DIR"):
            runtime = f"/run/user/{os.getuid()}"
            if os.path.isdir(runtime):
                os.environ["XDG_RUNTIME_DIR"] = runtime
        if not os.environ.get("DBUS_SESSION_BUS_ADDRESS"):
            runtime = os.environ.get("XDG_RUNTIME_DIR") or f"/run/user/{os.getuid()}"
            bus_path = f"{runtime}/bus"
            if os.path.exists(bus_path):
                os.environ["DBUS_SESSION_BUS_ADDRESS"] = f"unix:path={bus_path}"
        env = {
            key: value
            for key, value in os.environ.items()
            if key.upper() in keep and not _looks_secret_env_key(key)
        }
        # W2: mark every spawned worker session autonomous so its usage_stamp / clock_stamp
        # agent-time hooks fire on-by-default. Every provider branch below carries `env`
        # forward (claude_env/devin_free_env/free_api_env do dict(base_env)), so this rides through.
        env[AUTONOMOUS_ENV_VAR] = "1"
        bun_tmpdir = _TEIKOKU_ROOT / "conductor" / "state" / "worker-bun-tmp"
        bun_tmpdir.mkdir(parents=True, exist_ok=True)
        env.setdefault("BUN_TMPDIR", str(bun_tmpdir))
        if (
            worker_providers.is_devin_free_harness(slot.harness)
            or free_api_worker.is_free_api_harness(slot.harness)
        ):
            # Free workers must not inherit owner project hooks from ~/.claude.
            worker_claude_cfg = _TEIKOKU_ROOT / "conductor" / "state" / "worker-claude-config"
            worker_claude_cfg.mkdir(parents=True, exist_ok=True)
            env["CLAUDE_CONFIG_DIR"] = str(worker_claude_cfg)
        if slot.harness == "codex":
            codex_home = slot.codex_home or f"/root/.codex-accounts/{slot.account_id or 'default'}"
            env["CODEX_HOME"] = codex_home
            if slot.account_id:
                env["CONDUCTOR_WORKER_ACCOUNT"] = slot.account_id
            return env
        if slot.harness == grok_provider.GROK_HARNESS:
            # The project fleet GROK_HOME is fixed for this stopped ELAI harness. It contains
            # the approved Luna-fast and Sol-fast catalog rows. Preserve explicit overrides for
            # isolated tests and owner-controlled callers.
            os.environ[grok_provider.FLEET_GROK_HOME_ENV] = str(
                grok_provider.PROJECT_FLEET_GROK_HOME
            )
            grok_env = grok_provider.build_grok_env(os.environ, fleet=True)
            grok_env["HOME"] = os.environ.get("HOME", str(Path.home()))
            grok_env["PATH"] = os.environ.get("PATH", grok_env.get("PATH", ""))
            # Per-dispatch PRIVATE leader socket: concurrent sessions on one leader cancel
            # headless turns (the grok_adapter lesson); two parallel posthog workers must
            # never share a leader.
            grok_env["GROK_LEADER_SOCKET"] = str(
                Path(grok_env["GROK_HOME"]) / f"worker-{uuid.uuid4().hex[:12]}.sock"
            )
            # systemd-run --user cage (_scope_wrap) needs the session bus; carry the vars the
            # keep-list injected above (build_grok_env strips them).
            for bus_key in ("XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS", "USER", "LOGNAME"):
                if env.get(bus_key):
                    grok_env[bus_key] = env[bus_key]
            grok_env[AUTONOMOUS_ENV_VAR] = "1"
            if slot.account_id:
                grok_env["CONDUCTOR_WORKER_ACCOUNT"] = slot.account_id
            return grok_env
        if worker_providers.is_claude_harness(slot.harness):  # P4-B claude OAuth lane (no base_url)
            env = worker_providers.claude_env(env)
            env = worker_providers.context_cap_env(env, slot.model, "claude")
            if slot.account_id:
                env["CONDUCTOR_WORKER_ACCOUNT"] = slot.account_id
            return env
        if worker_providers.is_devin_free_harness(slot.harness):  # P4-B devin bridge lane
            env = worker_providers.devin_free_env(env, bridge_url=slot.proxy_base_url)
            env = worker_providers.context_cap_env(env, slot.model, "devin-free")
            if slot.account_id:
                env["CONDUCTOR_WORKER_ACCOUNT"] = slot.account_id
            return env
        if free_api_worker.is_free_api_harness(slot.harness):  # P4-B free-api proxy env
            proxy_url = slot.proxy_base_url
            env = free_api_worker.free_api_env(env, proxy_base_url=proxy_url, account_id=slot.account_id)
            env = worker_providers.context_cap_env(env, slot.model, "free-api")
            if slot.account_id:
                env["CONDUCTOR_WORKER_ACCOUNT"] = slot.account_id
            return env
        if slot.account_id:
            env["CONDUCTOR_WORKER_ACCOUNT"] = slot.account_id
        return env

    def _attach_pid_recorder(self, db_path: str | Path | None, lease_id: str | None) -> None:
        if db_path is None or lease_id is None or not hasattr(self.provider, "on_start"):
            if hasattr(self.provider, "on_start"):
                setattr(self.provider, "on_start", None)
            return

        def record_pid(pid: int) -> None:
            LeaseManager(db_path).update_execution_refs(lease_id, worker_pid=pid)

        setattr(self.provider, "on_start", record_pid)

    def _git_touched_files(self, worktree_path: Path) -> list[str]:
        result = subprocess.run(
            ["git", "-C", str(worktree_path), "status", "--porcelain=v1", "--untracked-files=all"],
            capture_output=True,
            text=True,
            timeout=60,
        )
        if result.returncode != 0:
            return []
        return _status_paths(result.stdout)

    def _write_diff(self, worktree_path: Path, diff_artifact: Path) -> None:
        _make_untracked_visible_to_diff(worktree_path)
        result = subprocess.run(
            ["git", "-C", str(worktree_path), "diff", "--binary"],
            capture_output=True,
            text=True,
            timeout=60,
        )
        text = result.stdout if result.returncode == 0 else ""
        diff_artifact.write_text(redact_text(text), encoding="utf-8")

    def _parse_atif_notes(self, atif_path: Path) -> str | None:
        if not atif_path.exists():
            return None
        try:
            data = json.loads(atif_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            return None
        redacted = _redacted_atif(data)
        atif_path.write_text(json.dumps(redacted, indent=2, sort_keys=True), encoding="utf-8")
        for key in ("final_message", "message", "notes"):
            if isinstance(redacted.get(key), str):
                return redacted[key]
        steps = redacted.get("steps", [])
        if isinstance(steps, list):
            for step in reversed(steps):
                if isinstance(step, dict):
                    message = step.get("message") or step.get("content")
                    if isinstance(message, str):
                        return message
        return None

    def _parse_session_id(self, output: str) -> str | None:
        match = re.search(r"session[_ -]?id[:=]\s*([A-Za-z0-9_.:-]+)", output, re.IGNORECASE)
        return match.group(1) if match else None

    def _status_hint(
        self,
        exit_code: int,
        rate_limited: bool,
        disconnected_after_mutation: bool,
        touched_files: list[str],
        violations: list[str],
        error_category: str | None = None,
    ) -> str:
        if rate_limited:
            return "RATE_LIMITED"
        if error_category == "transient_provider_error" and not disconnected_after_mutation:
            return "TRANSIENT_PROVIDER_ERROR"
        if disconnected_after_mutation:
            return "NEEDS_REVIEW"
        if violations:
            return "BLOCKED"
        if exit_code != 0:
            return "FAILED"
        return "APPLIED" if touched_files else "NO_CHANGE"


def _redacted_atif(value: Any) -> Any:
    from redaction import redact_json

    return redact_json(value)


def classify_rate_limit(stderr: str, exit_code: int, timed_out: bool) -> bool:
    if timed_out:
        return False
    if exit_code == 0:
        return False
    lower = stderr.lower()
    return any(
        token in lower
        for token in (
            "429",
            "quota",
            "rate limit",
            "rate-limit",
            "throttle",
            "exhausted",
            "concurrency limit",
        )
    )


_TRANSIENT_PROVIDER_TOKENS = (
    "524",
    "error code: 524",
    "cf-ray:",
    "cloudflare",
    "stream disconnected",
    "idle timeout waiting for sse",
    "idle timeout",
    "reconnecting",
    "unexpected status 5",
    "bad gateway",
    "service unavailable",
    "gateway timeout",
    "connection reset",
    "connection refused",
    "econnreset",
    "econnrefused",
    "epipe",
    "socket hang up",
    "etimedout",
)


def classify_transient_provider_error(output: str, exit_code: int, timed_out: bool) -> bool:
    if exit_code == 0 and not timed_out:
        return False
    lower = output.lower()
    return any(token in lower for token in _TRANSIENT_PROVIDER_TOKENS)


def _looks_secret_env_key(key: str) -> bool:
    upper = key.upper()
    return any(token in upper for token in ("KEY", "TOKEN", "SECRET", "PASSWORD", "CREDENTIAL"))


def classify_error_category(
    *,
    rate_limited: bool,
    timed_out: bool,
    cancelled: bool,
    violations: list[str],
    exit_code: int,
    combined_output: str = "",
) -> str | None:
    if rate_limited:
        return "rate_limited"
    if classify_transient_provider_error(combined_output, exit_code, timed_out):
        return "transient_provider_error"
    # TEIKOKU-IR TR-1: a size-overflow rejection is NOT a transient error -- retrying the same
    # small window fails again. Classify it distinctly so the scheduler escalates to a bigger window.
    # The classifier lives in worker_providers (extracted for MR-1, keeping this file under 1000).
    if exit_code != 0 and worker_providers.classify_context_overflow(combined_output):
        return "context_window_blocked"
    if timed_out:
        return "timeout"
    if cancelled:
        return "cancelled"
    if violations:
        return "allowed_write_violation"
    if exit_code != 0:
        return "worker_failed"
    return None


def _wsl_path(path: Path) -> str:
    text = str(path.resolve())
    if len(text) >= 2 and text[1] == ":":
        drive = text[0].lower()
        rest = text[2:].replace("\\", "/")
        return f"/mnt/{drive}{rest}"
    return text.replace("\\", "/")


def _head_tail(text: str, limit: int) -> str:
    """Bound text to ~limit chars keeping BOTH ends. Head-only truncation ([:limit])
    amputated every recon VERDICT line (N-2/N-14/O-1, 2026-08-12): the typed verdict
    lives in the worker's FINAL message, at the tail of the notes."""
    if len(text) <= limit:
        return text
    half = max(1, (limit - 20) // 2)
    return text[:half] + "\n...[middle cut]...\n" + text[-half:]


def _make_untracked_visible_to_diff(worktree_path: Path) -> None:
    result = subprocess.run(
        ["git", "-C", str(worktree_path), "ls-files", "--others", "--exclude-standard", "-z"],
        capture_output=True,
        timeout=60,
    )
    if result.returncode != 0 or not result.stdout:
        return
    paths = [
        path.decode("utf-8", errors="replace")
        for path in result.stdout.split(b"\0")
        if path and not _is_conductor_metadata_path(path.decode("utf-8", errors="replace"))
    ]
    if paths:
        subprocess.run(
            ["git", "-C", str(worktree_path), "add", "-N", "--", *paths],
            capture_output=True,
            text=True,
            timeout=60,
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
                    paths.add(normalize_repo_path(path))
        else:
            path = payload.strip('"')
            if not _is_conductor_metadata_path(path):
                paths.add(normalize_repo_path(path))
    return sorted(path for path in paths if path)


def _is_conductor_metadata_path(path: str) -> bool:
    normalized = normalize_repo_path(path)
    return normalized == ".task" or normalized.startswith(".task/")


def coerce_taskspec(value: dict[str, Any] | TaskSpec) -> TaskSpec:
    if isinstance(value, TaskSpec):
        return value
    allowed_write_set = value.get("allowed_write_set", value.get("allowed_write_set_json", []))
    context_refs = value.get("context_refs", value.get("context_refs_json", []))
    forbidden = value.get("forbidden", [])
    if isinstance(forbidden, str):
        forbidden = [line for line in forbidden.splitlines() if line.strip()] or [forbidden]
    verifier = value.get("verifier", [])
    if isinstance(verifier, str):
        try:
            verifier = json.loads(verifier)
        except json.JSONDecodeError:
            verifier = [{"name": "legacy", "command_text": verifier}]
    task_id = str(value["task_id"])
    base_commit = str(value.get("base_commit", "HEAD"))
    idem_key = str(value.get("idem_key") or stable_idem_key([task_id, base_commit]))
    return TaskSpec(
        task_id=task_id,
        plan_id=str(value.get("plan_id", task_id)),
        wave=str(value.get("wave", "IMPLEMENT")),
        role=str(value.get("role", "EXECUTE")),
        objective=str(value.get("objective", "Dry-run worker validation task.")),
        base_commit=base_commit,
        allowed_write_set=parse_json_list(allowed_write_set),
        context_refs=parse_json_list(context_refs),
        production_hook=value.get("production_hook"),
        acceptance=str(value.get("acceptance", value.get("acceptance_text", "Dry-run acceptance."))),
        deterministic_check=value.get("deterministic_check", value.get("deterministic_check_text")),
        forbidden=[str(item) for item in forbidden],
        verifier=verifier,
        lease_ttl_seconds=int(value.get("lease_ttl_seconds", value.get("lease_ttl", 3600))),
        idem_key=idem_key,
        token_budget=int(value.get("token_budget", 100000)),
        model_tier=str(value.get("model_tier", "FREE")).upper(),
        priority=int(value.get("priority", 0)),
        max_attempts=int(value.get("max_attempts", 3)),
        pinned_model=(str(value["pinned_model"]) if value.get("pinned_model") else None),
        required_context_window=(
            int(value["required_context_window"]) if value.get("required_context_window") else None
        ),
    )


def dry_run_taskspec(task_id: str, model_tier: str = "FREE") -> TaskSpec:
    return TaskSpec(
        task_id=task_id,
        plan_id="DOCTOR",
        wave="IMPLEMENT",
        role="EXECUTE",
        objective="Validate TaskSpec rendering and worker command construction without launching a live worker.",
        base_commit="HEAD",
        allowed_write_set=["conductor/worker_adapter.py"],
        context_refs=["IMPLEMENTATION_PLAN.md"],
        production_hook="worker_adapter.py::dispatch_task",
        acceptance="Dry-run adapter command uses --config and does not return DONE.",
        deterministic_check="python3 worker_adapter.py --dry-run --task-id DOCTOR",
        forbidden=[
            "no gate edits",
            "no gate relock",
            "no destructive git",
            "no commit",
            "no push",
            "no credential reads",
        ],
        verifier=[{"name": "dry_run", "command": ["python3", "worker_adapter.py", "--dry-run"]}],
        lease_ttl_seconds=3600,
        idem_key=stable_idem_key([task_id, "HEAD", "dry-run"]),
        token_budget=0,
        model_tier=model_tier,
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Conductor worker adapter (base-cc / Codex)")
    parser.add_argument("--task-id", default="WORKER-DRYRUN")
    parser.add_argument("--taskspec")
    parser.add_argument("--model-tier", default="FREE", choices=["FREE", "PAID"])
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--worktree-base", default=DEFAULT_WORKTREE_BASE)
    parser.add_argument("--artifact-root", default=DEFAULT_ARTIFACT_ROOT)
    parser.add_argument("--repo-root", default=DEFAULT_REPO_ROOT)
    parser.add_argument("--json", action="store_true")
    return parser.parse_args()


def main() -> int:
    logging.basicConfig(level=logging.WARNING, format="%(levelname)s %(message)s")
    args = parse_args()
    if args.taskspec:
        task = coerce_taskspec(json.loads(Path(args.taskspec).read_text(encoding="utf-8")))
    else:
        task = dry_run_taskspec(args.task_id, args.model_tier)
    adapter = WorkerAdapter(
        {
            "worktree_base": args.worktree_base,
            "artifact_root": args.artifact_root,
            "repo_root": args.repo_root,
        }
    )
    if not args.dry_run:
        print("ERROR: real worker dispatch is disabled from the standalone CLI for now; use --dry-run", file=sys.stderr)
        return 2
    # Route through the public dispatch_task entrypoint so it has a real
    # production caller (the worker adapter's own CLI) per wiring gate R-8.
    result = dispatch_task(task, dry_run=True)
    print(json.dumps(result, indent=2, sort_keys=True) if args.json else json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
