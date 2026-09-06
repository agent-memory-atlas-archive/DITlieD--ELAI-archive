#!/usr/bin/env python3
"""Gate runner for the ELAI Conductor.

Production hook: conductor/gate_runner.py::run_gates
"""
from __future__ import annotations

import argparse
import json
import logging
import os
import sqlite3
import subprocess
import sys
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from conductor_config import normalize_path
from conductor_db import ControlDB
from redaction import redact_text
from scope_utils import normalize_repo_path, scope_violations
from time_utils import iso_utc, utc_now


DEFAULT_DB_PATH = r"F:\.ELAI_workflow\conductor\state\control.db"
DEFAULT_REPO_ROOT = r"F:\.ELAI_workflow\.elai_cc"
DEFAULT_WORKFLOW_ROOT = r"F:\.ELAI_workflow"
DEFAULT_ARTIFACT_ROOT = r"F:\.ELAI_workflow\conductor\artifacts"

LOGGER = logging.getLogger("gate_runner")

# TEIKOKU P3-C W1 (wiring-enforcement R1): META gates are INTEGRITY GUARDS over the
# gate-selection logic itself (freeze / manifest compare), not verifications of the
# TASK's own code. A task that runs ONLY meta gates has verified ZERO of its own code,
# so it must NOT pass. run_gates counts SUBSTANTIVE (non-meta, non-synthetic-guard)
# gate results; zero substantive gates is a hard FAIL. This subsumes the old empty
# command list case AND closes the post-lock fail-OPEN where, once the controller locks
# the distro manifest, a generic no-manifest project would yield only distro_gate_freeze.
META_GATE_NAMES = frozenset({"gate_freeze_check", "distro_gate_freeze"})
# Synthetic results that are NOT verifications of the task's code (a pre-loop policy FAIL
# and the substantive-gate guard itself); excluded from the substantive count.
_NON_SUBSTANTIVE_SYNTHETIC = frozenset({"allowed_write_enforcement", "no_substantive_gates"})


@dataclass(frozen=True)
class GateCommand:
    name: str
    command: list[str]
    cwd: Path
    timeout_seconds: int


@dataclass(frozen=True)
class GateContext:
    task_id: str
    worktree: Path
    base_commit: str
    db_path: Path
    workflow_root: Path
    repo_root: Path
    artifact_root: Path
    allowed_write_set: list[str]
    plan_id: str | None
    touched_files: list[str]
    touched_rs: list[str]
    touched_py: list[str]
    touched_test_rs: list[str]
    new_rust_pub_symbols: list[str]
    new_py_pub_symbols: list[str]
    # TEIKOKU P3-A (master-plan 3.1): the project's active capability packs, which decide
    # the gate set (see gate_packs.build_gate_commands_for_packs). Defaults to the rust pack
    # so the ~4 existing run_gates callers that do not pass packs keep the ELAI/Rust gate set.
    packs: tuple[str, ...] = ("rust",)


