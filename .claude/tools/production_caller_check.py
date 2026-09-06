#!/usr/bin/env python3
"""production_caller_check.py -- the D-030 production-caller gate
(.claude/rules/production-caller-gate.md).

FAIL-CLOSED static (Layer-2, regex-unsound, necessary-not-sufficient) leg of
the wire-dark-machinery fix: every NEW pub fn in a changeset must have at
least one non-test, non-import reference OUTSIDE its defining file, or carry
a WIRE-DARK[<plan-id>] registration marker near its definition. This is
wiring-enforcement R6 (producer-without-consumer is a build break) + R7
(test-file callers do not satisfy production reachability) made runnable at
changeset grain, off-agent (the tool is R8-frozen in .claude/gate-manifest.txt).

Stdlib-only (matches the other .claude/tools/*.py). --json for the auditor.

Usage:
  python production_caller_check.py --files a.rs,b.rs            # changeset mode
  python production_caller_check.py --files a.rs --json
  python production_caller_check.py --symbols Foo,bar_baz        # explicit-symbol mode
  python production_caller_check.py --files a.rs --expect-symbols  # self-test: 0 new pub = exit 2

Changeset mode derives NEW pub symbols per file: for git-tracked files, the
added (+) lines of `git diff HEAD -U0`; for untracked/ignored files, every
pub symbol in the file. --symbols skips extraction (weaker grain: the
defining-file exclusion is unavailable, definition-pattern lines are
excluded instead).

Verdicts per symbol:
  WIRED            >=1 production reference outside the defining file
  INTERNAL         zero external refs but non-test same-file refs exist
                   (transitive helper) -> WARN (auditor verifies the chain)
  DARK             zero production references and kind == fn  -> BLOCK (exit 1)
  DARK-TYPE        zero production references, kind != fn     -> WARN (a type
                   used only inside its own file by a wired fn is legitimate;
                   the auditor judges)
  REGISTERED-DARK  WIRE-DARK[..] marker within 5 lines above the definition
                   -> WARN (auditor verifies the stubs.md / allowlist row)
  UNSOUND-NAME     fn name too common for name-grep (new/get/run/.. or <4
                   chars) -> WARN (auditor resolves via codegraph_callers)

Exit codes: 0 = clean (WARNs allowed), 1 = >=1 DARK fn (BLOCK), 2 =
scan-config error fail-closed (git unreachable, no input files resolved,
--expect-symbols found zero new pub symbols).

There is NO allowlist flag and NO static-only opt-out on this tool (R3): the
only escape is the in-code WIRE-DARK[..] marker, which is visible in the
diff and auditable.

UNSOUNDNESS (documented, per wiring-enforcement residual-risk discipline):
name-grep is not a call graph (R5). False-WIRED: an unrelated same-name
symbol elsewhere; a macro token-match. False-DARK: callers reached only via
macro-generated code whose source never spells the name; multi-line `pub
fn` signatures split before the name (non-rustfmt). cfg(test) spans are
brace-matched, not parsed. The authoritative legs stay Layer-3/4 runtime
evidence; this gate only guarantees a production-looking caller EXISTS
lexically, early enough for the executor to fix in-changeset.

SELF-TEST FIXTURES (auditor-run, scratch-dir, never shipped as crate source):
  (a) untracked .rs with an unreferenced `pub fn`            => exit 1, DARK
  (b) same file, fn with `// WIRE-DARK[X-0]:` 1 line above   => WARN only
  (c) --files pointing at a missing path                     => exit 2
  (d) a real wired symbol (e.g. prompt_hardening.rs)         => exit 0, WIRED
  (e) --expect-symbols over a no-new-pub changeset           => exit 2
"""

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

WORKFLOW_ROOT = Path(__file__).resolve().parents[2]

