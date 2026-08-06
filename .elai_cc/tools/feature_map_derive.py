#!/usr/bin/env python3
"""Derive mechanical feature-map facts without modifying the map."""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

_STATUS_PATTERNS = {
    "WIRED": re.compile(r"\[WIRED\]", re.I),
    "DARK": re.compile(r"\[DARK(?:\([^]]*\))?\]", re.I),
    "PARTIAL": re.compile(r"\[PARTIAL\]", re.I),
    "PLANNED": re.compile(r"\[PLANNED(?:\([^]]*\))?\]", re.I),
    "DEFERRED": re.compile(r"\[DEFERRED\]", re.I),
    "BENCH_PENDING": re.compile(r"\[BENCH-PENDING\]", re.I),
}


def _section_one(text):
    match = re.search(r"(?im)^\s*(?:#+\s*)?SECTION\s+1\b.*$", text)
    if not match:
        return text
    tail = text[match.end():]
    end = re.search(r"(?im)^\s*(?:#+\s*)?SECTION\s+2\b.*$", tail)
    return tail[:end.start()] if end else tail


def _derive_schema(repo_root):
    ssb = repo_root / ".elai_cc/crates/orchestrator/src/ssb.rs"
    lock = repo_root / ".elai_cc/crates/orchestrator/tests/action_type_lock.rs"
    ss = ssb.read_text(encoding="utf-8-sig") if ssb.exists() else ""
    epoch_match = re.search(r"\bSCHEMA_VERSION_DDL_EPOCH\s*:\s*i64\s*=\s*(-?\d+)", ss)
    epoch = int(epoch_match.group(1)) if epoch_match else 0
    array_match = re.search(r"\bSSB_ACTION_TYPES\s*:\s*&\s*\[\s*SsbActionType\s*\]\s*=\s*&\s*\[", ss)
    length = 0
    source_line = 1
    if array_match:
        source_line = ss.count("\n", 0, array_match.start()) + 1
        close = ss.find("];", array_match.end())
        body = ss[array_match.end(): close if close >= 0 else len(ss)]
        length = len(re.findall(r"\bSsbActionType\s*::", body))
    lock_text = lock.read_text(encoding="utf-8-sig") if lock.exists() else ""
    total_line_match = re.search(r"(?m)^.*\bTOTAL_VARIANTS\b.*$", lock_text)
    lock_line = lock_text.count("\n", 0, total_line_match.start()) + 1 if total_line_match else 1
    return {"ddl_epoch": epoch, "ssb_action_types_len": length,
            "schema_version": epoch + length, "total_variants": length,
            "sources": {"ssb_rs": f"{ssb}:{source_line}",
                        "action_type_lock": f"{lock}:{lock_line}"}}


def _status_counts(section):
    counts = {key: len(pattern.findall(section)) for key, pattern in _STATUS_PATTERNS.items()}
    bracket_count = len(re.findall(r"\[(?:WIRED|DARK(?:\([^]]*\))?|PARTIAL|PLANNED(?:\([^]]*\))?|DEFERRED|BENCH-PENDING)\]", section, re.I))
    counts["other"] = max(0, bracket_count - sum(counts.values()))
    return counts


def _stated_counts(text):
    found = {}
    labels = {"WIRED": "WIRED", "DARK": "DARK", "PARTIAL": "PARTIAL", "PLANNED": "PLANNED", "DEFERRED": "DEFERRED", "BENCH_PENDING": "BENCH[-_ ]PENDING"}
    for key, label in labels.items():
        matches = list(re.finditer(rf"(?i)\b{label}\b\s*(?:[:=]|[-–])\s*(\d+)\b|\b(\d+)\s+{label}\b", text))
        if matches:
            nums = [int(m.group(1) or m.group(2)) for m in matches]
            found[key] = nums[-1]
    return found


def _edges(section):
    edges = set()
    # Explicit relationship prose: consumer first, dependency second.
    for line in section.splitlines():
        for pattern, kind in ((r"(?i)([^\n:]+?)\s+EXTENDS\s+([^\n;,]+)", "EXTENDS"),
                              (r"(?i)([^\n:]+?)\s+composes\s+(?:with\s+)?([^\n;,]+)", "COMPOSES"),
                              (r"(?i)([^\n:]+?)\s+GATED[- ]on\s+([^\n;,]+)", "GATED-on"),
                              (r"(?i)([^\n:]+?)\s+ACTIVATES\s+([^\n;,]+)", "ACTIVATES")):
            for match in re.finditer(pattern, line):
                consumer, dependency = (x.strip(" #[]()\t") for x in match.groups())
                if consumer and dependency and len(consumer) < 160 and len(dependency) < 160:
                    edges.add((consumer, dependency, kind))
    return [{"consumer": c, "dependency": d, "kind": k} for c, d, k in sorted(edges)]


