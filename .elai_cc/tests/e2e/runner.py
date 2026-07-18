"""ELAI e2e test runner: drives the REAL compiled binary and asserts on off-agent SSB evidence.

Each scenario runs in a FRESH isolated ELAI home (tempdir) so runs never share state.
The runner shells .elai_cc/target/debug/elai, captures stdout+stderr+exit code,
opens the run's ssb.db read-only, and hands the results to the scenario for assertion.

Invocation:
    python .elai_cc/tests/e2e/runner.py
    python .elai_cc/tests/e2e/runner.py --scenario s1_prebind
    python .elai_cc/tests/e2e/runner.py --binary /path/to/elai

Per-scenario timeout: @register(..., timeout=<seconds>) overrides the 60s default.

Exit code: 0 if all scenarios PASS, 1 if any FAIL.
"""

from __future__ import annotations

import argparse
import importlib
import os
import pkgutil
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import traceback
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable, Optional

# e2e package root
E2E_ROOT = Path(__file__).resolve().parent
SCENARIOS_PKG = "scenarios"

# Ensure the e2e root is on sys.path so scenario modules can `from runner import ...`
# and `import msgpack_min` when runner.py is invoked as a script.
if str(E2E_ROOT) not in sys.path:
    sys.path.insert(0, str(E2E_ROOT))

# When runner.py is invoked as a script (__main__), register it as the `runner`
# module so scenario files that do `from runner import register` land in the
# same _REGISTRY the main() loop reads.
if __name__ == "__main__" and "runner" not in sys.modules:
    sys.modules["runner"] = sys.modules["__main__"]

# default binary: look for the elai repo root by walking up to find .elai_cc/crates
def _find_default_binary() -> Path:
    here = E2E_ROOT
    for parent in [here, *here.parents]:
        if (parent / ".elai_cc" / "crates").is_dir():
            candidate = parent / ".elai_cc" / "target" / "debug" / "elai"
            if candidate.exists():
                return candidate
    return here.parents[2] / "target" / "debug" / "elai"

DEFAULT_BINARY = _find_default_binary()


@dataclass
class RunResult:
    """What a scenario sees from driving the binary."""
    argv: list[str]
    cwd: Path
    home: Path
    stdout: str
    stderr: str
    exit_code: int
    ssb_db_path: Path

    def ssb_rows(self, action_type: Optional[str] = None) -> list[dict[str, Any]]:
        """Read rows from this run's ssb.db. Returns list of dicts with decoded payload."""
        if not self.ssb_db_path.exists():
            return []
        conn = sqlite3.connect(f"file:{self.ssb_db_path}?mode=ro", uri=True)
        try:
            conn.row_factory = sqlite3.Row
            cur = conn.cursor()
            if action_type is not None:
                cur.execute(
                    "SELECT seq, project_id, session_id, agent_id, superstep, "
                    "action_type, payload, created_at FROM ssb WHERE action_type = ? "
                    "ORDER BY seq",
                    (action_type,),
                )
            else:
                cur.execute(
                    "SELECT seq, project_id, session_id, agent_id, superstep, "
                    "action_type, payload, created_at FROM ssb ORDER BY seq"
                )
            rows = []
            for r in cur.fetchall():
                rows.append({
                    "seq": r["seq"],
                    "project_id": r["project_id"],
                    "session_id": r["session_id"],
                    "agent_id": r["agent_id"],
                    "superstep": r["superstep"],
                    "action_type": r["action_type"],
                    "payload": _decode_payload(r["payload"]),
                    "created_at": r["created_at"],
                })
            return rows
        finally:
            conn.close()


def _decode_payload(blob: bytes) -> Any:
    from msgpack_min import decode as mp_decode
    return mp_decode(blob)


@dataclass
class ScenarioResult:
    name: str
    passed: bool
    detail: str
    evidence: str = ""


@dataclass
class Scenario:
    name: str
    fn: Callable[[RunResult], None]
    argv: list[str]
    env_extra: dict[str, str] = field(default_factory=dict)
    timeout: int = 60


_REGISTRY: list[Scenario] = []


def register(name: str, argv: list[str], env_extra: Optional[dict] = None, timeout: int = 60) -> Callable:
    """Decorator: register a scenario fn the runner will call with a RunResult."""
    def deco(fn: Callable[[RunResult], None]) -> Callable[[RunResult], None]:
        _REGISTRY.append(Scenario(
            name=name,
            fn=fn,
            argv=list(argv),
            env_extra=dict(env_extra or {}),
            timeout=timeout,
        ))
        return fn
    return deco


