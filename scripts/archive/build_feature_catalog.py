#!/usr/bin/env python3
"""Build a deterministic, historical feature catalog from ELAI plan documents.

The exhaustive feature list is the primary inventory.  Original PHASE and B-Q
plan files are used to repair source coverage and preserve document links.  The
script deliberately reads headings and first non-empty source lines only; it
does not infer implementation state from plan bodies.
Output goes to the canonical catalog and RESEARCH-OVERVIEW.md; the short root
README is maintained separately.

Stdlib-only usage from the repository root::

    python3 scripts/archive/build_feature_catalog.py
    python3 scripts/archive/build_feature_catalog.py --check
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional


FAMILIES = ("PHASE",) + tuple("BCDEFGHIJKLMNOPQ")
LETTER_FAMILIES = tuple("BCDEFGHIJKLMNOPQ")
EXHAUSTIVE_RELATIVE = Path("docs/thefinalglance/ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md")
CATALOG_RELATIVE = Path("docs/FEATURE-CATALOG.md")
README_RELATIVE = Path("RESEARCH-OVERVIEW.md")
README_START = "<!-- FEATURE-CATALOG:START -->"
README_END = "<!-- FEATURE-CATALOG:END -->"

SYSTEM_RE = re.compile(r"^## System:\s+(PHASE|[B-Q])\b")
HEADING_RE = re.compile(r"^(#{1,6})\s+(.+?)\s*$")
SOURCE_ID_RE = re.compile(
    r"^([B-Q]-\d+(?:\.\d+)?[a-z]?(?:-[A-Z]+-\d+)?)_.*\.md$"
)
STATUS_WORD_RE = re.compile(
    r"\b(?:shipped|done|planned|partial|deferred|wire[- ]dark|dark|"
    r"build|activation|demand[- ]gated|not planned|runpod|no gpu|"
    r"post[- ]mvp|future|reserved|rejected|refuted|veto)\b",
    re.IGNORECASE,
)
BRACKET_RE = re.compile(r"\s*\[[^\]]*\]")
PAREN_STATUS_RE = re.compile(
    r"\s*\([^)]*\b(?:deferred|planned|shipped|done|partial|wire[- ]dark|"
    r"post[- ]mvp|future|reserved)\b[^)]*\)",
    re.IGNORECASE,
)
ABSOLUTE_PATH_RE = re.compile(
    r"(?:(?:[A-Za-z]:[\\/])|(?:/(?:home|mnt|Users|workspaces?|workspace)/))[^\s`),;]+",
    re.IGNORECASE,
)


@dataclass(frozen=True)
class Heading:
    family: str
    unit_id: str
    raw_title: str
    line: int
    source: str = "exhaustive"


@dataclass(frozen=True)
class SourceDoc:
    family: str
    source_id: str
    path: Path
    title: str
    first_line: int
    is_index: bool
    has_markdown_heading: bool


@dataclass
class CatalogEntry:
    family: str
    unit_id: str
    phrase: str
    exhaustive_id: Optional[str] = None
    exhaustive_line: Optional[int] = None
    original_docs: list[SourceDoc] = field(default_factory=list)
    source_only: bool = False
    kind: str = "unit"
    notes: list[str] = field(default_factory=list)


@dataclass(frozen=True)
class ExcludedHeading:
    family: str
    label: str
    line: int
    reason: str


@dataclass
class CatalogData:
    repo_root: Path
    exhaustive_path: Path
    entries: list[CatalogEntry]
    primary_headings: list[Heading]
    excluded_headings: list[ExcludedHeading]
    source_docs: list[SourceDoc]
    source_only_docs: list[SourceDoc]
    structural_source_docs: list[SourceDoc]
    aliases: list[tuple[str, str, str]]
    duplicate_primary_ids: list[str]
    duplicate_source_mappings: list[str]
    unparseable_records: list[str]
    plain_title_records: list[str]
    summary_files: dict[str, list[Path]]
    summary_missing_mentions: dict[str, list[str]]
    summary_missing_entries: dict[str, list[str]]
    summary_mention_counts: dict[str, tuple[int, int]]
    summary_entry_counts: dict[str, tuple[int, int]]
    sequence_gaps: dict[str, list[str]]
    raw_level3_headings: int
    outside_system_level3_headings: int


def repo_relative(path: Path, repo_root: Path) -> str:
    """Return a stable POSIX path relative to the repository root."""

    return path.resolve().relative_to(repo_root.resolve()).as_posix()


def link_path(repo_root: Path, target: Path, virtual_from: Path) -> str:
    """Build a link relative to a rendered document's logical repository path."""

    target_abs = target.resolve()
    from_abs = (repo_root / virtual_from).resolve()
    return Path(os.path.relpath(target_abs, from_abs.parent)).as_posix()


def scrub_private_paths(value: str) -> str:
    """Prevent source prose from leaking machine-specific absolute paths."""

    return ABSOLUTE_PATH_RE.sub("[path]", value)


def concise_phrase(value: str) -> str:
    """Normalize a heading into a short, status-free historical phrase."""

    phrase = scrub_private_paths(value)
    phrase = BRACKET_RE.sub("", phrase)
    phrase = PAREN_STATUS_RE.sub("", phrase)
    if " — " in phrase:
        left, right = phrase.split(" — ", 1)
        if STATUS_WORD_RE.search(right):
            phrase = left
    phrase = phrase.replace("**", "").replace("__", "")
    phrase = re.sub(r"\s+", " ", phrase).strip(" .:;—–-")
    if not phrase:
        return "Untitled historical entry"
    max_length = 180
    if len(phrase) > max_length:
        shortened = phrase[: max_length - 1].rsplit(" ", 1)[0].rstrip(" .,:;—–-")
        phrase = f"{shortened}…"
    return phrase


