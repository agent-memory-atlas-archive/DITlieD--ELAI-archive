#!/usr/bin/env python3
"""Depot offload for conductor cargo / heavy CI — keep the owner PC free.

WHY
  Agent worktrees each grow multi-GB cargo target dirs and thrash the host CPU.
  Depot already has remote runners (proved 2026-07-29 live-finish: D-U3/D-U4/D-U5).
  This module is the production hook that makes the NEW conductor USE those runners
  by default instead of shelling local `cargo`.

POLICY (TEIKOKU_DEPOT_OFFLOAD)
  remote  — default. Heavy cargo MUST run on Depot; fail closed if preflight fails.
  prefer  — use the remote route but fail closed on remote failure.
  local   — never call Depot (emergency / offline tests).

SCOPE
  Offloads: cargo check | build | test | clippy (and gate names cargo_*).
  Does NOT place fleet LLM workers on Depot CI (P-F4 REJECTED-FOR-NOW stays).
  CUA / desktop proof: separate workflow path (see desktop_proof_workflow).

CLI
  python3 depot_offload.py preflight
  python3 depot_offload.py run -- cargo check --workspace
  python3 depot_offload.py classify /path/to/cwd
"""
from __future__ import annotations

import argparse
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import time
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence, Tuple

SCHEMA_VERSION = 1

# Default org / repos proved live 2026-07-29 (inventory D-U1..D-U4 CLOSED-GREEN).
DEFAULT_ORG = "example-org"
DEFAULT_TEIKOKU_REPO = "example/automation-workspace"
DEFAULT_ELAI_REPO = "example/ELAI"
DEFAULT_PROJECT_ID = "example-project"
DEFAULT_WORKFLOW = ".depot/workflows/conductor-cargo.yml"
DEFAULT_ELAI_WORKFLOW = ".depot/workflows/elai-cargo.yml"
REQUEST_REL = Path(".depot") / "conductor-cargo-request.json"
# Repos where `depot ci run` is known live (proved 2026-08-04: LIA elai-cargo).
# example/automation-workspace currently returns permission_denied until the Depot GitHub App
# is installed — zenov uses depot *build* fallback instead.
DEFAULT_CI_REPOS = ("example/ELAI",)

HEAVY_CARGO_SUBCMDS = frozenset(
    {
        "check",
        "build",
        "test",
        "clippy",
        "bench",
        "rustc",
        "run",
        "install",
        "fix",
        "doc",
        "rustdoc",
        "package",
    }
)
SMALL_CARGO_SUBCMDS = frozenset(
    {
        "version",
        "help",
        "list",
        "fmt",
        "metadata",
        "locate-project",
        "read-manifest",
        "pkgid",
        "verify-project",
        "tree",
    }
)
CONTAINER_COMMAND_NAMES = frozenset({"docker", "docker.exe", "podman", "podman.exe"})
CARGO_GLOBAL_FLAGS = frozenset(
    {
        "--frozen",
        "--locked",
        "--offline",
        "--quiet",
        "--verbose",
        "-q",
        "-v",
        "-vv",
        "-vvv",
        "-h",
        "--help",
        "-V",
        "--version",
        "--list",
    }
)
CARGO_GLOBAL_OPTIONS_WITH_VALUES = frozenset(
    {"--color", "--config", "--manifest-path", "-Z"}
)
CARGO_SUBCOMMAND_OPTIONS_WITH_VALUES = {
    "metadata": frozenset({"--features", "--filter-platform", "--format-version"}),
}
CARGO_SUBCOMMAND_FLAGS = {
    "metadata": frozenset({"--all-features", "--no-deps"}),
    "fmt": frozenset({"--all", "--check"}),
}
GATE_NAMES_OFFLOAD = frozenset({"cargo_check", "cargo_test", "cargo_build", "cargo_clippy"})
PROFILE_REGISTRY_PATH = Path(__file__).with_name("profile_registry.py")

VALID_HEAD_RE = re.compile(r"^[0-9a-fA-F]{40}$")


class DepotOffloadError(RuntimeError):
    """Fail-closed: remote required but unavailable or remote job failed."""


@dataclass(frozen=True)
class OffloadConfig:
    mode: str  # remote | prefer | local
    org: str
    teikoku_repo: str
    elai_repo: str
    project_id: str
    workflow_teikoku: str
    workflow_elai: str
    timeout_sec: int
    depot_bin: str
    ci_repos: Tuple[str, ...]
    build_fallback: bool

    @staticmethod
    def load(path: Optional[Path] = None) -> "OffloadConfig":
        env_mode = (os.environ.get("TEIKOKU_DEPOT_OFFLOAD") or "").strip().lower()
        cfg_path = path
        if cfg_path is None:
            env_p = (os.environ.get("TEIKOKU_DEPOT_OFFLOAD_CONFIG") or "").strip()
            if env_p:
                cfg_path = Path(env_p)
            else:
                here = Path(__file__).resolve()
                candidates = [
                    here.parent / "depot_offload_config.json",
                    Path("/home/researcher/teikoku/conductor/state/depot_offload_config.json"),
                    Path.home() / ".config" / "teikoku" / "depot_offload_config.json",
                ]
                for c in candidates:
                    if c.is_file():
                        cfg_path = c
                        break
        raw: Dict[str, Any] = {}
        if cfg_path is not None and cfg_path.is_file():
            try:
                loaded = json.loads(cfg_path.read_text(encoding="utf-8"))
                if isinstance(loaded, dict):
                    raw = loaded
            except (OSError, json.JSONDecodeError):
                raw = {}
        mode = env_mode or str(raw.get("mode") or "remote").strip().lower()
        if mode not in {"remote", "prefer", "local"}:
            mode = "remote"
        depot_bin = (
            (os.environ.get("DEPOT_BIN") or "").strip()
            or str(raw.get("depot_bin") or "depot").strip()
            or "depot"
        )
        ci_raw = raw.get("ci_repos") or list(DEFAULT_CI_REPOS)
        if isinstance(ci_raw, str):
            ci_repos = tuple(x.strip() for x in ci_raw.split(",") if x.strip())
        else:
            ci_repos = tuple(str(x) for x in ci_raw)
        build_fallback = raw.get("build_fallback")
        if build_fallback is None:
            build_fallback = True
        return OffloadConfig(
            mode=mode,
            org=str(raw.get("org") or os.environ.get("DEPOT_ORG") or DEFAULT_ORG),
            teikoku_repo=str(raw.get("teikoku_repo") or DEFAULT_TEIKOKU_REPO),
            elai_repo=str(raw.get("elai_repo") or DEFAULT_ELAI_REPO),
            project_id=str(
                raw.get("project_id")
                or os.environ.get("DEPOT_PROJECT_ID")
                or DEFAULT_PROJECT_ID
            ),
            workflow_teikoku=str(raw.get("workflow_teikoku") or DEFAULT_WORKFLOW),
            workflow_elai=str(raw.get("workflow_elai") or DEFAULT_ELAI_WORKFLOW),
            timeout_sec=int(raw.get("timeout_sec") or 3600),
            depot_bin=depot_bin,
            ci_repos=ci_repos,
            build_fallback=bool(build_fallback),
        )