def run_gates(
    task_id: str,
    worktree_path: str,
    base_commit: str,
    db_path: str = DEFAULT_DB_PATH,
    allowed_write_set: list[str] | None = None,
    json_output: bool = False,
    *,
    workflow_root: str = DEFAULT_WORKFLOW_ROOT,
    repo_root: str = DEFAULT_REPO_ROOT,
    artifact_root: str = DEFAULT_ARTIFACT_ROOT,
    packs: tuple[str, ...] | list[str] | None = None,
) -> dict[str, Any]:
    start = utc_now()
    worktree = normalize_path(worktree_path)
    db = ControlDB(normalize_path(db_path))
    if not worktree.exists():
        return _error_result(task_id, f"Worktree does not exist: {worktree}", start)
    resolved_base = resolve_base_commit(worktree, base_commit)
    if resolved_base is None:
        return _error_result(task_id, f"Failed to resolve base commit: {base_commit}", start)

    try:
        task_row = db.get_task_row(task_id)
        if allowed_write_set is None:
            allowed_write_set = _allowed_write_set_from_row(task_row)
        plan_id = str(task_row["plan_id"]) if task_row is not None else None
        _make_untracked_visible_to_diff(worktree)
        touched_files = get_touched_files(worktree, resolved_base)
        violations = scope_violations(touched_files, allowed_write_set)
        new_rust_pub_symbols = extract_new_rust_pub_symbols(worktree, resolved_base)
        new_py_pub_symbols = _extract_new_py_pub_symbols(worktree, resolved_base)
        context = GateContext(
            task_id=task_id,
            worktree=worktree,
            base_commit=resolved_base,
            db_path=db.db_path,
            workflow_root=normalize_path(workflow_root),
            repo_root=normalize_path(repo_root),
            artifact_root=normalize_path(artifact_root),
            allowed_write_set=allowed_write_set,
            plan_id=plan_id,
            touched_files=touched_files,
            touched_rs=[f for f in touched_files if f.endswith(".rs")],
            touched_py=[f for f in touched_files if f.endswith(".py")],
            touched_test_rs=[
                f
                for f in touched_files
                if f.endswith(".rs") and ("test" in f or f.endswith("_test.rs") or f.endswith("_contract.rs"))
            ],
            new_rust_pub_symbols=new_rust_pub_symbols,
            new_py_pub_symbols=new_py_pub_symbols,
            packs=tuple(packs) if packs else ("rust",),
        )

        gate_results: list[dict[str, Any]] = []
        structured_feedback: list[str] = []
        overall = "PASS"

        if violations:
            result = _synthetic_result(
                "allowed_write_enforcement",
                "FAIL",
                f"Files outside allowed_write_set were modified: {', '.join(violations)}",
            )
            gate_results.append(result)
            record_gate_result(db, context, result, None)
            structured_feedback.append(f"[allowed_write_enforcement]\n{result['output']}")
            overall = "FAIL"

        gate_commands = build_gate_commands(context)
        for command in gate_commands:
            started_at = _iso_utc_micro()
            db.append_event("GATE_START", task_id, {"gate": command.name, "command": command.command}, created_at=started_at)
            result = execute_gate_command(command, context)
            gate_results.append(result)
            record_gate_result(db, context, result, command, started_at=started_at)
            if result["verdict"] == "PASS":
                db.append_event("GATE_PASS", task_id, {"gate": command.name})
            else:
                db.append_event("GATE_FAIL", task_id, {"gate": command.name, "verdict": result["verdict"]})
                if result["verdict"] == "ERROR" and overall != "FAIL":
                    overall = "ERROR"
                elif result["verdict"] in {"FAIL", "BLOCK", "TIMEOUT", "SKIP"}:
                    overall = "FAIL"
                structured_feedback.append(f"[{command.name}]\n{result['output']}")
            if command.name == "gate_freeze_check" and result["verdict"] != "PASS":
                break

        # fix-b + P3-C W1 (wiring-enforcement R1): a task that runs ZERO SUBSTANTIVE
        # gates must NOT pass. Substantive = a real verification of the task's own code:
        # any gate_result whose name is NOT a META integrity guard (freeze / manifest
        # compare) and NOT a non-verification synthetic guard. Counting substantive
        # RESULTS (not total commands) subsumes the empty-list case (empty -> zero
        # substantive -> FAIL) AND closes the post-lock fail-OPEN: once the controller
        # locks the distro manifest, distro_gate_freeze is prepended for every pack, so a
        # generic no-manifest project would otherwise yield only [distro_gate_freeze]
        # (non-empty) and slip past the old empty-list guard. The verdict authority lives
        # here so the invariant is uniform across packs.
        substantive = [
            r
            for r in gate_results
            if r["gate_name"] not in META_GATE_NAMES
            and r["gate_name"] not in _NON_SUBSTANTIVE_SYNTHETIC
        ]
        if not substantive and overall == "PASS":
            result = _synthetic_result(
                "no_substantive_gates",
                "FAIL",
                "No substantive gates ran for this task: only integrity/meta gates "
                "(gate freeze / manifest compare) executed, so none of the task's own "
                "code was verified (no applicable pack gates and no recognized project "
                "manifest). Per wiring-enforcement R1, missing evidence is a hard FAIL, "
                "not a PASS.",
            )
            gate_results.append(result)
            record_gate_result(db, context, result, None)
            structured_feedback.append("[no_substantive_gates]\n" + result["output"])
            overall = "FAIL"

        return {
            "task_id": task_id,
            "overall_verdict": overall,
            "gate_results": gate_results,
            "touched_files": touched_files,
            "touched_rs": context.touched_rs,
            "touched_py": context.touched_py,
            "new_rust_pub_symbols": context.new_rust_pub_symbols,
            "new_py_pub_symbols": context.new_py_pub_symbols,
            "failure_category": _failure_category(overall, gate_results),
            "regression_module": None,
            "structured_feedback": "\n\n".join(structured_feedback).strip(),
            "timestamp": iso_utc(),
            "duration_s": (utc_now() - start).total_seconds(),
        }
    finally:
        if hasattr(db, "close"):
            db.close()