def split_heading(raw: str) -> tuple[str, str]:
    """Split a unit heading at its first colon or em-dash separator."""

    if ":" in raw:
        unit_id, title = raw.split(":", 1)
        return unit_id.strip(), title.strip()
    for separator in (" — ", " – ", " - "):
        if separator in raw:
            unit_id, title = raw.split(separator, 1)
            return unit_id.strip(), title.strip()
    return raw.strip(), raw.strip()


def is_catalog_heading(family: str, unit_id: str) -> bool:
    """Recognize the meaningful unit/entry headings counted by the source list."""

    if family == "PHASE":
        return bool(
            re.fullmatch(r"(?:phase-[A-Za-z0-9]+(?:-[A-Za-z0-9]+)*|pre-phase-0)", unit_id)
            or re.fullmatch(r"[1-4][A-Z](?:-[A-Z0-9]+)?", unit_id)
            or unit_id == "IDX"
        )

    numeric = rf"{re.escape(family)}-\d+(?:\.\d+)?[a-z]?\+?"
    if family == "B":
        return bool(
            re.fullmatch(numeric, unit_id)
            or unit_id in {"B-wide", "B-cross-cutting", "B-registers"}
        )
    if family == "C" or family == "D" or family == "J" or family == "P":
        return bool(re.fullmatch(numeric, unit_id))
    if family == "E":
        return bool(re.fullmatch(numeric, unit_id) or unit_id in {"E-cross", "E-program"})
    if family == "F":
        return bool(
            re.fullmatch(numeric, unit_id)
            or unit_id in {"F-global", "F-WD", "F-registers"}
        )
    if family == "G":
        return bool(re.fullmatch(numeric, unit_id) or unit_id == "G-registers")
    if family == "H":
        return unit_id.startswith("H-")
    if family == "I":
        return bool(
            re.fullmatch(numeric, unit_id)
            or unit_id in {"I-GROUNDING", "I-STEAL", "CROSS", "RELATIONAL", "IX", "SD", "misc"}
        )
    if family == "K":
        return bool(
            re.fullmatch(numeric, unit_id)
            or unit_id in {"KGM", "KMB", "K-X", "FIX", "registers"}
        )
    if family == "L":
        return bool(
            re.fullmatch(numeric, unit_id)
            or unit_id in {"L-CGG", "L-OQF", "registers"}
        )
    if family == "M":
        return bool(
            re.fullmatch(numeric, unit_id)
            or unit_id in {"M-registers", "M-summary-wide"}
        )
    if family == "N":
        return bool(
            re.fullmatch(numeric, unit_id)
            or unit_id in {"N-registers", "N-summary-wide"}
        )
    if family == "O":
        return bool(
            re.fullmatch(numeric, unit_id)
            or unit_id in {"O-registers", "O-summary-wide"}
        )
    if family == "Q":
        return bool(re.fullmatch(numeric, unit_id) or unit_id == "QG")
    return False


def excluded_reason(label: str) -> str:
    lower = label.lower()
    if lower.startswith("user-facing surfaces"):
        return "surface inventory, not a plan unit"
    if lower.startswith("audit"):
        return "audit block, not a plan unit"
    if "register" in lower or lower.startswith("relational"):
        return "relational/register material, not a plan unit"
    if "superseded" in lower or "family-level" in lower:
        return "historical register material, not a plan unit"
    if lower.startswith("repair addendum"):
        return "repair/addendum material, not a plan unit"
    return "supporting heading outside the unit grammar"


def parse_exhaustive(path: Path) -> tuple[list[Heading], list[ExcludedHeading], list[str], int, int]:
    headings: list[Heading] = []
    excluded: list[ExcludedHeading] = []
    unparseable: list[str] = []
    raw_level3_headings = 0
    outside_system_level3_headings = 0
    family: Optional[str] = None
    try:
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError as exc:
        raise RuntimeError(f"cannot read exhaustive list: {path}: {exc}") from exc

    for line_number, line in enumerate(lines, start=1):
        system_match = SYSTEM_RE.match(line)
        if system_match:
            family = system_match.group(1)
            continue
        if line.startswith("## "):
            family = None
            continue
        match = HEADING_RE.match(line)
        if not match or match.group(1) != "###":
            continue
        raw_level3_headings += 1
        if family is None:
            outside_system_level3_headings += 1
            continue
        raw = match.group(2)
        unit_id, title = split_heading(raw)
        if is_catalog_heading(family, unit_id):
            headings.append(Heading(family, unit_id, title, line_number))
        else:
            excluded.append(ExcludedHeading(family, raw, line_number, excluded_reason(raw)))
            if re.match(r"^[A-Za-z0-9]+(?:-[A-Za-z0-9+]+)+$", unit_id):
                unparseable.append(f"{family}:{unit_id} at exhaustive line {line_number}")

    return headings, excluded, unparseable, raw_level3_headings, outside_system_level3_headings


def first_nonempty_line(path: Path) -> tuple[str, int, bool]:
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    for line_number, line in enumerate(lines, start=1):
        stripped = line.strip()
        if not stripped:
            continue
        heading_match = HEADING_RE.match(stripped)
        if heading_match:
            return heading_match.group(2).strip(), line_number, True
        return stripped, line_number, False
    return "", 0, False