def _cargo_subcommand(command: Sequence[str]) -> Optional[str]:
    arguments = list(command[1:])
    index = 0
    help_seen = False
    special_probe: Optional[str] = None
    subcommand: Optional[str] = None
    while index < len(arguments):
        argument = arguments[index]
        if argument == "--":
            break
        if argument.startswith("+") and len(argument) > 1:
            index += 1
            continue
        if argument in CARGO_GLOBAL_OPTIONS_WITH_VALUES:
            if (
                index + 1 >= len(arguments)
                or arguments[index + 1] == "--"
                or arguments[index + 1].startswith("-")
            ):
                return None
            index += 2
            continue
        if any(argument.startswith(option + "=") for option in CARGO_GLOBAL_OPTIONS_WITH_VALUES):
            index += 1
            continue
        if argument in CARGO_GLOBAL_FLAGS:
            if argument in {"--help", "-h"}:
                help_seen = True
            elif argument in {"--version", "-V"}:
                special_probe = "version"
            elif argument == "--list":
                special_probe = "list"
            index += 1
            continue
        if subcommand is not None:
            options = CARGO_SUBCOMMAND_OPTIONS_WITH_VALUES.get(subcommand, frozenset())
            flags = CARGO_SUBCOMMAND_FLAGS.get(subcommand, frozenset())
            if argument in options:
                if (
                    index + 1 >= len(arguments)
                    or arguments[index + 1] == "--"
                    or arguments[index + 1].startswith("-")
                ):
                    return None
                index += 2
                continue
            if any(argument.startswith(option + "=") for option in options):
                index += 1
                continue
            if argument in flags:
                index += 1
                continue
            return None
        if argument.startswith("-"):
            return None
        subcommand = argument
        index += 1
    return subcommand or ("help" if help_seen else special_probe)


def _cargo_help_probe(command: Sequence[str]) -> bool:
    if not command or Path(command[0]).name not in {"cargo", "cargo.exe"}:
        return False
    arguments = list(command[1:])
    before_separator = arguments[: arguments.index("--")] if "--" in arguments else arguments
    if not any(argument in {"-h", "--help"} for argument in before_separator):
        return False
    subcommand = _cargo_subcommand(command)
    return subcommand in HEAVY_CARGO_SUBCMDS or subcommand in SMALL_CARGO_SUBCMDS or subcommand == "help"





def resolve_profile(cwd: Path | str, need: str) -> Dict[str, Any]:
    """Load the shared profile registry and resolve one local project route."""
    if need not in {"verify", "artifact"}:
        raise DepotOffloadError("need must be verify or artifact")
    if not PROFILE_REGISTRY_PATH.is_file():
        raise DepotOffloadError(f"profile registry not found: {PROFILE_REGISTRY_PATH}")
    spec = importlib.util.spec_from_file_location(
        "teikoku_depot_profile_registry", PROFILE_REGISTRY_PATH
    )
    if spec is None or spec.loader is None:
        raise DepotOffloadError(f"cannot load profile registry: {PROFILE_REGISTRY_PATH}")
    module = importlib.util.module_from_spec(spec)
    try:
        spec.loader.exec_module(module)
        result = module.resolve_profile(cwd, need)
    except (OSError, ImportError, AttributeError, ValueError) as exc:
        raise DepotOffloadError(f"profile resolution failed: {exc}") from exc
    if not isinstance(result, dict):
        raise DepotOffloadError("profile resolution returned a non-object")
    return result


def _compact_profile_resolution(resolved: Mapping[str, Any]) -> Dict[str, Any]:
    profile = resolved.get("profile")
    route = resolved.get("route")
    profile_map = profile if isinstance(profile, Mapping) else {}
    route_map = route if isinstance(route, Mapping) else {}
    return {
        "source": resolved.get("source"),
        "profile_id": profile_map.get("id"),
        "resolved_root": resolved.get("resolved_root"),
        "need": resolved.get("need"),
        "status": route_map.get("status"),
        "adapter": route_map.get("adapter"),
    }


def _require_ready_verify_profile(cwd: Path, need: str) -> Dict[str, Any]:
    resolved = resolve_profile(cwd, need)
    route = resolved.get("route")
    route_map = route if isinstance(route, Mapping) else {}
    status = route_map.get("status")
    adapter = route_map.get("adapter")
    if need == "artifact":
        if status != "ready":
            raise DepotOffloadError(
                f"artifact profile route is {status or 'invalid'} for {resolved.get('resolved_root')}"
            )
        raise DepotOffloadError(
            "ougi-depot-offload is verification-only and cannot export named artifacts"
        )
    if status != "ready" or adapter != "ougi-depot-offload":
        raise DepotOffloadError(
            "verification profile route is not ready for ougi-depot-offload: "
            f"status={status!r} adapter={adapter!r}"
        )
    return resolved


