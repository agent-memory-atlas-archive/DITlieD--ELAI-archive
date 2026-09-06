#!/usr/bin/env python3
"""python_wire_check.py -- the D-030 production-caller gate for Python.
(.claude/rules/production-caller-gate.md, wiring-enforcement R6/R7)

FAIL-CLOSED static (Layer-2, AST-sound at module-level, necessary-not-sufficient)
sibling of production_caller_check.py (which is Rust-only).  Mirrors that
tool's contract exactly: same CLI flags, same verdict names, same exit codes,
same WIRE-DARK[<id>] escape hatch.

PURPOSE: detect Python functions/classes that are defined but never referenced
outside their defining file (wire-dark machinery), so the executor can wire
them in-changeset before the auditor's off-agent pass.  The load-bearing proof
remains the Layer-3/4 runtime legs; this gate only guarantees a
production-looking reference EXISTS lexically.

SCOPE v1 -- MODULE-LEVEL SYMBOLS ONLY
Public module-level `def` / `async def` / `class` whose name does NOT start
with `_` (dunder/private) are scanned.  Public METHODS are OUT OF SCOPE for
v1: scanning a method reference requires resolving which class an attribute
access belongs to, which demands a type-inference pass (ast alone is
unsound).  Module-level is where wire-dark LLM executors produce dead code
most often; method-level gap is documented as residual risk.

SOUNDNESS vs. RUST SIBLING
Python is fully parseable via the stdlib `ast` module, so reference detection
uses real AST nodes (ast.Name / ast.Attribute / ast.ImportFrom / ast.Import)
rather than raw regex.  This eliminates the comment-and-string false-positives
the Rust tool faces but introduces different blind spots:

RESIDUAL RISK / UNSOUNDNESS (documented -- the runtime leg stays authoritative):
  - Dynamic dispatch: getattr(obj, fn_name)(), globals()[fn_name](), plugin
    registries keyed by string, and __all__ re-exports hide callers from static
    analysis.
  - Decorator registration: @app.route, @router.get, @celery.task, @pytest.mark,
    @register("name") etc. -- the decorated function IS referenced by the
    framework at import time, but we see only the decorator AST node, not the
    eventual call site.  We detect the "Attribute.func.attr == symbol_name" case
    (i.e. a decorator whose last name matches), so common framework decorators
    will often be WIRED already; exotic registry decorators may still DARK-false.
  - __all__-gated public API: a module that lists a symbol in __all__ but never
    imports it itself, with all callers reaching it via `from pkg import *`,
    appears DARK.  Uncommon in application code; note as residual risk.
  - Cross-package imports not on the scan root: callers outside --root are not
    seen.  Auto mode derives a package root separately for each file-backed target.
  - Conditional imports (TYPE_CHECKING guards): from __future__ import annotations
    + if TYPE_CHECKING: only-callers look like DARK.

CONTRACT (mirrors production_caller_check.py exactly):
  Verdicts:
    WIRED          >=1 non-test reference outside the defining file
    INTERNAL       zero external refs but non-test same-file refs exist -> WARN
    DARK           public FUNCTION with zero non-test refs -> BLOCK (exit 1)
    DARK-TYPE      public CLASS with zero external refs but same-file use -> WARN
    REGISTERED-DARK  # WIRE-DARK[<id>]: comment within ~3 lines above def -> WARN
    UNSOUND-NAME   (not used in v1 -- AST avoids most name-collision noise;
                   reserved field, never emitted currently)

  Exit codes:
    0  clean  (WARNs allowed)
    1  >=1 DARK fn -> BLOCK
    2  scan-config error (fail-closed): missing input file, git unreachable when
       needed, --expect-symbols found zero new public symbols, unhandled exception

  No allowlist flag, no static-only opt-out (R3).  The only escape is the
  in-code # WIRE-DARK[<id>]: marker.

Usage:
  python python_wire_check.py --files a.py,b.py [--root pkg/] [--json]
  python python_wire_check.py --files a.py b.py [--root pkg/] [--json]
  python python_wire_check.py --files a.py --expect-symbols   # self-test
  python python_wire_check.py --symbols foo,Bar               # explicit names (weaker)
"""

import argparse
import ast
import json
import subprocess
import sys
from pathlib import Path
from typing import Optional

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

WORKFLOW_ROOT = Path(__file__).resolve().parents[2]