def phase_source_id(path: Path) -> Optional[str]:
    stem = path.stem
    if stem == "pre-phase-0":
        return stem
    if stem.startswith("phase-"):
        return stem
    return None


def plan_source_id(path: Path, family: str) -> Optional[str]:
    match = SOURCE_ID_RE.match(path.name)
    if not match or match.group(1)[0] != family:
        return None
    return match.group(1)


def load_source_docs(repo_root: Path) -> tuple[list[SourceDoc], list[str]]:
    docs: list[SourceDoc] = []
    issues: list[str] = []
    phase_dir = repo_root / "docs/specs/guide/steps"
    for path in sorted(phase_dir.glob("*.txt")):
        source_id = phase_source_id(path)
        if source_id is None:
            continue
        try:
            title, first_line, has_heading = first_nonempty_line(path)
        except OSError as exc:
            issues.append(f"{repo_relative(path, repo_root)}: {exc}")
            continue
        if not title:
            issues.append(f"{repo_relative(path, repo_root)}: no non-empty title line")
        docs.append(
            SourceDoc(
                "PHASE",
                source_id,
                path,
                title,
                first_line,
                source_id.startswith("phase-index-"),
                has_heading,
            )
        )

    for family in LETTER_FAMILIES:
        plan_dir = repo_root / f"docs/thefinalglance/{family}-plan"
        for path in sorted(plan_dir.glob("*.md")):
            source_id = plan_source_id(path, family)
            if source_id is None:
                continue
            try:
                title, first_line, has_heading = first_nonempty_line(path)
            except OSError as exc:
                issues.append(f"{repo_relative(path, repo_root)}: {exc}")
                continue
            if not title:
                issues.append(f"{repo_relative(path, repo_root)}: no non-empty title line")
            docs.append(
                SourceDoc(
                    family,
                    source_id,
                    path,
                    title,
                    first_line,
                    source_id.endswith("-0") or source_id.endswith("-0_INDEX"),
                    has_heading,
                )
            )
    return docs, issues


def source_keys(doc: SourceDoc) -> list[str]:
    """Return exhaustive IDs that a source document can legitimately satisfy."""

    if doc.family == "PHASE":
        if doc.source_id == "phase-index-pre0-2D":
            return ["IDX"]
        if doc.source_id.startswith("phase-"):
            return [doc.source_id, doc.source_id[len("phase-") :]]
        return [doc.source_id]
    if doc.source_id == "H-28-WIRE-1":
        return [doc.source_id, "H-28W"]
    return [doc.source_id]


def display_source_title(doc: SourceDoc) -> str:
    title = doc.title
    if doc.family == "PHASE" and title.upper().startswith("PHASE INDEX"):
        return concise_phrase(title)
    return concise_phrase(title)


def summary_paths(repo_root: Path, family: str) -> list[Path]:
    plan_dir = repo_root / f"docs/thefinalglance/{family}-plan"
    paths = sorted(plan_dir.glob("*-0_INDEX.md")) + sorted(plan_dir.glob("*-SUMMARY_INDEX.md"))
    if family == "B":
        paths.append(plan_dir / "B-PLAN-SUMMARY.md")
    return sorted(set(paths))


def summary_heading_ids(text: str, family: str) -> set[str]:
    ids: set[str] = set()
    pattern = re.compile(
        rf"^\s*(?:#{{1,6}}\s+|[-*]\s+\[)({re.escape(family)}-\d+(?:\.\d+)?[a-z]?(?:-[A-Z]+-\d+)?)(?:\b|\])",
        re.MULTILINE,
    )
    for match in pattern.finditer(text):
        ids.add(match.group(1))
    return ids


def token_present(text: str, token: str) -> bool:
    return bool(re.search(rf"(?<![A-Za-z0-9]){re.escape(token)}(?![A-Za-z0-9])", text))


def summary_coverage(
    repo_root: Path,
    source_docs: list[SourceDoc],
    primary_by_key: dict[tuple[str, str], Heading],
) -> tuple[
    dict[str, list[Path]],
    dict[str, list[str]],
    dict[str, list[str]],
    dict[str, tuple[int, int]],
    dict[str, tuple[int, int]],
]:
    files: dict[str, list[Path]] = {}
    missing_mentions: dict[str, list[str]] = {}
    missing_entries: dict[str, list[str]] = {}
    mention_counts: dict[str, tuple[int, int]] = {}
    entry_counts: dict[str, tuple[int, int]] = {}

    for family in LETTER_FAMILIES:
        paths = summary_paths(repo_root, family)
        files[family] = paths
        text = "\n".join(path.read_text(encoding="utf-8", errors="replace") for path in paths)
        heading_ids = summary_heading_ids(text, family)
        detail_docs = [doc for doc in source_docs if doc.family == family and not doc.is_index]
        missing_m = []
        missing_e = []
        mention_hit = 0
        entry_hit = 0
        for doc in detail_docs:
            primary_ids = [key for key in source_keys(doc) if (family, key) in primary_by_key]
            tokens = [doc.source_id] + primary_ids
            has_mention = any(token_present(text, token) for token in tokens)
            has_entry = any(token in heading_ids for token in tokens)
            label = doc.source_id
            if has_mention:
                mention_hit += 1
            else:
                missing_m.append(label)
            if has_entry:
                entry_hit += 1
            else:
                missing_e.append(label)
        mention_counts[family] = (mention_hit, len(detail_docs))
        entry_counts[family] = (entry_hit, len(detail_docs))
        missing_mentions[family] = missing_m
        missing_entries[family] = missing_e
    return files, missing_mentions, missing_entries, mention_counts, entry_counts