PUB_DEF = re.compile(
    r"^[ \t]*pub(?:[ \t]*\((?:crate|super|self|in[ \t]+[^)]*)\))?[ \t]+"
    r"(?:async[ \t]+|unsafe[ \t]+|const[ \t]+|extern[ \t]+\"[^\"]*\"[ \t]+)*"
    r"(fn|struct|enum|trait|type|const|static)[ \t]+([A-Za-z_][A-Za-z0-9_]*)",
    re.MULTILINE,
)
WIRE_DARK_MARKER = re.compile(r"WIRE-DARK\[[^\]]+\]")
IMPORT_LINE = re.compile(r"^[ \t]*(?:pub[ \t]+)?use\b")
CFG_TEST = re.compile(r"#\[\s*cfg\s*\(\s*test\s*\)\s*\]")
# fn names where a bare word-boundary grep proves nothing (every type has ::new).
COMMON_FN_NAMES = frozenset(
    "new default from into build run init get set len name kind id fmt clone "
    "next read write open close start stop call apply check emit push pop".split()
)
TEST_PATH_PARTS = ("tests", "benches", "examples")
SKIP_DIR_NAMES = {"target", ".git", "node_modules", "__pycache__", ".venv"}


def is_test_path(path):
    parts = [p.lower() for p in path.parts]
    if any(t in parts for t in TEST_PATH_PARTS):
        return True
    stem = path.stem.lower()
    return stem.endswith(("_test", "_tests")) or path.name == "build.rs"


def strip_comments_and_strings(text):
    """Blank Rust comments and string-literal bodies with spaces, preserving
    length and newlines so offsets/line numbers stay correct. Unsound on
    raw strings beyond one '#' and on char literals, like the sibling
    checkers; eliminates the common comment/string false-positives."""
    out = list(text)
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if c == "/" and nxt == "/":
            while i < n and text[i] != "\n":
                out[i] = " "
                i += 1
        elif c == "/" and nxt == "*":
            depth, j = 1, i + 2
            while j < n and depth:
                if text[j] == "/" and j + 1 < n and text[j + 1] == "*":
                    depth += 1
                    j += 2
                elif text[j] == "*" and j + 1 < n and text[j + 1] == "/":
                    depth -= 1
                    j += 2
                else:
                    j += 1
            for k in range(i, min(j, n)):
                if out[k] != "\n":
                    out[k] = " "
            i = j
        elif c == "r" and nxt in ('"', "#"):
            m = re.match(r'r(#*)"', text[i:])
            if m:
                close = '"' + m.group(1)
                j = text.find(close, i + len(m.group(0)))
                j = n if j < 0 else j + len(close)
                for k in range(i + len(m.group(0)), j - len(close)):
                    if out[k] != "\n":
                        out[k] = " "
                i = j
            else:
                i += 1
        elif c == '"':
            j = i + 1
            while j < n:
                if text[j] == "\\":
                    j += 2
                elif text[j] == '"':
                    j += 1
                    break
                else:
                    j += 1
            for k in range(i + 1, min(j - 1, n)):
                if out[k] != "\n":
                    out[k] = " "
            i = j
        else:
            i += 1
    return "".join(out)


def cfg_test_spans(stripped):
    """Character spans covered by #[cfg(test)] items, via brace matching on
    comment/string-stripped text."""
    spans = []
    for m in CFG_TEST.finditer(stripped):
        j = stripped.find("{", m.end())
        if j < 0:
            continue
        depth, k = 1, j + 1
        n = len(stripped)
        while k < n and depth:
            if stripped[k] == "{":
                depth += 1
            elif stripped[k] == "}":
                depth -= 1
            k += 1
        spans.append((m.start(), k))
    return spans


def in_spans(pos, spans):
    return any(a <= pos < b for a, b in spans)


def git(args):
    proc = subprocess.run(
        ["git", "-C", str(WORKFLOW_ROOT)] + args,
        capture_output=True, text=True, encoding="utf-8", errors="replace",
    )
    return proc


def resolve_input_files(raw):
    files, missing = [], []
    for token in raw:
        p = Path(token)
        if not p.is_absolute():
            p = WORKFLOW_ROOT / p
        if p.is_file():
            files.append(p.resolve())
        else:
            missing.append(token)
    return files, missing


