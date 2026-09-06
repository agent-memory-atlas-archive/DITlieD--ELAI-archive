#!/usr/bin/env python3
"""traced_test_assert_check.py -- auditor checker for .claude/rules/tracing-test-assert.md.

FAIL-CLOSED static (Layer-2, regex-unsound, necessary-not-sufficient) leg of the
tracing-test mandatory-assert rule. Rejects any #[traced_test] test whose body
lacks an ASSERTED logs_contain/logs_assert call, and bans BatchSpanProcessor in
test files.

Stdlib-only (matches the other .claude/tools/*.py). --json for the auditor.

Usage:
  python traced_test_assert_check.py --files a.rs,b.rs        # scan a changeset
  python traced_test_assert_check.py --root .elai_cc/crates    # scan all tests/+_contract/_test
  python traced_test_assert_check.py --files x.rs --json
  python traced_test_assert_check.py --root .elai_cc/crates --expect-traced  # R2 self-test

Exit codes: 0 = clean, 1 = >=1 violation (BLOCK), 2 = scan-config error /
no files reached (R2 self-test: a --expect-traced scan that touched files but
found zero #[traced_test] is a FAIL, surfaced as exit 2).

UNSOUNDNESS (documented, per wiring-enforcement residual-risk discipline):
this is a regex/brace-matcher, NOT a Rust parser. It can false-positive on
logs_contain inside a string literal or comment, and false-negative on an
assert macro aliased or wrapped behind a helper fn. The load-bearing backstop
is the Layer-4 harness's real assert! firing at runtime; this checker only
guarantees the assert! token EXISTS lexically inside the test body.

SELF-TEST / FIXTURE NOTE:
The intended fixtures (auditor-run, not shipped as Rust source so they do not
pollute the real test suite) prove the checker fails-closed:
  (a) a .rs with `#[traced_test]` + bare `logs_contain("x");`        => exit 1, T-1
  (b) a .rs with `#[traced_test]` + no logs_* call at all            => exit 1, T-2
  (c) a .rs containing `BatchSpanProcessor::builder()`               => exit 1, T-3
  (d) a .rs with `assert!(logs_contain("x"))`                        => exit 0
  (e) `--expect-traced` over a dir with zero #[traced_test]          => exit 2
Drop these five one-line .rs fixtures in a scratch dir and run the checker on
each to confirm the exit codes above before trusting a live run.
"""
import argparse
import json
import re
import sys
from pathlib import Path

