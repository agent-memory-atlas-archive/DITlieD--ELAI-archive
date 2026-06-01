"""
ELAI full-workflow eval against REAL SWE-Bench Pro tasks, v5.

v4 -> v5 delta (the substrate hypothesis):

v2/v3/v4 drove the model via raw `httpx.post('/v1/messages')` against the
local sidecar with hand-rolled BM25 / code-vec / cross-encoder retrieval
in Python. The Rust orchestrator FSM, per-role PerRoleRuntime,
A-RAG/DKG-backed retrieval, adversary loop, critic-of-design, refusal
gate, audit pipeline, and SSB observability layer were ALL bypassed.
v4 plateaued at composite=0.073.

v5 routes each task through `elai task <problem> --repo-root <repo> ...`,
the non-interactive autonomous subcommand shipped in s-25120. The Rust
substrate now owns the loop: PLANNING -> TESTING_FIRST -> EXECUTING ->
AUDITING -> MEMORIZING with the FSM, ELAI tools (Read/Write/Edit/Bash)
operate on the cloned working tree, retrieval flows through the Rust
knowledge stack (BM25 + code embeddings + DKG, ELAI_CODE_VEC=1 +
ELAI_ARAG_POPULATE_FROM_DKG=1), and the patch is captured post-run from
the working tree via `git diff`. SSB rows are tailed for FSM signals
(phase transitions, agent dispatches, per-level audit verdicts,
findings, refusal decisions). v4 helpers (score_patch, files_in_patch,
clone_repo, load_corpus) are reused unchanged.
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import sqlite3
import subprocess
import sys
import time
import traceback
from pathlib import Path
from typing import Any

_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

# Reuse v2 primitives (corpus, clone, gold-patch parsing, structural scoring).
from swe_bench_pro_elai_workflow_v2 import (  # noqa: E402
    DATASET_NAME,
    DATASET_SPLIT,
    clone_repo,
    files_in_patch,
    load_corpus,
    score_patch,
)


DEFAULT_REPOS_DIR = "/root/swe_eval_repos"
DEFAULT_OUT_JSONL = "/root/swe_bench_pro_elai_v5.jsonl"
DEFAULT_AGG_JSON = "/root/swe_bench_pro_elai_v5.agg.json"
DEFAULT_SIDECAR_BASE_URL = "http://localhost:8100"

# Subprocess wall budget. The Rust side enforces --max-wall-seconds inside;
# we add a 60s grace so the Python subprocess.run timeout is the outer
# circuit-breaker but never the primary deadline.
DEFAULT_TASK_WALL_SECONDS = 1800
SUBPROCESS_GRACE_SECONDS = 60

# SSB action types we tail. Source of truth: SsbActionType in
# orchestrator::ssb (variants kept stable since SCHEMA_VERSION 45).
SSB_ACTIONS_OF_INTEREST = {
    "phase_transition",
    "fsm_transition",
    "agent_lifecycle",
    "agent_dispatch",
    "file_write",
    "finding",
    "per_level_audit_verdict",
    "refusal_decision",
    "phase_complete",
    "cloud_spend",
    "task_submitted",
}


# ------------------------------------------------------------------
# Helpers: toml snapshot/restore, git reset, ssb tail, summary parse
# ------------------------------------------------------------------


def _safe_slug(repo: str, base_commit: str) -> str:
    """Match v2's slug convention so we hit the existing checkout cache."""
    import re

    safe_repo = re.sub(r"[^A-Za-z0-9_.-]", "_", repo)
    short = (base_commit or "nocommit")[:12]
    return f"{safe_repo}__{short}"


def _decode_bytes(b: bytes | str | None) -> str:
    """Decode subprocess output bytes tolerant to non-UTF-8 (e.g. binary git
    diff payloads, latin-1 filenames). Never raises UnicodeDecodeError."""
    if b is None:
        return ""
    if isinstance(b, str):
        return b
    try:
        return b.decode("utf-8", errors="replace")
    except Exception:  # noqa: BLE001
        return ""


