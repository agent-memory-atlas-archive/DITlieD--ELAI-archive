from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import Any, Sequence

SCRIPT_ROOT = Path(__file__).resolve().parent
ELAI_ROOT = SCRIPT_ROOT.parent
DEFAULT_MANIFEST = ELAI_ROOT / "benchmarks" / "harness_trust" / "suites.json"
REF_PATTERN = re.compile(r"^[0-9a-f]{40}$")
SUITE_ID_PATTERN = re.compile(r"^[a-z0-9][a-z0-9-]+$")
CLONEABLE_STATUS = "cloneable"
PAPER_ONLY_STATUS = "paper_only"


class TrustbenchError(Exception):
    """Raised for invalid setup metadata or guarded runner failures."""


def path_inside(path: Path, root: Path) -> Path:
    resolved = path.expanduser().resolve()
    root_resolved = root.expanduser().resolve()
    try:
        resolved.relative_to(root_resolved)
    except ValueError as exc:
        raise TrustbenchError(f"path escapes project boundary: {path}") from exc
    return resolved


def project_path(relative_path: str) -> Path:
    return path_inside(ELAI_ROOT / relative_path, ELAI_ROOT)


def relative_project_path(path: Path) -> str:
    return path_inside(path, ELAI_ROOT).relative_to(ELAI_ROOT).as_posix()


def relative_lexical_project_path(path: Path) -> str:
    absolute = path if path.is_absolute() else Path.cwd() / path
    try:
        return absolute.absolute().relative_to(ELAI_ROOT).as_posix()
    except ValueError as exc:
        raise TrustbenchError(f"path is not inside project boundary: {path}") from exc