def canonical_primary_id(doc: SourceDoc, primary_by_key: dict[tuple[str, str], Heading]) -> Optional[str]:
    matches = [key for key in source_keys(doc) if (doc.family, key) in primary_by_key]
    return matches[0] if matches else None


def is_companion_entry(family: str, unit_id: str) -> bool:
    """Identify headings counted as supporting entries by the exhaustive audit."""

    companion_ids = {
        "B": {"B-wide", "B-cross-cutting", "B-registers"},
        "E": {"E-cross", "E-program"},
        "F": {"F-global", "F-WD", "F-registers"},
        "G": {"G-registers"},
        "I": {"I-GROUNDING", "I-STEAL", "CROSS", "RELATIONAL", "IX", "SD", "misc"},
        "K": {"KGM", "KMB", "K-X", "FIX", "registers"},
        "L": {"L-CGG", "L-OQF", "registers"},
        "M": {"M-registers", "M-summary-wide"},
        "N": {"N-registers", "N-summary-wide"},
        "O": {"O-registers", "O-summary-wide"},
    }
    if family == "H":
        return unit_id != "H-28W" and not re.fullmatch(r"H-\d+(?:\.\d+)?[a-z]?", unit_id)
    if family == "Q" and unit_id == "QG":
        return True
    return unit_id in companion_ids.get(family, set())


def numeric_sequence_gaps(
    family: str, source_docs: list[SourceDoc], primary_by_key: dict[tuple[str, str], Heading]
) -> list[str]:
    """Report missing integer IDs where a family has a clear 0..N sequence."""

    ids = {
        int(match.group(1))
        for doc in source_docs
        if doc.family == family
        for match in [re.fullmatch(rf"{re.escape(family)}-(\d+)", doc.source_id)]
        if match
    }
    if not ids or max(ids) < 3:
        return []
    gaps = [f"{family}-{number}" for number in range(min(ids), max(ids) + 1) if number not in ids]
    return gaps


def companion_source(repo_root: Path, family: str, unit_id: str) -> Optional[Path]:
    """Link grounding/summary companions when a concrete supporting file exists."""

    candidates = {
        ("B", "B-wide"): "docs/thefinalglance/B-plan/B-0_INDEX.md",
        ("B", "B-cross-cutting"): "docs/thefinalglance/B-plan/B-PLAN-SUMMARY.md",
        ("B", "B-registers"): "docs/thefinalglance/B-plan/B-0_INDEX.md",
        ("E", "E-cross"): "docs/thefinalglance/E-plan/E-SUMMARY_INDEX.md",
        ("E", "E-program"): "docs/thefinalglance/E-plan/E-SUMMARY_INDEX.md",
        ("F", "F-global"): "docs/thefinalglance/F-plan/F-0_INDEX.md",
        ("F", "F-WD"): "docs/thefinalglance/F-plan/F-WIRING-DOCTRINE-WHY.md",
        ("F", "F-registers"): "docs/thefinalglance/F-plan/F-SUMMARY_INDEX.md",
        ("G", "G-registers"): "docs/thefinalglance/G-plan/G-SUMMARY_INDEX.md",
        ("H", "H-cross"): "docs/thefinalglance/H-plan/H-SUMMARY_INDEX.md",
        ("I", "I-GROUNDING"): "docs/thefinalglance/I-plan/_grounding/I-GROUNDING-MAP.md",
        ("I", "I-STEAL"): "docs/thefinalglance/I-plan/_grounding/I-STEAL-GROUNDING-MAP.md",
        ("I", "CROSS"): "docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md",
        ("I", "RELATIONAL"): "docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md",
        ("I", "IX"): "docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md",
        ("I", "SD"): "docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md",
        ("I", "misc"): "docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md",
        ("K", "K-X"): "docs/thefinalglance/K-plan/K-SUMMARY_INDEX.md",
        ("K", "KGM"): "docs/thefinalglance/K-plan/_grounding/K-GROUNDING-MAP.md",
        ("K", "KMB"): "docs/thefinalglance/K-plan/K-MASTER-BRIEF-2026-06-12.md",
        ("K", "FIX"): "docs/thefinalglance/K-plan/K-0_INDEX.md",
        ("K", "registers"): "docs/thefinalglance/K-plan/K-SUMMARY_INDEX.md",
        ("L", "L-CGG"): "docs/thefinalglance/L-plan/_grounding/L-CGG-GROUNDING-MAP.md",
        ("L", "L-OQF"): "docs/thefinalglance/L-plan/_grounding/L-OQF-GROUNDING-MAP.md",
        ("L", "registers"): "docs/thefinalglance/L-plan/L-SUMMARY_INDEX.md",
        ("M", "M-registers"): "docs/thefinalglance/M-plan/M-SUMMARY_INDEX.md",
        ("M", "M-summary-wide"): "docs/thefinalglance/M-plan/M-SUMMARY_INDEX.md",
        ("N", "N-registers"): "docs/thefinalglance/N-plan/N-SUMMARY_INDEX.md",
        ("N", "N-summary-wide"): "docs/thefinalglance/N-plan/N-SUMMARY_INDEX.md",
        ("O", "O-registers"): "docs/thefinalglance/O-plan/O-SUMMARY_INDEX.md",
        ("O", "O-summary-wide"): "docs/thefinalglance/O-plan/O-SUMMARY_INDEX.md",
        ("Q", "QG"): "docs/thefinalglance/Q-plan/_grounding/Q-RESEARCH-CREATIVITY-GROUNDING-MAP.md",
    }
    path = repo_root / candidates.get((family, unit_id), "")
    return path if path.is_file() else None