def _run(
    cmd: list[str],
    cwd: str | None = None,
    timeout_s: int = 600,
    env: dict[str, str] | None = None,
) -> tuple[int, str, str]:
    # Capture in BYTES mode and decode ourselves with errors="replace".
    # text=True uses the system locale and dies on non-UTF-8 bytes (the v5
    # crash root cause: git diff against NodeBB emitted bytes 0x81 in a
    # binary-patch chunk that the Windows cp1252-or-strict utf-8 codec
    # rejected before any task ever started).
    try:
        p = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            text=False,
            timeout=timeout_s,
            env=env,
        )
        return p.returncode, _decode_bytes(p.stdout), _decode_bytes(p.stderr)
    except subprocess.TimeoutExpired as exc:
        return 124, _decode_bytes(exc.stdout), _decode_bytes(exc.stderr) or "timeout"
    except FileNotFoundError as exc:
        return 127, "", str(exc)
    except OSError as exc:
        return 126, "", f"oserror: {exc}"


def reset_repo_to_base(repo_root: Path, base_commit: str) -> tuple[bool, str]:
    """Restore working tree to base_commit (idempotent between tasks).

    The previous task may have left dirty state; SWE-Bench Pro scoring
    requires a clean tree before each run. We use the cheap path
    (checkout + clean) before falling back to a re-clone.
    """
    if not repo_root.exists():
        return False, "repo_root missing"
    rc, _, err = _run(
        ["git", "reset", "--hard"],
        cwd=str(repo_root),
        timeout_s=120,
    )
    if rc != 0:
        return False, f"reset failed: {err[:200]}"
    rc, _, err = _run(
        ["git", "clean", "-fdx"],
        cwd=str(repo_root),
        timeout_s=120,
    )
    if rc != 0:
        return False, f"clean failed: {err[:200]}"
    if base_commit:
        rc, _, err = _run(
            ["git", "checkout", base_commit],
            cwd=str(repo_root),
            timeout_s=120,
        )
        if rc != 0:
            return False, f"checkout failed: {err[:200]}"
    return True, "reset_ok"


def snapshot_toml(repo_root: Path) -> tuple[bytes | None, Path]:
    """Snapshot elai_cc.toml prior to running `elai task` so we can restore
    after the run (the per-role overrides are durable; gotcha #1 in the
    implementation report)."""
    toml_path = repo_root / "elai_cc.toml"
    if not toml_path.exists():
        return None, toml_path
    try:
        return toml_path.read_bytes(), toml_path
    except OSError:
        return None, toml_path


def restore_toml(snapshot: bytes | None, toml_path: Path) -> bool:
    if snapshot is None:
        # Nothing was there before; if elai task created one, remove it.
        if toml_path.exists():
            try:
                toml_path.unlink()
            except OSError:
                return False
        return True
    try:
        toml_path.write_bytes(snapshot)
        return True
    except OSError:
        return False


def _git_diff_bytes_capture(
    cmd: list[str], cwd: str, timeout_s: int = 120
) -> tuple[int, str]:
    """Run a git diff in bytes mode and decode with errors='replace'.

    Git diff output for repos with binary files (images, compiled
    artifacts, mixed-encoding source) emits raw bytes that are NOT valid
    UTF-8. text=True crashes the driver before any task body runs. The
    SWE-Bench Pro NodeBB row crashed on byte 0x81 at position 0 which is
    a classic Windows-1252 or binary-patch payload. We tolerate it via
    bytes capture + lossy decode (the structural scorer only needs
    +/- line prefixes and file paths, both ASCII)."""
    try:
        p = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            text=False,
            timeout=timeout_s,
        )
        return p.returncode, _decode_bytes(p.stdout)
    except subprocess.TimeoutExpired as exc:
        return 124, _decode_bytes(exc.stdout)
    except (FileNotFoundError, OSError):
        return 127, ""


def git_diff_from_base(repo_root: Path, base_commit: str) -> str:
    """Capture the patch from the working tree post-run. ELAI's task_cmd
    leaves TaskSummary.patch=null (RUNPOD-PENDING in the implementation
    report); we read it ourselves the SWE-Bench Pro way."""
    if not repo_root.exists():
        return ""
    # Cover both cases: ELAI committed its changes, or left them in the
    # working tree. `git diff <base>` covers both because git compares the
    # base tree to the current HEAD (if committed) plus we suffix the
    # working-tree diff separately.
    parts: list[str] = []
    if base_commit:
        rc, out = _git_diff_bytes_capture(
            ["git", "diff", base_commit, "HEAD"],
            cwd=str(repo_root),
            timeout_s=120,
        )
        if rc == 0 and out:
            parts.append(out)
    rc, out = _git_diff_bytes_capture(
        ["git", "diff"],
        cwd=str(repo_root),
        timeout_s=120,
    )
    if rc == 0 and out:
        parts.append(out)
    return "\n".join(parts)