# Test-file / test-directory detection (mirrors R7 test-callers-dont-count).
TEST_DIR_NAMES = {"tests", "test", "audit-scratch", "conftest"}
TEST_FILE_PATTERNS = ("test_", "_test", "conftest")

# WIRE-DARK marker regex (plain string match, fast).
WIRE_DARK_TOKEN = "WIRE-DARK["

# Very short or extremely generic names for which even an AST reference is
# ambiguous (e.g. `get`, `run`).  Reserved for UNSOUND-NAME in v1 but
# currently not emitted (AST is sounder than grep).
_SHORT_THRESHOLD = 3  # names <= this length are potentially noisy

SKIP_DIR_NAMES = {".git", "__pycache__", ".venv", "venv", "node_modules",
                  "target", ".mypy_cache", ".pytest_cache", "build", "dist"}


# ---------------------------------------------------------------------------
# Path helpers
# ---------------------------------------------------------------------------

def is_test_file(path: Path) -> bool:
    """True if path is under a test directory or is a test/conftest file."""
    parts_lower = [p.lower() for p in path.parts]
    for name in parts_lower:
        if name in TEST_DIR_NAMES:
            return True
    stem = path.stem.lower()
    for pat in TEST_FILE_PATTERNS:
        if stem.startswith(pat) or stem.endswith(pat):
            return True
    return False


def _rel(path: Path) -> str:
    try:
        return path.relative_to(WORKFLOW_ROOT).as_posix()
    except ValueError:
        return str(path)


# ---------------------------------------------------------------------------
# Git helpers
# ---------------------------------------------------------------------------

def _git(*args) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["git", "-C", str(WORKFLOW_ROOT)] + list(args),
        capture_output=True, text=True, encoding="utf-8", errors="replace",
    )


def _is_tracked(path: Path) -> bool:
    proc = _git("ls-files", "--", _rel(path))
    return bool(proc.stdout.strip())


def _git_available() -> bool:
    return _git("rev-parse", "--show-toplevel").returncode == 0


# ---------------------------------------------------------------------------
# WIRE-DARK marker detection
# ---------------------------------------------------------------------------

def _has_wire_dark_marker(source_lines: list[str], def_lineno: int) -> bool:
    """Check the ~3 lines directly above def_lineno (1-indexed) for a
    # WIRE-DARK[<id>]: marker.  A blank line or non-comment non-decorator line
    stops the search (prevents a marker from being claimed by the next symbol
    below)."""
    # def_lineno is 1-indexed; scan backwards from the line above.
    idx = def_lineno - 2  # 0-indexed line just before the def
    scanned = 0
    while idx >= 0 and scanned < 4:
        line = source_lines[idx].strip()
        if WIRE_DARK_TOKEN in line:
            return True
        # Stop on blank lines or lines that look like another definition
        # (prevents marker "leaking" across symbol boundaries).
        if not line:
            break
        if not (line.startswith("#") or line.startswith("@") or line.startswith("def ")
                or line.startswith("async def") or line.startswith("class ")):
            # non-comment, non-decorator, non-def line -- stop
            break
        idx -= 1
        scanned += 1
    # also check the def line itself (inline comment)
    def_line = source_lines[def_lineno - 1] if def_lineno - 1 < len(source_lines) else ""
    return WIRE_DARK_TOKEN in def_line


# ---------------------------------------------------------------------------
# New-symbol extraction
# ---------------------------------------------------------------------------

