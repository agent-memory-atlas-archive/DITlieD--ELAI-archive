#!/usr/bin/env python3
"""maintainability_check.py -- auditor checker for .claude/rules/maintainability-review.md.

The DETERMINISTIC subset of the thermo-nuclear maintainability review (the
qualitative catalog lives in the rule + the auditor's REVIEW pass). This is the
off-agent, measure-the-diff leg: it runs over a CHANGESET (not the agent's own
self-review) and flags the mechanically-checkable code-entropy patterns LLM
coders default into -- file bloat, type-erasure escape hatches, over-long
functions a change touches, and spaghetti-branching density.

WHY OFF-AGENT (wiring-enforcement.md R4/R8): an agent that reviews its own
output rubber-stamps. thermo-nuclear's only teeth are linguistic (a demanding
persona), which is exactly that failure mode. This checker gives the AUDITOR a
diff-grounded artifact the executing agent did not author. It is necessary-not-
sufficient (Layer-2 static, regex-unsound); the qualitative catalog in the rule
is the rest. Both are auditor-applied, never executor-self-applied.

Stdlib-only (matches the other .claude/tools/*.py). --json for the auditor.

Usage:
  python maintainability_check.py --files a.rs,b.ts          # explicit changeset vs --base (default HEAD)
  python maintainability_check.py --git-staged               # files from `git diff --cached`
  python maintainability_check.py --files a.rs --base main   # diff against a ref
  python maintainability_check.py --git-staged --json
  python maintainability_check.py --files a.rs --warn-as-block  # presumptive-block mode (WARN also fails)

Exit codes: 0 = clean (or only WARN, default), 1 = >=1 BLOCK finding (or any
WARN with --warn-as-block), 2 = scan-config error (no files / git absent).

Severity split (only the robust patterns BLOCK; the noisy heuristics WARN):
  MR-1 file-size boundary crossing (<=MAX_FILE_LINES -> >MAX_FILE_LINES)   BLOCK
  MR-2 type-erasure escape hatch introduced (as any / @ts-ignore / unsafe) BLOCK
  MR-2 soft type/lint loosening introduced (: any / as <numeric> / allow)  WARN
  MR-3 a change touches/grows a function over MAX_FN_LINES                  WARN
  MR-4 net-added branching density in one file over ADDED_CONDITIONAL_WARN  WARN
  MR-1 already-large file (> MAX_FILE_LINES) grown by > BIG_FILE_GROWTH     WARN

UNSOUNDNESS (documented, per wiring-enforcement residual-risk discipline):
regex + brace/indent matcher, NOT a real parser. It can false-positive on a
pattern inside a string/comment (mitigated by a light strip) and false-negative
on aliased/macro-generated forms. The load-bearing backstop is the auditor's
qualitative REVIEW against the rule catalog; this only catches the mechanical
subset deterministically.

SELF-TEST / FIXTURE NOTE (auditor-run, not shipped as source):
  (a) a file diffed from 998 -> 1003 lines                      => exit 1, MR-1
  (b) a .ts diff adding `x as any`                              => exit 1, MR-2
  (c) a .rs diff adding `unsafe {`                              => exit 1, MR-2
  (d) a .ts diff adding `let x: any`                            => exit 0 + WARN
  (e) a clean small refactor                                    => exit 0
  (f) --warn-as-block over (d)                                  => exit 1
Drop these fixtures in a scratch git tree and confirm the exit codes before
trusting a live run.
"""
import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

# --- thresholds (named constants; no magic numbers per CLAUDE.md) -------------
MAX_FILE_LINES = 1000          # thermo-nuclear dimension #1: the 1k file boundary
BIG_FILE_GROWTH_WARN = 50      # an already-over-1k file grown by more than this
MAX_FN_LINES = 80              # a function body longer than this is a split smell
ADDED_CONDITIONAL_WARN = 10    # net-new branch keywords in one file = spaghetti smell

BRACE_LANGS = {".rs", ".ts", ".tsx", ".js", ".jsx", ".svelte"}
PY_LANGS = {".py"}
SOURCE_EXTS = BRACE_LANGS | PY_LANGS