def is_heavy_container_command(command: Sequence[str]) -> bool:
    if not command or Path(command[0]).name not in CONTAINER_COMMAND_NAMES:
        return False
    arguments = list(command[1:])
    return (
        arguments[:1] == ["build"]
        or arguments[:2] == ["buildx", "build"]
        or arguments[:2] == ["compose", "build"]
    )


def is_heavy_cargo_command(command: Sequence[str]) -> bool:
    if not command:
        return False
    name = Path(command[0]).name
    if name in {"rustc", "rustc.exe"}:
        return True
    if name not in {"cargo", "cargo.exe"}:
        return False
    if _cargo_help_probe(command):
        return False
    subcommand = _cargo_subcommand(command)
    if subcommand in HEAVY_CARGO_SUBCMDS:
        return True
    return subcommand not in SMALL_CARGO_SUBCMDS


def is_offload_gate(gate_name: str, command: Sequence[str]) -> bool:
    if is_heavy_container_command(command):
        return True
    if not is_heavy_cargo_command(command) and (
        _cargo_help_probe(command)
        or _cargo_subcommand(command) in SMALL_CARGO_SUBCMDS
    ):
        return False
    if gate_name in GATE_NAMES_OFFLOAD or gate_name.startswith("cargo_"):
        return True
    return is_heavy_cargo_command(command)


def classify_cargo_cwd(cwd: Path) -> str:
    """Return zenov | elai | teikoku | other for repo/workflow selection."""
    p = cwd.resolve()
    parts = {x.lower() for x in p.parts}
    s = str(p).replace("\\", "/").lower()
    if "zenov" in parts or "/ougi/zenov" in s or s.endswith("/zenov"):
        return "zenov"
    if ".elai_cc" in parts or "/elai/" in s or s.rstrip("/").endswith("/elai"):
        return "elai"
    if (p / "Cargo.toml").is_file():
        text = ""
        try:
            text = (p / "Cargo.toml").read_text(encoding="utf-8", errors="replace")[:2000]
        except OSError:
            pass
        if "zenov" in text.lower():
            return "zenov"
        if "elai" in text.lower():
            return "elai"
    # walk up for markers
    for parent in [p, *p.parents]:
        if parent.name == "zenov" and (parent / "Cargo.toml").is_file():
            return "zenov"
        if parent.name == ".elai_cc":
            return "elai"
        if parent.name == "elai" and (parent / ".elai_cc").is_dir():
            return "elai"
        if (parent / "depot.json").is_file() and (parent / "Ougi").is_dir():
            return "teikoku"
        if parent == parent.parent:
            break
    return "other"


def find_git_root(start: Path) -> Optional[Path]:
    cur = start.resolve()
    for parent in [cur, *cur.parents]:
        if (parent / ".git").exists():
            return parent
        if parent == parent.parent:
            break
    return None