def read_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except OSError as exc:
        raise TrustbenchError(f"could not read {path}: {exc}") from exc
    except json.JSONDecodeError as exc:
        raise TrustbenchError(f"invalid JSON in {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise TrustbenchError(f"manifest must contain a JSON object: {path}")
    return value


def validate_manifest(manifest: dict[str, Any]) -> None:
    required_roots = ("checkout_root", "venv_root", "result_root", "suites")
    missing = [key for key in required_roots if key not in manifest]
    if missing:
        raise TrustbenchError(f"manifest missing keys: {', '.join(missing)}")
    if not isinstance(manifest["suites"], list) or not manifest["suites"]:
        raise TrustbenchError("manifest suites must be a non-empty list")
    seen: set[str] = set()
    for suite in manifest["suites"]:
        if not isinstance(suite, dict):
            raise TrustbenchError("every suite entry must be an object")
        suite_id = suite.get("id")
        if not isinstance(suite_id, str) or not SUITE_ID_PATTERN.fullmatch(suite_id):
            raise TrustbenchError(f"invalid suite id: {suite_id!r}")
        if suite_id in seen:
            raise TrustbenchError(f"duplicate suite id: {suite_id}")
        seen.add(suite_id)
        status = suite.get("status")
        if status not in (CLONEABLE_STATUS, PAPER_ONLY_STATUS):
            raise TrustbenchError(f"invalid status for {suite_id}: {status!r}")
        source = suite.get("source")
        if not isinstance(source, str) or not source.startswith("https://"):
            raise TrustbenchError(f"invalid source for {suite_id}")
        ref = suite.get("ref")
        if status == CLONEABLE_STATUS:
            if not isinstance(ref, str) or not REF_PATTERN.fullmatch(ref):
                raise TrustbenchError(f"invalid pinned ref for {suite_id}")
            install = suite.get("install")
            if not isinstance(install, dict):
                raise TrustbenchError(f"cloneable suite has no install metadata: {suite_id}")
        elif ref is not None:
            raise TrustbenchError(f"paper-only suite must not have a ref: {suite_id}")


def load_manifest(path: Path = DEFAULT_MANIFEST) -> dict[str, Any]:
    manifest = read_json(path.expanduser().resolve())
    validate_manifest(manifest)
    return manifest


def suites_by_id(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    return {suite["id"]: suite for suite in manifest["suites"]}


def suite_paths(manifest: dict[str, Any], suite_id: str) -> tuple[Path, Path, Path]:
    checkout = project_path(f"{manifest['checkout_root']}/{suite_id}")
    venv = project_path(f"{manifest['venv_root']}/{suite_id}")
    result = project_path(f"{manifest['result_root']}/{suite_id}")
    return checkout, venv, result


def command_output(argv: Sequence[str], cwd: Path | None = None) -> str:
    process = subprocess.run(
        [str(arg) for arg in argv],
        cwd=str(cwd) if cwd else None,
        check=False,
        capture_output=True,
        text=True,
    )
    if process.returncode != 0:
        detail = process.stderr.strip() or process.stdout.strip()
        raise TrustbenchError(
            f"command failed ({process.returncode}): {shlex.join([str(arg) for arg in argv])}"
            + (f"\n{detail}" if detail else "")
        )
    return process.stdout.strip()


def run_streaming(argv: Sequence[str], cwd: Path) -> None:
    process = subprocess.run([str(arg) for arg in argv], cwd=str(cwd), check=False)
    if process.returncode != 0:
        raise TrustbenchError(
            f"command failed ({process.returncode}): {shlex.join([str(arg) for arg in argv])}"
        )


def normalize_remote(url: str) -> str:
    normalized = url.strip().rstrip("/")
    if normalized.endswith(".git"):
        normalized = normalized[:-4]
    return normalized


def inspect_checkout(suite: dict[str, Any], checkout: Path) -> dict[str, Any]:
    expected_source = normalize_remote(suite["source"])
    result: dict[str, Any] = {
        "path": relative_project_path(checkout),
        "exists": checkout.exists(),
        "valid": False,
        "remote": None,
        "head": None,
        "dirty": None,
        "reason": None,
    }
    if not checkout.exists():
        result["reason"] = "missing_checkout"
        return result
    if not checkout.is_dir() or not (checkout / ".git").exists():
        result["reason"] = "not_a_git_checkout"
        return result
    try:
        remote = command_output(["git", "-C", str(checkout), "remote", "get-url", "origin"])
        head = command_output(["git", "-C", str(checkout), "rev-parse", "HEAD"])
        dirty = command_output(["git", "-C", str(checkout), "status", "--porcelain"])
    except TrustbenchError as exc:
        result["reason"] = str(exc)
        return result
    result.update(
        {
            "remote": remote,
            "head": head,
            "dirty": bool(dirty),
        }
    )
    if normalize_remote(remote) != expected_source:
        result["reason"] = "remote_mismatch"
    elif head != suite["ref"]:
        result["reason"] = "ref_mismatch"
    elif dirty:
        result["reason"] = "dirty_checkout"
    else:
        result["valid"] = True
        result["reason"] = "ready"
    return result


def ensure_checkout(suite: dict[str, Any], checkout: Path) -> None:
    state = inspect_checkout(suite, checkout)
    if state["valid"]:
        return
    if state["exists"]:
        raise TrustbenchError(
            f"refusing to reuse {checkout}: {state['reason']}; inspect or move it manually"
        )
    checkout.parent.mkdir(parents=True, exist_ok=True)
    run_streaming(["git", "clone", "--depth", "1", suite["source"], str(checkout)], ELAI_ROOT)
    run_streaming(["git", "-C", str(checkout), "checkout", "--detach", suite["ref"]], ELAI_ROOT)
    final_state = inspect_checkout(suite, checkout)
    if not final_state["valid"]:
        raise TrustbenchError(f"cloned checkout failed validation: {final_state['reason']}")


def version_tuple(executable: str | Path) -> tuple[int, int, int]:
    output = command_output(
        [str(executable), "-c", "import sys; print('.'.join(str(v) for v in sys.version_info[:3]))"]
    )
    parts = output.split(".")
    if len(parts) != 3 or not all(part.isdigit() for part in parts):
        raise TrustbenchError(f"could not parse Python version from {executable}: {output}")
    return tuple(int(part) for part in parts)  # type: ignore[return-value]


def resolve_python(requested: str | None) -> str:
    candidates = [requested] if requested else ["python3.12", "python3.11", "python3", sys.executable]
    for candidate in candidates:
        if not candidate:
            continue
        resolved = shutil.which(candidate) or (candidate if Path(candidate).exists() else None)
        if not resolved:
            continue
        try:
            version = version_tuple(resolved)
        except TrustbenchError:
            continue
        if version >= (3, 11, 0):
            return str(resolved)
    raise TrustbenchError("no installed Python 3.11+ executable was found")


def venv_python(venv: Path) -> Path:
    return venv / "bin" / "python"


def ensure_venv(suite: dict[str, Any], checkout: Path, venv: Path, python_executable: str) -> Path:
    target = venv_python(venv)
    if not target.exists():
        venv.parent.mkdir(parents=True, exist_ok=True)
        run_streaming([python_executable, "-m", "venv", str(venv)], ELAI_ROOT)
    if not target.exists():
        raise TrustbenchError(f"virtualenv creation did not produce {target}")
    if version_tuple(target) < (3, 11, 0):
        raise TrustbenchError(f"virtualenv Python is older than 3.11: {target}")
    install = suite["install"]
    base = [str(target), "-m", "pip", "install", "--disable-pip-version-check", "--no-input"]
    if install["kind"] == "editable":
        run_streaming(base + list(install["args"]), checkout)
    elif install["kind"] == "requirements":
        requirements = checkout / install["path"]
        if not requirements.is_file():
            raise TrustbenchError(f"requirements file missing for {suite['id']}: {requirements}")
        run_streaming(base + ["-r", str(requirements)], checkout)
    else:
        raise TrustbenchError(f"unsupported install kind for {suite['id']}: {install['kind']}")
    for optional in suite.get("optional_installs", []):
        optional_path = checkout / optional["relative_path"]
        if optional_path.exists():
            run_streaming(base + list(optional["args"]), checkout)
    return target


def import_check(python: Path, modules: Sequence[str]) -> tuple[bool, str | None]:
    if not modules:
        return True, None
    code = "import importlib; " + "; ".join(
        f"importlib.import_module({module!r})" for module in modules
    )
    process = subprocess.run(
        [str(python), "-c", code],
        check=False,
        capture_output=True,
        text=True,
    )
    if process.returncode == 0:
        return True, None
    detail = process.stderr.strip() or process.stdout.strip()
    return False, detail[-1000:] if detail else "import check failed"


def suite_doctor(manifest: dict[str, Any], suite: dict[str, Any]) -> dict[str, Any]:
    checkout, venv, _ = suite_paths(manifest, suite["id"])
    if suite["status"] == PAPER_ONLY_STATUS:
        return {
            "suite": suite["id"],
            "status": PAPER_ONLY_STATUS,
            "source": suite["source"],
            "ready": False,
            "reason": suite["adapter"]["notes"],
        }
    checkout_state = inspect_checkout(suite, checkout)
    result: dict[str, Any] = {
        "suite": suite["id"],
        "status": suite["status"],
        "checkout": checkout_state,
        "venv": {
            "path": relative_project_path(venv),
            "exists": venv.exists(),
            "python": None,
            "version": None,
            "imports_ok": None,
            "import_error": None,
        },
        "docker_cli": bool(shutil.which("docker")),
        "adapter_status": "adapter_required",
        "ready": False,
    }
    python = venv_python(venv)
    if python.exists():
        result["venv"]["python"] = relative_lexical_project_path(python)
        try:
            result["venv"]["version"] = ".".join(str(v) for v in version_tuple(python))
            imports_ok, import_error = import_check(python, suite.get("imports", []))
            result["venv"]["imports_ok"] = imports_ok
            result["venv"]["import_error"] = import_error
        except TrustbenchError as exc:
            result["venv"]["import_error"] = str(exc)
    if checkout_state["valid"] and python.exists() and result["venv"]["imports_ok"]:
        result["ready"] = True
    elif not checkout_state["valid"]:
        result["reason"] = checkout_state["reason"]
    elif not python.exists():
        result["reason"] = "missing_venv"
    else:
        result["reason"] = "imports_unavailable"
    if "docker_for_sandbox_settings" in suite.get("capabilities", []) and not result["docker_cli"]:
        result["capability_warning"] = "Docker CLI is missing for sandbox-backed settings"
    return result


def parse_adapter_commands(values: Sequence[str]) -> dict[str, list[str]]:
    commands: dict[str, list[str]] = {}
    for value in values:
        if "=" not in value:
            raise TrustbenchError("--adapter-command must use SUITE=COMMAND syntax")
        suite_id, raw_command = value.split("=", 1)
        if not SUITE_ID_PATTERN.fullmatch(suite_id) or not raw_command.strip():
            raise TrustbenchError(f"invalid adapter command: {value}")
        try:
            command = shlex.split(raw_command)
        except ValueError as exc:
            raise TrustbenchError(f"invalid adapter command quoting: {value}") from exc
        if not command:
            raise TrustbenchError(f"empty adapter command: {value}")
        commands[suite_id] = command
    return commands


def substitute_command(command: Sequence[str], context: dict[str, str]) -> list[str]:
    result = []
    for argument in command:
        value = argument
        for key, replacement in context.items():
            value = value.replace("{" + key + "}", replacement)
        result.append(value)
    return result


def redact_argument(argument: str, redact_next: bool) -> tuple[str, bool]:
    lowered = argument.lower()
    if redact_next:
        return "<redacted>", False
    if any(
        token in lowered
        for token in ("api_key", "api-key", "apikey", "token", "secret", "password")
    ):
        if "=" in argument:
            return argument.split("=", 1)[0] + "=<redacted>", False
        return argument, True
    if "_key=" in lowered or "_token=" in lowered or "secret=" in lowered:
        return argument.split("=", 1)[0] + "=<redacted>", False
    return argument, False


def redact_command(command: Sequence[str]) -> list[str]:
    redacted: list[str] = []
    redact_next = False
    for argument in command:
        safe, redact_next = redact_argument(argument, redact_next)
        redacted.append(safe)
    return redacted


def numeric_score(value: Any) -> int | float | None:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        return None
    return value


def normalize_artifact(path: Path, suite_id: str, metadata: dict[str, Any]) -> list[dict[str, Any]]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as exc:
        raise TrustbenchError(f"could not read adapter artifact {path}: {exc}") from exc
    rows: list[dict[str, Any]] = []
    for index, line in enumerate(lines, start=1):
        if not line.strip():
            continue
        try:
            value = json.loads(line)
        except json.JSONDecodeError as exc:
            raise TrustbenchError(f"malformed JSONL at {path}:{index}: {exc}") from exc
        if not isinstance(value, dict):
            raise TrustbenchError(f"adapter row at {path}:{index} is not a JSON object")
        rows.append(
            {
                "suite": suite_id,
                "row_index": index,
                "status": value.get("status", "completed"),
                "score": numeric_score(value.get("score")),
                "metadata": metadata,
                "raw": value,
            }
        )
    if not rows:
        raise TrustbenchError(f"adapter artifact is empty: {path}")
    return rows


def write_json(path: Path, value: Any) -> None:
    path_inside(path, ELAI_ROOT)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def run_id(dry_run: bool) -> str:
    if dry_run:
        return "dry-run"
    return time.strftime("%Y%m%dT%H%M%SZ", time.gmtime()) + f"-{time.time_ns() % 1_000_000:06d}"


def select_suites(
    manifest: dict[str, Any], requested: Sequence[str], include_all: bool
) -> list[dict[str, Any]]:
    by_id = suites_by_id(manifest)
    if include_all and requested:
        raise TrustbenchError("use --all or --suite, not both")
    if include_all:
        return list(manifest["suites"])
    if not requested:
        raise TrustbenchError("run requires --all or at least one --suite")
    unknown = [suite_id for suite_id in requested if suite_id not in by_id]
    if unknown:
        raise TrustbenchError(f"unknown suite: {', '.join(unknown)}")
    return [by_id[suite_id] for suite_id in requested]


def run_one_suite(
    manifest: dict[str, Any],
    suite: dict[str, Any],
    adapter_commands: dict[str, list[str]],
    run_root: Path,
    harness: str,
    model: str,
    seed: int,
    dry_run: bool,
    timeout: int,
) -> dict[str, Any]:
    suite_id = suite["id"]
    checkout, venv, _ = suite_paths(manifest, suite_id)
    base: dict[str, Any] = {
        "suite": suite_id,
        "name": suite["name"],
        "status": suite["status"],
        "harness": harness,
        "model": model,
        "seed": seed,
        "checkout": relative_project_path(checkout),
        "venv": relative_project_path(venv),
        "workspace": None,
        "raw_artifact": None,
        "normalized_artifact": None,
        "stdout_log": None,
        "stderr_log": None,
        "command": [],
        "message": None,
        "rows": 0,
    }
    if suite["status"] == PAPER_ONLY_STATUS:
        base["message"] = suite["adapter"]["notes"]
        return base
    adapter = adapter_commands.get(suite_id)
    if adapter is None:
        base["status"] = "adapter_required"
        base["message"] = suite["adapter"]["notes"]
        return base
    suite_root = run_root / suite_id
    workspace = path_inside(suite_root / "workspace", ELAI_ROOT)
    artifact = path_inside(suite_root / "adapter.jsonl", ELAI_ROOT)
    normalized = path_inside(suite_root / "normalized.jsonl", ELAI_ROOT)
    stdout_log = path_inside(suite_root / "stdout.log", ELAI_ROOT)
    stderr_log = path_inside(suite_root / "stderr.log", ELAI_ROOT)
    context = {
        "harness": harness,
        "model": model,
        "seed": str(seed),
        "suite": suite_id,
        "suite_root": str(checkout),
        "workspace": str(workspace),
        "output": str(artifact),
        "artifact": str(artifact),
    }
    command = substitute_command(adapter, context)
    base.update(
        {
            "workspace": relative_project_path(workspace),
            "raw_artifact": relative_project_path(artifact),
            "normalized_artifact": relative_project_path(normalized),
            "stdout_log": relative_project_path(stdout_log),
            "stderr_log": relative_project_path(stderr_log),
            "command": redact_command(command),
        }
    )
    if dry_run:
        base["status"] = "dry_run_ready"
        base["message"] = "command constructed; pass --execute after bootstrap to run it"
        return base
    if not inspect_checkout(suite, checkout)["valid"]:
        base["status"] = "capability_missing"
        base["message"] = "bootstrap the suite before execution"
        return base
    if not venv_python(venv).exists():
        base["status"] = "capability_missing"
        base["message"] = "suite virtualenv is missing; bootstrap the suite before execution"
        return base
    workspace.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env.update(
        {
            "HARNESS_TRUST_SUITE": suite_id,
            "HARNESS_TRUST_HARNESS": harness,
            "HARNESS_TRUST_MODEL": model,
            "HARNESS_TRUST_SEED": str(seed),
            "HARNESS_TRUST_WORKSPACE": str(workspace),
            "HARNESS_TRUST_OUTPUT": str(artifact),
            "PATH": os.pathsep.join([str(venv / "bin"), env.get("PATH", "")]),
        }
    )
    try:
        with stdout_log.open("w", encoding="utf-8") as stdout, stderr_log.open(
            "w", encoding="utf-8"
        ) as stderr:
            process = subprocess.run(
                command,
                cwd=str(checkout),
                env=env,
                stdout=stdout,
                stderr=stderr,
                check=False,
                timeout=timeout,
                text=True,
            )
    except subprocess.TimeoutExpired:
        base["status"] = "failed"
        base["message"] = f"adapter exceeded timeout of {timeout} seconds"
        return base
    if process.returncode != 0:
        base["status"] = "failed"
        base["message"] = f"adapter exited with status {process.returncode}"
        return base
    try:
        rows = normalize_artifact(
            artifact,
            suite_id,
            {
                "harness": harness,
                "model": model,
                "seed": seed,
            },
        )
        normalized.write_text(
            "".join(json.dumps(row, sort_keys=True) + "\n" for row in rows), encoding="utf-8"
        )
    except TrustbenchError as exc:
        base["status"] = "failed"
        base["message"] = str(exc)
        return base
    base["status"] = "completed"
    base["rows"] = len(rows)
    base["message"] = "adapter completed; aggregate scoring remains disabled"
    return base


def command_run(args: argparse.Namespace, manifest: dict[str, Any]) -> int:
    if not args.dry_run and not args.execute:
        raise TrustbenchError("run requires --dry-run or --execute")
    selected = select_suites(manifest, args.suite, args.all)
    commands = parse_adapter_commands(args.adapter_command)
    known = suites_by_id(manifest)
    unknown_commands = sorted(set(commands) - set(known))
    if unknown_commands:
        raise TrustbenchError(f"adapter command names unknown suite: {', '.join(unknown_commands)}")
    output_base = (
        Path(args.output_dir).expanduser()
        if Path(args.output_dir).is_absolute()
        else ELAI_ROOT / args.output_dir
    )
    output_base = path_inside(output_base, ELAI_ROOT)
    current_run_id = run_id(args.dry_run)
    run_root = path_inside(output_base / current_run_id, ELAI_ROOT)
    run_root.mkdir(parents=True, exist_ok=True)
    suite_reports = [
        run_one_suite(
            manifest,
            suite,
            commands,
            run_root,
            args.harness,
            args.model,
            args.seed,
            args.dry_run,
            args.timeout,
        )
        for suite in selected
    ]
    report = {
        "schema_version": 1,
        "benchmark": manifest["name"],
        "run_id": current_run_id,
        "dry_run": args.dry_run,
        "fixed": {
            "harness": args.harness,
            "model": args.model,
            "seed": args.seed,
        },
        "suites": suite_reports,
        "aggregate": {
            "enabled": False,
            "score": None,
            "reason": manifest["aggregate"]["reason"],
        },
    }
    report_path = run_root / "report.json"
    write_json(report_path, report)
    print(json.dumps({"report": relative_project_path(report_path), **report}, indent=2, sort_keys=True))
    return 0


def command_report(args: argparse.Namespace) -> int:
    report_path = Path(args.path).expanduser().resolve()
    report = read_json(report_path)
    if report.get("schema_version") != 1 or not isinstance(report.get("suites"), list):
        raise TrustbenchError(f"unsupported trustbench report: {report_path}")
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
        return 0
    print(f"benchmark: {report.get('benchmark')}")
    print(f"run_id: {report.get('run_id')}")
    print(f"harness: {report.get('fixed', {}).get('harness')}")
    print(f"model: {report.get('fixed', {}).get('model')}")
    print("suites:")
    for suite in report["suites"]:
        print(f"  {suite.get('suite')}: {suite.get('status')} ({suite.get('message') or 'no message'})")
    print(f"aggregate: disabled ({report.get('aggregate', {}).get('reason')})")
    return 0


def command_list(args: argparse.Namespace, manifest: dict[str, Any]) -> int:
    if args.json:
        print(json.dumps(manifest["suites"], indent=2, sort_keys=True))
        return 0
    print("id\tstatus\tref\tsource")
    for suite in manifest["suites"]:
        print(
            "\t".join(
                [
                    suite["id"],
                    suite["status"],
                    suite["ref"] or "-",
                    suite["source"],
                ]
            )
        )
    return 0


def command_doctor(args: argparse.Namespace, manifest: dict[str, Any]) -> int:
    report = {
        "python": {
            "executable": sys.executable,
            "version": ".".join(str(value) for value in sys.version_info[:3]),
            "supported": sys.version_info >= (3, 11),
        },
        "git": bool(shutil.which("git")),
        "docker_cli": bool(shutil.which("docker")),
        "suites": [suite_doctor(manifest, suite) for suite in manifest["suites"]],
    }
    report["ready"] = bool(report["python"]["supported"] and report["git"])
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if report["ready"] else 1


def command_bootstrap(args: argparse.Namespace, manifest: dict[str, Any]) -> int:
    selected = select_suites(manifest, args.suite, args.all)
    python_executable = resolve_python(args.python)
    print(f"bootstrap Python: {python_executable}")
    for suite in selected:
        if suite["status"] == PAPER_ONLY_STATUS:
            print(f"{suite['id']}: paper_only, no checkout or environment")
            continue
        checkout, venv, _ = suite_paths(manifest, suite["id"])
        print(f"{suite['id']}: validating checkout")
        ensure_checkout(suite, checkout)
        print(f"{suite['id']}: creating environment and installing declared dependencies")
        ensure_venv(suite, checkout, venv, python_executable)
        print(f"{suite['id']}: ready at {relative_project_path(checkout)}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="trustbench")
    subparsers = parser.add_subparsers(dest="command", required=True)

    list_parser = subparsers.add_parser("list")
    list_parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST))
    list_parser.add_argument("--json", action="store_true")

    doctor_parser = subparsers.add_parser("doctor")
    doctor_parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST))

    bootstrap_parser = subparsers.add_parser("bootstrap")
    bootstrap_parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST))
    bootstrap_parser.add_argument("--suite", action="append", default=[])
    bootstrap_parser.add_argument("--all", action="store_true")
    bootstrap_parser.add_argument("--python")

    run_parser = subparsers.add_parser("run")
    run_parser.add_argument("--manifest", default=str(DEFAULT_MANIFEST))
    run_parser.add_argument("--harness", required=True)
    run_parser.add_argument("--model", required=True)
    run_parser.add_argument("--seed", type=int, default=0)
    run_parser.add_argument("--suite", action="append", default=[])
    run_parser.add_argument("--all", action="store_true")
    run_parser.add_argument("--adapter-command", action="append", default=[])
    run_parser.add_argument(
        "--output-dir",
        default="benchmarks/harness_trust/results",
    )
    run_parser.add_argument("--timeout", type=int, default=3600)
    run_parser.add_argument("--dry-run", action="store_true")
    run_parser.add_argument("--execute", action="store_true")

    report_parser = subparsers.add_parser("report")
    report_parser.add_argument("path")
    report_parser.add_argument("--json", action="store_true")

    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        manifest = load_manifest(Path(args.manifest)) if hasattr(args, "manifest") else None
        if args.command == "list":
            if manifest is None:
                raise TrustbenchError("list requires a manifest")
            return command_list(args, manifest)
        if args.command == "doctor":
            if manifest is None:
                raise TrustbenchError("doctor requires a manifest")
            return command_doctor(args, manifest)
        if args.command == "bootstrap":
            if manifest is None:
                raise TrustbenchError("bootstrap requires a manifest")
            return command_bootstrap(args, manifest)
        if args.command == "run":
            if manifest is None:
                raise TrustbenchError("run requires a manifest")
            return command_run(args, manifest)
        if args.command == "report":
            return command_report(args)
        raise TrustbenchError(f"unsupported command: {args.command}")
    except TrustbenchError as exc:
        print(f"trustbench: error: {exc}", file=sys.stderr)
        return 2
    except KeyboardInterrupt:
        print("trustbench: interrupted", file=sys.stderr)
        return 130


if __name__ == "__main__":
    raise SystemExit(main())