def parse_task_summary(stdout: str | bytes | None) -> dict[str, Any]:
    """The last non-empty line of stdout is the JSONL TaskSummary."""
    if not stdout:
        return {}
    if isinstance(stdout, bytes):
        stdout = _decode_bytes(stdout)
    for line in reversed(stdout.splitlines()):
        line = line.strip()
        if not line:
            continue
        if not line.startswith("{"):
            # The summary line is JSON; skip non-JSON noise above it.
            continue
        try:
            return json.loads(line)
        except json.JSONDecodeError:
            continue
    return {}


def tail_ssb(ssb_path: str, task_id: str | None) -> dict[str, Any]:
    """Pull a compact post-hoc summary out of the SSB sqlite for this task.

    Read-only attach via the sqlite URI form. We accept rows by action
    type and (when possible) filter by project/task id; column names
    follow the SsbEntry schema (created_at, action_type, payload JSON
    blob, project_id). If the schema in this build differs slightly, we
    degrade gracefully by returning what we could parse.
    """
    if not ssb_path:
        return {}
    p = Path(ssb_path)
    if not p.exists():
        return {"ssb_error": f"not found: {ssb_path}"}

    summary: dict[str, Any] = {
        "ssb_path": ssb_path,
        "rows_by_action_type": {},
        "phase_transitions": [],
        "audit_verdicts": [],
        "findings": [],
        "agent_dispatches": [],
        "refusal_decisions": [],
        "ssb_error": "",
    }

    uri = f"file:{p.as_posix()}?mode=ro"
    try:
        conn = sqlite3.connect(uri, uri=True, timeout=10)
    except sqlite3.Error as exc:
        summary["ssb_error"] = f"connect failed: {exc}"
        return summary

    try:
        cur = conn.cursor()
        # Discover the actual table; the canonical name is `ssb_entries`
        # but defensively probe both common variants.
        table = None
        for candidate in ("ssb_entries", "ssb", "entries"):
            try:
                cur.execute(f"SELECT 1 FROM {candidate} LIMIT 1")
                table = candidate
                break
            except sqlite3.OperationalError:
                continue
        if table is None:
            summary["ssb_error"] = "no ssb table found"
            return summary

        # Counts per action type (whole DB; per-task filtering happens by
        # action-payload inspection below where applicable).
        try:
            cur.execute(
                f"SELECT action_type, COUNT(*) FROM {table} GROUP BY action_type"
            )
            for action, n in cur.fetchall():
                summary["rows_by_action_type"][action] = int(n)
        except sqlite3.OperationalError as exc:
            summary["ssb_error"] = f"count query failed: {exc}"

        # Fetch rows of interest. Limit to a generous cap so a runaway
        # build can't OOM the driver.
        placeholders = ",".join("?" for _ in SSB_ACTIONS_OF_INTEREST)
        try:
            cur.execute(
                f"SELECT action_type, payload FROM {table} "
                f"WHERE action_type IN ({placeholders}) "
                f"ORDER BY id ASC LIMIT 5000",
                tuple(SSB_ACTIONS_OF_INTEREST),
            )
            rows = cur.fetchall()
        except sqlite3.OperationalError as exc:
            summary["ssb_error"] = f"select failed: {exc}"
            rows = []

        for action_type, payload_raw in rows:
            try:
                # sqlite TEXT columns sometimes come back as bytes when the
                # row was inserted as a Python bytes blob. Decode lossy so
                # one corrupted SSB row does not poison the whole tail.
                if isinstance(payload_raw, bytes):
                    payload_raw = payload_raw.decode("utf-8", errors="replace")
                payload = (
                    json.loads(payload_raw)
                    if isinstance(payload_raw, str)
                    else {}
                )
            except (json.JSONDecodeError, TypeError, UnicodeDecodeError):
                continue
            if action_type in {"phase_transition", "fsm_transition"}:
                summary["phase_transitions"].append(
                    {
                        "from": payload.get("from_state") or payload.get("from"),
                        "to": payload.get("to_state") or payload.get("to"),
                        "ts": payload.get("ts") or payload.get("created_at"),
                    }
                )
            elif action_type == "per_level_audit_verdict":
                summary["audit_verdicts"].append(
                    {
                        "level": payload.get("level"),
                        "verdict": payload.get("verdict"),
                        "blocks": payload.get("blocks"),
                        "warns": payload.get("warns"),
                    }
                )
            elif action_type == "finding":
                summary["findings"].append(
                    {
                        "severity": payload.get("severity"),
                        "category": payload.get("category"),
                        "message": (payload.get("message") or "")[:200],
                    }
                )
            elif action_type in {"agent_lifecycle", "agent_dispatch"}:
                summary["agent_dispatches"].append(
                    {
                        "agent": payload.get("agent")
                        or payload.get("agent_id")
                        or payload.get("role"),
                        "phase": payload.get("phase_kind") or payload.get("phase"),
                        "event": payload.get("event") or action_type,
                    }
                )
            elif action_type == "refusal_decision":
                summary["refusal_decisions"].append(
                    {
                        "verdict": payload.get("verdict"),
                        "reason": (payload.get("reason") or "")[:200],
                        "score": payload.get("score"),
                    }
                )
    finally:
        try:
            conn.close()
        except sqlite3.Error:
            pass

    return summary