def build_data(repo_root: Path) -> CatalogData:
    exhaustive_path = repo_root / EXHAUSTIVE_RELATIVE
    (
        primary_headings,
        excluded,
        parse_issues,
        raw_level3_headings,
        outside_system_level3_headings,
    ) = parse_exhaustive(exhaustive_path)
    source_docs, source_issues = load_source_docs(repo_root)
    unparseable = parse_issues + source_issues
    plain_titles = [
        f"{doc.source_id} ({repo_relative(doc.path, repo_root)})"
        for doc in source_docs
        if not doc.has_markdown_heading
    ]

    primary_by_key: dict[tuple[str, str], Heading] = {}
    duplicate_primary: list[str] = []
    for heading in primary_headings:
        key = (heading.family, heading.unit_id)
        if key in primary_by_key:
            duplicate_primary.append(f"{heading.family}:{heading.unit_id}")
        else:
            primary_by_key[key] = heading

    mapped_sources: dict[tuple[str, str], list[SourceDoc]] = defaultdict(list)
    aliases: list[tuple[str, str, str]] = []
    for doc in source_docs:
        for key in source_keys(doc):
            if (doc.family, key) not in primary_by_key:
                continue
            mapped_sources[(doc.family, key)].append(doc)
            if key != doc.source_id:
                aliases.append((doc.family, doc.source_id, key))

    duplicate_source_mappings = [
        f"{family}:{unit_id} <- {', '.join(doc.source_id for doc in docs)}"
        for (family, unit_id), docs in sorted(mapped_sources.items())
        if len(docs) > 1
    ]

    entries: list[CatalogEntry] = []
    for heading in primary_headings:
        source_matches = mapped_sources.get((heading.family, heading.unit_id), [])
        phrase = concise_phrase(heading.raw_title)
        entries.append(
            CatalogEntry(
                family=heading.family,
                unit_id=heading.unit_id,
                phrase=phrase,
                exhaustive_id=heading.unit_id,
                exhaustive_line=heading.line,
                original_docs=list(source_matches),
                kind="companion" if is_companion_entry(heading.family, heading.unit_id) else "unit",
            )
        )

    source_only_docs: list[SourceDoc] = []
    structural_source_docs: list[SourceDoc] = []
    for doc in source_docs:
        if canonical_primary_id(doc, primary_by_key) is not None:
            continue
        if doc.is_index:
            structural_source_docs.append(doc)
        else:
            source_only_docs.append(doc)
            phrase = display_source_title(doc)
            entries.append(
                CatalogEntry(
                    family=doc.family,
                    unit_id=doc.source_id,
                    phrase=phrase,
                    original_docs=[doc],
                    source_only=True,
                    notes=["added from original plan because no matching exhaustive heading was found"],
                )
            )

    entries.sort(key=lambda entry: (FAMILIES.index(entry.family), entry.exhaustive_line is None, entry.exhaustive_line or 0, entry.unit_id))

    summary_files, missing_mentions, missing_entries, mention_counts, entry_counts = summary_coverage(
        repo_root, source_docs, primary_by_key
    )
    sequence_gaps = {
        family: numeric_sequence_gaps(family, source_docs, primary_by_key)
        for family in LETTER_FAMILIES
    }
    sequence_gaps = {family: gaps for family, gaps in sequence_gaps.items() if gaps}

    return CatalogData(
        repo_root=repo_root,
        exhaustive_path=exhaustive_path,
        entries=entries,
        primary_headings=primary_headings,
        excluded_headings=excluded,
        source_docs=source_docs,
        source_only_docs=source_only_docs,
        structural_source_docs=structural_source_docs,
        aliases=sorted(set(aliases)),
        duplicate_primary_ids=sorted(set(duplicate_primary)),
        duplicate_source_mappings=sorted(set(duplicate_source_mappings)),
        unparseable_records=sorted(set(unparseable)),
        plain_title_records=sorted(set(plain_titles)),
        summary_files=summary_files,
        summary_missing_mentions=missing_mentions,
        summary_missing_entries=missing_entries,
        summary_mention_counts=mention_counts,
        summary_entry_counts=entry_counts,
        sequence_gaps=sequence_gaps,
        raw_level3_headings=raw_level3_headings,
        outside_system_level3_headings=outside_system_level3_headings,
    )


def validate_data(data: CatalogData) -> None:
    keys = [(entry.family, entry.unit_id) for entry in data.entries]
    if len(keys) != len(set(keys)):
        raise RuntimeError("catalog contains duplicate family/unit IDs")
    if data.duplicate_primary_ids:
        raise RuntimeError("duplicate primary exhaustive headings: " + ", ".join(data.duplicate_primary_ids))

    for entry in data.entries:
        if not entry.phrase:
            raise RuntimeError(f"empty phrase for {entry.family}:{entry.unit_id}")
        if ABSOLUTE_PATH_RE.search(entry.phrase):
            raise RuntimeError(f"absolute path leaked into phrase {entry.family}:{entry.unit_id}")
        for doc in entry.original_docs:
            if not doc.path.is_file():
                raise RuntimeError(f"missing source document for {entry.family}:{entry.unit_id}: {doc.path}")
    for doc in data.structural_source_docs:
        if not doc.path.is_file():
            raise RuntimeError(f"missing structural source document: {doc.path}")


