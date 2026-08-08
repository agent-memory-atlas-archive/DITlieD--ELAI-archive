#!/usr/bin/env python3
"""claims_policy_lint.py -- deterministic no-claim-without-a-number gate (K-1).

Enforces KD-1 (K-MASTER-BRIEF-2026-06-12.md 3.4, restated docs/strategy/
ELAI-CLAIMS-POLICY.md): no externally-visible claim (README, positioning
copy, a strategy doc) may assert a quantified number or a superlative
without EITHER

  (a) an [EXTERNAL] or [EXTERNAL, unverified] tag naming a third-party
      source (a KW-N pointer, a URL, a paper id), or
  (b) a [DESIGN] tag (an intentional, non-measured design choice, never
      presented as fact), or
  (c) a [MEASURED] tag or an adjacent recompute pointer (an `elai bench
      <subcmd>` invocation, a `file.rs:LINE` source anchor, or the literal
      word PENDING when the bench has not produced the number yet).

WHY THIS EXISTS: the B-40/B-41/B-42 bad-cite incidents (CLAUDE.md
feedback_paper_citation_verify) shipped external numbers into positioning
copy without a verifiable source. KD-1 is the doctrine; this lint is its
enforcement at the marketing surface, mirroring how wiring-enforcement.md
enforces "no claim without production evidence" at the code surface.

ADJACENCY IS PER-PARAGRAPH-BLOCK (blank-line-delimited), NOT WHOLE-FILE: a
tag or pointer ANYWHERE in the same blank-line-separated block clears every
number/superlative in that block, so a multi-line prose paragraph that
states a number and names its source two lines later still passes, and a
BLOCK with no tag anywhere still fails in full. RESIDUAL RISK (documented,
not silently assumed): a dense multi-claim block with NO blank lines
between distinct claims (e.g. several claims crammed into one markdown
table with no per-row blank line) can leak clearance from one row's tag to
a neighboring untagged row inside the SAME block. Authors of table-shaped
copy MUST put one claim per block (a blank line between rows, or one claim
per file) to get the intended per-claim enforcement; this is a stated
authoring constraint, not a lint bug the checker silently papers over.

Stdlib-only (matches the other .claude/tools/*.py). Deliberately narrow
pattern sets (numbers: %, x-multiplier, pp, $; superlatives: a short banned
list) per the K-1 plan's own risk note -- widen only after observing real
false positives, never speculatively (GD-4 posture for a text gate).

Usage:
  python claims_policy_lint.py --files a.md,b.md
  python claims_policy_lint.py --files docs/strategy/*.md --json

Exit codes: 0 = clean, 1 = >=1 violation (BLOCK, fail-closed per R1), 2 =
scan-config error (no --files given, a named file does not exist, or the
resolved file set is empty -- missing evidence is never read as PASS, R2).
"""
import argparse
import json
import re
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# pattern sets (narrow by design -- see module docstring)
# ---------------------------------------------------------------------------

NUMBER_PATTERNS = [
    ("PERCENT", re.compile(r"\b\d+(?:\.\d+)?\s?%")),
    ("MULTIPLIER", re.compile(r"\b\d+(?:\.\d+)?\s?[xX]\b")),
    ("PERCENTAGE_POINTS", re.compile(r"\b\d+(?:\.\d+)?\s?pp\b", re.IGNORECASE)),
    ("DOLLAR", re.compile(r"\$\d+(?:\.\d+)?\s?[MBKmbk]?\b")),
    ("N_TIMES", re.compile(r"\b\d+(?:\.\d+)?\s?times\b", re.IGNORECASE)),
]

# Narrow, curated banned-superlative list. Each is a case-insensitive
# literal phrase match (word-boundary-anchored where it matters), not a
# generic adjective ban -- ordinary prose ("the fastest way to check this
# in") must not trip it, hence "fastest" alone is excluded; only the
# claim-shaped compounds are banned.
SUPERLATIVE_PHRASES = [
    "the safest",
    "the best",
    "the fastest",
    "the most secure",
    "the most trusted",
    "industry-leading",
    "industry leading",
    "state-of-the-art",
    "state of the art",
    "unbeatable",
    "unmatched",
    "world-class",
    "world class",
    "best-in-class",
    "best in class",
    "number one",
    "#1",
    "guaranteed",
    "bulletproof",
    "flawless",
    "zero bugs",
    "never fails",
    "owasp certified",
    "fully covered",
    "fully protected",
]
# Mixed punctuation/multi-word phrases above (e.g. "#1", "state-of-the-art")
# do not tolerate a wrapping \b, so this matches the escaped phrase text
# directly rather than word-boundary-anchoring the whole alternation.
SUPERLATIVE_RE = re.compile(
    "(?i)(" + "|".join(re.escape(p) for p in SUPERLATIVE_PHRASES) + ")"
)

# Clearing markers: presence ANYWHERE in the same paragraph block (see
# module docstring ADJACENCY note) clears every number/superlative in it.
TAG_RE = re.compile(r"\[(EXTERNAL(?:,\s*unverified)?|DESIGN|MEASURED)\]", re.IGNORECASE)
POINTER_RE = re.compile(
    r"(`?elai (bench|wrap)\b)"       # a recompute CLI invocation
    r"|(\b[\w./\\-]+\.(?:rs|py|ts|svelte):\d+\b)"  # a source:line anchor
    r"|(\bPENDING\b)",               # explicit not-yet-measured marker
    re.IGNORECASE,
)