# --- MR-2 escape-hatch patterns (matched on NET-ADDED diff lines) -------------
# (regex, severity, label). Net = added_hits - removed_hits; only positive flagged.
TS_EXTS = {".ts", ".tsx", ".js", ".jsx", ".svelte"}
ESCAPE_HATCHES = {
    "ts": [
        (re.compile(r"\bas\s+any\b"), "BLOCK", "`as any` type-erasure"),
        (re.compile(r"\bas\s+unknown\b"), "BLOCK", "`as unknown` type-erasure"),
        (re.compile(r"@ts-ignore\b"), "BLOCK", "@ts-ignore suppresses the type checker"),
        (re.compile(r"@ts-nocheck\b"), "BLOCK", "@ts-nocheck disables the type checker"),
        (re.compile(r":\s*any\b"), "WARN", "`: any` annotation muddies the contract"),
        (re.compile(r"<any>"), "WARN", "`<any>` cast muddies the contract"),
        (re.compile(r"eslint-disable"), "WARN", "eslint-disable suppresses a lint"),
    ],
    "rust": [
        (re.compile(r"\bunsafe\s*\{"), "BLOCK", "new `unsafe` block"),
        (re.compile(r"\bas\s+(?:u8|u16|u32|u64|u128|usize|i8|i16|i32|i64|i128|isize|f32|f64)\b"),
         "WARN", "`as` numeric cast (prefer .try_into() for narrowing)"),
        (re.compile(r"#\[allow\("), "WARN", "#[allow(..)] suppresses a lint"),
    ],
    "py": [
        (re.compile(r"#\s*type:\s*ignore"), "WARN", "# type: ignore suppresses mypy"),
        (re.compile(r"\bcast\s*\("), "WARN", "typing.cast() erases a real type"),
        (re.compile(r"#\s*noqa"), "WARN", "# noqa suppresses a lint"),
    ],
}

# --- MR-4 branch keywords per language family ---------------------------------
BRANCH_KW = {
    "ts": re.compile(r"\b(?:if|else\s+if|switch|case|catch)\b"),
    "rust": re.compile(r"\b(?:if|else\s+if|match)\b"),
    "py": re.compile(r"\b(?:if|elif|except)\b"),
}

# --- MR-3 function headers ----------------------------------------------------
RUST_FN = re.compile(r"\b(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_]\w*)\s*[<(]")
TS_FN = re.compile(
    r"\bfunction\s+([A-Za-z_]\w*)\s*\("
    r"|([A-Za-z_]\w*)\s*\([^;{}]*\)\s*(?::\s*[\w<>\[\],\s|&]+)?\s*\{"
)
PY_DEF = re.compile(r"^(\s*)def\s+([A-Za-z_]\w*)\s*\(")


def _lang_of(path: Path):
    ext = path.suffix.lower()
    if ext == ".rs":
        return "rust"
    if ext in TS_EXTS:
        return "ts"
    if ext in PY_LANGS:
        return "py"
    return None


def _strip_braces_noise(text):
    """Blank out // and /* */ comments and "..."/'...' string contents so a
    brace or keyword inside prose/strings does not skew brace-matching or counts.
    Preserves length + newline positions so line numbers stay correct. Light /
    unsound (no raw-string or char-literal awareness), same posture as the
    tracing-test checker's stripper."""
    out = []
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        nxt = text[i + 1] if i + 1 < n else ""
        if c == "/" and nxt == "/":
            j = text.find("\n", i)
            j = n if j == -1 else j
            out.append(" " * (j - i))
            i = j
            continue
        if c == "/" and nxt == "*":
            j = text.find("*/", i + 2)
            end = n if j == -1 else j + 2
            out.append("".join(ch if ch == "\n" else " " for ch in text[i:end]))
            i = end
            continue
        if c == "#":  # python / shell comment to EOL
            j = text.find("\n", i)
            j = n if j == -1 else j
            out.append(" " * (j - i))
            i = j
            continue
        if c in ('"', "'"):
            quote = c
            out.append(quote)
            i += 1
            while i < n:
                ch = text[i]
                if ch == "\\" and i + 1 < n:
                    out.append("  ")
                    i += 2
                    continue
                if ch == quote:
                    out.append(quote)
                    i += 1
                    break
                out.append("\n" if ch == "\n" else " ")
                i += 1
            continue
        out.append(c)
        i += 1
    return "".join(out)


# --- git plumbing -------------------------------------------------------------
def _run_git(args):
    try:
        r = subprocess.run(
            ["git"] + args, capture_output=True, text=True, check=False
        )
    except FileNotFoundError:
        return None, "", "git not on PATH"
    return r.returncode, r.stdout, (r.stderr or "").strip()