def source_link(
    data: CatalogData,
    target: Path,
    label: str,
    virtual_from: Path,
    fragment: str = "",
) -> str:
    relative = link_path(data.repo_root, target, virtual_from)
    if fragment:
        relative = f"{relative}{fragment}"
    return f"[{label}]({relative})"


def entry_links(data: CatalogData, entry: CatalogEntry, virtual_from: Path) -> str:
    links: list[str] = []
    if entry.exhaustive_line is not None:
        links.append(
            source_link(
                data,
                data.exhaustive_path,
                f"exhaustive:L{entry.exhaustive_line}",
                virtual_from,
                f"#L{entry.exhaustive_line}",
            )
        )
    for doc in entry.original_docs:
        links.append(
            source_link(
                data,
                doc.path,
                f"original:{doc.source_id}",
                virtual_from,
            )
        )
    if entry.exhaustive_line is not None and not entry.original_docs:
        companion = companion_source(data.repo_root, entry.family, entry.unit_id)
        if companion is not None:
            links.append(source_link(data, companion, "supporting", virtual_from))
    if not links:
        links.append("source link unavailable")
    return "; ".join(links)


def family_entries(data: CatalogData, family: str) -> list[CatalogEntry]:
    return [entry for entry in data.entries if entry.family == family]


def family_primary_count(data: CatalogData, family: str) -> int:
    return sum(1 for heading in data.primary_headings if heading.family == family)


def family_source_count(data: CatalogData, family: str) -> int:
    return sum(1 for doc in data.source_docs if doc.family == family)


def family_original_link_count(data: CatalogData, family: str) -> int:
    return sum(1 for entry in family_entries(data, family) if entry.original_docs)


def family_exhaustive_only_count(data: CatalogData, family: str) -> int:
    return sum(1 for entry in family_entries(data, family) if not entry.original_docs)


def family_source_only_count(data: CatalogData, family: str) -> int:
    return sum(1 for doc in data.source_only_docs if doc.family == family)


def summary_missing_text(values: list[str]) -> str:
    return ", ".join(f"`{value}`" for value in values) if values else "—"