def rel_posix(path):
    # Worktree-absolute --root (integrator candidate gates) lives outside the
    # canonical repo; fall back to absolute posix so symbols-mode hits do not
    # raise ValueError and fail-closed the whole pack (D-030 still greps names).
    try:
        return path.relative_to(WORKFLOW_ROOT).as_posix()
    except ValueError:
        return path.resolve().as_posix()


def new_symbols_for_file(path, tracked):
    """[(kind, name)] of NEW pub symbols: diff-added for tracked files,
    all pub symbols for untracked/ignored files."""
    if tracked:
        proc = git(["diff", "HEAD", "-U0", "--", rel_posix(path)])
        if proc.returncode not in (0, 1):
            raise RuntimeError("git diff failed for %s: %s" % (path, proc.stderr.strip()))
        names = []
        for line in proc.stdout.splitlines():
            if line.startswith("+") and not line.startswith("+++"):
                m = PUB_DEF.match(line[1:])
                if m:
                    names.append((m.group(1), m.group(2)))
        return names
    text = path.read_text(encoding="utf-8", errors="replace")
    return [(m.group(1), m.group(2)) for m in PUB_DEF.finditer(strip_comments_and_strings(text))]


def classify_definition(path, kind, name):
    """Locate the definition in the CURRENT file content; return
    (line_no, exempt_reason) where exempt_reason in (None, 'cfg-test',
    'registered-dark', 'not-found')."""
    text = path.read_text(encoding="utf-8", errors="replace")
    stripped = strip_comments_and_strings(text)
    spans = cfg_test_spans(stripped)
    pat = re.compile(
        r"^[ \t]*pub(?:[ \t]*\([^)]*\))?[ \t]+(?:[a-z]+[ \t]+|extern[ \t]+\"[^\"]*\"[ \t]+)*"
        + kind + r"[ \t]+" + re.escape(name) + r"\b",
        re.MULTILINE,
    )
    m = pat.search(stripped)
    if not m:
        return None, "not-found"
    line_no = stripped.count("\n", 0, m.start()) + 1
    if in_spans(m.start(), spans):
        return line_no, "cfg-test"
    raw_lines = text.splitlines()
    if line_no - 1 < len(raw_lines) and WIRE_DARK_MARKER.search(raw_lines[line_no - 1]):
        return line_no, "registered-dark"
    # contiguous comment/attribute block directly above the def, max 3 lines:
    # a blank line or another item definition ends the block (so a marker can
    # never be claimed by the NEXT symbol below it).
    for i in range(line_no - 2, max(-1, line_no - 5), -1):
        line = raw_lines[i].strip() if 0 <= i < len(raw_lines) else ""
        if not line or PUB_DEF.match(raw_lines[i]) or line.startswith("fn "):
            break
        if WIRE_DARK_MARKER.search(line):
            return line_no, "registered-dark"
    return line_no, None


def in_file_refs(def_file, name):
    """Non-test, non-import, non-definition references to `name` inside its
    own defining file (the transitive-helper case)."""
    text = def_file.read_text(encoding="utf-8", errors="replace")
    stripped = strip_comments_and_strings(text)
    spans = cfg_test_spans(stripped)
    word = re.compile(r"\b" + re.escape(name) + r"\b")
    def_line = re.compile(
        r"^[ \t]*(?:pub(?:[ \t]*\([^)]*\))?[ \t]+)?(?:[a-z]+[ \t]+)*"
        r"(?:fn|struct|enum|trait|type|const|static)[ \t]+" + re.escape(name) + r"\b"
    )
    count, offset = 0, 0
    for line in stripped.splitlines(True):
        if word.search(line) and not IMPORT_LINE.match(line) \
                and not def_line.match(line) and not in_spans(offset, spans):
            count += 1
        offset += len(line)
    return count


