#!/usr/bin/env python3
"""slop_semantic_check.py -- off-agent checker for two slop classes the existing
deterministic gates are blind to but the merge-lane GLM review keeps catching:

  TT (tautological / source-grep tests): a test that asserts on SOURCE TEXT instead
      of behaviour -- e.g. include_str! a .rs then assert it .contains("fn foo"), or
      assert!(src.contains("pub ")). These go green without exercising anything; they
      are the providers-slice "assert source CONTAINS strings not behavior" finding.

  TV (trust-model violations): writing fabricated evidence past the integrity layer --
      PRAGMA ignore_check_constraints=ON, PRAGMA foreign_keys=OFF, or Python opening /
      writing the Rust-owned SSB db directly. This is the python-slice finding (worker
      wrote a fabricated True acceptance row into ssb.db via ignore_check_constraints).

WHY OFF-AGENT (wiring-enforcement.md R4/R8): an agent that reviews its own output
rubber-stamps. This is a diff-grounded artifact the executing agent did not author. It
is Layer-2 static, regex-unsound (necessary-not-sufficient); the load-bearing backstop
is the air-gapped GLM review + the auditor. It exists so the per-task gate catches these
at task time instead of letting them poison a whole crate slice at merge time.

Stdlib-only (matches the other .claude/tools/*.py). Mirrors maintainability_check.py's
interface so the slop_cleaner gate stage can invoke it identically.

Usage:
  python slop_semantic_check.py --files a.rs,b.py --base HEAD --json
  python slop_semantic_check.py --git-staged --json

Exit codes: 0 = clean, 1 = >=1 BLOCK finding, 2 = scan-config error (no files / git absent).
All findings are BLOCK (both classes are disqualifying, not style nits).

UNSOUNDNESS (documented): scans NET-ADDED diff lines with regex, not a parser. It can
false-negative on aliased / multi-line forms (an include_str! and its .contains() split
far apart in a huge diff) and could false-positive on production code that legitimately
greps source -- mitigated by requiring an assert context (TT-1) or a test-path heuristic
(TT-2). The GLM review is the semantic backstop.
"""
import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

SOURCE_EXTS = {".rs", ".py", ".ts", ".tsx", ".js", ".jsx", ".svelte"}

# --- TT: tautological / source-grep test patterns ----------------------------
# code-ish tokens whose presence inside a `.contains("...")` argument means the
# test is asserting on SOURCE TEXT, not on a runtime value.
_CODEISH = r"(?:pub\s|fn\s|struct\s|impl\s|trait\s|enum\s|mod\s|async\s|def\s|class\s|import\s|#\[|use\s)"
# TT-1: an assert on a `.contains(<code-ish literal>)` -- low-FP because it requires
# an assert context AND a source-keyword needle.
_TT1 = re.compile(
    r"assert\w*[!(].*\.contains\(\s*[rbf]?\"\s*" + _CODEISH,
    re.IGNORECASE,
)
# TT-2 components (test-path-gated): reading source then grepping it.
_INCLUDE_SRC = re.compile(r"""include_str!\(\s*[^)]*\.(?:rs|py|ts)["']""")
_READ_SRC = re.compile(
    r"""(?:read_to_string|fs::read|open\(|read_text)\([^)]*(?:\.rs|\.py|/src/)""",
    re.IGNORECASE,
)
_DOT_CONTAINS = re.compile(r"\.contains\(")

# --- TV: trust-model violation patterns (any language) -----------------------
_IGNORE_CONSTRAINTS = re.compile(r"ignore_check_constraints", re.IGNORECASE)
_FK_OFF = re.compile(r"foreign_keys\s*=\s*(?:off|0|false)", re.IGNORECASE)
# Python writing the Rust-owned SSB db directly (connect/INSERT/UPDATE against ssb.db).
_SSB_DB = re.compile(r"ssb\.db", re.IGNORECASE)
_DB_WRITE = re.compile(r"\b(?:INSERT\s+INTO|UPDATE\s+|REPLACE\s+INTO|connect\()", re.IGNORECASE)


def _is_test_path(path):
    s = str(path).replace("\\", "/").lower()
    segments = s.split("/")
    name = path.name.lower()
    return (
        "tests" in segments
        or "test" in segments
        or name.startswith("test_")
        or name.endswith("_test.rs")
        or name.endswith("_test.py")
        or name.endswith("_contract.rs")
        or "test" in name
    )


# --- git plumbing (mirrors maintainability_check.py) -------------------------
def _run_git(args):
    try:
        r = subprocess.run(["git"] + args, capture_output=True, text=True, check=False)
    except FileNotFoundError:
        return None, "", "git not on PATH"
    return r.returncode, r.stdout, (r.stderr or "").strip()


