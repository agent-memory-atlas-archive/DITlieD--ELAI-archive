#!/usr/bin/env python3
"""TEIKOKU P4-B codex paid lane (TEIKOKU-P4-REVISION-2026-06-20 D3).

The codex-exec / gpt-5.5 PAID worker provider, harvested as a sibling to the base-cc lanes.
codex-exec is a DIFFERENT CLI + protocol (OpenAI Responses API via the bmapi proxy), not
base-cc, so it is its own provider module. Two account profiles (account-a / account-b) are
distinguished by CODEX_HOME (each account's config.toml dir). Secrets (auth.json) live in the
codex-vm distro under root, 0600, NEVER copied into teikoku git -- this module references the
config path only, never a key value.

Extracted from worker_adapter.py to keep that file under the 1000-line MR-1 boundary (the
unified base-cc env logic lives in worker_providers.py; the codex provider logic lives here).

Stdlib only.
"""
from __future__ import annotations

import json
from pathlib import Path


def _wsl_path(path: Path) -> str:
    """A Windows path -> its /mnt/<drive> WSL form for `codex exec -C`. A POSIX path passes
    through unchanged (the teikoku distro worktrees are already POSIX)."""
    text = str(path)
    if len(text) >= 2 and text[1] == ":":
        drive = text[0].lower()
        rest = text[2:].replace("\\", "/")
        return f"/mnt/{drive}{rest}"
    return text.replace("\\", "/")


def build_codex_command(
    slot,
    task,
    taskspec_file: Path,
    worktree_path: Path,
) -> tuple[list[str], str]:
    """The codex-exec CLI invocation (WSL interop into the codex-vm distro) + the stdin prompt.
    Uses --ephemeral + per-account --config <codex_home>/config.toml (the account selector)."""
    codex_home = slot.codex_home or f"/root/.codex-accounts/{slot.account_id or 'default'}"
    config_path = f"{codex_home}/config.toml"
    wsl_distro = slot.wsl_distro or "codex-vm"
    sandbox = "workspace-write" if task.allowed_write_set else "read-only"
    command = [
        "wsl",
        "-d",
        wsl_distro,
        "codex",
        "exec",
        "--ephemeral",
        "--json",
        "--color",
        "never",
        "--sandbox",
        sandbox,
        "-c",
        'approval_policy="never"',
        "-c",
        'shell_environment_policy.inherit="none"',
        "-c",
        'shell_environment_policy.set.PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"',
        "-c",
        'shell_environment_policy.set.HOME="/root"',
        "-c",
        "model_providers.OpenAI.request_max_retries=0",
        "-c",
        "model_providers.OpenAI.stream_max_retries=5",
        "-c",
        "model_providers.OpenAI.stream_idle_timeout_ms=120000",
        "--config",
        config_path,
        "-C",
        _wsl_path(worktree_path),
        "-",
    ]
    prompt = taskspec_file.read_text(encoding="utf-8")
    return command, prompt


def parse_codex_events(lines: list[str]) -> dict[str, object]:
    """Parse codex-exec's --json event stream into a small notes dict."""
    final_messages: list[str] = []
    errors: list[str] = []
    turn_started = False
    turn_completed = False
    for line in lines:
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        event_type = str(event.get("type", ""))
        turn_started = turn_started or event_type == "turn.started"
        turn_completed = turn_completed or event_type == "turn.completed"
        if event_type == "error" or "failed" in event_type:
            errors.append(json.dumps(event, ensure_ascii=False))
        item = event.get("item")
        if isinstance(item, dict) and item.get("type") == "agent_message":
            text = item.get("text")
            if isinstance(text, str):
                final_messages.append(text)
    return {
        "turn_started": turn_started,
        "turn_completed": turn_completed,
        "final_message": final_messages[-1] if final_messages else "",
        "errors": errors,
    }


def parse_codex_notes(stdout: str, stderr: str, atif_path: Path) -> str | None:
    """Extract the codex final message + persist a small ATIF-shaped record."""
    parsed = parse_codex_events(stdout.splitlines())
    if atif_path:
        atif_path.parent.mkdir(parents=True, exist_ok=True)
        atif_path.write_text(
            json.dumps(
                {
                    "turn_started": parsed["turn_started"],
                    "turn_completed": parsed["turn_completed"],
                    "final_message": parsed["final_message"],
                    "errors": parsed["errors"],
                    "stderr": stderr,
                },
                indent=2,
                sort_keys=True,
            ),
            encoding="utf-8",
        )
    return parsed["final_message"]