def _discover_scenarios() -> None:
    """Import all modules under scenarios/ so @register fires."""
    pkg_init = E2E_ROOT / SCENARIOS_PKG / "__init__.py"
    pkg_init.touch(exist_ok=True)
    # Ensure E2E_ROOT is on sys.path for the scenarios package import
    if str(E2E_ROOT) not in sys.path:
        sys.path.insert(0, str(E2E_ROOT))
    pkg = importlib.import_module(SCENARIOS_PKG)
    for mod_info in pkgutil.iter_modules(pkg.__path__):
        if mod_info.name.startswith("_"):
            continue
        try:
            importlib.import_module(f"{SCENARIOS_PKG}.{mod_info.name}")
        except Exception as e:
            print(f"  WARN: failed to import scenario {mod_info.name}: {e}", file=sys.stderr)


def _run_scenario(sc: Scenario, binary: Path) -> ScenarioResult:
    home = Path(tempfile.mkdtemp(prefix=f"elai_e2e_{sc.name}_"))
    ssb_db = home / ".elai_cc" / "sessions" / "ssb.db"
    env = dict(os.environ)
    env.update(sc.env_extra)

    try:
        proc = subprocess.run(
            [str(binary)] + sc.argv,
            cwd=str(home),
            capture_output=True,
            text=True,
            timeout=sc.timeout,
            env=env,
        )
        result = RunResult(
            argv=sc.argv,
            cwd=home,
            home=home,
            stdout=proc.stdout,
            stderr=proc.stderr,
            exit_code=proc.returncode,
            ssb_db_path=ssb_db,
        )
        try:
            sc.fn(result)
            return ScenarioResult(
                name=sc.name,
                passed=True,
                detail="PASS",
                evidence=f"exit={proc.returncode}, ssb_rows={len(result.ssb_rows())}",
            )
        except AssertionError as e:
            return ScenarioResult(
                name=sc.name,
                passed=False,
                detail=f"ASSERT FAIL: {e}",
                evidence=f"exit={proc.returncode}, stdout={proc.stdout[:200]}",
            )
        except Exception as e:
            return ScenarioResult(
                name=sc.name,
                passed=False,
                detail=f"ERROR: {e}\n{traceback.format_exc()}",
                evidence=f"exit={proc.returncode}, stderr={proc.stderr[:300]}",
            )
    except subprocess.TimeoutExpired:
        return ScenarioResult(name=sc.name, passed=False, detail=f"TIMEOUT ({sc.timeout}s)", evidence="")
    finally:
        shutil.rmtree(home, ignore_errors=True)


def main() -> int:
    parser = argparse.ArgumentParser(description="ELAI e2e test runner")
    parser.add_argument("--binary", default=str(DEFAULT_BINARY), help="path to elai binary")
    parser.add_argument("--scenario", default=None, help="run only this scenario")
    parser.add_argument("--list", action="store_true", help="list scenarios and exit")
    args = parser.parse_args()

    _discover_scenarios()

    if args.list:
        for sc in _REGISTRY:
            print(f"  {sc.name}: {' '.join(sc.argv)}")
        return 0

    binary = Path(args.binary)
    if not binary.exists():
        print(f"ERROR: binary not found at {binary}", file=sys.stderr)
        return 2

    scenarios = [sc for sc in _REGISTRY if args.scenario is None or sc.name == args.scenario]
    if not scenarios:
        print(f"ERROR: no scenario named '{args.scenario}'", file=sys.stderr)
        return 2

    results: list[ScenarioResult] = []
    for sc in scenarios:
        sys.stdout.write(f"  [{sc.name}] ... ")
        sys.stdout.flush()
        res = _run_scenario(sc, binary)
        results.append(res)
        status = "PASS" if res.passed else "FAIL"
        print(f"{status}  {res.detail}")
        if res.evidence:
            print(f"         evidence: {res.evidence}")

    passed = sum(1 for r in results if r.passed)
    failed = sum(1 for r in results if not r.passed)
    print(f"\n{'='*60}")
    print(f"  {passed} passed, {failed} failed, {len(results)} total")
    print(f"{'='*60}")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