def inspect_worktree(cwd: Path) -> Dict[str, Any]:
    """HEAD sha, dirty flag, main-head match, worktree abspath for route selection.

    Fail-closed: if git is unreadable, dirty=1 and at_main_head=False so the
    CI-on-main path (D-257 false green) is never selected by accident.
    """
    worktree = str(Path(cwd).resolve())
    git_root = find_git_root(Path(cwd))
    head = ""
    dirty = 1
    status_read = False
    at_main_head = False
    main_sha = ""
    if git_root is None:
        return {
            "head": "unknown",
            "dirty": 1,
            "status_read": False,
            "at_main_head": False,
            "worktree": worktree,
            "git_root": None,
            "main_sha": "",
        }
    try:
        head_p = subprocess.run(
            ["git", "-C", str(git_root), "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            timeout=30,
            errors="replace",
        )
        if head_p.returncode == 0:
            head = (head_p.stdout or "").strip()
    except (OSError, subprocess.TimeoutExpired):
        pass
    try:
        st = subprocess.run(
            ["git", "-C", str(git_root), "status", "--porcelain"],
            capture_output=True,
            text=True,
            timeout=120,
            errors="replace",
        )
        if st.returncode == 0:
            status_read = True
            dirty = 1 if (st.stdout or "").strip() else 0
    except (OSError, subprocess.TimeoutExpired):
        dirty = 1
    for ref in (
        "origin/main",
        "refs/remotes/origin/main",
        "main",
        "refs/heads/main",
    ):
        try:
            mp = subprocess.run(
                ["git", "-C", str(git_root), "rev-parse", ref],
                capture_output=True,
                text=True,
                timeout=30,
                errors="replace",
            )
            if mp.returncode == 0 and (mp.stdout or "").strip():
                main_sha = (mp.stdout or "").strip()
                break
        except (OSError, subprocess.TimeoutExpired):
            continue
    at_main_head = bool(head and main_sha and head == main_sha)
    return {
        "head": head or "unknown",
        "dirty": int(dirty),
        "status_read": status_read,
        "at_main_head": at_main_head,
        "worktree": worktree,
        "git_root": str(git_root),
        "main_sha": main_sha,
    }


def format_depot_provenance(
    path: str,
    head: str,
    dirty: int,
    worktree: str,
) -> str:
    """One-line provenance contract capturable by land gates and repros.\n\n    path is build|ci on success, or refuse when a remote check is declined.\n    """
    return (
        f"DEPOT_PROVENANCE path={path} head={head} "
        f"dirty={int(dirty)} worktree={worktree}"
    )


def emit_depot_provenance(path: str, state: Mapping[str, Any]) -> str:
    """Print and return the DEPOT_PROVENANCE line for a remote check."""
    dirty_raw = state.get("dirty")
    try:
        dirty = int(dirty_raw)
    except (TypeError, ValueError):
        dirty = 1
    if dirty not in {0, 1}:
        dirty = 1
    line = format_depot_provenance(
        path,
        str(state.get("head") or "unknown"),
        dirty,
        str(state.get("worktree") or ""),
    )
    print(line, flush=True)
    return line


def _valid_remote_state(state: Mapping[str, Any]) -> bool:
    head = state.get("head")
    worktree = state.get("worktree")
    git_root = state.get("git_root")
    return (
        isinstance(head, str)
        and VALID_HEAD_RE.fullmatch(head) is not None
        and isinstance(worktree, str)
        and os.path.isabs(worktree)
        and isinstance(git_root, str)
        and os.path.isabs(git_root)
        and Path(git_root).is_dir()
        and state.get("status_read") is True
        and type(state.get("dirty")) is int
        and state.get("dirty") in {0, 1}
    )


def _validate_remote_result(
    result: Mapping[str, Any], state: Mapping[str, Any], path: str
) -> None:
    expected = format_depot_provenance(
        path,
        str(state["head"]),
        int(state["dirty"]),
        str(state["worktree"]),
    )
    output_lines = [
        line for line in str(result.get("output") or "").splitlines()
        if "DEPOT_PROVENANCE" in line
    ]
    if (
        result.get("depot_provenance") != expected
        or not output_lines
        or any(line != expected for line in output_lines)
    ):
        refusal = emit_depot_provenance("refuse", state)
        raise DepotOffloadError(
            f"{refusal}\n"
            f"remote result provenance does not match requested {path} state"
        )


def _remote_provenance_or_refuse(path: str, state: Mapping[str, Any]) -> str:
    if path in {"build", "ci"} and not _valid_remote_state(state):
        provenance = emit_depot_provenance("refuse", state)
        raise DepotOffloadError(
            f"{provenance}\n"
            "refusing Depot execution because Git HEAD, Git root, or worktree "
            "provenance is not resolvable"
        )
    return emit_depot_provenance(path, state)


def ci_eligible(state: Mapping[str, Any]) -> bool:
    """True only for valid clean state whose HEAD equals a local main ref."""
    if not _valid_remote_state(state):
        return False
    dirty_raw = state.get("dirty")
    try:
        dirty = 1 if dirty_raw is None else int(dirty_raw)
    except (TypeError, ValueError):
        return False
    return dirty == 0 and bool(state.get("at_main_head"))


elai_ci_eligible = ci_eligible


def select_remote_path(kind: str, state: Mapping[str, Any], cfg: "OffloadConfig") -> str:
    """Return 'ci' | 'build' | 'refuse' for the remote route decision table.

    elai: clean main-HEAD -> ci (cached); dirty or non-main-HEAD -> build
    (or refuse when build_fallback is off). Other kinds prefer build when
    build_fallback is on, and use CI only when the tree is clean at main.
    """
    if kind == "elai":
        if ci_eligible(state):
            return "ci"
        if cfg.build_fallback:
            return "build"
        return "refuse"
    if cfg.build_fallback:
        return "build"
    if ci_eligible(state):
        return "ci"
    return "refuse"


def find_workflow(cfg: OffloadConfig, kind: str, git_root: Path) -> Path:
    if kind == "elai":
        rel = cfg.workflow_elai
    else:
        rel = cfg.workflow_teikoku
    candidates = [
        git_root / rel,
        git_root / ".depot" / "workflows" / Path(rel).name,
        Path("/home/researcher/teikoku") / rel,
        Path("/home/researcher/teikoku/Ougi") / rel,
        Path("/workspace/ELAI") / rel,
        Path("/home/researcher/teikoku/Ougi/.depot/workflows/conductor-cargo.yml"),
        Path("/home/researcher/teikoku/.depot/workflows/conductor-cargo.yml"),
        Path("/workspace/ELAI/.depot/workflows/elai-cargo.yml"),
    ]
    for c in candidates:
        if c.is_file():
            return c
    raise DepotOffloadError(f"depot workflow not found for kind={kind}: tried {rel}")


def preflight(cfg: Optional[OffloadConfig] = None) -> Dict[str, Any]:
    """Check CLI + login without printing secrets."""
    cfg = cfg or OffloadConfig.load()
    depot = shutil.which(cfg.depot_bin) or (
        str(Path.home() / ".depot" / "bin" / "depot")
        if (Path.home() / ".depot" / "bin" / "depot").is_file()
        else None
    )
    out: Dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "mode": cfg.mode,
        "depot_bin": depot,
        "ready": False,
        "org": cfg.org,
        "reasons": [],
    }
    if cfg.mode == "local":
        out["ready"] = False
        out["reasons"].append("mode=local")
        return out
    if not depot:
        out["reasons"].append("depot CLI not on PATH")
        return out
    try:
        ver = subprocess.run(
            [depot, "version"],
            capture_output=True,
            text=True,
            timeout=30,
            errors="replace",
        )
        out["version"] = (ver.stdout or ver.stderr or "").strip().splitlines()[:1]
    except (OSError, subprocess.TimeoutExpired) as exc:
        out["reasons"].append(f"depot version failed: {exc}")
        return out
    # projects list needs auth
    try:
        proj = subprocess.run(
            [depot, "projects", "list"],
            capture_output=True,
            text=True,
            timeout=45,
            errors="replace",
        )
        out["projects_exit"] = proj.returncode
        body = (proj.stdout or "") + (proj.stderr or "")
        if proj.returncode != 0:
            out["reasons"].append("depot projects list failed (not logged in?)")
            out["projects_excerpt"] = body[:300]
            return out
        if "example-project" in body or "default" in body.lower() or "project" in body.lower():
            out["projects_ok"] = True
        else:
            out["projects_ok"] = True  # non-empty success is enough
    except (OSError, subprocess.TimeoutExpired) as exc:
        out["reasons"].append(f"depot projects list error: {exc}")
        return out
    out["ready"] = True
    out["reasons"] = []
    return out