# ------------------------------------------------------------------
# Per-task runner
# ------------------------------------------------------------------


def build_elai_task_cmd(
    problem_statement: str,
    repo_root: Path,
    mode: str,
    executor_runtime: str,
    auditor_runtime: str,
    max_wall_seconds: int,
    elai_bin: str,
) -> list[str]:
    return [
        elai_bin,
        "task",
        problem_statement,
        "--repo-root",
        str(repo_root),
        "--mode",
        mode,
        "--output-format",
        "json",
        "--max-wall-seconds",
        str(max_wall_seconds),
        "--executor-runtime",
        executor_runtime,
        "--auditor-runtime",
        auditor_runtime,
    ]


def build_elai_task_env(
    mode: str,
    anthropic_api_key: str | None,
    anthropic_base_url: str,
) -> dict[str, str]:
    env: dict[str, str] = {**os.environ}
    feature_flags = {
        "ELAI_PLANNING_DECOMPOSITION_ENABLE": "1",
        "ELAI_ADVERSARY_ENABLE": "1",
        "ELAI_REASONING_QUALITY_PROBE_ENABLE": "1",
        "ELAI_CRITIC_OF_DESIGN_ENABLE": "1",
        "ELAI_SKILL_SYSTEM_ENABLE": "1",
        "ELAI_AI_ROADMAP_SIGNALS_ENABLE": "1",
        "ELAI_CODE_VEC": "1",
        "ELAI_ARAG_POPULATE_FROM_DKG": "1",
        "ANTHROPIC_BASE_URL": anthropic_base_url,
    }
    env.update(feature_flags)
    if mode in {"hybrid", "cloud"}:
        if not anthropic_api_key:
            raise ValueError(
                f"--mode={mode} requires --anthropic-api-key (or ANTHROPIC_API_KEY env)"
            )
        env["ANTHROPIC_API_KEY"] = anthropic_api_key
    elif anthropic_api_key:
        # Local mode: don't leak a real key into the env, use the dummy
        # so the AnthropicClient still constructs against the sidecar.
        env["ANTHROPIC_API_KEY"] = "local-dummy"
    else:
        env["ANTHROPIC_API_KEY"] = "local-dummy"
    return env


def derive_outcome(
    exit_code: int,
    summary: dict[str, Any],
    patch: str,
) -> str:
    """Map (exit_code, TaskSummary, patch) to a single outcome label.

    The Rust side already sets summary['outcome'] using the same decision
    tree (see task_cmd.rs::emit_summary); we trust it when present and
    only synthesize when missing/corrupt.
    """
    if summary.get("outcome"):
        return str(summary["outcome"])
    if exit_code == 2:
        return "timeout"
    if exit_code == 3:
        return "config_error"
    if exit_code != 0:
        return "wiring_error"
    if summary.get("refusal_decision"):
        return "refused"
    if (summary.get("findings_block") or 0) > 0:
        return "blocked"
    if patch.strip():
        return "completed"
    return "no_patch"