def build_gate_commands(context: GateContext) -> list[GateCommand]:
    """Build the ordered gate list for the task's worktree.

    TEIKOKU P3-A (master-plan 3.1): the gate SET is now selected by the project's active
    packs instead of being hardcoded ELAI/Rust here. The rust pack reproduces the exact
    pre-P3-A set; a non-rust project never reaches cargo / elai.map. The pack-keyed
    selection lives in gate_packs to keep this module under the maintainability size
    boundary (MR-1 / MR-3).
    """
    from gate_packs import build_gate_commands_for_packs

    return build_gate_commands_for_packs(context, context.packs)

def execute_gate_command(command: GateCommand, context: GateContext) -> dict[str, Any]:
    if _has_forbidden_gate_lock(command.command):
        return _synthetic_result(command.name, "ERROR", "gate_freeze_check --lock is forbidden in conductor runs")
    executable = Path(command.command[1]) if command.command and Path(command.command[0]).name.startswith("python") else Path(command.command[0])
    if len(command.command) > 1 and command.command[1].endswith(".py") and not Path(command.command[1]).exists():
        return _synthetic_result(command.name, "ERROR", f"Verifier missing: {command.command[1]}")
    env = os.environ.copy()
    env["ELAI_PROJECT_ROOT"] = str(context.worktree)
    env["ELAI_MAP"] = str(context.workflow_root / ".claude" / "tools" / "elai.map")
    env.setdefault("CARGO_TARGET_DIR", str(context.worktree / ".cargo_target"))
    try:
        proc = subprocess.run(
            command.command,
            cwd=str(command.cwd),
            capture_output=True,
            text=True,
            errors="replace",
            timeout=command.timeout_seconds,
            env=env,
        )
        output = redact_text((proc.stdout or "") + ("\n" if proc.stdout and proc.stderr else "") + (proc.stderr or ""))
        return {
            "gate_name": command.name,
            "verdict": _verdict_for_exit(command.name, proc.returncode),
            "output": output,
            "exit_code": proc.returncode,
            "duration_s": 0.0,
        }
    except subprocess.TimeoutExpired as exc:
        output = redact_text((exc.stdout or "") + "\n" + (exc.stderr or ""))
        return {
            "gate_name": command.name,
            "verdict": "TIMEOUT",
            "output": output or f"{command.name} timed out",
            "exit_code": -1,
            "duration_s": float(command.timeout_seconds),
        }
    except FileNotFoundError as exc:
        return _synthetic_result(command.name, "ERROR", str(exc))