def render_coverage_table(data: CatalogData) -> str:
    lines = [
        "| Family | Catalog entries | Exhaustive headings | Original plan/index files | Entries with original doc | Exhaustive-only entries | Original-only additions | Summary mentions | Dedicated summary entries |",
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for family in FAMILIES:
        mention_hit, mention_total = data.summary_mention_counts.get(family, (0, 0))
        entry_hit, entry_total = data.summary_entry_counts.get(family, (0, 0))
        lines.append(
            f"| {family} | {len(family_entries(data, family))} | {family_primary_count(data, family)} | "
            f"{family_source_count(data, family)} | {family_original_link_count(data, family)} | "
            f"{family_exhaustive_only_count(data, family)} | {family_source_only_count(data, family)} | "
            f"{mention_hit}/{mention_total if mention_total else '—'} | "
            f"{entry_hit}/{entry_total if entry_total else '—'} |"
        )
    lines.append(
        f"| **Total** | **{len(data.entries)}** | **{len(data.primary_headings)}** | **{len(data.source_docs)}** | "
        f"**{sum(1 for entry in data.entries if entry.original_docs)}** | "
        f"**{sum(1 for entry in data.entries if not entry.original_docs)}** | **{len(data.source_only_docs)}** | — | — |"
    )
    return "\n".join(lines)


def render_summary_gap_section(data: CatalogData) -> str:
    lines = [
        "## Summary-index reconciliation",
        "",
        "The summary figures below distinguish a token mention from a dedicated entry heading. Their denominator is the matched original detail-file set; exhaustive-only headings are tracked separately. A mention can be a cross-reference or a stale count; neither is treated as a complete summary entry.",
        "",
        "| Family | Missing any summary mention | Missing dedicated summary entry | Summary files inspected |",
        "|---|---|---|---|",
    ]
    for family in LETTER_FAMILIES:
        paths = data.summary_files.get(family, [])
        path_text = ", ".join(f"`{repo_relative(path, data.repo_root)}`" for path in paths)
        lines.append(
            f"| {family} | {summary_missing_text(data.summary_missing_mentions.get(family, []))} | "
            f"{summary_missing_text(data.summary_missing_entries.get(family, []))} | {path_text or '—'} |"
        )
    return "\n".join(lines)


def render_catalog(data: CatalogData) -> str:
    virtual_from = CATALOG_RELATIVE
    lines = [
        "# ELAI historical feature catalog",
        "",
        "This file is generated by `scripts/archive/build_feature_catalog.py`. It extracts one concise phrase from meaningful `###` headings in the exhaustive list, then reconciles those headings with the original PHASE and B–Q plan documents.",
        "",
        "The entries record historical intentions and research-plan units. They do not prove that a capability works, is wired, remains current, or was shipped. Status fields and behavior bullets are intentionally excluded so shipment claims are not copied into this catalog.",
        "",
        "## Source and parsing rules",
        "",
        f"- Primary inventory: `{repo_relative(data.exhaustive_path, data.repo_root)}`.",
        "- Original coverage: PHASE step files under `docs/specs/guide/steps/` and numeric B–Q plan files under `docs/thefinalglance/<family>-plan/`.",
        "- A feature phrase is the text after a recognized unit ID in a `###` heading, with bracketed labels and status-like suffixes removed, bounded to a deterministic length.",
        "- Family-wide companion entries are counted when the exhaustive document treats them as entries; user-surface inventories, audit blocks, and relational registers are retained in the audit section below rather than presented as plan units.",
        "- Links are repository-relative and point only to documents included in this archive; exhaustive references use GitHub line fragments such as `#L108`.",
        "",
        "## Coverage",
        "",
        render_coverage_table(data),
        "",
        f"The exhaustive list contains **{data.raw_level3_headings} level-three headings**. The parser recognizes **{len(data.primary_headings)} meaningful unit/entry headings** and retains **{len(data.excluded_headings)} supporting headings inside family sections** for audit; **{data.outside_system_level3_headings}** other level-three headings are outside family unit sections. The catalog contains **{len(data.entries)} records**, including **{sum(1 for entry in data.entries if not entry.original_docs)}** exhaustive-only records. {len(data.source_only_docs)} unmatched original feature plan(s) required an additional catalog line after alias normalization.",
        "",
        render_summary_gap_section(data),
        "",
        "## Family entries",
        "",
    ]

    for family in FAMILIES:
        entries = family_entries(data, family)
        lines.extend(
            [
                f"<details>",
                f"<summary>{family} — {len(entries)} entries</summary>",
                "",
            ]
        )
        for entry in entries:
            suffix = ""
            if entry.source_only:
                suffix = " — original-only addition"
            elif entry.kind == "companion":
                suffix = " — supporting entry"
            lines.append(
                f"- **{entry.unit_id}** — {entry.phrase}{suffix} ({entry_links(data, entry, virtual_from)})"
            )
        lines.extend(["", "</details>", ""])

    lines.extend(
        [
            "## Reconciliation and parser audit",
            "",
            "### Aliases and duplicates",
            "",
        ]
    )
    if data.aliases:
        lines.append("Source-to-exhaustive aliases are normalized so one historical unit is not counted twice:")
        lines.append("")
        for family, source_id, exhaustive_id in data.aliases:
            lines.append(f"- `{family}:{source_id}` → exhaustive `{family}:{exhaustive_id}`.")
    else:
        lines.append("- None.")
    lines.append("")
    if data.duplicate_primary_ids:
        lines.append("Duplicate exhaustive IDs:")
        lines.extend(f"- `{value}`" for value in data.duplicate_primary_ids)
    else:
        lines.append("- Duplicate exhaustive IDs: none.")
    if data.duplicate_source_mappings:
        lines.append("")
        lines.append("Source mappings with more than one document:")
        lines.extend(f"- `{value}`" for value in data.duplicate_source_mappings)
    lines.extend(["", "### Source-only and structural records", ""])
    if data.source_only_docs:
        lines.append("Original plan files without a matching exhaustive heading were added as original-only entries:")
        lines.extend(
            f"- `{doc.source_id}` — `{repo_relative(doc.path, data.repo_root)}`."
            for doc in data.source_only_docs
        )
    else:
        lines.append("- No unmatched original feature plan files.")
    if data.structural_source_docs:
        lines.append("")
        lines.append("Original index documents without a feature-unit heading in the exhaustive list:")
        lines.extend(
            f"- `{doc.source_id}` — `{repo_relative(doc.path, data.repo_root)}` (structural index, not counted as a missing feature)."
            for doc in data.structural_source_docs
        )
    exhaustive_only_entries = [entry for entry in data.entries if not entry.original_docs]
    lines.append("")
    if exhaustive_only_entries:
        lines.append("Meaningful exhaustive entries without a matched original plan file are retained with their exhaustive source link:")
        lines.extend(
            f"- `{entry.family}:{entry.unit_id}` — {entry.phrase} (exhaustive line {entry.exhaustive_line}; no matched original plan file)."
            for entry in exhaustive_only_entries
        )
    else:
        lines.append("- No exhaustive-only meaningful entries.")
    if data.sequence_gaps:
        lines.append("")
        lines.append("Numeric sequence gaps in original plan filenames:")
        for family, gaps in data.sequence_gaps.items():
            lines.append(f"- {family}: {', '.join(f'`{gap}`' for gap in gaps)}.")
    else:
        lines.append("")
        lines.append("- No numeric sequence gaps in the original plan filenames.")
    lines.extend(["", "### Plain-title and unparseable records", ""])
    if data.plain_title_records:
        lines.append("These source files have a usable first line but no Markdown heading marker:")
        lines.extend(f"- `{value}`." for value in data.plain_title_records)
    else:
        lines.append("- No plain-title source files.")
    if data.unparseable_records:
        lines.append("")
        lines.append("Unparseable records requiring review:")
        lines.extend(f"- `{value}`." for value in data.unparseable_records)
    else:
        lines.append("")
        lines.append("- Unparseable records: none.")
    lines.extend(["", "### Excluded exhaustive headings", ""])
    lines.append(
        "The following `###` headings were seen inside family sections but are excluded from the one-line unit list because they are surface inventories, audits, registers, addenda, or other supporting material. They remain visible here so the parser does not silently drop them."
    )
    lines.append("")
    lines.append("| Family | Heading | Exhaustive line | Reason |")
    lines.append("|---|---|---:|---|")
    for heading in data.excluded_headings:
        label = scrub_private_paths(heading.label).replace("|", "\\|")
        lines.append(
            f"| {heading.family} | {label} | {heading.line} | {heading.reason} ([source]("
            f"{link_path(data.repo_root, data.exhaustive_path, virtual_from)}#L{heading.line})) |"
        )
    lines.extend(
        [
            "",
            "## Completeness checks",
            "",
            "A future archive refresh should run the generator, then verify that: every recognized exhaustive ID is unique; every original feature file maps to an exhaustive ID or appears as an original-only addition; aliases are reviewed rather than silently merged; every entry has at least one existing source document; summary mention coverage and dedicated-entry coverage are both reviewed; numeric gaps such as `K-10` are explained; and the generated output is unchanged under `--check`.",
            "",
            "The generator performs those scoped checks before writing. It does not run project tests, infer runtime wiring, or validate implementation claims.",
            "",
        ]
    )
    return "\n".join(lines)


def render_readme_contents(data: CatalogData) -> str:
    virtual_from = README_RELATIVE
    lines = [
        "## Historical feature catalog",
        "",
        "This compact index records historical plan intentions and research units. It is not evidence that any capability works, is wired, remains current, or was shipped. The [canonical catalog](docs/FEATURE-CATALOG.md) contains provenance, reconciliation notes, and parser checks.",
        "",
    ]
    for family in FAMILIES:
        entries = family_entries(data, family)
        lines.extend([f"<details>", f"<summary>{family} — {len(entries)} entries</summary>", ""])
        for entry in entries:
            suffix = ""
            if entry.source_only:
                suffix = " — original-only addition"
            elif entry.kind == "companion":
                suffix = " — supporting entry"
            lines.append(
                f"- **{entry.unit_id}** — {entry.phrase}{suffix} ({entry_links(data, entry, virtual_from)})"
            )
        lines.extend(["", "</details>", ""])
    lines.extend(
        [
            f"Coverage: **{len(data.primary_headings)} meaningful exhaustive unit/entry headings** across PHASE and B–Q. The full catalog records aliases, source-only/structural records, summary-index gaps, excluded headings, and unparseable records.",
            "",
        ]
    )
    return "\n".join(lines)


def render_readme_block(data: CatalogData) -> str:
    return f"{README_START}\n{render_readme_contents(data).rstrip()}\n{README_END}\n"


def replace_readme_block(readme: str, block: str) -> str:
    start_positions = [match.start() for match in re.finditer(re.escape(README_START), readme)]
    end_positions = [match.start() for match in re.finditer(re.escape(README_END), readme)]
    if len(start_positions) != 1 or len(end_positions) != 1:
        raise RuntimeError("Overview must contain exactly one FEATURE-CATALOG start marker and one end marker")
    start = start_positions[0]
    end = end_positions[0]
    if end < start:
        raise RuntimeError("Overview feature catalog markers are out of order")
    block_end = end + len(README_END)
    return readme[:start] + block.rstrip("\n") + readme[block_end:]


def ensure_no_private_paths(text: str) -> None:
    if ABSOLUTE_PATH_RE.search(text):
        raise RuntimeError("generated output contains a personal or external absolute path")


def write_or_check(path: Path, content: str, check: bool) -> None:
    if check:
        if not path.is_file():
            raise RuntimeError(f"--check expected generated file: {path}")
        existing = path.read_text(encoding="utf-8", errors="replace")
        if existing != content:
            raise RuntimeError(f"generated output is stale: {path}")
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8", newline="\n")


def parse_args(argv: Optional[list[str]] = None) -> argparse.Namespace:
    script_root = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=script_root)
    parser.add_argument("--catalog", type=Path, default=None)
    parser.add_argument("--readme", type=Path, default=None, help="document containing the managed marker block (default: RESEARCH-OVERVIEW.md)")
    parser.add_argument(
        "--readme-snippet",
        type=Path,
        default=None,
        help="optional path for a standalone copy of the generated overview marker block",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="verify the catalog and managed overview block without writing; an optional snippet is checked only when supplied",
    )
    return parser.parse_args(argv)