def run_one_task(
    task: dict,
    repos_dir: Path,
    mode: str,
    executor_runtime: str,
    auditor_runtime: str,
    max_wall_seconds: int,
    elai_bin: str,
    anthropic_api_key: str | None,
    anthropic_base_url: str,
) -> dict:
    record: dict[str, Any] = {
        "instance_id": task["instance_id"],
        "repo": task["repo"],
        "base_commit": task.get("base_commit", ""),
        "ts_unix": int(time.time()),
        "version": "v5",
        "mode": mode,
        "executor_runtime": executor_runtime,
        "auditor_runtime": auditor_runtime,
        "exit_code": None,
        "outcome": "",
        "clone_status": "not_attempted",
        "reset_status": "not_attempted",
    }

    # Step 1: clone (idempotent) — reuses v2's logic.
    repo_root, clone_status = clone_repo(
        task["repo"], task["base_commit"], repos_dir
    )
    record["clone_status"] = clone_status
    if repo_root is None or not repo_root.exists():
        record["outcome"] = "clone_failed"
        record["score"] = score_patch("", task["gold_patch"])
        return record

    # Step 2: reset working tree to base_commit before each run (defensive,
    # the prior task may have left dirty state).
    ok, reset_status = reset_repo_to_base(repo_root, task["base_commit"])
    record["reset_status"] = reset_status
    if not ok:
        # Hard fallback: wipe + re-clone so we never run against contaminated
        # state. We tolerate clone failure with a recorded outcome.
        shutil.rmtree(repo_root, ignore_errors=True)
        repo_root, clone_status = clone_repo(
            task["repo"], task["base_commit"], repos_dir
        )
        record["clone_status"] = f"recloned:{clone_status}"
        if repo_root is None or not repo_root.exists():
            record["outcome"] = "reset_failed"
            record["score"] = score_patch("", task["gold_patch"])
            return record

    # Step 3: snapshot elai_cc.toml so the per-role override is non-durable
    # across tasks (implementation-report gotcha #1).
    toml_snap, toml_path = snapshot_toml(repo_root)

    # Step 4: build and dispatch `elai task ...`.
    try:
        env = build_elai_task_env(
            mode=mode,
            anthropic_api_key=anthropic_api_key,
            anthropic_base_url=anthropic_base_url,
        )
    except ValueError as exc:
        record["outcome"] = "config_error"
        record["error"] = str(exc)
        record["score"] = score_patch("", task["gold_patch"])
        restore_toml(toml_snap, toml_path)
        return record

    cmd = build_elai_task_cmd(
        problem_statement=task["problem_statement"],
        repo_root=repo_root,
        mode=mode,
        executor_runtime=executor_runtime,
        auditor_runtime=auditor_runtime,
        max_wall_seconds=max_wall_seconds,
        elai_bin=elai_bin,
    )

    t0 = time.time()
    timeout_outer = max_wall_seconds + SUBPROCESS_GRACE_SECONDS
    try:
        # Bytes-mode + tolerant decode. The elai subprocess may emit
        # non-UTF-8 bytes via tracing layers on Windows, or via embedded
        # repo paths with non-ASCII filenames. text=True would crash the
        # driver before any task body ran.
        proc = subprocess.run(
            cmd,
            env=env,
            capture_output=True,
            text=False,
            timeout=timeout_outer,
        )
        exit_code = proc.returncode
        stdout = _decode_bytes(proc.stdout)
        stderr = _decode_bytes(proc.stderr)
        elai_timed_out = False
    except subprocess.TimeoutExpired as exc:
        exit_code = 124  # outer-timeout convention (separate from Rust's 2)
        stdout = _decode_bytes(exc.stdout)
        stderr = _decode_bytes(exc.stderr)
        elai_timed_out = True

    wall_s = round(time.time() - t0, 2)
    record["exit_code"] = exit_code
    record["wall_s"] = wall_s
    record["elai_outer_timeout"] = elai_timed_out

    # Step 5: parse the TaskSummary JSONL.
    summary = parse_task_summary(stdout)
    record["raw_summary"] = summary
    if not summary:
        record["raw_stdout_tail"] = stdout[-2000:] if stdout else ""
        record["raw_stderr_tail"] = stderr[-2000:] if stderr else ""

    # Step 6: pull the patch from the working tree (TaskSummary.patch is null
    # per the implementation report's RUNPOD-PENDING note).
    patch = git_diff_from_base(repo_root, task["base_commit"])
    record["patch"] = patch[:6000]
    record["patch_full_len"] = len(patch)

    # Step 7: SSB tail (read-only).
    ssb_path = summary.get("ssb_path") or ""
    ssb_summary = tail_ssb(ssb_path, summary.get("task_id"))
    record["ssb_summary"] = ssb_summary
    record["agent_dispatch_count"] = len(ssb_summary.get("agent_dispatches", []))
    record["phase_transition_count"] = len(ssb_summary.get("phase_transitions", []))
    record["audit_verdict"] = summary.get("audit_verdict")
    record["refusal_decision"] = summary.get("refusal_decision")
    record["findings_total"] = summary.get("findings_total")
    record["findings_block"] = summary.get("findings_block")
    record["cost_usd"] = summary.get("cost_usd")
    record["tokens_in"] = summary.get("tokens_in")
    record["tokens_out"] = summary.get("tokens_out")
    record["files_touched"] = summary.get("files_touched", [])

    # Step 8: outcome label + structural scoring.
    outcome = derive_outcome(exit_code, summary, patch)
    record["outcome"] = outcome
    if not patch.strip():
        record["score"] = {
            "files_overlap": 0.0,
            "hunks_overlap": 0.0,
            "funcs_overlap": 0.0,
            "composite": 0.0,
            "patch_present": False,
        }
    else:
        record["score"] = score_patch(patch, task["gold_patch"])

    # Cross-check: do the FileWrite SSB rows agree with the diff?
    diff_files = files_in_patch(patch) if patch else set()
    record["diff_files"] = sorted(diff_files)
    record["files_touched_ssb"] = sorted(set(summary.get("files_touched", []) or []))
    record["gold_patch_preview"] = task["gold_patch"][:600]

    # Step 9: restore elai_cc.toml so the next task starts clean.
    record["toml_restored"] = restore_toml(toml_snap, toml_path)

    return record