# A #[traced_test] attribute in any of its accepted spellings.
TRACED_ATTR = re.compile(r"#\[\s*(?:tracing_test\s*::\s*)?traced_test\b")
# An assert-family macro invocation whose argument list contains a
# logs_contain / logs_assert call. [\s\S] spans newlines so a multi-line
# assert!(\n    logs_contain(..)\n) still matches.
ASSERTED_LOGS = re.compile(
    r"(?:assert|assert_eq|assert_ne|debug_assert|debug_assert_eq)\s*!\s*\("
    r"[\s\S]*?(?:logs_contain|logs_assert)\s*\("
)
# Any logs_contain/logs_assert call at all (to distinguish "present but
# un-asserted" T-1 from "entirely absent" T-2).
ANY_LOGS = re.compile(r"(?:logs_contain|logs_assert)\s*\(")
# Banned span processor in test setup (T-3).
BATCH_PROC = re.compile(r"\bBatchSpanProcessor\b")
FN_HEADER = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(")


def _strip_comments_and_strings(text):
    """Replace Rust line comments (// and ///), block comments (/* */), and
    string-literal contents with spaces of equal length, preserving newline
    positions and total length so line numbers stay correct.

    This is what keeps the checker from false-flagging a banned token (e.g.
    `BatchSpanProcessor`) or a `logs_contain` that only appears inside a doc
    comment or a string literal -- including this checker's own harness, whose
    OTel note documents the BatchSpanProcessor ban in prose. Still unsound
    (no awareness of raw strings r#"..."# nesting depth beyond one '#', or of
    char literals), but eliminates the common comment/string false-positives.
    """
    out = []
    i = 0
    n = len(text)
    while i < n:
        c = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        # Line comment // ... (also covers /// and //!) to end of line.
        if c == "/" and nxt == "/":
            j = text.find("\n", i)
            if j == -1:
                j = n
            out.append(" " * (j - i))
            i = j
            continue
        # Block comment /* ... */ (non-nested approximation).
        if c == "/" and nxt == "*":
            j = text.find("*/", i + 2)
            end = (j + 2) if j != -1 else n
            seg = text[i:end]
            # Preserve newlines inside the block comment for line accuracy.
            out.append("".join(ch if ch == "\n" else " " for ch in seg))
            i = end
            continue
        # String literal "..." with \" escapes.
        if c == '"':
            out.append('"')
            i += 1
            while i < n:
                ch = text[i]
                if ch == "\\" and i + 1 < n:
                    out.append("  ")
                    i += 2
                    continue
                if ch == '"':
                    out.append('"')
                    i += 1
                    break
                out.append("\n" if ch == "\n" else " ")
                i += 1
            continue
        out.append(c)
        i += 1
    return "".join(out)


def _fn_body_span(text, attr_end):
    """Given an index just past a #[traced_test] attr, find the next fn header
    and return (fn_name, body_text) by brace-matching the first { ... } block.
    Returns (None, '') if no fn header follows, or (name, '') if no body."""
    m = FN_HEADER.search(text, attr_end)
    if not m:
        return None, ""
    name = m.group(1)
    brace_start = text.find("{", m.end())
    if brace_start == -1:
        return name, ""
    depth = 0
    i = brace_start
    while i < len(text):
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return name, text[brace_start : i + 1]
        i += 1
    return name, text[brace_start:]  # unbalanced; return tail, treated as body


def scan_file(path):
    """Return (traced_test_count, [violation_dict])."""
    try:
        raw = path.read_text(encoding="utf-8", errors="replace")
    except OSError as e:
        return 0, [{"file": str(path), "rule": "IO", "detail": f"read failed: {e}"}]
    # Strip comments + string-literal contents so a banned token or a
    # logs_contain mention inside prose/strings does not false-positive.
    # Length + newline positions are preserved, so line numbers stay correct.
    text = _strip_comments_and_strings(raw)
    viols = []
    traced = 0
    # T-3: BatchSpanProcessor anywhere in a test file is banned.
    for bm in BATCH_PROC.finditer(text):
        line = text.count("\n", 0, bm.start()) + 1
        viols.append({
            "file": str(path), "line": line, "rule": "T-3",
            "detail": "BatchSpanProcessor banned in test setup (flush timing fail-opens span "
                      "assertions; use SimpleSpanProcessor + InMemorySpanExporter)",
        })
    for am in TRACED_ATTR.finditer(text):
        traced += 1
        name, body = _fn_body_span(text, am.end())
        line = text.count("\n", 0, am.start()) + 1
        label = name or "<unknown_fn>"
        if not body:
            viols.append({
                "file": str(path), "line": line, "rule": "T-1", "fn": label,
                "detail": "#[traced_test] with no resolvable fn body",
            })
            continue
        if not ANY_LOGS.search(body):
            viols.append({
                "file": str(path), "line": line, "rule": "T-2", "fn": label,
                "detail": "#[traced_test] body asserts NOTHING (no logs_contain/logs_assert) "
                          "-- wire-dark, BLOCK",
            })
        elif not ASSERTED_LOGS.search(body):
            viols.append({
                "file": str(path), "line": line, "rule": "T-1", "fn": label,
                "detail": "logs_contain/logs_assert present but NOT inside an assert!-family "
                          "macro (returns bool, does not auto-fail) -- silent pass, BLOCK",
            })
    return traced, viols


def collect_files(args):
    files = []
    if args.files:
        files = [Path(p.strip()) for p in args.files.split(",") if p.strip()]
    elif args.root:
        root = Path(args.root)
        for p in root.rglob("*.rs"):
            posix = p.as_posix()
            if (
                "/tests/" in posix
                or p.name.endswith("_test.rs")
                or p.name.endswith("_contract.rs")
            ):
                files.append(p)
    return [f for f in files if f.suffix == ".rs"]


def main():
    ap = argparse.ArgumentParser(description="tracing-test mandatory-assert checker")
    ap.add_argument("--files", help="comma-separated .rs files")
    ap.add_argument("--root", help="dir to recurse for test files")
    ap.add_argument(
        "--expect-traced",
        action="store_true",
        help="R2 self-test: FAIL (exit 2) if zero #[traced_test] found across the scan",
    )
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    files = collect_files(args)
    if not files:
        msg = "scan config error: no .rs files selected (need --files or --root)"
        print(json.dumps({"error": msg}) if args.json else msg, file=sys.stderr)
        return 2

    total_traced = 0
    all_viols = []
    for f in files:
        traced, viols = scan_file(f)
        total_traced += traced
        all_viols.extend(viols)

    if args.expect_traced and total_traced == 0:
        msg = (
            "R2 self-test FAIL: scan reached files but found zero #[traced_test] "
            "(evidence path dead)"
        )
        print(
            json.dumps({"error": msg, "files_scanned": len(files)})
            if args.json
            else msg,
            file=sys.stderr,
        )
        return 2

    if args.json:
        print(json.dumps({
            "files_scanned": len(files),
            "traced_tests": total_traced,
            "violations": all_viols,
            "clean": not all_viols,
        }, indent=2))
    else:
        for v in all_viols:
            loc = f"{v['file']}:{v.get('line', '?')}"
            print(f"BLOCK [{v['rule']}] {loc} {v.get('fn', '')}: {v['detail']}")
        print(
            f"scanned {len(files)} file(s), {total_traced} #[traced_test], "
            f"{len(all_viols)} violation(s)"
        )
    return 1 if all_viols else 0


if __name__ == "__main__":
    sys.exit(main())