def main(argv: Optional[list[str]] = None) -> int:
    args = parse_args(argv)
    repo_root = args.repo_root.resolve()
    catalog_path = (args.catalog or (repo_root / CATALOG_RELATIVE)).resolve()
    readme_path = (args.readme or (repo_root / README_RELATIVE)).resolve()

    data = build_data(repo_root)
    validate_data(data)
    catalog = render_catalog(data)
    readme_block = render_readme_block(data)
    ensure_no_private_paths(catalog)
    ensure_no_private_paths(readme_block)
    if not readme_path.is_file():
        raise RuntimeError(f"Overview not found: {readme_path}")
    readme_existing = readme_path.read_text(encoding="utf-8", errors="replace")
    readme_expected = replace_readme_block(readme_existing, readme_block)
    write_or_check(catalog_path, catalog, args.check)
    write_or_check(readme_path, readme_expected, args.check)
    if args.readme_snippet is not None:
        snippet_path = args.readme_snippet.resolve()
        write_or_check(snippet_path, readme_block, args.check)

    print(
        f"feature catalog: {len(data.entries)} entries; "
        f"{len(data.primary_headings)} exhaustive headings; "
        f"{len(data.source_only_docs)} original-only additions; "
        f"{len(data.excluded_headings)} excluded supporting headings"
    )
    print(f"catalog: {repo_relative(catalog_path, repo_root) if catalog_path.is_relative_to(repo_root) else 'external output'}")
    print(f"Overview: {repo_relative(readme_path, repo_root) if readme_path.is_relative_to(repo_root) else 'external output'}")
    if args.readme_snippet is not None:
        print(f"readme snippet: {repo_relative(snippet_path, repo_root) if snippet_path.is_relative_to(repo_root) else 'temporary output'}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RuntimeError as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)