def _numstat(base, cached, path):
    """Return (added, removed) line counts for path, or (None, None) if binary
    or git failed."""
    args = ["diff", "--numstat"]
    if cached:
        args.append("--cached")
    elif base:
        args.append(base)
    args += ["--", path]
    rc, out, _ = _run_git(args)
    if rc != 0 or not out.strip():
        return None, None
    parts = out.split("\n")[0].split("\t")
    if len(parts) < 2 or parts[0] == "-" or parts[1] == "-":
        return None, None
    try:
        return int(parts[0]), int(parts[1])
    except ValueError:
        return None, None


def _patch(base, cached, path):
    args = ["diff", "--unified=0"]
    if cached:
        args.append("--cached")
    elif base:
        args.append(base)
    args += ["--", path]
    rc, out, _ = _run_git(args)
    return out if rc == 0 else ""


HUNK = re.compile(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@")


def _parse_patch(patch):
    """Return (added_line_nums, added_texts, removed_texts) where added_line_nums
    are NEW-file line numbers of '+' lines (for MR-3 range intersection)."""
    added_nums, added_texts, removed_texts = [], [], []
    new_ln = 0
    for line in patch.splitlines():
        m = HUNK.match(line)
        if m:
            new_ln = int(m.group(1))
            continue
        if line.startswith("+++") or line.startswith("---"):
            continue
        if line.startswith("+"):
            added_nums.append(new_ln)
            added_texts.append(line[1:])
            new_ln += 1
        elif line.startswith("-"):
            removed_texts.append(line[1:])
        else:
            new_ln += 1
    return added_nums, added_texts, removed_texts


# --- function-range extraction (MR-3) ----------------------------------------
def _brace_fn_ranges(text):
    """[(name, start_line, end_line, body_lines)] for brace-delimited fns."""
    stripped = _strip_braces_noise(text)
    ranges = []
    for m in list(RUST_FN.finditer(stripped)) + list(TS_FN.finditer(stripped)):
        name = next((g for g in m.groups() if g), "<fn>")
        brace = stripped.find("{", m.start())
        if brace == -1:
            continue
        depth, i = 0, brace
        while i < len(stripped):
            ch = stripped[i]
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        start_line = stripped.count("\n", 0, m.start()) + 1
        end_line = stripped.count("\n", 0, i) + 1
        ranges.append((name, start_line, end_line, end_line - start_line + 1))
    return ranges


def _py_fn_ranges(text):
    lines = text.splitlines()
    stripped = _strip_braces_noise(text).splitlines()
    ranges = []
    for idx, raw in enumerate(stripped):
        m = PY_DEF.match(raw)
        if not m:
            continue
        indent = len(m.group(1))
        name = m.group(2)
        end = idx
        for j in range(idx + 1, len(lines)):
            s = stripped[j]
            if not s.strip():
                continue
            cur_indent = len(s) - len(s.lstrip())
            if cur_indent <= indent:
                break
            end = j
        ranges.append((name, idx + 1, end + 1, end - idx + 1))
    return ranges


def _fn_ranges(path, text):
    return _py_fn_ranges(text) if path.suffix.lower() in PY_LANGS else _brace_fn_ranges(text)


# --- per-file evaluation ------------------------------------------------------
def _net_hits(patterns, added_texts, removed_texts):
    """For each (regex, sev, label): net = sum(added matches) - sum(removed
    matches). Yield (sev, label, net) for net > 0."""
    out = []
    for rx, sev, label in patterns:
        add = sum(len(rx.findall(t)) for t in added_texts)
        rem = sum(len(rx.findall(t)) for t in removed_texts)
        if add - rem > 0:
            out.append((sev, label, add - rem))
    return out


def scan_file(path, base, cached):
    findings = []
    lang = _lang_of(path)
    if lang is None:
        return findings
    added, removed = _numstat(base, cached, str(path))
    patch = _patch(base, cached, str(path))
    added_nums, added_texts, removed_texts = _parse_patch(patch)

    # MR-1: file-size boundary.
    try:
        post = sum(1 for _ in path.open("r", encoding="utf-8", errors="replace"))
    except OSError:
        post = None
    if post is not None and added is not None and removed is not None:
        pre = post - added + removed
        if pre <= MAX_FILE_LINES < post:
            findings.append({
                "file": str(path), "rule": "MR-1", "sev": "BLOCK",
                "detail": f"this change pushes the file across the {MAX_FILE_LINES}-line "
                          f"boundary ({pre} -> {post}); split new code into a focused module",
            })
        elif pre > MAX_FILE_LINES and added > BIG_FILE_GROWTH_WARN:
            findings.append({
                "file": str(path), "rule": "MR-1", "sev": "WARN",
                "detail": f"already-large file ({pre} lines) grown by {added} lines; "
                          f"prefer extracting rather than appending",
            })

    # MR-2: type-erasure escape hatches introduced by the diff.
    for sev, label, net in _net_hits(ESCAPE_HATCHES[lang], added_texts, removed_texts):
        findings.append({
            "file": str(path), "rule": "MR-2", "sev": sev,
            "detail": f"diff introduces {net}x {label}",
        })

    # MR-4: net-added branching density.
    kw = BRANCH_KW[lang]
    add_branch = sum(len(kw.findall(_strip_braces_noise(t))) for t in added_texts)
    rem_branch = sum(len(kw.findall(_strip_braces_noise(t))) for t in removed_texts)
    if add_branch - rem_branch > ADDED_CONDITIONAL_WARN:
        findings.append({
            "file": str(path), "rule": "MR-4", "sev": "WARN",
            "detail": f"net +{add_branch - rem_branch} branch keywords in one diff "
                      f"(spaghetti smell; consider a typed model or dispatcher)",
        })

    # MR-3: a touched function over the length boundary.
    if post is not None and added_nums:
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            text = ""
        added_set = set(added_nums)
        for name, start, end, body_lines in _fn_ranges(path, text):
            if body_lines > MAX_FN_LINES and any(start <= ln <= end for ln in added_set):
                findings.append({
                    "file": str(path), "rule": "MR-3", "sev": "WARN", "line": start,
                    "detail": f"fn `{name}` is {body_lines} lines (> {MAX_FN_LINES}) and this "
                              f"change touches it; extract a helper / split the function",
                })
    return findings


def collect_files(args):
    if args.files:
        names = [p.strip() for p in args.files.split(",") if p.strip()]
    else:  # --git-staged (default)
        rc, out, _ = _run_git(["diff", "--name-only", "--cached"])
        names = [ln.strip() for ln in out.splitlines() if ln.strip()] if rc == 0 else []
    return [Path(n) for n in names if Path(n).suffix.lower() in SOURCE_EXTS]


def main():
    ap = argparse.ArgumentParser(description="thermo-nuclear maintainability deterministic checker")
    ap.add_argument("--files", help="comma-separated changed source files")
    ap.add_argument("--git-staged", action="store_true", help="files from git diff --cached")
    ap.add_argument("--base", default="HEAD",
                    help="git ref to diff against in --files mode (default HEAD; ignored with --git-staged)")
    ap.add_argument("--warn-as-block", action="store_true",
                    help="presumptive-block mode: any WARN also exits 1")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    rc, _, err = _run_git(["rev-parse", "--is-inside-work-tree"])
    if rc is None:
        print(json.dumps({"error": err}) if args.json else f"error: {err}", file=sys.stderr)
        return 2

    files = collect_files(args)
    if not files:
        msg = "scan config error: no source files selected (need --files or staged changes)"
        print(json.dumps({"error": msg}) if args.json else msg, file=sys.stderr)
        return 2

    cached = args.git_staged or not args.files
    base = None if cached else args.base
    all_findings = []
    for f in files:
        all_findings.extend(scan_file(f, base, cached))

    blocks = [x for x in all_findings if x["sev"] == "BLOCK"]
    warns = [x for x in all_findings if x["sev"] == "WARN"]

    if args.json:
        print(json.dumps({
            "files_scanned": len(files),
            "blocks": len(blocks), "warns": len(warns),
            "findings": all_findings,
            "clean": not all_findings,
        }, indent=2))
    else:
        for v in all_findings:
            loc = f"{v['file']}:{v['line']}" if v.get("line") else v["file"]
            print(f"{v['sev']} [{v['rule']}] {loc}: {v['detail']}")
        print(f"scanned {len(files)} file(s): {len(blocks)} BLOCK, {len(warns)} WARN")

    fail = bool(blocks) or (args.warn_as_block and bool(warns))
    return 1 if fail else 0


if __name__ == "__main__":
    sys.exit(main())