def _top_level_public_symbols(tree: ast.Module, source_lines: list[str]) -> list[dict]:
    """Return list of {name, kind, lineno, has_wire_dark} for each public
    top-level def/class in the module."""
    results = []
    for node in ast.iter_child_nodes(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            name = node.name
            if name.startswith("_"):
                continue
            kind = "fn"
        elif isinstance(node, ast.ClassDef):
            name = node.name
            if name.startswith("_"):
                continue
            kind = "class"
        else:
            continue
        lineno = node.lineno
        has_marker = _has_wire_dark_marker(source_lines, lineno)
        results.append({"name": name, "kind": kind, "lineno": lineno,
                        "has_wire_dark": has_marker})
    return results


def _added_symbol_names(path: Path) -> set[str]:
    """From git diff HEAD -U0, return the set of symbol names whose def line
    appeared in a + hunk.  Returns None if git diff fails."""
    proc = _git("diff", "HEAD", "-U0", "--", _rel(path))
    if proc.returncode not in (0, 1):
        return None
    added_names: set[str] = set()
    for line in proc.stdout.splitlines():
        if not (line.startswith("+") and not line.startswith("+++")):
            continue
        stripped = line[1:].strip()
        # Match `def NAME`, `async def NAME`, `class NAME`
        for prefix in ("def ", "async def ", "class "):
            if stripped.startswith(prefix):
                rest = stripped[len(prefix):].split("(")[0].split(":")[0].strip()
                # rest should be the name; validate it's a valid Python id
                if rest.isidentifier():
                    added_names.add(rest)
    return added_names


def new_symbols_for_file(path: Path) -> list[dict]:
    """Return [{name, kind, lineno, has_wire_dark}] for NEW public symbols.
    For git-tracked files: only symbols whose def line appeared in a + hunk.
    For untracked files: all public top-level symbols."""
    try:
        source = path.read_text(encoding="utf-8", errors="replace")
        source_lines = source.splitlines()
        tree = ast.parse(source, filename=str(path))
    except SyntaxError as exc:
        raise RuntimeError("SyntaxError in %s: %s" % (_rel(path), exc))

    all_syms = _top_level_public_symbols(tree, source_lines)

    if _is_tracked(path):
        added = _added_symbol_names(path)
        if added is None:
            raise RuntimeError("git diff failed for %s" % _rel(path))
        return [s for s in all_syms if s["name"] in added]
    # untracked/ignored: treat every public symbol as new
    return all_syms


# ---------------------------------------------------------------------------
# Reference scanning (AST-based)
# ---------------------------------------------------------------------------

class _RefScanner(ast.NodeVisitor):
    """Collect Name and Attribute references to a target symbol name."""

    def __init__(self, target: str):
        self.target = target
        self.refs: list[int] = []  # line numbers

    # A bare name reference: `foo` used as a value
    def visit_Name(self, node: ast.Name):
        if node.id == self.target and isinstance(node.ctx, (ast.Load, ast.Del)):
            self.refs.append(node.lineno)
        self.generic_visit(node)

    # An attribute access: `mod.foo` or `obj.foo`
    def visit_Attribute(self, node: ast.Attribute):
        if node.attr == self.target and isinstance(node.ctx, (ast.Load, ast.Del)):
            self.refs.append(node.lineno)
        self.generic_visit(node)

    # `from module import foo` or `from module import foo as bar`
    def visit_ImportFrom(self, node: ast.ImportFrom):
        for alias in node.names:
            if alias.name == self.target:
                self.refs.append(node.lineno)
        self.generic_visit(node)

    # `import module` -- symbol matches if name matches (rare; included for completeness)
    def visit_Import(self, node: ast.Import):
        for alias in node.names:
            # match `import foo` or `import pkg.foo`
            short = alias.name.rsplit(".", 1)[-1]
            if short == self.target:
                self.refs.append(node.lineno)
        self.generic_visit(node)


def _refs_in_file(path: Path, symbol: str) -> list[int]:
    """Return list of line numbers where symbol is referenced in path (AST scan).
    Returns [] on SyntaxError (don't crash the scan for one bad file)."""
    try:
        source = path.read_text(encoding="utf-8", errors="replace")
        tree = ast.parse(source, filename=str(path))
    except SyntaxError:
        return []
    scanner = _RefScanner(symbol)
    scanner.visit(tree)
    return scanner.refs


def _same_file_non_def_refs(path: Path, symbol: str, def_lineno: int) -> int:
    """Count non-def, non-import, same-file references to symbol (INTERNAL check)."""
    try:
        source = path.read_text(encoding="utf-8", errors="replace")
        tree = ast.parse(source, filename=str(path))
    except SyntaxError:
        return 0

    count = 0
    for node in ast.walk(tree):
        if isinstance(node, ast.Name):
            if (node.id == symbol and isinstance(node.ctx, ast.Load)
                    and node.lineno != def_lineno):
                count += 1
        elif isinstance(node, ast.Attribute):
            if (node.attr == symbol and isinstance(node.ctx, ast.Load)
                    and node.lineno != def_lineno):
                count += 1
    return count


# ---------------------------------------------------------------------------
# Package root discovery
# ---------------------------------------------------------------------------

def _find_package_root(path: Path) -> Path:
    """Walk up from path's parent until we leave Python package territory
    (no __init__.py) or hit the workflow root.  Returns the outermost
    package directory that still contains __init__.py."""
    candidate = path.parent
    while candidate != WORKFLOW_ROOT and candidate.parent != candidate:
        if not (candidate / "__init__.py").exists():
            # We've left the package -- go back one step
            return candidate
        candidate = candidate.parent
    return path.parent


def _collect_py_files(root: Path) -> list[Path]:
    """All .py files under root, excluding skip dirs and test files."""
    results = []
    for p in root.rglob("*.py"):
        # skip hidden dirs, venvs, caches, etc.
        if any(part.lower() in SKIP_DIR_NAMES for part in p.parts):
            continue
        if is_test_file(p):
            continue
        results.append(p)
    return results


# ---------------------------------------------------------------------------
# Core verdict logic
# ---------------------------------------------------------------------------

def verdict_for_symbol(
    sym: dict,
    def_file: Path,
    scan_root: Path,
) -> dict:
    """Compute verdict entry for one symbol."""
    name = sym["name"]
    kind = sym["kind"]
    lineno = sym["lineno"]
    has_marker = sym.get("has_wire_dark", False)

    entry = {
        "symbol": name,
        "file": _rel(def_file),
        "line": lineno,
        "kind": kind,
        "verdict": "",
        "evidence": [],
    }

    # REGISTERED-DARK: marker in source -> downgrade to WARN
    if has_marker:
        entry["verdict"] = "REGISTERED-DARK"
        return entry

    # Scan the rest of the package for references (R7: exclude test files,
    # exclude the defining file itself).
    py_files = [p for p in _collect_py_files(scan_root) if not p.samefile(def_file)]

    external_refs: list[str] = []
    for p in py_files:
        lines = _refs_in_file(p, name)
        for ln in lines:
            external_refs.append("%s:%d" % (_rel(p), ln))
            if len(external_refs) >= 3:
                break
        if len(external_refs) >= 3:
            break

    entry["evidence"] = external_refs

    if external_refs:
        entry["verdict"] = "WIRED"
        return entry

    # No external refs -- check same-file refs for INTERNAL
    same_refs = _same_file_non_def_refs(def_file, name, lineno)

    if same_refs:
        if kind == "fn":
            entry["verdict"] = "INTERNAL"
        else:
            entry["verdict"] = "DARK-TYPE"  # class used locally only
        return entry

    # Zero refs anywhere
    if kind == "fn":
        entry["verdict"] = "DARK"
    else:
        entry["verdict"] = "DARK-TYPE"
    return entry


# ---------------------------------------------------------------------------
# File resolution
# ---------------------------------------------------------------------------

def resolve_files(raw_tokens: list[str]) -> tuple[list[Path], list[str]]:
    files, missing = [], []
    for tok in raw_tokens:
        p = Path(tok)
        if not p.is_absolute():
            p = WORKFLOW_ROOT / p
        if p.is_file():
            files.append(p.resolve())
        else:
            missing.append(tok)
    return files, missing


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    ap = argparse.ArgumentParser(
        description="D-030 Python production-caller gate (R6/R7 at changeset grain)"
    )
    ap.add_argument("--files", nargs="+", default=[],
                    help="one or more changeset .py files; each value can be comma-separated")
    ap.add_argument("--symbols", default="",
                    help="comma-separated explicit symbol names (skips extraction; "
                         "weaker: defining-file exclusion unavailable)")
    ap.add_argument("--root", default="",
                    help="caller-scan root directory (default: auto-detect per file)")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--expect-symbols", action="store_true",
                    help="self-test: zero new public symbols extracted = exit 2")
    args = ap.parse_args()

    # ---- output helper ----
    def finish(payload: dict, code: int):
        if args.json:
            print(json.dumps(payload, indent=2))
        else:
            for s in payload["symbols"]:
                print("%-16s %s %s (%s:%s)  %s" % (
                    s["verdict"], s["kind"], s["symbol"],
                    s.get("file", "?"), s.get("line", "?"),
                    "; ".join(s.get("evidence", [])),
                ))
            for note in payload["notes"]:
                print("NOTE: " + note)
            print("VERDICT: %s  (%d dark fn, %d warn, %d wired)" % (
                payload["verdict"],
                payload["dark_fn_count"],
                payload["warn_count"],
                payload["wired_count"],
            ))
        sys.exit(code)

    payload: dict = {
        "verdict": "PASS",
        "symbols": [],
        "notes": [],
        "dark_fn_count": 0,
        "warn_count": 0,
        "wired_count": 0,
    }

    file_values = list(args.files)

    # ---- collect targets (kind, name, def_file | None) ----
    targets: list[tuple[str, str, Optional[Path]]] = []

    if file_values:
        # need git for tracked-file diff; fail-closed if unavailable
        if not _git_available():
            payload["verdict"] = "ERROR"
            payload["notes"].append(
                "git unreachable at %s (R2: dead evidence path)" % WORKFLOW_ROOT)
            finish(payload, 2)

        raw_tokens = [
            token.strip()
            for value in file_values
            for token in value.split(",")
            if token.strip()
        ]
        if not raw_tokens:
            payload["verdict"] = "ERROR"
            payload["notes"].append("no --files paths after comma splitting")
            finish(payload, 2)
        files, missing = resolve_files(raw_tokens)

        if missing:
            payload["verdict"] = "ERROR"
            payload["notes"].append("missing input files: %s" % ", ".join(missing))
            finish(payload, 2)

        py_files = [f for f in files if f.suffix == ".py"]
        skipped = [f for f in files if f.suffix != ".py"]
        if skipped:
            payload["notes"].append(
                "non-Python inputs noted, not scanned (v1 is Python-only): %s"
                % ", ".join(_rel(f) for f in skipped)
            )

        if not py_files and not args.symbols.strip():
            payload["notes"].append("no .py files in changeset; nothing to gate")
            finish(payload, 0)

        for f in py_files:
            if is_test_file(f):
                payload["notes"].append("test-path file skipped (R7): %s" % _rel(f))
                continue
            syms = new_symbols_for_file(f)
            for s in syms:
                targets.append((s["kind"], s["name"], f, s))

    # explicit symbols (weaker, no file exclusion)
    if args.symbols.strip():
        for name in [t.strip() for t in args.symbols.split(",") if t.strip()]:
            targets.append(("fn", name, None, {"name": name, "kind": "fn",
                                               "lineno": 0, "has_wire_dark": False}))

    # ---- --expect-symbols self-test ----
    if not targets:
        if args.expect_symbols:
            payload["verdict"] = "ERROR"
            payload["notes"].append(
                "--expect-symbols: zero NEW public symbols extracted (self-test FAIL)")
            finish(payload, 2)
        if not file_values and not args.symbols.strip():
            payload["verdict"] = "ERROR"
            payload["notes"].append("no --files and no --symbols given")
            finish(payload, 2)
        payload["notes"].append("zero NEW public symbols in changeset; nothing to gate")
        finish(payload, 0)

    # ---- determine scan root ----
    if args.root.strip():
        scan_root: Optional[Path] = Path(args.root)
        if not scan_root.is_absolute():
            scan_root = WORKFLOW_ROOT / scan_root
        if not scan_root.is_dir():
            payload["verdict"] = "ERROR"
            payload["notes"].append("--root not found: %s" % scan_root)
            finish(payload, 2)
    else:
        # Auto mode derives a root per file-backed target; --symbols uses the workflow root.
        scan_root = None

    # ---- run verdicts ----
    seen: set[tuple] = set()
    for kind, name, def_file, sym_dict in targets:
        key = (kind, name, str(def_file))
        if key in seen:
            continue
        seen.add(key)

        if def_file is not None:
            target_root = scan_root if scan_root is not None else _find_package_root(def_file)
            entry = verdict_for_symbol(sym_dict, def_file, target_root)
        else:
            # --symbols mode: scan whole root, no def_file exclusion
            target_root = scan_root if scan_root is not None else WORKFLOW_ROOT
            refs: list[str] = []
            for p in _collect_py_files(target_root):
                for ln in _refs_in_file(p, name):
                    refs.append("%s:%d" % (_rel(p), ln))
                    if len(refs) >= 3:
                        break
                if len(refs) >= 3:
                    break
            entry = {
                "symbol": name, "file": "(--symbols mode)", "line": 0,
                "kind": kind, "verdict": "WIRED" if refs else "DARK",
                "evidence": refs,
            }

        v = entry["verdict"]
        if v == "WIRED":
            payload["wired_count"] += 1
        elif v == "DARK":
            payload["dark_fn_count"] += 1
        else:
            payload["warn_count"] += 1

        payload["symbols"].append(entry)

    # ---- final verdict ----
    if payload["dark_fn_count"]:
        payload["verdict"] = "BLOCK"
        finish(payload, 1)
    finish(payload, 0)


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception as exc:
        # fail-closed: a crashed scan is a config error (2), never PASS/BLOCK
        print("SCAN ERROR (fail-closed, exit 2): %s: %s" % (type(exc).__name__, exc),
              file=sys.stderr)
        sys.exit(2)