def record_gate_result(
    db: ControlDB,
    context: GateContext,
    result: dict[str, Any],
    command: GateCommand | None,
    started_at: str | None = None,
) -> None:
    now = _iso_utc_micro()
    started_at = started_at or now
    gate_run_id = str(uuid.uuid4())
    gate_dir = context.artifact_root / context.task_id / "gates" / gate_run_id
    gate_dir.mkdir(parents=True, exist_ok=True)
    stdout_artifact = gate_dir / "output.txt"
    stdout_artifact.write_text(result["output"], encoding="utf-8")
    verdict = result["verdict"]
    db_verdict = "FAIL" if verdict == "SKIP" else verdict
    if db_verdict not in {"PASS", "FAIL", "ERROR", "TIMEOUT", "BLOCK"}:
        db_verdict = "ERROR"
    command_json = json.dumps(command.command) if command is not None else json.dumps([])
    cwd_str = str(command.cwd) if command is not None else str(context.worktree)
    summary = (result["output"] or "")[:1000]
    # gate_runs.verdict has a CHECK constraint limited to
    # ('PASS','FAIL','ERROR','TIMEOUT'); map BLOCK (a failing verdict from
    # production_caller_check) down to FAIL for that table. The verdicts
    # table has no such constraint and keeps the original db_verdict.
    gate_run_verdict = "FAIL" if db_verdict == "BLOCK" else db_verdict
    with db.transaction() as conn:
        conn.execute(
            """
            INSERT INTO verdicts (task_id, gate_name, verdict, output, timestamp)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(task_id, gate_name) DO UPDATE SET
              verdict=excluded.verdict,
              output=excluded.output,
              timestamp=excluded.timestamp
            """,
            (
                context.task_id,
                result["gate_name"],
                db_verdict,
                result["output"][:1000],
                now,
            ),
        )
        conn.execute(
            """
            INSERT INTO gate_runs (
                gate_run_id, task_id, attempt_id, gate_name, command_json, cwd,
                started_at, finished_at, exit_code, verdict,
                stdout_artifact, stderr_artifact, summary
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                gate_run_id,
                context.task_id,
                None,
                result["gate_name"],
                command_json,
                cwd_str,
                started_at,
                now,
                result.get("exit_code"),
                gate_run_verdict,
                str(stdout_artifact),
                None,
                summary,
            ),
        )


def resolve_base_commit(worktree: Path, base_commit: str) -> str | None:
    if base_commit and base_commit != "HEAD":
        return base_commit
    result = subprocess.run(
        ["git", "-C", str(worktree), "rev-parse", "HEAD"],
        capture_output=True,
        text=True,
        errors="replace",
        timeout=30,
    )
    return result.stdout.strip() if result.returncode == 0 else None


def get_touched_files(worktree: Path, base_commit: str) -> list[str]:
    paths: set[str] = set()
    result = subprocess.run(
        ["git", "-C", str(worktree), "diff", "--name-only", base_commit],
        capture_output=True,
        text=True,
        errors="replace",
        timeout=60,
    )
    if result.returncode == 0:
        paths.update(
            normalize_repo_path(path)
            for path in result.stdout.splitlines()
            if path.strip() and not _is_conductor_metadata_path(path)
        )
    status = subprocess.run(
        ["git", "-C", str(worktree), "status", "--porcelain=v1", "--untracked-files=all"],
        capture_output=True,
        text=True,
        errors="replace",
        timeout=60,
    )
    if status.returncode == 0:
        paths.update(_status_paths(status.stdout))
    return sorted(path for path in paths if path)


def extract_new_rust_pub_symbols(worktree: Path, base_commit: str) -> list[str]:
    """Extract new Rust pub symbol names (bare, no prefix) from the worktree diff.

    Used for production_caller_check --symbols mode, which is worktree-compatible
    (unlike --files mode which requires the worktree to be the git repo root).
    """
    result = subprocess.run(
        ["git", "-C", str(worktree), "diff", base_commit],
        capture_output=True,
        text=True,
        errors="replace",
        timeout=60,
    )
    if result.returncode != 0:
        return []
    symbols: set[str] = set()
    for pattern in (
        r"^\+pub\s+fn\s+(\w+)",
        r"^\+pub\s+struct\s+(\w+)",
        r"^\+pub\s+enum\s+(\w+)",
        r"^\+pub\s+trait\s+(\w+)",
        r"^\+pub\s+type\s+(\w+)",
        r"^\+pub\s+const\s+(\w+)",
        r"^\+pub\s+static\s+(\w+)",
    ):
        for match in __import__("re").findall(pattern, result.stdout, __import__("re").MULTILINE):
            symbols.add(match)
    return sorted(symbols)


def _extract_new_py_pub_symbols(worktree: Path, base_commit: str) -> list[str]:
    """Extract new module-level Python public symbol names from the worktree diff.

    Matches added lines starting with ``def``/``class`` at column 0 of the added
    line (unified-diff ``+`` prefix), which corresponds to module-level defs and
    classes; indented methods (``+    def``) are excluded. Used for
    python_wire_check --symbols mode, which is worktree-compatible like the Rust
    production_caller_check --symbols invocation.
    """
    import re

    result = subprocess.run(
        ["git", "-C", str(worktree), "diff", base_commit],
        capture_output=True,
        text=True,
        errors="replace",
        timeout=60,
    )
    if result.returncode != 0:
        return []
    symbols: set[str] = set()
    for pattern in (
        r"^\+def\s+(\w+)\s*\(",
        r"^\+class\s+(\w+)",
    ):
        for match in re.findall(pattern, result.stdout, re.MULTILINE):
            # Skip private names (leading underscore) — only public symbols
            # are subject to the production-wiring gate (R-8).
            if match.startswith("_"):
                continue
            symbols.add(match)
    return sorted(symbols)


def _make_untracked_visible_to_diff(worktree: Path) -> None:
    result = subprocess.run(
        ["git", "-C", str(worktree), "ls-files", "--others", "--exclude-standard", "-z"],
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
            ["git", "-C", str(worktree), "add", "-N", "--", *paths],
            capture_output=True,
            text=True,
            errors="replace",
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


def resolve_plan_file(context: GateContext) -> Path | None:
    if not context.plan_id:
        return None
    candidates = [
        context.workflow_root / "docs" / f"{context.plan_id}.md",
        context.workflow_root / "docs" / "thefinalglance" / f"{context.plan_id}.md",
    ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return None


def affected_crate(context: GateContext, cargo_cwd: Path) -> str | None:
    for touched in context.touched_rs:
        path = context.worktree / touched
        for parent in [path.parent, *path.parents]:
            manifest = parent / "Cargo.toml"
            if manifest.exists() and parent != cargo_cwd:
                # `cargo test -p <X>` needs the PACKAGE name from Cargo.toml, not
                # the directory name. They can differ (crates/orchestrator ->
                # package "elai-orchestrator"), so using the dir name made every
                # Rust task's cargo_test ERROR with "did not match any packages".
                pkg = _cargo_package_name(manifest)
                if pkg:
                    return pkg
            if parent == cargo_cwd:
                break
    return None


def _cargo_package_name(manifest: Path) -> str | None:
    """Read the [package] name from a Cargo.toml (manifest dir name != package)."""
    try:
        in_package = False
        for line in manifest.read_text(encoding="utf-8", errors="replace").splitlines():
            stripped = line.strip()
            if stripped.startswith("["):
                in_package = stripped == "[package]"
                continue
            if in_package and stripped.startswith("name"):
                value = stripped.split("=", 1)[1].strip().strip(chr(34)).strip(chr(39))
                return value or None
    except OSError:
        return None
    return None


def _allowed_write_set_from_row(row: sqlite3.Row | None) -> list[str]:
    if row is None:
        return []
    raw = row["allowed_write_set_json"] if "allowed_write_set_json" in row.keys() else "[]"
    try:
        data = json.loads(raw)
    except json.JSONDecodeError:
        return []
    return [str(item) for item in data]


def _abs_csv(worktree: Path, files: list[str]) -> str:
    return ",".join(str(worktree / file) for file in files)


def _iso_utc_micro() -> str:
    return utc_now().isoformat(timespec="microseconds").replace("+00:00", "Z")


def _has_forbidden_gate_lock(command: list[str]) -> bool:
    return any(part == "--lock" for part in command)


def _verdict_for_exit(gate_name: str, exit_code: int) -> str:
    if exit_code == 0:
        return "PASS"
    if gate_name == "gate_freeze_check" and exit_code == 3:
        return "BLOCK"
    if gate_name == "gate_freeze_check" and exit_code == 2:
        return "ERROR"
    if gate_name == "production_caller_check":
        return "BLOCK" if exit_code == 1 else "ERROR"
    if gate_name == "slop_semantic_check":
        # W4: exit 1 = >=1 TT/TV/DUP finding, all disqualifying -> BLOCK; exit 2
        # (scan-config error: git absent / no files despite the pre-filter) -> ERROR,
        # never a silent pass (fail-closed, wiring-enforcement R1/R2).
        return "BLOCK" if exit_code == 1 else "ERROR"
    if gate_name in {"python_wire_check", "traced_test_assert_check"}:
        return "FAIL" if exit_code == 1 else "ERROR"
    if gate_name == "wiring_contract_check":
        return "FAIL" if exit_code == 1 else ("SKIP" if exit_code == 2 else "ERROR")
    if gate_name == "spec_link_check":
        return "FAIL"
    return "FAIL" if exit_code == 1 else "ERROR"


def _synthetic_result(gate_name: str, verdict: str, output: str) -> dict[str, Any]:
    return {
        "gate_name": gate_name,
        "verdict": verdict,
        "output": redact_text(output),
        "exit_code": -1 if verdict in {"ERROR", "TIMEOUT"} else 1,
        "duration_s": 0.0,
    }


def _failure_category(overall: str, gate_results: list[dict[str, Any]]) -> str | None:
    if overall == "PASS":
        return None
    if any(item["gate_name"] == "allowed_write_enforcement" for item in gate_results):
        return "allowed_write_violation"
    if any(item["gate_name"] == "gate_freeze_check" and item["verdict"] != "PASS" for item in gate_results):
        return "frozen_gate_block"
    if overall == "ERROR":
        return "gate_setup_error"
    return "gate_failure"


def _error_result(task_id: str, message: str, start: Any) -> dict[str, Any]:
    return {
        "task_id": task_id,
        "overall_verdict": "ERROR",
        "gate_results": [],
        "touched_files": [],
        "touched_rs": [],
        "touched_py": [],
        "new_rust_pub_symbols": [],
        "new_py_pub_symbols": [],
        "failure_category": "setup_error",
        "regression_module": None,
        "structured_feedback": redact_text(message),
        "timestamp": iso_utc(),
        "duration_s": (utc_now() - start).total_seconds(),
    }


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run frozen verifier gates against a worktree")
    parser.add_argument("--task-id")
    parser.add_argument("--worktree")
    parser.add_argument("--base-commit")
    parser.add_argument("--db", default=DEFAULT_DB_PATH)
    parser.add_argument("--workflow-root", default=DEFAULT_WORKFLOW_ROOT)
    parser.add_argument("--repo-root", default=DEFAULT_REPO_ROOT)
    parser.add_argument("--artifact-root", default=DEFAULT_ARTIFACT_ROOT)
    parser.add_argument("--json", action="store_true")
    parser.add_argument(
        "--test-smoke",
        action="store_true",
        help="Run smoke test (wire-dark BLOCK / wired PASS) using --symbols mode",
    )
    return parser


def run_smoke_test() -> dict[str, Any]:
    """Run smoke test proving wire-dark BLOCK and wired PASS via --symbols mode.

    Exercises both the Rust production_caller_check path and the Python
    python_wire_check path. For each: a public symbol with no caller must be
    blocked (exit 1), and the same symbol with a caller at a real entrypoint
    must pass (exit 0).
    """
    import shutil

    tools = Path(DEFAULT_WORKFLOW_ROOT) / ".claude" / "tools"
    rust_verifier = tools / "production_caller_check.py"
    py_verifier = tools / "python_wire_check.py"
    if not rust_verifier.exists():
        return {"status": "FAIL", "output": f"Verifier missing: {rust_verifier}"}
    if not py_verifier.exists():
        return {"status": "FAIL", "output": f"Verifier missing: {py_verifier}"}

    # The frozen verifiers use rel_posix() which requires scanned files to be
    # under WORKFLOW_ROOT, so the temp project must live under F:\.ELAI_workflow.
    tmp_root = Path(DEFAULT_WORKFLOW_ROOT) / "conductor" / "test_fixtures" / "smoke_tmp"
    if tmp_root.exists():
        shutil.rmtree(tmp_root, ignore_errors=True)
    tmp_root.mkdir(parents=True, exist_ok=True)
    try:
        # ---- Rust wire-check path ----
        rust_project = tmp_root / "smoke_test_rust"
        rust_project.mkdir()

        # Create Cargo.toml so --root sees a valid project
        (rust_project / "Cargo.toml").write_text(
            '[package]\nname = "smoke_test"\nversion = "0.1.0"\nedition = "2021"\n\n'
            '[lib]\nname = "smoke_test"\npath = "lib.rs"\n',
            encoding="utf-8",
        )

        # Wire-dark: pub fn with no caller
        (rust_project / "lib.rs").write_text(
            "//! Smoke test library\n\n"
            "/// A wire-dark public function (no caller)\n"
            "pub fn wire_dark_function() -> i32 {\n    42\n}\n",
            encoding="utf-8",
        )

        cmd = [
            sys.executable,
            str(rust_verifier),
            "--symbols", "wire_dark_function",
            "--root", str(rust_project),
            "--json",
        ]
        proc = subprocess.run(cmd, capture_output=True, text=True, errors="replace", timeout=30)
        if proc.returncode != 1:
            return {
                "status": "FAIL",
                "output": (
                    f"Rust wire-dark test expected exit code 1, got {proc.returncode}\n"
                    f"{proc.stdout}\n{proc.stderr}"
                ),
            }

        # Wired: pub fn with a caller in main()
        (rust_project / "lib.rs").write_text(
            "//! Smoke test library\n\n"
            "/// A wired public function (has caller in main)\n"
            "pub fn wired_function() -> i32 {\n    42\n}\n\n"
            "fn main() {\n    let _ = wired_function();\n}\n",
            encoding="utf-8",
        )

        cmd_wired = [
            sys.executable,
            str(rust_verifier),
            "--symbols", "wired_function",
            "--root", str(rust_project),
            "--json",
        ]
        proc = subprocess.run(cmd_wired, capture_output=True, text=True, errors="replace", timeout=30)
        if proc.returncode != 0:
            return {
                "status": "FAIL",
                "output": (
                    f"Rust wired test expected exit code 0, got {proc.returncode}\n"
                    f"{proc.stdout}\n{proc.stderr}"
                ),
            }

        # ---- Python wire-check path ----
        py_project = tmp_root / "smoke_test_py"
        py_project.mkdir()

        # Wire-dark: module-level public def with no caller
        (py_project / "module.py").write_text(
            '"""Smoke test module."""\n\n\n'
            'def wire_dark_py_function():\n'
            '    """A wire-dark public function (no caller)."""\n'
            '    return 42\n',
            encoding="utf-8",
        )

        cmd_py_dark = [
            sys.executable,
            str(py_verifier),
            "--symbols", "wire_dark_py_function",
            "--root", str(py_project),
            "--json",
        ]
        proc = subprocess.run(
            cmd_py_dark, capture_output=True, text=True, errors="replace", timeout=30
        )
        if proc.returncode != 1:
            return {
                "status": "FAIL",
                "output": (
                    f"Python wire-dark test expected exit code 1, got {proc.returncode}\n"
                    f"{proc.stdout}\n{proc.stderr}"
                ),
            }

        # Wired: same public def, now called from a real entrypoint (main.py)
        (py_project / "module.py").write_text(
            '"""Smoke test module."""\n\n\n'
            'def wired_py_function():\n'
            '    """A wired public function (has caller in main)."""\n'
            '    return 42\n',
            encoding="utf-8",
        )
        (py_project / "main.py").write_text(
            '"""Smoke test entrypoint."""\n\n'
            'from module import wired_py_function\n\n\n'
            'def main() -> int:\n'
            '    return wired_py_function()\n\n\n'
            'if __name__ == "__main__":\n'
            '    raise SystemExit(main())\n',
            encoding="utf-8",
        )

        cmd_py_wired = [
            sys.executable,
            str(py_verifier),
            "--symbols", "wired_py_function",
            "--root", str(py_project),
            "--json",
        ]
        proc = subprocess.run(
            cmd_py_wired, capture_output=True, text=True, errors="replace", timeout=30
        )
        if proc.returncode != 0:
            return {
                "status": "FAIL",
                "output": (
                    f"Python wired test expected exit code 0, got {proc.returncode}\n"
                    f"{proc.stdout}\n{proc.stderr}"
                ),
            }

        return {
            "status": "PASS",
            "output": (
                "Smoke test passed: Rust wire-dark blocked (exit 1), Rust wired passed (exit 0), "
                "Python wire-dark blocked (exit 1), Python wired passed (exit 0)"
            ),
        }
    except Exception as exc:
        return {"status": "ERROR", "output": f"Smoke test exception: {exc}"}
    finally:
        shutil.rmtree(tmp_root, ignore_errors=True)


def main() -> int:
    logging.basicConfig(level=logging.INFO, format="%(levelname)s %(message)s")
    args = build_parser().parse_args()
    if args.test_smoke:
        result = run_smoke_test()
        if args.json:
            print(json.dumps(result, indent=2, sort_keys=True))
        else:
            print(f"Smoke test: {result['status']}")
            print(result["output"])
        return 0 if result["status"] == "PASS" else 1
    if not args.task_id or not args.worktree or not args.base_commit:
        parser.error("--task-id, --worktree, and --base-commit are required for gate run")
    result = run_gates(
        task_id=args.task_id,
        worktree_path=args.worktree,
        base_commit=args.base_commit,
        db_path=args.db,
        workflow_root=args.workflow_root,
        repo_root=args.repo_root,
        artifact_root=args.artifact_root,
    )
    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print(f"Task: {result['task_id']}")
        print(f"Overall verdict: {result['overall_verdict']}")
        for gate in result["gate_results"]:
            print(f"  {gate['gate_name']}: {gate['verdict']}")
        if result["structured_feedback"]:
            print(result["structured_feedback"])
    return 0 if result["overall_verdict"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