# ------------------------------------------------------------------
# Main
# ------------------------------------------------------------------


def _load_tasks_from_jsonl(path: Path, n_tasks: int) -> list[dict]:
    out: list[dict] = []
    # errors="replace": local task JSONL caches may contain non-UTF-8
    # bytes copied verbatim from HF dataset rows (problem_statement /
    # gold_patch can contain odd encodings from upstream issue trackers).
    # Crashing the corpus loader before any task runs is strictly worse
    # than lossy decode of a single character in a problem statement.
    with path.open("r", encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            # Required keys for v5: instance_id, repo, base_commit,
            # problem_statement, gold_patch.
            if not row.get("repo") or not row.get("problem_statement"):
                continue
            # Coerce bytes -> str for any HF-cached fields that arrived as
            # bytes (rare but observed on Windows builds of pyarrow).
            for k, v in list(row.items()):
                if isinstance(v, bytes):
                    row[k] = v.decode("utf-8", errors="replace")
            out.append(row)
            if len(out) >= n_tasks:
                break
    return out


def main() -> int:
    p = argparse.ArgumentParser(
        description=(
            "v5 SWE-Bench Pro driver. Subprocess-wraps `elai task` so the "
            "Rust FSM owns the loop; computes structural composite vs gold "
            "patch and tails the SSB sqlite for per-level audit verdicts, "
            "agent dispatches, refusal decisions, and findings."
        )
    )
    p.add_argument(
        "--num-tasks",
        type=int,
        default=5,
        help="Number of unique-repo tasks to evaluate.",
    )
    p.add_argument(
        "--mode",
        choices=("local", "hybrid", "cloud"),
        default="hybrid",
        help=(
            "PerRoleRuntime mode. hybrid = local sidecar executor + cloud "
            "auditor (recommended for ELAI substrate signal). local = "
            "sidecar everywhere; cloud = anthropic_api everywhere."
        ),
    )
    p.add_argument(
        "--anthropic-api-key",
        default=os.environ.get("ANTHROPIC_API_KEY", ""),
        help=(
            "Required when --mode=hybrid or cloud. Defaults to "
            "$ANTHROPIC_API_KEY when present."
        ),
    )
    p.add_argument(
        "--anthropic-base-url",
        default=os.environ.get("ANTHROPIC_BASE_URL", DEFAULT_SIDECAR_BASE_URL),
        help="Local sidecar tunnel URL (default http://localhost:8100).",
    )
    p.add_argument(
        "--executor-runtime",
        default="sidecar",
        help="Per-role runtime backend for the executor role.",
    )
    p.add_argument(
        "--auditor-runtime",
        default="anthropic_api",
        help=(
            "Per-role runtime backend for the auditor role. Only meaningful "
            "for hybrid/cloud modes."
        ),
    )
    p.add_argument(
        "--max-wall-seconds",
        type=int,
        default=DEFAULT_TASK_WALL_SECONDS,
        help="Per-task wall budget passed to `elai task --max-wall-seconds`.",
    )
    p.add_argument(
        "--elai-bin",
        default="elai",
        help="Path to the elai binary (defaults to PATH lookup).",
    )
    p.add_argument(
        "--repos-dir",
        default=DEFAULT_REPOS_DIR,
        help="Directory under which task repos are shallow-cloned.",
    )
    p.add_argument(
        "--output-jsonl",
        default=DEFAULT_OUT_JSONL,
        help="Per-task JSONL output path.",
    )
    p.add_argument(
        "--aggregate-json",
        default=DEFAULT_AGG_JSON,
        help="Aggregate summary JSON output path.",
    )
    p.add_argument(
        "--tasks-jsonl",
        default="",
        help=(
            "Optional pre-baked task JSONL (one row per task with "
            "instance_id/repo/base_commit/problem_statement/gold_patch). "
            "When set, bypasses the HuggingFace stream and reads locally."
        ),
    )
    p.add_argument(
        "--pod-host",
        default="",
        help=(
            "Informational only: SSH host of the GPU pod. The driver runs ON "
            "the pod, so this is for record-keeping in the aggregate."
        ),
    )
    p.add_argument(
        "--pod-port",
        type=int,
        default=0,
        help="Informational only: SSH port of the GPU pod.",
    )
    p.add_argument(
        "--pod-key",
        default="",
        help="Informational only: SSH key path of the GPU pod.",
    )
    args = p.parse_args()

    repos_dir = Path(args.repos_dir)
    repos_dir.mkdir(parents=True, exist_ok=True)

    # Load tasks: either from a local JSONL or from the public dataset.
    if args.tasks_jsonl:
        tasks_path = Path(args.tasks_jsonl)
        if not tasks_path.exists():
            sys.stderr.write(f"tasks-jsonl not found: {tasks_path}\n")
            return 2
        print(
            f"[corpus] loading tasks from {tasks_path} (cap={args.num_tasks})...",
            flush=True,
        )
        tasks = _load_tasks_from_jsonl(tasks_path, args.num_tasks)
    else:
        print(
            f"[corpus] streaming {args.num_tasks} tasks from {DATASET_NAME} "
            f"split={DATASET_SPLIT}...",
            flush=True,
        )
        tasks = load_corpus(args.num_tasks)
    # Coerce bytes-typed fields to str across all task rows, regardless of
    # source (local JSONL or HF stream). HF datasets stored as parquet on
    # Windows sometimes hand back bytes for nullable string columns.
    for t in tasks:
        for k, v in list(t.items()):
            if isinstance(v, bytes):
                t[k] = v.decode("utf-8", errors="replace")
    print(f"[corpus] loaded {len(tasks)} tasks", flush=True)
    print(
        f"[config] mode={args.mode} executor={args.executor_runtime} "
        f"auditor={args.auditor_runtime} wall={args.max_wall_seconds}s "
        f"repos_dir={repos_dir} elai_bin={args.elai_bin}",
        flush=True,
    )

    if args.mode in {"hybrid", "cloud"} and not args.anthropic_api_key:
        sys.stderr.write(
            f"--mode={args.mode} requires --anthropic-api-key or "
            "ANTHROPIC_API_KEY env\n"
        )
        return 3

    all_records: list[dict] = []
    out_path = Path(args.output_jsonl)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    with out_path.open("w", encoding="utf-8") as f:
        for i, task in enumerate(tasks):
            print(
                f"[task {i+1}/{len(tasks)}] {task['instance_id']} "
                f"({task['repo']}) base={task.get('base_commit', '')[:12]}",
                flush=True,
            )
            try:
                rec = run_one_task(
                    task,
                    repos_dir=repos_dir,
                    mode=args.mode,
                    executor_runtime=args.executor_runtime,
                    auditor_runtime=args.auditor_runtime,
                    max_wall_seconds=args.max_wall_seconds,
                    elai_bin=args.elai_bin,
                    anthropic_api_key=args.anthropic_api_key or None,
                    anthropic_base_url=args.anthropic_base_url,
                )
            except Exception as exc:  # noqa: BLE001  (driver-level catch is intentional)
                # Full traceback so a single task failure is diagnosable
                # without re-running. str(exc) alone hid the v5
                # UnicodeDecodeError frame which made the crash look like
                # it came from anywhere in the pipeline.
                tb_str = traceback.format_exc()
                try:
                    gold_score = score_patch("", task.get("gold_patch", "") or "")
                except Exception:  # noqa: BLE001
                    gold_score = {"composite": 0.0}
                rec = {
                    "instance_id": task.get("instance_id", "unknown"),
                    "repo": task.get("repo", "unknown"),
                    "version": "v5",
                    "outcome": "driver_error",
                    "error": str(exc)[:500],
                    "error_type": type(exc).__name__,
                    "error_traceback": tb_str[-4000:],
                    "score": gold_score,
                }

            f.write(json.dumps(rec) + "\n")
            f.flush()
            all_records.append(rec)

            sc = rec.get("score", {})
            print(
                f"  outcome={rec.get('outcome')} exit={rec.get('exit_code')} "
                f"composite={sc.get('composite', 0.0)} "
                f"files={sc.get('files_overlap', 0.0)} "
                f"hunks={sc.get('hunks_overlap', 0.0)} "
                f"funcs={sc.get('funcs_overlap', 0.0)} "
                f"agents={rec.get('agent_dispatch_count', 0)} "
                f"phases={rec.get('phase_transition_count', 0)} "
                f"wall={rec.get('wall_s', 0.0)}s",
                flush=True,
            )

    if all_records:
        comps = [r.get("score", {}).get("composite", 0.0) for r in all_records]
        files_overlaps = [
            r.get("score", {}).get("files_overlap", 0.0) for r in all_records
        ]
        hunks_overlaps = [
            r.get("score", {}).get("hunks_overlap", 0.0) for r in all_records
        ]
        funcs_overlaps = [
            r.get("score", {}).get("funcs_overlap", 0.0) for r in all_records
        ]
        walls = [r.get("wall_s", 0.0) for r in all_records]
        exit_codes: dict[str, int] = {}
        outcomes: dict[str, int] = {}
        for r in all_records:
            exit_codes[str(r.get("exit_code"))] = (
                exit_codes.get(str(r.get("exit_code")), 0) + 1
            )
            outcomes[str(r.get("outcome"))] = (
                outcomes.get(str(r.get("outcome")), 0) + 1
            )
        agg = {
            "n_tasks": len(all_records),
            "mean_composite": round(sum(comps) / len(comps), 4),
            "max_composite": round(max(comps), 4) if comps else 0.0,
            "mean_files_overlap": round(
                sum(files_overlaps) / len(files_overlaps), 4
            ),
            "mean_hunks_overlap": round(
                sum(hunks_overlaps) / len(hunks_overlaps), 4
            ),
            "mean_funcs_overlap": round(
                sum(funcs_overlaps) / len(funcs_overlaps), 4
            ),
            "mean_wall_s": round(sum(walls) / max(len(walls), 1), 2),
            "exit_code_dist": exit_codes,
            "outcome_dist": outcomes,
            "mode": args.mode,
            "executor_runtime": args.executor_runtime,
            "auditor_runtime": args.auditor_runtime,
            "max_wall_seconds": args.max_wall_seconds,
            "dataset": DATASET_NAME if not args.tasks_jsonl else "local_jsonl",
            "dataset_split": DATASET_SPLIT if not args.tasks_jsonl else "",
            "tasks_jsonl": args.tasks_jsonl,
            "version": "v5",
            "ts_unix": int(time.time()),
            "pod_host": args.pod_host,
            "pod_port": args.pod_port,
        }
        with Path(args.aggregate_json).open("w", encoding="utf-8") as fa:
            json.dump(agg, fa, indent=2)
        print("[aggregate]", json.dumps(agg, indent=2), flush=True)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
