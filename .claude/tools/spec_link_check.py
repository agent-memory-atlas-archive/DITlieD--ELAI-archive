#!/usr/bin/env python3
"""
spec_link_check.py — verify "Spec references:" claims in a B-N or plan file.

CLI:
  python spec_link_check.py --file PATH [--specs-dir PATH] [--json]

Exit codes: 0 = all references valid, 1 = >=1 missing/invalid.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

DEFAULT_SPECS_DIR = r"F:\.ELAI_workflow\docs\specs\guide\steps"


def parse_references(text: str) -> List[str]:
    """Extract reference tokens from a 'Spec references:' line + continuations."""
    out: List[str] = []
    lines = text.splitlines()
    in_block = False
    block_indent = -1
    for raw in lines:
        stripped = raw.strip()
        m = re.match(r"(?i)spec\s+references?\s*:\s*(.*)$", stripped)
        if m:
            tail = m.group(1).strip()
            if tail:
                out.extend(_split_refs(tail))
            in_block = True
            block_indent = len(raw) - len(raw.lstrip())
            continue
        if in_block:
            cur_indent = len(raw) - len(raw.lstrip())
            if not stripped:
                continue
            if cur_indent <= block_indent and not stripped.startswith(("-", "*")):
                in_block = False
                continue
            cleaned = stripped.lstrip("-*").strip()
            if cleaned:
                out.extend(_split_refs(cleaned))
    return out


def parse_reads_block(text: str) -> List[str]:
    """Find '## Reads (mandatory before coding)' and harvest entries."""
    out: List[str] = []
    lines = text.splitlines()
    start: Optional[int] = None
    for i, ln in enumerate(lines):
        if re.match(r"(?i)^##\s+reads\s*\(mandatory\s+before\s+coding\)\s*$",
                    ln.strip()):
            start = i + 1
            break
    if start is None:
        return out
    for j in range(start, len(lines)):
        ln = lines[j]
        if re.match(r"^##\s+", ln.strip()):
            break
        stripped = ln.strip().lstrip("-*").strip()
        if not stripped:
            continue
        out.extend(_split_refs(stripped))
    return out


def _split_refs(s: str) -> List[str]:
    # Trim quoting/backticks; split on commas and whitespace, but preserve
    # path-like tokens.
    cleaned = s.replace("`", "")
    parts = re.split(r"[,;]\s*", cleaned)
    refs: List[str] = []
    for p in parts:
        p = p.strip()
        if not p:
            continue
        # Drop trailing punctuation.
        p = p.rstrip(".)")
        # Filter to tokens that look like file references.
        if re.search(r"\.(txt|md|rs|py|ts|svelte|toml|yaml|yml)\b", p,
                     re.IGNORECASE) or "/" in p or "\\" in p:
            refs.append(p)
    return refs


def split_path_range(ref: str) -> Tuple[str, Optional[Tuple[int, int]]]:
    """Split a token like 'phase-4D.txt:515-1215' into (path, (start, end))."""
    # Find the LAST colon that introduces a numeric range; preserve drive letters.
    m = re.match(r"^(.*?)(?::(\d+)(?:-(\d+))?)?$", ref)
    if not m:
        return ref, None
    path = m.group(1)
    start = m.group(2)
    end = m.group(3)
    # Re-attach drive letter if accidentally split.
    if path and re.fullmatch(r"[A-Za-z]", path) and start and ":" not in path:
        # ref looked like 'C:foo'; treat first colon as drive separator.
        return ref, None
    if not start:
        return path, None
    s = int(start)
    e = int(end) if end else s
    return path, (s, e)


def resolve_spec_path(ref_path: str, specs_dir: str) -> Optional[str]:
    candidates = [ref_path]
    if not os.path.isabs(ref_path):
        candidates.append(os.path.join(specs_dir, ref_path))
        candidates.append(os.path.join(specs_dir, os.path.basename(ref_path)))
    for c in candidates:
        if os.path.exists(c):
            return c
    return None


def check_one(ref: str, specs_dir: str) -> Dict[str, Any]:
    path, rng = split_path_range(ref)
    resolved = resolve_spec_path(path, specs_dir)
    if not resolved:
        return {"ref": ref, "status": "MISSING",
                "evidence": f"path not found near {specs_dir}"}
    if rng is None:
        # Just confirm exists, capture first 200 chars.
        try:
            with open(resolved, "r", encoding="utf-8", errors="replace") as fh:
                ev = fh.read(200).replace("\n", " ").replace("\t", " ")
            return {"ref": ref, "status": "PASS", "resolved": resolved,
                    "evidence": ev}
        except OSError as e:
            return {"ref": ref, "status": "MISSING", "evidence": str(e)}
    # Range case.
    s, e = rng
    if s < 1 or e < s:
        return {"ref": ref, "status": "INVALID_RANGE",
                "evidence": f"range {s}-{e}"}
    try:
        with open(resolved, "r", encoding="utf-8", errors="replace") as fh:
            lines = fh.readlines()
    except OSError as exc:
        return {"ref": ref, "status": "MISSING", "evidence": str(exc)}
    total = len(lines)
    if s > total:
        return {"ref": ref, "status": "INVALID_RANGE",
                "evidence": f"start {s} > file length {total}"}
    e_capped = min(e, total)
    snippet = "".join(lines[s - 1:e_capped])[:200]
    snippet = snippet.replace("\n", " ").replace("\t", " ")
    if not snippet.strip():
        return {"ref": ref, "status": "INVALID_RANGE",
                "evidence": f"range {s}-{e_capped} empty"}
    return {"ref": ref, "status": "PASS", "resolved": resolved,
            "evidence": snippet}


def render_text(file_path: str, specs_dir: str,
                spec_refs: List[Dict[str, Any]],
                reads: List[Dict[str, Any]]) -> str:
    lines: List[str] = []
    lines.append(f"SPEC LINK CHECK — {file_path}")
    lines.append(f"Specs dir: {specs_dir}")
    lines.append("")
    lines.append("Spec references:")
    if not spec_refs:
        lines.append("  (none found)")
    for r in spec_refs:
        lines.append(f"  {r['ref']} | {r['status']} | {r['evidence'][:120]}")
    lines.append("")
    lines.append("Reads (mandatory before coding):")
    if not reads:
        lines.append("  (none found)")
    for r in reads:
        lines.append(f"  {r['ref']} | {r['status']} | {r['evidence'][:120]}")
    all_results = spec_refs + reads
    pass_n = sum(1 for r in all_results if r["status"] == "PASS")
    miss_n = sum(1 for r in all_results if r["status"] == "MISSING")
    inv_n = sum(1 for r in all_results if r["status"] == "INVALID_RANGE")
    lines.append("")
    lines.append(f"TOTAL: {pass_n} pass, {miss_n} missing, {inv_n} invalid_range")
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser(description="Verify Spec references in a file")
    ap.add_argument("--file", required=True)
    ap.add_argument("--specs-dir", default=DEFAULT_SPECS_DIR)
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    if not os.path.exists(args.file):
        msg = f"ERROR: input file not found: {args.file}"
        if args.json:
            print(json.dumps({"error": msg}))
        else:
            print(msg, file=sys.stderr)
        return 1

    text = Path(args.file).read_text(encoding="utf-8", errors="replace")
    spec_refs = parse_references(text)
    reads = parse_reads_block(text)

    spec_results = [check_one(r, args.specs_dir) for r in spec_refs]
    reads_results = [check_one(r, args.specs_dir) for r in reads]

    pass_n = sum(1 for r in spec_results + reads_results if r["status"] == "PASS")
    miss_n = sum(1 for r in spec_results + reads_results if r["status"] == "MISSING")
    inv_n = sum(1 for r in spec_results + reads_results
                if r["status"] == "INVALID_RANGE")
    rc = 0 if (miss_n == 0 and inv_n == 0) else 1

    if args.json:
        print(json.dumps({
            "file": args.file,
            "specs_dir": args.specs_dir,
            "spec_references": spec_results,
            "reads_mandatory": reads_results,
            "summary": {"pass": pass_n, "missing": miss_n,
                        "invalid_range": inv_n},
        }, indent=2))
    else:
        print(render_text(args.file, args.specs_dir, spec_results, reads_results))
    return rc


if __name__ == "__main__":
    sys.exit(main())