def _repo_for_kind(cfg: OffloadConfig, kind: str) -> str:
    if kind == "elai":
        return cfg.elai_repo
    return cfg.teikoku_repo


def _job_for_kind(kind: str, command: Sequence[str]) -> str:
    # conductor-cargo.yml jobs
    if kind == "elai":
        return "elai-hot-cargo"
    if kind == "zenov":
        if any(a == "test" for a in command):
            return "zenov-cargo"
        return "zenov-cargo"
    return "dynamic-cargo"


def write_request(
    git_root: Path,
    command: Sequence[str],
    cwd: Path,
    *,
    gate_name: str = "",
    request_id: str = "",
) -> Path:
    rel_cwd = os.path.relpath(str(cwd.resolve()), str(git_root.resolve()))
    if rel_cwd.startswith(".."):
        # cwd outside git root — use absolute path on runner only if under known trees
        rel_cwd = str(cwd)
    payload = {
        "schema_version": SCHEMA_VERSION,
        "request_id": request_id or uuid.uuid4().hex,
        "gate_name": gate_name,
        "cmd": list(command),
        "cwd": rel_cwd,
        "host": os.uname().nodename if hasattr(os, "uname") else "",
        "created_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    path = git_root / REQUEST_REL
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return path


def run_local(
    command: Sequence[str],
    cwd: Path,
    *,
    timeout_sec: int,
    env: Optional[Mapping[str, str]] = None,
) -> Dict[str, Any]:
    start = time.monotonic()
    try:
        proc = subprocess.run(
            list(command),
            cwd=str(cwd),
            capture_output=True,
            text=True,
            errors="replace",
            timeout=timeout_sec,
            env=dict(env) if env is not None else None,
        )
        output = (proc.stdout or "") + (
            "\n" if proc.stdout and proc.stderr else ""
        ) + (proc.stderr or "")
        return {
            "provider": "local",
            "exit_code": proc.returncode,
            "output": output,
            "duration_s": time.monotonic() - start,
            "command": list(command),
            "cwd": str(cwd),
        }
    except subprocess.TimeoutExpired as exc:
        output = (exc.stdout or "") + "\n" + (exc.stderr or "")
        return {
            "provider": "local",
            "exit_code": -1,
            "output": output or "local cargo timed out",
            "duration_s": float(timeout_sec),
            "command": list(command),
            "cwd": str(cwd),
            "timeout": True,
        }


def _shell_quote(parts: Sequence[str]) -> str:
    import shlex

    return " ".join(shlex.quote(str(p)) for p in parts)


def _depot_build_command_ok(command: Sequence[str]) -> bool:
    """True when command is non-empty cargo (or a heavy cargo subcommand).

    Empty sequences must not IndexError; callers emit path=refuse first (AC2).
    """
    if not command:
        return False
    if is_heavy_cargo_command(command):
        return True
    return Path(command[0]).name in {"cargo", "cargo.exe"}


def run_remote_depot_build(
    command: Sequence[str],
    cwd: Path,
    *,
    cfg: OffloadConfig,
    gate_name: str = "",
    timeout_sec: Optional[int] = None,
) -> Dict[str, Any]:
    """Run cargo on Depot *builders* (no GitHub Actions app required).

    Writes a throwaway Dockerfile into cwd, runs `depot build`, then removes it.
    Same product family as D-U3 live-finish (build ids on project example-project).
    Ships the LOCAL worktree as COPY context — the honest path for dirty trees.
    """
    # Inspect + provenance before command validation so empty/invalid input still
    # meets AC2 (every remote check emits DEPOT_PROVENANCE; never IndexError).
    state = inspect_worktree(cwd)
    if not _valid_remote_state(state):
        provenance = emit_depot_provenance("refuse", state)
        raise DepotOffloadError(
            f"{provenance}\n"
            "refusing depot build because Git HEAD, Git root, or worktree "
            "provenance is not resolvable"
        )
    if not _depot_build_command_ok(command):
        provenance = emit_depot_provenance("refuse", state)
        if is_heavy_container_command(command):
            message = (
                "known Docker or Podman heavy command requires a configured "
                "Depot container adapter"
            )
        else:
            message = f"depot build path refused empty or non-cargo command: {command!r}"
        raise DepotOffloadError(f"{provenance}\n{message}")
    _require_ready_verify_profile(cwd, "verify")
    provenance = emit_depot_provenance("build", state)
    depot = shutil.which(cfg.depot_bin) or cfg.depot_bin
    request_id = uuid.uuid4().hex[:12]
    docker_name = f"Dockerfile.conductor-offload-{request_id}"
    docker_path = cwd / docker_name
    dockerignore_path = cwd / ".dockerignore"
    dockerignore_backup: Optional[str] = None
    dockerignore_created = False
    cmd_s = _shell_quote(command)
    # Use official rust image; builders supply CPU/RAM off the owner host.
    # grok-build crates need protoc (xai-grok-tools-api build.rs). The stock
    # rust image has none. Install it before COPY so the apt layer stays cached.
    body = (
        "# generated by depot_offload.py — do not commit\n"
        "FROM rust:1-bookworm\n"
        "RUN apt-get update \\\n"
        "    && apt-get install -y --no-install-recommends protobuf-compiler ca-certificates pkg-config libssl-dev \\\n"
        "    && rm -rf /var/lib/apt/lists/*\n"
        "ENV PROTOC=/usr/bin/protoc\n"
        "WORKDIR /work\n"
        "COPY . /work\n"
        f"RUN {cmd_s}\n"
    )
    docker_path.write_text(body, encoding="utf-8")
    # Keep remote context small: exclude bulk non-source trees common under elai.
    ignore_extra = (
        "# conductor depot_offload ephemeral\n"
        ".git\n"
        "**/.git\n"
        "**/target\n"
        "**/models\n"
        "**/indexes\n"
        "**/.cache\n"
        "**/node_modules\n"
        "**/__pycache__\n"
        f"{docker_name}\n"
    )
    try:
        if dockerignore_path.is_file():
            dockerignore_backup = dockerignore_path.read_text(encoding="utf-8", errors="replace")
            if "conductor depot_offload ephemeral" not in dockerignore_backup:
                dockerignore_path.write_text(
                    dockerignore_backup.rstrip() + "\n" + ignore_extra, encoding="utf-8"
                )
        else:
            dockerignore_path.write_text(ignore_extra, encoding="utf-8")
            dockerignore_created = True
    except OSError:
        dockerignore_backup = None
        dockerignore_created = False
    argv = [
        depot,
        "build",
        "-f",
        docker_name,
        "--project",
        cfg.project_id,
        "--progress=plain",
        ".",
    ]
    tsec = int(timeout_sec or cfg.timeout_sec)
    start = time.monotonic()
    try:
        proc = subprocess.run(
            argv,
            cwd=str(cwd),
            capture_output=True,
            text=True,
            errors="replace",
            timeout=tsec,
        )
    except subprocess.TimeoutExpired as exc:
        # Exception body must carry DEPOT_PROVENANCE: execute_gate_via_offload
        # only surfaces str(exc) to land gates (AC2; never drop provenance on fail).
        raise DepotOffloadError(
            f"{provenance}\n"
            f"depot build timed out after {tsec}s: {gate_name or command}"
        ) from exc
    except FileNotFoundError as exc:
        raise DepotOffloadError(
            f"{provenance}\n"
            f"depot binary missing: {depot}"
        ) from exc
    finally:
        try:
            docker_path.unlink(missing_ok=True)  # type: ignore[call-arg]
        except TypeError:
            if docker_path.is_file():
                docker_path.unlink()
        except OSError:
            pass
        try:
            if dockerignore_created and dockerignore_path.is_file():
                dockerignore_path.unlink()
            elif dockerignore_backup is not None:
                dockerignore_path.write_text(dockerignore_backup, encoding="utf-8")
        except OSError:
            pass
    output = (proc.stdout or "") + (
        "\n" if proc.stdout and proc.stderr else ""
    ) + (proc.stderr or "")
    header = (
        f"{provenance}\n"
        f"[depot-offload] path=build request_id={request_id} project={cfg.project_id} "
        f"exit={proc.returncode} gate={gate_name}\n"
        f"[depot-offload] cmd={command} cwd={cwd}\n"
    )
    result = {
        "provider": "depot-build",
        "exit_code": proc.returncode,
        "output": header + output,
        "duration_s": time.monotonic() - start,
        "command": list(command),
        "cwd": str(cwd),
        "depot_argv": argv,
        "request_id": request_id,
        "kind": classify_cargo_cwd(cwd),
        "project_id": cfg.project_id,
        "depot_provenance": provenance,
        "worktree_state": dict(state),
    }
    _validate_remote_result(result, state, "build")
    return result


def run_remote_depot_ci(
    command: Sequence[str],
    cwd: Path,
    *,
    cfg: OffloadConfig,
    gate_name: str = "",
    timeout_sec: Optional[int] = None,
) -> Dict[str, Any]:
    """Dispatch cargo to Depot CI runners via `depot ci run` (patch-aware).

    HARD GUARD: dirty or non-main-HEAD worktrees are refused here so no
    caller can take the CI-on-main false-green path by accident.
    """
    kind = classify_cargo_cwd(cwd)
    state = inspect_worktree(cwd)
    if not _valid_remote_state(state):
        provenance = emit_depot_provenance("refuse", state)
        raise DepotOffloadError(
            f"{provenance}\n"
            "refusing depot CI because Git HEAD, Git root, or worktree "
            "provenance is not resolvable"
        )
    if not _depot_build_command_ok(command):
        provenance = emit_depot_provenance("refuse", state)
        if is_heavy_container_command(command):
            message = (
                "known Docker or Podman heavy command requires a configured "
                "Depot container adapter"
            )
        else:
            message = f"depot CI path refused empty or non-cargo command: {command!r}"
        raise DepotOffloadError(f"{provenance}\n{message}")
    if not ci_eligible(state):
        # Provenance first: every remote check must emit a line, including refusals.
        # path=refuse (not path=ci): an ineligible CI-on-main route is never labeled ci.
        provenance = emit_depot_provenance("refuse", state)
        raise DepotOffloadError(
            f"{provenance}\n"
            "refusing depot CI for dirty or non-main-HEAD worktree "
            f"(kind={kind} dirty={state.get('dirty')} at_main_head={state.get('at_main_head')} "
            f"head={state.get('head')})"
        )
    _require_ready_verify_profile(cwd, "verify")
    provenance = emit_depot_provenance("ci", state)
    git_root = Path(str(state["git_root"]))
    if not git_root.is_dir():
        refusal = emit_depot_provenance("refuse", state)
        raise DepotOffloadError(f"{refusal}\nGit root is not an absolute directory: {git_root}")

    # Prefer LIA CI for elai; teikoku CI is often unauthorized until GH App is linked.
    if kind == "elai":
        repo = cfg.elai_repo
        workflow = find_workflow(cfg, "elai", git_root)
        job = "elai-core-check"
    else:
        repo = cfg.teikoku_repo
        if repo not in cfg.ci_repos and cfg.build_fallback:
            raise DepotOffloadError(
                f"repo {repo} not in ci_repos {cfg.ci_repos}; use build fallback"
            )
        workflow = find_workflow(cfg, "teikoku", git_root)
        job = "dynamic-cargo"
        request_id = uuid.uuid4().hex[:12]
        write_request(
            git_root,
            command,
            cwd,
            gate_name=gate_name,
            request_id=request_id,
        )

    request_id = uuid.uuid4().hex[:12]
    if kind == "elai":
        # Fixed job; no request file required for elai-cargo.yml
        pass
    else:
        write_request(
            git_root,
            command,
            cwd,
            gate_name=gate_name,
            request_id=request_id,
        )
        workflow = find_workflow(cfg, "teikoku", git_root)
        job = "dynamic-cargo"

    depot = shutil.which(cfg.depot_bin) or cfg.depot_bin
    tsec = int(timeout_sec or cfg.timeout_sec)
    argv = [
        depot,
        "ci",
        "run",
        "--workflow",
        str(workflow),
        "--job",
        job,
        "--org",
        cfg.org,
        "--repo",
        repo,
        "--follow",
    ]
    start = time.monotonic()
    try:
        proc = subprocess.run(
            argv,
            cwd=str(git_root),
            capture_output=True,
            text=True,
            errors="replace",
            timeout=tsec,
        )
    except subprocess.TimeoutExpired as exc:
        # Exception body must carry DEPOT_PROVENANCE for gate adapter capture (AC2).
        raise DepotOffloadError(
            f"{provenance}\n"
            f"depot ci run timed out after {tsec}s: {gate_name or command}"
        ) from exc
    except FileNotFoundError as exc:
        raise DepotOffloadError(
            f"{provenance}\n"
            f"depot binary missing: {depot}"
        ) from exc

    output = (proc.stdout or "") + (
        "\n" if proc.stdout and proc.stderr else ""
    ) + (proc.stderr or "")
    unauthorized = "permission_denied" in output.lower() or "unauthorized" in output.lower()
    if unauthorized and cfg.build_fallback:
        raise DepotOffloadError(
            f"{provenance}\n"
            f"depot ci unauthorized for {repo}; build_fallback available"
        )
    header = (
        f"{provenance}\n"
        f"[depot-offload] path=ci request_id={request_id} kind={kind} job={job} "
        f"repo={repo} workflow={workflow} exit={proc.returncode}\n"
    )
    result = {
        "provider": "depot-ci",
        "exit_code": proc.returncode,
        "output": header + output,
        "duration_s": time.monotonic() - start,
        "command": list(command),
        "cwd": str(cwd),
        "depot_argv": argv,
        "request_id": request_id,
        "kind": kind,
        "job": job,
        "repo": repo,
        "workflow": str(workflow),
        "depot_provenance": provenance,
        "worktree_state": dict(state),
    }
    _validate_remote_result(result, state, "ci")
    return result


def run_remote(
    command: Sequence[str],
    cwd: Path,
    *,
    cfg: OffloadConfig,
    gate_name: str = "",
    timeout_sec: Optional[int] = None,
) -> Dict[str, Any]:
    """Pick CI only for clean main-HEAD worktrees, or use honest builders.

    Dirty or non-main-HEAD worktrees never take run_remote_depot_ci.
    They use run_remote_depot_build when enabled or an explicit refusal.
    """
    kind = classify_cargo_cwd(cwd)
    state = inspect_worktree(cwd)
    route = select_remote_path(kind, state, cfg)

    if kind == "elai":
        if route == "refuse":
            # Provenance before raise so refusal still meets AC2 (every remote check).
            provenance = emit_depot_provenance("refuse", state)
            raise DepotOffloadError(
                f"{provenance}\n"
                "elai worktree is dirty or not at origin/main HEAD; "
                "CI path refused (D-257) and build_fallback is disabled"
            )
        if route == "build":
            return run_remote_depot_build(
                command, cwd, cfg=cfg, gate_name=gate_name, timeout_sec=timeout_sec
            )
        # clean main-HEAD: CI preferred (faster, cached); fall back to build.
        try:
            return run_remote_depot_ci(
                command, cwd, cfg=cfg, gate_name=gate_name, timeout_sec=timeout_sec
            )
        except DepotOffloadError:
            if cfg.build_fallback:
                return run_remote_depot_build(
                    command, cwd, cfg=cfg, gate_name=gate_name, timeout_sec=timeout_sec
                )
            raise

    # zenov / other: prefer builders (no teikoku GH App required).
    if route == "refuse":
        provenance = emit_depot_provenance("refuse", state)
        raise DepotOffloadError(
            f"{provenance}\n"
            "worktree is dirty or not at a local main HEAD; "
            "CI path refused and build_fallback is disabled"
        )
    if route == "build":
        try:
            return run_remote_depot_build(
                command, cwd, cfg=cfg, gate_name=gate_name, timeout_sec=timeout_sec
            )
        except DepotOffloadError:
            if cfg.teikoku_repo in cfg.ci_repos and ci_eligible(state):
                return run_remote_depot_ci(
                    command, cwd, cfg=cfg, gate_name=gate_name, timeout_sec=timeout_sec
                )
            raise
    return run_remote_depot_ci(
        command, cwd, cfg=cfg, gate_name=gate_name, timeout_sec=timeout_sec
    )


def run_offloaded(
    command: Sequence[str],
    cwd: Path | str,
    *,
    gate_name: str = "",
    timeout_sec: Optional[int] = None,
    env: Optional[Mapping[str, str]] = None,
    cfg: Optional[OffloadConfig] = None,
    need: str = "verify",
) -> Dict[str, Any]:
    """Main entry: honor policy, return provider-tagged result dict."""
    if need not in {"verify", "artifact"}:
        raise DepotOffloadError("need must be verify or artifact")
    cfg = cfg or OffloadConfig.load()
    cwd_p = Path(cwd)
    tsec = int(timeout_sec or cfg.timeout_sec)
    if need == "artifact":
        _require_ready_verify_profile(cwd_p, need)
    if is_heavy_container_command(command):
        raise DepotOffloadError(
            "known Docker or Podman heavy command requires a configured "
            "Depot container adapter"
        )
    heavy = is_offload_gate(gate_name, command) or is_heavy_cargo_command(command)

    if not heavy:
        return run_local(command, cwd_p, timeout_sec=tsec, env=env)

    if cfg.mode == "local":
        result = run_local(command, cwd_p, timeout_sec=tsec, env=env)
        result["offload_mode"] = "local"
        return result

    profile_resolution = _require_ready_verify_profile(cwd_p, need)
    compact_profile = _compact_profile_resolution(profile_resolution)
    state = inspect_worktree(cwd_p)
    if not _valid_remote_state(state):
        provenance = emit_depot_provenance("refuse", state)
        raise DepotOffloadError(
            f"{provenance}\n"
            "refusing remote work because Git HEAD, Git root, or worktree "
            "provenance is not resolvable"
        )

    pf = preflight(cfg)
    if not pf.get("ready"):
        raise DepotOffloadError(
            f"TEIKOKU_DEPOT_OFFLOAD={cfg.mode} but Depot preflight failed: "
            + ", ".join(pf.get("reasons") or ["unknown"])
        )

    try:
        result = run_remote(
            command,
            cwd_p,
            cfg=cfg,
            gate_name=gate_name,
            timeout_sec=tsec,
        )
        provider = result.get("provider")
        remote_path = {"depot-build": "build", "depot-ci": "ci"}.get(provider)
        if remote_path is None:
            refusal = emit_depot_provenance("refuse", state)
            raise DepotOffloadError(f"{refusal}\nremote result has no Depot provider")
        _validate_remote_result(result, state, remote_path)
        result["offload_mode"] = cfg.mode
        result["preflight"] = {"ready": True, "org": cfg.org}
        result["profile"] = compact_profile
        return result
    except DepotOffloadError:
        raise


def execute_gate_via_offload(
    gate_name: str,
    command: Sequence[str],
    cwd: Path | str,
    *,
    timeout_sec: int,
    env: Optional[Mapping[str, str]] = None,
) -> Dict[str, Any]:
    """Adapter for gate_runner.execute_gate_command.

    Returns keys: gate_name, verdict-ready exit_code, output, duration_s, provider.
    """
    try:
        raw = run_offloaded(
            command,
            cwd,
            gate_name=gate_name,
            timeout_sec=timeout_sec,
            env=env,
        )
    except DepotOffloadError as exc:
        return {
            "gate_name": gate_name,
            "exit_code": 2,
            "output": f"[depot-offload] FAIL-CLOSED: {exc}",
            "duration_s": 0.0,
            "provider": "depot",
            "offload_error": str(exc),
        }
    return {
        "gate_name": gate_name,
        "exit_code": int(raw.get("exit_code") if raw.get("exit_code") is not None else 1),
        "output": str(raw.get("output") or ""),
        "duration_s": float(raw.get("duration_s") or 0.0),
        "provider": str(raw.get("provider") or "unknown"),
        "offload_mode": raw.get("offload_mode"),
        "request_id": raw.get("request_id"),
    }


def _cli(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(description="Teikoku Depot offload for cargo/gates")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_pf = sub.add_parser("preflight", help="Check Depot CLI + login")
    p_pf.add_argument("--json", action="store_true")

    p_cl = sub.add_parser("classify", help="Classify a cargo cwd")
    p_cl.add_argument("path", type=Path)

    p_run = sub.add_parser("run", help="Run a cargo command under offload policy")
    p_run.add_argument("--cwd", type=Path, default=None)
    p_run.add_argument("--gate-name", default="")
    p_run.add_argument("--timeout", type=int, default=None)
    p_run.add_argument("--need", choices=("verify", "artifact"), default="verify")
    p_run.add_argument(
        "cargo_argv",
        nargs=argparse.REMAINDER,
        help="Command after -- e.g. -- cargo check --workspace",
    )

    p_cfg = sub.add_parser("config-show", help="Print effective config")

    args = parser.parse_args(argv)
    cfg = OffloadConfig.load()

    if args.cmd == "preflight":
        pf = preflight(cfg)
        if args.json:
            print(json.dumps(pf, indent=2))
        else:
            print(
                f"ready={pf.get('ready')} mode={pf.get('mode')} "
                f"depot={pf.get('depot_bin')} reasons={pf.get('reasons')}"
            )
        return 0 if pf.get("ready") or cfg.mode == "local" else 1

    if args.cmd == "classify":
        print(classify_cargo_cwd(args.path))
        return 0

    if args.cmd == "config-show":
        print(
            json.dumps(
                {
                    "mode": cfg.mode,
                    "org": cfg.org,
                    "teikoku_repo": cfg.teikoku_repo,
                    "elai_repo": cfg.elai_repo,
                    "workflow_teikoku": cfg.workflow_teikoku,
                    "workflow_elai": cfg.workflow_elai,
                    "timeout_sec": cfg.timeout_sec,
                    "depot_bin": cfg.depot_bin,
                },
                indent=2,
            )
        )
        return 0

    if args.cmd == "run":
        cargo_argv = list(args.cargo_argv or [])
        if cargo_argv and cargo_argv[0] == "--":
            cargo_argv = cargo_argv[1:]
        if not cargo_argv:
            print("usage: depot_offload.py run -- cargo check --workspace", file=sys.stderr)
            return 2
        cwd = args.cwd or Path.cwd()
        try:
            result = run_offloaded(
                cargo_argv,
                cwd,
                gate_name=args.gate_name,
                timeout_sec=args.timeout,
                need=args.need,
                cfg=cfg,
            )
        except DepotOffloadError as exc:
            print(f"FAIL-CLOSED: {exc}", file=sys.stderr)
            return 2
        print(result.get("output") or "")
        print(
            f"\n[depot-offload] provider={result.get('provider')} "
            f"exit={result.get('exit_code')} mode={result.get('offload_mode')}",
            file=sys.stderr,
        )
        return int(result.get("exit_code") or 0)

    return 2


if __name__ == "__main__":
    raise SystemExit(_cli())