# OWASP coverage assertions must carry an explicit status or evidence marker.
OWASP_ID_RE = re.compile(r"\b(?:LLM(?:0[1-9]|10)|ASI(?:0[1-9]|10))\b", re.IGNORECASE)
OWASP_COVERAGE_RE = re.compile(
    r"\b(?:covered|protect(?:ed|s)?|secure|mitigat(?:ed|es|e)|defend(?:ed|s)?)\b",
    re.IGNORECASE,
)
OWASP_STATUS_RE = re.compile(
    r"\b(?:WIRED-MEASURED|WIRED-DESIGN|PARTIAL|PLANNED(?:\([^)]*\))?|N-A)\b",
    re.IGNORECASE,
)
OWASP_EVIDENCE_RE = re.compile(
    r"\[(?:DESIGN|MEASURED|EXTERNAL|BENCH-PENDING)\]|\bPENDING\b",
    re.IGNORECASE,
)


def find_claims(line: str):
    """Return list of (rule, match_text) for every number/superlative on this line."""
    findings = []
    for rule, pat in NUMBER_PATTERNS:
        m = pat.search(line)
        if m:
            findings.append((rule, m.group(0)))
    m = SUPERLATIVE_RE.search(line)
    if m:
        findings.append(("SUPERLATIVE", m.group(0)))
    return findings


def iter_blocks(lines: list[str]):
    """Yield (start_lineno, [lines]) for each blank-line-delimited paragraph block."""
    block: list[str] = []
    start = 1
    for i, line in enumerate(lines, start=1):
        if line.strip() == "":
            if block:
                yield start, block
                block = []
            start = i + 1
            continue
        if not block:
            start = i
        block.append(line)
    if block:
        yield start, block


def scan_file(path: Path):
    violations = []
    text = path.read_text(encoding="utf-8", errors="replace")
    lines = text.splitlines()
    for start, block in iter_blocks(lines):
        cleared = any(TAG_RE.search(bline) or POINTER_RE.search(bline) for bline in block)
        if cleared:
            continue
        for offset, bline in enumerate(block):
            for rule, matched in find_claims(bline):
                violations.append(
                    {
                        "file": str(path),
                        "line": start + offset,
                        "rule": rule,
                        "match": matched,
                        "text": bline.strip()[:200],
                    }
                )
            # A threat-id coverage assertion is a claim in its own right; do
            # not let an unrelated paragraph marker clear an untagged cell.
            if (OWASP_ID_RE.search(bline) or re.search(r"\bowasp\b", bline, re.IGNORECASE)) and OWASP_COVERAGE_RE.search(bline):
                if not OWASP_STATUS_RE.search(bline) and not OWASP_EVIDENCE_RE.search(bline):
                    violations.append({
                        "file": str(path),
                        "line": start + offset,
                        "rule": "OWASP_COVERAGE_TAG",
                        "match": OWASP_COVERAGE_RE.search(bline).group(0),
                        "text": bline.strip()[:200],
                    })
    return violations


def main() -> int:
    ap = argparse.ArgumentParser(description="K-1 claims-policy deterministic lint (KD-1 enforcement)")
    ap.add_argument("--files", help="comma-separated file paths to scan")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    if not args.files:
        payload = {"error": "no --files given", "verdict": "SCAN_CONFIG_ERROR"}
        print(json.dumps(payload) if args.json else payload["error"], file=sys.stderr)
        return 2

    raw_paths = [p.strip() for p in args.files.split(",") if p.strip()]
    if not raw_paths:
        payload = {"error": "--files resolved to zero paths", "verdict": "SCAN_CONFIG_ERROR"}
        print(json.dumps(payload) if args.json else payload["error"], file=sys.stderr)
        return 2

    resolved = []
    missing = []
    for raw in raw_paths:
        p = Path(raw)
        if p.exists() and p.is_file():
            resolved.append(p)
        else:
            missing.append(raw)

    if missing:
        payload = {
            "error": f"missing input file(s): {missing}",
            "verdict": "SCAN_CONFIG_ERROR",
        }
        print(json.dumps(payload) if args.json else payload["error"], file=sys.stderr)
        return 2

    if not resolved:
        payload = {"error": "no files to scan after resolution", "verdict": "SCAN_CONFIG_ERROR"}
        print(json.dumps(payload) if args.json else payload["error"], file=sys.stderr)
        return 2

    all_violations = []
    for p in resolved:
        all_violations.extend(scan_file(p))

    verdict = "BLOCK" if all_violations else "PASS"
    payload = {
        "verdict": verdict,
        "files_scanned": [str(p) for p in resolved],
        "violation_count": len(all_violations),
        "violations": all_violations,
    }

    if args.json:
        print(json.dumps(payload, indent=2))
    else:
        if all_violations:
            print(f"BLOCK: {len(all_violations)} uncleared claim(s):")
            for v in all_violations:
                print(f"  {v['file']}:{v['line']} [{v['rule']}] '{v['match']}' -- {v['text']}")
        else:
            print(f"PASS: {len(resolved)} file(s) clean")

    return 1 if all_violations else 0


if __name__ == "__main__":
    sys.exit(main())