def _patch(base, cached, path, commit=None):
    args = ["diff", "--unified=0"]
    if commit:
        # Two-ref diff of a historical commit (sha~1..sha). READ-ONLY -- does not touch
        # the working tree, so the re-gate can scan integrated commits on a live repo.
        args += [f"{commit}~1", commit]
    elif cached:
        args.append("--cached")
    elif base:
        args.append(base)
    args += ["--", path]
    rc, out, _ = _run_git(args)
    return out if rc == 0 else ""


def _added_lines(patch):
    """NEW-file (line_no, text) for every '+' line in the patch."""
    out = []
    new_ln = 0
    hunk = re.compile(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@")
    for line in patch.splitlines():
        m = hunk.match(line)
        if m:
            new_ln = int(m.group(1))
            continue
        if line.startswith("+++") or line.startswith("---"):
            continue
        if line.startswith("+"):
            out.append((new_ln, line[1:]))
            new_ln += 1
        elif line.startswith("-"):
            continue
        else:
            new_ln += 1
    return out


def scan_file(path, base, cached, commit=None):
    findings = []
    if path.suffix.lower() not in SOURCE_EXTS:
        return findings
    patch = _patch(base, cached, str(path), commit=commit)
    if not patch:
        return findings
    added = _added_lines(patch)
    is_test = _is_test_path(path)
    saw_src_read = False
    saw_contains = False
    for ln, text in added:
        # TV: trust violations -- any file, hard block.
        if _IGNORE_CONSTRAINTS.search(text):
            findings.append({
                "file": str(path), "rule": "TV-1", "sev": "BLOCK", "line": ln,
                "detail": "PRAGMA ignore_check_constraints bypasses the SSB integrity layer "
                          "(fabricated-evidence vector); never write past CHECK constraints",
            })
        if _FK_OFF.search(text):
            findings.append({
                "file": str(path), "rule": "TV-2", "sev": "BLOCK", "line": ln,
                "detail": "foreign_keys=OFF disables referential integrity to force a write",
            })
        if path.suffix.lower() == ".py" and _SSB_DB.search(text) and _DB_WRITE.search(text):
            findings.append({
                "file": str(path), "rule": "TV-3", "sev": "BLOCK", "line": ln,
                "detail": "Python writing the Rust-owned SSB db (ssb.db) directly; the SSB is "
                          "single-writer Rust-owned -- emit via the real seam, never a side-write",
            })
        # TT-1: tautological assert on source text (assert + .contains(<code keyword>)).
        if _TT1.search(text):
            findings.append({
                "file": str(path), "rule": "TT-1", "sev": "BLOCK", "line": ln,
                "detail": "test asserts on SOURCE TEXT (.contains of a code keyword), not on "
                          "behaviour; assert a runtime value, not that the source contains a string",
            })
        # TT-2 components (gated to test files): include_str!/read of a source file + a .contains.
        if is_test:
            if _INCLUDE_SRC.search(text) or _READ_SRC.search(text):
                saw_src_read = True
            if _DOT_CONTAINS.search(text):
                saw_contains = True
    if is_test and saw_src_read and saw_contains:
        findings.append({
            "file": str(path), "rule": "TT-2", "sev": "BLOCK",
            "detail": "test reads a source file (include_str!/read_to_string of .rs/.py) and "
                      "greps it with .contains(); this is a source-grep test, not a behaviour test",
        })
    return findings


# --- DUP: new-file-is-a-near-copy-of-an-existing-sibling -----------------------
# The acceptance_evidence_<taskid>.rs class: a worker, told to "fix acceptance",
# copied the canonical module to a new per-task file instead of editing it -> 4
# near-duplicate 800-line modules. maintainability MR-1 misses it (each is < 1000
# lines) and a per-file diff scan can't see it (the dup is across files), so this
# needs the surrounding tree.
DUP_OVERLAP_BLOCK = 0.80      # an added file sharing >= this fraction of a sibling's lines = copy
DUP_NAMING_OVERLAP = 0.50     # lower bar when the NAME already screams a copy (foo_x.rs next to foo.rs)
_DUP_SUFFIX = re.compile(r"^(?P<stem>.+?)_(?P<suffix>[a-z]*\d[\w.-]*)$", re.IGNORECASE)


def _existed_at(ref, relpath):
    rc, _, _ = _run_git(["cat-file", "-e", f"{ref}:{relpath}"])
    return rc == 0


def _read_at(post_ref, relpath):
    if post_ref is None:
        try:
            return Path(relpath).read_text(encoding="utf-8", errors="replace")
        except OSError:
            return ""
    rc, out, _ = _run_git(["show", f"{post_ref}:{relpath}"])
    return out if rc == 0 else ""


def _list_siblings(reldir, post_ref):
    reldir = reldir or "."
    if post_ref is None:
        try:
            return [str((Path(reldir) / n).as_posix()) for n in __import__("os").listdir(reldir or ".")]
        except OSError:
            return []
    rc, out, _ = _run_git(["ls-tree", "--name-only", post_ref, "--",
                           (reldir + "/") if reldir != "." else ""])
    return [ln.strip() for ln in out.splitlines() if ln.strip()]


def _line_set(text):
    return {ln.strip() for ln in text.splitlines() if ln.strip()}


def _overlap(a, b):
    """Overlap coefficient |a&b| / min(|a|,|b|) -- ~1.0 when one file is a near-copy
    of (or a copy-with-additions of) the other."""
    if not a or not b:
        return 0.0
    return len(a & b) / min(len(a), len(b))


def scan_duplication(files, before_ref, post_ref):
    """For each NEW source file, flag it if it is a near-copy of an existing sibling in
    the same directory (DUP-1) or carries the foo_<id>.rs naming tell next to foo.rs
    (DUP-2). Works in worktree mode (post_ref=None, read from disk) and --commit mode
    (post_ref=sha, read via git)."""
    findings = []
    for path in files:
        rel = str(path).replace("\\", "/")
        if path.suffix.lower() not in SOURCE_EXTS:
            continue
        if _existed_at(before_ref, rel):
            continue  # not a new file -> editing an existing file is the GOOD path
        new_text = _read_at(post_ref, rel)
        new_lines = _line_set(new_text)
        if len(new_lines) < 40:
            continue  # too small to meaningfully duplicate an 800-line module
        reldir = str(path.parent).replace("\\", "/")
        stem, ext = path.stem, path.suffix
        m = _DUP_SUFFIX.match(stem)
        named_origin = f"{m.group('stem')}{ext}" if m else None
        for sib in _list_siblings(reldir, post_ref):
            sib_name = sib.rsplit("/", 1)[-1]
            if sib == rel or not sib_name.endswith(ext):
                continue
            if not _existed_at(before_ref, sib):
                continue  # only compare against files that already existed
            ov = _overlap(new_lines, _line_set(_read_at(post_ref, sib)))
            is_named = named_origin is not None and sib_name == named_origin
            if is_named and ov >= DUP_NAMING_OVERLAP:
                findings.append({
                    "file": rel, "rule": "DUP-2", "sev": "BLOCK",
                    "detail": f"new file is a copy of existing sibling {sib_name} (naming tell "
                              f"+ {ov:.0%} line overlap); EDIT {sib_name}, do not fork a per-task copy",
                })
                break
            if ov >= DUP_OVERLAP_BLOCK:
                findings.append({
                    "file": rel, "rule": "DUP-1", "sev": "BLOCK",
                    "detail": f"new file shares {ov:.0%} of its lines with existing {sib_name}; "
                              f"this is a near-duplicate module -- edit/extend the original instead",
                })
                break
    return findings


def collect_files(args):
    if args.files:
        names = [p.strip() for p in args.files.split(",") if p.strip()]
    else:
        rc, out, _ = _run_git(["diff", "--name-only", "--cached"])
        names = [ln.strip() for ln in out.splitlines() if ln.strip()] if rc == 0 else []
    return [Path(n) for n in names if Path(n).suffix.lower() in SOURCE_EXTS]


def main():
    ap = argparse.ArgumentParser(description="tautological-test + trust-violation deterministic checker")
    ap.add_argument("--files", help="comma-separated changed source files")
    ap.add_argument("--git-staged", action="store_true", help="files from git diff --cached")
    ap.add_argument("--base", default="HEAD",
                    help="git ref to diff against in --files mode (default HEAD)")
    ap.add_argument("--commit",
                    help="re-gate mode: scan a historical commit's diff (sha~1..sha), read-only")
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
        all_findings.extend(scan_file(f, base, cached, commit=args.commit))
    # DUP needs the surrounding tree (cross-file), so it runs as a separate pass.
    # before_ref = what existed pre-change; post_ref = where to read post-change content.
    if args.commit:
        before_ref, post_ref = f"{args.commit}~1", args.commit
    elif cached:
        before_ref, post_ref = "HEAD", None
    else:
        before_ref, post_ref = base, None
    all_findings.extend(scan_duplication(files, before_ref, post_ref))

    blocks = [x for x in all_findings if x["sev"] == "BLOCK"]
    if args.json:
        print(json.dumps({
            "files_scanned": len(files),
            "blocks": len(blocks), "warns": 0,
            "findings": all_findings,
            "clean": not all_findings,
        }, indent=2))
    else:
        for v in all_findings:
            loc = f"{v['file']}:{v['line']}" if v.get("line") else v["file"]
            print(f"{v['sev']} [{v['rule']}] {loc}: {v['detail']}")
        print(f"scanned {len(files)} file(s): {len(blocks)} BLOCK")
    return 1 if blocks else 0


if __name__ == "__main__":
    sys.exit(main())