class CallerScanner:
    """One-pass cached scan of --root for production references."""

    def __init__(self, root):
        self.root = root
        self.files = [
            p for p in root.rglob("*.rs")
            if not (set(part.lower() for part in p.parts) & SKIP_DIR_NAMES)
            and not is_test_path(p)
        ]
        self._cache = {}

    def _stripped(self, path):
        if path not in self._cache:
            text = path.read_text(encoding="utf-8", errors="replace")
            s = strip_comments_and_strings(text)
            self._cache[path] = (s, cfg_test_spans(s))
        return self._cache[path]

    def production_refs(self, name, exclude_file):
        word = re.compile(r"\b" + re.escape(name) + r"\b")
        def_line = re.compile(
            r"^[ \t]*(?:pub(?:[ \t]*\([^)]*\))?[ \t]+)?(?:[a-z]+[ \t]+)*"
            r"(?:fn|struct|enum|trait|type|const|static)[ \t]+" + re.escape(name) + r"\b"
        )
        hits = []
        for path in self.files:
            if exclude_file is not None and path.samefile(exclude_file):
                continue
            stripped, spans = self._stripped(path)
            if name not in stripped:
                continue
            offset = 0
            for line_no, line in enumerate(stripped.splitlines(True), start=1):
                if word.search(line) and not IMPORT_LINE.match(line) \
                        and not def_line.match(line) and not in_spans(offset, spans):
                    hits.append("%s:%d" % (rel_posix(path), line_no))
                    if len(hits) >= 3:
                        return hits
                offset += len(line)
        return hits


