#!/usr/bin/env python3
"""entropy_rider_scan.py -- J-14 rider #7: observe-only CI lint for
unseeded RNG / uuid / wall-clock in deterministic-path modules.

JD-3: observe-only first. Exit 0 ALWAYS (even with findings); prints
findings to stdout for CI to surface without failing the build.
Allowlist: scripts/entropy_rider_allowlist.toml (file-based, auditable).

Patterns: thread_rng, Uuid::new_v4, uuid::Uuid::new_v4, rand::random,
          SystemTime::now, Instant::now, tokio::time::Instant::now,
          chrono::Utc::now, chrono::Local::now, OsRng::new,
          SeedableRng::from_entropy
(mirrors scripts/check_entropy_quarantine.sh PATTERN)

Filters (skip a hit if any applies):
  - line has trailing // entropy-allow: <reason> AND reason is in allowlist
  - line has trailing // TEST_KEEP
  - line is a doc-comment (/// or //) -- not code
  - file path matches an entry in allowlist [paths]
  - file name is clock.rs, entropy.rs, measure.rs, or bench_*.rs
  - directory named tests/, test/, examples/, benches/, .bkp_old/ is
    skipped during recursive scan (does not apply to the explicit --path root)

Usage:
  python3 scripts/entropy_rider_scan.py --path crates/orchestrator/src,crates/cli/src
  python3 scripts/entropy_rider_scan.py --path crates/orchestrator/src --json
  python3 scripts/entropy_rider_scan.py --path crates/orchestrator/src --allowlist scripts/entropy_rider_allowlist.toml

Exit codes: 0 always (observe-only, JD-3); 2 = scan-config error
(missing --path, path not found, allowlist parse error).
"""
import argparse
import json
import re
import sys
import tomllib
from dataclasses import dataclass, asdict
from pathlib import Path

PATTERN = re.compile(
    r"thread_rng|Uuid::new_v4|uuid::Uuid::new_v4|rand::random\b|"
    r"SystemTime::now|Instant::now|tokio::time::Instant::now|"
    r"chrono::Utc::now|chrono::Local::now|OsRng::new\b|"
    r"SeedableRng::from_entropy"
)
ENTROPY_ALLOW_RE = re.compile(r"//\s*entropy-allow:\s*(\S+)")
TEST_KEEP_RE = re.compile(r"//\s*TEST_KEEP")
DOC_COMMENT_RE = re.compile(r"^\s*///|^\s*//[^/]")

DIR_EXCLUDES = {"tests", "test", "examples", "benches", ".bkp_old"}
FILE_EXCLUDES = {"clock.rs", "entropy.rs", "measure.rs"}
FILE_EXCLUDE_PREFIXES = ("bench_",)


@dataclass
class Finding:
    file: str
    line: int
    pattern: str
    reason: str


def load_allowlist(path: Path):
    with open(path, "rb") as f:
        data = tomllib.load(f)
    reasons = set(data.get("reasons", []))
    paths = set(data.get("paths", []))
    return reasons, paths


def is_file_excluded(p: Path) -> bool:
    if p.name in FILE_EXCLUDES:
        return True
    if any(p.name.startswith(prefix) for prefix in FILE_EXCLUDE_PREFIXES):
        return True
    return False


def is_path_allowlisted(p: Path, allowlist_paths: set) -> bool:
    sp = str(p)
    for ap in allowlist_paths:
        if sp == ap or sp.endswith("/" + ap) or p.name == ap:
            return True
    return False


def scan_file(path: Path, reasons: set, allowlist_paths: set) -> list:
    findings = []
    if is_file_excluded(path):
        return findings
    if is_path_allowlisted(path, allowlist_paths):
        return findings
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return findings
    for i, line in enumerate(text.splitlines(), start=1):
        m = PATTERN.search(line)
        if not m:
            continue
        if DOC_COMMENT_RE.match(line):
            continue
        if TEST_KEEP_RE.search(line):
            continue
        am = ENTROPY_ALLOW_RE.search(line)
        if am and am.group(1) in reasons:
            continue
        findings.append(Finding(str(path), i, m.group(0), "unannotated"))
    return findings


def scan_path(target: str, reasons: set, allowlist_paths: set) -> list:
    p = Path(target)
    if p.is_file() and p.suffix == ".rs":
        return scan_file(p, reasons, allowlist_paths)
    if p.is_dir():
        results = []
        for rs in sorted(p.rglob("*.rs")):
            rel_parts = rs.relative_to(p).parts
            if any(part in DIR_EXCLUDES for part in rel_parts[:-1]):
                continue
            results.extend(scan_file(rs, reasons, allowlist_paths))
        return results
    print(f"Error: path not found: {target}", file=sys.stderr)
    sys.exit(2)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--path", required=True, help="comma-separated .rs paths")
    ap.add_argument(
        "--allowlist",
        default="scripts/entropy_rider_allowlist.toml",
    )
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    try:
        reasons, allowlist_paths = load_allowlist(Path(args.allowlist))
    except (OSError, tomllib.TOMLDecodeError) as e:
        print(f"Error: allowlist load failed: {e}", file=sys.stderr)
        sys.exit(2)
    findings = []
    for target in args.path.split(","):
        findings.extend(scan_path(target.strip(), reasons, allowlist_paths))
    if args.json:
        print(json.dumps([asdict(f) for f in findings], indent=2))
    else:
        for f in findings:
            print(f"{f.file}:{f.line}: {f.pattern} ({f.reason})")
        print(
            f"\nSummary: {len(findings)} finding(s) -- observe-only, exit 0 (JD-3)"
        )
    sys.exit(0)


if __name__ == "__main__":
    main()