def _i39(repo_root, section):
    finish = repo_root / "workflow/sessions/s-plan-audit-2026-06-04/FINISH"
    files = sorted(f for f in finish.rglob("*") if f.is_file()) if finish.is_dir() else []
    verdicts = []
    for path in files:
        text = path.read_text(encoding="utf-8", errors="replace")
        for line in text.splitlines():
            match = re.search(r"(?i)\bVERDICT\s*:\s*(.+)", line)
            if match:
                verdicts.append((path.name, match.group(1).strip().upper(), text))
    disagreements = []
    units = [(unit, "planned") for unit in re.findall(r"\[PLANNED\(([^)]+)\)\]", section, re.I)]
    units.extend((unit, "wired") for unit in re.findall(r"\[WIRED\].*?\b([A-Z]-\d+[A-Za-z0-9.-]*)\b", section))
    for unit, map_status in units:
        unit_re = re.escape(unit)
        relevant = [(name, verdict) for name, verdict, _ in verdicts if re.search(unit_re, name, re.I)]
        if map_status == "planned" and any(re.search(r"\b(?:WIRED|PASS)\b", verdict) for _, verdict in relevant):
            disagreements.append({"flag": "planned_but_finish_wired", "detail": f"{unit} has WIRED/PASS FINISH evidence"})
        if map_status == "wired" and any(re.search(r"\bFAIL(?:ED)?\b", verdict) for _, verdict in relevant):
            disagreements.append({"flag": "wired_but_finish_fail", "detail": f"{unit} has FAIL FINISH evidence"})
    return {"available": finish.is_dir(), "verdict_files": len(verdicts)}, disagreements


def derive_report(repo_root, feature_map, elai_map=None):
    if not feature_map.is_file():
        raise FileNotFoundError(f"feature map not found: {feature_map}")
    text = feature_map.read_text(encoding="utf-8", errors="replace")
    section = _section_one(text)
    schema = _derive_schema(repo_root)
    counts = _status_counts(section)
    disagreements = []
    stamps = []
    for pattern in (r"(?i)\bSCHEMA_VERSION\b\s*[:=~]?\s*(-?\d+)", r"(?i)\bLIVE\s+SCHEMA\b\s*[:=~]?\s*(-?\d+)", r"(?i)\bSCHEMA\s*~\s*(-?\d+)"):
        stamps.extend(int(m.group(1)) for m in re.finditer(pattern, text))
    tv_stamps = [int(m.group(1)) for m in re.finditer(r"(?i)\b(?:TOTAL_VARIANTS|TV)\s*~?\s*[:=]?\s*(\d+)", text)]
    if any(value != schema["schema_version"] for value in stamps) or any(value != schema["total_variants"] for value in tv_stamps):
        disagreements.append({"flag": "schema_stamp_stale", "detail": f"map schema={stamps}, tv={tv_stamps}; live schema={schema['schema_version']}, tv={schema['total_variants']}"})
    stated = _stated_counts(text)
    mismatches = {key: {"parsed": counts[key], "stated": value} for key, value in stated.items() if value != counts[key]}
    if mismatches:
        disagreements.append({"flag": "status_counter_mismatch", "detail": mismatches})
    edges = _edges(section)
    map_path = elai_map or repo_root / ".claude/tools/elai.map"
    map_present = map_path.is_file()
    map_lines = len(map_path.read_text(encoding="utf-8", errors="replace").splitlines()) if map_present else 0
    i39, i39_disagreements = _i39(repo_root, section)
    disagreements.extend(i39_disagreements)
    return {"tool": "feature_map_derive", "plan": "J-93-LEG-G", "repo_root": str(repo_root), "feature_map": str(feature_map), "schema": schema, "status_counts": counts, "edges": edges, "elai_map_present": map_present, "elai_map_line_count": map_lines, "i39_fragments": i39, "disagreements": disagreements, "verdict": "OK" if not disagreements else "DISAGREE"}


def main(argv=None):
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path)
    parser.add_argument("--feature-map", type=Path)
    parser.add_argument("--elai-map", type=Path)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--json-only", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args(argv)
    repo = args.repo_root.resolve() if args.repo_root else Path(__file__).resolve().parents[2]
    fmap = (args.feature_map or repo / "docs/thefinalglance/ELAI-FEATURE-MAP.md").resolve()
    try:
        report = derive_report(repo, fmap, args.elai_map.resolve() if args.elai_map else None)
    except (OSError, UnicodeError, FileNotFoundError) as exc:
        print(f"feature_map_derive: {exc}", file=sys.stderr)
        return 2
    print(json.dumps(report, sort_keys=True, indent=2))
    if not args.json_only:
        print(f"feature_map_derive: {report['verdict']} ({len(report['disagreements'])} disagreement(s))", file=sys.stderr)
    return 1 if args.check and report["disagreements"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
