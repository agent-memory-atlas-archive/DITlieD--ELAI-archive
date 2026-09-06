from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

from conductor_config import normalize_path
from models import TaskSpec
from redaction import redact_text
from scope_utils import normalize_repo_path


DEFAULT_MAX_TOTAL_CHARS = 12000
DEFAULT_MAX_FILE_CHARS = 2400


@dataclass(frozen=True)
class ContextPacket:
    task_id: str
    text: str
    digest: str
    source_paths: list[str]
    truncated: bool

    def to_json_dict(self) -> dict[str, object]:
        return {
            "schema_version": 1,
            "task_id": self.task_id,
            "digest": self.digest,
            "source_paths": list(self.source_paths),
            "truncated": self.truncated,
            "char_count": len(self.text),
        }


def build_context_packet_for_task(
    task: TaskSpec,
    repo_root: str | Path,
    *,
    max_total_chars: int = DEFAULT_MAX_TOTAL_CHARS,
    max_file_chars: int = DEFAULT_MAX_FILE_CHARS,
) -> ContextPacket:
    root = normalize_path(repo_root)
    constraint_block = _constraint_block_text(task)
    sections: list[tuple[str, str]] = [
        ("Task", _task_section(task)),
        ("Allowed Writes", "\n".join(f"- {normalize_repo_path(p)}" for p in sorted(task.allowed_write_set))),
        ("Forbidden", "\n".join(f"- {item}" for item in task.forbidden)),
        ("Verifier", json.dumps(task.verifier, indent=2, sort_keys=True)),
    ]

    source_paths: list[str] = []
    truncated = False
    for label, path in _iter_context_sources(task):
        resolved = _resolve_source(root, path)
        if resolved is None or not resolved.is_file():
            continue
        text, was_truncated = _read_bounded_text(resolved, max_file_chars)
        sections.append((label, f"Source: {resolved}\n\n{text}"))
        source_paths.append(str(resolved))
        truncated = truncated or was_truncated

    sections_budget = max(max_total_chars - len(constraint_block), 0)
    packet_text, total_truncated = _cap_sections(sections, sections_budget)
    packet_text = redact_text(packet_text)
    # M89 BLOCK-2: appended AFTER redact_text so a long allowed_write_set path in the
    # constraint block (>=40 chars) is never mistaken for a secret by LONG_TOKEN_RE.
    packet_text = packet_text.rstrip("\n") + constraint_block
    digest = hashlib.sha256(packet_text.encode("utf-8")).hexdigest()[:32]
    return ContextPacket(
        task_id=task.task_id,
        text=packet_text,
        digest=digest,
        source_paths=source_paths,
        truncated=truncated or total_truncated,
    )


def write_context_packet(packet: ContextPacket, task_dir: str | Path) -> tuple[Path, Path]:
    task_path = Path(task_dir)
    task_path.mkdir(parents=True, exist_ok=True)
    text_path = task_path / "context.md"
    meta_path = task_path / "context.json"
    text_path.write_text(packet.text, encoding="utf-8")
    meta_path.write_text(json.dumps(packet.to_json_dict(), indent=2, sort_keys=True), encoding="utf-8")
    return text_path, meta_path


def _task_section(task: TaskSpec) -> str:
    lines = [
        f"task_id: {task.task_id}",
        f"plan_id: {task.plan_id}",
        f"wave: {task.wave}",
        f"role: {task.role}",
        f"base_commit: {task.base_commit}",
        f"production_hook: {task.production_hook or ''}",
        "",
        "Objective:",
        task.objective,
        "",
        "Acceptance:",
        task.acceptance,
    ]
    if task.deterministic_check:
        lines.extend(["", "Deterministic Check:", task.deterministic_check])
    return "\n".join(lines)


def _constraint_block_text(task: TaskSpec) -> str:
    """M9 (teikoku-merge-addon-M8-M9-2026-07-05.md): the imperative CONSTRAINT BLOCK, shared
    with models.py TaskSpec.to_prompt_text's taskspec.txt via the same TaskSpec.
    constraint_lines() method. M89 fix: rendered separately from the section list and
    appended to packet_text AFTER _cap_sections/redact_text run (build_context_packet_for_task
    reserves its length out of max_total_chars up front) instead of living inside the Task
    section text that gets redacted -- a long allowed_write_set path (>=40 chars) is
    otherwise indistinguishable from a secret to redaction.py's LONG_TOKEN_RE and was being
    replaced with <REDACTED_TOKEN>; paths are not secrets."""
    return "\n\n" + "\n".join(task.constraint_lines())


def _iter_context_sources(task: TaskSpec) -> Iterable[tuple[str, str]]:
    seen: set[str] = set()
    for source in [*task.context_refs, *task.allowed_write_set]:
        normalized = normalize_repo_path(source)
        if normalized in seen:
            continue
        seen.add(normalized)
        yield "Context Source", source


def _resolve_source(repo_root: Path, raw_path: str) -> Path | None:
    path = normalize_path(raw_path)
    if path.is_absolute():
        return path
    candidate = repo_root / normalize_repo_path(raw_path)
    if candidate.exists():
        return candidate
    return None


def _read_bounded_text(path: Path, max_chars: int) -> tuple[str, bool]:
    try:
        data = path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        return f"<unreadable: {exc}>", False
    redacted = redact_text(data)
    if len(redacted) <= max_chars:
        return redacted, False
    return redacted[:max_chars] + "\n<truncated>", True


def _cap_sections(sections: list[tuple[str, str]], max_total_chars: int) -> tuple[str, bool]:
    rendered: list[str] = []
    used = 0
    truncated = False
    for title, body in sections:
        block = f"## {title}\n\n{body.strip()}\n"
        if used + len(block) > max_total_chars:
            remaining = max_total_chars - used
            if remaining > 64:
                rendered.append(block[:remaining] + "\n<truncated>")
            truncated = True
            break
        rendered.append(block)
        used += len(block)
    return "\n".join(rendered).strip() + "\n", truncated