def main():
    ap = argparse.ArgumentParser(description="D-030 production-caller gate (R6/R7 at changeset grain)")
    ap.add_argument("--files", default="", help="comma-separated changeset files (.rs scanned, rest noted)")
    ap.add_argument("--symbols", default="", help="comma-separated explicit symbol names (skips extraction)")
    ap.add_argument("--root", default=".elai_cc", help="caller-scan root (default .elai_cc)")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--expect-symbols", action="store_true",
                    help="self-test: zero NEW pub symbols extracted = exit 2")
    args = ap.parse_args()

    def finish(payload, code):
        if args.json:
            print(json.dumps(payload, indent=2))
        else:
            for s in payload["symbols"]:
                print("%-16s %s %s (%s) %s" % (
                    s["verdict"], s["kind"], s["name"], s.get("defined_in", "?"),
                    "; ".join(s.get("refs", []))))
            for note in payload["notes"]:
                print("NOTE: " + note)
            print("VERDICT: %s (%d dark fn, %d warn, %d wired)" % (
                payload["verdict"], payload["dark_fn_count"],
                payload["warn_count"], payload["wired_count"]))
        sys.exit(code)

    payload = {"verdict": "PASS", "symbols": [], "notes": [],
               "dark_fn_count": 0, "warn_count": 0, "wired_count": 0}

    root = Path(args.root)
    if not root.is_absolute():
        root = WORKFLOW_ROOT / root
    if not root.is_dir():
        payload["verdict"] = "ERROR"
        payload["notes"].append("caller-scan root not found: %s" % root)
        finish(payload, 2)

    targets = []  # (kind, name, defining_file_or_None)
    if args.files.strip():
        if git(["rev-parse", "--show-toplevel"]).returncode != 0:
            payload["verdict"] = "ERROR"
            payload["notes"].append("git unreachable at %s (R2: dead evidence path)" % WORKFLOW_ROOT)
            finish(payload, 2)
        files, missing = resolve_input_files(
            [t.strip() for t in args.files.split(",") if t.strip()])
        if missing:
            payload["verdict"] = "ERROR"
            payload["notes"].append("missing input files: %s" % ", ".join(missing))
            finish(payload, 2)
        rs_files = [f for f in files if f.suffix == ".rs"]
        skipped = [f for f in files if f.suffix != ".rs"]
        if skipped:
            payload["notes"].append("non-Rust inputs noted, not scanned (v1 is Rust-only): %s"
                                    % ", ".join(rel_posix(f) for f in skipped))
        if not rs_files and not args.symbols.strip():
            payload["notes"].append("no .rs files in changeset; nothing to gate")
            finish(payload, 0)
        ls = git(["ls-files", "--"] + [rel_posix(f) for f in rs_files])
        tracked_set = set(ls.stdout.split())
        for f in rs_files:
            if is_test_path(f.relative_to(WORKFLOW_ROOT)):
                payload["notes"].append("test-path file skipped (R7): %s" % rel_posix(f))
                continue
            for kind, name in new_symbols_for_file(f, rel_posix(f) in tracked_set):
                targets.append((kind, name, f))
    if args.symbols.strip():
        for name in [t.strip() for t in args.symbols.split(",") if t.strip()]:
            targets.append(("fn", name, None))

    if not targets:
        if args.expect_symbols:
            payload["verdict"] = "ERROR"
            payload["notes"].append("--expect-symbols: zero NEW pub symbols extracted (self-test FAIL)")
            finish(payload, 2)
        if not args.files.strip() and not args.symbols.strip():
            payload["verdict"] = "ERROR"
            payload["notes"].append("no --files and no --symbols given")
            finish(payload, 2)
        payload["notes"].append("zero NEW pub symbols in changeset; nothing to gate")
        finish(payload, 0)

    scanner = CallerScanner(root)
    seen = set()
    for kind, name, def_file in targets:
        key = (kind, name, str(def_file))
        if key in seen:
            continue
        seen.add(key)
        entry = {"kind": kind, "name": name,
                 "defined_in": rel_posix(def_file) if def_file else "(--symbols mode)"}
        if def_file is not None:
            line_no, exempt = classify_definition(def_file, kind, name)
            if exempt == "not-found":
                entry["verdict"] = "SKIPPED"
                payload["notes"].append("%s %s: in diff but not in current file (renamed?); skipped" % (kind, name))
                payload["symbols"].append(entry)
                continue
            entry["defined_in"] += ":%d" % line_no
            if exempt == "cfg-test":
                entry["verdict"] = "SKIPPED"
                payload["notes"].append("%s %s: defined under #[cfg(test)]; not production machinery" % (kind, name))
                payload["symbols"].append(entry)
                continue
            if exempt == "registered-dark":
                entry["verdict"] = "REGISTERED-DARK"
                payload["warn_count"] += 1
                payload["symbols"].append(entry)
                continue
        if kind == "fn" and (name in COMMON_FN_NAMES or len(name) < 4):
            # a bare word-grep on `new`/`run`/.. matches every type's method:
            # any verdict from refs would be unsound in BOTH directions.
            entry["verdict"] = "UNSOUND-NAME"
            payload["warn_count"] += 1
            payload["symbols"].append(entry)
            continue
        refs = scanner.production_refs(name, def_file)
        entry["refs"] = refs
        if refs:
            entry["verdict"] = "WIRED"
            payload["wired_count"] += 1
        elif def_file is not None and in_file_refs(def_file, name):
            # module-internal helper reached transitively by same-file code
            # (the caller's own wiring is judged separately) -- not a BLOCK,
            # but surfaced so the auditor can verify the transitive chain.
            entry["verdict"] = "INTERNAL"
            payload["warn_count"] += 1
        elif kind == "fn":
            entry["verdict"] = "DARK"
            payload["dark_fn_count"] += 1
        else:
            entry["verdict"] = "DARK-TYPE"
            payload["warn_count"] += 1
        payload["symbols"].append(entry)

    if payload["dark_fn_count"]:
        payload["verdict"] = "BLOCK"
        finish(payload, 1)
    finish(payload, 0)


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception as exc:  # fail-closed: a crashed scan is a config error (2), never PASS/BLOCK
        print("SCAN ERROR (fail-closed, exit 2): %s: %s" % (type(exc).__name__, exc), file=sys.stderr)
        sys.exit(2)
